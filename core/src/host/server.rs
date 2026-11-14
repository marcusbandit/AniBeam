//! The routes. Every one is a `GET` and reads, but for the playback calls
//! (`stream`); anything else is told so in a small JSON error. The snapshot is built at most once every thirty seconds
//! however many clients ask.

use std::collections::HashMap;
use std::path::Path as FsPath;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use anibeam_host_api as wire;
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};

use super::build::{self, LibraryPart};
use super::{convert, history, keys, peers, stream, trust};
use crate::contract::CoreError;
use crate::core::Core;
use crate::foundation::machine::hostname;
use crate::foundation::trust::{Asker, TailnetUser};
use crate::time;

/// How long a built snapshot answers every request.
pub const CACHE_FOR: Duration = Duration::from_secs(30);
/// How long a client may keep a poster before it asks again.
const POSTER_MAX_AGE: &str = "max-age=86400";

pub struct Served {
    pub(super) core: Weak<Core>,
    pub(super) streams: stream::Streams,
    host: String,
    /// The built snapshot per profile, each behind its own lock, so one person's
    /// build never holds up another's cached answer (#631).
    snapshot: std::sync::Mutex<HashMap<u64, Arc<SnapshotSlot>>>,
    library: tokio::sync::Mutex<Option<(Instant, Arc<LibraryPart>)>>,
    /// The profile each person was answered as, by Tailscale user id, the owner under
    /// "": a profile never changes its person, so it is looked up once per launch.
    profiles: std::sync::Mutex<HashMap<String, u64>>,
    peers: tokio::sync::Mutex<Option<(Instant, Arc<Vec<wire::PeerHost>>)>>,
}

pub fn router(core: Weak<Core>) -> Router {
    // The encoders are tried now, so the first phone to say hello hears
    // whether this machine converts.
    let _ = convert::known();
    let served = Arc::new(Served {
        core,
        streams: stream::Streams::default(),
        host: hostname(),
        snapshot: std::sync::Mutex::new(HashMap::new()),
        library: tokio::sync::Mutex::new(None),
        profiles: std::sync::Mutex::new(HashMap::new()),
        peers: tokio::sync::Mutex::new(None),
    });
    // The episodes themselves answer only while "Stream episodes" is on.
    let streaming = Router::new()
        .route("/api/v1/files/{id}", get(stream::file))
        .route("/api/v1/files/{id}/sidecars/{index}", get(stream::sidecar))
        .route("/api/v1/files/{id}/probe", get(stream::probe))
        .route(
            "/api/v1/files/{id}/subtitles/{index}",
            get(stream::subtitle),
        )
        .route("/api/v1/files/{id}/hls/{height}/{name}", get(stream::hls))
        .route(wire::CALL, post(stream::call))
        .route("/api/v1/sessions/{id}/skip", get(stream::skip))
        .layer(axum::middleware::from_fn_with_state(
            served.clone(),
            streaming_on,
        ));
    Router::new()
        .route(wire::HELLO, get(hello))
        .route(wire::SNAPSHOT, get(snapshot))
        .route(wire::PEERS, get(peer_list))
        .route(wire::HISTORY, get(history_document))
        .route("/api/v1/posters/{id}", get(poster))
        .merge(streaming)
        .fallback(not_found)
        .method_not_allowed_fallback(not_allowed)
        .layer(axum::middleware::from_fn_with_state(
            served.clone(),
            trusted,
        ))
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

/// One profile's cached snapshot.
type SnapshotSlot = tokio::sync::Mutex<Option<(Instant, Bytes)>>;

/// The profile a request is answered as (#631), set by [`trusted`] before any
/// route runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile(pub u64);

/// The person a request is answered as, as the tailnet names them right now, set beside
/// [`Profile`] (#633). None when the tailnet does not say, as on a tagged machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person(pub Option<TailnetUser>);

/// The sentence a phone shows when it asks for an episode here while
/// streaming is off.
pub const STREAMING_OFF: &str =
    "Streaming is off on that computer. Turn on Stream episodes there, under Settings, Phone app.";

/// Refuses the episode routes while "Stream episodes" is off. The playback
/// calls that only keep Not interested the same on both sides still pass.
async fn streaming_on(
    State(st): State<Arc<Served>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let on = st.core.upgrade().is_some_and(|c| c.host.streaming());
    if on || request.uri().path() == wire::CALL {
        next.run(request).await
    } else {
        error(StatusCode::FORBIDDEN, STREAMING_OFF)
    }
}

/// Refuses a request from a device the person has not trusted: one switched
/// off in Settings, one the host cannot place on the tailnet while any is,
/// and anything off the tailnet (`trust::decide`). A request may also carry
/// a proof of the device that sent it (`keys`): one that does not check out
/// is refused wherever it comes from, and one from a paired device is
/// answered from off the tailnet too. It runs before every route.
async fn trusted(
    State(st): State<Arc<Served>>,
    ConnectInfo(from): ConnectInfo<std::net::SocketAddr>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let Some(core) = st.core.upgrade() else {
        return error(StatusCode::SERVICE_UNAVAILABLE, "core is shutting down");
    };
    let header = request
        .headers()
        .get(wire::proof::HEADER)
        .map(|h| h.to_str().unwrap_or_default().to_string());
    let method = request.method().as_str().to_string();
    let path = request.uri().path_and_query().map_or_else(
        || request.uri().path().to_string(),
        |p| p.as_str().to_string(),
    );
    // Reading the tailnet can run `tailscale`, and the key the keyring, so
    // both stay off the async threads.
    let served = st.clone();
    let judged = tokio::task::spawn_blocking(move || {
        core.host.keys.mine(&core.paths.device_key_path());
        let verdict = core.host.verdict(from.ip());
        let proof = core
            .host
            .keys
            .judge(header.as_deref(), &method, &path, time::now_secs());
        // A paired proof opens only an address off the tailnet. One the host
        // cannot place there might be the device the person switched off, so it
        // stays refused whatever key it holds.
        let refused = match (&verdict, &proof) {
            (_, keys::Proof::Refused(why)) => Some(*why),
            (trust::Verdict::Outside, keys::Proof::Paired(_)) => None,
            _ => trust::refusal(&verdict),
        };
        if refused.is_some() {
            return Ok((verdict, proof, refused, None));
        }
        // Who asks (#631). A device paired from off the tailnet was paired by the
        // owner, so it is the owner's; a tailnet device is its Tailscale user's.
        let (asker, me) = match (&verdict, &proof) {
            (trust::Verdict::Outside, keys::Proof::Paired(_)) => (
                Asker::Owner,
                core.host.tailnet_now().and_then(|t| t.owner().cloned()),
            ),
            _ => core.host.asker(from.ip()),
        };
        // Who it is, as the tailnet names them now, for the hello (#633). The profile
        // id below is cached for the launch; the name is read fresh with every request.
        let who = match &asker {
            Asker::Owner => me.clone(),
            Asker::Person(user) => Some(user.clone()),
            Asker::Unknown => None,
        };
        let key = match &asker {
            Asker::Owner => String::new(),
            Asker::Person(user) => user.id.clone(),
            Asker::Unknown => return Ok((verdict, proof, None, None)),
        };
        let known = served
            .profiles
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&key)
            .copied();
        if let Some(id) = known {
            return Ok((verdict, proof, None, Some((id, who))));
        }
        let now = time::now_secs();
        let learned = me.is_some();
        let profile = core
            .store
            .write(move |c| crate::profiles::resolve(c, &asker, me.as_ref(), now))?;
        // The owner is remembered only once their Tailscale user is known, so
        // profile 1 learns it from the first request that can tell.
        if let Some(id) = profile
            && (!key.is_empty() || learned)
        {
            served
                .profiles
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(key, id);
        }
        Ok::<_, CoreError>((verdict, proof, None, profile.map(|id| (id, who))))
    })
    .await;
    let (verdict, proof, refused, profile) = match judged {
        Ok(Ok(judged)) => judged,
        Ok(Err(e)) => return failed(&e),
        Err(_) => (
            trust::Verdict::Unknown,
            keys::Proof::None,
            trust::refusal(&trust::Verdict::Unknown),
            None,
        ),
    };
    if let Some(why) = refused {
        tracing::debug!("host refused {from}: {verdict:?}, {proof:?}");
        return error(StatusCode::FORBIDDEN, why);
    }
    let Some((profile, who)) = profile else {
        tracing::debug!("host refused {from}: no person for it on the tailnet");
        return error(StatusCode::FORBIDDEN, trust::UNKNOWN_PERSON);
    };
    let mut request = request;
    request.extensions_mut().insert(Profile(profile));
    request.extensions_mut().insert(Person(who));
    next.run(request).await
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

async fn hello(
    State(st): State<Arc<Served>>,
    ConnectInfo(from): ConnectInfo<std::net::SocketAddr>,
    profile: Option<axum::Extension<Profile>>,
    person: Option<axum::Extension<Person>>,
) -> Response {
    // Downloads are offered only to the owner's devices, the only ones the call route
    // takes them from (#733).
    let owner = profile
        .as_ref()
        .is_some_and(|p| p.0.0 == crate::foundation::trust::OWNER);
    // `trusted` read the key before this route, so this only takes it.
    let key = st
        .core
        .upgrade()
        .and_then(|c| c.host.keys.mine(&c.paths.device_key_path()));
    let streaming = st.core.upgrade().is_some_and(|c| c.host.streaming());
    let relayed = st
        .core
        .upgrade()
        .is_some_and(|core| super::relayed(&core, from.ip()));
    // Whose progress this device gets (#633): named as the tailnet names them now, or as
    // the profile was last seen when the tailnet does not say.
    let id = profile.map(|p| p.0.0);
    let fresh = person
        .and_then(|p| p.0.0)
        .zip(id)
        .and_then(|(user, id)| crate::profiles::shown(&user, id));
    let named = match (fresh, st.core.upgrade(), id) {
        (Some(named), _, _) => Some(named),
        (None, Some(core), Some(id)) => {
            tokio::task::spawn_blocking(move || core.store.read(|c| crate::profiles::named(c, id)))
                .await
                .ok()
                .and_then(Result::ok)
                .flatten()
        }
        _ => None,
    };
    let hello = wire::Hello {
        app: wire::APP.to_string(),
        api: wire::API_VERSION,
        version: crate::VERSION.to_string(),
        host: st.host.clone(),
        now: time::now_secs(),
        features: features(streaming, key.is_some(), owner),
        key: key.map(|k| k.public()).unwrap_or_default(),
        streaming_off: !streaming,
        relayed,
        profile: named,
    };
    match serde_json::to_vec(&hello) {
        Ok(body) => json(Bytes::from(body)),
        Err(e) => failed(&e.into()),
    }
}

/// What this host serves past the snapshot: the list of other AniBeams and
/// this one's history always, since neither is streaming; downloads to the
/// owner's devices, the only ones that may send them; request
/// proofs while it has its key; streaming while its switch is on;
/// converting too when ffmpeg and an encoder work here, which the first
/// hello starts finding out rather than waits for, so hello stays quick.
fn features(streaming: bool, keyed: bool, owner: bool) -> Vec<String> {
    let mut out = vec![
        wire::FEATURE_PEERS.to_string(),
        wire::FEATURE_HISTORY.to_string(),
    ];
    if owner {
        out.push(wire::FEATURE_DOWNLOADS.to_string());
    }
    if keyed {
        out.push(wire::FEATURE_PROOF.to_string());
    }
    if !streaming {
        return out;
    }
    out.push(wire::FEATURE_STREAM.to_string());
    if convert::known() == Some(true) {
        out.push(wire::FEATURE_CONVERT.to_string());
    }
    out
}

/// The other AniBeams on the tailnet. Asking them all takes a moment, so the
/// list answers every request for `CACHE_FOR`; the slot stays locked while it
/// is made, so a burst of clients waits for one round of hellos. A tailnet
/// that cannot be read gives no list rather than an empty one, and nothing is
/// kept, so the next request asks again.
async fn peer_list(
    State(st): State<Arc<Served>>,
    ConnectInfo(from): ConnectInfo<std::net::SocketAddr>,
) -> Response {
    let mut slot = st.peers.lock().await;
    let fresh = slot
        .as_ref()
        .filter(|(at, _)| at.elapsed() < CACHE_FOR)
        .map(|(_, hosts)| hosts.clone());
    let hosts = match fresh {
        Some(hosts) => hosts,
        None => {
            let Some(core) = st.core.upgrade() else {
                return error(StatusCode::SERVICE_UNAVAILABLE, "core is shutting down");
            };
            let asked = tokio::task::spawn_blocking(move || {
                let tailnet = core.host.tailnet(false)?;
                Some(peers::candidates(
                    &tailnet,
                    &core.host.switched_off(),
                    wire::DEFAULT_PORT,
                ))
            })
            .await
            .ok()
            .flatten();
            let Some(asked) = asked else {
                return error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "this computer cannot read its tailnet right now",
                );
            };
            let hosts = Arc::new(peers::find(asked).await);
            *slot = Some((Instant::now(), hosts.clone()));
            hosts
        }
    };
    drop(slot);
    let list = wire::Peers {
        hosts: peers::others(&hosts, from.ip()),
    };
    match serde_json::to_vec(&list) {
        Ok(body) => json(Bytes::from(body)),
        Err(e) => failed(&e.into()),
    }
}

async fn snapshot(
    State(st): State<Arc<Served>>,
    axum::Extension(Profile(profile)): axum::Extension<Profile>,
) -> Response {
    match snapshot_body(&st, profile).await {
        Ok(body) => json(body),
        Err(e) => failed(&e),
    }
}

/// The cached snapshot of one profile, or a fresh one. The slots stay locked
/// while one is built, so a burst of clients waits for one build instead of
/// starting many.
async fn snapshot_body(st: &Served, profile: u64) -> Result<Bytes, CoreError> {
    let slot = st
        .snapshot
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .entry(profile)
        .or_default()
        .clone();
    let mut slot = slot.lock().await;
    if let Some((at, body)) = slot.as_ref()
        && at.elapsed() < CACHE_FOR
    {
        return Ok(body.clone());
    }
    let core = st
        .core
        .upgrade()
        .ok_or_else(|| CoreError::internal("core is shutting down"))?;
    let (snapshot, library) = build::snapshot(&core, &st.host, profile).await?;
    let body = Bytes::from(serde_json::to_vec(&snapshot)?);
    // The posters are everyone's; the owner's read keeps their table fresh.
    if profile == crate::foundation::trust::OWNER {
        *st.library.lock().await = Some((Instant::now(), Arc::new(library)));
    }
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
    let part = tokio::task::spawn_blocking(move || {
        build::library_part(&core, crate::foundation::trust::OWNER)
    })
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

/// This computer's series and history, for another AniBeam to merge
/// (`history`). The asker is pulled from in turn, so two computers that
/// see each other both end up with everything.
async fn history_document(
    State(st): State<Arc<Served>>,
    ConnectInfo(from): ConnectInfo<std::net::SocketAddr>,
) -> Response {
    let Some(core) = st.core.upgrade() else {
        return error(StatusCode::SERVICE_UNAVAILABLE, "core is shutting down");
    };
    let built = tokio::task::spawn_blocking(move || {
        let doc = history::document(&core);
        history::pull_back(&core, from.ip());
        doc
    })
    .await;
    match built {
        Ok(Ok(doc)) => match serde_json::to_vec(&doc) {
            Ok(body) => json(Bytes::from(body)),
            Err(e) => failed(&e.into()),
        },
        Ok(Err(e)) => failed(&e),
        Err(e) => error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn only_this_machine_or_an_address_passes_the_host_check() {
        use super::host_allowed;
        assert!(host_allowed("100.64.0.30:8790", "desktop"));
        assert!(host_allowed("[fd7a:115c:a1e0::1]:8790", "desktop"));
        assert!(host_allowed("desktop.example.ts.net:8790", "desktop"));
        assert!(host_allowed("desktop:8790", "desktop"));
        assert!(!host_allowed("evil.example:8790", "desktop"));
        assert!(!host_allowed("", "desktop"));
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
