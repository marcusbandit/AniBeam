//! "Everything behind an AniList id, written down." One routine, used by
//! every path that gives a series an identity: the auto-match, the manual
//! match, and the refresh. Its TVmaze twin sits below it and keeps the same
//! promises with one request instead of four.
//!
//! The shape is deliberate. All four provider calls happen first and the
//! whole record lands in one immediate transaction, so a series is never
//! half fetched: a failure halfway through leaves the row exactly as it
//! was rather than with recommendations and no relations. The images come
//! last and are waited for, so by the time the job reports the match the
//! record draws correctly with the network unplugged.

use rusqlite::{Transaction, params};

use crate::contract::{CoreError, Provider};
use crate::core::Core;
use crate::metadata::automatch::AUTO_MATCH_VERSION;
use crate::metadata::record;
use crate::net::anilist::{Enrichment, Media};
use crate::net::tvmaze::{self, Show};
use crate::store::sql::as_i64;

/// The whole record behind `anilist_id`, written against `series`.
///
/// `media` is the search reply when the caller already has it, which the
/// auto-match does: AniList's search answers with everything
/// `MEDIA_BY_ID_QUERY` would, so asking again would be a wasted request
/// against an 800 ms gap. The cost is that the `media` half of the `raw`
/// bundle is null in that case, because `Media` is a deserialise-only
/// struct and there is no JSON of it to keep; the enrichment and the
/// schedule halves are always real.
///
/// A Jikan failure is not a failure of the match: AniList carries no
/// episode titles and Jikan is the side-fetch that fills them in, so an
/// outage costs the series its titles and nothing else. It goes through
/// the core's outage window, which warns at most once every ten minutes.
///
/// The enrichment is the exception. `write_media` replaces the relations
/// and the recommendations unconditionally, so writing a build that has no
/// enrichment would empty both tables for this series; an enrichment that
/// cannot be had fails the whole call instead.
pub async fn fetch_and_write(
    core: &Core,
    series: u64,
    anilist_id: u64,
    media: Option<Media>,
    confirmed: bool,
    now: i64,
) -> Result<(), CoreError> {
    let (media, media_raw) = match media {
        Some(media) => (media, None),
        None => {
            let (media, raw) = core.anilist.media_by_id_raw(anilist_id).await?;
            (media.ok_or_else(|| no_such_media(anilist_id))?, Some(raw))
        }
    };
    let (enrichment, enrichment_raw) = core.anilist.enrichment_raw(anilist_id).await?;
    let enrichment: Enrichment = enrichment.ok_or_else(|| no_such_media(anilist_id))?;
    let (schedule, schedule_raw) = core.anilist.schedule_raw(anilist_id).await?;

    let mal_id = media.id_mal.or(enrichment.id_mal);
    let jikan = match mal_id {
        Some(mal_id) => match core.jikan.episodes(mal_id).await {
            Ok(episodes) => episodes,
            Err(e) => {
                core.report_jikan_outage(&message_of(&e));
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    let write = record::build(&media, Some(&enrichment));
    let titles = record::streaming_titles(&enrichment.streaming_episodes);
    let episodes = record::merge_episodes(Some(&schedule), &titles, &jikan);
    let raw = record::raw_bundle(
        media_raw.as_ref(),
        Some(&enrichment_raw),
        Some(&schedule_raw),
        None,
    );
    let urls = record::image_urls(&write);
    let row_mal_id = write.mal_id;

    core.store
        .tx_async(move |tx| {
            record::write_media(tx, &write, &raw, now)?;
            // The schedule was just fetched, so the airing refresh has
            // nothing left to owe this row either.
            tx.execute(
                "UPDATE anilist_media SET airing_refreshed_at = ?2 WHERE id = ?1",
                params![as_i64(anilist_id), now],
            )?;
            record::write_episodes(tx, anilist_id, &episodes, false, now)?;
            write_match_only(
                tx,
                series,
                Provider::Anilist,
                Some(anilist_id),
                row_mal_id,
                confirmed,
                now,
            )
        })
        .await?;

    // Waited for rather than fired off: a card that reports a match and
    // then draws a blank poster for the next few seconds is worse than a
    // match that takes those seconds to report.
    core.images.ensure(&urls).await;
    reread_after_record(core, series).await;
    Ok(())
}

/// A record of specials or OVAs reads a series' numbered specials as its
/// episodes, and any other record, or none, reads them as specials (#167).
/// When the series' rows disagree with the record it has now, it is scanned.
pub(crate) async fn reread_after_record(core: &Core, series: u64) {
    let reread = core
        .store
        .write_async(move |c| crate::library::readings::record_matters(c, series))
        .await
        .unwrap_or(false);
    if reread {
        // A source that is away right now gets its scan when it is back.
        let _ = crate::library::scan::rescan_series(core, series);
    }
}

/// The series' match columns and nothing else: what `ApplyMatch` writes
/// before it goes and fetches, and the whole of a MAL-only match, which
/// has no media row to write.
///
/// `attempted_at` is stamped along with `matched_at`, so a match that is
/// later cleared does not put the series straight back in front of the
/// auto-match: the user cleared it because the answer was wrong, and the
/// same search would find the same wrong answer again.
pub fn write_match_only(
    tx: &Transaction,
    series: u64,
    provider: Provider,
    anilist_id: Option<u64>,
    mal_id: Option<u64>,
    confirmed: bool,
    now: i64,
) -> Result<(), CoreError> {
    tx.execute(
        "UPDATE series SET provider = ?2, anilist_id = ?3, mal_id = ?4, tmdb_id = NULL, tmdb_kind = NULL, tvmaze_id = NULL,
                confirmed = ?5, matched_at = ?6, attempted_at = ?6, attempt_version = ?7,
                episodes_set = CASE WHEN anilist_id IS ?3 AND mal_id IS ?4 THEN episodes_set END
         WHERE id = ?1",
        params![
            as_i64(series),
            provider.as_str(),
            anilist_id.map(as_i64),
            mal_id.map(as_i64),
            i64::from(confirmed),
            now,
            i64::from(AUTO_MATCH_VERSION),
        ],
    )?;
    Ok(())
}

// TVmaze ----------------------------------------------------------------------

/// The whole record behind a TVmaze show id, written against `series`: the
/// show, its numbered episodes, the match columns, and the poster and the
/// background waited for. One fetch with the episodes and the images
/// embedded, so there is no half-fetched shape to guard against; the show
/// and the match land in one transaction all the same.
pub async fn fetch_and_write_tvmaze(
    core: &Core,
    series: u64,
    show_id: u64,
    confirmed: bool,
    now: i64,
) -> Result<(), CoreError> {
    let show = fetch_show(core, show_id).await?;
    let urls = show_image_urls(&show);
    core.store
        .tx_async(move |tx| {
            write_show(tx, &show, now)?;
            write_tvmaze_match(tx, series, show_id, confirmed, now)
        })
        .await?;
    core.images.ensure(&urls).await;
    reread_after_record(core, series).await;
    Ok(())
}

/// The show behind an id, or the provider's 404 when TVmaze has none.
pub async fn fetch_show(core: &Core, show_id: u64) -> Result<Show, CoreError> {
    core.tvmaze
        .show(show_id)
        .await?
        .ok_or_else(|| no_such_show(show_id))
}

/// The two pictures a TVmaze series draws: the poster and the banner.
pub fn show_image_urls(show: &Show) -> Vec<String> {
    [show.poster_url(), show.background_url()]
        .into_iter()
        .flatten()
        .collect()
}

/// The show row and its episodes, replaced whole. TVmaze answers with the
/// complete episode list every time, so the old rows go rather than being
/// merged: an episode TVmaze renumbered must not linger under its old key.
pub fn write_show(tx: &Transaction, show: &Show, now: i64) -> Result<(), CoreError> {
    let genres = serde_json::to_string(&show.genres)?;
    let summary = show
        .summary
        .as_deref()
        .map(tvmaze::plain_summary)
        .filter(|s| !s.is_empty());
    tx.execute(
        "INSERT INTO tvmaze_shows (id, name, url, type, language, genres, status, premiered, ended, average_runtime,
                                   rating, network, summary, poster_url, background_url, imdb, thetvdb, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name, url = excluded.url, type = excluded.type, language = excluded.language,
            genres = excluded.genres, status = excluded.status, premiered = excluded.premiered,
            ended = excluded.ended, average_runtime = excluded.average_runtime, rating = excluded.rating,
            network = excluded.network, summary = excluded.summary, poster_url = excluded.poster_url,
            background_url = excluded.background_url, imdb = excluded.imdb, thetvdb = excluded.thetvdb,
            fetched_at = excluded.fetched_at",
        params![
            as_i64(show.id),
            show.name,
            show.url,
            show.kind,
            show.language,
            genres,
            show.status,
            show.premiered,
            show.ended,
            show.average_runtime,
            show.rating.average,
            show.network_name(),
            summary,
            show.poster_url(),
            show.background_url(),
            show.externals.imdb,
            show.externals.thetvdb.map(as_i64),
            now,
        ],
    )?;
    tx.execute(
        "DELETE FROM tvmaze_episodes WHERE show_id = ?1",
        params![as_i64(show.id)],
    )?;
    let mut insert = tx.prepare(
        // TVmaze names an episode it knows nothing about yet "TBA", which is
        // a placeholder, not a title.
        "INSERT INTO tvmaze_episodes (show_id, season, number, name, airstamp, runtime) VALUES (?1, ?2, ?3, NULLIF(?4, 'TBA'), ?5, ?6)",
    )?;
    for e in show.episodes() {
        insert.execute(params![
            as_i64(show.id),
            e.season,
            e.number,
            e.name,
            e.airstamp,
            e.runtime
        ])?;
    }
    Ok(())
}

/// The match columns of a TVmaze match: the show id and nothing else. A
/// TVmaze series carries no AniList or MAL id, which is what keeps every
/// tracker write away from it. `attempted_at` is stamped for the same
/// reason `write_match_only` stamps it.
pub fn write_tvmaze_match(
    tx: &Transaction,
    series: u64,
    show_id: u64,
    confirmed: bool,
    now: i64,
) -> Result<(), CoreError> {
    tx.execute(
        "UPDATE series SET provider = 'tvmaze', anilist_id = NULL, mal_id = NULL, tmdb_id = NULL, tmdb_kind = NULL,
                tvmaze_id = ?2, confirmed = ?3, matched_at = ?4, attempted_at = ?4, attempt_version = ?5,
                episodes_set = CASE WHEN tvmaze_id IS ?2 THEN episodes_set END
         WHERE id = ?1",
        params![
            as_i64(series),
            as_i64(show_id),
            i64::from(confirmed),
            now,
            i64::from(AUTO_MATCH_VERSION),
        ],
    )?;
    Ok(())
}

/// TVmaze answered that there is no such show: an attempt that was made.
fn no_such_show(show_id: u64) -> CoreError {
    CoreError::Provider {
        provider: Provider::Tvmaze,
        status: Some(404),
        message: format!("TVmaze has no show {show_id}"),
        retry_after: None,
    }
}

/// AniList answered, and what it said is that there is no such media. That
/// is the provider's answer rather than a transport failure, so it carries
/// a status and the caller treats it as an attempt that was made.
fn no_such_media(anilist_id: u64) -> CoreError {
    CoreError::Provider {
        provider: Provider::Anilist,
        status: Some(404),
        message: format!("AniList has no media {anilist_id}"),
        retry_after: None,
    }
}

/// A provider error's own message, without the `Provider:` prefix Display
/// adds: the log lines are Electron's and they carry the bare message.
pub fn message_of(e: &CoreError) -> String {
    match e {
        CoreError::Provider { message, .. } => message.clone(),
        other => other.to_string(),
    }
}
