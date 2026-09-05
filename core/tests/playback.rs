//! One run of the player over one file: what `OpenPlayback` tells the shell,
//! what a tick does to the history tables, and what a close leaves behind.

mod common;
mod fixtures;

use std::time::Duration;

use anibeam_core::events::Collector;
use anibeam_core::net::FakeHttp;
use anibeam_core::playback::session;
use anibeam_core::*;

/// Three episodes and one opening under a matched series, twelve episodes
/// published, both tracker ids carried.
struct Library {
    series: u64,
    ep1: u64,
    ep2: u64,
    ep3: u64,
    extra: u64,
}

fn library(core: &Core) -> Library {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    let ep1 = fixtures::insert_file(
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
    let ep3 = fixtures::insert_file(
        core,
        series,
        "/lib/Bebop/Episode 3.mkv",
        3.0,
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
        Some(12),
        "FINISHED",
        "TV",
        Some(86),
    );
    fixtures::insert_episode(core, 1, 2, Some("Stray Dog Strut"), None);
    fixtures::match_series(core, series, Some(1), Some(21));
    Library {
        series,
        ep1,
        ep2,
        ep3,
        extra,
    }
}

fn open(core: &Core, file: u64) -> PlaybackSession {
    match core.call(Call::OpenPlayback { file }).unwrap() {
        Reply::Playback { session } => *session,
        other => panic!("{other:?}"),
    }
}

fn settings(core: &Core) -> Settings {
    match core.call(Call::GetSettings).unwrap() {
        Reply::Settings { settings } => settings,
        other => panic!("{other:?}"),
    }
}

fn detail(core: &Core, series: u64) -> SeriesDetail {
    match core.call(Call::GetSeries { series }).unwrap() {
        Reply::SeriesDetail { detail } => *detail,
        other => panic!("{other:?}"),
    }
}

fn tick(core: &Core, session: u64, position: f64) {
    assert_eq!(
        core.call(Call::Tick {
            session,
            position,
            paused: false
        })
        .unwrap(),
        Reply::Ok
    );
}

/// Every resume point the session has announced so far, in order.
fn resume_positions(c: &Collector) -> Vec<Option<f64>> {
    c.bodies()
        .into_iter()
        .filter_map(|b| match b {
            EventBody::ResumePointChanged { position, .. } => Some(position),
            _ => None,
        })
        .collect()
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

fn resume_row(core: &Core, series: u64, key: &str) -> Option<(f64, f64)> {
    let key = key.to_string();
    // The resume upsert is posted, not written, so a read straight after a
    // tick can beat it to the connection. An empty write is the barrier:
    // the writer thread runs its queue in order, so this one lands last.
    core.store().write(|_| Ok(())).unwrap();
    core.store()
        .read(|c| {
            Ok(c.query_row(
                "SELECT position, duration FROM resume_points WHERE series_id = ?1 AND episode_key = ?2",
                rusqlite::params![series as i64, key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok())
        })
        .unwrap()
}

fn put_resume_row(core: &Core, series: u64, key: &str, position: f64) {
    let key = key.to_string();
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT INTO resume_points (series_id, episode_key, position, duration, at) VALUES (?1, ?2, ?3, 1400, ?4)",
                rusqlite::params![series as i64, key, position, anibeam_core::time::now_secs()],
            )?;
            Ok(())
        })
        .unwrap();
}

/// (1) The player has everything it needs to start: where it sits in the
/// series, where to seek to, and what to call the episode.
#[test]
fn open_playback_describes_the_episode_and_its_neighbours() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);

    let s = open(&core, lib.ep2);
    assert!(s.session > 0);
    assert_eq!(s.file, lib.ep2);
    assert_eq!(s.series, lib.series);
    assert_eq!(s.path, "/lib/Bebop/Episode 2.mkv");
    assert_eq!(s.series_title, "Cowboy Bebop");
    assert_eq!(s.episode_title.as_deref(), Some("Stray Dog Strut"));
    assert_eq!(s.code, "EP 2");
    assert!(!s.is_extra);
    assert!(!s.is_last_episode);
    assert_eq!(s.resume_from, None);
    assert_eq!(s.prev, Some(lib.ep1));
    assert_eq!(s.next, Some(lib.ep3));
    assert!(s.skip_windows.is_empty());
    assert_eq!(s.artwork, None);
    assert_eq!(s.track_choice, TrackChoice::default());
    assert_eq!(s.subtitle_defaults, settings(&core).subtitle_defaults);

    // An extra stands outside the episode order and is never the last one.
    let x = open(&core, lib.extra);
    assert_ne!(x.session, s.session);
    assert_eq!(x.code, "OP1");
    assert!(x.is_extra);
    assert!(!x.is_last_episode);
    assert_eq!(x.prev, None);
    assert_eq!(x.next, None);

    // A stored point is where the shell seeks to before the first frame.
    put_resume_row(&core, lib.series, "3", 620.0);
    assert_eq!(open(&core, lib.ep3).resume_from, Some(620.0));

    assert_eq!(
        core.call(Call::OpenPlayback { file: 9999 }).unwrap_err(),
        CoreError::NotFound {
            what: Entity::File,
            id: 9999
        }
    );
}

/// Where an episode sits in its series: the published total decides the last
/// one, the disk decides when no total is known, a film is the whole of what
/// it is, and a half-numbered recap stands outside the running order.
#[test]
fn the_last_episode_and_the_neighbours_follow_the_total_and_the_disk() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    let now = anibeam_core::time::now_secs();
    assert!(!open(&core, lib.ep3).is_last_episode);

    let twelfth = fixtures::insert_file(
        &core,
        lib.series,
        "/lib/Bebop/Episode 12.mkv",
        12.0,
        None,
        "episode",
        now,
    );
    let thirteenth = fixtures::insert_file(
        &core,
        lib.series,
        "/lib/Bebop/Episode 13.mkv",
        13.0,
        None,
        "episode",
        now,
    );
    let recap = fixtures::insert_file(
        &core,
        lib.series,
        "/lib/Bebop/Episode 12.5.mkv",
        12.5,
        None,
        "episode",
        now,
    );
    // Twelve of twelve published, whatever else turned up on disk.
    assert!(open(&core, twelfth).is_last_episode);
    // The recap takes the whole episodes either side of it, and is never a
    // neighbour itself.
    let r = open(&core, recap);
    assert_eq!(r.code, "EP 12.5");
    assert_eq!(r.prev, Some(twelfth));
    assert_eq!(r.next, Some(thirteenth));
    assert_eq!(open(&core, thirteenth).prev, Some(twelfth));

    // No published total: the last episode is the last one on disk.
    let src = fixtures::insert_source(&core, "/other");
    let unmatched = fixtures::insert_series(&core, src, SeriesKind::Show, "/other/Show", "Show");
    fixtures::insert_file(
        &core,
        unmatched,
        "/other/Show/Episode 1.mkv",
        1.0,
        None,
        "episode",
        now,
    );
    let last = fixtures::insert_file(
        &core,
        unmatched,
        "/other/Show/Episode 2.mkv",
        2.0,
        None,
        "episode",
        now,
    );
    assert!(open(&core, last).is_last_episode);

    // A film has nothing either side of it and is always its own last.
    let film_series =
        fixtures::insert_series(&core, src, SeriesKind::Movie, "/other/Film.mkv", "Film");
    let film = fixtures::insert_file(
        &core,
        film_series,
        "/other/Film.mkv",
        1.0,
        None,
        "episode",
        now,
    );
    let f = open(&core, film);
    assert!(f.is_last_episode);
    assert_eq!(f.prev, None);
    assert_eq!(f.next, None);
}

/// (2) to (5) One episode watched end to end: the view at thirty seconds, a
/// resume point on every tick after the fifth, the mark at 85 percent, the
/// completion inside the last thirty seconds, and a close that adds nothing.
#[test]
fn a_session_views_marks_completes_and_clears_its_resume_point() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"MediaList":{"progress":1,"status":"CURRENT"}}}"#,
    );
    http.push_for(
        "anilist",
        200,
        r#"{"data":{"SaveMediaListEntry":{"id":9,"progress":2,"status":"CURRENT"}}}"#,
    );

    let s = open(&core, lib.ep2);

    // (2) Thirty-five ticks a second apart. Thirty seconds of forward
    // movement is a view, and every tick from the fifth second on saves
    // where the playhead is.
    for step in 0..35 {
        tick(&core, s.session, f64::from(step));
    }
    let viewed: Vec<EventBody> = c
        .bodies()
        .into_iter()
        .filter(|b| matches!(b, EventBody::Viewed { .. }))
        .collect();
    assert_eq!(
        viewed,
        vec![EventBody::Viewed {
            series: lib.series,
            episode: "2".to_string()
        }]
    );
    let line = common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::Viewed { .. }),
        Duration::from_secs(1),
    );
    assert_eq!(line.message, "viewed Cowboy Bebop EP 2");
    assert_eq!(line.level, Level::Info);
    // Every four seconds from the fifth, not every tick.
    assert_eq!(
        resume_positions(&c),
        [5.0, 9.0, 13.0, 17.0, 21.0, 25.0, 29.0, 33.0]
            .map(Some)
            .to_vec()
    );
    assert_eq!(resume_row(&core, lib.series, "2"), Some((33.0, 0.0)));
    let ep2 = |d: &SeriesDetail| {
        d.episodes
            .iter()
            .find(|e| e.file == lib.ep2)
            .expect("episode 2 is on disk")
            .clone()
    };
    assert_eq!(
        ep2(&detail(&core, lib.series)).resume,
        Some(ResumePoint {
            position: 33.0,
            duration: 0.0
        })
    );

    // (3) A seek to the credits with no duration known marks nothing: there
    // is nothing to be 85 percent of yet. The duration arrives, and the next
    // tick crosses 1190.
    tick(&core, s.session, 1200.0);
    assert!(
        !c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::Marked { .. }))
    );
    session::report_chapters(&core, s.session, 1400.0, None);
    tick(&core, s.session, 1201.0);
    let marked = common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::Marked { .. }),
        Duration::from_secs(30),
    );
    match marked.body {
        EventBody::Marked {
            series, episode, ..
        } => {
            assert_eq!(series, lib.series);
            assert_eq!(episode, 2);
        }
        other => panic!("{other:?}"),
    }

    // (4) Inside the last thirty seconds the episode is done: the resume
    // point goes, the completion is recorded, and the card behind the player
    // is pushed so Next up moves on.
    let seen = c.events().len();
    tick(&core, s.session, 1375.0);
    let after: Vec<EventBody> = c.bodies().into_iter().skip(seen).collect();
    assert!(after.iter().any(|b| matches!(b, EventBody::ResumePointChanged { file, position: None } if *file == lib.ep2)), "{after:?}");
    assert!(
        after.iter().any(|b| matches!(b, EventBody::SeriesChanged { series } if series.iter().any(|card| card.id == lib.series))),
        "{after:?}"
    );
    assert_eq!(completed_keys(&core, lib.series), vec!["2".to_string()]);
    assert_eq!(resume_row(&core, lib.series, "2"), None);
    let d = detail(&core, lib.series);
    assert!(ep2(&d).watched);
    assert_eq!(ep2(&d).resume, None);
    assert_eq!(d.next_up, Some(lib.ep3));

    // (5) The file runs out and the shell closes the session. Everything the
    // rules had to say has been said, so the close writes nothing, and a
    // second one finds no session and is quiet about it.
    let before = c.events().len();
    for _ in 0..2 {
        let reply = core
            .call(Call::ClosePlayback {
                session: s.session,
                position: 1375.0,
                reason: CloseReason::Ended,
            })
            .unwrap();
        assert_eq!(reply, Reply::Ok);
    }
    assert_eq!(c.events().len(), before);
    assert_eq!(completed_keys(&core, lib.series), vec!["2".to_string()]);
    assert_eq!(
        c.bodies()
            .iter()
            .filter(|b| matches!(b, EventBody::Marked { .. }))
            .count(),
        1
    );

    // A tick on a closed session is the shell talking about something that
    // no longer exists.
    assert_eq!(
        core.call(Call::Tick {
            session: s.session,
            position: 1376.0,
            paused: false
        })
        .unwrap_err(),
        CoreError::NotFound {
            what: Entity::Session,
            id: s.session
        }
    );
}

/// (6) An extra shares its number with a real episode, so it never moves the
/// view history or a tracker. It only remembers where it was.
#[test]
fn an_extras_session_records_no_view_and_no_mark() {
    let (_dir, core, c) = common::open_core();
    let lib = library(&core);

    let s = open(&core, lib.extra);
    session::report_chapters(&core, s.session, 100.0, None);
    for step in 0..40 {
        tick(&core, s.session, f64::from(step));
    }
    assert!(
        !c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::Viewed { .. }))
    );
    assert_eq!(
        resume_row(&core, lib.series, "OP1.mkv"),
        Some((37.0, 100.0))
    );

    // Past 85 percent and inside the last thirty seconds at once: the extra
    // marks nothing, records no completion, and only forgets where it was.
    let seen = c.events().len();
    tick(&core, s.session, 90.0);
    let after: Vec<EventBody> = c.bodies().into_iter().skip(seen).collect();
    assert_eq!(
        after,
        vec![EventBody::ResumePointChanged {
            file: lib.extra,
            position: None
        }]
    );
    assert_eq!(resume_row(&core, lib.series, "OP1.mkv"), None);
    assert!(completed_keys(&core, lib.series).is_empty());
    assert!(
        !c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::Marked { .. }))
    );
    assert!(
        detail(&core, lib.series)
            .extras
            .iter()
            .all(|x| x.resume.is_none())
    );
}

/// (7) A session the player never reported on says nothing about where the
/// playhead is, so it must not wipe a real resume point with a zero.
#[test]
fn a_close_with_no_tick_leaves_the_resume_point_alone() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    put_resume_row(&core, lib.series, "1", 500.0);

    let s = open(&core, lib.ep1);
    let reply = core
        .call(Call::ClosePlayback {
            session: s.session,
            position: 3.0,
            reason: CloseReason::Stopped,
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
    assert_eq!(resume_row(&core, lib.series, "1"), Some((500.0, 1400.0)));
}

/// (8) The track choice is the series' memory, so the next episode opens on
/// the audio and subtitles the last one was watched with.
#[test]
fn a_track_choice_round_trips_through_open_playback() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);

    let audio = TrackRef {
        kind: TrackKind::Embedded,
        language: Some("jpn".into()),
        title: None,
    };
    let subtitle = SubtitleChoice::Track {
        track: TrackRef {
            kind: TrackKind::Sidecar,
            language: Some("eng".into()),
            title: Some("Full".into()),
        },
    };
    let reply = core
        .call(Call::SetTrackChoice {
            series: lib.series,
            audio: Some(audio.clone()),
            subtitle: Some(subtitle.clone()),
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);

    let s = open(&core, lib.ep2);
    assert_eq!(
        s.track_choice,
        TrackChoice {
            audio: Some(audio),
            subtitle: Some(subtitle)
        }
    );

    // Off is a choice of its own and survives the round trip.
    core.call(Call::SetTrackChoice {
        series: lib.series,
        audio: None,
        subtitle: Some(SubtitleChoice::Off),
    })
    .unwrap();
    assert_eq!(
        open(&core, lib.ep2).track_choice,
        TrackChoice {
            audio: None,
            subtitle: Some(SubtitleChoice::Off)
        }
    );

    assert_eq!(
        core.call(Call::SetTrackChoice {
            series: 4242,
            audio: None,
            subtitle: None
        })
        .unwrap_err(),
        CoreError::NotFound {
            what: Entity::Series,
            id: 4242
        }
    );
}

/// (9) The resume upsert is throttled to Electron's four seconds, so a
/// forty minute episode costs a few hundred writes rather than one per
/// tick. A pause and the close still write where the playhead actually
/// stopped, which is the number the shell resumes from.
#[test]
fn the_resume_point_is_throttled_but_a_pause_and_the_close_are_not() {
    let (_dir, core, c) = common::open_core();
    let lib = library(&core);

    let s = open(&core, lib.ep2);
    session::report_chapters(&core, s.session, 1400.0, None);
    for step in 0..12 {
        tick(&core, s.session, f64::from(step));
    }
    assert_eq!(
        resume_positions(&c),
        [5.0, 9.0].map(Some).to_vec(),
        "a write per tick is back"
    );

    // A pause one second on writes anyway: it could be the last word.
    core.call(Call::Tick {
        session: s.session,
        position: 12.0,
        paused: true,
    })
    .unwrap();
    assert_eq!(resume_row(&core, lib.series, "2"), Some((12.0, 1400.0)));

    // And so does the close, whatever the throttle says.
    core.call(Call::ClosePlayback {
        session: s.session,
        position: 13.0,
        reason: CloseReason::Stopped,
    })
    .unwrap();
    assert_eq!(resume_row(&core, lib.series, "2"), Some((13.0, 1400.0)));
}
