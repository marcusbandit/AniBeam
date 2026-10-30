//! Discord rich presence: with the switch on, a running Discord client shows
//! the series, the episode and the poster of what the player is playing.
//! Off by default, per machine, and silent when Discord is not there.
//!
//! The playback session feeds it. `open` builds the activity but shows
//! nothing, since a file that never plays is not being watched; the first
//! playing tick shows it, a pause shows it without the clock and arms a
//! one-shot timer that clears it if the pause outlasts `PAUSE_CLEAR`, and the
//! close of the session that owns it clears it. Only transitions send
//! anything, and `decide` is the pure rule over them. Sends go through a
//! bounded queue to one worker thread that owns the connection, so a tick
//! never waits on a socket.

pub mod discord;

use std::io;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::contract::*;
use crate::core::Core;
use crate::events::EventBus;
use crate::prefs;
use crate::sync::recover;

/// How long a pause may stand before the activity is cleared.
pub const PAUSE_CLEAR: Duration = Duration::from_secs(5 * 60);
/// The least time between two attempts to reach Discord.
pub const RETRY: Duration = Duration::from_secs(30);
/// Discord's limit on every text field of an activity, in characters.
pub const FIELD_MAX: usize = 128;
/// The art uploaded to the Discord application, shown when there is no
/// poster or Discord will not take a poster's URL.
pub const FALLBACK_ART: &str = "anibeam";
/// Room for a burst of transitions; past it a send is dropped, never waited on.
const QUEUE: usize = 16;
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
    /// Unix seconds playback started, which Discord counts up from. None
    /// while paused, so the clock stops.
    pub start: Option<u64>,
}

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
            Some("Extra".to_string())
        } else if is_film {
            None
        } else {
            Some(match episode_title.filter(|t| !t.trim().is_empty()) {
                Some(title) => format!("{code} - {title}"),
                None => code.to_string(),
            })
        };
        Activity {
            details: truncate(series_title),
            state: state.map(|s| truncate(&s)),
            poster: poster.filter(|p| p.starts_with("https://") || p.starts_with("http://")),
            large_text: truncate(series_title),
            start: None,
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
        v
    }
}

/// A field cut to Discord's limit, with an ellipsis where it was cut.
fn truncate(s: &str) -> String {
    if s.chars().count() <= FIELD_MAX {
        return s.to_string();
    }
    let mut out: String = s.chars().take(FIELD_MAX - 3).collect();
    out.push_str("...");
    out
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
    /// Opens a fresh connection, closing any that stands.
    fn connect(&mut self, client_id: &str) -> Result<(), SendError>;
    /// Sets the activity, or clears it with `None`.
    fn send(&mut self, activity: Option<&Activity>) -> Result<(), SendError>;
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
    start: u64,
    paused: bool,
    shown: bool,
    /// Bumped on every transition, so a pause timer that fires after one
    /// knows it is stale.
    generation: u64,
}

pub(crate) enum Input<'a> {
    /// The switch went on, with the session playing right now, if any.
    Enable(Option<(u64, &'a Activity)>),
    Disable,
    Tick {
        session: u64,
        activity: &'a Activity,
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
                Some((session, activity)) => play(s, session, activity, now),
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
                    play(s, session, activity, now)
                };
            }
            match (paused, s.paused) {
                (false, false) if s.shown => Vec::new(),
                (false, _) => {
                    s.paused = false;
                    s.shown = true;
                    s.generation += 1;
                    vec![Action::Set(Activity {
                        start: Some(s.start),
                        ..activity.clone()
                    })]
                }
                (true, true) => Vec::new(),
                (true, false) => {
                    s.paused = true;
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

/// A session taking the activity over, its clock starting now.
fn play(s: &mut State, session: u64, activity: &Activity, now: u64) -> Vec<Action> {
    s.owner = Some(session);
    s.start = now;
    s.paused = false;
    s.shown = true;
    s.generation += 1;
    vec![Action::Set(Activity {
        start: Some(now),
        ..activity.clone()
    })]
}

// ---------------------------------------------------------------------------
// The core's side
// ---------------------------------------------------------------------------

/// The core's handle on presence. `enabled` is the hot path's only cost
/// while the switch is off: one load per tick.
pub struct Presence {
    enabled: AtomicBool,
    connected: Arc<AtomicBool>,
    pause_clear_ms: AtomicU64,
    inner: Mutex<Inner>,
    bus: Arc<EventBus>,
}

#[derive(Default)]
struct Inner {
    state: State,
    worker: Option<SyncSender<Msg>>,
    /// A test's stand-in, taken by the worker when it starts.
    injected: Option<Box<dyn Transport>>,
    /// A stand-in was given, which makes presence available without an id.
    fake: bool,
}

enum Msg {
    Set(Activity),
    Clear,
    Disconnect,
    Flush(SyncSender<()>),
    Quit(SyncSender<()>),
}

impl Presence {
    pub fn new(bus: Arc<EventBus>) -> Presence {
        Presence {
            enabled: AtomicBool::new(false),
            connected: Arc::new(AtomicBool::new(false)),
            pause_clear_ms: AtomicU64::new(PAUSE_CLEAR.as_millis() as u64),
            inner: Mutex::new(Inner::default()),
            bus,
        }
    }

    /// Whether the switch can do anything in this build.
    pub fn available(&self) -> bool {
        bundled_client_id().is_some() || recover(&self.inner).fake
    }

    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
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

    /// For a test: returns once the worker has handled every send before it.
    pub fn flush(&self) {
        let worker = recover(&self.inner).worker.clone();
        if let Some(worker) = worker {
            let (tx, rx) = mpsc::sync_channel(1);
            if worker.send(Msg::Flush(tx)).is_ok() {
                let _ = rx.recv_timeout(FLUSH_WAIT);
            }
        }
    }
}

/// One tick of a session, after the sessions lock is let go.
pub(crate) fn tick(core: &Core, session: u64, activity: &Activity, paused: bool) {
    if !core.presence.enabled.load(Ordering::Relaxed) {
        return;
    }
    feed(
        core,
        Input::Tick {
            session,
            activity,
            paused,
        },
    );
}

pub(crate) fn close(core: &Core, session: u64) {
    if !core.presence.enabled.load(Ordering::Relaxed) {
        return;
    }
    feed(core, Input::Close { session });
}

/// The switch at launch, from the stored preference.
pub(crate) fn sync(core: &Core) {
    let on = core
        .store
        .read(prefs::load_preferences)
        .map(|p| p.discord_presence)
        .unwrap_or(false);
    set_enabled(core, on);
}

/// The switch. On shows the session playing now, if there is one; off
/// clears and lets the connection go. A build that cannot reach Discord
/// stays off whatever is asked.
pub(crate) fn set_enabled(core: &Core, on: bool) {
    let p = &core.presence;
    if on && !p.available() {
        return;
    }
    // Read before the presence lock is taken: a tick holds the sessions
    // lock and then takes this one, never the other way round.
    let playing = on.then(|| core.sessions.playing()).flatten();
    let mut inner = recover(&p.inner);
    p.enabled.store(on, Ordering::Relaxed);
    let input = match &playing {
        Some(s) => Input::Enable(Some((s.id, &s.presence))),
        None if on => Input::Enable(None),
        None => Input::Disable,
    };
    if on && inner.worker.is_none() {
        let transport = inner
            .injected
            .take()
            .unwrap_or_else(|| Box::new(discord::Discord::default()));
        let client_id = bundled_client_id().unwrap_or_default().to_string();
        inner.worker = spawn(transport, client_id, p.connected.clone(), p.bus.clone());
    }
    let actions = decide(&mut inner.state, input, crate::time::now_secs() as u64);
    run(core, &inner, actions);
}

/// Clears what Discord shows and ends the worker, waiting a moment for it.
pub(crate) fn shutdown(core: &Core) {
    let p = &core.presence;
    p.enabled.store(false, Ordering::Relaxed);
    let worker = {
        let mut inner = recover(&p.inner);
        inner.state = State::default();
        inner.worker.take()
    };
    if let Some(worker) = worker {
        let (tx, rx) = mpsc::sync_channel(1);
        if worker.try_send(Msg::Quit(tx)).is_ok() {
            let _ = rx.recv_timeout(Duration::from_secs(1));
        }
    }
}

fn feed(core: &Core, input: Input) {
    let mut inner = recover(&core.presence.inner);
    let actions = decide(&mut inner.state, input, crate::time::now_secs() as u64);
    run(core, &inner, actions);
}

/// Carries the actions out under the presence lock, so two callers' sends
/// reach the worker in the order their decisions were made. Nothing here
/// blocks: the queue is `try_send` and the timer is a spawned task.
fn run(core: &Core, inner: &Inner, actions: Vec<Action>) {
    for action in actions {
        let msg = match action {
            Action::Set(activity) => Msg::Set(activity),
            Action::Clear => Msg::Clear,
            Action::Disconnect => Msg::Disconnect,
            Action::Arm {
                session,
                generation,
            } => {
                arm(core, session, generation);
                continue;
            }
        };
        let Some(worker) = &inner.worker else {
            continue;
        };
        if let Err(TrySendError::Full(_)) = worker.try_send(msg) {
            tracing::debug!("discord presence queue full, a send was dropped");
        }
    }
}

/// The one-shot pause timer.
fn arm(core: &Core, session: u64, generation: u64) {
    let Some(owner) = core.arc() else { return };
    let weak = Arc::downgrade(&owner);
    drop(owner);
    let wait = Duration::from_millis(core.presence.pause_clear_ms.load(Ordering::Relaxed));
    core.handle.spawn(async move {
        tokio::time::sleep(wait).await;
        if let Some(core) = weak.upgrade() {
            elapsed(&core, session, generation);
        }
    });
}

/// The pause timer's handler.
pub(crate) fn elapsed(core: &Core, session: u64, generation: u64) {
    if !core.presence.enabled.load(Ordering::Relaxed) {
        return;
    }
    feed(
        core,
        Input::Elapsed {
            session,
            generation,
        },
    );
}

// ---------------------------------------------------------------------------
// The worker
// ---------------------------------------------------------------------------

fn spawn(
    transport: Box<dyn Transport>,
    client_id: String,
    connected: Arc<AtomicBool>,
    bus: Arc<EventBus>,
) -> Option<SyncSender<Msg>> {
    let (tx, rx) = mpsc::sync_channel(QUEUE);
    let mut worker = Worker {
        transport,
        client_id,
        connected,
        bus,
        want: None,
        dirty: false,
        last_attempt: None,
        posters: true,
    };
    let spawned = std::thread::Builder::new()
        .name("anibeam-discord".to_string())
        .spawn(move || worker.run(rx));
    match spawned {
        Ok(_) => Some(tx),
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
    bus: Arc<EventBus>,
    /// What Discord should show.
    want: Option<Activity>,
    /// `want` has not reached Discord yet.
    dirty: bool,
    last_attempt: Option<Instant>,
    /// Whether this connection takes a poster's URL as the image.
    posters: bool,
}

impl Worker {
    fn run(&mut self, rx: Receiver<Msg>) {
        loop {
            let msg = if self.dirty && self.want.is_some() && !self.is_connected() {
                let since = self.last_attempt.map_or(RETRY, |t| t.elapsed());
                match rx.recv_timeout(RETRY.saturating_sub(since)) {
                    Ok(msg) => Some(msg),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            } else {
                match rx.recv() {
                    Ok(msg) => Some(msg),
                    Err(_) => break,
                }
            };
            match msg {
                None => self.deliver(),
                Some(Msg::Set(activity)) => {
                    self.want = Some(activity);
                    self.dirty = true;
                    self.deliver();
                }
                Some(Msg::Clear) => {
                    self.want = None;
                    // Nothing shows on a connection that is not there.
                    self.dirty = self.is_connected();
                    self.deliver();
                }
                Some(Msg::Disconnect) => self.disconnect(),
                Some(Msg::Flush(done)) => {
                    let _ = done.send(());
                }
                Some(Msg::Quit(done)) => {
                    self.disconnect();
                    let _ = done.send(());
                    break;
                }
            }
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    fn set_connected(&self, on: bool) {
        self.connected.store(on, Ordering::Relaxed);
    }

    fn disconnect(&mut self) {
        if self.is_connected() {
            let _ = self.transport.send(None);
        }
        self.transport.close();
        self.set_connected(false);
        self.want = None;
        self.dirty = false;
        self.last_attempt = None;
    }

    /// Sends `want` if it has not landed, connecting first if need be.
    fn deliver(&mut self) {
        if !self.dirty {
            return;
        }
        if !self.is_connected() {
            if self.last_attempt.is_some_and(|t| t.elapsed() < RETRY) {
                return;
            }
            self.last_attempt = Some(Instant::now());
            match self.transport.connect(&self.client_id) {
                Ok(()) => {
                    self.set_connected(true);
                    self.posters = true;
                    self.bus.info(
                        Stage::Playback,
                        "Discord is showing what plays",
                        EventBody::Notice,
                    );
                }
                Err(e) => {
                    tracing::debug!("discord presence could not connect: {e:?}");
                    return;
                }
            }
        }
        let mut activity = self.want.clone();
        if !self.posters
            && let Some(a) = activity.as_mut()
        {
            a.poster = None;
        }
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
        }
        match result {
            Ok(()) => self.dirty = false,
            Err(SendError::Rejected(e)) => {
                tracing::debug!("discord refused the activity: {e}");
                self.dirty = false;
            }
            Err(SendError::Gone(e)) => {
                tracing::debug!("discord presence lost the connection: {e}");
                self.transport.close();
                self.set_connected(false);
                self.bus.info(
                    Stage::Playback,
                    "Discord stopped answering",
                    EventBody::Notice,
                );
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

    fn tick<'a>(session: u64, activity: &'a Activity, paused: bool) -> Input<'a> {
        Input::Tick {
            session,
            activity,
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
        assert!(decide(&mut s, tick(1, &a, false), 10).is_empty());
        assert!(decide(&mut s, tick(1, &a, true), 11).is_empty());
        assert!(decide(&mut s, Input::Close { session: 1 }, 12).is_empty());
        assert!(decide(&mut s, Input::Disable, 13).is_empty());
    }

    #[test]
    fn the_first_playing_tick_sets_once_and_repeats_send_nothing() {
        let a = bebop();
        let mut s = on();
        assert_eq!(
            decide(&mut s, tick(1, &a, false), 100),
            vec![Action::Set(started(&a, 100))]
        );
        assert!(decide(&mut s, tick(1, &a, false), 101).is_empty());
        assert!(decide(&mut s, tick(1, &a, false), 102).is_empty());
    }

    #[test]
    fn a_session_that_opens_paused_shows_nothing() {
        let a = bebop();
        let mut s = on();
        assert!(decide(&mut s, tick(1, &a, true), 100).is_empty());
    }

    #[test]
    fn a_pause_stops_the_clock_and_arms_rather_than_clears() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, false), 100);
        let actions = decide(&mut s, tick(1, &a, true), 150);
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
        assert!(decide(&mut s, tick(1, &a, true), 151).is_empty());
        // Playing again keeps the first start.
        assert_eq!(
            decide(&mut s, tick(1, &a, false), 200),
            vec![Action::Set(started(&a, 100))]
        );
    }

    #[test]
    fn the_timer_clears_only_when_nothing_moved_since() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, false), 100);
        decide(&mut s, tick(1, &a, true), 110);
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
        assert!(decide(&mut s, tick(1, &a, true), 412).is_empty());
        // Playing again shows it again.
        assert_eq!(
            decide(&mut s, tick(1, &a, false), 500),
            vec![Action::Set(started(&a, 100))]
        );
    }

    #[test]
    fn a_playing_tick_before_the_timer_fires_cancels_it() {
        let a = bebop();
        let mut s = on();
        decide(&mut s, tick(1, &a, false), 100);
        decide(&mut s, tick(1, &a, true), 110);
        let armed = s.generation;
        decide(&mut s, tick(1, &a, false), 120);
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
        decide(&mut s, tick(1, &a, false), 100);
        // The next episode took over before the old close arrived.
        assert_eq!(
            decide(&mut s, tick(2, &b, false), 200),
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
        decide(&mut s, tick(1, &a, false), 100);
        decide(&mut s, tick(1, &a, true), 110);
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
            decide(&mut s, Input::Enable(Some((4, &a))), 100),
            vec![Action::Set(started(&a, 100))]
        );
        assert!(decide(&mut s, Input::Enable(None), 101).is_empty());
        assert_eq!(
            decide(&mut s, Input::Disable, 102),
            vec![Action::Disconnect]
        );
        assert!(decide(&mut s, tick(4, &a, false), 103).is_empty());
        assert!(decide(&mut s, Input::Enable(None), 104).is_empty());
        assert_eq!(
            decide(&mut s, tick(4, &a, false), 105),
            vec![Action::Set(started(&a, 105))]
        );
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
