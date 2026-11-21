//! Marks, progress and scores: the guards that answer before a request is
//! made, the writes that go to every connected tracker, and what a failed
//! tracker leaves behind. Both trackers are connected through the fixtures
//! rather than through a flow, so these tests are about the writes alone.

mod common;
use common::{job_of, started};
mod fixtures;

use std::sync::Arc;
use std::time::Duration;

use anibeam_core::net::{
    Body, BoxFuture, FakeHttp, Http, HttpError, HttpRequest, HttpResponse, Method,
};
use anibeam_core::*;

/// One matched series, both ids carried, twelve episodes published, and the
/// file it needs to be listed at all.
fn library(core: &Core) -> u64 {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    fixtures::insert_episode_file(core, series, "/lib/Bebop/01.mkv", 1.0, now);
    fixtures::media(1, "Cowboy Bebop")
        .episodes(12)
        .score(86)
        .insert(core);
    fixtures::match_series(core, series, Some(1), Some(21));
    series
}

/// One series' card, off the list every page reads. Completing a series
/// files it into the Watched tab by default, so both places are looked in.
fn card(core: &Core, series: u64) -> SeriesCard {
    let cards = |tab: Tab| common::cards(core, tab, false);
    cards(Tab::All)
        .into_iter()
        .find(|c| c.id == series)
        .or_else(|| cards(Tab::Watched).into_iter().find(|c| c.id == series))
        .expect("the series is listed")
}

/// Every GraphQL request whose query names `operation`, newest last, as the
/// variables it carried.
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

/// Every MAL list write, as the form it carried.
fn mal_forms(http: &FakeHttp) -> Vec<Vec<(String, String)>> {
    http.requests()
        .into_iter()
        .filter(|r| r.method == Method::Patch)
        .filter_map(|r| match r.body {
            Some(Body::Form(pairs)) => Some(pairs),
            _ => None,
        })
        .collect()
}

fn outcome(outcomes: &[TrackerOutcome], t: Tracker) -> TrackerOutcome {
    outcomes
        .iter()
        .find(|o| o.tracker == t)
        .unwrap_or_else(|| panic!("no {t:?} outcome in {outcomes:?}"))
        .clone()
}

/// A mark reaches every connected tracker: AniList's entry is read first so
/// the guard has a number to compare, MAL answers 404 for an anime the user
/// has never added, and the twelfth of twelve completes both lists.
#[test]
fn a_mark_writes_to_both_trackers_and_the_card_follows() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":11,"status":"CURRENT"}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"progress":12,"status":"COMPLETED"}}}"#,
    );
    http.push_for("myanimelist", 404, r#"{"error":"not_found"}"#);
    http.push_for("myanimelist", 200, "{}");

    let job = started(
        &core,
        Call::MarkEpisode {
            series,
            episode: 12.5,
        },
    );
    let done = common::wait_job(&c, job);
    let (episode, outcomes) = match done.body {
        EventBody::Marked {
            episode, outcomes, ..
        } => (episode, outcomes),
        other => panic!("{other:?}"),
    };
    // A decimal episode is floored for a tracker: 12.5 is still episode 12.
    assert_eq!(episode, 12);
    assert_eq!(done.message, "Marked episode 12 of Cowboy Bebop");
    assert_eq!(outcomes.len(), 2);
    assert_eq!(
        outcome(&outcomes, Tracker::Anilist),
        TrackerOutcome {
            tracker: Tracker::Anilist,
            ok: true,
            progress: Some(12),
            reason: None,
            message: None
        }
    );
    assert_eq!(
        outcome(&outcomes, Tracker::Mal),
        TrackerOutcome {
            tracker: Tracker::Mal,
            ok: true,
            progress: Some(12),
            reason: None,
            message: None
        }
    );

    // The entry read carries the user id: `MediaList(mediaId)` without it
    // ignores the bearer token and answers with a stranger's progress.
    assert_eq!(
        graphql_variables(&http, "MediaList(userId:"),
        vec![serde_json::json!({ "userId": 42, "mediaId": 1 })]
    );
    let vars = graphql_variables(&http, "SaveMediaListEntry");
    assert_eq!(vars.len(), 1);
    // A finish with no date on the list dates it today (#211).
    let (day, _) = today();
    assert_eq!(
        vars[0],
        serde_json::json!({ "mediaId": 1, "progress": 12, "status": "COMPLETED", "completedAt": day })
    );
    let forms = mal_forms(&http);
    assert_eq!(forms.len(), 1);
    assert!(
        forms[0].contains(&("num_watched_episodes".to_string(), "12".to_string())),
        "{:?}",
        forms[0]
    );
    assert!(
        forms[0].contains(&("status".to_string(), "completed".to_string())),
        "{:?}",
        forms[0]
    );

    // The cache was patched before the card was built, so the card the job
    // pushed already carries the new number.
    assert_eq!(card(&core, series).watched, Some(12));
    // Both accounts were written to, so both say when they last were.
    let state = match core.call(Call::GetTrackers).unwrap() {
        Reply::Trackers { state } => state,
        other => panic!("{other:?}"),
    };
    assert!(state.anilist.last_sync.is_some());
    assert!(state.mal.last_sync.is_some());
    assert!(
        c.events()
            .iter()
            .any(|e| e.message == "anilist 11 -> 12 (mediaId 1) + finished"),
        "{:#?}",
        c.events()
            .iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
    );
}

/// The monotonic guard answers off the cache, before a request is made.
#[test]
fn a_mark_the_list_already_covers_is_refused_at_once() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 12, "completed", None);

    let err = core
        .call(Call::MarkEpisode {
            series,
            episode: 12.0,
        })
        .err()
        .unwrap();
    assert_eq!(
        err,
        CoreError::Refused {
            reason: Refusal::NotNewer
        }
    );
    assert!(http.requests().is_empty());
}

/// A hidden series is absent from every tracker write, whichever id carries
/// it.
#[test]
fn a_hidden_series_is_refused() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    core.call(Call::SetHidden {
        series,
        hidden: true,
    })
    .unwrap();

    for call in [
        Call::MarkEpisode {
            series,
            episode: 3.0,
        },
        Call::SetProgress {
            series,
            progress: 3,
            season: None,
        },
        Call::SetScore {
            series,
            score: Some(8.0),
        },
        Call::SetListStatus {
            series,
            status: ListStatus::Dropped,
        },
    ] {
        assert_eq!(
            core.call(call).err().unwrap(),
            CoreError::Refused {
                reason: Refusal::Hidden
            }
        );
    }
    assert!(http.requests().is_empty());
}

/// Nothing to write to: an unmatched series carries neither id. Progress, a
/// score and a status are kept here instead.
#[test]
fn an_unmatched_series_is_refused() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Nothing", "Nothing");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    assert_eq!(
        core.call(Call::MarkEpisode {
            series,
            episode: 1.0
        })
        .err()
        .unwrap(),
        CoreError::Refused {
            reason: Refusal::NoMatch
        }
    );
    // Progress, a score and a status still land: on this device, with no
    // job (#213)
    assert_eq!(
        core.call(Call::SetProgress {
            series,
            progress: 1,
            season: None,
        })
        .unwrap(),
        Reply::Ok
    );
    assert_eq!(
        core.call(Call::SetScore {
            series,
            score: None
        })
        .unwrap(),
        Reply::Ok
    );
    assert!(http.requests().is_empty());
}

/// With no account behind either id the refusal names the main tracker,
/// which is the one the Settings tab should open on.
#[test]
fn no_connected_tracker_names_the_main_one() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);

    assert_eq!(
        core.call(Call::MarkEpisode {
            series,
            episode: 1.0
        })
        .err()
        .unwrap(),
        CoreError::NotConnected {
            tracker: Tracker::Anilist
        }
    );
    core.call(Call::SetMainTracker {
        tracker: Tracker::Mal,
    })
    .unwrap();
    assert_eq!(
        core.call(Call::SetProgress {
            series,
            progress: 1,
            season: None,
        })
        .err()
        .unwrap(),
        CoreError::NotConnected {
            tracker: Tracker::Mal
        }
    );
    assert!(http.requests().is_empty());
}

/// A missing series is absent from every tracker write too, and no refusal
/// names it: as far as a write is concerned it is not there.
#[test]
fn a_missing_series_is_not_found() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::mark_missing(&core, series);

    let expected = CoreError::NotFound {
        what: Entity::Series,
        id: series,
    };
    assert_eq!(
        core.call(Call::MarkEpisode {
            series,
            episode: 1.0
        })
        .err()
        .unwrap(),
        expected
    );
    assert_eq!(
        core.call(Call::SetProgress {
            series,
            progress: 1,
            season: None,
        })
        .err()
        .unwrap(),
        expected
    );
    assert_eq!(
        core.call(Call::SetScore {
            series,
            score: Some(8.0)
        })
        .err()
        .unwrap(),
        expected
    );
    // A series that never existed answers the same way.
    assert_eq!(
        core.call(Call::MarkEpisode {
            series: 9999,
            episode: 1.0
        })
        .err()
        .unwrap(),
        CoreError::NotFound {
            what: Entity::Series,
            id: 9999
        }
    );
    assert!(http.requests().is_empty());
}

/// Episode nought or below is the caller's mistake, and so is a score off
/// the ten point scale.
#[test]
fn an_impossible_episode_or_score_is_invalid() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    for episode in [0.0, 0.5, -1.0, f64::NAN] {
        let err = core
            .call(Call::MarkEpisode { series, episode })
            .err()
            .unwrap();
        assert!(
            matches!(&err, CoreError::Invalid { field, .. } if field == "episode"),
            "{episode} gave {err:?}"
        );
    }
    for score in [-0.1, 10.5, f64::INFINITY] {
        let err = core
            .call(Call::SetScore {
                series,
                score: Some(score),
            })
            .err()
            .unwrap();
        assert!(
            matches!(&err, CoreError::Invalid { field, .. } if field == "score"),
            "{score} gave {err:?}"
        );
    }
    assert!(http.requests().is_empty());
}

/// Setting progress is the corrective path, so it takes any value, and
/// nought means the series is back on the plan-to-watch list.
#[test]
fn setting_progress_to_nought_plans_the_series() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":5,"status":"CURRENT"}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"progress":0,"status":"PLANNING"}}}"#,
    );

    let job = started(
        &core,
        Call::SetProgress {
            series,
            progress: 0,
            season: None,
        },
    );
    let done = common::wait_job(&c, job);
    let (progress, outcomes) = match done.body {
        EventBody::ProgressSet {
            progress, outcomes, ..
        } => (progress, outcomes),
        other => panic!("{other:?}"),
    };
    assert_eq!(progress, 0);
    // MAL carries an id but no account, so it is not written to at all.
    assert_eq!(outcomes.len(), 1);
    assert!(outcome(&outcomes, Tracker::Anilist).ok);

    let vars = graphql_variables(&http, "SaveMediaListEntry");
    assert_eq!(
        vars,
        vec![serde_json::json!({ "mediaId": 1, "progress": 0, "status": "PLANNING" })]
    );
    assert!(
        c.events()
            .iter()
            .any(|e| e.message == "anilist set 5 -> 0 (mediaId 1)"),
        "{:#?}",
        c.events()
            .iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(card(&core, series).watched, Some(0));
}

/// A score is one number across both trackers: AniList takes it raw out of
/// a hundred so the decimal survives whatever display format the user
/// picked, MAL takes a whole number. A list already at the last episode is
/// completed by the same call, and one that is not sends no status at all.
#[test]
fn a_score_goes_to_both_trackers_and_completes_a_finished_list() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":12,"status":"CURRENT"}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"score":8.7,"status":"COMPLETED"}}}"#,
    );
    http.push_for(
        "myanimelist",
        200,
        r#"{"my_list_status":{"num_episodes_watched":12}}"#,
    );
    http.push_for("myanimelist", 200, "{}");

    let job = started(
        &core,
        Call::SetScore {
            series,
            score: Some(8.7),
        },
    );
    let done = common::wait_job(&c, job);
    let (score, outcomes) = match done.body {
        EventBody::Scored {
            score, outcomes, ..
        } => (score, outcomes),
        other => panic!("{other:?}"),
    };
    assert_eq!(score, Some(8.7));
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(|o| o.ok), "{outcomes:?}");

    assert_eq!(
        graphql_variables(&http, "SaveMediaListEntry"),
        vec![
            serde_json::json!({ "mediaId": 1, "scoreRaw": 87, "status": "COMPLETED", "completedAt": today().0 })
        ]
    );
    let forms = mal_forms(&http);
    assert_eq!(forms.len(), 1);
    // 8.7 rounds the way MAL's own UI rounds a typed decimal.
    assert!(
        forms[0].contains(&("score".to_string(), "9".to_string())),
        "{:?}",
        forms[0]
    );
    assert!(
        forms[0].contains(&("status".to_string(), "completed".to_string())),
        "{:?}",
        forms[0]
    );
    assert_eq!(card(&core, series).my_score, Some(8.7));

    // A list three episodes in is nowhere near finished, so the mutation
    // carries no status and the form carries no status either.
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":3,"status":"CURRENT"}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"score":5}}}"#,
    );
    http.push_for(
        "myanimelist",
        200,
        r#"{"my_list_status":{"num_episodes_watched":3}}"#,
    );
    http.push_for("myanimelist", 200, "{}");
    let job = started(
        &core,
        Call::SetScore {
            series,
            score: Some(5.0),
        },
    );
    common::wait_job(&c, job);

    let vars = graphql_variables(&http, "SaveMediaListEntry");
    assert_eq!(vars.len(), 2);
    assert_eq!(vars[1], serde_json::json!({ "mediaId": 1, "scoreRaw": 50 }));
    let forms = mal_forms(&http);
    assert_eq!(forms.len(), 2);
    assert!(
        !forms[1].iter().any(|(k, _)| k == "status"),
        "{:?}",
        forms[1]
    );
}

/// A status from the pill's menu is one status across both trackers, sent
/// alone so the progress and the score stay as the lists have them, and the
/// card follows it at once.
#[test]
fn a_status_goes_to_both_trackers_and_the_card_follows() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":3,"status":"CURRENT"}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"status":"DROPPED"}}}"#,
    );
    http.push_for(
        "myanimelist",
        200,
        r#"{"my_list_status":{"num_episodes_watched":3}}"#,
    );
    http.push_for("myanimelist", 200, "{}");

    let job = started(
        &core,
        Call::SetListStatus {
            series,
            status: ListStatus::Dropped,
        },
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.level, Level::Info, "{done:?}");
    let (status, outcomes) = match done.body {
        EventBody::ListStatusSet {
            status, outcomes, ..
        } => (status, outcomes),
        other => panic!("{other:?}"),
    };
    assert_eq!(status, ListStatus::Dropped);
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(|o| o.ok), "{outcomes:?}");

    assert_eq!(
        graphql_variables(&http, "SaveMediaListEntry"),
        vec![serde_json::json!({ "mediaId": 1, "status": "DROPPED" })]
    );
    // MAL keeps a rewatch as a flag beside the status, so the write clears it
    let forms = mal_forms(&http);
    assert_eq!(forms.len(), 1);
    assert_eq!(
        forms[0],
        vec![
            ("status".to_string(), "dropped".to_string()),
            ("is_rewatching".to_string(), "false".to_string()),
        ]
    );
    let card = card(&core, series);
    assert_eq!(card.list_status, Some(ListStatus::Dropped));
    assert_eq!(
        card.watched,
        Some(3),
        "the progress stays as the list has it"
    );
}

/// Completed from the menu fills the list to the published total, on both
/// trackers and in the card, rather than leaving a Completed 3/12.
#[test]
fn completed_fills_the_list_to_the_total() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":3,"status":"CURRENT"}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"status":"COMPLETED"}}}"#,
    );
    http.push_for(
        "myanimelist",
        200,
        r#"{"my_list_status":{"num_episodes_watched":3}}"#,
    );
    http.push_for("myanimelist", 200, "{}");

    let job = started(
        &core,
        Call::SetListStatus {
            series,
            status: ListStatus::Completed,
        },
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.level, Level::Info, "{done:?}");

    assert_eq!(
        graphql_variables(&http, "SaveMediaListEntry"),
        vec![
            serde_json::json!({ "mediaId": 1, "progress": 12, "status": "COMPLETED", "completedAt": today().0 })
        ]
    );
    let forms = mal_forms(&http);
    assert_eq!(forms.len(), 1);
    assert!(
        forms[0].contains(&("num_watched_episodes".to_string(), "12".to_string())),
        "{:?}",
        forms[0]
    );
    let card = card(&core, series);
    assert_eq!(card.list_status, Some(ListStatus::Completed));
    assert_eq!(card.watched, Some(12));
}

/// Repeating is the trackers' own rewatch count, not a status the menu
/// offers, so it is refused before any request is made.
#[test]
fn repeating_is_not_a_status_to_set() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    let err = core
        .call(Call::SetListStatus {
            series,
            status: ListStatus::Repeating,
        })
        .err()
        .unwrap();
    assert!(matches!(err, CoreError::Invalid { .. }), "{err:?}");
    assert!(http.requests().is_empty());
}

/// A rate limit the limiter could not ride out is one tracker's failure,
/// reported in its outcome as a line a user can act on. The job itself
/// still finishes.
#[test]
fn an_exhausted_rate_limit_is_one_sanitised_outcome() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    // Seven replies: the first request and the six retries the schedule
    // allows. `Retry-After: 0` is what makes them run at once.
    for _ in 0..7 {
        http.push_with_headers(
            429,
            "slow down",
            vec![("Retry-After".to_string(), "0".to_string())],
        );
    }

    let job = started(
        &core,
        Call::MarkEpisode {
            series,
            episode: 5.0,
        },
    );
    let done = common::wait_job(&c, job);
    let outcomes = match done.body {
        EventBody::Marked { outcomes, .. } => outcomes,
        other => panic!("{other:?}"),
    };
    let anilist = outcome(&outcomes, Tracker::Anilist);
    assert!(!anilist.ok);
    assert_eq!(
        anilist.message.as_deref(),
        Some("AniList rate limited, try again in a minute.")
    );
    // Nothing was written, so the cache says nothing either.
    assert_eq!(card(&core, series).watched, None);
}

// Dates, rewatches and notes (#211) -------------------------------------------

/// Today as the trackers' dates read it: the UTC day, as AniList's fuzzy
/// date and as MAL's string.
fn today() -> (serde_json::Value, String) {
    let t = ::time::OffsetDateTime::now_utc();
    let (y, m, d) = (t.year(), u8::from(t.month()), t.day());
    (
        serde_json::json!({ "year": y, "month": m, "day": d }),
        format!("{y:04}-{m:02}-{d:02}"),
    )
}

fn form_has(form: &[(String, String)], key: &str, value: &str) -> bool {
    form.contains(&(key.to_string(), value.to_string()))
}

fn form_keys(form: &[(String, String)]) -> Vec<String> {
    form.iter().map(|(k, _)| k.clone()).collect()
}

/// The cached entry's own timestamp, put back by `secs`, as if the list
/// last changed that long ago.
fn age_entry(core: &Core, tracker: &str, media_id: u64, secs: i64) {
    let tracker = tracker.to_string();
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE tracker_entries SET updated_at = updated_at - ?3 WHERE tracker = ?1 AND media_id = ?2",
                rusqlite::params![tracker, media_id as i64, secs],
            )?;
            Ok(())
        })
        .unwrap();
}

/// The first episode watched starts the show on both lists, dated today,
/// and nothing finishes yet.
#[test]
fn the_first_episode_sets_the_start_date_on_both_trackers() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":0,"status":"PLANNING","repeat":0,"startedAt":{"year":null},"completedAt":{"year":null}}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"progress":1,"status":"CURRENT"}}}"#,
    );
    http.push_for("myanimelist", 404, r#"{"error":"not_found"}"#);
    http.push_for("myanimelist", 200, "{}");

    let job = started(
        &core,
        Call::MarkEpisode {
            series,
            episode: 1.0,
        },
    );
    common::wait_job(&c, job);

    let (day, mal_day) = today();
    assert_eq!(
        graphql_variables(&http, "SaveMediaListEntry"),
        vec![
            serde_json::json!({ "mediaId": 1, "progress": 1, "status": "CURRENT", "startedAt": day })
        ]
    );
    let forms = mal_forms(&http);
    assert!(
        form_has(&forms[0], "start_date", &mal_day),
        "{:?}",
        forms[0]
    );
    assert!(
        !form_keys(&forms[0]).contains(&"finish_date".to_string()),
        "{:?}",
        forms[0]
    );
}

/// Completing a show dates the finish where the list has none, and leaves
/// a date the list already has alone; a start date already there stays too.
#[test]
fn completing_sets_the_finish_date_only_where_there_is_none() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    // AniList already has both dates, from a finish somewhere else.
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":11,"status":"CURRENT","repeat":0,"startedAt":{"year":2024},"completedAt":{"year":2024}}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"progress":12,"status":"COMPLETED"}}}"#,
    );
    // MAL has the start and no finish.
    http.push_for(
        "myanimelist",
        200,
        r#"{"my_list_status":{"status":"watching","num_episodes_watched":11,"is_rewatching":false,"num_times_rewatched":0,"start_date":"2026-10-01"}}"#,
    );
    http.push_for("myanimelist", 200, "{}");

    let job = started(
        &core,
        Call::MarkEpisode {
            series,
            episode: 12.0,
        },
    );
    common::wait_job(&c, job);

    assert_eq!(
        graphql_variables(&http, "SaveMediaListEntry"),
        vec![serde_json::json!({ "mediaId": 1, "progress": 12, "status": "COMPLETED" })]
    );
    let (_, mal_day) = today();
    let forms = mal_forms(&http);
    assert!(
        form_has(&forms[0], "finish_date", &mal_day),
        "{:?}",
        forms[0]
    );
    assert!(
        !form_keys(&forms[0]).contains(&"start_date".to_string()),
        "{:?}",
        forms[0]
    );
}

/// The status menu dates what it says: Watching starts the show, Completed
/// finishes it without inventing a start date.
#[test]
fn the_status_menu_dates_watching_and_completed() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    let (day, _) = today();

    for (status, reply) in [
        (ListStatus::Watching, "CURRENT"),
        (ListStatus::Completed, "COMPLETED"),
    ] {
        http.push_for(
            "anilist",
            200,
            r#"{"data":{"MediaList":{"progress":0,"status":"PLANNING","repeat":0,"startedAt":{"year":null},"completedAt":{"year":null}}}}"#,
        );
        http.push_for(
            "anilist",
            200,
            format!(r#"{{"data":{{"SaveMediaListEntry":{{"id":9,"status":"{reply}"}}}}}}"#),
        );
        let job = started(&core, Call::SetListStatus { series, status });
        common::wait_job(&c, job);
    }
    let vars = graphql_variables(&http, "SaveMediaListEntry");
    assert_eq!(
        vars[0],
        serde_json::json!({ "mediaId": 1, "status": "CURRENT", "startedAt": day })
    );
    assert_eq!(
        vars[1],
        serde_json::json!({ "mediaId": 1, "progress": 12, "status": "COMPLETED", "completedAt": day })
    );
}

/// A completed show watched again from an earlier episode and finished
/// counts a rewatch on both lists, and the series page says so.
#[test]
fn finishing_a_completed_show_again_counts_a_rewatch() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 12, "completed", None);
    age_entry(&core, "anilist", 1, 30 * 86_400);
    // Episode 1 finished here since the list last changed: a rewatch.
    fixtures::insert_completed(&core, series, "1", anibeam_core::time::now_secs());

    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":12,"status":"COMPLETED","repeat":1,"startedAt":{"year":2025},"completedAt":{"year":2025}}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"progress":12,"status":"COMPLETED"}}}"#,
    );
    http.push_for(
        "myanimelist",
        200,
        r#"{"my_list_status":{"status":"completed","num_episodes_watched":12,"is_rewatching":false,"num_times_rewatched":0,"start_date":"2025-01-01","finish_date":"2025-02-01"}}"#,
    );
    http.push_for("myanimelist", 200, "{}");

    let job = started(
        &core,
        Call::MarkEpisode {
            series,
            episode: 12.0,
        },
    );
    let done = common::wait_job(&c, job);
    let outcomes = match done.body {
        EventBody::Marked { outcomes, .. } => outcomes,
        other => panic!("{other:?}"),
    };
    assert!(outcomes.iter().all(|o| o.ok), "{outcomes:?}");
    assert_eq!(
        graphql_variables(&http, "SaveMediaListEntry"),
        vec![
            serde_json::json!({ "mediaId": 1, "progress": 12, "status": "COMPLETED", "repeat": 2 })
        ]
    );
    let forms = mal_forms(&http);
    assert!(
        form_has(&forms[0], "num_times_rewatched", "1"),
        "{:?}",
        forms[0]
    );
    assert!(
        form_has(&forms[0], "is_rewatching", "false"),
        "{:?}",
        forms[0]
    );
    assert!(!form_keys(&forms[0]).contains(&"finish_date".to_string()));

    let Reply::SeriesDetail { detail } = core.call(Call::GetSeries { series }).unwrap() else {
        panic!()
    };
    assert_eq!(detail.rewatch_count, Some(2));
}

/// Two marks of one rewatch finish (the rule at the end and the player's
/// button, say) count one rewatch: the second finds the first still writing
/// and answers with its job, or, once that job is done, finds the list
/// covering the finish and is refused. The cached progress sits below the
/// last episode, so a plain mark would not be covered and its write would
/// race the rewatch's (#1042).
#[test]
fn a_second_mark_of_one_rewatch_finish_counts_one_rewatch() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 11, "completed", None);
    age_entry(&core, "anilist", 1, 30 * 86_400);
    fixtures::insert_completed(&core, series, "1", anibeam_core::time::now_secs());

    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":11,"status":"COMPLETED","repeat":1,"startedAt":{"year":2025},"completedAt":{"year":2025}}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"progress":12,"status":"COMPLETED"}}}"#,
    );

    let mark = Call::MarkEpisode {
        series,
        episode: 12.0,
    };
    let job = started(&core, mark.clone());
    let second = core.call(mark);
    let done = common::wait_job(&c, job);
    assert!(matches!(done.body, EventBody::Marked { .. }), "{done:?}");
    match second {
        Ok(Reply::Started { job: same }) => assert_eq!(same, job, "the first mark's job"),
        Err(CoreError::Refused {
            reason: Refusal::NotNewer,
        }) => {}
        other => panic!("{other:?}"),
    }
    assert_eq!(
        graphql_variables(&http, "SaveMediaListEntry"),
        vec![
            serde_json::json!({ "mediaId": 1, "progress": 12, "status": "COMPLETED", "repeat": 2 })
        ]
    );
}

/// A stand-in AniList that keeps one list entry and answers from it: a read
/// gets the entry as it stands, a save changes it. The save that counts a
/// rewatch is held until another save has landed, or for half a second
/// when none comes, so a write that does not wait its turn lands first.
struct HeldRewatch {
    entry: std::sync::Mutex<serde_json::Value>,
    landed: tokio::sync::Notify,
}

impl HeldRewatch {
    fn save(&self, variables: &serde_json::Value) -> serde_json::Value {
        let mut entry = self.entry.lock().unwrap();
        for key in ["progress", "status", "repeat"] {
            if let Some(v) = variables.get(key) {
                entry[key] = v.clone();
            }
        }
        serde_json::json!({ "data": { "SaveMediaListEntry": { "id": 9, "status": entry["status"] } } })
    }
}

impl Http for HeldRewatch {
    fn send(&self, req: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, HttpError>> {
        Box::pin(async move {
            let reply = |status: u16, body: String| HttpResponse {
                status,
                headers: vec![],
                body: body.into_bytes(),
            };
            let Some(Body::Json(v)) = req.body.as_ref().filter(|_| req.url.contains("anilist"))
            else {
                return Ok(reply(500, format!("no canned reply for {}", req.url)));
            };
            let query = v["query"].as_str().unwrap_or_default();
            let variables = &v["variables"];
            if !query.contains("SaveMediaListEntry") {
                let entry = self.entry.lock().unwrap().clone();
                return Ok(reply(
                    200,
                    serde_json::json!({ "data": { "MediaList": entry } }).to_string(),
                ));
            }
            if variables.get("repeat").is_some() {
                let _ =
                    tokio::time::timeout(Duration::from_millis(500), self.landed.notified()).await;
            }
            let body = self.save(variables).to_string();
            self.landed.notify_one();
            Ok(reply(200, body))
        })
    }
}

/// A status set while a rewatch is being written waits for it (#1112). Both
/// read the list as Completed before either writes, so without the wait the
/// status lands first and the rewatch's Completed lands over it. In turn,
/// the rewatch counts once and the status asked for last is what the list
/// says.
#[test]
fn a_status_set_during_a_rewatch_write_waits_for_it() {
    let anilist = Arc::new(HeldRewatch {
        entry: std::sync::Mutex::new(serde_json::json!({
            "progress": 11, "status": "COMPLETED", "repeat": 1,
            "startedAt": { "year": 2025 }, "completedAt": { "year": 2025 }
        })),
        landed: tokio::sync::Notify::new(),
    });
    let (_dir, core, c) = common::open_core_with_http(anilist.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 11, "completed", None);
    age_entry(&core, "anilist", 1, 30 * 86_400);
    fixtures::insert_completed(&core, series, "1", anibeam_core::time::now_secs());

    let mark = started(
        &core,
        Call::MarkEpisode {
            series,
            episode: 12.0,
        },
    );
    let status = started(
        &core,
        Call::SetListStatus {
            series,
            status: ListStatus::Watching,
        },
    );
    common::wait_job(&c, mark);
    common::wait_job(&c, status);

    let entry = anilist.entry.lock().unwrap().clone();
    assert_eq!(entry["repeat"], 2, "one rewatch counted: {entry}");
    assert_eq!(
        entry["status"], "CURRENT",
        "the status asked for last: {entry}"
    );
}

/// Playing only the last episode of a completed show again is not a
/// rewatch: the mark is refused before a request, as any covered mark is.
#[test]
fn the_last_episode_alone_again_is_not_a_rewatch() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::insert_episode_file(
        &core,
        series,
        "/lib/Bebop/12.mkv",
        12.0,
        anibeam_core::time::now_secs(),
    );
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 12, "completed", None);
    age_entry(&core, "anilist", 1, 30 * 86_400);
    fixtures::insert_completed(&core, series, "12", anibeam_core::time::now_secs());

    let err = core
        .call(Call::MarkEpisode {
            series,
            episode: 12.0,
        })
        .err()
        .unwrap();
    assert_eq!(
        err,
        CoreError::Refused {
            reason: Refusal::NotNewer
        }
    );
    assert!(http.requests().is_empty());
}

/// A rewatch under way stays one: a mark keeps AniList's Repeating rather
/// than turning it back into a first watch, and the last episode completes
/// it with one more rewatch counted.
#[test]
fn a_rewatch_under_way_keeps_repeating_and_its_end_counts() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    for (from, to) in [(3, 4.0), (11, 12.0)] {
        http.push_for(
            "anilist",
            200,
            format!(
                r#"{{"data":{{"MediaList":{{"progress":{from},"status":"REPEATING","repeat":2,"startedAt":{{"year":2025}},"completedAt":{{"year":2025}}}}}}}}"#
            ),
        );
        http.push_for(
            "anilist",
            200,
            r#"{"data":{"SaveMediaListEntry":{"id":9}}}"#,
        );
        let job = started(
            &core,
            Call::MarkEpisode {
                series,
                episode: to,
            },
        );
        common::wait_job(&c, job);
    }
    let vars = graphql_variables(&http, "SaveMediaListEntry");
    assert_eq!(
        vars,
        vec![
            serde_json::json!({ "mediaId": 1, "progress": 4, "status": "REPEATING" }),
            serde_json::json!({ "mediaId": 1, "progress": 12, "status": "COMPLETED", "repeat": 3 }),
        ]
    );
}

/// Notes go to both trackers, AniList's notes and MAL's comments, the
/// series page reads them back, and a blank clears them.
#[test]
fn notes_are_written_to_both_trackers_and_read_back() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    for _ in 0..2 {
        http.push_for(
            "anilist",
            200,
            r#"{"data":{"MediaList":{"progress":3,"status":"CURRENT","repeat":0}}}"#,
        );
        http.push_for(
            "anilist",
            200,
            r#"{"data":{"SaveMediaListEntry":{"id":9,"notes":"x"}}}"#,
        );
        http.push_for(
            "myanimelist",
            200,
            r#"{"my_list_status":{"status":"watching","num_episodes_watched":3}}"#,
        );
        http.push_for("myanimelist", 200, "{}");
    }

    let job = started(
        &core,
        Call::SetSeriesNotes {
            series,
            notes: "  Watch the dub next time  ".into(),
        },
    );
    let done = common::wait_job(&c, job);
    match done.body {
        EventBody::NotesSet {
            notes, outcomes, ..
        } => {
            assert_eq!(notes.as_deref(), Some("Watch the dub next time"));
            assert_eq!(outcomes.len(), 2);
            assert!(outcomes.iter().all(|o| o.ok), "{outcomes:?}");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        graphql_variables(&http, "notes: $notes"),
        vec![serde_json::json!({ "mediaId": 1, "notes": "Watch the dub next time" })]
    );
    assert_eq!(
        mal_forms(&http)[0],
        vec![(
            "comments".to_string(),
            "Watch the dub next time".to_string()
        )]
    );
    let detail = |core: &Core| match core.call(Call::GetSeries { series }).unwrap() {
        Reply::SeriesDetail { detail } => detail,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        detail(&core).notes.as_deref(),
        Some("Watch the dub next time")
    );

    let job = started(
        &core,
        Call::SetSeriesNotes {
            series,
            notes: "   ".into(),
        },
    );
    let done = common::wait_job(&c, job);
    assert!(matches!(done.body, EventBody::NotesSet { notes: None, .. }));
    assert_eq!(
        graphql_variables(&http, "notes: $notes")[1],
        serde_json::json!({ "mediaId": 1, "notes": "" })
    );
    assert_eq!(detail(&core).notes, None);
}

/// A note past MAL's ceiling is refused before anything is sent.
#[test]
fn a_note_too_long_is_invalid() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    let err = core
        .call(Call::SetSeriesNotes {
            series,
            notes: "n".repeat(2001),
        })
        .err()
        .unwrap();
    assert!(matches!(err, CoreError::Invalid { .. }), "{err:?}");
    assert!(http.requests().is_empty());
}

/// The status menu says where the list stands and never counts a rewatch:
/// Completed from Repeating sends no count on either tracker.
#[test]
fn completed_from_the_menu_ends_a_rewatch_without_counting_one() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");

    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":12,"status":"REPEATING","repeat":2,"startedAt":{"year":2025},"completedAt":{"year":2025}}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9}}}"#,
    );
    http.push_for(
        "myanimelist",
        200,
        r#"{"my_list_status":{"status":"watching","num_episodes_watched":12,"is_rewatching":true,"num_times_rewatched":2,"start_date":"2025-01-01","finish_date":"2025-02-01"}}"#,
    );
    http.push_for("myanimelist", 200, "{}");
    let job = started(
        &core,
        Call::SetListStatus {
            series,
            status: ListStatus::Completed,
        },
    );
    common::wait_job(&c, job);
    assert_eq!(
        graphql_variables(&http, "SaveMediaListEntry"),
        vec![serde_json::json!({ "mediaId": 1, "status": "COMPLETED" })]
    );
    let forms = mal_forms(&http);
    assert!(
        !form_keys(&forms[0]).contains(&"num_times_rewatched".to_string()),
        "{:?}",
        forms[0]
    );
}

/// A show the person put on Completed by hand partway through, then
/// finished here from where they were, was watched once: only a run from
/// the first episode is a rewatch, and a list with no stamp is no evidence.
#[test]
fn finishing_after_completing_by_hand_is_not_a_rewatch() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    let now = anibeam_core::time::now_secs();
    fixtures::insert_episode_file(&core, series, "/lib/Bebop/11.mkv", 11.0, now);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 12, "completed", None);
    age_entry(&core, "anilist", 1, 30 * 86_400);
    fixtures::insert_completed(&core, series, "11", now);
    let refused = |core: &Core| {
        core.call(Call::MarkEpisode {
            series,
            episode: 12.0,
        })
        .err()
            == Some(CoreError::Refused {
                reason: Refusal::NotNewer,
            })
    };
    assert!(refused(&core));

    // Episode 1 again, but the cached entry carries no stamp to compare.
    fixtures::insert_completed(&core, series, "1", now);
    core.store()
        .write(|c| {
            c.execute("UPDATE tracker_entries SET updated_at = NULL", [])?;
            Ok(())
        })
        .unwrap();
    assert!(refused(&core));
    assert!(http.requests().is_empty());
}

/// A film finished, then marked again the same evening, is the same
/// viewing; a day later it is a rewatch.
#[test]
fn a_film_counts_again_only_once_the_list_has_stood_still() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(&core, "/lib");
    let film = fixtures::insert_series(&core, src, SeriesKind::Movie, "/lib/Film.mkv", "Film");
    fixtures::insert_episode_file(&core, film, "/lib/Film.mkv", 1.0, now);
    fixtures::media(5, "Film")
        .episodes(1)
        .format("MOVIE")
        .insert(&core);
    fixtures::match_series(&core, film, Some(5), None);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 5, 1, "completed", None);

    let mark = |core: &Core| {
        core.call(Call::MarkEpisode {
            series: film,
            episode: 1.0,
        })
    };
    assert!(mark(&core).is_err());

    age_entry(&core, "anilist", 5, 86_400);
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":1,"status":"COMPLETED","repeat":0,"startedAt":{"year":2025},"completedAt":{"year":2025}}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9}}}"#,
    );
    let job = job_of(mark(&core).unwrap());
    common::wait_job(&c, job);
    assert_eq!(
        graphql_variables(&http, "SaveMediaListEntry"),
        vec![
            serde_json::json!({ "mediaId": 5, "progress": 1, "status": "COMPLETED", "repeat": 1 })
        ]
    );
    // The count moved the list's stamp, so the same viewing marked again
    // is refused before a request.
    assert!(mark(&core).is_err());
    assert_eq!(graphql_variables(&http, "SaveMediaListEntry").len(), 1);
}

/// Notes on a show the list does not hold would add it to the list, so
/// that tracker says so and nothing is written.
#[test]
fn notes_on_a_show_not_on_the_list_write_nothing() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    http.push_for(
        "anilist",
        404,
        r#"{"errors":[{"message":"Not Found.","status":404}],"data":{"MediaList":null}}"#,
    );
    let job = started(
        &core,
        Call::SetSeriesNotes {
            series,
            notes: "later".into(),
        },
    );
    let done = common::wait_job(&c, job);
    match done.body {
        EventBody::NotesSet { outcomes, .. } => {
            assert!(!outcomes[0].ok);
            assert!(
                outcomes[0]
                    .message
                    .as_deref()
                    .unwrap_or("")
                    .contains("Add it to a list"),
                "{outcomes:?}"
            );
        }
        other => panic!("{other:?}"),
    }
    assert!(graphql_variables(&http, "notes: $notes").is_empty());
}

// MAL behind, and MAL ids AniList does not carry (#782) ----------------------

/// The cheap guard reads every tracker the mark reaches, not only the main
/// one: a list behind on either side still gets its write, and the tracker
/// already there answers NotNewer from the job.
#[test]
fn a_mark_one_tracker_already_has_still_reaches_the_other() {
    for (ahead, behind) in [
        (Tracker::Anilist, Tracker::Mal),
        (Tracker::Mal, Tracker::Anilist),
    ] {
        let http = FakeHttp::new();
        let (_dir, core, c) = common::open_core_with_http(http.clone());
        let series = library(&core);
        fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
        fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");
        let id = |t: Tracker| if t == Tracker::Anilist { 1 } else { 21 };
        fixtures::insert_tracker_entry(&core, ahead.as_str(), id(ahead), 5, "current", None);
        fixtures::insert_tracker_entry(&core, behind.as_str(), id(behind), 2, "current", None);

        let (anilist_at, mal_at) = match ahead {
            Tracker::Anilist => (5, 2),
            Tracker::Mal => (2, 5),
        };
        http.push_for(
            "anilist",
            200,
            format!(r#"{{"data":{{"MediaList":{{"progress":{anilist_at},"status":"CURRENT"}}}}}}"#),
        );
        if behind == Tracker::Anilist {
            http.push_for(
                "anilist",
                200,
                r#"{"data":{"SaveMediaListEntry":{"id":9,"progress":5,"status":"CURRENT"}}}"#,
            );
        }
        http.push_for(
            "myanimelist",
            200,
            format!(
                r#"{{"my_list_status":{{"status":"watching","num_episodes_watched":{mal_at}}}}}"#
            ),
        );
        if behind == Tracker::Mal {
            http.push_for("myanimelist", 200, "{}");
        }

        let job = started(
            &core,
            Call::MarkEpisode {
                series,
                episode: 5.0,
            },
        );
        let outcomes = match common::wait_job(&c, job).body {
            EventBody::Marked { outcomes, .. } => outcomes,
            other => panic!("{other:?}"),
        };
        let behind_outcome = outcome(&outcomes, behind);
        assert!(behind_outcome.ok, "{ahead:?} ahead: {outcomes:?}");
        assert_eq!(behind_outcome.progress, Some(5));
        assert_eq!(outcome(&outcomes, ahead).reason, Some(Refusal::NotNewer));
        let mal_writes = mal_forms(&http);
        match behind {
            Tracker::Mal => {
                assert_eq!(mal_writes.len(), 1);
                assert!(form_has(&mal_writes[0], "num_watched_episodes", "5"));
            }
            Tracker::Anilist => {
                assert!(mal_writes.is_empty());
                assert_eq!(graphql_variables(&http, "SaveMediaListEntry").len(), 1);
            }
        }
    }
}

/// Every tracker the mark reaches already covering it is still refused at
/// once, without a request.
#[test]
fn a_mark_both_trackers_already_have_is_refused_at_once() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mtok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 5, "current", None);
    fixtures::insert_tracker_entry(&core, "mal", 21, 6, "watching", None);

    assert_eq!(
        core.call(Call::MarkEpisode {
            series,
            episode: 5.0
        })
        .err()
        .unwrap(),
        CoreError::Refused {
            reason: Refusal::NotNewer
        }
    );
    assert!(http.requests().is_empty());
}

/// A series AniList gave no MAL id, with only MAL connected.
fn anilist_only(core: &Core) -> u64 {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    fixtures::insert_episode_file(core, series, "/lib/Bebop/01.mkv", 1.0, now);
    fixtures::media(1, "Cowboy Bebop").episodes(26).insert(core);
    fixtures::match_series(core, series, Some(1), None);
    fixtures::connect_tracker(core, Tracker::Mal, 7, "mtok");
    series
}

fn no_match() -> TrackerOutcome {
    TrackerOutcome {
        tracker: Tracker::Mal,
        ok: false,
        progress: None,
        reason: Some(Refusal::NoMatch),
        message: None,
    }
}

fn searches(http: &FakeHttp) -> usize {
    http.requests()
        .iter()
        .filter(|r| r.url.contains("/anime?q="))
        .count()
}

fn saved_lookup(core: &Core) -> Option<Option<i64>> {
    core.store()
        .read(|c| {
            use rusqlite::OptionalExtension;
            Ok(c.query_row(
                "SELECT mal_id FROM mal_lookups WHERE anilist_id = 1",
                [],
                |r| r.get::<_, Option<i64>>(0),
            )
            .optional()?)
        })
        .unwrap()
}

fn mark(
    core: &Core,
    c: &anibeam_core::events::Collector,
    series: u64,
    episode: f64,
) -> Vec<TrackerOutcome> {
    let job = started(core, Call::MarkEpisode { series, episode });
    match common::wait_job(c, job).body {
        EventBody::Marked { outcomes, .. } => outcomes,
        other => panic!("{other:?}"),
    }
}

/// The first mark of a show with no MAL id searches MAL once, takes the one
/// result whose title and type agree, writes to it and keeps the answer, so
/// the next mark goes straight to the entry.
#[test]
fn a_show_with_no_mal_id_is_found_by_search_once() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = anilist_only(&core);

    http.push_for(
        "/anime?q=",
        200,
        r#"{"data":[
            {"node":{"id":21,"title":"Cowboy Bebop","alternative_titles":{"en":"Cowboy Bebop","ja":"","synonyms":[]},"start_date":"1998-04-03","media_type":"tv"}},
            {"node":{"id":5,"title":"Cowboy Bebop: Tengoku no Tobira","alternative_titles":{"en":"Cowboy Bebop: The Movie","ja":"","synonyms":[]},"start_date":"2001-09-01","media_type":"movie"}}
        ]}"#,
    );
    http.push_for("/anime/21", 404, r#"{"error":"not_found"}"#);
    http.push_for("/anime/21", 200, "{}");

    let outcomes = mark(&core, &c, series, 3.0);
    let mal = outcome(&outcomes, Tracker::Mal);
    assert!(mal.ok, "{outcomes:?}");
    assert_eq!(mal.progress, Some(3));
    assert_eq!(searches(&http), 1);
    let search = http
        .requests()
        .into_iter()
        .find(|r| r.url.contains("/anime?q="))
        .unwrap();
    assert!(search.url.contains("q=Cowboy%20Bebop"), "{}", search.url);
    assert!(form_has(&mal_forms(&http)[0], "num_watched_episodes", "3"));
    assert_eq!(saved_lookup(&core), Some(Some(21)));

    // The next mark goes by the saved id and never searches again.
    http.push_for(
        "/anime/21",
        200,
        r#"{"my_list_status":{"status":"watching","num_episodes_watched":3}}"#,
    );
    http.push_for("/anime/21", 200, "{}");
    let outcomes = mark(&core, &c, series, 4.0);
    assert!(outcome(&outcomes, Tracker::Mal).ok, "{outcomes:?}");
    assert_eq!(searches(&http), 1);
    assert_eq!(mal_forms(&http).len(), 2);
}

/// A search with no sure answer writes nowhere, says so as MAL's NoMatch,
/// and is kept as no answer, so the show is not searched again and the next
/// mark is refused at once.
#[test]
fn a_search_with_no_sure_answer_is_not_asked_again() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = anilist_only(&core);

    // Two results that both fit are as good as none.
    http.push_for(
        "/anime?q=",
        200,
        r#"{"data":[
            {"node":{"id":21,"title":"Cowboy Bebop","media_type":"tv"}},
            {"node":{"id":22,"title":"Cowboy Bebop","media_type":"tv"}}
        ]}"#,
    );
    let outcomes = mark(&core, &c, series, 3.0);
    assert_eq!(outcomes, vec![no_match()]);
    assert!(mal_forms(&http).is_empty());
    assert_eq!(saved_lookup(&core), Some(None));

    let requests = http.requests().len();
    assert_eq!(
        core.call(Call::MarkEpisode {
            series,
            episode: 4.0
        })
        .err()
        .unwrap(),
        CoreError::NotConnected {
            tracker: Tracker::Anilist
        }
    );
    assert_eq!(http.requests().len(), requests);
}

/// A search that fails is MAL's failed outcome and keeps nothing, so the
/// next mark searches again.
#[test]
fn a_failed_search_is_tried_again_next_time() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = anilist_only(&core);

    http.push_for("/anime?q=", 500, "oops");
    let outcomes = mark(&core, &c, series, 3.0);
    let mal = outcome(&outcomes, Tracker::Mal);
    assert!(!mal.ok);
    assert!(mal.message.is_some());
    assert_eq!(saved_lookup(&core), None);

    http.push_for("/anime?q=", 200, r#"{"data":[]}"#);
    let outcomes = mark(&core, &c, series, 3.0);
    assert_eq!(outcomes, vec![no_match()]);
    assert_eq!(searches(&http), 2);
    assert_eq!(saved_lookup(&core), Some(None));
}

/// A miss stands for a week: a show marked before MAL listed it is searched
/// again after that.
#[test]
fn a_week_old_miss_is_searched_again() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = anilist_only(&core);

    http.push_for("/anime?q=", 200, r#"{"data":[]}"#);
    assert_eq!(mark(&core, &c, series, 3.0), vec![no_match()]);
    core.store()
        .write(|c| {
            c.execute(
                "UPDATE mal_lookups SET looked_at = looked_at - 8 * 24 * 3600",
                [],
            )?;
            Ok(())
        })
        .unwrap();

    http.push_for(
        "/anime?q=",
        200,
        r#"{"data":[{"node":{"id":21,"title":"Cowboy Bebop","media_type":"tv"}}]}"#,
    );
    http.push_for("/anime/21", 404, r#"{"error":"not_found"}"#);
    http.push_for("/anime/21", 200, "{}");
    let outcomes = mark(&core, &c, series, 3.0);
    assert!(outcome(&outcomes, Tracker::Mal).ok, "{outcomes:?}");
    assert_eq!(searches(&http), 2);
    assert_eq!(saved_lookup(&core), Some(Some(21)));
}

/// A score on one series searches too, rather than being kept here because
/// MAL has no id yet.
#[test]
fn a_score_on_a_show_with_no_mal_id_searches() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let series = anilist_only(&core);

    http.push_for(
        "/anime?q=",
        200,
        r#"{"data":[{"node":{"id":21,"title":"Cowboy Bebop","media_type":"tv"}}]}"#,
    );
    http.push_for("/anime/21", 404, r#"{"error":"not_found"}"#);
    http.push_for("/anime/21", 200, "{}");
    let job = started(
        &core,
        Call::SetScore {
            series,
            score: Some(8.0),
        },
    );
    let outcomes = match common::wait_job(&c, job).body {
        EventBody::Scored { outcomes, .. } => outcomes,
        other => panic!("{other:?}"),
    };
    assert!(outcome(&outcomes, Tracker::Mal).ok, "{outcomes:?}");
    assert_eq!(searches(&http), 1);
    assert_eq!(mal_forms(&http).len(), 1);
}
