//! Whether a newer version of the app is out (#809). The core asks GitHub for the phone's
//! releases on this build's own track, once soon after launch and again when six hours
//! have passed on the wall clock, which a phone that slept through them sees on its first
//! second awake; More's About says so and opens the APK's download in the browser. A
//! build from the Play Store, one on the iPhone, or one built locally without a train
//! version never asks.
//!
//! The process outlives the activity, so when the check is due and what it found are kept
//! process-wide; a recreated window is handed the last answer instead of asking again.

use std::cell::RefCell;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use anibeam_core::{Call, CoreError, PhoneUpdate, Reply};
use slint::{ComponentHandle, Weak};

use crate::{AppData, AppWindow};

/// How long a check that answered holds until the next.
const AGAIN_AFTER: Duration = Duration::from_secs(6 * 60 * 60);
/// How long a check that failed, offline say, holds until the next.
const RETRY_AFTER: Duration = Duration::from_secs(30 * 60);

/// The flavor build.rs passed through: `play` for the Play Store's bundle, empty for the APK.
const FLAVOR: &str = env!("ANIBEAM_FLAVOR");

struct State {
    /// When the next check may go out, on the wall clock, and the wait that set it; `None`
    /// until the first.
    due: Option<(SystemTime, Duration)>,
    busy: bool,
    /// What the last answer offered.
    offer: Option<PhoneUpdate>,
}

static STATE: Mutex<State> = Mutex::new(State {
    due: None,
    busy: false,
    offer: None,
});

thread_local! {
    static WINDOW: RefCell<Option<Weak<AppWindow>>> = const { RefCell::new(None) };
}

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Whether a versionCode is a release train's: a local build of a version takes step 0
/// (`build.sh` gives 2.4.3 built by hand 204030000), and a plain cargo build has code 0.
fn train_build(code: &str) -> bool {
    code.parse::<u64>().is_ok_and(|c| c % 10_000 != 0)
}

/// Whether this build hears about its own updates: only an APK from a release train.
fn offered() -> bool {
    cfg!(target_os = "android") && FLAVOR != "play" && train_build(env!("ANIBEAM_VERSION_CODE"))
}

/// Whether a check is due at `now`. Time left past the wait that set the due time means
/// the clock was set back, and the check is due at once, so a wrong date cannot hold it
/// off for the whole distance.
fn due(at: Option<(SystemTime, Duration)>, now: SystemTime) -> bool {
    match at {
        None => true,
        Some((at, wait)) => at
            .duration_since(now)
            .ok()
            .is_none_or(|left| left.is_zero() || left > wait),
    }
}

/// The due time `wait` from now, with the wait beside it.
fn after(wait: Duration) -> Option<(SystemTime, Duration)> {
    Some((SystemTime::now() + wait, wait))
}

pub fn install(window: &AppWindow) {
    WINDOW.with(|w| *w.borrow_mut() = Some(window.as_weak()));
    window.global::<AppData>().on_open_update(|| {
        let url = state().offer.as_ref().map(|o| o.url.clone());
        if let Some(url) = url {
            crate::platform::open_url(&url);
        }
    });
    let offer = state().offer.clone();
    show(offer);
}

/// Called every second by the app's clock: asks when a check is due and the core is open.
pub fn tick() {
    if !offered() {
        return;
    }
    let mut st = state();
    if st.busy || !due(st.due, SystemTime::now()) {
        return;
    }
    let call = Call::CheckPhoneUpdate {
        current: env!("ANIBEAM_PHONE_VERSION").to_string(),
    };
    // A panic in the call still answers, so the check is never left busy for good
    st.busy = crate::local::call_off_thread("newer", call, heard);
}

/// On the UI thread: keeps the answer, then shows it.
fn heard(answer: Result<Reply, CoreError>) {
    let offer = {
        let mut st = state();
        st.busy = false;
        match answer {
            Ok(Reply::PhoneUpdate { update }) => {
                st.due = after(AGAIN_AFTER);
                st.offer = update;
            }
            Ok(other) => {
                crate::local::log(&format!("update check: unexpected reply {other:?}"));
                st.due = after(RETRY_AFTER);
                return;
            }
            Err(e) => {
                crate::local::log(&format!("update check: {e}"));
                st.due = after(RETRY_AFTER);
                return;
            }
        }
        st.offer.clone()
    };
    show(offer);
}

/// On the UI thread: the window says what is offered, or nothing.
fn show(offer: Option<PhoneUpdate>) {
    let Some(window) = WINDOW.with(|w| w.borrow().as_ref().and_then(|w| w.upgrade())) else {
        return;
    };
    let version = offer.map(|o| o.version).unwrap_or_default();
    window
        .global::<AppData>()
        .set_update_version(version.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_train_build_asks() {
        assert!(train_build("204030097"));
        assert!(train_build("204039999"));
        assert!(!train_build("204030000"));
        assert!(!train_build("0"));
        assert!(!train_build(""));
    }

    #[test]
    fn a_check_is_due_on_the_wall_clock() {
        let now = SystemTime::now();
        let set = |left: Duration, wait: Duration| Some((now + left, wait));
        assert!(due(None, now));
        assert!(due(Some((now - Duration::from_secs(1), AGAIN_AFTER)), now));
        assert!(!due(set(Duration::from_secs(60), AGAIN_AFTER), now));
        // A clock set back by days
        assert!(due(set(AGAIN_AFTER * 4, AGAIN_AFTER), now));
        // A retry's half hour, with the clock set back five hours, goes at once
        assert!(due(
            set(Duration::from_secs(5 * 60 * 60 + 30 * 60), RETRY_AFTER),
            now
        ));
        assert!(!due(set(Duration::from_secs(60), RETRY_AFTER), now));
    }
}
