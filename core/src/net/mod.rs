//! The network layer: one request type, one response type, one trait, and
//! the fake that stands in for all of it in tests. Nothing above this
//! module knows about reqwest.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::contract::{CoreError, Provider};

pub mod anilist;
pub mod aniskip;
pub mod doh;
pub mod imdb;
pub mod jikan;
pub mod limiter;
pub mod mal;
pub mod tvmaze;
pub mod wikidata;

/// A boxed future, so `Http` stays object safe and the fake and the real
/// client can be swapped behind one `Arc<dyn Http>`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Reads an explicit `null` as the type's default.
///
/// A struct-level `#[serde(default)]` covers a field the reply left out and
/// nothing else: a field present and null still fails, and one failed field
/// fails the whole reply. Providers send null where the schema promises a
/// list or an object often enough that every such field on a reply struct
/// goes through this, so one null costs the value rather than the series.
pub(crate) fn null_to_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    Ok(<Option<T> as serde::Deserialize>::deserialize(d)?.unwrap_or_default())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Patch,
}

/// The two body shapes the providers between them need: JSON for AniList's
/// GraphQL, a form for MAL's list writes and its token exchange.
#[derive(Clone, Debug, PartialEq)]
pub enum Body {
    Json(serde_json::Value),
    Form(Vec<(String, String)>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Body>,
    /// A ceiling on this one request, body included, in place of the
    /// client's. `None` keeps the client's 30 seconds, which is every
    /// request but the IMDb dump: megabytes rather than kilobytes, and a
    /// slow line should not cost a week's ratings.
    pub timeout: Option<Duration>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, CoreError> {
        serde_json::from_slice(&self.body).map_err(|e| CoreError::Internal {
            message: format!("json: {e}"),
        })
    }

    /// Header names are case-insensitive on the wire, so they are here too.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// The body as text, for a message. Truncated, since a provider that
    /// answers with an HTML error page should not put the page in the log.
    pub fn text(&self) -> String {
        let text = String::from_utf8_lossy(&self.body);
        let trimmed = text.trim();
        match trimmed.char_indices().nth(200) {
            Some((end, _)) => format!("{}...", &trimmed[..end]),
            None => trimmed.to_string(),
        }
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// A transport failure, and only that: a request that reached the server
/// comes back as a response whatever its status, so every provider judges
/// its own statuses.
#[derive(Clone, Debug, PartialEq)]
pub struct HttpError {
    pub message: String,
}

/// What a fetch from a host outside the contract's providers comes back
/// with when it brings nothing to write: IMDb's dataset mirror and
/// Wikidata's query service. Neither is a `Provider`, so a failure cannot
/// be a `CoreError::Provider` naming one; the job that asked turns this
/// into its own log line instead.
#[derive(Clone, Debug, PartialEq)]
pub enum Fault {
    /// Nothing answered: offline, a name that did not resolve, a timeout.
    Unreachable(String),
    /// The host answered, but not with anything to write: an error status,
    /// or a body that would not read.
    Refused(String),
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fault::Unreachable(message) => write!(f, "unreachable: {message}"),
            Fault::Refused(message) => f.write_str(message),
        }
    }
}

impl From<HttpError> for Fault {
    fn from(e: HttpError) -> Fault {
        Fault::Unreachable(e.message)
    }
}

pub trait Http: Send + Sync {
    fn send(&self, req: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, HttpError>>;
}

/// Which service a limiter paces. The contract's `Provider` names the
/// metadata sources; this names the five hosts the core actually talks to,
/// so Jikan and AniSkip can be paced separately while still reporting as
/// the MyAnimeList-keyed provider they belong to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Upstream {
    Anilist,
    Jikan,
    AniSkip,
    Mal,
    Tvmaze,
}

impl Upstream {
    pub fn provider(self) -> Provider {
        match self {
            Upstream::Anilist => Provider::Anilist,
            // Jikan and AniSkip are both keyed by MAL id and both answer
            // for MyAnimeList's data, so an error from either names MAL.
            Upstream::Jikan | Upstream::AniSkip | Upstream::Mal => Provider::Mal,
            Upstream::Tvmaze => Provider::Tvmaze,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Upstream::Anilist => "AniList",
            Upstream::Jikan => "Jikan",
            Upstream::AniSkip => "AniSkip",
            Upstream::Mal => "MAL",
            Upstream::Tvmaze => "TVmaze",
        }
    }
}

/// The real client. One per core, shared by every provider, with the
/// timeout applied per request.
pub struct ReqwestHttp {
    client: reqwest::Client,
}

impl ReqwestHttp {
    pub fn new(timeout: Duration) -> Result<ReqwestHttp, CoreError> {
        // rustls is the only TLS backend the manifest enables, so it is
        // already the default; asking for it by name calls a deprecated
        // builder method. The platform verifier comes with that feature.
        let builder = reqwest::Client::builder()
            .user_agent(format!("AniBeam/{}", crate::VERSION))
            .timeout(timeout);
        // Android's platform verifier needs a JNI handle and a Java helper
        // the app does not carry, so the phone checks against the bundled
        // Mozilla roots instead.
        #[cfg(target_os = "android")]
        let builder = {
            // Every root has to parse: a silently thinner store would turn
            // into certificate errors far from their cause.
            let roots = webpki_root_certs::TLS_SERVER_ROOT_CERTS
                .iter()
                .map(|der| reqwest::Certificate::from_der(der))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| CoreError::internal(format!("bundled TLS roots: {e}")))?;
            // Names are looked up over HTTPS: phone networks that block release sites by
            // name do it in plain DNS (see doh.rs). The resolvers' own client goes by
            // address, so it needs no lookup of its own.
            let resolvers = reqwest::Client::builder()
                .user_agent(format!("AniBeam/{}", crate::VERSION))
                .tls_certs_only(roots.clone())
                .build()
                .map_err(|e| CoreError::internal(format!("dns client: {e}")))?;
            builder
                .tls_certs_only(roots)
                .dns_resolver(std::sync::Arc::new(doh::Doh::new(resolvers)))
        };
        let client = builder
            .build()
            .map_err(|e| CoreError::internal(format!("http client: {e}")))?;
        Ok(ReqwestHttp { client })
    }
}

/// A client for the DNS-over-HTTPS resolvers, which it reaches by address: the phone's
/// bundled roots on Android, the system's elsewhere.
pub fn resolver_client() -> Result<reqwest::Client, CoreError> {
    let builder = reqwest::Client::builder().user_agent(format!("AniBeam/{}", crate::VERSION));
    #[cfg(target_os = "android")]
    let builder = {
        let roots = webpki_root_certs::TLS_SERVER_ROOT_CERTS
            .iter()
            .map(|der| reqwest::Certificate::from_der(der))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::internal(format!("bundled TLS roots: {e}")))?;
        builder.tls_certs_only(roots)
    };
    builder
        .build()
        .map_err(|e| CoreError::internal(format!("dns client: {e}")))
}

/// A transport error as a person reads it: the host, what went wrong and
/// why, never the request URL. reqwest's own text carries the whole URL, and
/// a Torznab feed URL holds the indexer's API key in its query, so that text
/// would put the key on screen and in the stored events.
fn describe(e: reqwest::Error) -> String {
    let host = e.url().and_then(|u| u.host_str()).map(str::to_owned);
    let e = e.without_url();
    let mut text = e.to_string();
    let mut source = std::error::Error::source(&e);
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    match host {
        Some(host) => format!("{host}: {text}"),
        None => text,
    }
}

impl Http for ReqwestHttp {
    fn send(&self, req: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, HttpError>> {
        let client = self.client.clone();
        Box::pin(async move {
            let method = match req.method {
                Method::Get => reqwest::Method::GET,
                Method::Post => reqwest::Method::POST,
                Method::Patch => reqwest::Method::PATCH,
            };
            let mut builder = client.request(method, &req.url);
            if let Some(timeout) = req.timeout {
                builder = builder.timeout(timeout);
            }
            for (name, value) in &req.headers {
                builder = builder.header(name, value);
            }
            builder = match &req.body {
                Some(Body::Json(value)) => builder.json(value),
                Some(Body::Form(pairs)) => builder.form(pairs),
                None => builder,
            };
            let response = builder.send().await.map_err(|e| HttpError {
                message: describe(e),
            })?;
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(name, value)| {
                    (
                        name.as_str().to_string(),
                        value.to_str().unwrap_or_default().to_string(),
                    )
                })
                .collect();
            let body = response
                .bytes()
                .await
                .map_err(|e| HttpError {
                    message: describe(e),
                })?
                .to_vec();
            Ok(HttpResponse {
                status,
                headers,
                body,
            })
        })
    }
}

/// One canned reply. `matcher` is `None` for a reply that answers any url.
struct Canned {
    matcher: Option<String>,
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

/// The `Http` every test uses. Replies are queued in front, requests are
/// recorded behind, and nothing touches the network.
#[derive(Default)]
pub struct FakeHttp {
    replies: Mutex<Vec<Canned>>,
    failures: Mutex<Vec<String>>,
    requests: Mutex<Vec<HttpRequest>>,
}

impl FakeHttp {
    pub fn new() -> Arc<FakeHttp> {
        Arc::new(FakeHttp::default())
    }

    pub fn push(&self, status: u16, body: impl Into<Vec<u8>>) {
        self.queue(None, status, vec![], body.into());
    }

    pub fn push_json(&self, status: u16, body: serde_json::Value) {
        self.queue(
            None,
            status,
            vec![("content-type".to_string(), "application/json".to_string())],
            body.to_string().into_bytes(),
        );
    }

    /// A reply that only answers a url containing `url_contains`, so a test
    /// that drives more than one provider does not depend on the order the
    /// job happens to call them in.
    pub fn push_for(&self, url_contains: &str, status: u16, body: impl Into<Vec<u8>>) {
        self.queue(Some(url_contains.to_string()), status, vec![], body.into());
    }

    pub fn push_with_headers(
        &self,
        status: u16,
        body: impl Into<Vec<u8>>,
        headers: Vec<(String, String)>,
    ) {
        self.queue(None, status, headers, body.into());
    }

    /// Both at once: a reply with headers that only answers a url
    /// containing `url_contains`. A test that queues a rate limit needs
    /// this, since an unmatched reply answers the next request whatever it
    /// was for, and an image fetch would eat the 429 meant for a query.
    pub fn push_for_with_headers(
        &self,
        url_contains: &str,
        status: u16,
        body: impl Into<Vec<u8>>,
        headers: Vec<(String, String)>,
    ) {
        self.queue(Some(url_contains.to_string()), status, headers, body.into());
    }

    /// The next `send` fails as if the connection never opened. Queued once
    /// per call, so two calls fail the next two sends.
    pub fn fail_next(&self, message: impl Into<String>) {
        self.failures.lock().unwrap().push(message.into());
    }

    pub fn requests(&self) -> Vec<HttpRequest> {
        self.requests.lock().unwrap().clone()
    }

    fn queue(
        &self,
        matcher: Option<String>,
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) {
        self.replies.lock().unwrap().push(Canned {
            matcher,
            status,
            headers,
            body,
        });
    }
}

impl Http for FakeHttp {
    fn send(&self, req: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, HttpError>> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(req.clone());
            {
                let mut failures = self.failures.lock().unwrap();
                if !failures.is_empty() {
                    return Err(HttpError {
                        message: failures.remove(0),
                    });
                }
            }
            let mut replies = self.replies.lock().unwrap();
            let found = replies.iter().position(|c| {
                c.matcher
                    .as_ref()
                    .is_none_or(|m| req.url.contains(m.as_str()))
            });
            match found {
                Some(index) => {
                    let canned = replies.remove(index);
                    Ok(HttpResponse {
                        status: canned.status,
                        headers: canned.headers,
                        body: canned.body,
                    })
                }
                None => Ok(HttpResponse {
                    status: 500,
                    headers: vec![],
                    body: format!("no canned reply for {}", req.url).into_bytes(),
                }),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(url: &str) -> HttpRequest {
        HttpRequest {
            method: Method::Get,
            url: url.into(),
            headers: vec![],
            body: None,
            timeout: None,
        }
    }

    /// A refused connection to a feed URL names the host and never the
    /// query, where a Torznab indexer keeps its API key.
    #[tokio::test]
    async fn a_transport_error_never_carries_the_request_url() {
        // Bound then dropped, so nothing listens on the port.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let http = ReqwestHttp::new(Duration::from_secs(5)).unwrap();
        let url = format!("http://127.0.0.1:{port}/api?t=search&apikey=s3cr3tkey&q=x");
        let err = http.send(get(&url)).await.unwrap_err();
        assert!(!err.message.contains("s3cr3tkey"), "{}", err.message);
        assert!(!err.message.contains("/api"), "{}", err.message);
        assert!(err.message.starts_with("127.0.0.1: "), "{}", err.message);
    }

    #[tokio::test]
    async fn canned_replies_come_back_in_order() {
        let http = FakeHttp::new();
        http.push(200, "first");
        http.push(201, "second");
        assert_eq!(http.send(get("https://x/")).await.unwrap().body, b"first");
        assert_eq!(http.send(get("https://x/")).await.unwrap().status, 201);
        assert_eq!(http.requests().len(), 2);
    }

    #[tokio::test]
    async fn a_matcher_only_answers_a_url_that_contains_it() {
        let http = FakeHttp::new();
        http.push_for("aniskip", 404, "miss");
        http.push(200, "anything");
        // The matched reply is skipped, so the unmatched one answers first.
        let r = http.send(get("https://graphql.anilist.co")).await.unwrap();
        assert_eq!(r.status, 200);
        // The matched reply is still queued and answers its own url.
        let r = http
            .send(get("https://api.aniskip.com/v2/skip-times/1/1"))
            .await
            .unwrap();
        assert_eq!(r.status, 404);
        // With the queue empty every url gets the same explanatory 500.
        let r = http.send(get("https://x/")).await.unwrap();
        assert_eq!(r.status, 500);
        assert!(String::from_utf8_lossy(&r.body).contains("no canned reply"));
    }

    #[tokio::test]
    async fn headers_read_case_insensitively_and_a_failure_is_a_transport_error() {
        let http = FakeHttp::new();
        http.push_with_headers(429, "no", vec![("Retry-After".into(), "3".into())]);
        let r = http.send(get("https://x/")).await.unwrap();
        assert_eq!(r.header("retry-after"), Some("3"));
        assert_eq!(r.header("RETRY-AFTER"), Some("3"));
        assert_eq!(r.header("x-nothing"), None);
        http.fail_next("connection refused");
        assert!(http.send(get("https://x/")).await.is_err());
    }
}
