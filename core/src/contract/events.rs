use super::*;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct JobRef {
    pub id: u64,
    pub kind: JobKind,
    pub phase: JobPhase,
}

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
    JobStarted {
        kind: JobKind,
    },
    JobProgress {
        done: u64,
        total: Option<u64>,
        label: String,
    },
    JobFailed {
        error: CoreError,
    },
    JobCancelled,
    SourceChanged {
        source: Source,
    },
    SourceRemoved {
        source: u64,
    },
    SeriesChanged {
        series: Vec<SeriesCard>,
    },
    SeriesRemoved {
        ids: Vec<u64>,
    },
    ScanFinished {
        source: Option<u64>,
        added: u64,
        changed: u64,
        removed: u64,
    },
    SubscriptionsListed {
        result: SubscriptionsResult,
    },
    SearchFinished {
        results: Vec<SearchResult>,
    },
    LinkResolved {
        target: MatchTarget,
    },
    MatchApplied {
        series: u64,
    },
    RefreshFinished {
        refreshed: u64,
        failed: u64,
    },
    AutoMatchFinished {
        backfilled: u64,
        matched: u64,
        unmatched: u64,
    },
    AiringRefreshed {
        series: u64,
        updated: bool,
    },
    ImagesCleared {
        removed: u64,
    },
    TrackersChanged {
        state: TrackerState,
    },
    AuthUrlReady {
        tracker: Tracker,
        open_url: String,
        redirect_url: String,
    },
    TrackerConnected {
        tracker: Tracker,
        username: String,
    },
    Marked {
        series: u64,
        episode: u32,
        outcomes: Vec<TrackerOutcome>,
    },
    ProgressSet {
        series: u64,
        progress: u32,
        outcomes: Vec<TrackerOutcome>,
    },
    Scored {
        series: u64,
        score: Option<f64>,
        outcomes: Vec<TrackerOutcome>,
    },
    ProgressRefreshed {
        tracker: Tracker,
    },
    WatchingRefreshed {
        list: WatchingList,
    },
    GraphChanged {
        root: u64,
    },
    CrawlFinished {
        fetched: u64,
        deferred: u64,
    },
    SkipWindowsReady {
        session: u64,
        windows: Vec<SkipWindow>,
    },
    ResumePointChanged {
        file: u64,
        position: Option<f64>,
    },
    Viewed {
        series: u64,
        episode: String,
    },
    PreferencesChanged {
        preferences: Preferences,
    },
    SettingsChanged,
    ExportFinished {
        path: String,
    },
    ImportFinished {
        summary: ImportSummary,
    },
    // Downloads
    DownloadsStatusChanged {
        status: DownloadsStatus,
    },
    ReleasesFound {
        search: ReleaseSearch,
    },
    ReleasesRefined {
        refinement: Refinement,
    },
    SubscriptionPreviewed {
        spec: SubscriptionSpec,
    },
    Subscribed {
        result: Subscribed,
    },
    TorrentsAdded {
        result: AddedTorrents,
    },
    SubscriptionsRead {
        rules: Vec<SubscriptionRule>,
    },
    TorrentsRead {
        torrents: Vec<Torrent>,
        transfer: TransferRate,
        /// The library series the torrents' `series_id`s name, once each.
        shows: Vec<ShowLink>,
    },
    /// What the endpoint's own feed says about a search's releases (the
    /// date each was posted, the counts a feed carries that anirss drops),
    /// read after the search answered. Joined to the releases by link,
    /// else title.
    ReleasesEnriched {
        query: String,
        endpoint: String,
        entries: Vec<FeedEntry>,
    },
    /// A subscription's feed, read for the page: `url` is the one asked
    /// for, so a late answer for a feed since closed can be told apart.
    FeedRead {
        url: String,
        entries: Vec<FeedEntry>,
    },
    /// A read that is not a job could not be made: which read, and why.
    DownloadsUnavailable {
        what: String,
        error: CoreError,
    },
    RemovalPlanned {
        plans: Vec<RemovePlan>,
    },
    SubscriptionRemoved {
        name: String,
    },
    /// What `RemoveShow` took: the rule when it had one, the torrents it
    /// took out of the client, the paths it moved to the trash.
    ShowRemoved {
        title: String,
        rule: Option<String>,
        hashes: Vec<String>,
        trashed: Vec<String>,
    },
    TorrentsControlled {
        hashes: Vec<String>,
        action: TorrentAction,
    },
}

/// The listener a shell hands to `subscribe`. Called by value on whatever
/// tokio thread produced the event; the shell owns the hop to its UI thread.
#[uniffi::export(foreign)]
pub trait EventListener: Send + Sync {
    fn on_event(&self, event: Event);
}
