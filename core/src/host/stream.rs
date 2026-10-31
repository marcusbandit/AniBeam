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

use super::convert::{self, Converter, Key, Source};
use super::server::{Served, error};
use crate::contract::{Call, CoreError, Sidecar};
use crate::core::call_name;
use crate::sync::recover;

/// The parts of the host only streaming uses, made on first use.
#[derive(Default)]
pub struct Streams {
    converter: OnceLock<Converter>,
    probes: Mutex<HashMap<u64, Arc<wire::Probe>>>,
}

/// What a request's `Range` asks of a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Range {
    /// No range: the whole file.
    Whole,
    /// The first and the last byte.
    Part(u64, u64),
    /// A range the file cannot meet.
    Unmet,
}

/// The one range a request asks for of a `len` byte file.
pub fn byte_range(header: Option<&str>, len: u64) -> Range {
    let Some(header) = header else {
        return Range::Whole;
    };
    let parsed = header
        .strip_prefix("bytes=")
        .and_then(|v| v.split(',').next())
        .and_then(|v| {
            let (a, b) = v.split_once('-')?;
            let (start, end) = match (a.trim(), b.trim()) {
                ("", suffix) => (
                    len.saturating_sub(suffix.parse().ok()?),
                    len.saturating_sub(1),
                ),
                (a, "") => (a.parse().ok()?, len.saturating_sub(1)),
                (a, b) => (
                    a.parse().ok()?,
                    b.parse::<u64>().ok()?.min(len.saturating_sub(1)),
                ),
            };
            (start <= end && start < len).then_some((start, end))
        });
    parsed.map_or(Range::Unmet, |(s, e)| Range::Part(s, e))
}

/// The file and its sidecars, by row id.
fn file_of(st: &Served, id: u64) -> Option<(PathBuf, Vec<Sidecar>)> {
    let core = st.core.upgrade()?;
    core.store
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
                crate::library::sidecar::sidecars_of(&side),
            )
        })
}

fn not_found() -> Response {
    error(StatusCode::NOT_FOUND, "not found")
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
    recover(&st.streams.probes).insert(id, p.clone());
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
    if !convert::available().await {
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "this machine cannot convert: is ffmpeg installed?",
        );
    }
    let (audio, burn) = picks(&uri);
    if name == "index.m3u8" {
        let list = convert::playlist(probe.duration, &query_of(audio, burn));
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
    let Some(core) = st.core.upgrade() else {
        return not_found();
    };
    let converter = st
        .streams
        .converter
        .get_or_init(|| Converter::new(std::path::Path::new(&core.paths.cache_dir)));
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

/// One player call, run on this core as if its own player had made it.
pub async fn call(State(st): State<Arc<Served>>, body: Bytes) -> Response {
    let call: Call = match serde_json::from_slice(&body) {
        Ok(c) => c,
        Err(e) => return error(StatusCode::BAD_REQUEST, &format!("not a call: {e}")),
    };
    let name = call_name(&call);
    if !wire::PLAYER_CALLS.contains(&name.as_str()) {
        return error(StatusCode::FORBIDDEN, "only playback calls are taken here");
    }
    let Some(core) = st.core.upgrade() else {
        return error(StatusCode::SERVICE_UNAVAILABLE, "core is shutting down");
    };
    let reply = tokio::task::spawn_blocking(move || core.call(call)).await;
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
    fn a_range_reads_like_a_browser_sends_it() {
        assert_eq!(byte_range(None, 100), Range::Whole);
        assert_eq!(byte_range(Some("bytes=0-"), 100), Range::Part(0, 99));
        assert_eq!(byte_range(Some("bytes=10-19"), 100), Range::Part(10, 19));
        assert_eq!(byte_range(Some("bytes=-10"), 100), Range::Part(90, 99));
        assert_eq!(byte_range(Some("bytes=50-500"), 100), Range::Part(50, 99));
        assert_eq!(byte_range(Some("bytes=200-"), 100), Range::Unmet);
        assert_eq!(byte_range(Some("pages=1"), 100), Range::Unmet);
    }

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
