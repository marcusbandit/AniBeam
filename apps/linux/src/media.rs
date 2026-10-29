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
    pub volume: f64,
    pub can_next: bool,
    pub can_prev: bool,
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
