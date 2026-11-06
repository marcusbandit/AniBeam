//! Request proofs end to end through the host's real routes (#555). The
//! asking address is set with axum's `MockConnectInfo`, so a request "from"
//! the local network or a tailnet phone runs every check a real one would,
//! on loopback, with no tailnet at all.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anibeam_core::*;
use anibeam_host_api::proof::{self, DeviceKey};
use anibeam_host_api::{HELLO, Hello};
use axum::extract::connect_info::MockConnectInfo;

const STATUS: &[u8] = br#"{
    "Self": {"ID": "nSelf", "HostName": "banditbox", "TailscaleIPs": ["100.65.105.9"]},
    "Peer": {
        "nodekey:aa": {"ID": "nPhone", "HostName": "pixel", "OS": "android", "Online": true,
                       "TailscaleIPs": ["100.70.1.2"]}
    }
}"#;

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

/// The status and the error text of a GET of `path`, with `proof` if any.
fn get(
    rt: &tokio::runtime::Runtime,
    base: &str,
    path: &str,
    proof: Option<String>,
) -> (u16, String) {
    rt.block_on(async {
        let mut r = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap()
            .get(format!("{base}{path}"));
        if let Some(p) = proof {
            r = r.header(proof::HEADER, p);
        }
        let r = r.send().await.unwrap();
        let status = r.status().as_u16();
        let body: serde_json::Value = r.json().await.unwrap_or_default();
        (
            status,
            body["error"].as_str().unwrap_or_default().to_string(),
        )
    })
}

fn hello(rt: &tokio::runtime::Runtime, base: &str) -> Hello {
    rt.block_on(async {
        reqwest::get(format!("{base}{HELLO}"))
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    })
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[test]
fn the_host_says_its_key_and_keeps_it() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let said = hello(&rt, &serve_as(&rt, &core, "100.70.1.2"));
    assert!(
        said.features
            .iter()
            .any(|f| f == anibeam_host_api::FEATURE_PROOF)
    );
    assert_eq!(said.key.len(), 64, "{said:?}");
    let again = hello(&rt, &serve_as(&rt, &core, "100.70.1.2"));
    assert_eq!(said.key, again.key);
}

#[test]
fn a_paired_device_is_answered_where_its_address_alone_is_not() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let host = hello(&rt, &serve_as(&rt, &core, "100.70.1.2")).key;
    let phone = DeviceKey::from_seed([7; 32]);
    let stranger = DeviceKey::from_seed([8; 32]);
    let lan = serve_as(&rt, &core, "192.168.1.9");

    // Off the tailnet and unpaired: refused, with or without a proof.
    assert_eq!(get(&rt, &lan, HELLO, None).0, 403);
    let unpaired = phone.sign(&host, now(), "n1", "GET", HELLO);
    assert_eq!(get(&rt, &lan, HELLO, Some(unpaired)).0, 403);

    anibeam_core::host::keys::pair(&core, &phone.public(), "pixel").unwrap();
    let paired = phone.sign(&host, now(), "n2", "GET", HELLO);
    assert_eq!(get(&rt, &lan, HELLO, Some(paired.clone())).0, 200);

    // The same proof again, a stale one, and one signed by another key under
    // the phone's name are all refused.
    let (status, why) = get(&rt, &lan, HELLO, Some(paired));
    assert_eq!(status, 403);
    assert!(why.contains("already used"), "{why}");
    let stale = phone.sign(&host, now() - proof::FRESH_FOR - 60, "n3", "GET", HELLO);
    let (status, why) = get(&rt, &lan, HELLO, Some(stale));
    assert_eq!(status, 403);
    assert!(why.contains("too old"), "{why}");
    let forged = stranger.sign(&host, now(), "n4", "GET", HELLO);
    let forged = forged.replacen(&stranger.public(), &phone.public(), 1);
    let (status, why) = get(&rt, &lan, HELLO, Some(forged));
    assert_eq!(status, 403);
    assert!(why.contains("did not check out"), "{why}");

    // A proof for one route does not open another.
    let snapshot = anibeam_host_api::SNAPSHOT;
    let for_hello = phone.sign(&host, now(), "n5", "GET", HELLO);
    assert_eq!(get(&rt, &lan, snapshot, Some(for_hello)).0, 403);

    // Unpairing closes it again.
    anibeam_core::host::keys::unpair(&core, &phone.public()).unwrap();
    let after = phone.sign(&host, now(), "n6", "GET", HELLO);
    assert_eq!(get(&rt, &lan, HELLO, Some(after)).0, 403);
}

#[test]
fn a_tailnet_device_is_answered_without_a_proof_but_never_with_a_bad_one() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let tailnet = serve_as(&rt, &core, "100.70.1.2");
    let host = hello(&rt, &tailnet).key;
    // An old phone signs nothing and is answered as before.
    assert_eq!(get(&rt, &tailnet, HELLO, None).0, 200);
    let phone = DeviceKey::from_seed([7; 32]);
    let good = phone.sign(&host, now(), "n1", "GET", HELLO);
    assert_eq!(get(&rt, &tailnet, HELLO, Some(good)).0, 200);
    let other_host = DeviceKey::from_seed([1; 32]).public();
    let wrong = phone.sign(&other_host, now(), "n2", "GET", HELLO);
    assert_eq!(get(&rt, &tailnet, HELLO, Some(wrong)).0, 403);
    assert_eq!(get(&rt, &tailnet, HELLO, Some("v1 junk".into())).0, 403);
}

#[test]
fn a_switched_off_device_stays_off_even_when_paired() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (_dir, core, _c) = common::open_core();
    core.set_tailscale_status(Some(STATUS.to_vec()));
    let tailnet = serve_as(&rt, &core, "100.70.1.2");
    let host = hello(&rt, &tailnet).key;
    let phone = DeviceKey::from_seed([7; 32]);
    anibeam_core::host::keys::pair(&core, &phone.public(), "pixel").unwrap();
    core.call(Call::SetTailnetDeviceTrust {
        device: "nPhone".into(),
        trusted: false,
    })
    .unwrap();
    let paired = phone.sign(&host, now(), "n1", "GET", HELLO);
    assert_eq!(get(&rt, &tailnet, HELLO, Some(paired)).0, 403);
}
