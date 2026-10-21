//! Keeping a subscription to what the search showed, on an endpoint anirss
//! narrows on its own side.
//!
//! anirss refines a generic RSS endpoint (1337x through Jackett) by fetching
//! the broader query and filtering the releases itself; the endpoint never
//! sees the narrowing words. Subscribing hands anirss the narrowed query,
//! and anirss builds the feed from all of it, so the feed searches the
//! endpoint for words it cannot match ("Helluva Boss S03 1080p WEB-DL"
//! finds nothing on 1337x where "Helluva Boss S03" finds six) and the rule
//! never fires. The fix keeps anirss's rule and moves it: the feed becomes
//! the one the search actually read, and the words narrowed away from it
//! go into the rule's own filters, which qBittorrent applies to every
//! article. Words the endpoint can answer after all ("TBK" on 1337x) are
//! probed back into the feed's search, so the feed holds what was picked
//! and not every release of the show. Erai is left alone: anirss already moves its terms into the
//! rule (its per-show feed), and nyaa searches server-side.

use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

use crate::contract::{FeedEntry, Release};

/// anirss's own field split: a `-"quoted phrase"`, a `"quoted"` or a word.
static FIELDS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-?"[^"]*"|\S+"#).unwrap());

/// What the search narrowed on anirss's side, past the query the endpoint
/// was asked: words every release must hold, and words none may.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocalTerms {
    pub must: Vec<String>,
    pub not: Vec<String>,
}

impl LocalTerms {
    pub fn is_empty(&self) -> bool {
        self.must.is_empty() && self.not.is_empty()
    }
}

fn split(query: &str) -> (Vec<String>, Vec<String>) {
    let mut positive = Vec::new();
    let mut excluded = Vec::new();
    for m in FIELDS.find_iter(query) {
        let field = m.as_str();
        if let Some(rest) = field.strip_prefix('-')
            && !rest.is_empty()
        {
            let core = rest.trim_matches('"').trim();
            if !core.is_empty() {
                excluded.push(core.to_string());
            }
            continue;
        }
        // qBittorrent reads `[` as a wildcard set, and a feed title rarely
        // carries a group's brackets anyway: the word alone
        let core = field
            .trim_matches('"')
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim();
        if !core.is_empty() {
            positive.push(core.to_string());
        }
    }
    (positive, excluded)
}

fn contains(list: &[String], word: &str) -> bool {
    list.iter().any(|w| w.eq_ignore_ascii_case(word))
}

/// The words in `query` that `wire` (the query the endpoint was asked)
/// does not carry.
pub fn local_terms(query: &str, wire: &str) -> LocalTerms {
    let (q_pos, q_not) = split(query);
    let (w_pos, w_not) = split(wire);
    let mut out = LocalTerms::default();
    for w in q_pos {
        if !contains(&w_pos, &w) && !contains(&out.must, &w) {
            out.must.push(w);
        }
    }
    for w in q_not {
        if !contains(&w_not, &w) && !contains(&out.not, &w) {
            out.not.push(w);
        }
    }
    out
}

/// The rule as qBittorrent holds it, reading `feed_url` and keeping the
/// local terms: each must-word added to `mustContain` (space is qBittorrent's
/// AND), each not-word to `mustNotContain` (`|` is its OR). Everything else
/// in the rule (save path, tags, episode filter) stays as anirss wrote it.
pub fn narrowed(def: &Value, terms: &LocalTerms, feed_url: &str) -> Value {
    let mut out = def.clone();
    let must_now = def
        .get("mustContain")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut must: Vec<String> = must_now.split_whitespace().map(str::to_string).collect();
    for w in &terms.must {
        if !contains(&must, w) {
            must.push(w.clone());
        }
    }
    let not_now = def
        .get("mustNotContain")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut not: Vec<String> = not_now
        .split('|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    for w in &terms.not {
        if !contains(&not, w) {
            not.push(w.clone());
        }
    }
    if let Some(map) = out.as_object_mut() {
        map.insert("mustContain".to_string(), Value::String(must.join(" ")));
        map.insert("mustNotContain".to_string(), Value::String(not.join("|")));
        map.insert(
            "affectedFeeds".to_string(),
            Value::Array(vec![Value::String(feed_url.to_string())]),
        );
    }
    out
}

/// Whether a feed still holds every picked release, judged by title as
/// the endpoint writes it (case and spacing aside): a torznab download link
/// can change from one search to the next, a title does not.
pub fn holds_all(entries: &[FeedEntry], picked: &[Release]) -> bool {
    let norm = |t: &str| {
        t.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    let titles: Vec<String> = entries.iter().map(|e| norm(&e.title)).collect();
    picked.iter().all(|r| titles.contains(&norm(&r.title)))
}

/// Whether any rule but `except` reads `feed_url`.
pub fn feed_shared(rules: &Value, except: &str, feed_url: &str) -> bool {
    rules.as_object().is_some_and(|map| {
        map.iter().any(|(name, def)| {
            name != except
                && def["affectedFeeds"]
                    .as_array()
                    .is_some_and(|f| f.iter().any(|u| u.as_str() == Some(feed_url)))
        })
    })
}

/// The search a feed URL carries: the `q` parameter (torznab, nyaa), else
/// `s` (Erai-style feeds), as (name, decoded value).
pub fn search_param(url: &str) -> Option<(String, String)> {
    let query = url.split_once('?')?.1;
    let pairs: Vec<(&str, &str)> = query.split('&').filter_map(|p| p.split_once('=')).collect();
    for want in ["q", "s"] {
        if let Some((k, v)) = pairs.iter().find(|(k, _)| *k == want) {
            let value = crate::percent::decode(v);
            if !value.trim().is_empty() {
                return Some((k.to_string(), value));
            }
        }
    }
    None
}

/// `url` with its `key` parameter set to `value`, the rest untouched.
pub fn with_search(url: &str, key: &str, value: &str) -> String {
    let Some((base, query)) = url.split_once('?') else {
        return url.to_string();
    };
    let parts: Vec<String> = query
        .split('&')
        .map(|p| match p.split_once('=') {
            Some((k, _)) if k == key => format!("{k}={}", crate::percent::encode(value)),
            _ => p.to_string(),
        })
        .collect();
    format!("{base}?{}", parts.join("&"))
}

/// The positive fields of a query as written (quotes kept), the order kept.
pub fn positive_fields(query: &str) -> Vec<String> {
    FIELDS
        .find_iter(query)
        .map(|m| m.as_str().to_string())
        .filter(|f| !(f.starts_with('-') && f.len() > 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_words_narrowed_past_the_wire_query_are_local() {
        let t = local_terms("Helluva.Boss. S03 1080p WEB-DL", "Helluva.Boss. S03");
        assert_eq!(t.must, vec!["1080p", "WEB-DL"]);
        assert!(t.not.is_empty());

        let t = local_terms(
            "Helluva Boss S03 -x265 \"AMZN WEB\" [TBK]",
            "helluva boss s03",
        );
        assert_eq!(t.must, vec!["AMZN WEB", "TBK"]);
        assert_eq!(t.not, vec!["x265"]);

        assert!(local_terms("Frieren 1080p", "Frieren 1080p").is_empty());
    }

    #[test]
    fn narrowing_keeps_the_rule_and_moves_its_feed() {
        let def = json!({
            "enabled": true, "mustContain": "", "mustNotContain": "HEVC",
            "affectedFeeds": ["http://j/x?q=Helluva+Boss+S03+1080p+WEB-DL"],
            "savePath": "/media/Helluva Boss", "torrentParams": {"tags": ["TV"]}
        });
        let terms = LocalTerms {
            must: vec!["1080p".into(), "WEB-DL".into()],
            not: vec!["x265".into(), "hevc".into()],
        };
        let out = narrowed(&def, &terms, "http://j/x?q=Helluva+Boss+S03");
        assert_eq!(out["mustContain"], "1080p WEB-DL");
        assert_eq!(out["mustNotContain"], "HEVC|x265");
        assert_eq!(
            out["affectedFeeds"],
            json!(["http://j/x?q=Helluva+Boss+S03"])
        );
        assert_eq!(out["savePath"], "/media/Helluva Boss");
        assert_eq!(out["torrentParams"]["tags"], json!(["TV"]));
        assert_eq!(out["enabled"], true);
    }

    #[test]
    fn a_feed_holds_the_picked_releases_by_title() {
        let entry = |t: &str| FeedEntry {
            title: t.to_string(),
            link: String::new(),
            page: String::new(),
            published: None,
            size: String::new(),
            size_bytes: None,
            seeders: None,
            leechers: None,
            downloads: None,
            info_hash: String::new(),
        };
        let pick = |t: &str| -> Release {
            serde_json::from_value(json!({"title": t, "link": "http://j/dl/other"})).unwrap()
        };
        let feed = [entry("Show.S03E04-TBK"), entry("Show  S03E05 TBK")];
        assert!(holds_all(
            &feed,
            &[pick("show.s03e04-tbk"), pick("Show S03E05 TBK")]
        ));
        assert!(!holds_all(&feed, &[pick("Show.S03E06-TBK")]));
        assert!(!holds_all(&[], &[pick("Show.S03E04-TBK")]));
    }

    #[test]
    fn a_feed_another_rule_reads_is_shared() {
        let rules = json!({
            "A": {"affectedFeeds": ["u1"]},
            "B": {"affectedFeeds": ["u2"]}
        });
        assert!(!feed_shared(&rules, "A", "u1"));
        assert!(feed_shared(&rules, "B", "u1"));
    }

    #[test]
    fn a_feeds_search_reads_and_rewrites() {
        let url = "http://127.0.0.1:9117/api/torznab/api?apikey=K&t=search&q=Helluva.Boss.+S03+TBK";
        assert_eq!(
            search_param(url),
            Some(("q".to_string(), "Helluva.Boss. S03 TBK".to_string()))
        );
        let shorter = with_search(url, "q", "Helluva.Boss. S03");
        assert_eq!(
            shorter,
            "http://127.0.0.1:9117/api/torznab/api?apikey=K&t=search&q=Helluva.Boss.%20S03"
        );
        assert_eq!(search_param(&shorter).unwrap().1, "Helluva.Boss. S03");
        let erai = "https://www.erai-raws.info/episodes/feed/?res=1080p&s=Ranma+1%2F2&token=t";
        assert_eq!(
            search_param(erai).unwrap(),
            ("s".to_string(), "Ranma 1/2".to_string())
        );
        assert_eq!(search_param("https://x/feed"), None);
        assert_eq!(
            positive_fields("Helluva S03 -HEVC \"AMZN WEB\""),
            vec!["Helluva", "S03", "\"AMZN WEB\""]
        );
    }
}
