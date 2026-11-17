//! The airing refresh: the cheap fetch that keeps a releasing series'
//! next broadcast right without asking for the whole record again.

mod common;
use common::started;
mod fixtures;
use std::time::Duration;

use anibeam_core::*;

/// AniList's schedule reply: the paginated page of nodes and the always
/// present next broadcast.
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

/// The same reply with the media's own status in it, which is how the
/// cheap fetch learns a series stopped airing.
fn schedule_json_status(
    id: u64,
    status: &str,
    next: Option<(u32, i64)>,
    nodes: &[(u32, i64)],
) -> serde_json::Value {
    let mut v = schedule_json(id, next, nodes);
    v["data"]["Media"]["status"] = serde_json::Value::String(status.into());
    v
}

/// Jikan's episode list, which is where a title would come from if this
/// job were allowed to write one.
fn jikan_json(episodes: &[(u32, &str)]) -> String {
    let data: Vec<serde_json::Value> = episodes
        .iter()
        .map(|(n, t)| serde_json::json!({ "mal_id": n, "episode": n, "title": t, "aired": serde_json::Value::Null }))
        .collect();
    serde_json::json!({ "data": data }).to_string()
}

/// A releasing series with a file on disk and a match: the one shape the
/// airing refresh exists for.
fn releasing(core: &Core, source: u64, name: &str, anilist_id: u64, mal_id: u64) -> u64 {
    matched(core, source, name, anilist_id, mal_id, "RELEASING")
}

fn matched(
    core: &Core,
    source: u64,
    name: &str,
    anilist_id: u64,
    mal_id: u64,
    status: &str,
) -> u64 {
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
        .status(status)
        .score(80)
        .insert(core);
    fixtures::match_series(core, series, Some(anilist_id), Some(mal_id));
    series
}

/// One stored episode row: the number, the title and the date.
type Row = (i64, Option<String>, Option<i64>);

/// The stored rows for one media id, in episode order.
fn episodes(core: &Core, anilist_id: u64) -> Vec<Row> {
    core.store()
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT number, title, aired_at FROM anilist_episodes WHERE anilist_id = ?1 ORDER BY number")?;
            let rows = stmt.query_map([anilist_id as i64], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .unwrap()
}

/// The whole point of the job: the next broadcast lands, the dates come
/// fresher than the stored ones, a title already written stays exactly as
/// it was, and a future row the provider has dropped goes.
#[test]
fn a_refresh_writes_the_next_broadcast_and_keeps_the_stored_titles() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = releasing(&core, src, "Sousou no Frieren", 1, 1001);

    // What a full metadata fetch left behind: a titled episode 1, and a
    // scheduled episode 10 that has since been dropped from the schedule.
    let now = time::now_secs();
    fixtures::insert_episode(
        &core,
        1,
        1,
        Some("The Journey's End"),
        Some(now - 9 * 86_400),
    );
    fixtures::insert_episode(&core, 1, 10, None, Some(now + 30 * 86_400));

    let next_at = now + 86_400;
    let nodes: Vec<(u32, i64)> = (1..=8)
        .map(|n| (n, now - i64::from(9 - n) * 86_400))
        .collect();
    http.push_for(
        "anilist",
        200,
        schedule_json(1, Some((9, next_at)), &nodes).to_string(),
    );
    http.push_for(
        "jikan.moe",
        200,
        jikan_json(&[(1, "A title this job may not write"), (2, "Nor this one")]),
    );

    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::AiringRefreshed {
            series,
            updated: true
        }
    );
    assert_eq!(done.level, Level::Debug);
    assert_eq!(http.requests().len(), 2);

    let rows = episodes(&core, 1);
    assert_eq!(
        rows.iter().map(|(n, _, _)| *n).collect::<Vec<_>>(),
        (1..=9).collect::<Vec<i64>>()
    );
    // The stored title survives: the schedule carries none and Jikan's is
    // not allowed to replace one that is already there.
    assert_eq!(rows[0].1.as_deref(), Some("The Journey's End"));
    // Episode 2 had no title stored, so Jikan's fills the gap.
    assert_eq!(rows[1].1.as_deref(), Some("Nor this one"));
    // The next broadcast, which is the whole reason the job exists.
    assert_eq!(rows[8], (9, None, Some(next_at)));

    // The card behind the series was reported before the terminal event.
    let seen = common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SeriesChanged { series: cards } if cards.iter().any(|card| card.id == series)),
        Duration::from_secs(5),
    );
    assert!(matches!(seen.body, EventBody::SeriesChanged { .. }));

    core.shutdown();
}

/// Six hours is the window. Inside it the job ends at once and asks
/// nothing; past it the same series is a candidate again.
#[test]
fn a_second_refresh_inside_the_window_asks_nothing() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = releasing(&core, src, "Sousou no Frieren", 1, 1001);
    let now = time::now_secs();
    fixtures::set_airing_refreshed_at(&core, 1, Some(now));

    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::AiringRefreshed {
            series,
            updated: false
        }
    );
    assert!(http.requests().is_empty());

    // Six hours and a minute later the row is worth asking about again.
    fixtures::set_airing_refreshed_at(&core, 1, Some(now - 6 * 3_600 - 60));
    http.push_for(
        "anilist",
        200,
        schedule_json(1, Some((9, now + 86_400)), &[]).to_string(),
    );
    http.push_for("jikan.moe", 200, jikan_json(&[]));
    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::AiringRefreshed {
            series,
            updated: true
        }
    );
    assert_eq!(http.requests().len(), 2);

    // An unknown series is not a job at all.
    let err = core
        .call(Call::RefreshAiring { series: 9_999 })
        .err()
        .unwrap();
    assert!(
        matches!(
            err,
            CoreError::NotFound {
                what: Entity::Series,
                id: 9_999
            }
        ),
        "{err:?}"
    );

    core.shutdown();
}

/// A finished series has no next episode to find, so it never becomes a
/// candidate however long ago it was refreshed.
#[test]
fn a_finished_series_is_never_a_candidate() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = matched(&core, src, "Perfect Blue", 2, 1002, "FINISHED");

    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::AiringRefreshed {
            series,
            updated: false
        }
    );
    assert!(http.requests().is_empty());

    core.shutdown();
}

/// The status rides the schedule reply, and writing it is the only way a
/// series that stopped airing ever leaves the airing rail: the full fetch
/// that normally carries it ran once, when the series was matched. A
/// series that ends has its status written and drops out of the
/// candidates the same turn.
#[test]
fn a_refresh_that_learns_the_series_ended_writes_the_status() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = releasing(&core, src, "Sousou no Frieren", 1, 1001);

    // The show ended: no next broadcast, no schedule left, status finished.
    http.push_for(
        "anilist",
        200,
        schedule_json_status(1, "FINISHED", None, &[]).to_string(),
    );
    http.push_for("jikan.moe", 200, jikan_json(&[]));
    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::AiringRefreshed {
            series,
            updated: true
        }
    );

    let status: Option<String> = core
        .store()
        .read(|conn| {
            Ok(
                conn.query_row("SELECT status FROM anilist_media WHERE id = 1", [], |r| {
                    r.get(0)
                })?,
            )
        })
        .unwrap();
    assert_eq!(status.as_deref(), Some("FINISHED"));

    // Finished, it is no longer a candidate: the sweep has nothing to ask.
    let now = time::now_secs();
    let work = core
        .store()
        .read(|conn| anibeam_core::metadata::airing::candidates(conn, now))
        .unwrap();
    assert!(work.is_empty(), "{work:?}");

    core.shutdown();
}

/// Jikan is the title side-fetch and nothing else, so an outage costs a
/// series its titles rather than its schedule. Inside the ten minute
/// window Jikan is not asked again, and the outage is one line in the log.
#[test]
fn a_jikan_failure_is_one_warning_and_no_second_request_inside_the_window() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let first = releasing(&core, src, "Sousou no Frieren", 1, 1001);
    let second = releasing(&core, src, "Dungeon Meshi", 2, 1002);

    let now = time::now_secs();
    for id in [1, 2] {
        http.push_for(
            "anilist",
            200,
            schedule_json(id, Some((9, now + 86_400)), &[]).to_string(),
        );
    }
    http.push_for("jikan.moe", 504, "gateway timeout");

    for series in [first, second] {
        let job = started(&core, Call::RefreshAiring { series });
        let done = common::wait_job(&c, job);
        // The schedule still landed: Jikan's failure costs the titles only.
        assert_eq!(
            done.body,
            EventBody::AiringRefreshed {
                series,
                updated: true
            }
        );
    }
    // Both schedules and one Jikan request: the second series' turn came
    // inside the window, so Jikan was left alone.
    assert_eq!(http.requests().len(), 3);

    let warnings: Vec<String> = c
        .events()
        .into_iter()
        .filter(|e| e.level == Level::Warn && e.message.contains("Jikan"))
        .map(|e| e.message)
        .collect();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(
        warnings[0].starts_with("Jikan is not answering:"),
        "{warnings:?}"
    );

    core.shutdown();
}

/// AniList not answering at all is not the series' turn having come. The
/// sweep ends there and stamps nothing, so the next launch asks about the
/// same series rather than counting a whole library refreshed against an
/// outage.
#[test]
fn a_transport_failure_ends_the_sweep_and_stamps_nothing() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    releasing(&core, src, "Aaa", 1, 1001);
    releasing(&core, src, "Bbb", 2, 1002);

    // A connection that never opened. The limiter does not retry one, so
    // the sweep makes a single request and never reaches the second row.
    http.fail_next("connection refused");

    let job = anibeam_core::metadata::airing::start_refresh_library(core.metadata());
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::JobFailed {
                error: CoreError::Provider { status: None, .. }
            }
        ),
        "{done:?}"
    );
    assert_eq!(http.requests().len(), 1);
    let stamped: i64 = core
        .store()
        .read(|conn| {
            Ok(conn.query_row(
                "SELECT count(*) FROM anilist_media WHERE airing_refreshed_at IS NOT NULL",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(stamped, 0, "an outage stamped a series as refreshed");
    core.shutdown();
}

/// One page of Jikan's episode list: episodes `first..=last`, titled,
/// saying whether another page follows.
fn jikan_page(first: u32, last: u32, has_next: bool) -> String {
    let data: Vec<serde_json::Value> = (first..=last)
        .map(|n| serde_json::json!({ "mal_id": n, "episode": n, "title": format!("Episode title {n}"), "aired": serde_json::Value::Null }))
        .collect();
    serde_json::json!({ "pagination": { "has_next_page": has_next }, "data": data }).to_string()
}

/// A long show's refresh asks Jikan only from the page that holds the
/// first episode it can still learn about (#1105): past the highest
/// stored title, or the first future row if one comes sooner. What it
/// read is whole from there on, so a dropped future row still goes.
#[test]
fn a_long_show_asks_jikan_only_from_the_page_it_can_still_learn_from() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = releasing(&core, src, "One Piece", 1, 21);

    let now = time::now_secs();
    fixtures::insert_episode(
        &core,
        1,
        1,
        Some("Romance Dawn"),
        Some(now - 9_000 * 86_400),
    );
    fixtures::insert_episode(
        &core,
        1,
        1050,
        Some("Stored long ago"),
        Some(now - 3 * 86_400),
    );
    // Scheduled once, since dropped by AniList, and not on Jikan either.
    fixtures::insert_episode(&core, 1, 1060, None, Some(now + 30 * 86_400));

    http.push_for(
        "anilist",
        200,
        schedule_json(
            1,
            Some((1053, now + 86_400)),
            &[(1051, now - 2 * 86_400), (1052, now - 86_400)],
        )
        .to_string(),
    );
    http.push_for("episodes?page=11", 200, jikan_page(1001, 1052, false));

    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::AiringRefreshed {
            series,
            updated: true
        }
    );
    let jikan: Vec<String> = http
        .requests()
        .into_iter()
        .map(|r| r.url)
        .filter(|u| u.contains("jikan.moe"))
        .collect();
    assert_eq!(
        jikan,
        vec!["https://api.jikan.moe/v4/anime/21/episodes?page=11".to_string()]
    );

    let rows = episodes(&core, 1);
    let get = |n: i64| rows.iter().find(|(m, _, _)| *m == n).cloned();
    // Stored titles stay; the new episode gets Jikan's.
    assert_eq!(get(1050).unwrap().1.as_deref(), Some("Stored long ago"));
    assert_eq!(get(1051).unwrap().1.as_deref(), Some("Episode title 1051"));
    assert_eq!(get(1053).unwrap().2, Some(now + 86_400));
    // The read was whole from page 11 on, so the dropped future row went.
    assert!(get(1060).is_none(), "{rows:?}");

    core.shutdown();
}

/// A read that stops at its time limit leaves one line in the log, not
/// one per page, and what it read is not whole, so no future row goes.
#[test]
fn a_jikan_read_cut_short_says_so_once() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_jikan_budget(Duration::ZERO);
    let src = fixtures::insert_source(&core, "/lib");
    let series = releasing(&core, src, "One Piece", 1, 21);

    let now = time::now_secs();
    fixtures::insert_episode(&core, 1, 150, None, Some(now + 30 * 86_400));
    http.push_for(
        "anilist",
        200,
        schedule_json(1, Some((3, now + 86_400)), &[(1, now - 86_400)]).to_string(),
    );
    http.push_for("jikan.moe", 200, jikan_page(1, 100, true));

    let job = started(&core, Call::RefreshAiring { series });
    common::wait_job(&c, job);
    assert_eq!(http.requests().len(), 2, "{:#?}", http.requests());

    let lines: Vec<String> = c
        .events()
        .into_iter()
        .filter(|e| e.message.starts_with("Jikan's episode list"))
        .map(|e| e.message)
        .collect();
    assert_eq!(
        lines,
        vec![
            "Jikan's episode list for MAL 21 stopped at its time limit, 100 episodes read"
                .to_string()
        ]
    );
    assert!(episodes(&core, 1).iter().any(|(n, _, _)| *n == 150));

    // The same show cut short again later in the run says nothing more.
    fixtures::set_airing_refreshed_at(&core, 1, Some(now - 7 * 3_600));
    http.push_for(
        "anilist",
        200,
        schedule_json(1, Some((3, now + 86_400)), &[(1, now - 86_400)]).to_string(),
    );
    http.push_for("jikan.moe", 200, jikan_page(1, 100, true));
    let job = started(&core, Call::RefreshAiring { series });
    common::wait_job(&c, job);
    assert_eq!(http.requests().len(), 4, "{:#?}", http.requests());
    let lines = c
        .events()
        .into_iter()
        .filter(|e| e.message.starts_with("Jikan's episode list"))
        .count();
    assert_eq!(lines, 1);

    core.shutdown();
}

/// An untitled episode that aired lately may still get Jikan's title, even
/// below the highest stored title, so the read reaches back to its page.
#[test]
fn a_late_untitled_episode_is_still_asked_about() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = releasing(&core, src, "One Piece", 1, 21);

    let now = time::now_secs();
    fixtures::insert_episode(&core, 1, 950, None, Some(now - 5 * 86_400));
    fixtures::insert_episode(&core, 1, 1100, Some("Stored"), Some(now - 86_400));
    http.push_for(
        "anilist",
        200,
        schedule_json(1, Some((1101, now + 86_400)), &[]).to_string(),
    );
    http.push_for("episodes?page=10", 200, jikan_page(901, 1000, true));
    http.push_for("episodes?page=11", 200, jikan_page(1001, 1100, false));

    let job = started(&core, Call::RefreshAiring { series });
    common::wait_job(&c, job);
    let jikan = http
        .requests()
        .into_iter()
        .filter(|r| r.url.contains("jikan.moe"))
        .count();
    assert_eq!(jikan, 2, "{:#?}", http.requests());
    let rows = episodes(&core, 1);
    let get = |n: i64| rows.iter().find(|(m, _, _)| *m == n).cloned();
    assert_eq!(get(950).unwrap().1.as_deref(), Some("Episode title 950"));
    assert_eq!(get(1100).unwrap().1.as_deref(), Some("Stored"));

    core.shutdown();
}
