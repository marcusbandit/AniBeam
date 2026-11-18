//! The screens' reads of the core, off the UI thread. The UI thread never waits on the core
//! or the store, not even for a short read: a read waits on the store's lock, and a busy
//! process can hold that for longer than a frame (#1206). One thread runs the reads in the
//! order they were asked, and each answer comes back through the event loop. A small write
//! that must keep the order it was made in, such as a preference, can go the same way.

use std::sync::OnceLock;
use std::sync::mpsc::{Sender, channel};

use anibeam_core::Core;

type Job = Box<dyn FnOnce() + Send>;

fn jobs() -> Option<&'static Sender<Job>> {
    static JOBS: OnceLock<Option<Sender<Job>>> = OnceLock::new();
    JOBS.get_or_init(|| {
        let (tx, rx) = channel::<Job>();
        std::thread::Builder::new()
            .name("screen-reads".into())
            .spawn(move || {
                for job in rx {
                    job();
                }
            })
            .ok()
            .map(|_| tx)
    })
    .as_ref()
}

/// Runs `read` against the core on the reads thread and hands what it gives to `land` on
/// the UI thread. False, and nothing runs, when the core is not open yet.
pub fn read<T: Send + 'static>(
    read: impl FnOnce(&Core) -> T + Send + 'static,
    land: impl FnOnce(T) + Send + 'static,
) -> bool {
    let (Some(core), Some(jobs)) = (crate::local::core(), jobs()) else {
        return false;
    };
    jobs.send(Box::new(move || {
        let answer = read(core.as_ref());
        let _ = slint::invoke_from_event_loop(move || land(answer));
    }))
    .is_ok()
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

#[cfg(test)]
mod tests {
    use super::Due;

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
