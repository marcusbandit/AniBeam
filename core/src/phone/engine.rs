//! The phone's torrent engine: librqbit inside the core, writing into the library folder.
//! It answers in qBittorrent's shapes (`Torrent`, its state words) so the pages and the
//! library overlay read it the way they read the desktop's client.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, Session, SessionOptions,
    SessionPersistenceConfig,
};
use serde::{Deserialize, Serialize};

use crate::contract::*;

/// What the engine itself does not keep: each torrent's title, folder and times, and
/// whether the person paused it (a hold for Wi-Fi resumes only the others).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Entry {
    title: String,
    folder: String,
    added_on: i64,
    completion_on: i64,
    paused_by_user: bool,
}

pub struct Engine {
    session: Arc<Session>,
    root: PathBuf,
    entries_path: PathBuf,
    entries: Mutex<HashMap<String, Entry>>,
    held: Mutex<bool>,
}

fn fail(e: impl std::fmt::Display) -> CoreError {
    CoreError::Service {
        service: "torrents".to_string(),
        code: "engine".to_string(),
        message: e.to_string(),
    }
}

fn now() -> i64 {
    crate::time::now_secs() as i64
}

impl Engine {
    /// Opens the session: torrents write under `root` (the library folder), and the engine's
    /// own state lives in `state_dir`, so downloads pick up where they were after a restart.
    pub async fn open(root: &Path, state_dir: &Path) -> Result<Engine, CoreError> {
        std::fs::create_dir_all(root).map_err(fail)?;
        std::fs::create_dir_all(state_dir).map_err(fail)?;
        let session = Session::new_with_opts(
            root.to_path_buf(),
            SessionOptions {
                fastresume: true,
                persistence: Some(SessionPersistenceConfig::Json {
                    folder: Some(state_dir.join("session")),
                }),
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
        let entries_path = state_dir.join("torrents.json");
        let entries = std::fs::read(&entries_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Ok(Engine {
            session,
            root: root.to_path_buf(),
            entries_path,
            entries: Mutex::new(entries),
            held: Mutex::new(true),
        })
    }

    fn save_entries(&self) {
        let entries = self.entries.lock().unwrap().clone();
        if let Ok(bytes) = serde_json::to_vec_pretty(&entries) {
            let tmp = self.entries_path.with_extension("json.tmp");
            if std::fs::write(&tmp, bytes).is_ok() {
                let _ = std::fs::rename(tmp, &self.entries_path);
            }
        }
    }

    /// Held from the start: nothing downloads until the app has looked at the network and
    /// said Wi-Fi or allowed mobile data.
    pub fn held(&self) -> bool {
        *self.held.lock().unwrap()
    }

    /// Adds `release` to download into `<root>/<folder>`, held at once when downloads are.
    pub async fn add(&self, release: &Release, folder: &str) -> Result<String, CoreError> {
        let held = self.held();
        let output = self.root.join(folder);
        let added = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            self.session.add_torrent(
                AddTorrent::from_url(&release.link),
                Some(AddTorrentOptions {
                    output_folder: Some(output.to_string_lossy().into_owned()),
                    overwrite: true,
                    ..Default::default()
                }),
            ),
        )
        .await;
        let added = match added {
            Ok(Ok(added)) => added,
            Ok(Err(e)) => return Err(fail(e)),
            Err(_) => {
                return Err(fail(
                    "no peer sent the torrent's details within a minute; try a release with more seeders",
                ));
            }
        };
        let handle = match added {
            AddTorrentResponse::Added(_, h) | AddTorrentResponse::AlreadyManaged(_, h) => h,
            AddTorrentResponse::ListOnly(_) => {
                return Err(fail("the engine only listed the torrent"));
            }
        };
        // A magnet has to fetch its few kilobytes of metadata before it can pause, so it is
        // added running and held right after: on mobile data only that little goes out.
        if held {
            self.session.pause(&handle).await.map_err(fail)?;
        }
        let hash = handle.info_hash().as_string();
        self.entries.lock().unwrap().insert(
            hash.clone(),
            Entry {
                title: release.title.clone(),
                folder: folder.to_string(),
                added_on: now(),
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
            let mut entries = self.entries.lock().unwrap();
            let entry = entries.entry(hash.clone()).or_default();
            if stats.finished && entry.completion_on == 0 {
                entry.completion_on = now();
                changed = true;
            }
            let entry = entry.clone();
            drop(entries);
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
            let files = h
                .with_metadata(|m| {
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
                                priority: 1,
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
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
