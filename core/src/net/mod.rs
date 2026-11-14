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
pub mod front;
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
    Delete,
}

/// The body shapes the services between them need: JSON for AniList's
/// GraphQL, a form for MAL's list writes and its token exchange, and a
/// multipart form for a .torrent file uploaded to qBittorrent.
#[derive(Clone, Debug, PartialEq)]
pub enum Body {
    Json(serde_json::Value),
    Form(Vec<(String, String)>),
    /// `multipart/form-data`: the text fields, then the files.
    Multipart {
        fields: Vec<(String, String)>,
        files: Vec<FilePart>,
    },
}

/// One file in a multipart body.
#[derive(Clone, Debug, PartialEq)]
pub struct FilePart {
    /// The form field it goes in.
    pub field: String,
    pub file_name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// A multipart body's bytes around `boundary`, which must appear in none
/// of the parts. Names are quoted with `"`, CR and LF percent-encoded, as
/// browsers send them.
pub fn multipart_bytes(fields: &[(String, String)], files: &[FilePart], boundary: &str) -> Vec<u8> {
    fn quoted(s: &str) -> String {
        s.replace('"', "%22")
            .replace('\r', "%0D")
            .replace('\n', "%0A")
    }
    let mut out = Vec::new();
    for (name, value) in fields {
        out.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"{}\"\r\n\r\n",
                quoted(name)
            )
            .as_bytes(),
        );
        out.extend_from_slice(value.as_bytes());
        out.extend_from_slice(b"\r\n");
    }
    for f in files {
        out.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n\
                 Content-Type: {}\r\n\r\n",
                quoted(&f.field),
                quoted(&f.file_name),
                f.content_type
            )
            .as_bytes(),
        );
        out.extend_from_slice(&f.bytes);
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    out
}

/// A boundary that appears in none of the parts.
fn boundary_for(fields: &[(String, String)], files: &[FilePart]) -> String {
    loop {
        let boundary = format!("AniBeamBoundary{:016x}", rand::random::<u64>());
        let inside = |b: &[u8]| b.windows(boundary.len()).any(|w| w == boundary.as_bytes());
        let taken = fields
            .iter()
            .any(|(k, v)| inside(k.as_bytes()) || inside(v.as_bytes()))
            || files
                .iter()
                .any(|f| inside(&f.bytes) || inside(f.file_name.as_bytes()));
        if !taken {
            return boundary;
        }
    }
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

    /// A request to an endpoint, the only kind a definition's front may reroute (`front.rs`):
    /// a host the core talks to for itself (a tracker, a metadata provider) is never fronted,
    /// whatever a definition names. Every other client sends it as any request.
    fn send_endpoint(&self, req: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, HttpError>> {
        self.send(req)
    }

    /// The hosts to front and each one's front, from the installed endpoint definitions
    /// (`front.rs`). Only a client that fronts keeps them; the rest ignore them.
    fn set_fronts(&self, _fronts: Vec<(String, String)>) {}
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
    /// The client for fronted hosts (front.rs), on the phone; None elsewhere.
    fronted: Option<Front>,
    /// Which hosts are fronted, as the installed definitions last said.
    fronts: front::Fronts,
}

/// Fronted requests' client, cookies and pace. A front's edge can hand out cookies for the
/// site asked for and treat a client that never sends them back as a bot (#135). The URL
/// names the front, so a cookie store on the client would file them under the front's name
/// and turn away their `Domain=` of the site; the jar is kept by the real host instead.
struct Front {
    client: reqwest::Client,
    #[cfg(phone_os)]
    jar: reqwest::cookie::Jar,
    gate: tokio::sync::Mutex<front::Gate>,
}

impl Front {
    #[cfg_attr(not(phone_os), allow(dead_code))]
    fn new(client: reqwest::Client) -> Front {
        Front {
            client,
            #[cfg(phone_os)]
            jar: reqwest::cookie::Jar::default(),
            gate: Default::default(),
        }
    }

    async fn send(
        &self,
        #[cfg_attr(not(phone_os), allow(unused_mut))] mut req: HttpRequest,
        url: String,
        host: String,
    ) -> Result<HttpResponse, HttpError> {
        let mut gate = self.gate.lock().await;
        let wait = gate
            .admit(std::time::Instant::now())
            .map_err(|left| HttpError {
                message: crate::i18n::tr(
                    "core-net-waiting",
                    &[
                        ("host", host.as_str().into()),
                        ("seconds", left.as_secs().max(1).to_string().into()),
                    ],
                ),
            })?;
        tokio::time::sleep(wait).await;
        #[cfg(phone_os)]
        let real = reqwest::Url::parse(&req.url).ok();
        #[cfg(phone_os)]
        if let Some(real) = &real
            && let Some(cookies) = reqwest::cookie::CookieStore::cookies(&self.jar, real)
            && let Ok(cookies) = cookies.to_str()
        {
            req.headers.push(("Cookie".into(), cookies.to_string()));
        }
        let answer = request(self.client.clone(), req, url, Some(host)).await;
        gate.finished(std::time::Instant::now(), answer.is_ok());
        #[cfg(phone_os)]
        if let (Ok(response), Some(real)) = (&answer, &real) {
            let set: Vec<reqwest::header::HeaderValue> = response
                .headers
                .iter()
                .filter(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
                .filter_map(|(_, value)| reqwest::header::HeaderValue::from_str(value).ok())
                .collect();
            reqwest::cookie::CookieStore::set_cookies(&self.jar, &mut set.iter(), real);
        }
        answer
    }
}

impl ReqwestHttp {
    pub fn new(timeout: Duration) -> Result<ReqwestHttp, CoreError> {
        // rustls is the only TLS backend the manifest enables, so it is
        // already the default; asking for it by name calls a deprecated
        // builder method. The platform verifier comes with that feature.
        // Set on the phone only (front.rs).
        let fronts = front::Fronts::default();
        #[cfg(not(phone_os))]
        let fronted = None;
        #[cfg(phone_os)]
        let fronted;
        let builder = reqwest::Client::builder()
            .user_agent(format!("AniBeam/{}", crate::VERSION))
            .timeout(timeout);
        #[cfg(phone_os)]
        let builder = {
            let roots = bundled_roots()?;
            // Names are looked up over HTTPS: phone networks that block release sites by
            // name do it in plain DNS (see doh.rs). The resolvers' own client goes by
            // address, so it needs no lookup of its own.
            let resolvers = with_roots(
                reqwest::Client::builder().user_agent(format!("AniBeam/{}", crate::VERSION)),
                roots.clone(),
            )
            .build()
            .map_err(|e| CoreError::internal(format!("dns client: {e}")))?;
            let doh = doh::Doh::new(resolvers);
            fronted = Some(Front::new(front::client(
                &doh,
                &fronts,
                roots.clone(),
                timeout,
            )?));
            with_roots(builder, roots).dns_resolver(std::sync::Arc::new(doh))
        };
        let client = builder
            .build()
            .map_err(|e| CoreError::internal(format!("http client: {e}")))?;
        Ok(ReqwestHttp {
            client,
            fronted,
            fronts,
        })
    }
}

/// A client for the DNS-over-HTTPS resolvers, which it reaches by address: the phone's
/// bundled roots on Android, the system's elsewhere.
pub fn resolver_client() -> Result<reqwest::Client, CoreError> {
    plain_builder()?
        .build()
        .map_err(|e| CoreError::internal(format!("dns client: {e}")))
}

/// A client for the watch together relay (#846): HTTP/1.1 only, since its WebSocket is an
/// HTTP/1.1 upgrade, and no overall timeout, which would cut the socket that upgrade
/// leaves open. The same TLS as every other client: the system's roots, or the phone's
/// bundled ones.
pub fn room_client(connect_timeout: Duration) -> Result<reqwest::Client, CoreError> {
    plain_builder()?
        .http1_only()
        .connect_timeout(connect_timeout)
        .build()
        .map_err(|e| CoreError::internal(format!("room client: {e}")))
}

/// A client builder with the user agent and, on Android, the bundled Mozilla roots.
fn plain_builder() -> Result<reqwest::ClientBuilder, CoreError> {
    let builder = reqwest::Client::builder().user_agent(format!("AniBeam/{}", crate::VERSION));
    Ok(with_roots(builder, bundled_roots()?))
}

/// The builder checking against `roots` only, or the platform's verifier without them.
fn with_roots(
    builder: reqwest::ClientBuilder,
    roots: Option<Vec<reqwest::Certificate>>,
) -> reqwest::ClientBuilder {
    match roots {
        Some(roots) => builder.tls_certs_only(roots),
        None => builder,
    }
}

/// The Mozilla roots the app carries, on Android only: its platform verifier needs a JNI
/// handle and a Java helper the app does not carry. Every other system, the iPhone's
/// included, verifies against its own store. Every root has to parse: a silently thinner
/// store would turn into certificate errors far from their cause.
fn bundled_roots() -> Result<Option<Vec<reqwest::Certificate>>, CoreError> {
    #[cfg(target_os = "android")]
    return webpki_root_certs::TLS_SERVER_ROOT_CERTS
        .iter()
        .map(|der| reqwest::Certificate::from_der(der))
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
        .map_err(|e| CoreError::internal(format!("bundled TLS roots: {e}")));
    #[cfg(not(target_os = "android"))]
    Ok(None)
}

/// A URL as a log or error line names it: scheme, host and a port the URL names, since a
/// login, a passkey or an API key can sit in the userinfo, the path or the query. Read
/// with the HTTP client's own parser, so a link it fetches with backslashes for slashes
/// cannot pass its path off as the host. A URL with no host is `a malformed link`.
pub fn url_shown(url: &str) -> String {
    parsed(url).map_or_else(
        || MALFORMED.to_string(),
        |u| format!("{}://{}", u.scheme(), host_and_port(&u)),
    )
}

/// The host alone, with a port the URL names, for an error's service name:
/// `t.example`, `127.0.0.1:9117`. Read the way [`url_shown`] reads it.
pub fn host_shown(url: &str) -> String {
    parsed(url).map_or_else(|| MALFORMED.to_string(), |u| host_and_port(&u))
}

/// `url` as the HTTP client reads it, when it names a host.
pub(crate) fn parsed(url: &str) -> Option<reqwest::Url> {
    reqwest::Url::parse(url)
        .ok()
        .filter(|u| u.host_str().is_some())
}

fn host_and_port(u: &reqwest::Url) -> String {
    let host = u.host_str().unwrap_or_default();
    match u.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_string(),
    }
}

const MALFORMED: &str = "a malformed link";

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
        let url = req.url.clone();
        Box::pin(request(self.client.clone(), req, url, None))
    }

    fn send_endpoint(&self, req: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, HttpError>> {
        // A host a network blocks by name goes to its server under its front's name.
        match (&self.fronted, self.fronts.fronted(&req.url)) {
            (Some(fronted), Some((url, host))) => Box::pin(fronted.send(req, url, host)),
            _ => {
                let url = req.url.clone();
                Box::pin(request(self.client.clone(), req, url, None))
            }
        }
    }

    fn set_fronts(&self, fronts: Vec<(String, String)>) {
        self.fronts.set(fronts);
    }
}

/// One request on `client` to `url`, with `host` as its Host header when it is fronted.
async fn request(
    client: reqwest::Client,
    req: HttpRequest,
    url: String,
    host: Option<String>,
) -> Result<HttpResponse, HttpError> {
    let method = match req.method {
        Method::Get => reqwest::Method::GET,
        Method::Post => reqwest::Method::POST,
        Method::Patch => reqwest::Method::PATCH,
        Method::Delete => reqwest::Method::DELETE,
    };
    let mut builder = client.request(method, &url);
    if let Some(host) = host {
        builder = builder.header("Host", host);
    }
    if let Some(timeout) = req.timeout {
        builder = builder.timeout(timeout);
    }
    for (name, value) in &req.headers {
        builder = builder.header(name, value);
    }
    builder = match &req.body {
        Some(Body::Json(value)) => builder.json(value),
        Some(Body::Form(pairs)) => builder.form(pairs),
        Some(Body::Multipart { fields, files }) => {
            let boundary = boundary_for(fields, files);
            builder
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(multipart_bytes(fields, files, &boundary))
        }
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
    fronts: Mutex<Vec<(String, String)>>,
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

    /// The fronts the core last handed over (`Http::set_fronts`).
    pub fn fronts(&self) -> Vec<(String, String)> {
        self.fronts.lock().unwrap().clone()
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

    fn set_fronts(&self, fronts: Vec<(String, String)>) {
        *self.fronts.lock().unwrap() = fronts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_url_is_shown_by_scheme_and_host_only() {
        // A login, a passkey in the path or the query and an API key stay out (#451).
        for (url, shown, host) in [
            (
                "https://user:0123abcd@t.example/rss",
                "https://t.example",
                "t.example",
            ),
            (
                "https://t.example:8443/0123abcd/announce?passkey=0123abcd",
                "https://t.example:8443",
                "t.example:8443",
            ),
            (
                "https://t.example/dl/0123abcd/file.torrent",
                "https://t.example",
                "t.example",
            ),
            (
                "https://user:0123abcd@t.example:8443/dl/0123abcd.torrent?passkey=0123abcd",
                "https://t.example:8443",
                "t.example:8443",
            ),
            // The client reads a backslash as a slash, and so does the log line.
            (
                "https://t.example\\0123abcd\\announce",
                "https://t.example",
                "t.example",
            ),
            (
                "https://t.example\\dl\\0123abcd\\file.torrent",
                "https://t.example",
                "t.example",
            ),
            (
                "http://127.0.0.1:9117/api?apikey=0123abcd",
                "http://127.0.0.1:9117",
                "127.0.0.1:9117",
            ),
            (
                "http://[2001:db8::1]:8080/x",
                "http://[2001:db8::1]:8080",
                "[2001:db8::1]:8080",
            ),
            // A default port is no port.
            (
                "https://t.example:443/rss",
                "https://t.example",
                "t.example",
            ),
            (
                "udp://t.example:1337/announce",
                "udp://t.example:1337",
                "t.example:1337",
            ),
        ] {
            assert_eq!(url_shown(url), shown, "{url}");
            assert_eq!(host_shown(url), host, "{url}");
        }
        // A UDP announce is not a special scheme, so a backslash is no separator and the
        // host does not parse; it reads as malformed rather than naming its path.
        for bad in [
            "udp://t.example\\0123abcd:1337/announce",
            "http://[t.example/0123abcd",
            "/rss/0123abcd",
            "0123abcd",
            "",
        ] {
            assert_eq!(url_shown(bad), "a malformed link", "{bad}");
            assert_eq!(host_shown(bad), "a malformed link", "{bad}");
        }
    }

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

    #[test]
    fn a_multipart_body_frames_each_part() {
        let body = multipart_bytes(
            &[("savepath".to_string(), "/srv/TV/Show".to_string())],
            &[FilePart {
                field: "torrents".to_string(),
                file_name: "a \"b\".torrent".to_string(),
                content_type: "application/x-bittorrent".to_string(),
                bytes: b"d4:infoe".to_vec(),
            }],
            "XyZ",
        );
        assert_eq!(
            String::from_utf8(body).unwrap(),
            "--XyZ\r\nContent-Disposition: form-data; name=\"savepath\"\r\n\r\n/srv/TV/Show\r\n\
             --XyZ\r\nContent-Disposition: form-data; name=\"torrents\"; filename=\"a %22b%22.torrent\"\r\n\
             Content-Type: application/x-bittorrent\r\n\r\nd4:infoe\r\n--XyZ--\r\n"
        );
        let file = FilePart {
            field: "f".to_string(),
            file_name: "x".to_string(),
            content_type: "application/octet-stream".to_string(),
            bytes: b"AniBeamBoundary".to_vec(),
        };
        assert!(boundary_for(&[], &[file]).starts_with("AniBeamBoundary"));
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
