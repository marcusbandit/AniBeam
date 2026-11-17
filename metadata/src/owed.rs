//! The episode titles a Jikan outage still owes (#1065).
//!
//! Jikan is the title side-fetch, so a match that lands while it is down
//! lands without titles. A releasing series gets them on its next airing
//! refresh, but a finished one is never refreshed on its own, so the full
//! fetch writes its id down here instead and the titles are asked for again
//! once Jikan is back. Nothing here runs on a timer: the list is walked at
//! launch and on the first whole Jikan reply after an outage window, and a
//! walk that finds Jikan down again stops and keeps the rest for next time.
//!
//! The walk asks Jikan alone and only fills what is missing. It has no
//! schedule to go with Jikan's list, so it deletes nothing, keeps every
//! title already in the table (a streaming title from AniList outranks
//! Jikan's) and keeps every date AniList already gave a row, which is to
//! the second where Jikan's is to the day.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::Metadata;
use crate::apply::card_for;
use crate::record::{self, EpisodeRow};
use anibeam_contract::*;
use anibeam_foundation::jobs::{Finished, JobCtx};
use anibeam_foundation::store::settings;
use anibeam_foundation::store::sql::as_i64;

/// The ids still owed their titles.
pub fn owed(conn: &Connection) -> Result<BTreeSet<u64>, CoreError> {
    settings::get_or_default(conn, settings::JIKAN_OWED)
}

/// Writes down, or crosses off, that `anilist_id` is owed Jikan's titles.
/// Every fetch that asks Jikan calls it, so a fetch that got them by any
/// route crosses the series off, and only one that met an outage leaves it
/// on the list.
pub fn mark(tx: &Transaction, anilist_id: u64, owed_now: bool) -> Result<(), CoreError> {
    let mut list = owed(tx)?;
    let changed = if owed_now {
        list.insert(anilist_id)
    } else {
        list.remove(&anilist_id)
    };
    if !changed {
        return Ok(());
    }
    if list.is_empty() {
        settings::remove(tx, settings::JIKAN_OWED)
    } else {
        settings::set(tx, settings::JIKAN_OWED, &list)
    }
}

/// Starts the walk when anything is owed and no walk is running. Returns
/// the job's id, or `None` when there was nothing to start, which is the
/// usual answer: a launch with nothing owed starts no job at all.
pub fn start(metadata: &Arc<Metadata>) -> Option<u64> {
    let any = metadata
        .services
        .store
        .read(|c| Ok(!owed(c)?.is_empty()))
        .unwrap_or(false);
    if !any || metadata.owed_walking.swap(true, Ordering::SeqCst) {
        return None;
    }
    let owner = metadata.clone();
    Some(
        metadata
            .services
            .jobs
            .clone()
            .start(JobKind::RefreshAiring, move |ctx| async move {
                let walked = walk(&owner, &ctx).await;
                owner.owed_walking.store(false, Ordering::SeqCst);
                let filled = walked?;
                Ok(Finished {
                    plain: None,
                    level: if filled == 0 {
                        Level::Debug
                    } else {
                        Level::Info
                    },
                    message: format!("Jikan titles filled for {filled} series"),
                    body: EventBody::Notice,
                })
            }),
    )
}

/// One owed series as the walk reads it: its MAL id, when it still has one,
/// the series matched to it, whose cards are drawn again afterwards, and
/// the name the progress line shows.
struct Owed {
    anilist_id: u64,
    mal_id: Option<u64>,
    series: Vec<u64>,
    name: String,
}

fn read_owed(conn: &Connection) -> Result<Vec<Owed>, CoreError> {
    let mut out = Vec::new();
    for anilist_id in owed(conn)? {
        let mal_id: Option<i64> = conn
            .query_row(
                "SELECT mal_id FROM anilist_media WHERE id = ?1",
                params![as_i64(anilist_id)],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        let mut stmt =
            conn.prepare("SELECT id, folder_name FROM series WHERE anilist_id = ?1 ORDER BY id")?;
        let series = stmt
            .query_map(params![as_i64(anilist_id)], |r| {
                Ok((r.get::<_, i64>(0)? as u64, r.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let name = series
            .first()
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| anilist_id.to_string());
        out.push(Owed {
            anilist_id,
            mal_id: mal_id.map(|v| v as u64),
            series: series.into_iter().map(|(id, _)| id).collect(),
            name,
        });
    }
    Ok(out)
}

/// Every owed series, one at a time, paced by the limiter. Returns how many
/// got titles they did not have.
async fn walk(metadata: &Metadata, ctx: &Arc<JobCtx>) -> Result<u64, CoreError> {
    let work = metadata
        .services
        .store
        .write_async(|c| read_owed(c))
        .await?;
    let total = work.len() as u64;
    let mut filled = 0u64;
    for (done, item) in work.into_iter().enumerate() {
        ctx.checkpoint()?;
        ctx.progress(done as u64, Some(total), &item.name);
        let anilist_id = item.anilist_id;
        // A media row without a MAL id has nothing to ask Jikan about, and
        // a later match may have moved it on; either way nothing is owed.
        let Some(mal_id) = item.mal_id else {
            metadata
                .services
                .store
                .tx_async(move |tx| mark(tx, anilist_id, false))
                .await?;
            continue;
        };
        let jikan = metadata.jikan_episodes(mal_id).await;
        // Down again: what came back is kept, and this series and the rest
        // stay owed for the next launch or the next reply.
        let down = !jikan.whole && metadata.jikan_down();
        let rows = record::merge_episodes(None, &[], &jikan.episodes);
        let changed = metadata
            .services
            .store
            .tx_async(move |tx| {
                let before = titled(tx, anilist_id)?;
                fill(tx, anilist_id, &rows)?;
                // A reply that was not whole and not an outage is this
                // series' own bad answer, which asking again will not mend.
                mark(tx, anilist_id, down)?;
                Ok(titled(tx, anilist_id)? != before)
            })
            .await?;
        if changed {
            filled += 1;
            for series in item.series {
                if let Some(card) = card_for(&metadata.services, series).await? {
                    ctx.changed(card);
                }
            }
        }
        if down {
            break;
        }
    }
    Ok(filled)
}

/// How many rows behind `anilist_id` carry a title.
fn titled(tx: &Transaction, anilist_id: u64) -> Result<i64, CoreError> {
    Ok(tx.query_row(
        "SELECT count(*) FROM anilist_episodes WHERE anilist_id = ?1 AND title IS NOT NULL",
        params![as_i64(anilist_id)],
        |r| r.get(0),
    )?)
}

/// Jikan's rows where the table has nothing: a new row goes in whole, and a
/// row already there gains a title or a date only where it has none.
fn fill(tx: &Transaction, anilist_id: u64, rows: &[EpisodeRow]) -> Result<(), CoreError> {
    for r in rows {
        tx.execute(
            "INSERT INTO anilist_episodes (anilist_id, number, title, aired_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(anilist_id, number) DO UPDATE SET
                title = coalesce(anilist_episodes.title, excluded.title),
                aired_at = coalesce(anilist_episodes.aired_at, excluded.aired_at)",
            params![as_i64(anilist_id), i64::from(r.number), r.title, r.aired_at],
        )?;
    }
    Ok(())
}
