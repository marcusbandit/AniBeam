//! The one-time rename of #602: an installed endpoint saved under a kind's old site name gets
//! the kind's shape name the next time the core opens, and nothing else about it changes.
//! The rows are shaped the way the core before #602 stored real installs: the owner's
//! definitions as `serde_json` wrote them, with their install ids, repo, versions, order and
//! switches, behind example hosts.

mod common;

use std::path::Path;

use anibeam_core::releases::kinds::{RENAMED, shape_of};
use anibeam_core::trackers::Secrets;
use anibeam_core::{Call, Core, CorePaths, Reply};
use serde_json::{Value, json};

const REPO: &str = "https://endpoints.example/index/general.json";

/// What the core before #602 kept per installed endpoint, field for field in the order
/// `Definition` serialises.
fn rows() -> Vec<(&'static str, &'static str, i64, Value, bool, i64)> {
    let (filtered, feed) = (RENAMED[0].0, RENAMED[1].0);
    vec![
        (
            "4f6c0a6e-2b1d-4c3a-9f0e-7d2b8a1c5e01",
            REPO,
            1,
            json!({"id": "tracker", "name": "Tracker", "version": 1, "lang": "all",
                   "kind": filtered, "url": "https://tracker.example/", "auth": [],
                   "front": "front.example", "rate": {"per_minute": 30}, "nsfw": false,
                   "category": "1_0", "filter": "0", "min_core": "2.1.0", "json": null}),
            true,
            1,
        ),
        (
            "8a2e6d14-93b7-4e0f-a1c2-5b6d7e8f9a02",
            REPO,
            1,
            json!({"id": "feed", "name": "Feed", "version": 1, "lang": "all",
                   "kind": feed, "url": "https://feed.example/json?q={query}", "auth": [],
                   "front": null, "rate": {"per_minute": 30}, "nsfw": false,
                   "category": null, "filter": null, "min_core": "2.1.0", "json": null}),
            false,
            2,
        ),
        (
            "c3d4e5f6-0718-4293-a4b5-c6d7e8f90a03",
            REPO,
            1,
            json!({"id": "keyed", "name": "Keyed", "version": 1, "lang": "en",
                   "kind": "rss",
                   "url": "https://keyed.example/feed/?res=1080p&s={query}&token={token}",
                   "auth": [{"key": "token", "label": "RSS token", "secret": true,
                             "help": "Copy the token= value from any feed link.",
                             "help_url": "https://keyed.example/rss-page/"}],
                   "front": null, "rate": {"per_minute": 20}, "nsfw": false,
                   "category": null, "filter": null, "min_core": "2.1.0", "json": null}),
            true,
            3,
        ),
        (
            "d4e5f607-1829-43a4-b5c6-d7e8f90a1b04",
            "https://endpoints.example/index/nsfw.json",
            1,
            json!({"id": "adult", "name": "Adult", "version": 1, "lang": "all",
                   "kind": filtered, "url": "https://adult.tracker.example/", "auth": [],
                   "front": null, "rate": {"per_minute": 30}, "nsfw": true,
                   "category": "0_0", "filter": "0", "min_core": "2.1.0", "json": null}),
            false,
            4,
        ),
        (
            "e5f60718-2a3b-44b5-c6d7-e8f90a1b2c05",
            "custom",
            1,
            json!({"id": "mine", "name": "Mine", "version": 1, "lang": "",
                   "kind": "rss", "url": "https://mine.example/rss?q={query}", "auth": [],
                   "front": null, "rate": null, "nsfw": false,
                   "category": null, "filter": null, "min_core": null, "json": null}),
            true,
            5,
        ),
    ]
}

/// The phone's list from before endpoint repos, as `SetPhoneSources` wrote it.
fn phone_sources() -> Value {
    let (filtered, feed) = (RENAMED[0].0, RENAMED[1].0);
    json!([
        {"name": "tracker", "kind": filtered, "url": "https://tracker.example/", "enabled": true},
        {"name": "feed", "kind": feed, "url": "https://feed.example/json?q={query}", "enabled": false},
        {"name": "keyed", "kind": "rss", "url": "https://keyed.example/feed/?s={query}", "enabled": true},
    ])
}

fn open(root: &Path) -> std::sync::Arc<Core> {
    let paths = CorePaths::under(root);
    let secrets = Secrets::file_only(paths.secrets_path());
    Core::open_with_secrets(paths, secrets).unwrap()
}

/// Every installed row as the store holds it: (install id, repo, id, version, definition,
/// enabled, position).
fn stored(root: &Path) -> Vec<(String, String, String, i64, Value, bool, i64)> {
    let conn = rusqlite::Connection::open(CorePaths::under(root).db_path()).unwrap();
    let mut st = conn
        .prepare(
            "SELECT install_id, repo_url, endpoint_id, version, definition, enabled, position
             FROM installed_endpoints ORDER BY position",
        )
        .unwrap();
    st.query_map([], |r| {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get(2)?,
            r.get(3)?,
            serde_json::from_str::<Value>(&r.get::<_, String>(4)?).unwrap(),
            r.get(5)?,
            r.get(6)?,
        ))
    })
    .unwrap()
    .collect::<Result<_, _>>()
    .unwrap()
}

/// A data directory as the core before #602 left it.
fn seed(root: &Path) {
    // Opening once creates the store, as the old core's first run did.
    drop(open(root));
    let conn = rusqlite::Connection::open(CorePaths::under(root).db_path()).unwrap();
    for (install, repo, version, def, enabled, position) in rows() {
        conn.execute(
            "INSERT INTO installed_endpoints (install_id, repo_url, endpoint_id, version, definition, enabled, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                install,
                repo,
                def["id"].as_str().unwrap(),
                version,
                serde_json::to_string(&def).unwrap(),
                enabled,
                position
            ],
        )
        .unwrap();
    }
    let config = Path::new(&CorePaths::under(root).config_dir).to_path_buf();
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(
        config.join("phone-sources.json"),
        serde_json::to_vec_pretty(&phone_sources()).unwrap(),
    )
    .unwrap();
}

#[test]
fn installed_endpoints_get_their_shape_kind_once_and_nothing_else_changes() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path());
    let before = stored(dir.path());

    let core = open(dir.path());
    let after = stored(dir.path());
    assert_eq!(after.len(), before.len());
    for (old, new) in before.iter().zip(&after) {
        let mut want = old.clone();
        let kind = shape_of(want.4["kind"].as_str().unwrap()).to_string();
        want.4["kind"] = kind.into();
        assert_eq!(new, &want, "only the kind changes for {}", old.2);
    }
    let kinds: Vec<&str> = after
        .iter()
        .map(|r| r.4["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        ["filtered-rss", "json-feed", "rss", "filtered-rss", "rss"]
    );

    // The catalog lists them under the new names, in their order, switches as they were.
    let Reply::EndpointCatalog { catalog } = core.call(Call::ListRepoEndpoints).unwrap() else {
        panic!("not a catalog");
    };
    let listed: Vec<(&str, &str, bool)> = catalog
        .installed
        .iter()
        .map(|i| (i.id.as_str(), i.kind.as_str(), i.enabled))
        .collect();
    assert_eq!(
        listed,
        [
            ("tracker", "filtered-rss", true),
            ("feed", "json-feed", false),
            ("keyed", "rss", true),
            ("adult", "filtered-rss", false),
            ("mine", "rss", true),
        ]
    );

    // The phone's saved list too, each row otherwise as it was.
    let saved: Value = serde_json::from_slice(
        &std::fs::read(
            Path::new(&CorePaths::under(dir.path()).config_dir).join("phone-sources.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut want = phone_sources();
    for s in want.as_array_mut().unwrap() {
        let kind = shape_of(s["kind"].as_str().unwrap()).to_string();
        s["kind"] = kind.into();
    }
    assert_eq!(saved, want);
    drop(core);

    // A second open finds nothing to rename and leaves every row as the first left it.
    let core = open(dir.path());
    assert_eq!(stored(dir.path()), after);
    drop(core);
}

#[test]
fn a_store_with_nothing_old_is_left_alone() {
    let (dir, core, _) = common::open_core();
    drop(core);
    let before = stored(dir.path());
    let core = open(dir.path());
    assert_eq!(stored(dir.path()), before);
    assert!(
        !Path::new(&CorePaths::under(dir.path()).config_dir)
            .join("phone-sources.json")
            .exists(),
        "no list is written where there was none"
    );
    drop(core);
}
