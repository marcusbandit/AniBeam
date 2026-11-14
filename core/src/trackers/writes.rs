//! The five writes a shell asks a tracker for: a mark, a corrective
//! progress, a score, a list status and the entry's notes. Every one of them answers the shell
//! before a request is made if it can, and otherwise becomes a job that
//! writes to every connected tracker the match carries an id for. The On
//! hold sweep (`on_hold`) also sends a status alone, through `send_status`.
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

use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::*;
use crate::core::Core;
use crate::i18n::{Arg, tr};
use crate::jobs::{Finished, JobCtx};
use crate::library::history::How;
use crate::metadata::apply::card_for;
use crate::metadata::fetch::message_of;
use crate::net::HttpResponse;
use crate::net::anilist::{
    MEDIA_LIST_ENTRY_QUERY, ON_HOLD_ENTRY_QUERY, SAVE_LIST_STATUS_MUTATION, SAVE_NOTES_MUTATION,
    SAVE_PROGRESS_MUTATION, SAVE_SCORE_MUTATION, SAVE_STATUS_MUTATION,
};
use crate::prefs;
use crate::time;
use crate::trackers::{
    TRACKER_TIMEOUT, accounts, as_count, as_u32, cache, local, no_token, on_hold,
};

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
pub fn sanitize(
    t: Tracker,
    status: Option<u32>,
    graphql_message: Option<&str>,
    mal_message: Option<&str>,
    raw: &str,
) -> String {
    let label = || ("tracker", Arg::from(t.label()));
    match status {
        Some(429) => return tr("core-trackers-rate-limited", &[label()]),
        Some(401 | 403) => return tr("core-trackers-auth-expired", &[label()]),
        Some(404) => return tr("core-trackers-entry-not-found", &[label()]),
        Some(s) if s >= 500 => {
            return tr(
                "core-trackers-server-error",
                &[label(), ("status", s.to_string().into())],
            );
        }
        _ => {}
    }
    for candidate in [graphql_message, mal_message, Some(raw)] {
        if let Some(message) = candidate.filter(|m| !m.is_empty() && m.chars().count() < 200) {
            return message.to_string();
        }
    }
    tr("core-trackers-error-see-log", &[label()])
}

/// The same line off a `CoreError`, which is the only shape a failure
/// takes here: the provider's status and the one message it carried, which
/// is that provider's own message field whichever provider it was.
pub(crate) fn sanitize_error(t: Tracker, e: &CoreError) -> String {
    let (status, raw) = match e {
        CoreError::Provider {
            status, message, ..
        } => (*status, message.clone()),
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
        Tracker::Anilist => {
            "SELECT EXISTS (SELECT 1 FROM series WHERE hidden = 1 AND anilist_id = ?1)"
        }
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
        return Err(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        });
    };
    if missing_since.is_some() {
        return Err(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        });
    }
    let anilist = anilist.map(|id| id as u64);
    // A total of nought is AniList saying it does not know yet, the same as
    // a NULL, so it must never complete a list on the first episode.
    let total = match anilist {
        Some(id) => conn
            .query_row(
                "SELECT episodes FROM anilist_media WHERE id = ?1",
                params![id as i64],
                |r| r.get::<_, Option<i64>>(0),
            )
            .optional()?
            .flatten()
            .and_then(|episodes| u32::try_from(episodes).ok())
            .filter(|total| *total > 0),
        None => None,
    };
    Ok(Targets {
        series,
        anilist,
        mal: mal.map(|id| id as u64),
        total,
        folder_name,
    })
}

// The calls ------------------------------------------------------------------

/// Marks an episode watched on every connected tracker. The number is
/// floored, since a tracker counts whole episodes and 12.5 is still the
/// twelfth.
pub fn mark(core: &Core, series: u64, episode: f64) -> Result<u64, CoreError> {
    let target = core.store.read(|c| targets(c, series))?;
    if !episode.is_finite() || episode.floor() < 1.0 {
        return Err(CoreError::invalid(
            "episode",
            tr(
                "core-trackers-episode-not-whole",
                &[("episode", episode.to_string().into())],
            ),
        ));
    }
    let episode = episode.floor() as u32;
    let main = guard(core, &target)?;
    // A completed show finished again is a rewatch, which is the one mark
    // the list may already cover.
    let again = core
        .store
        .read(|c| finished_again(c, &target, main, episode))?
        && rewatching().insert(series);
    let slot = again.then_some(RewatchSlot(series));
    // The cheap half of the monotonic rule: the cache already knows how
    // far the main tracker says the user has got, so a mark it covers is
    // refused without a request. The job checks again against the tracker
    // itself, which is what actually decides.
    if !again
        && core
            .store
            .read(|c| cached_progress(c, &target, main))?
            .is_some_and(|current| current >= episode)
    {
        return Err(CoreError::Refused {
            reason: Refusal::NotNewer,
        });
    }
    start_holding(
        core,
        JobKind::Mark,
        target,
        Action::Mark { episode, again },
        slot,
    )
}

/// How long a film's list must have stood still before watching it again
/// counts as a rewatch: a second mark the same evening is the same viewing.
const FILM_REWATCH_GAP: i64 = 12 * 3600;

/// Whether this mark finishes a completed show a second time: the list
/// says Completed, the episode is the last one, and the first episode was
/// watched to the end here since the list last changed, so the show was
/// watched again from its start. A film has no first episode to go by, so
/// it counts when the list has not changed in twelve hours. Playing only the
/// last episode again is not a rewatch, a list with no stamp gives no
/// evidence, and a show with no published total is never finished at all.
fn finished_again(
    conn: &Connection,
    target: &Targets,
    main: Tracker,
    episode: u32,
) -> Result<bool, CoreError> {
    if !target.completes(episode) {
        return Ok(false);
    }
    let Some((t, id)) = cached_id(target, main) else {
        return Ok(false);
    };
    let entry = conn
        .query_row(
            "SELECT status, updated_at FROM tracker_entries WHERE tracker = ?1 AND media_id = ?2",
            params![t.as_str(), id as i64],
            |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<i64>>(1)?)),
        )
        .optional()?;
    let Some((status, Some(updated_at))) = entry else {
        return Ok(false);
    };
    if status.as_deref().and_then(ListStatus::from_column) != Some(ListStatus::Completed) {
        return Ok(false);
    }
    if target.total == Some(1) {
        return Ok(updated_at < time::now_secs() - FILM_REWATCH_GAP);
    }
    let watched: i64 = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM completed c
             JOIN files f ON f.series_id = c.series_id AND f.episode_key = c.episode_key
             WHERE c.profile_id = 1 AND c.series_id = ?1 AND f.kind = 'episode'
               AND f.number >= 1 AND f.number < 2 AND c.at > ?2)",
        params![target.series as i64, updated_at],
        |r| r.get(0),
    )?;
    Ok(watched == 1)
}

/// The series whose rewatch is being written right now. Two marks of one
/// finish (the rule at the end and the player's button, say) both read the
/// list before either has written, so the second waits its turn out here
/// rather than counting the same rewatch twice.
static REWATCHING: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<u64>>> =
    std::sync::LazyLock::new(Default::default);

fn rewatching() -> std::sync::MutexGuard<'static, std::collections::HashSet<u64>> {
    REWATCHING.lock().unwrap_or_else(|e| e.into_inner())
}

/// A series' place in `REWATCHING`, given up when the job that holds it ends
/// however it ends.
struct RewatchSlot(u64);

impl Drop for RewatchSlot {
    fn drop(&mut self) {
        rewatching().remove(&self.0);
    }
}

/// The tracker and id a cached read goes by: the main tracker's when the
/// match carries one, otherwise the other's, the rule a card's own number
/// follows.
fn cached_id(target: &Targets, main: Tracker) -> Option<(Tracker, u64)> {
    [main, other(main)]
        .into_iter()
        .find_map(|t| target.id_for(t).map(|id| (t, id)))
}

/// Sets the watched count to an exact value, including a lower one. The
/// corrective path: `mark` only ever moves progress up, and this is how an
/// over-counted episode is undone.
///
/// A series no tracker knows (a TVmaze show, an unmatched folder) keeps its
/// watched episodes here instead: the episodes of `season` up to `progress`
/// are marked completed on this device and the ones after it are not, at
/// once, with no job. None then, where a tracker write answers with its job.
pub fn set_progress(
    core: &Core,
    series: u64,
    progress: u32,
    season: Option<u32>,
) -> Result<Option<u64>, CoreError> {
    let target = core.store.read(|c| targets(c, series))?;
    if !target.matched() {
        mark_here(core, series, progress, season.unwrap_or(1))?;
        return Ok(None);
    }
    guard(core, &target)?;
    start(
        core,
        JobKind::SetProgress,
        target,
        Action::Progress { progress },
    )
    .map(Some)
}

/// The local half of `set_progress`, within one season (a file naming none
/// is season 1's): every episode file from 1 up to `progress` is completed,
/// every one past it is not, and the card is pushed so the page redraws. A
/// half episode goes with the whole one before it, so 12.5 is watched with
/// 12, the way a tracker's count reads it. An episode 0 is never touched.
fn mark_here(core: &Core, series: u64, progress: u32, season: u32) -> Result<(), CoreError> {
    let now = time::now_secs();
    let past = f64::from(progress) + 1.0;
    core.store.tx(move |tx| {
        // Each episode this marks for the first time is a hand-set finish in
        // the watch log, which Stats leaves out (#593). One already watched
        // keeps its own finishes and gains none.
        tx.execute(
            "INSERT INTO watch_log (series_id, episode_key, at, how)
             SELECT DISTINCT f.series_id, f.episode_key, ?3, ?5 FROM files f
             WHERE f.series_id = ?1 AND f.kind = 'episode' AND COALESCE(f.season, 1) = ?4
               AND f.number >= 1 AND f.number < ?2
               AND NOT EXISTS (SELECT 1 FROM completed c
                               WHERE c.profile_id = 1 AND c.series_id = f.series_id
                                 AND c.episode_key = f.episode_key)",
            params![series as i64, past, now, season, How::Marked.as_str()],
        )?;
        tx.execute(
            "INSERT INTO completed (series_id, episode_key, at)
             SELECT DISTINCT series_id, episode_key, ?3 FROM files
             WHERE series_id = ?1 AND kind = 'episode' AND COALESCE(season, 1) = ?4
               AND number >= 1 AND number < ?2
             ON CONFLICT(profile_id, series_id, episode_key) DO NOTHING",
            params![series as i64, past, now, season],
        )?;
        tx.execute(
            "DELETE FROM completed WHERE profile_id = 1 AND series_id = ?1 AND episode_key IN (
                 SELECT episode_key FROM files
                 WHERE series_id = ?1 AND kind = 'episode' AND COALESCE(season, 1) = ?3
                   AND number >= ?2)",
            params![series as i64, past, season],
        )?;
        // Taking a count back takes those episodes' finishes with it, all but
        // the ones the player wrote: those episodes were watched here.
        tx.execute(
            "DELETE FROM watch_log WHERE profile_id = 1 AND series_id = ?1 AND how <> ?4 AND episode_key IN (
                 SELECT episode_key FROM files
                 WHERE series_id = ?1 AND kind = 'episode' AND COALESCE(season, 1) = ?3
                   AND number >= ?2)",
            params![series as i64, past, season, How::Played.as_str()],
        )?;
        Ok(())
    })?;
    let images_dir = core.paths.images_dir();
    let cards = core
        .store
        .read(|c| crate::library::cards::cards_for(c, &images_dir, &[series]))?;
    core.bus.debug(
        Stage::Library,
        "watched episodes set here",
        EventBody::SeriesChanged { series: cards },
    );
    Ok(())
}

/// Rates a series out of ten on every connected tracker. `None` is the
/// Clear button, which is a score of nought on both rather than a call of
/// its own.
///
/// A series no connected tracker carries keeps its score on this device
/// instead (#213), at once and with no job: None then, where a tracker
/// write answers with its job.
pub fn set_score(core: &Core, series: u64, score: Option<f64>) -> Result<Option<u64>, CoreError> {
    let target = core.store.read(|c| targets(c, series))?;
    let score = score.unwrap_or(0.0);
    if !(0.0..=10.0).contains(&score) {
        return Err(CoreError::invalid(
            "score",
            tr(
                "core-trackers-score-out-of-range",
                &[("score", score.to_string().into())],
            ),
        ));
    }
    if kept_here(core, &target)? {
        let now = time::now_secs();
        // Nought is unrated here as it is on a tracker.
        let rated = (score > 0.0).then_some(score);
        core.store
            .write(move |c| local::set_score(c, series, rated, now))?;
        push_card(core, series, "score kept here");
        return Ok(None);
    }
    start(core, JobKind::Score, target, Action::Score { score }).map(Some)
}

/// Puts a series on one list on every connected tracker: the status pill's
/// menu. Repeating is a rewatch the trackers keep their own count for, so
/// it is not a status this menu sets.
///
/// A series no connected tracker carries keeps its status on this device
/// instead (#213), the way `set_score` keeps a score.
pub fn set_list_status(
    core: &Core,
    series: u64,
    status: ListStatus,
) -> Result<Option<u64>, CoreError> {
    if status == ListStatus::Repeating {
        return Err(CoreError::invalid(
            "status",
            tr("core-trackers-rewatch-not-settable", &[]),
        ));
    }
    let target = core.store.read(|c| targets(c, series))?;
    if kept_here(core, &target)? {
        let now = time::now_secs();
        core.store
            .write(move |c| local::set_status(c, series, status, now))?;
        push_card(core, series, "list status kept here");
        return Ok(None);
    }
    start(core, JobKind::SetStatus, target, Action::Status { status }).map(Some)
}

/// The library's metadata for a show: its MAL id, episode count and three titles.
type LibraryMedia = (
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<String>,
);

/// The write targets for a show the library does not have (#779): the
/// AniList id, the MAL id and episode count the core holds for it, and its
/// title for the log. `series` is nought, which no series carries, so no
/// card follows the write. A show the library has goes through its series.
fn show_targets(conn: &Connection, anilist_id: u64) -> Result<Targets, CoreError> {
    let lang = prefs::load_preferences(conn)?.title_language;
    let media: Option<serde_json::Value> = conn
        .query_row(
            "SELECT media FROM rec_shows WHERE anilist_id = ?1 AND media IS NOT NULL",
            params![anilist_id as i64],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|t| serde_json::from_str(&t).ok());
    let library: Option<LibraryMedia> = conn
        .query_row(
            "SELECT mal_id, episodes, title_romaji, title_english, title_native FROM anilist_media
              WHERE id = ?1",
            params![anilist_id as i64],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?;
    let m = media.unwrap_or_default();
    let (lib_mal, lib_eps, romaji, english, native) = library.unwrap_or_default();
    let text = |v: &serde_json::Value| v.as_str().map(str::to_string);
    let folder_name = crate::foundation::titles::resolve(
        lang,
        text(&m["title"]["romaji"]).or(romaji).as_deref(),
        text(&m["title"]["english"]).or(english).as_deref(),
        text(&m["title"]["native"]).or(native).as_deref(),
        &format!("AniList {anilist_id}"),
    );
    Ok(Targets {
        series: 0,
        anilist: Some(anilist_id),
        mal: m["idMal"].as_u64().or(lib_mal.map(|id| id as u64)),
        total: m["episodes"]
            .as_u64()
            .or(lib_eps.map(|e| e as u64))
            .and_then(|e| u32::try_from(e).ok())
            .filter(|total| *total > 0),
        folder_name,
    })
}

/// The series the library has for an AniList id, when it has one.
fn series_of(core: &Core, anilist_id: u64) -> Result<Option<u64>, CoreError> {
    core.store.read(|c| {
        Ok(c.query_row(
            "SELECT MIN(id) FROM series WHERE anilist_id = ?1 AND missing_since IS NULL",
            params![anilist_id as i64],
            |r| r.get::<_, Option<i64>>(0),
        )?
        .map(|id| id as u64))
    })
}

/// A list status for a show the library does not have, on every connected
/// tracker (#779). With no tracker there is nowhere to keep it: the list on
/// this device is kept by series.
pub fn set_show_list_status(
    core: &Core,
    anilist_id: u64,
    status: ListStatus,
) -> Result<Option<u64>, CoreError> {
    if let Some(series) = series_of(core, anilist_id)? {
        return set_list_status(core, series, status);
    }
    if status == ListStatus::Repeating {
        return Err(CoreError::invalid(
            "status",
            tr("core-trackers-rewatch-not-settable", &[]),
        ));
    }
    let target = core.store.read(|c| show_targets(c, anilist_id))?;
    guard(core, &target)?;
    start(core, JobKind::SetStatus, target, Action::Status { status }).map(Some)
}

/// A score out of 10 for a show the library does not have, on every
/// connected tracker; nought clears it.
pub fn set_show_score(core: &Core, anilist_id: u64, score: f64) -> Result<Option<u64>, CoreError> {
    if let Some(series) = series_of(core, anilist_id)? {
        return set_score(core, series, Some(score));
    }
    if !(0.0..=10.0).contains(&score) {
        return Err(CoreError::invalid(
            "score",
            tr(
                "core-trackers-score-out-of-range",
                &[("score", score.to_string().into())],
            ),
        ));
    }
    let target = core.store.read(|c| show_targets(c, anilist_id))?;
    guard(core, &target)?;
    start(core, JobKind::Score, target, Action::Score { score }).map(Some)
}

/// Whether a status or a score stays on this device: the series has no
/// tracker match, or no account is connected behind the ids it has. A
/// hidden series is still refused, since hiding is about its tracker sync
/// and a person who hid it asked for nothing to go out.
fn kept_here(core: &Core, target: &Targets) -> Result<bool, CoreError> {
    // `guard` answers NoMatch before it looks at the hidden flag, which a
    // series with no ids has nothing else to say about.
    let hidden = core.store.read(|c| {
        Ok(c.query_row(
            "SELECT hidden FROM series WHERE id = ?1",
            params![target.series as i64],
            |r| r.get::<_, i64>(0),
        )? == 1)
    })?;
    if hidden {
        return Err(CoreError::Refused {
            reason: Refusal::Hidden,
        });
    }
    match guard(core, target) {
        Ok(_) => Ok(false),
        Err(CoreError::Refused {
            reason: Refusal::NoMatch,
        })
        | Err(CoreError::NotConnected { .. }) => Ok(true),
        Err(e) => Err(e),
    }
}

/// The series' card pushed after a write here, so the page redraws. The
/// write has already landed by now, so a card that cannot be read is a
/// line in the log rather than a failure of the call.
fn push_card(core: &Core, series: u64, line: &str) {
    let images_dir = core.paths.images_dir();
    match core
        .store
        .read(|c| crate::library::cards::cards_for(c, &images_dir, &[series]))
    {
        Ok(cards) => {
            core.bus.debug(
                Stage::Library,
                line.to_string(),
                EventBody::SeriesChanged { series: cards },
            );
        }
        Err(e) => tracing::warn!("{line}, but the card for series {series} was not read: {e}"),
    }
}

/// The longest note a write sends, one limit for both trackers so a note
/// never fits one and fails on the other. Short of what either keeps.
pub const NOTES_MAX: usize = 2000;

/// Writes the series' notes to every connected tracker: AniList's notes,
/// MAL's comments. Blank clears them on both.
pub fn set_notes(core: &Core, series: u64, notes: &str) -> Result<u64, CoreError> {
    let notes = notes.trim();
    if notes.chars().count() > NOTES_MAX {
        return Err(CoreError::invalid(
            "notes",
            tr(
                "core-trackers-notes-too-long",
                &[("max", NOTES_MAX.to_string().into())],
            ),
        ));
    }
    let target = core.store.read(|c| targets(c, series))?;
    guard(core, &target)?;
    let notes = cache::note_of(Some(notes));
    start(core, JobKind::Notes, target, Action::Notes { notes })
}

/// A score or a list status for several series at once (#208).
#[derive(Clone, Copy, Debug)]
pub enum ManyWrite {
    Score(Option<f64>),
    Status(ListStatus),
}

/// Writes one score or one list status to every pick, in one job whose
/// terminal is the only line the activity log gets. A pick the guards turn
/// away (no match, hidden, no account behind it) is left out and counted;
/// when every pick is turned away, the first refusal is the answer, as a
/// single write would give it.
///
/// A pick no connected tracker carries keeps the score or the status on
/// this device instead (#213), as one menu's write does. None when every
/// pick was kept here, so there was no job to start.
pub fn start_many(core: &Core, series: &[u64], write: ManyWrite) -> Result<Option<u64>, CoreError> {
    let action = match write {
        ManyWrite::Score(score) => {
            let score = score.unwrap_or(0.0);
            if !(0.0..=10.0).contains(&score) {
                return Err(CoreError::invalid(
                    "score",
                    tr(
                        "core-trackers-score-out-of-range",
                        &[("score", score.to_string().into())],
                    ),
                ));
            }
            Action::Score { score }
        }
        ManyWrite::Status(ListStatus::Repeating) => {
            return Err(CoreError::invalid(
                "status",
                tr("core-trackers-rewatch-not-settable", &[]),
            ));
        }
        ManyWrite::Status(status) => Action::Status { status },
    };
    let mut reachable = Vec::new();
    let mut refused = None;
    let mut left_out = LeftOut::default();
    let now = time::now_secs();
    for &id in series {
        let target = core.store.read(|c| targets(c, id))?;
        match kept_here(core, &target) {
            Ok(false) => reachable.push(target),
            Ok(true) => {
                core.store.write(move |c| match write {
                    ManyWrite::Score(score) => {
                        local::set_score(c, id, score.filter(|s| *s > 0.0), now)
                    }
                    ManyWrite::Status(status) => local::set_status(c, id, status, now),
                })?;
                push_card(core, id, "kept here");
                left_out.kept += 1;
            }
            Err(e) => {
                match &e {
                    CoreError::Refused {
                        reason: Refusal::Hidden,
                    } => left_out.hidden += 1,
                    CoreError::NotConnected { .. } => left_out.not_connected += 1,
                    _ => left_out.no_match += 1,
                }
                refused.get_or_insert(e);
            }
        }
    }
    if reachable.is_empty() {
        if left_out.kept > 0 {
            return Ok(None);
        }
        return Err(refused.unwrap_or_else(|| {
            CoreError::invalid("series", tr("core-trackers-nothing-picked", &[]))
        }));
    }
    let kind = match action {
        Action::Score { .. } => JobKind::Score,
        _ => JobKind::SetStatus,
    };
    let owner = core.owner()?;
    let jobs = owner.jobs.clone();
    Ok(Some(jobs.start(kind, move |ctx| async move {
        run_many(owner, ctx, reachable, action, left_out).await
    })))
}

async fn run_many(
    core: Arc<Core>,
    ctx: Arc<JobCtx>,
    targets: Vec<Targets>,
    action: Action,
    left_out: LeftOut,
) -> Result<Finished, CoreError> {
    let total = targets.len() as u64;
    let (mut done, mut failed) = (0u64, 0u64);
    let mut named = None;
    let mut landed: Vec<Tracker> = Vec::new();
    let mut reason: Option<String> = None;
    for (i, target) in targets.iter().enumerate() {
        ctx.checkpoint()?;
        ctx.progress(i as u64, Some(total), &target.folder_name);
        let outcomes = write_all(&core, &ctx, target, &action, false).await?;
        if outcomes.iter().any(|o| o.ok) {
            done += 1;
            named = Some(target.folder_name.clone());
            for o in outcomes.iter().filter(|o| o.ok) {
                if !landed.contains(&o.tracker) {
                    landed.push(o.tracker);
                }
            }
        } else {
            failed += 1;
            if reason.is_none() {
                reason = outcomes.iter().find_map(|o| o.message.clone());
            }
        }
        if let Some(card) = card_for(&core, target.series).await? {
            ctx.changed(card);
        }
    }
    let sites = sites_of(landed.iter().copied());
    // One shown by name, or a count
    let (named, name) = match (done, named) {
        (1, Some(name)) => ("yes", name),
        _ => ("no", String::new()),
    };
    let (act, score, status) = match &action {
        Action::Score { score } if *score > 0.0 => ("score", score.to_string(), ""),
        Action::Score { .. } => ("clear", String::new(), ""),
        Action::Status { status } => ("status", String::new(), status.as_str()),
        Action::Mark { .. } | Action::Progress { .. } | Action::Notes { .. } if done > 0 => {
            unreachable!("not a many-write")
        }
        Action::Mark { .. } | Action::Progress { .. } | Action::Notes { .. } => {
            ("none", String::new(), "")
        }
    };
    let reason = reason.map(|r| r.trim_end_matches('.').to_string());
    let said = crate::words!(
        "core-trackers-written",
        done = done,
        failed = failed,
        action = act,
        named = named,
        name = name,
        score = score,
        status = status,
        sites = sites.unwrap_or("none"),
        hasReason = if reason.is_some() { "yes" } else { "no" },
        reason = reason.unwrap_or_default(),
        noMatch = left_out.no_match,
        hidden = left_out.hidden,
        notConnected = left_out.not_connected,
        kept = left_out.kept,
    );
    let skipped = left_out.total();
    Ok(Finished {
        level: if done == 0 { Level::Warn } else { Level::Info },
        plain: Some(said),
        message: format!(
            "{} on {} series: {done} written, {failed} failed, {skipped} left out",
            action.verb(),
            total + skipped
        ),
        body: EventBody::Notice,
    })
}

/// The picks a many-write left out, by the guard that turned each away.
#[derive(Clone, Copy, Debug, Default)]
struct LeftOut {
    no_match: u64,
    hidden: u64,
    not_connected: u64,
    /// Kept on this device instead, with no tracker to send to (#213).
    kept: u64,
}

impl LeftOut {
    /// Every pick left out. The line names each reason in a sentence of its own, so it
    /// says what to do about each (`core-trackers-left-out`).
    fn total(self) -> u64 {
        self.no_match + self.hidden + self.not_connected + self.kept
    }
}

/// The sites a write landed on, as the catalog's `$sites` names them: `anilist`, `mal`, or
/// `both` for "AniList and MyAnimeList". None when it landed nowhere.
fn sites_of(landed: impl Iterator<Item = Tracker>) -> Option<&'static str> {
    let (mut anilist, mut mal) = (false, false);
    for t in landed {
        match t {
            Tracker::Anilist => anilist = true,
            Tracker::Mal => mal = true,
        }
    }
    match (anilist, mal) {
        (true, true) => Some("both"),
        (true, false) => Some("anilist"),
        (false, true) => Some("mal"),
        (false, false) => None,
    }
}

/// The guards every write shares, in the order the shell expects them:
/// nothing to write to, a series the user hid, and no account behind
/// either id. Answers with the main tracker, which is the one the
/// `NotConnected` refusal names and the one the monotonic guard reads.
fn guard(core: &Core, target: &Targets) -> Result<Tracker, CoreError> {
    core.store.read(|c| {
        if !target.matched() {
            return Err(CoreError::Refused {
                reason: Refusal::NoMatch,
            });
        }
        for t in [Tracker::Anilist, Tracker::Mal] {
            if let Some(id) = target.id_for(t)
                && is_hidden(c, t, id)?
            {
                return Err(CoreError::Refused {
                    reason: Refusal::Hidden,
                });
            }
        }
        let main = prefs::load_main_tracker(c)?;
        let mut connected = false;
        for t in [Tracker::Anilist, Tracker::Mal] {
            if target.id_for(t).is_some()
                && accounts::load_row(c, t)?.is_some_and(|row| row.connected_at.is_some())
            {
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
fn cached_progress(
    conn: &Connection,
    target: &Targets,
    main: Tracker,
) -> Result<Option<u32>, CoreError> {
    let Some((t, id)) = cached_id(target, main) else {
        return Ok(None);
    };
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

/// What one write does to an entry. The actions share a job, a read and a
/// cache patch, so they share a type rather than near-copies. `again` on a
/// mark is a completed show finished a second time (`finished_again`).
#[derive(Clone, Debug)]
pub(crate) enum Action {
    Mark { episode: u32, again: bool },
    Progress { progress: u32 },
    Score { score: f64 },
    Status { status: ListStatus },
    Notes { notes: Option<String> },
}

impl Action {
    /// What the activity log calls this write when it fails.
    fn verb(&self) -> &'static str {
        match self {
            Action::Mark { .. } => "mark",
            Action::Progress { .. } => "set progress",
            Action::Score { .. } => "score",
            Action::Status { .. } => "set status",
            Action::Notes { .. } => "save notes",
        }
    }
}

fn start(core: &Core, kind: JobKind, target: Targets, action: Action) -> Result<u64, CoreError> {
    start_holding(core, kind, target, action, None)
}

/// `start`, with the rewatch slot the job keeps until it ends.
fn start_holding(
    core: &Core,
    kind: JobKind,
    target: Targets,
    action: Action,
    slot: Option<RewatchSlot>,
) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    let jobs = owner.jobs.clone();
    Ok(jobs.start(kind, move |ctx| async move {
        let _slot = slot;
        run(owner, ctx, target, action).await
    }))
}

async fn run(
    core: Arc<Core>,
    ctx: Arc<JobCtx>,
    target: Targets,
    action: Action,
) -> Result<Finished, CoreError> {
    let outcomes = write_all(&core, &ctx, &target, &action, true).await?;
    // The cache was patched by each write above, so the card built here
    // already carries the new number. The runner flushes this before the
    // terminal event.
    if let Some(card) = card_for(&core, target.series).await? {
        ctx.changed(card);
    }
    Ok(finished(&target, action, outcomes))
}

/// One series' write to every connected tracker its match carries an id
/// for. `loud` says each tracker's failure in the plain log; a write over
/// several series keeps them to the developer log and counts them instead.
async fn write_all(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    target: &Targets,
    action: &Action,
    loud: bool,
) -> Result<Vec<TrackerOutcome>, CoreError> {
    let mut outcomes: Vec<TrackerOutcome> = Vec::new();
    for t in [Tracker::Anilist, Tracker::Mal] {
        ctx.checkpoint()?;
        // A tracker the match carries no id for has nothing to write to,
        // and one with no account behind it is not this write's business.
        let Some(media_id) = target.id_for(t) else {
            continue;
        };
        let row = core
            .store
            .write_async(move |c| accounts::load_row(c, t))
            .await?
            .unwrap_or_default();
        if row.connected_at.is_none() {
            continue;
        }
        outcomes.push(write_one(core, ctx, t, target, media_id, row.user_id, action, loud).await);
    }
    Ok(outcomes)
}

/// The terminal's level. A write that reached a tracker and was refused by
/// every one of them is not something that happened: the line has to read
/// as the failure it is, or the log says the mark landed.
fn level_of(outcomes: &[TrackerOutcome]) -> Level {
    if !outcomes.is_empty() && outcomes.iter().all(|o| !o.ok) {
        Level::Warn
    } else {
        Level::Info
    }
}

/// The sites a write landed on, as the catalog's `$sites` names them (`sites_of`). None
/// when it landed nowhere, since each failed site has said so on its own line.
fn landed_on(outcomes: &[TrackerOutcome]) -> Option<&'static str> {
    sites_of(outcomes.iter().filter(|o| o.ok).map(|o| o.tracker))
}

fn finished(target: &Targets, action: Action, outcomes: Vec<TrackerOutcome>) -> Finished {
    let series = target.series;
    let folder = &target.folder_name;
    let level = level_of(&outcomes);
    let sites = landed_on(&outcomes);
    match action {
        Action::Mark { episode, .. } => Finished {
            plain: sites.map(|s| {
                crate::words!(
                    "core-trackers-marked",
                    episode = episode,
                    show = folder,
                    sites = s
                )
            }),
            level,
            message: format!("Marked episode {episode} of {folder}"),
            body: EventBody::Marked {
                series,
                episode,
                outcomes,
            },
        },
        Action::Progress { progress } => Finished {
            plain: sites.map(|s| {
                crate::words!(
                    "core-trackers-progress-set",
                    progress = progress,
                    show = folder,
                    sites = s
                )
            }),
            level,
            message: format!("Progress of {folder} set to {progress}"),
            body: EventBody::ProgressSet {
                series,
                progress,
                outcomes,
            },
        },
        Action::Score { score } => {
            // A score of nought is unrated on both trackers, so the event
            // carries no score rather than a rating of nothing.
            let rated = (score > 0.0).then_some(score);
            Finished {
                plain: sites.map(|s| match rated {
                    Some(score) => crate::words!(
                        "core-trackers-scored",
                        score = score.to_string(),
                        show = folder,
                        sites = s
                    ),
                    None => crate::words!("core-trackers-score-cleared", show = folder, sites = s),
                }),
                level,
                message: match rated {
                    Some(score) => format!("score of {folder} set to {score}"),
                    None => format!("score of {folder} cleared"),
                },
                body: EventBody::Scored {
                    series,
                    score: rated,
                    outcomes,
                },
            }
        }
        Action::Status { status } => Finished {
            plain: sites.map(|s| {
                crate::words!(
                    "core-trackers-status-set",
                    status = status.as_str(),
                    show = folder,
                    sites = s
                )
            }),
            level,
            message: format!("status of {folder} set to {}", status.as_str()),
            body: EventBody::ListStatusSet {
                series,
                status,
                outcomes,
            },
        },
        Action::Notes { notes } => {
            let cleared = notes.is_none();
            Finished {
                plain: sites.map(|s| match cleared {
                    false => crate::words!("core-trackers-notes-saved", show = folder, sites = s),
                    true => crate::words!("core-trackers-notes-cleared", show = folder, sites = s),
                }),
                level,
                message: match cleared {
                    false => format!("notes of {folder} saved"),
                    true => format!("notes of {folder} cleared"),
                },
                body: EventBody::NotesSet {
                    series,
                    notes,
                    outcomes,
                },
            }
        }
    }
}

/// One tracker's outcome, whatever happened. A provider failure is this
/// tracker's alone: it becomes a `ok: false` outcome with a line the user
/// can act on and an Error line carrying the whole message, and the job
/// carries on to the other tracker.
#[allow(clippy::too_many_arguments)]
async fn write_one(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    t: Tracker,
    target: &Targets,
    media_id: u64,
    user_id: Option<u64>,
    action: &Action,
    loud: bool,
) -> TrackerOutcome {
    match attempt(core, ctx, t, target, media_id, user_id, action).await {
        Ok(outcome) => outcome,
        Err(e) if !loud => {
            ctx.emit(
                Level::Debug,
                format!(
                    "{} {} failed for {}: {}",
                    t.as_str(),
                    action.verb(),
                    target.folder_name,
                    message_of(&e)
                ),
                EventBody::Notice,
            );
            TrackerOutcome {
                tracker: t,
                ok: false,
                progress: None,
                reason: None,
                message: Some(sanitize_error(t, &e)),
            }
        }
        Err(e) => {
            ctx.say(
                Level::Error,
                crate::words!(
                    "core-trackers-update-failed",
                    site = super::oauth::site_name(t)
                ),
                format!(
                    "{} {} failed: {}",
                    t.as_str(),
                    action.verb(),
                    message_of(&e)
                ),
                EventBody::Notice,
            );
            TrackerOutcome {
                tracker: t,
                ok: false,
                progress: None,
                reason: None,
                message: Some(sanitize_error(t, &e)),
            }
        }
    }
}

/// The write itself: the tracker's current entry, the action's own guard,
/// the mutation, the account's `synced_at`, the cache patch and the line
/// the activity log gets.
pub(crate) async fn attempt(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    t: Tracker,
    target: &Targets,
    media_id: u64,
    user_id: Option<u64>,
    action: &Action,
) -> Result<TrackerOutcome, CoreError> {
    let token = accounts::access_token(core, t)
        .await?
        .ok_or_else(|| accounts::tracker_error(t, no_token()))?;
    let entry = read_entry(core, t, media_id, user_id, &token).await?;
    let current = entry.progress;
    let today = today();
    match action {
        &Action::Mark { episode, again } => {
            // The monotonic rule, decided against the tracker itself: a
            // mark only ever moves progress up, and a list already past
            // this episode is not wrong, it is ahead. A completed show
            // finished again is the one exception, and it counts a rewatch.
            let rewatch = again && entry.status == Some(ListStatus::Completed);
            if current >= episode && !rewatch {
                return Ok(TrackerOutcome {
                    tracker: t,
                    ok: false,
                    progress: Some(current),
                    reason: Some(Refusal::NotNewer),
                    message: None,
                });
            }
            let progress = current.max(episode);
            let status = watched_status(&entry, target.completes(progress));
            let extras = plan(&entry, status, true, rewatch, today);
            let stamp = send_progress(core, t, media_id, &token, progress, status, &extras).await?;
            record_progress(core, t, media_id, progress, status, stamp).await?;
            record_repeat(core, t, media_id, &extras).await?;
            // The terminal event carries every tracker's outcome, so a
            // line per tracker here would say the same thing twice.
            ctx.emit(
                Level::Debug,
                format!(
                    "{} {current} -> {progress} (mediaId {media_id}){}",
                    t.as_str(),
                    extras.tail()
                ),
                EventBody::Notice,
            );
            Ok(TrackerOutcome {
                tracker: t,
                ok: true,
                progress: Some(progress),
                reason: None,
                message: None,
            })
        }
        &Action::Progress { progress } => {
            let status = if progress == 0 {
                ListStatus::Planning
            } else {
                watched_status(&entry, target.completes(progress))
            };
            let extras = plan(&entry, status, progress > 0, false, today);
            let stamp = send_progress(core, t, media_id, &token, progress, status, &extras).await?;
            record_progress(core, t, media_id, progress, status, stamp).await?;
            record_repeat(core, t, media_id, &extras).await?;
            ctx.emit(
                Level::Debug,
                format!(
                    "{} set {current} -> {progress} (mediaId {media_id}){}",
                    t.as_str(),
                    extras.tail()
                ),
                EventBody::Notice,
            );
            Ok(TrackerOutcome {
                tracker: t,
                ok: true,
                progress: Some(progress),
                reason: None,
                message: None,
            })
        }
        &Action::Score { score } => {
            // Rating a series whose every episode is already watched is
            // also saying it is finished, which is what the list should
            // have said all along.
            let completed = target.total.is_some_and(|total| current >= total);
            let extras = match completed {
                true => plan(&entry, ListStatus::Completed, false, false, today),
                false => Extras::default(),
            };
            let (sent, stamp) =
                send_score(core, t, media_id, &token, score, completed, &extras).await?;
            record_score(core, t, media_id, sent, completed, stamp).await?;
            record_repeat(core, t, media_id, &extras).await?;
            let tail = if completed { " + completed" } else { "" };
            ctx.emit(
                Level::Debug,
                format!(
                    "{} score -> {sent} (mediaId {media_id}){tail}{}",
                    t.as_str(),
                    extras.tail()
                ),
                EventBody::Notice,
            );
            Ok(TrackerOutcome {
                tracker: t,
                ok: true,
                progress: None,
                reason: None,
                message: None,
            })
        }
        &Action::Status { status } => {
            // Completed fills the list to the published total, the way a
            // rating that completes does; every other status leaves the
            // progress as the tracker has it.
            let progress = match (status, target.total) {
                (ListStatus::Completed, Some(total)) if current < total => total,
                _ => current,
            };
            let fill = (progress != current).then_some(progress);
            let extras = plan(&entry, status, false, false, today);
            let stamp = send_list_status(core, t, media_id, &token, status, fill, &extras).await?;
            // A series not on the list yet has no cached entry to patch, so
            // the entry is upserted with the progress the tracker now has.
            record_progress(core, t, media_id, progress, status, stamp).await?;
            ctx.emit(
                Level::Debug,
                format!(
                    "{} status -> {} (mediaId {media_id}){}",
                    t.as_str(),
                    status.as_str(),
                    extras.tail()
                ),
                EventBody::Notice,
            );
            Ok(TrackerOutcome {
                tracker: t,
                ok: true,
                progress: Some(progress),
                reason: None,
                message: None,
            })
        }
        Action::Notes { notes } => {
            // Notes on a show the list does not hold would add it to the
            // list, on a list the person never picked.
            if entry.status.is_none() {
                return Ok(TrackerOutcome {
                    tracker: t,
                    ok: false,
                    progress: None,
                    reason: None,
                    message: Some(tr(
                        "core-trackers-notes-not-on-list",
                        &[("tracker", t.label().into())],
                    )),
                });
            }
            let stamp = send_notes(core, t, media_id, &token, notes.as_deref()).await?;
            record_notes(core, t, media_id, notes.clone(), stamp).await?;
            ctx.emit(
                Level::Debug,
                format!("{} notes saved (mediaId {media_id})", t.as_str()),
                EventBody::Notice,
            );
            Ok(TrackerOutcome {
                tracker: t,
                ok: true,
                progress: Some(current),
                reason: None,
                message: None,
            })
        }
    }
}

// Dates and rewatches --------------------------------------------------------

/// A list entry as a write reads it first: how far it has got, its status,
/// whether it carries a start and a finish date, and how many rewatches.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Current {
    pub progress: u32,
    pub status: Option<ListStatus>,
    pub started: bool,
    pub finished: bool,
    pub repeat: u32,
}

/// A calendar day, year, month and day.
pub(crate) type Day = (i32, u8, u8);

/// Today, as the tracker's dates read it. The core keeps UTC everywhere,
/// and a tracker keeps a day with no time, so this is the UTC day.
fn today() -> Day {
    let t = ::time::OffsetDateTime::from_unix_timestamp(time::now_secs())
        .unwrap_or(::time::OffsetDateTime::UNIX_EPOCH);
    (t.year(), u8::from(t.month()), t.day())
}

/// What a write adds beside its progress and status: a start date, a
/// finish date, and a rewatch count one up. Each is only ever filled in,
/// never moved: a date the list already has stays as it is.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Extras {
    pub start: Option<Day>,
    pub finish: Option<Day>,
    pub repeat: Option<u32>,
}

impl Extras {
    /// The end of the debug line, naming what rode along.
    fn tail(&self) -> String {
        let mut out = String::new();
        if self.start.is_some() {
            out.push_str(" + started");
        }
        if self.finish.is_some() {
            out.push_str(" + finished");
        }
        if let Some(repeat) = self.repeat {
            out.push_str(&format!(" + rewatch {repeat}"));
        }
        out
    }
}

/// The status a watched count sends: Completed when it reaches the total,
/// and otherwise Watching, or Repeating for a rewatch under way, which a
/// mark must not quietly turn back into a first watch.
fn watched_status(entry: &Current, completes: bool) -> ListStatus {
    match (completes, entry.status) {
        (true, _) => ListStatus::Completed,
        (false, Some(ListStatus::Repeating)) => ListStatus::Repeating,
        (false, _) => ListStatus::Watching,
    }
}

/// The dates and the rewatch a write sends, when the list has none.
///
/// - The start date: the write puts the entry on Watching from another
///   list, or it is a watched count leaving nought, so a film watched in
///   one go gets both dates.
/// - The finish date: the write puts the entry on Completed from another
///   list. Completing it from the status menu says when it was finished,
///   not when it was started, so that alone sets no start date.
/// - The rewatch: a watched count that completes a Repeating entry, or a
///   completed show finished again (`again`), counts one more. The status
///   menu and a rating never count one, on either tracker: they say where
///   the list stands, not that an episode was watched.
pub(crate) fn plan(
    entry: &Current,
    sending: ListStatus,
    watched: bool,
    again: bool,
    today: Day,
) -> Extras {
    let was = entry.status;
    let starts = (sending == ListStatus::Watching
        && !matches!(was, Some(ListStatus::Watching | ListStatus::Repeating)))
        || (watched && entry.progress == 0);
    let finishes = sending == ListStatus::Completed && was != Some(ListStatus::Completed);
    let rewatched = sending == ListStatus::Completed
        && watched
        && (was == Some(ListStatus::Repeating) || (was == Some(ListStatus::Completed) && again));
    Extras {
        start: (starts && !entry.started).then_some(today),
        finish: (finishes && !entry.finished).then_some(today),
        repeat: rewatched.then(|| entry.repeat.saturating_add(1)),
    }
}

/// The extras as AniList's variables: a `FuzzyDateInput` per date and the
/// count. A key left out leaves the field alone.
fn anilist_extras(variables: &mut serde_json::Value, extras: &Extras) {
    let date =
        |(year, month, day): Day| serde_json::json!({ "year": year, "month": month, "day": day });
    if let Some(day) = extras.start {
        variables["startedAt"] = date(day);
    }
    if let Some(day) = extras.finish {
        variables["completedAt"] = date(day);
    }
    if let Some(repeat) = extras.repeat {
        variables["repeat"] = serde_json::json!(repeat);
    }
}

/// The same as MAL's form fields. A rewatch finished is a rewatch no longer
/// under way, so the flag goes with the count.
fn mal_extras(form: &mut Vec<(String, String)>, extras: &Extras) {
    let date = |(year, month, day): Day| format!("{year:04}-{month:02}-{day:02}");
    if let Some(day) = extras.start {
        form.push(("start_date".to_string(), date(day)));
    }
    if let Some(day) = extras.finish {
        form.push(("finish_date".to_string(), date(day)));
    }
    if let Some(repeat) = extras.repeat {
        form.push(("num_times_rewatched".to_string(), repeat.to_string()));
        form.push(("is_rewatching".to_string(), "false".to_string()));
    }
}

/// Whether an AniList fuzzy date holds anything: a day and month with no
/// year is still a date the person wrote, and is never written over.
fn fuzzy_dated(date: &serde_json::Value) -> bool {
    ["year", "month", "day"]
        .iter()
        .any(|k| date[*k].as_u64().is_some())
}

// The provider calls ---------------------------------------------------------

/// The entry as the tracker itself has it. An anime the user has never
/// added is an empty entry rather than a failure: that is exactly what a
/// first mark is for, on either tracker.
async fn read_entry(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    user_id: Option<u64>,
    token: &str,
) -> Result<Current, CoreError> {
    match t {
        Tracker::Anilist => {
            // `MediaList(mediaId)` without the user id ignores the bearer
            // token and answers with some other user's entry, so a missing
            // user id is a broken account rather than a reason to guess.
            let user_id = user_id
                .ok_or_else(|| accounts::tracker_error(t, tr("core-trackers-no-user-id", &[])))?;
            let variables = serde_json::json!({ "userId": user_id, "mediaId": media_id });
            match core
                .anilist
                .graphql_within(
                    MEDIA_LIST_ENTRY_QUERY,
                    variables,
                    Some(token),
                    TRACKER_TIMEOUT,
                )
                .await
            {
                Ok(data) => {
                    let entry = &data["MediaList"];
                    Ok(Current {
                        progress: as_count(entry["progress"].as_u64()),
                        status: cache::normalize_status(t, entry["status"].as_str(), false),
                        started: fuzzy_dated(&entry["startedAt"]),
                        finished: fuzzy_dated(&entry["completedAt"]),
                        repeat: as_count(entry["repeat"].as_u64()),
                    })
                }
                Err(e) if e.is_provider_not_found() => Ok(Current::default()),
                Err(e) => Err(e),
            }
        }
        Tracker::Mal => {
            let url = format!(
                "{MAL_API}/anime/{media_id}?fields=my_list_status{{status,num_episodes_watched,is_rewatching,num_times_rewatched,start_date,finish_date}}"
            );
            let response = core.mal.get(&url, token).await?;
            if response.status == 404 {
                return Ok(Current::default());
            }
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
            let body: serde_json::Value = response.json()?;
            let list = &body["my_list_status"];
            let dated = |key: &str| list[key].as_str().is_some_and(|d| !d.is_empty());
            Ok(Current {
                progress: as_count(list["num_episodes_watched"].as_u64()),
                status: cache::normalize_status(
                    t,
                    list["status"].as_str(),
                    list["is_rewatching"].as_bool().unwrap_or(false),
                ),
                started: dated("start_date"),
                finished: dated("finish_date"),
                repeat: as_count(list["num_times_rewatched"].as_u64()),
            })
        }
    }
}

/// An entry as the tracker has it now, with the media's airing state, for
/// On hold's last look before a status write. `None` is an entry the list
/// no longer holds. MAL has no broadcast schedule, so its `next` is empty
/// and the library's schedule rows date its next episode.
pub(crate) async fn read_live(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    user_id: Option<u64>,
    token: &str,
) -> Result<Option<on_hold::Live>, CoreError> {
    match t {
        Tracker::Anilist => {
            let user_id = user_id
                .ok_or_else(|| accounts::tracker_error(t, tr("core-trackers-no-user-id", &[])))?;
            let variables = serde_json::json!({ "userId": user_id, "mediaId": media_id });
            let data = match core
                .anilist
                .graphql_within(ON_HOLD_ENTRY_QUERY, variables, Some(token), TRACKER_TIMEOUT)
                .await
            {
                Ok(data) => data,
                Err(e) if e.is_provider_not_found() => return Ok(None),
                Err(e) => return Err(e),
            };
            let entry = &data["MediaList"];
            if entry.is_null() {
                return Ok(None);
            }
            let media = &entry["media"];
            let next = &media["nextAiringEpisode"];
            Ok(Some(on_hold::Live {
                status: cache::normalize_status(t, entry["status"].as_str(), false),
                progress: as_count(entry["progress"].as_u64()),
                updated_at: entry["updatedAt"].as_i64().filter(|at| *at > 0),
                total: as_u32(media["episodes"].as_u64()).filter(|n| *n > 0),
                still_coming: media["status"].as_str().is_some_and(on_hold::still_coming),
                next: as_u32(next["episode"].as_u64()).zip(next["airingAt"].as_i64()),
            }))
        }
        Tracker::Mal => {
            let url =
                format!("{MAL_API}/anime/{media_id}?fields=my_list_status,status,num_episodes");
            let response = core.mal.get(&url, token).await?;
            if response.status == 404 {
                return Ok(None);
            }
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
            let body: serde_json::Value = response.json()?;
            let list = &body["my_list_status"];
            if list.is_null() {
                return Ok(None);
            }
            Ok(Some(on_hold::Live {
                status: cache::normalize_status(
                    t,
                    list["status"].as_str(),
                    list["is_rewatching"].as_bool().unwrap_or(false),
                ),
                progress: as_count(list["num_episodes_watched"].as_u64()),
                updated_at: list["updated_at"].as_str().and_then(cache::mal_instant),
                total: as_u32(body["num_episodes"].as_u64()).filter(|n| *n > 0),
                still_coming: body["status"].as_str().is_some_and(on_hold::still_coming),
                next: None,
            }))
        }
    }
}

/// The progress and the status, with whatever dates and rewatch count ride
/// along, answering with the tracker's own stamp for the write when its
/// reply carried one.
async fn send_progress(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    token: &str,
    progress: u32,
    status: ListStatus,
    extras: &Extras,
) -> Result<Option<i64>, CoreError> {
    match t {
        Tracker::Anilist => {
            let mut variables = serde_json::json!({ "mediaId": media_id, "progress": progress, "status": anilist_status(status) });
            anilist_extras(&mut variables, extras);
            let data = core
                .anilist
                .graphql_within(
                    SAVE_PROGRESS_MUTATION,
                    variables,
                    Some(token),
                    TRACKER_TIMEOUT,
                )
                .await?;
            Ok(anilist_stamp(&data))
        }
        Tracker::Mal => {
            let mut form = vec![
                ("num_watched_episodes".to_string(), progress.to_string()),
                ("status".to_string(), mal_status(status).to_string()),
            ];
            mal_extras(&mut form, extras);
            let response = core
                .mal
                .patch_form(&list_status_url(media_id), token, form)
                .await?;
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
            Ok(mal_stamp(&response))
        }
    }
}

/// The stamp a `SaveMediaListEntry` reply carries.
fn anilist_stamp(data: &serde_json::Value) -> Option<i64> {
    data["SaveMediaListEntry"]["updatedAt"]
        .as_i64()
        .filter(|at| *at > 0)
}

/// MAL answers a list PATCH with the whole list status, its `updated_at`
/// among it.
fn mal_stamp(response: &HttpResponse) -> Option<i64> {
    response
        .json::<serde_json::Value>()
        .ok()
        .and_then(|body| body["updated_at"].as_str().and_then(cache::mal_instant))
}

/// The score each tracker actually receives, which is not the same number:
/// AniList takes `scoreRaw` out of a hundred, so one write works whatever
/// display format the user picked and a decimal survives it; MAL takes a
/// whole number and rounds the way its own UI rounds a typed decimal. What
/// was sent comes back, so the cache and the log line agree with the list,
/// and beside it the tracker's stamp for the write when its reply carried one.
async fn send_score(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    token: &str,
    score: f64,
    completed: bool,
    extras: &Extras,
) -> Result<(f64, Option<i64>), CoreError> {
    match t {
        Tracker::Anilist => {
            let mut variables = serde_json::json!({ "mediaId": media_id, "scoreRaw": (score * 10.0).round() as i64 });
            // A mutation sent without the status variable leaves the
            // entry's own status alone, which is what a rating that
            // completes nothing should do.
            if completed {
                variables["status"] =
                    serde_json::Value::String(anilist_status(ListStatus::Completed).to_string());
            }
            anilist_extras(&mut variables, extras);
            let data = core
                .anilist
                .graphql_within(SAVE_SCORE_MUTATION, variables, Some(token), TRACKER_TIMEOUT)
                .await?;
            Ok((score, anilist_stamp(&data)))
        }
        Tracker::Mal => {
            let rounded = score.round();
            let mut form = vec![("score".to_string(), (rounded as i64).to_string())];
            if completed {
                form.push((
                    "status".to_string(),
                    mal_status(ListStatus::Completed).to_string(),
                ));
            }
            mal_extras(&mut form, extras);
            let response = core
                .mal
                .patch_form(&list_status_url(media_id), token, form)
                .await?;
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
            Ok((rounded, mal_stamp(&response)))
        }
    }
}

/// A status alone, which is what the On hold sweep sends: the progress and
/// the score stay as the list has them on both trackers, since neither
/// write names them. Answers with the tracker's own stamp for the change
/// when its reply carried one.
pub(crate) async fn send_status(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    token: &str,
    status: ListStatus,
) -> Result<Option<i64>, CoreError> {
    match t {
        Tracker::Anilist => {
            let variables =
                serde_json::json!({ "mediaId": media_id, "status": anilist_status(status) });
            let data = core
                .anilist
                .graphql_within(
                    SAVE_STATUS_MUTATION,
                    variables,
                    Some(token),
                    TRACKER_TIMEOUT,
                )
                .await?;
            Ok(anilist_stamp(&data))
        }
        Tracker::Mal => {
            let form = vec![("status".to_string(), mal_status(status).to_string())];
            let response = core
                .mal
                .patch_form(&list_status_url(media_id), token, form)
                .await?;
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
            Ok(mal_stamp(&response))
        }
    }
}

/// A status the user picked from the menu. MAL keeps a rewatch as a flag
/// beside the status, so the flag is cleared here, or a rewatch moved to
/// Watching would read back as Rewatching on the next refresh. `fill` is the
/// progress Completed raises the list to, when it is short of the total.
async fn send_list_status(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    token: &str,
    status: ListStatus,
    fill: Option<u32>,
    extras: &Extras,
) -> Result<Option<i64>, CoreError> {
    match (t, fill) {
        (Tracker::Anilist, Some(progress)) => {
            send_progress(core, t, media_id, token, progress, status, extras).await
        }
        (Tracker::Anilist, None) => {
            let mut variables =
                serde_json::json!({ "mediaId": media_id, "status": anilist_status(status) });
            anilist_extras(&mut variables, extras);
            let data = core
                .anilist
                .graphql_within(
                    SAVE_LIST_STATUS_MUTATION,
                    variables,
                    Some(token),
                    TRACKER_TIMEOUT,
                )
                .await?;
            Ok(anilist_stamp(&data))
        }
        (Tracker::Mal, fill) => {
            let mut form = vec![
                ("status".to_string(), mal_status(status).to_string()),
                ("is_rewatching".to_string(), "false".to_string()),
            ];
            if let Some(progress) = fill {
                form.push(("num_watched_episodes".to_string(), progress.to_string()));
            }
            mal_extras(&mut form, extras);
            let response = core
                .mal
                .patch_form(&list_status_url(media_id), token, form)
                .await?;
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
            Ok(mal_stamp(&response))
        }
    }
}

/// The entry's notes alone: AniList's notes, MAL's comments. None clears
/// them, which both trackers read from an empty string.
async fn send_notes(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    token: &str,
    notes: Option<&str>,
) -> Result<Option<i64>, CoreError> {
    let text = notes.unwrap_or("").to_string();
    match t {
        Tracker::Anilist => {
            let variables = serde_json::json!({ "mediaId": media_id, "notes": text });
            let data = core
                .anilist
                .graphql_within(SAVE_NOTES_MUTATION, variables, Some(token), TRACKER_TIMEOUT)
                .await?;
            Ok(anilist_stamp(&data))
        }
        Tracker::Mal => {
            let form = vec![("comments".to_string(), text)];
            let response = core
                .mal
                .patch_form(&list_status_url(media_id), token, form)
                .await?;
            if !response.is_success() {
                return Err(mal_failure(&response));
            }
            Ok(mal_stamp(&response))
        }
    }
}

fn list_status_url(media_id: u64) -> String {
    format!("{MAL_API}/anime/{media_id}/my_list_status")
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
    CoreError::provider(Provider::Mal, Some(u32::from(response.status)), message)
}

// What is left behind --------------------------------------------------------

/// The account's `synced_at` and the entry's progress, in one transaction:
/// two tables move together or neither does.
pub(crate) async fn record_progress(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    progress: u32,
    status: ListStatus,
    stamp: Option<i64>,
) -> Result<(), CoreError> {
    let now = time::now_secs();
    core.store
        .tx_async(move |tx| {
            mark_synced(tx, t, now)?;
            cache::patch_progress(tx, t, media_id, progress, Some(status), stamp, now)
        })
        .await
}

/// The same for a rating. A score of nought is unrated rather than a
/// rating of nothing, and a status is only written when the rating
/// completed the entry.
async fn record_score(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    score: f64,
    completed: bool,
    stamp: Option<i64>,
) -> Result<(), CoreError> {
    let now = time::now_secs();
    let rated = (score > 0.0).then_some(score);
    let status = completed.then_some(ListStatus::Completed);
    core.store
        .tx_async(move |tx| {
            mark_synced(tx, t, now)?;
            cache::patch_score(tx, t, media_id, rated, status, stamp, now)
        })
        .await
}

/// The rewatch count a write sent, when it sent one.
async fn record_repeat(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    extras: &Extras,
) -> Result<(), CoreError> {
    let Some(repeat) = extras.repeat else {
        return Ok(());
    };
    let now = time::now_secs();
    core.store
        .write_async(move |c| cache::patch_repeat(c, t, media_id, repeat, now))
        .await
}

/// The same for a notes write.
async fn record_notes(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    notes: Option<String>,
    stamp: Option<i64>,
) -> Result<(), CoreError> {
    let now = time::now_secs();
    core.store
        .tx_async(move |tx| {
            mark_synced(tx, t, now)?;
            cache::patch_notes(tx, t, media_id, notes.as_deref(), stamp, now)
        })
        .await
}

/// The same for a status On hold sent, with the tracker's stamp for it
/// when the reply carried one.
pub(crate) async fn record_status(
    core: &Arc<Core>,
    t: Tracker,
    media_id: u64,
    status: ListStatus,
    stamp: Option<i64>,
) -> Result<(), CoreError> {
    let now = time::now_secs();
    core.store
        .tx_async(move |tx| {
            mark_synced(tx, t, now)?;
            cache::patch_status(tx, t, media_id, status, stamp, now)
        })
        .await
}

fn mark_synced(conn: &Connection, t: Tracker, now: i64) -> Result<(), CoreError> {
    conn.execute(
        "UPDATE tracker_accounts SET synced_at = ?2 WHERE tracker = ?1",
        params![t.as_str(), now],
    )?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The status is read first, whatever the provider said, because a 429
    /// or a 401 is something the user can act on and "Too Many Requests."
    /// is not.
    #[test]
    fn a_status_decides_the_line_before_anything_the_provider_said() {
        let said = |status| {
            sanitize(
                Tracker::Anilist,
                Some(status),
                Some("Too Many Requests."),
                None,
                "raw",
            )
        };
        assert_eq!(said(429), "AniList rate limited, try again in a minute.");
        assert_eq!(said(401), "AniList auth expired, reconnect in Settings.");
        assert_eq!(said(403), "AniList auth expired, reconnect in Settings.");
        assert_eq!(said(404), "AniList entry not found.");
        assert_eq!(said(500), "AniList server error (500), try again later.");
        assert_eq!(said(503), "AniList server error (503), try again later.");
        // The label follows the tracker, so a MAL failure never says
        // AniList at the user.
        assert_eq!(
            sanitize(Tracker::Mal, Some(429), None, None, "raw"),
            "MAL rate limited, try again in a minute."
        );
    }

    /// Below the statuses that mean something, the provider's own words are
    /// the best line there is: GraphQL's first, then MAL's, then whatever
    /// the error itself carried.
    #[test]
    fn the_first_short_message_wins_in_order() {
        assert_eq!(
            sanitize(Tracker::Anilist, None, Some("gql"), Some("mal"), "raw"),
            "gql"
        );
        assert_eq!(
            sanitize(Tracker::Anilist, None, None, Some("mal"), "raw"),
            "mal"
        );
        assert_eq!(sanitize(Tracker::Anilist, None, None, None, "raw"), "raw");
        // A status with no rule of its own falls through to the messages.
        assert_eq!(
            sanitize(Tracker::Mal, Some(400), None, Some("Bad Request"), "raw"),
            "Bad Request"
        );
        // Nothing said is not a message.
        assert_eq!(
            sanitize(Tracker::Mal, None, Some(""), Some(""), "raw"),
            "raw"
        );
    }

    /// A blob of GraphQL, an HTML error page or a stack trace is worse in
    /// a toast than no message at all, so anything that long falls through
    /// to the log.
    #[test]
    fn a_long_message_falls_through_to_the_log() {
        let long = "x".repeat(600);
        assert_eq!(
            sanitize(Tracker::Anilist, None, None, None, &long),
            "AniList error, see activity log."
        );
        // The long one is skipped and the next short one still answers.
        assert_eq!(
            sanitize(Tracker::Mal, None, Some(&long), Some("short"), "raw"),
            "short"
        );
        assert_eq!(
            sanitize(Tracker::Mal, Some(302), Some(&long), Some(&long), &long),
            "MAL error, see activity log."
        );
        // Two hundred is the line: one under it reads, one on it does not.
        let edge = "y".repeat(199);
        assert_eq!(sanitize(Tracker::Anilist, None, None, None, &edge), edge);
        let over = "y".repeat(200);
        assert_eq!(
            sanitize(Tracker::Anilist, None, None, None, &over),
            "AniList error, see activity log."
        );
        assert_eq!(
            sanitize(Tracker::Anilist, None, None, None, ""),
            "AniList error, see activity log."
        );
    }

    const DAY: Day = (2026, 11, 6);

    fn entry(progress: u32, status: Option<ListStatus>, dated: bool, repeat: u32) -> Current {
        Current {
            progress,
            status,
            started: dated,
            finished: dated,
            repeat,
        }
    }

    /// Dates are only ever filled in: a list that has one keeps it,
    /// whatever the write.
    #[test]
    fn a_date_the_list_has_is_never_moved() {
        let dated = entry(0, None, true, 0);
        assert_eq!(
            plan(&dated, ListStatus::Completed, true, false, DAY),
            Extras::default()
        );
    }

    /// A film watched in one go starts and finishes the same day; the
    /// status menu's Completed dates the finish alone.
    #[test]
    fn a_watched_count_from_nought_starts_and_the_menu_only_finishes() {
        let fresh = entry(0, Some(ListStatus::Planning), false, 0);
        assert_eq!(
            plan(&fresh, ListStatus::Completed, true, false, DAY),
            Extras {
                start: Some(DAY),
                finish: Some(DAY),
                repeat: None
            }
        );
        assert_eq!(
            plan(&fresh, ListStatus::Completed, false, false, DAY),
            Extras {
                start: None,
                finish: Some(DAY),
                repeat: None
            }
        );
        // Watching from the menu starts it; staying on Watching does not.
        assert_eq!(
            plan(&fresh, ListStatus::Watching, false, false, DAY).start,
            Some(DAY)
        );
        let watching = entry(4, Some(ListStatus::Watching), false, 0);
        assert_eq!(
            plan(&watching, ListStatus::Watching, true, false, DAY),
            Extras::default()
        );
    }

    /// A rewatch counts when one finishes: from Repeating, or a completed
    /// show finished again. Completed alone, without the evidence, is not one.
    #[test]
    fn a_rewatch_counts_once_it_finishes() {
        let repeating = entry(11, Some(ListStatus::Repeating), true, 2);
        assert_eq!(
            plan(&repeating, ListStatus::Completed, true, false, DAY).repeat,
            Some(3)
        );
        assert_eq!(
            plan(&repeating, ListStatus::Repeating, true, false, DAY).repeat,
            None
        );
        let done = entry(12, Some(ListStatus::Completed), true, 0);
        assert_eq!(
            plan(&done, ListStatus::Completed, true, true, DAY).repeat,
            Some(1)
        );
        assert_eq!(
            plan(&done, ListStatus::Completed, true, false, DAY).repeat,
            None
        );
    }

    /// A mark on a rewatch under way stays Repeating until it completes.
    #[test]
    fn a_watched_count_keeps_a_rewatch_under_way() {
        let repeating = entry(3, Some(ListStatus::Repeating), true, 1);
        assert_eq!(watched_status(&repeating, false), ListStatus::Repeating);
        assert_eq!(watched_status(&repeating, true), ListStatus::Completed);
        assert_eq!(
            watched_status(&Current::default(), false),
            ListStatus::Watching
        );
    }

    /// A `CoreError` carries one message whichever provider it came from,
    /// and the status still decides the line where there is one.
    #[test]
    fn an_error_is_sanitised_by_its_status_then_its_message() {
        let limited = CoreError::provider(Provider::Anilist, Some(429), "AniList rate limited");
        assert_eq!(
            sanitize_error(Tracker::Anilist, &limited),
            "AniList rate limited, try again in a minute."
        );
        let refused = CoreError::provider(Provider::Mal, None, "connection refused");
        assert_eq!(sanitize_error(Tracker::Mal, &refused), "connection refused");
        // Anything that is not a provider failure still reads as one line.
        assert_eq!(
            sanitize_error(
                Tracker::Mal,
                &CoreError::Refused {
                    reason: Refusal::Hidden
                }
            ),
            "Cannot do that: the show is hidden."
        );
    }
}
