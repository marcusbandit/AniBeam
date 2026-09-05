//! Every card and every detail a shell shows is assembled here, and every
//! job builds its `SeriesChanged` batch through `cards_for`. The pure rules
//! at the top are ports of Electron's `airingUtils.ts` and
//! `SeriesDetailPage.tsx`; the snapshot in the middle is the handful of
//! queries one read needs; the assembly at the bottom turns the two into
//! the contract's records.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::SystemTime;

use rusqlite::{Connection, params_from_iter};
use serde_json::Value;

use crate::contract::*;
use crate::library::{labels, titles};
use crate::prefs;
use crate::time;

// ---------------------------------------------------------------------------
// The pure rules
// ---------------------------------------------------------------------------

/// The fraction of the strip reserved for the dark "total unknown" cap, so a
/// bar without a published episode count can never read as complete.
/// Electron's `UNKNOWN_TAIL_PCT = 15`.
pub const UNKNOWN_TAIL: f64 = 0.15;

/// The three segments of the strip on a card, as fractions of 0 to 1. The
/// port of `computeCardProgress`: watched, aired but unwatched (only while
/// the show is still releasing), and the unknown-total cap.
pub fn strip(
    watched: Option<u32>,
    total: Option<u32>,
    latest_aired: u32,
    next_scheduled: Option<u32>,
    latest_downloaded: u32,
    status: Option<AiringStatus>,
) -> Strip {
    let mut released = latest_aired
        .max(next_scheduled.map_or(0, |n| n.saturating_sub(1)))
        .max(latest_downloaded);
    let known_total = total.filter(|t| *t > 0);
    let finished = status == Some(AiringStatus::Finished);
    if let (true, Some(t)) = (finished, known_total) {
        released = t;
    }
    let fully_released = finished || known_total.is_some_and(|t| released >= t);
    // No tracker entry means there is no watch progress to be behind on, so
    // an untracked card never paints the aired-unwatched segment.
    let show_behind = watched.is_some() && !fully_released;

    if let Some(t) = known_total {
        let t = f64::from(t);
        let pct = |n: u32| (f64::from(n) / t).clamp(0.0, 1.0);
        return Strip {
            watched: pct(watched.unwrap_or(0)),
            aired_unwatched: if show_behind { pct(released) } else { 0.0 },
            unknown: 0.0,
        };
    }

    let extent = released.max(watched.unwrap_or(0));
    if extent == 0 {
        return Strip {
            watched: 0.0,
            aired_unwatched: 0.0,
            unknown: 0.0,
        };
    }
    let usable = 1.0 - UNKNOWN_TAIL;
    let pct = |n: u32| (f64::from(n) / f64::from(extent) * usable).clamp(0.0, usable);
    Strip {
        watched: pct(watched.unwrap_or(0)),
        aired_unwatched: if show_behind { pct(released) } else { 0.0 },
        unknown: UNKNOWN_TAIL,
    }
}

/// The port of `classifyWatchProgress`. Reachable is the highest episode
/// that could be played today: the later of the latest aired one and the
/// latest one on disk, because plenty of shows carry no air dates at all.
pub fn watched_state(
    watched: Option<u32>,
    total: Option<u32>,
    latest_aired: u32,
    latest_downloaded: u32,
) -> WatchedState {
    let Some(w) = watched else {
        return WatchedState::Unknown;
    };
    let known_total = total.filter(|t| *t > 0);
    if known_total.is_some_and(|t| w >= t) {
        return WatchedState::CaughtUp;
    }
    let reachable = latest_aired.max(latest_downloaded);
    if reachable > 0 && w < reachable {
        return WatchedState::Behind;
    }
    if known_total.is_none() {
        return WatchedState::Unknown;
    }
    WatchedState::CaughtUp
}

/// The known total, else the aired estimate (the later of aired and
/// watched) marked as one. The port of `formatWatchedLabel`'s denominator.
pub fn total_with_estimate(
    total: Option<u32>,
    latest_aired: u32,
    watched: Option<u32>,
) -> (Option<u32>, bool) {
    if let Some(t) = total.filter(|t| *t > 0) {
        return (Some(t), false);
    }
    if latest_aired > 0 {
        return (Some(latest_aired.max(watched.unwrap_or(0))), true);
    }
    (None, false)
}

/// `episodes` is (file id, number) sorted by number. `last_completed` is the
/// most recently completed episode's number. With something watched, the
/// episode after it; with nothing watched, the first one on disk, above zero
/// when a tracker entry says the series has been started at all.
pub fn next_up(
    episodes: &[(u64, f64)],
    last_completed: Option<f64>,
    tracker_progress: Option<u32>,
) -> Option<u64> {
    let last = last_completed
        .unwrap_or(0.0)
        .max(f64::from(tracker_progress.unwrap_or(0)));
    if last > 0.0 {
        return episodes
            .iter()
            .find(|(_, n)| *n == last + 1.0)
            .or_else(|| episodes.iter().find(|(_, n)| *n > last))
            .map(|(f, _)| *f);
    }
    if tracker_progress.is_some() {
        return episodes.iter().find(|(_, n)| *n > 0.0).map(|(f, _)| *f);
    }
    episodes.first().map(|(f, _)| *f)
}

// ---------------------------------------------------------------------------
// The rows one read loads
// ---------------------------------------------------------------------------

pub struct SeriesRow {
    pub id: u64,
    pub source_id: u64,
    pub kind: SeriesKind,
    pub path: String,
    pub folder_name: String,
    pub hidden: bool,
    pub missing: bool,
    pub added_at: i64,
    pub provider: Option<Provider>,
    pub anilist_id: Option<u64>,
    pub mal_id: Option<u64>,
    pub tmdb_id: Option<u64>,
    pub tmdb_kind: Option<TmdbKind>,
    pub confirmed: bool,
    pub track_choice: Option<String>,
    pub media: Option<MediaRow>,
    pub poster_path: Option<String>,
    pub banner_path: Option<String>,
}

pub struct MediaRow {
    pub title_romaji: Option<String>,
    pub title_english: Option<String>,
    pub title_native: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub year: Option<u32>,
    pub episodes: Option<u32>,
    pub description: Option<String>,
    pub average_score: Option<u32>,
    pub genres: Vec<String>,
    pub studio: Option<String>,
    pub tags: Value,
    pub characters: Value,
    pub site_url: Option<String>,
    pub cover_url: Option<String>,
    pub banner_url: Option<String>,
}

pub struct FileRow {
    pub id: u64,
    pub series_id: u64,
    pub path: String,
    pub mtime: i64,
    pub is_episode: bool,
    pub number: f64,
    pub season: Option<u32>,
    pub extra_kind: Option<ExtraKind>,
    pub extra_index: Option<u32>,
    pub label: String,
    pub episode_key: String,
    pub sidecars: Vec<Sidecar>,
}

pub struct AiringRow {
    pub number: u32,
    pub title: Option<String>,
    pub aired_at: Option<i64>,
}

pub struct TrackerRow {
    pub progress: u32,
    pub status: Option<ListStatus>,
    pub score: Option<f64>,
    pub repeat: u32,
}

/// Everything one read needs, in a handful of queries. `scope` restricts
/// every one of them to a set of series, so building the cards for a job's
/// batch never loads the whole library.
pub struct Snapshot {
    pub now: SystemTime,
    pub lang: TitleLanguage,
    pub main_tracker: Tracker,
    pub series: Vec<SeriesRow>,
    /// Series id to its position in `series`.
    index: HashMap<u64, usize>,
    pub files: HashMap<u64, Vec<FileRow>>,
    /// By AniList id.
    pub airing: HashMap<u64, Vec<AiringRow>>,
    pub anilist_entries: HashMap<u64, TrackerRow>,
    pub mal_entries: HashMap<u64, TrackerRow>,
    pub views: HashMap<u64, (String, i64)>,
    pub completed: HashMap<u64, Vec<(String, i64)>>,
    pub resume: HashMap<(u64, String), (f64, f64)>,
    /// AniList ids with at least one relation row.
    pub graph_seeds: HashSet<u64>,
    pub images_dir: String,
    /// Series whose media names a cover the image cache has no row for.
    /// The read hands this to the cache, which decides whether it is worth
    /// a fill; the loader gets it free from the join it already makes.
    pub gaps: Vec<u64>,
}

/// `?,?,?` for an `IN` list of `n` values. Ids go in as bound parameters,
/// never formatted into the SQL. SQLite accepts an empty `IN ()` and reads
/// it as false, so a scope that matches no series loads nothing rather than
/// failing to parse.
fn placeholders(n: usize) -> String {
    let mut out = String::with_capacity(n * 2);
    for i in 0..n {
        if i > 0 {
            out.push(',');
        }
        out.push('?');
    }
    out
}

fn ids_as_i64(ids: &[u64]) -> Vec<i64> {
    ids.iter().map(|i| *i as i64).collect()
}

fn json_strings(raw: &str) -> Vec<String> {
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Array(a)) => a
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn json_value(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or(Value::Null)
}

fn json_array(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn sidecars_of(raw: &str) -> Vec<Sidecar> {
    match serde_json::from_str::<Vec<Sidecar>>(raw) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("a file's sidecars did not parse, treating it as none: {e}");
            Vec::new()
        }
    }
}

impl Snapshot {
    /// Loads the whole library with `scope` None, or exactly the named
    /// series with `scope` Some. An empty scope loads nothing at all.
    pub fn load(
        conn: &Connection,
        images_dir: &Path,
        now: SystemTime,
        scope: Option<&[u64]>,
    ) -> Result<Snapshot, CoreError> {
        let mut snap = Snapshot {
            now,
            lang: prefs::load_preferences(conn)?.title_language,
            main_tracker: prefs::load_main_tracker(conn)?,
            series: Vec::new(),
            index: HashMap::new(),
            files: HashMap::new(),
            airing: HashMap::new(),
            anilist_entries: HashMap::new(),
            mal_entries: HashMap::new(),
            views: HashMap::new(),
            completed: HashMap::new(),
            resume: HashMap::new(),
            graph_seeds: HashSet::new(),
            images_dir: images_dir.to_string_lossy().into_owned(),
            gaps: Vec::new(),
        };
        if scope.is_some_and(<[u64]>::is_empty) {
            return Ok(snap);
        }

        snap.load_series(conn, scope, images_dir)?;
        let series_ids: Vec<u64> = snap.series.iter().map(|r| r.id).collect();
        let anilist_ids: Vec<u64> = snap.series.iter().filter_map(|r| r.anilist_id).collect();
        let media_ids: Vec<u64> = snap
            .series
            .iter()
            .flat_map(|r| [r.anilist_id, r.mal_id])
            .flatten()
            .collect();
        let scoped_series = scope.map(|_| series_ids.as_slice());

        snap.load_files(conn, scoped_series)?;
        snap.load_airing(conn, scope.map(|_| anilist_ids.as_slice()))?;
        snap.load_tracker_entries(conn, scope.map(|_| media_ids.as_slice()))?;
        snap.load_history(conn, scoped_series)?;
        snap.load_graph_seeds(conn, scope.map(|_| anilist_ids.as_slice()))?;
        Ok(snap)
    }

    fn load_series(
        &mut self,
        conn: &Connection,
        scope: Option<&[u64]>,
        images_dir: &Path,
    ) -> Result<(), CoreError> {
        let where_sql = match scope {
            Some(ids) => format!(" WHERE s.id IN ({})", placeholders(ids.len())),
            None => String::new(),
        };
        let sql = format!(
            "SELECT s.id, s.source_id, s.kind, s.path, s.folder_name, s.hidden, s.missing_since, s.added_at,
                    s.provider, s.anilist_id, s.mal_id, s.tmdb_id, s.tmdb_kind, s.confirmed, s.track_choice,
                    m.id, m.title_romaji, m.title_english, m.title_native, m.format, m.status, m.year, m.episodes,
                    m.description, m.average_score, m.genres, m.studio, m.tags, m.characters, m.site_url,
                    m.cover_url, m.banner_url, ip.path, ib.path
             FROM series s
             LEFT JOIN anilist_media m ON m.id = s.anilist_id
             LEFT JOIN images ip ON ip.url = m.cover_url
             LEFT JOIN images ib ON ib.url = m.banner_url
             {where_sql}
             ORDER BY s.id"
        );
        let local = |relative: Option<String>| {
            relative.map(|p| images_dir.join(p).to_string_lossy().into_owned())
        };
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(ids_as_i64(scope.unwrap_or(&[]))), |r| {
            let id: i64 = r.get(0)?;
            let kind_col: String = r.get(2)?;
            let Some(kind) = SeriesKind::from_column(&kind_col) else {
                tracing::warn!("skipping series {id}: unknown kind {kind_col:?}");
                return Ok(None);
            };
            let media = match r.get::<_, Option<i64>>(15)? {
                None => None,
                Some(_) => Some(MediaRow {
                    title_romaji: r.get(16)?,
                    title_english: r.get(17)?,
                    title_native: r.get(18)?,
                    format: r.get(19)?,
                    status: r.get(20)?,
                    year: r.get::<_, Option<i64>>(21)?.map(|v| v as u32),
                    episodes: r.get::<_, Option<i64>>(22)?.map(|v| v as u32),
                    description: r.get(23)?,
                    average_score: r.get::<_, Option<i64>>(24)?.map(|v| v as u32),
                    genres: json_strings(&r.get::<_, String>(25)?),
                    studio: r.get(26)?,
                    tags: json_value(&r.get::<_, String>(27)?),
                    characters: json_value(&r.get::<_, String>(28)?),
                    site_url: r.get(29)?,
                    cover_url: r.get(30)?,
                    banner_url: r.get(31)?,
                }),
            };
            Ok(Some(SeriesRow {
                id: id as u64,
                source_id: r.get::<_, i64>(1)? as u64,
                kind,
                path: r.get(3)?,
                folder_name: r.get(4)?,
                hidden: r.get::<_, i64>(5)? != 0,
                missing: r.get::<_, Option<i64>>(6)?.is_some(),
                added_at: r.get(7)?,
                provider: r
                    .get::<_, Option<String>>(8)?
                    .and_then(|s| Provider::from_column(&s)),
                anilist_id: r.get::<_, Option<i64>>(9)?.map(|v| v as u64),
                mal_id: r.get::<_, Option<i64>>(10)?.map(|v| v as u64),
                tmdb_id: r.get::<_, Option<i64>>(11)?.map(|v| v as u64),
                tmdb_kind: r
                    .get::<_, Option<String>>(12)?
                    .and_then(|s| TmdbKind::from_column(&s)),
                confirmed: r.get::<_, i64>(13)? != 0,
                track_choice: r.get(14)?,
                media,
                poster_path: local(r.get(32)?),
                banner_path: local(r.get(33)?),
            }))
        })?;
        for row in rows {
            if let Some(row) = row? {
                // The LEFT JOIN above already answered this: a cover the
                // series names with no path beside it is an image nobody
                // has fetched yet.
                if row.poster_path.is_none()
                    && row.media.as_ref().is_some_and(|m| m.cover_url.is_some())
                {
                    self.gaps.push(row.id);
                }
                self.index.insert(row.id, self.series.len());
                self.series.push(row);
            }
        }
        Ok(())
    }

    fn load_files(&mut self, conn: &Connection, scope: Option<&[u64]>) -> Result<(), CoreError> {
        let where_sql = match scope {
            Some(ids) => format!(" WHERE series_id IN ({})", placeholders(ids.len())),
            None => String::new(),
        };
        let sql = format!(
            "SELECT id, series_id, path, mtime, kind, number, season, extra_kind, extra_index, label, episode_key, sidecars
             FROM files{where_sql}
             ORDER BY series_id, season, number, path"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(ids_as_i64(scope.unwrap_or(&[]))), |r| {
            let kind: String = r.get(4)?;
            Ok(FileRow {
                id: r.get::<_, i64>(0)? as u64,
                series_id: r.get::<_, i64>(1)? as u64,
                path: r.get(2)?,
                mtime: r.get(3)?,
                // A row that is not marked an extra is a real episode, the
                // way Electron treated a file with no kind at all.
                is_episode: kind != "extra",
                number: r.get::<_, Option<f64>>(5)?.unwrap_or(0.0),
                season: r.get::<_, Option<i64>>(6)?.map(|v| v as u32),
                extra_kind: r
                    .get::<_, Option<String>>(7)?
                    .and_then(|s| ExtraKind::from_column(&s)),
                extra_index: r.get::<_, Option<i64>>(8)?.map(|v| v as u32),
                label: r.get(9)?,
                episode_key: r.get(10)?,
                sidecars: sidecars_of(&r.get::<_, String>(11)?),
            })
        })?;
        for row in rows {
            let row = row?;
            self.files.entry(row.series_id).or_default().push(row);
        }
        Ok(())
    }

    fn load_airing(&mut self, conn: &Connection, scope: Option<&[u64]>) -> Result<(), CoreError> {
        if scope.is_some_and(<[u64]>::is_empty) {
            return Ok(());
        }
        let where_sql = match scope {
            Some(ids) => format!("anilist_id IN ({})", placeholders(ids.len())),
            None => "anilist_id IN (SELECT anilist_id FROM series WHERE anilist_id IS NOT NULL)"
                .to_string(),
        };
        let sql = format!(
            "SELECT anilist_id, number, title, aired_at FROM anilist_episodes WHERE {where_sql} ORDER BY number"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(ids_as_i64(scope.unwrap_or(&[]))), |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                AiringRow {
                    number: r.get::<_, i64>(1)? as u32,
                    title: r.get(2)?,
                    aired_at: r.get(3)?,
                },
            ))
        })?;
        for row in rows {
            let (anilist_id, airing) = row?;
            self.airing.entry(anilist_id).or_default().push(airing);
        }
        Ok(())
    }

    fn load_tracker_entries(
        &mut self,
        conn: &Connection,
        scope: Option<&[u64]>,
    ) -> Result<(), CoreError> {
        if scope.is_some_and(<[u64]>::is_empty) {
            return Ok(());
        }
        let where_sql = match scope {
            Some(ids) => format!(" WHERE media_id IN ({})", placeholders(ids.len())),
            None => String::new(),
        };
        let sql = format!(
            "SELECT tracker, media_id, progress, status, score, repeat FROM tracker_entries{where_sql}"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(ids_as_i64(scope.unwrap_or(&[]))), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)? as u64,
                TrackerRow {
                    progress: r.get::<_, i64>(2)? as u32,
                    status: r
                        .get::<_, Option<String>>(3)?
                        .and_then(|s| ListStatus::from_column(&s)),
                    score: r.get(4)?,
                    repeat: r.get::<_, i64>(5)? as u32,
                },
            ))
        })?;
        for row in rows {
            let (tracker, media_id, entry) = row?;
            match Tracker::from_column(&tracker) {
                Some(Tracker::Anilist) => {
                    self.anilist_entries.insert(media_id, entry);
                }
                Some(Tracker::Mal) => {
                    self.mal_entries.insert(media_id, entry);
                }
                None => tracing::warn!("skipping a tracker entry: unknown tracker {tracker:?}"),
            }
        }
        Ok(())
    }

    fn load_history(&mut self, conn: &Connection, scope: Option<&[u64]>) -> Result<(), CoreError> {
        let where_sql = match scope {
            Some(ids) => format!(" WHERE series_id IN ({})", placeholders(ids.len())),
            None => String::new(),
        };
        let ids = ids_as_i64(scope.unwrap_or(&[]));

        let mut stmt = conn.prepare(&format!(
            "SELECT series_id, episode_key, at FROM views{where_sql}"
        ))?;
        let rows = stmt.query_map(params_from_iter(ids.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (series_id, key, at) = row?;
            self.views.insert(series_id, (key, at));
        }

        let mut stmt = conn.prepare(&format!(
            "SELECT series_id, episode_key, at FROM completed{where_sql}"
        ))?;
        let rows = stmt.query_map(params_from_iter(ids.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (series_id, key, at) = row?;
            self.completed.entry(series_id).or_default().push((key, at));
        }

        let mut stmt = conn.prepare(&format!(
            "SELECT series_id, episode_key, position, duration FROM resume_points{where_sql}"
        ))?;
        let rows = stmt.query_map(params_from_iter(ids.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, String>(1)?,
                r.get::<_, f64>(2)?,
                r.get::<_, f64>(3)?,
            ))
        })?;
        for row in rows {
            let (series_id, key, position, duration) = row?;
            self.resume.insert((series_id, key), (position, duration));
        }
        Ok(())
    }

    fn load_graph_seeds(
        &mut self,
        conn: &Connection,
        scope: Option<&[u64]>,
    ) -> Result<(), CoreError> {
        if scope.is_some_and(<[u64]>::is_empty) {
            return Ok(());
        }
        let (sql, ids) = match scope {
            Some(ids) => {
                let list = placeholders(ids.len());
                (
                    format!(
                        "SELECT DISTINCT from_id FROM relations WHERE from_id IN ({list}) UNION SELECT DISTINCT to_id FROM relations WHERE to_id IN ({list})"
                    ),
                    [ids_as_i64(ids), ids_as_i64(ids)].concat(),
                )
            }
            None => (
                "SELECT DISTINCT from_id FROM relations UNION SELECT DISTINCT to_id FROM relations"
                    .to_string(),
                Vec::new(),
            ),
        };
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(ids.iter()), |r| {
            Ok(r.get::<_, i64>(0)? as u64)
        })?;
        for row in rows {
            self.graph_seeds.insert(row?);
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // The assembly
    // -----------------------------------------------------------------------

    pub fn row(&self, id: u64) -> Option<&SeriesRow> {
        self.index.get(&id).and_then(|i| self.series.get(*i))
    }

    /// The main tracker's entry for this series, falling back to the other
    /// tracker's. The port of `TrackerProgressContext.lookupEntry`.
    pub fn tracker(&self, row: &SeriesRow) -> Option<&TrackerRow> {
        let (primary, primary_id, secondary, secondary_id) = match self.main_tracker {
            Tracker::Anilist => (
                &self.anilist_entries,
                row.anilist_id,
                &self.mal_entries,
                row.mal_id,
            ),
            Tracker::Mal => (
                &self.mal_entries,
                row.mal_id,
                &self.anilist_entries,
                row.anilist_id,
            ),
        };
        primary_id
            .and_then(|id| primary.get(&id))
            .or_else(|| secondary_id.and_then(|id| secondary.get(&id)))
    }

    fn files_of(&self, id: u64) -> &[FileRow] {
        self.files.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }

    fn airing_of(&self, row: &SeriesRow) -> &[AiringRow] {
        row.anilist_id
            .and_then(|id| self.airing.get(&id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// The episodes on disk, sorted by number, as (file id, number).
    fn episodes_on_disk(&self, id: u64) -> Vec<&FileRow> {
        let mut eps: Vec<&FileRow> = self.files_of(id).iter().filter(|f| f.is_episode).collect();
        eps.sort_by(|a, b| {
            a.number
                .partial_cmp(&b.number)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.path.cmp(&b.path))
        });
        eps
    }

    pub fn card(&self, id: u64) -> Option<SeriesCard> {
        self.row(id).map(|row| self.card_of(row))
    }

    pub fn cards(&self) -> Vec<SeriesCard> {
        self.series.iter().map(|row| self.card_of(row)).collect()
    }

    fn card_of(&self, row: &SeriesRow) -> SeriesCard {
        let now = time::to_secs(self.now);
        let files = self.files_of(row.id);
        let episodes = self.episodes_on_disk(row.id);
        let airing = self.airing_of(row);
        let media = row.media.as_ref();

        let latest_downloaded = episodes.last().map_or(0.0, |f| f.number).max(0.0) as u32;
        let (latest_aired, next_scheduled) = aired_and_scheduled(airing, now);
        let status = media
            .and_then(|m| m.status.as_deref())
            .and_then(AiringStatus::from_provider);
        let total = media.and_then(|m| m.episodes);
        let entry = self.tracker(row);
        let watched = entry.map(|e| e.progress);
        let (total_episodes, total_is_estimate) = total_with_estimate(total, latest_aired, watched);

        let code = match row.kind {
            SeriesKind::Movie => Some("Movie".to_string()),
            SeriesKind::Show => episodes
                .last()
                .map(|f| labels::episode_code(f.season, f.number)),
        };

        SeriesCard {
            id: row.id,
            kind: row.kind,
            path: row.path.clone(),
            title: self.title_of(row),
            titles: Titles {
                romaji: media.and_then(|m| m.title_romaji.clone()),
                english: media.and_then(|m| m.title_english.clone()),
                native: media.and_then(|m| m.title_native.clone()),
                folder: row.folder_name.clone(),
            },
            poster: row.poster_path.clone(),
            format: media.and_then(|m| m.format.clone()),
            status,
            hidden: row.hidden,
            missing: row.missing,
            match_info: row.provider.map(|provider| MatchInfo {
                provider,
                anilist_id: row.anilist_id,
                mal_id: row.mal_id,
                tmdb_id: row.tmdb_id,
                tmdb_kind: row.tmdb_kind,
                confirmed: row.confirmed,
            }),
            episodes_on_disk: episodes.len() as u64,
            extras_on_disk: (files.len() - episodes.len()) as u64,
            total_episodes,
            total_is_estimate,
            code,
            watched,
            watched_state: watched_state(watched, total, latest_aired, latest_downloaded),
            strip: strip(
                watched,
                total,
                latest_aired,
                next_scheduled.map(|(n, _)| n),
                latest_downloaded,
                status,
            ),
            community_score: media
                .and_then(|m| m.average_score)
                .map(|s| f64::from(s) / 10.0),
            my_score: entry.and_then(|e| e.score),
            list_status: entry.and_then(|e| e.status),
            next_airing: next_scheduled.map(|(episode, at)| Airing {
                episode,
                at: time::from_secs(at),
            }),
            last_viewed_at: self.views.get(&row.id).map(|(_, at)| time::from_secs(*at)),
            latest_activity_at: time::from_secs(
                self.latest_activity(row, &episodes, airing, files, now),
            ),
        }
    }

    fn title_of(&self, row: &SeriesRow) -> String {
        let media = row.media.as_ref();
        titles::resolve(
            self.lang,
            media.and_then(|m| m.title_romaji.as_deref()),
            media.and_then(|m| m.title_english.as_deref()),
            &row.folder_name,
        )
    }

    /// The port of `getAiringSortInfo`: the shown episode's own past air
    /// date, else the latest past-aired on-disk episode's, else the newest
    /// file's mtime, else the day the series was added.
    fn latest_activity(
        &self,
        row: &SeriesRow,
        episodes: &[&FileRow],
        airing: &[AiringRow],
        files: &[FileRow],
        now: i64,
    ) -> i64 {
        let mut best_aired: Option<(i64, u32)> = None;
        for a in airing {
            let Some(at) = a.aired_at else { continue };
            if at > now || !episodes.iter().any(|f| f.number == f64::from(a.number)) {
                continue;
            }
            if best_aired.is_none_or(|(t, _)| at > t) {
                best_aired = Some((at, a.number));
            }
        }
        let highest_on_disk = episodes.last().map_or(0.0, |f| f.number).max(0.0) as u32;
        let shown = highest_on_disk.max(best_aired.map_or(0, |(_, e)| e));
        let shown_air = airing
            .iter()
            .find(|a| a.number == shown)
            .and_then(|a| a.aired_at);
        match shown_air.filter(|t| *t <= now) {
            Some(t) => t,
            None => match best_aired {
                Some((t, _)) => t,
                None => files.iter().map(|f| f.mtime).max().unwrap_or(row.added_at),
            },
        }
    }

    /// The metadata table's row: what is on disk against what is expected.
    pub fn metadata_row(&self, id: u64) -> Option<MetadataRow> {
        let row = self.row(id)?;
        let card = self.card_of(row);
        let media = row.media.as_ref();
        let have = card.episodes_on_disk;
        let expected = media
            .and_then(|m| m.episodes)
            .filter(|t| *t > 0)
            .map(u64::from)
            .or_else(|| match self.airing_of(row).len() as u64 {
                0 => None,
                n => Some(n),
            })
            .or(match row.kind {
                SeriesKind::Movie => Some(1),
                SeriesKind::Show => None,
            });
        let alt_title = media
            .and_then(|m| m.title_romaji.clone())
            .filter(|r| !r.trim().is_empty() && *r != card.title);
        Some(MetadataRow {
            series: card,
            alt_title,
            provider: row.provider,
            have,
            expected,
            extra_on_disk: expected.map_or(0, |e| have.saturating_sub(e)),
        })
    }

    /// The series page. Takes the connection because the character images
    /// and the recommendations are one query each, and neither belongs in a
    /// whole-library snapshot.
    pub fn detail(&self, conn: &Connection, id: u64) -> Result<Option<SeriesDetail>, CoreError> {
        let Some(row) = self.row(id) else {
            return Ok(None);
        };
        let card = self.card_of(row);
        let media = row.media.as_ref();
        let now = time::to_secs(self.now);
        let airing = self.airing_of(row);
        let (latest_aired, _) = aired_and_scheduled(airing, now);
        let files = self.files_of(id);
        let episodes = self.episodes_on_disk(id);
        let entry = self.tracker(row);

        let total = media.and_then(|m| m.episodes).filter(|t| *t > 0);
        let on_disk = episodes.len() as u64;
        let (progress_total, estimate) = match total {
            Some(t) => (Some(t), false),
            None if latest_aired > 0 => (Some(latest_aired.max(card.watched.unwrap_or(0))), true),
            None => (Some(on_disk as u32), false),
        };

        let disk: Vec<(u64, f64)> = episodes.iter().map(|f| (f.id, f.number)).collect();
        let next = next_up(&disk, self.last_completed_number(id), card.watched);

        // Files numbered past a known episode count are almost always
        // misnamed, duplicates or stray specials; only the main season is
        // judged, so a multi-season folder is never false-flagged.
        let unmatched: HashSet<u64> = match (row.kind, total) {
            (SeriesKind::Show, Some(t)) => episodes
                .iter()
                .filter(|f| f.season.is_none_or(|s| s <= 1) && f.number > f64::from(t))
                .map(|f| f.id)
                .collect(),
            _ => HashSet::new(),
        };

        let completed_keys: HashSet<&str> = self
            .completed
            .get(&id)
            .map(|rows| rows.iter().map(|(k, _)| k.as_str()).collect())
            .unwrap_or_default();
        let episode = |f: &FileRow| {
            let air = airing.iter().find(|a| f64::from(a.number) == f.number);
            Episode {
                file: f.id,
                number: f.number,
                season: f.season,
                code: labels::episode_code(f.season, f.number),
                title: air.and_then(|a| a.title.clone()),
                air_date: air.and_then(|a| a.aired_at).map(time::from_secs),
                path: f.path.clone(),
                sidecars: f.sidecars.clone(),
                resume: self.resume_of(id, &f.episode_key),
                watched: completed_keys.contains(f.episode_key.as_str())
                    || card.watched.is_some_and(|w| f.number <= f64::from(w)),
                next_up: next == Some(f.id),
            }
        };

        let mut extras: Vec<&FileRow> = files.iter().filter(|f| !f.is_episode).collect();
        extras.sort_by(|a, b| {
            extra_order(a.extra_kind)
                .cmp(&extra_order(b.extra_kind))
                .then_with(|| a.extra_index.unwrap_or(0).cmp(&b.extra_index.unwrap_or(0)))
                .then_with(|| a.label.cmp(&b.label))
                .then_with(|| a.path.cmp(&b.path))
        });

        Ok(Some(SeriesDetail {
            banner: row.banner_path.clone(),
            synopsis: media
                .and_then(|m| m.description.clone())
                .unwrap_or_default(),
            year: media.and_then(|m| m.year),
            studio: media.and_then(|m| m.studio.clone()),
            genres: media.map(|m| m.genres.clone()).unwrap_or_default(),
            tags: media.map(|m| tags_of(&m.tags)).unwrap_or_default(),
            rewatch_count: entry.map(|e| e.repeat).filter(|r| *r > 0),
            site_url: media.and_then(|m| m.site_url.clone()),
            progress: ProgressLine {
                watched: card.watched,
                total: progress_total,
                estimate,
                on_disk,
            },
            next_up: next,
            episodes: episodes
                .iter()
                .filter(|f| !unmatched.contains(&f.id))
                .map(|f| episode(f))
                .collect(),
            extras: extras
                .iter()
                .map(|f| {
                    let kind = f.extra_kind.unwrap_or(ExtraKind::Other);
                    Extra {
                        file: f.id,
                        kind,
                        code: labels::extra_code_with_index(kind, f.extra_index),
                        label: f.label.clone(),
                        path: f.path.clone(),
                        sidecars: f.sidecars.clone(),
                        resume: self.resume_of(id, &f.episode_key),
                    }
                })
                .collect(),
            unmatched_files: episodes
                .iter()
                .filter(|f| unmatched.contains(&f.id))
                .map(|f| episode(f))
                .collect(),
            characters: self.characters(conn, media)?,
            recommendations: self.recommendations(conn, row.anilist_id)?,
            has_graph: row
                .anilist_id
                .is_some_and(|a| self.graph_seeds.contains(&a)),
            card,
        }))
    }

    fn resume_of(&self, series: u64, key: &str) -> Option<ResumePoint> {
        self.resume
            .get(&(series, key.to_string()))
            .map(|(position, duration)| ResumePoint {
                position: *position,
                duration: *duration,
            })
    }

    /// The number of the most recently completed episode, resolved through
    /// the file that carries its key, or the key itself when no file does.
    fn last_completed_number(&self, id: u64) -> Option<f64> {
        let (key, _) = self.completed.get(&id)?.iter().max_by_key(|(_, at)| *at)?;
        self.files_of(id)
            .iter()
            .find(|f| f.is_episode && f.episode_key == *key)
            .map(|f| f.number)
            .or_else(|| key.parse::<f64>().ok())
    }

    /// The top characters, their portraits resolved through the image cache
    /// in one query. A portrait that has not been fetched yet is None.
    fn characters(
        &self,
        conn: &Connection,
        media: Option<&MediaRow>,
    ) -> Result<Vec<Person>, CoreError> {
        let Some(media) = media else {
            return Ok(Vec::new());
        };
        let people = json_array(&media.characters);
        if people.is_empty() {
            return Ok(Vec::new());
        }
        let urls: Vec<String> = people
            .iter()
            .filter_map(|c| {
                c.get("image_url")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .collect();
        let mut cached: HashMap<String, String> = HashMap::new();
        if !urls.is_empty() {
            let sql = format!(
                "SELECT url, path FROM images WHERE url IN ({})",
                placeholders(urls.len())
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_from_iter(urls.iter()), |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (url, path) = row?;
                cached.insert(url, path);
            }
        }
        Ok(people
            .iter()
            .map(|c| Person {
                name: c
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                image: c
                    .get("image_url")
                    .and_then(Value::as_str)
                    .and_then(|u| cached.get(u))
                    .map(|p| {
                        Path::new(&self.images_dir)
                            .join(p)
                            .to_string_lossy()
                            .into_owned()
                    }),
                role: c
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
            .collect())
    }

    /// AniList's recommendations for this series, in AniList's own order,
    /// each one told whether the library owns it and where it sits on the
    /// list.
    fn recommendations(
        &self,
        conn: &Connection,
        anilist_id: Option<u64>,
    ) -> Result<Vec<Recommendation>, CoreError> {
        let Some(anilist_id) = anilist_id else {
            return Ok(Vec::new());
        };
        let mut stmt = conn.prepare(
            "SELECT r.recommended_id, m.title_romaji, m.title_english, i.path,
                    (SELECT s.id FROM series s WHERE s.anilist_id = m.id ORDER BY s.id LIMIT 1),
                    (SELECT ta.status FROM tracker_entries ta WHERE ta.tracker = 'anilist' AND ta.media_id = m.id),
                    (SELECT tm.status FROM tracker_entries tm WHERE tm.tracker = 'mal' AND tm.media_id = m.mal_id)
             FROM recommendations r
             JOIN anilist_media m ON m.id = r.recommended_id
             LEFT JOIN images i ON i.url = m.cover_url
             WHERE r.anilist_id = ?1
             ORDER BY r.rank",
        )?;
        let rows = stmt.query_map([anilist_id as i64], |r| {
            let romaji: Option<String> = r.get(1)?;
            let english: Option<String> = r.get(2)?;
            let anilist_status: Option<String> = r.get(5)?;
            let mal_status: Option<String> = r.get(6)?;
            let (main, other) = match self.main_tracker {
                Tracker::Anilist => (anilist_status, mal_status),
                Tracker::Mal => (mal_status, anilist_status),
            };
            Ok(Recommendation {
                anilist_id: r.get::<_, i64>(0)? as u64,
                title: titles::resolve(self.lang, romaji.as_deref(), english.as_deref(), ""),
                poster: r.get::<_, Option<String>>(3)?.map(|p| {
                    Path::new(&self.images_dir)
                        .join(p)
                        .to_string_lossy()
                        .into_owned()
                }),
                owned: r.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                list_status: main.or(other).and_then(|s| ListStatus::from_column(&s)),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

/// The highest episode that has aired, and the earliest one still to come.
fn aired_and_scheduled(airing: &[AiringRow], now: i64) -> (u32, Option<(u32, i64)>) {
    let mut latest_aired = 0;
    let mut next_scheduled: Option<(u32, i64)> = None;
    for a in airing {
        let Some(at) = a.aired_at else { continue };
        if at <= now {
            latest_aired = latest_aired.max(a.number);
        } else if next_scheduled.is_none_or(|(_, t)| at < t) {
            next_scheduled = Some((a.number, at));
        }
    }
    (latest_aired, next_scheduled)
}

/// Openings, then endings, previews, specials and the rest.
fn extra_order(kind: Option<ExtraKind>) -> u8 {
    match kind.unwrap_or(ExtraKind::Other) {
        ExtraKind::Op => 0,
        ExtraKind::Ed => 1,
        ExtraKind::Pv => 2,
        ExtraKind::Sp => 3,
        ExtraKind::Other => 4,
    }
}

fn tags_of(raw: &Value) -> Vec<Tag> {
    json_array(raw)
        .iter()
        .map(|t| Tag {
            name: t
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            rank: t.get("rank").and_then(Value::as_u64).unwrap_or(0) as u32,
            spoiler: t.get("spoiler").and_then(Value::as_bool).unwrap_or(false),
            adult: t.get("adult").and_then(Value::as_bool).unwrap_or(false),
        })
        .collect()
}

/// What every job uses to build a `SeriesChanged` batch: the cards for
/// exactly these series, in the order asked for.
pub fn cards_for(
    conn: &Connection,
    images_dir: &Path,
    ids: &[u64],
) -> Result<Vec<SeriesCard>, CoreError> {
    let snap = Snapshot::load(conn, images_dir, time::now(), Some(ids))?;
    Ok(ids.iter().filter_map(|id| snap.card(*id)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 0.0001, "{a} vs {b}");
    }

    #[test]
    fn strip_matches_electrons_cases() {
        let s = strip(
            Some(5),
            Some(12),
            8,
            Some(9),
            0,
            Some(AiringStatus::Releasing),
        );
        near(s.watched, 5.0 / 12.0);
        near(s.aired_unwatched, 8.0 / 12.0);
        near(s.unknown, 0.0);
        let s = strip(Some(5), Some(12), 12, None, 0, Some(AiringStatus::Finished));
        near(s.aired_unwatched, 0.0);
        let s = strip(
            Some(3),
            Some(12),
            12,
            None,
            0,
            Some(AiringStatus::Releasing),
        );
        near(s.aired_unwatched, 0.0);
        let s = strip(Some(5), None, 8, None, 0, Some(AiringStatus::Releasing));
        near(s.watched, 5.0 / 8.0 * 0.85);
        near(s.aired_unwatched, 0.85);
        near(s.unknown, 0.15);
        let s = strip(Some(5), None, 0, None, 0, None);
        near(s.watched, 0.85);
        near(s.aired_unwatched, 0.0);
        near(s.unknown, 0.15);
        let s = strip(Some(2), None, 10, None, 0, Some(AiringStatus::Finished));
        near(s.aired_unwatched, 0.0);
        near(s.unknown, 0.15);
        let s = strip(None, Some(12), 8, None, 0, Some(AiringStatus::Releasing));
        near(s.watched, 0.0);
        near(s.aired_unwatched, 0.0);
        let s = strip(
            Some(6),
            Some(12),
            0,
            Some(9),
            0,
            Some(AiringStatus::Releasing),
        );
        near(s.aired_unwatched, 8.0 / 12.0);
        let s = strip(
            Some(0),
            Some(12),
            0,
            Some(5),
            0,
            Some(AiringStatus::Releasing),
        );
        near(s.aired_unwatched, 4.0 / 12.0);
        let s = strip(Some(2), Some(12), 0, None, 7, Some(AiringStatus::Releasing));
        near(s.aired_unwatched, 7.0 / 12.0);
        let s = strip(Some(15), Some(12), 0, None, 0, Some(AiringStatus::Finished));
        near(s.watched, 1.0);
        let s = strip(None, None, 0, None, 0, None);
        near(s.watched, 0.0);
        near(s.aired_unwatched, 0.0);
        near(s.unknown, 0.0);
        let s = strip(
            Some(0),
            Some(12),
            0,
            Some(1),
            0,
            Some(AiringStatus::NotYetReleased),
        );
        near(s.watched, 0.0);
        near(s.aired_unwatched, 0.0);
    }

    #[test]
    fn watched_state_rules() {
        assert_eq!(watched_state(None, Some(12), 8, 0), WatchedState::Unknown);
        assert_eq!(watched_state(Some(4), Some(12), 8, 0), WatchedState::Behind);
        assert_eq!(
            watched_state(Some(8), Some(12), 8, 0),
            WatchedState::CaughtUp
        );
        assert_eq!(watched_state(Some(4), Some(12), 0, 8), WatchedState::Behind);
        assert_eq!(
            watched_state(Some(12), Some(12), 8, 0),
            WatchedState::CaughtUp
        );
        assert_eq!(watched_state(Some(3), None, 0, 0), WatchedState::Unknown);
        assert_eq!(watched_state(Some(3), None, 5, 0), WatchedState::Behind);
    }

    #[test]
    fn total_estimate_rule() {
        assert_eq!(total_with_estimate(Some(12), 8, Some(5)), (Some(12), false));
        assert_eq!(total_with_estimate(None, 8, Some(5)), (Some(8), true));
        assert_eq!(total_with_estimate(None, 8, Some(9)), (Some(9), true));
        assert_eq!(total_with_estimate(None, 0, Some(5)), (None, false));
        assert_eq!(total_with_estimate(Some(0), 0, None), (None, false));
    }

    #[test]
    fn next_up_rules() {
        let disk = vec![(10u64, 1.0), (11, 2.0), (12, 3.0), (13, 5.0)];
        assert_eq!(next_up(&disk, Some(2.0), Some(1)), Some(12));
        assert_eq!(next_up(&disk, Some(3.0), None), Some(13));
        assert_eq!(next_up(&disk, None, Some(0)), Some(10));
        assert_eq!(next_up(&disk, None, None), Some(10));
        assert_eq!(next_up(&disk, Some(5.0), Some(5)), None);
        assert_eq!(next_up(&[], Some(1.0), None), None);
    }
}
