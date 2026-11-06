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
    /// Set for a stream another machine serves, passed on from here.
    pub relay: Option<Relay>,
}

/// A stream another machine serves.
pub struct Relay {
    /// The address of its playlist's directory: every name under the token is fetched
    /// from there.
    pub dir: String,
    /// Its pieces in order, for a TV that takes them as one progressive stream. Empty for
    /// a receiver that reads the playlist itself.
    pub pieces: Arc<Vec<Piece>>,
}

/// One piece of a media playlist: its name under the playlist's directory, query
/// included, and the second of the episode it starts at.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub name: String,
    pub start: f64,
}

/// The pieces a media playlist lists, in order. A name that is not a plain relative name
/// with an optional plain query is left out, so a playlist never points this machine
/// anywhere but under its own directory.
pub fn read_pieces(playlist: &str) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut at = 0.0;
    let mut length: Option<f64> = None;
    for line in playlist.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("#EXTINF:") {
            length = rest.split(',').next().and_then(|v| v.trim().parse().ok());
        } else if !line.is_empty() && !line.starts_with('#') {
            let Some(len) = length.take() else { continue };
            if relative(line) {
                out.push(Piece {
                    name: line.to_string(),
                    start: at,
                });
            }
            at += len;
        }
    }
    out
}

/// The piece holding second `secs`: the last one starting at or before it.
pub fn piece_at(pieces: &[Piece], secs: f64) -> usize {
    pieces
        .iter()
        .rposition(|p| p.start <= secs + 1e-6)
        .unwrap_or(0)
}

/// A name plain enough to pass on: letters, digits, dots, dashes and underscores, and
/// never `.` or `..`, which would climb out of the directory it is joined to.
fn plain(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// A plain name, maybe with a query of plain keys and values.
fn relative(entry: &str) -> bool {
    let (name, query) = entry.split_once('?').unwrap_or((entry, ""));
    plain(name)
        && query.split('&').filter(|p| !p.is_empty()).all(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            plain(k) && (v.is_empty() || plain(v))
        })
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
    if let Some(r) = &served.relay {
        if name == "stream.ts" && !r.pieces.is_empty() {
            let from: f64 = q.get("t").and_then(|t| t.parse().ok()).unwrap_or(0.0);
            return relayed_stream(r, from);
        }
        return relay(&r.dir, &name, &q).await;
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
    let client = relay_client();
    // Only the names a playlist lists pass: no part of the request becomes a path elsewhere.
    if !plain(name) {
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

/// The client every fetch from another machine goes through. It follows no redirect, so
/// what a playlist names is all that machine can make this one fetch.
pub(crate) fn relay_client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_default()
    })
}

/// Every piece of a relayed stream from the one holding `from`, fetched from the other
/// machine one after another, as one MPEG-TS stream with no length. TS pieces join end to
/// end, and each carries the episode's own times, so a TV reads them as one file. It
/// seeks by asking again from another point.
fn relayed_stream(r: &Relay, from: f64) -> Response {
    let pieces = r.pieces.clone();
    let dir = r.dir.clone();
    let first = piece_at(&pieces, from);
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, std::io::Error>>(2);
    tokio::spawn(async move {
        for p in &pieces[first..] {
            let url = format!("{dir}{}", p.name);
            let got = match relay_client().get(&url).send().await {
                Ok(a) if a.status().is_success() => a.bytes().await.map(|b| b.to_vec()),
                Ok(a) => {
                    tracing::warn!("cast relay: {url} answered {}", a.status());
                    let _ = tx
                        .send(Err(std::io::Error::other(
                            "the other machine refused a piece",
                        )))
                        .await;
                    return;
                }
                Err(e) => Err(e),
            };
            let chunk = got.map_err(std::io::Error::other);
            let stop = chunk.is_err();
            if tx.send(chunk).await.is_err() || stop {
                return;
            }
        }
    });
    cors(
        (
            [
                (header::CONTENT_TYPE, "video/mp2t"),
                (
                    header::HeaderName::from_static("transfermode.dlna.org"),
                    "Streaming",
                ),
            ],
            Body::from_stream(Chunks(rx)),
        )
            .into_response(),
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_converted_playlist_reads_as_pieces_with_their_starts() {
        let list = crate::host::convert::playlist(15.0, "?audio=1&burn=0");
        let pieces = read_pieces(&list);
        assert_eq!(
            pieces,
            vec![
                Piece {
                    name: "seg0.ts?audio=1&burn=0".into(),
                    start: 0.0
                },
                Piece {
                    name: "seg1.ts?audio=1&burn=0".into(),
                    start: 6.0
                },
                Piece {
                    name: "seg2.ts?audio=1&burn=0".into(),
                    start: 12.0
                },
            ]
        );
        assert_eq!(piece_at(&pieces, 0.0), 0);
        assert_eq!(piece_at(&pieces, 5.9), 0);
        assert_eq!(piece_at(&pieces, 6.0), 1);
        assert_eq!(piece_at(&pieces, 13.0), 2);
        assert_eq!(piece_at(&pieces, 99.0), 2);
        assert_eq!(piece_at(&[], 3.0), 0);
    }

    #[test]
    fn a_name_passed_on_never_climbs_out_of_its_directory() {
        assert!(plain("seg0.ts") && plain("index.m3u8"));
        for name in ["", ".", "..", "a/b", "../x", "%2e%2e", "seg0.ts?x=1"] {
            assert!(!plain(name), "{name}");
        }
    }

    #[test]
    fn a_playlist_cannot_point_this_machine_anywhere_else() {
        let list = "#EXTM3U\n#EXTINF:6.0,\nhttp://elsewhere/seg0.ts\n#EXTINF:6.0,\n../seg1.ts\n\
                    #EXTINF:6.0,\na/seg2.ts\n#EXTINF:6.0,\nseg3.ts?x=../y\n#EXTINF:6.0,\n..\n\
                    #EXTINF:6.0,\nseg5.ts?burn=0\n#EXT-X-ENDLIST\n";
        let pieces = read_pieces(list);
        // The refused entries still count toward the time, so what is left starts where
        // the playlist says.
        assert_eq!(
            pieces,
            vec![Piece {
                name: "seg5.ts?burn=0".into(),
                start: 30.0
            }]
        );
    }
}
