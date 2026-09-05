//! The progress cache: both list fetches, the five minute rule, and what a
//! failed fetch leaves behind. The trackers are connected through the
//! fixtures rather than through a flow, so these tests are about the cache
//! alone.

mod common;
mod fixtures;

use std::time::Duration;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;

fn started(reply: Reply) -> u64 {
    match reply {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

/// One series' card, off the list every page reads.
fn card(core: &Core, series: u64) -> SeriesCard {
    match core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
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

/// A library of two matched series, one per tracker id, and the file each
/// one needs to be listed at all.
fn library(core: &Core) -> (u64, u64) {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let frieren = fixtures::insert_series(
        core,
        src,
        SeriesKind::Show,
        "/lib/Frieren",
        "Sousou no Frieren",
    );
    fixtures::insert_file(
        core,
        frieren,
        "/lib/Frieren/01.mkv",
        1.0,
        None,
        "episode",
        now,
    );
    fixtures::insert_media(
        core,
        154587,
        Some("Sousou no Frieren"),
        None,
        Some(28),
        "RELEASING",
        "TV",
        Some(91),
    );
    fixtures::match_series(core, frieren, Some(154587), Some(52991));
    let bebop = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    fixtures::insert_file(core, bebop, "/lib/Bebop/01.mkv", 1.0, None, "episode", now);
    fixtures::insert_media(
        core,
        1,
        Some("Cowboy Bebop"),
        None,
        Some(26),
        "FINISHED",
        "TV",
        Some(86),
    );
    fixtures::match_series(core, bebop, Some(1), Some(1));
    (frieren, bebop)
}

fn collection(frieren_progress: u64) -> serde_json::Value {
    serde_json::json!({
        "data": {
            "MediaListCollection": {
                "lists": [{
                    "entries": [
                        { "progress": frieren_progress, "status": "CURRENT", "score": 8.5, "repeat": 1, "media": { "id": 154587 } },
                        { "progress": 12, "status": "COMPLETED", "score": 0, "repeat": 0, "media": { "id": 1 } }
                    ]
                }]
            }
        }
    })
}

#[test]
fn the_anilist_list_fills_the_cache_and_the_five_minute_rule_holds_it() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (frieren, bebop) = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    http.push_json(200, collection(5));

    // MAL is not connected, so a refresh of every tracker is AniList's
    // alone.
    let job = started(core.call(Call::RefreshProgress { tracker: None }).unwrap());
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::ProgressRefreshed {
                tracker: Tracker::Anilist
            }
        ),
        "{done:?}"
    );

    let request = &http.requests()[0];
    assert!(
        request
            .headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer tok"),
        "{request:?}"
    );
    let sent = format!("{:?}", request.body);
    assert!(sent.contains("userId"), "{sent}");
    assert!(sent.contains("42"), "{sent}");

    let watching = card(&core, frieren);
    assert_eq!(watching.watched, Some(5));
    assert_eq!(watching.my_score, Some(8.5));
    assert_eq!(watching.list_status, Some(ListStatus::Watching));
    // A score of 0 is unrated, not a rating of nothing.
    let finished = card(&core, bebop);
    assert_eq!(finished.watched, Some(12));
    assert_eq!(finished.my_score, None);
    assert_eq!(finished.list_status, Some(ListStatus::Completed));

    // Inside the window nothing is asked for, and the job says so.
    let again = started(core.call(Call::RefreshProgress { tracker: None }).unwrap());
    let notice = common::wait_job(&c, again);
    assert!(matches!(notice.body, EventBody::Notice), "{notice:?}");
    assert_eq!(notice.message, "nothing to refresh");
    assert_eq!(http.requests().len(), 1);
    assert!(
        c.events()
            .iter()
            .any(|e| e.message == "anilist progress is fresh"),
        "{:#?}",
        c.events()
            .iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
    );

    // Past the window it fetches again, and the new numbers land.
    fixtures::age_progress(&core, Tracker::Anilist, 600);
    http.push_json(200, collection(6));
    let third = started(
        core.call(Call::RefreshProgress {
            tracker: Some(Tracker::Anilist),
        })
        .unwrap(),
    );
    let refreshed = common::wait_job(&c, third);
    assert!(
        matches!(
            refreshed.body,
            EventBody::ProgressRefreshed {
                tracker: Tracker::Anilist
            }
        ),
        "{refreshed:?}"
    );
    assert_eq!(http.requests().len(), 2);
    assert_eq!(card(&core, frieren).watched, Some(6));
}

/// MAL pages at a thousand, so a full page with a `paging.next` is asked to
/// go round again and a short page ends the walk.
#[test]
fn the_mal_list_pages_until_a_short_page_and_promotes_a_rewatch() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (frieren, bebop) = library(&core);
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    // A full first page of ids nothing in the library carries, with the
    // library's own two entries at the end of it and on the short page.
    let mut first: Vec<serde_json::Value> = (0..999)
        .map(|n| serde_json::json!({ "node": { "id": 900_000 + n }, "list_status": { "status": "completed", "num_episodes_watched": 1, "score": 5 } }))
        .collect();
    first.push(serde_json::json!({
        "node": { "id": 52991 },
        "list_status": { "status": "watching", "num_episodes_watched": 5, "is_rewatching": true, "num_times_rewatched": 2, "score": 8 }
    }));
    http.push_for(
        "animelist",
        200,
        serde_json::json!({ "data": first, "paging": { "next": "https://api.myanimelist.net/v2/users/@me/animelist?offset=1000" } }).to_string(),
    );
    http.push_for(
        "animelist",
        200,
        serde_json::json!({
            "data": [{ "node": { "id": 1 }, "list_status": { "status": "on_hold", "num_episodes_watched": 3, "score": 0 } }],
            "paging": {}
        })
        .to_string(),
    );

    let job = started(
        core.call(Call::RefreshProgress {
            tracker: Some(Tracker::Mal),
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::ProgressRefreshed {
                tracker: Tracker::Mal
            }
        ),
        "{done:?}"
    );

    let urls: Vec<String> = http.requests().iter().map(|r| r.url.clone()).collect();
    assert_eq!(urls.len(), 2, "{urls:?}");
    assert!(
        urls[0].contains("offset=0") && urls[0].contains("limit=1000"),
        "{urls:?}"
    );
    assert!(urls[0].contains("fields=list_status{status,num_episodes_watched,is_rewatching,num_times_rewatched,score}"), "{urls:?}");
    assert!(urls[1].contains("offset=1000"), "{urls:?}");
    assert!(
        http.requests()[1]
            .headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer mtok")
    );

    // Watching plus the rewatch flag is AniList's Repeating.
    let rewatching = card(&core, frieren);
    assert_eq!(rewatching.watched, Some(5));
    assert_eq!(rewatching.my_score, Some(8.0));
    assert_eq!(rewatching.list_status, Some(ListStatus::Repeating));
    let paused = card(&core, bebop);
    assert_eq!(paused.watched, Some(3));
    assert_eq!(paused.list_status, Some(ListStatus::Paused));
    assert_eq!(paused.my_score, None);
}

/// A fetch that fails leaves the cache exactly as it was, says so once, and
/// fails the job because it was the only tracker asked for.
#[test]
fn a_failed_fetch_keeps_the_rows_and_fails_a_job_with_nothing_else_to_do() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (frieren, _) = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    http.push_json(200, collection(5));
    let job = started(core.call(Call::RefreshProgress { tracker: None }).unwrap());
    common::wait_job(&c, job);
    assert_eq!(card(&core, frieren).watched, Some(5));

    fixtures::age_progress(&core, Tracker::Anilist, 600);
    http.fail_next("connection refused");
    let failed = started(
        core.call(Call::RefreshProgress {
            tracker: Some(Tracker::Anilist),
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, failed);
    assert!(matches!(done.body, EventBody::JobFailed { .. }), "{done:?}");
    assert!(
        c.events().iter().any(
            |e| e.level == Level::Warn && e.message.contains("anilist progress refresh failed")
        ),
        "{:#?}",
        c.events()
            .iter()
            .map(|e| (e.level, e.message.clone()))
            .collect::<Vec<_>>()
    );
    assert_eq!(card(&core, frieren).watched, Some(5));
}

/// Nothing connected is nothing to do, not a failure.
#[test]
fn a_disconnected_tracker_is_skipped() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    library(&core);
    let job = started(
        core.call(Call::RefreshProgress {
            tracker: Some(Tracker::Mal),
        })
        .unwrap(),
    );
    let done = common::wait_for(
        &c,
        |e| {
            e.job
                .as_ref()
                .is_some_and(|j| j.id == job && j.phase == JobPhase::Finished)
        },
        Duration::from_secs(10),
    );
    assert!(matches!(done.body, EventBody::Notice), "{done:?}");
    assert_eq!(done.message, "nothing to refresh");
    assert!(http.requests().is_empty());
}

/// The launch sweep is not a question about one tracker, so one dead token
/// beside one account that needs nothing is a Warn line and a quiet
/// terminal, never a red job on the shell's status strip. A refresh asked
/// for by name still fails red: there the shell asked about that tracker
/// and nothing else.
#[test]
fn a_sweep_with_one_dead_token_warns_rather_than_failing_the_job() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    // Both fill once, so both carry a fetch time.
    http.push_for("anilist", 200, collection(5).to_string());
    http.push_for(
        "animelist",
        200,
        serde_json::json!({ "data": [], "paging": {} }).to_string(),
    );
    let job = started(core.call(Call::RefreshProgress { tracker: None }).unwrap());
    common::wait_job(&c, job);

    // AniList is fresh and skipped; MAL is stale and its token is dead.
    fixtures::age_progress(&core, Tracker::Mal, 600);
    http.fail_next("connection refused");
    let job = started(core.call(Call::RefreshProgress { tracker: None }).unwrap());
    let done = common::wait_job(&c, job);
    assert!(matches!(done.body, EventBody::Notice), "{done:?}");
    assert_eq!(done.level, Level::Debug);
    assert!(
        c.events()
            .iter()
            .any(|e| e.level == Level::Warn && e.message.contains("mal progress refresh failed")),
        "{:#?}",
        c.events()
            .iter()
            .map(|e| (e.level, e.message.clone()))
            .collect::<Vec<_>>()
    );

    // The same failure, asked for by name, is the job's answer.
    http.fail_next("connection refused");
    let job = started(
        core.call(Call::RefreshProgress {
            tracker: Some(Tracker::Mal),
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, job);
    assert!(matches!(done.body, EventBody::JobFailed { .. }), "{done:?}");
}

/// A rate limit the limiter could not ride out reaches the shell as a rate
/// limit, with the tracker named and the provider's own message sanitised.
/// It must never arrive as a timeout: the whole point of capping one
/// request rather than the call is that the 429 schedule underneath is
/// allowed to run to its end.
#[test]
fn an_exhausted_rate_limit_is_a_rate_limit_and_not_a_timeout() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    // The first request plus the limiter's six retries. `Retry-After: 0`
    // is what keeps the test to the pacing gap rather than the 1, 2, 4, 8,
    // 16, 32 second schedule.
    for _ in 0..7 {
        http.push_for_with_headers(
            "graphql",
            429,
            "rate limited",
            vec![("Retry-After".to_string(), "0".to_string())],
        );
    }

    let job = started(
        core.call(Call::RefreshProgress {
            tracker: Some(Tracker::Anilist),
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::JobFailed {
                error: CoreError::Provider {
                    status: Some(429),
                    ..
                }
            }
        ),
        "{done:?}"
    );
    assert_eq!(http.requests().len(), 7, "the whole schedule was not spent");
    let warning = c
        .events()
        .into_iter()
        .find(|e| e.level == Level::Warn && e.message.contains("progress refresh failed"))
        .expect("one warning naming the tracker");
    assert!(
        !warning.message.contains("timed out"),
        "a rate limit read as a timeout: {}",
        warning.message
    );
}
