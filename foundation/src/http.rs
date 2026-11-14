//! The one transport every team sends through (#854): a request, a response, and the
//! trait the real client and the tests' fake both answer. `Services` holds it, so it
//! lives below the network team, which holds the real client, the fake, the limiter and
//! one client per provider over it, and hands these names on as `anibeam_core::net::...`.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use serde::de::DeserializeOwned;

use anibeam_contract::CoreError;

/// A boxed future, so `Http` stays object safe and the fake and the real
/// client can be swapped behind one `Arc<dyn Http>`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

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

impl HttpRequest {
    /// A request with no headers, no body and the client's timeout. The
    /// methods below add the rest, so a call site reads as what it sends.
    pub fn new(method: Method, url: impl Into<String>) -> HttpRequest {
        HttpRequest {
            method,
            url: url.into(),
            headers: Vec::new(),
            body: None,
            timeout: None,
        }
    }

    pub fn get(url: impl Into<String>) -> HttpRequest {
        HttpRequest::new(Method::Get, url)
    }

    pub fn post(url: impl Into<String>) -> HttpRequest {
        HttpRequest::new(Method::Post, url)
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> HttpRequest {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Adds every header in `headers`, after any already set.
    pub fn headers(mut self, headers: Vec<(String, String)>) -> HttpRequest {
        self.headers.extend(headers);
        self
    }

    pub fn body(mut self, body: impl Into<Option<Body>>) -> HttpRequest {
        self.body = body.into();
        self
    }

    pub fn timeout(mut self, timeout: impl Into<Option<Duration>>) -> HttpRequest {
        self.timeout = timeout.into();
        self
    }
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
