//! The shell's own files. The core has its four XDG directories; the shell adds theme.toml,
//! player.toml, the two theme directories, the three mpv.conf layers and the lock file.

use std::path::{Path, PathBuf};

use anibeam_core::CorePaths;

#[derive(Clone, Debug, PartialEq)]
pub struct ShellPaths {
    pub core: CorePaths,
    pub runtime_dir: PathBuf,
    pub builtin_themes: PathBuf,
    pub bundled_mpv_conf_override: Option<PathBuf>,
}

impl ShellPaths {
    /// Under `root` everything sits inside it, the runtime directory included, so a dev run
    /// or a test never touches the real files. Without a root the core's XDG paths apply
    /// and the lock sits under $XDG_RUNTIME_DIR.
    ///
    /// `ANIBEAM_THEMES_DIR` and `ANIBEAM_MPV_CONF` are read once here, into fields, rather
    /// than on every call: a test that never sets them still sees the packaged defaults.
    pub fn resolve(root: Option<&Path>) -> Result<ShellPaths, String> {
        let builtin_themes = std::env::var_os("ANIBEAM_THEMES_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/usr/share/anibeam/themes"));
        let bundled_mpv_conf_override = std::env::var_os("ANIBEAM_MPV_CONF").map(PathBuf::from);
        match root {
            Some(r) => Ok(ShellPaths {
                core: CorePaths::under(r),
                runtime_dir: r.join("runtime"),
                builtin_themes,
                bundled_mpv_conf_override,
            }),
            None => {
                let core = CorePaths::xdg().map_err(|e| e.to_string())?;
                let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
                    .map(PathBuf::from)
                    .ok_or_else(|| "XDG_RUNTIME_DIR is not set".to_string())?;
                Ok(ShellPaths {
                    core,
                    runtime_dir,
                    builtin_themes,
                    bundled_mpv_conf_override,
                })
            }
        }
    }
}

/// What the theme engine reads: theme.toml and the two theme directories.
impl ShellPaths {
    pub fn config_dir(&self) -> PathBuf {
        PathBuf::from(&self.core.config_dir)
    }
    pub fn theme_toml(&self) -> PathBuf {
        self.config_dir().join("theme.toml")
    }
    pub fn user_themes_dir(&self) -> PathBuf {
        self.config_dir().join("themes")
    }
    pub fn builtin_themes_dir(&self) -> PathBuf {
        self.builtin_themes.clone()
    }
}

/// What the player reads: player.toml, the three mpv.conf layers, and the lock file the
/// single instance is held by.
impl ShellPaths {
    pub fn player_toml(&self) -> PathBuf {
        self.config_dir().join("player.toml")
    }
    pub fn anibeam_mpv_conf(&self) -> PathBuf {
        self.config_dir().join("mpv.conf")
    }
    /// $XDG_CONFIG_HOME/mpv/mpv.conf: the user's own, behind the Use my mpv.conf setting.
    pub fn user_mpv_conf(&self) -> PathBuf {
        self.config_dir()
            .parent()
            .map(|p| p.join("mpv").join("mpv.conf"))
            .unwrap_or_default()
    }
    pub fn bundled_mpv_conf(&self) -> PathBuf {
        self.bundled_mpv_conf_override
            .clone()
            .unwrap_or_else(|| PathBuf::from("/usr/share/anibeam/mpv.conf"))
    }
    pub fn lock_path(&self) -> PathBuf {
        self.runtime_dir.join("anibeam.lock")
    }
    /// Where the subtitle preview writes its generated sample.srt: the same directory the
    /// core's own image cache lives under.
    pub fn cache_dir(&self) -> PathBuf {
        PathBuf::from(&self.core.cache_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_root_keeps_every_path_inside_it() {
        let p = ShellPaths::resolve(Some(Path::new("/tmp/sandbox"))).unwrap();
        assert_eq!(
            p.theme_toml(),
            PathBuf::from("/tmp/sandbox/config/theme.toml")
        );
        assert_eq!(
            p.player_toml(),
            PathBuf::from("/tmp/sandbox/config/player.toml")
        );
        assert_eq!(
            p.user_themes_dir(),
            PathBuf::from("/tmp/sandbox/config/themes")
        );
        assert_eq!(
            p.anibeam_mpv_conf(),
            PathBuf::from("/tmp/sandbox/config/mpv.conf")
        );
        assert_eq!(
            p.user_mpv_conf(),
            PathBuf::from("/tmp/sandbox/mpv/mpv.conf")
        );
        assert_eq!(
            p.lock_path(),
            PathBuf::from("/tmp/sandbox/runtime/anibeam.lock")
        );
        assert_eq!(
            p.bundled_mpv_conf(),
            PathBuf::from("/usr/share/anibeam/mpv.conf")
        );
        assert_eq!(p.cache_dir(), PathBuf::from("/tmp/sandbox/cache"));
    }
}
