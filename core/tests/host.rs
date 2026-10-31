//! The tailnet host end to end: the preference starts a real server on a
//! loopback port (`set_host_bind`, standing in for `ANIBEAM_HOST_BIND`), and
//! the three routes are read back over HTTP as the `anibeam-host-api` types
//! the phone app deserialises. Downloads is wired to `FakeHttp` and a
//! command that does not exist, so a snapshot sees a working qBittorrent and
//! a missing anirss.

mod common;
mod fixtures;

use std::time::{Duration, Instant};

use anibeam_core::net::FakeHttp;
use anibeam_core::*;
use anibeam_host_api as wire;
use serde_json::json;

fn host_status(core: &Core) -> HostStatus {
    match core.call(Call::GetHost).unwrap() {
        Reply::Host { status } => status,
        other => panic!("{other:?}"),
    }
}

fn share(core: &Core, on: bool) {
    let mut preferences = match core.call(Call::GetPreferences).unwrap() {
        Reply::Preferences { preferences } => preferences,
        other => panic!("{other:?}"),
    };
    preferences.share_over_tailscale = on;
    assert_eq!(
        core.call(Call::SetPreferences { preferences }).unwrap(),
        Reply::Ok
    );
}

/// The status once `pred` holds of it, polled: the supervisor binds on the
/// runtime, a beat after the call returns.
fn wait_status(core: &Core, pred: impl Fn(&HostStatus) -> bool) -> HostStatus {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = host_status(core);
        if pred(&status) {
            return status;
        }
        assert!(Instant::now() < deadline, "host stayed at {status:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap()
}

fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

#[test]
fn the_preference_starts_and_stops_a_server_that_serves_the_library() {
    let http = FakeHttp::new();
    let (dir, core, _c) = common::open_core_with_http(http.clone());
    // Saved settings leave nothing to default from the machine: a command
    // that does not exist, and a client that only FakeHttp answers.
    let settings = DownloadSettings {
        anirss_mode: AnirssMode::Sidecar,
        anirss_command: "/nonexistent/anirss-for-the-host-test".to_string(),
        anirss_url: "http://127.0.0.1:1".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_mode: TorrentMode::Existing,
        torrent_url: "http://localhost:9090".to_string(),
        torrent_username: "me".to_string(),
        has_torrent_password: false,
        jackett_command: String::new(),
        saved_clients: Vec::new(),
        active_client: String::new(),
        prune_finished_feeds: true,
        rolling: Default::default(),
        engine: Default::default(),
    };
    core.call(Call::SetDownloadSettings {
        settings,
        anirss_token: None,
        torrent_password: None,
    })
    .unwrap();
    http.push_for(
        "torrents/info",
        200,
        json!([{
            "hash": "a1b2c3", "name": "[SubsPlease] Sousou no Frieren - 08", "state": "downloading",
            "progress": 0.25, "size": 1500000000u64, "dlspeed": 1200000, "upspeed": 0, "eta": 600,
            "num_seeds": 1, "num_leechs": 1, "ratio": 0.0,
            "save_path": "/lib/Sousou no Frieren", "content_path": "/lib/Sousou no Frieren/08.mkv",
            "added_on": 1700000000, "completion_on": -1, "tags": "", "category": ""
        }])
        .to_string(),
    );
    http.push_for("transfer/info", 200, json!({}).to_string());

    // The library: a show with a poster on disk, a hidden show, a film.
    let now = time::now_secs();
    let src = fixtures::insert_source(&core, "/lib");
    let frieren = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/lib/Sousou no Frieren",
        "Sousou no Frieren",
    );
    for n in 1..=7 {
        fixtures::insert_file(
            &core,
            frieren,
            &format!("/lib/Sousou no Frieren/{n:02}.mkv"),
            n as f64,
            None,
            "episode",
            now - 1000 + n,
        );
    }
    fixtures::insert_media_with_cover(&core, 154587, "https://img.test/frieren.jpg", None);
    core.store()
        .write(|c| {
            c.execute(
                "UPDATE anilist_media SET title_romaji = 'Sousou no Frieren', title_english = 'Frieren', \
                 status = 'RELEASING', episodes = 12, year = 2023 WHERE id = 154587",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    fixtures::match_series(&core, frieren, Some(154587), Some(52991));
    for n in 1..=7 {
        fixtures::insert_airing(&core, 154587, n, now - 86400 * (8 - n));
    }
    // Three still to come, the first the soonest.
    fixtures::insert_airing(&core, 154587, 10, now + 3 * 86400);
    fixtures::insert_airing(&core, 154587, 8, now + 86400);
    fixtures::insert_airing(&core, 154587, 9, now + 2 * 86400);
    fixtures::insert_tracker_entry(&core, "anilist", 154587, 5, "watching", None);
    fixtures::insert_completed(&core, frieren, "7", now - 10);
    // The poster: the image cache's row and the file it names.
    let jpeg = [
        0xFF, 0xD8, 0xFF, 0xE0, 0, 0x10, b'J', b'F', b'I', b'F', 0, 1,
    ];
    let images = dir.path().join("cache").join("images");
    std::fs::create_dir_all(images.join("ab")).unwrap();
    std::fs::write(images.join("ab").join("frieren.jpg"), jpeg).unwrap();
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT INTO images (url, path, bytes, fetched_at, used_at) VALUES \
                 ('https://img.test/frieren.jpg', 'ab/frieren.jpg', 12, 1, 1)",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    let secret = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Secret", "Secret");
    fixtures::insert_file(
        &core,
        secret,
        "/lib/Secret/01.mkv",
        1.0,
        None,
        "episode",
        now,
    );
    core.call(Call::SetHidden {
        series: secret,
        hidden: true,
    })
    .unwrap();
    let film = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Movie,
        "/lib/Movies/Perfect Blue (1997).mkv",
        "Perfect Blue",
    );
    fixtures::insert_file(
        &core,
        film,
        "/lib/Movies/Perfect Blue (1997).mkv",
        1.0,
        None,
        "episode",
        now,
    );

    // Off until asked.
    core.set_host_bind(Some("127.0.0.1:0".to_string()));
    assert_eq!(
        host_status(&core),
        HostStatus {
            enabled: false,
            urls: Vec::new(),
            message: "Off".to_string()
        }
    );
    share(&core, true);
    let status = wait_status(&core, |s| !s.urls.is_empty());
    assert!(status.enabled);
    assert!(
        status.message.starts_with("Serving on 127.0.0.1:"),
        "{status:?}"
    );
    let base = status.urls[0].clone();
    assert!(base.starts_with("http://127.0.0.1:"), "{base}");

    block_on(async {
        let client = client();

        let hello: wire::Hello = client
            .get(format!("{base}{}", wire::HELLO))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(hello.app, wire::APP);
        assert_eq!(hello.api, wire::API_VERSION);
        assert_eq!(hello.version, anibeam_core::VERSION);
        assert!(!hello.host.is_empty());
        assert!((hello.now - time::now_secs()).abs() < 5);

        let reply = client
            .get(format!("{base}{}", wire::SNAPSHOT))
            .send()
            .await
            .unwrap();
        assert_eq!(reply.status(), 200);
        assert_eq!(reply.headers()["content-type"], "application/json");
        let snapshot: wire::Snapshot = reply.json().await.unwrap();
        assert_eq!(snapshot.api, wire::API_VERSION);
        assert_eq!(snapshot.version, anibeam_core::VERSION);
        let titles: Vec<&str> = snapshot.series.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(
            titles.len(),
            2,
            "the hidden show is left out, the film is in: {titles:?}"
        );

        let f = snapshot.series.iter().find(|s| s.id == frieren).unwrap();
        assert_eq!(f.kind, wire::Kind::Show);
        assert_eq!(f.title, "Sousou no Frieren");
        assert_eq!(f.titles.english.as_deref(), Some("Frieren"));
        assert_eq!(f.status.as_deref(), Some("Releasing"));
        assert_eq!(f.anilist_id, Some(154587));
        assert_eq!(f.mal_id, Some(52991));
        assert_eq!(f.total_episodes, Some(12));
        assert_eq!(f.episodes_on_disk, 7);
        assert_eq!(f.progress, Some(5));
        assert_eq!(f.year, Some(2023));
        let upcoming: Vec<u32> = f.upcoming.iter().map(|a| a.episode).collect();
        assert_eq!(upcoming, vec![8, 9, 10], "soonest first");
        assert_eq!(f.next_airing.as_ref(), f.upcoming.first());
        let codes: Vec<(&str, bool)> = f
            .episodes
            .iter()
            .map(|e| (e.code.as_str(), e.watched))
            .collect();
        assert_eq!(codes.len(), 7);
        assert_eq!(codes[0], ("E01", true), "tracker progress 5 covers 1 to 5");
        assert_eq!(codes[4], ("E05", true));
        assert_eq!(codes[5], ("E06", false));
        assert_eq!(codes[6], ("E07", true), "a completed view counts too");
        assert!(f.episodes[0].aired_at.is_some());
        assert_eq!(f.episodes[0].key, "1");
        let tag = f.poster.clone().expect("the poster file exists");
        assert_eq!(tag.len(), 10);
        let pb = snapshot.series.iter().find(|s| s.id == film).unwrap();
        assert_eq!(pb.kind, wire::Kind::Movie);
        assert_eq!(pb.poster, None);
        assert!(f.added_at > 0 && f.latest_activity_at > 0);

        // Downloads: qBittorrent answered, anirss is a command that is not there.
        let torrents = snapshot.torrents.as_ref().expect("the client answered");
        assert_eq!(torrents.len(), 1);
        assert_eq!(torrents[0].hash, "a1b2c3");
        assert_eq!(torrents[0].series_id, Some(frieren), "linked by folder");
        assert_eq!(torrents[0].eta, Some(600));
        assert_eq!(torrents[0].completed_on, None);
        assert!(snapshot.subscriptions.is_empty());
        assert_eq!(snapshot.problems, vec!["anirss is not running".to_string()]);

        // The poster: bytes, type, tag and a day of cache.
        let url = format!("{base}{}", wire::poster_path(frieren));
        let reply = client.get(&url).send().await.unwrap();
        assert_eq!(reply.status(), 200);
        assert_eq!(reply.headers()["content-type"], "image/jpeg");
        assert_eq!(reply.headers()["etag"], format!("\"{tag}\""));
        assert_eq!(reply.headers()["cache-control"], "max-age=86400");
        assert_eq!(reply.bytes().await.unwrap().as_ref(), jpeg);
        let again = client
            .get(&url)
            .header("if-none-match", format!("\"{tag}\""))
            .send()
            .await
            .unwrap();
        assert_eq!(again.status(), 304);
        for missing in [
            format!("{base}{}", wire::poster_path(film)),
            format!("{base}{}", wire::poster_path(secret)),
            format!("{base}{}", wire::poster_path(9999)),
            format!("{base}/api/v1/posters/not-a-number"),
            format!("{base}/api/v1/posters/..%2F..%2Fetc%2Fpasswd"),
            format!("{base}/nothing/here"),
        ] {
            let reply = client.get(&missing).send().await.unwrap();
            assert_eq!(reply.status(), 404, "{missing}");
            assert!(
                reply.json::<serde_json::Value>().await.unwrap()["error"].is_string(),
                "{missing}"
            );
        }

        // Read-only: nothing but GET gets through.
        let post = client
            .post(format!("{base}{}", wire::SNAPSHOT))
            .send()
            .await
            .unwrap();
        assert_eq!(post.status(), 405);
        assert!(post.json::<serde_json::Value>().await.unwrap()["error"].is_string());
        let delete = client.delete(&url).send().await.unwrap();
        assert_eq!(delete.status(), 405);
    });

    // Off again: the port stops answering and the status says so.
    share(&core, false);
    let status = wait_status(&core, |s| !s.enabled);
    assert!(status.urls.is_empty());
    assert_eq!(status.message, "Off");
    block_on(async {
        let deadline = Instant::now() + Duration::from_secs(10);
        let c = client();
        loop {
            if c.get(format!("{base}{}", wire::HELLO))
                .send()
                .await
                .is_err()
            {
                break;
            }
            assert!(Instant::now() < deadline, "the server kept answering");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
    core.shutdown();
}

#[test]
fn a_host_that_was_left_on_serves_again_after_a_restart() {
    let (dir, core, _c) = common::open_core();
    core.set_host_bind(Some("127.0.0.1:0".to_string()));
    share(&core, true);
    wait_status(&core, |s| !s.urls.is_empty());
    core.shutdown();
    drop(core);

    // A new core over the same data: `start` reads the preference.
    let paths = CorePaths::under(dir.path());
    let secrets = anibeam_core::trackers::Secrets::file_only(paths.secrets_path());
    let core = Core::open_with_secrets(paths, secrets).unwrap();
    core.set_host_bind(Some("127.0.0.1:0".to_string()));
    assert!(!host_status(&core).enabled, "nothing runs before start");
    core.start().unwrap();
    let status = wait_status(&core, |s| !s.urls.is_empty());
    assert!(status.enabled);
    core.shutdown();
}

#[test]
fn a_bind_that_is_not_an_address_is_reported_not_crashed_on() {
    let (_dir, core, _c) = common::open_core();
    core.set_host_bind(Some("banana".to_string()));
    share(&core, true);
    let status = wait_status(&core, |s| s.message.contains("banana"));
    assert!(status.enabled && status.urls.is_empty());
    core.shutdown();
}

#[test]
fn a_port_that_is_taken_says_so() {
    let (_dir, core, _c) = common::open_core();
    let squatter = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    core.set_host_bind(Some(squatter.local_addr().unwrap().to_string()));
    share(&core, true);
    let status = wait_status(&core, |s| s.message.contains("is taken"));
    assert_eq!(
        status.message,
        format!("Port {} is taken", squatter.local_addr().unwrap().port())
    );
    core.shutdown();
}
