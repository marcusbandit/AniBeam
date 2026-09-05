//! The airing refresh: the cheap fetch that keeps a releasing series'
//! next broadcast right without asking for the whole record again.

mod common;
mod fixtures;
use std::time::Duration;

use anibeam_core::*;

/// AniList's schedule reply: the paginated page of nodes and the always
/// present next broadcast.
fn schedule_json(id: u64, next: Option<(u32, i64)>, nodes: &[(u32, i64)]) -> serde_json::Value {
    let nodes: Vec<serde_json::Value> = nodes.iter().map(|(e, at)| serde_json::json!({ "episode": e, "airingAt": at })).collect();
    let next = match next {
        Some((e, at)) => serde_json::json!({ "episode": e, "airingAt": at }),
        None => serde_json::Value::Null,
    };
    serde_json::json!({ "data": { "Media": { "id": id, "nextAiringEpisode": next, "airingSchedule": { "nodes": nodes } } } })
}

/// Jikan's episode list, which is where a title would come from if this
/// job were allowed to write one.
fn jikan_json(episodes: &[(u32, &str)]) -> String {
    let data: Vec<serde_json::Value> = episodes
        .iter()
        .map(|(n, t)| serde_json::json!({ "mal_id": n, "episode": n, "title": t, "aired": serde_json::Value::Null }))
        .collect();
    serde_json::json!({ "data": data }).to_string()
}

/// A releasing series with a file on disk and a match: the one shape the
/// airing refresh exists for.
fn releasing(core: &Core, source: u64, name: &str, anilist_id: u64, mal_id: u64) -> u64 {
    matched(core, source, name, anilist_id, mal_id, "RELEASING")
}

fn matched(core: &Core, source: u64, name: &str, anilist_id: u64, mal_id: u64, status: &str) -> u64 {
    let series = fixtures::insert_series(core, source, SeriesKind::Show, &format!("/lib/{name}"), name);
    fixtures::insert_file(core, series, &format!("/lib/{name}/01.mkv"), 1.0, None, "episode", 1);
    fixtures::insert_media(core, anilist_id, Some(name), None, Some(12), status, "TV", Some(80));
    fixtures::match_series(core, series, Some(anilist_id), Some(mal_id));
    series
}

/// One stored episode row: the number, the title and the date.
type Row = (i64, Option<String>, Option<i64>);

/// The stored rows for one media id, in episode order.
fn episodes(core: &Core, anilist_id: u64) -> Vec<Row> {
    core.store()
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT number, title, aired_at FROM anilist_episodes WHERE anilist_id = ?1 ORDER BY number")?;
            let rows = stmt.query_map([anilist_id as i64], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .unwrap()
}

fn started(core: &Core, call: Call) -> u64 {
    match core.call(call).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

/// The whole point of the job: the next broadcast lands, the dates come
/// fresher than the stored ones, a title already written stays exactly as
/// it was, and a future row the provider has dropped goes.
#[test]
fn a_refresh_writes_the_next_broadcast_and_keeps_the_stored_titles() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = releasing(&core, src, "Sousou no Frieren", 1, 1001);

    // What a full metadata fetch left behind: a titled episode 1, and a
    // scheduled episode 10 that has since been dropped from the schedule.
    let now = time::now_secs();
    fixtures::insert_episode(&core, 1, 1, Some("The Journey's End"), Some(now - 9 * 86_400));
    fixtures::insert_episode(&core, 1, 10, None, Some(now + 30 * 86_400));

    let next_at = now + 86_400;
    let nodes: Vec<(u32, i64)> = (1..=8).map(|n| (n, now - i64::from(9 - n) * 86_400)).collect();
    http.push_for("anilist", 200, schedule_json(1, Some((9, next_at)), &nodes).to_string());
    http.push_for("jikan.moe", 200, jikan_json(&[(1, "A title this job may not write"), (2, "Nor this one")]));

    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(done.body, EventBody::AiringRefreshed { series, updated: true });
    assert_eq!(done.level, Level::Debug);
    assert_eq!(http.requests().len(), 2);

    let rows = episodes(&core, 1);
    assert_eq!(rows.iter().map(|(n, _, _)| *n).collect::<Vec<_>>(), (1..=9).collect::<Vec<i64>>());
    // The stored title survives: the schedule carries none and Jikan's is
    // not allowed to replace one that is already there.
    assert_eq!(rows[0].1.as_deref(), Some("The Journey's End"));
    // Episode 2 had no title stored, so Jikan's fills the gap.
    assert_eq!(rows[1].1.as_deref(), Some("Nor this one"));
    // The next broadcast, which is the whole reason the job exists.
    assert_eq!(rows[8], (9, None, Some(next_at)));

    // The card behind the series was reported before the terminal event.
    let seen = common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SeriesChanged { series: cards } if cards.iter().any(|card| card.id == series)),
        Duration::from_secs(5),
    );
    assert!(matches!(seen.body, EventBody::SeriesChanged { .. }));

    core.shutdown();
}

/// Six hours is the window. Inside it the job ends at once and asks
/// nothing; past it the same series is a candidate again.
#[test]
fn a_second_refresh_inside_the_window_asks_nothing() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = releasing(&core, src, "Sousou no Frieren", 1, 1001);
    let now = time::now_secs();
    fixtures::set_airing_refreshed_at(&core, 1, Some(now));

    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(done.body, EventBody::AiringRefreshed { series, updated: false });
    assert!(http.requests().is_empty());

    // Six hours and a minute later the row is worth asking about again.
    fixtures::set_airing_refreshed_at(&core, 1, Some(now - 6 * 3_600 - 60));
    http.push_for("anilist", 200, schedule_json(1, Some((9, now + 86_400)), &[]).to_string());
    http.push_for("jikan.moe", 200, jikan_json(&[]));
    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(done.body, EventBody::AiringRefreshed { series, updated: true });
    assert_eq!(http.requests().len(), 2);

    // An unknown series is not a job at all.
    let err = core.call(Call::RefreshAiring { series: 9_999 }).err().unwrap();
    assert!(matches!(err, CoreError::NotFound { what: Entity::Series, id: 9_999 }), "{err:?}");

    core.shutdown();
}

/// A finished series has no next episode to find, so it never becomes a
/// candidate however long ago it was refreshed.
#[test]
fn a_finished_series_is_never_a_candidate() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = matched(&core, src, "Perfect Blue", 2, 1002, "FINISHED");

    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(done.body, EventBody::AiringRefreshed { series, updated: false });
    assert!(http.requests().is_empty());

    core.shutdown();
}

/// Jikan is the title side-fetch and nothing else, so an outage costs a
/// series its titles rather than its schedule. Two series failing inside
/// the ten minute window is one line in the log, not two.
#[test]
fn two_jikan_failures_inside_the_window_write_one_warning() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let first = releasing(&core, src, "Sousou no Frieren", 1, 1001);
    let second = releasing(&core, src, "Dungeon Meshi", 2, 1002);

    let now = time::now_secs();
    for id in [1, 2] {
        http.push_for("anilist", 200, schedule_json(id, Some((9, now + 86_400)), &[]).to_string());
        http.push_for("jikan.moe", 504, "gateway timeout");
    }

    for series in [first, second] {
        let job = started(&core, Call::RefreshAiring { series });
        let done = common::wait_job(&c, job);
        // The schedule still landed: Jikan's failure costs the titles only.
        assert_eq!(done.body, EventBody::AiringRefreshed { series, updated: true });
    }
    assert_eq!(http.requests().len(), 4);

    let warnings: Vec<String> =
        c.events().into_iter().filter(|e| e.level == Level::Warn && e.message.contains("Jikan")).map(|e| e.message).collect();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].starts_with("Jikan is not answering:"), "{warnings:?}");

    core.shutdown();
}
