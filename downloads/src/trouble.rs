//! Why a torrent the client stopped cannot go on, and trying it again
//! (#1190). qBittorrent says only `error` or `missingFiles`; the core looks
//! at the torrent's save folder to tell an unplugged drive from the rest,
//! for a client that keeps its files on this machine. A torrent seen with
//! its folder missing is tried again by itself once a later torrent read
//! finds the folder back, so plugging the drive in is all it takes.

use std::collections::HashSet;
use std::path::Path;

use anibeam_contract::*;

use super::qbt::QbtClient;

/// qBittorrent's states for a torrent it stopped on a fault.
pub fn erred(state: &str) -> bool {
    state == "error" || state == "missingFiles"
}

/// Why `t` cannot go on, None for a torrent that is fine. `here` is whether
/// the client keeps its files on this machine (`settings::files_here`): the
/// folder of one that does not cannot be looked at from here.
pub fn problem_of(t: &Torrent, here: bool) -> Option<TorrentProblem> {
    problem_with(t, here, &|p| Path::new(p).exists())
}

fn problem_with(t: &Torrent, here: bool, exists: &dyn Fn(&str) -> bool) -> Option<TorrentProblem> {
    if !erred(&t.state) {
        return None;
    }
    if here && let Some(folder) = missing_folder(t, exists) {
        if from_elsewhere(&folder, exists) {
            return Some(TorrentProblem::FolderElsewhere { folder });
        }
        let drive = drive_of(&folder);
        let drive = if drive.is_empty() || exists(&drive) {
            String::new()
        } else {
            drive
        };
        return Some(TorrentProblem::FolderMissing { folder, drive });
    }
    Some(if t.state == "missingFiles" {
        TorrentProblem::FilesMissing
    } else {
        TorrentProblem::ClientError
    })
}

/// Fills in `problem` on every torrent.
pub fn annotate(torrents: &mut [Torrent], here: bool) {
    for t in torrents.iter_mut() {
        t.problem = problem_of(t, here);
    }
}

/// The torrent's save folder when it is not there.
fn missing_folder(t: &Torrent, exists: &dyn Fn(&str) -> bool) -> Option<String> {
    let folder = t.save_path.trim();
    (!folder.is_empty() && !exists(folder)).then(|| folder.to_string())
}

/// Whether `folder` sits in the home folder of a user this machine does not
/// have (`/home/qbtuser/...` on a Mac): a path written on another computer,
/// such as a save folder read from another machine's anirss config.
fn from_elsewhere(folder: &str, exists: &dyn Fn(&str) -> bool) -> bool {
    let parts: Vec<&str> = folder.split('/').filter(|p| !p.is_empty()).collect();
    match parts.as_slice() {
        [home @ ("home" | "Users"), user, ..] => !exists(&format!("/{home}/{user}")),
        _ => false,
    }
}

/// The folder a removable drive or a mount shows up as, read off the path:
/// `/Volumes/<name>` on a Mac, `/media/<user>/<name>`, `/run/media/<user>/<name>`
/// and `/mnt/<name>` on Linux, `E:` on Windows. Empty for any other path.
pub fn drive_of(path: &str) -> String {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return path[..2].to_string();
    }
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let take = match parts.as_slice() {
        ["Volumes", _, ..] | ["mnt", _, ..] => 2,
        ["media", _, _, ..] => 3,
        ["run", "media", _, _, ..] => 4,
        _ => return String::new(),
    };
    format!("/{}", parts[..take].join("/"))
}

/// The torrents to try again by themselves: those seen before with their
/// folder missing (`waiting`) that are still stopped on a fault and whose
/// folder is back. `waiting` keeps the hashes whose folder is still
/// missing and drops the rest.
pub fn back(torrents: &[Torrent], waiting: &mut HashSet<String>) -> Vec<String> {
    let mut retry = Vec::new();
    let mut still = HashSet::new();
    for t in torrents {
        match &t.problem {
            Some(TorrentProblem::FolderMissing { .. } | TorrentProblem::FolderElsewhere { .. }) => {
                still.insert(t.hash.clone());
            }
            Some(_) if waiting.contains(&t.hash) => retry.push(t.hash.clone()),
            _ => {}
        }
    }
    *waiting = still;
    retry
}

/// `TorrentAction::Retry`: refused while a torrent's save folder is still
/// missing (its drive, when that is what is gone), else qBittorrent checks
/// what each has on disk and starts it, which clears `missingFiles` and
/// `error` alike.
pub async fn retry(client: &QbtClient, hashes: &[String], here: bool) -> Result<(), CoreError> {
    if here {
        let named = client.torrents_named(hashes).await?;
        let exists = |p: &str| Path::new(p).exists();
        if let Some(folder) = named.iter().find_map(|t| missing_folder(t, &exists)) {
            let drive = drive_of(&folder);
            let path = if !drive.is_empty() && !exists(&drive) {
                drive
            } else {
                folder
            };
            return Err(CoreError::Unavailable { path });
        }
    }
    client.recheck(hashes).await?;
    client.start(hashes).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn torrent(hash: &str, state: &str, save: &str) -> Torrent {
        let mut t = super::super::qbt::tests_torrent();
        t.hash = hash.into();
        t.state = state.into();
        t.save_path = save.into();
        t
    }

    #[test]
    fn a_drive_reads_off_the_path() {
        assert_eq!(drive_of("/Volumes/Anime/Shows/Frieren"), "/Volumes/Anime");
        assert_eq!(
            drive_of("/run/media/me/Anime/Frieren"),
            "/run/media/me/Anime"
        );
        assert_eq!(drive_of("/media/me/Anime"), "/media/me/Anime");
        assert_eq!(drive_of("/mnt/anime/x"), "/mnt/anime");
        assert_eq!(drive_of("E:\\Anime\\Frieren"), "E:");
        assert_eq!(drive_of("/home/me/Anime"), "");
        assert_eq!(drive_of("/Volumes"), "");
    }

    #[test]
    fn a_missing_folder_names_its_drive_when_the_drive_is_gone_too() {
        let gone = |_: &str| false;
        let t = torrent("a", "error", "/Volumes/Anime/Frieren");
        assert_eq!(
            problem_with(&t, true, &gone),
            Some(TorrentProblem::FolderMissing {
                folder: "/Volumes/Anime/Frieren".into(),
                drive: "/Volumes/Anime".into(),
            })
        );
        let drive_there = |p: &str| p == "/Volumes/Anime";
        assert_eq!(
            problem_with(&t, true, &drive_there),
            Some(TorrentProblem::FolderMissing {
                folder: "/Volumes/Anime/Frieren".into(),
                drive: String::new(),
            })
        );
    }

    #[test]
    fn a_folder_in_a_home_this_machine_lacks_is_from_another_computer() {
        let only_me = |p: &str| p == "/home" || p == "/Users" || p == "/Users/me";
        assert_eq!(
            problem_with(
                &torrent("a", "error", "/home/qbtuser/Anime/F"),
                true,
                &only_me
            ),
            Some(TorrentProblem::FolderElsewhere {
                folder: "/home/qbtuser/Anime/F".into(),
            })
        );
        assert!(matches!(
            problem_with(&torrent("a", "error", "/Users/me/Anime/F"), true, &only_me),
            Some(TorrentProblem::FolderMissing { .. })
        ));
    }

    #[test]
    fn a_fine_torrent_or_a_remote_client_reads_no_folder() {
        let gone = |_: &str| false;
        assert_eq!(
            problem_with(&torrent("a", "downloading", "/x"), true, &gone),
            None
        );
        assert_eq!(
            problem_with(&torrent("a", "missingFiles", "/x"), false, &gone),
            Some(TorrentProblem::FilesMissing)
        );
        let there = |_: &str| true;
        assert_eq!(
            problem_with(&torrent("a", "error", "/x"), true, &there),
            Some(TorrentProblem::ClientError)
        );
    }

    #[test]
    fn a_torrent_is_tried_again_once_its_folder_is_back() {
        let mut waiting = HashSet::new();
        let mut t = torrent("a", "error", "/Volumes/Anime/F");
        t.problem = Some(TorrentProblem::FolderMissing {
            folder: "/Volumes/Anime/F".into(),
            drive: "/Volumes/Anime".into(),
        });
        assert!(back(std::slice::from_ref(&t), &mut waiting).is_empty());
        assert!(waiting.contains("a"));
        t.problem = Some(TorrentProblem::ClientError);
        assert_eq!(back(std::slice::from_ref(&t), &mut waiting), vec!["a"]);
        assert!(waiting.is_empty());
        // Once only: a torrent that still errs is not tried again on the next read
        assert!(back(std::slice::from_ref(&t), &mut waiting).is_empty());
    }
}
