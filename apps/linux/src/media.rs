//! What the player page publishes for the desktop's media controls, whichever backend
//! carries it: MPRIS on Linux (`dbus::mpris`), the Now Playing centre and the media keys on
//! the Mac (`macnp`). The page calls the Player singleton's `mpris*` invokables, they call
//! `handle()`, and a command from either backend comes back to the page as one Qt signal.

use cxx_qt_lib::QString;
use mpris_server::PlaybackStatus;

#[cfg(not(target_os = "macos"))]
pub use crate::dbus::mpris::handle;
#[cfg(target_os = "macos")]
pub use crate::macnp::handle;

#[derive(Clone, Debug)]
pub struct State {
    pub status: PlaybackStatus,
    pub title: String,
    pub artist: String,
    pub art_url: Option<String>,
    pub length_secs: f64,
    pub position_secs: f64,
    /// 0 to 1. Only MPRIS reads it: the Mac's Now Playing has no volume.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    pub volume: f64,
    pub can_next: bool,
    pub can_prev: bool,
    /// mpv's speed. Only the Mac reads it: MPRIS reports a fixed Rate of 1.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub rate: f64,
}

impl Default for State {
    fn default() -> Self {
        State {
            status: PlaybackStatus::Stopped,
            title: String::new(),
            artist: String::new(),
            art_url: None,
            length_secs: 0.0,
            position_secs: 0.0,
            volume: 1.0,
            can_next: false,
            can_prev: false,
            rate: 1.0,
        }
    }
}

/// Whether a player page is on screen. The page publishes Playing or Paused for as long as
/// it lives and Stopped as it goes, so the status is the whole answer; a widget that reads
/// it knows not to offer transport controls that would do nothing.
pub fn mounted(state: &State) -> bool {
    state.status != PlaybackStatus::Stopped
}

/// The page publishes no position with its state, so an update carries the one already
/// held: the next real position arrives with the next observed frame, which never comes
/// while playback is paused. A stop is the one update that does mean zero, which is what
/// the spec says Position reads with nothing playing, and a position the caller did send
/// is the newest thing anyone knows.
pub fn carry_position(prev: &State, next: &mut State) {
    if !mounted(next) {
        next.position_secs = 0.0;
    } else if next.position_secs == 0.0 {
        next.position_secs = prev.position_secs;
    }
}

/// How far a frame may sit from Now Playing's own clock before the clock is corrected.
/// Frames come 25 to 60 times a second, so a late one is hundredths off; 2 s is the player
/// page's own line between playing on and a jump (PlayerPage.qml's onTimePosChanged).
#[cfg(any(target_os = "macos", test))]
pub const DRIFT_SECS: f64 = 2.0;

/// The rate Now Playing's clock runs at: the speed while playing, still otherwise.
#[cfg(any(target_os = "macos", test))]
pub fn clock_rate(state: &State) -> f64 {
    if state.status == PlaybackStatus::Playing {
        state.rate
    } else {
        0.0
    }
}

/// Whether a frame says Now Playing's clock is wrong. The system runs that clock itself,
/// from the last elapsed time it was given at the rate given with it, so `since` seconds
/// on it reads `written + since * rate`. A frame further off than that is a jump no seek
/// announced (the resume point, which lands after the page has published 0) or a stall.
#[cfg(any(target_os = "macos", test))]
pub fn drifted(written: f64, rate: f64, since: f64, frame: f64) -> bool {
    (frame - (written + since * rate)).abs() > DRIFT_SECS
}

/// One command for the page to act on, queued on the Qt thread, so it is safe from any
/// thread: MPRIS answers on the runtime and the Mac's remote commands on a thread Apple
/// does not name. The Player singleton registers its thread handle when the QML engine
/// constructs it (Main.qml touches it at start); until then a command has nowhere to go.
pub fn send(name: &str, value: f64) {
    let Some(player) = crate::bridge::player::thread() else {
        return;
    };
    let name = name.to_string();
    player
        .queue(move |p| p.mpris_command(QString::from(&name), value))
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_position_carries_forward_and_a_stop_resets_it() {
        let prev = State {
            position_secs: 42.0,
            ..State::default()
        };

        let mut paused = State {
            status: PlaybackStatus::Paused,
            ..State::default()
        };
        carry_position(&prev, &mut paused);
        assert_eq!(paused.position_secs, 42.0, "a pause holds where it is");

        let mut stopped = State {
            position_secs: 7.0,
            ..State::default()
        };
        carry_position(&prev, &mut stopped);
        assert_eq!(
            stopped.position_secs, 0.0,
            "nothing playing is position zero"
        );

        let mut seeked = State {
            status: PlaybackStatus::Playing,
            position_secs: 5.0,
            ..State::default()
        };
        carry_position(&prev, &mut seeked);
        assert_eq!(seeked.position_secs, 5.0, "a position that was sent wins");
    }

    #[test]
    fn a_frame_where_the_clock_says_writes_nothing() {
        assert!(!drifted(10.0, 1.0, 5.0, 15.04));
        assert!(!drifted(10.0, 1.0, 5.0, 13.5), "inside the threshold");
    }

    #[test]
    fn a_frame_past_the_threshold_corrects_the_clock() {
        assert!(drifted(10.0, 1.0, 5.0, 12.5), "a stall falls behind it");
        assert!(drifted(10.0, 1.0, 5.0, 40.0), "a jump runs ahead of it");
    }

    #[test]
    fn a_paused_clock_stands_still() {
        let paused = State {
            status: PlaybackStatus::Paused,
            rate: 1.5,
            ..State::default()
        };
        assert_eq!(clock_rate(&paused), 0.0);
        assert!(drifted(30.0, 0.0, 5.0, 35.0), "a frame that moves is news");
        assert!(!drifted(30.0, 0.0, 5.0, 30.04), "a frame step is not");
    }

    #[test]
    fn the_clock_runs_at_the_speed() {
        let playing = State {
            status: PlaybackStatus::Playing,
            rate: 1.5,
            ..State::default()
        };
        assert_eq!(clock_rate(&playing), 1.5);
        assert!(!drifted(10.0, 1.5, 4.0, 16.0), "1.5x for 4 s is 6 s on");
        assert!(
            drifted(10.0, 1.0, 8.0, 22.0),
            "a clock told 1x falls behind 1.5x"
        );
    }

    #[test]
    fn the_resume_point_corrects_a_publish_of_zero() {
        assert!(drifted(0.0, 1.0, 0.05, 60.0));
    }

    #[test]
    fn the_transport_is_offered_only_while_a_page_is_up() {
        assert!(
            !mounted(&State::default()),
            "nothing playing, nothing to do"
        );
        assert!(mounted(&State {
            status: PlaybackStatus::Playing,
            ..State::default()
        }));
        assert!(mounted(&State {
            status: PlaybackStatus::Paused,
            ..State::default()
        }));
    }
}
