//! The auto-match: the background job that gives an unmatched folder its
//! identity, and the call that takes one away again.
//!
//! The rules are Electron's `posterMatch.findShowMatch`, carried over one
//! for one and then walked across every provider in the user's order,
//! AniList then TVmaze by default. The folder name goes to the provider's
//! relevance-ordered search verbatim, the same list the manual picker
//! shows. Every hit is scored against its titles with tokenised Dice
//! similarity (AniList's romaji, english and native titles and synonyms,
//! TVmaze's name), the best score wins with ties breaking toward the
//! provider's own order, and the winner is accepted only if it clears the
//! threshold and has a poster to draw. The first provider whose winner is
//! accepted is the match; the next is only asked when this one missed.
//!
//! Three rules sit on top of the similarity, all learned from "Kimi no Na
//! wa" landing on a one-minute Suntory ad that lists the film's title as a
//! synonym. A synonym scores a little under a main title, so the film's own
//! title beats a short that merely cites it. A film folder leans toward
//! MOVIE candidates and away from MUSIC and SPECIAL ones, as a bonus and a
//! penalty on the score rather than a filter, so a film folder holding an
//! OVA with a unique title still finds it. And an AniList candidate that is
//! a short, under five minutes or format MUSIC, is never auto-matched at
//! all: it stays in the manual picker's list, where the user can still
//! choose it on purpose.
//!
//! One attempt per series, ever. A miss stamps `attempted_at` once every
//! provider has answered with a miss, so the next run walks past it rather
//! than asking the same questions again; the manual picker is the
//! recovery, and `AUTO_MATCH_VERSION` is how a change to these rules earns
//! every miss a second look. A match the user cleared is stamped too, with
//! no version beside it, so no version bump ever brings it back.

use std::collections::HashSet;
use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

use crate::contract::*;
use crate::core::Core;
use crate::images;
use crate::jobs::Finished;
use crate::library::cards;
use crate::library::scan::LibraryState;
use crate::metadata::apply::provider_stopped;
use crate::metadata::fetch::{self, message_of};
use crate::metadata::similarity::best_title_score;
use crate::net::anilist::Media;
use crate::net::tvmaze::Show;
use crate::prefs;
use crate::store::settings;
use crate::store::sql::placeholders;
use crate::sync::recover;
use crate::time;

/// Accept at this score or better. Electron's bar, and it is a "pretty
/// close" one: anything further off is left for the manual picker.
pub const THRESHOLD: f64 = 0.5;

/// What a match on a synonym alone gives up against a match on the romaji,
/// English or native title. Small: a folder named by a synonym still
/// clears the threshold on it; it only loses to an entry that carries the
/// same words as its own title.
pub const SYNONYM_DISCOUNT: f64 = 0.05;

/// What a MOVIE candidate gains when the folder is a film.
pub const MOVIE_BONUS: f64 = 0.10;

/// What a MUSIC or SPECIAL candidate loses when the folder is a film.
/// Sized so an exact-title ad short loses to the film with the same title
/// and so a special with a unique title, exact at 1.0, still clears the
/// threshold from a film folder.
pub const SHORT_FORMAT_PENALTY: f64 = 0.20;

/// An AniList entry shorter than this many minutes is a short, and a short
/// never auto-matches: a folder with a film or a season in it is not a
/// one-minute ad, whatever the ad calls itself.
pub const MIN_DURATION: u32 = 5;

/// Bumped when the matcher's rules change. A stored version below this one
/// clears every failed attempt stamped under an older version so the whole
/// library is tried again under the new rules; a match already made, and a
/// match the user cleared, are never touched. Version 2 is the
/// provider order: every series AniList alone missed gets its TVmaze turn.
pub const AUTO_MATCH_VERSION: u32 = 2;

/// How many hits the search asks for: Electron's ten, the same page the
/// manual picker shows.
const RESULTS: u32 = 10;

/// An AniList entry the auto-match never picks: a known duration under
/// `MIN_DURATION`, or format MUSIC. The manual picker still lists it.
pub fn is_short(m: &Media) -> bool {
    m.duration.is_some_and(|d| d < MIN_DURATION) || m.format.as_deref() == Some("MUSIC")
}

/// One candidate's score for a folder of this kind: the best Dice over its
/// main titles, or over its synonyms less `SYNONYM_DISCOUNT`, whichever is
/// higher, then the format lean for a film folder. The lean is an
/// adjustment and not a cap, so a film with its exact title scores above
/// 1.0; the log prints what the pick saw.
pub fn score(query: &str, kind: SeriesKind, m: &Media) -> f64 {
    let main = best_title_score(
        query,
        &[
            m.title.romaji.as_deref(),
            m.title.english.as_deref(),
            m.title.native.as_deref(),
        ],
    );
    let synonyms: Vec<Option<&str>> = m.synonyms.iter().map(|s| Some(s.as_str())).collect();
    let synonym = best_title_score(query, &synonyms);
    let synonym = if synonym > 0.0 {
        synonym - SYNONYM_DISCOUNT
    } else {
        0.0
    };
    let mut score = main.max(synonym);
    if kind == SeriesKind::Movie {
        match m.format.as_deref() {
            Some("MOVIE") => score += MOVIE_BONUS,
            Some("MUSIC") | Some("SPECIAL") => score -= SHORT_FORMAT_PENALTY,
            _ => {}
        }
    }
    score
}

/// The best-scoring candidate and its score, or nothing.
///
/// Shorts are left out of the field before scoring, which is the one place
/// a candidate is filtered rather than scored: a short is never the right
/// answer for a folder, so leaving it in could only ever hide the right
/// one. The cover, by contrast, is a gate on the winner rather than a
/// filter over the field, which is Electron's order: the best-scoring
/// entry is the answer, and an answer with no poster to draw is refused
/// rather than quietly replaced by a worse-scoring one.
pub fn pick<'a>(query: &str, kind: SeriesKind, results: &'a [Media]) -> Option<(&'a Media, f64)> {
    let mut best: Option<(&Media, f64)> = None;
    for r in results.iter().filter(|r| !is_short(r)) {
        let score = score(query, kind, r);
        // Strict, so a tie keeps the earlier and more relevant result.
        if best.is_none_or(|(_, b)| score > b) {
            best = Some((r, score));
        }
    }
    let (m, score) = best?;
    if score < THRESHOLD {
        return None;
    }
    let cover = m
        .cover_image
        .as_ref()
        .and_then(|c| c.extra_large.clone().or_else(|| c.large.clone()));
    cover.map(|_| (m, score))
}

/// `pick` over TVmaze's shows: the same rule, scored on the name alone,
/// and the same poster gate on the winner.
pub fn pick_show<'a>(query: &str, results: &'a [Show]) -> Option<(&'a Show, f64)> {
    let mut best: Option<(&Show, f64)> = None;
    for r in results {
        let score = best_title_score(query, &[r.name.as_deref()]);
        if best.is_none_or(|(_, b)| score > b) {
            best = Some((r, score));
        }
    }
    let (show, score) = best?;
    if score < THRESHOLD {
        return None;
    }
    show.poster_url().map(|_| (show, score))
}

/// The title a log line names the match by.
fn matched_title(m: &Media) -> String {
    m.title
        .romaji
        .clone()
        .or_else(|| m.title.english.clone())
        .unwrap_or_else(|| "?".to_string())
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

/// The series with a settle timer still armed. Their folders are still
/// being copied into, so a match now would be a match against a half
/// arrived folder; the timer's own firing is what brings them here.
fn armed(core: &Core) -> Vec<u64> {
    recover(&core.library.settle).keys().copied().collect()
}

/// The next series worth a search: never matched, never attempted, its
/// files still on disk, and not one of `skip`. Ordered by id, so the run
/// is deterministic and the caller can walk it one at a time. The kind
/// comes along because a film folder scores its candidates differently.
fn next_candidate(
    conn: &Connection,
    skip: &[u64],
) -> Result<Option<(u64, String, SeriesKind)>, CoreError> {
    let mut sql = String::from(
        "SELECT id, folder_name, kind FROM series
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
            let kind: String = r.get(2)?;
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, String>(1)?,
                SeriesKind::from_column(&kind).unwrap_or(SeriesKind::Show),
            ))
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

/// What one candidate's turn produced: the provider, the score and the
/// title, or nothing at all because no hit cleared the bar.
struct Hit {
    provider: Provider,
    score: f64,
    title: String,
}

/// One series: each provider in the order, until one of them matches.
/// Any error ends the walk for this series: handing the series on to the
/// next provider would let it claim a series the first one may well have
/// matched, so TVmaze never grabs an anime during a brief AniList outage.
/// Only `Ok(None)` means every provider answered with a miss.
async fn attempt(
    core: &Core,
    series: u64,
    folder: &str,
    kind: SeriesKind,
    order: &[Provider],
) -> Result<Option<Hit>, (Provider, CoreError)> {
    for &provider in order {
        let outcome = match provider {
            Provider::Tvmaze => attempt_tvmaze(core, series, folder).await,
            _ => attempt_anilist(core, series, folder, kind).await,
        };
        match outcome {
            Ok(Some(hit)) => return Ok(Some(hit)),
            Ok(None) => continue,
            Err(e) => return Err((provider, e)),
        }
    }
    Ok(None)
}

/// AniList's turn: the search, the pick, and, on a hit, the whole record.
async fn attempt_anilist(
    core: &Core,
    series: u64,
    folder: &str,
    kind: SeriesKind,
) -> Result<Option<Hit>, CoreError> {
    let results = core.anilist.search(folder, RESULTS).await?;
    let Some((media, score)) = pick(folder, kind, &results) else {
        return Ok(None);
    };
    let title = matched_title(media);
    let anilist_id = media.id;
    let media = media.clone();
    // An auto-match is never confirmed: the user has not seen it, and only
    // their own pick or an import earns that flag.
    fetch::fetch_and_write(
        core,
        series,
        anilist_id,
        Some(media),
        false,
        time::now_secs(),
    )
    .await?;
    Ok(Some(Hit {
        provider: Provider::Anilist,
        score,
        title,
    }))
}

/// TVmaze's turn. The search hit carries no episodes or images, so a hit
/// costs one more request for the show itself.
async fn attempt_tvmaze(core: &Core, series: u64, folder: &str) -> Result<Option<Hit>, CoreError> {
    let results = core.tvmaze.search(folder).await?;
    let Some((show, score)) = pick_show(folder, &results) else {
        return Ok(None);
    };
    let title = show.name.clone().unwrap_or_else(|| "?".to_string());
    fetch::fetch_and_write_tvmaze(core, series, show.id, false, time::now_secs()).await?;
    Ok(Some(Hit {
        provider: Provider::Tvmaze,
        score,
        title,
    }))
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
                // A row with an attempt and no version is a match the user
                // cleared: it stays out, whatever the version.
                let cleared = tx.execute(
                    "UPDATE series SET attempted_at = NULL, attempt_version = NULL
                     WHERE provider IS NULL AND attempted_at IS NOT NULL
                       AND attempt_version IS NOT NULL AND attempt_version < ?1",
                    params![i64::from(AUTO_MATCH_VERSION)],
                )?;
                settings::set(tx, settings::AUTO_MATCH_VERSION, &AUTO_MATCH_VERSION)?;
                Ok(cleared as u64)
            })
            .await?;
        if reset > 0 {
            ctx.emit(Level::Info, format!("Auto-match v{AUTO_MATCH_VERSION}: re-attempting {reset} unmatched series"), EventBody::Notice);
        }
        let order = owner.store.write_async(|c| prefs::load_provider_order(c)).await?;

        // 3. The loop, one series at a time. `seen` keeps a series this run
        //    has already had a turn at from coming back round: a miss is
        //    stamped, but a turn a provider's error ended is left unstamped
        //    for the next run, and would otherwise be asked again at once.
        let (mut matched, mut unmatched) = (0u64, 0u64);
        let mut seen: HashSet<u64> = HashSet::new();
        let images_dir = owner.paths.images_dir();
        loop {
            ctx.checkpoint()?;
            let mut skip = armed(&owner);
            skip.extend(seen.iter().copied());
            skip.sort_unstable();
            skip.dedup();
            let Some((series, folder, kind)) = owner.store.write_async(move |c| next_candidate(c, &skip)).await? else { break };
            seen.insert(series);
            ctx.progress(matched + unmatched, None, &folder);

            let outcome = {
                let _in_flight = InFlight::begin(&owner.library, series);
                attempt(&owner, series, &folder, kind, &order).await
            };
            match outcome {
                Ok(Some(hit)) => {
                    matched += 1;
                    let score = hit.score;
                    let title = hit.title;
                    let label = hit.provider.label();
                    ctx.say(Level::Info, format!("Matched {folder} to {title}."), format!("Match ({label} {score:.2}): {folder} -> {title}"), EventBody::Notice);
                    let dir = images_dir.clone();
                    let cards = owner.store.write_async(move |c| cards::cards_for(c, &dir, &[series])).await?;
                    if let Some(card) = cards.into_iter().next() {
                        ctx.changed(card);
                    }
                }
                Ok(None) => {
                    unmatched += 1;
                    stamp_attempt(&owner, series, time::now_secs()).await?;
                    ctx.emit(Level::Info, format!("No match for {folder} (threshold {THRESHOLD})"), EventBody::Notice);
                }
                // A rate limit the limiter could not ride out, or nothing
                // answering at all, is the provider saying stop rather
                // than this series failing. Nothing is stamped and the job
                // ends: stamping here would cost every series left its one
                // attempt for a reason that had nothing to do with them,
                // and one outage is one state change.
                Err((_, e)) if provider_stopped(&e) => return Err(e),
                // The provider answered with an error, or with something
                // that did not parse. That is this series' own reply, so
                // the run moves on rather than failing: failing the job
                // would wedge the library behind it, since the same series
                // comes first again next run. It is not stamped, though:
                // only every provider answering with a miss is a miss, and
                // any provider after this one never had its turn. The next
                // run asks again; `seen` keeps it out for the rest of this
                // one.
                Err((provider, e))
                    if matches!(&e, CoreError::Provider { status: Some(_), .. }) || e.is_decode_failure() =>
                {
                    unmatched += 1;
                    ctx.emit(Level::Warn, format!("{} search failed for {folder}: {}", provider.label(), message_of(&e)), EventBody::Notice);
                }
                // A storage or an internal failure is the core's own, not
                // the provider's, and it is not this series' fault either.
                Err((_, e)) => return Err(e),
            }
        }

        images::sweep_after(&owner, "auto-match").await;

        let said = match (matched, unmatched) {
            (0, 0) => None,
            (m, 0) => Some(format!("Matched {m} {}.", if m == 1 { "show" } else { "shows" })),
            (0, u) => Some(format!("{u} {} a match.", if u == 1 { "show still needs" } else { "shows still need" })),
            (m, u) => Some(format!(
                "Matched {m} {}; {u} still {} a match.",
                if m == 1 { "show" } else { "shows" },
                if u == 1 { "needs" } else { "need" }
            )),
        };
        Ok(Finished {
            plain: said,
            level: Level::Info,
            message: format!("Auto-match: {matched} matched, {unmatched} unmatched, {backfilled} backfilled"),
            body: EventBody::AutoMatchFinished { backfilled, matched, unmatched },
        })
    })
}

/// Takes a series' identity away again, and keeps the auto-match off it
/// for good: the user cleared the match because it was wrong, and the same
/// search would only find the same wrong answer, so the series stays out
/// of the auto-match's way until the manual picker gives it one. That is
/// an attempt with no version beside it, which the version guard never
/// resets, unlike a miss, which always carries the version that made it.
pub fn clear_match(core: &Core, series: u64) -> Result<Reply, CoreError> {
    let now = time::now_secs();
    let changed = core.store.write(move |c| {
        Ok(c.execute(
            "UPDATE series SET provider = NULL, anilist_id = NULL, mal_id = NULL, tmdb_id = NULL, tmdb_kind = NULL,
                    tvmaze_id = NULL, confirmed = 0, matched_at = NULL,
                    attempted_at = COALESCE(attempted_at, ?2), attempt_version = NULL, episodes_set = NULL
             WHERE id = ?1",
            params![series as i64, now],
        )?)
    })?;
    if changed == 0 {
        return Err(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        });
    }
    let images_dir = core.paths.images_dir();
    let cards = core
        .store
        .read(|c| cards::cards_for(c, &images_dir, &[series]))?;
    let title = cards.first().map_or_else(String::new, |c| c.title.clone());
    core.bus.debug(
        Stage::Metadata,
        format!("match cleared: {title}"),
        EventBody::SeriesChanged { series: cards },
    );
    // No record, no record rule: specials it read as episodes read as specials again.
    crate::library::readings::reread_if_record_matters(core, series);
    Ok(Reply::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::anilist::{CoverImage, Title};

    fn media(id: u64, romaji: &str, cover: bool) -> Media {
        Media {
            id,
            title: Title {
                romaji: Some(romaji.to_string()),
                ..Default::default()
            },
            cover_image: cover.then(|| CoverImage {
                large: None,
                extra_large: Some(format!("https://img/{id}.jpg")),
            }),
            ..Default::default()
        }
    }

    /// The query is four tokens, so a candidate sharing two of its four
    /// scores 0.5, three of four scores 0.75, and one of five scores 0.4.
    #[test]
    fn pick_takes_the_best_over_the_threshold_and_needs_a_cover() {
        let query = "one two three four";
        // 0.4, 0.75 with no cover, 0.5 with one.
        let results = vec![
            media(1, "one alpha beta gamma", true),
            media(2, "one two three zzz", false),
            media(3, "one two yy zz", true),
        ];
        // Electron's order: the best-scoring candidate is chosen first and
        // then has to have a cover, so the coverless winner is not replaced
        // by the runner-up.
        assert!(pick(query, SeriesKind::Show, &results).is_none());

        // With a cover on it, that same best candidate wins.
        let mut with_cover = results.clone();
        with_cover[1].cover_image = Some(CoverImage {
            large: Some("https://img/2.jpg".into()),
            extra_large: None,
        });
        let (m, score) = pick(query, SeriesKind::Show, &with_cover).unwrap();
        assert_eq!(m.id, 2);
        assert!((score - 0.75).abs() < 1e-9, "{score}");

        // A tie keeps the earlier result: strict `>` never displaces it.
        let tie = vec![
            media(4, "one two yy zz", true),
            media(5, "one two aa bb", true),
        ];
        assert_eq!(pick(query, SeriesKind::Show, &tie).unwrap().0.id, 4);

        // Nothing clears the bar.
        assert!(
            pick(
                query,
                SeriesKind::Show,
                &[media(6, "nothing like it", true)]
            )
            .is_none()
        );
        assert!(pick(query, SeriesKind::Show, &[]).is_none());
    }

    /// The synonyms are scored too: AniList's romaji for a series is often
    /// not what the folder is called. A synonym match carries the small
    /// discount and still clears the bar on its own.
    #[test]
    fn a_synonym_can_win_the_match() {
        let m = Media {
            id: 1,
            title: Title {
                romaji: Some("Sousou no Frieren".into()),
                ..Default::default()
            },
            synonyms: vec!["Frieren at the Funeral".into()],
            cover_image: Some(CoverImage {
                large: Some("https://img/1.jpg".into()),
                extra_large: None,
            }),
            ..Default::default()
        };
        let (picked, score) = pick(
            "Frieren at the Funeral",
            SeriesKind::Show,
            std::slice::from_ref(&m),
        )
        .unwrap();
        assert_eq!(picked.id, 1);
        assert!((score - (1.0 - SYNONYM_DISCOUNT)).abs() < 1e-9, "{score}");
    }

    /// The real case behind the rules: AniList 21519 "Kimi no Na wa." the
    /// film, and 97962 "Suntory Minami Alps no Tennen Mizu x Kimi no Na
    /// wa", the one-minute ad short that lists the film's title among its
    /// synonyms. The search returns the short first.
    fn kimi_no_na_wa() -> Media {
        let mut m = media(21519, "Kimi no Na wa.", true);
        m.title.english = Some("Your Name.".into());
        m.format = Some("MOVIE".into());
        m.duration = Some(106);
        m.episodes = Some(1);
        m
    }

    fn suntory_short(format: &str, duration: Option<u32>) -> Media {
        let mut m = media(
            97962,
            "Suntory Minami Alps no Tennen Mizu x Kimi no Na wa",
            true,
        );
        m.synonyms = vec!["Kimi no Na wa".into()];
        m.format = Some(format.into());
        m.duration = duration;
        m.episodes = Some(1);
        m
    }

    /// Rule 3: the film's own title beats a short that lists the film's
    /// title as a synonym, and the synonym match itself scores a little
    /// under the title match. Kind Show here, so no format lean is in play
    /// and only the synonym discount separates them.
    #[test]
    fn kimi_no_na_wa_beats_the_suntory_short_on_its_synonym() {
        let short = suntory_short("ONA", None);
        let film = kimi_no_na_wa();
        let synonym = score("Kimi no Na wa", SeriesKind::Show, &short);
        let title = score("Kimi no Na wa", SeriesKind::Show, &film);
        assert!(
            (synonym - (1.0 - SYNONYM_DISCOUNT)).abs() < 1e-9,
            "{synonym}"
        );
        assert!((title - 1.0).abs() < 1e-9, "{title}");
        let results = vec![short, film];
        assert_eq!(
            pick("Kimi no Na wa", SeriesKind::Show, &results)
                .unwrap()
                .0
                .id,
            21519
        );
    }

    /// Rule 1: a film folder leans toward the MOVIE and away from the
    /// SPECIAL. The short's duration is unknown here so the gate does not
    /// remove it, and the lean alone has to decide.
    #[test]
    fn a_film_folder_prefers_the_film_over_the_suntory_special() {
        let short = suntory_short("SPECIAL", None);
        let film = kimi_no_na_wa();
        let penalised = score("Kimi no Na wa", SeriesKind::Movie, &short);
        let boosted = score("Kimi no Na wa", SeriesKind::Movie, &film);
        assert!(
            (penalised - (1.0 - SYNONYM_DISCOUNT - SHORT_FORMAT_PENALTY)).abs() < 1e-9,
            "{penalised}"
        );
        assert!((boosted - (1.0 + MOVIE_BONUS)).abs() < 1e-9, "{boosted}");
        let results = vec![short, film];
        let (m, s) = pick("Kimi no Na wa", SeriesKind::Movie, &results).unwrap();
        assert_eq!(m.id, 21519);
        assert!((s - boosted).abs() < 1e-9, "{s}");
        // A show folder gets no lean at all.
        let plain = score("Kimi no Na wa", SeriesKind::Show, &results[1]);
        assert!((plain - 1.0).abs() < 1e-9, "{plain}");
    }

    /// Rule 1's other half: the penalty is not a filter. A film folder
    /// holding an OVA or a special with a unique title still matches it.
    #[test]
    fn a_film_folder_still_matches_an_ova_or_a_special_with_a_unique_title() {
        let mut ova = media(1, "Kara no Kyoukai: Mirai Fukuin", true);
        ova.format = Some("OVA".into());
        ova.duration = Some(88);
        let (m, s) = pick(
            "Kara no Kyoukai Mirai Fukuin",
            SeriesKind::Movie,
            std::slice::from_ref(&ova),
        )
        .unwrap();
        assert_eq!(m.id, 1);
        assert!((s - 1.0).abs() < 1e-9, "{s}");

        let mut special = media(2, "Gintama: Shiroyasha Koutan", true);
        special.format = Some("SPECIAL".into());
        special.duration = Some(10);
        let (m, s) = pick(
            "Gintama Shiroyasha Koutan",
            SeriesKind::Movie,
            std::slice::from_ref(&special),
        )
        .unwrap();
        assert_eq!(m.id, 2);
        assert!((s - (1.0 - SHORT_FORMAT_PENALTY)).abs() < 1e-9, "{s}");
        assert!(s >= THRESHOLD);
    }

    /// Rule 2: a short never auto-matches. The Suntory ad as AniList really
    /// lists it, one minute long, is out of the field for a film folder and
    /// a show folder alike, and so is anything of format MUSIC, whatever
    /// its length. With the short out, the film behind it is the pick.
    #[test]
    fn the_suntory_short_never_auto_matches() {
        let one_minute = suntory_short("SPECIAL", Some(1));
        assert!(is_short(&one_minute));
        assert!(
            pick(
                "Kimi no Na wa",
                SeriesKind::Movie,
                std::slice::from_ref(&one_minute)
            )
            .is_none()
        );
        assert!(
            pick(
                "Kimi no Na wa",
                SeriesKind::Show,
                std::slice::from_ref(&one_minute)
            )
            .is_none()
        );

        let mut music = media(3, "Kimi no Na wa", true);
        music.format = Some("MUSIC".into());
        music.duration = Some(30);
        assert!(is_short(&music));
        assert!(
            pick(
                "Kimi no Na wa",
                SeriesKind::Show,
                std::slice::from_ref(&music)
            )
            .is_none()
        );

        // Five minutes is not a short; an unknown duration is not either.
        let mut five = media(4, "Kimi no Na wa", true);
        five.duration = Some(5);
        assert!(!is_short(&five));
        assert!(!is_short(&suntory_short("ONA", None)));

        let results = vec![one_minute, kimi_no_na_wa()];
        assert_eq!(
            pick("Kimi no Na wa", SeriesKind::Movie, &results)
                .unwrap()
                .0
                .id,
            21519
        );
    }
}
