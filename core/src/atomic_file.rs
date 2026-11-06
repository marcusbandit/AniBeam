//! Writing a whole file so that a reader sees the old contents or the new ones, never half
//! of either (#436). The bytes go to a sibling named `<name>.tmp` first, then a rename moves
//! it over the target, which the OS does in one step on the same filesystem. The parent
//! directory is made first when it is missing.
//!
//! Every file the core rewrites in place goes through here, so a later hardening (an fsync
//! on every write, keeping the old file's mode) is made once. [`write_private`] already
//! does both for the secrets file.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use crate::contract::CoreError;

/// A write that failed, with the path the failing step touched: the parent directory, the
/// temporary sibling or the target.
#[derive(Debug)]
pub struct WriteError {
    pub path: PathBuf,
    pub source: io::Error,
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.source)
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<WriteError> for CoreError {
    fn from(e: WriteError) -> CoreError {
        CoreError::io_at(e.path.to_string_lossy(), e.source)
    }
}

impl From<WriteError> for io::Error {
    fn from(e: WriteError) -> io::Error {
        e.source
    }
}

fn at(path: &Path) -> impl FnOnce(io::Error) -> WriteError + '_ {
    move |source| WriteError {
        path: path.to_path_buf(),
        source,
    }
}

/// `<name>.tmp` beside `target`. Errors when `target` names no file, such as `/` or `..`.
pub fn tmp_sibling(target: &Path) -> Result<PathBuf, WriteError> {
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

/// Replaces `target` with `bytes` in one step.
pub fn write(target: &Path, bytes: impl AsRef<[u8]>) -> Result<(), WriteError> {
    let tmp = tmp_sibling(target)?;
    if let Some(dir) = parent(target) {
        std::fs::create_dir_all(dir).map_err(at(dir))?;
    }
    std::fs::write(&tmp, bytes).map_err(at(&tmp))?;
    std::fs::rename(&tmp, target).map_err(at(target))
}

/// [`write`] for a file only its owner may read: the new file is mode 0600 on Unix, and its
/// bytes reach the disk before the rename, so a crash leaves the old file or the whole new
/// one. Windows has no mode bits; the file sits in the user's own profile there.
pub fn write_private(target: &Path, bytes: impl AsRef<[u8]>) -> Result<(), WriteError> {
    use std::io::Write;
    let tmp = tmp_sibling(target)?;
    if let Some(dir) = parent(target) {
        std::fs::create_dir_all(dir).map_err(at(dir))?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&tmp).map_err(at(&tmp))?;
    file.write_all(bytes.as_ref()).map_err(at(&tmp))?;
    file.sync_all().map_err(at(&tmp))?;
    drop(file);
    // `mode` above only applies when the open created the file, and a crashed write can
    // leave one behind with a wider mode, so say it again before the rename carries it over
    // the real file.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600)).map_err(at(&tmp))?;
    }
    std::fs::rename(&tmp, target).map_err(at(target))
}

/// [`write`] on tokio's blocking pool, for async callers.
pub async fn write_async(target: &Path, bytes: impl AsRef<[u8]>) -> Result<(), WriteError> {
    let tmp = tmp_sibling(target)?;
    if let Some(dir) = parent(target) {
        tokio::fs::create_dir_all(dir).await.map_err(at(dir))?;
    }
    tokio::fs::write(&tmp, bytes.as_ref())
        .await
        .map_err(at(&tmp))?;
    tokio::fs::rename(&tmp, target).await.map_err(at(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tmp_file_is_a_sibling_of_the_target() {
        assert_eq!(
            tmp_sibling(Path::new("/home/bandit/export.json")).unwrap(),
            PathBuf::from("/home/bandit/export.json.tmp")
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
    fn a_relative_file_name_with_no_folder_writes_in_place() {
        // `Path::parent` of a bare name is "", which must not reach create_dir_all as a
        // folder to make. A bare relative name only resolves against the cwd, so the test
        // checks the parent rule rather than writing into the cwd.
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
    fn a_rename_that_fails_names_the_target() {
        let dir = tempfile::tempdir().unwrap();
        // A folder with something in it cannot be replaced by a file.
        let path = dir.path().join("taken");
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("inside"), b"").unwrap();
        let e = write(&path, b"new").unwrap_err();
        assert_eq!(e.path, path);
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
        let e = WriteError {
            path: PathBuf::from("/x/y.tmp"),
            source: io::Error::other("disk full"),
        };
        match CoreError::from(e) {
            CoreError::Io { path, message } => {
                assert_eq!(path.as_deref(), Some("/x/y.tmp"));
                assert_eq!(message, "disk full");
            }
            other => panic!("{other:?}"),
        }
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

    #[tokio::test]
    async fn an_async_write_does_what_the_sync_one_does() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deep/export.json");
        write_async(&path, b"one".to_vec()).await.unwrap();
        write_async(&path, b"two").await.unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
        assert!(!dir.path().join("deep/export.json.tmp").exists());

        std::fs::create_dir(dir.path().join("deep/export.json.tmp")).unwrap();
        let e = write_async(&path, b"three").await.unwrap_err();
        assert_eq!(e.path, dir.path().join("deep/export.json.tmp"));
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
    }
}
