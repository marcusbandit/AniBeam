//! The three writes a tracker takes: a mark, a corrective progress and a
//! score. Every one of them answers the shell before a request is made if
//! it can, and otherwise becomes a job that writes to every connected
//! tracker the match carries an id for.
//!
//! Three rules shape the file. A write goes to *both* trackers, because a
//! user with two accounts connected expects both lists to move, so the
//! outcome is a `TrackerOutcome` per tracker rather than one answer; a
//! provider failure is that tracker's, never the job's, so one dead
//! account cannot cost the other its write; and nothing a provider says
//! reaches the shell raw, because a GraphQL blob or an HTML error page in
//! a toast is worse than no message at all. `sanitize` below is that last
//! rule, and it is the port of Electron's `sanitizeTrackerError`.
//!
//! Carried from `src/main/handlers/trackerHandler.ts` (`markEpisode`,
//! `setEpisodeProgress`, `setScore` and the six provider halves under
//! them), the hidden guard in `src/main/ipc/tracker.ts`, and
//! `src/shared/hiddenMatch.ts`.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::metadata::apply::card_for;
use crate::metadata::fetch::message_of;
use crate::net::anilist::{MEDIA_LIST_ENTRY_QUERY, SAVE_PROGRESS_MUTATION, SAVE_SCORE_MUTATION};
use crate::net::HttpResponse;
use crate::prefs;
use crate::time;
use crate::trackers::{accounts, cache};

/// How long one provider call has before it is given up on. A page is
/// waiting on a mark, so it gives up well before the transport's own
/// thirty seconds.
const WRITE_TIMEOUT: Duration = Duration::from_secs(15);

/// The account says it is connected but there is no token behind it: a
/// keyring that lost the entry, or a MAL session whose refresh has failed
/// and already said so.
const NO_TOKEN: &str = "no access token stored, reconnect in Settings";

/// MAL's list API. AniList's address lives with its queries.
const MAL_API: &str = "https://api.myanimelist.net/v2";

// The rules ------------------------------------------------------------------

/// What a user is told when a tracker write failed. The status decides the
/// line wherever there is one, because a 429 or a 401 means something the
/// user can act on and the provider's own words rarely do; below that the
/// first thing the provider said that is short enough to read wins, and a
/// blob of GraphQL or an HTML error page falls through to the log.
///
/// `graphql_message` is AniList's `errors[0].message` and `mal_message` is
/// MAL's `message` field, both already lifted out of their bodies by the
/// time a `CoreError` reaches here.
pub fn sanitize(t: Tracker, status: Option<u32>, graphql_message: Option<&str>, mal_message: Option<&str>, raw: &str) -> String {
    let label = t.label();
    match status {
        Some(429) => return format!("{label} rate limited, try again in a minute."),
        Some(401 | 403) => return format!("{label} auth expired, reconnect in Settings."),
        Some(404) => return format!("{label} entry not found."),
        Some(s) if s >= 500 => return format!("{label} server error ({s}), try again later."),
        _ => {}
    }
    for candidate in [graphql_message, mal_message, Some(raw)] {
        if let Some(message) = candidate.filter(|m| !m.is_empty() && m.chars().count() < 200) {
            return message.to_string();
        }
    }
    format!("{label} error, see activity log.")
}

/// The same line off a `CoreError`, which is the only shape a failure
/// takes here: the provider's status and the one message it carried, which
/// is that provider's own message field whichever provider it was.
pub(crate) fn sanitize_error(t: Tracker, e: &CoreError) -> String {
    let (status, raw) = match e {
        CoreError::Provider { status, message, .. } => (*status, message.clone()),
        other => (None, other.to_string()),
    };
    let (graphql, mal) = match t {
        Tracker::Anilist => (Some(raw.as_str()), None),
        Tracker::Mal => (None, Some(raw.as_str())),
    };
    sanitize(t, status, graphql, mal, &raw)
}

/// Whether any series carrying this id is hidden. A hidden series is
/// absent from every tracker write, and the ids never cross: an AniList id
/// is only ever matched against `anilist_id`, a MAL id only against
/// `mal_id`. The port of `isSeriesHidden`.
pub fn is_hidden(conn: &Connection, t: Tracker, media_id: u64) -> Result<bool, CoreError> {
    // Nothing carries the id nought, so nothing is hidden by it.
    if media_id == 0 {
        return Ok(false);
    }
    let sql = match t {
        Tracker::Anilist => "SELECT EXISTS (SELECT 1 FROM series WHERE hidden = 1 AND anilist_id = ?1)",
        Tracker::Mal => "SELECT EXISTS (SELECT 1 FROM series WHERE hidden = 1 AND mal_id = ?1)",
    };
    Ok(conn.query_row(sql, params![media_id as i64], |r| r.get::<_, i64>(0))? == 1)
}

/// Where one series' writes go: the ids the match carries, the published
/// total that decides whether a write completes the list, and the name the
/// activity log calls the series by.
#[derive(Clone, Debug, PartialEq)]
pub struct Targets {
    pub series: u64,
    pub anilist: Option<u64>,
    pub mal: Option<u64>,
    pub total: Option<u32>,
    pub folder_name: String,
}

impl Targets {
    fn id_for(&self, t: Tracker) -> Option<u64> {
        match t {
            Tracker::Anilist => self.anilist,
            Tracker::Mal => self.mal,
        }
    }

    fn matched(&self) -> bool {
        self.anilist.is_some() || self.mal.is_some()
    }

    /// Whether this number reaches the published total, which is what
    /// turns a write into a completion.
    fn completes(&self, progress: u32) -> bool {
        self.total.is_some_and(|total| progress >= total)
    }
}

/// One series' write targets, or `NotFound`. A series whose path is gone
/// answers the same way a series that never existed does: a missing series
/// is absent from every tracker write, and no refusal names it.
pub fn targets(conn: &Connection, series: u64) -> Result<Targets, CoreError> {
    let row = conn
        .query_row(
            "SELECT anilist_id, mal_id, folder_name, missing_since FROM series WHERE id = ?1",
            params![series as i64],
            |r| {
                Ok((
                    r.get::<_, Option<i64>>(0)?,
                    r.get::<_, Option<i64>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                ))
            },
        )
        .optional()?;
    let Some((anilist, mal, folder_name, missing_since)) = row else {
        return Err(CoreError::NotFound { what: Entity::Series, id: series });
    };
    if missing_since.is_some() {
        return Err(CoreError::NotFound { what: Entity::Series, id: series });
    }
    let anilist = anilist.map(|id| id as u64);
    // A total of nought is AniList saying it does not know yet, the same as
    // a NULL, so it must never complete a list on the first episode.
    let total = match anilist {
        Some(id) => conn
            .query_row("SELECT episodes FROM anilist_media WHERE id = ?1", params![id as i64], |r| r.get::<_, Option<i64>>(0))
            .optional()?
            .flatten()
            .and_then(|episodes| u32::try_from(episodes).ok())
            .filter(|total| *total > 0),
        None => None,
    };
    Ok(Targets { series, anilist, mal: mal.map(|id| id as u64), total, folder_name })
}

// The calls ------------------------------------------------------------------

/// Marks an episode watched on every connected tracker. The number is
/// floored, since a tracker counts whole episodes and 12.5 is still the
/// twelfth.
pub fn mark(core: &Core, series: u64, episode: f64) -> Result<u64, CoreError> {
    let target = core.store.read(|c| targets(c, series))?;
    if !episode.is_finite() || episode.floor() < 1.0 {
        return Err(CoreError::invalid("episode", format!("{episode} is not a whole episode above nought")));
    }
    let episode = episode.floor() as u32;
    let main = guard(core, &target)?;
    // The cheap half of the monotonic rule: the cache already knows how
    // far the main tracker says the user has got, so a mark it covers is
    // refused without a request. The job checks again against the tracker
    // itself, which is what actually decides.
    if core.store.read(|c| cached_progress(c, &target, main))?.is_some_and(|current| current >= episode) {
        return Err(CoreError::Refused { reason: Refusal::NotNewer });
    }
    start(core, JobKind::Mark, target, Action::Mark { episode })
}

/// Sets the watched count to an exact value, including a lower one. The
/// corrective path: `mark` only ever moves progress up, and this is how an
/// over-counted episode is undone.
pub fn set_progress(core: &Core, series: u64, progress: u32) -> Result<u64, CoreError> {
    let target = core.store.read(|c| targets(c, series))?;
    guard(core, &target)?;
    start(core, JobKind::SetProgress, target, Action::Progress { progress })
}

/// Rates a series out of ten on every connected tracker. `None` is the
/// Clear button, which is a score of nought on both rather than a call of
/// its own.
pub fn set_score(core: &Core, series: u64, score: Option<f64>) -> Result<u64, CoreError> {
    let target = core.store.read(|c| targets(c, series))?;
    let score = score.unwrap_or(0.0);
    if !(0.0..=10.0).contains(&score) {
        return Err(CoreError::invalid("score", format!("{score} is outside 0 to 10")));
    }
    guard(core, &target)?;
    start(core, JobKind::Score, target, Action::Score { score })
}

/// The guards every write shares, in the order the shell expects them:
/// nothing to write to, a series the user hid, and no account behind
/// either id. Answers with the main tracker, which is the one the
/// `NotConnected` refusal names and the one the monotonic guard reads.
fn guard(core: &Core, target: &Targets) -> Result<Tracker, CoreError> {
    core.store.read(|c| {
        if !target.matched() {
            return Err(CoreError::Refused { reason: Refusal::NoMatch });
        }
        for t in [Tracker::Anilist, Tracker::Mal] {
            if let Some(id) = target.id_for(t)
                && is_hidden(c, t, id)?
            {
                return Err(CoreError::Refused { reason: Refusal::Hidden });
            }
        }
        let main = prefs::load_main_tracker(c)?;
        let mut connected = false;
        for t in [Tracker::Anilist, Tracker::Mal] {
            if target.id_for(t).is_some() && accounts::load_row(c, t)?.is_some_and(|row| row.connected_at.is_some()) {
                connected = true;
            }
        }
        if !connected {
            return Err(CoreError::NotConnected { tracker: main });
        }
        Ok(main)
    })
}

/// What the cache says the main tracker's progress is, by the id the match
/// carries for it. A match with no id for the main tracker is read off the
/// other one instead, which is the rule a card's own number follows.
fn cached_progress(conn: &Connection, target: &Targets, main: Tracker) -> Result<Option<u32>, CoreError> {
    let t = match target.id_for(main) {
        Some(_) => main,
        None => other(main),
    };
    let Some(id) = target.id_for(t) else { return Ok(None) };
    let progress = conn
        .query_row(
            "SELECT progress FROM tracker_entries WHERE tracker = ?1 AND media_id = ?2",
            params![t.as_str(), id as i64],
            |r| r.get::<_, i64>(0),
        )
        .optional()?;
    Ok(progress.map(|p| u32::try_from(p).unwrap_or(0)))
}

fn other(t: Tracker) -> Tracker {
    match t {
        Tracker::Anilist => Tracker::Mal,
        Tracker::Mal => Tracker::Anilist,
    }
}

// The job --------------------------------------------------------------------

/// What one write does to an entry. The three actions share a job, a read
/// and a cache patch, so they share a type rather than three near-copies.
#[derive(Clone, Copy, Debug)]
enum Action {
    Mark { episode: u32 },
    Progress { progress: u32 },
    Score { score: f64 },
}

impl Action {
    /// What the activity log calls this write when it fails.
    fn verb(self) -> &'static str {
        match self {
            Action::Mark { .. } => "mark",
            Action::Progress { .. } => "set progress",
            Action::Score { .. } => "score",
        }
    }
}

fn start(core: &Core, kind: JobKind, target: Targets, action: Action) -> Result<u64, CoreError> {
    let owner = core.arc().ok_or_else(|| CoreError::internal("core is shutting down"))?;
    let jobs = owner.jobs.clone();
    Ok(jobs.start(kind, move |ctx| async move { run(owner, ctx, target, action).await }))
}

async fn run(core: Arc<Core>, ctx: Arc<JobCtx>, target: Targets, action: Action) -> Result<Finished, CoreError> {
    let mut outcomes: Vec<TrackerOutcome> = Vec::new();
    for t in [Tracker::Anilist, Tracker::Mal] {
        ctx.checkpoint()?;
        // A tracker the match carries no id for has nothing to write to,
        // and one with no account behind it is not this write's business.
        let Some(media_id) = target.id_for(t) else { continue };
        let row = core.store.write_async(move |c| accounts::load_row(c, t)).await?.unwrap_or_default();
        if row.connected_at.is_none() {
            continue;
        }
        outcomes.push(write_one(&core, &ctx, t, &target, media_id, row.user_id, action).await);
    }
    // The cache was patched by each write above, so the card built here
    // already carries the new number. The runner flushes this before the
    // terminal event.
    if let Some(card) = card_for(&core, target.series).await? {
        ctx.changed(card);
    }
    Ok(finished(&target, action, outcomes))
}

fn finished(target: &Targets, action: Action, outcomes: Vec<TrackerOutcome>) -> Finished {
    let series = target.series;
    let folder = &target.folder_name;
    match action {
        Action::Mark { episode } => Finished {
            level: Level::Info,
            message: format!("marked episode {episode} of {folder}"),
            body: EventBody::Marked { series, episode, outcomes },
        },
        Action::Progress { progress } => Finished {
            level: Level::Info,
            message: format!("progress of {folder} set to {progress}"),
            body: EventBody::ProgressSet { series, progress, outcomes },
        },
        Action::Score { score } => {
            // A score of nought is unrated on both trackers, so the event
            // carries no score rather than a rating of nothing.
            let rated = (score > 0.0).then_some(score);
            Finished {
                level: Level::Info,
                message: match rated {
                    Some(score) => format!("score of {folder} set to {score}"),
                    None => format!("score of {folder} cleared"),
                },
                body: EventBody::Scored { series, score: rated, outcomes },
            }
        }
    }
}

/// One tracker's outcome, whatever happened. A provider failure is this
/// tracker's alone: it becomes a `ok: false` outcome with a line the user
/// can act on and an Error line carrying the whole message, and the job
/// carries on to the other tracker.
async fn write_one(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    t: Tracker,
    target: &Targets,
    media_id: u64,
    user_id: Option<u64>,
    action: Action,
) -> TrackerOutcome {
    match attempt(core, ctx, t, target, media_id, user_id, action).await {
        Ok(outcome) => outcome,
        Err(e) => {
            ctx.emit(Level::Error, format!("{} {} failed: {}", t.as_str(), action.verb(), message_of(&e)), EventBody::Notice);
            TrackerOutcome { tracker: t, ok: false, progress: None, reason: None, message: Some(sanitize_error(t, &e)) }
        }
    }
}

/// The write itself: the tracker's current entry, the action's own guard,
/// the mutation, the account's `synced_at`, the cache patch and the line
/// the activity log gets.
async fn attempt(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    t: Tracker,
    target: &Targets,
    media_id: u64,
    user_id: Option<u64>,
    action: Action,
) -> Result<TrackerOutcome, CoreError> {
    let token = accounts::access_token(core, t).await?.ok_or_else(|| accounts::tracker_error(t, NO_TOKEN))?;
    let current = read_entry(core, t, media_id, user_id, &token).await?;
    match action {
        Action::Mark { episode } => {
            // The monotonic rule, decided against the tracker itself: a
            // mark only ever moves progress up, and a list already past
            // this episode is not wrong, it is ahead.
            if current >= episode {
                return Ok(TrackerOutcome { tracker: t, ok: false, progress: Some(current), reason: Some(Refusal::NotNewer), message: None });
            }
            let status = if target.completes(episode) { ListStatus::Completed } else { ListStatus::Watching };
            send_progress(core, t, media_id, &token, episode, status).await?;
            record_progress(core, t, media_id, episode, status).await?;
            ctx.emit(Level::Info, format!("{} {current} -> {episode} (mediaId {media_id})", t.as_str()), EventBody::Notice);
            Ok(TrackerOutcome { tracker: t, ok: true, progress: Some(episode), reason: None, message: None })
        }
        Action::Progress { progress } => {
            let status = if progress == 0 {
                ListStatus::Planning
            } else if target.completes(progress) {
                ListStatus::Completed
            } else {
                ListStatus::Watching
            };
            send_progress(core, t, media_id, &token, progress, status).await?;
            record_progress(core, t, media_id, progress, status).await?;
            ctx.emit(Level::Info, format!("{} set {current} -> {progress} (mediaId {media_id})", t.as_str()), EventBody::Notice);
            Ok(TrackerOutcome { tracker: t, ok: true, progress: Some(progress), reason: None, message: None })
        }
        Action::Score { score } => {
            // Rating a series whose every episode is already watched is
            // also saying it is finished, which is what the list should
            // have said all along.
            let completed = target.total.is_some_and(|total| current >= total);
            let sent = send_score(core, t, media_id, &token, score, completed).await?;
            record_score(core, t, media_id, sent, completed).await?;
            let tail = if completed { " + completed" } else { "" };
            ctx.emit(Level::Info, format!("{} score -> {sent} (mediaId {media_id}){tail}", t.as_str()), EventBody::Notice);
            Ok(TrackerOutcome { tracker: t, ok: true, progress: None, reason: None, message: None })
        }
    }
}

// The provider calls ---------------------------------------------------------

/// How far the tracker itself says the user has got. An anime the user has
/// never added is nought rather than a failure: that is exactly what a
/// first mark is for, on either tracker.
async fn read_entry(core: &Arc<Core>, t: Tracker, media_id: u64, user_id: Option<u64>, token: &str) -> Result<u32, CoreError> {
    match t {
        Tracker::Anilist => {
            // `MediaList(mediaId)` without the user id ignores the bearer
            // token and answers with some other user's entry, so a missing
            // user id is a broken account rather than a reason to guess.
            let user_id = user_id.ok_or_else(|| accounts::tracker_error(t, "the account carries no user id"))?;
            let variables = serde_json::json!({ "userId": user_id, "mediaId": media_id });
            match timed(t, "read", core.anilist.graphql(MEDIA_LIST_ENTRY_QUERY, variables, Some(token))).await {
                Ok(data) => Ok(as_count(data["MediaList"]["progress"].as_u64())),
                Err(e) if is_not_found(&e) => Ok(0),
                Err(e) => Err(e),
            }
        }
        Tracker::Mal => {
            let url = format!("{MAL_API}/anime/{media_id}?fields=my_list_status");
            let response = timed(t, "read", core.mal.get(&url, token)).await?;
            if response.status == 404 {
                return Ok(0);
            }
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
            let body: serde_json::Value = response.json()?;
            Ok(as_count(body["my_list_status"]["num_episodes_watched"].as_u64()))
        }
    }
}

async fn send_progress(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    token: &str,
    progress: u32,
    status: ListStatus,
) -> Result<(), CoreError> {
    match t {
        Tracker::Anilist => {
            let variables = serde_json::json!({ "mediaId": media_id, "progress": progress, "status": anilist_status(status) });
            timed(t, "write", core.anilist.graphql(SAVE_PROGRESS_MUTATION, variables, Some(token))).await?;
        }
        Tracker::Mal => {
            let form = vec![
                ("num_watched_episodes".to_string(), progress.to_string()),
                ("status".to_string(), mal_status(status).to_string()),
            ];
            let response = timed(t, "write", core.mal.patch_form(&list_status_url(media_id), token, form)).await?;
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
        }
    }
    Ok(())
}

/// The score each tracker actually receives, which is not the same number:
/// AniList takes `scoreRaw` out of a hundred, so one write works whatever
/// display format the user picked and a decimal survives it; MAL takes a
/// whole number and rounds the way its own UI rounds a typed decimal. What
/// was sent comes back, so the cache and the log line agree with the list.
async fn send_score(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    token: &str,
    score: f64,
    completed: bool,
) -> Result<f64, CoreError> {
    match t {
        Tracker::Anilist => {
            let mut variables = serde_json::json!({ "mediaId": media_id, "scoreRaw": (score * 10.0).round() as i64 });
            // A mutation sent without the status variable leaves the
            // entry's own status alone, which is what a rating that
            // completes nothing should do.
            if completed {
                variables["status"] = serde_json::Value::String(anilist_status(ListStatus::Completed).to_string());
            }
            timed(t, "score", core.anilist.graphql(SAVE_SCORE_MUTATION, variables, Some(token))).await?;
            Ok(score)
        }
        Tracker::Mal => {
            let rounded = score.round();
            let mut form = vec![("score".to_string(), (rounded as i64).to_string())];
            if completed {
                form.push(("status".to_string(), mal_status(ListStatus::Completed).to_string()));
            }
            let response = timed(t, "score", core.mal.patch_form(&list_status_url(media_id), token, form)).await?;
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
            Ok(rounded)
        }
    }
}

fn list_status_url(media_id: u64) -> String {
    format!("{MAL_API}/anime/{media_id}/my_list_status")
}

/// Every provider call one write makes, with the tracker timeout on it. A
/// call that never answered has no status to report, so the failure is the
/// provider's with the tracker and the action named in the message.
async fn timed<T>(t: Tracker, what: &str, call: impl Future<Output = Result<T, CoreError>>) -> Result<T, CoreError> {
    match tokio::time::timeout(WRITE_TIMEOUT, call).await {
        Ok(result) => result,
        Err(_) => Err(accounts::tracker_error(t, format!("{} {what} timed out after {}ms", t.label(), WRITE_TIMEOUT.as_millis()))),
    }
}

/// MAL's failure with the `message` field it usually carries lifted out of
/// the body: the body is JSON, and a truncated blob of it reads as nothing
/// at all in a toast.
fn mal_failure(response: &HttpResponse) -> CoreError {
    let text = response.text();
    let message = response
        .json::<serde_json::Value>()
        .ok()
        .and_then(|body| body["message"].as_str().map(str::to_string))
        .filter(|message| !message.is_empty())
        .unwrap_or(text);
    CoreError::Provider { provider: Provider::Mal, status: Some(u32::from(response.status)), message, retry_after: None }
}

// What is left behind --------------------------------------------------------

/// The account's `synced_at` and the entry's progress, in one transaction:
/// two tables move together or neither does.
async fn record_progress(core: &Arc<Core>, t: Tracker, media_id: u64, progress: u32, status: ListStatus) -> Result<(), CoreError> {
    let now = time::now_secs();
    core.store
        .tx_async(move |tx| {
            mark_synced(tx, t, now)?;
            cache::patch_progress(tx, t, media_id, progress, Some(status), now)
        })
        .await
}

/// The same for a rating. A score of nought is unrated rather than a
/// rating of nothing, and a status is only written when the rating
/// completed the entry.
async fn record_score(core: &Arc<Core>, t: Tracker, media_id: u64, score: f64, completed: bool) -> Result<(), CoreError> {
    let now = time::now_secs();
    let rated = (score > 0.0).then_some(score);
    let status = completed.then_some(ListStatus::Completed);
    core.store
        .tx_async(move |tx| {
            mark_synced(tx, t, now)?;
            cache::patch_score(tx, t, media_id, rated, status, now)
        })
        .await
}

fn mark_synced(conn: &Connection, t: Tracker, now: i64) -> Result<(), CoreError> {
    conn.execute("UPDATE tracker_accounts SET synced_at = ?2 WHERE tracker = ?1", params![t.as_str(), now])?;
    Ok(())
}

// Vocabulary -----------------------------------------------------------------

/// The core's status in AniList's words. The inverse of
/// `cache::normalize_status`, for the statuses a write ever sends.
fn anilist_status(status: ListStatus) -> &'static str {
    match status {
        ListStatus::Watching => "CURRENT",
        ListStatus::Planning => "PLANNING",
        ListStatus::Completed => "COMPLETED",
        ListStatus::Paused => "PAUSED",
        ListStatus::Dropped => "DROPPED",
        ListStatus::Repeating => "REPEATING",
    }
}

/// The same in MAL's. MAL has no rewatching status of its own, so a
/// repeat is watching there, which is how it came in.
fn mal_status(status: ListStatus) -> &'static str {
    match status {
        ListStatus::Watching | ListStatus::Repeating => "watching",
        ListStatus::Planning => "plan_to_watch",
        ListStatus::Completed => "completed",
        ListStatus::Paused => "on_hold",
        ListStatus::Dropped => "dropped",
    }
}

fn is_not_found(e: &CoreError) -> bool {
    matches!(e, CoreError::Provider { status: Some(404), .. })
}

/// A count off a provider's JSON, which is unsigned and small: anything
/// missing or absurd is nought rather than a wrap-around.
fn as_count(value: Option<u64>) -> u32 {
    value.and_then(|v| u32::try_from(v).ok()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The status is read first, whatever the provider said, because a 429
    /// or a 401 is something the user can act on and "Too Many Requests."
    /// is not.
    #[test]
    fn a_status_decides_the_line_before_anything_the_provider_said() {
        let said = |status| sanitize(Tracker::Anilist, Some(status), Some("Too Many Requests."), None, "raw");
        assert_eq!(said(429), "AniList rate limited, try again in a minute.");
        assert_eq!(said(401), "AniList auth expired, reconnect in Settings.");
        assert_eq!(said(403), "AniList auth expired, reconnect in Settings.");
        assert_eq!(said(404), "AniList entry not found.");
        assert_eq!(said(500), "AniList server error (500), try again later.");
        assert_eq!(said(503), "AniList server error (503), try again later.");
        // The label follows the tracker, so a MAL failure never says
        // AniList at the user.
        assert_eq!(sanitize(Tracker::Mal, Some(429), None, None, "raw"), "MAL rate limited, try again in a minute.");
    }

    /// Below the statuses that mean something, the provider's own words are
    /// the best line there is: GraphQL's first, then MAL's, then whatever
    /// the error itself carried.
    #[test]
    fn the_first_short_message_wins_in_order() {
        assert_eq!(sanitize(Tracker::Anilist, None, Some("gql"), Some("mal"), "raw"), "gql");
        assert_eq!(sanitize(Tracker::Anilist, None, None, Some("mal"), "raw"), "mal");
        assert_eq!(sanitize(Tracker::Anilist, None, None, None, "raw"), "raw");
        // A status with no rule of its own falls through to the messages.
        assert_eq!(sanitize(Tracker::Mal, Some(400), None, Some("Bad Request"), "raw"), "Bad Request");
        // Nothing said is not a message.
        assert_eq!(sanitize(Tracker::Mal, None, Some(""), Some(""), "raw"), "raw");
    }

    /// A blob of GraphQL, an HTML error page or a stack trace is worse in
    /// a toast than no message at all, so anything that long falls through
    /// to the log.
    #[test]
    fn a_long_message_falls_through_to_the_log() {
        let long = "x".repeat(600);
        assert_eq!(sanitize(Tracker::Anilist, None, None, None, &long), "AniList error, see activity log.");
        // The long one is skipped and the next short one still answers.
        assert_eq!(sanitize(Tracker::Mal, None, Some(&long), Some("short"), "raw"), "short");
        assert_eq!(sanitize(Tracker::Mal, Some(302), Some(&long), Some(&long), &long), "MAL error, see activity log.");
        // Two hundred is the line: one under it reads, one on it does not.
        let edge = "y".repeat(199);
        assert_eq!(sanitize(Tracker::Anilist, None, None, None, &edge), edge);
        let over = "y".repeat(200);
        assert_eq!(sanitize(Tracker::Anilist, None, None, None, &over), "AniList error, see activity log.");
        assert_eq!(sanitize(Tracker::Anilist, None, None, None, ""), "AniList error, see activity log.");
    }

    /// A `CoreError` carries one message whichever provider it came from,
    /// and the status still decides the line where there is one.
    #[test]
    fn an_error_is_sanitised_by_its_status_then_its_message() {
        let limited = CoreError::Provider { provider: Provider::Anilist, status: Some(429), message: "AniList rate limited".into(), retry_after: None };
        assert_eq!(sanitize_error(Tracker::Anilist, &limited), "AniList rate limited, try again in a minute.");
        let refused = CoreError::Provider { provider: Provider::Mal, status: None, message: "connection refused".into(), retry_after: None };
        assert_eq!(sanitize_error(Tracker::Mal, &refused), "connection refused");
        // Anything that is not a provider failure still reads as one line.
        assert_eq!(sanitize_error(Tracker::Mal, &CoreError::Refused { reason: Refusal::Hidden }), "refused: Hidden");
    }
}
