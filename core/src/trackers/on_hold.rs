//! On hold: a Watching entry nobody has touched for `on_hold_days` days
//! moves to the tracker's On hold list, AniList's Paused and MAL's On Hold.
//!
//! Touched is anything that dates the entry. Either tracker's own stamp on
//! it counts (a mark from here or from the phone, an edit on the website),
//! and so does a play here: a view, a completion or a resume point on a
//! series matched to it. An episode past the entry's progress coming out
//! restarts the clock too, so a show back from a break is not moved the day
//! it returns, and a show waiting on its next episode is never moved at
//! all: there is nothing on it to forget yet. An entry nothing dates is left
//! alone rather than guessed at.
//!
//! The sweep runs at the end of a tracker refresh (the launch's, a Sync, a
//! new connection), so it decides on a list fetched inside the five minute
//! window, and when the preferences switch it on. Nothing here is on a
//! timer. It sends the
//! status alone, one entry at a time through the limiter, and leaves one
//! line in the activity log for the whole pass. Marking an episode puts an
//! entry back on Watching, which every mark in `writes` already does.

use std::collections::BTreeSet;
use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::library::cards;
use crate::metadata::fetch::message_of;
use crate::prefs;
use crate::time;
use crate::trackers::{NO_TOKEN, accounts, cache, writes};

const DAY: i64 = 86_400;

/// The days a preference may ask for. Under a week would move a show the
/// user skipped one weekend; over a year is the same as never.
pub const MIN_DAYS: u32 = 7;
pub const MAX_DAYS: u32 = 365;

/// AniList's words for a media still coming out: airing, on a break, or not
/// started yet.
const STILL_COMING: [&str; 3] = ["RELEASING", "HIATUS", "NOT_YET_RELEASED"];

// The rule -------------------------------------------------------------------

/// What the sweep reads about one Watching entry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    /// The entry's own progress on the list.
    pub progress: u32,
    /// The published episode count, when there is one.
    pub total: Option<u32>,
    /// Whether the media is still coming out. Only read when the next
    /// episode's date is not known.
    pub still_coming: bool,
    /// The latest stamp either tracker's list carries for the media.
    pub tracker_touched: Option<i64>,
    /// The latest play here on a series matched to it.
    pub played: Option<i64>,
    /// When the episode after `progress` came out, or comes out.
    pub next_out: Option<i64>,
}

/// The instant an entry's untouched clock runs from, or `None` when the
/// entry is not waiting on the user at all.
pub fn idle_since(f: &Facts, now: i64) -> Option<i64> {
    // Every episode seen: whatever the list says, nothing is left to watch.
    if f.total.is_some_and(|total| f.progress >= total) {
        return None;
    }
    // Nothing dates the entry, so nothing says it was forgotten.
    let touched = f.tracker_touched.max(f.played)?;
    match f.next_out {
        // Waiting on the next episode is not forgetting the show.
        Some(at) if at > now => None,
        // Out: the clock runs from the later of the touch and the episode.
        Some(at) => Some(touched.max(at)),
        // Still coming out and no date for the next episode: whether there
        // is anything to watch at all is not known.
        None if f.still_coming => None,
        None => Some(touched),
    }
}

/// Whether an entry has sat untouched for more than `days` days.
pub fn is_due(f: &Facts, now: i64, days: u32) -> bool {
    idle_since(f, now).is_some_and(|since| now - since > i64::from(days) * DAY)
}

/// Whether a change to the preferences owes a sweep now: On hold switched
/// on. A change to the days waits for the next refresh, so a slider passing
/// through a short value on its way somewhere never moves anything.
pub fn wants_sweep(before: &Preferences, after: &Preferences) -> bool {
    after.on_hold_auto && !before.on_hold_auto
}

/// The days a preference asks for, inside the bounds the sweep keeps.
fn bounded(days: u32) -> u32 {
    days.clamp(MIN_DAYS, MAX_DAYS)
}

// The read -------------------------------------------------------------------

/// One entry the sweep moves, with the name and the idle days the debug
/// line reads.
#[derive(Clone, Debug, PartialEq)]
pub struct Due {
    pub media_id: u64,
    pub title: String,
    pub idle_days: i64,
}

/// Every Watching entry on one tracker's list, with the media row behind it
/// (by id on AniList, by the MAL id a media carries on MAL) and the other
/// tracker's stamp for the same media.
const WATCHING_SQL: &str = "SELECT t.media_id, t.progress, t.updated_at,
        m.id, m.mal_id, m.episodes, m.status, COALESCE(m.title_english, m.title_romaji),
        o.updated_at
     FROM tracker_entries t
     LEFT JOIN anilist_media m ON m.id = CASE WHEN ?1 = 'anilist' THEN t.media_id
         ELSE (SELECT a.id FROM anilist_media a WHERE a.mal_id = t.media_id ORDER BY a.id LIMIT 1) END
     LEFT JOIN tracker_entries o
         ON o.tracker = CASE WHEN ?1 = 'anilist' THEN 'mal' ELSE 'anilist' END
        AND o.media_id = CASE WHEN ?1 = 'anilist' THEN m.mal_id ELSE m.id END
     WHERE t.tracker = ?1 AND t.status = ?2";

/// The latest play here on any series matched to the media, by either id.
const PLAYED_SQL: &str = "SELECT MAX(at) FROM (
        SELECT v.at FROM views v JOIN series s ON s.id = v.series_id WHERE s.anilist_id = ?1 OR s.mal_id = ?2
        UNION ALL
        SELECT c.at FROM completed c JOIN series s ON s.id = c.series_id WHERE s.anilist_id = ?1 OR s.mal_id = ?2
        UNION ALL
        SELECT r.at FROM resume_points r JOIN series s ON s.id = r.series_id WHERE s.anilist_id = ?1 OR s.mal_id = ?2)";

/// A hidden series is absent from every tracker write, this one included.
const HIDDEN_SQL: &str =
    "SELECT EXISTS (SELECT 1 FROM series WHERE hidden = 1 AND (anilist_id = ?1 OR mal_id = ?2))";

const NEXT_OUT_SQL: &str =
    "SELECT aired_at FROM anilist_episodes WHERE anilist_id = ?1 AND number = ?2";

/// The entries on one tracker's list that are due, longest idle first.
pub fn due(conn: &Connection, t: Tracker, now: i64, days: u32) -> Result<Vec<Due>, CoreError> {
    struct Row {
        media_id: u64,
        progress: u32,
        updated_at: Option<i64>,
        anilist_id: Option<i64>,
        mal_id: Option<i64>,
        episodes: Option<i64>,
        status: Option<String>,
        title: Option<String>,
        other_updated_at: Option<i64>,
    }
    let rows: Vec<Row> = {
        let mut stmt = conn.prepare(WATCHING_SQL)?;
        let rows = stmt.query_map(params![t.as_str(), ListStatus::Watching.as_str()], |r| {
            Ok(Row {
                media_id: r.get::<_, i64>(0)? as u64,
                progress: u32::try_from(r.get::<_, i64>(1)?).unwrap_or(0),
                updated_at: r.get(2)?,
                anilist_id: r.get(3)?,
                mal_id: r.get(4)?,
                episodes: r.get(5)?,
                status: r.get(6)?,
                title: r.get(7)?,
                other_updated_at: r.get(8)?,
            })
        })?;
        rows.collect::<Result<_, _>>()?
    };

    let mut played = conn.prepare(PLAYED_SQL)?;
    let mut hidden = conn.prepare(HIDDEN_SQL)?;
    let mut next_out = conn.prepare(NEXT_OUT_SQL)?;
    let mut out: Vec<Due> = Vec::new();
    for row in rows {
        // Both ids the media goes by, whichever list this is: a series can
        // be matched by either.
        let (anilist_id, mal_id) = match t {
            Tracker::Anilist => (Some(row.media_id as i64), row.mal_id),
            Tracker::Mal => (row.anilist_id, Some(row.media_id as i64)),
        };
        if hidden.query_row(params![anilist_id, mal_id], |r| r.get::<_, i64>(0))? == 1 {
            continue;
        }
        let next = match anilist_id {
            Some(id) => next_out
                .query_row(params![id, i64::from(row.progress) + 1], |r| {
                    r.get::<_, Option<i64>>(0)
                })
                .optional()?
                .flatten(),
            None => None,
        };
        let facts = Facts {
            progress: row.progress,
            // Nought is AniList not knowing yet, the same as no count.
            total: row
                .episodes
                .and_then(|n| u32::try_from(n).ok())
                .filter(|n| *n > 0),
            still_coming: row
                .status
                .as_deref()
                .is_some_and(|s| STILL_COMING.contains(&s)),
            tracker_touched: row.updated_at.max(row.other_updated_at),
            played: played.query_row(params![anilist_id, mal_id], |r| r.get(0))?,
            next_out: next,
        };
        if !is_due(&facts, now, days) {
            continue;
        }
        let since = idle_since(&facts, now).unwrap_or(now);
        out.push(Due {
            media_id: row.media_id,
            title: row
                .title
                .unwrap_or_else(|| format!("{} {}", t.label(), row.media_id)),
            idle_days: (now - since) / DAY,
        });
    }
    out.sort_by_key(|d| std::cmp::Reverse(d.idle_days));
    Ok(out)
}

// The job --------------------------------------------------------------------

/// After a refresh that fetched a list: the sweep, when On hold is on. A
/// preference that cannot be read is no reason to fail the refresh, so it
/// is a debug line and no sweep.
pub(crate) async fn after_refresh(core: &Arc<Core>) {
    match core.store.write_async(|c| prefs::load_preferences(c)).await {
        Ok(p) if p.on_hold_auto => {
            start(core);
        }
        Ok(_) => {}
        Err(e) => tracing::debug!("On hold: the preferences could not be read: {e}"),
    }
}

/// What a change to the preferences calls when `wants_sweep` says so. The
/// sweep decides on a fresh list or not at all: at once when every
/// connected tracker's list is inside the window, else behind a refresh,
/// whose end starts it. The preferences are saved by then, so a failed
/// read here is a warning and a sweep that waits for the next refresh,
/// never a failed call.
pub(crate) fn kick(core: &Arc<Core>) {
    let now = time::now();
    let read = core.store.read(|c| {
        let (mut connected, mut stale) = (false, false);
        for t in [Tracker::Anilist, Tracker::Mal] {
            if let Some(row) = accounts::load_row(c, t)?
                && row.connected_at.is_some()
            {
                connected = true;
                stale |= !cache::is_fresh(row.progress_fetched_at, now);
            }
        }
        Ok((connected, stale))
    });
    match read {
        Ok((false, _)) => {}
        Ok((true, true)) => {
            cache::start_refresh(core, None, false);
        }
        Ok((true, false)) => {
            start(core);
        }
        Err(e) => tracing::warn!("On hold: the tracker accounts could not be read: {e}"),
    }
}

/// Starts the sweep. One at a time: a start while one runs answers with
/// the running one's id.
pub fn start(core: &Arc<Core>) -> u64 {
    let owner = core.clone();
    core.jobs.clone().start(
        JobKind::OnHold,
        move |ctx| async move { run(owner, ctx).await },
    )
}

/// What one pass did on one tracker.
#[derive(Default)]
struct Tally {
    moved: Vec<(Tracker, usize)>,
    failed: usize,
    /// The first failure, in the words a user can act on.
    failure: Option<String>,
}

async fn run(core: Arc<Core>, ctx: Arc<JobCtx>) -> Result<Finished, CoreError> {
    let p = core
        .store
        .write_async(|c| prefs::load_preferences(c))
        .await?;
    if !p.on_hold_auto {
        return Ok(quiet("On hold is switched off"));
    }
    let days = bounded(p.on_hold_days);
    let mut tally = Tally::default();
    let mut touched: Vec<(Tracker, u64)> = Vec::new();

    for t in [Tracker::Anilist, Tracker::Mal] {
        ctx.checkpoint()?;
        let row = core
            .store
            .write_async(move |c| accounts::load_row(c, t))
            .await?
            .unwrap_or_default();
        if row.connected_at.is_none() {
            continue;
        }
        // A list past the window is not decided on: the sweep runs behind
        // a refresh, and one that failed leaves an old list standing.
        if !cache::is_fresh(row.progress_fetched_at, time::now()) {
            ctx.emit(
                Level::Debug,
                format!("{} list is not fresh, On hold leaves it", t.as_str()),
                EventBody::Notice,
            );
            continue;
        }
        let now = time::now_secs();
        let due = core
            .store
            .write_async(move |c| due(c, t, now, days))
            .await?;
        if due.is_empty() {
            continue;
        }
        let token = match accounts::access_token(&core, t).await {
            Ok(Some(token)) => token,
            Ok(None) => {
                tally.failed += due.len();
                tally
                    .failure
                    .get_or_insert_with(|| format!("{} has {NO_TOKEN}", t.label()));
                continue;
            }
            Err(e) => {
                tally.failed += due.len();
                tally
                    .failure
                    .get_or_insert_with(|| writes::sanitize_error(t, &e));
                continue;
            }
        };
        let label = format!("Moving to On hold on {}", t.label());
        let mut moved = 0usize;
        for (i, entry) in due.iter().enumerate() {
            ctx.checkpoint()?;
            ctx.progress(i as u64, Some(due.len() as u64), &label);
            match move_one(&core, t, entry.media_id, &token).await {
                Ok(()) => {
                    moved += 1;
                    touched.push((t, entry.media_id));
                    ctx.emit(
                        Level::Debug,
                        format!(
                            "{} {} to On hold, untouched {} days (mediaId {})",
                            t.as_str(),
                            entry.title,
                            entry.idle_days,
                            entry.media_id
                        ),
                        EventBody::Notice,
                    );
                }
                Err(e) => {
                    tally.failed += 1;
                    ctx.emit(
                        Level::Debug,
                        format!(
                            "{} On hold failed for {} (mediaId {}): {}",
                            t.as_str(),
                            entry.title,
                            entry.media_id,
                            message_of(&e)
                        ),
                        EventBody::Notice,
                    );
                    tally
                        .failure
                        .get_or_insert_with(|| writes::sanitize_error(t, &e));
                    // A rate limit the limiter already waited out, or an
                    // account the tracker refuses, fails every entry after
                    // it the same way: the rest wait for the next pass.
                    if gives_up(&e) {
                        tally.failed += due.len() - i - 1;
                        break;
                    }
                }
            }
        }
        ctx.progress(due.len() as u64, Some(due.len() as u64), &label);
        if moved > 0 {
            tally.moved.push((t, moved));
        }
    }

    if !touched.is_empty() {
        let images_dir = core.paths.images_dir();
        let cards = core
            .store
            .write_async(move |c| {
                let ids = series_for(c, &touched)?;
                cards::cards_for(c, &images_dir, &ids)
            })
            .await?;
        ctx.changed_all(cards);
    }
    Ok(finished(&tally, days))
}

/// One entry to Paused on one tracker, and the cache after it.
async fn move_one(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    token: &str,
) -> Result<(), CoreError> {
    let stamp = writes::send_status(core, t, media_id, token, ListStatus::Paused).await?;
    writes::record_status(core, t, media_id, ListStatus::Paused, stamp).await
}

fn gives_up(e: &CoreError) -> bool {
    matches!(
        e,
        CoreError::Provider {
            status: Some(401 | 403 | 429),
            ..
        }
    )
}

/// Every library series matched to a media the sweep moved.
fn series_for(conn: &Connection, moved: &[(Tracker, u64)]) -> Result<Vec<u64>, CoreError> {
    let mut ids: BTreeSet<u64> = BTreeSet::new();
    for (t, media_id) in moved {
        let sql = match t {
            Tracker::Anilist => "SELECT id FROM series WHERE anilist_id = ?1",
            Tracker::Mal => "SELECT id FROM series WHERE mal_id = ?1",
        };
        let mut stmt = conn.prepare_cached(sql)?;
        let rows = stmt.query_map(params![*media_id as i64], |r| r.get::<_, i64>(0))?;
        for row in rows {
            ids.insert(row? as u64);
        }
    }
    Ok(ids.into_iter().collect())
}

fn quiet(message: &str) -> Finished {
    Finished {
        level: Level::Debug,
        message: message.to_string(),
        body: EventBody::Notice,
    }
}

/// The pass's one line: what moved where and after how long, then what
/// could not move and why.
fn finished(tally: &Tally, days: u32) -> Finished {
    let moved: usize = tally.moved.iter().map(|(_, n)| n).sum();
    if moved == 0 && tally.failed == 0 {
        return quiet("Nothing to move to On hold");
    }
    let mut message = String::new();
    if moved > 0 {
        let wheres: Vec<String> = tally
            .moved
            .iter()
            .enumerate()
            .map(|(i, (t, n))| {
                if i == 0 {
                    format!("{n} series to On hold on {}", t.label())
                } else {
                    format!("{n} on {}", t.label())
                }
            })
            .collect();
        message.push_str(&format!(
            "Moved {}, untouched for {days} days",
            wheres.join(" and ")
        ));
    }
    if tally.failed > 0 {
        if !message.is_empty() {
            message.push_str(". ");
        }
        let why = tally.failure.as_deref().unwrap_or("see the activity log");
        message.push_str(&format!(
            "{} series could not move to On hold: {why}",
            tally.failed
        ));
    }
    Finished {
        level: if tally.failed > 0 {
            Level::Warn
        } else {
            Level::Info
        },
        message,
        body: EventBody::Notice,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn days_ago(n: i64) -> Option<i64> {
        Some(NOW - n * DAY)
    }

    fn watching(touched: i64) -> Facts {
        Facts {
            progress: 3,
            total: Some(12),
            still_coming: false,
            tracker_touched: days_ago(touched),
            played: None,
            next_out: None,
        }
    }

    #[test]
    fn an_entry_untouched_past_the_days_is_due_and_one_inside_them_is_not() {
        assert!(is_due(&watching(61), NOW, 60));
        assert!(!is_due(&watching(59), NOW, 60));
        // "More than sixty days": the sixtieth day itself is not yet.
        assert!(!is_due(&watching(60), NOW, 60));
    }

    #[test]
    fn a_play_here_is_a_touch_as_much_as_the_trackers_stamp() {
        let f = Facts {
            played: days_ago(2),
            ..watching(200)
        };
        assert!(!is_due(&f, NOW, 60));
        // and the later of the two is the one that counts, whichever it is
        let f = Facts {
            played: days_ago(200),
            ..watching(2)
        };
        assert!(!is_due(&f, NOW, 60));
    }

    #[test]
    fn an_entry_nothing_dates_is_never_due() {
        let f = Facts {
            tracker_touched: None,
            played: None,
            ..watching(0)
        };
        assert_eq!(idle_since(&f, NOW), None);
        assert!(!is_due(&f, NOW, 7));
    }

    #[test]
    fn a_show_waiting_on_its_next_episode_is_never_due() {
        let f = Facts {
            next_out: Some(NOW + 3 * DAY),
            still_coming: true,
            ..watching(400)
        };
        assert!(!is_due(&f, NOW, 60));
    }

    #[test]
    fn an_episode_coming_out_restarts_the_clock() {
        // Caught up four months ago, the show came back from a break last
        // week: the week is what counts.
        let f = Facts {
            next_out: days_ago(7),
            still_coming: true,
            ..watching(120)
        };
        assert_eq!(idle_since(&f, NOW), days_ago(7));
        assert!(!is_due(&f, NOW, 60));
        // An episode out long before the last touch changes nothing.
        let f = Facts {
            next_out: days_ago(400),
            ..watching(90)
        };
        assert_eq!(idle_since(&f, NOW), days_ago(90));
        assert!(is_due(&f, NOW, 60));
    }

    #[test]
    fn a_show_still_coming_out_with_no_date_for_the_next_episode_is_left_alone() {
        let f = Facts {
            still_coming: true,
            next_out: None,
            ..watching(300)
        };
        assert!(!is_due(&f, NOW, 60));
        // Finished, or nothing known about the media at all: the plain rule.
        let f = Facts {
            total: None,
            ..watching(300)
        };
        assert!(is_due(&f, NOW, 60));
    }

    #[test]
    fn every_episode_seen_is_not_on_holds_business() {
        let f = Facts {
            progress: 12,
            ..watching(300)
        };
        assert!(!is_due(&f, NOW, 60));
    }

    #[test]
    fn only_switching_on_owes_a_sweep_now() {
        let off = Preferences {
            on_hold_auto: false,
            ..Preferences::default()
        };
        let on = Preferences::default();
        assert!(wants_sweep(&off, &on));
        assert!(!wants_sweep(&on, &off));
        assert!(!wants_sweep(&on, &on));
        // New days wait for the next refresh, shorter or longer.
        let shorter = Preferences {
            on_hold_days: 30,
            ..Preferences::default()
        };
        assert!(!wants_sweep(&on, &shorter));
        assert!(!wants_sweep(&shorter, &on));
    }

    #[test]
    fn the_days_stay_inside_their_bounds() {
        assert_eq!(bounded(0), MIN_DAYS);
        assert_eq!(bounded(60), 60);
        assert_eq!(bounded(10_000), MAX_DAYS);
    }

    #[test]
    fn the_line_says_what_moved_where_and_what_did_not() {
        let tally = Tally {
            moved: vec![(Tracker::Anilist, 16)],
            failed: 0,
            failure: None,
        };
        let f = finished(&tally, 60);
        assert_eq!(f.level, Level::Info);
        assert_eq!(
            f.message,
            "Moved 16 series to On hold on AniList, untouched for 60 days"
        );

        let tally = Tally {
            moved: vec![(Tracker::Anilist, 16), (Tracker::Mal, 3)],
            failed: 2,
            failure: Some("MAL rate limited, try again in a minute.".to_string()),
        };
        let f = finished(&tally, 60);
        assert_eq!(f.level, Level::Warn);
        assert_eq!(
            f.message,
            "Moved 16 series to On hold on AniList and 3 on MAL, untouched for 60 days. 2 series could not move to On hold: MAL rate limited, try again in a minute."
        );

        let f = finished(&Tally::default(), 60);
        assert_eq!(f.level, Level::Debug);
    }
}
