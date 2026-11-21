//! The phone's torrent engine: librqbit inside the core, writing into the library folder.
//! It answers in qBittorrent's shapes (`Torrent`, its state words, file priorities) so the
//! pages, the library overlay and the rolling downloads (`downloads::rolling`) read it the
//! way they read the desktop's client. Its bytes go through `storage`, which keeps the files
//! a torrent skips out of the library.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, Session, SessionOptions,
    SessionPersistenceConfig,
};
use serde::{Deserialize, Serialize};

use super::storage::Shelf;
use crate::rolling::{NORMAL, SKIP};
use anibeam_contract::*;
use anibeam_foundation::sync::recover;

/// What the engine itself does not keep: each torrent's title, folder and times, and
/// whether the person paused it (a hold for Wi-Fi resumes only the others).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Entry {
    title: String,
    folder: String,
    added_on: i64,
    completion_on: i64,
    paused_by_user: bool,
    /// Each file's priority in qBittorrent's numbers, when any is not normal: 0 skips it.
    /// The engine itself only knows fetched or not; the numbers are kept so a rolling
    /// download reads back what it set.
    #[serde(default)]
    priorities: Vec<i64>,
    /// How many pauses and resumes the person gave it in this run, so an add can tell a
    /// pause given while it ran from one an old entry carried.
    #[serde(skip)]
    asks: u64,
}

impl Entry {
    fn priority(&self, file: usize) -> i64 {
        self.priorities.get(file).copied().unwrap_or(NORMAL)
    }

    /// The files fetched, None when that is all of them.
    fn wanted(&self) -> Option<HashSet<usize>> {
        self.priorities.contains(&SKIP).then(|| {
            self.priorities
                .iter()
                .enumerate()
                .filter(|(_, p)| **p != SKIP)
                .map(|(i, _)| i)
                .collect()
        })
    }
}

/// What an add did: the torrent's info hash, and whether this add is what brought it in.
/// `new` is false for one the engine already held, got by hand or by an earlier round, so
/// a subscription never counts it as its own (#654).
pub struct Added {
    pub hash: String,
    pub new: bool,
}

/// A torrent's file list read before it is added, and what the add needs to start from it
/// without asking the peers again.
pub struct Peeked {
    pub files: Vec<TorrentFile>,
    bytes: Vec<u8>,
    peers: Vec<std::net::SocketAddr>,
    trackers: Vec<String>,
    hash: String,
}

pub struct Engine {
    session: Arc<Session>,
    shelf: Shelf,
    root: PathBuf,
    entries_path: PathBuf,
    /// Held from taking the list to its landing on disk, so saves land in the order
    /// their lists were taken (#582), while the list itself is let go at once.
    saving: Mutex<()>,
    /// The hold, the list with the person's pauses, and the turn that starts or stops one.
    wants: Wants,
    /// Wakes the tick early after anything that may start a torrent or change what is wanted.
    nudge: Arc<tokio::sync::Notify>,
    /// Follows each check to its end.
    watch: Watch,
    keeper: Keeper,
}

/// How often, while a torrent runs, each torrent is set to what the hold and the person
/// want. The pieces are written down every third tick (`Keeper::every`), at a hold and a
/// pause, and when a check ends.
const TICK: std::time::Duration = std::time::Duration::from_secs(5);

/// The same while a torrent checks its files, so one librqbit settles against what it was
/// asked during the check is set right soon after (#1186).
const CHECKING_TICK: std::time::Duration = std::time::Duration::from_millis(500);

/// Which torrents in librqbit's error state `settle` starts again; starting one makes
/// librqbit check it afresh.
#[derive(Clone, Copy, PartialEq)]
enum Retry {
    /// Only one whose first check a stop and a start that came close together broke: the
    /// check saw the stop, the start cleared it, and librqbit took the stopped check for a
    /// failed one.
    Broken,
    /// Those too that the hold had paused: lifting the hold starts them again.
    Held,
    /// Any: the person asked for it.
    Any,
}

/// Whether librqbit's error is a first check stopped by a pause that a start then took back.
/// The words are the ones librqbit 9.0's `initial_check` (`file_ops.rs`) stops with; a
/// librqbit upgrade needs them looked at again.
fn broken_check(error: Option<&str>) -> bool {
    error.is_some_and(|e| e.contains("initial check paused"))
}

/// What decides whether a torrent runs: the hold and the person's own pause, and the turn
/// everything that starts or stops a torrent takes, so each reads both and acts on them
/// before the next one does.
#[derive(Clone)]
struct Wants {
    session: Arc<Session>,
    held: Arc<Mutex<bool>>,
    entries: Arc<Mutex<HashMap<String, Entry>>>,
    steering: Arc<tokio::sync::Mutex<()>>,
    /// The engine's piece records, dropped before a start that begins a check.
    keeper: Keeper,
}

impl Wants {
    /// Starts or stops one torrent so it matches the hold and the person's pause, and pauses
    /// a finished one that still runs.
    /// It goes by librqbit's state, never its paused flag alone: librqbit keeps the flag it
    /// was last asked for, and an ask that lands while it still checks the torrent's files
    /// changes the flag but not what the torrent does once the check is done (#1186). That
    /// ask is still worth making: a check not running yet takes it, and the tick sets the
    /// torrent right once a running one ends. Run with `steering` held.
    async fn settle(&self, h: &Arc<librqbit::ManagedTorrent>, retry: Retry) -> anyhow::Result<()> {
        self.settle_reading(h, retry).await.1.map(|_| ())
    }

    /// `settle`, answering the stats it read and whether it asked librqbit for anything, so
    /// the stats stay good for a caller when it did not.
    async fn settle_reading(
        &self,
        h: &Arc<librqbit::ManagedTorrent>,
        retry: Retry,
    ) -> (librqbit::TorrentStats, anyhow::Result<bool>) {
        use librqbit::TorrentStatsState as S;
        let held = *recover(&self.held);
        let by_user = recover(&self.entries)
            .get(&h.info_hash().as_string())
            .is_some_and(|e| e.paused_by_user);
        let stats = h.stats();
        // A phone does not seed on battery and mobile data: a finished torrent still
        // running is paused, as `list` does, also while nothing lists.
        if stats.finished {
            if matches!(stats.state, S::Live) {
                let asked = self.session.pause(h).await;
                return (stats, asked.map(|()| true));
            }
            return (stats, Ok(false));
        }
        let stop = held || by_user;
        let restart = match retry {
            Retry::Any => true,
            Retry::Held => h.is_paused() || broken_check(stats.error.as_deref()),
            Retry::Broken => broken_check(stats.error.as_deref()),
        };
        let asked = match ask(&stats.state, stop, h.is_paused(), restart) {
            Ask::Pause => self.session.pause(h).await,
            Ask::Start => {
                // A start of a torrent in error or still checking begins a check, which can
                // take pieces out of librqbit's record: the engine's record goes first.
                if matches!(stats.state, S::Error | S::Initializing { .. }) {
                    self.keeper.drop_record(&h.info_hash().as_string());
                }
                self.session.unpause(h).await
            }
            // librqbit's start sets its paused flag back before it refuses a torrent
            // already live, and the refusal is the expected answer here.
            Ask::Unflag => {
                let _ = self.session.unpause(h).await;
                Ok(())
            }
            Ask::Nothing => return (stats, Ok(false)),
        };
        (stats, asked.map(|()| true))
    }

    /// Sets every torrent to what is wanted, and answers how soon to look again: soon while
    /// one checks its files, at `every` while one runs, and None when none does, since a
    /// paused, finished or failed torrent changes only when the engine is asked to. The turn
    /// is taken for one torrent at a time, so a tap or a list waits for one, not all.
    ///
    /// Every torrent still checking keeps a pace (`check_pace`), and each one is handed to
    /// `Watch`, which writes its pieces down and wakes the tick the moment its check ends.
    /// `checks` holds how far each check had got at the last tick.
    async fn tick(&self, every: std::time::Duration, checks: &mut HashMap<String, u64>) -> Ticked {
        use librqbit::TorrentStatsState as S;
        let handles: Vec<_> = self
            .session
            .with_torrents(|it| it.map(|(_, h)| h.clone()).collect());
        let mut next = None;
        let mut seen = HashMap::new();
        let mut checking = Vec::new();
        for h in &handles {
            let stats = {
                let _turn = self.steering.lock().await;
                match self.settle_reading(h, Retry::Broken).await {
                    (stats, Ok(false)) => stats,
                    (_, Ok(true)) => h.stats(),
                    (_, Err(e)) => {
                        tracing::warn!("settling {}: {e}", h.info_hash().as_string());
                        h.stats()
                    }
                }
            };
            let key = h.info_hash().as_string();
            let pace = match stats.state {
                S::Initializing { paused } => {
                    let got = stats.progress_bytes;
                    let pace = check_pace(paused, got, checks.get(&key).copied(), every);
                    seen.insert(key, got);
                    // One that stands still is not followed again: the tick's own pace
                    // looks at it, and an ask that starts it follows it anew.
                    if pace < every {
                        checking.push(h.clone());
                    }
                    Some(pace)
                }
                S::Live => Some(every),
                _ => None,
            };
            next = match (next, pace) {
                (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
                (a, b) => a.or(b),
            };
        }
        let ended = checks
            .keys()
            .filter(|k| !seen.contains_key(*k))
            .cloned()
            .collect();
        *checks = seen;
        Ticked {
            next,
            ended,
            checking,
        }
    }
}

/// What `settle` asks librqbit for.
#[derive(Debug, PartialEq)]
enum Ask {
    Pause,
    Start,
    /// Running, and wanted running, but librqbit's paused flag says otherwise: a
    /// fast-resume check asked to pause goes live keeping the flag, and the list would
    /// read the download as paused while it downloads.
    Unflag,
    Nothing,
}

/// What to ask librqbit for a torrent in `state` with its paused flag `flagged`, when the
/// hold or the person wants it stopped (`stop`), and whether one in error may start again.
fn ask(state: &librqbit::TorrentStatsState, stop: bool, flagged: bool, restart: bool) -> Ask {
    use librqbit::TorrentStatsState as S;
    match state {
        S::Live | S::Initializing { paused: false } if stop => Ask::Pause,
        S::Paused | S::Initializing { paused: true } if !stop => Ask::Start,
        S::Error if !stop && restart => Ask::Start,
        S::Live if !stop && flagged => Ask::Unflag,
        _ => Ask::Nothing,
    }
}

/// What a tick answers: how soon to look again, the torrents whose check ended since the
/// last one, and the ones still checking.
struct Ticked {
    next: Option<std::time::Duration>,
    ended: HashSet<String>,
    checking: Vec<Arc<librqbit::ManagedTorrent>>,
}

/// How soon to look again at a torrent checking its files. A running check, or one that
/// moved since the last look, is watched closely. One asked to pause that has not moved is
/// still looked at every `every`: it may be a full check the pause stopped, but it may be a
/// fast-resume check that ignores the pause, or one waiting for librqbit's turn to check
/// (three at a time), and either of those goes live when it ends whatever it was asked.
fn check_pace(
    paused: bool,
    got: u64,
    last: Option<u64>,
    every: std::time::Duration,
) -> std::time::Duration {
    if !paused || last != Some(got) {
        CHECKING_TICK.min(every)
    } else {
        every
    }
}

fn checking(h: &librqbit::ManagedTorrent) -> bool {
    h.with_state(|s| matches!(s, librqbit::ManagedTorrentState::Initializing(_)))
}

/// Follows each torrent's check to its end, then writes its pieces down and wakes the
/// tick. A check can find pieces bad and take them out of librqbit's record; the engine's
/// own record of them is written again as soon as it ends, so the next open never puts them
/// back. And a check that ends running against what was asked during it is set right at
/// once, not at the next tick.
#[derive(Clone)]
struct Watch {
    session: Arc<Session>,
    keeper: Keeper,
    nudge: Arc<tokio::sync::Notify>,
    /// The torrents followed now, each with whether another check began while its task
    /// was deciding to end.
    watching: Arc<Mutex<HashMap<String, bool>>>,
    every: std::time::Duration,
}

/// How many looks a paused check may sit still, having got somewhere, before its task ends.
/// That is a full check the pause stopped, which librqbit takes up again only on a start,
/// and that start follows it anew. It has no record to fix: librqbit cleared its record
/// before a full check, and the open drops the engine's record for one with none.
const STILL_LOOKS: u32 = 20;

impl Watch {
    /// Follows `h` when it is checking its files, after an ask that may have begun a new
    /// check: a task already following the hash follows it once more before it ends.
    fn watch(&self, h: &Arc<librqbit::ManagedTorrent>) {
        self.follow(h, true);
    }

    /// Follows `h` when it is checking and nothing follows it yet, as the tick does for
    /// every check it sees: seeing one is no sign a new check began.
    fn see(&self, h: &Arc<librqbit::ManagedTorrent>) {
        self.follow(h, false);
    }

    fn follow(&self, h: &Arc<librqbit::ManagedTorrent>, asked: bool) {
        if !checking(h) {
            return;
        }
        let key = h.info_hash().as_string();
        {
            let mut watching = recover(&self.watching);
            if let Some(again) = watching.get_mut(&key) {
                *again |= asked;
                return;
            }
            watching.insert(key.clone(), false);
        }
        let watch = self.clone();
        let h = h.clone();
        tokio::spawn(async move {
            use librqbit::TorrentStatsState as S;
            let id = librqbit::api::TorrentIdOrHash::Hash(h.info_hash());
            let (mut last, mut still) = (None, 0u32);
            loop {
                let now = watch.session.get(id);
                let ours = now.as_ref().is_some_and(|g| Arc::ptr_eq(g, &h));
                if watch.session.cancellation_token().is_cancelled() {
                    break;
                }
                if !ours {
                    // Removed, and maybe added again: a check of the new copy that began
                    // while this one was followed is followed in its own task.
                    let again = recover(&watch.watching).remove(&key) == Some(true);
                    if let Some(new) = now.filter(|_| again) {
                        watch.watch(&new);
                    }
                    return;
                }
                if !checking(&h) {
                    // Set right first, then the disk: a check that ended running against
                    // what was asked during it must not run on while the pieces are flushed.
                    watch.nudge.notify_one();
                    watch.keeper.keep_now(Some([key.clone()].into())).await;
                } else {
                    let stats = h.stats();
                    let paused = matches!(stats.state, S::Initializing { paused: true });
                    let got = stats.progress_bytes;
                    let before = last.replace(got);
                    still = if paused && before == Some(got) {
                        still + 1
                    } else {
                        0
                    };
                    if still < STILL_LOOKS {
                        let wait = check_pace(paused, got, before, watch.every);
                        tokio::time::sleep(wait).await;
                        continue;
                    }
                }
                // Ends here, unless a check began while it decided to.
                let began = {
                    let mut watching = recover(&watch.watching);
                    match watching.get_mut(&key) {
                        Some(again) if *again => {
                            *again = false;
                            true
                        }
                        _ => {
                            watching.remove(&key);
                            false
                        }
                    }
                };
                if !began {
                    return;
                }
                (last, still) = (None, 0);
            }
            recover(&watch.watching).remove(&key);
        });
    }
}

/// Keeps the pieces each torrent has in a record of the engine's own, `<state>/pieces/<info
/// hash>`, beside librqbit's. librqbit writes its record, `<session>/<info hash>.bitv`, only
/// every 16 MiB and when a torrent finishes, so a process that ends without a clean stop,
/// the way Android ends one, came back with what the last of those writes held and fetched
/// the rest again (#1181). The engine never writes librqbit's file while the session runs:
/// it adds its own record to it at open, before librqbit reads it (`fold_progress`).
#[derive(Clone)]
struct Keeper {
    session: Arc<Session>,
    shelf: Shelf,
    dir: PathBuf,
    /// Held for a whole pass, so two passes never write over each other.
    pass: Arc<Mutex<()>>,
    /// The pieces last written for each torrent, so one with nothing new is not written
    /// again. Held only to look up and to note, never across the disk.
    seen: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    /// When the last pass ran, so the tick writes at `every` however often it settles.
    last: Arc<Mutex<Option<std::time::Instant>>>,
    /// How often, at most, the tick writes the pieces down while a torrent runs; a hold
    /// and a pause write them at once.
    every: std::time::Duration,
    /// Set while a tick's pass is queued or running, so passes never pile up behind a slow
    /// flash.
    queued: Arc<std::sync::atomic::AtomicBool>,
    /// How many times each torrent's record was dropped, so a pass that read the pieces
    /// before a drop does not write them back after it.
    drops: Arc<Mutex<HashMap<String, u64>>>,
    /// The torrents whose record the open could not fold into librqbit's. Their record
    /// holds pieces librqbit does not know of, so for the rest of the run it is never
    /// dropped and only ever grows, and the next open folds it.
    pinned: Arc<HashSet<String>>,
}

/// Whether `haves` claims no piece `seen` did not.
fn within(haves: &[u8], seen: Option<&[u8]>) -> bool {
    seen.is_some_and(|s| s.len() == haves.len() && haves.iter().zip(s).all(|(h, s)| h & !s == 0))
}

impl Keeper {
    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(key)
    }

    /// Writes down the pieces of the torrents in `only`, or of every one. Blocks on the
    /// disk: run it off the runtime's workers.
    fn keep(&self, only: Option<&HashSet<String>>) {
        let _pass = recover(&self.pass);
        if only.is_none() {
            *recover(&self.last) = Some(std::time::Instant::now());
        }
        let api = librqbit::Api::new(self.session.clone(), None);
        let hashes: Vec<librqbit::dht::Id20> = self
            .session
            .with_torrents(|it| it.map(|(_, h)| h.info_hash()).collect());
        for hash in hashes {
            let key = hash.as_string();
            if only.is_some_and(|o| !o.contains(&key)) {
                continue;
            }
            // Read before the pieces, so a drop after this read is seen at the write.
            let drops = recover(&self.drops).get(&key).copied();
            // A torrent still checking its files has nothing to write yet, and a finished
            // one, like any other, has nothing new once its record holds what it has.
            let Ok((bits, _)) = api.api_dump_haves(librqbit::api::TorrentIdOrHash::Hash(hash))
            else {
                continue;
            };
            let haves = bits.as_raw_slice();
            let seen = recover(&self.seen).get(&key).cloned();
            if seen.as_deref() == Some(haves) {
                continue;
            }
            // The bytes of every piece read above reach the disk before the record that
            // claims them, so a power cut never leaves a record ahead of its files. A
            // record that claims no piece the last one did not is safe to write as it is,
            // and must not wait for a flush: it may be taking out pieces a check found bad.
            if !within(haves, seen.as_deref()) {
                // One a remove took off the shelf is on its way out: nothing to write.
                if self.shelf.is_retired(&key) {
                    continue;
                }
                let flushed = match self.shelf.get(&key) {
                    Some(files) => files.sync(),
                    None => Err(std::io::Error::other("its files are not on the shelf")),
                };
                if let Err(e) = flushed {
                    tracing::warn!("the files of {key} were not flushed: {e}");
                    continue;
                }
            }
            // `seen` is held across this one small write, so a drop meanwhile either comes
            // first and is seen here, or comes after and removes what is written.
            let mut seen = recover(&self.seen);
            if recover(&self.drops).get(&key).copied() != drops {
                continue;
            }
            // A pinned record keeps the pieces only it knows of.
            let bits = if self.pinned.contains(&key) {
                std::fs::read(self.path(&key))
                    .ok()
                    .filter(|r| r.len() == haves.len())
                    .map(|r| merge_pieces(&r, haves))
                    .unwrap_or_else(|| haves.to_vec())
            } else {
                haves.to_vec()
            };
            match anibeam_foundation::atomic_file::write(&self.path(&key), &bits) {
                Ok(()) => {
                    seen.insert(key, haves.to_vec());
                }
                Err(e) => tracing::warn!("the pieces of {key} were not written down: {e}"),
            }
        }
    }

    /// A pass now over `only`, or every torrent, waited for, off the runtime's workers.
    async fn keep_now(&self, only: Option<HashSet<String>>) {
        let keeper = self.clone();
        if let Err(e) = tokio::task::spawn_blocking(move || keeper.keep(only.as_ref())).await {
            tracing::warn!("the pieces were not written down: {e}");
        }
    }

    /// A pass in a task of its own when the last one is `every` old and none is running,
    /// so the tick never waits on the disk.
    fn keep_when_due(&self) {
        let due = recover(&self.last).is_none_or(|t| t.elapsed() >= self.every);
        if due && !self.queued.swap(true, std::sync::atomic::Ordering::SeqCst) {
            let keeper = self.clone();
            tokio::spawn(async move {
                keeper.keep_now(None).await;
                keeper
                    .queued
                    .store(false, std::sync::atomic::Ordering::SeqCst);
            });
        }
    }

    /// Drops a torrent's record at once, before librqbit is asked for a check that may take
    /// pieces out of its own record: no check can then write a smaller record while the
    /// engine's older one is still on disk. It waits for no pass; one that read the pieces
    /// before the drop does not write them.
    fn drop_record(&self, key: &str) {
        if self.pinned.contains(key) {
            return;
        }
        let mut seen = recover(&self.seen);
        *recover(&self.drops).entry(key.to_string()).or_default() += 1;
        seen.remove(key);
        match std::fs::remove_file(self.path(key)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                tracing::warn!("the pieces of {key} were not removed: {e}");
            }
            _ => {}
        }
    }

    /// A torrent removed: its record goes, and a torrent added again under its hash starts
    /// from nothing. It waits for a pass to end, off the runtime's workers.
    async fn forget(&self, key: String) {
        let keeper = self.clone();
        let gone = tokio::task::spawn_blocking(move || {
            let _pass = recover(&keeper.pass);
            recover(&keeper.seen).remove(&key);
            match std::fs::remove_file(keeper.path(&key)) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                    tracing::warn!("the pieces of {key} were not removed: {e}");
                }
                _ => {}
            }
        });
        if let Err(e) = gone.await {
            tracing::warn!("the pieces were not removed: {e}");
        }
    }
}

/// Adds each record the engine kept to librqbit's own, before the session opens and so
/// before librqbit holds or reads its files. librqbit then checks a sample of the pieces as
/// it opens, and checks every piece when one is not on disk. A record is dropped only when
/// librqbit has no record for its torrent or one of another length: the torrent is gone,
/// or was never checked. Any other error leaves it for the next open, and the torrents whose
/// record it leaves are the answer (`Keeper::pinned`).
fn fold_progress(pieces: &Path, session_dir: &Path) -> HashSet<String> {
    let mut left = HashSet::new();
    let Ok(records) = std::fs::read_dir(pieces) else {
        return left;
    };
    for record in records.flatten() {
        let path = record.path();
        let Some(key) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if key.ends_with(".tmp") {
            continue;
        }
        let theirs = session_dir.join(format!("{key}.bitv"));
        let on_disk = match std::fs::read(&theirs) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let _ = std::fs::remove_file(&path);
                continue;
            }
            Err(e) => {
                tracing::warn!("librqbit's record of {key} was not read: {e}");
                left.insert(key.to_string());
                continue;
            }
        };
        let ours = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::warn!("the pieces of {key} were not read: {e}");
                left.insert(key.to_string());
                continue;
            }
        };
        if ours.len() != on_disk.len() {
            let _ = std::fs::remove_file(&path);
            continue;
        }
        let merged = merge_pieces(&on_disk, &ours);
        // Folded, or with nothing to add, the record has done its job and goes; one that
        // could not be folded stays for the next open.
        let folded = merged == on_disk
            || match anibeam_foundation::atomic_file::write(&theirs, &merged) {
                Ok(()) => true,
                Err(e) => {
                    tracing::warn!("the pieces of {key} were not added to librqbit's record: {e}");
                    false
                }
            };
        if folded {
            let _ = std::fs::remove_file(&path);
        } else {
            left.insert(key.to_string());
        }
    }
    left
}

/// Both records' pieces: a piece in either stays.
fn merge_pieces(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b).map(|(x, y)| x | y).collect()
}

fn fail(e: impl std::fmt::Display) -> CoreError {
    CoreError::service("torrents", "engine", e.to_string())
}

fn now() -> i64 {
    anibeam_foundation::time::now_secs()
}

/// An add that gives up after a minute: a magnet nobody answers for would wait forever.
async fn within_a_minute<T>(
    add: impl std::future::Future<Output = anyhow::Result<T>>,
) -> Result<T, CoreError> {
    match tokio::time::timeout(std::time::Duration::from_secs(60), add).await {
        Ok(Ok(added)) => Ok(added),
        Ok(Err(e)) => Err(fail(e)),
        Err(_) => Err(fail(
            "no peer sent the torrent's details within a minute; try a release with more seeders",
        )),
    }
}

/// Each torrent librqbit's session will bring back, from its own record: its info-hash,
/// its folder and the files it fetches (None for all).
fn restored(path: &Path) -> Vec<(String, PathBuf, Option<HashSet<usize>>)> {
    let Some(db) = std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
    else {
        return Vec::new();
    };
    let Some(torrents) = db.get("torrents").and_then(|t| t.as_object()) else {
        return Vec::new();
    };
    torrents
        .values()
        .filter_map(|t| {
            let hash = t.get("info_hash")?.as_str()?.to_lowercase();
            let folder = PathBuf::from(t.get("output_folder")?.as_str()?);
            let wanted = t.get("only_files").and_then(|o| o.as_array()).map(|a| {
                a.iter()
                    .filter_map(|v| v.as_u64().map(|n| n as usize))
                    .collect()
            });
            Some((hash, folder, wanted))
        })
        .collect()
}

/// A torrent's files with how far each is and its priority.
fn files_of(
    h: &librqbit::ManagedTorrent,
    stats: &librqbit::TorrentStats,
    entry: &Entry,
) -> Vec<TorrentFile> {
    h.with_metadata(|m| {
        m.file_infos
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let done = stats.file_progress.get(i).copied().unwrap_or(0);
                let name = f.relative_filename.to_string_lossy().into_owned();
                TorrentFile {
                    episode: anibeam_releases::facts::read_name(&name).episode,
                    name,
                    size: f.len,
                    progress: if f.len == 0 {
                        1.0
                    } else {
                        done as f64 / f.len as f64
                    },
                    priority: entry.priority(i),
                }
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default()
}

impl Engine {
    /// Opens the session: torrents write under `root` (the library folder), and the engine's
    /// own state lives in `state_dir`, so downloads pick up where they were after a restart.
    pub async fn open(root: &Path, state_dir: &Path) -> Result<Engine, CoreError> {
        Engine::open_ticking(root, state_dir, TICK).await
    }

    /// `open`, with the tick every `every`; tests tick faster.
    async fn open_ticking(
        root: &Path,
        state_dir: &Path,
        every: std::time::Duration,
    ) -> Result<Engine, CoreError> {
        std::fs::create_dir_all(root).map_err(fail)?;
        std::fs::create_dir_all(state_dir).map_err(fail)?;
        let entries_path = state_dir.join("torrents.json");
        let entries: HashMap<String, Entry> = std::fs::read(&entries_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        // The session brings its torrents back as it opens, so each one's folder and the
        // files it fetches are on the shelf first: librqbit's own record of them, else ours.
        let shelf = Shelf::new(state_dir.join("parts"));
        let session_dir = state_dir.join("session");
        for (hash, folder, wanted) in restored(&session_dir.join("session.json")) {
            shelf.put(&hash, &folder, wanted);
        }
        for (hash, e) in &entries {
            if shelf.get(hash).is_none() {
                shelf.put(hash, &root.join(&e.folder), e.wanted());
            }
        }
        // The pieces the engine wrote down since librqbit last did go into librqbit's record
        // now, while nothing holds it.
        let pieces = state_dir.join("pieces");
        let pinned = Arc::new(fold_progress(&pieces, &session_dir));
        let session = Session::new_with_opts(
            root.to_path_buf(),
            SessionOptions {
                fastresume: true,
                persistence: Some(SessionPersistenceConfig::Json {
                    folder: Some(session_dir.clone()),
                }),
                default_storage_factory: Some(shelf.factory(root.to_path_buf())),
                dht: Some(librqbit::DhtSessionConfig {
                    persistence: Some(librqbit::dht::DhtPersistenceConfig {
                        config_filename: Some(state_dir.join("dht.json")),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await
        .map_err(fail)?;
        let nudge: Arc<tokio::sync::Notify> = Arc::default();
        let keeper = Keeper {
            session: session.clone(),
            shelf: shelf.clone(),
            dir: pieces,
            pass: Arc::default(),
            seen: Arc::default(),
            last: Arc::default(),
            every: every * 3,
            queued: Arc::default(),
            drops: Arc::default(),
            pinned,
        };
        let engine = Engine {
            session: session.clone(),
            shelf,
            root: root.to_path_buf(),
            entries_path,
            saving: Mutex::new(()),
            wants: Wants {
                session: session.clone(),
                held: Arc::new(Mutex::new(true)),
                entries: Arc::new(Mutex::new(entries)),
                steering: Arc::default(),
                keeper: keeper.clone(),
            },
            nudge: nudge.clone(),
            watch: Watch {
                session: session.clone(),
                keeper: keeper.clone(),
                nudge,
                watching: Arc::default(),
                every,
            },
            keeper,
        };
        // The torrents the session brought back are checking their files now.
        session
            .with_torrents(|it| it.map(|(_, h)| h.clone()).collect::<Vec<_>>())
            .iter()
            .for_each(|h| engine.watch.watch(h));
        // From here until the session stops, each tick sets every torrent to what is wanted
        // and writes the pieces down. It comes soon while a torrent checks its files, every
        // `every` while one runs, and waits for a nudge when none does.
        let (wants, keeper, watch) = (
            engine.wants.clone(),
            engine.keeper.clone(),
            engine.watch.clone(),
        );
        let nudge = engine.nudge.clone();
        let stopped = session.cancellation_token().clone();
        tokio::spawn(async move {
            let mut next = Some(CHECKING_TICK.min(every));
            let mut checks = HashMap::new();
            loop {
                let wake = async {
                    match next {
                        Some(wait) => {
                            let _ = tokio::time::timeout(wait, nudge.notified()).await;
                        }
                        None => nudge.notified().await,
                    }
                };
                tokio::select! {
                    _ = stopped.cancelled() => return,
                    _ = wake => {}
                }
                let ticked = wants.tick(every, &mut checks).await;
                next = ticked.next;
                ticked.checking.iter().for_each(|h| watch.see(h));
                // The disk is written off the tick, so settling never waits on a flush.
                if !ticked.ended.is_empty() {
                    let (keeper, ended) = (keeper.clone(), ticked.ended);
                    tokio::spawn(async move { keeper.keep_now(Some(ended)).await });
                }
                keeper.keep_when_due();
            }
        });
        Ok(engine)
    }

    /// Wakes the tick, after a change that may start a torrent or change what is wanted.
    fn nudge(&self) {
        self.nudge.notify_one();
    }

    /// Writes down the pieces every torrent has now, so they are there after the process
    /// ends without a clean stop. The engine also does it every 15 s while a torrent runs,
    /// and when a check ends.
    pub async fn keep_progress(&self) {
        self.keeper.keep_now(None).await;
    }

    /// Saves the list on its own turn, so two saves from separate paths land in the
    /// order their lists were taken and the newest is the one left on disk (#582). The
    /// list is held only while it is copied, never across the write.
    fn save_entries(&self) {
        let _turn = recover(&self.saving);
        let entries = recover(&self.wants.entries).clone();
        match serde_json::to_vec_pretty(&entries) {
            Ok(bytes) => {
                if let Err(e) = anibeam_foundation::atomic_file::write(&self.entries_path, bytes) {
                    tracing::warn!("the download list was not saved: {e}");
                }
            }
            Err(e) => tracing::warn!("the download list was not saved: {e}"),
        }
    }

    /// Held from the start: nothing downloads until the app has looked at the network and
    /// said Wi-Fi or allowed mobile data.
    pub fn held(&self) -> bool {
        *recover(&self.wants.held)
    }

    /// Adds `release` to download into `<root>/<folder>`, held at once when downloads are.
    /// `torrent` is the `.torrent` file when the core fetched it itself (an http link), so
    /// the engine never looks the host up on its own.
    pub async fn add(
        &self,
        release: &Release,
        folder: &str,
        torrent: Option<Vec<u8>>,
    ) -> Result<Added, CoreError> {
        let source = match torrent {
            Some(bytes) => AddTorrent::from_bytes(bytes),
            None => AddTorrent::from_url(&release.link),
        };
        // A magnet has to fetch its few kilobytes of metadata before it can pause, so it is
        // added running and held right after: on mobile data only that little goes out.
        let (hash, new) = self
            .start(
                release,
                folder,
                source,
                AddTorrentOptions::default(),
                Vec::new(),
                None,
            )
            .await?;
        Ok(Added { hash, new })
    }

    /// Reads a torrent's file list without adding it, so a rolling download can pick the
    /// files it fetches before a byte of the others is written. A magnet asks the peers for
    /// its details here, once: the add that follows starts from what came back.
    pub async fn peek(
        &self,
        release: &Release,
        torrent: Option<Vec<u8>>,
    ) -> Result<Peeked, CoreError> {
        let trackers = if torrent.is_none() {
            librqbit::Magnet::parse(&release.link)
                .map(|m| m.trackers)
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let source = match torrent {
            Some(bytes) => AddTorrent::from_bytes(bytes),
            None => AddTorrent::from_url(&release.link),
        };
        let listed = within_a_minute(self.session.add_torrent(
            source,
            Some(AddTorrentOptions {
                list_only: true,
                ..Default::default()
            }),
        ))
        .await?;
        let AddTorrentResponse::ListOnly(l) = listed else {
            return Err(fail(
                "the engine added the torrent it was only asked to read",
            ));
        };
        let files = l
            .info
            .iter_file_details()
            .map(|d| {
                let name = d.filename.to_pathbuf().to_string_lossy().into_owned();
                TorrentFile {
                    episode: anibeam_releases::facts::read_name(&name).episode,
                    name,
                    size: d.len,
                    progress: 0.0,
                    priority: NORMAL,
                }
            })
            .collect();
        Ok(Peeked {
            files,
            bytes: l.torrent_bytes.to_vec(),
            peers: l.seen_peers,
            trackers,
            hash: l.info_hash.as_string(),
        })
    }

    /// Adds a torrent `peek` read, each file at the priority given (all normal when the list
    /// is empty): the skipped ones are never fetched and never written into the library.
    pub async fn add_peeked(
        &self,
        release: &Release,
        folder: &str,
        peeked: Peeked,
        priorities: Vec<i64>,
    ) -> Result<String, CoreError> {
        // One the engine has already keeps its record and its selection: nothing about it
        // changes, the shelf included, or the storage would write files the session skips.
        if let Some(h) = self.handle(&peeked.hash) {
            return Ok(h.info_hash().as_string());
        }
        let entry = Entry {
            priorities,
            ..Default::default()
        };
        let wanted = entry.wanted();
        let options = AddTorrentOptions {
            only_files: wanted.map(|w| {
                let mut v: Vec<usize> = w.into_iter().collect();
                v.sort_unstable();
                v
            }),
            initial_peers: (!peeked.peers.is_empty()).then_some(peeked.peers),
            trackers: (!peeked.trackers.is_empty()).then_some(peeked.trackers),
            // From the file, nothing has to come in before it can be held.
            paused: self.held(),
            ..Default::default()
        };
        let (hash, _) = self
            .start(
                release,
                folder,
                AddTorrent::from_bytes(peeked.bytes),
                options,
                entry.priorities,
                Some(&peeked.hash),
            )
            .await?;
        Ok(hash)
    }

    async fn start(
        &self,
        release: &Release,
        folder: &str,
        source: AddTorrent<'_>,
        options: AddTorrentOptions,
        priorities: Vec<i64>,
        known: Option<&str>,
    ) -> Result<(String, bool), CoreError> {
        let output = self.root.join(folder);
        // When the hash is known before the add and librqbit does not have the torrent,
        // what an earlier copy left goes before librqbit can check or settle the new one:
        // its pause, its finish time and its piece record.
        if let Some(hash) = known
            && self.handle(hash).is_none()
        {
            if let Some(entry) = recover(&self.wants.entries).get_mut(hash) {
                entry.paused_by_user = false;
                entry.completion_on = 0;
            }
            self.keeper.drop_record(hash);
        }
        // The add starts the torrent's files anew, whatever an earlier copy of it left on
        // the shelf: its folder and its selection are this add's.
        let wanted: Option<HashSet<usize>> = options
            .only_files
            .as_ref()
            .map(|o| o.iter().copied().collect());
        // The person's asks on each entry before the add, so an entry left from an earlier
        // copy of the torrent can be told from a pause given while this add ran.
        let asks_before: HashMap<String, u64> = recover(&self.wants.entries)
            .iter()
            .map(|(k, e)| (k.clone(), e.asks))
            .collect();
        let added = within_a_minute(self.session.add_torrent(
            source,
            Some(AddTorrentOptions {
                output_folder: Some(output.to_string_lossy().into_owned()),
                overwrite: true,
                storage_factory: Some(self.shelf.adding(output.clone(), wanted, self.root.clone())),
                ..options
            }),
        ))
        .await?;
        let handle = match added {
            AddTorrentResponse::Added(_, h) => h,
            // One the engine has already keeps its record: its files stay where they are.
            AddTorrentResponse::AlreadyManaged(_, h) => {
                return Ok((h.info_hash().as_string(), false));
            }
            AddTorrentResponse::ListOnly(_) => {
                return Err(fail("the engine only listed the torrent"));
            }
        };
        let hash = handle.info_hash().as_string();
        {
            // The entry goes in on the same turn as the hold's say over the torrent, and
            // keeps a pause the person gave it while this add ran. One left from an earlier
            // copy of the torrent, which librqbit no longer had, starts over: its pause and
            // its finish time are not this download's, and nor are the pieces the engine
            // wrote down for it.
            let _turn = self.wants.steering.lock().await;
            {
                let mut entries = recover(&self.wants.entries);
                let entry = entries.entry(hash.clone()).or_default();
                let left_over = asks_before.get(&hash) == Some(&entry.asks);
                if left_over {
                    entry.paused_by_user = false;
                    entry.completion_on = 0;
                    // Known before the add, the record already went; otherwise it goes
                    // now, before the new copy's check can have written its own.
                    if known.is_none() {
                        self.keeper.drop_record(&hash);
                    }
                }
                entry.title = release.title.clone();
                entry.folder = folder.to_string();
                entry.added_on = now();
                entry.priorities = priorities;
            }
            // Refused, the torrent is still added: it is said in the log, as for a hold.
            if let Err(e) = self.wants.settle(&handle, Retry::Broken).await {
                tracing::warn!("hold for {hash} as it was added: {e}");
            }
        }
        self.watch.watch(&handle);
        self.nudge();
        self.save_entries();
        Ok((hash, true))
    }

    /// Every torrent as qBittorrent would list it. A finished one is paused here: a phone
    /// does not seed on battery and mobile data.
    pub async fn list(&self) -> Vec<Torrent> {
        let handles: Vec<_> = self
            .session
            .with_torrents(|it| it.map(|(_, h)| h.clone()).collect());
        let mut out = Vec::new();
        let mut changed = false;
        for h in handles {
            let hash = h.info_hash().as_string();
            let stats = h.stats();
            // The lock is held only in this block: the engine is awaited below.
            let entry = {
                let mut entries = recover(&self.wants.entries);
                let entry = entries.entry(hash.clone()).or_default();
                if stats.finished && entry.completion_on == 0 {
                    entry.completion_on = now();
                    changed = true;
                }
                // A rolling download's window moved on: it is downloading again. A torrent
                // still checking its files, or stopped by an error, reads as unfinished too,
                // and keeps its time.
                if !stats.finished
                    && stats.error.is_none()
                    && matches!(
                        stats.state,
                        librqbit::TorrentStatsState::Live | librqbit::TorrentStatsState::Paused
                    )
                    && entry.completion_on != 0
                {
                    entry.completion_on = 0;
                    changed = true;
                }
                entry.clone()
            };
            if stats.finished && !h.is_paused() {
                let _turn = self.wants.steering.lock().await;
                if !h.is_paused() {
                    let _ = self.session.pause(&h).await;
                }
            }
            let paused = matches!(
                stats.state,
                librqbit::TorrentStatsState::Paused
                    | librqbit::TorrentStatsState::Initializing { paused: true }
            );
            let live = stats.live.as_ref();
            let speed = live
                .map(|l| (l.download_speed.mbps * 1_048_576.0) as u64)
                .unwrap_or(0);
            let peers = live.map(|l| l.snapshot.peer_stats.live as u64).unwrap_or(0);
            let progress = if stats.total_bytes == 0 {
                0.0
            } else {
                // librqbit counts a check on from where a paused one stopped, so a check
                // started again can count past the whole torrent.
                (stats.progress_bytes as f64 / stats.total_bytes as f64).min(1.0)
            };
            // A first check a quick hold and lift broke is no failure: it starts again once
            // the hold lifts, and until then it reads as paused.
            let error = stats
                .error
                .as_ref()
                .filter(|e| !broken_check(Some(e.as_str())));
            let paused = paused || error.is_none() && stats.error.is_some();
            let state = match (&error, stats.finished, paused, &stats.state) {
                (Some(_), _, _, _) => "error",
                (None, true, _, _) => "pausedUP",
                (None, false, true, _) => "pausedDL",
                (None, false, false, librqbit::TorrentStatsState::Initializing { .. }) => {
                    "checkingDL"
                }
                (None, false, false, _) if peers == 0 => "stalledDL",
                _ => "downloading",
            };
            let eta = match (speed, stats.finished) {
                (_, true) => 0,
                (0, _) => 8_640_000,
                (s, _) => (stats.total_bytes.saturating_sub(stats.progress_bytes) / s) as i64,
            };
            let name = h.name().unwrap_or_else(|| entry.title.clone());
            let save_path = self.root.join(&entry.folder).to_string_lossy().into_owned();
            let files = files_of(&h, &stats, &entry);
            let content_path = match files.as_slice() {
                [one] => Path::new(&save_path)
                    .join(&one.name)
                    .to_string_lossy()
                    .into_owned(),
                _ => Path::new(&save_path)
                    .join(&name)
                    .to_string_lossy()
                    .into_owned(),
            };
            let facts = anibeam_releases::facts::read_name(&name);
            out.push(Torrent {
                hash,
                name,
                state: state.to_string(),
                progress,
                size: stats.total_bytes,
                dlspeed: speed,
                upspeed: 0,
                eta,
                num_seeds: peers,
                num_leechs: 0,
                ratio: 0.0,
                save_path,
                content_path,
                added_on: entry.added_on,
                completion_on: entry.completion_on,
                tags: String::new(),
                category: String::new(),
                series_id: None,
                episode: facts.episode,
                batch: facts.batch,
                files,
                seq_dl: false,
                f_l_piece_prio: false,
                tracker: error.cloned().unwrap_or_default(),
                group: facts.group,
                resolution: facts.resolution,
                provider: String::new(),
                roll: None,
            });
        }
        if changed {
            self.save_entries();
        }
        out.sort_by_key(|t| std::cmp::Reverse(t.added_on));
        out
    }

    fn handle(&self, hash: &str) -> Option<Arc<librqbit::ManagedTorrent>> {
        let id = librqbit::dht::Id20::from_str(hash).ok()?;
        self.session.get(librqbit::api::TorrentIdOrHash::Hash(id))
    }

    /// One torrent's files, as `list` gives them.
    pub fn files(&self, hash: &str) -> Result<Vec<TorrentFile>, CoreError> {
        let h = self
            .handle(hash)
            .ok_or_else(|| CoreError::invalid("hash", format!("no torrent {hash}")))?;
        let entry = recover(&self.wants.entries)
            .get(&h.info_hash().as_string())
            .cloned()
            .unwrap_or_default();
        Ok(files_of(&h, &h.stats(), &entry))
    }

    /// Sets files of a torrent to a priority, as qBittorrent's `filePrio`: 0 stops fetching
    /// them, anything else fetches them. A torrent paused because it had everything it was
    /// asked for starts again when it is asked for more, unless the person paused it or
    /// downloads are held.
    pub async fn set_priority(
        &self,
        hash: &str,
        ids: &[usize],
        priority: i64,
    ) -> Result<(), CoreError> {
        let h = self
            .handle(hash)
            .ok_or_else(|| CoreError::invalid("hash", format!("no torrent {hash}")))?;
        let key = h.info_hash().as_string();
        let count = h.with_metadata(|m| m.file_infos.len()).map_err(fail)?;
        // The new priorities are worked out on a copy and kept only once the engine took
        // them: librqbit refuses a change while a torrent still checks its files, and a
        // priority recorded but never applied would make the next pass see nothing to do.
        let (old, new, folder) = {
            let entries = recover(&self.wants.entries);
            let e = entries.get(&key).cloned().unwrap_or_default();
            let mut n = e.clone();
            n.priorities.resize(count, NORMAL);
            for &i in ids.iter().filter(|&&i| i < count) {
                n.priorities[i] = priority;
            }
            (e.wanted(), n, self.root.join(&e.folder))
        };
        let wanted = new.wanted();
        // The storage learns first, so a file wanted again takes its part file over before
        // the engine writes to it.
        let files = self.shelf.files(&key, &folder);
        files.set_wanted(wanted.clone());
        let all: HashSet<usize> = (0..count).collect();
        if let Err(e) = self
            .session
            .update_only_files(&h, wanted.as_ref().unwrap_or(&all))
            .await
        {
            files.set_wanted(old);
            return Err(fail(e));
        }
        recover(&self.wants.entries)
            .entry(key.clone())
            .or_default()
            .priorities = new.priorities;
        self.save_entries();
        // Asked for more, it may want to run again: settled on the turn, like any start.
        let _turn = self.wants.steering.lock().await;
        // The new priorities are in either way: a refused start or stop is only logged, as
        // for a hold, so a rolling download does not take the change for a failure.
        if let Err(e) = self.wants.settle(&h, Retry::Broken).await {
            tracing::warn!("settling {key} after its priorities changed: {e}");
        }
        self.watch.watch(&h);
        self.nudge();
        Ok(())
    }

    /// Files of the torrent were deleted underneath it: it lets go of the ones it held
    /// open, so their space comes back. librqbit has no recheck; a deleted file is one the
    /// torrent no longer fetches, so it is never asked for again.
    pub fn close_files(&self, hash: &str) {
        if let Some(files) = self.shelf.get(&hash.to_lowercase()) {
            files.close();
        }
    }

    /// Pauses, resumes or removes torrents. Each is tried, so one librqbit refuses never
    /// leaves the rest undone, and the list is saved whatever happened; the first refusal
    /// is the answer. Only a pause or a resume takes the turn: a remove can take a while to
    /// delete its files, and a hold must not wait for it.
    pub async fn control(&self, hashes: &[String], action: PhoneAction) -> Result<(), CoreError> {
        let mut refused = None;
        for hash in hashes {
            let Some(h) = self.handle(hash) else { continue };
            if let Err(e) = self.control_one(&h, action).await {
                tracing::warn!("{action:?} for {hash}: {e}");
                refused.get_or_insert(fail(e));
            }
        }
        self.save_entries();
        self.nudge();
        // A pause often comes before the app goes away: the pieces of what was paused are
        // written down, and the tap waits for that a moment at most. A slow flash finishes
        // the pass after the answer, off the runtime's workers.
        if action == PhoneAction::Pause {
            let paused = hashes
                .iter()
                .filter_map(|hash| self.handle(hash))
                .map(|h| h.info_hash().as_string())
                .collect();
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(2),
                self.keeper.keep_now(Some(paused)),
            )
            .await;
        }
        refused.map_or(Ok(()), Err)
    }

    /// One torrent of `control`. The entry is keyed by the torrent's own info-hash, the way
    /// every other read of it looks it up, whatever case came in.
    async fn control_one(
        &self,
        h: &Arc<librqbit::ManagedTorrent>,
        action: PhoneAction,
    ) -> anyhow::Result<()> {
        let key = h.info_hash().as_string();
        match action {
            // The person's pause is kept before librqbit is asked: a torrent the hold
            // already stopped refuses a second pause, and the hold lifting must not start
            // it again (#1198). A resume starts one in error too, which librqbit then checks
            // afresh. `settle` decides either way, as it does for the hold and the tick.
            PhoneAction::Pause | PhoneAction::Resume => {
                let _turn = self.wants.steering.lock().await;
                let resume = action == PhoneAction::Resume;
                {
                    let mut entries = recover(&self.wants.entries);
                    let entry = entries.entry(key).or_default();
                    entry.paused_by_user = !resume;
                    entry.asks += 1;
                }
                let retry = if resume { Retry::Any } else { Retry::Broken };
                let settled = self.wants.settle(h, retry).await;
                self.watch.watch(h);
                settled
            }
            PhoneAction::Remove | PhoneAction::Forget => {
                let files = action == PhoneAction::Remove;
                // Off the shelf first, so an add of the same torrent while librqbit deletes
                // this one gets files of its own, and the delete never reaches them. Not for
                // one in librqbit's error state: it has no storage of its own then, and its
                // delete makes one from the shelf, which must still hold its files.
                let errored = matches!(h.stats().state, librqbit::TorrentStatsState::Error);
                let retired = if errored {
                    None
                } else {
                    self.shelf.retire(&key)
                };
                let id = librqbit::api::TorrentIdOrHash::Hash(h.info_hash());
                if let Err(e) = self.session.delete(id, files).await {
                    if let Some(retired) = retired {
                        self.shelf.put_back(&key, retired);
                    }
                    return Err(e);
                }
                // What a copy added meanwhile has is left alone, so each step looks again.
                let gone = || self.handle(&key).is_none();
                if gone() {
                    recover(&self.wants.entries).remove(&key);
                }
                let retired = retired.or_else(|| gone().then(|| self.shelf.retire(&key)).flatten());
                if let Some(retired) = retired.filter(|_| gone()) {
                    retired.remove_parts();
                }
                if gone() {
                    self.keeper.forget(key).await;
                }
                Ok(())
            }
        }
    }

    /// Holds every unfinished download, or lets the ones the person did not pause run again,
    /// starting again one in error the hold had paused. One still checking its files is set
    /// right by the tick once the check is done.
    pub async fn set_held(&self, held: bool) -> Result<(), CoreError> {
        {
            let _turn = self.wants.steering.lock().await;
            *recover(&self.wants.held) = held;
            let handles: Vec<_> = self
                .session
                .with_torrents(|it| it.map(|(_, h)| h.clone()).collect());
            let retry = if held { Retry::Broken } else { Retry::Held };
            for h in handles {
                // One torrent the engine will not pause or start must not leave the rest
                // running on mobile data: each is tried, failures only logged.
                if let Err(e) = self.wants.settle(&h, retry).await {
                    tracing::warn!("hold {held} for {}: {e}", h.info_hash().as_string());
                }
                self.watch.watch(&h);
            }
        }
        self.nudge();
        // A hold comes when Android stops the download work, often just before it ends the
        // process: what came in so far is written down before the hold is answered. The
        // pass runs off the runtime's workers, so only this call waits for it.
        if held {
            self.keep_progress().await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(title: &str) -> Release {
        Release {
            title: title.to_string(),
            link: "https://example.org/show.torrent".to_string(),
            seeders: None,
            leechers: None,
            downloads: None,
            size: String::new(),
            category: String::new(),
            page: String::new(),
            source: String::new(),
            resolution: String::new(),
            infomkv: String::new(),
            linktype: String::new(),
        }
    }

    /// A three-episode batch made here, as a `.torrent` file's bytes.
    async fn batch(dir: &Path) -> Vec<u8> {
        batch_of(dir, 50_000).await
    }

    /// The same batch with `len` bytes an episode, its files under `<dir>/made/Show`.
    async fn batch_of(dir: &Path, len: usize) -> Vec<u8> {
        let src = dir.join("made").join("Show");
        std::fs::create_dir_all(&src).unwrap();
        for i in 1..=3u8 {
            std::fs::write(src.join(format!("Show - 0{i}.mkv")), vec![i; len]).unwrap();
        }
        let made = librqbit::create_torrent(
            &src,
            librqbit::CreateTorrentOptions {
                name: Some("Show"),
                trackers: Vec::new(),
                piece_length: Some(16_384),
            },
            &librqbit::spawn_utils::BlockingSpawner::new(1),
        )
        .await
        .unwrap();
        made.as_bytes().unwrap().to_vec()
    }

    fn on_disk(root: &Path) -> Vec<String> {
        let mut names: Vec<String> = walkdir::WalkDir::new(root)
            .into_iter()
            .flatten()
            .filter(|e| e.file_type().is_file())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    async fn settled(engine: &Engine, hash: &str) {
        for _ in 0..100 {
            let h = engine.handle(hash).unwrap();
            if !matches!(
                h.stats().state,
                librqbit::TorrentStatsState::Initializing { .. }
            ) {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        panic!("the torrent never left its first check");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_panic_with_the_list_held_still_lets_it_save() {
        // One panic with `entries` or `held` held used to break every later save or
        // hold check for the rest of the run, since each unwrapped the poison (#923).
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(&dir.path().join("Anime"), &dir.path().join("state"))
            .await
            .unwrap();
        std::thread::scope(|s| {
            let held = s.spawn(|| {
                let _entries = engine.wants.entries.lock().unwrap();
                let _held = engine.wants.held.lock().unwrap();
                panic!("boom, with the list and the hold held");
            });
            assert!(held.join().is_err());
        });
        assert!(engine.wants.entries.lock().is_err(), "the list is poisoned");
        assert!(engine.wants.held.lock().is_err(), "the hold is poisoned");
        recover(&engine.wants.entries).insert("abc".to_string(), Entry::default());
        engine.save_entries();
        let saved = std::fs::read_to_string(&engine.entries_path).unwrap();
        assert!(saved.contains("abc"), "the list was saved: {saved}");
        assert!(engine.held());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_skipped_episode_never_lands_in_the_library_even_after_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = batch(dir.path()).await;
        let root = dir.path().join("Anime");
        let state = dir.path().join("state");
        let engine = Engine::open(&root, &state).await.unwrap();
        let r = release("[G] Show (01-03) [Batch]");
        let peeked = engine.peek(&r, Some(bytes)).await.unwrap();
        assert_eq!(peeked.files.len(), 3);
        let two = peeked
            .files
            .iter()
            .position(|f| f.name.ends_with("02.mkv"))
            .unwrap();
        let mut wish = vec![NORMAL; 3];
        wish[two] = SKIP;
        let hash = engine.add_peeked(&r, "Show", peeked, wish).await.unwrap();
        settled(&engine, &hash).await;
        assert_eq!(on_disk(&root), ["Show - 01.mkv", "Show - 03.mkv"]);
        assert_eq!(engine.files(&hash).unwrap()[two].priority, SKIP);

        // Opened again, the session brings the torrent back with the same files skipped.
        engine.session.stop().await;
        drop(engine);
        let engine = Engine::open(&root, &state).await.unwrap();
        settled(&engine, &hash).await;
        assert_eq!(on_disk(&root), ["Show - 01.mkv", "Show - 03.mkv"]);

        // Wanted again, it is fetched: the file shows up and reads as normal.
        engine.set_priority(&hash, &[two], NORMAL).await.unwrap();
        assert_eq!(engine.files(&hash).unwrap()[two].priority, NORMAL);
        engine.session.stop().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_torrent_is_new_only_to_the_add_that_brought_it_in() {
        // A subscription owns what its add brought in, never one already here (#654).
        let dir = tempfile::tempdir().unwrap();
        let bytes = batch(dir.path()).await;
        let engine = Engine::open(&dir.path().join("Anime"), &dir.path().join("state"))
            .await
            .unwrap();
        let r = release("[G] Show (01-03) [Batch]");
        let first = engine.add(&r, "Show", Some(bytes.clone())).await.unwrap();
        let again = engine.add(&r, "Show", Some(bytes)).await.unwrap();
        assert!(first.new && !again.new);
        assert_eq!(first.hash, again.hash);
        engine.session.stop().await;
    }

    /// Waits up to ten seconds for `hash` to run, and says whether it did.
    async fn runs(engine: &Engine, hash: &str) -> bool {
        for _ in 0..200 {
            let state = engine.handle(hash).unwrap().stats().state;
            if matches!(state, librqbit::TorrentStatsState::Live) {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        false
    }

    /// A session that seeds the batch under `<dir>/made/Show` to whoever connects, sending
    /// at most `bps` bytes a second, and the address it listens on.
    async fn seeder(
        dir: &Path,
        bytes: Vec<u8>,
        bps: u32,
    ) -> (
        Arc<Session>,
        Arc<librqbit::ManagedTorrent>,
        std::net::SocketAddr,
    ) {
        let session = Session::new_with_opts(
            dir.join("seed"),
            SessionOptions {
                dht: None,
                listen: Some(librqbit::ListenerOptions {
                    listen_addr: (std::net::Ipv4Addr::LOCALHOST, 0).into(),
                    ..Default::default()
                }),
                disable_local_service_discovery: true,
                ratelimits: librqbit::limits::LimitsConfig {
                    upload_bps: std::num::NonZeroU32::new(bps),
                    download_bps: None,
                },
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let made = dir.join("made").join("Show");
        let seeding = session
            .add_torrent(
                AddTorrent::from_bytes(bytes),
                Some(AddTorrentOptions {
                    overwrite: true,
                    output_folder: Some(made.to_string_lossy().into_owned()),
                    ..Default::default()
                }),
            )
            .await
            .unwrap()
            .into_handle()
            .unwrap();
        seeding.wait_until_initialized().await.unwrap();
        assert!(seeding.stats().finished, "the seeder has the whole batch");
        let addr = session.listen_addr().unwrap();
        (session, seeding, addr)
    }

    #[test]
    fn a_paused_check_that_stands_still_is_still_looked_at() {
        // A fast-resume check ignores a pause and goes live when it ends, and one waiting
        // for librqbit's turn to check has not moved: neither may lose its pace (#1186).
        let every = std::time::Duration::from_secs(5);
        assert_eq!(check_pace(true, 100, Some(100), every), every);
        assert_eq!(check_pace(true, 0, Some(0), every), every);
        assert_eq!(check_pace(true, 200, Some(100), every), CHECKING_TICK);
        assert_eq!(check_pace(false, 100, Some(100), every), CHECKING_TICK);
    }

    #[test]
    fn a_running_download_librqbit_still_flags_as_paused_is_unflagged() {
        // A fast-resume check asked to pause goes live keeping the flag; lifting the hold
        // then has to clear it, or the list reads the download as paused.
        use librqbit::TorrentStatsState as S;
        assert_eq!(ask(&S::Live, false, true, false), Ask::Unflag);
        assert_eq!(ask(&S::Live, false, false, false), Ask::Nothing);
        assert_eq!(ask(&S::Live, true, true, false), Ask::Pause);
        assert_eq!(ask(&S::Paused, false, false, false), Ask::Start);
        assert_eq!(ask(&S::Error, false, false, true), Ask::Start);
        assert_eq!(ask(&S::Error, false, true, false), Ask::Nothing);
    }

    #[test]
    fn a_record_that_could_not_be_folded_stays_for_the_next_open() {
        // A full disk at open must not cost the pieces: the record waits for an open that
        // can write librqbit's record, and nothing at open removes it meanwhile.
        let dir = tempfile::tempdir().unwrap();
        let (ours, theirs) = (dir.path().join("pieces"), dir.path().join("session"));
        std::fs::create_dir_all(&ours).unwrap();
        std::fs::create_dir_all(&theirs).unwrap();
        std::fs::write(ours.join("aa"), [0b1100_0000u8]).unwrap();
        std::fs::write(theirs.join("aa.bitv"), [0b1000_0000u8]).unwrap();
        // The write's temporary file cannot be made, as on a full disk, even for root.
        let blocked = theirs.join("aa.bitv.tmp");
        std::fs::create_dir_all(&blocked).unwrap();
        assert_eq!(
            fold_progress(&ours, &theirs),
            HashSet::from(["aa".to_string()])
        );
        std::fs::remove_dir(&blocked).unwrap();
        assert_eq!(std::fs::read(ours.join("aa")).unwrap(), [0b1100_0000]);
        assert!(fold_progress(&ours, &theirs).is_empty());
        assert_eq!(
            std::fs::read(theirs.join("aa.bitv")).unwrap(),
            [0b1100_0000]
        );
        assert!(!ours.join("aa").exists());
    }

    /// Opens an engine, adds the batch held, waits out its first check and stops: what an
    /// earlier run leaves for the next open.
    async fn ran_once(dir: &Path, root: &Path, state: &Path) -> String {
        let bytes = batch(dir).await;
        let engine = Engine::open(root, state).await.unwrap();
        let r = release("[G] Show (01-03) [Batch]");
        let peeked = engine.peek(&r, Some(bytes)).await.unwrap();
        let hash = engine
            .add_peeked(&r, "Show", peeked, Vec::new())
            .await
            .unwrap();
        settled(&engine, &hash).await;
        engine.session.stop().await;
        hash
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_record_the_open_could_not_fold_is_kept_through_the_run() {
        // The record holds pieces librqbit's record lacks; dropping it in the same run, at
        // the hold lifting or at the end of the first check, lost them after all.
        let dir = tempfile::tempdir().unwrap();
        let (root, state) = (dir.path().join("Anime"), dir.path().join("state"));
        let hash = ran_once(dir.path(), &root, &state).await;
        let theirs = state.join("session").join(format!("{hash}.bitv"));
        let len = std::fs::read(&theirs).unwrap().len();
        let record = state.join("pieces").join(&hash);
        std::fs::create_dir_all(record.parent().unwrap()).unwrap();
        std::fs::write(&record, vec![0xffu8; len]).unwrap();
        // librqbit's record takes no write at this open.
        std::fs::create_dir_all(state.join("session").join(format!("{hash}.bitv.tmp"))).unwrap();
        let every = std::time::Duration::from_millis(200);
        let engine = Engine::open_ticking(&root, &state, every).await.unwrap();
        engine.set_held(false).await.unwrap();
        settled(&engine, &hash).await;
        engine.keep_progress().await;
        tokio::time::sleep(every * 5).await;
        let kept = std::fs::read(&record).unwrap();
        assert!(
            kept.iter().all(|b| *b == 0xff),
            "the record lost pieces: {kept:?}"
        );
        engine.session.stop().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_remove_of_a_download_in_error_takes_its_files_too() {
        // A torrent librqbit holds in its error state has no storage of its own, and its
        // delete makes one from the shelf; taken off the shelf first, the delete aimed at
        // the library root and the episodes stayed.
        let dir = tempfile::tempdir().unwrap();
        let (root, state) = (dir.path().join("Anime"), dir.path().join("state"));
        let hash = ran_once(dir.path(), &root, &state).await;
        // The episodes are in the library, and the next open checks them in full.
        std::fs::create_dir_all(root.join("Show")).unwrap();
        for i in 1..=3u8 {
            std::fs::write(
                root.join("Show").join(format!("Show - 0{i}.mkv")),
                vec![i; 50_000],
            )
            .unwrap();
        }
        let session = state.join("session");
        std::fs::remove_file(session.join(format!("{hash}.bitv"))).unwrap();
        // librqbit cannot store what the check found, so the torrent lands in error.
        std::fs::create_dir_all(session.join(format!("{hash}.bitv.tmp"))).unwrap();
        let engine = Engine::open(&root, &state).await.unwrap();
        let mut errored = false;
        for _ in 0..200 {
            let state = engine.handle(&hash).unwrap().stats().state;
            if matches!(state, librqbit::TorrentStatsState::Error) {
                errored = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        assert!(errored, "the torrent never failed its check");
        engine
            .control(std::slice::from_ref(&hash), PhoneAction::Remove)
            .await
            .unwrap();
        assert_eq!(on_disk(&root), Vec::<String>::new());
        engine.session.stop().await;
    }

    #[test]
    fn a_record_that_claims_nothing_new_needs_no_flush() {
        // One that takes pieces out, as after a check found some bad, is written at once.
        assert!(within(&[0b1000_0000], Some(&[0b1100_0000])));
        assert!(within(&[0b1100_0000], Some(&[0b1100_0000])));
        assert!(!within(&[0b1110_0000], Some(&[0b1100_0000])));
        assert!(!within(&[0b1000_0000], None));
        assert!(!within(&[0, 0], Some(&[0xff])));
    }

    // The files are held so the check is still running while the record is looked at.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test(flavor = "multi_thread")]
    async fn the_record_goes_while_a_check_runs() {
        // librqbit writes the record a full check leaves before the check ends; the
        // engine's older record must not be on disk then, or a kill folds it back.
        let dir = tempfile::tempdir().unwrap();
        let bytes = batch(dir.path()).await;
        let (root, state) = (dir.path().join("Anime"), dir.path().join("state"));
        let engine = Engine::open(&root, &state).await.unwrap();
        let r = release("[G] Show (01-03) [Batch]");
        let peeked = engine.peek(&r, Some(bytes)).await.unwrap();
        let record = state.join("pieces").join(&peeked.hash);
        std::fs::create_dir_all(record.parent().unwrap()).unwrap();
        std::fs::write(&record, [0xffu8, 0xff]).unwrap();
        let files = engine.shelf.files(&peeked.hash, &root.join("Show"));
        {
            let _checking = files.busy();
            let hash = engine
                .add_peeked(&r, "Show", peeked, Vec::new())
                .await
                .unwrap();
            // Gone by the time the add answers, with no wait: before librqbit could check.
            assert!(
                !record.exists(),
                "the old record is still there while it checks"
            );
            assert!(
                checking(&engine.handle(&hash).unwrap()),
                "the check ran free"
            );
        }
        engine.session.stop().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_check_that_ends_writes_its_pieces_down_again() {
        // A check can take pieces out of librqbit's record; the engine's record of them
        // is written again when the check ends, so the next open never puts them back.
        let dir = tempfile::tempdir().unwrap();
        let bytes = batch(dir.path()).await;
        let (root, state) = (dir.path().join("Anime"), dir.path().join("state"));
        let every = std::time::Duration::from_secs(60);
        let engine = Engine::open_ticking(&root, &state, every).await.unwrap();
        // The first tick's pass has run: from here only the end of a check writes.
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let r = release("[G] Show (01-03) [Batch]");
        let peeked = engine.peek(&r, Some(bytes)).await.unwrap();
        let record = state.join("pieces").join(&peeked.hash);
        std::fs::create_dir_all(record.parent().unwrap()).unwrap();
        std::fs::write(&record, [0xffu8, 0xff]).unwrap();
        engine
            .add_peeked(&r, "Show", peeked, Vec::new())
            .await
            .unwrap();
        let mut claimed = u32::MAX;
        for _ in 0..100 {
            claimed = std::fs::read(&record)
                .map(|b| b.iter().map(|x| x.count_ones()).sum())
                .unwrap_or(u32::MAX);
            if claimed == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        assert_eq!(
            claimed, 0,
            "the record still claims pieces the check did not find"
        );
        engine.session.stop().await;
    }

    #[test]
    fn librqbits_stopped_check_reads_as_a_broken_check() {
        // librqbit's first check gives up with this when it sees a pause, and keeps it as
        // the torrent's error when a start cleared the pause before it looked again.
        let stopped = anyhow::anyhow!("initial check paused");
        assert!(broken_check(Some(&format!("{stopped:?}"))));
        assert!(!broken_check(Some("error opening the file")));
        assert!(!broken_check(None));
    }

    #[test]
    fn the_pieces_kept_go_into_librqbits_record_at_open() {
        let dir = tempfile::tempdir().unwrap();
        let (ours, theirs) = (dir.path().join("pieces"), dir.path().join("session"));
        std::fs::create_dir_all(&ours).unwrap();
        std::fs::create_dir_all(&theirs).unwrap();
        // One torrent librqbit has a record of, one of another length, one it has none of.
        std::fs::write(ours.join("aa"), [0b1100_0000u8, 1]).unwrap();
        std::fs::write(theirs.join("aa.bitv"), [0b1010_0000u8, 0]).unwrap();
        std::fs::write(ours.join("bb"), [0xffu8]).unwrap();
        std::fs::write(theirs.join("bb.bitv"), [0u8, 0]).unwrap();
        std::fs::write(ours.join("cc"), [0xffu8]).unwrap();
        fold_progress(&ours, &theirs);
        // A piece in either record stays.
        assert_eq!(
            std::fs::read(theirs.join("aa.bitv")).unwrap(),
            [0b1110_0000, 1]
        );
        assert!(
            !ours.join("aa").exists(),
            "a folded record has done its job"
        );
        assert_eq!(std::fs::read(theirs.join("bb.bitv")).unwrap(), [0, 0]);
        assert!(!ours.join("bb").exists(), "another length is dropped");
        assert!(
            !theirs.join("cc.bitv").exists(),
            "no record is made for librqbit"
        );
        assert!(
            !ours.join("cc").exists(),
            "one librqbit has no record of is dropped"
        );
    }

    #[test]
    fn a_download_goes_on_from_where_it_was_after_the_process_dies() {
        // librqbit writes down the pieces a torrent has only every 16 MiB and when it
        // finishes, so a process ended part way, the way Android ends one, came back with
        // none of them and fetched it all again (#1181).
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Anime");
        let state = dir.path().join("state");
        let runtime = || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        };
        let first = runtime();
        let (had, total, hash, alive) = first.block_on(async {
            let bytes = batch_of(dir.path(), 300_000).await;
            let (seed, seeding, addr) = seeder(dir.path(), bytes.clone(), 32 * 1024).await;
            // A quick tick, so the one that writes the pieces down comes within the wait.
            let every = std::time::Duration::from_millis(200);
            let engine = Engine::open_ticking(&root, &state, every).await.unwrap();
            engine.set_held(false).await.unwrap();
            let r = release("[G] Show (01-03) [Batch]");
            let mut peeked = engine.peek(&r, Some(bytes)).await.unwrap();
            peeked.peers = vec![addr];
            let hash = engine
                .add_peeked(&r, "Show", peeked, Vec::new())
                .await
                .unwrap();
            // While it checks its files, its progress counts the bytes checked.
            settled(&engine, &hash).await;
            let h = engine.handle(&hash).unwrap();
            let total = h.stats().total_bytes;
            for _ in 0..1200 {
                if h.stats().progress_bytes >= total / 4 {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            // A quarter in, the seeder stops sending, and what came in has settled. The tick
            // writes it down: nothing here asks it to.
            seed.pause(&seeding).await.unwrap();
            let mut had = h.stats().progress_bytes;
            for _ in 0..100 {
                tokio::time::sleep(every).await;
                let now = h.stats().progress_bytes;
                if now == had {
                    break;
                }
                had = now;
            }
            // Waits until the tick's record holds as many pieces as came in.
            let record = state.join("pieces").join(&hash);
            let pieces = (had / 16_384) as u32;
            for _ in 0..300 {
                let kept: u32 = std::fs::read(&record)
                    .map(|b| b.iter().map(|x| x.count_ones()).sum())
                    .unwrap_or(0);
                if kept >= pieces {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            (had, total, hash, (seed, seeding, engine, h))
        });
        // The process dies: none of what librqbit does on a clean stop or a drop runs.
        first.shutdown_background();
        std::mem::forget(alive);
        assert!(had > 0 && had < total, "part way: {had} of {total}");

        runtime().block_on(async {
            let engine = Engine::open(&root, &state).await.unwrap();
            settled(&engine, &hash).await;
            let back = engine.handle(&hash).unwrap().stats().progress_bytes;
            assert!(back >= had, "came back with {back} of the {had} it had");
            engine.session.stop().await;
        });
    }

    // The files are held across the add on purpose: the check runs on another thread and
    // waits for them, which is what keeps it checking while the hold lifts.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test(flavor = "multi_thread")]
    async fn a_hold_lifted_during_the_first_check_lets_the_download_run() {
        // librqbit drops a start asked for while it still checks a torrent's files, and a
        // download brought back held after the process ended is checking just when the
        // app lifts the hold, so it stayed paused (#1186).
        let dir = tempfile::tempdir().unwrap();
        let bytes = batch(dir.path()).await;
        let root = dir.path().join("Anime");
        let engine = Engine::open(&root, &dir.path().join("state"))
            .await
            .unwrap();
        assert!(engine.held());
        let r = release("[G] Show (01-03) [Batch]");
        let peeked = engine.peek(&r, Some(bytes)).await.unwrap();
        // The check reads through the storage, so holding its files keeps it checking.
        let files = engine.shelf.files(&peeked.hash, &root.join("Show"));
        let hash = {
            let _checking = files.busy();
            let hash = engine
                .add_peeked(&r, "Show", peeked, Vec::new())
                .await
                .unwrap();
            // The add's storage is the `Files` held here, so the check really waits.
            assert!(Arc::ptr_eq(&engine.shelf.get(&hash).unwrap(), &files));
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            assert!(matches!(
                engine.handle(&hash).unwrap().stats().state,
                librqbit::TorrentStatsState::Initializing { .. }
            ));
            engine.set_held(false).await.unwrap();
            hash
        };
        assert!(
            runs(&engine, &hash).await,
            "still {:?}",
            engine.handle(&hash).unwrap().stats().state
        );
        engine.session.stop().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_download_held_when_the_process_ended_runs_once_the_hold_lifts() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = batch(dir.path()).await;
        let root = dir.path().join("Anime");
        let state = dir.path().join("state");
        let engine = Engine::open(&root, &state).await.unwrap();
        let r = release("[G] Show (01-03) [Batch]");
        let peeked = engine.peek(&r, Some(bytes)).await.unwrap();
        let hash = engine
            .add_peeked(&r, "Show", peeked, Vec::new())
            .await
            .unwrap();
        settled(&engine, &hash).await;
        engine.session.stop().await;
        drop(engine);

        let engine = Engine::open(&root, &state).await.unwrap();
        engine.set_held(false).await.unwrap();
        assert!(
            runs(&engine, &hash).await,
            "still {:?}",
            engine.handle(&hash).unwrap().stats().state
        );
        engine.session.stop().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn an_entry_left_from_an_earlier_copy_does_not_pause_a_fresh_add() {
        // Entries outlive a torrent librqbit failed to bring back, and a pause on one used
        // to stop the next add of the same release with no pause the person could see.
        let dir = tempfile::tempdir().unwrap();
        let bytes = batch(dir.path()).await;
        let engine = Engine::open(&dir.path().join("Anime"), &dir.path().join("state"))
            .await
            .unwrap();
        engine.set_held(false).await.unwrap();
        let r = release("[G] Show (01-03) [Batch]");
        let peeked = engine.peek(&r, Some(bytes)).await.unwrap();
        let left = Entry {
            paused_by_user: true,
            completion_on: 5,
            ..Default::default()
        };
        recover(&engine.wants.entries).insert(peeked.hash.clone(), left);
        let hash = engine
            .add_peeked(&r, "Show", peeked, Vec::new())
            .await
            .unwrap();
        let entry = recover(&engine.wants.entries)[&hash].clone();
        assert!(!entry.paused_by_user);
        assert_eq!(entry.completion_on, 0);
        assert!(runs(&engine, &hash).await);
        engine.session.stop().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_pause_given_in_capitals_is_kept_for_the_torrent() {
        // The entry was keyed by the hash as given, while the hold looked it up by the
        // torrent's own lowercase one, so lifting the hold undid the pause.
        let dir = tempfile::tempdir().unwrap();
        let bytes = batch(dir.path()).await;
        let engine = Engine::open(&dir.path().join("Anime"), &dir.path().join("state"))
            .await
            .unwrap();
        engine.set_held(false).await.unwrap();
        let r = release("[G] Show (01-03) [Batch]");
        let peeked = engine.peek(&r, Some(bytes)).await.unwrap();
        let hash = engine
            .add_peeked(&r, "Show", peeked, Vec::new())
            .await
            .unwrap();
        settled(&engine, &hash).await;
        engine
            .control(&[hash.to_uppercase()], PhoneAction::Pause)
            .await
            .unwrap();
        engine.set_held(true).await.unwrap();
        engine.set_held(false).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let state = engine.handle(&hash).unwrap().stats().state;
        assert!(
            matches!(state, librqbit::TorrentStatsState::Paused),
            "{state:?}"
        );
        assert!(recover(&engine.wants.entries)[&hash].paused_by_user);
        engine.session.stop().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_pause_on_a_held_download_stays_once_the_hold_lifts() {
        // The hold had paused it already, so librqbit refused the pause and the person's
        // pause was never kept: lifting the hold started it again (#1198).
        let dir = tempfile::tempdir().unwrap();
        let bytes = batch(dir.path()).await;
        let engine = Engine::open(&dir.path().join("Anime"), &dir.path().join("state"))
            .await
            .unwrap();
        let r = release("[G] Show (01-03) [Batch]");
        let peeked = engine.peek(&r, Some(bytes)).await.unwrap();
        let hash = engine
            .add_peeked(&r, "Show", peeked, Vec::new())
            .await
            .unwrap();
        settled(&engine, &hash).await;
        engine
            .control(std::slice::from_ref(&hash), PhoneAction::Pause)
            .await
            .unwrap();
        engine.set_held(false).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let state = engine.handle(&hash).unwrap().stats().state;
        assert!(
            matches!(state, librqbit::TorrentStatsState::Paused),
            "{state:?}"
        );
        engine.session.stop().await;
    }
}
