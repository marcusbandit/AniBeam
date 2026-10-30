//! Discord rich presence through the core: what a playback session sends to
//! the transport, and that nothing is sent with the switch off.

mod common;
mod fixtures;

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use anibeam_core::presence::{self, Activity, SendError, Transport};
use anibeam_core::*;

#[derive(Clone, Debug, PartialEq)]
enum Sent {
    Connect,
    Set(Activity),
    Clear,
    Close,
}

/// Records what the worker asks of it. `fail` makes every connect fail,
/// `reject_posters` refuses any activity that carries a poster URL.
#[derive(Clone, Default)]
struct Fake {
    log: Arc<(Mutex<Vec<Sent>>, Condvar)>,
    fail: bool,
    reject_posters: bool,
}

impl Fake {
    fn push(&self, sent: Sent) {
        let (log, cv) = &*self.log;
        log.lock().unwrap().push(sent);
        cv.notify_all();
    }

    fn sent(&self) -> Vec<Sent> {
        self.log.0.lock().unwrap().clone()
    }

    /// Blocks until `pred` holds for what has been sent, on the condvar.
    fn wait(&self, pred: impl Fn(&[Sent]) -> bool) -> Vec<Sent> {
        let (log, cv) = &*self.log;
        let guard = log.lock().unwrap();
        let (guard, timeout) = cv
            .wait_timeout_while(guard, Duration::from_secs(10), |s| !pred(s))
            .unwrap();
        assert!(!timeout.timed_out(), "timed out; sent {:#?}", *guard);
        guard.clone()
    }
}

impl Transport for Fake {
    fn connect(&mut self, _client_id: &str) -> Result<(), SendError> {
        self.push(Sent::Connect);
        if self.fail {
            Err(SendError::Gone("no Discord".to_string()))
        } else {
            Ok(())
        }
    }

    fn send(&mut self, activity: Option<&Activity>) -> Result<(), SendError> {
        if self.reject_posters && activity.is_some_and(|a| a.poster.is_some()) {
            return Err(SendError::Rejected("large_image".to_string()));
        }
        self.push(match activity {
            Some(a) => Sent::Set(a.clone()),
            None => Sent::Clear,
        });
        Ok(())
    }

    fn close(&mut self) {
        self.push(Sent::Close);
    }
}

struct Library {
    ep1: u64,
    ep2: u64,
    extra: u64,
}

fn library(core: &Core) -> Library {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Bebop", "Bebop");
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
    let extra = fixtures::insert_file(core, series, "/lib/Bebop/OP1.mkv", 1.0, None, "extra", now);
    fixtures::insert_media_with_cover(core, 1, "https://img.example/bebop.jpg", None);
    fixtures::insert_episode(core, 1, 2, Some("Stray Dog Strut"), None);
    fixtures::match_series(core, series, Some(1), None);
    Library { ep1, ep2, extra }
}

fn switch(core: &Core, on: bool) {
    core.call(Call::SetPreferences {
        preferences: Preferences {
            discord_presence: on,
            ..Preferences::default()
        },
    })
    .unwrap();
}

fn open(core: &Core, file: u64) -> u64 {
    match core.call(Call::OpenPlayback { file }).unwrap() {
        Reply::Playback { session } => session.session,
        other => panic!("{other:?}"),
    }
}

fn tick(core: &Core, session: u64, position: f64, paused: bool) {
    core.call(Call::Tick {
        session,
        position,
        paused,
    })
    .unwrap();
}

fn close(core: &Core, session: u64) {
    core.call(Call::ClosePlayback {
        session,
        position: 10.0,
        reason: CloseReason::Stopped,
    })
    .unwrap();
}

fn presence_reply(core: &Core) -> (bool, bool) {
    match core.call(Call::GetPresence).unwrap() {
        Reply::Presence {
            available,
            connected,
        } => (available, connected),
        other => panic!("{other:?}"),
    }
}

fn sets(sent: &[Sent]) -> Vec<&Activity> {
    sent.iter()
        .filter_map(|s| match s {
            Sent::Set(a) => Some(a),
            _ => None,
        })
        .collect()
}

#[test]
fn a_session_sets_on_play_stops_the_clock_on_pause_and_clears_on_close() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    let fake = Fake::default();
    core.set_presence_transport(Box::new(fake.clone()));
    switch(&core, true);

    // Opening announces nothing: a file that never plays is not watched.
    let s = open(&core, lib.ep2);
    core.presence_flush();
    assert!(fake.sent().is_empty());

    tick(&core, s, 1.0, false);
    tick(&core, s, 2.0, false);
    tick(&core, s, 3.0, false);
    core.presence_flush();
    let sent = fake.sent();
    assert_eq!(sent[0], Sent::Connect);
    assert_eq!(sent.len(), 2, "unchanged ticks send nothing: {sent:#?}");
    let Sent::Set(a) = &sent[1] else {
        panic!("{sent:#?}")
    };
    assert_eq!(a.details, "Media 1");
    assert_eq!(a.state.as_deref(), Some("EP 2 - Stray Dog Strut"));
    assert_eq!(a.poster.as_deref(), Some("https://img.example/bebop.jpg"));
    assert_eq!(a.large_text, "Media 1");
    let start = a.start.expect("a playing activity counts up");
    assert!(start > 0);

    tick(&core, s, 4.0, true);
    tick(&core, s, 4.0, false);
    close(&core, s);
    core.presence_flush();
    let sent = fake.sent();
    let all = sets(&sent);
    assert_eq!(all.len(), 3);
    assert_eq!(all[1].start, None, "paused, so no clock");
    assert_eq!(all[2].start, Some(start), "the clock keeps its first start");
    assert_eq!(sent.last(), Some(&Sent::Clear));
    assert_eq!(presence_reply(&core), (true, true));
}

#[test]
fn a_long_pause_clears_and_playing_again_shows_it_again() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    let fake = Fake::default();
    core.set_presence_transport(Box::new(fake.clone()));
    core.set_presence_pause_clear(Duration::from_millis(20));
    switch(&core, true);

    let s = open(&core, lib.ep2);
    tick(&core, s, 1.0, false);
    tick(&core, s, 2.0, true);
    fake.wait(|sent| sent.contains(&Sent::Clear));
    tick(&core, s, 2.0, false);
    core.presence_flush();
    let sent = fake.sent();
    assert!(matches!(sent.last(), Some(Sent::Set(a)) if a.start.is_some()));
    // Closing a session that shows again clears it once more.
    close(&core, s);
    core.presence_flush();
    assert_eq!(fake.sent().last(), Some(&Sent::Clear));
}

#[test]
fn a_stale_close_leaves_the_next_episode_alone() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    let fake = Fake::default();
    core.set_presence_transport(Box::new(fake.clone()));
    switch(&core, true);

    let first = open(&core, lib.ep1);
    tick(&core, first, 1.0, false);
    let second = open(&core, lib.ep2);
    tick(&core, second, 1.0, false);
    close(&core, first);
    core.presence_flush();
    let sent = fake.sent();
    assert!(!sent.contains(&Sent::Clear), "{sent:#?}");
    assert_eq!(sets(&sent).len(), 2);
    assert_eq!(sets(&sent)[0].state.as_deref(), Some("EP 1"));
}

#[test]
fn an_extra_says_extra() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    let fake = Fake::default();
    core.set_presence_transport(Box::new(fake.clone()));
    switch(&core, true);
    let s = open(&core, lib.extra);
    tick(&core, s, 1.0, false);
    core.presence_flush();
    assert_eq!(sets(&fake.sent())[0].state.as_deref(), Some("Extra"));
}

#[test]
fn nothing_is_sent_with_the_switch_off() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    let fake = Fake::default();
    core.set_presence_transport(Box::new(fake.clone()));

    let s = open(&core, lib.ep2);
    tick(&core, s, 1.0, false);
    tick(&core, s, 2.0, true);
    close(&core, s);
    core.presence_flush();
    assert!(fake.sent().is_empty());
    assert_eq!(presence_reply(&core), (true, false));
}

#[test]
fn switching_on_mid_episode_shows_it_and_off_lets_go() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    let fake = Fake::default();
    core.set_presence_transport(Box::new(fake.clone()));

    let s = open(&core, lib.ep2);
    tick(&core, s, 1.0, false);
    switch(&core, true);
    core.presence_flush();
    assert_eq!(sets(&fake.sent()).len(), 1);

    switch(&core, false);
    tick(&core, s, 2.0, false);
    // Back on with a fresh flush point: the off sent its clear and closed.
    switch(&core, true);
    core.presence_flush();
    let sent = fake.sent();
    let off = sent.iter().position(|x| *x == Sent::Close).expect("closed");
    assert_eq!(sent[off - 1], Sent::Clear);
    assert!(matches!(sent.last(), Some(Sent::Set(_))));
    assert_eq!(sent.iter().filter(|x| **x == Sent::Connect).count(), 2);
}

#[test]
fn a_discord_that_is_not_there_is_tried_once_and_never_panics() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    let fake = Fake {
        fail: true,
        ..Fake::default()
    };
    core.set_presence_transport(Box::new(fake.clone()));
    switch(&core, true);

    let s = open(&core, lib.ep2);
    for i in 0..20 {
        tick(&core, s, f64::from(i), i % 2 == 1);
    }
    close(&core, s);
    core.presence_flush();
    // Twenty transitions inside the retry window, one attempt.
    assert_eq!(fake.sent(), vec![Sent::Connect]);
    assert_eq!(presence_reply(&core), (true, false));
}

#[test]
fn a_refused_poster_falls_back_to_the_apps_art() {
    let (_dir, core, _c) = common::open_core();
    let lib = library(&core);
    let fake = Fake {
        reject_posters: true,
        ..Fake::default()
    };
    core.set_presence_transport(Box::new(fake.clone()));
    switch(&core, true);
    let s = open(&core, lib.ep2);
    tick(&core, s, 1.0, false);
    tick(&core, s, 2.0, true);
    core.presence_flush();
    let sent = fake.sent();
    let all = sets(&sent);
    assert_eq!(all.len(), 2);
    assert!(all.iter().all(|a| a.poster.is_none()));
    assert_eq!(
        all[0].to_json()["assets"]["large_image"],
        presence::FALLBACK_ART
    );
}

#[test]
fn get_presence_says_whether_this_build_has_an_application() {
    let (_dir, core, _c) = common::open_core();
    assert_eq!(
        presence_reply(&core),
        (presence::bundled_client_id().is_some(), false)
    );
}
