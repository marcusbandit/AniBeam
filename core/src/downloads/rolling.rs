//! Rolling download: a long show is fetched a window of episodes at a time,
//! so a 170-episode run never sits on the disk whole.
//!
//! A roll is one download held to a window. Its episodes are kept in order:
//! the files of a batch torrent in name order, or the single-episode
//! releases of a pick in episode order. The window starts after the last
//! episode watched since the roll began and reaches `ahead` episodes past
//! it, or further when the user asked for more (`until`). What the window
//! does to an episode depends on its shape:
//!
//! - a file of a batch is set to "do not download" in qBittorrent outside
//!   the window and back to normal inside it, the next one to watch at top
//!   priority so it finishes first;
//! - a single-episode release is held in the roll and only added to
//!   qBittorrent once the window reaches it.
//!
//! Only a completion recorded since the roll began counts as watched, so a
//! show seen once before and fetched again to rewatch starts at the start,
//! and a tracker's progress never moves the window.
//!
//! With `delete_watched`, a watched episode leaves the disk once a later
//! one is watched, so the one just finished is still there to go back to.
//! A single-episode torrent is removed with its file. A batch file is
//! deleted from the disk only when the client keeps its files on this
//! machine and the path sits inside the torrent's save folder; the torrent
//! is then rechecked so qBittorrent stops offering pieces it no longer has.
//!
//! Rolls are kept as JSON in the settings table. A pass runs right after
//! the add (a short burst until each torrent is found), on every torrent
//! read, when the player records an episode as finished, and when the user
//! changes a roll. The passes nobody asked for at that moment, the torrent
//! read's and the player's, never run in a shell that disabled unattended
//! changes; the ones the user's own download or click starts do, since
//! they act on that user's roll.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use super::qbt::QbtClient;
use super::replace::{magnet_hash, names_match, title_key};
use super::{facts, settings};
use crate::contract::*;
use crate::core::Core;
use crate::store::settings as kv;
use crate::time::now_secs;

/// The rolls, as JSON in the settings table.
pub const ROLLS: &str = "download_rolls";

/// A torrent counts as this roll's if the client stamped it up to this long
/// before the roll was made (clock skew on a remote host).
const BIND_SLACK_SECS: i64 = 300;
/// A roll whose torrent never showed up in the client goes after this long.
const UNBOUND_AGE_SECS: i64 = 3 * 24 * 3600;
/// The most a window may reach ahead, so a typo cannot fetch a whole run.
pub const MAX_AHEAD: u32 = 500;

/// qBittorrent's file priorities: skipped, normal and maximal.
const SKIP: i64 = 0;
const NORMAL: i64 = 1;
const FIRST: i64 = 7;

const VIDEO: &[&str] = &[
    "mkv", "mp4", "avi", "webm", "m4v", "mov", "ts", "m2ts", "wmv", "flv", "ogm",
];

/// One rolling download.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Roll {
    pub id: u64,
    pub created_at: i64,
    /// What the downloads page and the log call it.
    pub title: String,
    /// The folder name the download was given, for the releases added later.
    #[serde(default)]
    pub name: Option<String>,
    pub ahead: u32,
    pub delete_watched: bool,
    /// Rolls only if, once its files are counted, it holds more episodes
    /// than this; `None` rolls whatever the count.
    #[serde(default)]
    pub over: Option<u32>,
    /// The pre-download mark: every episode up to this one is wanted.
    #[serde(default)]
    pub until: Option<Key>,
    /// Stop rolling: fetch everything, then drop the roll.
    #[serde(default)]
    pub released: bool,
    /// Turn on download in order for each torrent as the client lists it,
    /// the held singles added later included.
    #[serde(default)]
    pub sequential: bool,
    pub parts: Vec<Part>,
}

/// An episode's place in a roll: its part, then its unit in that part. Both
/// are fixed once known, so a key stays put as the window moves.
pub type Key = (usize, usize);

/// One release of a roll: a batch torrent or a single-episode one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Part {
    pub release: Release,
    /// The torrent's info-hash, lowercase; empty until found.
    #[serde(default)]
    pub hash: String,
    /// Handed to the client. A held single is not, yet.
    pub added: bool,
    /// When it was handed over; a part the client never lists goes after
    /// a while counted from here.
    #[serde(default)]
    pub added_at: i64,
    /// One episode in the torrent, rather than a batch.
    pub single: bool,
    /// The episodes, once known: a single's at once, a batch's when the
    /// client has its file list.
    #[serde(default)]
    pub units: Vec<Unit>,
    #[serde(default)]
    pub counted: bool,
    /// The client listed the torrent at least once.
    #[serde(default)]
    pub seen: bool,
    /// The torrent left the client: the user removed it, or the roll did.
    #[serde(default)]
    pub gone: bool,
}

/// One episode of a part.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    /// The file's index in the torrent; `None` for a single's whole torrent.
    pub file: Option<usize>,
    pub name: String,
    #[serde(default)]
    pub watched: bool,
    /// Deleted by the roll.
    #[serde(default)]
    pub gone: bool,
}

// Planning -----------------------------------------------------------------------

/// A download split into what goes to the client now and the rolls that
/// hold the rest.
#[derive(Debug, Default, PartialEq)]
pub struct Plan {
    pub now: Vec<Release>,
    pub rolls: Vec<Roll>,
}

/// Whether and how a download rolls. A pick of single-episode releases
/// rolls as one roll when asked to, or by the settings when it is more than
/// `over` episodes: only the first window is added now. A batch rolls on its
/// own and is added whole, with its count judged once the client has its
/// file list. A pick that replaces episodes never rolls.
pub fn plan(
    items: &[Release],
    options: &DownloadOptions,
    s: &RollingSettings,
    name: Option<&str>,
    now: i64,
) -> Plan {
    let wish = options.rolling;
    if wish == Some(false) || (wish.is_none() && !s.big) || !options.replace.is_empty() {
        return Plan {
            now: items.to_vec(),
            rolls: Vec::new(),
        };
    }
    let ahead = s.ahead.clamp(1, MAX_AHEAD);
    let read: Vec<ReleaseFacts> = items.iter().map(facts::read).collect();
    let single = |f: &ReleaseFacts| !f.batch && f.episode.is_some();
    let base = |title: String, over: Option<u32>, parts: Vec<Part>| Roll {
        id: 0,
        created_at: now,
        title,
        name: name.map(str::to_string),
        ahead,
        delete_watched: s.delete_watched,
        over,
        until: None,
        released: false,
        sequential: options.sequential,
        parts,
    };
    let (singles, others): (Vec<usize>, Vec<usize>) =
        (0..items.len()).partition(|&i| single(&read[i]));
    let over = if wish == Some(true) {
        None
    } else {
        Some(s.over)
    };
    // Every batch rolls on its own, judged by its count once it is known
    let mut rolls: Vec<Roll> = others
        .iter()
        .map(|&i| {
            let r = &items[i];
            let part = Part {
                release: r.clone(),
                hash: magnet_hash(&r.link).unwrap_or_default(),
                added: true,
                added_at: now,
                single: false,
                units: Vec::new(),
                counted: false,
                seen: false,
                gone: false,
            };
            let title = name
                .map(str::to_string)
                .unwrap_or_else(|| show_title(&r.title));
            base(title, over, vec![part])
        })
        .collect();
    let mut now_items: Vec<Release> = others.iter().map(|&i| items[i].clone()).collect();
    // The single episodes of the pick roll together, in episode order
    let roll_singles = singles.len() > 1 && (wish == Some(true) || singles.len() > s.over as usize);
    if !roll_singles {
        return Plan {
            now: items.to_vec(),
            rolls,
        };
    }
    let mut order = singles;
    order.sort_by(|&a, &b| {
        let (fa, fb) = (&read[a], &read[b]);
        fa.season.unwrap_or(1).cmp(&fb.season.unwrap_or(1)).then(
            fa.episode
                .partial_cmp(&fb.episode)
                .unwrap_or(Ordering::Equal),
        )
    });
    let parts: Vec<Part> = order
        .iter()
        .enumerate()
        .map(|(n, &i)| {
            let r = &items[i];
            Part {
                release: r.clone(),
                hash: magnet_hash(&r.link).unwrap_or_default(),
                added: n < ahead as usize,
                added_at: now,
                single: true,
                units: vec![Unit {
                    file: None,
                    name: r.title.clone(),
                    watched: false,
                    gone: false,
                }],
                counted: true,
                seen: false,
                gone: false,
            }
        })
        .collect();
    now_items.extend(parts.iter().filter(|p| p.added).map(|p| p.release.clone()));
    let title = name
        .map(str::to_string)
        .unwrap_or_else(|| show_title(&items[order[0]].title));
    rolls.push(base(title, None, parts));
    Plan {
        now: now_items,
        rolls,
    }
}

/// A line for the job's message: what the download holds back.
pub fn describe(plan: &Plan) -> String {
    let held: usize = plan
        .rolls
        .iter()
        .flat_map(|r| &r.parts)
        .filter(|p| !p.added)
        .count();
    match plan.rolls.first() {
        None => String::new(),
        Some(r) if held > 0 => format!(
            ", rolling {} ahead with {} waiting",
            r.ahead,
            plural(held, "episode", "episodes")
        ),
        Some(r) if r.over.is_some() => format!(
            ", rolling {} ahead if it holds more than {} episodes",
            r.ahead,
            r.over.unwrap_or_default()
        ),
        Some(r) => format!(", rolling {} ahead", r.ahead),
    }
}

/// A release title without its group, episode and tags: "[Erai-raws] Black
/// Clover - 001 [1080p]" reads "Black Clover".
fn show_title(title: &str) -> String {
    let mut s = title.trim().to_string();
    while s.starts_with('[') {
        match s.find(']') {
            Some(end) => s = s[end + 1..].trim_start().to_string(),
            None => break,
        }
    }
    let cut = [" - ", " [", " ("]
        .iter()
        .filter_map(|m| s.find(m))
        .min()
        .unwrap_or(s.len());
    let t = s[..cut].trim();
    if t.is_empty() {
        title.trim().to_string()
    } else {
        t.to_string()
    }
}

/// The episodes of a batch: its video files in name order, numbers read as
/// numbers ("Episode 2" before "Episode 10").
pub fn units_of(files: &[TorrentFile]) -> Vec<Unit> {
    let mut v: Vec<(usize, &TorrentFile)> = files
        .iter()
        .enumerate()
        .filter(|(_, f)| is_video(&f.name))
        .collect();
    v.sort_by(|a, b| natural_cmp(&a.1.name, &b.1.name));
    v.into_iter()
        .map(|(i, f)| Unit {
            file: Some(i),
            name: f.name.clone(),
            watched: false,
            gone: false,
        })
        .collect()
}

fn is_video(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| VIDEO.contains(&e.to_ascii_lowercase().as_str()))
}

/// Compares names with runs of digits read as numbers.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut s = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        s.push(c);
                        it.next();
                    }
                    s
                };
                let (m, n) = (take(&mut x), take(&mut y));
                let (mt, nt) = (m.trim_start_matches('0'), n.trim_start_matches('0'));
                let o = mt.len().cmp(&nt.len()).then_with(|| mt.cmp(nt));
                if o != Ordering::Equal {
                    return o;
                }
            }
            (Some(c), Some(d)) => {
                let o = c.to_lowercase().cmp(d.to_lowercase());
                if o != Ordering::Equal {
                    return o;
                }
                x.next();
                y.next();
            }
        }
    }
}

// The window ------------------------------------------------------------------

/// The episodes of a roll in order, as far as they are known: every
/// counted part's, up to the first part not counted yet.
pub fn order(roll: &Roll) -> Vec<Key> {
    let mut out = Vec::new();
    for (p, part) in roll.parts.iter().enumerate() {
        if !part.counted {
            break;
        }
        out.extend((0..part.units.len()).map(|u| (p, u)));
    }
    out
}

fn unit(roll: &Roll, k: Key) -> &Unit {
    &roll.parts[k.0].units[k.1]
}

/// Where the window stands, by position in `order`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    /// The last episode watched since the roll began.
    pub last: Option<usize>,
    /// The next episode to watch.
    pub start: usize,
    /// One past the last episode the window reaches.
    pub end: usize,
    pub total: usize,
}

pub fn window(roll: &Roll, keys: &[Key]) -> Window {
    let total = keys.len();
    let last = keys.iter().rposition(|&k| unit(roll, k).watched);
    let start = last.map_or(0, |p| p + 1);
    if roll.released {
        return Window {
            last,
            start,
            end: total,
            total,
        };
    }
    let mut end = start + roll.ahead.max(1) as usize;
    if let Some(mark) = roll.until
        && let Some(p) = keys.iter().position(|&k| k == mark)
    {
        end = end.max(p + 1);
    }
    Window {
        last,
        start,
        end: end.min(total),
        total,
    }
}

/// What a pass does to each episode.
#[derive(Debug, Default, PartialEq)]
pub struct Moves {
    /// Batch files, by part, with the priority each should have.
    pub priorities: HashMap<usize, Vec<(usize, i64)>>,
    /// Held singles to add now, by part.
    pub add: Vec<usize>,
    /// Watched episodes to take off the disk.
    pub delete: Vec<Key>,
}

/// The moves that put a roll's episodes where the window says. Inside the
/// window an episode is wanted, the next one first; the last one watched
/// keeps its file; everything else is skipped (a held single simply stays
/// held). Deletion takes the watched episodes before the last one.
pub fn moves(roll: &Roll) -> Moves {
    let keys = order(roll);
    let w = window(roll, &keys);
    let mut m = Moves::default();
    for (pos, &k) in keys.iter().enumerate() {
        let (part, u) = (&roll.parts[k.0], unit(roll, k));
        if part.gone {
            continue;
        }
        // A file the roll deleted stays skipped, or the client fetches it again
        if u.gone {
            if let (false, Some(file)) = (part.single, u.file) {
                m.priorities.entry(k.0).or_default().push((file, SKIP));
            }
            continue;
        }
        let wanted = if roll.released {
            !u.watched || Some(pos) == w.last
        } else {
            (pos >= w.start && pos < w.end) || Some(pos) == w.last
        };
        if part.single {
            if wanted && !part.added {
                m.add.push(k.0);
            }
        } else if let Some(file) = u.file {
            let priority = if !wanted {
                SKIP
            } else if pos == w.start && !roll.released {
                FIRST
            } else {
                NORMAL
            };
            m.priorities.entry(k.0).or_default().push((file, priority));
        }
        if roll.delete_watched
            && u.watched
            && w.last.is_some_and(|last| pos < last)
            && (part.added || !part.single)
        {
            m.delete.push(k);
        }
    }
    m
}

/// The priority changes a batch needs: the files whose current priority
/// differs from the wish, grouped by the priority to set.
pub fn priority_changes(wish: &[(usize, i64)], files: &[TorrentFile]) -> Vec<(i64, Vec<usize>)> {
    let mut by: HashMap<i64, Vec<usize>> = HashMap::new();
    for &(i, p) in wish {
        if files.get(i).is_some_and(|f| f.priority != p) {
            by.entry(p).or_default().push(i);
        }
    }
    let mut out: Vec<(i64, Vec<usize>)> = by.into_iter().collect();
    out.sort_by_key(|(p, _)| *p);
    out
}

/// The roll as the downloads page shows it.
pub fn view(roll: &Roll) -> RollView {
    let keys = order(roll);
    let w = window(roll, &keys);
    let total: usize = roll.parts.iter().map(|p| p.units.len()).sum();
    RollView {
        id: roll.id,
        ahead: roll.ahead,
        next: (w.start + 1).min(total.max(1)) as u64,
        through: w.end as u64,
        total: total as u64,
        watched: roll
            .parts
            .iter()
            .flat_map(|p| &p.units)
            .filter(|u| u.watched)
            .count() as u64,
        delete_watched: roll.delete_watched,
    }
}

/// Moves the pre-download mark `count` episodes past the window's end.
pub fn more(roll: &mut Roll, count: u32) {
    let keys = order(roll);
    if keys.is_empty() {
        return;
    }
    let w = window(roll, &keys);
    let to = (w.end + count as usize).min(keys.len());
    if to > 0 {
        roll.until = Some(keys[to - 1]);
    }
}

// Binding and finishing --------------------------------------------------------

/// Binds each added part with no hash to its torrent: the earliest one added
/// since the roll was made whose name matches the release and that no
/// other part holds.
pub fn bind(rolls: &mut [Roll], torrents: &[Torrent]) {
    let mut taken: HashSet<String> = rolls
        .iter()
        .flat_map(|r| &r.parts)
        .filter(|p| !p.hash.is_empty())
        .map(|p| p.hash.clone())
        .collect();
    for r in rolls.iter_mut() {
        let since = r.created_at - BIND_SLACK_SECS;
        for p in r.parts.iter_mut().filter(|p| p.added && p.hash.is_empty()) {
            let found = torrents
                .iter()
                .filter(|t| {
                    !taken.contains(&t.hash.to_lowercase())
                        && t.added_on >= since
                        && same_release(&t.name, &p.release.title)
                })
                .min_by_key(|t| t.added_on);
            if let Some(t) = found {
                p.hash = t.hash.to_lowercase();
                taken.insert(p.hash.clone());
            }
        }
    }
}

/// A torrent's name is the release's: the names match, or the torrent is
/// named for the show alone (a batch saved as "Black Clover" from the
/// release "[Judas] Black Clover (01-170) [Batch]").
fn same_release(torrent_name: &str, title: &str) -> bool {
    names_match(torrent_name, title) || {
        let (a, b) = (title_key(torrent_name), title_key(&show_title(title)));
        !a.is_empty() && a == b
    }
}

/// Gives the parts just handed to the client the hashes the add answered
/// with, in the order they were added, when it answered one for each.
pub fn take_hashes(parts: &mut [&mut Part], hashes: &[String]) {
    if hashes.len() != parts.len() {
        return;
    }
    for (p, h) in parts.iter_mut().zip(hashes) {
        let h = h.trim().to_lowercase();
        if p.hash.is_empty() && h.len() == 40 && h.chars().all(|c| c.is_ascii_hexdigit()) {
            p.hash = h;
        }
    }
}

/// A roll with nothing left to do: every torrent it added gone, one whose
/// torrent never showed, or one whose every episode was watched (and,
/// when it deletes them, deleted all but the last).
pub fn finished(roll: &Roll, now: i64) -> bool {
    let added: Vec<&Part> = roll.parts.iter().filter(|p| p.added).collect();
    if !added.is_empty() && added.iter().all(|p| p.gone) {
        return true;
    }
    if roll
        .parts
        .iter()
        .any(|p| p.added && !p.seen && now - p.added_at.max(roll.created_at) > UNBOUND_AGE_SECS)
    {
        return true;
    }
    let keys = order(roll);
    let all_counted = roll.parts.iter().all(|p| p.counted);
    if !all_counted || keys.is_empty() {
        return false;
    }
    let done = |&(p, u): &Key| {
        let unit = &roll.parts[p].units[u];
        unit.watched || unit.gone || roll.parts[p].gone
    };
    keys.iter().all(done) && (!roll.delete_watched || moves(roll).delete.is_empty())
}

// Watched ---------------------------------------------------------------------------

/// Whether the library has the file at `path` (or, with `under`, any
/// episode file inside that folder) as an episode completed at or after
/// `since`.
pub fn watched_since(
    c: &Connection,
    path: &str,
    under: bool,
    since: i64,
) -> Result<bool, CoreError> {
    let trimmed = path.trim_end_matches('/');
    let like = format!(
        "{}/%",
        trimmed
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    let found: Option<i64> = c
        .query_row(
            "SELECT 1 FROM files f JOIN completed c
               ON c.series_id = f.series_id AND c.episode_key = f.episode_key
             WHERE f.kind = 'episode' AND c.at >= ?3
               AND (f.path = ?1 OR (?2 AND f.path LIKE ?4 ESCAPE '\\'))
             LIMIT 1",
            params![trimmed, under, since, like],
            |r| r.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

fn seen_watched(core: &Core, path: &str, under: bool, since: i64) -> bool {
    let mut paths = vec![path.to_string()];
    if let Ok(real) = std::fs::canonicalize(path) {
        let real = real.to_string_lossy().into_owned();
        if real != path {
            paths.push(real);
        }
    }
    paths.into_iter().any(|p| {
        core.store
            .read(move |c| watched_since(c, &p, under, since))
            .unwrap_or(false)
    })
}

/// Where a batch file sits on the disk, refused when its name climbs out
/// of the save folder.
pub fn file_path(save_path: &str, name: &str) -> Option<PathBuf> {
    let rel = Path::new(name);
    if save_path.trim().is_empty()
        || rel.is_absolute()
        || rel.components().any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    Some(Path::new(save_path.trim()).join(rel))
}

// Storage ------------------------------------------------------------------------

pub fn load(core: &Core) -> Result<Vec<Roll>, CoreError> {
    core.store
        .read(|c| Ok(kv::get::<Vec<Roll>>(c, ROLLS)?.unwrap_or_default()))
}

/// What the user sets on a roll; a pass never changes these.
fn users_part(r: &Roll) -> (u32, bool, Option<Key>, bool) {
    (r.ahead, r.delete_watched, r.until, r.released)
}

/// Applies a pass's changes by id onto what is stored now. A roll added
/// while the pass ran is kept as it is, and one the user changed meanwhile
/// (`SetRoll`, `RollMore`) keeps the change: the pass's copy takes the
/// user's settings, and a roll the pass would drop stays for the next pass.
fn save_changes(core: &Core, before: &[Roll], after: &[Roll]) -> Result<(), CoreError> {
    let mut changes: HashMap<u64, (Roll, Option<Roll>)> = HashMap::new();
    for b in before {
        match after.iter().find(|a| a.id == b.id) {
            None => {
                changes.insert(b.id, (b.clone(), None));
            }
            Some(a) if a != b => {
                changes.insert(b.id, (b.clone(), Some(a.clone())));
            }
            _ => {}
        }
    }
    if changes.is_empty() {
        return Ok(());
    }
    core.store.write(move |c| {
        let cur = kv::get::<Vec<Roll>>(c, ROLLS)?.unwrap_or_default();
        let next: Vec<Roll> = cur
            .into_iter()
            .filter_map(|r| match changes.get(&r.id) {
                None => Some(r),
                Some((b, change)) => {
                    let edited = users_part(&r) != users_part(b);
                    match change {
                        None if edited => Some(r),
                        None => None,
                        Some(new) => {
                            let mut n = new.clone();
                            (n.ahead, n.delete_watched, n.until, n.released) = users_part(&r);
                            Some(n)
                        }
                    }
                }
            })
            .collect();
        kv::set(c, ROLLS, &next)
    })
}

/// Saves new rolls, numbering them after the stored ones.
async fn add_rolls(core: &Arc<Core>, rolls: Vec<Roll>) -> Result<(), CoreError> {
    core.store
        .write_async(move |c| {
            let mut cur = kv::get::<Vec<Roll>>(c, ROLLS)?.unwrap_or_default();
            let mut next = cur.iter().map(|r| r.id).max().unwrap_or(0);
            for mut r in rolls {
                next += 1;
                r.id = next;
                cur.push(r);
            }
            kv::set(c, ROLLS, &cur)
        })
        .await
}

/// Changes one stored roll in place.
fn edit(core: &Core, id: u64, f: impl FnOnce(&mut Roll) + Send + 'static) -> Result<(), CoreError> {
    core.store.write(move |c| {
        let mut cur = kv::get::<Vec<Roll>>(c, ROLLS)?.unwrap_or_default();
        let Some(r) = cur.iter_mut().find(|r| r.id == id) else {
            return Err(CoreError::invalid(
                "id",
                format!("no rolling download {id}"),
            ));
        };
        f(r);
        kv::set(c, ROLLS, &cur)
    })
}

// The pass ----------------------------------------------------------------------------

/// One pass at a time: the page's poll, the burst after an add and the
/// player's nudge all run it.
static PASS: LazyLock<tokio::sync::Mutex<()>> = LazyLock::new(|| tokio::sync::Mutex::new(()));

/// One pass over the stored rolls against a fresh torrent read.
pub async fn tick(core: &Arc<Core>, client: &QbtClient, torrents: &[Torrent]) {
    let _one = PASS.lock().await;
    let before = match load(core) {
        Ok(r) if !r.is_empty() => r,
        Ok(_) => return,
        Err(e) => {
            tracing::warn!("rolling downloads not read: {e}");
            return;
        }
    };
    let here = core
        .store
        .read(settings::load_stored)
        .map(|s| settings::files_here(&s))
        .unwrap_or(false);
    let mut rolls = before.clone();
    bind(&mut rolls, torrents);
    let now = now_secs();
    let mut drop = HashSet::new();
    for roll in rolls.iter_mut() {
        if step(core, client, roll, torrents, here).await == Step::Drop {
            drop.insert(roll.id);
        }
    }
    rolls.retain(|r| !drop.contains(&r.id) && !finished(r, now));
    if let Err(e) = save_changes(core, &before, &rolls) {
        tracing::warn!("rolling downloads not saved: {e}");
    }
}

#[derive(PartialEq)]
enum Step {
    Keep,
    Drop,
}

async fn step(
    core: &Arc<Core>,
    client: &QbtClient,
    roll: &mut Roll,
    torrents: &[Torrent],
    here: bool,
) -> Step {
    // Find the torrents, and count a batch's episodes once its list is in
    let mut files_of: HashMap<usize, Vec<TorrentFile>> = HashMap::new();
    let mut counted_now = false;
    for (i, part) in roll.parts.iter_mut().enumerate() {
        if !part.added || part.gone || part.hash.is_empty() {
            continue;
        }
        let Some(t) = torrents
            .iter()
            .find(|t| t.hash.eq_ignore_ascii_case(&part.hash))
        else {
            if part.seen {
                part.gone = true;
            }
            continue;
        };
        if !part.seen
            && roll.sequential
            && let Err(e) = client
                .set_sequential(std::slice::from_ref(&t.hash), true)
                .await
        {
            tracing::warn!("{} not set to download in order: {e}", t.name);
        }
        part.seen = true;
        if part.single {
            continue;
        }
        let files = match client.files(&t.hash).await {
            Ok(f) => f,
            Err(e) => {
                tracing::debug!("files of {} not read: {e}", t.name);
                continue;
            }
        };
        if files.is_empty() {
            continue;
        }
        if !part.counted {
            part.units = units_of(&files);
            part.counted = true;
            counted_now = true;
        }
        files_of.insert(i, files);
    }
    if let Some(over) = roll.over
        && roll.parts.iter().all(|p| p.counted)
    {
        let total: usize = roll.parts.iter().map(|p| p.units.len()).sum();
        if total <= over as usize {
            return Step::Drop;
        }
        roll.over = None;
    }
    if roll.over.is_some() {
        return Step::Keep;
    }
    // Known now: say once how the download rolls
    let before = if counted_now {
        Window {
            last: None,
            start: usize::MAX,
            end: usize::MAX,
            total: 0,
        }
    } else {
        window(roll, &order(roll))
    };
    // Watched since the roll began
    let since = roll.created_at;
    for (i, part) in roll.parts.iter_mut().enumerate() {
        if !part.seen || part.gone {
            continue;
        }
        let Some(t) = torrents
            .iter()
            .find(|t| t.hash.eq_ignore_ascii_case(&part.hash))
        else {
            continue;
        };
        let files = files_of.get(&i);
        for u in part.units.iter_mut().filter(|u| !u.watched && !u.gone) {
            // A file nothing of which has come in cannot have been watched
            let started =
                |f: usize| files.is_none_or(|fs| fs.get(f).is_some_and(|f| f.progress > 0.0));
            let seen = match u.file {
                Some(f) => {
                    started(f)
                        && file_path(&t.save_path, &u.name)
                            .is_some_and(|p| seen_watched(core, &p.to_string_lossy(), false, since))
                }
                None => {
                    !t.content_path.is_empty() && seen_watched(core, &t.content_path, true, since)
                }
            };
            u.watched = seen;
        }
    }
    let m = moves(roll);
    let mut acted: Vec<String> = Vec::new();
    // Every batch still in the client must have had its file list read, or
    // its priorities are not known to be right
    let mut all_set = roll
        .parts
        .iter()
        .enumerate()
        .all(|(i, p)| p.single || p.gone || !p.added || files_of.contains_key(&i));
    // Batch files in and out of the window, the client's priorities kept
    // up to date with what was set
    for (part, wish) in &m.priorities {
        let Some(files) = files_of.get_mut(part) else {
            continue;
        };
        let hash = roll.parts[*part].hash.clone();
        for (priority, ids) in priority_changes(wish, files) {
            match client.set_file_priority(&hash, &ids, priority).await {
                Ok(()) => {
                    for &i in &ids {
                        if let Some(f) = files.get_mut(i) {
                            f.priority = priority;
                        }
                    }
                }
                Err(e) => {
                    all_set = false;
                    tracing::warn!("{}: file priorities not set: {e}", roll.title);
                }
            }
        }
    }
    // Held singles the window reached
    if !m.add.is_empty() {
        let items: Vec<Release> = m
            .add
            .iter()
            .map(|&p| roll.parts[p].release.clone())
            .collect();
        match super::ensure_client(core).await {
            Ok(anirss) => match anirss.download(&items, roll.name.as_deref()).await {
                Ok(added) => {
                    for &p in &m.add {
                        roll.parts[p].added = true;
                    }
                    let mut parts: Vec<&mut Part> = roll
                        .parts
                        .iter_mut()
                        .enumerate()
                        .filter(|(i, _)| m.add.contains(i))
                        .map(|(_, p)| p)
                        .collect();
                    take_hashes(&mut parts, &added.hashes);
                    acted.push(format!(
                        "added {}",
                        plural(m.add.len(), "episode", "episodes")
                    ));
                }
                Err(e) => tracing::warn!("{}: held episodes not added: {e}", roll.title),
            },
            Err(e) => tracing::warn!("{}: held episodes not added: {e}", roll.title),
        }
    }
    // Watched episodes off the disk
    let mut recheck: HashSet<usize> = HashSet::new();
    let mut deleted = 0;
    for &(p, u) in &m.delete {
        let part = &roll.parts[p];
        let Some(t) = torrents
            .iter()
            .find(|t| t.hash.eq_ignore_ascii_case(&part.hash))
        else {
            continue;
        };
        if part.single {
            match client.delete(std::slice::from_ref(&t.hash), true).await {
                Ok(()) => {
                    roll.parts[p].units[u].gone = true;
                    roll.parts[p].gone = true;
                    deleted += 1;
                }
                Err(e) => tracing::warn!("{}: watched episode not removed: {e}", t.name),
            }
            continue;
        }
        // Only a file the client has been told to skip, or it fetches it again
        let skipped = part.units[u].file.is_some_and(|f| {
            files_of
                .get(&p)
                .and_then(|fs| fs.get(f))
                .is_some_and(|f| f.priority == SKIP)
        });
        if !here || !skipped {
            continue;
        }
        let name = &part.units[u].name;
        let Some(path) = file_path(&t.save_path, name) else {
            continue;
        };
        if !path.is_file() {
            roll.parts[p].units[u].gone = true;
            continue;
        }
        match std::fs::remove_file(&path) {
            Ok(()) => {
                roll.parts[p].units[u].gone = true;
                recheck.insert(p);
                deleted += 1;
            }
            Err(e) => tracing::warn!("{} not deleted: {e}", path.display()),
        }
    }
    for p in recheck {
        let hash = roll.parts[p].hash.clone();
        if let Err(e) = client.recheck(std::slice::from_ref(&hash)).await {
            tracing::warn!("{}: recheck after deleting failed: {e}", roll.title);
        }
    }
    if deleted > 0 {
        acted.push(format!(
            "removed {} watched",
            plural(deleted, "episode", "episodes")
        ));
    }
    let after = window(roll, &order(roll));
    if after.total > 0
        && (after.start != before.start || after.end != before.end || !acted.is_empty())
    {
        let mut line = format!(
            "{}: episodes {} to {} of {}",
            roll.title,
            (after.start + 1).min(after.total),
            after.end,
            after.total
        );
        if !acted.is_empty() {
            line.push_str(&format!(", {}", acted.join(", ")));
        }
        core.bus.info(Stage::Downloads, line, EventBody::Notice);
    }
    if roll.released && m.add.is_empty() && all_set {
        return Step::Drop;
    }
    Step::Keep
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

// Entry points --------------------------------------------------------------------

/// After the torrents were added: save the rolls and keep reading the
/// client for a while, so each finds its torrent and a batch has its files
/// held back before much of them downloads.
pub async fn after_add(core: &Arc<Core>, mut rolls: Vec<Roll>, now: &[Release], hashes: &[String]) {
    if rolls.is_empty() {
        return;
    }
    // The add answered a hash per release it took: give each added part its own
    if hashes.len() == now.len() {
        for r in rolls.iter_mut() {
            for p in r.parts.iter_mut().filter(|p| p.added && p.hash.is_empty()) {
                if let Some(i) = now.iter().position(|n| n.link == p.release.link) {
                    take_hashes(&mut [p], std::slice::from_ref(&hashes[i]));
                }
            }
        }
    }
    if let Err(e) = add_rolls(core, rolls).await {
        tracing::warn!("rolling downloads not saved: {e}");
        return;
    }
    let core = core.clone();
    let handle = core.handle.clone();
    handle.spawn(async move {
        for _ in 0..60 {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let Ok(client) = super::qbt_client(&core).await else {
                continue;
            };
            match client.torrents().await {
                Ok(torrents) => tick(&core, &client, &torrents).await,
                Err(e) => tracing::debug!("rolling read failed: {e}"),
            }
            match load(&core) {
                Ok(r) if r.iter().all(settled) => break,
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
}

/// Every added part of the roll is found and counted.
fn settled(r: &Roll) -> bool {
    r.over.is_none()
        && r.parts
            .iter()
            .filter(|p| p.added)
            .all(|p| p.seen && p.counted)
}

/// An episode was watched to the end: move the windows on.
pub fn nudge(core: &Core) {
    if !core.downloads.unattended_allowed() {
        return;
    }
    if load(core).map(|r| r.is_empty()).unwrap_or(true) {
        return;
    }
    let Ok(owner) = core.owner() else { return };
    let handle = owner.handle.clone();
    handle.spawn(async move { pass_now(&owner).await });
}

/// A pass with a fresh read, then a torrent read for the page. Where the
/// read runs the pass itself, the read alone.
async fn pass_now(core: &Arc<Core>) {
    if core.downloads.unattended_allowed() {
        let _ = super::list_torrents(core);
        return;
    }
    let client = match super::qbt_client(core).await {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("rolling pass skipped: {e}");
            return;
        }
    };
    match client.torrents().await {
        Ok(torrents) => tick(core, &client, &torrents).await,
        Err(e) => tracing::debug!("rolling read failed: {e}"),
    }
    let _ = super::list_torrents(core);
}

/// `SetRoll`: how far ahead, whether watched episodes go, or stop rolling.
pub fn set(
    core: &Core,
    id: u64,
    ahead: Option<u32>,
    on: Option<bool>,
    delete_watched: Option<bool>,
) -> Result<Reply, CoreError> {
    if ahead == Some(0) {
        return Err(CoreError::invalid(
            "ahead",
            "keep at least one episode ahead",
        ));
    }
    edit(core, id, move |r| {
        if let Some(a) = ahead {
            r.ahead = a.min(MAX_AHEAD);
        }
        if let Some(d) = delete_watched {
            r.delete_watched = d;
        }
        if on == Some(false) {
            r.released = true;
        }
    })?;
    kick(core);
    Ok(Reply::Ok)
}

/// `RollMore`: fetch `count` more episodes now.
pub fn roll_more(core: &Core, id: u64, count: u32) -> Result<Reply, CoreError> {
    if count == 0 {
        return Err(CoreError::invalid("count", "nothing to fetch"));
    }
    edit(core, id, move |r| more(r, count.min(MAX_AHEAD)))?;
    kick(core);
    Ok(Reply::Ok)
}

/// A pass for a change the user asked for, in any shell.
fn kick(core: &Core) {
    if let Ok(owner) = core.owner() {
        let handle = owner.handle.clone();
        handle.spawn(async move { pass_now(&owner).await });
    }
}

/// Fills each torrent's roll, for the downloads page.
pub fn annotate(core: &Core, torrents: &mut [Torrent]) {
    let Ok(rolls) = load(core) else { return };
    if rolls.is_empty() {
        return;
    }
    for t in torrents.iter_mut() {
        if let Some(r) = rolls.iter().find(|r| {
            r.parts
                .iter()
                .any(|p| !p.hash.is_empty() && p.hash.eq_ignore_ascii_case(&t.hash))
        }) && r.over.is_none()
        {
            t.roll = Some(view(r));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(title: &str) -> Release {
        Release {
            title: title.to_string(),
            link: format!("https://example.org/{}.torrent", title.len()),
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

    fn file(name: &str, priority: i64) -> TorrentFile {
        TorrentFile {
            name: name.to_string(),
            size: 1,
            progress: 0.0,
            priority,
            episode: None,
        }
    }

    fn batch(n: usize, ahead: u32) -> Roll {
        let files: Vec<TorrentFile> = (1..=n)
            .map(|i| file(&format!("Show/Show - {i:03} [1080p].mkv"), 1))
            .collect();
        Roll {
            id: 1,
            created_at: 100,
            title: "Show".into(),
            name: None,
            ahead,
            delete_watched: true,
            over: None,
            until: None,
            released: false,
            sequential: false,
            parts: vec![Part {
                release: release("[G] Show (01-170) [Batch]"),
                hash: "aa".into(),
                added: true,
                added_at: 100,
                single: false,
                units: units_of(&files),
                counted: true,
                seen: true,
                gone: false,
            }],
        }
    }

    fn singles(titles: &[&str]) -> Vec<Release> {
        titles.iter().map(|t| release(t)).collect()
    }

    fn watch(r: &mut Roll, upto: usize) {
        for u in r.parts[0].units.iter_mut().take(upto) {
            u.watched = true;
        }
    }

    #[test]
    fn names_sort_with_numbers_read_as_numbers() {
        let mut v = vec!["Ep 10.mkv", "Ep 2.mkv", "Ep 1.mkv", "ep 3.mkv"];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(v, ["Ep 1.mkv", "Ep 2.mkv", "ep 3.mkv", "Ep 10.mkv"]);
        assert_eq!(natural_cmp("S01/E02", "S02/E01"), Ordering::Less);
        assert_eq!(natural_cmp("x 007", "x 7"), Ordering::Equal);
    }

    #[test]
    fn a_batch_counts_only_its_videos_in_name_order() {
        let files = vec![
            file("Show/Show - 10.mkv", 1),
            file("Show/Fonts/a.ttf", 1),
            file("Show/Show - 2.mkv", 1),
            file("Show/Show - 2.ass", 1),
            file("Show/Show - 1.MP4", 1),
        ];
        let u = units_of(&files);
        let names: Vec<(Option<usize>, &str)> =
            u.iter().map(|u| (u.file, u.name.as_str())).collect();
        assert_eq!(
            names,
            [
                (Some(4), "Show/Show - 1.MP4"),
                (Some(2), "Show/Show - 2.mkv"),
                (Some(0), "Show/Show - 10.mkv")
            ]
        );
    }

    #[test]
    fn the_window_starts_after_the_last_watched_and_reaches_ahead() {
        let mut r = batch(170, 12);
        let w = window(&r, &order(&r));
        assert_eq!((w.last, w.start, w.end, w.total), (None, 0, 12, 170));
        watch(&mut r, 1);
        let w = window(&r, &order(&r));
        assert_eq!((w.last, w.start, w.end), (Some(0), 1, 13));
        // Near the end the window stops at the last episode
        watch(&mut r, 165);
        let w = window(&r, &order(&r));
        assert_eq!((w.start, w.end), (165, 170));
    }

    #[test]
    fn more_reaches_past_the_window_until_watching_catches_up() {
        let mut r = batch(170, 12);
        more(&mut r, 12);
        assert_eq!(window(&r, &order(&r)).end, 24);
        // Watching inside the mark does not shrink it
        watch(&mut r, 5);
        assert_eq!(window(&r, &order(&r)).end, 24);
        // Past it, the window is ahead of the last watched again
        watch(&mut r, 14);
        assert_eq!(window(&r, &order(&r)).end, 26);
        // More again counts from the window's end
        more(&mut r, 4);
        assert_eq!(window(&r, &order(&r)).end, 30);
    }

    #[test]
    fn moves_skip_outside_the_window_and_put_the_next_first() {
        let mut r = batch(20, 3);
        watch(&mut r, 2);
        let m = moves(&r);
        let wish: HashMap<usize, i64> = m.priorities[&0].iter().copied().collect();
        let p = |n: usize| wish[&r.parts[0].units[n - 1].file.unwrap()];
        assert_eq!(p(1), SKIP);
        assert_eq!(p(2), NORMAL, "the last watched keeps its file");
        assert_eq!(p(3), FIRST);
        assert_eq!((p(4), p(5)), (NORMAL, NORMAL));
        assert_eq!(p(6), SKIP);
        assert_eq!(
            m.delete,
            vec![(0, 0)],
            "only watched episodes before the last go"
        );
        assert!(m.add.is_empty());
    }

    #[test]
    fn nothing_is_deleted_when_the_roll_keeps_watched_episodes() {
        let mut r = batch(20, 3);
        r.delete_watched = false;
        watch(&mut r, 5);
        assert!(moves(&r).delete.is_empty());
    }

    #[test]
    fn released_wants_everything_left() {
        let mut r = batch(20, 3);
        watch(&mut r, 2);
        r.released = true;
        let m = moves(&r);
        let skipped = m.priorities[&0].iter().filter(|(_, p)| *p == SKIP).count();
        assert_eq!(skipped, 1, "only the watched one before the last");
        // An episode skipped over before the last watched comes too
        r.parts[0].units[0].watched = false;
        assert!(moves(&r).priorities[&0].iter().all(|(_, p)| *p == NORMAL));
        assert!(m.priorities[&0].iter().all(|(_, p)| *p != FIRST));
    }

    #[test]
    fn only_differing_priorities_are_sent() {
        let files = vec![file("a.mkv", 1), file("b.mkv", 0), file("c.mkv", 7)];
        let wish = vec![(0, 1), (1, 1), (2, 0)];
        assert_eq!(
            priority_changes(&wish, &files),
            vec![(0, vec![2]), (1, vec![1])]
        );
    }

    #[test]
    fn a_long_pick_of_singles_adds_only_the_first_window_in_episode_order() {
        let titles: Vec<String> = (1..=40)
            .rev()
            .map(|i| format!("[Erai-raws] Black Clover - {i:03} [1080p].mkv"))
            .collect();
        let items = singles(&titles.iter().map(String::as_str).collect::<Vec<_>>());
        let s = RollingSettings::default();
        let p = plan(&items, &DownloadOptions::default(), &s, None, 5);
        assert_eq!(p.now.len(), 12);
        assert!(p.now[0].title.contains("- 001"));
        assert!(p.now[11].title.contains("- 012"));
        assert_eq!(p.rolls.len(), 1);
        let r = &p.rolls[0];
        assert_eq!(r.title, "Black Clover");
        assert_eq!(r.parts.len(), 40);
        assert_eq!(r.parts.iter().filter(|p| !p.added).count(), 28);
        // Watching the first adds the thirteenth
        let mut r = r.clone();
        r.parts[0].units[0].watched = true;
        assert_eq!(moves(&r).add, vec![12]);
    }

    #[test]
    fn a_short_pick_or_a_no_does_not_roll() {
        let items = singles(&["[G] Show - 01.mkv", "[G] Show - 02.mkv"]);
        let s = RollingSettings::default();
        let p = plan(&items, &DownloadOptions::default(), &s, None, 5);
        assert!(p.rolls.is_empty());
        assert_eq!(p.now.len(), 2);
        let on = DownloadOptions {
            rolling: Some(true),
            ..Default::default()
        };
        let mut s1 = s.clone();
        s1.ahead = 1;
        let p = plan(&items, &on, &s1, None, 5);
        assert_eq!((p.now.len(), p.rolls.len()), (1, 1));
        let titles: Vec<String> = (1..=40).map(|i| format!("[G] Show - {i:02}.mkv")).collect();
        let many = singles(&titles.iter().map(String::as_str).collect::<Vec<_>>());
        let off = DownloadOptions {
            rolling: Some(false),
            ..Default::default()
        };
        assert!(plan(&many, &off, &s, None, 5).rolls.is_empty());
        let quiet = RollingSettings {
            big: false,
            ..Default::default()
        };
        assert!(
            plan(&many, &DownloadOptions::default(), &quiet, None, 5)
                .rolls
                .is_empty()
        );
    }

    #[test]
    fn a_batch_is_added_whole_and_judged_by_its_count() {
        let items = singles(&["[Judas] Black Clover (Season 1-4) [1080p][Batch]"]);
        let p = plan(
            &items,
            &DownloadOptions::default(),
            &RollingSettings::default(),
            None,
            5,
        );
        assert_eq!(p.now.len(), 1);
        assert_eq!(p.rolls.len(), 1);
        assert_eq!(p.rolls[0].over, Some(30));
        assert!(!p.rolls[0].parts[0].counted);
    }

    #[test]
    fn binding_takes_the_matching_torrent_added_since_and_never_twice() {
        let mut r = batch(3, 2);
        r.parts[0].hash.clear();
        r.created_at = 1_000;
        let mut t1 = crate::downloads::qbt::tests_torrent();
        t1.hash = "OLD".into();
        t1.name = "[G] Show (01-170) [Batch]".into();
        t1.added_on = 10;
        let mut t2 = t1.clone();
        t2.hash = "NEW".into();
        t2.added_on = 1_001;
        let mut rolls = vec![r];
        bind(&mut rolls, &[t1, t2]);
        assert_eq!(rolls[0].parts[0].hash, "new");
    }

    #[test]
    fn a_batch_saved_under_the_show_name_still_binds() {
        let mut r = batch(3, 2);
        r.parts[0].hash.clear();
        r.parts[0].release.title = "[Test] Black Clover (01-40) [1080p][Batch]".into();
        let mut t = crate::downloads::qbt::tests_torrent();
        t.hash = "ABC".into();
        t.name = "Black Clover".into();
        t.added_on = 200;
        let mut rolls = vec![r];
        bind(&mut rolls, &[t]);
        assert_eq!(rolls[0].parts[0].hash, "abc");
    }

    #[test]
    fn an_add_that_answers_a_hash_per_release_gives_each_its_own() {
        let mut r = batch(3, 2);
        r.parts[0].hash.clear();
        let mut parts: Vec<&mut Part> = r.parts.iter_mut().collect();
        take_hashes(&mut parts, &["F".repeat(40)]);
        assert_eq!(r.parts[0].hash, "f".repeat(40));
        // A count that does not line up gives nothing
        let mut r = batch(3, 2);
        r.parts[0].hash.clear();
        let mut parts: Vec<&mut Part> = r.parts.iter_mut().collect();
        take_hashes(&mut parts, &["a".repeat(40), "b".repeat(40)]);
        assert!(r.parts[0].hash.is_empty());
    }

    #[test]
    fn a_held_single_added_late_is_given_its_own_time_to_show() {
        let titles: Vec<String> = (1..=40).map(|i| format!("[G] Show - {i:02}.mkv")).collect();
        let items = singles(&titles.iter().map(String::as_str).collect::<Vec<_>>());
        let mut r = plan(
            &items,
            &DownloadOptions::default(),
            &RollingSettings::default(),
            None,
            0,
        )
        .rolls
        .remove(0);
        for p in r.parts.iter_mut().filter(|p| p.added) {
            p.seen = true;
        }
        // Day 10: the thirteenth is handed over and not listed yet
        let day = 86_400;
        r.parts[12].added = true;
        r.parts[12].added_at = 10 * day;
        assert!(!finished(&r, 10 * day + 60));
        assert!(finished(&r, 14 * day));
    }

    #[test]
    fn a_pick_with_one_odd_title_still_rolls_its_singles() {
        let mut titles: Vec<String> = (1..=40).map(|i| format!("[G] Show - {i:02}.mkv")).collect();
        titles.push("[G] Show OVA [Batch]".into());
        let items = singles(&titles.iter().map(String::as_str).collect::<Vec<_>>());
        let p = plan(
            &items,
            &DownloadOptions::default(),
            &RollingSettings::default(),
            None,
            5,
        );
        assert_eq!(p.now.len(), 13, "the odd one and the first twelve");
        assert_eq!(p.rolls.len(), 2);
    }

    #[test]
    fn a_deleted_file_stays_skipped() {
        let mut r = batch(20, 3);
        watch(&mut r, 3);
        r.parts[0].units[0].gone = true;
        let m = moves(&r);
        let f = r.parts[0].units[0].file.unwrap();
        assert!(m.priorities[&0].contains(&(f, SKIP)));
        assert!(!m.delete.contains(&(0, 0)));
    }

    #[test]
    fn a_roll_finishes_when_its_torrents_are_all_gone() {
        let mut r = batch(3, 2);
        assert!(!finished(&r, 200));
        r.parts[0].gone = true;
        assert!(finished(&r, 200));
        // Every episode watched and the earlier ones deleted
        let mut r = batch(3, 2);
        watch(&mut r, 3);
        assert!(!finished(&r, 200));
        r.parts[0].units[0].gone = true;
        r.parts[0].units[1].gone = true;
        assert!(finished(&r, 200));
    }

    #[test]
    fn a_file_name_cannot_climb_out_of_the_save_folder() {
        assert_eq!(
            file_path("/dl", "Show/ep1.mkv"),
            Some(PathBuf::from("/dl/Show/ep1.mkv"))
        );
        assert_eq!(file_path("/dl", "../etc/passwd"), None);
        assert_eq!(file_path("/dl", "/etc/passwd"), None);
        assert_eq!(file_path("", "ep1.mkv"), None);
    }

    #[test]
    fn watched_counts_only_completions_since_the_roll_began() {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE files (series_id INTEGER, path TEXT, kind TEXT, episode_key TEXT);
             CREATE TABLE completed (series_id INTEGER, episode_key TEXT, at INTEGER);
             INSERT INTO files VALUES (1, '/lib/Show/ep1.mkv', 'episode', '1'),
                                      (1, '/lib/Show/ep2.mkv', 'episode', '2'),
                                      (1, '/lib/Single [x]/ep3.mkv', 'episode', '3');
             INSERT INTO completed VALUES (1, '1', 50), (1, '2', 150), (1, '3', 150);",
        )
        .unwrap();
        assert!(!watched_since(&c, "/lib/Show/ep1.mkv", false, 100).unwrap());
        assert!(watched_since(&c, "/lib/Show/ep2.mkv", false, 100).unwrap());
        assert!(watched_since(&c, "/lib/Single [x]", true, 100).unwrap());
        assert!(!watched_since(&c, "/lib/Single", true, 100).unwrap());
        assert!(!watched_since(&c, "/lib/Show", false, 100).unwrap());
    }

    #[test]
    fn a_show_title_drops_group_episode_and_tags() {
        assert_eq!(
            show_title("[Erai-raws] Black Clover - 001 [1080p][Multiple Subtitle]"),
            "Black Clover"
        );
        assert_eq!(show_title("Frieren (2023) [1080p]"), "Frieren");
    }

    #[test]
    fn the_view_counts_from_one() {
        let mut r = batch(170, 12);
        watch(&mut r, 3);
        let v = view(&r);
        assert_eq!((v.next, v.through, v.total, v.watched), (4, 15, 170, 3));
    }
}
