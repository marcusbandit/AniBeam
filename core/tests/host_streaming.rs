//! "Stream episodes" off: the host still shares the library, but its
//! episodes, their conversions and the playback calls refuse, and its hello
//! says why. Served on loopback through the real router.

mod common;

use std::sync::Arc;
use std::time::Duration;

use anibeam_core::*;
use anibeam_host_api as wire;
use axum::extract::connect_info::MockConnectInfo;

/// Serves the router as if every request came from this machine, which the
/// host always answers, so only the streaming switch decides.
fn serve(rt: &tokio::runtime::Runtime, core: &Arc<Core>) -> String {
    let me: std::net::SocketAddr = "127.0.0.1:40000".parse().unwrap();
    let app = anibeam_core::host::server::router(Arc::downgrade(core)).layer(MockConnectInfo(me));
    rt.block_on(async {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let at = l.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(l, app).await });
        format!("http://{at}")
    })
}

fn get(rt: &tokio::runtime::Runtime, url: &str) -> (u16, serde_json::Value) {
    rt.block_on(async {
        let r = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .unwrap()
            .get(url)
            .send()
            .await
            .unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or_default())
    })
}

fn post(rt: &tokio::runtime::Runtime, url: &str, call: &str) -> u16 {
    rt.block_on(async {
        reqwest::Client::new()
            .post(url)
            .header("content-type", "application/json")
            .body(call.to_string())
            .send()
            .await
            .unwrap()
            .status()
            .as_u16()
    })
}

fn set_streaming(core: &Core, on: bool) {
    let Reply::Preferences { mut preferences } = core.call(Call::GetPreferences).unwrap() else {
        panic!()
    };
    preferences.stream_over_tailscale = on;
    core.call(Call::SetPreferences { preferences }).unwrap();
}

#[test]
fn streaming_is_on_until_it_is_switched_off() {
    let (_dir, core, _c) = common::open_core();
    let Reply::Preferences { preferences } = core.call(Call::GetPreferences).unwrap() else {
        panic!()
    };
    assert!(preferences.stream_over_tailscale);
    // A preference blob from before the switch reads it as on.
    let mut blob = serde_json::to_value(Preferences::default()).unwrap();
    blob.as_object_mut()
        .unwrap()
        .remove("stream_over_tailscale");
    let old: Preferences = serde_json::from_value(blob).unwrap();
    assert!(old.stream_over_tailscale);
}

#[test]
fn with_streaming_off_only_the_library_answers() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (dir, core, _c) = common::open_core();
    let base = serve(&rt, &core);
    set_streaming(&core, false);

    let (status, hello) = get(&rt, &format!("{base}{}", wire::HELLO));
    assert_eq!(status, 200);
    let hello: wire::Hello = serde_json::from_value(hello).unwrap();
    assert!(hello.streaming_off);
    assert_eq!(
        hello.features,
        [wire::FEATURE_PEERS, wire::FEATURE_HISTORY],
        "only what streaming does not cover"
    );
    assert_eq!(get(&rt, &format!("{base}{}", wire::SNAPSHOT)).0, 200);

    for route in [
        wire::file_path(1),
        wire::sidecar_path(1, 0),
        wire::probe_path(1),
        wire::subtitle_path(1, 0),
        wire::hls_path(1, 720, None, None),
        wire::skip_path(1),
    ] {
        let (status, body) = get(&rt, &format!("{base}{route}"));
        assert_eq!(status, 403, "{route}");
        assert!(
            body["error"]
                .as_str()
                .unwrap_or_default()
                .contains("Streaming is off"),
            "{route}: {body}"
        );
    }
    let call = format!("{base}{}", wire::CALL);
    assert_eq!(
        post(
            &rt,
            &call,
            r#"{"Tick":{"session":1,"position":3.0,"paused":false}}"#
        ),
        403
    );
    assert_eq!(post(&rt, &call, r#""ListNotInterested""#), 200);
    // Not interested is not streaming: hiding a show still reaches the host.
    let hid = post(
        &rt,
        &call,
        r#"{"SetNotInterested":{"anilist_id":1,"not_interested":true}}"#,
    );
    assert_ne!(hid, 403, "SetNotInterested passes while streaming is off");

    // The next core on the same data starts with streaming off.
    let paths = CorePaths::under(dir.path());
    let secrets = anibeam_core::trackers::Secrets::file_only(paths.secrets_path());
    let again = Core::open_with_secrets(paths, secrets).unwrap();
    anibeam_core::host::sync(&again);
    let reopened = serve(&rt, &again);
    assert_eq!(
        get(&rt, &format!("{reopened}{}", wire::file_path(1))).0,
        403
    );

    // On again, hello offers streaming and the routes answer as they did.
    set_streaming(&core, true);
    let (_, hello) = get(&rt, &format!("{base}{}", wire::HELLO));
    assert!(hello.get("streaming_off").is_none(), "{hello}");
    let hello: wire::Hello = serde_json::from_value(hello).unwrap();
    assert!(hello.features.iter().any(|f| f == wire::FEATURE_STREAM));
    assert_eq!(get(&rt, &format!("{base}{}", wire::file_path(1))).0, 404);
}
