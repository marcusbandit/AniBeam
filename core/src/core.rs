use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};

use crate::contract::*;
use crate::events::{EventBus, Subscription};
use crate::images::{self, ImageCache};
use crate::jobs::Jobs;
use crate::library::reads;
use crate::library::scan::{self, LibraryState, ScanScope};
use crate::library::watcher::{self, Trigger, Watcher};
use crate::metadata::{airing, apply, automatch, OUTAGE_WINDOW};
use crate::net::anilist::AnilistClient;
use crate::net::aniskip::AniSkipClient;
use crate::net::jikan::JikanClient;
use crate::net::limiter::ProviderClient;
use crate::net::mal::MalClient;
use crate::net::{Http, ReqwestHttp, Upstream};
use crate::paths::CorePaths;
use crate::prefs;
use crate::store::Store;
use crate::trackers::secrets::{Secrets, KEYRING_UNAVAILABLE};

/// The core is one object. A shell opens it once, starts it once, subscribes
/// once, and from then on sends calls and receives events.
#[derive(uniffi::Object)]
pub struct Core {
    pub(crate) paths: CorePaths,
    pub(crate) store: Arc<Store>,
    pub(crate) bus: Arc<EventBus>,
    pub(crate) jobs: Arc<Jobs>,
    /// The poster, banner and portrait files under `<cache_dir>/images`,
    /// and the row per file that says what is there. Reads consult it,
    /// jobs fill it, and the sweep keeps it from growing without end.
    pub(crate) images: Arc<ImageCache>,
    /// The library's in-memory state: the movie folders each source's walk
    /// found, the watcher's queue of paths, and the settle timers. The core
    /// is already the `Arc`, so this is a plain field.
    pub(crate) library: LibraryState,
    /// Built by `start` and dropped first by `shutdown`. `None` before
    /// `start` and after `shutdown`, so a call that arrives on either side
    /// of the core's life finds nothing to watch with rather than failing.
    pub(crate) watcher: Mutex<Option<Watcher>>,
    /// Taken out and shut down exactly once, in `shutdown`. `None` after
    /// that: a plain `tokio::runtime::Runtime` panics if dropped from
    /// inside its own worker threads, so ownership lives behind a mutex
    /// rather than as a bare field, and `Drop` below hands it to
    /// `shutdown_background` if `shutdown` was never called.
    pub(crate) runtime: Mutex<Option<tokio::runtime::Runtime>>,
    /// Cloned from `runtime` at `open`, so later tasks can spawn work
    /// through it even while the runtime itself sits behind the mutex.
    pub(crate) handle: tokio::runtime::Handle,
    /// Where the tracker tokens live: the desktop keyring when there is
    /// one, `secrets.json` when there is not. Built by `open` without any
    /// I/O; which store it uses is settled on first use, warmed by `start`.
    pub(crate) secrets: Arc<Secrets>,
    /// One transport for every provider, so a test swaps the whole network
    /// out with one `FakeHttp`. The clients below each hold their own
    /// limiter over it.
    #[allow(dead_code)]
    pub(crate) http: Arc<dyn Http>,
    /// The provider clients. Tasks 16 onwards are the callers; the fields
    /// are built here so every job shares one limiter per upstream.
    #[allow(dead_code)]
    pub(crate) anilist: Arc<AnilistClient>,
    #[allow(dead_code)]
    pub(crate) jikan: Arc<JikanClient>,
    #[allow(dead_code)]
    pub(crate) aniskip: Arc<AniSkipClient>,
    #[allow(dead_code)]
    pub(crate) mal: Arc<MalClient>,
    /// When the core last said out loud that Jikan was not answering.
    /// Jikan is the episode-title side-fetch, so an outage costs a series
    /// its titles rather than its match, and a job walking a whole library
    /// through one must not write a warning per series into the log.
    pub(crate) jikan_outage: Mutex<Option<Instant>>,
    /// Jobs need an `Arc<Core>` of their own; exported methods take `&self`,
    /// so the core keeps a `Weak` to itself from `Arc::new_cyclic` and
    /// upgrades it.
    me: Weak<Core>,
    started: AtomicBool,
    closed: AtomicBool,
}

/// One client, one timeout, every provider. The tracker calls wrap their
/// own futures in a shorter `tokio::time::timeout` where a slow list read
/// should give up sooner than a slow image fetch.
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// The gap each upstream is paced to, from its published limit.
const ANILIST_GAP: Duration = Duration::from_millis(800);
const JIKAN_GAP: Duration = Duration::from_millis(1100);
const ANISKIP_GAP: Duration = Duration::from_millis(250);
const MAL_GAP: Duration = Duration::from_millis(500);

impl Core {
    /// `open` with the transport handed in, so a test drives every provider
    /// off canned replies. Not part of the contract and not exported.
    #[doc(hidden)]
    pub fn open_with_http(paths: CorePaths, http: Arc<dyn Http>) -> Result<Arc<Core>, CoreError> {
        let secrets = Secrets::init(paths.secrets_path());
        Core::open_with_http_and_secrets(paths, http, secrets)
    }

    /// `open` with the secrets facade handed in, so a caller that must not
    /// reach the machine's keyring, a test or a run rooted somewhere of its
    /// own, gets the file store alone. Not part of the contract and not
    /// exported.
    #[doc(hidden)]
    pub fn open_with_secrets(paths: CorePaths, secrets: Arc<Secrets>) -> Result<Arc<Core>, CoreError> {
        let http = Arc::new(ReqwestHttp::new(HTTP_TIMEOUT)?);
        Core::open_with_http_and_secrets(paths, http, secrets)
    }

    /// Both of the above at once, and where the core is actually built.
    /// Not part of the contract and not exported.
    #[doc(hidden)]
    pub fn open_with_http_and_secrets(paths: CorePaths, http: Arc<dyn Http>, secrets: Arc<Secrets>) -> Result<Arc<Core>, CoreError> {
        let store = Store::open(&paths.db_path())?;
        let bus = EventBus::new(store.clone())?;
        // Creating the cache directory is opening, the same as creating the
        // data directory the database file lives in.
        let images_dir = paths.images_dir();
        std::fs::create_dir_all(&images_dir).map_err(|e| CoreError::io_at(images_dir.to_string_lossy(), e))?;
        let images = ImageCache::new(store.clone(), images_dir, http.clone());
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("anibeam-core")
            .enable_all()
            .build()
            .map_err(|e| CoreError::internal(format!("runtime: {e}")))?;
        let handle = runtime.handle().clone();
        let jobs = Jobs::new(handle.clone(), bus.clone());
        let anilist = Arc::new(AnilistClient::new(ProviderClient::new(Upstream::Anilist, http.clone(), ANILIST_GAP)));
        let jikan = Arc::new(JikanClient::new(ProviderClient::new(Upstream::Jikan, http.clone(), JIKAN_GAP)));
        let aniskip = Arc::new(AniSkipClient::new(ProviderClient::new(Upstream::AniSkip, http.clone(), ANISKIP_GAP)));
        let mal = Arc::new(MalClient::new(ProviderClient::new(Upstream::Mal, http.clone(), MAL_GAP)));
        Ok(Arc::new_cyclic(|me| Core {
            paths,
            store,
            bus,
            jobs,
            images,
            library: LibraryState::default(),
            watcher: Mutex::new(None),
            runtime: Mutex::new(Some(runtime)),
            handle,
            secrets,
            http,
            anilist,
            jikan,
            aniskip,
            mal,
            jikan_outage: Mutex::new(None),
            me: me.clone(),
            started: AtomicBool::new(false),
            closed: AtomicBool::new(false),
        }))
    }

    /// `None` once the core is shutting down; callers treat that as "the
    /// core is going away" and end quietly rather than panicking.
    pub(crate) fn arc(&self) -> Option<Arc<Core>> {
        self.me.upgrade()
    }

    /// The store itself, for the integration tests' fixtures: they build
    /// library state with plain SQL rather than driving a scan. Not part of
    /// the contract and not exported to any shell.
    #[doc(hidden)]
    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    /// The secrets facade, with the store choice settled. `start` warms
    /// that choice on a blocking thread, so a call arriving after launch
    /// finds it made; a call that beats the warm-up makes it here, on the
    /// shell's calling thread, which is where keyring work belongs. Either
    /// way the "no keyring" line is written once, by whoever finished the
    /// probe.
    #[allow(dead_code)]
    pub(crate) fn secrets(&self) -> &Arc<Secrets> {
        if self.secrets.warm() {
            self.bus.info(Stage::System, KEYRING_UNAVAILABLE, EventBody::Notice);
        }
        &self.secrets
    }

    fn watcher(&self) -> MutexGuard<'_, Option<Watcher>> {
        self.watcher.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Installs the recursive watch on a source. Called by the Scan job,
    /// off the runtime's blocking pool, because notify installs one by
    /// walking every directory under the root. Nothing to do before `start`
    /// has built the watcher: `start` watches every source itself.
    pub(crate) fn install_watch(&self, path: &str) -> Result<(), CoreError> {
        match self.watcher().as_mut() {
            Some(watcher) => watcher.watch(path),
            None => Ok(()),
        }
    }

    pub(crate) fn unwatch_source(&self, path: &str) {
        if let Some(watcher) = self.watcher().as_mut() {
            watcher.unwatch(path);
        }
    }

    /// The watcher's whole reach into the core, called on notify's own
    /// thread: it queues the paths and asks for a scan, both of which are
    /// a lock and a spawn. Everything that touches the disk happens in the
    /// job that comes out of it.
    pub(crate) fn on_watch_triggers(self: &Arc<Self>, triggers: Vec<Trigger>) {
        if self.closed.load(Ordering::SeqCst) {
            return;
        }
        let mut paths: Vec<String> = Vec::new();
        let mut rescan = false;
        for trigger in triggers {
            match trigger {
                Trigger::Rescan => rescan = true,
                // A file speaks for the folder it is in: that is the series
                // the reconcile has to look at, whether the file arrived or
                // went away.
                Trigger::Ingest(p) | Trigger::Removed(p) => paths.push(watcher::parent_series_path(&p)),
                Trigger::NewDirectory(p) => paths.push(p),
            }
        }
        self.library.push_pending(paths);
        // A full scan covers every queued path, so the job takes them off
        // the queue itself rather than being asked for them twice.
        let scope = if rescan { ScanScope::All } else { ScanScope::Paths(Vec::new()) };
        scan::start(self, scope);
    }

    /// A folder has stopped changing, so it is worth a match. The entry
    /// goes first: the candidate query skips whatever is still armed, so a
    /// job already running picks this series up on its next time round the
    /// loop precisely because it no longer is.
    pub(crate) fn settle_fired(&self, series_id: u64) {
        self.library.settle.lock().unwrap_or_else(|e| e.into_inner()).remove(&series_id);
        let Some(core) = self.arc() else { return };
        automatch::start(&core);
    }

    /// Warns that Jikan is not answering, at most once every ten minutes.
    /// Every caller reports every failure; this is what decides which of
    /// them the user is told about.
    pub(crate) fn report_jikan_outage(&self, message: &str) {
        let mut last = self.jikan_outage.lock().unwrap_or_else(|e| e.into_inner());
        if last.is_some_and(|at| at.elapsed() < OUTAGE_WINDOW) {
            return;
        }
        *last = Some(Instant::now());
        // Dropped before the emit: a listener runs on this thread, and
        // nothing it does should be able to deadlock on this gate.
        drop(last);
        self.bus.warn(Stage::Metadata, format!("Jikan is not answering: {message}"), EventBody::Notice);
    }
}

#[uniffi::export]
impl Core {
    /// Opens and migrates the database, builds the runtime, the bus, the
    /// jobs registry and the provider clients. Nothing else, and nothing
    /// that touches the network.
    #[uniffi::constructor]
    pub fn open(paths: CorePaths) -> Result<Arc<Core>, CoreError> {
        // Building the client opens no socket, so this stays inside the
        // "nothing else" `open` promises.
        let http = Arc::new(ReqwestHttp::new(HTTP_TIMEOUT)?);
        Core::open_with_http(paths, http)
    }

    /// Watcher up, launch jobs queued. Task 31 fills this in. A second call
    /// is `Ok` and does nothing.
    pub fn start(&self) -> Result<(), CoreError> {
        if self.started.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        // Asking the Secret Service whether it exists is a D-Bus round
        // trip that can put a prompt on the screen, so it happens here
        // rather than in `open`, and off this thread: by the time a shell
        // asks which trackers are connected, the answer is waiting.
        let secrets = self.secrets.clone();
        let bus = self.bus.clone();
        self.handle.spawn_blocking(move || {
            if secrets.warm() {
                bus.info(Stage::System, KEYRING_UNAVAILABLE, EventBody::Notice);
            }
        });
        // A machine at its inotify limit still has a working library, just
        // one that only changes when a scan is asked for, so a watcher that
        // cannot be built is a warning rather than a failure to start.
        match Watcher::new(self.me.clone()) {
            Ok(watcher) => {
                // In the field before the first watch goes on, so a source
                // added while this is running installs its own rather than
                // finding nothing there and waiting for the next scan.
                *self.watcher() = Some(watcher);
                for path in self.store.read(scan::available_source_paths)? {
                    if let Err(e) = self.install_watch(&path) {
                        self.bus.warn(Stage::Library, format!("cannot watch {path}: {e}"), EventBody::Notice);
                    }
                }
            }
            Err(e) => {
                self.bus.warn(Stage::Library, format!("the watcher could not start: {e}"), EventBody::Notice);
            }
        }
        self.bus.info(Stage::System, format!("AniBeam core {} ready", crate::VERSION), EventBody::Ready);
        Ok(())
    }

    /// Returns fast, always. Every call after `shutdown` fails the same
    /// way, without touching anything else.
    pub fn call(&self, call: Call) -> Result<Reply, CoreError> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(CoreError::Internal { message: "core is shut down".to_string() });
        }
        match call {
            Call::About => Ok(Reply::About {
                about: About {
                    version: crate::VERSION.to_string(),
                    data_dir: self.paths.data_dir.clone(),
                    config_dir: self.paths.config_dir.clone(),
                    cache_dir: self.paths.cache_dir.clone(),
                    db_path: self.paths.db_path().to_string_lossy().into_owned(),
                },
            }),
            Call::RecentEvents { limit } => Ok(Reply::Events { events: self.bus.recent(limit)? }),
            Call::ClearEvents => {
                self.bus.clear()?;
                Ok(Reply::Ok)
            }
            Call::ListJobs => Ok(Reply::Jobs { jobs: self.jobs.list() }),
            Call::CancelJob { job } => {
                self.jobs.cancel(job)?;
                Ok(Reply::Ok)
            }
            Call::ListSources => scan::list_sources_call(self),
            Call::AddSource { path } => scan::add_source(self, &path),
            Call::RemoveSource { source } => scan::remove_source(self, source),
            Call::ForgetSeries { series } => scan::forget_series(self, series),
            Call::Scan { source } => scan::scan(self, source),
            Call::RescanSeries { series } => scan::rescan_series(self, series),
            // `reveal_hidden` is the shell's tab visibility, not a filter:
            // the Hidden tab always lists what it holds.
            Call::ListSeries { tab, query, sort, direction, reveal_hidden: _ } => reads::list_series(self, tab, &query, sort, direction),
            Call::ListAiring { offset, limit } => reads::list_airing(self, offset, limit),
            Call::GetSeries { series } => reads::get_series(self, series),
            Call::SetHidden { series, hidden } => reads::set_hidden(self, series, hidden),
            Call::ListMetadata { filter, query, reveal_hidden } => reads::list_metadata(self, filter, &query, reveal_hidden),
            Call::Lookup { path } => reads::lookup(self, &path),
            // Two counts off one table, so this answers off the reader
            // connection rather than becoming a job.
            Call::GetStorage => {
                let (image_count, image_bytes) = self.store.read(|c| self.images.storage(c))?;
                Ok(Reply::Storage { image_count, image_bytes })
            }
            Call::ClearImages => {
                let core = self.arc().ok_or_else(|| CoreError::internal("core is shutting down"))?;
                Ok(Reply::Started { job: images::start_clear(&core) })
            }
            Call::SearchProvider { provider, query, limit } => Ok(Reply::Started { job: apply::search(self, provider, &query, limit)? }),
            Call::ResolveLink { url } => Ok(Reply::Started { job: apply::resolve_link(self, &url)? }),
            Call::ApplyMatch { series, target } => Ok(Reply::Started { job: apply::apply_match(self, series, target)? }),
            Call::RefreshSeries { series } => Ok(Reply::Started { job: apply::refresh_series(self, series)? }),
            Call::RefreshAll => {
                let core = self.arc().ok_or_else(|| CoreError::internal("core is shutting down"))?;
                Ok(Reply::Started { job: apply::refresh_all(&core) })
            }
            Call::AutoMatch => {
                let core = self.arc().ok_or_else(|| CoreError::internal("core is shutting down"))?;
                Ok(Reply::Started { job: automatch::start(&core) })
            }
            Call::RefreshAiring { series } => Ok(Reply::Started { job: airing::start_refresh(self, series)? }),
            Call::ClearMatch { series } => automatch::clear_match(self, series),
            Call::GetPreferences => Ok(Reply::Preferences { preferences: self.store.read(prefs::load_preferences)? }),
            Call::SetPreferences { preferences } => {
                let p = preferences.clone();
                self.store.write(move |c| prefs::save_preferences(c, &p))?;
                self.bus.debug(Stage::Store, "preferences changed", EventBody::PreferencesChanged { preferences });
                Ok(Reply::Ok)
            }
            Call::GetSettings => Ok(Reply::Settings { settings: self.store.read(prefs::load_settings)? }),
            Call::SetSubtitleDefaults { defaults } => {
                prefs::validate_subtitle_defaults(&defaults)?;
                self.store.write(move |c| prefs::save_subtitle_defaults(c, &defaults))?;
                self.bus.debug(Stage::Store, "subtitle defaults changed", EventBody::SettingsChanged);
                Ok(Reply::Ok)
            }
            Call::SetAutoSkip { intro, outro } => {
                self.store.write(move |c| prefs::save_auto_skip(c, &AutoSkip { intro, outro }))?;
                self.bus.debug(Stage::Store, "auto-skip changed", EventBody::SettingsChanged);
                Ok(Reply::Ok)
            }
            other => Err(CoreError::Unsupported { what: format!("{} is not built yet", call_name(&other)) }),
        }
    }

    pub fn subscribe(&self, listener: Arc<dyn EventListener>) -> Arc<Subscription> {
        Subscription::new(self.bus.clone(), listener)
    }

    /// Stops the watcher, cancels every job, checkpoints and closes the
    /// store. Idempotent: a second call does nothing.
    pub fn shutdown(&self) {
        if self.closed.swap(true, Ordering::SeqCst) {
            return;
        }
        // First, and taken out of the mutex before it is stopped: the
        // debouncer's thread dispatches into this core, so it has to be
        // gone before the runtime and the store are.
        let watcher = self.watcher().take();
        if let Some(mut watcher) = watcher {
            watcher.stop();
        }
        self.jobs.cancel_all();
        // Taken out of the mutex and dropped by this `let` before the
        // blocking shutdown call, rather than matched straight off the
        // lock expression: `if let Some(x) = mutex.lock().unwrap().take()`
        // keeps the guard alive for the whole `if let` body, which would
        // hold this lock for up to five seconds.
        let runtime = self.runtime.lock().unwrap().take();
        if let Some(runtime) = runtime {
            runtime.shutdown_timeout(Duration::from_secs(5));
        }
        self.store.checkpoint();
        self.store.close();
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        // A plain drop of a `Runtime` from inside a runtime context panics,
        // and a shell that forgets to call `shutdown` should still exit
        // cleanly; hand any runtime still present to the background path
        // instead. Taken out of the mutex before that call, for the same
        // reason `shutdown` above does.
        let runtime = self.runtime.lock().unwrap().take();
        if let Some(runtime) = runtime {
            runtime.shutdown_background();
        }
    }
}

/// The variant name of a call, for messages: the externally tagged JSON key.
pub fn call_name(call: &Call) -> String {
    match serde_json::to_value(call) {
        Ok(serde_json::Value::String(s)) => s,
        Ok(serde_json::Value::Object(m)) => m.keys().next().cloned().unwrap_or_default(),
        _ => "?".into(),
    }
}
