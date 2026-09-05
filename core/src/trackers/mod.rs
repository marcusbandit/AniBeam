//! The trackers: AniList and MAL, where their credentials live and what
//! the core does with them.

pub mod accounts;
pub mod cache;
pub mod oauth;
pub mod secrets;
pub mod watching;
pub mod writes;

pub use secrets::{Secrets, StoreKind};

/// How long one tracker request may take. Per request, never per call: the
/// limiter's 429 schedule sits under this and keeps its own bounds, so a
/// rate limit is always reported as a rate limit and never as a timeout.
/// A user is waiting on every one of these, which is why they are capped
/// tighter than the client's own 30 seconds.
pub(crate) const TRACKER_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
