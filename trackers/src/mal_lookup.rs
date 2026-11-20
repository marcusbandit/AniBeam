//! The MAL id of a show AniList gives none for (#782). AniList's `idMal`
//! is the only place a match learns its MAL id, and it is often empty for
//! new shows, specials and OVAs, so a write to MAL had nowhere to go and
//! was skipped without a word. A write to a connected MAL account for such
//! a show asks MAL's own search once, with the AniList titles, and takes
//! the one result whose title, year and type all agree. Anything less
//! certain is no answer. The answer is kept in `mal_lookups` either way:
//! a hit for good, and a miss for a week, since a new show is often marked
//! before MAL lists it.
//!
//! One search per show, at its first write, through the shared limiter:
//! never a sweep over the library, since MAL is strict about bursts.

use rusqlite::{Connection, OptionalExtension, params};

use crate::Trackers;
use anibeam_contract::*;
use anibeam_foundation::percent::encode;
use anibeam_foundation::time;

/// MAL's list API, the same address the writes go to.
const MAL_API: &str = "https://api.myanimelist.net/v2";

/// MAL turns a longer query down, so the search is cut to this many
/// characters.
const QUERY_LIMIT: usize = 64;

/// How long a search that found nothing stands before the show may be
/// searched again: a new show is often marked before MAL lists it.
const MISS_STANDS: i64 = 7 * 24 * 3600;

/// What `mal_lookups` holds for an AniList id: `None` when it was never
/// looked up or its miss is older than [`MISS_STANDS`], `Some(None)` when
/// the search found nothing it could be sure of, and `Some(Some(id))` for
/// a hit.
pub(crate) fn saved(conn: &Connection, anilist_id: u64) -> Result<Option<Option<u64>>, CoreError> {
    let row = conn
        .query_row(
            "SELECT mal_id, looked_at FROM mal_lookups WHERE anilist_id = ?1",
            params![anilist_id as i64],
            |r| Ok((r.get::<_, Option<i64>>(0)?, r.get::<_, i64>(1)?)),
        )
        .optional()?;
    Ok(match row {
        Some((Some(id), _)) => Some(Some(id as u64)),
        Some((None, at)) if at > time::now_secs() - MISS_STANDS => Some(None),
        _ => None,
    })
}

/// Whether a write may still look this AniList id up: it never was, and
/// the library holds a title to search by.
pub(crate) fn can_look_up(conn: &Connection, anilist_id: u64) -> Result<bool, CoreError> {
    Ok(saved(conn, anilist_id)?.is_none() && wanted(conn, anilist_id)?.query.is_some())
}

/// The AniList side of a search: every title the show goes by, its start
/// year and its format.
#[derive(Debug, Default)]
struct Wanted {
    query: Option<String>,
    titles: Vec<String>,
    /// AniList's season year and the year of its start date, which differ
    /// for a show that starts in December and counts as the next winter.
    years: Vec<i64>,
    format: Option<String>,
}

fn wanted(conn: &Connection, anilist_id: u64) -> Result<Wanted, CoreError> {
    type Row = (
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        Option<i64>,
        Option<String>,
        Option<String>,
    );
    let row: Option<Row> = conn
        .query_row(
            "SELECT title_romaji, title_english, title_native, synonyms, year, format, start_date
               FROM anilist_media WHERE id = ?1",
            params![anilist_id as i64],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            },
        )
        .optional()?;
    let Some((romaji, english, native, synonyms, year, format, start)) = row else {
        return Ok(Wanted::default());
    };
    let synonyms: Vec<String> = serde_json::from_str(&synonyms).unwrap_or_default();
    // MAL names a show by its romaji title, so that is what it finds best.
    let query = romaji
        .clone()
        .or_else(|| english.clone())
        .filter(|q| q.chars().filter(|c| c.is_alphanumeric()).count() >= 3)
        .map(|q| {
            q.chars()
                .take(QUERY_LIMIT)
                .collect::<String>()
                .trim()
                .to_string()
        });
    let titles = [romaji, english, native]
        .into_iter()
        .flatten()
        .chain(synonyms)
        .collect();
    let years = year
        .into_iter()
        .chain(start.as_deref().and_then(year_of))
        .collect();
    Ok(Wanted {
        query,
        titles,
        years,
        format,
    })
}

/// The year a `YYYY-MM-DD` date (or a bare `YYYY`) starts with.
fn year_of(date: &str) -> Option<i64> {
    date.get(..4).and_then(|y| y.parse().ok())
}

/// A title reduced to its letters and digits, lowercased, so punctuation,
/// spacing and case never decide a match.
fn fold(title: &str) -> String {
    title
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// The kind of thing a show is, as far as telling two results apart
/// needs: a film, a music video, or episodes of some sort. AniList and MAL
/// often file one OVA, special or web series under different types, so
/// those all count as one kind; a film and a series of the same name do
/// not.
#[derive(PartialEq)]
enum Kind {
    Film,
    Music,
    Episodes,
}

fn kind_of_format(format: &str) -> Option<Kind> {
    match format {
        "MOVIE" => Some(Kind::Film),
        "MUSIC" => Some(Kind::Music),
        "TV" | "TV_SHORT" | "OVA" | "ONA" | "SPECIAL" => Some(Kind::Episodes),
        _ => None,
    }
}

fn kind_of_media_type(media_type: &str) -> Option<Kind> {
    match media_type {
        "movie" => Some(Kind::Film),
        "music" => Some(Kind::Music),
        "tv" | "ova" | "ona" | "special" | "tv_special" => Some(Kind::Episodes),
        _ => None,
    }
}

/// Whether AniList's format and MAL's media type name the same kind of
/// show. Either side unknown is no evidence against it.
fn same_kind(format: Option<&str>, media_type: Option<&str>) -> bool {
    match (
        format.and_then(kind_of_format),
        media_type.and_then(kind_of_media_type),
    ) {
        (Some(a), Some(b)) => a == b,
        _ => true,
    }
}

/// The one result that is surely the show, or None: every title it goes by
/// is compared with every AniList title, and its start year and type must
/// not disagree. Two results that pass are as good as none.
fn pick(wanted: &Wanted, body: &serde_json::Value) -> Option<u64> {
    let ours: Vec<String> = wanted
        .titles
        .iter()
        .map(|t| fold(t))
        .filter(|t| !t.is_empty())
        .collect();
    let mut hits = body["data"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|d| &d["node"])
        .filter(|node| {
            let alt = &node["alternative_titles"];
            let theirs = [&node["title"], &alt["en"], &alt["ja"]]
                .into_iter()
                .filter_map(|v| v.as_str())
                .chain(
                    alt["synonyms"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|v| v.as_str()),
                )
                .map(fold)
                .filter(|t| !t.is_empty());
            let named = theirs.into_iter().any(|t| ours.contains(&t));
            let year = node["start_date"].as_str().and_then(year_of);
            let same_year = match year {
                Some(y) if !wanted.years.is_empty() => wanted.years.contains(&y),
                _ => true,
            };
            named && same_year && same_kind(wanted.format.as_deref(), node["media_type"].as_str())
        })
        .filter_map(|node| node["id"].as_u64());
    let first = hits.next()?;
    hits.all(|id| id == first).then_some(first)
}

/// The MAL id for this AniList id: the saved answer when there is one,
/// otherwise one search, whose answer is saved before it is returned. A
/// failed search is the caller's error and saves nothing, so the next
/// write tries again.
pub(crate) async fn resolve(
    trackers: &Trackers,
    anilist_id: u64,
    token: &str,
) -> Result<Option<u64>, CoreError> {
    let (saved, wanted) = trackers
        .services
        .store
        .write_async(move |c| Ok((saved(c, anilist_id)?, wanted(c, anilist_id)?)))
        .await?;
    if let Some(answer) = saved {
        return Ok(answer);
    }
    // Nothing to search by yet: no answer, and nothing kept, so a show
    // whose record arrives later is still looked up.
    let Some(query) = &wanted.query else {
        return Ok(None);
    };
    let url = format!(
        "{MAL_API}/anime?q={}&limit=10&nsfw=true&fields=alternative_titles,start_date,media_type",
        encode(query)
    );
    let response = trackers.net.mal.get(&url, token).await?;
    let found = pick(&wanted, &crate::mal_json(&response)?);
    let now = time::now_secs();
    trackers
        .services
        .store
        .write_async(move |c| {
            c.execute(
                "INSERT INTO mal_lookups (anilist_id, mal_id, looked_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT (anilist_id) DO UPDATE SET mal_id = excluded.mal_id, looked_at = excluded.looked_at",
                params![anilist_id as i64, found.map(|id| id as i64), now],
            )?;
            Ok(())
        })
        .await?;
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn wanted() -> Wanted {
        Wanted {
            query: Some("Sousou no Frieren".into()),
            titles: vec![
                "Sousou no Frieren".into(),
                "Frieren: Beyond Journey's End".into(),
            ],
            years: vec![2023],
            format: Some("TV".into()),
        }
    }

    fn node(id: u64, title: &str, start: &str, media_type: &str) -> serde_json::Value {
        json!({ "node": {
            "id": id,
            "title": title,
            "alternative_titles": { "en": "", "ja": "", "synonyms": [] },
            "start_date": start,
            "media_type": media_type,
        }})
    }

    #[test]
    fn the_one_result_with_the_same_title_year_and_type_is_taken() {
        let body = json!({ "data": [
            node(52991, "Sousou no Frieren", "2023-09-29", "tv"),
            node(56885, "Sousou no Frieren: Marumaru no Mahou", "2023-10-06", "ona"),
        ]});
        assert_eq!(pick(&wanted(), &body), Some(52991));
    }

    #[test]
    fn an_english_title_on_mal_counts_as_the_same_name() {
        let mut n = node(52991, "Sousou no Frieren!", "2023-09-29", "tv");
        n["node"]["title"] = json!("Something else");
        n["node"]["alternative_titles"]["en"] = json!("Frieren - Beyond Journey's End");
        assert_eq!(pick(&wanted(), &json!({ "data": [n] })), Some(52991));
    }

    #[test]
    fn another_year_or_another_type_is_not_the_show() {
        let year = json!({ "data": [node(1, "Sousou no Frieren", "2026-01-10", "tv")] });
        assert_eq!(pick(&wanted(), &year), None);
        let kind = json!({ "data": [node(1, "Sousou no Frieren", "2023-09-29", "movie")] });
        assert_eq!(pick(&wanted(), &kind), None);
    }

    #[test]
    fn two_results_that_both_fit_are_no_answer() {
        let body = json!({ "data": [
            node(1, "Sousou no Frieren", "2023-09-29", "tv"),
            node(2, "Sousou no Frieren", "2023-01-01", "tv"),
        ]});
        assert_eq!(pick(&wanted(), &body), None);
    }

    #[test]
    fn a_title_that_only_starts_the_same_is_not_a_match() {
        let body = json!({ "data": [node(1, "Sousou no Frieren 2nd Season", "2023-09-29", "tv")] });
        assert_eq!(pick(&wanted(), &body), None);
    }

    #[test]
    fn an_unknown_year_or_type_on_either_side_is_no_evidence_against() {
        let mut w = wanted();
        w.years = Vec::new();
        w.format = None;
        let body = json!({ "data": [node(7, "Sousou no Frieren", "", "unknown")] });
        assert_eq!(pick(&w, &body), Some(7));
    }

    #[test]
    fn an_ova_on_one_side_and_a_special_on_the_other_is_the_same_show() {
        let mut w = wanted();
        w.format = Some("OVA".into());
        let body = json!({ "data": [node(3, "Sousou no Frieren", "2023-09-29", "special")] });
        assert_eq!(pick(&w, &body), Some(3));
    }

    #[test]
    fn a_december_start_matches_either_year_anilist_gives() {
        let mut w = wanted();
        w.years = vec![2024, 2023];
        let body = json!({ "data": [node(4, "Sousou no Frieren", "2023-12-28", "tv")] });
        assert_eq!(pick(&w, &body), Some(4));
    }
}
