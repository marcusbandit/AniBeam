//! The room state machine. Pure: no clock, no sockets, no randomness of its own.
//!
//! A relay keeps one [`Room`] per code and drives it with three calls:
//! [`Room::handle`] (or [`Room::handle_text`]) for every frame a socket sends,
//! [`Room::disconnect`] when a socket closes, and [`Room::tick`] about once a
//! second for the timers. Each returns the messages to send as [`Out`]s; the
//! relay resolves an [`Out::to`] with [`Room::targets`] right away, and closes a
//! connection after sending it a message whose [`ServerMsg::ends_connection`] is
//! true. The room serializes with serde, so the Worker can store it between
//! messages.

use serde::{Deserialize, Serialize};

use crate::code::hex;
use crate::limit::{self, Verdict};
use crate::proto::{
    Action, ClientMsg, EpisodeRef, ErrorCode, MAX_KEY, MAX_MEMBERS, MAX_POSITION, MAX_SEALED,
    Member, MemberId, PROTO, PlayState, Progress, Ready, RoomView, ServerMsg,
};
use crate::siphash::siphash24;

/// How long a dropped member keeps their place (and the host their role), in ms.
pub const GRACE_MS: u64 = 120_000;

/// How long a room with nobody in it lives, in ms.
pub const EMPTY_MS: u64 = 600_000;

pub use crate::limit::RATE_PER_SEC;

/// The longest name, in characters.
pub const NAME_MAX: usize = 32;

/// The reason in [`ServerMsg::Closed`] after a `bye`.
pub const CLOSED_BYE: &str = "bye";

/// The reason in [`ServerMsg::Closed`] when the same member connected again.
pub const CLOSED_REPLACED: &str = "replaced";

/// The transport's handle for one connection (a socket). The protocol never
/// shows it; members are named by [`MemberId`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConnId(pub u64);

/// Who an [`Out`] goes to. `All` and `AllBut` mean the members connected now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum To {
    One(ConnId),
    All,
    AllBut(ConnId),
}

/// One message to send.
#[derive(Clone, Debug, PartialEq)]
pub struct Out {
    pub to: To,
    pub msg: ServerMsg,
}

/// One member's place in the room. Kept through a dropped socket for
/// [`GRACE_MS`] so a rejoin gets the same id back.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Slot {
    id: MemberId,
    name: String,
    ready: Ready,
    joined_ms: u64,
    conn: Option<ConnId>,
    left_ms: Option<u64>,
    rejoin_key: String,
    window_ms: u64,
    window_count: u32,
    /// The public key the member's hello carried. A room stored before keys
    /// reads with none.
    #[serde(default)]
    key: Option<String>,
    /// How far the member's download of the episode is, while it runs.
    #[serde(default)]
    progress: Option<Progress>,
}

/// A watch together room.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Room {
    code: String,
    host_key: String,
    /// Keys the rejoin keys; never leaves the relay.
    secret: [u8; 16],
    keys_made: u64,
    episode: EpisodeRef,
    state: PlayState,
    max: u32,
    /// Members in the order they first joined.
    slots: Vec<Slot>,
    next_id: u32,
    host: Option<MemberId>,
    /// When the host role passes to the longest present member, while nobody
    /// connected holds it.
    handover_ms: Option<u64>,
    /// Since when nobody is connected.
    empty_ms: Option<u64>,
    closed: bool,
}

impl Room {
    /// A new room, paused at 0 on `episode`. Whoever says hello with `host_key`
    /// becomes the host. `secret` is 16 random bytes the room makes rejoin keys
    /// from; the caller supplies them because this crate has no randomness.
    pub fn new(
        code: impl Into<String>,
        host_key: impl Into<String>,
        episode: EpisodeRef,
        now_ms: u64,
        secret: [u8; 16],
    ) -> Room {
        Room {
            code: code.into(),
            host_key: host_key.into(),
            secret,
            keys_made: 0,
            episode,
            state: PlayState {
                playing: false,
                position: 0.0,
                rate: 1.0,
                at_ms: now_ms,
                rev: 0,
            },
            max: MAX_MEMBERS,
            slots: Vec::new(),
            next_id: 1,
            host: None,
            handover_ms: Some(now_ms + GRACE_MS),
            empty_ms: Some(now_ms),
            closed: false,
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn episode(&self) -> &EpisodeRef {
        &self.episode
    }

    pub fn state(&self) -> PlayState {
        self.state
    }

    /// The host, while they are connected.
    pub fn host(&self) -> Option<MemberId> {
        self.host.filter(|h| self.is_connected(*h))
    }

    /// Whether `conn` has said hello and is a member now.
    pub fn is_member(&self, conn: ConnId) -> bool {
        self.slot_of(conn).is_some()
    }

    /// The members connected now, in the order they first joined.
    pub fn members(&self) -> Vec<Member> {
        self.slots
            .iter()
            .filter(|s| s.conn.is_some())
            .map(|s| Member {
                id: s.id,
                name: s.name.clone(),
                host: self.host == Some(s.id),
                ready: s.ready,
                joined_ms: s.joined_ms,
                key: s.key.clone(),
                progress: s.progress,
            })
            .collect()
    }

    /// The whole room, as a joiner sees it.
    pub fn view(&self) -> RoomView {
        RoomView {
            code: self.code.clone(),
            host: self.host(),
            members: self.members(),
            episode: self.episode.clone(),
            state: self.state,
            max: self.max,
        }
    }

    /// True once the room has been empty for [`EMPTY_MS`]; the relay drops it.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// The connections an [`Out`] goes to, as the room stands now. `One` is
    /// always its connection, member or not.
    pub fn targets(&self, to: &To) -> Vec<ConnId> {
        match *to {
            To::One(conn) => vec![conn],
            To::All => self.slots.iter().filter_map(|s| s.conn).collect(),
            To::AllBut(but) => self
                .slots
                .iter()
                .filter_map(|s| s.conn)
                .filter(|c| *c != but)
                .collect(),
        }
    }

    /// A text frame from `conn`. Text that is not a client message gets
    /// `bad_message` and the connection stays open.
    pub fn handle_text(&mut self, conn: ConnId, text: &str, now_ms: u64) -> Vec<Out> {
        match ClientMsg::parse(text) {
            Ok(msg) => self.handle(conn, msg, now_ms),
            Err(err) => {
                if let Some(i) = self.slot_of(conn)
                    && let Some(outs) = self.over_limit(i, conn, now_ms)
                {
                    return outs;
                }
                vec![error(conn, ErrorCode::BadMessage, err.to_string())]
            }
        }
    }

    /// A message from `conn`.
    pub fn handle(&mut self, conn: ConnId, msg: ClientMsg, now_ms: u64) -> Vec<Out> {
        if self.closed {
            return vec![error(conn, ErrorCode::NotFound, "the room has closed")];
        }
        let Some(i) = self.slot_of(conn) else {
            return match msg {
                ClientMsg::Hello {
                    name,
                    proto,
                    client: _,
                    host_key,
                    rejoin_key,
                    key,
                } => self.hello(conn, name, proto, host_key, rejoin_key, key, now_ms),
                _ => vec![error(conn, ErrorCode::Proto, "hello must come first")],
            };
        };
        // Leaving is never dropped: a lost bye would hold the seat, and the
        // host role, for the whole grace.
        if !matches!(msg, ClientMsg::Bye)
            && let Some(outs) = self.over_limit(i, conn, now_ms)
        {
            return outs;
        }
        let id = self.slots[i].id;
        match msg {
            ClientMsg::Hello { .. } => {
                vec![error(conn, ErrorCode::BadMessage, "hello was already sent")]
            }
            ClientMsg::Ping { c } => vec![one(conn, ServerMsg::Pong { c, s: now_ms })],
            ClientMsg::Cmd { action, position } => {
                if !position.is_finite() || !(0.0..=MAX_POSITION).contains(&position) {
                    return vec![error(
                        conn,
                        ErrorCode::BadMessage,
                        "the position must be from 0 to 86400 seconds",
                    )];
                }
                match action {
                    Action::Play => self.state.playing = true,
                    Action::Pause => self.state.playing = false,
                    Action::Seek => {}
                }
                self.state.position = position;
                self.state.at_ms = now_ms;
                self.state.rev += 1;
                vec![all(ServerMsg::State {
                    state: self.state,
                    by: id,
                })]
            }
            ClientMsg::Episode { episode } => {
                if self.host != Some(id) {
                    return vec![error(
                        conn,
                        ErrorCode::NotAllowed,
                        "only the host changes the episode",
                    )];
                }
                self.episode = episode;
                self.state = PlayState {
                    playing: false,
                    position: 0.0,
                    rate: 1.0,
                    at_ms: now_ms,
                    rev: self.state.rev + 1,
                };
                for slot in &mut self.slots {
                    slot.ready = Ready::Loading;
                    slot.progress = None;
                }
                vec![
                    all(ServerMsg::Episode {
                        episode: self.episode.clone(),
                        state: self.state,
                        by: id,
                    }),
                    all(self.members_msg()),
                ]
            }
            ClientMsg::Status { ready, progress } => {
                let slot = &mut self.slots[i];
                slot.ready = ready;
                // Only someone without the episode is fetching it
                slot.progress = progress
                    .filter(|_| ready != Ready::Ready)
                    .and_then(Progress::clean);
                vec![all(self.members_msg())]
            }
            ClientMsg::Bye => self.bye(i, conn, now_ms),
            ClientMsg::Seal { to, data } => self.seal(id, conn, to, data),
        }
    }

    /// `conn` closed. Its member keeps their place for [`GRACE_MS`]; everyone
    /// else gets the new member list.
    pub fn disconnect(&mut self, conn: ConnId, now_ms: u64) -> Vec<Out> {
        let Some(i) = self.slot_of(conn) else {
            return Vec::new();
        };
        let slot = &mut self.slots[i];
        slot.conn = None;
        slot.left_ms = Some(now_ms);
        if self.host == Some(slot.id) {
            self.handover_ms = Some(now_ms + GRACE_MS);
        }
        self.note_if_empty(now_ms);
        vec![all(self.members_msg())]
    }

    /// The timers: members gone for [`GRACE_MS`] are dropped, the host role
    /// moves on when its holder has been gone that long, and a room empty for
    /// [`EMPTY_MS`] closes.
    pub fn tick(&mut self, now_ms: u64) -> Vec<Out> {
        if self.closed {
            return Vec::new();
        }
        self.prune(now_ms);
        let mut outs = Vec::new();
        if self.hand_over_if_due(now_ms) {
            outs.push(all(self.members_msg()));
        }
        if self.connected() == 0 && self.empty_ms.is_some_and(|t| now_ms >= t + EMPTY_MS) {
            self.closed = true;
        }
        outs
    }

    fn hello(
        &mut self,
        conn: ConnId,
        name: String,
        proto: u32,
        host_key: Option<String>,
        rejoin_key: Option<String>,
        key: Option<String>,
        now_ms: u64,
    ) -> Vec<Out> {
        if proto != PROTO {
            return vec![error(
                conn,
                ErrorCode::Proto,
                format!("this relay speaks protocol {PROTO}"),
            )];
        }
        let Some(name) = clean_name(&name) else {
            return vec![error(
                conn,
                ErrorCode::BadName,
                format!("a name is 1 to {NAME_MAX} characters"),
            )];
        };
        self.prune(now_ms);
        let mut outs = Vec::new();
        let rejoined = rejoin_key
            .as_deref()
            .and_then(|key| self.slots.iter().position(|s| same(&s.rejoin_key, key)));
        let i = match rejoined {
            Some(i) => {
                let slot = &mut self.slots[i];
                if let Some(old) = slot.conn.replace(conn)
                    && old != conn
                {
                    outs.push(one(
                        old,
                        ServerMsg::Closed {
                            reason: CLOSED_REPLACED.into(),
                        },
                    ));
                }
                slot.left_ms = None;
                slot.name = name;
                slot.key = clean_key(key);
                slot.window_ms = now_ms;
                slot.window_count = 0;
                i
            }
            None => {
                // Only the people connected count toward `max`, as the view
                // shows only them. A member in their grace comes back with the
                // rejoin key above even when that takes the room past `max`:
                // their place was kept. Twice `max` places in all bounds the
                // room however its members come and go.
                if self.connected() >= self.max as usize
                    || self.slots.len() >= 2 * self.max as usize
                {
                    return vec![error(conn, ErrorCode::Full, "the room is full")];
                }
                let id = MemberId(self.next_id);
                self.next_id += 1;
                let rejoin_key = self.make_key();
                self.slots.push(Slot {
                    id,
                    name,
                    ready: Ready::Loading,
                    joined_ms: now_ms,
                    conn: Some(conn),
                    left_ms: None,
                    rejoin_key,
                    window_ms: now_ms,
                    window_count: 0,
                    key: clean_key(key),
                    progress: None,
                });
                self.slots.len() - 1
            }
        };
        let id = self.slots[i].id;
        if host_key.is_some_and(|k| same(&self.host_key, &k)) {
            self.host = Some(id);
        }
        if self.host == Some(id) {
            self.handover_ms = None;
        }
        self.empty_ms = None;
        self.hand_over_if_due(now_ms);
        outs.push(one(
            conn,
            ServerMsg::Welcome {
                you: id,
                rejoin_key: self.slots[i].rejoin_key.clone(),
                room: self.view(),
                server_ms: now_ms,
            },
        ));
        outs.push(Out {
            to: To::AllBut(conn),
            msg: self.members_msg(),
        });
        outs
    }

    /// Passes a sealed payload from member `from` to member `to`, unread. Too
    /// large, or for nobody connected but the sender, it goes nowhere and the
    /// sender hears why; neither closes the connection.
    fn seal(&mut self, from: MemberId, conn: ConnId, to: MemberId, data: String) -> Vec<Out> {
        if data.len() > MAX_SEALED {
            return vec![error(
                conn,
                ErrorCode::BadMessage,
                format!("a sealed payload is at most {MAX_SEALED} characters"),
            )];
        }
        let target = self
            .slots
            .iter()
            .find(|s| s.id == to && s.id != from)
            .and_then(|s| s.conn);
        match target {
            Some(target) => vec![one(target, ServerMsg::Sealed { from, data })],
            None => vec![error(
                conn,
                ErrorCode::NotAllowed,
                "nobody else in the room has that id",
            )],
        }
    }

    fn bye(&mut self, i: usize, conn: ConnId, now_ms: u64) -> Vec<Out> {
        let slot = self.slots.remove(i);
        if self.host == Some(slot.id) {
            self.host = None;
            self.handover_ms = Some(now_ms);
            self.hand_over_if_due(now_ms);
        }
        self.note_if_empty(now_ms);
        vec![
            one(
                conn,
                ServerMsg::Closed {
                    reason: CLOSED_BYE.into(),
                },
            ),
            all(self.members_msg()),
        ]
    }

    /// Gives the host role to the longest present connected member when nobody
    /// connected holds it and the handover is due. True when it moved.
    fn hand_over_if_due(&mut self, now_ms: u64) -> bool {
        if self.host.is_some_and(|h| self.is_connected(h)) {
            return false;
        }
        if self.handover_ms.is_some_and(|t| now_ms < t) {
            return false;
        }
        let Some(next) = self
            .slots
            .iter()
            .filter(|s| s.conn.is_some())
            .min_by_key(|s| (s.joined_ms, s.id))
            .map(|s| s.id)
        else {
            return false;
        };
        self.host = Some(next);
        self.handover_ms = None;
        true
    }

    /// Counts a message against the member's rate. `None` lets it through;
    /// otherwise the message is dropped and these go out instead (one
    /// `rate_limited` per second at most).
    fn over_limit(&mut self, i: usize, conn: ConnId, now_ms: u64) -> Option<Vec<Out>> {
        let slot = &mut self.slots[i];
        match limit::count(&mut slot.window_ms, &mut slot.window_count, now_ms) {
            Verdict::Pass => None,
            Verdict::Refuse => Some(vec![one(conn, limit::refusal())]),
            Verdict::Drop => Some(Vec::new()),
        }
    }

    fn make_key(&mut self) -> String {
        let n = self.keys_made;
        self.keys_made += 1;
        let mut out = [0u8; 16];
        for half in 0..2u8 {
            let mut msg = [0u8; 9];
            msg[..8].copy_from_slice(&n.to_le_bytes());
            msg[8] = half;
            let h = siphash24(self.secret, &msg).to_le_bytes();
            out[usize::from(half) * 8..usize::from(half) * 8 + 8].copy_from_slice(&h);
        }
        hex(&out)
    }

    /// Drops members gone for [`GRACE_MS`], and the host role with them.
    fn prune(&mut self, now_ms: u64) {
        self.slots
            .retain(|s| s.left_ms.is_none_or(|t| now_ms < t + GRACE_MS));
        if let Some(h) = self.host
            && !self.slots.iter().any(|s| s.id == h)
        {
            self.host = None;
        }
    }

    fn note_if_empty(&mut self, now_ms: u64) {
        if self.connected() == 0 {
            self.empty_ms = Some(now_ms);
        }
    }

    fn members_msg(&self) -> ServerMsg {
        ServerMsg::Members {
            members: self.members(),
        }
    }

    fn slot_of(&self, conn: ConnId) -> Option<usize> {
        self.slots.iter().position(|s| s.conn == Some(conn))
    }

    fn is_connected(&self, id: MemberId) -> bool {
        self.slots.iter().any(|s| s.id == id && s.conn.is_some())
    }

    fn connected(&self) -> usize {
        self.slots.iter().filter(|s| s.conn.is_some()).count()
    }
}

/// A name trimmed, or `None` when it is empty, too long or has a control
/// character in it. The core checks a name with this before it says hello.
pub fn clean_name(name: &str) -> Option<String> {
    let name = name.trim();
    let len = name.chars().count();
    if len == 0 || len > NAME_MAX || name.chars().any(char::is_control) {
        return None;
    }
    Some(name.to_owned())
}

/// A public key as the room keeps it: base64 characters, at most [`MAX_KEY`].
/// Anything else is no key, so a bad one cannot reach the other members.
fn clean_key(key: Option<String>) -> Option<String> {
    key.filter(|k| {
        !k.is_empty()
            && k.len() <= MAX_KEY
            && k.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'=' | b'-' | b'_'))
    })
}

/// Compares two keys without stopping at the first difference.
fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn one(conn: ConnId, msg: ServerMsg) -> Out {
    Out {
        to: To::One(conn),
        msg,
    }
}

fn all(msg: ServerMsg) -> Out {
    Out { to: To::All, msg }
}

fn error(conn: ConnId, code: ErrorCode, message: impl Into<String>) -> Out {
    one(
        conn,
        ServerMsg::Error {
            code,
            message: message.into(),
        },
    )
}

#[cfg(test)]
mod tests;
