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
use crate::contract::*;
use crate::downloads::rolling::{NORMAL, SKIP};
use crate::sync::recover;

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
    entries: Mutex<HashMap<String, Entry>>,
    /// Held from taking the list to its landing on disk, so saves land in the order
    /// their lists were taken (#582), while the list itself is let go at once.
    saving: Mutex<()>,
    held: Mutex<bool>,
}

fn fail(e: impl std::fmt::Display) -> CoreError {
    CoreError::service("torrents", "engine", e.to_string())
}

fn now() -> i64 {
    crate::time::now_secs()
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
                    episode: crate::downloads::facts::read_name(&name).episode,
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
        let session = Session::new_with_opts(
            root.to_path_buf(),
            SessionOptions {
                fastresume: true,
                persistence: Some(SessionPersistenceConfig::Json {
                    folder: Some(session_dir),
                }),
                default_storage_factory: Some(shelf.factory(None, root.to_path_buf())),
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
        Ok(Engine {
            session,
            shelf,
            root: root.to_path_buf(),
            entries_path,
            entries: Mutex::new(entries),
            saving: Mutex::new(()),
            held: Mutex::new(true),
        })
    }

    /// Saves the list on its own turn, so two saves from separate paths land in the
    /// order their lists were taken and the newest is the one left on disk (#582). The
    /// list is held only while it is copied, never across the write.
    fn save_entries(&self) {
        let _turn = recover(&self.saving);
        let entries = self.entries.lock().unwrap().clone();
        match serde_json::to_vec_pretty(&entries) {
            Ok(bytes) => {
                if let Err(e) = crate::atomic_file::write(&self.entries_path, bytes) {
                    tracing::warn!("the download list was not saved: {e}");
                }
            }
            Err(e) => tracing::warn!("the download list was not saved: {e}"),
        }
    }

    /// Held from the start: nothing downloads until the app has looked at the network and
    /// said Wi-Fi or allowed mobile data.
    pub fn held(&self) -> bool {
        *self.held.lock().unwrap()
    }

    /// Adds `release` to download into `<root>/<folder>`, held at once when downloads are.
    /// `torrent` is the `.torrent` file when the core fetched it itself (an http link), so
    /// the engine never looks the host up on its own.
    pub async fn add(
        &self,
        release: &Release,
        folder: &str,
        torrent: Option<Vec<u8>>,
    ) -> Result<String, CoreError> {
        let source = match torrent {
            Some(bytes) => AddTorrent::from_bytes(bytes),
            None => AddTorrent::from_url(&release.link),
        };
        // A magnet has to fetch its few kilobytes of metadata before it can pause, so it is
        // added running and held right after: on mobile data only that little goes out.
        self.start(
            release,
            folder,
            source,
            AddTorrentOptions::default(),
            Vec::new(),
        )
        .await
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
                    episode: crate::downloads::facts::read_name(&name).episode,
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
        self.shelf
            .put(&peeked.hash, &self.root.join(folder), wanted.clone());
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
        self.start(
            release,
            folder,
            AddTorrent::from_bytes(peeked.bytes),
            options,
            entry.priorities,
        )
        .await
    }

    async fn start(
        &self,
        release: &Release,
        folder: &str,
        source: AddTorrent<'_>,
        options: AddTorrentOptions,
        priorities: Vec<i64>,
    ) -> Result<String, CoreError> {
        let held = self.held();
        let output = self.root.join(folder);
        let added = within_a_minute(self.session.add_torrent(
            source,
            Some(AddTorrentOptions {
                output_folder: Some(output.to_string_lossy().into_owned()),
                overwrite: true,
                storage_factory: Some(self.shelf.factory(Some(output.clone()), self.root.clone())),
                ..options
            }),
        ))
        .await?;
        let handle = match added {
            AddTorrentResponse::Added(_, h) => h,
            // One the engine has already keeps its record: its files stay where they are.
            AddTorrentResponse::AlreadyManaged(_, h) => return Ok(h.info_hash().as_string()),
            AddTorrentResponse::ListOnly(_) => {
                return Err(fail("the engine only listed the torrent"));
            }
        };
        if held && !handle.is_paused() {
            self.session.pause(&handle).await.map_err(fail)?;
        }
        let hash = handle.info_hash().as_string();
        self.entries.lock().unwrap().insert(
            hash.clone(),
            Entry {
                title: release.title.clone(),
                folder: folder.to_string(),
                added_on: now(),
                priorities,
                ..Default::default()
            },
        );
        self.save_entries();
        Ok(hash)
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
                let mut entries = self.entries.lock().unwrap();
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
                let _ = self.session.pause(&h).await;
            }
            let paused =
                h.is_paused() || matches!(stats.state, librqbit::TorrentStatsState::Paused);
            let live = stats.live.as_ref();
            let speed = live
                .map(|l| (l.download_speed.mbps * 1_048_576.0) as u64)
                .unwrap_or(0);
            let peers = live.map(|l| l.snapshot.peer_stats.live as u64).unwrap_or(0);
            let progress = if stats.total_bytes == 0 {
                0.0
            } else {
                stats.progress_bytes as f64 / stats.total_bytes as f64
            };
            let state = match (&stats.error, stats.finished, paused, &stats.state) {
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
            let facts = crate::downloads::facts::read_name(&name);
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
                tracker: stats.error.clone().unwrap_or_default(),
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
        let entry = self
            .entries
            .lock()
            .unwrap()
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
        let (old, new, by_user, folder) = {
            let entries = self.entries.lock().unwrap();
            let e = entries.get(&key).cloned().unwrap_or_default();
            let mut n = e.clone();
            n.priorities.resize(count, NORMAL);
            for &i in ids.iter().filter(|&&i| i < count) {
                n.priorities[i] = priority;
            }
            (e.wanted(), n, e.paused_by_user, self.root.join(&e.folder))
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
        self.entries
            .lock()
            .unwrap()
            .entry(key.clone())
            .or_default()
            .priorities = new.priorities;
        self.save_entries();
        if h.is_paused() && !by_user && !self.held() && !h.stats().finished {
            self.session.unpause(&h).await.map_err(fail)?;
        }
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

    pub async fn control(&self, hashes: &[String], action: PhoneAction) -> Result<(), CoreError> {
        for hash in hashes {
            let Some(h) = self.handle(hash) else { continue };
            match action {
                PhoneAction::Pause => {
                    self.session.pause(&h).await.map_err(fail)?;
                    self.entries
                        .lock()
                        .unwrap()
                        .entry(hash.clone())
                        .or_default()
                        .paused_by_user = true;
                }
                PhoneAction::Resume => {
                    self.entries
                        .lock()
                        .unwrap()
                        .entry(hash.clone())
                        .or_default()
                        .paused_by_user = false;
                    if !self.held() {
                        self.session.unpause(&h).await.map_err(fail)?;
                    }
                }
                PhoneAction::Remove | PhoneAction::Forget => {
                    let files = action == PhoneAction::Remove;
                    self.session
                        .delete(librqbit::api::TorrentIdOrHash::Hash(h.info_hash()), files)
                        .await
                        .map_err(fail)?;
                    self.entries.lock().unwrap().remove(hash);
                    self.shelf.forget(&h.info_hash().as_string());
                }
            }
        }
        self.save_entries();
        Ok(())
    }

    /// Holds every unfinished download, or lets the ones the person did not pause run again.
    pub async fn set_held(&self, held: bool) -> Result<(), CoreError> {
        *self.held.lock().unwrap() = held;
        let handles: Vec<_> = self
            .session
            .with_torrents(|it| it.map(|(_, h)| h.clone()).collect());
        for h in handles {
            if h.stats().finished {
                continue;
            }
            let by_user = self
                .entries
                .lock()
                .unwrap()
                .get(&h.info_hash().as_string())
                .is_some_and(|e| e.paused_by_user);
            // One torrent the engine will not pause or start (in error, say) must not leave
            // the rest running on mobile data: each is tried, failures only logged.
            let r = if held && !h.is_paused() {
                self.session.pause(&h).await
            } else if !held && !by_user && h.is_paused() {
                self.session.unpause(&h).await
            } else {
                Ok(())
            };
            if let Err(e) = r {
                tracing::warn!("hold {held} for {}: {e}", h.info_hash().as_string());
            }
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
        let src = dir.join("made").join("Show");
        std::fs::create_dir_all(&src).unwrap();
        for i in 1..=3u8 {
            std::fs::write(src.join(format!("Show - 0{i}.mkv")), vec![i; 50_000]).unwrap();
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
}
