//! Whether a newer version of the app is out (#809). The core asks GitHub for the phone's
//! releases on the phone's track, soon after the first launch and again when six hours
//! have passed on the wall clock, which a phone that slept through them sees on its first
//! second awake; More's About says so and opens the APK's download in the browser. A
//! build from the Play Store, one on the iPhone, or one built locally without a train
//! version never asks.
//!
//! The last answer, when it came and the track are kept in `update.json` beside the core's
//! folder, so a launch inside the six hours shows the answer it already has rather than
//! fetching the list again. The track is nightly from the first nightly build the phone
//! runs, and stays nightly when that phone takes a stable release, until the person picks
//! Stable in About. In memory the state is process-wide, since the process outlives the
//! activity.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anibeam_core::{Call, CoreError, PhoneUpdate, Reply};
use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, Weak};

use crate::{AppData, AppWindow};

/// How long a check that answered holds until the next.
const AGAIN_AFTER: Duration = Duration::from_secs(6 * 60 * 60);
/// How long a check that failed, offline say, holds until the next.
const RETRY_AFTER: Duration = Duration::from_secs(30 * 60);
/// How long a check may be out before another may go. An answer that never reached the UI
/// thread (the loop ended while it was out) would otherwise hold the check for good.
const OUT_AT_MOST: Duration = Duration::from_secs(2 * 60);
/// How far the clock may step back before it counts as set back: an NTP correction moves it
/// by seconds, and must not send the check again at once.
const SLACK: Duration = Duration::from_secs(5 * 60);

/// The flavor build.rs passed through: `play` for the Play Store's bundle, empty for the APK.
const FLAVOR: &str = env!("ANIBEAM_FLAVOR");
/// The version this build runs.
const RUNNING: &str = env!("ANIBEAM_PHONE_VERSION");

/// What `update.json` holds.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Kept {
    /// The phone follows nightlies.
    nightly: bool,
    /// The person picked the track in About, so a build no longer sets it.
    picked: bool,
    /// The build that last asked, and when its answer came, in seconds since 1970.
    asked_by: String,
    answered_at: u64,
    /// What that answer offered.
    offer: Option<PhoneUpdate>,
}

struct State {
    file: Option<PathBuf>,
    kept: Kept,
    /// When the next check may go out, on the wall clock, and the wait that set it; `None`
    /// until the first.
    due: Option<(SystemTime, Duration)>,
    /// When the check out now went.
    out_since: Option<SystemTime>,
    /// The number of the latest check; an answer to an older one is dropped.
    asked: u64,
}

static STATE: Mutex<State> = Mutex::new(State {
    file: None,
    kept: Kept {
        nightly: false,
        picked: false,
        asked_by: String::new(),
        answered_at: 0,
        offer: None,
    },
    due: None,
    out_since: None,
    asked: 0,
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
            .is_none_or(|left| left.is_zero() || left > wait + SLACK),
    }
}

/// The due time `wait` from now, with the wait beside it.
fn after(wait: Duration) -> Option<(SystemTime, Duration)> {
    Some((SystemTime::now() + wait, wait))
}

/// Whether a check that went at `since` still counts as out at `now`.
fn out(since: Option<SystemTime>, now: SystemTime) -> bool {
    since.is_some_and(|at| match now.duration_since(at) {
        Ok(gone) => gone < OUT_AT_MOST,
        // The clock stepped back while it was out: a small step still counts as out
        Err(back) => back.duration() < SLACK,
    })
}

/// What the phone keeps after this build starts: nightly once any nightly build ran, and
/// the last answer only when this build asked for it.
fn on_start(mut kept: Kept, running: &str) -> Kept {
    if !kept.picked {
        kept.nightly |= running.contains("-nightly.");
    }
    if kept.asked_by != running {
        kept.asked_by = running.to_string();
        kept.answered_at = 0;
        kept.offer = None;
    }
    kept
}

/// When the next check is due for what was kept: six hours after its answer.
fn due_for(kept: &Kept) -> Option<(SystemTime, Duration)> {
    // A damaged file's time past what the clock can hold asks at once
    let at = UNIX_EPOCH
        .checked_add(Duration::from_secs(kept.answered_at))?
        .checked_add(AGAIN_AFTER)?;
    (kept.answered_at > 0).then_some((at, AGAIN_AFTER))
}

fn file_in(data_dir: &Path) -> PathBuf {
    data_dir.join("update.json")
}

fn load(file: &Path) -> Kept {
    std::fs::read(file)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn save(file: &Path, kept: &Kept) {
    let bytes = match serde_json::to_vec_pretty(kept) {
        Ok(bytes) => bytes,
        Err(e) => return crate::local::log(&format!("update check: could not save: {e}")),
    };
    if let Err(e) = anibeam_core::atomic_file::write_private(file, bytes) {
        crate::local::log(&format!("update check: could not save: {e}"));
    }
}

pub fn install(window: &AppWindow, data_dir: &Path) {
    WINDOW.with(|w| *w.borrow_mut() = Some(window.as_weak()));
    window
        .global::<AppData>()
        .on_set_update_nightly(pick_nightly);
    window.global::<AppData>().on_open_update(|| {
        let url = state().kept.offer.as_ref().map(|o| o.url.clone());
        if let Some(url) = url {
            crate::platform::open_url(&url);
        }
    });
    let offer = {
        let mut st = state();
        if st.file.is_none() && offered() {
            let file = file_in(data_dir);
            let before = load(&file);
            let kept = on_start(before.clone(), RUNNING);
            if kept != before {
                save(&file, &kept);
            }
            st.due = due_for(&kept);
            st.kept = kept;
            st.file = Some(file);
        }
        st.kept.offer.clone()
    };
    show(offer);
}

/// Called every second by the app's clock: asks when a check is due and the core is open.
pub fn tick() {
    if !offered() {
        return;
    }
    let mut st = state();
    let now = SystemTime::now();
    if out(st.out_since, now) || !due(st.due, now) {
        return;
    }
    let call = Call::CheckPhoneUpdate {
        current: RUNNING.to_string(),
        nightly: Some(st.kept.nightly),
    };
    let number = st.asked + 1;
    // A panic in the call still answers, so the check is not left out for good
    if crate::local::call_off_thread("newer", call, move |answer| heard(number, answer)) {
        st.asked = number;
        st.out_since = Some(now);
    }
}

/// The person picked a track in About: kept, and asked again on it at once.
fn pick_nightly(nightly: bool) {
    {
        let mut st = state();
        if st.kept.picked && st.kept.nightly == nightly {
            return;
        }
        st.kept.nightly = nightly;
        st.kept.picked = true;
        st.kept.answered_at = 0;
        st.kept.offer = None;
        st.due = None;
        // An answer still out is for the other track
        st.asked += 1;
        st.out_since = None;
        if let Some(file) = &st.file {
            save(file, &st.kept);
        }
    }
    show(None);
}

/// On the UI thread: keeps the answer, then shows it.
fn heard(number: u64, answer: Result<Reply, CoreError>) {
    let offer = {
        let mut st = state();
        // A later check went out since, or the track changed: this answer is stale
        if number != st.asked {
            return;
        }
        st.out_since = None;
        match answer {
            Ok(Reply::PhoneUpdate { update }) => {
                st.due = after(AGAIN_AFTER);
                st.kept.answered_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs());
                st.kept.offer = update;
                if let Some(file) = &st.file {
                    save(file, &st.kept);
                }
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
        st.kept.offer.clone()
    };
    show(offer);
}

/// On the UI thread: the window says what is offered, or nothing.
fn show(offer: Option<PhoneUpdate>) {
    let Some(window) = WINDOW.with(|w| w.borrow().as_ref().and_then(|w| w.upgrade())) else {
        return;
    };
    let version = offer.map(|o| o.version).unwrap_or_default();
    let nightly = state().kept.nightly;
    let data = window.global::<AppData>();
    data.set_update_version(version.into());
    data.set_update_offered(offered());
    data.set_update_nightly(nightly);
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

    #[test]
    fn a_check_out_too_long_no_longer_holds_the_next() {
        let now = SystemTime::now();
        assert!(!out(None, now));
        assert!(out(Some(now - Duration::from_secs(10)), now));
        assert!(!out(Some(now - OUT_AT_MOST * 2), now));
    }

    #[test]
    fn a_nightly_phone_stays_on_nightlies_after_a_stable_release() {
        let first = on_start(Kept::default(), "2.4.3-nightly.20261117.97");
        assert!(first.nightly);
        let after_stable = on_start(first, "2.4.3");
        assert!(after_stable.nightly);
        assert!(!on_start(Kept::default(), "2.4.3").nightly);
    }

    #[test]
    fn a_picked_track_holds_whatever_build_runs() {
        let stable = Kept {
            nightly: false,
            picked: true,
            ..Kept::default()
        };
        assert!(!on_start(stable, "2.4.3-nightly.20261117.97").nightly);
    }

    #[test]
    fn a_damaged_answer_time_asks_at_once() {
        let kept = Kept {
            answered_at: u64::MAX,
            ..Kept::default()
        };
        assert_eq!(due_for(&kept), None);
    }

    #[test]
    fn a_small_clock_step_back_changes_nothing() {
        let now = SystemTime::now();
        let in_six_hours = Some((now + AGAIN_AFTER + Duration::from_secs(1), AGAIN_AFTER));
        assert!(!due(in_six_hours, now));
        // A check out a moment ago, with the clock stepped back a second, is still out
        assert!(out(Some(now + Duration::from_secs(1)), now));
    }

    #[test]
    fn a_launch_inside_six_hours_keeps_its_answer() {
        let offer = PhoneUpdate {
            version: "2.4.3-nightly.20261118.98".into(),
            url: "https://example.com/a.apk".into(),
        };
        let answered = Kept {
            nightly: true,
            picked: false,
            asked_by: "2.4.3-nightly.20261117.97".into(),
            answered_at: 1_800_000_000,
            offer: Some(offer.clone()),
        };
        let same = on_start(answered.clone(), "2.4.3-nightly.20261117.97");
        assert_eq!(same.offer, Some(offer));
        let (at, wait) = due_for(&same).unwrap();
        assert_eq!(wait, AGAIN_AFTER);
        assert_eq!(
            at,
            UNIX_EPOCH + Duration::from_secs(1_800_000_000) + AGAIN_AFTER
        );
        // The update was installed: what was kept is for another build, so it asks again
        let updated = on_start(answered, "2.4.3-nightly.20261118.98");
        assert_eq!(updated.offer, None);
        assert_eq!(due_for(&updated), None);
    }
}
