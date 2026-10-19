//! Keeping the display awake while an episode plays. Standalone mpv does this itself, but
//! the render API (`vo=libmpv`) has no window and ignores VOCTRL_KILL_SCREENSAVER, so the
//! embedding app has to ask the OS (#78).
//!
//! The state is one bool, wanted or not. A worker task follows it and takes or drops the
//! platform hold one change at a time, so a quick on-off-on never leaves two holds, and the
//! Qt thread never waits on the bus.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, watch};

#[cfg(not(target_os = "macos"))]
use crate::dbus::inhibit::{Backend, Hold};
#[cfg(target_os = "macos")]
use crate::iokit::{Backend, Hold};

/// A quit must not hang on a bus that never answers. The OS drops the hold with the
/// process anyway, so giving up is safe.
const RELEASE_TIMEOUT: Duration = Duration::from_secs(2);

struct Inner {
    backend: Backend,
    hold: Option<Hold>,
    /// Set by Drop under the lock, so a worker that was already waiting on it cannot take
    /// a hold after the release.
    closed: bool,
}

pub struct Inhibitor {
    wanted: watch::Sender<bool>,
    inner: Arc<Mutex<Inner>>,
}

impl Inhibitor {
    pub fn new() -> Self {
        let (wanted, mut rx) = watch::channel(false);
        let inner = Arc::new(Mutex::new(Inner {
            backend: Backend::new(),
            hold: None,
            closed: false,
        }));
        let task = inner.clone();
        crate::runtime::runtime().spawn(async move {
            while rx.changed().await.is_ok() {
                let want = *rx.borrow_and_update();
                let mut g = task.lock().await;
                if g.closed {
                    break;
                }
                match (want, g.hold.is_some()) {
                    (true, false) => {
                        let hold = g.backend.acquire().await;
                        g.hold = hold;
                    }
                    (false, true) => {
                        if let Some(hold) = g.hold.take() {
                            g.backend.release(hold).await;
                        }
                    }
                    _ => {}
                }
            }
        });
        Inhibitor { wanted, inner }
    }

    /// Idempotent: asking for the state already asked for does nothing.
    pub fn set(&self, on: bool) {
        self.wanted.send_if_modified(|v| {
            let changed = *v != on;
            *v = on;
            changed
        });
    }
}

impl Drop for Inhibitor {
    fn drop(&mut self) {
        self.wanted.send_replace(false);
        let inner = self.inner.clone();
        let release = async move {
            let mut g = inner.lock().await;
            g.closed = true;
            if let Some(hold) = g.hold.take() {
                g.backend.release(hold).await;
            }
        };
        // The timeout is built inside the future: it needs the runtime's time driver, which
        // only exists once block_on has entered it.
        let _ = crate::runtime::runtime()
            .block_on(async move { tokio::time::timeout(RELEASE_TIMEOUT, release).await });
    }
}
