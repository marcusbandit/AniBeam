use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use crate::act;
use crate::agent;
use crate::airing_now;
use crate::app;
use crate::cast;
use crate::contract::*;
use crate::downloads::{self, Downloads};
use crate::events::Subscription;
use crate::feed;
use crate::foundation::secrets::{KEYRING_UNAVAILABLE, Secrets};
use crate::foundation::services::Services;
use crate::franchise;
use crate::host::{self, Host};
use crate::images;
use crate::jobs::{started, started_or_ok};
use crate::launch;
use crate::library::scan;
use crate::library::watcher::Watcher;
use crate::library::{Library, delete, fixes, reads, report, schedule};
use crate::metadata::{Metadata, airing, apply, automatch};
use crate::net::{Http, Net, ReqwestHttp};
use crate::paths::CorePaths;
use crate::phone;
use crate::playback::session;
use crate::playback::{Playback, skip};
use crate::presence;
use crate::recommended;
use crate::releases::Releases;
use crate::repos;
use crate::room;
use crate::stats;
use crate::store::Store;
use crate::sync::recover;
use crate::trackers::{Trackers, accounts, cache, move_up, oauth, plan_to_watch, watching, writes};
use crate::transfer;
use crate::update;

/// The core is one object. A shell opens it once, starts it once, subscribes
/// once, and from then on sends calls and receives events.
///
/// It holds the shared plumbing and one struct per team (#854). Each team
/// holds the teams below it, and `call` hands every call to the team that
/// owns it.
#[derive(uniffi::Object)]
pub struct Core {
    /// The store, the bus, the jobs registry, the paths, the runtime
    /// handle, the transport and the secrets: what every team shares.
    pub(crate) services: Arc<Services>,
    pub(crate) net: Arc<Net>,
    pub(crate) library: Arc<Library>,
    pub(crate) metadata: Arc<Metadata>,
    pub(crate) trackers: Arc<Trackers>,
    pub(crate) releases: Arc<Releases>,
    pub(crate) downloads: Arc<Downloads>,
    pub(crate) playback: Arc<Playback>,
    /// The watch together room this core is in, if any (`Room*` calls),
    /// above the playback team whose session it drives.
    pub(crate) rooms: Arc<room::Rooms>,
    /// The tailnet host: the read-only API the phone app reads, served on
    /// this machine's Tailscale addresses while the preference is on.
    pub(crate) host: Arc<Host>,
    /// The launch's one-shot listener, held here so it lives as long as it
    /// is waiting and no longer: it takes itself out the moment the
    /// catch-up scan finishes, and `shutdown` takes it out if it never
    /// does. Dropping the handle unsubscribes.
    launch: Arc<launch::Slot>,
    /// Taken out and shut down exactly once, in `shutdown`. `None` after
    /// that: a plain `tokio::runtime::Runtime` panics if dropped from
    /// inside its own worker threads, so ownership lives behind a mutex
    /// rather than as a bare field, and `Drop` below hands it to
    /// `shutdown_background` if `shutdown` was never called.
    pub(crate) runtime: Mutex<Option<tokio::runtime::Runtime>>,
    started: AtomicBool,
    /// Shared with the library, whose watcher stops taking triggers the
    /// moment the core starts shutting down.
    closed: Arc<AtomicBool>,
}

/// One client, one timeout, every provider. This is the ceiling on a
/// single request; a tracker call caps itself tighter than an image fetch
/// through the limiter's own per-attempt timeout.
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// The line every start logs, around the version: "Anibeam core 2.1.0
/// ready". `earlier_starts` reads the version back out of it.
const READY_BEFORE: &str = "Anibeam core ";
const READY_AFTER: &str = " ready";

/// The version in a start's line. Builds before 2.5 logged the name with a capital B
/// (#1047), so the name matches in any case.
fn ready_version(message: &str) -> Option<&str> {
    let name = message.get(..READY_BEFORE.len())?;
    name.eq_ignore_ascii_case(READY_BEFORE)
        .then(|| message[READY_BEFORE.len()..].strip_suffix(READY_AFTER))
        .flatten()
}

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
    pub fn open_with_secrets(
        paths: CorePaths,
        secrets: Arc<Secrets>,
    ) -> Result<Arc<Core>, CoreError> {
        let http = Arc::new(ReqwestHttp::new(HTTP_TIMEOUT)?);
        Core::open_with_http_and_secrets(paths, http, secrets)
    }

    /// Both of the above at once, and where the core is actually built.
    /// Not part of the contract and not exported.
    #[doc(hidden)]
    pub fn open_with_http_and_secrets(
        paths: CorePaths,
        http: Arc<dyn Http>,
        secrets: Arc<Secrets>,
    ) -> Result<Arc<Core>, CoreError> {
        let (services, runtime) = Services::open(paths, http, secrets)?;
        // Each team over the ones below it, bottom up.
        let closed = Arc::new(AtomicBool::new(false));
        let net = Arc::new(Net::new(services.clone(), crate::library::cards::cards_for));
        let library = Arc::new(Library::new(services.clone(), net.clone(), closed.clone()));
        let metadata = Arc::new(Metadata::new(
            services.clone(),
            net.clone(),
            library.clone(),
        ));
        let trackers = Arc::new(Trackers::new(services.clone(), net.clone()));
        let releases = Arc::new(Releases::new(services.clone()));
        let downloads = Arc::new(Downloads::new(services.clone(), net.clone()));
        let playback = Arc::new(Playback::new(
            services.clone(),
            net.clone(),
            trackers.clone(),
            downloads.clone(),
        ));
        let rooms = Arc::new(room::Rooms::new(
            services.clone(),
            playback.clone(),
            downloads.clone(),
            library.clone(),
            metadata.clone(),
        ));
        let host = Arc::new(Host::new(
            services.clone(),
            net.clone(),
            library.clone(),
            downloads.clone(),
        ));
        // Nothing below names a team above it (#854). What a team asks of
        // the one above goes through a trait of its own, linked here; what it
        // only tells it is a signal on the bus, heard here.
        library
            .incoming
            .set(Arc::downgrade(&downloads) as Weak<dyn crate::library::Incoming>);
        releases
            .searcher
            .set(Arc::downgrade(&downloads) as Weak<dyn crate::releases::Searcher>);
        playback
            .follower
            .set(Arc::downgrade(&rooms) as Weak<dyn crate::playback::Follower>);
        playback
            .presence
            .rooms
            .set(Arc::downgrade(&rooms) as Weak<dyn crate::presence::JoinRooms>);
        metadata.listen();
        crate::downloads::watch::listen(&downloads);
        // Every SeriesChanged, from whichever job or scan, carries what is
        // still downloading, the same as a list read does
        let weak = Arc::downgrade(&downloads);
        services
            .bus
            .set_cards_hook(Arc::new(move |cards: &mut [SeriesCard]| {
                if let Some(downloads) = weak.upgrade() {
                    crate::downloads::overlay(&downloads, cards);
                }
            }));
        let core = Arc::new(Core {
            services,
            net,
            library,
            metadata,
            trackers,
            releases,
            downloads,
            playback,
            rooms,
            host,
            launch: Arc::new(launch::Slot::default()),
            runtime: Mutex::new(Some(runtime)),
            started: AtomicBool::new(false),
            closed,
        });
        core.host
            .front
            .set(Arc::downgrade(&core) as Weak<dyn host::Front>);
        // Here rather than in `launch`: a job that moves the schedule can
        // finish in a core that was never started, as the CLI's are.
        crate::calendar::listen(&core.metadata);
        // Before anything reads an installed endpoint: a kind saved under a site's name gets
        // its shape name (#602), in the installed endpoints and in the phone's saved sources.
        let renamed = crate::repos::rename_old_kinds(&core.services)
            .and_then(|n| Ok(n + crate::phone::rename_old_kinds(&core.services)?));
        match renamed {
            Ok(0) => {}
            Ok(renamed) => {
                core.services.bus.info(
                    Stage::Downloads,
                    format!("Gave {renamed} download sources the new name of their kind"),
                    EventBody::Notice,
                );
            }
            Err(e) => {
                core.services.bus.warn(
                    Stage::Downloads,
                    format!("could not give a download source the new name of its kind: {e}"),
                    EventBody::Notice,
                );
            }
        }
        Ok(core)
    }

    /// Where the tailnet host listens for this core alone, in place of the
    /// `ANIBEAM_HOST_BIND` env var, which every core in a test process would
    /// share. Takes effect the next time the host is switched on. Not part of
    /// the contract and not exported to any shell.
    #[doc(hidden)]
    pub fn set_host_bind(&self, bind: Option<String>) {
        self.host.set_bind(bind);
    }

    /// The tailnet host's router over this core's host team, for a test that
    /// drives it without a socket. Not part of the contract and not exported
    /// to any shell.
    #[doc(hidden)]
    pub fn host_router(&self) -> axum::Router {
        host::server::router(Arc::downgrade(&self.host))
    }

    /// The `tailscale status --json` document this core's host reads in place
    /// of asking the CLI, for a test. Not part of the contract.
    #[doc(hidden)]
    pub fn set_tailscale_status(&self, json: Option<Vec<u8>>) {
        self.host.set_status(json);
    }

    /// Keeps the tailnet host serving whatever "Share over Tailscale" says,
    /// for a shell that exists to serve other devices, or hands it back to
    /// the preference. Works before `start` too: the host then comes up with
    /// it. Not part of the contract.
    pub fn hold_host(&self, held: bool) {
        self.host.hold(held);
        if self.started.load(Ordering::SeqCst) {
            crate::host::sync(&self.host);
        }
    }

    /// Speaks Discord presence to `transport` instead of a Discord client,
    /// which also makes presence available in a build with no application
    /// id. Takes effect when the switch is first turned on. Not part of the
    /// contract and not exported to any shell.
    #[doc(hidden)]
    pub fn set_presence_transport(&self, transport: Box<dyn crate::presence::Transport>) {
        self.playback.presence.set_transport(transport);
    }

    /// Clears Discord presence after a pause of `wait` instead of
    /// `presence::PAUSE_CLEAR`. Not part of the contract.
    #[doc(hidden)]
    pub fn set_presence_pause_clear(&self, wait: std::time::Duration) {
        self.playback.presence.set_pause_clear(wait);
    }

    /// Looks at a quiet Discord connection after `wait` instead of
    /// `presence::CHECK`. Not part of the contract.
    #[doc(hidden)]
    pub fn set_presence_check(&self, wait: std::time::Duration) {
        self.playback.presence.set_check(wait);
    }

    /// Feeds Discord presence one tick of `session` directly, as a tick
    /// that lost the race with its session's close would arrive. Not part
    /// of the contract.
    #[doc(hidden)]
    pub fn presence_tick(
        &self,
        session: u64,
        activity: &crate::presence::Activity,
        position: f64,
        paused: bool,
    ) {
        crate::presence::tick(&self.playback, session, activity, position, paused);
    }

    /// Corrects a watch together player at most once per `gap` instead of once a
    /// second. Not part of the contract.
    #[doc(hidden)]
    pub fn set_room_correction_gap(&self, gap: std::time::Duration) {
        self.rooms.set_correction_gap(gap);
    }

    /// Drops the room's connection without a goodbye, as a lost network would.
    /// Not part of the contract.
    #[doc(hidden)]
    pub fn room_drop_connection(&self) {
        self.rooms.drop_connection();
    }

    /// Returns once every Discord presence send asked for so far has been
    /// handled. Not part of the contract.
    #[doc(hidden)]
    pub fn presence_flush(&self) {
        self.playback.presence.flush();
    }

    /// Moves the OAuth listener off the registered port, so two tests
    /// driving a flow at once do not fight over one socket. 0 lets the
    /// system pick a free port, and the redirect url names the one it got.
    /// Not part of the contract and not exported to any shell.
    #[doc(hidden)]
    pub fn set_oauth_port(&self, port: u16) {
        self.trackers.oauth_port.store(port, Ordering::SeqCst);
    }

    /// Shortens the recommender's own wait between two AniList requests, so
    /// a test of its whole read takes no minutes. Not part of the contract.
    #[doc(hidden)]
    pub fn set_recommender_gap(&self, gap: std::time::Duration) {
        self.metadata
            .recommender_gap_ms
            .store(gap.as_millis() as u64, Ordering::SeqCst);
    }

    /// Shortens how long Jikan is left alone after it failed, so a test can
    /// reach the reply that ends an outage without ten minutes' wait. Not
    /// part of the contract either.
    #[doc(hidden)]
    pub fn set_jikan_window(&self, window: std::time::Duration) {
        self.metadata
            .jikan_window_ms
            .store(window.as_millis() as u64, Ordering::SeqCst);
    }

    /// Shortens how long one Jikan episode read may spend asking for pages,
    /// so a test can reach a read cut short without thirty seconds of
    /// pages. Not part of the contract either.
    #[doc(hidden)]
    pub fn set_jikan_budget(&self, budget: std::time::Duration) {
        self.metadata.net.jikan.set_budget(budget);
    }

    /// Shortens how long the browser's callback is held for a sign-in to
    /// finish, so a test can reach the "still finishing" page without a
    /// minute's wait. Not part of the contract either.
    #[doc(hidden)]
    pub fn set_sign_in_hold(&self, hold: std::time::Duration) {
        self.trackers
            .sign_in_hold_ms
            .store(hold.as_millis() as u64, Ordering::SeqCst);
    }

    /// The version each earlier start logged, newest first, while the
    /// activity log still holds its line: `2.1.0`, `2.1.0.r6.gf1d3824`.
    /// Read it before `start`, which logs this run's. An empty list is a
    /// core that never started here, or a log that was cleared. What's new
    /// takes the version an install ran before it kept a record of its own
    /// from here (`changelog::version_of`).
    pub fn earlier_starts(&self) -> Vec<String> {
        self.services
            .store
            .write(|c| {
                let mut stmt = c.prepare(
                    "SELECT message FROM events WHERE body = '\"Ready\"' ORDER BY seq DESC",
                )?;
                let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
                Ok(rows
                    .filter_map(Result::ok)
                    .filter_map(|m| ready_version(&m).map(str::to_string))
                    .collect())
            })
            .unwrap_or_default()
    }

    /// Turns off, for this core's life, every change the downloads code
    /// makes on its own: pruning finished feeds, and the replace followups
    /// that trash old files and cancel or tidy torrents. A shell calls it
    /// when it runs a library copy against the owner's real anirss and
    /// qBittorrent, such as a drive script or a sandbox root.
    pub fn disable_unattended_changes(&self) {
        self.downloads.disable_unattended();
    }

    /// Lets a core under a root settle a fresh install on a qBittorrent it finds
    /// answering, which only the person's own install does (#637): the tests of that
    /// choice run under a root. Not part of the contract and not exported to any shell.
    #[doc(hidden)]
    pub fn let_sandbox_find_torrent_clients(&self) {
        self.downloads.let_sandbox_find_clients();
    }

    /// The store itself, for the integration tests' fixtures: they build
    /// library state with plain SQL rather than driving a scan. Not part of
    /// the contract and not exported to any shell.
    #[doc(hidden)]
    pub fn store(&self) -> &Arc<Store> {
        &self.services.store
    }

    /// The shared plumbing, for the integration tests that start a job
    /// straight from the module that owns it. Not part of the contract and
    /// not exported to any shell.
    #[doc(hidden)]
    pub fn services(&self) -> &Arc<Services> {
        &self.services
    }

    /// The metadata team, for the tests that start its jobs directly.
    #[doc(hidden)]
    pub fn metadata(&self) -> &Arc<Metadata> {
        &self.metadata
    }

    /// The host team, for the tests that drive its router and its keys.
    #[doc(hidden)]
    pub fn host(&self) -> &Arc<Host> {
        &self.host
    }

    /// The playback team, for the tests that report to a session directly.
    #[doc(hidden)]
    pub fn playback(&self) -> &Arc<Playback> {
        &self.playback
    }

    /// The downloads team, for the tests that open its backend directly.
    #[doc(hidden)]
    pub fn downloads(&self) -> &Arc<Downloads> {
        &self.downloads
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

    /// Watcher up, launch jobs queued. A second call is `Ok` and does
    /// nothing: the launch happens once per run of the core.
    pub fn start(&self) -> Result<(), CoreError> {
        if self.started.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        self.services.bus.info(
            Stage::System,
            format!("{READY_BEFORE}{}{READY_AFTER}", crate::VERSION),
            EventBody::Ready,
        );
        // Asking the Secret Service whether it exists is a D-Bus round
        // trip that can put a prompt on the screen, so it happens here
        // rather than in `open`, and off this thread: by the time a shell
        // asks which trackers are connected, the answer is waiting.
        let secrets = self.services.secrets.clone();
        let bus = self.services.bus.clone();
        self.services.handle.spawn_blocking(move || {
            if secrets.warm() {
                bus.info(Stage::System, KEYRING_UNAVAILABLE, EventBody::Notice);
            }
        });
        // A machine at its inotify limit still has a working library, just
        // one that only changes when a scan is asked for, so a watcher that
        // cannot be built is a warning rather than a failure to start.
        //
        // It goes into the field empty. Installing a watch means walking
        // every directory under a root, which is disk work at library
        // scale, so it belongs to the scan below and to every scan after
        // it rather than to this thread.
        match Watcher::new(Arc::downgrade(&self.library)) {
            Ok(watcher) => self.library.set_watcher(watcher),
            Err(e) => {
                self.services.bus.warn(
                    Stage::Library,
                    format!("the watcher could not start: {e}"),
                    EventBody::Notice,
                );
            }
        }
        // The host follows the stored preference from the first moment, so
        // a phone can reach a machine that was left sharing.
        crate::host::sync(&self.host);
        crate::presence::sync(&self.playback);
        // Off this thread: storing a secret can be a keyring round trip.
        let lifting = self.releases.clone();
        self.services.handle.spawn_blocking(move || {
            let lift = crate::repos::lift_stored_secrets(&lifting);
            #[cfg(feature = "phone")]
            let lifted = lift.is_ok();
            if let Err(e) = lift {
                lifting.services.bus.warn(
                    Stage::Downloads,
                    format!("could not move a source's secret to the secret store: {e}"),
                    EventBody::Notice,
                );
            }
            // After the lift: a subscription's secret is found among the stored ones.
            #[cfg(feature = "phone")]
            crate::phone::rekeep_rules(&lifting.services, lifted);
        });
        // A started core says when a followed episode airs (#212); the CLI's,
        // never started, leaves that to the window.
        crate::aired::listen(&self.metadata);
        launch::start(self, &self.launch)
    }

    /// A call from a device on the tailnet, answered as `profile` (#631): the
    /// owner's is [`Core::call`] itself. Anyone else's player writes their own
    /// history and resume points and touches only the sessions it opened. It
    /// marks nothing on a tracker, since the accounts here are the owner's
    /// (#632), and changes nothing everyone shares: a track choice is kept as
    /// it is and Not interested is refused.
    pub fn call_as(&self, profile: u64, call: Call) -> Result<Reply, CoreError> {
        if profile == crate::profiles::OWNER {
            return self.call(call);
        }
        if self.closed.load(Ordering::SeqCst) {
            return Err(CoreError::Internal {
                message: "core is shut down".to_string(),
            });
        }
        match &call {
            Call::OpenPlayback { file } => {
                return Ok(Reply::Playback {
                    session: Box::new(session::open_for(&self.playback, *file, profile)?),
                });
            }
            Call::Tick { session, .. }
            | Call::ClosePlayback { session, .. }
            | Call::ReportChapters { session, .. } => {
                // A session already gone goes on as it would for the owner: a
                // second close is quiet. Only another person's is not found.
                if self
                    .playback
                    .sessions
                    .profile_of(*session)
                    .is_some_and(|owner| owner != profile)
                {
                    return Err(CoreError::NotFound {
                        what: Entity::Session,
                        id: *session,
                    });
                }
            }
            Call::MarkEpisode { .. } => {
                return Err(CoreError::invalid(
                    "tracker",
                    crate::i18n::tr("core-core-guest-trackers", &[]),
                ));
            }
            Call::SetTrackChoice { .. } => return Ok(Reply::Ok),
            Call::SetNotInterested { .. } | Call::PruneNotInterested => {
                return Err(CoreError::invalid(
                    "not_interested",
                    crate::i18n::tr("core-core-guest-not-interested", &[]),
                ));
            }
            Call::SetShowListStatus { .. } | Call::SetShowScore { .. } => {
                return Err(CoreError::invalid(
                    "tracker",
                    crate::i18n::tr("core-core-guest-trackers", &[]),
                ));
            }
            Call::ListRecommended { .. } | Call::RefreshRecommended { .. } => {
                return Err(CoreError::invalid(
                    "recommended",
                    crate::i18n::tr("core-core-guest-recommended", &[]),
                ));
            }
            _ => {}
        }
        self.call(call)
    }

    /// Returns fast, except the calls that wait on the network, which a
    /// shell sends off its UI thread: among them the endpoint repo fetches,
    /// the phone's searches, `PhoneNetCheck` and `CheckPhoneUpdate` (#1369
    /// settles how each waits). Every call after `shutdown` fails the same way, without
    /// touching anything else.
    ///
    /// A router and nothing more: every arm is one call into the module that
    /// owns it, grouped by the team the call belongs to (#773, #854).
    pub fn call(&self, call: Call) -> Result<Reply, CoreError> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(CoreError::Internal {
                message: "core is shut down".to_string(),
            });
        }
        match call {
            // The front door: the core itself, its log, jobs and settings
            Call::About => app::about(&self.services),
            Call::RecentEvents { limit } => app::recent_events(&self.services, limit),
            Call::ClearEvents => app::clear_events(&self.services),
            Call::Resumed => {
                crate::aired::resumed(&self.metadata);
                Ok(Reply::Ok)
            }
            Call::ListJobs => app::list_jobs(&self.services),
            Call::CancelJob { job } => app::cancel_job(&self.services, job),
            Call::GetPreferences => app::preferences(&self.services),
            Call::SetPreferences { preferences } => app::set_preferences(self, preferences),
            Call::GetSettings => app::settings(&self.services),
            Call::SetSubtitleDefaults { defaults } => {
                app::set_subtitle_defaults(&self.services, defaults)
            }
            Call::SetAutoSkip { intro, outro } => app::set_auto_skip(&self.services, intro, outro),
            Call::SetProviderOrder { order } => app::set_provider_order(&self.services, order),
            Call::GetStats {
                year,
                utc_offset,
                month_starts,
            } => stats::get_call(&self.services, year, utc_offset, &month_starts),
            Call::ScheduleAgent => agent::available_call(),
            Call::AskScheduleAgent { series } => agent::start(self, series).map(started),
            Call::CheckPhoneUpdate { current, nightly } => {
                update::phone_call(&self.services, &current, nightly)
            }

            // Library: sources, scans, the library's reads and its files
            Call::ListSources => scan::list_sources_call(&self.library),
            Call::AddSource { path } => scan::add_source(&self.library, &path),
            Call::RemoveSource { source } => scan::remove_source(&self.library, source),
            Call::MoveSource { source, path } => scan::move_source(&self.library, source, &path),
            Call::ForgetSeries { series } => scan::forget_series(&self.services, series),
            Call::Scan { source } => scan::scan(&self.library, source),
            Call::RescanSeries { series } => scan::rescan_series(&self.library, series),
            Call::DeleteEpisodes { series, files } => {
                delete::delete_episodes(&self.library, series, files)
            }
            Call::DeleteWatched { finished_shows } => {
                delete::delete_watched(&self.library, finished_shows)
            }
            Call::FixFiles { series, fixes } => fixes::fix_files(&self.library, series, fixes),
            Call::ReportFix { series, files, app } => {
                report::report_fix(&self.services, series, files, app)
            }
            Call::TriageAgent => report::triage_agent(),
            // `reveal_hidden` is the shell's tab visibility, not a filter:
            // the Hidden tab always lists what it holds.
            Call::ListSeries {
                tab,
                query,
                sort,
                direction,
                reveal_hidden: _,
            } => reads::list_series(&self.library, tab, &query, sort, direction),
            Call::ListAiring { offset, limit } => reads::list_airing(&self.library, offset, limit),
            Call::ListLibrary {
                tab,
                query,
                sort,
                direction,
                reveal_hidden: _,
            } => reads::list_library(&self.library, tab, &query, sort, direction),
            Call::ListFeed { sort } => feed::list_call(&self.services, sort),
            Call::GetSeries { series } => reads::get_series(&self.library, series),
            Call::SetHidden { series, hidden } => reads::set_hidden(&self.services, series, hidden),
            Call::ActOnSeries { series, action } => act::act_on_series(self, series, action),
            Call::ParseSchedule { text } => Ok(schedule::parse_call(&text)),
            Call::SetScheduleNotes {
                series,
                season,
                notes,
            } => schedule::set_notes(&self.services, series, season, notes),
            Call::SetEpisodeCount { series, count } => {
                reads::set_episode_count(&self.services, series, count)
            }
            Call::ListMetadata {
                filter,
                query,
                reveal_hidden,
            } => reads::list_metadata(&self.services, filter, &query, reveal_hidden),
            Call::Lookup { path } => reads::lookup(&self.services, &path),

            // Net: the image cache
            Call::GetStorage => images::storage_call(&self.net),
            Call::ClearImages => Ok(started(images::start_clear(&self.net))),

            // Metadata: matching, airing, Airing now, Recommended, franchises
            Call::SearchProvider {
                provider,
                query,
                limit,
            } => apply::search(&self.metadata, provider, &query, limit).map(started),
            Call::ResolveLink { url } => apply::resolve_link(&self.metadata, &url).map(started),
            Call::ApplyMatch { series, target } => {
                apply::apply_match(&self.metadata, series, target).map(started)
            }
            Call::RefreshSeries { series } => {
                apply::refresh_series(&self.metadata, series).map(started)
            }
            Call::RefreshAll => Ok(started(apply::refresh_all(&self.metadata))),
            Call::AutoMatch => Ok(started(automatch::start(&self.metadata))),
            Call::RefreshAiring { series } => {
                airing::start_refresh(&self.metadata, series).map(started)
            }
            Call::ClearMatch { series } => automatch::clear_match(&self.metadata, series),
            Call::ListAiringNow { season } => airing_now::list_call(&self.metadata, season),
            Call::RefreshAiringNow { season, force } => {
                airing_now::refresh_call(&self.metadata, season, force)
            }
            Call::SetNotInterested {
                anilist_id,
                not_interested,
            } => airing_now::set_not_interested(&self.services, anilist_id, not_interested),
            Call::ListNotInterested => airing_now::list_not_interested(&self.services),
            Call::PruneNotInterested => airing_now::prune_and_list(&self.services),
            Call::PlanToWatch { anilist_id } => {
                plan_to_watch::plan_to_watch(&self.trackers, anilist_id)
            }
            Call::ListRecommended { scope } => recommended::list_call(&self.services, scope),
            Call::RefreshRecommended { force } => recommended::refresh_call(&self.metadata, force),
            Call::GetShowPage { anilist_id } => {
                recommended::page::get_call(&self.metadata, anilist_id)
            }
            Call::GetFranchiseGraph { series } => franchise::graph_call(&self.metadata, series),

            // Trackers: accounts, list writes, Watching and the list moves
            Call::GetTrackers => {
                accounts::state(&self.services).map(|state| Reply::Trackers { state })
            }
            Call::SetTrackerCredentials {
                tracker,
                client_id,
                client_secret,
            } => accounts::set_credentials(
                &self.services,
                tracker,
                &client_id,
                client_secret.as_deref(),
            ),
            Call::ConnectTracker { tracker } => {
                oauth::connect(&self.trackers, tracker).map(started)
            }
            Call::DisconnectTracker { tracker } => accounts::disconnect(&self.services, tracker),
            Call::SetMainTracker { tracker } => accounts::set_main(&self.services, tracker),
            Call::MarkEpisode { series, episode } => {
                writes::mark(&self.trackers, series, episode).map(started)
            }
            Call::SetProgress {
                series,
                progress,
                season,
            } => writes::set_progress(&self.trackers, series, progress, season).map(started_or_ok),
            Call::SetScore { series, score } => {
                writes::set_score(&self.trackers, series, score).map(started_or_ok)
            }
            Call::SetListStatus { series, status } => {
                writes::set_list_status(&self.trackers, series, status).map(started_or_ok)
            }
            Call::SetSeriesNotes { series, notes } => {
                writes::set_notes(&self.trackers, series, &notes).map(started)
            }
            Call::SetShowListStatus { anilist_id, status } => {
                writes::set_show_list_status(&self.trackers, anilist_id, status).map(started_or_ok)
            }
            Call::SetShowScore { anilist_id, score } => {
                writes::set_show_score(&self.trackers, anilist_id, score).map(started_or_ok)
            }
            Call::PreviewListMove { tracker } => move_up::preview_call(&self.services, tracker),
            Call::MoveLists { tracker, series } => {
                move_up::start(&self.trackers, tracker, series).map(started)
            }
            Call::RefreshProgress { tracker } => Ok(started(cache::start_refresh(
                &self.trackers,
                tracker,
                false,
            ))),
            Call::ListWatching => watching::list_call(&self.services),
            Call::ListPlanning => watching::planning_call(&self.services),
            Call::RefreshWatching { force } => watching::refresh_call(&self.trackers, force),

            // Downloads: the status is a read off the state, the rest are
            // jobs over anirss or the torrent client, except the list reads
            // the page polls, which answer through events
            Call::GetDownloads => {
                downloads::status(&self.downloads).map(|status| Reply::Downloads { status })
            }
            Call::ConnectDownloads => downloads::connect(&self.downloads).map(started),
            Call::SearchReleases { query, endpoint } => {
                downloads::search(&self.downloads, query, endpoint).map(started)
            }
            Call::RefineReleases {
                query,
                endpoint,
                items,
                op,
            } => downloads::refine(&self.downloads, query, endpoint, items, op).map(started),
            Call::PreviewSubscription {
                query,
                endpoint,
                items,
            } => downloads::preview(&self.downloads, query, endpoint, items).map(started),
            Call::Subscribe {
                query,
                endpoint,
                items,
                name,
                wire_query,
                series,
            } => downloads::subscribe(
                &self.downloads,
                query,
                endpoint,
                items,
                name,
                wire_query,
                series,
            )
            .map(started),
            Call::DownloadReleases { items, name } => {
                downloads::download(&self.downloads, items, name).map(started)
            }
            Call::DownloadReleasesWith {
                items,
                name,
                options,
            } => downloads::download_with(&self.downloads, items, name, options).map(started),
            Call::DownloadMovie { link, needs_video } => {
                downloads::movie(&self.downloads, link, needs_video).map(started)
            }
            Call::ReadMagnet { link } => Ok(Reply::Magnet {
                magnet: downloads::read_magnet(&link),
            }),
            Call::ListSubscriptionRules => downloads::list_rules(&self.downloads),
            Call::ListTorrents => downloads::list_torrents(&self.downloads),
            Call::ReadFeed { url, tag } => downloads::read_feed(&self.services, url, tag),
            Call::ReadRule { name } => downloads::read_rule(&self.downloads, name),
            Call::RepairSubscription { name } => {
                downloads::repair_subscription(&self.downloads, name).map(started)
            }
            Call::RemoveShow {
                title,
                rule,
                hashes,
                torrents,
                files,
                series,
            } => downloads::remove_show(
                &self.downloads,
                title,
                rule,
                hashes,
                torrents,
                files,
                series,
            )
            .map(started),
            Call::ControlTorrents { hashes, action } => {
                downloads::control(&self.downloads, hashes, action).map(started)
            }
            Call::SetRoll {
                id,
                ahead,
                on,
                delete_watched,
            } => downloads::rolling::set(&self.downloads, id, ahead, on, delete_watched),
            Call::RollMore { id, count } => {
                downloads::rolling::roll_more(&self.downloads, id, count)
            }
            Call::GetRollingSettings => downloads::rolling::settings_call(&self.services),
            Call::SetRollingSettings { rolling } => {
                downloads::rolling::set_settings(&self.services, rolling)
            }
            Call::SetDownloadSettings {
                settings,
                anirss_token,
                torrent_password,
            } => downloads::set_settings(
                &self.downloads,
                settings,
                anirss_token.as_deref(),
                torrent_password.as_deref(),
            ),
            Call::UseTorrentClient { id } => downloads::use_client(&self.downloads, id),
            Call::SetSaveFolder { folder } => downloads::set_save_folder(&self.downloads, folder),
            Call::TestTorrentClient => downloads::test_torrent_client(&self.downloads).map(started),
            Call::CheckEndpoints => downloads::check_endpoints(&self.downloads).map(started),
            Call::StartEndpoint { name } => {
                downloads::start_endpoint(&self.downloads, name).map(started)
            }
            Call::InspectRelease { link } => downloads::inspect_release(&self.services, link),

            // Releases: endpoint repos and the endpoints installed from them
            Call::ListRepoEndpoints => {
                repos::catalog(&self.releases).map(|catalog| Reply::EndpointCatalog { catalog })
            }
            Call::AddEndpointRepo { url } => repos::add_repo(&self.releases, url),
            Call::TrustEndpointRepo { url, key } => repos::trust_repo(&self.releases, url, key),
            Call::AttachEndpointRepo { url } => repos::attach_repo(&self.releases, url),
            Call::RemoveEndpointRepo { url } => repos::remove_repo(&self.releases, url),
            Call::RefreshEndpointRepos => repos::refresh(&self.releases),
            Call::InstallEndpoint {
                repo,
                id,
                credentials,
            } => repos::install(&self.releases, repo, id, credentials),
            Call::UpdateEndpoint {
                install,
                credentials,
            } => repos::update(&self.releases, install, credentials),
            Call::RemoveEndpoint { install } => repos::remove(&self.releases, install),
            Call::SetEndpointCredentials {
                install,
                credentials,
            } => repos::set_credentials(&self.releases, install, credentials),
            Call::SetEndpointEnabled { install, enabled } => {
                repos::set_enabled(&self.releases, install, enabled)
            }
            Call::AddCustomEndpoint {
                name,
                url,
                credentials,
            } => repos::add_custom(&self.releases, name, url, credentials),
            Call::EditCustomEndpoint { install, name, url } => {
                repos::edit_custom(&self.releases, install, name, url)
            }

            // Downloads on the phone: its own sources and torrent engine
            Call::PhoneSources => phone::sources_call(&self.services),
            Call::SetPhoneSources { sources } => phone::set_sources(&self.services, sources),
            Call::PhoneSearchSources => phone::search_sources_call(&self.services),
            Call::PhoneSearch {
                query,
                endpoint,
                fallback,
            } => phone::search(&self.services, query, endpoint, fallback),
            Call::PhoneFindShows { query, limit } => {
                phone::find_shows(&self.downloads, query, limit)
            }
            Call::PhoneRefine {
                query,
                endpoint,
                items,
                op,
            } => phone::refine(&self.services, query, endpoint, items, op),
            Call::PhoneDownload {
                items,
                name,
                limit_bytes,
                rolling,
            } => phone::download(&self.downloads, items, name, limit_bytes, rolling),
            Call::PhoneSubscribe {
                query,
                endpoint,
                items,
                name,
            } => phone::subscribe(&self.downloads, query, endpoint, items, name),
            Call::PhoneRules => phone::rules(&self.downloads),
            Call::PhoneUnsubscribe { names, files } => {
                phone::unsubscribe(&self.downloads, names, files)
            }
            Call::PhoneTorrents => phone::torrents(&self.downloads),
            Call::PhoneNetCheck => phone::check::run_call(&self.downloads),
            Call::PhoneControl { hashes, action } => {
                phone::control(&self.downloads, hashes, action)
            }
            Call::SetPhoneHeld { held } => phone::set_held(&self.downloads, held),

            // Host: the tailnet host, export and import
            Call::GetHost => host::status_call(&self.host),
            Call::ListTailnetDevices => host::devices_call(&self.host),
            Call::SetTailnetDeviceTrust { device, trusted } => {
                host::set_device_trust(&self.host, device, trusted)
            }
            Call::Export { path, private } => Ok(started(transfer::export::start(
                &self.services,
                path,
                private,
            ))),
            Call::Import { path } => transfer::import::start(&self.host, &path).map(started),

            // Playback: the player's session, Discord, casting and rooms
            Call::OpenPlayback { file } => session::open_call(&self.playback, file),
            Call::ReportChapters {
                session,
                chapters,
                duration,
            } => skip::start(&self.playback, session, chapters, duration).map(started),
            Call::Tick {
                session,
                position,
                paused,
            } => session::tick(&self.playback, session, position, paused).map(|()| Reply::Ok),
            Call::ClosePlayback {
                session,
                position,
                reason,
            } => session::close(&self.playback, session, position, reason).map(|()| Reply::Ok),
            Call::SetTrackChoice {
                series,
                audio,
                subtitle,
            } => session::set_track_choice(&self.services, series, audio, subtitle),
            Call::GetPresence => presence::status_call(&self.playback),
            Call::DiscordAnswer { user_id, allow } => {
                presence::join::answer_call(&self.playback, user_id, allow)
            }
            Call::DiscordBlocked => presence::join::blocked_call(&self.playback),
            Call::DiscordUnblock { user_id } => {
                presence::join::unblock_call(&self.playback, user_id)
            }
            Call::DiscordSetRoomOpen { open } => {
                presence::join::set_open_call(&self.playback, open)
            }
            Call::DiscordListen => presence::join::listen_call(&self.playback),
            Call::DiscordTakeJoin => presence::join::take_join_call(&self.playback),
            c @ (Call::CastDevices
            | Call::CastStart { .. }
            | Call::CastStartStream { .. }
            | Call::CastPlay
            | Call::CastPause
            | Call::CastSeek { .. }
            | Call::CastStop
            | Call::CastStatus) => cast::call(&self.playback, c),
            c @ (Call::RoomCreate { .. }
            | Call::RoomJoin { .. }
            | Call::RoomAttach { .. }
            | Call::RoomCommand { .. }
            | Call::RoomSetOffset { .. }
            | Call::RoomLeave
            | Call::RoomStatus
            | Call::RoomGetMedia { .. }
            | Call::RoomStart { .. }) => room::call(&self.rooms, c),
        }
    }

    /// Keeps the whole-library load for the rest of its state generation
    /// (at most 30 seconds), so reads in one beat share one snapshot. For a
    /// shell whose every write goes through calls; off by default.
    pub fn set_snapshot_cache(&self, on: bool) {
        self.library.snapshot_cache.store(on, Ordering::Relaxed);
        if !on {
            *recover(&self.library.kept_load) = None;
        }
    }

    /// The state generation (`EventBus::generation`): a shell may keep a
    /// read for as long as this has not moved.
    pub fn generation(&self) -> u64 {
        self.services.bus.generation()
    }

    pub fn subscribe(&self, listener: Arc<dyn EventListener>) -> Arc<Subscription> {
        Subscription::new(self.services.bus.clone(), listener)
    }

    /// Stops the watcher, cancels every job, checkpoints and closes the
    /// store. Idempotent: a second call does nothing.
    pub fn shutdown(&self) {
        if self.closed.swap(true, Ordering::SeqCst) {
            return;
        }
        // First: the debouncer's thread dispatches into the library, so it
        // has to be gone before the runtime and the store are.
        self.library.stop_watcher();
        // A launch whose scan never finished has no list left to run.
        launch::take(&self.launch);
        crate::calendar::stop(&self.metadata);
        crate::aired::stop(&self.metadata);
        // The anirss sidecar goes with the core that started it.
        self.downloads.stop();
        self.host.stop();
        // Discord drops the activity when the socket closes anyway; this
        // clears it first and ends the worker.
        crate::presence::shutdown(&self.playback);
        // A room says goodbye, so the others see this member leave at once.
        crate::room::shutdown(&self.rooms);
        self.services.jobs.cancel_all();
        // Taken out of the mutex and dropped by this `let` before the
        // blocking shutdown call, rather than matched straight off the
        // lock expression: `if let Some(x) = recover(&mutex).take()`
        // keeps the guard alive for the whole `if let` body, which would
        // hold this lock for as long as the wait below.
        let runtime = recover(&self.runtime).take();
        self.services.close(runtime);
    }
}

/// What the host below asks of the front door (#854): the phone's calls,
/// answered as the profile that sent them, and a session's skip windows.
impl host::Front for Core {
    fn answer(&self, profile: u64, call: Call) -> Result<Reply, CoreError> {
        self.call_as(profile, call)
    }

    fn skip_windows(&self, session: u64) -> Result<Vec<SkipWindow>, CoreError> {
        session::windows_of(&self.playback, session)
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        // A team can outlive the core in a task that still holds it. The
        // watcher went quiet when its `Weak<Core>` stopped upgrading; now
        // that it holds the library, the core stops it on the way out.
        self.closed.store(true, Ordering::SeqCst);
        self.library.stop_watcher();
        // A plain drop of a `Runtime` from inside a runtime context panics,
        // and a shell that forgets to call `shutdown` should still exit
        // cleanly; hand any runtime still present to the background path
        // instead. Taken out of the mutex before that call, for the same
        // reason `shutdown` above does.
        let runtime = recover(&self.runtime).take();
        if let Some(runtime) = runtime {
            runtime.shutdown_background();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Collector, Signal};

    /// A tracker list read again refits Recommended once a fit exists: the
    /// trackers say so on the bus rather than calling here, and the refit has
    /// started by the time `signal` returns (#854). Before the first fit it
    /// waits for Home to ask.
    #[test]
    fn a_tracker_refresh_refits_a_fit_that_exists() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            crate::net::FakeHttp::new(),
            crate::foundation::secrets::Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        let c = Arc::new(Collector::default());
        core.services.bus.subscribe(c.clone());
        let refits = |c: &Collector| {
            c.bodies()
                .iter()
                .filter(|b| {
                    matches!(
                        b,
                        EventBody::JobStarted {
                            kind: JobKind::RefreshRecommended
                        }
                    )
                })
                .count()
        };
        core.services.bus.signal(Signal::TrackersChanged);
        assert_eq!(refits(&c), 0);
        core.services
            .store
            .write(|c| {
                c.execute(
                    "INSERT INTO rec_state (id, fitted_at, state) VALUES (1, 1, 'Ready')",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        core.services.bus.signal(Signal::TrackersChanged);
        assert_eq!(refits(&c), 1);
        core.shutdown();
    }

    #[test]
    fn a_start_line_from_before_the_rename_still_gives_its_version() {
        assert_eq!(ready_version("Anibeam core 2.5.0 ready"), Some("2.5.0"));
        let old = format!("Ani{}eam core 2.4.0 ready", 'B');
        assert_eq!(ready_version(&old), Some("2.4.0"));
        assert_eq!(ready_version("Anibeam core 2.5.0"), None);
        assert_eq!(ready_version("core 2.5.0 ready"), None);
    }
}
