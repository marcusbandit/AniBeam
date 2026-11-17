//! A team's link to a team above it (#854). Each team holds the teams below
//! it outright. When it has to ask the team above it something, such as the
//! library asking what a torrent is still writing, it names a trait of its
//! own, the team above implements it, and this holds the link to it. What a
//! team only tells the teams above it goes on the bus instead
//! (`events::Signal`).

use std::sync::{Arc, OnceLock, Weak};

/// Set once by `Core::open`, after both teams exist. A `Weak`, so the two
/// never keep each other alive; `get` is `None` before the set and once the
/// team above has gone, and a caller treats that as nothing to ask.
pub struct Above<T: ?Sized>(OnceLock<Weak<T>>);

impl<T: ?Sized> Above<T> {
    pub fn set(&self, team: Weak<T>) {
        let _ = self.0.set(team);
    }

    pub fn get(&self) -> Option<Arc<T>> {
        self.0.get().and_then(Weak::upgrade)
    }
}

impl<T: ?Sized> Default for Above<T> {
    fn default() -> Self {
        Above(OnceLock::new())
    }
}
