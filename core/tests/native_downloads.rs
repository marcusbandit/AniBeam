//! Subscribing, adding torrents and removing subscriptions on the core's
//! own engine (#119), against `FakeHttp`: every rule, feed and torrent goes
//! to qBittorrent directly, in anirss's own shape, and anirss is never
//! asked. anirss is set up as a remote server, so a request to it would
//! show in `FakeHttp`.
#![cfg(unix)]

mod common;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anibeam_core::events::Collector;
use anibeam_core::net::{Body, FakeHttp, HttpRequest};
use anibeam_core::*;
use serde_json::{Value, json};

const QBT: &str = "http://localhost:9090";
const FEED: &str = "https://feed.example/rss?q={query}";
const SAVE_BASE: &str = "/srv/TV";

fn settings(torrent_url: &str) -> DownloadSettings {
    DownloadSettings {
        anirss_mode: AnirssMode::Remote,
        anirss_command: "anirss".to_string(),
        jackett_command: String::new(),
        anirss_url: "http://anirss.test".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_mode: TorrentMode::Existing,
        torrent_url: torrent_url.to_string(),
        torrent_username: "me".to_string(),
        has_torrent_password: false,
        saved_clients: Vec::new(),
        active_client: String::new(),
        prune_finished_feeds: true,
        rolling: Default::default(),
        engine: DownloadsEngine::Native,
    }
}

/// Points the XDG homes at `dir`, so no case reads the machine's own
/// anirss config or saved password. Process-wide, but every case points
/// them at an empty directory, which reads the same as any other.
fn isolate_xdg(dir: &Path) {
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", dir);
        std::env::set_var("XDG_STATE_HOME", dir);
    }
}

fn started(core: &Core, call: Call) -> u64 {
    match core.call(call).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

struct Rig {
    dir: common::Dir,
    core: Arc<Core>,
    c: Arc<Collector>,
    http: Arc<FakeHttp>,
}

/// The core's engine on one endpoint added by hand, its torrent client at
/// `torrent_url` with `save_base` as its default save folder.
fn rig_at(torrent_url: &str, save_base: &str) -> Rig {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    let reply = core
        .call(Call::SetDownloadSettings {
            settings: settings(torrent_url),
            anirss_token: Some("tok".to_string()),
            torrent_password: Some("pw".to_string()),
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
    core.call(Call::AddCustomEndpoint {
        name: "feed".into(),
        url: FEED.into(),
        credentials: vec![],
    })
    .unwrap();
    http.push_for("app/defaultSavePath", 200, save_base.to_string());
    Rig { dir, core, c, http }
}

fn rig() -> Rig {
    rig_at(QBT, SAVE_BASE)
}

fn release(title: &str, link: &str) -> Release {
    serde_json::from_value(json!({"title": title, "link": link, "source": "feed"})).unwrap()
}

fn episode(n: u32) -> Release {
    release(
        &format!("[Group] Show - {n:02} (1080p)"),
        &format!("magnet:?xt=urn:btih:{n:040x}"),
    )
}

fn form(r: &HttpRequest) -> Vec<(String, String)> {
    match &r.body {
        Some(Body::Form(pairs)) => pairs.clone(),
        other => panic!("not a form: {other:?}"),
    }
}

fn field(r: &HttpRequest, key: &str) -> Option<String> {
    form(r).into_iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// The requests to the torrent client's API, by the path after `api/v2/`.
fn qbt_calls(http: &FakeHttp) -> Vec<(String, HttpRequest)> {
    http.requests()
        .into_iter()
        .filter_map(|r| {
            let path = r.url.strip_prefix(&format!("{QBT}/api/v2/"))?.to_string();
            Some((path, r))
        })
        .collect()
}

fn paths_of(http: &FakeHttp) -> Vec<String> {
    qbt_calls(http).into_iter().map(|(p, _)| p).collect()
}

fn called<'a>(calls: &'a [(String, HttpRequest)], path: &str) -> &'a HttpRequest {
    &calls
        .iter()
        .find(|(p, _)| p == path)
        .unwrap_or_else(|| panic!("no {path} in {calls:?}"))
        .1
}

/// anirss was never asked anything.
fn no_anirss(http: &FakeHttp) {
    let sent: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
    assert!(
        !sent.iter().any(|u| u.contains("anirss.test")),
        "anirss was asked: {sent:?}"
    );
}

fn failed(event: &Event) -> &CoreError {
    match &event.body {
        EventBody::JobFailed { error } => error,
        other => panic!("expected a failure, got {other:?}"),
    }
}

fn subscribe(r: &Rig, query: &str, items: Vec<Release>, name: Option<&str>) -> Event {
    let job = started(
        &r.core,
        Call::Subscribe {
            query: query.to_string(),
            endpoint: "feed".to_string(),
            items,
            name: name.map(str::to_string),
            wire_query: None,
        },
    );
    common::wait_job(&r.c, job)
}

fn subscribed(event: &Event) -> &Subscribed {
    match &event.body {
        EventBody::Subscribed { result } => result,
        other => panic!("{other:?}"),
    }
}

// Subscribing ----------------------------------------------------------------------

#[test]
fn subscribe_writes_the_rule_and_feed_through_qbittorrent() {
    let r = rig();
    r.http.push_for("rss/rules", 200, "{}");
    r.http.push_for("rss/addFeed", 200, "");
    r.http.push_for("rss/setRule", 200, "");

    let done = subscribe(&r, "Show -HEVC", vec![episode(1)], None);
    let feed_url = "https://feed.example/rss?q=Show";
    assert_eq!(
        subscribed(&done),
        &Subscribed {
            name: "Show".to_string(),
            feed_url: feed_url.to_string(),
            save_path: "/srv/TV/Show".to_string(),
            must_contain: String::new(),
            must_not_contain: "HEVC".to_string(),
            tags: vec!["Anime".to_string()],
        }
    );

    let calls = qbt_calls(&r.http);
    let add = called(&calls, "rss/addFeed");
    assert_eq!(
        form(add),
        vec![
            ("url".to_string(), feed_url.to_string()),
            ("path".to_string(), "Show".to_string()),
        ]
    );
    let set = called(&calls, "rss/setRule");
    assert_eq!(field(set, "ruleName").as_deref(), Some("Show"));
    let def: Value = serde_json::from_str(&field(set, "ruleDef").unwrap()).unwrap();
    assert_eq!(
        def,
        json!({
            "enabled": true, "mustContain": "", "mustNotContain": "HEVC",
            "useRegex": false, "episodeFilter": "", "smartFilter": false,
            "previouslyMatchedEpisodes": [], "affectedFeeds": [feed_url],
            "ignoreDays": 0, "lastMatch": "", "addPaused": null, "assignedCategory": "",
            "savePath": "/srv/TV/Show",
            "torrentParams": {"save_path": "/srv/TV/Show", "use_auto_tmm": false, "tags": ["Anime"]}
        })
    );
    // The feed goes in before the rule that reads it, as anirss does it
    let order = paths_of(&r.http);
    let feed_at = order.iter().position(|p| p == "rss/addFeed").unwrap();
    let rule_at = order.iter().position(|p| p == "rss/setRule").unwrap();
    assert!(feed_at < rule_at, "{order:?}");
    no_anirss(&r.http);
}

#[test]
fn subscribing_the_same_show_from_another_endpoint_suffixes_the_name() {
    let r = rig();
    r.http.push_for(
        "rss/rules",
        200,
        json!({"Show": {"affectedFeeds": ["https://other.example/rss?q=Show"]}}).to_string(),
    );
    r.http.push_for("rss/addFeed", 200, "");
    r.http.push_for("rss/setRule", 200, "");

    let done = subscribe(&r, "Show", vec![episode(1)], None);
    let result = subscribed(&done);
    assert_eq!(result.name, "Show @feed");
    assert_eq!(result.save_path, "/srv/TV/Show @feed");
    let calls = qbt_calls(&r.http);
    assert_eq!(
        field(called(&calls, "rss/addFeed"), "path").as_deref(),
        Some("Show @feed")
    );
    assert_eq!(
        field(called(&calls, "rss/setRule"), "ruleName").as_deref(),
        Some("Show @feed")
    );
    no_anirss(&r.http);
}

#[test]
fn subscribing_the_same_feed_again_keeps_the_name() {
    let r = rig();
    r.http.push_for(
        "rss/rules",
        200,
        json!({"Show": {"affectedFeeds": ["https://feed.example/rss?q=Show"]}}).to_string(),
    );
    // qBittorrent already has the feed: no failure
    r.http.push_for("rss/addFeed", 409, "Conflict");
    r.http.push_for("rss/setRule", 200, "");

    let done = subscribe(&r, "Show", vec![episode(1)], None);
    assert_eq!(subscribed(&done).name, "Show");
    let calls = qbt_calls(&r.http);
    assert_eq!(
        field(called(&calls, "rss/setRule"), "ruleName").as_deref(),
        Some("Show")
    );
}

#[test]
fn subscribe_name_defaults_to_the_show_name_and_refuses_a_path() {
    let r = rig();
    let done = subscribe(&r, "Show", vec![episode(1)], Some("TV/Show"));
    assert!(
        matches!(failed(&done), CoreError::Invalid { field, .. } if field == "name"),
        "{done:?}"
    );
    assert!(
        paths_of(&r.http).is_empty(),
        "a bad name writes nothing: {:?}",
        paths_of(&r.http)
    );

    r.http.push_for("rss/rules", 200, "{}");
    r.http.push_for("rss/addFeed", 200, "");
    r.http.push_for("rss/setRule", 200, "");
    let done = subscribe(&r, "Show", vec![episode(1)], Some("  My Show  "));
    assert_eq!(subscribed(&done).name, "My Show");
    no_anirss(&r.http);
}

/// The port of the anirss engine's narrowed subscription: a generic RSS
/// endpoint cannot answer the refined query, so the rule is pointed at the
/// feed the search read and keeps the narrowing words. The core's own
/// endpoints decide that, not the list a connect to anirss would have read,
/// which here was never read at all.
#[test]
fn a_locally_narrowed_subscription_still_narrows_on_native() {
    let r = rig();
    let narrow = "https://feed.example/rss?q=Show+1080p";
    let wide = "https://feed.example/rss?q=Show";
    let feed = "<?xml version=\"1.0\"?><rss version=\"2.0\"><channel><title>f</title>\
                <item><title>[Group] Show - 01 (1080p)</title>\
                <link>magnet:?xt=urn:btih:0000000000000000000000000000000000000001</link></item>\
                </channel></rss>";
    for _ in 0..3 {
        r.http.push_for("feed.example", 200, feed);
    }
    r.http.push_for("rss/rules", 200, "{}");
    r.http.push_for("rss/addFeed", 200, "");
    r.http.push_for("rss/setRule", 200, "");
    r.http.push_for(
        "rss/rules",
        200,
        json!({"Show": {"enabled": true, "mustContain": "", "mustNotContain": "",
                        "affectedFeeds": [narrow], "savePath": "/srv/TV/Show"}})
        .to_string(),
    );
    r.http.push_for("rss/setRule", 200, "");
    r.http.push_for(
        "rss/items",
        200,
        json!({"Show": {"uid": "u", "url": narrow}}).to_string(),
    );
    r.http.push_for("rss/removeItem", 200, "");
    r.http.push_for("rss/addFeed", 200, "");

    let job = started(
        &r.core,
        Call::Subscribe {
            query: "Show 1080p".to_string(),
            endpoint: "feed".to_string(),
            items: vec![episode(1)],
            name: None,
            wire_query: Some("Show".to_string()),
        },
    );
    let done = common::wait_job(&r.c, job);
    let result = subscribed(&done);
    assert_eq!(result.feed_url, wide);
    assert_eq!(result.must_contain, "1080p");

    let rss: Vec<(String, HttpRequest)> = qbt_calls(&r.http)
        .into_iter()
        .filter(|(p, _)| p.starts_with("rss/"))
        .collect();
    let order: Vec<&str> = rss.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(
        order,
        vec![
            "rss/rules",
            "rss/addFeed",
            "rss/setRule",
            "rss/rules",
            "rss/setRule",
            "rss/items",
            "rss/removeItem",
            "rss/addFeed"
        ]
    );
    let def: Value = serde_json::from_str(&field(&rss[4].1, "ruleDef").unwrap()).unwrap();
    assert_eq!(def["mustContain"], "1080p");
    assert_eq!(def["affectedFeeds"], json!([wide]));
    assert_eq!(
        form(&rss[7].1),
        vec![
            ("url".to_string(), wide.to_string()),
            ("path".to_string(), "Show".to_string()),
        ]
    );
    no_anirss(&r.http);
}

// Listing ----------------------------------------------------------------------------

#[test]
fn list_rules_lists_every_rule_with_counts() {
    let r = rig();
    let alpha = r.dir.path().join("dl").join("alpha");
    std::fs::create_dir_all(alpha.join("S1")).unwrap();
    std::fs::write(alpha.join("01.mkv"), b"x").unwrap();
    std::fs::write(alpha.join("S1").join("02.mkv"), b"x").unwrap();
    let alpha = alpha.to_string_lossy().into_owned();
    // A rule anirss or a person wrote is listed as well as one of AniBeam's
    r.http.push_for(
        "rss/rules",
        200,
        json!({
            "Beta": {"enabled": false, "affectedFeeds": ["https://other.example/feed?q=Beta%20Show"],
                     "savePath": "/srv/TV/Beta"},
            "alpha": {"affectedFeeds": ["https://feed.example/rss?q=Alpha"], "savePath": alpha},
        })
        .to_string(),
    );
    r.http.push_for(
        "torrents/info",
        200,
        json!([
            {"hash": "a1", "name": "one", "save_path": format!("{alpha}/")},
            {"hash": "a2", "name": "two", "save_path": alpha},
            {"hash": "b1", "name": "three", "save_path": "/srv/TV/Beta"},
            {"hash": "c1", "name": "four", "save_path": "/srv/TV/Other"},
        ])
        .to_string(),
    );
    assert_eq!(r.core.call(Call::ListSubscriptionRules).unwrap(), Reply::Ok);
    let read = common::wait_for(
        &r.c,
        |e| matches!(e.body, EventBody::SubscriptionsRead { .. }),
        Duration::from_secs(10),
    );
    let EventBody::SubscriptionsRead { rules } = read.body else {
        unreachable!()
    };
    let names: Vec<&str> = rules.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, vec!["alpha", "Beta"], "by name, case ignored");
    assert_eq!(rules[0].feed_url, "https://feed.example/rss?q=Alpha");
    assert_eq!(rules[0].save_path, alpha);
    assert!(rules[0].rule_enabled);
    assert_eq!(rules[0].torrent_count, 2);
    assert_eq!(rules[0].file_count, 2);
    assert_eq!(rules[0].query, "Alpha");
    assert!(!rules[1].rule_enabled);
    assert_eq!(rules[1].torrent_count, 1);
    assert_eq!(rules[1].file_count, 0, "a folder not on this disk");
    assert_eq!(rules[1].query, "Beta Show");
    no_anirss(&r.http);
}

// Removing --------------------------------------------------------------------------

/// One rule, its feed and one torrent saving into its folder.
fn queue_one_rule(http: &FakeHttp, save_path: &str) {
    http.push_for(
        "rss/rules",
        200,
        json!({"Show": {"affectedFeeds": ["https://feed.example/rss?q=Show"], "savePath": save_path}})
            .to_string(),
    );
    http.push_for(
        "rss/items",
        200,
        json!({"TV": {"Show": {"uid": "u", "url": "https://feed.example/rss?q=Show"}}}).to_string(),
    );
    http.push_for(
        "torrents/info",
        200,
        json!([
            {"hash": "aa", "name": "[Group] Show - 01", "save_path": save_path},
            {"hash": "zz", "name": "Other", "save_path": "/srv/TV/Other"},
        ])
        .to_string(),
    );
}

const WRITES: [&str; 4] = [
    "torrents/delete",
    "rss/removeItem",
    "rss/removeRule",
    "torrents/add",
];

fn writes(http: &FakeHttp) -> Vec<String> {
    paths_of(http)
        .into_iter()
        .filter(|p| WRITES.contains(&p.as_str()))
        .collect()
}

fn remove_call(r: &Rig, dry_run: bool, torrents: bool, files: bool) -> Event {
    let call = if dry_run {
        Call::PlanRemoveSubscription {
            name: "Show".to_string(),
            torrents,
            files,
        }
    } else {
        Call::RemoveSubscription {
            name: "Show".to_string(),
            torrents,
            files,
        }
    };
    let job = started(&r.core, call);
    common::wait_job(&r.c, job)
}

#[test]
fn remove_plan_matches_remove() {
    let r = rig();
    queue_one_rule(&r.http, "/srv/TV/Show");
    let planned = remove_call(&r, true, true, false);
    let EventBody::RemovalPlanned { plans } = &planned.body else {
        panic!("{:?}", planned.body)
    };
    assert_eq!(
        plans,
        &vec![RemovePlan {
            name: "Show".to_string(),
            feed_url: "https://feed.example/rss?q=Show".to_string(),
            save_path: "/srv/TV/Show".to_string(),
            torrent_count_in_qb: 1,
            size_bytes: 0,
            torrents: vec!["[Group] Show - 01".to_string()],
        }]
    );
    assert!(writes(&r.http).is_empty(), "a plan changes nothing");

    queue_one_rule(&r.http, "/srv/TV/Show");
    r.http.push_for("app/defaultSavePath", 200, SAVE_BASE);
    r.http.push_for("torrents/delete", 200, "");
    r.http.push_for("rss/removeItem", 200, "");
    r.http.push_for("rss/removeRule", 200, "");
    let done = remove_call(&r, false, true, false);
    assert!(
        matches!(&done.body, EventBody::SubscriptionRemoved { name } if name == "Show"),
        "{:?}",
        done.body
    );
    assert_eq!(
        writes(&r.http),
        vec!["torrents/delete", "rss/removeItem", "rss/removeRule"]
    );
    let calls = qbt_calls(&r.http);
    let delete = called(&calls, "torrents/delete");
    assert_eq!(field(delete, "hashes").as_deref(), Some("aa"));
    assert_eq!(field(delete, "deleteFiles").as_deref(), Some("false"));
    assert_eq!(
        field(called(&calls, "rss/removeItem"), "path").as_deref(),
        Some("TV\\Show")
    );
    assert_eq!(
        field(called(&calls, "rss/removeRule"), "ruleName").as_deref(),
        Some("Show")
    );
    no_anirss(&r.http);
}

#[test]
fn remove_unknown_name_changes_nothing() {
    let r = rig();
    r.http.push_for("rss/rules", 200, "{}");
    let done = remove_call(&r, false, true, true);
    assert!(
        matches!(failed(&done), CoreError::Service { code, message, .. }
            if code == "not_found" && message.contains("Show")),
        "{done:?}"
    );
    assert!(writes(&r.http).is_empty());
    no_anirss(&r.http);
}

#[test]
fn remove_with_files_refuses_outside_the_save_base() {
    let r = rig();
    queue_one_rule(&r.http, "/elsewhere/Show");
    let done = remove_call(&r, true, true, true);
    assert!(
        matches!(failed(&done), CoreError::Invalid { field, .. } if field == "files"),
        "a dry run refuses too: {done:?}"
    );
    queue_one_rule(&r.http, "/elsewhere/Show");
    r.http.push_for("app/defaultSavePath", 200, SAVE_BASE);
    let done = remove_call(&r, false, true, true);
    assert!(
        matches!(failed(&done), CoreError::Invalid { field, .. } if field == "files"),
        "{done:?}"
    );
    assert!(writes(&r.http).is_empty());
}

/// On this machine the torrents go without their files, and the folder
/// goes to the trash: nothing is deleted for good.
#[test]
fn remove_with_files_trashes_and_never_deletes() {
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("XDG_DATA_HOME", dir.path().join("share")) };
    let base = dir.path().join("dl");
    let show = base.join("Show");
    std::fs::create_dir_all(&show).unwrap();
    std::fs::write(show.join("01.mkv"), b"abcd").unwrap();
    let base = base.to_string_lossy().into_owned();
    let show_path = show.to_string_lossy().into_owned();
    let r = rig_at(QBT, &base);

    queue_one_rule(&r.http, &show_path);
    let planned = remove_call(&r, true, true, true);
    let EventBody::RemovalPlanned { plans } = &planned.body else {
        panic!("{:?}", planned.body)
    };
    assert_eq!(plans[0].size_bytes, 4);
    assert!(show.exists(), "a plan takes nothing");

    queue_one_rule(&r.http, &show_path);
    r.http.push_for("app/defaultSavePath", 200, base.clone());
    r.http.push_for("torrents/delete", 200, "");
    r.http.push_for("rss/removeItem", 200, "");
    r.http.push_for("rss/removeRule", 200, "");
    let done = remove_call(&r, false, true, true);
    assert!(
        matches!(&done.body, EventBody::SubscriptionRemoved { .. }),
        "{:?}",
        done.body
    );
    let calls = qbt_calls(&r.http);
    assert_eq!(
        field(called(&calls, "torrents/delete"), "deleteFiles").as_deref(),
        Some("false"),
        "never the client's own delete with files on this machine"
    );
    assert!(!show.exists(), "the folder went to the trash");
    no_anirss(&r.http);
}

/// A client on another machine keeps its files out of the trash's reach:
/// its own delete takes them, and files without the torrents are refused.
#[test]
fn a_remote_client_deletes_the_files_with_its_torrents() {
    let remote = "http://192.0.2.1:9090";
    let r = rig_at(remote, SAVE_BASE);
    let on_remote = |http: &FakeHttp, path: &str| {
        http.requests()
            .into_iter()
            .find(|q| q.url == format!("{remote}/api/v2/{path}"))
    };
    let done = remove_call(&r, false, false, true);
    assert!(
        matches!(failed(&done), CoreError::Invalid { field, .. } if field == "files"),
        "{done:?}"
    );

    queue_one_rule(&r.http, "/srv/TV/Show");
    r.http.push_for("torrents/delete", 200, "");
    r.http.push_for("rss/removeItem", 200, "");
    r.http.push_for("rss/removeRule", 200, "");
    let done = remove_call(&r, false, true, true);
    assert!(
        matches!(&done.body, EventBody::SubscriptionRemoved { .. }),
        "{:?}",
        done.body
    );
    let delete = on_remote(&r.http, "torrents/delete").expect("torrents deleted");
    assert_eq!(field(&delete, "deleteFiles").as_deref(), Some("true"));
    no_anirss(&r.http);
}

// Adding torrents --------------------------------------------------------------------

fn added(event: &Event) -> &AddedTorrents {
    match &event.body {
        EventBody::TorrentsAdded { result } => result,
        other => panic!("{other:?}"),
    }
}

#[test]
fn download_adds_magnets_and_urls_with_savepath_and_tags() {
    let r = rig();
    r.http.push_for("torrents/add", 200, "Ok.");
    let items = vec![
        episode(1),
        release(
            "[Group] Show - 02 (1080p)",
            "https://dl.example/[Group] Show - 02.torrent",
        ),
    ];
    let job = started(&r.core, Call::DownloadReleases { items, name: None });
    let done = common::wait_job(&r.c, job);
    assert_eq!(
        added(&done),
        &AddedTorrents {
            count: 2,
            save_path: "/srv/TV/Show".to_string(),
            tags: vec!["Anime".to_string()],
            // A magnet's hash is known up front; a link qBittorrent fetches is not
            hashes: vec![format!("{:040x}", 1)],
        }
    );
    let calls = qbt_calls(&r.http);
    let add = called(&calls, "torrents/add");
    assert_eq!(
        form(add),
        vec![
            (
                "urls".to_string(),
                format!(
                    "magnet:?xt=urn:btih:{:040x}\nhttps://dl.example/%5BGroup%5D%20Show%20-%2002.torrent",
                    1
                )
            ),
            ("savepath".to_string(), "/srv/TV/Show".to_string()),
            ("tags".to_string(), "Anime".to_string()),
        ]
    );
    no_anirss(&r.http);
}

#[test]
fn a_failed_add_body_is_an_error() {
    let r = rig();
    r.http.push_for(
        "torrents/add",
        200,
        json!({"success_count": 0, "failure_count": 1}).to_string(),
    );
    let job = started(
        &r.core,
        Call::DownloadReleases {
            items: vec![episode(1)],
            name: Some("Show".to_string()),
        },
    );
    let done = common::wait_job(&r.c, job);
    assert!(
        matches!(failed(&done), CoreError::Service { code, .. } if code == "add_failed"),
        "{done:?}"
    );
    no_anirss(&r.http);
}

#[test]
fn movie_adds_into_the_movie_path() {
    let r = rig();
    r.http.push_for("torrents/add", 200, "Ok.");
    let link = format!("magnet:?xt=urn:btih:{:040x}", 7);
    let job = started(&r.core, Call::DownloadMovie { link: link.clone() });
    let done = common::wait_job(&r.c, job);
    let result = added(&done);
    // No films' folder set anywhere: the default save folder, no subfolder
    assert_eq!(result.save_path, SAVE_BASE);
    assert_eq!(result.hashes, vec![format!("{:040x}", 7)]);
    let calls = qbt_calls(&r.http);
    let add = called(&calls, "torrents/add");
    assert_eq!(field(add, "urls"), Some(link));
    assert_eq!(field(add, "savepath").as_deref(), Some(SAVE_BASE));
    no_anirss(&r.http);
}

/// A rolling pick of forty single episodes adds the first twelve now, and
/// the add's hashes come back one per release, so the roll can follow
/// each part in qBittorrent.
#[test]
fn download_with_rolling_works_on_native() {
    let r = rig();
    r.http.push_for("torrents/add", 200, "Ok.");
    let items: Vec<Release> = (1..=40)
        .rev()
        .map(|n| {
            release(
                &format!("[Group] Show - {n:03} [1080p].mkv"),
                &format!("magnet:?xt=urn:btih:{n:040x}"),
            )
        })
        .collect();
    let job = started(
        &r.core,
        Call::DownloadReleasesWith {
            items,
            name: Some("Show".to_string()),
            options: DownloadOptions {
                rolling: Some(true),
                ..DownloadOptions::default()
            },
        },
    );
    let done = common::wait_job(&r.c, job);
    let result = added(&done);
    let hashes: Vec<String> = (1..=12).map(|n| format!("{n:040x}")).collect();
    assert_eq!(result.count, 12);
    assert_eq!(result.hashes, hashes);
    let calls = qbt_calls(&r.http);
    let urls = field(called(&calls, "torrents/add"), "urls").unwrap();
    assert_eq!(urls.lines().count(), 12);
    no_anirss(&r.http);
}

/// A small but whole .torrent file.
const TORRENT: &[u8] = b"d8:announce15:http://x.test/a4:infod6:lengthi1e4:name5:a.mkvee";

fn torrent_hash() -> String {
    anibeam_core::releases::stats::info_hash(TORRENT)
        .unwrap()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A web link's .torrent is fetched here and uploaded, so its hash is
/// known and qBittorrent never fetches it itself.
#[test]
fn a_torrent_url_is_fetched_and_uploaded_with_its_hash() {
    let r = rig();
    r.http
        .push_for("dl.example/a.torrent", 200, TORRENT.to_vec());
    r.http.push_for("torrents/info", 200, "[]");
    r.http.push_for("torrents/add", 200, "Ok.");
    let job = started(
        &r.core,
        Call::DownloadReleases {
            items: vec![release(
                "[Group] Show - 01 (1080p)",
                "https://dl.example/a.torrent",
            )],
            name: Some("Show".to_string()),
        },
    );
    let done = common::wait_job(&r.c, job);
    assert_eq!(added(&done).hashes, vec![torrent_hash()]);
    let calls = qbt_calls(&r.http);
    assert!(
        calls
            .iter()
            .any(|(p, _)| *p == format!("torrents/info?hashes={}", torrent_hash())),
        "{calls:?}"
    );
    let add = called(&calls, "torrents/add");
    let Some(Body::Multipart { fields, files }) = &add.body else {
        panic!("not an upload: {:?}", add.body)
    };
    assert_eq!(
        fields,
        &vec![
            ("savepath".to_string(), "/srv/TV/Show".to_string()),
            ("tags".to_string(), "Anime".to_string()),
        ]
    );
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].field, "torrents");
    assert_eq!(files[0].file_name, "a.torrent");
    assert_eq!(files[0].bytes, TORRENT);
    assert_eq!(
        calls.iter().filter(|(p, _)| p == "torrents/add").count(),
        1,
        "no second add by URL"
    );
    no_anirss(&r.http);
}

#[test]
fn a_duplicate_upload_is_skipped() {
    let r = rig();
    r.http
        .push_for("dl.example/a.torrent", 200, TORRENT.to_vec());
    r.http.push_for(
        "torrents/info",
        200,
        json!([{"hash": torrent_hash(), "name": "a.mkv"}]).to_string(),
    );
    let job = started(
        &r.core,
        Call::DownloadReleases {
            items: vec![release(
                "[Group] Show - 01 (1080p)",
                "https://dl.example/a.torrent",
            )],
            name: Some("Show".to_string()),
        },
    );
    let done = common::wait_job(&r.c, job);
    let result = added(&done);
    assert_eq!(result.count, 1);
    assert_eq!(result.hashes, vec![torrent_hash()]);
    assert!(
        !paths_of(&r.http).iter().any(|p| p == "torrents/add"),
        "qBittorrent already has it: {:?}",
        paths_of(&r.http)
    );
    no_anirss(&r.http);
}
