//! The routes. Every one is a `GET` and reads, but for the playback calls
//! (`stream`); anything else is told so in a small JSON error. The snapshot is built at most once every thirty seconds
//! however many clients ask.

use std::path::Path as FsPath;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use anibeam_host_api as wire;
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};

use super::build::{self, LibraryPart};
use super::{convert, stream};
use crate::contract::CoreError;
use crate::core::Core;
use crate::time;

/// How long a built snapshot answers every request.
pub const CACHE_FOR: Duration = Duration::from_secs(30);
/// How long a client may keep a poster before it asks again.
const POSTER_MAX_AGE: &str = "max-age=86400";

pub struct Served {
    pub(super) core: Weak<Core>,
    pub(super) streams: stream::Streams,
    host: String,
    snapshot: tokio::sync::Mutex<Option<(Instant, Bytes)>>,
    library: tokio::sync::Mutex<Option<(Instant, Arc<LibraryPart>)>>,
}

/// The machine's name, for `Hello::host`.
pub fn hostname() -> String {
    #[cfg(unix)]
    return rustix::system::uname()
        .nodename()
        .to_string_lossy()
        .into_owned();
    #[cfg(not(unix))]
    return std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows".to_string());
}

pub fn router(core: Weak<Core>) -> Router {
    let served = Arc::new(Served {
        core,
        streams: stream::Streams::default(),
        host: hostname(),
        snapshot: tokio::sync::Mutex::new(None),
        library: tokio::sync::Mutex::new(None),
    });
    Router::new()
        .route(wire::HELLO, get(hello))
        .route(wire::SNAPSHOT, get(snapshot))
        .route("/api/v1/posters/{id}", get(poster))
        .route("/api/v1/files/{id}", get(stream::file))
        .route("/api/v1/files/{id}/sidecars/{index}", get(stream::sidecar))
        .route("/api/v1/files/{id}/probe", get(stream::probe))
        .route(
            "/api/v1/files/{id}/subtitles/{index}",
            get(stream::subtitle),
        )
        .route("/api/v1/files/{id}/hls/{height}/{name}", get(stream::hls))
        .route(wire::CALL, post(stream::call))
        .fallback(not_found)
        .method_not_allowed_fallback(not_allowed)
        .layer(axum::middleware::from_fn(known_host))
        .with_state(served)
}

/// Refuses a request whose `Host` names somewhere other than this machine,
/// so a web page that rebinds its own domain to a tailnet address cannot
/// read the library. An address, the machine's name, or a tailnet name
/// passes.
async fn known_host(request: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if host_allowed(host, &hostname()) {
        next.run(request).await
    } else {
        error(StatusCode::MISDIRECTED_REQUEST, "unknown host")
    }
}

pub fn host_allowed(host: &str, machine: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(v6) => {
            return v6
                .split(']')
                .next()
                .is_some_and(|ip| ip.parse::<std::net::Ipv6Addr>().is_ok());
        }
        None => host.rsplit_once(':').map_or(host, |(h, _)| h),
    }
    .trim_end_matches('.')
    .to_ascii_lowercase();
    if name.parse::<std::net::IpAddr>().is_ok() {
        return true;
    }
    let machine = machine.to_ascii_lowercase();
    let first = name.split('.').next().unwrap_or_default();
    name.ends_with(".ts.net")
        || name == "localhost"
        || (!machine.is_empty() && first == machine.split('.').next().unwrap_or_default())
}

pub(super) fn error(status: StatusCode, message: &str) -> Response {
    let body = serde_json::json!({ "error": message }).to_string();
    (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}

async fn not_found() -> Response {
    error(StatusCode::NOT_FOUND, "not found")
}

async fn not_allowed() -> Response {
    error(
        StatusCode::METHOD_NOT_ALLOWED,
        "this host answers GET, and POST for playback calls",
    )
}

fn json(body: Bytes) -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        Body::from(body),
    )
        .into_response()
}

fn failed(e: &CoreError) -> Response {
    tracing::warn!("host request failed: {e}");
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "could not read the library",
    )
}

async fn hello(State(st): State<Arc<Served>>) -> Response {
    let hello = wire::Hello {
        app: wire::APP.to_string(),
        api: wire::API_VERSION,
        version: crate::VERSION.to_string(),
        host: st.host.clone(),
        now: time::now_secs(),
        features: features().await,
    };
    match serde_json::to_vec(&hello) {
        Ok(body) => json(Bytes::from(body)),
        Err(e) => failed(&e.into()),
    }
}

/// What this host serves past the snapshot. Streaming always; converting
/// when ffmpeg and an encoder work here.
async fn features() -> Vec<String> {
    let mut out = vec![wire::FEATURE_STREAM.to_string()];
    if convert::available().await {
        out.push(wire::FEATURE_CONVERT.to_string());
    }
    out
}

async fn snapshot(State(st): State<Arc<Served>>) -> Response {
    match snapshot_body(&st).await {
        Ok(body) => json(body),
        Err(e) => failed(&e),
    }
}

/// The cached snapshot, or a fresh one. The slot stays locked while it is
/// built, so a burst of clients waits for one build instead of starting many.
async fn snapshot_body(st: &Served) -> Result<Bytes, CoreError> {
    let mut slot = st.snapshot.lock().await;
    if let Some((at, body)) = slot.as_ref()
        && at.elapsed() < CACHE_FOR
    {
        return Ok(body.clone());
    }
    let core = st
        .core
        .upgrade()
        .ok_or_else(|| CoreError::internal("core is shutting down"))?;
    let (snapshot, library) = build::snapshot(&core, &st.host).await?;
    let body = Bytes::from(serde_json::to_vec(&snapshot)?);
    *st.library.lock().await = Some((Instant::now(), Arc::new(library)));
    *slot = Some((Instant::now(), body.clone()));
    Ok(body)
}

/// The library half alone, for a poster request that comes before any
/// snapshot, or long after the last one.
async fn library(st: &Served) -> Result<Arc<LibraryPart>, CoreError> {
    let mut slot = st.library.lock().await;
    if let Some((at, part)) = slot.as_ref()
        && at.elapsed() < CACHE_FOR
    {
        return Ok(part.clone());
    }
    let core = st
        .core
        .upgrade()
        .ok_or_else(|| CoreError::internal("core is shutting down"))?;
    let part = tokio::task::spawn_blocking(move || build::library_part(&core))
        .await
        .map_err(|e| CoreError::internal(format!("library read: {e}")))??;
    let part = Arc::new(part);
    *slot = Some((Instant::now(), part.clone()));
    Ok(part)
}

/// The poster of one series. The id picks a file out of the table the last
/// snapshot made; no part of the request is ever used as a path.
async fn poster(
    State(st): State<Arc<Served>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Ok(id) = id.parse::<u64>() else {
        return not_found().await;
    };
    let part = match library(&st).await {
        Ok(part) => part,
        Err(e) => return failed(&e),
    };
    let Some(poster) = part.posters.get(&id) else {
        return not_found().await;
    };
    let etag = format!("\"{}\"", poster.tag);
    let cache = [
        (header::ETAG, etag.clone()),
        (header::CACHE_CONTROL, POSTER_MAX_AGE.to_string()),
    ];
    let matches = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .any(|t| t.trim().trim_start_matches("W/") == etag)
        });
    if matches {
        return (StatusCode::NOT_MODIFIED, cache).into_response();
    }
    let Ok(bytes) = tokio::fs::read(&poster.path).await else {
        return not_found().await;
    };
    let kind = content_type(&poster.path, &bytes);
    (
        [
            (header::CONTENT_TYPE, kind.to_string()),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, POSTER_MAX_AGE.to_string()),
        ],
        Body::from(bytes),
    )
        .into_response()
}

/// By extension first; a file whose extension says nothing is read by its
/// first bytes.
pub fn content_type(path: &FsPath, bytes: &[u8]) -> &'static str {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "avif" => "image/avif",
        _ if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) => "image/jpeg",
        _ if bytes.starts_with(&[0x89, b'P', b'N', b'G']) => "image/png",
        _ if bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" => "image/webp",
        _ if bytes.starts_with(b"GIF8") => "image/gif",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn only_this_machine_or_an_address_passes_the_host_check() {
        use super::host_allowed;
        assert!(host_allowed("100.94.40.21:8790", "banditbox"));
        assert!(host_allowed("[fd7a:115c:a1e0::1]:8790", "banditbox"));
        assert!(host_allowed(
            "banditbox.tail24c6a9.ts.net:8790",
            "banditbox"
        ));
        assert!(host_allowed("banditbox:8790", "banditbox"));
        assert!(!host_allowed("evil.example:8790", "banditbox"));
        assert!(!host_allowed("", "banditbox"));
    }

    use super::*;

    #[test]
    fn the_content_type_follows_the_extension_then_the_bytes() {
        assert_eq!(content_type(FsPath::new("a/b.JPG"), b""), "image/jpeg");
        assert_eq!(content_type(FsPath::new("a/b.png"), b""), "image/png");
        assert_eq!(content_type(FsPath::new("a/b.webp"), b""), "image/webp");
        assert_eq!(
            content_type(FsPath::new("a/b"), &[0xFF, 0xD8, 0xFF, 0xE0]),
            "image/jpeg"
        );
        assert_eq!(
            content_type(FsPath::new("a/b"), b"RIFF\0\0\0\0WEBPVP8 "),
            "image/webp"
        );
        assert_eq!(
            content_type(FsPath::new("a/b"), b"zzzz"),
            "application/octet-stream"
        );
    }
}
