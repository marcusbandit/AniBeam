//! The IMDb refresh: IMDb's rating on every card whose match leads to an
//! IMDb title, kept current by the launch and by nothing else.
//!
//! Two downloads, each on its own week. The ratings are IMDb's own dump,
//! every rated title in it rather than the library's, so a series matched
//! later already has its rating. The links are Wikidata's map from AniList
//! and MAL ids to IMDb titles; a TVmaze match needs none, since TVmaze
//! names the title itself. Each table is replaced whole in one transaction
//! that also stamps it, so a stamp only ever lands beside the rows it
//! vouches for. A download that fails, or that comes back too small to be
//! the real thing, rolls back and leaves the old rows exactly as they were,
//! unstamped, and the next launch asks again.
//!
//! Failures are a line in the log rather than a red job. Neither host is
//! one of the contract's providers, so there is no `CoreError::Provider`
//! to fail with, and a week-old rating on a card is still the best answer
//! there is while the network is away. A launch that reaches neither host
//! writes one Warn line, never one per download.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rusqlite::{Connection, Transaction, TransactionBehavior, params};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::library::cards;
use crate::library::snapshot::{ImdbPick, imdb_picks};
use crate::net::Fault;
use crate::net::imdb::{self, Dump, Ratings};
use crate::net::wikidata::{self, ImdbLink};
use crate::store::settings;
use crate::store::sql::as_i64;
use crate::time;

/// How old the stored dump may get before a launch asks IMDb again. IMDb
/// rebuilds it daily, but a rating does not move enough in a week for a
/// card to show it, and an unchanged dump costs a 304 anyway.
pub const RATINGS_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// How old the links may get. Wikidata's anime ids change slower still.
pub const LINKS_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Fewer titles than this is not IMDb's dump, whatever it parsed as: the
/// real one holds about 1.7 million. Replacing the table with it would
/// empty most cards' ratings for a week.
const MIN_RATINGS: u64 = 100_000;

/// Fewer links than this is a query that went wrong: the real answer is
/// close to ten thousand rows.
const MIN_LINKS: usize = 1_000;

/// What one download came to.
#[derive(Debug)]
enum Step {
    /// Not due: fetched inside its week, so nothing was asked.
    Fresh,
    /// IMDb answered 304: the stored dump is still its latest.
    Unchanged,
    /// Replaced whole: how many rows went in, and the series whose IMDb
    /// title, rating or votes moved with them.
    Replaced { rows: u64, changed: Vec<u64> },
    /// Nothing written, for this reason. The old rows stand.
    Failed(Fault),
}

/// Starts the RefreshImdb job. The launch is its only caller, and it runs
/// one at a time besides.
pub fn start_refresh(core: &Arc<Core>) -> u64 {
    let owner = core.clone();
    core.jobs
        .clone()
        .start(JobKind::RefreshImdb, move |ctx| async move {
            run(&owner, &ctx).await
        })
}

async fn run(core: &Core, ctx: &Arc<JobCtx>) -> Result<Finished, CoreError> {
    let now = time::now_secs();
    let (links_at, ratings_at, since) = core
        .store
        .write_async(|c| {
            Ok((
                settings::get::<i64>(c, settings::IMDB_LINKS_FETCHED_AT)?,
                settings::get::<i64>(c, settings::IMDB_RATINGS_FETCHED_AT)?,
                settings::get::<String>(c, settings::IMDB_RATINGS_LAST_MODIFIED)?,
            ))
        })
        .await?;

    let links = if due(links_at, now, LINKS_TTL) {
        ctx.checkpoint()?;
        ctx.progress(0, None, "Wikidata's IMDb links");
        refresh_links(core, now).await?
    } else {
        Step::Fresh
    };
    let ratings = if due(ratings_at, now, RATINGS_TTL) {
        ctx.checkpoint()?;
        ctx.progress(0, None, "IMDb ratings");
        refresh_ratings(core, since, now).await?
    } else {
        Step::Fresh
    };

    let mut changed: Vec<u64> = [&links, &ratings]
        .into_iter()
        .filter_map(|step| match step {
            Step::Replaced { changed, .. } => Some(changed.iter().copied()),
            _ => None,
        })
        .flatten()
        .collect();
    changed.sort_unstable();
    changed.dedup();
    if !changed.is_empty() {
        let dir = core.paths.images_dir();
        let ids = changed.clone();
        let cards = core
            .store
            .write_async(move |c| cards::cards_for(c, &dir, &ids))
            .await?;
        ctx.changed_all(cards);
    }
    Ok(finished(&links, &ratings, changed.len()))
}

/// Never fetched, or fetched longer ago than the window.
fn due(at: Option<i64>, now: i64, ttl: Duration) -> bool {
    let ttl = i64::try_from(ttl.as_secs()).unwrap_or(i64::MAX);
    at.is_none_or(|at| now.saturating_sub(at) >= ttl)
}

/// The one line the job ends on. Debug when nothing was due or IMDb had
/// nothing new, since every launch runs this; Info when something came
/// down; Warn when a download failed, whatever else happened beside it.
fn finished(links: &Step, ratings: &Step, changed: usize) -> Finished {
    let mut parts: Vec<String> = Vec::new();
    for (what, step) in [("IMDb ratings", ratings), ("IMDb links", links)] {
        match step {
            Step::Fresh => {}
            Step::Unchanged => parts.push(format!("{what} unchanged")),
            Step::Replaced { rows, .. } => parts.push(format!("{what} updated, {rows} rows")),
            Step::Failed(fault) => parts.push(format!("{what} not refreshed: {fault}")),
        }
    }
    let failed = [links, ratings]
        .iter()
        .any(|s| matches!(s, Step::Failed(_)));
    let replaced = [links, ratings]
        .iter()
        .any(|s| matches!(s, Step::Replaced { .. }));
    let level = if failed {
        Level::Warn
    } else if replaced {
        Level::Info
    } else {
        Level::Debug
    };
    let message = if parts.is_empty() {
        "IMDb ratings are current".to_string()
    } else if replaced {
        format!("{} ({changed} series changed)", parts.join("; "))
    } else {
        parts.join("; ")
    };
    Finished {
        plain: None,
        level,
        message,
        body: EventBody::Notice,
    }
}

/// Wikidata's links, replaced whole.
async fn refresh_links(core: &Core, now: i64) -> Result<Step, CoreError> {
    let links = match wikidata::imdb_links(core.http.as_ref()).await {
        Ok(links) => links,
        Err(fault) => return Ok(Step::Failed(fault)),
    };
    if links.len() < MIN_LINKS {
        return Ok(Step::Failed(Fault::Refused(format!(
            "Wikidata answered with {} links, short of the {MIN_LINKS} it holds",
            links.len()
        ))));
    }
    core.store
        .tx_async(move |tx| replace_links(tx, &links, now))
        .await
}

fn replace_links(tx: &Transaction, links: &[ImdbLink], now: i64) -> Result<Step, CoreError> {
    let before = imdb_picks(tx, None)?;
    tx.execute("DELETE FROM imdb_links", [])?;
    {
        // The same id can sit on two Wikidata items naming one title, so
        // a repeat is dropped rather than failing the key.
        let mut insert = tx.prepare(
            "INSERT OR IGNORE INTO imdb_links (provider, provider_id, imdb) VALUES (?1, ?2, ?3)",
        )?;
        for link in links {
            insert.execute(params![
                link.provider.as_str(),
                as_i64(link.provider_id),
                as_i64(link.imdb)
            ])?;
        }
    }
    settings::set(tx, settings::IMDB_LINKS_FETCHED_AT, &now)?;
    let after = imdb_picks(tx, None)?;
    Ok(Step::Replaced {
        rows: links.len() as u64,
        changed: moved(&before, &after),
    })
}

/// IMDb's dump, asked for only if it changed since the one stored.
async fn refresh_ratings(core: &Core, since: Option<String>, now: i64) -> Result<Step, CoreError> {
    match imdb::fetch(core.http.as_ref(), since.as_deref()).await {
        Err(fault) => Ok(Step::Failed(fault)),
        // Still the dump the table holds, so only the clock moves.
        Ok(Dump::Unchanged) => {
            core.store
                .write_async(move |c| settings::set(c, settings::IMDB_RATINGS_FETCHED_AT, &now))
                .await?;
            Ok(Step::Unchanged)
        }
        Ok(Dump::Fresh { gz, last_modified }) => {
            core.store
                .write_async(move |c| replace_ratings(c, &gz, last_modified.as_deref(), now))
                .await
        }
    }
}

/// The dump into the table, decompressed and parsed row by row straight
/// into the insert, so 1.7 million rows never sit in memory at once. It
/// holds the writer for the second or two that takes, once a week.
///
/// The transaction is opened here rather than through `Store::tx` because
/// a dump that turns out wrong is an answer, not a storage failure: the
/// transaction is dropped, which rolls the delete back, and the job hears
/// a `Step::Failed` while the old table stands.
fn replace_ratings(
    conn: &mut Connection,
    gz: &[u8],
    last_modified: Option<&str>,
    now: i64,
) -> Result<Step, CoreError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let before = imdb_picks(&tx, None)?;
    tx.execute("DELETE FROM imdb_ratings", [])?;
    let rows = {
        let ratings = match Ratings::new(gz) {
            Ok(ratings) => ratings,
            Err(fault) => return Ok(Step::Failed(fault)),
        };
        let mut insert = tx.prepare(
            "INSERT OR REPLACE INTO imdb_ratings (id, rating, votes) VALUES (?1, ?2, ?3)",
        )?;
        let mut rows = 0u64;
        for rating in ratings {
            let rating = match rating {
                Ok(rating) => rating,
                Err(fault) => return Ok(Step::Failed(fault)),
            };
            insert.execute(params![
                as_i64(rating.id),
                rating.tenths,
                as_i64(rating.votes)
            ])?;
            rows += 1;
        }
        rows
    };
    if rows < MIN_RATINGS {
        return Ok(Step::Failed(Fault::Refused(format!(
            "the IMDb dump held {rows} titles, short of the {MIN_RATINGS} a real one holds"
        ))));
    }
    settings::set(&tx, settings::IMDB_RATINGS_FETCHED_AT, &now)?;
    // A dump served without a date cannot be asked about by date, so the
    // old one goes rather than vouching for rows it never saw.
    match last_modified {
        Some(at) => settings::set(&tx, settings::IMDB_RATINGS_LAST_MODIFIED, &at)?,
        None => settings::remove(&tx, settings::IMDB_RATINGS_LAST_MODIFIED)?,
    }
    let after = imdb_picks(&tx, None)?;
    tx.commit()?;
    Ok(Step::Replaced {
        rows,
        changed: moved(&before, &after),
    })
}

/// Every series whose pick differs between the two reads: a title gained,
/// lost or swapped, or the same title with a new rating or vote count.
fn moved(before: &HashMap<u64, ImdbPick>, after: &HashMap<u64, ImdbPick>) -> Vec<u64> {
    let mut ids: Vec<u64> = before
        .keys()
        .chain(after.keys())
        .copied()
        .filter(|id| before.get(id) != after.get(id))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pick(id: u64, rating: Option<u32>, votes: Option<u64>) -> ImdbPick {
        ImdbPick { id, rating, votes }
    }

    #[test]
    fn a_download_is_due_when_never_made_or_a_week_old() {
        let week = 7 * 24 * 60 * 60;
        assert!(due(None, 1_000_000, RATINGS_TTL));
        assert!(!due(Some(1_000_000 - week + 1), 1_000_000, RATINGS_TTL));
        assert!(due(Some(1_000_000 - week), 1_000_000, RATINGS_TTL));
        // A clock set back past the stamp is not a week gone.
        assert!(!due(Some(2_000_000), 1_000_000, LINKS_TTL));
    }

    #[test]
    fn only_a_pick_that_differs_counts_as_moved() {
        let before = HashMap::from([
            (1, pick(10, Some(80), Some(100))),
            (2, pick(20, Some(70), Some(50))),
            (3, pick(30, None, None)),
            (4, pick(40, Some(60), Some(9))),
        ]);
        let after = HashMap::from([
            (1, pick(10, Some(80), Some(100))),
            (2, pick(20, Some(70), Some(51))),
            (3, pick(30, Some(55), Some(3))),
            (5, pick(50, Some(90), Some(1))),
        ]);
        assert_eq!(moved(&before, &after), vec![2, 3, 4, 5]);
        assert!(moved(&after, &after).is_empty());
    }

    #[test]
    fn the_closing_line_is_quiet_unless_something_came_down_or_failed() {
        let quiet = finished(&Step::Fresh, &Step::Fresh, 0);
        assert_eq!(quiet.level, Level::Debug);
        assert_eq!(quiet.message, "IMDb ratings are current");

        let unchanged = finished(&Step::Fresh, &Step::Unchanged, 0);
        assert_eq!(unchanged.level, Level::Debug);
        assert_eq!(unchanged.message, "IMDb ratings unchanged");

        let replaced = finished(
            &Step::Fresh,
            &Step::Replaced {
                rows: 1_720_646,
                changed: vec![1, 2],
            },
            2,
        );
        assert_eq!(replaced.level, Level::Info);
        assert_eq!(
            replaced.message,
            "IMDb ratings updated, 1720646 rows (2 series changed)"
        );

        let offline = finished(
            &Step::Failed(Fault::Unreachable("dns error".into())),
            &Step::Failed(Fault::Unreachable("dns error".into())),
            0,
        );
        assert_eq!(offline.level, Level::Warn);
        assert_eq!(
            offline.message,
            "IMDb ratings not refreshed: unreachable: dns error; IMDb links not refreshed: unreachable: dns error"
        );
    }
}
