//! What the disk holds, watched against not (#266), and Delete watched: every watched
//! episode, or every show confirmed done with. A show whose end is only guessed, or that
//! is still airing, is never taken as finished.

mod common;
mod fixtures;

use std::path::Path;

use anibeam_core::*;

fn cards(core: &Core) -> Vec<SeriesCard> {
    let mut out = Vec::new();
    for tab in [Tab::All, Tab::Watched, Tab::Hidden] {
        match core
            .call(Call::ListSeries {
                tab,
                query: String::new(),
                sort: Sort::Alpha,
                direction: Direction::Asc,
                reveal_hidden: true,
            })
            .unwrap()
        {
            Reply::Series { series } => out.extend(series),
            other => panic!("{other:?}"),
        }
    }
    out
}

fn card(core: &Core, id: u64) -> SeriesCard {
    cards(core)
        .into_iter()
        .find(|c| c.id == id)
        .expect("the series is listed")
}

/// A show folder under `lib` with episodes 1 to `files` on disk, `bytes` each, matched to
/// AniList `anilist` with `status` and `total`, and tracker progress `progress`.
#[allow(clippy::too_many_arguments)]
fn show(
    core: &Core,
    src: u64,
    lib: &Path,
    name: &str,
    anilist: u64,
    status: &str,
    total: Option<u32>,
    files: u32,
    progress: Option<u32>,
) -> u64 {
    let dir = lib.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let id = fixtures::insert_series(core, src, SeriesKind::Show, &dir.to_string_lossy(), name);
    fixtures::insert_media(core, anilist, Some(name), None, total, status, "TV", None);
    fixtures::match_series(core, id, Some(anilist), None);
    for n in 1..=files {
        let p = dir.join(format!("{name} - {n:02}.mkv"));
        std::fs::write(&p, vec![0u8; 100]).unwrap();
        let f = fixtures::insert_file(core, id, &p.to_string_lossy(), n as f64, None, "episode", 0);
        set_size(core, f, 100);
    }
    if let Some(w) = progress {
        fixtures::insert_tracker_entry(core, "anilist", anilist, w, "current", None);
    }
    id
}

fn set_size(core: &Core, file: u64, size: i64) {
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE files SET size = ?2 WHERE id = ?1",
                rusqlite::params![file as i64, size],
            )?;
            Ok(())
        })
        .unwrap()
}

#[test]
fn usage_counts_watched_files_and_only_a_confirmed_end_is_finished() {
    let (dir, core, _c) = common::open_core();
    let lib = dir.path().join("lib");
    let src = fixtures::insert_source(&core, &lib.to_string_lossy());

    // Every episode tracked as watched, and the provider says it finished
    let done = show(&core, src, &lib, "Done", 1, "FINISHED", Some(3), 3, Some(3));
    // One tracked, one played to the end here, one not watched
    let part = show(&core, src, &lib, "Part", 2, "FINISHED", Some(3), 3, Some(1));
    fixtures::insert_completed(&core, part, "2", 0);
    // Watched as far as it goes, but still airing
    let airing = show(&core, src, &lib, "Airing", 3, "RELEASING", None, 2, Some(2));
    // Finished, but no published count: its end is a guess
    let guess = show(&core, src, &lib, "Guess", 4, "FINISHED", None, 2, Some(2));

    let c = card(&core, done);
    assert_eq!(
        (c.bytes_on_disk, c.watched_bytes, c.watched_on_disk),
        (300, 300, 3)
    );
    assert!(c.finished_watched);

    let c = card(&core, part);
    assert_eq!(
        (c.bytes_on_disk, c.watched_bytes, c.watched_on_disk),
        (300, 200, 2)
    );
    assert!(!c.finished_watched);

    let c = card(&core, airing);
    assert_eq!(c.watched_bytes, 200);
    assert!(!c.finished_watched, "a show still airing is not finished");

    let c = card(&core, guess);
    assert_eq!(c.watched_bytes, 200);
    assert!(
        !c.finished_watched,
        "a show with no published count is not finished"
    );
}

#[test]
fn delete_watched_takes_watched_episodes_or_finished_shows_only() {
    let (dir, core, _c) = common::open_core();
    // The trash lands in the test's own data home
    unsafe { std::env::set_var("XDG_DATA_HOME", dir.path().join("share")) };
    let lib = dir.path().join("lib");
    let src = fixtures::insert_source(&core, &lib.to_string_lossy());
    let done = show(&core, src, &lib, "Done", 1, "FINISHED", Some(2), 2, Some(2));
    let part = show(&core, src, &lib, "Part", 2, "FINISHED", Some(3), 3, Some(1));
    let ep = |name: &str, n: u32| lib.join(name).join(format!("{name} - {n:02}.mkv"));

    match core
        .call(Call::DeleteWatched {
            finished_shows: true,
        })
        .unwrap()
    {
        Reply::Deleted {
            series,
            episodes,
            bytes,
        } => assert_eq!((series, episodes, bytes), (1, 2, 200)),
        other => panic!("{other:?}"),
    }
    assert!(!ep("Done", 1).exists() && !ep("Done", 2).exists());
    assert!(ep("Part", 1).exists(), "a show not finished stays whole");

    match core
        .call(Call::DeleteWatched {
            finished_shows: false,
        })
        .unwrap()
    {
        Reply::Deleted { episodes, .. } => assert_eq!(episodes, 1),
        other => panic!("{other:?}"),
    }
    assert!(!ep("Part", 1).exists());
    assert!(
        ep("Part", 2).exists() && ep("Part", 3).exists(),
        "unwatched episodes stay"
    );
    let _ = (done, part);
    core.shutdown();
}
