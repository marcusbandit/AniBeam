//! Episode out (#212): the core says once when an episode of a followed show
//! airs, from the schedule it already keeps and with no polling.

mod common;
mod fixtures;

use std::time::Duration;

use anibeam_core::*;

fn outs(c: &anibeam_core::events::Collector) -> Vec<EpisodeOut> {
    c.events()
        .into_iter()
        .filter_map(|e| match e.body {
            EventBody::EpisodeOut { episode } => Some(episode),
            _ => None,
        })
        .collect()
}

/// A followed show in the library, one on AniList only, and one the user
/// is not watching, each with an episode due a couple of seconds from now.
fn shows(core: &Core, at: i64) -> u64 {
    let src = fixtures::insert_source(core, "/lib");
    let frieren = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Frieren", "Frieren");
    fixtures::insert_file(
        core,
        frieren,
        "/lib/Frieren/01.mkv",
        1.0,
        None,
        "episode",
        1,
    );
    fixtures::insert_media(
        core,
        1,
        Some("Frieren"),
        None,
        Some(28),
        "RELEASING",
        "TV",
        Some(90),
    );
    fixtures::match_series(core, frieren, Some(1), None);
    fixtures::insert_media(
        core,
        2,
        Some("Dandadan"),
        None,
        Some(12),
        "RELEASING",
        "TV",
        Some(85),
    );
    fixtures::insert_media(
        core,
        3,
        Some("Not Followed"),
        None,
        Some(12),
        "RELEASING",
        "TV",
        Some(70),
    );
    fixtures::insert_tracker_entry(core, "anilist", 1, 6, "watching", None);
    fixtures::insert_tracker_entry(core, "anilist", 2, 3, "watching", None);
    fixtures::insert_tracker_entry(core, "anilist", 3, 0, "planning", None);
    fixtures::insert_airing(core, 1, 7, at);
    fixtures::insert_airing(core, 2, 4, at);
    fixtures::insert_airing(core, 3, 1, at);
    frieren
}

#[test]
fn an_episode_of_a_followed_show_is_said_once_when_it_airs() {
    let (_dir, core, c) = common::open_core();
    let now = anibeam_core::time::now_secs();
    let frieren = shows(&core, now + 2);
    core.start().unwrap();

    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::EpisodeOut { .. }),
        Duration::from_secs(15),
    );
    // Both followed shows aired at the same moment; give the second its turn.
    let both = c.wait_for(
        |events| {
            events
                .iter()
                .filter(|e| matches!(e.body, EventBody::EpisodeOut { .. }))
                .count()
                >= 2
        },
        Duration::from_secs(5),
    );
    assert!(both, "{:#?}", c.events());
    let said = outs(&c);
    assert_eq!(
        said.len(),
        2,
        "the show only planned stays quiet: {said:#?}"
    );
    let first = said.iter().find(|o| o.anilist_id == 1).unwrap();
    assert_eq!(first.episode, 7);
    assert_eq!(first.title, "Frieren");
    assert_eq!(
        first.series,
        Some(frieren),
        "on disk, so it opens the series"
    );
    let other = said.iter().find(|o| o.anilist_id == 2).unwrap();
    assert_eq!(other.series, None, "not in the library");
    assert!(
        c.events()
            .iter()
            .any(|e| e.plain.as_deref() == Some("Episode 7 of Frieren is out.")),
        "{:#?}",
        c.events()
    );

    // A change that re-arms the alarm says nothing new.
    core.call(Call::SetHidden {
        series: frieren,
        hidden: false,
    })
    .unwrap();
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(outs(&c).len(), 2);
}

#[test]
fn the_first_launch_says_nothing_about_episodes_already_out() {
    let (_dir, core, c) = common::open_core();
    let now = anibeam_core::time::now_secs();
    shows(&core, now - 600);
    core.start().unwrap();
    core.call(Call::GetPreferences).unwrap();
    std::thread::sleep(Duration::from_millis(1500));
    assert!(outs(&c).is_empty(), "{:#?}", outs(&c));
}

#[test]
fn a_hidden_show_is_not_followed() {
    let (_dir, core, c) = common::open_core();
    let now = anibeam_core::time::now_secs();
    let frieren = shows(&core, now + 2);
    core.call(Call::SetHidden {
        series: frieren,
        hidden: true,
    })
    .unwrap();
    core.start().unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::EpisodeOut { .. }),
        Duration::from_secs(15),
    );
    std::thread::sleep(Duration::from_millis(500));
    let ids: Vec<u64> = outs(&c).iter().map(|o| o.anilist_id).collect();
    assert_eq!(ids, vec![2]);
}
