//! Moving a show's files to the trash. "Delete files" on the downloads page
//! never deletes: it moves what the torrents downloaded and the library
//! series' folder to the freedesktop trash, where they can be put back, on
//! the same drive as the files (`$topdir/.Trash-$uid`), so a season is a
//! rename, not a copy.
//!
//! Every target is checked before anything goes. A path that is not
//! absolute, holds a `..`, does not exist, is the filesystem root, is the
//! home folder or holds it, or is a library source or holds one, is
//! refused, and with it the whole removal: one bad target means the
//! request was not what it looked like.

use std::path::{Component, Path, PathBuf};

use crate::contract::*;

/// One path offered for the trash, and where it came from.
#[derive(Clone, Debug, PartialEq)]
pub enum Candidate {
    /// A torrent's content path. `save_path` is the folder the client saved
    /// it under: content that is that folder itself (a torrent laid out
    /// without its own subfolder) would take everything else saved there,
    /// so it is refused unless it is also the series' own folder.
    Torrent { content: String, save_path: String },
    /// The library series' own folder (a film: its file), which must sit
    /// strictly inside a library source.
    Series { path: String },
}

fn refuse(path: &str, why: &str) -> CoreError {
    CoreError::invalid("files", format!("{path} {why}, so nothing was removed"))
}

fn clean(path: &str) -> PathBuf {
    PathBuf::from(path.trim().trim_end_matches(std::path::is_separator))
}

fn has_parent_step(p: &Path) -> bool {
    p.components().any(|c| matches!(c, Component::ParentDir))
}

/// The paths to move, checked and with any path inside another dropped. A
/// candidate that is simply not on disk any more (a torrent the client
/// remembers under a mount that is gone) is skipped, not refused.
pub fn targets(
    candidates: &[Candidate],
    sources: &[String],
    home: Option<&Path>,
) -> Result<Vec<PathBuf>, CoreError> {
    let sources: Vec<PathBuf> = sources.iter().map(|s| clean(s)).collect();
    let series_paths: Vec<PathBuf> = candidates
        .iter()
        .filter_map(|c| match c {
            Candidate::Series { path } => Some(clean(path)),
            _ => None,
        })
        .collect();
    let mut out: Vec<PathBuf> = Vec::new();
    for c in candidates {
        let (raw, p) = match c {
            Candidate::Torrent { content, .. } | Candidate::Series { path: content } => {
                (content.as_str(), clean(content))
            }
        };
        if raw.trim().is_empty() {
            continue;
        }
        if !p.is_absolute() || has_parent_step(&p) {
            return Err(refuse(raw, "is not a plain absolute path"));
        }
        if p.parent().is_none() {
            return Err(refuse(raw, "is the filesystem root"));
        }
        if std::fs::symlink_metadata(&p).is_err() {
            continue;
        }
        if let Some(home) = home
            && home.starts_with(&p)
        {
            return Err(refuse(raw, "is the home folder or holds it"));
        }
        if sources.iter().any(|s| s.starts_with(&p)) {
            return Err(refuse(raw, "is a library folder or holds one"));
        }
        match c {
            Candidate::Series { .. } => {
                if !sources.iter().any(|s| p.starts_with(s)) {
                    return Err(refuse(raw, "is not inside a library folder"));
                }
            }
            Candidate::Torrent { save_path, .. } => {
                if clean(save_path) == p && !series_paths.contains(&p) {
                    return Err(refuse(
                        raw,
                        "is the folder the torrent was saved in, not what it downloaded",
                    ));
                }
            }
        }
        out.push(p);
    }
    out.sort();
    out.dedup();
    // Drop a path inside another: the outer one takes it along
    let mut kept: Vec<PathBuf> = Vec::new();
    for p in out {
        if !kept.iter().any(|k| p.starts_with(k)) {
            kept.push(p);
        }
    }
    Ok(kept)
}

/// Where `move_to_trash` sends paths, as a message says it.
#[cfg(not(target_os = "android"))]
pub const WHERE: &str = "to the trash";
#[cfg(target_os = "android")]
pub const WHERE: &str = "deleted";

/// Moves the paths to the trash, all or as many as it can before the
/// first failure, which it reports. Android has no trash, so a phone
/// deletes them.
pub fn move_to_trash(paths: &[PathBuf]) -> Result<(), CoreError> {
    if paths.is_empty() {
        return Ok(());
    }
    #[cfg(target_os = "android")]
    return delete_outright(paths);
    #[cfg(not(target_os = "android"))]
    trash::delete_all(paths).map_err(|e| CoreError::Service {
        service: "trash".to_string(),
        code: "failed".to_string(),
        message: e.to_string(),
    })
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn delete_outright(paths: &[PathBuf]) -> Result<(), CoreError> {
    for p in paths {
        let gone = if p.is_dir() {
            std::fs::remove_dir_all(p)
        } else {
            std::fs::remove_file(p)
        };
        match gone {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(CoreError::Service {
                    service: "trash".to_string(),
                    code: "failed".to_string(),
                    message: format!("{}: {e}", p.display()),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn deleting_outright_takes_files_and_folders_and_skips_what_is_gone() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.mkv");
        let folder = dir.path().join("Show");
        std::fs::write(&file, b"x").unwrap();
        std::fs::create_dir_all(folder.join("S1")).unwrap();
        std::fs::write(folder.join("S1/b.mkv"), b"x").unwrap();
        let gone = dir.path().join("never-there.mkv");
        delete_outright(&[file.clone(), folder.clone(), gone]).unwrap();
        assert!(!file.exists() && !folder.exists());
    }
    use super::*;

    fn tree() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for p in ["lib/Frieren", "lib/Frieren/sub", "dl/Frieren", "home/me"] {
            std::fs::create_dir_all(d.path().join(p)).unwrap();
        }
        std::fs::write(d.path().join("dl/one.mkv"), b"x").unwrap();
        d
    }

    fn s(d: &tempfile::TempDir, p: &str) -> String {
        d.path().join(p).to_string_lossy().into_owned()
    }

    #[test]
    fn the_series_folder_and_what_the_torrents_wrote_go_inner_paths_folded() {
        let d = tree();
        let got = targets(
            &[
                Candidate::Series {
                    path: s(&d, "lib/Frieren"),
                },
                Candidate::Torrent {
                    content: s(&d, "lib/Frieren/sub"),
                    save_path: s(&d, "lib/Frieren"),
                },
                Candidate::Torrent {
                    content: s(&d, "dl/one.mkv"),
                    save_path: s(&d, "dl"),
                },
                Candidate::Torrent {
                    content: "/srv/gone/Frieren/01.mkv".into(),
                    save_path: "/srv/gone/Frieren".into(),
                },
            ],
            &[s(&d, "lib")],
            Some(&d.path().join("home/me")),
        )
        .unwrap();
        assert_eq!(
            got,
            vec![d.path().join("dl/one.mkv"), d.path().join("lib/Frieren")]
        );
    }

    #[test]
    fn a_library_root_home_or_save_folder_refuses_the_whole_request() {
        let d = tree();
        let sources = [s(&d, "lib")];
        let home = d.path().join("home/me");
        let refused = |c: Candidate| targets(&[c], &sources, Some(&home)).is_err();
        assert!(
            refused(Candidate::Series { path: s(&d, "lib") }),
            "the source itself"
        );
        assert!(
            refused(Candidate::Torrent {
                content: d.path().to_string_lossy().into_owned(),
                save_path: "/x".into()
            }),
            "holds the source"
        );
        assert!(
            refused(Candidate::Torrent {
                content: s(&d, "home"),
                save_path: "/x".into()
            }),
            "holds home"
        );
        assert!(
            refused(Candidate::Torrent {
                content: s(&d, "dl"),
                save_path: s(&d, "dl")
            }),
            "the save folder itself"
        );
        assert!(
            refused(Candidate::Series {
                path: s(&d, "dl/Frieren")
            }),
            "a series outside every source"
        );
        assert!(
            refused(Candidate::Series {
                path: format!("{}/../dl", s(&d, "lib"))
            }),
            "a .. step"
        );
        assert!(refused(Candidate::Torrent {
            content: "relative/x".into(),
            save_path: "relative".into()
        }));
        assert!(refused(Candidate::Torrent {
            content: "/".into(),
            save_path: "/".into()
        }));
    }

    #[test]
    fn a_save_folder_that_is_also_the_series_folder_may_go() {
        let d = tree();
        let got = targets(
            &[
                Candidate::Series {
                    path: s(&d, "lib/Frieren"),
                },
                Candidate::Torrent {
                    content: s(&d, "lib/Frieren"),
                    save_path: s(&d, "lib/Frieren"),
                },
            ],
            &[s(&d, "lib")],
            None,
        )
        .unwrap();
        assert_eq!(got, vec![d.path().join("lib/Frieren")]);
    }

    /// Run by hand against a real drive: `ANIBEAM_TRASH_PROBE=/mnt/x/probe
    /// cargo test -p anibeam-core probe_a_real_trash -- --ignored`.
    #[test]
    #[ignore]
    fn probe_a_real_trash() {
        let probe = std::env::var("ANIBEAM_TRASH_PROBE").expect("ANIBEAM_TRASH_PROBE");
        let p = PathBuf::from(&probe);
        std::fs::create_dir_all(p.join("inner")).unwrap();
        std::fs::write(p.join("inner/ep.mkv"), b"x").unwrap();
        move_to_trash(std::slice::from_ref(&p)).unwrap();
        assert!(!p.exists(), "moved away");
    }
}
