//! The progress cache: what each tracker says about every anime on the
//! user's list, kept in `tracker_entries` so a card can draw its progress,
//! its status and its score without a request.
//!
//! Two rules shape it. A list is one request per tracker, never one per
//! series, so the whole list is fetched and the table replaced; and a
//! fetched list is taken to be current for five minutes, which is short
//! enough that a watching session sees its own marks and long enough that
//! opening and closing the app a few times does not hammer AniList.
//!
//! Carried from Electron's `fetchAnilistProgressMap`,
//! `fetchMalProgressMap`, `refreshProgress` and `refreshAllProgress` in
//! `src/main/handlers/trackerHandler.ts`, and `setProgressEntry` and
//! `setProgressScoreAndStatus` in `src/main/services/trackerStore.ts`.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rusqlite::{Connection, Transaction, params};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::library::cards;
use crate::net::anilist::MEDIA_LIST_COLLECTION_QUERY;
use crate::time;
use crate::trackers::accounts;
use crate::trackers::writes;

/// How long a fetched list is taken to be current. Electron's
/// `PROGRESS_FRESHNESS_MS`.
pub const PROGRESS_FRESH: Duration = Duration::from_secs(5 * 60);

/// How long one list read has before it is given up on. A list is what a
/// page is waiting for, so it gives up well before the transport's own
/// thirty seconds.
const FETCH_TIMEOUT: Duration = Duration::from_secs(15);

/// MAL pages its list, and hands back a `paging.next` while there is more.
/// The page count is a hard bound rather than a real limit: fifty pages is
/// fifty thousand entries, far past the largest list anyone has, and it is
/// what keeps a malformed cursor from looping for ever.
const MAL_PAGE: u64 = 1000;
const MAL_MAX_PAGES: u32 = 50;

/// The account says it is connected but there is no token behind it: a
/// keyring that lost the entry, or a MAL session whose refresh has failed
/// and already said so.
const NO_TOKEN: &str = "no access token stored, reconnect in Settings";

/// One anime on one tracker's list. `progress` and `repeat` are counts, so
/// nothing there is optional; `score` and `status` are absent when the
/// entry carries neither.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub media_id: u64,
    pub progress: u32,
    pub status: Option<ListStatus>,
    pub score: Option<f64>,
    pub repeat: u32,
    pub updated_at: Option<i64>,
}

/// Both trackers' list vocabularies in the one the core stores. MAL has no
/// rewatching status at all: it keeps `watching` and raises a flag, so the
/// flag is what promotes an entry to Repeating and makes the two trackers
/// comparable. Anything unrecognised is no status rather than a guess.
pub fn normalize_status(t: Tracker, raw: Option<&str>, is_rewatching: bool) -> Option<ListStatus> {
    let raw = raw?;
    match t {
        Tracker::Anilist => match raw {
            "CURRENT" => Some(ListStatus::Watching),
            "PLANNING" => Some(ListStatus::Planning),
            "COMPLETED" => Some(ListStatus::Completed),
            "PAUSED" => Some(ListStatus::Paused),
            "DROPPED" => Some(ListStatus::Dropped),
            "REPEATING" => Some(ListStatus::Repeating),
            _ => None,
        },
        Tracker::Mal => match raw {
            "watching" if is_rewatching => Some(ListStatus::Repeating),
            "watching" => Some(ListStatus::Watching),
            "plan_to_watch" => Some(ListStatus::Planning),
            "completed" => Some(ListStatus::Completed),
            "on_hold" => Some(ListStatus::Paused),
            "dropped" => Some(ListStatus::Dropped),
            _ => None,
        },
    }
}

// The fetches ----------------------------------------------------------------

/// AniList's whole anime list in one query, keyed by the AniList media id
/// a match carries. `score(format: POINT_10_DECIMAL)` is what makes the
/// number comparable whatever display format the user picked, and a score
/// of 0 is AniList's unrated rather than a rating.
pub async fn fetch_anilist(
    core: &Arc<Core>,
    user_id: u64,
    token: &str,
) -> Result<Vec<Entry>, CoreError> {
    let data = tokio::time::timeout(
        FETCH_TIMEOUT,
        core.anilist.graphql(
            MEDIA_LIST_COLLECTION_QUERY,
            serde_json::json!({ "userId": user_id }),
            Some(token),
        ),
    )
    .await
    .map_err(|_| timed_out(Tracker::Anilist, "AniList MediaListCollection"))??;
    let mut entries = Vec::new();
    for list in data["MediaListCollection"]["lists"]
        .as_array()
        .into_iter()
        .flatten()
    {
        for entry in list["entries"].as_array().into_iter().flatten() {
            let Some(media_id) = entry["media"]["id"].as_u64() else {
                continue;
            };
            entries.push(Entry {
                media_id,
                progress: as_count(entry["progress"].as_u64()),
                status: normalize_status(Tracker::Anilist, entry["status"].as_str(), false),
                score: entry["score"].as_f64().filter(|s| *s > 0.0),
                repeat: as_count(entry["repeat"].as_u64()),
                // The collection query asks for no timestamp, so an entry
                // brings none and `replace` keeps whatever the watching
                // list left behind.
                updated_at: None,
            });
        }
    }
    Ok(entries)
}

/// MAL's list, a page at a time, keyed by the MAL anime id a match
/// carries. The walk ends when MAL stops sending a next cursor or hands
/// back a page that was not full, whichever comes first.
pub async fn fetch_mal(core: &Arc<Core>, token: &str) -> Result<Vec<Entry>, CoreError> {
    let mut entries = Vec::new();
    let mut offset = 0u64;
    for page in 0..MAL_MAX_PAGES {
        let url = format!(
            "https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status{{status,num_episodes_watched,is_rewatching,num_times_rewatched,score}}&limit={MAL_PAGE}&offset={offset}"
        );
        let response = tokio::time::timeout(FETCH_TIMEOUT, core.mal.get(&url, token))
            .await
            .map_err(|_| timed_out(Tracker::Mal, &format!("MAL animelist page {page}")))??;
        if !response.is_success() {
            return Err(CoreError::Provider {
                provider: Provider::Mal,
                status: Some(u32::from(response.status)),
                message: response.text(),
                retry_after: None,
            });
        }
        let body: serde_json::Value = response.json()?;
        let page_entries = body["data"].as_array().cloned().unwrap_or_default();
        for item in &page_entries {
            let Some(media_id) = item["node"]["id"].as_u64() else {
                continue;
            };
            let status = &item["list_status"];
            entries.push(Entry {
                media_id,
                progress: as_count(status["num_episodes_watched"].as_u64()),
                status: normalize_status(
                    Tracker::Mal,
                    status["status"].as_str(),
                    status["is_rewatching"].as_bool().unwrap_or(false),
                ),
                score: status["score"].as_f64().filter(|s| *s > 0.0),
                repeat: as_count(status["num_times_rewatched"].as_u64()),
                updated_at: None,
            });
        }
        if body["paging"]["next"].as_str().is_none() || (page_entries.len() as u64) < MAL_PAGE {
            break;
        }
        offset += MAL_PAGE;
    }
    Ok(entries)
}

// The table ------------------------------------------------------------------

/// A tracker's whole list, replacing whatever was cached for it. The list
/// is the truth about itself, so a row the tracker no longer carries goes;
/// what survives is an `updated_at` the fetch had none of, since the
/// collection query carries no timestamp and the watching list does.
pub fn replace(tx: &Transaction, t: Tracker, entries: &[Entry], now: i64) -> Result<(), CoreError> {
    let mut kept: HashMap<u64, i64> = HashMap::new();
    {
        let mut stmt = tx.prepare("SELECT media_id, updated_at FROM tracker_entries WHERE tracker = ?1 AND updated_at IS NOT NULL")?;
        let rows = stmt.query_map(params![t.as_str()], |r| {
            Ok((r.get::<_, i64>(0)? as u64, r.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (media_id, at) = row?;
            kept.insert(media_id, at);
        }
    }
    tx.execute(
        "DELETE FROM tracker_entries WHERE tracker = ?1",
        params![t.as_str()],
    )?;
    {
        // `OR REPLACE` rather than a plain insert: a list that named one
        // media twice is the provider's business, and the last word wins
        // the way Electron's map did.
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO tracker_entries (tracker, media_id, status, progress, score, repeat, updated_at, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for entry in entries {
            stmt.execute(params![
                t.as_str(),
                entry.media_id as i64,
                entry.status.map(ListStatus::as_str),
                i64::from(entry.progress),
                entry.score,
                i64::from(entry.repeat),
                entry
                    .updated_at
                    .or_else(|| kept.get(&entry.media_id).copied()),
                now,
            ])?;
        }
    }
    tx.execute(
        "UPDATE tracker_accounts SET progress_fetched_at = ?2 WHERE tracker = ?1",
        params![t.as_str(), now],
    )?;
    Ok(())
}

/// One entry's progress, after a mark or a set: the score and the rewatch
/// count are the list's and are left alone, and a status of `None` keeps
/// the one already cached. Electron's `setProgressEntry`.
pub fn patch_progress(
    conn: &Connection,
    t: Tracker,
    media_id: u64,
    progress: u32,
    status: Option<ListStatus>,
    now: i64,
) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO tracker_entries (tracker, media_id, status, progress, score, repeat, updated_at, fetched_at)
         VALUES (?1, ?2, ?3, ?4, NULL, 0, NULL, ?5)
         ON CONFLICT(tracker, media_id) DO UPDATE SET
             progress = excluded.progress,
             status = COALESCE(excluded.status, tracker_entries.status),
             fetched_at = excluded.fetched_at",
        params![t.as_str(), media_id as i64, status.map(ListStatus::as_str), i64::from(progress), now],
    )?;
    Ok(())
}

/// One entry's score, after a rating: the progress and the rewatch count
/// are left alone, and a score of `None` is stored as unrated rather than
/// dropped. Electron's `setProgressScoreAndStatus`.
pub fn patch_score(
    conn: &Connection,
    t: Tracker,
    media_id: u64,
    score: Option<f64>,
    status: Option<ListStatus>,
    now: i64,
) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO tracker_entries (tracker, media_id, status, progress, score, repeat, updated_at, fetched_at)
         VALUES (?1, ?2, ?3, 0, ?4, 0, NULL, ?5)
         ON CONFLICT(tracker, media_id) DO UPDATE SET
             score = excluded.score,
             status = COALESCE(excluded.status, tracker_entries.status),
             fetched_at = excluded.fetched_at",
        params![t.as_str(), media_id as i64, status.map(ListStatus::as_str), score, now],
    )?;
    Ok(())
}

// The job --------------------------------------------------------------------

/// Starts the RefreshProgress job: the tracker named, or every connected
/// one. `force` waives the five minute window, which is what a fresh
/// connection wants and what a page opening does not.
pub fn start_refresh(core: &Arc<Core>, tracker: Option<Tracker>, force: bool) -> u64 {
    let owner = core.clone();
    core.jobs
        .clone()
        .start(JobKind::RefreshProgress, move |ctx| async move {
            run(owner, ctx, tracker, force).await
        })
}

async fn run(
    core: Arc<Core>,
    ctx: Arc<JobCtx>,
    tracker: Option<Tracker>,
    force: bool,
) -> Result<Finished, CoreError> {
    let wanted: Vec<Tracker> = match tracker {
        Some(t) => vec![t],
        None => vec![Tracker::Anilist, Tracker::Mal],
    };
    // The tracker whose `ProgressRefreshed` has not been emitted yet: the
    // job ends with the last one, so each is held until either another
    // takes its place or the job is over.
    let mut pending: Option<(Tracker, usize)> = None;
    let mut refreshed: Vec<Tracker> = Vec::new();
    let mut failure: Option<CoreError> = None;

    for t in wanted {
        ctx.checkpoint()?;
        let row = core
            .store
            .write_async(move |c| accounts::load_row(c, t))
            .await?
            .unwrap_or_default();
        // Not connected is nothing to fetch, whether the tracker was asked
        // for by name or swept up by a refresh of everything.
        if row.connected_at.is_none() {
            continue;
        }
        if !force && is_fresh(row.progress_fetched_at, time::now()) {
            ctx.emit(
                Level::Debug,
                format!("{} progress is fresh", t.as_str()),
                EventBody::Notice,
            );
            continue;
        }
        match fetch_and_replace(&core, t, row.user_id).await {
            Ok(count) => {
                if let Some((previous, n)) = pending.replace((t, count)) {
                    ctx.emit(
                        Level::Info,
                        refreshed_line(previous, n),
                        EventBody::ProgressRefreshed { tracker: previous },
                    );
                }
                refreshed.push(t);
            }
            Err(e) => {
                // The rows stay: a list that could not be fetched is still
                // the best answer there is, and a card with yesterday's
                // progress reads better than a card with none.
                ctx.emit(
                    Level::Warn,
                    format!(
                        "{} progress refresh failed: {}",
                        t.as_str(),
                        writes::sanitize_error(t, &e)
                    ),
                    EventBody::Notice,
                );
                failure = Some(e);
            }
        }
    }

    if !refreshed.is_empty() {
        let cards = matched_cards(&core, &refreshed).await?;
        ctx.changed_all(cards);
    }
    match pending {
        Some((t, count)) => Ok(Finished {
            level: Level::Info,
            message: refreshed_line(t, count),
            body: EventBody::ProgressRefreshed { tracker: t },
        }),
        // Nothing was refreshed, so a failure here is every tracker that
        // was asked for: the job failed. One failure among several never
        // gets this far, since the one that worked ends the job.
        None => match failure {
            Some(e) => Err(e),
            None => Ok(Finished {
                level: Level::Debug,
                message: "nothing to refresh".to_string(),
                body: EventBody::Notice,
            }),
        },
    }
}

/// One tracker's list into the table, and how many entries it held.
async fn fetch_and_replace(
    core: &Arc<Core>,
    t: Tracker,
    user_id: Option<u64>,
) -> Result<usize, CoreError> {
    let token = accounts::access_token(core, t)
        .await?
        .ok_or_else(|| accounts::tracker_error(t, NO_TOKEN))?;
    let entries = match t {
        Tracker::Anilist => {
            // AniList's collection is asked for by user id, so an account
            // whose profile never carried one cannot be read at all.
            let user_id = user_id
                .ok_or_else(|| accounts::tracker_error(t, "the account carries no user id"))?;
            fetch_anilist(core, user_id, &token).await?
        }
        Tracker::Mal => fetch_mal(core, &token).await?,
    };
    let count = entries.len();
    let now = time::now_secs();
    core.store
        .tx_async(move |tx| replace(tx, t, &entries, now))
        .await?;
    Ok(count)
}

/// Every matched series whose match carries a refreshed tracker's id, as
/// cards: the numbers on all of them just changed, so they leave in one
/// batch rather than one event each.
async fn matched_cards(
    core: &Arc<Core>,
    refreshed: &[Tracker],
) -> Result<Vec<SeriesCard>, CoreError> {
    let images_dir = core.paths.images_dir();
    let refreshed = refreshed.to_vec();
    core.store
        .write_async(move |c| {
            let mut ids: BTreeSet<u64> = BTreeSet::new();
            for t in refreshed {
                let sql = match t {
                    Tracker::Anilist => "SELECT id FROM series WHERE anilist_id IS NOT NULL",
                    Tracker::Mal => "SELECT id FROM series WHERE mal_id IS NOT NULL",
                };
                let mut stmt = c.prepare(sql)?;
                let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
                for row in rows {
                    ids.insert(row? as u64);
                }
            }
            let ids: Vec<u64> = ids.into_iter().collect();
            cards::cards_for(c, &images_dir, &ids)
        })
        .await
}

/// Whether a tracker's cache is inside the five minute window. A stamp in
/// the future is a clock that moved rather than a stale cache, so it
/// counts as fresh.
fn is_fresh(fetched_at: Option<SystemTime>, now: SystemTime) -> bool {
    let Some(at) = fetched_at else { return false };
    match now.duration_since(at) {
        Ok(elapsed) => elapsed < PROGRESS_FRESH,
        Err(_) => true,
    }
}

fn refreshed_line(t: Tracker, count: usize) -> String {
    format!("{} progress cache refreshed ({count} entries)", t.as_str())
}

/// A list read that ran out of time. The provider never answered, so there
/// is no status to carry.
fn timed_out(t: Tracker, label: &str) -> CoreError {
    accounts::tracker_error(
        t,
        format!("{label} timed out after {}ms", FETCH_TIMEOUT.as_millis()),
    )
}

/// A count off a provider's JSON, which is unsigned and small: anything
/// missing or absurd is nought rather than a wrap-around.
fn as_count(value: Option<u64>) -> u32 {
    value.and_then(|v| u32::try_from(v).ok()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anilist_statuses_map_one_to_one() {
        let map = |raw| normalize_status(Tracker::Anilist, Some(raw), false);
        assert_eq!(map("CURRENT"), Some(ListStatus::Watching));
        assert_eq!(map("PLANNING"), Some(ListStatus::Planning));
        assert_eq!(map("COMPLETED"), Some(ListStatus::Completed));
        assert_eq!(map("PAUSED"), Some(ListStatus::Paused));
        assert_eq!(map("DROPPED"), Some(ListStatus::Dropped));
        assert_eq!(map("REPEATING"), Some(ListStatus::Repeating));
    }

    /// MAL has no rewatching status of its own: it keeps `watching` and
    /// raises a flag, and the flag is what makes the two trackers speak the
    /// same vocabulary.
    #[test]
    fn mal_statuses_map_and_a_rewatch_is_promoted() {
        let map = |raw| normalize_status(Tracker::Mal, Some(raw), false);
        assert_eq!(map("watching"), Some(ListStatus::Watching));
        assert_eq!(map("plan_to_watch"), Some(ListStatus::Planning));
        assert_eq!(map("completed"), Some(ListStatus::Completed));
        assert_eq!(map("on_hold"), Some(ListStatus::Paused));
        assert_eq!(map("dropped"), Some(ListStatus::Dropped));
        assert_eq!(
            normalize_status(Tracker::Mal, Some("watching"), true),
            Some(ListStatus::Repeating)
        );
        // The flag only promotes what is being watched.
        assert_eq!(
            normalize_status(Tracker::Mal, Some("completed"), true),
            Some(ListStatus::Completed)
        );
    }

    /// Nothing, and nothing either tracker has ever sent, is no status
    /// rather than a guess at one. AniList's vocabulary is not MAL's, so
    /// neither tracker reads the other's words.
    #[test]
    fn an_unknown_status_is_none() {
        for t in [Tracker::Anilist, Tracker::Mal] {
            assert_eq!(normalize_status(t, None, false), None);
            assert_eq!(normalize_status(t, Some(""), false), None);
            assert_eq!(normalize_status(t, Some("REWATCHING"), false), None);
        }
        assert_eq!(
            normalize_status(Tracker::Anilist, Some("watching"), false),
            None
        );
        assert_eq!(normalize_status(Tracker::Mal, Some("CURRENT"), false), None);
    }

    #[test]
    fn a_cache_inside_the_window_is_fresh_and_one_past_it_is_not() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        assert!(!is_fresh(None, now));
        assert!(is_fresh(Some(now - Duration::from_secs(1)), now));
        assert!(is_fresh(
            Some(now - PROGRESS_FRESH + Duration::from_secs(1)),
            now
        ));
        assert!(!is_fresh(Some(now - PROGRESS_FRESH), now));
        assert!(!is_fresh(Some(now - Duration::from_secs(600)), now));
        // A clock that moved back is not a reason to fetch again.
        assert!(is_fresh(Some(now + Duration::from_secs(60)), now));
    }

    /// A provider count is unsigned and small; anything else is nought.
    #[test]
    fn a_count_survives_whatever_a_provider_sends() {
        assert_eq!(as_count(None), 0);
        assert_eq!(as_count(Some(12)), 12);
        assert_eq!(as_count(Some(u64::MAX)), 0);
    }
}
