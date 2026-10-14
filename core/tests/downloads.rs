//! The downloads domain end to end, against `FakeHttp`: the settings and
//! their secrets, the sidecar that `ConnectDownloads` starts, a search and
//! a refine through anirss, and the torrent client's list read. The one
//! case that needs a real process, the sidecar, runs a fake `anirss` from a
//! temp `PATH`. Every sidecar case lives in one test because `PATH` and
//! `HOME` are process-wide and this binary runs its tests side by side.

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
        anirss_url: "http://127.0.0.1:8787".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_url: "http://localhost:9090".to_string(),
        torrent_username: "me".to_string(),
        has_torrent_password: false,
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

/// The fake `anirss`: writes its arguments and its token into `capture`,
/// then stays up the way a server would. The token is written last, so a
/// reader that sees it knows the arguments are already there.
fn fake_anirss(capture: &Path) -> String {
    format!(
        "#!/bin/sh\necho \"$*\" > '{args}'\nprintf '%s' \"$ANIRSS_API_TOKEN\" > '{token}'\nexec sleep 300\n",
        args = capture.join("args").display(),
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
fn the_sidecar_connects_searches_and_reports_a_missing_command() {
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
        json!({"ok": true, "version": "0.3.4"}).to_string(),
    );
    http.push_for(
        "/health",
        200,
        json!({"ok": true, "version": "0.3.4"}).to_string(),
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
    assert_eq!(status.anirss.version, "0.3.4");
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

    core.shutdown();
    unsafe {
        match &original_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        std::env::set_var("PATH", &original_path);
    }
}

#[test]
fn list_torrents_emits_the_torrents_and_the_transfer_totals() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());

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
