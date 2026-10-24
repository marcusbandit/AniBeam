//! What a file is read as beyond its name. The walk reads every name on its
//! own; two things it cannot see change that reading before the scan writes
//! it (#167, #170):
//!
//! - The series' own record. A series matched to a run of specials or OVAs
//!   (AniList format SPECIAL or OVA) that holds no regular episode reads its
//!   numbered specials as its episodes: `OVA 3` of the Girls und Panzer OVA
//!   Series is that record's episode 3, not a special of some other show.
//! - The user's quick fixes, stored by path in `file_fixes`, which win over
//!   both the name and the record.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::*;
use crate::library::classifier::{self, Classified};
use crate::library::labels;
use crate::library::walk::ScannedFile;

/// The record formats whose numbered specials are the series' own episodes.
fn specials_are_episodes(format: Option<&str>) -> bool {
    matches!(format, Some("SPECIAL" | "OVA"))
}

fn numbered_special(c: &Classified) -> bool {
    c.extra == Some(ExtraKind::Sp) && c.extra_index.is_some()
}

/// Whether the record turns this series' numbered specials into its
/// episodes: the record is a run of specials or OVAs, a file reads as a
/// numbered special, and none reads as a regular episode.
fn record_rule_applies(format: Option<&str>, files: &[ScannedFile]) -> bool {
    specials_are_episodes(format)
        && files.iter().all(|f| f.classified.extra.is_some())
        && files.iter().any(|f| numbered_special(&f.classified))
}

/// One series' walked files, read past their names: the record rule first,
/// then the fixes, keyed by path.
pub(crate) fn apply(
    format: Option<&str>,
    files: &mut [ScannedFile],
    fixes: &HashMap<String, FileReading>,
) {
    if record_rule_applies(format, files) {
        for f in files.iter_mut().filter(|f| numbered_special(&f.classified)) {
            let number = f64::from(f.classified.extra_index.unwrap_or(0));
            set(
                f,
                &FileReading::Episode {
                    number,
                    season: None,
                },
            );
        }
    }
    for f in files.iter_mut() {
        if let Some(reading) = fixes.get(&f.path) {
            set(f, reading);
        }
    }
}

fn set(f: &mut ScannedFile, reading: &FileReading) {
    let (classified, label, key) = parts(reading, &file_name(&f.path));
    f.classified = classified;
    f.label = label;
    f.episode_key = key;
}

/// What a reading writes for a file of this name: the classification, the
/// row's label and the history key, the way the walk writes its own.
pub(crate) fn parts(reading: &FileReading, name: &str) -> (Classified, String, String) {
    match *reading {
        FileReading::Episode { number, season } => (
            Classified {
                extra: None,
                number,
                season,
                extra_index: None,
                extra_variant: None,
                raw_label: None,
            },
            classifier::clean_episode_title(name),
            classifier::number_key(season, number),
        ),
        FileReading::Extra { kind, index } => (
            Classified {
                extra: Some(kind),
                number: f64::from(index.unwrap_or(0)),
                season: (kind == ExtraKind::Sp).then_some(0),
                extra_index: index,
                extra_variant: None,
                raw_label: None,
            },
            labels::extra_label(kind, index, None, None),
            name.to_string(),
        ),
    }
}

pub(crate) fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The reading a stored file row carries.
pub(crate) fn of_row(
    kind: &str,
    number: Option<f64>,
    season: Option<u32>,
    extra_kind: Option<&str>,
    extra_index: Option<u32>,
) -> FileReading {
    if kind == "extra" {
        FileReading::Extra {
            kind: extra_kind
                .and_then(ExtraKind::from_column)
                .unwrap_or(ExtraKind::Other),
            index: extra_index,
        }
    } else {
        FileReading::Episode {
            number: number.unwrap_or(0.0),
            season,
        }
    }
}

/// A reading in words: `Episode 3`, `Season 2 episode 5`, `Opening 1`,
/// `Special`.
pub(crate) fn describe(reading: &FileReading) -> String {
    match *reading {
        FileReading::Episode {
            number,
            season: None | Some(1),
        } => format!("Episode {}", classifier::format_number(number)),
        FileReading::Episode {
            number,
            season: Some(s),
        } => format!("Season {s} episode {}", classifier::format_number(number)),
        FileReading::Extra { kind, index } => labels::extra_label(kind, index, None, None),
    }
}

/// Every quick fix, by path. One row per fixed file, few by nature, so a scan
/// reads them all once.
pub(crate) fn load_fixes(conn: &Connection) -> Result<HashMap<String, FileReading>, CoreError> {
    let mut stmt = conn.prepare("SELECT path, reading FROM file_fixes")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut out = HashMap::new();
    for row in rows {
        let (path, reading) = row?;
        // A reading this build cannot parse is left to the name, not fatal.
        if let Ok(reading) = serde_json::from_str(&reading) {
            out.insert(path, reading);
        }
    }
    Ok(out)
}

/// The series' AniList record format, `SPECIAL` or `TV`, when it has one.
pub(crate) fn record_format(conn: &Connection, series: u64) -> Result<Option<String>, CoreError> {
    Ok(conn
        .query_row(
            "SELECT m.format FROM series s JOIN anilist_media m ON m.id = s.anilist_id WHERE s.id = ?1",
            params![series as i64],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten())
}

/// Whether a record landing for `series` can change how its files read: a
/// file's name reads as a numbered special, which the record rule reads one
/// way or the other. Then the series is worth a scan.
pub(crate) fn record_matters(conn: &Connection, series: u64) -> Result<bool, CoreError> {
    let mut stmt = conn.prepare("SELECT path FROM files WHERE series_id = ?1")?;
    let paths = stmt
        .query_map(params![series as i64], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(paths
        .iter()
        .any(|p| numbered_special(&classifier::classify(&file_name(p)))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::classifier::classify;

    fn walked(path: &str) -> ScannedFile {
        let name = file_name(path);
        let classified = classify(&name);
        ScannedFile {
            path: path.to_string(),
            size: 1,
            mtime: 1,
            label: String::new(),
            episode_key: classifier::episode_key(&classified, &name),
            classified,
            sidecars: Vec::new(),
        }
    }

    fn gup() -> Vec<ScannedFile> {
        (1..=6)
            .map(|n| {
                walked(&format!(
                    "/lib/GuP Specials/[ak-Submarines] Girls und Panzer - OVA {n} [BD 1080p].mkv"
                ))
            })
            .collect()
    }

    #[test]
    fn a_specials_record_reads_its_numbered_specials_as_episodes() {
        let mut files = gup();
        apply(Some("SPECIAL"), &mut files, &HashMap::new());
        for (i, f) in files.iter().enumerate() {
            let n = (i + 1) as f64;
            assert_eq!(
                (f.classified.extra, f.classified.number, f.classified.season),
                (None, n, None)
            );
            assert_eq!(f.episode_key, classifier::format_number(n));
        }
        let mut files = gup();
        apply(Some("OVA"), &mut files, &HashMap::new());
        assert!(files.iter().all(|f| f.classified.extra.is_none()));
    }

    #[test]
    fn any_other_record_or_a_regular_episode_leaves_them_specials() {
        for format in [None, Some("TV"), Some("MOVIE")] {
            let mut files = gup();
            apply(format, &mut files, &HashMap::new());
            assert!(
                files
                    .iter()
                    .all(|f| f.classified.extra == Some(ExtraKind::Sp)),
                "{format:?}"
            );
        }
        let mut files = gup();
        files.push(walked("/lib/GuP Specials/Girls und Panzer - 01.mkv"));
        apply(Some("SPECIAL"), &mut files, &HashMap::new());
        assert_eq!(
            files
                .iter()
                .filter(|f| f.classified.extra.is_some())
                .count(),
            6
        );
    }

    #[test]
    fn a_fix_wins_over_the_name_and_the_record() {
        let mut files = gup();
        let mut fixes = HashMap::new();
        fixes.insert(
            files[2].path.clone(),
            FileReading::Extra {
                kind: ExtraKind::Op,
                index: Some(1),
            },
        );
        fixes.insert(
            files[3].path.clone(),
            FileReading::Episode {
                number: 9.0,
                season: Some(2),
            },
        );
        apply(Some("SPECIAL"), &mut files, &fixes);
        assert_eq!(
            (files[2].classified.extra, files[2].label.as_str()),
            (Some(ExtraKind::Op), "Opening 1")
        );
        assert_eq!(files[2].episode_key, file_name(&files[2].path));
        assert_eq!(
            (files[3].classified.number, files[3].episode_key.as_str()),
            (9.0, "S02E09")
        );
        assert_eq!(files[0].classified.number, 1.0);
    }

    #[test]
    fn readings_in_words() {
        assert_eq!(
            describe(&FileReading::Episode {
                number: 3.0,
                season: None
            }),
            "Episode 3"
        );
        assert_eq!(
            describe(&FileReading::Episode {
                number: 5.0,
                season: Some(2)
            }),
            "Season 2 episode 5"
        );
        assert_eq!(
            describe(&FileReading::Extra {
                kind: ExtraKind::Sp,
                index: None
            }),
            "Special"
        );
        assert_eq!(
            describe(&FileReading::Extra {
                kind: ExtraKind::Ed,
                index: Some(2)
            }),
            "Ending 2"
        );
        assert_eq!(
            of_row("extra", None, Some(0), Some("sp"), None),
            FileReading::Extra {
                kind: ExtraKind::Sp,
                index: None
            }
        );
        assert_eq!(
            of_row("episode", Some(4.0), None, None, None),
            FileReading::Episode {
                number: 4.0,
                season: None
            }
        );
    }
}
