//! The host's episodes, for a client to play where it is: the file itself with
//! byte ranges, its subtitle sidecars, what is in it, the same converted as
//! HLS (`convert`), and the playback calls that keep its history here.
//!
//! A file is named by its row id in the library and nothing else, so no part
//! of a request is ever used as a path.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use anibeam_host_api as wire;
use axum::body::{Body, Bytes};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rusqlite::{OptionalExtension, params};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::Front;
use super::convert::{self, Converter, Key, Source};
use super::server::{Served, error};
use anibeam_contract::{Call, CoreError, Sidecar, call_name};
use anibeam_foundation::range::{Range, byte_range};
use anibeam_foundation::services::Services;
use anibeam_foundation::sync::recover;

/// The parts of the host only streaming uses, made on first use.
#[derive(Default)]
pub struct Streams {
    converter: OnceLock<Converter>,
    probes: Mutex<HashMap<u64, Arc<wire::Probe>>>,
    subtitles: Mutex<HashMap<(u64, usize), Bytes>>,
}

/// The file and its sidecars, by row id.
fn file_of(st: &Served, id: u64) -> Option<(PathBuf, Vec<Sidecar>)> {
    let team = st.live()?;
    team.services
        .store
        .read(|c| {
            Ok(c.query_row(
                "SELECT path, sidecars FROM files WHERE id = ?1",
                params![id as i64],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?)
        })
        .ok()
        .flatten()
        .map(|(path, side)| {
            (
                PathBuf::from(path),
                anibeam_library::sidecar::sidecars_of(&side),
            )
        })
}

fn not_found() -> Response {
    error(StatusCode::NOT_FOUND, "not found")
}

/// A file whose probed length is missing, broken or past a day, which no
/// playlist lists.
fn unconvertible() -> Response {
    error(
        StatusCode::UNPROCESSABLE_ENTITY,
        "the file's length is missing or past a day, so it cannot be converted",
    )
}

/// By extension, for the players that care.
fn video_type(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .as_deref()
    {
        Some("mp4" | "m4v") => "video/mp4",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        Some("ts" | "m2ts") => "video/mp2t",
        Some("ogm") => "video/ogg",
        _ => "video/x-matroska",
    }
}

pub async fn file(
    State(st): State<Arc<Served>>,
    Path(id): Path<u64>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let Some((path, _)) = file_of(&st, id) else {
        return not_found();
    };
    let kind = video_type(&path);
    ranged(path, kind, &method, &headers).await
}

pub async fn sidecar(
    State(st): State<Arc<Served>>,
    Path((id, index)): Path<(u64, usize)>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let Some(side) = file_of(&st, id).and_then(|(_, s)| s.into_iter().nth(index)) else {
        return not_found();
    };
    ranged(
        PathBuf::from(side.path),
        "text/plain; charset=utf-8",
        &method,
        &headers,
    )
    .await
}

/// The file as it is, honouring one range.
async fn ranged(path: PathBuf, kind: &str, method: &Method, headers: &HeaderMap) -> Response {
    let Ok(mut f) = tokio::fs::File::open(&path).await else {
        return not_found();
    };
    let len = f.metadata().await.map(|m| m.len()).unwrap_or(0);
    let asked = headers.get(header::RANGE).and_then(|v| v.to_str().ok());
    let mut h = HeaderMap::new();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(kind).unwrap_or(HeaderValue::from_static("video/x-matroska")),
    );
    h.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    let (status, start, count) = match byte_range(asked, len) {
        Range::Part(s, e) => {
            if let Ok(v) = HeaderValue::from_str(&format!("bytes {s}-{e}/{len}")) {
                h.insert(header::CONTENT_RANGE, v);
            }
            (StatusCode::PARTIAL_CONTENT, s, e - s + 1)
        }
        Range::Whole => (StatusCode::OK, 0, len),
        Range::Unmet => {
            if let Ok(v) = HeaderValue::from_str(&format!("bytes */{len}")) {
                h.insert(header::CONTENT_RANGE, v);
            }
            return (StatusCode::RANGE_NOT_SATISFIABLE, h).into_response();
        }
    };
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(count));
    if *method == Method::HEAD {
        return (status, h).into_response();
    }
    if f.seek(std::io::SeekFrom::Start(start)).await.is_err() {
        return not_found();
    }
    let stream = tokio_util::io::ReaderStream::with_capacity(f.take(count), 256 * 1024);
    (status, h, Body::from_stream(stream)).into_response()
}

async fn probe_of(st: &Served, id: u64) -> Option<(PathBuf, Arc<wire::Probe>)> {
    let (path, _) = file_of(st, id)?;
    if let Some(p) = recover(&st.streams.probes).get(&id) {
        return Some((path, p.clone()));
    }
    let p = Arc::new(convert::probe(&path).await?);
    // A length no playlist lists may only mean the file is still arriving,
    // so it is read again next time rather than kept.
    if wire::hls_converts(p.duration) {
        remember(&st.streams.probes, id, p.clone());
    }
    Some((path, p))
}

pub async fn probe(State(st): State<Arc<Served>>, Path(id): Path<u64>) -> Response {
    match probe_of(&st, id).await {
        Some((_, p)) => match serde_json::to_vec(&*p) {
            Ok(body) => json(StatusCode::OK, body),
            Err(_) => not_found(),
        },
        None => error(
            StatusCode::SERVICE_UNAVAILABLE,
            "this machine cannot read the file: is ffprobe installed?",
        ),
    }
}

pub async fn subtitle(
    State(st): State<Arc<Served>>,
    Path((id, index)): Path<(u64, usize)>,
) -> Response {
    let ass = |bytes: Bytes| {
        (
            [(header::CONTENT_TYPE, "text/x-ssa; charset=utf-8")],
            Body::from(bytes),
        )
            .into_response()
    };
    if let Some(bytes) = recover(&st.streams.subtitles).get(&(id, index)) {
        return ass(bytes.clone());
    }
    let Some((path, probe)) = probe_of(&st, id).await else {
        return not_found();
    };
    match probe.subtitles.get(index) {
        Some(t) if !convert::is_bitmap(&t.codec) => {}
        _ => return not_found(),
    }
    let Some(bytes) = convert::subtitle(&path, index).await else {
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "the subtitles could not be read",
        );
    };
    let bytes = Bytes::from(bytes);
    remember(&st.streams.subtitles, (id, index), bytes.clone());
    ass(bytes)
}

pub async fn skip(State(st): State<Arc<Served>>, Path(session): Path<u64>) -> Response {
    let Some(front) = st.live().and_then(|team| team.front.get()) else {
        return not_found();
    };
    match front.skip_windows(session) {
        Ok(windows) => match serde_json::to_vec(&windows) {
            Ok(body) => json(StatusCode::OK, body),
            Err(_) => not_found(),
        },
        Err(_) => not_found(),
    }
}

/// A cache of things read from files holds at most this many; past it, it
/// starts over.
const CACHE_CAP: usize = 64;

fn remember<K: std::hash::Hash + Eq, V>(map: &Mutex<HashMap<K, V>>, key: K, value: V) {
    let mut map = recover(map);
    if map.len() >= CACHE_CAP {
        map.clear();
    }
    map.insert(key, value);
}

/// `audio` and `burn` from the query, the only two it takes.
fn picks(uri: &Uri) -> (Option<usize>, Option<usize>) {
    let mut audio = None;
    let mut burn = None;
    for pair in uri.query().unwrap_or_default().split('&') {
        match pair.split_once('=') {
            Some(("audio", v)) => audio = v.parse().ok(),
            Some(("burn", v)) => burn = v.parse().ok(),
            _ => {}
        }
    }
    (audio, burn)
}

/// `?audio=1&burn=0`, or nothing, so each piece asks for the same conversion.
fn query_of(audio: Option<usize>, burn: Option<usize>) -> String {
    let mut parts = Vec::new();
    if let Some(a) = audio {
        parts.push(format!("audio={a}"));
    }
    if let Some(b) = burn {
        parts.push(format!("burn={b}"));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

pub async fn hls(
    State(st): State<Arc<Served>>,
    Path((id, height, name)): Path<(u64, u32, String)>,
    uri: Uri,
) -> Response {
    let Some((path, probe)) = probe_of(&st, id).await else {
        return not_found();
    };
    if !wire::hls_converts(probe.duration) {
        return unconvertible();
    }
    if !convert::available().await {
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "this machine cannot convert: is ffmpeg installed?",
        );
    }
    let (audio, burn) = picks(&uri);
    // A track the file does not have would only make ffmpeg fail.
    if audio.is_some_and(|a| a >= probe.audio.len())
        || burn.is_some_and(|b| b >= probe.subtitles.len())
    {
        return error(StatusCode::BAD_REQUEST, "the file has no such track");
    }
    if name == "index.m3u8" {
        let Some(list) = wire::hls_playlist(probe.duration, &query_of(audio, burn)) else {
            return unconvertible();
        };
        return (
            [
                (header::CONTENT_TYPE, "application/vnd.apple.mpegurl"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            list,
        )
            .into_response();
    }
    let Some(i) = name
        .strip_prefix("seg")
        .and_then(|n| n.strip_suffix(".ts"))
        .and_then(|n| n.parse::<u32>().ok())
    else {
        return not_found();
    };
    let key = Key {
        file: id,
        height: convert::height_for(height),
        audio,
        burn,
    };
    let source = Source {
        path,
        probe: (*probe).clone(),
    };
    let Some(team) = st.live() else {
        return not_found();
    };
    let converter = st
        .streams
        .converter
        .get_or_init(|| Converter::new(std::path::Path::new(&team.services.paths.cache_dir)));
    match converter.piece(&key, &source, i).await {
        Some(bytes) => ([(header::CONTENT_TYPE, "video/mp2t")], Body::from(bytes)).into_response(),
        None => error(StatusCode::SERVICE_UNAVAILABLE, "the conversion failed"),
    }
}

fn json(status: StatusCode, body: Vec<u8>) -> Response {
    (
        status,
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
        .into_response()
}

/// What a device of someone else on the tailnet is told when it sends a download.
pub const DOWNLOADS_OWNER_ONLY: &str =
    "Only the devices of the person this computer belongs to can download to it.";

/// How long a download call waits for its job, under the phone's 20 second limit on a
/// request. Adding a torrent takes a second or two; a torrent client that is still
/// starting can take longer. Past this the reply is the job's id, and the torrents show
/// up in the next snapshot.
const DOWNLOAD_WAIT: std::time::Duration = std::time::Duration::from_secs(15);

/// A call that starts a job, run to the job's end: the device that sent it hears whether
/// the torrents went in, not only that a job started. The listener is up before the call,
/// so a job that ends at once is not missed.
pub(crate) fn run_to_end(
    services: &Services,
    front: &dyn Front,
    profile: u64,
    call: Call,
    wait: std::time::Duration,
) -> Result<anibeam_contract::Reply, CoreError> {
    use anibeam_contract::{Event, EventBody, EventListener, JobPhase, Reply};
    struct Relay(std::sync::mpsc::Sender<Event>);
    impl EventListener for Relay {
        fn on_event(&self, event: Event) {
            let _ = self.0.send(event);
        }
    }
    let (tx, rx) = std::sync::mpsc::channel();
    // Removed from the bus when this function returns, on every path.
    let _listening =
        anibeam_foundation::events::Listening::new(services.bus.clone(), Arc::new(Relay(tx)));
    let job = match front.answer(profile, call) {
        Ok(Reply::Started { job }) => job,
        other => return other,
    };
    let until = std::time::Instant::now() + wait;
    loop {
        let left = until.saturating_duration_since(std::time::Instant::now());
        let Ok(event) = rx.recv_timeout(left) else {
            break Ok(Reply::Started { job });
        };
        let Some(r) = event.job.as_ref().filter(|r| r.id == job) else {
            continue;
        };
        match event.body {
            EventBody::JobFailed { error } => break Err(error),
            EventBody::JobCancelled => {
                break Err(CoreError::internal("the download was cancelled there"));
            }
            _ if r.phase == JobPhase::Finished => break Ok(Reply::Ok),
            _ => {}
        }
    }
}

/// One player call, run on this core as if its own player had made it.
pub async fn call(
    State(st): State<Arc<Served>>,
    axum::Extension(super::server::Profile(profile)): axum::Extension<super::server::Profile>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // A web page can send a plain form or text POST anywhere without asking,
    // but JSON only after a preflight this host never answers, so a page in
    // a browser on the tailnet cannot make these calls.
    let is_json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.trim().starts_with("application/json"));
    if !is_json {
        return error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "send the call as application/json",
        );
    }
    let call: Call = match serde_json::from_slice(&body) {
        Ok(c) => c,
        Err(e) => return error(StatusCode::BAD_REQUEST, &format!("not a call: {e}")),
    };
    let name = call_name(&call);
    let download = wire::DOWNLOAD_CALLS.contains(&name.as_str());
    if !download && !wire::PLAYER_CALLS.contains(&name.as_str()) {
        return error(
            StatusCode::FORBIDDEN,
            "only playback and download calls are taken here",
        );
    }
    // Downloads fill this computer's disk and its torrent client, so only the
    // person it belongs to may send them (#733).
    if download && profile != anibeam_foundation::trust::OWNER {
        return error(StatusCode::FORBIDDEN, DOWNLOADS_OWNER_ONLY);
    }
    let Some((team, front)) = st
        .live()
        .and_then(|team| team.front.get().map(|front| (team, front)))
    else {
        return error(StatusCode::SERVICE_UNAVAILABLE, "core is shutting down");
    };
    let hiding = matches!(name.as_str(), "SetNotInterested" | "ListNotInterested");
    if !hiding && !download && !team.streaming() {
        return error(StatusCode::FORBIDDEN, super::server::STREAMING_OFF);
    }
    let reply = tokio::task::spawn_blocking(move || {
        if download {
            run_to_end(&team.services, &*front, profile, call, DOWNLOAD_WAIT)
        } else {
            front.answer(profile, call)
        }
    })
    .await;
    match reply {
        Ok(Ok(reply)) => match serde_json::to_vec(&reply) {
            Ok(body) => json(StatusCode::OK, body),
            Err(e) => error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        },
        Ok(Err(e)) => match serde_json::to_vec(&CallError::from(&e)) {
            Ok(body) => json(StatusCode::UNPROCESSABLE_ENTITY, body),
            Err(_) => error(StatusCode::UNPROCESSABLE_ENTITY, &e.to_string()),
        },
        Err(e) => error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// A refused call: the plain message, and the core's error for a client that
/// reads it.
#[derive(serde::Serialize)]
struct CallError {
    error: String,
    core: CoreError,
}

impl From<&CoreError> for CallError {
    fn from(e: &CoreError) -> Self {
        CallError {
            error: e.to_string(),
            core: e.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_picks_round_trip_through_the_query() {
        let uri: Uri = "/x/index.m3u8?audio=1&burn=0".parse().unwrap();
        assert_eq!(picks(&uri), (Some(1), Some(0)));
        assert_eq!(query_of(Some(1), Some(0)), "?audio=1&burn=0");
        assert_eq!(query_of(None, None), "");
        let bare: Uri = "/x/index.m3u8".parse().unwrap();
        assert_eq!(picks(&bare), (None, None));
    }
}
