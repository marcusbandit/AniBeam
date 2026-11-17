//! The library team (#854): the sources, the scans and the watcher over
//! them, and the reads that turn the tables into cards.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::reads::KeptLoad;
use crate::scan::{self, LibraryState, ScanScope};
use crate::watcher::{self, Trigger, Watcher};
use anibeam_contract::{CoreError, SeriesCard, SeriesDetail};
use anibeam_foundation::above::Above;
use anibeam_foundation::events::Signal;
use anibeam_foundation::services::Services;
use anibeam_foundation::sync::recover;
use anibeam_net::Net;

/// What the library asks of the team that brings files into it, the
/// downloads above it (#854). The downloads implement it and `Core::open`
/// links them to `Library::incoming`, so the library names no team above it.
pub trait Incoming: Send + Sync {
    /// Lays what a torrent is still writing over these cards.
    fn overlay(self: Arc<Self>, cards: &mut [SeriesCard]);
    /// The same over one series' detail, its episodes marked as well.
    fn overlay_detail(self: Arc<Self>, detail: &mut SeriesDetail);
    /// Whether a torrent is still writing this file.
    fn is_writing(&self, path: &str) -> bool;
    /// These files went to the trash: a torrent that held one leaves the
    /// client too (#266).
    fn files_gone(self: Arc<Self>, gone: Vec<std::path::PathBuf>);
    /// A scan brought files in: where each came from is recorded (#241).
    fn files_arrived(self: Arc<Self>);
}

pub struct Library {
    pub(crate) services: Arc<Services>,
    pub(crate) net: Arc<Net>,
    /// The library's in-memory state: the movie folders each source's walk
    /// found, the watcher's queue of paths, and the settle timers.
    pub state: LibraryState,
    /// Built by `Core::start` and dropped first by `Core::shutdown`. `None`
    /// before `start` and after `shutdown`, so a call that arrives on either
    /// side of the core's life finds nothing to watch with rather than
    /// failing.
    watcher: Mutex<Option<Arc<Watcher>>>,
    /// The core's own shut-down flag, so a watcher trigger that lands while
    /// the core is going away starts nothing.
    closed: Arc<AtomicBool>,
    /// Off unless a shell turns it on: the whole-library load is kept for
    /// the rest of its state generation (`library::reads`). Off by default
    /// because a test that writes rows behind the core's back emits no
    /// event and would read its own stale load.
    pub snapshot_cache: AtomicBool,
    pub kept_load: Mutex<Option<KeptLoad>>,
    /// What is still downloading, for the cards, and the torrents a delete
    /// takes with it: the downloads, through the library's own trait.
    pub incoming: Above<dyn Incoming>,
}

impl Library {
    pub fn new(services: Arc<Services>, net: Arc<Net>, closed: Arc<AtomicBool>) -> Library {
        Library {
            services,
            net,
            state: LibraryState::default(),
            watcher: Mutex::new(None),
            closed,
            snapshot_cache: AtomicBool::new(false),
            kept_load: Mutex::new(None),
            incoming: Above::default(),
        }
    }

    fn watcher(&self) -> MutexGuard<'_, Option<Arc<Watcher>>> {
        recover(&self.watcher)
    }

    /// Puts the watcher in its slot, empty. Installing a watch means walking
    /// every directory under a root, which is disk work at library scale, so
    /// it belongs to the scans rather than to the caller.
    pub fn set_watcher(&self, watcher: Watcher) {
        *self.watcher() = Some(Arc::new(watcher));
    }

    /// Takes the watcher out of its slot and stops it. Taken out of the mutex
    /// before it is stopped: the debouncer's thread dispatches into this
    /// team, so it has to be gone before the runtime and the store are.
    pub fn stop_watcher(&self) {
        let watcher = self.watcher().take();
        if let Some(watcher) = watcher {
            watcher.stop();
        }
    }

    /// Installs the recursive watch on a source. Called by the Scan job,
    /// off the runtime's blocking pool, because notify installs one by
    /// walking every directory under the root. Nothing to do before `start`
    /// has built the watcher: the launch's catch-up scan is what puts the
    /// first watch on every source.
    pub(crate) fn install_watch(&self, path: &str) -> Result<(), CoreError> {
        // Cloned out of the slot and the guard dropped before the walk:
        // the watcher keeps its own lock around the notify call, so this
        // one is held for a clone and nothing more.
        let watcher = self.watcher().clone();
        match watcher {
            Some(watcher) => watcher.watch(path),
            None => Ok(()),
        }
    }

    pub(crate) fn unwatch_source(&self, path: &str) {
        let watcher = self.watcher().clone();
        if let Some(watcher) = watcher {
            watcher.unwatch(path);
            // The presence watches may wait on an install walk, and a call
            // answers in milliseconds, so they go off this thread.
            let path = path.to_string();
            self.services
                .handle
                .spawn_blocking(move || watcher.forget_source(&path));
        }
    }

    /// A source's folder is not there: watch above it for its return
    /// (#1191). Called by the Scan job, off the runtime's blocking pool.
    pub(crate) fn await_source(self: &Arc<Self>, path: &str) {
        let watcher = self.watcher().clone();
        if let Some(watcher) = watcher
            && !watcher.await_source(path)
        {
            self.on_watch_triggers(vec![Trigger::Returned(path.to_string())]);
        }
    }

    /// A source read fine: stop waiting for it and watch for it leaving.
    /// Called by the Scan job, off the runtime's blocking pool.
    pub(crate) fn source_present(&self, path: &str) {
        let watcher = self.watcher().clone();
        if let Some(watcher) = watcher {
            watcher.source_present(path);
        }
    }

    /// The watcher's whole reach into the core, called on notify's own
    /// thread: it queues the paths and asks for a scan, both of which are
    /// a lock and a spawn. Everything that touches the disk happens in the
    /// job that comes out of it.
    pub(crate) fn on_watch_triggers(self: &Arc<Self>, triggers: Vec<Trigger>) {
        if self.closed.load(Ordering::SeqCst) {
            return;
        }
        let mut paths: Vec<String> = Vec::new();
        let mut rescan = false;
        for trigger in triggers {
            match trigger {
                Trigger::Rescan => rescan = true,
                // A file speaks for the folder it is in: that is the series
                // the reconcile has to look at, whether the file arrived or
                // went away.
                Trigger::Ingest(p) | Trigger::Removed(p) => {
                    paths.push(watcher::parent_series_path(&p))
                }
                // A source that came back or went away is scanned, and the
                // scan is what flips it, says so and moves its watches.
                Trigger::NewDirectory(p) | Trigger::Returned(p) | Trigger::Gone(p) => paths.push(p),
                Trigger::Closer(p) => {
                    let library = self.clone();
                    self.services
                        .handle
                        .spawn_blocking(move || library.await_source(&p));
                }
                // Comes back through here as an Ingest once it is written.
                Trigger::Settling(p) => self.settle_file(p),
            }
        }
        if paths.is_empty() && !rescan {
            return;
        }
        self.state.push_pending(paths);
        // A full scan covers every queued path, so the job takes them off
        // the queue itself rather than being asked for them twice.
        let scope = if rescan {
            ScanScope::All
        } else {
            ScanScope::Paths(Vec::new())
        };
        scan::start(self, scope);
    }

    /// Waits, on the runtime, for a file a backend without close-write
    /// reported to stop growing, then hands it back as an ordinary ingest.
    /// One wait per path: a file being copied reports again and again.
    fn settle_file(self: &Arc<Self>, path: String) {
        if !recover(&self.state.settling).insert(path.clone()) {
            return;
        }
        let library = Arc::downgrade(self);
        self.services.handle.spawn(async move {
            let written = watcher::size_holds(
                std::path::Path::new(&path),
                watcher::SETTLE_POLL,
                watcher::SETTLE_HOLD,
            )
            .await;
            let Some(library) = library.upgrade() else {
                return;
            };
            recover(&library.state.settling).remove(&path);
            if written {
                library.on_watch_triggers(vec![Trigger::Ingest(path)]);
            }
        });
    }

    /// A folder has stopped changing, so it is worth a match. The entry
    /// goes first: the candidate query skips whatever is still armed, so a
    /// job already running picks this series up on its next time round the
    /// loop precisely because it no longer is.
    pub(crate) fn settle_fired(&self, series_id: u64) {
        recover(&self.state.settle).remove(&series_id);
        self.services.bus.signal(Signal::FolderSettled);
    }
}
