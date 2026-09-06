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
use std::sync::Arc;

use anibeam_core::events::Subscription;
use anibeam_core::{
    Call, Chapter, CloseReason, CoreError, Direction, Event, EventBody, EventListener, FeedSort,
    JobPhase, Level, MatchTarget, MetadataFilter, Preferences, Provider, Sort, SubscriptionsResult,
    SubtitleChoice, SubtitleDefaults, Tab, TrackRef, Tracker,
};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QJsonArray, QJsonObject, QString};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::json::{self, from_qjson_object, to_qjson_array, to_qjson_object};

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
        #[qinvokable]
        fn search_provider(self: &Door, query: &QString, limit: i32) -> QJsonObject;
        #[qinvokable]
        fn resolve_link(self: &Door, url: &QString) -> QJsonObject;
        #[qinvokable]
        fn apply_match(self: &Door, series: f64, target: &QJsonObject) -> QJsonObject;
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
        /// Any call by name, the CLI's door.
        #[qinvokable]
        fn call(self: &Door, name: &QString, args: &QJsonObject) -> QJsonObject;
        #[qinvokable]
        fn mark_log_seen(self: Pin<&mut Door>);

        // The envelope and the derived job signal
        #[qsignal]
        fn event(self: Pin<&mut Door>, envelope: QJsonObject);
        #[qsignal]
        fn job_finished(self: Pin<&mut Door>, job: f64, kind: QString, ok: bool);
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
        fn auth_url_ready(
            self: Pin<&mut Door>,
            tracker: QString,
            open_url: QString,
            redirect_url: QString,
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
        .map_err(|_| CoreError::invalid(field, format!("unknown value {s}")))
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

impl qobject::Door {
    fn dispatch(&self, call: Result<Call, CoreError>) -> QJsonObject {
        let v = match call {
            Ok(c) => json::dispatch(crate::runtime::core(), c),
            Err(e) => json!({ "error": json::error_json(&e) }),
        };
        to_qjson_object(&v)
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
            let jobs: Vec<Value> = j["jobs"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|job| {
                    json!({
                        "id": job["id"],
                        "kind": job["kind"],
                        "done": job["progress"]["done"],
                        "total": job["progress"]["total"],
                        "label": job["progress"]["label"],
                    })
                })
                .collect();
            self.as_mut().rust_mut().jobs = jobs.clone();
            self.as_mut()
                .set_running_jobs(to_qjson_array(&Value::Array(jobs)));
        }
    }

    /// On the Qt thread: the shared state, then the envelope, then the body's own signal.
    pub fn receive(mut self: Pin<&mut Self>, event: Event, envelope: Value) {
        let job = event.job.clone();
        let finished = job.as_ref().is_some_and(|j| j.phase == JobPhase::Finished);
        {
            let mut rust = self.as_mut().rust_mut();
            if let Some(j) = &job {
                match (&event.body, finished) {
                    (EventBody::JobStarted { kind }, _) => {
                        rust.jobs.push(json!({
                            "id": j.id,
                            "kind": serde_json::to_value(kind).unwrap_or(Value::Null),
                            "done": 0,
                            "total": null,
                            "label": "",
                        }));
                    }
                    (EventBody::JobProgress { done, total, label }, _) => {
                        if let Some(entry) = rust.jobs.iter_mut().find(|e| e["id"] == j.id) {
                            entry["done"] = json!(done);
                            entry["total"] = json!(total);
                            entry["label"] = json!(label);
                        }
                    }
                    (_, true) => rust.jobs.retain(|e| e["id"] != j.id),
                    _ => {}
                }
            }
        }
        let jobs = self.as_ref().jobs.clone();
        self.as_mut()
            .set_running_jobs(to_qjson_array(&Value::Array(jobs)));
        if event.level >= Level::Info {
            self.as_mut().set_latest_line(to_qjson_object(&envelope));
        }
        if event.level == Level::Error {
            let n = *self.as_ref().unseen_errors() + 1;
            self.as_mut().set_unseen_errors(n);
        }
        self.as_mut().event(to_qjson_object(&envelope));
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
                    .auth_url_ready(s("tracker"), s("open_url"), s("redirect_url"))
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
        self.dispatch(Ok(Call::SearchProvider {
            provider: Provider::Anilist,
            query: query.to_string(),
            limit: limit.max(1) as u32,
        }))
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
        // else is `{ Track: { track: TrackRef } }`.
        let subtitle = if subtitle.contains(&QString::from("off")) {
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
    pub fn call(&self, name: &QString, args: &QJsonObject) -> QJsonObject {
        self.dispatch(
            json::call_from(&name.to_string(), from_qjson_object(args))
                .map_err(|e| CoreError::invalid("call", e)),
        )
    }
}
