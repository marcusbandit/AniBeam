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

use std::io;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, LockResult, Mutex};
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
/// The fewest characters Discord takes in `details`, `state` and
/// `large_text`; a shorter one gets the whole activity refused.
pub const FIELD_MIN: usize = 2;
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
            details: fit(series_title),
            state: state.map(|s| fit(&s)),
            poster: poster.filter(|p| p.starts_with("https://") || p.starts_with("http://")),
            large_text: fit(series_title),
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

/// A text field Discord takes: trimmed, cut to `FIELD_MAX`, and padded to
/// `FIELD_MIN`. The padding is a zero-width space (U+200B): Discord trims
/// whitespace before it checks the length, so a space would not count, but
/// U+200B is not whitespace to it and shows as nothing.
fn fit(s: &str) -> String {
    let mut out = truncate(s.trim());
    while out.chars().count() < FIELD_MIN {
        out.push('\u{200B}');
    }
    out
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
    /// The clock's start as last sent: now less the position.
    start: u64,
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
                    s.start = start;
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
    s.start = start;
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
    pause_clear_ms: AtomicU64,
    inner: Mutex<Inner>,
    bus: Arc<EventBus>,
}

#[derive(Default)]
struct Inner {
    state: State,
    worker: Option<Arc<Mailbox>>,
    /// A test's stand-in, taken by the worker when it starts.
    injected: Option<Box<dyn Transport>>,
    /// A stand-in was given, which makes presence available without an id.
    fake: bool,
}

/// What the worker is to bring Discord to. A send overwrites it rather than
/// queueing behind the last, so a worker stuck on a socket comes back to
/// the newest state alone, and a clear can never be dropped.
#[derive(Default)]
struct Desired {
    /// What Discord should show; `None` clears it.
    want: Option<Activity>,
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
pub(crate) fn tick(core: &Core, session: u64, activity: &Activity, position: f64, paused: bool) {
    if !core.presence.enabled.load(Ordering::Relaxed) {
        return;
    }
    let mut inner = recover(&core.presence.inner);
    // A tick runs after the sessions lock is let go, so its session's close
    // can come in between. `close` takes the session out of the map before
    // it reaches presence, and this looks in the map under the presence
    // lock: a tick that lost that race finds its session gone and shows
    // nothing, and one that won is decided before the close, which clears.
    if !core.sessions.is_open(session) {
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
        crate::time::now_secs() as u64,
    );
    run(core, &inner, actions);
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
    // Read before the presence lock is taken: presence takes the sessions
    // lock inside its own, never the other way round.
    let playing = on.then(|| core.sessions.playing()).flatten();
    let mut inner = recover(&p.inner);
    // Checked again under the presence lock, as a tick is.
    let playing = playing.filter(|s| core.sessions.is_open(s.id));
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
        worker.post(|d| d.quit = true);
        worker.wait_while(Duration::from_secs(1), |d| !d.stopped);
    }
}

fn feed(core: &Core, input: Input) {
    let mut inner = recover(&core.presence.inner);
    let actions = decide(&mut inner.state, input, crate::time::now_secs() as u64);
    run(core, &inner, actions);
}

/// Carries the actions out under the presence lock, so two callers' sends
/// reach the worker in the order their decisions were made. Nothing here
/// blocks: a send only overwrites the desired state, and the timer is a
/// spawned task.
fn run(core: &Core, inner: &Inner, actions: Vec<Action>) {
    for action in actions {
        let change: Box<dyn FnOnce(&mut Desired)> = match action {
            Action::Set(activity) => Box::new(move |d| d.want = Some(activity)),
            Action::Clear => Box::new(|d| d.want = None),
            Action::Disconnect => Box::new(|d| {
                d.want = None;
                d.disconnect = true;
            }),
            Action::Arm {
                session,
                generation,
            } => {
                arm(core, session, generation);
                continue;
            }
        };
        if let Some(worker) = &inner.worker {
            worker.post(change);
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
) -> Option<Arc<Mailbox>> {
    let mailbox = Arc::new(Mailbox::default());
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
    bus: Arc<EventBus>,
    /// What Discord should show.
    want: Option<Activity>,
    /// `want` has not reached Discord yet.
    dirty: bool,
    last_attempt: Option<Instant>,
    /// Whether this connection takes a poster's URL as the image.
    posters: bool,
}

/// What the worker does next.
enum Step {
    /// The desired state changed: show `want`, letting the connection go
    /// first if asked.
    Apply {
        want: Option<Activity>,
        disconnect: bool,
        version: u64,
    },
    /// Time to try reaching Discord again.
    Retry,
    Quit,
}

impl Worker {
    fn run(&mut self, mailbox: &Mailbox) {
        let mut seen = 0;
        loop {
            match self.next(mailbox, seen) {
                Step::Apply {
                    want,
                    disconnect,
                    version,
                } => {
                    seen = version;
                    if disconnect {
                        self.disconnect();
                    }
                    // Nothing shows on a connection that is not there.
                    self.dirty = want.is_some() || self.is_connected();
                    self.want = want;
                    self.deliver();
                    recover(&mailbox.desired).done = version;
                    mailbox.handled.notify_all();
                }
                Step::Retry => self.deliver(),
                Step::Quit => {
                    self.disconnect();
                    recover(&mailbox.desired).stopped = true;
                    mailbox.handled.notify_all();
                    return;
                }
            }
        }
    }

    /// Waits for the desired state to move past `seen`, or, while there is
    /// something to show and no Discord to show it, for the next retry.
    fn next(&self, mailbox: &Mailbox, seen: u64) -> Step {
        let mut d = recover(&mailbox.desired);
        loop {
            if d.quit {
                return Step::Quit;
            }
            if d.version != seen {
                return Step::Apply {
                    want: d.want.clone(),
                    disconnect: std::mem::take(&mut d.disconnect),
                    version: d.version,
                };
            }
            d = if self.dirty && self.want.is_some() && !self.is_connected() {
                let since = self.last_attempt.map_or(RETRY, |t| t.elapsed());
                let wait = RETRY.saturating_sub(since);
                if wait.is_zero() {
                    return Step::Retry;
                }
                unpoison(mailbox.changed.wait_timeout(d, wait)).0
            } else {
                unpoison(mailbox.changed.wait(d))
            };
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
        assert_eq!(s.start, 1_000);
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
    fn one_letter_fields_are_padded_to_discords_minimum() {
        let k = Activity::describe("K", "1", None, false, false, None);
        assert_eq!(k.details, "K\u{200B}");
        assert_eq!(k.large_text, "K\u{200B}");
        assert_eq!(k.state.as_deref(), Some("1\u{200B}"));
        // Whitespace is trimmed first, since Discord would not count it.
        let spaced = Activity::describe(" K ", "EP 1", None, false, false, None);
        assert_eq!(spaced.details, "K\u{200B}");
        let empty = Activity::describe("", "EP 1", None, false, false, None);
        assert_eq!(empty.details.chars().count(), FIELD_MIN);
        // Two characters are enough as they are.
        assert_eq!(Activity::describe("K2", "EP 1", None, false, false, None).details, "K2");
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
