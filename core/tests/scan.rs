mod common;
use anibeam_core::*;
use std::fs;
use std::path::Path;

fn touch(p: &Path) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, b"x").unwrap();
}

fn started(reply: Reply) -> u64 {
    match reply {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

#[test]
fn add_source_scans_reconciles_marks_missing_and_forgets() {
    let (dir, core, c) = common::open_core();
    let lib = dir.path().join("lib");
    for n in 1..=3 {
        touch(&lib.join("Show A").join(format!("Show A - {n:02}.mkv")));
    }
    touch(&lib.join("Movies").join("Film (2001).mkv"));

    let source = match core
        .call(Call::AddSource {
            path: lib.to_string_lossy().into_owned(),
        })
        .unwrap()
    {
        Reply::Source { source } => {
            assert!(source.available);
            source.id
        }
        other => panic!("{other:?}"),
    };
    let scan = common::wait_for(
        &c,
        |e| {
            matches!(
                e.body,
                EventBody::JobStarted {
                    kind: JobKind::Scan
                }
            )
        },
        std::time::Duration::from_secs(5),
    )
    .job
    .unwrap()
    .id;
    let done = common::wait_job(&c, scan);
    assert!(
        matches!(
            done.body,
            EventBody::ScanFinished {
                added: 2,
                changed: 0,
                removed: 0,
                ..
            }
        ),
        "{done:?}"
    );
    assert!(
        c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::SeriesChanged { series } if series.len() == 2))
    );

    match core.call(Call::ListSources).unwrap() {
        Reply::Sources { sources } => {
            assert_eq!(sources[0].series_count, 2);
            assert_eq!(
                sources[0].movie_folders,
                vec![lib.join("Movies").to_string_lossy().into_owned()]
            );
        }
        other => panic!("{other:?}"),
    }
    let cards = match core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Series { series } => series,
        other => panic!("{other:?}"),
    };
    let show = cards.iter().find(|s| s.kind == SeriesKind::Show).unwrap();
    assert_eq!((show.title.as_str(), show.episodes_on_disk), ("Show A", 3));
    assert_eq!(
        cards
            .iter()
            .find(|s| s.kind == SeriesKind::Movie)
            .unwrap()
            .title,
        "Film"
    );

    // A fourth file lands, one is deleted: a rescan is "changed", not "added".
    touch(&lib.join("Show A").join("Show A - 04.mkv"));
    fs::remove_file(lib.join("Show A").join("Show A - 01.mkv")).unwrap();
    let job = started(
        core.call(Call::Scan {
            source: Some(source),
        })
        .unwrap(),
    );
    assert!(matches!(
        common::wait_job(&c, job).body,
        EventBody::ScanFinished {
            added: 0,
            changed: 1,
            removed: 0,
            ..
        }
    ));

    // The folder goes away: the series is missing, its history stays, Forget removes it.
    fs::remove_dir_all(lib.join("Show A")).unwrap();
    let job = started(core.call(Call::Scan { source: None }).unwrap());
    assert!(matches!(
        common::wait_job(&c, job).body,
        EventBody::ScanFinished {
            added: 0,
            changed: 0,
            removed: 1,
            ..
        }
    ));
    // The series that went missing leaves in the scan's own batch, card and
    // all, so a shell patching its grid from events can drop it.
    assert!(
        c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::SeriesChanged { series }
            if series.iter().any(|s| s.title == "Show A" && s.missing && s.episodes_on_disk == 0))),
        "{:?}",
        c.bodies()
    );
    let all = match core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Series { series } => series,
        other => panic!("{other:?}"),
    };
    assert_eq!(all.len(), 1);
    let missing = match core
        .call(Call::ListMetadata {
            filter: MetadataFilter::MissingFiles,
            query: String::new(),
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Metadata { rows, .. } => rows,
        other => panic!("{other:?}"),
    };
    assert_eq!(missing.len(), 1);
    assert!(missing[0].series.missing);
    let film = all[0].id;
    assert!(matches!(
        core.call(Call::ForgetSeries { series: film }),
        Err(CoreError::Refused {
            reason: Refusal::OnDisk
        })
    ));
    assert!(matches!(
        core.call(Call::ForgetSeries {
            series: missing[0].series.id
        })
        .unwrap(),
        Reply::Ok
    ));
    assert!(c.bodies().iter().any(
        |b| matches!(b, EventBody::SeriesRemoved { ids } if ids == &vec![missing[0].series.id])
    ));

    // The folder returns: the row reattaches.
    for n in 2..=4 {
        touch(&lib.join("Show B").join(format!("Show B - {n:02}.mkv")));
    }
    let job = started(core.call(Call::RescanSeries { series: film }).unwrap());
    assert!(matches!(
        common::wait_job(&c, job).body,
        EventBody::ScanFinished {
            added: 0,
            changed: 0,
            removed: 0,
            ..
        }
    ));
    let job = started(core.call(Call::Scan { source: None }).unwrap());
    assert!(matches!(
        common::wait_job(&c, job).body,
        EventBody::ScanFinished { added: 1, .. }
    ));

    // Removing the source takes its series with it.
    assert!(matches!(
        core.call(Call::RemoveSource { source }).unwrap(),
        Reply::Ok
    ));
    assert!(
        c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::SourceRemoved { source: s } if *s == source))
    );
    assert!(
        matches!(core.call(Call::ListSources).unwrap(), Reply::Sources { sources } if sources.is_empty())
    );
}

/// A file only ever moves between series by being reclassified, since a
/// move on disk changes its path and nothing collides. One season under a
/// folder makes the folder itself the series; a second season makes it a
/// wrapper, and the first season's files, at the very same paths, belong to
/// a new row. Both rows exist inside one transaction, so the scan has to
/// hand the paths over rather than trip `files.path`'s UNIQUE constraint.
#[test]
fn a_reclassified_file_changes_series_without_colliding_on_its_path() {
    let (dir, core, c) = common::open_core();
    let lib = dir.path().join("lib");
    for n in 1..=3 {
        touch(
            &lib.join("Show A")
                .join("Season 1")
                .join(format!("Show A - {n:02}.mkv")),
        );
    }
    let source = match core
        .call(Call::AddSource {
            path: lib.to_string_lossy().into_owned(),
        })
        .unwrap()
    {
        Reply::Source { source } => source.id,
        other => panic!("{other:?}"),
    };
    let scan = common::wait_for(
        &c,
        |e| {
            matches!(
                e.body,
                EventBody::JobStarted {
                    kind: JobKind::Scan
                }
            )
        },
        std::time::Duration::from_secs(5),
    )
    .job
    .unwrap()
    .id;
    let done = common::wait_job(&c, scan);
    assert!(
        matches!(done.body, EventBody::ScanFinished { added: 1, .. }),
        "{done:?}"
    );

    for n in 1..=2 {
        touch(
            &lib.join("Show A")
                .join("Season 2")
                .join(format!("Show A - S02E{n:02}.mkv")),
        );
    }
    let job = started(
        core.call(Call::Scan {
            source: Some(source),
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::ScanFinished {
                added: 2,
                changed: 0,
                removed: 1,
                ..
            }
        ),
        "{done:?}"
    );

    let cards = match core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Series { series } => series,
        other => panic!("{other:?}"),
    };
    let mut on_disk: Vec<u64> = cards.iter().map(|s| s.episodes_on_disk).collect();
    on_disk.sort_unstable();
    assert_eq!(on_disk, vec![2, 3], "{cards:?}");
    assert!(
        cards
            .iter()
            .any(|s| s.path == lib.join("Show A").join("Season 1").to_string_lossy())
    );

    // The folder that used to be the series is missing, not deleted.
    let missing = match core
        .call(Call::ListMetadata {
            filter: MetadataFilter::MissingFiles,
            query: String::new(),
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Metadata { rows, .. } => rows,
        other => panic!("{other:?}"),
    };
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].series.path, lib.join("Show A").to_string_lossy());
}

/// Two sources that overlap would walk one tree twice and fight over every
/// file in it, so the second one is refused when it is added rather than
/// left to fail the reconcile. A trailing separator is not a difference.
#[test]
fn an_overlapping_source_is_refused_at_the_door() {
    let (dir, core, c) = common::open_core();
    let lib = dir.path().join("lib");
    for n in 1..=2 {
        touch(&lib.join("Show A").join(format!("Show A - {n:02}.mkv")));
    }
    // Added with a trailing slash: the column holds the path without one.
    match core
        .call(Call::AddSource {
            path: format!("{}/", lib.to_string_lossy()),
        })
        .unwrap()
    {
        Reply::Source { source } => assert_eq!(source.path, lib.to_string_lossy()),
        other => panic!("{other:?}"),
    }

    let refusal = |path: String| match core.call(Call::AddSource { path }) {
        Err(CoreError::Invalid { field, message }) => {
            assert_eq!(field, "path");
            message
        }
        other => panic!("{other:?}"),
    };
    assert_eq!(
        refusal(lib.to_string_lossy().into_owned()),
        "already a source"
    );
    assert_eq!(
        refusal(format!("{}/", lib.to_string_lossy())),
        "already a source"
    );
    assert_eq!(
        refusal(lib.join("Show A").to_string_lossy().into_owned()),
        "nested inside an existing source"
    );
    assert_eq!(
        refusal(dir.path().to_string_lossy().into_owned()),
        "nested inside an existing source"
    );

    match core.call(Call::ListSources).unwrap() {
        Reply::Sources { sources } => assert_eq!(sources.len(), 1),
        other => panic!("{other:?}"),
    }
    // A source beside the first one, sharing no tree, is still fine.
    let other = dir.path().join("other");
    fs::create_dir_all(&other).unwrap();
    assert!(matches!(
        core.call(Call::AddSource {
            path: other.to_string_lossy().into_owned()
        })
        .unwrap(),
        Reply::Source { .. }
    ));

    let job = started(core.call(Call::Scan { source: None }).unwrap());
    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::ScanFinished { .. }),
        "{done:?}"
    );
}

#[test]
fn an_unavailable_source_is_kept_and_untouched() {
    let (dir, core, c) = common::open_core();
    let gone = dir.path().join("gone");
    match core
        .call(Call::AddSource {
            path: gone.to_string_lossy().into_owned(),
        })
        .unwrap()
    {
        Reply::Source { source } => assert!(!source.available),
        other => panic!("{other:?}"),
    }
    let scan = common::wait_for(
        &c,
        |e| {
            matches!(
                e.body,
                EventBody::JobStarted {
                    kind: JobKind::Scan
                }
            )
        },
        std::time::Duration::from_secs(5),
    )
    .job
    .unwrap()
    .id;
    assert!(matches!(
        common::wait_job(&c, scan).body,
        EventBody::ScanFinished {
            added: 0,
            changed: 0,
            removed: 0,
            ..
        }
    ));
    assert!(matches!(
        core.call(Call::AddSource {
            path: "relative".into()
        }),
        Err(CoreError::Invalid { .. })
    ));
}
