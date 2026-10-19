//! What happens to a download after qBittorrent has it: download in order,
//! and replacing episodes the library already has.
//!
//! anirss adds torrents and takes no flags, so anything that has to be
//! true of a new torrent is applied after the add. Each such wish is a
//! `Followup`, one per new torrent, saved in the settings table so it
//! survives a restart. Every torrent read (and a short burst of reads right
//! after the add) binds a followup to its torrent, by info-hash when the
//! link was a magnet and by name otherwise, and then:
//!
//! - turns on download in order, once (the client only has a toggle, so
//!   `QbtClient::set_sequential` reads the state first);
//! - for each episode the torrent replaces, waits until that episode's file
//!   has finished, then moves the old files to the trash.
//!
//! Replacing is conservative on purpose. The old files are the ones the
//! library listed for that series and episode when the download was asked
//! for, checked against the library again at the moment of trashing; a
//! path the new torrent itself wrote is never one of them; and the trash
//! module's own checks (inside a library source, not the home folder)
//! apply. A refusal drops that replacement and leaves everything in place.
//! A followup whose torrent the user removed, or that nothing bound to
//! within three days, is dropped without touching a file.
//!
//! Old torrents follow their files: one whose every file was trashed leaves
//! the client (without the client deleting anything); one that keeps other
//! files has the trashed ones set to "do not download", so a recheck does
//! not fetch them again. An unfinished torrent being replaced is cancelled
//! the same way when every episode it still fetches is being replaced, and
//! only has those files skipped otherwise.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::qbt::QbtClient;
use super::{facts, trash};
use crate::contract::*;
use crate::core::Core;
use crate::store::{Store, settings};
use crate::time::now_secs;

/// Followups still to do, as JSON in the settings table.
pub const FOLLOWUPS: &str = "download_followups";
/// Which endpoint each release came from, so Activity can say where a
/// download comes from; qBittorrent keeps nothing of the kind.
pub const SOURCES: &str = "download_sources";

const MAX_AGE_SECS: i64 = 30 * 24 * 3600;
const UNBOUND_AGE_SECS: i64 = 3 * 24 * 3600;
/// A torrent counts as added by this followup if the client stamped it up
/// to this long before the followup was made (clock skew on a remote host).
const BIND_SLACK_SECS: i64 = 300;
const MAX_SOURCES: usize = 400;

/// One episode a new torrent replaces, and the files it replaces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Replacement {
    pub episode: u32,
    /// The library's files for the episode as they were when asked.
    pub old: Vec<String>,
}

/// What to do to one new torrent once the client has it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Followup {
    pub id: u64,
    pub created_at: i64,
    /// The release title, to find the torrent by when the link gave no hash.
    pub title: String,
    /// The torrent's info-hash, lowercase; empty until bound.
    #[serde(default)]
    pub hash: String,
    /// Download in order is still to be turned on.
    #[serde(default)]
    pub sequential: bool,
    #[serde(default)]
    pub series: Option<u64>,
    #[serde(default)]
    pub replace: Vec<Replacement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub key: String,
    pub endpoint: String,
    pub at: i64,
}

/// One tick at a time: the page's poll and the burst after an add both
/// run it.
static TICK: LazyLock<tokio::sync::Mutex<()>> = LazyLock::new(|| tokio::sync::Mutex::new(()));

// Reading names -----------------------------------------------------------------

/// The info-hash a magnet link names, as lowercase hex (the base32 form is
/// converted). None for anything that is not a magnet with a hash.
pub fn magnet_hash(link: &str) -> Option<String> {
    let query = link.trim().strip_prefix("magnet:?")?;
    for part in query.split('&') {
        let Some(v) = part.strip_prefix("xt=") else {
            continue;
        };
        let lower = v.to_ascii_lowercase();
        let Some(h) = lower
            .strip_prefix("urn:btih:")
            .or_else(|| lower.strip_prefix("urn%3abtih%3a"))
        else {
            continue;
        };
        if h.len() == 40 && h.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(h.to_string());
        }
        if h.len() == 32 {
            return base32_hex(h);
        }
    }
    None
}

fn base32_hex(s: &str) -> Option<String> {
    let mut bits: u64 = 0;
    let mut n = 0;
    let mut out = String::new();
    for c in s.chars() {
        let v = match c {
            'a'..='z' => c as u64 - 'a' as u64,
            '2'..='7' => c as u64 - '2' as u64 + 26,
            _ => return None,
        };
        bits = (bits << 5) | v;
        n += 5;
        if n >= 8 {
            n -= 8;
            out.push_str(&format!("{:02x}", (bits >> n) & 0xff));
            bits &= (1 << n) - 1;
        }
    }
    (out.len() == 40).then_some(out)
}

/// A title or torrent name as letters and digits only, without the file
/// extension a client adds or drops, so the two compare.
pub fn title_key(s: &str) -> String {
    let lower = s.trim().to_lowercase();
    let stem = [".mkv", ".mp4", ".avi", ".torrent"]
        .iter()
        .find_map(|e| lower.strip_suffix(e))
        .unwrap_or(&lower);
    stem.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// The torrent's name is the release's title, give or take an extension or
/// punctuation. A containment counts only for names long enough to be
/// specific.
pub fn names_match(torrent_name: &str, title: &str) -> bool {
    let (a, b) = (title_key(torrent_name), title_key(title));
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a == b || (a.len() >= 16 && b.len() >= 16 && (a.contains(&b) || b.contains(&a)))
}

fn read_name(name: &str) -> ReleaseFacts {
    let base = name.rsplit('/').next().unwrap_or(name);
    facts::read(&Release {
        title: base.to_string(),
        link: String::new(),
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
}

/// The whole-number episode a file or torrent name carries.
pub fn episode_of(name: &str) -> Option<u32> {
    let f = read_name(name);
    if f.batch {
        return None;
    }
    f.episode
        .filter(|n| *n >= 0.0 && n.fract() == 0.0)
        .map(|n| n as u32)
}

// Binding and readiness ---------------------------------------------------------------

/// Binds each unbound followup to the torrent that is its own: the earliest
/// one added since the followup was made whose name matches the title and
/// that no other followup holds. Returns whether anything bound.
pub fn bind(followups: &mut [Followup], torrents: &[Torrent]) -> bool {
    let mut taken: HashSet<String> = followups
        .iter()
        .filter(|f| !f.hash.is_empty())
        .map(|f| f.hash.to_lowercase())
        .collect();
    let mut changed = false;
    for f in followups.iter_mut().filter(|f| f.hash.is_empty()) {
        let found = torrents
            .iter()
            .filter(|t| {
                !taken.contains(&t.hash.to_lowercase())
                    && t.added_on >= f.created_at - BIND_SLACK_SECS
                    && names_match(&t.name, &f.title)
            })
            .min_by_key(|t| t.added_on);
        if let Some(t) = found {
            f.hash = t.hash.to_lowercase();
            taken.insert(f.hash.clone());
            changed = true;
        }
    }
    changed
}

/// The path of each file the torrent wrote, or its content path when the
/// client gave no file list.
pub fn written_paths(t: &Torrent, files: &[TorrentFile]) -> Vec<String> {
    if files.is_empty() {
        return if t.content_path.is_empty() {
            Vec::new()
        } else {
            vec![t.content_path.clone()]
        };
    }
    files
        .iter()
        .map(|f| {
            Path::new(&t.save_path)
                .join(&f.name)
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// This torrent has finished the file of `episode`. A single-file torrent is
/// its episode (unless its file name says another); a batch is read file
/// by file, and a skipped file never counts.
pub fn episode_ready(episode: u32, t: &Torrent, files: &[TorrentFile]) -> bool {
    if files.len() <= 1 {
        let name = files.first().map(|f| f.name.as_str()).unwrap_or(&t.name);
        let said = episode_of(name);
        return t.progress >= 1.0 && said.is_none_or(|n| n == episode);
    }
    files
        .iter()
        .any(|f| f.priority > 0 && f.progress >= 1.0 && episode_of(&f.name) == Some(episode))
}

/// The old files to trash: those the library listed, minus any path the new
/// torrent wrote.
pub fn old_to_trash(old: &[String], written: &[String]) -> Vec<String> {
    old.iter()
        .filter(|p| !written.iter().any(|w| w == *p))
        .cloned()
        .collect()
}

// The old torrents ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// Every file of the torrent was trashed: it leaves the client.
    Remove { hash: String },
    /// Some files were: they are set to "do not download".
    Skip { hash: String, ids: Vec<usize> },
}

/// What to do to the torrents that carried the trashed files. A torrent in
/// `keep` (the new ones) is never touched, and one whose files are not
/// all known stays as it is.
pub fn plan_old(
    trashed: &[PathBuf],
    torrents: &[(Torrent, Vec<TorrentFile>)],
    keep: &HashSet<String>,
) -> Vec<Effect> {
    let mut out = Vec::new();
    for (t, files) in torrents {
        if keep.contains(&t.hash.to_lowercase()) {
            continue;
        }
        let paths: Vec<PathBuf> = written_paths(t, files)
            .into_iter()
            .map(PathBuf::from)
            .collect();
        let hit: Vec<usize> = paths
            .iter()
            .enumerate()
            .filter(|(_, p)| trashed.contains(p))
            .map(|(i, _)| i)
            .collect();
        if hit.is_empty() {
            continue;
        }
        if hit.len() == paths.len() {
            out.push(Effect::Remove {
                hash: t.hash.clone(),
            });
        } else if !files.is_empty() {
            out.push(Effect::Skip {
                hash: t.hash.clone(),
                ids: hit,
            });
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq)]
pub enum CancelPlan {
    /// Leave it alone.
    Nothing,
    /// Every episode it fetches is being replaced: it goes, with what it wrote.
    Whole,
    /// It also fetches other episodes: only these files stop.
    Skip(Vec<usize>),
}

/// How an unfinished torrent gives way to a replacement of `replace`. A
/// finished torrent is a file to replace, not a download to stop. A file
/// whose episode cannot be read counts as one that is not being replaced.
pub fn cancel_plan(replace: &BTreeSet<u32>, t: &Torrent, files: &[TorrentFile]) -> CancelPlan {
    if t.progress >= 1.0 || replace.is_empty() {
        return CancelPlan::Nothing;
    }
    if files.len() <= 1 {
        let name = files.first().map(|f| f.name.as_str()).unwrap_or(&t.name);
        let ep = episode_of(name).or_else(|| episode_of(&t.name));
        return match ep {
            Some(e) if replace.contains(&e) => CancelPlan::Whole,
            _ => CancelPlan::Nothing,
        };
    }
    let wanted: Vec<(usize, Option<u32>)> = files
        .iter()
        .enumerate()
        .filter(|(_, f)| f.priority > 0)
        .map(|(i, f)| (i, episode_of(&f.name)))
        .collect();
    if !wanted.is_empty()
        && wanted
            .iter()
            .all(|(_, e)| e.is_some_and(|e| replace.contains(&e)))
    {
        return CancelPlan::Whole;
    }
    let skip: Vec<usize> = wanted
        .iter()
        .filter(|(i, e)| files[*i].progress < 1.0 && e.is_some_and(|e| replace.contains(&e)))
        .map(|(i, _)| *i)
        .collect();
    if skip.is_empty() {
        CancelPlan::Nothing
    } else {
        CancelPlan::Skip(skip)
    }
}

// The library -------------------------------------------------------------------

/// The library's files for one episode of a series.
fn library_files(
    c: &rusqlite::Connection,
    series: u64,
    episode: u32,
) -> Result<Vec<String>, CoreError> {
    let mut stmt = c.prepare(
        "SELECT path FROM files WHERE series_id = ?1 AND kind = 'episode' AND number = ?2 ORDER BY path",
    )?;
    let rows = stmt.query_map(params![series as i64, episode as f64], |r| {
        r.get::<_, String>(0)
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn library_sources(c: &rusqlite::Connection) -> Result<Vec<String>, CoreError> {
    let mut stmt = c.prepare("SELECT path FROM sources")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// What to trash for one replacement, decided against the library as it is
/// now: only paths the library still lists for this series and episode,
/// that were asked for, that the new torrent did not write, and that are
/// plain files. Returns the paths and the library sources for the checks.
pub fn trash_candidates(
    c: &rusqlite::Connection,
    series: u64,
    entry: &Replacement,
    written: &[String],
) -> Result<(Vec<String>, Vec<String>), CoreError> {
    let listed = library_files(c, series, entry.episode)?;
    let asked: Vec<String> = entry
        .old
        .iter()
        .filter(|p| listed.contains(p))
        .cloned()
        .collect();
    let paths: Vec<String> = old_to_trash(&asked, written)
        .into_iter()
        .filter(|p| {
            std::fs::symlink_metadata(p)
                .map(|m| m.is_file())
                .unwrap_or(false)
        })
        .collect();
    Ok((paths, library_sources(c)?))
}

// Storage ----------------------------------------------------------------------

fn load(core: &Core) -> Result<Vec<Followup>, CoreError> {
    core.store
        .read(|c| Ok(settings::get::<Vec<Followup>>(c, FOLLOWUPS)?.unwrap_or_default()))
}

/// What a tick changed, by id: `None` is a followup it retired.
fn diff(before: &[Followup], after: &[Followup]) -> HashMap<u64, Option<Followup>> {
    let mut changes = HashMap::new();
    for b in before {
        match after.iter().find(|a| a.id == b.id) {
            None => {
                changes.insert(b.id, None);
            }
            Some(a) if a != b => {
                changes.insert(b.id, Some(a.clone()));
            }
            _ => {}
        }
    }
    changes
}

/// The stored followups with a tick's changes applied by id: one added
/// while the tick ran is kept as it is.
fn merge(cur: Vec<Followup>, changes: &HashMap<u64, Option<Followup>>) -> Vec<Followup> {
    cur.into_iter()
        .filter_map(|f| match changes.get(&f.id) {
            None => Some(f),
            Some(None) => None,
            Some(Some(new)) => Some(new.clone()),
        })
        .collect()
}

fn save_changes(core: &Core, before: &[Followup], after: &[Followup]) -> Result<(), CoreError> {
    let changes = diff(before, after);
    if changes.is_empty() {
        return Ok(());
    }
    core.store.write(move |c| {
        let cur = settings::get::<Vec<Followup>>(c, FOLLOWUPS)?.unwrap_or_default();
        settings::set(c, FOLLOWUPS, &merge(cur, &changes))
    })
}

// Asking for a download -------------------------------------------------------------

/// The followups a download asks for: one per item that is to be read in
/// order or that replaces episodes. The old files are read from the
/// library here, before anything is added, and an item index that is not
/// one of the items is refused.
pub fn prepare(
    store: &Store,
    items: &[Release],
    options: &DownloadOptions,
) -> Result<Vec<Followup>, CoreError> {
    let mut replace: HashMap<usize, Vec<u32>> = HashMap::new();
    for r in &options.replace {
        if r.item as usize >= items.len() {
            return Err(CoreError::invalid(
                "options",
                format!("item {} is not one of the {} releases", r.item, items.len()),
            ));
        }
        replace
            .entry(r.item as usize)
            .or_default()
            .extend(r.episodes.iter().copied());
    }
    let now = now_secs();
    let mut out = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let mut episodes = replace.remove(&i).unwrap_or_default();
        episodes.sort_unstable();
        episodes.dedup();
        let mut entries = Vec::new();
        if let Some(series) = options.series {
            for e in episodes {
                let old = store.read(|c| library_files(c, series, e))?;
                if !old.is_empty() {
                    entries.push(Replacement { episode: e, old });
                }
            }
        }
        if !options.sequential && entries.is_empty() {
            continue;
        }
        out.push(Followup {
            id: 0,
            created_at: now,
            title: item.title.clone(),
            hash: magnet_hash(&item.link).unwrap_or_default(),
            sequential: options.sequential,
            series: options.series,
            replace: entries,
        });
    }
    Ok(out)
}

/// A line for the job's message: what the download will also do.
pub fn describe(options: &DownloadOptions, followups: &[Followup]) -> String {
    let mut bits = Vec::new();
    if options.sequential {
        bits.push("in order".to_string());
    }
    let n: usize = followups.iter().map(|f| f.replace.len()).sum();
    if n > 0 {
        bits.push(format!(
            "replacing {n} {} once the new files finish",
            if n == 1 { "episode" } else { "episodes" }
        ));
    }
    if bits.is_empty() {
        String::new()
    } else {
        format!(", {}", bits.join(", "))
    }
}

/// After the torrents were added: save the followups and where the releases
/// came from, stop the torrents being replaced, and keep reading the client
/// for a while so a followup binds without waiting for the page. Failures
/// are logged; the add already happened and must not read as failed.
pub async fn after_add(
    core: &Arc<Core>,
    client: &Arc<QbtClient>,
    items: &[Release],
    options: &DownloadOptions,
    followups: Vec<Followup>,
) {
    let now = now_secs();
    let sources: Vec<Source> = if options.provider.trim().is_empty() {
        Vec::new()
    } else {
        items
            .iter()
            .map(|i| Source {
                key: title_key(&i.title),
                endpoint: options.provider.trim().to_string(),
                at: now,
            })
            .collect()
    };
    let new_hashes: HashSet<String> = followups
        .iter()
        .filter(|f| !f.hash.is_empty())
        .map(|f| f.hash.clone())
        .collect();
    let saved = core.store.write_async(move |c| {
        let mut cur = settings::get::<Vec<Followup>>(c, FOLLOWUPS)?.unwrap_or_default();
        let mut next = cur.iter().map(|f| f.id).max().unwrap_or(0);
        for mut f in followups {
            next += 1;
            f.id = next;
            cur.push(f);
        }
        settings::set(c, FOLLOWUPS, &cur)?;
        if !sources.is_empty() {
            let mut all = settings::get::<Vec<Source>>(c, SOURCES)?.unwrap_or_default();
            all.retain(|s| !sources.iter().any(|n| n.key == s.key));
            all.extend(sources);
            let over = all.len().saturating_sub(MAX_SOURCES);
            all.drain(..over);
            settings::set(c, SOURCES, &all)?;
        }
        Ok(())
    });
    if let Err(e) = saved.await {
        tracing::warn!("download followups not saved: {e}");
        return;
    }
    let replacing: BTreeSet<u32> = options
        .replace
        .iter()
        .flat_map(|r| r.episodes.iter().copied())
        .collect();
    if !options.cancel.is_empty() && !replacing.is_empty() {
        cancel_in_flight(core, client, &options.cancel, &replacing, &new_hashes).await;
    }
    let (core, client) = (core.clone(), client.clone());
    let wants_ticks = options.sequential || !replacing.is_empty();
    if wants_ticks {
        let handle = core.handle.clone();
        handle.spawn(async move {
            // The client may take a moment to list a torrent it was just given
            for _ in 0..15 {
                tokio::time::sleep(Duration::from_secs(3)).await;
                match client.torrents().await {
                    Ok(torrents) => tick(&core, &client, &torrents).await,
                    Err(e) => tracing::debug!("followup read failed: {e}"),
                }
                match load(&core) {
                    Ok(f) if f.iter().all(|f| !f.hash.is_empty()) => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        });
    }
}

/// Stops the unfinished torrents the new ones stand in for, as far as
/// `cancel_plan` allows.
async fn cancel_in_flight(
    core: &Arc<Core>,
    client: &QbtClient,
    hashes: &[String],
    replacing: &BTreeSet<u32>,
    keep: &HashSet<String>,
) {
    let wanted: Vec<String> = hashes
        .iter()
        .map(|h| h.trim().to_lowercase())
        .filter(|h| !h.is_empty() && !keep.contains(h))
        .collect();
    let known = match client.torrents_named(&wanted).await {
        Ok(k) => k,
        Err(e) => {
            tracing::warn!("torrents to cancel not read: {e}");
            return;
        }
    };
    for t in known {
        let files = client.files(&t.hash).await.unwrap_or_default();
        let plan = cancel_plan(replacing, &t, &files);
        let done = match &plan {
            CancelPlan::Nothing => continue,
            CancelPlan::Skip(ids) => client
                .set_file_priority(&t.hash, ids, 0)
                .await
                .map(|_| format!("Skipped {} in {}", plural_files(ids.len()), t.name)),
            CancelPlan::Whole => cancel_whole(core, client, &t).await,
        };
        match done {
            Ok(msg) => {
                core.bus.info(Stage::Downloads, msg, EventBody::Notice);
            }
            Err(e) => tracing::warn!("{} not cancelled: {e}", t.name),
        }
    }
}

fn plural_files(n: usize) -> String {
    if n == 1 {
        "1 file".to_string()
    } else {
        format!("{n} files")
    }
}

/// Takes the torrent out of the client and what it wrote to the trash. The
/// trash targets are checked first; if they are refused the torrent stays.
async fn cancel_whole(
    core: &Arc<Core>,
    client: &QbtClient,
    t: &Torrent,
) -> Result<String, CoreError> {
    let sources = core.store.read(library_sources)?;
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let targets = trash::targets(
        &[trash::Candidate::Torrent {
            content: t.content_path.clone(),
            save_path: t.save_path.clone(),
        }],
        &sources,
        home.as_deref(),
    )?;
    client.delete(std::slice::from_ref(&t.hash), false).await?;
    let n = targets.len();
    if !targets.is_empty() {
        tokio::task::spawn_blocking(move || trash::move_to_trash(&targets))
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("trash: {e}"),
            })??;
    }
    Ok(format!(
        "Cancelled {}, {n} to the trash",
        if t.name.is_empty() {
            "a download"
        } else {
            &t.name
        }
    ))
}

// The tick ---------------------------------------------------------------------

/// One pass over the stored followups against a fresh torrent read.
pub async fn tick(core: &Arc<Core>, client: &QbtClient, torrents: &[Torrent]) {
    let _one = TICK.lock().await;
    let before = match load(core) {
        Ok(f) if !f.is_empty() => f,
        Ok(_) => return,
        Err(e) => {
            tracing::warn!("download followups not read: {e}");
            return;
        }
    };
    let mut fus = before.clone();
    bind(&mut fus, torrents);
    let now = now_secs();
    let live: HashSet<String> = torrents.iter().map(|t| t.hash.to_lowercase()).collect();
    let new_hashes: HashSet<String> = fus
        .iter()
        .filter(|f| !f.hash.is_empty())
        .map(|f| f.hash.clone())
        .collect();
    for f in fus.iter_mut() {
        if f.hash.is_empty() {
            continue;
        }
        let Some(t) = torrents
            .iter()
            .find(|t| t.hash.eq_ignore_ascii_case(&f.hash))
        else {
            continue;
        };
        if f.sequential {
            match client
                .set_sequential(std::slice::from_ref(&t.hash), true)
                .await
            {
                Ok(()) => f.sequential = false,
                Err(e) => tracing::warn!("{} not set to download in order: {e}", t.name),
            }
        }
        if f.replace.is_empty() {
            continue;
        }
        let files = if t.files.is_empty() {
            client.files(&t.hash).await.unwrap_or_default()
        } else {
            t.files.clone()
        };
        let written = written_paths(t, &files);
        let mut waiting = Vec::new();
        for entry in std::mem::take(&mut f.replace) {
            if !episode_ready(entry.episode, t, &files) {
                waiting.push(entry);
                continue;
            }
            // Ready: done with this entry whatever comes of it
            if let Some(series) = f.series
                && let Err(e) = replace_episode(
                    core,
                    client,
                    series,
                    &entry,
                    &written,
                    torrents,
                    &new_hashes,
                )
                .await
            {
                tracing::warn!("episode {} not replaced: {e}", entry.episode);
                core.bus.warn(
                    Stage::Downloads,
                    format!("Episode {} was not replaced: {e}", entry.episode),
                    EventBody::Notice,
                );
            }
        }
        f.replace = waiting;
    }
    fus.retain(|f| {
        if !f.sequential && f.replace.is_empty() {
            return false;
        }
        if now - f.created_at > MAX_AGE_SECS {
            return false;
        }
        if f.hash.is_empty() {
            return now - f.created_at <= UNBOUND_AGE_SECS;
        }
        live.contains(&f.hash.to_lowercase())
    });
    if let Err(e) = save_changes(core, &before, &fus) {
        tracing::warn!("download followups not saved: {e}");
    }
}

/// Trashes the old files of one episode and tidies the old torrents.
async fn replace_episode(
    core: &Arc<Core>,
    client: &QbtClient,
    series: u64,
    entry: &Replacement,
    written: &[String],
    torrents: &[Torrent],
    new_hashes: &HashSet<String>,
) -> Result<(), CoreError> {
    let entry_c = entry.clone();
    let written_c = written.to_vec();
    let (paths, sources) = core
        .store
        .read(move |c| trash_candidates(c, series, &entry_c, &written_c))?;
    if paths.is_empty() {
        return Ok(());
    }
    let cands: Vec<trash::Candidate> = paths
        .iter()
        .map(|p| trash::Candidate::Series { path: p.clone() })
        .collect();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let targets = trash::targets(&cands, &sources, home.as_deref())?;
    if targets.is_empty() {
        return Ok(());
    }
    let moved = targets.clone();
    tokio::task::spawn_blocking(move || trash::move_to_trash(&moved))
        .await
        .map_err(|e| CoreError::Internal {
            message: format!("trash: {e}"),
        })??;
    core.bus.info(
        Stage::Downloads,
        format!(
            "Replaced episode {}: {} to the trash",
            entry.episode,
            plural_files(targets.len())
        ),
        EventBody::Notice,
    );
    // The torrents that carried those files
    let mut carriers = Vec::new();
    for t in torrents {
        if new_hashes.contains(&t.hash.to_lowercase()) {
            continue;
        }
        let content = Path::new(&t.content_path);
        if !t.content_path.is_empty() && targets.iter().any(|p| p.starts_with(content)) {
            let files = client.files(&t.hash).await.unwrap_or_default();
            carriers.push((t.clone(), files));
        }
    }
    for effect in plan_old(&targets, &carriers, new_hashes) {
        let outcome = match &effect {
            Effect::Remove { hash } => client.delete(std::slice::from_ref(hash), false).await,
            Effect::Skip { hash, ids } => client.set_file_priority(hash, ids, 0).await,
        };
        if let Err(e) = outcome {
            tracing::warn!("old torrent not tidied: {e}");
        }
    }
    Ok(())
}

// Reading ------------------------------------------------------------------------

/// Fills what the torrents' names and the saved sources say: the group and
/// resolution of the release, and the endpoint it was picked on.
pub fn annotate(core: &Core, torrents: &mut [Torrent]) {
    let sources: HashMap<String, String> = core
        .store
        .read(|c| Ok(settings::get::<Vec<Source>>(c, SOURCES)?.unwrap_or_default()))
        .unwrap_or_default()
        .into_iter()
        .map(|s| (s.key, s.endpoint))
        .collect();
    for t in torrents.iter_mut() {
        let f = read_name(&t.name);
        t.group = f.group;
        t.resolution = f.resolution;
        if !sources.is_empty()
            && let Some(e) = sources.get(&title_key(&t.name))
        {
            t.provider = e.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloads::qbt::tests_torrent;
    use crate::store::Store;

    fn torrent(hash: &str, name: &str, added_on: i64) -> Torrent {
        let mut t = tests_torrent();
        t.hash = hash.into();
        t.name = name.into();
        t.added_on = added_on;
        t
    }

    fn file(name: &str, progress: f64, priority: i64) -> TorrentFile {
        TorrentFile {
            name: name.into(),
            size: 100,
            progress,
            priority,
            episode: None,
        }
    }

    fn followup(title: &str, created_at: i64) -> Followup {
        Followup {
            id: 1,
            created_at,
            title: title.into(),
            hash: String::new(),
            sequential: false,
            series: Some(1),
            replace: Vec::new(),
        }
    }

    #[test]
    fn a_magnet_gives_its_hash_in_hex_or_base32() {
        let hex = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            magnet_hash(&format!("magnet:?xt=urn:btih:{}&dn=x", hex.to_uppercase())).as_deref(),
            Some(hex)
        );
        // 20 bytes of 0x00 .. 0x13 in base32
        assert_eq!(
            magnet_hash("magnet:?dn=a&xt=urn:btih:AAAQEAYEAUDAOCAJBIFQYDIOB4IBCEQT&tr=x")
                .as_deref(),
            Some("000102030405060708090a0b0c0d0e0f10111213")
        );
        assert_eq!(magnet_hash("https://nyaa.si/download/1.torrent"), None);
        assert_eq!(magnet_hash("magnet:?xt=urn:btih:short"), None);
    }

    #[test]
    fn a_name_matches_its_title_through_extension_and_punctuation() {
        let title = "[SubsPlease] Kino no Tabi - 03 (1080p) [ABCD1234].mkv";
        assert!(names_match(
            "[SubsPlease] Kino no Tabi - 03 (1080p) [ABCD1234]",
            title
        ));
        assert!(names_match(
            "[SubsPlease]_Kino_no_Tabi_-_03_(1080p)_[ABCD1234].mkv",
            title
        ));
        assert!(!names_match(
            "[SubsPlease] Kino no Tabi - 04 (1080p) [ABCD1234]",
            title
        ));
        assert!(!names_match("", title));
        // Containment needs a long name: "a" is in everything
        assert!(!names_match("Kino", title));
        assert!(names_match(&format!("{title} extra"), title));
    }

    #[test]
    fn binding_takes_the_earliest_new_torrent_with_the_name_and_never_twice() {
        let title = "[Erai-raws] Show - 03 [1080p][Multiple Subtitle]";
        let mut fus = vec![followup(title, 1000), {
            let mut f = followup(title, 1000);
            f.id = 2;
            f
        }];
        let ts = vec![
            torrent("old", title, 100),
            torrent("a", title, 1010),
            torrent("b", title, 1020),
            torrent("c", "something else entirely", 1030),
        ];
        assert!(bind(&mut fus, &ts));
        assert_eq!(
            fus[0].hash, "a",
            "an older torrent of that name is not ours"
        );
        assert_eq!(fus[1].hash, "b", "each torrent binds once");
        assert!(!bind(&mut fus, &ts), "nothing left to bind");
    }

    #[test]
    fn a_followup_with_a_hash_is_not_rebound() {
        let mut f = followup("Show - 01", 10);
        f.hash = "mine".into();
        let mut fus = vec![f];
        assert!(!bind(&mut fus, &[torrent("other", "Show - 01", 20)]));
        assert_eq!(fus[0].hash, "mine");
    }

    #[test]
    fn episodes_are_read_off_file_names() {
        assert_eq!(
            episode_of("Kino/[G] Kino no Tabi - 03 (1080p).mkv"),
            Some(3)
        );
        assert_eq!(episode_of("[G] Kino no Tabi (01-12) [Batch]"), None);
        assert_eq!(episode_of("[G] Kino no Tabi - 03.5 (1080p).mkv"), None);
        assert_eq!(episode_of("random.mkv"), None);
    }

    #[test]
    fn a_batch_episode_is_ready_when_its_own_file_is_done_and_wanted() {
        let t = torrent("h", "[G] Show (01-03)", 0);
        let files = vec![
            file("Show/[G] Show - 01 (1080p).mkv", 1.0, 1),
            file("Show/[G] Show - 02 (1080p).mkv", 0.4, 1),
            file("Show/[G] Show - 03 (1080p).mkv", 1.0, 0),
        ];
        assert!(episode_ready(1, &t, &files));
        assert!(!episode_ready(2, &t, &files), "still downloading");
        assert!(!episode_ready(3, &t, &files), "skipped files never count");
        assert!(!episode_ready(4, &t, &files), "not in the batch");
    }

    #[test]
    fn a_single_file_is_ready_only_when_the_torrent_is_whole_and_the_name_agrees() {
        let mut t = torrent("h", "[G] Show - 02 (1080p).mkv", 0);
        t.progress = 0.99;
        assert!(!episode_ready(2, &t, &[]));
        t.progress = 1.0;
        assert!(episode_ready(2, &t, &[]));
        assert!(episode_ready(
            2,
            &t,
            &[file("[G] Show - 02 (1080p).mkv", 1.0, 1)]
        ));
        assert!(!episode_ready(
            3,
            &t,
            &[file("[G] Show - 02 (1080p).mkv", 1.0, 1)]
        ));
    }

    #[test]
    fn what_the_new_torrent_wrote_is_never_an_old_file() {
        let old = vec!["/lib/Show/a.mkv".to_string(), "/lib/Show/b.mkv".to_string()];
        let written = vec!["/lib/Show/b.mkv".to_string()];
        assert_eq!(
            old_to_trash(&old, &written),
            vec!["/lib/Show/a.mkv".to_string()]
        );
        assert_eq!(old_to_trash(&old, &[]), old);
    }

    #[test]
    fn an_old_torrent_whose_files_all_went_leaves_and_one_with_others_skips_them() {
        let mut single = torrent("single", "[G] Show - 02 (1080p).mkv", 0);
        single.save_path = "/dl".into();
        single.content_path = "/dl/[G] Show - 02 (1080p).mkv".into();
        let mut batch = torrent("batch", "[G] Show (01-03)", 0);
        batch.save_path = "/dl".into();
        batch.content_path = "/dl/Show".into();
        let batch_files = vec![
            file("Show/e01.mkv", 1.0, 1),
            file("Show/e02.mkv", 1.0, 1),
            file("Show/e03.mkv", 1.0, 1),
        ];
        let mut fresh = torrent("fresh", "[H] Show - 02", 0);
        fresh.save_path = "/dl".into();
        fresh.content_path = "/dl/[H] Show - 02.mkv".into();
        let trashed = vec![
            PathBuf::from("/dl/[G] Show - 02 (1080p).mkv"),
            PathBuf::from("/dl/Show/e02.mkv"),
            PathBuf::from("/dl/[H] Show - 02.mkv"),
        ];
        let pairs = vec![
            (single, Vec::new()),
            (batch, batch_files),
            (fresh, Vec::new()),
        ];
        let keep: HashSet<String> = ["fresh".to_string()].into();
        let plan = plan_old(&trashed, &pairs, &keep);
        assert_eq!(
            plan,
            vec![
                Effect::Remove {
                    hash: "single".into()
                },
                Effect::Skip {
                    hash: "batch".into(),
                    ids: vec![1]
                },
            ],
            "the new torrent is never touched, the batch only loses episode 2"
        );
    }

    #[test]
    fn a_torrent_with_no_trashed_file_is_left_alone() {
        let mut t = torrent("h", "x", 0);
        t.save_path = "/dl".into();
        t.content_path = "/dl/x.mkv".into();
        assert!(
            plan_old(
                &[PathBuf::from("/dl/other.mkv")],
                &[(t, Vec::new())],
                &HashSet::new()
            )
            .is_empty()
        );
    }

    fn eps(list: &[u32]) -> BTreeSet<u32> {
        list.iter().copied().collect()
    }

    #[test]
    fn cancelling_takes_a_torrent_only_when_every_episode_it_fetches_is_replaced() {
        let mut t = torrent("h", "[G] Show (01-03)", 0);
        t.progress = 0.3;
        let files = [
            file("S/e01.mkv", 1.0, 1),
            file("S/[G] Show - 02.mkv", 0.2, 1),
            file("S/[G] Show - 03.mkv", 0.0, 0),
        ];
        // Episodes 1 and 2 are fetched (3 is skipped) and both are replaced
        assert_eq!(
            cancel_plan(
                &eps(&[1, 2]),
                &t,
                &[
                    file("S/[G] Show - 01.mkv", 1.0, 1),
                    files[1].clone(),
                    files[2].clone()
                ]
            ),
            CancelPlan::Whole
        );
        // Replacing only 2: the rest of the batch goes on, file 2 stops
        let files = vec![
            file("S/[G] Show - 01.mkv", 1.0, 1),
            file("S/[G] Show - 02.mkv", 0.2, 1),
        ];
        assert_eq!(
            cancel_plan(&eps(&[2]), &t, &files),
            CancelPlan::Skip(vec![1])
        );
        // Nothing of it is being replaced
        assert_eq!(cancel_plan(&eps(&[7]), &t, &files), CancelPlan::Nothing);
        // A file whose episode cannot be read keeps the whole torrent
        let odd = vec![
            file("S/[G] Show - 02.mkv", 0.2, 1),
            file("S/extras.mkv", 0.2, 1),
        ];
        assert_eq!(cancel_plan(&eps(&[2]), &t, &odd), CancelPlan::Skip(vec![0]));
    }

    #[test]
    fn cancelling_a_single_episode_torrent_and_never_a_finished_one() {
        let mut t = torrent("h", "[G] Show - 05 (1080p).mkv", 0);
        t.progress = 0.5;
        assert_eq!(cancel_plan(&eps(&[5]), &t, &[]), CancelPlan::Whole);
        assert_eq!(cancel_plan(&eps(&[4]), &t, &[]), CancelPlan::Nothing);
        assert_eq!(cancel_plan(&eps(&[]), &t, &[]), CancelPlan::Nothing);
        t.progress = 1.0;
        assert_eq!(cancel_plan(&eps(&[5]), &t, &[]), CancelPlan::Nothing);
        // A name that says no episode is never cancelled
        let mut u = torrent("u", "Some Film (2020)", 0);
        u.progress = 0.1;
        assert_eq!(cancel_plan(&eps(&[1]), &u, &[]), CancelPlan::Nothing);
    }

    /// A library with one series, a file per episode, and an extra, in a
    /// real directory so the file checks run.
    fn library() -> (tempfile::TempDir, std::sync::Arc<Store>, Vec<String>) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Anime");
        let show = root.join("Show");
        std::fs::create_dir_all(&show).unwrap();
        let store = Store::open(&dir.path().join("a.db")).unwrap();
        let mut paths = Vec::new();
        for n in 1..=3 {
            let p = show.join(format!("[Old] Show - 0{n}.mkv"));
            std::fs::write(&p, b"x").unwrap();
            paths.push(p.to_string_lossy().into_owned());
        }
        let extra = show.join("NCOP.mkv");
        std::fs::write(&extra, b"x").unwrap();
        paths.push(extra.to_string_lossy().into_owned());
        let (r, s, ps) = (
            root.to_string_lossy().into_owned(),
            show.to_string_lossy().into_owned(),
            paths.clone(),
        );
        store
            .write(move |c| {
                c.execute("INSERT INTO sources (id, path, added_at) VALUES (1, ?1, 0)", [&r])?;
                c.execute(
                    "INSERT INTO series (id, source_id, kind, path, folder_name, added_at) VALUES (1, 1, 'show', ?1, 'Show', 0)",
                    [&s],
                )?;
                for (i, p) in ps.iter().enumerate() {
                    let (kind, number): (&str, Option<f64>) =
                        if i < 3 { ("episode", Some(i as f64 + 1.0)) } else { ("extra", None) };
                    c.execute(
                        "INSERT INTO files (series_id, path, size, mtime, kind, number, label, episode_key, seen_at) VALUES (1, ?1, 1, 0, ?2, ?3, 'l', 'k', 0)",
                        params![p, kind, number],
                    )?;
                }
                Ok(())
            })
            .unwrap();
        (dir, store, paths)
    }

    #[test]
    fn only_the_listed_episode_files_are_candidates() {
        let (_d, store, paths) = library();
        // The request names episode 2's file and, wrongly, an extra and a stranger
        let entry = Replacement {
            episode: 2,
            old: vec![paths[1].clone(), paths[3].clone(), "/etc/passwd".into()],
        };
        let (got, sources) = store.read(|c| trash_candidates(c, 1, &entry, &[])).unwrap();
        assert_eq!(
            got,
            vec![paths[1].clone()],
            "an extra and a stranger are not episode 2"
        );
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn a_file_the_new_torrent_wrote_or_one_already_gone_is_not_a_candidate() {
        let (_d, store, paths) = library();
        let entry = Replacement {
            episode: 1,
            old: vec![paths[0].clone()],
        };
        let (got, _) = store
            .read(|c| trash_candidates(c, 1, &entry, std::slice::from_ref(&paths[0])))
            .unwrap();
        assert!(got.is_empty(), "the new file landed on the old path");
        std::fs::remove_file(&paths[0]).unwrap();
        let (got, _) = store.read(|c| trash_candidates(c, 1, &entry, &[])).unwrap();
        assert!(got.is_empty(), "already gone");
        // Another series' episode 1 is no candidate for this one
        let (got, _) = store.read(|c| trash_candidates(c, 2, &entry, &[])).unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn a_directory_in_a_listed_path_is_never_trashed() {
        let (_d, store, paths) = library();
        std::fs::remove_file(&paths[2]).unwrap();
        std::fs::create_dir(&paths[2]).unwrap();
        let entry = Replacement {
            episode: 3,
            old: vec![paths[2].clone()],
        };
        let (got, _) = store.read(|c| trash_candidates(c, 1, &entry, &[])).unwrap();
        assert!(got.is_empty());
    }

    fn release(title: &str, link: &str) -> Release {
        Release {
            title: title.into(),
            link: link.into(),
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
        }
    }

    #[test]
    fn prepare_reads_the_old_files_once_per_item_and_skips_what_asks_for_nothing() {
        let (_d, store, paths) = library();
        let hex = "0123456789abcdef0123456789abcdef01234567";
        let items = vec![
            release("[New] Show - 02", &format!("magnet:?xt=urn:btih:{hex}")),
            release("[New] Show (01-03)", "https://x/y.torrent"),
            release("[New] Show - 09", "https://x/z.torrent"),
        ];
        let options = DownloadOptions {
            sequential: false,
            provider: "erai".into(),
            series: Some(1),
            replace: vec![
                ReplaceItem {
                    item: 0,
                    episodes: vec![2],
                },
                // Episodes 1, 3 and 7 (the library has no 7), repeated
                ReplaceItem {
                    item: 1,
                    episodes: vec![3, 1, 7, 1],
                },
            ],
            cancel: Vec::new(),
        };
        let got = prepare(&store, &items, &options).unwrap();
        assert_eq!(got.len(), 2, "item 2 asks for nothing");
        assert_eq!(got[0].hash, hex);
        assert_eq!(
            got[0].replace,
            vec![Replacement {
                episode: 2,
                old: vec![paths[1].clone()]
            }]
        );
        assert_eq!(got[1].hash, "");
        assert_eq!(
            got[1].replace,
            vec![
                Replacement {
                    episode: 1,
                    old: vec![paths[0].clone()]
                },
                Replacement {
                    episode: 3,
                    old: vec![paths[2].clone()]
                },
            ],
            "sorted, deduplicated, and an episode with no file is no replacement"
        );
    }

    #[test]
    fn prepare_with_download_in_order_makes_a_followup_for_every_item() {
        let (_d, store, _) = library();
        let items = vec![release("a", "x"), release("b", "y")];
        let options = DownloadOptions {
            sequential: true,
            ..Default::default()
        };
        let got = prepare(&store, &items, &options).unwrap();
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|f| f.sequential && f.replace.is_empty()));
        // Nothing asked for: nothing to follow
        assert!(
            prepare(&store, &items, &DownloadOptions::default())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn prepare_refuses_an_item_that_is_not_there() {
        let (_d, store, _) = library();
        let options = DownloadOptions {
            series: Some(1),
            replace: vec![ReplaceItem {
                item: 5,
                episodes: vec![1],
            }],
            ..Default::default()
        };
        assert!(prepare(&store, &[release("a", "x")], &options).is_err());
    }

    #[test]
    fn a_show_not_in_the_library_has_nothing_to_replace() {
        let (_d, store, _) = library();
        let options = DownloadOptions {
            series: None,
            replace: vec![ReplaceItem {
                item: 0,
                episodes: vec![1],
            }],
            ..Default::default()
        };
        assert!(
            prepare(&store, &[release("a", "x")], &options)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn changes_merge_by_id_and_leave_a_followup_added_meanwhile() {
        let a = Followup {
            id: 1,
            ..followup("a", 1)
        };
        let b = Followup {
            id: 2,
            ..followup("b", 1)
        };
        let c = Followup {
            id: 3,
            ..followup("c", 2)
        };
        // The tick bound a and retired b; c arrived while it ran
        let bound = Followup {
            hash: "h".into(),
            ..a.clone()
        };
        let changes = diff(&[a.clone(), b.clone()], std::slice::from_ref(&bound));
        assert_eq!(changes.len(), 2);
        let got = merge(vec![a, b, c.clone()], &changes);
        assert_eq!(got, vec![bound, c]);
        // A tick that changed nothing changes nothing
        assert!(diff(&[followup("x", 1)], &[followup("x", 1)]).is_empty());
    }
}
