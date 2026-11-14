//! Everything that differs by platform, behind one door: the rest of the app calls
//! `crate::platform::*` and never names a platform itself.
//!
//! `android.rs` is the phone: its entry point and the calls only Java answers, over JNI.
//! `other.rs` stands in on every other target (the desktop the tests and host clippy run
//! on, and any shell that has not brought its own yet) and does the harmless thing: nothing,
//! None, or the value a phone without the feature would give. Both are re-exported through
//! the one list in `calls!`: the app reaches only the calls named there, so a helper made
//! `pub` in one file stays out of it, and a listed call one file lacks fails that target's
//! build. What every platform shares sits here.

/// Re-exports the platform calls from `$p`. A new call goes in this list and in every
/// platform file.
macro_rules! calls {
    ($p:ident) => {
        pub use $p::{
            app_icon, ask_notifications, bars, brightness, copy_text, crease, episode_ready, fold,
            give_audio_focus, line_colours, log, multicast, notification_settings,
            notifications_allowed, open_url, pick_file, player_awake, player_screen, relocalize,
            set_app_icon, set_brightness, set_volume, start_service, stop_service, system_dark,
            system_locale, take_audio_focus, unmetered, update_service, volume,
        };
        // Only the player calls these, and it links libmpv on Android alone.
        #[cfg_attr(not(libmpv), allow(unused_imports))]
        pub use $p::{emulator, prepare_mpv};
    };
}

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
calls!(android);

#[cfg(not(target_os = "android"))]
mod other;
#[cfg(not(target_os = "android"))]
calls!(other);

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
