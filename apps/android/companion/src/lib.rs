//! Everything the phone app does that is not drawing: talking to the host, keeping the
//! last snapshot and the posters on disk, asking AniList for airing times while the host
//! is away, and turning all of that into the rows the screens show.
//!
//! None of it knows about Android or Slint, so it builds and tests on the desktop.

pub use anibeam_host_api as api;

pub mod address;
pub mod anilist;
pub mod client;
pub mod format;
pub mod library;
pub mod schedule;
pub mod store;
pub mod sync;

/// The phone's clock, in Unix seconds.
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
