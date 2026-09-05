use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct JobRef { pub id: u64, pub kind: JobKind, pub phase: JobPhase }

/// Every event is one envelope. `message` is the activity log line, written
/// by the core once; the shell never composes log text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Event {
    pub seq: u64,
    pub at: SystemTime,
    pub level: Level,
    pub stage: Stage,
    pub message: String,
    pub job: Option<JobRef>,
    pub body: EventBody,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum EventBody {
    Ready,
    Notice,
    JobStarted { kind: JobKind },
    JobProgress { done: u64, total: Option<u64>, label: String },
    JobFailed { error: CoreError },
    JobCancelled,
    SourceChanged { source: Source },
    SourceRemoved { source: u64 },
    SeriesChanged { series: Vec<SeriesCard> },
    SeriesRemoved { ids: Vec<u64> },
    ScanFinished { source: Option<u64>, added: u64, changed: u64, removed: u64 },
    SubscriptionsListed { result: SubscriptionsResult },
    SearchFinished { results: Vec<SearchResult> },
    LinkResolved { target: MatchTarget },
    MatchApplied { series: u64 },
    RefreshFinished { refreshed: u64, failed: u64 },
    AutoMatchFinished { backfilled: u64, matched: u64, unmatched: u64 },
    AiringRefreshed { series: u64, updated: bool },
    ImagesCleared { removed: u64 },
    TrackersChanged { state: TrackerState },
    AuthUrlReady { tracker: Tracker, open_url: String, redirect_url: String },
    TrackerConnected { tracker: Tracker, username: String },
    Marked { series: u64, episode: u32, outcomes: Vec<TrackerOutcome> },
    ProgressSet { series: u64, progress: u32, outcomes: Vec<TrackerOutcome> },
    Scored { series: u64, score: Option<f64>, outcomes: Vec<TrackerOutcome> },
    ProgressRefreshed { tracker: Tracker },
    WatchingRefreshed { list: WatchingList },
    GraphChanged { root: u64 },
    CrawlFinished { fetched: u64, deferred: u64 },
    SkipWindowsReady { session: u64, windows: Vec<SkipWindow> },
    ResumePointChanged { file: u64, position: Option<f64> },
    Viewed { series: u64, episode: String },
    PreferencesChanged { preferences: Preferences },
    SettingsChanged,
    ExportFinished { path: String },
    ImportFinished { summary: ImportSummary },
}

/// The listener a shell hands to `subscribe`. Called by value on whatever
/// tokio thread produced the event; the shell owns the hop to its UI thread.
#[uniffi::export(foreign)]
pub trait EventListener: Send + Sync {
    fn on_event(&self, event: Event);
}
