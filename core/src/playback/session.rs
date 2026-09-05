//! One run of the player over one file. `open` tells the shell everything it
//! needs before the first frame, `tick` is the single input the rules read,
//! and `close` ends the run.
//!
//! Four things happen to a session, each at most once: a view, a mark, a
//! completion and, on every tick after the fifth second, a resume point. The
//! ordering lives in `effects`, which is pure over the session and testable
//! without a store; everything that writes is below it.
//!
//! The thresholds are Electron's, carried one for one:
//! `src/main/services/mpvPlayback.ts` (`MAX_TICK_DELTA_SEC`, the one-second
//! poll and its "only forward movement at roughly real time" rule),
//! `src/main/services/viewHistory.ts` (`markViewed` and its newer-wins
//! guard), `src/main/handlers/externalPlaybackHandler.ts` (thirty seconds,
//! 85 percent, the extra rule, the no-id warning and the `tracked: false`
//! rule), `src/renderer/pages/VideoPlayer.tsx` (`autoMarkAt`, the tail write
//! and the `ended` handler) and `src/renderer/utils/playbackProgress.ts`
//! (`RESUME_HEAD_SKIP`, `RESUME_TAIL_SKIP`).

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::*;
use crate::core::Core;
use crate::library::cards;
use crate::library::{labels, titles};
use crate::prefs;
use crate::time;
use crate::trackers::writes;

/// Seconds of forward playback before the series counts as viewed.
pub const VIEW_SECS: f64 = 30.0;
/// The largest advance between two ticks that still reads as playing rather
/// than as a seek: one and a half times the shell's one-second tick.
pub const MAX_TICK_DELTA: f64 = 1.5;
/// How far through a file counts as watched when no outro is known. The
/// number AniList and MAL use themselves.
pub const MARK_FRACTION: f64 = 0.85;
/// A position under this is not worth resuming to.
pub const RESUME_HEAD: f64 = 5.0;
/// How far the playhead has to move before another resume point is worth
/// writing. Electron's `VideoPlayer.tsx` saves on the same four second
/// rule; a write per tick is about 1400 writer round trips per episode,
/// each of them queued behind whatever transaction the writer is running.
pub const RESUME_INTERVAL: f64 = 4.0;
/// A position within this of the end is a completion, not a resume point.
pub const RESUME_TAIL: f64 = 30.0;

/// What the core remembers about one run of the player. It lives in memory
/// for the life of the session and never reaches a table: the rows the rules
/// write are the record, and this is only what decides when to write them.
#[derive(Clone, Debug)]
pub struct Session {
    pub id: u64,
    pub file: u64,
    pub series: u64,
    /// The history key: the number for an episode, the file name for an
    /// extra or a film. Every row these rules write is keyed by it.
    pub episode_key: String,
    /// None for an extra, which has no episode number of its own.
    pub number: Option<f64>,
    pub is_extra: bool,
    pub is_film: bool,
    pub anilist_id: Option<u64>,
    pub mal_id: Option<u64>,
    /// Known only once the shell has reported the chapters, so the first
    /// ticks of a session have neither a mark line nor a tail to reach.
    pub duration: Option<f64>,
    pub outro_start: Option<f64>,
    pub last_position: Option<f64>,
    pub watched_secs: f64,
    pub viewed: bool,
    pub marked: bool,
    pub completed: bool,
    /// The position the last resume point was written at, which is what the
    /// four second throttle measures against.
    pub resume_written_at: Option<f64>,
}

/// Every open session, and the counter that names the next one. Ids are per
/// launch: a session does not survive a restart, and neither does the player.
#[derive(Default)]
pub struct Sessions {
    next: AtomicU64,
    map: Mutex<HashMap<u64, Session>>,
}

impl Sessions {
    /// A poisoned lock is recovered rather than propagated: the map holds
    /// plain data, so a panic elsewhere leaves it perfectly usable.
    fn lock(&self) -> MutexGuard<'_, HashMap<u64, Session>> {
        self.map.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// One session as it stands, cloned so the caller holds no lock while
    /// it works. The skip windows job validates through this and then
    /// carries the clone across its awaits.
    pub(crate) fn get(&self, id: u64) -> Option<Session> {
        self.lock().get(&id).cloned()
    }
}

// ---------------------------------------------------------------------------
// The rules
// ---------------------------------------------------------------------------

/// What one tick asks the store to do. The list comes out in the order the
/// rules fire, and the caller applies it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    View,
    Mark,
    Complete,
    Resume(f64),
}

/// How a tick arrived. A paused tick and a session's last tick both write
/// their resume point whatever the throttle says, since either could be the
/// last word on where the playhead was.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TickKind {
    Playing,
    Paused,
    Closing,
}

/// The four rules over one tick. Pure but for the session's own flags, each
/// of which latches the first time its rule fires, so no rule can happen
/// twice in one session.
pub fn effects(s: &mut Session, position: f64, kind: TickKind) -> Vec<Effect> {
    let paused = kind == TickKind::Paused;
    let mut out = Vec::new();
    // A player that reports nonsense says nothing about where the playhead
    // is. Nothing is written and nothing is remembered, so the next real
    // tick still measures against the last real one.
    if !position.is_finite() || position < 0.0 {
        return out;
    }
    if let Some(last) = s.last_position {
        let delta = position - last;
        // Only forward movement at roughly real time counts. A backwards or
        // oversized jump is a seek, so scrubbing to the credits never reads
        // as having watched the episode.
        if !paused && delta > 0.0 && delta <= MAX_TICK_DELTA {
            s.watched_secs += delta;
        }
    }
    s.last_position = Some(position);

    if !s.viewed && !s.is_extra && s.watched_secs >= VIEW_SECS {
        s.viewed = true;
        out.push(Effect::View);
    }
    if !s.marked && !s.is_extra {
        // The earlier of a known outro's start and 85 percent of the
        // duration, so the mark still fires when no outro is known or the
        // outro is unusually short.
        let mut at: Option<f64> = s.duration.map(|d| d * MARK_FRACTION);
        if let Some(o) = s.outro_start {
            at = Some(at.map_or(o, |a| a.min(o)));
        }
        if at.is_some_and(|a| position >= a) {
            s.marked = true;
            out.push(Effect::Mark);
        }
    }
    let tail = s.duration.is_some_and(|d| position >= d - RESUME_TAIL);
    let outro = s.outro_start.is_some_and(|o| position >= o);
    if !s.completed && (tail || outro) {
        s.completed = true;
        out.push(Effect::Complete);
    } else if !s.completed && position >= RESUME_HEAD {
        // Throttled by position, not by the clock: a tick that moved the
        // playhead less than the interval says nothing new, and a seek in
        // either direction says a great deal.
        let moved = s
            .resume_written_at
            .is_none_or(|last| (position - last).abs() >= RESUME_INTERVAL);
        if moved || kind != TickKind::Playing {
            s.resume_written_at = Some(position);
            out.push(Effect::Resume(position));
        }
    }
    out
}

/// The completion a `ClosePlayback { Ended }` applies whatever the position
/// says: the file ran out, so the episode is done.
fn ended(s: &mut Session) -> Option<Effect> {
    if s.completed {
        return None;
    }
    s.completed = true;
    Some(Effect::Complete)
}

// ---------------------------------------------------------------------------
// Opening a session
// ---------------------------------------------------------------------------

/// The file row a session stands on.
struct FileRow {
    series: u64,
    path: String,
    is_extra: bool,
    number: f64,
    season: Option<u32>,
    extra_kind: Option<ExtraKind>,
    extra_index: Option<u32>,
    episode_key: String,
    sidecars: Vec<Sidecar>,
}

/// The series row behind it, with the match and the media it carries.
struct SeriesRow {
    kind: SeriesKind,
    folder_name: String,
    anilist_id: Option<u64>,
    mal_id: Option<u64>,
    track_choice: TrackChoice,
    title_romaji: Option<String>,
    title_english: Option<String>,
    total: Option<u32>,
    poster: Option<String>,
}

fn file_row(conn: &Connection, file: u64) -> Result<FileRow, CoreError> {
    let row = conn
        .query_row(
            "SELECT series_id, path, kind, number, season, extra_kind, extra_index, episode_key, sidecars FROM files WHERE id = ?1",
            params![file as i64],
            |r| {
                Ok(FileRow {
                    series: r.get::<_, i64>(0)? as u64,
                    path: r.get(1)?,
                    // A row that is not marked an extra is a real episode,
                    // the way the cards read the same column.
                    is_extra: r.get::<_, String>(2)? == "extra",
                    number: r.get::<_, Option<f64>>(3)?.unwrap_or(0.0),
                    season: r.get::<_, Option<i64>>(4)?.map(|v| v as u32),
                    extra_kind: r.get::<_, Option<String>>(5)?.and_then(|s| ExtraKind::from_column(&s)),
                    extra_index: r.get::<_, Option<i64>>(6)?.map(|v| v as u32),
                    episode_key: r.get(7)?,
                    sidecars: sidecars_of(&r.get::<_, String>(8)?),
                })
            },
        )
        .optional()?;
    row.ok_or(CoreError::NotFound {
        what: Entity::File,
        id: file,
    })
}

fn series_row(conn: &Connection, series: u64, images_dir: &Path) -> Result<SeriesRow, CoreError> {
    let row = conn
        .query_row(
            "SELECT s.kind, s.folder_name, s.anilist_id, s.mal_id, s.track_choice,
                    m.title_romaji, m.title_english, m.episodes, i.path
             FROM series s
             LEFT JOIN anilist_media m ON m.id = s.anilist_id
             LEFT JOIN images i ON i.url = m.cover_url
             WHERE s.id = ?1",
            params![series as i64],
            |r| {
                Ok(SeriesRow {
                    kind: SeriesKind::from_column(&r.get::<_, String>(0)?)
                        .unwrap_or(SeriesKind::Show),
                    folder_name: r.get(1)?,
                    anilist_id: r.get::<_, Option<i64>>(2)?.map(|v| v as u64),
                    mal_id: r.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                    track_choice: track_choice_of(r.get::<_, Option<String>>(4)?.as_deref()),
                    title_romaji: r.get(5)?,
                    title_english: r.get(6)?,
                    // A published total of nought is AniList saying it does
                    // not know yet, so it never makes an episode the last.
                    total: r
                        .get::<_, Option<i64>>(7)?
                        .map(|v| v as u32)
                        .filter(|t| *t > 0),
                    poster: r
                        .get::<_, Option<String>>(8)?
                        .map(|p| images_dir.join(p).to_string_lossy().into_owned()),
                })
            },
        )
        .optional()?;
    row.ok_or(CoreError::NotFound {
        what: Entity::Series,
        id: series,
    })
}

/// A file's sidecar subtitles. A row that does not parse is a file with no
/// sidecars rather than a session that cannot open.
fn sidecars_of(raw: &str) -> Vec<Sidecar> {
    match serde_json::from_str::<Vec<Sidecar>>(raw) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("a file's sidecars did not parse, treating it as none: {e}");
            Vec::new()
        }
    }
}

fn track_choice_of(raw: Option<&str>) -> TrackChoice {
    let Some(raw) = raw else {
        return TrackChoice::default();
    };
    match serde_json::from_str::<TrackChoice>(raw) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("a series' track choice did not parse, starting from the defaults: {e}");
            TrackChoice::default()
        }
    }
}

/// The episodes either side of this one on disk. Only whole-numbered
/// episodes are neighbours, so a 12.5 recap sits between twelve and thirteen
/// rather than in the running order, and Next from it opens thirteen.
fn neighbours(
    conn: &Connection,
    series: u64,
    number: f64,
) -> Result<(Option<u64>, Option<u64>), CoreError> {
    let mut stmt =
        conn.prepare("SELECT id, number FROM files WHERE series_id = ?1 AND kind = 'episode' AND number IS NOT NULL ORDER BY number, id")?;
    let rows = stmt.query_map(params![series as i64], |r| {
        Ok((r.get::<_, i64>(0)? as u64, r.get::<_, f64>(1)?))
    })?;
    let (mut prev, mut next) = (None, None);
    for row in rows {
        let (id, n) = row?;
        if n.fract() != 0.0 {
            continue;
        }
        if n < number {
            prev = Some(id);
        } else if n > number && next.is_none() {
            next = Some(id);
        }
    }
    Ok((prev, next))
}

/// The windows the last session on this episode settled on. A `none` row is
/// a miss the skip windows job will ask about again, so it reads as absent.
fn cached_windows(conn: &Connection, series: u64, key: &str) -> Result<Vec<SkipWindow>, CoreError> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT windows FROM skip_windows WHERE series_id = ?1 AND episode_key = ?2 AND source <> 'none'",
            params![series as i64, key],
            |r| r.get(0),
        )
        .optional()?;
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    Ok(
        serde_json::from_str::<Vec<SkipWindow>>(&raw).unwrap_or_else(|e| {
            tracing::warn!("cached skip windows did not parse, asking again: {e}");
            Vec::new()
        }),
    )
}

/// Everything the player needs before the first frame, and the session the
/// ticks that follow belong to.
pub fn open(core: &Core, file: u64) -> Result<PlaybackSession, CoreError> {
    let images_dir = core.paths.images_dir();
    let (f, s, resume_from, skip_windows, prev, next, episode_title, subtitle_defaults, lang) =
        core.store.read(|c| {
            let f = file_row(c, file)?;
            let s = series_row(c, f.series, &images_dir)?;
            let resume_from: Option<f64> = c
                .query_row(
                    "SELECT position FROM resume_points WHERE series_id = ?1 AND episode_key = ?2",
                    params![f.series as i64, f.episode_key],
                    |r| r.get(0),
                )
                .optional()?;
            let skip_windows = cached_windows(c, f.series, &f.episode_key)?;
            // An extra stands outside the running order, and a film is its own
            // series, so neither has an episode either side of it.
            let (prev, next) = match (f.is_extra, s.kind) {
                (false, SeriesKind::Show) => neighbours(c, f.series, f.number)?,
                _ => (None, None),
            };
            let episode_title: Option<String> = match (s.anilist_id, f.is_extra) {
                (Some(id), false) => c
                    .query_row(
                        "SELECT title FROM anilist_episodes WHERE anilist_id = ?1 AND number = ?2",
                        params![id as i64, f.number],
                        |r| r.get(0),
                    )
                    .optional()?
                    .flatten(),
                _ => None,
            };
            let subtitle_defaults = prefs::load_settings(c)?.subtitle_defaults;
            let lang = prefs::load_preferences(c)?.title_language;
            Ok((
                f,
                s,
                resume_from,
                skip_windows,
                prev,
                next,
                episode_title,
                subtitle_defaults,
                lang,
            ))
        })?;

    let is_film = s.kind == SeriesKind::Movie;
    let code = if f.is_extra {
        labels::extra_code_with_index(f.extra_kind.unwrap_or(ExtraKind::Other), f.extra_index)
    } else {
        labels::episode_code(f.season, f.number)
    };
    // A film is the whole of what it is, an extra is never the end of
    // anything, and a show ends where the published total says, or where the
    // disk runs out when no total is known.
    let is_last_episode = if f.is_extra {
        false
    } else if is_film {
        true
    } else {
        match s.total {
            Some(total) => f.number >= f64::from(total),
            None => next.is_none(),
        }
    };

    let id = core.sessions.next_id();
    core.sessions.lock().insert(
        id,
        Session {
            id,
            file,
            series: f.series,
            episode_key: f.episode_key.clone(),
            number: (!f.is_extra).then_some(f.number),
            is_extra: f.is_extra,
            is_film,
            anilist_id: s.anilist_id,
            mal_id: s.mal_id,
            duration: None,
            outro_start: None,
            last_position: None,
            watched_secs: 0.0,
            viewed: false,
            marked: false,
            completed: false,
            resume_written_at: None,
        },
    );

    Ok(PlaybackSession {
        session: id,
        file,
        path: f.path,
        series: f.series,
        series_title: titles::resolve(
            lang,
            s.title_romaji.as_deref(),
            s.title_english.as_deref(),
            &s.folder_name,
        ),
        episode_title,
        code,
        is_extra: f.is_extra,
        is_last_episode,
        resume_from,
        prev,
        next,
        sidecars: f.sidecars,
        skip_windows,
        artwork: s.poster,
        subtitle_defaults,
        track_choice: s.track_choice,
    })
}

/// What the shell learned when the file loaded. Called twice by the skip
/// windows job: once with the duration alone, and again once an outro is
/// known. A session that has closed in between is nothing to report.
pub fn report_chapters(core: &Core, session: u64, duration: f64, outro_start: Option<f64>) {
    let mut map = core.sessions.lock();
    let Some(s) = map.get_mut(&session) else {
        tracing::debug!("chapters for session {session}, which has already closed");
        return;
    };
    if duration.is_finite() && duration > 0.0 {
        s.duration = Some(duration);
    }
    if let Some(outro) = outro_start.filter(|o| o.is_finite() && *o >= 0.0) {
        s.outro_start = Some(outro);
    }
}

// ---------------------------------------------------------------------------
// The tick and the close
// ---------------------------------------------------------------------------

/// One tick from the player. The rules run under the sessions lock, so each
/// fires exactly once; nothing that touches the store runs while it is held.
pub fn tick(core: &Core, session: u64, position: f64, paused: bool) -> Result<(), CoreError> {
    let (s, fired) = {
        let mut map = core.sessions.lock();
        let Some(s) = map.get_mut(&session) else {
            return Err(CoreError::NotFound {
                what: Entity::Session,
                id: session,
            });
        };
        let fired = effects(
            s,
            position,
            if paused {
                TickKind::Paused
            } else {
                TickKind::Playing
            },
        );
        (s.clone(), fired)
    };
    apply(core, &s, &fired)
}

/// The end of a session: a last tick, the completion the end of the file
/// implies, and the session gone. Closing one that is already closed is the
/// same as closing it once, so a shell that says goodbye twice is fine.
pub fn close(
    core: &Core,
    session: u64,
    position: f64,
    reason: CloseReason,
) -> Result<(), CoreError> {
    let taken = core.sessions.lock().remove(&session);
    let Some(mut s) = taken else {
        tracing::debug!("close for session {session}, which is already closed");
        return Ok(());
    };
    let mut fired = effects(&mut s, position, TickKind::Closing);
    if reason == CloseReason::Ended
        && let Some(effect) = ended(&mut s)
    {
        // The resume point this tick was about to write is one completion
        // would delete in the same breath.
        fired.retain(|e| !matches!(e, Effect::Resume(_)));
        fired.push(effect);
    }
    apply(core, &s, &fired)
}

fn apply(core: &Core, s: &Session, fired: &[Effect]) -> Result<(), CoreError> {
    for effect in fired {
        match effect {
            Effect::View => record_view(core, s)?,
            Effect::Mark => mark(core, s),
            Effect::Complete => complete(core, s)?,
            Effect::Resume(position) => write_resume(core, s, *position),
        }
    }
    Ok(())
}

/// The library's Last viewed sort, one row per series. The write only lands
/// when it is newer than what is there, so a session that outlived a clock
/// change cannot clobber a fresher one.
fn record_view(core: &Core, s: &Session) -> Result<(), CoreError> {
    let (series, key, now) = (s.series, s.episode_key.clone(), time::now_secs());
    core.store.write(move |c| {
        c.execute(
            "INSERT INTO views (series_id, episode_key, at) VALUES (?1, ?2, ?3)
             ON CONFLICT(series_id) DO UPDATE SET episode_key = excluded.episode_key, at = excluded.at
             WHERE excluded.at > views.at",
            params![series as i64, key, now],
        )?;
        Ok(())
    })?;
    let (folder, code) = core.store.read(|c| line_labels(c, s))?;
    core.bus.info(
        Stage::Playback,
        format!("viewed {folder} {code}"),
        EventBody::Viewed {
            series: s.series,
            episode: s.episode_key.clone(),
        },
    );
    Ok(())
}

/// The mark the rule fires, as the Mark job every other mark goes through.
/// The rule has already happened by the time this runs, so whatever the
/// write answers is the outcome and none of it is a failure of the tick.
fn mark(core: &Core, s: &Session) {
    let Some(number) = s.number else { return };
    if s.anilist_id.is_none() && s.mal_id.is_none() {
        let folder = core
            .store
            .read(|c| line_labels(c, s))
            .map(|(folder, _)| folder)
            .unwrap_or_default();
        core.bus.warn(
            Stage::Playback,
            format!("watched to the end but {folder} has no AniList or MAL id"),
            EventBody::Notice,
        );
        return;
    }
    // The hidden guard, ahead of the job's own: a series the user hid never
    // gets a mark because it was played, and it says nothing about it.
    let hidden = core
        .store
        .read(|c| {
            for (tracker, id) in [(Tracker::Anilist, s.anilist_id), (Tracker::Mal, s.mal_id)] {
                if let Some(id) = id
                    && writes::is_hidden(c, tracker, id)?
                {
                    return Ok(true);
                }
            }
            Ok(false)
        })
        .unwrap_or(false);
    if hidden {
        return;
    }
    if let Err(e) = writes::mark(core, s.series, number) {
        tracing::debug!("the mark rule wrote nothing for series {}: {e}", s.series);
    }
}

/// The episode is done: the resume point goes, the completion is recorded,
/// and the card behind the player is pushed so Next up moves on. An extra
/// only forgets where it was, since it is not an episode of anything.
fn complete(core: &Core, s: &Session) -> Result<(), CoreError> {
    let (series, key, now, record) = (
        s.series,
        s.episode_key.clone(),
        time::now_secs(),
        !s.is_extra,
    );
    core.store.tx(move |tx| {
        tx.execute(
            "DELETE FROM resume_points WHERE series_id = ?1 AND episode_key = ?2",
            params![series as i64, key],
        )?;
        if record {
            tx.execute(
                "INSERT INTO completed (series_id, episode_key, at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(series_id, episode_key) DO UPDATE SET at = excluded.at",
                params![series as i64, key, now],
            )?;
        }
        Ok(())
    })?;
    core.bus.debug(
        Stage::Playback,
        "resume point cleared",
        EventBody::ResumePointChanged {
            file: s.file,
            position: None,
        },
    );
    if s.is_extra {
        return Ok(());
    }
    let images_dir = core.paths.images_dir();
    let cards = core
        .store
        .read(|c| cards::cards_for(c, &images_dir, &[s.series]))?;
    let title = cards.first().map_or_else(String::new, |c| c.title.clone());
    let what = if s.is_film { "film" } else { "episode" };
    core.bus.debug(
        Stage::Playback,
        format!("{title} finished a {what}"),
        EventBody::SeriesChanged { series: cards },
    );
    Ok(())
}

/// Posted rather than written: a tick comes in on the shell's own thread,
/// and what it needs back is the event, which goes out at once. The row is
/// queued behind whatever the writer is doing, and the queue is FIFO, so a
/// completion that deletes the row still runs after the upsert before it.
fn write_resume(core: &Core, s: &Session, position: f64) {
    let (series, key, now) = (s.series, s.episode_key.clone(), time::now_secs());
    // A duration is only known once the shell has reported the chapters, and
    // the row wants a number either way; nought reads as "not known yet" to
    // every fraction drawn off it.
    let duration = s.duration.unwrap_or(0.0);
    let named = key.clone();
    core.store.post(move |c| {
        if let Err(e) = c.execute(
            "INSERT INTO resume_points (series_id, episode_key, position, duration, at) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(series_id, episode_key) DO UPDATE SET position = excluded.position, duration = excluded.duration, at = excluded.at",
            params![series as i64, key, position, duration, now],
        ) {
            tracing::warn!("the resume point for series {series} {named} was not written: {e}");
        }
    });
    core.bus.debug(
        Stage::Playback,
        "resume point saved",
        EventBody::ResumePointChanged {
            file: s.file,
            position: Some(position),
        },
    );
}

/// What the lines these rules write call the file: the series' folder name
/// and the episode's own code. Read when a rule fires, at most once each per
/// session, rather than carried on every session for the two that need it.
fn line_labels(conn: &Connection, s: &Session) -> Result<(String, String), CoreError> {
    let row = conn
        .query_row(
            "SELECT s.folder_name, f.kind, f.number, f.season, f.extra_kind, f.extra_index
             FROM files f JOIN series s ON s.id = f.series_id
             WHERE f.id = ?1",
            params![s.file as i64],
            |r| {
                let folder: String = r.get(0)?;
                let is_extra: bool = r.get::<_, String>(1)? == "extra";
                let number: f64 = r.get::<_, Option<f64>>(2)?.unwrap_or(0.0);
                let season: Option<u32> = r.get::<_, Option<i64>>(3)?.map(|v| v as u32);
                let kind = r
                    .get::<_, Option<String>>(4)?
                    .and_then(|k| ExtraKind::from_column(&k));
                let index: Option<u32> = r.get::<_, Option<i64>>(5)?.map(|v| v as u32);
                let code = if is_extra {
                    labels::extra_code_with_index(kind.unwrap_or(ExtraKind::Other), index)
                } else {
                    labels::episode_code(season, number)
                };
                Ok((folder, code))
            },
        )
        .optional()?;
    // The file went away mid-session. The rule still stands, so the line is
    // written with what the session itself knows.
    Ok(row.unwrap_or_else(|| (String::new(), s.episode_key.clone())))
}

// ---------------------------------------------------------------------------
// The track choice
// ---------------------------------------------------------------------------

/// The series' playback memory: which audio and which subtitles the last
/// episode was watched with, so the next one opens on the same pair.
pub fn set_track_choice(
    core: &Core,
    series: u64,
    audio: Option<TrackRef>,
    subtitle: Option<SubtitleChoice>,
) -> Result<Reply, CoreError> {
    let json = serde_json::to_string(&TrackChoice { audio, subtitle })?;
    let changed = core.store.write(move |c| {
        Ok(c.execute(
            "UPDATE series SET track_choice = ?1 WHERE id = ?2",
            params![json, series as i64],
        )?)
    })?;
    if changed == 0 {
        return Err(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        });
    }
    Ok(Reply::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(is_extra: bool, duration: Option<f64>) -> Session {
        Session {
            id: 1,
            file: 10,
            series: 2,
            episode_key: "3".to_string(),
            number: (!is_extra).then_some(3.0),
            is_extra,
            is_film: false,
            anilist_id: Some(1),
            mal_id: None,
            duration,
            outro_start: None,
            last_position: None,
            watched_secs: 0.0,
            viewed: false,
            marked: false,
            completed: false,
            resume_written_at: None,
        }
    }

    /// One tick a second from nought: the view lands at thirty, and a
    /// resume point comes every four seconds from the fifth second on.
    #[test]
    fn a_view_needs_thirty_seconds_of_forward_movement() {
        let mut s = session(false, None);
        let mut views = 0;
        let mut resumes = Vec::new();
        for step in 0..40 {
            for effect in effects(&mut s, f64::from(step), TickKind::Playing) {
                match effect {
                    Effect::View => views += 1,
                    Effect::Resume(p) => resumes.push(p),
                    other => panic!("{other:?}"),
                }
            }
        }
        assert_eq!(views, 1);
        assert_eq!(
            resumes,
            vec![5.0, 9.0, 13.0, 17.0, 21.0, 25.0, 29.0, 33.0, 37.0]
        );
        assert_eq!(s.watched_secs, 39.0);
    }

    /// Electron's four second rule, carried over: a tick that barely moved
    /// the playhead says nothing new, and one write per tick is about 1400
    /// writer round trips per episode. A pause and the session's last tick
    /// write whatever the throttle says, since either could be the last
    /// word on where the playhead was, and so does a seek.
    #[test]
    fn a_resume_point_waits_four_seconds_unless_the_tick_is_the_last_word() {
        let mut s = session(false, Some(1400.0));
        let mut resumes = Vec::new();
        for step in 0..20 {
            for effect in effects(&mut s, f64::from(step), TickKind::Playing) {
                if let Effect::Resume(p) = effect {
                    resumes.push(p);
                }
            }
        }
        assert_eq!(resumes, vec![5.0, 9.0, 13.0, 17.0]);
        assert_eq!(
            effects(&mut s, 20.0, TickKind::Paused),
            vec![Effect::Resume(20.0)]
        );
        assert_eq!(
            effects(&mut s, 21.0, TickKind::Closing),
            vec![Effect::Resume(21.0)]
        );
        assert_eq!(
            effects(&mut s, 8.0, TickKind::Playing),
            vec![Effect::Resume(8.0)]
        );
    }

    /// A seek carries no watch time, in either direction, and neither does a
    /// tick that arrives while the player is paused.
    #[test]
    fn a_seek_and_a_pause_count_nothing() {
        let mut s = session(false, None);
        effects(&mut s, 0.0, TickKind::Playing);
        effects(&mut s, 1.0, TickKind::Playing);
        assert_eq!(s.watched_secs, 1.0);
        effects(&mut s, 600.0, TickKind::Playing);
        effects(&mut s, 10.0, TickKind::Playing);
        assert_eq!(s.watched_secs, 1.0);
        effects(&mut s, 11.0, TickKind::Paused);
        assert_eq!(s.watched_secs, 1.0);
        // Exactly one and a half times the tick still reads as playing.
        effects(&mut s, 12.5, TickKind::Playing);
        assert_eq!(s.watched_secs, 2.5);
        effects(&mut s, 14.1, TickKind::Playing);
        assert_eq!(s.watched_secs, 2.5);
    }

    /// The earlier of the two lines wins, and each rule fires once.
    #[test]
    fn the_mark_takes_the_earlier_of_the_outro_and_85_percent() {
        let mut s = session(false, Some(1400.0));
        assert_eq!(
            effects(&mut s, 1189.0, TickKind::Playing),
            vec![Effect::Resume(1189.0)]
        );
        // One second on from the last resume point, so the mark is all
        // this tick has to say.
        assert_eq!(
            effects(&mut s, 1190.0, TickKind::Playing),
            vec![Effect::Mark]
        );
        assert_eq!(
            effects(&mut s, 1200.0, TickKind::Playing),
            vec![Effect::Resume(1200.0)]
        );

        let mut s = session(false, Some(1400.0));
        s.outro_start = Some(1100.0);
        assert_eq!(
            effects(&mut s, 1099.0, TickKind::Playing),
            vec![Effect::Resume(1099.0)]
        );
        assert_eq!(
            effects(&mut s, 1100.0, TickKind::Playing),
            vec![Effect::Mark, Effect::Complete]
        );

        // An outro with no duration still carries a mark line of its own.
        let mut s = session(false, None);
        s.outro_start = Some(300.0);
        assert_eq!(
            effects(&mut s, 300.0, TickKind::Playing),
            vec![Effect::Mark, Effect::Complete]
        );
    }

    /// Completion is the tail or the outro, and it replaces the resume point
    /// rather than sitting beside it.
    #[test]
    fn completion_takes_the_tail_and_never_writes_a_resume_point() {
        let mut s = session(false, Some(1400.0));
        s.marked = true;
        assert_eq!(
            effects(&mut s, 1369.0, TickKind::Playing),
            vec![Effect::Resume(1369.0)]
        );
        assert_eq!(
            effects(&mut s, 1370.0, TickKind::Playing),
            vec![Effect::Complete]
        );
        assert_eq!(effects(&mut s, 1380.0, TickKind::Playing), Vec::new());
        assert_eq!(effects(&mut s, 1399.0, TickKind::Playing), Vec::new());
    }

    /// An extra shares its number with a real episode, so it never records a
    /// view and never moves a tracker. It still remembers where it was.
    #[test]
    fn an_extra_only_ever_writes_a_resume_point() {
        let mut s = session(true, Some(100.0));
        let mut fired = Vec::new();
        for step in 0..40 {
            fired.extend(effects(&mut s, f64::from(step), TickKind::Playing));
        }
        assert!(fired.iter().all(|e| matches!(e, Effect::Resume(_))));
        assert_eq!(
            effects(&mut s, 90.0, TickKind::Playing),
            vec![Effect::Complete]
        );
    }

    /// A player reporting nonsense says nothing about where the playhead is,
    /// so the session is left exactly as it was.
    #[test]
    fn a_bad_position_changes_nothing() {
        let mut s = session(false, Some(1400.0));
        effects(&mut s, 10.0, TickKind::Playing);
        effects(&mut s, 11.0, TickKind::Playing);
        assert_eq!(effects(&mut s, f64::NAN, TickKind::Playing), Vec::new());
        assert_eq!(
            effects(&mut s, f64::INFINITY, TickKind::Playing),
            Vec::new()
        );
        assert_eq!(effects(&mut s, -5.0, TickKind::Playing), Vec::new());
        assert_eq!(s.last_position, Some(11.0));
        assert_eq!(s.watched_secs, 1.0);
        // The next real tick still measures against the last real one. It
        // writes no resume point, being two seconds on from the one at ten.
        assert_eq!(effects(&mut s, 12.0, TickKind::Playing), Vec::new());
        assert_eq!(s.watched_secs, 2.0);
    }

    /// The end of the file completes whatever the position says, and a
    /// second close has nothing left to do.
    #[test]
    fn the_end_of_the_file_completes_once() {
        let mut s = session(false, None);
        assert_eq!(ended(&mut s), Some(Effect::Complete));
        assert_eq!(ended(&mut s), None);
    }
}
