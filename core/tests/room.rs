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
    let made = room(ann.core.call(Call::RoomCreate {
        session: Some(ann_s),
    }))
    .unwrap();
    assert_eq!(made.code.len(), 6);
    assert_eq!(made.link, format!("{}/{}", relay.url, made.code));
    assert!(made.host);
    assert!(made.connected);
    assert_eq!(made.session, Some(ann_s));
    assert_eq!(made.members.len(), 1);
    assert_eq!(made.episode.as_ref().unwrap().anilist_id, Some(1));
    assert_eq!(made.episode.as_ref().unwrap().mal_id, Some(21));
    assert_eq!(made.episode.as_ref().unwrap().episode_key, "1");
    assert_eq!(made.episode.as_ref().unwrap().series_title, "Cowboy Bebop");
    assert_eq!(made.episode.as_ref().unwrap().opening_start, Some(90.0));
    assert_eq!(made.episode.as_ref().unwrap().duration, Some(1420.0));
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
    let made = room(ann.core.call(Call::RoomCreate {
        session: Some(ann_1),
    }))
    .unwrap();

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
    assert_eq!(moved.episode.as_ref().unwrap().episode_key, "2");
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
    assert_eq!(ben_room.episode.as_ref().unwrap().episode_key, "2");
    assert_eq!(ben_room.session, None);
}

#[test]
fn a_member_s_player_on_the_old_episode_does_not_take_the_new_one() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0, 2.0], true);
    // Dee has episode 1 only.
    let dee = person(&relay, "Dee", &[1.0], true);
    let ann_1 = open(&ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate {
        session: Some(ann_1),
    }))
    .unwrap();
    room(dee.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    let dee_1 = open(&dee, dee.files[0]);
    room(dee.core.call(Call::RoomAttach { session: dee_1 }));

    // Ann moves on to episode 2, which Dee lacks.
    let ann_2 = open(&ann, ann.files[1]);
    room(ann.core.call(Call::RoomAttach { session: ann_2 })).unwrap();
    common::wait_for(
        &dee.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { file, episode, .. }
                if file.is_none() && episode.episode_key == "2")
        },
        WAIT,
    );
    // Dee's player, still on episode 1, asks to attach again, as the shell can while
    // the move reaches it: the room stays on episode 2 without her.
    let after = room(dee.core.call(Call::RoomAttach { session: dee_1 })).unwrap();
    assert_eq!(after.session, None);
    assert!(after.can_fetch, "Get media stays on offer");
    common::wait_for(
        &ann.c,
        |e| {
            matches!(&e.body, EventBody::RoomChanged { room: Some(r) }
                if r.members.iter().any(|m| m.name == "Dee" && m.ready == RoomReady::Missing))
        },
        WAIT,
    );
}

#[test]
fn a_dropped_connection_comes_back_and_leaving_is_for_good() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0], true);
    let ben = person(&relay, "Ben", &[1.0], true);

    let ann_s = open(&ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate {
        session: Some(ann_s),
    }))
    .unwrap();
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

/// Ann hosts episode 1 and Ben joins and opens his copy: both players are
/// attached, paused at the start.
fn two_attached(relay: &Relay) -> (Person, u64, Person, u64, RoomInfo) {
    let ann = person(relay, "Ann", &[1.0, 2.0], true);
    let ben = person(relay, "Ben", &[1.0, 2.0], true);
    let ann_s = open(&ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate {
        session: Some(ann_s),
    }))
    .unwrap();
    room(ben.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    let ben_s = open(&ben, ben.files[0]);
    room(ben.core.call(Call::RoomAttach { session: ben_s }));
    applied(&ben.c, ben_s, |playing, _, _| !playing);
    // Ann sees Ben ready, so her first play is not held for him (#922)
    seen_ready(&ann, "Ben");
    (ann, ann_s, ben, ben_s, made)
}

/// Waits until `p` sees `name` with the episode open.
fn seen_ready(p: &Person, name: &str) {
    common::wait_for(
        &p.c,
        |e| {
            matches!(&e.body, EventBody::RoomChanged { room: Some(r) }
                if r.members.iter().any(|m| m.name == name && m.ready == RoomReady::Ready))
        },
        WAIT,
    );
}

/// Waits until `p`'s room shows the gate in `gate`.
fn gate_is(p: &Person, gate: RoomGate) -> RoomInfo {
    let e = common::wait_for(
        &p.c,
        |e| matches!(&e.body, EventBody::RoomChanged { room: Some(r) } if r.gate == gate),
        WAIT,
    );
    match e.body {
        EventBody::RoomChanged { room: Some(r) } => r,
        _ => unreachable!(),
    }
}

fn applies(c: &Collector, session: u64) -> Vec<(bool, Option<f64>)> {
    c.events()
        .iter()
        .filter_map(|e| match &e.body {
            EventBody::RoomApply {
                session: s,
                playing,
                seek_to,
                ..
            } if *s == session => Some((*playing, *seek_to)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_tick_puts_right_a_player_that_disagrees_on_playing() {
    let relay = relay();
    let (ann, _, ben, ben_s, _) = two_attached(&relay);

    // The room is paused at 40, and Ben's player keeps playing.
    ann.core
        .call(Call::RoomCommand {
            action: RoomAction::Pause,
            position: 40.0,
        })
        .unwrap();
    applied(&ben.c, ben_s, |playing, seek, _| {
        !playing && seek == Some(40.0)
    });
    ben.core
        .call(Call::Tick {
            session: ben_s,
            position: 41.0,
            paused: false,
        })
        .unwrap();
    wait_count(&ben.c, 2, |e| {
        matches!(&e.body, EventBody::RoomApply { session, playing: false, seek_to: Some(t), .. }
            if *session == ben_s && *t == 40.0)
    });

    // The room plays, and Ben's player sits paused.
    ann.core
        .call(Call::RoomCommand {
            action: RoomAction::Play,
            position: 50.0,
        })
        .unwrap();
    applied(&ben.c, ben_s, |playing, _, _| playing);
    let before = applies(&ben.c, ben_s).len();
    ben.core
        .call(Call::Tick {
            session: ben_s,
            position: 50.0,
            paused: true,
        })
        .unwrap();
    wait_count(
        &ben.c,
        before + 1,
        |e| matches!(&e.body, EventBody::RoomApply { session, .. } if *session == ben_s),
    );
    assert!(applies(&ben.c, ben_s).last().unwrap().0, "told to play");
}

#[test]
fn a_held_arrow_key_reaches_the_room_with_its_last_seek() {
    let relay = relay();
    let (ann, ann_s, ben, ben_s, _) = two_attached(&relay);
    let before = applies(&ben.c, ben_s).len();

    // Forty seeks as fast as a held key can make them: well past the relay's
    // twenty messages a second.
    for i in 0..40 {
        ann.core
            .call(Call::RoomCommand {
                action: RoomAction::Seek,
                position: 100.0 + f64::from(i) * 5.0,
            })
            .unwrap();
    }
    // Ben gets the last one, not whichever came before the relay's limit.
    applied(&ben.c, ben_s, |_, seek, _| seek == Some(295.0));
    let seeks = applies(&ben.c, ben_s).len() - before;
    assert!(seeks <= 5, "{seeks} seeks reached Ben");
    // Ann's own player is never pulled back to one of her earlier seeks.
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        applies(&ann.c, ann_s)
            .iter()
            .all(|(_, seek)| seek.is_none_or(|t| t == 0.0)),
        "{:?}",
        applies(&ann.c, ann_s)
    );
}

#[test]
fn the_hosts_move_while_the_connection_is_down_is_made_again() {
    let relay = relay();
    let (ann, _, ben, ben_1, _) = two_attached(&relay);

    // Ann's connection drops, and before it is back her player moves to
    // episode 2.
    ann.core.room_drop_connection();
    common::wait_for(&ann.c, |e| said(e, "core-room-lost"), WAIT);
    let ann_2 = open(&ann, ann.files[1]);
    let moved = room(ann.core.call(Call::RoomAttach { session: ann_2 })).unwrap();
    assert_eq!(moved.episode.as_ref().unwrap().episode_key, "2");
    common::wait_for(&ann.c, |e| said(e, "core-room-back"), WAIT);

    // Back, she stays on episode 2 and the room follows her there.
    applied(&ann.c, ann_2, |playing, seek, _| {
        !playing && seek == Some(0.0)
    });
    applied(&ben.c, ben_1, |playing, seek, _| !playing && seek.is_none());
    common::wait_for(
        &ben.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { file, episode, .. }
                if *file == Some(ben.files[1]) && episode.episode_key == "2")
        },
        WAIT,
    );
    let ann_room = status(&ann).unwrap();
    assert_eq!(ann_room.episode.as_ref().unwrap().episode_key, "2");
    assert_eq!(ann_room.session, Some(ann_2));
    assert!(
        !ann.c
            .events()
            .iter()
            .any(|e| matches!(&e.body, EventBody::RoomEpisode { .. })),
        "Ann is never sent back to episode 1"
    );
}

#[test]
fn a_new_host_keeps_the_offset_stored_for_their_file() {
    let relay = relay();
    let (ann, ann_s, ben, ben_s, _) = two_attached(&relay);

    // Ben's copy runs 3 seconds later than Ann's, set by hand and kept.
    let set = room(ben.core.call(Call::RoomSetOffset { offset: 3.0 })).unwrap();
    assert_eq!(set.offset, 3.0);

    // Ann, the host, may set one too; her file is the room's timeline, so it
    // starts at nought and is not stored for later rooms.
    let ann_set = room(ann.core.call(Call::RoomSetOffset { offset: 1.0 })).unwrap();
    assert_eq!(ann_set.offset, 1.0);
    ann.core
        .call(Call::ClosePlayback {
            session: ann_s,
            position: 5.0,
            reason: CloseReason::Stopped,
        })
        .unwrap();
    let ann_again = open(&ann, ann.files[0]);
    let back = room(ann.core.call(Call::RoomAttach { session: ann_again })).unwrap();
    assert_eq!(back.offset, 1.0, "the timeline's offset lasts the room");

    // Ann leaves: Ben is the host now, on Ann's timeline still.
    room(ann.core.call(Call::RoomLeave));
    common::wait_for(
        &ben.c,
        |e| matches!(&e.body, EventBody::RoomChanged { room: Some(r) } if r.host),
        WAIT,
    );
    ben.core
        .call(Call::ClosePlayback {
            session: ben_s,
            position: 5.0,
            reason: CloseReason::Stopped,
        })
        .unwrap();
    let ben_again = open(&ben, ben.files[0]);
    let kept = room(ben.core.call(Call::RoomAttach { session: ben_again })).unwrap();
    assert!(kept.host);
    assert_eq!(
        kept.episode.as_ref().unwrap().episode_key,
        "1",
        "the room stays put"
    );
    assert_eq!(kept.offset, 3.0, "the stored offset is kept");
}

// A room that starts before anything is picked, and the ready gate (#922)

#[test]
fn a_room_starts_with_nothing_picked_and_the_hosts_picks_move_everyone() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0, 2.0], true);
    let ben = person(&relay, "Ben", &[2.0, 1.0], true);

    // Ann starts a room with nothing playing.
    let made = room(ann.core.call(Call::RoomCreate { session: None })).unwrap();
    assert!(made.host);
    assert!(made.connected);
    assert_eq!(made.episode, None);
    assert_eq!(made.session, None);
    assert_eq!(made.gate, RoomGate::Off);
    common::wait_for(&ann.c, |e| said(e, "core-room-created"), WAIT);

    // Ben joins: he sees the room and who is in it, and nothing to open yet.
    let joined = room(ben.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }))
    .unwrap();
    assert_eq!(joined.episode, None);
    assert_eq!(joined.members.len(), 2);
    assert!(joined.members.iter().any(|m| m.name == "Ann" && m.host));
    common::wait_for(
        &ann.c,
        |e| said_name(e, "core-room-member-joined", "Ben"),
        WAIT,
    );
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !ben.c
            .events()
            .iter()
            .any(|e| matches!(&e.body, EventBody::RoomEpisode { .. })),
        "nothing to open before Ann picks"
    );
    // Only the host picks: Ben's player on his own does not move the room.
    let ben_own = open(&ben, ben.files[0]);
    let err = ben
        .core
        .call(Call::RoomAttach { session: ben_own })
        .unwrap_err();
    assert_eq!(service_code(err), "not_in_room");
    ben.core
        .call(Call::ClosePlayback {
            session: ben_own,
            position: 0.0,
            reason: CloseReason::Stopped,
        })
        .unwrap();

    // Ann opens episode 1: the room takes it, and Ben is told which file it is.
    let ann_1 = open(&ann, ann.files[0]);
    let picked = room(ann.core.call(Call::RoomAttach { session: ann_1 })).unwrap();
    assert_eq!(picked.episode.as_ref().unwrap().episode_key, "1");
    assert_eq!(picked.session, Some(ann_1));
    applied(&ann.c, ann_1, |playing, seek, _| {
        !playing && seek == Some(0.0)
    });
    common::wait_for(
        &ben.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { file, episode, .. }
                if *file == Some(ben.files[1]) && episode.episode_key == "1")
        },
        WAIT,
    );
    let ben_1 = open(&ben, ben.files[1]);
    room(ben.core.call(Call::RoomAttach { session: ben_1 }));
    applied(&ben.c, ben_1, |playing, seek, _| {
        !playing && seek == Some(0.0)
    });

    // Ann changes her mind before anyone plays: episode 2 it is, and Ben follows
    // without joining again.
    let ann_2 = open(&ann, ann.files[1]);
    let changed = room(ann.core.call(Call::RoomAttach { session: ann_2 })).unwrap();
    assert_eq!(changed.episode.as_ref().unwrap().episode_key, "2");
    applied(&ben.c, ben_1, |playing, seek, _| !playing && seek.is_none());
    common::wait_for(
        &ben.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { file, episode, .. }
                if *file == Some(ben.files[0]) && episode.episode_key == "2")
        },
        WAIT,
    );
    let ben_room = status(&ben).unwrap();
    assert_eq!(ben_room.episode.as_ref().unwrap().episode_key, "2");
    assert_eq!(
        ben_room.you, joined.you,
        "the same member, never reconnected"
    );
}

#[test]
fn the_hosts_first_play_waits_for_everyone_when_asked_to() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0], true);
    let ben = person(&relay, "Ben", &[1.0], true);
    let cid = person(&relay, "Cid", &[1.0], true);

    let ann_s = open(&ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate {
        session: Some(ann_s),
    }))
    .unwrap();
    room(ben.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    let ben_s = open(&ben, ben.files[0]);
    room(ben.core.call(Call::RoomAttach { session: ben_s }));
    seen_ready(&ann, "Ben");
    // Cid joins and has the episode, but has not opened it yet.
    room(cid.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    common::wait_for(
        &ann.c,
        |e| said_name(e, "core-room-member-joined", "Cid"),
        WAIT,
    );

    // Ann presses play: the room holds, her player stays paused where she
    // pressed, and she is asked.
    let held = room(ann.core.call(Call::RoomCommand {
        action: RoomAction::Play,
        position: 12.0,
    }))
    .unwrap();
    assert_eq!(held.gate, RoomGate::Asking);
    applied(&ann.c, ann_s, |playing, seek, _| {
        !playing && seek == Some(12.0)
    });
    // She waits for everyone.
    let waiting = room(ann.core.call(Call::RoomStart { wait: true })).unwrap();
    assert_eq!(waiting.gate, RoomGate::Waiting);
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        applies(&ben.c, ben_s).iter().all(|(playing, _)| !playing),
        "nobody plays while the room waits"
    );

    // Cid opens his copy: everyone is ready, and the room starts on its own
    // from where Ann pressed play.
    let cid_s = open(&cid, cid.files[0]);
    room(cid.core.call(Call::RoomAttach { session: cid_s }));
    applied(&ann.c, ann_s, |playing, seek, _| {
        playing && seek.is_some_and(|t| (12.0..14.0).contains(&t))
    });
    applied(&ben.c, ben_s, |playing, seek, _| {
        playing && seek.is_some_and(|t| (12.0..14.0).contains(&t))
    });
    applied(&cid.c, cid_s, |playing, _, _| playing);
    gate_is(&ann, RoomGate::Off);

    // Once it has played, a pause and a play go straight through.
    ann.core
        .call(Call::RoomCommand {
            action: RoomAction::Pause,
            position: 30.0,
        })
        .unwrap();
    applied(&ben.c, ben_s, |playing, seek, _| {
        !playing && seek == Some(30.0)
    });
    let again = room(ann.core.call(Call::RoomCommand {
        action: RoomAction::Play,
        position: 30.0,
    }))
    .unwrap();
    assert_eq!(again.gate, RoomGate::Off);
    applied(&ben.c, ben_s, |playing, seek, _| {
        playing && seek.is_some_and(|t| (30.0..32.0).contains(&t))
    });
}

#[test]
fn the_host_starts_anyway_and_a_late_copy_joins_in_sync() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0, 2.0], true);
    let ben = person(&relay, "Ben", &[1.0, 2.0], true);
    // Cid lacks episode 1.
    let cid = person(&relay, "Cid", &[2.0], true);

    let ann_s = open(&ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate {
        session: Some(ann_s),
    }))
    .unwrap();
    room(ben.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    let ben_s = open(&ben, ben.files[0]);
    room(ben.core.call(Call::RoomAttach { session: ben_s }));
    seen_ready(&ann, "Ben");
    room(cid.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    common::wait_for(
        &ann.c,
        |e| {
            matches!(&e.body, EventBody::RoomChanged { room: Some(r) }
                if r.members.iter().any(|m| m.name == "Cid" && m.ready == RoomReady::Missing))
        },
        WAIT,
    );
    // Cid, without the episode, is never asked anything: only the host chooses.
    assert_eq!(status(&cid).unwrap().gate, RoomGate::Off);

    let held = room(ann.core.call(Call::RoomCommand {
        action: RoomAction::Play,
        position: 0.0,
    }))
    .unwrap();
    assert_eq!(held.gate, RoomGate::Asking);
    let ready = held
        .members
        .iter()
        .filter(|m| m.ready == RoomReady::Ready)
        .count();
    assert_eq!((ready, held.members.len()), (2, 3));

    // Start anyway: Ann and Ben play, Cid still lacks it.
    let started = room(ann.core.call(Call::RoomStart { wait: false })).unwrap();
    assert_eq!(started.gate, RoomGate::Off);
    applied(&ann.c, ann_s, |playing, _, _| playing);
    applied(&ben.c, ben_s, |playing, _, _| playing);
    assert!(status(&cid).unwrap().can_fetch);

    // The next episode moves everyone, Cid included, and its first play is
    // held again until the host chooses.
    let ann_2 = open(&ann, ann.files[1]);
    let moved = room(ann.core.call(Call::RoomAttach { session: ann_2 })).unwrap();
    assert_eq!(moved.episode.as_ref().unwrap().episode_key, "2");
    assert_eq!(moved.gate, RoomGate::Off);
    common::wait_for(
        &cid.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { file, episode, .. }
                if *file == Some(cid.files[0]) && episode.episode_key == "2")
        },
        WAIT,
    );
    common::wait_for(
        &ben.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { file, episode, .. }
                if *file == Some(ben.files[1]) && episode.episode_key == "2")
        },
        WAIT,
    );
    let cid_2 = open(&cid, cid.files[0]);
    room(cid.core.call(Call::RoomAttach { session: cid_2 }));
    seen_ready(&ann, "Cid");
    let held = room(ann.core.call(Call::RoomCommand {
        action: RoomAction::Play,
        position: 0.0,
    }))
    .unwrap();
    assert_eq!(held.gate, RoomGate::Asking, "Ben has yet to open episode 2");
    // Ben opens it late: everyone is ready, but the room only starts on its own
    // after the host chose to wait, so it still asks.
    let ben_2 = open(&ben, ben.files[1]);
    room(ben.core.call(Call::RoomAttach { session: ben_2 }));
    seen_ready(&ann, "Ben");
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(status(&ann).unwrap().gate, RoomGate::Asking);
    assert!(
        applies(&cid.c, cid_2).iter().all(|(playing, _)| !playing),
        "nobody plays until the host chooses"
    );
    // Waiting with everyone ready starts at once.
    let started = room(ann.core.call(Call::RoomStart { wait: true })).unwrap();
    assert_eq!(started.gate, RoomGate::Off);
    applied(&cid.c, cid_2, |playing, _, _| playing);
    applied(&ben.c, ben_2, |playing, _, _| playing);
}

// ---------------------------------------------------------------------------
// Joining through Discord (#966), against a stand-in Discord
// ---------------------------------------------------------------------------

use anibeam_core::presence::{self, Incoming, Party};

#[derive(Clone, Debug, PartialEq)]
enum Wire {
    /// An activity set, by the party it carried.
    Set(Option<Party>),
    Clear,
    Answer(String, bool),
}

#[derive(Clone, Default)]
struct Discord {
    said: Arc<std::sync::Mutex<Vec<Incoming>>>,
    wire: Arc<std::sync::Mutex<Vec<Wire>>>,
}

impl presence::Transport for Discord {
    fn connect(&mut self, _client_id: &str) -> Result<(), presence::SendError> {
        Ok(())
    }
    fn send(&mut self, activity: Option<&presence::Activity>) -> Result<(), presence::SendError> {
        self.wire.lock().unwrap().push(match activity {
            Some(a) => Wire::Set(a.party.clone()),
            None => Wire::Clear,
        });
        Ok(())
    }
    fn poll(&mut self) -> Result<Vec<Incoming>, presence::SendError> {
        Ok(std::mem::take(&mut *self.said.lock().unwrap()))
    }
    fn answer(&mut self, user_id: &str, invite: bool) -> Result<(), presence::SendError> {
        self.wire
            .lock()
            .unwrap()
            .push(Wire::Answer(user_id.to_string(), invite));
        Ok(())
    }
    fn close(&mut self) {}
}

impl Discord {
    fn ask(&self, user: &str) {
        self.said.lock().unwrap().push(Incoming::JoinRequest {
            user_id: user.into(),
            name: format!("Friend {user}"),
            avatar: None,
        });
    }
    fn wire(&self) -> Vec<Wire> {
        self.wire.lock().unwrap().clone()
    }
    fn wait(&self, what: &str, pred: impl Fn(&[Wire]) -> bool) -> Vec<Wire> {
        let deadline = std::time::Instant::now() + WAIT;
        loop {
            let w = self.wire();
            if pred(&w) {
                return w;
            }
            assert!(std::time::Instant::now() < deadline, "{what}: {w:#?}");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    fn last_party(&self) -> Option<Party> {
        self.wire().iter().rev().find_map(|w| match w {
            Wire::Set(p) => Some(p.clone()),
            _ => None,
        })?
    }
}

/// `p` with Discord presence on, speaking to a stand-in.
fn on_discord(p: &Person) -> Discord {
    let d = Discord::default();
    p.core.set_presence_transport(Box::new(d.clone()));
    let mut prefs = match p.core.call(Call::GetPreferences).unwrap() {
        Reply::Preferences { preferences } => preferences,
        other => panic!("{other:?}"),
    };
    prefs.discord_presence = true;
    p.core
        .call(Call::SetPreferences { preferences: prefs })
        .unwrap();
    d
}

fn asks(p: &Person) -> usize {
    p.c.events()
        .iter()
        .filter(|e| matches!(e.body, EventBody::DiscordJoinAsk { .. }))
        .count()
}

#[test]
fn a_yes_on_discord_starts_a_room_and_the_invite_hands_over_its_code() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0], true);
    let discord = on_discord(&ann);
    let s = open(&ann, ann.files[0]);
    ann.core
        .call(Call::Tick {
            session: s,
            position: 3.0,
            paused: false,
        })
        .unwrap();
    discord.wait("the episode shows", |w| {
        w.iter()
            .any(|x| matches!(x, Wire::Set(Some(p)) if p.secret == presence::ASK_SECRET))
    });

    discord.ask("777");
    wait_count(&ann.c, 1, |e| {
        matches!(e.body, EventBody::DiscordJoinAsk { .. })
    });
    let made = room(ann.core.call(Call::DiscordAnswer {
        user_id: "777".into(),
        allow: true,
    }))
    .expect("a room");
    assert!(made.host);

    // The room's secret is on Discord before the invite that hands it over.
    let wire = discord.wait("the invite", |w| {
        w.contains(&Wire::Answer("777".into(), true))
    });
    let invite = wire
        .iter()
        .position(|x| *x == Wire::Answer("777".into(), true))
        .unwrap();
    let secret = format!("{}{}", presence::ROOM_SECRET, made.code);
    assert!(
        wire[..invite]
            .iter()
            .any(|x| matches!(x, Wire::Set(Some(p)) if p.secret == secret && !p.open)),
        "{wire:#?}"
    );

    // Someone let in once is let in again with no question.
    discord.ask("777");
    discord.wait("the second invite", |w| {
        w.iter()
            .filter(|x| **x == Wire::Answer("777".into(), true))
            .count()
            == 2
    });
    assert_eq!(asks(&ann), 1);

    // An open room lets anyone in with no ask at all.
    ann.core
        .call(Call::DiscordSetRoomOpen { open: true })
        .unwrap();
    discord.wait("the open party", |_| {
        discord.last_party().is_some_and(|p| p.open)
    });
    discord.ask("888");
    discord.wait("the stranger's invite", |w| {
        w.contains(&Wire::Answer("888".into(), true))
    });
    assert_eq!(asks(&ann), 1);
}

#[test]
fn the_room_shows_on_discord_with_everyone_in_it_while_nothing_plays() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0], true);
    let ben = person(&relay, "Ben", &[1.0], true);
    let discord = on_discord(&ann);
    let made = room(ann.core.call(Call::RoomCreate { session: None })).expect("a room");
    discord.wait("the room", |_| {
        discord
            .last_party()
            .is_some_and(|p| p.secret.ends_with(&made.code) && p.size == 1)
    });
    room(ben.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }))
    .expect("joined");
    discord.wait("two in the room", |_| {
        discord.last_party().is_some_and(|p| p.size == 2)
    });
    // Leaving takes the room off Discord.
    ann.core.call(Call::RoomLeave).unwrap();
    discord.wait("the room gone", |w| {
        w.iter().rev().find(|x| !matches!(x, Wire::Answer(..))) == Some(&Wire::Clear)
    });
}

#[test]
fn switching_discord_off_in_a_room_takes_the_room_off_and_keeps_it_off() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0], true);
    let discord = on_discord(&ann);
    let made = room(ann.core.call(Call::RoomCreate { session: None })).expect("a room");
    discord.wait("the room", |_| {
        discord
            .last_party()
            .is_some_and(|p| p.secret.ends_with(&made.code))
    });
    let mut prefs = match ann.core.call(Call::GetPreferences).unwrap() {
        Reply::Preferences { preferences } => preferences,
        other => panic!("{other:?}"),
    };
    prefs.discord_presence = false;
    ann.core
        .call(Call::SetPreferences { preferences: prefs })
        .unwrap();
    ann.core.presence_flush();
    let off = discord.wire().len();
    std::thread::sleep(Duration::from_millis(1500));
    let after = discord.wire();
    assert_eq!(after.last(), Some(&Wire::Clear), "{after:#?}");
    assert_eq!(after.len(), off, "nothing shows again: {after:#?}");
}

#[test]
fn a_new_host_on_an_episode_that_has_played_is_not_asked_again() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[1.0], true);
    let cid = person(&relay, "Cid", &[1.0], true);
    let ben = person(&relay, "Ben", &[1.0], true);
    // Dee lacks episode 1.
    let dee = person(&relay, "Dee", &[2.0], true);

    // Ann, alone, plays the episode and pauses it at 30 s.
    let ann_s = open(&ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate {
        session: Some(ann_s),
    }))
    .unwrap();
    let played = room(ann.core.call(Call::RoomCommand {
        action: RoomAction::Play,
        position: 0.0,
    }))
    .unwrap();
    assert_eq!(played.gate, RoomGate::Off);
    ann.core
        .call(Call::RoomCommand {
            action: RoomAction::Pause,
            position: 30.0,
        })
        .unwrap();
    // The relay has the pause before anyone else comes in
    std::thread::sleep(Duration::from_millis(300));

    // Cid joins the paused room first and sees no play himself; then Ben and Dee.
    room(cid.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    let cid_s = open(&cid, cid.files[0]);
    room(cid.core.call(Call::RoomAttach { session: cid_s }));
    applied(&cid.c, cid_s, |playing, seek, _| {
        !playing && seek.is_some_and(|t| (t - 30.0).abs() < 0.01)
    });
    seen_ready(&ann, "Cid");
    room(ben.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    let ben_s = open(&ben, ben.files[0]);
    room(ben.core.call(Call::RoomAttach { session: ben_s }));
    seen_ready(&cid, "Ben");
    room(dee.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }));
    common::wait_for(
        &cid.c,
        |e| {
            matches!(&e.body, EventBody::RoomChanged { room: Some(r) }
                if r.members.iter().any(|m| m.name == "Dee" && m.ready == RoomReady::Missing))
        },
        WAIT,
    );

    // Ann leaves: Cid, there longest, is the host. Dee still lacks the episode,
    // but it has played, so Cid's play goes straight through.
    room(ann.core.call(Call::RoomLeave));
    common::wait_for(
        &cid.c,
        |e| matches!(&e.body, EventBody::RoomChanged { room: Some(r) } if r.host),
        WAIT,
    );
    let again = room(cid.core.call(Call::RoomCommand {
        action: RoomAction::Play,
        position: 30.0,
    }))
    .unwrap();
    assert_eq!(again.gate, RoomGate::Off);
    applied(&ben.c, ben_s, |playing, seek, _| {
        playing && seek.is_some_and(|t| (30.0..32.0).contains(&t))
    });
}
