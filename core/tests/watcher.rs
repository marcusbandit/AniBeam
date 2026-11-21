#![cfg(any(target_os = "linux", target_os = "macos"))]

mod common;
use anibeam_core::*;
use std::fs;
use std::io::Write;
use std::time::Duration;

/// The watcher end to end on inotify, and on FSEvents: a finished write is
/// ingested, a folder that appears is walked, the settle timer fires for a
/// series that has never been matched and hands it to the auto-match, and a
/// deleted file takes its episode away again. On a Mac "finished" means the
/// size held still, since FSEvents has no close-write.
///
/// The network is canned, because the settle timer's whole point is that it
/// ends in a search. Every reply is an empty page, so every series the
/// walks produce misses and is stamped, which is the quietest outcome the
/// chain has: nothing is fetched and nothing is written.
#[test]
fn a_file_landing_in_a_watched_source_is_ingested_once_written() {
    let http = anibeam_core::net::FakeHttp::new();
    for _ in 0..6 {
        http.push_for(
            "graphql.anilist.co",
            200,
            serde_json::json!({ "data": { "Page": { "media": [] } } }).to_string(),
        );
    }
    let (dir, core, c) = common::open_core_with_http(http);
    // Canonical, because a Mac's temp directory sits behind the /var ->
    // /private/var link and FSEvents reports the real path.
    let lib = dir.path().canonicalize().unwrap().join("lib");
    fs::create_dir_all(lib.join("Show")).unwrap();
    fs::write(lib.join("Show").join("Show - 01.mkv"), b"x").unwrap();
    core.call(Call::AddSource {
        path: lib.to_string_lossy().into_owned(),
    })
    .unwrap();
    core.start().unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ScanFinished { .. }),
        Duration::from_secs(10),
    );

    // Written in two goes with a pause between them: only the close (or on
    // a Mac the size holding still), not either write, may be taken as the
    // file being ready.
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
    // The settle timer's end of the chain: a folder that has stopped
    // changing and has never been matched is handed to the auto-match.
    common::wait_for(
        &c,
        |e| {
            matches!(
                &e.body,
                EventBody::JobStarted {
                    kind: JobKind::AutoMatch
                }
            )
        },
        Duration::from_secs(10),
    );

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
    let listed = common::cards(&core, Tab::All, false);
    assert!(!listed.iter().any(|s| s.title == "Other"), "{listed:?}");
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
    assert!(
        missing
            .iter()
            .any(|r| r.series.title == "Other" && r.series.missing),
        "{missing:?}"
    );
    core.shutdown();
}

/// Whether this event says the source at `path` turned `available`.
fn source_turned(e: &Event, path: &str, available: bool) -> bool {
    matches!(&e.body, EventBody::SourceChanged { source }
        if source.path == path && source.available == available)
}

/// A source whose folder is missing at launch (a drive not plugged in) is
/// watched from the nearest directory above it that exists, so when the
/// folder appears it is marked available, scanned and watched with no
/// restart, and a download landing in it afterwards is taken in (#1191).
/// The reverse too: the folder going away while running makes the source
/// unavailable rather than every series in it missing.
#[test]
fn a_source_that_comes_back_is_scanned_and_watched_again() {
    // Canned, as above: the new show is handed to the auto-match, and the
    // launch jobs must not reach the real network.
    let http = anibeam_core::net::FakeHttp::new();
    for _ in 0..6 {
        http.push_for(
            "graphql.anilist.co",
            200,
            serde_json::json!({ "data": { "Page": { "media": [] } } }).to_string(),
        );
    }
    let (dir, core, c) = common::open_core_with_http(http);
    // Canonical for the same reason as above: FSEvents reports real paths.
    let drive = dir.path().canonicalize().unwrap().join("drive");
    let lib = drive.join("Anime");
    let path = lib.to_string_lossy().into_owned();
    core.call(Call::AddSource { path: path.clone() }).unwrap();
    core.start().unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ScanFinished { .. }),
        Duration::from_secs(10),
    );
    match core.call(Call::ListSources).unwrap() {
        Reply::Sources { sources } => {
            assert!(sources.iter().all(|s| !s.available), "{sources:?}")
        }
        other => panic!("{other:?}"),
    }

    // The drive is plugged in: the source and a show on it appear at once.
    fs::create_dir_all(lib.join("Show")).unwrap();
    fs::write(lib.join("Show").join("Show - 01.mkv"), b"x").unwrap();
    common::wait_for(
        &c,
        |e| source_turned(e, &path, true),
        Duration::from_secs(10),
    );
    common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SeriesChanged { series } if series.iter().any(|s| s.title == "Show" && s.episodes_on_disk == 1)),
        Duration::from_secs(10),
    );

    // The source is watched again: a download that lands now is ingested.
    {
        let mut f = fs::File::create(lib.join("Show").join("Show - 02.mkv")).unwrap();
        f.write_all(b"x").unwrap();
    }
    common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SeriesChanged { series } if series.iter().any(|s| s.title == "Show" && s.episodes_on_disk == 2)),
        Duration::from_secs(10),
    );

    // The drive is unplugged: the source goes unavailable, its show stays.
    // Linux only: inotify reports a folder deleted under a watch, while
    // FSEvents stays silent about a watched folder deleted with its parent.
    // A Mac sees an unmount through the watch on the mount point's parent,
    // which a temp directory on the boot volume has nothing to stand for.
    #[cfg(target_os = "linux")]
    gone_and_back(&c, &drive, &lib, &path);
    core.shutdown();
}

/// The second half of the test above: unplugged, then plugged in again.
#[cfg(target_os = "linux")]
fn gone_and_back(
    c: &anibeam_core::events::Collector,
    drive: &std::path::Path,
    lib: &std::path::Path,
    path: &str,
) {
    // `AddSource` already said it was unavailable once, so it is the next
    // such event that counts.
    let gone_before = c
        .events()
        .iter()
        .filter(|e| source_turned(e, path, false))
        .count();
    fs::remove_dir_all(drive).unwrap();
    let gone = c.wait_for(
        |events| {
            events
                .iter()
                .filter(|e| source_turned(e, path, false))
                .count()
                > gone_before
        },
        Duration::from_secs(10),
    );
    assert!(gone, "the source never went unavailable");
    assert!(
        !c.events().iter().any(|e| matches!(&e.body, EventBody::SeriesChanged { series } if series.iter().any(|s| s.title == "Show" && s.missing))),
        "the show read as missing instead of its source as unavailable"
    );

    // And plugged in again, with what it held.
    fs::create_dir_all(lib.join("Show")).unwrap();
    fs::write(lib.join("Show").join("Show - 01.mkv"), b"x").unwrap();
    let arrived = c.wait_for(
        |events| {
            events
                .iter()
                .filter(|e| source_turned(e, path, true))
                .count()
                == 2
        },
        Duration::from_secs(10),
    );
    assert!(arrived, "the source never came back a second time");
}
