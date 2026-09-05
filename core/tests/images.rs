//! The image cache end to end: a gap read starts the fill, the fill fetches
//! each url once and hands the local path back on the next read, and Clear
//! images empties both the table and the directory.

mod common;
mod fixtures;

use std::time::Duration;

use anibeam_core::*;

fn list(core: &Core) -> Vec<SeriesCard> {
    let call = Call::ListSeries {
        tab: Tab::All,
        query: String::new(),
        sort: Sort::Alpha,
        direction: Direction::Asc,
        reveal_hidden: false,
    };
    match core.call(call).unwrap() {
        Reply::Series { series } => series,
        other => panic!("{other:?}"),
    }
}

#[test]
fn ensure_fetches_once_and_clear_images_empties_everything() {
    let http = anibeam_core::net::FakeHttp::new();
    http.push_for("xl.jpg", 200, vec![0xFF, 0xD8, 0xFF, 0xE0]);
    let (dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let s = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/A", "A");
    fixtures::insert_file(&core, s, "/lib/A/01.mkv", 1.0, None, "episode", 1);
    fixtures::insert_media_with_cover(&core, 1, "https://img/xl.jpg", None);
    fixtures::match_series(&core, s, Some(1), None);

    // The first read has a gap: the media row carries a cover url that no
    // image row answers, so the card comes back without a poster and the
    // read starts the fill.
    let cards = list(&core);
    assert!(cards[0].poster.is_none());
    let fill = common::wait_for(
        &c,
        |e| {
            matches!(
                e.body,
                EventBody::JobStarted {
                    kind: JobKind::FillImages
                }
            )
        },
        Duration::from_secs(5),
    )
    .job
    .unwrap()
    .id;
    common::wait_job(&c, fill);

    let cards = list(&core);
    let poster = cards[0].poster.clone().unwrap();
    assert!(
        poster.starts_with(dir.path().join("cache").join("images").to_str().unwrap()),
        "{poster}"
    );
    assert!(std::path::Path::new(&poster).exists());
    assert_eq!(http.requests().len(), 1);
    assert!(matches!(
        core.call(Call::GetStorage).unwrap(),
        Reply::Storage {
            image_count: 1,
            image_bytes: 4
        }
    ));

    // The second read had no gap, so it started no second fill, and the
    // fetched url is never asked for twice.
    assert_eq!(http.requests().len(), 1);

    let clear = match core.call(Call::ClearImages).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    common::wait_job(&c, clear);
    assert!(!std::path::Path::new(&poster).exists());
    assert!(matches!(
        core.call(Call::GetStorage).unwrap(),
        Reply::Storage { image_count: 0, .. }
    ));
    assert!(
        c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::ImagesCleared { removed: 1 }))
    );
    // Clearing tells every matched series its poster is gone.
    assert!(c.bodies().iter().any(
        |b| matches!(b, EventBody::SeriesChanged { series } if series.iter().any(|c| c.id == s))
    ));
}

#[test]
fn a_failed_fetch_is_reported_per_url_and_the_batch_still_finishes() {
    let http = anibeam_core::net::FakeHttp::new();
    http.push_for("good.jpg", 200, vec![1, 2, 3]);
    http.push_for("bad.jpg", 404, b"nope".to_vec());
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let good = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/G", "G");
    let bad = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/B", "B");
    fixtures::insert_media_with_cover(&core, 1, "https://img/good.jpg", None);
    fixtures::insert_media_with_cover(&core, 2, "https://img/bad.jpg", None);
    fixtures::match_series(&core, good, Some(1), None);
    fixtures::match_series(&core, bad, Some(2), None);

    let _ = list(&core);
    let fill = common::wait_for(
        &c,
        |e| {
            matches!(
                e.body,
                EventBody::JobStarted {
                    kind: JobKind::FillImages
                }
            )
        },
        Duration::from_secs(5),
    )
    .job
    .unwrap()
    .id;
    let finished = common::wait_job(&c, fill);
    assert_eq!(
        finished.body,
        EventBody::Notice,
        "the fill ends in a notice, not a failure"
    );
    assert!(
        finished.message.contains("1 fetched"),
        "{}",
        finished.message
    );
    assert!(
        finished.message.contains("1 failed"),
        "{}",
        finished.message
    );

    // The one that answered is cached; the one that 404ed is not, and no
    // row was written for it.
    assert!(matches!(
        core.call(Call::GetStorage).unwrap(),
        Reply::Storage {
            image_count: 1,
            image_bytes: 3
        }
    ));
}
