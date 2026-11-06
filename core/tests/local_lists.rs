//! The list kept on this device (#213): with no tracker account, or for a
//! series no connected tracker carries, the status menu and the score
//! picker still work, the card reads them back, and export carries them.

mod common;
mod fixtures;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;

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

fn card(core: &Core, series: u64) -> SeriesCard {
    match core.call(Call::GetSeries { series }).unwrap() {
        Reply::SeriesDetail { detail } => detail.card,
        other => panic!("{other:?}"),
    }
}

/// No account at all: a status and a score are kept here, answered at
/// once with no job and no request, and the card shows both.
#[test]
fn with_no_account_a_status_and_a_score_are_kept_here() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);

    assert_eq!(
        core.call(Call::SetListStatus {
            series,
            status: ListStatus::Watching
        })
        .unwrap(),
        Reply::Ok
    );
    assert_eq!(
        core.call(Call::SetScore {
            series,
            score: Some(8.5)
        })
        .unwrap(),
        Reply::Ok
    );
    assert!(http.requests().is_empty());
    let c = card(&core, series);
    assert_eq!(c.list_status, Some(ListStatus::Watching));
    assert_eq!(c.my_score, Some(8.5));

    // Clear is a score of none, and the status stays.
    core.call(Call::SetScore {
        series,
        score: None,
    })
    .unwrap();
    let c = card(&core, series);
    assert_eq!(c.my_score, None);
    assert_eq!(c.list_status, Some(ListStatus::Watching));
}

/// A show no tracker knows (no match at all) keeps them here too, even with
/// an account connected, since there is nothing to write to.
#[test]
fn an_unmatched_show_keeps_them_here_with_an_account_connected() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(&core, "/lib");
    let series = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/lib/Home Video",
        "Home Video",
    );
    fixtures::insert_file(
        &core,
        series,
        "/lib/Home Video/01.mkv",
        1.0,
        None,
        "episode",
        now,
    );
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    assert_eq!(
        core.call(Call::SetListStatus {
            series,
            status: ListStatus::Planning
        })
        .unwrap(),
        Reply::Ok
    );
    assert!(http.requests().is_empty());
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Planning));
}

/// With an account behind the match the write goes to the tracker as
/// before, and once the tracker holds an entry the card shows its words,
/// not the ones kept here.
#[test]
fn a_connected_tracker_writes_as_before_and_its_entry_wins() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let series = library(&core);
    core.call(Call::SetScore {
        series,
        score: Some(4.0),
    })
    .unwrap();

    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 3, "watching", Some(9.0));
    assert_eq!(card(&core, series).my_score, Some(9.0));
    assert_eq!(card(&core, series).list_status, Some(ListStatus::Watching));
    assert!(matches!(
        core.call(Call::SetScore {
            series,
            score: Some(8.0)
        })
        .unwrap(),
        Reply::Started { .. }
    ));
}

/// A hidden series is still refused: hiding is about its tracker sync.
#[test]
fn a_hidden_matched_series_is_still_refused() {
    let (_dir, core, _c) = common::open_core_with_http(FakeHttp::new());
    let series = library(&core);
    core.call(Call::SetHidden {
        series,
        hidden: true,
    })
    .unwrap();
    assert_eq!(
        core.call(Call::SetScore {
            series,
            score: Some(7.0)
        })
        .err(),
        Some(CoreError::Refused {
            reason: Refusal::Hidden
        })
    );
}

/// Export carries the list kept here beside its series, and an import into
/// a fresh library brings it back.
#[test]
fn export_carries_the_list_kept_here_and_import_reads_it_back() {
    let (dir, core, c) = common::open_core_with_http(FakeHttp::new());
    let series = library(&core);
    core.call(Call::SetListStatus {
        series,
        status: ListStatus::Completed,
    })
    .unwrap();
    core.call(Call::SetScore {
        series,
        score: Some(7.5),
    })
    .unwrap();

    let out = dir.path().join("export.json");
    let path = out.to_string_lossy().into_owned();
    let job = match core
        .call(Call::Export {
            path: path.clone(),
            private: false,
        })
        .unwrap()
    {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    common::wait_job(&c, job);
    let (doc, ignored) =
        anibeam_core::transfer::format::parse(&std::fs::read(&out).unwrap()).unwrap();
    assert!(ignored.is_empty(), "{ignored:?}");
    let list = doc.series[0].list.clone().expect("the list goes out");
    assert_eq!(list.status, Some(ListStatus::Completed));
    assert_eq!(list.score, Some(7.5));

    let (_dir2, fresh, c2) = common::open_core_with_http(FakeHttp::new());
    fixtures::insert_source(&fresh, "/lib");
    let job = match fresh.call(Call::Import { path }).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    common::wait_job(&c2, job);
    let stored: Vec<(Option<String>, Option<f64>)> = fresh
        .store()
        .read(|c| {
            let mut stmt = c.prepare("SELECT status, score FROM local_entries")?;
            let rows = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .unwrap();
    assert_eq!(stored, vec![(Some("completed".to_string()), Some(7.5))]);
}
