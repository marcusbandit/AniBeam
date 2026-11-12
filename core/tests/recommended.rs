//! Recommended for you (#779): a cold read of everything the recommender
//! needs, a ranking held to what the user has yet to watch, three scopes,
//! a second read that sends nothing, and no spoiler tag named anywhere.

mod common;
mod fixtures;

use std::sync::Arc;
use std::time::Duration;

use anibeam_core::events::Collector;
use anibeam_core::net::{FakeHttp, Method};
use anibeam_core::*;
use serde_json::{Value, json};

/// The spoiler tag the user loves. It must never appear in a reply.
const SPOILER: &str = "Twist Villain";

fn listed(core: &Core, scope: RecommendedScope) -> Recommended {
    match core.call(Call::ListRecommended { scope }).unwrap() {
        Reply::Recommended { recommended, .. } => recommended,
        other => panic!("{other:?}"),
    }
}

fn ids(r: &Recommended) -> Vec<u64> {
    let mut ids: Vec<u64> = r.shows.iter().map(|s| s.anilist_id).collect();
    ids.sort_unstable();
    ids
}

fn anilist_posts(http: &FakeHttp) -> usize {
    http.requests()
        .iter()
        .filter(|r| r.method == Method::Post && r.url.contains("graphql.anilist.co"))
        .count()
}

fn media(id: u64, extra: Value) -> Value {
    let mut m = json!({
        "id": id, "idMal": null, "status": "FINISHED", "format": "TV", "episodes": 12,
        "duration": 24, "seasonYear": 2020, "averageScore": 75, "popularity": 1000,
        "isAdult": false, "genres": ["Drama"],
        "description": format!("Show {id} begins here. It goes on.<br>(Source: somewhere)"),
        "siteUrl": format!("https://anilist.co/anime/{id}"),
        "title": { "romaji": format!("Show {id}"), "english": null, "native": null },
        "coverImage": { "large": format!("https://img.test/{id}.jpg"), "color": "#336699" },
        "tags": [], "studios": { "nodes": [] }, "relations": { "edges": [] },
        "staff": { "edges": [
            { "role": "Director", "node": { "id": 900 + id % 2, "name": { "full": "D" } } }
        ] },
    });
    if let (Value::Object(m), Value::Object(extra)) = (&mut m, extra) {
        m.extend(extra);
    }
    m
}

/// The user loves a spoiler tag: their 9s carry it.
fn spoiler_tag() -> Value {
    json!([{ "name": SPOILER, "rank": 90, "category": "Theme", "isMediaSpoiler": true }])
}

/// My twenty rated shows: even ids at 9, odd ids at 5. The 9s carry the
/// spoiler tag the user loves.
fn my_show(id: u64) -> Value {
    if id.is_multiple_of(2) {
        media(id, json!({ "tags": spoiler_tag() }))
    } else {
        media(id, json!({}))
    }
}

fn my_score(id: u64) -> f64 {
    if id.is_multiple_of(2) { 9.0 } else { 5.0 }
}

/// Three people who rate my shows the way I do and love every candidate.
fn their_list() -> Value {
    let mut entries: Vec<Value> = (1..=20)
        .map(|id| json!({ "mediaId": id, "status": "COMPLETED", "score": my_score(id) * 10.0, "repeat": 0 }))
        .collect();
    for id in [200, 201, 202, 203, 204, 205] {
        entries.push(json!({ "mediaId": id, "status": "COMPLETED", "score": 95, "repeat": 0 }));
    }
    json!({ "lists": [{ "isCustomList": false, "entries": entries }] })
}

fn queue_cold(http: &FakeHttp) {
    // 1. My shows, the library's and the list's: one request of 24.
    let mut mine: Vec<Value> = (1..=20).map(my_show).collect();
    for id in [50, 51, 52, 60] {
        mine.push(media(id, json!({})));
    }
    http.push_json(200, json!({ "data": { "Page": { "media": mine } } }));
    // 2. The finishers of my twenty favourites, ten a request. Every alias
    // in both replies, since the order the favourites go out in is the
    // recommender's own.
    let raters: serde_json::Map<String, Value> = (1..=20)
        .map(|id| {
            (
                format!("m{id}"),
                json!({ "mediaList": [
                    { "userId": 100, "score": my_score(id) * 10.0 },
                    { "userId": 101, "score": my_score(id) * 10.0 },
                    { "userId": 102, "score": my_score(id) * 10.0 },
                    // The user themselves, who is never their own neighbour
                    { "userId": 42, "score": my_score(id) * 10.0 },
                ] }),
            )
        })
        .collect();
    for _ in 0..2 {
        http.push_json(200, json!({ "data": raters }));
    }
    // 3. Their lists, one request of three.
    http.push_json(
        200,
        json!({ "data": { "u100": their_list(), "u101": their_list(), "u102": their_list() } }),
    );
    // 4. AniList's "if you liked this" for the favourites, ten a request.
    let similar: serde_json::Map<String, Value> = (1..=20)
        .map(|id| {
            let nodes = if id == 2 {
                json!([{ "rating": 12, "mediaRecommendation": { "id": 300, "type": "ANIME" } }])
            } else {
                json!([])
            };
            (
                format!("m{id}"),
                json!({ "recommendations": { "nodes": nodes } }),
            )
        })
        .collect();
    for _ in 0..2 {
        http.push_json(200, json!({ "data": similar }));
    }
    // 5. The candidates.
    http.push_json(
        200,
        json!({ "data": { "Page": { "media": [
            media(200, json!({})),
            // A sequel of a show the user has not seen: not a place to start
            media(201, json!({ "relations": { "edges": [
                { "relationType": "PREQUEL", "node": { "id": 202, "type": "ANIME" } } ] } })),
            media(202, json!({})),
            // The next season of a show the user finished
            media(203, json!({ "relations": { "edges": [
                { "relationType": "PREQUEL", "node": { "id": 1, "type": "ANIME" } } ] } })),
            // Behind it, the spoiler tag the user loves
            media(204, json!({ "tags": spoiler_tag() })),
            media(205, json!({ "isAdult": true })),
            media(300, json!({})),
        ] } } }),
    );
    for id in [200, 204, 2] {
        http.push_for(
            &format!("img.test/{id}.jpg"),
            200,
            vec![0xFF, 0xD8, 0xFF, 0xE0],
        );
    }
}

type World = (common::Dir, Arc<Core>, Arc<Collector>, Arc<FakeHttp>, u64);

fn setup() -> World {
    let http = FakeHttp::new();
    queue_cold(&http);
    let (dir, core, c) = common::open_core_with_http(http.clone());
    core.set_recommender_gap(Duration::ZERO);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    for id in 1..=20 {
        fixtures::insert_tracker_entry(&core, "anilist", id, 12, "completed", Some(my_score(id)));
    }
    fixtures::insert_tracker_entry(&core, "anilist", 50, 0, "planning", None);
    // Barely started, so still to watch; then well into it, so not
    fixtures::insert_tracker_entry(&core, "anilist", 51, 1, "watching", None);
    fixtures::insert_tracker_entry(&core, "anilist", 52, 8, "watching", None);
    let src = fixtures::insert_source(&core, "/lib");
    let owned = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Sixty", "Sixty");
    fixtures::insert_episode_file(&core, owned, "/lib/Sixty/01.mkv", 1.0, 1);
    fixtures::media(60, "Sixty").episodes(12).insert(&core);
    fixtures::match_series(&core, owned, Some(60), None);
    (dir, core, c, http, owned)
}

fn run(core: &Core, c: &Collector, force: bool) {
    let job = match core.call(Call::RefreshRecommended { force }).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    let done = common::wait_job(c, job);
    assert_eq!(done.body, EventBody::RecommendedRefreshed, "{done:?}");
}

#[test]
fn a_cold_read_ranks_what_the_user_has_yet_to_watch() {
    let (_dir, core, c, http, owned) = setup();

    let before = listed(&core, RecommendedScope::All);
    assert_eq!(before.state, RecommendedState::NeverFitted);
    assert!(before.shows.is_empty());
    assert_eq!(anilist_posts(&http), 0, "a read asks AniList nothing");

    run(&core, &c, false);
    assert_eq!(
        anilist_posts(&http),
        7,
        "shows, two of raters, lists, two of similar, candidates"
    );
    let first = format!("{:?}", http.requests()[0].body);
    assert!(
        !http.requests()[0]
            .headers
            .iter()
            .any(|(k, _)| k == "Authorization"),
        "a public read sends no token: {first}"
    );

    let all = listed(&core, RecommendedScope::All);
    assert_eq!(all.state, RecommendedState::Ready);
    assert!(all.fitted_at.is_some());
    // 201 follows a show not seen, 205 is adult, 52 is well under way, and
    // everything rated is watched.
    assert_eq!(ids(&all), vec![50, 51, 60, 200, 202, 203, 204, 300]);
    assert_eq!(all.counts.all, 8);
    assert_eq!(all.counts.in_library, 1);
    assert_eq!(all.counts.on_list, 2);

    let library = listed(&core, RecommendedScope::InLibrary);
    assert_eq!(ids(&library), vec![60]);
    assert_eq!(library.shows[0].owned, Some(owned));
    let on_list = listed(&core, RecommendedScope::OnList);
    assert_eq!(ids(&on_list), vec![50, 51]);
    assert!(on_list.shows[0].list_status.is_some());

    let pick = all.shows.iter().find(|s| s.anilist_id == 204).unwrap();
    // It beats every 5 on the list, and the 9s carry the same spoiler tag.
    assert_eq!(pick.forecast, 50, "people like the user love it");
    assert_eq!(pick.line, "Show 204 begins here.");
    assert_eq!(pick.color.as_deref(), Some("#336699"));
    assert!(pick.poster.is_some(), "the cover is a local file");
    assert!(pick.hidden_loved >= 1, "the spoiler tag counts");
    assert!(!pick.because.is_empty(), "the user's own shows behind it");
    assert!(
        pick.because.iter().all(|b| b.score.is_some()),
        "each with the user's score"
    );

    // Nothing anywhere names the spoiler tag.
    for scope in [
        RecommendedScope::All,
        RecommendedScope::InLibrary,
        RecommendedScope::OnList,
    ] {
        let text = serde_json::to_string(&listed(&core, scope)).unwrap();
        assert!(
            !text.contains(SPOILER),
            "{scope:?} names the spoiler: {text}"
        );
    }
    let (page, reading) = match core.call(Call::GetShowPage { anilist_id: 204 }).unwrap() {
        Reply::ShowPage { page, reading } => (page, reading),
        other => panic!("{other:?}"),
    };
    // The page's own read was never made, so the opening starts it. Nothing
    // is queued for it, so it fails, and the page stays as the core holds it.
    common::wait_job(&c, reading.expect("a first opening reads the page"));
    assert!(page.known);
    assert_eq!(page.forecast, Some(pick.forecast));
    let text = serde_json::to_string(&page).unwrap();
    assert!(
        !text.contains(SPOILER),
        "the page names the spoiler: {text}"
    );

    // A second fit straight after reads nothing: everything is cached.
    assert_eq!(anilist_posts(&http), 8, "the page's read");
    run(&core, &c, true);
    assert_eq!(anilist_posts(&http), 8, "a second fit reads nothing");

    // Not interested keeps a show off the row too.
    core.call(Call::SetNotInterested {
        anilist_id: 200,
        not_interested: true,
    })
    .unwrap();
    assert!(!ids(&listed(&core, RecommendedScope::All)).contains(&200));
}

#[test]
fn a_fresh_fit_is_not_run_again_unless_forced() {
    let (_dir, core, c, _http, _owned) = setup();
    run(&core, &c, false);
    assert_eq!(
        core.call(Call::RefreshRecommended { force: false })
            .unwrap(),
        Reply::Ok
    );
}

#[test]
fn a_list_with_nothing_rated_says_so() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 0, "planning", None);
    run(&core, &c, false);
    assert_eq!(
        listed(&core, RecommendedScope::All).state,
        RecommendedState::NoScores
    );
    assert_eq!(anilist_posts(&http), 0);
}

#[test]
fn a_rewatch_is_not_a_status_to_set_on_a_show() {
    let (_dir, core, _c, _http, _owned) = setup();
    let e = core
        .call(Call::SetShowListStatus {
            anilist_id: 300,
            status: ListStatus::Repeating,
        })
        .unwrap_err();
    assert!(matches!(e, CoreError::Invalid { .. }), "{e:?}");
}
