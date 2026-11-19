//! The screens' reads of the core, off the UI thread (#1206). A read waits on the store's
//! lock, and a busy process can hold that for longer than a frame, so the tab changes, the
//! clock's tick, opening a series or a show, and the sources read here, never on the UI
//! thread. The taps still left on the UI thread are listed in #1208. Each kind
//! of read goes to a worker that runs them in the order asked, and each answer comes back
//! through the event loop. A small write that must keep the order it was made in, such as a
//! preference, can go the same way.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;
use std::sync::mpsc::{SendError, Sender, channel};

use anibeam_core::Core;

pub type Job = Box<dyn FnOnce() + Send>;

/// A named thread that runs jobs in the order they come, for the life of the process. A job
/// that panics is caught there, so the thread and the jobs queued behind it carry on. With no
/// thread (it could not start), each job gets a thread of its own.
pub struct Worker {
    name: &'static str,
    jobs: OnceLock<Option<Sender<Job>>>,
}

impl Worker {
    pub const fn new(name: &'static str) -> Worker {
        Worker {
            name,
            jobs: OnceLock::new(),
        }
    }

    pub fn run(&self, job: Job) {
        let jobs = self.jobs.get_or_init(|| {
            let (tx, rx) = channel::<Job>();
            std::thread::Builder::new()
                .name(self.name.into())
                .spawn(move || {
                    for job in rx {
                        let _ = catch_unwind(AssertUnwindSafe(job));
                    }
                })
                .ok()
                .map(|_| tx)
        });
        let job = match jobs {
            Some(jobs) => match jobs.send(job) {
                Ok(()) => return,
                Err(SendError(job)) => job,
            },
            None => job,
        };
        // A job a thread cannot be found for is dropped, and its `Landing` says so.
        let _ = std::thread::Builder::new()
            .name(self.name.into())
            .spawn(move || {
                let _ = catch_unwind(AssertUnwindSafe(job));
            });
    }
}

/// Hands a job's answer to the UI thread, exactly once: what the job gave, or `None` when it
/// never finished, because it panicked or no thread would take it. A page waiting on a read
/// never waits for good.
pub struct Landing<T: Send + 'static>(Option<Box<dyn FnOnce(Option<T>) + Send>>);

impl<T: Send + 'static> Landing<T> {
    pub fn new(land: impl FnOnce(Option<T>) + Send + 'static) -> Landing<T> {
        Landing(Some(Box::new(land)))
    }

    pub fn land(mut self, answer: T) {
        self.post(Some(answer));
    }

    fn post(&mut self, answer: Option<T>) {
        if let Some(land) = self.0.take() {
            let _ = slint::invoke_from_event_loop(move || land(answer));
        }
    }
}

impl<T: Send + 'static> Drop for Landing<T> {
    fn drop(&mut self) {
        self.post(None);
    }
}

/// The reads the pages make on their own: the tick's, a tab's, the sources'.
static READS: Worker = Worker::new("screen-reads");
/// The reads a person waits on after a tap, apart from those, so they never queue behind
/// them.
static ASKED: Worker = Worker::new("screen-asks");

/// Runs `read` against the core on the reads thread and hands what it gives to `land` on the
/// UI thread, or `None` if the read never finished. False, and nothing runs, only when the
/// core is not open yet: with the core open, `land` always comes.
pub fn read<T: Send + 'static>(
    read: impl FnOnce(&Core) -> T + Send + 'static,
    land: impl FnOnce(Option<T>) + Send + 'static,
) -> bool {
    on(&READS, read, land)
}

/// `read`, for a read the person is waiting on: it never queues behind the pages' own.
pub fn read_now<T: Send + 'static>(
    read: impl FnOnce(&Core) -> T + Send + 'static,
    land: impl FnOnce(Option<T>) + Send + 'static,
) -> bool {
    on(&ASKED, read, land)
}

fn on<T: Send + 'static>(
    worker: &'static Worker,
    read: impl FnOnce(&Core) -> T + Send + 'static,
    land: impl FnOnce(Option<T>) + Send + 'static,
) -> bool {
    let Some(core) = crate::local::core() else {
        return false;
    };
    let landing = Landing::new(land);
    worker.run(Box::new(move || landing.land(read(core.as_ref()))));
    true
}

/// Whether a read is due: marked when what it reads may have changed, and started one at a
/// time. A change while a read is out leaves it due again, so one more read follows when that
/// one lands, and a burst of changes never queues a burst of reads.
#[derive(Debug)]
pub struct Due {
    due: bool,
    out: bool,
}

impl Due {
    /// Due at once: nothing has been read yet.
    pub const fn now() -> Due {
        Due {
            due: true,
            out: false,
        }
    }

    /// What it reads may have changed.
    pub fn mark(&mut self) {
        self.due = true;
    }

    /// Whether to start the read now: it is due and none is out. Starting takes the mark.
    pub fn start(&mut self) -> bool {
        if !self.due || self.out {
            return false;
        }
        self.due = false;
        self.out = true;
        true
    }

    /// The read landed.
    pub fn landed(&mut self) {
        self.out = false;
    }

    /// The read could not start: it stays due for the next try.
    pub fn failed(&mut self) {
        self.out = false;
        self.due = true;
    }
}

/// A `Due` starts due: nothing has been read yet.
impl Default for Due {
    fn default() -> Due {
        Due::now()
    }
}

#[cfg(test)]
mod tests {
    use super::{Due, Worker};
    use std::time::Duration;

    /// A read that panics must not take the reads thread down: every `Due` waiting on a
    /// read behind it would stay out for good (#1206).
    #[test]
    fn a_job_that_panics_leaves_the_worker_running_the_next() {
        static WORKER: Worker = Worker::new("test-worker");
        let (tx, rx) = std::sync::mpsc::channel();
        WORKER.run(Box::new(|| panic!("a read went wrong")));
        WORKER.run(Box::new(move || tx.send(7).unwrap()));
        assert_eq!(rx.recv_timeout(Duration::from_secs(10)), Ok(7));
    }

    #[test]
    fn changes_while_a_read_is_out_make_one_more_read() {
        let mut d = Due::now();
        assert!(d.start());
        // Three changes while it is out: none starts a read of its own.
        for _ in 0..3 {
            d.mark();
            assert!(!d.start());
        }
        // They make one more once it lands, and that one ends the run.
        d.landed();
        assert!(d.start());
        d.landed();
        assert!(!d.start());
    }

    #[test]
    fn nothing_is_read_again_until_something_changes() {
        let mut d = Due::now();
        assert!(d.start());
        d.landed();
        assert!(!d.start());
        d.mark();
        assert!(d.start());
    }

    #[test]
    fn a_read_that_could_not_start_is_tried_again() {
        let mut d = Due::now();
        assert!(d.start());
        d.failed();
        assert!(d.start());
    }
}
