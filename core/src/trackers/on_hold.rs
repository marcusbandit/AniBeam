//! On hold: a Watching entry nobody has touched for `on_hold_days` days
//! moves to the tracker's On hold list, AniList's Paused and MAL's On Hold.
//!
//! Touched is anything that dates the entry. Either tracker's own stamp on
//! it counts (a mark from here or from the phone, an edit on the website),
//! except the other tracker's stamp on an entry already on hold, which is a
//! move rather than a touch; and so does a play here: a view, a completion
//! or a resume point on a series matched to it. An episode past the entry's
//! progress coming out restarts the clock too, so a show back from a break
//! is not moved the week it returns, and a show waiting for its next
//! episode is never moved at all: there is nothing on it to forget yet. An
//! entry nothing dates is left alone rather than guessed at.
//!
//! The sweep runs at the end of a tracker refresh (the launch's, a Sync, a
//! new connection), so it decides on a list fetched inside the five minute
//! window, and when the preferences switch it on. Nothing here is on a
//! timer. It picks every tracker's candidates before it writes anything,
//! then reads each one live and decides again just before sending the
//! status alone, one entry at a time through the limiter. One line in the
//! activity log covers the pass.
//!
//! The way back is a play: a view here of a series on hold puts it on
//! Watching again (`played`), and a mark does too, as every mark in
//! `writes` already does. The phone never runs any of this. It has no
//! switch for it, and it only knows its own plays, so a show watched on the
//! desktop would read as untouched there.

use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::library::cards;
use crate::metadata::apply::card_for;
use crate::metadata::fetch::message_of;
use crate::prefs;
use crate::time;
use crate::trackers::{NO_TOKEN, accounts, cache, writes};

const DAY: i64 = 86_400;
const WEEK: i64 = 7 * DAY;

/// The days a preference may ask for. Under a week would move a show the
/// user skipped one weekend; over a year is the same as never.
pub const MIN_DAYS: u32 = 7;
pub const MAX_DAYS: u32 = 365;

/// Whether On hold runs in this build at all: the desktop's, never the
/// phone's (the module comment says why).
const RUNS_HERE: bool = !cfg!(feature = "phone");

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
    /// The latest stamp either tracker carries for the media.
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

/// Whether a provider's airing word says the media is still coming out:
/// airing, on a break, or not started yet.
pub fn still_coming(status: &str) -> bool {
    matches!(
        AiringStatus::from_provider(status),
        Some(AiringStatus::Releasing | AiringStatus::Hiatus | AiringStatus::NotYetReleased)
    )
}

/// When the episode after `progress` came out, or comes out. A schedule
/// row is exact and wins. Without one, the next broadcast says it: that
/// episode itself, or one further on, in which case the one after
/// `progress` is taken to have aired a week per episode before it. The
/// guess runs late across a break in the run, which only ever moves less.
pub fn next_out(progress: u32, scheduled: Option<i64>, next: Option<(u32, i64)>) -> Option<i64> {
    if scheduled.is_some() {
        return scheduled;
    }
    let (episode, at) = next?;
    let after = progress.saturating_add(1);
    Some(if episode <= after {
        at
    } else {
        at - WEEK * i64::from(episode - after)
    })
}

/// Whether a change to the preferences owes a sweep now: On hold switched
/// on. A change to the days waits for the next refresh, so a slider passing
/// through a short value on its way somewhere never moves anything.
pub fn wants_sweep(before: &Preferences, after: &Preferences) -> bool {
    after.on_hold_auto && !before.on_hold_auto
}

/// The days a stored preference asks for, inside the bounds `SetPreferences`
/// keeps: a blob written some other way is held to them too.
fn bounded(days: u32) -> u32 {
    days.clamp(MIN_DAYS, MAX_DAYS)
}

// The read -------------------------------------------------------------------

/// One entry the pre-check found due, with what the live check needs.
#[derive(Clone, Debug, PartialEq)]
pub struct Due {
    pub media_id: u64,
    /// Both ids the media goes by, whichever list this is.
    pub anilist_id: Option<i64>,
    pub mal_id: Option<i64>,
    pub title: String,
    pub idle_days: i64,
    /// The cached stamp the pre-check read, so a write here since then
    /// shows as a moved stamp.
    pub updated_at: Option<i64>,
}

/// An entry as the tracker has it right now, read just before a status
/// write: the decision is made again on this, never on the list alone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Live {
    pub status: Option<ListStatus>,
    pub progress: u32,
    pub updated_at: Option<i64>,
    pub total: Option<u32>,
    pub still_coming: bool,
    /// The next broadcast's episode and time. AniList only.
    pub next: Option<(u32, i64)>,
}

/// Every Watching entry on one tracker's list, with the media row behind it
/// (by id on AniList, by the MAL id a media carries on MAL).
const WATCHING_SQL: &str = "SELECT t.media_id, t.progress, t.updated_at,
        m.id, m.mal_id, m.episodes, m.status, COALESCE(m.title_english, m.title_romaji)
     FROM tracker_entries t
     LEFT JOIN anilist_media m ON m.id = CASE WHEN ?1 = 'anilist' THEN t.media_id
         ELSE (SELECT a.id FROM anilist_media a WHERE a.mal_id = t.media_id ORDER BY a.id LIMIT 1) END
     WHERE t.tracker = ?1 AND t.status = ?2";

/// The other tracker's stamp for the same media, unless its entry is on
/// hold: that stamp is the move, quite possibly this sweep's own, and
/// counting it would keep the entry here from ever following.
const OTHER_SQL: &str = "SELECT updated_at FROM tracker_entries
     WHERE tracker = ?1 AND media_id = ?2 AND (status IS NULL OR status != 'paused')";

/// The latest play here on any series matched to the media, by either id.
const PLAYED_SQL: &str = "SELECT MAX(at) FROM (
        SELECT v.at FROM views v JOIN series s ON s.id = v.series_id WHERE s.anilist_id = ?1 OR s.mal_id = ?2
        UNION ALL
        SELECT c.at FROM completed c JOIN series s ON s.id = c.series_id WHERE s.anilist_id = ?1 OR s.mal_id = ?2
        UNION ALL
        SELECT r.at FROM resume_points r JOIN series s ON s.id = r.series_id WHERE s.anilist_id = ?1 OR s.mal_id = ?2)";

const SCHEDULED_SQL: &str =
    "SELECT aired_at FROM anilist_episodes WHERE anilist_id = ?1 AND number = ?2";

/// What the library and the other tracker say about one media: the other
/// tracker's stamp, the latest play here, and when the episode after
/// `progress` is scheduled.
struct Local {
    other_touched: Option<i64>,
    played: Option<i64>,
    scheduled: Option<i64>,
}

fn local(
    conn: &Connection,
    t: Tracker,
    anilist_id: Option<i64>,
    mal_id: Option<i64>,
    progress: u32,
) -> Result<Local, CoreError> {
    let (other, other_id) = match t {
        Tracker::Anilist => (Tracker::Mal, mal_id),
        Tracker::Mal => (Tracker::Anilist, anilist_id),
    };
    let other_touched = match other_id {
        Some(id) => conn
            .prepare_cached(OTHER_SQL)?
            .query_row(params![other.as_str(), id], |r| r.get::<_, Option<i64>>(0))
            .optional()?
            .flatten(),
        None => None,
    };
    let played = conn
        .prepare_cached(PLAYED_SQL)?
        .query_row(params![anilist_id, mal_id], |r| r.get(0))?;
    let scheduled = match anilist_id {
        Some(id) => conn
            .prepare_cached(SCHEDULED_SQL)?
            .query_row(params![id, i64::from(progress) + 1], |r| {
                r.get::<_, Option<i64>>(0)
            })
            .optional()?
            .flatten(),
        None => None,
    };
    Ok(Local {
        other_touched,
        played,
        scheduled,
    })
}

/// A hidden series is absent from every tracker write, this one included,
/// by whichever id it carries.
fn hidden(
    conn: &Connection,
    anilist_id: Option<i64>,
    mal_id: Option<i64>,
) -> Result<bool, CoreError> {
    for (t, id) in [(Tracker::Anilist, anilist_id), (Tracker::Mal, mal_id)] {
        if let Some(id) = id
            && writes::is_hidden(conn, t, id as u64)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The entries on one tracker's cached list that look due, longest idle
/// first. Each one is read live and decided again before anything is
/// written (`move_one`), so this errs toward including an entry.
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
            })
        })?;
        rows.collect::<Result<_, _>>()?
    };

    let mut out: Vec<Due> = Vec::new();
    for row in rows {
        let (anilist_id, mal_id) = match t {
            Tracker::Anilist => (Some(row.media_id as i64), row.mal_id),
            Tracker::Mal => (row.anilist_id, Some(row.media_id as i64)),
        };
        if hidden(conn, anilist_id, mal_id)? {
            continue;
        }
        let here = local(conn, t, anilist_id, mal_id, row.progress)?;
        let facts = Facts {
            progress: row.progress,
            // Nought is AniList not knowing yet, the same as no count.
            total: row
                .episodes
                .and_then(|n| u32::try_from(n).ok())
                .filter(|n| *n > 0),
            still_coming: row.status.as_deref().is_some_and(still_coming),
            tracker_touched: row.updated_at.max(here.other_touched),
            played: here.played,
            next_out: here.scheduled,
        };
        if !is_due(&facts, now, days) {
            continue;
        }
        let since = idle_since(&facts, now).unwrap_or(now);
        out.push(Due {
            media_id: row.media_id,
            anilist_id,
            mal_id,
            title: row
                .title
                .unwrap_or_else(|| format!("{} {}", t.label(), row.media_id)),
            idle_days: (now - since) / DAY,
            updated_at: row.updated_at,
        });
    }
    out.sort_by_key(|d| std::cmp::Reverse(d.idle_days));
    Ok(out)
}

/// The live decision: the entry as the tracker has it now, with the
/// library's and the other tracker's word beside it, or why it stays.
fn decide(live: &Live, here: &Local, now: i64, days: u32) -> Result<(), &'static str> {
    if live.status != Some(ListStatus::Watching) {
        return Err("its status changed");
    }
    let facts = Facts {
        progress: live.progress,
        total: live.total,
        still_coming: live.still_coming,
        tracker_touched: live.updated_at.max(here.other_touched),
        played: here.played,
        next_out: next_out(live.progress, here.scheduled, live.next),
    };
    if is_due(&facts, now, days) {
        Ok(())
    } else {
        Err("it is not due on the live list")
    }
}

// The sweep ------------------------------------------------------------------

/// After a refresh that left a list inside the window: the sweep, when On
/// hold is on. A preference that cannot be read is no reason to fail the
/// refresh, so it is a debug line and no sweep.
pub(crate) async fn after_refresh(core: &Arc<Core>) {
    if !RUNS_HERE {
        return;
    }
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
/// whose end starts it. A refresh already running for one named tracker is
/// handed back by the job registry instead, and a stale list it does not
/// cover is left for the next refresh (a launch or a Sync). The
/// preferences are saved by then, so a failed read here is a warning and a
/// sweep that waits for the next refresh, never a failed call.
pub(crate) fn kick(core: &Arc<Core>) {
    if !RUNS_HERE {
        return;
    }
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

/// What one pass did.
#[derive(Default)]
struct Tally {
    moved: Vec<(Tracker, usize)>,
    failed: usize,
    /// The first failure, in the words a user can act on.
    failure: Option<String>,
}

/// One tracker's share of a pass, picked before anything is written.
struct Pass {
    tracker: Tracker,
    user_id: Option<u64>,
    due: Vec<Due>,
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

    // Every tracker's candidates first: a move on one tracker restamps its
    // entry, and read after that, the stamp would pass for a touch on the
    // other.
    let mut passes: Vec<Pass> = Vec::new();
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
        passes.push(Pass {
            tracker: t,
            user_id: row.user_id,
            due,
        });
    }

    let mut tally = Tally::default();
    'passes: for pass in passes {
        let t = pass.tracker;
        if pass.due.is_empty() {
            continue;
        }
        let token = match accounts::access_token(&core, t).await {
            Ok(Some(token)) => token,
            Ok(None) => {
                tally.failed += pass.due.len();
                tally
                    .failure
                    .get_or_insert_with(|| format!("{} has {NO_TOKEN}", t.label()));
                continue;
            }
            Err(e) => {
                tally.failed += pass.due.len();
                tally
                    .failure
                    .get_or_insert_with(|| writes::sanitize_error(t, &e));
                continue;
            }
        };
        let label = format!("Moving to On hold on {}", t.label());
        let total = pass.due.len();
        let mut moved = 0usize;
        for (i, entry) in pass.due.iter().enumerate() {
            ctx.progress(i as u64, Some(total as u64), &label);
            // A cancel, or the switch going off, ends the pass where it
            // stands, and what it already moved still gets its line.
            if let Err(e) = ctx.checkpoint() {
                cut_short(&ctx, &mut tally, t, moved, days);
                return Err(e);
            }
            let on = core
                .store
                .write_async(|c| prefs::load_preferences(c))
                .await?
                .on_hold_auto;
            if !on {
                cut_short(&ctx, &mut tally, t, moved, days);
                return Ok(quiet("On hold was switched off during the pass"));
            }
            match move_one(&core, &ctx, t, pass.user_id, &token, entry, days).await {
                Ok(None) => {
                    moved += 1;
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
                Ok(Some(why)) => ctx.emit(
                    Level::Debug,
                    format!(
                        "{} {} stays on Watching: {why} (mediaId {})",
                        t.as_str(),
                        entry.title,
                        entry.media_id
                    ),
                    EventBody::Notice,
                ),
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
                    // Only a refusal of this one entry lets the pass go
                    // on. A rate limit the limiter already waited out, a
                    // refused account, a server error or a network that
                    // stopped answering fails every entry after it the
                    // same way, so the rest wait for the next pass.
                    if !refuses_one(&e) {
                        tally.failed += total - i - 1;
                        if moved > 0 {
                            tally.moved.push((t, moved));
                        }
                        continue 'passes;
                    }
                }
            }
        }
        ctx.progress(total as u64, Some(total as u64), &label);
        if moved > 0 {
            tally.moved.push((t, moved));
        }
    }
    Ok(finished(&tally, days))
}

/// The line for a pass that ends before its last entry, with what it
/// moved up to there.
fn cut_short(ctx: &Arc<JobCtx>, tally: &mut Tally, t: Tracker, moved: usize, days: u32) {
    if moved > 0 {
        tally.moved.push((t, moved));
    }
    let line = finished(tally, days);
    if line.level > Level::Debug {
        ctx.emit(line.level, line.message, EventBody::Notice);
    }
}

/// One entry to Paused on one tracker, or why it stays. The cached row must
/// still be the one the pass read, the entry as the tracker has it now must
/// still be due, and only then does the status go. The cards move with it,
/// one entry at a time, so a pass cut short leaves no card behind its
/// list.
async fn move_one(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    t: Tracker,
    user_id: Option<u64>,
    token: &str,
    entry: &Due,
    days: u32,
) -> Result<Option<&'static str>, CoreError> {
    let media_id = entry.media_id;
    let cached = core
        .store
        .write_async(move |c| {
            c.query_row(
                "SELECT status, updated_at FROM tracker_entries WHERE tracker = ?1 AND media_id = ?2",
                params![t.as_str(), media_id as i64],
                |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<i64>>(1)?)),
            )
            .optional()
            .map_err(CoreError::from)
        })
        .await?;
    match cached {
        Some((Some(status), stamp))
            if status == ListStatus::Watching.as_str() && stamp == entry.updated_at => {}
        _ => return Ok(Some("it changed here during the pass")),
    }
    let Some(live) = writes::read_live(core, t, media_id, user_id, token).await? else {
        return Ok(Some("it is no longer on the list"));
    };
    let (anilist_id, mal_id, progress) = (entry.anilist_id, entry.mal_id, live.progress);
    let here = core
        .store
        .write_async(move |c| local(c, t, anilist_id, mal_id, progress))
        .await?;
    if let Err(why) = decide(&live, &here, time::now_secs(), days) {
        return Ok(Some(why));
    }
    let stamp = writes::send_status(core, t, media_id, token, ListStatus::Paused).await?;
    writes::record_status(core, t, media_id, ListStatus::Paused, stamp).await?;
    let images_dir = core.paths.images_dir();
    let cards = core
        .store
        .write_async(move |c| {
            let ids = series_for(c, t, media_id)?;
            cards::cards_for(c, &images_dir, &ids)
        })
        .await?;
    ctx.changed_all(cards);
    Ok(None)
}

/// A provider refusing this one entry: a bad request, an entry it cannot
/// find, a conflict. Anything else is the tracker or the network, and says
/// the same about every entry after it.
fn refuses_one(e: &CoreError) -> bool {
    matches!(
        e,
        CoreError::Provider {
            status: Some(400 | 404 | 409 | 422),
            ..
        }
    )
}

/// Every library series matched to one media on one tracker.
fn series_for(conn: &Connection, t: Tracker, media_id: u64) -> Result<Vec<u64>, CoreError> {
    let sql = match t {
        Tracker::Anilist => "SELECT id FROM series WHERE anilist_id = ?1",
        Tracker::Mal => "SELECT id FROM series WHERE mal_id = ?1",
    };
    let mut stmt = conn.prepare_cached(sql)?;
    let rows = stmt.query_map(params![media_id as i64], |r| r.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row? as u64);
    }
    Ok(ids)
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

// The way back ---------------------------------------------------------------

/// A view here of a series on hold: watching it again puts it back on
/// Watching, on every connected tracker whose cached entry is Paused. With
/// On hold switched on only. Called by the playback session once a play
/// counts as a view; whatever goes wrong is a debug line, never the
/// session's failure.
pub(crate) fn played(core: &Core, series: u64) {
    if !RUNS_HERE {
        return;
    }
    let paused = match core.store.read(|c| paused_entries(c, series)) {
        Ok(paused) => paused,
        Err(e) => {
            tracing::debug!("On hold: series {series} could not be read: {e}");
            return;
        }
    };
    if paused.is_empty() {
        return;
    }
    let Some(owner) = core.arc() else {
        return;
    };
    let jobs = owner.jobs.clone();
    jobs.start(JobKind::SetStatus, move |ctx| async move {
        back_to_watching(owner, ctx, series, paused).await
    });
}

/// The paused entries a play of this series takes back: by each tracker id
/// the match carries, while that tracker is connected, and never for a
/// hidden or missing series.
fn paused_entries(
    conn: &Connection,
    series: u64,
) -> Result<Vec<(Tracker, u64, Option<u64>)>, CoreError> {
    if !prefs::load_preferences(conn)?.on_hold_auto {
        return Ok(Vec::new());
    }
    let target = match writes::targets(conn, series) {
        Ok(target) => target,
        Err(CoreError::NotFound { .. }) => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut out = Vec::new();
    for (t, id) in [
        (Tracker::Anilist, target.anilist),
        (Tracker::Mal, target.mal),
    ] {
        let Some(id) = id else { continue };
        if writes::is_hidden(conn, t, id)? {
            return Ok(Vec::new());
        }
        let Some(row) = accounts::load_row(conn, t)? else {
            continue;
        };
        if row.connected_at.is_none() {
            continue;
        }
        let status: Option<String> = conn
            .query_row(
                "SELECT status FROM tracker_entries WHERE tracker = ?1 AND media_id = ?2",
                params![t.as_str(), id as i64],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        if status.as_deref() == Some(ListStatus::Paused.as_str()) {
            out.push((t, id, row.user_id));
        }
    }
    Ok(out)
}

/// The SetStatus job behind `played`: each entry read live first, so one
/// moved off hold somewhere else since the list was read is left alone.
async fn back_to_watching(
    core: Arc<Core>,
    ctx: Arc<JobCtx>,
    series: u64,
    paused: Vec<(Tracker, u64, Option<u64>)>,
) -> Result<Finished, CoreError> {
    let mut moved: Vec<Tracker> = Vec::new();
    let mut failure: Option<String> = None;
    for (t, media_id, user_id) in paused {
        ctx.checkpoint()?;
        let attempt = async {
            let token = accounts::access_token(&core, t)
                .await?
                .ok_or_else(|| accounts::tracker_error(t, NO_TOKEN))?;
            let live = writes::read_live(&core, t, media_id, user_id, &token).await?;
            if live.is_none_or(|l| l.status != Some(ListStatus::Paused)) {
                return Ok(false);
            }
            let stamp =
                writes::send_status(&core, t, media_id, &token, ListStatus::Watching).await?;
            writes::record_status(&core, t, media_id, ListStatus::Watching, stamp).await?;
            Ok::<bool, CoreError>(true)
        };
        match attempt.await {
            Ok(true) => moved.push(t),
            Ok(false) => {}
            Err(e) => {
                ctx.emit(
                    Level::Debug,
                    format!(
                        "{} back to Watching failed (mediaId {media_id}): {}",
                        t.as_str(),
                        message_of(&e)
                    ),
                    EventBody::Notice,
                );
                failure.get_or_insert_with(|| writes::sanitize_error(t, &e));
            }
        }
    }
    if let Some(card) = card_for(&core, series).await? {
        ctx.changed(card);
    }
    let name = core
        .store
        .write_async(move |c| {
            c.query_row(
                "SELECT folder_name FROM series WHERE id = ?1",
                params![series as i64],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(CoreError::from)
        })
        .await?
        .unwrap_or_else(|| format!("series {series}"));
    Ok(match (moved.is_empty(), failure) {
        (false, _) => Finished {
            level: Level::Info,
            message: format!(
                "{name} is back on Watching on {}",
                moved
                    .iter()
                    .map(|t| t.label())
                    .collect::<Vec<_>>()
                    .join(" and ")
            ),
            body: EventBody::Notice,
        },
        (true, Some(why)) => Finished {
            level: Level::Warn,
            message: format!("{name} could not go back on Watching: {why}"),
            body: EventBody::Notice,
        },
        (true, None) => quiet("Nothing to take off hold"),
    })
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
    fn the_next_episode_is_dated_by_its_schedule_row_else_by_the_next_broadcast() {
        let at = NOW + 2 * DAY;
        // A schedule row is exact, whatever the broadcast says.
        assert_eq!(next_out(3, days_ago(30), Some((9, at))), days_ago(30));
        // The next broadcast is the episode after the progress: waiting.
        assert_eq!(next_out(3, None, Some((4, at))), Some(at));
        // Five further on: aired five weeks before that broadcast.
        assert_eq!(next_out(3, None, Some((9, at))), Some(at - 5 * WEEK));
        // A list ahead of the schedule waits for the broadcast too.
        assert_eq!(next_out(9, None, Some((4, at))), Some(at));
        assert_eq!(next_out(3, None, None), None);
    }

    #[test]
    fn both_airing_words_count_as_still_coming() {
        for s in [
            "RELEASING",
            "HIATUS",
            "NOT_YET_RELEASED",
            "currently_airing",
            "not_yet_aired",
        ] {
            assert!(still_coming(s), "{s}");
        }
        for s in ["FINISHED", "CANCELLED", "finished_airing", ""] {
            assert!(!still_coming(s), "{s}");
        }
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
    fn only_a_refusal_of_one_entry_lets_a_pass_go_on() {
        let provider = |status| CoreError::Provider {
            provider: Provider::Anilist,
            status: Some(status),
            message: String::new(),
            retry_after: None,
        };
        for status in [400, 404, 409, 422] {
            assert!(refuses_one(&provider(status)), "{status}");
        }
        for status in [401, 403, 429, 500, 503] {
            assert!(!refuses_one(&provider(status)), "{status}");
        }
        assert!(!refuses_one(&CoreError::internal("timed out")));
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
