//! Whether a newer version of the app is out (#809). The core asks GitHub for the phone's
//! releases on this build's own track, once soon after launch and again every few hours
//! while the app stays open; More's About says so and opens the APK's download in the
//! browser. A build from the Play Store, or one on the iPhone, never asks: the store
//! updates it.

use std::cell::RefCell;
use std::time::{Duration, Instant};

use anibeam_core::{Call, PhoneUpdate, Reply};
use slint::{ComponentHandle, Timer, TimerMode, Weak};

use crate::{AppData, AppWindow};

/// How long a check that answered holds until the next.
const AGAIN_AFTER: Duration = Duration::from_secs(6 * 60 * 60);
/// How long a check that failed, offline say, holds until the next.
const RETRY_AFTER: Duration = Duration::from_secs(30 * 60);
/// How often the timer looks whether a check is due. It asks nothing itself: the core opens
/// on its own thread after launch, and this is what waits for it.
const LOOK_EVERY: Duration = Duration::from_secs(20);

struct Newer {
    window: Weak<AppWindow>,
    /// When the next check may go out; `None` until the first.
    due: Option<Instant>,
    busy: bool,
    /// Where the offered version is got.
    url: String,
    _timer: Timer,
}

thread_local! {
    static NEWER: RefCell<Option<Newer>> = const { RefCell::new(None) };
}

fn with(f: impl FnOnce(&mut Newer)) {
    NEWER.with(|cell| {
        if let Some(n) = cell.borrow_mut().as_mut() {
            f(n);
        }
    });
}

/// Whether this build hears about its own updates: only an APK put on the phone by hand.
fn offered() -> bool {
    cfg!(target_os = "android") && env!("ANIBEAM_FLAVOR") != "play"
}

pub fn install(window: &AppWindow) {
    window.global::<AppData>().on_open_update(|| {
        with(|n| {
            if !n.url.is_empty() {
                crate::platform::open_url(&n.url);
            }
        })
    });
    if !offered() {
        return;
    }
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, LOOK_EVERY, || with(Newer::look));
    NEWER.with(|cell| {
        *cell.borrow_mut() = Some(Newer {
            window: window.as_weak(),
            due: None,
            busy: false,
            url: String::new(),
            _timer: timer,
        })
    });
    with(Newer::look);
}

impl Newer {
    fn look(&mut self) {
        if self.busy || self.due.is_some_and(|at| Instant::now() < at) {
            return;
        }
        let Some(core) = crate::local::core() else {
            return;
        };
        self.busy = true;
        let spawned = std::thread::Builder::new()
            .name("newer".into())
            .spawn(move || {
                let answer = core.call(Call::CheckPhoneUpdate {
                    current: env!("ANIBEAM_PHONE_VERSION").to_string(),
                });
                let _ = slint::invoke_from_event_loop(move || with(|n| n.heard(answer)));
            });
        if spawned.is_err() {
            self.busy = false;
            self.due = Some(Instant::now() + RETRY_AFTER);
        }
    }

    fn heard(&mut self, answer: Result<Reply, anibeam_core::CoreError>) {
        self.busy = false;
        let update = match answer {
            Ok(Reply::PhoneUpdate { update }) => update,
            Ok(_) => None,
            Err(e) => {
                crate::local::log(&format!("update check: {e}"));
                self.due = Some(Instant::now() + RETRY_AFTER);
                return;
            }
        };
        self.due = Some(Instant::now() + AGAIN_AFTER);
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let (version, url) = match update {
            Some(PhoneUpdate { version, url }) => (version, url),
            None => (String::new(), String::new()),
        };
        self.url = url;
        window.global::<AppData>().set_update_version(version.into());
    }
}
