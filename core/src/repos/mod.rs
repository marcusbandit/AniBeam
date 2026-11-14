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
use crate::downloads::EndpointSync;
use crate::i18n::tr;
use crate::net::HttpRequest;
use crate::sync::recover;
use index::{Definition, Index, RepoFault};

pub use pace::{FLOOR, Paced};

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
            tr("core-repos-paste-address", &[]),
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
        return Err(CoreError::invalid(
            "repo url",
            tr("core-repos-must-be-https", &[]),
        ));
    }
    Ok(url.to_string())
}

/// Plain http only for this machine or the emulator's view of it, for testing a repo.
fn is_local_http(url: &str) -> bool {
    matches!(
        index::host_of(url).as_str(),
        "localhost" | "127.0.0.1" | "::1" | "10.0.2.2"
    )
}

/// Fetches a repo file: over the network for an address, from disk for a local path.
async fn fetch_bytes(core: &Core, url: &str) -> Result<Vec<u8>, RepoFault> {
    if url.starts_with("https://") || url.starts_with("http://") {
        // A repo's host has no rate of its own; it waits the floor like any endpoint host.
        let http = Paced::new(core.http.clone(), Vec::new());
        let req = HttpRequest::get(url.to_string())
            .header("Accept", "application/json")
            .timeout(FETCH_TIMEOUT);
        let res = crate::net::Http::send(&http, req).await.map_err(|e| {
            RepoFault::new(
                "unreachable",
                tr(
                    "core-repos-unreachable",
                    &[("message", e.message.as_str().into())],
                ),
            )
        })?;
        if res.status == 404 {
            return Err(RepoFault::new("missing", tr("core-repos-not-found", &[])));
        }
        if !res.is_success() {
            return Err(RepoFault::new(
                "unreachable",
                tr(
                    "core-repos-http-status",
                    &[("status", res.status.to_string().into())],
                ),
            ));
        }
        return Ok(res.body);
    }
    let path = url.strip_prefix("file://").unwrap_or(url);
    std::fs::read(path).map_err(|e| {
        RepoFault::new(
            "missing",
            tr(
                "core-repos-unreadable",
                &[("path", path.into()), ("error", e.to_string().into())],
            ),
        )
    })
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
        .ok_or_else(|| CoreError::invalid("endpoint", tr("core-repos-no-such-endpoint", &[])))
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

/// The same for a repo already saved; nothing when it was removed in the meantime.
fn update_index(core: &Core, url: &str, idx: &Index) -> Result<(), CoreError> {
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
            "UPDATE endpoint_repos SET name = ?2, nsfw = ?3, serial = ?5, payload = ?6,
                 fetched_at = ?7, error = NULL
             WHERE url = ?1 AND key = ?4",
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
            tr("core-repos-key-changed", &[]),
        ));
    }
    if idx.serial < stored.serial {
        return Err(RepoFault::new(
            "stale",
            tr(
                "core-repos-stale",
                &[
                    ("serial", idx.serial.to_string().into()),
                    ("kept", stored.serial.to_string().into()),
                ],
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
    let synced = read_synced(core);
    // One engine and one anirss view for the whole catalog, not a read per row.
    let engine = crate::downloads::backend::engine(core)?;
    let view = crate::downloads::anirss_view(core);
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
                address: (i.repo_url == CUSTOM).then(|| i.def.url.clone()),
                offered: next.is_some() || i.repo_url == CUSTOM,
                desktop_note: note_for(engine, view.as_ref(), &i.def, &synced),
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
    // One preview at a time: a shell shows one, so an older one is let go.
    let mut held = recover(&core.repos.indexes);
    held.clear();
    held.insert(url, idx);
    Ok(Reply::EndpointRepoPreview { preview })
}

pub fn trust_repo(core: &Core, url: String, key: String) -> Result<Reply, CoreError> {
    let url = normal_url(&url)?;
    let pending = recover(&core.repos.indexes).remove(&url);
    let idx = match pending {
        Some(idx) => idx,
        None => core.handle.block_on(fetch(core, &url))?,
    };
    if idx.key != key {
        return Err(RepoFault::new("key_changed", tr("core-repos-key-not-shown", &[])).into());
    }
    let stored = stored_repo(core, &url)?;
    if let Some(stored) = &stored {
        admit(stored, &idx)?;
    }
    save_index(core, &url, &idx)?;
    if stored.is_none() {
        core.bus.say(
            Level::Info,
            Stage::Downloads,
            crate::words!("core-repos-repo-added"),
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
    core.bus.say(
        Level::Info,
        Stage::Downloads,
        crate::words!("core-repos-repo-removed"),
        format!("Removed the endpoint repo {}", stored.name),
        EventBody::Notice,
    );
    reply(core)
}

/// Fetches every repo again. A repo that fails keeps its index and gets the reason on its row;
/// nothing is ever removed by a refresh.
pub fn refresh(core: &Core) -> Result<Reply, CoreError> {
    let repos = core.store.read(read_repos)?;
    let owner = core.owner()?;
    // All at once, so a dead repo costs one timeout, not one each.
    let tasks: Vec<_> = repos
        .iter()
        .map(|r| {
            let (owner, url) = (owner.clone(), r.url.clone());
            core.handle.spawn(async move { fetch(&owner, &url).await })
        })
        .collect();
    let fetched: Vec<Result<Index, RepoFault>> = core.handle.block_on(async {
        let mut out = Vec::new();
        for t in tasks {
            out.push(
                t.await
                    .unwrap_or_else(|e| Err(RepoFault::new("unreachable", e.to_string()))),
            );
        }
        out
    });
    for (stored, got) in repos.iter().zip(fetched) {
        match got.and_then(|idx| admit(stored, &idx).map(|()| idx)) {
            // An update only: a repo removed while its fetch ran stays removed.
            Ok(idx) => update_index(core, &stored.url, &idx)?,
            Err(fault) => save_error(core, &stored.url, &fault.message)?,
        }
    }
    reply(core)
}

/// Checks the person's values against an endpoint's fields: one for each, none left empty,
/// nothing that is not a field, and a bare host for `{host}`.
fn credentials_for(
    def: &Definition,
    given: &[EndpointCredential],
    all: bool,
) -> Result<Vec<(String, String)>, CoreError> {
    for c in given {
        if !def.auth.iter().any(|a| a.key == c.key) {
            return Err(CoreError::invalid(
                "credentials",
                tr(
                    "core-repos-no-field",
                    &[
                        ("name", def.name.as_str().into()),
                        ("key", format!("{:?}", c.key).into()),
                    ],
                ),
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
                    tr(
                        "core-repos-needs-field",
                        &[
                            ("name", def.name.as_str().into()),
                            ("label", field.label.as_str().into()),
                        ],
                    ),
                ));
            }
            continue;
        }
        if field.key == index::HOST_KEY && def.host_is_placeholder() && !index::bare_host(&value) {
            return Err(CoreError::invalid(
                "credentials",
                tr(
                    "core-repos-host-only",
                    &[
                        ("name", def.name.as_str().into()),
                        ("label", field.label.as_str().into()),
                    ],
                ),
            ));
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
        .ok_or_else(|| CoreError::invalid("repo", tr("core-repos-not-added", &[])))?;
    index::reread(&stored.payload, CORE_VERSION)
        .into_iter()
        .find(|d| d.id == id)
        .ok_or_else(|| {
            CoreError::invalid(
                "endpoint",
                tr(
                    "core-repos-not-offered",
                    &[("repo", stored.name.as_str().into()), ("id", id.into())],
                ),
            )
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
    if let Some(other) = existing.iter().find(|i| i.def.id == id) {
        return Err(CoreError::invalid(
            "endpoint",
            if other.repo_url == repo {
                tr(
                    "core-repos-already-installed",
                    &[("name", def.name.as_str().into())],
                )
            } else if other.repo_url == CUSTOM {
                tr("core-repos-taken-by-you", &[("id", id.as_str().into())])
            } else {
                tr("core-repos-taken-by-repo", &[("id", id.as_str().into())])
            },
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
    core.bus.say(
        Level::Info,
        Stage::Downloads,
        crate::words!("core-repos-source-added"),
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
    core.bus.say(
        Level::Info,
        Stage::Downloads,
        crate::words!("core-repos-source-updated"),
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
    core.bus.say(
        Level::Info,
        Stage::Downloads,
        crate::words!("core-repos-source-removed"),
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
            tr("core-repos-no-such-endpoint", &[]),
        ));
    }
    changed(core)
}

// Endpoints added by hand ---------------------------------------------------------------

/// The repo url of an endpoint the person added by hand, outside any repo.
pub const CUSTOM: &str = "";

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
/// like (`redact::SECRET_PARAMS`) becomes a placeholder and an `auth` field, and its
/// value a credential, and so does the password of a `user:password@` login. Any other
/// placeholder the person wrote (`{user}`) becomes a field too, to be filled from `given`.
fn lift_secrets(url: &str) -> (String, Vec<index::AuthField>, Vec<(String, String)>) {
    let mut fields: Vec<index::AuthField> = Vec::new();
    let mut lifted: Vec<(String, String)> = Vec::new();
    let mut lift = |key: &str, label: &str, value: &str| {
        if !fields.iter().any(|f| f.key == key) {
            fields.push(index::AuthField {
                key: key.to_string(),
                label: label.to_string(),
                secret: true,
                help: String::new(),
                help_url: None,
            });
            lifted.push((key.to_string(), value.to_string()));
        }
    };
    let (base, query) = match url.split_once('?') {
        Some((b, q)) => (b.to_string(), Some(q.to_string())),
        None => (url.to_string(), None),
    };
    // A login in front of the host: the user name stays, the password is a secret. A login
    // with no password is a key on its own.
    let hidden = |part: &str| part.is_empty() || part.starts_with('{');
    let mut template = match index::split_login(&base) {
        Some((scheme, login, host)) => match login.split_once(':') {
            Some((user, pw)) if !hidden(pw) => {
                lift("login_password", "Password", &percent_decode(pw));
                format!("{scheme}{user}:{{login_password}}@{host}")
            }
            // An empty password: the user name is the key. A placeholder password was lifted
            // already, so the user name stays.
            Some((user, pw)) if pw.is_empty() && !hidden(user) => {
                lift("login_key", "Login", &percent_decode(user));
                format!("{scheme}{{login_key}}:{pw}@{host}")
            }
            None if !hidden(login) => {
                lift("login_key", "Login", &percent_decode(login));
                format!("{scheme}{{login_key}}@{host}")
            }
            _ => base,
        },
        None => base,
    };
    if let Some(query) = query {
        let pairs: Vec<String> = query
            .split('&')
            .map(|pair| {
                let Some((k, v)) = pair.split_once('=') else {
                    return pair.to_string();
                };
                let Some(label) = crate::redact::secret_param(k) else {
                    return pair.to_string();
                };
                if v.is_empty() || v.starts_with('{') {
                    return pair.to_string();
                }
                let lower = k.to_lowercase();
                lift(&lower, label, &crate::releases::endpoints::unquote_plus(v));
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

/// A login's percent escapes undone; unlike a query value, its `+` is a plus.
fn percent_decode(s: &str) -> String {
    crate::releases::endpoints::unquote_plus(&s.replace('+', "%2B"))
}

/// The definition of a source added by hand, and the secrets lifted out of its address.
/// `kept` is the kind of the source being edited: one that builds its own search from a bare
/// address keeps doing so when the address stays bare.
fn custom_definition(
    name: &str,
    url: &str,
    kept: Option<&str>,
) -> Result<(Definition, Vec<(String, String)>), CoreError> {
    let name = name.trim().to_string();
    let id = id_of(&name);
    if id.is_empty() {
        return Err(CoreError::invalid(
            "endpoint name",
            tr("core-repos-give-name", &[]),
        ));
    }
    let url = url.trim();
    let url = match url.split_once("://") {
        Some((scheme, rest)) => format!("{}://{rest}", scheme.to_ascii_lowercase()),
        None => url.to_string(),
    };
    let (template, auth, lifted) = lift_secrets(&url);
    // An address with the search in it is a feed. A kind that builds its own search from a
    // bare address comes from a repo, whose definition names it, or was added by hand before
    // #602 and is being edited.
    let kind = if template.contains("{query}") {
        "rss"
    } else if let Some(kind) = kept.filter(|k| *k == crate::releases::kinds::FILTERED_RSS) {
        kind
    } else {
        return Err(CoreError::invalid(
            "endpoint url",
            tr("core-repos-put-query", &[]),
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
        json: None,
        quirks: Default::default(),
        trusted_groups: Vec::new(),
    };
    index::check_custom(&def).map_err(|e| CoreError::invalid("endpoint", e))?;
    Ok((def, lifted))
}

/// Adds an endpoint by hand, beside whatever repos there are: a name and an address, for a
/// site no repo carries (a site behind a search proxy, a private tracker's feed). A secret in the
/// address goes to the secret store like a repo endpoint's.
pub fn add_custom(
    core: &Core,
    name: String,
    url: String,
    credentials: Vec<EndpointCredential>,
) -> Result<Reply, CoreError> {
    let (def, lifted) = custom_definition(&name, &url, None)?;
    let (id, name) = (def.id.clone(), def.name.clone());
    let installed = core.store.read(read_installed)?;
    if installed.iter().any(|i| i.def.id == id) {
        return Err(CoreError::invalid(
            "endpoint name",
            tr("core-repos-name-taken", &[("id", id.as_str().into())]),
        ));
    }
    let mut given = credentials;
    for (key, value) in lifted {
        if !given.iter().any(|c| c.key == key) {
            given.push(EndpointCredential { key, value });
        }
    }
    // A placeholder the person typed may be filled in afterwards; until it is, the endpoint
    // says what it needs and is left out of searches.
    let values = credentials_for(&def, &given, false)?;
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
    core.bus.say(
        Level::Info,
        Stage::Downloads,
        crate::words!("core-repos-source-added"),
        format!("Added the endpoint {name}"),
        EventBody::Notice,
    );
    changed(core)
}

/// Changes a source added by hand: its name and its address. A secret typed into the new
/// address replaces the stored one; one left out, or left as its `{key}`, is kept.
pub fn edit_custom(
    core: &Core,
    install: String,
    name: String,
    url: String,
) -> Result<Reply, CoreError> {
    let row = find_install(core, &install)?;
    if row.repo_url != CUSTOM {
        return Err(CoreError::invalid(
            "endpoint",
            tr("core-repos-only-by-hand", &[]),
        ));
    }
    let (mut def, lifted) = custom_definition(&name, &url, Some(&row.def.kind))?;
    // A secret the address had as a `{key}` must stay one: a value typed in its place would
    // be kept in the open, and only some keys are lifted out of an address by their name.
    for old in &row.def.auth {
        let kept = def.auth.iter().any(|a| a.key == old.key);
        let lifted_again = lifted.iter().any(|(k, _)| *k == old.key);
        if !kept && !lifted_again && !url.contains(&format!("{{{}}}", old.key)) {
            return Err(CoreError::invalid(
                "endpoint url",
                tr(
                    "core-repos-keep-key",
                    &[
                        ("key", old.key.as_str().into()),
                        ("label", old.label.as_str().into()),
                    ],
                ),
            ));
        }
    }
    // A secret the address keeps as its `{key}` keeps the label it was given when added.
    for field in &mut def.auth {
        if let Some(old) = row.def.auth.iter().find(|a| a.key == field.key) {
            *field = old.clone();
        }
    }
    let installed = core.store.read(read_installed)?;
    if installed
        .iter()
        .any(|i| i.install_id != install && i.def.id == def.id)
    {
        return Err(CoreError::invalid(
            "endpoint name",
            tr("core-repos-name-taken", &[("id", def.id.as_str().into())]),
        ));
    }
    let given: Vec<EndpointCredential> = lifted
        .into_iter()
        .map(|(key, value)| EndpointCredential { key, value })
        .collect();
    let values = credentials_for(&def, &given, false)?;
    store_credentials(core, &install, &values)?;
    // A secret the new address no longer has is not left behind in the store.
    for old in &row.def.auth {
        if !def.auth.iter().any(|a| a.key == old.key) {
            core.secrets().delete(&secret_key(&install, &old.key))?;
        }
    }
    let (row_id, row_def, json) = (
        install.clone(),
        def.id.clone(),
        serde_json::to_string(&def).map_err(|e| CoreError::internal(e.to_string()))?,
    );
    core.store.write(move |conn| {
        conn.execute(
            "UPDATE installed_endpoints SET endpoint_id = ?2, definition = ?3 WHERE install_id = ?1",
            params![row_id, row_def, json],
        )?;
        Ok(())
    })?;
    core.bus.say(
        Level::Info,
        Stage::Downloads,
        crate::words!("core-repos-source-changed"),
        format!("Changed the endpoint {}", def.name),
        EventBody::Notice,
    );
    changed(core)
}

/// Installed endpoints saved before #602 can carry their kind under a site's name. This gives
/// each its shape name in place and changes nothing else about it, and the phone's saved
/// sources the same. A row already under a shape name is left alone, so after the first open
/// this does nothing. Returns how many were renamed.
pub fn rename_old_kinds(core: &Core) -> Result<usize, CoreError> {
    // Read first: after the first open nothing is left to rename, and a write would take the
    // store's lock on every open, a CLI's beside a running desktop's.
    let old = core.store.read(|conn| {
        let mut n = 0i64;
        for (old, _) in crate::releases::kinds::RENAMED {
            n += conn.query_row(
                "SELECT COUNT(*) FROM installed_endpoints
                 WHERE json_valid(definition) AND json_extract(definition, '$.kind') = ?1",
                params![old],
                |r| r.get::<_, i64>(0),
            )?;
        }
        Ok(n)
    })?;
    let mut renamed = if old == 0 {
        0
    } else {
        rewrite_old_kinds(core)?
    };
    renamed += crate::phone::rename_old_kinds(core)?;
    if renamed > 0 {
        core.bus.info(
            Stage::Downloads,
            format!("Gave {renamed} download sources the new name of their kind"),
            EventBody::Notice,
        );
    }
    Ok(renamed)
}

fn rewrite_old_kinds(core: &Core) -> Result<usize, CoreError> {
    core.store.write(|conn| {
        let mut n = 0;
        for (old, now) in crate::releases::kinds::RENAMED {
            n += conn.execute(
                "UPDATE installed_endpoints SET definition = json_set(definition, '$.kind', ?2)
                 WHERE json_valid(definition) AND json_extract(definition, '$.kind') = ?1",
                params![old, now],
            )?;
        }
        Ok(n)
    })
}

/// Sources added by hand before #385 can still hold a secret in their stored address. This
/// lifts it into the secret store as adding one does now, and the address keeps the
/// placeholder. An address with nothing left to lift is untouched, so after the first start
/// this does nothing.
pub fn lift_stored_secrets(core: &Core) -> Result<(), CoreError> {
    let installed = core.store.read(read_installed)?;
    let mut any = false;
    let mut first_error = None;
    for row in installed.into_iter().filter(|r| r.repo_url == CUSTOM) {
        let (template, auth, lifted) = lift_secrets(&row.def.url);
        if lifted.is_empty() {
            continue;
        }
        let mut def = row.def.clone();
        def.url = template;
        // A field the address already had keeps the label it was given when added.
        def.auth = auth
            .into_iter()
            .map(|f| {
                row.def
                    .auth
                    .iter()
                    .find(|a| a.key == f.key)
                    .cloned()
                    .unwrap_or(f)
            })
            .collect();
        // The secret is stored before the address loses it, so nothing is lost on a crash.
        // One source that fails does not hold back the rest.
        match lift_one(core, &row, &def, &lifted) {
            Ok(()) => any = true,
            Err(e) => first_error = first_error.or(Some(e)),
        }
    }
    if any {
        core.bus.info(
            Stage::Downloads,
            "Moved a secret in a source added by hand to the secret store",
            EventBody::Notice,
        );
        crate::downloads::repo_endpoints_changed(core);
    }
    first_error.map_or(Ok(()), Err)
}

/// Stores one source's lifted secrets, then rewrites its address.
fn lift_one(
    core: &Core,
    row: &InstallRow,
    def: &Definition,
    lifted: &[(String, String)],
) -> Result<(), CoreError> {
    store_credentials(core, &row.install_id, lifted)?;
    let row_id = row.install_id.clone();
    let json = serde_json::to_string(def).map_err(|e| CoreError::internal(e.to_string()))?;
    core.store.write(move |conn| {
        conn.execute(
            "UPDATE installed_endpoints SET definition = ?2 WHERE install_id = ?1",
            params![row_id, json],
        )?;
        Ok(())
    })
}

// The desktop's anirss ------------------------------------------------------------------

/// The kinds anirss itself can search, under their shape names; every other kind is
/// AniBeam's own. `spec_of` sends each under the name anirss knows it by
/// (`kinds::anirss_kind`).
const ANIRSS_KINDS: [&str; 3] = [crate::releases::kinds::FILTERED_RSS, "rss", "torznab"];

/// What AniBeam last wrote into anirss: endpoint name to a hash of the spec it sent. anirss
/// lists its endpoints with their secrets masked, so a listed address can never be compared
/// with the one AniBeam holds; the hash says whether anything changed since.
type Synced = std::collections::BTreeMap<String, String>;

fn read_synced(core: &Core) -> Synced {
    core.store
        .read(|c| crate::store::settings::get(c, crate::store::settings::ANIRSS_SYNCED_ENDPOINTS))
        .ok()
        .flatten()
        .unwrap_or_default()
}

/// Why the desktop does not search an installed endpoint, when it does not: anirss has its own
/// of that name from its config, cannot search the kind, does not list
/// [`ENDPOINT_WRITES`](crate::downloads::anirss::ENDPOINT_WRITES), whatever its version says,
/// or is one the desktop never writes into (a server the person runs, a sandbox run).
/// None on the phone, and before the desktop has read its anirss.
///
/// Only the anirss engine has these limits. AniBeam's own engine searches every installed
/// endpoint itself (`Native::for_core`), whatever anirss's version, kinds or hand-written
/// endpoints, so there is nothing to note then. The Downloads page still lists its endpoints
/// from anirss; that is its own item in #297.
fn note_for(
    engine: DownloadsEngine,
    view: Option<&crate::downloads::AnirssView>,
    def: &Definition,
    synced: &Synced,
) -> Option<String> {
    if engine == DownloadsEngine::Native {
        return None;
    }
    let view = view?;
    if !ANIRSS_KINDS.contains(&def.kind.as_str()) {
        return Some(tr(
            "core-repos-note-kind",
            &[("kind", def.kind.as_str().into())],
        ));
    }
    let listed = view.endpoints.contains(&def.id);
    let by_hand = || tr("core-repos-note-by-hand", &[("id", def.id.as_str().into())]);
    match view.sync {
        // A server the person runs keeps its own config, and searches what it lists: once
        // they have added the endpoint there, there is nothing to note (#499).
        EndpointSync::Theirs => {
            (!listed).then(|| tr("core-repos-note-theirs", &[("id", def.id.as_str().into())]))
        }
        // A sandbox run reads the owner's anirss as it is, notes and all, and adds nothing.
        EndpointSync::Sandbox if listed => (!synced.contains_key(&def.id)).then(by_hand),
        EndpointSync::Sandbox => Some(tr(
            "core-repos-note-sandbox",
            &[("id", def.id.as_str().into())],
        )),
        EndpointSync::Syncs if !view.endpoint_writes => Some(tr(
            "core-repos-note-old",
            &[
                ("version", view.version.to_string().into()),
                (
                    "min",
                    crate::downloads::anirss::MIN_VERSION.to_string().into(),
                ),
            ],
        )),
        EndpointSync::Syncs => (listed && !synced.contains_key(&def.id)).then(by_hand),
    }
}

fn spec_of(e: &crate::releases::Endpoint) -> serde_json::Value {
    let kind = crate::releases::kinds::anirss_kind(&e.kind);
    let mut spec = serde_json::json!({"name": e.name, "kind": kind, "url": e.url});
    if e.kind == crate::releases::kinds::FILTERED_RSS {
        spec["category"] = e.category.clone().into();
        spec["filter"] = e.filter.clone().into();
    }
    spec
}

fn hash_of(spec: &serde_json::Value) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(spec.to_string().as_bytes())[..16]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Puts the installed endpoints into the desktop's anirss, which searches only what its own
/// config lists: each enabled one with its credentials, under its id. Only names AniBeam
/// wrote are ever changed or removed; one anirss already had from its config stays as it was.
/// anirss keeps the address as it is given, so a credential lands in its config.toml, the way
/// a hand-written one does today; the secret store stays the record AniBeam reads. A kind
/// anirss cannot search (`json-feed`) stays out, and an anirss whose `/health` does not list
/// [`ENDPOINT_WRITES`](crate::downloads::anirss::ENDPOINT_WRITES) is left alone, whatever
/// its version says.
/// `have` is anirss's list as just read; true when anirss was changed and its list needs
/// reading again.
pub(crate) async fn sync_anirss(
    core: &Core,
    client: &crate::downloads::anirss::AnirssClient,
    health: &crate::downloads::anirss::Health,
    have: &[SearchEndpoint],
) -> bool {
    if !health.has(crate::downloads::anirss::ENDPOINT_WRITES) {
        return false;
    }
    let want: Vec<Runnable> = runnable(core)
        .into_iter()
        .filter(|r| ANIRSS_KINDS.contains(&r.endpoint.kind.as_str()))
        .collect();
    let mut synced = read_synced(core);
    let before = synced.clone();
    let mut changed = false;
    for r in &want {
        let e = &r.endpoint;
        let listed = have.iter().any(|h| h.name == e.name);
        if listed && !synced.contains_key(&e.name) {
            continue;
        }
        let spec = spec_of(e);
        let hash = hash_of(&spec);
        if listed && synced.get(&e.name) == Some(&hash) {
            continue;
        }
        match client.put_endpoint(spec).await {
            Ok(()) => {
                changed = true;
                synced.insert(e.name.clone(), hash);
            }
            Err(err) => tracing::warn!("anirss refused the endpoint {}: {err}", e.name),
        }
    }
    for name in before.keys() {
        if want.iter().any(|r| &r.endpoint.name == name) {
            continue;
        }
        if have.iter().any(|h| &h.name == name) {
            if let Err(err) = client.delete_endpoint(name).await {
                tracing::warn!("anirss kept the endpoint {name}: {err}");
                continue;
            }
            changed = true;
        }
        synced.remove(name);
    }
    if synced != before {
        let _ = core.store.write(move |c| {
            crate::store::settings::set(c, crate::store::settings::ANIRSS_SYNCED_ENDPOINTS, &synced)
        });
    }
    changed
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
    /// Each secret's placeholder (`{token}`), its value as the address carries it, and the
    /// value itself.
    pub secrets: Vec<Secret>,
    /// The address with every field but the secrets filled in: `{query}` and each secret's
    /// placeholder stay.
    pub template: String,
    /// The release groups its definition trusts (`Definition::trusted_groups`).
    pub trusted_groups: Vec<String>,
}

pub struct Secret {
    pub placeholder: String,
    pub encoded: String,
    pub raw: String,
}

/// A secret shorter than this could stand for any short value in a feed's address (a filter
/// `f=1` beside a `uid=1`), so a feed carrying one is never searched for it.
const SHORTEST_FOUND: usize = 8;

impl Runnable {
    /// The subscription `spec` made from this endpoint for `query`, as it is kept on disk
    /// (#149): each secret back as its placeholder. The secrets are found again in the feed
    /// the search built, which may be one of several shapes, so when one could still be in it
    /// (a short value, or any trace of one after the swap) the feed kept is the plain search
    /// feed built from `template`, which never held a secret.
    pub fn kept(&self, spec: &SubscriptionSpec, query: &str) -> SubscriptionSpec {
        let masked = self.template_of(&spec.feed_url);
        if !self.traced(&masked) {
            return SubscriptionSpec {
                feed_url: masked,
                ..spec.clone()
            };
        }
        let mut plain = self.endpoint.clone();
        plain.url = self.template.clone();
        let (feed_url, must_contain, excluded) =
            crate::releases::endpoints::subscription_feed_resolved(&plain, query, None);
        SubscriptionSpec {
            feed_url,
            must_contain,
            must_not_contain: excluded.join("|"),
            endpoint_name: spec.endpoint_name.clone(),
        }
    }

    /// A feed a rule kept before #149 with this endpoint's secrets in its address, as it
    /// is kept now: the same swap and trace check as [`Runnable::kept`]. None when the feed
    /// is on another host, holds none of these secrets, or could still hold one after the
    /// swap; such a rule is left as it is.
    pub fn rekept_feed(&self, feed: &str) -> Option<String> {
        if self.secrets.is_empty() || index::host_of(feed) != self.host {
            return None;
        }
        let masked = self.template_of(feed);
        (masked != feed && !self.traced(&masked)).then_some(masked)
    }

    /// Whether a secret could still be in `masked`: one too short to find for sure, or any
    /// trace of one after the swap.
    fn traced(&self, masked: &str) -> bool {
        self.secrets.iter().any(|s| {
            s.raw.chars().count() < SHORTEST_FOUND
                || [s.raw.as_str(), &s.encoded, &s.encoded.replace("%20", "+")]
                    .iter()
                    .any(|v| !v.is_empty() && masked.contains(v))
        })
    }

    /// `url` with each secret that stands as a whole part of it back as its placeholder.
    fn template_of(&self, url: &str) -> String {
        // Each value as `filled_url` encodes it, and as a feed rebuilt with `urlencode` does,
        // a space as `+`.
        let mut forms: Vec<(&str, String)> = self
            .secrets
            .iter()
            .filter(|s| !s.encoded.is_empty())
            .flat_map(|s| {
                [
                    (s.placeholder.as_str(), s.encoded.clone()),
                    (s.placeholder.as_str(), s.encoded.replace("%20", "+")),
                ]
            })
            .collect();
        // The longest first, so a value inside another is not cut out of it.
        forms.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
        let mut url = url.to_string();
        for (placeholder, value) in forms {
            url = replace_whole(&url, &value, placeholder);
        }
        url
    }

    /// A kept address with each placeholder filled with the stored secret.
    fn fill(&self, template: &str) -> String {
        let mut url = template.to_string();
        for s in &self.secrets {
            url = url.replace(&s.placeholder, &s.encoded);
        }
        url
    }
}

/// `url` with every `value` that stands as a whole part of the address (a query value, a
/// path segment, a login) replaced by `with`; a value that is only part of something else,
/// a search word say, stays.
fn replace_whole(url: &str, value: &str, with: &str) -> String {
    let opens = |c: char| matches!(c, '=' | '/' | ':' | '@' | '?' | '&' | ';' | ',');
    let closes = |c: char| matches!(c, '&' | '#' | '/' | '@' | ':' | '?' | ';' | ',');
    let mut out = String::with_capacity(url.len());
    let mut rest = url;
    while let Some(at) = rest.find(value) {
        let before = rest[..at].chars().last().or_else(|| out.chars().last());
        let after = rest[at + value.len()..].chars().next();
        out.push_str(&rest[..at]);
        if before.is_some_and(opens) && after.is_none_or(closes) {
            out.push_str(with);
        } else {
            out.push_str(value);
        }
        rest = &rest[at + value.len()..];
    }
    out.push_str(rest);
    out
}

/// The subscription `spec` from the installed endpoint it names, as a subscription keeps it
/// (`Runnable::kept`). A spec from anywhere else stays as it is.
pub fn kept_spec(core: &Core, spec: &SubscriptionSpec, query: &str) -> SubscriptionSpec {
    match fillers(core)
        .iter()
        .find(|r| r.endpoint.name == spec.endpoint_name)
    {
        Some(r) => r.kept(spec, query),
        None => spec.clone(),
    }
}

/// Rewrites the subscriptions kept before #149, the secrets of an installed endpoint in
/// their feed, to the placeholder form, once per install (#463). A rule is matched to the
/// endpoint whose secrets its feed holds ([`Runnable::rekept_feed`]), the one whose template
/// it fits best when two would do, and keeps its name and what it has taken. One that
/// matches no endpoint (a pasted feed, a rotated key, a removed endpoint) stays as it is.
/// `store` is changed in one atomic write ([`RuleStore::edit`](crate::releases::rules::RuleStore::edit)).
///
/// The run is marked done only when it could see every secret: `lifted` says this launch's
/// lift of hand-added sources' secrets went through, and every installed endpoint has to
/// have its credentials. Otherwise it runs again on the next launch, which rewrites only
/// what this one could not. How many rules were rewritten.
pub fn rekeep_old_rules(
    core: &Core,
    store: &crate::releases::rules::RuleStore,
    lifted: bool,
) -> Result<usize, CoreError> {
    use crate::store::settings::{self, PHONE_RULES_REKEPT};
    if core
        .store
        .read(|c| settings::get::<bool>(c, PHONE_RULES_REKEPT))?
        .unwrap_or(false)
    {
        return Ok(0);
    }
    let installed = core.store.read(read_installed)?;
    let fillers = fillers(core);
    let mut ids: Vec<&str> = installed
        .iter()
        .filter(|r| crate::releases::endpoints::VALID_KINDS.contains(&r.def.kind.as_str()))
        .map(|r| r.def.id.as_str())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let complete = lifted && ids.len() == fillers.len();
    let rewritten = store.edit(|rules| {
        let mut n = 0;
        for rule in rules.iter_mut().filter(|r| r.endpoint.is_empty()) {
            let best = fillers
                .iter()
                .filter_map(|f| f.rekept_feed(&rule.feed_url).map(|kept| (f, kept)))
                .max_by_key(|(f, kept)| shared_start(kept, &f.template));
            if let Some((f, kept)) = best {
                rule.feed_url = kept;
                rule.endpoint = f.endpoint.name.clone();
                n += 1;
            }
        }
        (n, n > 0)
    })?;
    if complete {
        core.store
            .write(|c| settings::set(c, PHONE_RULES_REKEPT, &true))?;
    }
    Ok(rewritten)
}

/// The endpoint a definition runs as at `url`, its address filled in. A category or filter
/// the definition leaves out or leaves empty is anirss's default, as anirss reads it
/// (`raw.get("category") or "1_0"`): every anime upload, no filter (#546).
fn endpoint_for(def: &Definition, url: &str) -> crate::releases::Endpoint {
    let mut endpoint = crate::releases::Endpoint::new(&def.id, &def.kind, url);
    if let Some(c) = def
        .category
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
    {
        endpoint.category = c.to_string();
    }
    if let Some(f) = def
        .filter
        .as_deref()
        .map(str::trim)
        .filter(|f| !f.is_empty())
    {
        endpoint.filter = f.to_string();
    }
    endpoint.json = def.json.clone();
    endpoint.quirks = def.quirks.clone();
    endpoint
}

/// How many characters `a` and `b` start with in common.
fn shared_start(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

/// Whether `url` holds a placeholder: `{` then letters, digits, `.`, `_` or `-`, then `}`.
fn has_placeholder(url: &str) -> bool {
    let mut rest = url;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        if let Some(close) = after.find('}')
            && close > 0
            && after[..close]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        {
            return true;
        }
        rest = after;
    }
    false
}

/// A kept address ready to read, its placeholders filled from the credentials of `endpoint`
/// among `fillers`. None when a placeholder is left that the endpoint can no longer fill (it
/// was removed, or its credentials were), so the address never goes out half filled.
pub fn filled_feed(fillers: &[Runnable], endpoint: &str, url: &str) -> Option<String> {
    if !has_placeholder(url) {
        return Some(url.to_string());
    }
    let filled = fillers
        .iter()
        .find(|r| r.endpoint.name == endpoint)?
        .fill(url);
    (!has_placeholder(&filled)).then_some(filled)
}

/// The enabled installed endpoints, in order, with every credential they need. One whose
/// credentials are missing is left out.
pub fn runnable(core: &Core) -> Vec<Runnable> {
    installed_runnable(core, false)
}

/// Every installed endpoint with its credentials, switched off or not: what fills a kept
/// subscription feed, which keeps being read while its endpoint is off for search.
pub fn fillers(core: &Core) -> Vec<Runnable> {
    installed_runnable(core, true)
}

fn installed_runnable(core: &Core, with_off: bool) -> Vec<Runnable> {
    let Ok(rows) = core.store.read(read_installed) else {
        return Vec::new();
    };
    let secrets = core.secrets();
    let mut out: Vec<Runnable> = Vec::new();
    for row in rows.into_iter().filter(|r| with_off || r.enabled) {
        if !crate::releases::endpoints::VALID_KINDS.contains(&row.def.kind.as_str()) {
            continue;
        }
        // Two repos offering the same id: the first installed wins the name.
        if out.iter().any(|r| r.endpoint.name == row.def.id) {
            continue;
        }
        let raw = |key: &str| {
            secrets
                .get(&secret_key(&row.install_id, key), None)
                .ok()
                .flatten()
                .map(|(v, _)| v)
        };
        let Some(values) = row.def.filled_values(raw) else {
            continue;
        };
        let url = row.def.fill_with(&values, |_| true);
        let template = row.def.fill_with(&values, |f| !f.secret);
        let kept = values
            .iter()
            .filter(|(field, _)| field.secret)
            .map(|(field, encoded)| Secret {
                placeholder: format!("{{{}}}", field.key),
                encoded: encoded.clone(),
                raw: raw(&field.key).unwrap_or_default(),
            })
            .collect();
        let endpoint = endpoint_for(&row.def, &url);
        out.push(Runnable {
            endpoint,
            shown_url: row.def.url.clone(),
            // The filled address's host: a torznab `{host}` is the person's own server.
            host: index::host_of(&url),
            per_minute: row.def.rate.map(|r| r.per_minute),
            secrets: kept,
            template,
            trusted_groups: row.def.trusted_groups.clone(),
        });
    }
    out
}

/// Whether the installed endpoint of this name narrows a subscription in its rule, as its
/// definition says (`Quirks::narrows_in_rule`). Reads the definitions alone, no credentials.
pub fn installed_narrows(core: &Core, name: &str) -> bool {
    core.store
        .read(read_installed)
        .unwrap_or_default()
        .iter()
        .find(|r| r.def.id == name)
        .is_some_and(|r| r.def.quirks.narrows_in_rule())
}

/// The gap a definition's `rate` asks between two requests to its host.
fn rate_gap(per_minute: u32) -> Duration {
    Duration::from_secs_f64(60.0 / f64::from(per_minute.max(1)))
}

/// Wraps the core's client so every host waits the floor between requests, and a host with a
/// slower rate from its definition waits that.
pub fn paced_http(core: &Core, runnable: &[Runnable]) -> Arc<dyn crate::net::Http> {
    set_fronts(core);
    let gaps = runnable
        .iter()
        .filter_map(|r| r.per_minute.map(|n| (r.host.clone(), rate_gap(n))))
        .collect();
    Arc::new(Paced::new(core.http.clone(), gaps))
}

/// Hands the client the hosts the installed definitions front, switched off or not, since
/// the front is the site's: each host's requests then carry its front's name (`net::front`).
/// Read again before every endpoint fetch, so a definition installed, updated or removed
/// changes what is fronted from the next request on.
fn set_fronts(core: &Core) {
    // A store that cannot be read keeps the table it had, rather than unfronting every host.
    let Ok(rows) = core.store.read(read_installed) else {
        return;
    };
    let fronts = rows
        .into_iter()
        .filter_map(|r| Some((r.def.host(), r.def.front.clone()?.to_ascii_lowercase())))
        .filter(|(host, _)| !host.is_empty() && !host.contains('{'))
        .collect();
    core.http.set_fronts(fronts);
}

/// The core's client for an endpoint fetch outside the search (a feed read again after
/// anirss's search, a subscription's feed, a `.torrent`): paced like the search, to every
/// installed definition's rate, switched off or not, since the rate is the site's. Reads no
/// credentials.
pub fn endpoint_http(core: &Core) -> Paced {
    set_fronts(core);
    let secrets = core.secrets();
    let gaps = core
        .store
        .read(read_installed)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|r| {
            let rate = r.def.rate?;
            // The host the requests go to, as the search fills it: a torznab `{host}` is
            // the person's own server, never the literal placeholder.
            let filled = r.def.filled_url(|key| {
                secrets
                    .get(&secret_key(&r.install_id, key), None)
                    .ok()
                    .flatten()
                    .map(|(v, _)| v)
            });
            let host = filled.map_or_else(|| r.def.host(), |url| index::host_of(&url));
            Some((host, rate_gap(rate.per_minute)))
        })
        .collect();
    Paced::new(core.http.clone(), gaps)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::downloads::AnirssView;
    use crate::downloads::anirss::{AnirssClient, Health};
    use crate::net::FakeHttp;
    use crate::trackers::Secrets;
    use crate::{CorePaths, core::Core};

    #[test]
    fn the_native_engine_leaves_no_endpoint_out() {
        let def: Definition = serde_json::from_value(
            serde_json::json!({"id": "feed", "name": "feed", "version": 1,
                "kind": "json-feed", "url": "https://t.example/json?q={query}"}),
        )
        .unwrap();
        let synced = Synced::new();
        let current = view("0.6.0", true, &[]);
        let old = view("0.5.0", false, &[]);
        let by_hand = view("0.6.0", true, &["feed"]);
        assert!(
            note_for(DownloadsEngine::Anirss, Some(&current), &def, &synced)
                .is_some_and(|n| n.contains("cannot search the json-feed kind"))
        );
        assert!(note_for(DownloadsEngine::Anirss, Some(&old), &def, &synced).is_some());
        // The native engine searches the installed endpoint itself, so none of the three
        // anirss limits applies to it.
        for view in [&current, &old, &by_hand] {
            assert_eq!(
                note_for(DownloadsEngine::Native, Some(view), &def, &synced),
                None
            );
        }
    }

    fn view(version: &str, endpoint_writes: bool, endpoints: &[&str]) -> AnirssView {
        AnirssView {
            version: version.to_string(),
            endpoint_writes,
            endpoints: endpoints.iter().map(|e| e.to_string()).collect(),
            sync: EndpointSync::Syncs,
        }
    }

    /// An anirss the desktop never syncs endpoints into, a server the person runs (Remote)
    /// or the owner's own behind a sandbox run, is not searched for one it lacks, whatever
    /// it lists: the note says what to do there instead of asking for an update (#499).
    #[test]
    fn the_endpoint_note_says_when_the_desktop_never_syncs_into_that_anirss() {
        let def: Definition = serde_json::from_value(
            serde_json::json!({"id": "mine", "name": "mine", "version": 1,
                "kind": "rss", "url": "https://a.example/rss?q={query}"}),
        )
        .unwrap();
        let synced = Synced::new();
        let at = |sync, writes, endpoints: &[&str]| AnirssView {
            sync,
            ..view("0.7.0", writes, endpoints)
        };
        let note = |v: AnirssView| note_for(DownloadsEngine::Anirss, Some(&v), &def, &synced);
        // A server the person runs: to its own config, whether or not it lists writes.
        for writes in [true, false] {
            let n = note(at(EndpointSync::Theirs, writes, &[])).unwrap();
            assert!(n.contains("add mine to that anirss's own config"), "{n}");
            assert!(!n.contains("update"), "{n}");
        }
        // Once the person has added it there, it is searched and the note goes.
        assert_eq!(note(at(EndpointSync::Theirs, false, &["mine"])), None);
        // A sandbox run leaves the owner's anirss alone: searched only if it is there, and
        // one there by hand says so, as the owner's own run does.
        let n = note(at(EndpointSync::Sandbox, true, &[])).unwrap();
        assert!(n.contains("never writes into anirss's config"), "{n}");
        let n = note(at(EndpointSync::Sandbox, true, &["mine"])).unwrap();
        assert!(n.contains("written by hand"), "{n}");
        let mut ours = Synced::new();
        ours.insert("mine".to_string(), String::new());
        let v = at(EndpointSync::Sandbox, true, &["mine"]);
        assert_eq!(
            note_for(DownloadsEngine::Anirss, Some(&v), &def, &ours),
            None
        );
        // A kind anirss cannot search says so first, wherever anirss runs.
        let feed: Definition =
            serde_json::from_value(serde_json::json!({"id": "t", "name": "t", "version": 1,
                "kind": "json-feed", "url": "https://a.example/json?q={query}"}))
            .unwrap();
        let n = note_for(
            DownloadsEngine::Anirss,
            Some(&at(EndpointSync::Theirs, true, &[])),
            &feed,
            &synced,
        )
        .unwrap();
        assert!(n.contains("cannot search the json-feed kind"), "{n}");
    }

    fn health(version: &str, features: &[&str]) -> Health {
        Health {
            version: version.to_string(),
            features: features.iter().map(|f| f.to_string()).collect(),
        }
    }

    /// Whether anirss takes endpoints is what it lists, not its version: a checkout that
    /// reports `dev`, or a version no parser reads, gets them when it lists
    /// `endpoint_writes`, and a 0.6 that does not list it gets none (#476).
    #[test]
    fn the_endpoint_note_follows_the_feature_anirss_lists() {
        let def: Definition = serde_json::from_value(
            serde_json::json!({"id": "mine", "name": "mine", "version": 1,
                "kind": "rss", "url": "https://a.example/rss?q={query}"}),
        )
        .unwrap();
        let synced = Synced::new();
        for version in ["dev", "0.7.0+local", ""] {
            assert_eq!(
                note_for(
                    DownloadsEngine::Anirss,
                    Some(&view(version, true, &[])),
                    &def,
                    &synced
                ),
                None,
                "{version}"
            );
        }
        let note = note_for(
            DownloadsEngine::Anirss,
            Some(&view("0.6.0", false, &[])),
            &def,
            &synced,
        )
        .unwrap();
        // It names the anirss running, and no threshold of its own.
        assert!(
            note.starts_with("anirss 0.6.0 cannot take endpoints"),
            "{note}"
        );
        assert!(!note.contains("or later"), "{note}");
    }

    /// Installs `def` as a repo's endpoint would be, straight into the store.
    fn install_raw(core: &Core, def: serde_json::Value, enabled: bool) {
        let id = def["id"].as_str().unwrap().to_string();
        let json = def.to_string();
        core.store
            .write(move |conn| {
                conn.execute(
                    "INSERT INTO installed_endpoints (install_id, repo_url, endpoint_id, version, definition, enabled, position)
                     VALUES (?1, 'https://repo.example/index.json', ?1, 1, ?2, ?3,
                             (SELECT COALESCE(MAX(position), 0) + 1 FROM installed_endpoints))",
                    params![id, json, enabled],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn fake_core(dir: &std::path::Path) -> (Arc<Core>, Arc<FakeHttp>) {
        let paths = CorePaths::under(dir);
        let http = FakeHttp::new();
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            http.clone(),
            Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        (core, http)
    }

    #[test]
    fn a_host_is_fronted_only_as_its_definition_says() {
        let dir = tempfile::tempdir().unwrap();
        let (core, http) = fake_core(dir.path());
        install_raw(
            &core,
            serde_json::json!({"id": "tracker", "name": "Tracker", "version": 1,
                "kind": "rss", "url": "https://tracker.example/?q={query}",
                "front": "edge.example"}),
            // Switched off, the front is still the site's.
            false,
        );
        install_raw(
            &core,
            serde_json::json!({"id": "feed", "name": "Feed", "version": 1,
                "kind": "rss", "url": "https://feed.example/?q={query}"}),
            true,
        );
        let _ = endpoint_http(&core);
        assert_eq!(
            http.fronts(),
            [("tracker.example".to_string(), "edge.example".to_string())]
        );
        // The search's client reads them again too, so a removed definition's front goes.
        core.store
            .write(|c| {
                c.execute(
                    "DELETE FROM installed_endpoints WHERE endpoint_id = 'tracker'",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        let _ = paced_http(&core, &runnable(&core));
        assert!(http.fronts().is_empty());
    }

    #[test]
    fn best_fit_trusts_the_groups_the_enabled_definitions_name() {
        let dir = tempfile::tempdir().unwrap();
        let (core, _) = fake_core(dir.path());
        let def = |id: &str, groups: &[&str]| {
            serde_json::json!({"id": id, "name": id, "version": 1, "kind": "rss",
                "url": format!("https://{id}.example/?q={{query}}"), "trusted_groups": groups})
        };
        install_raw(&core, def("one", &["GroupA", "GroupB"]), true);
        install_raw(&core, def("two", &["groupb", "GroupC"]), true);
        install_raw(&core, def("off", &["GroupD"]), false);
        let native = crate::releases::native::Native::for_core(&core, Vec::new(), None);
        assert_eq!(
            native.bestfit.preferred_groups,
            ["GroupA", "GroupB", "GroupC"],
            "in order, each once, and none from a switched off endpoint"
        );
        // With none installed, the core trusts no group of its own.
        let empty = tempfile::tempdir().unwrap();
        let (bare, _) = fake_core(empty.path());
        assert!(
            crate::releases::native::Native::for_core(&bare, Vec::new(), None)
                .bestfit
                .preferred_groups
                .is_empty()
        );
    }

    fn custom(core: &Core, name: &str, url: &str) {
        add_custom(core, name.into(), url.into(), Vec::new()).unwrap();
    }

    #[test]
    fn a_bare_source_added_by_hand_before_the_rename_can_still_be_edited() {
        let dir = tempfile::tempdir().unwrap();
        let paths = CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            FakeHttp::new(),
            Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        custom(&core, "mirror", "https://tracker.example/rss?q={query}");
        // As the rename left a bare address added by hand before #602.
        core.store
            .write(|conn| {
                let def: String =
                    conn.query_row("SELECT definition FROM installed_endpoints", [], |r| {
                        r.get(0)
                    })?;
                let mut def: Definition = serde_json::from_str(&def).unwrap();
                def.kind = crate::releases::kinds::FILTERED_RSS.into();
                def.url = "https://tracker.example/".into();
                conn.execute(
                    "UPDATE installed_endpoints SET definition = ?1",
                    params![serde_json::to_string(&def).unwrap()],
                )?;
                Ok(())
            })
            .unwrap();
        let install = core
            .store
            .read(read_installed)
            .unwrap()
            .remove(0)
            .install_id;
        edit_custom(
            &core,
            install.clone(),
            "renamed".into(),
            "https://tracker.example/".into(),
        )
        .unwrap();
        let row = core.store.read(read_installed).unwrap().remove(0);
        assert_eq!(
            (row.def.name.as_str(), row.def.kind.as_str()),
            ("renamed", crate::releases::kinds::FILTERED_RSS)
        );
        // A new source by hand still needs its {query}.
        assert!(
            add_custom(
                &core,
                "other".into(),
                "https://other.example/".into(),
                vec![]
            )
            .is_err()
        );
    }

    #[test]
    fn a_secret_stored_in_an_old_address_moves_to_the_secret_store() {
        let dir = tempfile::tempdir().unwrap();
        let paths = CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            FakeHttp::new(),
            Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        custom(&core, "old", "https://a.example/rss?q={query}");
        // As a source added before #385 was stored: the key in the address itself.
        let old_url = "https://a.example/rss?q={query}&rsskey=s3cret";
        core.store
            .write(move |conn| {
                let def: String =
                    conn.query_row("SELECT definition FROM installed_endpoints", [], |r| {
                        r.get(0)
                    })?;
                let mut def: Definition = serde_json::from_str(&def).unwrap();
                def.url = old_url.to_string();
                conn.execute(
                    "UPDATE installed_endpoints SET definition = ?1",
                    params![serde_json::to_string(&def).unwrap()],
                )?;
                Ok(())
            })
            .unwrap();

        lift_stored_secrets(&core).unwrap();
        let row = core.store.read(read_installed).unwrap().remove(0);
        assert_eq!(
            row.def.url,
            "https://a.example/rss?q={query}&rsskey={rsskey}"
        );
        assert_eq!(row.def.auth.len(), 1);
        assert_eq!(row.def.auth[0].key, "rsskey");
        let stored = core
            .secrets()
            .get(&secret_key(&row.install_id, "rsskey"), None)
            .unwrap()
            .map(|(v, _)| v);
        assert_eq!(stored.as_deref(), Some("s3cret"));

        // Once lifted, a second start changes nothing.
        lift_stored_secrets(&core).unwrap();
        assert_eq!(
            core.store.read(read_installed).unwrap().remove(0).def,
            row.def
        );
    }

    #[test]
    fn a_stored_feed_keeps_the_placeholder_and_is_filled_when_read() {
        let dir = tempfile::tempdir().unwrap();
        let paths = CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            FakeHttp::new(),
            Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        custom(
            &core,
            "mine",
            "https://a.example/rss?q={query}&token=t%201secret",
        );
        let spec = |url: &str| SubscriptionSpec {
            feed_url: url.into(),
            must_contain: String::new(),
            must_not_contain: String::new(),
            endpoint_name: "mine".into(),
        };
        let feed = "https://a.example/rss?q=Show+1080p&token=t%201secret";

        // What a subscription keeps on disk: the placeholder, never the token (#149).
        let kept = kept_spec(&core, &spec(feed), "Show 1080p").feed_url;
        assert_eq!(kept, "https://a.example/rss?q=Show+1080p&token={token}");
        // Read again with the credential from the secret store.
        let fillers = fillers(&core);
        assert_eq!(filled_feed(&fillers, "mine", &kept).as_deref(), Some(feed));

        // A feed rebuilt with `urlencode` writes the space as `+`: the token still comes out.
        let rebuilt = "https://a.example/show/feed/?token=t+1secret";
        assert_eq!(
            kept_spec(&core, &spec(rebuilt), "Show").feed_url,
            "https://a.example/show/feed/?token={token}"
        );
        // A shape the swap does not know keeps the plain search feed instead, from the
        // template, never the token.
        let odd = "https://a.example/rss/t%201secret-feed?q=Show";
        assert_eq!(
            kept_spec(&core, &spec(odd), "Show").feed_url,
            "https://a.example/rss?q=Show&token={token}"
        );

        // A feed with nothing to fill is read as it is, whichever endpoint it came from, and
        // so is one with a brace that is no placeholder.
        for plain in [
            "https://b.example/rss?q=Show",
            "https://b.example/rss?q={Show 2}",
        ] {
            assert_eq!(filled_feed(&fillers, "gone", plain).as_deref(), Some(plain));
        }
        // A placeholder nobody can fill any more is not sent out as it is.
        assert_eq!(filled_feed(&fillers, "gone", &kept), None);
        core.shutdown();
    }

    /// A phone subscription as it was kept before #149: the filled feed, no endpoint.
    fn old_rule(name: &str, feed: &str) -> crate::releases::rules::Rule {
        crate::releases::rules::Rule {
            name: name.into(),
            feed_url: feed.into(),
            endpoint: String::new(),
            must_contain: String::new(),
            must_not_contain: String::new(),
            save_path: format!("/x/{name}"),
            enabled: true,
            taken: [format!("{name}-ep1")].into(),
        }
    }

    fn core_with_keyed_endpoint(dir: &std::path::Path) -> Arc<Core> {
        let paths = CorePaths::under(dir);
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            FakeHttp::new(),
            Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        custom(
            &core,
            "mine",
            "https://a.example/rss?q={query}&token=longsecret1",
        );
        core
    }

    #[test]
    fn old_phone_rules_lose_their_secret_once_and_unmatched_ones_stay() {
        let dir = tempfile::tempdir().unwrap();
        let core = core_with_keyed_endpoint(dir.path());
        let store = crate::releases::rules::RuleStore::new(dir.path().join("phone-rules.json"));
        let filled = "https://a.example/rss?q=Show+1080p&token=longsecret1";
        let pasted = "https://b.example/rss?q=Other&key=pastedsecret";
        let rotated = "https://a.example/rss?q=Old&token=oldsecret99";
        store
            .save(&[
                old_rule("Show", filled),
                old_rule("Pasted", pasted),
                old_rule("Rotated", rotated),
            ])
            .unwrap();

        assert_eq!(rekeep_old_rules(&core, &store, true).unwrap(), 1);
        let rules = store.load();
        assert_eq!(rules.len(), 3);
        let show = &rules[0];
        assert_eq!(
            show.feed_url,
            "https://a.example/rss?q=Show+1080p&token={token}"
        );
        assert_eq!(show.endpoint, "mine");
        assert_eq!(show.taken, old_rule("Show", filled).taken);
        // Filled from the secret store, it is the feed it was.
        assert_eq!(
            filled_feed(&fillers(&core), &show.endpoint, &show.feed_url).as_deref(),
            Some(filled)
        );
        // A pasted feed and one whose token was changed since match no endpoint: as they were.
        assert_eq!(rules[1], old_rule("Pasted", pasted));
        assert_eq!(rules[2], old_rule("Rotated", rotated));

        // The next launch does nothing, even with an old rule in the file again.
        let mut again = rules.clone();
        again.push(old_rule("Later", filled));
        store.save(&again).unwrap();
        assert_eq!(rekeep_old_rules(&core, &store, true).unwrap(), 0);
        assert_eq!(store.load(), again);
        core.shutdown();
    }

    #[test]
    fn a_launch_that_could_not_see_every_secret_runs_again() {
        let dir = tempfile::tempdir().unwrap();
        let core = core_with_keyed_endpoint(dir.path());
        let store = crate::releases::rules::RuleStore::new(dir.path().join("phone-rules.json"));
        let filled = "https://a.example/rss?q=Show&token=longsecret1";
        store.save(&[old_rule("Show", filled)]).unwrap();
        // The lift failed this launch: what could be rewritten is, and the next launch
        // looks again.
        assert_eq!(rekeep_old_rules(&core, &store, false).unwrap(), 1);
        store
            .save(&[store.load().remove(0), old_rule("Later", filled)])
            .unwrap();
        assert_eq!(rekeep_old_rules(&core, &store, true).unwrap(), 1);
        assert!(store.load().iter().all(|r| r.endpoint == "mine"));
        core.shutdown();
    }

    #[test]
    fn an_old_rule_goes_to_the_endpoint_its_feed_fits_best() {
        let dir = tempfile::tempdir().unwrap();
        let core = core_with_keyed_endpoint(dir.path());
        // Two endpoints on one server with one key, told apart by their path.
        custom(
            &core,
            "first",
            "https://own.example/1/api?t=search&q={query}&apikey=sharedkey1",
        );
        custom(
            &core,
            "second",
            "https://own.example/2/api?t=search&q={query}&apikey=sharedkey1",
        );
        let store = crate::releases::rules::RuleStore::new(dir.path().join("phone-rules.json"));
        store
            .save(&[old_rule(
                "Show",
                "https://own.example/2/api?t=search&q=Show&apikey=sharedkey1",
            )])
            .unwrap();
        assert_eq!(rekeep_old_rules(&core, &store, true).unwrap(), 1);
        let rule = store.load().remove(0);
        assert_eq!(rule.endpoint, "second");
        assert_eq!(
            rule.feed_url,
            "https://own.example/2/api?t=search&q=Show&apikey={apikey}"
        );
        core.shutdown();
    }

    #[test]
    fn a_rewrite_that_cannot_be_written_leaves_the_file_and_runs_again() {
        let dir = tempfile::tempdir().unwrap();
        let core = core_with_keyed_endpoint(dir.path());
        let path = dir.path().join("phone-rules.json");
        let store = crate::releases::rules::RuleStore::new(path.clone());
        let filled = "https://a.example/rss?q=Show&token=longsecret1";
        store.save(&[old_rule("Show", filled)]).unwrap();
        let before = std::fs::read(&path).unwrap();
        // The new file is written beside the old one first; make that impossible.
        std::fs::create_dir(path.with_extension("json.tmp")).unwrap();

        assert!(rekeep_old_rules(&core, &store, true).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);

        std::fs::remove_dir(path.with_extension("json.tmp")).unwrap();
        assert_eq!(rekeep_old_rules(&core, &store, true).unwrap(), 1);
        assert_eq!(store.load()[0].endpoint, "mine");
        core.shutdown();
    }

    #[test]
    fn a_short_secret_keeps_the_plain_search_feed() {
        let dir = tempfile::tempdir().unwrap();
        let paths = CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            FakeHttp::new(),
            Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        custom(&core, "short", "https://a.example/rss?q={query}&f=1&uid=1");
        let spec = SubscriptionSpec {
            feed_url: "https://a.example/rss?q=Show&f=1&uid=1".into(),
            must_contain: String::new(),
            must_not_contain: String::new(),
            endpoint_name: "short".into(),
        };
        // `f=1` is not the secret: the kept feed is the template's, which only names `uid`.
        assert_eq!(
            kept_spec(&core, &spec, "Show").feed_url,
            "https://a.example/rss?q=Show&f=1&uid={uid}"
        );
        core.shutdown();
    }

    #[test]
    fn a_switched_off_endpoint_still_fills_its_subscriptions() {
        let dir = tempfile::tempdir().unwrap();
        let paths = CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            FakeHttp::new(),
            Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        custom(
            &core,
            "mine",
            "https://a.example/rss?q={query}&token=longsecret",
        );
        core.store
            .write(|c| {
                c.execute("UPDATE installed_endpoints SET enabled = 0", [])?;
                Ok(())
            })
            .unwrap();
        assert!(runnable(&core).is_empty());
        assert_eq!(
            filled_feed(
                &fillers(&core),
                "mine",
                "https://a.example/rss?q=S&token={token}"
            )
            .as_deref(),
            Some("https://a.example/rss?q=S&token=longsecret")
        );
        core.shutdown();
    }

    #[test]
    fn only_a_whole_part_of_the_address_becomes_the_placeholder() {
        // A key that also sits inside the search stays there.
        assert_eq!(
            replace_whole(
                "https://a.example/rss?q=Show+abc1&key=abc1",
                "abc1",
                "{key}"
            ),
            "https://a.example/rss?q=Show+abc1&key={key}"
        );
        assert_eq!(
            replace_whole("https://a.example/rss/abc/feed?q=abcd", "abc", "{key}"),
            "https://a.example/rss/{key}/feed?q=abcd"
        );
        assert_eq!(
            replace_whole("https://me:pw@a.example/rss", "pw", "{login_password}"),
            "https://me:{login_password}@a.example/rss"
        );
        assert_eq!(
            replace_whole("https://a.example/rss?abc&q=x", "abc", "{key}"),
            "https://a.example/rss?{key}&q=x"
        );
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
        custom(
            &core,
            "proxied",
            "http://127.0.0.1:9117/x?apikey=k&q={query}",
        );
        let anirss = AnirssClient::new("http://127.0.0.1:1", "tok", http.clone());

        // anirss has its own search-proxy endpoint from config.toml; AniBeam writes only "mine".
        let theirs = |name: &str, url: &str| SearchEndpoint {
            name: name.into(),
            kind: "rss".into(),
            url: url.into(),
            client_side_refine: false,
            on_demand: None,
            running: None,
            followable: true,
        };
        // An anirss that does not list endpoint writes has no endpoint API, whatever its
        // version says: nothing is sent.
        let have = vec![theirs("proxied", "http://elsewhere/")];
        let without = health("0.6.0", &["qbt_env_session"]);
        assert!(
            !core
                .handle
                .block_on(sync_anirss(&core, &anirss, &without, &have))
        );
        assert!(http.requests().is_empty());

        // One that lists it takes them, even with a version no parser reads.
        let writes = health("dev", &["qbt_env_session", "endpoint_writes"]);
        http.push_json(201, serde_json::json!({}));
        assert!(
            core.handle
                .block_on(sync_anirss(&core, &anirss, &writes, &have))
        );
        let sent = http.requests();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].method, crate::net::Method::Post);
        let body = format!("{:?}", sent[0].body);
        assert!(
            body.contains("\"mine\"") && body.contains("token=t1"),
            "{body}"
        );

        // Listed back with its token masked, as anirss lists it: unchanged, so not sent again.
        let have = vec![
            theirs("proxied", "http://elsewhere/"),
            theirs("mine", "https://a.example/rss?q={query}&token=***"),
        ];
        assert!(
            !core
                .handle
                .block_on(sync_anirss(&core, &anirss, &writes, &have))
        );
        assert_eq!(http.requests().len(), 1);

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
        http.push_json(200, serde_json::json!({}));
        let have = vec![
            theirs("proxied", "http://elsewhere/"),
            theirs("mine", "https://a.example/rss?q={query}&token={token}"),
        ];
        assert!(
            core.handle
                .block_on(sync_anirss(&core, &anirss, &writes, &have))
        );
        let sent = http.requests();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[1].method, crate::net::Method::Delete);
        assert!(sent[1].url.ends_with("/endpoints/mine"));
        // Nothing left to do: nothing is sent.
        let have = vec![theirs("proxied", "http://elsewhere/")];
        assert!(
            !core
                .handle
                .block_on(sync_anirss(&core, &anirss, &writes, &have))
        );
        assert_eq!(http.requests().len(), 2);
        core.shutdown();
    }

    #[test]
    fn a_torznab_endpoint_goes_to_anirss_as_rss() {
        let url = "http://10.0.0.2:9696/1/api?t=search&q={query}&apikey=k";
        let spec = spec_of(&crate::releases::Endpoint::new("indexer", "torznab", url));
        assert_eq!(
            spec,
            serde_json::json!({"name": "indexer", "kind": "rss", "url": url})
        );
        // filtered-rss goes over under anirss's own name for it, with its category and
        // filter; every other kind goes over as itself.
        let filtered = spec_of(&crate::releases::Endpoint::new(
            "example",
            crate::releases::kinds::FILTERED_RSS,
            "https://example.org/",
        ));
        assert_eq!(
            filtered,
            serde_json::json!({"name": "example", "kind": crate::releases::kinds::RENAMED[0].0,
                "url": "https://example.org/", "category": "1_0", "filter": "0"})
        );
        let feed = spec_of(&crate::releases::Endpoint::new(
            "feed",
            "rss",
            "https://example.org/?q={query}",
        ));
        assert_eq!(feed["kind"], "rss");
    }

    #[test]
    fn a_definition_with_an_empty_category_or_filter_searches_as_anirss_would() {
        let mut def = Definition {
            id: "site".to_string(),
            name: "site".to_string(),
            version: 1,
            lang: String::new(),
            kind: crate::releases::kinds::FILTERED_RSS.to_string(),
            url: "https://tracker.example/".to_string(),
            auth: Vec::new(),
            front: None,
            rate: None,
            nsfw: false,
            category: None,
            filter: None,
            min_core: None,
            json: None,
            quirks: Default::default(),
            trusted_groups: Vec::new(),
        };
        let shape = |def: &Definition| {
            let e = endpoint_for(def, &def.url);
            (e.category, e.filter)
        };
        let anirss_default = ("1_0".to_string(), "0".to_string());
        assert_eq!(shape(&def), anirss_default, "left out");
        def.category = Some(String::new());
        def.filter = Some(" ".to_string());
        assert_eq!(shape(&def), anirss_default, "left empty");
        def.category = Some("1_2".to_string());
        def.filter = Some("2".to_string());
        assert_eq!(
            shape(&def),
            ("1_2".to_string(), "2".to_string()),
            "trusted only"
        );
        def.category = Some(" 1_2".to_string());
        def.filter = Some("2 ".to_string());
        assert_eq!(
            shape(&def),
            ("1_2".to_string(), "2".to_string()),
            "spaces around"
        );
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
            "http://127.0.0.1:9117/api/v2.0/indexers/proxied/results/torznab/api?apikey=abc123&t=search&q={query}",
        );
        assert_eq!(
            template,
            "http://127.0.0.1:9117/api/v2.0/indexers/proxied/results/torznab/api?apikey={apikey}&t=search&q={query}"
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
    fn a_login_password_moves_out_of_the_address() {
        let (template, fields, lifted) =
            lift_secrets("https://me:p%40ss+1@t.example/rss?q={query}");
        assert_eq!(
            template,
            "https://me:{login_password}@t.example/rss?q={query}"
        );
        assert_eq!(fields[0].label, "Password");
        assert_eq!(
            lifted,
            vec![("login_password".to_string(), "p@ss+1".to_string())]
        );
        let (template, _, lifted) = lift_secrets("https://tok@t.example/rss?q={query}");
        assert_eq!(template, "https://{login_key}@t.example/rss?q={query}");
        assert_eq!(lifted, vec![("login_key".to_string(), "tok".to_string())]);
        let (template, _, lifted) = lift_secrets("https://tok:@t.example/rss?q={query}");
        assert_eq!(template, "https://{login_key}:@t.example/rss?q={query}");
        assert_eq!(lifted, vec![("login_key".to_string(), "tok".to_string())]);
        // An address already lifted keeps its user name.
        let (template, _, lifted) =
            lift_secrets("https://me:{login_password}@t.example/rss?q={query}");
        assert_eq!(
            template,
            "https://me:{login_password}@t.example/rss?q={query}"
        );
        assert!(lifted.is_empty());
    }

    #[test]
    fn a_login_password_and_a_password_parameter_stay_apart() {
        let (template, _, lifted) = lift_secrets("https://me:a@t.example/rss?q={query}&password=b");
        assert_eq!(
            template,
            "https://me:{login_password}@t.example/rss?q={query}&password={password}"
        );
        assert_eq!(lifted.len(), 2);
    }

    #[test]
    fn a_login_comes_back_filled_and_checks_as_a_custom_source() {
        let (def, lifted) =
            custom_definition("Tracker", "https://me:p%40ss@t.example/rss?q={query}", None)
                .unwrap();
        assert!(!def.url.contains("p%40ss"), "{}", def.url);
        let filled = def
            .filled_url(|k| lifted.iter().find(|(l, _)| l == k).map(|(_, v)| v.clone()))
            .unwrap();
        assert_eq!(filled, "https://me:p%40ss@t.example/rss?q={query}");
    }

    #[test]
    fn other_key_names_move_out_whatever_their_case() {
        let (template, fields, lifted) = lift_secrets(
            "https://t.example/rss?q={query}&RssKey=a&authkey=b&key=c&torrent_pass=d&monkey=e",
        );
        assert_eq!(
            template,
            "https://t.example/rss?q={query}&RssKey={rsskey}&authkey={authkey}&key={key}&torrent_pass={torrent_pass}&monkey=e"
        );
        let keys: Vec<&str> = fields.iter().map(|f| f.key.as_str()).collect();
        assert_eq!(keys, ["rsskey", "authkey", "key", "torrent_pass"]);
        assert_eq!(lifted.len(), 4);
    }

    #[test]
    fn ids_come_from_names() {
        assert_eq!(id_of("proxied"), "proxied");
        assert_eq!(id_of(" My Tracker! "), "my-tracker");
        assert_eq!(id_of("!!"), "");
    }

    #[test]
    fn plain_http_only_for_this_machine() {
        assert!(normal_url("http://10.0.2.2:8000/general.json").is_ok());
        assert!(normal_url("http://[::1]:8080/general.json").is_ok());
        assert!(normal_url("http://example.com/x.json").is_err());
        assert!(normal_url("http://evil.com\\@127.0.0.1/x.json").is_err());
        assert!(normal_url("https://example.com/x.json").is_ok());
        assert!(normal_url("/tmp/general.json").is_ok());
        assert!(normal_url("  ").is_err());
        assert_eq!(
            normal_url("Https://example.com/x.json").unwrap(),
            "https://example.com/x.json"
        );
    }
}
