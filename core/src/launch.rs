//! What the core does once, at launch, and never again until the next one.
//!
//! One catch-up walk of the library, and behind it the jobs a fresh start
//! owes: the match, the backfill, the crawl, the airing schedules, the
//! tracker lists, the images. Nothing here is on a timer. The scan's own
//! terminal event is the trigger, so the list runs when the library is
//! actually on the tables rather than at some guessed moment after it.
//!
//! Terminal means terminal, whatever it says: a scan the user cancelled from
//! the job list and a scan that failed both end the launch's waiting exactly
//! as a finished one does. The rest of the list is what the session owes
//! either way, and losing all of it to one cancelled walk would be the worse
//! bug by far.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use crate::contract::*;
use crate::core::Core;
use crate::franchise::crawl;
use crate::images;
use crate::library::scan::{self, ScanScope};
use crate::metadata::{airing, apply, automatch};
use crate::time;
use crate::trackers::cache;

/// The catch-up walk, and the one-shot listener that turns its end into the
/// rest of the launch. Called by `Core::start` once the watcher is up.
///
/// The scan installs the recursive watch on every source itself, at the top
/// of its own loop, so this is also what puts the library under the watcher.
///
/// Nothing here says the machine has no keyring: whichever caller finishes
/// the probe writes that line, and `Core::start`'s warm-up is one of them.
pub fn start(core: &Arc<Core>) -> Result<(), CoreError> {
    // Registered before the scan starts, not after: a library with no
    // sources at all has nothing to walk, so its scan can be over before
    // `scan::start` has even returned an id here, and a listener added
    // afterwards would have missed the one event it exists for.
    let listener = Arc::new(AfterScan {
        core: Arc::downgrade(core),
        scan: AtomicU64::new(0),
        fired: AtomicBool::new(false),
    });
    core.set_launch_listener(core.subscribe(listener.clone()));
    listener.latch(scan::start(core, ScanScope::All));
    Ok(())
}

/// Waits for the catch-up scan to end and then runs the list, once. It holds
/// a `Weak`, so a core that goes away mid-scan takes this with it rather
/// than keeping itself alive through the bus.
struct AfterScan {
    core: Weak<Core>,
    /// The catch-up scan's id, latched the instant `scan::start` returns it.
    /// Zero until then, since job ids start at one.
    scan: AtomicU64,
    fired: AtomicBool,
}

impl AfterScan {
    fn latch(&self, job: u64) {
        self.scan.store(job, Ordering::SeqCst);
    }

    /// Whether this is the end of the scan the launch is waiting on. The
    /// body is not looked at: `ScanFinished`, `JobCancelled` and `JobFailed`
    /// all arrive at `JobPhase::Finished`, and all three mean the same thing
    /// here, which is that the walk is over and the rest of the launch is
    /// due.
    ///
    /// An unset latch is the window between the scan starting and
    /// `scan::start` handing its id back, which an empty library can finish
    /// inside. Only one Scan runs at a time, so the only job that can end in
    /// that window is the catch-up scan itself, or the `AddSource` scan it
    /// was folded into, which is the same walk under an earlier id.
    fn is_the_scan(&self, event: &Event) -> bool {
        let Some(job) = event.job.as_ref() else {
            return false;
        };
        if job.kind != JobKind::Scan || job.phase != JobPhase::Finished {
            return false;
        }
        match self.scan.load(Ordering::SeqCst) {
            0 => true,
            latched => job.id == latched,
        }
    }
}

impl EventListener for AfterScan {
    fn on_event(&self, event: Event) {
        if !self.is_the_scan(&event) {
            return;
        }
        if self.fired.swap(true, Ordering::SeqCst) {
            return;
        }
        let Some(core) = self.core.upgrade() else {
            return;
        };
        // Dropping the handle unsubscribes, so the bus stops calling this
        // the moment the launch has had its one scan.
        core.take_launch_listener();
        after_scan(&core);
    }
}

/// The list, in this order, each started without waiting for the one before
/// it: they run one at a time per kind and pace themselves against their own
/// upstream, so the order is the order they get their turn in, not a queue
/// anything blocks on.
fn after_scan(core: &Arc<Core>) {
    automatch::start(core);
    apply::backfill_stubs(core);
    crawl::start_gap_crawl(core);
    airing::start_refresh_library(core);
    cache::start_refresh(core, None, false);
    images::start_fill(core);

    // The sweep is bookkeeping rather than a job: one transaction over the
    // rows and one walk of the image directory, so it goes on the runtime
    // with nothing waiting on it and reports to the trace log.
    let owner = core.clone();
    let now = time::now_secs();
    core.handle.spawn(async move {
        let cache = owner.images.clone();
        match owner.store.write_async(move |c| cache.sweep(c, now)).await {
            Ok(report) => tracing::debug!(
                "image sweep at launch: {} rows removed, {} evicted, {} files removed",
                report.removed_rows,
                report.evicted,
                report.removed_files
            ),
            Err(e) => tracing::warn!("the image sweep at launch failed: {e}"),
        }
    });
}
