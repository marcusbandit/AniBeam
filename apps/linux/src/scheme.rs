//! `anibeam://` links reach the app (#888), and so do magnet links (#1391). An install made
//! before rooms wrote its desktop entry without `x-scheme-handler/anibeam`, so a browser's
//! "Open in Anibeam" went nowhere, and one made before magnets lacks
//! `x-scheme-handler/magnet`. At launch the user's own entry for this app gets both on its
//! MimeType line and a `%U` on its Exec when it has no field code for files or links yet,
//! the desktop's cache is rebuilt, and the entry becomes the handler of each when nothing
//! else is. Settings, Downloads makes it the magnet handler when another app is.
//!
//! Only the user's entry is touched (`$XDG_DATA_HOME/applications`), never a system
//! one, and nothing at all when there is none. It runs off the main thread, says
//! nothing, and a failure leaves startup alone.

/// Checks the entry once, in the background. A sandbox run (`--root`, a drive or a shot)
/// leaves the user's files alone.
pub fn ensure() {
    #[cfg(target_os = "linux")]
    {
        let args = crate::runtime::args();
        if args.root.is_some() || args.drive.is_some() || args.shoot.is_some() {
            return;
        }
        let _ = std::thread::Builder::new()
            .name("scheme".into())
            .spawn(linux::ensure);
    }
}

/// The scheme's MIME type, as desktop entries and xdg-mime name it.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
const MIME: &str = "x-scheme-handler/anibeam";
/// Magnet links' MIME type.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
const MAGNET: &str = "x-scheme-handler/magnet";
/// Every link type the entry declares.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
const TYPES: [&str; 2] = [MIME, MAGNET];

/// Who opens magnet links: "anibeam", "other" or "none". None where the desktop has no such
/// setting, or no entry of this app's to name.
pub fn magnet_handler() -> Option<&'static str> {
    #[cfg(target_os = "linux")]
    {
        linux::magnet_handler()
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Makes this app the magnet handler, then says who is. A sandbox run changes nothing.
pub fn make_magnet_default() -> Option<&'static str> {
    #[cfg(target_os = "linux")]
    {
        let args = crate::runtime::args();
        if args.root.is_none() && args.drive.is_none() && args.shoot.is_none() {
            linux::make_magnet_default();
        }
        linux::magnet_handler()
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// `entry` with every type of `TYPES` on its MimeType line and the `%U` it needs, or
/// `None` when it has them all. An Exec
/// that already takes files or links (`%f`, `%F`, `%u` or `%U`) stays as it is: the spec
/// allows one of the four. Only the `[Desktop Entry]` group changes; actions and other
/// groups keep their own Exec lines.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn patch(entry: &str) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    let mut in_main = false;
    let mut main_end: Option<usize> = None;
    let mut has_mime = false;
    let mut changed = false;
    for line in entry.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if in_main {
                main_end = Some(out.len());
            }
            in_main = trimmed == "[Desktop Entry]";
            out.push(line.to_owned());
            continue;
        }
        if !in_main {
            out.push(line.to_owned());
            continue;
        }
        if let Some(value) = line.strip_prefix("MimeType=") {
            has_mime = true;
            let missing: Vec<&str> = TYPES
                .iter()
                .copied()
                .filter(|t| !value.split(';').any(|m| m.trim() == *t))
                .collect();
            if missing.is_empty() {
                out.push(line.to_owned());
            } else {
                let value = value.trim_end();
                let sep = if value.is_empty() || value.ends_with(';') {
                    ""
                } else {
                    ";"
                };
                out.push(format!("MimeType={value}{sep}{};", missing.join(";")));
                changed = true;
            }
        } else if let Some(value) = line.strip_prefix("Exec=") {
            if ["%f", "%F", "%u", "%U"]
                .iter()
                .any(|code| value.contains(code))
            {
                out.push(line.to_owned());
            } else {
                out.push(format!("Exec={} %U", value.trim_end()));
                changed = true;
            }
        } else {
            out.push(line.to_owned());
        }
    }
    if in_main && main_end.is_none() {
        main_end = Some(out.len());
    }
    let main_end = main_end?;
    if !has_mime {
        // After the group's last line that says something, so a blank line before the
        // next group stays where it was
        let mut at = main_end;
        while at > 0 && out[at - 1].trim().is_empty() {
            at -= 1;
        }
        out.insert(at, format!("MimeType={};", TYPES.join(";")));
        changed = true;
    }
    if !changed {
        return None;
    }
    let mut text = out.join("\n");
    if entry.ends_with('\n') {
        text.push('\n');
    }
    Some(text)
}

/// Who `current`, the entry xdg-mime names, is: `ours` (this run's own entry), another app,
/// or nobody. The other build's entry (Dev beside an installed Nightly) counts as another
/// app, since links then open in that build and not this one.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn handler_word(current: &str, ours: &str) -> &'static str {
    let current = current.trim();
    if current.is_empty() {
        "none"
    } else if current == ours {
        "anibeam"
    } else {
        "other"
    }
}

/// The program an Exec line runs: the first word that is not `env` or a `NAME=value`
/// assignment, unquoted.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn program(exec: &str) -> Option<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut any = false;
    let mut chars = exec.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                quoted = !quoted;
                any = true;
            }
            '\\' if quoted => {
                if let Some(next) = chars.next() {
                    word.push(next);
                }
            }
            c if c.is_whitespace() && !quoted => {
                if any || !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                    any = false;
                }
            }
            c => word.push(c),
        }
    }
    if any || !word.is_empty() {
        words.push(word);
    }
    let mut words = words.into_iter().peekable();
    if words
        .peek()
        .is_some_and(|w| w == "env" || w.ends_with("/env"))
    {
        words.next();
    }
    words.find(|w| !(w.contains('=') && !w.starts_with('/')) && !w.starts_with('-'))
}

#[cfg(target_os = "linux")]
pub(crate) mod linux {
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use super::{MAGNET, TYPES, patch, program};

    const MAIN: &str = "com.marcusrosado.Anibeam.desktop";
    const DEV: &str = "com.marcusrosado.Anibeam.Dev.desktop";

    fn applications() -> Option<PathBuf> {
        let data = std::env::var_os("XDG_DATA_HOME")
            .filter(|d| !d.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
        Some(data.join("applications"))
    }

    /// What is running: the AppImage file rather than its mount, else the binary.
    fn running() -> Option<PathBuf> {
        let exe = std::env::var_os("APPIMAGE")
            .filter(|a| !a.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::current_exe().ok())?;
        exe.canonicalize().ok()
    }

    fn runs(entry: &Path, exe: &Path) -> bool {
        let Ok(text) = std::fs::read_to_string(entry) else {
            return false;
        };
        text.lines()
            .find_map(|l| l.strip_prefix("Exec="))
            .and_then(program)
            .and_then(|p| Path::new(&p).canonicalize().ok())
            .is_some_and(|p| p == exe)
    }

    /// The entry that launched this run: the one the launcher named, else the one whose
    /// Exec is this binary, else the app's own entry when it exists.
    fn pick(dir: &Path) -> Option<&'static str> {
        let ours = [MAIN, DEV];
        if let Some(launched) = std::env::var_os("GIO_LAUNCHED_DESKTOP_FILE").map(PathBuf::from)
            && launched.parent() == Some(dir)
            && let Some(name) = ours
                .iter()
                .find(|n| launched.file_name() == Some(n.as_ref()))
        {
            return Some(name);
        }
        if let Some(exe) = running()
            && let Some(name) = ours.iter().find(|n| runs(&dir.join(n), &exe))
        {
            return Some(name);
        }
        dir.join(MAIN).is_file().then_some(MAIN)
    }

    pub fn ensure() {
        let Some(dir) = applications() else { return };
        let Some(name) = pick(&dir) else { return };
        let path = dir.join(name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return;
        };
        if let Some(patched) = patch(&text) {
            if !replace(&path, &patched) {
                return;
            }
            quiet(Command::new("update-desktop-database").arg(&dir));
        }
        for mime in TYPES {
            if default_of(mime).is_some_and(|d| d.is_empty()) {
                quiet(Command::new("xdg-mime").args(["default", name, mime]));
            }
        }
    }

    /// The desktop entry xdg-mime names for `mime`, empty when there is none; None when
    /// xdg-mime itself cannot be run.
    fn default_of(mime: &str) -> Option<String> {
        let out = Command::new("xdg-mime")
            .args(["query", "default", mime])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    /// This app's entry by name: the user's own that launched it, else the first of the
    /// app's entries any data directory holds (a package's, under /usr/share).
    fn entry_name() -> Option<&'static str> {
        if let Some(name) = applications().as_deref().and_then(pick) {
            return Some(name);
        }
        let dirs = std::env::var("XDG_DATA_DIRS")
            .ok()
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
        [MAIN, DEV].into_iter().find(|name| {
            dirs.split(':')
                .any(|d| Path::new(d).join("applications").join(name).is_file())
        })
    }

    pub fn magnet_handler() -> Option<&'static str> {
        let ours = entry_name()?;
        let current = default_of(MAGNET)?;
        Some(super::handler_word(&current, ours))
    }

    pub fn make_magnet_default() {
        if let Some(name) = entry_name() {
            quiet(Command::new("xdg-mime").args(["default", name, MAGNET]));
        }
    }

    /// The user's own entry that launched this run, for another handler to start the app
    /// the same way (#966).
    pub fn launcher_entry() -> Option<PathBuf> {
        let dir = applications()?;
        let name = pick(&dir)?;
        Some(dir.join(name))
    }

    /// Puts `text` in the file at `path`: written beside the file and moved over it, so a
    /// reader never sees half a file. An entry that is a symlink (into a dotfiles checkout,
    /// say) gets its target written and stays a link.
    pub(crate) fn replace(path: &Path, text: &str) -> bool {
        let Ok(target) = path.canonicalize() else {
            return false;
        };
        let (Some(dir), Some(name)) = (target.parent(), target.file_name()) else {
            return false;
        };
        let tmp = dir.join(format!(".{}.anibeam-tmp", name.to_string_lossy()));
        let written = std::fs::write(&tmp, text).is_ok()
            && std::fs::metadata(&target)
                .and_then(|m| std::fs::set_permissions(&tmp, m.permissions()))
                .is_ok()
            && std::fs::rename(&tmp, &target).is_ok();
        if !written {
            let _ = std::fs::remove_file(&tmp);
        }
        written
    }

    pub(crate) fn quiet(cmd: &mut Command) {
        let _ = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD: &str = "[Desktop Entry]\nType=Application\nName=Anibeam\n\
        Exec=env APPIMAGELAUNCHER_DISABLE=1 /opt/anibeam %U\nIcon=com.marcusrosado.Anibeam\n\
        Terminal=false\n";

    #[test]
    fn an_entry_from_before_rooms_gets_the_scheme() {
        let patched = patch(OLD).unwrap();
        assert!(patched.ends_with(
            "Terminal=false\nMimeType=x-scheme-handler/anibeam;x-scheme-handler/magnet;\n"
        ));
        assert!(patched.contains("Exec=env APPIMAGELAUNCHER_DISABLE=1 /opt/anibeam %U\n"));
        assert_eq!(patch(&patched), None);
    }

    #[test]
    fn an_exec_without_a_link_gets_one() {
        let patched = patch("[Desktop Entry]\nExec=env A=1 /x/anibeam\n").unwrap();
        assert_eq!(
            patched,
            "[Desktop Entry]\nExec=env A=1 /x/anibeam %U\nMimeType=x-scheme-handler/anibeam;x-scheme-handler/magnet;\n"
        );
        for code in ["%f", "%F", "%u", "%U"] {
            assert_eq!(
                patch(&format!(
                    "[Desktop Entry]\nExec=anibeam {code}\nMimeType=x-scheme-handler/anibeam;x-scheme-handler/magnet;\n"
                )),
                None,
                "{code}"
            );
        }
        assert_eq!(
            patch("[Desktop Entry]\nExec=anibeam %F\n").unwrap(),
            "[Desktop Entry]\nExec=anibeam %F\nMimeType=x-scheme-handler/anibeam;x-scheme-handler/magnet;\n"
        );
    }

    #[test]
    fn other_types_stay_and_the_scheme_joins_them() {
        assert_eq!(
            patch("[Desktop Entry]\nExec=a %U\nMimeType=video/mp4;\n").unwrap(),
            "[Desktop Entry]\nExec=a %U\nMimeType=video/mp4;x-scheme-handler/anibeam;x-scheme-handler/magnet;\n"
        );
        assert_eq!(
            patch("[Desktop Entry]\nExec=a %U\nMimeType=video/mp4\n").unwrap(),
            "[Desktop Entry]\nExec=a %U\nMimeType=video/mp4;x-scheme-handler/anibeam;x-scheme-handler/magnet;\n"
        );
    }

    #[test]
    fn an_entry_from_before_magnets_gets_the_magnet_type_only() {
        assert_eq!(
            patch("[Desktop Entry]\nExec=a %u\nMimeType=x-scheme-handler/anibeam;\n").unwrap(),
            "[Desktop Entry]\nExec=a %u\nMimeType=x-scheme-handler/anibeam;x-scheme-handler/magnet;\n"
        );
        assert_eq!(
            patch(
                "[Desktop Entry]\nExec=a %u\nMimeType=x-scheme-handler/magnet;x-scheme-handler/anibeam;\n"
            ),
            None
        );
    }

    #[test]
    fn the_magnet_handler_is_ours_anothers_or_nobodys() {
        let ours = "com.marcusrosado.Anibeam.desktop";
        assert_eq!(
            handler_word("com.marcusrosado.Anibeam.desktop\n", ours),
            "anibeam"
        );
        // The other build's entry opens links in that build, not in this one
        assert_eq!(
            handler_word("com.marcusrosado.Anibeam.Dev.desktop", ours),
            "other"
        );
        assert_eq!(handler_word("org.example.Torrents.desktop", ours), "other");
        assert_eq!(handler_word("  ", ours), "none");
    }

    #[test]
    fn actions_keep_their_own_lines() {
        let entry = "[Desktop Entry]\nExec=a %U\n\n[Desktop Action play]\nExec=a --play\n";
        assert_eq!(
            patch(entry).unwrap(),
            "[Desktop Entry]\nExec=a %U\nMimeType=x-scheme-handler/anibeam;x-scheme-handler/magnet;\n\n[Desktop Action play]\nExec=a --play\n"
        );
        assert_eq!(patch("Name=x\n"), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_symlinked_entry_stays_a_link() {
        let home = tempfile::tempdir().unwrap();
        let dotfiles = home.path().join("dotfiles");
        let apps = home.path().join("applications");
        std::fs::create_dir_all(&dotfiles).unwrap();
        std::fs::create_dir_all(&apps).unwrap();
        let real = dotfiles.join("entry.desktop");
        let link = apps.join("entry.desktop");
        std::fs::write(&real, OLD).unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert!(linux::replace(&link, "patched\n"));
        assert!(
            std::fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read_to_string(&real).unwrap(), "patched\n");
        assert_eq!(std::fs::read_dir(&apps).unwrap().count(), 1);
        assert_eq!(std::fs::read_dir(&dotfiles).unwrap().count(), 1);

        let plain = apps.join("plain.desktop");
        std::fs::write(&plain, OLD).unwrap();
        assert!(linux::replace(&plain, "again\n"));
        assert_eq!(std::fs::read_to_string(&plain).unwrap(), "again\n");
        assert!(!linux::replace(&apps.join("missing.desktop"), "x"));
    }

    #[test]
    fn the_program_an_exec_runs() {
        assert_eq!(
            program("env APPIMAGELAUNCHER_DISABLE=1 /opt/example/bin/anibeam %U").as_deref(),
            Some("/opt/example/bin/anibeam")
        );
        assert_eq!(
            program("env A=/x B=2 \"/path with space/anibeam\" %u").as_deref(),
            Some("/path with space/anibeam")
        );
        assert_eq!(program("anibeam %u").as_deref(), Some("anibeam"));
    }
}
