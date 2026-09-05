//! The two loopback OAuth flows, driven end to end: the core binds the
//! port, the test plays the browser coming back to it, and the provider
//! answers off `FakeHttp`.

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use anibeam_core::*;

/// The browser's half of the flow: one request to the core's listener,
/// read to EOF, since every reply closes the connection.
fn http_get(port: u16, path: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(s, "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out
}

fn ready_url(c: &anibeam_core::events::Collector, job: u64) -> String {
    let ready = common::wait_for(
        c,
        |e| matches!(e.body, EventBody::AuthUrlReady { .. }) && e.job.as_ref().is_some_and(|j| j.id == job),
        Duration::from_secs(5),
    );
    match ready.body {
        EventBody::AuthUrlReady { open_url, .. } => open_url,
        other => panic!("{other:?}"),
    }
}

fn started(reply: Reply) -> u64 {
    match reply {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

/// The value of one query parameter of an authorize url.
fn param(url: &str, name: &str) -> String {
    let needle = format!("{name}=");
    url.split(&needle).nth(1).unwrap().split('&').next().unwrap().to_string()
}

#[test]
fn anilist_implicit_grant_through_the_forwarder_connects() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_oauth_port(53690);
    core.call(Call::SetTrackerCredentials { tracker: Tracker::Anilist, client_id: "123".into(), client_secret: None })
        .unwrap();
    http.push_json(200, serde_json::json!({ "data": { "Viewer": { "id": 42, "name": "bandit" } } }));
    let job = started(core.call(Call::ConnectTracker { tracker: Tracker::Anilist }).unwrap());
    let ready = common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::AuthUrlReady { .. }) && e.job.as_ref().is_some_and(|j| j.id == job),
        Duration::from_secs(5),
    );
    match ready.body {
        EventBody::AuthUrlReady { open_url, redirect_url, .. } => {
            assert_eq!(open_url, "https://anilist.co/api/v2/oauth/authorize?client_id=123&response_type=token");
            assert_eq!(redirect_url, "http://127.0.0.1:53690/callback");
        }
        other => panic!("{other:?}"),
    }

    // The authorize url never reaches the events table: MAL's carries the
    // PKCE verifier, and secrets never enter the database. What the log
    // keeps is the notice that the flow is waiting for the browser.
    match core.call(Call::RecentEvents { limit: 50 }).unwrap() {
        Reply::Events { events } => {
            for event in &events {
                let json = serde_json::to_string(event).unwrap();
                assert!(!json.contains("response_type"), "an authorize url was persisted: {json}");
            }
            assert!(
                events.iter().any(|e| e.message == "AniList sign-in: waiting for the browser"),
                "{events:#?}"
            );
        }
        other => panic!("{other:?}"),
    }

    assert!(http_get(53690, "/nope").starts_with("HTTP/1.1 404"));
    // The token is in the fragment, which the listener never sees, so the
    // first hit is answered with the page that re-issues it as a query.
    let first = http_get(53690, "/callback");
    assert!(first.contains("window.location.hash"), "{first}");
    let second = http_get(53690, "/callback?access_token=tok&expires_in=3600&token_type=Bearer");
    assert!(second.contains("AniList connected"), "{second}");

    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::TrackerConnected { tracker: Tracker::Anilist, ref username } if username == "bandit"),
        "{done:?}"
    );
    match core.call(Call::GetTrackers).unwrap() {
        Reply::Trackers { state } => {
            assert!(state.anilist.connected);
            assert_eq!(state.anilist.username.as_deref(), Some("bandit"));
            assert_eq!(state.anilist.user_id, Some(42));
            assert_eq!(state.anilist.client_id, "123");
            assert!(state.anilist.expires_at.is_some());
        }
        other => panic!("{other:?}"),
    }
    let viewer = &http.requests()[0];
    assert!(viewer.headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer tok"), "{viewer:?}");

    assert!(matches!(core.call(Call::DisconnectTracker { tracker: Tracker::Anilist }).unwrap(), Reply::Ok));
    match core.call(Call::GetTrackers).unwrap() {
        Reply::Trackers { state } => {
            assert!(!state.anilist.connected);
            assert_eq!(state.anilist.username, None);
            // The credentials survive a disconnect; only the account and
            // its tokens go.
            assert_eq!(state.anilist.client_id, "123");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn mal_pkce_flow_checks_state_and_exchanges_the_code() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_oauth_port(53691);
    core.call(Call::SetTrackerCredentials {
        tracker: Tracker::Mal,
        client_id: "mid".into(),
        client_secret: Some("sec".into()),
    })
    .unwrap();
    http.push_for(
        "oauth2/token",
        200,
        serde_json::json!({ "access_token": "mtok", "refresh_token": "mref", "expires_in": 2_415_600 }).to_string(),
    );
    http.push_for("users/@me", 200, serde_json::json!({ "id": 7, "name": "bandit" }).to_string());

    let job = started(core.call(Call::ConnectTracker { tracker: Tracker::Mal }).unwrap());
    let open_url = ready_url(&c, job);
    assert!(
        open_url.starts_with(
            "https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id=mid&redirect_uri=http%3A%2F%2F127.0.0.1%3A53691%2Fcallback&state="
        ),
        "{open_url}"
    );
    assert!(open_url.contains("&code_challenge_method=plain"), "{open_url}");

    // A state that is not the one sent is a CSRF attempt, not a callback.
    let bad = http_get(53691, "/callback?code=abc&state=wrong");
    assert!(bad.contains("state mismatch"), "{bad}");
    let failed = common::wait_job(&c, job);
    assert!(matches!(failed.body, EventBody::JobFailed { .. }), "{failed:?}");

    let job = started(core.call(Call::ConnectTracker { tracker: Tracker::Mal }).unwrap());
    let open_url = ready_url(&c, job);
    let state = param(&open_url, "state");
    let verifier = param(&open_url, "code_challenge");
    let ok = http_get(53691, &format!("/callback?code=abc&state={state}"));
    assert!(ok.contains("MyAnimeList connected"), "{ok}");
    let done = common::wait_job(&c, job);
    assert!(matches!(done.body, EventBody::TrackerConnected { tracker: Tracker::Mal, .. }), "{done:?}");

    let exchange = http.requests().into_iter().find(|r| r.url.contains("oauth2/token")).unwrap();
    match exchange.body {
        Some(anibeam_core::net::Body::Form(f)) => {
            assert!(f.contains(&("code_verifier".into(), verifier)), "{f:?}");
            assert!(f.contains(&("code".into(), "abc".into())), "{f:?}");
            assert!(f.contains(&("grant_type".into(), "authorization_code".into())), "{f:?}");
            assert!(f.contains(&("client_secret".into(), "sec".into())), "{f:?}");
            assert!(f.contains(&("redirect_uri".into(), "http://127.0.0.1:53691/callback".into())), "{f:?}");
        }
        other => panic!("{other:?}"),
    }
    match core.call(Call::GetTrackers).unwrap() {
        Reply::Trackers { state } => {
            assert!(state.mal.connected);
            assert_eq!(state.mal.username.as_deref(), Some("bandit"));
            assert!(state.mal.has_client_secret);
        }
        other => panic!("{other:?}"),
    }
}
