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
pub fn preview(conn: &Connection, t: Tracker) -> Result<Vec<MoveRow>, CoreError> {
    let id_column = match t {
        Tracker::Anilist => "s.anilist_id",
        Tracker::Mal => "s.mal_id",
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
/// have, or a score it does not have.
fn changes_something(row: &MoveRow) -> bool {
    (row.status.is_some() && row.status != row.tracker_status)
        || (row.score.is_some() && row.score != row.tracker_score)
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
        let target = core
            .store
            .write_async(move |c| writes::targets(c, series))
            .await?;
        let Some(media_id) = (match t {
            Tracker::Anilist => target.anilist,
            Tracker::Mal => target.mal,
        }) else {
            continue;
        };
        let mut ok = true;
        if let Some(status) = row.status.filter(|s| Some(*s) != row.tracker_status) {
            let action = writes::Action::Status { status };
            ok &= sent(&core, &ctx, t, &target, media_id, user_id, action).await;
        }
        if let Some(score) = row.score.filter(|s| Some(*s) != row.tracker_score)
            && ok
        {
            let action = writes::Action::Score { score };
            ok &= sent(&core, &ctx, t, &target, media_id, user_id, action).await;
        }
        if ok {
            moved += 1;
        } else {
            failed += 1;
        }
        if let Some(card) = crate::metadata::apply::card_for(&core, series).await? {
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
/// log with the tracker's own words, sanitised, and the move goes on.
async fn sent(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    t: Tracker,
    target: &writes::Targets,
    media_id: u64,
    user_id: Option<u64>,
    action: writes::Action,
) -> bool {
    match writes::attempt(core, ctx, t, target, media_id, user_id, action).await {
        Ok(outcome) => outcome.ok,
        Err(e) => {
            ctx.emit(
                Level::Debug,
                format!(
                    "{} move of {} failed: {}",
                    t.as_str(),
                    target.folder_name,
                    writes::sanitize_error(t, &e)
                ),
                EventBody::Notice,
            );
            false
        }
    }
}
