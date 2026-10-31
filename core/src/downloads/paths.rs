//! Where the core's own engine saves what it adds, and the folder names it
//! saves under, as anirss decides both.
//!
//! Each of the three settings is taken from the first place that has it: the
//! core's own blob (`store::settings::DOWNLOAD_PATHS`, unset until something
//! sets it), then anirss's `config.toml` when there is one (read, never
//! written), then qBittorrent's own default save folder, the films' folder
//! being the same, and the tag `Anime`, as anirss's defaults tag.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::contract::*;
use crate::core::Core;
use crate::store::settings as kv;

use super::qbt::QbtClient;
use super::settings;

/// The settings blob's key.
pub const KEY: &str = kv::DOWNLOAD_PATHS;

/// anirss's `[qbittorrent].tags` default.
const DEFAULT_TAG: &str = "Anime";

/// Where one add or subscription saves, and how it is tagged.
#[derive(Clone, Debug, PartialEq)]
pub struct SavePaths {
    /// Each show's own folder goes under this.
    pub save_base: String,
    /// Films go straight into this, with no folder of their own.
    pub movie_path: String,
    pub tags: Vec<String>,
}

/// What one place says, each field `None` where it says nothing. An empty
/// tag list is a choice of no tags, not silence.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Chosen {
    #[serde(default)]
    pub save_base: Option<String>,
    #[serde(default)]
    pub movie_path: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

impl Chosen {
    /// Each field from `self`, else from `other`.
    fn or(self, other: Chosen) -> Chosen {
        let path = |p: Option<String>| p.filter(|p| !p.trim().is_empty());
        Chosen {
            save_base: path(self.save_base).or(path(other.save_base)),
            movie_path: path(self.movie_path).or(path(other.movie_path)),
            tags: self.tags.or(other.tags),
        }
    }
}

/// anirss's `[downloads] save_base`, `movie_path` and `[qbittorrent] tags`,
/// with `~` expanded as anirss expands them. Nothing when the file is
/// missing or does not read.
pub fn from_anirss_config(path: &Path) -> Chosen {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Chosen::default();
    };
    let Ok(doc) = toml::from_str::<toml::Table>(&text) else {
        return Chosen::default();
    };
    let downloads = doc.get("downloads");
    let path_of = |key: &str| {
        downloads
            .and_then(|d| d.get(key))
            .and_then(|v| v.as_str())
            .map(expand_home)
    };
    let tags = doc
        .get("qbittorrent")
        .and_then(|q| q.get("tags"))
        .and_then(|t| t.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|t| t.as_str().map(str::to_string))
                .collect()
        });
    Chosen {
        save_base: path_of("save_base"),
        movie_path: path_of("movie_path"),
        tags,
    }
}

/// The three settings as they stand: the blob, anirss's config, then
/// qBittorrent's default folder, asked only when nothing else names one.
pub async fn resolve(core: &Arc<Core>, qbt: &QbtClient) -> Result<SavePaths, CoreError> {
    let stored: Chosen = core.store.read(|c| kv::get(c, KEY))?.unwrap_or_default();
    let stored = Chosen {
        save_base: stored.save_base.map(|p| expand_home(&p)),
        movie_path: stored.movie_path.map(|p| expand_home(&p)),
        tags: stored.tags,
    };
    let config = settings::anirss_config_path()
        .map(|p| from_anirss_config(&p))
        .unwrap_or_default();
    let chosen = stored.or(config);
    let save_base = match chosen.save_base {
        Some(base) => base,
        None => {
            let base = qbt.default_save_path().await?;
            if base.is_empty() {
                return Err(CoreError::invalid(
                    "save_path",
                    "qBittorrent names no default save folder; set one in qBittorrent",
                ));
            }
            base
        }
    };
    Ok(SavePaths {
        movie_path: chosen.movie_path.unwrap_or_else(|| save_base.clone()),
        tags: chosen.tags.unwrap_or_else(|| vec![DEFAULT_TAG.to_string()]),
        save_base,
    })
}

/// A download or subscription name becomes a folder, so it is one plain
/// path component (anirss `validate_name`).
pub(crate) fn validate_name(name: &str) -> Result<String, CoreError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::invalid("name", "the name cannot be empty"));
    }
    if name.contains(['/', '\\']) {
        return Err(CoreError::invalid(
            "name",
            "the name must not contain / or \\",
        ));
    }
    if name == "." || name == ".." || name.chars().any(|c| (c as u32) < 32 || c as u32 == 127) {
        return Err(CoreError::invalid(
            "name",
            format!("{name:?} is not a folder name"),
        ));
    }
    Ok(name.to_string())
}

/// The folder for releases: the name given, else the show's from the first
/// title, else `fallback`, as anirss names a download or a subscription.
pub(crate) fn name_for(
    name: Option<&str>,
    items: &[Release],
    fallback: &str,
) -> Result<String, CoreError> {
    match name.map(str::trim).filter(|n| !n.is_empty()) {
        Some(n) => validate_name(n),
        None => {
            let shown = items
                .first()
                .map(|i| crate::releases::titles::show_name(&i.title))
                .unwrap_or_default();
            validate_name(if shown.is_empty() { fallback } else { &shown })
        }
    }
}

/// `base/name`, as Python's `posixpath.join` makes it: a forward slash on
/// every host, since the path is qBittorrent's and Windows reads it too.
pub fn join(base: &str, name: &str) -> String {
    if name.starts_with('/') || base.is_empty() {
        name.to_string()
    } else if base.ends_with('/') {
        format!("{base}{name}")
    } else {
        format!("{base}/{name}")
    }
}

/// A path as anirss compares them: `~` expanded, `.`, `..` and repeated
/// slashes folded away (Python's `normpath`), and no trailing slash.
pub fn norm(path: &str) -> String {
    let path = expand_home(path);
    if path.is_empty() {
        return ".".to_string();
    }
    let rooted = path.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|p| *p != "..") => {
                parts.pop();
            }
            ".." if rooted => {}
            p => parts.push(p),
        }
    }
    let joined = parts.join("/");
    let out = if rooted {
        format!("/{joined}")
    } else if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    };
    out.trim_end_matches('/').to_string()
}

/// `~` and `~/...` under the home folder; anything else as it is.
fn expand_home(path: &str) -> String {
    let path = path.trim();
    let rest = match path.strip_prefix('~') {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => rest,
        _ => return path.to_string(),
    };
    match std::env::home_dir() {
        Some(home) => format!("{}{rest}", home.to_string_lossy()),
        None => path.to_string(),
    }
}

/// Whether `path` lies strictly inside one of `roots`, links resolved where
/// they exist (anirss `_is_inside`).
pub fn inside(path: &str, roots: &[&str]) -> bool {
    let at = real(path);
    roots.iter().filter(|r| !r.trim().is_empty()).any(|root| {
        let root = real(root);
        at != root && at.starts_with(&root)
    })
}

/// The path with links resolved as far as it exists, the rest kept as it
/// reads (Python's `realpath`).
pub fn real(path: &str) -> PathBuf {
    let normed = PathBuf::from(norm(path));
    let mut rest = Vec::new();
    let mut at = normed.as_path();
    loop {
        if let Ok(found) = std::fs::canonicalize(at) {
            return rest.iter().rev().fold(found, |p, part| p.join(part));
        }
        match (at.parent(), at.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name.to_os_string());
                at = parent;
            }
            _ => return normed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_fold_as_python_folds_them() {
        assert_eq!(norm("/a//b/./c/"), "/a/b/c");
        assert_eq!(norm("/a/b/../c"), "/a/c");
        assert_eq!(norm("/.."), "");
        assert_eq!(norm("a/../.."), "..");
        assert_eq!(norm(""), ".");
        assert_eq!(join("/media/TV/", "Show"), "/media/TV/Show");
        assert_eq!(join("/media/TV", "Show"), "/media/TV/Show");
    }

    #[test]
    fn a_name_is_one_folder() {
        assert_eq!(validate_name("  Show ").unwrap(), "Show");
        for bad in ["", "a/b", "a\\b", "..", "a\nb"] {
            assert!(validate_name(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn only_a_path_strictly_inside_a_root_counts() {
        assert!(inside("/srv/media/Show", &["/srv/media"]));
        assert!(inside("/srv/media/a/../Show", &["/srv/media/"]));
        assert!(!inside("/srv/media", &["/srv/media"]));
        assert!(!inside("/srv/mediaX/Show", &["/srv/media"]));
        assert!(!inside("/srv/media/../etc", &["/srv/media"]));
        assert!(!inside("/srv/media/Show", &[""]));
    }

    #[test]
    fn anirss_config_gives_the_folders_and_tags() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.toml");
        std::fs::write(
            &p,
            "[qbittorrent]\nurl = \"http://localhost:9090\"\ntags = [\"TV\", \"New\"]\n\n\
             [downloads]\nsave_base = \"/srv/TV\"\nmovie_path = \"/srv/Films\"\n",
        )
        .unwrap();
        assert_eq!(
            from_anirss_config(&p),
            Chosen {
                save_base: Some("/srv/TV".into()),
                movie_path: Some("/srv/Films".into()),
                tags: Some(vec!["TV".into(), "New".into()]),
            }
        );
        assert_eq!(
            from_anirss_config(&dir.path().join("missing.toml")),
            Chosen::default()
        );
    }

    #[test]
    fn each_setting_comes_from_the_first_place_that_has_it() {
        let stored = Chosen {
            save_base: Some(" ".into()),
            movie_path: Some("/mine/Films".into()),
            tags: Some(Vec::new()),
        };
        let config = Chosen {
            save_base: Some("/srv/TV".into()),
            movie_path: Some("/srv/Films".into()),
            tags: Some(vec!["TV".into()]),
        };
        assert_eq!(
            stored.or(config),
            Chosen {
                save_base: Some("/srv/TV".into()),
                movie_path: Some("/mine/Films".into()),
                tags: Some(Vec::new()),
            }
        );
    }
}
