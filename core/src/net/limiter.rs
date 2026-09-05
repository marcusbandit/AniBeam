//! One limiter per upstream: a governor cell that paces every request to
//! the provider's published gap, and backon's schedule on top of it for a
//! 429. The gap is the floor, the 429 schedule is the ceiling, and nothing
//! else in the core ever calls a provider directly.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use backon::{ExponentialBuilder, Retryable};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};

use super::{Http, HttpRequest, HttpResponse, Upstream};
use crate::contract::CoreError;

/// The retry schedule the spec fixes: 1, 2, 4, 8, 16, 32 seconds, capped
/// at 60, six retries after the first request.
const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);
const DEFAULT_MAX_ATTEMPTS: usize = 6;

pub struct ProviderClient {
    upstream: Upstream,
    http: Arc<dyn Http>,
    limiter: DefaultDirectRateLimiter,
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
        let quota = Quota::with_period(gap)
            .unwrap_or_else(|| Quota::per_second(NonZeroU32::MIN))
            .allow_burst(NonZeroU32::MIN);
        ProviderClient {
            upstream,
            http,
            limiter: RateLimiter::direct(quota),
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
            self.limiter.until_ready().await;
            let sent = match per_attempt {
                Some(d) => match tokio::time::timeout(d, self.http.send(req.clone())).await {
                    Ok(sent) => sent,
                    // A request that never answered is a transport failure
                    // like any other: nothing is retried and the caller
                    // decides what to do about it.
                    Err(_) => Err(super::HttpError {
                        message: format!("timed out after {}ms", d.as_millis()),
                    }),
                },
                None => self.http.send(req.clone()).await,
            };
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
                    Attempt::Limited(r) => Some(
                        retry_after(r)
                            .map(|s| {
                                Duration::from_secs_f64(s.clamp(0.0, MAX_RETRY_DELAY.as_secs_f64()))
                            })
                            .unwrap_or(planned),
                    ),
                    Attempt::Transport(_) => Some(planned),
                }
            })
            .await;
        match result {
            Ok(r) => Ok(r),
            Err(Attempt::Limited(r)) => Err(CoreError::Provider {
                provider: self.upstream.provider(),
                status: Some(429),
                message: format!("{} rate limited", self.upstream.label()),
                retry_after: retry_after(&r),
            }),
            Err(Attempt::Transport(message)) => Err(CoreError::Provider {
                provider: self.upstream.provider(),
                status: None,
                message,
                retry_after: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Provider;
    use crate::net::{FakeHttp, Method};
    use std::time::Instant;

    fn get(url: &str) -> HttpRequest {
        HttpRequest {
            method: Method::Get,
            url: url.into(),
            headers: vec![],
            body: None,
        }
    }

    /// A server that takes its time and then answers.
    struct Slow {
        delay: Duration,
    }

    impl Http for Slow {
        fn send(
            &self,
            _req: HttpRequest,
        ) -> crate::net::BoxFuture<'_, Result<HttpResponse, crate::net::HttpError>> {
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
}
