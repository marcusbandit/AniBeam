//! The metadata team (#854): matching and fetching, the provider records
//! every match, refresh and crawl step writes through (`record`), the
//! franchise crawl and closure (`franchise`), Recommended (`recommended`),
//! Airing now (`airing_now`), the aired episodes (`aired`) and the calendar
//! feed (`calendar`). The core hands it on as `anibeam_core::metadata`, with
//! those five at the core's root where they were.

use std::time::Duration;

pub mod aired;
pub mod airing;
pub mod airing_now;
pub mod apply;
pub mod automatch;
pub mod calendar;
pub mod fetch;
pub mod franchise;
pub mod imdb;
pub mod link;
pub mod recommended;
pub mod record;
pub mod similarity;
mod team;

pub use team::Metadata;

/// How often the core is willing to say out loud that Jikan is down. Jikan
/// is the episode-title side-fetch and nothing else, so an outage costs a
/// series its titles rather than its match; a job walking a whole library
/// through a Jikan outage would otherwise write one warning per series
/// into an activity log that is meant to carry state changes only.
pub(crate) const OUTAGE_WINDOW: Duration = Duration::from_secs(600);
