use super::*;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Source {
    pub id: u64,
    pub path: String,
    pub available: bool,
    pub series_count: u64,
    pub movie_folders: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Titles {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub folder: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct MatchInfo {
    pub provider: Provider,
    pub anilist_id: Option<u64>,
    pub mal_id: Option<u64>,
    pub tmdb_id: Option<u64>,
    pub tmdb_kind: Option<TmdbKind>,
    pub tvmaze_id: Option<u64>,
    pub confirmed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Strip {
    pub watched: f64,
    pub aired_unwatched: f64,
    pub unknown: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Airing {
    pub episode: u32,
    pub at: SystemTime,
    /// The episode's season for a provider that numbers within one (TVmaze),
    /// None for AniList, whose numbers run through a media. `serde(default)`
    /// because older blobs predate it.
    #[serde(default)]
    pub season: Option<u32>,
    /// A guess from the dates of the episodes already out, where the record
    /// names no date ahead (`library::cards::predict_next`). The shell says
    /// "predicted" beside it.
    #[serde(default)]
    pub predicted: bool,
    /// The last episode landing at the same moment, when a drop releases
    /// several at once (7 to 9); None for one episode.
    #[serde(default)]
    pub through: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SeriesCard {
    pub id: u64,
    pub kind: SeriesKind,
    pub path: String,
    pub title: String,
    pub titles: Titles,
    pub poster: Option<String>,
    pub format: Option<String>,
    pub status: Option<AiringStatus>,
    pub genres: Vec<String>,
    pub hidden: bool,
    pub missing: bool,
    pub match_info: Option<MatchInfo>,
    pub episodes_on_disk: u64,
    pub extras_on_disk: u64,
    pub total_episodes: Option<u32>,
    pub total_is_estimate: bool,
    pub code: Option<String>,
    pub watched: Option<u32>,
    pub watched_state: WatchedState,
    pub strip: Strip,
    /// The episode cubes: one state code per episode from 1 to the total,
    /// at most 400 (`library::cubes`). Empty for a film or a show with no
    /// count at all. The codes and their colours are one table each side.
    #[serde(default)]
    pub cubes: String,
    /// The whole episode numbers with a file in the library, in order. The cubes put
    /// watched ahead of on disk, so an episode watched elsewhere reads `w` with no file
    /// here; this is what is actually here (#156).
    #[serde(default)]
    pub episodes_here: Vec<u32>,
    pub community_score: Option<f64>,
    pub my_score: Option<f64>,
    pub list_status: Option<ListStatus>,
    pub next_airing: Option<Airing>,
    pub last_viewed_at: Option<SystemTime>,
    pub latest_activity_at: SystemTime,
    /// When the folder was first seen by a scan: when the person decided
    /// to start on it, the order they expect a list in.
    #[serde(default = "epoch")]
    pub added_at: SystemTime,
    /// When the newest file arrived (its mtime), the "recently
    /// downloaded" order; None with nothing on disk.
    #[serde(default)]
    pub downloaded_at: Option<SystemTime>,
    /// The episodes a torrent is still writing and how far each is, laid on
    /// by the downloads overlay (`downloads::overlay`), never by the library
    /// scan. An episode listed here is not playable yet.
    #[serde(default)]
    pub downloading: Vec<EpisodeDownload>,
    /// The franchise this series shares with other library series, or None
    /// when it relates to no other series in the library
    /// (`franchise::groups`).
    #[serde(default)]
    pub franchise: Option<FranchiseTag>,
    /// The release year: AniList's seasonYear, else the start year; TVmaze's
    /// premiere year.
    #[serde(default)]
    pub year: Option<u32>,
    /// The anime season the series started in, as the shell shows it:
    /// "Fall 2025", "Winter 2024". AniList's own season word when it carries
    /// one, else derived from the start date's month (Jan to Mar Winter, Apr
    /// to Jun Spring, Jul to Sep Summer, Oct to Dec Fall); None without a date.
    #[serde(default)]
    pub season: Option<String>,
    /// The IMDb title behind the match, as IMDb writes it: "tt" and at least
    /// seven digits, "tt0213338". A TVmaze match names its own; an AniList
    /// or MAL match is linked through Wikidata. None for a TMDB match, an
    /// unmatched series, or a match nothing links to IMDb. Set even when the
    /// title has no rating yet.
    #[serde(default)]
    pub imdb_id: Option<String>,
    /// IMDb's average rating, already 0 to 10, from IMDb's own dump.
    #[serde(default)]
    pub imdb_score: Option<f64>,
    /// How many votes that average stands on.
    #[serde(default)]
    pub imdb_votes: Option<u64>,
}

/// One franchise as the library page groups it: the connected component of
/// library series over the relations the graph walks (never CHARACTER or
/// OTHER). Only a component of two or more series is tagged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct FranchiseTag {
    /// Stable for as long as the members stay: the smallest AniList id
    /// among the component's library series, as a decimal string.
    pub id: String,
    /// The group's name: the members' shared title stem, else the
    /// earliest-released member's title with its season suffix stripped.
    pub title: String,
    /// How many library series are in the component, missing ones left
    /// out, hidden ones counted whichever tab is open.
    pub size: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Tag {
    pub name: String,
    pub rank: u32,
    pub spoiler: bool,
    pub adult: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ProgressLine {
    pub watched: Option<u32>,
    pub total: Option<u32>,
    pub estimate: bool,
    pub on_disk: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Person {
    pub name: String,
    pub image: Option<String>,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Recommendation {
    pub anilist_id: u64,
    pub title: String,
    pub poster: Option<String>,
    pub owned: Option<u64>,
    pub list_status: Option<ListStatus>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ResumePoint {
    pub position: f64,
    pub duration: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Sidecar {
    pub path: String,
    pub language: Option<String>,
    pub title: Option<String>,
}

/// One episode of a series that a torrent is still writing, `progress` 0 to 1.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EpisodeDownload {
    pub number: u32,
    pub progress: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Episode {
    pub file: u64,
    pub number: f64,
    pub season: Option<u32>,
    pub code: String,
    pub title: Option<String>,
    pub air_date: Option<SystemTime>,
    pub path: String,
    pub sidecars: Vec<Sidecar>,
    pub resume: Option<ResumePoint>,
    /// A resume point short of the episode's end line: the outro's start
    /// when one is known, else the 85 percent the tracker mark uses. Past
    /// that line the episode was seen to its end, whatever is left.
    pub stopped_short: bool,
    pub watched: bool,
    pub next_up: bool,
    /// How far a torrent has written this episode, while one is still on it;
    /// None once it is whole. Laid on by the downloads overlay: a file that is
    /// being downloaded is on disk but is not playable.
    #[serde(default)]
    pub downloading: Option<f64>,
    /// The episode's cube code (`library::cards::cubes`): `w` watched and
    /// tracked online, `l` watched here and not tracked yet, `d` on disk.
    #[serde(default)]
    pub cube: String,
    /// Read this way because the user said so (`FixFiles`), not by its name.
    #[serde(default)]
    pub fixed: bool,
}

/// An episode that has aired and has no file, and the tracker does not
/// cover: a row on the series page that cannot play. `cube` is `m` (missing, a later episode is on disk) or
/// `g` (not downloaded), the codes the card's cube row gives it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct AbsentEpisode {
    pub number: u32,
    pub season: Option<u32>,
    pub code: String,
    pub title: Option<String>,
    pub air_date: Option<SystemTime>,
    pub cube: String,
}

/// An episode still to come: when the record schedules it, or when it is
/// guessed to land (`predicted`, `library::cards::upcoming`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct UpcomingEpisode {
    pub number: u32,
    pub season: Option<u32>,
    pub code: String,
    pub title: Option<String>,
    pub at: SystemTime,
    pub predicted: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Extra {
    pub file: u64,
    pub kind: ExtraKind,
    pub code: String,
    pub label: String,
    pub path: String,
    pub sidecars: Vec<Sidecar>,
    pub resume: Option<ResumePoint>,
    /// The extra's own index, `1` in `OP1`; None without one.
    #[serde(default)]
    pub index: Option<u32>,
    /// As `Episode::fixed`.
    #[serde(default)]
    pub fixed: bool,
}

/// What a file is, as the user says it (#170): an episode with its number and
/// season, or an extra of a kind with its index. `FixFiles` stores it against
/// the file's path, and every scan after reads the file this way.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum FileReading {
    Episode { number: f64, season: Option<u32> },
    Extra { kind: ExtraKind, index: Option<u32> },
}

/// One file's fix. `reading` None hands the file back to the scan's own
/// reading of its name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct FileFix {
    pub file: u64,
    pub reading: Option<FileReading>,
}

/// A misreading the series page can see on its own, said in one sentence,
/// and the fixes that would undo it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SuggestedFix {
    pub text: String,
    pub fixes: Vec<FileFix>,
}

/// A filled in GitHub issue for a quick fix (`ReportFix`): the page to open,
/// and whose triage it carries when a coding agent ran one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct FixReport {
    pub url: String,
    pub title: String,
    pub body: String,
    /// `Claude Code` or `Codex` when its triage is in the body.
    pub triaged_by: Option<String>,
    /// Why an agent that was found wrote no triage: it failed or ran out of time.
    pub triage_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SeriesDetail {
    pub card: SeriesCard,
    pub banner: Option<String>,
    pub synopsis: String,
    pub year: Option<u32>,
    pub studio: Option<String>,
    /// How many minutes an episode runs, a film's runtime for a film: the
    /// AniList record's duration, or the TVmaze show's average runtime.
    /// None when the record gives no figure.
    pub episode_minutes: Option<u32>,
    pub genres: Vec<String>,
    pub tags: Vec<Tag>,
    pub rewatch_count: Option<u32>,
    pub site_url: Option<String>,
    pub progress: ProgressLine,
    pub next_up: Option<u64>,
    pub episodes: Vec<Episode>,
    /// The aired episodes with no file, in number order.
    #[serde(default)]
    pub absent: Vec<AbsentEpisode>,
    /// The episodes still to come, in number order.
    #[serde(default)]
    pub upcoming: Vec<UpcomingEpisode>,
    pub extras: Vec<Extra>,
    pub unmatched_files: Vec<Episode>,
    pub characters: Vec<Person>,
    pub recommendations: Vec<Recommendation>,
    pub has_graph: bool,
    /// A misreading of this series' files the core can see, with its fix.
    #[serde(default)]
    pub suggested_fix: Option<SuggestedFix>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum FeedReason {
    Aired {
        episode: u32,
        at: SystemTime,
    },
    Downloaded {
        at: SystemTime,
    },
    Scheduled {
        episode: u32,
        at: SystemTime,
        /// As `Airing::season`.
        #[serde(default)]
        season: Option<u32>,
    },
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct FeedCard {
    pub series: SeriesCard,
    pub reason: FeedReason,
    pub highest_on_disk: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct MetadataRow {
    pub series: SeriesCard,
    pub alt_title: Option<String>,
    pub provider: Option<Provider>,
    pub have: u64,
    pub expected: Option<u64>,
    pub extra_on_disk: u64,
}

/// How many series each library tab holds, no query applied: what the
/// page needs to decide which tabs to offer and whether the library is
/// empty at all.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TabCounts {
    pub all: u64,
    pub series: u64,
    pub movies: u64,
    pub watched: u64,
    pub hidden: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct FilterCounts {
    pub all: u64,
    pub series: u64,
    pub movies: u64,
    pub missing_files: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SearchResult {
    pub provider: Provider,
    pub id: u64,
    pub title: String,
    pub alt_title: Option<String>,
    pub format: Option<String>,
    pub year: Option<u32>,
    pub episodes: Option<u32>,
    pub cover_url: Option<String>,
    /// Every name the provider knows the show by, so the shell can show
    /// the one that was searched for and search releases by another.
    #[serde(default)]
    pub romaji: Option<String>,
    #[serde(default)]
    pub english: Option<String>,
    /// The title in its own script, kana and kanji for a Japanese show
    #[serde(default)]
    pub native: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    /// Still getting episodes, or not started yet: worth following for new ones.
    #[serde(default)]
    pub airing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackerAccount {
    pub connected: bool,
    pub username: Option<String>,
    pub user_id: Option<u64>,
    pub expires_at: Option<SystemTime>,
    pub last_sync: Option<SystemTime>,
    pub client_id: String,
    pub has_client_secret: bool,
    pub bundled_credentials: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackerState {
    pub main: Tracker,
    pub anilist: TrackerAccount,
    pub mal: TrackerAccount,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackerOutcome {
    pub tracker: Tracker,
    pub ok: bool,
    pub progress: Option<u32>,
    pub reason: Option<Refusal>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct WatchingEntry {
    pub anilist_id: u64,
    pub title: String,
    pub poster: Option<String>,
    pub progress: u32,
    pub total: Option<u32>,
    pub updated_at: SystemTime,
    pub owned: Option<u64>,
    pub repeating: bool,
    pub site_url: Option<String>,
    pub next_airing: Option<Airing>,
    pub score: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct WatchingList {
    pub entries: Vec<WatchingEntry>,
    pub fetched_at: Option<SystemTime>,
}

/// One show on Airing now. `owned` is the library series that carries it,
/// `list_status` its entry on a connected tracker, AniList first.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct AiringShow {
    pub anilist_id: u64,
    pub mal_id: Option<u64>,
    pub title: String,
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub poster: Option<String>,
    pub format: Option<String>,
    pub episodes: Option<u32>,
    /// AniList's 0 to 100, as it came.
    pub average_score: Option<u32>,
    pub site_url: Option<String>,
    pub next_airing: Option<Airing>,
    /// The newest episode out in the last two days, if one is.
    pub latest: Option<AiredEpisode>,
    pub owned: Option<u64>,
    pub list_status: Option<ListStatus>,
}

/// One episode that went out: the strip along the top of Airing now.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct AiredEpisode {
    pub anilist_id: u64,
    pub title: String,
    pub episode: u32,
    pub at: SystemTime,
}

/// Airing now as the page draws it: the shows, most popular first, with
/// every one marked not interested left out, and the episodes out in the
/// last two days, newest first.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct AiringNow {
    pub season: SeasonPick,
    /// "Airing now", or the season by name: "Winter 2027".
    pub label: String,
    pub shows: Vec<AiringShow>,
    pub aired: Vec<AiredEpisode>,
    /// How many of the season's shows are marked not interested.
    pub not_interested: u32,
    pub fetched_at: Option<SystemTime>,
}

/// A show marked not interested, as Settings lists it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct NotInterestedShow {
    pub anilist_id: u64,
    pub title: String,
    pub poster: Option<String>,
    pub marked_at: SystemTime,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct GraphNode {
    pub anilist_id: u64,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub title: String,
    pub poster: Option<String>,
    pub owned: Option<u64>,
    pub released: bool,
    pub format: Option<String>,
    pub year: Option<u32>,
    pub relation: Option<String>,
    pub list_status: Option<ListStatus>,
    pub current: bool,
    pub root: bool,
    pub pending: bool,
    pub site_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct GraphEdge {
    pub from: u64,
    pub to: u64,
    pub relation: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct FranchiseLayout {
    pub root: u64,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub complete: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Chapter {
    pub title: String,
    pub start: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SkipWindow {
    pub kind: SkipKind,
    pub start: f64,
    pub end: f64,
    pub source: SkipSource,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TextStyle {
    pub font: String,
    pub colour: Colour,
    pub outline_size: f64,
    pub outline_colour: Colour,
    pub shadow_offset: f64,
    pub box_opacity: f64,
    pub bold: bool,
    pub position: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SubtitleDefaults {
    pub subtitle_languages: Vec<String>,
    pub audio_languages: Vec<String>,
    pub scale: f64,
    pub ass_override: AssOverride,
    pub text_style: TextStyle,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct AutoSkip {
    pub intro: bool,
    pub outro: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackRef {
    pub kind: TrackKind,
    pub language: Option<String>,
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum SubtitleChoice {
    Off,
    Track { track: TrackRef },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackChoice {
    pub audio: Option<TrackRef>,
    pub subtitle: Option<SubtitleChoice>,
}

/// A Chromecast or a DLNA TV found on this network.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct CastDevice {
    pub id: String,
    pub name: String,
    pub kind: CastKind,
    /// The model the device names, "Chromecast" or "Google TV Streamer", when it says.
    pub model: Option<String>,
    pub address: String,
}

/// The subtitles to send with a cast.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum CastSubtitle {
    None,
    /// A text track in the file, by its Matroska track number.
    Track {
        number: u64,
    },
    /// A subtitle file beside the episode.
    File {
        path: String,
    },
}

/// What a cast session is doing. Positions are seconds into the episode.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct CastState {
    pub device: String,
    pub kind: CastKind,
    pub file: u64,
    pub state: CastPlayState,
    pub position: f64,
    pub duration: f64,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct PlaybackSession {
    pub session: u64,
    pub file: u64,
    pub path: String,
    pub series: u64,
    pub series_title: String,
    pub episode_title: Option<String>,
    pub code: String,
    pub is_extra: bool,
    pub is_last_episode: bool,
    pub resume_from: Option<f64>,
    pub prev: Option<u64>,
    pub next: Option<u64>,
    pub sidecars: Vec<Sidecar>,
    pub skip_windows: Vec<SkipWindow>,
    pub artwork: Option<String>,
    pub subtitle_defaults: SubtitleDefaults,
    pub track_choice: TrackChoice,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Preferences {
    // Each choice reads leniently: a value another build wrote that this one does not know
    // (a newer sort, say) falls back to its default instead of failing the whole blob, and
    // with it every job that reads the preferences.
    #[serde(deserialize_with = "lenient::title_language")]
    pub title_language: TitleLanguage,
    #[serde(deserialize_with = "lenient::tab")]
    pub library_tab: Tab,
    #[serde(deserialize_with = "lenient::sort")]
    pub library_sort: Sort,
    #[serde(deserialize_with = "lenient::direction")]
    pub library_direction: Direction,
    #[serde(deserialize_with = "lenient::feed_sort")]
    pub feed_sort: FeedSort,
    /// Whether series that are done with (`cards::watched_out`) also sit in
    /// the main tabs, or only in the Watched tab. Default off: a
    /// non-rewatcher's finished series leaves the grid and the airing rail
    /// behind. `serde(default)` because older blobs predate it.
    #[serde(default)]
    pub library_show_watched: bool,
    /// How the library page arranges its cards. `serde(default)` because
    /// older blobs predate it, which is the whole migration.
    #[serde(default, deserialize_with = "lenient::arrange")]
    pub library_arrange: Arrange,
    /// Whether the library pulls a franchise's series (two or more in the
    /// library) out into a section of their own ahead of the cut, or leaves
    /// every series under its own season, type or genre. Default on.
    #[serde(default = "default_true")]
    pub library_group_franchises: bool,
    /// Whether the core serves the library, subscriptions and downloads as
    /// read-only JSON on this machine's Tailscale addresses, for the phone
    /// app (`host`). Default off, and per machine: an export never carries it.
    #[serde(default)]
    pub share_over_tailscale: bool,
    /// Whether the core moves a Watching entry nobody has touched in
    /// `on_hold_days` days to On hold on every connected tracker (AniList's
    /// Paused, MAL's On Hold), after each tracker refresh (`trackers::on_hold`).
    /// Default on. `serde(default)` because older blobs predate it.
    #[serde(default = "default_true")]
    pub on_hold_auto: bool,
    /// How many days a Watching entry sits untouched before `on_hold_auto`
    /// moves it, `ON_HOLD_DAYS` by default.
    #[serde(default = "default_on_hold_days")]
    pub on_hold_days: u32,
}

/// The default for `Preferences::on_hold_days`: two months untouched.
pub const ON_HOLD_DAYS: u32 = 60;

fn default_true() -> bool {
    true
}

fn default_on_hold_days() -> u32 {
    ON_HOLD_DAYS
}

/// Preferences fields that fall back to `Preferences::default()`'s value
/// when the stored one is not a variant this build knows.
mod lenient {
    use super::*;
    use serde::Deserializer;
    use serde::de::DeserializeOwned;

    fn or<'de, D: Deserializer<'de>, T: DeserializeOwned>(
        d: D,
        fallback: T,
    ) -> Result<T, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        Ok(serde_json::from_value(v).unwrap_or(fallback))
    }

    pub fn title_language<'de, D: Deserializer<'de>>(d: D) -> Result<TitleLanguage, D::Error> {
        or(d, Preferences::default().title_language)
    }
    pub fn tab<'de, D: Deserializer<'de>>(d: D) -> Result<Tab, D::Error> {
        or(d, Preferences::default().library_tab)
    }
    pub fn sort<'de, D: Deserializer<'de>>(d: D) -> Result<Sort, D::Error> {
        or(d, Preferences::default().library_sort)
    }
    pub fn direction<'de, D: Deserializer<'de>>(d: D) -> Result<Direction, D::Error> {
        or(d, Preferences::default().library_direction)
    }
    pub fn feed_sort<'de, D: Deserializer<'de>>(d: D) -> Result<FeedSort, D::Error> {
        or(d, Preferences::default().feed_sort)
    }
    pub fn arrange<'de, D: Deserializer<'de>>(d: D) -> Result<Arrange, D::Error> {
        or(d, Preferences::default().library_arrange)
    }
}

fn epoch() -> SystemTime {
    SystemTime::UNIX_EPOCH
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Settings {
    pub subtitle_defaults: SubtitleDefaults,
    pub auto_skip: AutoSkip,
    pub main_tracker: Tracker,
    /// The match providers in the order the auto-match tries them and the
    /// match panel offers them. Always every one of `MATCH_PROVIDERS`,
    /// each exactly once.
    pub provider_order: Vec<Provider>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ImportSummary {
    pub sources_added: u64,
    pub sources_unavailable: u64,
    pub series_created: u64,
    pub matches_applied: u64,
    pub views_merged: u64,
    pub completed_merged: u64,
    pub resume_points_merged: u64,
    pub accounts_imported: u64,
    pub fields_ignored: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Feed {
    pub name: String,
    pub active: bool,
    pub torrents: u64,
    pub query: String,
    pub save_path: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum SubscriptionsResult {
    Ok { feeds: Vec<Feed> },
    Missing,
    NeedsAuth,
    Timeout,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Progress {
    pub done: u64,
    pub total: Option<u64>,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct JobInfo {
    pub id: u64,
    pub kind: JobKind,
    pub started_at: SystemTime,
    pub progress: Option<Progress>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct About {
    pub version: String,
    pub data_dir: String,
    pub config_dir: String,
    pub cache_dir: String,
    pub db_path: String,
}

impl Default for SubtitleDefaults {
    fn default() -> Self {
        SubtitleDefaults {
            subtitle_languages: vec!["en".into()],
            audio_languages: vec!["ja".into()],
            scale: 1.0,
            ass_override: AssOverride::ScaleOnly,
            text_style: TextStyle {
                font: "sans-serif".into(),
                colour: Colour {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
                outline_size: 1.65,
                outline_colour: Colour {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                shadow_offset: 0.0,
                box_opacity: 0.0,
                bold: false,
                position: 100.0,
            },
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            title_language: TitleLanguage::Romaji,
            library_tab: Tab::All,
            library_sort: Sort::Alpha,
            library_direction: Direction::Asc,
            feed_sort: FeedSort::Recent,
            library_show_watched: false,
            library_arrange: Arrange::Related,
            library_group_franchises: true,
            share_over_tailscale: false,
            on_hold_auto: true,
            on_hold_days: ON_HOLD_DAYS,
        }
    }
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            subtitle_defaults: SubtitleDefaults::default(),
            auto_skip: AutoSkip::default(),
            main_tracker: Tracker::Anilist,
            provider_order: MATCH_PROVIDERS.to_vec(),
        }
    }
}

/// What the tailnet host is doing, for the Settings page: whether the
/// preference is on, the URLs it answers on, and one plain sentence for the
/// state ("Serving on 100.65.105.9", "Tailscale is not connected").
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct HostStatus {
    pub enabled: bool,
    /// `http://<address>:<port>` for every address it is bound to, then the
    /// MagicDNS name's when `tailscale` could say it. Empty while it is not
    /// serving.
    pub urls: Vec<String>,
    pub message: String,
}
