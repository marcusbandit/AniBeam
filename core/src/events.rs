use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::time::{Duration, Instant};

use rusqlite::params;

use crate::contract::*;
use crate::store::Store;
use crate::time;

/// The size of the events ring: the store keeps this many of the newest
/// Info-and-above rows, oldest dropped first.
pub const RING_SIZE: u64 = 2000;

/// Every call and every job emits through this. Persistence is
/// fire-and-forget (`Store::post`), so `emit` never blocks on a database
/// round trip; listeners are called inline, by value, on the emitting
/// thread.
pub struct EventBus {
    seq: AtomicU64,
    /// Bumped by every event that can stand for a change of state, before
    /// any listener hears it. A shell's read cache stamps what it keeps
    /// with this and drops it the moment the number moves.
    generation: AtomicU64,
    next_listener: AtomicU64,
    listeners: RwLock<Vec<(u64, Arc<dyn EventListener>)>>,
    store: Arc<Store>,
    /// Laid over the cards of every `SeriesChanged` before anyone hears it:
    /// the live state a card read from the database cannot know (what a
    /// torrent is still writing), whichever job or scan built the cards.
    cards_hook: RwLock<Option<CardsHook>>,
}

pub type CardsHook = Arc<dyn Fn(&mut [SeriesCard]) + Send + Sync>;

impl EventBus {
    /// Reads the ring's highest `seq` so numbering continues across
    /// launches instead of restarting at zero.
    pub fn new(store: Arc<Store>) -> Result<Arc<EventBus>, CoreError> {
        let last: i64 = store.write(|c| {
            Ok(c.query_row("SELECT coalesce(max(seq), 0) FROM events", [], |r| r.get(0))?)
        })?;
        scrub_stored(&store)?;
        Ok(Arc::new(EventBus {
            seq: AtomicU64::new(last as u64),
            generation: AtomicU64::new(0),
            next_listener: AtomicU64::new(1),
            listeners: RwLock::new(Vec::new()),
            store,
            cards_hook: RwLock::new(None),
        }))
    }

    /// Sets the hook every `SeriesChanged` passes through.
    pub fn set_cards_hook(&self, hook: CardsHook) {
        *self.cards_hook.write().unwrap() = Some(hook);
    }

    /// Stamps `seq` and `at`, persists Info and above to the ring, then
    /// calls every listener with the finished event. A line emitted here has
    /// no plain wording of its own: at Info and below only the developer log
    /// shows it, and a warning or an error gets the general line for its
    /// stage, so a problem never goes unsaid. `emit_said` takes the wording,
    /// and with `None` keeps even a problem to the developer log.
    pub fn emit(
        &self,
        level: Level,
        stage: Stage,
        message: impl Into<String>,
        job: Option<JobRef>,
        body: EventBody,
    ) -> Event {
        let plain = (level >= Level::Warn).then(|| stage_problem(stage));
        self.emit_said(level, stage, plain, message, job, body)
    }

    /// `emit` with the line a person reads in the plain activity log beside
    /// the developer's `message`.
    pub fn emit_said(
        &self,
        level: Level,
        stage: Stage,
        plain: Option<String>,
        message: impl Into<String>,
        job: Option<JobRef>,
        mut body: EventBody,
    ) -> Event {
        if let EventBody::SeriesChanged { series } = &mut body {
            let hook = self.cards_hook.read().unwrap().clone();
            if let Some(hook) = hook {
                hook(series);
            }
        }
        let event = Event {
            seq: self.seq.fetch_add(1, Ordering::SeqCst) + 1,
            at: time::now(),
            level,
            stage,
            message: message.into(),
            plain,
            job,
            body,
        };
        // Before the listeners, so a shell that rereads on hearing the
        // event never finds its cache still stamped current. Progress is
        // never a change of state on its own: every write behind it is
        // followed by its own event.
        // A torrent read is not one either: it is the client's list as polled every few
        // seconds, and a series whose downloading changed is announced by its own
        // SeriesChanged (downloads::list_torrents). Moving the generation on each read
        // emptied the shell's read cache and the kept snapshot every few seconds while a
        // download ran, so no page ever found its library read cached (#289).
        // Nor is a job starting: what it goes on to change is announced by its own events.
        // Nor Discord coming and going: no read the shell caches depends on it.
        if !matches!(
            event.body,
            EventBody::JobProgress { .. }
                | EventBody::JobStarted { .. }
                | EventBody::TorrentsRead { .. }
                | EventBody::PresenceChanged { .. }
        ) {
            self.generation.fetch_add(1, Ordering::SeqCst);
        }
        if level != Level::Debug {
            self.persist(&event);
        }
        // Snapshot the listener list and drop the lock before calling out,
        // so a listener that turns around and subscribes or unsubscribes
        // never deadlocks on `listeners`.
        let listeners: Vec<Arc<dyn EventListener>> = self
            .listeners
            .read()
            .unwrap()
            .iter()
            .map(|(_, l)| l.clone())
            .collect();
        for l in listeners {
            l.on_event(event.clone());
        }
        event
    }

    pub fn debug(&self, stage: Stage, message: impl Into<String>, body: EventBody) -> Event {
        self.emit(Level::Debug, stage, message, None, body)
    }
    pub fn info(&self, stage: Stage, message: impl Into<String>, body: EventBody) -> Event {
        self.emit(Level::Info, stage, message, None, body)
    }
    pub fn warn(&self, stage: Stage, message: impl Into<String>, body: EventBody) -> Event {
        self.emit(Level::Warn, stage, message, None, body)
    }
    pub fn error(&self, stage: Stage, message: impl Into<String>, body: EventBody) -> Event {
        self.emit(Level::Error, stage, message, None, body)
    }
    /// A line for both logs: `plain` for the person, `message` for the
    /// developer view. The two may say the same thing when the developer
    /// line is already plain.
    pub fn say(
        &self,
        level: Level,
        stage: Stage,
        plain: impl Into<String>,
        message: impl Into<String>,
        body: EventBody,
    ) -> Event {
        self.emit_said(level, stage, Some(plain.into()), message, None, body)
    }

    /// Fire-and-forget: queued on the writer thread with no reply, so an
    /// `emit` from inside a `write` or `tx` closure never waits on itself.
    /// A failed insert is logged and dropped; it never propagates.
    ///
    /// The stored copy keeps no secret, whichever event carries one (#509): a feed's
    /// or a tracker's address can hold a login and a key, so the message, the plain
    /// line and the body lose both (`net::scrub_secrets`). Listeners hear the event
    /// whole, since a release's link has to reach the client as it is.
    fn persist(&self, event: &Event) {
        let mut e = event.clone();
        e.message = crate::net::scrub_secrets(&e.message);
        e.plain = e.plain.map(|p| crate::net::scrub_secrets(&p));
        self.store.post(move |c| {
            let (job_id, job_kind, job_phase) = match &e.job {
                Some(j) => (Some(j.id as i64), Some(j.kind.as_str()), Some(j.phase.as_str())),
                None => (None, None, None),
            };
            let body = match serde_json::to_value(&e.body).map(scrub_value) {
                Ok(body) => body.to_string(),
                Err(err) => {
                    tracing::warn!("failed to serialise event body for seq {}: {err}", e.seq);
                    return;
                }
            };
            if let Err(err) = c.execute(
                "INSERT INTO events (seq, at, level, stage, message, job_id, job_kind, job_phase, body, plain) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![e.seq as i64, time::to_secs(e.at), e.level.as_str(), e.stage.as_str(), e.message, job_id, job_kind, job_phase, body, e.plain],
            ) {
                tracing::warn!("failed to insert event seq {}: {err}", e.seq);
                return;
            }
            if let Err(err) = c.execute(
                "DELETE FROM events WHERE seq <= (SELECT seq FROM events ORDER BY seq DESC LIMIT 1 OFFSET ?1)",
                params![RING_SIZE as i64],
            ) {
                tracing::warn!("failed to trim the events ring: {err}");
            }
        });
    }

    /// The state generation: moves on every event but a job's progress.
    /// Two reads under the same number saw the same state, up to the time
    /// of day.
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    pub fn subscribe(&self, listener: Arc<dyn EventListener>) -> u64 {
        let id = self.next_listener.fetch_add(1, Ordering::SeqCst);
        self.listeners.write().unwrap().push((id, listener));
        id
    }

    pub fn unsubscribe(&self, id: u64) {
        self.listeners.write().unwrap().retain(|(i, _)| *i != id);
    }

    /// Oldest first. A row whose level, stage, job kind, job phase or body
    /// does not parse is skipped, with a warning naming its `seq`, rather
    /// than repaired into a default; a genuine rusqlite error still
    /// propagates.
    pub fn recent(&self, limit: u64) -> Result<Vec<Event>, CoreError> {
        // The ring never holds more than this, so a shell asking for a
        // billion asks for the ring; the cast below is only safe because
        // of it.
        let limit = limit.min(RING_SIZE);
        self.store.write(move |c| {
            let mut stmt = c.prepare_cached(
                "SELECT seq, at, level, stage, message, job_id, job_kind, job_phase, body, plain FROM events ORDER BY seq DESC LIMIT ?1",
            )?;
            let mut rows: Vec<Event> = stmt
                .query_map(params![limit as i64], |r| {
                    let seq: i64 = r.get(0)?;
                    let at: i64 = r.get(1)?;
                    let level_col: String = r.get(2)?;
                    let stage_col: String = r.get(3)?;
                    let message: String = r.get(4)?;
                    let job_id: Option<i64> = r.get(5)?;
                    let job_kind_col: Option<String> = r.get(6)?;
                    let job_phase_col: Option<String> = r.get(7)?;
                    let body_col: String = r.get(8)?;
                    let plain: Option<String> = r.get(9)?;

                    let Some(level) = Level::from_column(&level_col) else {
                        tracing::warn!("skipping event seq {seq}: unknown level {level_col:?}");
                        return Ok(None);
                    };
                    let Some(stage) = Stage::from_column(&stage_col) else {
                        tracing::warn!("skipping event seq {seq}: unknown stage {stage_col:?}");
                        return Ok(None);
                    };
                    let job = match (job_id, job_kind_col, job_phase_col) {
                        (Some(id), Some(kind_col), Some(phase_col)) => {
                            let Some(kind) = JobKind::from_column(&kind_col) else {
                                tracing::warn!("skipping event seq {seq}: unknown job kind {kind_col:?}");
                                return Ok(None);
                            };
                            let Some(phase) = JobPhase::from_column(&phase_col) else {
                                tracing::warn!("skipping event seq {seq}: unknown job phase {phase_col:?}");
                                return Ok(None);
                            };
                            Some(JobRef { id: id as u64, kind, phase })
                        }
                        _ => None,
                    };
                    let Ok(body) = serde_json::from_str(&body_col) else {
                        tracing::warn!("skipping event seq {seq}: body did not deserialise");
                        return Ok(None);
                    };

                    Ok(Some(Event { seq: seq as u64, at: time::from_secs(at), level, stage, message, plain, job, body }))
                })?
                .collect::<Result<Vec<Option<Event>>, _>>()?
                .into_iter()
                .flatten()
                .collect();
            rows.reverse();
            Ok(rows)
        })
    }

    pub fn clear(&self) -> Result<(), CoreError> {
        self.store.write(|c| {
            c.execute("DELETE FROM events", [])?;
            Ok(())
        })
    }
}

/// The plain line for a warning or an error whose site has no wording of its
/// own: which part of the app it touched, and where the details are.
pub fn stage_problem(stage: Stage) -> String {
    let area = match stage {
        Stage::Library => "your library",
        Stage::Metadata => "show info",
        Stage::Trackers => "your lists",
        Stage::Franchise => "related shows",
        Stage::Playback => "playback",
        Stage::Store => "saving your data",
        Stage::System => "AniBeam",
        Stage::Downloads => "downloads",
    };
    format!("Something went wrong with {area}. Developer options have the details.")
}

/// Handed back by `Core::subscribe`; dropping it or calling `unsubscribe`
/// removes the listener.
#[derive(uniffi::Object)]
pub struct Subscription {
    bus: Arc<EventBus>,
    id: u64,
    active: AtomicBool,
}

impl Subscription {
    pub fn new(bus: Arc<EventBus>, listener: Arc<dyn EventListener>) -> Arc<Subscription> {
        let id = bus.subscribe(listener);
        Arc::new(Subscription {
            bus,
            id,
            active: AtomicBool::new(true),
        })
    }
}

#[uniffi::export]
impl Subscription {
    pub fn unsubscribe(&self) {
        if self.active.swap(false, Ordering::SeqCst) {
            self.bus.unsubscribe(self.id);
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

/// A listener that keeps everything it saw. Tests and the CLI's `--wait`
/// use it.
#[doc(hidden)]
#[derive(Default)]
pub struct Collector {
    events: Mutex<Vec<Event>>,
    condvar: Condvar,
}

impl Collector {
    pub fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }

    pub fn bodies(&self) -> Vec<EventBody> {
        self.events().into_iter().map(|e| e.body).collect()
    }

    /// Blocks until `pred` holds over the events seen so far, or `timeout`
    /// elapses. Built on a condvar `on_event` notifies, so callers never
    /// sleep-poll.
    pub fn wait_for(&self, pred: impl Fn(&[Event]) -> bool, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut guard = self.events.lock().unwrap();
        loop {
            if pred(&guard) {
                return true;
            }
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return false;
            };
            let (next, result) = self.condvar.wait_timeout(guard, remaining).unwrap();
            guard = next;
            if result.timed_out() {
                return pred(&guard);
            }
        }
    }
}

impl EventListener for Collector {
    fn on_event(&self, event: Event) {
        self.events.lock().unwrap().push(event);
        self.condvar.notify_all();
    }
}

/// Every string in a body, scrubbed one by one, so the JSON stays whole whatever a
/// key's value holds.
fn scrub_value(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::String(s) => Value::String(crate::net::scrub_secrets(&s)),
        Value::Array(items) => Value::Array(items.into_iter().map(scrub_value).collect()),
        Value::Object(map) => {
            Value::Object(map.into_iter().map(|(k, v)| (k, scrub_value(v))).collect())
        }
        other => other,
    }
}

/// The rows stored before every event was scrubbed, scrubbed as the bus starts (#509),
/// as 0005 dropped the rows of an earlier leak. Cheap, since the ring holds 2000 rows
/// at most, and a row with nothing to scrub is left as it is.
fn scrub_stored(store: &Store) -> Result<(), CoreError> {
    store.write(|c| {
        let rows: Vec<(i64, String, Option<String>, String)> = {
            let mut stmt = c.prepare("SELECT seq, message, plain, body FROM events")?;
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
                .collect::<Result<_, _>>()?
        };
        for (seq, message, plain, body) in rows {
            let new_message = crate::net::scrub_secrets(&message);
            let new_plain = plain.as_deref().map(crate::net::scrub_secrets);
            let new_body = serde_json::from_str(&body)
                .map(|v| scrub_value(v).to_string())
                .unwrap_or_else(|_| crate::net::scrub_secrets(&body));
            if new_message != message || new_plain != plain || new_body != body {
                c.execute(
                    "UPDATE events SET message = ?2, plain = ?3, body = ?4 WHERE seq = ?1",
                    params![seq, new_message, new_plain, new_body],
                )?;
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn bus() -> (tempfile::TempDir, Arc<EventBus>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("anibeam.db")).unwrap();
        (dir, EventBus::new(store).unwrap())
    }

    /// Every column of every stored row, as the database holds it.
    fn stored_text(bus: &EventBus) -> String {
        bus.store
            .write(|c| {
                let mut stmt = c.prepare(
                    "SELECT message, coalesce(plain, ''), body FROM events ORDER BY seq",
                )?;
                let rows = stmt.query_map([], |r| {
                    Ok(format!(
                        "{} | {} | {}",
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?
                    ))
                })?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?.join("\n"))
            })
            .unwrap()
    }

    #[test]
    fn no_stored_event_keeps_a_feeds_login_or_key() {
        // One feed with a login and a key, and a tracker with a passkey inside a
        // magnet, through every Info event that can carry an address (#509).
        const FEED: &str = "https://alice:hunter2@feed.example/rss?q=show&apikey=SEKRIT1&page=2";
        const MAGNET: &str = "magnet:?xt=urn:btih:abc&tr=https%3A%2F%2Fbob%3Apw9%40t.example%2Fannounce%3Fpasskey%3DSEKRIT2%26x%3D1";
        let (_d, bus) = bus();
        let heard = Arc::new(Heard(Mutex::new(Vec::new())));
        bus.subscribe(heard.clone());
        let spec = SubscriptionSpec {
            feed_url: FEED.into(),
            must_contain: String::new(),
            must_not_contain: String::new(),
            endpoint_name: "feed".into(),
        };
        let subscribed = Subscribed {
            name: "Show".into(),
            feed_url: FEED.into(),
            save_path: "/lib/Show".into(),
            must_contain: String::new(),
            must_not_contain: String::new(),
            tags: vec![],
        };
        let rule: SubscriptionRule = serde_json::from_value(serde_json::json!({
            "name": "Show", "feed_url": FEED, "save_path": "/lib/Show",
        }))
        .unwrap();
        let view: RuleView = serde_json::from_value(serde_json::json!({
            "name": "Show", "enabled": true, "must_contain": "", "must_not_contain": "",
            "save_path": "/lib/Show", "feeds": [FEED], "last_match": "", "matching": [],
        }))
        .unwrap();
        let bodies = vec![
            EventBody::SubscriptionPreviewed { spec },
            EventBody::Subscribed { result: subscribed },
            EventBody::SubscriptionsRead { rules: vec![rule] },
            EventBody::RuleRead { rule: view },
            EventBody::ReleaseInspected {
                link: MAGNET.into(),
                episodes: vec![],
                files: 0,
            },
            EventBody::DownloadsUnavailable {
                what: "feed".into(),
                error: CoreError::internal(format!("could not read {FEED}")),
            },
        ];
        for body in bodies {
            bus.say(
                Level::Info,
                Stage::Downloads,
                format!("Read {FEED} and {MAGNET}"),
                format!("read {FEED} via {MAGNET}"),
                body,
            );
        }
        let stored = stored_text(&bus);
        for secret in ["alice", "hunter2", "SEKRIT1", "bob", "pw9", "SEKRIT2"] {
            assert!(!stored.contains(secret), "{secret} is stored:\n{stored}");
        }
        // The rest of the address stays, so the line still says which feed it was.
        assert!(
            stored.contains("https://feed.example/rss?q=show&apikey=...&page=2"),
            "{stored}"
        );
        assert!(
            stored.contains("https%3A%2F%2Ft.example%2Fannounce%3Fpasskey%3D...%26x%3D1"),
            "{stored}"
        );
        // The stored bodies still read back, and the listeners' copy was whole.
        assert_eq!(bus.recent(100).unwrap().len(), 6);
        let heard = heard.0.lock().unwrap().join("\n");
        assert!(heard.contains(FEED) && heard.contains(MAGNET), "{heard}");
    }

    /// What a listener heard, each event as its debug text.
    struct Heard(Mutex<Vec<String>>);

    impl EventListener for Heard {
        fn on_event(&self, event: Event) {
            self.0.lock().unwrap().push(format!("{event:?}"));
        }
    }

    #[test]
    fn a_key_whose_value_json_escapes_is_stored_without_it() {
        let (_d, bus) = bus();
        bus.info(
            Stage::Downloads,
            "read",
            EventBody::ReleaseInspected {
                link: r#"https://h/?apikey=ab\cd\ef&x=1"#.into(),
                episodes: vec![],
                files: 0,
            },
        );
        let stored = stored_text(&bus);
        assert!(!stored.contains("cd") && !stored.contains("ef"), "{stored}");
        assert_eq!(bus.recent(10).unwrap().len(), 1);
    }

    #[test]
    fn rows_stored_before_the_scrub_lose_their_keys_on_the_next_start() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("anibeam.db");
        {
            let store = Store::open(&path).unwrap();
            store
                .write(|c| {
                    c.execute(
                        "INSERT INTO events (seq, at, level, stage, message, body, plain) VALUES (1, 0, 'info', 'downloads', 'read https://u:pw@h/?passkey=SEKRIT', '\"Notice\"', 'read https://u:pw@h/?passkey=SEKRIT')",
                        [],
                    )?;
                    Ok(())
                })
                .unwrap();
            store.close();
        }
        let bus = EventBus::new(Store::open(&path).unwrap()).unwrap();
        let stored = stored_text(&bus);
        assert!(
            !stored.contains("SEKRIT") && !stored.contains("pw@"),
            "{stored}"
        );
        assert!(stored.contains("https://h/?passkey=..."), "{stored}");
    }

    #[test]
    fn info_persists_debug_does_not() {
        let (_d, bus) = bus();
        bus.info(Stage::System, "hello", EventBody::Notice);
        bus.debug(
            Stage::Library,
            "progress",
            EventBody::JobProgress {
                done: 1,
                total: None,
                label: "".into(),
            },
        );
        let recent = bus.recent(100).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].message, "hello");
        assert_eq!(recent[0].level, Level::Info);
    }

    #[test]
    fn a_torrent_read_leaves_the_generation_where_it_was() {
        let (_d, bus) = bus();
        let g0 = bus.generation();
        bus.debug(
            Stage::Downloads,
            "3 torrents read",
            EventBody::TorrentsRead {
                torrents: vec![],
                transfer: Default::default(),
                shows: vec![],
            },
        );
        assert_eq!(bus.generation(), g0);
    }

    #[test]
    fn a_problem_without_wording_still_reaches_the_plain_log() {
        let (_d, bus) = bus();
        let quiet = bus.info(Stage::Library, "scan finished", EventBody::Notice);
        let loud = bus.warn(Stage::Library, "watcher error: too many", EventBody::Notice);
        let kept = bus.emit_said(
            Level::Warn,
            Stage::Library,
            None,
            "chore",
            None,
            EventBody::Notice,
        );
        assert_eq!(quiet.plain, None);
        assert_eq!(
            loud.plain.as_deref(),
            Some(stage_problem(Stage::Library).as_str())
        );
        assert_eq!(
            kept.plain, None,
            "emit_said with None keeps it to the developer log"
        );
        let stored = bus.recent(10).unwrap();
        assert_eq!(
            stored[1].plain, loud.plain,
            "the plain line is kept in the ring"
        );
    }

    /// What a listener saw the generation as, while it was being told.
    struct GenerationProbe {
        bus: std::sync::Weak<EventBus>,
        seen: Mutex<Vec<u64>>,
    }

    impl EventListener for GenerationProbe {
        fn on_event(&self, _event: Event) {
            if let Some(bus) = self.bus.upgrade() {
                self.seen.lock().unwrap().push(bus.generation());
            }
        }
    }

    #[test]
    fn the_generation_moves_on_every_change_before_any_listener_hears_it() {
        let (_d, bus) = bus();
        let probe = Arc::new(GenerationProbe {
            bus: Arc::downgrade(&bus),
            seen: Mutex::new(vec![]),
        });
        bus.subscribe(probe.clone());
        let g0 = bus.generation();
        bus.info(
            Stage::Library,
            "changed",
            EventBody::SeriesChanged { series: vec![] },
        );
        let g1 = bus.generation();
        assert!(g1 > g0, "a change moves the generation");
        assert_eq!(
            probe.seen.lock().unwrap().as_slice(),
            &[g1],
            "the listener already sees the moved generation"
        );
        bus.debug(
            Stage::Library,
            "progress",
            EventBody::JobProgress {
                done: 1,
                total: None,
                label: "".into(),
            },
        );
        assert_eq!(bus.generation(), g1, "progress alone is not a change");
        bus.debug(Stage::System, "a debug line", EventBody::Notice);
        assert!(bus.generation() > g1, "a debug-level event still counts");
    }

    #[test]
    fn listeners_receive_every_event_and_stop_after_unsubscribe() {
        let (_d, bus) = bus();
        let c = Arc::new(Collector::default());
        let id = bus.subscribe(c.clone());
        bus.debug(Stage::System, "a", EventBody::Notice);
        bus.unsubscribe(id);
        bus.debug(Stage::System, "b", EventBody::Notice);
        let seen = c.events();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].message, "a");
    }

    #[test]
    fn seq_is_monotonic_and_continues_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("anibeam.db");
        let first = {
            let store = Store::open(&path).unwrap();
            let bus = EventBus::new(store.clone()).unwrap();
            let a = bus.info(Stage::System, "a", EventBody::Notice);
            let b = bus.info(Stage::System, "b", EventBody::Notice);
            assert!(b.seq > a.seq);
            store.close();
            b.seq
        };
        let store = Store::open(&path).unwrap();
        let bus = EventBus::new(store).unwrap();
        let c = bus.info(Stage::System, "c", EventBody::Notice);
        assert!(c.seq > first);
    }

    #[test]
    fn the_ring_keeps_the_last_2000_oldest_first() {
        let (_d, bus) = bus();
        for i in 0..2010 {
            bus.info(Stage::System, format!("m{i}"), EventBody::Notice);
        }
        let all = bus.recent(5000).unwrap();
        assert_eq!(all.len(), 2000);
        assert_eq!(all[0].message, "m10");
        assert_eq!(all[1999].message, "m2009");
        let last = bus.recent(2).unwrap();
        assert_eq!(last[0].message, "m2008");
        bus.clear().unwrap();
        assert!(bus.recent(10).unwrap().is_empty());
    }

    #[test]
    fn dropping_a_subscription_unsubscribes() {
        let (_d, bus) = bus();
        let c = Arc::new(Collector::default());
        {
            let _sub = Subscription::new(bus.clone(), c.clone());
            bus.debug(Stage::System, "a", EventBody::Notice);
        }
        bus.debug(Stage::System, "b", EventBody::Notice);
        assert_eq!(c.events().len(), 1);
    }

    #[test]
    fn wait_for_returns_true_when_an_event_arrives_from_another_thread() {
        let (_d, bus) = bus();
        let c = Arc::new(Collector::default());
        bus.subscribe(c.clone());
        let emitter = bus.clone();
        let handle = std::thread::spawn(move || {
            emitter.info(Stage::System, "from another thread", EventBody::Notice);
        });
        let arrived = c.wait_for(
            |events| events.iter().any(|e| e.message == "from another thread"),
            Duration::from_secs(2),
        );
        handle.join().unwrap();
        assert!(arrived);
    }

    #[test]
    fn wait_for_returns_false_on_timeout() {
        let (_d, bus) = bus();
        let c = Arc::new(Collector::default());
        bus.subscribe(c.clone());
        let arrived = c.wait_for(|_| false, Duration::from_millis(50));
        assert!(!arrived);
    }

    #[test]
    fn a_corrupt_body_row_is_skipped_and_good_rows_still_come_back() {
        let (_d, bus) = bus();
        let good = bus.info(Stage::System, "good", EventBody::Notice);
        // Seq 0 never collides with the bus's own counter, which starts
        // handing out 1 and up, so this sits in the table without a
        // primary key clash.
        bus.store
            .write(|c| {
                c.execute(
                    "INSERT INTO events (seq, at, level, stage, message, job_id, job_kind, job_phase, body) VALUES (0, 0, 'info', 'system', 'corrupt', NULL, NULL, NULL, 'not json')",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        let also_good = bus.info(Stage::System, "also good", EventBody::Notice);
        assert!(also_good.seq > good.seq);
        let recent = bus.recent(100).unwrap();
        let messages: Vec<&str> = recent.iter().map(|e| e.message.as_str()).collect();
        assert_eq!(messages, vec!["good", "also good"]);
    }
}
