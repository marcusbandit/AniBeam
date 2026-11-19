//! The phone's update check end to end (#809): `CheckPhoneUpdate` asks GitHub's release
//! list over the core's transport, answers `PhoneUpdate` with the newer phone release on
//! the running build's track, turns a refusal into an error the phone retries on, and
//! answers from a runtime worker as well as from a shell's own thread.

mod common;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;
use serde_json::json;

fn list() -> Vec<u8> {
    json!([
        { "tag_name": "v2.5.0-nightly.20261117.97", "html_url": "desktop", "prerelease": true },
        {
            "tag_name": "android-v2.4.3-nightly.20261117.97",
            "html_url": "https://example.com/release/97",
            "prerelease": true,
            "assets": [{
                "name": "Anibeam-2.4.3-nightly.20261117.97.apk", "size": 9,
                "browser_download_url": "https://example.com/Anibeam-2.4.3-nightly.20261117.97.apk"
            }]
        }
    ])
    .to_string()
    .into_bytes()
}

fn check(core: &Core, current: &str) -> Result<Option<PhoneUpdate>, CoreError> {
    match core.call(Call::CheckPhoneUpdate {
        current: current.into(),
    })? {
        Reply::PhoneUpdate { update } => Ok(update),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_phone_hears_of_a_newer_nightly_and_its_apk() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    http.push_for("/releases", 200, list());
    let update = check(&core, "2.4.3-nightly.20261116.96").unwrap().unwrap();
    assert_eq!(update.version, "2.4.3-nightly.20261117.97");
    assert_eq!(
        update.url,
        "https://example.com/Anibeam-2.4.3-nightly.20261117.97.apk"
    );
    let sent = http.requests();
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0].url.contains("api.github.com/repos/"),
        "{}",
        sent[0].url
    );
    let accept = sent[0]
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("Accept"))
        .map(|(_, v)| v.as_str());
    assert_eq!(accept, Some("application/vnd.github+json"));
}

/// A listener runs on a runtime worker, where a plain `block_on` panics; the check has
/// to answer there too.
#[test]
fn the_check_answers_from_a_runtime_worker() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    http.push_for("/releases", 200, list());
    let shell = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();
    // A clone goes to the worker, so the core is never dropped inside the runtime
    let on_worker = core.clone();
    let answer = shell
        .block_on(tokio::spawn(async move {
            check(&on_worker, "2.4.3-nightly.20261116.96")
        }))
        .unwrap();
    assert_eq!(
        answer.unwrap().map(|u| u.version).as_deref(),
        Some("2.4.3-nightly.20261117.97")
    );
}

#[test]
fn an_up_to_date_phone_hears_nothing() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    http.push_for("/releases", 200, list());
    assert_eq!(check(&core, "2.4.3-nightly.20261117.97").unwrap(), None);
}

#[test]
fn a_refused_list_is_an_error_not_up_to_date() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    http.push_for("/releases", 403, b"{\"message\":\"rate limited\"}".to_vec());
    assert!(check(&core, "2.4.3-nightly.20261116.96").is_err());
}
