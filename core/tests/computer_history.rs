//! Two AniBeam computers merge their watch history, resume points and
//! matches (#561): two cores with the same show in different folders, one
//! serving its real router on loopback and the other pulling from it.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use anibeam_core::*;
use axum::extract::connect_info::MockConnectInfo;

fn serve(rt: &tokio::runtime::Runtime, core: &Arc<Core>) -> String {
    let me: SocketAddr = "127.0.0.1:40000".parse().unwrap();
    let app = anibeam_core::host::server::router(Arc::downgrade(core)).layer(MockConnectInfo(me));
    rt.block_on(async {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let at = l.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(l, app).await });
        format!("http://{at}")
    })
}

fn exec(core: &Core, sql: &'static str) {
    core.store()
        .write(move |c| {
            c.execute_batch(sql)?;
            Ok(())
        })
        .unwrap();
}

/// (episode key, at) rows of one history table for the show.
fn rows(core: &Core, sql: &'static str) -> Vec<(String, i64)> {
    core.store()
        .read(move |c| {
            let mut stmt = c.prepare(sql)?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .unwrap()
}

fn completed(core: &Core) -> Vec<(String, i64)> {
    rows(
        core,
        "SELECT episode_key, at FROM completed ORDER BY episode_key",
    )
}

fn views(core: &Core) -> Vec<(String, i64)> {
    rows(core, "SELECT episode_key, at FROM views")
}

fn resume(core: &Core) -> Vec<(String, i64)> {
    rows(
        core,
        "SELECT episode_key, CAST(position AS INTEGER) FROM resume_points",
    )
}

fn the_match(core: &Core) -> (Option<i64>, i64) {
    core.store()
        .read(|c| {
            Ok(
                c.query_row("SELECT tvmaze_id, confirmed FROM series", [], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })?,
            )
        })
        .unwrap()
}

fn pull(
    rt: &tokio::runtime::Runtime,
    core: &Arc<Core>,
    base: &str,
) -> anibeam_core::host::history::Merged {
    rt.block_on(anibeam_core::host::history::pull(core, base))
        .unwrap()
}

#[test]
fn history_from_another_computer_lands_on_the_same_show_here() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (a_dir, a, a_c) = common::open_core();
    let (b_dir, b, b_c) = common::open_core();
    common::library_show(a_dir.path(), &a, &a_c);
    common::library_show(b_dir.path(), &b, &b_c);
    // On the other computer: episode 1 done, episode 2 half watched.
    exec(
        &b,
        "INSERT INTO completed (series_id, episode_key, at) VALUES (1, '1', 1800000100);
         INSERT INTO views (series_id, episode_key, at) VALUES (1, '2', 1800000200);
         INSERT INTO resume_points (series_id, episode_key, position, duration, at)
             VALUES (1, '2', 600, 1400, 1800000200);",
    );
    let base = serve(&rt, &b);

    let merged = pull(&rt, &a, &base);
    assert_eq!(
        (merged.views, merged.completed, merged.resume_points),
        (1, 1, 1)
    );
    assert_eq!(completed(&a), [("1".to_string(), 1800000100)]);
    assert_eq!(views(&a), [("2".to_string(), 1800000200)]);
    assert_eq!(resume(&a), [("2".to_string(), 600)]);

    // The same history again changes nothing.
    assert!(!pull(&rt, &a, &base).any());
}

#[test]
fn what_this_computer_watched_later_stays() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (a_dir, a, a_c) = common::open_core();
    let (b_dir, b, b_c) = common::open_core();
    common::library_show(a_dir.path(), &a, &a_c);
    common::library_show(b_dir.path(), &b, &b_c);
    exec(
        &a,
        "INSERT INTO views (series_id, episode_key, at) VALUES (1, '2', 1800000900);
         INSERT INTO resume_points (series_id, episode_key, position, duration, at)
             VALUES (1, '2', 900, 1400, 1800000900);",
    );
    exec(
        &b,
        "INSERT INTO views (series_id, episode_key, at) VALUES (1, '1', 1800000200);
         INSERT INTO resume_points (series_id, episode_key, position, duration, at)
             VALUES (1, '2', 300, 1400, 1800000200);",
    );
    let base = serve(&rt, &b);
    pull(&rt, &a, &base);
    assert_eq!(views(&a), [("2".to_string(), 1800000900)]);
    assert_eq!(resume(&a), [("2".to_string(), 900)]);
}

#[test]
fn a_match_comes_over_only_onto_a_show_with_none_confirmed() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (a_dir, a, a_c) = common::open_core();
    let (b_dir, b, b_c) = common::open_core();
    common::library_show(a_dir.path(), &a, &a_c);
    common::library_show(b_dir.path(), &b, &b_c);
    exec(
        &b,
        "UPDATE series SET provider = 'tvmaze', tvmaze_id = 21, confirmed = 1;
         INSERT INTO completed (series_id, episode_key, at) VALUES (1, '1', 1800000100);",
    );
    let base = serve(&rt, &b);
    let merged = pull(&rt, &a, &base);
    assert_eq!(merged.matches, 1);
    assert_eq!(the_match(&a), (Some(21), 1));
    assert_eq!(completed(&a).len(), 1);

    // A show here the person matched to something else is another show:
    // its match stays and nothing of the other one's history lands on it.
    let (c_dir, c, c_c) = common::open_core();
    common::library_show(c_dir.path(), &c, &c_c);
    exec(
        &c,
        "UPDATE series SET provider = 'tvmaze', tvmaze_id = 99, confirmed = 1;",
    );
    let merged = pull(&rt, &c, &base);
    assert!(!merged.any(), "{merged:?}");
    assert_eq!(the_match(&c), (Some(99), 1));
    assert!(completed(&c).is_empty());
}

#[test]
fn a_show_only_the_other_computer_has_is_left_there() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (a_dir, a, a_c) = common::open_core();
    let (b_dir, b, b_c) = common::open_core();
    common::library_show(a_dir.path(), &a, &a_c);
    common::library_show(b_dir.path(), &b, &b_c);
    exec(
        &b,
        "UPDATE series SET path = path || ' Elsewhere';
         INSERT INTO completed (series_id, episode_key, at) VALUES (1, '1', 1800000100);",
    );
    let base = serve(&rt, &b);
    assert!(!pull(&rt, &a, &base).any());
    let count: i64 = a
        .store()
        .read(|c| Ok(c.query_row("SELECT COUNT(*) FROM series", [], |r| r.get(0))?))
        .unwrap();
    assert_eq!(count, 1, "nothing is created here");
    assert!(completed(&a).is_empty());
}

#[test]
fn the_history_document_carries_nothing_that_belongs_to_one_machine() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (b_dir, b, b_c) = common::open_core();
    common::library_show(b_dir.path(), &b, &b_c);
    let base = serve(&rt, &b);
    let doc: serde_json::Value = rt.block_on(async {
        reqwest::get(format!("{base}{}", anibeam_host_api::HISTORY))
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    });
    assert_eq!(doc["format"], "anibeam-export");
    assert_eq!(doc["series"].as_array().unwrap().len(), 1);
    assert!(doc["history"].is_object(), "{}", doc["history"]);
    for gone in ["accounts", "keys", "preferences"] {
        assert!(doc[gone].is_null(), "{gone}: {}", doc[gone]);
    }
    assert!(doc["sources"].as_array().is_none_or(|s| s.is_empty()));
}

#[test]
fn a_resume_point_for_an_episode_finished_here_later_stays_gone() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (a_dir, a, a_c) = common::open_core();
    let (b_dir, b, b_c) = common::open_core();
    common::library_show(a_dir.path(), &a, &a_c);
    common::library_show(b_dir.path(), &b, &b_c);
    // Half watched there at 200; finished here at 900, which took the resume point.
    exec(
        &b,
        "INSERT INTO resume_points (series_id, episode_key, position, duration, at)
             VALUES (1, '2', 600, 1400, 1800000200);",
    );
    exec(
        &a,
        "INSERT INTO completed (series_id, episode_key, at) VALUES (1, '2', 1800000900);",
    );
    let base = serve(&rt, &b);
    pull(&rt, &a, &base);
    assert!(resume(&a).is_empty(), "{:?}", resume(&a));
}

#[test]
fn a_match_the_other_computer_only_guessed_stays_there() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (a_dir, a, a_c) = common::open_core();
    let (b_dir, b, b_c) = common::open_core();
    common::library_show(a_dir.path(), &a, &a_c);
    common::library_show(b_dir.path(), &b, &b_c);
    exec(
        &b,
        "UPDATE series SET provider = 'tvmaze', tvmaze_id = 21, confirmed = 0;",
    );
    let base = serve(&rt, &b);
    let merged = pull(&rt, &a, &base);
    assert_eq!(merged.matches, 0);
    assert_eq!(the_match(&a), (None, 0));
}
