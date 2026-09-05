mod common;
mod fixtures;
use anibeam_core::*;
use rusqlite::params;

fn media_json(id: u64, romaji: &str, cover: bool) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "idMal": id + 1000,
        "title": { "romaji": romaji, "english": null, "native": null },
        "synonyms": [],
        "episodes": 12,
        "status": "FINISHED",
        "format": "TV",
        "seasonYear": 2020,
        "startDate": { "year": 2020, "month": 1, "day": 1 },
        "averageScore": 80,
        "coverImage": if cover {
            serde_json::json!({ "large": format!("https://img/{id}-l.jpg"), "extraLarge": format!("https://img/{id}-xl.jpg") })
        } else {
            serde_json::Value::Null
        },
        "studios": { "nodes": [] }
    })
}

fn list_cards(core: &Core) -> Vec<SeriesCard> {
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
        Reply::Series { series } => series,
        other => panic!("{other:?}"),
    }
}

fn started(core: &Core, call: Call) -> u64 {
    match core.call(call).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

fn attempted_at(core: &Core, series: u64) -> Option<i64> {
    core.store()
        .read(|c| {
            Ok(c.query_row(
                "SELECT attempted_at FROM series WHERE id = ?1",
                params![series as i64],
                |r| r.get(0),
            )?)
        })
        .unwrap()
}

#[test]
fn auto_match_matches_once_at_the_gate_and_never_rehammers() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let hit = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/lib/Sousou no Frieren",
        "Sousou no Frieren",
    );
    fixtures::insert_file(
        &core,
        hit,
        "/lib/Sousou no Frieren/01.mkv",
        1.0,
        None,
        "episode",
        1,
    );
    let miss = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/lib/Zzz Unknown",
        "Zzz Unknown",
    );
    fixtures::insert_file(
        &core,
        miss,
        "/lib/Zzz Unknown/01.mkv",
        1.0,
        None,
        "episode",
        1,
    );
    let no_files = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Empty", "Empty");

    // Search for the hit, enrichment, schedule, Jikan; then the search for the miss.
    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "media": [ media_json(1, "Sousou no Frieren", true), media_json(2, "Other", true) ] } } }),
    );
    http.push_json(
        200,
        serde_json::json!({ "data": { "Media": { "id": 1, "idMal": 1001, "type": "ANIME", "streamingEpisodes": [], "tags": [], "studios": { "edges": [] }, "characters": { "edges": [] }, "recommendations": { "edges": [] }, "relations": { "edges": [] } } } }),
    );
    http.push_json(200, serde_json::json!({ "data": { "Media": { "id": 1, "nextAiringEpisode": null, "airingSchedule": { "nodes": [] } } } }));
    http.push_for(
        "jikan.moe",
        200,
        serde_json::json!({ "data": [] }).to_string(),
    );
    http.push_for("img/1-xl.jpg", 200, vec![1, 2, 3]);
    http.push_json(200, serde_json::json!({ "data": { "Page": { "media": [ media_json(3, "Completely Different Title", true) ] } } }));

    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::AutoMatchFinished {
                backfilled: 0,
                matched: 1,
                unmatched: 1
            }
        ),
        "{done:?}"
    );
    assert!(
        c.events()
            .iter()
            .any(|e| e.message == "no match for Zzz Unknown (threshold 0.5)"),
        "{:#?}",
        c.events()
    );
    assert!(
        c.events()
            .iter()
            .any(|e| e.message == "match (AniList 1.00): Sousou no Frieren -> Sousou no Frieren"),
        "{:#?}",
        c.events()
    );

    let cards = list_cards(&core);
    let matched = cards.iter().find(|s| s.id == hit).unwrap();
    assert_eq!(
        matched
            .match_info
            .as_ref()
            .map(|m| (m.provider, m.anilist_id, m.mal_id, m.confirmed)),
        Some((Provider::Anilist, Some(1), Some(1001), false))
    );
    assert!(matched.poster.is_some(), "{matched:?}");
    assert!(
        cards
            .iter()
            .find(|s| s.id == miss)
            .unwrap()
            .match_info
            .is_none()
    );
    assert!(cards.iter().any(|s| s.id == no_files));

    // A second run touches nothing: no new requests.
    let before = http.requests().len();
    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::AutoMatchFinished {
                matched: 0,
                unmatched: 0,
                ..
            }
        ),
        "{done:?}"
    );
    assert_eq!(http.requests().len(), before);

    // Clear match keeps the series and its attempt.
    assert!(matches!(
        core.call(Call::ClearMatch { series: hit }).unwrap(),
        Reply::Ok
    ));
    let cleared = list_cards(&core);
    assert!(
        cleared
            .iter()
            .find(|s| s.id == hit)
            .unwrap()
            .match_info
            .is_none()
    );
    assert!(
        attempted_at(&core, hit).is_some(),
        "clearing a match must not clear the attempt"
    );
    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::AutoMatchFinished { matched: 0, .. }),
        "{done:?}"
    );
    assert_eq!(http.requests().len(), before);

    assert!(matches!(
        core.call(Call::ClearMatch { series: 9999 }),
        Err(CoreError::NotFound {
            what: Entity::Series,
            id: 9999
        })
    ));
    core.shutdown();
}

/// The provider saying "stop" is not the series failing. A 429 the limiter
/// could not ride out ends the job and stamps nothing, so the series is
/// still a candidate the next time the job runs.
#[test]
fn an_exhausted_rate_limit_fails_the_job_and_stamps_nothing() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/lib/Sousou no Frieren",
        "Sousou no Frieren",
    );
    fixtures::insert_file(
        &core,
        series,
        "/lib/Sousou no Frieren/01.mkv",
        1.0,
        None,
        "episode",
        1,
    );

    // The first request plus the limiter's six retries. `Retry-After: 0` is
    // what keeps the test to the pacing gap rather than the 1, 2, 4, 8, 16,
    // 32 second schedule.
    for _ in 0..7 {
        http.push_with_headers(
            429,
            "rate limited",
            vec![("Retry-After".to_string(), "0".to_string())],
        );
    }

    let job = started(&core, Call::AutoMatch);
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
    assert!(
        attempted_at(&core, series).is_none(),
        "a rate limit must not count as an attempt"
    );
    core.shutdown();
}
