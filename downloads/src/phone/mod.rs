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

use crate::Downloads;
#[cfg(feature = "phone")]
use crate::paths::name_for;
use anibeam_contract::*;
use anibeam_foundation::services::Services;

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

fn sources_path(services: &Services) -> PathBuf {
    PathBuf::from(&services.paths.config_dir).join("phone-sources.json")
}

/// `PhoneSources`: the sources saved here, as the call's reply.
pub fn sources_call(services: &Services) -> Result<Reply, CoreError> {
    Ok(Reply::PhoneSources {
        sources: sources(services),
    })
}

/// `PhoneSearchSources`: [`search_sources`] as the call's reply.
pub fn search_sources_call(services: &Services) -> Result<Reply, CoreError> {
    Ok(Reply::PhoneSources {
        sources: search_sources(services),
    })
}

/// The phone ships with no source of its own (#152): endpoints come from a repo the person
/// adds, or by hand. This list is only what someone saved here before repos existed.
/// Each kind is read under its shape name, should the file still hold one from before #602.
pub fn sources(services: &Services) -> Vec<PhoneSource> {
    let mut sources = saved_sources(services);
    for s in &mut sources {
        s.kind = anibeam_releases::kinds::shape_of(s.kind.trim()).to_string();
    }
    sources
}

fn saved_sources(services: &Services) -> Vec<PhoneSource> {
    std::fs::read(sources_path(services))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// The saved sources whose kind has a name from before #602 get its shape name, in place and
/// once, beside `repos::rename_old_kinds` in `Core::open`. Returns how many.
pub fn rename_old_kinds(services: &Services) -> Result<usize, CoreError> {
    let mut saved = saved_sources(services);
    let mut renamed = 0;
    for s in &mut saved {
        let now = anibeam_releases::kinds::shape_of(s.kind.trim());
        if now != s.kind.trim() {
            s.kind = now.to_string();
            renamed += 1;
        }
    }
    if renamed == 0 {
        return Ok(0);
    }
    let path = sources_path(services);
    let bytes =
        serde_json::to_vec_pretty(&saved).map_err(|e| CoreError::internal(e.to_string()))?;
    anibeam_foundation::atomic_file::write(&path, bytes)
        .map_err(|e| CoreError::internal(e.source.to_string()))?;
    Ok(renamed)
}

pub fn set_sources(services: &Services, sources: Vec<PhoneSource>) -> Result<Reply, CoreError> {
    // A row saved before a rule covered it (#248, #260) stays as it is, so the phone can
    // still remove the other sources around it; search leaves it out (`searched`).
    let saved = self::sources(services);
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
        anibeam_releases::endpoints::validate(&s.name, &s.kind, &s.url)?;
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
                anibeam_contract::i18n::tr(
                    "core-phone-name-taken",
                    &[("name", format!("{:?}", s.name).into())],
                ),
            ));
        }
    }
    let sources: Vec<PhoneSource> = checked.into_iter().map(|(s, _)| s).collect();
    let path = sources_path(services);
    let bytes =
        serde_json::to_vec_pretty(&sources).map_err(|e| CoreError::internal(e.to_string()))?;
    anibeam_foundation::atomic_file::write(&path, bytes)
        .map_err(|e| CoreError::internal(e.source.to_string()))?;
    Ok(Reply::PhoneSources { sources })
}

/// What a search tries, in order: the enabled feeds saved before repos, then the installed
/// endpoints. An installed endpoint whose id is a saved feed's name takes that feed's place,
/// so installing an endpoint from a repo replaces an old saved one rather than searching it twice.
fn searched(services: &Services) -> Vec<(PhoneSource, anibeam_releases::Endpoint)> {
    let mut installed = anibeam_releases::repos::runnable(services);
    let mut out: Vec<(PhoneSource, anibeam_releases::Endpoint)> = Vec::new();
    // Trimmed, so the name the search page shows is the endpoint's name.
    for s in sources(services).iter().filter(|s| s.enabled).map(trimmed) {
        if out.iter().any(|(o, _)| o.name == s.name) {
            continue;
        }
        if let Some(at) = installed.iter().position(|r| r.endpoint.name == s.name) {
            let r = installed.remove(at);
            out.push((installed_source(&r), r.endpoint));
        } else if anibeam_releases::endpoints::check_shape(&s.name, &s.kind, &s.url).is_ok() {
            let endpoint = anibeam_releases::Endpoint::new(&s.name, &s.kind, &s.url);
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
fn followed_sources(services: &Services) -> Vec<anibeam_releases::Endpoint> {
    let mut out: Vec<anibeam_releases::Endpoint> = anibeam_releases::repos::fillers(services)
        .into_iter()
        .map(|r| r.endpoint)
        .collect();
    for s in sources(services).iter().map(trimmed) {
        if !out.iter().any(|e| e.name == s.name)
            && anibeam_releases::endpoints::check_shape(&s.name, &s.kind, &s.url).is_ok()
        {
            out.push(anibeam_releases::Endpoint::new(&s.name, &s.kind, &s.url));
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

fn installed_source(r: &anibeam_releases::repos::Runnable) -> PhoneSource {
    PhoneSource {
        name: r.endpoint.name.clone(),
        kind: r.endpoint.kind.clone(),
        url: r.shown_url.clone(),
        enabled: true,
    }
}

/// `searched`'s sources, as the search page lists them.
pub fn search_sources(services: &Services) -> Vec<PhoneSource> {
    searched(services).into_iter().map(|(s, _)| s).collect()
}

/// The sources as the ported anirss endpoints, enabled ones in order, each host held to the
/// pace its repo definition asks for.
fn native(services: &Services) -> anibeam_releases::native::Native {
    let sources = searched(services).into_iter().map(|(_, e)| e).collect();
    anibeam_releases::native::Native::for_core(services, sources, None)
}

/// How long the show search waits for covers before answering without them.
const COVER_WAIT: std::time::Duration = std::time::Duration::from_millis(1500);

/// The show search: AniList's hits for `query` with the covers that came in time, so the phone
/// can list shows before anyone looks at a release.
pub fn find_shows(downloads: &Downloads, query: String, limit: u32) -> Result<Reply, CoreError> {
    let query = query.trim().to_string();
    if query.chars().count() < 2 {
        return Ok(Reply::PhoneShows {
            shows: Vec::new(),
            posters: Vec::new(),
        });
    }
    let media = downloads
        .services
        .handle
        .block_on(downloads.net.anilist.search(&query, limit.clamp(1, 20)))?;
    let shows: Vec<SearchResult> = media
        .iter()
        .map(anibeam_metadata::apply::to_result)
        .collect();
    let urls: Vec<String> = shows.iter().filter_map(|s| s.cover_url.clone()).collect();
    // The hits go back after COVER_WAIT at most. Each cover is its own task, so the ones
    // already cached or quick come back and a slow one only loses its own poster; it still
    // finishes into the cache (the cache's sweep bounds it), so the same search shows it
    // next time.
    let fetching: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let images = downloads.net.images.clone();
            downloads
                .services
                .handle
                .spawn(async move { images.ensure(&[url]).await })
        })
        .collect();
    let fetched = downloads.services.handle.block_on(async move {
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
    services: &Services,
    query: String,
    endpoint: Option<String>,
    fallback: bool,
) -> Result<Reply, CoreError> {
    let native = native(services);
    let found = services
        .handle
        .block_on(native.search(&query, endpoint.as_deref(), fallback))?;
    let fell_back = found.fell_back();
    let items = if found.items.iter().any(|i| i.seeders.is_none()) {
        services.handle.block_on(native.enrich(found.items.clone()))
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
        let (facts, show) = crate::annotate(services, &a.query, &a.items);
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
    services: &Services,
    query: String,
    endpoint: String,
    items: Vec<Release>,
    op: RefineOp,
) -> Result<Reply, CoreError> {
    let native = native(services);
    let mut refinement = services
        .handle
        .block_on(native.apply(&query, &endpoint, &items, &op))?;
    let (facts, show) = crate::annotate(services, &refinement.query, &refinement.items);
    refinement.facts = facts;
    refinement.show = show;
    Ok(Reply::PhoneRefined { refinement })
}

#[cfg(feature = "phone")]
fn rules_store(services: &Services) -> anibeam_releases::rules::RuleStore {
    anibeam_releases::rules::RuleStore::new(
        PathBuf::from(&services.paths.config_dir).join("phone-rules.json"),
    )
}

/// Takes the secrets out of the subscriptions kept before #149, once, at launch (#463).
/// `lifted` says the launch's lift of hand-added sources' secrets went through.
#[cfg(feature = "phone")]
pub fn rekeep_rules(services: &Services, lifted: bool) {
    match anibeam_releases::repos::rekeep_old_rules(services, &rules_store(services), lifted) {
        Ok(0) => {}
        Ok(n) => {
            tracing::info!(
                "{n} subscriptions kept before #149 now keep their key as a placeholder"
            );
            services.bus.info(
                Stage::Downloads,
                "Shows followed earlier no longer keep their source's key in the app's files",
                EventBody::Notice,
            );
        }
        Err(e) => {
            services.bus.warn(
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
fn library_dir(services: &Services) -> PathBuf {
    let data = PathBuf::from(&services.paths.data_dir);
    // `files/core/data` up to `files`.
    data.ancestors()
        .nth(2)
        .map(|files| files.join(LIBRARY_DIR))
        .unwrap_or_else(|| data.join(LIBRARY_DIR))
}

#[cfg(feature = "phone")]
async fn engine(downloads: &std::sync::Arc<Downloads>) -> Result<&engine::Engine, CoreError> {
    let root = library_dir(&downloads.services);
    let state = PathBuf::from(&downloads.services.paths.state_dir).join("torrents");
    let engine = downloads
        .phone
        .engine
        .get_or_try_init(|| engine::Engine::open(&root, &state))
        .await?;
    // The subscriptions start being read with the engine that downloads for them, once.
    if !downloads
        .phone
        .polling
        .swap(true, std::sync::atomic::Ordering::AcqRel)
    {
        let owner = downloads.clone();
        downloads.services.handle.spawn(async move {
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
async fn torrent_file(services: &Services, link: &str) -> Result<Option<Vec<u8>>, CoreError> {
    if !(link.starts_with("http://") || link.starts_with("https://")) {
        return Ok(None);
    }
    let http = anibeam_releases::repos::endpoint_http(services);
    let reply = anibeam_net::Http::send(
        &http,
        anibeam_net::HttpRequest::get(link).timeout(std::time::Duration::from_secs(30)),
    )
    .await
    .map_err(|e| CoreError::service(anibeam_net::host_shown(link), "unreachable", e.message))?;
    Ok((reply.is_success() && reply.body.first() == Some(&b'd')).then(|| reply.body.to_vec()))
}

#[cfg(feature = "phone")]
pub fn download(
    downloads: &std::sync::Arc<Downloads>,
    items: Vec<Release>,
    name: Option<String>,
    limit_bytes: Option<u64>,
    rolling: Option<bool>,
) -> Result<Reply, CoreError> {
    use crate::rolling;
    if items.is_empty() {
        return Err(CoreError::invalid(
            "items",
            anibeam_contract::i18n::tr("core-phone-nothing-to-download", &[]),
        ));
    }
    let folder = name_for(name.as_deref(), &items, "Downloads")?;
    // A long show is held to a window of episodes, as on the desktop: a pick of single
    // episodes adds only the first window, and a batch is added with only its first window
    // fetched, picked from its file list before a byte of the rest is written.
    let settings = crate::settings::read(&downloads.services)?.rolling;
    let options = DownloadOptions {
        rolling,
        ..Default::default()
    };
    let mut plan = rolling::plan(
        &items,
        &options,
        &settings,
        Some(&folder),
        anibeam_foundation::time::now_secs(),
    );
    let owner = downloads.clone();
    let (added, failed) = downloads.services.handle.block_on(async {
        let engine = engine(downloads).await?;
        // Each rolling batch's file list first, so its first window is known before
        // anything is added and only that counts against the storage cap.
        let mut peeks: HashMap<String, (engine::Peeked, Option<Vec<i64>>)> = HashMap::new();
        for r in &plan.rolls {
            let [p] = r.parts.as_slice() else { continue };
            if p.single {
                continue;
            }
            let file = torrent_file(&downloads.services, &p.release.link).await?;
            let peeked = engine.peek(&p.release, file).await?;
            let first = rolling::first_window(r, 0, &peeked.files);
            peeks.insert(p.release.link.clone(), (peeked, first));
        }
        if let Some(limit) = limit_bytes {
            // On disk, plus what the engine still has to write, plus these; a release whose
            // size the feed did not give counts as a typical episode.
            let used = used_bytes(&library_dir(&downloads.services));
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
                    None => anibeam_foundation::size::size_bytes(&i.size).unwrap_or(1_500_000_000),
                })
                .sum();
            if used + pending + size > limit {
                return Err(CoreError::invalid(
                    "limit",
                    anibeam_contract::i18n::tr(
                        "core-phone-over-cap",
                        &[
                            (
                                "used",
                                format!("{:.1}", (used + pending) as f64 / 1e9).into(),
                            ),
                            ("cap", format!("{:.1}", limit as f64 / 1e9).into()),
                        ],
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
                None => match torrent_file(&downloads.services, &item.link).await {
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
        // A torrent a stored roll already holds (the same release picked again) keeps that
        // roll: two would fight over its files.
        let held: std::collections::HashSet<String> = rolling::load(&downloads.services)?
            .iter()
            .flat_map(|r| &r.parts)
            .filter(|p| !p.hash.is_empty())
            .map(|p| p.hash.clone())
            .collect();
        plan.rolls.retain(|r| {
            !r.parts.is_empty()
                && r.parts.iter().any(|p| p.added && !p.gone)
                && !r.parts.iter().any(|p| held.contains(&p.hash))
        });
        if !plan.rolls.is_empty() {
            let rolls = std::mem::take(&mut plan.rolls);
            rolling::add_rolls(&owner.services, rolls).await?;
            let torrents = engine.list().await;
            rolling::tick(&owner, &PhoneRolls { downloads, engine }, &torrents).await;
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
    downloads.services.bus.say(
        Level::Info,
        Stage::Downloads,
        anibeam_contract::words!(
            "core-phone-download-started",
            count = count,
            folder = &folder,
        ),
        if held > 0 {
            format!("Downloading {what}, {held} more waiting to roll in")
        } else {
            format!("Downloading {what}")
        },
        EventBody::Notice,
    );
    // Some went in: the download stands, and what did not is said in the log rather than
    // as a failure, so a second Get does not add the same releases again.
    if let Some(e) = failed {
        downloads.services.bus.say(
            Level::Warn,
            Stage::Downloads,
            anibeam_contract::words!(
                "core-phone-download-partial",
                folder = &folder,
                error = e.to_string(),
            ),
            format!("Download of {folder} stopped part way: {e}"),
            EventBody::Notice,
        );
    }
    Ok(Reply::Ok)
}

/// The phone's engine as the rolling downloads' client.
#[cfg(feature = "phone")]
struct PhoneRolls<'a> {
    downloads: &'a Downloads,
    engine: &'a engine::Engine,
}

#[cfg(feature = "phone")]
impl crate::rolling::RollClient for PhoneRolls<'_> {
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

    fn files_here(&self, _: &Services) -> bool {
        true
    }

    async fn add(
        &self,
        _: &std::sync::Arc<Downloads>,
        items: &[Release],
        roll: &crate::rolling::Roll,
    ) -> Result<Vec<String>, CoreError> {
        let folder = name_for(roll.name.as_deref(), items, "Downloads")?;
        let mut hashes = Vec::new();
        for item in items {
            let file = torrent_file(&self.downloads.services, &item.link).await?;
            hashes.push(self.engine.add(item, &folder, file).await?);
        }
        Ok(hashes)
    }
}

/// A pass over the rolling downloads against the engine now: after an episode was watched
/// to the end, or a roll was changed.
#[cfg(feature = "phone")]
pub async fn roll_pass(downloads: &std::sync::Arc<Downloads>) {
    let engine = match engine(downloads).await {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!("rolling pass skipped: {e}");
            return;
        }
    };
    let torrents = engine.list().await;
    let client = PhoneRolls { downloads, engine };
    crate::rolling::tick(downloads, &client, &torrents).await;
}

/// Subscribes to a search (`Subscribe`): the core keeps the feed and its rule, takes what
/// the feed already holds that the rule matches, as qBittorrent's auto-downloader does,
/// and checks the feed again on its own after that.
#[cfg(feature = "phone")]
pub fn subscribe(
    downloads: &std::sync::Arc<Downloads>,
    query: String,
    endpoint: String,
    items: Vec<Release>,
    name: Option<String>,
) -> Result<Reply, CoreError> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err(CoreError::invalid(
            "query",
            anibeam_contract::i18n::tr("core-phone-empty-search", &[]),
        ));
    }
    if items.is_empty() {
        return Err(CoreError::invalid(
            "items",
            anibeam_contract::i18n::tr(
                "core-phone-no-results",
                &[("query", query.as_str().into())],
            ),
        ));
    }
    let native = native(&downloads.services);
    let spec = downloads
        .services
        .handle
        .block_on(native.preview(&query, &endpoint, &items))?;
    let name = name_for(name.as_deref(), &items, &query)?;
    let store = rules_store(&downloads.services);
    // The rules file keeps the endpoint's secrets as placeholders, never the values (#149).
    let kept = anibeam_releases::repos::kept_spec(&downloads.services, &spec, &query);
    let fillers = anibeam_releases::repos::fillers(&downloads.services);
    let fill =
        |feed: &str, endpoint: &str| anibeam_releases::repos::filled_feed(&fillers, endpoint, feed);
    let filled = fill(&kept.feed_url, &kept.endpoint_name).unwrap_or_else(|| spec.feed_url.clone());
    let (_, subscribed) = store.subscribe(
        &name,
        &kept,
        &filled,
        |r| fill(&r.feed_url, &r.endpoint),
        &library_dir(&downloads.services),
    )?;
    // What the feed holds now for episodes the library already has is taken, so the first
    // round does not download them again (qBittorrent's previously matched episodes).
    let from = feed_sources(&downloads.services, &fillers);
    if let Ok(now) = downloads.services.handle.block_on(feed(
        &downloads.services,
        &filled,
        &subscribed.name,
        &kept.endpoint_name,
        &from,
    )) {
        let (facts, _) = crate::annotate(&downloads.services, &query, &now);
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
    downloads.services.bus.say(
        Level::Info,
        Stage::Downloads,
        anibeam_contract::words!("core-phone-subscribed", name = &subscribed.name),
        format!("Subscribed to {}", subscribed.name),
        EventBody::Notice,
    );
    downloads.services.handle.block_on(poll(downloads));
    Ok(Reply::PhoneSubscribed { subscribed })
}

#[cfg(feature = "phone")]
pub fn rules(downloads: &std::sync::Arc<Downloads>) -> Result<Reply, CoreError> {
    let torrents = downloads.services.handle.block_on(async {
        match engine(downloads).await {
            Ok(e) => e.list().await,
            Err(_) => Vec::new(),
        }
    });
    let sources = followed_sources(&downloads.services);
    let rules = rules_store(&downloads.services)
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
            let mut info = anibeam_releases::rules::info(r, in_engine, files);
            info.followable =
                anibeam_releases::endpoints::follows(&sources, &r.feed_url, Some(&r.endpoint));
            info
        })
        .collect();
    Ok(Reply::PhoneRules { rules })
}

#[cfg(feature = "phone")]
pub fn unsubscribe(
    downloads: &std::sync::Arc<Downloads>,
    names: Vec<String>,
    files: bool,
) -> Result<Reply, CoreError> {
    let gone = rules_store(&downloads.services).remove(&names)?;
    if files {
        downloads.services.handle.block_on(async {
            let engine = engine(downloads).await?;
            for rule in &gone {
                // Only the torrents the rule added: the show's folder is shared with
                // downloads got by hand, and those stay (#654).
                let hashes: Vec<String> = engine
                    .list()
                    .await
                    .into_iter()
                    .filter(|t| rule.added.contains(&t.hash.to_lowercase()))
                    .map(|t| t.hash)
                    .collect();
                engine.control(&hashes, PhoneAction::Remove).await?;
                // The folder goes only when the rule's torrents took the last of it.
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
    services: &Services,
    url: &str,
    name: &str,
    endpoint: &str,
    sources: &[anibeam_releases::Endpoint],
) -> Result<Vec<Release>, CoreError> {
    let timeout = anibeam_releases::fetch::DEFAULT_TIMEOUT;
    let http = anibeam_releases::repos::endpoint_http(services);
    if is_json_feed(sources, url, endpoint) {
        anibeam_releases::json_feed::fetch(&http, url, name, timeout).await
    } else {
        anibeam_releases::fetch::fetch_rss(&http, url, name, timeout).await
    }
}

/// The sources a subscription's feed can come from: every installed endpoint, then the
/// sources saved here before repos, which a rule made on one still names.
#[cfg_attr(not(feature = "phone"), allow(dead_code))]
fn feed_sources(
    services: &Services,
    installed: &[anibeam_releases::repos::Runnable],
) -> Vec<anibeam_releases::Endpoint> {
    let mut out: Vec<anibeam_releases::Endpoint> =
        installed.iter().map(|r| r.endpoint.clone()).collect();
    for s in sources(services).iter().map(trimmed) {
        if !out.iter().any(|e| e.name == s.name) {
            out.push(anibeam_releases::Endpoint::new(&s.name, &s.kind, &s.url));
        }
    }
    out
}

/// Whether the feed `url` of a rule on `endpoint` comes from a json-feed source.
#[cfg_attr(not(feature = "phone"), allow(dead_code))]
fn is_json_feed(sources: &[anibeam_releases::Endpoint], url: &str, endpoint: &str) -> bool {
    anibeam_releases::endpoints::source_of(sources, url, Some(endpoint))
        .is_some_and(|e| e.kind == anibeam_releases::kinds::JSON_FEED)
}

/// How often the core reads the subscriptions' feeds: qBittorrent's default.
#[cfg(feature = "phone")]
const POLL_EVERY: std::time::Duration = std::time::Duration::from_secs(30 * 60);

/// Reads every subscription's feed and downloads what each rule takes that it has not
/// taken before (qBittorrent's RSS auto-downloader). A feed that does not answer is left
/// for the next round.
#[cfg(feature = "phone")]
pub async fn poll(downloads: &std::sync::Arc<Downloads>) {
    if let Ok(engine) = engine(downloads).await {
        poll_with(downloads, engine).await;
    }
}

#[cfg(feature = "phone")]
async fn poll_with(downloads: &Downloads, engine: &engine::Engine) {
    // One round at a time: a subscribe's round and the timer's would add the same releases.
    let _one = downloads.phone.poll_lock.lock().await;
    let store = rules_store(&downloads.services);
    let fillers = anibeam_releases::repos::fillers(&downloads.services);
    let from = feed_sources(&downloads.services, &fillers);
    for rule in store.load().into_iter().filter(|r| r.enabled) {
        let Some(url) =
            anibeam_releases::repos::filled_feed(&fillers, &rule.endpoint, &rule.feed_url)
        else {
            // Its endpoint was removed, or its credentials were: said once, not every round.
            if downloads
                .phone
                .stopped
                .lock()
                .unwrap()
                .insert(rule.name.clone())
            {
                downloads.services.bus.say(
                    Level::Warn,
                    Stage::Downloads,
                    anibeam_contract::words!("core-phone-rule-stopped", name = &rule.name),
                    format!(
                        "Subscription {} stopped: no credentials for {}",
                        rule.name, rule.endpoint
                    ),
                    EventBody::Notice,
                );
            }
            continue;
        };
        downloads.phone.stopped.lock().unwrap().remove(&rule.name);
        let items = match feed(&downloads.services, &url, &rule.name, &rule.endpoint, &from).await {
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
        // What the engine held before this round is not the rule's, even when the rule
        // asks for the same torrent: a download got by hand stays the person's.
        let before = engine.hashes();
        for item in anibeam_releases::rules::due(&rule, &items) {
            let file = match torrent_file(&downloads.services, &item.link).await {
                Ok(file) => file,
                // The rest are left for the next round, like a feed that does not answer.
                Err(e) => {
                    tracing::warn!("subscription {} stopped: {e}", rule.name);
                    break;
                }
            };
            match engine.add(item, &folder, file).await {
                Ok(hash) => {
                    // Noted as it lands, so an unsubscribe right after finds it.
                    let own = (!before.contains(&hash.to_lowercase())).then_some(hash.as_str());
                    if let Err(e) = store.mark_added(&rule.name, &item.link, own) {
                        tracing::warn!("subscription {} not saved: {e}", rule.name);
                    }
                    downloads.services.bus.say(
                        Level::Info,
                        Stage::Downloads,
                        anibeam_contract::words!("core-phone-new-episode", name = &rule.name),
                        format!("New for {}: {}", rule.name, item.title),
                        EventBody::Notice,
                    );
                }
                Err(e) => tracing::warn!("{} not added: {e}", item.title),
            }
        }
    }
}

/// The engine's torrents, each rolling one with its roll. The read is also when the
/// rolling downloads move on, as the desktop's torrent read is.
#[cfg(feature = "phone")]
pub fn torrents(downloads: &std::sync::Arc<Downloads>) -> Result<Reply, CoreError> {
    downloads.services.handle.block_on(async {
        let engine = engine(downloads).await?;
        let mut torrents = engine.list().await;
        if downloads.unattended_allowed()
            && crate::rolling::tick(downloads, &PhoneRolls { downloads, engine }, &torrents).await
        {
            torrents = engine.list().await;
        }
        crate::rolling::annotate(&downloads.services, &mut torrents);
        // Which torrent each library file came from, as the desktop records it from
        // qBittorrent (#620), off this call's path so the list never waits on the store.
        record_origins(&downloads.services, torrents.clone(), "on a list read");
        Ok(Reply::PhoneTorrents {
            torrents,
            held: engine.held(),
        })
    })
}

/// After a scan brought files in: the origins of the new files, from the engine's list,
/// only when the engine is already open, so a scan never starts it (#620).
#[cfg(feature = "phone")]
pub(crate) fn after_scan(downloads: &std::sync::Arc<Downloads>) {
    if downloads.phone.engine.get().is_none() {
        return;
    }
    let owner = downloads.clone();
    downloads.services.handle.spawn(async move {
        let Some(engine) = owner.phone.engine.get() else {
            return;
        };
        let torrents = engine.list().await;
        let recorded = crate::origins::record_from(&owner.services, None, &torrents).await;
        crate::origins::logged(recorded, "after a scan");
    });
}

/// Records the origins of `torrents` in a task of its own; a failure is a debug line and
/// the files stay unrecorded until the next read.
#[cfg(feature = "phone")]
fn record_origins(services: &std::sync::Arc<Services>, torrents: Vec<Torrent>, when: &'static str) {
    let owner = services.clone();
    services.handle.spawn(async move {
        let recorded = crate::origins::record_from(&owner, None, &torrents).await;
        crate::origins::logged(recorded, when);
    });
}

#[cfg(feature = "phone")]
pub fn control(
    downloads: &std::sync::Arc<Downloads>,
    hashes: Vec<String>,
    action: PhoneAction,
) -> Result<Reply, CoreError> {
    downloads
        .services
        .handle
        .block_on(async { engine(downloads).await?.control(&hashes, action).await })?;
    Ok(Reply::Ok)
}

#[cfg(feature = "phone")]
pub fn set_held(downloads: &std::sync::Arc<Downloads>, held: bool) -> Result<Reply, CoreError> {
    downloads
        .services
        .handle
        .block_on(async { engine(downloads).await?.set_held(held).await })?;
    Ok(Reply::Ok)
}

#[cfg(not(feature = "phone"))]
fn unsupported() -> CoreError {
    CoreError::Unsupported {
        what: anibeam_contract::i18n::tr("core-phone-own-engine", &[]),
    }
}

#[cfg(not(feature = "phone"))]
pub fn download(
    _: &Downloads,
    _: Vec<Release>,
    _: Option<String>,
    _: Option<u64>,
    _: Option<bool>,
) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn subscribe(
    _: &Downloads,
    _: String,
    _: String,
    _: Vec<Release>,
    _: Option<String>,
) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn rules(_: &Downloads) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn unsubscribe(_: &Downloads, _: Vec<String>, _: bool) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn torrents(_: &Downloads) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn control(_: &Downloads, _: Vec<String>, _: PhoneAction) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(not(feature = "phone"))]
pub fn set_held(_: &Downloads, _: bool) -> Result<Reply, CoreError> {
    Err(unsupported())
}

#[cfg(all(test, feature = "phone"))]
mod origin_tests {
    use crate::testing::Teams;

    /// A scan reads origins only through an engine already open (#620): it never opens one.
    #[test]
    fn a_scan_does_not_open_the_engine() {
        let dir = tempfile::tempdir().unwrap();
        let teams = Teams::open(dir.path(), anibeam_net::FakeHttp::new());
        crate::origins::after_scan(&teams.downloads);
        assert!(teams.downloads.phone.engine.get().is_none());
        teams.shutdown();
    }
}

#[cfg(test)]
mod feed_tests {
    use super::*;
    use anibeam_foundation::testing::Running;
    use anibeam_releases::Endpoint;

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
        let running = Running::open(dir.path(), anibeam_net::FakeHttp::new());
        let saved = serde_json::json!([{"name": "old", "kind": "json-feed",
            "url": "https://feed.example/json?q={query}", "enabled": true}]);
        std::fs::create_dir_all(&running.services.paths.config_dir).unwrap();
        std::fs::write(sources_path(&running.services), saved.to_string()).unwrap();
        let from = feed_sources(&running.services, &[]);
        assert!(is_json_feed(
            &from,
            "https://feed.example/json?q=Show",
            "old"
        ));
        assert!(is_json_feed(&from, "https://feed.example/json?q=Show", ""));
    }
}
