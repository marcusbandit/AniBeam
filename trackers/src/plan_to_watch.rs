//! Plan to watch (#202): a show from Airing now, or any AniList id, put on
//! Planning on every connected tracker that does not list it yet. A tracker
//! write, so it belongs to the trackers team (#854); the page it is asked
//! from stays in `airing_now`.

use std::sync::Arc;

use rusqlite::{OptionalExtension, params};

use crate::Trackers;
use crate::{accounts, cache, quiet, writes};
use anibeam_contract::*;
use anibeam_foundation::jobs::{Finished, JobCtx};
use anibeam_foundation::prefs;
use anibeam_foundation::titles::{self, fallback_title};

/// The `PlanToWatch` call: Planning on every connected tracker that does
/// not list the show yet. A show already on a list keeps its status there,
/// whatever it is; this never moves an entry. `Ok` when every connected
/// tracker lists it already. Refused when the library hides the show, the
/// rule every tracker write keeps, or when no connected tracker has an id
/// for it (a MAL-only account and a show AniList gives no MAL id), and
/// `NotConnected` when no tracker is connected at all.
pub fn plan_to_watch(trackers: &Arc<Trackers>, anilist_id: u64) -> Result<Reply, CoreError> {
    let (targets, title) = trackers.services.store.read(|c| {
        let lang = prefs::load_preferences(c)?.title_language;
        let found = c
            .query_row(
                "SELECT mal_id, title_romaji, title_english, title_native FROM season_shows
                  WHERE anilist_id = ?1
                 UNION ALL
                 SELECT mal_id, title_romaji, title_english, title_native FROM anilist_media
                  WHERE id = ?1
                 LIMIT 1",
                params![anilist_id as i64],
                |r| {
                    Ok((
                        r.get::<_, Option<i64>>(0)?.map(|id| id as u64),
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .optional()?
            .unwrap_or_default();
        let title = titles::resolve(
            lang,
            found.1.as_deref(),
            found.2.as_deref(),
            found.3.as_deref(),
            &fallback_title(anilist_id),
        );
        let ids = [
            (Tracker::Anilist, Some(anilist_id)),
            (Tracker::Mal, found.0),
        ];
        if writes::any_hidden(c, Some(anilist_id), found.0)? {
            return Err(CoreError::Refused {
                reason: Refusal::Hidden,
            });
        }
        let mut targets: Vec<(Tracker, u64, Option<u64>)> = Vec::new();
        let (mut connected, mut listed) = (false, false);
        for (t, id) in ids {
            let Some(row) = accounts::connected(c, t)? else {
                continue;
            };
            connected = true;
            let Some(id) = id else { continue };
            if cache::cached(c, t, id)?.is_some() {
                listed = true;
            } else {
                targets.push((t, id, row.user_id));
            }
        }
        if !connected {
            return Err(CoreError::NotConnected {
                tracker: Tracker::Anilist,
            });
        }
        if targets.is_empty() && !listed {
            return Err(CoreError::Refused {
                reason: Refusal::NoMatch,
            });
        }
        Ok((targets, title))
    })?;
    if targets.is_empty() {
        return Ok(Reply::Ok);
    }
    // Its place in the write line is taken now, so a status the user sets
    // after this lands after it (#1136).
    let mut turn = trackers
        .write_line
        .join_entries(targets.iter().map(|&(t, id, _)| (t, id)).collect());
    let owner = trackers.clone();
    let job = trackers
        .services
        .jobs
        .clone()
        .start(JobKind::SetStatus, move |ctx| async move {
            turn.come().await;
            plan(owner, ctx, title, targets).await
        });
    Ok(Reply::Started { job })
}

/// A status as a sentence says it.
fn status_words(status: ListStatus) -> &'static str {
    match status {
        ListStatus::Watching => "Watching",
        ListStatus::Repeating => "Rewatching",
        ListStatus::Planning => "Planning",
        ListStatus::Completed => "Completed",
        ListStatus::Paused => "On hold",
        ListStatus::Dropped => "Dropped",
    }
}

/// The SetStatus job behind `PlanToWatch`. Each tracker is read live first:
/// an entry made somewhere else since the list was read is left as it is
/// there, and the cache takes it, so the card says where the show stands.
async fn plan(
    trackers: Arc<Trackers>,
    ctx: Arc<JobCtx>,
    title: String,
    targets: Vec<(Tracker, u64, Option<u64>)>,
) -> Result<Finished, CoreError> {
    let mut planned: Vec<Tracker> = Vec::new();
    let mut found: Vec<(Tracker, ListStatus)> = Vec::new();
    let mut failure: Option<String> = None;
    for (t, media_id, user_id) in targets {
        ctx.checkpoint()?;
        let attempt = async {
            let remote = writes::Remote::connect(&trackers, t, media_id).await?;
            if let Some(live) = remote.read_live(user_id).await?
                && let Some(status) = live.status
            {
                remote
                    .record_progress(live.progress, status, live.updated_at)
                    .await?;
                return Ok::<Option<ListStatus>, CoreError>(Some(status));
            }
            let stamp = remote.send_status(ListStatus::Planning).await?;
            remote
                .record_progress(0, ListStatus::Planning, stamp)
                .await?;
            Ok(None)
        };
        match attempt.await {
            Ok(None) => planned.push(t),
            Ok(Some(status)) => found.push((t, status)),
            Err(e) => {
                failure.get_or_insert_with(|| writes::sanitize_error(t, &e));
            }
        }
    }
    let names = |ts: &[Tracker]| {
        ts.iter()
            .map(|t| crate::oauth::site_name(*t))
            .collect::<Vec<_>>()
            .join(" and ")
    };
    let labels = |ts: &[Tracker]| {
        ts.iter()
            .map(|t| t.label())
            .collect::<Vec<_>>()
            .join(" and ")
    };
    Ok(match (planned.is_empty(), found.first(), failure) {
        (false, _, _) => Finished {
            plain: Some(match planned.as_slice() {
                [one, other] => anibeam_contract::words!(
                    "core-airing-now-planned-both",
                    title = &title,
                    site = crate::oauth::site_name(*one),
                    other = crate::oauth::site_name(*other),
                ),
                ts => anibeam_contract::words!(
                    "core-airing-now-planned",
                    title = &title,
                    site = names(ts)
                ),
            }),
            level: Level::Info,
            message: format!("{title} is on Planning on {}", labels(&planned)),
            body: EventBody::Notice,
        },
        (true, Some((t, status)), _) => Finished {
            plain: Some(anibeam_contract::words!(
                "core-airing-now-already-on",
                title = &title,
                site = crate::oauth::site_name(*t),
                status = status.as_str(),
            )),
            level: Level::Info,
            message: format!(
                "{title} is already on {} as {}",
                t.label(),
                status_words(*status)
            ),
            body: EventBody::Notice,
        },
        (true, None, Some(why)) => Finished {
            plain: Some(anibeam_contract::words!(
                "core-airing-now-plan-failed",
                title = &title
            )),
            level: Level::Warn,
            message: format!("{title} could not go on Planning: {why}"),
            body: EventBody::Notice,
        },
        (true, None, None) => quiet(format!("{title} needed no change")),
    })
}
