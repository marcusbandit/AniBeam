//! The downloads settings: one JSON blob in the settings table, the two
//! secrets in the secret store. Fresh defaults are borrowed from anirss's
//! own `config.toml` where it has one, so a machine that already runs
//! anirss points at the same qBittorrent without retyping anything.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::contract::*;
use crate::store::settings;
use crate::trackers::Secrets;

/// The settings-table key.
pub const KEY: &str = settings::DOWNLOADS;
/// The anirss bearer token, remote mode only.
pub const ANIRSS_TOKEN: &str = "anirss.token";
/// The torrent client's WebUI password: the key of the client a
/// single-client install had. Every other saved client's has its id after it.
pub const TORRENT_PASSWORD: &str = "qbittorrent.password";
/// The id of the client a blob from before the saved list is read as. It
/// keeps the password key it always had, so nothing in the store moves.
pub const LEGACY_CLIENT: &str = "default";

/// The secret-store key of a saved client's password.
pub fn password_key(id: &str) -> String {
    if id.is_empty() || id == LEGACY_CLIENT {
        TORRENT_PASSWORD.to_string()
    } else {
        format!("{TORRENT_PASSWORD}.{id}")
    }
}

/// The key of the password the client in use logs in with. AniBeam's own
/// needs none, so it has none.
pub fn active_password_key(s: &DownloadSettings) -> Option<String> {
    (s.torrent_mode != TorrentMode::Managed).then(|| password_key(&s.active_client))
}

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
    adopt_client(&mut s);
    s.has_anirss_token = false;
    s.has_torrent_password = false;
    for c in &mut s.saved_clients {
        c.has_password = false;
    }
    Ok(s)
}

pub fn load(conn: &Connection, secrets: &Secrets) -> Result<DownloadSettings, CoreError> {
    let mut s = load_stored(conn)?;
    fill_secret_flags(&mut s, |key| secrets.has(key));
    Ok(s)
}

/// Sets every `has_*` flag from `has`, which asks the secret store.
pub fn fill_secret_flags(s: &mut DownloadSettings, has: impl Fn(&str) -> bool) {
    s.has_anirss_token = has(ANIRSS_TOKEN);
    s.has_torrent_password = active_password_key(s).is_some_and(|k| has(&k));
    for c in &mut s.saved_clients {
        c.has_password = has(&password_key(&c.id));
    }
}

/// A blob from before the saved list, set up by hand with one client, reads
/// as that one client saved and in use. Auto and Managed have none to adopt.
fn adopt_client(s: &mut DownloadSettings) {
    if s.torrent_mode == TorrentMode::Existing && s.saved_clients.is_empty() {
        s.saved_clients.push(SavedTorrentClient {
            id: LEGACY_CLIENT.to_string(),
            name: name_for(&s.torrent_url),
            url: s.torrent_url.clone(),
            username: s.torrent_username.clone(),
            has_password: false,
        });
        s.active_client = LEGACY_CLIENT.to_string();
    }
}

pub fn save(conn: &Connection, s: &DownloadSettings) -> Result<(), CoreError> {
    settings::set(conn, KEY, s)
}

/// Trims every field and refuses what cannot work: an empty command, a
/// URL that is not http(s).
///
/// `retired` are the clients saved before this change. A new client never
/// gets one of their ids: the id names a password, and a removed client's
/// must not come back as a new client's.
pub fn validate(s: &mut DownloadSettings, retired: &[SavedTorrentClient]) -> Result<(), CoreError> {
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
    settle_clients(s, retired)
}

/// Makes the saved clients agree with the rest: each trimmed and checked,
/// an id for a new one, and in `Existing` mode the address and user in use
/// written into the active client, which is made from them when there is
/// none yet.
pub fn settle_clients(
    s: &mut DownloadSettings,
    retired: &[SavedTorrentClient],
) -> Result<(), CoreError> {
    for c in &mut s.saved_clients {
        c.id = c.id.trim().to_string();
        c.name = c.name.trim().to_string();
        c.url = c.url.trim().trim_end_matches('/').to_string();
        c.username = c.username.trim().to_string();
    }
    s.active_client = s.active_client.trim().to_string();
    for i in 0..s.saved_clients.len() {
        if s.saved_clients[i].id.is_empty() {
            s.saved_clients[i].id = next_id(&s.saved_clients, retired);
        }
    }
    if s.torrent_mode == TorrentMode::Existing {
        let at = match s.saved_clients.iter().position(|c| c.id == s.active_client) {
            Some(at) => at,
            None => {
                let id = if s.saved_clients.is_empty() {
                    LEGACY_CLIENT.to_string()
                } else {
                    next_id(&s.saved_clients, retired)
                };
                s.saved_clients.push(SavedTorrentClient {
                    id: id.clone(),
                    name: String::new(),
                    url: String::new(),
                    username: String::new(),
                    has_password: false,
                });
                s.active_client = id;
                s.saved_clients.len() - 1
            }
        };
        s.saved_clients[at].url = s.torrent_url.clone();
        s.saved_clients[at].username = s.torrent_username.clone();
    } else if !s.saved_clients.iter().any(|c| c.id == s.active_client) {
        s.active_client.clear();
    }
    let (mut ids, mut names) = (Vec::new(), Vec::new());
    for c in &mut s.saved_clients {
        if !c
            .id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        {
            return Err(CoreError::invalid(
                "saved_clients",
                format!(
                    "{:?} is not a client id: use letters, digits, - and _",
                    c.id
                ),
            ));
        }
        if ids.contains(&c.id) {
            return Err(CoreError::invalid(
                "saved_clients",
                format!("two clients have the id {}", c.id),
            ));
        }
        if !is_http(&c.url) {
            return Err(CoreError::invalid(
                "saved_clients",
                format!(
                    "the address of {} must start with http:// or https://",
                    c.name
                ),
            ));
        }
        if c.name.is_empty() {
            c.name = name_for(&c.url);
        }
        let name = c.name.to_lowercase();
        if names.contains(&name) {
            return Err(CoreError::invalid(
                "saved_clients",
                format!(
                    "two clients are called {}; give each a name of its own",
                    c.name
                ),
            ));
        }
        ids.push(c.id.clone());
        names.push(name);
    }
    Ok(())
}

/// An id no saved client has, nor a retired one: `c2`, `c3`, and so on.
fn next_id(clients: &[SavedTorrentClient], retired: &[SavedTorrentClient]) -> String {
    (2..)
        .map(|n| format!("c{n}"))
        .find(|id| clients.iter().chain(retired).all(|c| &c.id != id))
        .expect("an unused id")
}

/// Whether two addresses name the same server: scheme and host compared
/// without case, `localhost` and `::1` read as `127.0.0.1`, the scheme's
/// default port filled in, a trailing slash and any user info ignored.
pub fn same_address(a: &str, b: &str) -> bool {
    address_key(a) == address_key(b)
}

fn address_key(url: &str) -> String {
    let url = url.trim().trim_end_matches('/');
    let (scheme, rest) = url.split_once("://").unwrap_or(("http", url));
    let scheme = scheme.to_ascii_lowercase();
    let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let (host, port) = match host_port.strip_prefix('[') {
        Some(v6) => match v6.split_once(']') {
            Some((h, p)) => (h.to_string(), p.strip_prefix(':').unwrap_or("").to_string()),
            None => (v6.to_string(), String::new()),
        },
        None => match host_port.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.to_string()),
            None => (host_port.to_string(), String::new()),
        },
    };
    let host = match host.to_ascii_lowercase().as_str() {
        "localhost" | "::1" => "127.0.0.1".to_string(),
        h => h.to_string(),
    };
    let port = if port.is_empty() {
        if scheme == "https" { "443" } else { "80" }.to_string()
    } else {
        port
    };
    format!("{scheme}://{host}:{port}{path}")
}

/// Whether the client in use keeps its downloads on this machine's disk, so
/// the core may move them to the trash itself. AniBeam's own always does; a
/// client of the user's does when it is at a local address. A seedbox's paths
/// are its own, and the same path here is somebody else's folder.
pub fn files_here(s: &DownloadSettings) -> bool {
    s.torrent_mode == TorrentMode::Managed || is_loopback(&s.torrent_url)
}

/// Whether AniBeam's own may listen at the address in use: it has to be on
/// this machine, and not where one of the user's saved clients is, or it
/// would be started at the user's own qBittorrent's port.
pub fn fits_managed(s: &DownloadSettings) -> bool {
    is_loopback(&s.torrent_url)
        && !s
            .saved_clients
            .iter()
            .any(|c| same_address(&c.url, &s.torrent_url))
}

/// A name for a client that has none: its host and port.
pub fn name_for(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host = host.rsplit_once('@').map_or(host, |(_, h)| h);
    if host.is_empty() {
        "qBittorrent".to_string()
    } else {
        host.to_string()
    }
}

/// Whether `url` points at this machine.
pub fn is_loopback(url: &str) -> bool {
    let host = name_for(url);
    let host = match host.strip_prefix('[') {
        Some(v6) => v6.split(']').next().unwrap_or(v6).to_string(),
        None => host
            .rsplit_once(':')
            .map_or(host.as_str(), |(h, _)| h)
            .to_string(),
    };
    matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1")
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
        // Empty: each endpoint's own start in anirss's config is found when it is started
        jackett_command: String::new(),
        anirss_url: "http://127.0.0.1:8787".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_mode: TorrentMode::Auto,
        torrent_url: url,
        torrent_username: username,
        has_torrent_password: false,
        saved_clients: Vec::new(),
        active_client: String::new(),
        prune_finished_feeds: true,
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

    fn client(id: &str, name: &str, url: &str) -> SavedTorrentClient {
        SavedTorrentClient {
            id: id.to_string(),
            name: name.to_string(),
            url: url.to_string(),
            username: "admin".to_string(),
            has_password: false,
        }
    }

    #[test]
    fn addresses_match_however_they_are_spelled() {
        assert!(same_address(
            "http://localhost:8080/",
            "http://127.0.0.1:8080"
        ));
        assert!(same_address("HTTP://LocalHost:8080", "http://[::1]:8080"));
        assert!(same_address("http://box", "http://box:80"));
        assert!(same_address("https://box", "https://box:443"));
        assert!(same_address("http://me:pw@box:8080", "http://box:8080"));
        assert!(!same_address("http://box:8080", "https://box:8080"));
        assert!(!same_address("http://box:8080", "http://box:8081"));
        assert!(!same_address("http://box:8080/a", "http://box:8080"));
    }

    #[test]
    fn a_new_client_never_gets_a_retired_id() {
        let mut s = defaults();
        s.torrent_mode = TorrentMode::Managed;
        s.saved_clients = vec![
            client(LEGACY_CLIENT, "a", "http://192.0.2.1:1"),
            client("", "b", "http://192.0.2.2:1"),
        ];
        settle_clients(&mut s, &[client("c2", "gone", "http://192.0.2.3:1")]).unwrap();
        assert_eq!(s.saved_clients[1].id, "c3");
    }

    #[test]
    fn anibeams_own_does_not_fit_at_a_saved_clients_address() {
        let mut s = defaults();
        s.torrent_mode = TorrentMode::Managed;
        s.torrent_url = "http://127.0.0.1:8080".to_string();
        assert!(fits_managed(&s));
        s.saved_clients = vec![client(LEGACY_CLIENT, "mine", "http://localhost:8080")];
        assert!(!fits_managed(&s));
        s.torrent_url = "http://192.0.2.10:8080".to_string();
        s.saved_clients.clear();
        assert!(!fits_managed(&s));
    }

    #[test]
    fn the_first_client_keeps_the_old_password_key() {
        assert_eq!(password_key(""), TORRENT_PASSWORD);
        assert_eq!(password_key(LEGACY_CLIENT), TORRENT_PASSWORD);
        assert_eq!(password_key("c2"), "qbittorrent.password.c2");
        let mut s = defaults();
        s.torrent_mode = TorrentMode::Managed;
        assert_eq!(active_password_key(&s), None);
        s.torrent_mode = TorrentMode::Existing;
        s.active_client = "c2".to_string();
        assert_eq!(
            active_password_key(&s).as_deref(),
            Some("qbittorrent.password.c2")
        );
    }

    #[test]
    fn a_blob_from_before_the_list_reads_as_one_saved_client() {
        let mut s = defaults();
        s.torrent_mode = TorrentMode::Existing;
        s.torrent_url = "http://192.0.2.10:8080".to_string();
        adopt_client(&mut s);
        assert_eq!(s.active_client, LEGACY_CLIENT);
        assert_eq!(
            s.saved_clients,
            vec![SavedTorrentClient {
                username: s.torrent_username.clone(),
                ..client(LEGACY_CLIENT, "192.0.2.10:8080", "http://192.0.2.10:8080")
            }]
        );
        // Auto and Managed have no client of the user's to adopt
        let mut a = defaults();
        adopt_client(&mut a);
        assert!(a.saved_clients.is_empty() && a.active_client.is_empty());
    }

    #[test]
    fn settling_writes_the_client_in_use_and_names_new_ones() {
        let mut s = defaults();
        s.torrent_mode = TorrentMode::Existing;
        s.torrent_url = "http://localhost:8080".to_string();
        s.saved_clients = vec![
            client("default", "Seedbox", "http://192.0.2.10:8080/"),
            client("", "", "http://nas:9000"),
        ];
        s.active_client = "default".to_string();
        settle_clients(&mut s, &[]).unwrap();
        assert_eq!(s.saved_clients[0].url, "http://localhost:8080");
        assert_eq!(
            (
                s.saved_clients[1].id.as_str(),
                s.saved_clients[1].name.as_str()
            ),
            ("c2", "nas:9000")
        );
        // An active id the list lacks is made from the address in use
        s.active_client = "gone".to_string();
        settle_clients(&mut s, &[]).unwrap();
        assert_eq!(s.saved_clients.len(), 3);
        assert_eq!(s.active_client, "c3");
        // Off the list in another mode, the active id is forgotten
        s.torrent_mode = TorrentMode::Managed;
        s.saved_clients.truncate(2);
        settle_clients(&mut s, &[]).unwrap();
        assert!(s.active_client.is_empty());
    }

    #[test]
    fn settling_refuses_what_cannot_be_told_apart_or_reached() {
        let refused = |clients: Vec<SavedTorrentClient>| {
            let mut s = defaults();
            s.saved_clients = clients;
            matches!(settle_clients(&mut s, &[]), Err(CoreError::Invalid { field, .. }) if field == "saved_clients")
        };
        assert!(refused(vec![
            client("a", "Box", "http://a"),
            client("b", "box", "http://b")
        ]));
        assert!(refused(vec![
            client("a", "A", "http://a"),
            client("a", "B", "http://b")
        ]));
        assert!(refused(vec![client("a b", "A", "http://a")]));
        assert!(refused(vec![client("../x", "A", "http://a")]));
        assert!(refused(vec![client("a", "A", "localhost:8080")]));
    }

    #[test]
    fn names_and_loopback_come_off_the_address() {
        assert_eq!(name_for("http://192.0.2.10:8080/"), "192.0.2.10:8080");
        assert_eq!(name_for("https://me:pw@box.example/qbt"), "box.example");
        assert!(is_loopback("http://127.0.0.1:47321"));
        assert!(is_loopback("http://localhost:8080"));
        assert!(is_loopback("http://[::1]:8080"));
        assert!(!is_loopback("http://192.0.2.10:8080"));
        assert!(!is_loopback("http://localhost.example:8080"));
    }

    #[test]
    fn validate_trims_and_refuses_a_non_http_address() {
        let mut s = defaults();
        s.torrent_url = " http://localhost:9090/ ".to_string();
        s.torrent_username = " me ".to_string();
        validate(&mut s, &[]).unwrap();
        assert_eq!(s.torrent_url, "http://localhost:9090");
        assert_eq!(s.torrent_username, "me");
        s.torrent_url = "localhost:9090".to_string();
        assert!(
            matches!(validate(&mut s, &[]), Err(CoreError::Invalid { field, .. }) if field == "torrent_url")
        );
        let mut r = defaults();
        r.anirss_mode = AnirssMode::Remote;
        r.anirss_url = "ftp://x".to_string();
        assert!(
            matches!(validate(&mut r, &[]), Err(CoreError::Invalid { field, .. }) if field == "anirss_url")
        );
        let mut c = defaults();
        c.anirss_command = "  ".to_string();
        assert!(
            matches!(validate(&mut c, &[]), Err(CoreError::Invalid { field, .. }) if field == "anirss_command")
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
