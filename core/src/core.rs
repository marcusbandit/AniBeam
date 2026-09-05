use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Duration;

use rusqlite::{params, OptionalExtension};

use crate::contract::*;
use crate::events::{EventBus, Subscription};
use crate::jobs::Jobs;
use crate::library::reads;
use crate::library::scan::{self, LibraryState, ScanScope};
use crate::library::watcher::{self, Trigger, Watcher};
use crate::paths::CorePaths;
use crate::prefs;
use crate::store::Store;

/// The core is one object. A shell opens it once, starts it once, subscribes
/// once, and from then on sends calls and receives events.
#[derive(uniffi::Object)]
pub struct Core {
    pub(crate) paths: CorePaths,
    pub(crate) store: Arc<Store>,
    pub(crate) bus: Arc<EventBus>,
    pub(crate) jobs: Arc<Jobs>,
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
    #[allow(dead_code)]
    pub(crate) handle: tokio::runtime::Handle,
    // Task 13 adds `http: Arc<dyn Http>` here.
    /// Jobs need an `Arc<Core>` of their own; exported methods take `&self`,
    /// so the core keeps a `Weak` to itself from `Arc::new_cyclic` and
    /// upgrades it.
    me: Weak<Core>,
    started: AtomicBool,
    closed: AtomicBool,
}

impl Core {
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

    /// A folder has stopped changing. Task 16 starts the auto-match here;
    /// until then the timer is only made visible, so the watcher's end of
    /// the chain can be seen working.
    pub(crate) fn settle_fired(&self, series_id: u64) {
        let path: Option<String> = self
            .store
            .read(|c| Ok(c.query_row("SELECT path FROM series WHERE id = ?1", params![series_id as i64], |r| r.get(0)).optional()?))
            .unwrap_or(None);
        if let Some(path) = path {
            self.bus.debug(Stage::Library, format!("folder settled: {path}"), EventBody::Notice);
        }
    }
}

#[uniffi::export]
impl Core {
    /// Opens and migrates the database, builds the runtime, the bus and the
    /// jobs registry. Nothing else.
    #[uniffi::constructor]
    pub fn open(paths: CorePaths) -> Result<Arc<Core>, CoreError> {
        let store = Store::open(&paths.db_path())?;
        let bus = EventBus::new(store.clone())?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("anibeam-core")
            .enable_all()
            .build()
            .map_err(|e| CoreError::internal(format!("runtime: {e}")))?;
        let handle = runtime.handle().clone();
        let jobs = Jobs::new(handle.clone(), bus.clone());
        Ok(Arc::new_cyclic(|me| Core {
            paths,
            store,
            bus,
            jobs,
            library: LibraryState::default(),
            watcher: Mutex::new(None),
            runtime: Mutex::new(Some(runtime)),
            handle,
            me: me.clone(),
            started: AtomicBool::new(false),
            closed: AtomicBool::new(false),
        }))
    }

    /// Watcher up, launch jobs queued. Task 31 fills this in. A second call
    /// is `Ok` and does nothing.
    pub fn start(&self) -> Result<(), CoreError> {
        if self.started.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
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
