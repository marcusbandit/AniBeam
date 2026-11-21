//! Discord starts the app for a Join or an accepted invite (#966) by opening
//! `discord-<application id>://`, so the desktop needs a handler for that scheme. A small
//! hidden entry of its own, `discord-<id>.desktop`, runs the app the way the entry that
//! launched it does, and becomes the handler when nothing else is. The join itself then comes over
//! Discord's socket once the app is up.
//!
//! The hidden entry always goes in the user's own folder. An AppImage's launcher entry is
//! there too; a package (anibeam-bin, deb, rpm) installs its entry under `/usr/share`, and
//! the hidden entry runs what that one runs (#1167). A package's launch never replaces a
//! hidden entry that still runs something, so it cannot take Join from an install the user
//! set up themselves; one left pointing at a program that is gone is written again.
//!
//! Only the user's own files are touched, never with a sandbox root, a drive or a shot, and
//! nothing when the build has no Discord application.

/// Writes the entry once, in the background.
pub fn ensure() {
    #[cfg(target_os = "linux")]
    {
        let args = crate::runtime::args();
        if args.root.is_some() || args.drive.is_some() || args.shoot.is_some() {
            return;
        }
        let Some(id) = anibeam_core::presence::bundled_client_id() else {
            return;
        };
        let id = id.to_string();
        let _ = std::thread::Builder::new()
            .name("discord-link".into())
            .spawn(move || linux::ensure(&id));
    }
}

/// The entry for `id`, running `exec` (the launching entry's Exec line) with the link as its
/// URL.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn entry(id: &str, exec: &str) -> String {
    let exec = exec.trim();
    let exec = ["%U", "%u", "%F", "%f"]
        .iter()
        .find(|code| exec.contains(*code))
        .map(|code| exec.replace(code, "%u"))
        .unwrap_or_else(|| format!("{exec} %u"));
    format!(
        "[Desktop Entry]\nType=Application\nName=Anibeam\nExec={exec}\nNoDisplay=true\n\
         Terminal=false\nMimeType=x-scheme-handler/discord-{id};\n"
    )
}

/// The Exec line of an entry's `[Desktop Entry]` group, never an action's.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn main_exec(entry: &str) -> Option<String> {
    let mut in_main = false;
    for line in entry.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_main = t == "[Desktop Entry]";
        } else if in_main && let Some(exec) = t.strip_prefix("Exec=") {
            return Some(exec.to_string());
        }
    }
    None
}

/// Whether the hidden entry already there (`current`) stays: always when it is the one
/// wanted, and when a package launched this run and it still runs something (`live`).
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn keep(current: Option<&str>, want: &str, mine: bool, live: impl Fn(&str) -> bool) -> bool {
    match current {
        None => false,
        Some(c) if c == want => true,
        Some(c) => !mine && main_exec(c).is_some_and(|e| live(&e)),
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::path::Path;
    use std::process::{Command, Stdio};

    use crate::scheme::linux::{applications, launching_entry, quiet, resolve};

    pub fn ensure(id: &str) {
        let Some(dir) = applications() else { return };
        let Some((own, mine)) = launching_entry() else {
            return;
        };
        match place(&dir, &own, mine, id) {
            Some(true) => quiet(Command::new("update-desktop-database").arg(&dir)),
            Some(false) => {}
            None => return,
        }
        let name = format!("discord-{id}.desktop");
        let mime = format!("x-scheme-handler/discord-{id}");
        let current = Command::new("xdg-mime")
            .args(["query", "default", &mime])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        if let Ok(out) = current
            && String::from_utf8_lossy(&out.stdout).trim().is_empty()
        {
            quiet(Command::new("xdg-mime").args(["default", &name, &mime]));
        }
    }

    /// Writes `discord-<id>.desktop` into `dir`, made when missing, from the Exec of `own`.
    /// Some(true) when it wrote, Some(false) when the entry there stays, None on a failure.
    pub(super) fn place(dir: &Path, own: &Path, mine: bool, id: &str) -> Option<bool> {
        let exec = std::fs::read_to_string(own)
            .ok()
            .and_then(|t| super::main_exec(&t))?;
        std::fs::create_dir_all(dir).ok()?;
        let name = format!("discord-{id}.desktop");
        let path = dir.join(&name);
        let want = super::entry(id, &exec);
        let current = std::fs::read_to_string(&path).ok();
        let path_var = std::env::var_os("PATH");
        let live = |exec: &str| {
            crate::scheme::program(exec).is_some_and(|p| resolve(&p, path_var.as_deref()).is_some())
        };
        if super::keep(current.as_deref(), &want, mine, live) {
            return Some(false);
        }
        let tmp = dir.join(format!(".{name}.anibeam-tmp"));
        if std::fs::write(&tmp, &want).is_err() || std::fs::rename(&tmp, &path).is_err() {
            let _ = std::fs::remove_file(&tmp);
            return None;
        }
        Some(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_entry_runs_the_app_as_its_own_entry_does_with_the_link() {
        let e = entry("123", "env A=1 /opt/anibeam %U");
        assert!(e.contains("\nExec=env A=1 /opt/anibeam %u\n"));
        assert!(e.contains("MimeType=x-scheme-handler/discord-123;"));
        assert!(e.contains("NoDisplay=true"));
        assert!(entry("9", "/opt/anibeam").contains("\nExec=/opt/anibeam %u\n"));
    }

    #[test]
    fn a_package_launch_keeps_a_hidden_entry_that_still_runs() {
        let want = entry("1", "anibeam %u");
        let dev = entry("1", "/opt/dev/anibeam %u");
        let live = |e: &str| e.starts_with("/opt/dev");
        assert!(keep(Some(&want), &want, false, live));
        assert!(keep(Some(&dev), &want, false, live));
        assert!(!keep(Some(&dev), &want, true, live));
        assert!(!keep(Some(&dev), &want, false, |_| false));
        assert!(!keep(None, &want, false, live));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_package_install_gets_the_entry_in_a_user_folder_that_was_not_there() {
        let root = tempfile::tempdir().unwrap();
        let own = root.path().join("com.marcusrosado.Anibeam.desktop");
        std::fs::write(&own, "[Desktop Entry]\nExec=/opt/example/anibeam %u\n").unwrap();
        let dir = root.path().join("data/applications");
        assert_eq!(linux::place(&dir, &own, false, "7"), Some(true));
        let written = std::fs::read_to_string(dir.join("discord-7.desktop")).unwrap();
        assert!(written.contains("\nExec=/opt/example/anibeam %u\n"));
        assert_eq!(linux::place(&dir, &own, false, "7"), Some(false));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    }

    #[test]
    fn only_the_main_group_s_exec_counts() {
        let e = "[Desktop Action new]\nExec=/opt/other\n\n[Desktop Entry]\nName=A\nExec=/opt/anibeam %U\n";
        assert_eq!(main_exec(e).as_deref(), Some("/opt/anibeam %U"));
        assert_eq!(main_exec("[Desktop Action x]\nExec=/a\n"), None);
    }
}
