mod common;
mod fixtures;
use anibeam_core::*;

#[test]
fn feed_recent_and_upcoming() {
    let (dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    let now = anibeam_core::time::now_secs();

    // (a) Downloaded: eight files on disk, episodes 1 to 7 have a past air
    // date, episode 8 has none, and file 8 carries the newest mtime.
    let a = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/A", "A");
    for n in 1..=8 {
        fixtures::insert_file(&core, a, &format!("/lib/A/{n:02}.mkv"), n as f64, None, "episode", now - 100 + n * 10);
    }
    fixtures::insert_media(&core, 1001, Some("A"), None, None, "RELEASING", "TV", None);
    fixtures::match_series(&core, a, Some(1001), None);
    for n in 1..=7 {
        fixtures::insert_airing(&core, 1001, n, now - 200_000 - n * 100);
    }

    // (b) Aired: the highest on-disk episode, 5, itself has a past air date.
    let b = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/B", "B");
    for n in 1..=5 {
        fixtures::insert_file(&core, b, &format!("/lib/B/{n:02}.mkv"), n as f64, None, "episode", now - 5000);
    }
    fixtures::insert_media(&core, 1002, Some("B"), None, None, "RELEASING", "TV", None);
    fixtures::match_series(&core, b, Some(1002), None);
    for n in 1..=4 {
        fixtures::insert_airing(&core, 1002, n, now - 300_000 - n * 100);
    }
    fixtures::insert_airing(&core, 1002, 5, now - 500);

    // (c) Upcoming: episode 9 is scheduled in the future, with 8 already on disk.
    let c = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/C", "C");
    for n in 1..=8 {
        fixtures::insert_file(&core, c, &format!("/lib/C/{n:02}.mkv"), n as f64, None, "episode", now - 2000);
    }
    fixtures::insert_media(&core, 1003, Some("C"), None, None, "RELEASING", "TV", None);
    fixtures::match_series(&core, c, Some(1003), None);
    fixtures::insert_airing(&core, 1003, 9, now + 86_400);

    // (d) A film: always Downloaded from its own newest mtime, unmatched.
    let d = fixtures::insert_series(&core, src, SeriesKind::Movie, "/lib/Movies/D.mkv", "D");
    fixtures::insert_file(&core, d, "/lib/Movies/D.mkv", 1.0, None, "episode", now - 50_000);

    // (e) A hidden and a missing series: both absent from every feed.
    let hidden = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Hidden", "Hidden");
    fixtures::insert_file(&core, hidden, "/lib/Hidden/01.mkv", 1.0, None, "episode", now);
    core.call(Call::SetHidden { series: hidden, hidden: true }).unwrap();
    let missing = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Missing", "Missing");
    fixtures::insert_file(&core, missing, "/lib/Missing/01.mkv", 1.0, None, "episode", now);
    fixtures::mark_missing(&core, missing);

    let recent = match core.call(Call::ListFeed { sort: FeedSort::Recent }).unwrap() {
        Reply::Feed { cards } => cards,
        other => panic!("{other:?}"),
    };
    // Newest first: A's file mtime, then B's own air date, then C's file
    // mtime, then D's much older file mtime.
    assert_eq!(recent.iter().map(|c| c.series.id).collect::<Vec<_>>(), vec![a, b, c, d]);

    assert!(matches!(recent[0].reason, FeedReason::Downloaded { .. }));
    assert_eq!(recent[0].highest_on_disk, Some(8.0));

    match recent[1].reason {
        FeedReason::Aired { episode, .. } => assert_eq!(episode, 5),
        ref other => panic!("{other:?}"),
    }

    assert!(matches!(recent[3].reason, FeedReason::Downloaded { .. }));
    assert!(!recent.iter().any(|c| c.series.id == hidden || c.series.id == missing));

    let upcoming = match core.call(Call::ListFeed { sort: FeedSort::Upcoming }).unwrap() {
        Reply::Feed { cards } => cards,
        other => panic!("{other:?}"),
    };
    // C floats to the top with its scheduled episode 9, badged with what is
    // already on disk; the rest keep their Recent order behind it.
    assert_eq!(upcoming[0].series.id, c);
    match upcoming[0].reason {
        FeedReason::Scheduled { episode, .. } => assert_eq!(episode, 9),
        ref other => panic!("{other:?}"),
    }
    assert_eq!(upcoming[0].highest_on_disk, Some(8.0));
    assert_eq!(upcoming.iter().skip(1).map(|c| c.series.id).collect::<Vec<_>>(), vec![a, b, d]);
    assert!(!upcoming.iter().any(|c| c.series.id == hidden || c.series.id == missing));

    drop(dir);
}

/// Two series that land on the exact same instant, the way a batch import
/// often does (every file it writes lands in the same second), must break
/// the tie the same way every time: ascending series id, never whichever
/// order a hasher's iteration happened to produce. Checked across two
/// consecutive calls, both for the plain `Recent` sort and for the "rest"
/// tail behind `Upcoming`'s scheduled group.
#[test]
fn feed_ties_break_by_ascending_series_id() {
    let (dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    let now = anibeam_core::time::now_secs();

    let e = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/E", "E");
    fixtures::insert_file(&core, e, "/lib/E/01.mkv", 1.0, None, "episode", now - 10);
    let f = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/F", "F");
    fixtures::insert_file(&core, f, "/lib/F/01.mkv", 1.0, None, "episode", now - 10);

    for _ in 0..2 {
        let recent = match core.call(Call::ListFeed { sort: FeedSort::Recent }).unwrap() {
            Reply::Feed { cards } => cards,
            other => panic!("{other:?}"),
        };
        assert_eq!(recent.iter().map(|c| c.series.id).collect::<Vec<_>>(), vec![e, f]);

        let upcoming = match core.call(Call::ListFeed { sort: FeedSort::Upcoming }).unwrap() {
            Reply::Feed { cards } => cards,
            other => panic!("{other:?}"),
        };
        // Neither series has a scheduled episode, so both fall into the
        // rest tail; the same ascending-id tiebreak applies there.
        assert_eq!(upcoming.iter().map(|c| c.series.id).collect::<Vec<_>>(), vec![e, f]);
    }

    drop(dir);
}
