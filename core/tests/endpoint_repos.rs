//! Endpoint repos (#143) against copies of the owner's two published groups,
//! `index/general.json` and `index/nsfw.json` in marcusbandit/anibeam-endpoints, and against
//! indexes the test signs itself for what those two cannot show: a changed key, a stale
//! serial, an update.

mod common;

use std::path::{Path, PathBuf};

use anibeam_core::net::FakeHttp;
use anibeam_core::{
    Call, CoreError, EndpointCatalog, EndpointCredential, EndpointRepoPreview, Reply,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{Value, json};

fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/endpoint-repos")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn preview(core: &anibeam_core::Core, url: &str) -> EndpointRepoPreview {
    match core
        .call(Call::AddEndpointRepo { url: url.into() })
        .unwrap()
    {
        Reply::EndpointRepoPreview { preview } => preview,
        other => panic!("{other:?}"),
    }
}

fn catalog(reply: Result<Reply, CoreError>) -> EndpointCatalog {
    match reply.unwrap() {
        Reply::EndpointCatalog { catalog } => catalog,
        other => panic!("{other:?}"),
    }
}

fn trust(core: &anibeam_core::Core, url: &str) -> EndpointCatalog {
    let p = preview(core, url);
    catalog(core.call(Call::TrustEndpointRepo {
        url: url.into(),
        key: p.key,
    }))
}

fn token(value: &str) -> Vec<EndpointCredential> {
    vec![EndpointCredential {
        key: "token".into(),
        value: value.into(),
    }]
}

fn service_code(e: &CoreError) -> &str {
    match e {
        CoreError::Service { code, .. } => code,
        other => panic!("not a service error: {other:?}"),
    }
}

fn install_id(c: &EndpointCatalog, id: &str) -> String {
    c.installed
        .iter()
        .find(|i| i.id == id)
        .unwrap_or_else(|| panic!("{id} not installed"))
        .install_id
        .clone()
}

// The owner's published groups -------------------------------------------------------------

#[test]
fn the_general_group_reads_and_verifies() {
    let (_dir, core, _) = common::open_core();
    let p = preview(&core, &fixture("general.json"));
    assert_eq!(p.name, "Bandit's endpoints");
    assert!(!p.nsfw);
    assert!(!p.trusted);
    assert_eq!(p.fingerprint.split(' ').count(), 8);
    let mut ids: Vec<&str> = p.endpoints.iter().map(|e| e.id.as_str()).collect();
    ids.sort();
    assert_eq!(
        ids,
        [
            "animetosho",
            "erai",
            "knaben",
            "limetorrents",
            "nyaa",
            "subsplease"
        ]
    );
    let erai = p.endpoints.iter().find(|e| e.id == "erai").unwrap();
    assert_eq!(erai.auth.len(), 1);
    assert_eq!(erai.auth[0].key, "token");
    assert!(erai.auth[0].secret);
    let nyaa = p.endpoints.iter().find(|e| e.id == "nyaa").unwrap();
    assert_eq!(nyaa.front.as_deref(), Some("ddos-guard.net"));
    assert_eq!(nyaa.host, "nyaa.si");
}

#[test]
fn the_nsfw_group_is_its_own_repo_under_the_same_key() {
    let (_dir, core, _) = common::open_core();
    let p = preview(&core, &fixture("nsfw.json"));
    assert!(p.nsfw);
    assert_eq!(p.endpoints.len(), 1);
    let sukebei = &p.endpoints[0];
    assert_eq!(sukebei.id, "sukebei");
    assert!(sukebei.nsfw);
    assert_eq!(sukebei.front, None);

    trust(&core, &fixture("general.json"));
    let c = trust(&core, &fixture("nsfw.json"));
    assert_eq!(c.repos.len(), 2, "keyed by index URL, not by repo");
    assert_eq!(c.repos[0].fingerprint, c.repos[1].fingerprint);
    assert_eq!(c.offered.len(), 7);
}

#[test]
fn nothing_is_saved_until_the_repo_is_trusted() {
    let (_dir, core, _) = common::open_core();
    preview(&core, &fixture("general.json"));
    assert!(catalog(core.call(Call::ListRepoEndpoints)).repos.is_empty());
}

#[test]
fn trusting_needs_the_key_that_was_shown() {
    let (_dir, core, _) = common::open_core();
    let url = fixture("general.json");
    preview(&core, &url);
    let e = core
        .call(Call::TrustEndpointRepo {
            url: url.clone(),
            key: "ed25519:AAAA".into(),
        })
        .unwrap_err();
    assert_eq!(service_code(&e), "key_changed");
}

// Installing -------------------------------------------------------------------------------

#[test]
fn an_endpoint_without_auth_installs_and_joins_the_search() {
    let (_dir, core, _) = common::open_core();
    trust(&core, &fixture("general.json"));
    let c = catalog(core.call(Call::InstallEndpoint {
        repo: fixture("general.json"),
        id: "knaben".into(),
        credentials: vec![],
    }));
    let knaben = c.installed.iter().find(|i| i.id == "knaben").unwrap();
    assert!(knaben.enabled && knaben.missing.is_empty() && knaben.offered);
    assert_eq!(
        c.offered
            .iter()
            .find(|o| o.id == "knaben")
            .unwrap()
            .installed
            .as_deref(),
        Some(knaben.install_id.as_str())
    );
    let Reply::PhoneSources { sources } = core.call(Call::PhoneSearchSources).unwrap() else {
        panic!()
    };
    let names: Vec<&str> = sources.iter().map(|s| s.name.as_str()).collect();
    // Nothing is preloaded: the installed endpoint is the whole list.
    assert_eq!(names, ["knaben"]);
}

#[test]
fn an_installed_endpoint_takes_the_place_of_a_source_with_its_name() {
    let (_dir, core, _) = common::open_core();
    // A list saved before repos existed.
    let saved = |name: &str, kind: &str, url: &str| anibeam_core::PhoneSource {
        name: name.into(),
        kind: kind.into(),
        url: url.into(),
        enabled: true,
    };
    core.call(Call::SetPhoneSources {
        sources: vec![
            saved("nyaa", "nyaa", "https://nyaa.si/"),
            saved(
                "animetosho",
                "tosho",
                "https://animetosho.org/feed/json?q={query}&old=1",
            ),
        ],
    })
    .unwrap();
    trust(&core, &fixture("general.json"));
    for id in ["animetosho", "nyaa"] {
        core.call(Call::InstallEndpoint {
            repo: fixture("general.json"),
            id: id.into(),
            credentials: vec![],
        })
        .unwrap();
    }
    let Reply::PhoneSources { sources } = core.call(Call::PhoneSearchSources).unwrap() else {
        panic!()
    };
    let names: Vec<&str> = sources.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["nyaa", "animetosho"]);
    assert_eq!(sources[1].url, "https://animetosho.org/feed/json?q={query}");
}

#[test]
fn a_phone_source_that_searches_by_address_needs_the_query() {
    let (_dir, core, _) = common::open_core();
    let source = |kind: &str, url: &str| anibeam_core::PhoneSource {
        name: "feed".into(),
        kind: kind.into(),
        url: url.into(),
        enabled: true,
    };
    for kind in ["tosho", "rss", "torznab"] {
        let err = core
            .call(Call::SetPhoneSources {
                sources: vec![source(kind, "https://feed.example/json")],
            })
            .unwrap_err();
        assert!(
            matches!(&err, CoreError::Invalid { message, .. } if message.contains("{query}")),
            "{kind}: {err:?}"
        );
        core.call(Call::SetPhoneSources {
            sources: vec![source(kind, "https://feed.example/json?q={query}")],
        })
        .unwrap();
    }
}

#[test]
fn a_tosho_source_saved_without_the_query_does_not_block_removing_others() {
    let (dir, core, _) = common::open_core();
    let source = |name: &str, kind: &str, url: &str| anibeam_core::PhoneSource {
        name: name.into(),
        kind: kind.into(),
        url: url.into(),
        enabled: true,
    };
    let old = source("old", "tosho", "https://feed.example/json");
    let nyaa = source("nyaa", "nyaa", "https://nyaa.si/");
    // Saved before the rule covered tosho.
    let config = anibeam_core::CorePaths::under(dir.path()).config_dir;
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(
        std::path::Path::new(&config).join("phone-sources.json"),
        serde_json::to_vec(&[&old, &nyaa]).unwrap(),
    )
    .unwrap();
    core.call(Call::SetPhoneSources {
        sources: vec![old.clone()],
    })
    .unwrap();
}

#[test]
fn erai_needs_its_token_and_the_token_never_reaches_the_database() {
    let (dir, core, _) = common::open_core();
    trust(&core, &fixture("general.json"));
    let e = core
        .call(Call::InstallEndpoint {
            repo: fixture("general.json"),
            id: "erai".into(),
            credentials: vec![],
        })
        .unwrap_err();
    assert!(matches!(e, CoreError::Invalid { .. }), "{e:?}");

    let secret = "sekrit-erai-token-1234";
    let c = catalog(core.call(Call::InstallEndpoint {
        repo: fixture("general.json"),
        id: "erai".into(),
        credentials: token(secret),
    }));
    let erai = c.installed.iter().find(|i| i.id == "erai").unwrap();
    assert!(erai.missing.is_empty());

    let Reply::PhoneSources { sources } = core.call(Call::PhoneSearchSources).unwrap() else {
        panic!()
    };
    let shown = sources.iter().find(|s| s.name == "erai").unwrap();
    assert!(shown.url.contains("token={token}"), "{}", shown.url);
    assert!(!shown.url.contains(secret));

    let secrets = std::fs::read_to_string(dir.path().join("data/secrets.json")).unwrap();
    assert!(secrets.contains(secret));
    assert!(secrets.contains(&format!("endpoint/{}/token", erai.install_id)));
    core.shutdown();
    // The database and its journal files, beside secrets.json in the data folder.
    for f in std::fs::read_dir(dir.path().join("data")).unwrap() {
        let path = f.unwrap().path();
        if !path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("anibeam.db")
        {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            !bytes.windows(secret.len()).any(|w| w == secret.as_bytes()),
            "the token is in {}",
            path.display()
        );
    }
}

fn find(root: &Path, name: &str) -> Option<PathBuf> {
    walk(root)
        .into_iter()
        .find(|p| p.file_name().is_some_and(|n| n == name))
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return out;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else {
            out.push(p);
        }
    }
    out
}

#[test]
fn a_search_through_erai_sends_the_token_and_the_activity_log_never_shows_it() {
    let http = FakeHttp::new();
    http.push_for(
        "erai-raws.info",
        200,
        "<?xml version=\"1.0\"?><rss><channel><title>t</title></channel></rss>",
    );
    let (_dir, core, events) = common::open_core_with_http(http.clone());
    trust(&core, &fixture("general.json"));
    let secret = "tok en&1";
    core.call(Call::InstallEndpoint {
        repo: fixture("general.json"),
        id: "erai".into(),
        credentials: token(secret),
    })
    .unwrap();
    let _ = core.call(Call::PhoneSearch {
        query: "Frieren".into(),
        endpoint: Some("erai".into()),
        fallback: false,
    });
    let sent: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
    let erai = sent
        .iter()
        .find(|u| u.contains("erai-raws.info"))
        .unwrap_or_else(|| panic!("no erai request in {sent:?}"));
    assert!(erai.contains("token=tok%20en%261"), "{erai}");
    assert!(erai.contains("s=Frieren"), "{erai}");
    assert!(events.wait_for(
        |evs| format!("{evs:?}").contains("Installed the endpoint Erai-raws"),
        std::time::Duration::from_secs(5)
    ));
    assert!(!format!("{:?}", events.events()).contains("tok en"));
}

#[test]
fn removing_an_endpoint_takes_its_credentials_too() {
    let (dir, core, _) = common::open_core();
    trust(&core, &fixture("general.json"));
    let c = catalog(core.call(Call::InstallEndpoint {
        repo: fixture("general.json"),
        id: "erai".into(),
        credentials: token("gone-after-remove"),
    }));
    let id = install_id(&c, "erai");
    let c = catalog(core.call(Call::RemoveEndpoint { install: id }));
    assert!(c.installed.is_empty());
    let secrets = find(dir.path(), "secrets.json")
        .map(|p| std::fs::read_to_string(p).unwrap())
        .unwrap_or_default();
    assert!(!secrets.contains("gone-after-remove"));
}

#[test]
fn a_disabled_endpoint_is_left_out_of_the_search() {
    let (_dir, core, _) = common::open_core();
    trust(&core, &fixture("general.json"));
    let c = catalog(core.call(Call::InstallEndpoint {
        repo: fixture("general.json"),
        id: "knaben".into(),
        credentials: vec![],
    }));
    core.call(Call::SetEndpointEnabled {
        install: install_id(&c, "knaben"),
        enabled: false,
    })
    .unwrap();
    let Reply::PhoneSources { sources } = core.call(Call::PhoneSearchSources).unwrap() else {
        panic!()
    };
    assert!(sources.iter().all(|s| s.name != "knaben"));
}

// Trust over time: indexes the test signs ---------------------------------------------------

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn key_text(k: &SigningKey) -> String {
    format!("ed25519:{}", B64.encode(k.verifying_key().to_bytes()))
}

fn endpoint(id: &str, version: u32, url: &str, auth: bool) -> Value {
    let mut d = json!({"id": id, "name": id, "version": version, "kind": "rss", "url": url});
    if auth {
        d["auth"] = json!([{"key": "token", "label": "Token", "secret": true}]);
    }
    d
}

fn write_repo(path: &Path, k: &SigningKey, serial: u64, endpoints: Vec<Value>) {
    let index = json!({"name": "Signed here", "key": key_text(k), "serial": serial, "endpoints": endpoints});
    let payload = serde_json::to_vec(&index).unwrap();
    let file = json!({
        "format": "anibeam-endpoint-repo",
        "version": 1,
        "payload": B64.encode(&payload),
        "signature": B64.encode(k.sign(&payload).to_bytes()),
    });
    std::fs::write(path, serde_json::to_vec(&file).unwrap()).unwrap();
}

#[test]
fn a_new_key_is_refused_and_the_last_good_index_stays() {
    let (dir, core, _) = common::open_core();
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    write_repo(
        &path,
        &key(1),
        10,
        vec![endpoint("a", 1, "https://a.example/?q={query}", false)],
    );
    trust(&core, &url);

    write_repo(
        &path,
        &key(2),
        11,
        vec![endpoint("b", 1, "https://b.example/?q={query}", false)],
    );
    let c = catalog(core.call(Call::RefreshEndpointRepos));
    let repo = &c.repos[0];
    assert!(repo.error.as_deref().unwrap().contains("different key"));
    assert_eq!(repo.serial, 10);
    assert_eq!(c.offered.len(), 1);
    assert_eq!(c.offered[0].id, "a");

    // Back under the trusted key, the refresh goes through and clears the error.
    write_repo(
        &path,
        &key(1),
        12,
        vec![endpoint("b", 1, "https://b.example/?q={query}", false)],
    );
    let c = catalog(core.call(Call::RefreshEndpointRepos));
    assert_eq!(c.repos[0].error, None);
    assert_eq!(c.repos[0].serial, 12);
    assert_eq!(c.offered[0].id, "b");
}

#[test]
fn an_older_serial_is_refused() {
    let (dir, core, _) = common::open_core();
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    write_repo(
        &path,
        &key(1),
        20,
        vec![endpoint("a", 1, "https://a.example/?q={query}", false)],
    );
    trust(&core, &url);
    write_repo(&path, &key(1), 19, vec![]);
    let c = catalog(core.call(Call::RefreshEndpointRepos));
    assert!(c.repos[0].error.as_deref().unwrap().contains("older index"));
    assert_eq!(c.offered.len(), 1);
}

#[test]
fn a_tampered_file_is_refused_on_refresh() {
    let (dir, core, _) = common::open_core();
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    write_repo(&path, &key(1), 20, vec![]);
    trust(&core, &url);
    let mut file: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    file["signature"] = B64.encode([0u8; 64]).into();
    std::fs::write(&path, serde_json::to_vec(&file).unwrap()).unwrap();
    let c = catalog(core.call(Call::RefreshEndpointRepos));
    assert!(c.repos[0].error.as_deref().unwrap().contains("signature"));
}

#[test]
fn an_update_on_the_same_host_keeps_the_token() {
    let (dir, core, _) = common::open_core();
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    let v1 = "https://a.example/?q={query}&token={token}";
    write_repo(&path, &key(1), 1, vec![endpoint("a", 1, v1, true)]);
    trust(&core, &url);
    let c = catalog(core.call(Call::InstallEndpoint {
        repo: url.clone(),
        id: "a".into(),
        credentials: token("kept"),
    }));
    let id = install_id(&c, "a");

    write_repo(
        &path,
        &key(1),
        2,
        vec![endpoint(
            "a",
            2,
            "https://a.example/v2?q={query}&token={token}",
            true,
        )],
    );
    let c = catalog(core.call(Call::RefreshEndpointRepos));
    let a = &c.installed[0];
    assert_eq!((a.update, a.update_asks), (Some(2), false));
    let c = catalog(core.call(Call::UpdateEndpoint {
        install: id,
        credentials: vec![],
    }));
    let a = &c.installed[0];
    assert_eq!((a.version, a.update), (2, None));
    assert!(a.missing.is_empty());
}

#[test]
fn an_update_that_moves_the_token_to_another_host_asks_again() {
    let (dir, core, _) = common::open_core();
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    write_repo(
        &path,
        &key(1),
        1,
        vec![endpoint(
            "a",
            1,
            "https://a.example/?q={query}&token={token}",
            true,
        )],
    );
    trust(&core, &url);
    let c = catalog(core.call(Call::InstallEndpoint {
        repo: url.clone(),
        id: "a".into(),
        credentials: token("old-host-token"),
    }));
    let id = install_id(&c, "a");

    write_repo(
        &path,
        &key(1),
        2,
        vec![endpoint(
            "a",
            2,
            "https://elsewhere.example/?q={query}&token={token}",
            true,
        )],
    );
    let c = catalog(core.call(Call::RefreshEndpointRepos));
    let a = &c.installed[0];
    assert!(a.update_asks);
    assert_eq!(a.update_host.as_deref(), Some("elsewhere.example"));
    let e = core
        .call(Call::UpdateEndpoint {
            install: id.clone(),
            credentials: vec![],
        })
        .unwrap_err();
    assert!(matches!(e, CoreError::Invalid { .. }));
    // Until the person answers, the installed version keeps running against the old host.
    assert_eq!(
        catalog(core.call(Call::ListRepoEndpoints)).installed[0].version,
        1
    );

    let c = catalog(core.call(Call::UpdateEndpoint {
        install: id,
        credentials: token("new-host-token"),
    }));
    assert_eq!(c.installed[0].version, 2);
    assert_eq!(c.installed[0].host, "elsewhere.example");
}

#[test]
fn an_endpoint_the_repo_stops_offering_stays_installed() {
    let (dir, core, _) = common::open_core();
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    write_repo(
        &path,
        &key(1),
        1,
        vec![endpoint("a", 1, "https://a.example/?q={query}", false)],
    );
    trust(&core, &url);
    core.call(Call::InstallEndpoint {
        repo: url.clone(),
        id: "a".into(),
        credentials: vec![],
    })
    .unwrap();
    write_repo(&path, &key(1), 2, vec![]);
    let c = catalog(core.call(Call::RefreshEndpointRepos));
    assert_eq!(c.installed.len(), 1);
    assert!(!c.installed[0].offered);

    // Removing the repo leaves it installed as well.
    let c = catalog(core.call(Call::RemoveEndpointRepo { url }));
    assert!(c.repos.is_empty());
    assert_eq!(c.installed.len(), 1);
}

#[test]
fn a_json_endpoint_searches_through_the_map_its_definition_supplies() {
    let http = FakeHttp::new();
    let body = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/json-search.json"),
    )
    .unwrap();
    http.push_for("search.example.org", 200, body);
    let (dir, core, _) = common::open_core_with_http(http.clone());
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    write_repo(
        &path,
        &key(1),
        1,
        vec![json!({
            "id": "example-json", "name": "Example search", "version": 1, "kind": "json",
            "url": "https://search.example.org/api?q={query}&limit=50",
            "json": {
                "items": "data.hits", "title": "label", "hash": "torrent.infohash",
                "size": "torrent.bytes", "page": "https://example.org/view/{ref.post}",
                "skip": "flags.hidden", "error": "status.failed",
            },
        })],
    );
    trust(&core, &url);
    core.call(Call::InstallEndpoint {
        repo: url,
        id: "example-json".into(),
        credentials: vec![],
    })
    .unwrap();

    let Reply::PhoneSearch { search } = core
        .call(Call::PhoneSearch {
            query: "Show -HEVC".into(),
            endpoint: Some("example-json".into()),
            fallback: false,
        })
        .unwrap()
    else {
        panic!()
    };
    let sent: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(
        sent,
        ["https://search.example.org/api?q=Show&limit=50"],
        "the exclusion is applied here, not sent"
    );
    assert_eq!(search.endpoint, "example-json");
    // Auto resolution narrows to 1080p, which takes the 720p release off; the HEVC one is
    // excluded, and the hidden and hashless ones never arrive at all.
    assert_eq!(search.auto_resolution.as_deref(), Some("1080p"));
    let titles: Vec<&str> = search.items.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(titles, ["[Group] Show - 01 (1080p)"]);
    assert!(
        search.items[0]
            .link
            .starts_with("magnet:?xt=urn:btih:00112233445566778899aabbccddeeff00112233&dn="),
        "{}",
        search.items[0].link
    );
    assert_eq!(search.items[0].page, "");
    assert_eq!(search.items[0].source, "example-json");
}

#[test]
fn a_json_endpoint_without_its_map_is_left_out_of_the_repo() {
    let (dir, core, _) = common::open_core();
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    write_repo(
        &path,
        &key(1),
        1,
        vec![json!({
            "id": "example-json", "name": "Example search", "version": 1, "kind": "json",
            "url": "https://search.example.org/api?q={query}",
        })],
    );
    trust(&core, &url);
    assert!(
        core.call(Call::InstallEndpoint {
            repo: url,
            id: "example-json".into(),
            credentials: vec![],
        })
        .is_err()
    );
}

#[test]
fn a_phone_source_of_the_json_kind_is_refused() {
    let (_dir, core, _) = common::open_core();
    let err = core
        .call(Call::SetPhoneSources {
            sources: vec![anibeam_core::PhoneSource {
                name: "feed".into(),
                kind: "json".into(),
                url: "https://search.example.org/api?q={query}".into(),
                enabled: true,
            }],
        })
        .unwrap_err();
    assert!(
        matches!(&err, CoreError::Invalid { message, .. } if message.contains("endpoint repo")),
        "{err:?}"
    );
}

#[test]
fn a_missing_repo_file_says_so() {
    let (dir, core, _) = common::open_core();
    let e = core
        .call(Call::AddEndpointRepo {
            url: dir.path().join("nope.json").to_string_lossy().into_owned(),
        })
        .unwrap_err();
    assert_eq!(service_code(&e), "missing");
}

#[test]
fn a_repo_over_https_is_fetched_through_the_core_client() {
    let http = FakeHttp::new();
    let body = std::fs::read(fixture("general.json")).unwrap();
    http.push_for("raw.example", 200, body);
    let (_dir, core, _) = common::open_core_with_http(http.clone());
    let p = preview(&core, "https://raw.example/index/general.json");
    assert_eq!(p.endpoints.len(), 6);
    assert_eq!(http.requests().len(), 1);
}

// Endpoints added by hand -------------------------------------------------------------------

fn add_custom(core: &anibeam_core::Core, name: &str, url: &str) -> Result<Reply, CoreError> {
    core.call(Call::AddCustomEndpoint {
        name: name.into(),
        url: url.into(),
        credentials: vec![],
    })
}

#[test]
fn a_jackett_feed_added_by_hand_keeps_its_key_out_of_the_database() {
    let http = FakeHttp::new();
    http.push_for(
        "127.0.0.1:9117",
        200,
        "<?xml version=\"1.0\"?><rss><channel><title>t</title></channel></rss>",
    );
    let (dir, core, _) = common::open_core_with_http(http.clone());
    // Beside a repo, not instead of one.
    trust(&core, &fixture("general.json"));
    let secret = "jackettkey0123456789";
    let c = catalog(add_custom(
        &core,
        "1337x",
        &format!(
            "http://127.0.0.1:9117/api/v2.0/indexers/1337x/results/torznab/api?apikey={secret}&t=search&q={{query}}"
        ),
    ));
    let x = c.installed.iter().find(|i| i.id == "1337x").unwrap();
    assert!(x.custom && x.offered && x.missing.is_empty());
    assert_eq!(x.auth[0].key, "apikey");
    assert_eq!(c.repos.len(), 1);

    let Reply::PhoneSources { sources } = core.call(Call::PhoneSearchSources).unwrap() else {
        panic!()
    };
    let shown = sources.iter().find(|s| s.name == "1337x").unwrap();
    assert!(shown.url.contains("apikey={apikey}"), "{}", shown.url);

    let _ = core.call(Call::PhoneSearch {
        query: "Frieren".into(),
        endpoint: Some("1337x".into()),
        fallback: false,
    });
    let sent: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
    assert!(
        sent.iter()
            .any(|u| u.contains(&format!("apikey={secret}")) && u.contains("q=Frieren")),
        "{sent:?}"
    );

    core.shutdown();
    for f in std::fs::read_dir(dir.path().join("data")).unwrap() {
        let path = f.unwrap().path();
        if path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("anibeam.db")
        {
            let bytes = std::fs::read(&path).unwrap();
            assert!(!bytes.windows(secret.len()).any(|w| w == secret.as_bytes()));
        }
    }
}

#[test]
fn an_endpoint_by_hand_needs_a_search_and_a_free_name() {
    let (_dir, core, _) = common::open_core();
    let e = add_custom(&core, "plain", "https://a.example/feed").unwrap_err();
    assert!(matches!(e, CoreError::Invalid { .. }), "{e:?}");
    add_custom(&core, "Mine", "https://a.example/rss?q={query}").unwrap();
    let e = add_custom(&core, "mine", "https://b.example/rss?q={query}").unwrap_err();
    assert!(matches!(e, CoreError::Invalid { .. }), "{e:?}");
    // A placeholder of its own may be filled afterwards; until then it is not searched.
    let c = catalog(add_custom(
        &core,
        "other",
        "https://c.example/rss?q={query}&u={user}",
    ));
    let other = c.installed.iter().find(|i| i.id == "other").unwrap();
    assert_eq!(other.missing, ["user"]);
    let Reply::PhoneSources { sources } = core.call(Call::PhoneSearchSources).unwrap() else {
        panic!()
    };
    assert!(sources.iter().all(|s| s.name != "other"));
    let c = catalog(core.call(Call::SetEndpointCredentials {
        install: other.install_id.clone(),
        credentials: vec![EndpointCredential {
            key: "user".into(),
            value: "me".into(),
        }],
    }));
    assert!(c.installed.iter().all(|i| i.missing.is_empty()));
}

#[test]
fn a_bare_nyaa_address_by_hand_is_the_nyaa_kind() {
    let (_dir, core, _) = common::open_core();
    let c = catalog(add_custom(&core, "nyaa-mirror", "https://nyaa.land/"));
    assert_eq!(c.installed[0].kind, "nyaa");
}

#[test]
fn a_repo_endpoint_cannot_take_the_name_of_one_added_by_hand() {
    let (_dir, core, _) = common::open_core();
    trust(&core, &fixture("general.json"));
    add_custom(&core, "nyaa", "https://nyaa.land/").unwrap();
    let e = core
        .call(Call::InstallEndpoint {
            repo: fixture("general.json"),
            id: "nyaa".into(),
            credentials: vec![],
        })
        .unwrap_err();
    assert!(e.to_string().contains("added by you"), "{e}");
}

#[test]
fn a_repo_removed_during_a_refresh_stays_removed() {
    let (dir, core, _) = common::open_core();
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    write_repo(&path, &key(1), 1, vec![]);
    trust(&core, &url);
    write_repo(&path, &key(1), 2, vec![]);
    catalog(core.call(Call::RemoveEndpointRepo { url: url.clone() }));
    let c = catalog(core.call(Call::RefreshEndpointRepos));
    assert!(c.repos.is_empty());
}

#[test]
fn the_phone_ships_with_no_source() {
    let (_dir, core, _) = common::open_core();
    let Reply::PhoneSources { sources } = core.call(Call::PhoneSources).unwrap() else {
        panic!()
    };
    assert!(sources.is_empty());
}

// Torznab through the person's own Prowlarr (#148) -------------------------------------------

#[test]
fn a_torznab_endpoint_searches_the_host_it_was_given() {
    let http = FakeHttp::new();
    http.push_for(
        "10.0.0.2:9696",
        200,
        r#"<?xml version="1.0"?><rss version="2.0" xmlns:torznab="http://torznab.com/schemas/2015/feed"><channel><title>Example</title>
<item><title>[Group] Show - 01 (1080p)</title><link>magnet:?xt=urn:btih:aaa</link><torznab:attr name="seeders" value="42"/></item>
<item><title>[Other] Show - 02 [1080p]</title><link>magnet:?xt=urn:btih:bbb</link><torznab:attr name="seeders" value="7"/></item>
</channel></rss>"#,
    );
    let (dir, core, _) = common::open_core_with_http(http.clone());
    let path = dir.path().join("repo.json");
    let url = path.to_string_lossy().into_owned();
    write_repo(
        &path,
        &key(1),
        1,
        vec![json!({
            "id": "indexer", "name": "Indexer", "version": 1, "kind": "torznab",
            "url": "http://{host}/1/api?t=search&q={query}&apikey={apikey}",
            "auth": [
                {"key": "host", "label": "Prowlarr address", "secret": false},
                {"key": "apikey", "label": "API key", "secret": true},
            ],
        })],
    );
    trust(&core, &url);
    let credentials = |host: &str| {
        vec![
            EndpointCredential {
                key: "host".into(),
                value: host.into(),
            },
            EndpointCredential {
                key: "apikey".into(),
                value: "k3y".into(),
            },
        ]
    };
    // An address with a path in it is no host.
    let e = core
        .call(Call::InstallEndpoint {
            repo: url.clone(),
            id: "indexer".into(),
            credentials: credentials("evil.example/x"),
        })
        .unwrap_err();
    assert!(matches!(e, CoreError::Invalid { .. }), "{e:?}");

    catalog(core.call(Call::InstallEndpoint {
        repo: url,
        id: "indexer".into(),
        credentials: credentials(" 10.0.0.2:9696 "),
    }));
    let Reply::PhoneSearch { search } = core
        .call(Call::PhoneSearch {
            query: "Show".into(),
            endpoint: Some("indexer".into()),
            fallback: false,
        })
        .unwrap()
    else {
        panic!()
    };
    let sent: Vec<String> = http
        .requests()
        .into_iter()
        .map(|r| r.url)
        .filter(|u| u.contains("/1/api"))
        .collect();
    assert_eq!(
        sent,
        ["http://10.0.0.2:9696/1/api?t=search&q=Show&apikey=k3y"]
    );
    let mut seen: Vec<(String, Option<i64>)> = search
        .items
        .iter()
        .map(|i| (i.title.clone(), i.seeders))
        .collect();
    seen.sort();
    assert_eq!(
        seen,
        [
            ("[Group] Show - 01 (1080p)".to_string(), Some(42)),
            ("[Other] Show - 02 [1080p]".to_string(), Some(7)),
        ]
    );
}
