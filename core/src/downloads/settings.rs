//! The downloads settings: one JSON blob in the settings table, the two
//! secrets in the secret store. Fresh defaults are borrowed from anirss's
//! own `config.toml` where it has one, so a machine that already runs
//! anirss points at the same qBittorrent without retyping anything.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use rusqlite::Connection;
use serde::Deserialize;

use super::home::AnirssHome;
use crate::Core;
use crate::contract::*;
use crate::releases::BestfitConfig;
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
pub fn load_stored(conn: &Connection, home: &AnirssHome) -> Result<DownloadSettings, CoreError> {
    let mut s: DownloadSettings = match settings::get(conn, KEY) {
        Ok(Some(s)) => s,
        Ok(None) => fresh(home),
        Err(e) if e.is_decode_failure() => {
            tracing::warn!("reading the downloads settings as the defaults: {e}");
            // A stored anirss outlives a blob this build cannot read: the defaults
            // would move the person onto the native engine without a word (#552).
            let mut d = fresh(home);
            if stored_engine_value(conn) == Some(DownloadsEngine::Anirss) {
                d.engine = DownloadsEngine::Anirss;
            }
            d
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

/// `load_stored` for `core`, off the home its paths give anirss.
pub fn read(core: &Core) -> Result<DownloadSettings, CoreError> {
    let home = AnirssHome::of(&core.paths);
    core.store.read(move |c| load_stored(c, &home))
}

pub fn load(
    conn: &Connection,
    secrets: &Secrets,
    home: &AnirssHome,
) -> Result<DownloadSettings, CoreError> {
    let mut s = load_stored(conn, home)?;
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

/// Writes the settings over what is stored, so a key a newer build added survives a save
/// here. An engine this build does not know reads as the native engine, the default (a newer
/// build wrote it), so a save that still says native keeps the stored value rather than undo
/// the newer build's choice; picking anirss replaces it. This build cannot tell a native
/// engine read that way from one picked on purpose, and in this build both mean native.
pub fn save(conn: &Connection, s: &DownloadSettings) -> Result<(), CoreError> {
    let serde_json::Value::Object(fields) = serde_json::to_value(s)? else {
        return Err(CoreError::internal(
            "the downloads settings are not an object",
        ));
    };
    let mut blob = match settings::get::<serde_json::Value>(conn, KEY) {
        Ok(Some(serde_json::Value::Object(stored))) => stored,
        Ok(_) => serde_json::Map::new(),
        Err(e) if e.is_decode_failure() => serde_json::Map::new(),
        Err(e) => return Err(e),
    };
    let unknown = blob
        .get("engine")
        .is_some_and(|e| !e.is_null() && DownloadsEngine::deserialize(e).is_err());
    let keep = unknown && s.engine == DownloadsEngine::default();
    for (k, v) in fields {
        if !(keep && k == "engine") {
            blob.insert(k, v);
        }
    }
    settings::set(conn, KEY, &blob)
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
            crate::i18n::tr("core-downloads-settings-command-empty", &[]),
        ));
    }
    if s.anirss_mode == AnirssMode::Remote && !is_http(&s.anirss_url) {
        return Err(CoreError::invalid(
            "anirss_url",
            crate::i18n::tr("core-downloads-settings-anirss-address", &[]),
        ));
    }
    if !is_http(&s.torrent_url) {
        return Err(CoreError::invalid(
            "torrent_url",
            crate::i18n::tr(
                "core-downloads-settings-client-address",
                &[("client", s.torrent_client.as_str().into())],
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
                // The legacy id carries a saved password, so it is only
                // reused for the server that password belongs to.
                let id = if s.saved_clients.is_empty()
                    && retired
                        .iter()
                        .filter(|c| c.id == LEGACY_CLIENT)
                        .all(|c| same_address(&c.url, &s.torrent_url))
                {
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
                crate::i18n::tr(
                    "core-downloads-settings-bad-id",
                    &[("id", format!("{:?}", c.id).into())],
                ),
            ));
        }
        if ids.contains(&c.id) {
            return Err(CoreError::invalid(
                "saved_clients",
                crate::i18n::tr(
                    "core-downloads-settings-same-id",
                    &[("id", c.id.as_str().into())],
                ),
            ));
        }
        if !is_http(&c.url) {
            return Err(CoreError::invalid(
                "saved_clients",
                crate::i18n::tr(
                    "core-downloads-settings-saved-address",
                    &[("name", c.name.as_str().into())],
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
                crate::i18n::tr(
                    "core-downloads-settings-same-name",
                    &[("name", c.name.as_str().into())],
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
    let url = url.trim();
    let with_scheme;
    let url = if url.contains("://") {
        url
    } else {
        with_scheme = format!("http://{url}");
        &with_scheme
    };
    let Some(u) = parsed(url) else {
        return url.trim_end_matches('/').to_string();
    };
    let host = match bare_host(&u).as_str() {
        "localhost" | "::1" => "127.0.0.1".to_string(),
        h => h.to_string(),
    };
    let port = u.port_or_known_default().unwrap_or(0);
    let path = u.path().trim_end_matches('/');
    let query = u.query().map_or(String::new(), |q| format!("?{q}"));
    format!("{}://{host}:{port}{path}{query}", u.scheme())
}

/// Whether the client in use keeps its downloads on this machine's disk, so
/// the core may move them to the trash itself. AniBeam's own always does; a
/// client of the user's does when its address is this machine: loopback,
/// one of the machine's own addresses, or its name. A seedbox's paths are
/// its own, and the same path here is somebody else's folder.
pub fn files_here(s: &DownloadSettings) -> bool {
    s.torrent_mode == TorrentMode::Managed
        || is_loopback(&s.torrent_url)
        || is_this_machine(&host_of(&s.torrent_url))
}

fn is_this_machine(host: &str) -> bool {
    names_this_machine(
        host,
        &crate::foundation::machine::hostname(),
        crate::foundation::machine::local_addresses,
        |name| remembered(&LOOKUPS, name, Instant::now(), bounded_lookup),
    )
}

/// How long a looked-up answer stands before the name is looked up again. `files_here`
/// runs on every Downloads poll, and the answer only changes with the settings or the
/// network.
const LOOKUP_FOR: Duration = Duration::from_secs(60);
/// The longest a lookup may hold up its caller, the shell's own thread on the Download
/// path.
const LOOKUP_LIMIT: Duration = Duration::from_secs(2);

type Lookups = Mutex<HashMap<String, (Instant, Vec<IpAddr>)>>;
static LOOKUPS: LazyLock<Lookups> = LazyLock::new(Lookups::default);

/// The addresses `name` resolves to, looked up at most once a `LOOKUP_FOR`. A lookup that
/// fails keeps the last answer, so one DNS hiccup never moves where the files are; None
/// when no lookup has answered yet.
fn remembered(
    cache: &Lookups,
    name: &str,
    now: Instant,
    lookup: impl FnOnce(&str) -> Option<Vec<IpAddr>>,
) -> Option<Vec<IpAddr>> {
    let last = cache.lock().ok()?.get(name).cloned();
    if let Some((at, addrs)) = &last
        && now.duration_since(*at) < LOOKUP_FOR
    {
        return Some(addrs.clone());
    }
    match lookup(name) {
        Some(addrs) => {
            if let Ok(mut cache) = cache.lock() {
                cache.insert(name.to_string(), (now, addrs.clone()));
            }
            Some(addrs)
        }
        None => last.map(|(_, addrs)| addrs),
    }
}

/// The system's lookup of `name`, given `LOOKUP_LIMIT` at most. None when it fails, finds
/// nothing or runs out of time; a lookup that runs out finishes on its own thread.
fn bounded_lookup(name: &str) -> Option<Vec<IpAddr>> {
    use std::net::ToSocketAddrs;
    let (tx, rx) = std::sync::mpsc::channel();
    let owned = name.to_string();
    std::thread::Builder::new()
        .name("qbt-lookup".into())
        .spawn(move || {
            let found = (owned.as_str(), 0)
                .to_socket_addrs()
                .map(|all| all.map(|a| a.ip()).collect::<Vec<_>>());
            let _ = tx.send(found);
        })
        .ok()?;
    match rx.recv_timeout(LOOKUP_LIMIT) {
        Ok(Ok(addrs)) if !addrs.is_empty() => Some(addrs),
        _ => None,
    }
}

/// Whether `host` is this machine, named `hostname` and holding `locals`. An address
/// counts when it is loopback, unspecified or one of `locals`, and a name when it is the
/// bare hostname. A longer name that only starts with it, a tailnet's
/// `banditbox.tail1234.ts.net` or a lookalike `banditbox.example.net`, counts only when it
/// resolves here, since the core trashes files at the paths that client reports (#446).
/// Only such a name is looked up, and `locals` is listed only when an address needs it.
fn names_this_machine(
    host: &str,
    hostname: &str,
    locals: impl Fn() -> Vec<IpAddr>,
    resolve: impl Fn(&str) -> Option<Vec<IpAddr>>,
) -> bool {
    let here = |ips: &[IpAddr]| {
        let locals = locals();
        ips.iter().any(|ip| {
            let ip = ip.to_canonical();
            ip.is_loopback() || ip.is_unspecified() || locals.contains(&ip)
        })
    };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if let Ok(ip) = host.parse::<IpAddr>() {
        return here(&[ip]);
    }
    let name = hostname.trim_end_matches('.').to_ascii_lowercase();
    let first = |n: &str| n.split('.').next().unwrap_or_default().to_string();
    if name.is_empty() || host.is_empty() || first(&host) != first(&name) {
        return false;
    }
    if host == name || !host.contains('.') {
        return true;
    }
    resolve(&host).is_some_and(|ips| here(&ips))
}

/// An address as the HTTP client reads it, so every check here agrees with
/// the machine the client really talks to: a backslash ends the host, case
/// and short forms like `127.1` are settled the same way (#401).
fn parsed(url: &str) -> Option<reqwest::Url> {
    reqwest::Url::parse(url.trim())
        .ok()
        .filter(|u| u.host_str().is_some_and(|h| !h.is_empty()))
}

/// A parsed address's host, without IPv6 brackets.
fn bare_host(u: &reqwest::Url) -> String {
    let host = u.host_str().unwrap_or_default();
    host.strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(host)
        .to_string()
}

/// The host part of a URL, lower case, without port or IPv6 brackets.
fn host_of(url: &str) -> String {
    parsed(url).map_or(String::new(), |u| bare_host(&u))
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

/// A name for a client that has none: its host, and its port when it is not
/// the scheme's own.
pub fn name_for(url: &str) -> String {
    let Some(u) = parsed(url) else {
        return "qBittorrent".to_string();
    };
    let host = u.host_str().unwrap_or_default();
    match u.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_string(),
    }
}

/// Whether `url` points at this machine.
pub fn is_loopback(url: &str) -> bool {
    matches!(host_of(url).as_str(), "127.0.0.1" | "localhost" | "::1")
}

fn is_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// The engine the stored blob names, read on its own, so a blob the rest of
/// which this build cannot decode still says which engine it picked.
fn stored_engine_value(conn: &Connection) -> Option<DownloadsEngine> {
    let blob = settings::get::<serde_json::Value>(conn, KEY).ok()??;
    serde_json::from_value(blob.get("engine")?.clone()).ok()
}

/// How many endpoints anirss's own `config.toml` names, nought when there is
/// no file or none it can read. AniBeam itself does not read them (#552).
pub fn anirss_config_endpoints(path: &Path) -> usize {
    let Ok(text) = std::fs::read_to_string(path) else {
        return 0;
    };
    let Ok(doc) = toml::from_str::<toml::Table>(&text) else {
        return 0;
    };
    doc.get("endpoint")
        .and_then(|e| e.as_array())
        .map_or(0, |a| a.len())
}

/// Fresh settings: [`defaults`], with qBittorrent where the config of the
/// anirss in `home` says it is.
pub fn fresh(home: &AnirssHome) -> DownloadSettings {
    let mut s = defaults();
    if let Some((url, username)) = home.config_path().and_then(|p| anirss_config_qbt(&p)) {
        s.torrent_url = url;
        s.torrent_username = username;
    }
    s
}

/// The stock settings: the native engine (#552), with the search helper's own
/// sidecar off `anirss` on PATH ready for whoever picks it, and qBittorrent at
/// its own stock address.
pub fn defaults() -> DownloadSettings {
    let (url, username) = ("http://localhost:8080".to_string(), "admin".to_string());
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
        rolling: RollingSettings::default(),
        engine: DownloadsEngine::Native,
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

/// The config of the anirss `core` runs: in the person's own home, or in a
/// sandboxed run's own (`AnirssHome`).
pub fn anirss_config_path(core: &Core) -> Option<PathBuf> {
    AnirssHome::of(&core.paths).config_path()
}

/// The qBittorrent password anirss's CLI saved (`qbt.pass` in its state
/// dir, mode 600), so a machine that already logs in through anirss works
/// before a password is saved here. `None` when there is none.
pub fn anirss_saved_password(home: &AnirssHome) -> Option<String> {
    let text = std::fs::read_to_string(home.state_dir()?.join("qbt.pass")).ok()?;
    let pw = text.trim_end_matches(['\n', '\r']).to_string();
    (!pw.is_empty()).then_some(pw)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::franchise::testing::conn as store;

    /// A home with nothing in it, so no test reads the machine's own anirss.
    fn nowhere() -> AnirssHome {
        AnirssHome::Sandbox(PathBuf::from("/nonexistent/anibeam-test-home"))
    }

    fn stored_engine(c: &Connection) -> serde_json::Value {
        settings::get::<serde_json::Value>(c, KEY).unwrap().unwrap()["engine"].clone()
    }

    #[test]
    fn a_save_keeps_an_engine_this_build_does_not_know() {
        let c = store();
        let mut blob = serde_json::to_value(defaults()).unwrap();
        blob["engine"] = serde_json::json!("Telepathy");
        settings::set(&c, KEY, &blob).unwrap();
        // Read as the default, native, here, and saved back with something else changed.
        let mut s = load_stored(&c, &nowhere()).unwrap();
        assert_eq!(s.engine, DownloadsEngine::Native);
        s.prune_finished_feeds = false;
        save(&c, &s).unwrap();
        assert_eq!(stored_engine(&c), serde_json::json!("Telepathy"));
        assert!(!load_stored(&c, &nowhere()).unwrap().prune_finished_feeds);
        // Picking another engine this build knows replaces it.
        s.engine = DownloadsEngine::Anirss;
        save(&c, &s).unwrap();
        assert_eq!(stored_engine(&c), serde_json::json!("Anirss"));
    }

    #[test]
    fn a_save_keeps_keys_a_newer_build_added() {
        let c = store();
        let mut blob = serde_json::to_value(defaults()).unwrap();
        blob["from_the_future"] = serde_json::json!({"on": true});
        settings::set(&c, KEY, &blob).unwrap();
        save(&c, &load_stored(&c, &nowhere()).unwrap()).unwrap();
        let stored = settings::get::<serde_json::Value>(&c, KEY)
            .unwrap()
            .unwrap();
        assert_eq!(stored["from_the_future"], serde_json::json!({"on": true}));
    }

    #[test]
    fn a_missing_null_or_shaped_engine_is_not_kept_as_unknown() {
        for engine in [
            None,
            Some(serde_json::Value::Null),
            Some(serde_json::json!({"Native": {}})),
        ] {
            let c = store();
            let mut blob = serde_json::to_value(defaults()).unwrap();
            match engine.clone() {
                Some(e) => blob["engine"] = e,
                None => {
                    blob.as_object_mut().unwrap().remove("engine");
                }
            }
            settings::set(&c, KEY, &blob).unwrap();
            save(&c, &load_stored(&c, &nowhere()).unwrap()).unwrap();
            let kept = stored_engine(&c);
            match engine {
                // A shape this build cannot read is unknown too, and kept.
                Some(e) if e.is_object() => assert_eq!(kept, e),
                _ => assert_eq!(kept, serde_json::json!("Native")),
            }
        }
    }

    /// A blob this build cannot decode still keeps a stored anirss (#552).
    #[test]
    fn an_unreadable_blob_keeps_its_stored_anirss() {
        let c = store();
        let mut blob = serde_json::to_value(defaults()).unwrap();
        blob["engine"] = serde_json::json!("Anirss");
        blob["torrent_mode"] = serde_json::json!("Telepathic");
        settings::set(&c, KEY, &blob).unwrap();
        assert_eq!(
            load_stored(&c, &nowhere()).unwrap().engine,
            DownloadsEngine::Anirss
        );
        blob["engine"] = serde_json::json!("Native");
        settings::set(&c, KEY, &blob).unwrap();
        assert_eq!(
            load_stored(&c, &nowhere()).unwrap().engine,
            DownloadsEngine::Native
        );
    }

    #[test]
    fn anirss_endpoints_are_counted_off_its_config() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.toml");
        assert_eq!(anirss_config_endpoints(&p), 0);
        std::fs::write(
            &p,
            "[[endpoint]]\nname = \"a\"\n\n[[endpoint]]\nname = \"b\"\n",
        )
        .unwrap();
        assert_eq!(anirss_config_endpoints(&p), 2);
    }

    #[test]
    fn reads_qbittorrent_off_an_anirss_config() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.toml");
        std::fs::write(
            &p,
            "[qbittorrent]\nurl = \"http://localhost:9090/\"\nusername = \"bandit\"\n\n[[endpoint]]\nname = \"tracker\"\n",
        )
        .unwrap();
        assert_eq!(
            anirss_config_qbt(&p),
            Some(("http://localhost:9090".to_string(), "bandit".to_string()))
        );
        assert_eq!(anirss_config_qbt(&dir.path().join("missing.toml")), None);
    }

    #[test]
    fn only_a_name_that_resolves_here_is_this_machine() {
        let locals = || -> Vec<IpAddr> {
            vec![
                "192.168.1.20".parse().unwrap(),
                "100.65.105.9".parse().unwrap(),
            ]
        };
        let asked = std::cell::RefCell::new(Vec::new());
        let dns = |name: &str| -> Option<Vec<IpAddr>> {
            asked.borrow_mut().push(name.to_string());
            match name {
                "banditbox.tail1234.ts.net" => Some(vec!["100.65.105.9".parse().unwrap()]),
                "banditbox.example.net" => Some(vec!["203.0.113.7".parse().unwrap()]),
                _ => None,
            }
        };
        let here = |host: &str| names_this_machine(host, "banditbox", locals, dns);
        assert!(here("banditbox"));
        assert!(here("BanditBox."));
        assert!(here("banditbox.tail1234.ts.net"));
        assert!(here("BANDITBOX.TAIL1234.TS.NET."));
        assert!(here("192.168.1.20"));
        assert!(here("127.0.0.1") && here("::1") && here("::ffff:127.0.0.1") && here("0.0.0.0"));
        // A lookalike whose first label is this machine's name is somebody
        // else's box, and so is a name that resolves nowhere (#446).
        assert!(!here("banditbox.example.net"));
        assert!(!here("banditbox.nowhere.invalid"));
        assert!(!here("seedbox.example.net"));
        assert!(!here("203.0.113.7"));
        // Only a name that starts like this machine's is looked up.
        assert!(
            asked.borrow().iter().all(|n| n.starts_with("banditbox.")),
            "{:?}",
            asked.borrow()
        );
        // No hostname, no match; a dotted hostname still knows its bare name.
        assert!(!names_this_machine("banditbox", "", locals, dns));
        assert!(names_this_machine(
            "banditbox",
            "banditbox.local",
            locals,
            dns
        ));
        assert!(names_this_machine(
            "banditbox.local",
            "banditbox.local",
            locals,
            dns
        ));
    }

    #[test]
    fn a_failed_lookup_keeps_the_last_answer() {
        let cache = Lookups::default();
        let at = Instant::now();
        let tailnet = vec!["100.65.105.9".parse::<IpAddr>().unwrap()];
        let found = |_: &str| Some(vec!["100.65.105.9".parse().unwrap()]);
        let fails = |_: &str| None;
        let never = |_: &str| -> Option<Vec<IpAddr>> { panic!("looked up within LOOKUP_FOR") };
        let name = "banditbox.tail1234.ts.net";
        assert_eq!(remembered(&cache, "other.example", at, fails), None);
        assert_eq!(remembered(&cache, name, at, found), Some(tailnet.clone()));
        assert_eq!(
            remembered(&cache, name, at + LOOKUP_FOR / 2, never),
            Some(tailnet.clone())
        );
        // Past LOOKUP_FOR it asks again, and a lookup that fails keeps what was known.
        assert_eq!(
            remembered(&cache, name, at + LOOKUP_FOR * 2, fails),
            Some(tailnet)
        );
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
    fn a_new_server_does_not_inherit_the_retired_default_and_its_password() {
        let mut s = defaults();
        s.torrent_mode = TorrentMode::Existing;
        s.torrent_url = "http://192.0.2.9:1".to_string();
        let gone = [client(LEGACY_CLIENT, "old", "http://192.0.2.1:1")];
        settle_clients(&mut s, &gone).unwrap();
        assert_ne!(s.active_client, LEGACY_CLIENT);

        let mut same = defaults();
        same.torrent_mode = TorrentMode::Existing;
        same.torrent_url = "http://192.0.2.1:1".to_string();
        settle_clients(&mut same, &gone).unwrap();
        assert_eq!(same.active_client, LEGACY_CLIENT);
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
    fn a_backslash_ends_the_host_as_the_http_client_reads_it() {
        // The client talks to 192.0.2.1 here, so nothing may read it as this
        // machine or as the anirss config's address (#401).
        let url = "http://192.0.2.1\\@127.0.0.1:8080";
        assert_eq!(name_for(url), "192.0.2.1");
        assert!(!is_loopback(url));
        assert!(!same_address(url, "http://127.0.0.1:8080"));
        let mut s = defaults();
        s.torrent_mode = TorrentMode::Existing;
        s.torrent_url = url.to_string();
        assert!(!files_here(&s));
        assert!(!fits_managed(&s));
        // A login before the host still comes off.
        assert_eq!(name_for("http://me:pw@box:8080/qbt"), "box:8080");
        assert!(same_address("http://me:pw@box:8080", "http://box:8080/"));
        // Case, short forms and a trailing backslash read as the client reads them.
        assert!(is_loopback("http://LOCALHOST:8080"));
        assert!(is_loopback("http://127.1:8080"));
        assert!(same_address(
            "http://127.0.0.1:8080\\",
            "http://127.0.0.1:8080"
        ));
        assert_eq!(name_for("http://[::1]:8080/"), "[::1]:8080");
        assert_eq!(name_for("not an address"), "qBittorrent");
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

/// anirss's `[bestfit]` over anirss's own defaults, merged the way anirss merges its config:
/// what the person saved there (the groups they trust, the source order, the resolution and
/// the codec-normalised size target) ranks the native engine's best fit as it ranks anirss's
/// (#545). `None` when there is no file. One that does not read is anirss's defaults, as
/// anirss warns about it and ranks with them. A key of the wrong shape keeps the default.
pub fn anirss_bestfit(path: &Path) -> Option<BestfitConfig> {
    if !path.exists() {
        return None;
    }
    let mut cfg = BestfitConfig::default();
    let doc = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| toml::from_str::<toml::Table>(&text).ok());
    let Some(saved) = doc
        .as_ref()
        .and_then(|d| d.get("bestfit"))
        .and_then(|b| b.as_table())
    else {
        return Some(cfg);
    };
    // A list with anything but words in it is the wrong shape as a whole
    let words = |key: &str| -> Option<Vec<String>> {
        saved
            .get(key)?
            .as_array()?
            .iter()
            .map(|v| v.as_str().map(str::to_string))
            .collect()
    };
    let number = |key: &str| -> Option<f64> {
        let v = saved.get(key)?;
        v.as_float().or_else(|| v.as_integer().map(|i| i as f64))
    };
    if let Some(groups) = words("preferred_groups") {
        cfg.preferred_groups = groups;
    }
    if let Some(order) = words("source_order") {
        cfg.source_order = order;
    }
    // anirss reads the target through str(), so 1080 written as a number is "1080"
    match saved.get("preferred_resolution") {
        Some(toml::Value::String(s)) => cfg.preferred_resolution = s.clone(),
        Some(toml::Value::Integer(i)) => cfg.preferred_resolution = i.to_string(),
        Some(toml::Value::Float(f)) => cfg.preferred_resolution = format!("{f:?}"),
        _ => {}
    }
    if let Some(mib) = number("target_mib_per_episode") {
        cfg.target_mib_per_episode = mib;
    }
    if let Some(ratio) = number("max_size_ratio") {
        cfg.max_size_ratio = ratio;
    }
    if let Some(source) = saved.get("assumed_source").and_then(|v| v.as_str()) {
        cfg.assumed_source = source.to_string();
    }
    Some(cfg)
}

#[cfg(test)]
mod bestfit_tests {
    use super::*;
    use crate::releases::bestfit::best_item;

    fn read(text: &str) -> Option<BestfitConfig> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, text).unwrap();
        anirss_bestfit(&path)
    }

    fn items(titles: &[&str]) -> Vec<Release> {
        titles
            .iter()
            .enumerate()
            .map(|(i, t)| Release {
                title: t.to_string(),
                link: format!("link-{i}"),
                seeders: None,
                leechers: None,
                downloads: None,
                size: String::new(),
                category: String::new(),
                page: String::new(),
                source: String::new(),
                resolution: String::new(),
                infomkv: String::new(),
                linktype: String::new(),
            })
            .collect()
    }

    /// anirss's own test config, `_BESTFIT_CFG` in test_anirss.py.
    const ANIRSS_TEST_CFG: &str = r#"
[bestfit]
preferred_groups = ["Kumo-Subs", "Lumen", "ASW"]
source_order = ["WEB-DL", "WEB", "BluRay", "WEBRip", "HDTV"]
preferred_resolution = "highest"
target_mib_per_episode = 1000
max_size_ratio = 3.0
assumed_source = "WEB"
"#;

    #[test]
    fn no_config_is_nothing_and_one_that_does_not_read_is_anirss_s_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(anirss_bestfit(&dir.path().join("missing.toml")), None);
        // anirss warns about a bad config and ranks with its defaults
        assert_eq!(
            read("[bestfit\npreferred_groups = ["),
            Some(BestfitConfig::default())
        );
    }

    #[test]
    fn a_config_without_bestfit_ranks_as_anirss_does_out_of_the_box() {
        // test_config.py: the default config trusts no group
        let cfg = read("[downloads]\nsave_base = \"~/TV\"\n").unwrap();
        assert_eq!(cfg, BestfitConfig::default());
        assert!(cfg.preferred_groups.is_empty());
    }

    #[test]
    fn every_saved_key_is_read() {
        let cfg = read(ANIRSS_TEST_CFG).unwrap();
        assert_eq!(
            cfg,
            BestfitConfig {
                preferred_groups: vec!["Kumo-Subs".into(), "Lumen".into(), "ASW".into()],
                source_order: ["WEB-DL", "WEB", "BluRay", "WEBRip", "HDTV"]
                    .map(String::from)
                    .to_vec(),
                preferred_resolution: "highest".into(),
                target_mib_per_episode: 1000.0,
                max_size_ratio: 3.0,
                assumed_source: "WEB".into(),
            }
        );
    }

    #[test]
    fn a_partial_section_keeps_the_rest_of_anirss_s_defaults() {
        let cfg = read("[bestfit]\npreferred_resolution = 1080\nmax_size_ratio = 2\n").unwrap();
        assert_eq!(
            cfg.preferred_resolution, "1080",
            "a number reads as anirss's str() of it"
        );
        assert_eq!(cfg.max_size_ratio, 2.0);
        assert_eq!(cfg.source_order, BestfitConfig::default().source_order);
        assert_eq!(cfg.target_mib_per_episode, 1000.0);
        // A key of the wrong shape keeps the default rather than failing the whole section.
        let cfg = read("[bestfit]\npreferred_groups = \"Beta\"\nassumed_source = 3\n").unwrap();
        assert!(cfg.preferred_groups.is_empty());
        assert_eq!(cfg.assumed_source, "WEB");
        let cfg = read("[bestfit]\npreferred_groups = [\"Beta\", 1]\n").unwrap();
        assert!(
            cfg.preferred_groups.is_empty(),
            "{:?}",
            cfg.preferred_groups
        );
    }

    #[test]
    fn a_saved_resolution_target_avoids_4k() {
        // test_anirss.py: test_preferred_resolution_target_avoids_4k
        let text = ANIRSS_TEST_CFG.replace("\"highest\"", "\"1080\"");
        let cfg = read(&text).unwrap();
        let found = items(&[
            "[Kumo-Subs] Show 2160p WEB-DL",
            "[Kumo-Subs] Show 1080p WEB-DL",
        ]);
        assert_eq!(
            best_item(&found, &cfg).unwrap().title,
            "[Kumo-Subs] Show 1080p WEB-DL"
        );
        // anirss's stock "highest" takes the sharper one.
        let stock = read(ANIRSS_TEST_CFG).unwrap();
        assert_eq!(
            best_item(&found, &stock).unwrap().title,
            "[Kumo-Subs] Show 2160p WEB-DL"
        );
    }

    #[test]
    fn the_saved_group_order_decides_between_equal_releases() {
        let found = items(&["[Alpha] Show - 01 [1080p]", "[Beta] Show - 01 [1080p]"]);
        let pick = |groups: &str| {
            let cfg = read(&format!("[bestfit]\npreferred_groups = {groups}\n")).unwrap();
            best_item(&found, &cfg).unwrap().title.clone()
        };
        assert_eq!(pick(r#"["Beta", "Alpha"]"#), "[Beta] Show - 01 [1080p]");
        assert_eq!(pick(r#"["Alpha", "Beta"]"#), "[Alpha] Show - 01 [1080p]");
    }

    #[test]
    fn a_saved_size_target_reads_a_codec_s_efficiency_as_anirss_does() {
        // One episode of 600 MiB: starved against anirss's 1000 MiB x264 target, adequate
        // once the person lowers it. HEVC is credited for its efficiency either way.
        let found = |title: &str| Release {
            size: "600 MiB".into(),
            ..items(&[title])[0].clone()
        };
        let x264 = found("[A] Show - 01 [1080p x264]");
        let hevc = found("[B] Show - 01 [1080p HEVC]");
        let stock = read("[bestfit]\n").unwrap();
        let lowered = read("[bestfit]\ntarget_mib_per_episode = 500\n").unwrap();
        let both = [x264.clone(), hevc.clone()];
        assert_eq!(best_item(&both, &stock).unwrap().title, hevc.title);
        assert!(
            crate::releases::bestfit::size_ratio(&x264, &lowered, None).unwrap()
                > crate::releases::bestfit::size_ratio(&x264, &stock, None).unwrap()
        );
    }
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
name = "tracker"
kind = "rss"
url = "https://tracker.example/?q={query}"

[[endpoint]]
name = "proxied"
kind = "rss"
url = "http://127.0.0.1:9117/x?q={query}"
start = "podman pod start x"
ready = "http://127.0.0.1:9117/caps"
timeout = 45
"#,
        )
        .unwrap();
        assert_eq!(anirss_ondemand(&path, "tracker"), None);
        assert_eq!(
            anirss_ondemand(&path, "proxied"),
            Some(OnDemand {
                ready: "http://127.0.0.1:9117/caps".to_string(),
                timeout: 45
            })
        );
        assert_eq!(anirss_ondemand(&path, "keyed"), None);
    }
}
