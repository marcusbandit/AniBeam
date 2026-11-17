//! Numbers about watching that playback, the library, the trackers and the
//! preferences all read.

/// How far through a file counts as watched when no outro is known. The
/// number AniList and MAL use themselves.
pub const MARK_FRACTION: f64 = 0.85;

/// The days the On hold preference may ask for. Under a week would move a
/// show the user skipped one weekend; over a year is the same as never.
pub const ON_HOLD_MIN_DAYS: u32 = 7;
pub const ON_HOLD_MAX_DAYS: u32 = 365;
