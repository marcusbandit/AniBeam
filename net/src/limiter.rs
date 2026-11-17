//! One limiter per upstream: a pace that spaces every request by the
//! provider's published gap, or wider when the provider's replies report a
//! lower live limit (AniList's 30 a minute while degraded, #732), and
//! backon's schedule on top of it for a 429. The pace keeps a run under the
//! limit on its own, the 429 schedule is the ceiling for when it does not,
//! and nothing else in the core ever calls a provider directly.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use backon::{ExponentialBuilder, Retryable};
use tokio::time::Instant;

use super::{Http, HttpRequest, HttpResponse, Upstream};
use anibeam_contract::CoreError;

/// The retry schedule the spec fixes: 1, 2, 4, 8, 16, 32 seconds, capped
/// at 60, six retries after the first request.
const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);
const DEFAULT_MAX_ATTEMPTS: usize = 6;

/// How far under a reported limit the live pace sits: 10% more time per
/// request than the limit allows, so a run never sits right on the line.
const LIVE_HEADROOM: f64 = 1.1;

/// The pacing one client shares between every caller. Every request waits
/// for the gap after the one before it, and the gap is the larger of the
/// published floor and what the provider's own replies say it allows now.
struct Pace {
    /// The published gap, the floor that holds whatever the replies say.
    floor: Duration,
    /// The window an upstream's `X-RateLimit-Limit` counts over, for an
    /// upstream that reports one; `None` ignores the headers.
    window: Option<Duration>,
    state: Mutex<PaceState>,
}

#[derive(Default)]
struct PaceState {
    /// When the last request went out.
    last_sent: Option<Instant>,
    /// The gap the last reported limit asks for, nought until a reply says.
    live_gap: Duration,
    /// When the request whose reply set `live_gap` went out. A reply to an
    /// older request that lands later is stale and never undoes it.
    live_from: Option<Instant>,
    /// No request goes out before this: the provider said none are left.
    hold_until: Option<Instant>,
}

impl Pace {
    fn new(floor: Duration) -> Pace {
        Pace {
            floor,
            window: None,
            state: Mutex::new(PaceState::default()),
        }
    }

    /// Waits for this caller's turn, takes it and says when it was. The
    /// turn is worked out again on every wake, so a reply that lowered the
    /// limit or emptied the budget while this caller slept still holds it
    /// back.
    async fn turn(&self) -> Instant {
        loop {
            let when = {
                let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
                let now = Instant::now();
                let gap = self.floor.max(st.live_gap);
                let mut when = st.last_sent.map_or(now, |t| t + gap);
                if let Some(hold) = st.hold_until {
                    when = when.max(hold);
                }
                if when <= now {
                    st.last_sent = Some(now);
                    return now;
                }
                when
            };
            tokio::time::sleep_until(when).await;
        }
    }

    /// Reads the live limit off the reply to a request sent at `sent_at`.
    /// `X-RateLimit-Limit` sets the pace to the window over that limit, with
    /// headroom, unless a newer request's reply already spoke.
    /// `X-RateLimit-Remaining` at nought holds every caller for the reply's
    /// `Retry-After`, else until its `X-RateLimit-Reset`, else a whole
    /// window, never longer than the window. A header that does not parse
    /// changes nothing.
    fn observe(&self, sent_at: Instant, r: &HttpResponse) {
        let Some(window) = self.window else {
            return;
        };
        let limit = header_u64(r, "x-ratelimit-limit").filter(|&n| n > 0);
        let remaining = header_u64(r, "x-ratelimit-remaining");
        if limit.is_none() && remaining.is_none() {
            return;
        }
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(limit) = limit
            && st.live_from.is_none_or(|t| sent_at >= t)
        {
            st.live_gap = window.mul_f64(LIVE_HEADROOM) / u32::try_from(limit).unwrap_or(u32::MAX);
            st.live_from = Some(sent_at);
        }
        if remaining == Some(0) {
            // Retry-After is a length, so no clock skew can shorten it; a
            // reset that reads as already past is skew too, and a skewed
            // reset must never send the next request straight into a 429.
            let wait = retry_after(r)
                .map(|s| Duration::from_secs_f64(s.max(0.0)))
                .or_else(|| {
                    header_u64(r, "x-ratelimit-reset")
                        .and_then(|reset| reset_in(reset, anibeam_foundation::time::now_secs()))
                })
                .unwrap_or(window)
                .min(window);
            let hold = Instant::now() + wait;
            st.hold_until = Some(st.hold_until.map_or(hold, |h| h.max(hold)));
        }
    }
}

fn header_u64(r: &HttpResponse, name: &str) -> Option<u64> {
    r.header(name).and_then(|s| s.trim().parse::<u64>().ok())
}

/// How long until an `X-RateLimit-Reset` comes round. AniList sends a Unix
/// time; a value too small to be one is read as seconds from now, which is
/// what some providers send instead. A reset that is not ahead says nothing
/// about when the budget comes back, so it is `None`.
fn reset_in(reset: u64, now_unix: i64) -> Option<Duration> {
    const A_UNIX_TIME: u64 = 1_000_000_000;
    let secs = if reset >= A_UNIX_TIME {
        u64::try_from(i64::try_from(reset).ok()?.checked_sub(now_unix)?).ok()?
    } else {
        reset
    };
    (secs > 0).then(|| Duration::from_secs(secs))
}

pub struct ProviderClient {
    upstream: Upstream,
    http: Arc<dyn Http>,
    pace: Pace,
    min_delay: Duration,
    max_attempts: usize,
    /// A cap on one request, never on the whole call. A timeout around the
    /// retries would swallow the 429 schedule and report a rate limit as a
    /// timeout, which is the one thing a caller must not be told.
    attempt_timeout: Option<Duration>,
}

/// A 429 by status, or AniList's 429 inside a 200 body. AniList answers a
/// rate limit with HTTP 200 and the code in `errors[].status`, so a client
/// that only reads the status never backs off.
fn is_rate_limited(r: &HttpResponse) -> bool {
    if r.status == 429 {
        return true;
    }
    if r.status == 200
        && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&r.body)
    {
        return value["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|e| e["status"].as_u64() == Some(429)));
    }
    false
}

/// The provider's own wait in seconds. A header that does not parse is
/// ignored rather than guessed at.
fn retry_after(r: &HttpResponse) -> Option<f64> {
    r.header("retry-after")
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|s| s.is_finite())
}

/// How long to wait before the next attempt: the provider's own
/// `Retry-After` when it sent one readable, clamped to the schedule's
/// ceiling so a header of an hour cannot park a job for an hour, and the
/// schedule's planned wait when it sent none.
fn wait_before_retry(asked: Option<f64>, planned: Duration) -> Duration {
    match asked {
        Some(seconds) => Duration::from_secs_f64(seconds.clamp(0.0, MAX_RETRY_DELAY.as_secs_f64())),
        None => planned,
    }
}

/// What one attempt can go wrong with. `Limited` is retried, `Transport`
/// is not: a connection that never opened will not open on the next tick
/// either, and the job above decides whether to try again later.
#[derive(Debug)]
enum Attempt {
    Limited(Box<HttpResponse>),
    Transport(String),
}

impl ProviderClient {
    pub fn new(upstream: Upstream, http: Arc<dyn Http>, gap: Duration) -> ProviderClient {
        // A zero gap would be no pacing at all, which no provider wants;
        // one per second is the safe reading of a caller's mistake.
        let gap = if gap.is_zero() {
            Duration::from_secs(1)
        } else {
            gap
        };
        ProviderClient {
            upstream,
            http,
            pace: Pace::new(gap),
            min_delay: Duration::from_secs(1),
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            attempt_timeout: None,
        }
    }

    /// Caps every request this client makes, retries included, at `d`. For
    /// a client whose calls all want the same cap; a client shared between
    /// callers that want different ones uses `send_within` instead.
    pub fn with_attempt_timeout(mut self, d: Duration) -> Self {
        self.attempt_timeout = Some(d);
        self
    }

    /// Tests shrink the backoff; production keeps the one second floor.
    #[doc(hidden)]
    pub fn with_min_delay(mut self, delay: Duration) -> Self {
        self.min_delay = delay;
        self
    }

    /// How many retries follow the first request, so `n` here means at most
    /// `n + 1` requests in total. Six in production, which is the spec's
    /// 1, 2, 4, 8, 16, 32 schedule; nought in a test that wants a 429 to
    /// exhaust in a single request.
    #[doc(hidden)]
    pub fn with_max_attempts(mut self, attempts: usize) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Paces by the limit the provider's replies report as well as by the
    /// gap: `X-RateLimit-Limit` requests in every `window`, and none at all
    /// once `X-RateLimit-Remaining` reaches nought. For an upstream whose
    /// limit moves, as AniList's drops to 30 a minute while it is degraded.
    pub fn with_live_limit(mut self, window: Duration) -> Self {
        self.pace.window = Some(window);
        self
    }

    pub async fn send(&self, req: HttpRequest) -> Result<HttpResponse, CoreError> {
        self.send_capped(req, self.attempt_timeout).await
    }

    /// `send` with a cap on each request. The retries keep their own
    /// schedule, so a 429 storm still ends as a rate limit rather than as
    /// a timeout, and one wedged connection still ends in a bounded wait.
    pub async fn send_within(
        &self,
        req: HttpRequest,
        timeout: Duration,
    ) -> Result<HttpResponse, CoreError> {
        self.send_capped(req, Some(timeout)).await
    }

    async fn send_capped(
        &self,
        req: HttpRequest,
        per_attempt: Option<Duration>,
    ) -> Result<HttpResponse, CoreError> {
        let attempt = || async {
            let sent_at = self.pace.turn().await;
            let sent = match per_attempt {
                Some(d) => match tokio::time::timeout(d, self.http.send(req.clone())).await {
                    Ok(sent) => sent,
                    // A request that never answered is a transport failure
                    // like any other: nothing is retried and the caller
                    // decides what to do about it.
                    Err(_) => Err(super::HttpError {
                        message: anibeam_contract::i18n::tr(
                            "core-net-timed-out",
                            &[("ms", d.as_millis().to_string().into())],
                        ),
                    }),
                },
                None => self.http.send(req.clone()).await,
            };
            if let Ok(r) = &sent {
                self.pace.observe(sent_at, r);
            }
            match sent {
                Ok(r) if is_rate_limited(&r) => Err(Attempt::Limited(Box::new(r))),
                Ok(r) => Ok(r),
                Err(e) => Err(Attempt::Transport(e.message)),
            }
        };
        let schedule = ExponentialBuilder::default()
            .with_min_delay(self.min_delay)
            .with_factor(2.0)
            .with_max_delay(MAX_RETRY_DELAY)
            .with_max_times(self.max_attempts);
        let result = attempt
            .retry(schedule)
            .when(|e| matches!(e, Attempt::Limited(_)))
            // The provider's own Retry-After beats the schedule, clamped to
            // the same ceiling so a bad header cannot park a job for hours.
            // The schedule still decides *whether* to retry: a `None` here
            // is backon's "out of attempts", and a Retry-After header must
            // never talk it into going round again.
            .adjust(|e, planned| {
                let planned = planned?;
                match e {
                    Attempt::Limited(r) => Some(wait_before_retry(retry_after(r), planned)),
                    Attempt::Transport(_) => Some(planned),
                }
            })
            .await;
        match result {
            Ok(r) => Ok(r),
            Err(Attempt::Limited(r)) => Err(CoreError::provider_retry(
                self.upstream.provider(),
                Some(429),
                anibeam_contract::i18n::tr(
                    "core-net-rate-limited",
                    &[("provider", self.upstream.label().into())],
                ),
                retry_after(&r),
            )),
            Err(Attempt::Transport(message)) => {
                Err(CoreError::provider(self.upstream.provider(), None, message))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FakeHttp;
    use anibeam_contract::Provider;
    use std::time::Instant;

    fn get(url: &str) -> HttpRequest {
        HttpRequest::get(url)
    }

    /// A server that takes its time and then answers.
    struct Slow {
        delay: Duration,
    }

    impl Http for Slow {
        fn send(
            &self,
            _req: HttpRequest,
        ) -> crate::BoxFuture<'_, Result<HttpResponse, crate::HttpError>> {
            let delay = self.delay;
            Box::pin(async move {
                tokio::time::sleep(delay).await;
                Ok(HttpResponse {
                    status: 200,
                    headers: vec![],
                    body: b"{}".to_vec(),
                })
            })
        }
    }

    /// The timeout is on one request, never on the call. A wedged
    /// connection still ends in a bounded transport failure, and a 429
    /// storm whose backoff outlasts the cap several times over still
    /// reports as the rate limit it is: a caller told "timed out" would
    /// retry into a provider that asked it to stop.
    #[tokio::test]
    async fn a_per_attempt_timeout_ends_one_request_and_never_the_retry_schedule() {
        let slow = Arc::new(Slow {
            delay: Duration::from_millis(200),
        });
        let client = ProviderClient::new(Upstream::Anilist, slow, Duration::from_millis(1));
        let err = client
            .send_within(get("https://x/"), Duration::from_millis(20))
            .await
            .err()
            .unwrap();
        assert!(
            matches!(&err, CoreError::Provider { status: None, message, .. } if message.contains("timed out")),
            "{err:?}"
        );

        let http = FakeHttp::new();
        for _ in 0..3 {
            http.push(429, "no");
        }
        let client = ProviderClient::new(Upstream::Anilist, http, Duration::from_millis(1))
            .with_min_delay(Duration::from_millis(40))
            .with_max_attempts(2);
        let start = Instant::now();
        let err = client
            .send_within(get("https://x/"), Duration::from_millis(5))
            .await
            .err()
            .unwrap();
        assert!(
            matches!(
                err,
                CoreError::Provider {
                    status: Some(429),
                    ..
                }
            ),
            "{err:?}"
        );
        assert!(
            start.elapsed() >= Duration::from_millis(120),
            "the schedule was cut short: {:?}",
            start.elapsed()
        );
    }

    /// A client built with a cap applies it to every plain `send` too,
    /// which is how the MAL client caps its calls without every call site
    /// saying so.
    #[tokio::test]
    async fn a_client_built_with_a_cap_applies_it_to_every_send() {
        let slow = Arc::new(Slow {
            delay: Duration::from_millis(200),
        });
        let client = ProviderClient::new(Upstream::Mal, slow, Duration::from_millis(1))
            .with_attempt_timeout(Duration::from_millis(20));
        let err = client.send(get("https://x/")).await.err().unwrap();
        assert!(
            matches!(&err, CoreError::Provider { status: None, message, .. } if message.contains("timed out")),
            "{err:?}"
        );
    }

    /// The provider's own wait wins, up to the ceiling. A header asking for
    /// an hour must never park a job for an hour, and one asking for
    /// nothing must not be talked into the schedule's wait instead.
    #[test]
    fn a_retry_after_is_taken_but_clamped_to_the_ceiling() {
        let planned = Duration::from_secs(8);
        assert_eq!(
            wait_before_retry(None, planned),
            planned,
            "no header, so the schedule decides"
        );
        assert_eq!(
            wait_before_retry(Some(0.0), planned),
            Duration::ZERO,
            "a wait of nothing is a wait of nothing"
        );
        assert_eq!(
            wait_before_retry(Some(5.5), planned),
            Duration::from_secs_f64(5.5)
        );
        assert_eq!(
            wait_before_retry(Some(3600.0), planned),
            MAX_RETRY_DELAY,
            "an hour is clamped to the ceiling"
        );
        assert_eq!(
            wait_before_retry(Some(-30.0), planned),
            Duration::ZERO,
            "a negative wait is nought, never a panic"
        );
    }

    #[tokio::test]
    async fn requests_are_paced_by_the_gap() {
        let http = FakeHttp::new();
        http.push(200, "a");
        http.push(200, "b");
        http.push(200, "c");
        let client =
            ProviderClient::new(Upstream::Anilist, http.clone(), Duration::from_millis(120));
        let start = Instant::now();
        for _ in 0..3 {
            client.send(get("https://x/")).await.unwrap();
        }
        assert!(
            start.elapsed() >= Duration::from_millis(240),
            "{:?}",
            start.elapsed()
        );
    }

    #[tokio::test]
    async fn a_429_is_retried_honouring_retry_after_then_succeeds() {
        let http = FakeHttp::new();
        http.push_with_headers(429, "slow down", vec![("Retry-After".into(), "1".into())]);
        http.push(200, "ok");
        let client = ProviderClient::new(Upstream::Jikan, http.clone(), Duration::from_millis(1));
        let start = Instant::now();
        let r = client.send(get("https://x/")).await.unwrap();
        assert_eq!(r.status, 200);
        assert!(
            start.elapsed() >= Duration::from_millis(900),
            "{:?}",
            start.elapsed()
        );
        assert_eq!(http.requests().len(), 2);
    }

    #[tokio::test]
    async fn exhausted_retries_become_a_provider_error_with_retry_after() {
        let http = FakeHttp::new();
        for _ in 0..7 {
            http.push_with_headers(429, "no", vec![("Retry-After".into(), "0".into())]);
        }
        let client = ProviderClient::new(Upstream::Anilist, http.clone(), Duration::from_millis(1))
            .with_min_delay(Duration::from_millis(1));
        let err = client.send(get("https://x/")).await.err().unwrap();
        assert!(
            matches!(
                err,
                CoreError::Provider {
                    provider: Provider::Anilist,
                    status: Some(429),
                    ..
                }
            ),
            "{err:?}"
        );
        assert_eq!(http.requests().len(), 7);
    }

    #[tokio::test]
    async fn one_attempt_exhausts_at_once_and_reports_the_upstream_as_mal() {
        let http = FakeHttp::new();
        http.push_with_headers(429, "no", vec![("Retry-After".into(), "9".into())]);
        let client = ProviderClient::new(Upstream::Jikan, http.clone(), Duration::from_millis(1))
            .with_max_attempts(0);
        let err = client.send(get("https://x/")).await.err().unwrap();
        assert!(
            matches!(err, CoreError::Provider { provider: Provider::Mal, status: Some(429), retry_after: Some(w), .. } if w == 9.0),
            "{err:?}"
        );
        assert_eq!(http.requests().len(), 1);
    }

    #[tokio::test]
    async fn a_429_inside_a_200_body_counts_as_rate_limited() {
        let http = FakeHttp::new();
        http.push(
            200,
            r#"{"data":null,"errors":[{"message":"Too Many Requests.","status":429}]}"#,
        );
        http.push(200, r#"{"data":{"Media":{"id":1}}}"#);
        let client = ProviderClient::new(Upstream::Anilist, http.clone(), Duration::from_millis(1))
            .with_min_delay(Duration::from_millis(1));
        let r = client
            .send(get("https://graphql.anilist.co"))
            .await
            .unwrap();
        assert!(String::from_utf8_lossy(&r.body).contains("Media"));
        assert_eq!(http.requests().len(), 2);
    }

    #[tokio::test]
    async fn other_statuses_pass_through_and_transport_failures_are_provider_errors() {
        let http = FakeHttp::new();
        http.push(500, "boom");
        let client = ProviderClient::new(Upstream::Anilist, http.clone(), Duration::from_millis(1));
        assert_eq!(client.send(get("https://x/")).await.unwrap().status, 500);

        let http = FakeHttp::new();
        http.fail_next("connection refused");
        let client = ProviderClient::new(Upstream::Anilist, http.clone(), Duration::from_millis(1));
        assert!(matches!(
            client.send(get("https://x/")).await,
            Err(CoreError::Provider { status: None, .. })
        ));
        assert_eq!(http.requests().len(), 1);
    }

    /// A server that keeps the limit AniList does: `limit` requests in any
    /// `window`, said on every reply in `X-RateLimit-Limit`, and a 429 for
    /// the one past it. It never says what remains, so a client that stays
    /// under it does so on the pace alone, never on a hold.
    struct Limited {
        limit: usize,
        window: Duration,
        seen: std::sync::Mutex<Vec<Instant>>,
        refused: std::sync::atomic::AtomicUsize,
    }

    impl Limited {
        fn new(limit: usize, window: Duration) -> Arc<Limited> {
            Arc::new(Limited {
                limit,
                window,
                seen: std::sync::Mutex::new(vec![]),
                refused: std::sync::atomic::AtomicUsize::new(0),
            })
        }
    }

    impl Http for Limited {
        fn send(
            &self,
            _req: HttpRequest,
        ) -> crate::BoxFuture<'_, Result<HttpResponse, crate::HttpError>> {
            let now = Instant::now();
            let mut seen = self.seen.lock().unwrap();
            seen.retain(|t| now.duration_since(*t) < self.window);
            let over = seen.len() >= self.limit;
            if over {
                self.refused
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            } else {
                seen.push(now);
            }
            let headers = vec![("X-RateLimit-Limit".to_string(), self.limit.to_string())];
            Box::pin(async move {
                Ok(HttpResponse {
                    status: if over { 429 } else { 200 },
                    headers,
                    body: b"{}".to_vec(),
                })
            })
        }
    }

    /// A Sync of many shows against AniList's degraded limit (30 a minute,
    /// here 5 in 400 ms) never gets a 429: the client reads the limit off
    /// the first reply and slows to it on its own, rather than running at
    /// its own gap and leaning on the backoff.
    #[tokio::test]
    async fn a_long_run_stays_under_the_limit_the_replies_report() {
        let window = Duration::from_millis(400);
        let server = Limited::new(5, window);
        let client =
            ProviderClient::new(Upstream::Anilist, server.clone(), Duration::from_millis(1))
                .with_live_limit(window)
                .with_min_delay(Duration::from_millis(1));
        for _ in 0..16 {
            client
                .send(get("https://graphql.anilist.co"))
                .await
                .unwrap();
        }
        assert_eq!(
            server.refused.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "the client ran past the limit the replies reported"
        );
    }

    /// A reply that says nothing is left holds the next request until the
    /// budget comes back: the reset when the reply gives one, a whole
    /// window when it does not.
    #[tokio::test]
    async fn none_remaining_holds_the_next_request_for_the_window() {
        let window = Duration::from_millis(300);
        let http = FakeHttp::new();
        let spent = vec![
            ("X-RateLimit-Limit".to_string(), "1000".to_string()),
            ("X-RateLimit-Remaining".to_string(), "0".to_string()),
        ];
        http.push_with_headers(200, "a", spent);
        http.push(200, "b");
        let client = ProviderClient::new(Upstream::Anilist, http.clone(), Duration::from_millis(1))
            .with_live_limit(window);
        let start = Instant::now();
        client.send(get("https://x/")).await.unwrap();
        client.send(get("https://x/")).await.unwrap();
        assert!(
            start.elapsed() >= window,
            "the second request went out with nothing left: {:?}",
            start.elapsed()
        );
    }

    /// An upstream not built with a live limit ignores the headers, so a
    /// provider whose numbers mean something else is never held by them.
    #[tokio::test]
    async fn without_a_live_limit_the_headers_change_nothing() {
        let http = FakeHttp::new();
        let spent = vec![
            ("X-RateLimit-Limit".to_string(), "1".to_string()),
            ("X-RateLimit-Remaining".to_string(), "0".to_string()),
        ];
        http.push_with_headers(200, "a", spent);
        http.push(200, "b");
        let client = ProviderClient::new(Upstream::Jikan, http.clone(), Duration::from_millis(1));
        let start = Instant::now();
        client.send(get("https://x/")).await.unwrap();
        client.send(get("https://x/")).await.unwrap();
        assert!(
            start.elapsed() < Duration::from_millis(200),
            "{:?}",
            start.elapsed()
        );
    }

    /// The pace follows the limit both ways: a degraded 30 widens it, and a
    /// limit back at 90 lets it close again, never below the floor. Only a
    /// reply to a newer request moves it.
    #[test]
    fn the_live_gap_follows_the_reported_limit() {
        let mut pace = Pace::new(Duration::from_millis(800));
        pace.window = Some(Duration::from_secs(60));
        let reply = |limit: &str| HttpResponse {
            status: 200,
            headers: vec![
                ("X-RateLimit-Limit".to_string(), limit.to_string()),
                ("X-RateLimit-Remaining".to_string(), "20".to_string()),
            ],
            body: vec![],
        };
        pace.observe(Instant::now().into(), &reply("30"));
        let gap = pace.state.lock().unwrap().live_gap;
        assert_eq!(
            gap,
            Duration::from_millis(2200),
            "30 a minute with headroom"
        );
        pace.observe(Instant::now().into(), &reply("90"));
        let gap = pace.state.lock().unwrap().live_gap;
        assert!(gap < pace.floor, "{gap:?}");
        pace.observe(Instant::now().into(), &reply("not a number"));
        let gap = pace.state.lock().unwrap().live_gap;
        assert!(
            gap < pace.floor,
            "a bad header keeps the last limit: {gap:?}"
        );

        // Two requests in flight as AniList degrades: the newer one's reply
        // says 30 and lands first, the older one's says 90 and lands after.
        let older: tokio::time::Instant = Instant::now().into();
        let newer = older + Duration::from_millis(5);
        pace.observe(newer, &reply("30"));
        pace.observe(older, &reply("90"));
        let gap = pace.state.lock().unwrap().live_gap;
        assert_eq!(
            gap,
            Duration::from_millis(2200),
            "a stale reply undid the newer limit"
        );
    }

    #[test]
    fn a_reset_reads_as_a_unix_time_or_as_seconds_from_now() {
        let now: i64 = 1_800_000_000;
        assert_eq!(
            reset_in(now as u64 + 42, now),
            Some(Duration::from_secs(42))
        );
        assert_eq!(reset_in(now as u64 - 5, now), None, "already past");
        assert_eq!(reset_in(now as u64, now), None, "now is not ahead");
        assert_eq!(reset_in(17, now), Some(Duration::from_secs(17)));
    }

    /// A reset that reads as already past is clock skew, so the hold falls
    /// back to the window rather than to nothing; a Retry-After, which no
    /// skew can move, wins over the reset.
    #[tokio::test]
    async fn a_skewed_reset_holds_the_window_and_retry_after_wins_over_it() {
        let window = Duration::from_millis(300);
        let past = vec![
            ("X-RateLimit-Limit".to_string(), "1000".to_string()),
            ("X-RateLimit-Remaining".to_string(), "0".to_string()),
            ("X-RateLimit-Reset".to_string(), "1000000001".to_string()),
        ];
        let http = FakeHttp::new();
        http.push_with_headers(200, "a", past.clone());
        http.push(200, "b");
        let client = ProviderClient::new(Upstream::Anilist, http.clone(), Duration::from_millis(1))
            .with_live_limit(window);
        let start = Instant::now();
        client.send(get("https://x/")).await.unwrap();
        client.send(get("https://x/")).await.unwrap();
        assert!(start.elapsed() >= window, "{:?}", start.elapsed());

        let window = Duration::from_secs(5);
        let mut asked = past;
        asked.push(("Retry-After".to_string(), "0.1".to_string()));
        let http = FakeHttp::new();
        http.push_with_headers(200, "a", asked);
        http.push(200, "b");
        let client = ProviderClient::new(Upstream::Anilist, http.clone(), Duration::from_millis(1))
            .with_live_limit(window);
        let start = Instant::now();
        client.send(get("https://x/")).await.unwrap();
        client.send(get("https://x/")).await.unwrap();
        let took = start.elapsed();
        assert!(
            took >= Duration::from_millis(100) && took < Duration::from_secs(2),
            "{took:?}"
        );
    }
}
