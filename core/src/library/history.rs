//! Watch history follows its file. History is keyed by series and episode key,
//! never by file id, so when a file's key changes (a better reading of its
//! name, the series' record, a quick fix) the history under the old key has to
//! move with it, or the file reads unwatched and its resume point is lost.
//!
//! A key that a file which did not move still carries keeps its history: that
//! is a second release of the same episode, and the history is its as much as
//! the moved file's.
//!
//! Two keys that land on one merge (#468): `SP01.mkv` and `SP01v2.mkv` read as
//! one extra now, so each file's history becomes the one key's, once. The
//! newest watched date and the furthest resume point win, and the watch log
//! keeps the finishes of both.

use std::collections::HashSet;

use rusqlite::{Transaction, params};

use crate::contract::CoreError;

/// One file whose key changed.
pub(crate) struct Move {
    pub file: u64,
    pub from: String,
    pub to: String,
}

/// The tables keyed by series and episode key. `views` keys by series alone
/// and is moved on its own.
const KEYED: [&str; 3] = ["completed", "resume_points", "skip_windows"];

/// The tables whose rows under a key just move, with nothing to merge: the
/// view, and the watch log, where two finishes are two rows wherever they land.
const LOOSE: [&str; 2] = ["views", "watch_log"];

/// Where a finish in the watch log came from (#593). The migration that made
/// the log writes a fourth, `copied`, for the history from before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum How {
    /// The player reached the end of the episode here.
    Played,
    /// Progress set by hand. Stats leaves it out.
    Marked,
    /// An export file's completion.
    Imported,
}

impl How {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            How::Played => "played",
            How::Marked => "marked",
            How::Imported => "imported",
        }
    }
}

/// Appends one finish of an episode to the watch log. `completed` keeps the
/// latest finish on its own; the log keeps every one.
pub(crate) fn log_finish(
    conn: &rusqlite::Connection,
    series: u64,
    key: &str,
    at: i64,
    how: How,
) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO watch_log (series_id, episode_key, at, how) VALUES (?1, ?2, ?3, ?4)",
        params![series as i64, key, at, how.as_str()],
    )?;
    Ok(())
}

/// Moves each file's history from its old key to its new one, once every row
/// of the series carries its new key. Two steps, through keys no file can
/// carry, so a chain of moves (1 to 2, 2 to 3) never lands on itself. History
/// that lands on a key holding history of its own merges into it: the newer
/// completion, the resume point further in, and the skip windows already
/// there.
pub(crate) fn follow(tx: &Transaction, series: u64, moves: &[Move]) -> Result<(), CoreError> {
    if moves.iter().all(|m| m.from == m.to) {
        return Ok(());
    }
    let moved: HashSet<u64> = moves.iter().map(|m| m.file).collect();
    let kept: HashSet<String> = {
        let mut stmt = tx.prepare("SELECT id, episode_key FROM files WHERE series_id = ?1")?;
        let rows = stmt.query_map(params![series as i64], |r| {
            Ok((r.get::<_, i64>(0)? as u64, r.get::<_, String>(1)?))
        })?;
        let mut kept = HashSet::new();
        for row in rows {
            let (id, key) = row?;
            if !moved.contains(&id) {
                kept.insert(key);
            }
        }
        kept
    };
    let moves: Vec<&Move> = moves
        .iter()
        .filter(|m| m.from != m.to && !kept.contains(&m.from))
        .collect();
    // A video file's name always has an extension, so no key looks like this.
    let parked = |i: usize| format!("~moving~{i}");
    for (i, m) in moves.iter().enumerate() {
        for table in KEYED {
            tx.execute(
                &format!(
                    "UPDATE {table} SET episode_key = ?3 WHERE series_id = ?1 AND episode_key = ?2"
                ),
                params![series as i64, m.from, parked(i)],
            )?;
        }
        for table in LOOSE {
            tx.execute(
                &format!(
                    "UPDATE {table} SET episode_key = ?3 WHERE series_id = ?1 AND episode_key = ?2"
                ),
                params![series as i64, m.from, parked(i)],
            )?;
        }
    }
    for (i, m) in moves.iter().enumerate() {
        let (series, parked, to) = (series as i64, parked(i), m.to.as_str());
        // The new key's own history, where it has some, takes the better of the two.
        tx.execute(
            "UPDATE completed SET at = max(at, (SELECT at FROM completed WHERE series_id = ?1 AND episode_key = ?2))
             WHERE series_id = ?1 AND episode_key = ?3
               AND EXISTS (SELECT 1 FROM completed WHERE series_id = ?1 AND episode_key = ?2)",
            params![series, parked, to],
        )?;
        tx.execute(
            "UPDATE resume_points SET (position, duration, at) =
               (SELECT position, duration, at FROM resume_points WHERE series_id = ?1 AND episode_key = ?2)
             WHERE series_id = ?1 AND episode_key = ?3
               AND position < (SELECT position FROM resume_points WHERE series_id = ?1 AND episode_key = ?2)",
            params![series, parked, to],
        )?;
        for table in KEYED {
            tx.execute(
                &format!(
                    "UPDATE OR IGNORE {table} SET episode_key = ?3 WHERE series_id = ?1 AND episode_key = ?2"
                ),
                params![series, parked, to],
            )?;
            // What is left had a row under the new key already, merged above.
            tx.execute(
                &format!("DELETE FROM {table} WHERE series_id = ?1 AND episode_key = ?2"),
                params![series, parked],
            )?;
        }
        for table in LOOSE {
            tx.execute(
                &format!(
                    "UPDATE {table} SET episode_key = ?3 WHERE series_id = ?1 AND episode_key = ?2"
                ),
                params![series, parked, to],
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn setup(
        keys: &[&str],
        watched: &[&str],
    ) -> (tempfile::TempDir, std::sync::Arc<Store>, Vec<u64>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("anibeam.db")).unwrap();
        let keys: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
        let watched: Vec<String> = watched.iter().map(|k| k.to_string()).collect();
        let ids = store
            .tx(move |tx| {
                tx.execute("INSERT INTO sources (path, available, added_at) VALUES ('/lib', 1, 0)", [])?;
                tx.execute(
                    "INSERT INTO series (source_id, kind, path, folder_name, added_at) VALUES (1, 'show', '/lib/S', 'S', 0)",
                    [],
                )?;
                let mut ids = Vec::new();
                for (i, key) in keys.iter().enumerate() {
                    tx.execute(
                        "INSERT INTO files (series_id, path, size, mtime, kind, number, label, episode_key, seen_at)
                         VALUES (1, ?1, 1, 1, 'episode', 1, 'x', ?2, 0)",
                        params![format!("/lib/S/{i}.mkv"), key],
                    )?;
                    ids.push(tx.last_insert_rowid() as u64);
                }
                for (i, key) in watched.iter().enumerate() {
                    tx.execute(
                        "INSERT INTO completed (series_id, episode_key, at) VALUES (1, ?1, ?2)",
                        params![key, i as i64],
                    )?;
                }
                Ok(ids)
            })
            .unwrap();
        (dir, store, ids)
    }

    fn set_keys(store: &Store, ids: &[u64], keys: &[&str]) {
        let pairs: Vec<(u64, String)> = ids
            .iter()
            .copied()
            .zip(keys.iter().map(|k| k.to_string()))
            .collect();
        store
            .tx(move |tx| {
                for (id, key) in &pairs {
                    tx.execute(
                        "UPDATE files SET episode_key = ?2 WHERE id = ?1",
                        params![*id as i64, key],
                    )?;
                }
                Ok(())
            })
            .unwrap();
    }

    /// The completion stamps by key, which say which file's history landed where.
    fn completed(store: &Store) -> Vec<(String, i64)> {
        store
            .read(|c| {
                let mut stmt =
                    c.prepare("SELECT episode_key, at FROM completed ORDER BY episode_key")?;
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            })
            .unwrap()
    }

    fn moves(ids: &[u64], from: &[&str], to: &[&str]) -> Vec<Move> {
        ids.iter()
            .zip(from.iter().zip(to.iter()))
            .map(|(id, (f, t))| Move {
                file: *id,
                from: f.to_string(),
                to: t.to_string(),
            })
            .collect()
    }

    #[test]
    fn a_renumbered_run_takes_its_history_along() {
        // 1, 2, 3 watched, then read as 2, 3, 4 in one go.
        let (_dir, store, ids) = setup(&["1", "2", "3"], &["1", "2", "3"]);
        set_keys(&store, &ids, &["2", "3", "4"]);
        let m = moves(&ids, &["1", "2", "3"], &["2", "3", "4"]);
        store.tx(move |tx| follow(tx, 1, &m)).unwrap();
        assert_eq!(
            completed(&store),
            vec![("2".into(), 0), ("3".into(), 1), ("4".into(), 2)]
        );
    }

    #[test]
    fn a_key_another_file_keeps_keeps_its_history() {
        // Two releases of episode 5; one is read as 6 now, the other stays 5.
        let (_dir, store, ids) = setup(&["5", "5"], &["5"]);
        set_keys(&store, &ids[..1], &["6"]);
        let m = moves(&ids[..1], &["5"], &["6"]);
        store.tx(move |tx| follow(tx, 1, &m)).unwrap();
        assert_eq!(completed(&store), vec![("5".into(), 0)]);
    }

    #[test]
    fn a_file_name_key_becomes_a_number_and_the_view_follows() {
        let (_dir, store, ids) = setup(&["OVA 1.mkv"], &["OVA 1.mkv"]);
        store
            .tx(|tx| {
                tx.execute("INSERT INTO views (series_id, episode_key, at) VALUES (1, 'OVA 1.mkv', 9)", [])?;
                tx.execute(
                    "INSERT INTO resume_points (series_id, episode_key, position, duration, at) VALUES (1, 'OVA 1.mkv', 60, 1500, 9)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        set_keys(&store, &ids, &["1"]);
        let m = moves(&ids, &["OVA 1.mkv"], &["1"]);
        store.tx(move |tx| follow(tx, 1, &m)).unwrap();
        assert_eq!(completed(&store), vec![("1".into(), 0)]);
        let (view, resume): (String, String) = store
            .read(|c| {
                Ok((
                    c.query_row("SELECT episode_key FROM views", [], |r| r.get(0))?,
                    c.query_row("SELECT episode_key FROM resume_points", [], |r| r.get(0))?,
                ))
            })
            .unwrap();
        assert_eq!((view.as_str(), resume.as_str()), ("1", "1"));
    }

    /// The watch log keeps every finish, so each file's finishes move with
    /// it, and two keys that merge keep the finishes of both.
    #[test]
    fn the_watch_log_follows_its_file_and_keeps_every_finish() {
        let (_dir, store, ids) = setup(&["1", "2", "SP01.mkv"], &[]);
        store
            .tx(|tx| {
                tx.execute_batch(
                    "INSERT INTO watch_log (series_id, episode_key, at, how) VALUES
                       (1, '1', 1, 'played'), (1, '1', 2, 'played'), (1, '2', 3, 'played'),
                       (1, 'SP01.mkv', 4, 'played'), (1, 'SP01v2.mkv', 5, 'played');",
                )?;
                Ok(())
            })
            .unwrap();
        // 1 and 2 read as 2 and 3; SP01.mkv lands on SP01v2.mkv's history.
        set_keys(&store, &ids, &["2", "3", "SP01v2.mkv"]);
        let m = moves(&ids, &["1", "2", "SP01.mkv"], &["2", "3", "SP01v2.mkv"]);
        store.tx(move |tx| follow(tx, 1, &m)).unwrap();
        let log: Vec<(String, i64)> = store
            .read(|c| {
                let mut stmt = c.prepare("SELECT episode_key, at FROM watch_log ORDER BY at")?;
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            })
            .unwrap();
        assert_eq!(
            log,
            vec![
                ("2".into(), 1),
                ("2".into(), 2),
                ("3".into(), 3),
                ("SP01v2.mkv".into(), 4),
                ("SP01v2.mkv".into(), 5),
            ]
        );
    }

    #[test]
    fn history_landing_on_history_merges_into_it() {
        // The new key already has history of its own (an import, say): one
        // episode's history now, under the newer completion.
        let (_dir, store, ids) = setup(&["63"], &["63", "1"]);
        set_keys(&store, &ids, &["1"]);
        let m = moves(&ids, &["63"], &["1"]);
        store.tx(move |tx| follow(tx, 1, &m)).unwrap();
        assert_eq!(completed(&store), vec![("1".into(), 1)]);
    }

    fn resume(store: &Store, key: &str, position: f64, at: i64) {
        let key = key.to_string();
        store
            .tx(move |tx| {
                tx.execute(
                    "INSERT INTO resume_points (series_id, episode_key, position, duration, at) VALUES (1, ?1, ?2, 1500, ?3)",
                    params![key, position, at],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn resumes(store: &Store) -> Vec<(String, f64, i64)> {
        store
            .read(|c| {
                let mut stmt = c.prepare(
                    "SELECT episode_key, position, at FROM resume_points ORDER BY episode_key",
                )?;
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            })
            .unwrap()
    }

    #[test]
    fn two_revisions_of_an_extra_merge_their_history_once() {
        // SP01 and SP01v2 each kept their own history while they read as two
        // extras (#468). Read as one, the v2's history lands on the v1's key:
        // the newer watched date and the resume point further in win.
        let (_dir, store, ids) = setup(
            &["S - SP01.mkv", "S - SP01v2.mkv"],
            &["S - SP01.mkv", "S - SP01v2.mkv"],
        );
        resume(&store, "S - SP01.mkv", 600.0, 5);
        resume(&store, "S - SP01v2.mkv", 100.0, 9);
        set_keys(&store, &ids[1..], &["S - SP01.mkv"]);
        let m = moves(&ids[1..], &["S - SP01v2.mkv"], &["S - SP01.mkv"]);
        store.tx(move |tx| follow(tx, 1, &m)).unwrap();
        assert_eq!(completed(&store), vec![("S - SP01.mkv".into(), 1)]);
        assert_eq!(resumes(&store), vec![("S - SP01.mkv".into(), 600.0, 5)]);

        // The other way round, the moved file's resume point is the further one.
        let (_dir, store, ids) = setup(&["S - SP01.mkv", "S - SP01v2.mkv"], &["S - SP01v2.mkv"]);
        resume(&store, "S - SP01.mkv", 100.0, 5);
        resume(&store, "S - SP01v2.mkv", 900.0, 9);
        set_keys(&store, &ids[1..], &["S - SP01.mkv"]);
        let m = moves(&ids[1..], &["S - SP01v2.mkv"], &["S - SP01.mkv"]);
        store.tx(move |tx| follow(tx, 1, &m)).unwrap();
        assert_eq!(completed(&store), vec![("S - SP01.mkv".into(), 0)]);
        assert_eq!(resumes(&store), vec![("S - SP01.mkv".into(), 900.0, 9)]);
    }
}
