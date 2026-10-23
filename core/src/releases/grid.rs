//! The desktop's show view (`apps/linux/qml/GetShow.qml`) without its drawing: a provider's
//! releases for one show as rows of episode cubes, one row per release group and quality
//! and one per batch, under the library's own row; the picks, the group that fills the
//! most gaps picked first; and the plan Get and Follow carry out. Moved into the core so
//! every shell draws the same grid from the same rules.

use std::collections::BTreeMap;

use crate::contract::*;

/// The longest grid drawn, as the desktop's.
const MAX_EPISODES: u32 = 2000;

/// What the grid needs to know about the show.
#[derive(Clone, Debug, Default)]
pub struct Show {
    pub title: String,
    /// The names it goes by, searched in order until one answers.
    pub queries: Vec<String>,
    /// Its episode count, 0 when not known.
    pub total: u32,
    pub airing: bool,
    /// The library's cube row (`library::cards::cubes`).
    pub cubes: String,
    /// The folder downloads go into.
    pub folder: String,
}

/// One row of the grid.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub key: String,
    pub group: String,
    pub resolution: String,
    pub batch: bool,
    /// Episode number to the index of the release that carries it.
    pub eps: BTreeMap<u32, usize>,
    pub count: usize,
    pub seeds: i64,
    pub size: u64,
    /// For a batch, the episodes it holds as runs: `1-12, 14`.
    pub label: String,
}

fn stop_word(w: &str) -> bool {
    matches!(w, "the" | "and" | "season" | "part" | "movie")
}

/// One word list per name the show goes by.
fn show_words(show: &Show) -> Vec<Vec<String>> {
    let names: Vec<&String> = if show.queries.is_empty() {
        vec![&show.title]
    } else {
        show.queries.iter().collect()
    };
    names
        .into_iter()
        .map(|q| {
            q.to_lowercase()
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|w| w.len() >= 3 && !stop_word(w))
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .filter(|ws| !ws.is_empty())
        .collect()
}

/// Whether a release names the show: at least half of one of its names' words, outside the
/// leading `[group]` (a search matches the group too: "Naruto" finds Naruto-Kun.Hu's One
/// Piece).
pub fn names_show(show: &Show, title: &str) -> bool {
    let words = show_words(show);
    if words.is_empty() {
        return true;
    }
    let lower = title.to_lowercase();
    let body = match lower
        .trim_start()
        .strip_prefix('[')
        .and_then(|r| r.split_once(']'))
    {
        Some((_, rest)) => rest.to_string(),
        None => lower.clone(),
    };
    let t = format!(
        " {} ",
        body.split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    );
    words.iter().any(|ws| {
        let hit = ws.iter().filter(|w| t.contains(&format!(" {w}"))).count();
        hit >= ws.len().div_ceil(2)
    })
}

/// The releases that name the show, with their facts; and how many were dropped.
pub fn of_show(
    show: &Show,
    items: &[Release],
    facts: &[ReleaseFacts],
) -> (Vec<Release>, Vec<ReleaseFacts>, usize) {
    let mut out_items = Vec::new();
    let mut out_facts = Vec::new();
    for (i, item) in items.iter().enumerate() {
        if names_show(show, &item.title) {
            out_items.push(item.clone());
            if let Some(f) = facts.get(i) {
                out_facts.push(f.clone());
            }
        }
    }
    let dropped = items.len() - out_items.len();
    (out_items, out_facts, dropped)
}

fn whole(n: f64) -> Option<u32> {
    (n >= 0.0 && n.fract() == 0.0).then_some(n as u32)
}

/// Episode number to how far its torrent is, for every episode a torrent of this show
/// carries and has not finished; a batch is read file by file.
pub fn in_flight(torrents: &[Torrent]) -> BTreeMap<u32, f64> {
    let mut out = BTreeMap::new();
    for t in torrents.iter().filter(|t| t.progress < 1.0) {
        if !t.files.is_empty() {
            for f in t.files.iter().filter(|f| f.priority != 0) {
                if let Some(n) = f.episode.and_then(whole) {
                    out.insert(n, f.progress);
                }
            }
        } else if let Some(n) = t.episode.and_then(whole) {
            out.insert(n, t.progress);
        }
    }
    out
}

/// How many episodes the grid runs to.
pub fn total(show: &Show, facts: &[ReleaseFacts], flight: &BTreeMap<u32, f64>) -> u32 {
    let lib = show.cubes.chars().count() as u32;
    let mut t = if show.total > 0 { show.total } else { lib };
    // A finished show's own count is the count: a stray number in a release name must not
    // stretch the grid past its last episode.
    if show.total > 0 && !show.airing {
        let fixed = flight.keys().copied().fold(show.total, u32::max);
        return fixed.min(MAX_EPISODES);
    }
    for f in facts {
        if !f.batch
            && let Some(n) = f.episode.and_then(whole)
        {
            t = t.max(n);
        }
        if f.batch
            && let Some(last) = f.last_episode
        {
            t = t.max(last);
        }
    }
    t = flight.keys().copied().fold(t, u32::max);
    t.min(MAX_EPISODES)
}

/// The library's row: its cube codes, with episodes a torrent carries shown on the way.
pub fn library(show: &Show, total: u32, flight: &BTreeMap<u32, f64>) -> Vec<char> {
    let codes: Vec<char> = show.cubes.chars().collect();
    (1..=total)
        .map(|n| {
            let mut c = codes.get(n as usize - 1).copied().unwrap_or('g');
            if let Some(p) = flight.get(&n)
                && c != 'w'
                && c != 'l'
            {
                c = if *p >= 1.0 { 'd' } else { 't' };
            }
            c
        })
        .collect()
}

/// On disk, or on its way.
pub fn have(library: &[char], n: u32) -> bool {
    matches!(library.get(n as usize - 1), Some('d' | 'l' | 'w' | 't'))
}

fn seeds(item: &Release) -> i64 {
    item.seeders.unwrap_or(-1)
}

/// Episode numbers as short runs: `1-12, 14, 20-24`.
pub fn runs(list: &[u32]) -> String {
    let mut out = Vec::new();
    let mut i = 0;
    while i < list.len() {
        let mut j = i;
        while j + 1 < list.len() && list[j + 1] == list[j] + 1 {
            j += 1;
        }
        out.push(if j > i {
            format!("{}-{}", list[i], list[j])
        } else {
            list[i].to_string()
        });
        i = j + 1;
    }
    out.join(", ")
}

/// A provider's releases as rows: group and quality with the best-seeded release per
/// episode, and each batch a row of its own that lights every episode it names (all of
/// them when it names none). Most episodes first, then episode rows before batches, then
/// the better seeded.
pub fn rows(items: &[Release], facts: &[ReleaseFacts], total: u32, provider: &str) -> Vec<Row> {
    let mut order: Vec<Row> = Vec::new();
    let mut packs: Vec<Row> = Vec::new();
    for (i, (item, f)) in items.iter().zip(facts).enumerate() {
        let group = if f.group.is_empty() {
            provider.to_string()
        } else {
            f.group.clone()
        };
        if f.batch {
            let list: Vec<u32> = match f.first_episode {
                Some(first) => (first..=f.last_episode.unwrap_or(total).min(total)).collect(),
                None => (1..=total).collect(),
            };
            let mut eps = BTreeMap::new();
            for n in &list {
                eps.insert(*n, i);
            }
            packs.push(Row {
                key: format!("batch|{i}"),
                group,
                resolution: f.resolution.clone(),
                batch: true,
                count: eps.len(),
                eps,
                seeds: seeds(item).max(0),
                size: f.size_bytes.unwrap_or(0),
                label: runs(&list),
            });
            continue;
        }
        let Some(n) = f.episode.and_then(whole) else {
            continue;
        };
        let key = format!("{group}|{}", f.resolution);
        let row = match order.iter_mut().position(|r| r.key == key) {
            Some(at) => &mut order[at],
            None => {
                order.push(Row {
                    key,
                    group,
                    resolution: f.resolution.clone(),
                    batch: false,
                    eps: BTreeMap::new(),
                    count: 0,
                    seeds: 0,
                    size: 0,
                    label: String::new(),
                });
                order.last_mut().unwrap()
            }
        };
        match row.eps.get(&n) {
            Some(&prev) if seeds(item) <= seeds(&items[prev]) => {}
            _ => {
                row.eps.insert(n, i);
            }
        }
    }
    for row in &mut order {
        row.count = row.eps.len();
        row.seeds = row.eps.values().map(|&i| seeds(&items[i]).max(0)).sum();
        row.size = row
            .eps
            .values()
            .map(|&i| facts[i].size_bytes.unwrap_or(0))
            .sum();
    }
    order.extend(packs);
    // A stable sort, as the desktop's JavaScript sort is.
    order.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then((a.batch as u8).cmp(&(b.batch as u8)))
            .then(b.seeds.cmp(&a.seeds))
    });
    order
}

/// The episodes a row would fill that the library lacks.
pub fn wanted(row: &Row, library: &[char], total: u32) -> Vec<u32> {
    (1..=total)
        .filter(|n| !have(library, *n) && row.eps.contains_key(n))
        .collect()
}

/// The row that fills the most gaps, the larger on a tie.
pub fn best_row(rows: &[Row], library: &[char], total: u32) -> Option<usize> {
    let mut best = None;
    let mut score = -1i64;
    for (i, r) in rows.iter().enumerate() {
        let s = wanted(r, library, total).len() as i64 * 1000 + r.count as i64;
        if s > score {
            score = s;
            best = Some(i);
        }
    }
    best
}

/// The picks: episode number to the release index, and the row they came from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picks {
    pub map: BTreeMap<u32, usize>,
    pub chosen: String,
}

/// Picks a row: its releases for every episode the library lacks. `explicit` is a tap on
/// the row: with nothing missing it takes everything the row covers, to replace it. The
/// automatic pick never does.
pub fn pick_row(row: &Row, library: &[char], total: u32, explicit: bool) -> Picks {
    let mut w = wanted(row, library, total);
    if w.is_empty() && explicit {
        w = (1..=total).filter(|n| row.eps.contains_key(n)).collect();
    }
    Picks {
        map: w.iter().map(|n| (*n, row.eps[n])).collect(),
        chosen: row.key.clone(),
    }
}

/// A tap on one cube. A batch comes whole, so a tap on a gap takes or drops every gap it
/// fills; a tap on an episode the library has picks just that one, to replace it.
pub fn toggle_cube(picks: &mut Picks, row: &Row, n: u32, library: &[char], total: u32) {
    let Some(&item) = row.eps.get(&n) else { return };
    if row.batch && !have(library, n) {
        let on = picks.map.get(&n) == Some(&item);
        for w in wanted(row, library, total) {
            if on {
                if picks.map.get(&w) == Some(&row.eps[&w]) {
                    picks.map.remove(&w);
                }
            } else {
                picks.map.insert(w, row.eps[&w]);
            }
        }
    } else if picks.map.get(&n) == Some(&item) {
        picks.map.remove(&n);
    } else {
        picks.map.insert(n, item);
    }
    picks.chosen = row.key.clone();
}

/// A row's cubes: `n` none, `p` picked, `o` the library has it, `t` on its way, `a` there
/// to take.
pub fn codes_of(row: &Row, picks: &Picks, library: &[char], total: u32) -> Vec<char> {
    (1..=total)
        .map(|n| match row.eps.get(&n) {
            None => 'n',
            Some(i) if picks.map.get(&n) == Some(i) => 'p',
            Some(_) if have(library, n) => {
                if library.get(n as usize - 1) == Some(&'t') {
                    't'
                } else {
                    'o'
                }
            }
            Some(_) => 'a',
        })
        .collect()
}

/// What Get and Follow will do.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    /// The episodes Get fetches.
    pub episodes: Vec<u32>,
    /// The torrents behind them: a batch once however many episodes it covers.
    pub items: Vec<Release>,
    pub bytes: u64,
    /// The groups they come from: "SubsPlease", "Erai-raws's batch".
    pub groups: Vec<String>,
    /// Follow's search and the releases it is made from; empty when there is no row to
    /// follow.
    pub follow_query: String,
    pub follow_items: Vec<Release>,
    pub name: String,
}

/// The feed a pick follows: a batch row follows its group's episode row, when there is one.
fn follow_of<'a>(rows: &'a [Row], chosen: &str) -> Option<&'a Row> {
    let row = rows.iter().find(|r| r.key == chosen)?;
    if !row.batch {
        return Some(row);
    }
    let key = format!("{}|{}", row.group, row.resolution);
    rows.iter().find(|r| !r.batch && r.key == key)
}

/// The plan for the picks: with `replace` off, episodes the library has are left out.
#[allow(clippy::too_many_arguments)]
pub fn plan(
    show: &Show,
    query: &str,
    items: &[Release],
    facts: &[ReleaseFacts],
    rows: &[Row],
    picks: &Picks,
    library: &[char],
    replace: bool,
    provider: &str,
) -> Plan {
    let list: Vec<(u32, usize)> = picks
        .map
        .iter()
        .map(|(n, i)| (*n, *i))
        .filter(|(n, _)| replace || !have(library, *n))
        .collect();
    let mut seen = Vec::new();
    for (_, i) in &list {
        if !seen.contains(i) {
            seen.push(*i);
        }
    }
    let mut groups = Vec::new();
    for i in &seen {
        let f = &facts[*i];
        let g = format!(
            "{}{}",
            if f.group.is_empty() {
                provider
            } else {
                &f.group
            },
            if f.batch { "'s batch" } else { "" }
        );
        if !groups.contains(&g) {
            groups.push(g);
        }
    }
    let follow = follow_of(rows, &picks.chosen);
    Plan {
        episodes: list.iter().map(|(n, _)| *n).collect(),
        bytes: seen.iter().map(|i| facts[*i].size_bytes.unwrap_or(0)).sum(),
        items: seen.iter().map(|i| items[*i].clone()).collect(),
        groups,
        follow_query: follow
            .map(|r| {
                format!(
                    "{query} {}{}",
                    r.group,
                    if r.resolution.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", r.resolution)
                    }
                )
            })
            .map(|q| q.trim().to_string())
            .unwrap_or_default(),
        follow_items: follow
            .map(|r| r.eps.values().map(|i| items[*i].clone()).collect())
            .unwrap_or_default(),
        name: if show.folder.is_empty() {
            show.title.clone()
        } else {
            show.folder.clone()
        },
    }
}

/// "Episodes 1 to 4, 7" for a plan.
pub fn episodes_phrase(ns: &[u32]) -> String {
    if ns.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    let (mut a, mut b) = (ns[0], ns[0]);
    for &n in &ns[1..] {
        if n == b + 1 {
            b = n;
            continue;
        }
        parts.push(if a == b {
            a.to_string()
        } else {
            format!("{a} to {b}")
        });
        a = n;
        b = n;
    }
    parts.push(if a == b {
        a.to_string()
    } else {
        format!("{a} to {b}")
    });
    format!(
        "{} {}",
        if ns.len() == 1 { "Episode" } else { "Episodes" },
        parts.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloads::facts;

    fn rel(title: &str, seeders: i64) -> Release {
        Release {
            title: title.into(),
            link: format!("magnet:?xt=urn:btih:{title}"),
            seeders: Some(seeders),
            leechers: None,
            downloads: None,
            size: "1.4 GiB".into(),
            category: String::new(),
            page: String::new(),
            source: String::new(),
            resolution: String::new(),
            infomkv: String::new(),
            linktype: String::new(),
        }
    }

    fn show(cubes: &str) -> Show {
        Show {
            title: "Sousou no Frieren".into(),
            queries: vec!["Sousou no Frieren".into()],
            total: 4,
            airing: false,
            cubes: cubes.into(),
            folder: "Sousou no Frieren".into(),
        }
    }

    #[test]
    fn a_release_must_name_the_show_outside_its_group() {
        let s = Show {
            title: "Naruto".into(),
            ..Default::default()
        };
        assert!(names_show(&s, "[SubsPlease] Naruto - 01 (1080p)"));
        assert!(!names_show(&s, "[Naruto-Kun.Hu] One Piece - 1100"));
    }

    #[test]
    fn the_group_that_fills_the_most_gaps_is_picked_and_the_plan_follows_it() {
        let items = vec![
            rel("[SubsPlease] Sousou no Frieren - 01 (1080p)", 50),
            rel("[SubsPlease] Sousou no Frieren - 02 (1080p)", 50),
            rel("[SubsPlease] Sousou no Frieren - 03 (1080p)", 50),
            rel("[Erai-raws] Sousou no Frieren - 03 [1080p]", 90),
            rel("[Erai-raws] Sousou no Frieren - 04 [1080p]", 90),
        ];
        let f: Vec<ReleaseFacts> = items.iter().map(facts::read).collect();
        let s = show("dd"); // 1 and 2 on disk
        let flight = BTreeMap::new();
        let t = total(&s, &f, &flight);
        assert_eq!(t, 4);
        let lib = library(&s, t, &flight);
        let rs = rows(&items, &f, t, "nyaa");
        assert_eq!(rs.len(), 2);
        let best = best_row(&rs, &lib, t).unwrap();
        assert_eq!(rs[best].group, "Erai-raws", "fills 3 and 4");
        let picks = pick_row(&rs[best], &lib, t, false);
        assert_eq!(picks.map.keys().copied().collect::<Vec<_>>(), vec![3, 4]);
        let codes: String = codes_of(&rs[best], &picks, &lib, t).into_iter().collect();
        assert_eq!(codes, "nnpp");
        let p = plan(
            &s,
            "Sousou no Frieren",
            &items,
            &f,
            &rs,
            &picks,
            &lib,
            false,
            "nyaa",
        );
        assert_eq!(p.episodes, vec![3, 4]);
        assert_eq!(p.items.len(), 2);
        assert_eq!(p.follow_query, "Sousou no Frieren Erai-raws 1080p");
        assert_eq!(episodes_phrase(&p.episodes), "Episodes 3 to 4");
    }

    #[test]
    fn a_batch_lights_its_range_and_comes_whole() {
        let items = vec![rel("[Judas] Sousou no Frieren (01-04) [1080p][Batch]", 20)];
        let f: Vec<ReleaseFacts> = items.iter().map(facts::read).collect();
        let s = show("");
        let t = total(&s, &f, &BTreeMap::new());
        let lib = library(&s, t, &BTreeMap::new());
        let rs = rows(&items, &f, t, "nyaa");
        assert!(rs[0].batch);
        assert_eq!(rs[0].label, "1-4");
        let mut picks = Picks::default();
        toggle_cube(&mut picks, &rs[0], 2, &lib, t);
        assert_eq!(picks.map.len(), 4);
        toggle_cube(&mut picks, &rs[0], 3, &lib, t);
        assert!(picks.map.is_empty());
        assert_eq!(runs(&[1, 2, 3, 5, 7, 8]), "1-3, 5, 7-8");
    }
}
