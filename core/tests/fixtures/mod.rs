//! Library state built with plain SQL, so a test can set up any shape of
//! library without a scan and without the network. Everything goes through
//! `core.store()`, the same writer connection the core itself uses, so a
//! read that follows a fixture always sees it.
#![allow(dead_code)]

use anibeam_core::library::{classifier, labels};
use anibeam_core::{time, Core, ExtraKind, SeriesKind};
use rusqlite::params;

pub fn insert_source(core: &Core, path: &str) -> u64 {
    let path = path.to_string();
    core.store()
        .write(move |c| {
            c.execute("INSERT INTO sources (path, available, added_at) VALUES (?1, 1, ?2)", params![path, time::now_secs()])?;
            Ok(c.last_insert_rowid() as u64)
        })
        .unwrap()
}

pub fn insert_series(core: &Core, source: u64, kind: SeriesKind, path: &str, folder_name: &str) -> u64 {
    let (path, folder_name) = (path.to_string(), folder_name.to_string());
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT INTO series (source_id, kind, path, folder_name, added_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![source as i64, kind.as_str(), path, folder_name, time::now_secs()],
            )?;
            Ok(c.last_insert_rowid() as u64)
        })
        .unwrap()
}

/// The file row a scan would write: the classifier fills in the extra's
/// kind, index and label, and the history key, so a fixture only has to say
/// where the file is and which episode it is.
pub fn insert_file(core: &Core, series: u64, path: &str, number: f64, season: Option<u32>, kind: &str, mtime: i64) -> u64 {
    let name = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    let classified = classifier::classify(&name);
    let is_extra = kind == "extra";
    let extra_kind = if is_extra { Some(classified.extra.unwrap_or(ExtraKind::Other)) } else { None };
    let label = match extra_kind {
        Some(k) => labels::extra_label(k, classified.extra_index, classified.extra_variant.as_deref(), classified.raw_label.as_deref()),
        None => labels::episode_code(season, number),
    };
    let episode_key = if is_extra { name } else { classifier::format_number(number) };
    let (path, kind) = (path.to_string(), kind.to_string());
    let extra_index = if is_extra { classified.extra_index } else { None };
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT INTO files (series_id, path, size, mtime, kind, number, season, extra_kind, extra_index, label, episode_key, sidecars, seen_at)
                 VALUES (?1, ?2, 0, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, '[]', ?11)",
                params![
                    series as i64,
                    path,
                    mtime,
                    kind,
                    number,
                    season.map(i64::from),
                    extra_kind.map(ExtraKind::as_str),
                    extra_index.map(i64::from),
                    label,
                    episode_key,
                    time::now_secs(),
                ],
            )?;
            Ok(c.last_insert_rowid() as u64)
        })
        .unwrap()
}

/// `average_score` is AniList's own 0 to 100 integer, normalised at read.
#[allow(clippy::too_many_arguments)]
pub fn insert_media(
    core: &Core,
    anilist_id: u64,
    romaji: Option<&str>,
    english: Option<&str>,
    episodes: Option<u32>,
    status: &str,
    format: &str,
    average_score: Option<u32>,
) {
    let (romaji, english) = (romaji.map(str::to_string), english.map(str::to_string));
    let (status, format) = (status.to_string(), format.to_string());
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO anilist_media (id, media_type, title_romaji, title_english, format, status, episodes, average_score, fetched_at)
                 VALUES (?1, 'ANIME', ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    anilist_id as i64,
                    romaji,
                    english,
                    format,
                    status,
                    episodes.map(i64::from),
                    average_score.map(i64::from),
                    time::now_secs(),
                ],
            )?;
            Ok(())
        })
        .unwrap()
}

pub fn match_series(core: &Core, series: u64, anilist_id: Option<u64>, mal_id: Option<u64>) {
    let provider = if anilist_id.is_some() { "anilist" } else { "mal" };
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE series SET provider = ?1, anilist_id = ?2, mal_id = ?3, confirmed = 1, matched_at = ?4 WHERE id = ?5",
                params![provider, anilist_id.map(|v| v as i64), mal_id.map(|v| v as i64), time::now_secs(), series as i64],
            )?;
            Ok(())
        })
        .unwrap()
}

pub fn insert_airing(core: &Core, anilist_id: u64, number: i64, aired_at: i64) {
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO anilist_episodes (anilist_id, number, title, aired_at) VALUES (?1, ?2, NULL, ?3)",
                params![anilist_id as i64, number, aired_at],
            )?;
            Ok(())
        })
        .unwrap()
}

pub fn insert_tracker_entry(core: &Core, tracker: &str, media_id: u64, progress: u32, status: &str, score: Option<f64>) {
    let (tracker, status) = (tracker.to_string(), status.to_string());
    core.store()
        .write(move |c| {
            let now = time::now_secs();
            c.execute(
                "INSERT OR REPLACE INTO tracker_entries (tracker, media_id, status, progress, score, repeat, updated_at, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
                params![tracker, media_id as i64, status, i64::from(progress), score, now],
            )?;
            Ok(())
        })
        .unwrap()
}

pub fn insert_completed(core: &Core, series: u64, key: &str, at: i64) {
    let key = key.to_string();
    core.store()
        .write(move |c| {
            c.execute("INSERT OR REPLACE INTO completed (series_id, episode_key, at) VALUES (?1, ?2, ?3)", params![series as i64, key, at])?;
            Ok(())
        })
        .unwrap()
}

pub fn insert_view(core: &Core, series: u64, key: &str, at: i64) {
    let key = key.to_string();
    core.store()
        .write(move |c| {
            c.execute("INSERT OR REPLACE INTO views (series_id, episode_key, at) VALUES (?1, ?2, ?3)", params![series as i64, key, at])?;
            Ok(())
        })
        .unwrap()
}

/// The path went away: the series lingers with its match and its history,
/// its files do not.
pub fn mark_missing(core: &Core, series: u64) {
    core.store()
        .write(move |c| {
            c.execute("UPDATE series SET missing_since = ?1 WHERE id = ?2", params![time::now_secs(), series as i64])?;
            c.execute("DELETE FROM files WHERE series_id = ?1", params![series as i64])?;
            Ok(())
        })
        .unwrap()
}
