//! The qBittorrent AniBeam runs itself, for someone who has none of their
//! own: `qbittorrent-nox` with a profile under AniBeam's data, its Web UI on
//! loopback only and open to loopback without a login. It keeps running when
//! AniBeam quits, because the subscriptions download through its RSS rules;
//! the next start finds it up and uses it as it is.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

/// The Web UI user the profile names. Loopback needs no login, so this is
/// only what anirss and the settings page show.
pub const USERNAME: &str = "anibeam";
/// How long a fresh start gets to answer on its Web UI.
pub const READY_TIMEOUT: Duration = Duration::from_secs(30);
pub const READY_POLL: Duration = Duration::from_millis(250);
/// Set by a shell whose package carries its own qbittorrent-nox.
const BUNDLED_ENV: &str = "ANIBEAM_QBITTORRENT_NOX";
const NAME: &str = "qbittorrent-nox";
/// Lines of the log kept for a failure message.
const LOG_TAIL: usize = 3;

/// The binary to run: the package's own copy when the shell named one, else
/// `qbittorrent-nox` on the PATH the sidecars get.
pub fn binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(BUNDLED_ENV).map(PathBuf::from)
        && path.is_file()
    {
        return Some(path);
    }
    let path = super::anirss::child_path()
        .or_else(|| std::env::var("PATH").ok())
        .unwrap_or_default();
    which(NAME, &path)
}

/// `name` in the first directory of `path` that has it as a file, with
/// `.exe` on Windows.
fn which(name: &str, path: &str) -> Option<PathBuf> {
    let name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    std::env::split_paths(path)
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(|dir| dir.join(&name))
        .find(|candidate| candidate.is_file())
}

/// Where the profile lives: `<data>/qbittorrent`.
pub fn profile_dir(data_dir: &str) -> PathBuf {
    Path::new(data_dir).join("qbittorrent")
}

/// Where finished downloads go unless anirss names a folder: the same
/// `~/Downloads/Anime` anirss itself defaults to.
pub fn save_dir() -> Option<PathBuf> {
    Some(std::env::home_dir()?.join("Downloads").join("Anime"))
}

/// The Web UI port an earlier setup wrote into the profile, if there was one.
pub fn configured_port(profile: &Path) -> Option<u16> {
    let text = std::fs::read_to_string(config_file(profile)).ok()?;
    text.lines()
        .find_map(|l| l.trim().strip_prefix("WebUI\\Port="))
        .and_then(|p| p.trim().parse().ok())
}

/// qBittorrent names a custom profile's settings file `.ini` on macOS and Windows and `.conf`
/// elsewhere (Private::CustomProfile::applicationSettings). A `.conf` beside a macOS profile
/// is never read: the Web UI then asks for a login and keeps its default user.
fn config_file(profile: &Path) -> PathBuf {
    let name = if cfg!(any(windows, target_os = "macos")) {
        "qBittorrent.ini"
    } else {
        "qBittorrent.conf"
    };
    profile.join("qBittorrent").join("config").join(name)
}

fn log_file(profile: &Path) -> PathBuf {
    profile.join("qbittorrent-nox.log")
}

/// Writes the keys AniBeam relies on into the profile's config and leaves
/// every other key as qBittorrent last saved it. The save folder is only
/// filled in when the config has none, so a folder picked later sticks.
pub fn write_config(profile: &Path, port: u16, save: Option<&Path>) -> std::io::Result<()> {
    let file = config_file(profile);
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut text = std::fs::read_to_string(&file).unwrap_or_default();
    let port = port.to_string();
    let always = [
        ("LegalNotice", "Accepted", "true"),
        ("Preferences", "WebUI\\Enabled", "true"),
        ("Preferences", "WebUI\\Address", "127.0.0.1"),
        ("Preferences", "WebUI\\Port", port.as_str()),
        ("Preferences", "WebUI\\LocalHostAuth", "false"),
        ("Preferences", "WebUI\\Username", USERNAME),
        // anirss's subscriptions are qBittorrent RSS rules: without these
        // the feeds are never read and nothing downloads by itself.
        ("RSS", "Session\\EnableProcessing", "true"),
        ("RSS", "AutoDownloader\\EnableProcessing", "true"),
    ];
    for (section, key, value) in always {
        text = set_ini(&text, section, key, value, true);
    }
    if let Some(save) = save {
        // A backslash is an escape in a QSettings file; qBittorrent writes
        // Windows paths with forward slashes.
        let save = save.to_string_lossy().replace('\\', "/");
        text = set_ini(
            &text,
            "BitTorrent",
            "Session\\DefaultSavePath",
            &save,
            false,
        );
        text = set_ini(&text, "Preferences", "Downloads\\SavePath", &save, false);
    }
    std::fs::write(&file, text)
}

/// `text` with `key=value` in `[section]`: the section and the key are added
/// when missing, and an existing key is replaced only with `overwrite`.
fn set_ini(text: &str, section: &str, key: &str, value: &str, overwrite: bool) -> String {
    let header = format!("[{section}]");
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let Some(start) = lines.iter().position(|l| l.trim() == header) else {
        if lines.last().is_some_and(|l| !l.trim().is_empty()) {
            lines.push(String::new());
        }
        lines.push(header);
        lines.push(format!("{key}={value}"));
        return lines.join("\n") + "\n";
    };
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.trim_start().starts_with('['))
        .map_or(lines.len(), |i| start + 1 + i);
    let found = (start + 1..end).find(|&i| {
        lines[i]
            .split_once('=')
            .is_some_and(|(k, _)| k.trim() == key)
    });
    match found {
        Some(i) if overwrite => lines[i] = format!("{key}={value}"),
        Some(_) => {}
        None => {
            // After the section's last setting, not after its trailing blank line
            let mut at = end;
            while at > start + 1 && lines[at - 1].trim().is_empty() {
                at -= 1;
            }
            lines.insert(at, format!("{key}={value}"));
        }
    }
    lines.join("\n") + "\n"
}

/// Starts it on its own, outside AniBeam's process group, so it outlives
/// the app. Its output goes to a log in the profile. A task reaps it if it
/// exits while AniBeam still runs.
pub fn spawn(binary: &Path, profile: &Path, port: u16) -> std::io::Result<()> {
    std::fs::create_dir_all(profile)?;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file(profile))?;
    let mut cmd = tokio::process::Command::new(binary);
    cmd.arg(format!("--profile={}", profile.display()))
        .arg(format!("--webui-port={port}"))
        .arg("--confirm-legal-notice")
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .kill_on_drop(false);
    #[cfg(unix)]
    cmd.process_group(0);
    // CREATE_NEW_PROCESS_GROUP, so a console Ctrl+C never reaches it either.
    #[cfg(windows)]
    cmd.creation_flags(crate::process::NO_WINDOW | 0x0000_0200);
    let mut child = cmd.spawn()?;
    tokio::spawn(async move {
        let _ = child.wait().await;
    });
    Ok(())
}

/// The last lines of the profile's log, for a failure message.
pub fn log_tail(profile: &Path) -> String {
    let text = std::fs::read_to_string(log_file(profile)).unwrap_or_default();
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    format!(
        ": {}",
        lines[lines.len().saturating_sub(LOG_TAIL)..].join(" / ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_config_gets_every_section() {
        let dir = tempfile::tempdir().unwrap();
        write_config(
            dir.path(),
            47321,
            Some(Path::new("/home/u/Downloads/Anime")),
        )
        .unwrap();
        let text = std::fs::read_to_string(config_file(dir.path())).unwrap();
        assert!(text.contains("[LegalNotice]\nAccepted=true\n"));
        assert!(text.contains("WebUI\\Port=47321\n"));
        assert!(text.contains("WebUI\\LocalHostAuth=false\n"));
        assert!(text.contains(
            "[RSS]\nSession\\EnableProcessing=true\nAutoDownloader\\EnableProcessing=true\n"
        ));
        assert!(text.contains("Session\\DefaultSavePath=/home/u/Downloads/Anime\n"));
    }

    #[test]
    fn a_second_write_moves_the_port_and_keeps_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let file = config_file(dir.path());
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            "[BitTorrent]\nSession\\DefaultSavePath=/mnt/anime\nSession\\Port=51413\n\n\
             [Preferences]\nWebUI\\Port=8080\nGeneral\\Locale=en\n",
        )
        .unwrap();
        write_config(
            dir.path(),
            47321,
            Some(Path::new("/home/u/Downloads/Anime")),
        )
        .unwrap();
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("Session\\DefaultSavePath=/mnt/anime\nSession\\Port=51413\n"));
        assert!(!text.contains("WebUI\\Port=8080"));
        assert!(text.contains("WebUI\\Port=47321\n"));
        assert!(text.contains("General\\Locale=en\n"));
        assert_eq!(text.matches("[Preferences]").count(), 1);
    }

    #[test]
    fn set_ini_adds_a_key_before_the_blank_line_that_ends_a_section() {
        let text = "[A]\nx=1\n\n[B]\ny=2\n";
        assert_eq!(
            set_ini(text, "A", "z", "3", true),
            "[A]\nx=1\nz=3\n\n[B]\ny=2\n"
        );
        assert_eq!(set_ini(text, "B", "y", "9", false), text);
        assert_eq!(set_ini(text, "B", "y", "9", true), "[A]\nx=1\n\n[B]\ny=9\n");
        assert_eq!(set_ini("", "C", "k", "v", true), "[C]\nk=v\n");
    }

    #[test]
    fn the_port_an_earlier_setup_wrote_is_read_back() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(configured_port(dir.path()), None);
        write_config(dir.path(), 47321, None).unwrap();
        assert_eq!(configured_port(dir.path()), Some(47321));
    }

    #[test]
    fn which_takes_the_first_directory_that_has_it() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let file = if cfg!(windows) {
            format!("{NAME}.exe")
        } else {
            NAME.to_string()
        };
        std::fs::write(b.path().join(&file), "").unwrap();
        let path = std::env::join_paths([a.path(), b.path()]).unwrap();
        let path = path.to_string_lossy();
        assert_eq!(which(NAME, &path), Some(b.path().join(&file)));
        assert_eq!(which(NAME, &a.path().display().to_string()), None);
    }
}
