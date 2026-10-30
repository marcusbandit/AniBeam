//! Deleting episodes (#158): the files of chosen episodes, with their subtitle sidecars, out
//! of the library. On a desktop they go to the trash, where they can be put back; a phone has
//! no trash and deletes them (`downloads::trash`). The series is scanned again at once, so its
//! page and cubes follow without waiting on the watcher. A torrent that held one of the files
//! leaves the client with them (#266), and Delete watched does the same across the whole
//! library at once.

use std::path::{Path, PathBuf};

use rusqlite::params;

use crate::contract::*;
use crate::core::Core;

/// One file row as the delete needs it.
struct Row {
    series: u64,
    path: String,
    /// The episode number, for counting episodes rather than files.
    number: Option<f64>,
    sidecars: Vec<Sidecar>,
}

/// The paths to remove for `files`, every one checked: each must be a row of `series`, an
/// absolute plain path, and sit strictly inside a library source. One that fails refuses the
/// whole request, as the trash does for a show.
fn paths(
    rows: &[(u64, Option<Row>)],
    series: u64,
    sources: &[String],
) -> Result<Vec<PathBuf>, CoreError> {
    let sources: Vec<PathBuf> = sources
        .iter()
        .map(|s| PathBuf::from(s.trim_end_matches('/')))
        .collect();
    let mut out = Vec::new();
    for (id, row) in rows {
        let row = row
            .as_ref()
            .ok_or_else(|| CoreError::invalid("files", format!("no file {id} in the library")))?;
        if row.series != series {
            return Err(CoreError::invalid(
                "files",
                format!("file {id} belongs to another series, so nothing was removed"),
            ));
        }
        let main = PathBuf::from(&row.path);
        for p in
            std::iter::once(main.clone()).chain(row.sidecars.iter().map(|s| PathBuf::from(&s.path)))
        {
            let plain = p.is_absolute()
                && !p
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir));
            let inside = sources.iter().any(|s| p.starts_with(s) && p != *s);
            if !plain || !inside {
                return Err(CoreError::invalid(
                    "files",
                    format!(
                        "{} is not a file inside a library folder, so nothing was removed",
                        p.display()
                    ),
                ));
            }
            // Already gone (a sidecar, or a file removed by hand that the library still
            // lists): nothing to move, and the rescan below drops its row.
            if !Path::new(&p).exists() {
                continue;
            }
            out.push(p);
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

pub fn delete_episodes(core: &Core, series: u64, files: Vec<u64>) -> Result<Reply, CoreError> {
    if files.is_empty() {
        return Ok(Reply::Ok);
    }
    let (rows, sources, title) = core.store.read(|c| {
        let mut rows = Vec::new();
        for id in &files {
            let row = c
                .query_row(
                    "SELECT series_id, path, sidecars, number FROM files WHERE id = ?1",
                    params![*id as i64],
                    |r| {
                        Ok(Row {
                            series: r.get::<_, i64>(0)? as u64,
                            path: r.get(1)?,
                            sidecars: serde_json::from_str(&r.get::<_, String>(2)?)
                                .unwrap_or_default(),
                            number: r.get(3)?,
                        })
                    },
                )
                .ok();
            rows.push((*id, row));
        }
        let mut stmt = c.prepare("SELECT path FROM sources")?;
        let sources = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let title: String = c
            .query_row(
                "SELECT folder_name FROM series WHERE id = ?1",
                params![series as i64],
                |r| r.get(0),
            )
            .unwrap_or_default();
        Ok((rows, sources, title))
    })?;
    let targets = paths(&rows, series, &sources)?;
    // Episodes, not files: two releases of one episode are one episode deleted.
    let mut numbers: Vec<String> = rows
        .iter()
        .filter_map(|(_, r)| r.as_ref())
        .map(|r| r.number.map_or_else(|| r.path.clone(), |n| n.to_string()))
        .collect();
    numbers.sort();
    numbers.dedup();
    let n = numbers.len();
    // A trash that fails part way has still moved what it moved: the library is read again
    // either way, and the failure is reported after.
    let moved = crate::downloads::trash::move_to_trash(&targets);
    crate::downloads::drop_torrents_of(core, gone(&targets));
    if moved.is_ok() {
        core.bus.info(
            Stage::Library,
            format!(
                "{} {} of {title} {}",
                n,
                if n == 1 { "episode" } else { "episodes" },
                if crate::downloads::trash::WHERE == "deleted" {
                    "deleted"
                } else {
                    "moved to the trash"
                }
            ),
            EventBody::Notice,
        );
    }
    let scan = super::scan::rescan_series(core, series);
    moved?;
    scan
}

/// What of `targets` is really gone: a trash that failed part way leaves the rest, and a
/// torrent of a file still here stays in the client.
fn gone(targets: &[PathBuf]) -> Vec<PathBuf> {
    targets.iter().filter(|p| !p.exists()).cloned().collect()
}

/// One series' share of a Delete watched: its id, its file rows, and their bytes.
type Picked = (u64, Vec<(u64, Option<Row>)>, u64);

/// Deletes across the whole library (#266): every watched episode's file, or with
/// `finished_shows` every file of each show confirmed done with
/// (`SeriesCard::finished_watched`), its extras too. To the trash, as `delete_episodes`
/// does, and a file a torrent is still writing stays. Every path is checked first and one
/// that fails refuses the lot. Each series it touched is scanned again.
pub fn delete_watched(core: &Core, finished_shows: bool) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let now = crate::time::now();
    let (picked, sources) = core.store.read(|c| {
        let snap = super::snapshot::Snapshot::load(c, &images_dir, now, None)?;
        let mut picked: Vec<Picked> = Vec::new();
        for row in &snap.series {
            // A series whose folder is gone has nothing left to delete
            let w = snap.watched_of(row);
            if row.missing || (finished_shows && !snap.finished_watched(row, &w)) {
                continue;
            }
            let mut rows = Vec::new();
            let mut bytes = 0;
            for f in snap.files_of(row.id) {
                // A finished show takes its extras too; a file of another season or past the
                // count, which nothing says was watched, stays either way
                let take = w.file(f) || (finished_shows && !f.is_episode);
                // Gone already (an earlier delete the rescan has not caught up with) is not
                // counted as going now
                if !take
                    || crate::downloads::is_writing(core, &f.path)
                    || !Path::new(&f.path).exists()
                {
                    continue;
                }
                bytes += f.size;
                rows.push((
                    f.id,
                    Some(Row {
                        series: row.id,
                        path: f.path.clone(),
                        number: f.is_episode.then_some(f.number),
                        sidecars: f.sidecars.clone(),
                    }),
                ));
            }
            if !rows.is_empty() {
                picked.push((row.id, rows, bytes));
            }
        }
        let mut stmt = c.prepare("SELECT path FROM sources")?;
        let sources = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((picked, sources))
    })?;
    let mut targets = Vec::new();
    let mut episodes = 0u64;
    let mut bytes = 0u64;
    for (series, rows, b) in &picked {
        targets.extend(paths(rows, *series, &sources)?);
        let mut numbers: Vec<String> = rows
            .iter()
            .filter_map(|(_, r)| r.as_ref())
            .filter_map(|r| r.number.map(|n| n.to_string()))
            .collect();
        numbers.sort();
        numbers.dedup();
        episodes += numbers.len() as u64;
        bytes += b;
    }
    if targets.is_empty() {
        return Ok(Reply::Deleted {
            series: 0,
            episodes: 0,
            bytes: 0,
        });
    }
    let moved = crate::downloads::trash::move_to_trash(&targets);
    crate::downloads::drop_torrents_of(core, gone(&targets));
    if moved.is_ok() {
        let what = if finished_shows {
            format!(
                "{} finished {}",
                picked.len(),
                if picked.len() == 1 { "show" } else { "shows" }
            )
        } else {
            format!(
                "{} watched {} of {} {}",
                episodes,
                if episodes == 1 { "episode" } else { "episodes" },
                picked.len(),
                "series"
            )
        };
        core.bus.info(
            Stage::Library,
            format!(
                "{what} {}",
                if crate::downloads::trash::WHERE == "deleted" {
                    "deleted"
                } else {
                    "moved to the trash"
                }
            ),
            EventBody::Notice,
        );
    }
    // A trash that fails part way has still moved what it moved: every series is read again
    for (series, _, _) in &picked {
        let _ = super::scan::rescan_series(core, *series);
    }
    moved?;
    Ok(Reply::Deleted {
        series: picked.len() as u64,
        episodes,
        bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(series: u64, path: &str) -> Option<Row> {
        Some(Row {
            series,
            path: path.into(),
            number: Some(1.0),
            sidecars: Vec::new(),
        })
    }

    #[test]
    fn only_files_of_the_series_inside_a_library_folder() {
        let dir = tempfile::tempdir().unwrap();
        let lib = dir.path().join("lib");
        std::fs::create_dir_all(lib.join("Show")).unwrap();
        let ep = lib.join("Show/01.mkv");
        std::fs::write(&ep, b"x").unwrap();
        let outside = dir.path().join("01.mkv");
        std::fs::write(&outside, b"x").unwrap();
        let sources = vec![lib.to_string_lossy().into_owned()];
        let p = |s: &std::path::Path| s.to_string_lossy().into_owned();
        assert_eq!(
            paths(&[(1, row(7, &p(&ep)))], 7, &sources).unwrap(),
            vec![ep.clone()]
        );
        assert!(paths(&[(1, row(8, &p(&ep)))], 7, &sources).is_err());
        assert!(paths(&[(1, row(7, &p(&outside)))], 7, &sources).is_err());
        assert!(paths(&[(1, row(7, &p(&lib)))], 7, &sources).is_err());
        assert!(
            paths(
                &[(1, row(7, &format!("{}/Show/../../01.mkv", p(&lib))))],
                7,
                &sources
            )
            .is_err()
        );
        assert!(paths(&[(1, None)], 7, &sources).is_err());
        // A file the library still lists but that is already gone is left to the rescan.
        let gone = lib.join("Show/02.mkv");
        assert!(
            paths(&[(2, row(7, &p(&gone)))], 7, &sources)
                .unwrap()
                .is_empty()
        );
    }
}
