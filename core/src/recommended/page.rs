//! The page for one AniList show the library may not have (#779). It draws
//! what the core already holds: the recommender's copy of the show, else
//! the library's metadata, else Airing now's row. The first opening reads
//! AniList once for what only the page draws, the relations by name,
//! AniList's own recommendations and the trailer, and a week later the
//! next opening reads it again.

use std::path::Path;
use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use super::mine::Mine;
use super::{Display, because, fit, parse, poster, queries, sync, text};
use crate::contract::*;
use crate::core::Core;
use crate::jobs::Finished;
use crate::prefs;
use crate::time;

/// How long the page's own read is good for.
pub const PAGE_TTL: i64 = 7 * 86_400;

/// What the core holds for one show: its best copy, the page's own read and when that
/// read was made.
type Held = (Option<Value>, Option<Value>, Option<i64>);

/// What the core holds for one show, best copy first.
fn held(conn: &Connection, id: u64) -> Result<Held, CoreError> {
    let row: Option<(Option<String>, Option<String>, Option<i64>)> = conn
        .query_row(
            "SELECT media, page, page_fetched_at FROM rec_shows WHERE anilist_id = ?1",
            params![id as i64],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let (media, page, at) = row.unwrap_or_default();
    let json = |t: Option<String>| t.and_then(|t| serde_json::from_str::<Value>(&t).ok());
    let media = match json(media) {
        Some(m) => Some(m),
        None => library_copy(conn, id)?,
    };
    Ok((media, json(page), at))
}

/// The library's metadata row, or Airing now's, in the shape AniList sends,
/// for a show the recommender never read.
fn library_copy(conn: &Connection, id: u64) -> Result<Option<Value>, CoreError> {
    let media = conn
        .query_row(
            "SELECT mal_id, title_romaji, title_english, title_native, format, status, year,
                    episodes, duration, description, average_score, genres, tags, cover_url,
                    banner_url, site_url
               FROM anilist_media WHERE id = ?1 AND fetched_at IS NOT NULL",
            params![id as i64],
            |r| {
                let list = |i: usize| -> rusqlite::Result<Value> {
                    let text: String = r.get(i)?;
                    Ok(serde_json::from_str(&text).unwrap_or(json!([])))
                };
                // The library keeps one spoiler flag where AniList has two.
                let tags: Vec<Value> = list(12)?
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|t| {
                        json!({ "name": t["name"], "rank": t["rank"],
                                "isMediaSpoiler": t["spoiler"], "isAdult": t["adult"] })
                    })
                    .collect();
                Ok(json!({
                    "id": id,
                    "idMal": r.get::<_, Option<i64>>(0)?,
                    "title": { "romaji": r.get::<_, Option<String>>(1)?,
                               "english": r.get::<_, Option<String>>(2)?,
                               "native": r.get::<_, Option<String>>(3)? },
                    "format": r.get::<_, Option<String>>(4)?,
                    "status": r.get::<_, Option<String>>(5)?,
                    "seasonYear": r.get::<_, Option<i64>>(6)?,
                    "episodes": r.get::<_, Option<i64>>(7)?,
                    "duration": r.get::<_, Option<i64>>(8)?,
                    "description": r.get::<_, Option<String>>(9)?,
                    "averageScore": r.get::<_, Option<i64>>(10)?,
                    "genres": list(11)?,
                    "tags": tags,
                    "coverImage": { "large": r.get::<_, Option<String>>(13)? },
                    "bannerImage": r.get::<_, Option<String>>(14)?,
                    "siteUrl": r.get::<_, Option<String>>(15)?,
                }))
            },
        )
        .optional()?;
    if media.is_some() {
        return Ok(media);
    }
    Ok(conn
        .query_row(
            "SELECT mal_id, title_romaji, title_english, title_native, format, episodes,
                    average_score, cover_url, banner_url, site_url, duration
               FROM season_shows WHERE anilist_id = ?1 LIMIT 1",
            params![id as i64],
            |r| {
                Ok(json!({
                    "id": id,
                    "idMal": r.get::<_, Option<i64>>(0)?,
                    "title": { "romaji": r.get::<_, Option<String>>(1)?,
                               "english": r.get::<_, Option<String>>(2)?,
                               "native": r.get::<_, Option<String>>(3)? },
                    "format": r.get::<_, Option<String>>(4)?,
                    "status": "RELEASING",
                    "episodes": r.get::<_, Option<i64>>(5)?,
                    "averageScore": r.get::<_, Option<i64>>(6)?,
                    "coverImage": { "large": r.get::<_, Option<String>>(7)? },
                    "bannerImage": r.get::<_, Option<String>>(8)?,
                    "siteUrl": r.get::<_, Option<String>>(9)?,
                    "duration": r.get::<_, Option<i64>>(10)?,
                }))
            },
        )
        .optional()?)
}

/// The biggest cover AniList has, for the page's poster.
fn big_cover(media: &Value) -> Option<String> {
    let c = &media["coverImage"];
    c["extraLarge"]
        .as_str()
        .or(c["large"].as_str())
        .map(str::to_string)
}

fn trailer(page: &Value) -> Option<String> {
    let id = page["trailer"]["id"].as_str()?;
    match page["trailer"]["site"].as_str()? {
        "youtube" => Some(format!("https://www.youtube.com/watch?v={id}")),
        "dailymotion" => Some(format!("https://www.dailymotion.com/video/{id}")),
        _ => None,
    }
}

/// The order relations read in: what comes before, what comes after, then
/// the rest as AniList listed them.
fn relation_order(kind: &str) -> u8 {
    match kind {
        "PREQUEL" => 0,
        "SEQUEL" => 1,
        "PARENT" => 2,
        "SIDE_STORY" => 3,
        _ => 4,
    }
}

/// The page off the database, no request and no waiting, and when the
/// page's own read was last made.
pub fn read(
    conn: &Connection,
    images_dir: &Path,
    id: u64,
) -> Result<(ShowPage, Option<i64>), CoreError> {
    let lang = prefs::load_preferences(conn)?.title_language;
    let mine = Mine::read(conn)?;
    let (media, page, read_at) = held(conn, id)?;
    let known = media.is_some();
    let media = media.unwrap_or_else(|| json!({ "id": id }));
    let d = Display { media: &media };
    let (romaji, english, native) = d.names();
    let ranked: Option<(i64, String)> = conn
        .query_row(
            "SELECT forecast, why FROM rec_ranked WHERE anilist_id = ?1",
            params![id as i64],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let why: Option<fit::Why> = ranked
        .as_ref()
        .and_then(|(_, w)| serde_json::from_str(w).ok());
    let listed = mine.listed.get(&id).copied();

    let page = page.unwrap_or(Value::Null);
    let mut relations: Vec<(u8, ShowRelation)> = Vec::new();
    for edge in page["named"]["edges"].as_array().into_iter().flatten() {
        let node = &edge["node"];
        let (Some(rid), Some("ANIME")) = (node["id"].as_u64(), node["type"].as_str()) else {
            continue;
        };
        let kind = edge["relationType"].as_str().unwrap_or("OTHER");
        // A cameo is no relation a page names: the same rule the
        // franchise graph keeps.
        if matches!(kind, "CHARACTER" | "OTHER") {
            continue;
        }
        let nd = Display { media: node };
        relations.push((
            relation_order(kind),
            ShowRelation {
                anilist_id: rid,
                title: nd.title(lang, rid),
                poster: poster(conn, images_dir, nd.cover_url().as_deref())?,
                relation: kind.to_string(),
                format: nd.string("format"),
                owned: mine.owned.get(&rid).copied(),
            },
        ));
    }
    relations.sort_by_key(|(order, _)| *order);
    let mut recommendations = Vec::new();
    for n in page["recommendations"]["nodes"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let m = &n["mediaRecommendation"];
        let (Some(rid), Some("ANIME")) = (m["id"].as_u64(), m["type"].as_str()) else {
            continue;
        };
        if n["rating"].as_i64().unwrap_or(0) <= 0 {
            continue;
        }
        let md = Display { media: m };
        recommendations.push(Recommendation {
            anilist_id: rid,
            title: md.title(lang, rid),
            poster: poster(conn, images_dir, md.cover_url().as_deref())?,
            owned: mine.owned.get(&rid).copied(),
            list_status: mine.listed.get(&rid).map(|l| l.status),
        });
    }
    let studios: Vec<String> = {
        let all = page["allStudios"]["nodes"].as_array();
        let mut names: Vec<(bool, String)> = all
            .into_iter()
            .flatten()
            .filter_map(|s| {
                Some((
                    s["isAnimationStudio"].as_bool() != Some(true),
                    s["name"].as_str()?.to_string(),
                ))
            })
            .collect();
        names.sort_by_key(|(not_animation, _)| *not_animation);
        let mut names: Vec<String> = names.into_iter().map(|(_, n)| n).collect();
        if names.is_empty() {
            names = media["studios"]["nodes"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|s| s["name"].as_str().map(str::to_string))
                .collect();
        }
        names.dedup();
        names
    };
    Ok((
        ShowPage {
            anilist_id: id,
            mal_id: media["idMal"].as_u64(),
            title: d.title(lang, id),
            romaji,
            english,
            native,
            poster: match poster(conn, images_dir, big_cover(&media).as_deref())? {
                Some(p) => Some(p),
                None => poster(conn, images_dir, d.cover_url().as_deref())?,
            },
            banner: poster(conn, images_dir, media["bannerImage"].as_str())?,
            color: media["coverImage"]["color"].as_str().map(str::to_string),
            synopsis: media["description"]
                .as_str()
                .map(text::story)
                .filter(|s| !s.is_empty()),
            format: d.string("format"),
            status: d.string("status"),
            year: media["seasonYear"].as_i64().map(|y| y as i32),
            episodes: d.u32("episodes"),
            duration: d.u32("duration"),
            average_score: d.u32("averageScore"),
            genres: parse::strings(&media["genres"]),
            tags: d
                .tags()
                .into_iter()
                .map(|(name, rank)| Tag {
                    name,
                    rank,
                    spoiler: false,
                    adult: false,
                })
                .collect(),
            studios,
            trailer: trailer(&page),
            relations: relations.into_iter().map(|(_, r)| r).collect(),
            recommendations,
            site_url: d
                .string("siteUrl")
                .or_else(|| Some(format!("https://anilist.co/anime/{id}"))),
            owned: mine.owned.get(&id).copied(),
            list_status: listed.map(|l| l.status),
            score: listed.and_then(|l| l.score),
            progress: listed.map_or(0, |l| l.progress),
            forecast: ranked.map(|(f, _)| f.clamp(0, 100) as u32),
            hidden_loved: why.as_ref().map_or(0, |w| w.hidden),
            because: match &why {
                Some(w) => because(conn, images_dir, lang, &mine, w)?,
                None => Vec::new(),
            },
            known,
        },
        read_at,
    ))
}

/// How long after a page read started another may start for the same show:
/// a read that failed is not asked again on every redraw.
const RETRY_SECS: i64 = 10 * 60;

/// The `GetShowPage` call: the page as the core holds it, and the read it
/// started when the page's own read is missing or a week old. One read runs
/// per show; a second opening joins the first one's, and one that failed is
/// tried again ten minutes on at the soonest.
pub fn get_call(core: &Core, anilist_id: u64) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let (page, read_at) = core.store.read(|c| read(c, &images_dir, anilist_id))?;
    let now = time::now_secs();
    let reading = if read_at.is_some_and(|at| now - at < PAGE_TTL) {
        None
    } else {
        start_read(core, anilist_id, now)?
    };
    Ok(Reply::ShowPage {
        page: Box::new(page),
        reading,
    })
}

fn start_read(core: &Core, anilist_id: u64, now: i64) -> Result<Option<u64>, CoreError> {
    let running: std::collections::HashSet<u64> =
        core.jobs.list().into_iter().map(|j| j.id).collect();
    let mut reads = core.show_reads.lock().unwrap();
    // A read that ended and is past the retry gap is forgotten, so the map
    // only ever holds the reads of the last few minutes.
    reads.retain(|_, (job, at)| running.contains(job) || now - *at < RETRY_SECS);
    if let Some((job, _)) = reads.get(&anilist_id) {
        return Ok(running.contains(job).then_some(*job));
    }
    let owner = core.owner()?;
    let job = core
        .jobs
        .clone()
        .start(JobKind::ReadShowPage, move |_| async move {
            read_page(owner, anilist_id).await
        });
    reads.insert(anilist_id, (job, now));
    Ok(Some(job))
}

async fn read_page(core: Arc<Core>, id: u64) -> Result<Finished, CoreError> {
    let data = core
        .anilist
        .graphql(&queries::page(), json!({ "id": id }), None)
        .await?;
    let media = data["Media"].clone();
    if media.is_null() {
        return Err(CoreError::invalid(
            "anilist_id",
            format!("AniList has no anime {id}"),
        ));
    }
    let mut urls: Vec<String> = Vec::new();
    urls.extend(big_cover(&media));
    urls.extend(Display { media: &media }.cover_url());
    urls.extend(media["bannerImage"].as_str().map(str::to_string));
    for edge in media["named"]["edges"].as_array().into_iter().flatten() {
        urls.extend(
            Display {
                media: &edge["node"],
            }
            .cover_url(),
        );
    }
    for n in media["recommendations"]["nodes"]
        .as_array()
        .into_iter()
        .flatten()
    {
        urls.extend(
            Display {
                media: &n["mediaRecommendation"],
            }
            .cover_url(),
        );
    }
    let now = time::now_secs();
    let text = media.to_string();
    core.store
        .write_async(move |c| {
            sync::put_show(c, id, Some(&text), now)?;
            c.execute(
                "UPDATE rec_shows SET page = ?2, page_fetched_at = ?3 WHERE anilist_id = ?1",
                params![id as i64, text, now],
            )?;
            Ok(())
        })
        .await?;
    // Every picture before the event, so the page draws local files.
    for (url, outcome) in core.images.ensure(&urls).await {
        if let Err(e) = outcome {
            tracing::debug!("a show page picture {url} was not fetched: {e}");
        }
    }
    Ok(Finished {
        plain: None,
        level: Level::Debug,
        message: format!("show page of AniList {id} read"),
        body: EventBody::ShowPageRead { anilist_id: id },
    })
}
