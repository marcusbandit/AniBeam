//! Quick fixes (#170): the user saying what a file is when its name read
//! wrong. A fix is stored by the file's path with the reading it replaced, the
//! row is rewritten at once so the page follows without waiting on a scan, and
//! the file's watch history moves to its new key. Every later scan reads the
//! file the same way (`readings::apply`). Undoing a fix puts the replaced
//! reading back and lets the scan read the name again.

use rusqlite::{OptionalExtension, Transaction, params};

use crate::contract::*;
use crate::core::Core;
use crate::library::{cards, readings};
use crate::time;

/// One file row as a fix needs it.
struct Row {
    series: u64,
    path: String,
    reading: FileReading,
    key: String,
}

fn row(tx: &Transaction, file: u64) -> Result<Option<Row>, CoreError> {
    Ok(tx
        .query_row(
            "SELECT series_id, path, kind, number, season, extra_kind, extra_index, episode_key FROM files WHERE id = ?1",
            params![file as i64],
            |r| {
                let kind: String = r.get(2)?;
                let extra_kind: Option<String> = r.get(5)?;
                Ok(Row {
                    series: r.get::<_, i64>(0)? as u64,
                    path: r.get(1)?,
                    reading: readings::of_row(
                        &kind,
                        r.get(3)?,
                        r.get::<_, Option<i64>>(4)?.map(|v| v as u32),
                        extra_kind.as_deref(),
                        r.get::<_, Option<i64>>(6)?.map(|v| v as u32),
                    ),
                    key: r.get(7)?,
                })
            },
        )
        .optional()?)
}

/// A reading the library can hold: a real episode number, a season and an
/// index of sane size.
fn check(reading: &FileReading) -> Result<(), CoreError> {
    match *reading {
        FileReading::Episode { number, season } => {
            if !number.is_finite() || !(0.0..=10_000.0).contains(&number) {
                return Err(CoreError::invalid(
                    "fixes",
                    "an episode number runs from 0 to 10000",
                ));
            }
            if season.is_some_and(|s| s > 200) {
                return Err(CoreError::invalid("fixes", "a season runs up to 200"));
            }
        }
        FileReading::Extra { index, .. } => {
            if index.is_some_and(|i| i > 10_000) {
                return Err(CoreError::invalid(
                    "fixes",
                    "an extra's index runs up to 10000",
                ));
            }
        }
    }
    Ok(())
}

/// Moves a file's watch history from its old key to its new one, unless
/// another file of the series still carries the old key (a second release of
/// the same episode keeps it) or the new key has history of its own.
fn move_history(
    tx: &Transaction,
    series: u64,
    file: u64,
    from: &str,
    to: &str,
) -> Result<(), CoreError> {
    if from == to {
        return Ok(());
    }
    let shared: i64 = tx.query_row(
        "SELECT count(*) FROM files WHERE series_id = ?1 AND episode_key = ?2 AND id <> ?3",
        params![series as i64, from, file as i64],
        |r| r.get(0),
    )?;
    if shared > 0 {
        return Ok(());
    }
    for table in ["completed", "resume_points", "skip_windows"] {
        tx.execute(
            &format!(
                "UPDATE OR IGNORE {table} SET episode_key = ?3 WHERE series_id = ?1 AND episode_key = ?2"
            ),
            params![series as i64, from, to],
        )?;
    }
    tx.execute(
        "UPDATE views SET episode_key = ?3 WHERE series_id = ?1 AND episode_key = ?2",
        params![series as i64, from, to],
    )?;
    Ok(())
}

/// Writes one reading onto a file row: kind, numbers, label and key.
fn rewrite(
    tx: &Transaction,
    file: u64,
    path: &str,
    reading: &FileReading,
) -> Result<String, CoreError> {
    let (c, label, key) = readings::parts(reading, &readings::file_name(path));
    tx.execute(
        "UPDATE files SET kind = ?2, number = ?3, season = ?4, extra_kind = ?5, extra_index = ?6, label = ?7, episode_key = ?8 WHERE id = ?1",
        params![
            file as i64,
            if c.extra.is_some() { "extra" } else { "episode" },
            c.extra.is_none().then_some(c.number),
            c.season,
            c.extra.map(|k| k.as_str()),
            c.extra_index,
            label,
            key
        ],
    )?;
    Ok(key)
}

/// Applies the fixes in one transaction; nothing is written if one file is
/// not the series' own. Returns how many files changed.
pub(crate) fn write_fixes(
    tx: &Transaction,
    series: u64,
    fixes: &[FileFix],
    now: i64,
) -> Result<usize, CoreError> {
    let mut changed = 0;
    for fix in fixes {
        let row = row(tx, fix.file)?.ok_or_else(|| {
            CoreError::invalid("fixes", format!("no file {} in the library", fix.file))
        })?;
        if row.series != series {
            return Err(CoreError::invalid(
                "fixes",
                format!(
                    "file {} belongs to another series, so nothing was fixed",
                    fix.file
                ),
            ));
        }
        let target = match &fix.reading {
            Some(reading) => {
                // The first fix keeps what the scan read; a fix of a fix only
                // replaces the reading.
                tx.execute(
                    "INSERT INTO file_fixes (path, reading, was, fixed_at) VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(path) DO UPDATE SET reading = excluded.reading, fixed_at = excluded.fixed_at",
                    params![
                        row.path,
                        serde_json::to_string(reading)?,
                        serde_json::to_string(&row.reading)?,
                        now
                    ],
                )?;
                reading.clone()
            }
            None => {
                let was: Option<String> = tx
                    .query_row(
                        "SELECT was FROM file_fixes WHERE path = ?1",
                        params![row.path],
                        |r| r.get(0),
                    )
                    .optional()?;
                tx.execute("DELETE FROM file_fixes WHERE path = ?1", params![row.path])?;
                match was.and_then(|w| serde_json::from_str::<FileReading>(&w).ok()) {
                    Some(was) => was,
                    // Never fixed: nothing to undo.
                    None => continue,
                }
            }
        };
        if target != row.reading {
            let key = rewrite(tx, fix.file, &row.path, &target)?;
            move_history(tx, series, fix.file, &row.key, &key)?;
            changed += 1;
        }
    }
    Ok(changed)
}

pub fn fix_files(core: &Core, series: u64, fixes: Vec<FileFix>) -> Result<Reply, CoreError> {
    if fixes.is_empty() {
        return Ok(Reply::Ok);
    }
    for reading in fixes.iter().filter_map(|f| f.reading.as_ref()) {
        check(reading)?;
    }
    let now = time::now_secs();
    let undo = fixes.iter().all(|f| f.reading.is_none());
    let n = fixes.len();
    let changed = core.store.tx(move |tx| {
        let found: Option<i64> = tx
            .query_row(
                "SELECT id FROM series WHERE id = ?1",
                params![series as i64],
                |r| r.get(0),
            )
            .optional()?;
        if found.is_none() {
            return Err(CoreError::NotFound {
                what: Entity::Series,
                id: series,
            });
        }
        write_fixes(tx, series, &fixes, now)
    })?;
    if changed > 0 {
        let images_dir = core.paths.images_dir();
        let cards = core
            .store
            .read(|c| cards::cards_for(c, &images_dir, &[series]))?;
        let title = cards.first().map_or_else(String::new, |c| c.title.clone());
        let files = if n == 1 {
            "1 file".to_string()
        } else {
            format!("{n} files")
        };
        core.bus.info(
            Stage::Library,
            if undo {
                format!("{title}: {files} read by name again")
            } else {
                format!("{title}: fixed how {files} read")
            },
            EventBody::SeriesChanged { series: cards },
        );
    }
    // The walk reads the names again, the fixes on top, so an undone file
    // lands on whatever its name says today.
    super::scan::rescan_series(core, series)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn setup() -> (tempfile::TempDir, std::sync::Arc<Store>, u64, Vec<u64>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("anibeam.db")).unwrap();
        let (series, files) = store
            .tx(|tx| {
                tx.execute("INSERT INTO sources (path, available, added_at) VALUES ('/lib', 1, 0)", [])?;
                tx.execute(
                    "INSERT INTO series (source_id, kind, path, folder_name, added_at) VALUES (1, 'show', '/lib/GuP', 'GuP', 0)",
                    [],
                )?;
                let series = tx.last_insert_rowid() as u64;
                let mut files = Vec::new();
                for n in 1..=2 {
                    let name = format!("GuP - OVA {n}.mkv");
                    tx.execute(
                        "INSERT INTO files (series_id, path, size, mtime, kind, number, season, extra_kind, extra_index, label, episode_key, seen_at)
                         VALUES (?1, ?2, 1, 1, 'extra', NULL, 0, 'sp', NULL, 'Special', ?3, 0)",
                        params![series as i64, format!("/lib/GuP/{name}"), name],
                    )?;
                    files.push(tx.last_insert_rowid() as u64);
                }
                // Watched as an extra before the fix.
                tx.execute(
                    "INSERT INTO completed (series_id, episode_key, at) VALUES (?1, 'GuP - OVA 1.mkv', 5)",
                    params![series as i64],
                )?;
                tx.execute(
                    "INSERT INTO resume_points (series_id, episode_key, position, duration, at) VALUES (?1, 'GuP - OVA 2.mkv', 60, 1500, 6)",
                    params![series as i64],
                )?;
                Ok((series, files))
            })
            .unwrap();
        (dir, store, series, files)
    }

    fn episode(n: f64) -> Option<FileReading> {
        Some(FileReading::Episode {
            number: n,
            season: None,
        })
    }

    #[test]
    fn a_fix_rewrites_the_row_keeps_what_was_read_and_moves_the_history() {
        let (_dir, store, series, files) = setup();
        let fixes = vec![
            FileFix {
                file: files[0],
                reading: episode(1.0),
            },
            FileFix {
                file: files[1],
                reading: episode(2.0),
            },
        ];
        let changed = store
            .tx(move |tx| write_fixes(tx, series, &fixes, 10))
            .unwrap();
        assert_eq!(changed, 2);
        let rows: Vec<(String, Option<f64>, String, String)> = store
            .read(|c| {
                let mut stmt =
                    c.prepare("SELECT kind, number, label, episode_key FROM files ORDER BY id")?;
                let rows =
                    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            })
            .unwrap();
        assert_eq!(
            rows[0],
            (
                "episode".into(),
                Some(1.0),
                "GuP - OVA 1".into(),
                "1".into()
            )
        );
        assert_eq!(rows[1].3, "2");
        let (completed, resume): (String, String) = store
            .read(|c| {
                Ok((
                    c.query_row("SELECT episode_key FROM completed", [], |r| r.get(0))?,
                    c.query_row("SELECT episode_key FROM resume_points", [], |r| r.get(0))?,
                ))
            })
            .unwrap();
        assert_eq!((completed.as_str(), resume.as_str()), ("1", "2"));
        let was: String = store
            .read(|c| {
                Ok(c.query_row(
                    "SELECT was FROM file_fixes WHERE path = '/lib/GuP/GuP - OVA 1.mkv'",
                    [],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(
            serde_json::from_str::<FileReading>(&was).unwrap(),
            FileReading::Extra {
                kind: ExtraKind::Sp,
                index: None
            }
        );
    }

    #[test]
    fn a_second_fix_keeps_the_first_reading_and_an_undo_puts_it_back() {
        let (_dir, store, series, files) = setup();
        let f = files[0];
        store
            .tx(move |tx| {
                write_fixes(
                    tx,
                    series,
                    &[FileFix {
                        file: f,
                        reading: episode(1.0),
                    }],
                    10,
                )
            })
            .unwrap();
        store
            .tx(move |tx| {
                write_fixes(
                    tx,
                    series,
                    &[FileFix {
                        file: f,
                        reading: episode(3.0),
                    }],
                    11,
                )
            })
            .unwrap();
        let (n, was): (i64, String) = store
            .read(|c| {
                Ok(
                    c.query_row("SELECT count(*), max(was) FROM file_fixes", [], |r| {
                        Ok((r.get(0)?, r.get(1)?))
                    })?,
                )
            })
            .unwrap();
        assert_eq!(n, 1);
        assert!(was.contains("Extra"));
        store
            .tx(move |tx| {
                write_fixes(
                    tx,
                    series,
                    &[FileFix {
                        file: f,
                        reading: None,
                    }],
                    12,
                )
            })
            .unwrap();
        let (kind, key, completed, fixes): (String, String, String, i64) = store
            .read(move |c| {
                Ok((
                    c.query_row(
                        "SELECT kind FROM files WHERE id = ?1",
                        params![f as i64],
                        |r| r.get(0),
                    )?,
                    c.query_row(
                        "SELECT episode_key FROM files WHERE id = ?1",
                        params![f as i64],
                        |r| r.get(0),
                    )?,
                    c.query_row("SELECT episode_key FROM completed", [], |r| r.get(0))?,
                    c.query_row("SELECT count(*) FROM file_fixes", [], |r| r.get(0))?,
                ))
            })
            .unwrap();
        assert_eq!(
            (kind.as_str(), key.as_str(), fixes),
            ("extra", "GuP - OVA 1.mkv", 0)
        );
        assert_eq!(completed, "GuP - OVA 1.mkv");
    }

    #[test]
    fn another_series_file_refuses_the_whole_fix() {
        let (_dir, store, series, files) = setup();
        let fixes = vec![
            FileFix {
                file: files[0],
                reading: episode(1.0),
            },
            FileFix {
                file: 999,
                reading: episode(2.0),
            },
        ];
        assert!(
            store
                .tx(move |tx| write_fixes(tx, series, &fixes, 10))
                .is_err()
        );
        let n: i64 = store
            .read(|c| Ok(c.query_row("SELECT count(*) FROM file_fixes", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(n, 0);
        let other = series + 1;
        let f = files[0];
        assert!(
            store
                .tx(move |tx| write_fixes(
                    tx,
                    other,
                    &[FileFix {
                        file: f,
                        reading: episode(1.0)
                    }],
                    10
                ))
                .is_err()
        );
        assert!(
            check(&FileReading::Episode {
                number: f64::NAN,
                season: None
            })
            .is_err()
        );
        assert!(
            check(&FileReading::Episode {
                number: -1.0,
                season: None
            })
            .is_err()
        );
        assert!(
            check(&FileReading::Episode {
                number: 12.5,
                season: Some(2)
            })
            .is_ok()
        );
    }
}
