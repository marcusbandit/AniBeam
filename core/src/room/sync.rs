//! The rules that keep a member's player with the room, pure over numbers so
//! they test without a relay: the relay's clock, where the room is now, when a
//! new play state seeks, when a tick nudges the speed or seeks, and the offset
//! two openings suggest.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use anibeam_room::proto::PlayState;

/// How many clock samples are kept; the one with the shortest round trip wins.
/// Five pings on connect fill it, and the one a minute after that replaces the
/// oldest, so the estimate follows a clock that drifts.
pub const CLOCK_SAMPLES: usize = 5;

/// A new play state seeks the player when it is further off than this, in seconds.
pub const STATE_SLACK: f64 = 0.5;

/// A tick under this far off, in seconds, needs nothing.
pub const TICK_SLACK: f64 = 0.25;

/// Once a nudged player is under this far off, it plays at normal speed again.
pub const SETTLED: f64 = 0.1;

/// Past this far off, a tick seeks rather than nudging the speed.
pub const TICK_SEEK: f64 = 2.0;

/// The speeds a player catches up or falls back at.
pub const FASTER: f64 = 1.05;
pub const SLOWER: f64 = 0.95;

/// Two openings further apart than this, in seconds, are not the same cut of the
/// episode, so they suggest nothing.
pub const SUGGEST_MAX: f64 = 30.0;

/// How far past its last tick a playing player is taken to be, at most: ticks come
/// once a second, so anything longer is a player that stopped saying.
const TICK_REACH: f64 = 2.0;

/// How far the relay's clock is from this machine's, from pings.
#[derive(Clone, Debug, Default)]
pub struct Clock {
    /// Round trip and offset, both in ms, newest last.
    samples: VecDeque<(u64, i64)>,
    /// From the welcome, before any ping came back: its round trip is unknown.
    rough: Option<i64>,
}

impl Clock {
    /// The relay's clock in the welcome, read at `local_ms`. Used only until a
    /// ping answers.
    pub fn rough(&mut self, server_ms: u64, local_ms: u64) {
        self.rough = Some(server_ms as i64 - local_ms as i64);
    }

    /// A pong: the ping left at `c`, the relay read `s`, and it came back at `now`,
    /// all in ms. The relay is taken to have read its clock halfway through.
    pub fn pong(&mut self, c: u64, s: u64, now: u64) {
        let Some(rtt) = now.checked_sub(c) else {
            return;
        };
        let offset = s as i64 - (c + rtt / 2) as i64;
        if self.samples.len() == CLOCK_SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back((rtt, offset));
    }

    /// Relay ms minus local ms: the sample with the shortest round trip, else the
    /// welcome's, else nothing.
    pub fn offset(&self) -> i64 {
        self.samples
            .iter()
            .min_by_key(|(rtt, _)| *rtt)
            .map(|(_, offset)| *offset)
            .or(self.rough)
            .unwrap_or(0)
    }

    /// The relay's clock now, from this machine's.
    pub fn server_now(&self, local_ms: u64) -> u64 {
        (local_ms as i64 + self.offset()).max(0) as u64
    }
}

/// Where the room is on the canonical timeline at `server_ms`: it moves on from
/// the state's position only while it plays.
pub fn canonical(state: &PlayState, server_ms: u64) -> f64 {
    if !state.playing {
        return state.position;
    }
    let elapsed = (server_ms as i64 - state.at_ms as i64) as f64 / 1000.0;
    (state.position + elapsed * state.rate).max(0.0)
}

/// The player's last tick: where it was, whether it was paused, and when.
#[derive(Clone, Copy, Debug)]
pub struct LocalTick {
    pub position: f64,
    pub paused: bool,
    pub at: Instant,
}

impl LocalTick {
    /// Where the player is at `now`, reckoned from the tick.
    pub fn now(&self, now: Instant) -> f64 {
        if self.paused {
            return self.position;
        }
        let since = now.saturating_duration_since(self.at).as_secs_f64();
        self.position + since.min(TICK_REACH)
    }
}

/// What the room has done to one player so far.
#[derive(Clone, Copy, Debug)]
pub struct Drift {
    /// The speed the player was last told to play at.
    pub speed: f64,
    /// When the player was last told anything.
    pub last: Option<Instant>,
    pub tick: Option<LocalTick>,
}

impl Default for Drift {
    fn default() -> Self {
        Drift {
            speed: 1.0,
            last: None,
            tick: None,
        }
    }
}

impl Drift {
    /// Whether `gap` has passed since the last thing the player was told.
    pub fn may_correct(&self, now: Instant, gap: Duration) -> bool {
        self.last
            .is_none_or(|last| now.saturating_duration_since(last) >= gap)
    }
}

/// On a new play state: where to seek, if anywhere. A player that has not ticked
/// yet is somewhere unknown, so it seeks.
pub fn on_state(local: Option<f64>, target: f64) -> Option<f64> {
    match local {
        Some(local) if (local - target).abs() <= STATE_SLACK => None,
        _ => Some(target),
    }
}

/// One correction for a player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Correction {
    pub seek_to: Option<f64>,
    pub speed: f64,
}

/// On a tick: what to change, given where the player is, where the room is, the
/// speed it plays at and whether both play. Under `TICK_SLACK` nothing (and the
/// speed goes back to normal under `SETTLED`); up to `TICK_SEEK` a nudge of the
/// speed; past it a seek. A paused room only ever seeks, and only past
/// `STATE_SLACK`, since speed moves nothing while paused.
pub fn on_tick(local: f64, target: f64, speed: f64, playing: bool) -> Option<Correction> {
    let off = target - local;
    let far = off.abs();
    if !playing {
        return (far > STATE_SLACK).then_some(Correction {
            seek_to: Some(target),
            speed: 1.0,
        });
    }
    if far > TICK_SEEK {
        return Some(Correction {
            seek_to: Some(target),
            speed: 1.0,
        });
    }
    let want = if far >= TICK_SLACK {
        if off > 0.0 { FASTER } else { SLOWER }
    } else if far < SETTLED {
        1.0
    } else {
        // Between settled and slack: carry on at whatever speed it plays at.
        speed
    };
    (want != speed).then_some(Correction {
        seek_to: None,
        speed: want,
    })
}

/// The offset two openings suggest: this file's opening less the host's, when
/// both are known and under `SUGGEST_MAX` apart.
pub fn suggest_offset(host: Option<f64>, mine: Option<f64>) -> Option<f64> {
    let (host, mine) = (host?, mine?);
    let diff = mine - host;
    (diff.is_finite() && diff.abs() < SUGGEST_MAX).then_some((diff * 1000.0).round() / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(playing: bool, position: f64, at_ms: u64) -> PlayState {
        PlayState {
            playing,
            position,
            rate: 1.0,
            at_ms,
            rev: 1,
        }
    }

    #[test]
    fn the_clock_keeps_the_shortest_round_trip() {
        let mut clock = Clock::default();
        assert_eq!(clock.offset(), 0);
        clock.rough(10_500, 10_000);
        assert_eq!(clock.offset(), 500, "the welcome stands until a pong");
        // 100 ms there and back, the relay read 1_000 ahead of the midpoint.
        clock.pong(1_000, 2_050, 1_100);
        assert_eq!(clock.offset(), 1_000);
        // A slower trip, read wrong by its asymmetry, loses to the faster one.
        clock.pong(2_000, 3_400, 2_400);
        assert_eq!(clock.offset(), 1_000);
        // A faster one wins.
        clock.pong(3_000, 4_005, 3_010);
        assert_eq!(clock.offset(), 1_000);
        clock.pong(4_000, 5_003, 4_002);
        assert_eq!(clock.offset(), 1_002);
        assert_eq!(clock.server_now(10_000), 11_002);
        // A pong from before its ping says nothing.
        clock.pong(9_000, 1, 8_000);
        assert_eq!(clock.offset(), 1_002);
    }

    #[test]
    fn the_clock_forgets_its_oldest_samples() {
        let mut clock = Clock::default();
        clock.pong(0, 100, 0);
        for i in 1..=CLOCK_SAMPLES as u64 {
            clock.pong(i * 1000, i * 1000 + 300, i * 1000 + 50);
        }
        // The zero round trip fell out of the window.
        assert_eq!(clock.offset(), 275);
    }

    #[test]
    fn the_room_moves_only_while_it_plays() {
        assert_eq!(canonical(&state(false, 42.0, 1_000), 9_000), 42.0);
        assert_eq!(canonical(&state(true, 42.0, 1_000), 3_500), 44.5);
        let mut fast = state(true, 10.0, 0);
        fast.rate = 2.0;
        assert_eq!(canonical(&fast, 1_000), 12.0);
        // A clock a little behind the state's never reads before nought.
        assert_eq!(canonical(&state(true, 0.1, 5_000), 4_000), 0.0);
    }

    #[test]
    fn a_tick_reckons_on_while_playing() {
        let at = Instant::now();
        let tick = LocalTick {
            position: 10.0,
            paused: false,
            at,
        };
        assert_eq!(tick.now(at + Duration::from_millis(500)), 10.5);
        assert_eq!(tick.now(at + Duration::from_secs(9)), 12.0);
        let paused = LocalTick {
            paused: true,
            ..tick
        };
        assert_eq!(paused.now(at + Duration::from_secs(9)), 10.0);
    }

    #[test]
    fn a_new_state_seeks_past_half_a_second() {
        assert_eq!(on_state(None, 12.0), Some(12.0));
        assert_eq!(on_state(Some(11.6), 12.0), None);
        assert_eq!(on_state(Some(12.4), 12.0), None);
        assert_eq!(on_state(Some(11.4), 12.0), Some(12.0));
        assert_eq!(on_state(Some(13.0), 12.0), Some(12.0));
    }

    #[test]
    fn ticks_nudge_then_seek() {
        // Close enough: nothing.
        assert_eq!(on_tick(10.0, 10.2, 1.0, true), None);
        // Behind: faster. Ahead: slower.
        assert_eq!(
            on_tick(10.0, 10.5, 1.0, true),
            Some(Correction {
                seek_to: None,
                speed: FASTER
            })
        );
        assert_eq!(
            on_tick(11.0, 10.0, 1.0, true),
            Some(Correction {
                seek_to: None,
                speed: SLOWER
            })
        );
        // Already at that speed: nothing new to say.
        assert_eq!(on_tick(10.0, 10.5, FASTER, true), None);
        // Caught up part way: keep going until settled.
        assert_eq!(on_tick(10.0, 10.15, FASTER, true), None);
        assert_eq!(
            on_tick(10.0, 10.05, FASTER, true),
            Some(Correction {
                seek_to: None,
                speed: 1.0
            })
        );
        // Far off: seek, at normal speed.
        assert_eq!(
            on_tick(10.0, 12.5, FASTER, true),
            Some(Correction {
                seek_to: Some(12.5),
                speed: 1.0
            })
        );
        assert_eq!(
            on_tick(15.0, 12.5, 1.0, true),
            Some(Correction {
                seek_to: Some(12.5),
                speed: 1.0
            })
        );
    }

    #[test]
    fn a_paused_room_only_seeks() {
        assert_eq!(on_tick(10.0, 10.4, 1.0, false), None);
        assert_eq!(
            on_tick(10.0, 11.0, 1.0, false),
            Some(Correction {
                seek_to: Some(11.0),
                speed: 1.0
            })
        );
    }

    #[test]
    fn corrections_wait_their_gap() {
        let now = Instant::now();
        let mut drift = Drift::default();
        assert!(drift.may_correct(now, Duration::from_secs(1)));
        drift.last = Some(now);
        assert!(!drift.may_correct(now + Duration::from_millis(900), Duration::from_secs(1)));
        assert!(drift.may_correct(now + Duration::from_secs(1), Duration::from_secs(1)));
    }

    #[test]
    fn openings_suggest_an_offset_when_close() {
        assert_eq!(suggest_offset(Some(90.0), Some(95.5)), Some(5.5));
        assert_eq!(suggest_offset(Some(95.0), Some(90.0)), Some(-5.0));
        assert_eq!(suggest_offset(Some(90.0), Some(90.0)), Some(0.0));
        assert_eq!(suggest_offset(Some(10.0), Some(45.0)), None);
        assert_eq!(suggest_offset(None, Some(45.0)), None);
        assert_eq!(suggest_offset(Some(10.0), None), None);
    }
}
