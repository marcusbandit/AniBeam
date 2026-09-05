//! The watcher: one recursive notify watch per available source, debounced,
//! turned into scan triggers on notify's own thread.
//!
//! The handler here does no work of its own beyond classifying: it upgrades
//! its `Weak<Core>`, hands the triggers to `Core::on_watch_triggers`, and
//! returns. Everything after that is a Scan job on the runtime, so notify's
//! event thread is never the thing waiting on a disk walk.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, TryLockError, Weak};
use std::time::Duration;

use notify::event::{AccessKind, AccessMode, CreateKind, EventKind, Flag, ModifyKind, RenameMode};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache, new_debouncer,
};

use crate::contract::*;
use crate::core::Core;
use crate::library::walk::{is_ignored_name, is_video};
use crate::sync::recover;

/// How long the debouncer waits for a path to go quiet before it reports.
/// notify ticks at a quarter of this on its own.
pub const DEBOUNCE: Duration = Duration::from_millis(500);

/// What one filesystem event means to the library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// A video finished being written and should be taken in.
    Ingest(String),
    /// A video went away.
    Removed(String),
    /// A directory appeared: walk it, since the files already inside it may
    /// have landed before notify had a watch on it.
    NewDirectory(String),
    /// Events were missed, or the OS ran out of watches. Scan everything.
    Rescan,
}

/// A dot entry, or the name a downloader gives an unfinished file, anywhere
/// below the watched root. The root's own components are exempt: the walk
/// treats a source under a hidden directory like any other source, and a
/// watcher that disagreed would go silent for the whole library. Roots are
/// matched longest first, so nested-looking roots cannot shorten each other.
fn ignored(p: &Path, roots: &[String]) -> bool {
    let below = roots
        .iter()
        .filter_map(|r| p.strip_prefix(r).ok().map(|rel| (r.len(), rel)))
        .max_by_key(|(len, _)| *len)
        .map_or(p, |(_, rel)| rel);
    below
        .components()
        .any(|c| is_ignored_name(&c.as_os_str().to_string_lossy()))
}

fn name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn text(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// The pure half of the watcher: one debounced event to the triggers it
/// means. `roots` are the watched source paths, used only to decide which
/// components an ignored name applies to.
pub fn classify(event: &DebouncedEvent, roots: &[String]) -> Vec<Trigger> {
    // Set when the backend knows it dropped events, and by notify's own
    // `MaxFilesWatch` path. Nothing else about the event is trustworthy
    // then, so the whole library is what gets looked at.
    if event.flag() == Some(Flag::Rescan) {
        return vec![Trigger::Rescan];
    }
    let kept: Vec<&PathBuf> = event.paths.iter().filter(|p| !ignored(p, roots)).collect();
    match event.kind {
        // The write is finished: this, and the rename below, are the only
        // two ways a file becomes ready. `Create` never is, since the file
        // it announces is usually still being filled.
        EventKind::Access(AccessKind::Close(AccessMode::Write))
        | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => kept
            .into_iter()
            .filter(|p| is_video(&name(p)))
            .map(|p| Trigger::Ingest(text(p)))
            .collect(),
        // Both halves of a rename in one event: the old path leaves the
        // library and the new one joins it, so it is two triggers.
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            let mut out = Vec::new();
            if let Some(from) = event
                .paths
                .first()
                .filter(|p| !ignored(p, roots) && is_video(&name(p)))
            {
                out.push(Trigger::Removed(text(from)));
            }
            if let Some(to) = event.paths.get(1).filter(|p| !ignored(p, roots)) {
                if is_video(&name(to)) {
                    out.push(Trigger::Ingest(text(to)));
                } else if to.is_dir() {
                    out.push(Trigger::NewDirectory(text(to)));
                }
            }
            out
        }
        // A gone path can no longer be stat'd, so a directory is told apart
        // from a file by having no extension rather than by asking the
        // filesystem; either way the series above it wants a look.
        EventKind::Modify(ModifyKind::Name(RenameMode::From)) | EventKind::Remove(_) => kept
            .into_iter()
            .filter(|p| is_video(&name(p)) || p.extension().is_none())
            .map(|p| Trigger::Removed(text(p)))
            .collect(),
        EventKind::Create(CreateKind::Folder) => kept
            .into_iter()
            .map(|p| Trigger::NewDirectory(text(p)))
            .collect(),
        // A backend that does not say what it created: only a directory is
        // worth walking, and a directory is still there to be asked.
        EventKind::Create(_) => kept
            .into_iter()
            .filter(|p| p.is_dir())
            .map(|p| Trigger::NewDirectory(text(p)))
            .collect(),
        _ => Vec::new(),
    }
}

/// The series a file belongs to is its folder, so a file event is reported
/// against the directory above it. A path with no parent speaks for itself.
pub fn parent_series_path(p: &str) -> String {
    Path::new(p)
        .parent()
        .map(text)
        .unwrap_or_else(|| p.to_string())
}

/// The macOS fallback for a backend with no close-write. Not built on Linux,
/// and never on an exported item.
///
/// FSEvents reports `Create` and `Modify(Data)` and never says a write is
/// finished, so readiness has to be inferred from the size holding still.
/// The Mac shell's own task builds that; until then a file already there
/// when a directory is walked is still taken in, and a file written into a
/// watched folder is picked up by the next scan.
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
fn size_stable_fallback(_paths: &[PathBuf]) -> Vec<Trigger> {
    Vec::new()
}

/// One debouncer and the roots it watches. Dropping this stops the watch.
pub struct Watcher {
    /// `None` once `stop` has taken it. `Debouncer::stop` consumes itself,
    /// which is why the field is an option rather than the debouncer.
    ///
    /// Behind its own lock, taken only around a call into notify. Notify
    /// wants `&mut` and installs a recursive watch by walking every
    /// directory under the root, which is disk work at library scale, so
    /// this lock is held for as long as that walk takes. Keeping it here
    /// rather than on the core's slot is what stops everything else that
    /// touches the watcher from waiting on the walk.
    debouncer: Mutex<Option<Debouncer<RecommendedWatcher, RecommendedCache>>>,
    /// Shared with the handler, which needs them to tell a hidden entry in
    /// the library from a hidden directory the library merely sits under.
    /// Never held across a call into the debouncer, so it can never take
    /// part in a lock cycle with notify's own.
    roots: Arc<Mutex<Vec<String>>>,
    /// Paths a `RemoveSource` asked to drop while an install walk had the
    /// debouncer. Notify wants `&mut`, so an unwatch cannot run beside a
    /// watch, and a call has to answer in milliseconds rather than wait for
    /// a library-scale walk: the path waits here instead, and whichever
    /// call next holds the debouncer drops it.
    pending_unwatch: Mutex<Vec<String>>,
}

/// Every queued drop, applied. `installed` is the path the caller has just
/// watched, if any: a drop queued before that install is superseded by the
/// caller asking for the watch again, so it is thrown away rather than
/// undoing the install.
fn drain_unwatches(
    pending: &Mutex<Vec<String>>,
    debouncer: &mut Debouncer<RecommendedWatcher, RecommendedCache>,
    installed: Option<&str>,
) {
    for waiting in std::mem::take(&mut *recover(pending)) {
        if installed == Some(waiting.as_str()) {
            continue;
        }
        let _ = debouncer.unwatch(Path::new(&waiting));
    }
}

/// One drop against a debouncer that may already have been stopped.
fn drop_watch(debouncer: Option<&mut Debouncer<RecommendedWatcher, RecommendedCache>>, path: &str) {
    if let Some(debouncer) = debouncer {
        let _ = debouncer.unwatch(Path::new(path));
    }
}

impl Watcher {
    pub fn new(core: Weak<Core>) -> Result<Watcher, CoreError> {
        let roots: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let handler_roots = roots.clone();
        let debouncer = new_debouncer(DEBOUNCE, None, move |result: DebounceEventResult| {
            // The core is already going away: there is nothing left to tell.
            let Some(core) = core.upgrade() else { return };
            match result {
                Ok(events) => {
                    let roots = recover(&handler_roots).clone();
                    let triggers: Vec<Trigger> =
                        events.iter().flat_map(|e| classify(e, &roots)).collect();
                    if triggers.is_empty() {
                        return;
                    }
                    core.on_watch_triggers(triggers);
                }
                Err(errors) => {
                    for e in errors {
                        core.bus.warn(
                            Stage::Library,
                            format!("watcher error: {e}"),
                            EventBody::Notice,
                        );
                        // Out of inotify watches: what is watched from here
                        // on is arbitrary, so the library is re-read once
                        // and the user has the warning above to act on.
                        if matches!(e.kind, notify::ErrorKind::MaxFilesWatch) {
                            core.on_watch_triggers(vec![Trigger::Rescan]);
                        }
                    }
                }
            }
        })
        .map_err(|e| CoreError::internal(format!("watcher: {e}")))?;
        Ok(Watcher {
            debouncer: Mutex::new(Some(debouncer)),
            roots,
            pending_unwatch: Mutex::new(Vec::new()),
        })
    }

    /// Installs one recursive watch on a source. Idempotent per root, which
    /// matters because every Scan job asks for it again.
    pub fn watch(&self, path: &str) -> Result<(), CoreError> {
        if recover(&self.roots).iter().any(|r| r == path) {
            return Ok(());
        }
        {
            let mut held = self.debouncer();
            let Some(debouncer) = held.as_mut() else {
                return Ok(());
            };
            let installed = debouncer
                .watch(Path::new(path), RecursiveMode::Recursive)
                .map_err(|e| CoreError::Io {
                    path: Some(path.to_string()),
                    message: e.to_string(),
                });
            // Whatever the install did, this thread is holding the lock the
            // queued drops could not get, so it is the one that owes them.
            drain_unwatches(&self.pending_unwatch, debouncer, Some(path));
            installed?;
        }
        recover(&self.roots).push(path.to_string());
        Ok(())
    }

    /// Drops a source's watch, without ever waiting for one to be installed.
    ///
    /// `RemoveSource` is a call, and a call answers in milliseconds; an
    /// install walk can hold the debouncer for as long as a library takes to
    /// walk. So the roots list is corrected at once, and the notify half is
    /// either done now or left for whoever has the lock. A path that was
    /// never watched, or a watch the OS already dropped with the directory,
    /// is not an error worth raising.
    pub fn unwatch(&self, path: &str) {
        // First and under its own lock, whichever way the debouncer goes:
        // this list is what `watch` tests for idempotence, so a path still
        // in it could never be watched again.
        recover(&self.roots).retain(|r| r != path);
        match self.debouncer.try_lock() {
            Ok(mut held) => drop_watch(held.as_mut(), path),
            Err(TryLockError::Poisoned(held)) => drop_watch(held.into_inner().as_mut(), path),
            Err(TryLockError::WouldBlock) => recover(&self.pending_unwatch).push(path.to_string()),
        }
    }

    /// Stops the debouncer and waits for its thread, so nothing can still be
    /// dispatching into a core that is shutting down. A second call does
    /// nothing. This one may wait for an install walk: it joins the
    /// debouncer's own thread anyway, and it is the end of the core's life
    /// rather than a call anybody is waiting on.
    pub fn stop(&self) {
        let debouncer = self.debouncer().take();
        if let Some(mut debouncer) = debouncer {
            drain_unwatches(&self.pending_unwatch, &mut debouncer, None);
            debouncer.stop();
        }
        recover(&self.pending_unwatch).clear();
        recover(&self.roots).clear();
    }

    /// How many drops are waiting on an install walk. For the test that
    /// proves an unwatch never blocks.
    #[cfg(test)]
    fn pending_unwatch_len(&self) -> usize {
        recover(&self.pending_unwatch).len()
    }

    fn debouncer(
        &self,
    ) -> std::sync::MutexGuard<'_, Option<Debouncer<RecommendedWatcher, RecommendedCache>>> {
        self.debouncer.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::Event as RawEvent;
    use notify::event::RemoveKind;
    use std::time::Instant;

    fn event(kind: EventKind, paths: &[&str]) -> DebouncedEvent {
        let mut e = RawEvent::new(kind);
        for p in paths {
            e = e.add_path(PathBuf::from(*p));
        }
        DebouncedEvent::new(e, Instant::now())
    }

    /// The debouncer lives behind the watcher's own lock, so a watcher is
    /// driven through a shared reference. That is what keeps the core's
    /// slot free: it is held for a clone and never for the recursive
    /// install walk, which is disk work at library scale and used to leave
    /// RemoveSource and shutdown waiting behind it. This test only compiles
    /// while `watch`, `unwatch` and `stop` take `&self`.
    #[test]
    fn a_watcher_is_driven_through_a_shared_reference() {
        let dir = tempfile::tempdir().unwrap();
        let deep = dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        let other = dir.path().join("other");
        std::fs::create_dir_all(&other).unwrap();

        let watcher = Arc::new(Watcher::new(Weak::new()).unwrap());
        let installing = Arc::clone(&watcher);
        let root = dir.path().to_string_lossy().into_owned();
        let installer = std::thread::spawn(move || installing.watch(&root));
        // Nothing here takes the installer's reference away from it.
        watcher.unwatch(&other.to_string_lossy());
        installer.join().unwrap().unwrap();

        watcher.watch(&dir.path().to_string_lossy()).unwrap();
        watcher.stop();
        // A stopped watcher answers rather than failing, so a scan that
        // outlives shutdown asks for nothing it cannot have.
        watcher.watch(&deep.to_string_lossy()).unwrap();
    }

    /// An unwatch must answer in milliseconds: RemoveSource is a call on the
    /// shell's own thread, and notify installs a recursive watch by walking
    /// every directory under the root, which is disk work at library scale.
    /// A path whose drop cannot get the debouncer leaves the roots at once
    /// and waits for whoever is holding it.
    #[test]
    fn an_unwatch_never_waits_on_an_install_walk() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b, c) = (
            dir.path().join("a"),
            dir.path().join("b"),
            dir.path().join("c"),
        );
        for p in [&a, &b, &c] {
            std::fs::create_dir_all(p).unwrap();
        }
        let (a, b, c) = (
            a.to_string_lossy().into_owned(),
            b.to_string_lossy().into_owned(),
            c.to_string_lossy().into_owned(),
        );

        let watcher = Arc::new(Watcher::new(Weak::new()).unwrap());
        watcher.watch(&a).unwrap();
        watcher.watch(&b).unwrap();

        // Another thread holds the debouncer, exactly as an install walk
        // does for as long as the walk takes.
        let holder = Arc::clone(&watcher);
        let (release, wait_for_release) = std::sync::mpsc::channel::<()>();
        let (holding, held) = std::sync::mpsc::channel::<()>();
        let hold = std::thread::spawn(move || {
            let _guard = holder.debouncer();
            holding.send(()).unwrap();
            let _ = wait_for_release.recv();
        });
        held.recv().unwrap();

        let began = Instant::now();
        watcher.unwatch(&b);
        let waited = began.elapsed();
        assert!(
            waited < Duration::from_millis(50),
            "the unwatch waited {waited:?}"
        );
        assert!(
            !recover(&watcher.roots).contains(&b),
            "the root outlived its unwatch"
        );
        assert_eq!(watcher.pending_unwatch_len(), 1);

        release.send(()).unwrap();
        hold.join().unwrap();

        // The next install drains the queue, and an install of the very
        // path that was queued wins: the caller asked for the watch back.
        watcher.watch(&b).unwrap();
        assert_eq!(watcher.pending_unwatch_len(), 0);
        assert!(recover(&watcher.roots).contains(&b));

        // A drop of another path is applied by the next install rather than
        // thrown away with it.
        let holder = Arc::clone(&watcher);
        let (release, wait_for_release) = std::sync::mpsc::channel::<()>();
        let (holding, held) = std::sync::mpsc::channel::<()>();
        let hold = std::thread::spawn(move || {
            let _guard = holder.debouncer();
            holding.send(()).unwrap();
            let _ = wait_for_release.recv();
        });
        held.recv().unwrap();
        watcher.unwatch(&a);
        assert_eq!(watcher.pending_unwatch_len(), 1);
        release.send(()).unwrap();
        hold.join().unwrap();

        watcher.watch(&c).unwrap();
        assert_eq!(watcher.pending_unwatch_len(), 0);
        assert!(!recover(&watcher.roots).contains(&a));
        watcher.stop();
    }

    #[test]
    fn a_finished_write_of_a_video_is_an_ingest() {
        let e = event(
            EventKind::Access(AccessKind::Close(AccessMode::Write)),
            &["/lib/a/ep.mkv"],
        );
        assert_eq!(
            classify(&e, &[]),
            vec![Trigger::Ingest("/lib/a/ep.mkv".into())]
        );
    }

    #[test]
    fn an_unfinished_download_is_not_an_ingest() {
        let e = event(
            EventKind::Access(AccessKind::Close(AccessMode::Write)),
            &["/lib/a/ep.mkv.part"],
        );
        assert!(classify(&e, &[]).is_empty());
    }

    #[test]
    fn a_rename_of_a_video_removes_the_old_path_and_ingests_the_new_one() {
        let e = event(
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
            &["/lib/a/from.mkv", "/lib/a/to.mkv"],
        );
        assert_eq!(
            classify(&e, &[]),
            vec![
                Trigger::Removed("/lib/a/from.mkv".into()),
                Trigger::Ingest("/lib/a/to.mkv".into())
            ]
        );
    }

    #[test]
    fn a_rename_into_the_library_is_an_ingest_on_its_own() {
        let e = event(
            EventKind::Modify(ModifyKind::Name(RenameMode::To)),
            &["/lib/a/ep.mkv"],
        );
        assert_eq!(
            classify(&e, &[]),
            vec![Trigger::Ingest("/lib/a/ep.mkv".into())]
        );
    }

    #[test]
    fn a_deleted_video_is_a_removal() {
        let e = event(EventKind::Remove(RemoveKind::File), &["/lib/a/ep.mkv"]);
        assert_eq!(
            classify(&e, &[]),
            vec![Trigger::Removed("/lib/a/ep.mkv".into())]
        );
    }

    /// A gone directory has no extension to go by and cannot be stat'd, so
    /// it is reported as a removal and the scan decides what it meant.
    #[test]
    fn a_deleted_folder_is_a_removal() {
        let e = event(EventKind::Remove(RemoveKind::Folder), &["/lib/a"]);
        assert_eq!(classify(&e, &[]), vec![Trigger::Removed("/lib/a".into())]);
    }

    #[test]
    fn a_new_folder_is_walked() {
        let e = event(EventKind::Create(CreateKind::Folder), &["/lib/b"]);
        assert_eq!(
            classify(&e, &[]),
            vec![Trigger::NewDirectory("/lib/b".into())]
        );
    }

    #[test]
    fn a_created_file_is_never_an_ingest() {
        let e = event(EventKind::Create(CreateKind::File), &["/lib/a/ep.mkv"]);
        assert!(classify(&e, &[]).is_empty());
    }

    #[test]
    fn the_rescan_flag_is_a_full_scan() {
        let e = DebouncedEvent::new(
            RawEvent::new(EventKind::Other).set_flag(Flag::Rescan),
            Instant::now(),
        );
        assert_eq!(classify(&e, &[]), vec![Trigger::Rescan]);
    }

    #[test]
    fn a_dot_entry_below_a_watched_root_is_dropped_but_a_dotted_root_is_not() {
        let hidden = event(
            EventKind::Access(AccessKind::Close(AccessMode::Write)),
            &["/tmp/.tmpXY/lib/.trash/ep.mkv"],
        );
        assert!(classify(&hidden, &["/tmp/.tmpXY/lib".to_string()]).is_empty());
        let under_a_dotted_root = event(
            EventKind::Access(AccessKind::Close(AccessMode::Write)),
            &["/tmp/.tmpXY/lib/a/ep.mkv"],
        );
        assert_eq!(
            classify(&under_a_dotted_root, &["/tmp/.tmpXY/lib".to_string()]),
            vec![Trigger::Ingest("/tmp/.tmpXY/lib/a/ep.mkv".into())]
        );
    }

    #[test]
    fn a_files_series_is_the_folder_above_it() {
        assert_eq!(parent_series_path("/lib/Show/ep01.mkv"), "/lib/Show");
        assert_eq!(parent_series_path("/"), "/");
    }
}
