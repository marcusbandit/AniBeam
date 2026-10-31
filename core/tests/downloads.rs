//! The downloads domain end to end, against `FakeHttp`: the settings and
//! their secrets, the sidecar that `ConnectDownloads` starts, a search and
//! a refine through anirss, and the torrent client's list read. The one
//! case that needs a real process, the sidecar, runs a fake `anirss` from a
//! temp `PATH`. Every sidecar case lives in one test because `PATH` and
//! `HOME` are process-wide and this binary runs its tests side by side.
//! The fakes are shell scripts, so the file is Unix only.
#![cfg(unix)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

use anibeam_core::net::FakeHttp;
use anibeam_core::*;
use serde_json::{Value, json};

/// The settings every case saves: a sidecar off the given anirss command,
/// qBittorrent at localhost:9090.
fn settings(anirss_command: &str) -> DownloadSettings {
    DownloadSettings {
        anirss_mode: AnirssMode::Sidecar,
        anirss_command: anirss_command.to_string(),
        jackett_command: String::new(),
        anirss_url: "http://127.0.0.1:8787".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_mode: TorrentMode::Existing,
        torrent_url: "http://localhost:9090".to_string(),
        torrent_username: "me".to_string(),
        has_torrent_password: false,
        saved_clients: Vec::new(),
        active_client: String::new(),
        prune_finished_feeds: true,
        rolling: Default::default(),
        engine: Default::default(),
    }
}

/// Points the XDG config and state homes at `dir`, so `settings::defaults()`
/// and the saved qBittorrent password never read the machine's own anirss
/// files. Every test calls it before its first call. The variables are
/// process-wide, but each test points them at an empty directory of its
/// own, and a missing file reads the same as an empty directory.
fn isolate_xdg(dir: &Path) {
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", dir);
        std::env::set_var("XDG_STATE_HOME", dir);
    }
}

fn downloads_status(core: &Core) -> DownloadsStatus {
    match core.call(Call::GetDownloads).unwrap() {
        Reply::Downloads { status } => status,
        other => panic!("{other:?}"),
    }
}

fn save(core: &Core, settings: DownloadSettings, token: Option<&str>, password: Option<&str>) {
    let reply = core
        .call(Call::SetDownloadSettings {
            settings,
            anirss_token: token.map(str::to_string),
            torrent_password: password.map(str::to_string),
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
}

fn started(core: &Core, call: Call) -> u64 {
    match core.call(call).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

/// One release as anirss's feed reads give it. A `null` count is a feed
/// that did not carry it.
fn release_json(n: u32, seeders: Value) -> Value {
    json!({
        "title": format!("[SubsPlease] Sousou no Frieren - {n:02} (1080p)"),
        "link": format!("magnet:?xt=urn:btih:{n:040}"),
        "seeders": seeders,
        "source": "nyaa",
        "resolution": "1080p"
    })
}

fn release(n: u32, seeders: Value) -> Release {
    serde_json::from_value(release_json(n, seeders)).unwrap()
}

/// The fake `anirss`: writes its arguments, the qBittorrent it was told to
/// use and its token into `capture`, then stays up the way a server would.
/// The token is written last, so a reader that sees it knows the rest is
/// already there.
fn fake_anirss(capture: &Path) -> String {
    format!(
        "#!/bin/sh\necho \"$*\" > '{args}'\n\
         printf '%s|%s|%s' \"$ANIRSS_QBT_URL\" \"$ANIRSS_QBT_USERNAME\" \"$ANIRSS_QBT_PASSWORD\" > '{qbt}'\n\
         printf '%s' \"$ANIRSS_API_TOKEN\" > '{token}'\nexec sleep 300\n",
        args = capture.join("args").display(),
        qbt = capture.join("qbt").display(),
        token = capture.join("token").display(),
    )
}

/// What the fake wrote into `capture`, once it has written it. The sidecar
/// is a child process, and FakeHttp answers its health check without the
/// script having run, so the file may not exist yet when the job finishes.
fn captured(capture: &Path, name: &str) -> String {
    let path = capture.join(name);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match std::fs::read_to_string(&path) {
            Ok(text) if !text.is_empty() => return text,
            _ => {}
        }
        assert!(
            Instant::now() < deadline,
            "the fake anirss never wrote {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn a_fresh_core_reports_anirss_off_and_no_torrent_password() {
    let (dir, core, _c) = common::open_core();
    isolate_xdg(dir.path());

    let status = downloads_status(&core);
    assert_eq!(status.anirss.state, AnirssState::Off);
    assert!(!status.settings.has_torrent_password);
    assert!(!status.settings.has_anirss_token);
}

#[test]
fn saving_the_settings_keeps_the_torrent_password_until_it_is_cleared() {
    let (dir, core, c) = common::open_core();
    isolate_xdg(dir.path());

    let mut s = settings("anirss");
    s.torrent_url = "http://localhost:9090/".to_string();
    save(&core, s.clone(), None, Some("pw"));
    common::wait_for(
        &c,
        |e| {
            matches!(&e.body, EventBody::DownloadsStatusChanged { status }
                if status.settings.has_torrent_password)
        },
        Duration::from_secs(10),
    );
    let status = downloads_status(&core);
    assert!(status.settings.has_torrent_password);
    // The trailing slash is trimmed on the way in.
    assert_eq!(status.settings.torrent_url, "http://localhost:9090");

    // `None` keeps what is stored.
    save(&core, s.clone(), None, None);
    assert!(downloads_status(&core).settings.has_torrent_password);

    // `Some("")` removes it.
    save(&core, s, None, Some(""));
    assert!(!downloads_status(&core).settings.has_torrent_password);
}

#[test]
fn a_torrent_url_without_a_scheme_is_refused_as_invalid() {
    let (dir, core, _c) = common::open_core();
    isolate_xdg(dir.path());

    let mut s = settings("anirss");
    s.torrent_url = "localhost:9090".to_string();
    let err = core
        .call(Call::SetDownloadSettings {
            settings: s,
            anirss_token: None,
            torrent_password: None,
        })
        .unwrap_err();
    assert!(
        matches!(&err, CoreError::Invalid { field, .. } if field == "torrent_url"),
        "{err:?}"
    );
    // Refused before anything is written.
    assert_ne!(
        downloads_status(&core).settings.torrent_url,
        "localhost:9090"
    );
}

/// The sidecar cases, in order: it starts and both halves read; a search
/// goes through it; a command that is not installed finishes the job as
/// `Missing`. One function, because `PATH` and `HOME` are edited here for
/// the whole of it.
#[test]
fn the_sidecar_connects_searches_and_reports_an_old_or_missing_command() {
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let anirss_path = bin.path().join("anirss");
    std::fs::write(&anirss_path, fake_anirss(capture.path())).unwrap();
    std::fs::set_permissions(&anirss_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    let original_path = std::env::var("PATH").unwrap_or_default();
    let original_home = std::env::var("HOME").ok();
    // HOME is redirected as well as PATH: the core puts `~/.local/bin` ahead
    // of PATH when it is missing, so a real anirss installed there would
    // win over the fake.
    unsafe {
        std::env::set_var("HOME", home.path());
        std::env::set_var(
            "PATH",
            format!("{}:{}", bin.path().display(), original_path),
        );
    }

    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());

    // The sidecar starts and both halves read. Health is asked twice: once
    // by the readiness poll, once by the read that follows it.
    save(&core, settings("anirss"), None, None);
    http.push_for(
        "/health",
        200,
        json!({"ok": true, "version": "0.5.0", "features": ["qbt_clients", "qbt_env_session"]})
            .to_string(),
    );
    http.push_for(
        "/health",
        200,
        json!({"ok": true, "version": "0.5.0", "features": ["qbt_clients", "qbt_env_session"]})
            .to_string(),
    );
    http.push_for(
        "/endpoints",
        200,
        json!({"endpoints": [{"name": "nyaa", "kind": "nyaa", "url": "https://nyaa.si/", "client_side_refine": false}]}).to_string(),
    );
    http.push_for(
        "/qbt/status",
        200,
        json!({"url": "http://localhost:9090", "reachable": true, "authenticated": false, "login_failed": false}).to_string(),
    );
    http.push_for("app/version", 200, "v5.2.4");
    let job = started(&core, Call::ConnectDownloads);
    let done = common::wait_job(&c, job);
    let EventBody::DownloadsStatusChanged { status } = &done.body else {
        panic!("{:?}", done.body)
    };
    assert_eq!(status.anirss.state, AnirssState::Ready);
    assert_eq!(status.anirss.version, "0.5.0");
    assert_eq!(status.endpoints.len(), 1);
    assert_eq!(status.endpoints[0].name, "nyaa");
    assert!(status.torrent.connected);
    assert_eq!(status.torrent.version, "v5.2.4");
    assert_eq!(done.level, Level::Info);

    // The health requests went to the port the sidecar was started on, with
    // the bearer it was given in its environment.
    let sent = http.requests();
    let health: Vec<_> = sent
        .iter()
        .filter(|r| r.url.ends_with("/api/v1/health"))
        .collect();
    assert!(!health.is_empty());
    let port: u16 = health[0]
        .url
        .strip_prefix("http://127.0.0.1:")
        .and_then(|rest| rest.strip_suffix("/api/v1/health"))
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| panic!("health went to {}", health[0].url));
    assert_ne!(port, 0);
    let args = captured(capture.path(), "args");
    assert_eq!(args.trim_end(), format!("--serve --port {port}"));
    let token = captured(capture.path(), "token");
    for request in &health {
        assert!(
            request
                .headers
                .iter()
                .any(|(k, v)| k == "Authorization" && v == &format!("Bearer {token}")),
            "{:?}",
            request.headers
        );
    }

    // A search goes to the same sidecar: the feed's items, then the analysis.
    let items = json!([release_json(1, json!(42)), release_json(2, Value::Null)]);
    http.push_for(
        "/search",
        200,
        json!({
            "query": "Frieren 1080p",
            "endpoint": "nyaa",
            "fell_back": false,
            "tried": [{"endpoint": "nyaa", "status": "ok", "count": 2, "error": null}],
            "items": items.clone()
        })
        .to_string(),
    );
    http.push_for(
        "/refine/analyze",
        200,
        json!({
            "query": "Frieren 1080p",
            "client_side": false,
            "auto_resolution": {"applied": false},
            "items": items.clone(),
            "groups": [{"label": "SubsPlease", "tokens": ["[SubsPlease]"], "member_count": 1, "has_poster": false, "members": [0]}],
            "best_fit": {"available": true, "query": "Frieren 1080p [SubsPlease]", "item": release_json(1, json!(42)), "profile": ["[SubsPlease]"]}
        })
        .to_string(),
    );
    let job = started(
        &core,
        Call::SearchReleases {
            query: "Frieren 1080p".to_string(),
            endpoint: None,
        },
    );
    let done = common::wait_job(&c, job);
    let EventBody::ReleasesFound { search } = &done.body else {
        panic!("{:?}", done.body)
    };
    assert_eq!(search.endpoint, "nyaa");
    assert_eq!(search.items.len(), 2);
    assert_eq!(search.items[0].seeders, Some(42));
    assert_eq!(search.items[1].seeders, None);
    assert_eq!(search.groups.len(), 1);
    assert!(search.best_fit.available);

    // An anirss from before the server mode: `--serve` is a usage error,
    // and the status names the version and the one AniBeam needs.
    let old_path = bin.path().join("anirss-old");
    std::fs::write(
        &old_path,
        "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'anirss 0.3.4'; exit 0; fi\n\
         echo 'anirss: error: unrecognized arguments: --serve' >&2\nexit 2\n",
    )
    .unwrap();
    std::fs::set_permissions(&old_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    save(&core, settings("anirss-old"), None, None);
    http.push_for("app/version", 200, "v5.2.4");
    let job = started(&core, Call::ConnectDownloads);
    let done = common::wait_job(&c, job);
    let EventBody::DownloadsStatusChanged { status } = &done.body else {
        panic!("{:?}", done.body)
    };
    assert_eq!(status.anirss.state, AnirssState::Failed);
    assert!(
        status
            .anirss
            .message
            .starts_with("anirss 0.3.4 is too old for AniBeam, which needs 0.5.0"),
        "{}",
        status.anirss.message
    );

    // One that serves but does not list the feature AniBeam relies on is
    // stopped and reported, not used: it would carry one client's login to
    // another. An anirss from before 0.5.0 lists no features at all.
    save(&core, settings("anirss"), None, None);
    // One health answer: the readiness poll's, after which it is stopped
    http.push_for(
        "/health",
        200,
        json!({"ok": true, "version": "0.4.2"}).to_string(),
    );
    http.push_for("app/version", 200, "v5.2.4");
    let job = started(&core, Call::ConnectDownloads);
    let done = common::wait_job(&c, job);
    let EventBody::DownloadsStatusChanged { status } = &done.body else {
        panic!("{:?}", done.body)
    };
    assert_eq!(status.anirss.state, AnirssState::Failed);
    assert!(
        status
            .anirss
            .message
            .starts_with("anirss 0.4.2 cannot keep each torrent client's login to itself"),
        "{}",
        status.anirss.message
    );

    // The feature is what counts, not the version: a fork or a checkout that
    // reports an older one but lists it is used.
    save(&core, settings("anirss"), None, None);
    for _ in 0..2 {
        http.push_for(
            "/health",
            200,
            json!({"ok": true, "version": "0.4.2", "features": ["qbt_env_session"]}).to_string(),
        );
    }
    http.push_for("/endpoints", 200, json!({"endpoints": []}).to_string());
    http.push_for(
        "/qbt/status",
        200,
        json!({"url": "http://localhost:9090", "reachable": true, "authenticated": false, "login_failed": false}).to_string(),
    );
    http.push_for("app/version", 200, "v5.2.4");
    let job = started(&core, Call::ConnectDownloads);
    let done = common::wait_job(&c, job);
    let EventBody::DownloadsStatusChanged { status } = &done.body else {
        panic!("{:?}", done.body)
    };
    assert_eq!(
        status.anirss.state,
        AnirssState::Ready,
        "{}",
        status.anirss.message
    );
    assert_eq!(status.anirss.version, "0.4.2");

    // A command the machine does not have: the job still finishes. The
    // status says Missing and why, and the torrent half connects anyway.
    save(&core, settings("anirss-definitely-not-here"), None, None);
    http.push_for("app/version", 200, "v5.2.4");
    let job = started(&core, Call::ConnectDownloads);
    let done = common::wait_job(&c, job);
    let EventBody::DownloadsStatusChanged { status } = &done.body else {
        panic!("{:?}", done.body)
    };
    assert_eq!(status.anirss.state, AnirssState::Missing);
    assert!(status.anirss.message.contains("anirss-definitely-not-here"));
    assert!(status.torrent.connected);
    assert_eq!(done.level, Level::Warn);

    // Switching clients starts anirss again on the one now in use, with that
    // client's own address, user and password in its environment.
    let switched = tempfile::tempdir().unwrap();
    let switch_path = bin.path().join("anirss-switch");
    std::fs::write(&switch_path, fake_anirss(switched.path())).unwrap();
    std::fs::set_permissions(&switch_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut s = settings("anirss-switch");
    s.saved_clients = vec![saved("c2", "local", "http://localhost:8080", "admin")];
    save(&core, s, None, Some("seedbox-pw"));
    assert_eq!(
        core.call(Call::UseTorrentClient {
            id: Some("c2".to_string())
        })
        .unwrap(),
        Reply::Ok
    );
    save(
        &core,
        downloads_status(&core).settings,
        None,
        Some("local-pw"),
    );
    for _ in 0..2 {
        http.push_for(
            "/health",
            200,
            json!({"ok": true, "version": "0.5.0", "features": ["qbt_clients", "qbt_env_session"]})
                .to_string(),
        );
    }
    http.push_for("/endpoints", 200, json!({"endpoints": []}).to_string());
    http.push_for(
        "/qbt/status",
        200,
        json!({"url": "http://localhost:8080", "reachable": true, "authenticated": false, "login_failed": false}).to_string(),
    );
    http.push_for("app/version", 200, "v5.2.4");
    let job = started(&core, Call::ConnectDownloads);
    let done = common::wait_job(&c, job);
    let EventBody::DownloadsStatusChanged { status } = &done.body else {
        panic!("{:?}", done.body)
    };
    assert_eq!(status.anirss.state, AnirssState::Ready);
    captured(switched.path(), "token");
    assert_eq!(
        std::fs::read_to_string(switched.path().join("qbt")).unwrap(),
        "http://localhost:8080|admin|local-pw"
    );

    core.shutdown();
    unsafe {
        match &original_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        std::env::set_var("PATH", &original_path);
    }
}

fn saved(id: &str, name: &str, url: &str, username: &str) -> SavedTorrentClient {
    SavedTorrentClient {
        id: id.to_string(),
        name: name.to_string(),
        url: url.to_string(),
        username: username.to_string(),
        has_password: false,
    }
}

fn use_client(core: &Core, id: Option<&str>) -> Result<Reply, CoreError> {
    core.call(Call::UseTorrentClient {
        id: id.map(str::to_string),
    })
}

#[test]
fn saved_clients_switch_and_keep_their_own_passwords() {
    let (dir, core, _c) = common::open_core();
    isolate_xdg(dir.path());

    // The one client a settings save names becomes the first saved one, and
    // its password goes where a single-client install always kept it.
    let mut s = settings("anirss");
    s.torrent_url = "http://192.0.2.10:8080".to_string();
    s.torrent_username = "me".to_string();
    save(&core, s, None, Some("seedbox-pw"));
    let st = downloads_status(&core).settings;
    assert_eq!(st.active_client, "default");
    assert_eq!(st.saved_clients.len(), 1);
    assert_eq!(st.saved_clients[0].name, "192.0.2.10:8080");
    assert!(st.saved_clients[0].has_password && st.has_torrent_password);

    // A second one saves beside it, with no password yet.
    let mut s = st.clone();
    s.saved_clients
        .push(saved("", "This Mac", "http://localhost:8080", "admin"));
    save(&core, s, None, None);
    let st = downloads_status(&core).settings;
    assert_eq!(st.saved_clients[1].id, "c2");
    assert!(!st.saved_clients[1].has_password);

    // Using it moves the address, the user and the password flag with it.
    assert_eq!(use_client(&core, Some("c2")).unwrap(), Reply::Ok);
    let st = downloads_status(&core).settings;
    assert_eq!(st.torrent_mode, TorrentMode::Existing);
    assert_eq!(st.active_client, "c2");
    assert_eq!(st.torrent_url, "http://localhost:8080");
    assert_eq!(st.torrent_username, "admin");
    assert!(!st.has_torrent_password);

    // Its own password, saved while it is in use, leaves the other's alone.
    save(&core, st, None, Some("local-pw"));
    let st = downloads_status(&core).settings;
    assert!(st.saved_clients.iter().all(|c| c.has_password));
    assert_eq!(use_client(&core, Some("default")).unwrap(), Reply::Ok);
    let st = downloads_status(&core).settings;
    assert_eq!(st.torrent_url, "http://192.0.2.10:8080");
    assert_eq!(st.torrent_username, "me");
    assert!(st.has_torrent_password);

    // Taking one off the list takes its password: added back under the same
    // id, it has none.
    let mut s = st.clone();
    s.saved_clients.retain(|c| c.id != "c2");
    save(&core, s, None, None);
    let mut s = downloads_status(&core).settings;
    assert_eq!(s.saved_clients.len(), 1);
    s.saved_clients
        .push(saved("c2", "This Mac", "http://localhost:8080", "admin"));
    save(&core, s, None, None);
    let st = downloads_status(&core).settings;
    assert!(!st.saved_clients[1].has_password);
    assert!(st.saved_clients[0].has_password);

    // An unknown id changes nothing.
    assert!(matches!(
        use_client(&core, Some("nas")),
        Err(CoreError::Invalid { .. })
    ));
    assert_eq!(downloads_status(&core).settings.active_client, "default");

    // Two clients by one name is refused.
    let mut s = st.clone();
    s.saved_clients[1].name = "192.0.2.10:8080".to_string();
    let refused = core.call(Call::SetDownloadSettings {
        settings: s,
        anirss_token: None,
        torrent_password: None,
    });
    assert!(matches!(refused, Err(CoreError::Invalid { field, .. }) if field == "saved_clients"));
}

#[test]
fn a_removed_clients_password_never_passes_to_a_new_one() {
    let (dir, core, _c) = common::open_core();
    isolate_xdg(dir.path());
    let mut s = settings("anirss");
    s.torrent_url = "http://192.0.2.10:8080".to_string();
    save(&core, s, None, Some("seedbox-pw"));
    let mut s = downloads_status(&core).settings;
    s.saved_clients
        .push(saved("c2", "NAS", "http://192.0.2.20:8080", "admin"));
    s.active_client = "c2".to_string();
    s.torrent_url = "http://192.0.2.20:8080".to_string();
    save(&core, s, None, Some("nas-pw"));

    // One save that takes c2 off and adds a client without an id: the new
    // one is not c2 again, so it has no password of its own yet.
    let mut s = downloads_status(&core).settings;
    s.saved_clients.retain(|c| c.id != "c2");
    s.saved_clients
        .push(saved("", "Other", "http://192.0.2.30:8080", "admin"));
    s.active_client = "default".to_string();
    s.torrent_url = "http://192.0.2.10:8080".to_string();
    save(&core, s, None, None);
    let st = downloads_status(&core).settings;
    let other = st.saved_clients.iter().find(|c| c.name == "Other").unwrap();
    assert_ne!(other.id, "c2");
    assert!(!other.has_password);
}

#[test]
fn anibeams_own_never_starts_at_a_local_clients_port() {
    let (dir, core, _c) = common::open_core();
    isolate_xdg(dir.path());
    let mut s = settings("anirss");
    s.torrent_url = "http://localhost:8080".to_string();
    save(&core, s, None, None);

    // Picking AniBeam's own in Settings while the address is still the
    // user's own qBittorrent on this machine moves it off that port.
    let mut s = downloads_status(&core).settings;
    s.torrent_mode = TorrentMode::Managed;
    save(&core, s, None, None);
    let st = downloads_status(&core).settings;
    assert_eq!(st.torrent_mode, TorrentMode::Managed);
    assert!(
        st.torrent_url.starts_with("http://127.0.0.1:") && !st.torrent_url.ends_with(":8080"),
        "{}",
        st.torrent_url
    );
    assert_eq!(st.saved_clients[0].url, "http://localhost:8080");
}

#[test]
fn picking_the_client_in_use_keeps_it() {
    let (dir, core, _c) = common::open_core();
    isolate_xdg(dir.path());
    let mut s = settings("anirss");
    s.torrent_url = "http://192.0.2.10:8080".to_string();
    save(&core, s, None, Some("seedbox-pw"));
    let before = downloads_status(&core).settings;
    assert_eq!(use_client(&core, Some("default")).unwrap(), Reply::Ok);
    assert_eq!(downloads_status(&core).settings, before);
}

#[test]
fn anibeams_own_never_starts_at_a_remote_clients_address() {
    let (dir, core, _c) = common::open_core();
    isolate_xdg(dir.path());
    let mut s = settings("anirss");
    s.torrent_url = "http://192.0.2.10:8080".to_string();
    save(&core, s, None, Some("seedbox-pw"));

    // Switching to AniBeam's own from a client elsewhere gives it an address
    // on this machine, and its user; the saved client stays on the list.
    assert_eq!(use_client(&core, None).unwrap(), Reply::Ok);
    let st = downloads_status(&core).settings;
    assert_eq!(st.torrent_mode, TorrentMode::Managed);
    assert!(
        st.torrent_url.starts_with("http://127.0.0.1:"),
        "{}",
        st.torrent_url
    );
    assert_eq!(st.torrent_username, "anibeam");
    assert!(!st.has_torrent_password);
    assert_eq!(st.saved_clients.len(), 1);
    let own = st.torrent_url.clone();

    // Back to the saved one, and its password is still there.
    assert_eq!(use_client(&core, Some("default")).unwrap(), Reply::Ok);
    let st = downloads_status(&core).settings;
    assert_eq!(st.torrent_url, "http://192.0.2.10:8080");
    assert!(st.has_torrent_password);

    // A settings save that picks AniBeam's own with the remote address still
    // in the field gets the same treatment; a loopback one is kept.
    let mut s = st.clone();
    s.torrent_mode = TorrentMode::Managed;
    save(&core, s, None, None);
    let st = downloads_status(&core).settings;
    assert!(
        st.torrent_url.starts_with("http://127.0.0.1:"),
        "{}",
        st.torrent_url
    );
    let mut s = st.clone();
    s.torrent_url = own.clone();
    save(&core, s, None, None);
    assert_eq!(downloads_status(&core).settings.torrent_url, own);
}

#[test]
fn list_torrents_emits_the_torrents_and_the_transfer_totals() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    // The user's own client: a fresh install would probe for one first.
    save(&core, settings("anirss"), None, None);

    http.push_for(
        "torrents/info",
        200,
        json!([{
            "hash": "a1b2c3",
            "name": "[SubsPlease] Sousou no Frieren - 01",
            "state": "downloading",
            "progress": 0.25,
            "size": 1500000000u64,
            "dlspeed": 1200000,
            "upspeed": 0,
            "eta": 600,
            "num_seeds": 12,
            "num_leechs": 3,
            "ratio": 0.0,
            "save_path": "/downloads/frieren",
            "content_path": "/downloads/frieren/01.mkv",
            "added_on": 1700000000,
            "completion_on": -1,
            "tags": "anime",
            "category": ""
        }])
        .to_string(),
    );
    http.push_for(
        "transfer/info",
        200,
        json!({"dl_info_speed": 1200, "up_info_speed": 0, "connection_status": "connected", "dht_nodes": 10}).to_string(),
    );

    assert_eq!(core.call(Call::ListTorrents).unwrap(), Reply::Ok);
    let event = common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::TorrentsRead { .. }),
        Duration::from_secs(10),
    );
    let EventBody::TorrentsRead {
        torrents, transfer, ..
    } = &event.body
    else {
        panic!("{:?}", event.body)
    };
    assert_eq!(torrents.len(), 1);
    assert_eq!(torrents[0].hash, "a1b2c3");
    assert_eq!(torrents[0].state, "downloading");
    assert_eq!(torrents[0].progress, 0.25);
    assert_eq!(torrents[0].dlspeed, 1_200_000);
    assert_eq!(transfer.dl_speed, 1200);
    assert_eq!(transfer.connection_status, "connected");
    assert_eq!(transfer.dht_nodes, 10);
    assert!(downloads_status(&core).torrent.connected);
}

#[test]
fn a_refine_that_changes_nothing_ends_with_applied_false() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());

    // Remote mode: the core talks to a server somebody else runs, so there
    // is no sidecar to start and no PATH to set up.
    let mut s = settings("anirss");
    s.anirss_mode = AnirssMode::Remote;
    s.anirss_url = "http://anirss.test:8787/".to_string();
    save(&core, s, Some("tok"), None);

    http.push_for(
        "/refine/apply",
        200,
        json!({
            "applied": false,
            "reason": "noop",
            "mode": "client",
            "query": "Frieren 1080p",
            "endpoint": "nyaa",
            "items": [release_json(1, json!(42))],
            "previous_count": 1,
            "delta": 0,
            "groups": [],
            "best_fit": {"available": false, "query": "", "item": null, "profile": []}
        })
        .to_string(),
    );
    let job = started(
        &core,
        Call::RefineReleases {
            query: "Frieren 1080p".to_string(),
            endpoint: "nyaa".to_string(),
            items: vec![release(1, json!(42))],
            op: RefineOp::Tokens {
                tokens: vec!["[SubsPlease]".to_string()],
            },
        },
    );
    let done = common::wait_job(&c, job);
    let EventBody::ReleasesRefined { refinement } = &done.body else {
        panic!("{:?}", done.body)
    };
    assert!(!refinement.applied);
    assert_eq!(refinement.reason.as_deref(), Some("noop"));
    assert_eq!(refinement.items.len(), 1);
    assert_eq!(
        http.requests()[0].url,
        "http://anirss.test:8787/api/v1/refine/apply"
    );
}

/// A torrent saved under another mount than the library's still finds the
/// series its folder fills, by the folder's name, and the series travels
/// beside the list once.
#[test]
fn a_torrent_is_linked_to_the_library_series_its_folder_fills() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    // The user's own client: a fresh install would probe for one first.
    save(&core, settings("anirss"), None, None);
    let lib = dir.path().join("lib");
    let show = lib.join("Sousou no Frieren");
    std::fs::create_dir_all(&show).unwrap();
    for n in 1..=3 {
        std::fs::write(
            show.join(format!(
                "[SubsPlease] Sousou no Frieren - {n:02} (1080p).mkv"
            )),
            b"x",
        )
        .unwrap();
    }
    core.call(Call::AddSource {
        path: lib.to_string_lossy().into_owned(),
    })
    .unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ScanFinished { .. }),
        Duration::from_secs(10),
    );

    let torrent = |hash: &str, save: &str| {
        json!({"hash": hash, "name": hash, "state": "uploading", "progress": 1.0, "size": 1,
               "save_path": save, "content_path": format!("{save}/{hash}.mkv")})
    };
    http.push_for(
        "torrents/info",
        200,
        json!([
            torrent("aa", "/srv/media/Anime/Sousou no Frieren"),
            torrent("bb", "/srv/media/Anime/Sousou no Frieren/"),
            torrent("cc", "/srv/media/Anime/Something Else"),
        ])
        .to_string(),
    );
    http.push_for("transfer/info", 200, json!({}).to_string());

    assert_eq!(core.call(Call::ListTorrents).unwrap(), Reply::Ok);
    let event = common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::TorrentsRead { .. }),
        Duration::from_secs(10),
    );
    let EventBody::TorrentsRead {
        torrents, shows, ..
    } = &event.body
    else {
        panic!("{:?}", event.body)
    };
    assert_eq!(shows.len(), 1, "one series, named once: {shows:?}");
    let s = &shows[0];
    assert_eq!(torrents[0].series_id, Some(s.series_id));
    assert_eq!(
        torrents[1].series_id,
        Some(s.series_id),
        "a trailing slash is the same folder"
    );
    assert_eq!(torrents[2].series_id, None);
    assert_eq!(s.episodes_on_disk, 3);
    assert!(s.path.ends_with("Sousou no Frieren"));
    assert!(
        !s.complete,
        "an unmatched series has no total, so it is never complete"
    );
}

/// The feed is the core's own read: entries arrive as `FeedRead` under the
/// URL asked for, a refusal as `DownloadsUnavailable` with what "feed".
#[test]
fn read_feed_answers_with_the_entries_or_says_the_feed_is_unavailable() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    let url = "https://nyaa.si/?page=rss&q=Frieren";
    http.push_for(
        "nyaa.si",
        200,
        r#"<rss xmlns:nyaa="https://nyaa.si/xmlns/nyaa"><channel><item>
             <title>[SubsPlease] Sousou no Frieren - 01 (1080p)</title>
             <link>https://nyaa.si/download/1.torrent</link>
             <guid>https://nyaa.si/view/1</guid>
             <nyaa:seeders>9</nyaa:seeders>
             <nyaa:infoHash>ABCDEF0123456789ABCDEF0123456789ABCDEF01</nyaa:infoHash>
           </item></channel></rss>"#,
    );
    assert_eq!(
        core.call(Call::ReadFeed {
            url: url.to_string()
        })
        .unwrap(),
        Reply::Ok
    );
    let event = common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::FeedRead { .. }),
        Duration::from_secs(10),
    );
    let EventBody::FeedRead { url: read, entries } = &event.body else {
        panic!("{:?}", event.body)
    };
    assert_eq!(read, url);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].seeders, Some(9));
    assert_eq!(
        entries[0].info_hash,
        "abcdef0123456789abcdef0123456789abcdef01"
    );
    assert_eq!(entries[0].page, "https://nyaa.si/view/1");

    http.push_for("nyaa.si", 502, "bad gateway");
    core.call(Call::ReadFeed {
        url: url.to_string(),
    })
    .unwrap();
    let event = common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::DownloadsUnavailable { .. }),
        Duration::from_secs(10),
    );
    let EventBody::DownloadsUnavailable { what, error } = &event.body else {
        panic!("{:?}", event.body)
    };
    assert_eq!(what, "feed");
    assert!(
        matches!(error, CoreError::Service { code, .. } if code == "http_502"),
        "{error:?}"
    );

    assert!(matches!(
        core.call(Call::ReadFeed {
            url: "file:///etc/passwd".to_string()
        }),
        Err(CoreError::Invalid { .. })
    ));
}

/// Remote mode with a token, so the core reaches anirss through `FakeHttp`
/// with no sidecar to start.
fn remote() -> DownloadSettings {
    DownloadSettings {
        anirss_mode: AnirssMode::Remote,
        anirss_url: "http://anirss.test".to_string(),
        ..settings("anirss")
    }
}

fn form_of(r: &anibeam_core::net::HttpRequest) -> String {
    match &r.body {
        Some(anibeam_core::net::Body::Form(pairs)) => pairs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&"),
        Some(anibeam_core::net::Body::Json(v)) => v.to_string(),
        None => String::new(),
    }
}

/// Unsubscribing with the torrents ticked: the rule goes through anirss
/// with neither torrents nor files (the core takes the torrents itself),
/// and the torrents go from the client with their data kept.
#[test]
fn remove_show_unsubscribes_and_takes_the_torrents_keeping_the_files() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save(&core, remote(), Some("tok"), Some("pw"));
    http.push_for(
        "subscriptions/remove",
        200,
        json!({"dry_run": false, "plans": []}).to_string(),
    );
    http.push_for("torrents/delete", 200, "");

    let job = started(
        &core,
        Call::RemoveShow {
            title: "Frieren".to_string(),
            rule: Some("Frieren".to_string()),
            hashes: vec!["AA".to_string(), " bb ".to_string()],
            torrents: true,
            files: false,
            series: None,
        },
    );
    let event = common::wait_job(&c, job);
    let EventBody::ShowRemoved {
        rule,
        hashes,
        trashed,
        ..
    } = &event.body
    else {
        panic!("{:?}", event.body)
    };
    assert_eq!(rule.as_deref(), Some("Frieren"));
    assert_eq!(hashes, &vec!["aa".to_string(), "bb".to_string()]);
    assert!(trashed.is_empty());

    let requests = http.requests();
    let remove = requests
        .iter()
        .find(|r| r.url.contains("subscriptions/remove"))
        .expect("the rule went");
    let body = form_of(remove);
    assert!(
        body.contains("\"torrents\":false") && body.contains("\"files\":false"),
        "{body}"
    );
    let delete = requests
        .iter()
        .find(|r| r.url.contains("torrents/delete"))
        .expect("the torrents went");
    assert_eq!(form_of(delete), "hashes=aa|bb&deleteFiles=false");
}

/// A client on another machine: its files are its own, so the core trashes
/// nothing here and has the client delete them with the torrents.
#[test]
fn remove_show_leaves_a_remote_clients_files_to_the_client() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save(
        &core,
        DownloadSettings {
            torrent_url: "http://192.0.2.10:8080".to_string(),
            ..remote()
        },
        Some("tok"),
        Some("pw"),
    );
    http.push_for("torrents/delete", 200, "");

    let job = started(
        &core,
        Call::RemoveShow {
            title: "Frieren".to_string(),
            rule: None,
            hashes: vec!["aa".to_string()],
            torrents: true,
            files: true,
            series: None,
        },
    );
    let event = common::wait_job(&c, job);
    let EventBody::ShowRemoved { trashed, .. } = &event.body else {
        panic!("{:?}", event.body)
    };
    assert!(trashed.is_empty());
    let requests = http.requests();
    assert!(!requests.iter().any(|r| r.url.contains("torrents/info")));
    let delete = requests
        .iter()
        .find(|r| r.url.contains("torrents/delete"))
        .expect("the torrents went");
    assert_eq!(form_of(delete), "hashes=aa&deleteFiles=true");
}

/// A file target that holds a library folder refuses the whole removal:
/// the rule and the torrents stay, nothing is moved.
#[test]
fn remove_show_refuses_a_library_root_and_removes_nothing() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save(&core, remote(), Some("tok"), Some("pw"));
    let lib = dir.path().join("lib");
    std::fs::create_dir_all(lib.join("Frieren")).unwrap();
    core.call(Call::AddSource {
        path: lib.to_string_lossy().into_owned(),
    })
    .unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ScanFinished { .. }),
        Duration::from_secs(10),
    );

    // A torrent saved into the base with no folder of its own reports the
    // base itself as its content
    http.push_for(
        "torrents/info",
        200,
        json!([{"hash": "cc", "name": "x", "state": "uploading", "progress": 1.0,
                "save_path": dir.path().to_string_lossy(), "content_path": lib.to_string_lossy()}])
        .to_string(),
    );
    let job = started(
        &core,
        Call::RemoveShow {
            title: "Frieren".to_string(),
            rule: Some("Frieren".to_string()),
            hashes: vec!["cc".to_string()],
            torrents: true,
            files: true,
            series: None,
        },
    );
    let failed = common::wait_for(
        &c,
        |e| {
            e.job.as_ref().is_some_and(|j| j.id == job)
                && matches!(e.body, EventBody::JobFailed { .. })
        },
        Duration::from_secs(10),
    );
    let EventBody::JobFailed { error } = &failed.body else {
        panic!("{:?}", failed.body)
    };
    assert!(matches!(error, CoreError::Invalid { .. }), "{error:?}");
    let requests = http.requests();
    assert!(
        !requests
            .iter()
            .any(|r| r.url.contains("subscriptions/remove")),
        "the rule stays"
    );
    assert!(
        !requests.iter().any(|r| r.url.contains("torrents/delete")),
        "the torrents stay"
    );
    assert!(lib.join("Frieren").exists());

    assert!(matches!(
        core.call(Call::RemoveShow {
            title: String::new(),
            rule: None,
            hashes: vec![],
            torrents: true,
            files: false,
            series: None,
        }),
        Err(CoreError::Invalid { .. })
    ));
}

/// A subscription narrowed on anirss's side (1337x through Jackett): the
/// rule anirss made reads a feed that searches for every word and finds
/// nothing, so the core points it at the feed the search read and keeps
/// the narrowing words in the rule; the rule first, then the empty feed
/// out, then the search's feed in.
#[test]
fn a_locally_narrowed_subscription_reads_the_searched_feed_and_filters_in_the_rule() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save(&core, remote(), Some("tok"), Some("pw"));
    // Connect so the core knows 1337x is a generic RSS endpoint
    http.push_for(
        "/health",
        200,
        json!({"ok": true, "version": "0.5.0"}).to_string(),
    );
    http.push_for(
        "/endpoints",
        200,
        json!({"endpoints": [{"name": "1337x", "kind": "rss", "url": "http://127.0.0.1:9117/x?q={query}", "client_side_refine": true}]}).to_string(),
    );
    http.push_for(
        "/qbt/status",
        200,
        json!({"url": "http://localhost:9090", "reachable": true, "authenticated": true})
            .to_string(),
    );
    http.push_for("app/version", 200, "v5.2.4");
    let job = started(&core, Call::ConnectDownloads);
    common::wait_job(&c, job);

    let narrow = "http://127.0.0.1:9117/x?q=Helluva+Boss+S03+1080p+WEB-DL";
    let wide = "http://127.0.0.1:9117/x?q=Helluva+Boss+S03";
    http.push_for(
        "api/v1/subscriptions",
        201,
        json!({"name": "Helluva Boss", "feed_url": narrow, "save_path": "/media/TV/Helluva Boss",
               "must_contain": "", "must_not_contain": "", "tags": ["TV"]})
        .to_string(),
    );
    http.push_for(
        "subscriptions/preview",
        200,
        json!({"feed_url": wide, "must_contain": "", "must_not_contain": "", "endpoint_name": "1337x"}).to_string(),
    );
    http.push_for(
        "rss/rules",
        200,
        json!({"Helluva Boss": {"enabled": true, "mustContain": "", "mustNotContain": "", "affectedFeeds": [narrow],
                                "savePath": "/media/TV/Helluva Boss"}}).to_string(),
    );
    http.push_for("rss/setRule", 200, "");
    http.push_for(
        "rss/items",
        200,
        json!({"Helluva Boss": {"uid": "u", "url": narrow}}).to_string(),
    );
    http.push_for("rss/removeItem", 200, "");
    http.push_for("rss/addFeed", 200, "");

    let job = started(
        &core,
        Call::Subscribe {
            query: "Helluva Boss S03 1080p WEB-DL".to_string(),
            endpoint: "1337x".to_string(),
            items: vec![release(1, json!(5))],
            name: Some("Helluva Boss".to_string()),
            wire_query: Some("Helluva Boss S03".to_string()),
        },
    );
    let event = common::wait_job(&c, job);
    let EventBody::Subscribed { result } = &event.body else {
        panic!("{:?}", event.body)
    };
    assert_eq!(result.feed_url, wide);
    assert_eq!(result.must_contain, "1080p WEB-DL");

    let rss: Vec<_> = http
        .requests()
        .into_iter()
        .filter(|r| r.url.contains("/rss/"))
        .collect();
    let order: Vec<&str> = rss
        .iter()
        .map(|r| r.url.rsplit('/').next().unwrap_or(""))
        .collect();
    assert_eq!(
        order,
        vec!["rules", "setRule", "items", "removeItem", "addFeed"],
        "the rule moves before any feed does"
    );
    let set = form_of(&rss[1]);
    assert!(set.contains("\"mustContain\":\"1080p WEB-DL\""), "{set}");
    assert!(
        set.contains(&format!("\"affectedFeeds\":[\"{wide}\"]")),
        "{set}"
    );
    assert_eq!(form_of(&rss[3]), "path=Helluva Boss");
    assert_eq!(form_of(&rss[4]), format!("url={wide}&path=Helluva Boss"));
}

/// The words the endpoint can answer go back into the feed's search: the
/// feed holds what was picked, not every release of the show. A word stays
/// only when the feed still holds every picked release and gets smaller;
/// one that loses a picked release is left to the rule.
#[test]
fn a_locally_narrowed_subscription_probes_the_words_the_endpoint_answers() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save(&core, remote(), Some("tok"), Some("pw"));
    http.push_for(
        "/health",
        200,
        json!({"ok": true, "version": "0.3.4"}).to_string(),
    );
    http.push_for(
        "/endpoints",
        200,
        json!({"endpoints": [{"name": "1337x", "kind": "rss", "url": "http://127.0.0.1:9117/x?q={query}", "client_side_refine": true}]}).to_string(),
    );
    http.push_for(
        "/qbt/status",
        200,
        json!({"url": "http://localhost:9090", "reachable": true, "authenticated": true})
            .to_string(),
    );
    http.push_for("app/version", 200, "v5.2.4");
    let job = started(&core, Call::ConnectDownloads);
    common::wait_job(&c, job);

    let tbk4 = "Helluva.Boss.S03E04.A.Trial.of.Errors.1080p.AMZN.WEB-DL.H264-TBK";
    let tbk5 = "Helluva.Boss.S03E05.The.Auction.1080p.AMZN.WEB-DL.H264-TBK";
    let item =
        |t: &str, n: u32| format!("<item><title>{t}</title><link>http://j/dl/{n}</link></item>");
    let channel = |items: &[String]| format!("<rss><channel>{}</channel></rss>", items.join(""));
    let narrow = "http://127.0.0.1:9117/x?q=Helluva+Boss+S03+TBK+1080p";
    let wide = "http://127.0.0.1:9117/x?q=Helluva+Boss+S03";
    let tight = "http://127.0.0.1:9117/x?q=Helluva%20Boss%20S03%20TBK";
    http.push_for(
        "api/v1/subscriptions",
        201,
        json!({"name": "Helluva Boss", "feed_url": narrow, "save_path": "/media/TV/Helluva Boss",
               "must_contain": "", "must_not_contain": "", "tags": ["TV"]})
        .to_string(),
    );
    http.push_for(
        "subscriptions/preview",
        200,
        json!({"feed_url": wide, "must_contain": "", "must_not_contain": "", "endpoint_name": "1337x"}).to_string(),
    );
    http.push_for(
        "q=Helluva+Boss+S03",
        200,
        channel(&[
            item(tbk4, 1),
            item(tbk5, 2),
            item("Helluva.Boss.S03E04.1080p.HEVC.x265-MeGusta", 3),
            item("Hazbin Hotel Season 1 [1080p]", 4),
        ]),
    );
    http.push_for("S03%20TBK%201080p", 200, channel(&[item(tbk4, 1)]));
    http.push_for("S03%20TBK", 200, channel(&[item(tbk4, 1), item(tbk5, 2)]));
    http.push_for(
        "rss/rules",
        200,
        json!({"Helluva Boss": {"enabled": true, "mustContain": "", "mustNotContain": "", "affectedFeeds": [narrow],
                                "savePath": "/media/TV/Helluva Boss"}}).to_string(),
    );
    http.push_for("rss/setRule", 200, "");
    http.push_for(
        "rss/items",
        200,
        json!({"Helluva Boss": {"uid": "u", "url": narrow}}).to_string(),
    );
    http.push_for("rss/removeItem", 200, "");
    http.push_for("rss/addFeed", 200, "");

    let picked: Vec<Release> = [tbk4, tbk5]
        .iter()
        .map(|t| serde_json::from_value(json!({"title": t, "link": "http://j/dl/x"})).unwrap())
        .collect();
    let job = started(
        &core,
        Call::Subscribe {
            query: "Helluva Boss S03 TBK 1080p".to_string(),
            endpoint: "1337x".to_string(),
            items: picked,
            name: Some("Helluva Boss".to_string()),
            wire_query: Some("Helluva Boss S03".to_string()),
        },
    );
    let event = common::wait_job(&c, job);
    let EventBody::Subscribed { result } = &event.body else {
        panic!("{:?}", event.body)
    };
    assert_eq!(result.feed_url, tight);
    assert_eq!(result.must_contain, "TBK 1080p");
    let added = http
        .requests()
        .into_iter()
        .find(|r| r.url.contains("rss/addFeed"))
        .unwrap();
    assert_eq!(form_of(&added), format!("url={tight}&path=Helluva Boss"));
}

/// A rule whose feed finds nothing: the core shortens the feed's search
/// word by word until the endpoint answers, keeps the dropped words in the
/// rule, and swaps the feed in the order a fresh subscription does.
#[test]
fn repair_shortens_an_empty_feed_until_it_answers_and_keeps_the_words() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save(&core, remote(), Some("tok"), Some("pw"));
    let broken = "http://127.0.0.1:9117/x?apikey=K&t=search&q=Helluva.Boss.+S03+TBK";
    let empty = r#"<rss><channel><title>1337x</title></channel></rss>"#;
    let six = r#"<rss><channel>
        <item><title>Helluva.Boss.S03E01.1080p.WEB-DL-TBK</title><link>http://j/dl/1</link></item>
        <item><title>Helluva.Boss.S03E01.1080p.HEVC.x265-MeGusta</title><link>http://j/dl/2</link></item>
      </channel></rss>"#;
    http.push_for(
        "rss/rules",
        200,
        json!({"Helluva.Boss.": {"enabled": true, "mustContain": "", "mustNotContain": "", "affectedFeeds": [broken]}}).to_string(),
    );
    http.push_for("q=Helluva.Boss.+S03+TBK", 200, empty);
    http.push_for("q=Helluva.Boss.%20S03", 200, six);
    http.push_for("rss/setRule", 200, "");
    http.push_for(
        "rss/items",
        200,
        json!({"Helluva.Boss.": {"uid": "u", "url": broken}}).to_string(),
    );
    http.push_for("rss/removeItem", 200, "");
    http.push_for("rss/addFeed", 200, "");

    let job = started(
        &core,
        Call::RepairSubscription {
            name: "Helluva.Boss.".to_string(),
        },
    );
    let event = common::wait_job(&c, job);
    let EventBody::SubscriptionRepaired {
        feed_url,
        must_contain,
        found,
        ..
    } = &event.body
    else {
        panic!("{:?}", event.body)
    };
    assert_eq!(
        feed_url,
        "http://127.0.0.1:9117/x?apikey=K&t=search&q=Helluva.Boss.%20S03"
    );
    assert_eq!(must_contain, "TBK");
    assert_eq!(*found, 2);
    let rss: Vec<String> = http
        .requests()
        .into_iter()
        .filter(|r| r.url.contains("/rss/"))
        .map(|r| r.url.rsplit('/').next().unwrap_or("").to_string())
        .collect();
    assert_eq!(
        rss,
        vec!["rules", "setRule", "items", "removeItem", "addFeed"]
    );
}

/// A fresh install settles the torrent client on first use: a qBittorrent
/// that answers at the address is the user's, and with none AniBeam starts
/// its own (a fake qbittorrent-nox here) and points the settings at it.
/// Both cases live in one test because the bundled binary is named through
/// a process-wide variable.
#[test]
fn a_fresh_install_uses_the_qbittorrent_it_finds_or_starts_its_own() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    http.push_for("app/version", 200, "v5.1.2");
    http.push_for("app/version", 200, "v5.1.2");
    let job = started(&core, Call::TestTorrentClient);
    common::wait_job(&c, job);
    let status = downloads_status(&core);
    assert_eq!(status.settings.torrent_mode, TorrentMode::Existing);
    assert!(status.torrent.connected);
    core.shutdown();

    let bin = tempfile::tempdir().unwrap();
    let nox = bin.path().join("qbittorrent-nox");
    std::fs::write(
        &nox,
        format!(
            "#!/bin/sh\necho \"$*\" > '{}'\nexec sleep 5\n",
            bin.path().join("args").display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&nox, std::fs::Permissions::from_mode(0o755)).unwrap();
    unsafe { std::env::set_var("ANIBEAM_QBITTORRENT_NOX", &nox) };
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    // Nothing at the stock address, nothing on the new port until it starts
    http.fail_next("connection refused");
    http.fail_next("connection refused");
    http.push_for("app/version", 200, "v5.1.2");
    http.push_for("app/version", 200, "v5.1.2");
    let job = started(&core, Call::TestTorrentClient);
    common::wait_job(&c, job);
    let status = downloads_status(&core);
    assert_eq!(status.settings.torrent_mode, TorrentMode::Managed);
    assert_eq!(status.settings.torrent_username, "anibeam");
    assert!(status.torrent.connected, "{}", status.torrent.message);
    let port = status
        .settings
        .torrent_url
        .strip_prefix("http://127.0.0.1:")
        .expect("a loopback address")
        .to_string();
    let args = captured(bin.path(), "args");
    assert!(args.contains(&format!("--webui-port={port}")), "{args}");
    let profile = dir.path().join("data").join("qbittorrent");
    assert!(
        args.contains(&format!("--profile={}", profile.display())),
        "{args}"
    );
    // qBittorrent's own name for a custom profile's settings file
    let name = if cfg!(any(windows, target_os = "macos")) {
        "qBittorrent.ini"
    } else {
        "qBittorrent.conf"
    };
    let conf = std::fs::read_to_string(profile.join("qBittorrent/config").join(name)).unwrap();
    assert!(conf.contains(&format!("WebUI\\Port={port}\n")), "{conf}");
    assert!(conf.contains("WebUI\\LocalHostAuth=false\n"), "{conf}");

    // Asked to pick again while its own runs, it keeps its own on the same
    // port rather than taking it for the user's client.
    let mut again = status.settings.clone();
    again.torrent_mode = TorrentMode::Auto;
    save(&core, again, None, None);
    for _ in 0..4 {
        http.push_for("app/version", 200, "v5.1.2");
    }
    let job = started(&core, Call::TestTorrentClient);
    common::wait_job(&c, job);
    let settled = downloads_status(&core).settings;
    assert_eq!(settled.torrent_mode, TorrentMode::Managed);
    assert_eq!(settled.torrent_url, status.settings.torrent_url);
    unsafe { std::env::remove_var("ANIBEAM_QBITTORRENT_NOX") };
    core.shutdown();
}

/// A download that asks for download in order and to replace episode 2:
/// the order is set once the client lists the torrent, and the old episode
/// 2 goes to the trash only after the new file has finished; episodes 1
/// and 3 are never touched. The trash lands in the test's own data home.
#[test]
fn a_replacing_download_is_set_in_order_and_trashes_the_old_file_only_when_the_new_one_is_done() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    unsafe { std::env::set_var("XDG_DATA_HOME", dir.path().join("share")) };
    save(&core, remote(), Some("tok"), Some("pw"));

    let lib = dir.path().join("lib");
    let show = lib.join("Sousou no Frieren");
    std::fs::create_dir_all(&show).unwrap();
    let old = |n: u32| {
        show.join(format!(
            "[SubsPlease] Sousou no Frieren - {n:02} (1080p).mkv"
        ))
    };
    for n in 1..=3 {
        std::fs::write(old(n), b"x").unwrap();
    }
    core.call(Call::AddSource {
        path: lib.to_string_lossy().into_owned(),
    })
    .unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ScanFinished { .. }),
        Duration::from_secs(10),
    );

    let mut item = release(2, Value::Null);
    item.title = "[Erai-raws] Sousou no Frieren - 02 [1080p]".to_string();
    let hash = format!("{:040}", 2);
    http.push_for(
        "/downloads",
        200,
        json!({"count": 1, "save_path": "/dl/Frieren", "hashes": [hash]}).to_string(),
    );
    let job = started(
        &core,
        Call::DownloadReleasesWith {
            items: vec![item.clone()],
            name: Some("Sousou no Frieren".to_string()),
            options: DownloadOptions {
                sequential: true,
                provider: "erai".to_string(),
                series: Some(1),
                replace: vec![ReplaceItem {
                    item: 0,
                    episodes: vec![2],
                }],
                cancel: Vec::new(),
                rolling: None,
            },
        },
    );
    let done = common::wait_job(&c, job);
    assert!(
        matches!(&done.body, EventBody::TorrentsAdded { .. }),
        "{:?}",
        done.body
    );

    let new_torrent = |progress: f64| {
        json!([{"hash": hash, "name": "[Erai-raws] Sousou no Frieren - 02 [1080p]", "state": "downloading",
                "progress": progress, "size": 100, "save_path": "/dl/Frieren",
                "content_path": "/dl/Frieren/[Erai-raws] Sousou no Frieren - 02 [1080p].mkv",
                "added_on": 1, "completion_on": -1, "seq_dl": false}])
        .to_string()
    };
    let new_file = |progress: f64| {
        json!([{"name": "[Erai-raws] Sousou no Frieren - 02 [1080p].mkv", "size": 100,
                "progress": progress, "priority": 1}])
        .to_string()
    };

    // Round one: half done. In order gets switched on; nothing is trashed.
    http.push_for("torrents/info", 200, new_torrent(0.5));
    http.push_for("torrents/info", 200, new_torrent(0.5));
    http.push_for("torrents/files", 200, new_file(0.5));
    http.push_for("torrents/files", 200, new_file(0.5));
    http.push_for("torrents/toggleSequentialDownload", 200, "");
    http.push_for("transfer/info", 200, json!({}).to_string());
    assert_eq!(core.call(Call::ListTorrents).unwrap(), Reply::Ok);
    common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::TorrentsRead { torrents, .. } if torrents.len() == 1 && torrents[0].progress < 1.0),
        Duration::from_secs(10),
    );
    let toggles: Vec<_> = http
        .requests()
        .into_iter()
        .filter(|r| r.url.contains("toggleSequentialDownload"))
        .collect();
    assert_eq!(toggles.len(), 1, "switched on once");
    assert_eq!(form_of(&toggles[0]), format!("hashes={hash}"));
    for n in 1..=3 {
        assert!(
            old(n).exists(),
            "episode {n} stays while the new one downloads"
        );
    }

    // Round two: finished. Only the old episode 2 goes; order is not asked again.
    http.push_for("torrents/info", 200, new_torrent(1.0));
    http.push_for("torrents/files", 200, new_file(1.0));
    http.push_for("transfer/info", 200, json!({}).to_string());
    assert_eq!(core.call(Call::ListTorrents).unwrap(), Reply::Ok);
    common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::TorrentsRead { torrents, .. } if torrents.len() == 1 && torrents[0].progress >= 1.0),
        Duration::from_secs(10),
    );
    assert!(!old(2).exists(), "the replaced episode went to the trash");
    assert!(
        old(1).exists() && old(3).exists(),
        "no other episode is touched"
    );
    let toggles = http
        .requests()
        .into_iter()
        .filter(|r| r.url.contains("toggleSequentialDownload"))
        .count();
    assert_eq!(toggles, 1, "still the one toggle");
}

/// Getting episodes for a show already in the library saves them straight
/// into its folder, wherever it is; a download for no show leaves the
/// folder to anirss.
#[test]
fn episodes_for_a_library_show_are_saved_into_its_folder() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save(&core, remote(), Some("tok"), Some("pw"));

    let lib = dir.path().join("TV");
    let show = lib.join("Avatar Seven Havens");
    std::fs::create_dir_all(&show).unwrap();
    std::fs::write(show.join("Avatar.Seven.Havens.S01E01.1080p.mkv"), b"x").unwrap();
    core.call(Call::AddSource {
        path: lib.to_string_lossy().into_owned(),
    })
    .unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ScanFinished { .. }),
        Duration::from_secs(10),
    );

    let mut item = release(11, Value::Null);
    item.title = "Avatar.Seven.Havens.S01E11.1080p.x265-ELiTE".to_string();
    let get = |series: Option<u64>| {
        http.push_for(
            "/downloads",
            200,
            json!({"count": 1, "save_path": "/x", "hashes": []}).to_string(),
        );
        let job = started(
            &core,
            Call::DownloadReleasesWith {
                items: vec![item.clone()],
                name: Some("Avatar Seven Havens".to_string()),
                options: DownloadOptions {
                    series,
                    rolling: Some(false),
                    ..Default::default()
                },
            },
        );
        common::wait_job(&c, job);
        let posts: Vec<_> = http
            .requests()
            .into_iter()
            .filter(|r| r.url.ends_with("/downloads"))
            .collect();
        let body: Value = serde_json::from_str(&form_of(posts.last().unwrap())).unwrap();
        body
    };
    let body = get(Some(1));
    assert_eq!(body["save_path"], json!(show.to_string_lossy()));
    let body = get(None);
    assert!(body.get("save_path").is_none(), "{body}");
    // A client on another machine names its paths its own way
    save(
        &core,
        DownloadSettings {
            torrent_url: "http://seedbox.example:8080".to_string(),
            ..remote()
        },
        Some("tok"),
        Some("pw"),
    );
    let body = get(Some(1));
    assert!(body.get("save_path").is_none(), "{body}");
    core.shutdown();
}

/// Deleting an episode takes the torrent that held it out of the client (#266), so the same
/// release can be got again: a season pack whose folder held it goes, its data kept; a
/// torrent saved straight into the library folder goes only when its own file list names
/// the file; one still downloading, or of another show, stays.
#[test]
fn deleting_an_episode_removes_the_torrent_that_held_it() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    unsafe { std::env::set_var("XDG_DATA_HOME", dir.path().join("share")) };
    save(&core, settings("anirss"), None, None);
    let lib = std::fs::canonicalize(dir.path()).unwrap().join("lib");
    let show = lib.join("Sousou no Frieren");
    std::fs::create_dir_all(&show).unwrap();
    let ep = |n: u32| {
        show.join(format!(
            "[SubsPlease] Sousou no Frieren - {n:02} (1080p).mkv"
        ))
    };
    for n in 1..=2 {
        std::fs::write(ep(n), b"x").unwrap();
    }
    std::fs::write(lib.join("Other.mkv"), b"x").unwrap();
    core.call(Call::AddSource {
        path: lib.to_string_lossy().into_owned(),
    })
    .unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ScanFinished { .. }),
        Duration::from_secs(10),
    );
    let series = match core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: true,
        })
        .unwrap()
    {
        Reply::Series { series } => series
            .into_iter()
            .find(|s| s.path.ends_with("Sousou no Frieren"))
            .expect("the show is in the library"),
        other => panic!("{other:?}"),
    };
    let file = match core.call(Call::GetSeries { series: series.id }).unwrap() {
        Reply::SeriesDetail { detail } => {
            detail
                .episodes
                .into_iter()
                .find(|e| e.number == 1.0)
                .expect("episode 1")
                .file
        }
        other => panic!("{other:?}"),
    };

    let lib_s = lib.to_string_lossy().into_owned();
    let show_s = show.to_string_lossy().into_owned();
    let torrent = |hash: &str, progress: f64, save: &str, content: &str| {
        json!({"hash": hash, "name": hash, "state": "uploading", "progress": progress, "size": 1,
               "save_path": save, "content_path": content})
    };
    http.push_for(
        "torrents/info",
        200,
        json!([
            torrent("aa", 1.0, &lib_s, &show_s),
            torrent("bb", 0.5, &lib_s, &show_s),
            torrent("cc", 1.0, &lib_s, &lib_s),
            torrent("dd", 1.0, &lib_s, &format!("{lib_s}/Something Else")),
        ])
        .to_string(),
    );
    http.push_for(
        "torrents/files?hash=cc",
        200,
        json!([{"name": "Other.mkv", "size": 1, "progress": 1.0, "priority": 1}]).to_string(),
    );
    http.push_for(
        "torrents/files?hash=aa",
        200,
        json!([{"name": "Sousou no Frieren/[SubsPlease] Sousou no Frieren - 01 (1080p).mkv",
                "size": 1, "progress": 1.0, "priority": 1}])
        .to_string(),
    );
    http.push_for("torrents/delete", 200, "");

    core.call(Call::DeleteEpisodes {
        series: series.id,
        files: vec![file],
    })
    .unwrap();
    assert!(!ep(1).exists() && ep(2).exists());

    let started = Instant::now();
    let delete = loop {
        if let Some(r) = http
            .requests()
            .into_iter()
            .find(|r| r.url.contains("torrents/delete"))
        {
            break r;
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "no torrent was removed"
        );
        std::thread::sleep(Duration::from_millis(25));
    };
    assert_eq!(form_of(&delete), "hashes=aa&deleteFiles=false");
    core.shutdown();
}
