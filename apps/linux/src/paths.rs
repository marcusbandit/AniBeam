//! The shell's own files. The core has its four XDG directories; the shell adds theme.toml,
//! player.toml, the two theme directories, the three mpv.conf layers and the lock file.
//!
//! On a Mac the packaged files sit in the bundle's `Contents/Resources` rather than
//! `/usr/share/anibeam`, the lock sits in the per-user `$TMPDIR`, since macOS has no
//! `$XDG_RUNTIME_DIR`, and the user's own mpv.conf is still `~/.config/mpv/mpv.conf`, which
//! is where mpv itself reads it on a Mac, not beside the core's `Application Support`
//! directory. Nothing else differs.

use std::path::{Path, PathBuf};

use anibeam_core::CorePaths;

#[derive(Clone, Debug, PartialEq)]
pub struct ShellPaths {
    pub core: CorePaths,
    pub runtime_dir: PathBuf,
    pub builtin_themes: PathBuf,
    pub bundled_mpv_conf_override: Option<PathBuf>,
    /// The directory mpv's own `mpv/` folder sits in, when that is not simply the one above
    /// the core's config directory: only a Mac outside a sandbox sets it.
    pub mpv_config_home: Option<PathBuf>,
}

impl ShellPaths {
    /// Under `root` everything sits inside it, the runtime directory included, so a dev run
    /// or a test never touches the real files. Without a root the core's XDG paths apply
    /// and the lock sits under $XDG_RUNTIME_DIR (on a Mac, $TMPDIR).
    ///
    /// `ANIBEAM_THEMES_DIR` and `ANIBEAM_MPV_CONF` are read once here, into fields, rather
    /// than on every call: a test that never sets them still sees the packaged defaults.
    pub fn resolve(root: Option<&Path>) -> Result<ShellPaths, String> {
        let builtin_themes = std::env::var_os("ANIBEAM_THEMES_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| share_dir().join("themes"));
        let bundled_mpv_conf_override = std::env::var_os("ANIBEAM_MPV_CONF").map(PathBuf::from);
        match root {
            Some(r) => Ok(ShellPaths {
                core: CorePaths::under(r),
                runtime_dir: r.join("runtime"),
                builtin_themes,
                bundled_mpv_conf_override,
                mpv_config_home: None,
            }),
            None => {
                let core = CorePaths::xdg().map_err(|e| e.to_string())?;
                let runtime_dir = user_runtime_dir()?;
                Ok(ShellPaths {
                    core,
                    runtime_dir,
                    builtin_themes,
                    bundled_mpv_conf_override,
                    mpv_config_home: mac_config_home(),
                })
            }
        }
    }
}

/// Where the package puts the built-in themes and the bundled mpv.conf. On a Mac that is
/// the `Contents/Resources` of the bundle the binary runs from (`Contents/MacOS/anibeam`, two
/// levels up). Elsewhere it is `share/anibeam` beside the binary's `bin`, when there is one:
/// `/usr/share/anibeam` for the PKGBUILD's `/usr/bin/anibeam`, and the same pair inside an
/// AppImage, which is mounted somewhere new on every run. A binary with no such directory
/// beside it, such as a cargo build, reads the installed package's.
pub fn share_dir() -> PathBuf {
    let prefix = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.parent()?.to_path_buf()));
    #[cfg(target_os = "macos")]
    if let Some(contents) = &prefix {
        return contents.join("Resources");
    }
    beside_bin(prefix.as_deref()).unwrap_or_else(|| PathBuf::from("/usr/share/anibeam"))
}

/// Names the Downloads helpers a package carries to the core, which looks for them in these
/// variables: anirss under the share dir, qbittorrent-nox beside the binary (the Mac app and
/// the AppImage carry both). A variable already set is left alone, so a run can point at
/// another copy. A setenv, so it has to run before any thread exists.
pub fn name_bundled_helpers() {
    let beside_exe = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("qbittorrent-nox")));
    let helpers = [
        (
            "ANIBEAM_ANIRSS",
            Some(share_dir().join("anirss/bin/anirss")),
        ),
        ("ANIBEAM_QBITTORRENT_NOX", beside_exe),
    ];
    for (name, path) in helpers {
        if let Some(path) = path.filter(|p| p.is_file())
            && std::env::var_os(name).is_none()
        {
            // SAFETY: called from main before the core's runtime or Qt starts a thread.
            unsafe { std::env::set_var(name, path) };
        }
    }
}

/// `<prefix>/share/anibeam` when that directory exists.
fn beside_bin(prefix: Option<&std::path::Path>) -> Option<PathBuf> {
    let dir = prefix?.join("share/anibeam");
    dir.is_dir().then_some(dir)
}

/// `$XDG_CONFIG_HOME`, else `~/.config`, on a Mac; `None` elsewhere, where the core's own
/// config directory already sits in it.
fn mac_config_home() -> Option<PathBuf> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
}

/// The per-user directory the lock lives in. Linux has `$XDG_RUNTIME_DIR` for exactly
/// this; macOS has no such thing, and its `$TMPDIR` is already per user
/// (`/var/folders/.../T/`), so that is where a Mac keeps it.
fn user_runtime_dir() -> Result<PathBuf, String> {
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        return Ok(PathBuf::from(dir));
    }
    if cfg!(target_os = "macos") {
        return Ok(std::env::temp_dir());
    }
    Err("XDG_RUNTIME_DIR is not set".to_string())
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
        self.mpv_config_home
            .clone()
            .or_else(|| self.config_dir().parent().map(Path::to_path_buf))
            .map(|p| p.join("mpv").join("mpv.conf"))
            .unwrap_or_default()
    }
    pub fn bundled_mpv_conf(&self) -> PathBuf {
        self.bundled_mpv_conf_override
            .clone()
            .unwrap_or_else(|| share_dir().join("mpv.conf"))
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
        assert_eq!(p.bundled_mpv_conf(), share_dir().join("mpv.conf"));
        assert_eq!(p.cache_dir(), PathBuf::from("/tmp/sandbox/cache"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn the_package_installs_under_usr_share() {
        assert_eq!(share_dir(), PathBuf::from("/usr/share/anibeam"));
    }

    #[test]
    fn a_share_dir_beside_the_binary_wins() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(beside_bin(Some(root.path())), None);
        std::fs::create_dir_all(root.path().join("share/anibeam")).unwrap();
        assert_eq!(
            beside_bin(Some(root.path())),
            Some(root.path().join("share/anibeam"))
        );
        assert_eq!(beside_bin(None), None);
    }
}
