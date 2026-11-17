//! What the core does once, at launch, and never again until the next one.
//!
//! One catch-up walk of the library, and behind it the jobs a fresh start
//! owes: the match, the backfill, the crawl, the airing schedules, the
//! IMDb ratings, the tracker lists, the images. Nothing here is on a timer.
//! The scan's own terminal event is the trigger, so the list runs when the
//! library is actually on the tables rather than at some guessed moment
//! after it.
//!
//! Terminal means terminal, whatever it says: a scan the user cancelled from
//! the job list and a scan that failed both end the launch's waiting exactly
//! as a finished one does. The rest of the list is what the session owes
//! either way, and losing all of it to one cancelled walk would be the worse
//! bug by far.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::contract::*;
use crate::core::Core;
use crate::downloads::Downloads;
use crate::events::Subscription;
use crate::franchise::crawl;
use crate::images;
use crate::library::scan::{self, ScanScope};
use crate::metadata::{Metadata, airing, apply, automatch, imdb, owed};
use crate::sync::recover;
use crate::trackers::{Trackers, cache};

/// Where `Core` keeps the launch's one-shot listener while it waits.
pub(crate) type Slot = Mutex<Option<Arc<Subscription>>>;

/// The catch-up walk, and the one-shot listener that turns its end into the
/// rest of the launch. Called by `Core::start` once the watcher is up.
///
/// The scan installs the recursive watch on every source itself, at the top
/// of its own loop, so this is also what puts the library under the watcher.
///
/// Nothing here says the machine has no keyring: whichever caller finishes
/// the probe writes that line, and `Core::start`'s warm-up is one of them.
pub fn start(core: &Core, slot: &Arc<Slot>) -> Result<(), CoreError> {
    // Registered before the scan starts, not after: a library with no
    // sources at all has nothing to walk, so its scan can be over before
    // `scan::start` has even returned an id here, and a listener added
    // afterwards would have missed the one event it exists for.
    let listener = Arc::new(AfterScan {
        slot: Arc::downgrade(slot),
        metadata: Arc::downgrade(&core.metadata),
        trackers: Arc::downgrade(&core.trackers),
        downloads: Arc::downgrade(&core.downloads),
        scan: AtomicU64::new(0),
        fired: AtomicBool::new(false),
    });
    *recover(slot) = Some(Subscription::new(
        core.services.bus.clone(),
        listener.clone(),
    ));
    listener.latch(scan::start(&core.library, ScanScope::All));
    Ok(())
}

/// Takes the launch listener out and drops it, which unsubscribes it.
/// Called by the listener itself once it has fired, and by `shutdown`
/// for the launch that never got its scan.
pub(crate) fn take(slot: &Slot) {
    // Taken out of the mutex and dropped by this `let` rather than
    // inside the lock's own statement: the drop unsubscribes, which
    // takes the bus's listener lock, and that should never happen with
    // this one held.
    let sub = recover(slot).take();
    drop(sub);
}

/// Waits for the catch-up scan to end and then runs the list, once. It holds
/// `Weak`s, so a core that goes away mid-scan takes this with it rather
/// than keeping itself alive through the bus.
struct AfterScan {
    slot: Weak<Slot>,
    metadata: Weak<Metadata>,
    trackers: Weak<Trackers>,
    downloads: Weak<Downloads>,
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
        let Some(slot) = self.slot.upgrade() else {
            return;
        };
        // Dropping the handle unsubscribes, so the bus stops calling this
        // the moment the launch has had its one scan.
        take(&slot);
        let (Some(metadata), Some(trackers), Some(downloads)) = (
            self.metadata.upgrade(),
            self.trackers.upgrade(),
            self.downloads.upgrade(),
        ) else {
            return;
        };
        after_scan(&metadata, &trackers, &downloads);
    }
}

/// The list, in this order, each started without waiting for the one before
/// it: they run one at a time per kind and pace themselves against their own
/// upstream, so the order is the order they get their turn in, not a queue
/// anything blocks on.
fn after_scan(metadata: &Arc<Metadata>, trackers: &Arc<Trackers>, downloads: &Arc<Downloads>) {
    automatch::start(metadata);
    apply::backfill_stubs(metadata);
    crawl::start_gap_crawl(metadata);
    airing::start_refresh_library(metadata);
    // The titles a Jikan outage left owed, asked for again (#1065).
    owed::start(metadata);
    imdb::start_refresh(&metadata.services);
    cache::start_refresh(trackers, None, false);
    images::start_fill(&metadata.net);
    // The followed shows' next episodes, from the rules last read (#215)
    crate::downloads::watch::arm(downloads);

    // The sweep is bookkeeping rather than a job: one transaction over the
    // rows and one walk of the image directory, so it goes on the runtime
    // with nothing waiting on it.
    let net = metadata.net.clone();
    metadata.services.handle.spawn(async move {
        images::sweep_after(&net, "a launch").await;
    });
}
