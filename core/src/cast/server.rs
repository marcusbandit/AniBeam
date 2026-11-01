//! The server a receiver fetches from. Each cast gets an unguessable path; under it sit the
//! HLS playlists and segments of a prepared episode, its subtitles, the file itself with
//! Range for receivers that take it as it is, and one progressive MP4 stream from any point
//! for TVs that do not speak HLS. Receivers cannot send headers, so the token is the lock.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::media::Prepared;
use crate::host::stream::{Range, byte_range};

/// What one token serves.
pub struct Served {
    pub path: PathBuf,
    pub mime: String,
    pub prepared: Option<Arc<Prepared>>,
    /// Set for a stream another machine serves: the address of its playlist's directory,
    /// and every name under the token is fetched from there.
    pub relay: Option<String>,
}

#[derive(Default)]
pub struct Registry {
    entries: Mutex<HashMap<String, Arc<Served>>>,
}

impl Registry {
    pub fn add(&self, token: String, served: Served) {
        self.entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(token, Arc::new(served));
    }

    pub fn remove(&self, token: &str) {
        self.entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(token);
    }

    fn get(&self, token: &str) -> Option<Arc<Served>> {
        self.entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(token)
            .cloned()
    }
}

/// Binds on every interface, on `port` or any free one, and serves until the runtime ends.
pub async fn serve(registry: Arc<Registry>, port: u16) -> std::io::Result<SocketAddr> {
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    let at = listener.local_addr()?;
    let app = Router::new()
        .route("/c/{token}/{name}", get(file).options(preflight))
        .route("/c/{token}/seg/{name}", get(segment).options(preflight))
        .with_state(registry);
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::warn!("cast server ended: {e}");
        }
    });
    Ok(at)
}

fn cors(mut r: Response) -> Response {
    let h = r.headers_mut();
    h.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    h.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("*"),
    );
    h.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, HEAD, OPTIONS"),
    );
    h.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static("Content-Length, Content-Range, Accept-Ranges"),
    );
    r
}

async fn preflight() -> Response {
    cors(StatusCode::NO_CONTENT.into_response())
}

fn body(status: StatusCode, mime: &str, bytes: Vec<u8>) -> Response {
    cors((status, [(header::CONTENT_TYPE, mime.to_string())], bytes).into_response())
}

fn missing() -> Response {
    cors(StatusCode::NOT_FOUND.into_response())
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    tokio::task::spawn_blocking(f).await.ok()
}

async fn segment(
    State(reg): State<Arc<Registry>>,
    Path((token, name)): Path<(String, String)>,
) -> Response {
    let (Some(served), Some(i)) = (
        reg.get(&token),
        name.strip_suffix(".m4s")
            .and_then(|n| n.parse::<usize>().ok()),
    ) else {
        return missing();
    };
    let Some(p) = served.prepared.clone() else {
        return missing();
    };
    match blocking(move || p.segment(i)).await {
        Some(Ok(bytes)) => body(StatusCode::OK, "video/mp4", bytes),
        Some(Err(e)) => {
            tracing::warn!("cast segment {i}: {e}");
            cors(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        None => missing(),
    }
}

async fn file(
    State(reg): State<Arc<Registry>>,
    Path((token, name)): Path<(String, String)>,
    Query(q): Query<HashMap<String, String>>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let Some(served) = reg.get(&token) else {
        return missing();
    };
    if let Some(dir) = &served.relay {
        return relay(dir, &name, &q).await;
    }
    let p = served.prepared.clone();
    match (name.as_str(), p) {
        ("master.m3u8", Some(p)) => body(
            StatusCode::OK,
            "application/vnd.apple.mpegurl",
            p.master().into_bytes(),
        ),
        ("media.m3u8", Some(p)) => body(
            StatusCode::OK,
            "application/vnd.apple.mpegurl",
            p.media_playlist().into_bytes(),
        ),
        ("subs.m3u8", Some(p)) => body(
            StatusCode::OK,
            "application/vnd.apple.mpegurl",
            p.subs_playlist().into_bytes(),
        ),
        ("init.mp4", Some(p)) => body(StatusCode::OK, "video/mp4", p.init.clone()),
        ("subs.vtt", Some(p)) => body(
            StatusCode::OK,
            "text/vtt; charset=utf-8",
            p.vtt().into_bytes(),
        ),
        ("subs.srt", Some(p)) => body(
            StatusCode::OK,
            "text/srt; charset=utf-8",
            p.srt().into_bytes(),
        ),
        ("stream.mp4", Some(p)) => {
            let from: f64 = q.get("t").and_then(|t| t.parse().ok()).unwrap_or(0.0);
            progressive(p, from)
        }
        (n, _) if n.starts_with("file") => ranged(&served, &method, &headers).await,
        _ => missing(),
    }
}

/// One playlist or piece of a stream another machine serves, fetched from it whole. Its
/// pieces are a few seconds each, so holding one costs little, and a fetch may wait as long
/// as the other machine takes to convert it.
async fn relay(dir: &str, name: &str, q: &HashMap<String, String>) -> Response {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    let client = CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default()
    });
    // Only the names a playlist lists pass: no part of the request becomes a path elsewhere.
    let plain = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-');
    if !plain {
        return missing();
    }
    let mut url = format!("{dir}{name}");
    let mut sep = '?';
    let mut keys: Vec<_> = q.iter().collect();
    keys.sort();
    for (k, v) in keys {
        url.push(sep);
        url.push_str(&format!(
            "{}={}",
            crate::percent::encode(k),
            crate::percent::encode(v)
        ));
        sep = '&';
    }
    let answer = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            tracing::warn!("cast relay: {url} answered {}", r.status());
            return cors(StatusCode::BAD_GATEWAY.into_response());
        }
        Err(e) => {
            tracing::warn!("cast relay: {url}: {e}");
            return cors(StatusCode::BAD_GATEWAY.into_response());
        }
    };
    let kind = answer
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or(if name.ends_with(".m3u8") {
            "application/vnd.apple.mpegurl"
        } else {
            "video/mp2t"
        })
        .to_string();
    match answer.bytes().await {
        Ok(bytes) => body(StatusCode::OK, &kind, bytes.to_vec()),
        Err(_) => cors(StatusCode::BAD_GATEWAY.into_response()),
    }
}

/// The init segment, then every segment from the one holding `from`, as one MP4 stream with
/// no length: a TV seeks it by asking again from another point.
fn progressive(p: Arc<Prepared>, from: f64) -> Response {
    let first = p.segment_at((from * 1000.0) as u64);
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, std::io::Error>>(2);
    let init = p.init.clone();
    std::thread::spawn(move || {
        if tx.blocking_send(Ok(init)).is_err() {
            return;
        }
        for i in first..p.segments.len() {
            let chunk = p.segment(i).map_err(std::io::Error::other);
            let stop = chunk.is_err();
            if tx.blocking_send(chunk).is_err() || stop {
                return;
            }
        }
    });
    cors(
        (
            [(header::CONTENT_TYPE, "video/mp4")],
            Body::from_stream(Chunks(rx)),
        )
            .into_response(),
    )
}

/// The segments' channel as the stream a body reads.
struct Chunks(tokio::sync::mpsc::Receiver<Result<Vec<u8>, std::io::Error>>);

impl futures_core::Stream for Chunks {
    type Item = Result<Vec<u8>, std::io::Error>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_recv(cx)
    }
}

/// The file as it is, honouring one Range, which is how receivers seek it.
async fn ranged(served: &Served, method: &Method, headers: &HeaderMap) -> Response {
    let Ok(mut f) = tokio::fs::File::open(&served.path).await else {
        return missing();
    };
    let len = f.metadata().await.map(|m| m.len()).unwrap_or(0);
    let asked = headers.get(header::RANGE).and_then(|v| v.to_str().ok());
    let range = byte_range(asked, len);
    let mut h = HeaderMap::new();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&served.mime).unwrap_or(HeaderValue::from_static("video/mp4")),
    );
    h.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    // DLNA renderers look for these before they seek by byte.
    h.insert(
        "transferMode.dlna.org",
        HeaderValue::from_static("Streaming"),
    );
    h.insert(
        "contentFeatures.dlna.org",
        HeaderValue::from_static(
            "DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000",
        ),
    );
    let (status, start, count) = match range {
        Range::Part(s, e) => {
            h.insert(
                header::CONTENT_RANGE,
                HeaderValue::from_str(&format!("bytes {s}-{e}/{len}")).unwrap(),
            );
            (StatusCode::PARTIAL_CONTENT, s, e - s + 1)
        }
        Range::Unmet => {
            h.insert(
                header::CONTENT_RANGE,
                HeaderValue::from_str(&format!("bytes */{len}")).unwrap(),
            );
            return cors((StatusCode::RANGE_NOT_SATISFIABLE, h).into_response());
        }
        Range::Whole => (StatusCode::OK, 0, len),
    };
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(count));
    if *method == Method::HEAD {
        return cors((status, h).into_response());
    }
    if f.seek(std::io::SeekFrom::Start(start)).await.is_err() {
        return missing();
    }
    let stream = tokio_util::io::ReaderStream::with_capacity(f.take(count), 256 * 1024);
    cors((status, h, Body::from_stream(stream)).into_response())
}
