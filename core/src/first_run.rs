//! The desktop's first run: whether the setup walkthrough opens as the app starts, and the
//! folder it makes for someone who has no anime folder yet.
//!
//! The walkthrough is for a fresh install only. An install that ran before, by any sign the
//! shell can read, never lands in it on update: it is marked done the first time it starts
//! with this build, and Settings can still run it by hand.

use std::io;
use std::path::{Path, PathBuf};

/// What the shell knows as it starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    /// The walkthrough was finished or skipped before, kept in shell.conf.
    pub done: bool,
    /// The core logged a start before this one.
    pub used_before: bool,
    /// What's new kept a version, which only an install that ran keeps.
    pub notes_read: bool,
    /// At least one library folder is set.
    pub has_library: bool,
    /// AniList or MyAnimeList is connected.
    pub tracked: bool,
}

/// What the shell does with the walkthrough as it starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// Nothing: it was done before.
    Done,
    /// Keep it as done without showing it: this install was in use already.
    Settle,
    /// Show it.
    Open,
}

impl Start {
    pub fn as_str(self) -> &'static str {
        match self {
            Start::Done => "done",
            Start::Settle => "settle",
            Start::Open => "open",
        }
    }
}

pub fn start(facts: Facts) -> Start {
    if facts.done {
        Start::Done
    } else if facts.used_before || facts.notes_read || facts.has_library || facts.tracked {
        Start::Settle
    } else {
        Start::Open
    }
}

/// The folder's name inside the platform's videos folder.
pub const STARTER_NAME: &str = "Anime";

/// The folder offered to someone with none: `Anime` in the platform's videos folder.
pub fn starter_folder(videos: &Path) -> PathBuf {
    videos.join(STARTER_NAME)
}

/// Makes the starter folder, and the videos folder above it when that is missing too. A
/// folder already there is used as it is, files and all.
pub fn make_starter_folder(videos: &Path) -> io::Result<PathBuf> {
    if !videos.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            crate::i18n::tr("core-first-run-not-absolute", &[]),
        ));
    }
    let dir = starter_folder(videos);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_install_opens_it() {
        assert_eq!(start(Facts::default()), Start::Open);
    }

    #[test]
    fn once_done_it_stays_done() {
        let facts = Facts {
            done: true,
            ..Facts::default()
        };
        assert_eq!(start(facts), Start::Done);
    }

    #[test]
    fn an_install_in_use_settles_without_showing_it() {
        for facts in [
            Facts {
                used_before: true,
                ..Facts::default()
            },
            Facts {
                notes_read: true,
                ..Facts::default()
            },
            Facts {
                has_library: true,
                ..Facts::default()
            },
            Facts {
                tracked: true,
                ..Facts::default()
            },
        ] {
            assert_eq!(start(facts), Start::Settle, "{facts:?}");
        }
    }

    #[test]
    fn making_it_creates_videos_too() {
        let tmp = tempfile::tempdir().unwrap();
        let videos = tmp.path().join("Videos");
        let made = make_starter_folder(&videos).unwrap();
        assert_eq!(made, videos.join("Anime"));
        assert!(made.is_dir());
    }

    #[test]
    fn a_folder_already_there_is_kept() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("Anime");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("keep.mkv"), b"x").unwrap();
        let made = make_starter_folder(tmp.path()).unwrap();
        assert_eq!(made, dir);
        assert!(dir.join("keep.mkv").is_file());
    }

    #[test]
    fn a_relative_videos_folder_is_refused() {
        assert!(make_starter_folder(Path::new("Videos")).is_err());
    }
}
