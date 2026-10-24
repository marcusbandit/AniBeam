//! A definition's `rate`: a gap between requests to its host, on top of everything the core
//! already does (`net::front`'s gate, the provider limiters). It can only slow requests
//! down. The slots are kept per host for the whole process, so two endpoints on one site and
//! two searches in a row share one budget.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::net::{BoxFuture, Http, HttpError, HttpRequest, HttpResponse};

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

    /// How long a request to `host` at `now` waits, and books its slot.
    fn book(&self, host: &str, now: Instant) -> Duration {
        let Some(gap) = self.gaps.get(host) else {
            return Duration::ZERO;
        };
        let mut next = NEXT.lock().unwrap_or_else(|e| e.into_inner());
        let start = next.get(host).copied().unwrap_or(now).max(now);
        next.insert(host.to_string(), start + *gap);
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
        assert_eq!(p.book("other.example", now), Duration::ZERO);
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
