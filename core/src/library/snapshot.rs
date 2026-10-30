//! The handful of queries one read makes, and the rows they come back as.
//!
//! Everything a card or a detail is built from is loaded here, once, with
//! an optional scope so a job's batch never reads the whole library. The
//! two files beside this one turn these rows into the contract's records:
//! `cards` for a card, `detail` for a page.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;
use std::time::SystemTime;

use rusqlite::{Connection, params_from_iter};
use serde_json::Value;

use crate::contract::*;
use crate::library::cards::aired_and_scheduled;
use crate::library::sidecar::sidecars_of;
use crate::net::tvmaze;
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
    pub tvmaze_id: Option<u64>,
    pub confirmed: bool,
    pub track_choice: Option<String>,
    /// The episode count the person set by hand (#239), in force or not.
    pub episodes_set: Option<u32>,
    /// Whether that count is in force (`apply_episodes_set`).
    pub episodes_set_applied: bool,
    /// The record behind the match: the AniList media, or for a TVmaze
    /// match the show, read into the same shape so every reader draws
    /// either without asking which it is.
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
    /// AniList's raw season word (WINTER, SPRING, SUMMER, FALL); None for
    /// TVmaze, whose shows carry only a premiere date.
    pub season: Option<String>,
    /// The start date as `YYYY-MM-DD` (`format_date` fills a missing month
    /// or day with 01); TVmaze's `premiered` on the show path.
    pub start_date: Option<String>,
    pub episodes: Option<u32>,
    /// Minutes an episode runs (a film's runtime): AniList's `duration`,
    /// TVmaze's `averageRuntime` on the show path.
    pub duration: Option<u32>,
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
    /// Read this way by the user's quick fix (#170), not by its name.
    pub fixed: bool,
    /// The file's size in bytes, as the last scan found it.
    pub size: u64,
}

pub struct AiringRow {
    pub number: u32,
    pub title: Option<String>,
    pub aired_at: Option<i64>,
}

/// The earliest episode still to come. The season is TVmaze's, whose
/// numbers restart every season; an AniList schedule carries none.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NextEpisode {
    pub season: Option<u32>,
    pub number: u32,
    pub at: i64,
}

pub struct TrackerRow {
    pub progress: u32,
    pub status: Option<ListStatus>,
    pub score: Option<f64>,
    pub repeat: u32,
}

/// The IMDb title a series' match resolves to, and its rating when IMDb's
/// dump has one: in tenths, the way the table holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImdbPick {
    pub id: u64,
    pub rating: Option<u32>,
    pub votes: Option<u64>,
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
    /// By TVmaze show id, then by season. TVmaze numbers its episodes
    /// within a season, so a file is looked up by both.
    pub tv_seasons: HashMap<u64, BTreeMap<u32, Vec<AiringRow>>>,
    pub anilist_entries: HashMap<u64, TrackerRow>,
    pub mal_entries: HashMap<u64, TrackerRow>,
    pub views: HashMap<u64, (String, i64)>,
    pub completed: HashMap<u64, Vec<(String, i64)>>,
    pub resume: HashMap<(u64, String), (f64, f64)>,
    /// The outro's start by (series, episode key), for the episodes whose
    /// skip windows are cached and name one.
    pub outro: HashMap<(u64, String), f64>,
    /// AniList ids with at least one relation row.
    pub graph_seeds: HashSet<u64>,
    /// By series id, only for a series whose match leads to an IMDb title.
    pub imdb: HashMap<u64, ImdbPick>,
    /// The franchise each grouped series belongs to, by series id. Always
    /// computed over the whole library, scoped load or not, so a job's
    /// `SeriesChanged` card carries the same tag a list does.
    pub franchises: HashMap<u64, FranchiseTag>,
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

/// A TVmaze show read into the media row every reader already draws:
/// columns 35 to 45 and 49 of the series query. TVmaze's own status
/// becomes the AniList word for it, its 0 to 10 rating the 0 to 100 the
/// row stores, its network the studio and its average runtime the
/// duration.
fn show_media(r: &rusqlite::Row) -> rusqlite::Result<MediaRow> {
    let status: Option<String> = r.get(37)?;
    let premiered: Option<String> = r.get(38)?;
    let rating: Option<f64> = r.get(39)?;
    Ok(MediaRow {
        title_romaji: r.get(36)?,
        title_english: None,
        title_native: None,
        // TVmaze carries television and nothing else.
        format: Some("TV".to_string()),
        status: status
            .as_deref()
            .and_then(tvmaze::airing_status)
            .map(|s| s.anilist_word().to_string()),
        year: tvmaze::year_of(premiered.as_deref()),
        season: None,
        start_date: premiered,
        episodes: None,
        duration: r
            .get::<_, Option<i64>>(49)?
            .and_then(|v| u32::try_from(v).ok()),
        description: r.get(42)?,
        average_score: rating
            .filter(|v| v.is_finite() && *v >= 0.0)
            .map(|v| (v * 10.0).round() as u32),
        genres: json_strings(&r.get::<_, String>(40)?),
        studio: r.get(41)?,
        tags: Value::Null,
        characters: Value::Null,
        site_url: r.get(43)?,
        cover_url: r.get(44)?,
        banner_url: r.get(45)?,
    })
}

/// The IMDb title behind each series' current match, one row per series
/// that has one. A TVmaze show names its own. An AniList match goes
/// through Wikidata's links for its AniList id, and through its MAL id's
/// only when Wikidata links the AniList id to nothing; a MAL match takes
/// its MAL id's. A TMDB match and an unmatched series have none. Where one
/// id links to several titles the most voted wins, a title with no rating
/// counting as fewer votes than any rated one, and the lower id breaks a
/// tie so the pick never wavers between reads.
///
/// `{scope}` is the series filter, nothing when the whole library is read.
const IMDB_SQL: &str = "WITH scoped AS (
        SELECT id, provider, anilist_id, mal_id, tvmaze_id FROM series {scope}
    ),
    candidates (series, fallback, imdb) AS (
        SELECT s.id, 0, CAST(substr(t.imdb, 3) AS INTEGER)
        FROM scoped s JOIN tvmaze_shows t ON t.id = s.tvmaze_id
        WHERE s.provider = 'tvmaze' AND t.imdb GLOB 'tt[0-9]*' AND substr(t.imdb, 3) NOT GLOB '*[^0-9]*'
        UNION ALL
        SELECT s.id, 0, l.imdb
        FROM scoped s JOIN imdb_links l ON l.provider = 'anilist' AND l.provider_id = s.anilist_id
        WHERE s.provider = 'anilist'
        UNION ALL
        SELECT s.id, s.provider = 'anilist', l.imdb
        FROM scoped s JOIN imdb_links l ON l.provider = 'mal' AND l.provider_id = s.mal_id
        WHERE s.provider IN ('anilist', 'mal')
    )
    SELECT series, imdb, rating, votes FROM (
        SELECT c.series, c.imdb, r.rating, r.votes,
               row_number() OVER (
                   PARTITION BY c.series
                   ORDER BY c.fallback, coalesce(r.votes, -1) DESC, c.imdb
               ) AS pick
        FROM candidates c LEFT JOIN imdb_ratings r ON r.id = c.imdb
        WHERE c.imdb > 0
    )
    WHERE pick = 1";

/// `IMDB_SQL` over the whole library with `scope` None, or over exactly
/// the named series. The cards read it, and the IMDb refresh reads it
/// either side of a replace to learn which cards it changed.
pub(crate) fn imdb_picks(
    conn: &Connection,
    scope: Option<&[u64]>,
) -> Result<HashMap<u64, ImdbPick>, CoreError> {
    if scope.is_some_and(<[u64]>::is_empty) {
        return Ok(HashMap::new());
    }
    let filter = match scope {
        Some(ids) => format!("WHERE id IN ({})", placeholders(ids.len())),
        None => String::new(),
    };
    let mut stmt = conn.prepare(&IMDB_SQL.replace("{scope}", &filter))?;
    let rows = stmt.query_map(params_from_iter(ids_as_i64(scope.unwrap_or(&[]))), |r| {
        Ok((
            r.get::<_, i64>(0)? as u64,
            ImdbPick {
                id: r.get::<_, i64>(1)? as u64,
                rating: r.get::<_, Option<i64>>(2)?.map(|v| v as u32),
                votes: r.get::<_, Option<i64>>(3)?.map(|v| v as u64),
            },
        ))
    })?;
    Ok(rows.collect::<Result<HashMap<_, _>, _>>()?)
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
            tv_seasons: HashMap::new(),
            anilist_entries: HashMap::new(),
            mal_entries: HashMap::new(),
            views: HashMap::new(),
            completed: HashMap::new(),
            resume: HashMap::new(),
            outro: HashMap::new(),
            graph_seeds: HashSet::new(),
            imdb: HashMap::new(),
            franchises: HashMap::new(),
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

        let show_ids: Vec<u64> = snap.series.iter().filter_map(|r| r.tvmaze_id).collect();
        snap.load_files(conn, scoped_series)?;
        snap.load_airing(conn, scope.map(|_| anilist_ids.as_slice()))?;
        snap.load_tv_episodes(conn, scope.map(|_| show_ids.as_slice()))?;
        snap.fill_tvmaze_totals();
        snap.apply_episodes_set();
        snap.load_tracker_entries(conn, scope.map(|_| media_ids.as_slice()))?;
        snap.load_history(conn, scoped_series)?;
        snap.load_graph_seeds(conn, scope.map(|_| anilist_ids.as_slice()))?;
        snap.imdb = imdb_picks(conn, scoped_series)?;
        snap.franchises = crate::franchise::groups::load(conn, snap.lang)?;
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
                    m.cover_url, m.banner_url, ip.path, ib.path,
                    s.tvmaze_id, t.id, t.name, t.status, t.premiered, t.rating, t.genres, t.network, t.summary,
                    t.url, t.poster_url, t.background_url, m.season, m.start_date, m.duration,
                    t.average_runtime, s.episodes_set
             FROM series s
             LEFT JOIN anilist_media m ON m.id = s.anilist_id
             LEFT JOIN tvmaze_shows t ON t.id = s.tvmaze_id AND s.provider = 'tvmaze'
             LEFT JOIN images ip ON ip.url = CASE WHEN t.id IS NOT NULL THEN t.poster_url ELSE m.cover_url END
             LEFT JOIN images ib ON ib.url = CASE WHEN t.id IS NOT NULL THEN t.background_url ELSE m.banner_url END
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
            let show = match r.get::<_, Option<i64>>(35)? {
                None => None,
                Some(_) => Some(show_media(r)?),
            };
            let media = match r.get::<_, Option<i64>>(15)? {
                _ if show.is_some() => show,
                None => None,
                Some(_) => Some(MediaRow {
                    title_romaji: r.get(16)?,
                    title_english: r.get(17)?,
                    title_native: r.get(18)?,
                    format: r.get(19)?,
                    status: r.get(20)?,
                    year: r.get::<_, Option<i64>>(21)?.map(|v| v as u32),
                    season: r.get(46)?,
                    start_date: r.get(47)?,
                    episodes: r.get::<_, Option<i64>>(22)?.map(|v| v as u32),
                    duration: r
                        .get::<_, Option<i64>>(48)?
                        .and_then(|v| u32::try_from(v).ok()),
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
                tvmaze_id: r.get::<_, Option<i64>>(34)?.map(|v| v as u64),
                confirmed: r.get::<_, i64>(13)? != 0,
                track_choice: r.get(14)?,
                episodes_set: r
                    .get::<_, Option<i64>>(50)?
                    .and_then(|v| u32::try_from(v).ok())
                    .filter(|n| *n > 0),
                episodes_set_applied: false,
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
            "SELECT id, series_id, path, mtime, kind, number, season, extra_kind, extra_index, label, episode_key, sidecars,
                    EXISTS (SELECT 1 FROM file_fixes x WHERE x.path = files.path), size
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
                fixed: r.get(12)?,
                size: r.get::<_, i64>(13)?.max(0) as u64,
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

    /// The episodes of every TVmaze show the scope's series are matched
    /// to, grouped by season.
    fn load_tv_episodes(
        &mut self,
        conn: &Connection,
        scope: Option<&[u64]>,
    ) -> Result<(), CoreError> {
        if scope.is_some_and(<[u64]>::is_empty) {
            return Ok(());
        }
        let where_sql = match scope {
            Some(ids) => format!("show_id IN ({})", placeholders(ids.len())),
            None => "show_id IN (SELECT tvmaze_id FROM series WHERE provider = 'tvmaze' AND tvmaze_id IS NOT NULL)"
                .to_string(),
        };
        let sql = format!(
            // TVmaze names an episode it knows nothing about yet "TBA",
            // which is a placeholder, not a title.
            "SELECT show_id, season, number, NULLIF(name, 'TBA'), airstamp FROM tvmaze_episodes WHERE {where_sql} ORDER BY season, number"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(ids_as_i64(scope.unwrap_or(&[]))), |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, i64>(1)? as u32,
                AiringRow {
                    number: r.get::<_, i64>(2)? as u32,
                    title: r.get(3)?,
                    aired_at: r.get(4)?,
                },
            ))
        })?;
        for row in rows {
            let (show, season, airing) = row?;
            self.tv_seasons
                .entry(show)
                .or_default()
                .entry(season)
                .or_default()
                .push(airing);
        }
        Ok(())
    }

    /// A TVmaze show publishes no episode count, and its list is the
    /// whole run rather than one season's. Once a show has ended, every
    /// season the folder holds is complete, so the total is their lengths
    /// added up, which is what the folder's own count is measured against;
    /// while it runs, the list is only what is announced so far, which is
    /// the aired estimate's job rather than a total's.
    fn fill_tvmaze_totals(&mut self) {
        let mut totals: Vec<(usize, u32)> = Vec::new();
        for (i, row) in self.series.iter().enumerate() {
            let finished = row.media.as_ref().is_some_and(|m| {
                m.status.as_deref().and_then(AiringStatus::from_provider)
                    == Some(AiringStatus::Finished)
            });
            if row.provider == Some(Provider::Tvmaze) && finished {
                let n = self.tv_span(row).count() as u32;
                if n > 0 {
                    totals.push((i, n));
                }
            }
        }
        for (i, n) in totals {
            if let Some(m) = self.series[i].media.as_mut() {
                m.episodes = Some(n);
            }
        }
    }

    /// A count the person set by hand (#239) stands in while the record names
    /// none of its own, so nothing past it is predicted or scheduled, until an
    /// episode past it airs or a file of one lands: then what really happened
    /// wins and the series reads as before. A record that publishes its own
    /// count always wins. TVmaze shows are left alone: their counts are per
    /// season and a hand-set one has no season to belong to.
    fn apply_episodes_set(&mut self) {
        let now = crate::time::to_secs(self.now);
        let mut applied: Vec<(usize, u32)> = Vec::new();
        for (i, row) in self.series.iter().enumerate() {
            let Some(set) = row.episodes_set else {
                continue;
            };
            let own = row.media.as_ref().map(|m| m.episodes);
            if row.provider == Some(Provider::Tvmaze) || own != Some(None) {
                continue;
            }
            // Only what really came out proves the count wrong: a date the
            // schedule names ahead is a plan, and plans run long
            let aired = self
                .airing_of(row)
                .iter()
                .filter(|a| a.aired_at.is_some_and(|at| at <= now))
                .map(|a| a.number)
                .max()
                .unwrap_or(0);
            let on_disk = self
                .files_of(row.id)
                .iter()
                .filter(|f| f.is_episode && f.number >= 1.0 && f.number.fract() == 0.0)
                .map(|f| f.number as u32)
                .max()
                .unwrap_or(0);
            if aired.max(on_disk) <= set {
                applied.push((i, set));
            }
        }
        for (i, n) in applied {
            let row = &mut self.series[i];
            row.episodes_set_applied = true;
            if let Some(m) = row.media.as_mut() {
                m.episodes = Some(n);
            }
            // Nothing past the count is scheduled either
            if let Some(list) = row.anilist_id.and_then(|id| self.airing.get_mut(&id)) {
                list.retain(|a| a.number <= n);
            }
        }
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

        let mut stmt = conn.prepare(&format!(
            "SELECT series_id, episode_key, windows FROM skip_windows{where_sql}"
        ))?;
        let rows = stmt.query_map(params_from_iter(ids.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (series_id, key, windows) = row?;
            let windows: Vec<SkipWindow> = serde_json::from_str(&windows).unwrap_or_default();
            if let Some(o) = windows.iter().find(|w| w.kind == SkipKind::Outro) {
                self.outro.insert((series_id, key), o.start);
            }
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

    /// The episodes a card matches its files against by number. For an
    /// AniList match that is the media's schedule; for a TVmaze match it is
    /// one season of the show, the one `card_season` names, since TVmaze
    /// numbers its episodes within a season.
    pub(crate) fn airing_of(&self, row: &SeriesRow) -> &[AiringRow] {
        if let Some(season) = self.card_season(row) {
            return self.tv_season(row, season).unwrap_or(&[]);
        }
        row.anilist_id
            .and_then(|id| self.airing.get(&id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// The season a TVmaze card is about: the highest one on disk, since
    /// that is the one being watched, with a file that names no season
    /// read as season 1, and season 1 for a folder with no episodes yet.
    /// None for every other provider, whose numbers carry no season.
    pub(crate) fn card_season(&self, row: &SeriesRow) -> Option<u32> {
        (row.provider == Some(Provider::Tvmaze))
            .then(|| self.seasons_on_disk(row).last().copied().unwrap_or(1))
    }

    /// One season of a TVmaze match's show, as TVmaze lists it.
    fn tv_season(&self, row: &SeriesRow, season: u32) -> Option<&[AiringRow]> {
        row.tvmaze_id
            .and_then(|id| self.tv_seasons.get(&id))
            .and_then(|seasons| seasons.get(&season))
            .map(Vec::as_slice)
    }

    /// The seasons a TVmaze folder holds episodes of, a file naming no
    /// season read as season 1, and season 1 alone for a folder with no
    /// episodes yet.
    fn seasons_on_disk(&self, row: &SeriesRow) -> BTreeSet<u32> {
        let mut seasons: BTreeSet<u32> = self
            .files_of(row.id)
            .iter()
            .filter(|f| f.is_episode)
            .map(|f| f.season.unwrap_or(1))
            .collect();
        if seasons.is_empty() {
            seasons.insert(1);
        }
        seasons
    }

    /// Every episode of every season a TVmaze folder holds: what its count
    /// on disk is measured against.
    fn tv_span<'a>(&'a self, row: &'a SeriesRow) -> impl Iterator<Item = &'a AiringRow> + 'a {
        self.seasons_on_disk(row)
            .into_iter()
            .filter_map(move |s| self.tv_season(row, s))
            .flatten()
    }

    /// How far a series has aired, and the next episode still to come.
    ///
    /// For an AniList match, the highest episode aired and the earliest
    /// one scheduled. For a TVmaze match, the count aired across every
    /// season on disk, which is the number the folder's count is measured
    /// in, and the earliest episode still to come in any season, so an
    /// announced next season is what is next.
    pub(crate) fn aired_and_next(&self, row: &SeriesRow, now: i64) -> (u32, Option<NextEpisode>) {
        if row.provider != Some(Provider::Tvmaze) {
            return aired_and_scheduled(self.airing_of(row), now);
        }
        let aired = self
            .tv_span(row)
            .filter(|a| a.aired_at.is_some_and(|at| at <= now))
            .count() as u32;
        let next = row
            .tvmaze_id
            .and_then(|id| self.tv_seasons.get(&id))
            .into_iter()
            .flat_map(|seasons| {
                seasons
                    .iter()
                    .flat_map(|(season, eps)| eps.iter().map(move |a| (*season, a)))
            })
            .filter_map(|(season, a)| {
                a.aired_at.filter(|at| *at > now).map(|at| NextEpisode {
                    season: Some(season),
                    number: a.number,
                    at,
                })
            })
            .min_by_key(|n| n.at);
        (aired, next)
    }

    /// How many episodes the provider lists for what the folder holds: an
    /// AniList media's schedule, or every season on disk of a TVmaze show.
    pub(crate) fn episodes_listed(&self, row: &SeriesRow) -> usize {
        if row.provider == Some(Provider::Tvmaze) {
            return self.tv_span(row).count();
        }
        self.airing_of(row).len()
    }

    /// A file numbered past the end of the list it belongs to: almost
    /// always misnamed, a duplicate or a stray special. `total` is the
    /// series' known episode count. An AniList count is one season's, so
    /// only the main season is judged and a multi-season folder is never
    /// false-flagged; a TVmaze file is judged against its own season's
    /// length, when TVmaze lists that season at all.
    pub(crate) fn past_the_end(&self, row: &SeriesRow, file: &FileRow, total: u32) -> bool {
        if row.provider == Some(Provider::Tvmaze) {
            return self
                .tv_season(row, file.season.unwrap_or(1))
                .is_some_and(|s| !s.is_empty() && file.number > s.len() as f64);
        }
        file.season.is_none_or(|s| s <= 1) && file.number > f64::from(total)
    }

    /// The episodes on disk a card matches against `airing_of` by number:
    /// every one, or for a TVmaze card the card's season alone.
    pub(crate) fn card_episodes<'a>(
        &self,
        row: &SeriesRow,
        episodes: &[&'a FileRow],
    ) -> Vec<&'a FileRow> {
        let season = self.card_season(row);
        episodes
            .iter()
            .copied()
            .filter(|f| season.is_none_or(|s| f.season.unwrap_or(1) == s))
            .collect()
    }

    /// One file's own episode in the provider's list: by number for an
    /// AniList match, by season and number for a TVmaze one.
    pub(crate) fn episode_airing(&self, row: &SeriesRow, file: &FileRow) -> Option<&AiringRow> {
        let rows: &[AiringRow] = if row.provider == Some(Provider::Tvmaze) {
            self.tv_season(row, file.season.unwrap_or(1)).unwrap_or(&[])
        } else {
            self.airing_of(row)
        };
        rows.iter().find(|a| f64::from(a.number) == file.number)
    }

    /// The episodes on disk in season then number order, a file naming no
    /// season read as season 1, so a folder holding S01E10 and S02E08 ends
    /// on S02E08 and the card's code, latest download and next up follow
    /// the latest season.
    pub(super) fn episodes_on_disk(&self, id: u64) -> Vec<&FileRow> {
        let mut eps: Vec<&FileRow> = self.files_of(id).iter().filter(|f| f.is_episode).collect();
        eps.sort_by(|a, b| {
            season_of(a)
                .cmp(&season_of(b))
                .then_with(|| a.number.partial_cmp(&b.number).unwrap_or(Ordering::Equal))
                .then_with(|| a.path.cmp(&b.path))
        });
        eps
    }
}

/// The season a file sorts and keys under: its own, else season 1.
pub(crate) fn season_of(f: &FileRow) -> u32 {
    f.season.unwrap_or(1)
}

impl Snapshot {}
