//! A team's link to a team above it (#854). Each team holds the teams below
//! it outright; a call that still goes up, such as the library asking the
//! downloads for what is still arriving, goes through one of these until the
//! call becomes an event or a trait of the team below.

use std::sync::{Arc, OnceLock, Weak};

/// Set once by `Core::open`, after both teams exist. A `Weak`, so the two
/// never keep each other alive; `get` is `None` before the set and once the
/// core has gone, and a caller treats that as nothing to tell.
pub struct Above<T>(OnceLock<Weak<T>>);

impl<T> Above<T> {
    pub(crate) fn set(&self, team: &Arc<T>) {
        let _ = self.0.set(Arc::downgrade(team));
    }

    pub(crate) fn get(&self) -> Option<Arc<T>> {
        self.0.get().and_then(Weak::upgrade)
    }

    /// The link itself, for a holder that upgrades it later. Empty before
    /// the set.
    pub(crate) fn weak(&self) -> Weak<T> {
        self.0.get().cloned().unwrap_or_default()
    }
}

impl<T> Default for Above<T> {
    fn default() -> Self {
        Above(OnceLock::new())
    }
}
