use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::contract::CoreError;

/// Where the core keeps its files. Every field is a directory path as a
/// String, per the contract's closed type set; the accessors return PathBuf
/// for the code inside the crate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct CorePaths {
    pub data_dir: String,
    pub config_dir: String,
    pub cache_dir: String,
    pub state_dir: String,
}

impl CorePaths {
    /// The XDG directories named `anibeam`, which on Linux are
    /// ~/.local/share/anibeam, ~/.config/anibeam, ~/.cache/anibeam and
    /// ~/.local/state/anibeam.
    pub fn xdg() -> Result<CorePaths, CoreError> {
        let dirs = ProjectDirs::from("", "", "anibeam").ok_or_else(|| CoreError::Io {
            path: None,
            message: "no home directory".to_string(),
        })?;
        let state = dirs
            .state_dir()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| dirs.data_local_dir().join("state"));
        Ok(CorePaths {
            data_dir: dirs.data_local_dir().to_string_lossy().into_owned(),
            config_dir: dirs.config_dir().to_string_lossy().into_owned(),
            cache_dir: dirs.cache_dir().to_string_lossy().into_owned(),
            state_dir: state.to_string_lossy().into_owned(),
        })
    }

    /// Every directory under one root: data/, config/, cache/, state/. Tests
    /// and the CLI's --root flag use it.
    pub fn under(root: &Path) -> CorePaths {
        CorePaths {
            data_dir: root.join("data").to_string_lossy().into_owned(),
            config_dir: root.join("config").to_string_lossy().into_owned(),
            cache_dir: root.join("cache").to_string_lossy().into_owned(),
            state_dir: root.join("state").to_string_lossy().into_owned(),
        }
    }

    pub fn db_path(&self) -> PathBuf {
        Path::new(&self.data_dir).join("anibeam.db")
    }

    pub fn secrets_path(&self) -> PathBuf {
        Path::new(&self.data_dir).join("secrets.json")
    }

    pub fn images_dir(&self) -> PathBuf {
        Path::new(&self.cache_dir).join("images")
    }
}

/// Path containment: a path is under a root when it is the root or sits
/// below it. String prefixes are not enough, or `/lib2` would count as
/// inside `/lib`. The rule the scan reconciles with and the import
/// attaches a series with.
pub(crate) fn under(path: &str, root: &str) -> bool {
    path == root || path.starts_with(&format!("{}/", root.trim_end_matches('/')))
}

/// A trailing separator is not part of a path's identity, so it never
/// reaches a column: `/lib/` and `/lib` are one path, not two. A bare root
/// stays itself. Every path a source or an import writes goes through
/// this, so the two always agree about what one path is.
pub(crate) fn normalise(path: &str) -> String {
    let trimmed = path.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn under_is_a_path_prefix_not_a_string_prefix() {
        assert!(under("/lib/Show", "/lib"));
        assert!(under("/lib", "/lib"));
        assert!(!under("/lib2/Show", "/lib"));
        assert!(under("/lib/Show", "/lib/"));
    }

    #[test]
    fn a_trailing_separator_is_not_part_of_a_path() {
        assert_eq!(normalise("/lib/"), "/lib");
        assert_eq!(normalise("/lib"), "/lib");
        assert_eq!(normalise("/"), "/");
        assert_eq!(normalise("  /lib/anime/  "), "/lib/anime");
    }

    #[test]
    fn xdg_paths_end_in_anibeam() {
        let p = CorePaths::xdg().unwrap();
        for dir in [&p.data_dir, &p.config_dir, &p.cache_dir, &p.state_dir] {
            assert!(dir.ends_with("/anibeam"), "{dir}");
        }
        assert!(p.db_path().ends_with("anibeam/anibeam.db"));
        assert!(p.secrets_path().ends_with("anibeam/secrets.json"));
        assert!(p.images_dir().ends_with("anibeam/images"));
    }

    #[test]
    fn paths_for_a_directory_put_everything_under_it() {
        let dir = tempfile::tempdir().unwrap();
        let p = CorePaths::under(dir.path());
        assert_eq!(p.db_path(), dir.path().join("data").join("anibeam.db"));
        assert_eq!(p.images_dir(), dir.path().join("cache").join("images"));
    }
}
