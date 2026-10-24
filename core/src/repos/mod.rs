//! Endpoint repos (#143): signed indexes of endpoint definitions, added by URL the way Mihon
//! adds extension repos, and the endpoints installed from them. A definition is data the core
//! already knows how to run (`releases::Endpoint`); nothing in a repo is code.
//!
//! A repo is keyed by its index URL, so a site that publishes several groups is several repos.
//! Its key is pinned when the person trusts it; a later index under another key, or with a
//! lower serial, is refused and the last good index stays. Credentials go to the secret store
//! under `endpoint/<install id>/<key>` and are filled into the address only in memory, at
//! search time.

pub mod index;
mod pace;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::params;

use crate::contract::*;
use crate::core::Core;
use crate::net::{HttpRequest, Method};
use index::{Definition, Index, RepoFault};

pub use pace::Paced;

/// This core's version, which `min_core` is compared against.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

const FETCH_TIMEOUT: Duration = Duration::from_secs(20);

/// Indexes fetched by `AddEndpointRepo` and not yet trusted, so trusting one needs no
/// second fetch.
#[derive(Default)]
pub struct Pending {
    indexes: Mutex<HashMap<String, Index>>,
}

fn secret_key(install: &str, key: &str) -> String {
    format!("endpoint/{install}/{key}")
}

fn new_install_id() -> String {
    let a = rand::random::<u64>();
    let b = rand::random::<u64>();
    // A version 4 UUID from 128 random bits.
    let a = (a & 0xffff_ffff_ffff_0fff) | 0x0000_0000_0000_4000;
    let b = (b & 0x3fff_ffff_ffff_ffff) | 0x8000_0000_0000_0000;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        a >> 32,
        (a >> 16) & 0xffff,
        a & 0xffff,
        b >> 48,
        b & 0xffff_ffff_ffff
    )
}

/// The repo's address, trimmed; a local path is taken as it is.
fn normal_url(url: &str) -> Result<String, CoreError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(CoreError::invalid(
            "repo url",
            "paste the address of a repo",
        ));
    }
    // A phone keyboard capitalises the first letter: "Http://" is still an address.
    let url = match url.split_once("://") {
        Some((scheme, rest))
            if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") =>
        {
            format!("{}://{rest}", scheme.to_ascii_lowercase())
        }
        _ => url.to_string(),
    };
    let url = url.as_str();
    if url.starts_with("http://") && !is_local_http(url) {
        return Err(CoreError::invalid("repo url", "the address must be https"));
    }
    Ok(url.to_string())
}

/// Plain http only for this machine or the emulator's view of it, for testing a repo.
fn is_local_http(url: &str) -> bool {
    matches!(
        index::host_of(url).as_str(),
        "localhost" | "127.0.0.1" | "10.0.2.2"
    )
}

/// Fetches a repo file: over the network for an address, from disk for a local path.
async fn fetch_bytes(core: &Core, url: &str) -> Result<Vec<u8>, RepoFault> {
    if url.starts_with("https://") || url.starts_with("http://") {
        let http = Paced::new(core.http.clone(), Vec::new());
        let req = HttpRequest {
            method: Method::Get,
            url: url.to_string(),
            headers: vec![("Accept".into(), "application/json".into())],
            body: None,
            timeout: Some(FETCH_TIMEOUT),
        };
        let res = crate::net::Http::send(&http, req).await.map_err(|e| {
            RepoFault::new(
                "unreachable",
                format!("could not reach the repo: {}", e.message),
            )
        })?;
        if res.status == 404 {
            return Err(RepoFault::new(
                "missing",
                "nothing at that address (a private GitHub repo answers 404 too)",
            ));
        }
        if !res.is_success() {
            return Err(RepoFault::new(
                "unreachable",
                format!("the repo answered HTTP {}", res.status),
            ));
        }
        return Ok(res.body);
    }
    let path = url.strip_prefix("file://").unwrap_or(url);
    std::fs::read(path)
        .map_err(|e| RepoFault::new("missing", format!("could not read {path}: {e}")))
}

async fn fetch(core: &Core, url: &str) -> Result<Index, RepoFault> {
    let bytes = fetch_bytes(core, url).await?;
    index::parse(&bytes, CORE_VERSION)
}

// The store ---------------------------------------------------------------------------------

struct RepoRow {
    url: String,
    name: String,
    nsfw: bool,
    key: String,
    serial: u64,
    payload: Vec<u8>,
    fetched_at: i64,
    error: Option<String>,
}

struct InstallRow {
    install_id: String,
    repo_url: String,
    version: u32,
    def: Definition,
    enabled: bool,
}

fn read_repos(conn: &rusqlite::Connection) -> Result<Vec<RepoRow>, CoreError> {
    let mut st = conn.prepare(
        "SELECT url, name, nsfw, key, serial, payload, fetched_at, error FROM endpoint_repos ORDER BY rowid",
    )?;
    let rows = st
        .query_map([], |r| {
            Ok(RepoRow {
                url: r.get(0)?,
                name: r.get(1)?,
                nsfw: r.get(2)?,
                key: r.get(3)?,
                serial: r.get::<_, i64>(4)? as u64,
                payload: r.get(5)?,
                fetched_at: r.get(6)?,
                error: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn read_installed(conn: &rusqlite::Connection) -> Result<Vec<InstallRow>, CoreError> {
    let mut st = conn.prepare(
        "SELECT install_id, repo_url, version, definition, enabled FROM installed_endpoints ORDER BY position, rowid",
    )?;
    let rows = st
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, bool>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(install_id, repo_url, version, def, enabled)| {
            let def: Definition = serde_json::from_str(&def).ok()?;
            Some(InstallRow {
                install_id,
                repo_url,
                version: version as u32,
                def,
                enabled,
            })
        })
        .collect())
}

fn find_install(core: &Core, install: &str) -> Result<InstallRow, CoreError> {
    let all = core.store.read(read_installed)?;
    all.into_iter()
        .find(|r| r.install_id == install)
        .ok_or_else(|| CoreError::invalid("endpoint", "no installed endpoint with that id"))
}

/// Saves a checked index for a repo, the trusted key held to.
fn save_index(core: &Core, url: &str, idx: &Index) -> Result<(), CoreError> {
    let (url, name, nsfw, key, serial, payload) = (
        url.to_string(),
        idx.name.clone(),
        idx.nsfw,
        idx.key.clone(),
        idx.serial as i64,
        idx.payload.clone(),
    );
    core.store.write(move |conn| {
        conn.execute(
            "INSERT INTO endpoint_repos (url, name, nsfw, key, serial, payload, fetched_at, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)
             ON CONFLICT(url) DO UPDATE SET name = ?2, nsfw = ?3, serial = ?5, payload = ?6,
                 fetched_at = ?7, error = NULL",
            params![
                url,
                name,
                nsfw,
                key,
                serial,
                payload,
                crate::time::now_secs()
            ],
        )?;
        Ok(())
    })
}

fn save_error(core: &Core, url: &str, error: &str) -> Result<(), CoreError> {
    let (url, error) = (url.to_string(), error.to_string());
    core.store.write(move |conn| {
        conn.execute(
            "UPDATE endpoint_repos SET error = ?2 WHERE url = ?1",
            params![url, error],
        )?;
        Ok(())
    })
}

/// Whether a fetched index may replace the one in hand: the pinned key, and a serial that
/// did not go back.
fn admit(stored: &RepoRow, idx: &Index) -> Result<(), RepoFault> {
    if idx.key != stored.key {
        return Err(RepoFault::new(
            "key_changed",
            "the repo is now signed by a different key than the one you trusted. Remove it and add it again to accept the new key.",
        ));
    }
    if idx.serial < stored.serial {
        return Err(RepoFault::new(
            "stale",
            format!(
                "the repo served an older index (serial {} after {}); keeping the newer one",
                idx.serial, stored.serial
            ),
        ));
    }
    Ok(())
}

// The calls ---------------------------------------------------------------------------------

pub fn catalog(core: &Core) -> Result<EndpointCatalog, CoreError> {
    let (repos, installed) = core
        .store
        .read(|conn| Ok((read_repos(conn)?, read_installed(conn)?)))?;
    let offered_defs: Vec<(String, Definition)> = repos
        .iter()
        .flat_map(|r| {
            index::reread(&r.payload, CORE_VERSION)
                .into_iter()
                .map(move |d| (r.url.clone(), d))
        })
        .collect();
    let offered = offered_defs
        .iter()
        .map(|(repo, d)| OfferedEndpoint {
            repo_url: repo.clone(),
            id: d.id.clone(),
            name: d.name.clone(),
            version: d.version,
            lang: d.lang.clone(),
            kind: d.kind.clone(),
            host: d.host(),
            front: d.front.clone(),
            auth: d.fields(),
            nsfw: d.nsfw,
            installed: installed
                .iter()
                .find(|i| &i.repo_url == repo && i.def.id == d.id)
                .map(|i| i.install_id.clone()),
        })
        .collect();
    let secrets = core.secrets();
    let shadowed = shadowed_in_anirss(core);
    let installed = installed
        .iter()
        .map(|i| {
            let next = offered_defs
                .iter()
                .find(|(repo, d)| repo == &i.repo_url && d.id == i.def.id)
                .map(|(_, d)| d);
            let update = next.filter(|d| d.version > i.version);
            InstalledEndpoint {
                install_id: i.install_id.clone(),
                repo_url: i.repo_url.clone(),
                id: i.def.id.clone(),
                name: i.def.name.clone(),
                version: i.version,
                kind: i.def.kind.clone(),
                host: i.def.host(),
                front: i.def.front.clone(),
                auth: i.def.fields(),
                nsfw: i.def.nsfw,
                enabled: i.enabled,
                missing: i
                    .def
                    .auth
                    .iter()
                    .filter(|a| !secrets.has(&secret_key(&i.install_id, &a.key)))
                    .map(|a| a.key.clone())
                    .collect(),
                custom: i.repo_url == CUSTOM,
                offered: next.is_some() || i.repo_url == CUSTOM,
                desktop_note: shadowed.contains(&i.def.id).then(|| {
                    format!(
                        "anirss has its own {}, written by hand; the desktop searches that one",
                        i.def.id
                    )
                }),
                update: update.map(|d| d.version),
                update_asks: update.is_some_and(|d| !i.def.keeps_credentials(d)),
                update_host: update
                    .filter(|d| d.host() != i.def.host())
                    .map(|d| d.host()),
            }
        })
        .collect();
    let repos = repos
        .into_iter()
        .map(|r| EndpointRepo {
            fingerprint: index::fingerprint(&r.key),
            url: r.url,
            name: r.name,
            nsfw: r.nsfw,
            serial: r.serial,
            fetched_at: r.fetched_at,
            error: r.error,
        })
        .collect();
    Ok(EndpointCatalog {
        repos,
        offered,
        installed,
    })
}

fn reply(core: &Core) -> Result<Reply, CoreError> {
    Ok(Reply::EndpointCatalog {
        catalog: catalog(core)?,
    })
}

/// The reply after a change to what is installed: the desktop's anirss follows first.
fn changed(core: &Core) -> Result<Reply, CoreError> {
    crate::downloads::repo_endpoints_changed(core);
    reply(core)
}

fn stored_repo(core: &Core, url: &str) -> Result<Option<RepoRow>, CoreError> {
    let url = url.to_string();
    core.store
        .read(move |conn| Ok(read_repos(conn)?.into_iter().find(|r| r.url == url)))
}

pub fn add_repo(core: &Core, url: String) -> Result<Reply, CoreError> {
    let url = normal_url(&url)?;
    let idx = core.handle.block_on(fetch(core, &url))?;
    let trusted = stored_repo(core, &url)?.is_some_and(|r| r.key == idx.key);
    let preview = EndpointRepoPreview {
        url: url.clone(),
        name: idx.name.clone(),
        nsfw: idx.nsfw,
        key: idx.key.clone(),
        fingerprint: index::fingerprint(&idx.key),
        serial: idx.serial,
        endpoints: idx
            .endpoints
            .iter()
            .map(|d| OfferedEndpoint {
                repo_url: url.clone(),
                id: d.id.clone(),
                name: d.name.clone(),
                version: d.version,
                lang: d.lang.clone(),
                kind: d.kind.clone(),
                host: d.host(),
                front: d.front.clone(),
                auth: d.fields(),
                nsfw: d.nsfw,
                installed: None,
            })
            .collect(),
        trusted,
    };
    core.repos
        .indexes
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(url, idx);
    Ok(Reply::EndpointRepoPreview { preview })
}

pub fn trust_repo(core: &Core, url: String, key: String) -> Result<Reply, CoreError> {
    let url = normal_url(&url)?;
    let pending = core
        .repos
        .indexes
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&url);
    let idx = match pending {
        Some(idx) => idx,
        None => core.handle.block_on(fetch(core, &url))?,
    };
    if idx.key != key {
        return Err(RepoFault::new(
            "key_changed",
            "the repo's key is not the one you were shown. Add it again and compare the fingerprint.",
        )
        .into());
    }
    let stored = stored_repo(core, &url)?;
    if let Some(stored) = &stored {
        admit(stored, &idx)?;
    }
    save_index(core, &url, &idx)?;
    if stored.is_none() {
        core.bus.info(
            Stage::Downloads,
            format!("Added the endpoint repo {}", idx.name),
            EventBody::Notice,
        );
    }
    reply(core)
}

pub fn remove_repo(core: &Core, url: String) -> Result<Reply, CoreError> {
    let url = url.trim().to_string();
    let Some(stored) = stored_repo(core, &url)? else {
        return reply(core);
    };
    core.store.write(move |conn| {
        conn.execute("DELETE FROM endpoint_repos WHERE url = ?1", params![url])?;
        Ok(())
    })?;
    core.bus.info(
        Stage::Downloads,
        format!("Removed the endpoint repo {}", stored.name),
        EventBody::Notice,
    );
    reply(core)
}

/// Fetches every repo again. A repo that fails keeps its index and gets the reason on its row;
/// nothing is ever removed by a refresh.
pub fn refresh(core: &Core) -> Result<Reply, CoreError> {
    let repos = core.store.read(read_repos)?;
    let fetched = core.handle.block_on(async {
        let mut out = Vec::new();
        for r in &repos {
            out.push(fetch(core, &r.url).await);
        }
        out
    });
    for (stored, got) in repos.iter().zip(fetched) {
        match got.and_then(|idx| admit(stored, &idx).map(|()| idx)) {
            Ok(idx) => save_index(core, &stored.url, &idx)?,
            Err(fault) => save_error(core, &stored.url, &fault.message)?,
        }
    }
    reply(core)
}

/// Checks the person's values against an endpoint's fields: one for each, none left empty,
/// nothing that is not a field.
fn credentials_for(
    def: &Definition,
    given: &[EndpointCredential],
    all: bool,
) -> Result<Vec<(String, String)>, CoreError> {
    for c in given {
        if !def.auth.iter().any(|a| a.key == c.key) {
            return Err(CoreError::invalid(
                "credentials",
                format!("{} has no field {:?}", def.name, c.key),
            ));
        }
    }
    let mut out = Vec::new();
    for field in &def.auth {
        let value = given
            .iter()
            .find(|c| c.key == field.key)
            .map(|c| c.value.trim().to_string())
            .unwrap_or_default();
        if value.is_empty() {
            if all {
                return Err(CoreError::invalid(
                    "credentials",
                    format!("{} needs its {}", def.name, field.label),
                ));
            }
            continue;
        }
        out.push((field.key.clone(), value));
    }
    Ok(out)
}

fn store_credentials(
    core: &Core,
    install: &str,
    values: &[(String, String)],
) -> Result<(), CoreError> {
    for (key, value) in values {
        core.secrets().set(&secret_key(install, key), value)?;
    }
    Ok(())
}

fn offered_def(core: &Core, repo: &str, id: &str) -> Result<Definition, CoreError> {
    let stored = stored_repo(core, repo)?
        .ok_or_else(|| CoreError::invalid("repo", "that repo is not added"))?;
    index::reread(&stored.payload, CORE_VERSION)
        .into_iter()
        .find(|d| d.id == id)
        .ok_or_else(|| {
            CoreError::invalid("endpoint", format!("{} does not offer {id}", stored.name))
        })
}

pub fn install(
    core: &Core,
    repo: String,
    id: String,
    credentials: Vec<EndpointCredential>,
) -> Result<Reply, CoreError> {
    let repo = repo.trim().to_string();
    let def = offered_def(core, &repo, &id)?;
    let values = credentials_for(&def, &credentials, true)?;
    let existing = core.store.read(read_installed)?;
    if existing
        .iter()
        .any(|i| i.repo_url == repo && i.def.id == id)
    {
        return Err(CoreError::invalid(
            "endpoint",
            format!("{} is already installed", def.name),
        ));
    }
    let install_id = new_install_id();
    store_credentials(core, &install_id, &values)?;
    let (row_id, row_repo, row_def, version, json) = (
        install_id.clone(),
        repo.clone(),
        def.id.clone(),
        def.version as i64,
        serde_json::to_string(&def).map_err(|e| CoreError::internal(e.to_string()))?,
    );
    core.store.write(move |conn| {
        conn.execute(
            "INSERT INTO installed_endpoints (install_id, repo_url, endpoint_id, version, definition, enabled, position)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, (SELECT COALESCE(MAX(position), 0) + 1 FROM installed_endpoints))",
            params![row_id, row_repo, row_def, version, json],
        )?;
        Ok(())
    })?;
    core.bus.info(
        Stage::Downloads,
        format!("Installed the endpoint {}", def.name),
        EventBody::Notice,
    );
    changed(core)
}

pub fn update(
    core: &Core,
    install: String,
    credentials: Vec<EndpointCredential>,
) -> Result<Reply, CoreError> {
    let row = find_install(core, &install)?;
    let next = offered_def(core, &row.repo_url, &row.def.id)?;
    if next.version <= row.version {
        return reply(core);
    }
    if row.def.keeps_credentials(&next) {
        // Values given anyway replace the stored ones.
        let values = credentials_for(&next, &credentials, false)?;
        store_credentials(core, &install, &values)?;
    } else {
        let values = credentials_for(&next, &credentials, true)?;
        for field in &row.def.auth {
            core.secrets().delete(&secret_key(&install, &field.key))?;
        }
        store_credentials(core, &install, &values)?;
    }
    let (id, version, json) = (
        install.clone(),
        next.version as i64,
        serde_json::to_string(&next).map_err(|e| CoreError::internal(e.to_string()))?,
    );
    core.store.write(move |conn| {
        conn.execute(
            "UPDATE installed_endpoints SET version = ?2, definition = ?3 WHERE install_id = ?1",
            params![id, version, json],
        )?;
        Ok(())
    })?;
    core.bus.info(
        Stage::Downloads,
        format!(
            "Updated the endpoint {} to version {}",
            next.name, next.version
        ),
        EventBody::Notice,
    );
    changed(core)
}

pub fn remove(core: &Core, install: String) -> Result<Reply, CoreError> {
    let row = find_install(core, &install)?;
    for field in &row.def.auth {
        core.secrets().delete(&secret_key(&install, &field.key))?;
    }
    core.store.write(move |conn| {
        conn.execute(
            "DELETE FROM installed_endpoints WHERE install_id = ?1",
            params![install],
        )?;
        Ok(())
    })?;
    core.bus.info(
        Stage::Downloads,
        format!("Removed the endpoint {}", row.def.name),
        EventBody::Notice,
    );
    changed(core)
}

pub fn set_credentials(
    core: &Core,
    install: String,
    credentials: Vec<EndpointCredential>,
) -> Result<Reply, CoreError> {
    let row = find_install(core, &install)?;
    let values = credentials_for(&row.def, &credentials, false)?;
    store_credentials(core, &install, &values)?;
    changed(core)
}

pub fn set_enabled(core: &Core, install: String, enabled: bool) -> Result<Reply, CoreError> {
    let rows = core.store.write(move |conn| {
        Ok(conn.execute(
            "UPDATE installed_endpoints SET enabled = ?2 WHERE install_id = ?1",
            params![install, enabled],
        )?)
    })?;
    if rows == 0 {
        return Err(CoreError::invalid(
            "endpoint",
            "no installed endpoint with that id",
        ));
    }
    changed(core)
}

// Endpoints added by hand ---------------------------------------------------------------

/// The repo url of an endpoint the person added by hand, outside any repo.
pub const CUSTOM: &str = "";

/// Query parameters that hold a secret: their value moves to the secret store and the
/// address keeps a placeholder (the same names `releases::native::mask` hides).
const SECRET_PARAMS: [(&str, &str); 4] = [
    ("apikey", "API key"),
    ("api_key", "API key"),
    ("token", "Token"),
    ("passkey", "Passkey"),
];

/// An id from a name: lower case, anything but a-z, 0-9, '.', '_' and '-' becomes '-'.
fn id_of(name: &str) -> String {
    let id: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect();
    id.trim_matches('-').to_string()
}

/// Takes the secret values out of a pasted address: each `apikey=...`, `token=...` and the
/// like becomes a placeholder and an `auth` field, and its value a credential. Any other
/// placeholder the person wrote (`{user}`) becomes a field too, to be filled from `given`.
fn lift_secrets(url: &str) -> (String, Vec<index::AuthField>, Vec<(String, String)>) {
    let mut fields: Vec<index::AuthField> = Vec::new();
    let mut lifted: Vec<(String, String)> = Vec::new();
    let (base, query) = match url.split_once('?') {
        Some((b, q)) => (b.to_string(), Some(q.to_string())),
        None => (url.to_string(), None),
    };
    let mut template = base;
    if let Some(query) = query {
        let pairs: Vec<String> = query
            .split('&')
            .map(|pair| {
                let Some((k, v)) = pair.split_once('=') else {
                    return pair.to_string();
                };
                let lower = k.to_lowercase();
                let Some((_, label)) = SECRET_PARAMS.iter().find(|(p, _)| *p == lower) else {
                    return pair.to_string();
                };
                if v.is_empty() || v.starts_with('{') {
                    return pair.to_string();
                }
                if !fields.iter().any(|f| f.key == lower) {
                    fields.push(index::AuthField {
                        key: lower.clone(),
                        label: label.to_string(),
                        secret: true,
                        help: String::new(),
                        help_url: None,
                    });
                    lifted.push((lower.clone(), crate::releases::endpoints::unquote_plus(v)));
                }
                format!("{k}={{{lower}}}")
            })
            .collect();
        template = format!("{template}?{}", pairs.join("&"));
    }
    // Placeholders the person wrote themselves.
    let mut rest = template.as_str();
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else { break };
        let key = &after[..close];
        if key != "query" && !fields.iter().any(|f| f.key == key) {
            fields.push(index::AuthField {
                key: key.to_string(),
                label: key.to_string(),
                secret: true,
                help: String::new(),
                help_url: None,
            });
        }
        rest = &after[close + 1..];
    }
    (template, fields, lifted)
}

/// Adds an endpoint by hand, beside whatever repos there are: a name and an address, for a
/// site no repo carries (1337x through Jackett, a private tracker's feed). A secret in the
/// address goes to the secret store like a repo endpoint's.
pub fn add_custom(
    core: &Core,
    name: String,
    url: String,
    credentials: Vec<EndpointCredential>,
) -> Result<Reply, CoreError> {
    let name = name.trim().to_string();
    let id = id_of(&name);
    if id.is_empty() {
        return Err(CoreError::invalid("endpoint name", "give it a name"));
    }
    let url = url.trim();
    let url = match url.split_once("://") {
        Some((scheme, rest)) => format!("{}://{rest}", scheme.to_ascii_lowercase()),
        None => url.to_string(),
    };
    let (template, auth, lifted) = lift_secrets(&url);
    // An address with the search in it is a feed; a bare site address is nyaa's kind,
    // which adds its own search.
    let kind = if template.contains("{query}") {
        "rss"
    } else if index::host_of(&template).contains("nyaa") {
        "nyaa"
    } else {
        return Err(CoreError::invalid(
            "endpoint url",
            "put {query} in the address where the search goes",
        ));
    };
    let def = Definition {
        id: id.clone(),
        name: name.clone(),
        version: 1,
        lang: String::new(),
        kind: kind.to_string(),
        url: template,
        auth,
        front: None,
        rate: None,
        nsfw: false,
        category: None,
        filter: None,
        min_core: None,
    };
    index::check_custom(&def).map_err(|e| CoreError::invalid("endpoint", e))?;
    let installed = core.store.read(read_installed)?;
    if installed.iter().any(|i| i.def.id == id) {
        return Err(CoreError::invalid(
            "endpoint name",
            format!("an endpoint called {id} is already installed"),
        ));
    }
    let mut given = credentials;
    for (key, value) in lifted {
        if !given.iter().any(|c| c.key == key) {
            given.push(EndpointCredential { key, value });
        }
    }
    let values = credentials_for(&def, &given, true)?;
    let install_id = new_install_id();
    store_credentials(core, &install_id, &values)?;
    let (row_id, row_def, json) = (
        install_id.clone(),
        def.id.clone(),
        serde_json::to_string(&def).map_err(|e| CoreError::internal(e.to_string()))?,
    );
    core.store.write(move |conn| {
        conn.execute(
            "INSERT INTO installed_endpoints (install_id, repo_url, endpoint_id, version, definition, enabled, position)
             VALUES (?1, ?2, ?3, 1, ?4, 1, (SELECT COALESCE(MAX(position), 0) + 1 FROM installed_endpoints))",
            params![row_id, CUSTOM, row_def, json],
        )?;
        Ok(())
    })?;
    core.bus.info(
        Stage::Downloads,
        format!("Added the endpoint {name}"),
        EventBody::Notice,
    );
    changed(core)
}

// The desktop's anirss ------------------------------------------------------------------

/// Installed endpoints whose name anirss already holds from someone's own hand: the sync
/// leaves those alone.
fn shadowed_in_anirss(core: &Core) -> Vec<String> {
    let theirs = crate::downloads::anirss_endpoint_names(core);
    if theirs.is_empty() {
        return Vec::new();
    }
    let synced: Vec<String> = core
        .store
        .read(|c| crate::store::settings::get(c, crate::store::settings::ANIRSS_SYNCED_ENDPOINTS))
        .ok()
        .flatten()
        .unwrap_or_default();
    theirs.into_iter().filter(|n| !synced.contains(n)).collect()
}

/// Puts the installed endpoints into the desktop's anirss, which searches only what its own
/// config lists: each enabled one with its credentials, under its id. Only names AniBeam
/// wrote are ever changed or removed; one anirss already had from its config stays as it was.
/// anirss keeps the address as it is given, so a credential lands in its config.toml, the way
/// a hand-written one does today; the secret store stays the record AniBeam reads.
pub(crate) async fn sync_anirss(core: &Core, client: &crate::downloads::anirss::AnirssClient) {
    let Ok(have) = client.endpoints().await else {
        return;
    };
    let want = runnable(core);
    let mut synced: Vec<String> = core
        .store
        .read(|c| crate::store::settings::get(c, crate::store::settings::ANIRSS_SYNCED_ENDPOINTS))
        .ok()
        .flatten()
        .unwrap_or_default();
    let before = synced.clone();
    for r in &want {
        let e = &r.endpoint;
        let theirs = have.iter().find(|h| h.name == e.name);
        if theirs.is_some() && !synced.contains(&e.name) {
            continue;
        }
        if theirs.is_some_and(|h| h.url == e.url && h.kind == e.kind) {
            continue;
        }
        let mut spec = serde_json::json!({"name": e.name, "kind": e.kind, "url": e.url});
        if e.kind == "nyaa" {
            spec["category"] = e.category.clone().into();
            spec["filter"] = e.filter.clone().into();
        }
        match client.put_endpoint(spec).await {
            Ok(()) => {
                if !synced.contains(&e.name) {
                    synced.push(e.name.clone());
                }
            }
            Err(err) => tracing::warn!("anirss refused the endpoint {}: {err}", e.name),
        }
    }
    for name in before {
        if want.iter().any(|r| r.endpoint.name == name) {
            continue;
        }
        if have.iter().any(|h| h.name == name)
            && let Err(err) = client.delete_endpoint(&name).await
        {
            tracing::warn!("anirss kept the endpoint {name}: {err}");
            continue;
        }
        synced.retain(|n| n != &name);
    }
    let _ = core.store.write(move |c| {
        crate::store::settings::set(c, crate::store::settings::ANIRSS_SYNCED_ENDPOINTS, &synced)
    });
}

// For the search --------------------------------------------------------------------------

/// One installed endpoint ready to search: the runnable endpoint with its credentials filled,
/// the address as it may be shown, and the pace its definition asks for.
pub struct Runnable {
    pub endpoint: crate::releases::Endpoint,
    /// The template, no secrets in it.
    pub shown_url: String,
    pub host: String,
    pub per_minute: Option<u32>,
}

/// The enabled installed endpoints, in order, with every credential they need. One whose
/// credentials are missing is left out.
pub fn runnable(core: &Core) -> Vec<Runnable> {
    let Ok(rows) = core.store.read(read_installed) else {
        return Vec::new();
    };
    let secrets = core.secrets();
    let mut out: Vec<Runnable> = Vec::new();
    for row in rows.into_iter().filter(|r| r.enabled) {
        if !crate::releases::endpoints::VALID_KINDS.contains(&row.def.kind.as_str()) {
            continue;
        }
        // Two repos offering the same id: the first installed wins the name.
        if out.iter().any(|r| r.endpoint.name == row.def.id) {
            continue;
        }
        let filled = row.def.filled_url(|key| {
            secrets
                .get(&secret_key(&row.install_id, key), None)
                .ok()
                .flatten()
                .map(|(v, _)| v)
        });
        let Some(url) = filled else { continue };
        let mut endpoint = crate::releases::Endpoint::new(&row.def.id, &row.def.kind, &url);
        if let Some(c) = &row.def.category {
            endpoint.category = c.clone();
        }
        if let Some(f) = &row.def.filter {
            endpoint.filter = f.clone();
        }
        out.push(Runnable {
            endpoint,
            shown_url: row.def.url.clone(),
            host: row.def.host(),
            per_minute: row.def.rate.map(|r| r.per_minute),
        });
    }
    out
}

/// Wraps the core's client so each host with a rate from its definition is paced to it.
pub fn paced_http(core: &Core, runnable: &[Runnable]) -> Arc<dyn crate::net::Http> {
    let gaps = runnable
        .iter()
        .filter_map(|r| {
            r.per_minute.map(|n| {
                (
                    r.host.clone(),
                    Duration::from_secs_f64(60.0 / f64::from(n.max(1))),
                )
            })
        })
        .collect();
    Arc::new(Paced::new(core.http.clone(), gaps))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::downloads::anirss::AnirssClient;
    use crate::net::FakeHttp;
    use crate::trackers::Secrets;
    use crate::{CorePaths, core::Core};

    fn custom(core: &Core, name: &str, url: &str) {
        add_custom(core, name.into(), url.into(), Vec::new()).unwrap();
    }

    #[test]
    fn anirss_gets_ours_and_keeps_what_was_written_by_hand() {
        let dir = tempfile::tempdir().unwrap();
        let paths = CorePaths::under(dir.path());
        let http = FakeHttp::new();
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            http.clone(),
            Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        custom(&core, "mine", "https://a.example/rss?q={query}&token=t1");
        custom(&core, "1337x", "http://127.0.0.1:9117/x?apikey=k&q={query}");
        let anirss = AnirssClient::new("http://127.0.0.1:1", "tok", http.clone());

        // anirss has its own 1337x from config.toml; AniBeam writes only "mine".
        http.push_json(
            200,
            serde_json::json!({"endpoints": [{"name": "1337x", "kind": "rss", "url": "http://elsewhere/"}]}),
        );
        http.push_json(201, serde_json::json!({}));
        core.handle.block_on(sync_anirss(&core, &anirss));
        let sent = http.requests();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[1].method, crate::net::Method::Post);
        let body = format!("{:?}", sent[1].body);
        assert!(
            body.contains("\"mine\"") && body.contains("token=t1"),
            "{body}"
        );

        // Switched off: it comes out of anirss again, and the hand-written one stays.
        let mine = core
            .store
            .read(read_installed)
            .unwrap()
            .into_iter()
            .find(|r| r.def.id == "mine")
            .unwrap();
        core.store
            .write(move |c| {
                c.execute(
                    "UPDATE installed_endpoints SET enabled = 0 WHERE install_id = ?1",
                    params![mine.install_id],
                )?;
                Ok(())
            })
            .unwrap();
        http.push_json(
            200,
            serde_json::json!({"endpoints": [
                {"name": "1337x", "kind": "rss", "url": "http://elsewhere/"},
                {"name": "mine", "kind": "rss", "url": "https://a.example/rss?q={query}&token=t1"}
            ]}),
        );
        http.push_json(200, serde_json::json!({}));
        core.handle.block_on(sync_anirss(&core, &anirss));
        let sent = http.requests();
        assert_eq!(sent.len(), 4);
        assert_eq!(sent[3].method, crate::net::Method::Delete);
        assert!(sent[3].url.ends_with("/endpoints/mine"));
        core.shutdown();
    }

    #[test]
    fn install_ids_are_version_4_uuids() {
        let id = new_install_id();
        assert_eq!(id.len(), 36);
        assert_eq!(&id[14..15], "4");
        assert!(matches!(&id[19..20], "8" | "9" | "a" | "b"));
        assert_ne!(id, new_install_id());
    }

    #[test]
    fn a_pasted_api_key_moves_out_of_the_address() {
        let (template, fields, lifted) = lift_secrets(
            "http://127.0.0.1:9117/api/v2.0/indexers/1337x/results/torznab/api?apikey=abc123&t=search&q={query}",
        );
        assert_eq!(
            template,
            "http://127.0.0.1:9117/api/v2.0/indexers/1337x/results/torznab/api?apikey={apikey}&t=search&q={query}"
        );
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].label, "API key");
        assert_eq!(lifted, vec![("apikey".to_string(), "abc123".to_string())]);
    }

    #[test]
    fn a_placeholder_written_by_hand_becomes_a_field() {
        let (template, fields, lifted) =
            lift_secrets("https://t.example/rss?q={query}&user={user}&passkey=zz");
        assert_eq!(
            template,
            "https://t.example/rss?q={query}&user={user}&passkey={passkey}"
        );
        let keys: Vec<&str> = fields.iter().map(|f| f.key.as_str()).collect();
        assert_eq!(keys, ["passkey", "user"]);
        assert_eq!(lifted.len(), 1);
    }

    #[test]
    fn ids_come_from_names() {
        assert_eq!(id_of("1337x"), "1337x");
        assert_eq!(id_of(" My Tracker! "), "my-tracker");
        assert_eq!(id_of("!!"), "");
    }

    #[test]
    fn plain_http_only_for_this_machine() {
        assert!(normal_url("http://10.0.2.2:8000/general.json").is_ok());
        assert!(normal_url("http://example.com/x.json").is_err());
        assert!(normal_url("https://example.com/x.json").is_ok());
        assert!(normal_url("/tmp/general.json").is_ok());
        assert!(normal_url("  ").is_err());
        assert_eq!(
            normal_url("Https://example.com/x.json").unwrap(),
            "https://example.com/x.json"
        );
    }
}
