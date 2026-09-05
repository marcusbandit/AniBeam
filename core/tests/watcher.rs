mod common;
use anibeam_core::*;
use std::fs;
use std::io::Write;
use std::time::Duration;

/// The watcher end to end on inotify: a finished write is ingested, a folder
/// that appears is walked, the settle timer fires for a series that has
/// never been matched, and a deleted file takes its episode away again.
#[cfg(target_os = "linux")]
#[test]
fn a_file_landing_in_a_watched_source_is_ingested_after_close_write() {
    let (dir, core, c) = common::open_core();
    let lib = dir.path().join("lib");
    fs::create_dir_all(lib.join("Show")).unwrap();
    fs::write(lib.join("Show").join("Show - 01.mkv"), b"x").unwrap();
    core.call(Call::AddSource { path: lib.to_string_lossy().into_owned() }).unwrap();
    core.start().unwrap();
    common::wait_for(&c, |e| matches!(e.body, EventBody::ScanFinished { .. }), Duration::from_secs(10));

    // Written in two goes with a pause between them: only the close, not
    // either write, may be taken as the file being ready.
    {
        let mut f = fs::File::create(lib.join("Show").join("Show - 02.mkv")).unwrap();
        f.write_all(b"partial").unwrap();
        std::thread::sleep(Duration::from_millis(50));
        f.write_all(b"more").unwrap();
    }
    let changed = common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SeriesChanged { series } if series.iter().any(|s| s.title == "Show" && s.episodes_on_disk == 2)),
        Duration::from_secs(10),
    );
    assert!(changed.job.is_some());

    // A whole folder appears at once: the file inside it may well land
    // before notify has a watch on the new directory, so the directory
    // itself is what gets walked.
    fs::create_dir_all(lib.join("Other")).unwrap();
    fs::write(lib.join("Other").join("Other - 01.mkv"), b"x").unwrap();
    common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SeriesChanged { series } if series.iter().any(|s| s.title == "Other")),
        Duration::from_secs(10),
    );
    common::wait_for(&c, |e| e.message.starts_with("folder settled"), Duration::from_secs(10));

    fs::remove_file(lib.join("Show").join("Show - 02.mkv")).unwrap();
    common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SeriesChanged { series } if series.iter().any(|s| s.title == "Show" && s.episodes_on_disk == 1)),
        Duration::from_secs(10),
    );

    // A folder that goes away under a watched source goes missing on the
    // scan the watcher starts, not only on the next full one. The card
    // leaves in the same batch every other change does, carrying
    // `missing`, so a grid patched from events can drop it.
    fs::remove_dir_all(lib.join("Other")).unwrap();
    common::wait_for(
        &c,
        |e| {
            matches!(&e.body, EventBody::SeriesChanged { series }
                if series.iter().any(|s| s.title == "Other" && s.missing && s.episodes_on_disk == 0))
        },
        Duration::from_secs(10),
    );
    let listed = match core.call(Call::ListSeries { tab: Tab::All, query: String::new(), sort: Sort::Alpha, direction: Direction::Asc, reveal_hidden: false }).unwrap() {
        Reply::Series { series } => series,
        other => panic!("{other:?}"),
    };
    assert!(!listed.iter().any(|s| s.title == "Other"), "{listed:?}");
    let missing = match core.call(Call::ListMetadata { filter: MetadataFilter::MissingFiles, query: String::new(), reveal_hidden: false }).unwrap() {
        Reply::Metadata { rows, .. } => rows,
        other => panic!("{other:?}"),
    };
    assert!(missing.iter().any(|r| r.series.title == "Other" && r.series.missing), "{missing:?}");
    core.shutdown();
}
