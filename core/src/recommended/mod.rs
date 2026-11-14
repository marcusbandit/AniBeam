//! Recommended for you (#779): shows ranked for the user from their own
//! list, by the recommender in `anibeam-recs`.
//!
//! Three rules shape the module. It reads its own tables (`rec_*`) and never
//! writes a recommended show into the library's, so a show only suggested
//! never reaches the franchise graph or the image fill. Every AniList read
//! is cached with a time to live and paced on top of the shared limiter, so
//! a refresh after the first one sends a handful of requests. And no spoiler
//! tag leaves this module by name: they count toward the ranking, and the
//! records only ever carry how many the user loves.
//!
//! The fit runs as a job after every tracker refresh, off the cache, with
//! the reads it is missing first.

pub mod fit;
pub mod mine;
pub mod page;
pub mod parse;
pub mod queries;
pub mod sync;
pub mod text;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use anibeam_recs::DropReasons;
use anibeam_recs::signal::signals;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

use crate::contract::*;
use crate::core::Core;
use crate::foundation::titles;
use crate::jobs::{Finished, JobCtx};
use crate::prefs;
use crate::time;
use mine::Mine;

/// The recommender's own least gap between two AniList requests, in
/// milliseconds: 24 a minute, under the 30 AniList allows while degraded.
pub const GAP_MS: u64 = 2_500;

/// How old a fit may be before `RefreshRecommended` without `force` runs
/// again. A tracker refresh runs one whatever its age.
pub const STALE_AFTER_SECS: i64 = 12 * 3_600;

/// The most shows one read of the row hands the shell, per scope.
const SHOWN: usize = 120;

/// Covers fetched after a fit: the best ranked, which is what the row and
/// See all draw, and the user's own shows behind them.
const COVERS: usize = 150;

// The read ----------------------------------------------------------------------

/// A show's display fields, off its cached reply.
pub(crate) struct Display<'a> {
    pub media: &'a Value,
}

impl Display<'_> {
    pub fn title(&self, lang: TitleLanguage, id: u64) -> String {
        let t = &self.media["title"];
        titles::resolve(
            lang,
            t["romaji"].as_str(),
            t["english"].as_str(),
            t["native"].as_str(),
            &format!("AniList {id}"),
        )
    }

    pub fn names(&self) -> (Option<String>, Option<String>, Option<String>) {
        let t = &self.media["title"];
        let text = |v: &Value| {
            v.as_str()
                .filter(|s| !s.trim().is_empty())
                .map(str::to_string)
        };
        (text(&t["romaji"]), text(&t["english"]), text(&t["native"]))
    }

    /// The cover a tile draws: AniList's large one.
    pub fn cover_url(&self) -> Option<String> {
        let c = &self.media["coverImage"];
        c["large"]
            .as_str()
            .or(c["extraLarge"].as_str())
            .map(str::to_string)
    }

    pub fn u32(&self, key: &str) -> Option<u32> {
        self.media[key].as_u64().and_then(|v| u32::try_from(v).ok())
    }

    pub fn string(&self, key: &str) -> Option<String> {
        self.media[key]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }

    /// Tags a person may read, strongest first: never a spoiler, never one
    /// AniList marks adult.
    pub fn tags(&self) -> Vec<(String, u32)> {
        let mut tags: Vec<(String, u32)> = self.media["tags"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|t| !parse::is_spoiler(t) && t["isAdult"].as_bool() != Some(true))
            .filter_map(|t| {
                Some((
                    t["name"].as_str()?.to_string(),
                    t["rank"].as_u64().unwrap_or(0) as u32,
                ))
            })
            .collect();
        tags.sort_by_key(|t| std::cmp::Reverse(t.1));
        tags
    }
}

/// The cached reply for each show in `ids`, off `rec_shows`.
pub(crate) fn media_of(conn: &Connection, ids: &[u64]) -> Result<HashMap<u64, Value>, CoreError> {
    let mut stmt = conn.prepare_cached(
        "SELECT media FROM rec_shows WHERE anilist_id = ?1 AND media IS NOT NULL",
    )?;
    let mut out = HashMap::new();
    for id in ids {
        let text: Option<String> = stmt
            .query_row(params![*id as i64], |r| r.get(0))
            .optional()?;
        if let Some(v) = text.and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
            out.insert(*id, v);
        }
    }
    Ok(out)
}

/// A cached image's absolute path, while its file is still there.
pub(crate) fn poster(
    conn: &Connection,
    images_dir: &Path,
    url: Option<&str>,
) -> Result<Option<String>, CoreError> {
    let Some(url) = url else { return Ok(None) };
    let relative: Option<String> = conn
        .prepare_cached("SELECT path FROM images WHERE url = ?1")?
        .query_row(params![url], |r| r.get(0))
        .optional()?;
    Ok(relative
        .map(|relative| images_dir.join(relative))
        .filter(|path| path.is_file())
        .map(|path| path.to_string_lossy().into_owned()))
}

/// The user's shows behind one pick, as the row draws them.
pub(crate) fn because(
    conn: &Connection,
    images_dir: &Path,
    lang: TitleLanguage,
    mine: &Mine,
    why: &fit::Why,
) -> Result<Vec<BecauseShow>, CoreError> {
    let ids: Vec<u64> = why.near.iter().map(|n| n.id).collect();
    let media = media_of(conn, &ids)?;
    let mut out = Vec::new();
    for near in &why.near {
        let Some(m) = media.get(&near.id) else {
            continue;
        };
        let d = Display { media: m };
        out.push(BecauseShow {
            anilist_id: near.id,
            title: d.title(lang, near.id),
            poster: poster(conn, images_dir, d.cover_url().as_deref())?,
            score: Some(f64::from((near.score * 10.0).round() / 10.0)),
            owned: mine.owned.get(&near.id).copied(),
        });
    }
    Ok(out)
}

fn state_of(conn: &Connection) -> Result<(RecommendedState, Option<i64>), CoreError> {
    let row: Option<(String, Option<i64>)> = conn
        .query_row(
            "SELECT state, fitted_at FROM rec_state WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(match row {
        Some((state, at)) => (
            RecommendedState::from_column(&state).unwrap_or_default(),
            at,
        ),
        None => (RecommendedState::NeverFitted, None),
    })
}

/// The row off the database: the last fit, best first, held to `scope` and
/// to what the user has yet to watch as the list reads now. A show marked
/// Not interested stays off, and so does one the library hides.
pub fn list(
    conn: &Connection,
    images_dir: &Path,
    scope: RecommendedScope,
) -> Result<Recommended, CoreError> {
    let lang = prefs::load_preferences(conn)?.title_language;
    let mine = Mine::read(conn)?;
    let (state, fitted_at) = state_of(conn)?;
    let not_interested: HashSet<u64> = {
        let mut stmt = conn.prepare("SELECT anilist_id FROM not_interested")?;
        let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
        rows.filter_map(Result::ok).map(|id| id as u64).collect()
    };
    let mut stmt = conn.prepare(
        "SELECT r.anilist_id, r.forecast, r.why, s.media FROM rec_ranked r
           JOIN rec_shows s ON s.anilist_id = r.anilist_id
          WHERE s.media IS NOT NULL
          ORDER BY r.rank",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)? as u64,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    })?;
    let mut counts = RecommendedCounts::default();
    let mut shows = Vec::new();
    for row in rows {
        let (id, forecast, why, media) = row?;
        if not_interested.contains(&id) || mine.hidden.contains(&id) {
            continue;
        }
        let Ok(media) = serde_json::from_str::<Value>(&media) else {
            continue;
        };
        let d = Display { media: &media };
        let episodes = d.u32("episodes");
        if !mine.unwatched(id, episodes) {
            continue;
        }
        let owned = mine.owned.get(&id).copied();
        let on_list = mine.on_list(id, episodes);
        counts.all += 1;
        counts.in_library += u32::from(owned.is_some());
        counts.on_list += u32::from(on_list);
        let fits = match scope {
            RecommendedScope::All => true,
            RecommendedScope::InLibrary => owned.is_some(),
            RecommendedScope::OnList => on_list,
        };
        if !fits || shows.len() >= SHOWN {
            continue;
        }
        let why: fit::Why = serde_json::from_str(&why).unwrap_or(fit::Why {
            near: Vec::new(),
            hidden: 0,
        });
        let (romaji, english, native) = d.names();
        shows.push(RecommendedShow {
            anilist_id: id,
            mal_id: media["idMal"].as_u64(),
            title: d.title(lang, id),
            romaji,
            english,
            native,
            poster: poster(conn, images_dir, d.cover_url().as_deref())?,
            color: media["coverImage"]["color"].as_str().map(str::to_string),
            format: d.string("format"),
            year: media["seasonYear"].as_i64().map(|y| y as i32),
            episodes,
            duration: d.u32("duration"),
            average_score: d.u32("averageScore"),
            genres: parse::strings(&media["genres"]),
            tags: d.tags().into_iter().take(5).map(|(name, _)| name).collect(),
            line: media["description"]
                .as_str()
                .map(text::line)
                .unwrap_or_default(),
            forecast: forecast.clamp(0, 100) as u32,
            hidden_loved: why.hidden,
            because: because(conn, images_dir, lang, &mine, &why)?,
            owned,
            list_status: mine.listed.get(&id).map(|l| l.status),
            site_url: d.string("siteUrl"),
        });
    }
    Ok(Recommended {
        scope,
        state,
        shows,
        counts,
        fitted_at: fitted_at.map(time::from_secs),
    })
}

/// The `ListRecommended` call: the last fit at once, and the id of a fit
/// already running so the row can say one is on its way.
pub fn list_call(core: &Core, scope: RecommendedScope) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let recommended = core.store.read(|c| list(c, &images_dir, scope))?;
    Ok(Reply::Recommended {
        recommended,
        refreshing: core.jobs.running(JobKind::RefreshRecommended),
    })
}

/// The `RefreshRecommended` call: the job when the last whole read is older
/// than `STALE_AFTER_SECS` (or there never was one) or `force` says so, `Ok`
/// when it is fresh. A run AniList stopped short is never fresh. One runs at a time; a second ask joins it.
pub fn refresh_call(core: &Core, force: bool) -> Result<Reply, CoreError> {
    let read_at: Option<i64> = core.store.read(|c| {
        Ok(
            c.query_row("SELECT read_at FROM rec_state WHERE id = 1", [], |r| {
                r.get(0)
            })
            .optional()?
            .flatten(),
        )
    })?;
    let now = time::now_secs();
    if !force && read_at.is_some_and(|at| now - at < STALE_AFTER_SECS) {
        return Ok(Reply::Ok);
    }
    Ok(Reply::Started {
        job: start(&core.owner()?),
    })
}

/// Starts the fit, or answers with the one running.
pub fn start(core: &Arc<Core>) -> u64 {
    let owner = core.clone();
    core.jobs
        .clone()
        .start(JobKind::RefreshRecommended, move |ctx| async move {
            refresh(owner, ctx).await
        })
}

// The job -----------------------------------------------------------------------

/// What a run leaves behind. `whole` says the read got through everything it
/// set out to; a run AniList stopped short still saves its fit, but only a
/// whole read moves `read_at`, so the next ask goes back for the rest.
fn save_state(
    tx: &rusqlite::Transaction,
    state: RecommendedState,
    found: Option<&sync::Found>,
    rated: usize,
    whole: bool,
    now: i64,
) -> Result<(), CoreError> {
    let found = found.cloned().unwrap_or_default();
    tx.execute(
        "INSERT INTO rec_state (id, people, pool, trusted, fitted_at, read_at, rated, state)
         VALUES (1, ?1, ?2, ?3, ?4, CASE WHEN ?5 THEN ?4 END, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET people = excluded.people, pool = excluded.pool,
            trusted = excluded.trusted, fitted_at = excluded.fitted_at,
            read_at = CASE WHEN ?5 THEN ?4 ELSE rec_state.read_at END,
            rated = excluded.rated, state = excluded.state",
        params![
            serde_json::json!(found.people).to_string(),
            serde_json::json!(found.pool).to_string(),
            serde_json::json!(found.trusted).to_string(),
            now,
            whole,
            rated as i64,
            state.as_str(),
        ],
    )?;
    Ok(())
}

fn finished(message: String) -> Finished {
    Finished {
        plain: None,
        level: Level::Debug,
        message,
        body: EventBody::RecommendedRefreshed,
    }
}

async fn refresh(core: Arc<Core>, ctx: Arc<JobCtx>) -> Result<Finished, CoreError> {
    let mine = core.store.read(Mine::read)?;
    let now = time::now_secs();
    let empty = |state: RecommendedState| {
        move |tx: &rusqlite::Transaction| {
            tx.execute("DELETE FROM rec_ranked", [])?;
            save_state(tx, state, None, 0, true, now)
        }
    };
    if mine.entries.is_empty() {
        core.store.tx_async(empty(RecommendedState::NoList)).await?;
        return Ok(finished(
            "Recommended for you: no list to learn from".to_string(),
        ));
    }
    let Some((mean, signals)) = signals(&mine.entries, &DropReasons::new()) else {
        core.store
            .tx_async(empty(RecommendedState::NoScores))
            .await?;
        return Ok(finished(
            "Recommended for you: nothing on the list is rated".to_string(),
        ));
    };

    let mut net = sync::Net::new(core.clone(), ctx.clone());
    let mut whole = true;
    let found = match sync::read(&mut net, &mine, &signals, mean).await {
        Ok(found) => found,
        // Read what could be read; the fit runs on the cache, with the
        // people and candidates the last full run found.
        Err(sync::Halt::Stop(why)) => {
            whole = false;
            ctx.emit(
                Level::Debug,
                format!("Recommended for you stopped reading AniList: {why}"),
                EventBody::Notice,
            );
            core.store.read(sync::saved)?
        }
        Err(sync::Halt::Fail(e)) => return Err(e),
    };
    ctx.checkpoint()?;
    ctx.progress(0, None, &crate::i18n::tr("core-recommended-ranking", &[]));

    let (catalog, neighbours) = {
        let people = found.people.clone();
        core.store
            .read(|c| Ok((sync::catalog(c)?, sync::neighbours(c, &people)?)))?
    };
    let rated = signals.len();
    let ranked = {
        let mine = mine.clone();
        let trusted = found.trusted.clone();
        tokio::task::spawn_blocking(move || {
            fit::rank(&mine, &signals, mean, &catalog, &neighbours, &trusted)
        })
        .await
        .map_err(|e| CoreError::internal(format!("the fit stopped: {e}")))?
    };
    let count = ranked.len();
    let covers_for: Vec<u64> = ranked
        .iter()
        .take(COVERS)
        .flat_map(|r| std::iter::once(r.id).chain(r.why.near.iter().map(|n| n.id)))
        .collect();
    let saved = found.clone();
    core.store
        .tx_async(move |tx| {
            tx.execute("DELETE FROM rec_ranked", [])?;
            let mut insert = tx.prepare(
                "INSERT INTO rec_ranked (anilist_id, rank, forecast, predicted, why)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for (rank, r) in ranked.iter().enumerate() {
                insert.execute(params![
                    r.id as i64,
                    rank as i64,
                    i64::from(r.forecast),
                    f64::from(r.predicted),
                    serde_json::to_string(&r.why).unwrap_or_default(),
                ])?;
            }
            save_state(tx, RecommendedState::Ready, Some(&saved), rated, whole, now)
        })
        .await?;

    // Every cover before the event, so the tiles draw local files.
    let urls: Vec<String> = core.store.read(|c| {
        let media = media_of(c, &covers_for)?;
        Ok(covers_for
            .iter()
            .filter_map(|id| {
                Display {
                    media: media.get(id)?,
                }
                .cover_url()
            })
            .collect())
    })?;
    for (url, outcome) in core.images.ensure(&urls).await {
        if let Err(e) = outcome {
            tracing::debug!("a recommended cover {url} was not fetched: {e}");
        }
    }
    Ok(finished(format!(
        "Recommended for you: {count} shows ranked from {rated} on your list, {} AniList requests",
        net.sent
    )))
}

/// Run after a tracker refresh: the list the fit learns from just changed,
/// so a fit that exists is made again. The first one waits for Home to ask
/// (`RefreshRecommended`), so a list nobody looks recommendations up for
/// never sends the recommender's reads.
pub fn after_tracker_refresh(core: &Arc<Core>) {
    match core.store.read(state_of) {
        Ok((_, Some(_))) => {
            start(core);
        }
        Ok((_, None)) => {}
        Err(e) => tracing::debug!("Recommended for you was not refitted: {e}"),
    }
}
