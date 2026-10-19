//! The downloads domain's records: what a search endpoint returns, what a
//! refine step does to it, what anirss registers in qBittorrent, and what
//! qBittorrent is doing right now. The release shapes are anirss's own
//! `Item` and the objects beside it, field for field, so a release the core
//! received from anirss goes back to it unchanged (anirss keeps no search
//! state; the caller sends `{endpoint, query, items}` with every step).

use super::{Airing, AiringStatus, SeriesKind};
use serde::{Deserialize, Serialize};

/// One release from a feed: anirss's `Item`. The three counts are `None`
/// when the feed did not carry them, which is unknown rather than zero;
/// Erai-raws and most plain RSS feeds carry none.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Release {
    pub title: String,
    pub link: String,
    #[serde(default)]
    pub seeders: Option<i64>,
    #[serde(default)]
    pub leechers: Option<i64>,
    #[serde(default)]
    pub downloads: Option<i64>,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub page: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub resolution: String,
    #[serde(default)]
    pub infomkv: String,
    #[serde(default)]
    pub linktype: String,
}

/// A token, or tokens with one membership, that splits the result set:
/// `[SubsPlease]`, `1080p`, `HEVC`. `members` are indexes into the items
/// the group was computed over. Applying one is a `Tokens` refine op with
/// `tokens` echoed back verbatim.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ReleaseGroup {
    pub label: String,
    pub tokens: Vec<String>,
    pub member_count: u64,
    #[serde(default)]
    pub has_poster: bool,
    #[serde(default)]
    pub members: Vec<u64>,
}

/// What a best-fit step would do: `available` is false when it would
/// change nothing, `item` is the release it would pin the profile of.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct BestFit {
    pub available: bool,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub item: Option<Release>,
    #[serde(default)]
    pub profile: Vec<String>,
}

/// One endpoint consulted by a search: `status` is `ok`, `empty` or
/// `unreachable`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EndpointTry {
    pub endpoint: String,
    pub status: String,
    #[serde(default)]
    pub count: u64,
    #[serde(default)]
    pub error: Option<String>,
}

/// A search endpoint as anirss lists it. `url` is display-safe: anirss
/// masks any token or API key in it, so it is not a usable feed URL.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SearchEndpoint {
    pub name: String,
    pub kind: String,
    pub url: String,
    #[serde(default)]
    pub client_side_refine: bool,
    /// For an endpoint anirss starts on demand (a pod behind Jackett), the
    /// seconds anirss gives it to come up; None for one that is always
    /// there. Read from anirss's own config by the core.
    #[serde(default)]
    pub on_demand: Option<u64>,
    /// For an on-demand endpoint, whether its ready URL answered the last
    /// probe; None for an endpoint that is always there, or one with no
    /// ready URL to ask. The probe is a plain GET and never starts it.
    #[serde(default)]
    pub running: Option<bool>,
}

/// A search and its first analysis in one: the items, where they came
/// from (`endpoint` differs from the one asked for when `fell_back`), the
/// groups they split by and the best fit on offer. `query` and `items` are
/// the state after auto-resolution, which may have appended a resolution
/// token (`auto_resolution` names it); they are what every later step sends.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ReleaseSearch {
    pub query: String,
    pub endpoint: String,
    pub items: Vec<Release>,
    pub groups: Vec<ReleaseGroup>,
    pub best_fit: BestFit,
    pub tried: Vec<EndpointTry>,
    pub fell_back: bool,
    pub client_side: bool,
    pub auto_resolution: Option<String>,
    /// Index for index with `items`.
    #[serde(default)]
    pub facts: Vec<ReleaseFacts>,
    /// The library series the query names, when one fits.
    #[serde(default)]
    pub show: Option<ShowLink>,
    /// The query the endpoint itself was asked, before anirss narrowed the
    /// result on its own side (an auto-resolution on a generic RSS
    /// endpoint): what a subscription's feed should search.
    #[serde(default)]
    pub fetched_query: String,
}

/// One refine step. `Tokens` narrows to a group, `Term` adds a positive
/// term, `Exclude` appends `-term`, `BestFit` pins the top release's
/// profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum RefineOp {
    Tokens { tokens: Vec<String> },
    Term { term: String },
    Exclude { term: String },
    BestFit,
}

/// What a refine step did. With `applied` false the state is unchanged and
/// `reason` (`noop`, `zero_results`, `no_match`, `empty_term`) says why;
/// `mode` is `refetch` (nyaa searched again) or `client` (filtered locally).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Refinement {
    pub applied: bool,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub mode: String,
    pub query: String,
    pub endpoint: String,
    pub items: Vec<Release>,
    pub groups: Vec<ReleaseGroup>,
    pub best_fit: BestFit,
    #[serde(default)]
    pub previous_count: u64,
    #[serde(default)]
    pub delta: i64,
    /// Index for index with `items`; the core fills these, anirss does not.
    #[serde(default)]
    pub facts: Vec<ReleaseFacts>,
    #[serde(default)]
    pub show: Option<ShowLink>,
}

/// What subscribing would register, before it does.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SubscriptionSpec {
    pub feed_url: String,
    #[serde(default)]
    pub must_contain: String,
    #[serde(default)]
    pub must_not_contain: String,
    #[serde(default)]
    pub endpoint_name: String,
}

/// A subscription made: the rule name actually used (it may gain an
/// ` @endpoint` suffix), the feed and where episodes land.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Subscribed {
    pub name: String,
    pub feed_url: String,
    pub save_path: String,
    #[serde(default)]
    pub must_contain: String,
    #[serde(default)]
    pub must_not_contain: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Torrents handed to qBittorrent. `hashes` are the info-hashes known up
/// front (magnets); a `.torrent` URL qBittorrent fetches itself has none.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct AddedTorrents {
    pub count: u64,
    pub save_path: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub hashes: Vec<String>,
}

/// One subscription as qBittorrent holds it: an RSS feed and an
/// auto-download rule. `file_count` is files on disk under `save_path`,
/// zero when qBittorrent runs on another host.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SubscriptionRule {
    pub name: String,
    pub feed_url: String,
    pub save_path: String,
    #[serde(default = "default_true")]
    pub rule_enabled: bool,
    #[serde(default)]
    pub torrent_count: u64,
    #[serde(default)]
    pub file_count: u64,
    /// The query the feed searches, read off its `q` parameter; empty
    /// when the feed has none.
    #[serde(default)]
    pub query: String,
    /// The library series the rule's folder fills, when the library holds
    /// it: the core fills this in, anirss knows nothing of it.
    #[serde(default)]
    pub show: Option<ShowLink>,
}

/// A library series as the downloads page shows it beside a rule or a
/// group of torrents. Linked by folder: the exact path first, then the
/// folder's name, then the name with case and punctuation dropped, since
/// a client's save path can name the same folder under another mount.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ShowLink {
    pub series_id: u64,
    pub kind: SeriesKind,
    pub title: String,
    pub path: String,
    pub poster: Option<String>,
    pub status: Option<AiringStatus>,
    /// Distinct episode numbers on disk, so a v2 next to its v1 counts
    /// once.
    pub episodes_on_disk: u64,
    pub total_episodes: Option<u32>,
    pub total_is_estimate: bool,
    pub next_airing: Option<Airing>,
    /// Every episode from 1 to the total is on disk and the show has
    /// finished airing (or was cancelled): nothing more will come, so the
    /// subscription has nothing left to do. A film is complete once its
    /// file is there.
    pub complete: bool,
    /// The episode numbers from 1 to the total that are not on disk, when
    /// the total is known. Empty for a complete show.
    pub missing: Vec<u32>,
    /// The series card's episode cubes, carried along for the page.
    #[serde(default)]
    pub cubes: String,
}

fn default_true() -> bool {
    true
}

/// What removing a subscription would take with it, shown before it does.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct RemovePlan {
    pub name: String,
    #[serde(default)]
    pub feed_url: String,
    #[serde(default)]
    pub save_path: String,
    #[serde(default)]
    pub torrent_count_in_qb: u64,
    #[serde(default)]
    pub size_bytes: u64,
    /// Names of the torrents that would go, when torrents or files were
    /// asked for.
    #[serde(default)]
    pub torrents: Vec<String>,
}

/// What the core reads off a release beyond anirss's fields, index for
/// index with the items it came with. `episode` is None for a batch and
/// for a title naming none; `on_disk` is true when the library series the
/// search names already has that episode.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, uniffi::Record)]
pub struct ReleaseFacts {
    pub episode: Option<f64>,
    pub season: Option<u32>,
    pub batch: bool,
    /// The episodes a batch names, `(01-28)` as 1 and 28; None when it
    /// names none (a season pack, "Complete Season").
    #[serde(default)]
    pub first_episode: Option<u32>,
    #[serde(default)]
    pub last_episode: Option<u32>,
    pub group: String,
    pub resolution: String,
    pub size_bytes: Option<u64>,
    pub on_disk: bool,
}

/// A subscription's rule as qBittorrent holds it now, and what it takes:
/// `matching` is the titles of the articles in its feeds that the rule
/// accepts, as qBittorrent judges them (an empty list before qBittorrent
/// has read the feed, or when nothing in it fits the rule yet).
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, uniffi::Record)]
pub struct RuleView {
    pub name: String,
    pub enabled: bool,
    pub must_contain: String,
    pub must_not_contain: String,
    pub save_path: String,
    pub feeds: Vec<String>,
    /// qBittorrent's own words for when it last took something, or empty.
    pub last_match: String,
    pub matching: Vec<String>,
}

/// One torrent as qBittorrent reports it, trimmed to what the page draws.
/// Units are qBittorrent's: `progress` 0 to 1, sizes in bytes, speeds in
/// bytes a second, `eta` in seconds (8640000 when unknown), times in Unix
/// seconds. Paths are as qBittorrent sees them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Torrent {
    pub hash: String,
    pub name: String,
    pub state: String,
    pub progress: f64,
    pub size: u64,
    pub dlspeed: u64,
    pub upspeed: u64,
    pub eta: i64,
    pub num_seeds: u64,
    pub num_leechs: u64,
    pub ratio: f64,
    pub save_path: String,
    pub content_path: String,
    pub added_on: i64,
    pub completion_on: i64,
    pub tags: String,
    pub category: String,
    /// The library series this torrent's folder fills, linked as a
    /// `ShowLink` is; its record travels beside the list.
    #[serde(default)]
    pub series_id: Option<u64>,
    /// The episode the torrent's name says it carries, read as a release
    /// title is (`downloads::facts`); None for a batch or a name that
    /// names none.
    #[serde(default)]
    pub episode: Option<f64>,
    /// The name says it is a batch: several episodes in one torrent.
    #[serde(default)]
    pub batch: bool,
    /// The files inside, each with its own progress, read for a torrent of
    /// several files that is not finished; empty otherwise.
    #[serde(default)]
    pub files: Vec<TorrentFile>,
    /// qBittorrent's "download in order" is on for this torrent.
    #[serde(default)]
    pub seq_dl: bool,
    /// qBittorrent's "first and last pieces first" is on for this torrent.
    #[serde(default)]
    pub f_l_piece_prio: bool,
    /// The host of the tracker the client talks to, as `nyaa.tracker.wf`;
    /// empty for a torrent with none working yet.
    #[serde(default)]
    pub tracker: String,
    /// The release group in the torrent's name, read as a release title is
    /// (`downloads::facts`).
    #[serde(default)]
    pub group: String,
    /// The resolution in the torrent's name.
    #[serde(default)]
    pub resolution: String,
    /// The anirss endpoint (erai, nyaa) this torrent was picked on, when
    /// AniBeam added it; qBittorrent keeps no such thing, so older
    /// torrents carry none.
    #[serde(default)]
    pub provider: String,
}

/// One file inside a torrent: its name, size and progress, and the episode
/// its name carries, so a batch shows how far along each episode is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct TorrentFile {
    pub name: String,
    pub size: u64,
    pub progress: f64,
    /// qBittorrent's priority, 0 when the file is skipped.
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub episode: Option<f64>,
}

/// One release in a subscription's feed, read by the core straight from
/// the feed so the page can show what the feed holds without leaving the
/// app. Nyaa's own fields and torznab's attributes both land here; a field
/// the feed did not carry is empty or `None`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct FeedEntry {
    pub title: String,
    /// The torrent or magnet link.
    pub link: String,
    /// The release's own page, when the feed names one.
    pub page: String,
    /// When it was posted, Unix seconds.
    pub published: Option<i64>,
    /// The size as the feed wrote it ("1.4 GiB"), or formatted from bytes.
    pub size: String,
    pub size_bytes: Option<u64>,
    pub seeders: Option<i64>,
    pub leechers: Option<i64>,
    pub downloads: Option<i64>,
    /// Lower case hex, empty when the feed carried none.
    pub info_hash: String,
}

/// qBittorrent's transfer totals.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, uniffi::Record)]
pub struct TransferRate {
    pub dl_speed: u64,
    pub up_speed: u64,
    pub connection_status: String,
    pub dht_nodes: u64,
}

/// What to do to a set of torrents.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum TorrentAction {
    Stop,
    Start,
    /// Removes the torrents from qBittorrent; with `files` their data too.
    Delete {
        files: bool,
    },
    /// Download in order (qBittorrent's sequential download) on or off.
    /// Idempotent: the client's toggle is only pressed for a torrent
    /// whose setting differs.
    Sequential {
        on: bool,
    },
    /// First and last pieces first on or off, set the same way.
    FirstLastPieces {
        on: bool,
    },
}

/// What a download asks for beyond the releases themselves: the order the
/// client fetches in, and which episodes the new torrents replace.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadOptions {
    /// Turn on download in order for the new torrents once the client has them.
    #[serde(default)]
    pub sequential: bool,
    /// The endpoint the releases came from, kept so Activity can say so.
    #[serde(default)]
    pub provider: String,
    /// The library series the episodes belong to; None for a show not
    /// scanned in yet, which has no files to replace.
    #[serde(default)]
    pub series: Option<u64>,
    /// Which items replace which episodes: `item` is an index into the
    /// call's items.
    #[serde(default)]
    pub replace: Vec<ReplaceItem>,
    /// Unfinished torrents (by info-hash) the new ones stand in for.
    /// Each is cancelled, with what it wrote, if every episode it is still
    /// fetching is being replaced; otherwise only those files are skipped.
    #[serde(default)]
    pub cancel: Vec<String>,
}

/// One item of a download and the episodes it replaces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct ReplaceItem {
    pub item: u64,
    pub episodes: Vec<u32>,
}

/// How the core reaches anirss: as a sidecar it starts itself and talks to
/// over a port and token it chose, or as a server somebody else runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum AnirssMode {
    Sidecar,
    Remote,
}

/// The torrent clients the downloads page can drive. One for now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum TorrentClient {
    Qbittorrent,
}

impl TorrentClient {
    pub fn as_str(self) -> &'static str {
        match self {
            TorrentClient::Qbittorrent => "qBittorrent",
        }
    }
}

/// The downloads settings, secrets left out. The anirss token (remote mode)
/// and the torrent client's password live in the secret store; the two
/// `has_*` flags say whether one is held.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadSettings {
    pub anirss_mode: AnirssMode,
    /// The command a sidecar is started with: a name on PATH or a path.
    pub anirss_command: String,
    /// The server's base URL in remote mode, without `/api/v1`.
    pub anirss_url: String,
    #[serde(default)]
    pub has_anirss_token: bool,
    pub torrent_client: TorrentClient,
    pub torrent_url: String,
    pub torrent_username: String,
    #[serde(default)]
    pub has_torrent_password: bool,
}

/// Where the sidecar is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum AnirssState {
    /// Not started, or stopped.
    Off,
    Starting,
    Ready,
    /// The command was not found.
    Missing,
    /// It started and then failed, or never answered.
    Failed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct AnirssStatus {
    pub state: AnirssState,
    pub version: String,
    pub url: String,
    /// What went wrong, when `state` is `Missing` or `Failed`.
    pub message: String,
    /// qBittorrent as anirss sees it, from its own config.
    pub qbt_url: String,
    pub qbt_reachable: bool,
    pub qbt_authenticated: bool,
    pub qbt_login_failed: bool,
    /// Where anirss's own `config.toml` is expected, so a shell can offer
    /// to open it: the endpoints and the save folders live there.
    #[serde(default)]
    pub config_path: String,
}

impl Default for AnirssStatus {
    fn default() -> Self {
        AnirssStatus {
            state: AnirssState::Off,
            version: String::new(),
            url: String::new(),
            message: String::new(),
            qbt_url: String::new(),
            qbt_reachable: false,
            qbt_authenticated: false,
            qbt_login_failed: false,
            config_path: String::new(),
        }
    }
}

/// The torrent client as the core reaches it directly.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, uniffi::Record)]
pub struct TorrentClientStatus {
    pub connected: bool,
    pub version: String,
    pub message: String,
    /// `anibeam` when the password is the one saved here, `anirss` when
    /// it was borrowed from anirss's own saved password, else `none`.
    pub password_source: String,
}

/// Everything the downloads page and the settings tab need in one read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadsStatus {
    pub settings: DownloadSettings,
    pub anirss: AnirssStatus,
    pub torrent: TorrentClientStatus,
    pub endpoints: Vec<SearchEndpoint>,
}
