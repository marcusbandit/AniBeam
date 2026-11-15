//! Where the phone's engine keeps a torrent's bytes. librqbit's own storage opens every file
//! of a torrent at once, the skipped ones included, so a batch held to a few episodes
//! (`downloads::rolling`) would leave an empty file for every other episode in the library
//! folder, and the library would list them all as there. This one keeps qBittorrent's
//! answer: a file the torrent fetches is written where it belongs, created on its first
//! write; the bytes of a skipped file that a shared piece brings in go to a part file in the
//! engine's own folder, outside the library. When the file is wanted again its part file
//! moves into place, so nothing fetched is fetched twice.
//!
//! The state of each torrent, its folder and which files it fetches, is shared by the
//! storage librqbit holds and the engine, which changes the selection.

use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use crate::sync::{recover, recover_read, recover_write};
use anyhow::Context;
use librqbit::storage::filesystem::FilesystemStorageFactory;
use librqbit::storage::{BoxStorageFactory, StorageFactory, TorrentStorage};
use librqbit::{ManagedTorrentShared, TorrentMetadata};

/// One torrent's files.
pub struct Files {
    folder: PathBuf,
    parts: PathBuf,
    names: RwLock<Vec<PathBuf>>,
    /// The files the torrent fetches; None while it fetches them all.
    wanted: RwLock<Option<HashSet<usize>>>,
    /// Open files, each with whether it is the real one rather than a part file.
    open: Mutex<HashMap<usize, (bool, Arc<File>)>>,
}

impl Files {
    fn new(folder: PathBuf, parts: PathBuf, wanted: Option<HashSet<usize>>) -> Files {
        Files {
            folder,
            parts,
            names: RwLock::new(Vec::new()),
            wanted: RwLock::new(wanted),
            open: Mutex::new(HashMap::new()),
        }
    }

    /// Changes which files the torrent fetches. A file wanted again moves its part file
    /// into place on its next write.
    pub fn set_wanted(&self, wanted: Option<HashSet<usize>>) {
        *recover_write(&self.wanted) = wanted;
    }

    fn is_wanted(&self, id: usize) -> bool {
        recover_read(&self.wanted)
            .as_ref()
            .is_none_or(|w| w.contains(&id))
    }

    /// Lets go of every open file, so space a deleted file held comes back.
    pub fn close(&self) {
        recover(&self.open).clear();
    }

    fn real(&self, id: usize) -> anyhow::Result<PathBuf> {
        let names = recover_read(&self.names);
        let name = names.get(id).context("no such file")?;
        Ok(self.folder.join(name))
    }

    fn part(&self, id: usize) -> PathBuf {
        self.parts.join(id.to_string())
    }

    /// The file to read or write `id` through. Writing creates it: the real file when the
    /// torrent wants it or it is there already, else the part file. A wanted file whose bytes
    /// so far are in a part file takes that file over first.
    fn file(&self, id: usize, write: bool) -> anyhow::Result<Arc<File>> {
        let wanted = self.is_wanted(id);
        let mut open = recover(&self.open);
        match open.get(&id) {
            Some((real, f)) if *real || !wanted => return Ok(f.clone()),
            // Wanted now, and still open as its part file: move it into place below.
            Some(_) => {
                open.remove(&id);
            }
            None => {}
        }
        let real = self.real(id)?;
        let part = self.part(id);
        let (is_real, path) = if real.exists() {
            (true, real)
        } else if wanted && part.exists() {
            if let Some(dir) = real.parent() {
                std::fs::create_dir_all(dir)?;
            }
            move_file(&part, &real)?;
            (true, real)
        } else if wanted && write {
            (true, real)
        } else if write || part.exists() {
            (false, part)
        } else {
            anyhow::bail!("{} is not on disk", real.display());
        };
        if write && let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(write)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("opening {}", path.display()))?;
        let f = Arc::new(f);
        open.insert(id, (is_real, f.clone()));
        Ok(f)
    }

    /// Takes the torrent's part files off the disk.
    pub fn remove_parts(&self) {
        self.close();
        if let Err(e) = std::fs::remove_dir_all(&self.parts)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!("part files at {} not removed: {e}", self.parts.display());
        }
    }
}

/// A rename, or a copy where the two sit on different file systems.
fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    match std::fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(_) => {
            std::fs::copy(from, to)?;
            std::fs::remove_file(from)
        }
    }
}

/// Every torrent's [`Files`], by info-hash, shared by the engine and the storage.
#[derive(Clone)]
pub struct Shelf {
    parts: PathBuf,
    torrents: Arc<Mutex<HashMap<String, Arc<Files>>>>,
}

impl Shelf {
    pub fn new(parts: PathBuf) -> Shelf {
        Shelf {
            parts,
            torrents: Arc::default(),
        }
    }

    /// The torrent's files, made for `folder` when it has none yet.
    pub fn files(&self, hash: &str, folder: &Path) -> Arc<Files> {
        recover(&self.torrents)
            .entry(hash.to_string())
            .or_insert_with(|| {
                Arc::new(Files::new(
                    folder.to_path_buf(),
                    self.parts.join(hash),
                    None,
                ))
            })
            .clone()
    }

    /// Sets a torrent up before librqbit opens it: its folder and the files it fetches.
    pub fn put(&self, hash: &str, folder: &Path, wanted: Option<HashSet<usize>>) {
        let files = self.files(hash, folder);
        files.set_wanted(wanted);
    }

    pub fn get(&self, hash: &str) -> Option<Arc<Files>> {
        recover(&self.torrents).get(hash).cloned()
    }

    pub fn forget(&self, hash: &str) {
        if let Some(files) = recover(&self.torrents).remove(hash) {
            files.remove_parts();
        }
    }

    /// The storage for torrents added now into `folder`, or, with None, for the ones the
    /// session brings back, which the shelf was given at open.
    pub fn factory(&self, folder: Option<PathBuf>, fallback: PathBuf) -> BoxStorageFactory {
        Box::new(Factory {
            shelf: self.clone(),
            folder,
            fallback,
        })
    }
}

#[derive(Clone)]
struct Factory {
    shelf: Shelf,
    folder: Option<PathBuf>,
    /// Where a torrent the shelf does not know writes: the library root.
    fallback: PathBuf,
}

impl StorageFactory for Factory {
    type Storage = Box<dyn TorrentStorage>;

    fn create(
        &self,
        shared: &ManagedTorrentShared,
        _metadata: &TorrentMetadata,
    ) -> anyhow::Result<Box<dyn TorrentStorage>> {
        let hash = shared.info_hash.as_string();
        let files = match (&self.folder, self.shelf.get(&hash)) {
            (Some(folder), _) => self.shelf.files(&hash, folder),
            (None, Some(files)) => files,
            (None, None) => self.shelf.files(&hash, &self.fallback),
        };
        Ok(Box::new(Storage { files }))
    }

    /// The files sit where librqbit's own storage puts them, each at its path under the
    /// torrent's folder, so the session's saved record of a torrent holds for this one too.
    /// librqbit's JSON record refuses any other storage, and its own middlewares answer the
    /// same way for the filesystem storage under them.
    fn is_type_id(&self, type_id: std::any::TypeId) -> bool {
        type_id == std::any::TypeId::of::<FilesystemStorageFactory>()
    }

    fn clone_box(&self) -> BoxStorageFactory {
        Box::new(self.clone())
    }
}

pub struct Storage {
    files: Arc<Files>,
}

impl TorrentStorage for Storage {
    fn init(
        &mut self,
        _shared: &ManagedTorrentShared,
        metadata: &TorrentMetadata,
    ) -> anyhow::Result<()> {
        *recover_write(&self.files.names) = metadata
            .file_infos
            .iter()
            .map(|f| f.relative_filename.clone())
            .collect();
        std::fs::create_dir_all(&self.files.folder)?;
        Ok(())
    }

    fn pread_exact(&self, file_id: usize, offset: u64, buf: &mut [u8]) -> anyhow::Result<()> {
        Ok(self
            .files
            .file(file_id, false)?
            .read_exact_at(buf, offset)?)
    }

    fn pwrite_all(&self, file_id: usize, offset: u64, buf: &[u8]) -> anyhow::Result<()> {
        Ok(self.files.file(file_id, true)?.write_all_at(buf, offset)?)
    }

    fn remove_file(&self, file_id: usize, filename: &Path) -> anyhow::Result<()> {
        recover(&self.files.open).remove(&file_id);
        for path in [self.files.folder.join(filename), self.files.part(file_id)] {
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    fn remove_directory_if_empty(&self, path: &Path) -> anyhow::Result<()> {
        let path = self.files.folder.join(path);
        if path.is_dir() && std::fs::read_dir(&path)?.next().is_none() {
            std::fs::remove_dir(&path)?;
        }
        Ok(())
    }

    /// Only a file the torrent fetches is asked for, so the real one is made at its length,
    /// as librqbit's own storage does.
    fn ensure_file_length(&self, file_id: usize, length: u64) -> anyhow::Result<()> {
        Ok(self.files.file(file_id, true)?.set_len(length)?)
    }

    /// A paused torrent lets go of its files; the next one opens them again.
    fn take(&self) -> anyhow::Result<Box<dyn TorrentStorage>> {
        self.files.close();
        Ok(Box::new(Storage {
            files: self.files.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(dir: &Path, names: &[&str], wanted: Option<&[usize]>) -> Files {
        let f = Files::new(
            dir.join("lib"),
            dir.join("parts"),
            wanted.map(|w| w.iter().copied().collect()),
        );
        *f.names.write().unwrap() = names.iter().map(PathBuf::from).collect();
        f
    }

    #[test]
    fn a_skipped_file_writes_outside_the_library_and_moves_in_when_wanted() {
        let dir = tempfile::tempdir().unwrap();
        let f = files(dir.path(), &["Show/01.mkv", "Show/02.mkv"], Some(&[0]));
        f.file(0, true).unwrap().write_all_at(b"one", 0).unwrap();
        // A piece shared with episode 2 brings a few of its bytes in.
        f.file(1, true).unwrap().write_all_at(b"tw", 0).unwrap();
        assert!(dir.path().join("lib/Show/01.mkv").exists());
        assert!(!dir.path().join("lib/Show/02.mkv").exists());
        assert!(dir.path().join("parts/1").exists());
        // Read back from where it went.
        let mut buf = [0u8; 2];
        f.file(1, false)
            .unwrap()
            .read_exact_at(&mut buf, 0)
            .unwrap();
        assert_eq!(&buf, b"tw");

        f.set_wanted(Some([0, 1].into()));
        f.file(1, true).unwrap().write_all_at(b"o", 2).unwrap();
        assert!(!dir.path().join("parts/1").exists());
        assert_eq!(
            std::fs::read(dir.path().join("lib/Show/02.mkv")).unwrap(),
            b"two"
        );
    }

    #[test]
    fn nothing_is_made_for_a_skipped_file_that_is_only_read() {
        let dir = tempfile::tempdir().unwrap();
        let f = files(dir.path(), &["a.mkv", "b.mkv"], Some(&[0]));
        assert!(f.file(1, false).is_err());
        assert!(!dir.path().join("lib/b.mkv").exists());
        assert!(!dir.path().join("parts/1").exists());
    }

    #[test]
    fn a_file_already_in_place_is_used_even_when_skipped() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("lib")).unwrap();
        std::fs::write(dir.path().join("lib/a.mkv"), b"abc").unwrap();
        let f = files(dir.path(), &["a.mkv"], Some(&[]));
        f.file(0, true).unwrap().write_all_at(b"z", 3).unwrap();
        assert_eq!(
            std::fs::read(dir.path().join("lib/a.mkv")).unwrap(),
            b"abcz"
        );
        assert!(!dir.path().join("parts/0").exists());
    }

    #[test]
    fn everything_is_wanted_without_a_selection() {
        let dir = tempfile::tempdir().unwrap();
        let f = files(dir.path(), &["a.mkv"], None);
        f.file(0, true).unwrap().write_all_at(b"x", 0).unwrap();
        assert!(dir.path().join("lib/a.mkv").exists());
    }
}
