//! The native watch together relay (#846): rooms in memory, one
//! [`anibeam_room::Room`] per code, driven over WebSocket.
//!
//! Routes:
//!
//! - `POST /rooms` with `{"episode": EpisodeRef}` makes a room: `201` with
//!   `{"code", "host_key"}`.
//! - `GET /rooms/{code}/ws` joins one over WebSocket; `404` when it does not exist.
//! - `GET /{code}` is the join page a shared link opens.
//! - `GET /health` answers `ok`.
//!
//! Each address gets 10 room creations, 30 WebSocket opens and 60 join pages a
//! minute (an IPv6 address counts by its /64). Behind a reverse proxy every
//! client has the proxy's address, so they all share one limit. Each socket
//! may send 20 messages a second, and one that has not said hello within 10 s
//! is closed.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anibeam_room::limit::{self, HELLO_MS, Verdict, Window, limit_key};
use anibeam_room::proto::{ClientMsg, CreateRoom, CreatedRoom};
use anibeam_room::{ConnId, Out, Room, code, page};
use axum::Router;
use axum::body::Bytes;
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, DefaultBodyLimit, Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

/// Room creations one address may make a minute.
pub const CREATES_PER_MINUTE: u32 = 10;

/// WebSocket opens one address may make a minute.
pub const OPENS_PER_MINUTE: u32 = 30;

/// Join pages one address may load a minute.
pub const PAGES_PER_MINUTE: u32 = 60;

/// The most rooms open at once.
const MAX_ROOMS: usize = 10_000;

/// The largest `POST /rooms` body and the largest WebSocket message.
const MAX_BODY: usize = 16 * 1024;

/// Frames waiting for one socket; a client that falls this far behind is dropped.
const QUEUE: usize = 256;

/// What a relay may be set up with.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// How long a socket may stay open without saying hello.
    pub hello_wait: Duration,
    /// The time in ms since the epoch, as every limit and room timer reads
    /// it. Tests stop it so a rate holds however slow the machine is.
    pub clock: fn() -> u64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            hello_wait: Duration::from_millis(HELLO_MS),
            clock: now_ms,
        }
    }
}

/// Serves the relay on `listener` until the process ends. Tests bind
/// `127.0.0.1:0` and spawn this.
pub async fn serve(listener: TcpListener) -> std::io::Result<()> {
    serve_with(listener, Options::default()).await
}

/// [`serve`] with options other than the defaults.
pub async fn serve_with(listener: TcpListener, options: Options) -> std::io::Result<()> {
    let relay = Relay::default();
    relay.lock().options = options;
    tokio::spawn(tick_loop(Arc::downgrade(&relay.0)));
    let app = Router::new()
        .route("/health", get(health))
        .route("/rooms", post(create))
        .route("/rooms/{code}/ws", get(join))
        .route("/{code}", get(join_page))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .with_state(relay);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}

#[derive(Clone, Default)]
struct Relay(Arc<Mutex<Inner>>);

#[derive(Default)]
struct Inner {
    rooms: HashMap<String, Entry>,
    limits: HashMap<IpAddr, Limit>,
    next_conn: u64,
    options: Options,
}

/// A room and the sockets in it, member or not yet.
struct Entry {
    room: Room,
    sockets: HashMap<ConnId, mpsc::Sender<Frame>>,
}

enum Frame {
    Text(String),
    Close,
}

/// One address's counts for the minute that started at `start_ms`.
#[derive(Default)]
struct Limit {
    start_ms: u64,
    creates: u32,
    opens: u32,
    pages: u32,
}

#[derive(Clone, Copy)]
enum Kind {
    Create,
    Open,
    Page,
}

impl Relay {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        lock(&self.0)
    }
}

fn lock(inner: &Mutex<Inner>) -> MutexGuard<'_, Inner> {
    inner
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Inner {
    /// Counts one creation or open against `ip`; false when it is over the limit.
    fn allow(&mut self, ip: IpAddr, kind: Kind, now_ms: u64) -> bool {
        let limit = self.limits.entry(limit_key(ip)).or_insert(Limit {
            start_ms: now_ms,
            ..Limit::default()
        });
        if now_ms >= limit.start_ms + 60_000 {
            *limit = Limit {
                start_ms: now_ms,
                ..Limit::default()
            };
        }
        let (count, max) = match kind {
            Kind::Create => (&mut limit.creates, CREATES_PER_MINUTE),
            Kind::Open => (&mut limit.opens, OPENS_PER_MINUTE),
            Kind::Page => (&mut limit.pages, PAGES_PER_MINUTE),
        };
        if *count >= max {
            return false;
        }
        *count += 1;
        true
    }
}

impl Entry {
    /// Sends each message to its connections, and closes a connection after a
    /// message that ends it.
    fn send(&mut self, outs: Vec<Out>) {
        for out in outs {
            let text = out.msg.to_json();
            let ends = out.msg.ends_connection();
            for conn in self.room.targets(&out.to) {
                self.push(conn, Frame::Text(text.clone()));
                if ends {
                    self.push(conn, Frame::Close);
                }
            }
        }
    }

    fn push(&mut self, conn: ConnId, frame: Frame) {
        if let Some(tx) = self.sockets.get(&conn)
            && tx.try_send(frame).is_err()
        {
            // Too far behind, or gone: dropping the sender ends its socket.
            self.sockets.remove(&conn);
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn random<const N: usize>() -> Option<[u8; N]> {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).ok()?;
    Some(buf)
}

async fn health() -> &'static str {
    "ok"
}

async fn create(
    State(relay): State<Relay>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Bytes,
) -> Response {
    let now = relay.now();
    let mut inner = relay.lock();
    if !inner.allow(addr.ip(), Kind::Create, now) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "too many rooms, try again in a minute",
        )
            .into_response();
    }
    let Ok(request) = serde_json::from_slice::<CreateRoom>(&body) else {
        return (StatusCode::BAD_REQUEST, "expected {\"episode\": ...}").into_response();
    };
    if inner.rooms.len() >= MAX_ROOMS {
        return (StatusCode::SERVICE_UNAVAILABLE, "the relay is full").into_response();
    }
    let mut room_code = None;
    for _ in 0..32 {
        let Some(bytes) = random::<{ code::LEN }>() else {
            break;
        };
        let candidate = code::generate(bytes);
        if !inner.rooms.contains_key(&candidate) {
            room_code = Some(candidate);
            break;
        }
    }
    let (Some(room_code), Some(key), Some(secret)) = (room_code, random::<16>(), random::<16>())
    else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "no room code").into_response();
    };
    let host_key = code::key(key);
    let room = Room::new(
        room_code.clone(),
        host_key.clone(),
        request.episode,
        now,
        secret,
    );
    inner.rooms.insert(
        room_code.clone(),
        Entry {
            room,
            sockets: HashMap::new(),
        },
    );
    drop(inner);
    let body = serde_json::to_string(&CreatedRoom {
        code: room_code,
        host_key,
    })
    .expect("a created room serializes");
    (
        StatusCode::CREATED,
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
        .into_response()
}

async fn join(
    State(relay): State<Relay>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(raw): Path<String>,
    upgrade: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> Response {
    let now = relay.now();
    let room_code = {
        let mut inner = relay.lock();
        if !inner.allow(addr.ip(), Kind::Open, now) {
            return (StatusCode::TOO_MANY_REQUESTS, "too many connections").into_response();
        }
        match code::normalize(&raw) {
            Some(c) if inner.rooms.contains_key(&c) => c,
            _ => return (StatusCode::NOT_FOUND, "no such room").into_response(),
        }
    };
    match upgrade {
        Ok(upgrade) => upgrade
            .max_message_size(MAX_BODY)
            .on_upgrade(move |socket| run_socket(relay, room_code, socket)),
        Err(rejection) => rejection.into_response(),
    }
}

async fn join_page(
    State(relay): State<Relay>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(raw): Path<String>,
) -> Response {
    let Some(room_code) = code::normalize(&raw) else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    let view = {
        let mut inner = relay.lock();
        let now = (inner.options.clock)();
        if !inner.allow(addr.ip(), Kind::Page, now) {
            return (StatusCode::TOO_MANY_REQUESTS, "too many pages").into_response();
        }
        inner.rooms.get(&room_code).map(|e| e.room.view())
    };
    let status = if view.is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    };
    (status, Html(page::join_html(&room_code, view.as_ref()))).into_response()
}

/// One socket's life: frames in go to the room, frames out come from the queue
/// the room's messages are pushed onto. Each socket has its own rate, member or
/// not, and one that has not said hello by the deadline is closed.
async fn run_socket(relay: Relay, room_code: String, mut socket: WebSocket) {
    let (tx, mut rx) = mpsc::channel(QUEUE);
    let (conn, hello_wait, clock) = {
        let mut inner = relay.lock();
        let conn = ConnId(inner.next_conn);
        inner.next_conn += 1;
        let Options { hello_wait, clock } = inner.options;
        let conn = inner.rooms.get_mut(&room_code).map(|entry| {
            entry.sockets.insert(conn, tx);
            conn
        });
        (conn, hello_wait, clock)
    };
    let Some(conn) = conn else {
        // The room closed between the upgrade and now.
        let _ = socket.send(Message::Close(None)).await;
        return;
    };
    let mut window = Window::new(clock());
    let deadline = tokio::time::sleep(hello_wait);
    tokio::pin!(deadline);
    let mut awaiting_hello = true;
    loop {
        tokio::select! {
            _ = &mut deadline, if awaiting_hello => {
                awaiting_hello = false;
                if !relay.is_member(&room_code, conn) {
                    let _ = socket.send(Message::Text(limit::no_hello().to_json().into())).await;
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }
            }
            frame = rx.recv() => match frame {
                Some(Frame::Text(text)) => {
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        // The client may have closed right after its last
                        // words, a bye most often: read those before ending.
                        drain(&mut socket, &relay, &room_code, conn).await;
                        break;
                    }
                }
                Some(Frame::Close) | None => {
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }
            },
            incoming = socket.recv() => {
                let text = match incoming {
                    Some(Ok(Message::Text(text))) => text,
                    // The protocol is text only; anything else gets bad_message.
                    Some(Ok(Message::Binary(_))) => Default::default(),
                    Some(Ok(Message::Ping(_) | Message::Pong(_))) => continue,
                    Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                };
                match window.hit(clock()) {
                    Verdict::Pass => relay.on_text(&room_code, conn, text.as_str()),
                    // Leaving is never dropped, however fast the socket was.
                    _ if is_bye(text.as_str()) => relay.on_text(&room_code, conn, text.as_str()),
                    Verdict::Refuse => {
                        let refusal = limit::refusal().to_json();
                        if socket.send(Message::Text(refusal.into())).await.is_err() {
                            break;
                        }
                    }
                    Verdict::Drop => {}
                }
            }
        }
    }
    relay.on_close(&room_code, conn);
}

impl Relay {
    fn now(&self) -> u64 {
        (self.lock().options.clock)()
    }

    fn is_member(&self, room_code: &str, conn: ConnId) -> bool {
        self.lock()
            .rooms
            .get(room_code)
            .is_some_and(|entry| entry.room.is_member(conn))
    }

    fn on_text(&self, room_code: &str, conn: ConnId, text: &str) {
        let now = self.now();
        let mut inner = self.lock();
        if let Some(entry) = inner.rooms.get_mut(room_code) {
            let outs = entry.room.handle_text(conn, text, now);
            entry.send(outs);
        }
    }

    fn on_close(&self, room_code: &str, conn: ConnId) {
        let now = self.now();
        let mut inner = self.lock();
        if let Some(entry) = inner.rooms.get_mut(room_code) {
            entry.sockets.remove(&conn);
            let outs = entry.room.disconnect(conn, now);
            entry.send(outs);
        }
    }
}

/// Once a second: the rooms' timers, dropping closed rooms, and forgetting old
/// limit counts. Ends when the relay is gone.
async fn tick_loop(relay: Weak<Mutex<Inner>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        interval.tick().await;
        let Some(relay) = relay.upgrade() else {
            return;
        };
        let mut inner = lock(&relay);
        let now = (inner.options.clock)();
        for entry in inner.rooms.values_mut() {
            let outs = entry.room.tick(now);
            entry.send(outs);
        }
        inner.rooms.retain(|_, entry| !entry.room.is_closed());
        inner.limits.retain(|_, l| now < l.start_ms + 60_000);
    }
}

/// Reads what a socket sent before it closed, once a write to it has failed,
/// so a `bye` that raced a broadcast still leaves the room properly.
async fn drain(socket: &mut WebSocket, relay: &Relay, room_code: &str, conn: ConnId) {
    let wait = std::time::Duration::from_millis(500);
    while let Ok(Some(Ok(Message::Text(text)))) = tokio::time::timeout(wait, socket.recv()).await {
        relay.on_text(room_code, conn, text.as_str());
    }
}

/// Whether a frame is a `bye`, which the per socket rate never drops.
fn is_bye(text: &str) -> bool {
    matches!(ClientMsg::parse(text), Ok(ClientMsg::Bye))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_count_per_kind_and_reset_each_minute() {
        let mut inner = Inner::default();
        let ip: IpAddr = "198.51.100.7".parse().unwrap();
        for _ in 0..CREATES_PER_MINUTE {
            assert!(inner.allow(ip, Kind::Create, 1_000));
        }
        assert!(!inner.allow(ip, Kind::Create, 1_000));
        for _ in 0..OPENS_PER_MINUTE {
            assert!(inner.allow(ip, Kind::Open, 2_000));
        }
        assert!(!inner.allow(ip, Kind::Open, 2_000));
        for _ in 0..PAGES_PER_MINUTE {
            assert!(inner.allow(ip, Kind::Page, 3_000));
        }
        assert!(!inner.allow(ip, Kind::Page, 3_000));
        let other: IpAddr = "203.0.113.9".parse().unwrap();
        assert!(inner.allow(other, Kind::Create, 2_000));
        assert!(inner.allow(ip, Kind::Create, 61_000));
    }
}
