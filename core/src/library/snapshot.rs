//! The handful of queries one read makes, and the rows they come back as.
//!
//! Everything a card or a detail is built from is loaded here, once, with
//! an optional scope so a job's batch never reads the whole library. The
//! two files beside this one turn these rows into the contract's records:
//! `cards` for a card, `detail` for a page.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::SystemTime;

use rusqlite::{Connection, params_from_iter};
use serde_json::Value;

use crate::contract::*;
use crate::library::sidecar::sidecars_of;
use crate::prefs;
use crate::store::sql::placeholders;

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
    pub(super) index: HashMap<u64, usize>,
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

    pub(super) fn files_of(&self, id: u64) -> &[FileRow] {
        self.files.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub(super) fn airing_of(&self, row: &SeriesRow) -> &[AiringRow] {
        row.anilist_id
            .and_then(|id| self.airing.get(&id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// The episodes on disk, sorted by number, as (file id, number).
    pub(super) fn episodes_on_disk(&self, id: u64) -> Vec<&FileRow> {
        let mut eps: Vec<&FileRow> = self.files_of(id).iter().filter(|f| f.is_episode).collect();
        eps.sort_by(|a, b| {
            a.number
                .partial_cmp(&b.number)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.path.cmp(&b.path))
        });
        eps
    }
}
