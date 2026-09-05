//! Provider records: the pure translation from a provider's reply to the
//! rows behind it. Every match, every refresh and every crawl step writes
//! through here, so the rules live in one place rather than once per job.

use std::time::Duration;

pub mod automatch;
pub mod fetch;
pub mod record;
pub mod similarity;

/// How often the core is willing to say out loud that Jikan is down. Jikan
/// is the episode-title side-fetch and nothing else, so an outage costs a
/// series its titles rather than its match; a job walking a whole library
/// through a Jikan outage would otherwise write one warning per series
/// into an activity log that is meant to carry state changes only.
pub(crate) const OUTAGE_WINDOW: Duration = Duration::from_secs(600);
