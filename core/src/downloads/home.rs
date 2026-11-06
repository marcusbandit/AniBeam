//! The home anirss sees (#637). An installed AniBeam shares the person's own home, so it
//! reads the anirss they set up and the qBittorrent password anirss saved. Any other paths,
//! a `--root` run, a test or the phone, get a home of their own at `<state>/home`: the core
//! reads anirss's config and saved password from there, AniBeam's own qBittorrent saves into
//! it, and every anirss the core starts gets it as `HOME` and its XDG homes, so a sandboxed
//! run never reads, writes or logs in with the person's own setup.

use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::CorePaths;

#[derive(Clone, Debug, PartialEq)]
pub enum AnirssHome {
    /// The person's own home: an installed AniBeam on its XDG paths.
    Shared,
    /// A home inside the run's own state directory.
    Sandbox(PathBuf),
}

impl AnirssHome {
    pub fn of(paths: &CorePaths) -> AnirssHome {
        if CorePaths::xdg().is_ok_and(|xdg| &xdg == paths) {
            AnirssHome::Shared
        } else {
            AnirssHome::Sandbox(Path::new(&paths.state_dir).join("home"))
        }
    }

    /// The home a sandboxed run gives anirss; `None` for the person's own.
    pub fn sandbox(&self) -> Option<&Path> {
        match self {
            AnirssHome::Shared => None,
            AnirssHome::Sandbox(dir) => Some(dir),
        }
    }

    /// The folder anirss expands `~` against: the person's own home, or the sandbox's.
    pub fn dir(&self) -> Option<PathBuf> {
        match self {
            AnirssHome::Shared => std::env::home_dir(),
            AnirssHome::Sandbox(dir) => Some(dir.clone()),
        }
    }

    /// `~/.config/anirss/config.toml`, honouring `XDG_CONFIG_HOME` on the person's own home.
    pub fn config_path(&self) -> Option<PathBuf> {
        let base = match self {
            AnirssHome::Shared => match std::env::var("XDG_CONFIG_HOME") {
                Ok(d) if !d.is_empty() => PathBuf::from(d),
                _ => std::env::home_dir()?.join(".config"),
            },
            AnirssHome::Sandbox(dir) => dir.join(".config"),
        };
        Some(base.join("anirss").join("config.toml"))
    }

    /// anirss's state directory, where its CLI saves the qBittorrent password.
    pub fn state_dir(&self) -> Option<PathBuf> {
        let base = match self {
            AnirssHome::Shared => match std::env::var("XDG_STATE_HOME") {
                Ok(d) if !d.is_empty() => PathBuf::from(d),
                _ => std::env::home_dir()?.join(".local").join("state"),
            },
            AnirssHome::Sandbox(dir) => dir.join(".local").join("state"),
        };
        Some(base.join("anirss"))
    }

    /// Where AniBeam's own qBittorrent saves: `~/Downloads/Anime`.
    pub fn save_dir(&self) -> Option<PathBuf> {
        Some(self.dir()?.join("Downloads").join("Anime"))
    }

    /// Points a child at the sandboxed home: `HOME` (`USERPROFILE` on Windows) and the four
    /// XDG homes inside it, made first so the child finds them. Python's user site stays
    /// where it was, so an anirss installed with `pip --user` still imports. Nothing changes
    /// for the person's own home.
    pub fn apply(&self, cmd: &mut Command) {
        let Some(dir) = self.sandbox() else {
            return;
        };
        let _ = std::fs::create_dir_all(dir);
        if cfg!(target_os = "linux")
            && std::env::var_os("PYTHONUSERBASE").is_none()
            && let Some(real) = std::env::home_dir()
        {
            cmd.env("PYTHONUSERBASE", real.join(".local"));
        }
        cmd.env("HOME", dir)
            .env("XDG_CONFIG_HOME", dir.join(".config"))
            .env("XDG_DATA_HOME", dir.join(".local").join("share"))
            .env("XDG_STATE_HOME", dir.join(".local").join("state"))
            .env("XDG_CACHE_HOME", dir.join(".cache"));
        if cfg!(windows) {
            cmd.env("USERPROFILE", dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_under_a_root_get_a_home_inside_it() {
        let dir = tempfile::tempdir().unwrap();
        let home = AnirssHome::of(&CorePaths::under(dir.path()));
        let inside = dir.path().join("state").join("home");
        assert_eq!(home.sandbox(), Some(inside.as_path()));
        assert_eq!(
            home.config_path().unwrap(),
            inside.join(".config/anirss/config.toml")
        );
        assert_eq!(
            home.state_dir().unwrap(),
            inside.join(".local/state/anirss")
        );
        assert_eq!(home.save_dir().unwrap(), inside.join("Downloads/Anime"));
    }

    #[test]
    fn the_xdg_paths_share_the_persons_own_home() {
        let Ok(xdg) = CorePaths::xdg() else {
            return;
        };
        assert_eq!(AnirssHome::of(&xdg), AnirssHome::Shared);
    }
}
