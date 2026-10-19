//! The downloads settings: one JSON blob in the settings table, the two
//! secrets in the secret store. Fresh defaults are borrowed from anirss's
//! own `config.toml` where it has one, so a machine that already runs
//! anirss points at the same qBittorrent without retyping anything.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::jackett;
use crate::contract::*;
use crate::store::settings;
use crate::trackers::Secrets;

/// The settings-table key.
pub const KEY: &str = settings::DOWNLOADS;
/// The anirss bearer token, remote mode only.
pub const ANIRSS_TOKEN: &str = "anirss.token";
/// The torrent client's WebUI password.
pub const TORRENT_PASSWORD: &str = "qbittorrent.password";

/// The stored blob, or the defaults; the two `has_*` flags are false here
/// and set by `load`, which asks the secret store.
pub fn load_stored(conn: &Connection) -> Result<DownloadSettings, CoreError> {
    let mut s: DownloadSettings = match settings::get(conn, KEY) {
        Ok(Some(s)) => s,
        Ok(None) => defaults(),
        Err(e) if e.is_decode_failure() => {
            tracing::warn!("reading the downloads settings as the defaults: {e}");
            defaults()
        }
        Err(e) => return Err(e),
    };
    s.has_anirss_token = false;
    s.has_torrent_password = false;
    Ok(s)
}

pub fn load(conn: &Connection, secrets: &Secrets) -> Result<DownloadSettings, CoreError> {
    let mut s = load_stored(conn)?;
    s.has_anirss_token = secrets.has(ANIRSS_TOKEN);
    s.has_torrent_password = secrets.has(TORRENT_PASSWORD);
    Ok(s)
}

pub fn save(conn: &Connection, s: &DownloadSettings) -> Result<(), CoreError> {
    settings::set(conn, KEY, s)
}

/// Trims every field and refuses what cannot work: an empty command, a
/// URL that is not http(s).
pub fn validate(s: &mut DownloadSettings) -> Result<(), CoreError> {
    s.anirss_command = s.anirss_command.trim().to_string();
    s.anirss_url = s.anirss_url.trim().trim_end_matches('/').to_string();
    s.torrent_url = s.torrent_url.trim().trim_end_matches('/').to_string();
    s.torrent_username = s.torrent_username.trim().to_string();
    s.jackett_command = s.jackett_command.trim().to_string();
    if s.anirss_mode == AnirssMode::Sidecar && s.anirss_command.is_empty() {
        return Err(CoreError::invalid(
            "anirss_command",
            "the anirss command cannot be empty",
        ));
    }
    if s.anirss_mode == AnirssMode::Remote && !is_http(&s.anirss_url) {
        return Err(CoreError::invalid(
            "anirss_url",
            "the anirss address must start with http:// or https://",
        ));
    }
    if !is_http(&s.torrent_url) {
        return Err(CoreError::invalid(
            "torrent_url",
            format!(
                "the {} address must start with http:// or https://",
                s.torrent_client.as_str()
            ),
        ));
    }
    Ok(())
}

fn is_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Fresh settings: a sidecar off `anirss` on PATH, and qBittorrent where
/// anirss's config says it is, else qBittorrent's own stock address.
pub fn defaults() -> DownloadSettings {
    let (url, username) = anirss_config_path()
        .and_then(|p| anirss_config_qbt(&p))
        .unwrap_or_else(|| ("http://localhost:8080".to_string(), "admin".to_string()));
    DownloadSettings {
        anirss_mode: AnirssMode::Sidecar,
        anirss_command: "anirss".to_string(),
        jackett_command: anirss_config_path()
            .and_then(|p| jackett::anirss_start(&p, None))
            .unwrap_or_default(),
        anirss_url: "http://127.0.0.1:8787".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_url: url,
        torrent_username: username,
        has_torrent_password: false,
    }
}

/// `[qbittorrent] url` and `username` out of an anirss `config.toml`.
/// `None` when the file is missing, unreadable or has no `url`.
pub fn anirss_config_qbt(path: &Path) -> Option<(String, String)> {
    let text = std::fs::read_to_string(path).ok()?;
    let doc: toml::Table = toml::from_str(&text).ok()?;
    let qbt = doc.get("qbittorrent")?;
    let url = qbt.get("url")?.as_str()?.trim().trim_end_matches('/');
    if url.is_empty() {
        return None;
    }
    let username = qbt
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("admin")
        .trim();
    Some((url.to_string(), username.to_string()))
}

/// `~/.config/anirss/config.toml`, honouring `XDG_CONFIG_HOME`.
pub fn anirss_config_path() -> Option<PathBuf> {
    let base = match std::env::var("XDG_CONFIG_HOME") {
        Ok(d) if !d.is_empty() => PathBuf::from(d),
        _ => PathBuf::from(std::env::var("HOME").ok()?).join(".config"),
    };
    Some(base.join("anirss").join("config.toml"))
}

/// The qBittorrent password anirss's CLI saved (`qbt.pass` in its state
/// dir, mode 600), so a machine that already logs in through anirss works
/// before a password is saved here. `None` when there is none.
pub fn anirss_saved_password() -> Option<String> {
    let base = match std::env::var("XDG_STATE_HOME") {
        Ok(d) if !d.is_empty() => PathBuf::from(d),
        _ => PathBuf::from(std::env::var("HOME").ok()?)
            .join(".local")
            .join("state"),
    };
    let text = std::fs::read_to_string(base.join("anirss").join("qbt.pass")).ok()?;
    let pw = text.trim_end_matches(['\n', '\r']).to_string();
    (!pw.is_empty()).then_some(pw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_qbittorrent_off_an_anirss_config() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.toml");
        std::fs::write(
            &p,
            "[qbittorrent]\nurl = \"http://localhost:9090/\"\nusername = \"bandit\"\n\n[[endpoint]]\nname = \"nyaa\"\n",
        )
        .unwrap();
        assert_eq!(
            anirss_config_qbt(&p),
            Some(("http://localhost:9090".to_string(), "bandit".to_string()))
        );
        assert_eq!(anirss_config_qbt(&dir.path().join("missing.toml")), None);
    }

    #[test]
    fn validate_trims_and_refuses_a_non_http_address() {
        let mut s = defaults();
        s.torrent_url = " http://localhost:9090/ ".to_string();
        s.torrent_username = " me ".to_string();
        validate(&mut s).unwrap();
        assert_eq!(s.torrent_url, "http://localhost:9090");
        assert_eq!(s.torrent_username, "me");
        s.torrent_url = "localhost:9090".to_string();
        assert!(
            matches!(validate(&mut s), Err(CoreError::Invalid { field, .. }) if field == "torrent_url")
        );
        let mut r = defaults();
        r.anirss_mode = AnirssMode::Remote;
        r.anirss_url = "ftp://x".to_string();
        assert!(
            matches!(validate(&mut r), Err(CoreError::Invalid { field, .. }) if field == "anirss_url")
        );
        let mut c = defaults();
        c.anirss_command = "  ".to_string();
        assert!(
            matches!(validate(&mut c), Err(CoreError::Invalid { field, .. }) if field == "anirss_command")
        );
    }
}

/// An endpoint anirss starts on demand (it has a `start` command): the URL
/// that answers once it is up, and how long anirss waits for that.
#[derive(Clone, Debug, PartialEq)]
pub struct OnDemand {
    pub ready: String,
    pub timeout: u64,
}

/// The named endpoint's on-demand settings from anirss's own config, or
/// None for an endpoint that is always there (or a config that cannot be
/// read: then nothing is known and nothing is claimed).
pub fn anirss_ondemand(path: &Path, name: &str) -> Option<OnDemand> {
    let text = std::fs::read_to_string(path).ok()?;
    let doc: toml::Table = toml::from_str(&text).ok()?;
    let endpoint = doc
        .get("endpoint")?
        .as_array()?
        .iter()
        .filter_map(|e| e.as_table())
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some(name))?;
    endpoint
        .get("start")?
        .as_str()
        .filter(|s| !s.trim().is_empty())?;
    let timeout = endpoint
        .get("timeout")
        .and_then(|v| v.as_integer().or_else(|| v.as_float().map(|f| f as i64)))
        .filter(|t| *t > 0)
        .unwrap_or(60) as u64;
    Some(OnDemand {
        ready: endpoint
            .get("ready")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string(),
        timeout,
    })
}

#[cfg(test)]
mod ondemand_tests {
    use super::*;

    #[test]
    fn an_endpoint_with_a_start_command_is_on_demand() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
[[endpoint]]
name = "nyaa"
kind = "nyaa"
url = "https://nyaa.si/"

[[endpoint]]
name = "1337x"
kind = "rss"
url = "http://127.0.0.1:9117/x?q={query}"
start = "podman pod start x"
ready = "http://127.0.0.1:9117/caps"
timeout = 45
"#,
        )
        .unwrap();
        assert_eq!(anirss_ondemand(&path, "nyaa"), None);
        assert_eq!(
            anirss_ondemand(&path, "1337x"),
            Some(OnDemand {
                ready: "http://127.0.0.1:9117/caps".to_string(),
                timeout: 45
            })
        );
        assert_eq!(anirss_ondemand(&path, "erai"), None);
    }
}
