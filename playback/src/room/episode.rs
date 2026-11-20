//! A room's episode: built from the host's playback session, carried to the
//! contract, and found again in a joiner's library.

use rusqlite::{Connection, OptionalExtension, params};

use anibeam_room::proto;

use crate::session::{Session, opening_of};
use crate::skip;
use anibeam_contract::*;

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
        film: s.is_film,
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
        film: e.film,
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

/// Whether a member's session plays the room's episode: a show id in common, and
/// the same key or, when the keys differ, the same number, as a copy filed under
/// its season ("S03E06" for "6") is. Keys that both name a season must name the
/// same one: a TVmaze show shares its id across seasons, so S01E06 is not S02E06.
pub fn plays(mine: &proto::EpisodeRef, room: &proto::EpisodeRef) -> bool {
    let shared = |a: Option<i64>, b: Option<i64>| a.is_some() && a == b;
    let show = shared(mine.anilist_id, room.anilist_id)
        || shared(mine.mal_id, room.mal_id)
        || shared(mine.tvmaze_id, room.tvmaze_id);
    let season = match (season_of(&mine.episode_key), season_of(&room.episode_key)) {
        (Some(a), Some(b)) => a == b,
        _ => true,
    };
    let episode = mine.episode_key == room.episode_key
        || (season
            && matches!((mine.number, room.number), (Some(a), Some(b)) if (a - b).abs() < 0.001));
    show && episode
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
/// then its TVmaze id, and in those the file by the episode's history key, else
/// by its number (`by_number`). A series that matches without the file is still
/// named, so the shell can say the episode is missing from a show the person has.
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
    // The key first; then the episode by its number, since two copies of one
    // episode can carry different keys ("6" here, "S03E06" in a folder named for
    // its season)
    let mut found = None;
    for &series in &candidates {
        let file: Option<(i64, String)> = conn
            .query_row(
                "SELECT id, episode_key FROM files WHERE series_id = ?1 AND episode_key = ?2 ORDER BY id LIMIT 1",
                params![series as i64, e.episode_key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if let Some(file) = file {
            found = Some((series, file));
            break;
        }
    }
    if found.is_none()
        && let Some(number) = e.number.filter(|n| n.is_finite())
    {
        for &series in &candidates {
            if let Some(file) = by_number(conn, series, number, season_of(&e.episode_key))? {
                found = Some((series, file));
                break;
            }
        }
    }
    if let Some((series, (file, key))) = found {
        return found_at(conn, series, file, &key);
    }
    Ok(Local {
        series: candidates.first().copied(),
        file: None,
        opening: None,
    })
}

/// The episode's file in one series, with where its opening starts.
fn found_at(conn: &Connection, series: u64, file: i64, key: &str) -> Result<Local, CoreError> {
    let opening = skip::cached(conn, series, key, anibeam_foundation::time::now_secs())?
        .and_then(|windows| opening_of(&windows));
    Ok(Local {
        series: Some(series),
        file: Some(file as u64),
        opening,
    })
}

/// What the library holds at a download's path: the file itself, or the files
/// of the folder it made. The file is the download's own when the path is a
/// file, else the one in the folder with the episode's history key, else by
/// its number as `by_number` picks one. With no such file yet, the series of
/// the folder's first file, so a show that has the folder is still named.
/// None while the library holds nothing there.
pub fn at_path(
    conn: &Connection,
    content: &str,
    e: &proto::EpisodeRef,
) -> Result<Option<Local>, CoreError> {
    let path = content.replace('\\', "/");
    let folder = format!("{}/", path.trim_end_matches('/'));
    let mut stmt = conn.prepare_cached(
        "SELECT id, series_id, path, episode_key, kind, number, season FROM files
         WHERE path = ?1 OR substr(path, 1, length(?2)) = ?2 ORDER BY id",
    )?;
    let files = stmt
        .query_map(params![path, folder], |r| {
            Ok(Here {
                id: r.get(0)?,
                series: r.get::<_, i64>(1)? as u64,
                path: r.get(2)?,
                key: r.get(3)?,
                episode: r.get::<_, String>(4)? == "episode",
                number: r.get(5)?,
                season: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let Some(first) = files.first() else {
        return Ok(None);
    };
    let by_number = || {
        let number = e.number.filter(|n| n.is_finite())?;
        let fits: Vec<&Here> = files
            .iter()
            .filter(|f| f.episode && f.number.is_some_and(|n| (n - number).abs() < 0.001))
            .collect();
        one_of(&fits, season_of(&e.episode_key), |f| f.season).copied()
    };
    let file = files
        .iter()
        .find(|f| f.path == path)
        .or_else(|| files.iter().find(|f| f.key == e.episode_key))
        .or_else(by_number);
    Ok(Some(match file {
        Some(f) => found_at(conn, f.series, f.id, &f.key)?,
        None => Local {
            series: Some(first.series),
            file: None,
            opening: None,
        },
    }))
}

/// A file `at_path` reads.
struct Here {
    id: i64,
    series: u64,
    path: String,
    key: String,
    episode: bool,
    number: Option<f64>,
    season: Option<i64>,
}

/// The season a history key names: 3 for `S03E06`, None for a bare number,
/// which is the first season or a file that names none.
pub(super) fn season_of(key: &str) -> Option<i64> {
    let rest = key.strip_prefix(['S', 's'])?;
    let (season, _) = rest.split_once(['E', 'e'])?;
    season.parse().ok()
}

/// Of the files with an episode's number, the one of the room's season when
/// the key names one; else, or when none is of that season, only a file that is
/// the one with that number, since several seasons in one folder each have an
/// episode 6 and the lowest id is no answer.
fn one_of<T>(fits: &[T], want: Option<i64>, season: impl Fn(&T) -> Option<i64>) -> Option<&T> {
    if let Some(want) = want
        && let Some(f) = fits.iter().find(|f| season(f) == Some(want))
    {
        return Some(f);
    }
    match fits {
        [one] => Some(one),
        _ => None,
    }
}

/// The file of one series with the episode's number, as `one_of` picks it.
fn by_number(
    conn: &Connection,
    series: u64,
    number: f64,
    season: Option<i64>,
) -> Result<Option<(i64, String)>, CoreError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, episode_key, season FROM files WHERE series_id = ?1 AND kind = 'episode'
         AND abs(number - ?2) < 0.001 ORDER BY id",
    )?;
    let files = stmt
        .query_map(params![series as i64, number], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(one_of(&files, season, |(_, _, s)| *s).map(|(id, key, _)| (*id, key.clone())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        c.pragma_update(None, "foreign_keys", "ON").unwrap();
        anibeam_foundation::store::migrations::apply(&mut c).unwrap();
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
            number: Some(key.parse().unwrap_or(1.0)),
            series_title: "Show".into(),
            episode_title: None,
            duration: None,
            opening_start: None,
            film: false,
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
    fn a_copy_under_another_key_is_found_by_its_number() {
        let c = conn();
        series(&c, 1, Some(21), None);
        c.execute(
            "INSERT INTO files (id, series_id, path, size, mtime, kind, number, season, label, episode_key, sidecars, seen_at)
             VALUES (12, 1, '/lib/1/12.mkv', 0, 0, 'episode', 6, 3, 'EP 6', 'S03E06', '[]', 0)",
            [],
        )
        .unwrap();
        let found = find_local(&c, &episode(Some(21), None, "6")).unwrap();
        assert_eq!(found.file, Some(12));
    }

    fn file_of_season(c: &Connection, id: i64, season: i64, number: i64) {
        let key = if season == 1 {
            number.to_string()
        } else {
            format!("S{season:02}E{number:02}")
        };
        c.execute(
            "INSERT INTO files (id, series_id, path, size, mtime, kind, number, season, label, episode_key, sidecars, seen_at)
             VALUES (?1, 1, ?2, 0, 0, 'episode', ?3, ?4, 'EP', ?5, '[]', 0)",
            params![id, format!("/lib/1/{id}.mkv"), number, season, key],
        )
        .unwrap();
    }

    #[test]
    fn the_number_finds_the_room_s_season_and_never_guesses_between_seasons() {
        let c = conn();
        series(&c, 1, Some(21), None);
        file_of_season(&c, 10, 2, 6);
        file_of_season(&c, 12, 3, 6);
        // The room names season 3 under a key this folder writes otherwise
        let mut s3 = episode(Some(21), None, "6");
        s3.episode_key = "S3E6".into();
        assert_eq!(find_local(&c, &s3).unwrap().file, Some(12));
        // A season none of them is, or none named: two files have episode 6
        let mut s4 = s3.clone();
        s4.episode_key = "S04E06".into();
        assert_eq!(find_local(&c, &s4).unwrap().file, None);
        assert_eq!(
            find_local(&c, &episode(Some(21), None, "6")).unwrap().file,
            None
        );
        assert_eq!(season_of("S03E06"), Some(3));
        assert_eq!(season_of("12.5"), None);
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
            params![serde_json::to_string(&windows).unwrap(), anibeam_foundation::time::now_secs()],
        )
        .unwrap();
        let found = find_local(&c, &episode(Some(21), None, "1")).unwrap();
        assert_eq!(found.opening, Some(92.5));
    }

    #[test]
    fn a_session_plays_the_room_s_episode_only_in_the_same_season() {
        let at = |key: &str, number: f64| proto::EpisodeRef {
            number: Some(number),
            ..episode(None, Some(5), key)
        };
        assert!(plays(&at("S02E06", 6.0), &at("S02E06", 6.0)));
        // A copy filed under its season, or with a key that names none
        assert!(plays(&at("S03E06", 6.0), &at("6", 6.0)));
        assert!(plays(&at("6", 6.0), &at("S3E6", 6.0)));
        // The same number in another season is another episode
        assert!(!plays(&at("S01E06", 6.0), &at("S02E06", 6.0)));
        assert!(!plays(&at("S02E05", 5.0), &at("S02E06", 6.0)));
        // Another show altogether
        let mut other = at("S02E06", 6.0);
        other.tvmaze_id = Some(6);
        assert!(!plays(&other, &at("S02E06", 6.0)));
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
