//! The five jobs a user starts by hand: search a provider, follow a
//! pasted link, apply a match, refresh one series, refresh the library.
//! Plus the stub backfill, which is a refresh the launch queues rather
//! than one anybody asks for.
//!
//! Everything here ends in `fetch::fetch_and_write`, the one routine that
//! turns an AniList id into a record, or its TVmaze twin
//! `fetch_and_write_tvmaze`. What this module adds is the order
//! around it: what is validated before a job is worth starting, what the
//! user is told while it runs, and which failures are the series' and
//! which are the provider's.

use std::sync::Arc;

use rusqlite::{OptionalExtension, params};

use crate::contract::*;
use crate::core::Core;
use crate::images;
use crate::jobs::Finished;
use crate::library::cards;
use crate::metadata::fetch::{self, message_of};
use crate::metadata::link::{self, Link};
use crate::metadata::record;
use crate::net::anilist::Media;
use crate::net::tvmaze::Show;
use crate::time;

/// What the modal says when the text was a URL the core knows the host of
/// but cannot follow: a themoviedb.org page, an AniList user profile, an
/// id with a typo in it.
const UNREADABLE: &str =
    "Couldn't read that link. Paste an AniList, MyAnimeList, TVmaze or IMDb page URL.";

/// AniList answered, and the answer was that it carries no entry for that
/// MyAnimeList id. Deliberate rather than silent: `ResolveLink` is how a
/// pasted link becomes a match, and a link that resolves to nothing has to
/// say so instead of quietly writing a MAL-only match nobody asked for.
const NO_MAL_ENTRY: &str = "AniList has no entry for that MyAnimeList id.";

/// The same for a pasted IMDb link: TVmaze has no show behind that title.
const NO_IMDB_SHOW: &str = "TVmaze has no show for that IMDb title.";

/// A shortest sensible query. One character matches most of AniList.
const MIN_QUERY: usize = 2;

/// The bounds a search's `perPage` is held to: the shell asks for what it
/// wants to draw, and AniList's page is 50.
const MAX_RESULTS: u32 = 50;

// The calls -----------------------------------------------------------------

/// Starts the Search job on one match provider: AniList or TVmaze. MAL is
/// a tracker rather than a matching provider and TMDB has no client at
/// all, so neither can be searched.
pub fn search(core: &Core, provider: Provider, query: &str, limit: u32) -> Result<u64, CoreError> {
    if !MATCH_PROVIDERS.contains(&provider) {
        return Err(CoreError::Unsupported {
            what: format!("search on {}", provider.as_str()),
        });
    }
    let query = query.trim().to_string();
    if query.chars().count() < MIN_QUERY {
        return Err(CoreError::invalid(
            "query",
            "Type at least two characters to search.",
        ));
    }
    let per_page = limit.clamp(1, MAX_RESULTS);
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::Search, move |_ctx| async move {
            let results: Vec<SearchResult> = match provider {
                // TVmaze has no page size; it answers with its best ten
                // or so, and the limit is applied here instead.
                Provider::Tvmaze => owner
                    .tvmaze
                    .search(&query)
                    .await?
                    .iter()
                    .take(per_page as usize)
                    .map(show_result)
                    .collect(),
                _ => owner
                    .anilist
                    .search(&query, per_page)
                    .await?
                    .iter()
                    .map(to_result)
                    .collect(),
            };
            let n = results.len();
            Ok(Finished {
                plain: None,
                level: Level::Debug,
                message: format!("Search: {n} {} results for \"{query}\"", provider.label()),
                body: EventBody::SearchFinished { results },
            })
        }))
}

/// What a readable link left the job to do: an AniList id is the answer
/// already, a MAL id is one lookup away from it.
enum Pasted {
    Anilist(u64),
    Mal(u64),
    Tvmaze(u64),
    Imdb(String),
}

/// Starts the ResolveLink job: a pasted URL turned into the target
/// `ApplyMatch` takes. An AniList or a TVmaze link is already that target
/// and needs no request; a MyAnimeList link goes through AniList's
/// `Media(idMal:)`, because a MAL id alone is not something a series page
/// can be drawn from, and an IMDb link goes through TVmaze's lookup.
pub fn resolve_link(core: &Core, url: &str) -> Result<u64, CoreError> {
    let pasted = match link::parse(url).ok_or_else(|| CoreError::invalid("url", "not a link"))? {
        Link::Anilist { id } => Pasted::Anilist(id),
        Link::Mal { id } => Pasted::Mal(id),
        Link::Tvmaze { id } => Pasted::Tvmaze(id),
        Link::Imdb { id } => Pasted::Imdb(id),
        Link::Unknown => return Err(CoreError::invalid("url", UNREADABLE)),
    };
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::ResolveLink, move |_ctx| async move {
            let target = match pasted {
                Pasted::Anilist(id) => MatchTarget::Anilist { id, season: None },
                Pasted::Mal(mal_id) => {
                    let id = owner.anilist.resolve_by_mal(mal_id).await?.ok_or_else(|| {
                        CoreError::Provider {
                            provider: Provider::Anilist,
                            status: Some(404),
                            message: NO_MAL_ENTRY.to_string(),
                            retry_after: None,
                        }
                    })?;
                    MatchTarget::Anilist { id, season: None }
                }
                Pasted::Tvmaze(id) => MatchTarget::Tvmaze { id },
                Pasted::Imdb(imdb) => {
                    let id = owner.tvmaze.lookup_imdb(&imdb).await?.ok_or_else(|| {
                        CoreError::Provider {
                            provider: Provider::Tvmaze,
                            status: Some(404),
                            message: NO_IMDB_SHOW.to_string(),
                            retry_after: None,
                        }
                    })?;
                    MatchTarget::Tvmaze { id }
                }
            };
            let named = match &target {
                MatchTarget::Tvmaze { id } => format!("TVmaze {id}"),
                MatchTarget::Anilist { id, .. } | MatchTarget::Mal { id } => {
                    format!("AniList {id}")
                }
            };
            Ok(Finished {
                plain: None,
                level: Level::Debug,
                message: format!("Link resolved to {named}"),
                body: EventBody::LinkResolved { target },
            })
        }))
}

/// Starts the ApplyMatch job: the user's own pick, so the match is written
/// confirmed and the auto-match never touches it again.
///
/// The `season` hint on an AniList target is accepted and changes nothing
/// stored: the titles come from the media row, and the folder already
/// carries its own season. Electron's ` (Season N)` display suffix leaves
/// with its `title` field.
pub fn apply_match(core: &Core, series: u64, target: MatchTarget) -> Result<u64, CoreError> {
    let folder = folder_name(core, series)?;
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::ApplyMatch, move |ctx| async move {
            // A MAL target is resolved first, because whether there is an
            // AniList id behind it decides which of the two matches is
            // written. A MAL id AniList has never heard of is still a match
            // the user asked for: it is written MAL-only, titled by the
            // folder, and left alone by the auto-match because its `provider`
            // is set.
            let (anilist_id, pasted_mal) = match target {
                MatchTarget::Anilist { id, season: _ } => (id, None),
                // TVmaze is one fetch, so the show comes first and the
                // match is written with it in one transaction: a show
                // TVmaze does not have, or a fetch that never lands, leaves
                // the series' old match exactly as it was.
                MatchTarget::Tvmaze { id } => {
                    fetch::fetch_and_write_tvmaze(&owner, series, id, true, time::now_secs())
                        .await?;
                    if let Some(card) = card_for(&owner, series).await? {
                        ctx.changed(card);
                    }
                    return Ok(Finished {
                        plain: Some(format!("Matched {folder}.")),
                        level: Level::Info,
                        message: format!("matched {folder} to TVmaze {id}"),
                        body: EventBody::MatchApplied { series },
                    });
                }
                MatchTarget::Mal { id } => match owner.anilist.resolve_by_mal(id).await? {
                    Some(anilist_id) => (anilist_id, Some(id)),
                    None => {
                        let now = time::now_secs();
                        owner
                            .store
                            .tx_async(move |tx| {
                                fetch::write_match_only(
                                    tx,
                                    series,
                                    Provider::Mal,
                                    None,
                                    Some(id),
                                    true,
                                    now,
                                )
                            })
                            .await?;
                        if let Some(card) = card_for(&owner, series).await? {
                            ctx.changed(card);
                        }
                        // A MAL-only match carries no record format.
                        fetch::reread_after_record(&owner, series).await;
                        return Ok(Finished {
                            plain: Some(format!("Matched {folder}.")),
                            level: Level::Info,
                            message: format!("Matched {folder} to MyAnimeList {id}"),
                            body: EventBody::MatchApplied { series },
                        });
                    }
                },
            };

            // The match columns first, in their own transaction, and reported
            // straight away rather than through the batch: the modal closes on
            // this card, so a quarter of a second of "unmatched" behind it is
            // exactly what the step exists to avoid.
            let now = time::now_secs();
            owner
                .store
                .tx_async(move |tx| {
                    // The series row's `anilist_id` is a foreign key, so the
                    // media row has to exist before it can point at one. A stub
                    // is the honest shape for it: the id is known and nothing
                    // else is yet. If the fetch below never lands, the backfill
                    // is exactly what comes back for it.
                    record::write_stub(
                        tx,
                        &record::StubWrite {
                            id: anilist_id,
                            mal_id: pasted_mal,
                            ..Default::default()
                        },
                    )?;
                    fetch::write_match_only(
                        tx,
                        series,
                        Provider::Anilist,
                        Some(anilist_id),
                        pasted_mal,
                        true,
                        now,
                    )
                })
                .await?;
            if let Some(card) = card_for(&owner, series).await? {
                ctx.emit(
                    Level::Debug,
                    "match applied",
                    EventBody::SeriesChanged { series: vec![card] },
                );
            }

            // Then the record behind the id, which is four provider calls and
            // the images.
            fetch::fetch_and_write(&owner, series, anilist_id, None, true, time::now_secs())
                .await?;
            // The row is AniList-keyed, so its `mal_id` is AniList's `idMal`.
            // When the reply carried none, the pasted id is the only MAL id
            // anybody knows and it stays.
            if let Some(mal_id) = pasted_mal {
                owner
                    .store
                    .write_async(move |c| {
                        c.execute(
                            "UPDATE series SET mal_id = ?2 WHERE id = ?1 AND mal_id IS NULL",
                            params![series as i64, mal_id as i64],
                        )?;
                        Ok(())
                    })
                    .await?;
            }
            if let Some(card) = card_for(&owner, series).await? {
                ctx.changed(card);
            }

            Ok(Finished {
                plain: Some(format!("Matched {folder}.")),
                level: Level::Info,
                message: format!("Matched {folder} to AniList {anilist_id}"),
                body: EventBody::MatchApplied { series },
            })
        }))
}

/// Starts the Refresh job for one series: the same fetch the match ran,
/// against the id the series already carries.
pub fn refresh_series(core: &Core, series: u64) -> Result<u64, CoreError> {
    let target = core.store.read(|c| refresh_target(c, series))?;
    let Some(target) = target else {
        return Err(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        });
    };
    // An unmatched series has nothing to refresh, and the Match button is
    // the recovery. A MAL-only or an imported TMDB match has no AniList id
    // to ask about, which is a different answer again.
    if target.provider.is_none() {
        return Err(CoreError::Refused {
            reason: Refusal::Unmatched,
        });
    }
    let Some(id) = target.id else {
        return Err(CoreError::Unsupported {
            what: "refresh of a MAL-only or TMDB match".to_string(),
        });
    };
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::Refresh, move |ctx| async move {
            let folder = target.folder;
            let (refreshed, failed) = match fetch_id(&owner, series, id, target.confirmed).await {
                Ok(()) => {
                    if let Some(card) = card_for(&owner, series).await? {
                        ctx.changed(card);
                    }
                    (1, 0)
                }
                // A rate limit the limiter could not ride out is the
                // provider saying stop, not this series failing.
                Err(e) if is_rate_limited(&e) => return Err(e),
                // Anything else leaves the match standing, so it is this
                // series' failure and not the job's.
                Err(e) => {
                    ctx.emit(
                        Level::Warn,
                        format!("refresh failed for {folder}: {}", message_of(&e)),
                        EventBody::Notice,
                    );
                    (0, 1)
                }
            };
            images::sweep_after(&owner, "a refresh").await;
            Ok(Finished {
                plain: Some(if failed == 0 {
                    format!("Updated the info for {folder}.")
                } else {
                    format!("Could not update the info for {folder}. Try again later.")
                }),
                level: if failed == 0 {
                    Level::Info
                } else {
                    Level::Warn
                },
                message: if failed == 0 {
                    format!("refreshed {folder}")
                } else {
                    format!("refresh failed for {folder}")
                },
                body: EventBody::RefreshFinished { refreshed, failed },
            })
        }))
}

/// Starts the RefreshAll job: every series carrying an AniList or a TVmaze
/// id, one at a time, paced by each provider's limiter.
pub fn refresh_all(core: &Arc<Core>) -> u64 {
    start_refresh_walk(core, JobKind::RefreshAll, "refresh", ALL_MATCHED_SQL)
}

/// Starts the stub backfill: every series whose media row was never
/// fetched, only known from an edge, a recommendation, a list or an
/// import. The native form of Electron's `backfillRelationsForLibrary`,
/// and Task 31 queues it once at launch. A TVmaze match with no show row
/// behind it, which is what an import leaves, is one of these too.
pub fn backfill_stubs(core: &Arc<Core>) -> u64 {
    start_refresh_walk(core, JobKind::Refresh, "backfill", STUBS_SQL)
}

// The walk ------------------------------------------------------------------

const ALL_MATCHED_SQL: &str = "SELECT id, folder_name, anilist_id, tvmaze_id, confirmed FROM series
     WHERE anilist_id IS NOT NULL OR (provider = 'tvmaze' AND tvmaze_id IS NOT NULL)
     ORDER BY id";

const STUBS_SQL: &str = "SELECT s.id, s.folder_name, s.anilist_id, NULL, s.confirmed
     FROM series s JOIN anilist_media m ON m.id = s.anilist_id
     WHERE m.fetched_at IS NULL
     UNION ALL
     SELECT s.id, s.folder_name, NULL, s.tvmaze_id, s.confirmed
     FROM series s
     WHERE s.provider = 'tvmaze' AND s.tvmaze_id IS NOT NULL
       AND NOT EXISTS (SELECT 1 FROM tvmaze_shows t WHERE t.id = s.tvmaze_id)
     ORDER BY 1";

/// The provider id a refresh asks about: the AniList media or the TVmaze
/// show behind the match.
#[derive(Clone, Copy, Debug)]
enum RecordId {
    Anilist(u64),
    Tvmaze(u64),
}

/// The fetch behind either id, writing against `series`.
async fn fetch_id(
    core: &Core,
    series: u64,
    id: RecordId,
    confirmed: bool,
) -> Result<(), CoreError> {
    match id {
        RecordId::Anilist(anilist_id) => {
            fetch::fetch_and_write(core, series, anilist_id, None, confirmed, time::now_secs())
                .await
        }
        RecordId::Tvmaze(show_id) => {
            fetch::fetch_and_write_tvmaze(core, series, show_id, confirmed, time::now_secs()).await
        }
    }
}

/// One series' turn in a refresh walk.
struct Candidate {
    series: u64,
    folder: String,
    id: RecordId,
    confirmed: bool,
}

/// The body both walks share. `label` names the run in the progress and
/// the terminal line; the SQL is what decides whose turn it is. The whole
/// library runs one at a time under `RefreshAll`; the backfill is a plain
/// `Refresh`, which is what lets Task 31 queue it at launch beside
/// whatever else is starting.
///
/// The list is read once, up front, so the loop is finite whatever the
/// fetches write back to the series rows.
fn start_refresh_walk(
    core: &Arc<Core>,
    kind: JobKind,
    label: &'static str,
    sql: &'static str,
) -> u64 {
    let owner = core.clone();
    core.jobs.clone().start(kind, move |ctx| async move {
        let work = owner.store.write_async(move |c| candidates(c, sql)).await?;
        let total = work.len() as u64;
        let (mut refreshed, mut failed) = (0u64, 0u64);
        for (done, c) in work.into_iter().enumerate() {
            ctx.checkpoint()?;
            ctx.progress(done as u64, Some(total), label);
            match fetch_id(&owner, c.series, c.id, c.confirmed).await {
                Ok(()) => {
                    refreshed += 1;
                    if let Some(card) = card_for(&owner, c.series).await? {
                        ctx.changed(card);
                    }
                }
                // A provider saying stop, or nothing answering at all, ends
                // the walk rather than counting one failed series and
                // asking again for the next.
                Err(e) if provider_stopped(&e) => return Err(e),
                Err(e) => {
                    failed += 1;
                    ctx.emit(
                        Level::Warn,
                        format!("refresh failed for {}: {}", c.folder, message_of(&e)),
                        EventBody::Notice,
                    );
                }
            }
        }
        // Both walks write records, so both bring covers, banners and
        // portraits in, and both owe the sweep that keeps the directory
        // from only ever growing.
        images::sweep_after(&owner, label).await;
        let shows = |n: u64| crate::words::pick(n, "show", "shows");
        let said = match (refreshed, failed) {
            (0, 0) => None,
            (n, 0) => Some(format!("Updated the info for {n} {}.", shows(n))),
            (0, f) => Some(format!(
                "Could not update the info for {f} {}. Try again later.",
                shows(f)
            )),
            (n, f) => Some(format!(
                "Updated the info for {n} {}; {f} could not be updated.",
                shows(n)
            )),
        };
        Ok(Finished {
            plain: said,
            level: if failed == 0 {
                Level::Info
            } else {
                Level::Warn
            },
            message: format!("{label}: {refreshed} refreshed, {failed} failed"),
            body: EventBody::RefreshFinished { refreshed, failed },
        })
    })
}

fn candidates(conn: &rusqlite::Connection, sql: &str) -> Result<Vec<Candidate>, CoreError> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |r| {
        let anilist_id = r.get::<_, Option<i64>>(2)?;
        let tvmaze_id = r.get::<_, Option<i64>>(3)?;
        Ok((
            r.get::<_, i64>(0)? as u64,
            r.get::<_, String>(1)?,
            anilist_id,
            tvmaze_id,
            r.get::<_, i64>(4)? == 1,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (series, folder, anilist_id, tvmaze_id, confirmed) = row?;
        // AniList first: a row that somehow carries both is an AniList
        // match, the way every read takes it.
        let id = match (anilist_id, tvmaze_id) {
            (Some(a), _) => RecordId::Anilist(a as u64),
            (None, Some(t)) => RecordId::Tvmaze(t as u64),
            (None, None) => continue,
        };
        out.push(Candidate {
            series,
            folder,
            id,
            confirmed,
        });
    }
    Ok(out)
}

// The pieces ----------------------------------------------------------------

/// One search hit as the modal draws it. AniList orders by relevance and
/// the shell keeps that order, so nothing here sorts or filters.
pub(crate) fn to_result(m: &Media) -> SearchResult {
    let title = m
        .title
        .romaji
        .clone()
        .or_else(|| m.title.english.clone())
        .or_else(|| m.title.native.clone())
        .unwrap_or_else(|| "?".to_string());
    SearchResult {
        provider: Provider::Anilist,
        id: m.id,
        // The second line under the title, and only when it says something
        // the first line did not.
        alt_title: m.title.english.clone().filter(|e| *e != title),
        title,
        format: m.format.clone(),
        year: m
            .season_year
            .or_else(|| m.start_date.as_ref().and_then(|d| d.year)),
        episodes: m.episodes,
        cover_url: m
            .cover_image
            .as_ref()
            .and_then(|c| c.extra_large.clone().or_else(|| c.large.clone())),
        romaji: m.title.romaji.clone(),
        english: m.title.english.clone(),
        native: m.title.native.clone(),
        synonyms: m.synonyms.clone(),
        airing: matches!(
            m.status.as_deref(),
            Some("RELEASING") | Some("NOT_YET_RELEASED")
        ),
    }
}

/// A TVmaze search hit as the modal draws it. The second line is the
/// network, which is what tells two shows of one name apart, and the
/// cover is the small image: a picker has no use for the originals.
fn show_result(s: &Show) -> SearchResult {
    SearchResult {
        provider: Provider::Tvmaze,
        id: s.id,
        title: s.name.clone().unwrap_or_else(|| "?".to_string()),
        alt_title: None,
        format: s.network_name().or_else(|| s.kind.clone()),
        year: s.year(),
        episodes: None,
        cover_url: s.thumbnail_url(),
        romaji: None,
        english: s.name.clone(),
        native: None,
        synonyms: Vec::new(),
        airing: false,
    }
}

/// What a refresh needs to know before it is worth starting a job for.
struct RefreshTarget {
    folder: String,
    provider: Option<String>,
    id: Option<RecordId>,
    confirmed: bool,
}

fn refresh_target(
    conn: &rusqlite::Connection,
    series: u64,
) -> Result<Option<RefreshTarget>, CoreError> {
    Ok(conn
        .query_row(
            "SELECT folder_name, provider, anilist_id, tvmaze_id, confirmed FROM series WHERE id = ?1",
            params![series as i64],
            |r| {
                let provider: Option<String> = r.get(1)?;
                let anilist_id = r.get::<_, Option<i64>>(2)?.map(|v| RecordId::Anilist(v as u64));
                let tvmaze_id = r.get::<_, Option<i64>>(3)?.map(|v| RecordId::Tvmaze(v as u64));
                let id = match provider.as_deref() {
                    Some("tvmaze") => tvmaze_id,
                    _ => anilist_id,
                };
                Ok(RefreshTarget {
                    folder: r.get(0)?,
                    provider,
                    id,
                    confirmed: r.get::<_, i64>(4)? == 1,
                })
            },
        )
        .optional()?)
}

fn folder_name(core: &Core, series: u64) -> Result<String, CoreError> {
    let folder: Option<String> = core.store.read(|c| {
        Ok(c.query_row(
            "SELECT folder_name FROM series WHERE id = ?1",
            params![series as i64],
            |r| r.get(0),
        )
        .optional()?)
    })?;
    folder.ok_or(CoreError::NotFound {
        what: Entity::Series,
        id: series,
    })
}

/// The one card a single-series job reports. `None` when the series went
/// away underneath the job, which is not worth failing over.
pub(crate) async fn card_for(core: &Core, series: u64) -> Result<Option<SeriesCard>, CoreError> {
    let dir = core.paths.images_dir();
    let cards = core
        .store
        .write_async(move |c| cards::cards_for(c, &dir, &[series]))
        .await?;
    Ok(cards.into_iter().next())
}

/// A rate limit the limiter could not ride out: the provider saying stop.
pub(crate) fn is_rate_limited(e: &CoreError) -> bool {
    matches!(
        e,
        CoreError::Provider {
            status: Some(429),
            ..
        }
    )
}

/// Nothing answered at all: a dead socket, a name that did not resolve, a
/// request that timed out. The provider never had an opinion about this
/// item, so the item is never stamped for it.
pub(crate) fn provider_unreachable(e: &CoreError) -> bool {
    matches!(e, CoreError::Provider { status: None, .. })
}

/// The provider saying stop rather than this item failing. Either way the
/// walk ends and stamps nothing: one outage is one state change, and the
/// next run starts where this one stopped instead of walking a whole
/// library of series it could not reach and calling each of them answered.
pub(crate) fn provider_stopped(e: &CoreError) -> bool {
    is_rate_limited(e) || provider_unreachable(e)
}
