//! The five writes a shell asks a tracker for: a mark, a corrective
//! progress, a score, a list status and the entry's notes. Every one of them answers the shell
//! before a request is made if it can, and otherwise becomes a job that
//! writes to every connected tracker the match carries an id for. The On
//! hold sweep (`on_hold`) also sends a status alone, through
//! `Remote::send_status`.
//!
//! Three rules shape the file. A write goes to *both* trackers, because a
//! user with two accounts connected expects both lists to move, so the
//! outcome is a `TrackerOutcome` per tracker rather than one answer; a
//! provider failure is that tracker's, never the job's, so one dead
//! account cannot cost the other its write; and nothing a provider says
//! reaches the shell raw, because a GraphQL blob or an HTML error page in
//! a toast is worse than no message at all. `sanitize_error` below is that
//! last rule, and it is the port of Electron's `sanitizeTrackerError`.
//!
//! Carried from `src/main/handlers/trackerHandler.ts` (`markEpisode`,
//! `setEpisodeProgress`, `setScore` and the six provider halves under
//! them), the hidden guard in `src/main/ipc/tracker.ts`, and
//! `src/shared/hiddenMatch.ts`.

use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};
use tokio::sync::watch;

use crate::Trackers;
use crate::{TRACKER_TIMEOUT, accounts, as_count, as_u32, cache, local, mal_lookup, on_hold};
use anibeam_contract::i18n::{Arg, tr};
use anibeam_contract::*;
use anibeam_foundation::jobs::{Finished, JobCtx};
use anibeam_foundation::prefs;
use anibeam_foundation::services::Services;
use anibeam_foundation::sync::recover;
use anibeam_foundation::time;
use anibeam_library::history::How;
use anibeam_metadata::apply::card_for;
use anibeam_metadata::fetch::message_of;
use anibeam_net::anilist::{
    MEDIA_LIST_ENTRY_QUERY, ON_HOLD_ENTRY_QUERY, SAVE_LIST_STATUS_MUTATION, SAVE_NOTES_MUTATION,
    SAVE_PROGRESS_MUTATION, SAVE_SCORE_MUTATION, SAVE_STATUS_MUTATION,
};

/// MAL's list API. AniList's address lives with its queries.
const MAL_API: &str = "https://api.myanimelist.net/v2";

// The rules ------------------------------------------------------------------

/// What a user is told when a tracker write failed. The status decides the
/// line wherever there is one, because a 429 or a 401 means something the
/// user can act on and the provider's own words rarely do; below that the
/// provider's message is the line when it is short enough to read, and a
/// blob of GraphQL or an HTML error page falls through to the log. The
/// message is the one the error carries, AniList's `errors[0].message` or
/// MAL's `message` field, already lifted out of the body.
pub(crate) fn sanitize_error(t: Tracker, e: &CoreError) -> String {
    let (status, message) = match e {
        CoreError::Provider {
            status, message, ..
        } => (*status, message.clone()),
        other => (None, other.to_string()),
    };
    let label = || ("tracker", Arg::from(t.label()));
    match status {
        Some(429) => tr("core-trackers-rate-limited", &[label()]),
        Some(401 | 403) => tr("core-trackers-auth-expired", &[label()]),
        Some(404) => tr("core-trackers-entry-not-found", &[label()]),
        Some(s) if s >= 500 => tr(
            "core-trackers-server-error",
            &[label(), ("status", s.to_string().into())],
        ),
        _ if !message.is_empty() && message.chars().count() < 200 => message,
        _ => tr("core-trackers-error-see-log", &[label()]),
    }
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

/// Whether either id is hidden: [`is_hidden`] for a show that may carry
/// both.
pub(crate) fn any_hidden(
    conn: &Connection,
    anilist: Option<u64>,
    mal: Option<u64>,
) -> Result<bool, CoreError> {
    for (t, id) in [(Tracker::Anilist, anilist), (Tracker::Mal, mal)] {
        if let Some(id) = id
            && is_hidden(conn, t, id)?
        {
            return Ok(true);
        }
    }
    Ok(false)
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
    pub(crate) fn id_for(&self, t: Tracker) -> Option<u64> {
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
    let Some((anilist, mal, folder_name, None)) = row else {
        return Err(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        });
    };
    let anilist = anilist.map(|id| id as u64);
    // A match AniList gave no MAL id goes by the one MAL's search found.
    let mal = match (mal, anilist) {
        (Some(id), _) => Some(id as u64),
        (None, Some(a)) => mal_lookup::saved(conn, a)?.flatten(),
        (None, None) => None,
    };
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
        mal,
        total,
        folder_name,
    })
}

// The calls ------------------------------------------------------------------

/// Marks an episode watched on every connected tracker. The number is
/// floored, since a tracker counts whole episodes and 12.5 is still the
/// twelfth.
pub fn mark(trackers: &Arc<Trackers>, series: u64, episode: f64) -> Result<u64, CoreError> {
    let target = trackers.services.store.read(|c| targets(c, series))?;
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
    let (main, reach) = guard_reach(&trackers.services, &target, true)?;
    // A completed show finished again is a rewatch, which is the one mark
    // the list may already cover. A second mark of the same finish finds
    // the slot taken and answers with the first mark's job, so the shell
    // hears how that one went: judged as a plain mark, its own write would
    // race the rewatch's for the list's status and count (#1042).
    let finished = trackers
        .services
        .store
        .read(|c| finished_again(c, &target, main, episode))?;
    let slot = if finished {
        match trackers.rewatching.hold(series) {
            Ok(slot) => Some(slot),
            Err(Some(job)) => return Ok(job),
            // The first mark is between taking the slot and starting its
            // job, so there is no job to answer with yet.
            Err(None) => {
                return Err(CoreError::Refused {
                    reason: Refusal::NotNewer,
                });
            }
        }
    } else {
        None
    };
    let again = slot.is_some();
    // The cheap half of the monotonic rule: the cache already knows how
    // far each tracker says the user has got, so a mark every tracker it
    // reaches already covers is refused without a request. One tracker
    // behind is enough for the job (#782): MAL connected after AniList, or
    // a list moved elsewhere, still gets its write. The job checks again
    // against each tracker itself, which is what actually decides.
    if !again
        && trackers
            .services
            .store
            .read(|c| covered(c, &target, &reach, episode))?
    {
        return Err(CoreError::Refused {
            reason: Refusal::NotNewer,
        });
    }
    start_holding(
        trackers,
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
    let Some(cache::Cached {
        status: Some(ListStatus::Completed),
        updated_at: Some(updated_at),
        ..
    }) = cache::cached(conn, t, id)?
    else {
        return Ok(false);
    };
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

/// The series whose rewatch is being written right now, one set per trackers
/// team, so two cores in one process never share a slot (#776). Two marks of
/// one finish (the rule at the end and the player's button, say) both read the
/// list before either has written, so the second finds the series held here
/// and answers with the first's job, leaving the one rewatch to it. Each held
/// series keeps the id of the job writing it, once that job has started.
#[derive(Clone, Default)]
pub(crate) struct Rewatching(Arc<std::sync::Mutex<std::collections::HashMap<u64, JobCell>>>);

/// The id of the job a rewatch slot belongs to, set once the job starts.
type JobCell = Arc<std::sync::OnceLock<u64>>;

impl Rewatching {
    /// The series' slot, or, while another mark holds it, that mark's job
    /// (None until its job has started).
    fn hold(&self, series: u64) -> Result<RewatchSlot, Option<u64>> {
        use std::collections::hash_map::Entry;
        match recover(&self.0).entry(series) {
            Entry::Occupied(held) => Err(held.get().get().copied()),
            Entry::Vacant(free) => {
                let job = free.insert(JobCell::default()).clone();
                Ok(RewatchSlot {
                    set: self.clone(),
                    series,
                    job,
                })
            }
        }
    }
}

/// A series' place in its team's [`Rewatching`], given up when the job that
/// holds it ends however it ends.
struct RewatchSlot {
    set: Rewatching,
    series: u64,
    job: JobCell,
}

impl Drop for RewatchSlot {
    fn drop(&mut self) {
        recover(&self.set.0).remove(&self.series);
    }
}

/// Each list entry's writes in the order they were asked for, one set of
/// lines per trackers team as [`Rewatching`] is. A write reads the entry
/// before it writes, so two writes of one entry at once would both read the
/// same list and the one that landed last would decide it: a status set
/// while a rewatch is being written could leave the rewatch's Completed over
/// it (#1112). Each write takes its place when it is asked for and starts
/// once every write asked for before it on the same entries has ended,
/// however that one ended. Lines go by entry rather than by series, so two
/// series matched to one show, or a show the library does not have, share
/// theirs. Move up, On hold and Plan to watch write from jobs of their own
/// and take their places here too (#1136).
#[derive(Clone, Default)]
pub(crate) struct WriteLine(Arc<LineInner>);

#[derive(Default)]
struct LineInner {
    next: std::sync::atomic::AtomicU64,
    /// Each entry's writes still in line, oldest first.
    waiting: std::sync::Mutex<std::collections::HashMap<ListEntry, Vec<Waiting>>>,
}

/// One list entry: a tracker and its media id.
pub(crate) type ListEntry = (Tracker, u64);

/// A write still in line: its ticket, and the end the writes after it wait on.
type Waiting = (u64, watch::Receiver<()>);

impl WriteLine {
    /// A place at the back of the line of every entry the target writes to.
    fn join(&self, target: &Targets) -> Turn {
        self.join_entries(
            [Tracker::Anilist, Tracker::Mal]
                .into_iter()
                .filter_map(|t| target.id_for(t).map(|id| (t, id)))
                .collect(),
        )
    }

    /// One place at the back of the line of each of these entries.
    pub(crate) fn join_entries(&self, entries: Vec<ListEntry>) -> Turn {
        let ticket = self
            .0
            .next
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (done, end) = watch::channel(());
        let mut waiting = recover(&self.0.waiting);
        let mut before = Vec::new();
        for entry in &entries {
            let line = waiting.entry(*entry).or_default();
            before.extend(line.iter().map(|(_, end)| end.clone()));
            line.push((ticket, end.clone()));
        }
        Turn {
            line: self.clone(),
            entries,
            ticket,
            before,
            _done: done,
        }
    }
}

/// A write's place in its entries' lines. The writes after it may start
/// once this is dropped: when the job holding it ends, or for a job that
/// writes many entries, once it has written the one this place is for. A
/// job holding several places waits for them in the order it took them, so
/// two such jobs never wait on each other.
pub(crate) struct Turn {
    line: WriteLine,
    entries: Vec<ListEntry>,
    ticket: u64,
    /// The ends of every write ahead of this one. Each write waits on all of
    /// them rather than only the last, so one that leaves the line before
    /// its own turn came lets nobody past the ones still ahead of it.
    before: Vec<watch::Receiver<()>>,
    _done: watch::Sender<()>,
}

impl Turn {
    /// Waits for every write ahead of this one to end. Dropping the wait
    /// keeps the place, so the next wait picks up where it stopped.
    pub(crate) async fn come(&mut self) {
        while let Some(end) = self.before.last_mut() {
            // Nothing is ever sent: the sender dropping is how a write ends.
            let _ = end.changed().await;
            self.before.pop();
        }
    }
}

impl Drop for Turn {
    fn drop(&mut self) {
        let mut waiting = recover(&self.line.0.waiting);
        for entry in &self.entries {
            if let Some(line) = waiting.get_mut(entry) {
                line.retain(|(t, _)| *t != self.ticket);
                if line.is_empty() {
                    waiting.remove(entry);
                }
            }
        }
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
    trackers: &Arc<Trackers>,
    series: u64,
    progress: u32,
    season: Option<u32>,
) -> Result<Option<u64>, CoreError> {
    let target = trackers.services.store.read(|c| targets(c, series))?;
    if !target.matched() {
        mark_here(&trackers.services, series, progress, season.unwrap_or(1))?;
        return Ok(None);
    }
    guard(&trackers.services, &target)?;
    start(
        trackers,
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
fn mark_here(
    services: &Services,
    series: u64,
    progress: u32,
    season: u32,
) -> Result<(), CoreError> {
    let now = time::now_secs();
    let past = f64::from(progress) + 1.0;
    services.store.tx(move |tx| {
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
    push_card(services, series, "watched episodes set here");
    Ok(())
}

/// Rates a series out of ten on every connected tracker. `None` is the
/// Clear button, which is a score of nought on both rather than a call of
/// its own.
///
/// A series no connected tracker carries keeps its score on this device
/// instead (#213), at once and with no job: None then, where a tracker
/// write answers with its job.
pub fn set_score(
    trackers: &Arc<Trackers>,
    series: u64,
    score: Option<f64>,
) -> Result<Option<u64>, CoreError> {
    let target = trackers.services.store.read(|c| targets(c, series))?;
    let score = out_of_ten(score.unwrap_or(0.0))?;
    if kept_here(&trackers.services, &target, true)? {
        let now = time::now_secs();
        // Nought is unrated here as it is on a tracker.
        let rated = (score > 0.0).then_some(score);
        trackers
            .services
            .store
            .write(move |c| local::set_score(c, series, rated, now))?;
        push_card(&trackers.services, series, "score kept here");
        return Ok(None);
    }
    start(trackers, JobKind::Score, target, Action::Score { score }).map(Some)
}

/// Puts a series on one list on every connected tracker: the status pill's
/// menu. Repeating is a rewatch the trackers keep their own count for, so
/// it is not a status this menu sets.
///
/// A series no connected tracker carries keeps its status on this device
/// instead (#213), the way `set_score` keeps a score.
pub fn set_list_status(
    trackers: &Arc<Trackers>,
    series: u64,
    status: ListStatus,
) -> Result<Option<u64>, CoreError> {
    settable(status)?;
    let target = trackers.services.store.read(|c| targets(c, series))?;
    if kept_here(&trackers.services, &target, true)? {
        let now = time::now_secs();
        trackers
            .services
            .store
            .write(move |c| local::set_status(c, series, status, now))?;
        push_card(&trackers.services, series, "list status kept here");
        return Ok(None);
    }
    start(
        trackers,
        JobKind::SetStatus,
        target,
        Action::Status { status },
    )
    .map(Some)
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
    let folder_name = anibeam_foundation::titles::resolve(
        lang,
        text(&m["title"]["romaji"]).or(romaji).as_deref(),
        text(&m["title"]["english"]).or(english).as_deref(),
        text(&m["title"]["native"]).or(native).as_deref(),
        &format!("AniList {anilist_id}"),
    );
    Ok(Targets {
        series: 0,
        anilist: Some(anilist_id),
        mal: match m["idMal"].as_u64().or(lib_mal.map(|id| id as u64)) {
            Some(id) => Some(id),
            None => mal_lookup::saved(conn, anilist_id)?.flatten(),
        },
        total: m["episodes"]
            .as_u64()
            .or(lib_eps.map(|e| e as u64))
            .and_then(|e| u32::try_from(e).ok())
            .filter(|total| *total > 0),
        folder_name,
    })
}

/// The series the library has for an AniList id, when it has one.
fn series_of(services: &Services, anilist_id: u64) -> Result<Option<u64>, CoreError> {
    services.store.read(|c| {
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
    trackers: &Arc<Trackers>,
    anilist_id: u64,
    status: ListStatus,
) -> Result<Option<u64>, CoreError> {
    if let Some(series) = series_of(&trackers.services, anilist_id)? {
        return set_list_status(trackers, series, status);
    }
    settable(status)?;
    let target = trackers
        .services
        .store
        .read(|c| show_targets(c, anilist_id))?;
    guard(&trackers.services, &target)?;
    start(
        trackers,
        JobKind::SetStatus,
        target,
        Action::Status { status },
    )
    .map(Some)
}

/// A score out of 10 for a show the library does not have, on every
/// connected tracker; nought clears it.
pub fn set_show_score(
    trackers: &Arc<Trackers>,
    anilist_id: u64,
    score: f64,
) -> Result<Option<u64>, CoreError> {
    if let Some(series) = series_of(&trackers.services, anilist_id)? {
        return set_score(trackers, series, Some(score));
    }
    let score = out_of_ten(score)?;
    let target = trackers
        .services
        .store
        .read(|c| show_targets(c, anilist_id))?;
    guard(&trackers.services, &target)?;
    start(trackers, JobKind::Score, target, Action::Score { score }).map(Some)
}

/// A score out of ten, the only scale a write takes.
fn out_of_ten(score: f64) -> Result<f64, CoreError> {
    if !(0.0..=10.0).contains(&score) {
        return Err(CoreError::invalid(
            "score",
            tr(
                "core-trackers-score-out-of-range",
                &[("score", score.to_string().into())],
            ),
        ));
    }
    Ok(score)
}

/// A status the menus may set. Repeating is a rewatch the trackers keep
/// their own count for, so no menu sets it.
fn settable(status: ListStatus) -> Result<(), CoreError> {
    if status == ListStatus::Repeating {
        return Err(CoreError::invalid(
            "status",
            tr("core-trackers-rewatch-not-settable", &[]),
        ));
    }
    Ok(())
}

/// Whether a status or a score stays on this device: the series has no
/// tracker match, or no account is connected behind the ids it has. A
/// hidden series is still refused, since hiding is about its tracker sync
/// and a person who hid it asked for nothing to go out. `lookup` is
/// `guard_reach`'s: a write to one series may look a missing MAL id up.
fn kept_here(services: &Services, target: &Targets, lookup: bool) -> Result<bool, CoreError> {
    // `guard` answers NoMatch before it looks at the hidden flag, which a
    // series with no ids has nothing else to say about.
    let hidden = services.store.read(|c| {
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
    match guard_reach(services, target, lookup) {
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
fn push_card(services: &Services, series: u64, line: &str) {
    let images_dir = services.paths.images_dir();
    match services
        .store
        .read(|c| anibeam_library::cards::cards_for(c, &images_dir, &[series]))
    {
        Ok(cards) => {
            services.bus.debug(
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
pub fn set_notes(trackers: &Arc<Trackers>, series: u64, notes: &str) -> Result<u64, CoreError> {
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
    let target = trackers.services.store.read(|c| targets(c, series))?;
    guard(&trackers.services, &target)?;
    let notes = cache::note_of(Some(notes));
    start(trackers, JobKind::Notes, target, Action::Notes { notes })
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
pub fn start_many(
    trackers: &Arc<Trackers>,
    series: &[u64],
    write: ManyWrite,
) -> Result<Option<u64>, CoreError> {
    let action = match write {
        ManyWrite::Score(score) => Action::Score {
            score: out_of_ten(score.unwrap_or(0.0))?,
        },
        ManyWrite::Status(status) => Action::Status {
            status: {
                settable(status)?;
                status
            },
        },
    };
    let mut reachable = Vec::new();
    let mut refused = None;
    let mut left_out = LeftOut::default();
    let now = time::now_secs();
    for &id in series {
        let target = trackers.services.store.read(|c| targets(c, id))?;
        match kept_here(&trackers.services, &target, false) {
            Ok(false) => reachable.push(target),
            Ok(true) => {
                trackers.services.store.write(move |c| match write {
                    ManyWrite::Score(score) => {
                        local::set_score(c, id, score.filter(|s| *s > 0.0), now)
                    }
                    ManyWrite::Status(status) => local::set_status(c, id, status, now),
                })?;
                push_card(&trackers.services, id, "kept here");
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
    let owner = trackers.clone();
    let jobs = owner.services.jobs.clone();
    let turns = reachable
        .iter()
        .map(|target| owner.write_line.join(target))
        .collect();
    Ok(Some(jobs.start(kind, move |ctx| async move {
        run_many(owner, ctx, reachable, turns, action, left_out).await
    })))
}

async fn run_many(
    trackers: Arc<Trackers>,
    ctx: Arc<JobCtx>,
    targets: Vec<Targets>,
    turns: Vec<Turn>,
    action: Action,
    left_out: LeftOut,
) -> Result<Finished, CoreError> {
    let total = targets.len() as u64;
    let (mut done, mut failed) = (0u64, 0u64);
    let mut named = None;
    let mut landed: Vec<Tracker> = Vec::new();
    let mut reason: Option<String> = None;
    // Every pick's place was taken when the write was asked for, so a
    // write to one of them asked for later waits until this loop has
    // written it. Each place is given up as soon as its pick is written.
    for ((i, target), mut turn) in targets.iter().enumerate().zip(turns) {
        turn.come().await;
        ctx.checkpoint()?;
        ctx.progress(i as u64, Some(total), &target.folder_name);
        let outcomes = write_all(&trackers, &ctx, target, &action, false).await?;
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
        if let Some(card) = card_for(&trackers.services, target.series).await? {
            ctx.changed(card);
        }
    }
    let sites = sites_of(landed.into_iter());
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
    let said = anibeam_contract::words!(
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
    let (anilist, mal) = landed.fold((false, false), |(a, m), t| {
        (a || t == Tracker::Anilist, m || t == Tracker::Mal)
    });
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
/// `NotConnected` refusal names and the one a rewatch is judged by.
fn guard(services: &Services, target: &Targets) -> Result<Tracker, CoreError> {
    guard_reach(services, target, true).map(|(main, _)| main)
}

/// `guard`, with the trackers the write reaches (`reach`). `lookup` says
/// whether this write may look a missing MAL id up, which only a write to
/// one series does.
fn guard_reach(
    services: &Services,
    target: &Targets,
    lookup: bool,
) -> Result<(Tracker, Vec<Tracker>), CoreError> {
    services.store.read(|c| {
        if !target.matched() {
            return Err(CoreError::Refused {
                reason: Refusal::NoMatch,
            });
        }
        if any_hidden(c, target.anilist, target.mal)? {
            return Err(CoreError::Refused {
                reason: Refusal::Hidden,
            });
        }
        let main = prefs::load_main_tracker(c)?;
        let reach = reach(c, target, lookup)?;
        if reach.is_empty() {
            return Err(CoreError::NotConnected { tracker: main });
        }
        Ok((main, reach))
    })
}

/// The trackers a write to this target goes to: every connected one the
/// match carries an id for, and a connected MAL whose id the write may
/// still look up (`mal_lookup`).
fn reach(conn: &Connection, target: &Targets, lookup: bool) -> Result<Vec<Tracker>, CoreError> {
    let mut reach = Vec::new();
    for t in [Tracker::Anilist, Tracker::Mal] {
        if accounts::connected(conn, t)?.is_none() {
            continue;
        }
        let found = match (t, target.id_for(t), target.anilist) {
            (_, Some(_), _) => true,
            (Tracker::Mal, None, Some(anilist)) if lookup => {
                mal_lookup::can_look_up(conn, anilist)?
            }
            _ => false,
        };
        if found {
            reach.push(t);
        }
    }
    Ok(reach)
}

/// Whether every tracker in `reach` already has this episode in the
/// cache. A tracker with no id yet or no cached entry has not, since
/// there is nothing to say it is that far.
pub(crate) fn covered(
    conn: &Connection,
    target: &Targets,
    reach: &[Tracker],
    episode: u32,
) -> Result<bool, CoreError> {
    for &t in reach {
        let Some(id) = target.id_for(t) else {
            return Ok(false);
        };
        if !cache::cached(conn, t, id)?.is_some_and(|e| e.progress >= episode) {
            return Ok(false);
        }
    }
    Ok(!reach.is_empty())
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

fn start(
    trackers: &Arc<Trackers>,
    kind: JobKind,
    target: Targets,
    action: Action,
) -> Result<u64, CoreError> {
    start_holding(trackers, kind, target, action, None)
}

/// `start`, with the rewatch slot the job keeps until it ends.
fn start_holding(
    trackers: &Arc<Trackers>,
    kind: JobKind,
    target: Targets,
    action: Action,
    slot: Option<RewatchSlot>,
) -> Result<u64, CoreError> {
    let owner = trackers.clone();
    let jobs = owner.services.jobs.clone();
    let cell = slot.as_ref().map(|s| s.job.clone());
    let mut turn = owner.write_line.join(&target);
    let job = jobs.start(kind, move |ctx| async move {
        let _slot = slot;
        turn.come().await;
        run(owner, ctx, target, action).await
    });
    if let Some(cell) = cell {
        let _ = cell.set(job);
    }
    Ok(job)
}

async fn run(
    trackers: Arc<Trackers>,
    ctx: Arc<JobCtx>,
    target: Targets,
    action: Action,
) -> Result<Finished, CoreError> {
    let outcomes = write_all(&trackers, &ctx, &target, &action, true).await?;
    // The cache was patched by each write above, so the card built here
    // already carries the new number. The runner flushes this before the
    // terminal event.
    if let Some(card) = card_for(&trackers.services, target.series).await? {
        ctx.changed(card);
    }
    Ok(finished(&target, action, outcomes))
}

/// One series' write to every connected tracker its match carries an id
/// for. `loud` says each tracker's failure in the plain log; a write over
/// several series keeps them to the developer log and counts them instead.
async fn write_all(
    trackers: &Trackers,
    ctx: &Arc<JobCtx>,
    target: &Targets,
    action: &Action,
    loud: bool,
) -> Result<Vec<TrackerOutcome>, CoreError> {
    let mut outcomes: Vec<TrackerOutcome> = Vec::new();
    let mut missed = false;
    for t in [Tracker::Anilist, Tracker::Mal] {
        ctx.checkpoint()?;
        // A tracker with no account behind it is not this write's business.
        let Some(row) = accounts::connected_async(&trackers.services, t).await? else {
            continue;
        };
        // A tracker the match carries no id for has nothing to write to,
        // unless it is MAL and its search finds the show (#782). A write
        // over several series never searches, so a pick is never a sweep.
        let media_id = match (target.id_for(t), t, target.anilist) {
            (Some(id), _, _) => id,
            (None, Tracker::Mal, Some(anilist)) if loud => {
                match look_up(trackers, ctx, target, anilist).await {
                    Ok(Some(id)) => id,
                    Ok(None) => {
                        missed = true;
                        continue;
                    }
                    Err(e) => {
                        outcomes.push(failed(ctx, t, target, action, loud, &e));
                        continue;
                    }
                }
            }
            _ => continue,
        };
        let outcome = attempt(trackers, ctx, t, target, media_id, row.user_id, action).await;
        outcomes.push(outcome.unwrap_or_else(|e| failed(ctx, t, target, action, loud, &e)));
    }
    // A write that went nowhere because MAL's search found nothing says so,
    // or the job would read as done. Beside another tracker's outcome the
    // miss stays quiet, since that write did land.
    if outcomes.is_empty() && missed {
        outcomes.push(TrackerOutcome {
            tracker: Tracker::Mal,
            ok: false,
            progress: None,
            reason: Some(Refusal::NoMatch),
            message: None,
        });
    }
    Ok(outcomes)
}

/// The MAL id MAL's own search finds for a show AniList gave none for,
/// said in the developer log when a search was made.
async fn look_up(
    trackers: &Trackers,
    ctx: &Arc<JobCtx>,
    target: &Targets,
    anilist: u64,
) -> Result<Option<u64>, CoreError> {
    let saved = trackers
        .services
        .store
        .read(|c| mal_lookup::saved(c, anilist))?;
    if let Some(answer) = saved {
        return Ok(answer);
    }
    let token = accounts::token(trackers, Tracker::Mal).await?;
    let found = mal_lookup::resolve(trackers, anilist, &token).await?;
    ctx.emit(
        Level::Debug,
        match found {
            Some(id) => format!("mal search found {id} for {}", target.folder_name),
            None => format!("mal search found nothing sure for {}", target.folder_name),
        },
        EventBody::Notice,
    );
    Ok(found)
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
    let show = &target.folder_name;
    let level = level_of(&outcomes);
    let sites = landed_on(&outcomes);
    use anibeam_contract::words;
    let (plain, message, body) = match action {
        Action::Mark { episode, .. } => (
            sites.map(|s| {
                words!(
                    "core-trackers-marked",
                    episode = episode,
                    show = show,
                    sites = s
                )
            }),
            format!("Marked episode {episode} of {show}"),
            EventBody::Marked {
                series,
                episode,
                outcomes,
            },
        ),
        Action::Progress { progress } => (
            sites.map(|s| {
                words!(
                    "core-trackers-progress-set",
                    progress = progress,
                    show = show,
                    sites = s
                )
            }),
            format!("Progress of {show} set to {progress}"),
            EventBody::ProgressSet {
                series,
                progress,
                outcomes,
            },
        ),
        // A score of nought is unrated on both trackers, so the event
        // carries no score rather than a rating of nothing.
        Action::Score { score } => {
            let rated = (score > 0.0).then_some(score);
            (
                sites.map(|s| match rated {
                    Some(score) => words!(
                        "core-trackers-scored",
                        score = score.to_string(),
                        show = show,
                        sites = s
                    ),
                    None => words!("core-trackers-score-cleared", show = show, sites = s),
                }),
                match rated {
                    Some(score) => format!("score of {show} set to {score}"),
                    None => format!("score of {show} cleared"),
                },
                EventBody::Scored {
                    series,
                    score: rated,
                    outcomes,
                },
            )
        }
        Action::Status { status } => (
            sites.map(|s| {
                words!(
                    "core-trackers-status-set",
                    status = status.as_str(),
                    show = show,
                    sites = s
                )
            }),
            format!("status of {show} set to {}", status.as_str()),
            EventBody::ListStatusSet {
                series,
                status,
                outcomes,
            },
        ),
        Action::Notes { notes } => (
            sites.map(|s| match notes {
                Some(_) => words!("core-trackers-notes-saved", show = show, sites = s),
                None => words!("core-trackers-notes-cleared", show = show, sites = s),
            }),
            match notes {
                Some(_) => format!("notes of {show} saved"),
                None => format!("notes of {show} cleared"),
            },
            EventBody::NotesSet {
                series,
                notes,
                outcomes,
            },
        ),
    };
    Finished {
        plain,
        level,
        message,
        body,
    }
}

/// A tracker's failed write. A provider failure is this tracker's alone:
/// it becomes an `ok: false` outcome with a line the user can act on, the
/// whole message goes to the log, loud or kept to the developer log, and
/// the job carries on to the other tracker.
fn failed(
    ctx: &Arc<JobCtx>,
    t: Tracker,
    target: &Targets,
    action: &Action,
    loud: bool,
    e: &CoreError,
) -> TrackerOutcome {
    if loud {
        ctx.say(
            Level::Error,
            anibeam_contract::words!(
                "core-trackers-update-failed",
                site = super::oauth::site_name(t)
            ),
            format!("{} {} failed: {}", t.as_str(), action.verb(), message_of(e)),
            EventBody::Notice,
        );
    } else {
        ctx.emit(
            Level::Debug,
            format!(
                "{} {} failed for {}: {}",
                t.as_str(),
                action.verb(),
                target.folder_name,
                message_of(e)
            ),
            EventBody::Notice,
        );
    }
    TrackerOutcome {
        tracker: t,
        ok: false,
        progress: None,
        reason: None,
        message: Some(sanitize_error(t, e)),
    }
}

/// The write itself: the tracker's current entry, the action's own guard,
/// the mutation, the account's `synced_at`, the cache patch and the line
/// the activity log gets.
pub(crate) async fn attempt(
    trackers: &Trackers,
    ctx: &Arc<JobCtx>,
    t: Tracker,
    target: &Targets,
    media_id: u64,
    user_id: Option<u64>,
    action: &Action,
) -> Result<TrackerOutcome, CoreError> {
    let remote = Remote::connect(trackers, t, media_id).await?;
    let entry = remote.read_entry(user_id).await?;
    let current = entry.progress;
    let today = today();
    let outcome = |ok, progress, reason, message| TrackerOutcome {
        tracker: t,
        ok,
        progress,
        reason,
        message,
    };
    // What landed, as the debug line names it: the line's head, and what
    // rode along after the media id.
    let (progress, head, tail) = match action {
        &Action::Mark { episode, again } => {
            // The monotonic rule, decided against the tracker itself: a
            // mark only ever moves progress up, and a list already past
            // this episode is not wrong, it is ahead. A completed show
            // finished again is the one exception, and it counts a rewatch.
            let rewatch = again && entry.status == Some(ListStatus::Completed);
            if current >= episode && !rewatch {
                return Ok(outcome(false, Some(current), Some(Refusal::NotNewer), None));
            }
            let progress = current.max(episode);
            let status = watched_status(&entry, target.completes(progress));
            let extras = plan(&entry, status, true, rewatch, today);
            remote.watched(progress, status, &extras).await?;
            // The terminal event carries every tracker's outcome, so a
            // line per tracker here would say the same thing twice.
            (
                Some(progress),
                format!("{current} -> {progress}"),
                extras.tail(),
            )
        }
        &Action::Progress { progress } => {
            let status = if progress == 0 {
                ListStatus::Planning
            } else {
                watched_status(&entry, target.completes(progress))
            };
            let extras = plan(&entry, status, progress > 0, false, today);
            remote.watched(progress, status, &extras).await?;
            (
                Some(progress),
                format!("set {current} -> {progress}"),
                extras.tail(),
            )
        }
        &Action::Score { score } => {
            // Rating a series whose every episode is already watched is
            // also saying it is finished, which is what the list should
            // have said all along.
            let completed = target.completes(current);
            let extras = match completed {
                true => plan(&entry, ListStatus::Completed, false, false, today),
                false => Extras::default(),
            };
            let (sent, stamp) = remote.send_score(score, completed, &extras).await?;
            // A score of nought is unrated rather than a rating of nothing,
            // and a status is only written when the rating completed it.
            let rated = (sent > 0.0).then_some(sent);
            let status = completed.then_some(ListStatus::Completed);
            remote
                .record(move |tx, now| {
                    cache::patch_score(tx, t, media_id, rated, status, stamp, now)
                })
                .await?;
            remote.record_repeat(&extras).await?;
            let completed = if completed { " + completed" } else { "" };
            (
                None,
                format!("score -> {sent}"),
                format!("{completed}{}", extras.tail()),
            )
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
            let stamp = remote.send_list_status(status, fill, &extras).await?;
            // A series not on the list yet has no cached entry to patch, so
            // the entry is upserted with the progress the tracker now has.
            remote.record_progress(progress, status, stamp).await?;
            (
                Some(progress),
                format!("status -> {}", status.as_str()),
                extras.tail(),
            )
        }
        Action::Notes { notes } => {
            // Notes on a show the list does not hold would add it to the
            // list, on a list the person never picked.
            if entry.status.is_none() {
                return Ok(outcome(
                    false,
                    None,
                    None,
                    Some(tr(
                        "core-trackers-notes-not-on-list",
                        &[("tracker", t.label().into())],
                    )),
                ));
            }
            let stamp = remote.send_notes(notes.as_deref()).await?;
            let notes = notes.clone();
            remote
                .record(move |tx, now| {
                    cache::patch_notes(tx, t, media_id, notes.as_deref(), stamp, now)
                })
                .await?;
            (Some(current), "notes saved".to_string(), String::new())
        }
    };
    ctx.emit(
        Level::Debug,
        format!("{} {head} (mediaId {media_id}){tail}", t.as_str()),
        EventBody::Notice,
    );
    Ok(outcome(true, progress, None, None))
}

// Dates and rewatches --------------------------------------------------------

/// A list entry as a write reads it first: how far it has got, its status,
/// whether it carries a start and a finish date, and how many rewatches.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Current {
    progress: u32,
    status: Option<ListStatus>,
    started: bool,
    finished: bool,
    repeat: u32,
}

/// A calendar day, year, month and day.
type Day = (i32, u8, u8);

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
struct Extras {
    start: Option<Day>,
    finish: Option<Day>,
    repeat: Option<u32>,
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

    /// The extras as AniList's variables: a `FuzzyDateInput` per date and
    /// the count. A key left out leaves the field alone.
    fn anilist(&self, variables: &mut serde_json::Value) {
        let date = |(year, month, day): Day| serde_json::json!({ "year": year, "month": month, "day": day });
        if let Some(day) = self.start {
            variables["startedAt"] = date(day);
        }
        if let Some(day) = self.finish {
            variables["completedAt"] = date(day);
        }
        if let Some(repeat) = self.repeat {
            variables["repeat"] = serde_json::json!(repeat);
        }
    }

    /// The same as MAL's form fields. A rewatch finished is a rewatch no
    /// longer under way, so the flag goes with the count.
    fn mal(&self, form: &mut Vec<(String, String)>) {
        let date = |(year, month, day): Day| format!("{year:04}-{month:02}-{day:02}");
        if let Some(day) = self.start {
            form.push(field("start_date", date(day)));
        }
        if let Some(day) = self.finish {
            form.push(field("finish_date", date(day)));
        }
        if let Some(repeat) = self.repeat {
            form.push(field("num_times_rewatched", repeat));
            form.push(field("is_rewatching", false));
        }
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
fn plan(entry: &Current, sending: ListStatus, watched: bool, again: bool, today: Day) -> Extras {
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

/// Whether an AniList fuzzy date holds anything: a day and month with no
/// year is still a date the person wrote, and is never written over.
fn fuzzy_dated(date: &serde_json::Value) -> bool {
    ["year", "month", "day"]
        .iter()
        .any(|k| date[*k].as_u64().is_some())
}

// The provider calls ---------------------------------------------------------

/// One entry on one tracker's list, and the token that reaches it: every
/// read and write of a list entry goes through here, so each provider is
/// asked one way.
pub(crate) struct Remote<'a> {
    pub trackers: &'a Trackers,
    pub t: Tracker,
    pub media_id: u64,
    pub token: String,
}

/// One MAL form field.
fn field(name: &str, value: impl ToString) -> (String, String) {
    (name.to_string(), value.to_string())
}

impl<'a> Remote<'a> {
    /// The entry on a tracker whose token is read now: no token is the
    /// tracker's failure, in the words a person reads.
    pub(crate) async fn connect(
        trackers: &'a Trackers,
        t: Tracker,
        media_id: u64,
    ) -> Result<Remote<'a>, CoreError> {
        Ok(Remote {
            trackers,
            t,
            media_id,
            token: accounts::token(trackers, t).await?,
        })
    }

    /// The entry off an AniList query that takes the user and the media,
    /// `Null` for one the list does not hold. `MediaList(mediaId)` without
    /// the user id ignores the bearer token and answers with some other
    /// user's entry, so a missing user id is a broken account rather than a
    /// reason to guess.
    async fn anilist_entry(
        &self,
        query: &str,
        user_id: Option<u64>,
    ) -> Result<serde_json::Value, CoreError> {
        let user_id = user_id
            .ok_or_else(|| accounts::tracker_error(self.t, tr("core-trackers-no-user-id", &[])))?;
        let variables = serde_json::json!({ "userId": user_id, "mediaId": self.media_id });
        match self
            .trackers
            .net
            .anilist
            .graphql_within(query, variables, Some(&self.token), TRACKER_TIMEOUT)
            .await
        {
            // `get_mut` rather than indexing: a reply whose `data` is not an
            // object is no entry, never a panic.
            Ok(mut data) => Ok(data
                .get_mut("MediaList")
                .map(serde_json::Value::take)
                .unwrap_or_default()),
            Err(e) if e.is_provider_not_found() => Ok(serde_json::Value::Null),
            Err(e) => Err(e),
        }
    }

    /// The anime off MAL with these fields, None for one MAL does not have.
    async fn mal_anime(&self, fields: &str) -> Result<Option<serde_json::Value>, CoreError> {
        let url = format!("{MAL_API}/anime/{}?fields={fields}", self.media_id);
        let response = self.trackers.net.mal.get(&url, &self.token).await?;
        if response.status == 404 {
            return Ok(None);
        }
        crate::mal_json(&response).map(Some)
    }

    /// The entry as the tracker itself has it. An anime the user has never
    /// added is an empty entry rather than a failure: that is exactly what a
    /// first mark is for, on either tracker.
    async fn read_entry(&self, user_id: Option<u64>) -> Result<Current, CoreError> {
        match self.t {
            Tracker::Anilist => {
                let entry = self.anilist_entry(MEDIA_LIST_ENTRY_QUERY, user_id).await?;
                Ok(Current {
                    progress: as_count(entry["progress"].as_u64()),
                    status: cache::normalize_status(self.t, entry["status"].as_str(), false),
                    started: fuzzy_dated(&entry["startedAt"]),
                    finished: fuzzy_dated(&entry["completedAt"]),
                    repeat: as_count(entry["repeat"].as_u64()),
                })
            }
            Tracker::Mal => {
                let body = self
                    .mal_anime("my_list_status{status,num_episodes_watched,is_rewatching,num_times_rewatched,start_date,finish_date}")
                    .await?
                    .unwrap_or_default();
                let list = &body["my_list_status"];
                let dated = |key: &str| list[key].as_str().is_some_and(|d| !d.is_empty());
                Ok(Current {
                    progress: as_count(list["num_episodes_watched"].as_u64()),
                    status: cache::mal_list_status(list),
                    started: dated("start_date"),
                    finished: dated("finish_date"),
                    repeat: as_count(list["num_times_rewatched"].as_u64()),
                })
            }
        }
    }

    /// The entry as the tracker has it now, with the media's airing state,
    /// for a last look before a status write. `None` is an entry the list no
    /// longer holds. MAL has no broadcast schedule, so its `next` is empty
    /// and the library's schedule rows date its next episode.
    pub(crate) async fn read_live(
        &self,
        user_id: Option<u64>,
    ) -> Result<Option<on_hold::Live>, CoreError> {
        match self.t {
            Tracker::Anilist => {
                let entry = self.anilist_entry(ON_HOLD_ENTRY_QUERY, user_id).await?;
                if entry.is_null() {
                    return Ok(None);
                }
                let media = &entry["media"];
                let next = &media["nextAiringEpisode"];
                Ok(Some(on_hold::Live {
                    status: cache::normalize_status(self.t, entry["status"].as_str(), false),
                    progress: as_count(entry["progress"].as_u64()),
                    updated_at: entry["updatedAt"].as_i64().filter(|at| *at > 0),
                    total: as_u32(media["episodes"].as_u64()).filter(|n| *n > 0),
                    still_coming: media["status"].as_str().is_some_and(on_hold::still_coming),
                    next: as_u32(next["episode"].as_u64()).zip(next["airingAt"].as_i64()),
                }))
            }
            Tracker::Mal => {
                let Some(body) = self.mal_anime("my_list_status,status,num_episodes").await? else {
                    return Ok(None);
                };
                let list = &body["my_list_status"];
                if list.is_null() {
                    return Ok(None);
                }
                Ok(Some(on_hold::Live {
                    status: cache::mal_list_status(list),
                    progress: as_count(list["num_episodes_watched"].as_u64()),
                    updated_at: list["updated_at"].as_str().and_then(cache::mal_instant),
                    total: as_u32(body["num_episodes"].as_u64()).filter(|n| *n > 0),
                    still_coming: body["status"].as_str().is_some_and(on_hold::still_coming),
                    next: None,
                }))
            }
        }
    }

    /// One `SaveMediaListEntry`, answering with the tracker's own stamp for
    /// the write when its reply carried one.
    async fn save(
        &self,
        mutation: &str,
        mut variables: serde_json::Value,
    ) -> Result<Option<i64>, CoreError> {
        variables["mediaId"] = serde_json::json!(self.media_id);
        let data = self
            .trackers
            .net
            .anilist
            .graphql_within(mutation, variables, Some(&self.token), TRACKER_TIMEOUT)
            .await?;
        Ok(data["SaveMediaListEntry"]["updatedAt"]
            .as_i64()
            .filter(|at| *at > 0))
    }

    /// One PATCH of the entry's list status. MAL answers with the whole list
    /// status, its `updated_at` among it.
    async fn patch(&self, form: Vec<(String, String)>) -> Result<Option<i64>, CoreError> {
        let url = format!("{MAL_API}/anime/{}/my_list_status", self.media_id);
        let response = self
            .trackers
            .net
            .mal
            .patch_form(&url, &self.token, form)
            .await?;
        if !response.is_success() {
            return Err(crate::mal_failure(&response));
        }
        Ok(response
            .json::<serde_json::Value>()
            .ok()
            .and_then(|body| body["updated_at"].as_str().and_then(cache::mal_instant)))
    }

    /// The progress and the status, with whatever dates and rewatch count
    /// ride along.
    async fn send_progress(
        &self,
        progress: u32,
        status: ListStatus,
        extras: &Extras,
    ) -> Result<Option<i64>, CoreError> {
        match self.t {
            Tracker::Anilist => {
                let mut variables =
                    serde_json::json!({ "progress": progress, "status": anilist_status(status) });
                extras.anilist(&mut variables);
                self.save(SAVE_PROGRESS_MUTATION, variables).await
            }
            Tracker::Mal => {
                let mut form = vec![
                    field("num_watched_episodes", progress),
                    field("status", mal_status(status)),
                ];
                extras.mal(&mut form);
                self.patch(form).await
            }
        }
    }

    /// A watched count sent, then kept: the progress, the status and the
    /// rewatch count it carried.
    async fn watched(
        &self,
        progress: u32,
        status: ListStatus,
        extras: &Extras,
    ) -> Result<(), CoreError> {
        let stamp = self.send_progress(progress, status, extras).await?;
        self.record_progress(progress, status, stamp).await?;
        self.record_repeat(extras).await
    }

    /// The score each tracker actually receives, which is not the same
    /// number: AniList takes `scoreRaw` out of a hundred, so one write works
    /// whatever display format the user picked and a decimal survives it;
    /// MAL takes a whole number and rounds the way its own UI rounds a typed
    /// decimal. What was sent comes back, so the cache and the log line
    /// agree with the list, and beside it the tracker's stamp for the write.
    async fn send_score(
        &self,
        score: f64,
        completed: bool,
        extras: &Extras,
    ) -> Result<(f64, Option<i64>), CoreError> {
        match self.t {
            Tracker::Anilist => {
                let mut variables =
                    serde_json::json!({ "scoreRaw": (score * 10.0).round() as i64 });
                // A mutation sent without the status variable leaves the
                // entry's own status alone, which is what a rating that
                // completes nothing should do.
                if completed {
                    variables["status"] = anilist_status(ListStatus::Completed).into();
                }
                extras.anilist(&mut variables);
                Ok((score, self.save(SAVE_SCORE_MUTATION, variables).await?))
            }
            Tracker::Mal => {
                let rounded = score.round();
                let mut form = vec![field("score", rounded as i64)];
                if completed {
                    form.push(field("status", mal_status(ListStatus::Completed)));
                }
                extras.mal(&mut form);
                Ok((rounded, self.patch(form).await?))
            }
        }
    }

    /// A status alone, which is what On hold and Plan to watch send: the
    /// progress and the score stay as the list has them on both trackers,
    /// since neither write names them.
    pub(crate) async fn send_status(&self, status: ListStatus) -> Result<Option<i64>, CoreError> {
        match self.t {
            Tracker::Anilist => {
                let variables = serde_json::json!({ "status": anilist_status(status) });
                self.save(SAVE_STATUS_MUTATION, variables).await
            }
            Tracker::Mal => self.patch(vec![field("status", mal_status(status))]).await,
        }
    }

    /// A status the user picked from the menu. MAL keeps a rewatch as a flag
    /// beside the status, so the flag is cleared here, or a rewatch moved to
    /// Watching would read back as Rewatching on the next refresh. `fill` is
    /// the progress Completed raises the list to, when it is short of the
    /// total.
    async fn send_list_status(
        &self,
        status: ListStatus,
        fill: Option<u32>,
        extras: &Extras,
    ) -> Result<Option<i64>, CoreError> {
        match (self.t, fill) {
            (Tracker::Anilist, Some(progress)) => {
                self.send_progress(progress, status, extras).await
            }
            (Tracker::Anilist, None) => {
                let mut variables = serde_json::json!({ "status": anilist_status(status) });
                extras.anilist(&mut variables);
                self.save(SAVE_LIST_STATUS_MUTATION, variables).await
            }
            (Tracker::Mal, fill) => {
                let mut form = vec![
                    field("status", mal_status(status)),
                    field("is_rewatching", false),
                ];
                if let Some(progress) = fill {
                    form.push(field("num_watched_episodes", progress));
                }
                extras.mal(&mut form);
                self.patch(form).await
            }
        }
    }

    /// The entry's notes alone: AniList's notes, MAL's comments. None clears
    /// them, which both trackers read from an empty string.
    async fn send_notes(&self, notes: Option<&str>) -> Result<Option<i64>, CoreError> {
        let text = notes.unwrap_or("");
        match self.t {
            Tracker::Anilist => {
                self.save(SAVE_NOTES_MUTATION, serde_json::json!({ "notes": text }))
                    .await
            }
            Tracker::Mal => self.patch(vec![field("comments", text)]).await,
        }
    }

    // What is left behind ----------------------------------------------------

    /// The account's `synced_at` and the entry's cache patch, in one
    /// transaction: two tables move together or neither does.
    async fn record<F>(&self, patch: F) -> Result<(), CoreError>
    where
        F: FnOnce(&rusqlite::Transaction, i64) -> Result<(), CoreError> + Send + 'static,
    {
        let (t, now) = (self.t, time::now_secs());
        self.trackers
            .services
            .store
            .tx_async(move |tx| {
                tx.execute(
                    "UPDATE tracker_accounts SET synced_at = ?2 WHERE tracker = ?1 AND profile_id = 1",
                    params![t.as_str(), now],
                )?;
                patch(tx, now)
            })
            .await
    }

    /// The progress and the status a write left, with the tracker's stamp
    /// for it when the reply carried one.
    pub(crate) async fn record_progress(
        &self,
        progress: u32,
        status: ListStatus,
        stamp: Option<i64>,
    ) -> Result<(), CoreError> {
        let (t, media_id) = (self.t, self.media_id);
        self.record(move |tx, now| {
            cache::patch_progress(tx, t, media_id, progress, Some(status), stamp, now)
        })
        .await
    }

    /// A status alone, as On hold sent it.
    pub(crate) async fn record_status(
        &self,
        status: ListStatus,
        stamp: Option<i64>,
    ) -> Result<(), CoreError> {
        let (t, media_id) = (self.t, self.media_id);
        self.record(move |tx, now| cache::patch_status(tx, t, media_id, status, stamp, now))
            .await
    }

    /// The rewatch count a write sent, when it sent one.
    async fn record_repeat(&self, extras: &Extras) -> Result<(), CoreError> {
        let Some(repeat) = extras.repeat else {
            return Ok(());
        };
        let (t, media_id, now) = (self.t, self.media_id, time::now_secs());
        self.trackers
            .services
            .store
            .write_async(move |c| cache::patch_repeat(c, t, media_id, repeat, now))
            .await
    }
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

    #[test]
    fn a_rewatch_slot_is_held_once_and_freed_when_dropped() {
        let set = Rewatching::default();
        let slot = set.hold(7).expect("a free series gives its slot");
        assert_eq!(
            set.hold(7).err(),
            Some(None),
            "a held series refuses a second hold"
        );
        slot.job.set(31).unwrap();
        assert_eq!(
            set.hold(7).err(),
            Some(Some(31)),
            "and names its job once started"
        );
        assert!(set.hold(8).is_ok(), "another series is free");
        assert!(
            Rewatching::default().hold(7).is_ok(),
            "another team's set never shares the slot"
        );
        drop(slot);
        assert!(set.hold(7).is_ok(), "a dropped slot frees the series");
    }

    #[tokio::test]
    async fn a_write_waits_for_every_write_ahead_of_it_on_its_entries_only() {
        let line = WriteLine::default();
        let target = |series, anilist| Targets {
            series,
            anilist: Some(anilist),
            mal: None,
            total: None,
            folder_name: String::new(),
        };
        async fn waits(turn: &mut Turn) -> bool {
            let soon = std::time::Duration::from_millis(50);
            tokio::time::timeout(soon, turn.come()).await.is_err()
        }
        let first = line.join(&target(1, 100));
        let cancelled = line.join(&target(1, 100));
        // Another series matched to the same show shares its line.
        let mut third = line.join(&target(2, 100));
        let mut elsewhere = line.join(&target(3, 300));
        assert!(!waits(&mut elsewhere).await, "another show does not wait");
        assert!(waits(&mut third).await, "a later write waits");
        drop(cancelled);
        assert!(
            waits(&mut third).await,
            "a write leaving the line lets nobody past the one still ahead"
        );
        drop(first);
        assert!(!waits(&mut third).await);
        drop((third, elsewhere));
        assert!(
            recover(&line.0.waiting).is_empty(),
            "an empty line is forgotten"
        );
    }

    /// The line a provider failure with this status and message reads as.
    fn said(t: Tracker, status: Option<u32>, message: &str) -> String {
        let provider = match t {
            Tracker::Anilist => Provider::Anilist,
            Tracker::Mal => Provider::Mal,
        };
        sanitize_error(t, &CoreError::provider(provider, status, message))
    }

    /// The status is read first, whatever the provider said, because a 429
    /// or a 401 is something the user can act on and "Too Many Requests."
    /// is not.
    #[test]
    fn a_status_decides_the_line_before_anything_the_provider_said() {
        let anilist = |status| said(Tracker::Anilist, Some(status), "Too Many Requests.");
        assert_eq!(anilist(429), "AniList rate limited, try again in a minute.");
        assert_eq!(anilist(401), "AniList auth expired, reconnect in Settings.");
        assert_eq!(anilist(403), "AniList auth expired, reconnect in Settings.");
        assert_eq!(anilist(404), "AniList entry not found.");
        assert_eq!(anilist(500), "AniList server error (500), try again later.");
        assert_eq!(anilist(503), "AniList server error (503), try again later.");
        // The label follows the tracker, so a MAL failure never says
        // AniList at the user.
        assert_eq!(
            said(Tracker::Mal, Some(429), "raw"),
            "MAL rate limited, try again in a minute."
        );
    }

    /// Below the statuses that mean something, the provider's own words are
    /// the best line there is, and nothing said is not a message.
    #[test]
    fn a_status_with_no_rule_falls_through_to_the_message() {
        assert_eq!(said(Tracker::Mal, Some(400), "Bad Request"), "Bad Request");
        assert_eq!(said(Tracker::Anilist, None, "gql"), "gql");
        assert_eq!(said(Tracker::Mal, None, ""), "MAL error, see activity log.");
    }

    /// A blob of GraphQL, an HTML error page or a stack trace is worse in
    /// a toast than no message at all, so anything that long falls through
    /// to the log.
    #[test]
    fn a_long_message_falls_through_to_the_log() {
        let long = "x".repeat(600);
        assert_eq!(
            said(Tracker::Anilist, None, &long),
            "AniList error, see activity log."
        );
        assert_eq!(
            said(Tracker::Mal, Some(302), &long),
            "MAL error, see activity log."
        );
        // Two hundred is the line: one under it reads, one on it does not.
        let edge = "y".repeat(199);
        assert_eq!(said(Tracker::Anilist, None, &edge), edge);
        assert_eq!(
            said(Tracker::Anilist, None, &"y".repeat(200)),
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
