//! Marks, progress and scores: the guards that answer before a request is
//! made, the writes that go to every connected tracker, and what a failed
//! tracker leaves behind. Both trackers are connected through the fixtures
//! rather than through a flow, so these tests are about the writes alone.

mod common;
mod fixtures;

use anibeam_core::net::{Body, FakeHttp, Method};
use anibeam_core::*;

fn started(reply: Reply) -> u64 {
    match reply {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

/// One matched series, both ids carried, twelve episodes published, and the
/// file it needs to be listed at all.
fn library(core: &Core) -> u64 {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    fixtures::insert_file(core, series, "/lib/Bebop/01.mkv", 1.0, None, "episode", now);
    fixtures::insert_media(
        core,
        1,
        Some("Cowboy Bebop"),
        None,
        Some(12),
        "FINISHED",
        "TV",
        Some(86),
    );
    fixtures::match_series(core, series, Some(1), Some(21));
    series
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
        core.call(Call::MarkEpisode {
            series,
            episode: 12.5,
        })
        .unwrap(),
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
    assert_eq!(done.message, "marked episode 12 of Cowboy Bebop");
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
    assert_eq!(
        vars[0],
        serde_json::json!({ "mediaId": 1, "progress": 12, "status": "COMPLETED" })
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
            .any(|e| e.message == "anilist 11 -> 12 (mediaId 1)"),
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
        },
        Call::SetScore {
            series,
            score: Some(8.0),
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

/// Nothing to write to: an unmatched series carries neither id.
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
    assert_eq!(
        core.call(Call::SetProgress {
            series,
            progress: 1
        })
        .err()
        .unwrap(),
        CoreError::Refused {
            reason: Refusal::NoMatch
        }
    );
    assert_eq!(
        core.call(Call::SetScore {
            series,
            score: None
        })
        .err()
        .unwrap(),
        CoreError::Refused {
            reason: Refusal::NoMatch
        }
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
            progress: 1
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
            progress: 1
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
        core.call(Call::SetProgress {
            series,
            progress: 0,
        })
        .unwrap(),
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
        core.call(Call::SetScore {
            series,
            score: Some(8.7),
        })
        .unwrap(),
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
        vec![serde_json::json!({ "mediaId": 1, "scoreRaw": 87, "status": "COMPLETED" })]
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
        core.call(Call::SetScore {
            series,
            score: Some(5.0),
        })
        .unwrap(),
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
        core.call(Call::MarkEpisode {
            series,
            episode: 5.0,
        })
        .unwrap(),
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
