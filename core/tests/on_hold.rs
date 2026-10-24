//! On hold: what a refresh's sweep moves to the tracker's On hold list, what
//! it leaves alone, and the switch that turns it off and on. The trackers
//! are connected through the fixtures and answered by the fake client, so
//! these tests are about the sweep alone.

mod common;
mod fixtures;

use std::time::Duration;

use anibeam_core::events::Collector;
use anibeam_core::net::{Body, FakeHttp, Method};
use anibeam_core::*;

const DAY: i64 = 86_400;

fn started(reply: Reply) -> u64 {
    match reply {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

/// The sweep's terminal event, whichever job id it ran under.
fn wait_sweep(c: &Collector) -> Event {
    common::wait_for(
        c,
        |e| {
            e.job
                .as_ref()
                .is_some_and(|j| j.kind == JobKind::OnHold && j.phase == JobPhase::Finished)
        },
        Duration::from_secs(30),
    )
}

fn swept(c: &Collector) -> bool {
    c.events()
        .iter()
        .any(|e| e.job.as_ref().is_some_and(|j| j.kind == JobKind::OnHold))
}

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

/// One matched show with a file, so it is listed: `episodes` published, the
/// AniList id and the MAL id both carried.
fn show(core: &Core, name: &str, anilist: u64, mal: u64, episodes: u32, status: &str) -> u64 {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, &format!("/lib/{name}"));
    let path = format!("/lib/{name}/{name}");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, &path, name);
    fixtures::insert_file(
        core,
        series,
        &format!("{path}/01.mkv"),
        1.0,
        None,
        "episode",
        now,
    );
    fixtures::insert_media(
        core,
        anilist,
        Some(name),
        None,
        Some(episodes),
        status,
        "TV",
        Some(80),
    );
    fixtures::match_series(core, series, Some(anilist), Some(mal));
    series
}

/// AniList's list as the progress refresh asks for it, `updatedAt` included.
fn collection(entries: &[(u64, u32, &str, i64)]) -> String {
    let entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|(id, progress, status, updated_at)| {
            serde_json::json!({ "progress": progress, "status": status, "score": 0, "repeat": 0,
                                "updatedAt": updated_at, "media": { "id": id } })
        })
        .collect();
    serde_json::json!({ "data": { "MediaListCollection": { "lists": [{ "entries": entries }] } } })
        .to_string()
}

/// Every GraphQL request whose query names `operation`, as its variables.
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

fn status_writes(http: &FakeHttp) -> Vec<serde_json::Value> {
    graphql_variables(
        http,
        "SaveMediaListEntry(mediaId: $mediaId, status: $status)",
    )
}

fn refresh(core: &Core, c: &Collector) {
    let job = started(core.call(Call::RefreshProgress { tracker: None }).unwrap());
    common::wait_job(c, job);
}

/// Ninety days untouched on a sixty day setting: the refresh's sweep sends
/// AniList the status alone, the card says On hold at once, and the
/// activity log gets the one line.
#[test]
fn an_entry_untouched_past_the_days_moves_to_on_hold_after_a_refresh() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let series = show(&core, "Bocchi", 130003, 47917, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    http.push_for(
        "anilist",
        200,
        collection(&[(130003, 3, "CURRENT", now - 90 * DAY)]),
    );
    http.push_for(
        "anilist",
        200,
        serde_json::json!({ "data": { "SaveMediaListEntry": { "id": 9, "status": "PAUSED", "updatedAt": now } } })
            .to_string(),
    );
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(done.level, Level::Info, "{done:?}");
    assert_eq!(
        done.message,
        "Moved 1 series to On hold on AniList, untouched for 60 days"
    );

    let writes = status_writes(&http);
    assert_eq!(writes.len(), 1, "{writes:?}");
    assert_eq!(writes[0]["mediaId"], 130003);
    assert_eq!(writes[0]["status"], "PAUSED");
    // The status alone: the progress and the score stay as the list has them.
    assert!(writes[0].get("progress").is_none(), "{writes:?}");
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Paused));
}

/// What the sweep leaves alone: an entry touched inside the days, one
/// played here lately whatever the list's stamp says, one waiting on an
/// episode that is not out yet, a hidden one, and one nothing dates.
#[test]
fn touched_waiting_hidden_and_undated_entries_stay_on_watching() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let touched = show(&core, "Touched", 1, 101, 12, "FINISHED");
    let played = show(&core, "Played", 2, 102, 12, "FINISHED");
    fixtures::insert_view(&core, played, "4", now - 5 * DAY);
    let waiting = show(&core, "Waiting", 3, 103, 24, "RELEASING");
    fixtures::insert_airing(&core, 3, 9, now + 3 * DAY);
    let hidden = show(&core, "Hidden", 4, 104, 12, "FINISHED");
    core.call(Call::SetHidden {
        series: hidden,
        hidden: true,
    })
    .unwrap();
    let undated = show(&core, "Undated", 5, 105, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    http.push_for(
        "anilist",
        200,
        collection(&[
            (1, 3, "CURRENT", now - 10 * DAY),
            (2, 3, "CURRENT", now - 200 * DAY),
            (3, 8, "CURRENT", now - 200 * DAY),
            (4, 3, "CURRENT", now - 200 * DAY),
            // AniList's nought is no stamp at all.
            (5, 3, "CURRENT", 0),
        ]),
    );
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(done.message, "Nothing to move to On hold", "{done:?}");
    assert!(status_writes(&http).is_empty());
    // A hidden series is listed under its own tab only, and no write named it.
    let _ = hidden;
    for series in [touched, played, waiting, undated] {
        assert_eq!(card(&core, series).list_status, Some(ListStatus::Watching));
    }
}

/// An episode coming out restarts the clock: caught up for months while
/// the show was on a break, and the break ended a week ago.
#[test]
fn a_show_back_from_a_break_is_not_moved_the_week_it_returns() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let series = show(&core, "Break", 7, 107, 24, "RELEASING");
    fixtures::insert_airing(&core, 7, 6, now - 7 * DAY);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    http.push_for(
        "anilist",
        200,
        collection(&[(7, 5, "CURRENT", now - 120 * DAY)]),
    );
    refresh(&core, &c);
    wait_sweep(&c);
    assert!(status_writes(&http).is_empty());
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Watching));
}

/// Switched off, a refresh starts no sweep at all. Switching it on sweeps
/// at once on the list just fetched, with no second fetch.
#[test]
fn switched_off_nothing_moves_and_switching_on_sweeps_at_once() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let series = show(&core, "Idle", 8, 108, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    let off = Preferences {
        on_hold_auto: false,
        ..Preferences::default()
    };
    core.call(Call::SetPreferences {
        preferences: off.clone(),
    })
    .unwrap();

    http.push_for(
        "anilist",
        200,
        collection(&[(8, 2, "CURRENT", now - 100 * DAY)]),
    );
    refresh(&core, &c);
    assert!(!swept(&c), "a sweep ran while switched off");
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Watching));

    http.push_for(
        "anilist",
        200,
        serde_json::json!({ "data": { "SaveMediaListEntry": { "id": 9, "status": "PAUSED", "updatedAt": now } } })
            .to_string(),
    );
    core.call(Call::SetPreferences {
        preferences: Preferences {
            on_hold_auto: true,
            ..off
        },
    })
    .unwrap();
    let done = wait_sweep(&c);
    assert_eq!(done.level, Level::Info, "{done:?}");
    assert_eq!(status_writes(&http).len(), 1);
    // One list fetch: the sweep decided on the one still inside the window.
    assert_eq!(graphql_variables(&http, "MediaListCollection").len(), 1);
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Paused));
}

/// MAL's list carries its stamp as an RFC 3339 instant, and its write is a
/// PATCH with the status alone.
#[test]
fn a_mal_entry_moves_with_a_status_only_patch() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = show(&core, "Mal", 9, 109, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "myanimelist",
        200,
        serde_json::json!({
            "data": [{ "node": { "id": 109 }, "list_status": {
                "status": "watching", "num_episodes_watched": 4, "is_rewatching": false,
                "num_times_rewatched": 0, "score": 0, "updated_at": "2020-01-01T00:00:00+00:00" } }],
            "paging": {}
        })
        .to_string(),
    );
    http.push_for(
        "myanimelist",
        200,
        r#"{"status":"on_hold","num_episodes_watched":4,"updated_at":"2026-10-24T20:00:00+00:00"}"#,
    );
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(
        done.message, "Moved 1 series to On hold on MAL, untouched for 60 days",
        "{done:?}"
    );
    let patches: Vec<Vec<(String, String)>> = http
        .requests()
        .into_iter()
        .filter(|r| r.method == Method::Patch)
        .filter_map(|r| match r.body {
            Some(Body::Form(pairs)) => Some(pairs),
            _ => None,
        })
        .collect();
    assert_eq!(
        patches,
        vec![vec![("status".to_string(), "on_hold".to_string())]]
    );
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Paused));
}

/// A refused write is one Warn line for the pass, and a 401 stops the pass
/// rather than failing every entry after it the same way.
#[test]
fn a_refused_write_is_one_warn_line_and_an_expired_account_stops_the_pass() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let first = show(&core, "First", 10, 110, 12, "FINISHED");
    let second = show(&core, "Second", 11, 111, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    http.push_for(
        "anilist",
        200,
        collection(&[
            (10, 1, "CURRENT", now - 300 * DAY),
            (11, 1, "CURRENT", now - 200 * DAY),
        ]),
    );
    http.push_for(
        "anilist",
        401,
        r#"{"errors":[{"message":"Invalid token","status":401}]}"#,
    );
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(done.level, Level::Warn, "{done:?}");
    assert!(
        done.message
            .starts_with("2 series could not move to On hold: AniList auth expired"),
        "{done:?}"
    );
    // The second entry was never sent.
    assert_eq!(status_writes(&http).len(), 1);
    for series in [first, second] {
        assert_eq!(card(&core, series).list_status, Some(ListStatus::Watching));
    }
}

/// The days have bounds: a week at the least, a year at the most.
#[test]
fn the_days_outside_a_week_to_a_year_are_refused() {
    let (_dir, core, _c) = common::open_core();
    for days in [0, 6, 366] {
        let refused = core.call(Call::SetPreferences {
            preferences: Preferences {
                on_hold_days: days,
                ..Preferences::default()
            },
        });
        assert!(
            matches!(refused, Err(CoreError::Invalid { .. })),
            "{days}: {refused:?}"
        );
    }
    for days in [7, 60, 365] {
        core.call(Call::SetPreferences {
            preferences: Preferences {
                on_hold_days: days,
                ..Preferences::default()
            },
        })
        .unwrap();
    }
}
