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

/// The account says it is connected but there is no token behind it: an
/// AniList keyring entry the store lost, or a MAL session whose refresh has
/// failed and already said so.
pub(crate) const NO_TOKEN: &str = "no access token stored, reconnect in Settings";

/// A count off a provider's JSON, which is unsigned and small: anything
/// missing or absurd is nought rather than a wrap-around.
pub(crate) fn as_count(value: Option<u64>) -> u32 {
    as_u32(value).unwrap_or(0)
}

/// The same number when the provider sent one it can hold, and nothing when
/// it sent none or sent something absurd.
pub(crate) fn as_u32(value: Option<u64>) -> Option<u32> {
    value.and_then(|v| u32::try_from(v).ok())
}
