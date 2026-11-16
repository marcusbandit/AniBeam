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
