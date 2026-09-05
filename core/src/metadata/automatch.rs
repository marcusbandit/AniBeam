//! The auto-match: the background job that gives an unmatched folder its
//! AniList identity, and the call that takes one away again.
//!
//! The rules are Electron's `posterMatch.findShowMatch`, carried over one
//! for one. The folder name goes to AniList's relevance-ordered search
//! verbatim, the same list the manual picker shows. Every hit is scored
//! against its romaji, english and native titles and its synonyms with
//! tokenised Dice similarity, the best score wins with ties breaking
//! toward AniList's own order, and the winner is accepted only if it
//! clears the threshold and has a cover image to draw.
//!
//! One attempt per series, ever. A miss stamps `attempted_at`, so the next
//! run walks past it rather than asking AniList the same question again;
//! the manual picker is the recovery, and `AUTO_MATCH_VERSION` is how a
//! change to these rules earns the whole library a second look.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{params, params_from_iter, Connection, OptionalExtension};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::Finished;
use crate::library::cards;
use crate::library::scan::LibraryState;
use crate::metadata::fetch::{self, message_of};
use crate::metadata::similarity::best_title_score;
use crate::net::anilist::Media;
use crate::store::settings;
use crate::time;

/// Accept at this score or better. Electron's bar, and it is a "pretty
/// close" one: anything further off is left for the manual picker.
pub const THRESHOLD: f64 = 0.5;

/// Bumped when the matcher's rules change. A stored version below this one
/// clears every failed attempt so the whole library is tried again under
/// the new rules; a match already made is never touched.
pub const AUTO_MATCH_VERSION: u32 = 1;

/// How many hits the search asks for: Electron's ten, the same page the
/// manual picker shows.
const RESULTS: u32 = 10;

/// The best-scoring candidate and its score, or nothing.
///
/// The cover is a gate on the winner rather than a filter over the field,
/// which is Electron's order: the best-scoring entry is the answer, and an
/// answer with no poster to draw is refused rather than quietly replaced
/// by a worse-scoring one.
pub fn pick<'a>(query: &str, results: &'a [Media]) -> Option<(&'a Media, f64)> {
    let mut best: Option<(&Media, f64)> = None;
    for r in results {
        let mut titles: Vec<Option<&str>> = vec![r.title.romaji.as_deref(), r.title.english.as_deref(), r.title.native.as_deref()];
        titles.extend(r.synonyms.iter().map(|s| Some(s.as_str())));
        let score = best_title_score(query, &titles);
        // Strict, so a tie keeps the earlier and more relevant result.
        if best.is_none_or(|(_, b)| score > b) {
            best = Some((r, score));
        }
    }
    let (m, score) = best?;
    if score < THRESHOLD {
        return None;
    }
    let cover = m.cover_image.as_ref().and_then(|c| c.extra_large.clone().or_else(|| c.large.clone()));
    cover.map(|_| (m, score))
}

/// The title a log line names the match by.
fn matched_title(m: &Media) -> String {
    m.title.romaji.clone().or_else(|| m.title.english.clone()).unwrap_or_else(|| "?".to_string())
}

/// Marks a series as being matched right now for as long as this lives.
/// A guard rather than a pair of calls, so a cancelled job that drops its
/// future mid-fetch still takes the id back out; a stuck id would keep the
/// watcher from ever arming a settle timer for that folder again.
struct InFlight<'a> {
    state: &'a LibraryState,
    id: u64,
}

impl<'a> InFlight<'a> {
    fn begin(state: &'a LibraryState, id: u64) -> InFlight<'a> {
        recover(&state.match_in_flight).insert(id);
        InFlight { state, id }
    }
}

impl Drop for InFlight<'_> {
    fn drop(&mut self) {
        recover(&self.state.match_in_flight).remove(&self.id);
    }
}

/// Nothing behind these locks is worth poisoning for the rest of the
/// process: they hold plain sets and join handles.
fn recover<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|e| e.into_inner())
}

/// The series with a settle timer still armed. Their folders are still
/// being copied into, so a match now would be a match against a half
/// arrived folder; the timer's own firing is what brings them here.
fn armed(core: &Core) -> Vec<u64> {
    recover(&core.library.settle).keys().copied().collect()
}

/// `?,?,?` for an `IN` list. Ids are bound, never formatted into the SQL.
fn placeholders(n: usize) -> String {
    std::iter::repeat_n("?", n).collect::<Vec<_>>().join(",")
}

/// The next series worth a search: never matched, never attempted, its
/// files still on disk, and not one of `skip`. Ordered by id, so the run
/// is deterministic and the caller can walk it one at a time.
fn next_candidate(conn: &Connection, skip: &[u64]) -> Result<Option<(u64, String)>, CoreError> {
    let mut sql = String::from(
        "SELECT id, folder_name FROM series
         WHERE provider IS NULL AND attempted_at IS NULL AND missing_since IS NULL
           AND EXISTS (SELECT 1 FROM files WHERE files.series_id = series.id)",
    );
    if !skip.is_empty() {
        sql.push_str(&format!(" AND id NOT IN ({})", placeholders(skip.len())));
    }
    sql.push_str(" ORDER BY id LIMIT 1");
    let mut stmt = conn.prepare(&sql)?;
    let row = stmt
        .query_row(params_from_iter(skip.iter().map(|id| *id as i64)), |r| {
            Ok((r.get::<_, i64>(0)? as u64, r.get::<_, String>(1)?))
        })
        .optional()?;
    Ok(row)
}

/// Records that this series was looked at and found nothing, under the
/// rules of this version of the matcher.
async fn stamp_attempt(core: &Core, series: u64, now: i64) -> Result<(), CoreError> {
    core.store
        .write_async(move |c| {
            c.execute(
                "UPDATE series SET attempted_at = ?2, attempt_version = ?3 WHERE id = ?1",
                params![series as i64, now, i64::from(AUTO_MATCH_VERSION)],
            )?;
            Ok(())
        })
        .await
}

/// What one candidate's turn produced: the score and the title, or nothing
/// at all because no hit cleared the bar.
struct Hit {
    score: f64,
    title: String,
}

/// One series: the search, the pick, and, on a hit, the whole record.
async fn attempt(core: &Core, series: u64, folder: &str) -> Result<Option<Hit>, CoreError> {
    let results = core.anilist.search(folder, RESULTS).await?;
    let Some((media, score)) = pick(folder, &results) else { return Ok(None) };
    let title = matched_title(media);
    let anilist_id = media.id;
    let media = media.clone();
    // An auto-match is never confirmed: the user has not seen it, and only
    // their own pick or an import earns that flag.
    fetch::fetch_and_write(core, series, anilist_id, Some(media), false, time::now_secs()).await?;
    Ok(Some(Hit { score, title }))
}

/// Starts the AutoMatch job and returns its id. It runs one at a time, so
/// a second call while one is running is handed the running job's id: a
/// series that settles mid-run is picked up by the run already going,
/// because it is no longer armed by the time the loop asks again.
pub fn start(core: &Arc<Core>) -> u64 {
    let owner = core.clone();
    core.jobs.clone().start(JobKind::AutoMatch, move |ctx| async move {
        // 1. The backfill. A series carrying an AniList id with no provider
        //    beside it is an import, or a row from a build that wrote the
        //    id alone; either way it is matched and the column should say
        //    so before anything decides it is a candidate.
        let backfilled = owner
            .store
            .write_async(|c| {
                Ok(c.execute("UPDATE series SET provider = 'anilist' WHERE provider IS NULL AND anilist_id IS NOT NULL", [])? as u64)
            })
            .await?;

        // 2. The version guard. New rules earn every failed attempt a
        //    second look; a match already made is left alone.
        let reset = owner
            .store
            .tx_async(|tx| {
                let stored: u32 = settings::get(tx, settings::AUTO_MATCH_VERSION)?.unwrap_or(0);
                if stored >= AUTO_MATCH_VERSION {
                    return Ok(0u64);
                }
                let cleared =
                    tx.execute("UPDATE series SET attempted_at = NULL, attempt_version = NULL WHERE provider IS NULL AND attempted_at IS NOT NULL", [])?;
                settings::set(tx, settings::AUTO_MATCH_VERSION, &AUTO_MATCH_VERSION)?;
                Ok(cleared as u64)
            })
            .await?;
        if reset > 0 {
            ctx.emit(Level::Info, format!("auto-match v{AUTO_MATCH_VERSION}: re-attempting {reset} unmatched series"), EventBody::Notice);
        }

        // 3. The loop, one series at a time. `seen` is what keeps it
        //    finite: a transport failure deliberately stamps nothing, so
        //    without it the same series would come back round for ever.
        let (mut matched, mut unmatched) = (0u64, 0u64);
        let mut seen: HashSet<u64> = HashSet::new();
        let images_dir = owner.paths.images_dir();
        loop {
            ctx.checkpoint()?;
            let mut skip = armed(&owner);
            skip.extend(seen.iter().copied());
            skip.sort_unstable();
            skip.dedup();
            let Some((series, folder)) = owner.store.write_async(move |c| next_candidate(c, &skip)).await? else { break };
            seen.insert(series);
            ctx.progress(matched + unmatched, None, &folder);

            let outcome = {
                let _in_flight = InFlight::begin(&owner.library, series);
                attempt(&owner, series, &folder).await
            };
            match outcome {
                Ok(Some(hit)) => {
                    matched += 1;
                    let score = hit.score;
                    let title = hit.title;
                    ctx.emit(Level::Info, format!("match (AniList {score:.2}): {folder} -> {title}"), EventBody::Notice);
                    let dir = images_dir.clone();
                    let cards = owner.store.write_async(move |c| cards::cards_for(c, &dir, &[series])).await?;
                    if let Some(card) = cards.into_iter().next() {
                        ctx.changed(card);
                    }
                }
                Ok(None) => {
                    unmatched += 1;
                    stamp_attempt(&owner, series, time::now_secs()).await?;
                    ctx.emit(Level::Info, format!("no match for {folder} (threshold {THRESHOLD})"), EventBody::Notice);
                }
                // A rate limit the limiter could not ride out is AniList
                // saying stop, not this series failing. Nothing is
                // stamped and the job ends: stamping here would cost the
                // series its one attempt for a reason that had nothing to
                // do with it.
                Err(e) if matches!(&e, CoreError::Provider { status: Some(429), .. }) => return Err(e),
                // The provider answered, and the answer was an error. It
                // had its turn, so it is stamped and the run moves on.
                Err(e) if matches!(&e, CoreError::Provider { status: Some(_), .. }) => {
                    unmatched += 1;
                    ctx.emit(Level::Warn, format!("AniList search failed for {folder}: {}", message_of(&e)), EventBody::Notice);
                    stamp_attempt(&owner, series, time::now_secs()).await?;
                }
                // Nothing answered at all. The series never had its turn,
                // so nothing is stamped and the next run tries again.
                Err(e) if matches!(&e, CoreError::Provider { status: None, .. }) => {
                    unmatched += 1;
                    ctx.emit(Level::Warn, format!("AniList search failed for {folder}: {}", message_of(&e)), EventBody::Notice);
                }
                // A storage or an internal failure is the core's own, not
                // the provider's, and it is not this series' fault either.
                Err(e) => return Err(e),
            }
        }

        // The match brought covers, banners and portraits in; the sweep is
        // what keeps the directory from only ever growing. Its report is
        // bookkeeping, so it goes to the trace log rather than the
        // activity log.
        let cache = owner.images.clone();
        let now = time::now_secs();
        match owner.store.write_async(move |c| cache.sweep(c, now)).await {
            Ok(report) => tracing::debug!(
                "image sweep after auto-match: {} rows removed, {} evicted, {} files removed",
                report.removed_rows,
                report.evicted,
                report.removed_files
            ),
            Err(e) => tracing::debug!("the image sweep after auto-match failed: {e}"),
        }

        Ok(Finished {
            level: Level::Info,
            message: format!("auto-match: {matched} matched, {unmatched} unmatched, {backfilled} backfilled"),
            body: EventBody::AutoMatchFinished { backfilled, matched, unmatched },
        })
    })
}

/// Takes a series' identity away again. `attempted_at` deliberately stays:
/// the user cleared the match because it was wrong, and the same search
/// would only find the same wrong answer, so the series stays out of the
/// auto-match's way until the manual picker gives it one.
pub fn clear_match(core: &Core, series: u64) -> Result<Reply, CoreError> {
    let changed = core.store.write(move |c| {
        Ok(c.execute(
            "UPDATE series SET provider = NULL, anilist_id = NULL, mal_id = NULL, tmdb_id = NULL, tmdb_kind = NULL,
                    confirmed = 0, matched_at = NULL
             WHERE id = ?1",
            params![series as i64],
        )?)
    })?;
    if changed == 0 {
        return Err(CoreError::NotFound { what: Entity::Series, id: series });
    }
    let images_dir = core.paths.images_dir();
    let cards = core.store.read(|c| cards::cards_for(c, &images_dir, &[series]))?;
    let title = cards.first().map_or_else(String::new, |c| c.title.clone());
    core.bus.debug(Stage::Metadata, format!("match cleared: {title}"), EventBody::SeriesChanged { series: cards });
    Ok(Reply::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::anilist::{CoverImage, Title};

    fn media(id: u64, romaji: &str, cover: bool) -> Media {
        Media {
            id,
            title: Title { romaji: Some(romaji.to_string()), ..Default::default() },
            cover_image: cover.then(|| CoverImage { large: None, extra_large: Some(format!("https://img/{id}.jpg")) }),
            ..Default::default()
        }
    }

    /// The query is four tokens, so a candidate sharing two of its four
    /// scores 0.5, three of four scores 0.75, and one of five scores 0.4.
    #[test]
    fn pick_takes_the_best_over_the_threshold_and_needs_a_cover() {
        let query = "one two three four";
        // 0.4, 0.75 with no cover, 0.5 with one.
        let results = vec![media(1, "one alpha beta gamma", true), media(2, "one two three zzz", false), media(3, "one two yy zz", true)];
        // Electron's order: the best-scoring candidate is chosen first and
        // then has to have a cover, so the coverless winner is not replaced
        // by the runner-up.
        assert!(pick(query, &results).is_none());

        // With a cover on it, that same best candidate wins.
        let mut with_cover = results.clone();
        with_cover[1].cover_image = Some(CoverImage { large: Some("https://img/2.jpg".into()), extra_large: None });
        let (m, score) = pick(query, &with_cover).unwrap();
        assert_eq!(m.id, 2);
        assert!((score - 0.75).abs() < 1e-9, "{score}");

        // A tie keeps the earlier result: strict `>` never displaces it.
        let tie = vec![media(4, "one two yy zz", true), media(5, "one two aa bb", true)];
        assert_eq!(pick(query, &tie).unwrap().0.id, 4);

        // Nothing clears the bar.
        assert!(pick(query, &[media(6, "nothing like it", true)]).is_none());
        assert!(pick(query, &[]).is_none());
    }

    /// The synonyms are scored too: AniList's romaji for a series is often
    /// not what the folder is called.
    #[test]
    fn a_synonym_can_win_the_match() {
        let m = Media {
            id: 1,
            title: Title { romaji: Some("Sousou no Frieren".into()), ..Default::default() },
            synonyms: vec!["Frieren at the Funeral".into()],
            cover_image: Some(CoverImage { large: Some("https://img/1.jpg".into()), extra_large: None }),
            ..Default::default()
        };
        let (picked, score) = pick("Frieren at the Funeral", std::slice::from_ref(&m)).unwrap();
        assert_eq!(picked.id, 1);
        assert!((score - 1.0).abs() < 1e-9, "{score}");
    }
}
