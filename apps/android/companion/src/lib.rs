//! Everything the phone app does that is not drawing: talking to the host, keeping the
//! last snapshot and the posters on disk, asking AniList for airing times while the host
//! is away, and turning all of that into the rows the screens show.
//!
//! None of it knows about Android or Slint, so it builds and tests on the desktop.

pub use anibeam_host_api as api;

/// A line from the catalog by message ID, in the language the app speaks now:
/// `tr!("fmt-ago-now")`, `tr!("fmt-ago-hours", count = n)`. A number argument picks the
/// plural form. Declared before the modules so every one of them sees it.
macro_rules! tr {
    ($id:expr) => {
        $crate::i18n::tr($id, &[])
    };
    ($id:expr, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::i18n::tr($id, &[$((stringify!($name), $crate::i18n::Arg::from($value))),+])
    };
}

pub mod address;
pub mod anilist;
pub mod calendar;
pub mod client;
pub mod format;
pub mod i18n;
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
