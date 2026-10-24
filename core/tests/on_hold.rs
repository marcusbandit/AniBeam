//! On hold: what a refresh's sweep moves to the tracker's On hold list, what
//! it leaves alone, the live read it decides on before every write, the
//! switch that turns it off and on, and the play that takes a series off
//! hold again. The trackers are connected through the fixtures and answered
//! by the fake client, so these tests are about On hold alone.

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

fn ended(e: &Event, kind: JobKind) -> bool {
    e.job
        .as_ref()
        .is_some_and(|j| j.kind == kind && j.phase == JobPhase::Finished)
}

/// The `n`th terminal event of a job kind, counting from one, whichever id
/// each ran under.
fn nth_end(c: &Collector, kind: JobKind, n: usize) -> Event {
    let arrived = c.wait_for(
        |events| events.iter().filter(|e| ended(e, kind)).count() >= n,
        Duration::from_secs(30),
    );
    assert!(arrived, "timed out waiting; saw {:#?}", c.events());
    c.events()
        .into_iter()
        .filter(|e| ended(e, kind))
        .nth(n - 1)
        .expect("just confirmed present")
}

fn wait_sweep(c: &Collector) -> Event {
    nth_end(c, JobKind::OnHold, 1)
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
/// AniList id and the MAL id both carried, and the AniList media row
/// knowing its MAL id the way a fetch leaves it.
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
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE anilist_media SET mal_id = ?2 WHERE id = ?1",
                rusqlite::params![anilist as i64, mal as i64],
            )?;
            Ok(())
        })
        .unwrap();
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

/// MAL's list, one page.
fn mal_list(entries: &[(u64, u32, &str, &str)]) -> String {
    let data: Vec<serde_json::Value> = entries
        .iter()
        .map(|(id, watched, status, updated_at)| {
            serde_json::json!({ "node": { "id": id }, "list_status": {
                "status": status, "num_episodes_watched": watched, "is_rewatching": false,
                "num_times_rewatched": 0, "score": 0, "updated_at": updated_at } })
        })
        .collect();
    serde_json::json!({ "data": data, "paging": {} }).to_string()
}

/// AniList's live read of one entry: the entry, and its media's airing state.
fn live(
    status: &str,
    progress: u32,
    updated_at: i64,
    media: &str,
    next: Option<(u32, i64)>,
) -> String {
    let next = next.map(|(episode, at)| serde_json::json!({ "episode": episode, "airingAt": at }));
    serde_json::json!({ "data": { "MediaList": {
        "status": status, "progress": progress, "updatedAt": updated_at,
        "media": { "status": media, "episodes": 12, "nextAiringEpisode": next } } } })
    .to_string()
}

/// MAL's live read of one entry.
fn mal_live(status: &str, watched: u32, updated_at: &str) -> String {
    serde_json::json!({ "id": 1, "status": "finished_airing", "num_episodes": 12, "my_list_status": {
        "status": status, "num_episodes_watched": watched, "is_rewatching": false, "updated_at": updated_at } })
    .to_string()
}

fn saved(status: &str, now: i64) -> String {
    serde_json::json!({ "data": { "SaveMediaListEntry": { "id": 9, "status": status, "updatedAt": now } } })
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

fn live_reads(http: &FakeHttp) -> Vec<serde_json::Value> {
    graphql_variables(http, "MediaList(userId: $userId, mediaId: $mediaId)")
}

fn mal_patches(http: &FakeHttp) -> Vec<Vec<(String, String)>> {
    http.requests()
        .into_iter()
        .filter(|r| r.method == Method::Patch)
        .filter_map(|r| match r.body {
            Some(Body::Form(pairs)) => Some(pairs),
            _ => None,
        })
        .collect()
}

fn refresh(core: &Core, c: &Collector) {
    let job = started(core.call(Call::RefreshProgress { tracker: None }).unwrap());
    common::wait_job(c, job);
}

/// Ninety days untouched on a sixty day setting: the sweep reads the entry
/// live, sends AniList the status alone, the card says On hold at once, and
/// the activity log gets the one line.
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
        live("CURRENT", 3, now - 90 * DAY, "FINISHED", None),
    );
    http.push_for("anilist", 200, saved("PAUSED", now));
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(done.level, Level::Info, "{done:?}");
    assert_eq!(
        done.message,
        "Moved 1 series to On hold on AniList, untouched for 60 days"
    );

    let reads = live_reads(&http);
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert_eq!(reads[0]["mediaId"], 130003);
    let writes = status_writes(&http);
    assert_eq!(writes.len(), 1, "{writes:?}");
    assert_eq!(writes[0]["mediaId"], 130003);
    assert_eq!(writes[0]["status"], "PAUSED");
    // The status alone: the progress and the score stay as the list has them.
    assert!(writes[0].get("progress").is_none(), "{writes:?}");
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Paused));
}

/// What the pre-check leaves alone without a request: an entry touched
/// inside the days, one played here lately whatever the list's stamp says,
/// one waiting on an episode that is not out yet, a hidden one, and one
/// nothing dates.
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
    // One request, the list: nothing got as far as a live read.
    assert_eq!(http.requests().len(), 1, "{:?}", http.requests());
    // A hidden series is listed under its own tab only, and nothing named it.
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

/// The schedule row for the episode after the progress is what dates an
/// airing show: one that came out a hundred days ago, with nothing touched
/// since, is due even though the show is still airing.
#[test]
fn an_airing_show_whose_next_episode_came_out_long_ago_moves() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let series = show(&core, "Airing", 40, 140, 24, "RELEASING");
    fixtures::insert_airing(&core, 40, 4, now - 100 * DAY);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    http.push_for(
        "anilist",
        200,
        collection(&[(40, 3, "CURRENT", now - 120 * DAY)]),
    );
    http.push_for(
        "anilist",
        200,
        live(
            "CURRENT",
            3,
            now - 120 * DAY,
            "RELEASING",
            Some((15, now + 2 * DAY)),
        ),
    );
    http.push_for("anilist", 200, saved("PAUSED", now));
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(done.level, Level::Info, "{done:?}");
    assert_eq!(status_writes(&http).len(), 1);
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Paused));
}

/// The live read decides: an entry completed somewhere else since the list
/// was fetched, and one outside the library whose next episode the live
/// read says is not out yet, both stay, with no write sent.
#[test]
fn the_live_read_keeps_a_changed_entry_and_a_waiting_one_off_hold() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let changed = show(&core, "Changed", 50, 150, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    // 51 is not in the library and has no media row at all, so only the
    // live read knows it is airing.
    http.push_for(
        "anilist",
        200,
        collection(&[
            (50, 3, "CURRENT", now - 300 * DAY),
            (51, 7, "CURRENT", now - 200 * DAY),
        ]),
    );
    http.push_for(
        "anilist",
        200,
        live("COMPLETED", 12, now - 60, "FINISHED", None),
    );
    http.push_for(
        "anilist",
        200,
        live(
            "CURRENT",
            7,
            now - 200 * DAY,
            "RELEASING",
            Some((8, now + 3 * DAY)),
        ),
    );
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(done.message, "Nothing to move to On hold", "{done:?}");
    assert_eq!(live_reads(&http).len(), 2);
    assert!(status_writes(&http).is_empty());
    assert_eq!(card(&core, changed).list_status, Some(ListStatus::Watching));
}

/// With both trackers connected, one pass moves the entry on both: the
/// stamp AniList's move leaves is a move, never a touch for MAL.
#[test]
fn both_trackers_move_in_one_pass() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let series = show(&core, "Both", 20, 120, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "anilist",
        200,
        collection(&[(20, 2, "CURRENT", now - 90 * DAY)]),
    );
    http.push_for(
        "myanimelist",
        200,
        mal_list(&[(120, 2, "watching", "2020-01-01T00:00:00+00:00")]),
    );
    http.push_for(
        "anilist",
        200,
        live("CURRENT", 2, now - 90 * DAY, "FINISHED", None),
    );
    http.push_for("anilist", 200, saved("PAUSED", now));
    http.push_for(
        "myanimelist",
        200,
        mal_live("watching", 2, "2020-01-01T00:00:00+00:00"),
    );
    http.push_for(
        "myanimelist",
        200,
        r#"{"status":"on_hold","num_episodes_watched":2,"updated_at":"2026-10-24T20:00:00+00:00"}"#,
    );
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(
        done.message, "Moved 1 series to On hold on AniList and 1 on MAL, untouched for 60 days",
        "{done:?}"
    );
    assert_eq!(status_writes(&http).len(), 1);
    assert_eq!(
        mal_patches(&http),
        vec![vec![("status".to_string(), "on_hold".to_string())]]
    );
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Paused));
}

/// MAL's list carries its stamp as an RFC 3339 instant, its entry is read
/// live first, and its write is a PATCH with the status alone.
#[test]
fn a_mal_entry_moves_with_a_status_only_patch() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = show(&core, "Mal", 9, 109, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "myanimelist",
        200,
        mal_list(&[(109, 4, "watching", "2020-01-01T00:00:00+00:00")]),
    );
    http.push_for(
        "myanimelist",
        200,
        mal_live("watching", 4, "2020-01-01T00:00:00+00:00"),
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
    assert!(
        http.requests()
            .iter()
            .any(|r| r.url.contains("/anime/109?fields=my_list_status")),
        "{:?}",
        http.requests()
    );
    assert_eq!(
        mal_patches(&http),
        vec![vec![("status".to_string(), "on_hold".to_string())]]
    );
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Paused));
}

/// On MAL the AniList media row is found by the MAL id it carries, so a
/// MAL entry waiting on a scheduled episode stays without a request.
#[test]
fn a_mal_entry_waiting_on_its_next_episode_stays() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let series = show(&core, "MalWaiting", 30, 130, 24, "RELEASING");
    fixtures::insert_airing(&core, 30, 6, now + 3 * DAY);
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "myanimelist",
        200,
        mal_list(&[(130, 5, "watching", "2020-01-01T00:00:00+00:00")]),
    );
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(done.message, "Nothing to move to On hold", "{done:?}");
    assert_eq!(http.requests().len(), 1, "{:?}", http.requests());
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
        live("CURRENT", 2, now - 100 * DAY, "FINISHED", None),
    );
    http.push_for("anilist", 200, saved("PAUSED", now));
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

/// A Sync inside the five minute window fetches nothing, and still sweeps
/// on the list it has: new days take effect there.
#[test]
fn a_sync_inside_the_window_sweeps_with_the_new_days() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let series = show(&core, "Shorter", 12, 112, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    http.push_for(
        "anilist",
        200,
        collection(&[(12, 2, "CURRENT", now - 45 * DAY)]),
    );
    refresh(&core, &c);
    assert_eq!(wait_sweep(&c).message, "Nothing to move to On hold");

    core.call(Call::SetPreferences {
        preferences: Preferences {
            on_hold_days: 30,
            ..Preferences::default()
        },
    })
    .unwrap();
    http.push_for(
        "anilist",
        200,
        live("CURRENT", 2, now - 45 * DAY, "FINISHED", None),
    );
    http.push_for("anilist", 200, saved("PAUSED", now));
    refresh(&core, &c);
    let done = nth_end(&c, JobKind::OnHold, 2);
    assert_eq!(
        done.message, "Moved 1 series to On hold on AniList, untouched for 30 days",
        "{done:?}"
    );
    assert_eq!(graphql_variables(&http, "MediaListCollection").len(), 1);
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Paused));
}

/// A tracker refusing one entry costs that entry alone, and the pass goes
/// on to the next. Either way the pass is one line.
#[test]
fn one_refused_entry_lets_the_pass_go_on() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let first = show(&core, "First", 10, 110, 12, "FINISHED");
    let second = show(&core, "Second", 11, 111, 12, "FINISHED");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    // Longest idle first: 10, then 11.
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
        200,
        live("CURRENT", 1, now - 300 * DAY, "FINISHED", None),
    );
    http.push_for(
        "anilist",
        404,
        r#"{"errors":[{"message":"Not Found.","status":404}]}"#,
    );
    http.push_for(
        "anilist",
        200,
        live("CURRENT", 1, now - 200 * DAY, "FINISHED", None),
    );
    http.push_for("anilist", 200, saved("PAUSED", now));
    refresh(&core, &c);
    let done = wait_sweep(&c);
    assert_eq!(done.level, Level::Warn, "{done:?}");
    assert_eq!(
        done.message,
        "Moved 1 series to On hold on AniList, untouched for 60 days. 1 series could not move to On hold: AniList entry not found."
    );
    assert_eq!(card(&core, first).list_status, Some(ListStatus::Watching));
    assert_eq!(card(&core, second).list_status, Some(ListStatus::Paused));
}

/// An account the tracker refuses stops the pass at the first entry, since
/// every entry after it would fail the same way.
#[test]
fn an_expired_account_stops_the_pass_at_the_first_entry() {
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
    // The list, then the one live read the 401 answered: the second entry
    // was never asked about.
    assert_eq!(http.requests().len(), 2, "{:?}", http.requests());
    assert!(status_writes(&http).is_empty());
    for series in [first, second] {
        assert_eq!(card(&core, series).list_status, Some(ListStatus::Watching));
    }
}

/// Playing a series on hold puts it back on Watching: thirty seconds of
/// play is a view, and the view takes the entry off hold on the tracker
/// once a live read says it is still there.
#[test]
fn a_play_takes_a_series_off_hold() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let series = show(&core, "Again", 61, 161, 12, "FINISHED");
    let file = match core.call(Call::GetSeries { series }).unwrap() {
        Reply::SeriesDetail { detail } => detail.episodes[0].file,
        other => panic!("{other:?}"),
    };
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    fixtures::insert_tracker_entry(&core, "anilist", 61, 0, "paused", None);
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Paused));

    http.push_for(
        "anilist",
        200,
        live("PAUSED", 0, now - 90 * DAY, "FINISHED", None),
    );
    http.push_for("anilist", 200, saved("CURRENT", now));
    let session = match core.call(Call::OpenPlayback { file }).unwrap() {
        Reply::Playback { session } => session.session,
        other => panic!("{other:?}"),
    };
    for step in 0..35 {
        core.call(Call::Tick {
            session,
            position: f64::from(step),
            paused: false,
        })
        .unwrap();
    }
    let done = nth_end(&c, JobKind::SetStatus, 1);
    assert_eq!(done.level, Level::Info, "{done:?}");
    assert_eq!(done.message, "Again is back on Watching on AniList");
    let writes = status_writes(&http);
    assert_eq!(writes.len(), 1, "{writes:?}");
    assert_eq!(writes[0]["status"], "CURRENT");
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Watching));
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
