//! The planning list (#210): the AniList entries on Planning, read off the
//! same refresh as the watching list, each saying whether the library has it.

mod common;
mod fixtures;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;

fn planning(core: &Core) -> WatchingList {
    match core.call(Call::ListPlanning).unwrap() {
        Reply::Planning { list } => list,
        other => panic!("{other:?}"),
    }
}

fn watching(core: &Core) -> WatchingList {
    match core.call(Call::ListWatching).unwrap() {
        Reply::Watching { list, .. } => list,
        other => panic!("{other:?}"),
    }
}

fn entry(id: u64, status: &str, title: &str, updated_at: i64) -> serde_json::Value {
    serde_json::json!({
        "progress": 0, "status": status, "score": null, "updatedAt": updated_at,
        "media": {
            "id": id, "idMal": null, "siteUrl": format!("https://anilist.co/anime/{id}"),
            "episodes": 12, "averageScore": 80,
            "title": { "romaji": title, "english": null },
            "coverImage": { "large": null },
            "nextAiringEpisode": null
        }
    })
}

#[test]
fn the_refresh_keeps_planning_entries_apart_from_watching() {
    let http = FakeHttp::new();
    http.push_json(
        200,
        serde_json::json!({ "data": { "MediaListCollection": { "lists": [ { "entries": [
            entry(1, "CURRENT", "Cowboy Bebop", 3000),
            entry(2, "PLANNING", "Mushishi", 1000),
            entry(3, "PLANNING", "Planetes", 2000),
            entry(4, "COMPLETED", "Monster", 4000),
        ] } ] } } }),
    );
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let mushishi =
        fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Mushishi", "Mushishi");
    fixtures::insert_file(
        &core,
        mushishi,
        "/lib/Mushishi/01.mkv",
        1.0,
        None,
        "episode",
        1,
    );
    fixtures::insert_media(
        &core,
        2,
        Some("Mushishi"),
        None,
        Some(26),
        "FINISHED",
        "TV",
        Some(86),
    );
    fixtures::match_series(&core, mushishi, Some(2), None);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    let job = match core.call(Call::RefreshWatching { force: true }).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    common::wait_job(&c, job);

    let watched: Vec<u64> = watching(&core)
        .entries
        .iter()
        .map(|e| e.anilist_id)
        .collect();
    assert_eq!(watched, vec![1], "Planning never reaches the watching list");

    let list = planning(&core);
    // Most recently updated first, as the watching list orders.
    let ids: Vec<u64> = list.entries.iter().map(|e| e.anilist_id).collect();
    assert_eq!(ids, vec![3, 2]);
    assert_eq!(list.entries[0].title, "Planetes");
    assert_eq!(list.entries[0].owned, None, "not in the library");
    assert_eq!(list.entries[1].owned, Some(mushishi), "on disk");
    assert!(list.fetched_at.is_some());

    // Planetes comes off Planning on AniList: the next refresh drops it here too.
    http.push_json(
        200,
        serde_json::json!({ "data": { "MediaListCollection": { "lists": [ { "entries": [
            entry(1, "CURRENT", "Cowboy Bebop", 3000),
            entry(2, "PLANNING", "Mushishi", 1000),
        ] } ] } } }),
    );
    let job = match core.call(Call::RefreshWatching { force: true }).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    common::wait_job(&c, job);
    let ids: Vec<u64> = planning(&core)
        .entries
        .iter()
        .map(|e| e.anilist_id)
        .collect();
    assert_eq!(ids, vec![2]);
}

#[test]
fn a_planned_show_moved_to_watching_leaves_the_planning_list() {
    let http = FakeHttp::new();
    http.push_json(
        200,
        serde_json::json!({ "data": { "MediaListCollection": { "lists": [ { "entries": [
            entry(2, "PLANNING", "Mushishi", 1000),
        ] } ] } } }),
    );
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let mushishi =
        fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Mushishi", "Mushishi");
    fixtures::insert_file(
        &core,
        mushishi,
        "/lib/Mushishi/01.mkv",
        1.0,
        None,
        "episode",
        1,
    );
    fixtures::insert_media(
        &core,
        2,
        Some("Mushishi"),
        None,
        Some(26),
        "FINISHED",
        "TV",
        Some(86),
    );
    fixtures::match_series(&core, mushishi, Some(2), None);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    let job = match core.call(Call::RefreshWatching { force: true }).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    common::wait_job(&c, job);
    assert_eq!(planning(&core).entries.len(), 1);

    // Start watching is a status write; its cache patch moves the entry over.
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":0,"status":"PLANNING"}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"status":"CURRENT"}}}"#,
    );
    let job = match core
        .call(Call::SetListStatus {
            series: mushishi,
            status: ListStatus::Watching,
        })
        .unwrap()
    {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    common::wait_job(&c, job);
    assert!(planning(&core).entries.is_empty());
    assert_eq!(watching(&core).entries.len(), 1);
}

#[test]
fn a_hidden_series_keeps_its_entry_off_the_planning_list() {
    let (_dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    let s = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Mushishi", "Mushishi");
    fixtures::insert_media(
        &core,
        2,
        Some("Mushishi"),
        None,
        Some(26),
        "FINISHED",
        "TV",
        Some(86),
    );
    fixtures::match_series(&core, s, Some(2), None);
    fixtures::insert_tracker_entry(&core, "anilist", 2, 0, "planning", None);
    assert_eq!(planning(&core).entries.len(), 1);
    core.call(Call::SetHidden {
        series: s,
        hidden: true,
    })
    .unwrap();
    assert!(planning(&core).entries.is_empty());
}
