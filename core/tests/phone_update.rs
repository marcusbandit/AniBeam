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
    check_on(core, current, None)
}

fn check_on(
    core: &Core,
    current: &str,
    nightly: Option<bool>,
) -> Result<Option<PhoneUpdate>, CoreError> {
    match core.call(Call::CheckPhoneUpdate {
        current: current.into(),
        nightly,
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
        sent[0].url.contains("api.github.com/repos/") && sent[0].url.contains("per_page=100"),
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
    let task = shell.spawn(async move { check(&on_worker, "2.4.3-nightly.20261116.96") });
    let answer = shell.block_on(task).unwrap();
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

/// A caller on a current-thread runtime, a `#[tokio::test]` say, gets an answer too. The
/// core is made and dropped outside that runtime; only the call is made inside it.
#[test]
fn the_check_answers_on_a_current_thread_runtime() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    http.push_for("/releases", 200, list());
    let caller = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let answer = caller
        .block_on(async { check(&core, "2.4.3-nightly.20261116.96") })
        .unwrap();
    assert_eq!(
        answer.map(|u| u.version).as_deref(),
        Some("2.4.3-nightly.20261117.97")
    );
}

/// The track the phone keeps decides, not the running build: a nightly phone on the stable
/// release hears of the next nightly, and a stable phone does not.
#[test]
fn the_kept_track_decides_what_is_offered() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let list = json!([
        { "tag_name": "android-v2.4.4-nightly.20261201.120", "html_url": "https://example.com/release/120" },
        { "tag_name": "android-v2.4.3", "html_url": "https://example.com/release/243" }
    ])
    .to_string();
    http.push_for("/releases", 200, list.clone());
    http.push_for("/releases", 200, list);
    let nightly = check_on(&core, "2.4.3", Some(true)).unwrap();
    assert_eq!(
        nightly.map(|u| u.version).as_deref(),
        Some("2.4.4-nightly.20261201.120")
    );
    assert_eq!(check_on(&core, "2.4.3", Some(false)).unwrap(), None);
}

/// The list is asked for gzip'd, and a gzip'd answer is unpacked before it is read.
#[test]
fn the_list_comes_gzipped_and_is_unpacked() {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(&list()).unwrap();
    http.push_for_with_headers(
        "/releases",
        200,
        gz.finish().unwrap(),
        vec![("Content-Encoding".into(), "gzip".into())],
    );
    let update = check(&core, "2.4.3-nightly.20261116.96").unwrap();
    assert_eq!(
        update.map(|u| u.version).as_deref(),
        Some("2.4.3-nightly.20261117.97")
    );
    let sent = http.requests();
    let asked = sent[0]
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("Accept-Encoding"))
        .map(|(_, v)| v.as_str());
    assert_eq!(asked, Some("gzip"));
}
