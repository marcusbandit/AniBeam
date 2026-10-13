//! The two loopback OAuth flows, driven end to end: the core binds the
//! port, the test plays the browser coming back to it, and the provider
//! answers off `FakeHttp`.

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use anibeam_core::trackers::accounts;
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
        |e| {
            matches!(e.body, EventBody::AuthUrlReady { .. })
                && e.job.as_ref().is_some_and(|j| j.id == job)
        },
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
    url.split(&needle)
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap()
        .to_string()
}

#[test]
fn anilist_implicit_grant_through_the_forwarder_connects() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_oauth_port(53690);
    core.call(Call::SetTrackerCredentials {
        tracker: Tracker::Anilist,
        client_id: "123".into(),
        client_secret: None,
    })
    .unwrap();
    http.push_json(
        200,
        serde_json::json!({ "data": { "Viewer": { "id": 42, "name": "bandit" } } }),
    );
    // The list the connection just gave access to: a finished connect
    // starts a forced refresh of its own, and this is what it reads.
    http.push_json(
        200,
        serde_json::json!({ "data": { "MediaListCollection": { "lists": [
            { "entries": [{ "progress": 5, "status": "CURRENT", "score": 8.5, "repeat": 0, "media": { "id": 154587 } }] }
        ] } } }),
    );
    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Anilist,
        })
        .unwrap(),
    );
    let ready = common::wait_for(
        &c,
        |e| {
            matches!(e.body, EventBody::AuthUrlReady { .. })
                && e.job.as_ref().is_some_and(|j| j.id == job)
        },
        Duration::from_secs(5),
    );
    match ready.body {
        EventBody::AuthUrlReady {
            open_url,
            redirect_url,
            ..
        } => {
            assert_eq!(
                open_url,
                "https://anilist.co/api/v2/oauth/authorize?client_id=123&response_type=token"
            );
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
                assert!(
                    !json.contains("response_type"),
                    "an authorize url was persisted: {json}"
                );
            }
            assert!(
                events
                    .iter()
                    .any(|e| e.message == "AniList sign-in: waiting for the browser"),
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
    let second = http_get(
        53690,
        "/callback?access_token=tok&expires_in=3600&token_type=Bearer",
    );
    assert!(second.contains("AniList is connected to"), "{second}");

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
    assert!(
        viewer
            .headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer tok"),
        "{viewer:?}"
    );

    // The connect ends with a forced progress refresh, so the account's
    // list is cached before any page asks for a number.
    common::wait_for(
        &c,
        |e| {
            matches!(
                e.body,
                EventBody::ProgressRefreshed {
                    tracker: Tracker::Anilist
                }
            )
        },
        Duration::from_secs(5),
    );

    assert!(matches!(
        core.call(Call::DisconnectTracker {
            tracker: Tracker::Anilist
        })
        .unwrap(),
        Reply::Ok
    ));
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
    http.push_for(
        "users/@me",
        200,
        serde_json::json!({ "id": 7, "name": "bandit" }).to_string(),
    );
    // The forced refresh the connect ends with. Matched on the list path,
    // since `myanimelist.net` carries `animelist` in the host itself.
    http.push_for(
        "users/@me/animelist",
        200,
        serde_json::json!({ "data": [{ "node": { "id": 52991 }, "list_status": { "status": "watching", "num_episodes_watched": 5 } }], "paging": {} })
            .to_string(),
    );

    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Mal,
        })
        .unwrap(),
    );
    let open_url = ready_url(&c, job);
    assert!(
        open_url.starts_with(
            "https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id=mid&redirect_uri=http%3A%2F%2F127.0.0.1%3A53691%2Fcallback&state="
        ),
        "{open_url}"
    );
    assert!(
        open_url.contains("&code_challenge_method=plain"),
        "{open_url}"
    );

    // A state that is not the one sent is a CSRF attempt, not a callback.
    let bad = http_get(53691, "/callback?code=abc&state=wrong");
    assert!(bad.contains("state mismatch"), "{bad}");
    let failed = common::wait_job(&c, job);
    assert!(
        matches!(failed.body, EventBody::JobFailed { .. }),
        "{failed:?}"
    );

    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Mal,
        })
        .unwrap(),
    );
    let open_url = ready_url(&c, job);
    let state = param(&open_url, "state");
    let verifier = param(&open_url, "code_challenge");
    let ok = http_get(53691, &format!("/callback?code=abc&state={state}"));
    assert!(ok.contains("MyAnimeList is connected to"), "{ok}");
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::TrackerConnected {
                tracker: Tracker::Mal,
                ..
            }
        ),
        "{done:?}"
    );

    let exchange = http
        .requests()
        .into_iter()
        .find(|r| r.url.contains("oauth2/token"))
        .unwrap();
    match exchange.body {
        Some(anibeam_core::net::Body::Form(f)) => {
            assert!(f.contains(&("code_verifier".into(), verifier)), "{f:?}");
            assert!(f.contains(&("code".into(), "abc".into())), "{f:?}");
            assert!(
                f.contains(&("grant_type".into(), "authorization_code".into())),
                "{f:?}"
            );
            assert!(f.contains(&("client_secret".into(), "sec".into())), "{f:?}");
            assert!(
                f.contains(&(
                    "redirect_uri".into(),
                    "http://127.0.0.1:53691/callback".into()
                )),
                "{f:?}"
            );
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
    common::wait_for(
        &c,
        |e| {
            matches!(
                e.body,
                EventBody::ProgressRefreshed {
                    tracker: Tracker::Mal
                }
            )
        },
        Duration::from_secs(5),
    );
}

/// The form of the one token request a test's core made.
fn token_form(http: &anibeam_core::net::FakeHttp) -> Vec<(String, String)> {
    let exchange = http
        .requests()
        .into_iter()
        .find(|r| r.url.contains("oauth2/token"))
        .expect("a token request");
    match exchange.body {
        Some(anibeam_core::net::Body::Form(f)) => f,
        other => panic!("{other:?}"),
    }
}

/// A MAL app registered with App Type "other" is a public client: no
/// secret exists, and the exchange carries the id and the verifier alone.
/// The field is left out, not sent empty.
#[test]
fn mal_without_a_secret_signs_in_as_a_public_client() {
    // A build that bundles a MAL secret always has one to send, so there
    // is no secretless flow to drive in it.
    if accounts::bundled_client_secret(Tracker::Mal).is_some() {
        return;
    }
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_oauth_port(53693);
    core.call(Call::SetTrackerCredentials {
        tracker: Tracker::Mal,
        client_id: "mid".into(),
        client_secret: None,
    })
    .unwrap();
    http.push_for(
        "oauth2/token",
        200,
        serde_json::json!({ "access_token": "mtok", "refresh_token": "mref", "expires_in": 2_415_600 }).to_string(),
    );
    http.push_for(
        "users/@me",
        200,
        serde_json::json!({ "id": 7, "name": "bandit" }).to_string(),
    );
    http.push_for(
        "users/@me/animelist",
        200,
        serde_json::json!({ "data": [], "paging": {} }).to_string(),
    );

    // No secret anywhere is no longer a refusal: the call starts a job.
    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Mal,
        })
        .unwrap(),
    );
    let open_url = ready_url(&c, job);
    let state = param(&open_url, "state");
    let verifier = param(&open_url, "code_challenge");
    let ok = http_get(53693, &format!("/callback?code=abc&state={state}"));
    assert!(ok.contains("MyAnimeList is connected to"), "{ok}");
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::TrackerConnected {
                tracker: Tracker::Mal,
                ..
            }
        ),
        "{done:?}"
    );

    let form = token_form(&http);
    assert!(
        form.iter().all(|(k, _)| k != "client_secret"),
        "a public client sent a secret field: {form:?}"
    );
    assert!(
        form.contains(&("client_id".into(), "mid".into())),
        "{form:?}"
    );
    assert!(
        form.contains(&("code_verifier".into(), verifier)),
        "{form:?}"
    );
    match core.call(Call::GetTrackers).unwrap() {
        Reply::Trackers { state } => {
            assert!(state.mal.connected);
            assert!(!state.mal.has_client_secret);
        }
        other => panic!("{other:?}"),
    }
}

/// An app registered as "web" is confidential, and MAL answers a request
/// without its secret with `invalid_client`. That is the one refusal a
/// user can fix themselves, so the job fails saying how.
#[test]
fn mal_invalid_client_without_a_secret_asks_for_the_secret() {
    if accounts::bundled_client_secret(Tracker::Mal).is_some() {
        return;
    }
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_oauth_port(53694);
    core.call(Call::SetTrackerCredentials {
        tracker: Tracker::Mal,
        client_id: "mid".into(),
        client_secret: None,
    })
    .unwrap();
    http.push_for(
        "oauth2/token",
        401,
        serde_json::json!({ "error": "invalid_client", "message": "Client authentication failed" })
            .to_string(),
    );

    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Mal,
        })
        .unwrap(),
    );
    let open_url = ready_url(&c, job);
    let state = param(&open_url, "state");
    // The tab is answered once the exchange has an outcome, so it says what
    // the app will say rather than "connected".
    let page = http_get(53694, &format!("/callback?code=abc&state={state}"));
    assert!(page.starts_with("HTTP/1.1 400"), "{page}");
    assert!(page.contains("client secret"), "{page}");
    assert!(page.contains("try again"), "{page}");
    let failed = common::wait_job(&c, job);
    match failed.body {
        EventBody::JobFailed {
            error:
                CoreError::Provider {
                    provider: Provider::Mal,
                    status: Some(401),
                    ref message,
                    ..
                },
        } => {
            assert!(message.contains("client secret"), "{message}");
            assert!(message.contains("App Type \"other\""), "{message}");
        }
        other => panic!("{other:?}"),
    }
    assert!(token_form(&http).iter().all(|(k, _)| k != "client_secret"));
}

/// A token AniList will not accept fails the job, and the tab that
/// brought it is told so instead of being shown a success.
#[test]
fn an_anilist_token_that_fails_validation_gets_the_error_page() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_oauth_port(53696);
    core.call(Call::SetTrackerCredentials {
        tracker: Tracker::Anilist,
        client_id: "123".into(),
        client_secret: None,
    })
    .unwrap();
    http.push_json(
        400,
        serde_json::json!({ "errors": [{ "message": "Invalid token", "status": 400 }], "data": null }),
    );
    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Anilist,
        })
        .unwrap(),
    );
    ready_url(&c, job);
    let page = http_get(53696, "/callback?access_token=bogus&expires_in=60");
    assert!(page.starts_with("HTTP/1.1 400"), "{page}");
    assert!(!page.contains("is connected to"), "{page}");
    assert!(page.contains("try again"), "{page}");
    let failed = common::wait_job(&c, job);
    assert!(
        matches!(failed.body, EventBody::JobFailed { .. }),
        "{failed:?}"
    );
    match core.call(Call::GetTrackers).unwrap() {
        Reply::Trackers { state } => assert!(!state.anilist.connected),
        other => panic!("{other:?}"),
    }
}

/// A sign-in the provider keeps waiting does not hold the browser forever:
/// past the hold the tab gets the "still finishing" page, and the sign-in
/// carries on to the outcome the app reports.
#[test]
fn a_slow_sign_in_lets_the_tab_go_and_still_finishes() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_oauth_port(53697);
    core.set_sign_in_hold(Duration::from_millis(200));
    core.call(Call::SetTrackerCredentials {
        tracker: Tracker::Anilist,
        client_id: "123".into(),
        client_secret: None,
    })
    .unwrap();
    // The validation is rate limited once, for a second: five holds' worth.
    http.push_with_headers(429, "{}", vec![("Retry-After".into(), "1".into())]);
    http.push_json(
        200,
        serde_json::json!({ "data": { "Viewer": { "id": 42, "name": "bandit" } } }),
    );
    http.push_json(
        200,
        serde_json::json!({ "data": { "MediaListCollection": { "lists": [] } } }),
    );
    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Anilist,
        })
        .unwrap(),
    );
    ready_url(&c, job);
    let page = http_get(53697, "/callback?access_token=tok&expires_in=3600");
    assert!(page.starts_with("HTTP/1.1 200"), "{page}");
    assert!(page.contains("Still finishing"), "{page}");
    assert!(!page.contains("is connected to"), "{page}");
    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::TrackerConnected { .. }),
        "{done:?}"
    );
}

/// A sign-in cancelled while the tab is held answers the tab with the
/// cancelled page rather than dropping the connection on it.
#[test]
fn a_sign_in_cancelled_while_held_tells_the_tab() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_oauth_port(53698);
    core.call(Call::SetTrackerCredentials {
        tracker: Tracker::Anilist,
        client_id: "123".into(),
        client_secret: None,
    })
    .unwrap();
    // Long enough that the cancel lands while the validation waits on it.
    http.push_with_headers(429, "{}", vec![("Retry-After".into(), "5".into())]);
    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Anilist,
        })
        .unwrap(),
    );
    ready_url(&c, job);
    let browser =
        std::thread::spawn(|| http_get(53698, "/callback?access_token=tok&expires_in=3600"));
    // The rate-limited validation is the only request, and nothing emits an
    // event when it is made, so this watches the fake's own log.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while http.requests().is_empty() {
        assert!(
            std::time::Instant::now() < deadline,
            "the validation never went out"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    core.call(Call::CancelJob { job }).unwrap();
    let page = browser.join().unwrap();
    assert!(page.contains("This sign-in was cancelled"), "{page}");
    let done = common::wait_job(&c, job);
    assert!(matches!(done.body, EventBody::JobCancelled), "{done:?}");
}

/// The same refusal with a secret sent is not about a missing secret, so
/// it is passed on as MAL gave it rather than asking for one.
#[test]
fn mal_invalid_client_with_a_secret_is_passed_on() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.set_oauth_port(53695);
    core.call(Call::SetTrackerCredentials {
        tracker: Tracker::Mal,
        client_id: "mid".into(),
        client_secret: Some("wrong".into()),
    })
    .unwrap();
    http.push_for(
        "oauth2/token",
        401,
        serde_json::json!({ "error": "invalid_client" }).to_string(),
    );
    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Mal,
        })
        .unwrap(),
    );
    let open_url = ready_url(&c, job);
    let state = param(&open_url, "state");
    http_get(53695, &format!("/callback?code=abc&state={state}"));
    match common::wait_job(&c, job).body {
        EventBody::JobFailed {
            error: CoreError::Provider { ref message, .. },
        } => assert!(!message.contains("App Type"), "{message}"),
        other => panic!("{other:?}"),
    }
    assert!(token_form(&http).contains(&("client_secret".into(), "wrong".into())));
}

/// A build's bundled id is all either tracker needs to sign in with one
/// click: AniList has no secret, and MAL's is optional. Whatever this
/// build bundles, the flag follows the id alone, and a secret is only
/// reported when one exists.
#[test]
fn bundled_credentials_follow_the_bundled_id_alone() {
    let (_dir, core, _c) = common::open_core();
    match core.call(Call::GetTrackers).unwrap() {
        Reply::Trackers { state } => {
            assert_eq!(
                state.anilist.bundled_credentials,
                accounts::bundled_client_id(Tracker::Anilist).is_some()
            );
            assert_eq!(
                state.mal.bundled_credentials,
                accounts::bundled_client_id(Tracker::Mal).is_some()
            );
            assert_eq!(
                state.mal.has_client_secret,
                accounts::bundled_client_secret(Tracker::Mal).is_some()
            );
            assert!(!state.anilist.has_client_secret);
            assert_eq!(
                state.mal.client_id,
                accounts::bundled_client_id(Tracker::Mal).unwrap_or_default()
            );
        }
        other => panic!("{other:?}"),
    }
}

/// The secret's three answers: absent keeps it, so re-saving the id does
/// not mean pasting it again; empty removes it; a value replaces it.
#[test]
fn an_absent_secret_keeps_and_an_empty_one_removes() {
    // A bundled secret would still report as present after the removal.
    if accounts::bundled_client_secret(Tracker::Mal).is_some() {
        return;
    }
    let (_dir, core, _c) = common::open_core();
    let save = |secret: Option<&str>| {
        core.call(Call::SetTrackerCredentials {
            tracker: Tracker::Mal,
            client_id: "mid".into(),
            client_secret: secret.map(str::to_string),
        })
        .unwrap();
        match core.call(Call::GetTrackers).unwrap() {
            Reply::Trackers { state } => state.mal.has_client_secret,
            other => panic!("{other:?}"),
        }
    };
    assert!(save(Some("sec")));
    assert!(save(None), "re-saving the id dropped the secret");
    assert!(!save(Some("")), "an empty secret did not remove it");
    assert!(!save(None));
    assert!(save(Some("  again  ")));
    assert!(!save(Some("   ")));
}

/// A connect whose port is still held waits for it instead of failing at
/// once. That is not a hypothetical: a new connect cancels the flow in
/// flight, and that flow's listener is dropped by its own task, so a
/// double click reaches the bind before the port is free. Here the test
/// plays the part of the flow that has not let go yet.
#[test]
fn a_connect_waits_for_a_port_that_is_still_held() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let port = 53692;
    core.set_oauth_port(port);
    core.call(Call::SetTrackerCredentials {
        tracker: Tracker::Anilist,
        client_id: "123".into(),
        client_secret: None,
    })
    .unwrap();

    let held = std::net::TcpListener::bind(("127.0.0.1", port)).unwrap();
    let job = started(
        core.call(Call::ConnectTracker {
            tracker: Tracker::Anilist,
        })
        .unwrap(),
    );
    // Long enough that the first bind has certainly been refused.
    std::thread::sleep(Duration::from_millis(300));
    drop(held);

    let ready = common::wait_for(
        &c,
        |e| {
            matches!(e.body, EventBody::AuthUrlReady { .. })
                && e.job.as_ref().is_some_and(|j| j.id == job)
        },
        Duration::from_secs(5),
    );
    assert!(matches!(ready.body, EventBody::AuthUrlReady { .. }));
    core.shutdown();
}
