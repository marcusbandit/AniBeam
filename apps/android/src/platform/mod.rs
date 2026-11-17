//! Everything that differs by platform, behind one door: the rest of the app calls
//! `crate::platform::*` and never names a platform itself.
//!
//! `android.rs` is the phone: its entry point and the calls only Java answers, over JNI.
//! `ios.rs` is the iPhone: its entry point and the hinge, with `other.rs`'s stand-ins for
//! the rest, which it names one by one. `other.rs` stands in on every other target (the
//! desktop the tests and host clippy run on, and any shell that has not brought its own yet)
//! and does the harmless thing: nothing, None, or the value a phone without the feature
//! would give. The target's file is re-exported through the one list below: the app reaches
//! only the calls named there, so a helper made `pub` in one file stays out of it, and a
//! listed call a target lacks fails that target's build. What every platform shares sits
//! here.

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
use android as target;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
use ios as target;
#[cfg(target_os = "ios")]
pub use ios::ios_main;

#[cfg(not(target_os = "android"))]
mod other;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use other as target;

// The platform calls. A new call goes in this list and in every platform file.
pub use target::{
    app_icon, ask_notifications, bars, brightness, copy_text, crease, downloads_paused,
    episode_ready, fold, give_audio_focus, line_colours, log, multicast, notification_settings,
    notifications_allowed, open_url, pick_file, player_awake, player_screen, relocalize,
    rest_service, service_go_on, set_app_icon, set_brightness, set_volume, start_service,
    stop_service, system_dark, system_locale, take_audio_focus, unmetered, update_service, volume,
    window_up, work_state,
};
// Only the player calls these, and it links libmpv only where libmpv was fetched.
#[cfg_attr(not(libmpv), allow(unused_imports))]
pub use target::{emulator, prepare_mpv};

/// What the platform said to a request for the audio focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Off the phone every request is granted.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub enum FocusAsk {
    Granted,
    /// Refused, as during a call: the episode should not play.
    Refused,
    /// The request could not be made, so focus changes would never arrive.
    Unavailable,
}

/// Where the work that keeps downloads going with the screen off stands, as the platform
/// says (`TorrentService.state` on Android). The platform sees every start, stop and cancel,
/// so the app reads this rather than guessing from what the downloads look like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Work {
    pub state: WorkState,
    /// Goes up on every change of state, so a stop and a rerun between two looks still show.
    pub generation: u32,
    /// The app is in front, not only kept up by the work.
    pub on_screen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkState {
    /// None: never started, ended by the app, or dropped because the app was opened.
    Idle,
    Running,
    /// Scheduled, and the platform has not run it yet.
    Waiting,
    /// Stopped by the platform, or rested by the app; it goes on when the app is opened.
    Stopped,
    /// Stopped by the person from the phone's list of running apps: it goes on once the app
    /// was away and is opened again.
    StoppedByUser,
}

impl Work {
    /// What a process starts with.
    pub const NONE: Work = Work {
        state: WorkState::Idle,
        generation: 0,
        on_screen: true,
    };

    /// `TorrentService.state`'s answer: the generation from bit 4 up, bit 3 for on screen,
    /// the state below.
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    pub fn from_bits(bits: i32) -> Work {
        let state = match bits & 7 {
            1 => WorkState::Running,
            2 => WorkState::Waiting,
            3 => WorkState::Stopped,
            4 => WorkState::StoppedByUser,
            _ => WorkState::Idle,
        };
        Work {
            state,
            generation: (bits as u32) >> 4,
            on_screen: bits & 8 != 0,
        }
    }

    /// Running or about to: what the app asks of it is a change in place.
    pub fn up(&self) -> bool {
        matches!(self.state, WorkState::Running | WorkState::Waiting)
    }

    pub fn stopped(&self) -> bool {
        matches!(self.state, WorkState::Stopped | WorkState::StoppedByUser)
    }
}

/// Bytes free and in total on the filesystem that holds `dir`.
pub fn space(dir: &std::path::Path) -> Option<(u64, u64)> {
    let s = rustix::fs::statvfs(dir).ok()?;
    Some((s.f_bavail * s.f_frsize, s.f_blocks * s.f_frsize))
}

/// Bytes the files under `dir` take.
pub fn used(dir: &std::path::Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            match e.metadata() {
                Ok(m) if m.is_dir() => stack.push(e.path()),
                Ok(m) => total += m.len(),
                Err(_) => {}
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_work_state_reads_as_java_packs_it() {
        let w = Work::from_bits((7 << 4) | 8 | 3);
        assert_eq!(w.state, WorkState::Stopped);
        assert_eq!(w.generation, 7);
        assert!(w.on_screen && w.stopped() && !w.up());
        let w = Work::from_bits((1 << 4) | 2);
        assert_eq!(w.state, WorkState::Waiting);
        assert!(!w.on_screen && w.up());
        assert_eq!(Work::from_bits(4).state, WorkState::StoppedByUser);
    }
}
