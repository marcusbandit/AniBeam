use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum MatchTarget {
    Anilist { id: u64, season: Option<u32> },
    Mal { id: u64 },
    Tvmaze { id: u64 },
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
    /// The library page in one read: `ListSeries`'s answer, the airing
    /// rail (`ListAiring` with no limit) and every tab's count, all off one
    /// snapshot. `reveal_hidden` is the shell's tab visibility, as on
    /// `ListSeries`.
    ListLibrary {
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
    /// `client_secret`: absent keeps the stored secret, empty removes it,
    /// anything else replaces it.
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
    /// Starts the AniList watching refresh when the cached list is older
    /// than five minutes, or always with `force`. `Started` with the job
    /// (the running one if a refresh is under way), or `Ok` when the cache
    /// is fresh or AniList is not connected. The job ends in
    /// `WatchingRefreshed`.
    RefreshWatching {
        force: bool,
    },
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
    /// The tailnet host's state as last known: off, waiting for Tailscale,
    /// or serving and on which URLs. Never touches the network; changes
    /// arrive as `HostChanged`.
    GetHost,
    GetSettings,
    SetSubtitleDefaults {
        defaults: SubtitleDefaults,
    },
    SetAutoSkip {
        intro: bool,
        outro: bool,
    },
    SetProviderOrder {
        order: Vec<Provider>,
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
    // Downloads (anirss and the torrent client)
    /// The status as last known, settings and endpoints included. Never
    /// touches the network; `ConnectDownloads` is what moves it.
    GetDownloads,
    /// Starts anirss when it is the core's sidecar, reads its health,
    /// endpoints and qBittorrent view, and tries the torrent client
    /// directly. Ends in `DownloadsStatusChanged`.
    ConnectDownloads,
    /// Searches an endpoint (the first one when `None`), falling through
    /// to the others when it is empty or unreachable, and analyses the
    /// result. Ends in `ReleasesFound`.
    SearchReleases {
        query: String,
        endpoint: Option<String>,
    },
    /// One refine step over the state the shell holds. Ends in
    /// `ReleasesRefined`.
    RefineReleases {
        query: String,
        endpoint: String,
        items: Vec<Release>,
        op: RefineOp,
    },
    /// What subscribing to this state would register, without doing it.
    /// Ends in `SubscriptionPreviewed`.
    PreviewSubscription {
        query: String,
        endpoint: String,
        items: Vec<Release>,
    },
    /// Registers an RSS feed and an auto-download rule in qBittorrent.
    /// `name` defaults to the show name anirss parses off the first item.
    /// `wire_query` is the query the endpoint itself was last asked, when
    /// the page narrowed past it on anirss's side: the subscription then
    /// reads that query's feed and keeps the narrowing words in its rule.
    /// Ends in `Subscribed`.
    Subscribe {
        query: String,
        endpoint: String,
        items: Vec<Release>,
        name: Option<String>,
        #[serde(default)]
        wire_query: Option<String>,
    },
    /// Hands these releases to qBittorrent now, under `<save_base>/<name>`.
    /// Ends in `TorrentsAdded`.
    DownloadReleases {
        items: Vec<Release>,
        name: Option<String>,
    },
    /// `DownloadReleases` with options: download in order, and episodes the
    /// new torrents replace (see `downloads::replace`). Ends in
    /// `TorrentsAdded`.
    DownloadReleasesWith {
        items: Vec<Release>,
        name: Option<String>,
        options: DownloadOptions,
    },
    /// One link into the movie folder, no subdirectory. Ends in
    /// `TorrentsAdded`.
    DownloadMovie {
        link: String,
    },
    /// Reads the subscriptions from anirss. Replies `Ok` at once; the list
    /// arrives as `SubscriptionsRead`, or `DownloadsUnavailable` when it
    /// could not be read. Not a job: the page asks on open and after every
    /// change, and a read is not work worth a line in the log.
    ListSubscriptionRules,
    /// Reads every torrent and the transfer totals from the torrent client
    /// directly. Replies `Ok` at once; the list arrives as `TorrentsRead`,
    /// or `DownloadsUnavailable`. Not a job, for the same reason: the page
    /// polls this while it is open.
    ListTorrents,
    /// Reads a subscription's feed itself (the RSS anirss registered), so
    /// the page can show what it holds. Replies `Ok` at once; the entries
    /// arrive as `FeedRead`, a failure as `DownloadsUnavailable` with
    /// `what` set to `feed`. Not a job: opening a show reads it.
    ReadFeed {
        url: String,
    },
    /// Mends a subscription whose feed finds nothing because it asks a
    /// generic RSS endpoint for more words than the endpoint can match:
    /// drops words from the end of the feed's search until the endpoint
    /// answers, and keeps the dropped words in the rule. Ends in
    /// `SubscriptionRepaired`.
    RepairSubscription {
        name: String,
    },
    /// Reads a subscription's rule from the torrent client, with the
    /// articles it accepts. Replies `Ok` at once; ends in `RuleRead`, or
    /// `DownloadsUnavailable` with `what` set to `rule`. Not a job.
    ReadRule {
        name: String,
    },
    /// What removing a subscription would take with it. Ends in
    /// `RemovalPlanned`; nothing changes.
    PlanRemoveSubscription {
        name: String,
        torrents: bool,
        files: bool,
    },
    /// Removes the feed and rule, and with `torrents` their torrents, and
    /// with `files` their data on disk. Ends in `SubscriptionRemoved`.
    RemoveSubscription {
        name: String,
        torrents: bool,
        files: bool,
    },
    /// Takes a show out of downloads in one go, as far as asked: the
    /// subscription (`rule`: its feed and download rule), with `torrents`
    /// the client's torrents for it (`hashes`), and with `files` what they
    /// downloaded and the library series' folder (`series`), moved to the
    /// trash rather than deleted. `files` implies `torrents`. Every file
    /// target is checked before anything is removed: a library root, the
    /// home folder or anything holding one is refused and nothing changes.
    /// `title` names the show in the log. Ends in `ShowRemoved`.
    RemoveShow {
        title: String,
        rule: Option<String>,
        hashes: Vec<String>,
        torrents: bool,
        files: bool,
        series: Option<u64>,
    },
    /// Stops, starts or deletes torrents by info-hash. Ends in
    /// `TorrentsControlled`.
    ControlTorrents {
        hashes: Vec<String>,
        action: TorrentAction,
    },
    /// Saves the downloads settings. The two secrets have three answers
    /// each: `None` keeps what is stored, `Some("")` removes it, anything
    /// else replaces it. A running sidecar is stopped, since its
    /// environment is built from these; `ConnectDownloads` starts it
    /// again. Emits `DownloadsStatusChanged`.
    SetDownloadSettings {
        settings: DownloadSettings,
        anirss_token: Option<String>,
        torrent_password: Option<String>,
    },
    /// Tries the torrent client with the saved settings. Ends in
    /// `DownloadsStatusChanged`.
    TestTorrentClient,
    /// Switches the downloads to another torrent client: the saved client
    /// `id`, or AniBeam's own when `id` is `None`. A running sidecar is
    /// stopped, as a save stops it; `ConnectDownloads` starts it again on
    /// the new client. Emits `DownloadsStatusChanged`.
    UseTorrentClient {
        id: Option<String>,
    },
    /// Asks every on-demand endpoint's ready URL whether it is up, and
    /// nothing else: it never starts one. Ends in `DownloadsStatusChanged`.
    CheckEndpoints,
    /// Starts the service behind the named on-demand endpoint (Jackett for
    /// 1337x) with the saved command, waits for its ready URL to answer and
    /// probes it again. Only the shell's confirmed click makes this call.
    /// Ends in `DownloadsStatusChanged`, or fails with a `Service` error
    /// whose code is `missing`, `failed` or `timeout`.
    StartEndpoint {
        name: String,
    },
    /// Reads the `.torrent` behind a release's link and counts the episodes
    /// in its file list, for a batch whose title names no range. Not a job:
    /// replies `Ok`, the count arrives as `ReleaseInspected`.
    InspectRelease {
        link: String,
    },
    // The phone's own downloads: search and a torrent engine inside the
    // core, no anirss and no qBittorrent. Built with the `phone` feature;
    // without it each answers `Unsupported`. Not jobs: they block the
    // calling thread, so a shell calls them off its UI thread.
    /// The sources the phone searches, in order.
    PhoneSources,
    SetPhoneSources {
        sources: Vec<PhoneSource>,
    },
    /// The desktop's search on the phone: one source (else the first) with fallback
    /// to the others, auto-resolution, the refine groups and best fit, as
    /// `SearchReleases` gives them, from the core's own port of anirss.
    PhoneSearch {
        query: String,
        endpoint: Option<String>,
        /// Try the other sources when this one has nothing; the show view asks each source
        /// on its own and turns this off.
        fallback: bool,
    },
    /// One refine step, as `RefineReleases`.
    PhoneRefine {
        query: String,
        endpoint: String,
        items: Vec<Release>,
        op: RefineOp,
    },
    /// Downloads releases into the library folder under `name`, else the show's own
    /// name. `limit_bytes`, when set, refuses what would take the library past it.
    PhoneDownload {
        items: Vec<Release>,
        name: Option<String>,
        limit_bytes: Option<u64>,
    },
    /// Subscribes to a search, as `Subscribe`: its feed and rule are kept by the core,
    /// which checks the feed and downloads each new episode once.
    PhoneSubscribe {
        query: String,
        endpoint: String,
        items: Vec<Release>,
        name: Option<String>,
    },
    PhoneRules,
    /// Takes subscriptions out; with `files`, their downloads and episodes too.
    PhoneUnsubscribe {
        names: Vec<String>,
        files: bool,
    },
    PhoneTorrents,
    PhoneControl {
        hashes: Vec<String>,
        action: PhoneAction,
    },
    /// Holds every download (no Wi-Fi and no consent for mobile data), or
    /// lets them run again.
    SetPhoneHeld {
        held: bool,
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
    Library {
        series: Vec<SeriesCard>,
        airing: Vec<SeriesCard>,
        counts: TabCounts,
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
    Host {
        status: HostStatus,
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
    Downloads {
        status: DownloadsStatus,
    },
    PhoneSources {
        sources: Vec<PhoneSource>,
    },
    PhoneSearch {
        search: ReleaseSearch,
    },
    PhoneRefined {
        refinement: Refinement,
    },
    PhoneSubscribed {
        subscribed: Subscribed,
    },
    PhoneRules {
        rules: Vec<SubscriptionRule>,
    },
    PhoneTorrents {
        torrents: Vec<Torrent>,
        held: bool,
    },
}
