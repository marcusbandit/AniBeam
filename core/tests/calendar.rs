//! The calendar feed: with the switch on, `<data_dir>/anibeam.ics` holds the
//! airing times of the shows on the user's lists, and follows the schedule.

mod common;
mod fixtures;
use std::time::Duration;

use anibeam_core::*;

/// A releasing show with a file on disk, matched to `anilist_id` and on the
/// AniList list with `status`, if any.
fn show(core: &Core, source: u64, name: &str, anilist_id: u64, status: Option<&str>) -> u64 {
    let series = fixtures::insert_series(
        core,
        source,
        SeriesKind::Show,
        &format!("/lib/{name}"),
        name,
    );
    fixtures::insert_file(
        core,
        series,
        &format!("/lib/{name}/01.mkv"),
        1.0,
        None,
        "episode",
        1,
    );
    fixtures::insert_media(
        core,
        anilist_id,
        Some(name),
        None,
        Some(12),
        "RELEASING",
        "TV",
        Some(80),
    );
    fixtures::match_series(core, series, Some(anilist_id), None);
    if let Some(status) = status {
        fixtures::insert_tracker_entry(core, "anilist", anilist_id, 1, status, None);
    }
    series
}

fn set_feed(core: &Core, on: bool) {
    core.call(Call::SetPreferences {
        preferences: Preferences {
            calendar_feed: on,
            ..Preferences::default()
        },
    })
    .unwrap();
}

fn utc(secs: i64) -> String {
    let t = ::time::OffsetDateTime::from_unix_timestamp(secs).unwrap();
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        t.year(),
        u8::from(t.month()),
        t.day(),
        t.hour(),
        t.minute(),
        t.second()
    )
}

#[test]
fn the_feed_follows_the_switch_the_lists_and_the_schedule() {
    let http = anibeam_core::net::FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    let path = CorePaths::under(dir.path()).calendar_path();
    let src = fixtures::insert_source(&core, "/lib");
    let now = time::now_secs();

    let watching = show(&core, src, "Sousou no Frieren", 1, Some("watching"));
    show(&core, src, "Dropped Show", 2, Some("dropped"));
    show(&core, src, "Unlisted Show", 3, None);
    fixtures::insert_airing(&core, 1, 1, now - 10 * 86_400);
    let next_at = now + 86_400;
    fixtures::insert_airing(&core, 1, 2, next_at);
    fixtures::insert_airing(&core, 2, 5, now + 86_400);
    fixtures::insert_airing(&core, 3, 5, now + 86_400);

    // Off by default: nothing on disk.
    assert!(!path.exists());

    // On: the file is there at once, with the listed show's coming episode.
    set_feed(&core, true);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("BEGIN:VCALENDAR\r\n"));
    assert!(!text.replace("\r\n", "").contains('\n'));
    assert!(text.contains(&format!("DTSTART:{}\r\n", utc(next_at))));
    assert!(text.contains("SUMMARY:Sousou no Frieren - Episode 2\r\n"));
    assert!(text.contains(&format!("UID:anibeam-{watching}-2@anibeam\r\n")));
    // Long gone, dropped, and on no list: none of them.
    assert!(!text.contains("Episode 1\r\n"));
    assert!(!text.contains("Dropped Show"));
    assert!(!text.contains("Unlisted Show"));
    assert_eq!(text.matches("BEGIN:VEVENT").count(), 1);

    // The schedule moves: a refresh brings episode 3, and the file follows.
    let third_at = now + 8 * 86_400;
    let schedule = serde_json::json!({ "data": { "Media": {
        "id": 1,
        "nextAiringEpisode": { "episode": 3, "airingAt": third_at },
        "airingSchedule": { "nodes": [
            { "episode": 2, "airingAt": next_at },
            { "episode": 3, "airingAt": third_at },
        ] },
    } } });
    http.push_for("anilist", 200, schedule.to_string());
    let job = match core.call(Call::RefreshAiring { series: watching }).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    let done = common::wait_job(&c, job);
    common::wait_for(
        &c,
        |e| e.seq > done.seq && e.message == "calendar written",
        Duration::from_secs(30),
    );
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("SUMMARY:Sousou no Frieren - Episode 3\r\n"));
    assert!(text.contains(&format!("DTSTART:{}\r\n", utc(third_at))));

    // Off: the file goes.
    set_feed(&core, false);
    assert!(!path.exists());
}
