//! Where an episode's intro and outro are: the file's own chapters first,
//! AniSkip second, and the answer cached on the episode so the next play
//! costs nothing.

mod common;
mod fixtures;

use std::time::Duration;

use anibeam_core::events::Collector;
use anibeam_core::net::FakeHttp;
use anibeam_core::*;

/// Eight days, comfortably past the seven day retry window on a miss.
const EIGHT_DAYS: i64 = 8 * 24 * 60 * 60;

/// Two episodes and one opening under a series matched to both providers,
/// so AniSkip has a MAL id to be asked about.
struct Library {
    series: u64,
    ep2: u64,
    extra: u64,
}

fn library(core: &Core) -> Library {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    fixtures::insert_file(
        core,
        series,
        "/lib/Bebop/Episode 1.mkv",
        1.0,
        None,
        "episode",
        now,
    );
    let ep2 = fixtures::insert_file(
        core,
        series,
        "/lib/Bebop/Episode 2.mkv",
        2.0,
        None,
        "episode",
        now,
    );
    let extra = fixtures::insert_file(core, series, "/lib/Bebop/OP1.mkv", 1.0, None, "extra", now);
    fixtures::insert_media(
        core,
        1,
        Some("Cowboy Bebop"),
        None,
        Some(26),
        "FINISHED",
        "TV",
        Some(86),
    );
    fixtures::match_series(core, series, Some(1), Some(21));
    Library { series, ep2, extra }
}

fn open(core: &Core, file: u64) -> PlaybackSession {
    match core.call(Call::OpenPlayback { file }).unwrap() {
        Reply::Playback { session } => *session,
        other => panic!("{other:?}"),
    }
}

fn chapter(title: &str, start: f64) -> Chapter {
    Chapter {
        title: title.to_string(),
        start,
    }
}

fn report(core: &Core, session: u64, chapters: Vec<Chapter>, duration: f64) -> u64 {
    match core
        .call(Call::ReportChapters {
            session,
            chapters,
            duration,
        })
        .unwrap()
    {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

/// The terminal event of the SkipWindows job `job`, waited for.
fn ready(c: &Collector, job: u64) -> Event {
    let e = common::wait_job(c, job);
    assert!(
        matches!(e.body, EventBody::SkipWindowsReady { .. }),
        "{:?}",
        e.body
    );
    e
}

fn windows_of(e: &Event) -> Vec<SkipWindow> {
    match &e.body {
        EventBody::SkipWindowsReady { windows, .. } => windows.clone(),
        other => panic!("{other:?}"),
    }
}

/// The cached row as (windows JSON, source), or nothing when the episode
/// has no row at all.
fn cache_row(core: &Core, series: u64, key: &str) -> Option<(String, String)> {
    let key = key.to_string();
    core.store()
        .read(|c| {
            Ok(c.query_row(
                "SELECT windows, source FROM skip_windows WHERE series_id = ?1 AND episode_key = ?2",
                rusqlite::params![series as i64, key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok())
        })
        .unwrap()
}

fn completed_keys(core: &Core, series: u64) -> Vec<String> {
    core.store()
        .read(|c| {
            let mut stmt = c.prepare(
                "SELECT episode_key FROM completed WHERE series_id = ?1 ORDER BY episode_key",
            )?;
            let rows = stmt.query_map([series as i64], |r| r.get::<_, String>(0))?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .unwrap()
}

/// (1) The file says where its own opening and ending are, so the answer
/// is instant, AniSkip is never asked, and the row is cached as chapters.
#[test]
fn chapters_answer_without_a_request_and_cache_themselves() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    let s = open(&core, lib.ep2);

    let chapters = vec![
        chapter("Opening", 0.0),
        chapter("Part A", 90.0),
        chapter("Ending", 1300.0),
        chapter("Preview", 1390.0),
    ];
    let e = ready(&c, report(&core, s.session, chapters, 1400.0));
    assert_eq!(
        windows_of(&e),
        vec![
            SkipWindow {
                kind: SkipKind::Intro,
                start: 0.0,
                end: 90.0,
                source: SkipSource::Chapters
            },
            SkipWindow {
                kind: SkipKind::Outro,
                start: 1300.0,
                end: 1390.0,
                source: SkipSource::Chapters
            },
        ]
    );
    assert_eq!(e.message, "skip windows: intro, outro");
    assert_eq!(e.level, Level::Debug);
    assert!(http.requests().is_empty(), "{:?}", http.requests());

    let (json, source) = cache_row(&core, lib.series, "2").expect("the chapters answer is cached");
    assert_eq!(source, "chapters");
    assert_eq!(
        serde_json::from_str::<Vec<SkipWindow>>(&json).unwrap(),
        windows_of(&e)
    );

    // The call's own arguments are the only thing it fails on.
    assert_eq!(
        core.call(Call::ReportChapters {
            session: 9999,
            chapters: vec![],
            duration: 1400.0
        })
        .unwrap_err(),
        CoreError::NotFound {
            what: Entity::Session,
            id: 9999
        }
    );
    for duration in [0.0, -3.0, f64::NAN] {
        match core
            .call(Call::ReportChapters {
                session: s.session,
                chapters: vec![],
                duration,
            })
            .unwrap_err()
        {
            CoreError::Invalid { field, .. } => assert_eq!(field, "duration"),
            other => panic!("{other:?}"),
        }
    }
}

/// (2) No chapters, so AniSkip is asked by MAL id, its answer is cached,
/// and the outro reaches the session: a tick at the outro's start is the
/// end of the episode, well before the last thirty seconds.
#[test]
fn aniskip_answers_when_the_file_has_no_chapters_and_its_outro_completes_the_episode() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    let s = open(&core, lib.ep2);
    http.push_json(
        200,
        serde_json::json!({ "found": true, "results": [
            { "interval": { "startTime": 85.0, "endTime": 175.0 }, "skipType": "op" },
            { "interval": { "startTime": 1320.0, "endTime": 1410.0 }, "skipType": "mixed-ed" }
        ] }),
    );

    let e = ready(&c, report(&core, s.session, vec![], 1440.0));
    assert_eq!(
        windows_of(&e),
        vec![
            SkipWindow {
                kind: SkipKind::Intro,
                start: 85.0,
                end: 175.0,
                source: SkipSource::AniSkip
            },
            SkipWindow {
                kind: SkipKind::Outro,
                start: 1320.0,
                end: 1410.0,
                source: SkipSource::AniSkip
            },
        ]
    );
    assert_eq!(e.message, "skip windows: intro, outro");
    let requests = http.requests();
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(
        requests[0].url,
        "https://api.aniskip.com/v2/skip-times/21/2?types[]=op&types[]=ed&episodeLength=1440"
    );
    assert_eq!(
        cache_row(&core, lib.series, "2").map(|(_, source)| source),
        Some("aniskip".to_string())
    );

    // The outro is the earlier of the two completion lines: 1320 is inside
    // it but nowhere near the last thirty seconds of 1440.
    assert_eq!(
        core.call(Call::Tick {
            session: s.session,
            position: 1320.0,
            paused: false
        })
        .unwrap(),
        Reply::Ok
    );
    assert_eq!(completed_keys(&core, lib.series), vec!["2".to_string()]);
    let cleared = common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ResumePointChanged { file, position: None } if file == lib.ep2),
        Duration::from_secs(1),
    );
    assert_eq!(cleared.level, Level::Debug);

    // The next session on the episode opens with the windows already on it.
    assert_eq!(open(&core, lib.ep2).skip_windows, windows_of(&e));
}

/// (3) A 404 is AniSkip saying it holds nothing, not a failure to ask, so
/// the miss is cached and the next session answers from it in silence.
#[test]
fn a_miss_is_cached_and_the_next_session_sends_no_request() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    let s = open(&core, lib.ep2);
    http.push(404, r#"{"found":false,"message":"Not Found"}"#);

    let e = ready(&c, report(&core, s.session, vec![], 1400.0));
    assert!(windows_of(&e).is_empty(), "{:?}", windows_of(&e));
    assert_eq!(e.message, "skip windows: none");
    assert_eq!(
        cache_row(&core, lib.series, "2"),
        Some(("[]".to_string(), "none".to_string()))
    );
    assert_eq!(http.requests().len(), 1);

    let again = ready(&c, report(&core, s.session, vec![], 1400.0));
    assert!(windows_of(&again).is_empty());
    assert_eq!(http.requests().len(), 1, "a cached miss must not ask again");
}

/// (4) A miss is only good for seven days, since AniSkip's data arrives
/// over time: eight days on, the third call asks again.
#[test]
fn a_miss_older_than_the_retry_window_is_asked_about_again() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    let s = open(&core, lib.ep2);
    http.push(404, r#"{"found":false}"#);

    ready(&c, report(&core, s.session, vec![], 1400.0));
    ready(&c, report(&core, s.session, vec![], 1400.0));
    assert_eq!(http.requests().len(), 1);

    fixtures::age_skip_cache(&core, lib.series, "2", EIGHT_DAYS);
    http.push_json(
        200,
        serde_json::json!({ "found": true, "results": [
            { "interval": { "startTime": 0.0, "endTime": 90.0 }, "skipType": "op" }
        ] }),
    );
    let e = ready(&c, report(&core, s.session, vec![], 1400.0));
    assert_eq!(
        windows_of(&e),
        vec![SkipWindow {
            kind: SkipKind::Intro,
            start: 0.0,
            end: 90.0,
            source: SkipSource::AniSkip
        }]
    );
    assert_eq!(e.message, "skip windows: intro");
    assert_eq!(http.requests().len(), 2);
    assert_eq!(
        cache_row(&core, lib.series, "2").map(|(_, source)| source),
        Some("aniskip".to_string())
    );
}

/// (5) An extra has no episode number, so AniSkip has nothing to be asked
/// about: its chapters are the only source it has.
#[test]
fn an_extra_uses_its_chapters_alone() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    let s = open(&core, lib.extra);

    let e = ready(&c, report(&core, s.session, vec![], 90.0));
    assert!(windows_of(&e).is_empty(), "{:?}", windows_of(&e));
    assert_eq!(e.message, "skip windows: none");
    assert!(http.requests().is_empty(), "{:?}", http.requests());
    assert_eq!(cache_row(&core, lib.series, "OP1.mkv"), None);

    // Its own chapters still answer, and those are cached like any other.
    let e = ready(
        &c,
        report(
            &core,
            s.session,
            vec![chapter("Opening", 0.0), chapter("Part A", 60.0)],
            90.0,
        ),
    );
    assert_eq!(
        windows_of(&e),
        vec![SkipWindow {
            kind: SkipKind::Intro,
            start: 0.0,
            end: 60.0,
            source: SkipSource::Chapters
        }]
    );
    assert!(http.requests().is_empty(), "{:?}", http.requests());
    assert_eq!(
        cache_row(&core, lib.series, "OP1.mkv").map(|(_, source)| source),
        Some("chapters".to_string())
    );
}
