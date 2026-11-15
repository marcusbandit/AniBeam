//! The event bus and the activity log live in the foundation crate (#854); this
//! hands them on as `crate::events`, beside the one piece that has to stay in
//! the core: `Subscription`, the handle `Core::subscribe` gives a shell, which
//! uniffi exports from the crate that holds `Core`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use anibeam_foundation::events::*;

use crate::contract::EventListener;

/// Handed back by `Core::subscribe`; dropping it or calling `unsubscribe`
/// removes the listener.
#[derive(uniffi::Object)]
pub struct Subscription {
    bus: Arc<EventBus>,
    id: u64,
    active: AtomicBool,
}

impl Subscription {
    pub fn new(bus: Arc<EventBus>, listener: Arc<dyn EventListener>) -> Arc<Subscription> {
        let id = bus.subscribe(listener);
        Arc::new(Subscription {
            bus,
            id,
            active: AtomicBool::new(true),
        })
    }
}

#[uniffi::export]
impl Subscription {
    pub fn unsubscribe(&self) {
        if self.active.swap(false, Ordering::SeqCst) {
            self.bus.unsubscribe(self.id);
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{EventBody, Stage};
    use crate::store::Store;

    #[test]
    fn dropping_a_subscription_unsubscribes() {
        let d = tempfile::tempdir().unwrap();
        let bus = EventBus::new(Store::open(&d.path().join("anibeam.db")).unwrap()).unwrap();
        let c = Arc::new(Collector::default());
        {
            let _sub = Subscription::new(bus.clone(), c.clone());
            bus.debug(Stage::System, "a", EventBody::Notice);
        }
        bus.debug(Stage::System, "b", EventBody::Notice);
        assert_eq!(c.events().len(), 1);
    }
}
