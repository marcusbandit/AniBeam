//! "Everything behind an AniList id, written down." One routine, used by
//! every path that gives a series an identity: the auto-match, the manual
//! match, and the refresh.
//!
//! The shape is deliberate. All four provider calls happen first and the
//! whole record lands in one immediate transaction, so a series is never
//! half fetched: a failure halfway through leaves the row exactly as it
//! was rather than with recommendations and no relations. The images come
//! last and are waited for, so by the time the job reports the match the
//! record draws correctly with the network unplugged.

use rusqlite::{params, Transaction};

use crate::contract::{CoreError, Provider};
use crate::core::Core;
use crate::metadata::automatch::AUTO_MATCH_VERSION;
use crate::metadata::record;
use crate::net::anilist::{Enrichment, Media};

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
    let raw = record::raw_bundle(media_raw.as_ref(), Some(&enrichment_raw), Some(&schedule_raw), None);
    let urls = record::image_urls(&write);
    let row_mal_id = write.mal_id;

    core.store
        .tx_async(move |tx| {
            record::write_media(tx, &write, &raw, now)?;
            // The schedule was just fetched, so the airing refresh has
            // nothing left to owe this row either.
            tx.execute("UPDATE anilist_media SET airing_refreshed_at = ?2 WHERE id = ?1", params![as_i64(anilist_id), now])?;
            record::write_episodes(tx, anilist_id, &episodes, false, now)?;
            write_match_only(tx, series, Provider::Anilist, Some(anilist_id), row_mal_id, confirmed, now)
        })
        .await?;

    // Waited for rather than fired off: a card that reports a match and
    // then draws a blank poster for the next few seconds is worse than a
    // match that takes those seconds to report.
    core.images.ensure(&urls).await;
    Ok(())
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
        "UPDATE series SET provider = ?2, anilist_id = ?3, mal_id = ?4, tmdb_id = NULL, tmdb_kind = NULL,
                confirmed = ?5, matched_at = ?6, attempted_at = ?6, attempt_version = ?7
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

/// An id on its way to an INTEGER column, saturating rather than wrapping
/// an absurd value round to a negative one.
fn as_i64(v: u64) -> i64 {
    i64::try_from(v).unwrap_or(i64::MAX)
}
