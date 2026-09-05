pub mod migrations;
pub mod settings;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use tokio::sync::oneshot;

use crate::contract::CoreError;

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

impl Store {
    pub fn open(db_path: &Path) -> Result<Arc<Store>, CoreError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::io_at(parent.to_string_lossy(), e))?;
        }
        let mut conn = Connection::open(db_path)?;
        apply_pragmas(&conn)?;
        migrations::apply(&mut conn)?;

        // Same flags and pragmas as `reader()`, kept open for the life of
        // the store behind a mutex so `read` never opens a fresh file
        // handle per call.
        let reader_conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        reader_conn.pragma_update(None, "foreign_keys", "ON")?;
        reader_conn.busy_timeout(Duration::from_secs(5))?;

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

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn send_task(&self, task: Task) -> Result<(), CoreError> {
        let sender = self
            .tx
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| CoreError::internal("store is closed"))?;
        sender
            .send(task)
            .map_err(|_| CoreError::internal("store thread is gone"))
    }

    /// Runs `f` on the writer thread and blocks the calling thread for the
    /// result. For a plain thread: a shell's calling thread, the event bus,
    /// a test. This panics if called from inside a tokio runtime context;
    /// use `write_async` there instead.
    pub fn write<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, CoreError> + Send + 'static,
    {
        let (rtx, rrx) = oneshot::channel::<Result<T, CoreError>>();
        self.send_task(Box::new(move |conn| {
            let _ = rtx.send(f(conn));
        }))?;
        rrx.blocking_recv()
            .map_err(|_| CoreError::internal("store thread dropped the reply"))?
    }

    /// The `write` a job body on the tokio runtime calls instead: same
    /// task, awaited rather than blocked on.
    pub async fn write_async<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, CoreError> + Send + 'static,
    {
        let (rtx, rrx) = oneshot::channel::<Result<T, CoreError>>();
        self.send_task(Box::new(move |conn| {
            let _ = rtx.send(f(conn));
        }))?;
        rrx.await
            .map_err(|_| CoreError::internal("store thread dropped the reply"))?
    }

    pub fn tx<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&Transaction) -> Result<T, CoreError> + Send + 'static,
    {
        self.write(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let out = f(&tx)?;
            tx.commit()?;
            Ok(out)
        })
    }

    /// The `tx` a job body on the tokio runtime calls instead.
    pub async fn tx_async<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&Transaction) -> Result<T, CoreError> + Send + 'static,
    {
        self.write_async(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let out = f(&tx)?;
            tx.commit()?;
            Ok(out)
        })
        .await
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
        let conn = self.reader_conn.lock().unwrap_or_else(|e| e.into_inner());
        f(&conn)
    }

    pub fn reader(&self) -> Result<Connection, CoreError> {
        let conn = Connection::open_with_flags(
            &self.path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(Duration::from_secs(5))?;
        Ok(conn)
    }

    pub fn checkpoint(&self) {
        let _ = self.write(|c| {
            c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
            Ok(())
        });
    }

    /// Drops the sender so the thread drains and exits, then joins it.
    pub fn close(&self) {
        self.tx.lock().unwrap().take();
        if let Some(t) = self.thread.lock().unwrap().take() {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp() -> (tempfile::TempDir, Arc<Store>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("data").join("anibeam.db")).unwrap();
        (dir, store)
    }

    #[test]
    fn open_creates_the_directory_and_sets_wal() {
        let (dir, store) = open_temp();
        assert!(dir.path().join("data").join("anibeam.db").exists());
        let mode: String = store
            .write(|c| Ok(c.query_row("PRAGMA journal_mode", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(mode, "wal");
        let fk: i64 = store
            .write(|c| Ok(c.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn migrations_validate_and_reach_the_latest_version() {
        migrations::MIGRATIONS.validate().unwrap();
        let (_dir, store) = open_temp();
        let v: i64 = store
            .write(|c| Ok(c.query_row("PRAGMA user_version", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(v as usize, migrations::SCHEMA_VERSION);
        let tables: i64 = store
            .write(|c| Ok(c.query_row("SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name IN ('sources','series','files','anilist_media','anilist_episodes','recommendations','relations','tracker_accounts','tracker_entries','views','completed','resume_points','skip_windows','settings','images','events')", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(tables, 16);
    }

    #[test]
    fn a_newer_database_is_refused_naming_both_versions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("anibeam.db");
        {
            let c = rusqlite::Connection::open(&path).unwrap();
            c.pragma_update(None, "user_version", 99).unwrap();
        }
        let err = Store::open(&path).err().unwrap();
        match err {
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

    #[test]
    fn tx_rolls_back_on_error_and_commits_on_ok() {
        let (_dir, store) = open_temp();
        let err = store.tx(|t| {
            t.execute("INSERT INTO sources (path, added_at) VALUES ('/a', 1)", [])?;
            Err::<(), _>(CoreError::internal("boom"))
        });
        assert!(err.is_err());
        let n: i64 = store
            .write(|c| Ok(c.query_row("SELECT count(*) FROM sources", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(n, 0);
        store
            .tx(|t| {
                t.execute("INSERT INTO sources (path, added_at) VALUES ('/a', 1)", [])?;
                Ok(())
            })
            .unwrap();
        let n: i64 = store
            .write(|c| Ok(c.query_row("SELECT count(*) FROM sources", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn readers_see_committed_writes() {
        let (_dir, store) = open_temp();
        store
            .tx(|t| {
                t.execute("INSERT INTO sources (path, added_at) VALUES ('/b', 1)", [])?;
                Ok(())
            })
            .unwrap();
        let reader = store.reader().unwrap();
        let n: i64 = reader
            .query_row("SELECT count(*) FROM sources", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
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
    }

    #[test]
    fn post_is_seen_by_a_write_queued_after_it() {
        let (_dir, store) = open_temp();
        store.post(|c| {
            if let Err(e) = c.execute(
                "INSERT INTO sources (path, added_at) VALUES ('/posted', 1)",
                [],
            ) {
                tracing::warn!("posted write failed: {e}");
            }
        });
        // The writer thread runs tasks in the order they were queued, so
        // this write only sees the posted row if post really ran first.
        let n: i64 = store
            .write(|c| Ok(c.query_row("SELECT count(*) FROM sources", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn read_sees_a_committed_write() {
        let (_dir, store) = open_temp();
        store
            .tx(|t| {
                t.execute("INSERT INTO sources (path, added_at) VALUES ('/c', 1)", [])?;
                Ok(())
            })
            .unwrap();
        let n: i64 = store
            .read(|c| Ok(c.query_row("SELECT count(*) FROM sources", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(n, 1);
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
            .read(|c| Ok(c.query_row("SELECT count(*) FROM sources", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn write_async_runs_from_a_tokio_runtime() {
        let (_dir, store) = open_temp();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(store.write_async(|c| {
            c.execute("INSERT INTO sources (path, added_at) VALUES ('/d', 1)", [])?;
            Ok(())
        }))
        .unwrap();
        let n: i64 = rt
            .block_on(store.write_async(|c| {
                Ok(c.query_row("SELECT count(*) FROM sources", [], |r| r.get(0))?)
            }))
            .unwrap();
        assert_eq!(n, 1);
    }
}
