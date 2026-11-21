//! The desktop's show view (`apps/linux/qml/GetShow.qml`) without its drawing: a provider's
//! releases for one show as rows of episode cubes, one row per release group and quality
//! and one per batch, under the library's own row; the picks, the group that fills the
//! most gaps picked first; and the plan Get and Follow carry out. Moved into the core so
//! every shell draws the same grid from the same rules.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;

use anibeam_contract::*;

use super::facts;

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
    /// The episodes with a file here (`SeriesCard::episodes_here`). The cubes mark an
    /// episode watched elsewhere as watched with no file here; with this set, only what is
    /// in it counts as had. None trusts the cubes alone.
    pub here: Option<Vec<u32>>,
    /// The folder downloads go into.
    pub folder: String,
    /// An AniList entry. AniList keeps each season apart, as an entry of its own, so one
    /// whose names state no season is its first (`another_season`). False for a TVmaze
    /// show, numbered within its seasons, and for one made from a torrent's name.
    pub anilist: bool,
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

static YEAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:19|20)\d{2}\b").expect("year pattern compiles"));
static NUMBER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d").expect("number pattern compiles"));
static ROMAN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:I|II|III|IV|V|VI|VII|VIII|IX|X|XI|XII)\b").expect("roman pattern compiles")
});
static SEASON_WORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:season|part|cour|final|zoku|second|third|fourth|fifth|sequel)\b")
        .expect("season word pattern compiles")
});
/// Non-Latin season marks: 第 and 期 or 季 (kanji), 기 (Korean), ภาค and ซีซั่น (Thai), сезон.
static SEASON_SCRIPT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)[第期季기]|ภาค|ซีซั่น|сезон").expect("season script pattern compiles")
});

/// The longest run AniList keeps as one entry: a year of weekly episodes. A longer one (One
/// Piece, Naruto Shippuuden) is a run its releases cut into seasons of their own.
const ONE_SEASON: u32 = 52;

/// The names the show goes by: its queries and its title.
fn names(show: &Show) -> impl Iterator<Item = &str> {
    show.queries
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(show.title.as_str()))
}

/// The season one of the show's names states: "2nd Season", "Season 4", "S2".
fn stated_season(show: &Show) -> Option<u32> {
    names(show).find_map(facts::season_word)
}

/// Whether a name hints at a season `stated_season` cannot read: a number that is not a
/// year, a roman numeral, a word such as Part or Final, or a non-Latin season mark.
fn season_hint(name: &str) -> bool {
    let bare = YEAR_RE.replace_all(name, " ");
    NUMBER_RE.is_match(&bare)
        || ROMAN_RE.is_match(name)
        || SEASON_WORD_RE.is_match(name)
        || SEASON_SCRIPT_RE.is_match(name)
}

/// Whether the release `title`, whose facts read season `theirs`, is another season of the
/// show. Against a season the show's names state, any other is. A show whose names state
/// none is taken for its first season only when nothing says otherwise: it is an AniList
/// entry, short enough to be one season, none of its names hints at a season, and the
/// release carries every word of one of its names. A later season known by its arc's name
/// ("Yuukaku-hen") is left out of a release that says only "Season 2", which may be it.
fn another_season(show: &Show, title: &str, theirs: u32) -> bool {
    if let Some(ours) = stated_season(show) {
        return theirs != ours;
    }
    theirs != 1
        && show.anilist
        && (1..=ONE_SEASON).contains(&show.total)
        && !names(show).any(season_hint)
        && carries_a_name(show, title)
}

/// A release title's words: lower case, without the leading `[group]`, one space between
/// them and one at either end.
fn title_words(title: &str) -> String {
    let lower = title.to_lowercase();
    let body = match lower
        .trim_start()
        .strip_prefix('[')
        .and_then(|r| r.split_once(']'))
    {
        Some((_, rest)) => rest.to_string(),
        None => lower.clone(),
    };
    format!(
        " {} ",
        body.split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    )
}

/// Whether a release carries every word of one of the show's names.
fn carries_a_name(show: &Show, title: &str) -> bool {
    let t = title_words(title);
    show_words(show)
        .iter()
        .any(|ws| ws.iter().all(|w| t.contains(&format!(" {w}"))))
}

/// Whether a release names the show: at least half of one of its names' words, outside the
/// leading `[group]` (a search matches the group too: "Naruto" finds Naruto-Kun.Hu's One
/// Piece).
pub fn names_show(show: &Show, title: &str) -> bool {
    let words = show_words(show);
    if words.is_empty() {
        return true;
    }
    let t = title_words(title);
    words.iter().any(|ws| {
        let hit = ws.iter().filter(|w| t.contains(&format!(" {w}"))).count();
        hit >= ws.len().div_ceil(2)
    })
}

/// The releases that name the show and no other season of it (`another_season`, by the
/// season their facts read), with their facts; and how many were dropped.
pub fn of_show(
    show: &Show,
    items: &[Release],
    facts: &[ReleaseFacts],
) -> (Vec<Release>, Vec<ReleaseFacts>, usize) {
    let mut out_items = Vec::new();
    let mut out_facts = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let season = facts.get(i).and_then(|f| f.season);
        if names_show(show, &item.title)
            && !season.is_some_and(|s| another_season(show, &item.title, s))
        {
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
    let last_here = show
        .here
        .as_ref()
        .and_then(|h| h.iter().max().copied())
        .unwrap_or(0);
    (1..=total)
        .map(|n| {
            let mut c = codes.get(n as usize - 1).copied().unwrap_or('g');
            // Watched, but the file is not here: as far as getting it goes, it is not had.
            if matches!(c, 'w' | 'l')
                && let Some(here) = &show.here
                && !here.contains(&n)
            {
                c = if n < last_here { 'm' } else { 'g' };
            }
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
    // A batch of another season (29-38 under a 28-episode show) holds nothing here.
    order.extend(packs.into_iter().filter(|p| p.count > 0));
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
    /// The groups they come from: "Lumen", "Kumo-Subs's batch".
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
        let group = if f.group.is_empty() {
            provider
        } else {
            f.group.as_str()
        };
        let g = if f.batch {
            anibeam_contract::i18n::tr("core-releases-group-batch", &[("group", group.into())])
        } else {
            group.to_string()
        };
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
        parts.push(if a == b { a.to_string() } else { range(a, b) });
        a = n;
        b = n;
    }
    parts.push(if a == b { a.to_string() } else { range(a, b) });
    anibeam_contract::i18n::tr(
        "core-releases-episodes-phrase",
        &[
            ("count", ns.len().into()),
            ("list", parts.join(", ").into()),
        ],
    )
}

/// "3 to 5": a run of episode numbers.
fn range(a: u32, b: u32) -> String {
    anibeam_contract::i18n::tr(
        "core-releases-episode-range",
        &[("from", a.to_string().into()), ("to", b.to_string().into())],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts;

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
            here: None,
            folder: "Sousou no Frieren".into(),
            anilist: true,
        }
    }

    #[test]
    fn an_episode_watched_elsewhere_is_not_had_here() {
        let mut s = show("wwwg");
        s.total = 4;
        s.here = Some(vec![1]);
        let row = library(&s, 4, &BTreeMap::new());
        assert_eq!(row, vec!['w', 'g', 'g', 'g']);
        assert!(have(&row, 1));
        assert!(!have(&row, 2) && !have(&row, 3));
        // Without the set, the cubes alone decide, as before.
        s.here = None;
        assert!(have(&library(&s, 4, &BTreeMap::new()), 2));
    }

    #[test]
    fn a_release_must_name_the_show_outside_its_group() {
        let s = Show {
            title: "Naruto".into(),
            ..Default::default()
        };
        assert!(names_show(&s, "[Lumen] Naruto - 01 (1080p)"));
        assert!(!names_show(&s, "[Naruto-Kun.Hu] One Piece - 1100"));
    }

    #[test]
    fn the_group_that_fills_the_most_gaps_is_picked_and_the_plan_follows_it() {
        let items = vec![
            rel("[Lumen] Sousou no Frieren - 01 (1080p)", 50),
            rel("[Lumen] Sousou no Frieren - 02 (1080p)", 50),
            rel("[Lumen] Sousou no Frieren - 03 (1080p)", 50),
            rel("[Kumo-Subs] Sousou no Frieren - 03 [1080p]", 90),
            rel("[Kumo-Subs] Sousou no Frieren - 04 [1080p]", 90),
        ];
        let f: Vec<ReleaseFacts> = items.iter().map(facts::read).collect();
        let s = show("dd"); // 1 and 2 on disk
        let flight = BTreeMap::new();
        let t = total(&s, &f, &flight);
        assert_eq!(t, 4);
        let lib = library(&s, t, &flight);
        let rs = rows(&items, &f, t, "tracker");
        assert_eq!(rs.len(), 2);
        let best = best_row(&rs, &lib, t).unwrap();
        assert_eq!(rs[best].group, "Kumo-Subs", "fills 3 and 4");
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
            "tracker",
        );
        assert_eq!(p.episodes, vec![3, 4]);
        assert_eq!(p.items.len(), 2);
        assert_eq!(p.follow_query, "Sousou no Frieren Kumo-Subs 1080p");
        assert_eq!(episodes_phrase(&p.episodes), "Episodes 3 to 4");
    }

    /// An AniList entry by these names, `total` episodes long.
    fn entry(title: &str, queries: &[&str], total: u32) -> Show {
        Show {
            title: title.into(),
            queries: queries.iter().map(|q| q.to_string()).collect(),
            total,
            anilist: true,
            ..Default::default()
        }
    }

    /// Whether `of_show` keeps the release `title` for the show.
    fn keeps(show: &Show, title: &str) -> bool {
        let item = rel(title, 1);
        let f = facts::read(&item);
        of_show(show, &[item], &[f]).0.len() == 1
    }

    /// Re:Zero's first season and its fourth, by the names AniList gives them.
    fn rezero() -> (Show, Show) {
        let first = entry(
            "Re Zero",
            &[
                "Re:Zero kara Hajimeru Isekai Seikatsu",
                "Re:ZERO -Starting Life in Another World-",
                "Re Zero",
                "ReZero",
            ],
            25,
        );
        let fourth = entry(
            "Re:ZERO -Starting Life in Another World- Season 4",
            &[
                "Re:Zero kara Hajimeru Isekai Seikatsu 4th Season",
                "Re:ZERO -Starting Life in Another World- Season 4",
            ],
            19,
        );
        (first, fourth)
    }

    const FOURTH_BATCH: &str = "[Judas] Re:Zero kara Hajimeru Isekai Seikatsu (Re:Zero - Starting Life in Another World) (Season 04) [1080p][HEVC x265 10bit][Dual-Audio][Multi-Subs] (Batch)";

    #[test]
    fn a_release_of_another_season_is_not_the_shows() {
        let (first, fourth) = rezero();
        assert!(!keeps(&first, FOURTH_BATCH));
        assert!(keeps(&fourth, FOURTH_BATCH));
        // A release that names no season, or the show's own, stays
        let plain = "[Lumen] Re Zero kara Hajimeru Isekai Seikatsu - 05 (1080p)";
        assert!(keeps(&first, plain) && keeps(&fourth, plain));
        let s1 = "[Lumen] Re Zero kara Hajimeru Isekai Seikatsu - S01E05 (1080p)";
        assert!(keeps(&first, s1) && !keeps(&fourth, s1));
        assert!(!keeps(
            &first,
            "[Lumen] Re Zero kara Hajimeru Isekai Seikatsu - S02E05 (1080p)"
        ));
        // A real first season whose names state none, Kimetsu no Yaiba's
        let kny = entry(
            "Kimetsu no Yaiba",
            &["Kimetsu no Yaiba", "Demon Slayer: Kimetsu no Yaiba", "KnY"],
            26,
        );
        assert!(!keeps(
            &kny,
            "[Judas] Kimetsu no Yaiba (Season 2) [1080p][Batch]"
        ));
    }

    #[test]
    fn a_show_naming_no_season_keeps_what_it_cannot_rule_out() {
        // A later season known by its arc's name: "Season 2" may well be it
        let arc = entry(
            "Kimetsu no Yaiba: Yuukaku-hen",
            &[
                "Kimetsu no Yaiba: Yuukaku-hen",
                "Demon Slayer: Kimetsu no Yaiba Entertainment District Arc",
            ],
            11,
        );
        assert!(keeps(
            &arc,
            "[Judas] Kimetsu no Yaiba (Season 2) [1080p][Batch]"
        ));
        // A long run whose releases cut seasons of their own, its count known or not
        let shippuuden = entry("NARUTO: Shippuuden", &["NARUTO: Shippuuden"], 500);
        assert!(keeps(
            &shippuuden,
            "[Judas] Naruto Shippuuden (Season 02) [1080p][Batch]"
        ));
        let one_piece = entry("ONE PIECE", &["ONE PIECE"], 0);
        assert!(keeps(
            &one_piece,
            "[Judas] One Piece (Season 21) [1080p][Batch]"
        ));
        // Names that hint at a season this cannot read: a roman numeral, a number
        let overlord = entry("Overlord III", &["Overlord III", "Over Lord 3"], 13);
        assert!(keeps(&overlord, "[Grp] Overlord III S03E05 [1080p]"));
        let mob = entry("Mob Psycho 100", &["Mob Psycho 100"], 12);
        assert!(keeps(&mob, "Mob Psycho 100 S02E01 1080p WEB h264-GRP"));
        // A year is no hint: the 2019 first season leaves its second out
        let fruits = entry("Fruits Basket (2019)", &["Fruits Basket (2019)"], 25);
        assert!(!keeps(
            &fruits,
            "Fruits Basket (2019) S02E01 1080p WEB h264-GRP"
        ));
        // Not an AniList entry: a TVmaze show numbered within its seasons
        let tv = Show {
            anilist: false,
            ..entry("Breaking Bad", &["Breaking Bad"], 13)
        };
        assert!(keeps(&tv, "Breaking Bad S05E01 1080p WEB h264-GRP"));
    }

    #[test]
    fn another_seasons_batch_is_never_the_pick() {
        let (first, _) = rezero();
        let items = vec![
            rel(
                "[Judas] Re:Zero kara Hajimeru Isekai Seikatsu (Season 04) [1080p][Batch]",
                90,
            ),
            rel(
                "[Lumen] Re Zero kara Hajimeru Isekai Seikatsu - 01 (1080p)",
                10,
            ),
            rel(
                "[Lumen] Re Zero kara Hajimeru Isekai Seikatsu - 02 (1080p)",
                10,
            ),
        ];
        let f: Vec<ReleaseFacts> = items.iter().map(facts::read).collect();
        let (kept, kept_facts, dropped) = of_show(&first, &items, &f);
        assert_eq!(dropped, 1);
        let flight = BTreeMap::new();
        let t = total(&first, &kept_facts, &flight);
        let lib = library(&first, t, &flight);
        let rs = rows(&kept, &kept_facts, t, "tracker");
        let best = best_row(&rs, &lib, t).unwrap();
        assert_eq!(
            (rs[best].group.as_str(), rs[best].batch, rs[best].count),
            ("Lumen", false, 2)
        );
    }

    #[test]
    fn a_batch_lights_its_range_and_comes_whole() {
        let items = vec![rel("[Judas] Sousou no Frieren (01-04) [1080p][Batch]", 20)];
        let f: Vec<ReleaseFacts> = items.iter().map(facts::read).collect();
        let s = show("");
        let t = total(&s, &f, &BTreeMap::new());
        let lib = library(&s, t, &BTreeMap::new());
        let rs = rows(&items, &f, t, "tracker");
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
