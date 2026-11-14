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

/// One drop on Home's week (#236): an episode, or several released at once
/// (`through`), when it airs or aired, and whether every episode of it is
/// here: a file of that season on disk, or watched already on the tracker.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct WeekDrop {
    pub episode: u32,
    pub through: Option<u32>,
    pub season: Option<u32>,
    pub at: SystemTime,
    pub here: bool,
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
    /// A film, by our own kind or the provider's format (`library::cards::is_film`).
    /// The shells read this rather than repeating the check.
    #[serde(default)]
    pub film: bool,
    pub status: Option<AiringStatus>,
    pub genres: Vec<String>,
    pub hidden: bool,
    pub missing: bool,
    pub match_info: Option<MatchInfo>,
    pub episodes_on_disk: u64,
    pub extras_on_disk: u64,
    pub total_episodes: Option<u32>,
    pub total_is_estimate: bool,
    /// The episode count the person set by hand (#239), whether or not it
    /// is in force: `total_episodes` follows it until the schedule or a
    /// file goes past it.
    #[serde(default)]
    pub episodes_set: Option<u32>,
    /// Whether `episodes_set` is in force: false once an episode past it
    /// aired or landed, or while the record names a count of its own.
    #[serde(default)]
    pub episodes_set_applied: bool,
    /// The season a count set by hand belongs to: a TVmaze card's own season,
    /// since TVmaze numbers within one (#283). None for AniList.
    #[serde(default)]
    pub count_season: Option<u32>,
    /// The bytes every file of the series takes on disk, extras included (#266).
    #[serde(default)]
    pub bytes_on_disk: u64,
    /// The bytes of the files whose episode is watched: within the tracker's
    /// progress or played to the end here. An extra is never watched.
    #[serde(default)]
    pub watched_bytes: u64,
    /// The watched episodes with a file here, two releases of one counted once.
    #[serde(default)]
    pub watched_on_disk: u64,
    /// Confirmed done with (#266): the provider says Finished, the episode
    /// count is the provider's own rather than a guess or one set by hand,
    /// and every episode of it is watched (or the tracker says Completed).
    /// A film: Finished and watched. What Delete watched shows takes.
    #[serde(default)]
    pub finished_watched: bool,
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
    /// How many different episodes the files on disk hold: each season counts its own,
    /// and two releases of one episode count once. `episodes_on_disk` counts files (#323).
    #[serde(default)]
    pub distinct_episodes: u64,
    pub community_score: Option<f64>,
    pub my_score: Option<f64>,
    pub list_status: Option<ListStatus>,
    pub next_airing: Option<Airing>,
    /// Every episode the schedule puts from three days back to eight days
    /// ahead, aired or not, soonest first, a drop of several at once as one
    /// entry: the week on Home keeps what already aired today on its day
    /// and says whether it landed (#236).
    #[serde(default)]
    pub week_airings: Vec<WeekDrop>,
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
    /// The character's own AniList id, for a link to its page; None for a row stored
    /// without one.
    pub anilist_id: Option<u64>,
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
    /// Every file of this episode (the same season and number) when there is
    /// more than one, this one among them; empty for a lone file. Each is
    /// named by its release, so a shell can offer a pick between them.
    #[serde(default)]
    pub versions: Vec<EpisodeVersion>,
}

/// One file of an episode that is on disk more than once: two releases of it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EpisodeVersion {
    pub file: u64,
    /// The release, `group, 1080p WEB-DL H264`; the file name itself where
    /// two versions of the episode would otherwise read alike.
    pub label: String,
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

/// What the user told a series' schedule that its provider does not say
/// (#418): episodes `first` to `last` land on `at`, or some time in `year`,
/// or on no known date when both are None. The predictor never guesses an
/// episode a note without a date covers, nor any after it until a known
/// date; a provider's own date wins over `at`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ScheduleNote {
    pub first: u32,
    pub last: u32,
    pub at: Option<SystemTime>,
    pub year: Option<i32>,
    /// The month, 1 to 12, of `year`, when the day is not known.
    #[serde(default)]
    pub month: Option<u32>,
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
    /// As `Episode::versions`: every file of this extra when there is more than one,
    /// two releases of it (`SP01` and `SP01v2`, #468); empty for a lone file.
    #[serde(default)]
    pub versions: Vec<EpisodeVersion>,
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
    /// The notes on the series' list entry, off the same tracker as the
    /// card's status: AniList's notes, MAL's comments (#211).
    #[serde(default)]
    pub notes: Option<String>,
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
    /// The schedule notes of the card's season, in number order (#418).
    #[serde(default)]
    pub schedule_notes: Vec<ScheduleNote>,
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

/// One series on the list kept on this device, as moving it up to a
/// tracker would change it (#580): what is kept here, and what the tracker
/// has now, none for a show not on its list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct MoveRow {
    pub series: u64,
    pub title: String,
    pub status: Option<ListStatus>,
    pub score: Option<f64>,
    pub tracker_status: Option<ListStatus>,
    pub tracker_score: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackerOutcome {
    pub tracker: Tracker,
    pub ok: bool,
    pub progress: Option<u32>,
    pub reason: Option<Refusal>,
    pub message: Option<String>,
}

/// One episode of a followed show that has just aired (#212): a show on
/// the Watching list, by its AniList id. `series` is the library series
/// with files that carries it, None when it is not on disk.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EpisodeOut {
    pub anilist_id: u64,
    pub episode: u32,
    pub at: SystemTime,
    /// The display title, by the title-language preference.
    pub title: String,
    /// AniList's own titles, for Get episodes: it searches by the latin ones.
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub series: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct WatchingEntry {
    pub anilist_id: u64,
    /// The display title, by the title-language preference.
    pub title: String,
    /// AniList's own titles, for a hand-off that needs each one (Get
    /// episodes searches by the latin titles, whatever the page shows).
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
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

/// A year of watching (#216). Minutes count each finished episode at its
/// record's length, or a plain default where the record names none.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Stats {
    pub year: i32,
    /// Every year with any episode finished, newest first.
    pub years: Vec<i32>,
    pub minutes: u64,
    pub episodes: u32,
    pub series_completed: u32,
    /// January to December.
    pub months: Vec<StatMonth>,
    pub genres: Vec<StatCount>,
    pub studios: Vec<StatCount>,
    /// The seasons the shows came out in ("Fall 2025").
    pub seasons: Vec<StatCount>,
    /// The shows watched most, by minutes.
    pub top_series: Vec<StatSeries>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct StatMonth {
    pub month: u32,
    pub episodes: u32,
    pub minutes: u64,
    pub series_completed: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct StatCount {
    pub name: String,
    pub episodes: u32,
    pub series: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct StatSeries {
    pub series: u64,
    pub title: String,
    pub poster: Option<String>,
    pub episodes: u32,
    pub minutes: u64,
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
    /// The wide banner, a local file, when AniList has one and the show has
    /// an episode in or near Out now: the only place that draws it.
    pub banner: Option<String>,
    pub format: Option<String>,
    pub episodes: Option<u32>,
    /// AniList's 0 to 100, as it came.
    pub average_score: Option<u32>,
    /// AniList's trending score, as it came.
    pub trending: Option<u32>,
    /// How many AniList users favourited it.
    pub favourites: Option<u32>,
    /// Minutes an episode runs.
    pub duration: Option<u32>,
    /// The first episode's date as yyyymmdd; a month or day AniList does not
    /// know yet reads 00.
    pub start_date: Option<u32>,
    pub site_url: Option<String>,
    pub next_airing: Option<Airing>,
    /// The newest episode out in the last 24 hours, if one is.
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
/// last 24 hours, newest first.
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

/// Home's Recommended for you row (#779): the shows the recommender ranks,
/// best first, held to one scope. `counts` says how many each scope holds, so
/// the switch above the row can name them without a read per scope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Recommended {
    pub scope: RecommendedScope,
    pub state: RecommendedState,
    pub shows: Vec<RecommendedShow>,
    pub counts: RecommendedCounts,
    pub fitted_at: Option<SystemTime>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct RecommendedCounts {
    pub all: u32,
    pub in_library: u32,
    pub on_list: u32,
}

/// One recommended show. `forecast` is the share of the user's rated shows it
/// is predicted to beat, 0 to 100. `genres` and `tags` never carry a spoiler
/// tag; `hidden_loved` only counts the spoiler tags the user loves that sit
/// behind the pick, and nothing names them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct RecommendedShow {
    pub anilist_id: u64,
    pub mal_id: Option<u64>,
    pub title: String,
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub poster: Option<String>,
    /// AniList's accent colour for the cover, "#e4a15d".
    pub color: Option<String>,
    pub format: Option<String>,
    pub year: Option<i32>,
    pub episodes: Option<u32>,
    /// Minutes an episode runs.
    pub duration: Option<u32>,
    /// AniList's 0 to 100, as it came.
    pub average_score: Option<u32>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    /// The first sentence of the description, source notes cut.
    pub line: String,
    pub forecast: u32,
    pub hidden_loved: u32,
    /// The user's own shows this one is most like, closest first.
    pub because: Vec<BecauseShow>,
    pub owned: Option<u64>,
    pub list_status: Option<ListStatus>,
    pub site_url: Option<String>,
}

/// One of the user's shows, as the evidence for a recommendation: its
/// poster, and the score they gave it out of 10.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct BecauseShow {
    pub anilist_id: u64,
    pub title: String,
    pub poster: Option<String>,
    pub score: Option<f64>,
    pub owned: Option<u64>,
}

/// The page for one AniList show the library may not have (#779): what the
/// core holds for it, the user's list entry, and the recommender's word on it
/// when it ranked it. `tags` never carry a spoiler tag.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ShowPage {
    pub anilist_id: u64,
    pub mal_id: Option<u64>,
    pub title: String,
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub poster: Option<String>,
    pub banner: Option<String>,
    pub color: Option<String>,
    pub synopsis: Option<String>,
    pub format: Option<String>,
    /// AniList's airing state: "FINISHED", "RELEASING", "NOT_YET_RELEASED".
    pub status: Option<String>,
    pub year: Option<i32>,
    pub episodes: Option<u32>,
    pub duration: Option<u32>,
    /// AniList's 0 to 100, as it came.
    pub average_score: Option<u32>,
    pub genres: Vec<String>,
    pub tags: Vec<Tag>,
    pub studios: Vec<String>,
    /// A link to the trailer, when AniList names one on a site it can link.
    pub trailer: Option<String>,
    pub relations: Vec<ShowRelation>,
    pub recommendations: Vec<Recommendation>,
    pub site_url: Option<String>,
    /// The library series that carries it. The page is that series' own
    /// page once this is set.
    pub owned: Option<u64>,
    pub list_status: Option<ListStatus>,
    /// The user's score out of 10.
    pub score: Option<f64>,
    pub progress: u32,
    pub forecast: Option<u32>,
    pub hidden_loved: u32,
    pub because: Vec<BecauseShow>,
    /// Whether anything is known about the show at all. A page for an id the
    /// core never saw is blank until its read lands.
    pub known: bool,
}

/// A show related to the one on a show page: a prequel, a sequel, a side
/// story. `relation` is AniList's own word, "SEQUEL".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ShowRelation {
    pub anilist_id: u64,
    pub title: String,
    pub poster: Option<String>,
    pub relation: String,
    pub format: Option<String>,
    pub owned: Option<u64>,
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
    /// The track is flagged forced, the signs-only kind that often shares its language and
    /// title with the full one. A choice stored before this field reads as not forced.
    #[serde(default)]
    pub forced: bool,
    /// Which one it was, from 0, of the file's tracks that share all of the above: two
    /// "French" tracks in one file are told apart by their order alone.
    #[serde(default)]
    pub nth: u32,
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

/// A watch together room as this member sees it (#846). Everyone plays their own copy;
/// the room carries the episode's identity, the play state and who is in it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct RoomInfo {
    /// The 6 character code, uppercase: what a person types to join.
    pub code: String,
    /// `<relay>/<code>`, the join page a shared link opens.
    pub link: String,
    /// This member's id in the room, 0 before the relay has said it.
    pub you: u32,
    /// Whether this member is the host, who moves the room to another episode.
    pub host: bool,
    /// Who is connected now, in the order they joined.
    pub members: Vec<RoomMember>,
    pub episode: EpisodeRef,
    /// Seconds this member's file is from the host's: local position is the room's
    /// position plus the offset.
    pub offset: f64,
    /// The offset the openings suggest, when both openings are known and under 30
    /// seconds apart.
    pub suggested_offset: Option<f64>,
    /// The playback session the room drives, None until `RoomAttach` names one.
    pub session: Option<u64>,
    /// False while the connection to the relay is down and being tried again.
    pub connected: bool,
    /// Where this member's Get media is (#889).
    #[serde(default)]
    pub fetch: RoomFetch,
    /// Whether Get media can start: this library lacks the episode, and the host's app
    /// can be asked for it.
    #[serde(default)]
    pub can_fetch: bool,
}

/// One person in a room.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct RoomMember {
    pub id: u32,
    pub name: String,
    pub host: bool,
    pub ready: RoomReady,
    /// While `ready` is `Downloading`: how far, from 0 to 100.
    #[serde(default)]
    pub progress: Option<f64>,
    /// While `ready` is `Downloading`: seconds left, when the torrent client can tell.
    #[serde(default)]
    pub eta: Option<u64>,
}

/// Which episode a room watches: its identity and nothing that leads to the video.
/// Positions in the room are on the host's file, whose length and opening are here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EpisodeRef {
    pub anilist_id: Option<u64>,
    pub mal_id: Option<u64>,
    pub tvmaze_id: Option<u64>,
    /// The history key: the number for an episode, the file name for an extra.
    pub episode_key: String,
    pub number: Option<f64>,
    pub series_title: String,
    pub episode_title: Option<String>,
    /// The host's file length, in seconds.
    pub duration: Option<f64>,
    /// Where the opening starts in the host's file, in seconds.
    pub opening_start: Option<f64>,
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
    /// older blobs predate it, which is the whole migration: they read as
    /// Related, what they showed before the key existed, while a fresh install
    /// starts on Flat.
    #[serde(
        default = "arrange_before_the_key",
        deserialize_with = "lenient::arrange"
    )]
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
    /// Whether the host also serves the episodes themselves while it shares:
    /// the files, their conversions and the playback calls. Default on, so a
    /// phone that streams today keeps streaming. Per machine like
    /// `share_over_tailscale`: an export never carries it.
    #[serde(default = "default_true")]
    pub stream_over_tailscale: bool,
    /// The tailnet devices the host refuses, by Tailscale's stable node id.
    /// Every other device on the tailnet is trusted. Per machine like
    /// `share_over_tailscale`: an export never carries it.
    #[serde(default)]
    pub tailnet_switched_off: Vec<String>,
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
    /// Whether the player tells a running Discord client what is playing
    /// (`presence`). Default off, and per machine like `share_over_tailscale`:
    /// an export never carries it. `serde(default)` because older blobs predate it.
    #[serde(default)]
    pub discord_presence: bool,
    /// Whether the core keeps an iCal file of the airing times of the shows
    /// on the user's lists (`calendar`). Default off, and per machine like
    /// `discord_presence`. `serde(default)` because older blobs predate it.
    #[serde(default)]
    pub calendar_feed: bool,
    /// Whether the desktop shows a notification when an episode of a show
    /// on the Watching list airs (#212). Default on, and per machine.
    /// `serde(default)` because older blobs predate it.
    #[serde(default = "default_true")]
    pub episode_notifications: bool,
    /// Whether Home's Coming up board leaves out the days with nothing on
    /// them (#688). Default off, and per machine like `episode_notifications`.
    /// `serde(default)` because older blobs predate it.
    #[serde(default)]
    pub home_hide_empty_days: bool,
    /// Which shows Home's Recommended for you row holds (#779).
    #[serde(default, deserialize_with = "lenient::recommended")]
    pub home_recommended: RecommendedScope,
    /// The name others in a watch together room see (#846). Empty until the person
    /// gives one: the shell asks before the first room. Per machine.
    #[serde(default)]
    pub room_name: String,
    /// The relay rooms are made on, `ROOM_RELAY` by default. Per machine.
    #[serde(default = "default_room_relay")]
    pub room_relay: String,
}

/// The default for `Preferences::on_hold_days`: two months untouched.
pub const ON_HOLD_DAYS: u32 = 60;

fn default_true() -> bool {
    true
}

/// What a preferences blob written before `library_arrange` existed shows
fn arrange_before_the_key() -> Arrange {
    Arrange::Related
}

fn default_on_hold_days() -> u32 {
    ON_HOLD_DAYS
}

/// The relay the app makes rooms on unless the person picks another.
pub const ROOM_RELAY: &str = "https://rooms.anibeam.app";

fn default_room_relay() -> String {
    ROOM_RELAY.to_string()
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
    pub fn recommended<'de, D: Deserializer<'de>>(d: D) -> Result<RecommendedScope, D::Error> {
        or(d, Preferences::default().home_recommended)
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
    /// Files whose row an imported quick fix rewrote.
    pub fixes_applied: u64,
    pub fields_ignored: Vec<String>,
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
            library_arrange: Arrange::Flat,
            library_group_franchises: true,
            share_over_tailscale: false,
            stream_over_tailscale: true,
            tailnet_switched_off: Vec::new(),
            on_hold_auto: true,
            on_hold_days: ON_HOLD_DAYS,
            discord_presence: false,
            calendar_feed: false,
            episode_notifications: true,
            home_hide_empty_days: false,
            home_recommended: RecommendedScope::All,
            room_name: String::new(),
            room_relay: ROOM_RELAY.to_string(),
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

/// A device on the tailnet, for the per-device switch in Settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct TailnetDevice {
    /// Tailscale's stable node id, which `SetTailnetDeviceTrust` takes.
    pub id: String,
    pub name: String,
    /// The operating system Tailscale reports ("android", "linux"), or empty.
    pub os: String,
    pub online: bool,
    /// Whether the host answers it: true unless the person switched it off.
    pub trusted: bool,
    /// The person whose profile the host answers it with, as Tailscale names them
    /// (#633). Empty when the tailnet does not say.
    #[serde(default)]
    pub person: String,
    /// Tailscale ties it to no person while the host tells people apart, so the host
    /// refuses it whatever the switch says.
    #[serde(default)]
    pub nobody: bool,
}

/// A person whose devices the tailnet host serves, for Settings (#633): the Tailscale
/// user, each answered with their own profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct TailnetPerson {
    /// Tailscale's stable user id.
    pub id: String,
    /// The name Tailscale shows for them, or their login when it shows none.
    pub name: String,
    /// "someone@example.com", or a GitHub login. May be empty.
    pub login: String,
    /// The person this computer belongs to: their devices see its own history.
    pub owner: bool,
    /// How many of their devices the host serves: on the tailnet and not switched
    /// off, this computer left out.
    pub devices: u32,
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
