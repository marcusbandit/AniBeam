//! Airing now (#202): every show AniList has on air, not only the
//! library's, and the shows the user said they do not care about.
//!
//! Three rules shape the file. The page is AniList's whole season, so it
//! reads its own tables (`season_shows`, `season_aired`) and never writes a
//! browsed show into the library's: a season the user only looked at must
//! not turn up in the franchise graph or the image fill. Not interested is
//! one AniList entry and nothing else; nothing here looks at relations, so
//! a show's next season is a new entry and comes back on its own. And the
//! page refreshes the way Watching does, on open when the cache is stale,
//! with no interval of its own: an episode whose time has passed reads as
//! out from the cached schedule, no request needed.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::library::titles;
use crate::net::anilist::{AIRED_PAGE_QUERY, SEASON_PAGE_QUERY};
use crate::prefs;
use crate::time;
use crate::trackers::{NO_TOKEN, accounts, as_u32, writes};

/// How old a season may be before `RefreshAiringNow` without `force` goes
/// back to AniList.
pub const STALE_AFTER_SECS: i64 = 30 * 60;

/// How far back the strip of episodes out reaches.
pub const AIRED_WINDOW_SECS: i64 = 2 * 86_400;

/// Fifty shows a page. Ten pages is more than any season has carried, and
/// a cap all the same: a reply that keeps saying there is more must not
/// keep the job asking.
const MAX_PAGES: u32 = 10;

// The season ------------------------------------------------------------------

/// What a pick asks AniList for, and what the page calls it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Season {
    /// The `season_shows.season` the rows sit under.
    pub key: String,
    pub label: String,
    pub status: Option<&'static str>,
    pub season: Option<&'static str>,
    pub year: Option<i32>,
}

const SEASONS: [(&str, &str); 4] = [
    ("WINTER", "Winter"),
    ("SPRING", "Spring"),
    ("SUMMER", "Summer"),
    ("FALL", "Fall"),
];

/// `Now` is everything releasing, whatever season it started in, which is
/// what "on air" means to someone watching a long runner. `Next` is the
/// season after the one this moment falls in, by AniList's quarters.
pub fn season_of(pick: SeasonPick, now: i64) -> Season {
    match pick {
        SeasonPick::Now => Season {
            key: "NOW".to_string(),
            label: "Airing now".to_string(),
            status: Some("RELEASING"),
            season: None,
            year: None,
        },
        SeasonPick::Next => {
            let date = ::time::OffsetDateTime::from_unix_timestamp(now)
                .unwrap_or(::time::OffsetDateTime::UNIX_EPOCH);
            let quarter = (u8::from(date.month()) as usize - 1) / 3;
            let (index, year) = if quarter == 3 {
                (0, date.year() + 1)
            } else {
                (quarter + 1, date.year())
            };
            let (name, label) = SEASONS[index];
            Season {
                key: format!("{name}-{year}"),
                label: format!("{label} {year}"),
                status: None,
                season: Some(name),
                year: Some(year),
            }
        }
    }
}

// The read --------------------------------------------------------------------

/// The season's shows, most popular first, leaving out every one marked
/// not interested and every one the library holds and hides. `owned` and
/// the list status are read live, so a show put on Planning or downloaded
/// reads that way on the next read with no refresh.
const SHOWS_SQL: &str = "SELECT s.anilist_id, s.mal_id, s.title_romaji, s.title_english, s.title_native,
        s.format, s.episodes, s.average_score, s.site_url, s.next_episode, s.next_at, i.path,
        (SELECT se.id FROM series se WHERE se.anilist_id = s.anilist_id AND se.missing_since IS NULL ORDER BY se.id LIMIT 1),
        COALESCE(
          (SELECT t.status FROM tracker_entries t JOIN tracker_accounts a ON a.tracker = t.tracker
            WHERE t.tracker = 'anilist' AND t.media_id = s.anilist_id AND a.connected_at IS NOT NULL),
          (SELECT t.status FROM tracker_entries t JOIN tracker_accounts a ON a.tracker = t.tracker
            WHERE t.tracker = 'mal' AND t.media_id = s.mal_id AND a.connected_at IS NOT NULL))
     FROM season_shows s
     LEFT JOIN images i ON i.url = s.cover_url
     WHERE s.season = ?1
       AND NOT EXISTS (SELECT 1 FROM not_interested n WHERE n.anilist_id = s.anilist_id)
       AND NOT EXISTS (SELECT 1 FROM series h WHERE h.anilist_id = s.anilist_id AND h.hidden = 1)
     ORDER BY s.rank";

/// Every episode out in the window, from the schedule AniList sent and from
/// each show's next episode whose moment has passed since: the strip stays
/// current between refreshes without asking again. On air means the `NOW`
/// rows whichever pick is showing.
const AIRED_SQL: &str = "SELECT anilist_id, episode, aired_at FROM (
        SELECT anilist_id, episode, aired_at FROM season_aired
        UNION
        SELECT anilist_id, next_episode, next_at FROM season_shows
         WHERE season = 'NOW' AND next_episode IS NOT NULL AND next_at IS NOT NULL
     )
     WHERE aired_at > ?1 AND aired_at <= ?2
       AND anilist_id IN (SELECT anilist_id FROM season_shows WHERE season = 'NOW')
       AND anilist_id NOT IN (SELECT anilist_id FROM not_interested)
       AND anilist_id NOT IN (SELECT anilist_id FROM series WHERE hidden = 1 AND anilist_id IS NOT NULL)
     ORDER BY aired_at DESC, anilist_id, episode DESC";

/// Titles for the strip, off the `NOW` rows.
const TITLES_SQL: &str = "SELECT title_romaji, title_english, title_native FROM season_shows
     WHERE season = 'NOW' AND anilist_id = ?1";

fn fallback_title(anilist_id: u64) -> String {
    format!("AniList {anilist_id}")
}

/// The page off the database: no request and no waiting.
pub fn list(
    conn: &Connection,
    images_dir: &Path,
    pick: SeasonPick,
    now: i64,
) -> Result<AiringNow, CoreError> {
    let season = season_of(pick, now);
    let lang = prefs::load_preferences(conn)?.title_language;

    let mut aired: Vec<AiredEpisode> = Vec::new();
    {
        let mut seen: HashSet<(u64, u32)> = HashSet::new();
        let mut stmt = conn.prepare(AIRED_SQL)?;
        let rows = stmt.query_map(params![now - AIRED_WINDOW_SECS, now], |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, i64>(1)? as u32,
                r.get::<_, i64>(2)?,
            ))
        })?;
        let mut titles_stmt = conn.prepare(TITLES_SQL)?;
        for row in rows {
            let (anilist_id, episode, at) = row?;
            if !seen.insert((anilist_id, episode)) {
                continue;
            }
            let names = titles_stmt
                .query_row(params![anilist_id as i64], |r| {
                    Ok((
                        r.get::<_, Option<String>>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, Option<String>>(2)?,
                    ))
                })
                .optional()?
                .unwrap_or_default();
            aired.push(AiredEpisode {
                anilist_id,
                title: titles::resolve(
                    lang,
                    names.0.as_deref(),
                    names.1.as_deref(),
                    names.2.as_deref(),
                    &fallback_title(anilist_id),
                ),
                episode,
                at: time::from_secs(at),
            });
        }
    }

    let mut stmt = conn.prepare(SHOWS_SQL)?;
    let rows = stmt.query_map(params![season.key], |r| {
        let anilist_id = r.get::<_, i64>(0)? as u64;
        let romaji: Option<String> = r.get(2)?;
        let english: Option<String> = r.get(3)?;
        let native: Option<String> = r.get(4)?;
        let next_episode: Option<i64> = r.get(9)?;
        let next_at: Option<i64> = r.get(10)?;
        Ok(AiringShow {
            anilist_id,
            mal_id: r.get::<_, Option<i64>>(1)?.map(|id| id as u64),
            title: titles::resolve(
                lang,
                romaji.as_deref(),
                english.as_deref(),
                native.as_deref(),
                &fallback_title(anilist_id),
            ),
            romaji,
            english,
            native,
            poster: local_path(images_dir, r.get(11)?),
            format: r.get(5)?,
            episodes: r.get::<_, Option<i64>>(6)?.map(|e| e as u32),
            average_score: r.get::<_, Option<i64>>(7)?.map(|s| s as u32),
            site_url: r.get(8)?,
            // An episode whose moment has passed is the strip's now, not
            // a countdown to something already out.
            next_airing: next_episode.zip(next_at).filter(|(_, at)| *at > now).map(
                |(episode, at)| Airing {
                    episode: episode as u32,
                    at: time::from_secs(at),
                    season: None,
                    predicted: false,
                    through: None,
                },
            ),
            latest: None,
            owned: r.get::<_, Option<i64>>(12)?.map(|id| id as u64),
            list_status: r
                .get::<_, Option<String>>(13)?
                .as_deref()
                .and_then(ListStatus::from_column),
        })
    })?;
    let mut shows = rows.collect::<Result<Vec<AiringShow>, _>>()?;
    for show in &mut shows {
        // The strip is newest first, so the first row for a show is its
        // latest.
        show.latest = aired
            .iter()
            .find(|a| a.anilist_id == show.anilist_id)
            .cloned();
    }

    let not_interested: i64 = conn.query_row(
        "SELECT COUNT(*) FROM season_shows s JOIN not_interested n ON n.anilist_id = s.anilist_id
         WHERE s.season = ?1",
        params![season.key],
        |r| r.get(0),
    )?;
    let fetched_at: Option<i64> = conn
        .query_row(
            "SELECT fetched_at FROM season_fetches WHERE season = ?1",
            params![season.key],
            |r| r.get(0),
        )
        .optional()?;

    Ok(AiringNow {
        season: pick,
        label: season.label,
        shows,
        aired,
        not_interested: not_interested as u32,
        fetched_at: fetched_at.map(time::from_secs),
    })
}

/// The `ListAiringNow` call: the cached season at once, and the id of a
/// refresh already running so the page can say one is on its way.
pub fn list_call(core: &Core, pick: SeasonPick) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let now = time::now_secs();
    let listed = core.store.read(|c| list(c, &images_dir, pick, now))?;
    Ok(Reply::AiringNow {
        now: listed,
        refreshing: running(core),
    })
}

fn running(core: &Core) -> Option<u64> {
    core.jobs
        .list()
        .into_iter()
        .find(|j| j.kind == JobKind::RefreshAiringNow)
        .map(|j| j.id)
}

/// The `RefreshAiringNow` call: the job when the season is stale (or was
/// never read) or `force` says so, `Ok` when it is fresh. One read runs at
/// a time; a second call gets the running one's id, and its event names
/// the season it read.
pub fn refresh_call(core: &Core, pick: SeasonPick, force: bool) -> Result<Reply, CoreError> {
    let now = time::now_secs();
    let key = season_of(pick, now).key;
    let fetched_at: Option<i64> = core.store.read(|c| {
        c.query_row(
            "SELECT fetched_at FROM season_fetches WHERE season = ?1",
            params![key],
            |r| r.get(0),
        )
        .optional()
        .map_err(CoreError::from)
    })?;
    let fresh = fetched_at.is_some_and(|at| now - at < STALE_AFTER_SECS);
    if fresh && !force {
        return Ok(Reply::Ok);
    }
    let owner = core.owner()?;
    let job = core
        .jobs
        .clone()
        .start(JobKind::RefreshAiringNow, move |_| async move {
            refresh(owner, pick).await
        });
    Ok(Reply::Started { job })
}

/// A cached image's absolute path, while its file is still there.
fn local_path(images_dir: &Path, relative: Option<String>) -> Option<String> {
    relative
        .map(|relative| images_dir.join(relative))
        .filter(|path| path.is_file())
        .map(|path| path.to_string_lossy().into_owned())
}

// The job ---------------------------------------------------------------------

/// One show as AniList listed it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SeasonRow {
    pub anilist_id: u64,
    pub mal_id: Option<u64>,
    pub title_romaji: Option<String>,
    pub title_english: Option<String>,
    pub title_native: Option<String>,
    pub format: Option<String>,
    pub episodes: Option<u32>,
    pub average_score: Option<u32>,
    pub cover_url: Option<String>,
    pub site_url: Option<String>,
    pub next: Option<(u32, i64)>,
}

async fn refresh(core: Arc<Core>, pick: SeasonPick) -> Result<Finished, CoreError> {
    let now = time::now_secs();
    let season = season_of(pick, now);
    let shows = fetch_shows(&core, &season).await?;
    // The strip belongs to what is on air, so only that read asks for it.
    let aired = if pick == SeasonPick::Now {
        let ids: HashSet<u64> = shows.iter().map(|s| s.anilist_id).collect();
        fetch_aired(&core, now)
            .await?
            .into_iter()
            .filter(|(id, _, _)| ids.contains(id))
            .collect()
    } else {
        Vec::new()
    };

    let covers: Vec<String> = shows.iter().filter_map(|s| s.cover_url.clone()).collect();
    let count = shows.len();
    let next_key = season_of(SeasonPick::Next, now).key;
    let key = season.key.clone();
    core.store
        .tx_async(move |tx| write(tx, &key, &next_key, &shows, &aired, now))
        .await?;
    // Every cover before the event, so the cards draw local files.
    for (url, outcome) in core.images.ensure(&covers).await {
        if let Err(e) = outcome {
            tracing::debug!("an Airing now cover {url} was not fetched: {e}");
        }
    }
    Ok(Finished {
        level: Level::Debug,
        message: format!("{} refreshed: {count} shows", season.label),
        body: EventBody::AiringNowRefreshed { season: pick },
    })
}

/// Every page of the season, most popular first, each show once: AniList
/// can hand a show back on two pages when the order shifts mid-read.
async fn fetch_shows(core: &Arc<Core>, season: &Season) -> Result<Vec<SeasonRow>, CoreError> {
    let mut seen: HashSet<u64> = HashSet::new();
    let mut rows: Vec<SeasonRow> = Vec::new();
    for page in 1..=MAX_PAGES {
        let data = core
            .anilist
            .graphql(SEASON_PAGE_QUERY, season_variables(season, page), None)
            .await?;
        for row in parse_shows(&data) {
            if seen.insert(row.anilist_id) {
                rows.push(row);
            }
        }
        if !has_next(&data) {
            break;
        }
    }
    Ok(rows)
}

/// The episodes out in the window, newest first.
async fn fetch_aired(core: &Arc<Core>, now: i64) -> Result<Vec<(u64, u32, i64)>, CoreError> {
    let mut out: Vec<(u64, u32, i64)> = Vec::new();
    for page in 1..=MAX_PAGES {
        let data = core
            .anilist
            .graphql(
                AIRED_PAGE_QUERY,
                serde_json::json!({
                    "page": page,
                    "from": now - AIRED_WINDOW_SECS,
                    "to": now + 1,
                }),
                None,
            )
            .await?;
        out.extend(parse_aired(&data));
        if !has_next(&data) {
            break;
        }
    }
    Ok(out)
}

/// The query's variables, with every filter the pick does not use left out
/// rather than sent as null: AniList reads `season: null` as "has no
/// season", which is every Chinese web series and nothing else.
pub fn season_variables(season: &Season, page: u32) -> serde_json::Value {
    let mut vars = serde_json::Map::new();
    vars.insert("page".to_string(), page.into());
    if let Some(status) = season.status {
        vars.insert("status".to_string(), status.into());
    }
    if let Some(name) = season.season {
        vars.insert("season".to_string(), name.into());
    }
    if let Some(year) = season.year {
        vars.insert("seasonYear".to_string(), year.into());
    }
    serde_json::Value::Object(vars)
}

fn has_next(data: &serde_json::Value) -> bool {
    data["Page"]["pageInfo"]["hasNextPage"]
        .as_bool()
        .unwrap_or(false)
}

/// One page of `SEASON_PAGE_QUERY`. A show with no id is skipped; anything
/// else missing is left blank.
pub fn parse_shows(data: &serde_json::Value) -> Vec<SeasonRow> {
    data["Page"]["media"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| {
            let anilist_id = m["id"].as_u64()?;
            let next = &m["nextAiringEpisode"];
            Some(SeasonRow {
                anilist_id,
                mal_id: m["idMal"].as_u64(),
                title_romaji: text(&m["title"]["romaji"]),
                title_english: text(&m["title"]["english"]),
                title_native: text(&m["title"]["native"]),
                format: text(&m["format"]),
                episodes: as_u32(m["episodes"].as_u64()),
                average_score: as_u32(m["averageScore"].as_u64()),
                cover_url: text(&m["coverImage"]["large"]),
                site_url: text(&m["siteUrl"]),
                next: as_u32(next["episode"].as_u64()).zip(next["airingAt"].as_i64()),
            })
        })
        .collect()
}

/// One page of `AIRED_PAGE_QUERY`, as (show, episode, moment).
pub fn parse_aired(data: &serde_json::Value) -> Vec<(u64, u32, i64)> {
    data["Page"]["airingSchedules"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|a| {
            Some((
                a["mediaId"].as_u64()?,
                as_u32(a["episode"].as_u64())?,
                a["airingAt"].as_i64()?,
            ))
        })
        .collect()
}

/// A string off the reply, an empty one read as absent.
fn text(value: &serde_json::Value) -> Option<String> {
    value
        .as_str()
        .filter(|t| !t.trim().is_empty())
        .map(str::to_string)
}

/// The season replaced whole, in one transaction: a show AniList stopped
/// listing goes with the old rows. Seasons that are neither on air nor next
/// go too, and the strip keeps only its window.
fn write(
    tx: &Transaction,
    key: &str,
    next_key: &str,
    shows: &[SeasonRow],
    aired: &[(u64, u32, i64)],
    now: i64,
) -> Result<(), CoreError> {
    tx.execute("DELETE FROM season_shows WHERE season = ?1", params![key])?;
    tx.execute(
        "DELETE FROM season_shows WHERE season NOT IN ('NOW', ?1)",
        params![next_key],
    )?;
    tx.execute(
        "DELETE FROM season_fetches WHERE season NOT IN ('NOW', ?1)",
        params![next_key],
    )?;
    let mut insert = tx.prepare(
        "INSERT INTO season_shows (season, anilist_id, rank, mal_id, title_romaji, title_english, title_native,
            format, episodes, average_score, cover_url, site_url, next_episode, next_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
    )?;
    for (rank, s) in shows.iter().enumerate() {
        insert.execute(params![
            key,
            s.anilist_id as i64,
            rank as i64,
            s.mal_id.map(|id| id as i64),
            s.title_romaji,
            s.title_english,
            s.title_native,
            s.format,
            s.episodes.map(i64::from),
            s.average_score.map(i64::from),
            s.cover_url,
            s.site_url,
            s.next.map(|(e, _)| i64::from(e)),
            s.next.map(|(_, at)| at),
        ])?;
    }
    tx.execute(
        "DELETE FROM season_aired WHERE aired_at <= ?1",
        params![now - AIRED_WINDOW_SECS],
    )?;
    let mut aired_insert = tx.prepare(
        "INSERT INTO season_aired (anilist_id, episode, aired_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(anilist_id, episode) DO UPDATE SET aired_at = excluded.aired_at",
    )?;
    for (id, episode, at) in aired {
        aired_insert.execute(params![*id as i64, i64::from(*episode), at])?;
    }
    tx.execute(
        "INSERT INTO season_fetches (season, fetched_at) VALUES (?1, ?2)
         ON CONFLICT(season) DO UPDATE SET fetched_at = excluded.fetched_at",
        params![key, now],
    )?;
    Ok(())
}

// Not interested --------------------------------------------------------------

/// The `SetNotInterested` call. Marking keeps the show's title and cover as
/// they read now, from a season row or else the library's media row, so
/// Settings can name it after every season has moved on. Marking twice
/// keeps the first moment; taking the mark back is a plain delete.
pub fn set_not_interested(core: &Core, anilist_id: u64, on: bool) -> Result<Reply, CoreError> {
    let now = time::now_secs();
    core.store.write(move |c| {
        if !on {
            c.execute(
                "DELETE FROM not_interested WHERE anilist_id = ?1",
                params![anilist_id as i64],
            )?;
            return Ok(());
        }
        let lang = prefs::load_preferences(c)?.title_language;
        let found = c
            .query_row(
                "SELECT title_romaji, title_english, title_native, cover_url FROM season_shows
                  WHERE anilist_id = ?1
                 UNION ALL
                 SELECT title_romaji, title_english, title_native, cover_url FROM anilist_media
                  WHERE id = ?1
                 LIMIT 1",
                params![anilist_id as i64],
                |r| {
                    Ok((
                        r.get::<_, Option<String>>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .optional()?
            .unwrap_or_default();
        let title = titles::resolve(
            lang,
            found.0.as_deref(),
            found.1.as_deref(),
            found.2.as_deref(),
            &fallback_title(anilist_id),
        );
        c.execute(
            "INSERT INTO not_interested (anilist_id, title, cover_url, marked_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(anilist_id) DO NOTHING",
            params![anilist_id as i64, title, found.3, now],
        )?;
        Ok(())
    })?;
    Ok(Reply::Ok)
}

/// The `ListNotInterested` call: newest mark first.
pub fn list_not_interested(core: &Core) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let shows = core.store.read(|c| {
        let mut stmt = c.prepare(
            "SELECT n.anilist_id, n.title, i.path, n.marked_at FROM not_interested n
             LEFT JOIN images i ON i.url = n.cover_url
             ORDER BY n.marked_at DESC, n.anilist_id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(NotInterestedShow {
                anilist_id: r.get::<_, i64>(0)? as u64,
                title: r.get(1)?,
                poster: local_path(&images_dir, r.get(2)?),
                marked_at: time::from_secs(r.get(3)?),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })?;
    Ok(Reply::NotInterested { shows })
}

// Plan to watch ---------------------------------------------------------------

/// The `PlanToWatch` call: Planning on every connected tracker that does
/// not list the show yet. A show already on a list keeps its status there,
/// whatever it is; this never moves an entry. `Ok` when every connected
/// tracker lists it already, `NotConnected` when none is connected.
pub fn plan_to_watch(core: &Core, anilist_id: u64) -> Result<Reply, CoreError> {
    let (targets, connected, title) = core.store.read(|c| {
        let lang = prefs::load_preferences(c)?.title_language;
        let found = c
            .query_row(
                "SELECT mal_id, title_romaji, title_english, title_native FROM season_shows
                  WHERE anilist_id = ?1
                 UNION ALL
                 SELECT mal_id, title_romaji, title_english, title_native FROM anilist_media
                  WHERE id = ?1
                 LIMIT 1",
                params![anilist_id as i64],
                |r| {
                    Ok((
                        r.get::<_, Option<i64>>(0)?.map(|id| id as u64),
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .optional()?
            .unwrap_or_default();
        let title = titles::resolve(
            lang,
            found.1.as_deref(),
            found.2.as_deref(),
            found.3.as_deref(),
            &fallback_title(anilist_id),
        );
        let mut targets: Vec<(Tracker, u64, Option<u64>)> = Vec::new();
        let mut connected = false;
        for (t, id) in [
            (Tracker::Anilist, Some(anilist_id)),
            (Tracker::Mal, found.0),
        ] {
            let Some(row) = accounts::load_row(c, t)? else {
                continue;
            };
            if row.connected_at.is_none() {
                continue;
            }
            connected = true;
            let Some(id) = id else { continue };
            let listed: bool = c.query_row(
                "SELECT EXISTS (SELECT 1 FROM tracker_entries WHERE tracker = ?1 AND media_id = ?2)",
                params![t.as_str(), id as i64],
                |r| r.get(0),
            )?;
            if !listed {
                targets.push((t, id, row.user_id));
            }
        }
        Ok((targets, connected, title))
    })?;
    if !connected {
        return Err(CoreError::NotConnected {
            tracker: Tracker::Anilist,
        });
    }
    if targets.is_empty() {
        return Ok(Reply::Ok);
    }
    let owner = core.owner()?;
    let job = core
        .jobs
        .clone()
        .start(JobKind::SetStatus, move |ctx| async move {
            plan(owner, ctx, title, targets).await
        });
    Ok(Reply::Started { job })
}

/// The SetStatus job behind `PlanToWatch`. Each tracker is read live first:
/// an entry made somewhere else since the list was read is left as it is.
async fn plan(
    core: Arc<Core>,
    ctx: Arc<JobCtx>,
    title: String,
    targets: Vec<(Tracker, u64, Option<u64>)>,
) -> Result<Finished, CoreError> {
    let mut planned: Vec<Tracker> = Vec::new();
    let mut failure: Option<String> = None;
    for (t, media_id, user_id) in targets {
        ctx.checkpoint()?;
        let attempt = async {
            let token = accounts::access_token(&core, t)
                .await?
                .ok_or_else(|| accounts::tracker_error(t, NO_TOKEN))?;
            let live = writes::read_live(&core, t, media_id, user_id, &token).await?;
            if live.is_some_and(|l| l.status.is_some()) {
                return Ok::<bool, CoreError>(false);
            }
            let stamp =
                writes::send_status(&core, t, media_id, &token, ListStatus::Planning).await?;
            let now = time::now_secs();
            core.store
                .write_async(move |c| record_planned(c, t, media_id, stamp, now))
                .await?;
            Ok(true)
        };
        match attempt.await {
            Ok(true) => planned.push(t),
            Ok(false) => {}
            Err(e) => {
                failure.get_or_insert_with(|| writes::sanitize_error(t, &e));
            }
        }
    }
    Ok(match (planned.is_empty(), failure) {
        (false, _) => Finished {
            level: Level::Info,
            message: format!(
                "{title} is on Planning on {}",
                planned
                    .iter()
                    .map(|t| t.label())
                    .collect::<Vec<_>>()
                    .join(" and ")
            ),
            body: EventBody::Notice,
        },
        (true, Some(why)) => Finished {
            level: Level::Warn,
            message: format!("{title} could not go on Planning: {why}"),
            body: EventBody::Notice,
        },
        (true, None) => Finished {
            level: Level::Debug,
            message: format!("{title} was already on a list"),
            body: EventBody::Notice,
        },
    })
}

/// The new entry in the cache, so the card reads Planning at once.
fn record_planned(
    conn: &Connection,
    t: Tracker,
    media_id: u64,
    stamp: Option<i64>,
    now: i64,
) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO tracker_entries (tracker, media_id, status, progress, score, repeat, updated_at, fetched_at)
         VALUES (?1, ?2, ?3, 0, NULL, 0, ?4, ?5)
         ON CONFLICT(tracker, media_id) DO UPDATE SET
            status = excluded.status,
            updated_at = COALESCE(excluded.updated_at, tracker_entries.updated_at),
            fetched_at = excluded.fetched_at",
        params![
            t.as_str(),
            media_id as i64,
            ListStatus::Planning.as_str(),
            stamp,
            now
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-28 and 2026-05-01, UTC.
    const OCTOBER: i64 = 1_793_145_600;
    const MAY: i64 = 1_777_593_600;

    #[test]
    fn now_is_everything_releasing_whatever_its_season() {
        let s = season_of(SeasonPick::Now, OCTOBER);
        assert_eq!(s.key, "NOW");
        assert_eq!(s.status, Some("RELEASING"));
        assert_eq!((s.season, s.year), (None, None));
    }

    #[test]
    fn next_season_crosses_the_year_after_fall() {
        let s = season_of(SeasonPick::Next, OCTOBER);
        assert_eq!(s.key, "WINTER-2027");
        assert_eq!(s.label, "Winter 2027");
        assert_eq!(
            (s.season, s.year, s.status),
            (Some("WINTER"), Some(2027), None)
        );
        let s = season_of(SeasonPick::Next, MAY);
        assert_eq!(s.key, "SUMMER-2026");
    }

    #[test]
    fn a_filter_the_pick_does_not_use_is_never_sent_as_null() {
        let now = season_variables(&season_of(SeasonPick::Now, OCTOBER), 2);
        assert_eq!(now, serde_json::json!({ "page": 2, "status": "RELEASING" }));
        let next = season_variables(&season_of(SeasonPick::Next, OCTOBER), 1);
        assert_eq!(
            next,
            serde_json::json!({ "page": 1, "season": "WINTER", "seasonYear": 2027 })
        );
    }

    #[test]
    fn a_show_with_no_id_is_skipped_and_blanks_stay_blank() {
        let data = serde_json::json!({ "Page": { "pageInfo": { "hasNextPage": true }, "media": [
            { "id": 1, "idMal": 2, "title": { "romaji": "A", "english": "", "native": null },
              "coverImage": { "large": "https://img/a.jpg" }, "episodes": null,
              "nextAiringEpisode": { "episode": 7, "airingAt": 100 } },
            { "idMal": 3, "title": { "romaji": "No id" } },
        ] } });
        let rows = parse_shows(&data);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title_romaji.as_deref(), Some("A"));
        assert_eq!(rows[0].title_english, None);
        assert_eq!(rows[0].next, Some((7, 100)));
        assert!(has_next(&data));
    }

    #[test]
    fn an_aired_row_missing_a_field_is_skipped() {
        let data = serde_json::json!({ "Page": { "airingSchedules": [
            { "mediaId": 1, "episode": 5, "airingAt": 90 },
            { "mediaId": 2, "airingAt": 80 },
        ] } });
        assert_eq!(parse_aired(&data), vec![(1, 5, 90)]);
        assert!(!has_next(&data));
    }
}
