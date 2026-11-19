pub mod migrations;
pub mod settings;
pub mod sql;
#[doc(hidden)]
pub mod testing;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use tokio::sync::oneshot;

use crate::sync::recover;
use anibeam_contract::CoreError;

type Task = Box<dyn FnOnce(&mut Connection) + Send + 'static>;

/// One SQLite file behind one writer connection on its own thread. Every
/// call's reads and writes go through `write` (`write_async` from a tokio
/// job); `reader` opens a fresh connection for a long read inside a job;
/// `read` runs a short read on the calling thread against one shared
/// read-only connection, so it never waits behind a long write transaction.
/// `Connection` is `Send` and not `Sync`, so only one thread at a time ever
/// touches a given connection; the `Mutex` around each one is what makes
/// that safe to share.
pub struct Store {
    path: PathBuf,
    tx: Mutex<Option<Sender<Task>>>,
    thread: Mutex<Option<JoinHandle<()>>>,
    reader_conn: Mutex<Connection>,
}

fn apply_pragmas(conn: &Connection) -> Result<(), CoreError> {
    let mode: String = conn.pragma_update_and_check(None, "journal_mode", "WAL", |r| r.get(0))?;
    if mode.to_lowercase() != "wal" {
        return Err(CoreError::Storage {
            message: format!(
                "journal_mode is {mode}, not wal; is the data directory on a network filesystem?"
            ),
        });
    }
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(())
}

/// A read-only connection to the file, with the writer's foreign keys and
/// busy timeout.
fn open_reader(path: &Path) -> Result<Connection, CoreError> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(conn)
}

/// `f` inside an immediate transaction, committed when it returns Ok and
/// rolled back when it fails.
fn in_tx<T>(
    conn: &mut Connection,
    f: impl FnOnce(&Transaction) -> Result<T, CoreError>,
) -> Result<T, CoreError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let out = f(&tx)?;
    tx.commit()?;
    Ok(out)
}

fn dropped_reply<E>(_: E) -> CoreError {
    CoreError::internal("store thread dropped the reply")
}

/// Waits on the writer's reply from whatever thread called. A plain thread
/// blocks on the receiver. Inside a runtime `blocking_recv` panics, and a
/// shell listener runs inline on the tokio thread that emitted the event,
/// so a listener that turns around and calls a writing call would take a
/// worker down: there the wait goes through `block_in_place`, which hands
/// the worker's other tasks to a sibling before it blocks.
fn recv_blocking<T>(rrx: oneshot::Receiver<T>) -> Result<T, CoreError> {
    // `block_in_place` itself panics on a current-thread runtime, so it is
    // only for the multi-thread one the core builds.
    let on_a_worker = matches!(
        tokio::runtime::Handle::try_current().map(|h| h.runtime_flavor()),
        Ok(tokio::runtime::RuntimeFlavor::MultiThread)
    );
    let received = if on_a_worker {
        tokio::task::block_in_place(|| rrx.blocking_recv())
    } else {
        rrx.blocking_recv()
    };
    received.map_err(dropped_reply)
}

impl Store {
    pub fn open(db_path: &Path) -> Result<Arc<Store>, CoreError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::io_at(parent.to_string_lossy(), e))?;
        }
        let mut conn = Connection::open(db_path)?;
        apply_pragmas(&conn)?;
        migrations::apply(&mut conn)?;

        // Kept open for the life of the store behind a mutex, so `read`
        // never opens a fresh file handle per call.
        let reader_conn = open_reader(db_path)?;

        let (tx, rx) = mpsc::channel::<Task>();
        let thread = std::thread::Builder::new()
            .name("anibeam-store".into())
            .spawn(move || {
                let mut conn = conn;
                while let Ok(task) = rx.recv() {
                    task(&mut conn);
                }
                let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)");
            })
            .map_err(|e| CoreError::internal(format!("store thread: {e}")))?;

        Ok(Arc::new(Store {
            path: db_path.to_path_buf(),
            tx: Mutex::new(Some(tx)),
            thread: Mutex::new(Some(thread)),
            reader_conn: Mutex::new(reader_conn),
        }))
    }

    fn send_task(&self, task: Task) -> Result<(), CoreError> {
        let sender = recover(&self.tx)
            .clone()
            .ok_or_else(|| CoreError::internal("store is closed"))?;
        sender
            .send(task)
            .map_err(|_| CoreError::internal("store thread is gone"))
    }

    /// Queues `f` on the writer thread; its result comes back on the receiver.
    fn queue<T, F>(&self, f: F) -> Result<oneshot::Receiver<Result<T, CoreError>>, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, CoreError> + Send + 'static,
    {
        let (rtx, rrx) = oneshot::channel();
        self.send_task(Box::new(move |conn| {
            let _ = rtx.send(f(conn));
        }))?;
        Ok(rrx)
    }

    /// Runs `f` on the writer thread and blocks the calling thread for the
    /// result. For a plain thread: a shell's calling thread, the event bus,
    /// a test. A job body on the runtime uses `write_async` instead, but
    /// this stays safe on a runtime thread all the same: see
    /// `recv_blocking`.
    pub fn write<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, CoreError> + Send + 'static,
    {
        recv_blocking(self.queue(f)?)?
    }

    /// The `write` a job body on the tokio runtime calls instead: same
    /// task, awaited rather than blocked on.
    pub async fn write_async<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, CoreError> + Send + 'static,
    {
        self.queue(f)?.await.map_err(dropped_reply)?
    }

    pub fn tx<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&Transaction) -> Result<T, CoreError> + Send + 'static,
    {
        self.write(move |conn| in_tx(conn, f))
    }

    /// The `tx` a job body on the tokio runtime calls instead.
    pub async fn tx_async<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&Transaction) -> Result<T, CoreError> + Send + 'static,
    {
        self.write_async(move |conn| in_tx(conn, f)).await
    }

    /// Queues `f` on the writer thread with no reply, for a fire-and-forget
    /// write: the event bus's log line, the image cache's bookkeeping.
    /// There is no reply channel to report failure through, so `f` is
    /// responsible for logging its own failure with `tracing::warn!`. If
    /// the store is closed, `f` is dropped silently.
    pub fn post<F>(&self, f: F)
    where
        F: FnOnce(&mut Connection) + Send + 'static,
    {
        let _ = self.send_task(Box::new(f));
    }

    /// Runs `f` on the calling thread against one shared read-only
    /// connection, the lock held only for the duration of `f`. For a call
    /// that only reads, so it never waits behind a long transaction on the
    /// writer thread.
    pub fn read<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        F: FnOnce(&Connection) -> Result<T, CoreError>,
    {
        // A panic inside `f` would otherwise poison the mutex for the rest
        // of the process. Recover it: a closure that only reads leaves the
        // connection itself perfectly usable, poisoned or not.
        let conn = recover(&self.reader_conn);
        f(&conn)
    }

    pub fn reader(&self) -> Result<Connection, CoreError> {
        open_reader(&self.path)
    }

    pub fn checkpoint(&self) {
        let _ = self.write(|c| {
            c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
            Ok(())
        });
    }

    /// Drops the sender so the thread drains and exits, then joins it.
    pub fn close(&self) {
        recover(&self.tx).take();
        // Taken out first, so a second close never waits on the lock for this join.
        let thread = recover(&self.thread).take();
        if let Some(t) = thread {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::types::FromSql;

    fn open_temp() -> (tempfile::TempDir, Arc<Store>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("data").join("anibeam.db")).unwrap();
        (dir, store)
    }

    /// A database migrated as far as `version` and holding `seed`, then
    /// opened as the store, which takes it through every later migration.
    fn migrated_from(version: usize, seed: &str) -> (tempfile::TempDir, Arc<Store>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("anibeam.db");
        let mut c = Connection::open(&path).unwrap();
        migrations::MIGRATIONS.to_version(&mut c, version).unwrap();
        let seeded: i64 = c
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            seeded as usize, version,
            "the seed sits at the version it names"
        );
        c.execute_batch(seed).unwrap();
        drop(c);
        let store = Store::open(&path).unwrap();
        (dir, store)
    }

    /// Every row `sql` returns, through `read`'s shared connection, so every
    /// test that reads this way also checks that connection sees what the
    /// writer committed, migrations included.
    fn rows<T>(store: &Store, sql: &str) -> Vec<T>
    where
        T: for<'a> TryFrom<&'a rusqlite::Row<'a>, Error = rusqlite::Error>,
    {
        store
            .read(|c| {
                let mut stmt = c.prepare(sql)?;
                let rows = stmt.query_map([], |r| T::try_from(r))?;
                Ok(rows.collect::<Result<Vec<T>, _>>()?)
            })
            .unwrap()
    }

    /// The first column of the one row `sql` returns, read the same way.
    fn one<T: FromSql>(store: &Store, sql: &str) -> T {
        store
            .read(|c| Ok(c.query_row(sql, [], |r| r.get(0))?))
            .unwrap()
    }

    /// The same, asked of the writer, for what only its own connection
    /// answers or what it has queued but not yet run.
    fn on_writer<T: FromSql + Send + 'static>(store: &Store, sql: &'static str) -> T {
        store
            .write(move |c| Ok(c.query_row(sql, [], |r| r.get(0))?))
            .unwrap()
    }

    #[test]
    fn open_creates_the_directory_and_sets_wal() {
        let (dir, store) = open_temp();
        assert!(dir.path().join("data").join("anibeam.db").exists());
        assert_eq!(on_writer::<String>(&store, "PRAGMA journal_mode"), "wal");
        assert_eq!(on_writer::<i64>(&store, "PRAGMA foreign_keys"), 1);
    }

    #[test]
    fn migrations_validate_and_reach_the_latest_version() {
        migrations::MIGRATIONS.validate().unwrap();
        let (_dir, store) = open_temp();
        assert_eq!(
            one::<i64>(&store, "PRAGMA user_version") as usize,
            migrations::SCHEMA_VERSION
        );
        let tables = |names: &str| -> i64 {
            one(
                &store,
                &format!(
                    "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name IN ({names})"
                ),
            )
        };
        assert_eq!(
            tables(
                "'sources','series','files','anilist_media','anilist_episodes','recommendations','relations','tracker_accounts','tracker_entries','views','completed','resume_points','skip_windows','settings','images','events'"
            ),
            16
        );
        assert_eq!(tables("'tvmaze_shows','tvmaze_episodes'"), 2);
        assert_eq!(tables("'imdb_ratings','imdb_links'"), 2);
    }

    /// A version 1 database takes every later migration in place: its
    /// series keep every column and gain an empty `tvmaze_id`, and the
    /// two TVmaze tables arrive empty.
    #[test]
    fn a_version_1_database_migrates_to_2_keeping_its_rows() {
        let (_dir, store) = migrated_from(
            1,
            "INSERT INTO sources (id, path, added_at) VALUES (1, '/tv', 0);
             INSERT INTO series (id, source_id, kind, path, folder_name, added_at, attempted_at, attempt_version)
             VALUES (58, 1, 'show', '/tv/Avatar Seven Havens', 'Avatar Seven Havens', 0, 100, 1);",
        );
        assert_eq!(
            one::<i64>(&store, "PRAGMA user_version") as usize,
            migrations::SCHEMA_VERSION
        );
        assert_eq!(
            rows::<(String, Option<i64>, Option<i64>)>(
                &store,
                "SELECT folder_name, attempted_at, tvmaze_id FROM series WHERE id = 58"
            ),
            [("Avatar Seven Havens".to_string(), Some(100), None)]
        );
        assert_eq!(one::<i64>(&store, "SELECT count(*) FROM tvmaze_shows"), 0);
    }

    /// A version 2 database keys every episode by its bare number. 0003
    /// moves a later season's files to `S02E01` keys and takes the history
    /// rows that can only have been theirs along: a folder holding season
    /// 2 alone keeps its watched episodes. Where season 1 carries the same
    /// number the row is ambiguous and stays season 1's.
    #[test]
    fn a_version_2_database_migrates_its_season_keys_keeping_unambiguous_history() {
        let (_dir, store) = migrated_from(
            2,
            "INSERT INTO sources (id, path, added_at) VALUES (1, '/tv', 0);
             INSERT INTO series (id, source_id, kind, path, folder_name, added_at) VALUES
               (1, 1, 'show', '/tv/Only S2', 'Only S2', 0),
               (2, 1, 'show', '/tv/Both', 'Both', 0);
             INSERT INTO files (id, series_id, path, size, mtime, kind, number, season, label, episode_key, sidecars, seen_at) VALUES
               (10, 1, '/tv/Only S2/S02E01.mkv', 0, 0, 'episode', 1, 2, 'S02E01', '1', '[]', 0),
               (11, 1, '/tv/Only S2/S02E02.mkv', 0, 0, 'episode', 2, 2, 'S02E02', '2', '[]', 0),
               (20, 2, '/tv/Both/S01E01.mkv', 0, 0, 'episode', 1, 1, 'S01E01', '1', '[]', 0),
               (21, 2, '/tv/Both/S02E01.mkv', 0, 0, 'episode', 1, 2, 'S02E01', '1', '[]', 0),
               (22, 2, '/tv/Both/S02E02.mkv', 0, 0, 'episode', 2, 2, 'S02E02', '2', '[]', 0),
               (23, 2, '/tv/Both/OP1.mkv', 0, 0, 'extra', 0, NULL, 'OP1', 'OP1.mkv', '[]', 0);
             INSERT INTO completed (series_id, episode_key, at) VALUES (1, '1', 5), (2, '1', 6), (2, '2', 7), (2, 'OP1.mkv', 8);
             INSERT INTO resume_points (series_id, episode_key, position, duration, at) VALUES (1, '2', 10, 100, 5);
             INSERT INTO views (series_id, episode_key, at) VALUES (1, '2', 5), (2, '1', 6);",
        );
        let keys = |sql: &str| -> Vec<(i64, String)> { rows(&store, sql) };
        let key = |n: i64, k: &str| (n, k.to_string());
        assert_eq!(
            keys("SELECT id, episode_key FROM files ORDER BY id"),
            [
                key(10, "S02E01"),
                key(11, "S02E02"),
                key(20, "1"),
                key(21, "S02E01"),
                key(22, "S02E02"),
                key(23, "OP1.mkv"),
            ]
        );
        assert_eq!(
            keys("SELECT series_id, episode_key FROM completed ORDER BY series_id, at"),
            [
                key(1, "S02E01"),
                key(2, "1"),
                key(2, "S02E02"),
                key(2, "OP1.mkv")
            ]
        );
        assert_eq!(
            keys("SELECT series_id, episode_key FROM resume_points"),
            [key(1, "S02E02")]
        );
        assert_eq!(
            keys("SELECT series_id, episode_key FROM views ORDER BY series_id"),
            [key(1, "S02E02"), key(2, "1")]
        );
    }

    /// Version 5 drops the stored events that carry an indexer API key,
    /// written while transport errors still quoted the request URL, and
    /// keeps every other event.
    #[test]
    fn a_version_4_database_loses_only_the_events_with_an_api_key() {
        let (_dir, store) = migrated_from(
            4,
            "INSERT INTO events (seq, at, level, stage, message, body) VALUES
               (1, 0, 'Warn', 'Downloads', 'feed failed: error sending request for url (https://x/api?t=tvsearch&apikey=abc)', '{}'),
               (2, 0, 'Warn', 'Downloads', 'feed failed', '{\"message\":\"https://x/api?apikey=abc\"}'),
               (3, 0, 'Info', 'Library', 'Every folder: nothing changed', '{}');",
        );
        assert_eq!(
            rows::<(i64,)>(&store, "SELECT seq FROM events ORDER BY seq"),
            [(3,)]
        );
    }

    /// Version 6 clears a match to a short (under five minutes, or MUSIC)
    /// with no attempt left behind, so the auto-match takes the series
    /// again, and leaves every other match, confirmed or not, as it was.
    #[test]
    fn a_version_5_database_unmatches_its_shorts_for_the_auto_match() {
        let (_dir, store) = migrated_from(
            5,
            "INSERT INTO sources (id, path, added_at) VALUES (1, '/a', 0);
             INSERT INTO anilist_media (id, format, duration, fetched_at, raw) VALUES
               (97962, 'SPECIAL', 1, 0, '{}'),
               (21519, 'MOVIE', 106, 0, '{}'),
               (500, 'MUSIC', NULL, 0, '{}'),
               (600, 'SPECIAL', 24, 0, '{}');
             INSERT INTO series (id, source_id, kind, path, folder_name, added_at,
                                 provider, anilist_id, mal_id, confirmed, matched_at) VALUES
               (1, 1, 'movie', '/a/Kimi no Na Wa.mkv', 'Kimi no Na Wa', 0, 'anilist', 97962, 33902, 1, 5),
               (2, 1, 'movie', '/a/Koe no Katachi.mkv', 'Koe no Katachi', 0, 'anilist', 21519, 28851, 1, 5),
               (3, 1, 'show', '/a/OP', 'OP', 0, 'anilist', 500, NULL, 1, 5),
               (4, 1, 'show', '/a/Specials', 'Specials', 0, 'anilist', 600, NULL, 0, 5);",
        );
        assert_eq!(
            rows::<(i64, Option<i64>, i64, Option<i64>)>(
                &store,
                "SELECT id, anilist_id, confirmed, attempted_at FROM series
                 WHERE provider IS NULL OR anilist_id IS NOT NULL ORDER BY id"
            ),
            [
                (1, None, 0, None),
                (2, Some(21519), 1, None),
                (3, None, 0, None),
                (4, Some(600), 0, None),
            ]
        );
    }

    #[test]
    fn a_newer_database_is_refused_naming_both_versions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("anibeam.db");
        Connection::open(&path)
            .unwrap()
            .pragma_update(None, "user_version", 99)
            .unwrap();
        match Store::open(&path).err().unwrap() {
            CoreError::Storage { message } => {
                assert!(message.contains("99"), "{message}");
                assert!(
                    message.contains(&migrations::SCHEMA_VERSION.to_string()),
                    "{message}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    const ADD_SOURCE: &str = "INSERT INTO sources (path, added_at) VALUES ('/a', 1)";
    const SOURCES: &str = "SELECT count(*) FROM sources";

    #[test]
    fn tx_rolls_back_on_error_and_commits_on_ok() {
        let (_dir, store) = open_temp();
        let err = store.tx(|t| {
            t.execute(ADD_SOURCE, [])?;
            Err::<(), _>(CoreError::internal("boom"))
        });
        assert!(err.is_err());
        assert_eq!(one::<i64>(&store, SOURCES), 0);
        store
            .tx(|t| {
                t.execute(ADD_SOURCE, [])?;
                Ok(())
            })
            .unwrap();
        assert_eq!(one::<i64>(&store, SOURCES), 1);
    }

    /// A committed write is seen by a fresh `reader` and by `read`'s shared
    /// connection alike.
    #[test]
    fn readers_see_committed_writes() {
        let (_dir, store) = open_temp();
        store
            .tx(|t| {
                t.execute(ADD_SOURCE, [])?;
                Ok(())
            })
            .unwrap();
        let fresh: i64 = store
            .reader()
            .unwrap()
            .query_row(SOURCES, [], |r| r.get(0))
            .unwrap();
        let shared: i64 = store
            .read(|c| Ok(c.query_row(SOURCES, [], |r| r.get(0))?))
            .unwrap();
        assert_eq!((fresh, shared), (1, 1));
    }

    #[test]
    fn settings_round_trip_json() {
        let (_dir, store) = open_temp();
        store
            .write(|c| settings::set(c, settings::AUTO_MATCH_VERSION, &3u32))
            .unwrap();
        let v: Option<u32> = store
            .write(|c| settings::get(c, settings::AUTO_MATCH_VERSION))
            .unwrap();
        assert_eq!(v, Some(3));
        let missing: Option<u32> = store.write(|c| settings::get(c, "nope")).unwrap();
        assert_eq!(missing, None);
        store
            .write(|c| settings::remove(c, settings::AUTO_MATCH_VERSION))
            .unwrap();
        let fallback: u32 = store
            .write(|c| settings::get_or_default(c, settings::AUTO_MATCH_VERSION))
            .unwrap();
        assert_eq!(fallback, 0);
    }

    #[test]
    fn post_is_seen_by_a_write_queued_after_it() {
        let (_dir, store) = open_temp();
        store.post(|c| {
            if let Err(e) = c.execute(ADD_SOURCE, []) {
                tracing::warn!("posted write failed: {e}");
            }
        });
        // The writer thread runs tasks in the order they were queued, so
        // this write only sees the posted row if post really ran first.
        assert_eq!(on_writer::<i64>(&store, SOURCES), 1);
    }

    #[test]
    fn read_recovers_from_a_panic_in_the_closure() {
        let (_dir, store) = open_temp();
        let store_for_panic = Arc::clone(&store);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            store_for_panic.read(|_conn| -> Result<(), CoreError> {
                panic!("boom, mid read");
            })
        }));
        assert!(result.is_err());

        // The mutex would be poisoned here without the recovery in `read`;
        // this must not panic at the lock.
        let n: i64 = store
            .read(|c| Ok(c.query_row(SOURCES, [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(n, 0);
    }

    /// A shell listener runs inline on the tokio thread that emitted the
    /// event, and a listener is free to call back into the core, so the
    /// blocking `write` has to survive a runtime worker. Without
    /// `block_in_place` the spawned task panics on `blocking_recv` and the
    /// join below fails.
    #[test]
    fn write_survives_a_call_from_a_runtime_worker() {
        let (_dir, store) = open_temp();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let inner = Arc::clone(&store);
        let n: i64 = rt
            .block_on(async move {
                tokio::spawn(async move {
                    inner.write(|c| {
                        c.execute(ADD_SOURCE, [])?;
                        Ok(c.query_row(SOURCES, [], |r| r.get(0))?)
                    })
                })
                .await
                .expect("the task must not panic on the writer round trip")
            })
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn write_async_and_tx_async_run_from_a_tokio_runtime() {
        let (_dir, store) = open_temp();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(store.write_async(|c| {
            c.execute(ADD_SOURCE, [])?;
            Ok(())
        }))
        .unwrap();
        let failed = rt.block_on(store.tx_async(|t| {
            t.execute("INSERT INTO sources (path, added_at) VALUES ('/b', 1)", [])?;
            Err::<(), _>(CoreError::internal("boom"))
        }));
        assert!(failed.is_err(), "a failed tx_async rolls back");
        let n: i64 = rt
            .block_on(store.tx_async(|t| Ok(t.query_row(SOURCES, [], |r| r.get(0))?)))
            .unwrap();
        assert_eq!(n, 1);
    }

    /// Once closed, the store refuses work rather than hanging, drops what is
    /// posted to it, and a second close is a no-op.
    #[test]
    fn a_closed_store_refuses_writes_and_closes_twice() {
        let (dir, store) = open_temp();
        store.close();
        assert!(store.write(|_| Ok(())).is_err());
        assert!(store.tx(|_| Ok(())).is_err());
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(store.write_async(|_| Ok(()))).is_err());
        assert!(rt.block_on(store.tx_async(|_| Ok(()))).is_err());
        store.post(|c| {
            let _ = c.execute(ADD_SOURCE, []);
        });
        store.close();
        let reopened = Store::open(&dir.path().join("data").join("anibeam.db")).unwrap();
        assert_eq!(
            one::<i64>(&reopened, SOURCES),
            0,
            "a closed store runs nothing posted to it"
        );
    }

    /// The watch log (#593) starts from the history already there: every
    /// completion a database at the version before it holds becomes one
    /// finish, copied once, and `completed` keeps every row it had.
    #[test]
    fn the_version_before_the_watch_log_copies_its_history_in_keeping_every_row() {
        let (_dir, store) = migrated_from(
            migrations::WATCH_LOG - 1,
            "INSERT INTO sources (id, path, added_at) VALUES (1, '/tv', 0);
             INSERT INTO series (id, source_id, kind, path, folder_name, added_at) VALUES
               (1, 1, 'show', '/tv/A', 'A', 0),
               (2, 1, 'show', '/tv/B', 'B', 0);
             INSERT INTO completed (series_id, episode_key, at) VALUES (1, '1', 5), (1, '2', 6), (2, 'S02E01', 7);
             INSERT INTO views (series_id, episode_key, at) VALUES (1, '2', 6);
             INSERT INTO resume_points (series_id, episode_key, position, duration, at) VALUES (2, 'S02E02', 10, 100, 8);",
        );
        let history = |sql: &str| -> Vec<(i64, String, i64)> { rows(&store, sql) };
        let row = |s: i64, k: &str, at: i64| (s, k.to_string(), at);
        let finished = [row(1, "1", 5), row(1, "2", 6), row(2, "S02E01", 7)];
        assert_eq!(
            history("SELECT series_id, episode_key, at FROM completed ORDER BY at"),
            finished
        );
        assert_eq!(
            history(
                "SELECT series_id, episode_key, at FROM watch_log WHERE how = 'copied' ORDER BY at"
            ),
            finished
        );
        assert_eq!(one::<i64>(&store, "SELECT count(*) FROM watch_log"), 3);
        assert_eq!(
            history("SELECT series_id, episode_key, at FROM views"),
            [row(1, "2", 6)]
        );
        assert_eq!(
            history("SELECT series_id, episode_key, at FROM resume_points"),
            [row(2, "S02E02", 8)]
        );
    }

    /// #630: history, resume points and tracker accounts gain a profile. A database at
    /// the version before keeps every row, each now the owner's (profile 1); a write that
    /// names no profile is the owner's; and a second person's row for the same episode
    /// sits beside the owner's instead of replacing it.
    #[test]
    fn the_version_before_profiles_keeps_every_row_as_the_owners() {
        let (_dir, store) = migrated_from(
            migrations::PROFILES - 1,
            "INSERT INTO sources (id, path, added_at) VALUES (1, '/tv', 0);
             INSERT INTO series (id, source_id, kind, path, folder_name, added_at) VALUES (1, 1, 'show', '/tv/A', 'A', 0);
             INSERT INTO views (series_id, episode_key, at) VALUES (1, '2', 6);
             INSERT INTO completed (series_id, episode_key, at) VALUES (1, '1', 5), (1, '2', 6);
             INSERT INTO watch_log (series_id, episode_key, at, how) VALUES (1, '1', 4, 'played'), (1, '1', 5, 'played');
             INSERT INTO resume_points (series_id, episode_key, position, duration, at) VALUES (1, '3', 10, 100, 8);
             INSERT INTO tracker_accounts (tracker, user_id, username, connected_at, secret_store) VALUES ('anilist', 42, 'owner', 1, 'file');
             INSERT INTO tracker_entries (tracker, media_id, status, progress, fetched_at, notes) VALUES ('anilist', 7, 'CURRENT', 2, 9, 'good');",
        );
        let count = |sql: &str| -> i64 { one(&store, sql) };
        for (table, rows) in [
            ("views", 1),
            ("completed", 2),
            ("watch_log", 2),
            ("resume_points", 1),
            ("tracker_accounts", 1),
            ("tracker_entries", 1),
        ] {
            assert_eq!(
                count(&format!("SELECT count(*) FROM {table}")),
                rows,
                "{table}"
            );
            assert_eq!(
                count(&format!(
                    "SELECT count(*) FROM {table} WHERE profile_id = 1"
                )),
                rows,
                "{table} is the owner's"
            );
        }
        assert_eq!(count("SELECT count(*) FROM profiles"), 1);
        assert_eq!(
            count("SELECT count(*) FROM tracker_entries WHERE notes = 'good' AND progress = 2"),
            1
        );
        assert_eq!(
            count(
                "SELECT count(*) FROM tracker_accounts WHERE username = 'owner' AND secret_store = 'file'"
            ),
            1
        );
        store
            .write(|c| {
                c.execute_batch(
                    "INSERT INTO profiles (id, tailnet_user, created_at) VALUES (2, 'u2', 0);
                     INSERT INTO completed (profile_id, series_id, episode_key, at) VALUES (2, 1, '1', 50);
                     INSERT INTO completed (series_id, episode_key, at) VALUES (1, '3', 7);
                     INSERT INTO tracker_accounts (profile_id, tracker, username) VALUES (2, 'anilist', 'friend');",
                )?;
                Ok(())
            })
            .unwrap();
        assert_eq!(
            count("SELECT count(*) FROM completed WHERE episode_key = '1'"),
            2
        );
        assert_eq!(
            count("SELECT profile_id FROM completed WHERE episode_key = '3'"),
            1,
            "a write naming no profile is the owner's"
        );
        assert_eq!(
            count("SELECT at FROM completed WHERE profile_id = 1 AND episode_key = '1'"),
            5
        );
        assert_eq!(count("SELECT count(*) FROM tracker_accounts"), 2);
    }

    /// Review of #653: the profile comes last in each key, so a read or a delete by
    /// series, or by tracker, still finds its rows through the key instead of a scan.
    #[test]
    fn history_reads_by_series_keep_their_index() {
        let (_dir, store) = open_temp();
        let plans: Vec<String> = store
            .read(|c| {
                let mut out = Vec::new();
                for sql in [
                    "SELECT at FROM completed WHERE series_id = 1",
                    "DELETE FROM resume_points WHERE series_id = 1 AND episode_key = '1'",
                    "SELECT at FROM views WHERE series_id = 1",
                    "SELECT 1 FROM watch_log WHERE series_id = 1 AND episode_key = '1' AND at = 5",
                    "SELECT 1 FROM tracker_entries WHERE tracker = 'anilist' AND media_id = 7",
                    "SELECT 1 FROM tracker_accounts WHERE tracker = 'anilist'",
                ] {
                    let mut stmt = c.prepare(&format!("EXPLAIN QUERY PLAN {sql}"))?;
                    let detail: Vec<String> = stmt
                        .query_map([], |r| r.get::<_, String>(3))?
                        .collect::<Result<_, _>>()?;
                    out.push(format!("{sql} => {}", detail.join("; ")));
                }
                Ok(out)
            })
            .unwrap();
        for plan in plans {
            assert!(!plan.contains("=> SCAN"), "{plan}");
        }
    }
}
