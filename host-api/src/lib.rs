//! What an AniBeam host serves over a tailnet, and what the phone app reads.
//!
//! The desktop app, with "Share over Tailscale" on, listens on its Tailscale
//! addresses only, so the tailnet itself is the access control. Nothing a
//! client sends changes the library itself: the routes are `GET`s, apart from
//! the playback calls below, which keep watch history. Two documents and one
//! kind of file:
//!
//! - [`HELLO`]: who answers ([`Hello`]). Cheap; a client uses it to tell "host
//!   is up" from "host is down".
//! - [`SNAPSHOT`]: everything a client keeps ([`Snapshot`]): the library, the
//!   subscriptions, the torrents. A client stores the last one it got and shows
//!   it while the host is away; the airing times in it stay true without the
//!   host, and the AniList ids let a client refresh them from AniList itself.
//! - [`poster_path`]: one series' poster, as the host cached it (jpeg, png or
//!   webp; the `Content-Type` says which). [`Series::poster`] changes whenever
//!   the image does, so it doubles as the client's cache key.
//!
//! A host that lists [`FEATURE_STREAM`] in [`Hello::features`] also serves its
//! episodes, so a client plays them where it is:
//!
//! - [`file_path`]: the file as it is, honouring a `Range`, for a player that
//!   seeks by byte. [`sidecar_path`] is one of its external subtitle files.
//! - [`probe_path`]: what is in the file ([`Probe`]), for the client to pick
//!   between the file and a conversion.
//! - [`hls_path`], with [`FEATURE_CONVERT`]: the file converted as it plays to
//!   8-bit H.264 and stereo AAC at a height the client picks, as HLS. The host
//!   converts only the part being watched, so a seek costs a few seconds.
//! - [`CALL`]: `POST` one playback call of the core's contract as JSON, and
//!   read its reply. Only the calls a player makes pass ([`PLAYER_CALLS`]), so
//!   an episode watched on the client keeps its place, its history and its
//!   tracker progress on the host, as if it had played there.
//!
//! Times are Unix seconds. Every optional field may be absent or `null`, and a
//! client must ignore fields it does not know: [`API_VERSION`] only moves for a
//! change an old client would misread.

use serde::{Deserialize, Serialize};

/// Bumped only for a change an older client would misread.
pub const API_VERSION: u32 = 1;
/// The port a host listens on unless told otherwise.
pub const DEFAULT_PORT: u16 = 8790;
/// `GET`: [`Hello`].
pub const HELLO: &str = "/api/v1/hello";
/// `GET`: [`Snapshot`].
pub const SNAPSHOT: &str = "/api/v1/snapshot";
/// What [`Hello::app`] says, so a client knows it reached AniBeam and not
/// something else on the port.
pub const APP: &str = "anibeam";

/// `POST`: one playback call ([`PLAYER_CALLS`]) as the core's `Call` JSON.
/// The answer is the core's `Reply` JSON, or `{"error": ...}` with a status
/// that is not 200.
pub const CALL: &str = "/api/v1/call";
/// In [`Hello::features`] when the host serves its files and playback calls.
pub const FEATURE_STREAM: &str = "stream";
/// In [`Hello::features`] when the host can also convert ([`hls_path`]).
pub const FEATURE_CONVERT: &str = "convert";
/// The `Call` variants [`CALL`] takes: the ones a player makes.
pub const PLAYER_CALLS: &[&str] = &[
    "OpenPlayback",
    "ReportChapters",
    "Tick",
    "ClosePlayback",
    "MarkEpisode",
    "SetTrackChoice",
];
/// The heights a conversion comes in. A client asking for another gets the
/// nearest one below it.
pub const CONVERT_HEIGHTS: &[u32] = &[1080, 720, 480];

/// `GET` this for the poster of the series with this [`Series::id`].
pub fn poster_path(series_id: u64) -> String {
    format!("/api/v1/posters/{series_id}")
}

/// `GET` the episode file with this [`Episode::file`], ranges honoured.
pub fn file_path(file: u64) -> String {
    format!("/api/v1/files/{file}")
}

/// `GET` the external subtitle file at this index of the file's sidecars, in
/// the order the playback session lists them.
pub fn sidecar_path(file: u64, index: usize) -> String {
    format!("/api/v1/files/{file}/sidecars/{index}")
}

/// `GET` the skip windows found so far for a session opened through [`CALL`],
/// as the core's `SkipWindow` JSON list. The host finds them after the
/// player reports its chapters, a few seconds into the episode.
pub fn skip_path(session: u64) -> String {
    format!("/api/v1/sessions/{session}/skip")
}

/// `GET` the text subtitle track at this index of [`Probe::subtitles`] as an ASS file,
/// for a client playing a conversion, which carries none. A bitmap track has none.
pub fn subtitle_path(file: u64, index: usize) -> String {
    format!("/api/v1/files/{file}/subtitles/{index}")
}

/// `GET`: [`Probe`] of the episode file.
pub fn probe_path(file: u64) -> String {
    format!("/api/v1/files/{file}/probe")
}

/// `GET` the HLS playlist of the file converted to `height` lines, with the
/// audio track `audio` (an index into [`Probe::audio`], or the file's default
/// when `None`). Subtitles stay out: a client that wants them loads the
/// file's own tracks from [`file_path`] or a sidecar. With `burn` set to a
/// [`Probe::subtitles`] index, that track is drawn into the picture instead,
/// styling and all, for a receiver that cannot draw it.
pub fn hls_path(file: u64, height: u32, audio: Option<usize>, burn: Option<usize>) -> String {
    let mut path = format!("/api/v1/files/{file}/hls/{height}/index.m3u8");
    let mut sep = '?';
    if let Some(a) = audio {
        path.push_str(&format!("{sep}audio={a}"));
        sep = '&';
    }
    if let Some(b) = burn {
        path.push_str(&format!("{sep}burn={b}"));
    }
    path
}

/// `GET /api/v1/hello`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hello {
    /// Always [`APP`].
    pub app: String,
    /// The host's [`API_VERSION`].
    pub api: u32,
    /// The AniBeam version, as its About page shows it.
    pub version: String,
    /// The machine's name, for a client to show ("banditbox").
    pub host: String,
    /// The host's clock, so a client can tell how far apart they are.
    pub now: i64,
    /// What the host serves past the snapshot: [`FEATURE_STREAM`],
    /// [`FEATURE_CONVERT`]. Empty from a host older than streaming.
    #[serde(default)]
    pub features: Vec<String>,
}

/// `GET` [`probe_path`]: what one episode file holds.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Probe {
    /// Seconds.
    pub duration: f64,
    /// Bytes.
    pub size: u64,
    /// Bits per second over the whole file.
    pub bitrate: u64,
    /// ffmpeg's codec name: `h264`, `hevc`, `av1`.
    #[serde(default)]
    pub video_codec: Option<String>,
    /// `main`, `high`, `high 10`.
    #[serde(default)]
    pub video_profile: Option<String>,
    /// 8, 10.
    #[serde(default)]
    pub bit_depth: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub audio: Vec<Track>,
    #[serde(default)]
    pub subtitles: Vec<Track>,
}

/// One audio or subtitle track of a [`Probe`], in the file's order.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Track {
    /// ffmpeg's codec name: `aac`, `flac`, `ass`, `subrip`.
    pub codec: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub default: bool,
}

/// `GET /api/v1/snapshot`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub api: u32,
    pub host: String,
    pub version: String,
    /// When the host put this together.
    pub generated_at: i64,
    /// The library as its Home page shows it, hidden series left out.
    pub series: Vec<Series>,
    /// The anirss subscriptions. Empty when there are none, and also when the
    /// Downloads half could not be read: [`Snapshot::problems`] says which.
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
    /// What the torrent client holds. `None` when it could not be read, which
    /// is not the same as holding nothing.
    #[serde(default)]
    pub torrents: Option<Vec<Torrent>>,
    /// One plain sentence per part the host could not read this time
    /// ("qBittorrent is not running"). A client shows them as they are.
    #[serde(default)]
    pub problems: Vec<String>,
}

/// One series or film in the library.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Series {
    /// The host's id for it. Stable for the life of the host's library, and
    /// the key for [`poster_path`].
    pub id: u64,
    pub kind: Kind,
    /// In the language the host's user picked.
    pub title: String,
    #[serde(default)]
    pub titles: Titles,
    /// AniList's words: `Releasing`, `Finished`, `NotYetReleased`,
    /// `Cancelled`, `Hiatus`.
    #[serde(default)]
    pub status: Option<String>,
    /// `TV`, `MOVIE`, `OVA` and so on.
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub year: Option<u32>,
    /// "Fall 2025".
    #[serde(default)]
    pub season: Option<String>,
    #[serde(default)]
    pub anilist_id: Option<u64>,
    #[serde(default)]
    pub mal_id: Option<u64>,
    #[serde(default)]
    pub tvmaze_id: Option<u64>,
    /// How many episodes the show has or will have, when known.
    #[serde(default)]
    pub total_episodes: Option<u32>,
    /// Episodes on the host's disk.
    pub episodes_on_disk: u64,
    /// The episode count the user's tracker (AniList or MAL) says they have
    /// watched.
    #[serde(default)]
    pub progress: Option<u32>,
    /// The next episode to air, when one is scheduled.
    #[serde(default)]
    pub next_airing: Option<Airing>,
    /// Every scheduled episode the host knows of, soonest first, the next one
    /// included. A client keeps counting down through these while the host
    /// is away.
    #[serde(default)]
    pub upcoming: Vec<Airing>,
    /// Set when the host has a poster: a tag that changes whenever the image
    /// does. Fetch it from [`poster_path`].
    #[serde(default)]
    pub poster: Option<String>,
    /// The episodes on disk, in order.
    #[serde(default)]
    pub episodes: Vec<Episode>,
    pub added_at: i64,
    /// The newest of: added, a new file, watched.
    pub latest_activity_at: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Show,
    Movie,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Titles {
    #[serde(default)]
    pub romaji: Option<String>,
    #[serde(default)]
    pub english: Option<String>,
    #[serde(default)]
    pub native: Option<String>,
}

/// One episode's air time.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Airing {
    pub episode: u32,
    /// Set for a second or later season numbered on its own (TVmaze shows).
    #[serde(default)]
    pub season: Option<u32>,
    pub at: i64,
}

/// One episode file on the host.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    /// The host's episode key: `"7"`, or `"S02E01"` past the first season.
    pub key: String,
    /// The host's id for the file, for [`file_path`] and the playback calls.
    /// Absent from a host older than streaming.
    #[serde(default)]
    pub file: Option<u64>,
    pub number: f64,
    #[serde(default)]
    pub season: Option<u32>,
    /// What the app shows for it: `"E07"`, `"S02E01"`.
    pub code: String,
    #[serde(default)]
    pub title: Option<String>,
    pub watched: bool,
    #[serde(default)]
    pub aired_at: Option<i64>,
}

/// One anirss subscription: a feed qBittorrent downloads from by itself.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Subscription {
    /// The rule's name in qBittorrent.
    pub name: String,
    /// The RSS feed the rule reads. A client may read it directly to see new
    /// releases while the host is away.
    pub feed_url: String,
    /// What was searched for to make it.
    #[serde(default)]
    pub query: String,
    pub enabled: bool,
    pub torrent_count: u64,
    pub file_count: u64,
    /// The library series its folder fills, when there is one.
    #[serde(default)]
    pub series_id: Option<u64>,
    /// That series' title, or the rule's name when there is no series.
    pub title: String,
    #[serde(default)]
    pub next_airing: Option<Airing>,
    /// Every aired episode is on disk and the show has finished.
    #[serde(default)]
    pub complete: bool,
    /// Aired episode numbers that are not on disk.
    #[serde(default)]
    pub missing: Vec<u32>,
}

/// One torrent in the client.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Torrent {
    pub hash: String,
    pub name: String,
    /// qBittorrent's state word: `downloading`, `uploading`, `stalledDL`,
    /// `pausedUP` and the rest.
    pub state: String,
    /// 0 to 1.
    pub progress: f64,
    /// Bytes.
    pub size: u64,
    /// Bytes per second.
    #[serde(default)]
    pub download_speed: u64,
    /// Seconds left, when the client has an estimate.
    #[serde(default)]
    pub eta: Option<i64>,
    pub added_on: i64,
    #[serde(default)]
    pub completed_on: Option<i64>,
    /// The library series it fills, when there is one.
    #[serde(default)]
    pub series_id: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixture_reads_as_a_snapshot() {
        let text = include_str!("../fixtures/snapshot.json");
        let snapshot: Snapshot = serde_json::from_str(text).unwrap();
        assert_eq!(snapshot.api, API_VERSION);
        assert_eq!(snapshot.series.len(), 3);
        let frieren = &snapshot.series[0];
        assert_eq!(frieren.kind, Kind::Show);
        assert_eq!(frieren.upcoming[0], frieren.next_airing.clone().unwrap());
        assert_eq!(snapshot.subscriptions[0].series_id, Some(frieren.id));
        assert_eq!(snapshot.torrents.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn unknown_fields_and_missing_optionals_are_fine() {
        let text = r#"{"api": 1, "host": "h", "version": "2.1.0", "generated_at": 1,
            "series": [{"id": 1, "kind": "movie", "title": "T", "episodes_on_disk": 1,
                        "added_at": 1, "latest_activity_at": 1, "from_the_future": true}],
            "something_new": [1, 2]}"#;
        let snapshot: Snapshot = serde_json::from_str(text).unwrap();
        assert_eq!(snapshot.series[0].kind, Kind::Movie);
        assert!(snapshot.torrents.is_none());
        assert!(snapshot.subscriptions.is_empty());
    }

    #[test]
    fn posters_live_under_the_api() {
        assert_eq!(poster_path(42), "/api/v1/posters/42");
    }
}
