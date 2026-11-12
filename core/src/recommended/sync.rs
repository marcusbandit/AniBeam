//! The recommender's reads from AniList, each from the cache first. A first
//! run sends some seventy requests; after that a run sends only what is new
//! or past its time to live, a handful at most.
//!
//! Every request goes through the shared limiter like any other AniList
//! call, and on top of it this read keeps its own pace, so a run stays under
//! AniList's limit on its own instead of leaning on 429s (#732). A rate
//! limit the limiter could not ride out, an AniList that does not answer, or
//! the run's budget spent stops the reading; the fit then runs on what the
//! cache holds.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use anibeam_recs::neighbors::Neighborhood;
use anibeam_recs::signal::Signal;
use anibeam_recs::{Catalog, Neighbor, ShowId};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use super::{mine::Mine, parse, queries};
use crate::contract::*;
use crate::core::Core;
use crate::jobs::JobCtx;
use crate::metadata::apply::{is_rate_limited, provider_unreachable};
use crate::time;

const DAY: i64 = 86_400;
/// How long a show's details are good for: staff and tags hardly move.
pub const SHOW_TTL: i64 = 30 * DAY;
/// How long a user's list, a show's raters and its "if you liked this" are
/// good for.
pub const PEOPLE_TTL: i64 = 14 * DAY;

const SHOWS_PER_REQUEST: usize = 25;
const RATERS_PER_REQUEST: usize = 10;
const USERS_PER_REQUEST: usize = 8;
const SIMILAR_PER_REQUEST: usize = 10;

/// The user's best-rated shows whose finishers are read for neighbours.
pub const FAVOURITES: usize = 20;
/// Other users' lists read.
pub const PEOPLE: usize = 120;
/// Candidate shows beyond the list and the library.
pub const POOL: usize = 400;
/// The most requests one run sends, cold start included.
pub const BUDGET: u32 = 150;

/// Why the reading stopped short.
pub enum Halt {
    /// AniList said stop, did not answer, or the budget is spent. The fit
    /// runs on the cache.
    Stop(String),
    /// Anything else, cancellation included, which ends the job.
    Fail(CoreError),
}

impl From<CoreError> for Halt {
    fn from(e: CoreError) -> Halt {
        if is_rate_limited(&e) || provider_unreachable(&e) {
            Halt::Stop(e.to_string())
        } else {
            Halt::Fail(e)
        }
    }
}

/// The paced sender. `sent` is what the run's log line counts.
pub struct Net {
    core: Arc<Core>,
    ctx: Arc<JobCtx>,
    pub sent: u32,
    last: Option<Instant>,
}

impl Net {
    pub fn new(core: Arc<Core>, ctx: Arc<JobCtx>) -> Net {
        Net {
            core,
            ctx,
            sent: 0,
            last: None,
        }
    }

    async fn send(&mut self, query: &str, variables: Value) -> Result<Value, Halt> {
        if self.sent >= BUDGET {
            return Err(Halt::Stop(format!("{BUDGET} requests sent")));
        }
        let gap = Duration::from_millis(self.core.recommender_gap_ms.load(Ordering::SeqCst));
        if let Some(last) = self.last
            && let Some(wait) = gap.checked_sub(last.elapsed())
        {
            tokio::time::sleep(wait).await;
        }
        self.ctx.checkpoint()?;
        self.sent += 1;
        self.last = Some(Instant::now());
        Ok(self.core.anilist.graphql(query, variables, None).await?)
    }

    fn label(&self, done: u64, total: u64, what: &str) {
        self.ctx.progress(done, Some(total), what);
    }
}

/// An AniList reply refusing the query itself, such as one too complex to
/// run, rather than AniList being away or saying stop: halving the batch is
/// worth a try.
fn refused(halt: &Halt) -> bool {
    matches!(halt, Halt::Fail(CoreError::Provider { status, .. }) if *status != Some(429))
}

// The cache -------------------------------------------------------------------

fn stale(conn: &Connection, sql: &str, id: u64, ttl: i64, now: i64) -> Result<bool, CoreError> {
    let at: Option<i64> = conn
        .query_row(sql, params![id as i64], |r| r.get(0))
        .optional()?;
    Ok(at.is_none_or(|at| now - at >= ttl))
}

fn missing(core: &Core, sql: &'static str, ids: &[u64], ttl: i64) -> Result<Vec<u64>, CoreError> {
    let now = time::now_secs();
    let ids = ids.to_vec();
    core.store.read(move |c| {
        let mut out = Vec::new();
        for id in ids {
            if stale(c, sql, id, ttl, now)? {
                out.push(id);
            }
        }
        Ok(out)
    })
}

/// Every show the cache holds that AniList still has, read for the models.
pub fn catalog(conn: &Connection) -> Result<Catalog, CoreError> {
    let mut stmt = conn.prepare("SELECT media FROM rec_shows WHERE media IS NOT NULL")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut out = Catalog::new();
    for row in rows {
        if let Some(show) = serde_json::from_str::<Value>(&row?)
            .ok()
            .as_ref()
            .and_then(parse::show)
        {
            out.insert(show.id, show);
        }
    }
    Ok(out)
}

/// The finishers of each favourite and their score of it, out of 100.
fn raters(
    conn: &Connection,
    favourites: &[u64],
) -> Result<HashMap<u64, Vec<(u64, f32)>>, CoreError> {
    let mut out = HashMap::new();
    for id in favourites {
        let text: Option<String> = conn
            .query_row(
                "SELECT raters FROM rec_raters WHERE anilist_id = ?1",
                params![*id as i64],
                |r| r.get(0),
            )
            .optional()?;
        let rows: Vec<(u64, f32)> = text
            .and_then(|t| serde_json::from_str::<Vec<(u64, f32)>>(&t).ok())
            .unwrap_or_default();
        out.insert(*id, rows);
    }
    Ok(out)
}

/// The readable lists among `ids`.
pub fn neighbours(conn: &Connection, ids: &[u64]) -> Result<Vec<Neighbor>, CoreError> {
    let mut out = Vec::new();
    for id in ids {
        let text: Option<Option<String>> = conn
            .query_row(
                "SELECT entries FROM rec_users WHERE user_id = ?1",
                params![*id as i64],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(Some(text)) = text {
            out.push(Neighbor {
                user: *id,
                entries: parse::unpack(&text),
            });
        }
    }
    Ok(out)
}

fn similar(conn: &Connection, ids: &[u64]) -> Result<Vec<u64>, CoreError> {
    let mut out: Vec<u64> = Vec::new();
    for id in ids {
        let text: Option<String> = conn
            .query_row(
                "SELECT similar FROM rec_similar WHERE anilist_id = ?1",
                params![*id as i64],
                |r| r.get(0),
            )
            .optional()?;
        for x in text.as_deref().map(parse::ids).unwrap_or_default() {
            if !out.contains(&x) {
                out.push(x);
            }
        }
    }
    Ok(out)
}

// The reads ---------------------------------------------------------------------

/// Details for every show in `ids` the cache lacks or holds past its time.
pub async fn shows(net: &mut Net, ids: &[u64], label: &str) -> Result<(), Halt> {
    let todo = missing(
        &net.core,
        "SELECT fetched_at FROM rec_shows WHERE anilist_id = ?1",
        ids,
        SHOW_TTL,
    )?;
    let total = todo.len().div_ceil(SHOWS_PER_REQUEST) as u64;
    for (i, chunk) in todo.chunks(SHOWS_PER_REQUEST).enumerate() {
        net.label(i as u64, total, label);
        fetch_shows(net, chunk).await?;
    }
    Ok(())
}

async fn fetch_shows(net: &mut Net, chunk: &[u64]) -> Result<(), Halt> {
    let reply = net
        .send(&queries::shows(chunk.len()), json!({ "ids": chunk }))
        .await;
    let v = match reply {
        // A batch AniList will not run is halved and tried again; one show
        // it will not run is left for the next run.
        Err(halt) if refused(&halt) && chunk.len() > 1 => {
            let (a, b) = chunk.split_at(chunk.len() / 2);
            Box::pin(fetch_shows(net, a)).await?;
            return Box::pin(fetch_shows(net, b)).await;
        }
        Err(halt) if refused(&halt) => return Ok(()),
        other => other?,
    };
    let media: Vec<Value> = v["Page"]["media"].as_array().cloned().unwrap_or_default();
    let chunk = chunk.to_vec();
    let now = time::now_secs();
    net.core
        .store
        .write_async(move |c| {
            let mut got = HashSet::new();
            for m in &media {
                if let Some(id) = m["id"].as_u64() {
                    got.insert(id);
                    put_show(c, id, Some(&m.to_string()), now)?;
                }
            }
            // Gone from AniList: remembered, so it is not asked for again soon.
            for id in chunk.iter().filter(|id| !got.contains(id)) {
                put_show(c, *id, None, now)?;
            }
            Ok(())
        })
        .await?;
    Ok(())
}

pub fn put_show(c: &Connection, id: u64, media: Option<&str>, now: i64) -> Result<(), CoreError> {
    // A show AniList no longer lists keeps the copy it had: a page drawn
    // from it beats a blank one.
    c.execute(
        "INSERT INTO rec_shows (anilist_id, media, fetched_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(anilist_id) DO UPDATE SET media = COALESCE(excluded.media, rec_shows.media),
            fetched_at = excluded.fetched_at",
        params![id as i64, media, now],
    )?;
    Ok(())
}

async fn fetch_raters(net: &mut Net, favourites: &[u64]) -> Result<(), Halt> {
    let todo = missing(
        &net.core,
        "SELECT fetched_at FROM rec_raters WHERE anilist_id = ?1",
        favourites,
        PEOPLE_TTL,
    )?;
    let total = todo.len().div_ceil(RATERS_PER_REQUEST) as u64;
    for (i, chunk) in todo.chunks(RATERS_PER_REQUEST).enumerate() {
        net.label(i as u64, total, "Finding people with your taste");
        let v = net.send(&queries::raters(chunk), Value::Null).await?;
        let chunk = chunk.to_vec();
        let now = time::now_secs();
        net.core
            .store
            .write_async(move |c| {
                for id in chunk {
                    let rows: Vec<(u64, f32)> = v[format!("m{id}")]["mediaList"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|r| {
                            Some((
                                r["userId"].as_u64()?,
                                r["score"].as_f64().unwrap_or(0.0) as f32,
                            ))
                        })
                        .collect();
                    c.execute(
                    "INSERT INTO rec_raters (anilist_id, raters, fetched_at) VALUES (?1, ?2, ?3)
                     ON CONFLICT(anilist_id) DO UPDATE SET raters = excluded.raters,
                        fetched_at = excluded.fetched_at",
                    params![id as i64, json!(rows).to_string(), now],
                )?;
                }
                Ok(())
            })
            .await?;
    }
    Ok(())
}

async fn fetch_users(net: &mut Net, ids: &[u64]) -> Result<(), Halt> {
    let todo = missing(
        &net.core,
        "SELECT fetched_at FROM rec_users WHERE user_id = ?1",
        ids,
        PEOPLE_TTL,
    )?;
    let total = todo.len().div_ceil(USERS_PER_REQUEST) as u64;
    for (i, chunk) in todo.chunks(USERS_PER_REQUEST).enumerate() {
        net.label(i as u64, total, "Reading their lists");
        fetch_user_chunk(net, chunk).await?;
    }
    Ok(())
}

async fn fetch_user_chunk(net: &mut Net, chunk: &[u64]) -> Result<(), Halt> {
    let v = match net.send(&queries::user_lists(chunk), Value::Null).await {
        // One account AniList will not show sinks the whole batch, so the
        // batch is halved until that one stands alone, and it reads as
        // private.
        Err(halt) if refused(&halt) && chunk.len() > 1 => {
            let (a, b) = chunk.split_at(chunk.len() / 2);
            Box::pin(fetch_user_chunk(net, a)).await?;
            return Box::pin(fetch_user_chunk(net, b)).await;
        }
        Err(halt) if refused(&halt) => json!({}),
        other => other?,
    };
    let chunk = chunk.to_vec();
    let now = time::now_secs();
    net.core
        .store
        .write_async(move |c| {
            for id in chunk {
                let list = &v[format!("u{id}")];
                let entries = (!list.is_null()).then(|| parse::pack(&parse::collection(list)));
                c.execute(
                    "INSERT INTO rec_users (user_id, entries, fetched_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(user_id) DO UPDATE SET entries = excluded.entries,
                    fetched_at = excluded.fetched_at",
                    params![id as i64, entries, now],
                )?;
            }
            Ok(())
        })
        .await?;
    Ok(())
}

async fn fetch_similar(net: &mut Net, ids: &[u64]) -> Result<(), Halt> {
    let todo = missing(
        &net.core,
        "SELECT fetched_at FROM rec_similar WHERE anilist_id = ?1",
        ids,
        PEOPLE_TTL,
    )?;
    for chunk in todo.chunks(SIMILAR_PER_REQUEST) {
        let v = net.send(&queries::similar(chunk), Value::Null).await?;
        let chunk = chunk.to_vec();
        let now = time::now_secs();
        net.core
            .store
            .write_async(move |c| {
                for id in chunk {
                    let found: Vec<u64> = v[format!("m{id}")]["recommendations"]["nodes"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|n| n["rating"].as_i64().unwrap_or(0) > 0)
                        .filter(|n| n["mediaRecommendation"]["type"].as_str() == Some("ANIME"))
                        .filter_map(|n| n["mediaRecommendation"]["id"].as_u64())
                        .collect();
                    c.execute(
                    "INSERT INTO rec_similar (anilist_id, similar, fetched_at) VALUES (?1, ?2, ?3)
                     ON CONFLICT(anilist_id) DO UPDATE SET similar = excluded.similar,
                        fetched_at = excluded.fetched_at",
                    params![id as i64, json!(found).to_string(), now],
                )?;
                }
                Ok(())
            })
            .await?;
    }
    Ok(())
}

// The run -----------------------------------------------------------------------

/// What a run read beyond the list and the library, saved for the fit and
/// for the next run.
#[derive(Clone, Debug, Default)]
pub struct Found {
    pub people: Vec<u64>,
    pub pool: Vec<u64>,
    pub trusted: Vec<u64>,
}

/// The user's favourites: their best-rated shows the cache has details for.
fn favourites(signals: &[Signal], known: &HashSet<ShowId>) -> Vec<u64> {
    let mut loved: Vec<&Signal> = signals
        .iter()
        .filter(|s| !s.dropped && known.contains(&s.show))
        .collect();
    loved.sort_by(|a, b| b.value.total_cmp(&a.value).then(a.show.cmp(&b.show)));
    loved.iter().take(FAVOURITES).map(|s| s.show).collect()
}

/// The people whose lists are read: the users who finished the most of the
/// user's favourites, the higher their scores of them the better.
fn people(raters: &HashMap<u64, Vec<(u64, f32)>>, me: Option<u64>) -> Vec<u64> {
    let mut count: HashMap<u64, (u32, f32)> = HashMap::new();
    for rows in raters.values() {
        for (user, score) in rows {
            if Some(*user) != me && *score > 0.0 {
                let c = count.entry(*user).or_default();
                c.0 += 1;
                c.1 += score;
            }
        }
    }
    let mut picked: Vec<(u64, (u32, f32))> = count.into_iter().collect();
    picked.sort_by(|a, b| {
        b.1.0
            .cmp(&a.1.0)
            .then(b.1.1.total_cmp(&a.1.1))
            .then(a.0.cmp(&b.0))
    });
    picked.into_iter().take(PEOPLE).map(|(u, _)| u).collect()
}

/// The candidates: half what many of those people saw, which leans
/// popular, and half what they loved most however few saw it, which is
/// where the less known shows come from.
fn pool(hood: &Neighborhood, on_list: &HashSet<ShowId>) -> Vec<u64> {
    let mut seen: Vec<(ShowId, u32)> = hood
        .rated()
        .into_iter()
        .filter(|(id, n)| *n >= 3 && !on_list.contains(id))
        .collect();
    seen.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut loved: Vec<(ShowId, f32)> = seen
        .iter()
        .filter_map(|(id, _)| Some((*id, hood.predict(*id)?.love)))
        .collect();
    loved.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut out: Vec<u64> = seen.iter().take(POOL / 2).map(|(id, _)| *id).collect();
    let mut taken: HashSet<u64> = out.iter().copied().collect();
    for (id, _) in loved {
        if out.len() >= POOL {
            break;
        }
        if taken.insert(id) {
            out.push(id);
        }
    }
    out
}

/// The whole read: the user's and the library's shows, the people who
/// finished the user's favourites, their lists, and the candidates those
/// lists and AniList's "if you liked this" point at.
pub async fn read(
    net: &mut Net,
    mine: &Mine,
    signals: &[Signal],
    mean: f32,
) -> Result<Found, Halt> {
    let mut theirs: Vec<u64> = mine.entries.iter().map(|e| e.show).collect();
    theirs.extend(super::mine::owned_ids(mine));
    theirs.sort_unstable();
    theirs.dedup();
    shows(net, &theirs, "Reading your shows").await?;

    let core = net.core.clone();
    let known: HashSet<ShowId> = core.store.read(|c| {
        let mut stmt = c.prepare("SELECT anilist_id FROM rec_shows WHERE media IS NOT NULL")?;
        let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
        Ok(rows.filter_map(Result::ok).map(|id| id as u64).collect())
    })?;
    let favs = favourites(signals, &known);
    fetch_raters(net, &favs).await?;
    let raters = core.store.read(|c| raters(c, &favs))?;
    let people = people(&raters, mine.user);
    fetch_users(net, &people).await?;
    let neighbours = core.store.read(|c| neighbours(c, &people))?;

    let hood = Neighborhood::build(signals, mean, &neighbours, anibeam_recs::model::PEERS);
    let on_list: HashSet<ShowId> = mine.entries.iter().map(|e| e.show).collect();
    let pool = pool(&hood, &on_list);
    fetch_similar(net, &favs).await?;
    let trusted = core.store.read(|c| similar(c, &favs))?;
    let mut wanted: Vec<u64> = pool.iter().chain(&trusted).copied().collect();
    wanted.sort_unstable();
    wanted.dedup();
    shows(net, &wanted, "Reading shows to suggest").await?;
    Ok(Found {
        people,
        pool,
        trusted,
    })
}

/// What the last run found, for a run that reads nothing.
pub fn saved(conn: &Connection) -> Result<Found, CoreError> {
    let row: Option<(String, String, String)> = conn
        .query_row(
            "SELECT people, pool, trusted FROM rec_state WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    Ok(row
        .map(|(people, pool, trusted)| Found {
            people: parse::ids(&people),
            pool: parse::ids(&pool),
            trusted: parse::ids(&trusted),
        })
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn people_are_who_finished_most_favourites_and_never_the_user() {
        let mut raters = HashMap::new();
        raters.insert(1, vec![(10, 90.0), (11, 80.0), (99, 100.0), (12, 0.0)]);
        raters.insert(2, vec![(11, 70.0), (10, 95.0), (99, 100.0)]);
        raters.insert(3, vec![(11, 60.0)]);
        // 11 finished all three; 10 two; 12 left no score; 99 is the user.
        assert_eq!(people(&raters, Some(99)), vec![11, 10]);
    }
}
