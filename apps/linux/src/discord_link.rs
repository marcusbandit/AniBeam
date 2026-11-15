//! Discord starts the app for a Join or an accepted invite (#966) by opening
//! `discord-<application id>://`, so the desktop needs a handler for that scheme. A small
//! hidden entry of its own, `discord-<id>.desktop`, runs the app the way the user's own entry
//! does, and becomes the handler when nothing else is. The join itself then comes over
//! Discord's socket once the app is up.
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

/// The entry for `id`, running `exec` (the user's own Exec line) with the link as its URL.
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

#[cfg(target_os = "linux")]
mod linux {
    use std::process::{Command, Stdio};

    use crate::scheme::linux::{launcher_entry, quiet};

    pub fn ensure(id: &str) {
        let Some(own) = launcher_entry() else { return };
        let Some(dir) = own.parent().map(|d| d.to_path_buf()) else {
            return;
        };
        let Some(exec) = std::fs::read_to_string(&own)
            .ok()
            .and_then(|t| super::main_exec(&t))
        else {
            return;
        };
        let name = format!("discord-{id}.desktop");
        let path = dir.join(&name);
        let want = super::entry(id, &exec);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(want.as_str()) {
            let tmp = dir.join(format!(".{name}.anibeam-tmp"));
            if std::fs::write(&tmp, &want).is_err() || std::fs::rename(&tmp, &path).is_err() {
                let _ = std::fs::remove_file(&tmp);
                return;
            }
            quiet(Command::new("update-desktop-database").arg(&dir));
        }
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
    fn only_the_main_group_s_exec_counts() {
        let e = "[Desktop Action new]\nExec=/opt/other\n\n[Desktop Entry]\nName=A\nExec=/opt/anibeam %U\n";
        assert_eq!(main_exec(e).as_deref(), Some("/opt/anibeam %U"));
        assert_eq!(main_exec("[Desktop Action x]\nExec=/a\n"), None);
    }
}
