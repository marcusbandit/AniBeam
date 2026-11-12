//! Several series at once (#208): the library's picking hides, rates, rescans,
//! matches and moves a selection to a list, and the activity log gets one line
//! for the whole selection rather than one per series.

mod common;
use common::job_of;
mod fixtures;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;

/// Matched series, each with one file, its own AniList id and twelve episodes.
fn matched_shows(core: &Core, titles: &[&str]) -> Vec<u64> {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    titles
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let id = i as u64 + 1;
            let path = format!("/lib/{title}");
            let series = fixtures::insert_series(core, src, SeriesKind::Show, &path, title);
            fixtures::insert_episode_file(core, series, &format!("{path}/01.mkv"), 1.0, now);
            fixtures::media(id, title)
                .episodes(12)
                .score(80)
                .insert(core);
            fixtures::match_series(core, series, Some(id), None);
            series
        })
        .collect()
}

fn cards(core: &Core, tab: Tab) -> Vec<SeriesCard> {
    common::cards(core, tab, true)
}

fn card(core: &Core, series: u64) -> SeriesCard {
    [Tab::All, Tab::Watched, Tab::Hidden]
        .into_iter()
        .flat_map(|t| cards(core, t))
        .find(|c| c.id == series)
        .expect("the series is listed")
}

/// Every line a person reads in the plain activity log, oldest first.
fn plain_lines(c: &anibeam_core::events::Collector) -> Vec<String> {
    c.events().into_iter().filter_map(|e| e.plain).collect()
}

/// AniList's two answers for one status or score write: the entry read, then the save.
fn anilist_write(http: &FakeHttp) {
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":1,"status":"CURRENT"}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"status":"PLANNING"}}}"#,
    );
}

#[test]
fn hiding_several_hides_each_and_says_so_once() {
    let (_dir, core, c) = common::open_core();
    let ids = matched_shows(&core, &["Bebop", "Trigun", "Monster"]);

    let reply = core
        .call(Call::ActOnSeries {
            series: ids.clone(),
            action: SeriesAction::Hide { hidden: true },
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
    assert!(
        ids.iter().all(|id| card(&core, *id).hidden),
        "every pick is hidden"
    );
    assert_eq!(plain_lines(&c), vec!["Hid 3 shows.".to_string()]);

    core.call(Call::ActOnSeries {
        series: ids.clone(),
        action: SeriesAction::Hide { hidden: false },
    })
    .unwrap();
    assert!(ids.iter().all(|id| !card(&core, *id).hidden));
    assert_eq!(plain_lines(&c).last().unwrap(), "3 shows are back.");
}

#[test]
fn one_series_is_named_rather_than_counted() {
    let (_dir, core, c) = common::open_core();
    let ids = matched_shows(&core, &["Bebop"]);
    core.call(Call::ActOnSeries {
        series: ids,
        action: SeriesAction::Hide { hidden: true },
    })
    .unwrap();
    assert_eq!(plain_lines(&c), vec!["Hid Bebop.".to_string()]);
}

#[test]
fn nothing_picked_is_refused() {
    let (_dir, core, _c) = common::open_core();
    let err = core
        .call(Call::ActOnSeries {
            series: vec![],
            action: SeriesAction::Rescan,
        })
        .unwrap_err();
    assert!(matches!(err, CoreError::Invalid { .. }), "{err:?}");
}

#[test]
fn a_series_that_is_gone_is_not_found() {
    let (_dir, core, _c) = common::open_core();
    let err = core
        .call(Call::ActOnSeries {
            series: vec![99],
            action: SeriesAction::Hide { hidden: true },
        })
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound { id: 99, .. }), "{err:?}");
}

#[test]
fn a_list_status_goes_to_every_pick_in_one_job_and_one_line() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let ids = matched_shows(&core, &["Bebop", "Trigun"]);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    anilist_write(&http);
    anilist_write(&http);

    let job = job_of(
        core.call(Call::ActOnSeries {
            series: ids.clone(),
            action: SeriesAction::ListStatus {
                status: ListStatus::Planning,
            },
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.level, Level::Info, "{done:?}");
    assert_eq!(
        plain_lines(&c),
        vec!["Set 2 shows to Planning on AniList.".to_string()]
    );
    for id in ids {
        assert_eq!(card(&core, id).list_status, Some(ListStatus::Planning));
    }
}

#[test]
fn a_pick_with_nowhere_to_write_is_left_out_and_counted() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let ids = matched_shows(&core, &["Bebop"]);
    let src = fixtures::insert_source(&core, "/other");
    let loose = fixtures::insert_series(&core, src, SeriesKind::Show, "/other/Loose", "Loose");
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    anilist_write(&http);

    let job = job_of(
        core.call(Call::ActOnSeries {
            series: vec![ids[0], loose],
            action: SeriesAction::ListStatus {
                status: ListStatus::Dropped,
            },
        })
        .unwrap(),
    );
    common::wait_job(&c, job);
    assert_eq!(
        plain_lines(&c),
        vec![
            "Set Bebop to Dropped on AniList. 1 show is kept on this computer, with no tracker to send to."
                .to_string()
        ]
    );
}

/// A pick no tracker carries keeps the score on this computer (#213), so a
/// selection of only those answers at once with no job.
#[test]
fn picks_no_tracker_carries_keep_the_score_here() {
    let (_dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    let loose = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Loose", "Loose");
    assert_eq!(
        core.call(Call::ActOnSeries {
            series: vec![loose],
            action: SeriesAction::Score { score: Some(8.0) },
        })
        .unwrap(),
        Reply::Ok
    );
    let kept: Option<f64> = core
        .store()
        .read(|c| {
            Ok(c.query_row(
                "SELECT score FROM local_entries WHERE series_id = ?1",
                [loose as i64],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(kept, Some(8.0));
}

/// Only hidden picks still say why nothing could be written.
#[test]
fn when_no_pick_can_be_written_the_call_says_why() {
    let (_dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    let loose = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Loose", "Loose");
    core.call(Call::SetHidden {
        series: loose,
        hidden: true,
    })
    .unwrap();
    let err = core
        .call(Call::ActOnSeries {
            series: vec![loose],
            action: SeriesAction::Score { score: Some(8.0) },
        })
        .unwrap_err();
    assert!(
        matches!(
            err,
            CoreError::Refused {
                reason: Refusal::Hidden
            }
        ),
        "{err:?}"
    );
}

#[test]
fn a_failed_write_is_counted_in_the_one_line_not_said_per_series() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let ids = matched_shows(&core, &["Bebop", "Trigun"]);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    anilist_write(&http);
    http.push_for("anilist", 500, r#"{"errors":[{"message":"boom"}]}"#);

    let job = job_of(
        core.call(Call::ActOnSeries {
            series: ids,
            action: SeriesAction::Score { score: Some(8.0) },
        })
        .unwrap(),
    );
    common::wait_job(&c, job);
    let lines = plain_lines(&c);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].starts_with("Scored Bebop 8 on AniList. 1 could not be updated: "),
        "the line says why: {lines:?}"
    );
}

#[test]
fn every_write_failing_reads_as_a_warning() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let ids = matched_shows(&core, &["Bebop", "Trigun"]);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    http.push_for("anilist", 500, r#"{"errors":[{"message":"boom"}]}"#);
    http.push_for("anilist", 500, r#"{"errors":[{"message":"boom"}]}"#);

    let job = job_of(
        core.call(Call::ActOnSeries {
            series: ids,
            action: SeriesAction::Score { score: None },
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.level, Level::Warn, "{done:?}");
    let lines = plain_lines(&c);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].starts_with("2 shows could not be updated: "),
        "{lines:?}"
    );
}

#[test]
fn find_a_match_searches_only_the_picks_without_one() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let ids = matched_shows(&core, &["Bebop"]);
    let src = fixtures::insert_source(&core, "/other");
    let loose = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/other/Zzz Unknown",
        "Zzz Unknown",
    );
    fixtures::insert_episode_file(&core, loose, "/other/Zzz Unknown/01.mkv", 1.0, 1);
    // A match the user cleared stays out of the automatic run, but asking is
    // asking: the pick is searched all the same.
    core.call(Call::ClearMatch { series: loose }).unwrap();
    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "media": [] } } }),
    );
    http.push_for("api.tvmaze.com/search/shows", 200, "[]");

    let job = job_of(
        core.call(Call::ActOnSeries {
            series: vec![ids[0], loose],
            action: SeriesAction::FindMatch,
        })
        .unwrap(),
    );
    common::wait_job(&c, job);
    assert_eq!(
        plain_lines(&c),
        vec!["No match found for Zzz Unknown. 1 show had a match already.".to_string()]
    );
    // One search on each provider for the one series without a match.
    let searches = http
        .requests()
        .iter()
        .filter(|r| r.url.contains("tvmaze") || r.url.contains("anilist"))
        .count();
    assert_eq!(searches, 2, "{:#?}", http.requests());
}

#[test]
fn a_rescan_of_several_is_one_scan() {
    let dir = tempfile::tempdir().unwrap();
    let (_cdir, core, c) = common::open_core();
    let lib = dir.path().join("lib");
    for show in ["Bebop", "Trigun"] {
        std::fs::create_dir_all(lib.join(show)).unwrap();
        std::fs::write(lib.join(show).join("Show - 01.mkv"), b"x").unwrap();
    }
    core.call(Call::AddSource {
        path: lib.to_string_lossy().into_owned(),
    })
    .unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ScanFinished { .. }),
        std::time::Duration::from_secs(10),
    );
    let ids: Vec<u64> = cards(&core, Tab::All).iter().map(|c| c.id).collect();
    assert_eq!(ids.len(), 2);
    for show in ["Bebop", "Trigun"] {
        std::fs::write(lib.join(show).join("Show - 02.mkv"), b"x").unwrap();
    }
    let job = job_of(
        core.call(Call::ActOnSeries {
            series: ids,
            action: SeriesAction::Rescan,
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.job.as_ref().unwrap().kind, JobKind::Scan);
    assert!(
        matches!(done.body, EventBody::ScanFinished { changed: 2, .. }),
        "one scan covers every pick: {done:?}"
    );
    assert_eq!(
        done.plain.as_deref(),
        Some("Checked your folders: 2 shows changed.")
    );
}

fn attempt_version(core: &Core, series: u64) -> (Option<i64>, Option<i64>) {
    core.store()
        .read(|c| {
            Ok(c.query_row(
                "SELECT attempted_at, attempt_version FROM series WHERE id = ?1",
                rusqlite::params![series as i64],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?)
        })
        .unwrap()
}

#[test]
fn a_cleared_match_that_still_finds_nothing_stays_cleared() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let loose = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/lib/Zzz Unknown",
        "Zzz Unknown",
    );
    fixtures::insert_episode_file(&core, loose, "/lib/Zzz Unknown/01.mkv", 1.0, 1);
    core.call(Call::ClearMatch { series: loose }).unwrap();
    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "media": [] } } }),
    );
    http.push_for("api.tvmaze.com/search/shows", 200, "[]");

    let job = job_of(
        core.call(Call::ActOnSeries {
            series: vec![loose],
            action: SeriesAction::FindMatch,
        })
        .unwrap(),
    );
    common::wait_job(&c, job);
    let (at, version) = attempt_version(&core, loose);
    assert!(at.is_some());
    assert_eq!(
        version, None,
        "still the user's clear, which no matcher version resets"
    );
}

#[test]
fn a_search_that_fails_is_said_as_one_to_try_again_not_a_miss() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let loose = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/lib/Zzz Unknown",
        "Zzz Unknown",
    );
    fixtures::insert_episode_file(&core, loose, "/lib/Zzz Unknown/01.mkv", 1.0, 1);
    // A reply that does not parse: the provider answered, wrongly. (An error
    // reply reads as the provider gone, which ends the job the way the
    // automatic run ends.)
    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "media": [ { "id": "not a number" } ] } } }),
    );

    let job = job_of(
        core.call(Call::ActOnSeries {
            series: vec![loose],
            action: SeriesAction::FindMatch,
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.level, Level::Warn, "{done:?}");
    assert_eq!(
        plain_lines(&c),
        vec!["1 show could not be searched; try again later.".to_string()]
    );
    assert_eq!(attempt_version(&core, loose).0, None, "nothing stamped");
}

#[test]
fn a_pick_gone_from_disk_is_not_counted_as_matched() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/away");
    let gone = fixtures::insert_series(&core, src, SeriesKind::Show, "/away/Gone", "Gone");
    fixtures::mark_missing(&core, gone);
    let ids = matched_shows(&core, &["Bebop"]);

    let job = job_of(
        core.call(Call::ActOnSeries {
            series: vec![gone, ids[0]],
            action: SeriesAction::FindMatch,
        })
        .unwrap(),
    );
    common::wait_job(&c, job);
    assert_eq!(
        plain_lines(&c),
        vec!["1 show had a match already.".to_string()]
    );
}

#[test]
fn a_hidden_pick_is_named_as_hidden_not_as_unmatched() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let ids = matched_shows(&core, &["Bebop", "Trigun"]);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    core.call(Call::SetHidden {
        series: ids[1],
        hidden: true,
    })
    .unwrap();
    anilist_write(&http);

    let job = job_of(
        core.call(Call::ActOnSeries {
            series: ids,
            action: SeriesAction::ListStatus {
                status: ListStatus::Planning,
            },
        })
        .unwrap(),
    );
    common::wait_job(&c, job);
    assert_eq!(
        plain_lines(&c),
        vec!["Set Bebop to Planning on AniList. 1 show is hidden, so its sync is off.".to_string()]
    );
}
