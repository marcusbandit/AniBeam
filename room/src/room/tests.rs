use super::*;

const KEY: &str = "0123456789abcdef0123456789abcdef";

fn ep(key: &str) -> EpisodeRef {
    EpisodeRef {
        anilist_id: Some(1),
        mal_id: None,
        tvmaze_id: None,
        episode_key: key.into(),
        number: Some(1.0),
        series_title: "Cowboy Bebop".into(),
        episode_title: None,
        duration: None,
        opening_start: None,
    }
}

fn room() -> Room {
    Room::new("K7QM2X", KEY, ep("e1"), 0, [9; 16])
}

fn hello(name: &str, host_key: Option<&str>, rejoin_key: Option<&str>) -> ClientMsg {
    ClientMsg::Hello {
        name: name.into(),
        proto: PROTO,
        client: "anibeam/2.5.0".into(),
        host_key: host_key.map(Into::into),
        rejoin_key: rejoin_key.map(Into::into),
    }
}

/// Says hello on `conn` and returns the member id and rejoin key from the welcome.
fn join(
    room: &mut Room,
    conn: u64,
    name: &str,
    host_key: Option<&str>,
    now: u64,
) -> (MemberId, String) {
    let outs = room.handle(ConnId(conn), hello(name, host_key, None), now);
    welcome(&outs, conn)
}

fn welcome(outs: &[Out], conn: u64) -> (MemberId, String) {
    for out in outs {
        if out.to == To::One(ConnId(conn))
            && let ServerMsg::Welcome {
                you, rejoin_key, ..
            } = &out.msg
        {
            return (*you, rejoin_key.clone());
        }
    }
    panic!("no welcome for {conn} in {outs:?}");
}

fn cmd(action: Action, position: f64) -> ClientMsg {
    ClientMsg::Cmd { action, position }
}

fn error_code(outs: &[Out]) -> Option<ErrorCode> {
    outs.iter().find_map(|o| match &o.msg {
        ServerMsg::Error { code, .. } => Some(*code),
        _ => None,
    })
}

fn member_ids(room: &Room) -> Vec<u32> {
    room.members().iter().map(|m| m.id.0).collect()
}

// Hello and names

#[test]
fn a_message_before_hello_is_a_fatal_proto_error() {
    let mut r = room();
    let outs = r.handle(ConnId(1), cmd(Action::Play, 0.0), 10);
    assert_eq!(outs.len(), 1);
    assert_eq!(outs[0].to, To::One(ConnId(1)));
    assert_eq!(error_code(&outs), Some(ErrorCode::Proto));
    assert!(outs[0].msg.ends_connection());
    assert!(r.members().is_empty());
}

#[test]
fn bad_json_before_hello_keeps_the_connection() {
    let mut r = room();
    let outs = r.handle_text(ConnId(1), "{nope", 10);
    assert_eq!(error_code(&outs), Some(ErrorCode::BadMessage));
    assert!(!outs[0].msg.ends_connection());
    let outs = r.handle_text(ConnId(1), &hello("Ann", None, None).to_json(), 20);
    welcome(&outs, 1);
}

#[test]
fn an_unknown_type_is_a_bad_message() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    let outs = r.handle_text(ConnId(1), r#"{"t":"dance","speed":2}"#, 10);
    assert_eq!(outs.len(), 1);
    assert_eq!(error_code(&outs), Some(ErrorCode::BadMessage));
    assert!(!outs[0].msg.ends_connection());
    assert_eq!(member_ids(&r), vec![1]);
}

#[test]
fn another_protocol_version_is_refused() {
    let mut r = room();
    let msg = ClientMsg::Hello {
        name: "Ann".into(),
        proto: 2,
        client: "anibeam/9".into(),
        host_key: None,
        rejoin_key: None,
    };
    let outs = r.handle(ConnId(1), msg, 0);
    assert_eq!(error_code(&outs), Some(ErrorCode::Proto));
    assert!(outs[0].msg.ends_connection());
    assert!(r.members().is_empty());
}

#[test]
fn names_are_trimmed() {
    let mut r = room();
    join(&mut r, 1, "  Ann \t", None, 0);
    assert_eq!(r.members()[0].name, "Ann");
}

#[test]
fn bad_names_are_refused_and_the_hello_can_be_retried() {
    let mut r = room();
    let long = "a".repeat(NAME_MAX + 1);
    for name in ["", "   ", long.as_str(), "A\nB", "A\u{7}", "\u{0}"] {
        let outs = r.handle(ConnId(1), hello(name, None, None), 0);
        assert_eq!(error_code(&outs), Some(ErrorCode::BadName), "{name:?}");
        assert!(!outs[0].msg.ends_connection());
    }
    assert!(r.members().is_empty());
    let longest = "\u{3042}".repeat(NAME_MAX);
    join(&mut r, 1, &longest, None, 0);
    assert_eq!(r.members()[0].name, longest);
}

#[test]
fn a_second_hello_is_a_bad_message() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    let outs = r.handle(ConnId(1), hello("Bea", None, None), 5);
    assert_eq!(error_code(&outs), Some(ErrorCode::BadMessage));
    assert_eq!(r.members()[0].name, "Ann");
}

#[test]
fn welcome_carries_the_room_and_the_others_hear_of_the_joiner() {
    let mut r = room();
    let (ann, ann_key) = join(&mut r, 1, "Ann", Some(KEY), 100);
    assert_eq!(ann, MemberId(1));
    assert_eq!(ann_key.len(), 32);
    assert!(ann_key.bytes().all(|b| b.is_ascii_hexdigit()));

    let outs = r.handle(ConnId(2), hello("Bea", None, None), 200);
    assert_eq!(outs.len(), 2);
    match &outs[0] {
        Out {
            to: To::One(ConnId(2)),
            msg:
                ServerMsg::Welcome {
                    you,
                    rejoin_key,
                    room,
                    server_ms,
                },
        } => {
            assert_eq!(*you, MemberId(2));
            assert_ne!(rejoin_key, &ann_key);
            assert_eq!(*server_ms, 200);
            assert_eq!(room.code, "K7QM2X");
            assert_eq!(room.host, Some(ann));
            assert_eq!(room.max, 10);
            assert_eq!(room.episode, ep("e1"));
            assert_eq!(room.members.len(), 2);
            assert!(room.members[0].host);
            assert!(!room.members[1].host);
            assert_eq!(room.members[1].joined_ms, 200);
            assert_eq!(room.members[1].ready, Ready::Loading);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(outs[1].to, To::AllBut(ConnId(2)));
    assert!(matches!(&outs[1].msg, ServerMsg::Members { members } if members.len() == 2));
    assert_eq!(r.targets(&outs[1].to), vec![ConnId(1)]);
}

#[test]
fn the_host_key_makes_the_host_and_a_wrong_one_does_not() {
    let mut r = room();
    let (bea, _) = join(
        &mut r,
        2,
        "Bea",
        Some("ffffffffffffffffffffffffffffffff"),
        0,
    );
    assert_eq!(r.host(), None);
    let (ann, _) = join(&mut r, 1, "Ann", Some(KEY), 10);
    assert_eq!(r.host(), Some(ann));
    assert_ne!(bea, ann);
    let members = r.members();
    assert!(!members[0].host);
    assert!(members[1].host);
}

#[test]
fn rejoin_keys_differ_between_members_and_between_rooms() {
    let mut a = room();
    let mut b = Room::new("K7QM2X", KEY, ep("e1"), 0, [10; 16]);
    let (_, a1) = join(&mut a, 1, "Ann", None, 0);
    let (_, a2) = join(&mut a, 2, "Bea", None, 0);
    let (_, b1) = join(&mut b, 1, "Ann", None, 0);
    assert_ne!(a1, a2);
    assert_ne!(a1, b1);
}

// Capacity

#[test]
fn the_eleventh_member_gets_full() {
    let mut r = room();
    for conn in 1..=10 {
        join(&mut r, conn, &format!("P{conn}"), None, conn);
    }
    let outs = r.handle(ConnId(11), hello("Late", None, None), 20);
    assert_eq!(outs.len(), 1);
    assert_eq!(error_code(&outs), Some(ErrorCode::Full));
    assert!(outs[0].msg.ends_connection());
    assert_eq!(r.members().len(), 10);
}

#[test]
fn a_member_in_grace_holds_their_place_and_a_bye_frees_it() {
    let mut r = room();
    for conn in 1..=10 {
        join(&mut r, conn, &format!("P{conn}"), None, 0);
    }
    r.disconnect(ConnId(10), 5);
    let outs = r.handle(ConnId(11), hello("Late", None, None), 10);
    assert_eq!(error_code(&outs), Some(ErrorCode::Full));
    r.handle(ConnId(9), ClientMsg::Bye, 20);
    join(&mut r, 11, "Late", None, 30);
    // Once the grace has run out, the place is free too.
    r.tick(5 + GRACE_MS);
    join(&mut r, 12, "Later", None, 5 + GRACE_MS);
    assert_eq!(r.members().len(), 10);
}

// Commands

#[test]
fn play_pause_and_seek_set_the_state_and_reach_everyone() {
    let mut r = room();
    let (ann, _) = join(&mut r, 1, "Ann", Some(KEY), 0);
    let (bea, _) = join(&mut r, 2, "Bea", None, 0);

    let outs = r.handle(ConnId(2), cmd(Action::Play, 12.5), 1_000);
    assert_eq!(outs.len(), 1);
    assert_eq!(outs[0].to, To::All);
    assert_eq!(r.targets(&To::All), vec![ConnId(1), ConnId(2)]);
    let want = PlayState {
        playing: true,
        position: 12.5,
        rate: 1.0,
        at_ms: 1_000,
        rev: 1,
    };
    assert_eq!(
        outs[0].msg,
        ServerMsg::State {
            state: want,
            by: bea
        }
    );

    let outs = r.handle(ConnId(1), cmd(Action::Seek, 300.0), 2_000);
    let ServerMsg::State { state, by } = &outs[0].msg else {
        panic!("{outs:?}")
    };
    assert_eq!(*by, ann);
    assert!(state.playing, "seek keeps playing");
    assert_eq!(state.position, 300.0);
    assert_eq!(state.rev, 2);
    assert_eq!(state.at_ms, 2_000);

    r.handle(ConnId(1), cmd(Action::Pause, 301.0), 3_000);
    r.handle(ConnId(1), cmd(Action::Seek, 10.0), 4_000);
    let s = r.state();
    assert!(!s.playing, "seek keeps paused");
    assert_eq!(s.position, 10.0);
    assert_eq!(s.rev, 4);
    assert_eq!(s.at_ms, 4_000);
}

#[test]
fn positions_must_be_finite_and_within_a_day() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    for bad in [-0.5, 86_400.01, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let outs = r.handle(ConnId(1), cmd(Action::Seek, bad), 10);
        assert_eq!(error_code(&outs), Some(ErrorCode::BadMessage), "{bad}");
    }
    assert_eq!(r.state().rev, 0);
    r.handle(ConnId(1), cmd(Action::Seek, 0.0), 20);
    r.handle(ConnId(1), cmd(Action::Seek, 86_400.0), 30);
    assert_eq!(r.state().rev, 2);
    assert_eq!(r.state().position, 86_400.0);
}

// Episode

#[test]
fn only_the_host_changes_the_episode() {
    let mut r = room();
    let (ann, _) = join(&mut r, 1, "Ann", Some(KEY), 0);
    join(&mut r, 2, "Bea", None, 0);
    r.handle(ConnId(1), cmd(Action::Play, 500.0), 10);
    r.handle(
        ConnId(2),
        ClientMsg::Status {
            ready: Ready::Ready,
        },
        11,
    );

    let outs = r.handle(ConnId(2), ClientMsg::Episode { episode: ep("e2") }, 20);
    assert_eq!(outs.len(), 1);
    assert_eq!(outs[0].to, To::One(ConnId(2)));
    assert_eq!(error_code(&outs), Some(ErrorCode::NotAllowed));
    assert_eq!(r.episode(), &ep("e1"));

    let outs = r.handle(ConnId(1), ClientMsg::Episode { episode: ep("e2") }, 30);
    assert_eq!(r.episode(), &ep("e2"));
    let want = PlayState {
        playing: false,
        position: 0.0,
        rate: 1.0,
        at_ms: 30,
        rev: 2,
    };
    assert_eq!(r.state(), want);
    assert_eq!(
        outs[0],
        Out {
            to: To::All,
            msg: ServerMsg::Episode {
                episode: ep("e2"),
                state: want,
                by: ann
            }
        }
    );
    assert!(matches!(&outs[1].msg, ServerMsg::Members { .. }));
    assert!(r.members().iter().all(|m| m.ready == Ready::Loading));
}

// Status and ping

#[test]
fn status_updates_the_member_list_for_everyone() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    join(&mut r, 2, "Bea", None, 0);
    let outs = r.handle(
        ConnId(2),
        ClientMsg::Status {
            ready: Ready::Missing,
        },
        10,
    );
    assert_eq!(outs.len(), 1);
    assert_eq!(outs[0].to, To::All);
    let ServerMsg::Members { members } = &outs[0].msg else {
        panic!("{outs:?}")
    };
    assert_eq!(members[1].ready, Ready::Missing);
    assert_eq!(members[0].ready, Ready::Loading);
}

#[test]
fn ping_gets_a_pong_with_the_relay_clock() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    let outs = r.handle(ConnId(1), ClientMsg::Ping { c: 42 }, 777);
    assert_eq!(
        outs,
        vec![Out {
            to: To::One(ConnId(1)),
            msg: ServerMsg::Pong { c: 42, s: 777 }
        }]
    );
}

// Leaving and rejoining

#[test]
fn a_dropped_socket_tells_the_others() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    join(&mut r, 2, "Bea", None, 0);
    let outs = r.disconnect(ConnId(2), 10);
    assert_eq!(outs.len(), 1);
    assert_eq!(r.targets(&outs[0].to), vec![ConnId(1)]);
    assert!(matches!(&outs[0].msg, ServerMsg::Members { members } if members.len() == 1));
    assert!(
        r.disconnect(ConnId(2), 11).is_empty(),
        "a second close is a no-op"
    );
    assert!(r.disconnect(ConnId(99), 12).is_empty());
}

#[test]
fn a_rejoin_within_the_grace_gets_the_same_id_and_host_flag() {
    let mut r = room();
    let (ann, key) = join(&mut r, 1, "Ann", Some(KEY), 0);
    join(&mut r, 2, "Bea", None, 0);
    r.disconnect(ConnId(1), 1_000);
    assert_eq!(r.host(), None, "the view shows no host while they are away");
    r.tick(1_000 + GRACE_MS - 1);
    let outs = r.handle(
        ConnId(3),
        hello("Annie", None, Some(&key)),
        1_000 + GRACE_MS - 1,
    );
    let (id, again) = welcome(&outs, 3);
    assert_eq!(id, ann);
    assert_eq!(again, key);
    assert_eq!(r.host(), Some(ann));
    assert_eq!(r.members()[0].name, "Annie");
    assert_eq!(r.members()[0].joined_ms, 0);
    // No handover happens later either: the host is back.
    assert!(r.tick(1_000 + GRACE_MS + 5).is_empty());
    assert_eq!(r.host(), Some(ann));
}

#[test]
fn a_rejoin_after_the_grace_is_a_new_member() {
    let mut r = room();
    join(&mut r, 2, "Bea", None, 0);
    let (ann, key) = join(&mut r, 1, "Ann", None, 0);
    r.disconnect(ConnId(1), 1_000);
    r.tick(1_000 + GRACE_MS);
    let outs = r.handle(ConnId(3), hello("Ann", None, Some(&key)), 1_000 + GRACE_MS);
    let (id, again) = welcome(&outs, 3);
    assert_ne!(id, ann);
    assert_ne!(again, key);
}

#[test]
fn a_late_rejoin_is_refused_even_before_the_tick() {
    let mut r = room();
    join(&mut r, 2, "Bea", None, 0);
    let (ann, key) = join(&mut r, 1, "Ann", None, 0);
    r.disconnect(ConnId(1), 1_000);
    let outs = r.handle(ConnId(3), hello("Ann", None, Some(&key)), 1_000 + GRACE_MS);
    let (id, _) = welcome(&outs, 3);
    assert_ne!(id, ann);
}

#[test]
fn a_rejoin_on_a_new_socket_closes_the_old_one() {
    let mut r = room();
    let (ann, key) = join(&mut r, 1, "Ann", None, 0);
    join(&mut r, 2, "Bea", None, 0);
    let outs = r.handle(ConnId(3), hello("Ann", None, Some(&key)), 50);
    assert_eq!(
        outs[0],
        Out {
            to: To::One(ConnId(1)),
            msg: ServerMsg::Closed {
                reason: CLOSED_REPLACED.into()
            }
        }
    );
    assert!(outs[0].msg.ends_connection());
    assert_eq!(welcome(&outs, 3).0, ann);
    assert!(
        r.disconnect(ConnId(1), 60).is_empty(),
        "the old socket is gone"
    );
    assert_eq!(member_ids(&r), vec![1, 2]);
    assert_eq!(r.targets(&To::All), vec![ConnId(3), ConnId(2)]);
}

#[test]
fn a_wrong_rejoin_key_is_a_new_member() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    r.disconnect(ConnId(1), 10);
    let outs = r.handle(ConnId(2), hello("Ann", None, Some("nope")), 20);
    assert_eq!(welcome(&outs, 2).0, MemberId(2));
}

#[test]
fn bye_leaves_for_good() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    let (_, key) = join(&mut r, 2, "Bea", None, 0);
    let outs = r.handle(ConnId(2), ClientMsg::Bye, 10);
    assert_eq!(
        outs[0],
        Out {
            to: To::One(ConnId(2)),
            msg: ServerMsg::Closed {
                reason: CLOSED_BYE.into()
            }
        }
    );
    assert!(outs[0].msg.ends_connection());
    assert_eq!(r.targets(&outs[1].to), vec![ConnId(1)]);
    assert!(matches!(&outs[1].msg, ServerMsg::Members { members } if members.len() == 1));
    let outs = r.handle(ConnId(3), hello("Bea", None, Some(&key)), 20);
    assert_eq!(welcome(&outs, 3).0, MemberId(3), "the rejoin key is spent");
}

// The host role

#[test]
fn a_host_who_says_bye_hands_over_at_once() {
    let mut r = room();
    join(&mut r, 1, "Ann", Some(KEY), 0);
    join(&mut r, 2, "Bea", None, 10);
    join(&mut r, 3, "Cid", None, 20);
    let outs = r.handle(ConnId(1), ClientMsg::Bye, 30);
    assert_eq!(r.host(), Some(MemberId(2)));
    let ServerMsg::Members { members } = &outs[1].msg else {
        panic!("{outs:?}")
    };
    assert!(members[0].host);
}

#[test]
fn a_dropped_host_hands_over_after_the_grace_to_the_longest_present() {
    let mut r = room();
    join(&mut r, 3, "Cid", None, 0);
    join(&mut r, 1, "Ann", Some(KEY), 10);
    join(&mut r, 2, "Bea", None, 20);
    // Cid joined first, but left and came back as a new member: Bea has been
    // present longest.
    r.handle(ConnId(3), ClientMsg::Bye, 30);
    join(&mut r, 4, "Cid", None, 40);

    r.disconnect(ConnId(1), 1_000);
    assert!(r.tick(1_000 + GRACE_MS - 1).is_empty());
    assert_eq!(r.host(), None);

    let outs = r.tick(1_000 + GRACE_MS);
    assert_eq!(r.host(), Some(MemberId(3)), "Bea is member 3");
    assert_eq!(outs.len(), 1);
    assert_eq!(outs[0].to, To::All);
    let ServerMsg::Members { members } = &outs[0].msg else {
        panic!("{outs:?}")
    };
    assert_eq!(members.iter().filter(|m| m.host).count(), 1);
    assert!(members.iter().find(|m| m.name == "Bea").unwrap().host);
    assert!(
        r.tick(1_000 + GRACE_MS + 1).is_empty(),
        "the handover happens once"
    );
}

#[test]
fn a_host_who_never_comes_is_replaced_after_the_grace() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 5);
    join(&mut r, 2, "Bea", None, 6);
    assert!(r.tick(GRACE_MS - 1).is_empty());
    assert_eq!(r.host(), None);
    r.tick(GRACE_MS);
    assert_eq!(r.host(), Some(MemberId(1)));
}

#[test]
fn a_joiner_into_a_room_overdue_for_a_host_becomes_it() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, GRACE_MS + 50);
    assert_eq!(r.host(), Some(MemberId(1)));
}

#[test]
fn the_host_key_takes_the_role_back() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    r.tick(GRACE_MS);
    assert_eq!(r.host(), Some(MemberId(1)));
    let (owner, _) = join(&mut r, 2, "Own", Some(KEY), GRACE_MS + 10);
    assert_eq!(r.host(), Some(owner));
    let hosts: Vec<_> = r.members().into_iter().filter(|m| m.host).collect();
    assert_eq!(hosts.len(), 1);
}

#[test]
fn nobody_connected_means_no_handover() {
    let mut r = room();
    join(&mut r, 1, "Ann", Some(KEY), 0);
    join(&mut r, 2, "Bea", None, 0);
    r.disconnect(ConnId(1), 10);
    r.disconnect(ConnId(2), 20);
    assert!(r.tick(10 + GRACE_MS).is_empty());
    assert_eq!(r.host(), None);
    // Bea's grace has run out too by the time Cid arrives; Cid gets the role.
    join(&mut r, 3, "Cid", None, 20 + GRACE_MS);
    assert_eq!(r.host(), Some(MemberId(3)));
}

// Expiry

#[test]
fn a_room_nobody_joins_closes_after_ten_minutes() {
    let mut r = room();
    r.tick(EMPTY_MS - 1);
    assert!(!r.is_closed());
    r.tick(EMPTY_MS);
    assert!(r.is_closed());
}

#[test]
fn the_empty_clock_starts_when_the_last_one_leaves() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    r.tick(EMPTY_MS * 2);
    assert!(!r.is_closed(), "someone is in it");
    r.disconnect(ConnId(1), EMPTY_MS * 2);
    r.tick(EMPTY_MS * 3 - 1);
    assert!(!r.is_closed());
    r.tick(EMPTY_MS * 3);
    assert!(r.is_closed());
}

#[test]
fn a_join_stops_the_empty_clock() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, EMPTY_MS - 10);
    r.tick(EMPTY_MS * 5);
    assert!(!r.is_closed());
}

#[test]
fn a_closed_room_refuses_everything() {
    let mut r = room();
    r.tick(EMPTY_MS);
    let outs = r.handle(ConnId(1), hello("Ann", None, None), EMPTY_MS + 1);
    assert_eq!(error_code(&outs), Some(ErrorCode::NotFound));
    assert!(outs[0].msg.ends_connection());
    assert!(r.tick(EMPTY_MS * 2).is_empty());
}

// Rate limit

#[test]
fn twenty_messages_a_second_then_rate_limited() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    for i in 0..RATE_PER_SEC {
        let outs = r.handle(ConnId(1), ClientMsg::Ping { c: u64::from(i) }, 1_000);
        assert!(matches!(outs[0].msg, ServerMsg::Pong { .. }), "message {i}");
    }
    let outs = r.handle(ConnId(1), cmd(Action::Play, 5.0), 1_500);
    assert_eq!(error_code(&outs), Some(ErrorCode::RateLimited));
    assert!(!outs[0].msg.ends_connection());
    assert_eq!(r.state().rev, 0, "the message is dropped");
    assert!(
        r.handle(ConnId(1), ClientMsg::Ping { c: 0 }, 1_999)
            .is_empty()
    );
    let outs = r.handle(ConnId(1), ClientMsg::Ping { c: 0 }, 2_000);
    assert!(
        matches!(outs[0].msg, ServerMsg::Pong { .. }),
        "a new second"
    );
}

#[test]
fn bad_messages_count_against_the_rate() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    for _ in 0..RATE_PER_SEC {
        let outs = r.handle_text(ConnId(1), "junk", 1_000);
        assert_eq!(error_code(&outs), Some(ErrorCode::BadMessage));
    }
    let outs = r.handle_text(ConnId(1), "junk", 1_000);
    assert_eq!(error_code(&outs), Some(ErrorCode::RateLimited));
    assert!(r.handle_text(ConnId(1), "junk", 1_000).is_empty());
}

#[test]
fn the_rate_is_per_member() {
    let mut r = room();
    join(&mut r, 1, "Ann", None, 0);
    join(&mut r, 2, "Bea", None, 0);
    for _ in 0..=RATE_PER_SEC {
        r.handle(ConnId(1), ClientMsg::Ping { c: 0 }, 1_000);
    }
    let outs = r.handle(ConnId(2), ClientMsg::Ping { c: 0 }, 1_000);
    assert!(matches!(outs[0].msg, ServerMsg::Pong { .. }));
}

// Storage and views

#[test]
fn a_room_survives_a_trip_through_json() {
    let mut r = room();
    let (_, key) = join(&mut r, 1, "Ann", Some(KEY), 0);
    join(&mut r, 2, "Bea", None, 0);
    r.handle(ConnId(1), cmd(Action::Play, 3.0), 10);
    r.disconnect(ConnId(1), 20);
    let json = serde_json::to_string(&r).unwrap();
    let mut back: Room = serde_json::from_str(&json).unwrap();
    assert_eq!(back, r);
    // The restored room keeps working, rejoin keys included.
    let outs = back.handle(ConnId(7), hello("Ann", None, Some(&key)), 30);
    assert_eq!(welcome(&outs, 7).0, MemberId(1));
    assert_eq!(back.host(), Some(MemberId(1)));
    assert_eq!(back.state().position, 3.0);
}

#[test]
fn the_view_shows_connected_members_only() {
    let mut r = room();
    join(&mut r, 1, "Ann", Some(KEY), 0);
    join(&mut r, 2, "Bea", None, 0);
    r.disconnect(ConnId(1), 5);
    let view = r.view();
    assert_eq!(view.host, None);
    assert_eq!(view.members.len(), 1);
    assert_eq!(view.members[0].name, "Bea");
    assert_eq!(r.targets(&To::All), vec![ConnId(2)]);
    assert_eq!(r.targets(&To::One(ConnId(99))), vec![ConnId(99)]);
    assert_eq!(r.targets(&To::AllBut(ConnId(2))), Vec::<ConnId>::new());
}

#[test]
fn a_new_room_is_paused_at_zero() {
    let r = Room::new("ABCDEF", KEY, ep("e1"), 1_234, [0; 16]);
    assert_eq!(r.code(), "ABCDEF");
    assert_eq!(
        r.state(),
        PlayState {
            playing: false,
            position: 0.0,
            rate: 1.0,
            at_ms: 1_234,
            rev: 0
        }
    );
    assert!(r.members().is_empty());
    assert!(!r.is_closed());
}
