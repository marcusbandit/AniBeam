use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum MatchTarget {
    Anilist { id: u64, season: Option<u32> },
    Mal { id: u64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum Call {
    // Library
    ListSources,
    AddSource {
        path: String,
    },
    RemoveSource {
        source: u64,
    },
    ForgetSeries {
        series: u64,
    },
    Scan {
        source: Option<u64>,
    },
    RescanSeries {
        series: u64,
    },
    Lookup {
        path: String,
    },
    ListSeries {
        tab: Tab,
        query: String,
        sort: Sort,
        direction: Direction,
        reveal_hidden: bool,
    },
    ListAiring {
        offset: u64,
        limit: u64,
    },
    GetSeries {
        series: u64,
    },
    SetHidden {
        series: u64,
        hidden: bool,
    },
    ListFeed {
        sort: FeedSort,
    },
    ListMetadata {
        filter: MetadataFilter,
        query: String,
        reveal_hidden: bool,
    },
    ListSubscriptions,
    // Metadata
    SearchProvider {
        provider: Provider,
        query: String,
        limit: u32,
    },
    ResolveLink {
        url: String,
    },
    ApplyMatch {
        series: u64,
        target: MatchTarget,
    },
    ClearMatch {
        series: u64,
    },
    RefreshSeries {
        series: u64,
    },
    RefreshAll,
    AutoMatch,
    RefreshAiring {
        series: u64,
    },
    GetStorage,
    ClearImages,
    // Trackers
    GetTrackers,
    SetTrackerCredentials {
        tracker: Tracker,
        client_id: String,
        client_secret: Option<String>,
    },
    ConnectTracker {
        tracker: Tracker,
    },
    DisconnectTracker {
        tracker: Tracker,
    },
    SetMainTracker {
        tracker: Tracker,
    },
    MarkEpisode {
        series: u64,
        episode: f64,
    },
    SetProgress {
        series: u64,
        progress: u32,
    },
    SetScore {
        series: u64,
        score: Option<f64>,
    },
    RefreshProgress {
        tracker: Option<Tracker>,
    },
    ListWatching,
    // Franchise
    GetFranchiseGraph {
        series: u64,
    },
    // Playback
    OpenPlayback {
        file: u64,
    },
    ReportChapters {
        session: u64,
        chapters: Vec<Chapter>,
        duration: f64,
    },
    Tick {
        session: u64,
        position: f64,
        paused: bool,
    },
    ClosePlayback {
        session: u64,
        position: f64,
        reason: CloseReason,
    },
    SetTrackChoice {
        series: u64,
        audio: Option<TrackRef>,
        subtitle: Option<SubtitleChoice>,
    },
    // Store
    About,
    GetPreferences,
    SetPreferences {
        preferences: Preferences,
    },
    GetSettings,
    SetSubtitleDefaults {
        defaults: SubtitleDefaults,
    },
    SetAutoSkip {
        intro: bool,
        outro: bool,
    },
    Export {
        path: String,
        private: bool,
    },
    Import {
        path: String,
    },
    RecentEvents {
        limit: u64,
    },
    ClearEvents,
    ListJobs,
    CancelJob {
        job: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum Reply {
    Ok,
    Started {
        job: u64,
    },
    Sources {
        sources: Vec<Source>,
    },
    Source {
        source: Source,
    },
    Lookup {
        series: Option<u64>,
        file: Option<u64>,
    },
    Series {
        series: Vec<SeriesCard>,
    },
    SeriesDetail {
        detail: Box<SeriesDetail>,
    },
    Feed {
        cards: Vec<FeedCard>,
    },
    Metadata {
        rows: Vec<MetadataRow>,
        counts: FilterCounts,
    },
    Storage {
        image_count: u64,
        image_bytes: u64,
    },
    Trackers {
        state: TrackerState,
    },
    Watching {
        list: WatchingList,
        refreshing: Option<u64>,
    },
    Graph {
        layout: Option<FranchiseLayout>,
    },
    Playback {
        session: Box<PlaybackSession>,
    },
    About {
        about: About,
    },
    Preferences {
        preferences: Preferences,
    },
    Settings {
        settings: Settings,
    },
    Events {
        events: Vec<Event>,
    },
    Jobs {
        jobs: Vec<JobInfo>,
    },
}
