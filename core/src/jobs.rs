use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;

use crate::contract::*;
use crate::events::EventBus;
use crate::time;

/// The throttle window for both `JobProgress` and batched `SeriesChanged`:
/// at most four events a second per job, latest value always flushed.
pub const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);

/// What a job body returns on success. The runner emits it verbatim as the
/// terminal event.
pub struct Finished {
    pub level: Level,
    pub message: String,
    pub body: EventBody,
}

struct Running {
    kind: JobKind,
    started_at: SystemTime,
    cancel: CancellationToken,
    progress: Arc<Mutex<Option<Progress>>>,
}

/// The job registry: one at a time per `JobKind::one_at_a_time`, every job
/// cancellable, every job's terminal event guaranteed.
pub struct Jobs {
    runtime: Handle,
    next_id: AtomicU64,
    running: Mutex<HashMap<u64, Running>>,
    bus: Arc<EventBus>,
}

/// Handed to a job body. `progress` and `changed` are throttled to four
/// events a second; `checkpoint` is how a tight loop notices cancellation.
pub struct JobCtx {
    pub id: u64,
    pub kind: JobKind,
    pub cancel: CancellationToken,
    bus: Arc<EventBus>,
    runtime: Handle,
    progress: Arc<Mutex<Option<Progress>>>,
    throttle: Mutex<Throttle>,
    changed: Mutex<ChangedThrottle>,
    /// Set by the runner, under the same locks the scheduled flushes drain
    /// with, before the terminal event. A scheduled flush that observes
    /// this checks it while holding that lock, so it can never emit after
    /// the runner has already flushed and moved on to the terminal event.
    finished: AtomicBool,
}

#[derive(Default)]
struct Throttle {
    last_emit: Option<Instant>,
    pending: Option<Progress>,
    flush_scheduled: bool,
}

#[derive(Default)]
struct ChangedThrottle {
    last_emit: Option<Instant>,
    pending: HashMap<u64, SeriesCard>,
    flush_scheduled: bool,
}

impl JobCtx {
    fn job_ref(&self, phase: JobPhase) -> JobRef {
        JobRef { id: self.id, kind: self.kind, phase }
    }

    pub fn emit(&self, level: Level, message: impl Into<String>, body: EventBody) {
        self.bus.emit(level, self.kind.stage(), message, Some(self.job_ref(JobPhase::Running)), body);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// `?` on this at the top of every loop body; the runner turns the
    /// error into `JobCancelled`.
    pub fn checkpoint(&self) -> Result<(), CoreError> {
        if self.is_cancelled() { Err(CoreError::internal("cancelled")) } else { Ok(()) }
    }

    /// At most four `JobProgress` events a second per job. A burst of calls
    /// keeps only the latest value, flushed 250 ms after the last emit.
    pub fn progress(self: &Arc<Self>, done: u64, total: Option<u64>, label: &str) {
        let p = Progress { done, total, label: label.to_string() };
        *self.progress.lock().unwrap() = Some(p.clone());
        let mut t = self.throttle.lock().unwrap();
        let due = t.last_emit.is_none_or(|l| l.elapsed() >= PROGRESS_INTERVAL);
        if due {
            t.last_emit = Some(Instant::now());
            t.pending = None;
            drop(t);
            self.emit_progress(p);
        } else {
            t.pending = Some(p);
            if !t.flush_scheduled {
                t.flush_scheduled = true;
                let wait = PROGRESS_INTERVAL.saturating_sub(t.last_emit.unwrap().elapsed());
                let me = self.clone();
                self.runtime.spawn(async move {
                    tokio::time::sleep(wait).await;
                    // Held for the whole check-drain-emit: whichever of this
                    // task and the runner's `flush_progress` reaches the
                    // lock first completes entirely, emit included, before
                    // the other can start, so a flush from here can never
                    // land after the terminal event.
                    let mut t = me.throttle.lock().unwrap();
                    t.flush_scheduled = false;
                    if me.finished.load(Ordering::SeqCst) {
                        return;
                    }
                    if let Some(p) = t.pending.take() {
                        t.last_emit = Some(Instant::now());
                        me.emit_progress(p);
                    }
                });
            }
        }
    }

    fn emit_progress(&self, p: Progress) {
        self.emit(Level::Debug, format!("{} {}", self.kind.as_str(), p.label), EventBody::JobProgress { done: p.done, total: p.total, label: p.label });
    }

    /// Emits whatever progress is buffered right now, so a job never ends
    /// with a value still waiting on a scheduled flush. A no-op when
    /// nothing is pending. Holds the throttle lock through the emit for
    /// the same reason the scheduled flush above does.
    pub fn flush_progress(&self) {
        let mut t = self.throttle.lock().unwrap();
        if let Some(p) = t.pending.take() {
            t.last_emit = Some(Instant::now());
            self.emit_progress(p);
        }
    }

    /// At most four `SeriesChanged` events a second per job, batching every
    /// card pushed in between. A card for the same series id pushed twice
    /// in one window keeps only the latest.
    pub fn changed(self: &Arc<Self>, card: SeriesCard) {
        self.changed_all(vec![card]);
    }

    /// The batch form of `changed`, for a job step that produced several
    /// cards at once: they leave together as one event, rather than the
    /// first immediately and the rest on the next flush. Same window and
    /// same guarantees.
    pub fn changed_all(self: &Arc<Self>, cards: Vec<SeriesCard>) {
        if cards.is_empty() {
            return;
        }
        let mut t = self.changed.lock().unwrap();
        let due = t.last_emit.is_none_or(|l| l.elapsed() >= PROGRESS_INTERVAL);
        if due {
            t.last_emit = Some(Instant::now());
            // Drain whatever is already buffered into the same batch as
            // the new cards (the new ones winning on a same-id collision),
            // so a due emission never leaves an older card behind for a
            // scheduled flush to deliver stale, after this one.
            let mut batch = std::mem::take(&mut t.pending);
            for card in cards {
                batch.insert(card.id, card);
            }
            drop(t);
            self.emit_changed(batch.into_values().collect());
        } else {
            for card in cards {
                t.pending.insert(card.id, card);
            }
            if !t.flush_scheduled {
                t.flush_scheduled = true;
                let wait = PROGRESS_INTERVAL.saturating_sub(t.last_emit.unwrap().elapsed());
                let me = self.clone();
                self.runtime.spawn(async move {
                    tokio::time::sleep(wait).await;
                    // Held for the whole check-drain-emit, for the same
                    // reason `progress`'s scheduled flush holds it.
                    let mut t = me.changed.lock().unwrap();
                    t.flush_scheduled = false;
                    if me.finished.load(Ordering::SeqCst) {
                        return;
                    }
                    let cards: Vec<SeriesCard> = std::mem::take(&mut t.pending).into_values().collect();
                    if !cards.is_empty() {
                        t.last_emit = Some(Instant::now());
                        me.emit_changed(cards);
                    }
                });
            }
        }
    }

    /// Emits whatever is buffered right now, so a job never ends with cards
    /// still waiting on a scheduled flush. A no-op when nothing is pending.
    /// Holds the lock through the emit for the same reason the scheduled
    /// flush above does.
    pub fn flush_changed(&self) {
        let mut t = self.changed.lock().unwrap();
        let cards: Vec<SeriesCard> = std::mem::take(&mut t.pending).into_values().collect();
        if !cards.is_empty() {
            t.last_emit = Some(Instant::now());
            self.emit_changed(cards);
        }
    }

    fn emit_changed(&self, cards: Vec<SeriesCard>) {
        let n = cards.len();
        self.emit(Level::Debug, format!("{} updated {n} series", self.kind.as_str()), EventBody::SeriesChanged { series: cards });
    }

    /// Called once by the runner, after the job's own future has completed
    /// or been dropped, and before the terminal event: marks the job
    /// finished so no scheduled flush can land after it, then flushes
    /// whatever is still buffered.
    fn finish(&self) {
        self.finished.store(true, Ordering::SeqCst);
        self.flush_progress();
        self.flush_changed();
    }
}

/// Guarantees a job's registry entry and terminal event are never lost to a
/// panic in its body. Created before the body runs; the normal path (the
/// `tokio::select!` resolving, whatever the outcome) disarms it by setting
/// `done`. If it drops still armed, the body panicked: it removes the
/// entry and emits `JobFailed` itself so the slot is never left wedged.
struct JobGuard {
    jobs: Arc<Jobs>,
    ctx: Arc<JobCtx>,
    id: u64,
    kind: JobKind,
    done: bool,
}

impl Drop for JobGuard {
    fn drop(&mut self) {
        if self.done {
            return;
        }
        // No further scheduled flush should emit for a job ending this way
        // either.
        self.ctx.finished.store(true, Ordering::SeqCst);
        self.jobs.running.lock().unwrap().remove(&self.id);
        let finished = JobRef { id: self.id, kind: self.kind, phase: JobPhase::Finished };
        if std::thread::panicking() {
            self.jobs.bus.emit(
                Level::Error,
                self.kind.stage(),
                format!("{} panicked", self.kind.as_str()),
                Some(finished),
                EventBody::JobFailed { error: CoreError::internal("job panicked") },
            );
        } else {
            // Still armed at drop but the thread is not unwinding: the
            // body's future was dropped without running to completion,
            // which is what a runtime shutdown produces (and, in
            // principle, a `select!` whose cancelled arm won before this
            // guard's `done` flag could be set). Either way the job did
            // not fail, it was cancelled.
            self.jobs.bus.emit(
                Level::Info,
                self.kind.stage(),
                format!("{} cancelled", self.kind.as_str()),
                Some(finished),
                EventBody::JobCancelled,
            );
        }
    }
}

impl Jobs {
    pub fn new(runtime: Handle, bus: Arc<EventBus>) -> Arc<Jobs> {
        Arc::new(Jobs { runtime, next_id: AtomicU64::new(1), running: Mutex::new(HashMap::new()), bus })
    }

    pub fn running(&self, kind: JobKind) -> Option<u64> {
        self.running.lock().unwrap().iter().find(|(_, r)| r.kind == kind).map(|(id, _)| *id)
    }

    /// Starts `f` on the runtime under a fresh id, unless `kind` runs one at
    /// a time and one is already running, in which case that job's id comes
    /// back and nothing new starts.
    pub fn start<F, Fut>(self: &Arc<Self>, kind: JobKind, f: F) -> u64
    where
        F: FnOnce(Arc<JobCtx>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Finished, CoreError>> + Send + 'static,
    {
        let mut running = self.running.lock().unwrap();
        if kind.one_at_a_time()
            && let Some((id, _)) = running.iter().find(|(_, r)| r.kind == kind)
        {
            return *id;
        }
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let cancel = CancellationToken::new();
        let progress = Arc::new(Mutex::new(None));
        running.insert(id, Running { kind, started_at: time::now(), cancel: cancel.clone(), progress: progress.clone() });
        drop(running);

        let ctx = Arc::new(JobCtx {
            id,
            kind,
            cancel: cancel.clone(),
            bus: self.bus.clone(),
            runtime: self.runtime.clone(),
            progress,
            throttle: Mutex::new(Throttle::default()),
            changed: Mutex::new(ChangedThrottle::default()),
            finished: AtomicBool::new(false),
        });
        self.bus.emit(Level::Debug, kind.stage(), format!("{} started", kind.as_str()), Some(JobRef { id, kind, phase: JobPhase::Started }), EventBody::JobStarted { kind });

        let jobs = self.clone();
        self.runtime.spawn(async move {
            // Armed before the body runs; disarmed the instant `select!`
            // resolves, whatever the outcome. If the body panics instead,
            // this never gets to disarm and its `Drop` cleans up the
            // registry and emits the terminal event itself.
            let mut guard = JobGuard { jobs: jobs.clone(), ctx: ctx.clone(), id, kind, done: false };
            let outcome = tokio::select! {
                _ = cancel.cancelled() => None,
                r = f(ctx.clone()) => Some(r),
            };
            guard.done = true;
            jobs.running.lock().unwrap().remove(&id);
            ctx.finish();
            let finished = JobRef { id, kind, phase: JobPhase::Finished };
            match outcome {
                None => {
                    jobs.bus.emit(Level::Info, kind.stage(), format!("{} cancelled", kind.as_str()), Some(finished), EventBody::JobCancelled);
                }
                Some(Err(e)) if e == CoreError::internal("cancelled") => {
                    jobs.bus.emit(Level::Info, kind.stage(), format!("{} cancelled", kind.as_str()), Some(finished), EventBody::JobCancelled);
                }
                Some(Err(e)) => {
                    jobs.bus.emit(Level::Error, kind.stage(), format!("{} failed: {e}", kind.as_str()), Some(finished), EventBody::JobFailed { error: e });
                }
                Some(Ok(done)) => {
                    jobs.bus.emit(done.level, kind.stage(), done.message, Some(finished), done.body);
                }
            }
        });
        id
    }

    pub fn cancel(&self, id: u64) -> Result<(), CoreError> {
        match self.running.lock().unwrap().get(&id) {
            Some(r) => {
                r.cancel.cancel();
                Ok(())
            }
            None => Err(CoreError::NotFound { what: Entity::Job, id }),
        }
    }

    pub fn cancel_all(&self) {
        for r in self.running.lock().unwrap().values() {
            r.cancel.cancel();
        }
    }

    pub fn list(&self) -> Vec<JobInfo> {
        let mut out: Vec<JobInfo> = self
            .running
            .lock()
            .unwrap()
            .iter()
            .map(|(id, r)| JobInfo { id: *id, kind: r.kind, started_at: r.started_at, progress: r.progress.lock().unwrap().clone() })
            .collect();
        out.sort_by_key(|j| j.id);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Collector, EventBus};
    use crate::store::Store;
    use std::time::Duration;

    fn setup() -> (tempfile::TempDir, tokio::runtime::Runtime, Arc<Jobs>, Arc<Collector>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("anibeam.db")).unwrap();
        let bus = EventBus::new(store).unwrap();
        let collector = Arc::new(Collector::default());
        bus.subscribe(collector.clone());
        let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
        let jobs = Jobs::new(rt.handle().clone(), bus);
        (dir, rt, jobs, collector)
    }

    /// Blocks on the bus's own condvar rather than sleep-polling.
    fn wait_for_finished(c: &Collector, id: u64) {
        let arrived = c.wait_for(
            |events| events.iter().any(|e| e.job.as_ref().is_some_and(|j| j.id == id && j.phase == JobPhase::Finished)),
            Duration::from_secs(2),
        );
        if !arrived {
            panic!("job {id} never finished; events seen: {:?}", c.events());
        }
    }

    fn card(id: u64) -> SeriesCard {
        SeriesCard {
            id,
            kind: SeriesKind::Show,
            path: format!("/lib/series-{id}"),
            title: format!("Series {id}"),
            titles: Titles { romaji: None, english: None, native: None, folder: format!("series-{id}") },
            poster: None,
            format: None,
            status: None,
            hidden: false,
            missing: false,
            match_info: None,
            episodes_on_disk: 0,
            extras_on_disk: 0,
            total_episodes: None,
            total_is_estimate: false,
            code: None,
            watched: None,
            watched_state: WatchedState::Unknown,
            strip: Strip { watched: 0.0, aired_unwatched: 0.0, unknown: 0.0 },
            community_score: None,
            my_score: None,
            list_status: None,
            next_airing: None,
            last_viewed_at: None,
            latest_activity_at: time::now(),
        }
    }

    #[test]
    fn a_job_starts_runs_and_finishes_with_its_own_terminal_event() {
        let (_d, _rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Search, |ctx| async move {
            ctx.emit(Level::Debug, "half way", EventBody::Notice);
            Ok(Finished { level: Level::Debug, message: "search done".into(), body: EventBody::SearchFinished { results: vec![] } })
        });
        wait_for_finished(&c, id);
        let mine: Vec<Event> = c.events().into_iter().filter(|e| e.job.as_ref().is_some_and(|j| j.id == id)).collect();
        assert_eq!(mine[0].job.as_ref().unwrap().phase, JobPhase::Started);
        assert!(matches!(mine[0].body, EventBody::JobStarted { kind: JobKind::Search }));
        assert_eq!(mine[1].job.as_ref().unwrap().phase, JobPhase::Running);
        assert!(matches!(mine.last().unwrap().body, EventBody::SearchFinished { .. }));
        assert!(jobs.list().is_empty());
    }

    #[test]
    fn a_failing_job_ends_in_job_failed() {
        let (_d, _rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Search, |_ctx| async move { Err(CoreError::internal("nope")) });
        wait_for_finished(&c, id);
        let last = c.events().into_iter().rfind(|e| e.job.as_ref().is_some_and(|j| j.id == id)).unwrap();
        assert!(matches!(last.body, EventBody::JobFailed { .. }));
        assert_eq!(last.level, Level::Error);
    }

    #[test]
    fn cancel_ends_a_job_with_job_cancelled() {
        let (_d, _rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Scan, |ctx| async move {
            loop {
                ctx.checkpoint()?;
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });
        std::thread::sleep(Duration::from_millis(30));
        jobs.cancel(id).unwrap();
        wait_for_finished(&c, id);
        let last = c.events().into_iter().rfind(|e| e.job.as_ref().is_some_and(|j| j.id == id)).unwrap();
        assert!(matches!(last.body, EventBody::JobCancelled));
        assert!(matches!(jobs.cancel(id), Err(CoreError::NotFound { what: Entity::Job, .. })));
    }

    #[test]
    fn one_at_a_time_kinds_return_the_running_id() {
        let (_d, _rt, jobs, c) = setup();
        let first = jobs.start(JobKind::Scan, |ctx| async move {
            while !ctx.is_cancelled() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Ok(Finished { level: Level::Info, message: "".into(), body: EventBody::ScanFinished { source: None, added: 0, changed: 0, removed: 0 } })
        });
        let second = jobs.start(JobKind::Scan, |_ctx| async move { panic!("must not run") });
        assert_eq!(first, second);
        let other = jobs.start(JobKind::Search, |_ctx| async move { Ok(Finished { level: Level::Debug, message: "".into(), body: EventBody::SearchFinished { results: vec![] } }) });
        assert_ne!(other, first);
        jobs.cancel(first).unwrap();
        wait_for_finished(&c, first);
    }

    #[test]
    fn progress_is_throttled_to_four_a_second_and_the_last_value_lands() {
        let (_d, _rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Scan, |ctx| async move {
            for i in 0..100u64 {
                ctx.progress(i, Some(100), "walking");
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
            Ok(Finished { level: Level::Info, message: "".into(), body: EventBody::ScanFinished { source: None, added: 0, changed: 0, removed: 0 } })
        });
        wait_for_finished(&c, id);
        let progress: Vec<u64> = c
            .events()
            .into_iter()
            .filter_map(|e| match e.body {
                EventBody::JobProgress { done, .. } if e.job.as_ref().is_some_and(|j| j.id == id) => Some(done),
                _ => None,
            })
            .collect();
        assert!(progress.len() <= 3, "{progress:?}");
        assert_eq!(*progress.last().unwrap(), 99);
    }

    #[test]
    fn changed_batches_and_flushes_before_the_terminal_event() {
        let (_d, _rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Scan, |ctx| async move {
            for i in 0..50u64 {
                ctx.changed(card(i));
            }
            Ok(Finished { level: Level::Info, message: "".into(), body: EventBody::ScanFinished { source: None, added: 0, changed: 0, removed: 0 } })
        });
        wait_for_finished(&c, id);
        let events = c.events();
        let mine: Vec<&Event> = events.iter().filter(|e| e.job.as_ref().is_some_and(|j| j.id == id)).collect();
        let changed_events: Vec<&Event> = mine.iter().filter(|e| matches!(e.body, EventBody::SeriesChanged { .. })).copied().collect();
        assert!(changed_events.len() <= 3, "{changed_events:?}");

        let mut ids: Vec<u64> = changed_events
            .iter()
            .flat_map(|e| match &e.body {
                EventBody::SeriesChanged { series } => series.iter().map(|c| c.id).collect::<Vec<_>>(),
                _ => vec![],
            })
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 50, "{ids:?}");

        let last_changed_seq = changed_events.last().unwrap().seq;
        let terminal_seq = mine.iter().find(|e| e.job.as_ref().unwrap().phase == JobPhase::Finished).unwrap().seq;
        assert!(last_changed_seq < terminal_seq);
    }

    /// Asserts that among this job's events, the terminal one has the
    /// highest `seq`: nothing, including a lagging scheduled flush,
    /// slipped out after it.
    fn assert_nothing_follows_the_terminal_event(mine: &[Event]) {
        let terminal_seq = mine.iter().find(|e| e.job.as_ref().unwrap().phase == JobPhase::Finished).unwrap().seq;
        let max_seq = mine.iter().map(|e| e.seq).max().unwrap();
        assert_eq!(max_seq, terminal_seq, "an event landed after the terminal event: {mine:?}");
    }

    #[test]
    fn progress_never_lands_after_the_terminal_event() {
        let (_d, _rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Scan, |ctx| async move {
            ctx.progress(1, Some(10), "walking");
            ctx.progress(2, Some(10), "walking");
            Ok(Finished { level: Level::Info, message: "".into(), body: EventBody::ScanFinished { source: None, added: 0, changed: 0, removed: 0 } })
        });
        wait_for_finished(&c, id);
        // Give any lagging scheduled flush (armed by the second, throttled
        // `progress` call) its full window to fire, if it's going to.
        std::thread::sleep(PROGRESS_INTERVAL + Duration::from_millis(100));
        let mine: Vec<Event> = c.events().into_iter().filter(|e| e.job.as_ref().is_some_and(|j| j.id == id)).collect();
        assert_nothing_follows_the_terminal_event(&mine);
    }

    #[test]
    fn progress_never_lands_after_the_terminal_event_on_cancel() {
        let (_d, _rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Scan, |ctx| async move {
            ctx.progress(1, Some(10), "walking");
            ctx.progress(2, Some(10), "walking");
            loop {
                ctx.checkpoint()?;
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });
        // Cancel well inside the throttle window, so the second `progress`
        // call's flush is still armed and pending when the job ends.
        std::thread::sleep(Duration::from_millis(30));
        jobs.cancel(id).unwrap();
        wait_for_finished(&c, id);
        std::thread::sleep(PROGRESS_INTERVAL + Duration::from_millis(100));
        let mine: Vec<Event> = c.events().into_iter().filter(|e| e.job.as_ref().is_some_and(|j| j.id == id)).collect();
        assert_nothing_follows_the_terminal_event(&mine);
    }

    #[test]
    fn a_panicking_job_ends_in_job_failed_and_frees_the_slot() {
        let (_d, _rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Scan, |_ctx| async move {
            panic!("boom");
        });
        wait_for_finished(&c, id);
        let last = c.events().into_iter().rfind(|e| e.job.as_ref().is_some_and(|j| j.id == id)).unwrap();
        assert!(matches!(last.body, EventBody::JobFailed { .. }), "{:?}", last.body);
        assert_eq!(last.level, Level::Error);
        assert!(jobs.list().is_empty());
        assert!(matches!(jobs.cancel(id), Err(CoreError::NotFound { what: Entity::Job, .. })));

        // The slot is free: the same one-at-a-time kind starts a new job
        // under a new id rather than handing back the dead one.
        let second = jobs.start(JobKind::Scan, |_ctx| async move {
            Ok(Finished { level: Level::Info, message: "".into(), body: EventBody::ScanFinished { source: None, added: 0, changed: 0, removed: 0 } })
        });
        assert_ne!(id, second);
        wait_for_finished(&c, second);
    }

    #[test]
    fn a_job_dropped_by_runtime_shutdown_ends_cancelled_not_failed() {
        let (_d, rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Scan, |ctx| async move {
            ctx.checkpoint()?;
            ctx.emit(Level::Debug, "about to hang", EventBody::Notice);
            // Never resolves: the only way this task ends is by being
            // dropped, either by `cancel` (not exercised here) or by the
            // runtime shutting down out from under it, which is what this
            // test does below.
            std::future::pending::<Result<Finished, CoreError>>().await
        });

        // Wait until the body has actually been polled and reached the
        // pending await, so shutting the runtime down below drops a task
        // that genuinely started running rather than one that was queued
        // but never polled even once.
        let started = c.wait_for(
            |events| events.iter().any(|e| e.job.as_ref().is_some_and(|j| j.id == id) && matches!(e.body, EventBody::Notice)),
            Duration::from_secs(2),
        );
        assert!(started, "job never reached the pending await; events seen: {:?}", c.events());

        // Drops the still-pending task without ever resuming it: the
        // `JobGuard`'s `done` flag is never set on this path, so its
        // `Drop` is what has to tell a runtime-shutdown drop apart from a
        // real panic.
        rt.shutdown_timeout(Duration::from_millis(100));

        // The store's writer thread is a plain OS thread independent of
        // the tokio runtime just shut down, so the bus and its listeners
        // are unaffected; assert on the collector, not `bus.recent`, since
        // the point of this test is what the shell saw, not the ring.
        let arrived = c.wait_for(
            |events| events.iter().any(|e| e.job.as_ref().is_some_and(|j| j.id == id && j.phase == JobPhase::Finished)),
            Duration::from_secs(2),
        );
        assert!(arrived, "job never got a terminal event; events seen: {:?}", c.events());
        let last = c.events().into_iter().rfind(|e| e.job.as_ref().is_some_and(|j| j.id == id)).unwrap();
        assert!(matches!(last.body, EventBody::JobCancelled), "{:?}", last.body);
        assert_eq!(last.level, Level::Info);
    }

    #[test]
    fn changed_never_delivers_a_stale_card_after_a_newer_one_for_the_same_id() {
        let (_d, _rt, jobs, c) = setup();
        let id = jobs.start(JobKind::Scan, |ctx| async move {
            // First push for id 2 is the very first `changed` call ever on
            // this ctx, so it is due and seeds `last_emit`.
            ctx.changed(card(2));
            // A second push for the same id, right behind it, is not due:
            // it is buffered and arms a flush ~250 ms out.
            let mut buffered = card(2);
            buffered.title = "first".into();
            ctx.changed(buffered);
            // Spin past the throttle window on this task's own clock. This
            // races this call against the armed flush's timer; whichever
            // wins, the fix must still deliver "second" last for id 2.
            let deadline = Instant::now() + PROGRESS_INTERVAL;
            while Instant::now() < deadline {
                tokio::task::yield_now().await;
            }
            let mut newer = card(2);
            newer.title = "second".into();
            ctx.changed(newer);
            Ok(Finished { level: Level::Info, message: "".into(), body: EventBody::ScanFinished { source: None, added: 0, changed: 0, removed: 0 } })
        });
        wait_for_finished(&c, id);
        let events = c.events();
        let mentions_2: Vec<&Event> = events
            .iter()
            .filter(|e| e.job.as_ref().is_some_and(|j| j.id == id))
            .filter(|e| matches!(&e.body, EventBody::SeriesChanged { series } if series.iter().any(|c| c.id == 2)))
            .collect();
        assert!(!mentions_2.is_empty(), "no SeriesChanged mentioned id 2");
        let last = mentions_2.last().unwrap();
        let EventBody::SeriesChanged { series } = &last.body else { unreachable!() };
        let c2 = series.iter().find(|c| c.id == 2).unwrap();
        assert_eq!(c2.title, "second", "a stale card for id 2 landed after the newer one");
    }
}
