//! Watch together rooms (#846) end to end: the native relay runs in this
//! process, and two or three cores, each with its own library of the same show,
//! create, join, follow each other and leave.

mod common;
mod fixtures;

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anibeam_core::events::Collector;
use anibeam_core::*;

const WAIT: Duration = Duration::from_secs(15);

/// The relay on a port of its own, alive as long as this is.
struct Relay {
    _rt: tokio::runtime::Runtime,
    url: String,
}

fn relay() -> Relay {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let listener = rt
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let addr = listener.local_addr().unwrap();
    rt.spawn(async move {
        let _ = anibeam_relay::serve(listener).await;
    });
    Relay {
        _rt: rt,
        url: format!("http://{addr}"),
    }
}

/// One person: a core whose library has Cowboy Bebop with some of its episodes.
struct Person {
    _dir: common::Dir,
    core: Arc<Core>,
    c: Arc<Collector>,
    series: u64,
    /// The file of each episode asked for, in the order asked.
    files: Vec<u64>,
}

/// `anilist` false matches the show by its MAL id alone.
fn person(relay: &Relay, name: &str, episodes: &[f64], anilist: bool) -> Person {
    let (dir, core, c) = common::open_core();
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(&core, "/lib");
    // A show before it, so this library's ids are not the host's.
    let other = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Other", "Other");
    fixtures::insert_episode_file(&core, other, "/lib/Other/Episode 1.mkv", 1.0, now);
    let series =
        fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    let files = episodes
        .iter()
        .map(|n| {
            fixtures::insert_episode_file(
                &core,
                series,
                &format!("/lib/Bebop/Episode {n}.mkv"),
                *n,
                now,
            )
        })
        .collect();
    fixtures::media(1, "Cowboy Bebop")
        .episodes(26)
        .insert(&core);
    fixtures::match_series(&core, series, anilist.then_some(1), Some(21));
    let mut p = match core.call(Call::GetPreferences).unwrap() {
        Reply::Preferences { preferences } => preferences,
        other => panic!("{other:?}"),
    };
    p.room_name = name.into();
    p.room_relay = relay.url.clone();
    core.call(Call::SetPreferences { preferences: p }).unwrap();
    core.set_room_correction_gap(Duration::ZERO);
    Person {
        _dir: dir,
        core,
        c,
        series,
        files,
    }
}

fn open(p: &Person, file: u64) -> u64 {
    match p.core.call(Call::OpenPlayback { file }).unwrap() {
        Reply::Playback { session } => session.session,
        other => panic!("{other:?}"),
    }
}

/// Reports chapters with the opening at `opening`, and waits for the skip windows.
fn chapters(p: &Person, session: u64, opening: f64) {
    let chapters = vec![
        Chapter {
            title: "Opening".into(),
            start: opening,
        },
        Chapter {
            title: "Part A".into(),
            start: opening + 90.0,
        },
        Chapter {
            title: "Ending".into(),
            start: 1300.0,
        },
        Chapter {
            title: "Preview".into(),
            start: 1390.0,
        },
    ];
    p.core
        .call(Call::ReportChapters {
            session,
            chapters,
            duration: 1420.0,
        })
        .unwrap();
    common::wait_for(
        &p.c,
        |e| matches!(&e.body, EventBody::SkipWindowsReady { session: s, .. } if *s == session),
        WAIT,
    );
}

fn room(reply: Result<Reply, CoreError>) -> Option<RoomInfo> {
    match reply.unwrap() {
        Reply::Room { room } => room,
        other => panic!("{other:?}"),
    }
}

fn said(e: &Event, id: &str) -> bool {
    e.plain_words.as_ref().is_some_and(|w| w.id == id)
}

fn said_name(e: &Event, id: &str, name: &str) -> bool {
    e.plain_words.as_ref().is_some_and(|w| {
        w.id == id
            && w.args.get("name")
                == Some(&WordValue::Text {
                    value: name.to_string(),
                })
    })
}

/// Waits until `count` events satisfy `pred`.
fn wait_count(c: &Collector, count: usize, pred: impl Fn(&Event) -> bool) {
    let arrived = c.wait_for(
        |events| events.iter().filter(|e| pred(e)).count() >= count,
        WAIT,
    );
    assert!(arrived, "timed out waiting; saw {:#?}", c.events());
}

fn applied(c: &Collector, session: u64, pred: impl Fn(bool, Option<f64>, f64) -> bool) -> Event {
    common::wait_for(
        c,
        |e| match &e.body {
            EventBody::RoomApply {
                session: s,
                playing,
                seek_to,
                speed,
            } => *s == session && pred(*playing, *seek_to, *speed),
            _ => false,
        },
        WAIT,
    )
}

/// The code as a person might type it: lowercase, with a space and a dash.
fn messy(code: &str) -> String {
    let lower = code.to_lowercase();
    format!(" {} {}-{} ", &lower[..2], &lower[2..4], &lower[4..])
}

fn status(p: &Person) -> Option<RoomInfo> {
    room(p.core.call(Call::RoomStatus))
}

fn service_code(err: CoreError) -> String {
    match err {
        CoreError::Service { code, .. } => code,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_room_keeps_two_players_together() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0, 2.0], true);
    let ben = person(&relay, "Ben", &[2.0, 1.0], true);

    // Ann plays episode 1, whose opening is at 90 seconds, and starts a room.
    let ann_s = open(&ann, ann.files[0]);
    chapters(&ann, ann_s, 90.0);
    let made = room(ann.core.call(Call::RoomCreate { session: ann_s })).unwrap();
    assert_eq!(made.code.len(), 6);
    assert_eq!(made.link, format!("{}/{}", relay.url, made.code));
    assert!(made.host);
    assert!(made.connected);
    assert_eq!(made.session, Some(ann_s));
    assert_eq!(made.members.len(), 1);
    assert_eq!(made.episode.anilist_id, Some(1));
    assert_eq!(made.episode.mal_id, Some(21));
    assert_eq!(made.episode.episode_key, "1");
    assert_eq!(made.episode.series_title, "Cowboy Bebop");
    assert_eq!(made.episode.opening_start, Some(90.0));
    assert_eq!(made.episode.duration, Some(1420.0));
    common::wait_for(&ann.c, |e| said(e, "core-room-created"), WAIT);

    // Ben joins with the code typed loosely, and is told which of his files it is.
    let joined = room(ben.core.call(Call::RoomJoin {
        code: messy(&made.code),
    }))
    .unwrap();
    assert_eq!(joined.code, made.code);
    assert!(!joined.host);
    assert_eq!(joined.members.len(), 2);
    assert_ne!(joined.you, made.you);
    common::wait_for(
        &ben.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { series, file, episode }
                if *series == Some(ben.series) && *file == Some(ben.files[1]) && episode.episode_key == "1")
        },
        WAIT,
    );
    common::wait_for(&ben.c, |e| said(e, "core-room-joined"), WAIT);
    common::wait_for(
        &ann.c,
        |e| said_name(e, "core-room-member-joined", "Ben"),
        WAIT,
    );

    // Ben's copy has its opening 5 seconds later: the room suggests and applies +5,
    // and his player goes to the room's start on his file.
    let ben_s = open(&ben, ben.files[1]);
    chapters(&ben, ben_s, 95.0);
    let attached = room(ben.core.call(Call::RoomAttach { session: ben_s })).unwrap();
    assert_eq!(attached.suggested_offset, Some(5.0));
    assert_eq!(attached.offset, 5.0);
    assert_eq!(attached.session, Some(ben_s));
    applied(&ben.c, ben_s, |playing, seek, _| {
        !playing && seek == Some(5.0)
    });
    common::wait_for(
        &ann.c,
        |e| {
            matches!(&e.body, EventBody::RoomChanged { room: Some(r) }
                if r.members.iter().any(|m| m.name == "Ben" && m.ready == RoomReady::Ready))
        },
        WAIT,
    );

    // Ann plays from 100: Ben's player plays from 105 on his file.
    ann.core
        .call(Call::RoomCommand {
            action: RoomAction::Play,
            position: 100.0,
        })
        .unwrap();
    applied(&ben.c, ben_s, |playing, seek, speed| {
        playing && seek.is_some_and(|t| (105.0..110.0).contains(&t)) && speed == 1.0
    });

    // Ben pauses at 205 on his file: Ann's player pauses at 200 on hers.
    ben.core
        .call(Call::RoomCommand {
            action: RoomAction::Pause,
            position: 205.0,
        })
        .unwrap();
    applied(&ann.c, ann_s, |playing, seek, _| {
        !playing && seek == Some(200.0)
    });

    // Paused, a player that drifted half a second or more is put back.
    ben.core
        .call(Call::Tick {
            session: ben_s,
            position: 210.0,
            paused: true,
        })
        .unwrap();
    applied(&ben.c, ben_s, |playing, seek, _| {
        !playing && seek == Some(205.0)
    });

    // Playing, a player a second behind speeds up, and slows back once level.
    ann.core
        .call(Call::RoomCommand {
            action: RoomAction::Play,
            position: 300.0,
        })
        .unwrap();
    let start = applied(&ben.c, ben_s, |playing, seek, _| {
        playing && seek.is_some_and(|t| (305.0..310.0).contains(&t))
    });
    let EventBody::RoomApply {
        seek_to: Some(seek),
        ..
    } = start.body
    else {
        unreachable!()
    };
    let now = |late: f64| {
        seek + SystemTime::now()
            .duration_since(start.at)
            .unwrap_or_default()
            .as_secs_f64()
            - late
    };
    ben.core
        .call(Call::Tick {
            session: ben_s,
            position: now(1.0),
            paused: false,
        })
        .unwrap();
    applied(&ben.c, ben_s, |playing, seek, speed| {
        playing && seek.is_none() && speed == 1.05
    });
    ben.core
        .call(Call::Tick {
            session: ben_s,
            position: now(0.0),
            paused: false,
        })
        .unwrap();
    applied(&ben.c, ben_s, |playing, seek, speed| {
        playing && seek.is_none() && speed == 1.0
    });

    // An offset set by hand moves the player and is kept for the file.
    let moved = room(ben.core.call(Call::RoomSetOffset { offset: 4.5 })).unwrap();
    assert_eq!(moved.offset, 4.5);
    assert_eq!(moved.suggested_offset, Some(5.0));
    applied(&ben.c, ben_s, |playing, seek, _| {
        playing && seek.is_some_and(|t| (304.5..310.0).contains(&t))
    });
    ben.core
        .call(Call::ClosePlayback {
            session: ben_s,
            position: 306.0,
            reason: CloseReason::Stopped,
        })
        .unwrap();
    let again = open(&ben, ben.files[1]);
    let kept = room(ben.core.call(Call::RoomAttach { session: again })).unwrap();
    assert_eq!(kept.offset, 4.5, "the offset stored for the file wins");

    // Corrections never reach the activity log.
    assert!(
        ben.c
            .events()
            .iter()
            .filter(|e| matches!(e.body, EventBody::RoomApply { .. }))
            .all(|e| e.level == Level::Debug && e.plain_words.is_none())
    );
}

#[test]
fn a_missing_episode_is_said_and_the_host_moves_everyone_on() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0, 2.0], true);
    let ben = person(&relay, "Ben", &[1.0, 2.0], true);
    // Cid has only episode 2, matched by MAL alone.
    let cid = person(&relay, "Cid", &[2.0], false);

    let ann_1 = open(&ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate { session: ann_1 })).unwrap();

    room(cid.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    common::wait_for(
        &cid.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { series, file, .. }
                if *series == Some(cid.series) && file.is_none())
        },
        WAIT,
    );
    common::wait_for(
        &ann.c,
        |e| {
            matches!(&e.body, EventBody::RoomChanged { room: Some(r) }
                if r.members.iter().any(|m| m.name == "Cid" && m.ready == RoomReady::Missing))
        },
        WAIT,
    );

    room(ben.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    let ben_1 = open(&ben, ben.files[0]);
    room(ben.core.call(Call::RoomAttach { session: ben_1 }));

    // Ann's player moves on to episode 2: the room follows.
    let ann_2 = open(&ann, ann.files[1]);
    let moved = room(ann.core.call(Call::RoomAttach { session: ann_2 })).unwrap();
    assert_eq!(moved.episode.episode_key, "2");
    applied(&ann.c, ann_2, |playing, seek, _| {
        !playing && seek == Some(0.0)
    });
    // Ben's old episode stops, and he is told which file is episode 2.
    applied(&ben.c, ben_1, |playing, seek, _| !playing && seek.is_none());
    common::wait_for(
        &ben.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { file, episode, .. }
                if *file == Some(ben.files[1]) && episode.episode_key == "2")
        },
        WAIT,
    );
    // Cid has that one.
    common::wait_for(
        &cid.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { file, episode, .. }
                if *file == Some(cid.files[0]) && episode.episode_key == "2")
        },
        WAIT,
    );
    let ben_room = status(&ben).unwrap();
    assert_eq!(ben_room.episode.episode_key, "2");
    assert_eq!(ben_room.session, None);
}

#[test]
fn a_dropped_connection_comes_back_and_leaving_is_for_good() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0], true);
    let ben = person(&relay, "Ben", &[1.0], true);

    let ann_s = open(&ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate { session: ann_s })).unwrap();
    let joined = room(ben.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }))
    .unwrap();

    // The network drops Ben: he says so, comes back as the same member, and says
    // that too.
    ben.core.room_drop_connection();
    common::wait_for(&ben.c, |e| said(e, "core-room-lost"), WAIT);
    common::wait_for(&ben.c, |e| said(e, "core-room-back"), WAIT);
    let back = status(&ben).unwrap();
    assert!(back.connected);
    assert_eq!(back.you, joined.you);
    wait_count(&ann.c, 2, |e| {
        said_name(e, "core-room-member-joined", "Ben")
    });

    // Leaving is for good: Ann sees him go, and Ben is in no room.
    assert_eq!(room(ben.core.call(Call::RoomLeave)), None);
    common::wait_for(&ben.c, |e| said(e, "core-room-left"), WAIT);
    wait_count(&ann.c, 2, |e| said_name(e, "core-room-member-left", "Ben"));
    assert_eq!(status(&ben), None);
    assert_eq!(status(&ann).unwrap().members.len(), 1);
    let err = ben
        .core
        .call(Call::RoomCommand {
            action: RoomAction::Play,
            position: 1.0,
        })
        .unwrap_err();
    assert_eq!(service_code(err), "not_in_room");
}

#[test]
fn joining_needs_a_name_and_a_real_code() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0], true);

    let err = ann
        .core
        .call(Call::RoomJoin {
            code: "nope".into(),
        })
        .unwrap_err();
    assert!(
        matches!(err, CoreError::Invalid { ref field, .. } if field == "code"),
        "{err:?}"
    );

    let err = ann
        .core
        .call(Call::RoomJoin {
            code: "abc def".into(),
        })
        .unwrap_err();
    assert_eq!(service_code(err), "not_found");
    assert_eq!(status(&ann), None);

    let mut p = match ann.core.call(Call::GetPreferences).unwrap() {
        Reply::Preferences { preferences } => preferences,
        other => panic!("{other:?}"),
    };
    p.room_name = String::new();
    ann.core
        .call(Call::SetPreferences {
            preferences: p.clone(),
        })
        .unwrap();
    let err = ann
        .core
        .call(Call::RoomJoin {
            code: "ABCDEF".into(),
        })
        .unwrap_err();
    assert!(
        matches!(err, CoreError::Invalid { ref field, .. } if field == "room_name"),
        "{err:?}"
    );

    p.room_relay = "rooms.example.com".into();
    let err = ann
        .core
        .call(Call::SetPreferences { preferences: p })
        .unwrap_err();
    assert!(
        matches!(err, CoreError::Invalid { ref field, .. } if field == "room_relay"),
        "{err:?}"
    );
}
