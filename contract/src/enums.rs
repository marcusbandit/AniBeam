use serde::{Deserialize, Serialize};

/// `as_str` and `from_column` for an enum whose variants each stand for one
/// stored string. Each pair is written once, so the two directions cannot
/// drift apart, and a string given to two variants fails the build. Exported for the
/// core's own column enums; not part of what a shell uses.
#[doc(hidden)]
#[macro_export]
macro_rules! column_strings {
    ($(#[doc = $doc:literal])* $name:ident { $($variant:ident => $col:literal),+ $(,)? }) => {
        impl $name {
            $(#[doc = $doc])*
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $col,)+
                }
            }

            #[deny(unreachable_patterns)]
            pub fn from_column(s: &str) -> Option<Self> {
                match s {
                    $($col => Some(Self::$variant),)+
                    _ => None,
                }
            }

            /// Every variant reads back from the string it writes.
            #[cfg(test)]
            fn assert_columns_round_trip() {
                $(assert_eq!(Self::from_column(Self::$variant.as_str()), Some(Self::$variant));)+
                assert_eq!(Self::from_column("not a column value"), None);
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Provider {
    Anilist,
    Mal,
    Tmdb,
    Tvmaze,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Tracker {
    Anilist,
    Mal,
}

impl Tracker {
    /// The label a user reads: AniList or MAL.
    pub fn label(self) -> &'static str {
        match self {
            Tracker::Anilist => "AniList",
            Tracker::Mal => "MAL",
        }
    }
}

column_strings!(
    /// The value every `tracker` column holds.
    Tracker {
        Anilist => "anilist",
        Mal => "mal",
    }
);

impl Provider {
    /// The name a user reads: AniList, MAL, TMDB or TVmaze.
    pub fn label(self) -> &'static str {
        match self {
            Provider::Anilist => "AniList",
            Provider::Mal => "MAL",
            Provider::Tmdb => "TMDB",
            Provider::Tvmaze => "TVmaze",
        }
    }
}

column_strings!(Provider {
    Anilist => "anilist",
    Mal => "mal",
    Tmdb => "tmdb",
    Tvmaze => "tvmaze",
});

/// The providers a series can be matched against, in the default order the
/// auto-match tries them. MAL is a tracker and TMDB has no client on the
/// native line, so neither is one. The `provider_order` setting is always
/// a reordering of exactly this list.
pub const MATCH_PROVIDERS: [Provider; 2] = [Provider::Anilist, Provider::Tvmaze];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum SeriesKind {
    Show,
    Movie,
}

column_strings!(SeriesKind {
    Show => "show",
    Movie => "movie",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum TmdbKind {
    Tv,
    Movie,
}

column_strings!(TmdbKind {
    Tv => "tv",
    Movie => "movie",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum AiringStatus {
    Releasing,
    Finished,
    NotYetReleased,
    Cancelled,
    Hiatus,
}

impl AiringStatus {
    /// Applies Electron's `normalizeStatus` (`src/shared/airingStatus.ts`):
    /// trim, lower case, and collapse whitespace and hyphens to underscores
    /// before matching.
    /// AniList's spelling of the status, the vocabulary a stored media
    /// row speaks and `from_provider` reads back.
    pub fn anilist_word(self) -> &'static str {
        match self {
            AiringStatus::Releasing => "RELEASING",
            AiringStatus::Finished => "FINISHED",
            AiringStatus::NotYetReleased => "NOT_YET_RELEASED",
            AiringStatus::Cancelled => "CANCELLED",
            AiringStatus::Hiatus => "HIATUS",
        }
    }

    pub fn from_provider(s: &str) -> Option<AiringStatus> {
        let normalised = normalise_status(s);
        match normalised.as_str() {
            "releasing" | "currently_airing" | "airing" | "ongoing" => {
                Some(AiringStatus::Releasing)
            }
            "finished" | "finished_airing" | "ended" | "completed" => Some(AiringStatus::Finished),
            "not_yet_released" | "not_yet_aired" | "upcoming" | "tba" => {
                Some(AiringStatus::NotYetReleased)
            }
            "cancelled" | "canceled" => Some(AiringStatus::Cancelled),
            "hiatus" | "on_hiatus" => Some(AiringStatus::Hiatus),
            _ => None,
        }
    }
}

fn normalise_status(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_separator = false;
    for c in s.trim().chars() {
        if c.is_whitespace() || c == '-' {
            if !last_was_separator {
                out.push('_');
                last_was_separator = true;
            }
        } else {
            out.push(c.to_ascii_lowercase());
            last_was_separator = false;
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum WatchedState {
    Behind,
    CaughtUp,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum ListStatus {
    Watching,
    Planning,
    Completed,
    Paused,
    Dropped,
    Repeating,
}

column_strings!(ListStatus {
    Watching => "watching",
    Planning => "planning",
    Completed => "completed",
    Paused => "paused",
    Dropped => "dropped",
    Repeating => "repeating",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Tab {
    All,
    Series,
    Movies,
    Watched,
    Hidden,
}

column_strings!(Tab {
    All => "all",
    Series => "series",
    Movies => "movies",
    Watched => "watched",
    Hidden => "hidden",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Sort {
    Alpha,
    LastViewed,
    Progress,
    CommunityScore,
    MyScore,
    /// The newest file's arrival, newest first by default.
    Downloaded,
}

column_strings!(
    /// The export's own keys, camel cased.
    Sort {
        Alpha => "alpha",
        LastViewed => "lastViewed",
        Progress => "progress",
        CommunityScore => "communityScore",
        MyScore => "myScore",
        Downloaded => "downloaded",
    }
);

/// How the library page arranges its cards: one box per franchise (the
/// default), by type, by genre, by watch state, or a flat grid. `Flat` is
/// "None" on the wire, which keeps it clear of `Option::None` here and in
/// every binding.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum,
)]
pub enum Arrange {
    Related,
    Type,
    Genre,
    Season,
    WatchState,
    /// No grouping, the fresh-install default
    #[serde(rename = "None")]
    #[default]
    Flat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Direction {
    Asc,
    Desc,
}

column_strings!(Direction {
    Asc => "asc",
    Desc => "desc",
});

/// Which shows Airing now lists: everything on air today, whatever season
/// it started in, the season after the current one, or any season by name
/// and year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum SeasonPick {
    Now,
    Next,
    Season { name: SeasonName, year: i32 },
}

/// One of AniList's four quarters, in calendar order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum SeasonName {
    Winter,
    Spring,
    Summer,
    Fall,
}

/// Recently released, coming soon; exports as `recent` | `upcoming`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum FeedSort {
    Recent,
    Upcoming,
}

column_strings!(FeedSort {
    Recent => "recent",
    Upcoming => "upcoming",
});

/// Which recommended shows Home's row holds (#779). `All` is every show the
/// recommender ranks that the user has not watched; `InLibrary` only those
/// the library has files for; `OnList` only those on a tracker list, Planning
/// or barely started, whether the library has them or not.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum,
)]
pub enum RecommendedScope {
    #[default]
    All,
    InLibrary,
    OnList,
}

column_strings!(RecommendedScope {
    All => "All",
    InLibrary => "InLibrary",
    OnList => "OnList",
});

/// Where the recommender stands, which is what an empty row says. `Ready`
/// is a fit to read; `NeverFitted` has not run yet; `NoList` found no list
/// to learn from; `NoScores` found a list with nothing rated on it.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum,
)]
pub enum RecommendedState {
    #[default]
    NeverFitted,
    Ready,
    NoList,
    NoScores,
}

column_strings!(RecommendedState {
    NeverFitted => "NeverFitted",
    Ready => "Ready",
    NoList => "NoList",
    NoScores => "NoScores",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum MetadataFilter {
    All,
    Series,
    Movies,
    MissingFiles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum ExtraKind {
    Op,
    Ed,
    Pv,
    Sp,
    Other,
}

column_strings!(ExtraKind {
    Op => "op",
    Ed => "ed",
    Pv => "pv",
    Sp => "sp",
    Other => "other",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum SkipKind {
    Intro,
    Outro,
}

/// What a member of a watch together room does to everyone's player (#846): `Play` and
/// `Pause` carry the position too, `Seek` keeps whether the room plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum RoomAction {
    Play,
    Pause,
    Seek,
}

/// Whether a member of a room has the episode open: `Ready` once their player is
/// attached, `Missing` when their library lacks it, `Loading` in between.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum RoomReady {
    Ready,
    Missing,
    Loading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum SkipSource {
    Chapters,
    AniSkip,
}

column_strings!(SkipSource {
    Chapters => "chapters",
    AniSkip => "aniskip",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum TitleLanguage {
    Romaji,
    English,
    /// The title in its own script: kana and kanji for Japanese shows
    Native,
}

column_strings!(TitleLanguage {
    Romaji => "romaji",
    English => "english",
    Native => "native",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum AssOverride {
    AsScripted,
    ScaleOnly,
    Force,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum TrackKind {
    Embedded,
    Sidecar,
}

/// The kind of device an episode is cast to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum CastKind {
    Chromecast,
    Dlna,
}

/// Where a cast session is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum CastPlayState {
    /// Reaching the device and reading the episode.
    Connecting,
    Buffering,
    Playing,
    Paused,
    /// The episode played to its end.
    Ended,
    /// It stopped with `error` saying why.
    Failed,
    /// The session was stopped from here or on the device.
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum CloseReason {
    Ended,
    Stopped,
    Switched,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, uniffi::Enum,
)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

column_strings!(Level {
    Debug => "debug",
    Info => "info",
    Warn => "warn",
    Error => "error",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Stage {
    Library,
    Metadata,
    Trackers,
    Franchise,
    Playback,
    Store,
    System,
    Downloads,
}

column_strings!(Stage {
    Library => "library",
    Metadata => "metadata",
    Trackers => "trackers",
    Franchise => "franchise",
    Playback => "playback",
    Store => "store",
    System => "system",
    Downloads => "downloads",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum JobPhase {
    Started,
    Running,
    Finished,
}

column_strings!(JobPhase {
    Started => "Started",
    Running => "Running",
    Finished => "Finished",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum JobKind {
    Scan,
    AutoMatch,
    Search,
    ResolveLink,
    ApplyMatch,
    Refresh,
    RefreshAll,
    RefreshAiring,
    RefreshImdb,
    ClearImages,
    FillImages,
    ConnectTracker,
    Mark,
    SetProgress,
    Score,
    RefreshProgress,
    RefreshWatching,
    /// Airing now's read of one season from AniList.
    RefreshAiringNow,
    /// The recommender's read of what it is missing from AniList, and its fit (#779).
    RefreshRecommended,
    /// The show page's one read of a show the library does not have (#779).
    ReadShowPage,
    /// The sweep that moves Watching entries left untouched to On hold.
    OnHold,
    /// One series' entries to a list status, as a play does to a series on
    /// hold.
    SetStatus,
    /// The list kept on this device, moved up to a tracker (#580).
    MoveLists,
    /// One series' notes, to every connected tracker (#211).
    Notes,
    Crawl,
    SkipWindows,
    Export,
    Import,
    /// No call starts it any more (#851); kept so the activity log still reads
    /// the jobs it logged before.
    Subscriptions,
    ConnectDownloads,
    SearchReleases,
    RefineReleases,
    Subscribe,
    Download,
    /// No call starts it any more (#851); kept so the activity log still reads
    /// the jobs it logged before.
    RemoveSubscription,
    ControlTorrents,
    TestTorrentClient,
    RemoveShow,
    RepairSubscription,
    CheckEndpoints,
    StartEndpoint,
    /// A coding agent's session on one show's schedule (#419).
    ScheduleAgent,
}

impl JobKind {
    pub fn stage(self) -> Stage {
        use JobKind::*;
        match self {
            Scan | Subscriptions => Stage::Library,
            AutoMatch | Search | ResolveLink | ApplyMatch | Refresh | RefreshAll
            | RefreshAiring | RefreshImdb | ClearImages | FillImages | RefreshAiringNow
            | RefreshRecommended | ReadShowPage | ScheduleAgent => Stage::Metadata,
            ConnectTracker | Mark | SetProgress | Score | RefreshProgress | RefreshWatching
            | OnHold | SetStatus | MoveLists | Notes => Stage::Trackers,
            Crawl => Stage::Franchise,
            SkipWindows => Stage::Playback,
            Export | Import => Stage::Store,
            ConnectDownloads | SearchReleases | RefineReleases | Subscribe | Download
            | RemoveSubscription | ControlTorrents | TestTorrentClient | RemoveShow
            | RepairSubscription | CheckEndpoints | StartEndpoint => Stage::Downloads,
        }
    }

    /// The kind's message in the catalog. Its value is [`JobKind::doing`]; its `.stopped`
    /// attribute is the plain line for a job of the kind that ended in a problem.
    pub fn words_id(self) -> &'static str {
        use JobKind::*;
        match self {
            Scan => "core-jobs-scan",
            AutoMatch => "core-jobs-auto-match",
            Search => "core-jobs-search",
            ResolveLink => "core-jobs-resolve-link",
            ApplyMatch => "core-jobs-apply-match",
            Refresh => "core-jobs-refresh",
            RefreshAll => "core-jobs-refresh-all",
            RefreshAiring => "core-jobs-refresh-airing",
            RefreshImdb => "core-jobs-refresh-imdb",
            ClearImages => "core-jobs-clear-images",
            FillImages => "core-jobs-fill-images",
            ConnectTracker => "core-jobs-connect-tracker",
            Mark => "core-jobs-mark",
            SetProgress => "core-jobs-set-progress",
            Score => "core-jobs-score",
            RefreshProgress => "core-jobs-refresh-progress",
            RefreshWatching => "core-jobs-refresh-watching",
            RefreshAiringNow => "core-jobs-refresh-airing-now",
            RefreshRecommended => "core-jobs-refresh-recommended",
            ReadShowPage => "core-jobs-read-show-page",
            OnHold => "core-jobs-on-hold",
            SetStatus => "core-jobs-set-status",
            MoveLists => "core-jobs-move-lists",
            Notes => "core-jobs-notes",
            Crawl => "core-jobs-crawl",
            SkipWindows => "core-jobs-skip-windows",
            Export => "core-jobs-export",
            Import => "core-jobs-import",
            Subscriptions => "core-jobs-subscriptions",
            ConnectDownloads => "core-jobs-connect-downloads",
            SearchReleases => "core-jobs-search-releases",
            RefineReleases => "core-jobs-refine-releases",
            Subscribe => "core-jobs-subscribe",
            Download => "core-jobs-download",
            RemoveSubscription => "core-jobs-remove-subscription",
            ControlTorrents => "core-jobs-control-torrents",
            TestTorrentClient => "core-jobs-test-torrent-client",
            RemoveShow => "core-jobs-remove-show",
            RepairSubscription => "core-jobs-repair-subscription",
            CheckEndpoints => "core-jobs-check-endpoints",
            StartEndpoint => "core-jobs-start-endpoint",
            ScheduleAgent => "core-jobs-schedule-agent",
        }
    }

    /// The kind as plain words for a status line while it runs: no variant
    /// names, no provider names beyond the two trackers a person connected.
    pub fn doing(self) -> String {
        crate::i18n::tr(self.words_id(), &[])
    }

    /// Kinds the app starts on its own, on a timer or after another job,
    /// rather than on something the person did. Their failures stay in the
    /// developer log: the next run retries, and the person has nothing to do.
    pub fn chore(self) -> bool {
        use JobKind::*;
        match self {
            RefreshAiring | RefreshImdb | FillImages | RefreshProgress | RefreshWatching
            | RefreshAiringNow | RefreshRecommended | ReadShowPage | OnHold | Crawl
            | SkipWindows | Subscriptions | ConnectDownloads | CheckEndpoints => true,
            Scan | AutoMatch | Search | ResolveLink | ApplyMatch | Refresh | RefreshAll
            | ClearImages | ConnectTracker | Mark | SetProgress | Score | SetStatus | MoveLists
            | Notes | Export | Import | SearchReleases | RefineReleases | Subscribe | Download
            | RemoveSubscription | ControlTorrents | TestTorrentClient | RemoveShow
            | RepairSubscription | StartEndpoint | ScheduleAgent => false,
        }
    }

    /// Kinds that run one at a time: a second call replies Started with the running id.
    pub fn one_at_a_time(self) -> bool {
        use JobKind::*;
        matches!(
            self,
            Scan | AutoMatch
                | RefreshAll
                | RefreshImdb
                | Crawl
                | RefreshProgress
                | RefreshWatching
                | RefreshRecommended
                | OnHold
                | FillImages
                | Subscriptions
                | ConnectDownloads
                | TestTorrentClient
                | CheckEndpoints
                | StartEndpoint
        )
    }
}

column_strings!(
    /// The variant name, verbatim.
    JobKind {
        Scan => "Scan",
        AutoMatch => "AutoMatch",
        Search => "Search",
        ResolveLink => "ResolveLink",
        ApplyMatch => "ApplyMatch",
        Refresh => "Refresh",
        RefreshAll => "RefreshAll",
        RefreshAiring => "RefreshAiring",
        RefreshImdb => "RefreshImdb",
        ClearImages => "ClearImages",
        FillImages => "FillImages",
        ConnectTracker => "ConnectTracker",
        Mark => "Mark",
        SetProgress => "SetProgress",
        Score => "Score",
        RefreshProgress => "RefreshProgress",
        RefreshWatching => "RefreshWatching",
        RefreshAiringNow => "RefreshAiringNow",
        RefreshRecommended => "RefreshRecommended",
        ReadShowPage => "ReadShowPage",
        OnHold => "OnHold",
        SetStatus => "SetStatus",
        MoveLists => "MoveLists",
        Notes => "Notes",
        Crawl => "Crawl",
        SkipWindows => "SkipWindows",
        Export => "Export",
        Import => "Import",
        Subscriptions => "Subscriptions",
        ConnectDownloads => "ConnectDownloads",
        SearchReleases => "SearchReleases",
        RefineReleases => "RefineReleases",
        Subscribe => "Subscribe",
        Download => "Download",
        RemoveSubscription => "RemoveSubscription",
        ControlTorrents => "ControlTorrents",
        TestTorrentClient => "TestTorrentClient",
        RemoveShow => "RemoveShow",
        RepairSubscription => "RepairSubscription",
        CheckEndpoints => "CheckEndpoints",
        StartEndpoint => "StartEndpoint",
        ScheduleAgent => "ScheduleAgent",
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_enum_round_trips() {
        Tracker::assert_columns_round_trip();
        Provider::assert_columns_round_trip();
        SeriesKind::assert_columns_round_trip();
        TmdbKind::assert_columns_round_trip();
        ListStatus::assert_columns_round_trip();
        Tab::assert_columns_round_trip();
        Sort::assert_columns_round_trip();
        Direction::assert_columns_round_trip();
        FeedSort::assert_columns_round_trip();
        RecommendedScope::assert_columns_round_trip();
        RecommendedState::assert_columns_round_trip();
        ExtraKind::assert_columns_round_trip();
        SkipSource::assert_columns_round_trip();
        TitleLanguage::assert_columns_round_trip();
        Level::assert_columns_round_trip();
        Stage::assert_columns_round_trip();
        JobPhase::assert_columns_round_trip();
        JobKind::assert_columns_round_trip();
    }
}
