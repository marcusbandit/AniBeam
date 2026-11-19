//! The phone's update check end to end (#809): `CheckPhoneUpdate` asks GitHub's release
//! list over the core's transport, answers `PhoneUpdate` with the newer phone release on
//! the running build's track, asks again with the list's `ETag` so an unchanged list costs
//! an empty 304, and turns a refusal into an error the phone retries on.

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
fn a_phone_hears_of_a_newer_nightly_and_the_unchanged_list_comes_back_empty() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    http.push_for_with_headers(
        "/releases",
        200,
        list(),
        vec![("ETag".into(), "\"one\"".into())],
    );
    let update = check(&core, "2.4.3-nightly.20261116.96").unwrap().unwrap();
    assert_eq!(update.version, "2.4.3-nightly.20261117.97");
    assert_eq!(
        update.url,
        "https://example.com/Anibeam-2.4.3-nightly.20261117.97.apk"
    );

    // Unchanged since: GitHub answers 304 with nothing, and the list held answers instead
    http.push_for("/releases", 304, Vec::new());
    let again = check(&core, "2.4.3-nightly.20261116.96").unwrap().unwrap();
    assert_eq!(again.version, "2.4.3-nightly.20261117.97");

    let sent = http.requests();
    assert_eq!(sent.len(), 2);
    assert!(
        sent[0].url.contains("api.github.com/repos/"),
        "{}",
        sent[0].url
    );
    let header = |i: usize, name: &str| {
        sent[i]
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    };
    assert_eq!(
        header(0, "Accept").as_deref(),
        Some("application/vnd.github+json")
    );
    assert_eq!(header(0, "If-None-Match"), None);
    assert_eq!(header(1, "If-None-Match").as_deref(), Some("\"one\""));
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
