//! MPRIS through mpris-server on the connection that also carries the application name.
//! State comes from the player page through the Player singleton; commands go back to it
//! as one Qt signal.

use std::sync::{Arc, Mutex};

use cxx_qt::CxxQtThread;
use cxx_qt_lib::QString;
use mpris_server::{
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, Property, RootInterface,
    Server, Signal, Time, TrackId, Volume,
    zbus::{Result as ZResult, fdo},
};

use crate::bridge::shell::qobject::Shell;

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

fn time(secs: f64) -> Time {
    Time::from_millis(if secs.is_finite() {
        (secs * 1000.0) as i64
    } else {
        0
    })
}

/// Whether a player page is on screen. The page publishes Playing or Paused for as long as
/// it lives and Stopped as it goes, so the status is the whole answer; a widget that reads
/// it knows not to offer transport controls that would do nothing.
fn mounted(state: &State) -> bool {
    state.status != PlaybackStatus::Stopped
}

/// The page publishes no position with its state, so an update carries the one already
/// held: the next real position arrives with the next observed frame, which never comes
/// while playback is paused. A stop is the one update that does mean zero, which is what
/// the spec says Position reads with nothing playing, and a position the caller did send
/// is the newest thing anyone knows.
fn carry_position(prev: &State, next: &mut State) {
    if !mounted(next) {
        next.position_secs = 0.0;
    } else if next.position_secs == 0.0 {
        next.position_secs = prev.position_secs;
    }
}

pub struct MprisPlayer {
    state: Arc<Mutex<State>>,
    shell: CxxQtThread<Shell>,
}

impl MprisPlayer {
    /// The Player singleton registers its thread handle when the QML engine constructs it
    /// (Main.qml touches it at start); until then a command has nowhere to go.
    fn send(&self, name: &str, value: f64) {
        let Some(player) = crate::bridge::player::thread() else {
            return;
        };
        let name = name.to_string();
        player
            .queue(move |p| p.mpris_command(QString::from(&name), value))
            .ok();
    }
    fn state(&self) -> State {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }
    fn metadata(&self) -> Metadata {
        let s = self.state();
        let mut b = Metadata::builder()
            .trackid(TrackId::NO_TRACK)
            .title(s.title.clone())
            .artist([s.artist.clone()])
            .length(time(s.length_secs));
        if let Some(url) = s.art_url {
            b = b.art_url(url)
        }
        b.build()
    }
}

impl RootInterface for MprisPlayer {
    async fn raise(&self) -> fdo::Result<()> {
        self.shell
            .queue(|s| s.activate_requested(QString::default()))
            .ok();
        Ok(())
    }
    async fn quit(&self) -> fdo::Result<()> {
        self.shell.queue(|s| s.quit_requested()).ok();
        Ok(())
    }
    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn set_fullscreen(&self, _f: bool) -> ZResult<()> {
        Ok(())
    }
    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn identity(&self) -> fdo::Result<String> {
        Ok("AniBeam".into())
    }
    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok(crate::APP_ID.into())
    }
    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }
    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }
}

impl PlayerInterface for MprisPlayer {
    async fn next(&self) -> fdo::Result<()> {
        self.send("next", 0.0);
        Ok(())
    }
    async fn previous(&self) -> fdo::Result<()> {
        self.send("previous", 0.0);
        Ok(())
    }
    async fn pause(&self) -> fdo::Result<()> {
        self.send("pause", 0.0);
        Ok(())
    }
    async fn play_pause(&self) -> fdo::Result<()> {
        self.send("playPause", 0.0);
        Ok(())
    }
    async fn stop(&self) -> fdo::Result<()> {
        self.send("stop", 0.0);
        Ok(())
    }
    async fn play(&self) -> fdo::Result<()> {
        self.send("play", 0.0);
        Ok(())
    }
    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        self.send("seek", offset.as_micros() as f64 / 1e6);
        Ok(())
    }
    async fn set_position(&self, _track: TrackId, position: Time) -> fdo::Result<()> {
        self.send("setPosition", position.as_micros() as f64 / 1e6);
        Ok(())
    }
    async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
        Ok(())
    }
    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        Ok(self.state().status)
    }
    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        Ok(LoopStatus::None)
    }
    async fn set_loop_status(&self, _l: LoopStatus) -> ZResult<()> {
        Ok(())
    }
    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }
    async fn set_rate(&self, _r: PlaybackRate) -> ZResult<()> {
        Ok(())
    }
    async fn shuffle(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn set_shuffle(&self, _s: bool) -> ZResult<()> {
        Ok(())
    }
    async fn metadata(&self) -> fdo::Result<Metadata> {
        Ok(MprisPlayer::metadata(self))
    }
    async fn volume(&self) -> fdo::Result<Volume> {
        Ok(self.state().volume)
    }
    async fn set_volume(&self, v: Volume) -> ZResult<()> {
        self.send("setVolume", (v * 100.0).clamp(0.0, 100.0));
        Ok(())
    }
    async fn position(&self) -> fdo::Result<Time> {
        Ok(time(self.state().position_secs))
    }
    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }
    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }
    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(self.state().can_next)
    }
    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(self.state().can_prev)
    }
    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(mounted(&self.state()))
    }
    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(mounted(&self.state()))
    }
    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(self.state().length_secs > 0.0)
    }
    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(mounted(&self.state()))
    }
}

#[derive(Clone)]
pub struct Handle {
    state: Arc<Mutex<State>>,
    server: Arc<Server<MprisPlayer>>,
}

impl Handle {
    /// Everything the page knows, with the position carried forward. CanControl is left
    /// out of the signal on purpose: the spec marks it as emitting no change, so a widget
    /// re-reads it rather than being told.
    pub fn update(&self, next: State) {
        let mut next = next;
        if let Ok(mut s) = self.state.lock() {
            carry_position(&s, &mut next);
            *s = next.clone();
        }
        let server = self.server.clone();
        let md = server.imp().metadata();
        crate::runtime::runtime().spawn(async move {
            server
                .properties_changed([
                    Property::PlaybackStatus(next.status),
                    Property::Metadata(md),
                    Property::Volume(next.volume),
                    Property::CanGoNext(next.can_next),
                    Property::CanGoPrevious(next.can_prev),
                    Property::CanSeek(next.length_secs > 0.0),
                    Property::CanPlay(mounted(&next)),
                    Property::CanPause(mounted(&next)),
                ])
                .await
                .ok();
        });
    }
    /// Position is read, not signalled: MPRIS says Position emits no PropertiesChanged, so
    /// this only keeps the value a Get answers with.
    pub fn position(&self, secs: f64) {
        if let Ok(mut s) = self.state.lock() {
            s.position_secs = secs;
        }
    }
    pub fn seeked(&self, secs: f64) {
        self.position(secs);
        let server = self.server.clone();
        crate::runtime::runtime().spawn(async move {
            server
                .emit(Signal::Seeked {
                    position: time(secs),
                })
                .await
                .ok();
        });
    }
}

static HANDLE: std::sync::OnceLock<Handle> = std::sync::OnceLock::new();
pub fn install(h: Handle) {
    HANDLE.set(h).ok();
}
pub fn handle() -> Option<Handle> {
    HANDLE.get().cloned()
}

/// Builds the MPRIS server, then serves org.freedesktop.Application on its connection and
/// requests the app id there. Both names carry the sandbox element under --root, so a dev
/// run never takes the real app's; two sandboxes still share one pair of names, since the
/// element says sandbox and not which root. None means the MPRIS name never came, from no
/// session bus or from a second sandbox: one line on stderr, no media keys.
pub async fn start(shell: CxxQtThread<Shell>) -> Option<Handle> {
    let state = Arc::new(Mutex::new(State::default()));
    let imp = MprisPlayer {
        state: state.clone(),
        shell: shell.clone(),
    };
    let server = match Server::new(&super::instance::mpris_bus_suffix(), imp).await {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("anibeam: MPRIS and media keys are off: {e}");
            return None;
        }
    };
    if let Err(e) = super::instance::serve(server.connection(), shell).await {
        eprintln!(
            "anibeam: could not own {}: {e}",
            super::instance::bus_name()
        );
    }
    Some(Handle { state, server })
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
