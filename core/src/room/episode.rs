//! A room's episode: built from the host's playback session, carried to the
//! contract, and found again in a joiner's library.

use rusqlite::{Connection, OptionalExtension, params};

use anibeam_room::proto;

use crate::contract::*;
use crate::playback::session::{Session, opening_of};
use crate::playback::skip;

/// The episode a session plays, as a room carries it.
pub fn of_session(s: &Session) -> proto::EpisodeRef {
    proto::EpisodeRef {
        anilist_id: s.anilist_id.and_then(|v| i64::try_from(v).ok()),
        mal_id: s.mal_id.and_then(|v| i64::try_from(v).ok()),
        tvmaze_id: s.tvmaze_id.and_then(|v| i64::try_from(v).ok()),
        episode_key: s.episode_key.clone(),
        number: s.number,
        series_title: s.series_title.clone(),
        episode_title: s.episode_title.clone(),
        duration: s.duration,
        opening_start: s.opening_start,
    }
}

/// The wire's episode as the contract shows it. An id below nought is no id.
pub fn to_contract(e: &proto::EpisodeRef) -> EpisodeRef {
    let id = |v: Option<i64>| v.and_then(|v| u64::try_from(v).ok());
    EpisodeRef {
        anilist_id: id(e.anilist_id),
        mal_id: id(e.mal_id),
        tvmaze_id: id(e.tvmaze_id),
        episode_key: e.episode_key.clone(),
        number: e.number,
        series_title: e.series_title.clone(),
        episode_title: e.episode_title.clone(),
        duration: e.duration,
        opening_start: e.opening_start,
    }
}

/// Whether two refs name the same episode of the same show. Titles, the length
/// and the opening are the host's file's and may differ between copies.
pub fn same(a: &proto::EpisodeRef, b: &proto::EpisodeRef) -> bool {
    a.episode_key == b.episode_key
        && a.anilist_id == b.anilist_id
        && a.mal_id == b.mal_id
        && a.tvmaze_id == b.tvmaze_id
}

/// What this library has of a room's episode.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Local {
    pub series: Option<u64>,
    pub file: Option<u64>,
    /// Where the opening starts in that file, when a skip lookup has found it.
    pub opening: Option<f64>,
}

/// Finds the room's episode here: the series by its AniList id, then its MAL id,
/// then its TVmaze id, and in those the file by the episode's history key. A
/// series that matches without the file is still named, so the shell can say the
/// episode is missing from a show the person has.
pub fn find_local(conn: &Connection, e: &proto::EpisodeRef) -> Result<Local, CoreError> {
    let mut candidates: Vec<u64> = Vec::new();
    let by = [
        (
            "SELECT id FROM series WHERE anilist_id = ?1 ORDER BY missing_since IS NOT NULL, id",
            e.anilist_id,
        ),
        (
            "SELECT id FROM series WHERE mal_id = ?1 ORDER BY missing_since IS NOT NULL, id",
            e.mal_id,
        ),
        (
            "SELECT id FROM series WHERE tvmaze_id = ?1 AND provider = 'tvmaze' ORDER BY missing_since IS NOT NULL, id",
            e.tvmaze_id,
        ),
    ];
    for (sql, id) in by {
        let Some(id) = id else {
            continue;
        };
        let mut stmt = conn.prepare_cached(sql)?;
        let ids = stmt
            .query_map(params![id], |r| r.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for series in ids {
            let series = series as u64;
            if !candidates.contains(&series) {
                candidates.push(series);
            }
        }
    }
    for &series in &candidates {
        let file: Option<i64> = conn
            .query_row(
                "SELECT id FROM files WHERE series_id = ?1 AND episode_key = ?2 ORDER BY id LIMIT 1",
                params![series as i64, e.episode_key],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(file) = file {
            let opening = skip::cached(conn, series, &e.episode_key, crate::time::now_secs())?
                .and_then(|windows| opening_of(&windows));
            return Ok(Local {
                series: Some(series),
                file: Some(file as u64),
                opening,
            });
        }
    }
    Ok(Local {
        series: candidates.first().copied(),
        file: None,
        opening: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        c.pragma_update(None, "foreign_keys", "ON").unwrap();
        crate::store::migrations::apply(&mut c).unwrap();
        c.execute(
            "INSERT INTO sources (id, path, available, added_at) VALUES (1, '/lib', 1, 0)",
            [],
        )
        .unwrap();
        c
    }

    fn series(c: &Connection, id: i64, mal: Option<i64>, tvmaze: Option<i64>) {
        let provider = if tvmaze.is_some() { "tvmaze" } else { "mal" };
        c.execute(
            "INSERT INTO series (id, source_id, kind, path, folder_name, added_at, provider, mal_id, tvmaze_id)
             VALUES (?1, 1, 'show', ?2, 'Show', 0, ?3, ?4, ?5)",
            params![id, format!("/lib/{id}"), provider, mal, tvmaze],
        )
        .unwrap();
    }

    fn file(c: &Connection, id: i64, series: i64, key: &str) {
        c.execute(
            "INSERT INTO files (id, series_id, path, size, mtime, kind, number, label, episode_key, sidecars, seen_at)
             VALUES (?1, ?2, ?3, 0, 0, 'episode', 1, 'EP 1', ?4, '[]', 0)",
            params![id, series, format!("/lib/{series}/{id}.mkv"), key],
        )
        .unwrap();
    }

    fn episode(mal: Option<i64>, tvmaze: Option<i64>, key: &str) -> proto::EpisodeRef {
        proto::EpisodeRef {
            anilist_id: None,
            mal_id: mal,
            tvmaze_id: tvmaze,
            episode_key: key.into(),
            number: Some(1.0),
            series_title: "Show".into(),
            episode_title: None,
            duration: None,
            opening_start: None,
        }
    }

    #[test]
    fn the_file_is_found_by_series_id_and_key() {
        let c = conn();
        series(&c, 1, Some(21), None);
        file(&c, 10, 1, "1");
        file(&c, 11, 1, "2");
        let found = find_local(&c, &episode(Some(21), None, "2")).unwrap();
        assert_eq!(
            found,
            Local {
                series: Some(1),
                file: Some(11),
                opening: None
            }
        );
    }

    #[test]
    fn a_series_without_the_file_is_still_named() {
        let c = conn();
        series(&c, 1, Some(21), None);
        file(&c, 10, 1, "1");
        let found = find_local(&c, &episode(Some(21), None, "7")).unwrap();
        assert_eq!(found.series, Some(1));
        assert_eq!(found.file, None);
        let nothing = find_local(&c, &episode(Some(99), None, "1")).unwrap();
        assert_eq!(nothing, Local::default());
    }

    #[test]
    fn a_later_id_finds_what_an_earlier_one_did_not() {
        let c = conn();
        // Matched by MAL with no file for the key, and a TVmaze match that has it.
        series(&c, 1, Some(21), None);
        series(&c, 2, None, Some(5));
        file(&c, 20, 2, "1");
        let found = find_local(&c, &episode(Some(21), Some(5), "1")).unwrap();
        assert_eq!(found.series, Some(2));
        assert_eq!(found.file, Some(20));
    }

    #[test]
    fn the_opening_comes_from_the_skip_cache() {
        let c = conn();
        series(&c, 1, Some(21), None);
        file(&c, 10, 1, "1");
        let windows = vec![SkipWindow {
            kind: SkipKind::Intro,
            start: 92.5,
            end: 182.5,
            source: SkipSource::Chapters,
        }];
        c.execute(
            "INSERT INTO skip_windows (series_id, episode_key, windows, source, fetched_at) VALUES (1, '1', ?1, 'chapters', ?2)",
            params![serde_json::to_string(&windows).unwrap(), crate::time::now_secs()],
        )
        .unwrap();
        let found = find_local(&c, &episode(Some(21), None, "1")).unwrap();
        assert_eq!(found.opening, Some(92.5));
    }

    #[test]
    fn refs_name_the_same_episode_by_ids_and_key() {
        let a = episode(Some(21), None, "1");
        let mut b = a.clone();
        b.series_title = "Another title".into();
        b.opening_start = Some(3.0);
        assert!(same(&a, &b));
        b.episode_key = "2".into();
        assert!(!same(&a, &b));
        let mut c = a.clone();
        c.mal_id = Some(22);
        assert!(!same(&a, &c));
        assert_eq!(to_contract(&a).mal_id, Some(21));
        let mut neg = a;
        neg.anilist_id = Some(-1);
        assert_eq!(to_contract(&neg).anilist_id, None);
    }
}
