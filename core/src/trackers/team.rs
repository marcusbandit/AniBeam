//! The trackers team (#854): AniList and MAL accounts, the list writes,
//! Watching, on hold and plan to watch.

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, AtomicU64};

use crate::foundation::services::Services;
use crate::net::Net;

/// The loopback port both providers redirect back to. Registered on their
/// developer panels, so it is a constant rather than a preference.
const DEFAULT_OAUTH_PORT: u16 = 53682;
/// A minute: long past any sign-in that is going well, and short of the
/// point where a browser gives up on the request itself.
const DEFAULT_SIGN_IN_HOLD_MS: u64 = 60_000;

pub struct Trackers {
    pub(crate) services: Arc<Services>,
    pub(crate) net: Arc<Net>,
    /// The loopback port the OAuth listener binds while a connect is in
    /// flight. Fixed rather than ephemeral: both providers only redirect
    /// to a URL registered on their developer panel, so the port is part
    /// of that registration. A test moves it out of the way.
    pub(crate) oauth_port: AtomicU16,
    /// How long, in milliseconds, the browser's callback request is held
    /// open for the sign-in to finish before the tab is let go. A test
    /// shortens it.
    pub(crate) sign_in_hold_ms: AtomicU64,
}

impl Trackers {
    pub(crate) fn new(services: Arc<Services>, net: Arc<Net>) -> Trackers {
        Trackers {
            services,
            net,
            oauth_port: AtomicU16::new(DEFAULT_OAUTH_PORT),
            sign_in_hold_ms: AtomicU64::new(DEFAULT_SIGN_IN_HOLD_MS),
        }
    }
}
