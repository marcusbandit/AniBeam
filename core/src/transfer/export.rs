//! The Export job: the library, and with the checkbox ticked everything
//! private with it, as one `anibeam-export` document at the current
//! version.
//!
//! Plain text throughout, tokens included, by decision: the checkbox is
//! the only guard and there is no encryption. What is left out is what can
//! be rebuilt, the franchise store, the image cache and the progress
//! caches among them.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use rusqlite::Connection;
use serde_json::Value;

use crate::atomic_file;
use crate::contract::*;
use crate::core::Core;
use crate::jobs::Finished;
use crate::library::{classifier, titles};
use crate::paths::under;
use crate::prefs;
use crate::time;
use crate::trackers::accounts::{self, access_key, refresh_key, secret_key};
use crate::transfer::format::{
    self, Account, Accounts, Completed, Document, ExportedBy, FixEntry, History, ListEntry,
    MatchEntry, PrefsEntry, ResumeEntry, SeriesEntry, SortEntry, SourceEntry, TmdbName,
    TrackerName, TvmazeName, View,
};

/// One series as the tables hold it, before it becomes a document entry.
struct Row {
    id: u64,
    kind: SeriesKind,
    path: String,
    folder_name: String,
    hidden: bool,
    provider: Option<Provider>,
    anilist_id: Option<u64>,
    mal_id: Option<u64>,
    tmdb_id: Option<u64>,
    tmdb_kind: Option<String>,
    tvmaze_id: Option<u64>,
    track_choice: Option<String>,
    episodes_set: Option<u32>,
    episodes_set_season: Option<u32>,
    romaji: Option<String>,
    english: Option<String>,
    list_status: Option<ListStatus>,
    list_score: Option<f64>,
}

/// The whole document, built off the reader connection and the secrets
/// store. Synchronous, and it reads the keyring when `private` is set, so
/// the job runs it through `spawn_blocking` rather than on a tokio worker.
pub fn build(core: &Core, private: bool, now: SystemTime) -> Result<Document, CoreError> {
    let (sources, rows, preferences, fixes) = core.store.read(|c| {
        Ok((
            source_paths(c)?,
            series_rows(c)?,
            prefs::load_preferences(c)?,
            fix_rows(c)?,
        ))
    })?;

    let mut doc = Document {
        format: format::FORMAT.to_string(),
        version: format::VERSION,
        exported_at: format::format_instant(time::to_secs(now)),
        exported_by: ExportedBy {
            app: "anibeam".to_string(),
            line: "native".to_string(),
            version: crate::VERSION.to_string(),
            extra: HashMap::new(),
        },
        private,
        sources: sources
            .into_iter()
            .map(|path| SourceEntry {
                path,
                extra: HashMap::new(),
            })
            .collect(),
        series: rows
            .iter()
            .map(|r| entry(r, preferences.title_language))
            .collect(),
        accounts: None,
        keys: None,
        history: None,
        preferences: None,
        extra: HashMap::new(),
    };
    // Every series path goes out in both shapes, so its fixes do too.
    attach_fixes(&mut doc.series, &rows, fixes);
    if !private {
        return Ok(doc);
    }

    let (main, anilist, mal, history, settings) = core.store.read(|c| {
        Ok((
            prefs::load_main_tracker(c)?,
            accounts::load_row(c, Tracker::Anilist)?,
            accounts::load_row(c, Tracker::Mal)?,
            history(c, &rows)?,
            prefs::load_settings(c)?,
        ))
    })?;
    doc.accounts = Some(Accounts {
        main: main.as_str().to_string(),
        anilist: account(core, Tracker::Anilist, anilist)?,
        mal: account(core, Tracker::Mal, mal)?,
        extra: HashMap::new(),
    });
    // There is no TMDB key in the native line, and no other key either, so
    // the section is present and empty rather than absent: a reader that
    // looks for it finds the shape it expects.
    doc.keys = Some(Value::Object(serde_json::Map::new()));
    doc.history = Some(history);
    doc.preferences = Some(PrefsEntry {
        title_language: preferences.title_language.as_str().to_string(),
        library_tab: preferences.library_tab.as_str().to_string(),
        library_sort: SortEntry {
            key: preferences.library_sort.as_str().to_string(),
            direction: preferences.library_direction.as_str().to_string(),
            extra: HashMap::new(),
        },
        feed_sort: preferences.feed_sort.as_str().to_string(),
        auto_skip: Some(settings.auto_skip),
        extra: HashMap::new(),
    });
    Ok(doc)
}

/// The history section alone, for another AniBeam computer to merge
/// (`host::history`). No accounts, so no keyring read.
pub(crate) fn history_section(core: &Core) -> Result<History, CoreError> {
    core.store.read(|c| {
        let rows = series_rows(c)?;
        history(c, &rows)
    })
}

/// Starts the Export job. The document is built on a blocking thread for
/// the keyring's sake, then written through a `.tmp` sibling and a rename,
/// so a reader either sees the whole file or the one that was there.
pub fn start(core: &Arc<Core>, path: String, private: bool) -> u64 {
    let owner = core.clone();
    core.jobs
        .clone()
        .start(JobKind::Export, move |_ctx| async move {
            let builder = owner.clone();
            let doc = owner
                .handle
                .spawn_blocking(move || build(&builder, private, time::now()))
                .await
                .map_err(|e| CoreError::internal(format!("export task: {e}")))??;
            let count = doc.series.len() as u64;
            let text = serde_json::to_string_pretty(&doc)?;
            atomic_file::write_async(Path::new(&path), text).await?;
            Ok(Finished {
                plain: Some(format!(
                    "Saved {count} {} to {}.",
                    crate::words::pick(count, "show", "shows"),
                    crate::library::scan::folder_of(&path)
                )),
                level: Level::Info,
                message: format!("Exported {count} series to {path}"),
                body: EventBody::ExportFinished { path },
            })
        })
}

// What the tables hold ------------------------------------------------------

fn source_paths(conn: &Connection) -> Result<Vec<String>, CoreError> {
    let mut stmt = conn.prepare("SELECT path FROM sources ORDER BY id")?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Every series, missing ones included: the file carries the match, the
/// flag and the history of a series whose disk is not plugged in.
fn series_rows(conn: &Connection) -> Result<Vec<Row>, CoreError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.kind, s.path, s.folder_name, s.hidden, s.provider, s.anilist_id, s.mal_id, s.tmdb_id, s.tmdb_kind,
                s.track_choice, COALESCE(t.name, m.title_romaji), m.title_english, s.tvmaze_id,
                s.episodes_set, s.episodes_set_season, l.status, l.score
         FROM series s
         LEFT JOIN anilist_media m ON m.id = s.anilist_id
         LEFT JOIN local_entries l ON l.series_id = s.id
         LEFT JOIN tvmaze_shows t ON t.id = s.tvmaze_id AND s.provider = 'tvmaze'
         ORDER BY s.id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Row {
                id: r.get::<_, i64>(0)? as u64,
                kind: SeriesKind::from_column(&r.get::<_, String>(1)?).unwrap_or(SeriesKind::Show),
                path: r.get(2)?,
                folder_name: r.get(3)?,
                hidden: r.get::<_, i64>(4)? == 1,
                provider: r
                    .get::<_, Option<String>>(5)?
                    .as_deref()
                    .and_then(Provider::from_column),
                anilist_id: r.get::<_, Option<i64>>(6)?.map(|v| v as u64),
                mal_id: r.get::<_, Option<i64>>(7)?.map(|v| v as u64),
                tmdb_id: r.get::<_, Option<i64>>(8)?.map(|v| v as u64),
                tmdb_kind: r.get(9)?,
                track_choice: r.get(10)?,
                romaji: r.get(11)?,
                english: r.get(12)?,
                tvmaze_id: r.get::<_, Option<i64>>(13)?.map(|v| v as u64),
                episodes_set: r.get::<_, Option<i64>>(14)?.map(|v| v as u32),
                episodes_set_season: r.get::<_, Option<i64>>(15)?.map(|v| v as u32),
                list_status: r
                    .get::<_, Option<String>>(16)?
                    .and_then(|s| ListStatus::from_column(&s)),
                list_score: r.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// One quick fix as `file_fixes` holds it: the file's full path, what it is
/// and what it read before.
struct Fix {
    path: String,
    reading: FileReading,
    was: Option<FileReading>,
}

/// Every quick fix. A reading this build cannot parse is left out, as the
/// scan leaves it to the name.
fn fix_rows(conn: &Connection) -> Result<Vec<Fix>, CoreError> {
    let mut stmt = conn.prepare("SELECT path, reading, was FROM file_fixes ORDER BY path")?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(path, reading, was)| {
            Some(Fix {
                path,
                reading: serde_json::from_str(&reading).ok()?,
                was: serde_json::from_str(&was).ok(),
            })
        })
        .collect())
}

/// Hangs each fix on the series that owns its file, the longest series path
/// above it, with the path below that series' folder. A fix under no series
/// in the document has nowhere to go and is left out. `series` and `rows`
/// are parallel, one entry per row.
fn attach_fixes(series: &mut [SeriesEntry], rows: &[Row], fixes: Vec<Fix>) {
    for fix in fixes {
        let Some(i) = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| under(&fix.path, &r.path))
            .max_by_key(|(_, r)| r.path.len())
            .map(|(i, _)| i)
        else {
            continue;
        };
        series[i].fixes.push(FixEntry {
            path: relative(&fix.path, &rows[i].path),
            reading: fix.reading,
            was: fix.was,
            extra: HashMap::new(),
        });
    }
}

/// `path` below `root` with forward slashes, empty when they are one path:
/// a film's fix is on the series' own file.
fn relative(path: &str, root: &str) -> String {
    let root = root.trim_end_matches(std::path::is_separator);
    let rest = path.strip_prefix(root).unwrap_or(path);
    Path::new(rest)
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn entry(row: &Row, lang: TitleLanguage) -> SeriesEntry {
    SeriesEntry {
        kind: match row.kind {
            SeriesKind::Show => "series".to_string(),
            SeriesKind::Movie => "movie".to_string(),
        },
        path: row.path.clone(),
        // The native store has no slug, so the row id is the id a human
        // reads. Nothing matches on it at either end.
        id: row.id.to_string(),
        title: titles::resolve(
            lang,
            row.romaji.as_deref(),
            row.english.as_deref(),
            // The file carries no native title, so Native exports romaji
            None,
            &row.folder_name,
        ),
        hidden: row.hidden,
        match_: match_entry(row),
        track_choice: row
            .track_choice
            .as_deref()
            .and_then(|json| serde_json::from_str::<TrackChoice>(json).ok()),
        episodes_set: row.episodes_set,
        episodes_set_season: row.episodes_set_season,
        fixes: Vec::new(),
        list: (row.list_status.is_some() || row.list_score.is_some()).then(|| ListEntry {
            status: row.list_status.map(|s| s.as_str().to_string()),
            score: row.list_score,
            extra: HashMap::new(),
        }),
        extra: HashMap::new(),
    }
}

/// Only the provider the row names, never the columns another provider
/// left behind.
fn match_entry(row: &Row) -> Option<MatchEntry> {
    match row.provider? {
        Provider::Anilist => Some(MatchEntry::Tracker {
            provider: TrackerName::Anilist,
            anilist_id: row.anilist_id,
            mal_id: row.mal_id,
        }),
        Provider::Mal => Some(MatchEntry::Tracker {
            provider: TrackerName::Mal,
            anilist_id: row.anilist_id,
            mal_id: row.mal_id,
        }),
        Provider::Tmdb => Some(MatchEntry::Tmdb {
            provider: TmdbName::Tmdb,
            tmdb_id: row.tmdb_id?,
            tmdb_kind: row
                .tmdb_kind
                .clone()
                .unwrap_or_else(|| TmdbKind::Movie.as_str().to_string()),
        }),
        Provider::Tvmaze => Some(MatchEntry::Tvmaze {
            provider: TvmazeName::Tvmaze,
            tvmaze_id: row.tvmaze_id?,
        }),
    }
}

/// A connected tracker with its tokens, or nothing at all: a disconnected
/// one is null rather than a partial record.
fn account(
    core: &Core,
    t: Tracker,
    row: Option<accounts::Row>,
) -> Result<Option<Account>, CoreError> {
    let Some(row) = row.filter(|r| r.connected_at.is_some()) else {
        return Ok(None);
    };
    let hint = row.secret_store;
    let secrets = core.secrets();
    let read = |key: String| -> Result<Option<String>, CoreError> {
        Ok(secrets.get(&key, hint)?.map(|(value, _)| value))
    };
    Ok(Some(Account {
        user_id: row.user_id,
        username: row.username.clone(),
        client_id: row.client_id.clone().unwrap_or_default(),
        client_secret: read(secret_key(t))?,
        // An empty string when the store holds none, which is the answer
        // Electron's own export gave.
        access_token: read(access_key(t))?.unwrap_or_default(),
        refresh_token: read(refresh_key(t))?,
        expires_at: row
            .expires_at
            .map(|at| Value::String(format::format_instant(time::to_secs(at)))),
        extra: HashMap::new(),
    }))
}

/// The three history tables, keyed the way the format keys them: by the
/// series' path and the episode's number, and by file where there is no
/// number to key by.
fn history(conn: &Connection, rows: &[Row]) -> Result<History, CoreError> {
    let by_id: HashMap<u64, &Row> = rows.iter().map(|r| (r.id, r)).collect();

    let mut views = Vec::new();
    {
        let mut stmt =
            conn.prepare("SELECT series_id, episode_key, at FROM views ORDER BY series_id")?;
        let read = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in read {
            let (id, key, at) = row?;
            let Some(series) = by_id.get(&id) else {
                continue;
            };
            let (last_episode, season, file) = entry_of(series, &key);
            views.push(View {
                series: series.path.clone(),
                last_episode,
                season,
                file,
                at: Value::String(format::format_instant(at)),
                extra: HashMap::new(),
            });
        }
    }

    let mut completed = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT series_id, episode_key, at FROM completed ORDER BY series_id, episode_key",
        )?;
        let read = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in read {
            let (id, key, at) = row?;
            let Some(series) = by_id.get(&id) else {
                continue;
            };
            let (episode, season, file) = entry_of(series, &key);
            completed.push(Completed {
                series: series.path.clone(),
                episode,
                season,
                file,
                at: Value::String(format::format_instant(at)),
                extra: HashMap::new(),
            });
        }
    }

    let mut resume_points = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT series_id, episode_key, position, duration, at FROM resume_points ORDER BY series_id, episode_key")?;
        let read = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, String>(1)?,
                r.get::<_, f64>(2)?,
                r.get::<_, f64>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })?;
        for row in read {
            let (id, key, position, duration, at) = row?;
            let Some(series) = by_id.get(&id) else {
                continue;
            };
            let at = Value::String(format::format_instant(at));
            resume_points.push(match classifier::parse_episode_key(&key) {
                Some((season, episode)) => ResumeEntry::Series {
                    series: series.path.clone(),
                    episode,
                    season: later_season(season),
                    position,
                    duration,
                    at,
                },
                // No number to key by: a film, an OP, an ED, an SP. The
                // key is the file's own name, and the file sits inside the
                // series' folder unless the series is the file.
                None => ResumeEntry::File {
                    file: file_of(series, &key),
                    position,
                    duration,
                    at,
                },
            });
        }
    }

    Ok(History {
        views,
        completed,
        resume_points,
        extra: HashMap::new(),
    })
}

/// The number, season and file a view or a completed entry writes for a
/// history key. A key with no number writes 0: a film's key is its own
/// file, which the series path already names, and a show's extra (an OP,
/// an ED, an SP) names its file the way a resume point does. Season 1 is
/// the format's default and is left out.
fn entry_of(series: &Row, key: &str) -> (f64, Option<u32>, Option<String>) {
    match classifier::parse_episode_key(key) {
        Some((season, number)) => (number, later_season(season), None),
        None => (
            0.0,
            None,
            (series.kind == SeriesKind::Show).then(|| file_of(series, key)),
        ),
    }
}

/// Season 1 is what a document without a season means, so only a later
/// season is written.
fn later_season(season: u32) -> Option<u32> {
    (season != 1).then_some(season)
}

fn file_of(series: &Row, key: &str) -> String {
    match series.kind {
        SeriesKind::Movie => series.path.clone(),
        SeriesKind::Show => format!("{}/{key}", series.path.trim_end_matches('/')),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(kind: SeriesKind, path: &str) -> Row {
        Row {
            id: 1,
            kind,
            path: path.to_string(),
            folder_name: crate::transfer::file_name(path),
            hidden: false,
            provider: None,
            anilist_id: None,
            mal_id: None,
            tmdb_id: None,
            tmdb_kind: None,
            tvmaze_id: None,
            track_choice: None,
            episodes_set: None,
            episodes_set_season: None,
            romaji: None,
            english: None,
            list_status: None,
            list_score: None,
        }
    }

    #[test]
    fn a_history_key_with_no_number_names_the_file_it_belongs_to() {
        assert_eq!(
            file_of(&row(SeriesKind::Show, "/lib/Show"), "NCOP1.mkv"),
            "/lib/Show/NCOP1.mkv"
        );
        assert_eq!(
            file_of(&row(SeriesKind::Movie, "/lib/Movies/Film.mkv"), "Film.mkv"),
            "/lib/Movies/Film.mkv"
        );
        let show = row(SeriesKind::Show, "/lib/Show");
        let film = row(SeriesKind::Movie, "/lib/Movies/Film.mkv");
        assert_eq!(entry_of(&show, "12"), (12.0, None, None));
        assert_eq!(entry_of(&show, "12.5"), (12.5, None, None));
        assert_eq!(entry_of(&show, "S02E01"), (1.0, Some(2), None));
        assert_eq!(
            entry_of(&show, "NCOP1.mkv"),
            (0.0, None, Some("/lib/Show/NCOP1.mkv".to_string()))
        );
        // A film's path is already its file.
        assert_eq!(entry_of(&film, "Film.mkv"), (0.0, None, None));
    }

    /// Only the provider the row names: applying an AniList match over a
    /// TMDB one leaves the old columns behind, and the document must not
    /// carry them.
    #[test]
    fn a_match_carries_one_provider_and_the_ids_it_owns() {
        let mut r = row(SeriesKind::Show, "/lib/Show");
        assert!(match_entry(&r).is_none());
        r.provider = Some(Provider::Anilist);
        r.anilist_id = Some(7);
        r.tmdb_id = Some(9);
        assert!(matches!(
            match_entry(&r),
            Some(MatchEntry::Tracker {
                anilist_id: Some(7),
                mal_id: None,
                ..
            })
        ));
        r.provider = Some(Provider::Tmdb);
        r.tmdb_kind = Some("tv".to_string());
        assert!(
            matches!(match_entry(&r), Some(MatchEntry::Tmdb { tmdb_id: 9, tmdb_kind, .. }) if tmdb_kind == "tv")
        );
    }

    /// A fix goes to the closest series above its file, with the path below
    /// that series; a film's own file is the empty path, and a fix under no
    /// series is left out.
    #[test]
    fn a_fix_hangs_on_the_series_that_owns_its_file() {
        let rows = vec![
            row(SeriesKind::Show, "/lib/Show"),
            row(SeriesKind::Show, "/lib/Show/Season 2"),
            row(SeriesKind::Movie, "/lib/Movies/Film.mkv"),
        ];
        let mut series: Vec<SeriesEntry> = rows
            .iter()
            .map(|r| entry(r, TitleLanguage::Romaji))
            .collect();
        let ep = |n: f64| FileReading::Episode {
            number: n,
            season: None,
        };
        let fix = |path: &str| Fix {
            path: path.to_string(),
            reading: ep(13.0),
            was: Some(ep(1.0)),
        };
        attach_fixes(
            &mut series,
            &rows,
            vec![
                fix("/lib/Show/Extras/Show - 01.mkv"),
                fix("/lib/Show/Season 2/Show - 02.mkv"),
                fix("/lib/Movies/Film.mkv"),
                fix("/lib/Other/Show - 01.mkv"),
            ],
        );
        let paths =
            |i: usize| -> Vec<&str> { series[i].fixes.iter().map(|f| f.path.as_str()).collect() };
        assert_eq!(paths(0), vec!["Extras/Show - 01.mkv"]);
        assert_eq!(paths(1), vec!["Show - 02.mkv"]);
        assert_eq!(paths(2), vec![""]);
        assert_eq!(series[0].fixes[0].was, Some(ep(1.0)));
    }
}
