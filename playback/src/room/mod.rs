//! Watch together rooms (#846): the core's side of a room. Everyone in a room
//! plays their own copy of the episode; the relay passes the play state between
//! them, and this module turns it into what each player does.
//!
//! One room at a time. `RoomCreate` makes one on the relay for the episode a
//! playback session plays and joins it as the host; `RoomJoin` joins one by its
//! code, and `RoomEpisode` then says which file to open (`episode::find_local`).
//! `RoomAttach` names the session the room drives. From then on every tick of that
//! session goes through `tick`, and every play state another member sets comes
//! back as `RoomApply`: pause or play, a seek, a speed (the rules are in `sync`).
//!
//! The relay's order is the room's. A member's own `RoomCommand` is taken as done
//! when it is sent, so the player moves at once, and its echo applies nothing when
//! nothing came between. When someone else's state came in between, the relay put
//! it first, and the echo of this member's last command is applied, since it holds
//! what the room ended up as. Seeks go out at most every `SEEK_GAP`, the last one
//! always, so a held arrow key stays inside the relay's rate.
//!
//! Positions on the wire are on the host's file. Each member converts with their
//! offset, kept per file under `settings::ROOM_OFFSETS`, and suggested from the two
//! openings when nothing is stored.
//!
//! `socket` owns the connection: one task per room on the core's runtime, which
//! reconnects with the rejoin key and gives the room up after two minutes. Its
//! callbacks below take the room's lock, change it, and emit only after letting go,
//! so a listener may call straight back into the core.

pub mod episode;
pub mod media;
pub mod seal;
mod socket;
pub mod sync;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anibeam_room::proto::{
    self, Action, ClientMsg, Member, MemberId, PlayState, Ready, RoomView, ServerMsg,
};
use rusqlite::Connection;
use tokio::sync::{Notify, mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use self::sync::{Clock, Drift, LocalTick};
use crate::presence::JoinRooms;
use crate::presence::join::{not_in_room, room_error as error};
use crate::{Follower, Playback};
use anibeam_contract::*;
use anibeam_downloads::Downloads;
use anibeam_foundation::prefs;
use anibeam_foundation::services::Services;
use anibeam_foundation::store::settings;
use anibeam_library::Library;
use anibeam_metadata::Metadata;

/// The largest offset a member may set, in seconds either way.
pub const MAX_OFFSET: f64 = 600.0;

/// How long `RoomCreate` and `RoomJoin` wait for the relay before giving up.
const JOIN_WAIT: Duration = Duration::from_secs(20);

/// The least time between two corrections of one player.
const CORRECTION_GAP: Duration = Duration::from_secs(1);

/// The least time between two seeks sent to the relay. Seeks in between wait,
/// and the last of them goes when the gap is up: a held arrow key repeats 25 to
/// 33 times a second, past the relay's 20 messages.
pub const SEEK_GAP: Duration = Duration::from_millis(150);

/// How long an own command may go unanswered before it is taken as lost (the
/// relay drops a message over its rate) and the relay's last state wins again.
const LOST_AFTER: Duration = Duration::from_secs(5);

/// The watch together team (#854): the room this core is in, if any, over
/// the playback team whose session it drives. It sits above playback and
/// presence, and neither names it: the session rules tell it about a tick, a
/// close and an opening through `playback::Follower`, and Discord's join asks
/// it about the room through `presence::JoinRooms`, both linked in `Core::open`.
pub struct Rooms {
    pub(crate) services: Arc<Services>,
    pub(crate) playback: Arc<Playback>,
    /// What a room's Get media downloads through.
    pub(crate) downloads: Arc<Downloads>,
    /// The library Get media brings a file into, and the matching that
    /// points it at the room's show (#889).
    pub(crate) library: Arc<Library>,
    pub(crate) metadata: Arc<Metadata>,
    active: Mutex<Option<Active>>,
    /// Names each room this core joins, so a task that outlived its room finds
    /// it gone rather than changing the next one.
    next: AtomicU64,
    /// `CORRECTION_GAP` in ms; a test shortens it.
    gap_ms: AtomicU64,
}

impl Rooms {
    pub fn new(
        services: Arc<Services>,
        playback: Arc<Playback>,
        downloads: Arc<Downloads>,
        library: Arc<Library>,
        metadata: Arc<Metadata>,
    ) -> Rooms {
        Rooms {
            services,
            playback,
            downloads,
            library,
            metadata,
            active: Mutex::new(None),
            next: AtomicU64::new(0),
            gap_ms: AtomicU64::new(CORRECTION_GAP.as_millis() as u64),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Option<Active>> {
        anibeam_foundation::sync::recover(&self.active)
    }

    /// The least time between two corrections, in place of a second. Not part of
    /// the contract.
    pub fn set_correction_gap(&self, gap: Duration) {
        self.gap_ms.store(gap.as_millis() as u64, Ordering::Relaxed);
    }

    /// Drops the connection without a goodbye, as a network would, so the task
    /// connects again. Not part of the contract.
    pub fn drop_connection(&self) {
        if let Some(a) = self.lock().as_ref() {
            a.kick.notify_one();
        }
    }
}

/// The session rules' side of a room: what happened to a session, where
/// they used to call this module directly.
impl Follower for Rooms {
    fn tick(&self, session: u64, position: f64, paused: bool) {
        tick(self, session, position, paused);
    }

    fn closed(&self, session: u64) {
        closed(self, session);
    }

    fn opening_known(&self, session: u64, start: f64) {
        opening_known(self, session, start);
    }
}

/// Discord's join: the room's status, and a room opened for a friend let in.
impl JoinRooms for Rooms {
    fn status(&self) -> Option<RoomInfo> {
        self.lock().as_ref().and_then(Active::info)
    }

    fn create(self: Arc<Self>, session: Option<u64>) -> Result<RoomInfo, CoreError> {
        create(&self, session)
    }
}

/// One room, as this member is in it.
struct Active {
    serial: u64,
    /// The relay's address, no slash at the end.
    relay: String,
    code: String,
    /// Who this member is, from the first welcome on.
    you: Option<MemberId>,
    members: Vec<Member>,
    /// None until the first welcome for a joiner.
    episode: Option<proto::EpisodeRef>,
    /// The state the player is held to: the relay's, or this member's own
    /// command while the relay has yet to answer it.
    state: PlayState,
    /// The last state the relay sent, in its order.
    relayed: PlayState,
    /// Own commands sent that the relay has not echoed yet, and when the last went.
    in_flight: u32,
    sent_at: Option<Instant>,
    /// Someone else's state came while own commands were out: the relay put it
    /// first, so the echo of the last own command is applied, not trusted.
    stale: bool,
    /// A seek held back by `SEEK_GAP`, in canonical seconds, and when the last
    /// seek went. `flush_armed` while a task waits to send the held one.
    held_seek: Option<f64>,
    seek_sent: Option<Instant>,
    flush_armed: bool,
    connected: bool,
    clock: Clock,
    /// The playback session the room drives, its file and its episode.
    session: Option<u64>,
    file: Option<u64>,
    mine: Option<proto::EpisodeRef>,
    /// The file whose timeline is the room's because this member's player set it
    /// (made the room, or moved it as the host), and the offset it plays at. It
    /// plays at nought unless set by hand, and that offset is kept in memory
    /// only: stored offsets are measured against another member's file.
    timeline: Option<(u64, f64)>,
    offset: f64,
    /// Whether the offset was stored or set by hand, so a suggestion leaves it.
    offset_fixed: bool,
    suggested: Option<f64>,
    ready: Ready,
    drift: Drift,
    /// This member's key pair for the room: others seal payloads to its public
    /// half, which the hello carries (#889).
    keys: Arc<seal::Keys>,
    /// Where Get media is, the download's progress the status carries, and the
    /// fetch running now: its number, and the token that stops it. `whole_show`
    /// when the member asked for the rest of the show with it (#922), and `rest`
    /// once the search for that rest has started for the room's episode: it runs
    /// once an episode, and stops with the room or when the host moves on.
    fetch: RoomFetch,
    progress: Option<proto::Progress>,
    fetching: Option<(u64, CancellationToken)>,
    whole_show: bool,
    rest: Option<CancellationToken>,
    /// The host's ready gate (#922), and the position on the room's timeline the
    /// host pressed play at. `started` once the episode has played: the gate asks
    /// only before that, so a pause later never brings it back.
    gate: RoomGate,
    gate_at: f64,
    started: bool,
    /// Why the last Get media stopped short, until the next one starts.
    fetch_error: Option<Words>,
    tx: mpsc::UnboundedSender<ClientMsg>,
    cancel: CancellationToken,
    kick: Arc<Notify>,
    /// For the task that sends a held seek.
    rooms: Weak<Rooms>,
}

impl Active {
    fn info(&self) -> Option<RoomInfo> {
        // A joiner the relay has yet to welcome is not in the room yet
        if self.you.is_none() && self.episode.is_none() {
            return None;
        }
        Some(RoomInfo {
            code: self.code.clone(),
            link: format!("{}/{}", self.relay, self.code),
            you: self.you.map_or(0, |m| m.0),
            host: self.is_host(),
            members: self.members.iter().map(member).collect(),
            episode: self.episode.as_ref().map(episode::to_contract),
            offset: self.offset,
            suggested_offset: self.suggested,
            session: self.session,
            connected: self.connected,
            fetch: self.fetch,
            can_fetch: self.can_fetch(),
            gate: self.gate,
            fetch_error: self.fetch_error.clone(),
        })
    }

    /// Whether everyone in the room has the episode open: this member by its own
    /// state, which the relay's list may not carry yet.
    fn all_ready(&self) -> bool {
        self.members.iter().all(|m| {
            if Some(m.id) == self.you {
                self.ready == Ready::Ready
            } else {
                m.ready == Ready::Ready
            }
        })
    }

    /// A new episode: it has not played, and nothing waits to start it.
    fn fresh_episode(&mut self) {
        self.gate = RoomGate::Off;
        self.started = false;
        if let Some(rest) = self.rest.take() {
            rest.cancel();
        }
    }

    /// The host starts the room at the position play was pressed at, for
    /// everyone, ready or not (#922).
    fn open_gate(&mut self) -> Vec<Say> {
        self.gate = RoomGate::Off;
        self.started = true;
        let now = Instant::now();
        let at = self.gate_at;
        self.state.playing = true;
        self.state.position = at;
        self.state.at_ms = self.clock.server_now(local_ms());
        self.held_seek = None;
        self.send_cmd(Action::Play, at, now);
        let mut out = Vec::new();
        if let Some(session) = self.session {
            out.push(self.bring(session));
        }
        out.push(Say::debug(
            format!("Room {}: the host starts it at {at:.2}", self.code),
            self.changed(),
        ));
        out
    }

    /// The status this member's relay sees: ready or not, and the download's
    /// progress while Get media runs.
    fn status(&self) -> ClientMsg {
        ClientMsg::Status {
            ready: self.ready,
            progress: self.progress.filter(|_| self.ready != Ready::Ready),
        }
    }

    /// The host, when it is someone else and its app can be sealed to.
    fn host_key(&self) -> Option<(MemberId, String)> {
        self.members
            .iter()
            .find(|m| m.host && Some(m.id) != self.you)
            .and_then(|m| Some((m.id, m.key.clone()?)))
    }

    /// Whether Get media can start: the episode is missing here, nothing is
    /// fetching it, and the host's app can be asked.
    fn can_fetch(&self) -> bool {
        self.connected
            && self.session.is_none()
            && self.ready == Ready::Missing
            && matches!(self.fetch, RoomFetch::Idle | RoomFetch::Failed)
            && self.host_key().is_some()
    }

    /// Stops following a Get media. The download itself stays with the
    /// torrent client.
    fn stop_fetch(&mut self) {
        if let Some((_, token)) = self.fetching.take() {
            token.cancel();
        }
        self.fetch = RoomFetch::Idle;
        self.progress = None;
        self.fetch_error = None;
    }

    /// Stops what the room runs here: Get media, and the connection's task. A goodbye
    /// queued before this still goes out.
    fn end(&mut self) {
        self.stop_fetch();
        self.cancel.cancel();
    }

    fn is_host(&self) -> bool {
        self.you
            .is_some_and(|you| self.members.iter().any(|m| m.id == you && m.host))
    }

    /// Where this member's player should be now, on their own file.
    fn target(&self) -> f64 {
        let server_now = self.clock.server_now(local_ms());
        (sync::canonical(&self.state, server_now) + self.offset).max(0.0)
    }

    fn send(&self, msg: ClientMsg) {
        // A closed channel is a task that has ended; the room is on its way out.
        let _ = self.tx.send(msg);
    }

    /// Sends one of this member's own commands.
    fn send_cmd(&mut self, action: Action, position: f64, now: Instant) {
        self.send(ClientMsg::Cmd { action, position });
        self.in_flight += 1;
        self.sent_at = Some(now);
        if matches!(action, Action::Seek) {
            self.seek_sent = Some(now);
        }
    }

    /// Whether the relay has yet to hear or answer something this member did.
    fn ahead(&self) -> bool {
        self.in_flight > 0 || self.held_seek.is_some()
    }

    /// Forgets own commands: the relay's state is the room's from here.
    fn settle(&mut self, relayed: PlayState) {
        self.relayed = relayed;
        self.state = relayed;
        self.in_flight = 0;
        self.stale = false;
        self.held_seek = None;
    }

    /// Whether the attached file is the one whose timeline is the room's.
    fn on_timeline(&self) -> bool {
        self.file
            .is_some_and(|f| self.timeline.is_some_and(|(t, _)| t == f))
    }

    /// Lets go of the session: the player stays open, the room no longer drives it.
    fn detach(&mut self) {
        self.session = None;
        self.file = None;
        self.mine = None;
        self.offset = 0.0;
        self.offset_fixed = false;
        self.suggested = None;
        self.drift = Drift::default();
        self.ready = Ready::Loading;
        self.stop_fetch();
    }

    /// Tells the player where the room is, now.
    fn bring(&mut self, session: u64) -> Say {
        self.drift.last = Some(Instant::now());
        self.drift.speed = 1.0;
        apply(session, self.state.playing, Some(self.target()), 1.0)
    }

    fn changed(&self) -> EventBody {
        EventBody::RoomChanged { room: self.info() }
    }
}

fn member(m: &Member) -> RoomMember {
    let progress = m.progress.filter(|_| m.ready != Ready::Ready);
    RoomMember {
        id: m.id.0,
        name: m.name.clone(),
        host: m.host,
        ready: match (m.ready, progress) {
            (Ready::Ready, _) => RoomReady::Ready,
            (_, Some(_)) => RoomReady::Downloading,
            (Ready::Missing, None) => RoomReady::Missing,
            (Ready::Loading, None) => RoomReady::Loading,
        },
        progress: progress.map(|p| p.percent),
        eta: progress.and_then(|p| p.eta),
    }
}

/// An event decided under the lock and emitted after it.
struct Say {
    level: Level,
    words: Option<Words>,
    message: String,
    body: EventBody,
}

impl Say {
    fn debug(message: impl Into<String>, body: EventBody) -> Say {
        Say {
            level: Level::Debug,
            words: None,
            message: message.into(),
            body,
        }
    }

    fn said(level: Level, words: Words, message: impl Into<String>, body: EventBody) -> Say {
        Say {
            level,
            words: Some(words),
            message: message.into(),
            body,
        }
    }
}

fn emit(services: &Services, out: Vec<Say>) {
    for s in out {
        services
            .bus
            .emit_said(s.level, Stage::Playback, s.words, s.message, None, s.body);
    }
}

/// What a player does to keep up. Debug: corrections never reach the activity log.
fn apply(session: u64, playing: bool, seek_to: Option<f64>, speed: f64) -> Say {
    let mut message = format!(
        "Room: session {session} {}",
        if playing { "plays" } else { "pauses" }
    );
    if let Some(to) = seek_to {
        message.push_str(&format!(", seeks to {to:.2}"));
    }
    if speed != 1.0 {
        message.push_str(&format!(", at {speed}x"));
    }
    Say::debug(
        message,
        EventBody::RoomApply {
            session,
            playing,
            seek_to,
            speed,
        },
    )
}

/// The room paused at the start of its timeline, as of `at_ms` on the relay's clock.
fn at_start(at_ms: u64, rev: u64) -> PlayState {
    PlayState {
        playing: false,
        position: 0.0,
        rate: 1.0,
        at_ms,
        rev,
    }
}

/// Milliseconds since the Unix epoch on this machine's clock.
pub(crate) fn local_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

// A name as the relay takes it, and whether a relay address can be used: the
// relay crate's rule, and the one the preferences check on a save (#854).
use anibeam_foundation::prefs::relay_ok;
use anibeam_room::room::clean_name;

/// The name and the relay from the preferences, or why they cannot be used.
fn who(services: &Services) -> Result<(String, String), CoreError> {
    let p = services.store.read(prefs::load_preferences)?;
    let name = clean_name(&p.room_name).ok_or_else(|| {
        CoreError::invalid(
            "room_name",
            anibeam_contract::i18n::tr("core-room-no-name", &[]),
        )
    })?;
    if !relay_ok(&p.room_relay) {
        return Err(CoreError::invalid(
            "room_relay",
            anibeam_contract::i18n::tr("core-prefs-room-relay", &[]),
        ));
    }
    Ok((name, p.room_relay.trim().trim_end_matches('/').to_string()))
}

fn offsets(conn: &Connection) -> Result<HashMap<String, f64>, CoreError> {
    Ok(settings::get(conn, settings::ROOM_OFFSETS)?.unwrap_or_default())
}

/// The offset kept for a file, if one was set.
fn stored_offset(conn: &Connection, file: u64) -> Result<Option<f64>, CoreError> {
    Ok(offsets(conn)?.get(&file.to_string()).copied())
}

fn save_offset(conn: &Connection, file: u64, offset: f64) -> Result<(), CoreError> {
    let mut all = offsets(conn)?;
    all.insert(file.to_string(), offset);
    settings::set(conn, settings::ROOM_OFFSETS, &all)
}

// ---------------------------------------------------------------------------
// The calls
// ---------------------------------------------------------------------------

pub fn call(rooms: &Arc<Rooms>, call: Call) -> Result<Reply, CoreError> {
    let room = match call {
        Call::RoomCreate { session } => Some(create(rooms, session)?),
        Call::RoomStart { wait } => start_room(rooms, wait)?,
        Call::RoomJoin { code } => Some(join(rooms, &code)?),
        Call::RoomAttach { session } => attach(rooms, session)?,
        Call::RoomCommand { action, position } => command(rooms, action, position)?,
        Call::RoomSetOffset { offset } => set_offset(rooms, offset)?,
        Call::RoomLeave => {
            leave(rooms);
            None
        }
        Call::RoomStatus => rooms.lock().as_ref().and_then(Active::info),
        Call::RoomGetMedia { whole_show } => media::get(rooms, whole_show)?,
        other => {
            return Err(CoreError::internal(format!(
                "{} is not a room call",
                anibeam_contract::call_name(&other)
            )));
        }
    };
    Ok(Reply::Room { room })
}

/// What `start` needs to begin a room.
struct Begin {
    client: reqwest::Client,
    relay: String,
    code: String,
    name: String,
    host_key: Option<String>,
    episode: Option<proto::EpisodeRef>,
    /// The host's session and its file.
    session: Option<(u64, u64)>,
}

fn create(rooms: &Arc<Rooms>, session: Option<u64>) -> Result<RoomInfo, CoreError> {
    let s = match session {
        Some(id) => Some(rooms.playback.sessions.get(id).ok_or(CoreError::NotFound {
            what: Entity::Session,
            id,
        })?),
        None => None,
    };
    let (name, relay) = who(&rooms.services)?;
    let client = anibeam_net::room_client(socket::CONNECT_WAIT)?;
    let episode = s.as_ref().map(episode::of_session);
    leave(rooms);
    let created = rooms
        .services
        .handle
        .block_on(async {
            tokio::time::timeout(JOIN_WAIT, socket::create(&client, &relay, episode.as_ref())).await
        })
        .map_err(|_| socket::Fail::Unreachable("timed out".into()).to_error())?
        .map_err(|f| f.to_error())?;
    let info = start(
        rooms,
        Begin {
            client,
            relay: relay.clone(),
            code: created.code,
            name,
            host_key: Some(created.host_key),
            episode,
            session: s.map(|s| (s.id, s.file)),
        },
    )?;
    let what = match &info.episode {
        Some(e) => format!("for {} {}", e.series_title, e.episode_key),
        None => "with nothing picked yet".into(),
    };
    say_room(
        rooms,
        Level::Info,
        anibeam_contract::words!("core-room-created", code = &info.code),
        format!(
            "Room {} created on {} {what}",
            info.code,
            anibeam_net::url_shown(&relay),
        ),
        Some(info.clone()),
    );
    Ok(info)
}

fn join(rooms: &Arc<Rooms>, code: &str) -> Result<RoomInfo, CoreError> {
    let code = anibeam_room::code::normalize(code).ok_or_else(|| {
        CoreError::invalid(
            "code",
            anibeam_contract::i18n::tr("core-room-bad-code", &[]),
        )
    })?;
    let (name, relay) = who(&rooms.services)?;
    let client = anibeam_net::room_client(socket::CONNECT_WAIT)?;
    leave(rooms);
    let info = start(
        rooms,
        Begin {
            client,
            relay: relay.clone(),
            code,
            name,
            host_key: None,
            episode: None,
            session: None,
        },
    )?;
    say_room(
        rooms,
        Level::Info,
        anibeam_contract::words!("core-room-joined", code = &info.code),
        format!(
            "Joined room {} on {}",
            info.code,
            anibeam_net::url_shown(&relay)
        ),
        Some(info.clone()),
    );
    Ok(info)
}

/// Says which room this member is in now, made or joined, or that it is in none.
fn say_room(rooms: &Rooms, level: Level, words: Words, message: String, room: Option<RoomInfo>) {
    rooms.services.bus.emit_said(
        level,
        Stage::Playback,
        Some(words),
        message,
        None,
        EventBody::RoomChanged { room },
    );
}

/// Starts the room's task and waits for its first welcome.
fn start(rooms: &Arc<Rooms>, b: Begin) -> Result<RoomInfo, CoreError> {
    let owner = rooms.clone();
    let serial = rooms.next.fetch_add(1, Ordering::SeqCst) + 1;
    let (tx, rx) = mpsc::unbounded_channel();
    let cancel = CancellationToken::new();
    let kick = Arc::new(Notify::new());
    let (first, answer) = oneshot::channel();
    let hosting = b.session.is_some();
    let keys = Arc::new(seal::Keys::new());
    let paused = at_start(0, 0);
    *rooms.lock() = Some(Active {
        serial,
        relay: b.relay.clone(),
        code: b.code.clone(),
        you: None,
        members: Vec::new(),
        mine: hosting.then(|| b.episode.clone()).flatten(),
        episode: b.episode,
        state: paused,
        relayed: paused,
        in_flight: 0,
        sent_at: None,
        stale: false,
        held_seek: None,
        seek_sent: None,
        flush_armed: false,
        connected: false,
        clock: Clock::default(),
        session: b.session.map(|(s, _)| s),
        file: b.session.map(|(_, f)| f),
        timeline: b.session.map(|(_, f)| (f, 0.0)),
        offset: 0.0,
        offset_fixed: hosting,
        suggested: None,
        ready: if hosting {
            Ready::Ready
        } else {
            Ready::Loading
        },
        drift: Drift::default(),
        keys: keys.clone(),
        fetch: RoomFetch::Idle,
        progress: None,
        fetching: None,
        whole_show: false,
        rest: None,
        gate: RoomGate::Off,
        gate_at: 0.0,
        started: false,
        fetch_error: None,
        tx,
        cancel: cancel.clone(),
        kick: kick.clone(),
        rooms: Arc::downgrade(&owner),
    });
    rooms.services.handle.spawn(socket::run(
        Arc::downgrade(&owner),
        socket::Start {
            serial,
            relay: b.relay,
            code: b.code,
            name: b.name,
            host_key: b.host_key,
            key: keys.public(),
            client: b.client,
            rx,
            cancel: cancel.clone(),
            kick,
            first,
        },
    ));
    drop(owner);
    let waited = rooms
        .services
        .handle
        .block_on(async { tokio::time::timeout(JOIN_WAIT, answer).await });
    let result = match waited {
        Ok(Ok(Ok(()))) => Ok(()),
        Ok(Ok(Err(e))) => Err(e),
        Ok(Err(_)) => Err(socket::Fail::Unreachable("the connection closed".into()).to_error()),
        Err(_) => Err(socket::Fail::Unreachable("timed out".into()).to_error()),
    };
    let mut slot = rooms.lock();
    let ours = slot.as_ref().is_some_and(|a| a.serial == serial);
    match result {
        Ok(()) => slot
            .as_ref()
            .filter(|a| a.serial == serial)
            .and_then(Active::info)
            .ok_or_else(not_in_room),
        Err(e) => {
            if ours {
                *slot = None;
            }
            cancel.cancel();
            Err(e)
        }
    }
}

fn attach(rooms: &Rooms, session: u64) -> Result<Option<RoomInfo>, CoreError> {
    let s = rooms
        .playback
        .sessions
        .get(session)
        .ok_or(CoreError::NotFound {
            what: Entity::Session,
            id: session,
        })?;
    let stored = rooms.services.store.read(|c| stored_offset(c, s.file))?;
    let mut out = Vec::new();
    let info = {
        let mut slot = rooms.lock();
        let a = slot.as_mut().ok_or_else(not_in_room)?;
        let host = a.is_host();
        // Only the host picks for a room that has nothing yet (#922)
        if a.episode.is_none() && !host {
            return Err(not_in_room());
        }
        let mine = episode::of_session(&s);
        // A member's player still on another episode, as the room moves on,
        // does not take the room's: it stays as it was, Get media on offer
        if !host
            && let Some(room) = &a.episode
            && !episode::plays(&mine, room)
        {
            return Ok(a.info());
        }
        // The room's episode, unless this session moves the room off it or
        // picks the first one
        let room_episode = a
            .episode
            .clone()
            .filter(|r| !host || episode::same(&mine, r));
        a.drift = Drift::default();
        a.session = Some(session);
        a.file = Some(s.file);
        a.ready = Ready::Ready;
        // Whatever Get media was doing, the episode is open now
        a.stop_fetch();
        a.mine = Some(mine.clone());
        if let Some(room_episode) = room_episode {
            // The file that set the timeline plays at its own offset, nought
            // unless set by hand. Every other file, a new host's after a
            // handover included, keeps the offset stored for it or takes the
            // openings' suggestion.
            if let Some((_, offset)) = a.timeline.filter(|(f, _)| *f == s.file) {
                a.suggested = None;
                (a.offset, a.offset_fixed) = (offset, true);
            } else {
                a.suggested = sync::suggest_offset(room_episode.opening_start, s.opening_start);
                (a.offset, a.offset_fixed) = match stored {
                    Some(offset) => (offset, true),
                    None => (a.suggested.unwrap_or(0.0), false),
                };
            }
            a.send(a.status());
            out.push(a.bring(session));
        } else {
            // The whole room moves, or takes its first episode, and this file
            // is its timeline now. The relay starts it paused at nought, and
            // its answer brings this player there too; until then the room is
            // taken to be there already, so no tick corrects against the old
            // episode.
            a.fresh_episode();
            a.timeline = Some((s.file, 0.0));
            a.offset = 0.0;
            a.offset_fixed = true;
            a.suggested = None;
            let reset = at_start(a.clock.server_now(local_ms()), a.relayed.rev);
            a.settle(a.relayed);
            a.state = reset;
            a.episode = Some(mine.clone());
            a.send(ClientMsg::Episode { episode: mine });
        }
        out.push(Say::debug(
            format!("Room: session {session} attached"),
            a.changed(),
        ));
        a.info()
    };
    emit(&rooms.services, out);
    Ok(info)
}

fn command(
    rooms: &Rooms,
    action: RoomAction,
    position: f64,
) -> Result<Option<RoomInfo>, CoreError> {
    if !position.is_finite() || !(0.0..=proto::MAX_POSITION).contains(&position) {
        return Err(CoreError::invalid(
            "position",
            anibeam_contract::i18n::tr(
                "core-prefs-outside",
                &[
                    ("value", position.to_string().into()),
                    ("low", "0".into()),
                    ("high", proto::MAX_POSITION.to_string().into()),
                ],
            ),
        ));
    }
    let mut out = Vec::new();
    let info = {
        let mut slot = rooms.lock();
        let a = slot.as_mut().ok_or_else(not_in_room)?;
        if a.you.is_none() {
            return Err(not_in_room());
        }
        command_in(rooms, a, action, position, &mut out);
        a.info()
    };
    emit(&rooms.services, out);
    Ok(info)
}

/// `command`, under the room's lock.
fn command_in(
    rooms: &Rooms,
    a: &mut Active,
    action: RoomAction,
    position: f64,
    out: &mut Vec<Say>,
) {
    let canonical = (position - a.offset).clamp(0.0, proto::MAX_POSITION);
    let action = match action {
        RoomAction::Play => Action::Play,
        RoomAction::Pause => Action::Pause,
        RoomAction::Seek => Action::Seek,
    };
    // The ready gate (#922): the host's first play of an episode waits while
    // someone lacks it or is still opening it, and the host chooses.
    if a.gate != RoomGate::Off || (action == Action::Play && a.is_host() && !a.started) {
        match action {
            Action::Play if !a.all_ready() => {
                a.gate_at = canonical;
                if a.gate == RoomGate::Off {
                    a.gate = RoomGate::Asking;
                }
                // The player played on its own press: it stays paused here
                if let Some(session) = a.session {
                    out.push(apply(session, false, Some(position), 1.0));
                }
                out.push(Say::debug(
                    format!("Room {}: not everyone is ready", a.code),
                    a.changed(),
                ));
                return;
            }
            Action::Seek => a.gate_at = canonical,
            // A pause takes the wait back; a play with everyone ready starts
            _ => a.gate = RoomGate::Off,
        }
    }
    // Taken as done at once, so the ticks before the relay's answer measure
    // against what this member just did rather than what came before.
    let now = Instant::now();
    match action {
        Action::Play => a.state.playing = true,
        Action::Pause => a.state.playing = false,
        Action::Seek => {}
    }
    a.state.position = canonical;
    a.state.at_ms = a.clock.server_now(local_ms());
    a.drift.last = Some(now);
    match action {
        Action::Seek => {
            let due = a
                .seek_sent
                .map(|sent| sent + SEEK_GAP)
                .filter(|due| now < *due);
            if let Some(due) = due {
                // Too soon after the last: held, and sent when the gap is up
                // unless another seek replaces it first.
                a.held_seek = Some(canonical);
                if !a.flush_armed {
                    a.flush_armed = true;
                    let (owner, serial) = (a.rooms.clone(), a.serial);
                    rooms.services.handle.spawn(async move {
                        tokio::time::sleep_until(tokio::time::Instant::from_std(due)).await;
                        if let Some(rooms) = owner.upgrade() {
                            flush_seek(&rooms, serial);
                        }
                    });
                }
            } else {
                a.held_seek = None;
                a.send_cmd(Action::Seek, canonical, now);
            }
        }
        _ => {
            // A held seek goes first, so the relay hears them in the order made.
            if let Some(held) = a.held_seek.take() {
                a.send_cmd(Action::Seek, held, now);
            }
            a.send_cmd(action, canonical, now);
        }
    }
}

/// `RoomStart`: the host's answer to the ready gate. Waiting with everyone
/// ready already starts at once. Anyone else, or no gate, changes nothing.
fn start_room(rooms: &Rooms, wait: bool) -> Result<Option<RoomInfo>, CoreError> {
    let mut out = Vec::new();
    let info = {
        let mut slot = rooms.lock();
        let a = slot.as_mut().ok_or_else(not_in_room)?;
        if a.is_host() && a.gate != RoomGate::Off {
            if wait && !a.all_ready() {
                a.gate = RoomGate::Waiting;
                out.push(Say::debug(
                    format!("Room {}: waiting for everyone", a.code),
                    a.changed(),
                ));
            } else {
                out.extend(a.open_gate());
            }
        }
        a.info()
    };
    emit(&rooms.services, out);
    Ok(info)
}

/// Sends the seek `command` held back, once `SEEK_GAP` is up.
fn flush_seek(rooms: &Rooms, serial: u64) {
    let mut slot = rooms.lock();
    let Some(a) = slot.as_mut().filter(|a| a.serial == serial) else {
        return;
    };
    a.flush_armed = false;
    if let Some(held) = a.held_seek.take() {
        a.send_cmd(Action::Seek, held, Instant::now());
    }
}

fn set_offset(rooms: &Rooms, offset: f64) -> Result<Option<RoomInfo>, CoreError> {
    if !offset.is_finite() || offset.abs() > MAX_OFFSET {
        return Err(CoreError::invalid(
            "offset",
            anibeam_contract::i18n::tr(
                "core-prefs-outside",
                &[
                    ("value", offset.to_string().into()),
                    ("low", (-MAX_OFFSET).to_string().into()),
                    ("high", MAX_OFFSET.to_string().into()),
                ],
            ),
        ));
    }
    let mut out = Vec::new();
    let (file, info) = {
        let mut slot = rooms.lock();
        let a = slot.as_mut().ok_or_else(not_in_room)?;
        a.offset = offset;
        a.offset_fixed = true;
        // The timeline's own file keeps its offset in memory; see `timeline`.
        let keep = !a.on_timeline();
        if !keep {
            a.timeline = a.file.map(|f| (f, offset));
        }
        if let Some(session) = a.session
            && a.you.is_some()
        {
            out.push(a.bring(session));
        }
        out.push(Say::debug(
            format!("Room: offset {offset:+.2} s"),
            a.changed(),
        ));
        (a.file.filter(|_| keep), a.info())
    };
    if let Some(file) = file {
        rooms
            .services
            .store
            .write(move |c| save_offset(c, file, offset))?;
    }
    emit(&rooms.services, out);
    Ok(info)
}

/// Leaves the room for good: the relay frees the place at once.
fn leave(rooms: &Rooms) {
    let taken = rooms.lock().take();
    let Some(mut a) = taken else {
        return;
    };
    a.send(ClientMsg::Bye);
    a.end();
    say_room(
        rooms,
        Level::Info,
        anibeam_contract::words!("core-room-left"),
        format!("Left room {}", a.code),
        None,
    );
}

/// The core is closing: say goodbye, quietly.
pub fn shutdown(rooms: &Rooms) {
    if let Some(mut a) = rooms.lock().take() {
        a.send(ClientMsg::Bye);
        a.end();
    }
}

// ---------------------------------------------------------------------------
// The player
// ---------------------------------------------------------------------------

/// One tick of a playback session. Only the attached session's count. A player
/// that disagrees with the room on playing is put right, unless this member's
/// own command is still out, which the relay's answer settles.
fn tick(rooms: &Rooms, session: u64, position: f64, paused: bool) {
    if !position.is_finite() {
        return;
    }
    let gap = Duration::from_millis(rooms.gap_ms.load(Ordering::Relaxed));
    let out = {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut() else {
            return;
        };
        if a.session != Some(session) {
            return;
        }
        let now = Instant::now();
        a.drift.tick = Some(LocalTick {
            position,
            paused,
            at: now,
        });
        if !a.connected || a.you.is_none() {
            return;
        }
        // An own command the relay never answered was lost on the way or
        // dropped over the rate: the relay's last state is the room's again.
        if a.in_flight > 0
            && a.sent_at
                .is_some_and(|t| now.saturating_duration_since(t) >= LOST_AFTER)
        {
            a.in_flight = 0;
            if a.held_seek.is_none() {
                a.settle(a.relayed);
            }
        }
        if !a.drift.may_correct(now, gap) {
            return;
        }
        if a.state.playing == paused {
            if a.ahead() {
                return;
            }
            let seek_to = sync::on_state(Some(position), a.target());
            a.drift.last = Some(now);
            a.drift.speed = 1.0;
            apply(session, a.state.playing, seek_to, 1.0)
        } else {
            let Some(c) = sync::on_tick(position, a.target(), a.drift.speed, a.state.playing)
            else {
                return;
            };
            a.drift.last = Some(now);
            a.drift.speed = c.speed;
            apply(session, a.state.playing, c.seek_to, c.speed)
        }
    };
    emit(&rooms.services, vec![out]);
}

/// The session closed: the room stops driving it, and the others see this member
/// is no longer ready.
fn closed(rooms: &Rooms, session: u64) {
    let out = {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut() else {
            return;
        };
        if a.session != Some(session) {
            return;
        }
        a.detach();
        a.send(a.status());
        Say::debug(format!("Room: session {session} closed"), a.changed())
    };
    emit(&rooms.services, vec![out]);
}

/// The skip windows job found where the attached session's opening starts: the
/// suggestion follows, and moves the offset when none was stored or set.
fn opening_known(rooms: &Rooms, session: u64, start: f64) {
    let mut out = Vec::new();
    {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut() else {
            return;
        };
        if a.session != Some(session) || a.on_timeline() {
            return;
        }
        let Some(host) = a.episode.as_ref().map(|e| e.opening_start) else {
            return;
        };
        a.suggested = sync::suggest_offset(host, Some(start));
        if !a.offset_fixed
            && let Some(suggested) = a.suggested
            && suggested != a.offset
        {
            a.offset = suggested;
            if a.connected {
                out.push(a.bring(session));
            }
        }
        out.push(Say::debug(
            format!("Room: the opening is at {start:.2}"),
            a.changed(),
        ));
    }
    emit(&rooms.services, out);
}

// ---------------------------------------------------------------------------
// What the relay says (called by the socket task)
// ---------------------------------------------------------------------------

/// The relay welcomed this member, first or again. False when the room is no
/// longer this core's.
fn on_welcome(rooms: &Rooms, serial: u64, you: MemberId, room: RoomView, server_ms: u64) -> bool {
    let mut out = Vec::new();
    let to_match;
    {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.serial == serial) else {
            return false;
        };
        let first = a.you.is_none();
        let was_down = !first && !a.connected;
        a.you = Some(you);
        a.clock.rough(server_ms, local_ms());
        a.members = room.members;
        a.connected = true;
        // Anything sent before this welcome was dropped with the connection.
        a.settle(room.state);
        if played(&room.state) {
            a.started = true;
        }
        let same = |a: Option<&proto::EpisodeRef>, b: Option<&proto::EpisodeRef>| match (a, b) {
            (Some(a), Some(b)) => episode::same(a, b),
            (None, None) => true,
            _ => false,
        };
        let moved = !same(a.episode.as_ref(), room.episode.as_ref());
        // The host moved the room while the connection was down: that move
        // never reached the relay, so it is made again rather than undone.
        let remake = (!first && moved && room.host == Some(you) && a.session.is_some())
            .then(|| a.mine.clone())
            .flatten()
            .filter(|mine| !same(Some(mine), room.episode.as_ref()));
        if let Some(mine) = remake {
            a.state = at_start(a.clock.server_now(local_ms()), room.state.rev);
            a.episode = Some(mine.clone());
            a.send(ClientMsg::Episode { episode: mine });
            to_match = None;
        } else {
            a.episode = room.episode.clone();
            if moved && !first {
                // The host moved on while this member was away.
                a.detach();
                a.timeline = None;
                a.fresh_episode();
                if played(&room.state) {
                    a.started = true;
                }
            }
            if let Some(session) = a.session
                && !first
            {
                out.push(a.bring(session));
            }
            // A member who comes back to the same episode was already told about it.
            to_match = (a.session.is_none() && (first || moved))
                .then(|| room.episode.clone())
                .flatten();
        }
        a.send(a.status());
        if was_down {
            out.push(Say::said(
                Level::Info,
                anibeam_contract::words!("core-room-back"),
                format!("Back in room {}", a.code),
                a.changed(),
            ));
        } else if !first {
            out.push(Say::debug(
                format!("Room {}: welcomed again", a.code),
                a.changed(),
            ));
        }
    }
    emit(&rooms.services, out);
    if let Some(e) = to_match {
        find(rooms, serial, e);
    }
    true
}

/// Whether a room's state shows its episode has played: playing now, or
/// paused past the start. A core that joins late, comes back after a restart
/// or takes over as host learns it from here, having seen no play itself.
fn played(state: &PlayState) -> bool {
    state.playing || state.position > 0.0
}

/// Looks for the room's episode in this library and says what it found.
fn find(rooms: &Rooms, serial: u64, e: proto::EpisodeRef) {
    let local = match rooms.services.store.read(|c| episode::find_local(c, &e)) {
        Ok(local) => local,
        Err(err) => {
            tracing::warn!("room: could not look for the episode: {err}");
            episode::Local::default()
        }
    };
    show(rooms, serial, e, local);
}

/// Says `RoomEpisode` with what this library has of the room's episode.
fn show(rooms: &Rooms, serial: u64, e: proto::EpisodeRef, local: episode::Local) {
    let mut out = Vec::new();
    {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.serial == serial) else {
            return;
        };
        // Attached in the meantime, or moved on to another episode.
        if a.session.is_some() || a.episode.as_ref().is_none_or(|now| !episode::same(now, &e)) {
            return;
        }
        a.ready = if local.file.is_some() {
            Ready::Loading
        } else {
            Ready::Missing
        };
        a.send(a.status());
        a.suggested = sync::suggest_offset(e.opening_start, local.opening);
        out.push(Say {
            level: Level::Info,
            words: None,
            message: match local.file {
                Some(file) => format!(
                    "Room {}: {} {} is file {file} here",
                    a.code, e.series_title, e.episode_key
                ),
                None => format!(
                    "Room {}: {} {} is not in this library",
                    a.code, e.series_title, e.episode_key
                ),
            },
            body: EventBody::RoomEpisode {
                episode: episode::to_contract(&e),
                series: local.series,
                file: local.file,
            },
        });
        out.push(Say::debug(
            format!("Room {}: ready state", a.code),
            a.changed(),
        ));
    }
    emit(&rooms.services, out);
}

fn on_pong(rooms: &Rooms, serial: u64, c: u64, s: u64) {
    if let Some(a) = rooms.lock().as_mut().filter(|a| a.serial == serial) {
        a.clock.pong(c, s, local_ms());
    }
}

/// A play state from the relay, and what the player does about it, if anything.
fn take_state(a: &mut Active, state: PlayState, by: MemberId) -> Option<Say> {
    if state.rev < a.relayed.rev {
        return None;
    }
    a.relayed = state;
    if state.playing {
        // The episode has started, however it came to: no gate from here
        a.started = true;
        a.gate = RoomGate::Off;
    }
    let own = a.you == Some(by);
    if own {
        a.in_flight = a.in_flight.saturating_sub(1);
    }
    if a.ahead() {
        // The relay has yet to answer what this member did since, and put this
        // state before it; that answer carries the room as it ends up, so the
        // player waits for it.
        a.stale |= !own;
        return None;
    }
    // An own command's echo applies nothing when the player already did it and
    // nobody came between. Otherwise the relay's order is the room's, and the
    // player follows it.
    let disagrees = a.drift.tick.is_some_and(|t| t.paused == state.playing);
    let follow = !own || a.stale || disagrees;
    a.stale = false;
    a.state = state;
    let session = a.session.filter(|_| follow)?;
    let now = Instant::now();
    let local = a.drift.tick.map(|t| t.now(now));
    let seek_to = sync::on_state(local, a.target());
    a.drift.last = Some(now);
    a.drift.speed = 1.0;
    Some(apply(session, state.playing, seek_to, 1.0))
}

/// Every other message from the relay. False when the room is no longer this
/// core's.
fn on_message(rooms: &Rooms, serial: u64, msg: ServerMsg) -> bool {
    let mut out = Vec::new();
    let mut to_match = None;
    let mut secret = None;
    {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.serial == serial) else {
            return false;
        };
        match msg {
            ServerMsg::State { state, by } => out.extend(take_state(a, state, by)),
            ServerMsg::Members { members } => {
                let before = std::mem::replace(&mut a.members, members);
                // The gate ends with the host role. Waiting for everyone opens it
                // once everyone is ready; asking still waits for the host's choice.
                if a.gate != RoomGate::Off {
                    if !a.is_host() {
                        a.gate = RoomGate::Off;
                    } else if a.gate == RoomGate::Waiting && a.all_ready() {
                        out.extend(a.open_gate());
                    }
                }
                let info = a.info();
                let others = |m: &&Member| Some(m.id) != a.you;
                let mut said = false;
                for m in a.members.iter().filter(others) {
                    if !before.iter().any(|b| b.id == m.id) {
                        said = true;
                        out.push(Say::said(
                            Level::Info,
                            anibeam_contract::words!("core-room-member-joined", name = &m.name),
                            format!("Room {}: {} joined", a.code, m.name),
                            EventBody::RoomChanged { room: info.clone() },
                        ));
                    }
                }
                for m in before.iter().filter(others) {
                    if !a.members.iter().any(|now| now.id == m.id) {
                        said = true;
                        out.push(Say::said(
                            Level::Info,
                            anibeam_contract::words!("core-room-member-left", name = &m.name),
                            format!("Room {}: {} left", a.code, m.name),
                            EventBody::RoomChanged { room: info.clone() },
                        ));
                    }
                }
                if !said {
                    out.push(Say::debug(
                        format!("Room {}: members", a.code),
                        EventBody::RoomChanged { room: info },
                    ));
                }
            }
            ServerMsg::Episode { episode, state, by } => {
                a.episode = Some(episode.clone());
                a.settle(state);
                a.fresh_episode();
                if a.you == Some(by) {
                    // This member, the host, moved the room: their player follows
                    // the fresh state, paused at the start.
                    if let Some(session) = a.session {
                        a.ready = Ready::Ready;
                        out.push(a.bring(session));
                    }
                    a.send(a.status());
                } else {
                    // The old episode stops; the shell opens the new one.
                    if let Some(session) = a.session {
                        out.push(apply(session, false, None, 1.0));
                    }
                    a.detach();
                    a.timeline = None;
                    to_match = Some(episode.clone());
                }
                out.push(Say::debug(
                    format!(
                        "Room {}: now {} {}",
                        a.code, episode.series_title, episode.episode_key
                    ),
                    a.changed(),
                ));
            }
            ServerMsg::Sealed { from, data } => {
                // Sealed to this member's key by the sender's: one that does not
                // open was not for this pair, or was changed on the way. The
                // sender's key is the one the relay listed, unchecked (#982)
                let theirs = a
                    .members
                    .iter()
                    .find(|m| m.id == from)
                    .and_then(|m| m.key.clone());
                let opened = theirs
                    .and_then(|k| a.keys.open(&a.code, &k, &data))
                    .and_then(|plain| serde_json::from_slice::<media::Secret>(&plain).ok());
                match opened {
                    Some(s) => secret = Some((from, s)),
                    None => tracing::debug!(
                        "Room {}: a sealed payload from member {} did not open",
                        a.code,
                        from.0
                    ),
                }
            }
            _ => {}
        }
    }
    emit(&rooms.services, out);
    if let Some(e) = to_match {
        find(rooms, serial, e);
    }
    if let Some((from, s)) = secret {
        media::on_secret(rooms, serial, from, s);
    }
    true
}

/// The connection dropped and the task is trying again. Said once per outage.
/// False when the room is no longer this core's.
fn on_lost(rooms: &Rooms, serial: u64) -> bool {
    let out = {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.serial == serial) else {
            return false;
        };
        if !a.connected {
            return true;
        }
        a.connected = false;
        Say::said(
            Level::Warn,
            anibeam_contract::words!("core-room-lost"),
            format!("Room {}: the connection dropped, trying again", a.code),
            a.changed(),
        )
    };
    emit(&rooms.services, vec![out]);
    true
}

/// The room is gone: the relay closed it or refused this member, or two minutes
/// of trying again came to nothing.
fn finish(rooms: &Rooms, serial: u64) {
    let taken = rooms.lock().take_if(|a| a.serial == serial);
    let Some(mut a) = taken else {
        return;
    };
    a.end();
    say_room(
        rooms,
        Level::Warn,
        anibeam_contract::words!("core-room-closed"),
        format!("Room {} closed", a.code),
        None,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_cleaned_like_the_relay_does() {
        assert_eq!(clean_name("  Ann "), Some("Ann".into()));
        assert_eq!(clean_name(""), None);
        assert_eq!(clean_name("   "), None);
        assert_eq!(clean_name("a\u{7}b"), None);
        assert_eq!(clean_name(&"x".repeat(32)), Some("x".repeat(32)));
        assert_eq!(clean_name(&"x".repeat(33)), None);
    }

    /// A member, id 2, in a room with id 1, attached to session 7 and paused at 0.
    fn active() -> Active {
        let paused = at_start(local_ms(), 0);
        let (tx, _rx) = mpsc::unbounded_channel();
        Active {
            serial: 1,
            relay: "http://relay.example".into(),
            code: "K7QM2X".into(),
            you: Some(MemberId(2)),
            members: Vec::new(),
            episode: None,
            state: paused,
            relayed: paused,
            in_flight: 0,
            sent_at: None,
            stale: false,
            held_seek: None,
            seek_sent: None,
            flush_armed: false,
            connected: true,
            clock: Clock::default(),
            session: Some(7),
            file: Some(70),
            mine: None,
            timeline: None,
            offset: 0.0,
            offset_fixed: false,
            suggested: None,
            ready: Ready::Ready,
            drift: Drift::default(),
            keys: Arc::new(seal::Keys::new()),
            fetch: RoomFetch::Idle,
            progress: None,
            fetching: None,
            whole_show: false,
            rest: None,
            gate: RoomGate::Off,
            gate_at: 0.0,
            started: false,
            fetch_error: None,
            tx,
            cancel: CancellationToken::new(),
            kick: Arc::new(Notify::new()),
            rooms: Weak::new(),
        }
    }

    fn state(playing: bool, position: f64, rev: u64) -> PlayState {
        PlayState {
            playing,
            position,
            rate: 1.0,
            at_ms: local_ms(),
            rev,
        }
    }

    /// What a `Say` tells the player: playing, and where to seek.
    fn told(say: Option<Say>) -> Option<(bool, Option<f64>)> {
        match say?.body {
            EventBody::RoomApply {
                playing, seek_to, ..
            } => Some((playing, seek_to)),
            other => panic!("{other:?}"),
        }
    }

    /// The player's last tick: paused or not, at `position`.
    fn ticked(a: &mut Active, position: f64, paused: bool) {
        a.drift.tick = Some(LocalTick {
            position,
            paused,
            at: Instant::now(),
        });
    }

    #[test]
    fn an_own_echo_with_nothing_between_applies_nothing() {
        let mut a = active();
        ticked(&mut a, 10.0, true);
        a.state.playing = true;
        a.send_cmd(Action::Play, 10.0, Instant::now());
        ticked(&mut a, 10.0, false);
        assert_eq!(
            told(take_state(&mut a, state(true, 10.0, 1), MemberId(2))),
            None
        );
        assert!(!a.ahead());
        assert!(a.state.playing);
    }

    #[test]
    fn the_relay_order_wins_a_play_and_pause_race() {
        // This member pauses at 20 while member 1 plays from 10. The relay puts
        // the play first: its state comes before this member's own echo.
        let mut a = active();
        a.relayed = state(true, 15.0, 4);
        a.state = a.relayed;
        ticked(&mut a, 20.0, false);
        a.state.playing = false;
        a.state.position = 20.0;
        a.send_cmd(Action::Pause, 20.0, Instant::now());
        ticked(&mut a, 20.0, true);
        // Member 1's play: the player waits, its own pause is still out.
        assert_eq!(
            told(take_state(&mut a, state(true, 10.0, 5), MemberId(1))),
            None
        );
        assert!(!a.state.playing, "the player is held to its own pause");
        // The echo of the pause holds the room as it ends up: paused at 20. The
        // player is there already, but the echo is applied all the same.
        assert_eq!(
            told(take_state(&mut a, state(false, 20.0, 6), MemberId(2))),
            Some((false, None))
        );
        assert!(!a.state.playing);
        assert!(!a.stale);
    }

    #[test]
    fn an_own_echo_the_player_disagrees_with_is_applied() {
        // Member 1's play reached this player before this member paused, so the
        // player plays; the relay put the pause last.
        let mut a = active();
        a.send_cmd(Action::Pause, 20.0, Instant::now());
        ticked(&mut a, 20.0, false);
        assert_eq!(
            told(take_state(&mut a, state(false, 20.0, 1), MemberId(2))),
            Some((false, None))
        );
    }

    #[test]
    fn an_older_state_is_ignored() {
        let mut a = active();
        a.relayed = state(true, 50.0, 9);
        assert_eq!(
            told(take_state(&mut a, state(false, 1.0, 8), MemberId(1))),
            None
        );
        assert_eq!(a.relayed.rev, 9);
    }

    #[test]
    fn a_held_seek_keeps_the_player_from_snapping_back() {
        // Seeks at 30 then 35 inside the gap: 30 goes, 35 waits. The echo of 30
        // must not pull the player back from 35.
        let mut a = active();
        let now = Instant::now();
        a.send_cmd(Action::Seek, 30.0, now);
        a.held_seek = Some(35.0);
        a.state.position = 35.0;
        ticked(&mut a, 35.0, true);
        assert_eq!(
            told(take_state(&mut a, state(false, 30.0, 1), MemberId(2))),
            None
        );
        assert_eq!(a.state.position, 35.0);
    }

    #[test]
    fn a_relay_is_an_http_address_with_a_host() {
        assert!(relay_ok("https://rooms.example.com"));
        assert!(relay_ok("http://127.0.0.1:8795/"));
        assert!(!relay_ok("ftp://rooms.example.com"));
        assert!(!relay_ok("rooms.example.com"));
        assert!(!relay_ok("https://"));
        assert!(!relay_ok(""));
    }
}
