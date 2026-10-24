//! Door: the shell's one way into the core. One invokable per call, one signal per event
//! body, JSON for anything deep. The listener hops every event to the Qt thread through
//! CxxQtThread::queue, and the state QML shares (preferences, settings, trackers, running
//! jobs, the latest line, unseen errors) is kept current here from those events.
//!
//! Two names had to move to fit Qt's one namespace per object. The `about` invokable is
//! `getAbout()`, because QML cannot reach both a property and a method called `about`; it
//! now reads like its neighbours `getPreferences`, `getSettings` and `getTrackers`. Those
//! three, and `setPreferences`, are free because the four JSON state properties name their
//! own accessors the Qt way (`preferences()`, not `getPreferences()`) and carry the event
//! signal as their NOTIFY, which is why `preferences`, `settings` and `trackers` have no
//! generated setter and go through `put_*` below.

use core::pin::Pin;
use std::cell::RefCell;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use anibeam_core::events::Subscription;
use anibeam_core::{
    Call, Chapter, CloseReason, CoreError, Direction, DownloadOptions, DownloadSettings, Event,
    EventBody, EventListener, FeedSort, JobPhase, Level, MatchTarget, MetadataFilter, Preferences,
    Provider, RefineOp, Release, Sort, SubscriptionsResult, SubtitleChoice, SubtitleDefaults, Tab,
    TorrentAction, TrackRef, Tracker,
};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QJsonArray, QJsonObject, QString};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::json::{self, from_qjson_object, to_qjson_array, to_qjson_object};
use crate::read_cache::{self, ReadCache};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qjsonobject.h");
        type QJsonObject = cxx_qt_lib::QJsonObject;
        include!("cxx-qt-lib/qjsonarray.h");
        type QJsonArray = cxx_qt_lib::QJsonArray;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(bool, ready)]
        #[qproperty(bool, reveal_hidden)]
        #[qproperty(QJsonObject, preferences, READ = preferences, NOTIFY = preferences_changed)]
        #[qproperty(QJsonObject, settings, READ = settings, NOTIFY = settings_changed)]
        #[qproperty(QJsonObject, trackers, READ = trackers, NOTIFY = trackers_changed)]
        #[qproperty(QJsonObject, about, READ = about, WRITE, NOTIFY)]
        #[qproperty(QJsonArray, running_jobs)]
        #[qproperty(QJsonObject, latest_line)]
        #[qproperty(i32, unseen_errors)]
        type Door = super::DoorRust;

        // The four state properties read through these rather than through a generated
        // `getX`, which leaves `getPreferences`, `getSettings`, `getTrackers` and
        // `getAbout` to the invokables. Not invokable themselves, so QML sees one name.
        fn preferences(self: &Door) -> QJsonObject;
        fn settings(self: &Door) -> QJsonObject;
        fn trackers(self: &Door) -> QJsonObject;
        fn about(self: &Door) -> QJsonObject;

        // Library
        #[qinvokable]
        fn list_sources(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn add_source(self: &Door, path: &QString) -> QJsonObject;
        #[qinvokable]
        fn remove_source(self: &Door, source: f64) -> QJsonObject;
        #[qinvokable]
        fn forget_series(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable]
        fn scan(self: &Door, source: f64) -> QJsonObject;
        #[qinvokable]
        fn rescan_series(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable]
        fn lookup(self: &Door, path: &QString) -> QJsonObject;
        #[qinvokable]
        fn list_series(
            self: &Door,
            tab: &QString,
            query: &QString,
            sort: &QString,
            direction: &QString,
            reveal_hidden: bool,
        ) -> QJsonObject;
        #[qinvokable]
        fn list_airing(self: &Door, offset: f64, limit: f64) -> QJsonObject;
        /// The library page in one read: `{ series, airing, counts: { all, series,
        /// movies, watched, hidden } }`, off one snapshot. Every card, here and in
        /// listSeries, carries `franchise: { id, title, size }` when other library
        /// series share its franchise, else null (core `franchise::groups`).
        #[qinvokable]
        fn list_library(
            self: &Door,
            tab: &QString,
            query: &QString,
            sort: &QString,
            direction: &QString,
            reveal_hidden: bool,
        ) -> QJsonObject;
        #[qinvokable]
        fn get_series(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable]
        fn set_hidden(self: &Door, series: f64, hidden: bool) -> QJsonObject;
        #[qinvokable]
        fn list_feed(self: &Door, sort: &QString) -> QJsonObject;
        #[qinvokable]
        fn list_metadata(
            self: &Door,
            filter: &QString,
            query: &QString,
            reveal_hidden: bool,
        ) -> QJsonObject;
        #[qinvokable]
        fn list_subscriptions(self: &Door) -> QJsonObject;
        // Metadata
        /// AniList's search, kept for the panel as it stands.
        #[qinvokable]
        fn search_provider(self: &Door, query: &QString, limit: i32) -> QJsonObject;
        /// The search on one named provider: `Anilist` or `Tvmaze`.
        #[qinvokable]
        fn search_on(self: &Door, provider: &QString, query: &QString, limit: i32) -> QJsonObject;
        #[qinvokable]
        fn resolve_link(self: &Door, url: &QString) -> QJsonObject;
        #[qinvokable]
        fn apply_match(self: &Door, series: f64, target: &QJsonObject) -> QJsonObject;
        /// `apply_match` with a TVmaze show id, so a result row needs no target object.
        #[qinvokable]
        fn apply_tvmaze(self: &Door, series: f64, show: f64) -> QJsonObject;
        #[qinvokable]
        fn clear_match(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable]
        fn refresh_series(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable]
        fn refresh_all(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn auto_match(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn refresh_airing(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable]
        fn get_storage(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn clear_images(self: &Door) -> QJsonObject;
        // Trackers
        #[qinvokable]
        fn get_trackers(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn set_tracker_credentials(
            self: &Door,
            tracker: &QString,
            client_id: &QString,
            client_secret: &QString,
        ) -> QJsonObject;
        #[qinvokable]
        fn connect_tracker(self: &Door, tracker: &QString) -> QJsonObject;
        #[qinvokable]
        fn disconnect_tracker(self: &Door, tracker: &QString) -> QJsonObject;
        #[qinvokable]
        fn set_main_tracker(self: &Door, tracker: &QString) -> QJsonObject;
        #[qinvokable]
        fn mark_episode(self: &Door, series: f64, episode: f64) -> QJsonObject;
        #[qinvokable]
        fn set_progress(self: &Door, series: f64, progress: i32) -> QJsonObject;
        #[qinvokable]
        fn set_score(self: &Door, series: f64, score: f64) -> QJsonObject;
        #[qinvokable]
        fn refresh_progress(self: &Door, tracker: &QString) -> QJsonObject;
        #[qinvokable]
        fn list_watching(self: &Door) -> QJsonObject;
        /// Starts the AniList watching refresh when the cached list is over five minutes
        /// old, or always with `force`. `Started` with the job, or `Ok` when fresh or
        /// not connected; the job ends in `watchingRefreshed`.
        #[qinvokable]
        fn refresh_watching(self: &Door, force: bool) -> QJsonObject;
        // Franchise
        #[qinvokable]
        fn get_franchise_graph(self: &Door, series: f64) -> QJsonObject;
        // Playback
        #[qinvokable]
        fn open_playback(self: &Door, file: f64) -> QJsonObject;
        #[qinvokable]
        fn report_chapters(
            self: &Door,
            session: f64,
            chapters: &QJsonArray,
            duration: f64,
        ) -> QJsonObject;
        #[qinvokable]
        fn tick(self: &Door, session: f64, position: f64, paused: bool) -> QJsonObject;
        #[qinvokable]
        fn close_playback(
            self: &Door,
            session: f64,
            position: f64,
            reason: &QString,
        ) -> QJsonObject;
        #[qinvokable]
        fn set_track_choice(
            self: &Door,
            series: f64,
            audio: &QJsonObject,
            subtitle: &QJsonObject,
        ) -> QJsonObject;
        // Store
        /// The `About` call. Named `getAbout` so the `about` property keeps its own name.
        #[qinvokable]
        fn get_about(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn get_preferences(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn set_preferences(self: &Door, preferences: &QJsonObject) -> QJsonObject;
        #[qinvokable]
        fn get_settings(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn set_subtitle_defaults(self: &Door, defaults: &QJsonObject) -> QJsonObject;
        #[qinvokable]
        fn set_auto_skip(self: &Door, intro: bool, outro: bool) -> QJsonObject;
        /// The match providers in order, as their names: `["Tvmaze", "Anilist"]`. The
        /// current order is `settings.provider_order`.
        #[qinvokable]
        fn set_provider_order(self: &Door, order: &QJsonArray) -> QJsonObject;
        #[qinvokable]
        fn export_library(self: &Door, path: &QString, private_data: bool) -> QJsonObject;
        #[qinvokable]
        fn import_library(self: &Door, path: &QString) -> QJsonObject;
        #[qinvokable]
        fn recent_events(self: &Door, limit: f64) -> QJsonObject;
        #[qinvokable]
        fn clear_events(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn list_jobs(self: &Door) -> QJsonObject;
        #[qinvokable]
        fn cancel_job(self: &Door, job: f64) -> QJsonObject;
        // Downloads (anirss and the torrent client). Every item list crosses as anirss's
        // own Item shape; the page sends back what it was given.
        /// The tailnet host's state: whether it is on, the URLs it serves on and one
        /// plain sentence. No network; changes arrive as `hostChanged`.
        #[qinvokable]
        fn get_host(self: &Door) -> QJsonObject;
        /// The status as last known, settings and endpoints included. No network.
        #[qinvokable]
        fn get_downloads(self: &Door) -> QJsonObject;
        /// Starts the sidecar if needed and reads both halves; ends in
        /// `downloadsStatusChanged`.
        #[qinvokable]
        fn connect_downloads(self: &Door) -> QJsonObject;
        /// An empty `endpoint` means anirss's first one. Ends in `releasesFound(job, search)`.
        #[qinvokable]
        fn search_releases(self: &Door, query: &QString, endpoint: &QString) -> QJsonObject;
        /// `op` is `{ type: "tokens", tokens: [...] }`, `{ type: "term", term }`,
        /// `{ type: "exclude", term }` or `{ type: "best_fit" }`. Ends in
        /// `releasesRefined(job, refinement)`.
        #[qinvokable]
        fn refine_releases(
            self: &Door,
            query: &QString,
            endpoint: &QString,
            items: &QJsonArray,
            op: &QJsonObject,
        ) -> QJsonObject;
        #[qinvokable]
        fn preview_subscription(
            self: &Door,
            query: &QString,
            endpoint: &QString,
            items: &QJsonArray,
        ) -> QJsonObject;
        /// An empty `name` lets anirss name the rule off the first release.
        #[qinvokable]
        fn subscribe(
            self: &Door,
            query: &QString,
            endpoint: &QString,
            items: &QJsonArray,
            name: &QString,
            wire_query: &QString,
        ) -> QJsonObject;
        #[qinvokable]
        fn download_releases(self: &Door, items: &QJsonArray, name: &QString) -> QJsonObject;
        /// `download_releases` plus `{ sequential, provider, series, replace: [{ item,
        /// episodes }], cancel: [hash] }`: read in order, and the episodes the new torrents
        /// replace.
        #[qinvokable]
        fn download_releases_with(
            self: &Door,
            items: &QJsonArray,
            name: &QString,
            options: &QJsonObject,
        ) -> QJsonObject;
        #[qinvokable]
        fn download_movie(self: &Door, link: &QString) -> QJsonObject;
        /// Replies `Ok`; the list arrives as `subscriptionsRead`, or `downloadsUnavailable`.
        #[qinvokable]
        fn list_subscription_rules(self: &Door) -> QJsonObject;
        /// Replies `Ok`; the list arrives as `torrentsRead`, or `downloadsUnavailable`.
        #[qinvokable]
        fn list_torrents(self: &Door) -> QJsonObject;
        /// Replies `Ok`; the entries arrive as `feedRead`, or `downloadsUnavailable` with
        /// `what` "feed".
        #[qinvokable]
        fn read_feed(self: &Door, url: &QString) -> QJsonObject;
        /// Replies `Ok`; the rule arrives as `ruleRead`, or `downloadsUnavailable` with
        /// `what` "rule".
        #[qinvokable]
        fn read_rule(self: &Door, name: &QString) -> QJsonObject;
        #[qinvokable]
        fn repair_subscription(self: &Door, name: &QString) -> QJsonObject;
        #[qinvokable]
        fn plan_remove_subscription(
            self: &Door,
            name: &QString,
            torrents: bool,
            files: bool,
        ) -> QJsonObject;
        #[qinvokable]
        fn remove_subscription(
            self: &Door,
            name: &QString,
            torrents: bool,
            files: bool,
        ) -> QJsonObject;
        /// `action` is `Stop`, `Start` or `Delete`; `files` only matters to `Delete`.
        #[qinvokable]
        fn control_torrents(
            self: &Door,
            hashes: &QJsonArray,
            action: &QString,
            files: bool,
        ) -> QJsonObject;
        /// One removal for a show: `rule` empty for none, `series` below 0 for none.
        #[qinvokable]
        fn remove_show(
            self: &Door,
            title: &QString,
            rule: &QString,
            hashes: &QJsonArray,
            torrents: bool,
            files: bool,
            series: f64,
        ) -> QJsonObject;
        /// `secrets` carries `anirss_token` and `torrent_password` only when they are to
        /// change: a key left out keeps what is stored, an empty string removes it.
        #[qinvokable]
        fn set_download_settings(
            self: &Door,
            settings: &QJsonObject,
            secrets: &QJsonObject,
        ) -> QJsonObject;
        #[qinvokable]
        fn test_torrent_client(self: &Door) -> QJsonObject;
        /// Switches to the saved torrent client `id`; an empty id is AniBeam's own.
        #[qinvokable]
        fn use_torrent_client(self: &Door, id: &QString) -> QJsonObject;
        /// Any call by name, the CLI's door.
        #[qinvokable]
        fn call(self: &Door, name: &QString, args: &QJsonObject) -> QJsonObject;
        /// The same off the Qt thread, for a call that waits on the network (fetching an
        /// endpoint repo): returns a token at once, and the reply comes back as
        /// `callDone(token, reply)` in the shape `call` returns.
        #[qinvokable]
        fn call_async(self: &Door, name: &QString, args: &QJsonObject) -> f64;
        #[qinvokable]
        fn mark_log_seen(self: Pin<&mut Door>);

        // The envelope and the derived job signal
        #[qsignal]
        fn event(self: Pin<&mut Door>, envelope: QJsonObject);
        #[qsignal]
        fn job_finished(self: Pin<&mut Door>, job: f64, kind: QString, ok: bool);
        #[qsignal]
        fn call_done(self: Pin<&mut Door>, token: f64, reply: QJsonObject);
        // One per event body. `ready` is `ready()` in QML; the Rust name steps aside so the
        // `ready` property keeps its generated getter.
        #[qsignal]
        #[cxx_name = "ready"]
        fn ready_signal(self: Pin<&mut Door>);
        #[qsignal]
        fn notice(self: Pin<&mut Door>);
        #[qsignal]
        fn job_started(self: Pin<&mut Door>, job: f64, kind: QString);
        #[qsignal]
        fn job_progress(
            self: Pin<&mut Door>,
            job: f64,
            kind: QString,
            done: f64,
            total: f64,
            label: QString,
        );
        #[qsignal]
        fn job_failed(self: Pin<&mut Door>, job: f64, kind: QString, error: QJsonObject);
        #[qsignal]
        fn job_cancelled(self: Pin<&mut Door>, job: f64, kind: QString);
        #[qsignal]
        fn source_changed(self: Pin<&mut Door>, source: QJsonObject);
        #[qsignal]
        fn source_removed(self: Pin<&mut Door>, source: f64);
        #[qsignal]
        fn series_changed(self: Pin<&mut Door>, cards: QJsonArray);
        #[qsignal]
        fn series_removed(self: Pin<&mut Door>, ids: QJsonArray);
        #[qsignal]
        fn scan_finished(self: Pin<&mut Door>, source: f64, added: f64, changed: f64, removed: f64);
        #[qsignal]
        fn subscriptions_listed(self: Pin<&mut Door>, result: QJsonObject);
        #[qsignal]
        fn search_finished(self: Pin<&mut Door>, job: f64, results: QJsonArray);
        #[qsignal]
        fn link_resolved(self: Pin<&mut Door>, job: f64, target: QJsonObject);
        #[qsignal]
        fn match_applied(self: Pin<&mut Door>, series: f64);
        #[qsignal]
        fn refresh_finished(self: Pin<&mut Door>, refreshed: f64, failed: f64);
        #[qsignal]
        fn auto_match_finished(self: Pin<&mut Door>, backfilled: f64, matched: f64, unmatched: f64);
        #[qsignal]
        fn airing_refreshed(self: Pin<&mut Door>, series: f64, updated: bool);
        #[qsignal]
        fn images_cleared(self: Pin<&mut Door>, removed: f64);
        #[qsignal]
        fn trackers_changed(self: Pin<&mut Door>, state: QJsonObject);
        #[qsignal]
        /// `job` is the connect job the url belongs to, so a shell can drop the url of a
        /// flow it has since replaced instead of opening a sign-in nobody is waiting on.
        fn auth_url_ready(
            self: Pin<&mut Door>,
            tracker: QString,
            open_url: QString,
            redirect_url: QString,
            job: f64,
        );
        #[qsignal]
        fn tracker_connected(self: Pin<&mut Door>, tracker: QString, username: QString);
        #[qsignal]
        fn marked(self: Pin<&mut Door>, series: f64, episode: i32, outcomes: QJsonArray);
        #[qsignal]
        fn progress_set(self: Pin<&mut Door>, series: f64, progress: i32, outcomes: QJsonArray);
        #[qsignal]
        fn scored(self: Pin<&mut Door>, series: f64, score: f64, outcomes: QJsonArray);
        #[qsignal]
        fn progress_refreshed(self: Pin<&mut Door>, tracker: QString);
        #[qsignal]
        fn watching_refreshed(self: Pin<&mut Door>, list: QJsonObject);
        #[qsignal]
        fn graph_changed(self: Pin<&mut Door>, root: f64);
        #[qsignal]
        fn crawl_finished(self: Pin<&mut Door>, fetched: f64, deferred: f64);
        #[qsignal]
        fn skip_windows_ready(self: Pin<&mut Door>, session: f64, windows: QJsonArray);
        #[qsignal]
        fn resume_point_changed(self: Pin<&mut Door>, file: f64, position: f64);
        #[qsignal]
        fn viewed(self: Pin<&mut Door>, series: f64, episode: QString);
        #[qsignal]
        fn preferences_changed(self: Pin<&mut Door>, preferences: QJsonObject);
        #[qsignal]
        fn settings_changed(self: Pin<&mut Door>);
        #[qsignal]
        fn export_finished(self: Pin<&mut Door>, path: QString);
        #[qsignal]
        fn import_finished(self: Pin<&mut Door>, summary: QJsonObject);
        #[qsignal]
        fn host_changed(self: Pin<&mut Door>, status: QJsonObject);
        // Downloads
        #[qsignal]
        fn downloads_status_changed(self: Pin<&mut Door>, status: QJsonObject);
        #[qsignal]
        fn releases_found(self: Pin<&mut Door>, job: f64, search: QJsonObject);
        #[qsignal]
        fn releases_refined(self: Pin<&mut Door>, job: f64, refinement: QJsonObject);
        #[qsignal]
        fn subscription_previewed(self: Pin<&mut Door>, job: f64, spec: QJsonObject);
        #[qsignal]
        fn subscribed(self: Pin<&mut Door>, job: f64, result: QJsonObject);
        #[qsignal]
        fn torrents_added(self: Pin<&mut Door>, job: f64, result: QJsonObject);
        #[qsignal]
        fn subscriptions_read(self: Pin<&mut Door>, rules: QJsonArray);
        #[qsignal]
        fn torrents_read(
            self: Pin<&mut Door>,
            torrents: QJsonArray,
            transfer: QJsonObject,
            shows: QJsonArray,
        );
        #[qsignal]
        fn feed_read(self: Pin<&mut Door>, url: QString, entries: QJsonArray);
        #[qsignal]
        fn release_inspected(self: Pin<&mut Door>, link: QString, episodes: QJsonArray, files: f64);
        #[qsignal]
        fn rule_read(self: Pin<&mut Door>, rule: QJsonObject);
        #[qsignal]
        fn subscription_repaired(
            self: Pin<&mut Door>,
            job: f64,
            name: QString,
            feed_url: QString,
            must_contain: QString,
            found: f64,
        );
        #[qsignal]
        fn releases_enriched(
            self: Pin<&mut Door>,
            query: QString,
            endpoint: QString,
            entries: QJsonArray,
        );
        #[qsignal]
        fn downloads_unavailable(self: Pin<&mut Door>, what: QString, error: QJsonObject);
        #[qsignal]
        fn removal_planned(self: Pin<&mut Door>, job: f64, plans: QJsonArray);
        #[qsignal]
        fn subscription_removed(self: Pin<&mut Door>, name: QString);
        #[qsignal]
        fn torrents_controlled(self: Pin<&mut Door>, hashes: QJsonArray, action: QString);
        /// `rule` is empty when the show had none.
        #[qsignal]
        fn show_removed(
            self: Pin<&mut Door>,
            job: f64,
            title: QString,
            rule: QString,
            hashes: QJsonArray,
            trashed: QJsonArray,
        );
    }

    impl cxx_qt::Threading for Door {}
    impl cxx_qt::Initialize for Door {}
}

#[derive(Default)]
pub struct DoorRust {
    ready: bool,
    reveal_hidden: bool,
    preferences: QJsonObject,
    settings: QJsonObject,
    trackers: QJsonObject,
    about: QJsonObject,
    running_jobs: QJsonArray,
    latest_line: QJsonObject,
    unseen_errors: i32,
    subscription: Option<Arc<Subscription>>,
    jobs: Vec<Value>,
    /// Whole-library answers kept while the core's generation stands still.
    read_cache: RefCell<ReadCache<QJsonObject>>,
}

impl Default for ReadCache<QJsonObject> {
    /// On unless `ANIBEAM_READ_CACHE=0`, which is there to measure what it saves.
    fn default() -> Self {
        ReadCache::new(std::env::var("ANIBEAM_READ_CACHE").as_deref() != Ok("0"))
    }
}

/// `ANIBEAM_TIMING=1` prints every Door call's cost to stderr: the core's work, the walk
/// to a JSON value, and the walk to a QJsonObject, or a cache hit.
fn timing() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("ANIBEAM_TIMING").as_deref() == Ok("1"))
}

fn call_name(call: &Call) -> String {
    match serde_json::to_value(call) {
        Ok(Value::String(s)) => s,
        Ok(Value::Object(m)) => m.keys().next().cloned().unwrap_or_default(),
        _ => "?".into(),
    }
}

fn ms(d: std::time::Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// Runs on the core's threads; every event becomes one queued closure on the Qt thread.
struct Forwarder {
    qt: cxx_qt::CxxQtThread<qobject::Door>,
}

impl EventListener for Forwarder {
    fn on_event(&self, event: Event) {
        let envelope = json::event_json(&event);
        self.qt
            .queue(move |door: Pin<&mut qobject::Door>| door.receive(event, envelope))
            .ok();
    }
}

impl cxx_qt::Initialize for qobject::Door {
    fn initialize(mut self: Pin<&mut Self>) {
        let core = crate::runtime::core();
        // Subscribed before start, so the Ready line and a fast job's first events are seen.
        let sub = core.subscribe(Arc::new(Forwarder {
            qt: self.qt_thread(),
        }));
        self.as_mut().rust_mut().subscription = Some(sub);
        // Every write the shell makes goes through a call that emits, so the core may keep
        // its whole-library load for the rest of a generation; the same switch as the
        // Door's own read cache.
        core.set_snapshot_cache(std::env::var("ANIBEAM_READ_CACHE").as_deref() != Ok("0"));
        if let Err(e) = core.start() {
            eprintln!("anibeam: core start: {e}");
        }
        self.as_mut().refresh_shared();
        self.as_mut().set_ready(true);
    }
}

fn id(v: f64) -> u64 {
    if v.is_finite() && v >= 0.0 {
        v as u64
    } else {
        0
    }
}

fn opt_id(v: f64) -> Option<u64> {
    if v.is_finite() && v >= 0.0 {
        Some(v as u64)
    } else {
        None
    }
}

fn parse_enum<T: DeserializeOwned>(field: &str, s: &QString) -> Result<T, CoreError> {
    serde_json::from_value(Value::String(s.to_string()))
        .map_err(|_| CoreError::invalid(field, format!("{s} is not a valid choice")))
}

fn parse_object<T: DeserializeOwned>(field: &str, o: &QJsonObject) -> Result<T, CoreError> {
    serde_json::from_value(from_qjson_object(o))
        .map_err(|e| CoreError::invalid(field, e.to_string()))
}

fn parse_option<T: DeserializeOwned>(field: &str, o: &QJsonObject) -> Result<Option<T>, CoreError> {
    if o.is_empty() {
        Ok(None)
    } else {
        parse_object(field, o).map(Some)
    }
}

/// An empty QString is `None`: the optional names and endpoints the downloads page sends.
fn non_empty(s: &QString) -> Option<String> {
    let s = s.to_string();
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// The page's item list back into anirss's `Item` shape, one bad item failing the call
/// rather than vanishing: a list sent back minus an item would shift every group's
/// member indexes.
fn releases(items: &QJsonArray) -> Result<Vec<Release>, CoreError> {
    serde_json::from_value(Value::Array(
        items.iter().map(|i| json::from_qjson(&i)).collect(),
    ))
    .map_err(|e| CoreError::invalid("items", e.to_string()))
}

/// `{ type, tokens | term }` from QML into the contract's op.
fn refine_op(o: &QJsonObject) -> Result<RefineOp, CoreError> {
    let v = from_qjson_object(o);
    let kind = v["type"].as_str().unwrap_or("");
    let term = || v["term"].as_str().unwrap_or("").to_string();
    match kind {
        "tokens" => {
            let tokens: Vec<String> = v["tokens"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|t| t.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            if tokens.is_empty() {
                return Err(CoreError::invalid("op", "a tokens op needs tokens"));
            }
            Ok(RefineOp::Tokens { tokens })
        }
        "term" => Ok(RefineOp::Term { term: term() }),
        "exclude" => Ok(RefineOp::Exclude { term: term() }),
        "best_fit" => Ok(RefineOp::BestFit),
        other => Err(CoreError::invalid(
            "op",
            format!("{other} is not a refine op"),
        )),
    }
}

/// What a panic carried, for the two payload shapes `panic!` produces.
fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "panicked".to_string()
    }
}

/// One running job as QML reads it. A job seeded from `ListJobs` before it has reported
/// progress takes the same shape as one that has just started, so a delegate never has to
/// tell the two apart.
fn job_entry(id: &Value, kind: &Value, progress: &Value) -> Value {
    json!({
        "id": id,
        "kind": kind,
        "done": progress.get("done").cloned().unwrap_or_else(|| json!(0)),
        "total": progress.get("total").cloned().unwrap_or(Value::Null),
        "label": progress.get("label").cloned().unwrap_or_else(|| json!("")),
    })
}

/// The `ListJobs` reply as the running-jobs list.
fn seed_jobs(reply: &Value) -> Vec<Value> {
    reply["jobs"]
        .as_array()
        .map(|jobs| {
            jobs.iter()
                .map(|j| job_entry(&j["id"], &j["kind"], &j["progress"]))
                .collect()
        })
        .unwrap_or_default()
}

/// One event's effect on the running-jobs list; true when the list actually changed.
///
/// `JobStarted` is an upsert rather than a push: `Core::start` files the launch job before
/// it emits, so the `ListJobs` the door reads while initialising can already hold that id,
/// and a blind push would leave two copies of one job with only the first ever taking its
/// progress.
fn apply_job(jobs: &mut Vec<Value>, id: u64, finished: bool, body: &EventBody) -> bool {
    let at = jobs.iter().position(|e| e["id"] == id);
    match body {
        EventBody::JobStarted { kind } => {
            let entry = job_entry(
                &json!(id),
                &serde_json::to_value(kind).unwrap_or(Value::Null),
                &Value::Null,
            );
            match at {
                Some(i) if jobs[i] == entry => false,
                Some(i) => {
                    jobs[i] = entry;
                    true
                }
                None => {
                    jobs.push(entry);
                    true
                }
            }
        }
        EventBody::JobProgress { done, total, label } => {
            let Some(i) = at else { return false };
            jobs[i]["done"] = json!(done);
            jobs[i]["total"] = json!(total);
            jobs[i]["label"] = json!(label);
            true
        }
        _ if finished => {
            let Some(i) = at else { return false };
            jobs.remove(i);
            true
        }
        _ => false,
    }
}

impl qobject::Door {
    /// Every invokable goes through here, so this is where a panic stops: one crossing the
    /// FFI boundary aborts the process, and QML asking for something the core is unhappy
    /// about should not take the window with it. A caught panic comes back as the ordinary
    /// `Internal` error shape.
    fn dispatch(&self, call: Result<Call, CoreError>) -> QJsonObject {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match call {
            Ok(c) => self.answer(c),
            Err(e) => to_qjson_object(&json!({ "error": json::error_json(&e) })),
        }))
        .unwrap_or_else(|payload| {
            let e = CoreError::internal(panic_text(&*payload));
            to_qjson_object(&json!({ "error": json::error_json(&e) }))
        })
    }

    /// One call, from the read cache when it is a whole-library read the core's state has
    /// not moved past. The generation is read before the call runs, so a change that lands
    /// during the read leaves its answer stamped old.
    fn answer(&self, call: Call) -> QJsonObject {
        let core = crate::runtime::core();
        let key = read_cache::key_of(&call);
        let name = if timing() {
            call_name(&call)
        } else {
            String::new()
        };
        let generation = core.generation();
        let t0 = Instant::now();
        if let Some(k) = &key
            && let Some(hit) = self.read_cache.borrow_mut().get(k, generation, t0)
        {
            if timing() {
                eprintln!("timing {name} hit {:.3}ms", ms(t0.elapsed()));
            }
            return hit;
        }
        let reply = core.call(call);
        let t1 = Instant::now();
        let ok = reply.is_ok();
        let v = json::reply_json(reply);
        let t2 = Instant::now();
        let o = to_qjson_object(&v);
        let t3 = Instant::now();
        if ok && let Some(k) = key {
            self.read_cache
                .borrow_mut()
                .put(k, generation, t0, o.clone());
        }
        if timing() {
            eprintln!(
                "timing {name} core {:.3}ms json {:.3}ms qjson {:.3}ms total {:.3}ms",
                ms(t1 - t0),
                ms(t2 - t1),
                ms(t3 - t2),
                ms(t3 - t0)
            );
        }
        o
    }

    fn reply_of(&self, call: Call) -> Option<Value> {
        let v = json::dispatch(crate::runtime::core(), call);
        v.get("reply").cloned()
    }

    // The four state properties' READ accessors.
    pub fn preferences(&self) -> QJsonObject {
        self.preferences.clone()
    }
    pub fn settings(&self) -> QJsonObject {
        self.settings.clone()
    }
    pub fn trackers(&self) -> QJsonObject {
        self.trackers.clone()
    }
    pub fn about(&self) -> QJsonObject {
        self.about.clone()
    }

    /// The three properties whose NOTIFY is the event signal have no generated setter, so
    /// the write and the emit are written out here. Both happen every time: an event that
    /// says preferences changed is worth reporting even when the value came back equal.
    fn put_preferences(mut self: Pin<&mut Self>, v: QJsonObject) {
        self.as_mut().rust_mut().preferences = v.clone();
        self.as_mut().preferences_changed(v);
    }

    fn put_settings(mut self: Pin<&mut Self>, v: QJsonObject) {
        self.as_mut().rust_mut().settings = v;
        self.as_mut().settings_changed();
    }

    fn put_trackers(mut self: Pin<&mut Self>, v: QJsonObject) {
        self.as_mut().rust_mut().trackers = v.clone();
        self.as_mut().trackers_changed(v);
    }

    /// Preferences, settings, trackers, about and the running jobs, read once after start
    /// and again whenever an event says one of them changed.
    fn refresh_shared(mut self: Pin<&mut Self>) {
        if let Some(p) = self.as_ref().reply_of(Call::GetPreferences) {
            self.as_mut()
                .put_preferences(to_qjson_object(&p["preferences"]));
        }
        if let Some(s) = self.as_ref().reply_of(Call::GetSettings) {
            self.as_mut().put_settings(to_qjson_object(&s["settings"]));
        }
        if let Some(t) = self.as_ref().reply_of(Call::GetTrackers) {
            self.as_mut().put_trackers(to_qjson_object(&t["state"]));
        }
        if let Some(a) = self.as_ref().reply_of(Call::About) {
            self.as_mut().set_about(to_qjson_object(&a["about"]));
        }
        if let Some(j) = self.as_ref().reply_of(Call::ListJobs) {
            let jobs = seed_jobs(&j);
            self.as_mut().rust_mut().jobs = jobs.clone();
            self.as_mut()
                .set_running_jobs(to_qjson_array(&Value::Array(jobs)));
        }
    }

    /// The other half of the panic barrier: an event is not worth the process. A panic here
    /// drops that one event and says so; the core has no shell-facing logger, so stderr is
    /// where it goes.
    pub fn receive(self: Pin<&mut Self>, event: Event, envelope: Value) {
        let seq = event.seq;
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            self.deliver(event, envelope)
        }));
        if let Err(payload) = caught {
            eprintln!("anibeam: event {seq} dropped: {}", panic_text(&*payload));
        }
    }

    /// On the Qt thread: the shared state, then the envelope, then the body's own signal.
    fn deliver(mut self: Pin<&mut Self>, event: Event, envelope: Value) {
        let job = event.job.clone();
        let finished = job.as_ref().is_some_and(|j| j.phase == JobPhase::Finished);
        // The borrow ends here, before any signal goes out.
        let jobs_changed = match &job {
            Some(j) => {
                let mut rust = self.as_mut().rust_mut();
                apply_job(&mut rust.jobs, j.id, finished, &event.body)
            }
            None => false,
        };
        if jobs_changed {
            let jobs = self.as_ref().jobs.clone();
            self.as_mut()
                .set_running_jobs(to_qjson_array(&Value::Array(jobs)));
        }
        // One walk of the envelope; the copy Qt makes of a QJsonObject is shared, not deep.
        let wire = to_qjson_object(&envelope);
        if event.level >= Level::Info {
            self.as_mut().set_latest_line(wire.clone());
        }
        if event.level == Level::Error {
            let n = *self.as_ref().unseen_errors() + 1;
            self.as_mut().set_unseen_errors(n);
        }
        self.as_mut().event(wire);
        if let Some(j) = &job
            && finished
        {
            let ok = !matches!(
                event.body,
                EventBody::JobFailed { .. } | EventBody::JobCancelled
            );
            let kind = QString::from(
                &serde_json::to_value(j.kind)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default(),
            );
            self.as_mut().job_finished(j.id as f64, kind, ok);
        }
        let job_id = job.as_ref().map(|j| j.id as f64).unwrap_or(-1.0);
        let job_kind = || {
            QString::from(
                &job.as_ref()
                    .and_then(|j| serde_json::to_value(j.kind).ok())
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default(),
            )
        };
        let body = &envelope["body"];
        let s = |k: &str| QString::from(body[k].as_str().unwrap_or(""));
        let n = |k: &str| body[k].as_f64().unwrap_or(-1.0);
        let arr = |k: &str| to_qjson_array(&body[k]);
        let obj = |k: &str| to_qjson_object(&body[k]);
        match &event.body {
            EventBody::Ready => self.as_mut().ready_signal(),
            EventBody::Notice => self.as_mut().notice(),
            EventBody::JobStarted { .. } => self.as_mut().job_started(job_id, job_kind()),
            EventBody::JobProgress { .. } => {
                self.as_mut()
                    .job_progress(job_id, job_kind(), n("done"), n("total"), s("label"))
            }
            EventBody::JobFailed { error } => self.as_mut().job_failed(
                job_id,
                job_kind(),
                to_qjson_object(&json::error_json(error)),
            ),
            EventBody::JobCancelled => self.as_mut().job_cancelled(job_id, job_kind()),
            EventBody::SourceChanged { .. } => self.as_mut().source_changed(obj("source")),
            EventBody::SourceRemoved { .. } => self.as_mut().source_removed(n("source")),
            EventBody::SeriesChanged { .. } => self.as_mut().series_changed(arr("series")),
            EventBody::SeriesRemoved { .. } => self.as_mut().series_removed(arr("ids")),
            EventBody::ScanFinished { .. } => {
                self.as_mut()
                    .scan_finished(n("source"), n("added"), n("changed"), n("removed"))
            }
            EventBody::SubscriptionsListed { result } => {
                let flat = match result {
                    SubscriptionsResult::Ok { .. } => {
                        json!({ "kind": "Ok", "feeds": body["result"]["Ok"]["feeds"] })
                    }
                    other => json!({
                        "kind": serde_json::to_value(other)
                            .ok()
                            .and_then(|v| v.as_str().map(String::from))
                            .unwrap_or_default(),
                    }),
                };
                self.as_mut().subscriptions_listed(to_qjson_object(&flat))
            }
            EventBody::SearchFinished { .. } => {
                self.as_mut().search_finished(job_id, arr("results"))
            }
            EventBody::LinkResolved { .. } => self.as_mut().link_resolved(job_id, obj("target")),
            EventBody::MatchApplied { .. } => self.as_mut().match_applied(n("series")),
            EventBody::RefreshFinished { .. } => {
                self.as_mut().refresh_finished(n("refreshed"), n("failed"))
            }
            EventBody::AutoMatchFinished { .. } => {
                self.as_mut()
                    .auto_match_finished(n("backfilled"), n("matched"), n("unmatched"))
            }
            EventBody::AiringRefreshed { updated, .. } => {
                self.as_mut().airing_refreshed(n("series"), *updated)
            }
            EventBody::ImagesCleared { .. } => self.as_mut().images_cleared(n("removed")),
            EventBody::TrackersChanged { .. } => self.as_mut().put_trackers(obj("state")),
            EventBody::AuthUrlReady { .. } => {
                self.as_mut()
                    .auth_url_ready(s("tracker"), s("open_url"), s("redirect_url"), job_id)
            }
            EventBody::TrackerConnected { .. } => {
                self.as_mut().tracker_connected(s("tracker"), s("username"))
            }
            EventBody::Marked { episode, .. } => {
                self.as_mut()
                    .marked(n("series"), *episode as i32, arr("outcomes"))
            }
            EventBody::ProgressSet { progress, .. } => {
                self.as_mut()
                    .progress_set(n("series"), *progress as i32, arr("outcomes"))
            }
            EventBody::Scored { .. } => {
                self.as_mut()
                    .scored(n("series"), n("score"), arr("outcomes"))
            }
            EventBody::ProgressRefreshed { .. } => self.as_mut().progress_refreshed(s("tracker")),
            EventBody::WatchingRefreshed { .. } => self.as_mut().watching_refreshed(obj("list")),
            EventBody::GraphChanged { .. } => self.as_mut().graph_changed(n("root")),
            EventBody::CrawlFinished { .. } => {
                self.as_mut().crawl_finished(n("fetched"), n("deferred"))
            }
            // The desktop does not cast yet; the phone does.
            EventBody::CastDevicesChanged { .. } | EventBody::CastChanged { .. } => {}
            EventBody::SkipWindowsReady { .. } => self
                .as_mut()
                .skip_windows_ready(n("session"), arr("windows")),
            EventBody::ResumePointChanged { .. } => {
                self.as_mut().resume_point_changed(n("file"), n("position"))
            }
            EventBody::Viewed { .. } => self.as_mut().viewed(n("series"), s("episode")),
            EventBody::PreferencesChanged { .. } => {
                self.as_mut().put_preferences(obj("preferences"))
            }
            // The event says settings changed but not to what, so they are re-read. A read
            // that fails leaves the last good value in place and still says so.
            EventBody::SettingsChanged => match self.as_ref().reply_of(Call::GetSettings) {
                Some(v) => self.as_mut().put_settings(to_qjson_object(&v["settings"])),
                None => self.as_mut().settings_changed(),
            },
            EventBody::ExportFinished { .. } => self.as_mut().export_finished(s("path")),
            EventBody::ImportFinished { .. } => self.as_mut().import_finished(obj("summary")),
            EventBody::HostChanged { .. } => self.as_mut().host_changed(obj("status")),
            EventBody::DownloadsStatusChanged { .. } => {
                self.as_mut().downloads_status_changed(obj("status"))
            }
            EventBody::ReleasesFound { .. } => self.as_mut().releases_found(job_id, obj("search")),
            EventBody::ReleasesRefined { .. } => {
                self.as_mut().releases_refined(job_id, obj("refinement"))
            }
            EventBody::SubscriptionPreviewed { .. } => {
                self.as_mut().subscription_previewed(job_id, obj("spec"))
            }
            EventBody::Subscribed { .. } => self.as_mut().subscribed(job_id, obj("result")),
            EventBody::TorrentsAdded { .. } => self.as_mut().torrents_added(job_id, obj("result")),
            EventBody::SubscriptionsRead { .. } => self.as_mut().subscriptions_read(arr("rules")),
            EventBody::TorrentsRead { .. } => {
                self.as_mut()
                    .torrents_read(arr("torrents"), obj("transfer"), arr("shows"))
            }
            EventBody::FeedRead { .. } => self.as_mut().feed_read(s("url"), arr("entries")),
            EventBody::ReleaseInspected { files, .. } => {
                self.as_mut()
                    .release_inspected(s("link"), arr("episodes"), *files as f64)
            }
            EventBody::RuleRead { .. } => self.as_mut().rule_read(obj("rule")),
            EventBody::SubscriptionRepaired { .. } => self.as_mut().subscription_repaired(
                job_id,
                s("name"),
                s("feed_url"),
                s("must_contain"),
                n("found"),
            ),
            EventBody::ReleasesEnriched { .. } => {
                self.as_mut()
                    .releases_enriched(s("query"), s("endpoint"), arr("entries"))
            }
            EventBody::DownloadsUnavailable { error, .. } => self
                .as_mut()
                .downloads_unavailable(s("what"), to_qjson_object(&json::error_json(error))),
            EventBody::RemovalPlanned { .. } => self.as_mut().removal_planned(job_id, arr("plans")),
            EventBody::SubscriptionRemoved { .. } => self.as_mut().subscription_removed(s("name")),
            EventBody::ShowRemoved { rule, .. } => self.as_mut().show_removed(
                job_id,
                s("title"),
                QString::from(rule.as_deref().unwrap_or("")),
                arr("hashes"),
                arr("trashed"),
            ),
            EventBody::TorrentsControlled { action, .. } => {
                let word = match action {
                    TorrentAction::Stop => "Stop",
                    TorrentAction::Start => "Start",
                    TorrentAction::Delete { .. } => "Delete",
                    TorrentAction::Sequential { on: true } => "Sequential",
                    TorrentAction::Sequential { on: false } => "Unsequential",
                    TorrentAction::FirstLastPieces { on: true } => "FirstLast",
                    TorrentAction::FirstLastPieces { on: false } => "UnFirstLast",
                };
                self.as_mut()
                    .torrents_controlled(arr("hashes"), QString::from(word))
            }
        }
    }

    pub fn mark_log_seen(self: Pin<&mut Self>) {
        self.set_unseen_errors(0);
    }

    // Library
    pub fn list_sources(&self) -> QJsonObject {
        self.dispatch(Ok(Call::ListSources))
    }
    pub fn add_source(&self, path: &QString) -> QJsonObject {
        self.dispatch(Ok(Call::AddSource {
            path: path.to_string(),
        }))
    }
    pub fn remove_source(&self, source: f64) -> QJsonObject {
        self.dispatch(Ok(Call::RemoveSource { source: id(source) }))
    }
    pub fn forget_series(&self, series: f64) -> QJsonObject {
        self.dispatch(Ok(Call::ForgetSeries { series: id(series) }))
    }
    pub fn scan(&self, source: f64) -> QJsonObject {
        self.dispatch(Ok(Call::Scan {
            source: opt_id(source),
        }))
    }
    pub fn rescan_series(&self, series: f64) -> QJsonObject {
        self.dispatch(Ok(Call::RescanSeries { series: id(series) }))
    }
    pub fn lookup(&self, path: &QString) -> QJsonObject {
        self.dispatch(Ok(Call::Lookup {
            path: path.to_string(),
        }))
    }
    pub fn list_series(
        &self,
        tab: &QString,
        query: &QString,
        sort: &QString,
        direction: &QString,
        reveal_hidden: bool,
    ) -> QJsonObject {
        self.dispatch((|| -> Result<Call, CoreError> {
            Ok(Call::ListSeries {
                tab: parse_enum::<Tab>("tab", tab)?,
                query: query.to_string(),
                sort: parse_enum::<Sort>("sort", sort)?,
                direction: parse_enum::<Direction>("direction", direction)?,
                reveal_hidden,
            })
        })())
    }
    pub fn list_airing(&self, offset: f64, limit: f64) -> QJsonObject {
        self.dispatch(Ok(Call::ListAiring {
            offset: id(offset),
            limit: id(limit),
        }))
    }
    pub fn list_library(
        &self,
        tab: &QString,
        query: &QString,
        sort: &QString,
        direction: &QString,
        reveal_hidden: bool,
    ) -> QJsonObject {
        self.dispatch((|| -> Result<Call, CoreError> {
            Ok(Call::ListLibrary {
                tab: parse_enum::<Tab>("tab", tab)?,
                query: query.to_string(),
                sort: parse_enum::<Sort>("sort", sort)?,
                direction: parse_enum::<Direction>("direction", direction)?,
                reveal_hidden,
            })
        })())
    }
    pub fn get_series(&self, series: f64) -> QJsonObject {
        self.dispatch(Ok(Call::GetSeries { series: id(series) }))
    }
    pub fn set_hidden(&self, series: f64, hidden: bool) -> QJsonObject {
        self.dispatch(Ok(Call::SetHidden {
            series: id(series),
            hidden,
        }))
    }
    pub fn list_feed(&self, sort: &QString) -> QJsonObject {
        self.dispatch(parse_enum::<FeedSort>("sort", sort).map(|sort| Call::ListFeed { sort }))
    }
    pub fn list_metadata(
        &self,
        filter: &QString,
        query: &QString,
        reveal_hidden: bool,
    ) -> QJsonObject {
        self.dispatch(
            parse_enum::<MetadataFilter>("filter", filter).map(|filter| Call::ListMetadata {
                filter,
                query: query.to_string(),
                reveal_hidden,
            }),
        )
    }
    pub fn list_subscriptions(&self) -> QJsonObject {
        self.dispatch(Ok(Call::ListSubscriptions))
    }
    // Metadata
    pub fn search_provider(&self, query: &QString, limit: i32) -> QJsonObject {
        // A limit of zero is a caller's mistake, not a request for one result; it comes
        // back as the same Invalid every other bad argument does.
        let call = if limit < 1 {
            Err(CoreError::invalid(
                "limit",
                format!("must be at least 1, got {limit}"),
            ))
        } else {
            Ok(Call::SearchProvider {
                provider: Provider::Anilist,
                query: query.to_string(),
                limit: limit as u32,
            })
        };
        self.dispatch(call)
    }
    pub fn search_on(&self, provider: &QString, query: &QString, limit: i32) -> QJsonObject {
        let call = if limit < 1 {
            Err(CoreError::invalid(
                "limit",
                format!("must be at least 1, got {limit}"),
            ))
        } else {
            parse_enum::<Provider>("provider", provider).map(|provider| Call::SearchProvider {
                provider,
                query: query.to_string(),
                limit: limit as u32,
            })
        };
        self.dispatch(call)
    }
    pub fn resolve_link(&self, url: &QString) -> QJsonObject {
        self.dispatch(Ok(Call::ResolveLink {
            url: url.to_string(),
        }))
    }
    pub fn apply_match(&self, series: f64, target: &QJsonObject) -> QJsonObject {
        self.dispatch(parse_object::<MatchTarget>("target", target).map(|target| {
            Call::ApplyMatch {
                series: id(series),
                target,
            }
        }))
    }
    pub fn apply_tvmaze(&self, series: f64, show: f64) -> QJsonObject {
        self.dispatch(Ok(Call::ApplyMatch {
            series: id(series),
            target: MatchTarget::Tvmaze { id: id(show) },
        }))
    }
    pub fn clear_match(&self, series: f64) -> QJsonObject {
        self.dispatch(Ok(Call::ClearMatch { series: id(series) }))
    }
    pub fn refresh_series(&self, series: f64) -> QJsonObject {
        self.dispatch(Ok(Call::RefreshSeries { series: id(series) }))
    }
    pub fn refresh_all(&self) -> QJsonObject {
        self.dispatch(Ok(Call::RefreshAll))
    }
    pub fn auto_match(&self) -> QJsonObject {
        self.dispatch(Ok(Call::AutoMatch))
    }
    pub fn refresh_airing(&self, series: f64) -> QJsonObject {
        self.dispatch(Ok(Call::RefreshAiring { series: id(series) }))
    }
    pub fn get_storage(&self) -> QJsonObject {
        self.dispatch(Ok(Call::GetStorage))
    }
    pub fn clear_images(&self) -> QJsonObject {
        self.dispatch(Ok(Call::ClearImages))
    }
    // Trackers
    pub fn get_trackers(&self) -> QJsonObject {
        self.dispatch(Ok(Call::GetTrackers))
    }
    pub fn set_tracker_credentials(
        &self,
        tracker: &QString,
        client_id: &QString,
        client_secret: &QString,
    ) -> QJsonObject {
        let secret = client_secret.to_string();
        self.dispatch(parse_enum::<Tracker>("tracker", tracker).map(|tracker| {
            Call::SetTrackerCredentials {
                tracker,
                client_id: client_id.to_string(),
                client_secret: if secret.is_empty() {
                    None
                } else {
                    Some(secret)
                },
            }
        }))
    }
    pub fn connect_tracker(&self, tracker: &QString) -> QJsonObject {
        self.dispatch(
            parse_enum::<Tracker>("tracker", tracker)
                .map(|tracker| Call::ConnectTracker { tracker }),
        )
    }
    pub fn disconnect_tracker(&self, tracker: &QString) -> QJsonObject {
        self.dispatch(
            parse_enum::<Tracker>("tracker", tracker)
                .map(|tracker| Call::DisconnectTracker { tracker }),
        )
    }
    pub fn set_main_tracker(&self, tracker: &QString) -> QJsonObject {
        self.dispatch(
            parse_enum::<Tracker>("tracker", tracker)
                .map(|tracker| Call::SetMainTracker { tracker }),
        )
    }
    pub fn mark_episode(&self, series: f64, episode: f64) -> QJsonObject {
        self.dispatch(Ok(Call::MarkEpisode {
            series: id(series),
            episode,
        }))
    }
    pub fn set_progress(&self, series: f64, progress: i32) -> QJsonObject {
        self.dispatch(Ok(Call::SetProgress {
            series: id(series),
            progress: progress.max(0) as u32,
        }))
    }
    pub fn set_score(&self, series: f64, score: f64) -> QJsonObject {
        self.dispatch(Ok(Call::SetScore {
            series: id(series),
            score: if score < 0.0 { None } else { Some(score) },
        }))
    }
    pub fn refresh_progress(&self, tracker: &QString) -> QJsonObject {
        let call = if tracker.to_string().is_empty() {
            Ok(Call::RefreshProgress { tracker: None })
        } else {
            parse_enum::<Tracker>("tracker", tracker)
                .map(|t| Call::RefreshProgress { tracker: Some(t) })
        };
        self.dispatch(call)
    }
    pub fn list_watching(&self) -> QJsonObject {
        self.dispatch(Ok(Call::ListWatching))
    }
    pub fn refresh_watching(&self, force: bool) -> QJsonObject {
        self.dispatch(Ok(Call::RefreshWatching { force }))
    }
    // Franchise
    pub fn get_franchise_graph(&self, series: f64) -> QJsonObject {
        self.dispatch(Ok(Call::GetFranchiseGraph { series: id(series) }))
    }
    // Playback
    pub fn open_playback(&self, file: f64) -> QJsonObject {
        self.dispatch(Ok(Call::OpenPlayback { file: id(file) }))
    }
    pub fn report_chapters(
        &self,
        session: f64,
        chapters: &QJsonArray,
        duration: f64,
    ) -> QJsonObject {
        let list: Result<Vec<Chapter>, CoreError> = serde_json::from_value(Value::Array(
            chapters.iter().map(|c| json::from_qjson(&c)).collect(),
        ))
        .map_err(|e| CoreError::invalid("chapters", e.to_string()));
        self.dispatch(list.map(|chapters| Call::ReportChapters {
            session: id(session),
            chapters,
            duration,
        }))
    }
    pub fn tick(&self, session: f64, position: f64, paused: bool) -> QJsonObject {
        self.dispatch(Ok(Call::Tick {
            session: id(session),
            position,
            paused,
        }))
    }
    pub fn close_playback(&self, session: f64, position: f64, reason: &QString) -> QJsonObject {
        self.dispatch(parse_enum::<CloseReason>("reason", reason).map(|reason| {
            Call::ClosePlayback {
                session: id(session),
                position,
                reason,
            }
        }))
    }
    pub fn set_track_choice(
        &self,
        series: f64,
        audio: &QJsonObject,
        subtitle: &QJsonObject,
    ) -> QJsonObject {
        // `{ off: true }` from QML is SubtitleChoice::Off; an empty object is none; anything
        // else, `{ off: false }` included, is `{ Track: { track: TrackRef } }`. The value is
        // what decides, not the presence of the key.
        let subtitle = if subtitle.value(&QString::from("off")).to_bool() {
            Ok(Some(SubtitleChoice::Off))
        } else {
            parse_option::<SubtitleChoice>("subtitle", subtitle)
        };
        self.dispatch((|| -> Result<Call, CoreError> {
            Ok(Call::SetTrackChoice {
                series: id(series),
                audio: parse_option::<TrackRef>("audio", audio)?,
                subtitle: subtitle?,
            })
        })())
    }
    // Store
    pub fn get_about(&self) -> QJsonObject {
        self.dispatch(Ok(Call::About))
    }
    pub fn get_preferences(&self) -> QJsonObject {
        self.dispatch(Ok(Call::GetPreferences))
    }
    pub fn set_preferences(&self, preferences: &QJsonObject) -> QJsonObject {
        self.dispatch(
            parse_object::<Preferences>("preferences", preferences)
                .map(|preferences| Call::SetPreferences { preferences }),
        )
    }
    pub fn get_settings(&self) -> QJsonObject {
        self.dispatch(Ok(Call::GetSettings))
    }
    pub fn set_subtitle_defaults(&self, defaults: &QJsonObject) -> QJsonObject {
        self.dispatch(
            parse_object::<SubtitleDefaults>("defaults", defaults)
                .map(|defaults| Call::SetSubtitleDefaults { defaults }),
        )
    }
    pub fn set_auto_skip(&self, intro: bool, outro: bool) -> QJsonObject {
        self.dispatch(Ok(Call::SetAutoSkip { intro, outro }))
    }
    pub fn set_provider_order(&self, order: &QJsonArray) -> QJsonObject {
        let order: Result<Vec<Provider>, CoreError> = serde_json::from_value(Value::Array(
            order.iter().map(|p| json::from_qjson(&p)).collect(),
        ))
        .map_err(|e| CoreError::invalid("order", e.to_string()));
        self.dispatch(order.map(|order| Call::SetProviderOrder { order }))
    }
    pub fn export_library(&self, path: &QString, private_data: bool) -> QJsonObject {
        self.dispatch(Ok(Call::Export {
            path: path.to_string(),
            private: private_data,
        }))
    }
    pub fn import_library(&self, path: &QString) -> QJsonObject {
        self.dispatch(Ok(Call::Import {
            path: path.to_string(),
        }))
    }
    pub fn recent_events(&self, limit: f64) -> QJsonObject {
        self.dispatch(Ok(Call::RecentEvents { limit: id(limit) }))
    }
    pub fn clear_events(&self) -> QJsonObject {
        self.dispatch(Ok(Call::ClearEvents))
    }
    pub fn list_jobs(&self) -> QJsonObject {
        self.dispatch(Ok(Call::ListJobs))
    }
    pub fn cancel_job(&self, job: f64) -> QJsonObject {
        self.dispatch(Ok(Call::CancelJob { job: id(job) }))
    }
    pub fn get_host(&self) -> QJsonObject {
        self.dispatch(Ok(Call::GetHost))
    }
    // Downloads
    pub fn get_downloads(&self) -> QJsonObject {
        self.dispatch(Ok(Call::GetDownloads))
    }
    pub fn connect_downloads(&self) -> QJsonObject {
        self.dispatch(Ok(Call::ConnectDownloads))
    }
    pub fn search_releases(&self, query: &QString, endpoint: &QString) -> QJsonObject {
        self.dispatch(Ok(Call::SearchReleases {
            query: query.to_string(),
            endpoint: non_empty(endpoint),
        }))
    }
    pub fn refine_releases(
        &self,
        query: &QString,
        endpoint: &QString,
        items: &QJsonArray,
        op: &QJsonObject,
    ) -> QJsonObject {
        self.dispatch((|| -> Result<Call, CoreError> {
            Ok(Call::RefineReleases {
                query: query.to_string(),
                endpoint: endpoint.to_string(),
                items: releases(items)?,
                op: refine_op(op)?,
            })
        })())
    }
    pub fn preview_subscription(
        &self,
        query: &QString,
        endpoint: &QString,
        items: &QJsonArray,
    ) -> QJsonObject {
        self.dispatch(releases(items).map(|items| Call::PreviewSubscription {
            query: query.to_string(),
            endpoint: endpoint.to_string(),
            items,
        }))
    }
    pub fn subscribe(
        &self,
        query: &QString,
        endpoint: &QString,
        items: &QJsonArray,
        name: &QString,
        wire_query: &QString,
    ) -> QJsonObject {
        self.dispatch(releases(items).map(|items| Call::Subscribe {
            query: query.to_string(),
            endpoint: endpoint.to_string(),
            items,
            name: non_empty(name),
            wire_query: non_empty(wire_query),
        }))
    }
    pub fn download_releases(&self, items: &QJsonArray, name: &QString) -> QJsonObject {
        self.dispatch(releases(items).map(|items| Call::DownloadReleases {
            items,
            name: non_empty(name),
        }))
    }
    pub fn download_releases_with(
        &self,
        items: &QJsonArray,
        name: &QString,
        options: &QJsonObject,
    ) -> QJsonObject {
        let options = serde_json::from_value::<DownloadOptions>(from_qjson_object(options))
            .map_err(|e| CoreError::invalid("options", e.to_string()));
        self.dispatch(releases(items).and_then(|items| {
            Ok(Call::DownloadReleasesWith {
                items,
                name: non_empty(name),
                options: options?,
            })
        }))
    }
    pub fn download_movie(&self, link: &QString) -> QJsonObject {
        self.dispatch(Ok(Call::DownloadMovie {
            link: link.to_string(),
        }))
    }
    pub fn list_subscription_rules(&self) -> QJsonObject {
        self.dispatch(Ok(Call::ListSubscriptionRules))
    }
    pub fn list_torrents(&self) -> QJsonObject {
        self.dispatch(Ok(Call::ListTorrents))
    }
    pub fn repair_subscription(&self, name: &QString) -> QJsonObject {
        self.dispatch(Ok(Call::RepairSubscription {
            name: name.to_string(),
        }))
    }
    pub fn read_rule(&self, name: &QString) -> QJsonObject {
        self.dispatch(Ok(Call::ReadRule {
            name: name.to_string(),
        }))
    }
    pub fn read_feed(&self, url: &QString) -> QJsonObject {
        self.dispatch(Ok(Call::ReadFeed {
            url: url.to_string(),
        }))
    }
    pub fn plan_remove_subscription(
        &self,
        name: &QString,
        torrents: bool,
        files: bool,
    ) -> QJsonObject {
        self.dispatch(Ok(Call::PlanRemoveSubscription {
            name: name.to_string(),
            torrents,
            files,
        }))
    }
    pub fn remove_subscription(&self, name: &QString, torrents: bool, files: bool) -> QJsonObject {
        self.dispatch(Ok(Call::RemoveSubscription {
            name: name.to_string(),
            torrents,
            files,
        }))
    }
    pub fn control_torrents(
        &self,
        hashes: &QJsonArray,
        action: &QString,
        files: bool,
    ) -> QJsonObject {
        let hashes: Vec<String> = hashes
            .iter()
            .filter_map(|h| json::from_qjson(&h).as_str().map(String::from))
            .collect();
        let action = match action.to_string().as_str() {
            "Stop" => Ok(TorrentAction::Stop),
            "Start" => Ok(TorrentAction::Start),
            "Delete" => Ok(TorrentAction::Delete { files }),
            "Sequential" => Ok(TorrentAction::Sequential { on: true }),
            "Unsequential" => Ok(TorrentAction::Sequential { on: false }),
            "FirstLast" => Ok(TorrentAction::FirstLastPieces { on: true }),
            "UnFirstLast" => Ok(TorrentAction::FirstLastPieces { on: false }),
            other => Err(CoreError::invalid(
                "action",
                format!("{other} is not a torrent action"),
            )),
        };
        self.dispatch(action.map(|action| Call::ControlTorrents { hashes, action }))
    }
    pub fn remove_show(
        &self,
        title: &QString,
        rule: &QString,
        hashes: &QJsonArray,
        torrents: bool,
        files: bool,
        series: f64,
    ) -> QJsonObject {
        let hashes: Vec<String> = hashes
            .iter()
            .filter_map(|h| json::from_qjson(&h).as_str().map(String::from))
            .collect();
        self.dispatch(Ok(Call::RemoveShow {
            title: title.to_string(),
            rule: non_empty(rule),
            hashes,
            torrents,
            files,
            series: (series >= 0.0).then_some(series as u64),
        }))
    }
    pub fn set_download_settings(
        &self,
        settings: &QJsonObject,
        secrets: &QJsonObject,
    ) -> QJsonObject {
        let secret = |key: &str| -> Option<String> {
            let k = QString::from(key);
            if secrets.contains(&k) {
                Some(secrets.value(&k).to_string().to_string())
            } else {
                None
            }
        };
        self.dispatch(
            parse_object::<DownloadSettings>("settings", settings).map(|settings| {
                Call::SetDownloadSettings {
                    settings,
                    anirss_token: secret("anirss_token"),
                    torrent_password: secret("torrent_password"),
                }
            }),
        )
    }
    pub fn test_torrent_client(&self) -> QJsonObject {
        self.dispatch(Ok(Call::TestTorrentClient))
    }
    pub fn use_torrent_client(&self, id: &QString) -> QJsonObject {
        let id = id.to_string();
        self.dispatch(Ok(Call::UseTorrentClient {
            id: (!id.is_empty()).then_some(id),
        }))
    }
    pub fn call_async(&self, name: &QString, args: &QJsonObject) -> f64 {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let token = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as f64;
        let call = json::call_from(&name.to_string(), from_qjson_object(args))
            .map_err(|e| CoreError::invalid("call", e));
        let qt = self.qt_thread();
        let spawned = std::thread::Builder::new()
            .name("door-call".into())
            .spawn(move || {
                let reply = match call {
                    Ok(c) => json::reply_json(crate::runtime::core().call(c)),
                    Err(e) => json!({ "error": json::error_json(&e) }),
                };
                qt.queue(move |door: Pin<&mut qobject::Door>| {
                    door.call_done(token, to_qjson_object(&reply))
                })
                .ok();
            });
        if spawned.is_err() {
            return -1.0;
        }
        token
    }

    pub fn call(&self, name: &QString, args: &QJsonObject) -> QJsonObject {
        self.dispatch(
            json::call_from(&name.to_string(), from_qjson_object(args))
                .map_err(|e| CoreError::invalid("call", e)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::JobKind;

    fn started(kind: JobKind) -> EventBody {
        EventBody::JobStarted { kind }
    }

    #[test]
    fn a_job_seeded_from_list_jobs_and_one_that_starts_take_the_same_shape() {
        // ListJobs before the job has reported anything: progress is null.
        let seeded = seed_jobs(&json!({"jobs": [
            {"id": 4, "kind": "Scan", "started_at": 1.0, "progress": null},
            {"id": 5, "kind": "Crawl", "started_at": 1.0, "progress": {"done": 3, "total": 9, "label": "one"}},
        ]}));
        assert_eq!(
            seeded[0],
            json!({"id": 4, "kind": "Scan", "done": 0, "total": null, "label": ""}),
            "a job with no progress reads like a job that just started"
        );
        assert_eq!(
            seeded[1],
            json!({"id": 5, "kind": "Crawl", "done": 3, "total": 9, "label": "one"})
        );

        let mut fresh = vec![];
        assert!(apply_job(&mut fresh, 4, false, &started(JobKind::Scan)));
        assert_eq!(fresh, vec![seeded[0].clone()]);
    }

    #[test]
    fn job_started_upserts_so_the_launch_scan_is_never_counted_twice() {
        // Core::start files the launch job before it emits, so ListJobs already has it.
        let mut jobs = seed_jobs(&json!({"jobs": [{"id": 4, "kind": "Scan", "progress": null}]}));
        assert!(
            !apply_job(&mut jobs, 4, false, &started(JobKind::Scan)),
            "the same job started again is not a change"
        );
        assert_eq!(jobs.len(), 1, "one scan, one row");

        // And the progress lands on that one row rather than on a stale first copy.
        let progress = EventBody::JobProgress {
            done: 12,
            total: Some(46),
            label: "Frieren".into(),
        };
        assert!(apply_job(&mut jobs, 4, false, &progress));
        assert_eq!(jobs[0]["done"], 12);
        assert_eq!(jobs[0]["total"], 46);
        assert_eq!(jobs[0]["label"], "Frieren");
    }

    #[test]
    fn a_finished_job_leaves_and_an_unknown_id_changes_nothing() {
        let mut jobs = vec![];
        assert!(apply_job(&mut jobs, 7, false, &started(JobKind::Search)));
        assert!(!apply_job(
            &mut jobs,
            9,
            false,
            &EventBody::JobProgress {
                done: 1,
                total: None,
                label: String::new()
            }
        ));
        assert!(apply_job(&mut jobs, 7, true, &EventBody::JobCancelled));
        assert!(jobs.is_empty());
        assert!(
            !apply_job(&mut jobs, 7, true, &EventBody::JobCancelled),
            "a job that already left is not a change"
        );
    }

    #[test]
    fn a_panic_payload_gives_up_its_message() {
        let caught = std::panic::catch_unwind(|| panic!("the door fell off")).unwrap_err();
        assert_eq!(panic_text(&*caught), "the door fell off");
        let owned = std::panic::catch_unwind(|| panic!("{}", "and again".to_string())).unwrap_err();
        assert_eq!(panic_text(&*owned), "and again");
    }
}
