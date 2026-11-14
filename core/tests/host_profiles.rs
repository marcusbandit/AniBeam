//! Several people on one tailnet (#562, #631): the host answers each device as the
//! profile of the Tailscale user it belongs to. The owner's devices see the owner's
//! progress, a friend's tablet sees the friend's own, and a device the host cannot tie to
//! anyone gets nothing. The asking address is set per server with `MockConnectInfo`, as
//! in `host_trust.rs`.

mod common;
mod fixtures;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anibeam_core::*;
use axum::extract::connect_info::MockConnectInfo;

/// The owner (user 100) has this machine and a phone; a friend (user 200) has a tablet;
/// the laptop names no user.
const STATUS: &[u8] = br#"{
    "Self": {"ID": "nSelf", "HostName": "desk", "UserID": 100, "TailscaleIPs": ["100.64.0.1"]},
    "Peer": {
        "nodekey:aa": {"ID": "nPhone", "HostName": "pixel", "UserID": 100, "TailscaleIPs": ["100.70.1.2"]},
        "nodekey:bb": {"ID": "nTablet", "HostName": "tab", "UserID": 200, "TailscaleIPs": ["100.70.1.3"]},
        "nodekey:cc": {"ID": "nLaptop", "HostName": "lap", "TailscaleIPs": ["100.70.1.4"]}
    },
    "User": {
        "100": {"ID": 100, "LoginName": "owner@example.com", "DisplayName": "Owner"},
        "200": {"ID": 200, "LoginName": "friend@example.com", "DisplayName": "Friend"}
    }
}"#;

const PHONE: &str = "100.70.1.2";
const TABLET: &str = "100.70.1.3";
const LAPTOP: &str = "100.70.1.4";

fn serve_as(rt: &tokio::runtime::Runtime, core: &Arc<Core>, from: &str) -> String {
    let from: SocketAddr = format!("{from}:40000").parse().unwrap();
    let app = anibeam_core::host::server::router(Arc::downgrade(core)).layer(MockConnectInfo(from));
    rt.block_on(async {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let at = l.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(l, app).await });
        format!("http://{at}")
    })
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
}

fn get(rt: &tokio::runtime::Runtime, url: &str) -> (u16, serde_json::Value) {
    rt.block_on(async {
        let r = client().get(url).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or_default())
    })
}

fn call(rt: &tokio::runtime::Runtime, base: &str, call: &Call) -> (u16, serde_json::Value) {
    let body = serde_json::to_vec(call).unwrap();
    rt.block_on(async {
        let r = client()
            .post(format!("{base}{}", anibeam_host_api::CALL))
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or_default())
    })
}

/// The episode keys a snapshot says are watched.
fn watched(rt: &tokio::runtime::Runtime, base: &str) -> Vec<String> {
    let (status, body) = get(rt, &format!("{base}{}", anibeam_host_api::SNAPSHOT));
    assert_eq!(status, 200, "{body}");
    let mut keys: Vec<String> = body["series"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["episodes"].as_array().cloned().unwrap_or_default())
        .filter(|e| e["watched"].as_bool() == Some(true))
        .map(|e| e["key"].as_str().unwrap().to_string())
        .collect();
    keys.sort();
    keys
}

/// Opens a file and plays it to the end over the host, as a phone's player does.
fn play_through(rt: &tokio::runtime::Runtime, base: &str, file: u64) -> u64 {
    let (status, reply) = call(rt, base, &Call::OpenPlayback { file });
    assert_eq!(status, 200, "{reply}");
    let session = reply["Playback"]["session"]["session"].as_u64().unwrap();
    let (status, reply) = call(
        rt,
        base,
        &Call::ClosePlayback {
            session,
            position: 1400.0,
            reason: CloseReason::Ended,
        },
    );
    assert_eq!(status, 200, "{reply}");
    session
}

#[test]
fn each_person_on_the_tailnet_sees_their_own_progress() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let src = fixtures::insert_source(&core, "/lib");
    let series = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Show", "Show");
    let now = anibeam_core::time::now_secs();
    let files: Vec<u64> = (1..=3)
        .map(|n| {
            fixtures::insert_episode_file(
                &core,
                series,
                &format!("/lib/Show/Episode {n}.mkv"),
                f64::from(n),
                now,
            )
        })
        .collect();
    // The owner watched episode 1 here, on this machine's own player.
    fixtures::insert_completed(&core, series, "1", now - 60);

    let phone = serve_as(&rt, &core, PHONE);
    let tablet = serve_as(&rt, &core, TABLET);
    assert_eq!(
        watched(&rt, &phone),
        vec!["1"],
        "the owner's phone is the owner"
    );
    assert!(
        watched(&rt, &tablet).is_empty(),
        "the friend starts with nothing"
    );

    // The friend plays episode 2 to the end on the tablet: it is theirs alone.
    play_through(&rt, &tablet, files[1]);
    // A snapshot is kept for thirty seconds, so a fresh server reads it anew.
    let tablet_later = serve_as(&rt, &core, TABLET);
    assert_eq!(watched(&rt, &tablet_later), vec!["2"]);
    assert_eq!(
        watched(&rt, &serve_as(&rt, &core, PHONE)),
        vec!["1"],
        "the owner's phone does not see the friend's episode"
    );
    let owner = match core.call(Call::GetSeries { series }).unwrap() {
        Reply::SeriesDetail { detail } => detail,
        other => panic!("{other:?}"),
    };
    let mine: Vec<f64> = owner
        .episodes
        .iter()
        .filter(|e| e.watched)
        .map(|e| e.number)
        .collect();
    assert_eq!(mine, vec![1.0], "the desktop still shows the owner's own");
    let profiles: Vec<(i64, String)> = core
        .store()
        .read(|c| {
            let mut stmt = c.prepare(
                "SELECT p.id, p.name FROM completed c JOIN profiles p ON p.id = c.profile_id
                 WHERE c.episode_key = '2'",
            )?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .unwrap();
    assert_eq!(profiles, vec![(2, "Friend".to_string())]);

    // The owner's tracker is not the friend's to mark, and an owner's session is not the
    // friend's to move.
    let (status, _) = call(
        &rt,
        &tablet,
        &Call::MarkEpisode {
            series,
            episode: 2.0,
        },
    );
    assert_eq!(status, 422);
    let mine = play_open(&rt, &phone, files[2]);
    let (status, _) = call(
        &rt,
        &tablet,
        &Call::Tick {
            session: mine,
            position: 10.0,
            paused: false,
        },
    );
    assert_eq!(
        status, 422,
        "a session the tablet did not open is not found"
    );

    // A second close of the tablet's own session is quiet, as it is for the owner.
    let theirs = play_through(&rt, &tablet, files[0]);
    let (status, _) = call(
        &rt,
        &tablet,
        &Call::ClosePlayback {
            session: theirs,
            position: 1400.0,
            reason: CloseReason::Ended,
        },
    );
    assert_eq!(status, 200);
    // Not interested is the owner's list.
    let (status, _) = call(
        &rt,
        &tablet,
        &Call::SetNotInterested {
            anilist_id: 1,
            not_interested: true,
        },
    );
    assert_eq!(status, 422);
}

fn play_open(rt: &tokio::runtime::Runtime, base: &str, file: u64) -> u64 {
    let (status, reply) = call(rt, base, &Call::OpenPlayback { file });
    assert_eq!(status, 200, "{reply}");
    reply["Playback"]["session"]["session"].as_u64().unwrap()
}

#[test]
fn a_device_the_host_cannot_tie_to_a_person_gets_nothing() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let laptop = serve_as(&rt, &core, LAPTOP);
    let (status, body) = get(&rt, &format!("{laptop}{}", anibeam_host_api::SNAPSHOT));
    assert_eq!(status, 403);
    assert_eq!(
        body["error"].as_str(),
        Some(
            "That computer cannot tell whose device this is on the tailnet, so it does not answer."
        )
    );
}

/// Downloads fill the host's disk, so only the owner's devices may send them (#733): a
/// friend's tablet is refused at the door, and the owner's phone reaches the core, which
/// answers for itself. The owner's call carries no releases, so nothing is ever handed
/// to a torrent client here.
#[test]
fn only_the_owners_devices_download_to_the_host() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let release: Release = serde_json::from_value(serde_json::json!({
        "title": "[Group] Show - 01 (1080p).mkv",
        "link": "magnet:?xt=urn:btih:0000000000000000000000000000000000000000",
    }))
    .unwrap();
    let download = |items: Vec<Release>| Call::DownloadReleasesWith {
        items,
        name: Some("Show".into()),
        options: DownloadOptions::default(),
    };

    let features = |base: &str| -> Vec<String> {
        let (_, hello) = get(&rt, &format!("{base}{}", anibeam_host_api::HELLO));
        serde_json::from_value(hello["features"].clone()).unwrap()
    };
    let tablet = serve_as(&rt, &core, TABLET);
    assert!(
        !features(&tablet)
            .iter()
            .any(|f| f == anibeam_host_api::FEATURE_DOWNLOADS),
        "a friend's device is not offered downloads"
    );
    let (status, body) = call(&rt, &tablet, &download(vec![release]));
    assert_eq!(status, 403, "{body}");
    assert_eq!(
        body["error"].as_str(),
        Some(anibeam_core::host::stream::DOWNLOADS_OWNER_ONLY)
    );
    let follow = Call::Subscribe {
        query: "Show".into(),
        endpoint: "feed".into(),
        items: Vec::new(),
        name: None,
        wire_query: None,
        series: None,
    };
    assert_eq!(call(&rt, &tablet, &follow).0, 403);

    let phone = serve_as(&rt, &core, PHONE);
    assert!(
        features(&phone)
            .iter()
            .any(|f| f == anibeam_host_api::FEATURE_DOWNLOADS)
    );
    let (status, body) = call(&rt, &phone, &download(Vec::new()));
    assert_eq!(status, 422, "the core's own answer, not the door's: {body}");
    assert_eq!(
        body["core"]["Invalid"]["field"].as_str(),
        Some("items"),
        "{body}"
    );
}

/// Each device's hello says whose progress it gets (#633): the owner's phone names the
/// owner as the computer's own person, the friend's tablet names the friend.
#[test]
fn each_hello_names_whose_progress_it_answers_with() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let profile = |base: &str| -> Option<anibeam_host_api::Profile> {
        let (status, hello) = get(&rt, &format!("{base}{}", anibeam_host_api::HELLO));
        assert_eq!(status, 200, "{hello}");
        let hello: anibeam_host_api::Hello = serde_json::from_value(hello).unwrap();
        hello.profile
    };
    let phone = profile(&serve_as(&rt, &core, PHONE)).expect("the owner is named");
    assert_eq!(
        (phone.name.as_str(), phone.login.as_str(), phone.owner),
        ("Owner", "owner@example.com", true)
    );
    let tablet = profile(&serve_as(&rt, &core, TABLET)).expect("the friend is named");
    assert_eq!(
        (tablet.name.as_str(), tablet.login.as_str(), tablet.owner),
        ("Friend", "friend@example.com", false)
    );
}
