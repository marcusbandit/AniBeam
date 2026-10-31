//! The pace of every endpoint fetch: a gap between requests to one host, the larger of the
//! core's `FLOOR` and the gap a definition's `rate` asks for, on top of everything the core
//! already does (`net::front`'s gate). It can only slow requests down. The slots are kept per
//! host for the whole process, so two endpoints on one site, two searches in a row and a feed
//! read again after its search share one budget.
//!
//! Endpoints are not in `net::limiter`: that budgets a fixed set of metadata providers known
//! at build time, where an endpoint is any host a person's repo or their own hand supplies.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::net::{BoxFuture, Http, HttpError, HttpRequest, HttpResponse};

/// The least gap between two requests to one endpoint host, whatever its definition says
/// or when it has none: `net::front`'s gap, which keeps DDoS-Guard answering (#135).
pub const FLOOR: Duration = crate::net::front::GAP;

/// Host to the earliest moment its next request may start.
static NEXT: LazyLock<Mutex<HashMap<String, Instant>>> = LazyLock::new(Default::default);

pub struct Paced {
    inner: Arc<dyn Http>,
    gaps: HashMap<String, Duration>,
}

impl Paced {
    pub fn new(inner: Arc<dyn Http>, gaps: Vec<(String, Duration)>) -> Paced {
        let mut map: HashMap<String, Duration> = HashMap::new();
        for (host, gap) in gaps {
            // Two definitions on one host: the slower one sets the pace.
            let slot = map.entry(host).or_default();
            *slot = (*slot).max(gap);
        }
        Paced { inner, gaps: map }
    }

    /// A request to `url`'s host that went out another way (anirss's own search) takes its
    /// slot here too, so the next one through this client waits its gap after it.
    pub fn passed(&self, url: &str) {
        self.book(&super::index::host_of(url), Instant::now());
    }

    /// How long a request to `host` at `now` waits, and books its slot.
    fn book(&self, host: &str, now: Instant) -> Duration {
        let gap = self.gaps.get(host).copied().unwrap_or_default().max(FLOOR);
        let mut next = NEXT.lock().unwrap_or_else(|e| e.into_inner());
        let start = next.get(host).copied().unwrap_or(now).max(now);
        next.insert(host.to_string(), start + gap);
        start - now
    }
}

impl Http for Paced {
    fn send(&self, req: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, HttpError>> {
        Box::pin(async move {
            let wait = self.book(&super::index::host_of(&req.url), Instant::now());
            if !wait.is_zero() {
                tokio::time::sleep(wait).await;
            }
            self.inner.send(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;

    #[test]
    fn a_paced_host_books_one_slot_after_another() {
        let host = "pace-test-a.example";
        let p = Paced::new(FakeHttp::new(), vec![(host.into(), Duration::from_secs(2))]);
        let now = Instant::now();
        assert_eq!(p.book(host, now), Duration::ZERO);
        assert_eq!(p.book(host, now), Duration::from_secs(2));
        assert_eq!(p.book(host, now), Duration::from_secs(4));
        assert_eq!(p.book("pace-test-other.example", now), Duration::ZERO);
    }

    #[test]
    fn a_host_with_no_rate_still_waits_the_floor() {
        let host = "pace-test-c.example";
        let p = Paced::new(FakeHttp::new(), Vec::new());
        let now = Instant::now();
        assert_eq!(p.book(host, now), Duration::ZERO);
        assert_eq!(p.book(host, now), FLOOR);
        assert_eq!(p.book(host, now + FLOOR * 3), Duration::ZERO);
    }

    #[test]
    fn a_rate_faster_than_the_floor_is_held_to_it_and_a_slower_one_wins() {
        let (fast, slow) = ("pace-test-d.example", "pace-test-e.example");
        let p = Paced::new(
            FakeHttp::new(),
            vec![(fast.into(), FLOOR / 4), (slow.into(), FLOOR * 6)],
        );
        let now = Instant::now();
        p.book(fast, now);
        assert_eq!(p.book(fast, now), FLOOR);
        p.book(slow, now);
        assert_eq!(p.book(slow, now), FLOOR * 6);
    }

    #[test]
    fn a_request_that_went_out_another_way_holds_back_the_next() {
        let url = "https://pace-test-f.example/?page=rss&q=x";
        let p = Paced::new(FakeHttp::new(), Vec::new());
        p.passed(url);
        let wait = p.book("pace-test-f.example", Instant::now());
        assert!(wait > Duration::ZERO && wait <= FLOOR, "{wait:?}");
    }

    #[test]
    fn the_slower_of_two_rates_on_one_host_wins() {
        let host = "pace-test-b.example";
        let p = Paced::new(
            FakeHttp::new(),
            vec![
                (host.into(), Duration::from_secs(1)),
                (host.into(), Duration::from_secs(3)),
            ],
        );
        let now = Instant::now();
        p.book(host, now);
        assert_eq!(p.book(host, now), Duration::from_secs(3));
    }
}
