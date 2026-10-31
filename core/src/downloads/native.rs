//! The core's own engine on the torrent client (#119): subscribing, adding
//! torrents, listing and removing subscriptions, written to qBittorrent
//! directly where anirss's engine asks anirss. Each one is anirss's own
//! (`qbt/actions.py` and `api.py`) step for step, so a rule either engine
//! wrote lists, updates and goes away the same through the other.
//!
//! One difference is deliberate: files a removal takes go to the trash
//! (`trash`), never deleted for good. A client on another machine keeps its
//! files out of the trash's reach, so there its own delete takes them.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Value, json};

use crate::contract::*;
use crate::core::Core;
use crate::releases::native::Native;

use super::anirss::url_safe;
use super::paths::{self, SavePaths};
use super::qbt::{self, QbtClient};
use super::{settings, trash};

/// How `CoreError::Service` names a refusal that is the engine's own,
/// not the torrent client's.
const SERVICE: &str = "AniBeam";

/// The folder a download with no name and no show to name it after goes
/// into (anirss's fallback name).
const FALLBACK_NAME: &str = "anirss";

fn not_found(message: String) -> CoreError {
    CoreError::Service {
        service: SERVICE.to_string(),
        code: "not_found".to_string(),
        message,
    }
}

// Subscribing ------------------------------------------------------------------

/// A feed and an auto-download rule saving into `save_base/<name>`
/// (anirss `subscribe`). The name defaults to the show's, from the first
/// release; the same show from another endpoint gains ` @endpoint`.
pub(super) async fn subscribe(
    core: &Arc<Core>,
    native: &Native,
    query: &str,
    endpoint: &str,
    items: &[Release],
    name: Option<&str>,
) -> Result<Subscribed, CoreError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(CoreError::invalid("query", "empty query"));
    }
    if items.is_empty() {
        return Err(not_found(format!("no results for {query:?}")));
    }
    let name = paths::name_for(name, items, query)?;
    let spec = native.preview(query, endpoint, items).await?;
    let qbt = super::qbt_client(core).await?;
    let saving = paths::resolve(core, &qbt).await?;
    // Naming is check-then-act: one subscription at a time
    let _one = core.downloads.rules.lock().await;
    let name = unique_name(&qbt, &name, &spec).await;
    let save_path = paths::join(&saving.save_base, &name);
    let tags = qbt::clean_tags(&saving.tags);
    qbt.add_feed(&spec.feed_url, &name).await?;
    qbt.set_rule(&name, &rule_def(&spec, &save_path, &tags))
        .await?;
    Ok(Subscribed {
        name,
        feed_url: spec.feed_url,
        save_path,
        must_contain: spec.must_contain,
        must_not_contain: spec.must_not_contain,
        tags,
    })
}

/// `name`, or `name @endpoint` when a rule of that name reads another feed
/// (anirss `_unique_rule_name`). Subscribing the same feed again keeps the
/// name and overwrites the rule. A read that fails keeps the name.
async fn unique_name(qbt: &QbtClient, name: &str, spec: &SubscriptionSpec) -> String {
    let Ok(rules) = qbt.rules().await else {
        return name.to_string();
    };
    let Some(rule) = rules.get(name) else {
        return name.to_string();
    };
    let reads_it = rule["affectedFeeds"]
        .as_array()
        .is_some_and(|f| f.iter().any(|u| u.as_str() == Some(&spec.feed_url)));
    if reads_it || spec.endpoint_name.is_empty() {
        name.to_string()
    } else {
        format!("{name} @{}", spec.endpoint_name)
    }
}

/// The rule exactly as anirss writes it. qBittorrent 4.6 learned tags
/// through `torrentParams`, which then overrides the flat keys whole, so
/// the save path is said again inside it; older builds ignore the block.
pub(super) fn rule_def(spec: &SubscriptionSpec, save_path: &str, tags: &[String]) -> Value {
    let mut rule = json!({
        "enabled": true,
        "mustContain": spec.must_contain,
        "mustNotContain": spec.must_not_contain,
        "useRegex": false,
        "episodeFilter": "",
        "smartFilter": false,
        "previouslyMatchedEpisodes": [],
        "affectedFeeds": [spec.feed_url],
        "ignoreDays": 0,
        "lastMatch": "",
        "addPaused": null,
        "assignedCategory": "",
        "savePath": save_path,
    });
    if !tags.is_empty() {
        rule["torrentParams"] = json!({
            "save_path": save_path,
            "use_auto_tmm": false,
            "tags": tags,
        });
    }
    rule
}

// Adding torrents ----------------------------------------------------------------

/// The releases into `save_base/<name>` (anirss `download_items`).
pub(super) async fn download(
    core: &Arc<Core>,
    items: &[Release],
    name: Option<&str>,
) -> Result<AddedTorrents, CoreError> {
    if items.is_empty() {
        return Err(CoreError::invalid("items", "no items given"));
    }
    let name = paths::name_for(name, items, FALLBACK_NAME)?;
    let links = checked_links(items.iter().map(|i| i.link.as_str()))?;
    let qbt = super::qbt_client(core).await?;
    let saving = paths::resolve(core, &qbt).await?;
    let save_path = paths::join(&saving.save_base, &name);
    add_links(&qbt, &links, &save_path, &saving).await
}

/// One torrent straight into the films' folder, with no folder of its own
/// (anirss `movie`).
pub(super) async fn movie(core: &Arc<Core>, link: &str) -> Result<AddedTorrents, CoreError> {
    let links = checked_links([link])?;
    let qbt = super::qbt_client(core).await?;
    let saving = paths::resolve(core, &qbt).await?;
    add_links(&qbt, &links, &saving.movie_path, &saving).await
}

/// The links as qBittorrent should get them (anirss `_check_links`): a
/// magnet or an http(s) URL, a web link percent-encoded where it must be,
/// and nothing with whitespace left in it, since the links go
/// newline-separated and one could smuggle in another.
fn checked_links<'a>(links: impl IntoIterator<Item = &'a str>) -> Result<Vec<String>, CoreError> {
    let mut out = Vec::new();
    for link in links {
        let fits = ["magnet:", "http://", "https://"]
            .iter()
            .any(|s| link.starts_with(s))
            && !link.chars().any(|c| c.is_control());
        let safe = url_safe(link);
        if !fits || safe.chars().any(char::is_whitespace) {
            return Err(CoreError::invalid(
                "links",
                format!("links must be magnet: or http(s) URLs, got {link:?}"),
            ));
        }
        out.push(safe);
    }
    if out.is_empty() {
        return Err(CoreError::invalid("links", "no links given"));
    }
    Ok(out)
}

/// Hands the links to qBittorrent (anirss `add_links`). The hashes are the
/// ones known up front, from magnets.
async fn add_links(
    qbt: &QbtClient,
    links: &[String],
    save_path: &str,
    saving: &SavePaths,
) -> Result<AddedTorrents, CoreError> {
    qbt.add_urls(links, save_path, &saving.tags).await?;
    let hashes = links
        .iter()
        .filter_map(|l| super::replace::magnet_hash(l))
        .collect();
    Ok(AddedTorrents {
        count: links.len() as u64,
        save_path: save_path.to_string(),
        tags: qbt::clean_tags(&saving.tags),
        hashes,
    })
}

// Listing ------------------------------------------------------------------------

/// Every rule qBittorrent holds, whoever wrote it, by name with case
/// ignored, each with the torrents saving into its folder and the files in
/// it (anirss `list_subscriptions`). Files are counted only when the client
/// saves on this machine; a path on another is someone else's folder.
pub(super) async fn subscriptions(core: &Arc<Core>) -> Result<Vec<SubscriptionRule>, CoreError> {
    let qbt = super::qbt_client(core).await?;
    let rules = qbt.rules().await?;
    let rules: Vec<(String, Value)> = rules
        .as_object()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    if rules.is_empty() {
        return Ok(Vec::new());
    }
    let mut counts: HashMap<String, u64> = HashMap::new();
    for t in qbt.torrents().await? {
        *counts.entry(paths::norm(&t.save_path)).or_default() += 1;
    }
    let here = files_here(core)?;
    let mut out: Vec<SubscriptionRule> = rules
        .iter()
        .map(|(name, rule)| {
            let feed_url = first_feed(rule);
            let save_path = rule["savePath"].as_str().unwrap_or("").to_string();
            SubscriptionRule {
                name: name.clone(),
                query: crate::subscriptions::decode_nyaa_query(&feed_url).unwrap_or_default(),
                feed_url,
                torrent_count: counts.get(&paths::norm(&save_path)).copied().unwrap_or(0),
                file_count: 0,
                rule_enabled: rule["enabled"].as_bool().unwrap_or(true),
                save_path,
                show: None,
            }
        })
        .collect();
    out.sort_by_key(|r| r.name.to_lowercase());
    if here {
        let folders: Vec<String> = out.iter().map(|r| r.save_path.clone()).collect();
        let found: Vec<u64> = blocking(move || {
            folders
                .iter()
                .map(|f| walk(Path::new(f)).0)
                .collect::<Vec<u64>>()
        })
        .await?;
        for (r, n) in out.iter_mut().zip(found) {
            r.file_count = n;
        }
    }
    Ok(out)
}

fn first_feed(rule: &Value) -> String {
    rule["affectedFeeds"][0].as_str().unwrap_or("").to_string()
}

// Removing ------------------------------------------------------------------------

/// What removing one subscription touches (anirss `RemovePlan`).
struct Plan {
    name: String,
    feed_url: String,
    feed_path: Option<String>,
    save_path: String,
    torrents: Vec<Torrent>,
    size_bytes: u64,
}

/// Removes the subscriptions (feed and rule), with their torrents and the
/// files in their folders when asked (anirss `remove`). Every name is
/// checked before anything changes, so a typo cannot half-remove a list;
/// with `dry_run` the plan comes back and nothing changes, the same
/// function deciding both so they cannot disagree. Files are only taken
/// from strictly inside the save folders.
pub(super) async fn remove(
    core: &Arc<Core>,
    names: &[String],
    torrents: bool,
    files: bool,
    dry_run: bool,
) -> Result<Vec<RemovePlan>, CoreError> {
    let mut unique: Vec<String> = Vec::new();
    for n in names {
        if !unique.contains(n) {
            unique.push(n.clone());
        }
    }
    if unique.is_empty() {
        return Err(CoreError::invalid("names", "no subscription names given"));
    }
    let here = files_here(core)?;
    if files && !torrents && !here {
        return Err(CoreError::invalid(
            "files",
            "the torrent client keeps its files on another machine, so they can only go with \
             its torrents",
        ));
    }
    let qbt = super::qbt_client(core).await?;
    let saving = paths::resolve(core, &qbt).await?;
    let _one = core.downloads.rules.lock().await;
    let plans = plan(&qbt, &unique, torrents, files, here, &saving.save_base).await?;
    if files {
        confined(core, &plans, &saving, here)?;
    }
    if !dry_run {
        execute(&qbt, &plans, torrents, files, here).await?;
    }
    Ok(plans
        .into_iter()
        .map(|p| RemovePlan {
            name: p.name,
            feed_url: p.feed_url,
            save_path: p.save_path,
            torrent_count_in_qb: p.torrents.len() as u64,
            size_bytes: p.size_bytes,
            torrents: p.torrents.into_iter().map(|t| t.name).collect(),
        })
        .collect())
}

async fn plan(
    qbt: &QbtClient,
    names: &[String],
    torrents: bool,
    files: bool,
    here: bool,
    save_base: &str,
) -> Result<Vec<Plan>, CoreError> {
    let rules = qbt.rules().await?;
    let missing: Vec<&str> = names
        .iter()
        .filter(|n| !rules.get(n.as_str()).is_some_and(Value::is_object))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        return Err(not_found(format!(
            "unknown feed(s): {}",
            missing.join(", ")
        )));
    }
    let mut plans = Vec::new();
    let mut feeds: Option<Vec<(String, String)>> = None;
    let mut held: Option<Vec<Torrent>> = None;
    for name in names {
        let rule = &rules[name.as_str()];
        let feed_url = first_feed(rule);
        let saved = rule["savePath"]
            .as_str()
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| paths::join(save_base, name));
        let save_path = paths::norm(&saved);
        let feed_path = if feed_url.is_empty() {
            None
        } else {
            if feeds.is_none() {
                feeds = Some(qbt.feeds().await?);
            }
            feeds
                .iter()
                .flatten()
                .find(|(_, url)| *url == feed_url)
                .map(|(path, _)| path.clone())
        };
        let mut taken = Vec::new();
        if torrents || files {
            if held.is_none() {
                held = Some(qbt.torrents().await?);
            }
            taken = held
                .iter()
                .flatten()
                .filter(|t| paths::norm(&t.save_path) == save_path)
                .cloned()
                .collect();
        }
        let size_bytes = if files && here {
            let folder = PathBuf::from(&save_path);
            blocking(move || walk(&folder).1).await?
        } else {
            0
        };
        plans.push(Plan {
            name: name.clone(),
            feed_url,
            feed_path,
            save_path,
            torrents: taken,
            size_bytes,
        });
    }
    Ok(plans)
}

/// Refuses files outside the save folders (anirss `check_confined`), and on
/// this machine a folder that is the home folder, holds it, or holds a
/// library folder, as the trash's own checks refuse them.
fn confined(core: &Core, plans: &[Plan], saving: &SavePaths, here: bool) -> Result<(), CoreError> {
    let roots = [saving.save_base.as_str(), saving.movie_path.as_str()];
    let outside: Vec<&str> = plans
        .iter()
        .filter(|p| !paths::inside(&p.save_path, &roots))
        .map(|p| p.save_path.as_str())
        .collect();
    if !outside.is_empty() {
        return Err(CoreError::invalid(
            "files",
            format!(
                "refusing to remove files outside the download folders: {}",
                outside.join(", ")
            ),
        ));
    }
    if !here {
        return Ok(());
    }
    let sources: Vec<String> = core.store.read(|c| {
        let mut stmt = c.prepare("SELECT path FROM sources")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut all = Vec::new();
        for row in rows {
            all.push(row?);
        }
        Ok(all)
    })?;
    let home = std::env::home_dir();
    for p in plans {
        let folder = Path::new(&p.save_path);
        let holds = |inner: &Path| inner.starts_with(folder);
        if home.as_deref().is_some_and(holds) {
            return Err(CoreError::invalid(
                "files",
                format!(
                    "{} is the home folder or holds it, so nothing was removed",
                    p.save_path
                ),
            ));
        }
        if sources.iter().any(|s| holds(Path::new(&paths::norm(s)))) {
            return Err(CoreError::invalid(
                "files",
                format!(
                    "{} is a library folder or holds one, so nothing was removed",
                    p.save_path
                ),
            ));
        }
    }
    Ok(())
}

/// Carries the plans out (anirss `execute_remove`): the torrents, the feed,
/// the rule, then the folder. On this machine the torrents go without
/// their files and the folder goes to the trash; on another the client's
/// own delete takes the files.
async fn execute(
    qbt: &QbtClient,
    plans: &[Plan],
    torrents: bool,
    files: bool,
    here: bool,
) -> Result<(), CoreError> {
    for p in plans {
        if torrents && !p.torrents.is_empty() {
            let hashes: Vec<String> = p.torrents.iter().map(|t| t.hash.clone()).collect();
            qbt.delete(&hashes, files && !here).await?;
        }
        match &p.feed_path {
            Some(path) => qbt.remove_item(path).await?,
            None if !p.feed_url.is_empty() => {
                tracing::warn!("no feed in qBittorrent reads the rule {}'s feed", p.name)
            }
            None => {}
        }
        qbt.remove_rule(&p.name).await?;
        if files && here && Path::new(&p.save_path).is_dir() {
            let folder = vec![PathBuf::from(&p.save_path)];
            blocking(move || trash::move_to_trash(&folder)).await??;
        }
    }
    Ok(())
}

// Disk ------------------------------------------------------------------------

/// Whether the client in use saves on this machine (`settings::files_here`).
fn files_here(core: &Core) -> Result<bool, CoreError> {
    Ok(settings::files_here(
        &core.store.read(settings::load_stored)?,
    ))
}

/// The files under `dir` and their bytes, links not followed; nothing for
/// a folder that is not there.
fn walk(dir: &Path) -> (u64, u64) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (0, 0);
    };
    let mut total = (0, 0);
    for entry in entries.flatten() {
        let Ok(meta) = entry.path().symlink_metadata() else {
            continue;
        };
        if meta.is_dir() {
            let (n, bytes) = walk(&entry.path());
            total.0 += n;
            total.1 += bytes;
        } else {
            total.0 += 1;
            total.1 += meta.len();
        }
    }
    total
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<T, CoreError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| CoreError::Internal {
            message: format!("blocking task: {e}"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> SubscriptionSpec {
        SubscriptionSpec {
            feed_url: "https://feed.example/rss?q=Show".into(),
            must_contain: "1080p".into(),
            must_not_contain: "HEVC|x265".into(),
            endpoint_name: "feed".into(),
        }
    }

    #[test]
    fn the_rule_reads_as_anirss_writes_it() {
        let untagged = rule_def(&spec(), "/srv/TV/Show", &[]);
        assert_eq!(
            untagged,
            json!({
                "enabled": true, "mustContain": "1080p", "mustNotContain": "HEVC|x265",
                "useRegex": false, "episodeFilter": "", "smartFilter": false,
                "previouslyMatchedEpisodes": [], "affectedFeeds": ["https://feed.example/rss?q=Show"],
                "ignoreDays": 0, "lastMatch": "", "addPaused": null, "assignedCategory": "",
                "savePath": "/srv/TV/Show"
            })
        );
        let tagged = rule_def(&spec(), "/srv/TV/Show", &["Anime".into()]);
        assert_eq!(
            tagged["torrentParams"],
            json!({"save_path": "/srv/TV/Show", "use_auto_tmm": false, "tags": ["Anime"]})
        );
    }

    #[test]
    fn a_link_with_a_space_is_encoded_and_a_bad_one_refused() {
        assert_eq!(
            checked_links(["https://dl.example/a b.torrent"]).unwrap(),
            vec!["https://dl.example/a%20b.torrent"]
        );
        assert!(checked_links(["ftp://dl.example/a.torrent"]).is_err());
        assert!(checked_links(["magnet:?xt=urn:btih:aa\nmagnet:?xt=x"]).is_err());
        assert!(checked_links(["magnet:?xt=urn:btih:aa&dn=a b"]).is_err());
        assert!(checked_links([] as [&str; 0]).is_err());
    }

    #[test]
    fn a_walk_counts_files_and_bytes_and_a_missing_folder_is_nothing() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("S1")).unwrap();
        std::fs::write(d.path().join("a.mkv"), b"abc").unwrap();
        std::fs::write(d.path().join("S1/b.mkv"), b"de").unwrap();
        assert_eq!(walk(d.path()), (2, 5));
        assert_eq!(walk(&d.path().join("gone")), (0, 0));
    }
}
