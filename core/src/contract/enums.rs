use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Provider {
    Anilist,
    Mal,
    Tmdb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Tracker {
    Anilist,
    Mal,
}

impl Tracker {
    /// The value every `tracker` column holds.
    pub fn as_str(self) -> &'static str {
        match self {
            Tracker::Anilist => "anilist",
            Tracker::Mal => "mal",
        }
    }

    pub fn from_column(s: &str) -> Option<Tracker> {
        match s {
            "anilist" => Some(Tracker::Anilist),
            "mal" => Some(Tracker::Mal),
            _ => None,
        }
    }

    /// The label a user reads: AniList or MAL.
    pub fn label(self) -> &'static str {
        match self {
            Tracker::Anilist => "AniList",
            Tracker::Mal => "MAL",
        }
    }
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Anilist => "anilist",
            Provider::Mal => "mal",
            Provider::Tmdb => "tmdb",
        }
    }

    pub fn from_column(s: &str) -> Option<Provider> {
        match s {
            "anilist" => Some(Provider::Anilist),
            "mal" => Some(Provider::Mal),
            "tmdb" => Some(Provider::Tmdb),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum SeriesKind {
    Show,
    Movie,
}

impl SeriesKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SeriesKind::Show => "show",
            SeriesKind::Movie => "movie",
        }
    }

    pub fn from_column(s: &str) -> Option<SeriesKind> {
        match s {
            "show" => Some(SeriesKind::Show),
            "movie" => Some(SeriesKind::Movie),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum TmdbKind {
    Tv,
    Movie,
}

impl TmdbKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TmdbKind::Tv => "tv",
            TmdbKind::Movie => "movie",
        }
    }

    pub fn from_column(s: &str) -> Option<TmdbKind> {
        match s {
            "tv" => Some(TmdbKind::Tv),
            "movie" => Some(TmdbKind::Movie),
            _ => None,
        }
    }
}

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
    pub fn from_provider(s: &str) -> Option<AiringStatus> {
        let normalised = normalise_status(s);
        match normalised.as_str() {
            "releasing" | "currently_airing" | "airing" | "ongoing" => Some(AiringStatus::Releasing),
            "finished" | "finished_airing" | "ended" | "completed" => Some(AiringStatus::Finished),
            "not_yet_released" | "not_yet_aired" | "upcoming" | "tba" => Some(AiringStatus::NotYetReleased),
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

impl ListStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ListStatus::Watching => "watching",
            ListStatus::Planning => "planning",
            ListStatus::Completed => "completed",
            ListStatus::Paused => "paused",
            ListStatus::Dropped => "dropped",
            ListStatus::Repeating => "repeating",
        }
    }

    pub fn from_column(s: &str) -> Option<ListStatus> {
        match s {
            "watching" => Some(ListStatus::Watching),
            "planning" => Some(ListStatus::Planning),
            "completed" => Some(ListStatus::Completed),
            "paused" => Some(ListStatus::Paused),
            "dropped" => Some(ListStatus::Dropped),
            "repeating" => Some(ListStatus::Repeating),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Tab {
    All,
    Series,
    Movies,
    Hidden,
}

impl Tab {
    pub fn as_str(self) -> &'static str {
        match self {
            Tab::All => "all",
            Tab::Series => "series",
            Tab::Movies => "movies",
            Tab::Hidden => "hidden",
        }
    }

    pub fn from_column(s: &str) -> Option<Tab> {
        match s {
            "all" => Some(Tab::All),
            "series" => Some(Tab::Series),
            "movies" => Some(Tab::Movies),
            "hidden" => Some(Tab::Hidden),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Sort {
    Alpha,
    LastViewed,
    Progress,
    CommunityScore,
    MyScore,
}

impl Sort {
    /// The export's own keys, camel cased.
    pub fn as_str(self) -> &'static str {
        match self {
            Sort::Alpha => "alpha",
            Sort::LastViewed => "lastViewed",
            Sort::Progress => "progress",
            Sort::CommunityScore => "communityScore",
            Sort::MyScore => "myScore",
        }
    }

    pub fn from_column(s: &str) -> Option<Sort> {
        match s {
            "alpha" => Some(Sort::Alpha),
            "lastViewed" => Some(Sort::LastViewed),
            "progress" => Some(Sort::Progress),
            "communityScore" => Some(Sort::CommunityScore),
            "myScore" => Some(Sort::MyScore),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Direction {
    Asc,
    Desc,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Asc => "asc",
            Direction::Desc => "desc",
        }
    }

    pub fn from_column(s: &str) -> Option<Direction> {
        match s {
            "asc" => Some(Direction::Asc),
            "desc" => Some(Direction::Desc),
            _ => None,
        }
    }
}

/// Recently released, coming soon; exports as `recent` | `upcoming`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum FeedSort {
    Recent,
    Upcoming,
}

impl FeedSort {
    pub fn as_str(self) -> &'static str {
        match self {
            FeedSort::Recent => "recent",
            FeedSort::Upcoming => "upcoming",
        }
    }

    pub fn from_column(s: &str) -> Option<FeedSort> {
        match s {
            "recent" => Some(FeedSort::Recent),
            "upcoming" => Some(FeedSort::Upcoming),
            _ => None,
        }
    }
}

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

impl ExtraKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ExtraKind::Op => "op",
            ExtraKind::Ed => "ed",
            ExtraKind::Pv => "pv",
            ExtraKind::Sp => "sp",
            ExtraKind::Other => "other",
        }
    }

    pub fn from_column(s: &str) -> Option<ExtraKind> {
        match s {
            "op" => Some(ExtraKind::Op),
            "ed" => Some(ExtraKind::Ed),
            "pv" => Some(ExtraKind::Pv),
            "sp" => Some(ExtraKind::Sp),
            "other" => Some(ExtraKind::Other),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum SkipKind {
    Intro,
    Outro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum SkipSource {
    Chapters,
    AniSkip,
}

impl SkipSource {
    pub fn as_str(self) -> &'static str {
        match self {
            SkipSource::Chapters => "chapters",
            SkipSource::AniSkip => "aniskip",
        }
    }

    pub fn from_column(s: &str) -> Option<SkipSource> {
        match s {
            "chapters" => Some(SkipSource::Chapters),
            "aniskip" => Some(SkipSource::AniSkip),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum TitleLanguage {
    Romaji,
    English,
}

impl TitleLanguage {
    pub fn as_str(self) -> &'static str {
        match self {
            TitleLanguage::Romaji => "romaji",
            TitleLanguage::English => "english",
        }
    }

    pub fn from_column(s: &str) -> Option<TitleLanguage> {
        match s {
            "romaji" => Some(TitleLanguage::Romaji),
            "english" => Some(TitleLanguage::English),
            _ => None,
        }
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum CloseReason {
    Ended,
    Stopped,
    Switched,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, uniffi::Enum)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
        }
    }

    pub fn from_column(s: &str) -> Option<Level> {
        match s {
            "debug" => Some(Level::Debug),
            "info" => Some(Level::Info),
            "warn" => Some(Level::Warn),
            "error" => Some(Level::Error),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Stage {
    Library,
    Metadata,
    Trackers,
    Franchise,
    Playback,
    Store,
    System,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Library => "library",
            Stage::Metadata => "metadata",
            Stage::Trackers => "trackers",
            Stage::Franchise => "franchise",
            Stage::Playback => "playback",
            Stage::Store => "store",
            Stage::System => "system",
        }
    }

    pub fn from_column(s: &str) -> Option<Stage> {
        match s {
            "library" => Some(Stage::Library),
            "metadata" => Some(Stage::Metadata),
            "trackers" => Some(Stage::Trackers),
            "franchise" => Some(Stage::Franchise),
            "playback" => Some(Stage::Playback),
            "store" => Some(Stage::Store),
            "system" => Some(Stage::System),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum JobPhase {
    Started,
    Running,
    Finished,
}

impl JobPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            JobPhase::Started => "Started",
            JobPhase::Running => "Running",
            JobPhase::Finished => "Finished",
        }
    }

    pub fn from_column(s: &str) -> Option<JobPhase> {
        match s {
            "Started" => Some(JobPhase::Started),
            "Running" => Some(JobPhase::Running),
            "Finished" => Some(JobPhase::Finished),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum JobKind {
    Scan, AutoMatch, Search, ResolveLink, ApplyMatch, Refresh, RefreshAll, RefreshAiring,
    ClearImages, FillImages, ConnectTracker, Mark, SetProgress, Score, RefreshProgress,
    RefreshWatching, Crawl, SkipWindows, Export, Import, Subscriptions,
}

impl JobKind {
    pub fn stage(self) -> Stage {
        use JobKind::*;
        match self {
            Scan | Subscriptions => Stage::Library,
            AutoMatch | Search | ResolveLink | ApplyMatch | Refresh | RefreshAll | RefreshAiring
            | ClearImages | FillImages => Stage::Metadata,
            ConnectTracker | Mark | SetProgress | Score | RefreshProgress | RefreshWatching => Stage::Trackers,
            Crawl => Stage::Franchise,
            SkipWindows => Stage::Playback,
            Export | Import => Stage::Store,
        }
    }

    /// Kinds that run one at a time: a second call replies Started with the running id.
    pub fn one_at_a_time(self) -> bool {
        use JobKind::*;
        matches!(self, Scan | AutoMatch | RefreshAll | Crawl | RefreshProgress | RefreshWatching | FillImages | Subscriptions)
    }

    /// The variant name, verbatim.
    pub fn as_str(self) -> &'static str {
        use JobKind::*;
        match self {
            Scan => "Scan",
            AutoMatch => "AutoMatch",
            Search => "Search",
            ResolveLink => "ResolveLink",
            ApplyMatch => "ApplyMatch",
            Refresh => "Refresh",
            RefreshAll => "RefreshAll",
            RefreshAiring => "RefreshAiring",
            ClearImages => "ClearImages",
            FillImages => "FillImages",
            ConnectTracker => "ConnectTracker",
            Mark => "Mark",
            SetProgress => "SetProgress",
            Score => "Score",
            RefreshProgress => "RefreshProgress",
            RefreshWatching => "RefreshWatching",
            Crawl => "Crawl",
            SkipWindows => "SkipWindows",
            Export => "Export",
            Import => "Import",
            Subscriptions => "Subscriptions",
        }
    }

    pub fn from_column(s: &str) -> Option<JobKind> {
        use JobKind::*;
        match s {
            "Scan" => Some(Scan),
            "AutoMatch" => Some(AutoMatch),
            "Search" => Some(Search),
            "ResolveLink" => Some(ResolveLink),
            "ApplyMatch" => Some(ApplyMatch),
            "Refresh" => Some(Refresh),
            "RefreshAll" => Some(RefreshAll),
            "RefreshAiring" => Some(RefreshAiring),
            "ClearImages" => Some(ClearImages),
            "FillImages" => Some(FillImages),
            "ConnectTracker" => Some(ConnectTracker),
            "Mark" => Some(Mark),
            "SetProgress" => Some(SetProgress),
            "Score" => Some(Score),
            "RefreshProgress" => Some(RefreshProgress),
            "RefreshWatching" => Some(RefreshWatching),
            "Crawl" => Some(Crawl),
            "SkipWindows" => Some(SkipWindows),
            "Export" => Some(Export),
            "Import" => Some(Import),
            "Subscriptions" => Some(Subscriptions),
            _ => None,
        }
    }
}
