//! `anibeam://` links reach the app (#888). An install made before rooms wrote its
//! desktop entry without `x-scheme-handler/anibeam`, so a browser's "Open in Anibeam"
//! went nowhere. At launch the user's own entry for this app gets the MimeType line
//! and a `%U` on its Exec when they are missing, the desktop's cache is rebuilt, and
//! the entry becomes the handler when nothing else is.
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

/// `entry` with the MimeType and the `%U` it needs, or `None` when it has both. Only the
/// `[Desktop Entry]` group changes; actions and other groups keep their own Exec lines.
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
            if value.split(';').any(|m| m.trim() == MIME) {
                out.push(line.to_owned());
            } else {
                let value = value.trim_end();
                let sep = if value.is_empty() || value.ends_with(';') {
                    ""
                } else {
                    ";"
                };
                out.push(format!("MimeType={value}{sep}{MIME};"));
                changed = true;
            }
        } else if let Some(value) = line.strip_prefix("Exec=") {
            if ["%u", "%U"].iter().any(|code| value.contains(code)) {
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
        out.insert(at, format!("MimeType={MIME};"));
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
mod linux {
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use super::{MIME, patch, program};

    const MAIN: &str = "com.marcusrosado.AniBeam.desktop";
    const DEV: &str = "com.marcusrosado.AniBeam.Dev.desktop";

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
            // Written beside it and moved over it, so a reader never sees half a file
            let tmp = dir.join(format!(".{name}.anibeam-tmp"));
            let written = std::fs::write(&tmp, patched).is_ok()
                && std::fs::metadata(&path)
                    .and_then(|m| std::fs::set_permissions(&tmp, m.permissions()))
                    .is_ok()
                && std::fs::rename(&tmp, &path).is_ok();
            if !written {
                let _ = std::fs::remove_file(&tmp);
                return;
            }
            quiet(Command::new("update-desktop-database").arg(&dir));
        }
        let current = Command::new("xdg-mime")
            .args(["query", "default", MIME])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        if let Ok(out) = current
            && String::from_utf8_lossy(&out.stdout).trim().is_empty()
        {
            quiet(Command::new("xdg-mime").args(["default", name, MIME]));
        }
    }

    fn quiet(cmd: &mut Command) {
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

    const OLD: &str = "[Desktop Entry]\nType=Application\nName=AniBeam\n\
        Exec=env APPIMAGELAUNCHER_DISABLE=1 /opt/anibeam %U\nIcon=com.marcusrosado.AniBeam\n\
        Terminal=false\n";

    #[test]
    fn an_entry_from_before_rooms_gets_the_scheme() {
        let patched = patch(OLD).unwrap();
        assert!(patched.ends_with("Terminal=false\nMimeType=x-scheme-handler/anibeam;\n"));
        assert!(patched.contains("Exec=env APPIMAGELAUNCHER_DISABLE=1 /opt/anibeam %U\n"));
        assert_eq!(patch(&patched), None);
    }

    #[test]
    fn an_exec_without_a_link_gets_one() {
        let patched = patch("[Desktop Entry]\nExec=env A=1 /x/anibeam\n").unwrap();
        assert_eq!(
            patched,
            "[Desktop Entry]\nExec=env A=1 /x/anibeam %U\nMimeType=x-scheme-handler/anibeam;\n"
        );
        assert_eq!(
            patch("[Desktop Entry]\nExec=anibeam %u\nMimeType=x-scheme-handler/anibeam;\n"),
            None
        );
    }

    #[test]
    fn other_types_stay_and_the_scheme_joins_them() {
        assert_eq!(
            patch("[Desktop Entry]\nExec=a %U\nMimeType=video/mp4;\n").unwrap(),
            "[Desktop Entry]\nExec=a %U\nMimeType=video/mp4;x-scheme-handler/anibeam;\n"
        );
        assert_eq!(
            patch("[Desktop Entry]\nExec=a %U\nMimeType=video/mp4\n").unwrap(),
            "[Desktop Entry]\nExec=a %U\nMimeType=video/mp4;x-scheme-handler/anibeam;\n"
        );
    }

    #[test]
    fn actions_keep_their_own_lines() {
        let entry = "[Desktop Entry]\nExec=a %U\n\n[Desktop Action play]\nExec=a --play\n";
        assert_eq!(
            patch(entry).unwrap(),
            "[Desktop Entry]\nExec=a %U\nMimeType=x-scheme-handler/anibeam;\n\n[Desktop Action play]\nExec=a --play\n"
        );
        assert_eq!(patch("Name=x\n"), None);
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
