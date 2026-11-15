//! Everything that differs by platform, behind one door: the rest of the app calls
//! `crate::platform::*` and never names a platform itself.
//!
//! `android.rs` is the phone: its entry point and the calls only Java answers, over JNI.
//! `ios.rs` is the iPhone: its entry point and the hinge, with `other.rs`'s stand-ins for
//! the rest. `other.rs` stands in on every other target (the desktop the tests and host
//! clippy run on, and any shell that has not brought its own yet) and does the harmless
//! thing: nothing, None, or the value a phone without the feature would give. All offer the
//! same free functions with the same signatures, and this module re-exports the one for the
//! target. What every platform shares sits here.

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::*;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::*;

#[cfg(not(target_os = "android"))]
mod other;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use other::*;

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
