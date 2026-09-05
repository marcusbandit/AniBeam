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
    next_listener: AtomicU64,
    listeners: RwLock<Vec<(u64, Arc<dyn EventListener>)>>,
    store: Arc<Store>,
}

impl EventBus {
    /// Reads the ring's highest `seq` so numbering continues across
    /// launches instead of restarting at zero.
    pub fn new(store: Arc<Store>) -> Result<Arc<EventBus>, CoreError> {
        let last: i64 = store.write(|c| Ok(c.query_row("SELECT coalesce(max(seq), 0) FROM events", [], |r| r.get(0))?))?;
        Ok(Arc::new(EventBus {
            seq: AtomicU64::new(last as u64),
            next_listener: AtomicU64::new(1),
            listeners: RwLock::new(Vec::new()),
            store,
        }))
    }

    /// Stamps `seq` and `at`, persists Info and above to the ring, then
    /// calls every listener with the finished event.
    pub fn emit(&self, level: Level, stage: Stage, message: impl Into<String>, job: Option<JobRef>, body: EventBody) -> Event {
        let event = Event {
            seq: self.seq.fetch_add(1, Ordering::SeqCst) + 1,
            at: time::now(),
            level,
            stage,
            message: message.into(),
            job,
            body,
        };
        if level != Level::Debug {
            self.persist(&event);
        }
        // Snapshot the listener list and drop the lock before calling out,
        // so a listener that turns around and subscribes or unsubscribes
        // never deadlocks on `listeners`.
        let listeners: Vec<Arc<dyn EventListener>> = self.listeners.read().unwrap().iter().map(|(_, l)| l.clone()).collect();
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

    /// Fire-and-forget: queued on the writer thread with no reply, so an
    /// `emit` from inside a `write` or `tx` closure never waits on itself.
    /// A failed insert is logged and dropped; it never propagates.
    fn persist(&self, event: &Event) {
        let e = event.clone();
        self.store.post(move |c| {
            let (job_id, job_kind, job_phase) = match &e.job {
                Some(j) => (Some(j.id as i64), Some(j.kind.as_str()), Some(j.phase.as_str())),
                None => (None, None, None),
            };
            let body = match serde_json::to_string(&e.body) {
                Ok(body) => body,
                Err(err) => {
                    tracing::warn!("failed to serialise event body for seq {}: {err}", e.seq);
                    return;
                }
            };
            if let Err(err) = c.execute(
                "INSERT INTO events (seq, at, level, stage, message, job_id, job_kind, job_phase, body) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![e.seq as i64, time::to_secs(e.at), e.level.as_str(), e.stage.as_str(), e.message, job_id, job_kind, job_phase, body],
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
        self.store.write(move |c| {
            let mut stmt = c.prepare_cached(
                "SELECT seq, at, level, stage, message, job_id, job_kind, job_phase, body FROM events ORDER BY seq DESC LIMIT ?1",
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

                    Ok(Some(Event { seq: seq as u64, at: time::from_secs(at), level, stage, message, job, body }))
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
        Arc::new(Subscription { bus, id, active: AtomicBool::new(true) })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn bus() -> (tempfile::TempDir, Arc<EventBus>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("anibeam.db")).unwrap();
        (dir, EventBus::new(store).unwrap())
    }

    #[test]
    fn info_persists_debug_does_not() {
        let (_d, bus) = bus();
        bus.info(Stage::System, "hello", EventBody::Notice);
        bus.debug(Stage::Library, "progress", EventBody::JobProgress { done: 1, total: None, label: "".into() });
        let recent = bus.recent(100).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].message, "hello");
        assert_eq!(recent[0].level, Level::Info);
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
        let arrived = c.wait_for(|events| events.iter().any(|e| e.message == "from another thread"), Duration::from_secs(2));
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
