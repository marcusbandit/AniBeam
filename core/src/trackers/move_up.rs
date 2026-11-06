//! Moving the list kept on this device up to a tracker (#213, #580). Once a
//! tracker is connected and its list read, the shell asks for a preview:
//! every series whose local status or score would change something on that
//! tracker, beside what the tracker has now. Nothing leaves the device until
//! the person confirms; the move then writes each one through the same
//! status and score writes the menus use, to that tracker alone. The rows
//! here stay as they are: the tracker's entry wins on every read from then
//! on, and an export still carries them.

use std::sync::Arc;

use rusqlite::{Connection, params};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::trackers::{accounts, writes};

/// What the preview shows for one series: what is kept here, and what the
/// tracker has now (none for a show not on its list).
///
/// A row is only offered while it is newer than the tracker's entry: once
/// the tracker has changed since (a move landed, or the person set it on
/// the phone), the tracker's word is the newer one and nothing here may
/// overwrite it. A show that any hidden folder shares a match with is left
/// out, the way every tracker write leaves it out.
pub fn preview(conn: &Connection, t: Tracker) -> Result<Vec<MoveRow>, CoreError> {
    let (id_column, bare) = match t {
        Tracker::Anilist => ("s.anilist_id", "anilist_id"),
        Tracker::Mal => ("s.mal_id", "mal_id"),
    };
    let lang = crate::prefs::load_preferences(conn)?.title_language;
    let sql = format!(
        "SELECT s.id, s.folder_name, l.status, l.score, e.status, e.score,
                m.title_romaji, m.title_english, m.title_native
         FROM local_entries l
         JOIN series s ON s.id = l.series_id
         LEFT JOIN anilist_media m ON m.id = s.anilist_id
         LEFT JOIN tracker_entries e ON e.tracker = ?1 AND e.media_id = {id_column}
         WHERE {id_column} IS NOT NULL AND s.hidden = 0 AND s.missing_since IS NULL
           AND (e.updated_at IS NULL OR l.updated_at > e.updated_at)
           AND NOT EXISTS (SELECT 1 FROM series h WHERE h.hidden = 1 AND h.{bare} = {id_column})
         ORDER BY s.folder_name COLLATE NOCASE"
    );
    let mut stmt = conn.prepare(&sql)?;
    let status = |raw: Option<String>| raw.and_then(|s| ListStatus::from_column(&s));
    let rows = stmt
        .query_map(params![t.as_str()], |r| {
            let folder: String = r.get(1)?;
            let (romaji, english, native): (Option<String>, Option<String>, Option<String>) =
                (r.get(6)?, r.get(7)?, r.get(8)?);
            Ok(MoveRow {
                series: r.get::<_, i64>(0)? as u64,
                title: crate::library::titles::resolve(
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
pub fn start(core: &Core, t: Tracker, series: Vec<u64>) -> Result<u64, CoreError> {
    if series.is_empty() {
        return Err(CoreError::invalid("series", "no series picked to move"));
    }
    let connected = core
        .store
        .read(|c| accounts::load_row(c, t))?
        .is_some_and(|row| row.connected_at.is_some());
    if !connected {
        return Err(CoreError::NotConnected { tracker: t });
    }
    let owner = core
        .arc()
        .ok_or_else(|| CoreError::internal("core is shutting down"))?;
    let jobs = owner.jobs.clone();
    Ok(jobs.start(JobKind::MoveLists, move |ctx| async move {
        run(owner, ctx, t, series).await
    }))
}

/// Each picked series in turn, read afresh from the preview so a row the
/// tracker already matches, or one that left the list kept here, sends
/// nothing. A status goes first and a score second, each through the write
/// the menus use, to this tracker alone; one the tracker refuses is that
/// series' failure, never the job's.
async fn run(
    core: Arc<Core>,
    ctx: Arc<JobCtx>,
    t: Tracker,
    picked: Vec<u64>,
) -> Result<Finished, CoreError> {
    let rows: Vec<MoveRow> = core
        .store
        .write_async(move |c| preview(c, t))
        .await?
        .into_iter()
        .filter(|r| picked.contains(&r.series))
        .collect();
    let user_id = core
        .store
        .write_async(move |c| accounts::load_row(c, t))
        .await?
        .and_then(|row| row.user_id);
    let total = rows.len() as u64;
    let (mut moved, mut failed) = (0u32, 0u32);
    for (n, row) in rows.iter().enumerate() {
        ctx.checkpoint()?;
        ctx.progress(n as u64, Some(total), &row.title);
        let series = row.series;
        // A series gone since the preview is that series' failure alone.
        let Ok(target) = core
            .store
            .write_async(move |c| writes::targets(c, series))
            .await
        else {
            failed += 1;
            continue;
        };
        let Some(media_id) = (match t {
            Tracker::Anilist => target.anilist,
            Tracker::Mal => target.mal,
        }) else {
            continue;
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
            match sent(&core, &ctx, t, &target, media_id, user_id, action).await {
                true => landed = true,
                false => refused = true,
            }
        }
        if let Some(score) = row.score.filter(|s| Some(*s) != row.tracker_score)
            && !refused
        {
            let action = writes::Action::Score { score };
            match sent(&core, &ctx, t, &target, media_id, user_id, action).await {
                true => landed = true,
                false => refused = true,
            }
        }
        if landed {
            // The tracker now has the newer word, whether or not its reply
            // carried a stamp, so the preview never offers this row again.
            let now = crate::time::now_secs();
            core.store
                .write_async(move |c| {
                    c.execute(
                        "UPDATE tracker_entries SET updated_at = MAX(COALESCE(updated_at, 0), ?3)
                         WHERE tracker = ?1 AND media_id = ?2",
                        params![t.as_str(), media_id as i64, now],
                    )?;
                    Ok(())
                })
                .await?;
            moved += 1;
        } else if refused {
            failed += 1;
        }
        if let Ok(Some(card)) = crate::metadata::apply::card_for(&core, series).await {
            ctx.changed(card);
        }
    }
    let site = super::oauth::site_name(t);
    let plain = match (moved, failed) {
        (0, 0) => None,
        (m, 0) => Some(format!("Moved {} from this computer to {site}.", shows(m))),
        (m, f) => Some(format!(
            "Moved {} to {site}; {site} did not take {}.",
            shows(m),
            shows(f)
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

/// "1 show" or "3 shows".
fn shows(n: u32) -> String {
    if n == 1 {
        "1 show".to_string()
    } else {
        format!("{n} shows")
    }
}

/// One write to one tracker, true when it landed. A failure says so in the
/// activity log, the way any tracker write's failure does, with the
/// tracker's own words sanitised, and the move goes on.
async fn sent(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    t: Tracker,
    target: &writes::Targets,
    media_id: u64,
    user_id: Option<u64>,
    action: writes::Action,
) -> bool {
    match writes::attempt(core, ctx, t, target, media_id, user_id, &action).await {
        Ok(outcome) => outcome.ok,
        Err(e) => {
            let site = super::oauth::site_name(t);
            ctx.say(
                Level::Error,
                format!(
                    "{site} did not take {}: {}",
                    target.folder_name,
                    writes::sanitize_error(t, &e)
                ),
                format!("{} move of {} failed: {e}", t.as_str(), target.folder_name),
                EventBody::Notice,
            );
            false
        }
    }
}
