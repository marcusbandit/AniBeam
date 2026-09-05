use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Source { pub id: u64, pub path: String, pub available: bool, pub series_count: u64, pub movie_folders: Vec<String> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Titles { pub romaji: Option<String>, pub english: Option<String>, pub native: Option<String>, pub folder: String }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct MatchInfo { pub provider: Provider, pub anilist_id: Option<u64>, pub mal_id: Option<u64>, pub tmdb_id: Option<u64>, pub tmdb_kind: Option<TmdbKind>, pub confirmed: bool }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Strip { pub watched: f64, pub aired_unwatched: f64, pub unknown: f64 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Airing { pub episode: u32, pub at: SystemTime }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SeriesCard {
    pub id: u64, pub kind: SeriesKind, pub path: String,
    pub title: String, pub titles: Titles, pub poster: Option<String>, pub format: Option<String>,
    pub status: Option<AiringStatus>, pub hidden: bool, pub missing: bool, pub match_info: Option<MatchInfo>,
    pub episodes_on_disk: u64, pub extras_on_disk: u64, pub total_episodes: Option<u32>, pub total_is_estimate: bool,
    pub code: Option<String>, pub watched: Option<u32>, pub watched_state: WatchedState, pub strip: Strip,
    pub community_score: Option<f64>, pub my_score: Option<f64>, pub list_status: Option<ListStatus>,
    pub next_airing: Option<Airing>, pub last_viewed_at: Option<SystemTime>, pub latest_activity_at: SystemTime,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Tag { pub name: String, pub rank: u32, pub spoiler: bool, pub adult: bool }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ProgressLine { pub watched: Option<u32>, pub total: Option<u32>, pub estimate: bool, pub on_disk: u64 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Person { pub name: String, pub image: Option<String>, pub role: String }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Recommendation { pub anilist_id: u64, pub title: String, pub poster: Option<String>, pub owned: Option<u64>, pub list_status: Option<ListStatus> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ResumePoint { pub position: f64, pub duration: f64 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Sidecar { pub path: String, pub language: Option<String>, pub title: Option<String> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Episode {
    pub file: u64, pub number: f64, pub season: Option<u32>, pub code: String, pub title: Option<String>,
    pub air_date: Option<SystemTime>, pub path: String, pub sidecars: Vec<Sidecar>,
    pub resume: Option<ResumePoint>, pub watched: bool, pub next_up: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Extra { pub file: u64, pub kind: ExtraKind, pub code: String, pub label: String, pub path: String, pub sidecars: Vec<Sidecar>, pub resume: Option<ResumePoint> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SeriesDetail {
    pub card: SeriesCard, pub banner: Option<String>, pub synopsis: String, pub year: Option<u32>, pub studio: Option<String>,
    pub genres: Vec<String>, pub tags: Vec<Tag>, pub rewatch_count: Option<u32>, pub site_url: Option<String>,
    pub progress: ProgressLine, pub next_up: Option<u64>, pub episodes: Vec<Episode>, pub extras: Vec<Extra>,
    pub unmatched_files: Vec<Episode>, pub characters: Vec<Person>, pub recommendations: Vec<Recommendation>, pub has_graph: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum FeedReason { Aired { episode: u32, at: SystemTime }, Downloaded { at: SystemTime }, Scheduled { episode: u32, at: SystemTime }, None }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct FeedCard { pub series: SeriesCard, pub reason: FeedReason, pub highest_on_disk: Option<f64> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct MetadataRow { pub series: SeriesCard, pub alt_title: Option<String>, pub provider: Option<Provider>, pub have: u64, pub expected: Option<u64>, pub extra_on_disk: u64 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct FilterCounts { pub all: u64, pub series: u64, pub movies: u64, pub missing_files: u64 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SearchResult { pub provider: Provider, pub id: u64, pub title: String, pub alt_title: Option<String>, pub format: Option<String>, pub year: Option<u32>, pub episodes: Option<u32>, pub cover_url: Option<String> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackerAccount { pub connected: bool, pub username: Option<String>, pub user_id: Option<u64>, pub expires_at: Option<SystemTime>, pub last_sync: Option<SystemTime>, pub client_id: String, pub has_client_secret: bool, pub bundled_credentials: bool }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackerState { pub main: Tracker, pub anilist: TrackerAccount, pub mal: TrackerAccount }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackerOutcome { pub tracker: Tracker, pub ok: bool, pub progress: Option<u32>, pub reason: Option<Refusal>, pub message: Option<String> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct WatchingEntry {
    pub anilist_id: u64, pub title: String, pub poster: Option<String>, pub progress: u32, pub total: Option<u32>,
    pub updated_at: SystemTime, pub owned: Option<u64>, pub repeating: bool,
    pub site_url: Option<String>, pub next_airing: Option<Airing>, pub score: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct WatchingList { pub entries: Vec<WatchingEntry>, pub fetched_at: Option<SystemTime> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct GraphNode {
    pub anilist_id: u64, pub x: f64, pub y: f64, pub w: f64, pub h: f64, pub title: String, pub poster: Option<String>,
    pub owned: Option<u64>, pub released: bool, pub format: Option<String>, pub year: Option<u32>, pub relation: Option<String>,
    pub list_status: Option<ListStatus>, pub current: bool, pub root: bool, pub pending: bool, pub site_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct GraphEdge { pub from: u64, pub to: u64, pub relation: String }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct FranchiseLayout { pub root: u64, pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge>, pub complete: bool }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Chapter { pub title: String, pub start: f64 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SkipWindow { pub kind: SkipKind, pub start: f64, pub end: f64, pub source: SkipSource }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Colour { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TextStyle { pub font: String, pub colour: Colour, pub outline_size: f64, pub outline_colour: Colour, pub shadow_offset: f64, pub box_opacity: f64, pub bold: bool, pub position: f64 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SubtitleDefaults { pub subtitle_languages: Vec<String>, pub audio_languages: Vec<String>, pub scale: f64, pub ass_override: AssOverride, pub text_style: TextStyle }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct AutoSkip { pub intro: bool, pub outro: bool }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackRef { pub kind: TrackKind, pub language: Option<String>, pub title: Option<String> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum SubtitleChoice { Off, Track { track: TrackRef } }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TrackChoice { pub audio: Option<TrackRef>, pub subtitle: Option<SubtitleChoice> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct PlaybackSession {
    pub session: u64, pub file: u64, pub path: String, pub series: u64, pub series_title: String, pub episode_title: Option<String>,
    pub code: String, pub is_extra: bool, pub is_last_episode: bool, pub resume_from: Option<f64>, pub prev: Option<u64>, pub next: Option<u64>,
    pub sidecars: Vec<Sidecar>, pub skip_windows: Vec<SkipWindow>, pub artwork: Option<String>,
    pub subtitle_defaults: SubtitleDefaults, pub track_choice: TrackChoice,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Preferences { pub title_language: TitleLanguage, pub library_tab: Tab, pub library_sort: Sort, pub library_direction: Direction, pub feed_sort: FeedSort }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Settings { pub subtitle_defaults: SubtitleDefaults, pub auto_skip: AutoSkip, pub main_tracker: Tracker }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ImportSummary { pub sources_added: u64, pub sources_unavailable: u64, pub series_created: u64, pub matches_applied: u64, pub views_merged: u64, pub completed_merged: u64, pub resume_points_merged: u64, pub accounts_imported: u64, pub fields_ignored: Vec<String> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Feed { pub name: String, pub active: bool, pub torrents: u64, pub query: String, pub save_path: String, pub url: String }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum SubscriptionsResult { Ok { feeds: Vec<Feed> }, Missing, NeedsAuth, Timeout }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Progress { pub done: u64, pub total: Option<u64>, pub label: String }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct JobInfo { pub id: u64, pub kind: JobKind, pub started_at: SystemTime, pub progress: Option<Progress> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct About { pub version: String, pub data_dir: String, pub config_dir: String, pub cache_dir: String, pub db_path: String }

impl Default for SubtitleDefaults {
    fn default() -> Self {
        SubtitleDefaults {
            subtitle_languages: vec!["en".into()],
            audio_languages: vec!["ja".into()],
            scale: 1.0,
            ass_override: AssOverride::ScaleOnly,
            text_style: TextStyle {
                font: "sans-serif".into(),
                colour: Colour { r: 255, g: 255, b: 255, a: 255 },
                outline_size: 1.65,
                outline_colour: Colour { r: 0, g: 0, b: 0, a: 255 },
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
        Preferences { title_language: TitleLanguage::Romaji, library_tab: Tab::All, library_sort: Sort::Alpha, library_direction: Direction::Asc, feed_sort: FeedSort::Recent }
    }
}
impl Default for Settings {
    fn default() -> Self {
        Settings { subtitle_defaults: SubtitleDefaults::default(), auto_skip: AutoSkip::default(), main_tracker: Tracker::Anilist }
    }
}
