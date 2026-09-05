//! The image cache: one file per url under `<cache_dir>/images`, one row per
//! file in `images`, and the two jobs that fill and empty it.
//!
//! Nothing here goes through a provider limiter. The posters, banners and
//! portraits all come from AniList's CDN rather than its API, so the only
//! bound on a fetch is a semaphore of four permits; an error from one still
//! names AniList, since that is whose CDN answered.
//!
//! A read never fetches. It reports the gap it found and the cache decides,
//! at most once every five minutes, whether that is worth a fill job.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params, params_from_iter};
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;

use crate::contract::*;
use crate::core::Core;
use crate::jobs::Finished;
use crate::library::cards;
use crate::net::{Http, HttpRequest, Method};
use crate::store::Store;
use crate::time;

/// Fetches in flight at once.
const CONCURRENCY: usize = 4;

/// Urls per round of a fill, so a big library reports progress and notices
/// cancellation between batches rather than only at the end.
const BATCH: usize = 20;

/// How long an image no row references survives after its last use.
const ORPHAN_AGE: i64 = 30 * 86_400;

/// The row cap. Above it the least recently used non-pinned rows go.
const MAX_ROWS: u64 = 5_000;

/// `used_at` is bumped at most once a day per image, so a read that hands
/// out a hundred posters queues one narrow update rather than a hundred.
const USED_GAP: i64 = 86_400;

/// A read that finds a gap starts a fill at most this often. Without it a
/// core with no network would start one on every read it ever answers.
const FILL_INTERVAL: Duration = Duration::from_secs(300);

/// The extensions a url is allowed to name. Anything else is judged by the
/// content type instead.
const IMAGE_EXTENSIONS: [&str; 6] = ["jpg", "jpeg", "png", "gif", "webp", "avif"];

/// Every url whose image the library would draw if it had it.
const REFERENCED_SQL: &str =
    "SELECT cover_url AS url FROM anilist_media WHERE cover_url IS NOT NULL
     UNION SELECT banner_url FROM anilist_media WHERE banner_url IS NOT NULL
     UNION SELECT value ->> 'image_url' FROM anilist_media, json_each(anilist_media.characters)
           WHERE value ->> 'image_url' IS NOT NULL";

/// The urls of a series the library actually owns. These are never evicted
/// by the cap: losing them is the one loss a user would see at once.
const PINNED_SQL: &str = "SELECT m.cover_url AS url FROM anilist_media m JOIN series s ON s.anilist_id = m.id WHERE m.cover_url IS NOT NULL
     UNION SELECT m.banner_url FROM anilist_media m JOIN series s ON s.anilist_id = m.id WHERE m.banner_url IS NOT NULL";

/// What a fill goes and gets: everything an owned series draws, the covers
/// of what it recommends, and the covers behind the watching list.
const MISSING_SQL: &str = "SELECT DISTINCT url FROM (
        SELECT m.cover_url AS url FROM anilist_media m JOIN series s ON s.anilist_id = m.id
        UNION SELECT m.banner_url FROM anilist_media m JOIN series s ON s.anilist_id = m.id
        UNION SELECT j.value ->> 'image_url' FROM anilist_media m JOIN series s ON s.anilist_id = m.id
                     CROSS JOIN json_each(m.characters) j
        UNION SELECT rm.cover_url FROM recommendations r
                     JOIN series s ON s.anilist_id = r.anilist_id
                     JOIN anilist_media rm ON rm.id = r.recommended_id
        UNION SELECT tm.cover_url FROM tracker_entries t
                     JOIN anilist_media tm ON (t.tracker = 'anilist' AND tm.id = t.media_id)
                                           OR (t.tracker = 'mal' AND tm.mal_id = t.media_id)
     )
     WHERE url IS NOT NULL AND url NOT IN (SELECT url FROM images)
     ORDER BY url";

// ---------------------------------------------------------------------------
// The pure rules
// ---------------------------------------------------------------------------

/// A url's identity on disk: the sha256 of the url itself, lowercase hex.
/// The url is the key everywhere, so nothing has to parse a CDN path.
pub fn key(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    let mut out = String::with_capacity(64);
    for byte in digest.as_slice() {
        // Writing into a String is infallible; the result exists only
        // because `Write` is one trait for files and strings alike.
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// `<first two hex>/<hash>.<ext>`, relative to the images directory. The
/// shard keeps any one directory to a few hundred files on a big library.
pub fn relative_path(url: &str, ext: &str) -> String {
    let hash = key(url);
    format!("{}/{hash}.{ext}", &hash[..2])
}

/// What to call the file. The url's own extension wins, since AniList's CDN
/// names its files honestly; the content type answers when it does not; jpg
/// is the last resort, because the byte content is what a decoder reads and
/// the extension is only a hint to it.
pub fn extension(url: &str, content_type: Option<&str>) -> &'static str {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let name = path.rsplit('/').next().unwrap_or(path);
    if let Some((_, ext)) = name.rsplit_once('.') {
        let lower = ext.to_ascii_lowercase();
        if let Some(known) = IMAGE_EXTENSIONS.iter().find(|e| **e == lower) {
            return known;
        }
    }
    let mime = content_type.unwrap_or_default();
    let mime = mime
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match mime.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/avif" => "avif",
        _ => "jpg",
    }
}

/// `?,?,?` for an `IN` list, the values always bound rather than formatted.
fn placeholders(n: usize) -> String {
    let mut out = String::with_capacity(n * 2);
    for i in 0..n {
        if i > 0 {
            out.push(',');
        }
        out.push('?');
    }
    out
}

/// A fetch that failed. The CDN is AniList's, so that is the provider a
/// shell is told about; `status` is None when nothing answered at all.
fn cdn_error(status: Option<u32>, message: impl Into<String>) -> CoreError {
    CoreError::Provider {
        provider: Provider::Anilist,
        status,
        message: message.into(),
        retry_after: None,
    }
}

// ---------------------------------------------------------------------------
// The cache
// ---------------------------------------------------------------------------

/// What one sweep did. Rows removed for the first three reasons are
/// `removed_rows`; rows the cap took are `evicted`; every file deleted from
/// the directory, whatever the reason, is `removed_files`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SweepReport {
    pub removed_rows: u64,
    pub removed_files: u64,
    pub evicted: u64,
}

pub struct ImageCache {
    store: Arc<Store>,
    dir: PathBuf,
    http: Arc<dyn Http>,
    permits: Arc<Semaphore>,
    /// When a fill last actually started. A coalesced call does not touch
    /// it, so the five minute gate measures real fills.
    fill_started_at: Mutex<Option<Instant>>,
}

impl ImageCache {
    pub fn new(store: Arc<Store>, images_dir: PathBuf, http: Arc<dyn Http>) -> Arc<ImageCache> {
        Arc::new(ImageCache {
            store,
            dir: images_dir,
            http,
            permits: Arc::new(Semaphore::new(CONCURRENCY)),
            fill_started_at: Mutex::new(None),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Nothing in here is worth poisoning a lock over: the guarded value is
    /// one `Instant`, and a panicking job must not wedge the gate for the
    /// rest of the process.
    fn fill_gate(&self) -> MutexGuard<'_, Option<Instant>> {
        self.fill_started_at
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// Whether a read that found a gap should ask for a fill.
    pub fn fill_is_due(&self) -> bool {
        self.fill_gate()
            .is_none_or(|at| at.elapsed() >= FILL_INTERVAL)
    }

    pub fn mark_fill_started(&self) {
        *self.fill_gate() = Some(Instant::now());
    }

    /// The local absolute path of every url, fetching what is missing, at
    /// most four at a time. A url whose row has lost its file is refetched.
    /// A failure is that url's own: the batch always comes back whole.
    ///
    /// The fetches are spawned, so this is called from inside the runtime:
    /// a job body, or a task on it. Dropping the future, which is what
    /// cancelling a job does, aborts whatever is still in flight.
    pub async fn ensure(&self, urls: &[String]) -> Vec<(String, Result<String, CoreError>)> {
        let mut wanted: Vec<String> = urls.to_vec();
        wanted.sort_unstable();
        wanted.dedup();
        if wanted.is_empty() {
            return Vec::new();
        }

        let lookup = wanted.clone();
        let known = match self.store.write_async(move |c| rows_for(c, &lookup)).await {
            Ok(known) => known,
            Err(e) => return urls.iter().map(|u| (u.clone(), Err(e.clone()))).collect(),
        };

        let mut done: HashMap<String, Result<String, CoreError>> = HashMap::new();
        let mut stale: Vec<String> = Vec::new();
        let mut fetch: Vec<String> = Vec::new();
        for url in wanted {
            match known.get(&url) {
                Some(relative) if self.dir.join(relative).is_file() => {
                    done.insert(
                        url,
                        Ok(self.dir.join(relative).to_string_lossy().into_owned()),
                    );
                }
                Some(_) => {
                    stale.push(url.clone());
                    fetch.push(url);
                }
                None => fetch.push(url),
            }
        }
        // The row goes before the refetch, so a fetch that fails leaves the
        // library telling the truth: no image rather than a broken path.
        if !stale.is_empty()
            && let Err(e) = self
                .store
                .write_async(move |c| delete_rows(c, &stale))
                .await
        {
            tracing::warn!("clearing image rows whose files were gone failed: {e}");
        }

        let mut set = tokio::task::JoinSet::new();
        for url in fetch {
            let http = self.http.clone();
            let store = self.store.clone();
            let dir = self.dir.clone();
            let permits = self.permits.clone();
            set.spawn(async move {
                let outcome = match permits.acquire_owned().await {
                    Ok(_permit) => fetch_one(http.as_ref(), &store, &dir, &url).await,
                    Err(e) => Err(CoreError::internal(format!("image permits: {e}"))),
                };
                (url, outcome)
            });
        }
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok((url, outcome)) => {
                    done.insert(url, outcome);
                }
                Err(e) => tracing::warn!("an image fetch ended without a result: {e}"),
            }
        }

        urls.iter()
            .map(|url| {
                let outcome = done
                    .get(url)
                    .cloned()
                    .unwrap_or_else(|| Err(CoreError::internal(format!("no outcome for {url}"))));
                (url.clone(), outcome)
            })
            .collect()
    }

    /// The local path of a url already cached, and only when its file is
    /// still there: a row pointing at nothing is the same as no row.
    pub fn path_for(&self, conn: &Connection, url: &str) -> Result<Option<String>, CoreError> {
        let relative: Option<String> = conn
            .query_row(
                "SELECT path FROM images WHERE url = ?1",
                params![url],
                |r| r.get(0),
            )
            .optional()?;
        Ok(relative
            .map(|relative| self.dir.join(relative))
            .filter(|path| path.is_file())
            .map(|path| path.to_string_lossy().into_owned()))
    }

    /// Marks images as used, on the writer thread, with no reply waited
    /// for: a read that hands out posters must not pay for the bookkeeping
    /// that keeps them out of the sweep.
    pub fn bump_used(&self, urls: Vec<String>) {
        if urls.is_empty() {
            return;
        }
        let now = time::now_secs();
        self.store.post(move |c| {
            let sql = format!(
                "UPDATE images SET used_at = ? WHERE url IN ({}) AND used_at < ?",
                placeholders(urls.len())
            );
            let mut values: Vec<Value> = Vec::with_capacity(urls.len() + 2);
            values.push(Value::Integer(now));
            values.extend(urls.into_iter().map(Value::Text));
            values.push(Value::Integer(now - USED_GAP));
            if let Err(e) = c.execute(&sql, params_from_iter(values)) {
                tracing::warn!("marking images used failed: {e}");
            }
        });
    }

    pub fn storage(&self, conn: &Connection) -> Result<(u64, u64), CoreError> {
        let (count, bytes): (i64, i64) = conn.query_row(
            "SELECT count(*), COALESCE(sum(bytes), 0) FROM images",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        Ok((count.max(0) as u64, bytes.max(0) as u64))
    }

    /// The four steps, in one transaction for the rows and one walk of the
    /// directory for the files: a row whose file is gone, an orphan past
    /// its thirty days, and the cap, all decided inside the transaction;
    /// then, once it has committed, every file the surviving rows do not
    /// name is deleted, which is both the second step and the disk half of
    /// the third and fourth.
    pub fn sweep(&self, conn: &mut Connection, now: i64) -> Result<SweepReport, CoreError> {
        self.sweep_to(conn, now, MAX_ROWS)
    }

    fn sweep_to(
        &self,
        conn: &mut Connection,
        now: i64,
        cap: u64,
    ) -> Result<SweepReport, CoreError> {
        let mut report = SweepReport::default();
        let keep: HashSet<String>;
        {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let mut rows = all_rows(&tx)?;
            let referenced = urls_from(&tx, REFERENCED_SQL)?;
            let pinned = urls_from(&tx, PINNED_SQL)?;

            // (1) the file is gone, so the row is a lie.
            let mut removed: Vec<String> = Vec::new();
            rows.retain(|row| {
                if self.dir.join(&row.path).is_file() {
                    true
                } else {
                    removed.push(row.url.clone());
                    false
                }
            });
            // (3) nothing references it and nothing has used it in a month.
            rows.retain(|row| {
                if !referenced.contains(&row.url) && row.used_at < now - ORPHAN_AGE {
                    removed.push(row.url.clone());
                    false
                } else {
                    true
                }
            });
            report.removed_rows = removed.len() as u64;

            // (4) the cap, orphans first and least recently used next, and
            // never a poster or a banner of a series the library owns.
            if rows.len() as u64 > cap {
                let mut candidates: Vec<usize> = (0..rows.len())
                    .filter(|i| !pinned.contains(&rows[*i].url))
                    .collect();
                candidates.sort_by_key(|i| {
                    (
                        u8::from(referenced.contains(&rows[*i].url)),
                        rows[*i].used_at,
                    )
                });
                let over = (rows.len() as u64 - cap) as usize;
                let evicted: HashSet<String> = candidates
                    .into_iter()
                    .take(over)
                    .map(|i| rows[i].url.clone())
                    .collect();
                report.evicted = evicted.len() as u64;
                rows.retain(|row| !evicted.contains(&row.url));
                removed.extend(evicted);
            }

            delete_rows(&tx, &removed)?;
            keep = rows.into_iter().map(|row| row.path).collect();
            tx.commit()?;
        }
        // (2) every file no surviving row names, which is a file that never
        // had a row, a leftover .tmp, and the files of everything the steps
        // above just deleted.
        report.removed_files = self.delete_files_outside(&keep);
        Ok(report)
    }

    /// Deletes every file the surviving rows do not name. Run this on the
    /// writer thread, inside the same task as the transaction above: the
    /// row a fetch is about to insert is then queued behind this sweep
    /// rather than racing it. A file written by a fetch whose row has not
    /// been queued yet can still be taken, and that heals itself, since a
    /// row with no file is deleted and refetched by the next `ensure`.
    fn delete_files_outside(&self, keep: &HashSet<String>) -> u64 {
        let mut removed = 0;
        for entry in walkdir::WalkDir::new(&self.dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let Ok(relative) = entry.path().strip_prefix(&self.dir) else {
                continue;
            };
            if keep.contains(relative.to_string_lossy().as_ref()) {
                continue;
            }
            match std::fs::remove_file(entry.path()) {
                Ok(()) => removed += 1,
                // Something else got there first, which is the outcome
                // this wanted anyway.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => tracing::warn!("cannot delete {}: {e}", entry.path().display()),
            }
        }
        removed
    }
}

// ---------------------------------------------------------------------------
// The queries
// ---------------------------------------------------------------------------

struct CachedRow {
    url: String,
    path: String,
    used_at: i64,
}

fn all_rows(conn: &Connection) -> Result<Vec<CachedRow>, CoreError> {
    let mut stmt = conn.prepare("SELECT url, path, used_at FROM images")?;
    let rows = stmt.query_map([], |r| {
        Ok(CachedRow {
            url: r.get(0)?,
            path: r.get(1)?,
            used_at: r.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn urls_from(conn: &Connection, sql: &str) -> Result<HashSet<String>, CoreError> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<Result<HashSet<_>, _>>()?)
}

/// Url to relative path, for the urls asked about that have a row.
fn rows_for(conn: &Connection, urls: &[String]) -> Result<HashMap<String, String>, CoreError> {
    let mut out = HashMap::new();
    for chunk in urls.chunks(500) {
        let sql = format!(
            "SELECT url, path FROM images WHERE url IN ({})",
            placeholders(chunk.len())
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (url, path) = row?;
            out.insert(url, path);
        }
    }
    Ok(out)
}

fn delete_rows(conn: &Connection, urls: &[String]) -> Result<(), CoreError> {
    for chunk in urls.chunks(500) {
        let sql = format!(
            "DELETE FROM images WHERE url IN ({})",
            placeholders(chunk.len())
        );
        conn.execute(&sql, params_from_iter(chunk.iter()))?;
    }
    Ok(())
}

/// The series a set of fetched urls belongs to: the ones whose card or page
/// now has a picture it did not have a moment ago.
fn series_for_urls(conn: &Connection, urls: &[String]) -> Result<Vec<u64>, CoreError> {
    if urls.is_empty() {
        return Ok(Vec::new());
    }
    let list = placeholders(urls.len());
    let sql = format!(
        "SELECT s.id FROM series s JOIN anilist_media m ON m.id = s.anilist_id
         WHERE m.cover_url IN ({list}) OR m.banner_url IN ({list})"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(urls.iter().chain(urls.iter())), |r| {
        r.get::<_, i64>(0)
    })?;
    Ok(rows
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|id| id as u64)
        .collect())
}

fn matched_series(conn: &Connection) -> Result<Vec<u64>, CoreError> {
    let mut stmt =
        conn.prepare("SELECT id FROM series WHERE anilist_id IS NOT NULL ORDER BY id")?;
    let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
    Ok(rows
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|id| id as u64)
        .collect())
}

fn missing_urls(conn: &Connection) -> Result<Vec<String>, CoreError> {
    let mut stmt = conn.prepare(MISSING_SQL)?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

// ---------------------------------------------------------------------------
// The fetch
// ---------------------------------------------------------------------------

/// One url: get it, write it beside its final name, rename it into place,
/// then record the row. The rename is what makes a half-written file
/// impossible to read as a cached image, since only a complete file ever
/// wears the real name.
async fn fetch_one(
    http: &dyn Http,
    store: &Store,
    dir: &Path,
    url: &str,
) -> Result<String, CoreError> {
    let request = HttpRequest {
        method: Method::Get,
        url: url.to_string(),
        headers: Vec::new(),
        body: None,
    };
    let response = http
        .send(request)
        .await
        .map_err(|e| cdn_error(None, e.message))?;
    if !response.is_success() {
        return Err(cdn_error(
            Some(u32::from(response.status)),
            format!("image fetch failed: {}", response.text()),
        ));
    }
    let relative = relative_path(url, extension(url, response.header("content-type")));
    let path = dir.join(&relative);
    let parent = path
        .parent()
        .ok_or_else(|| CoreError::internal("an image path has no parent"))?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|e| CoreError::io_at(parent.to_string_lossy(), e))?;
    let tmp = dir.join(format!("{relative}.tmp"));
    tokio::fs::write(&tmp, &response.body)
        .await
        .map_err(|e| CoreError::io_at(tmp.to_string_lossy(), e))?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| CoreError::io_at(path.to_string_lossy(), e))?;

    let bytes = response.body.len() as i64;
    let now = time::now_secs();
    let (row_url, row_path) = (url.to_string(), relative);
    store
        .write_async(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO images (url, path, bytes, fetched_at, used_at) VALUES (?1, ?2, ?3, ?4, ?4)",
                params![row_url, row_path, bytes, now],
            )?;
            Ok(())
        })
        .await?;
    Ok(path.to_string_lossy().into_owned())
}

// ---------------------------------------------------------------------------
// The jobs
// ---------------------------------------------------------------------------

/// The fill: every url an owned series, its recommendations or the tracker
/// lists would draw, that has no row yet. It runs one at a time, so a read
/// asking for a second one is handed the running job's id.
pub fn start_fill(core: &Arc<Core>) -> u64 {
    // A coalesced call is not a fill starting, so it must not reset the
    // gate the reads consult.
    let coalesced = core.jobs.running(JobKind::FillImages).is_some();
    let owner = core.clone();
    let id = core
        .jobs
        .clone()
        .start(JobKind::FillImages, move |ctx| async move {
            let urls = owner.store.write_async(|c| missing_urls(c)).await?;
            let total = urls.len() as u64;
            let (mut fetched, mut failed) = (0u64, 0u64);
            let mut touched: HashSet<u64> = HashSet::new();

            for (round, batch) in urls.chunks(BATCH).enumerate() {
                ctx.checkpoint()?;
                ctx.progress((round * BATCH) as u64, Some(total), "fetching images");
                let mut arrived: Vec<String> = Vec::new();
                for (url, outcome) in owner.images.ensure(batch).await {
                    match outcome {
                        Ok(_) => {
                            fetched += 1;
                            arrived.push(url);
                        }
                        Err(e) => {
                            failed += 1;
                            tracing::warn!("image {url} was not fetched: {e}");
                        }
                    }
                }
                if !arrived.is_empty() {
                    touched.extend(
                        owner
                            .store
                            .write_async(move |c| series_for_urls(c, &arrived))
                            .await?,
                    );
                }
            }

            if !touched.is_empty() {
                let mut ids: Vec<u64> = touched.into_iter().collect();
                ids.sort_unstable();
                let dir = owner.paths.images_dir();
                let cards = owner
                    .store
                    .write_async(move |c| cards::cards_for(c, &dir, &ids))
                    .await?;
                ctx.emit(
                    Level::Debug,
                    format!("fetched {fetched} images"),
                    EventBody::SeriesChanged { series: cards },
                );
            }
            Ok(Finished {
                level: Level::Info,
                message: format!("images filled: {fetched} fetched, {failed} failed"),
                body: EventBody::Notice,
            })
        });
    if !coalesced {
        core.images.mark_fill_started();
    }
    id
}

/// Clear images: every row and every file, and a card for every matched
/// series so a shell showing posters drops them at once.
pub fn start_clear(core: &Arc<Core>) -> u64 {
    let owner = core.clone();
    core.jobs
        .clone()
        .start(JobKind::ClearImages, move |ctx| async move {
            let removed = owner
                .store
                .write_async(|c| {
                    let count: i64 =
                        c.query_row("SELECT count(*) FROM images", [], |r| r.get(0))?;
                    c.execute("DELETE FROM images", [])?;
                    Ok(count.max(0) as u64)
                })
                .await?;

            // The whole tree goes rather than the listed files: a file with no
            // row is exactly what this call is also for.
            let dir = owner.paths.images_dir();
            match tokio::fs::remove_dir_all(&dir).await {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(CoreError::io_at(dir.to_string_lossy(), e)),
            }
            tokio::fs::create_dir_all(&dir)
                .await
                .map_err(|e| CoreError::io_at(dir.to_string_lossy(), e))?;

            let ids = owner.store.write_async(|c| matched_series(c)).await?;
            if !ids.is_empty() {
                let images_dir = dir.clone();
                let cards = owner
                    .store
                    .write_async(move |c| cards::cards_for(c, &images_dir, &ids))
                    .await?;
                ctx.emit(
                    Level::Debug,
                    format!("{} series lost their images", cards.len()),
                    EventBody::SeriesChanged { series: cards },
                );
            }
            Ok(Finished {
                level: Level::Info,
                message: format!("images cleared: {removed} removed"),
                body: EventBody::ImagesCleared { removed },
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;

    #[test]
    fn the_key_is_the_urls_sha256_and_the_path_shards_on_its_first_two_hex() {
        let k = key("https://img/xl.jpg");
        assert_eq!(k.len(), 64);
        assert!(
            k.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
        );
        assert_eq!(
            key("https://img/xl.jpg"),
            k,
            "the same url always hashes the same"
        );
        assert_ne!(key("https://img/other.jpg"), k);
        assert_eq!(
            relative_path("https://img/xl.jpg", "jpg"),
            format!("{}/{k}.jpg", &k[..2])
        );
    }

    #[test]
    fn the_extension_comes_from_the_url_then_the_content_type_then_jpg() {
        assert_eq!(extension("https://img/a.png", None), "png");
        assert_eq!(extension("https://img/a.PNG", None), "png");
        assert_eq!(extension("https://img/a.jpeg", Some("image/png")), "jpeg");
        assert_eq!(extension("https://img/a.webp?v=2", None), "webp");
        assert_eq!(extension("https://img/a.avif#x", None), "avif");
        assert_eq!(extension("https://img/a.bin", Some("image/gif")), "gif");
        assert_eq!(
            extension("https://img/nodot", Some("image/webp; charset=binary")),
            "webp"
        );
        assert_eq!(
            extension("https://img/nodot", Some("application/octet-stream")),
            "jpg"
        );
        assert_eq!(extension("https://img/nodot", None), "jpg");
        assert_eq!(
            extension("https://cdn.example.com/no-extension-here", None),
            "jpg"
        );
    }

    // The sweep is easier to judge against rows written by hand than
    // against a fill, so this builds the four cases directly.
    struct Bench {
        _dir: tempfile::TempDir,
        store: Arc<Store>,
        cache: Arc<ImageCache>,
        now: i64,
    }

    fn bench() -> Bench {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("data").join("anibeam.db")).unwrap();
        let images = dir.path().join("images");
        std::fs::create_dir_all(&images).unwrap();
        let cache = ImageCache::new(store.clone(), images, FakeHttp::new());
        Bench {
            _dir: dir,
            store,
            cache,
            now: 1_700_000_000,
        }
    }

    impl Bench {
        /// A row and, unless `on_disk` is false, the file it names.
        fn add(&self, url: &str, used_at: i64, on_disk: bool) {
            let relative = relative_path(url, "jpg");
            if on_disk {
                let path = self.cache.dir().join(&relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(&path, b"xx").unwrap();
            }
            let (url, relative) = (url.to_string(), relative);
            self.store
                .write(move |c| {
                    c.execute(
                        "INSERT INTO images (url, path, bytes, fetched_at, used_at) VALUES (?1, ?2, 2, ?3, ?3)",
                        params![url, relative, used_at],
                    )?;
                    Ok(())
                })
                .unwrap();
        }

        fn urls(&self) -> Vec<String> {
            self.store
                .read(|c| {
                    let mut stmt = c.prepare("SELECT url FROM images ORDER BY url")?;
                    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
                    Ok(rows.collect::<Result<Vec<_>, _>>()?)
                })
                .unwrap()
        }

        fn files(&self) -> usize {
            walkdir::WalkDir::new(self.cache.dir())
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .count()
        }
    }

    #[test]
    fn the_sweep_drops_lost_rows_stray_files_and_old_orphans_then_caps_what_is_left() {
        let b = bench();
        let (now, day) = (b.now, 86_400);
        // A matched series pins its cover and banner; a second media row is
        // referenced but owned by nothing.
        b.store
            .write(move |c| {
                c.execute(
                    "INSERT INTO anilist_media (id, cover_url, banner_url, characters) VALUES (1, 'pinned-cover', 'pinned-banner', '[{\"image_url\":\"portrait\"}]')",
                    [],
                )?;
                c.execute("INSERT INTO anilist_media (id, cover_url) VALUES (2, 'loose-cover')", [])?;
                c.execute("INSERT INTO sources (path, added_at) VALUES ('/lib', 1)", [])?;
                c.execute(
                    "INSERT INTO series (source_id, kind, path, folder_name, added_at, anilist_id) VALUES (1, 'show', '/lib/A', 'A', 1, 1)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

        b.add("pinned-cover", now, true);
        b.add("pinned-banner", now, false); // (1) the file is gone
        b.add("portrait", now - day, true);
        b.add("loose-cover", now - 10 * day, true);
        b.add("old-orphan", now - 40 * day, true); // (3) unreferenced and stale
        b.add("new-orphan", now, true); // young enough to survive step 3
        // (2) a file under the directory that no row names
        std::fs::write(b.cache.dir().join("stray.jpg"), b"x").unwrap();

        let cache = b.cache.clone();
        // (4) the cap, small enough to bite: four rows survive the first
        // three steps and two of them have to go.
        let report = b.store.write(move |c| cache.sweep_to(c, now, 2)).unwrap();

        assert_eq!(report.removed_rows, 2, "the lost row and the old orphan");
        assert_eq!(report.evicted, 2);
        // The old orphan's, the new orphan's, the loose cover's, the stray.
        assert_eq!(report.removed_files, 4);
        // The orphan goes before anything referenced, then the least
        // recently used, and the pinned cover is never a candidate.
        assert_eq!(
            b.urls(),
            vec!["pinned-cover".to_string(), "portrait".to_string()]
        );
        assert_eq!(b.files(), 2);
    }

    #[test]
    fn a_sweep_of_an_untouched_cache_changes_nothing() {
        let b = bench();
        b.add("kept", b.now, true);
        let cache = b.cache.clone();
        let now = b.now;
        let report = b.store.write(move |c| cache.sweep(c, now)).unwrap();
        assert_eq!(report, SweepReport::default());
        assert_eq!(b.urls(), vec!["kept".to_string()]);
        assert_eq!(b.files(), 1);
    }

    #[test]
    fn bump_used_only_moves_an_image_a_day_stale_and_path_for_ignores_a_lost_file() {
        let b = bench();
        // `bump_used` reads the real clock, so these rows are placed
        // against it rather than against the bench's fixed instant.
        let now = time::now_secs();
        b.add("fresh", now, true);
        b.add("stale", now - 5 * 86_400, true);
        b.add("lost", now - 5 * 86_400, false);
        b.cache
            .bump_used(vec!["fresh".into(), "stale".into(), "lost".into()]);
        // The next write runs after the posted one, so this sees it.
        let used: Vec<(String, i64)> = b
            .store
            .write(|c| {
                let mut stmt = c.prepare("SELECT url, used_at FROM images ORDER BY url")?;
                let rows =
                    stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            })
            .unwrap();
        let at = |url: &str| {
            used.iter()
                .find(|(u, _)| u == url)
                .map(|(_, at)| *at)
                .unwrap()
        };
        assert_eq!(
            at("fresh"),
            now,
            "a bump inside the day is not worth a write"
        );
        assert!(at("stale") >= now, "a day-stale row is moved up to now");

        let cache = b.cache.clone();
        let (kept, lost) = b
            .store
            .read(|c| Ok((cache.path_for(c, "fresh")?, cache.path_for(c, "lost")?)))
            .unwrap();
        assert!(kept.is_some_and(|p| p.ends_with(".jpg")));
        assert_eq!(lost, None, "a row whose file is gone is the same as no row");
    }

    #[tokio::test]
    async fn ensure_returns_the_path_once_and_never_fetches_a_cached_url_twice() {
        let b = bench();
        let http = FakeHttp::new();
        http.push_for("one.jpg", 200, vec![1, 2, 3]);
        http.push_for("two.jpg", 500, b"boom".to_vec());
        let cache = ImageCache::new(b.store.clone(), b.cache.dir().to_path_buf(), http.clone());

        let urls = vec![
            "https://img/one.jpg".to_string(),
            "https://img/two.jpg".to_string(),
        ];
        let out = cache.ensure(&urls).await;
        assert_eq!(out.len(), 2);
        let one = out[0].1.clone().unwrap();
        assert!(std::path::Path::new(&one).is_file());
        assert!(matches!(
            out[1].1,
            Err(CoreError::Provider {
                status: Some(500),
                ..
            })
        ));
        assert_eq!(http.requests().len(), 2);

        // The cached url comes back off the row; the failed one is asked
        // for again, since nothing was written for it.
        http.push_for("two.jpg", 200, vec![9]);
        let out = cache.ensure(&urls).await;
        assert_eq!(out[0].1.clone().unwrap(), one);
        assert!(out[1].1.is_ok());
        assert_eq!(http.requests().len(), 3);
    }
}
