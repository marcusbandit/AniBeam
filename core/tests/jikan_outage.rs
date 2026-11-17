//! A Jikan outage costs titles and nothing more: once Jikan has failed the
//! core leaves it alone for the outage window rather than waiting on it per
//! series, and a fetch without Jikan's half deletes none of the future rows
//! Jikan may have been the one to supply.

mod common;
use common::started;
mod fixtures;

use anibeam_core::*;

/// AniList's schedule reply: the next broadcast and the aired nodes.
fn schedule_json(id: u64, next: Option<(u32, i64)>, nodes: &[(u32, i64)]) -> serde_json::Value {
    let nodes: Vec<serde_json::Value> = nodes
        .iter()
        .map(|(e, at)| serde_json::json!({ "episode": e, "airingAt": at }))
        .collect();
    let next = match next {
        Some((e, at)) => serde_json::json!({ "episode": e, "airingAt": at }),
        None => serde_json::Value::Null,
    };
    serde_json::json!({ "data": { "Media": { "id": id, "nextAiringEpisode": next, "airingSchedule": { "nodes": nodes } } } })
}

/// A releasing series with a file on disk and a match, so the airing
/// refresh has it as a candidate.
fn releasing(core: &Core, source: u64, name: &str, anilist_id: u64, mal_id: u64) -> u64 {
    let series = fixtures::insert_series(
        core,
        source,
        SeriesKind::Show,
        &format!("/lib/{name}"),
        name,
    );
    fixtures::insert_episode_file(core, series, &format!("/lib/{name}/01.mkv"), 1.0, 1);
    fixtures::media(anilist_id, name)
        .episodes(12)
        .status("RELEASING")
        .score(80)
        .insert(core);
    fixtures::match_series(core, series, Some(anilist_id), Some(mal_id));
    series
}

/// The stored episode numbers for one media id, in order.
fn numbers(core: &Core, anilist_id: u64) -> Vec<i64> {
    core.store()
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT number FROM anilist_episodes WHERE anilist_id = ?1 ORDER BY number",
            )?;
            let rows = stmt.query_map([anilist_id as i64], |r| r.get(0))?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .unwrap()
}

fn jikan_requests(http: &anibeam_core::net::FakeHttp) -> usize {
    http.requests()
        .iter()
        .filter(|r| r.url.contains("jikan.moe"))
        .count()
}

fn jikan_warnings(c: &anibeam_core::events::Collector) -> usize {
    c.events()
        .into_iter()
        .filter(|e| e.level == Level::Warn && e.message.contains("Jikan"))
        .count()
}

/// The airing refresh through an outage: the first series asks Jikan and
/// keeps the future row only Jikan knew of, and the second, inside the
/// window, does not ask Jikan at all.
#[test]
fn an_airing_refresh_through_an_outage_keeps_the_rows_and_skips_jikan() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let first = releasing(&core, src, "Sousou no Frieren", 1, 1001);
    let second = releasing(&core, src, "Dungeon Meshi", 2, 1002);

    // A future episode the schedule does not carry, written by an earlier
    // fetch whose Jikan half answered.
    let now = time::now_secs();
    fixtures::insert_episode(
        &core,
        1,
        10,
        Some("Only Jikan knew"),
        Some(now + 30 * 86_400),
    );

    let nodes = [(1, now - 2 * 86_400), (2, now - 86_400)];
    for id in [1, 2] {
        http.push_for(
            "anilist",
            200,
            schedule_json(id, Some((3, now + 86_400)), &nodes).to_string(),
        );
    }
    http.push_for("jikan.moe", 504, "gateway timeout");

    let job = started(&core, Call::RefreshAiring { series: first });
    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::AiringRefreshed { updated: true, .. }),
        "{done:?}"
    );
    assert_eq!(numbers(&core, 1), vec![1, 2, 3, 10]);
    assert_eq!(jikan_requests(&http), 1);

    let job = started(&core, Call::RefreshAiring { series: second });
    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::AiringRefreshed { updated: true, .. }),
        "{done:?}"
    );
    assert_eq!(numbers(&core, 2), vec![1, 2, 3]);
    // Inside the window Jikan is not asked: the second refresh was the
    // schedule alone.
    assert_eq!(jikan_requests(&http), 1, "{:#?}", http.requests());
    assert_eq!(http.requests().len(), 3);
    assert_eq!(jikan_warnings(&c), 1);

    core.shutdown();
}

/// One AniList `Media` reply with a MAL id and no cover, so the fetch asks
/// for no image.
fn media_json(id: u64, romaji: &str, id_mal: u64) -> serde_json::Value {
    serde_json::json!({ "data": { "Media": {
        "id": id,
        "idMal": id_mal,
        "title": { "romaji": romaji, "english": null, "native": null },
        "synonyms": [],
        "episodes": 12,
        "status": "RELEASING",
        "format": "TV",
        "seasonYear": 2020,
        "startDate": { "year": 2020, "month": 1, "day": 1 },
        "averageScore": 80,
        "coverImage": null,
        "studios": { "nodes": [] }
    } } })
}

/// The enrichment with nothing on it.
fn bare_enrichment(id: u64, id_mal: u64) -> serde_json::Value {
    serde_json::json!({ "data": { "Media": {
        "id": id,
        "idMal": id_mal,
        "type": "ANIME",
        "streamingEpisodes": [],
        "tags": [],
        "studios": { "edges": [] },
        "characters": { "edges": [] },
        "recommendations": { "edges": [] },
        "relations": { "edges": [] }
    } } })
}

fn apply(core: &Core, c: &anibeam_core::events::Collector, series: u64, id: u64) {
    let job = started(
        core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Anilist { id, season: None },
        },
    );
    let done = common::wait_job(c, job);
    assert_eq!(done.body, EventBody::MatchApplied { series }, "{done:#?}");
}

/// The full fetch through an outage: the match lands, the future row only
/// Jikan knew of stays, and the next match inside the window does not ask
/// Jikan.
#[test]
fn a_match_through_an_outage_keeps_the_rows_and_skips_jikan() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let mut series = Vec::new();
    for name in ["Sousou no Frieren", "Dungeon Meshi"] {
        let id =
            fixtures::insert_series(&core, src, SeriesKind::Show, &format!("/lib/{name}"), name);
        fixtures::insert_episode_file(&core, id, &format!("/lib/{name}/01.mkv"), 1.0, 1);
        series.push(id);
    }

    // An earlier fetch's record, with a future episode the schedule does
    // not carry.
    fixtures::media(1, "Sousou no Frieren").insert(&core);
    let now = time::now_secs();
    fixtures::insert_episode(
        &core,
        1,
        10,
        Some("Only Jikan knew"),
        Some(now + 30 * 86_400),
    );
    let nodes = [(1, now - 2 * 86_400), (2, now - 86_400)];

    // Media by id, enrichment, schedule, then Jikan, which is down.
    http.push_json(200, media_json(1, "Sousou no Frieren", 1001));
    http.push_json(200, bare_enrichment(1, 1001));
    http.push_json(200, schedule_json(1, Some((3, now + 86_400)), &nodes));
    http.push_for("jikan.moe", 504, "gateway timeout");
    apply(&core, &c, series[0], 1);
    assert_eq!(numbers(&core, 1), vec![1, 2, 3, 10]);
    assert_eq!(jikan_requests(&http), 1);

    http.push_json(200, media_json(2, "Dungeon Meshi", 1002));
    http.push_json(200, bare_enrichment(2, 1002));
    http.push_json(200, schedule_json(2, Some((3, now + 86_400)), &nodes));
    apply(&core, &c, series[1], 2);
    assert_eq!(numbers(&core, 2), vec![1, 2, 3]);
    assert_eq!(jikan_requests(&http), 1, "{:#?}", http.requests());
    assert_eq!(http.requests().len(), 7);
    assert_eq!(jikan_warnings(&c), 1);

    core.shutdown();
}

/// A long show whose second page of Jikan episodes fails (#1040): the
/// first page's episodes are written, the future row only Jikan knew of
/// stays, and the failure warns once like any outage.
#[test]
fn a_failure_on_jikans_second_page_keeps_the_first_and_deletes_nothing() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = releasing(&core, src, "One Piece", 1, 1001);

    let now = time::now_secs();
    fixtures::insert_episode(
        &core,
        1,
        150,
        Some("Only Jikan knew"),
        Some(now + 30 * 86_400),
    );

    let nodes = [(1, now - 2 * 86_400), (2, now - 86_400)];
    http.push_for(
        "anilist",
        200,
        schedule_json(1, Some((3, now + 86_400)), &nodes).to_string(),
    );
    // Page one carries an episode the schedule does not, and says another
    // page follows; page two is down.
    http.push_for(
        "jikan.moe",
        200,
        serde_json::json!({
            "pagination": { "has_next_page": true },
            "data": [
                { "mal_id": 1, "episode": 1, "title": "Romance Dawn", "aired": null },
                { "mal_id": 5, "episode": 5, "title": "A Terrifying Mysterious Power", "aired": null }
            ]
        })
        .to_string(),
    );
    http.push_for("episodes?page=2", 504, "gateway timeout");

    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::AiringRefreshed { updated: true, .. }),
        "{done:?}"
    );
    assert_eq!(numbers(&core, 1), vec![1, 2, 3, 5, 150]);
    assert_eq!(jikan_requests(&http), 2, "{:#?}", http.requests());
    assert_eq!(jikan_warnings(&c), 1);

    core.shutdown();
}

/// Each stored row behind one media id: its number, title and date.
fn rows(core: &Core, anilist_id: u64) -> Vec<(i64, Option<String>, Option<i64>)> {
    core.store()
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT number, title, aired_at FROM anilist_episodes WHERE anilist_id = ?1 ORDER BY number",
            )?;
            let rows = stmt.query_map([anilist_id as i64], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .unwrap()
}

/// The AniList ids still owed Jikan's titles.
fn owed(core: &Core) -> Vec<u64> {
    core.store()
        .read(|c| Ok(anibeam_core::metadata::owed::owed(c)?.into_iter().collect()))
        .unwrap()
}

/// A whole page of Jikan episodes, titled.
fn jikan_page(titles: &[(u32, &str)]) -> String {
    let data: Vec<serde_json::Value> = titles
        .iter()
        .map(|(n, t)| serde_json::json!({ "mal_id": n, "episode": n, "title": t, "aired": "2020-01-01T00:00:00+00:00" }))
        .collect();
    serde_json::json!({ "pagination": { "has_next_page": false }, "data": data }).to_string()
}

/// A finished series with a file on disk and no match yet.
fn unmatched(core: &Core, source: u64, name: &str) -> u64 {
    let series = fixtures::insert_series(
        core,
        source,
        SeriesKind::Show,
        &format!("/lib/{name}"),
        name,
    );
    fixtures::insert_episode_file(core, series, &format!("/lib/{name}/01.mkv"), 1.0, 1);
    series
}

/// Queues the three AniList replies a match makes for a finished show.
fn finished_replies(
    http: &anibeam_core::net::FakeHttp,
    id: u64,
    name: &str,
    mal: u64,
    nodes: &[(u32, i64)],
) {
    let mut media = media_json(id, name, mal);
    media["data"]["Media"]["status"] = serde_json::json!("FINISHED");
    http.push_json(200, media);
    http.push_json(200, bare_enrichment(id, mal));
    http.push_json(200, schedule_json(id, None, nodes));
}

/// A finished series matched while Jikan was down is written down as owed
/// its titles (#1065). The walk launch starts leaves Jikan alone while the
/// window is open, and once it has passed fills the titles in, keeping the
/// dates AniList gave and crossing the series off.
#[test]
fn a_finished_series_matched_through_an_outage_gets_its_titles_once_jikan_answers() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = unmatched(&core, src, "Cowboy Bebop");

    let nodes = [(1, 1_000_000), (2, 1_600_000)];
    finished_replies(&http, 1, "Cowboy Bebop", 1001, &nodes);
    http.push_for("jikan.moe", 504, "gateway timeout");
    apply(&core, &c, series, 1);
    assert_eq!(
        rows(&core, 1),
        vec![(1, None, Some(1_000_000)), (2, None, Some(1_600_000))]
    );
    assert_eq!(owed(&core), vec![1]);

    // Inside the window the walk does not ask Jikan, and the series stays
    // owed.
    let job = anibeam_core::metadata::owed::start(core.metadata()).expect("something is owed");
    common::wait_job(&c, job);
    assert_eq!(jikan_requests(&http), 1, "{:#?}", http.requests());
    assert_eq!(owed(&core), vec![1]);

    // Jikan is back.
    core.set_jikan_window(std::time::Duration::ZERO);
    http.push_for(
        "anime/1001/episodes",
        200,
        jikan_page(&[
            (1, "Asteroid Blues"),
            (2, "Stray Dog Strut"),
            (3, "Honky Tonk Women"),
        ]),
    );
    let job = anibeam_core::metadata::owed::start(core.metadata()).expect("something is owed");
    let done = common::wait_job(&c, job);
    assert!(done.message.contains("filled for 1 series"), "{done:#?}");
    let filled = rows(&core, 1);
    assert_eq!(
        filled[0],
        (1, Some("Asteroid Blues".into()), Some(1_000_000))
    );
    assert_eq!(
        filled[1],
        (2, Some("Stray Dog Strut".into()), Some(1_600_000))
    );
    assert_eq!(filled[2].0, 3);
    assert_eq!(filled[2].1.as_deref(), Some("Honky Tonk Women"));
    assert!(owed(&core).is_empty());
    assert!(anibeam_core::metadata::owed::start(core.metadata()).is_none());
    assert_eq!(jikan_warnings(&c), 1);

    core.shutdown();
}

/// The first whole Jikan reply after the window has passed is Jikan back,
/// and starts the walk on its own: no timer, no launch.
#[test]
fn the_first_jikan_reply_after_an_outage_fills_what_it_left_owed() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let first = unmatched(&core, src, "Cowboy Bebop");
    let second = unmatched(&core, src, "Trigun");

    let nodes = [(1, 1_000_000), (2, 1_600_000)];
    finished_replies(&http, 1, "Cowboy Bebop", 1001, &nodes);
    http.push_for("jikan.moe", 504, "gateway timeout");
    apply(&core, &c, first, 1);
    assert_eq!(owed(&core), vec![1]);

    core.set_jikan_window(std::time::Duration::ZERO);
    finished_replies(&http, 2, "Trigun", 1002, &nodes);
    http.push_for(
        "anime/1002/episodes",
        200,
        jikan_page(&[(1, "The $60,000,000,000 Man"), (2, "Truth of Mistake")]),
    );
    http.push_for(
        "anime/1001/episodes",
        200,
        jikan_page(&[(1, "Asteroid Blues"), (2, "Stray Dog Strut")]),
    );
    apply(&core, &c, second, 2);
    assert_eq!(
        rows(&core, 2)[0].1.as_deref(),
        Some("The $60,000,000,000 Man")
    );

    common::wait_for(
        &c,
        |e| e.message.contains("Jikan titles filled for 1 series"),
        std::time::Duration::from_secs(10),
    );
    assert_eq!(
        rows(&core, 1),
        vec![
            (1, Some("Asteroid Blues".into()), Some(1_000_000)),
            (2, Some("Stray Dog Strut".into()), Some(1_600_000)),
        ]
    );
    assert!(owed(&core).is_empty());
    assert_eq!(jikan_requests(&http), 3, "{:#?}", http.requests());

    core.shutdown();
}
