//! The library page's sections, cut in Rust (#289). The page used to cut them in
//! JavaScript over every card object, which at 300 series was most of what a visit cost;
//! here it is a fraction of a millisecond, and QML is handed titles, facts and series ids
//! only. A card reads its own record when it is built (Door::card).
//!
//! The rules are the page's, unchanged: Airing first (a series airing now is there and
//! also wherever the cut puts it, except the current season's section), Dropped last and
//! nowhere else, franchises with two or more series as sections of their own when grouped
//! (in release order when every one has a date), then the cut: Season (newest first, "No
//! date" last), Type, Genre or Watch state (On hold last), or None (one plain list).

use std::collections::{HashMap, HashSet};

use serde_json::{Value, json};

pub struct Params<'a> {
    pub arrange: &'a str,
    pub group_franchises: bool,
    /// A genre picked on Home; "" for every series
    pub genre: &'a str,
    pub now_secs: f64,
    /// The season now, as a card's season label reads it ("Fall 2026")
    pub current_season: &'a str,
}

const SEASON_WORDS: [&str; 4] = ["Winter", "Spring", "Summer", "Fall"];

fn id_of(c: &Value) -> u64 {
    c["id"]
        .as_u64()
        .unwrap_or_else(|| c["id"].as_f64().unwrap_or(0.0) as u64)
}
fn str_of<'a>(c: &'a Value, k: &str) -> &'a str {
    c[k].as_str().unwrap_or("")
}
fn in_genre(c: &Value, genre: &str) -> bool {
    genre.is_empty()
        || c["genres"]
            .as_array()
            .is_some_and(|g| g.iter().any(|x| x.as_str() == Some(genre)))
}

fn type_of(c: &Value) -> &'static str {
    match str_of(c, "format") {
        "TV" | "TV_SHORT" => "TV",
        "MOVIE" => "Movies",
        "" if str_of(c, "kind") == "Movie" => "Movies",
        "OVA" | "ONA" | "SPECIAL" => "OVA, ONA and specials",
        "MUSIC" => "Music",
        _ => "Unknown",
    }
}

/// On hold is the tracker's list (AniList's Paused), whatever the episodes say
fn state_of(c: &Value) -> &'static str {
    if str_of(c, "list_status") == "Paused" {
        return "On hold";
    }
    match str_of(c, "watched_state") {
        "CaughtUp" => "Caught up",
        "Behind" => "Watching",
        _ if c["watched"].as_f64().unwrap_or(0.0) > 0.0 => "Watching",
        _ => "Not started",
    }
}

fn season_rank(c: &Value) -> i64 {
    let label = str_of(c, "season");
    let parts: Vec<&str> = label.split(' ').collect();
    let word = SEASON_WORDS
        .iter()
        .position(|w| Some(w) == parts.first())
        .unwrap_or(0) as i64;
    let year = c["year"]
        .as_f64()
        .map(|y| y as i64)
        .or_else(|| parts.last().and_then(|p| p.parse::<i64>().ok()));
    match year {
        Some(y) => y * 4 + word,
        None => -1,
    }
}

/// The date a franchise sorts by: the start date, else the year, else none
fn release_of(c: &Value) -> Option<String> {
    match &c["start_date"] {
        Value::String(s) if !s.is_empty() => return Some(s.clone()),
        Value::Number(n) => return Some(n.to_string()),
        _ => {}
    }
    match &c["year"] {
        Value::Number(n) => Some(
            n.as_f64()
                .map(|y| (y as i64).to_string())
                .unwrap_or_default(),
        ),
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

fn short_countdown(secs: f64) -> String {
    let t = secs.floor().max(0.0) as i64;
    let (d, h, m) = (t / 86400, t % 86400 / 3600, t % 3600 / 60);
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{}m", m.max(1))
    }
}

/// A section's one fact: what its name and count do not already say
fn fact_of(list: &[&Value], now: f64) -> String {
    let (mut going, mut fresh, mut done, mut files) = (0, 0, 0, 0u64);
    let mut soon: Option<f64> = None;
    for c in list {
        let w = c["watched"].as_f64().unwrap_or(0.0);
        files += c["episodes_on_disk"].as_f64().unwrap_or(0.0) as u64;
        let airing = !c["next_airing"].is_null();
        if str_of(c, "watched_state") == "CaughtUp" && !airing {
            done += 1;
        } else if w > 0.0 || str_of(c, "watched_state") == "Behind" {
            going += 1;
        } else {
            fresh += 1;
        }
        if let Some(at) = c["next_airing"]["at"].as_f64()
            && at > now
            && soon.is_none_or(|s| at < s)
        {
            soon = Some(at);
        }
    }
    let n = list.len();
    if let Some(at) = soon {
        return format!("Next episode in {}", short_countdown(at - now));
    }
    if going > 0 && going < n {
        return format!("{going} in progress");
    }
    if fresh > 0 && fresh < n {
        return format!("{fresh} not started");
    }
    if n > 0 && done == n {
        return "All watched".into();
    }
    if files > 0 {
        return format!(
            "{} on disk",
            crate::format::plural(files, "episode", "episodes")
        );
    }
    String::new()
}

fn dropped_fact(list: &[&Value]) -> String {
    let files: u64 = list
        .iter()
        .map(|c| c["episodes_on_disk"].as_f64().unwrap_or(0.0) as u64)
        .sum();
    format!(
        "{} still on disk",
        crate::format::plural(files, "file", "files")
    )
}

struct Section<'a> {
    title: String,
    cards: Vec<&'a Value>,
    airing: bool,
    dropped: bool,
}

fn any_airing(cards: &[&Value], ids: &HashSet<u64>) -> bool {
    cards.iter().any(|c| ids.contains(&id_of(c)))
}

fn season_sections<'a>(list: &[&'a Value], ids: &HashSet<u64>, current: &str) -> Vec<Section<'a>> {
    let mut buckets: HashMap<String, (i64, Vec<&'a Value>)> = HashMap::new();
    let mut keys: Vec<String> = vec![];
    let mut undated: Vec<&'a Value> = vec![];
    for c in list {
        let s = str_of(c, "season");
        if s.is_empty() {
            undated.push(c);
            continue;
        }
        // An airing series is already under Airing; the season now holds only the ones
        // done airing, so nothing is shown twice on the same ground
        if s == current && ids.contains(&id_of(c)) {
            continue;
        }
        let b = buckets.entry(s.to_string()).or_insert_with(|| {
            keys.push(s.to_string());
            (season_rank(c), vec![])
        });
        b.1.push(c);
    }
    keys.sort_by(|p, q| buckets[q].0.cmp(&buckets[p].0));
    let mut out: Vec<Section> = keys
        .into_iter()
        .map(|k| {
            let cards = buckets.remove(&k).map(|b| b.1).unwrap_or_default();
            Section {
                airing: any_airing(&cards, ids),
                title: k,
                cards,
                dropped: false,
            }
        })
        .collect();
    if !undated.is_empty() {
        out.push(Section {
            title: "No date".into(),
            airing: any_airing(&undated, ids),
            cards: undated,
            dropped: false,
        });
    }
    out
}

/// The sections of `reply` (ListLibrary's, lean or whole) as QML reads them: `{ key,
/// sections: [{ title, airing, dropped, count, ids }], facts: [one per section], ids, count,
/// counts }`. `key` names the structure, so a view that only moved its facts is seen as one.
pub fn view(reply: &Value, p: &Params) -> Value {
    let series: Vec<&Value> = reply["series"]
        .as_array()
        .map(|a| a.iter().filter(|c| in_genre(c, p.genre)).collect())
        .unwrap_or_default();
    let airing_ids: HashSet<u64> = reply["airing"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|c| in_genre(c, p.genre))
                .map(id_of)
                .collect()
        })
        .unwrap_or_default();
    let sections: Vec<Section> = if p.arrange == "None" {
        if series.is_empty() {
            vec![]
        } else {
            vec![Section {
                title: String::new(),
                cards: series.clone(),
                airing: false,
                dropped: false,
            }]
        }
    } else {
        cut(&series, &airing_ids, p)
    };
    let structure: Vec<Value> = sections
        .iter()
        .map(|s| {
            json!({
                "title": s.title,
                "airing": s.airing,
                "dropped": s.dropped,
                "count": s.cards.len(),
                "ids": s.cards.iter().map(|c| id_of(c)).collect::<Vec<_>>(),
            })
        })
        .collect();
    // The facts move with the clock ("Next episode in 4h 12m"); kept apart from the
    // structure, and the structure keyed by a hash, so the page's 30 s tick changes a few
    // labels and never rebuilds a section or a card (#289)
    let facts: Vec<String> = sections
        .iter()
        .map(|s| {
            if s.dropped {
                dropped_fact(&s.cards)
            } else {
                fact_of(&s.cards, p.now_secs)
            }
        })
        .collect();
    let key = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        serde_json::to_string(&structure)
            .unwrap_or_default()
            .hash(&mut h);
        h.finish().to_string()
    };
    json!({
        "key": key,
        "sections": structure,
        "facts": facts,
        "ids": series.iter().map(|c| id_of(c)).collect::<Vec<_>>(),
        "count": series.len(),
        "counts": reply["counts"].clone(),
    })
}

fn cut<'a>(series: &[&'a Value], ids: &HashSet<u64>, p: &Params) -> Vec<Section<'a>> {
    let airing: Vec<&Value> = series
        .iter()
        .copied()
        .filter(|c| ids.contains(&id_of(c)))
        .collect();
    let mut out: Vec<Section> = vec![];
    if !airing.is_empty() {
        out.push(Section {
            title: "Airing".into(),
            cards: airing,
            airing: true,
            dropped: false,
        });
    }
    let dropped: Vec<&Value> = series
        .iter()
        .copied()
        .filter(|c| str_of(c, "list_status") == "Dropped")
        .collect();
    let kept: Vec<&Value> = series
        .iter()
        .copied()
        .filter(|c| str_of(c, "list_status") != "Dropped")
        .collect();
    let mut rest: Vec<&Value> = vec![];
    if p.group_franchises {
        let mut fr: HashMap<String, (String, Vec<&Value>)> = HashMap::new();
        let mut order: Vec<String> = vec![];
        for c in &kept {
            let Some(f) = c["franchise"].as_object() else {
                rest.push(c);
                continue;
            };
            // The core sends the id as a string (`FranchiseTag::id`); one with no id stands alone
            let key = match f.get("id") {
                Some(Value::String(s)) => s.clone(),
                Some(v) if !v.is_null() => v.to_string(),
                _ => {
                    rest.push(c);
                    continue;
                }
            };
            let b = fr.entry(key.clone()).or_insert_with(|| {
                order.push(key);
                (
                    f.get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string(),
                    vec![],
                )
            });
            b.1.push(c);
        }
        let position: HashMap<u64, usize> = series
            .iter()
            .enumerate()
            .map(|(i, c)| (id_of(c), i))
            .collect();
        for k in order {
            let (title, mut cards) = fr.remove(&k).unwrap_or_default();
            if cards.len() < 2 {
                rest.extend(cards);
                continue;
            }
            if cards.iter().all(|c| release_of(c).is_some()) {
                cards.sort_by_key(|c| release_of(c));
            }
            out.push(Section {
                airing: any_airing(&cards, ids),
                title,
                cards,
                dropped: false,
            });
        }
        // The grouped ones keep the library's order among themselves
        rest.sort_by_key(|c| position.get(&id_of(c)).copied().unwrap_or(usize::MAX));
    } else {
        rest = kept;
    }
    if p.arrange == "Season" {
        out.extend(season_sections(&rest, ids, p.current_season));
    } else {
        let mut buckets: HashMap<String, Vec<&Value>> = HashMap::new();
        let mut order: Vec<String> = vec![];
        for c in rest {
            let key = match p.arrange {
                "Type" => type_of(c).to_string(),
                "Genre" => c["genres"][0].as_str().unwrap_or("Unmatched").to_string(),
                _ => state_of(c).to_string(),
            };
            buckets
                .entry(key.clone())
                .or_insert_with(|| {
                    order.push(key);
                    vec![]
                })
                .push(c);
        }
        // On hold closes the watch states, right before Dropped
        if let Some(i) = order.iter().position(|k| k == "On hold") {
            let k = order.remove(i);
            order.push(k);
        }
        for k in order {
            let cards = buckets.remove(&k).unwrap_or_default();
            out.push(Section {
                airing: any_airing(&cards, ids),
                title: k,
                cards,
                dropped: false,
            });
        }
    }
    if !dropped.is_empty() {
        out.push(Section {
            title: "Dropped".into(),
            cards: dropped,
            airing: false,
            dropped: true,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(id: u64, extra: Value) -> Value {
        let mut c = json!({ "id": id, "title": format!("S{id}"), "kind": "Show" });
        if let (Some(o), Some(e)) = (c.as_object_mut(), extra.as_object()) {
            for (k, v) in e {
                o.insert(k.clone(), v.clone());
            }
        }
        c
    }
    fn params(arrange: &'static str) -> Params<'static> {
        Params {
            arrange,
            group_franchises: true,
            genre: "",
            now_secs: 1000.0,
            current_season: "Fall 2026",
        }
    }
    fn titles(v: &Value) -> Vec<String> {
        v["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["title"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn airing_leads_dropped_closes_and_seasons_run_newest_first() {
        let reply = json!({
            "series": [
                card(1, json!({ "season": "Spring 2024", "year": 2024 })),
                card(2, json!({ "season": "Fall 2025", "year": 2025 })),
                card(3, json!({ "season": "Fall 2026", "year": 2026 })),
                card(4, json!({ "list_status": "Dropped", "episodes_on_disk": 3 })),
                card(5, json!({})),
            ],
            "airing": [card(3, json!({}))],
            "counts": { "all": 5 }
        });
        let v = view(&reply, &params("Season"));
        assert_eq!(
            titles(&v),
            ["Airing", "Fall 2025", "Spring 2024", "No date", "Dropped"]
        );
        assert_eq!(
            v["sections"][0]["ids"],
            json!([3]),
            "the current season's airing series is under Airing only"
        );
        assert_eq!(v["facts"][4], "3 files still on disk");
        assert_eq!(v["count"], 5);
    }

    #[test]
    fn a_franchise_of_two_is_its_own_section_in_release_order() {
        let f = json!({ "id": "7", "title": "Saga" });
        let reply = json!({
            "series": [
                card(1, json!({ "franchise": f, "start_date": "2020-04-01", "season": "Spring 2020" })),
                card(2, json!({ "franchise": f, "start_date": "2017-01-01", "season": "Winter 2017" })),
                card(3, json!({ "season": "Spring 2020", "year": 2020 })),
            ],
            "airing": []
        });
        let v = view(&reply, &params("Season"));
        assert_eq!(titles(&v), ["Saga", "Spring 2020"]);
        assert_eq!(v["sections"][0]["ids"], json!([2, 1]));
    }

    #[test]
    fn two_franchises_are_two_sections_each_under_its_own_name() {
        let a = json!({ "id": "101", "title": "Girls und Panzer" });
        let b = json!({ "id": "202", "title": "Link Click" });
        let reply = json!({
            "series": [
                card(1, json!({ "franchise": b, "start_date": "2021-04-30" })),
                card(2, json!({ "franchise": a, "start_date": "2012-10-09" })),
                card(3, json!({ "franchise": b, "start_date": "2023-07-14" })),
                card(4, json!({ "franchise": a, "start_date": "2015-11-21" })),
            ],
            "airing": []
        });
        let v = view(&reply, &params("Season"));
        assert_eq!(titles(&v), ["Link Click", "Girls und Panzer"]);
        assert_eq!(v["sections"][0]["ids"], json!([1, 3]));
        assert_eq!(v["sections"][1]["ids"], json!([2, 4]));
    }

    #[test]
    fn none_is_one_plain_list_and_a_genre_filters_first() {
        let reply = json!({ "series": [card(1, json!({ "genres": ["Action"] })), card(2, json!({ "genres": ["Drama"] }))], "airing": [] });
        let mut p = params("None");
        assert_eq!(view(&reply, &p)["sections"][0]["ids"], json!([1, 2]));
        p.genre = "Drama";
        assert_eq!(view(&reply, &p)["ids"], json!([2]));
    }

    #[test]
    fn the_clock_moves_the_facts_and_not_the_key() {
        let reply = json!({ "series": [card(1, json!({ "season": "Fall 2025", "next_airing": { "at": 5000.0 } }))], "airing": [] });
        let mut p = params("Season");
        let a = view(&reply, &p);
        p.now_secs = 2000.0;
        let b = view(&reply, &p);
        assert_eq!(a["key"], b["key"]);
        assert_ne!(a["facts"], b["facts"]);
    }

    #[test]
    fn facts_say_what_the_count_does_not() {
        let a = card(1, json!({ "watched": 2 }));
        let b = card(2, json!({}));
        assert_eq!(fact_of(&[&a, &b], 0.0), "1 in progress");
        let c = card(3, json!({ "next_airing": { "at": 1000.0 + 3600.0 * 5.0 } }));
        assert_eq!(fact_of(&[&c], 1000.0), "Next episode in 5h 0m");
        let d = card(4, json!({ "watched_state": "CaughtUp" }));
        assert_eq!(fact_of(&[&d], 0.0), "All watched");
    }
}
