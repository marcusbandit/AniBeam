//! Where an episode's intro and outro are.
//!
//! The file's own chapters come first: they are local, free and the
//! encoder's own word. When they name neither, the core asks AniSkip by
//! MAL id, and whatever comes back is cached on the episode so the next
//! play costs nothing. A miss is cached too, since asking again on every
//! play is what Electron did; it is only good for seven days, because
//! AniSkip's data arrives over time.
//!
//! Carried from Electron's `src/main/utils/chapterProbe.ts` (`OP_PATTERN`,
//! `ED_PATTERN`, `MAX_CHAPTER_SECONDS`) and
//! `src/main/handlers/aniSkipHandler.ts` (chapters first, the 404 that is
//! a miss rather than an error, and the network failure that caches
//! nothing so the next session asks again).

use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::metadata::apply::owner;
use crate::metadata::fetch::message_of;
use crate::playback::session::{self, Session};
use crate::time;

/// The longest a chapter may run and still describe an intro or an outro.
/// Some files carry one chapter over the whole episode labelled Opening,
/// and skipping the episode is not what the label meant.
pub const MAX_CHAPTER_SECS: f64 = 300.0;

/// How long a cached miss stands before AniSkip is asked about the episode
/// again.
pub const MISS_RETRY: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// The `source` column of a cached miss. Not a `SkipSource`: the contract
/// carries only the two sources an answer can come from, and this row says
/// there was no answer.
const MISS: &str = "none";

/// Anchored at the start of the title and closed on a word boundary, so
/// "Episode 1" and "Chapter 1" never match and neither does "Operations".
static OP_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(intro|opening|prologue|op(?:\s*[0-9]+)?)\b").unwrap());
static ED_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(outro|ending|closing|credits|end\s*credits|ed(?:\s*[0-9]+)?)\b").unwrap()
});

// ---------------------------------------------------------------------------
// The chapters
// ---------------------------------------------------------------------------

/// The windows the file's own chapters describe. A `Chapter` carries a
/// title and a start, so its span runs to the next chapter's start, or to
/// the duration for the last one. The first chapter of each kind wins.
pub fn from_chapters(chapters: &[Chapter], duration: f64) -> Vec<SkipWindow> {
    let mut intro: Option<SkipWindow> = None;
    let mut outro: Option<SkipWindow> = None;
    for (i, chapter) in chapters.iter().enumerate() {
        if intro.is_some() && outro.is_some() {
            break;
        }
        let title = chapter.title.trim();
        if title.is_empty() {
            continue;
        }
        let (start, end) = (
            chapter.start,
            chapters.get(i + 1).map_or(duration, |next| next.start),
        );
        // A span that runs backwards, or nowhere, or off a player
        // reporting nonsense, describes nothing to skip.
        if !start.is_finite() || !end.is_finite() || end <= start || end - start > MAX_CHAPTER_SECS
        {
            continue;
        }
        if intro.is_none() && OP_PATTERN.is_match(title) {
            intro = Some(SkipWindow {
                kind: SkipKind::Intro,
                start,
                end,
                source: SkipSource::Chapters,
            });
        } else if outro.is_none() && ED_PATTERN.is_match(title) {
            outro = Some(SkipWindow {
                kind: SkipKind::Outro,
                start,
                end,
                source: SkipSource::Chapters,
            });
        }
    }
    intro.into_iter().chain(outro).collect()
}

// ---------------------------------------------------------------------------
// The cache
// ---------------------------------------------------------------------------

/// What the last session settled on for this episode. A found answer
/// stands; a miss is only good for `MISS_RETRY`, after which it reads as
/// absent and the job asks again.
pub fn cached(
    conn: &Connection,
    series: u64,
    key: &str,
    now: i64,
) -> Result<Option<Vec<SkipWindow>>, CoreError> {
    let row: Option<(String, String, i64)> = conn
        .query_row(
            "SELECT windows, source, fetched_at FROM skip_windows WHERE series_id = ?1 AND episode_key = ?2",
            params![series as i64, key],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((raw, source, fetched_at)) = row else {
        return Ok(None);
    };
    if source == MISS && now.saturating_sub(fetched_at) >= miss_retry_secs() {
        return Ok(None);
    }
    match serde_json::from_str::<Vec<SkipWindow>>(&raw) {
        Ok(windows) => Ok(Some(windows)),
        // A row nothing can be read out of is worth no more than no row.
        Err(e) => {
            tracing::warn!("cached skip windows did not parse, asking again: {e}");
            Ok(None)
        }
    }
}

fn miss_retry_secs() -> i64 {
    i64::try_from(MISS_RETRY.as_secs()).unwrap_or(i64::MAX)
}

/// The answer, and where it came from, against the episode. One row per
/// episode, replaced whole every time a session settles on one.
async fn write_cache(
    core: &Core,
    s: &Session,
    windows: &[SkipWindow],
    source: &str,
) -> Result<(), CoreError> {
    let json = serde_json::to_string(windows)?;
    let (series, key, source, now) = (
        s.series,
        s.episode_key.clone(),
        source.to_string(),
        time::now_secs(),
    );
    core.store
        .write_async(move |c| {
            c.execute(
                "INSERT INTO skip_windows (series_id, episode_key, windows, source, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(series_id, episode_key) DO UPDATE SET
                     windows = excluded.windows, source = excluded.source, fetched_at = excluded.fetched_at",
                params![series as i64, key, json, source, now],
            )?;
            Ok(())
        })
        .await
}

// ---------------------------------------------------------------------------
// The job
// ---------------------------------------------------------------------------

/// Answers `ReportChapters`: the duration lands on the session at once, so
/// the mark and the completion rules have it from this tick on, and the
/// windows follow as the SkipWindows job.
pub fn start(
    core: &Core,
    session: u64,
    chapters: Vec<Chapter>,
    duration: f64,
) -> Result<u64, CoreError> {
    let s = core.sessions.get(session).ok_or(CoreError::NotFound {
        what: Entity::Session,
        id: session,
    })?;
    if !duration.is_finite() || duration <= 0.0 {
        return Err(CoreError::invalid(
            "duration",
            "a duration is a positive number of seconds",
        ));
    }
    session::report_chapters(core, session, duration, None);
    let owner = owner(core)?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::SkipWindows, move |ctx| async move {
            let windows = resolve(&owner, &ctx, &s, &chapters, duration).await?;
            // The outro's start is both a mark line and a completion line, so
            // the session takes it the moment it is known. A session that has
            // closed while the lookup was in flight is nothing to report.
            if let Some(outro) = windows.iter().find(|w| w.kind == SkipKind::Outro) {
                session::report_chapters(&owner, s.id, duration, Some(outro.start));
            }
            Ok(Finished {
                level: Level::Debug,
                message: format!("skip windows: {}", summary(&windows)),
                body: EventBody::SkipWindowsReady {
                    session: s.id,
                    windows,
                },
            })
        }))
}

/// Chapters, then the cache, then AniSkip. Each step that produces an
/// answer writes it against the episode, so the next session has it
/// before the first frame.
async fn resolve(
    core: &Core,
    ctx: &JobCtx,
    s: &Session,
    chapters: &[Chapter],
    duration: f64,
) -> Result<Vec<SkipWindow>, CoreError> {
    // The encoder's own word, and free: when the file names an opening or
    // an ending there is nothing to ask anyone about.
    let from_file = from_chapters(chapters, duration);
    if !from_file.is_empty() {
        write_cache(core, s, &from_file, SkipSource::Chapters.as_str()).await?;
        return Ok(from_file);
    }
    let (series, key, now) = (s.series, s.episode_key.clone(), time::now_secs());
    if let Some(windows) = core
        .store
        .write_async(move |c| cached(c, series, &key, now))
        .await?
    {
        return Ok(windows);
    }
    // An extra has no episode number, so AniSkip has nothing to be asked
    // about and nothing to cache: its chapters are all it can ever have.
    if s.is_extra {
        return Ok(Vec::new());
    }
    let (Some(mal_id), Some(number)) = (s.mal_id, s.number) else {
        return Ok(Vec::new());
    };
    if !number.is_finite() || number <= 0.0 {
        return Ok(Vec::new());
    }
    // Both casts stand on the guards above: the number is finite and
    // positive here, and so is the duration `start` let through.
    let episode = number.floor() as u32;
    match core
        .aniskip
        .skip_times(mal_id, episode, duration.round() as u64)
        .await
    {
        Ok(Some(windows)) => {
            write_cache(core, s, &windows, SkipSource::AniSkip.as_str()).await?;
            Ok(windows)
        }
        // AniSkip holds nothing for this episode yet. Cached as a miss, so
        // the next play asks nobody, and asked about again in seven days.
        Ok(None) => {
            write_cache(core, s, &[], MISS).await?;
            Ok(Vec::new())
        }
        // A failure to ask says nothing about the episode, so nothing is
        // cached and the next session asks again.
        Err(e) => {
            ctx.emit(
                Level::Warn,
                format!(
                    "AniSkip did not answer for episode {episode}: {}",
                    message_of(&e)
                ),
                EventBody::Notice,
            );
            Ok(Vec::new())
        }
    }
}

/// What the terminal line names: the kinds found, in the order they were
/// found, or `none`.
fn summary(windows: &[SkipWindow]) -> String {
    if windows.is_empty() {
        return "none".to_string();
    }
    windows
        .iter()
        .map(|w| match w.kind {
            SkipKind::Intro => "intro",
            SkipKind::Outro => "outro",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chapter(title: &str, start: f64) -> Chapter {
        Chapter {
            title: title.to_string(),
            start,
        }
    }

    fn intro(start: f64, end: f64) -> SkipWindow {
        SkipWindow {
            kind: SkipKind::Intro,
            start,
            end,
            source: SkipSource::Chapters,
        }
    }

    fn outro(start: f64, end: f64) -> SkipWindow {
        SkipWindow {
            kind: SkipKind::Outro,
            start,
            end,
            source: SkipSource::Chapters,
        }
    }

    /// A chapter's span runs to the next chapter's start, and to the
    /// duration for the last one.
    #[test]
    fn a_chapters_span_runs_to_the_next_one() {
        let chapters = vec![
            chapter("Opening", 0.0),
            chapter("Part A", 90.0),
            chapter("Ending", 1300.0),
        ];
        assert_eq!(
            from_chapters(&chapters, 1400.0),
            vec![intro(0.0, 90.0), outro(1300.0, 1400.0)]
        );
    }

    /// The match is anchored and closed on a word boundary, so a numbered
    /// episode or chapter never reads as an opening, and neither does a
    /// word that merely starts with op.
    #[test]
    fn numbers_and_longer_words_never_match() {
        for title in [
            "Episode 1",
            "Chapter 1",
            "Operations",
            "Editorial",
            "",
            "   ",
        ] {
            let chapters = vec![chapter(title, 0.0), chapter("Part A", 90.0)];
            assert!(
                from_chapters(&chapters, 1400.0).is_empty(),
                "{title} matched"
            );
        }
        // A numbered opening or ending is still one.
        let chapters = vec![
            chapter("OP 2", 0.0),
            chapter("Part A", 90.0),
            chapter("ED2", 1300.0),
        ];
        assert_eq!(
            from_chapters(&chapters, 1400.0),
            vec![intro(0.0, 90.0), outro(1300.0, 1400.0)]
        );
    }

    /// One chapter labelled Opening over the whole episode is a label, not
    /// an instruction to skip the episode.
    #[test]
    fn a_chapter_longer_than_five_minutes_is_ignored() {
        assert!(from_chapters(&[chapter("Opening", 0.0)], 1400.0).is_empty());
        // Exactly five minutes is still an opening; a second past it is not.
        assert_eq!(
            from_chapters(&[chapter("Opening", 0.0)], 300.0),
            vec![intro(0.0, 300.0)]
        );
        assert!(from_chapters(&[chapter("Opening", 0.0)], 301.0).is_empty());
    }

    /// The first chapter of each kind wins, and a chapter that says
    /// nothing about where it ends says nothing at all.
    #[test]
    fn the_first_of_each_kind_wins_and_a_bad_span_is_skipped() {
        let chapters = vec![
            chapter("Opening", 0.0),
            chapter("Part A", 90.0),
            chapter("Opening", 600.0),
            chapter("Part B", 700.0),
            chapter("Credits", 1300.0),
            chapter("Ending", 1350.0),
        ];
        assert_eq!(
            from_chapters(&chapters, 1400.0),
            vec![intro(0.0, 90.0), outro(1300.0, 1350.0)]
        );

        // A chapter that starts where the one after it does, or after it,
        // has no span to skip.
        assert!(
            from_chapters(&[chapter("Opening", 90.0), chapter("Part A", 90.0)], 1400.0).is_empty()
        );
        assert!(
            from_chapters(
                &[chapter("Opening", f64::NAN), chapter("Part A", 90.0)],
                1400.0
            )
            .is_empty()
        );
        assert!(from_chapters(&[chapter("Ending", 1400.0)], 1400.0).is_empty());
    }
}
