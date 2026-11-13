//! The watch together relay (#846) as a Cloudflare Worker on the free plan.
//!
//! The same routes as the native relay in `apps/relay`:
//!
//! - `POST /rooms` with `{"episode": EpisodeRef}` makes a room: `201` with
//!   `{"code", "host_key"}`.
//! - `GET /rooms/{code}/ws` joins one over WebSocket; `404` when it does not exist.
//! - `GET /{code}` is the join page a shared link opens.
//! - `GET /health` answers `ok`.
//!
//! Each room is one SQLite-backed Durable Object named by its code, holding one
//! [`anibeam_room::Room`]. Sockets use the WebSocket Hibernation API, so a room
//! nobody is talking in is evicted from memory and costs nothing. Because
//! eviction loses memory, the room is written to storage after every change that
//! others see, and each socket carries its [`ConnId`] as a tag. The room's
//! timers (the 120 s grace and the 10 minute empty close) run on a Durable
//! Object alarm that is only set while one of them is pending.

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};

use anibeam_room::proto::{CreateRoom, CreatedRoom, EpisodeRef, RoomView};
use anibeam_room::room::{EMPTY_MS, GRACE_MS};
use anibeam_room::{ConnId, Out, Room, To, code, page};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use worker::{
    Context, Date, DurableObject, Env, Method, Request, RequestInit, Response, Result,
    ScheduledTime, State, WebSocket, WebSocketIncomingMessage, WebSocketPair, durable_object,
    event, js_sys,
};

/// The Durable Object namespace binding in `wrangler.toml`.
const ROOMS: &str = "ROOMS";

/// Rate limit bindings: room creations and WebSocket opens per address a minute
/// (10 and 30, set in `wrangler.toml`, as on the native relay).
const CREATE_LIMIT: &str = "CREATE_LIMIT";
const OPEN_LIMIT: &str = "OPEN_LIMIT";

/// The largest `POST /rooms` body and the largest WebSocket message.
const MAX_BODY: usize = 16 * 1024;

/// How many codes `POST /rooms` tries before giving up.
const CODE_TRIES: usize = 32;

/// The storage key the room lives under.
const KEY: &str = "room";

/// The most timer deadlines a room remembers. Past this, middle ones are
/// dropped: a later deadline still runs everything that is overdue by then.
const MAX_DEADLINES: usize = 32;

/// The host the Worker uses for its own requests to a room object.
const INTERNAL: &str = "https://room.internal";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = crypto, js_name = getRandomValues)]
    fn get_random_values(buf: &mut [u8]);
}

fn random<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    get_random_values(&mut buf);
    buf
}

fn now_ms() -> u64 {
    Date::now().as_millis()
}

fn text(status: u16, body: &str) -> Result<Response> {
    Ok(Response::ok(body)?.with_status(status))
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();
    let method = req.method();
    if path == "/health" {
        return match method {
            Method::Get | Method::Head => Response::ok("ok"),
            _ => text(405, "method not allowed"),
        };
    }
    if path == "/rooms" {
        return match method {
            Method::Post => create(req, &env).await,
            _ => text(405, "method not allowed"),
        };
    }
    if let Some(raw) = path
        .strip_prefix("/rooms/")
        .and_then(|rest| rest.strip_suffix("/ws"))
        .filter(|raw| !raw.contains('/'))
    {
        return match method {
            Method::Get => join(req, &env, raw).await,
            _ => text(405, "method not allowed"),
        };
    }
    if let Some(raw) = path.strip_prefix('/').filter(|raw| !raw.contains('/')) {
        return match method {
            Method::Get | Method::Head => join_page(&env, raw).await,
            _ => text(405, "method not allowed"),
        };
    }
    text(404, "not found")
}

/// Counts one request from the caller's address against `binding`. A missing
/// binding or a failed check lets the request through.
async fn allowed(env: &Env, binding: &str, req: &Request) -> bool {
    let Ok(limiter) = env.rate_limiter(binding) else {
        return true;
    };
    let ip = req
        .headers()
        .get("CF-Connecting-IP")
        .ok()
        .flatten()
        .unwrap_or_default();
    limiter
        .limit(limit_key(&ip))
        .await
        .map(|outcome| outcome.success)
        .unwrap_or(true)
}

/// An IPv6 address counts by its /64, which one machine usually has whole.
fn limit_key(ip: &str) -> String {
    match ip.parse::<IpAddr>().map(|ip| ip.to_canonical()) {
        Ok(IpAddr::V6(v6)) => {
            let s = v6.segments();
            Ipv6Addr::new(s[0], s[1], s[2], s[3], 0, 0, 0, 0).to_string()
        }
        Ok(v4) => v4.to_string(),
        Err(_) => ip.to_owned(),
    }
}

/// What the Worker hands a room object to set it up.
#[derive(Serialize, Deserialize)]
struct Init {
    code: String,
    episode: EpisodeRef,
}

async fn create(mut req: Request, env: &Env) -> Result<Response> {
    if !allowed(env, CREATE_LIMIT, &req).await {
        return text(429, "too many rooms, try again in a minute");
    }
    let declared = req
        .headers()
        .get("Content-Length")
        .ok()
        .flatten()
        .and_then(|len| len.parse::<usize>().ok());
    if declared.is_some_and(|len| len > MAX_BODY) {
        return text(413, "length limit exceeded");
    }
    let body = req.bytes().await?;
    if body.len() > MAX_BODY {
        return text(413, "length limit exceeded");
    }
    let Ok(request) = serde_json::from_slice::<CreateRoom>(&body) else {
        return text(400, "expected {\"episode\": ...}");
    };
    let namespace = env.durable_object(ROOMS)?;
    for _ in 0..CODE_TRIES {
        let room_code = code::generate(random());
        let init = serde_json::to_string(&Init {
            code: room_code.clone(),
            episode: request.episode.clone(),
        })?;
        let mut options = RequestInit::new();
        options
            .with_method(Method::Post)
            .with_body(Some(JsValue::from_str(&init)));
        let internal = Request::new_with_init(&format!("{INTERNAL}/init"), &options)?;
        let stub = namespace.id_from_name(&room_code)?.get_stub()?;
        let mut answer = stub.fetch_with_request(internal).await?;
        match answer.status_code() {
            201 => {
                let created: CreatedRoom = answer.json().await?;
                return Ok(Response::from_json(&created)?.with_status(201));
            }
            // That code is taken: try another.
            409 => continue,
            status => return text(status, "the room could not be made"),
        }
    }
    text(500, "no room code")
}

async fn join(req: Request, env: &Env, raw: &str) -> Result<Response> {
    if !allowed(env, OPEN_LIMIT, &req).await {
        return text(429, "too many connections");
    }
    let Some(room_code) = code::normalize(raw) else {
        return text(404, "no such room");
    };
    let stub = env
        .durable_object(ROOMS)?
        .id_from_name(&room_code)?
        .get_stub()?;
    stub.fetch_with_request(req).await
}

async fn join_page(env: &Env, raw: &str) -> Result<Response> {
    let Some(room_code) = code::normalize(raw) else {
        return text(404, "not found");
    };
    let stub = env
        .durable_object(ROOMS)?
        .id_from_name(&room_code)?
        .get_stub()?;
    let mut answer = stub.fetch_with_str(&format!("{INTERNAL}/view")).await?;
    let view = if answer.status_code() == 200 {
        answer.json::<RoomView>().await.ok()
    } else {
        None
    };
    let status = if view.is_some() { 200 } else { 404 };
    Ok(Response::from_html(page::join_html(&room_code, view.as_ref()))?.with_status(status))
}

/// What a room object keeps in storage.
#[derive(Serialize, Deserialize)]
struct Stored {
    room: Room,
    /// When the room's timers may next have work, soonest first.
    deadlines: Vec<u64>,
    /// The alarm last set, so an unchanged one is not written again.
    alarm: Option<u64>,
}

impl Stored {
    fn add_deadline(&mut self, at: u64) {
        if let Err(i) = self.deadlines.binary_search(&at) {
            self.deadlines.insert(i, at);
        }
        if self.deadlines.len() > MAX_DEADLINES {
            self.deadlines.remove(MAX_DEADLINES / 2);
        }
    }

    fn connected(&self) -> usize {
        self.room.targets(&To::All).len()
    }
}

#[derive(Default)]
struct Cache {
    /// True once storage has been read since the object woke.
    loaded: bool,
    /// `None` for a code with no room.
    stored: Option<Stored>,
}

/// One room. Its memory is a cache of what storage holds.
#[durable_object]
pub struct RoomObject {
    state: State,
    cache: RefCell<Cache>,
}

impl DurableObject for RoomObject {
    fn new(state: State, _env: Env) -> Self {
        Self {
            state,
            cache: RefCell::new(Cache::default()),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let path = req.path();
        match (req.method(), path.as_str()) {
            (Method::Post, "/init") => self.init(req).await,
            (Method::Get, "/view") => self.view().await,
            (Method::Get, p) if p.ends_with("/ws") => self.open(req).await,
            _ => text(404, "not found"),
        }
    }

    async fn websocket_message(
        &self,
        ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        let Some(conn) = self.conn_of(&ws) else {
            let _ = ws.close(Some(1011), Some("unknown socket"));
            return Ok(());
        };
        // The protocol is text only; anything else gets bad_message.
        let text = match message {
            WebSocketIncomingMessage::String(text) => text,
            WebSocketIncomingMessage::Binary(_) => String::new(),
        };
        if text.len() > MAX_BODY {
            let _ = ws.close(Some(1009), Some("message too big"));
            return Ok(());
        }
        self.load().await?;
        let now = now_ms();
        let changed = {
            let mut cache = self.cache.borrow_mut();
            let Some(stored) = cache.stored.as_mut() else {
                drop(cache);
                let _ = ws.close(Some(1000), Some("closed"));
                return Ok(());
            };
            let before = stored.connected();
            let outs = stored.room.handle_text(conn, &text, now);
            if before > 0 && stored.connected() == 0 {
                stored.add_deadline(now + EMPTY_MS);
            }
            // Only a message that others see changes what storage must hold.
            // A ping or a refused message changes the rate counters alone,
            // and losing those to an eviction is harmless.
            let changed = outs.iter().any(|out| !matches!(out.to, To::One(_)));
            self.send(&stored.room, outs);
            changed
        };
        if changed {
            self.save().await?;
        }
        Ok(())
    }

    async fn websocket_close(
        &self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        self.closed(ws).await
    }

    async fn websocket_error(&self, ws: WebSocket, _error: worker::Error) -> Result<()> {
        self.closed(ws).await
    }

    async fn alarm(&self) -> Result<Response> {
        self.load().await?;
        let now = now_ms();
        {
            let mut cache = self.cache.borrow_mut();
            let Some(stored) = cache.stored.as_mut() else {
                return Response::ok("");
            };
            let outs = stored.room.tick(now);
            stored.deadlines.retain(|at| *at > now);
            // The alarm just fired, so none is set now.
            stored.alarm = None;
            self.send(&stored.room, outs);
        }
        self.save().await?;
        Response::ok("")
    }
}

impl RoomObject {
    /// Reads the room from storage once after the object wakes.
    async fn load(&self) -> Result<()> {
        if self.cache.borrow().loaded {
            return Ok(());
        }
        let raw: Option<String> = self.state.storage().get(KEY).await?;
        let stored = match raw {
            Some(raw) => match serde_json::from_str::<Stored>(&raw) {
                Ok(stored) => Some(stored),
                Err(err) => {
                    worker::console_error!("a stored room did not read back: {err}");
                    None
                }
            },
            None => None,
        };
        let mut cache = self.cache.borrow_mut();
        if !cache.loaded {
            cache.loaded = true;
            cache.stored = stored;
        }
        Ok(())
    }

    /// Writes the room to storage and sets the alarm for its next deadline. A
    /// closed room is deleted instead, alarm included.
    async fn save(&self) -> Result<()> {
        let storage = self.state.storage();
        let write = {
            let mut cache = self.cache.borrow_mut();
            let Some(stored) = cache.stored.as_mut() else {
                return Ok(());
            };
            if stored.room.is_closed() {
                cache.stored = None;
                None
            } else {
                let next = stored.deadlines.first().copied();
                let alarm = (stored.alarm != next).then_some(next);
                stored.alarm = next;
                Some((serde_json::to_string(stored)?, alarm))
            }
        };
        let Some((raw, alarm)) = write else {
            for ws in self.state.get_websockets() {
                let _ = ws.close(Some(1000), Some("closed"));
            }
            storage.delete_alarm().await?;
            storage.delete_all().await?;
            return Ok(());
        };
        storage.put(KEY, raw).await?;
        match alarm {
            Some(Some(at)) => {
                let date = js_sys::Date::new(&JsValue::from_f64(at as f64));
                storage.set_alarm(ScheduledTime::new(date)).await?;
            }
            Some(None) => storage.delete_alarm().await?,
            None => {}
        }
        Ok(())
    }

    async fn init(&self, mut req: Request) -> Result<Response> {
        self.load().await?;
        if self.cache.borrow().stored.is_some() {
            return text(409, "taken");
        }
        let Init { code, episode } = req.json().await?;
        let now = now_ms();
        let host_key = code::key(random());
        let room = Room::new(code.clone(), host_key.clone(), episode, now, random());
        // The host role passes on if the host never comes, and an empty room closes.
        let mut stored = Stored {
            room,
            deadlines: Vec::new(),
            alarm: None,
        };
        stored.add_deadline(now + GRACE_MS);
        stored.add_deadline(now + EMPTY_MS);
        self.cache.borrow_mut().stored = Some(stored);
        self.save().await?;
        Ok(Response::from_json(&CreatedRoom { code, host_key })?.with_status(201))
    }

    async fn view(&self) -> Result<Response> {
        self.load().await?;
        let view = self
            .cache
            .borrow()
            .stored
            .as_ref()
            .map(|stored| stored.room.view());
        match view {
            Some(view) => Response::from_json(&view),
            None => text(404, "no such room"),
        }
    }

    async fn open(&self, req: Request) -> Result<Response> {
        self.load().await?;
        if self.cache.borrow().stored.is_none() {
            return text(404, "no such room");
        }
        let upgrade = req.headers().get("Upgrade").ok().flatten();
        if !upgrade.is_some_and(|u| u.eq_ignore_ascii_case("websocket")) {
            return text(426, "expected a websocket upgrade");
        }
        let taken: Vec<u64> = self
            .state
            .get_websockets()
            .iter()
            .filter_map(|ws| self.conn_of(ws))
            .map(|c| c.0)
            .collect();
        // A random id needs nothing written to stay unique across evictions.
        let conn = loop {
            let id = u64::from_le_bytes(random());
            if !taken.contains(&id) {
                break id;
            }
        };
        let pair = WebSocketPair::new()?;
        let tag = tag(ConnId(conn));
        self.state
            .accept_websocket_with_tags(&pair.server, &[tag.as_str()]);
        Response::from_websocket(pair.client)
    }

    /// A socket closed or failed: its member keeps their place for the grace.
    async fn closed(&self, ws: WebSocket) -> Result<()> {
        // Answers the client's close frame; an error means it is already done.
        let _ = ws.close(Some(1000), Some(""));
        let Some(conn) = self.conn_of(&ws) else {
            return Ok(());
        };
        self.load().await?;
        let now = now_ms();
        let changed = {
            let mut cache = self.cache.borrow_mut();
            let Some(stored) = cache.stored.as_mut() else {
                return Ok(());
            };
            if !stored.room.targets(&To::All).contains(&conn) {
                // Never said hello, said bye, or was replaced: nothing to tell.
                false
            } else {
                let outs = stored.room.disconnect(conn, now);
                stored.add_deadline(now + GRACE_MS);
                if stored.connected() == 0 {
                    stored.add_deadline(now + EMPTY_MS);
                }
                self.send(&stored.room, outs);
                true
            }
        };
        if changed {
            self.save().await?;
        }
        Ok(())
    }

    fn conn_of(&self, ws: &WebSocket) -> Option<ConnId> {
        self.state
            .get_tags(ws)
            .iter()
            .find_map(|t| t.strip_prefix('c').and_then(|n| n.parse().ok()))
            .map(ConnId)
    }

    /// Sends each message to its sockets, and closes a socket after a message
    /// that ends its connection.
    fn send(&self, room: &Room, outs: Vec<Out>) {
        if outs.is_empty() {
            return;
        }
        let sockets: HashMap<ConnId, WebSocket> = self
            .state
            .get_websockets()
            .into_iter()
            .filter_map(|ws| self.conn_of(&ws).map(|c| (c, ws)))
            .collect();
        for out in outs {
            let text = out.msg.to_json();
            let ends = out.msg.ends_connection();
            for conn in room.targets(&out.to) {
                let Some(ws) = sockets.get(&conn) else {
                    continue;
                };
                let _ = ws.send_with_str(&text);
                if ends {
                    let _ = ws.close(Some(1000), Some(""));
                }
            }
        }
    }
}

fn tag(conn: ConnId) -> String {
    format!("c{}", conn.0)
}
