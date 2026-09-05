//! The Export job: the library, and with the checkbox ticked everything
//! private with it, as one `anibeam-export` document at the current
//! version.
//!
//! Plain text throughout, tokens included, by decision: the checkbox is
//! the only guard and there is no encryption. What is left out is what can
//! be rebuilt, the franchise store, the image cache and the progress
//! caches among them.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use rusqlite::Connection;
use serde_json::Value;

use crate::contract::*;
use crate::core::Core;
use crate::jobs::Finished;
use crate::library::titles;
use crate::prefs;
use crate::time;
use crate::trackers::accounts::{self, access_key, refresh_key, secret_key};
use crate::transfer::format::{
    self, Account, Accounts, Completed, Document, ExportedBy, History, MatchEntry, PrefsEntry, ResumeEntry, SeriesEntry, SortEntry, SourceEntry,
    TmdbName, TrackerName, View,
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
    track_choice: Option<String>,
    romaji: Option<String>,
    english: Option<String>,
}

/// The whole document, built off the reader connection and the secrets
/// store. Synchronous, and it reads the keyring when `private` is set, so
/// the job runs it through `spawn_blocking` rather than on a tokio worker.
pub fn build(core: &Core, private: bool, now: SystemTime) -> Result<Document, CoreError> {
    let (sources, rows, preferences) = core
        .store
        .read(|c| Ok((source_paths(c)?, series_rows(c)?, prefs::load_preferences(c)?)))?;

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
        sources: sources.into_iter().map(|path| SourceEntry { path, extra: HashMap::new() }).collect(),
        series: rows.iter().map(|r| entry(r, preferences.title_language)).collect(),
        accounts: None,
        keys: None,
        history: None,
        preferences: None,
        extra: HashMap::new(),
    };
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

/// Starts the Export job. The document is built on a blocking thread for
/// the keyring's sake, then written through a `.tmp` sibling and a rename,
/// so a reader either sees the whole file or the one that was there.
pub fn start(core: &Arc<Core>, path: String, private: bool) -> u64 {
    let owner = core.clone();
    core.jobs.clone().start(JobKind::Export, move |_ctx| async move {
        let builder = owner.clone();
        let doc = owner
            .handle
            .spawn_blocking(move || build(&builder, private, time::now()))
            .await
            .map_err(|e| CoreError::internal(format!("export task: {e}")))??;
        let count = doc.series.len() as u64;
        let text = serde_json::to_string_pretty(&doc)?;
        write_atomically(&path, text.into_bytes()).await?;
        Ok(Finished {
            level: Level::Info,
            message: format!("exported {count} series to {path}"),
            body: EventBody::ExportFinished { path },
        })
    })
}

async fn write_atomically(path: &str, bytes: Vec<u8>) -> Result<(), CoreError> {
    let target = PathBuf::from(path);
    if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
        tokio::fs::create_dir_all(parent).await.map_err(|e| CoreError::io_at(parent.to_string_lossy(), e))?;
    }
    let tmp = tmp_sibling(&target);
    tokio::fs::write(&tmp, bytes).await.map_err(|e| CoreError::io_at(tmp.to_string_lossy(), e))?;
    tokio::fs::rename(&tmp, &target).await.map_err(|e| CoreError::io_at(target.to_string_lossy(), e))?;
    Ok(())
}

fn tmp_sibling(target: &Path) -> PathBuf {
    let mut name = target.file_name().map(std::ffi::OsString::from).unwrap_or_else(|| std::ffi::OsString::from("anibeam-export.json"));
    name.push(".tmp");
    target.with_file_name(name)
}

// What the tables hold ------------------------------------------------------

fn source_paths(conn: &Connection) -> Result<Vec<String>, CoreError> {
    let mut stmt = conn.prepare("SELECT path FROM sources ORDER BY id")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Every series, missing ones included: the file carries the match, the
/// flag and the history of a series whose disk is not plugged in.
fn series_rows(conn: &Connection) -> Result<Vec<Row>, CoreError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.kind, s.path, s.folder_name, s.hidden, s.provider, s.anilist_id, s.mal_id, s.tmdb_id, s.tmdb_kind,
                s.track_choice, m.title_romaji, m.title_english
         FROM series s LEFT JOIN anilist_media m ON m.id = s.anilist_id
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
                provider: r.get::<_, Option<String>>(5)?.as_deref().and_then(Provider::from_column),
                anilist_id: r.get::<_, Option<i64>>(6)?.map(|v| v as u64),
                mal_id: r.get::<_, Option<i64>>(7)?.map(|v| v as u64),
                tmdb_id: r.get::<_, Option<i64>>(8)?.map(|v| v as u64),
                tmdb_kind: r.get(9)?,
                track_choice: r.get(10)?,
                romaji: r.get(11)?,
                english: r.get(12)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
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
        title: titles::resolve(lang, row.romaji.as_deref(), row.english.as_deref(), &row.folder_name),
        hidden: row.hidden,
        match_: match_entry(row),
        track_choice: row.track_choice.as_deref().and_then(|json| serde_json::from_str::<TrackChoice>(json).ok()),
        extra: HashMap::new(),
    }
}

/// Only the provider the row names, never the columns another provider
/// left behind.
fn match_entry(row: &Row) -> Option<MatchEntry> {
    match row.provider? {
        Provider::Anilist => Some(MatchEntry::Tracker { provider: TrackerName::Anilist, anilist_id: row.anilist_id, mal_id: row.mal_id }),
        Provider::Mal => Some(MatchEntry::Tracker { provider: TrackerName::Mal, anilist_id: row.anilist_id, mal_id: row.mal_id }),
        Provider::Tmdb => Some(MatchEntry::Tmdb {
            provider: TmdbName::Tmdb,
            tmdb_id: row.tmdb_id?,
            tmdb_kind: row.tmdb_kind.clone().unwrap_or_else(|| TmdbKind::Movie.as_str().to_string()),
        }),
    }
}

/// A connected tracker with its tokens, or nothing at all: a disconnected
/// one is null rather than a partial record.
fn account(core: &Core, t: Tracker, row: Option<accounts::Row>) -> Result<Option<Account>, CoreError> {
    let Some(row) = row.filter(|r| r.connected_at.is_some()) else { return Ok(None) };
    let hint = row.secret_store;
    let secrets = core.secrets();
    let read = |key: String| -> Result<Option<String>, CoreError> { Ok(secrets.get(&key, hint)?.map(|(value, _)| value)) };
    Ok(Some(Account {
        user_id: row.user_id,
        username: row.username.clone(),
        client_id: row.client_id.clone().unwrap_or_default(),
        client_secret: read(secret_key(t))?,
        // An empty string when the store holds none, which is the answer
        // Electron's own export gave.
        access_token: read(access_key(t))?.unwrap_or_default(),
        refresh_token: read(refresh_key(t))?,
        expires_at: row.expires_at.map(|at| Value::String(format::format_instant(time::to_secs(at)))),
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
        let mut stmt = conn.prepare("SELECT series_id, episode_key, at FROM views ORDER BY series_id")?;
        let read = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))?;
        for row in read {
            let (id, key, at) = row?;
            let Some(series) = by_id.get(&id) else { continue };
            views.push(View {
                series: series.path.clone(),
                last_episode: number_of(&key),
                at: Value::String(format::format_instant(at)),
                extra: HashMap::new(),
            });
        }
    }

    let mut completed = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT series_id, episode_key, at FROM completed ORDER BY series_id, episode_key")?;
        let read = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))?;
        for row in read {
            let (id, key, at) = row?;
            let Some(series) = by_id.get(&id) else { continue };
            completed.push(Completed {
                series: series.path.clone(),
                episode: number_of(&key),
                at: Value::String(format::format_instant(at)),
                extra: HashMap::new(),
            });
        }
    }

    let mut resume_points = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT series_id, episode_key, position, duration, at FROM resume_points ORDER BY series_id, episode_key")?;
        let read = stmt.query_map([], |r| {
            Ok((r.get::<_, i64>(0)? as u64, r.get::<_, String>(1)?, r.get::<_, f64>(2)?, r.get::<_, f64>(3)?, r.get::<_, i64>(4)?))
        })?;
        for row in read {
            let (id, key, position, duration, at) = row?;
            let Some(series) = by_id.get(&id) else { continue };
            let at = Value::String(format::format_instant(at));
            resume_points.push(match key.parse::<f64>() {
                Ok(episode) => ResumeEntry::Series { series: series.path.clone(), episode, position, duration, at },
                // No number to key by: a film, an OP, an ED, an SP. The
                // key is the file's own name, and the file sits inside the
                // series' folder unless the series is the file.
                Err(_) => ResumeEntry::File { file: file_of(series, &key), position, duration, at },
            });
        }
    }

    Ok(History { views, completed, resume_points, extra: HashMap::new() })
}

/// A film's history key is its file name rather than a number, and the
/// format writes 0 for it.
fn number_of(key: &str) -> f64 {
    key.parse::<f64>().unwrap_or(0.0)
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
            track_choice: None,
            romaji: None,
            english: None,
        }
    }

    #[test]
    fn a_history_key_with_no_number_names_the_file_it_belongs_to() {
        assert_eq!(file_of(&row(SeriesKind::Show, "/lib/Show"), "NCOP1.mkv"), "/lib/Show/NCOP1.mkv");
        assert_eq!(file_of(&row(SeriesKind::Movie, "/lib/Movies/Film.mkv"), "Film.mkv"), "/lib/Movies/Film.mkv");
        assert_eq!(number_of("12"), 12.0);
        assert_eq!(number_of("12.5"), 12.5);
        assert_eq!(number_of("Film.mkv"), 0.0);
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
        assert!(matches!(match_entry(&r), Some(MatchEntry::Tracker { anilist_id: Some(7), mal_id: None, .. })));
        r.provider = Some(Provider::Tmdb);
        r.tmdb_kind = Some("tv".to_string());
        assert!(matches!(match_entry(&r), Some(MatchEntry::Tmdb { tmdb_id: 9, tmdb_kind, .. }) if tmdb_kind == "tv"));
    }

    #[test]
    fn the_tmp_file_is_a_sibling_of_the_target() {
        assert_eq!(tmp_sibling(Path::new("/home/bandit/export.json")), PathBuf::from("/home/bandit/export.json.tmp"));
    }
}
