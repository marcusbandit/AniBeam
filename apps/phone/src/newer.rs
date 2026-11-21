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
    /// When the last check failed, in seconds since 1970; 0 once one answers. The half
    /// hour's wait after it outlives a restart.
    failed_at: u64,
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
        failed_at: 0,
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
pub(crate) fn offered() -> bool {
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
        kept.failed_at = 0;
        kept.offer = None;
    }
    kept
}

/// When the next check is due for what was kept: six hours after its answer, or half an
/// hour after a failure since, whichever is later. A damaged file's time past what the
/// clock can hold counts as none.
fn due_for(kept: &Kept) -> Option<(SystemTime, Duration)> {
    let wait_from = |secs: u64, wait: Duration| {
        (secs > 0)
            .then(|| {
                UNIX_EPOCH
                    .checked_add(Duration::from_secs(secs))?
                    .checked_add(wait)
            })
            .flatten()
            .map(|at| (at, wait))
    };
    let answered = wait_from(kept.answered_at, AGAIN_AFTER);
    let failed = wait_from(kept.failed_at, RETRY_AFTER);
    match (answered, failed) {
        (Some(a), Some(f)) => Some(if f.0 > a.0 { f } else { a }),
        (a, f) => a.or(f),
    }
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
    let Some(number) = st.next(now) else {
        return;
    };
    let call = Call::CheckPhoneUpdate {
        current: RUNNING.to_string(),
        nightly: Some(st.kept.nightly),
    };
    // A panic in the call still answers, so the check is not left out for good
    if crate::local::call_off_thread("newer", call, move |answer| heard(number, answer)) {
        st.sent(number, now);
    }
}

/// `at` in seconds since 1970, 0 for a time before it.
fn secs(at: SystemTime) -> u64 {
    at.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// What covers More's About right now, as the app knows it.
pub(crate) struct View {
    pub foreground: bool,
    pub notes: bool,
    pub onboarding: bool,
    pub player: bool,
}

impl View {
    /// About is what the person sees: the window in front and nothing drawn over it.
    pub(crate) fn shows_about(&self) -> bool {
        self.foreground && !self.notes && !self.onboarding && !self.player
    }
}

/// What a pick did to the state.
#[derive(Debug, PartialEq)]
enum Pick {
    /// The track already picked: nothing to save.
    Same,
    /// The track already on, picked for the first time: saved, the answer kept.
    Kept,
    /// The other track: saved, the answer for the old one gone, asked again at once.
    Changed,
}

/// What an answer did to the state.
#[derive(Debug, PartialEq)]
enum Heard {
    /// A later check went out since, or the track changed: nothing changed.
    Stale,
    /// An offer, or none: show it, and save what is kept.
    Answered,
    /// No answer: try again in half an hour.
    Failed,
}

impl State {
    /// The number of the check to send now, when one is due and none is out.
    fn next(&self, now: SystemTime) -> Option<u64> {
        (!out(self.out_since, now) && due(self.due, now)).then_some(self.asked + 1)
    }

    /// Check `number` went out at `now`; an answer to any earlier one is stale from here.
    fn sent(&mut self, number: u64, now: SystemTime) {
        self.asked = number;
        self.out_since = Some(now);
    }

    /// The person picked a track.
    fn pick(&mut self, nightly: bool) -> Pick {
        if self.kept.nightly == nightly {
            // The track already on: the pick is kept, the answer for it stands
            if self.kept.picked {
                return Pick::Same;
            }
            self.kept.picked = true;
            return Pick::Kept;
        }
        self.kept.nightly = nightly;
        self.kept.picked = true;
        self.kept.answered_at = 0;
        self.kept.offer = None;
        self.due = None;
        // An answer still out is for the other track
        self.asked += 1;
        self.out_since = None;
        Pick::Changed
    }

    /// The answer to check `number`, at `now`.
    fn hear(
        &mut self,
        number: u64,
        answer: Result<Option<PhoneUpdate>, String>,
        now: SystemTime,
    ) -> Heard {
        if number != self.asked {
            return Heard::Stale;
        }
        self.out_since = None;
        match answer {
            Ok(update) => {
                self.due = Some((now + AGAIN_AFTER, AGAIN_AFTER));
                self.kept.answered_at = secs(now);
                self.kept.failed_at = 0;
                self.kept.offer = update;
                Heard::Answered
            }
            Err(_) => {
                self.due = Some((now + RETRY_AFTER, RETRY_AFTER));
                self.kept.failed_at = secs(now);
                Heard::Failed
            }
        }
    }
}

/// The person picked a track in About: kept, and asked again on it at once.
fn pick_nightly(nightly: bool) {
    {
        let mut st = state();
        let picked = st.pick(nightly);
        if picked == Pick::Same {
            return;
        }
        if let Some(file) = &st.file {
            save(file, &st.kept);
        }
        if picked == Pick::Kept {
            return;
        }
    }
    show(None);
}

/// On the UI thread: keeps the answer, then shows it.
fn heard(number: u64, answer: Result<Reply, CoreError>) {
    let answer = match answer {
        Ok(Reply::PhoneUpdate { update }) => Ok(update),
        Ok(other) => Err(format!("unexpected reply {other:?}")),
        Err(e) => Err(e.to_string()),
    };
    if let Err(e) = &answer {
        crate::local::log(&format!("update check: {e}"));
    }
    let offer = {
        let mut st = state();
        let heard = st.hear(number, answer, SystemTime::now());
        if heard == Heard::Stale {
            return;
        }
        // A failure is kept too, so its half hour holds across a restart
        if let Some(file) = &st.file {
            save(file, &st.kept);
        }
        if heard == Heard::Failed {
            return;
        }
        st.kept.offer.clone()
    };
    show(offer);
}

/// The window came to the front or went behind: About counts a version as seen only while
/// it is in front.
pub fn foreground(on: bool) {
    let Some(window) = WINDOW.with(|w| w.borrow().as_ref().and_then(|w| w.upgrade())) else {
        return;
    };
    window.global::<AppData>().set_foreground(on);
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

    fn fresh() -> State {
        State {
            file: None,
            kept: Kept::default(),
            due: None,
            out_since: None,
            asked: 0,
        }
    }

    impl State {
        /// Sends the next check at `now` and answers it with no offer.
        fn hear_again(&mut self, now: SystemTime) -> Heard {
            let number = self.asked + 1;
            self.sent(number, now);
            self.hear(number, Ok(None), now)
        }
    }

    fn offer(version: &str) -> Option<PhoneUpdate> {
        Some(PhoneUpdate {
            version: version.into(),
            url: "https://example.com/a.apk".into(),
        })
    }

    #[test]
    fn an_answer_to_an_older_check_is_dropped_and_the_latest_kept() {
        let now = SystemTime::now();
        let mut st = fresh();
        let first = st.next(now).unwrap();
        st.sent(first, now);
        assert_eq!(st.next(now), None, "one check out at a time");
        // Out too long: a second goes, and the first answers after it
        let later = now + OUT_AT_MOST * 2;
        let second = st.next(later).unwrap();
        st.sent(second, later);
        assert_eq!(st.hear(second, Ok(offer("2.4.4")), later), Heard::Answered);
        assert_eq!(st.hear(first, Err("late".into()), later), Heard::Stale);
        assert_eq!(st.kept.offer, offer("2.4.4"));
        assert_eq!(st.due.map(|(_, wait)| wait), Some(AGAIN_AFTER));
    }

    #[test]
    fn picking_a_track_drops_the_answer_for_the_other_one() {
        let now = SystemTime::now();
        let mut st = fresh();
        let number = st.next(now).unwrap();
        st.sent(number, now);
        assert_eq!(st.pick(true), Pick::Changed);
        assert_eq!(
            st.pick(true),
            Pick::Same,
            "the same pick again changes nothing"
        );
        assert_eq!(st.hear(number, Ok(offer("2.4.4")), now), Heard::Stale);
        assert_eq!(st.kept.offer, None);
        // And the track is asked about at once
        let again = st.next(now).unwrap();
        st.sent(again, now);
        assert_eq!(st.hear(again, Err("offline".into()), now), Heard::Failed);
        assert_eq!(st.due.map(|(_, wait)| wait), Some(RETRY_AFTER));
    }

    #[test]
    fn picking_the_track_already_on_keeps_its_answer() {
        let now = SystemTime::now();
        let mut st = fresh();
        st.kept.nightly = true;
        st.kept.answered_at = 1_800_000_000;
        st.kept.offer = offer("2.4.4-nightly.20261201.120");
        st.due = Some((now + AGAIN_AFTER, AGAIN_AFTER));
        let asked = st.asked;
        assert_eq!(st.pick(true), Pick::Kept);
        assert!(st.kept.picked);
        assert_eq!(st.kept.offer, offer("2.4.4-nightly.20261201.120"));
        assert_eq!(st.kept.answered_at, 1_800_000_000);
        assert_eq!(st.due, Some((now + AGAIN_AFTER, AGAIN_AFTER)));
        assert_eq!(st.asked, asked);
        assert_eq!(st.next(now), None, "nothing is asked again");
    }

    #[test]
    fn a_failed_check_waits_its_half_hour_across_a_restart() {
        let answered = 1_800_000_000;
        let failed = answered + 7 * 60 * 60;
        let kept = Kept {
            asked_by: "2.4.3-nightly.20261117.97".into(),
            answered_at: answered,
            failed_at: failed,
            ..Kept::default()
        };
        // The answer's six hours are past; the failure after it holds for half an hour
        let restarted = on_start(kept.clone(), "2.4.3-nightly.20261117.97");
        let at_failure = UNIX_EPOCH + Duration::from_secs(failed);
        assert_eq!(
            due_for(&restarted),
            Some((at_failure + RETRY_AFTER, RETRY_AFTER))
        );
        assert!(!due(
            due_for(&restarted),
            at_failure + Duration::from_secs(60)
        ));
        assert!(due(due_for(&restarted), at_failure + RETRY_AFTER));
        // A failure inside the answer's six hours does not shorten them
        let early = Kept {
            failed_at: answered + 60,
            ..kept.clone()
        };
        let six_hours = UNIX_EPOCH + Duration::from_secs(answered) + AGAIN_AFTER;
        assert_eq!(due_for(&early), Some((six_hours, AGAIN_AFTER)));
        // And a failure is what the state keeps when a check fails
        let now = SystemTime::now();
        let mut st = fresh();
        let number = st.next(now).unwrap();
        st.sent(number, now);
        assert_eq!(st.hear(number, Err("refused".into()), now), Heard::Failed);
        assert_eq!(st.kept.failed_at, secs(now));
        assert_eq!(st.hear_again(now), Heard::Answered);
        assert_eq!(st.kept.failed_at, 0);
    }

    #[test]
    fn about_counts_as_seen_only_when_nothing_covers_it() {
        let shown = View {
            foreground: true,
            notes: false,
            onboarding: false,
            player: false,
        };
        assert!(shown.shows_about());
        assert!(
            !View {
                foreground: false,
                ..shown
            }
            .shows_about()
        );
        assert!(
            !View {
                notes: true,
                ..shown
            }
            .shows_about()
        );
        assert!(
            !View {
                onboarding: true,
                ..shown
            }
            .shows_about()
        );
        assert!(
            !View {
                player: true,
                ..shown
            }
            .shows_about()
        );
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
            failed_at: 0,
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
