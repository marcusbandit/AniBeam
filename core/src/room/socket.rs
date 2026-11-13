//! The connection to the relay: `POST /rooms` to make a room, and one task per
//! room that owns its WebSocket, says hello, pings for the clock, hands every
//! message to the room, and connects again with the rejoin key when the
//! connection drops.
//!
//! The WebSocket is an HTTP/1.1 upgrade made by reqwest's own client, so it uses
//! the TLS every other request uses (rustls, the system's roots, the phone's
//! bundled ones); tokio-tungstenite only speaks frames over the upgraded stream.

use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use futures_util::{SinkExt, StreamExt};
use reqwest::StatusCode;
use reqwest::header::{CONNECTION, UPGRADE};
use tokio::sync::{Notify, mpsc, oneshot};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::{Role, WebSocketConfig};
use tokio_util::sync::CancellationToken;

use anibeam_room::proto::{
    ClientMsg, CreateRoom, CreatedRoom, EpisodeRef, ErrorCode, PROTO, ServerMsg,
};
use anibeam_room::room::{CLOSED_BYE, CLOSED_REPLACED};

use crate::contract::CoreError;
use crate::core::Core;

/// How long a connection to the relay may take to open.
pub const CONNECT_WAIT: Duration = Duration::from_secs(10);

/// Pings on connect, for the clock.
const CLOCK_PINGS: u32 = 5;

/// One ping a minute after those: the clock stays fresh and a NAT keeps the
/// connection open.
const PING_EVERY: Duration = Duration::from_secs(60);

/// The first wait before connecting again, doubled each time up to `BACKOFF_MAX`.
const BACKOFF_FIRST: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(30);

/// How long a dropped connection is tried again before the room is given up. The
/// relay keeps a member's place for as long.
const GIVE_UP: Duration = Duration::from_secs(120);

/// The largest message the relay sends is a welcome with ten members.
const MAX_MESSAGE: usize = 64 * 1024;

type Ws = WebSocketStream<reqwest::Upgraded>;

/// Why the relay could not be used.
#[derive(Debug)]
pub enum Fail {
    /// No room has that code (any more).
    NotFound,
    /// The relay answered, and said no.
    Refused(String),
    /// The relay did not answer.
    Unreachable(String),
}

impl Fail {
    pub fn to_error(&self) -> CoreError {
        match self {
            Fail::NotFound => super::error("not_found", crate::words!("core-room-not-found")),
            Fail::Refused(m) => {
                super::error("refused", crate::words!("core-room-refused", message = m))
            }
            Fail::Unreachable(m) => super::error(
                "unreachable",
                crate::words!("core-room-unreachable", message = m),
            ),
        }
    }
}

/// Makes a room on the relay: its code and the host key.
pub async fn create(
    client: &reqwest::Client,
    relay: &str,
    episode: &EpisodeRef,
) -> Result<CreatedRoom, Fail> {
    let response = client
        .post(format!("{relay}/rooms"))
        .json(&CreateRoom {
            episode: episode.clone(),
        })
        .send()
        .await
        .map_err(|e| Fail::Unreachable(crate::net::url_shown(relay) + ": " + &plain(&e)))?;
    let status = response.status();
    if status != StatusCode::CREATED {
        let body = response.text().await.unwrap_or_default();
        return Err(Fail::Refused(refusal(status, &body)));
    }
    response
        .json::<CreatedRoom>()
        .await
        .map_err(|e| Fail::Refused(plain(&e)))
}

/// Opens the room's WebSocket.
async fn open(client: &reqwest::Client, relay: &str, code: &str) -> Result<Ws, Fail> {
    let key = B64.encode(rand::random::<[u8; 16]>());
    let response = client
        .get(format!("{relay}/rooms/{code}/ws"))
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", &key)
        .send()
        .await
        .map_err(|e| Fail::Unreachable(crate::net::url_shown(relay) + ": " + &plain(&e)))?;
    match response.status() {
        StatusCode::SWITCHING_PROTOCOLS => {}
        StatusCode::NOT_FOUND => return Err(Fail::NotFound),
        status => {
            let body = response.text().await.unwrap_or_default();
            return Err(Fail::Refused(refusal(status, &body)));
        }
    }
    let accepted = response
        .headers()
        .get("sec-websocket-accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == derive_accept_key(key.as_bytes()));
    if !accepted {
        return Err(Fail::Refused("not a WebSocket answer".into()));
    }
    let upgraded = response
        .upgrade()
        .await
        .map_err(|e| Fail::Unreachable(plain(&e)))?;
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_MESSAGE))
        .max_frame_size(Some(MAX_MESSAGE));
    Ok(WebSocketStream::from_raw_socket(upgraded, Role::Client, Some(config)).await)
}

/// A reqwest error's own words, with its chain of causes.
fn plain(e: &dyn std::error::Error) -> String {
    let mut out = e.to_string();
    let mut source = e.source();
    while let Some(s) = source {
        out.push_str(": ");
        out.push_str(&s.to_string());
        source = s.source();
    }
    out
}

/// What a relay said when it would not do something: its status and the first
/// line of its answer.
fn refusal(status: StatusCode, body: &str) -> String {
    let line = body.lines().next().unwrap_or("").trim();
    let line: String = line.chars().take(200).collect();
    if line.is_empty() {
        status.to_string()
    } else {
        format!("{status}: {line}")
    }
}

/// Everything the task needs, handed over when the room starts.
pub struct Start {
    pub serial: u64,
    pub relay: String,
    pub code: String,
    pub name: String,
    pub host_key: Option<String>,
    pub client: reqwest::Client,
    pub rx: mpsc::UnboundedReceiver<ClientMsg>,
    pub cancel: CancellationToken,
    pub kick: Arc<Notify>,
    /// Answered once the first welcome came, or the first connection failed.
    pub first: oneshot::Sender<Result<(), CoreError>>,
}

/// How one connection ended.
enum End {
    /// This member left, or the room is no longer this core's: stop.
    Left,
    /// The relay ended it for good: the room closed, or the member was refused.
    Closed,
    /// It dropped; `welcomed` when it got as far as the welcome.
    Dropped { welcomed: bool },
}

/// The room's task: connects, and connects again with backoff for `GIVE_UP` after
/// a drop, until the member leaves or the room is gone.
pub async fn run(core: Weak<Core>, st: Start) {
    let Start {
        serial,
        relay,
        code,
        name,
        host_key,
        client,
        mut rx,
        cancel,
        kick,
        first,
    } = st;
    let mut first = Some(first);
    let mut rejoin: Option<String> = None;
    let mut lost: Option<Instant> = None;
    let mut wait = BACKOFF_FIRST;
    loop {
        let opened = tokio::select! {
            _ = cancel.cancelled() => return,
            r = tokio::time::timeout(CONNECT_WAIT, open(&client, &relay, &code)) => {
                r.unwrap_or_else(|_| Err(Fail::Unreachable("timed out".into())))
            }
        };
        let end = match opened {
            Ok(ws) => {
                let link = Link {
                    core: &core,
                    serial,
                    name: &name,
                    host_key: host_key.as_deref(),
                };
                link.serve(ws, &mut rx, &cancel, &kick, &mut rejoin, &mut first)
                    .await
            }
            Err(fail) => {
                if let Some(tx) = first.take() {
                    let _ = tx.send(Err(fail.to_error()));
                    return;
                }
                tracing::debug!("room {code}: could not connect: {fail:?}");
                match fail {
                    Fail::NotFound => End::Closed,
                    _ => End::Dropped { welcomed: false },
                }
            }
        };
        if let Some(tx) = first.take() {
            // The first connection ended before its welcome.
            let _ = tx.send(Err(
                Fail::Unreachable("the connection closed".into()).to_error()
            ));
            return;
        }
        match end {
            End::Left => return,
            End::Closed => {
                if let Some(core) = core.upgrade() {
                    super::finish(&core, serial);
                }
                return;
            }
            End::Dropped { welcomed } => {
                if welcomed {
                    lost = None;
                    wait = BACKOFF_FIRST;
                }
                let since = *lost.get_or_insert_with(Instant::now);
                let Some(strong) = core.upgrade() else {
                    return;
                };
                if !super::on_lost(&strong, serial) {
                    return;
                }
                if since.elapsed() >= GIVE_UP {
                    super::finish(&strong, serial);
                    return;
                }
                drop(strong);
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = tokio::time::sleep(wait) => {}
                }
                wait = (wait * 2).min(BACKOFF_MAX);
            }
        }
    }
}

/// One connection's view of the room it serves.
struct Link<'a> {
    core: &'a Weak<Core>,
    serial: u64,
    name: &'a str,
    host_key: Option<&'a str>,
}

impl Link<'_> {
    async fn serve(
        &self,
        ws: Ws,
        rx: &mut mpsc::UnboundedReceiver<ClientMsg>,
        cancel: &CancellationToken,
        kick: &Notify,
        rejoin: &mut Option<String>,
        first: &mut Option<oneshot::Sender<Result<(), CoreError>>>,
    ) -> End {
        let (mut sink, mut stream) = ws.split();
        let hello = ClientMsg::Hello {
            name: self.name.to_string(),
            proto: PROTO,
            client: format!("anibeam/{}", crate::VERSION),
            host_key: self.host_key.map(str::to_string),
            rejoin_key: rejoin.clone(),
        };
        if sink.send(text(&hello)).await.is_err() {
            return End::Dropped { welcomed: false };
        }
        let mut welcomed = false;
        let mut pings_left = 0;
        let mut minute =
            tokio::time::interval_at(tokio::time::Instant::now() + PING_EVERY, PING_EVERY);
        loop {
            // Biased so a `bye` queued before the cancel still goes out.
            tokio::select! {
                biased;
                out = rx.recv() => {
                    let Some(msg) = out else {
                        return End::Left;
                    };
                    let bye = matches!(msg, ClientMsg::Bye);
                    // Anything queued before the welcome is from before this
                    // connection: the welcome brings the room as it is now.
                    if !welcomed && !bye {
                        continue;
                    }
                    let sent = sink.send(text(&msg)).await.is_ok();
                    if bye {
                        let _ = sink.close().await;
                        return End::Left;
                    }
                    if !sent {
                        return End::Dropped { welcomed };
                    }
                }
                _ = cancel.cancelled() => {
                    let _ = sink.close().await;
                    return End::Left;
                }
                _ = kick.notified() => return End::Dropped { welcomed },
                _ = minute.tick() => {
                    if sink.send(text(&ping())).await.is_err() {
                        return End::Dropped { welcomed };
                    }
                }
                frame = stream.next() => {
                    let body = match frame {
                        Some(Ok(Message::Text(body))) => body,
                        Some(Ok(Message::Close(_))) | Some(Err(_)) | None => {
                            return End::Dropped { welcomed };
                        }
                        Some(Ok(_)) => continue,
                    };
                    let msg = match ServerMsg::parse(body.as_str()) {
                        Ok(msg) => msg,
                        Err(e) => {
                            tracing::debug!("room: a message that does not read: {e}");
                            continue;
                        }
                    };
                    let Some(core) = self.core.upgrade() else {
                        return End::Left;
                    };
                    match msg {
                        ServerMsg::Welcome { you, rejoin_key, room, server_ms } => {
                            *rejoin = Some(rejoin_key);
                            welcomed = true;
                            if !super::on_welcome(&core, self.serial, you, room, server_ms) {
                                return End::Left;
                            }
                            if let Some(tx) = first.take() {
                                let _ = tx.send(Ok(()));
                            }
                            pings_left = CLOCK_PINGS - 1;
                            if sink.send(text(&ping())).await.is_err() {
                                return End::Dropped { welcomed };
                            }
                        }
                        ServerMsg::Pong { c, s } => {
                            super::on_pong(&core, self.serial, c, s);
                            if pings_left > 0 {
                                pings_left -= 1;
                                if sink.send(text(&ping())).await.is_err() {
                                    return End::Dropped { welcomed };
                                }
                            }
                        }
                        ServerMsg::Error { code, message } => {
                            let fatal = code.is_fatal() || (code == ErrorCode::BadName && !welcomed);
                            if !fatal {
                                tracing::debug!("room: the relay said {code:?}: {message}");
                                continue;
                            }
                            if let Some(tx) = first.take() {
                                let _ = tx.send(Err(refused(code, &message)));
                                return End::Left;
                            }
                            tracing::debug!("room: the relay ended the connection, {code:?}: {message}");
                            return End::Closed;
                        }
                        ServerMsg::Closed { reason } => {
                            if reason == CLOSED_BYE {
                                return End::Left;
                            }
                            if reason == CLOSED_REPLACED {
                                tracing::debug!("room: another connection took this member's place");
                            }
                            return End::Closed;
                        }
                        other => {
                            if !super::on_message(&core, self.serial, other) {
                                return End::Left;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The error a refusing relay's code reads as.
fn refused(code: ErrorCode, message: &str) -> CoreError {
    match code {
        ErrorCode::NotFound => Fail::NotFound.to_error(),
        ErrorCode::Full => super::error("full", crate::words!("core-room-full")),
        ErrorCode::Proto => super::error("proto", crate::words!("core-room-proto")),
        _ => Fail::Refused(message.to_string()).to_error(),
    }
}

fn ping() -> ClientMsg {
    ClientMsg::Ping {
        c: super::local_ms(),
    }
}

fn text(msg: &ClientMsg) -> Message {
    Message::Text(msg.to_json().into())
}
