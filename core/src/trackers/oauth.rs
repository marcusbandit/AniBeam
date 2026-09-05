//! The two sign-in flows, both of them ending at a loopback listener the
//! core binds itself.
//!
//! AniList is an implicit grant: the token comes back in the URL fragment,
//! which a server never sees, so the first hit is answered with a page
//! that re-issues the request with the fragment as a query. MAL is an
//! authorization code with plain PKCE, which is what MAL supports: it
//! rejects S256 outright, so the challenge is the verifier.
//!
//! Everything here is carried from Electron's `trackerHandler.ts`: the
//! urls character for character, the three pages, the five minute window,
//! and the order the pages are served in. The browser is answered before
//! the token is validated or exchanged, so the tab finishes cleanly
//! whatever the provider says next.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::net::anilist::VIEWER_QUERY;
use crate::percent::decode;
use crate::time;
use crate::trackers::accounts::{self, Tokens};
use crate::trackers::cache;

/// The one path the listener serves. Both providers redirect here.
const CALLBACK_PATH: &str = "/callback";

/// AniList's authorize endpoint runs Laravel Passport with strict
/// parameter validation: sending `redirect_uri`, `state` or `scope`, even
/// when they match the registration, gets a generic
/// `unsupported_grant_type` back. The docs' own example sends the two
/// parameters below and nothing else, so that is what this sends.
const ANILIST_AUTHORIZE: &str = "https://anilist.co/api/v2/oauth/authorize";
const MAL_AUTHORIZE: &str = "https://myanimelist.net/v1/oauth2/authorize";
const MAL_ME: &str = "https://api.myanimelist.net/v2/users/@me";

/// How long the listener waits for the browser to come back. Long enough
/// to make an account and sign in, short enough that a forgotten tab does
/// not hold the port for the rest of the session.
const FLOW_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// The state token and the PKCE verifier, in bytes before base64url.
const STATE_BYTES: usize = 16;
const VERIFIER_BYTES: usize = 32;

/// A request head longer than this is not a callback, it is somebody
/// filling the socket, so the read stops there.
const MAX_REQUEST: usize = 8 * 1024;

/// How long one connection has to send its request line. Connections are
/// served one at a time, and a browser routinely opens a socket it then
/// says nothing on, so a connection that stays silent is dropped rather
/// than allowed to hold the whole flow until it times out.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

const TIMED_OUT: &str = "Authorization timed out, try again.";
const STATE_MISMATCH: &str = "state mismatch, possible CSRF, aborted.";
const NO_CODE: &str = "no code in callback";
const NO_TOKEN: &str = "no access_token in callback";

/// What the pages call each provider. The site's own name, so the tab a
/// user is looking at says what they just signed in to; `Tracker::label`
/// says MAL, which is the shorthand the rest of the interface uses.
fn site_name(t: Tracker) -> &'static str {
    match t {
        Tracker::Anilist => "AniList",
        Tracker::Mal => "MyAnimeList",
    }
}

// The call -------------------------------------------------------------------

/// Starts the ConnectTracker job. The credentials are resolved here, on
/// the calling thread, so a connect with no client id fails the call
/// rather than a job nobody is watching.
pub fn connect(core: &Core, t: Tracker) -> Result<u64, CoreError> {
    let (client_id, client_secret) = accounts::effective_credentials(core, t)?;
    // A new connect supersedes whatever is in flight: the listener holds
    // one fixed port, so two flows cannot both be waiting on it, and a
    // stalled flow (a closed tab, a provider error page that never
    // redirected) must not lock the user out of trying again.
    if let Some(running) = core.jobs.running(JobKind::ConnectTracker) {
        let _ = core.jobs.cancel(running);
    }
    let owner = core
        .arc()
        .ok_or_else(|| CoreError::internal("core is shutting down"))?;
    let port = core.oauth_port.load(Ordering::SeqCst);
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::ConnectTracker, move |ctx| async move {
            run(owner, ctx, t, client_id, client_secret, port).await
        }))
}

/// What the browser came back with, once the pages have been served.
enum Callback {
    Anilist {
        access_token: String,
        expires_at: Option<SystemTime>,
    },
    Mal {
        code: String,
    },
}

async fn run(
    core: Arc<Core>,
    ctx: Arc<JobCtx>,
    t: Tracker,
    client_id: String,
    client_secret: Option<String>,
    port: u16,
) -> Result<Finished, CoreError> {
    // First, before anything is generated or announced: a port that is
    // already taken means there is nothing to announce.
    let listener = bind_with_retry(port).await?;
    let state = random_token(STATE_BYTES);
    let verifier = random_token(VERIFIER_BYTES);
    let redirect_url = format!("http://127.0.0.1:{port}{CALLBACK_PATH}");
    let open_url = auth_url(t, &client_id, &state, &verifier, &redirect_url);
    // The line the activity log keeps, with no url in it.
    ctx.emit(
        Level::Info,
        format!("{} sign-in: waiting for the browser", site_name(t)),
        EventBody::Notice,
    );
    // Debug on purpose: the bus persists the body of every event above
    // Debug, and MAL's authorize url carries the PKCE verifier as its
    // `code_challenge`, so an Info event here would write a secret into
    // the events table. A Debug event still reaches every subscriber, and
    // the shell that has to open the url is subscribed live.
    ctx.emit(
        Level::Debug,
        format!(
            "{} authorize url ready, waiting on {redirect_url}",
            site_name(t)
        ),
        EventBody::AuthUrlReady {
            tracker: t,
            open_url,
            redirect_url: redirect_url.clone(),
        },
    );

    let callback = tokio::time::timeout(FLOW_TIMEOUT, serve(&listener, t, &state))
        .await
        .map_err(|_| accounts::tracker_error(t, TIMED_OUT))??;

    let (user_id, username, tokens) = match callback {
        Callback::Anilist {
            access_token,
            expires_at,
        } => {
            let profile = core
                .anilist
                .graphql(VIEWER_QUERY, serde_json::json!({}), Some(&access_token))
                .await?;
            let username = profile["Viewer"]["name"]
                .as_str()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| accounts::tracker_error(t, "the profile came back without a name"))?
                .to_string();
            (
                profile["Viewer"]["id"].as_u64(),
                username,
                Tokens {
                    access_token,
                    refresh_token: None,
                    expires_at,
                },
            )
        }
        Callback::Mal { code } => {
            let tokens = exchange(
                &core,
                &code,
                &verifier,
                &client_id,
                client_secret.as_deref(),
                &redirect_url,
            )
            .await?;
            let profile = core.mal.get(MAL_ME, &tokens.access_token).await?;
            if !profile.is_success() {
                return Err(CoreError::Provider {
                    provider: Provider::Mal,
                    status: Some(u32::from(profile.status)),
                    message: profile.text(),
                    retry_after: None,
                });
            }
            let profile: serde_json::Value = profile.json()?;
            let username = profile["name"]
                .as_str()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| accounts::tracker_error(t, "the profile came back without a name"))?
                .to_string();
            (profile["id"].as_u64(), username, tokens)
        }
    };

    let store = accounts::save_tokens(&core, t, &tokens).await?;
    accounts::save_connection(
        &core,
        t,
        user_id,
        &username,
        &client_id,
        tokens.expires_at,
        store,
    )
    .await?;
    let trackers = accounts::state_async(&core).await?;
    ctx.emit(
        Level::Debug,
        "trackers changed",
        EventBody::TrackersChanged { state: trackers },
    );
    // The account has a list and the core has none of it, so the fetch is
    // started here rather than waited for by the first page that wants a
    // number. Forced, because a connection that has just happened cannot
    // be inside anybody's freshness window.
    cache::start_refresh(&core, Some(t), true);
    Ok(Finished {
        level: Level::Info,
        message: format!("{} connected as {username}", site_name(t)),
        body: EventBody::TrackerConnected {
            tracker: t,
            username,
        },
    })
}

/// The code exchange. The secret rides in the form, which is what makes
/// MAL a confidential client; `effective_credentials` has already refused
/// a MAL connect without one.
async fn exchange(
    core: &Arc<Core>,
    code: &str,
    verifier: &str,
    client_id: &str,
    client_secret: Option<&str>,
    redirect_url: &str,
) -> Result<Tokens, CoreError> {
    let form = vec![
        ("client_id".to_string(), client_id.to_string()),
        (
            "client_secret".to_string(),
            client_secret.unwrap_or_default().to_string(),
        ),
        ("code".to_string(), code.to_string()),
        ("code_verifier".to_string(), verifier.to_string()),
        ("grant_type".to_string(), "authorization_code".to_string()),
        ("redirect_uri".to_string(), redirect_url.to_string()),
    ];
    let response = core.mal.post_form(accounts::MAL_TOKEN_URL, form).await?;
    if !response.is_success() {
        return Err(CoreError::Provider {
            provider: Provider::Mal,
            status: Some(u32::from(response.status)),
            message: response.text(),
            retry_after: None,
        });
    }
    accounts::parse_tokens(&response.body, time::now())
}

// The listener ---------------------------------------------------------------

/// The loopback socket, bound through std so the listener exists before
/// the authorize url is announced, then handed to tokio. std sets
/// `SO_REUSEADDR` on Unix, so a socket still in `TIME_WAIT` from the last
/// flow does not refuse the next one.
fn bind(port: u16) -> Result<TcpListener, CoreError> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", port)).map_err(|e| {
        tracing::debug!("the loopback listener could not bind {port}: {e}");
        // In practice this is always the port being taken, and by the time
        // a user sees it the useful half is what to do about it.
        CoreError::Io {
            path: None,
            message: format!("port {port} is in use, another AniBeam is mid-connect"),
        }
    })?;
    listener.set_nonblocking(true).map_err(io_error)?;
    TcpListener::from_std(listener).map_err(io_error)
}

/// The bind, retried while the port is still held: up to two seconds, a
/// tenth of a second apart. A new connect cancels the one in flight, and
/// that flow's listener is dropped by its own task, so the caller can and
/// does get here first. Two seconds is far longer than a drop takes and
/// far shorter than a user would wait to be told about a real clash.
async fn bind_with_retry(port: u16) -> Result<TcpListener, CoreError> {
    const GAP: Duration = Duration::from_millis(100);
    const ATTEMPTS: u32 = 20;
    for _ in 1..ATTEMPTS {
        if let Ok(listener) = bind(port) {
            return Ok(listener);
        }
        tokio::time::sleep(GAP).await;
    }
    bind(port)
}

/// One connection at a time until the flow is settled. Everything that is
/// not the callback the core is waiting for is answered and the loop goes
/// round again, so a favicon request or a stray reload cannot end a flow.
async fn serve(listener: &TcpListener, t: Tracker, state: &str) -> Result<Callback, CoreError> {
    loop {
        let (mut stream, _) = listener.accept().await.map_err(io_error)?;
        let target = match read_target(&mut stream).await {
            Some(target) => target,
            // Not a GET, or nothing readable at all. Answered rather than
            // dropped, so the client is never left hanging.
            None => {
                reply(&mut stream, NOT_FOUND, "not found").await;
                continue;
            }
        };
        let (path, query) = target.split_once('?').unwrap_or((target.as_str(), ""));
        if path != CALLBACK_PATH {
            reply(&mut stream, NOT_FOUND, "not found").await;
            continue;
        }
        let params = query_pairs(query);
        let get = |name: &str| {
            params
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
        };

        // AniList lands here first with the token still in the fragment.
        // Neither parameter present is that hop, and the page below reissues
        // the request with the fragment as a query the listener can read.
        if t == Tracker::Anilist && get("access_token").is_none() && get("error").is_none() {
            reply(&mut stream, OK, FRAGMENT_FORWARDER).await;
            continue;
        }
        if let Some(error) = get("error") {
            reply(&mut stream, BAD_REQUEST, &error_page(&error)).await;
            return Err(accounts::tracker_error(
                t,
                format!("{} returned an error: {error}", site_name(t)),
            ));
        }
        if t == Tracker::Mal {
            // MAL carries the state back; AniList is never sent one, so it
            // is only checked where it was sent.
            if get("state").as_deref() != Some(state) {
                reply(&mut stream, BAD_REQUEST, &error_page(STATE_MISMATCH)).await;
                return Err(accounts::tracker_error(t, STATE_MISMATCH));
            }
            let Some(code) = get("code") else {
                reply(&mut stream, BAD_REQUEST, &error_page(NO_CODE)).await;
                return Err(accounts::tracker_error(t, NO_CODE));
            };
            // The page goes out before the exchange, so the tab is finished
            // with whatever MAL says to the request that follows.
            reply(&mut stream, OK, &success_page(site_name(t))).await;
            return Ok(Callback::Mal { code });
        }
        let Some(access_token) = get("access_token") else {
            reply(&mut stream, BAD_REQUEST, &error_page(NO_TOKEN)).await;
            return Err(accounts::tracker_error(t, NO_TOKEN));
        };
        reply(&mut stream, OK, &success_page(site_name(t))).await;
        let expires_at = get("expires_in")
            .and_then(|seconds| seconds.parse::<u64>().ok())
            .map(|seconds| time::now() + Duration::from_secs(seconds));
        return Ok(Callback::Anilist {
            access_token,
            expires_at,
        });
    }
}

/// The request target off the first line, reading until the blank line
/// that ends the headers. Nothing here trusts what arrives: a client that
/// never sends that blank line stops at 8 KB or at the read timeout, and a
/// first line that is not a GET of a target is simply not a callback.
/// Every one of those is `None`, never a failure: one bad connection must
/// not end a sign-in the user is halfway through.
async fn read_target(stream: &mut TcpStream) -> Option<String> {
    let head = match tokio::time::timeout(READ_TIMEOUT, read_head(stream)).await {
        Ok(Ok(head)) => head,
        Ok(Err(e)) => {
            tracing::debug!("the callback connection could not be read: {e}");
            return None;
        }
        Err(_) => {
            tracing::debug!("the callback connection sent nothing within {READ_TIMEOUT:?}");
            return None;
        }
    };
    let text = String::from_utf8_lossy(&head);
    let line = text.lines().next()?;
    let mut parts = line.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some("GET"), Some(target)) => Some(target.to_string()),
        _ => None,
    }
}

async fn read_head(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut head = Vec::new();
    let mut chunk = [0u8; 1024];
    while head.len() < MAX_REQUEST && !ends_headers(&head) {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        head.extend_from_slice(&chunk[..read]);
    }
    Ok(head)
}

fn ends_headers(head: &[u8]) -> bool {
    head.windows(4).any(|w| w == b"\r\n\r\n")
}

const OK: &str = "200 OK";
const BAD_REQUEST: &str = "400 Bad Request";
const NOT_FOUND: &str = "404 Not Found";

/// One reply, one connection. The browser reads to the end of the body and
/// the socket closes behind it, which is what `Connection: close` promises
/// and what makes the next hit of the flow a fresh accept.
///
/// Best effort by nature: the page is a courtesy to whoever is looking at
/// the tab, so a client that has already gone away is worth a log line and
/// nothing more. The flow itself turns on what the request carried.
async fn reply(stream: &mut TcpStream, status: &str, body: &str) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    if let Err(e) = write_all(stream, &head, body).await {
        tracing::debug!("the callback page could not be written: {e}");
    }
    // A client reading to EOF needs the write half closed before it sees
    // the page as complete.
    let _ = stream.shutdown().await;
}

async fn write_all(stream: &mut TcpStream, head: &str, body: &str) -> Result<(), std::io::Error> {
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body.as_bytes()).await?;
    stream.flush().await
}

fn io_error(e: std::io::Error) -> CoreError {
    CoreError::Io {
        path: None,
        message: e.to_string(),
    }
}

// Urls and encoding ----------------------------------------------------------

fn auth_url(
    t: Tracker,
    client_id: &str,
    state: &str,
    verifier: &str,
    redirect_url: &str,
) -> String {
    match t {
        Tracker::Anilist => format!(
            "{ANILIST_AUTHORIZE}?client_id={}&response_type=token",
            encode(client_id)
        ),
        // Plain PKCE: MAL does not support S256, so the challenge is the
        // verifier, which is what their own documentation shows.
        Tracker::Mal => format!(
            "{MAL_AUTHORIZE}?response_type=code&client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=plain",
            encode(client_id),
            encode(redirect_url),
            encode(state),
            encode(verifier)
        ),
    }
}

/// A fresh state or verifier: random bytes as base64url with no padding,
/// which is both what the providers accept in a query and what Electron
/// sent.
fn random_token(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

/// Percent-encoding of one query value, RFC 3986 unreserved characters
/// left alone. base64url output passes through untouched, which is why the
/// state and the verifier read the same on the wire as they do in memory.
fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// The query as pairs, decoded. A pair with no `=` is a name with an empty
/// value, which is how a browser sends a bare flag.
fn query_pairs(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            (decode(name), decode(value))
        })
        .collect()
}

// The pages ------------------------------------------------------------------

/// The three pages, carried from Electron's `trackerHandler.ts`. They are
/// the only interface a user sees in the browser, so they look like the
/// app rather than like a server default.
fn success_page(site: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\">\n\
         <title>AniBeam, connected</title>\n\
         <style>html,body{{margin:0;height:100%;background:#0b0b10;color:#f1f5f9;font-family:'JetBrains Mono',ui-monospace,monospace;display:flex;align-items:center;justify-content:center;flex-direction:column;gap:0.5rem}}h1{{font-weight:500;font-size:1rem}}p{{color:#94a3b8;font-size:0.85rem}}</style></head>\n\
         <body><h1>{} connected</h1><p>You can close this tab and return to AniBeam.</p></body></html>",
        escape(site)
    )
}

/// The message comes off the query the browser was redirected with, so it
/// is escaped before it goes into the page: nothing a provider or a
/// crafted link says should be able to write markup here.
fn error_page(message: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\">\n\
         <title>AniBeam, auth failed</title>\n\
         <style>html,body{{margin:0;height:100%;background:#0b0b10;color:#f1f5f9;font-family:'JetBrains Mono',ui-monospace,monospace;display:flex;align-items:center;justify-content:center;flex-direction:column;gap:0.5rem}}h1{{font-weight:500;font-size:1rem;color:#f43f5e}}p{{color:#94a3b8;font-size:0.85rem}}</style></head>\n\
         <body><h1>Authentication failed</h1><p>{}</p></body></html>",
        escape(message)
    )
}

/// The implicit-grant hop: the fragment never reaches the server, so this
/// page reads it in the browser and reloads with the same parameters as a
/// query, which the listener can read.
const FRAGMENT_FORWARDER: &str = "<!doctype html><html><head><meta charset=\"utf-8\"><title>Connecting</title>\n\
<style>html,body{margin:0;height:100%;background:#0b0b10;color:#94a3b8;font-family:'JetBrains Mono',ui-monospace,monospace;display:flex;align-items:center;justify-content:center}</style></head>\n\
<body>Connecting<script>\n\
(function(){\n\
  var h=window.location.hash;\n\
  if(!h){document.body.textContent='No token in URL.';return;}\n\
  window.location.replace(window.location.pathname+'?'+h.slice(1));\n\
})();\n\
</script></body></html>";

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_anilist_url_carries_two_parameters_and_no_more() {
        let url = auth_url(
            Tracker::Anilist,
            "123",
            "st",
            "vf",
            "http://127.0.0.1:53682/callback",
        );
        assert_eq!(
            url,
            "https://anilist.co/api/v2/oauth/authorize?client_id=123&response_type=token"
        );
    }

    #[test]
    fn the_mal_url_encodes_the_redirect_and_sends_a_plain_challenge() {
        let url = auth_url(
            Tracker::Mal,
            "mid",
            "st-1",
            "vf_2",
            "http://127.0.0.1:53682/callback",
        );
        assert_eq!(
            url,
            "https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id=mid\
             &redirect_uri=http%3A%2F%2F127.0.0.1%3A53682%2Fcallback&state=st-1&code_challenge=vf_2&code_challenge_method=plain"
        );
    }

    /// base64url is exactly the alphabet percent-encoding leaves alone, so
    /// a state reads the same in the url as it does in memory.
    #[test]
    fn a_random_token_is_url_safe_and_the_length_asked_for() {
        let token = random_token(16);
        assert_eq!(token, encode(&token));
        assert_eq!(URL_SAFE_NO_PAD.decode(&token).unwrap().len(), 16);
        assert_ne!(token, random_token(16));
    }

    #[test]
    fn a_query_decodes_pairs_plus_signs_and_escapes() {
        let pairs = query_pairs("code=a%2Bb&state=one+two&flag&empty=");
        assert_eq!(pairs[0], ("code".to_string(), "a+b".to_string()));
        assert_eq!(pairs[1], ("state".to_string(), "one two".to_string()));
        assert_eq!(pairs[2], ("flag".to_string(), String::new()));
        assert_eq!(pairs[3], ("empty".to_string(), String::new()));
        assert!(query_pairs("").is_empty());
    }

    #[test]
    fn the_pages_name_the_site_and_escape_what_a_provider_sent() {
        assert!(success_page(site_name(Tracker::Mal)).contains("MyAnimeList connected"));
        assert!(success_page(site_name(Tracker::Anilist)).contains("AniList connected"));
        let page = error_page("<script>alert(1)</script>");
        assert!(!page.contains("<script>alert"));
        assert!(page.contains("&lt;script&gt;"));
        assert!(FRAGMENT_FORWARDER.contains("window.location.hash"));
    }

    #[test]
    fn the_header_end_is_found_across_a_split_read() {
        assert!(!ends_headers(b"GET /callback HTTP/1.1\r\n"));
        assert!(ends_headers(b"GET /callback HTTP/1.1\r\nHost: x\r\n\r\n"));
    }
}
