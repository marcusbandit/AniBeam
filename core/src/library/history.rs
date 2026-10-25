//! Watch history follows its file. History is keyed by series and episode key,
//! never by file id, so when a file's key changes (a better reading of its
//! name, the series' record, a quick fix) the history under the old key has to
//! move with it, or the file reads unwatched and its resume point is lost.
//!
//! A key that a file which did not move still carries keeps its history: that
//! is a second release of the same episode, and the history is its as much as
//! the moved file's.

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

/// Moves each file's history from its old key to its new one, once every row
/// of the series carries its new key. Two steps, through keys no file can
/// carry, so a chain of moves (1 to 2, 2 to 3) never lands on itself. History
/// that cannot land, because the new key holds history of its own, goes back
/// where it was.
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
        tx.execute(
            "UPDATE views SET episode_key = ?3 WHERE series_id = ?1 AND episode_key = ?2",
            params![series as i64, m.from, parked(i)],
        )?;
    }
    for (i, m) in moves.iter().enumerate() {
        for table in KEYED {
            for (from, to) in [(parked(i), m.to.as_str()), (parked(i), m.from.as_str())] {
                tx.execute(
                    &format!(
                        "UPDATE OR IGNORE {table} SET episode_key = ?3 WHERE series_id = ?1 AND episode_key = ?2"
                    ),
                    params![series as i64, from, to],
                )?;
            }
            // Neither key could take it: the same episode's history is on both.
            tx.execute(
                &format!("DELETE FROM {table} WHERE series_id = ?1 AND episode_key = ?2"),
                params![series as i64, parked(i)],
            )?;
        }
        tx.execute(
            "UPDATE views SET episode_key = ?3 WHERE series_id = ?1 AND episode_key = ?2",
            params![series as i64, parked(i), m.to],
        )?;
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

    #[test]
    fn history_that_cannot_land_stays_where_it_was() {
        // The new key already has history of its own (an import, say).
        let (_dir, store, ids) = setup(&["63"], &["63", "1"]);
        set_keys(&store, &ids, &["1"]);
        let m = moves(&ids, &["63"], &["1"]);
        store.tx(move |tx| follow(tx, 1, &m)).unwrap();
        assert_eq!(completed(&store), vec![("1".into(), 1), ("63".into(), 0)]);
    }
}
