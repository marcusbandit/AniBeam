//! Discord rich presence: with the switch on, a running Discord client shows
//! the series, the episode and the poster of what the player is playing.
//! Off by default, per machine, and silent when Discord is not there.
//!
//! The playback session feeds it. `open` builds the activity but shows
//! nothing, since a file that never plays is not being watched; the first
//! playing tick shows it, a pause shows it without the clock and arms a
//! one-shot timer that clears it if the pause outlasts `PAUSE_CLEAR`, and the
//! close of the session that owns it clears it. Only transitions and seeks
//! send anything, and `decide` is the pure rule over them. Discord's clock
//! reads the position in the episode, not the time since it was opened.
//! Sends set a desired state that one worker thread, the owner of the
//! connection, converges to, so a tick never waits on a socket and nothing
//! the worker must do is lost to a burst.

pub mod discord;
pub mod join;

use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, LockResult, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::Playback;
use anibeam_contract::*;
use anibeam_foundation::above::Above;
use anibeam_foundation::events::EventBus;
use anibeam_foundation::prefs;
use anibeam_foundation::sync::recover;

/// How long a pause may stand before the activity is cleared.
pub const PAUSE_CLEAR: Duration = Duration::from_secs(5 * 60);
/// The least time between two attempts to reach Discord.
pub const RETRY: Duration = Duration::from_secs(30);
/// How long a connection may go with nothing sent before the worker looks
/// at it: what plays is sent again, which finds a Discord that has quit, and
/// a connection with nothing to show is let go. Discord never says it left,
/// so without this Settings would go on saying it is connected.
pub const CHECK: Duration = Duration::from_secs(30);
/// Discord's limit on every text field of an activity, in UTF-16 code
/// units, which is how its JavaScript counts: an emoji takes two.
pub const FIELD_MAX: usize = 128;
/// The fewest characters Discord takes in a text field: it refuses a
/// shorter `details`, `state` or `large_text`, and the whole activity with it.
const FIELD_MIN: usize = 2;
/// What a field too short for Discord is padded with: a zero width space,
/// which counts as a character and shows as nothing.
const FIELD_PAD: char = '\u{200B}';
/// The art uploaded to the Discord application, shown when there is no
/// poster or Discord will not take a poster's URL.
pub const FALLBACK_ART: &str = "anibeam";
/// How far, in seconds, the position may move off the wall clock between two
/// ticks before Discord's clock is sent again. Past it is a seek; under it
/// is tick jitter, or a speed other than 1x, which never adds up to a jump.
const SEEK_SLACK: f64 = 10.0;
/// How long a shutdown or a test's flush waits for the worker.
const FLUSH_WAIT: Duration = Duration::from_secs(5);

/// The Discord application id compiled into this build, if any. Without
/// one the switch can do nothing. An id set to nothing is no id.
pub fn bundled_client_id() -> Option<&'static str> {
    option_env!("ANIBEAM_DISCORD_CLIENT_ID").filter(|id| !id.is_empty())
}

/// What Discord shows for one session.
#[derive(Clone, Debug, PartialEq)]
pub struct Activity {
    /// The series title.
    pub details: String,
    /// The episode line: its code and title, "Extra", or nothing for a film.
    pub state: Option<String>,
    /// The poster's remote URL, when the series has one.
    pub poster: Option<String>,
    pub large_text: String,
    /// Unix seconds Discord counts up from: now less the position, so the
    /// clock reads how far into the episode playback is. None while paused,
    /// so the clock stops.
    pub start: Option<u64>,
    /// Who can join and how: the worker puts it on from what the core says
    /// last, so the rule over playback never sees it.
    pub party: Option<Party>,
}

/// The party an activity carries, which is what makes Discord offer Ask to
/// Join, Join and invites. Its join secret is handed only to someone this
/// person lets in, or to anyone when the party is open.
#[derive(Clone, Debug, PartialEq)]
pub struct Party {
    /// Groups everyone in the same room on Discord; never the code itself.
    pub id: String,
    pub size: u32,
    pub max: u32,
    /// `room:<code>` in a room, `ask` outside one: a request then asks this
    /// person to start a room.
    pub secret: String,
    /// Anyone may join straight away, with no request.
    pub open: bool,
}

/// What Discord tells this app without being asked.
#[derive(Clone, Debug, PartialEq)]
pub enum Incoming {
    /// Someone pressed Ask to Join on this person's profile.
    JoinRequest {
        user_id: String,
        name: String,
        avatar: Option<String>,
    },
    /// This person pressed Join or accepted an invite: the other side's
    /// join secret.
    Join { secret: String },
}

/// The secret outside a room.
pub const ASK_SECRET: &str = "ask";
/// What a room's join secret starts with, before its code.
pub const ROOM_SECRET: &str = "room:";
/// How long an Ask to Join stands: Discord lets it go after 30 seconds.
pub const ASK_STANDS: Duration = Duration::from_secs(30);
/// How long the connection stays up for a Discord that launched the app to
/// hand over a join, with nothing to show.
pub const LISTEN: Duration = Duration::from_secs(60);
/// How often an open connection is read for events.
const POLL: Duration = Duration::from_millis(500);

impl Activity {
    /// The activity for one file. An extra says so in place of an episode
    /// line and a film has none.
    pub fn describe(
        series_title: &str,
        code: &str,
        episode_title: Option<&str>,
        is_extra: bool,
        is_film: bool,
        poster: Option<String>,
    ) -> Activity {
        let state = if is_extra {
            Some(anibeam_contract::i18n::tr("core-presence-extra", &[]))
        } else if is_film {
            None
        } else {
            Some(match episode_title.filter(|t| !t.trim().is_empty()) {
                Some(title) => format!("{code} - {title}"),
                None => code.to_string(),
            })
        };
        Activity {
            details: field(series_title),
            state: state.map(|s| field(&s)),
            poster: poster.filter(|p| p.starts_with("https://") || p.starts_with("http://")),
            large_text: field(series_title),
            start: None,
            party: None,
        }
    }

    /// The `activity` object of a `SET_ACTIVITY` command. Type 3 is
    /// Watching.
    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "type": 3,
            "details": self.details,
            "assets": {
                "large_image": self.poster.as_deref().unwrap_or(FALLBACK_ART),
                "large_text": self.large_text,
            },
        });
        if let Some(state) = &self.state {
            v["state"] = json!(state);
        }
        if let Some(start) = self.start {
            v["timestamps"] = json!({"start": start});
        }
        if let Some(p) = &self.party {
            v["party"] = json!({"id": p.id, "size": [p.size, p.max]});
            if p.open {
                v["party"]["privacy"] = json!(1);
            }
            v["secrets"] = json!({"join": p.secret});
            v["instance"] = json!(false);
        }
        v
    }
}

/// A field as Discord takes it: trimmed, cut to its limit with "..." where
/// it was cut, and padded to its minimum, so a one-letter title such as "K"
/// still shows. Both ends are counted in UTF-16 code units, as Discord
/// counts them, so this is not `words::cut`, which counts characters.
fn field(s: &str) -> String {
    let s = s.trim();
    let mut out = if s.encode_utf16().count() <= FIELD_MAX {
        s.to_string()
    } else {
        let mut kept = String::new();
        let mut units = 0;
        for c in s.chars() {
            if units + c.len_utf16() > FIELD_MAX - 3 {
                break;
            }
            units += c.len_utf16();
            kept.push(c);
        }
        format!("{}...", kept.trim_end())
    };
    let short = FIELD_MIN.saturating_sub(out.encode_utf16().count());
    out.extend(std::iter::repeat_n(FIELD_PAD, short));
    out
}

/// A party id for this run outside any room, so two runs never share one.
fn solo_party_id() -> String {
    format!("solo-{:032x}", rand::random::<u128>())
}

/// A lock's guard back from a condvar wait, poisoned or not, as `recover`.
fn unpoison<T>(r: LockResult<T>) -> T {
    r.unwrap_or_else(|e| e.into_inner())
}

/// Why a send did not land.
#[derive(Debug)]
pub enum SendError {
    /// There is no connection, or there is no longer one.
    Gone(String),
    /// Discord answered, and refused this activity.
    Rejected(String),
}

impl From<io::Error> for SendError {
    fn from(e: io::Error) -> SendError {
        SendError::Gone(e.to_string())
    }
}

/// The connection to Discord, behind a trait so a test can stand in for it.
pub trait Transport: Send {
    /// Opens a fresh connection, closing any that stands. `Gone` when no
    /// Discord answered, `Rejected` when one answered and refused, in its
    /// words: Settings names either (#1029).
    fn connect(&mut self, client_id: &str) -> Result<(), SendError>;
    /// Sets the activity, or clears it with `None`.
    fn send(&mut self, activity: Option<&Activity>) -> Result<(), SendError>;
    /// What Discord has said unasked since the last poll, never blocking.
    fn poll(&mut self) -> Result<Vec<Incoming>, SendError> {
        Ok(Vec::new())
    }
    /// Answers an Ask to Join: an invite, or a close of the request.
    fn answer(&mut self, _user_id: &str, _invite: bool) -> Result<(), SendError> {
        Ok(())
    }
    fn close(&mut self);
}

// ---------------------------------------------------------------------------
// The rule
// ---------------------------------------------------------------------------

/// What the rule remembers between transitions.
#[derive(Debug, Default)]
pub(crate) struct State {
    enabled: bool,
    /// The session whose activity Discord shows, or showed before a long
    /// pause cleared it.
    owner: Option<u64>,
    /// The last playing tick's position and the time it came, which the
    /// next tick measures a seek against. None from a pause, close or
    /// switch off until playback shows again.
    last: Option<(f64, u64)>,
    paused: bool,
    shown: bool,
    /// Bumped on every transition, so a pause timer that fires after one
    /// knows it is stale.
    generation: u64,
}

pub(crate) enum Input<'a> {
    /// The switch went on, with the session playing right now and its
    /// position, if any.
    Enable(Option<(u64, &'a Activity, f64)>),
    Disable,
    Tick {
        session: u64,
        activity: &'a Activity,
        /// Seconds into the file.
        position: f64,
        paused: bool,
    },
    /// A pause timer armed at `generation` ran out.
    Elapsed {
        session: u64,
        generation: u64,
    },
    Close {
        session: u64,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum Action {
    Set(Activity),
    Clear,
    /// Start a pause timer that comes back as `Input::Elapsed`.
    Arm {
        session: u64,
        generation: u64,
    },
    /// Clear and let the connection go.
    Disconnect,
}

/// What one input asks of Discord. `now` is in Unix seconds.
pub(crate) fn decide(s: &mut State, input: Input, now: u64) -> Vec<Action> {
    match input {
        Input::Enable(playing) => {
            if s.enabled {
                return Vec::new();
            }
            s.enabled = true;
            match playing {
                Some((session, activity, position)) => play(s, session, activity, now, position),
                None => Vec::new(),
            }
        }
        Input::Disable => {
            if !s.enabled {
                return Vec::new();
            }
            *s = State {
                generation: s.generation + 1,
                ..State::default()
            };
            vec![Action::Disconnect]
        }
        Input::Tick {
            session,
            activity,
            position,
            paused,
        } => {
            if !s.enabled {
                return Vec::new();
            }
            if s.owner != Some(session) {
                // A session that has not played yet shows nothing.
                return if paused {
                    Vec::new()
                } else {
                    play(s, session, activity, now, position)
                };
            }
            match (paused, s.paused) {
                // An ordinary tick moves the position about as far as the
                // time since the last, or steadily more or less at another
                // speed; one that jumps past the slack is a seek, and moves
                // the clock. Discord's clock runs at wall speed whatever
                // the player's, which is near enough to leave alone. A
                // position that is no number tells nothing either way.
                (false, false) if s.shown => {
                    if !position.is_finite() {
                        return Vec::new();
                    }
                    let here = seconds(position);
                    let seek = s.last.is_some_and(|(there, then)| {
                        let moved = here - there;
                        let passed = now as f64 - then as f64;
                        (moved - passed).abs() > SEEK_SLACK
                    });
                    s.last = Some((here, now));
                    if !seek {
                        return Vec::new();
                    }
                    let start = clock(now, position);
                    vec![Action::Set(Activity {
                        start: Some(start),
                        ..activity.clone()
                    })]
                }
                // A resume, or playing again after a long pause cleared it.
                (false, _) => play(s, session, activity, now, position),
                (true, true) => Vec::new(),
                (true, false) => {
                    s.paused = true;
                    s.last = None;
                    s.generation += 1;
                    vec![
                        Action::Set(Activity {
                            start: None,
                            ..activity.clone()
                        }),
                        Action::Arm {
                            session,
                            generation: s.generation,
                        },
                    ]
                }
            }
        }
        Input::Elapsed {
            session,
            generation,
        } => {
            if s.enabled
                && s.owner == Some(session)
                && s.generation == generation
                && s.paused
                && s.shown
            {
                s.shown = false;
                vec![Action::Clear]
            } else {
                Vec::new()
            }
        }
        Input::Close { session } => {
            // The player closes the old session before it opens the next,
            // so a close that comes late must not clear the new one.
            if s.owner != Some(session) {
                return Vec::new();
            }
            let shown = s.shown;
            s.owner = None;
            s.last = None;
            s.paused = false;
            s.shown = false;
            s.generation += 1;
            if shown {
                vec![Action::Clear]
            } else {
                Vec::new()
            }
        }
    }
}

/// A session showing as playing, its clock reading from `position`, and the
/// tick that starts the seek check afresh.
fn play(s: &mut State, session: u64, activity: &Activity, now: u64, position: f64) -> Vec<Action> {
    let start = clock(now, position);
    s.owner = Some(session);
    s.last = Some((seconds(position), now));
    s.paused = false;
    s.shown = true;
    s.generation += 1;
    vec![Action::Set(Activity {
        start: Some(start),
        ..activity.clone()
    })]
}

/// The clock's start for a position: the moment the episode would have
/// begun had it played straight through to here, so paused time never
/// counts. A position that is no number counts from now.
fn clock(now: u64, position: f64) -> u64 {
    now.saturating_sub(seconds(position) as u64)
}

/// Seconds into the file. A position that is no number, or before the
/// start, reads as the start.
fn seconds(position: f64) -> f64 {
    if position.is_finite() && position > 0.0 {
        position
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// The core's side
// ---------------------------------------------------------------------------

/// The core's handle on presence. `enabled` is the hot path's only cost
/// while the switch is off: one load per tick.
pub struct Presence {
    enabled: AtomicBool,
    connected: Arc<AtomicBool>,
    /// Why Discord shows nothing, as the worker last found (#1029).
    problem: Arc<Mutex<Option<PresenceProblem>>>,
    pause_clear_ms: AtomicU64,
    check_ms: Arc<AtomicU64>,
    inner: Mutex<Inner>,
    bus: Arc<EventBus>,
    /// The watch together room Discord's join opens and reads, linked by
    /// `Core::open`.
    pub rooms: Above<dyn JoinRooms>,
}

/// What Discord's join asks of the watch together room, which sits above
/// presence (#854): its status, and a room opened for a friend let in.
pub trait JoinRooms: Send + Sync {
    /// The room this person is in, if any, as `RoomStatus` answers.
    fn status(&self) -> Option<RoomInfo>;
    /// A room opened for `session`, or for none, as `RoomCreate` opens one.
    fn create(self: Arc<Self>, session: Option<u64>) -> Result<RoomInfo, CoreError>;
}

#[derive(Default)]
struct Inner {
    state: State,
    worker: Option<Arc<Mailbox>>,
    /// A test's stand-in, taken by the worker when it starts.
    injected: Option<Box<dyn Transport>>,
    /// A stand-in was given, which makes presence available without an id.
    fake: bool,
    /// The room Discord shows, as the room team last said.
    room: Option<RoomShow>,
    /// Anyone on Discord may join this person's room without asking.
    room_open: bool,
    /// The party id outside a room, fixed for the run.
    solo: String,
    /// The party id of the room this person is in, new with each room.
    room_party: String,
    /// Asks waiting on an answer, by Discord user id.
    asks: HashMap<String, Ask>,
    /// Who this person let in to the room they are in now: a second ask
    /// from them is let in again without a question.
    let_in: HashSet<String>,
    /// A room Discord handed over to join, until the shell takes it.
    pending_join: Option<(String, Instant)>,
    /// Asked to listen before the switch came on at launch: handed to the
    /// worker when it starts.
    listen: Option<Instant>,
}

/// The room as Discord shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomShow {
    pub code: String,
    /// People in the room, this person included.
    pub size: u32,
    /// The series the room is on, when it has picked one.
    pub title: Option<String>,
}

/// Someone who pressed Ask to Join and has not had an answer.
#[derive(Clone, Debug)]
struct Ask {
    name: String,
    /// Bumped per ask, so the timer of an older one lets a newer one stand.
    serial: u64,
}

/// Called on the worker's thread with each event Discord sends.
type Handler = Arc<dyn Fn(Incoming) + Send + Sync>;

/// What the worker is to bring Discord to. A send overwrites it rather than
/// queueing behind the last, so a worker stuck on a socket comes back to
/// the newest state alone, and a clear can never be dropped.
#[derive(Default)]
struct Desired {
    /// What Discord should show; `None` clears it.
    want: Option<Activity>,
    /// The party every activity carries, and what shows when nothing plays.
    party: Option<Party>,
    idle: Option<Activity>,
    /// Stay connected with nothing to show until then, so a Discord that
    /// launched the app can hand over a join.
    listen: Option<Instant>,
    /// Answers to Ask to Join, sent once what they hand over has been shown.
    answers: Vec<(String, bool)>,
    /// Let the connection go before showing `want`. It stays set until the
    /// worker has done it, so a show that follows cannot hide it.
    disconnect: bool,
    quit: bool,
    /// Bumped on every change.
    version: u64,
    /// The newest version the worker has handled.
    done: u64,
    /// The worker has quit.
    stopped: bool,
}

/// The one place the core and the worker meet. The core only ever holds
/// its lock for a write, never across anything the worker does.
#[derive(Default)]
struct Mailbox {
    desired: Mutex<Desired>,
    /// Wakes the worker when the desired state changes.
    changed: Condvar,
    /// Wakes a flush or a shutdown when the worker catches up.
    handled: Condvar,
}

impl Mailbox {
    fn post(&self, change: impl FnOnce(&mut Desired)) {
        let mut d = recover(&self.desired);
        change(&mut d);
        d.version += 1;
        self.changed.notify_one();
    }

    /// Waits up to `wait` for `until` to stop holding.
    fn wait_while(&self, wait: Duration, until: impl FnMut(&mut Desired) -> bool) {
        let d = recover(&self.desired);
        let _ = self.handled.wait_timeout_while(d, wait, until);
    }
}

impl Presence {
    pub fn new(bus: Arc<EventBus>) -> Presence {
        Presence {
            enabled: AtomicBool::new(false),
            connected: Arc::new(AtomicBool::new(false)),
            problem: Arc::default(),
            pause_clear_ms: AtomicU64::new(PAUSE_CLEAR.as_millis() as u64),
            check_ms: Arc::new(AtomicU64::new(CHECK.as_millis() as u64)),
            inner: Mutex::new(Inner {
                solo: solo_party_id(),
                ..Inner::default()
            }),
            bus,
            rooms: Above::default(),
        }
    }

    /// Whether the switch can do anything in this build.
    pub fn available(&self) -> bool {
        bundled_client_id().is_some() || recover(&self.inner).fake
    }

    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Why Discord shows nothing while the switch is on, if the worker found
    /// a reason: no Discord answered, or one refused.
    pub fn problem(&self) -> Option<PresenceProblem> {
        recover(&self.problem).clone()
    }

    /// For a test: speak to `transport` in place of Discord.
    pub fn set_transport(&self, transport: Box<dyn Transport>) {
        let mut inner = recover(&self.inner);
        inner.injected = Some(transport);
        inner.fake = true;
    }

    /// For a test: clear a pause after `wait` rather than `PAUSE_CLEAR`.
    pub fn set_pause_clear(&self, wait: Duration) {
        self.pause_clear_ms
            .store(wait.as_millis() as u64, Ordering::Relaxed);
    }

    /// For a test: look at a quiet connection after `wait` rather than `CHECK`.
    pub fn set_check(&self, wait: Duration) {
        self.check_ms
            .store(wait.as_millis() as u64, Ordering::Relaxed);
    }

    /// For a test: returns once the worker has handled every send before it.
    pub fn flush(&self) {
        let worker = recover(&self.inner).worker.clone();
        if let Some(worker) = worker {
            let target = recover(&worker.desired).version;
            worker.wait_while(FLUSH_WAIT, |d| d.done < target && !d.stopped);
        }
    }
}

impl Drop for Presence {
    /// A core dropped without a shutdown still ends the worker.
    fn drop(&mut self) {
        if let Some(worker) = recover(&self.inner).worker.take() {
            worker.post(|d| d.quit = true);
        }
    }
}

/// One tick of a session, after the sessions lock is let go.
pub fn tick(
    playback: &Arc<Playback>,
    session: u64,
    activity: &Activity,
    position: f64,
    paused: bool,
) {
    if !playback.presence.enabled.load(Ordering::Relaxed) {
        return;
    }
    let mut inner = recover(&playback.presence.inner);
    // A tick runs after the sessions lock is let go, so its session's close
    // can come in between. `close` takes the session out of the map before
    // it reaches presence, and this looks in the map under the presence
    // lock: a tick that lost that race finds its session gone and shows
    // nothing, and one that won is decided before the close, which clears.
    if !playback.sessions.is_open(session) {
        return;
    }
    let actions = decide(
        &mut inner.state,
        Input::Tick {
            session,
            activity,
            position,
            paused,
        },
        anibeam_foundation::time::now_secs() as u64,
    );
    run(playback, &inner, actions);
}

pub fn close(playback: &Arc<Playback>, session: u64) {
    if !playback.presence.enabled.load(Ordering::Relaxed) {
        return;
    }
    feed(playback, Input::Close { session });
}

/// The switch at launch, from the stored preference.
pub fn sync(playback: &Arc<Playback>) {
    let on = playback
        .services
        .store
        .read(prefs::load_preferences)
        .map(|p| p.discord_presence)
        .unwrap_or(false);
    set_enabled(playback, on);
}

/// `GetPresence`: whether this build can show what plays on Discord,
/// whether it is talking to a Discord client right now, and if not, why.
pub fn status_call(playback: &Playback) -> Result<Reply, CoreError> {
    Ok(Reply::Presence {
        available: playback.presence.available(),
        connected: playback.presence.connected(),
        room_open: recover(&playback.presence.inner).room_open,
        problem: playback.presence.problem(),
    })
}

/// The switch. On shows the session playing now, if there is one; off
/// clears and lets the connection go. A build that cannot reach Discord
/// stays off whatever is asked.
pub fn set_enabled(playback: &Arc<Playback>, on: bool) {
    let p = &playback.presence;
    if on && !p.available() {
        return;
    }
    // Read before the presence lock is taken: presence takes the sessions
    // lock inside its own, never the other way round.
    let playing = on.then(|| playback.sessions.playing()).flatten();
    // The room open now, read outside the lock as `follow_rooms` reads it.
    let room = on.then(|| join::room_now(playback)).flatten();
    let mut inner = recover(&p.inner);
    // Checked again under the presence lock, as a tick is.
    let playing = playing.filter(|s| playback.sessions.is_open(s.id));
    p.enabled.store(on, Ordering::Relaxed);
    let input = match &playing {
        Some(s) => Input::Enable(Some((
            s.id,
            &s.presence,
            s.last_position.unwrap_or_default(),
        ))),
        None if on => Input::Enable(None),
        None => Input::Disable,
    };
    let first = on && inner.worker.is_none();
    if first {
        // Known before the first post, so a room already open goes out with
        // what plays rather than in a second set after it.
        join::adopt_room(&mut inner, room);
        let transport = inner
            .injected
            .take()
            .unwrap_or_else(|| Box::new(discord::Discord::default()));
        let client_id = bundled_client_id().unwrap_or_default().to_string();
        let weak = Arc::downgrade(playback);
        let handler: Handler = Arc::new(move |event| {
            if let Some(playback) = weak.upgrade() {
                join::incoming(&playback, event);
            }
        });
        inner.worker = spawn(p, transport, client_id, handler);
    }
    let actions = decide(
        &mut inner.state,
        input,
        anibeam_foundation::time::now_secs() as u64,
    );
    // What plays, its party and the listen go in one post, so the worker
    // never shows the activity without the party a moment before it.
    let mut posts = posts(playback, actions);
    if on && inner.worker.is_some() {
        posts.push(join::party_post(&inner));
        if let Some(until) = inner.listen.take() {
            posts.push(join::listen_post(until));
        }
    }
    post_all(&inner, posts);
    drop(inner);
    // The room team says each change of room on the bus; presence follows it from the
    // first switch on, starting from the room open now.
    if first {
        join::follow_rooms(playback);
    }
}

/// Clears what Discord shows and ends the worker, waiting a moment for it.
pub fn shutdown(playback: &Playback) {
    let p = &playback.presence;
    p.enabled.store(false, Ordering::Relaxed);
    let worker = {
        let mut inner = recover(&p.inner);
        inner.state = State::default();
        inner.worker.take()
    };
    if let Some(worker) = worker {
        worker.post(|d| d.quit = true);
        worker.wait_while(Duration::from_secs(1), |d| !d.stopped);
    }
}

fn feed(playback: &Arc<Playback>, input: Input) {
    let mut inner = recover(&playback.presence.inner);
    let actions = decide(
        &mut inner.state,
        input,
        anibeam_foundation::time::now_secs() as u64,
    );
    run(playback, &inner, actions);
}

/// A change to the desired state, as one post carries it.
type Post = Box<dyn FnOnce(&mut Desired)>;

/// Carries the actions out under the presence lock, so two callers' sends
/// reach the worker in the order their decisions were made. Nothing here
/// blocks: a send only overwrites the desired state, and the timer is a
/// spawned task.
fn run(playback: &Arc<Playback>, inner: &Inner, actions: Vec<Action>) {
    post_all(inner, posts(playback, actions));
}

/// What the actions change in the desired state, arming any timer they ask for.
fn posts(playback: &Arc<Playback>, actions: Vec<Action>) -> Vec<Post> {
    let mut posts: Vec<Post> = Vec::new();
    for action in actions {
        posts.push(match action {
            Action::Set(activity) => Box::new(move |d| d.want = Some(activity)),
            Action::Clear => Box::new(|d| d.want = None),
            // Off takes the room and the party with it, so nothing shows again until on
            Action::Disconnect => Box::new(|d| {
                d.want = None;
                d.party = None;
                d.idle = None;
                d.answers.clear();
                d.disconnect = true;
            }),
            Action::Arm {
                session,
                generation,
            } => {
                arm(playback, session, generation);
                continue;
            }
        });
    }
    posts
}

/// Hands the worker every change as one: a worker that wakes between two
/// posts sends the state between them, so what belongs together goes in one.
fn post_all(inner: &Inner, posts: Vec<Post>) {
    if posts.is_empty() {
        return;
    }
    if let Some(worker) = &inner.worker {
        worker.post(move |d| posts.into_iter().for_each(|post| post(d)));
    }
}

/// The one-shot pause timer.
fn arm(playback: &Arc<Playback>, session: u64, generation: u64) {
    let weak = Arc::downgrade(playback);
    let wait = Duration::from_millis(playback.presence.pause_clear_ms.load(Ordering::Relaxed));
    playback.services.handle.spawn(async move {
        tokio::time::sleep(wait).await;
        if let Some(playback) = weak.upgrade() {
            elapsed(&playback, session, generation);
        }
    });
}

/// The pause timer's handler.
pub(crate) fn elapsed(playback: &Arc<Playback>, session: u64, generation: u64) {
    if !playback.presence.enabled.load(Ordering::Relaxed) {
        return;
    }
    feed(
        playback,
        Input::Elapsed {
            session,
            generation,
        },
    );
}

// ---------------------------------------------------------------------------
// The worker
// ---------------------------------------------------------------------------

/// The longest refusal Settings shows, in characters: Discord's own words
/// are a line, and anything past this is not one.
const REFUSAL_MAX: usize = 200;

/// A refusal as Settings names it, in Discord's words cut to a line.
fn refused(message: &str) -> PresenceProblem {
    PresenceProblem::Refused {
        message: message.trim().chars().take(REFUSAL_MAX).collect(),
    }
}

/// Starts the worker over `transport`, sharing what `p` shows Settings.
fn spawn(
    p: &Presence,
    transport: Box<dyn Transport>,
    client_id: String,
    on_event: Handler,
) -> Option<Arc<Mailbox>> {
    let mailbox = Arc::new(Mailbox::default());
    let mut worker = Worker {
        transport,
        client_id,
        connected: p.connected.clone(),
        problem: p.problem.clone(),
        check_ms: p.check_ms.clone(),
        bus: p.bus.clone(),
        on_event,
        want: None,
        party: None,
        idle: None,
        listen_until: None,
        answers: Vec::new(),
        dirty: false,
        last_attempt: None,
        last_sent: None,
        posters: true,
        privacy: true,
        parties: true,
    };
    let theirs = mailbox.clone();
    let spawned = std::thread::Builder::new()
        .name("anibeam-discord".to_string())
        .spawn(move || worker.run(&theirs));
    match spawned {
        Ok(_) => Some(mailbox),
        Err(e) => {
            tracing::warn!("the Discord presence thread did not start: {e}");
            None
        }
    }
}

/// Owns the connection. Connects when there is something to show, and
/// while Discord is not there tries again at most once every `RETRY`.
struct Worker {
    transport: Box<dyn Transport>,
    client_id: String,
    connected: Arc<AtomicBool>,
    /// Why Discord shows nothing, for Settings: set by a failed connect or a
    /// refused activity, cleared by one that lands or the switch going off.
    problem: Arc<Mutex<Option<PresenceProblem>>>,
    check_ms: Arc<AtomicU64>,
    bus: Arc<EventBus>,
    on_event: Handler,
    /// What plays, as the rule says.
    want: Option<Activity>,
    party: Option<Party>,
    /// What shows while nothing plays: the room, in one.
    idle: Option<Activity>,
    listen_until: Option<Instant>,
    /// Answers not yet sent.
    answers: Vec<(String, bool)>,
    /// `want` has not reached Discord yet.
    dirty: bool,
    last_attempt: Option<Instant>,
    /// When the connection last carried anything, for `CHECK`.
    last_sent: Option<Instant>,
    /// Whether this connection takes a poster's URL as the image.
    posters: bool,
    /// Whether this connection takes an open party.
    privacy: bool,
    /// Whether this connection takes a party at all.
    parties: bool,
}

/// A new desired state, as the worker takes it. Every field here is named where `next`
/// builds it and where `run` takes it apart, so one of these left unread does not
/// compile; a field added to `Desired` still needs adding here by hand.
struct Change {
    want: Option<Activity>,
    party: Option<Party>,
    idle: Option<Activity>,
    listen: Option<Instant>,
    answers: Vec<(String, bool)>,
    disconnect: bool,
    version: u64,
}

/// What the worker does next.
enum Step {
    /// The desired state changed: show `want`, letting the connection go
    /// first if asked.
    Apply(Box<Change>),
    /// Time to try reaching Discord again.
    Retry,
    /// The connection has been quiet for `CHECK`.
    Check,
    /// Time to read what Discord has said.
    Poll,
    Quit,
}

impl Worker {
    fn run(&mut self, mailbox: &Mailbox) {
        let mut seen = 0;
        loop {
            match self.next(mailbox, seen) {
                Step::Apply(change) => {
                    let Change {
                        want,
                        party,
                        idle,
                        listen,
                        answers,
                        disconnect,
                        version,
                    } = *change;
                    seen = version;
                    if disconnect {
                        self.disconnect("Discord presence switched off");
                        self.set_problem(None, Level::Debug, "Discord presence switched off");
                    }
                    self.want = want;
                    self.party = party;
                    self.idle = idle;
                    if listen.is_some() {
                        self.listen_until = listen;
                    }
                    self.answers.extend(answers);
                    // Nothing shows on a connection that is not there.
                    self.dirty = self.shown().is_some() || self.is_connected();
                    // With nothing to show, no Discord is no longer news: one
                    // opened since is found on the next episode, and Settings
                    // stops saying it is missing. A refusal stands until a
                    // send lands, since it says what is wrong.
                    if !self.dirty
                        && !self.listening()
                        && recover(&self.problem).as_ref() == Some(&PresenceProblem::NoDiscord)
                    {
                        self.set_problem(
                            None,
                            Level::Debug,
                            "Discord presence has nothing to show",
                        );
                    }
                    self.deliver();
                    self.answer();
                    self.poll();
                    recover(&mailbox.desired).done = version;
                    mailbox.handled.notify_all();
                }
                Step::Retry => {
                    self.deliver();
                    self.answer();
                }
                Step::Poll => self.poll(),
                Step::Check => {
                    if self.shown().is_some() || self.listening() {
                        // Sent again: a Discord that left answers Gone.
                        self.dirty = true;
                        self.deliver();
                    } else {
                        self.disconnect("Discord presence let go of an idle connection");
                    }
                }
                Step::Quit => {
                    self.disconnect("Discord presence closed");
                    recover(&mailbox.desired).stopped = true;
                    mailbox.handled.notify_all();
                    return;
                }
            }
        }
    }

    /// Waits for the desired state to move past `seen`; while there is
    /// something to show and no Discord to show it, for the next retry; and
    /// while connected with nothing new to send, for the next `CHECK`.
    fn next(&self, mailbox: &Mailbox, seen: u64) -> Step {
        let mut d = recover(&mailbox.desired);
        loop {
            if d.quit {
                return Step::Quit;
            }
            if d.version != seen {
                return Step::Apply(Box::new(Change {
                    want: d.want.clone(),
                    party: d.party.clone(),
                    idle: d.idle.clone(),
                    listen: d.listen.take(),
                    answers: std::mem::take(&mut d.answers),
                    disconnect: std::mem::take(&mut d.disconnect),
                    version: d.version,
                }));
            }
            let wants_up = (self.dirty && self.shown().is_some()) || self.listening();
            d = if wants_up && !self.is_connected() {
                let since = self.last_attempt.map_or(RETRY, |t| t.elapsed());
                let wait = RETRY.saturating_sub(since);
                if wait.is_zero() {
                    return Step::Retry;
                }
                unpoison(mailbox.changed.wait_timeout(d, wait)).0
            } else if self.is_connected() && !self.dirty {
                let check = Duration::from_millis(self.check_ms.load(Ordering::Relaxed));
                let since = self.last_sent.map_or(check, |t| t.elapsed());
                let wait = check.saturating_sub(since);
                if wait.is_zero() {
                    return Step::Check;
                }
                // Read for events between checks; the wait for a change is
                // cut short to do it.
                let (back, timeout) = unpoison(mailbox.changed.wait_timeout(d, wait.min(POLL)));
                if timeout.timed_out() && back.version == seen && !back.quit && wait > POLL {
                    return Step::Poll;
                }
                back
            } else {
                unpoison(mailbox.changed.wait(d))
            };
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    fn listening(&self) -> bool {
        self.listen_until.is_some_and(|t| Instant::now() < t)
    }

    /// What Discord should show: what plays, or the room while nothing
    /// does, either carrying the party.
    fn shown(&self) -> Option<Activity> {
        let mut a = self.want.clone().or_else(|| self.idle.clone())?;
        a.party = self.party.clone();
        Some(a)
    }

    /// Sends the answers that wait, once connected. One whose connection is
    /// gone is dropped: the ask it answered has lapsed with it.
    fn answer(&mut self) {
        if !self.is_connected() {
            if !self.listening() && self.shown().is_none() {
                self.answers.clear();
            }
            return;
        }
        for (user, invite) in std::mem::take(&mut self.answers) {
            match self.transport.answer(&user, invite) {
                Ok(()) => {}
                Err(SendError::Rejected(e)) => {
                    tracing::debug!("discord did not take the answer to a join request: {e}");
                }
                Err(SendError::Gone(e)) => {
                    tracing::debug!("discord presence lost the connection: {e}");
                    self.transport.close();
                    self.set_connected(false, Level::Info, "Discord stopped answering");
                    self.dirty = self.shown().is_some();
                    return;
                }
            }
        }
    }

    /// Reads what Discord said and hands it to the core.
    fn poll(&mut self) {
        if !self.is_connected() {
            return;
        }
        match self.transport.poll() {
            Ok(events) => {
                for event in events {
                    (self.on_event)(event);
                }
            }
            Err(e) => {
                tracing::debug!("discord presence lost the connection: {e:?}");
                self.transport.close();
                self.set_connected(false, Level::Info, "Discord stopped answering");
                self.dirty = self.shown().is_some();
            }
        }
    }

    /// Records the connection, and tells the shells when it really changed:
    /// `message` at `level` for the log, `PresenceChanged` for Settings.
    fn set_connected(&self, on: bool, level: Level, message: &str) {
        if self.connected.swap(on, Ordering::Relaxed) != on {
            self.said(level, message);
        }
    }

    /// Records why Discord shows nothing, and tells the shells when the
    /// reason really changed, so a Discord that stays away is said once and
    /// not on every retry.
    fn set_problem(&self, problem: Option<PresenceProblem>, level: Level, message: &str) -> bool {
        let changed = {
            let mut now = recover(&self.problem);
            let changed = *now != problem;
            *now = problem;
            changed
        };
        if changed {
            self.said(level, message);
        }
        changed
    }

    fn said(&self, level: Level, message: &str) {
        self.bus.emit(
            level,
            Stage::Playback,
            message,
            None,
            EventBody::PresenceChanged {
                connected: self.is_connected(),
                problem: recover(&self.problem).clone(),
            },
        );
    }

    /// Clears what Discord shows and lets the connection go: switched off,
    /// idle past `CHECK`, or shutting down. None of those is news for the
    /// activity log, so `why` goes to the developer log alone.
    fn disconnect(&mut self, why: &str) {
        if self.is_connected() {
            let _ = self.transport.send(None);
        }
        self.transport.close();
        self.set_connected(false, Level::Debug, why);
        self.want = None;
        self.listen_until = None;
        self.answers.clear();
        self.dirty = false;
        self.last_attempt = None;
    }

    /// Sends `want` if it has not landed, connecting first if need be.
    fn deliver(&mut self) {
        let listen = self.listening() && !self.is_connected();
        if !self.dirty && !listen {
            return;
        }
        if !self.is_connected() {
            if self.last_attempt.is_some_and(|t| t.elapsed() < RETRY) {
                return;
            }
            self.last_attempt = Some(Instant::now());
            match self.transport.connect(&self.client_id) {
                Ok(()) => {
                    // The reason first, so the one event that says connected
                    // carries no stale reason with it.
                    recover(&self.problem).take();
                    self.set_connected(true, Level::Info, "Discord is showing what plays");
                    self.posters = true;
                    self.privacy = true;
                    self.parties = true;
                }
                // Not debug: either leaves the profile empty, and Settings
                // and these lines are the only trace of why. Said once per
                // change, never on every retry.
                Err(SendError::Gone(e)) => {
                    let message = format!("Discord presence found no Discord: {e}");
                    if self.set_problem(Some(PresenceProblem::NoDiscord), Level::Info, &message) {
                        tracing::info!("{message}");
                    }
                    return;
                }
                Err(SendError::Rejected(e)) => {
                    let message = format!("Discord refused the connection: {e}");
                    let problem = refused(&e);
                    // Info on the bus: a Warn reads as playback failing in the
                    // plain log, and playback is fine. Settings names it.
                    if self.set_problem(Some(problem), Level::Info, &message) {
                        tracing::warn!("{message}");
                    }
                    return;
                }
            }
            if !self.dirty {
                return;
            }
        }
        let mut activity = self.shown();
        if let Some(a) = activity.as_mut() {
            if !self.posters {
                a.poster = None;
            }
            if !self.privacy
                && let Some(p) = a.party.as_mut()
            {
                p.open = false;
            }
            if !self.parties {
                a.party = None;
            }
        }
        self.last_sent = Some(Instant::now());
        let mut result = self.transport.send(activity.as_ref());
        // Discord may not take a URL as the image: once without it, and
        // never again on this connection.
        if let (Err(SendError::Rejected(e)), Some(a)) = (&result, activity.as_mut())
            && a.poster.is_some()
        {
            tracing::debug!("discord refused the poster, using the app's art: {e}");
            self.posters = false;
            a.poster = None;
            result = self.transport.send(Some(a));
            // Refused without the poster as well: the poster was not the
            // reason, so the next series tries its own again.
            if matches!(result, Err(SendError::Rejected(_))) {
                self.posters = true;
            }
        }
        // Nor an open party, which an older Discord may not know: shown as
        // one to ask, and never again on this connection.
        if let (Err(SendError::Rejected(e)), Some(a)) = (&result, activity.as_mut())
            && let Some(p) = a.party.as_mut()
            && p.open
        {
            tracing::debug!("discord refused an open party, showing it as one to ask: {e}");
            self.privacy = false;
            p.open = false;
            result = self.transport.send(Some(a));
            if matches!(result, Err(SendError::Rejected(_))) {
                self.privacy = true;
            }
        }
        // Nor a party at all, which a client without joins may not take: what plays
        // still shows, and the next connection tries the party again.
        if let (Err(SendError::Rejected(e)), Some(a)) = (&result, activity.as_mut())
            && a.party.is_some()
        {
            tracing::debug!("discord refused the party, showing the activity without it: {e}");
            a.party = None;
            result = self.transport.send(Some(a));
            if result.is_ok() {
                self.parties = false;
            }
        }
        match result {
            Ok(()) => {
                self.dirty = false;
                let message = if activity.is_some() {
                    "Discord is showing what plays"
                } else {
                    "Discord cleared what it showed"
                };
                self.set_problem(None, Level::Info, message);
            }
            Err(SendError::Rejected(e)) => {
                // Not debug: a refusal leaves the profile empty, and this
                // line and Settings are the only trace of why.
                tracing::warn!("discord refused the activity: {e}");
                self.dirty = false;
                self.set_problem(
                    Some(refused(&e)),
                    Level::Info,
                    &format!("Discord refused the activity: {e}"),
                );
            }
            Err(SendError::Gone(e)) => {
                tracing::debug!("discord presence lost the connection: {e}");
                self.transport.close();
                self.set_connected(false, Level::Info, "Discord stopped answering");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bebop() -> Activity {
        Activity::describe(
            "Cowboy Bebop",
            "EP 2",
            Some("Stray Dog Strut"),
            false,
            false,
            Some("https://img.example/bebop.jpg".to_string()),
        )
    }

    fn on() -> State {
        State {
            enabled: true,
            ..State::default()
        }
    }

    fn tick<'a>(session: u64, activity: &'a Activity, position: f64, paused: bool) -> Input<'a> {
        Input::Tick {
            session,
            activity,
            position,
            paused,
        }
    }

    fn started(a: &Activity, at: u64) -> Activity {
        Activity {
            start: Some(at),
            ..a.clone()
        }
    }

    #[test]
    fn off_gives_nothing() {
        let a = bebop();
        let mut s = State::default();
        assert!(decide(&mut s, tick(1, &a, 0.0, false), 10).is_empty());
        assert!(decide(&mut s, tick(1, &a, 1.0, true), 11).is_empty());
        assert!(decide(&mut s, Input::Close { session: 1 }, 12).is_empty());
        assert!(decide(&mut s, Input::Disable, 13).is_empty());
    }

    #[test]
    fn the_first_playing_tick_sets_once_and_repeats_send_nothing() {
        let a = bebop();
        let mut s = on();
        assert_eq!(
            decide(&mut s, tick(1, &a, 0.0, false), 100),
            vec![Action::Set(started(&a, 100))]
        );
        assert!(decide(&mut s, tick(1, &a, 1.0, false), 101).is_empty());
        assert!(decide(&mut s, tick(1, &a, 2.0, false), 102).is_empty());
    }

    #[test]
    fn a_session_that_opens_paused_shows_nothing() {
        let a = bebop();
        let mut s = on();
        assert!(decide(&mut s, tick(1, &a, 0.0, true), 100).is_empty());
    }

    #[test]
    fn a_pause_stops_the_clock_and_arms_rather_than_clears() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 100);
        let actions = decide(&mut s, tick(1, &a, 50.0, true), 150);
        let generation = s.generation;
        assert_eq!(
            actions,
            vec![
                Action::Set(a.clone()),
                Action::Arm {
                    session: 1,
                    generation
                }
            ]
        );
        assert!(!actions.contains(&Action::Clear));
        // A second paused tick is no transition.
        assert!(decide(&mut s, tick(1, &a, 50.0, true), 151).is_empty());
        // Playing again counts from the position, so the pause is not on
        // the clock.
        assert_eq!(
            decide(&mut s, tick(1, &a, 50.0, false), 200),
            vec![Action::Set(started(&a, 150))]
        );
    }

    #[test]
    fn the_timer_clears_only_when_nothing_moved_since() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 100);
        decide(&mut s, tick(1, &a, 10.0, true), 110);
        let armed = s.generation;
        assert_eq!(
            decide(
                &mut s,
                Input::Elapsed {
                    session: 1,
                    generation: armed
                },
                410
            ),
            vec![Action::Clear]
        );
        // Cleared once; the same timer again, or a paused tick, adds nothing.
        assert!(
            decide(
                &mut s,
                Input::Elapsed {
                    session: 1,
                    generation: armed
                },
                411
            )
            .is_empty()
        );
        assert!(decide(&mut s, tick(1, &a, 10.0, true), 412).is_empty());
        // Playing again shows it again, counting from the position.
        assert_eq!(
            decide(&mut s, tick(1, &a, 10.0, false), 500),
            vec![Action::Set(started(&a, 490))]
        );
    }

    #[test]
    fn a_playing_tick_before_the_timer_fires_cancels_it() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 100);
        decide(&mut s, tick(1, &a, 10.0, true), 110);
        let armed = s.generation;
        decide(&mut s, tick(1, &a, 10.0, false), 120);
        assert!(
            decide(
                &mut s,
                Input::Elapsed {
                    session: 1,
                    generation: armed
                },
                410
            )
            .is_empty()
        );
    }

    #[test]
    fn only_the_owner_closing_clears() {
        let a = bebop();
        let b = Activity {
            details: "Trigun".to_string(),
            ..bebop()
        };
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 100);
        // The next episode took over before the old close arrived.
        assert_eq!(
            decide(&mut s, tick(2, &b, 0.0, false), 200),
            vec![Action::Set(started(&b, 200))]
        );
        assert!(decide(&mut s, Input::Close { session: 1 }, 201).is_empty());
        assert_eq!(
            decide(&mut s, Input::Close { session: 2 }, 202),
            vec![Action::Clear]
        );
        // And once closed, a second close is nothing.
        assert!(decide(&mut s, Input::Close { session: 2 }, 203).is_empty());
    }

    #[test]
    fn a_close_after_a_long_pause_cleared_sends_nothing_more() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 100);
        decide(&mut s, tick(1, &a, 10.0, true), 110);
        let armed = s.generation;
        decide(
            &mut s,
            Input::Elapsed {
                session: 1,
                generation: armed,
            },
            410,
        );
        assert!(decide(&mut s, Input::Close { session: 1 }, 411).is_empty());
    }

    #[test]
    fn switching_on_shows_what_plays_and_off_disconnects() {
        let a = bebop();
        let mut s = State::default();
        assert_eq!(
            decide(&mut s, Input::Enable(Some((4, &a, 0.0))), 100),
            vec![Action::Set(started(&a, 100))]
        );
        assert!(decide(&mut s, Input::Enable(None), 101).is_empty());
        assert_eq!(
            decide(&mut s, Input::Disable, 102),
            vec![Action::Disconnect]
        );
        assert!(decide(&mut s, tick(4, &a, 3.0, false), 103).is_empty());
        assert!(decide(&mut s, Input::Enable(None), 104).is_empty());
        assert_eq!(
            decide(&mut s, tick(4, &a, 5.0, false), 105),
            vec![Action::Set(started(&a, 100))]
        );
    }

    #[test]
    fn a_resume_after_a_pause_counts_from_the_position() {
        let a = bebop();
        let mut s = on();
        // Ten minutes in, a twenty minute pause, then play on.
        decide(&mut s, tick(1, &a, 0.0, false), 10_000);
        decide(&mut s, tick(1, &a, 600.0, true), 10_600);
        assert_eq!(
            decide(&mut s, tick(1, &a, 600.0, false), 11_800),
            vec![Action::Set(started(&a, 11_800 - 600))]
        );
        // The pause is no seek to the tick after the resume.
        assert!(decide(&mut s, tick(1, &a, 601.0, false), 11_801).is_empty());
        // Switched on mid-episode, the clock reads the position too.
        let mut s = State::default();
        assert_eq!(
            decide(&mut s, Input::Enable(Some((2, &a, 90.5))), 5_000),
            vec![Action::Set(started(&a, 5_000 - 90))]
        );
    }

    #[test]
    fn a_seek_past_the_slack_moves_the_clock() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 1_000);
        assert!(decide(&mut s, tick(1, &a, 10.0, false), 1_010).is_empty());
        // Forward five minutes.
        assert_eq!(
            decide(&mut s, tick(1, &a, 311.0, false), 1_011),
            vec![Action::Set(started(&a, 700))]
        );
        assert!(decide(&mut s, tick(1, &a, 312.0, false), 1_012).is_empty());
        // And back to the start.
        assert_eq!(
            decide(&mut s, tick(1, &a, 1.0, false), 1_013),
            vec![Action::Set(started(&a, 1_012))]
        );
    }

    #[test]
    fn ordinary_ticks_and_jitter_send_nothing() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 1_000);
        for i in 1..=60 {
            assert!(
                decide(
                    &mut s,
                    tick(1, &a, f64::from(i) + 0.4, false),
                    1_000 + i as u64
                )
                .is_empty()
            );
        }
        // Ticks that come late or report a little ahead or behind, up to
        // the slack between one tick and the next.
        assert!(decide(&mut s, tick(1, &a, 75.0, false), 1_070).is_empty());
        assert!(decide(&mut s, tick(1, &a, 79.0, false), 1_080).is_empty());
        // A position that is no number is no seek, and the tick after it
        // still measures from the last real one.
        assert!(decide(&mut s, tick(1, &a, f64::NAN, false), 1_081).is_empty());
        assert!(decide(&mut s, tick(1, &a, 81.0, false), 1_082).is_empty());
    }

    /// Plays from `at` for `ticks` seconds of wall time at `speed`, and
    /// asserts that no tick sends anything.
    fn steady(s: &mut State, a: &Activity, at: u64, from: f64, speed: f64, ticks: u64) {
        for i in 1..=ticks {
            let position = from + speed * i as f64;
            assert!(
                decide(s, tick(1, a, position, false), at + i).is_empty(),
                "tick {i} at {speed}x sent"
            );
        }
    }

    #[test]
    fn a_steady_double_speed_sends_nothing() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 1_000);
        // Twelve minutes of wall time is the whole episode at 2x.
        steady(&mut s, &a, 1_000, 0.0, 2.0, 720);
    }

    #[test]
    fn a_steady_half_speed_sends_nothing() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 1_000);
        steady(&mut s, &a, 1_000, 0.0, 0.5, 2_880);
    }

    #[test]
    fn a_seek_at_1x_resends_once() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 1_000);
        steady(&mut s, &a, 1_000, 0.0, 1.0, 100);
        // Ninety seconds forward.
        assert_eq!(
            decide(&mut s, tick(1, &a, 191.0, false), 1_101),
            vec![Action::Set(started(&a, 1_101 - 191))]
        );
        steady(&mut s, &a, 1_101, 191.0, 1.0, 300);
    }

    #[test]
    fn a_seek_at_double_speed_resends_once() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, 0.0, false), 1_000);
        steady(&mut s, &a, 1_000, 0.0, 2.0, 100);
        // Ninety seconds forward, on top of the two a tick moves at 2x.
        assert_eq!(
            decide(&mut s, tick(1, &a, 292.0, false), 1_101),
            vec![Action::Set(started(&a, 1_101 - 292))]
        );
        steady(&mut s, &a, 1_101, 292.0, 2.0, 300);
    }

    #[test]
    fn long_fields_are_cut_to_discords_limit() {
        let long = "a".repeat(300);
        let a = Activity::describe(&long, "EP 1", Some(&"b".repeat(300)), false, false, None);
        assert_eq!(a.details.chars().count(), FIELD_MAX);
        assert!(a.details.ends_with("..."));
        assert_eq!(a.large_text.chars().count(), FIELD_MAX);
        assert_eq!(a.state.as_ref().unwrap().chars().count(), FIELD_MAX);
        assert!(a.state.as_ref().unwrap().starts_with("EP 1 - bbb"));
        // Characters, not bytes: a native title is not cut mid-character.
        let native = "\u{9032}".repeat(200);
        let n = Activity::describe(&native, "EP 1", None, false, false, None);
        assert_eq!(n.details.chars().count(), FIELD_MAX);
        // Short fields are left alone.
        assert_eq!(bebop().details, "Cowboy Bebop");
    }

    #[test]
    fn one_character_fields_are_padded_to_discords_minimum() {
        let a = Activity::describe("K", "E", None, false, false, None);
        assert_eq!(a.details, format!("K{FIELD_PAD}"));
        assert_eq!(a.large_text, format!("K{FIELD_PAD}"));
        assert_eq!(a.state.as_deref(), Some(format!("E{FIELD_PAD}").as_str()));
        let empty = Activity::describe("", "E1", None, false, false, None);
        let pad = FIELD_PAD.to_string().repeat(FIELD_MIN);
        let v = empty.to_json();
        assert_eq!(v["details"], pad.as_str());
        assert_eq!(v["assets"]["large_text"], pad.as_str());
        // Two characters are already enough.
        assert_eq!(v["state"], "E1");
        let two = Activity::describe("K2", "E1", None, false, false, None);
        assert_eq!(two.details, "K2");
        // Space around a title is not a character Discord counts.
        let spaced = Activity::describe(" K ", "E1", None, false, false, None);
        assert_eq!(spaced.details, format!("K{FIELD_PAD}"));
    }

    #[test]
    fn the_limit_counts_utf16_units_as_discord_does() {
        let emoji = "\u{1F600}".repeat(100);
        let a = Activity::describe(&emoji, "EP 1", None, false, false, None);
        assert!(
            a.details.encode_utf16().count() <= FIELD_MAX,
            "{}",
            a.details
        );
        assert!(a.details.ends_with("..."));
        // An emoji is never split in half.
        assert!(
            a.details
                .trim_end_matches("...")
                .chars()
                .all(|c| c == '\u{1F600}')
        );
    }

    #[test]
    fn episodes_extras_and_films_read_differently() {
        assert_eq!(bebop().state.as_deref(), Some("EP 2 - Stray Dog Strut"));
        let untitled = Activity::describe("Bebop", "S01E03", None, false, false, None);
        assert_eq!(untitled.state.as_deref(), Some("S01E03"));
        let extra = Activity::describe("Bebop", "OP1", None, true, false, None);
        assert_eq!(extra.state.as_deref(), Some("Extra"));
        assert_eq!(extra.details, "Bebop");
        let film = Activity::describe("Akira", "EP 1", None, false, true, None);
        assert_eq!(film.state, None);
        assert!(film.to_json().get("state").is_none());
    }

    #[test]
    fn no_poster_falls_back_to_the_apps_art() {
        let a = Activity::describe("Bebop", "EP 1", None, false, false, None);
        assert_eq!(a.to_json()["assets"]["large_image"], FALLBACK_ART);
        // A local path is not something Discord can fetch.
        let local = Activity::describe(
            "Bebop",
            "EP 1",
            None,
            false,
            false,
            Some("/home/me/.cache/poster.jpg".to_string()),
        );
        assert_eq!(local.poster, None);
        let v = Activity {
            start: Some(42),
            ..bebop()
        }
        .to_json();
        assert_eq!(v["assets"]["large_image"], "https://img.example/bebop.jpg");
        assert_eq!(v["assets"]["large_text"], "Cowboy Bebop");
        assert_eq!(v["details"], "Cowboy Bebop");
        assert_eq!(v["state"], "EP 2 - Stray Dog Strut");
        assert_eq!(v["timestamps"]["start"], 42);
        assert!(bebop().to_json().get("timestamps").is_none());
    }
}
