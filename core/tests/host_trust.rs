//! Who the tailnet host answers, end to end through its real routes. The
//! asking address is set per server with axum's `MockConnectInfo`, so a
//! request "from" a tailnet phone, a switched-off device or the local network
//! runs every check a real one would, on loopback, with no tailnet at all.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anibeam_core::*;
use axum::extract::connect_info::MockConnectInfo;

const STATUS: &[u8] = br#"{
    "Self": {"ID": "nSelf", "HostName": "banditbox", "TailscaleIPs": ["100.65.105.9"]},
    "Peer": {
        "nodekey:aa": {"ID": "nPhone", "HostName": "pixel", "OS": "android", "Online": true,
                       "TailscaleIPs": ["100.70.1.2", "fd7a:115c:a1e0::2"]},
        "nodekey:bb": {"ID": "nLaptop", "HostName": "kangaeru", "OS": "linux", "Online": false,
                       "TailscaleIPs": ["100.80.3.4"]}
    }
}"#;

/// Serves the host's router on loopback as if every request came from `from`,
/// and gives its base URL.
fn serve_as(rt: &tokio::runtime::Runtime, core: &Arc<Core>, from: &str) -> String {
    let from: SocketAddr = format!("{from}:40000")
        .parse()
        .or_else(|_| format!("[{from}]:40000").parse())
        .unwrap();
    let app = anibeam_core::host::server::router(Arc::downgrade(core)).layer(MockConnectInfo(from));
    rt.block_on(async {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let at = l.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(l, app).await });
        format!("http://{at}")
    })
}

/// The status and the error text of a GET.
fn get(rt: &tokio::runtime::Runtime, url: &str) -> (u16, String) {
    rt.block_on(async {
        let r = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap()
            .get(url)
            .send()
            .await
            .unwrap();
        let status = r.status().as_u16();
        let body: serde_json::Value = r.json().await.unwrap_or_default();
        (
            status,
            body["error"].as_str().unwrap_or_default().to_string(),
        )
    })
}

fn devices_now(core: &Core) -> Vec<TailnetDevice> {
    match core.call(Call::ListTailnetDevices).unwrap() {
        Reply::TailnetDevices { devices } => devices,
        other => panic!("{other:?}"),
    }
}

/// The list once the read it starts has landed: the call itself never waits
/// on `tailscale`, so the first answer can be the empty one from before.
fn devices(core: &Core) -> Vec<TailnetDevice> {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let list = devices_now(core);
        if !list.is_empty() || std::time::Instant::now() > deadline {
            return list;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn every_tailnet_device_is_answered_until_it_is_switched_off() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let hello = anibeam_host_api::HELLO;

    // Trusted by default: both devices are on, and both are answered.
    let listed = devices(&core);
    assert_eq!(
        listed
            .iter()
            .map(|d| (d.id.as_str(), d.name.as_str(), d.trusted))
            .collect::<Vec<_>>(),
        [("nLaptop", "kangaeru", true), ("nPhone", "pixel", true)]
    );
    let phone = serve_as(&rt, &core, "100.70.1.2");
    let phone6 = serve_as(&rt, &core, "fd7a:115c:a1e0::2");
    let laptop = serve_as(&rt, &core, "100.80.3.4");
    let stranger = serve_as(&rt, &core, "100.99.9.9");
    assert_eq!(get(&rt, &format!("{phone}{hello}")).0, 200);
    assert_eq!(get(&rt, &format!("{stranger}{hello}")).0, 200);

    // Switched off: every route refuses it, on each of its addresses, and
    // says why. The other device is still answered.
    core.call(Call::SetTailnetDeviceTrust {
        device: "nPhone".into(),
        trusted: false,
    })
    .unwrap();
    for route in [
        hello.to_string(),
        anibeam_host_api::SNAPSHOT.to_string(),
        anibeam_host_api::file_path(1),
        anibeam_host_api::hls_path(1, 720, None, None),
        anibeam_host_api::poster_path(1),
        "/no/such/route".to_string(),
    ] {
        for base in [&phone, &phone6] {
            let (status, why) = get(&rt, &format!("{base}{route}"));
            assert_eq!(status, 403, "{base}{route}");
            assert!(why.contains("switched off"), "{why}");
        }
    }
    let posted = rt.block_on(async {
        reqwest::Client::new()
            .post(format!("{phone}{}", anibeam_host_api::CALL))
            .header("content-type", "application/json")
            .body(r#"{"MarkEpisode":{"series":1,"episode":1.0}}"#)
            .send()
            .await
            .unwrap()
            .status()
            .as_u16()
    });
    assert_eq!(posted, 403, "a playback call from a switched-off device");
    assert_eq!(get(&rt, &format!("{laptop}{hello}")).0, 200);
    assert!(!devices(&core).iter().any(|d| d.id == "nPhone" && d.trusted));

    // With a device off, an address the tailnet cannot place might be it.
    assert_eq!(get(&rt, &format!("{stranger}{hello}")).0, 403);

    // A save of the whole preferences, from a page holding a copy made
    // before the switch, leaves the device off.
    let Reply::Preferences { mut preferences } = core.call(Call::GetPreferences).unwrap() else {
        panic!()
    };
    assert_eq!(preferences.tailnet_switched_off, ["nPhone"]);
    preferences.tailnet_switched_off.clear();
    core.call(Call::SetPreferences { preferences }).unwrap();
    assert_eq!(get(&rt, &format!("{phone}{hello}")).0, 403);

    // It stays off for the next core on the same data, from its first request.
    let paths = CorePaths::under(dir.path());
    let secrets = anibeam_core::trackers::Secrets::file_only(paths.secrets_path());
    let again = Core::open_with_secrets(paths, secrets).unwrap();
    anibeam_core::host::sync(&again);
    again.set_tailscale_status(Some(STATUS.to_vec()));
    let reopened = serve_as(&rt, &again, "100.70.1.2");
    assert_eq!(get(&rt, &format!("{reopened}{hello}")).0, 403);

    // Switched back on, it is answered again.
    core.call(Call::SetTailnetDeviceTrust {
        device: "nPhone".into(),
        trusted: true,
    })
    .unwrap();
    assert_eq!(get(&rt, &format!("{phone}{hello}")).0, 200);
    assert!(devices(&core).iter().all(|d| d.trusted));
}

#[test]
fn nothing_off_the_tailnet_is_answered_but_this_machine() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let hello = anibeam_host_api::HELLO;
    for from in ["192.168.1.20", "10.0.2.2", "fe80::1", "8.8.8.8"] {
        let base = serve_as(&rt, &core, from);
        let (status, why) = get(&rt, &format!("{base}{hello}"));
        assert_eq!(status, 403, "{from}");
        assert!(why.contains("tailnet"), "{why}");
    }
    for from in ["127.0.0.1", "::1", "100.65.105.9"] {
        let base = serve_as(&rt, &core, from);
        assert_eq!(get(&rt, &format!("{base}{hello}")).0, 200, "{from}");
    }
}

#[test]
fn without_tailscale_the_host_lists_no_devices_and_still_trusts_the_tailnet() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(b"tailscale is not running".to_vec()));
    assert!(devices(&core).is_empty());
    let base = serve_as(&rt, &core, "100.70.1.2");
    assert_eq!(
        get(&rt, &format!("{base}{}", anibeam_host_api::HELLO)).0,
        200
    );
}

/// The real listener, not a mocked address: the host started by the
/// preference hands every request its peer address, and loopback is this
/// machine, so it is answered.
#[test]
fn the_real_server_knows_who_is_asking() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    core.call(Call::SetTailnetDeviceTrust {
        device: "nPhone".into(),
        trusted: false,
    })
    .unwrap();
    core.set_host_bind(Some("127.0.0.1:0".to_string()));
    let Reply::Preferences { mut preferences } = core.call(Call::GetPreferences).unwrap() else {
        panic!()
    };
    preferences.share_over_tailscale = true;
    core.call(Call::SetPreferences { preferences }).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let base = loop {
        if let Reply::Host { status } = core.call(Call::GetHost).unwrap()
            && let Some(url) = status.urls.first()
        {
            break url.trim_end_matches('/').to_string();
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the host did not start"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(
        get(&rt, &format!("{base}{}", anibeam_host_api::HELLO)).0,
        200
    );
}

/// Hello says whether Tailscale carries the asking device through a relay,
/// as the status says of that device, and says nothing for a direct one.
#[test]
fn hello_says_when_the_path_to_the_asker_is_relayed() {
    const PATHS: &[u8] = br#"{
        "Self": {"ID": "nSelf", "HostName": "banditbox", "TailscaleIPs": ["100.65.105.9"]},
        "Peer": {
            "a": {"ID": "nPhone", "HostName": "pixel", "Online": true, "CurAddr": "", "Relay": "fra",
                  "TailscaleIPs": ["100.70.1.2"]},
            "b": {"ID": "nLaptop", "HostName": "kangaeru", "Online": true,
                  "CurAddr": "192.168.1.20:41641", "Relay": "fra", "TailscaleIPs": ["100.80.3.4"]}
        }
    }"#;
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(PATHS.to_vec()));
    let hello = |from: &str| -> serde_json::Value {
        let base = serve_as(&rt, &core, from);
        rt.block_on(async {
            reqwest::get(format!("{base}{}", anibeam_host_api::HELLO))
                .await
                .unwrap()
                .json()
                .await
                .unwrap()
        })
    };
    assert_eq!(hello("100.70.1.2")["relayed"], serde_json::json!(true));
    assert!(hello("100.80.3.4").get("relayed").is_none());
    assert!(hello("127.0.0.1").get("relayed").is_none());
}
