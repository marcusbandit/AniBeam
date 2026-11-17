//! The limits both relays share, pure so the Worker and the native relay apply
//! the same numbers: messages a second per connection, how long a socket may
//! go without saying hello, and the address a per address limit counts by.

use std::net::{IpAddr, Ipv6Addr};

use crate::proto::{ErrorCode, ServerMsg};

/// The most messages one connection, or one member, may send in a second.
pub const RATE_PER_SEC: u32 = 20;

/// How long a socket may stay open without saying hello, in ms. Past it the
/// relay closes the socket.
pub const HELLO_MS: u64 = 10_000;

/// What to do with one message under the rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Within the rate: handle it.
    Pass,
    /// The first one over: drop it and answer [`refusal`].
    Refuse,
    /// Further over: drop it without a word.
    Drop,
}

/// Counts messages in one second windows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Window {
    start_ms: u64,
    count: u32,
}

impl Window {
    pub fn new(now_ms: u64) -> Window {
        Window {
            start_ms: now_ms,
            count: 0,
        }
    }

    /// Counts one message sent at `now_ms`.
    pub fn hit(&mut self, now_ms: u64) -> Verdict {
        count(&mut self.start_ms, &mut self.count, now_ms)
    }
}

/// Counts one message against a window kept as two numbers, for a caller that
/// stores them itself (the room's members, whose layout is persisted).
pub(crate) fn count(start_ms: &mut u64, count: &mut u32, now_ms: u64) -> Verdict {
    if now_ms >= *start_ms + 1_000 || now_ms < *start_ms {
        *start_ms = now_ms;
        *count = 0;
    }
    *count += 1;
    if *count <= RATE_PER_SEC {
        Verdict::Pass
    } else if *count == RATE_PER_SEC + 1 {
        Verdict::Refuse
    } else {
        Verdict::Drop
    }
}

/// The answer to the first message over the rate. Not fatal.
pub fn refusal() -> ServerMsg {
    ServerMsg::Error {
        code: ErrorCode::RateLimited,
        message: format!("at most {RATE_PER_SEC} messages a second"),
    }
}

/// The answer to a socket that never said hello, just before it is closed.
pub fn no_hello() -> ServerMsg {
    ServerMsg::Error {
        code: ErrorCode::Proto,
        message: "hello must come first".into(),
    }
}

/// The address a per address limit counts: an IPv6 address by its /64, which
/// one machine usually has whole, and an IPv4 mapped one as plain IPv4.
pub fn limit_key(ip: IpAddr) -> IpAddr {
    match ip.to_canonical() {
        IpAddr::V6(v6) => {
            let s = v6.segments();
            IpAddr::V6(Ipv6Addr::new(s[0], s[1], s[2], s[3], 0, 0, 0, 0))
        }
        v4 => v4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twenty_a_second_then_one_refusal_then_silence() {
        let mut w = Window::new(1_000);
        for i in 0..RATE_PER_SEC {
            assert_eq!(w.hit(1_000 + u64::from(i)), Verdict::Pass, "message {i}");
        }
        assert_eq!(w.hit(1_500), Verdict::Refuse);
        assert_eq!(w.hit(1_999), Verdict::Drop);
        assert_eq!(w.hit(2_000), Verdict::Pass, "a new second");
    }

    #[test]
    fn a_clock_that_goes_back_starts_a_new_window() {
        let mut w = Window::new(5_000);
        for _ in 0..=RATE_PER_SEC {
            w.hit(5_000);
        }
        assert_eq!(w.hit(4_000), Verdict::Pass);
    }

    #[test]
    fn an_ipv6_address_counts_by_its_64() {
        let a: IpAddr = "2001:db8:1:2:aaaa::1".parse().unwrap();
        let b: IpAddr = "2001:db8:1:2:bbbb::9".parse().unwrap();
        let c: IpAddr = "2001:db8:1:3::1".parse().unwrap();
        assert_eq!(limit_key(a), limit_key(b));
        assert_ne!(limit_key(a), limit_key(c));
        let mapped: IpAddr = "::ffff:198.51.100.7".parse().unwrap();
        assert_eq!(limit_key(mapped), "198.51.100.7".parse::<IpAddr>().unwrap());
    }
}
