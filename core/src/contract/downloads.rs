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
/// search names has that episode from this very release, `other_on_disk`
/// when it has the episode from another one.
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
    /// Where the video came from as the title names it: "WEB-DL", "WEBRip",
    /// "WEB", "BluRay", "BDRip", "HDTV", "DVD"; empty when it names none.
    #[serde(default)]
    pub rip: String,
    /// True only when the file on disk for this episode is this release:
    /// the same name, or the same group, resolution, rip and version (#237).
    pub on_disk: bool,
    /// The episode is on disk, but from another release than this one.
    #[serde(default)]
    pub other_on_disk: bool,
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
    /// The rolling download this torrent belongs to, if any.
    #[serde(default)]
    pub roll: Option<RollView>,
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
    /// Hold the download to a rolling window of episodes
    /// (`downloads::rolling`): `Some(true)` always, `Some(false)` never,
    /// `None` when the settings say so for one this long.
    #[serde(default)]
    pub rolling: Option<bool>,
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

/// What the desktop's search, refine and subscription preview run on: anirss
/// (the sidecar or a remote server), or the core's own port of it
/// (`releases::native`). Subscribing and downloading stay on anirss with
/// either, for now (#119).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum DownloadsEngine {
    #[default]
    Anirss,
    Native,
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

/// Whose torrent client the downloads use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum TorrentMode {
    /// Not settled yet. The first connect picks `Existing` when a qBittorrent
    /// Web UI answers at the address, else `Managed` when AniBeam can run
    /// one, and saves the choice.
    #[default]
    Auto,
    /// The user's own client, at `torrent_url`.
    Existing,
    /// The qbittorrent-nox AniBeam runs itself: a profile of its own under
    /// AniBeam's data, the Web UI on loopback and open to it without a login.
    Managed,
}

/// One of the user's own qBittorrents, saved by name so the downloads can
/// switch between several: a seedbox and the one on this machine, say. Its
/// password lives in the secret store under a key of its own.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct SavedTorrentClient {
    /// Fixed for the life of the entry, since it names the password's key:
    /// letters, digits, `-` and `_`. Left empty on a new entry that is not
    /// the active one, the core picks it on save.
    pub id: String,
    pub name: String,
    pub url: String,
    pub username: String,
    #[serde(default)]
    pub has_password: bool,
}

/// The downloads settings, secrets left out. The anirss token (remote mode)
/// and the torrent clients' passwords live in the secret store; the `has_*`
/// flags say whether one is held.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct DownloadSettings {
    pub anirss_mode: AnirssMode,
    /// The command a sidecar is started with: a name on PATH or a path.
    pub anirss_command: String,
    /// The command that starts the service behind an on-demand endpoint
    /// (Jackett for 1337x), run through `sh -c` when the shell asks for it.
    /// Empty means anirss's own `start` for that endpoint, else `jackett`.
    #[serde(default)]
    pub jackett_command: String,
    /// The server's base URL in remote mode, without `/api/v1`.
    pub anirss_url: String,
    #[serde(default)]
    pub has_anirss_token: bool,
    pub torrent_client: TorrentClient,
    /// A blob saved before the mode existed belongs to someone who had set
    /// up a client by hand, so it reads as `Existing`; fresh settings start
    /// at `Auto`.
    #[serde(default = "existing_mode")]
    pub torrent_mode: TorrentMode,
    pub torrent_url: String,
    pub torrent_username: String,
    /// Whether the client in use has a password saved here.
    #[serde(default)]
    pub has_torrent_password: bool,
    /// The user's own clients. In `Existing` mode `active_client` names the
    /// one in use, and `torrent_url` and `torrent_username` are its address
    /// and user: a save writes them into it, and a blob from before the list
    /// existed reads as one client made of them.
    #[serde(default)]
    pub saved_clients: Vec<SavedTorrentClient>,
    #[serde(default)]
    pub active_client: String,
    /// Remove a finished show's RSS feed once its run ended long enough
    /// ago that no more episodes are due (`downloads::prune`). Settings
    /// saved before this existed read as on.
    #[serde(default = "default_true")]
    pub prune_finished_feeds: bool,
    /// How long downloads roll (`downloads::rolling`).
    #[serde(default)]
    pub rolling: RollingSettings,
    /// Settings saved before the choice existed, and a value this build
    /// does not know, read as anirss.
    #[serde(default, deserialize_with = "lenient_engine")]
    pub engine: DownloadsEngine,
}

/// A stored engine this build does not know falls back to the default
/// instead of failing the whole blob.
fn lenient_engine<'de, D: serde::Deserializer<'de>>(d: D) -> Result<DownloadsEngine, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    Ok(serde_json::from_value(v).unwrap_or_default())
}

/// Rolling download: a long show is fetched a window of episodes at a
/// time, and each one watched moves the window on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct RollingSettings {
    /// Roll a download of more than `over` episodes without being asked.
    #[serde(default = "default_true")]
    pub big: bool,
    /// Episodes kept downloaded ahead of the one being watched.
    #[serde(default = "default_ahead")]
    pub ahead: u32,
    /// The episode count above which a download rolls by itself.
    #[serde(default = "default_over")]
    pub over: u32,
    /// Watched episodes leave the disk; off keeps them all.
    #[serde(default = "default_true")]
    pub delete_watched: bool,
    /// Watched episodes kept behind the last one watched: with 1, finishing
    /// episode 3 deletes episode 1 and keeps 2.
    #[serde(default = "default_keep")]
    pub keep_watched: u32,
}

impl Default for RollingSettings {
    fn default() -> Self {
        RollingSettings {
            big: true,
            ahead: default_ahead(),
            over: default_over(),
            delete_watched: true,
            keep_watched: default_keep(),
        }
    }
}

fn default_keep() -> u32 {
    1
}

fn default_ahead() -> u32 {
    12
}

fn default_over() -> u32 {
    30
}

/// Where a rolling download stands, carried on each of its torrents.
/// Positions count episodes in the roll's order from 1.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct RollView {
    pub id: u64,
    pub ahead: u32,
    /// The next episode to watch.
    pub next: u64,
    /// The last episode the window reaches.
    pub through: u64,
    pub total: u64,
    /// Episodes watched since the roll began.
    pub watched: u64,
    pub delete_watched: bool,
}

fn existing_mode() -> TorrentMode {
    TorrentMode::Existing
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

/// A place the phone searches. `kind` is `nyaa` (the site's RSS with its
/// `q`, `c` and `f` parameters) or `rss` (any feed whose URL holds
/// `{query}`, erai's among them).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct PhoneSource {
    pub name: String,
    pub kind: String,
    pub url: String,
    pub enabled: bool,
}

// Endpoint repos (#143) ------------------------------------------------------------------

/// A signed index the person added by its URL. Several can come from one site, one per group
/// it publishes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EndpointRepo {
    pub url: String,
    pub name: String,
    /// The group is adult content.
    pub nsfw: bool,
    /// What to compare by eye: a short hash of the pinned key.
    pub fingerprint: String,
    pub serial: u64,
    /// When the index in hand was last fetched and passed its checks, in Unix seconds.
    pub fetched_at: i64,
    /// Why the last refresh kept the older index; None when it went through.
    pub error: Option<String>,
}

/// A repo fetched and checked but not trusted yet: what the person sees before saying yes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EndpointRepoPreview {
    pub url: String,
    pub name: String,
    pub nsfw: bool,
    /// The key as the index states it, `ed25519:<base64>`; `TrustEndpointRepo` sends it back.
    pub key: String,
    pub fingerprint: String,
    pub serial: u64,
    pub endpoints: Vec<OfferedEndpoint>,
    /// The repo is already trusted with this key, so trusting it again only refreshes it.
    pub trusted: bool,
}

/// One value an endpoint needs from the person: a token, a key, a user name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EndpointField {
    pub key: String,
    pub label: String,
    /// Shown as dots and kept only in the secret store.
    pub secret: bool,
    pub help: String,
    pub help_url: Option<String>,
}

/// An endpoint a trusted repo offers, as its index defines it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct OfferedEndpoint {
    pub repo_url: String,
    pub id: String,
    pub name: String,
    pub version: u32,
    pub lang: String,
    pub kind: String,
    /// The host requests go to.
    pub host: String,
    /// The name connections carry instead, for a fronted host.
    pub front: Option<String>,
    pub auth: Vec<EndpointField>,
    pub nsfw: bool,
    /// The install id when it is installed.
    pub installed: Option<String>,
}

/// An endpoint installed from a repo.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct InstalledEndpoint {
    pub install_id: String,
    pub repo_url: String,
    pub id: String,
    pub name: String,
    pub version: u32,
    pub kind: String,
    pub host: String,
    pub front: Option<String>,
    pub auth: Vec<EndpointField>,
    pub nsfw: bool,
    pub enabled: bool,
    /// Added by hand, outside any repo.
    pub custom: bool,
    /// On the desktop: why anirss searches its own endpoint of this name instead.
    pub desktop_note: Option<String>,
    /// `auth` keys with no value stored; the endpoint is left out of searches until they
    /// are filled.
    pub missing: Vec<String>,
    /// The repo still offers it.
    pub offered: bool,
    /// The version the repo offers, when it is higher than this one.
    pub update: Option<u32>,
    /// Taking the update needs the credentials again: its fields changed, or they would now
    /// go to a different host (`update_host`).
    pub update_asks: bool,
    pub update_host: Option<String>,
}

/// A value for one `auth` field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EndpointCredential {
    pub key: String,
    pub value: String,
}

/// The repos, what they offer, and what is installed, in one read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct EndpointCatalog {
    pub repos: Vec<EndpointRepo>,
    pub offered: Vec<OfferedEndpoint>,
    /// In search order.
    pub installed: Vec<InstalledEndpoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum PhoneAction {
    Pause,
    Resume,
    /// Takes the torrent out and deletes what it wrote.
    Remove,
    /// Takes a finished torrent off the list and keeps its files.
    Forget,
}

/// One line of the phone's connection check: a step against one host, whether it went
/// through, what came back, and how long it took.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct NetCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
    pub ms: u64,
}
