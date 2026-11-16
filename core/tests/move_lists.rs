//! Moving the list kept on this device up to a tracker (#580): the preview
//! reads the cache and sends nothing, and the move writes only the series
//! the person picked, to the one tracker named, after they confirm.

mod common;
mod fixtures;

use anibeam_core::net::{Body, FakeHttp};
use anibeam_core::*;

fn show(core: &Core, src: u64, name: &str, anilist: Option<u64>) -> u64 {
    let now = anibeam_core::time::now_secs();
    let path = format!("/lib/{name}");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, &path, name);
    fixtures::insert_episode_file(core, series, &format!("{path}/01.mkv"), 1.0, now);
    if let Some(id) = anilist {
        fixtures::media(id, name).episodes(12).insert(core);
        fixtures::match_series(core, series, Some(id), None);
    }
    series
}

fn preview(core: &Core) -> Vec<MoveRow> {
    match core
        .call(Call::PreviewListMove {
            tracker: Tracker::Anilist,
        })
        .unwrap()
    {
        Reply::ListMove { rows } => rows,
        other => panic!("{other:?}"),
    }
}

fn graphql_variables(http: &FakeHttp, operation: &str) -> Vec<serde_json::Value> {
    http.requests()
        .into_iter()
        .filter_map(|r| match r.body {
            Some(Body::Json(v)) if v["query"].as_str().is_some_and(|q| q.contains(operation)) => {
                Some(v["variables"].clone())
            }
            _ => None,
        })
        .collect()
}

/// The preview names each show the move would change, beside what the
/// tracker has now. A show with no AniList id has nowhere to go, and one the
/// tracker already matches changes nothing, so neither is listed. Nothing
/// is sent.
#[test]
fn the_preview_lists_what_would_change_and_sends_nothing() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let bebop = show(&core, src, "Bebop", Some(1));
    let same = show(&core, src, "Same", Some(2));
    let home = show(&core, src, "Home Video", None);
    for (series, status, score) in [
        (bebop, ListStatus::Watching, Some(8.5)),
        (same, ListStatus::Completed, None),
        (home, ListStatus::Planning, None),
    ] {
        core.call(Call::SetListStatus { series, status }).unwrap();
        if score.is_some() {
            core.call(Call::SetScore { series, score }).unwrap();
        }
    }
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 3, "planning", None);
    fixtures::insert_tracker_entry(&core, "anilist", 2, 12, "completed", None);
    // The tracker's entries last changed a day before the list kept here.
    core.store()
        .write(|c| {
            c.execute(
                "UPDATE tracker_entries SET updated_at = updated_at - 86400",
                [],
            )?;
            Ok(())
        })
        .unwrap();

    let rows = preview(&core);
    assert_eq!(
        rows,
        vec![MoveRow {
            series: bebop,
            title: "Bebop".into(),
            status: Some(ListStatus::Watching),
            score: Some(8.5),
            tracker_status: Some(ListStatus::Planning),
            tracker_score: None,
        }]
    );
    assert!(http.requests().is_empty());
}

/// A move to a tracker that is not connected is refused before anything.
#[test]
fn a_move_needs_the_tracker_connected() {
    let (_dir, core, _c) = common::open_core_with_http(FakeHttp::new());
    let src = fixtures::insert_source(&core, "/lib");
    let bebop = show(&core, src, "Bebop", Some(1));
    assert_eq!(
        core.call(Call::MoveLists {
            tracker: Tracker::Anilist,
            series: vec![bebop],
        })
        .err(),
        Some(CoreError::NotConnected {
            tracker: Tracker::Anilist
        })
    );
}

/// The confirmed move writes the status and the score of the picked show to
/// AniList and nothing for the one left out; the card then reads the
/// tracker's entry, and the preview has nothing left for that show.
#[test]
fn the_move_writes_only_what_was_picked() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let bebop = show(&core, src, "Bebop", Some(1));
    let other = show(&core, src, "Other", Some(2));
    for series in [bebop, other] {
        core.call(Call::SetListStatus {
            series,
            status: ListStatus::Watching,
        })
        .unwrap();
        core.call(Call::SetScore {
            series,
            score: Some(7.0),
        })
        .unwrap();
    }
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    assert_eq!(preview(&core).len(), 2);

    for reply in [
        r#"{"data":{"MediaList":null}}"#,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"status":"CURRENT"}}}"#,
        r#"{"data":{"MediaList":{"progress":0,"status":"CURRENT"}}}"#,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"score":7}}}"#,
    ] {
        http.push_for("anilist", 200, reply);
    }
    let job = match core
        .call(Call::MoveLists {
            tracker: Tracker::Anilist,
            series: vec![bebop],
        })
        .unwrap()
    {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::ListsMoved {
            tracker: Tracker::Anilist,
            moved: 1,
            failed: 0
        }
    );
    let saves = graphql_variables(&http, "SaveMediaListEntry");
    // A show not on the list yet goes onto Watching the way the status menu
    // puts it there: dated today (#211).
    let t = ::time::OffsetDateTime::now_utc();
    let today =
        serde_json::json!({ "year": t.year(), "month": u8::from(t.month()), "day": t.day() });
    assert_eq!(
        saves,
        vec![
            serde_json::json!({ "mediaId": 1, "status": "CURRENT", "startedAt": today }),
            serde_json::json!({ "mediaId": 1, "scoreRaw": 70 }),
        ]
    );
    let rows = preview(&core);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].series, other);
}

/// Once the tracker has changed since the list kept here was set, the
/// tracker's word is the newer one: the row is not offered, so a stale
/// local status can never overwrite it. A score alone, for a show not on
/// the tracker's list, waits for a status rather than picking a list for
/// the person; and Watching here against a rewatch there is no change.
#[test]
fn a_stale_row_a_lone_score_and_a_rewatch_are_not_offered() {
    let (_dir, core, _c) = common::open_core_with_http(FakeHttp::new());
    let src = fixtures::insert_source(&core, "/lib");
    let stale = show(&core, src, "Stale", Some(1));
    let lone = show(&core, src, "Lone", Some(2));
    let again = show(&core, src, "Again", Some(3));
    core.call(Call::SetListStatus {
        series: stale,
        status: ListStatus::Watching,
    })
    .unwrap();
    core.call(Call::SetScore {
        series: lone,
        score: Some(6.0),
    })
    .unwrap();
    core.call(Call::SetListStatus {
        series: again,
        status: ListStatus::Watching,
    })
    .unwrap();
    // The local rows are an hour old; the tracker changed after them.
    core.store()
        .write(|c| {
            c.execute(
                "UPDATE local_entries SET updated_at = updated_at - 3600",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 12, "completed", None);
    fixtures::insert_tracker_entry(&core, "anilist", 3, 2, "repeating", None);
    core.store()
        .write(|c| {
            c.execute(
                "UPDATE tracker_entries SET updated_at = updated_at - 7200 WHERE media_id = 3",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    assert!(preview(&core).is_empty(), "{:?}", preview(&core));
}
