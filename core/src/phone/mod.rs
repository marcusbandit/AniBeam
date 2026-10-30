//! The phone's own downloads: its sources, its search, and (with the `phone` feature) a
//! torrent engine inside the core. The desktop keeps anirss and qBittorrent; these calls
//! are the phone's, and they block the calling thread rather than run as jobs.

pub mod check;
#[cfg(feature = "phone")]
pub mod engine;

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
    /// Whether the subscription poller runs.
    #[cfg(feature = "phone")]
    polling: std::sync::atomic::AtomicBool,
    /// Held by one poll round at a time.
    #[cfg(feature = "phone")]
    poll_lock: tokio::sync::Mutex<()>,
}

fn sources_path(core: &Core) -> PathBuf {
    PathBuf::from(&core.paths.config_dir).join("phone-sources.json")
}

/// The phone ships with no source of its own (#152): endpoints come from a repo the person
/// adds, or by hand. This list is only what someone saved here before repos existed.
pub fn sources(core: &Core) -> Vec<PhoneSource> {
    std::fs::read(sources_path(core))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
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
        if matches!(s.kind.as_str(), "rss" | "torznab") && !s.url.contains("{query}") {
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

/// What a search tries, in order: the enabled feeds saved before repos, then the installed
/// endpoints. An installed endpoint whose id is a saved feed's name takes that feed's place,
/// so installing nyaa from a repo replaces an old saved one rather than searching it twice.
fn searched(core: &Core) -> Vec<(PhoneSource, crate::releases::Endpoint)> {
    let mut installed = crate::repos::runnable(core);
    let mut out = Vec::new();
    for s in sources(core).into_iter().filter(|s| s.enabled) {
        if let Some(at) = installed.iter().position(|r| r.endpoint.name == s.name) {
            let r = installed.remove(at);
            out.push((installed_source(&r), r.endpoint));
        } else {
            let endpoint = crate::releases::Endpoint::new(&s.name, &s.kind, &s.url);
            out.push((s, endpoint));
        }
    }
    for r in installed {
        out.push((installed_source(&r), r.endpoint));
    }
    out
}

fn installed_source(r: &crate::repos::Runnable) -> PhoneSource {
    PhoneSource {
        name: r.endpoint.name.clone(),
        kind: r.endpoint.kind.clone(),
        url: r.shown_url.clone(),
        enabled: true,
    }
}

/// `searched`'s sources, as the search page lists them.
pub fn search_sources(core: &Core) -> Vec<PhoneSource> {
    searched(core).into_iter().map(|(s, _)| s).collect()
}

/// The sources as the ported anirss endpoints, enabled ones in order, each host held to the
/// pace its repo definition asks for.
fn native(core: &Core) -> crate::releases::native::Native {
    let installed = crate::repos::runnable(core);
    let http = crate::repos::paced_http(core, &installed);
    let endpoints = searched(core).into_iter().map(|(_, e)| e).collect();
    crate::releases::native::Native::new(http, endpoints)
}

/// How long the show search waits for covers before answering without them.
const COVER_WAIT: std::time::Duration = std::time::Duration::from_millis(1500);

/// The show search: AniList's hits for `query` with the covers that came in time, so the phone
/// can list shows before anyone looks at a release.
pub fn find_shows(core: &Core, query: String, limit: u32) -> Result<Reply, CoreError> {
    let query = query.trim().to_string();
    if query.chars().count() < 2 {
        return Ok(Reply::PhoneShows {
            shows: Vec::new(),
            posters: Vec::new(),
        });
    }
    let media = core
        .handle
        .block_on(core.anilist.search(&query, limit.clamp(1, 20)))?;
    let shows: Vec<SearchResult> = media
        .iter()
        .map(crate::metadata::apply::to_result)
        .collect();
    let urls: Vec<String> = shows.iter().filter_map(|s| s.cover_url.clone()).collect();
    // The hits go back after COVER_WAIT at most. Each cover is its own task, so the ones
    // already cached or quick come back and a slow one only loses its own poster; it still
    // finishes into the cache (the cache's sweep bounds it), so the same search shows it
    // next time.
    let fetching: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let images = core.images.clone();
            core.handle
                .spawn(async move { images.ensure(&[url]).await })
        })
        .collect();
    let fetched = core.handle.block_on(async move {
        let deadline = tokio::time::Instant::now() + COVER_WAIT;
        let mut fetched = Vec::new();
        for task in fetching {
            if let Ok(Ok(found)) = tokio::time::timeout_at(deadline, task).await {
                fetched.extend(found);
            }
        }
        fetched
    });
    let posters = shows
        .iter()
        .map(|s| {
            s.cover_url
                .as_ref()
                .and_then(|u| fetched.iter().find(|(k, _)| k == u))
                .and_then(|(_, r)| r.as_ref().ok().cloned())
                .unwrap_or_default()
        })
        .collect();
    Ok(Reply::PhoneShows { shows, posters })
}

/// The desktop's search, on the phone (`SearchReleases`, run by the core's port of
/// anirss): fallback across the sources, seeders filled in where a feed has none,
/// auto-resolution, the refine groups and best fit, and each release's facts.
pub fn search(
    core: &Core,
    query: String,
    endpoint: Option<String>,
    fallback: bool,
) -> Result<Reply, CoreError> {
    let native = native(core);
    let found = core
        .handle
        .block_on(native.search(&query, endpoint.as_deref(), fallback))?;
    let fell_back = found.fell_back();
    let items = if found.items.iter().any(|i| i.seeders.is_none()) {
        core.handle.block_on(native.enrich(found.items.clone()))
    } else {
        found.items.clone()
    };
    let search = if items.is_empty() {
        ReleaseSearch {
            query: found.query.clone(),
            endpoint: found.endpoint.clone(),
            items: Vec::new(),
            groups: Vec::new(),
            best_fit: BestFit {
                available: false,
                query: String::new(),
                item: None,
                profile: Vec::new(),
            },
            tried: found.tried,
            fell_back,
            client_side: false,
            auto_resolution: None,
            facts: Vec::new(),
            show: None,
            fetched_query: found.query,
        }
    } else {
        let a = native.analyze(&found.query, &found.endpoint, &items, true)?;
        let (facts, show) = crate::downloads::annotate(core, &a.query, &a.items);
        ReleaseSearch {
            query: a.query,
            endpoint: found.endpoint,
            items: a.items,
            groups: a.groups,
            best_fit: a.best_fit,
            tried: found.tried,
            fell_back,
            client_side: a.client_side,
            auto_resolution: a.auto_resolution,
            facts,
            show,
            fetched_query: found.query,
        }
    };
    Ok(Reply::PhoneSearch { search })
}

/// One refine step (`RefineReleases`).
pub fn refine(
    core: &Core,
    query: String,
    endpoint: String,
    items: Vec<Release>,
    op: RefineOp,
) -> Result<Reply, CoreError> {
    let native = native(core);
    let mut refinement = core
        .handle
        .block_on(native.apply(&query, &endpoint, &items, &op))?;
    let (facts, show) = crate::downloads::annotate(core, &refinement.query, &refinement.items);
    refinement.facts = facts;
    refinement.show = show;
    Ok(Reply::PhoneRefined { refinement })
}

#[cfg(feature = "phone")]
/// A download or subscription name becomes a folder in the library, so it is one plain
/// path component (anirss `validate_name`).
fn validate_name(name: &str) -> Result<String, CoreError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::invalid("name", "the name cannot be empty"));
    }
    if name.contains(['/', '\\']) {
        return Err(CoreError::invalid(
            "name",
            "the name must not contain / or \\",
        ));
    }
    if name == "." || name == ".." || name.chars().any(|c| (c as u32) < 32 || c as u32 == 127) {
        return Err(CoreError::invalid(
            "name",
            format!("{name:?} is not a folder name"),
        ));
    }
    Ok(name.to_string())
}

#[cfg(feature = "phone")]
/// The folder for releases: the name given, else the show's from the first title, as
/// anirss names a download or a subscription.
fn name_for(name: Option<&str>, items: &[Release], fallback: &str) -> Result<String, CoreError> {
    match name.map(str::trim).filter(|n| !n.is_empty()) {
        Some(n) => validate_name(n),
        None => {
            let shown = items
                .first()
                .map(|i| crate::releases::titles::show_name(&i.title))
                .unwrap_or_default();
            validate_name(if shown.is_empty() { fallback } else { &shown })
        }
    }
}

#[cfg(feature = "phone")]
fn rules_store(core: &Core) -> crate::releases::rules::RuleStore {
    crate::releases::rules::RuleStore::new(
        PathBuf::from(&core.paths.config_dir).join("phone-rules.json"),
    )
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
    let engine = core
        .phone
        .engine
        .get_or_try_init(|| engine::Engine::open(&root, &state))
        .await?;
    // The subscriptions start being read with the engine that downloads for them, once.
    if !core
        .phone
        .polling
        .swap(true, std::sync::atomic::Ordering::AcqRel)
        && let Ok(owner) = core.owner()
    {
        core.handle.spawn(async move {
            loop {
                if let Some(engine) = owner.phone.engine.get() {
                    poll_with(&owner, engine).await;
                }
                tokio::time::sleep(POLL_EVERY).await;
            }
        });
    }
    Ok(engine)
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

/// The `.torrent` behind an http link, fetched by the core so its lookup goes over HTTPS
/// like every other request; None for a magnet, or when the site answered with something
/// else (the engine then tries the link itself). Err when the site did not answer: the
/// engine would only ask it again, so the caller stops there (#135).
#[cfg(feature = "phone")]
async fn torrent_file(core: &Core, link: &str) -> Result<Option<Vec<u8>>, CoreError> {
    if !(link.starts_with("http://") || link.starts_with("https://")) {
        return Ok(None);
    }
    let reply = core
        .http
        .send(crate::net::HttpRequest {
            method: crate::net::Method::Get,
            url: link.to_string(),
            headers: Vec::new(),
            body: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        })
        .await
        .map_err(|e| CoreError::Service {
            service: crate::downloads::feed::host_of(link),
            code: "unreachable".to_string(),
            message: e.message,
        })?;
    Ok((reply.is_success() && reply.body.first() == Some(&b'd')).then(|| reply.body.to_vec()))
}

#[cfg(feature = "phone")]
pub fn download(
    core: &Core,
    items: Vec<Release>,
    name: Option<String>,
    limit_bytes: Option<u64>,
) -> Result<Reply, CoreError> {
    if items.is_empty() {
        return Err(CoreError::invalid("items", "nothing to download"));
    }
    let folder = name_for(name.as_deref(), &items, "Downloads")?;
    if let Some(limit) = limit_bytes {
        // On disk, plus what the engine still has to write, plus these; a release whose
        // size the feed did not give counts as a typical episode.
        let used = used_bytes(&library_dir(core));
        let pending: u64 = core.handle.block_on(async {
            match engine(core).await {
                Ok(e) => e
                    .list()
                    .await
                    .iter()
                    .map(|t| t.size.saturating_sub((t.progress * t.size as f64) as u64))
                    .sum(),
                Err(_) => 0,
            }
        });
        let size: u64 = items
            .iter()
            .map(|i| crate::downloads::facts::size_bytes(&i.size).unwrap_or(1_500_000_000))
            .sum();
        if used + pending + size > limit {
            return Err(CoreError::invalid(
                "limit",
                format!(
                    "this would take AniBeam past its storage cap ({:.1} of {:.1} GB used or on the way); delete episodes or raise the cap",
                    (used + pending) as f64 / 1e9,
                    limit as f64 / 1e9
                ),
            ));
        }
    }
    core.handle.block_on(async {
        let engine = engine(core).await?;
        for item in &items {
            let file = torrent_file(core, &item.link).await?;
            engine.add(item, &folder, file).await?;
        }
        Ok::<(), CoreError>(())
    })?;
    let what = if items.len() == 1 {
        items[0].title.clone()
    } else {
        format!("{} releases of {folder}", items.len())
    };
    core.bus.info(
        Stage::Downloads,
        format!("Downloading {what}"),
        EventBody::Notice,
    );
    Ok(Reply::Ok)
}

/// Subscribes to a search (`Subscribe`): the core keeps the feed and its rule, takes what
/// the feed already holds that the rule matches, as qBittorrent's auto-downloader does,
/// and checks the feed again on its own after that.
#[cfg(feature = "phone")]
pub fn subscribe(
    core: &Core,
    query: String,
    endpoint: String,
    items: Vec<Release>,
    name: Option<String>,
) -> Result<Reply, CoreError> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err(CoreError::invalid("query", "the search is empty"));
    }
    if items.is_empty() {
        return Err(CoreError::invalid(
            "items",
            format!("no results for {query}"),
        ));
    }
    let native = native(core);
    let spec = core
        .handle
        .block_on(native.preview(&query, &endpoint, &items))?;
    let name = name_for(name.as_deref(), &items, &query)?;
    let store = rules_store(core);
    let (_, subscribed) = store.subscribe(&name, &spec, &library_dir(core))?;
    // What the feed holds now for episodes the library already has is taken, so the first
    // round does not download them again (qBittorrent's previously matched episodes).
    if let Ok(now) = core
        .handle
        .block_on(feed(core, &spec.feed_url, &subscribed.name))
    {
        let (facts, _) = crate::downloads::annotate(core, &query, &now);
        let had: Vec<String> = now
            .iter()
            .zip(&facts)
            .filter(|(_, f)| f.on_disk)
            .map(|(i, _)| i.link.clone())
            .collect();
        if !had.is_empty() {
            store.mark_taken(&subscribed.name, &had)?;
        }
    }
    core.bus.info(
        Stage::Downloads,
        format!("Subscribed to {}", subscribed.name),
        EventBody::Notice,
    );
    core.handle.block_on(poll(core));
    Ok(Reply::PhoneSubscribed { subscribed })
}

#[cfg(feature = "phone")]
pub fn rules(core: &Core) -> Result<Reply, CoreError> {
    let torrents = core.handle.block_on(async {
        match engine(core).await {
            Ok(e) => e.list().await,
            Err(_) => Vec::new(),
        }
    });
    let rules = rules_store(core)
        .load()
        .iter()
        .map(|r| {
            let in_engine = torrents
                .iter()
                .filter(|t| t.save_path == r.save_path)
                .count() as u64;
            let files = walkdir::WalkDir::new(&r.save_path)
                .into_iter()
                .flatten()
                .filter(|e| e.file_type().is_file())
                .count() as u64;
            crate::releases::rules::info(r, in_engine, files)
        })
        .collect();
    Ok(Reply::PhoneRules { rules })
}

#[cfg(feature = "phone")]
pub fn unsubscribe(core: &Core, names: Vec<String>, files: bool) -> Result<Reply, CoreError> {
    let gone = rules_store(core).remove(&names)?;
    if files {
        core.handle.block_on(async {
            let engine = engine(core).await?;
            for rule in &gone {
                let hashes: Vec<String> = engine
                    .list()
                    .await
                    .into_iter()
                    .filter(|t| t.save_path == rule.save_path)
                    .map(|t| t.hash)
                    .collect();
                engine.control(&hashes, PhoneAction::Remove).await?;
                // The folder is the show's, shared with downloads made by hand: it goes only
                // when the rule's torrents took the last of it.
                let _ = std::fs::remove_dir(&rule.save_path);
            }
            Ok::<(), CoreError>(())
        })?;
    }
    Ok(Reply::Ok)
}

/// A subscription's feed: AnimeTosho's JSON search, or RSS.
#[cfg(feature = "phone")]
async fn feed(core: &Core, url: &str, name: &str) -> Result<Vec<Release>, CoreError> {
    let timeout = crate::releases::fetch::DEFAULT_TIMEOUT;
    if url.contains("animetosho.org/feed/json") {
        crate::releases::tosho::fetch(core.http.as_ref(), url, name, timeout).await
    } else {
        crate::releases::fetch::fetch_rss(core.http.as_ref(), url, name, timeout).await
    }
}

/// How often the core reads the subscriptions' feeds: qBittorrent's default.
#[cfg(feature = "phone")]
const POLL_EVERY: std::time::Duration = std::time::Duration::from_secs(30 * 60);

/// Reads every subscription's feed and downloads what each rule takes that it has not
/// taken before (qBittorrent's RSS auto-downloader). A feed that does not answer is left
/// for the next round.
#[cfg(feature = "phone")]
pub async fn poll(core: &Core) {
    if let Ok(engine) = engine(core).await {
        poll_with(core, engine).await;
    }
}

#[cfg(feature = "phone")]
async fn poll_with(core: &Core, engine: &engine::Engine) {
    // One round at a time: a subscribe's round and the timer's would add the same releases.
    let _one = core.phone.poll_lock.lock().await;
    let store = rules_store(core);
    for rule in store.load().into_iter().filter(|r| r.enabled) {
        let items = match feed(core, &rule.feed_url, &rule.name).await {
            Ok(items) => items,
            Err(e) => {
                tracing::warn!("subscription {} not read: {e}", rule.name);
                continue;
            }
        };
        let folder = std::path::Path::new(&rule.save_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| rule.name.clone());
        let mut taken = Vec::new();
        for item in crate::releases::rules::due(&rule, &items) {
            let file = match torrent_file(core, &item.link).await {
                Ok(file) => file,
                // The rest are left for the next round, like a feed that does not answer.
                Err(e) => {
                    tracing::warn!("subscription {} stopped: {e}", rule.name);
                    break;
                }
            };
            match engine.add(item, &folder, file).await {
                Ok(_) => {
                    core.bus.info(
                        Stage::Downloads,
                        format!("New for {}: {}", rule.name, item.title),
                        EventBody::Notice,
                    );
                    taken.push(item.link.clone());
                }
                Err(e) => tracing::warn!("{} not added: {e}", item.title),
            }
        }
        if !taken.is_empty()
            && let Err(e) = store.mark_taken(&rule.name, &taken)
        {
            tracing::warn!("subscription {} not saved: {e}", rule.name);
        }
    }
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
pub fn download(
    _: &Core,
    _: Vec<Release>,
    _: Option<String>,
    _: Option<u64>,
) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn subscribe(
    _: &Core,
    _: String,
    _: String,
    _: Vec<Release>,
    _: Option<String>,
) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn rules(_: &Core) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn unsubscribe(_: &Core, _: Vec<String>, _: bool) -> Result<Reply, CoreError> {
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
