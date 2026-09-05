//! Sources, the Scan job, and the reconciliation that writes a walk into
//! the tables. A series the walk no longer produces goes missing rather
//! than away: its row, its match and its history stay until the user
//! forgets it.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::{Connection, ErrorCode, OptionalExtension, Transaction, params, params_from_iter};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::Finished;
use crate::library::cards;
use crate::library::walk::{self, ScannedSeries};
use crate::time;

/// How long a folder has to sit still before the core believes the copying
/// into it has finished and a match is worth attempting.
pub const SETTLE: Duration = Duration::from_secs(4);

/// The library's in-memory state, the part that has no column.
#[derive(Default)]
pub struct LibraryState {
    pub movie_folders: Mutex<HashMap<u64, Vec<String>>>,
    /// One armed settle timer per series, re-armed on every reconcile that
    /// touched it, so the timer only fires once the folder stops changing.
    pub settle: Mutex<HashMap<u64, tokio::task::JoinHandle<()>>>,
    /// Paths the watcher reported that no scan has looked at yet. Scan runs
    /// one at a time, so a trigger arriving while a scan is walking waits
    /// here and the running job comes back for it.
    pub pending_paths: Mutex<Vec<String>>,
    /// Series whose auto-match is running right now, filled by Task 16. A
    /// settle timer is never armed for one of these: the match it would ask
    /// for is already under way.
    pub match_in_flight: Mutex<HashSet<u64>>,
}

/// A panicking job must never wedge one of these for the rest of the
/// process; everything inside them is plain data, so a poisoned lock is
/// recovered rather than propagated.
fn recover<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|e| e.into_inner())
}

impl LibraryState {
    fn folders(&self) -> MutexGuard<'_, HashMap<u64, Vec<String>>> {
        recover(&self.movie_folders)
    }

    pub fn movie_folders_for(&self, source: u64) -> Vec<String> {
        self.folders().get(&source).cloned().unwrap_or_default()
    }

    fn set_movie_folders(&self, source: u64, folders: Vec<String>) {
        self.folders().insert(source, folders);
    }

    fn forget_source(&self, source: u64) {
        self.folders().remove(&source);
    }

    pub(crate) fn push_pending(&self, paths: Vec<String>) {
        recover(&self.pending_paths).extend(paths);
    }

    fn take_pending(&self) -> Vec<String> {
        std::mem::take(&mut *recover(&self.pending_paths))
    }

    fn pending_is_empty(&self) -> bool {
        recover(&self.pending_paths).is_empty()
    }

    fn is_matching(&self, series: u64) -> bool {
        recover(&self.match_in_flight).contains(&series)
    }
}

#[derive(Clone, Debug)]
pub enum ScanScope {
    All,
    Source(u64),
    Series(u64),
    Paths(Vec<String>),
}

#[derive(Default, Debug)]
pub struct Reconciled {
    pub added: Vec<u64>,
    pub changed: Vec<u64>,
    pub removed: Vec<u64>,
}

fn under(path: &str, root: &str) -> bool {
    path == root || path.starts_with(&format!("{}/", root.trim_end_matches('/')))
}

/// Writes one source's walk into the tables. `only_under` limits the work to
/// series whose path equals or lies under one of the given paths, and then
/// nothing is marked missing. The containment is tested both ways: a file
/// path deep inside a series folder still reaches the series row, and a
/// series-level path still reaches a loose film by its file path.
pub fn reconcile_source(
    tx: &Transaction,
    source_id: u64,
    scanned: &[ScannedSeries],
    only_under: Option<&[String]>,
    now: i64,
) -> Result<Reconciled, CoreError> {
    let mut out = Reconciled::default();
    let in_scope =
        |p: &str| only_under.is_none_or(|roots| roots.iter().any(|r| under(p, r) || under(r, p)));

    struct Existing {
        id: u64,
        missing: bool,
    }
    let mut existing: HashMap<(String, String), Existing> = HashMap::new();
    {
        let mut stmt =
            tx.prepare("SELECT id, kind, path, missing_since FROM series WHERE source_id = ?1")?;
        let rows = stmt.query_map(params![source_id as i64], |r| {
            Ok((
                (r.get::<_, String>(1)?, r.get::<_, String>(2)?),
                Existing {
                    id: r.get::<_, i64>(0)? as u64,
                    missing: r.get::<_, Option<i64>>(3)?.is_some(),
                },
            ))
        })?;
        for row in rows {
            let (key, value) = row?;
            existing.insert(key, value);
        }
    }

    let mut seen: HashSet<u64> = HashSet::new();
    // Series that lost a file to another one during this reconcile. A file
    // changes series without moving on disk whenever the walk reclassifies
    // the folder above it, and both rows exist at once inside this
    // transaction, so the row is handed over rather than inserted twice.
    let mut lost_a_file: HashSet<u64> = HashSet::new();
    for s in scanned.iter().filter(|s| in_scope(&s.path)) {
        let key = (s.kind.as_str().to_string(), s.path.clone());
        let (id, was_missing, is_new) = match existing.get(&key) {
            Some(e) => {
                tx.execute(
                    "UPDATE series SET folder_name = ?2, missing_since = NULL WHERE id = ?1",
                    params![e.id as i64, s.name],
                )?;
                (e.id, e.missing, false)
            }
            None => {
                tx.execute(
                    "INSERT INTO series (source_id, kind, path, folder_name, added_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![source_id as i64, s.kind.as_str(), s.path, s.name, now],
                )?;
                (tx.last_insert_rowid() as u64, false, true)
            }
        };
        seen.insert(id);

        let mut old: HashMap<String, (u64, i64)> = HashMap::new();
        {
            let mut stmt =
                tx.prepare_cached("SELECT id, path, mtime FROM files WHERE series_id = ?1")?;
            let rows = stmt.query_map(params![id as i64], |r| {
                Ok((
                    r.get::<_, String>(1)?,
                    (r.get::<_, i64>(0)? as u64, r.get::<_, i64>(2)?),
                ))
            })?;
            for row in rows {
                let (path, value) = row?;
                old.insert(path, value);
            }
        }

        let mut file_changed = false;
        let new_paths: HashSet<&str> = s.files.iter().map(|f| f.path.as_str()).collect();
        for (path, (file_id, _)) in &old {
            if !new_paths.contains(path.as_str()) {
                tx.execute("DELETE FROM files WHERE id = ?1", params![*file_id as i64])?;
                file_changed = true;
            }
        }
        for f in &s.files {
            let kind = if f.classified.extra.is_some() {
                "extra"
            } else {
                "episode"
            };
            let sidecars = serde_json::to_string(&f.sidecars)?;
            match old.get(&f.path) {
                Some((file_id, mtime)) => {
                    if *mtime != f.mtime {
                        file_changed = true;
                    }
                    tx.execute(
                        "UPDATE files SET size = ?2, mtime = ?3, kind = ?4, number = ?5, season = ?6, extra_kind = ?7, extra_index = ?8, label = ?9, episode_key = ?10, sidecars = ?11, seen_at = ?12 WHERE id = ?1",
                        params![
                            *file_id as i64,
                            f.size as i64,
                            f.mtime,
                            kind,
                            episode_number(f),
                            f.classified.season,
                            f.classified.extra.map(|k| k.as_str()),
                            f.classified.extra_index,
                            f.label,
                            f.episode_key,
                            sidecars,
                            now
                        ],
                    )?;
                }
                None => {
                    file_changed = true;
                    // A path exists on disk once, so a row for it under
                    // another series is stale by definition, whichever
                    // order the two series come in. Take it.
                    let owner: Option<i64> = {
                        let mut stmt =
                            tx.prepare_cached("SELECT series_id FROM files WHERE path = ?1")?;
                        stmt.query_row(params![f.path], |r| r.get(0)).optional()?
                    };
                    if let Some(other) = owner.map(|v| v as u64).filter(|other| *other != id) {
                        tx.execute("DELETE FROM files WHERE path = ?1", params![f.path])?;
                        lost_a_file.insert(other);
                    }
                    tx.execute(
                        "INSERT INTO files (series_id, path, size, mtime, kind, number, season, extra_kind, extra_index, label, episode_key, sidecars, seen_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                        params![
                            id as i64,
                            f.path,
                            f.size as i64,
                            f.mtime,
                            kind,
                            episode_number(f),
                            f.classified.season,
                            f.classified.extra.map(|k| k.as_str()),
                            f.classified.extra_index,
                            f.label,
                            f.episode_key,
                            sidecars,
                            now
                        ],
                    )?;
                }
            }
        }

        if is_new {
            out.added.push(id);
        } else if was_missing || file_changed {
            out.changed.push(id);
        }
    }

    // A scoped scan walked the whole source for context but speaks for its
    // own paths alone. It speaks for them fully, though: a series inside
    // the scope that the walk did not produce is gone, and it goes missing
    // exactly as a full scan would mark it. Series outside the scope are
    // the ones it says nothing about. Sorted so the terminal event's order
    // does not depend on a hash map's.
    let mut vanished: Vec<u64> = existing
        .iter()
        .filter(|((_, path), e)| !e.missing && !seen.contains(&e.id) && in_scope(path))
        .map(|(_, e)| e.id)
        .collect();
    vanished.sort_unstable();
    for id in vanished {
        tx.execute(
            "UPDATE series SET missing_since = ?2 WHERE id = ?1",
            params![id as i64, now],
        )?;
        tx.execute("DELETE FROM files WHERE series_id = ?1", params![id as i64])?;
        out.removed.push(id);
    }
    // Only a full walk can claim the source was scanned in full.
    if only_under.is_none() {
        tx.execute(
            "UPDATE sources SET scanned_at = ?2 WHERE id = ?1",
            params![source_id as i64, now],
        )?;
    }

    // A series that lost a file to another one is changed too, even when
    // its own turn saw nothing move, or it never had a turn because the
    // series that took the file belongs to another source. One that went
    // missing in the same pass is already reported as that, and counting it
    // twice would only inflate the terminal event.
    let mut lost: Vec<u64> = lost_a_file.into_iter().collect();
    lost.sort_unstable();
    for other in lost {
        if !out.added.contains(&other)
            && !out.changed.contains(&other)
            && !out.removed.contains(&other)
        {
            out.changed.push(other);
        }
    }
    Ok(out)
}

/// Pushes a series' settle timer out by another four seconds, replacing the
/// one already armed. When it finally fires, the folder has stopped
/// changing and `Core::settle_fired` decides what to do about it; Task 16
/// makes that an auto-match.
///
/// The task holds a `Weak`, never an `Arc`: a timer must not be the reason
/// the core is still alive.
pub fn arm_settle(core: &Arc<Core>, series_id: u64) {
    let mut settle = recover(&core.library.settle);
    if let Some(previous) = settle.remove(&series_id) {
        previous.abort();
    }
    let weak = Arc::downgrade(core);
    let handle = core.handle.spawn(async move {
        tokio::time::sleep(SETTLE).await;
        let Some(core) = weak.upgrade() else { return };
        {
            let mut armed = recover(&core.library.settle);
            // `abort` cannot reach a timer that is already past its sleep,
            // so a re-arm landing in that window would otherwise fire the
            // old timer as well. The map says which one is the live timer.
            match armed.get(&series_id) {
                Some(current) if current.id() == tokio::task::id() => armed.remove(&series_id),
                _ => return,
            };
        }
        core.settle_fired(series_id);
    });
    settle.insert(series_id, handle);
}

/// `?,?,?` for an `IN` list. Ids are bound, never formatted into the SQL.
fn placeholders(n: usize) -> String {
    std::iter::repeat_n("?", n).collect::<Vec<_>>().join(",")
}

/// Which of these series have never been matched and never been attempted:
/// the ones a settled folder is worth an auto-match for. A MAL-only
/// confirmed match has a `provider` and is left alone.
fn never_attempted(conn: &Connection, ids: &[u64]) -> Result<Vec<u64>, CoreError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT id FROM series WHERE attempted_at IS NULL AND provider IS NULL AND id IN ({})",
        placeholders(ids.len())
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(ids.iter().map(|id| *id as i64)), |r| {
        r.get::<_, i64>(0).map(|v| v as u64)
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// An extra's number column is NULL; an episode's is its number.
fn episode_number(f: &walk::ScannedFile) -> Option<f64> {
    if f.classified.extra.is_some() {
        None
    } else {
        Some(f.classified.number)
    }
}

/// One source the job is about to walk.
struct Target {
    id: u64,
    path: String,
    available: bool,
}

/// What the scope resolved to: the sources to walk, the paths the work is
/// limited to, and the source id the terminal event names.
struct Plan {
    targets: Vec<Target>,
    only_under: Option<Vec<String>>,
    reply_source: Option<u64>,
}

fn all_targets(conn: &Connection) -> Result<Vec<Target>, CoreError> {
    let mut stmt = conn.prepare("SELECT id, path, available FROM sources ORDER BY id")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Target {
                id: r.get::<_, i64>(0)? as u64,
                path: r.get(1)?,
                available: r.get::<_, i64>(2)? == 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn series_count(conn: &Connection, source: u64) -> Result<u64, CoreError> {
    let n: i64 = conn.query_row(
        "SELECT count(*) FROM series WHERE source_id = ?1 AND missing_since IS NULL",
        params![source as i64],
        |r| r.get(0),
    )?;
    Ok(n as u64)
}

/// The stored half of a `Source`: everything but the movie folders, which
/// live in memory.
fn source_row(conn: &Connection, id: u64) -> Result<Option<(String, bool)>, CoreError> {
    let row = conn
        .query_row(
            "SELECT path, available FROM sources WHERE id = ?1",
            params![id as i64],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? == 1)),
        )
        .optional()?;
    Ok(row)
}

pub fn load_source(
    conn: &Connection,
    state: &LibraryState,
    id: u64,
) -> Result<Option<Source>, CoreError> {
    let Some((path, available)) = source_row(conn, id)? else {
        return Ok(None);
    };
    Ok(Some(Source {
        id,
        path,
        available,
        series_count: series_count(conn, id)?,
        movie_folders: state.movie_folders_for(id),
    }))
}

pub fn list_sources(conn: &Connection, state: &LibraryState) -> Result<Vec<Source>, CoreError> {
    let ids: Vec<u64> = {
        let mut stmt = conn.prepare("SELECT id FROM sources ORDER BY id")?;
        stmt.query_map([], |r| r.get::<_, i64>(0).map(|v| v as u64))?
            .collect::<Result<Vec<_>, _>>()?
    };
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(source) = load_source(conn, state, id)? {
            out.push(source);
        }
    }
    Ok(out)
}

/// A job needs an `Arc<Core>` of its own; a core already shutting down has
/// none to give.
fn owner(core: &Core) -> Result<Arc<Core>, CoreError> {
    core.arc()
        .ok_or_else(|| CoreError::internal("core is shutting down"))
}

fn is_unique_violation(e: &rusqlite::Error) -> bool {
    matches!(e, rusqlite::Error::SqliteFailure(f, _) if f.code == ErrorCode::ConstraintViolation)
}

// The calls -------------------------------------------------------------

pub fn list_sources_call(core: &Core) -> Result<Reply, CoreError> {
    Ok(Reply::Sources {
        sources: core.store.read(|c| list_sources(c, &core.library))?,
    })
}

/// A trailing separator is not part of a path's identity, so it never
/// reaches the column: `/lib/` and `/lib` are one source, not two. A bare
/// root stays itself.
fn normalise_source_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Inserts the row, says so, and starts a scan scoped to it. A path that
/// is not a directory right now is still a source, just an unavailable
/// one; the job leaves everything under it alone.
pub fn add_source(core: &Core, path: &str) -> Result<Reply, CoreError> {
    if !Path::new(path).is_absolute() {
        return Err(CoreError::invalid("path", "a source path must be absolute"));
    }
    let owner = owner(core)?;
    let path = normalise_source_path(path);
    let available = Path::new(&path).is_dir();
    let now = time::now_secs();
    let insert = path.clone();
    // The overlap check and the insert share one transaction, so two calls
    // racing each other cannot both find the tree free.
    let id = core.store.tx(move |tx| {
        // Two sources that overlap walk the same tree and both claim every
        // file in it: the second one's series hit `UNIQUE (kind, path)` and
        // fail the reconcile, and every full scan after that fails the same
        // way. Refuse the overlap at the door instead.
        let existing: Vec<String> = {
            let mut stmt = tx.prepare("SELECT path FROM sources")?;
            stmt.query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?
        };
        for other in &existing {
            if *other == insert {
                return Err(CoreError::invalid("path", "already a source"));
            }
            if under(&insert, other) || under(other, &insert) {
                return Err(CoreError::invalid(
                    "path",
                    "nested inside an existing source",
                ));
            }
        }
        match tx.execute(
            "INSERT INTO sources (path, available, added_at) VALUES (?1, ?2, ?3)",
            params![insert, i64::from(available), now],
        ) {
            Ok(_) => Ok(tx.last_insert_rowid() as u64),
            // The check above is the real guard; the column's own
            // constraint stays the backstop.
            Err(e) if is_unique_violation(&e) => {
                Err(CoreError::invalid("path", "already a source"))
            }
            Err(e) => Err(e.into()),
        }
    })?;
    let source = Source {
        id,
        path: path.clone(),
        available,
        series_count: 0,
        movie_folders: Vec::new(),
    };
    core.bus.info(
        Stage::Library,
        format!("source added: {path}"),
        EventBody::SourceChanged {
            source: source.clone(),
        },
    );
    // A scan already running knows nothing about this source and the
    // one-at-a-time rule means the call below starts nothing. Queued here,
    // the running job comes back for it before it finishes; when no scan is
    // running, the scan the next line starts covers the path and takes it
    // straight back off the queue.
    core.library.push_pending(vec![path.clone()]);
    start(&owner, ScanScope::Source(id));
    Ok(Reply::Source { source })
}

/// Deletes the row; every series under it goes with it through the
/// cascade, and both removals are announced.
pub fn remove_source(core: &Core, source: u64) -> Result<Reply, CoreError> {
    let (path, ids) = core.store.tx(move |tx| {
        let path: Option<String> = tx
            .query_row(
                "SELECT path FROM sources WHERE id = ?1",
                params![source as i64],
                |r| r.get(0),
            )
            .optional()?;
        let path = path.ok_or(CoreError::NotFound {
            what: Entity::Source,
            id: source,
        })?;
        let ids: Vec<u64> = {
            let mut stmt = tx.prepare("SELECT id FROM series WHERE source_id = ?1 ORDER BY id")?;
            stmt.query_map(params![source as i64], |r| {
                r.get::<_, i64>(0).map(|v| v as u64)
            })?
            .collect::<Result<Vec<_>, _>>()?
        };
        tx.execute("DELETE FROM sources WHERE id = ?1", params![source as i64])?;
        Ok((path, ids))
    })?;
    core.library.forget_source(source);
    core.unwatch_source(&path);
    if !ids.is_empty() {
        core.bus.info(
            Stage::Library,
            format!("source removed with {} series", ids.len()),
            EventBody::SeriesRemoved { ids },
        );
    }
    core.bus.info(
        Stage::Library,
        format!("source removed: {path}"),
        EventBody::SourceRemoved { source },
    );
    Ok(Reply::Ok)
}

/// Drops a missing series for good, history and match included. A series
/// still on disk is refused: the scan would only bring it back.
pub fn forget_series(core: &Core, series: u64) -> Result<Reply, CoreError> {
    let name = core.store.tx(move |tx| {
        let row: Option<(String, Option<i64>)> = tx
            .query_row(
                "SELECT folder_name, missing_since FROM series WHERE id = ?1",
                params![series as i64],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let (name, missing_since) = row.ok_or(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        })?;
        if missing_since.is_none() {
            return Err(CoreError::Refused {
                reason: Refusal::OnDisk,
            });
        }
        tx.execute("DELETE FROM series WHERE id = ?1", params![series as i64])?;
        Ok(name)
    })?;
    core.bus.info(
        Stage::Library,
        format!("forgot {name}"),
        EventBody::SeriesRemoved { ids: vec![series] },
    );
    Ok(Reply::Ok)
}

pub fn scan(core: &Core, source: Option<u64>) -> Result<Reply, CoreError> {
    let scope = match source {
        Some(id) => ScanScope::Source(id),
        None => ScanScope::All,
    };
    Ok(Reply::Started {
        job: start(&owner(core)?, scope),
    })
}

/// Rescans one series in its source's context. A source that is not there
/// right now is refused rather than walked, so the series is never marked
/// missing by a scan that could not look.
pub fn rescan_series(core: &Core, series: u64) -> Result<Reply, CoreError> {
    let row: Option<(String, bool)> = core.store.read(|c| {
        Ok(c.query_row(
            "SELECT src.path, src.available FROM series s JOIN sources src ON src.id = s.source_id WHERE s.id = ?1",
            params![series as i64],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? == 1)),
        )
        .optional()?)
    })?;
    let (path, available) = row.ok_or(CoreError::NotFound {
        what: Entity::Series,
        id: series,
    })?;
    if !available {
        return Err(CoreError::Unavailable { path });
    }
    Ok(Reply::Started {
        job: start(&owner(core)?, ScanScope::Series(series)),
    })
}

// The job ---------------------------------------------------------------

/// Starts the Scan job and returns its id. Scan runs one at a time, so a
/// second call while one is running returns the running job's id.
pub fn start(core: &Arc<Core>, scope: ScanScope) -> u64 {
    let core = core.clone();
    core.jobs
        .clone()
        .start(JobKind::Scan, move |ctx| async move {
            let (mut added, mut changed, mut removed) = (0u64, 0u64, 0u64);
            let mut reply_source = None;
            let mut scope = scope;
            let mut first_pass = true;

            loop {
                // A `Paths` scan is the drain: it takes the whole queue, both
                // the paths it was handed and whatever the watcher reported
                // while an earlier scan was walking. An empty list means "just
                // the queue", which is how the watcher asks for one.
                if let ScanScope::Paths(handed) = &scope {
                    let mut all = core.library.take_pending();
                    all.extend(handed.iter().cloned());
                    all.sort_unstable();
                    all.dedup();
                    scope = ScanScope::Paths(all);
                }
                let plan = resolve(&core, &scope).await?;
                // Any other scope takes off the queue only what its own walk
                // covers anyway, so a scan of a source does not leave a
                // redundant follow-up pass behind it. What it does not cover
                // stays queued for the drain.
                if !matches!(scope, ScanScope::Paths(_)) {
                    recover(&core.library.pending_paths).retain(|p| !covered_by(&plan, p));
                }
                if first_pass {
                    reply_source = plan.reply_source;
                    first_pass = false;
                }
                let Plan {
                    targets,
                    only_under,
                    ..
                } = plan;
                let total = targets.len() as u64;

                for (i, target) in targets.into_iter().enumerate() {
                    ctx.checkpoint()?;
                    ctx.progress(i as u64, Some(total), &target.path);

                    let source_id = target.id;
                    let path = target.path;
                    let available = Path::new(&path).is_dir();
                    if available != target.available {
                        core.store
                            .write_async(move |c| {
                                c.execute(
                                    "UPDATE sources SET available = ?2 WHERE id = ?1",
                                    params![source_id as i64, i64::from(available)],
                                )?;
                                Ok(())
                            })
                            .await?;
                        if let Some((row_path, row_available)) = core
                            .store
                            .write_async(move |c| source_row(c, source_id))
                            .await?
                        {
                            let count = core
                                .store
                                .write_async(move |c| series_count(c, source_id))
                                .await?;
                            let source = Source {
                                id: source_id,
                                path: row_path,
                                available: row_available,
                                series_count: count,
                                movie_folders: core.library.movie_folders_for(source_id),
                            };
                            let message = if available {
                                format!("source available again: {path}")
                            } else {
                                format!("source unavailable: {path}")
                            };
                            ctx.emit(Level::Info, message, EventBody::SourceChanged { source });
                        }
                    }
                    if !available {
                        continue;
                    }

                    // The recursive watch is installed here rather than by
                    // `AddSource`, because notify installs one by walking every
                    // directory under the root: that is disk work at library
                    // scale and belongs to a job. Idempotent per root, so every
                    // later scan asking again costs a string compare.
                    let owner = core.clone();
                    let root = path.clone();
                    let watched = tokio::task::spawn_blocking(move || owner.install_watch(&root))
                        .await
                        .map_err(|e| CoreError::internal(e.to_string()))?;
                    if let Err(e) = watched {
                        ctx.emit(
                            Level::Warn,
                            format!("cannot watch {path}: {e}"),
                            EventBody::Notice,
                        );
                    }

                    let root = path.clone();
                    let scanned =
                        tokio::task::spawn_blocking(move || walk::scan_source(Path::new(&root)))
                            .await
                            .map_err(|e| CoreError::internal(e.to_string()))??;
                    let root = path.clone();
                    let movie_folders = tokio::task::spawn_blocking(move || {
                        walk::find_movie_folders(Path::new(&root))
                    })
                    .await
                    .map_err(|e| CoreError::internal(e.to_string()))?;
                    core.library.set_movie_folders(source_id, movie_folders);

                    let only = only_under.clone();
                    let now = time::now_secs();
                    let r = core
                        .store
                        .tx_async(move |tx| {
                            reconcile_source(tx, source_id, &scanned, only.as_deref(), now)
                        })
                        .await?;
                    added += r.added.len() as u64;
                    changed += r.changed.len() as u64;
                    removed += r.removed.len() as u64;

                    // The batch carries what went missing too: those cards say
                    // `missing: true`, which is how a shell patching its grid
                    // from events knows to drop them. `SeriesRemoved` stays
                    // what it was, the answer to Forget and RemoveSource.
                    let touched: Vec<u64> = r
                        .added
                        .iter()
                        .chain(r.changed.iter())
                        .chain(r.removed.iter())
                        .copied()
                        .collect();
                    if !touched.is_empty() {
                        let dir = core.paths.images_dir();
                        let cards = core
                            .store
                            .write_async(move |c| cards::cards_for(c, &dir, &touched))
                            .await?;
                        ctx.changed_all(cards);
                    }

                    // A folder that changed is a folder still being copied
                    // into, so every series this pass added or changed has its
                    // settle timer pushed out again. Only one that has never
                    // been matched and is not being matched right now is worth
                    // a timer at all, and a folder that just vanished is worth
                    // none.
                    let ids: Vec<u64> = r.added.iter().chain(r.changed.iter()).copied().collect();
                    if !ids.is_empty() {
                        let unmatched = core
                            .store
                            .write_async(move |c| never_attempted(c, &ids))
                            .await?;
                        for id in unmatched
                            .into_iter()
                            .filter(|id| !core.library.is_matching(*id))
                        {
                            arm_settle(&core, id);
                        }
                    }
                }

                // A trigger that arrived while this pass was walking is not in
                // it. Going round again inside this job is the only way to
                // reach it: Scan runs one at a time, so asking for a new job
                // from here would only be handed this one's own id back.
                if core.library.pending_is_empty() {
                    break;
                }
                scope = ScanScope::Paths(Vec::new());
            }

            Ok(Finished {
                level: Level::Info,
                message: format!(
                    "scan finished: {added} added, {changed} changed, {removed} missing"
                ),
                body: EventBody::ScanFinished {
                    source: reply_source,
                    added,
                    changed,
                    removed,
                },
            })
        })
}

/// Whether this run is going to look at `path` anyway: some target contains
/// it, and either the run is a full walk of that source or the path is in
/// its scope both ways round, the same test `reconcile_source` applies.
fn covered_by(plan: &Plan, path: &str) -> bool {
    plan.targets.iter().any(|t| under(path, &t.path))
        && plan
            .only_under
            .as_ref()
            .is_none_or(|roots| roots.iter().any(|r| under(path, r) || under(r, path)))
}

/// Turns a scope into the sources to walk and the paths to limit the work
/// to. A scoped scan still walks its source in full, since a folder's
/// classification only makes sense in the source's context.
async fn resolve(core: &Arc<Core>, scope: &ScanScope) -> Result<Plan, CoreError> {
    let sources = core.store.write_async(|c| all_targets(c)).await?;
    let plan = match scope {
        ScanScope::All => Plan {
            targets: sources,
            only_under: None,
            reply_source: None,
        },
        ScanScope::Source(id) => {
            let id = *id;
            Plan {
                targets: sources.into_iter().filter(|s| s.id == id).collect(),
                only_under: None,
                reply_source: Some(id),
            }
        }
        ScanScope::Series(id) => {
            let id = *id;
            let row: Option<(u64, String)> = core
                .store
                .write_async(move |c| {
                    Ok(c.query_row(
                        "SELECT source_id, path FROM series WHERE id = ?1",
                        params![id as i64],
                        |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, String>(1)?)),
                    )
                    .optional()?)
                })
                .await?;
            match row {
                Some((source_id, path)) => Plan {
                    targets: sources.into_iter().filter(|s| s.id == source_id).collect(),
                    only_under: Some(vec![path]),
                    reply_source: None,
                },
                // The series went away between the call and the job.
                None => Plan {
                    targets: Vec::new(),
                    only_under: None,
                    reply_source: None,
                },
            }
        }
        ScanScope::Paths(paths) => {
            let targets: Vec<Target> = sources
                .into_iter()
                .filter(|s| paths.iter().any(|p| under(p, &s.path)))
                .collect();
            Plan {
                targets,
                only_under: Some(paths.clone()),
                reply_source: None,
            }
        }
    };
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::classifier::Classified;
    use crate::store::Store;

    fn scanned(kind: SeriesKind, path: &str, name: &str, files: &[&str]) -> ScannedSeries {
        let files = files
            .iter()
            .enumerate()
            .map(|(i, p)| walk::ScannedFile {
                path: (*p).to_string(),
                size: 1,
                mtime: 1,
                classified: Classified {
                    extra: None,
                    number: (i + 1) as f64,
                    season: None,
                    extra_index: None,
                    extra_variant: None,
                    raw_label: None,
                },
                label: format!("Episode {}", i + 1),
                episode_key: format!("{}", i + 1),
                sidecars: Vec::new(),
            })
            .collect();
        ScannedSeries {
            kind,
            path: path.to_string(),
            name: name.to_string(),
            season_hint: None,
            part_hint: None,
            files,
        }
    }

    /// A file belongs to one series. A second source nested inside the
    /// first walks the same file and claims it, so the row moves and the
    /// series that lost it is reported changed, even though that series
    /// belongs to a source this reconcile never looked at.
    #[test]
    fn a_path_claimed_by_another_source_moves_and_counts_the_loser_changed() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("anibeam.db")).unwrap();
        let file = "/lib/A/ep01.mkv";

        let (show_id, inner_source) = store
            .tx(move |tx| {
                tx.execute(
                    "INSERT INTO sources (path, available, added_at) VALUES ('/lib', 1, 0)",
                    [],
                )?;
                let outer_source = tx.last_insert_rowid() as u64;
                tx.execute(
                    "INSERT INTO sources (path, available, added_at) VALUES ('/lib/A', 1, 0)",
                    [],
                )?;
                let inner_source = tx.last_insert_rowid() as u64;
                let r = reconcile_source(
                    tx,
                    outer_source,
                    &[scanned(SeriesKind::Show, "/lib/A", "A", &[file])],
                    None,
                    10,
                )?;
                Ok((r.added[0], inner_source))
            })
            .unwrap();

        let r = store
            .tx(move |tx| {
                reconcile_source(
                    tx,
                    inner_source,
                    &[scanned(SeriesKind::Movie, file, "ep01", &[file])],
                    None,
                    20,
                )
            })
            .unwrap();
        assert_eq!(r.added.len(), 1);
        assert_eq!(r.changed, vec![show_id]);
        assert!(r.removed.is_empty());

        // Exactly one row for that path, and it is the new series'.
        let (rows, owner): (i64, i64) = store
            .write(move |c| {
                Ok(c.query_row(
                    "SELECT count(*), max(series_id) FROM files WHERE path = ?1",
                    params![file],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?)
            })
            .unwrap();
        assert_eq!(rows, 1);
        assert_eq!(owner as u64, r.added[0]);
        store.close();
    }

    #[test]
    fn under_is_a_path_prefix_not_a_string_prefix() {
        assert!(under("/lib/Show A", "/lib"));
        assert!(under("/lib", "/lib"));
        assert!(under("/lib/Show A/ep.mkv", "/lib/Show A"));
        assert!(!under("/library/Show A", "/lib"));
        assert!(under("/lib/Show A", "/lib/"));
    }
}
