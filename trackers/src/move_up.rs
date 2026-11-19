//! Moving the list kept on this device up to a tracker (#213, #580). Once a
//! tracker is connected and its list read, the shell asks for a preview:
//! every series whose local status or score would change something on that
//! tracker, beside what the tracker has now. Nothing leaves the device until
//! the person confirms; the move then writes each one through the same
//! status and score writes the menus use, to that tracker alone. The rows
//! here stay as they are: the tracker's entry wins on every read from then
//! on, and an export still carries them.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rusqlite::{Connection, params};

use crate::Trackers;
use crate::{accounts, writes};
use anibeam_contract::*;
use anibeam_foundation::jobs::{Finished, JobCtx};
use anibeam_foundation::services::Services;

/// `PreviewListMove`: [`preview`] off the reader connection.
pub fn preview_call(services: &Services, t: Tracker) -> Result<Reply, CoreError> {
    Ok(Reply::ListMove {
        rows: services.store.read(|c| preview(c, t))?,
    })
}

/// What the preview shows for one series: what is kept here, and what the
/// tracker has now (none for a show not on its list).
///
/// A row is only offered while it is newer than the tracker's entry: once
/// the tracker has changed since (a move landed, or the person set it on
/// the phone), the tracker's word is the newer one and nothing here may
/// overwrite it. A show that any hidden folder shares a match with is left
/// out, the way every tracker write leaves it out.
pub fn preview(conn: &Connection, t: Tracker) -> Result<Vec<MoveRow>, CoreError> {
    offered(conn, t, None)
}

/// The preview's rows, or with `only` the one row of that series, when it
/// is still offered.
fn offered(conn: &Connection, t: Tracker, only: Option<u64>) -> Result<Vec<MoveRow>, CoreError> {
    let (id_column, bare) = match t {
        Tracker::Anilist => ("s.anilist_id", "anilist_id"),
        Tracker::Mal => ("s.mal_id", "mal_id"),
    };
    let lang = anibeam_foundation::prefs::load_preferences(conn)?.title_language;
    let sql = format!(
        "SELECT s.id, s.folder_name, l.status, l.score, e.status, e.score,
                m.title_romaji, m.title_english, m.title_native
         FROM local_entries l
         JOIN series s ON s.id = l.series_id
         LEFT JOIN anilist_media m ON m.id = s.anilist_id
         LEFT JOIN tracker_entries e ON e.tracker = ?1 AND e.media_id = {id_column} AND e.profile_id = 1
         WHERE {id_column} IS NOT NULL AND s.hidden = 0 AND s.missing_since IS NULL
           AND (e.updated_at IS NULL OR l.updated_at > e.updated_at)
           AND NOT EXISTS (SELECT 1 FROM series h WHERE h.hidden = 1 AND h.{bare} = {id_column})
           AND (?2 IS NULL OR s.id = ?2)
         ORDER BY s.folder_name COLLATE NOCASE"
    );
    let mut stmt = conn.prepare(&sql)?;
    let status = |raw: Option<String>| raw.and_then(|s| ListStatus::from_column(&s));
    let rows = stmt
        .query_map(params![t.as_str(), only.map(|s| s as i64)], |r| {
            let folder: String = r.get(1)?;
            let (romaji, english, native): (Option<String>, Option<String>, Option<String>) =
                (r.get(6)?, r.get(7)?, r.get(8)?);
            Ok(MoveRow {
                series: r.get::<_, i64>(0)? as u64,
                title: anibeam_foundation::titles::resolve(
                    lang,
                    romaji.as_deref(),
                    english.as_deref(),
                    native.as_deref(),
                    &folder,
                ),
                status: status(r.get(2)?),
                score: r.get(3)?,
                tracker_status: status(r.get(4)?),
                tracker_score: r.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows.into_iter().filter(changes_something).collect())
}

/// Whether moving this row would change the tracker: a status it does not
/// have, or a score it does not have. Watching here against a rewatch there
/// is the same list, and a move must not end the rewatch. A score alone,
/// for a show not on the tracker's list, would put the show on a list the
/// person never picked, so it waits until it has a status.
fn changes_something(row: &MoveRow) -> bool {
    let same_list = row.status == row.tracker_status
        || (row.status == Some(ListStatus::Watching)
            && row.tracker_status == Some(ListStatus::Repeating));
    let status_changes = row.status.is_some() && !same_list;
    let score_changes = row.score.is_some()
        && row.score != row.tracker_score
        && (row.status.is_some() || row.tracker_status.is_some());
    status_changes || score_changes
}

/// Starts the move: refused at once when the tracker is not connected, and
/// when the person picked nothing.
pub fn start(trackers: &Arc<Trackers>, t: Tracker, series: Vec<u64>) -> Result<u64, CoreError> {
    if series.is_empty() {
        return Err(CoreError::invalid(
            "series",
            anibeam_contract::i18n::tr("core-trackers-move-none-picked", &[]),
        ));
    }
    if trackers
        .services
        .store
        .read(|c| accounts::connected(c, t))?
        .is_none()
    {
        return Err(CoreError::NotConnected { tracker: t });
    }
    // Each pick's place in its entry's write line is taken now, in the
    // order picked, so a write to one of them asked for after the move
    // waits until the move has written it, and one asked for before goes
    // first (#1136).
    let mut seen = HashSet::new();
    let picked: Vec<u64> = series.into_iter().filter(|s| seen.insert(*s)).collect();
    let mut turns = HashMap::new();
    trackers.services.store.read(|c| {
        for &s in &picked {
            // A series gone already is the job's to count as failed.
            let Ok(target) = writes::targets(c, s) else {
                continue;
            };
            if let Some(id) = target.id_for(t) {
                turns.insert(s, (id, trackers.write_line.join_entries(vec![(t, id)])));
            }
        }
        Ok(())
    })?;
    let owner = trackers.clone();
    let jobs = owner.services.jobs.clone();
    Ok(jobs.start(JobKind::MoveLists, move |ctx| async move {
        run(owner, ctx, t, picked, turns).await
    }))
}

/// Each picked series in turn, read afresh from the preview once its turn
/// has come, so a row the tracker already matches, one a write ahead of it
/// in the line made the tracker's newer, or one that left the list kept
/// here, sends nothing. A status goes first and a score second, each through the write
/// the menus use, to this tracker alone; one the tracker refuses is that
/// series' failure, never the job's. The picks go in the order picked, the
/// order their places in the write line were taken, each waiting its turn
/// and giving it up once written.
async fn run(
    trackers: Arc<Trackers>,
    ctx: Arc<JobCtx>,
    t: Tracker,
    picked: Vec<u64>,
    mut turns: HashMap<u64, (u64, writes::Turn)>,
) -> Result<Finished, CoreError> {
    let order: HashMap<u64, usize> = picked.iter().enumerate().map(|(i, s)| (*s, i)).collect();
    let mut rows: Vec<MoveRow> = trackers
        .services
        .store
        .write_async(move |c| preview(c, t))
        .await?
        .into_iter()
        .filter(|r| order.contains_key(&r.series))
        .collect();
    rows.sort_by_key(|r| order[&r.series]);
    // A pick the move will not write needs no place.
    let kept: HashSet<u64> = rows.iter().map(|r| r.series).collect();
    turns.retain(|s, _| kept.contains(s));
    let user_id = trackers
        .services
        .store
        .write_async(move |c| accounts::load_row(c, t))
        .await?
        .and_then(|row| row.user_id);
    let total = rows.len() as u64;
    let (mut moved, mut failed) = (0u32, 0u32);
    for (n, listed) in rows.iter().enumerate() {
        ctx.checkpoint()?;
        ctx.progress(n as u64, Some(total), &listed.title);
        let series = listed.series;
        let mut turn = turns.remove(&series);
        if let Some((_, turn)) = turn.as_mut() {
            turn.come().await;
        }
        let Some(row) = trackers
            .services
            .store
            .write_async(move |c| offered(c, t, Some(series)))
            .await?
            .pop()
        else {
            continue;
        };
        // A series gone since the preview is that series' failure alone.
        let Ok(target) = trackers
            .services
            .store
            .write_async(move |c| writes::targets(c, series))
            .await
        else {
            failed += 1;
            continue;
        };
        let Some(media_id) = target.id_for(t) else {
            continue;
        };
        // Matched to another show since the move was asked for: that
        // show's line is the one to wait in. The old place goes first, so
        // the move never waits while holding one.
        let _rematched = if turn.as_ref().is_none_or(|(id, _)| *id != media_id) {
            drop(turn.take());
            let mut place = trackers.write_line.join_entries(vec![(t, media_id)]);
            place.come().await;
            Some(place)
        } else {
            None
        };
        // A status first, a score after. Whatever landed counts the show as
        // moved; a part the tracker refused says so in the log on its own.
        let mut landed = false;
        let mut refused = false;
        let rewatch = row.status == Some(ListStatus::Watching)
            && row.tracker_status == Some(ListStatus::Repeating);
        if let Some(status) = row
            .status
            .filter(|s| Some(*s) != row.tracker_status && !rewatch)
        {
            let action = writes::Action::Status { status };
            match sent(&trackers, &ctx, t, &target, media_id, user_id, action).await {
                true => landed = true,
                false => refused = true,
            }
        }
        if let Some(score) = row.score.filter(|s| Some(*s) != row.tracker_score)
            && !refused
        {
            let action = writes::Action::Score { score };
            match sent(&trackers, &ctx, t, &target, media_id, user_id, action).await {
                true => landed = true,
                false => refused = true,
            }
        }
        if landed {
            // The tracker now has the newer word, whether or not its reply
            // carried a stamp, so the preview never offers this row again.
            let now = anibeam_foundation::time::now_secs();
            trackers
                .services
                .store
                .write_async(move |c| {
                    c.execute(
                        "UPDATE tracker_entries SET updated_at = MAX(COALESCE(updated_at, 0), ?3)
                         WHERE tracker = ?1 AND media_id = ?2 AND profile_id = 1",
                        params![t.as_str(), media_id as i64, now],
                    )?;
                    Ok(())
                })
                .await?;
            moved += 1;
        } else if refused {
            failed += 1;
        }
        if let Ok(Some(card)) = anibeam_metadata::apply::card_for(&trackers.services, series).await
        {
            ctx.changed(card);
        }
    }
    let site = super::oauth::site_name(t);
    let plain = match (moved, failed) {
        (0, 0) => None,
        (m, 0) => Some(anibeam_contract::words!(
            "core-trackers-moved-up",
            count = m,
            site = site
        )),
        (m, f) => Some(anibeam_contract::words!(
            "core-trackers-moved-up-some-refused",
            moved = m,
            failed = f,
            site = site
        )),
    };
    Ok(Finished {
        level: if failed > 0 && moved == 0 {
            Level::Warn
        } else {
            Level::Info
        },
        plain,
        message: format!("{} lists moved: {moved}, refused: {failed}", t.as_str()),
        body: EventBody::ListsMoved {
            tracker: t,
            moved,
            failed,
        },
    })
}

/// One write to one tracker, true when it landed. A failure says so in the
/// activity log, the way any tracker write's failure does, with the
/// tracker's own words sanitised, and the move goes on.
async fn sent(
    trackers: &Trackers,
    ctx: &Arc<JobCtx>,
    t: Tracker,
    target: &writes::Targets,
    media_id: u64,
    user_id: Option<u64>,
    action: writes::Action,
) -> bool {
    match writes::attempt(trackers, ctx, t, target, media_id, user_id, &action).await {
        Ok(outcome) => outcome.ok,
        Err(e) => {
            let site = super::oauth::site_name(t);
            ctx.say(
                Level::Error,
                anibeam_contract::words!(
                    "core-trackers-move-refused",
                    site = site,
                    show = &target.folder_name,
                    error = writes::sanitize_error(t, &e)
                ),
                format!("{} move of {} failed: {e}", t.as_str(), target.folder_name),
                EventBody::Notice,
            );
            false
        }
    }
}
