//! What the first scan of a new folder found (#206), in the numbers the summary sheet
//! shows: how many series the folder holds, how many matched a provider, the ones that
//! need the user (in title order, for the walk through the match panel), how many
//! episodes are on disk and how many of those the tracker already counts as watched.
//!
//! The input is the series list the core answers `ListSeries` with; a series belongs to
//! the folder when its path is the folder or sits under it. One the scan marked missing is
//! left out.

use serde_json::{Value, json};

/// A path with its separators made one kind and no trailing one, for comparing.
fn norm(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_string()
}

fn under(path: &str, folder: &str) -> bool {
    let (p, f) = (norm(path), norm(folder));
    !f.is_empty() && (p == f || p.strip_prefix(&f).is_some_and(|rest| rest.starts_with('/')))
}

/// Matched means a provider record is attached, by any of the ids the match carries.
fn matched(card: &Value) -> bool {
    let m = &card["match_info"];
    ["anilist_id", "mal_id", "tvmaze_id", "tmdb_id"]
        .iter()
        .any(|k| !m[*k].is_null())
}

fn count(card: &Value, key: &str) -> u64 {
    card[key].as_u64().unwrap_or(0)
}

pub fn summarise(series: &[Value], folder: &str) -> Value {
    let mut cards: Vec<&Value> = series
        .iter()
        .filter(|c| !c["missing"].as_bool().unwrap_or(false))
        .filter(|c| under(c["path"].as_str().unwrap_or(""), folder))
        .collect();
    cards.sort_by_key(|c| c["title"].as_str().unwrap_or("").to_lowercase());
    let need: Vec<Value> = cards
        .iter()
        .filter(|c| !matched(c))
        .map(|c| {
            json!({
                "id": c["id"],
                "title": c["title"],
                "match_info": c["match_info"],
            })
        })
        .collect();
    // A film's file has no episode number, so it counts as the one episode it is
    let episodes: u64 = cards
        .iter()
        .map(|c| count(c, "distinct_episodes").max(count(c, "episodes_on_disk").min(1)))
        .sum();
    json!({
        "shows": cards.len(),
        "recognised": cards.len() - need.len(),
        "need_you": need,
        "episodes": episodes,
        "watched": cards.iter().map(|c| count(c, "watched_on_disk")).sum::<u64>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(id: u64, title: &str, path: &str, anilist: Option<u64>, eps: u64, watched: u64) -> Value {
        json!({
            "id": id,
            "title": title,
            "path": path,
            "missing": false,
            "match_info": anilist.map(|a| json!({
                "anilist_id": a, "mal_id": null, "tvmaze_id": null, "tmdb_id": null,
                "provider": "Anilist", "confirmed": false
            })),
            "distinct_episodes": eps,
            "episodes_on_disk": eps,
            "watched_on_disk": watched,
        })
    }

    #[test]
    fn counts_only_the_folders_own_series() {
        let series = vec![
            card(1, "Frieren", "/media/Anime/Frieren", Some(154587), 28, 12),
            card(2, "Mushishi", "/media/Anime/Mushishi", Some(457), 26, 0),
            card(3, "Old Show", "/media/Other/Old Show", Some(1), 10, 10),
            // A sibling folder that only shares a prefix is not inside
            card(4, "Prefix", "/media/Anime2/Prefix", Some(2), 5, 0),
        ];
        let s = summarise(&series, "/media/Anime");
        assert_eq!(s["shows"], 2);
        assert_eq!(s["recognised"], 2);
        assert_eq!(s["episodes"], 54);
        assert_eq!(s["watched"], 12);
        assert_eq!(s["need_you"], json!([]));
    }

    #[test]
    fn the_unmatched_need_the_user_in_title_order() {
        let series = vec![
            card(5, "zeta folder", "/a/zeta folder", None, 3, 0),
            card(6, "Alpha folder", "/a/Alpha folder", None, 2, 0),
            card(7, "Matched", "/a/Matched", Some(9), 1, 0),
        ];
        let s = summarise(&series, "/a/");
        assert_eq!(s["shows"], 3);
        assert_eq!(s["recognised"], 1);
        let need: Vec<u64> = s["need_you"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["id"].as_u64().unwrap())
            .collect();
        assert_eq!(need, vec![6, 5]);
    }

    #[test]
    fn a_match_with_only_another_providers_id_counts() {
        let mut c = card(8, "Show", "/a/Show", None, 1, 0);
        c["match_info"] = json!({ "anilist_id": null, "mal_id": null, "tvmaze_id": 42, "tmdb_id": null });
        assert_eq!(summarise(&[c], "/a")["recognised"], 1);
    }

    #[test]
    fn missing_series_are_left_out() {
        let mut c = card(9, "Gone", "/a/Gone", Some(1), 4, 0);
        c["missing"] = json!(true);
        assert_eq!(summarise(&[c], "/a")["shows"], 0);
    }

    #[test]
    fn a_film_counts_as_one_episode() {
        let mut c = card(10, "A Film", "/a/Movies/A Film", Some(3), 0, 0);
        c["episodes_on_disk"] = json!(1);
        assert_eq!(summarise(&[c], "/a")["episodes"], 1);
    }

    #[test]
    fn windows_paths_compare_by_folder() {
        let c = card(11, "Show", "C:\\Anime\\Show", Some(1), 2, 1);
        let s = summarise(&[c], "C:\\Anime");
        assert_eq!(s["shows"], 1);
        assert_eq!(s["watched"], 1);
    }

    #[test]
    fn an_empty_folder_finds_nothing() {
        let s = summarise(&[], "/a");
        assert_eq!(s["shows"], 0);
        assert_eq!(s["episodes"], 0);
    }
}
