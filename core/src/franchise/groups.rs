//! The library's franchises: which library series belong together on the
//! library page, and what the box around them is called.
//!
//! A group is a connected component of the library's series over the
//! `relations` table, walking only the edges `closure::TRAVERSABLE` names,
//! so a CHARACTER or OTHER edge never joins two franchises (a cameo would
//! otherwise glue every franchise it touches into one box). The walk runs
//! through any node the table holds, owned or not, so a first and a third
//! season join through a second the library lacks. Edges are taken in both
//! directions: a node the crawl has not fetched yet still joins through the
//! edge its neighbour stored. Nothing here fetches anything; a series with
//! no stored edges simply stands alone until the crawl fills them in.
//!
//! Read once per snapshot: one query for the library, one for the edges.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;
use rusqlite::{Connection, params_from_iter};

use super::{as_u64, closure};
use crate::contract::{CoreError, FranchiseTag, TitleLanguage};
use crate::library::titles;

/// One library series as the grouping sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub series: u64,
    pub anilist_id: u64,
    /// The card's own title, in the user's title language.
    pub title: String,
    /// AniList's start date ("2017-01-06", or as much of it as is known),
    /// else the year. Compared as text, which orders both forms correctly;
    /// None sorts last.
    pub released: Option<String>,
}

/// Every library series in a component of two or more, by series id. A
/// series that relates to no other library series is not in the map.
pub fn load(
    conn: &Connection,
    lang: TitleLanguage,
) -> Result<HashMap<u64, FranchiseTag>, CoreError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.anilist_id, s.folder_name, m.title_romaji, m.title_english, m.start_date, m.year
         FROM series s
         LEFT JOIN anilist_media m ON m.id = s.anilist_id
         WHERE s.anilist_id IS NOT NULL AND s.missing_since IS NULL
         ORDER BY s.id",
    )?;
    let members: Vec<Member> = stmt
        .query_map([], |r| {
            let folder: String = r.get(2)?;
            let romaji: Option<String> = r.get(3)?;
            let english: Option<String> = r.get(4)?;
            let start: Option<String> = r.get(5)?;
            let year: Option<i64> = r.get(6)?;
            Ok(Member {
                series: as_u64(r.get(0)?),
                anilist_id: as_u64(r.get(1)?),
                title: titles::resolve(lang, romaji.as_deref(), english.as_deref(), &folder),
                released: start
                    .filter(|s| !s.trim().is_empty())
                    .or_else(|| year.map(|y| y.to_string())),
            })
        })?
        .collect::<Result<_, _>>()?;
    if members.len() < 2 {
        return Ok(HashMap::new());
    }

    let list = vec!["?"; closure::TRAVERSABLE.len()].join(", ");
    let mut stmt = conn.prepare(&format!(
        "SELECT from_id, to_id FROM relations WHERE relation IN ({list})"
    ))?;
    let edges: Vec<(u64, u64)> = stmt
        .query_map(params_from_iter(closure::TRAVERSABLE.iter()), |r| {
            Ok((as_u64(r.get(0)?), as_u64(r.get(1)?)))
        })?
        .collect::<Result<_, _>>()?;
    Ok(components(&edges, &members))
}

/// The pure half: union the edges, bucket the members by component, and
/// tag every bucket of two or more. `edges` must already be the traversable
/// ones.
pub fn components(edges: &[(u64, u64)], members: &[Member]) -> HashMap<u64, FranchiseTag> {
    let mut parent: HashMap<u64, u64> = HashMap::new();
    fn find(parent: &mut HashMap<u64, u64>, x: u64) -> u64 {
        let mut root = x;
        while let Some(&p) = parent.get(&root) {
            if p == root {
                break;
            }
            root = p;
        }
        // Path compression, so a long chain costs its length once.
        let mut at = x;
        while at != root {
            let next = parent[&at];
            parent.insert(at, root);
            at = next;
        }
        root
    }
    for &(a, b) in edges {
        parent.entry(a).or_insert(a);
        parent.entry(b).or_insert(b);
        let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
        if ra != rb {
            parent.insert(ra.max(rb), ra.min(rb));
        }
    }

    let mut buckets: HashMap<u64, Vec<&Member>> = HashMap::new();
    for m in members {
        let root = if parent.contains_key(&m.anilist_id) {
            find(&mut parent, m.anilist_id)
        } else {
            m.anilist_id
        };
        buckets.entry(root).or_default().push(m);
    }

    let mut tags = HashMap::new();
    for mut group in buckets.into_values() {
        if group.len() < 2 {
            continue;
        }
        group.sort_by(|a, b| {
            match (&a.released, &b.released) {
                (Some(x), Some(y)) => x.cmp(y),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
            .then(a.anilist_id.cmp(&b.anilist_id))
            .then(a.series.cmp(&b.series))
        });
        let id = group.iter().map(|m| m.anilist_id).min().unwrap_or(0);
        let titles: Vec<&str> = group.iter().map(|m| m.title.as_str()).collect();
        let tag = FranchiseTag {
            id: id.to_string(),
            title: group_title(&titles),
            size: group.len() as u32,
        };
        for m in group {
            tags.insert(m.series, tag.clone());
        }
    }
    tags
}

// ---------------------------------------------------------------------------
// The group's name
// ---------------------------------------------------------------------------

/// `titles` earliest-released first. The members' shared stem when one
/// reads as a whole name, else the earliest member's stem.
pub fn group_title(titles: &[&str]) -> String {
    let stems: Vec<String> = titles.iter().map(|t| strip_season(t)).collect();
    if let Some(shared) = shared_stem(&stems) {
        return shared;
    }
    stems.into_iter().next().unwrap_or_default()
}

/// The longest run of whole words every stem starts with, when it is a name
/// in its own right: some member's stem is exactly it, or every longer stem
/// breaks there at a separator ("Link Click" from "Link Click" and "Link
/// Click: Bridon Arc"). A run that stops mid-name ("Karakai Jouzu no") is
/// no name, and neither is a lone article.
fn shared_stem(stems: &[String]) -> Option<String> {
    if stems.len() < 2 {
        return None;
    }
    let words: Vec<Vec<&str>> = stems
        .iter()
        .map(|s| s.split_whitespace().collect())
        .collect();
    let key = |w: &str| w.trim_end_matches(SEPARATORS).to_lowercase();
    let mut n = 0;
    'outer: loop {
        let Some(first) = words[0].get(n) else { break };
        for w in &words[1..] {
            match w.get(n) {
                Some(x) if key(x) == key(first) && !key(x).is_empty() => {}
                _ => break 'outer,
            }
        }
        n += 1;
    }
    if n == 0 {
        return None;
    }
    let stem = trim_tail(&words[0][..n].join(" ")).to_string();
    let letters = stem.chars().filter(|c| c.is_alphanumeric()).count();
    if letters < 4 || (n == 1 && ["the", "a", "an"].contains(&stem.to_lowercase().as_str())) {
        return None;
    }
    let whole = words.iter().any(|w| w.len() == n);
    let breaks = words.iter().all(|w| {
        w.len() == n
            || w[n - 1].ends_with(SEPARATORS)
            || w[n].starts_with(['-', '\u{2013}', '~', '(', ':'])
    });
    (whole || breaks).then_some(stem)
}

const SEPARATORS: &[char] = &[':', '-', '\u{2013}', '~', ',', ' '];

/// A title's tail cleaned of what a strip leaves behind: spaces, a colon,
/// a comma, a tilde, or a dash standing on its own. A dash closing a
/// bracket of dashes ("Re:ZERO -Starting Life in Another World-") stays.
fn trim_tail(s: &str) -> &str {
    let mut s = s.trim_end();
    loop {
        let next = if let Some(t) = s.strip_suffix([':', ',', '~']) {
            t
        } else if let Some(t) = s.strip_suffix(" -").or_else(|| s.strip_suffix(" \u{2013}")) {
            t
        } else {
            return s;
        };
        s = next.trim_end();
    }
}

/// The suffixes that say which part of a franchise a title is, rather than
/// which franchise. Each one strips to the end of the title, from the
/// separator before it: a colon or tilde, a dash with spaces round it, or
/// a plain space (so the dash closing "-Starting Life in Another World-"
/// is no separator).
static SUFFIXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        // "2nd Season", "4th Season Part 1 & 2", "4th Season: Second Year".
        r"(?i)(?:\s*[:~,]\s*|\s+[-\x{2013}]\s+|\s+)\d+(?:st|nd|rd|th)\s+season\b.*$",
        // "Season 2", "Season 3 Part 2".
        r"(?i)(?:\s*[:~,]\s*|\s+[-\x{2013}]\s+|\s+)season\s+\d+\b.*$",
        r"(?i)(?:\s*[:~,]\s*|\s+[-\x{2013}]\s+|\s+)(?:the\s+)?final\s+season\b.*$",
        // "Movie", ": The Movie", "the Movie 2".
        r"(?i)(?:\s*[:~,]\s*|\s+[-\x{2013}]\s+|\s+)(?:the\s+)?movie(?:\s+\d+)?$",
        r"(?i)(?:\s*[:~,]\s*|\s+[-\x{2013}]\s+|\s+)(?:specials?|ova|oad)$",
        r"(?i)(?:\s*[:~,]\s*|\s+[-\x{2013}]\s+|\s+)(?:part|cour)\s+\d+(?:\s*(?:&|and)\s*\d+)?$",
        // A remake's year: "Ranma 1/2 (2024)".
        r"\s*\((?:19|20)\d{2}\)$",
    ]
    .iter()
    .map(|p| Regex::new(p).unwrap())
    .collect()
});

/// A sequel's roman numeral, at the end or before a subtitle's colon:
/// "Youjo Senki II", "Mushoku Tensei III: Isekai Ittara Honki Dasu". V and
/// X are left alone: alone they are as often a letter as a number.
static NUMERAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+(?:II|III|IV|VI|VII|VIII|IX)(\s*:.*)?$").unwrap());

/// A sequel's bare number at the end: "Karakai Jouzu no Takagi-san 2".
/// Only 2 to 9, and never a number the title itself is about ("Kaiju No.
/// 8", "Mob Psycho 100").
static NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.*\S)\s+([2-9])$").unwrap());

/// The title with its season marker gone, where the marker is obvious;
/// otherwise the title as it is.
pub fn strip_season(title: &str) -> String {
    let mut current = trim_tail(title.trim()).to_string();
    loop {
        let mut next = current.clone();
        for re in SUFFIXES.iter() {
            next = re.replace(&next, "").into_owned();
        }
        next = NUMERAL.replace(&next, "$1").into_owned();
        if let Some(c) = NUMBER.captures(&next) {
            let head = c.get(1).map_or("", |m| m.as_str());
            let last = head.split_whitespace().last().unwrap_or("").to_lowercase();
            if !matches!(last.as_str(), "no." | "no" | "#" | "vol." | "vol" | "part") {
                next = head.to_string();
            }
        }
        let next = trim_tail(&next).to_string();
        // Never strip a title down to nothing: what is left must still
        // read as a name.
        if next == current || next.chars().filter(|c| c.is_alphanumeric()).count() < 2 {
            return current;
        }
        current = next;
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use rusqlite::params;

    fn member(series: u64, anilist_id: u64, title: &str, released: &str) -> Member {
        Member {
            series,
            anilist_id,
            title: title.into(),
            released: Some(released.into()),
        }
    }

    fn library(c: &Connection, series: u64, anilist_id: u64, title: &str, start: &str) {
        c.execute(
            "INSERT OR REPLACE INTO anilist_media (id, media_type, title_romaji, start_date) VALUES (?1, 'ANIME', ?2, ?3)",
            params![anilist_id as i64, title, start],
        )
        .unwrap();
        c.execute(
            "INSERT OR IGNORE INTO sources (id, path, available, added_at) VALUES (1, '/lib', 1, 0)",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO series (id, source_id, kind, path, folder_name, added_at, anilist_id) VALUES (?1, 1, 'show', ?2, ?3, 0, ?4)",
            params![series as i64, format!("/lib/{title}"), title, anilist_id as i64],
        )
        .unwrap();
    }

    /// The four shapes the library page has to get right, off the tables.
    #[test]
    fn components_follow_the_walked_relations_and_nothing_else() {
        let c = conn();
        // A TV series and its film, joined by a SEQUEL edge.
        library(&c, 1, 21613, "Youjo Senki", "2017-01-06");
        library(&c, 2, 100878, "Youjo Senki Movie", "2019-02-08");
        edge(&c, 21613, 100878, "SEQUEL");
        // A first and a third season, joined through a second the library
        // lacks (the edges stored on the missing node only).
        library(&c, 3, 10, "Show", "2010-01-01");
        library(&c, 4, 30, "Show 3", "2014-01-01");
        media(&c, 20, true);
        edge(&c, 20, 10, "PREQUEL");
        edge(&c, 20, 30, "SEQUEL");
        // A crossover joined to both franchises by CHARACTER and OTHER only.
        library(&c, 5, 500, "Isekai Quartet", "2019-04-09");
        edge(&c, 500, 21613, "CHARACTER");
        edge(&c, 500, 10, "OTHER");
        edge(&c, 30, 500, "CHARACTER");
        // A single.
        library(&c, 6, 900, "Koe no Katachi", "2016-09-17");

        let groups = load(&c, TitleLanguage::Romaji).unwrap();
        let tanya = &groups[&1];
        assert_eq!(
            groups[&2], *tanya,
            "the TV series and its film are one group"
        );
        assert_eq!(tanya.size, 2);
        assert_eq!(tanya.id, "21613");
        assert_eq!(tanya.title, "Youjo Senki");

        let show = &groups[&3];
        assert_eq!(
            groups[&4], *show,
            "joined through the season the library lacks"
        );
        assert_eq!(show.title, "Show");
        assert_ne!(show.id, tanya.id);

        assert!(
            !groups.contains_key(&5),
            "a CHARACTER or OTHER edge joins nothing: the crossover stands alone"
        );
        assert!(!groups.contains_key(&6), "a single has no group");
    }

    #[test]
    fn a_missing_series_is_no_member_and_an_unmatched_one_none_either() {
        let c = conn();
        library(&c, 1, 1, "A", "2001");
        library(&c, 2, 2, "A 2", "2002");
        edge(&c, 1, 2, "SEQUEL");
        c.execute("UPDATE series SET missing_since = 5 WHERE id = 2", [])
            .unwrap();
        c.execute(
            "INSERT INTO series (id, source_id, kind, path, folder_name, added_at) VALUES (3, 1, 'show', '/lib/x', 'x', 0)",
            [],
        )
        .unwrap();
        assert!(load(&c, TitleLanguage::Romaji).unwrap().is_empty());
    }

    #[test]
    fn the_group_id_is_the_smallest_member_and_the_title_the_earliest() {
        let members = [
            member(1, 135865, "Youjo Senki II", "2026-07-08"),
            member(2, 21613, "Youjo Senki", "2017-01-06"),
        ];
        let tags = components(&[(21613, 100878), (100878, 135865)], &members);
        assert_eq!(tags[&1].id, "21613");
        assert_eq!(tags[&1].title, "Youjo Senki");
        assert_eq!(tags[&1].size, 2);
    }

    /// Real titles off the owner's library: the stripping, one title at a
    /// time.
    #[test]
    fn season_markers_strip_where_they_are_obvious() {
        let cases = [
            ("Youjo Senki II", "Youjo Senki"),
            ("Saga of Tanya the Evil Season 2", "Saga of Tanya the Evil"),
            (
                "Karakai Jouzu no Takagi-san 2",
                "Karakai Jouzu no Takagi-san",
            ),
            (
                "Karakai Jouzu no Takagi-san Movie",
                "Karakai Jouzu no Takagi-san",
            ),
            (
                "Teasing Master Takagi-san: The Movie",
                "Teasing Master Takagi-san",
            ),
            ("Girls und Panzer Specials", "Girls und Panzer"),
            ("Girls und Panzer Movie", "Girls und Panzer"),
            ("Shiguang Dailiren III", "Shiguang Dailiren"),
            ("Link Click Season 3", "Link Click"),
            (
                "Hell Mode: Yarikomi-zuki no Gamer wa Haisettei no Isekai de Musou Suru 2nd Season",
                "Hell Mode: Yarikomi-zuki no Gamer wa Haisettei no Isekai de Musou Suru",
            ),
            (
                "Tensei Shitara Slime Datta Ken 4th Season Part 1 & 2",
                "Tensei Shitara Slime Datta Ken",
            ),
            (
                "Youkoso Jitsuryoku Shijou Shugi no Kyoushitsu e 4th Season 2-nensei-hen Ichi Gakki",
                "Youkoso Jitsuryoku Shijou Shugi no Kyoushitsu e",
            ),
            (
                "Classroom of the Elite 4th Season: Second Year, First Semester",
                "Classroom of the Elite",
            ),
            (
                "Mushoku Tensei III: Isekai Ittara Honki Dasu",
                "Mushoku Tensei: Isekai Ittara Honki Dasu",
            ),
            (
                "Mushoku Tensei: Jobless Reincarnation Season 3",
                "Mushoku Tensei: Jobless Reincarnation",
            ),
            (
                "Gaikotsu Kishi-sama, Tadaima Isekai e Odekakechuu II",
                "Gaikotsu Kishi-sama, Tadaima Isekai e Odekakechuu",
            ),
            ("Ranma 1/2 (2024) 3rd Season", "Ranma 1/2"),
            ("Ranma1/2 (2024) Season 3", "Ranma1/2"),
            (
                "Re:ZERO -Starting Life in Another World- Season 4",
                "Re:ZERO -Starting Life in Another World-",
            ),
            (
                "Yasei no Last Boss ga Arawareta! 2nd Season",
                "Yasei no Last Boss ga Arawareta!",
            ),
            // Left as they are: no marker, or a number that is the name.
            ("Link Click: Bridon Arc", "Link Click: Bridon Arc"),
            ("Girls und Panzer der Film", "Girls und Panzer der Film"),
            (
                "The Angel Next Door Spoils Me Rotten2",
                "The Angel Next Door Spoils Me Rotten2",
            ),
            ("Kaiju No. 8", "Kaiju No. 8"),
            ("Mob Psycho 100", "Mob Psycho 100"),
            ("Ranma 1/2", "Ranma 1/2"),
            ("Bakemonogatari", "Bakemonogatari"),
        ];
        for (title, want) in cases {
            assert_eq!(strip_season(title), want, "{title}");
        }
    }

    #[test]
    fn the_group_title_is_the_shared_name_when_there_is_one() {
        assert_eq!(
            group_title(&["Shiguang Dailiren: Yingdu Pian", "Shiguang Dailiren III"]),
            "Shiguang Dailiren"
        );
        assert_eq!(
            group_title(&["Link Click: Bridon Arc", "Link Click Season 3"]),
            "Link Click"
        );
        assert_eq!(
            group_title(&[
                "Girls und Panzer",
                "Girls und Panzer OVA Series",
                "Girls und Panzer: Kore ga Hontou no Anzio-sen Desu!",
                "Girls und Panzer der Film",
            ]),
            "Girls und Panzer"
        );
        // A run of words that stops mid-name is no name: the earliest
        // member's stem stands.
        assert_eq!(
            group_title(&[
                "Karakai Jouzu no Takagi-san",
                "Karakai Jouzu no (Moto) Takagi-san"
            ]),
            "Karakai Jouzu no Takagi-san"
        );
        // Nothing shared at all: the earliest member's title, stripped.
        assert_eq!(
            group_title(&["Bakemonogatari", "Nisemonogatari"]),
            "Bakemonogatari"
        );
        assert_eq!(group_title(&["Fate/Zero", "Fate/stay night"]), "Fate/Zero");
    }
}
