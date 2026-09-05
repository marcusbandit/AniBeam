//! The airing refresh: the cheapest fetch the core makes, and the only
//! one that runs on its own schedule.
//!
//! A releasing series' next broadcast moves. A full refresh would answer
//! that with four provider calls and a rewrite of the whole record; this
//! job asks AniList for the schedule alone, and Jikan for the titles when
//! there is a MAL id to ask about. That makes it cheap enough to run over
//! the library at launch, and it is why nothing it writes may replace a
//! title: the schedule carries none at all, so a title already in the
//! table is the best one there will ever be. `write_episodes`'
//! `keep_titles` is that rule.
//!
//! Jikan's failures are reported through `Core::report_jikan_outage`,
//! which warns at most once every `metadata::OUTAGE_WINDOW`. The window
//! lives beside the module rather than here because the full fetch shares
//! it: a launch sweep through an outage must write one line, not one line
//! per series.
//!
//! Electron's `airingRefreshCandidate`, `refreshAiringForSeries` and
//! `AIRING_REFRESH_TTL_MS`, carried over.

use std::sync::Arc;
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::metadata::apply::{card_for, is_rate_limited, owner};
use crate::metadata::fetch::message_of;
use crate::metadata::record;
use crate::time;

/// How long a series' schedule is taken to be current. Electron's
/// `AIRING_REFRESH_TTL_MS`: a broadcast slot does not move by the hour,
/// and a library of releasing series would otherwise cost a request each
/// every time a page is opened.
pub const TTL: Duration = Duration::from_secs(6 * 60 * 60);

/// One series worth asking about. `mal_id` decides whether Jikan is asked
/// at all; `folder_name` is what the log lines name it by.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub series: u64,
    pub anilist_id: u64,
    pub mal_id: Option<u64>,
    pub folder_name: String,
}

/// The join a candidate is decided from. The `anilist_media` join is what
/// makes an unmatched series and a MAL-only match disappear from it: a
/// NULL `anilist_id` matches no row.
const ELIGIBILITY_SQL: &str = "SELECT s.id, s.anilist_id, s.mal_id, s.folder_name, m.status, m.airing_refreshed_at,
       EXISTS (SELECT 1 FROM files f WHERE f.series_id = s.id) AS has_files
FROM series s JOIN anilist_media m ON m.id = s.anilist_id";

/// One row of that query, before the tests below have been applied.
struct Row {
    candidate: Candidate,
    status: Option<String>,
    refreshed_at: Option<i64>,
    has_files: bool,
}

fn read_row(r: &rusqlite::Row) -> rusqlite::Result<Row> {
    Ok(Row {
        candidate: Candidate {
            series: r.get::<_, i64>(0)? as u64,
            anilist_id: r.get::<_, i64>(1)? as u64,
            mal_id: r.get::<_, Option<i64>>(2)?.map(|v| v as u64),
            folder_name: r.get(3)?,
        },
        status: r.get(4)?,
        refreshed_at: r.get(5)?,
        has_files: r.get::<_, i64>(6)? == 1,
    })
}

/// Only a series that is actually still airing has a next episode, and
/// only one with files on disk is worth a request. `force` waives the six
/// hour window and nothing else: a finished series stays finished however
/// hard it is asked for.
fn eligible(row: &Row, now: i64, force: bool) -> bool {
    if !row.has_files {
        return false;
    }
    if row.status.as_deref().and_then(AiringStatus::from_provider) != Some(AiringStatus::Releasing) {
        return false;
    }
    force || row.refreshed_at.is_none_or(|at| now.saturating_sub(at) >= ttl_secs())
}

fn ttl_secs() -> i64 {
    i64::try_from(TTL.as_secs()).unwrap_or(i64::MAX)
}

/// Whether this one series is worth a fetch right now.
pub fn candidate(conn: &Connection, series: u64, now: i64, force: bool) -> Result<Option<Candidate>, CoreError> {
    let sql = format!("{ELIGIBILITY_SQL} WHERE s.id = ?1");
    let row = conn.query_row(&sql, params![series as i64], read_row).optional()?;
    Ok(row.filter(|row| eligible(row, now, force)).map(|row| row.candidate))
}

/// Every series worth a fetch right now, in id order, which is what the
/// launch sweep walks.
pub fn candidates(conn: &Connection, now: i64) -> Result<Vec<Candidate>, CoreError> {
    let sql = format!("{ELIGIBILITY_SQL} ORDER BY s.id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], read_row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows.into_iter().filter(|row| eligible(row, now, false)).map(|row| row.candidate).collect())
}

/// One series' fetch and write. The schedule is the point of it; Jikan
/// rides along for the titles of episodes the table has none for, and its
/// failure costs those titles rather than the whole refresh.
///
/// Returns whether any row actually changed. `write_episodes` cannot say,
/// so the rows are read either side of it inside the one transaction and
/// compared: the same numbers with the same titles and the same dates is
/// a refresh nobody needs to hear about.
pub async fn refresh_one(core: &Core, cand: &Candidate, now: i64) -> Result<bool, CoreError> {
    let schedule = core.anilist.schedule(cand.anilist_id).await?;
    let jikan = match cand.mal_id {
        Some(mal_id) => match core.jikan.episodes(mal_id).await {
            Ok(episodes) => episodes,
            Err(e) => {
                core.report_jikan_outage(&message_of(&e));
                Vec::new()
            }
        },
        None => Vec::new(),
    };
    // No streaming titles: this job never fetches the enrichment they come
    // from, and an empty list is not an instruction to clear anything.
    let rows = record::merge_episodes(Some(&schedule), &[], &jikan);
    let anilist_id = cand.anilist_id;
    core.store
        .tx_async(move |tx| {
            let before = stored(tx, anilist_id)?;
            record::write_episodes(tx, anilist_id, &rows, true, now)?;
            stamp(tx, anilist_id, now)?;
            Ok(before != stored(tx, anilist_id)?)
        })
        .await
}

/// One stored row as the comparison sees it: the number, the title and
/// the date, which is every column this job can write.
type StoredRow = (i64, Option<String>, Option<i64>);

/// The rows behind one media id, in episode order.
fn stored(tx: &Transaction, anilist_id: u64) -> Result<Vec<StoredRow>, CoreError> {
    let mut stmt = tx.prepare("SELECT number, title, aired_at FROM anilist_episodes WHERE anilist_id = ?1 ORDER BY number")?;
    let rows = stmt.query_map(params![anilist_id as i64], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// The series had its turn, whether or not anything came back. Without
/// this a series AniList holds no schedule for would be asked about again
/// every time the sweep runs.
fn stamp(tx: &Transaction, anilist_id: u64, now: i64) -> Result<(), CoreError> {
    tx.execute("UPDATE anilist_media SET airing_refreshed_at = ?2 WHERE id = ?1", params![anilist_id as i64, now])?;
    Ok(())
}

async fn stamp_async(core: &Core, anilist_id: u64, now: i64) -> Result<(), CoreError> {
    core.store.tx_async(move |tx| stamp(tx, anilist_id, now)).await
}

/// What one series' turn produced. A failure that is the provider's rather
/// than the core's still counts as the turn having happened.
async fn refresh_reported(core: &Core, ctx: &JobCtx, cand: &Candidate, now: i64) -> Result<bool, CoreError> {
    match refresh_one(core, cand, now).await {
        Ok(updated) => Ok(updated),
        // A rate limit the limiter could not ride out is AniList saying
        // stop, not this series failing.
        Err(e) if is_rate_limited(&e) => Err(e),
        Err(e) => {
            ctx.emit(Level::Warn, format!("airing refresh failed for {}: {}", cand.folder_name, message_of(&e)), EventBody::Notice);
            stamp_async(core, cand.anilist_id, now).await?;
            Ok(false)
        }
    }
}

/// Starts the RefreshAiring job for one series. A series that is not a
/// candidate ends the job at once rather than refusing the call: the shell
/// asks for this on its way into a page, and "nothing to do" is an answer
/// rather than an error.
pub fn start_refresh(core: &Core, series: u64) -> Result<u64, CoreError> {
    let known: Option<i64> = core.store.read(|c| {
        Ok(c.query_row("SELECT 1 FROM series WHERE id = ?1", params![series as i64], |r| r.get(0)).optional()?)
    })?;
    if known.is_none() {
        return Err(CoreError::NotFound { what: Entity::Series, id: series });
    }
    let owner = owner(core)?;
    Ok(owner.jobs.clone().start(JobKind::RefreshAiring, move |ctx| async move {
        let now = time::now_secs();
        let cand = owner.store.write_async(move |c| candidate(c, series, now, false)).await?;
        let Some(cand) = cand else {
            return Ok(Finished {
                level: Level::Debug,
                message: format!("airing: series {series} needs no refresh"),
                body: EventBody::AiringRefreshed { series, updated: false },
            });
        };
        let updated = refresh_reported(&owner, &ctx, &cand, now).await?;
        if updated
            && let Some(card) = card_for(&owner, series).await?
        {
            ctx.changed(card);
        }
        Ok(Finished {
            level: Level::Debug,
            message: if updated { format!("airing refreshed for {}", cand.folder_name) } else { format!("airing unchanged for {}", cand.folder_name) },
            body: EventBody::AiringRefreshed { series, updated },
        })
    }))
}

/// Starts the launch sweep: every releasing series whose schedule is older
/// than the window, one at a time, paced by the limiter. Electron's
/// `refreshAiringForLibrary`.
///
/// The list is read once, up front, so the loop is finite whatever the
/// fetches write back.
pub fn start_refresh_library(core: &Arc<Core>) -> u64 {
    let owner = core.clone();
    core.jobs.clone().start(JobKind::RefreshAiring, move |ctx| async move {
        let now = time::now_secs();
        let work = owner.store.write_async(move |c| candidates(c, now)).await?;
        let total = work.len() as u64;
        let mut updated = 0u64;
        for (done, cand) in work.into_iter().enumerate() {
            ctx.checkpoint()?;
            ctx.progress(done as u64, Some(total), &cand.folder_name);
            // Each series is stamped with the time its own turn came, not
            // the time the sweep started: a long walk would otherwise
            // stamp the last series hours before it was actually fetched.
            if refresh_reported(&owner, &ctx, &cand, time::now_secs()).await? {
                updated += 1;
                if let Some(card) = card_for(&owner, cand.series).await? {
                    ctx.changed(card);
                }
            }
        }
        Ok(Finished {
            level: Level::Info,
            message: format!("airing refreshed for {updated} series"),
            body: EventBody::Notice,
        })
    })
}
