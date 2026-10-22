//! The phone's own downloads: its sources, its search, and (with the `phone` feature) a
//! torrent engine inside the core. The desktop keeps anirss and qBittorrent; these calls
//! are the phone's, and they block the calling thread rather than run as jobs.

#[cfg(feature = "phone")]
pub mod engine;
pub mod search;

use std::path::PathBuf;

use crate::contract::*;
use crate::core::Core;

/// The library folder the phone downloads into, beside the core's own folders. The app
/// registers the same folder as its one library source.
pub const LIBRARY_DIR: &str = "Anime";

#[derive(Default)]
pub struct Phone {
    #[cfg(feature = "phone")]
    engine: tokio::sync::OnceCell<engine::Engine>,
}

fn sources_path(core: &Core) -> PathBuf {
    PathBuf::from(&core.paths.config_dir).join("phone-sources.json")
}

/// nyaa, which needs no key. erai needs the person's own token, so it is added by them.
pub fn default_sources() -> Vec<PhoneSource> {
    vec![PhoneSource {
        name: "nyaa".into(),
        kind: "nyaa".into(),
        url: "https://nyaa.si/".into(),
        enabled: true,
    }]
}

pub fn sources(core: &Core) -> Vec<PhoneSource> {
    std::fs::read(sources_path(core))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_else(default_sources)
}

pub fn set_sources(core: &Core, sources: Vec<PhoneSource>) -> Result<Reply, CoreError> {
    for s in &sources {
        if s.name.trim().is_empty() {
            return Err(CoreError::invalid("sources", "a source needs a name"));
        }
        if !(s.url.starts_with("http://") || s.url.starts_with("https://")) {
            return Err(CoreError::invalid(
                "sources",
                format!(
                    "{}: the address must start with http:// or https://",
                    s.name
                ),
            ));
        }
        if s.kind != "nyaa" && !s.url.contains("{query}") {
            return Err(CoreError::invalid(
                "sources",
                format!(
                    "{}: the address needs {{query}} where the search goes",
                    s.name
                ),
            ));
        }
    }
    let path = sources_path(core);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| CoreError::internal(e.to_string()))?;
    }
    let bytes =
        serde_json::to_vec_pretty(&sources).map_err(|e| CoreError::internal(e.to_string()))?;
    std::fs::write(&path, bytes).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok(Reply::PhoneSources { sources })
}

pub fn search(core: &Core, query: String, source: Option<String>) -> Result<Reply, CoreError> {
    let all = sources(core);
    let source = match source {
        Some(name) => all.into_iter().find(|s| s.name == name),
        None => all.into_iter().find(|s| s.enabled),
    }
    .ok_or_else(|| CoreError::invalid("source", "no source to search; add one in More"))?;
    if query.trim().is_empty() {
        return Ok(Reply::PhoneResults {
            source: source.name,
            results: Vec::new(),
            best: None,
        });
    }
    let url = search::query_url(&source, &query);
    let http = core.http.clone();
    let entries = core
        .handle
        .block_on(async move { crate::downloads::feed::read(http.as_ref(), &url).await })?;
    let releases = entries
        .iter()
        .map(|e| search::to_release(e, &source.name))
        .collect();
    let (results, best) = search::rank(releases);
    Ok(Reply::PhoneResults {
        source: source.name,
        results,
        best,
    })
}

#[cfg(feature = "phone")]
/// The library folder, from the core's paths: the app keeps the core under `files/core`
/// and the library at `files/Anime`.
fn library_dir(core: &Core) -> PathBuf {
    let data = PathBuf::from(&core.paths.data_dir);
    // `files/core/data` up to `files`.
    data.ancestors()
        .nth(2)
        .map(|files| files.join(LIBRARY_DIR))
        .unwrap_or_else(|| data.join(LIBRARY_DIR))
}

#[cfg(feature = "phone")]
async fn engine(core: &Core) -> Result<&engine::Engine, CoreError> {
    let root = library_dir(core);
    let state = PathBuf::from(&core.paths.state_dir).join("torrents");
    core.phone
        .engine
        .get_or_try_init(|| engine::Engine::open(&root, &state))
        .await
}

#[cfg(feature = "phone")]
fn used_bytes(dir: &std::path::Path) -> u64 {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

#[cfg(feature = "phone")]
pub fn download(
    core: &Core,
    release: Release,
    limit_bytes: Option<u64>,
) -> Result<Reply, CoreError> {
    if let Some(limit) = limit_bytes {
        let used = used_bytes(&library_dir(core));
        let size = crate::downloads::facts::size_bytes(&release.size).unwrap_or(0);
        if used + size > limit {
            return Err(CoreError::invalid(
                "limit",
                format!(
                    "this would take AniBeam past its storage limit ({} of {} GB used)",
                    used / 1_000_000_000,
                    limit / 1_000_000_000
                ),
            ));
        }
    }
    let folder = search::show_name(&release.title);
    core.handle
        .block_on(async { engine(core).await?.add(&release, &folder).await })?;
    core.bus.info(
        Stage::Downloads,
        format!("Downloading {}", release.title),
        EventBody::Notice,
    );
    Ok(Reply::Ok)
}

#[cfg(feature = "phone")]
pub fn torrents(core: &Core) -> Result<Reply, CoreError> {
    core.handle.block_on(async {
        let engine = engine(core).await?;
        Ok(Reply::PhoneTorrents {
            torrents: engine.list().await,
            held: engine.held(),
        })
    })
}

#[cfg(feature = "phone")]
pub fn control(core: &Core, hashes: Vec<String>, action: PhoneAction) -> Result<Reply, CoreError> {
    core.handle
        .block_on(async { engine(core).await?.control(&hashes, action).await })?;
    Ok(Reply::Ok)
}

#[cfg(feature = "phone")]
pub fn set_held(core: &Core, held: bool) -> Result<Reply, CoreError> {
    core.handle
        .block_on(async { engine(core).await?.set_held(held).await })?;
    Ok(Reply::Ok)
}

#[cfg(not(feature = "phone"))]
fn unsupported() -> CoreError {
    CoreError::Unsupported {
        what: "a torrent engine of its own".to_string(),
    }
}

#[cfg(not(feature = "phone"))]
pub fn download(_: &Core, _: Release, _: Option<u64>) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn torrents(_: &Core) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn control(_: &Core, _: Vec<String>, _: PhoneAction) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn set_held(_: &Core, _: bool) -> Result<Reply, CoreError> {
    Err(unsupported())
}
