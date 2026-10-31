//! An episode count set by hand (#239): it stands in while the record names
//! none, stops the schedule and the guesses past it, gives way to an episode
//! that really aired or landed past it, and is cleared by a rematch.

mod common;
mod fixtures;

use anibeam_core::*;

const DAY: i64 = 86_400;

fn card(core: &Core, series: u64) -> SeriesCard {
    match core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: true,
        })
        .unwrap()
    {
        Reply::Series { series: cards } => cards
            .into_iter()
            .find(|c| c.id == series)
            .expect("the series is listed"),
        other => panic!("{other:?}"),
    }
}

/// A releasing show with no published count: episodes 1 to `aired` out and
/// on disk, and the schedule naming more up to `scheduled`, one a week.
fn airing_show(core: &Core, anilist: u64, aired: i64, scheduled: i64) -> u64 {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let id = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Show", "Show");
    fixtures::insert_media(
        core,
        anilist,
        Some("Show"),
        None,
        None,
        "RELEASING",
        "TV",
        None,
    );
    fixtures::match_series(core, id, Some(anilist), None);
    for n in 1..=scheduled {
        fixtures::insert_airing(core, anilist, n, now + (n - aired) * 7 * DAY - DAY);
        if n <= aired {
            fixtures::insert_file(
                core,
                id,
                &format!("/lib/Show/{n:02}.mkv"),
                n as f64,
                None,
                "episode",
                now,
            );
        }
    }
    id
}

fn set(core: &Core, series: u64, count: Option<u32>) {
    core.call(Call::SetEpisodeCount { series, count }).unwrap();
}

#[test]
fn a_set_count_caps_the_total_and_the_schedule() {
    let (_dir, core, _c) = common::open_core();
    let id = airing_show(&core, 1, 5, 12);
    assert_eq!(card(&core, id).next_airing.map(|a| a.episode), Some(6));

    set(&core, id, Some(5));
    let c = card(&core, id);
    assert_eq!(c.episodes_set, Some(5));
    assert!(c.episodes_set_applied);
    assert_eq!(c.total_episodes, Some(5));
    assert!(!c.total_is_estimate);
    // Episode 6 is only planned, so it neither shows nor overrides the count
    assert_eq!(c.next_airing, None);

    set(&core, id, None);
    let c = card(&core, id);
    assert_eq!(c.episodes_set, None);
    assert_eq!(c.next_airing.map(|a| a.episode), Some(6));
}

#[test]
fn an_episode_past_the_count_that_aired_or_landed_wins() {
    let (_dir, core, _c) = common::open_core();
    let id = airing_show(&core, 2, 5, 12);
    set(&core, id, Some(4));
    let c = card(&core, id);
    assert_eq!(c.episodes_set, Some(4));
    assert!(!c.episodes_set_applied, "episode 5 aired and is on disk");
    assert_eq!(c.next_airing.map(|a| a.episode), Some(6));
}

#[test]
fn a_record_with_its_own_count_wins() {
    let (_dir, core, _c) = common::open_core();
    let id = airing_show(&core, 3, 5, 12);
    set(&core, id, Some(13));
    fixtures::insert_media(
        &core,
        3,
        Some("Show"),
        None,
        Some(12),
        "RELEASING",
        "TV",
        None,
    );
    let c = card(&core, id);
    assert!(!c.episodes_set_applied);
    assert_eq!(c.total_episodes, Some(12));
}

#[test]
fn an_unknown_series_is_not_found() {
    let (_dir, core, _c) = common::open_core();
    let err = core
        .call(Call::SetEpisodeCount {
            series: 999,
            count: Some(3),
        })
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound { .. }), "{err:?}");
}
