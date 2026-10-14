//! The live check behind issue #27, against TVmaze and AniList themselves
//! and a copy of a real library. Ignored, and never pointed at a live
//! database: it opens whatever `ANIBEAM_LIVE_ROOT` names, which must be a
//! scratch directory holding `data/anibeam.db`, a `sqlite3 .backup` of the
//! real one.
//!
//! ```sh
//! root=$(mktemp -d); mkdir -p "$root/data"
//! sqlite3 ~/.local/share/anibeam/anibeam.db ".backup '$root/data/anibeam.db'"
//! ANIBEAM_LIVE_ROOT=$root cargo test -p anibeam-core --test tvmaze_live -- --ignored --nocapture
//! ```
//!
//! Only the one series is left a candidate, so the run costs a handful of
//! requests rather than a whole library's worth.

mod common;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anibeam_core::events::Collector;
use anibeam_core::trackers::Secrets;
use anibeam_core::*;
use rusqlite::params;

/// Avatar Seven Havens on the owner's machine, in /mnt/wd_general/media/TV.
const AVATAR: u64 = 58;

#[test]
#[ignore = "talks to TVmaze and AniList; needs ANIBEAM_LIVE_ROOT"]
fn avatar_seven_havens_auto_matches_on_tvmaze_from_a_copy_of_the_real_library() {
    let root = std::env::var("ANIBEAM_LIVE_ROOT").expect("ANIBEAM_LIVE_ROOT");
    let real = std::env::var("HOME").unwrap() + "/.local/share/anibeam";
    assert!(
        !Path::new(&root).starts_with(&real),
        "refusing to open the live database"
    );
    let paths = CorePaths::under(Path::new(&root));
    let secrets = Secrets::file_only(paths.secrets_path());
    let core = Core::open_with_secrets(paths, secrets).unwrap();
    let c = Arc::new(Collector::default());
    let _sub = core.subscribe(c.clone());

    let version: i64 = core
        .store()
        .read(|c| Ok(c.query_row("PRAGMA user_version", [], |r| r.get(0))?))
        .unwrap();
    assert!(version >= 2, "the copy did not take migration 0002");

    // Every other unmatched series counts as already tried under the new
    // rules, and Avatar's version 1 stamp is cleared the way the version
    // bump clears it.
    core.store()
        .write(|c| {
            c.execute(
                "INSERT INTO settings (key, value) VALUES ('auto_match_version', '2')
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [],
            )?;
            c.execute(
                "UPDATE series SET attempted_at = 1, attempt_version = 2 WHERE provider IS NULL AND id != ?1",
                params![AVATAR as i64],
            )?;
            c.execute(
                "UPDATE series SET attempted_at = NULL, attempt_version = NULL WHERE id = ?1 AND provider IS NULL",
                params![AVATAR as i64],
            )?;
            Ok(())
        })
        .unwrap();

    let job = match core.call(Call::AutoMatch).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    let done = common::wait_job(&c, job);
    for e in c.events().iter().filter(|e| e.job.is_some()) {
        println!("{:?} {}", e.level, e.message);
    }
    assert!(
        matches!(done.body, EventBody::AutoMatchFinished { matched: 1, .. }),
        "{done:?}"
    );

    let detail = match core.call(Call::GetSeries { series: AVATAR }).unwrap() {
        Reply::SeriesDetail { detail } => *detail,
        other => panic!("{other:?}"),
    };
    let m = detail.card.match_info.clone().unwrap();
    println!(
        "{} | {:?} {:?} | poster {:?} | banner {:?}",
        detail.card.title, m.provider, m.tvmaze_id, detail.card.poster, detail.banner
    );
    println!("synopsis: {}", detail.synopsis);
    for e in &detail.episodes {
        println!("{} {:?} {:?}", e.code, e.title, e.air_date);
    }
    println!("next airing: {:?}", detail.card.next_airing);
    assert_eq!((m.provider, m.tvmaze_id), (Provider::Tvmaze, Some(83073)));
    assert!(detail.card.poster.as_deref().is_some_and(|p| Path::new(p).exists()));
    assert!(!detail.synopsis.is_empty());
    assert_eq!(detail.episodes.len(), 3);
    assert!(detail.episodes.iter().all(|e| e.title.is_some()));

    // The manual half's search, on an anime title TVmaze also carries.
    let job = match core
        .call(Call::SearchProvider {
            provider: Provider::Tvmaze,
            query: "Frieren".into(),
            limit: 5,
        })
        .unwrap()
    {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    let done = common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SearchFinished { .. }) && e.job.as_ref().is_some_and(|j| j.id == job),
        Duration::from_secs(30),
    );
    if let EventBody::SearchFinished { results } = done.body {
        for r in results {
            println!("TVmaze {} {} {:?} {:?}", r.id, r.title, r.year, r.format);
        }
    }
    core.shutdown();
}
