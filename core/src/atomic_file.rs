//! Writing a whole file so that a reader sees the old contents or the new ones, never half
//! of either (#436). The bytes go to a sibling named `<name>.tmp` first, then a rename moves
//! it over the target, which the OS does in one step on the same filesystem. The parent
//! folder is made first when it is missing, and a `.tmp` a failed write leaves is removed.
//!
//! Two saves of one file in this process take turns (#582). They share the one `.tmp`
//! name, so without turns the second would truncate the first's copy while the first
//! renames it into place, then go on writing into what is now the live file. Taking turns
//! keeps the fixed name, so a `.tmp` a crash left is reused and replaced on the next save
//! instead of piling up, and the secrets file's mode fix over a wide leftover still holds.
//! A path is locked as it is spelled: callers name each file one way.
//!
//! The files the core and the shell keep for themselves and rewrite whole go through here,
//! so a later hardening (an fsync on every write, keeping the old file's mode) is made once.
//! [`write_private`] already does both for the secrets file.

use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError};

use crate::contract::CoreError;

/// A write that failed, with the path the failing step touched: the parent folder, the
/// temporary sibling or the target.
#[derive(Debug, thiserror::Error)]
#[error("{}: {source}", path.display())]
pub struct WriteError {
    pub path: PathBuf,
    #[source]
    pub source: io::Error,
}

impl From<WriteError> for CoreError {
    fn from(e: WriteError) -> CoreError {
        CoreError::io_at(e.path.to_string_lossy(), e.source)
    }
}

/// Keeps the kind for callers that match on it, and the path in the message.
impl From<WriteError> for io::Error {
    fn from(e: WriteError) -> io::Error {
        io::Error::new(e.source.kind(), e)
    }
}

fn at(path: &Path) -> impl FnOnce(io::Error) -> WriteError + '_ {
    move |source| WriteError {
        path: path.to_path_buf(),
        source,
    }
}

/// `<name>.tmp` beside `target`. Errors when `target` names no file, such as `/` or `..`.
fn tmp_sibling(target: &Path) -> Result<PathBuf, WriteError> {
    let mut name = OsString::from(target.file_name().ok_or_else(|| WriteError {
        path: target.to_path_buf(),
        source: io::Error::new(io::ErrorKind::InvalidInput, "the path names no file"),
    })?);
    name.push(".tmp");
    Ok(target.with_file_name(name))
}

fn parent(target: &Path) -> Option<&Path> {
    target.parent().filter(|p| !p.as_os_str().is_empty())
}

/// One turn per file being saved right now. An entry lives only while a save of its file
/// holds or waits for it, so the map stays as small as the saves under way.
static LOCKS: LazyLock<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> = LazyLock::new(Default::default);

fn locks() -> MutexGuard<'static, HashMap<PathBuf, Arc<Mutex<()>>>> {
    LOCKS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Runs `save` on its own turn for `target`, after any save of the same file already
/// under way.
fn in_turn<T>(target: &Path, save: impl FnOnce() -> T) -> T {
    let turn = Arc::clone(locks().entry(target.to_path_buf()).or_default());
    let out = {
        let _held = turn.lock().unwrap_or_else(PoisonError::into_inner);
        save()
    };
    let mut all = locks();
    // Clones are made and given up only under the map's lock, so two (the map's and this
    // one) means no other save holds or waits for this file. This one goes before the map
    // is let go, or a save finishing beside it could count it and leave the entry behind.
    let last = Arc::strong_count(&turn) == 2;
    drop(turn);
    if last {
        all.remove(target);
    }
    out
}

/// The one copy of the sequence: make the folder, fill the sibling with `fill`, rename it
/// over the target, on the file's own turn. A failed fill or rename removes the sibling,
/// so a half-written copy (of the secrets, say) does not stay on disk.
fn replace(target: &Path, fill: impl FnOnce(&Path) -> io::Result<()>) -> Result<(), WriteError> {
    in_turn(target, || replace_now(target, fill))
}

fn replace_now(
    target: &Path,
    fill: impl FnOnce(&Path) -> io::Result<()>,
) -> Result<(), WriteError> {
    let tmp = tmp_sibling(target)?;
    if let Some(dir) = parent(target) {
        std::fs::create_dir_all(dir).map_err(at(dir))?;
    }
    let done = fill(&tmp)
        .map_err(at(&tmp))
        .and_then(|()| std::fs::rename(&tmp, target).map_err(at(target)));
    if done.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    done
}

/// Replaces `target` with `bytes` in one step.
pub fn write(target: &Path, bytes: impl AsRef<[u8]>) -> Result<(), WriteError> {
    replace(target, |tmp| std::fs::write(tmp, bytes))
}

/// [`write`] for a file only its owner may read: the new file is mode 0600 on Unix, and its
/// bytes reach the disk before the rename, so a crash leaves the old file or the whole new
/// one. Windows has no mode bits; the file sits in the user's own profile there.
pub fn write_private(target: &Path, bytes: impl AsRef<[u8]>) -> Result<(), WriteError> {
    replace(target, |tmp| {
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(tmp)?;
        file.write_all(bytes.as_ref())?;
        file.sync_all()?;
        drop(file);
        // `mode` above only applies when the open created the file, and a crashed write
        // can leave one behind with a wider mode, so say it again before the rename
        // carries it over the real file.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(tmp, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    })
}

/// [`write`] for async callers: one hop to tokio's blocking pool, with the bytes moved in
/// rather than copied.
pub async fn write_async(target: &Path, bytes: impl Into<Vec<u8>>) -> Result<(), WriteError> {
    let owned = target.to_path_buf();
    let bytes = bytes.into();
    tokio::task::spawn_blocking(move || write(&owned, bytes))
        .await
        .map_err(|e| WriteError {
            path: target.to_path_buf(),
            source: io::Error::other(e),
        })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tmp_file_is_a_sibling_of_the_target() {
        assert_eq!(
            tmp_sibling(Path::new("/home/user/export.json")).unwrap(),
            PathBuf::from("/home/user/export.json.tmp")
        );
        assert_eq!(
            tmp_sibling(Path::new("rules.json")).unwrap(),
            PathBuf::from("rules.json.tmp")
        );
    }

    #[test]
    fn a_path_that_names_no_file_is_refused_before_anything_is_written() {
        let e = tmp_sibling(Path::new("/")).unwrap_err();
        assert_eq!(e.source.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(e.path, PathBuf::from("/"));
    }

    #[test]
    fn a_write_replaces_the_file_and_leaves_no_tmp_behind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.json");
        std::fs::write(&path, b"old").unwrap();
        write(&path, b"new").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        assert!(!dir.path().join("a.json.tmp").exists());
    }

    #[test]
    fn a_write_makes_the_missing_parent_folders() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x/y/z.ics");
        write(&path, "text").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "text");
    }

    #[test]
    fn a_bare_file_name_has_no_folder_to_make() {
        // `Path::parent` of a bare name is "", which must not reach create_dir_all as a
        // folder. A bare name only resolves against the cwd, so the rule is checked here
        // rather than by writing into the cwd.
        assert_eq!(parent(Path::new("rules.json")), None);
        assert_eq!(parent(Path::new("a/rules.json")), Some(Path::new("a")));
    }

    #[test]
    fn a_tmp_that_cannot_be_written_leaves_the_old_file_and_names_the_tmp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rules.json");
        std::fs::write(&path, b"old").unwrap();
        std::fs::create_dir(dir.path().join("rules.json.tmp")).unwrap();
        let e = write(&path, b"new").unwrap_err();
        assert_eq!(e.path, dir.path().join("rules.json.tmp"));
        assert_eq!(std::fs::read(&path).unwrap(), b"old");
    }

    #[test]
    fn a_rename_that_fails_names_the_target_and_removes_the_tmp() {
        let dir = tempfile::tempdir().unwrap();
        // A folder with something in it cannot be replaced by a file.
        let path = dir.path().join("taken");
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("inside"), b"").unwrap();
        let e = write(&path, b"new").unwrap_err();
        assert_eq!(e.path, path);
        assert!(!dir.path().join("taken.tmp").exists());
    }

    #[test]
    fn a_parent_that_is_a_file_names_the_parent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file"), b"").unwrap();
        let path = dir.path().join("file/inner.json");
        let e = write(&path, b"x").unwrap_err();
        assert_eq!(e.path, dir.path().join("file"));
    }

    #[test]
    fn the_error_becomes_an_io_error_at_its_path() {
        let e = || WriteError {
            path: PathBuf::from("/x/y.tmp"),
            source: io::Error::new(io::ErrorKind::StorageFull, "disk full"),
        };
        match CoreError::from(e()) {
            CoreError::Io { path, message } => {
                assert_eq!(path.as_deref(), Some("/x/y.tmp"));
                assert_eq!(message, "disk full");
            }
            other => panic!("{other:?}"),
        }
        let io = io::Error::from(e());
        assert_eq!(io.kind(), io::ErrorKind::StorageFull);
        assert_eq!(io.to_string(), "/x/y.tmp: disk full");
    }

    #[cfg(unix)]
    #[test]
    fn a_private_write_is_owner_only_even_over_a_wide_leftover_tmp() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");
        let tmp = dir.path().join("secrets.json.tmp");
        std::fs::write(&tmp, b"crashed").unwrap();
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o644)).unwrap();
        write_private(&path, b"{}").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        assert_eq!(std::fs::read(&path).unwrap(), b"{}");
        assert!(!tmp.exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_private_write_that_fails_leaves_no_copy_of_the_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");
        // The rename cannot put a file over a folder that holds something.
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("inside"), b"").unwrap();
        assert!(write_private(&path, b"{\"token\":\"x\"}").is_err());
        assert!(!dir.path().join("secrets.json.tmp").exists());
    }

    /// #582: saves of one file that overlap (the phone's download list from two async
    /// paths, two fetches of one image) each finish whole, in turn: none writes into a
    /// `.tmp` another has just renamed into place, so every save succeeds, a reader never
    /// sees half a file, and no `.tmp` is left.
    #[test]
    fn overlapping_saves_of_one_file_each_land_whole() {
        const SIZE: usize = 256 * 1024;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("entries.json");
        write(&path, vec![b'a'; SIZE]).unwrap();
        let stop = std::sync::atomic::AtomicBool::new(false);
        std::thread::scope(|scope| {
            // Reading while another rename lands over the file is a Unix guarantee; on
            // Windows an open reader can refuse the rename, so only the writers run there.
            let reader = scope.spawn(|| {
                let mut torn = 0;
                while cfg!(unix) && !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    if let Ok(bytes) = std::fs::read(&path)
                        && (bytes.len() != SIZE || bytes.iter().any(|b| *b != bytes[0]))
                    {
                        torn += 1;
                    }
                }
                torn
            });
            let writers: Vec<_> = (0..8u8)
                .map(|i| {
                    let path = &path;
                    scope.spawn(move || {
                        (0..20)
                            .map(|_| write(path, vec![b'b' + i; SIZE]).map_err(|e| e.to_string()))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            let failed: Vec<String> = writers
                .into_iter()
                .flat_map(|w| w.join().unwrap())
                .filter_map(Result::err)
                .collect();
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
            assert!(failed.is_empty(), "every save lands: {failed:?}");
            assert_eq!(reader.join().unwrap(), 0, "a reader never sees half a file");
        });
        let last = std::fs::read(&path).unwrap();
        assert_eq!(last.len(), SIZE);
        assert!(last.iter().all(|b| *b == last[0]));
        assert!(!dir.path().join("entries.json.tmp").exists());
        assert!(
            !LOCKS.lock().unwrap().contains_key(&path),
            "the file's turn is given up once the last save ends"
        );
    }

    #[test]
    fn a_file_gives_up_its_lock_once_no_save_holds_it() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..50 {
            write(&dir.path().join(format!("{i}.json")), b"x").unwrap();
        }
        let held = LOCKS.lock().unwrap();
        assert!(
            held.keys().all(|k| !k.starts_with(dir.path())),
            "the locks of files no one is saving are dropped"
        );
    }

    #[tokio::test]
    async fn an_async_write_does_what_the_sync_one_does() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deep/export.json");
        write_async(&path, b"one".to_vec()).await.unwrap();
        write_async(&path, "two").await.unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
        assert!(!dir.path().join("deep/export.json.tmp").exists());

        std::fs::create_dir(dir.path().join("deep/export.json.tmp")).unwrap();
        let e = write_async(&path, "three").await.unwrap_err();
        assert_eq!(e.path, dir.path().join("deep/export.json.tmp"));
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
    }
}
