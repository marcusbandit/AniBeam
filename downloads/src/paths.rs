//! Where the core's own engine saves what it adds, and the folder names it
//! saves under, as anirss decides both.
//!
//! Each of the three settings is taken from the first place that has it: the
//! core's own blob (`store::settings::DOWNLOAD_PATHS`, its save folder set by
//! `SetSaveFolder` and only for a client that keeps its files here), then, for
//! Anibeam's own qBittorrent, the first library folder (#1005), then anirss's
//! `config.toml` when there is one (read, never written), then qBittorrent's
//! own default save folder, the films' folder being the same, and the tag
//! `Anime`, as anirss's defaults tag.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use anibeam_contract::*;
use anibeam_foundation::services::Services;
use anibeam_foundation::store::settings as kv;

use super::home::AnirssHome;
use super::qbt::QbtClient;
use super::settings;

/// The settings blob's key.
pub const KEY: &str = kv::DOWNLOAD_PATHS;

/// anirss's `[qbittorrent].tags` default.
const DEFAULT_TAG: &str = "Anime";

/// The films' folder under a library or picked save folder, the name the library scan reads
/// films from (`library::walk::is_movies_folder_name`).
const MOVIES: &str = "Movies";

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
/// with `~` expanded against `home`, as the anirss Anibeam runs expands them.
/// Nothing when the file is missing or does not read.
pub fn from_anirss_config(path: &Path, home: &AnirssHome) -> Chosen {
    let home = home.dir();
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
            .map(|p| expand_under(p, home.as_deref()))
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

/// The settings as far as the core can tell without asking the torrent client.
#[derive(Clone, Debug, PartialEq)]
pub struct Planned {
    /// `None` when only the client's own default folder is left.
    pub save_base: Option<String>,
    pub from: SaveFolderFrom,
    pub movie_path: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// The first library folder still on disk, in the order the folders were added: where
/// Anibeam's own qBittorrent saves when nothing else is picked, so a new show lands where
/// the library finds it. The disk decides, not the source's `available` mark, which only a
/// scan updates: a drive plugged in since then counts at once (#1193).
pub fn library_folder(services: &Services) -> Result<Option<String>, CoreError> {
    Ok(library_folders(services)?
        .into_iter()
        .find(|p| Path::new(p).is_dir()))
}

/// The first library folder that is not on disk now: its drive or share is not connected.
fn disconnected_library(services: &Services) -> Result<Option<String>, CoreError> {
    Ok(library_folders(services)?
        .into_iter()
        .find(|p| !Path::new(p).is_dir()))
}

/// Every library folder, in the order they were added.
fn library_folders(services: &Services) -> Result<Vec<String>, CoreError> {
    services.store.read(|c| {
        let mut stmt = c.prepare("SELECT path FROM sources ORDER BY added_at, id")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    })
}

/// The blob's folder, then the first library folder for Anibeam's own qBittorrent, then
/// anirss's config. A folder picked here is a path on this machine, so it counts only for a
/// client that keeps its files here (`here`, from `settings::files_here`) and only while it
/// is on disk: an unmounted drive falls back rather than being recreated under its mount
/// point. The user's own client never gets the library folder: it keeps the folders it had.
pub fn planned(
    services: &Services,
    s: &DownloadSettings,
    here: bool,
) -> Result<Planned, CoreError> {
    let stored: Chosen = services
        .store
        .read(|c| kv::get(c, KEY))?
        .unwrap_or_default();
    // A sandbox's `~` is its own home, the one the anirss it runs sees (#637)
    let home = AnirssHome::of(&services.paths);
    let dir = home.dir();
    let stored = Chosen {
        save_base: stored.save_base.map(|p| expand_under(&p, dir.as_deref())),
        movie_path: stored.movie_path.map(|p| expand_under(&p, dir.as_deref())),
        tags: stored.tags,
    };
    let mut config = settings::anirss_config_path(services)
        .map(|p| from_anirss_config(&p, &home))
        .unwrap_or_default();
    // anirss's folders for a client that keeps its files here name this machine's disk, so
    // a path from another machine's config (#1193) is not one: it counts only while it is
    // a folder here. A client on its own machine keeps the folders it was given.
    if here {
        let on_disk = |p: Option<String>| p.filter(|p| Path::new(p).is_dir());
        config.save_base = on_disk(config.save_base);
        config.movie_path = on_disk(config.movie_path);
    }
    let picked = stored
        .save_base
        .clone()
        .filter(|p| !p.trim().is_empty() && here && Path::new(p).is_dir());
    let library = match s.torrent_mode {
        TorrentMode::Managed if picked.is_none() => library_folder(services)?,
        _ => None,
    };
    // A library whose drive is not connected is where this add belongs: it waits for the
    // drive rather than saving anywhere else, anirss's folder included (#1193)
    if here
        && s.torrent_mode == TorrentMode::Managed
        && picked.is_none()
        && library.is_none()
        && let Some(folder) = disconnected_library(services)?
    {
        return Err(CoreError::invalid(
            "save_path",
            anibeam_contract::i18n::tr(
                "core-downloads-library-not-connected",
                &[("folder", folder.into())],
            ),
        ));
    }
    let (save_base, from) = match (picked, library) {
        (Some(base), _) => (Some(base), SaveFolderFrom::Chosen),
        (None, Some(base)) => (Some(base), SaveFolderFrom::Library),
        (None, None) => match config.save_base.clone() {
            Some(base) => (Some(base), SaveFolderFrom::Helper),
            None => (None, SaveFolderFrom::Client),
        },
    };
    // anirss's films' folder goes with its save folder. A folder picked here, or the
    // library, takes the films into a Movies folder under it, the folder the library reads
    // films from, so they never land loose beside the shows. Its tags stand either way.
    let theirs = matches!(from, SaveFolderFrom::Helper | SaveFolderFrom::Client);
    let films = match (&save_base, theirs) {
        (Some(base), false) => Some(join(base, MOVIES)),
        _ => config.movie_path.clone(),
    };
    let rest = Chosen {
        save_base: None,
        ..stored
    }
    .or(Chosen {
        save_base: None,
        movie_path: films,
        tags: config.tags.clone(),
    });
    Ok(Planned {
        save_base,
        from,
        movie_path: rest.movie_path,
        tags: rest.tags,
    })
}

/// Saves the folder new shows go under; `None` or an empty one goes back to the default.
/// The rest of the blob stays as it is.
pub fn set_save_folder(services: &Services, folder: Option<&str>) -> Result<(), CoreError> {
    let folder = folder.map(str::trim).filter(|f| !f.is_empty());
    if let Some(f) = folder
        && !Path::new(f).is_absolute()
    {
        return Err(CoreError::invalid(
            "folder",
            anibeam_contract::i18n::tr(
                "core-downloads-save-folder-not-absolute",
                &[("folder", f.into())],
            ),
        ));
    }
    if let Some(f) = folder {
        let at = real(f);
        let home = std::env::home_dir().map(|h| real(&h.to_string_lossy()));
        // The folder's own subfolders are what removing a subscription with its files
        // trashes, so the root and the home folder are never one
        if !Path::new(f).is_dir() || at.parent().is_none() || home.as_ref() == Some(&at) {
            return Err(CoreError::invalid(
                "folder",
                anibeam_contract::i18n::tr(
                    "core-downloads-save-folder-unusable",
                    &[("folder", f.into())],
                ),
            ));
        }
    }
    let folder = folder.map(str::to_string);
    services.store.write(move |c| {
        let mut blob: Chosen = kv::get(c, KEY)?.unwrap_or_default();
        blob.save_base = folder;
        kv::set(c, KEY, &blob)
    })
}

/// The three settings as they stand: `planned`, then qBittorrent's default folder, asked
/// only when nothing else names one.
pub async fn resolve(services: &Services, qbt: &QbtClient) -> Result<SavePaths, CoreError> {
    let s = settings::read(services)?;
    let chosen = planned(services, &s, settings::files_here(&s))?;
    let save_base = match chosen.save_base {
        Some(base) => base,
        None => {
            let base = qbt.default_save_path().await?;
            if base.is_empty() {
                return Err(CoreError::invalid(
                    "save_path",
                    anibeam_contract::i18n::tr("core-downloads-no-default-folder", &[]),
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

/// The folders removing a subscription with its files may trash under: where `saving`
/// saves now, and every folder Anibeam has saved under before for this client, so a
/// subscription made before the save folder moved (the library's turn in #1005, a folder
/// picked or reset) can still be removed with its files. A library show among them is
/// still guarded on its own (`confined`).
pub fn remove_roots(services: &Services, saving: &SavePaths) -> Result<Vec<String>, CoreError> {
    let s = settings::read(services)?;
    let home = AnirssHome::of(&services.paths);
    let config = settings::anirss_config_path(services)
        .map(|p| from_anirss_config(&p, &home))
        .unwrap_or_default();
    let mut roots = vec![saving.save_base.clone(), saving.movie_path.clone()];
    roots.extend(config.save_base);
    roots.extend(config.movie_path);
    if s.torrent_mode == TorrentMode::Managed {
        // The managed client's default before #1005, and the library folders since
        roots.extend(home.save_dir().map(|d| d.to_string_lossy().into_owned()));
        let folders: Vec<String> = services.store.read(|c| {
            let mut stmt = c.prepare("SELECT path FROM sources")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        })?;
        roots.extend(folders);
    }
    roots.retain(|r| !r.trim().is_empty());
    roots.dedup();
    Ok(roots)
}

/// A download or subscription name becomes a folder, so it is one plain
/// path component (anirss `validate_name`).
pub(crate) fn validate_name(name: &str) -> Result<String, CoreError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::invalid(
            "name",
            anibeam_contract::i18n::tr("core-downloads-name-empty", &[]),
        ));
    }
    if name.contains(['/', '\\']) {
        return Err(CoreError::invalid(
            "name",
            anibeam_contract::i18n::tr("core-downloads-name-slash", &[]),
        ));
    }
    if name == "." || name == ".." || name.chars().any(|c| (c as u32) < 32 || c as u32 == 127) {
        return Err(CoreError::invalid(
            "name",
            anibeam_contract::i18n::tr(
                "core-downloads-name-not-folder",
                &[("name", format!("{name:?}").into())],
            ),
        ));
    }
    Ok(name.to_string())
}

/// The folder for releases: the name given, else the show's from the first
/// title (a season pack's without its season), else `fallback`, as anirss names a download or a subscription.
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
                .map(|i| anibeam_releases::titles::folder_name(&i.title))
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

/// `~` and `~/...` under the home folder; anything else as it is. For the
/// paths the torrent client gives back, which are absolute already.
fn expand_home(path: &str) -> String {
    expand_under(path, std::env::home_dir().as_deref())
}

/// `~` and `~/...` under `home`; anything else, or anything with no home, as it is.
fn expand_under(path: &str, home: Option<&Path>) -> String {
    let path = path.trim();
    let rest = match path.strip_prefix('~') {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => rest,
        _ => return path.to_string(),
    };
    match home {
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

    use crate::testing::Teams;

    /// A sandbox core with a library source (`lib`, marked `available`) and an
    /// anirss config whose save folder is `save_base`.
    fn rig(available: bool, lib: Option<&str>, save_base: &str) -> (tempfile::TempDir, Teams) {
        let dir = tempfile::tempdir().unwrap();
        let teams = Teams::open(dir.path(), anibeam_net::FakeHttp::new());
        let cfg = AnirssHome::of(&teams.services.paths).config_path().unwrap();
        std::fs::create_dir_all(cfg.parent().unwrap()).unwrap();
        std::fs::write(&cfg, format!("[downloads]\nsave_base = \"{save_base}\"\n")).unwrap();
        if let Some(lib) = lib {
            let lib = lib.to_string();
            teams
                .services
                .store
                .write(move |c| {
                    c.execute(
                        "INSERT INTO sources (path, available, added_at) VALUES (?1, ?2, 0)",
                        rusqlite::params![lib, available as i64],
                    )?;
                    Ok(())
                })
                .unwrap();
        }
        (dir, teams)
    }

    fn mode(teams: &Teams, m: TorrentMode) -> DownloadSettings {
        DownloadSettings {
            torrent_mode: m,
            ..settings::read(&teams.services).unwrap()
        }
    }

    #[test]
    fn an_unconnected_library_is_an_error_for_a_client_here_not_anirsss_folder() {
        let (_d, t) = rig(false, Some("/Volumes/Gone/Anime"), "/home/qbtuser/Anime/");
        let s = mode(&t, TorrentMode::Managed);
        let e = planned(&t.services, &s, true).unwrap_err();
        assert!(e.to_string().contains("/Volumes/Gone/Anime"), "{e}");
    }

    #[test]
    fn an_unconnected_library_wins_over_an_anirss_folder_that_is_here() {
        let folder = tempfile::tempdir().unwrap();
        let base = folder.path().to_string_lossy().into_owned();
        let (_d, t) = rig(false, Some("/Volumes/Gone/Anime"), &base);
        let s = mode(&t, TorrentMode::Managed);
        let e = planned(&t.services, &s, true).unwrap_err();
        assert!(e.to_string().contains("/Volumes/Gone/Anime"), "{e}");
    }

    #[test]
    fn a_library_folder_on_disk_counts_while_its_source_is_still_marked_unavailable() {
        let lib = tempfile::tempdir().unwrap();
        let lib = lib.path().to_string_lossy().into_owned();
        let (_d, t) = rig(false, Some(&lib), "/home/qbtuser/Anime/");
        let s = mode(&t, TorrentMode::Managed);
        let p = planned(&t.services, &s, true).unwrap();
        assert_eq!(p.save_base, Some(lib));
        assert_eq!(p.from, SaveFolderFrom::Library);
    }

    #[test]
    fn a_remote_client_keeps_anirsss_folder_whatever_the_library() {
        let (_d, t) = rig(false, Some("/Volumes/Gone/Anime"), "/home/qbtuser/Anime/");
        let s = mode(&t, TorrentMode::Existing);
        let p = planned(&t.services, &s, false).unwrap();
        assert_eq!(p.save_base.as_deref(), Some("/home/qbtuser/Anime/"));
        assert_eq!(p.from, SaveFolderFrom::Helper);
    }

    #[test]
    fn anirsss_folder_that_exists_here_still_counts_with_no_library() {
        let folder = tempfile::tempdir().unwrap();
        let base = folder.path().to_string_lossy().into_owned();
        let (_d, t) = rig(false, None, &base);
        let s = mode(&t, TorrentMode::Managed);
        let p = planned(&t.services, &s, true).unwrap();
        assert_eq!(p.save_base, Some(base));
        assert_eq!(p.from, SaveFolderFrom::Helper);
    }

    #[test]
    fn a_foreign_anirss_folder_with_no_library_leaves_the_clients_own() {
        let (_d, t) = rig(false, None, "/home/qbtuser/Anime/");
        let s = mode(&t, TorrentMode::Managed);
        let p = planned(&t.services, &s, true).unwrap();
        assert_eq!(p.save_base, None);
        assert_eq!(p.from, SaveFolderFrom::Client);
    }

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
    fn a_batch_lands_in_its_show_folder() {
        let batch: Release = serde_json::from_value(serde_json::json!({
            "title": "[Demo] Cowboy Bebop - 02-03 [1080p]",
            "link": "magnet:?xt=urn:btih:0000000000000000000000000000000000000001",
            "source": "feed",
        }))
        .unwrap();
        assert_eq!(
            name_for(None, &[batch], "Downloads").unwrap(),
            "Cowboy Bebop"
        );
    }

    #[test]
    fn a_season_pack_lands_in_its_show_folder() {
        let pack: Release = serde_json::from_value(serde_json::json!({
            "title": "[Grp] Some Show S01 [1080p]",
            "link": "magnet:?xt=urn:btih:0000000000000000000000000000000000000002",
            "source": "feed",
        }))
        .unwrap();
        assert_eq!(name_for(None, &[pack], "Downloads").unwrap(), "Some Show");
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
             [downloads]\nsave_base = \"/srv/TV\"\nmovie_path = \"~/Films\"\n",
        )
        .unwrap();
        // A sandbox's `~` is the sandbox's home, never the person's own
        let home = AnirssHome::Sandbox(dir.path().join("home"));
        assert_eq!(
            from_anirss_config(&p, &home),
            Chosen {
                save_base: Some("/srv/TV".into()),
                movie_path: Some(dir.path().join("home/Films").to_string_lossy().into_owned()),
                tags: Some(vec!["TV".into(), "New".into()]),
            }
        );
        assert_eq!(
            from_anirss_config(&dir.path().join("missing.toml"), &home),
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
