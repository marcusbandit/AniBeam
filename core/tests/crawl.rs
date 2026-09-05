//! The franchise crawl end to end: a matched series whose node owes its
//! edges, the walk out along the traversable ones, a cameo the walk refuses
//! to follow, and a rate limit that parks a node for a minute rather than
//! losing it.

mod common;
mod fixtures;

use std::time::Duration;

use anibeam_core::franchise::crawl;
use anibeam_core::net::FakeHttp;
use anibeam_core::*;

/// One enrichment reply: the node itself and the edges leaving it. Only
/// the fields the crawl reads are filled in, which is the same set the
/// series page query asks AniList for.
fn enrichment(id: u64, edges: &[(&str, u64)]) -> String {
    let edges: Vec<serde_json::Value> = edges
        .iter()
        .map(|(relation, to)| {
            serde_json::json!({
                "relationType": relation,
                "node": {
                    "id": to,
                    "idMal": null,
                    "type": "ANIME",
                    "format": "TV",
                    "status": "FINISHED",
                    "seasonYear": 2000 + to,
                    "startDate": { "year": 2000 + to },
                    "siteUrl": format!("https://anilist.co/anime/{to}"),
                    "title": { "romaji": format!("T{to}"), "english": null },
                    "coverImage": { "large": format!("https://img/{to}.jpg") }
                }
            })
        })
        .collect();
    serde_json::json!({ "data": { "Media": {
        "id": id,
        "idMal": null,
        "type": "ANIME",
        "format": "TV",
        "status": "FINISHED",
        "seasonYear": 2000 + id,
        "startDate": { "year": 2000 + id },
        "siteUrl": format!("https://anilist.co/anime/{id}"),
        "title": { "romaji": format!("T{id}"), "english": null },
        "coverImage": { "large": format!("https://img/{id}.jpg") },
        "streamingEpisodes": [],
        "tags": [],
        "studios": { "edges": [] },
        "characters": { "edges": [] },
        "recommendations": { "edges": [] },
        "relations": { "edges": edges }
    } } })
    .to_string()
}

/// The AniList ids the crawl actually asked about, in order.
fn asked(http: &FakeHttp) -> Vec<u64> {
    http.requests()
        .iter()
        .filter(|r| r.url.contains("graphql.anilist.co"))
        .filter_map(|r| match &r.body {
            Some(net::Body::Json(v)) => v["variables"]["id"].as_u64(),
            _ => None,
        })
        .collect()
}

fn relation_rows(core: &Core) -> Vec<(u64, u64, String)> {
    core.store()
        .read(|c| {
            let mut stmt = c.prepare(
                "SELECT from_id, to_id, relation FROM relations ORDER BY from_id, to_id",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    u64::try_from(r.get::<_, i64>(0)?).unwrap_or(0),
                    u64::try_from(r.get::<_, i64>(1)?).unwrap_or(0),
                    r.get::<_, String>(2)?,
                ))
            })?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .unwrap()
}

/// `relations_fetched_at` and `crawl_deferred_until` for one media row, or
/// nothing at all when the row does not exist.
fn crawl_state(core: &Core, id: u64) -> Option<(Option<i64>, Option<i64>)> {
    core.store()
        .read(|c| {
            let mut stmt = c.prepare("SELECT relations_fetched_at, crawl_deferred_until FROM anilist_media WHERE id = ?1")?;
            let mut rows = stmt.query([i64::try_from(id).unwrap_or(i64::MAX)])?;
            match rows.next()? {
                Some(row) => Ok(Some((row.get(0)?, row.get(1)?))),
                None => Ok(None),
            }
        })
        .unwrap()
}

fn cached_images(core: &Core) -> Vec<String> {
    core.store()
        .read(|c| {
            let mut stmt = c.prepare("SELECT url FROM images ORDER BY url")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .unwrap()
}

fn graph_changed(c: &events::Collector, root: u64) -> usize {
    c.bodies()
        .iter()
        .filter(|b| matches!(b, EventBody::GraphChanged { root: r } if *r == root))
        .count()
}

fn warnings(c: &events::Collector) -> Vec<String> {
    c.events()
        .iter()
        .filter(|e| e.level == Level::Warn)
        .map(|e| e.message.clone())
        .collect()
}

/// The deferral the crawl wrote, taken back out again. A test uses this to
/// put a parked node back within reach of the database's own rule while the
/// job that parked it is still running.
fn clear_deferral(core: &Core, id: u64) {
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE anilist_media SET crawl_deferred_until = NULL WHERE id = ?1",
                [i64::try_from(id).unwrap_or(i64::MAX)],
            )?;
            Ok(())
        })
        .unwrap();
}

#[test]
fn the_crawl_walks_traversable_edges_stops_at_a_cameo_and_defers_a_rate_limit() {
    let http = FakeHttp::new();
    // The two nodes that answer, then the covers they name, then the wall
    // node 3 runs into: seven 429s asking for no wait at all, which is the
    // limiter's whole schedule spent in one go. Nodes 4 and 5 answer after
    // it, so the walk carries on past the deferral and closes the graph
    // twice more with node 3 still in the table.
    http.push_for(
        "graphql.anilist.co",
        200,
        enrichment(1, &[("SEQUEL", 2), ("CHARACTER", 50)]),
    );
    http.push_for(
        "graphql.anilist.co",
        200,
        enrichment(
            2,
            &[("PREQUEL", 1), ("SEQUEL", 3), ("SEQUEL", 4), ("SEQUEL", 5)],
        ),
    );
    http.push_for("img/1.jpg", 200, vec![0xFF, 0xD8, 0xFF, 0xE0]);
    http.push_for("img/2.jpg", 200, vec![0xFF, 0xD8, 0xFF, 0xE0]);
    for _ in 0..7 {
        http.push_with_headers(429, "no", vec![("Retry-After".into(), "0".into())]);
    }
    http.push_for("graphql.anilist.co", 200, enrichment(4, &[("PREQUEL", 2)]));
    http.push_for("img/4.jpg", 200, vec![0xFF, 0xD8, 0xFF, 0xE0]);
    http.push_for("graphql.anilist.co", 200, enrichment(5, &[("PREQUEL", 2)]));
    http.push_for("img/5.jpg", 200, vec![0xFF, 0xD8, 0xFF, 0xE0]);

    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let s = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/A", "A");
    fixtures::insert_file(&core, s, "/lib/A/01.mkv", 1.0, None, "episode", 1);
    fixtures::insert_media(
        &core,
        1,
        Some("T1"),
        None,
        Some(12),
        "FINISHED",
        "TV",
        Some(80),
    );
    fixtures::match_series(&core, s, Some(1), None);

    let job = crawl::start_gap_crawl(&core);

    // Nodes 1, 2 and 4 land, in that order, with node 3's deferral in
    // between. Once node 4 is in, the database's own hold on node 3 is
    // taken away while the same job is still walking: what keeps the job
    // from asking about 3 again is its own record of having parked it, and
    // there are two more closures to come in which it could get it wrong.
    assert!(c.wait_for(
        |events| {
            events
                .iter()
                .filter(|e| matches!(e.body, EventBody::GraphChanged { root: 1 }))
                .count()
                >= 3
        },
        Duration::from_secs(30)
    ));
    clear_deferral(&core, 3);

    let finished = common::wait_job(&c, job);
    assert!(
        matches!(
            finished.body,
            EventBody::CrawlFinished {
                fetched: 4,
                deferred: 1
            }
        ),
        "{:?}",
        finished.body
    );
    assert_eq!(finished.message, "crawl finished: 4 fetched, 1 deferred");

    // The cameo is drawn and stopped at: node 50 keeps its edge and its
    // stub row, and the crawl never asks AniList a thing about it. Node 3
    // is asked about once, over the limiter's whole schedule, and never
    // again in this job however the table reads afterwards.
    let ids = asked(&http);
    assert_eq!(ids, vec![1, 2, 3, 3, 3, 3, 3, 3, 3, 4, 5], "{ids:?}");
    assert!(!ids.contains(&50));

    assert_eq!(
        relation_rows(&core),
        vec![
            (1, 2, "SEQUEL".to_string()),
            (1, 50, "CHARACTER".to_string()),
            (2, 1, "PREQUEL".to_string()),
            (2, 3, "SEQUEL".to_string()),
            (2, 4, "SEQUEL".to_string()),
            (2, 5, "SEQUEL".to_string()),
            (4, 2, "PREQUEL".to_string()),
            (5, 2, "PREQUEL".to_string()),
        ]
    );

    // Every neighbour got its stub row, whether the walk followed it or not.
    for id in [2, 3, 4, 5, 50] {
        assert!(crawl_state(&core, id).is_some(), "no stub row for {id}");
    }
    for id in [1, 2, 4, 5] {
        assert!(
            crawl_state(&core, id).unwrap().0.is_some(),
            "node {id} owes nothing now"
        );
    }

    // The rate limit parked node 3 rather than losing it: it still owes its
    // edges. The test is what cleared its deferral, so the column says
    // nothing about it now, which is the point.
    assert_eq!(crawl_state(&core, 3).unwrap().0, None);

    // One GraphChanged per node whose edges landed, and the covers of those
    // nodes fetched by the crawl itself.
    assert_eq!(graph_changed(&c, 1), 4);
    assert_eq!(
        cached_images(&core),
        vec![
            "https://img/1.jpg".to_string(),
            "https://img/2.jpg".to_string(),
            "https://img/4.jpg".to_string(),
            "https://img/5.jpg".to_string(),
        ]
    );
    assert!(
        warnings(&c).is_empty(),
        "a rate limit is the terminal event's count, not a log line: {:?}",
        warnings(&c)
    );

    // Node 1 owes nothing now, so the gap crawl has no seed at all: node 3
    // owes its edges but no series is matched to it, so it is nobody's seed.
    let before = http.requests().len();
    let job = crawl::start_gap_crawl(&core);
    let finished = common::wait_job(&c, job);
    assert!(
        matches!(
            finished.body,
            EventBody::CrawlFinished {
                fetched: 0,
                deferred: 0
            }
        ),
        "{:?}",
        finished.body
    );
    assert_eq!(
        http.requests().len(),
        before,
        "a second gap crawl asked for nothing"
    );
}

/// The refetch flag is the only thing that can make a node whose edges are
/// already known worth asking about again. It is what the series page's
/// read passes, so a franchise opened by hand picks up a relation AniList
/// has added since. Node 9 has no reply waiting for it, so this is also
/// what a failure looks like: one line at the end of the job naming how
/// many nodes it cost, never one line per node.
#[test]
fn a_seed_refetch_takes_a_node_whose_edges_are_already_known() {
    let http = FakeHttp::new();
    http.push_for("graphql.anilist.co", 200, enrichment(7, &[("SEQUEL", 8)]));
    http.push_for("img/7.jpg", 200, vec![0xFF, 0xD8, 0xFF, 0xE0]);
    http.push_for(
        "graphql.anilist.co",
        200,
        enrichment(8, &[("PREQUEL", 7), ("SEQUEL", 9)]),
    );
    http.push_for("img/8.jpg", 200, vec![0xFF, 0xD8, 0xFF, 0xE0]);

    let (_dir, core, c) = common::open_core_with_http(http.clone());
    fixtures::insert_media(
        &core,
        7,
        Some("T7"),
        None,
        Some(12),
        "FINISHED",
        "TV",
        Some(80),
    );
    core.store()
        .write(|conn| {
            conn.execute(
                "UPDATE anilist_media SET relations_fetched_at = 1000 WHERE id = 7",
                [],
            )?;
            Ok(())
        })
        .unwrap();

    // Nothing owes anything, so the gap crawl has no seed and does no work.
    let job = crawl::start_gap_crawl(&core);
    let finished = common::wait_job(&c, job);
    assert!(
        matches!(
            finished.body,
            EventBody::CrawlFinished {
                fetched: 0,
                deferred: 0
            }
        ),
        "{:?}",
        finished.body
    );
    assert!(asked(&http).is_empty());

    let job = crawl::start(&core, vec![7], true);
    let finished = common::wait_job(&c, job);
    assert!(
        matches!(
            finished.body,
            EventBody::CrawlFinished {
                fetched: 2,
                deferred: 0
            }
        ),
        "{:?}",
        finished.body
    );
    assert_eq!(
        asked(&http),
        vec![7, 8, 9],
        "the seed is taken again, then the members it turned up"
    );
    assert_eq!(graph_changed(&c, 7), 2);

    // One Warn for the whole job, naming the count and what the first
    // failure looked like, and none of the per-node chatter that would
    // have filled the log on a walk of any size.
    let warned = warnings(&c);
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(
        warned[0].starts_with("crawl: 1 nodes failed, first: "),
        "{}",
        warned[0]
    );

    // The node that failed is stamped rather than left owing, so nothing
    // asks about it for ever.
    assert!(crawl_state(&core, 9).unwrap().0.is_some());
}
