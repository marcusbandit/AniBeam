//! The rules the download work follows, apart from the app so they can be tested: the hold
//! the core is asked for, how long nothing has moved, and what a list read does to the work.

use anibeam_core::Torrent;

/// How long nothing moves with the app away before the download work rests: 15 minutes.
pub const REST_TICKS: u32 = 15 * 60;

/// The hold the downloads wait on, as the app asks the core for it: one call at a time, so
/// they land in the order asked, and one the core refused asked again later and later.
#[derive(Default)]
pub struct Hold {
    /// The hold the core took last; None while one is on its way, or after a refusal.
    pub held: Option<bool>,
    /// The hold on its way to the core.
    asked: Option<bool>,
    retry: HoldRetry,
    /// Goes up with every hold sent, so a list read started before one does not count as
    /// showing it.
    epoch: u32,
}

impl Hold {
    /// Whether to send `want` at tick `now`: none on its way, no refusal still waiting, and
    /// the core does not hold that already.
    pub fn due(&self, want: bool, now: u32) -> bool {
        self.asked.is_none() && self.retry.due(now) && self.held != Some(want)
    }

    /// `want` went out. Unknown until the core took it, so no read taken meanwhile counts as
    /// showing it.
    pub fn sent(&mut self, want: bool) {
        self.held = None;
        self.asked = Some(want);
        self.epoch = self.epoch.wrapping_add(1);
    }

    /// The call never went out (its thread did not start): nothing changed, and it waits
    /// the way a refusal does. Returns the refusals before this one.
    pub fn not_sent(&mut self, now: u32) -> u32 {
        let tries = self.retry.tries;
        self.retry.refused(now);
        tries
    }

    /// The core answered the hold on its way, `took` it or not. Returns the refusals that
    /// came before.
    pub fn landed(&mut self, took: bool, now: u32) -> u32 {
        let tries = self.retry.tries;
        let asked = self.asked.take();
        if took {
            self.retry = HoldRetry::default();
            self.held = asked;
        } else {
            self.retry.refused(now);
        }
        tries
    }

    /// A refused hold waits to be asked again.
    pub fn waiting(&self) -> bool {
        self.retry.waiting()
    }

    /// What a list read is taken against: whether the core's hold is known, and which one.
    pub fn mark(&self) -> (bool, u32) {
        (self.held.is_some(), self.epoch)
    }

    /// Whether a read taken at `mark` shows the hold as it is now.
    pub fn shows(&self, mark: (bool, u32)) -> bool {
        mark.0 && mark.1 == self.epoch
    }
}

/// When a hold the core refused is asked again: two seconds after the first refusal,
/// doubling, at most a minute apart.
#[derive(Default)]
struct HoldRetry {
    /// Refusals in a row.
    tries: u32,
    /// The tick it may be asked again from.
    at: Option<u32>,
}

impl HoldRetry {
    const FIRST: u32 = 2;
    const MOST: u32 = 60;

    /// A refusal at tick `now`.
    fn refused(&mut self, now: u32) {
        self.tries = self.tries.saturating_add(1);
        let wait = Self::FIRST
            .saturating_mul(1 << (self.tries - 1).min(6))
            .min(Self::MOST);
        self.at = Some(now.wrapping_add(wait));
    }

    fn waiting(&self) -> bool {
        self.at.is_some()
    }

    /// Whether a hold may be asked at tick `now`.
    fn due(&self, now: u32) -> bool {
        // Ticks wrap: a wait not yet over is less than half the range ahead.
        self.at.is_none_or(|at| now.wrapping_sub(at) < u32::MAX / 2)
    }
}

/// How long nothing has moved with the app away. Only time the downloads could move counts:
/// a wait for Wi-Fi stops the count and does not end it.
#[derive(Default)]
pub struct Quiet {
    /// The tick of the last look, the ticks counted so far, and the bytes then.
    at: Option<(u32, u32, u64)>,
}

impl Quiet {
    /// A look at tick `now`, with `got` bytes downloaded across the unfinished downloads.
    /// `away`: the app is away from the screen. `could_move`: nothing but the peers keeps
    /// the downloads from moving, so the time since the last look counts. True once nothing
    /// moved for `REST_TICKS` of such time.
    pub fn look(&mut self, now: u32, got: u64, away: bool, could_move: bool) -> bool {
        match self.at {
            Some((last, still, before)) if away && before == got => {
                let still = if could_move {
                    still.saturating_add(now.wrapping_sub(last))
                } else {
                    still
                };
                self.at = Some((now, still, got));
                still >= REST_TICKS
            }
            _ => {
                self.at = Some((now, 0, got));
                false
            }
        }
    }

    pub fn reset(&mut self) {
        self.at = None;
    }
}

/// What a list read does to the download work.
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    /// Leave it as it is.
    Keep,
    /// Start it, showing the new words.
    Start,
    /// Change its notification in place.
    Update,
    /// End it: nothing is left to download.
    End,
    /// Nothing is up, and the words it showed go.
    Forget,
}

/// What a list read does to the work. `stopped`: Android, the person or the rest stopped it.
/// `unfinished`: a download is not whole yet. `shows`: there are words to show, something
/// working or waiting. `up`: it runs or is about to. `changed`: the words differ from the
/// ones up. `settled`: the read was taken after the hold landed and since the work last ran,
/// so it may end the work.
pub fn step(
    stopped: bool,
    unfinished: bool,
    shows: bool,
    up: bool,
    changed: bool,
    settled: bool,
) -> Step {
    // Stopped, the downloads are held and the work goes on as Android says; only the last
    // download going ends the stop, so its Paused does not outlive it.
    if stopped {
        return if !unfinished && settled {
            Step::End
        } else {
            Step::Keep
        };
    }
    match (shows, up) {
        // Running: the words change in place. Starting it again on every look would be
        // refused once the app is in the background.
        (true, true) if changed => Step::Update,
        (true, true) => Step::Keep,
        (true, false) => Step::Start,
        (false, true) if settled => Step::End,
        (false, true) => Step::Keep,
        (false, false) => Step::Forget,
    }
}

/// The hold is all that keeps `t` from running: unfinished and paused, and not by the
/// person. One in error, or one the person paused, stays as it is once the hold lifts, so
/// it does not keep the work up waiting for Wi-Fi.
pub fn held_back(t: &Torrent, paused_by_person: &[String]) -> bool {
    t.progress < 1.0 && t.state == "pausedDL" && !paused_by_person.contains(&t.hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refused_hold_is_asked_again_later_and_later() {
        let mut r = HoldRetry::default();
        assert!(r.due(0) && !r.waiting());
        r.refused(100);
        assert!(r.waiting());
        assert!(!r.due(100) && !r.due(101) && r.due(102));
        r.refused(102);
        assert!(!r.due(105) && r.due(106));
        r.refused(106);
        assert!(!r.due(113) && r.due(114));
        for _ in 0..20 {
            r.refused(1000);
        }
        // At most a minute apart, however many refusals.
        assert!(!r.due(1059) && r.due(1060));
        // Across the tick count wrapping.
        let mut r = HoldRetry::default();
        r.refused(u32::MAX);
        assert!(!r.due(u32::MAX) && !r.due(0) && r.due(1));
    }

    #[test]
    fn a_hold_whose_call_never_went_out_waits_like_a_refusal_and_blocks_nothing() {
        let mut h = Hold::default();
        assert!(h.due(true, 10));
        // The thread did not start: nothing was sent, so what the core holds stays known.
        assert_eq!(h.not_sent(10), 0);
        assert_eq!(h.held, None);
        assert!(h.waiting());
        assert!(!h.due(true, 11), "it waits out the backoff");
        assert!(h.due(true, 12), "then it is asked again");
        h.sent(true);
        assert!(!h.due(false, 13), "one on its way at a time");
        assert_eq!(h.landed(true, 13), 1);
        assert_eq!(h.held, Some(true));
        assert!(!h.waiting());
        assert!(!h.due(true, 14) && h.due(false, 14));
    }

    #[test]
    fn a_read_shows_the_hold_only_when_taken_after_it_landed() {
        let mut h = Hold::default();
        h.sent(true);
        let during = h.mark();
        h.landed(true, 1);
        let after = h.mark();
        assert!(!h.shows(during));
        assert!(h.shows(after));
        h.sent(false);
        assert!(!h.shows(after), "a newer hold went out since");
        h.landed(false, 2);
        assert_eq!(h.held, None, "refused: unknown until one lands");
        assert!(!h.shows(h.mark()));
    }

    #[test]
    fn a_wait_for_wifi_does_not_count_toward_the_rest() {
        let mut q = Quiet::default();
        assert!(!q.look(0, 500, true, true));
        // Ten minutes stalled with nothing else in the way.
        assert!(!q.look(600, 500, true, true));
        // An hour held off mobile data: not counted.
        assert!(!q.look(4200, 500, true, false));
        // Back on Wi-Fi and still stuck: the count goes on from ten minutes.
        assert!(!q.look(4400, 500, true, true));
        assert!(q.look(4500, 500, true, true));
    }

    #[test]
    fn moving_or_the_app_on_screen_starts_the_count_over() {
        let mut q = Quiet::default();
        q.look(0, 500, true, true);
        q.look(800, 500, true, true);
        assert!(!q.look(850, 600, true, true), "bytes moved");
        assert!(!q.look(1700, 600, true, true));
        assert!(!q.look(1740, 600, false, true), "the app came on screen");
        assert!(!q.look(2600, 600, true, true));
        assert!(q.look(2640, 600, true, true));
        q.reset();
        assert!(!q.look(5000, 600, true, true));
    }

    #[test]
    fn removing_the_last_download_ends_a_stop() {
        // Stopped with nothing left to download: the stop and its Paused end.
        assert_eq!(step(true, false, false, false, false, true), Step::End);
        // Not on a read from before the hold landed.
        assert_eq!(step(true, false, false, false, false, false), Step::Keep);
        // Something still waits: the stop stands.
        assert_eq!(step(true, true, false, false, false, true), Step::Keep);
    }

    #[test]
    fn a_read_moves_the_running_work_as_before() {
        assert_eq!(step(false, true, true, true, true, false), Step::Update);
        assert_eq!(step(false, true, true, true, false, true), Step::Keep);
        assert_eq!(step(false, true, true, false, false, false), Step::Start);
        assert_eq!(step(false, false, false, true, false, true), Step::End);
        assert_eq!(step(false, false, false, true, false, false), Step::Keep);
        assert_eq!(step(false, false, false, false, false, true), Step::Forget);
    }

    fn torrent(hash: &str, state: &str, progress: f64) -> Torrent {
        Torrent {
            hash: hash.into(),
            state: state.into(),
            progress,
            ..anibeam_core::downloads::qbt::tests_torrent()
        }
    }

    #[test]
    fn only_a_download_the_hold_paused_waits_for_wifi() {
        let by_person = vec!["b".to_string()];
        assert!(held_back(&torrent("a", "pausedDL", 0.4), &by_person));
        assert!(!held_back(&torrent("b", "pausedDL", 0.4), &by_person));
        assert!(!held_back(&torrent("c", "error", 0.4), &by_person));
        assert!(!held_back(&torrent("d", "pausedUP", 1.0), &by_person));
        assert!(!held_back(&torrent("e", "downloading", 0.4), &by_person));
    }
}
