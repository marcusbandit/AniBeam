//! The watching list: what AniList says the user is part way through,
//! read off the tables every other page reads and refreshed behind the
//! page that asked for it.
//!
//! Three rules shape the file. The list is AniList's rather than the main
//! tracker's, by the watching tab's own decision, and a rewatch counts as
//! watching, so the page is the `CURRENT` and `REPEATING` entries and
//! nothing else. Every visit refreshes, the way Electron's
//! `WatchingPage.tsx` did, so there is no staleness window here at all:
//! the call answers off the cache at once and hands back the id of the job
//! filling it. And the cache is the database rather than a session's
//! memory, which is what Electron never had: the page paints the last list
//! before the first reply arrives, launch after launch.
//!
//! Carried from `getAnilistWatchingList` in
//! `src/main/handlers/trackerHandler.ts` and the page that drew it.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use rusqlite::{params, Connection, Transaction};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::Finished;
use crate::metadata::record::{self, StubWrite};
use crate::net::anilist::WATCHING_LIST_QUERY;
use crate::store::settings;
use crate::time;
use crate::trackers::accounts;
use crate::trackers::cache::normalize_status;
use crate::trackers::writes;

/// How long the list read has before it is given up on, the same window
/// the progress fetch gives itself: a page is waiting on this, so it gives
/// up well before the transport's own thirty seconds.
const FETCH_TIMEOUT: Duration = Duration::from_secs(15);

/// The account says it is connected but there is no token behind it,
/// which for AniList's implicit grant means the keyring lost the entry.
const NO_TOKEN: &str = "no access token stored, reconnect in Settings";

/// The title of last resort. A stub the store has only ever seen on this
/// list still draws a card, and an id reads better than an empty one.
fn fallback_title(anilist_id: u64) -> String {
    format!("AniList {anilist_id}")
}

// The read -------------------------------------------------------------------

/// The cached list. The two statuses bound into it are the whole page: a
/// rewatch counts as watching, and everything else belongs somewhere else.
/// `images` answers the poster, `series` answers whether the library owns
/// the media, and a series that owns it and hides it takes the entry off
/// the page rather than turning it into an external card.
const LIST_SQL: &str = "SELECT t.media_id, t.progress, t.status, t.score, t.updated_at,
        m.title_romaji, m.title_english, m.episodes, m.site_url, i.path,
        (SELECT s.id FROM series s WHERE s.anilist_id = t.media_id ORDER BY s.id LIMIT 1)
     FROM tracker_entries t
     LEFT JOIN anilist_media m ON m.id = t.media_id
     LEFT JOIN images i ON i.url = m.cover_url
     WHERE t.tracker = ?1 AND t.status IN (?2, ?3)
       AND NOT EXISTS (SELECT 1 FROM series s WHERE s.anilist_id = t.media_id AND s.hidden = 1)";

/// The next broadcast of everything on the list, in one query rather than
/// a subquery per row: the earliest scheduled episode of each media.
const AIRING_SQL: &str = "SELECT e.anilist_id, e.number, e.aired_at
     FROM anilist_episodes e
     WHERE e.aired_at > ?1
       AND e.anilist_id IN (SELECT media_id FROM tracker_entries WHERE tracker = ?2 AND status IN (?3, ?4))
     ORDER BY e.aired_at";

/// The whole page off the database: no request, no waiting, whatever the
/// last refresh left behind. Ordered by the tracker's own timestamp, most
/// recently updated first, with an entry AniList gave none for at the
/// bottom and ties settled by title.
pub fn list(conn: &Connection, images_dir: &Path) -> Result<WatchingList, CoreError> {
    let now = time::now_secs();
    let mut next_airing: HashMap<u64, Airing> = HashMap::new();
    {
        let mut stmt = conn.prepare(AIRING_SQL)?;
        let rows = stmt.query_map(
            params![now, Tracker::Anilist.as_str(), ListStatus::Watching.as_str(), ListStatus::Repeating.as_str()],
            |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, i64>(1)? as u32, r.get::<_, i64>(2)?)),
        )?;
        for row in rows {
            let (anilist_id, episode, at) = row?;
            // The query is in date order, so the first row for a media is
            // its next broadcast.
            next_airing.entry(anilist_id).or_insert(Airing { episode, at: time::from_secs(at) });
        }
    }

    let mut stmt = conn.prepare(LIST_SQL)?;
    let rows = stmt.query_map(
        params![Tracker::Anilist.as_str(), ListStatus::Watching.as_str(), ListStatus::Repeating.as_str()],
        |r| {
            let anilist_id = r.get::<_, i64>(0)? as u64;
            let updated_at: Option<i64> = r.get(4)?;
            let romaji: Option<String> = r.get(5)?;
            let english: Option<String> = r.get(6)?;
            let poster: Option<String> = r.get(9)?;
            Ok((
                updated_at,
                WatchingEntry {
                    anilist_id,
                    title: present(romaji).or_else(|| present(english)).unwrap_or_else(|| fallback_title(anilist_id)),
                    poster: local_path(images_dir, poster),
                    progress: r.get::<_, i64>(1)? as u32,
                    total: r.get::<_, Option<i64>>(7)?.map(|e| e as u32),
                    // AniList sent no timestamp for this entry, so it sorts
                    // last and the shell shows nothing for the epoch.
                    updated_at: updated_at.map_or(UNIX_EPOCH, time::from_secs),
                    owned: r.get::<_, Option<i64>>(10)?.map(|id| id as u64),
                    repeating: r.get::<_, Option<String>>(2)?.as_deref().and_then(ListStatus::from_column) == Some(ListStatus::Repeating),
                    site_url: r.get(8)?,
                    next_airing: next_airing.get(&anilist_id).cloned(),
                    score: r.get(3)?,
                },
            ))
        },
    )?;
    let mut entries = rows.collect::<Result<Vec<(Option<i64>, WatchingEntry)>, _>>()?;
    entries.sort_by(|(a_at, a), (b_at, b)| {
        a_at.is_none()
            .cmp(&b_at.is_none())
            .then_with(|| b_at.cmp(a_at))
            .then_with(|| a.title.cmp(&b.title))
    });

    Ok(WatchingList {
        entries: entries.into_iter().map(|(_, entry)| entry).collect(),
        fetched_at: settings::get::<i64>(conn, settings::WATCHING_FETCHED_AT)?.map(time::from_secs),
    })
}

/// The call. The cached list leaves at once; a connected AniList also
/// leaves with the id of the job filling it, which is the running one when
/// a refresh is already under way.
pub fn list_call(core: &Core) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let (list, connected) = core.store.read(|c| Ok((list(c, &images_dir)?, is_connected(c)?)))?;
    // The row is checked here rather than in `start_refresh`, so a page
    // opened with no account sees an answer instead of a refusal.
    let refreshing = match connected {
        true => {
            let owner = core.arc().ok_or_else(|| CoreError::internal("core is shutting down"))?;
            Some(start_refresh(&owner)?)
        }
        false => None,
    };
    Ok(Reply::Watching { list, refreshing })
}

fn is_connected(conn: &Connection) -> Result<bool, CoreError> {
    Ok(accounts::load_row(conn, Tracker::Anilist)?.is_some_and(|row| row.connected_at.is_some()))
}

/// A stored string with something in it. An empty title is no title.
fn present(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

/// A cached image's absolute path, and only while its file is still there:
/// a row pointing at nothing is the same as no row.
fn local_path(images_dir: &Path, relative: Option<String>) -> Option<String> {
    relative
        .map(|relative| images_dir.join(relative))
        .filter(|path| path.is_file())
        .map(|path| path.to_string_lossy().into_owned())
}

// The job --------------------------------------------------------------------

/// One entry as the refresh writes it: the row `tracker_entries` keeps,
/// the media row behind it, and the broadcast the query carried.
struct Row {
    media_id: u64,
    progress: u32,
    status: ListStatus,
    score: Option<f64>,
    updated_at: Option<i64>,
    media: StubWrite,
    next_airing: Option<(u32, i64)>,
}

/// Starts the RefreshWatching job, one at a time per kind: a second call
/// while one runs hands back the running job's id. Refuses only when there
/// is no account to ask, which the call itself has already ruled out.
pub fn start_refresh(core: &Arc<Core>) -> Result<u64, CoreError> {
    if !core.store.read(is_connected)? {
        return Err(CoreError::NotConnected { tracker: Tracker::Anilist });
    }
    let owner = core.clone();
    Ok(core.jobs.clone().start(JobKind::RefreshWatching, move |_| async move { run(owner).await }))
}

async fn run(core: Arc<Core>) -> Result<Finished, CoreError> {
    refresh(&core).await.map_err(failed)
}

async fn refresh(core: &Arc<Core>) -> Result<Finished, CoreError> {
    let account = core.store.write_async(move |c| accounts::load_row(c, Tracker::Anilist)).await?.unwrap_or_default();
    // The collection is asked for by user id, so an account whose profile
    // never carried one cannot be read at all.
    let user_id = account.user_id.ok_or_else(|| tracker_error("the account carries no user id"))?;
    let token = accounts::access_token(core, Tracker::Anilist).await?.ok_or_else(|| tracker_error(NO_TOKEN))?;
    let rows = fetch(core, user_id, &token).await?;

    let covers: Vec<String> = rows.iter().filter_map(|row| row.media.cover_url.clone()).collect();
    let now = time::now_secs();
    core.store.tx_async(move |tx| write(tx, &rows, now)).await?;
    // Every cover before the job reports, so the page's posters are local
    // files rather than urls the shell has to go and get.
    for (url, outcome) in core.images.ensure(&covers).await {
        if let Err(e) = outcome {
            tracing::debug!("the watching list's cover {url} was not fetched: {e}");
        }
    }

    let images_dir = core.paths.images_dir();
    let list = core.store.write_async(move |c| list(c, &images_dir)).await?;
    Ok(Finished {
        level: Level::Debug,
        message: format!("watching list refreshed: {} entries", list.entries.len()),
        body: EventBody::WatchingRefreshed { list },
    })
}

/// The list itself. AniList repeats one media across custom lists, so the
/// first copy of it is the one kept, and anything that is not being
/// watched belongs to another page.
async fn fetch(core: &Arc<Core>, user_id: u64, token: &str) -> Result<Vec<Row>, CoreError> {
    let data = tokio::time::timeout(
        FETCH_TIMEOUT,
        core.anilist.graphql(WATCHING_LIST_QUERY, serde_json::json!({ "userId": user_id }), Some(token)),
    )
    .await
    .map_err(|_| tracker_error(format!("AniList watching list timed out after {}ms", FETCH_TIMEOUT.as_millis())))??;

    let mut seen: HashSet<u64> = HashSet::new();
    let mut rows: Vec<Row> = Vec::new();
    for list in data["MediaListCollection"]["lists"].as_array().into_iter().flatten() {
        for entry in list["entries"].as_array().into_iter().flatten() {
            let media = &entry["media"];
            let Some(media_id) = media["id"].as_u64() else { continue };
            // Anything else on the account is another page's entry.
            let Some(status @ (ListStatus::Watching | ListStatus::Repeating)) =
                normalize_status(Tracker::Anilist, entry["status"].as_str(), false)
            else {
                continue;
            };
            if !seen.insert(media_id) {
                continue;
            }
            let title = &media["title"];
            let next = &media["nextAiringEpisode"];
            rows.push(Row {
                media_id,
                progress: as_count(entry["progress"].as_u64()),
                status,
                // Nought is AniList's unrated rather than a rating.
                score: entry["score"].as_f64().filter(|s| *s > 0.0),
                updated_at: entry["updatedAt"].as_i64().filter(|at| *at > 0),
                media: StubWrite {
                    id: media_id,
                    mal_id: media["idMal"].as_u64(),
                    media_type: None,
                    title_romaji: media_title(&title["romaji"]),
                    title_english: media_title(&title["english"]),
                    format: None,
                    status: None,
                    year: None,
                    cover_url: media["coverImage"]["large"].as_str().map(str::to_string),
                    site_url: media["siteUrl"].as_str().map(str::to_string),
                    episodes: as_u32(media["episodes"].as_u64()),
                    average_score: as_u32(media["averageScore"].as_u64()),
                },
                next_airing: as_u32(next["episode"].as_u64()).zip(next["airingAt"].as_i64()),
            });
        }
    }
    Ok(rows)
}

/// One entry's row, keeping the rewatch count: the watching query never
/// asks for `repeat`, and a number the progress sweep fetched must not be
/// zeroed by a page read.
const ENTRY_UPSERT: &str = "INSERT INTO tracker_entries (tracker, media_id, status, progress, score, repeat, updated_at, fetched_at)
     VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)
     ON CONFLICT(tracker, media_id) DO UPDATE SET
        status = excluded.status,
        progress = excluded.progress,
        score = excluded.score,
        repeat = tracker_entries.repeat,
        updated_at = excluded.updated_at,
        fetched_at = excluded.fetched_at";

/// The next broadcast as an episode row. The list carries no titles at
/// all, so a title already in the table is the best one there will ever
/// be and this must never replace it.
const AIRING_UPSERT: &str = "INSERT INTO anilist_episodes (anilist_id, number, title, aired_at) VALUES (?1, ?2, NULL, ?3)
     ON CONFLICT(anilist_id, number) DO UPDATE SET aired_at = excluded.aired_at";

/// The whole list in one transaction: a half-written page would draw
/// entries whose media row is not there yet.
fn write(tx: &Transaction, rows: &[Row], now: i64) -> Result<(), CoreError> {
    for row in rows {
        tx.execute(
            ENTRY_UPSERT,
            params![
                Tracker::Anilist.as_str(),
                row.media_id as i64,
                row.status.as_str(),
                i64::from(row.progress),
                row.score,
                row.updated_at,
                now,
            ],
        )?;
        // Before the episode row, which references it, and only ever
        // filling blanks: the row may be a fetched one, and the list's
        // thinner copy of a title must not replace it.
        record::write_stub(tx, &row.media)?;
        if let Some((episode, at)) = row.next_airing {
            tx.execute(AIRING_UPSERT, params![row.media_id as i64, i64::from(episode), at])?;
        }
    }
    settings::set(tx, settings::WATCHING_FETCHED_AT, &now)?;
    Ok(())
}

/// Whatever went wrong, in the one line a page can show. The job's failure
/// is the shell's only account of it, so a GraphQL blob never reaches it.
fn failed(e: CoreError) -> CoreError {
    let message = writes::sanitize_error(Tracker::Anilist, &e);
    match e {
        CoreError::Provider { provider, status, retry_after, .. } => CoreError::Provider { provider, status, message, retry_after },
        _ => tracker_error(message),
    }
}

fn tracker_error(message: impl Into<String>) -> CoreError {
    accounts::tracker_error(Tracker::Anilist, message)
}

/// A title off the reply, treating an empty string as absent: the stub
/// must not fill a blank column with nothing.
fn media_title(value: &serde_json::Value) -> Option<String> {
    value.as_str().filter(|t| !t.trim().is_empty()).map(str::to_string)
}

/// A count off a provider's JSON, which is unsigned and small: anything
/// missing or absurd is nought rather than a wrap-around.
fn as_count(value: Option<u64>) -> u32 {
    as_u32(value).unwrap_or(0)
}

fn as_u32(value: Option<u64>) -> Option<u32> {
    value.and_then(|v| u32::try_from(v).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_media_with_no_titles_at_all_is_named_by_its_id() {
        assert_eq!(fallback_title(154587), "AniList 154587");
        assert_eq!(present(Some("  ".to_string())), None);
        assert_eq!(present(Some("Frieren".to_string())).as_deref(), Some("Frieren"));
    }

    #[test]
    fn a_count_survives_whatever_a_provider_sends() {
        assert_eq!(as_count(None), 0);
        assert_eq!(as_count(Some(12)), 12);
        assert_eq!(as_count(Some(u64::MAX)), 0);
        assert_eq!(as_u32(None), None);
        assert_eq!(as_u32(Some(1122)), Some(1122));
        assert_eq!(as_u32(Some(u64::MAX)), None);
    }

    /// The provider's raw words reach the shell sanitised, whatever shape
    /// the failure took.
    #[test]
    fn a_failure_carries_one_readable_line() {
        let limited = CoreError::Provider {
            provider: Provider::Anilist,
            status: Some(429),
            message: "AniList rate limited".to_string(),
            retry_after: Some(9.0),
        };
        let out = failed(limited);
        assert!(
            matches!(&out, CoreError::Provider { status: Some(429), message, retry_after: Some(w), .. }
                if message == "AniList rate limited, try again in a minute." && *w == 9.0),
            "{out:?}"
        );
        let internal = failed(CoreError::internal("the writer thread is gone"));
        assert!(matches!(&internal, CoreError::Provider { provider: Provider::Anilist, .. }), "{internal:?}");
    }
}
