//! The phone's own downloads: its sources, its search, and (with the `phone` feature) a
//! torrent engine inside the core. The desktop keeps anirss and qBittorrent; these calls
//! are the phone's, and they block the calling thread rather than run as jobs.

pub mod check;
#[cfg(feature = "phone")]
pub mod engine;
#[cfg(feature = "phone")]
pub mod storage;

#[cfg(feature = "phone")]
use std::collections::HashMap;
use std::path::PathBuf;

use crate::contract::*;
use crate::core::Core;
#[cfg(feature = "phone")]
use crate::downloads::paths::name_for;

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
    /// The subscriptions already said to have stopped, so the log says it once.
    #[cfg(feature = "phone")]
    stopped: std::sync::Mutex<std::collections::HashSet<String>>,
}

fn sources_path(core: &Core) -> PathBuf {
    PathBuf::from(&core.paths.config_dir).join("phone-sources.json")
}

/// The phone ships with no source of its own (#152): endpoints come from a repo the person
/// adds, or by hand. This list is only what someone saved here before repos existed.
/// Each kind is read under its shape name, should the file still hold one from before #602.
pub fn sources(core: &Core) -> Vec<PhoneSource> {
    let mut sources = saved_sources(core);
    for s in &mut sources {
        s.kind = crate::releases::kinds::shape_of(s.kind.trim()).to_string();
    }
    sources
}

fn saved_sources(core: &Core) -> Vec<PhoneSource> {
    std::fs::read(sources_path(core))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// The saved sources whose kind has a name from before #602 get its shape name, in place and
/// once (`repos::rename_old_kinds`). Returns how many.
pub(crate) fn rename_old_kinds(core: &Core) -> Result<usize, CoreError> {
    let mut saved = saved_sources(core);
    let mut renamed = 0;
    for s in &mut saved {
        let now = crate::releases::kinds::shape_of(s.kind.trim());
        if now != s.kind.trim() {
            s.kind = now.to_string();
            renamed += 1;
        }
    }
    if renamed == 0 {
        return Ok(0);
    }
    let path = sources_path(core);
    let bytes =
        serde_json::to_vec_pretty(&saved).map_err(|e| CoreError::internal(e.to_string()))?;
    crate::atomic_file::write(&path, bytes)
        .map_err(|e| CoreError::internal(e.source.to_string()))?;
    Ok(renamed)
}

pub fn set_sources(core: &Core, sources: Vec<PhoneSource>) -> Result<Reply, CoreError> {
    // A row saved before a rule covered it (#248, #260) stays as it is, so the phone can
    // still remove the other sources around it; search leaves it out (`searched`).
    let saved = self::sources(core);
    let mut checked: Vec<(PhoneSource, bool)> = Vec::with_capacity(sources.len());
    for s in sources {
        // Switching a saved row on or off is not a change to what it searches.
        if saved
            .iter()
            .any(|o| o.name == s.name && o.kind == s.kind && o.url == s.url)
        {
            checked.push((s, false));
            continue;
        }
        let s = trimmed(&s);
        crate::releases::endpoints::validate(&s.name, &s.kind, &s.url)?;
        checked.push((s, true));
    }
    // Search tells sources apart by name, so a new one takes a name of its own.
    for (s, new) in &checked {
        if *new
            && checked
                .iter()
                .filter(|(o, _)| o.name.trim() == s.name)
                .count()
                > 1
        {
            return Err(CoreError::invalid(
                "endpoint name",
                format!("{:?}: another source has that name", s.name),
            ));
        }
    }
    let sources: Vec<PhoneSource> = checked.into_iter().map(|(s, _)| s).collect();
    let path = sources_path(core);
    let bytes =
        serde_json::to_vec_pretty(&sources).map_err(|e| CoreError::internal(e.to_string()))?;
    crate::atomic_file::write(&path, bytes)
        .map_err(|e| CoreError::internal(e.source.to_string()))?;
    Ok(Reply::PhoneSources { sources })
}

/// What a search tries, in order: the enabled feeds saved before repos, then the installed
/// endpoints. An installed endpoint whose id is a saved feed's name takes that feed's place,
/// so installing an endpoint from a repo replaces an old saved one rather than searching it twice.
fn searched(core: &Core) -> Vec<(PhoneSource, crate::releases::Endpoint)> {
    let mut installed = crate::repos::runnable(core);
    let mut out: Vec<(PhoneSource, crate::releases::Endpoint)> = Vec::new();
    // Trimmed, so the name the search page shows is the endpoint's name.
    for s in sources(core).iter().filter(|s| s.enabled).map(trimmed) {
        if out.iter().any(|(o, _)| o.name == s.name) {
            continue;
        }
        if let Some(at) = installed.iter().position(|r| r.endpoint.name == s.name) {
            let r = installed.remove(at);
            out.push((installed_source(&r), r.endpoint));
        } else if crate::releases::endpoints::check_shape(&s.name, &s.kind, &s.url).is_ok() {
            let endpoint = crate::releases::Endpoint::new(&s.name, &s.kind, &s.url);
            out.push((s, endpoint));
        }
        // A source saved before the checks that would now refuse it searches one fixed
        // page or adds a second query: it is left out.
    }
    for r in installed {
        out.push((installed_source(&r), r.endpoint));
    }
    out
}

/// Every source a rule may have been made on, switched off for search or not: the installed
/// ones (`repos::fillers`, which the feed poller reads rules through too), then the feeds
/// saved before repos that no installed one replaced (#536).
#[cfg(feature = "phone")]
fn followed_sources(core: &Core) -> Vec<crate::releases::Endpoint> {
    let mut out: Vec<crate::releases::Endpoint> = crate::repos::fillers(core)
        .into_iter()
        .map(|r| r.endpoint)
        .collect();
    for s in sources(core).iter().map(trimmed) {
        if !out.iter().any(|e| e.name == s.name)
            && crate::releases::endpoints::check_shape(&s.name, &s.kind, &s.url).is_ok()
        {
            out.push(crate::releases::Endpoint::new(&s.name, &s.kind, &s.url));
        }
    }
    out
}

/// `s` with the spaces around its name, kind and address taken off.
fn trimmed(s: &PhoneSource) -> PhoneSource {
    PhoneSource {
        name: s.name.trim().to_string(),
        kind: s.kind.trim().to_string(),
        url: s.url.trim().to_string(),
        enabled: s.enabled,
    }
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
    let sources = searched(core).into_iter().map(|(_, e)| e).collect();
    crate::releases::native::Native::for_core(core, sources, None)
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
fn rules_store(core: &Core) -> crate::releases::rules::RuleStore {
    crate::releases::rules::RuleStore::new(
        PathBuf::from(&core.paths.config_dir).join("phone-rules.json"),
    )
}

/// Takes the secrets out of the subscriptions kept before #149, once, at launch (#463).
/// `lifted` says the launch's lift of hand-added sources' secrets went through.
#[cfg(feature = "phone")]
pub fn rekeep_rules(core: &Core, lifted: bool) {
    match crate::repos::rekeep_old_rules(core, &rules_store(core), lifted) {
        Ok(0) => {}
        Ok(n) => {
            tracing::info!(
                "{n} subscriptions kept before #149 now keep their key as a placeholder"
            );
            core.bus.info(
                Stage::Downloads,
                "Shows followed earlier no longer keep their source's key in the app's files",
                EventBody::Notice,
            );
        }
        Err(e) => {
            core.bus.warn(
                Stage::Downloads,
                format!("could not take the source keys out of the shows followed earlier: {e}"),
                EventBody::Notice,
            );
        }
    }
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
    let http = crate::repos::endpoint_http(core);
    let reply = crate::net::Http::send(
        &http,
        crate::net::HttpRequest {
            method: crate::net::Method::Get,
            url: link.to_string(),
            headers: Vec::new(),
            body: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        },
    )
    .await
    .map_err(|e| CoreError::Service {
        service: crate::net::host_shown(link),
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
    rolling: Option<bool>,
) -> Result<Reply, CoreError> {
    use crate::downloads::rolling;
    if items.is_empty() {
        return Err(CoreError::invalid("items", "nothing to download"));
    }
    let folder = name_for(name.as_deref(), &items, "Downloads")?;
    // A long show is held to a window of episodes, as on the desktop: a pick of single
    // episodes adds only the first window, and a batch is added with only its first window
    // fetched, picked from its file list before a byte of the rest is written.
    let settings = crate::downloads::settings::read(core)?.rolling;
    let options = DownloadOptions {
        rolling,
        ..Default::default()
    };
    let mut plan = rolling::plan(
        &items,
        &options,
        &settings,
        Some(&folder),
        crate::time::now_secs(),
    );
    let owner = core.owner()?;
    let (added, failed) = core.handle.block_on(async {
        let engine = engine(core).await?;
        // Each rolling batch's file list first, so its first window is known before
        // anything is added and only that counts against the storage cap.
        let mut peeks: HashMap<String, (engine::Peeked, Option<Vec<i64>>)> = HashMap::new();
        for r in &plan.rolls {
            let [p] = r.parts.as_slice() else { continue };
            if p.single {
                continue;
            }
            let file = torrent_file(core, &p.release.link).await?;
            let peeked = engine.peek(&p.release, file).await?;
            let first = rolling::first_window(r, 0, &peeked.files);
            peeks.insert(p.release.link.clone(), (peeked, first));
        }
        if let Some(limit) = limit_bytes {
            // On disk, plus what the engine still has to write, plus these; a release whose
            // size the feed did not give counts as a typical episode.
            let used = used_bytes(&library_dir(core));
            let pending: u64 = engine
                .list()
                .await
                .iter()
                .map(|t| t.size.saturating_sub((t.progress * t.size as f64) as u64))
                .sum();
            let size: u64 = plan
                .now
                .iter()
                .map(|i| match peeks.get(&i.link) {
                    Some((peeked, first)) => peeked
                        .files
                        .iter()
                        .enumerate()
                        .filter(|(n, _)| {
                            first
                                .as_ref()
                                .is_none_or(|w| w.get(*n) != Some(&rolling::SKIP))
                        })
                        .map(|(_, f)| f.size)
                        .sum(),
                    None => crate::downloads::facts::size_bytes(&i.size).unwrap_or(1_500_000_000),
                })
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
        let mut added: HashMap<String, String> = HashMap::new();
        let mut failed = None;
        for item in &plan.now {
            let got = match peeks.remove(&item.link) {
                Some((peeked, first)) => {
                    // Not long enough to roll: the whole batch comes, as any other.
                    if first.is_none()
                        && let Some(r) = plan
                            .rolls
                            .iter_mut()
                            .find(|r| r.parts.iter().any(|p| p.release.link == item.link))
                    {
                        r.parts.clear();
                    }
                    engine
                        .add_peeked(item, &folder, peeked, first.unwrap_or_default())
                        .await
                }
                None => match torrent_file(core, &item.link).await {
                    Ok(file) => engine.add(item, &folder, file).await,
                    Err(e) => Err(e),
                },
            };
            match got {
                Ok(hash) => {
                    added.insert(item.link.clone(), hash);
                }
                // The ones after it are left: the engine would likely refuse them too.
                Err(e) => {
                    failed = Some(e);
                    break;
                }
            }
        }
        // Each roll's added parts get their torrents; a single that did not go in waits in
        // its roll for the window, and a batch that did not go in has nothing to roll.
        for r in plan.rolls.iter_mut() {
            for p in r.parts.iter_mut().filter(|p| p.added) {
                match added.get(&p.release.link) {
                    Some(hash) => p.hash = hash.clone(),
                    None if p.single => p.added = false,
                    None => p.gone = true,
                }
            }
        }
        plan.rolls
            .retain(|r| !r.parts.is_empty() && r.parts.iter().any(|p| p.added && !p.gone));
        if !plan.rolls.is_empty() {
            let rolls = std::mem::take(&mut plan.rolls);
            rolling::add_rolls(&owner, rolls).await?;
            let torrents = engine.list().await;
            rolling::tick(&owner, &PhoneRolls { core, engine }, &torrents).await;
        }
        Ok::<_, CoreError>((added, failed))
    })?;
    if added.is_empty()
        && let Some(e) = failed
    {
        return Err(e);
    }
    let count = added.len();
    let what = if count == 1 {
        plan.now
            .iter()
            .find(|i| added.contains_key(&i.link))
            .map(|i| i.title.clone())
            .unwrap_or_default()
    } else {
        format!("{count} releases of {folder}")
    };
    let held = items.len().saturating_sub(plan.now.len());
    core.bus.say(
        Level::Info,
        Stage::Downloads,
        format!(
            "Started downloading {} of {folder}.",
            if count == 1 {
                "an episode".to_string()
            } else {
                format!("{count} episodes")
            }
        ),
        if held > 0 {
            format!("Downloading {what}, {held} more waiting to roll in")
        } else {
            format!("Downloading {what}")
        },
        EventBody::Notice,
    );
    match failed {
        Some(e) => Err(e),
        None => Ok(Reply::Ok),
    }
}

/// The phone's engine as the rolling downloads' client.
#[cfg(feature = "phone")]
struct PhoneRolls<'a> {
    core: &'a Core,
    engine: &'a engine::Engine,
}

#[cfg(feature = "phone")]
impl crate::downloads::rolling::RollClient for PhoneRolls<'_> {
    async fn files(&self, hash: &str) -> Result<Vec<TorrentFile>, CoreError> {
        self.engine.files(hash)
    }

    async fn set_file_priority(
        &self,
        hash: &str,
        ids: &[usize],
        priority: i64,
    ) -> Result<(), CoreError> {
        self.engine.set_priority(hash, ids, priority).await
    }

    /// librqbit always fetches a torrent's files in name order, which is episode order.
    async fn set_sequential(&self, _: &[String], _: bool) -> Result<(), CoreError> {
        Ok(())
    }

    async fn delete(&self, hashes: &[String], files: bool) -> Result<(), CoreError> {
        let action = if files {
            PhoneAction::Remove
        } else {
            PhoneAction::Forget
        };
        self.engine.control(hashes, action).await
    }

    async fn recheck(&self, hashes: &[String]) -> Result<(), CoreError> {
        for h in hashes {
            self.engine.close_files(h);
        }
        Ok(())
    }

    fn files_here(&self, _: &Core) -> bool {
        true
    }

    async fn add(
        &self,
        _: &std::sync::Arc<Core>,
        items: &[Release],
        roll: &crate::downloads::rolling::Roll,
    ) -> Result<Vec<String>, CoreError> {
        let folder = name_for(roll.name.as_deref(), items, "Downloads")?;
        let mut hashes = Vec::new();
        for item in items {
            let file = torrent_file(self.core, &item.link).await?;
            hashes.push(self.engine.add(item, &folder, file).await?);
        }
        Ok(hashes)
    }
}

/// A pass over the rolling downloads against the engine now: after an episode was watched
/// to the end, or a roll was changed.
#[cfg(feature = "phone")]
pub async fn roll_pass(core: &std::sync::Arc<Core>) {
    let engine = match engine(core).await {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!("rolling pass skipped: {e}");
            return;
        }
    };
    let torrents = engine.list().await;
    let client = PhoneRolls {
        core: core.as_ref(),
        engine,
    };
    crate::downloads::rolling::tick(core, &client, &torrents).await;
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
    // The rules file keeps the endpoint's secrets as placeholders, never the values (#149).
    let kept = crate::repos::kept_spec(core, &spec, &query);
    let fillers = crate::repos::fillers(core);
    let fill = |feed: &str, endpoint: &str| crate::repos::filled_feed(&fillers, endpoint, feed);
    let filled = fill(&kept.feed_url, &kept.endpoint_name).unwrap_or_else(|| spec.feed_url.clone());
    let (_, subscribed) = store.subscribe(
        &name,
        &kept,
        &filled,
        |r| fill(&r.feed_url, &r.endpoint),
        &library_dir(core),
    )?;
    // What the feed holds now for episodes the library already has is taken, so the first
    // round does not download them again (qBittorrent's previously matched episodes).
    let from = feed_sources(core, &fillers);
    if let Ok(now) = core.handle.block_on(feed(
        core,
        &filled,
        &subscribed.name,
        &kept.endpoint_name,
        &from,
    )) {
        let (facts, _) = crate::downloads::annotate(core, &query, &now);
        let had: Vec<String> = now
            .iter()
            .zip(&facts)
            .filter(|(_, f)| f.on_disk || f.other_on_disk)
            .map(|(i, _)| i.link.clone())
            .collect();
        if !had.is_empty() {
            store.mark_taken(&subscribed.name, &had)?;
        }
    }
    core.bus.say(
        Level::Info,
        Stage::Downloads,
        format!("Subscribed to {}.", subscribed.name),
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
    let sources = followed_sources(core);
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
            let mut info = crate::releases::rules::info(r, in_engine, files);
            info.followable =
                crate::releases::endpoints::follows(&sources, &r.feed_url, Some(&r.endpoint));
            info
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

/// A subscription's feed, read the way its source's kind answers: a json-feed source's
/// JSON, else RSS. The source is `endpoint` when the rule names one still installed, else
/// the one whose address the feed is.
#[cfg(feature = "phone")]
async fn feed(
    core: &Core,
    url: &str,
    name: &str,
    endpoint: &str,
    sources: &[crate::releases::Endpoint],
) -> Result<Vec<Release>, CoreError> {
    let timeout = crate::releases::fetch::DEFAULT_TIMEOUT;
    let http = crate::repos::endpoint_http(core);
    if is_json_feed(sources, url, endpoint) {
        crate::releases::json_feed::fetch(&http, url, name, timeout).await
    } else {
        crate::releases::fetch::fetch_rss(&http, url, name, timeout).await
    }
}

/// The sources a subscription's feed can come from: every installed endpoint, then the
/// sources saved here before repos, which a rule made on one still names.
#[cfg_attr(not(feature = "phone"), allow(dead_code))]
fn feed_sources(
    core: &Core,
    installed: &[crate::repos::Runnable],
) -> Vec<crate::releases::Endpoint> {
    let mut out: Vec<crate::releases::Endpoint> =
        installed.iter().map(|r| r.endpoint.clone()).collect();
    for s in sources(core).iter().map(trimmed) {
        if !out.iter().any(|e| e.name == s.name) {
            out.push(crate::releases::Endpoint::new(&s.name, &s.kind, &s.url));
        }
    }
    out
}

/// Whether the feed `url` of a rule on `endpoint` comes from a json-feed source.
#[cfg_attr(not(feature = "phone"), allow(dead_code))]
fn is_json_feed(sources: &[crate::releases::Endpoint], url: &str, endpoint: &str) -> bool {
    crate::releases::endpoints::source_of(sources, url, Some(endpoint))
        .is_some_and(|e| e.kind == crate::releases::kinds::JSON_FEED)
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
    let fillers = crate::repos::fillers(core);
    let from = feed_sources(core, &fillers);
    for rule in store.load().into_iter().filter(|r| r.enabled) {
        let Some(url) = crate::repos::filled_feed(&fillers, &rule.endpoint, &rule.feed_url) else {
            // Its endpoint was removed, or its credentials were: said once, not every round.
            if core.phone.stopped.lock().unwrap().insert(rule.name.clone()) {
                core.bus.say(
                    Level::Warn,
                    Stage::Downloads,
                    format!(
                        "{} is not checked for new episodes: its source is gone or needs its key again.",
                        rule.name
                    ),
                    format!("Subscription {} stopped: no credentials for {}", rule.name, rule.endpoint),
                    EventBody::Notice,
                );
            }
            continue;
        };
        core.phone.stopped.lock().unwrap().remove(&rule.name);
        let items = match feed(core, &url, &rule.name, &rule.endpoint, &from).await {
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
                    core.bus.say(
                        Level::Info,
                        Stage::Downloads,
                        format!("A new episode of {} is downloading.", rule.name),
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

/// The engine's torrents, each rolling one with its roll. The read is also when the
/// rolling downloads move on, as the desktop's torrent read is.
#[cfg(feature = "phone")]
pub fn torrents(core: &Core) -> Result<Reply, CoreError> {
    core.handle.block_on(async {
        let engine = engine(core).await?;
        let mut torrents = engine.list().await;
        if core.downloads.unattended_allowed()
            && let Ok(owner) = core.owner()
        {
            crate::downloads::rolling::tick(&owner, &PhoneRolls { core, engine }, &torrents).await;
            torrents = engine.list().await;
        }
        crate::downloads::rolling::annotate(core, &mut torrents);
        // Which torrent each library file came from, as the desktop records it from
        // qBittorrent (#620), off this call's path so the list never waits on the store.
        record_origins(core, torrents.clone(), "on a list read");
        Ok(Reply::PhoneTorrents {
            torrents,
            held: engine.held(),
        })
    })
}

/// After a scan brought files in: the origins of the new files, from the engine's list,
/// only when the engine is already open, so a scan never starts it (#620).
#[cfg(feature = "phone")]
pub(crate) fn after_scan(core: &Core) {
    if core.phone.engine.get().is_none() {
        return;
    }
    let Ok(owner) = core.owner() else { return };
    core.handle.spawn(async move {
        let Some(engine) = owner.phone.engine.get() else {
            return;
        };
        let torrents = engine.list().await;
        let recorded = crate::downloads::origins::record_from(&owner, None, &torrents).await;
        crate::downloads::origins::logged(recorded, "after a scan");
    });
}

/// Records the origins of `torrents` in a task of its own; a failure is a debug line and
/// the files stay unrecorded until the next read.
#[cfg(feature = "phone")]
fn record_origins(core: &Core, torrents: Vec<Torrent>, when: &'static str) {
    let Ok(owner) = core.owner() else { return };
    core.handle.spawn(async move {
        let recorded = crate::downloads::origins::record_from(&owner, None, &torrents).await;
        crate::downloads::origins::logged(recorded, when);
    });
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
    _: Option<bool>,
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

#[cfg(all(test, feature = "phone"))]
mod origin_tests {
    use super::*;

    /// A scan reads origins only through an engine already open (#620): it never opens one.
    #[test]
    fn a_scan_does_not_open_the_engine() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            crate::net::FakeHttp::new(),
            crate::trackers::Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        crate::downloads::origins::after_scan(&core);
        assert!(core.phone.engine.get().is_none());
        core.shutdown();
    }
}

#[cfg(test)]
mod feed_tests {
    use super::*;
    use crate::releases::Endpoint;

    #[test]
    fn a_feed_is_read_the_way_its_sources_kind_answers() {
        let sources = [
            Endpoint::new("feed", "json-feed", "https://feed.example/json?q={query}"),
            Endpoint::new("rss", "rss", "https://rss.example/?q={query}"),
        ];
        // By the rule's source, and by the address for a rule that names none.
        assert!(is_json_feed(&sources, "https://anywhere.example/x", "feed"));
        assert!(is_json_feed(
            &sources,
            "https://feed.example/json?q=Show",
            ""
        ));
        assert!(!is_json_feed(
            &sources,
            "https://rss.example/?q=Show",
            "rss"
        ));
        // A source no longer installed reads as RSS.
        assert!(!is_json_feed(
            &sources,
            "https://gone.example/json?q=Show",
            "gone"
        ));
    }

    #[test]
    fn a_source_saved_before_repos_still_says_how_its_feed_reads() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            crate::net::FakeHttp::new(),
            crate::trackers::Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        let saved = serde_json::json!([{"name": "old", "kind": "json-feed",
            "url": "https://feed.example/json?q={query}", "enabled": true}]);
        std::fs::create_dir_all(&core.paths.config_dir).unwrap();
        std::fs::write(sources_path(&core), saved.to_string()).unwrap();
        let from = feed_sources(&core, &[]);
        assert!(is_json_feed(
            &from,
            "https://feed.example/json?q=Show",
            "old"
        ));
        assert!(is_json_feed(&from, "https://feed.example/json?q=Show", ""));
    }
}
