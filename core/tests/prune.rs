//! Unsubscribing from finished shows, end to end against `FakeHttp`: a
//! subscription read prunes the rule of a show that finished 16 days ago
//! through the same anirss call a user's removal makes, once a day, and
//! never while the setting is off, the torrent client is down or the shell
//! turned it off.

mod common;
mod fixtures;

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anibeam_core::events::Collector;
use anibeam_core::net::{Body, FakeHttp, HttpRequest};
use anibeam_core::*;
use serde_json::json;

const DAY: i64 = 86_400;

fn isolate_xdg(dir: &Path) {
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", dir);
        std::env::set_var("XDG_STATE_HOME", dir);
    }
}

fn remote(prune: bool) -> DownloadSettings {
    DownloadSettings {
        anirss_mode: AnirssMode::Remote,
        anirss_command: "anirss".to_string(),
        anirss_url: "http://anirss.test".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_url: "http://localhost:9090".to_string(),
        torrent_username: "me".to_string(),
        has_torrent_password: false,
        prune_finished_feeds: prune,
    }
}

fn save(core: &Core, settings: DownloadSettings) {
    let reply = core
        .call(Call::SetDownloadSettings {
            settings,
            anirss_token: Some("tok".to_string()),
            torrent_password: Some("pw".to_string()),
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
}

/// A show in the library: matched to an AniList media of the given status
/// whose last episode aired `last_aired_days_ago` days ago, or never.
fn series(core: &Core, source: u64, id: u64, status: &str, last_aired_days_ago: Option<i64>) {
    let name = format!("Show {id}");
    let s = fixtures::insert_series(
        core,
        source,
        SeriesKind::Show,
        &format!("/lib/{name}"),
        &name,
    );
    fixtures::insert_media(core, id, Some(&name), None, Some(2), status, "TV", Some(80));
    fixtures::match_series(core, s, Some(id), Some(1000 + id));
    if let Some(days) = last_aired_days_ago {
        let now = time::now_secs();
        fixtures::insert_airing(core, id, 1, now - (days + 7) * DAY);
        fixtures::insert_airing(core, id, 2, now - days * DAY);
    }
}

fn rules_json(names: &[&str]) -> String {
    let rules: Vec<_> = names
        .iter()
        .map(|n| {
            json!({"name": n, "feed_url": format!("https://nyaa.si/?page=rss&q={n}"),
                   "save_path": format!("/lib/{n}")})
        })
        .collect();
    json!({ "subscriptions": rules }).to_string()
}

/// The library and the three canned anirss answers a pruning read needs.
fn setup(prune: bool) -> (common::Dir, Arc<Core>, Arc<Collector>, Arc<FakeHttp>) {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save(&core, remote(prune));
    let src = fixtures::insert_source(&core, "/lib");
    series(&core, src, 1, "FINISHED", Some(16));
    series(&core, src, 2, "FINISHED", Some(15));
    series(&core, src, 3, "RELEASING", Some(40));
    series(&core, src, 4, "FINISHED", None);
    series(&core, src, 5, "CANCELLED", Some(16));
    (dir, core, c, http)
}

fn push_read(http: &FakeHttp) {
    http.push_for(
        "/subscriptions",
        200,
        rules_json(&["Show 1", "Show 2", "Show 3", "Show 4", "Show 5", "Unlinked"]),
    );
}

fn body_of(r: &HttpRequest) -> String {
    match &r.body {
        Some(Body::Json(v)) => v.to_string(),
        Some(Body::Form(pairs)) => pairs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&"),
        None => String::new(),
    }
}

fn removals(http: &FakeHttp) -> Vec<HttpRequest> {
    http.requests()
        .into_iter()
        .filter(|r| r.url.contains("/subscriptions/remove"))
        .collect()
}

fn read(core: &Core, c: &Collector) {
    let seen = c.events().iter().map(|e| e.seq).max().unwrap_or(0);
    assert_eq!(core.call(Call::ListSubscriptionRules).unwrap(), Reply::Ok);
    common::wait_for(
        c,
        move |e| e.seq > seen && matches!(e.body, EventBody::SubscriptionsRead { .. }),
        Duration::from_secs(10),
    );
}

/// Waits until a request whose url contains `part` has gone out, then a
/// beat longer so a removal that would follow it has had its chance.
fn settle(http: &FakeHttp, part: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !http.requests().iter().any(|r| r.url.contains(part)) {
        assert!(Instant::now() < deadline, "no request for {part}");
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(300));
}

#[test]
fn a_show_finished_for_sixteen_days_loses_its_feed_and_only_its_feed() {
    let (_dir, core, c, http) = setup(true);
    push_read(&http);
    http.push_for("app/version", 200, "v5.0.0");
    http.push_for(
        "/subscriptions/remove",
        200,
        json!({"dry_run": false, "plans": []}).to_string(),
    );
    http.push_for(
        "/subscriptions/remove",
        200,
        json!({"dry_run": false, "plans": []}).to_string(),
    );

    assert_eq!(core.call(Call::ListSubscriptionRules).unwrap(), Reply::Ok);
    let removed = common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SubscriptionRemoved { name } if name == "Show 1"),
        Duration::from_secs(10),
    );
    assert_eq!(removed.level, Level::Info);
    assert_eq!(removed.stage, Stage::Downloads);
    assert_eq!(
        removed.message,
        "Unsubscribed from Show 1: finished airing 16 days ago"
    );
    // The cancelled show past its 16 days goes too, the 15 day one,
    // the airing one, the dateless one and the unlinked rule stay
    common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SubscriptionRemoved { name } if name == "Show 5"),
        Duration::from_secs(10),
    );
    let sent = removals(&http);
    assert_eq!(sent.len(), 2, "{sent:?}");
    assert!(body_of(&sent[0]).contains("\"names\":[\"Show 1\"]"));
    assert!(body_of(&sent[1]).contains("\"names\":[\"Show 5\"]"));

    // The user's own removal, files and torrents kept, is the same call
    http.push_for(
        "/subscriptions/remove",
        200,
        json!({"dry_run": false, "plans": []}).to_string(),
    );
    let job = match core
        .call(Call::RemoveSubscription {
            name: "Show 1".to_string(),
            torrents: false,
            files: false,
        })
        .unwrap()
    {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    common::wait_job(&c, job);
    let sent = removals(&http);
    assert_eq!(body_of(&sent[0]), body_of(&sent[2]));
    assert_eq!(sent[0].url, sent[2].url);

    // Nothing else was asked of anirss or the client but the reads, the
    // version probe and these removals
    assert!(
        !http
            .requests()
            .iter()
            .any(|r| r.url.contains("torrents/delete") || r.url.contains("torrents/")),
        "no torrent was touched"
    );
    // The library series are all still there
    let count: i64 = core
        .store()
        .read(|conn| Ok(conn.query_row("SELECT COUNT(*) FROM series", [], |r| r.get(0))?))
        .unwrap();
    assert_eq!(count, 5);
}

#[test]
fn the_check_runs_once_a_day() {
    let (_dir, core, c, http) = setup(true);
    push_read(&http);
    http.push_for("app/version", 200, "v5.0.0");
    for _ in 0..2 {
        http.push_for(
            "/subscriptions/remove",
            200,
            json!({"dry_run": false, "plans": []}).to_string(),
        );
    }
    read(&core, &c);
    common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SubscriptionRemoved { name } if name == "Show 5"),
        Duration::from_secs(10),
    );
    assert_eq!(removals(&http).len(), 2);

    // Anirss still lists both (a stale read): the second read is the same day
    push_read(&http);
    http.push_for("app/version", 200, "v5.0.0");
    http.push_for(
        "/subscriptions/remove",
        200,
        json!({"dry_run": false, "plans": []}).to_string(),
    );
    read(&core, &c);
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(removals(&http).len(), 2, "no second check within the day");
}

#[test]
fn nothing_is_removed_with_the_setting_off() {
    let (_dir, core, c, http) = setup(false);
    push_read(&http);
    http.push_for("app/version", 200, "v5.0.0");
    http.push_for("/subscriptions/remove", 200, "{}");
    read(&core, &c);
    std::thread::sleep(Duration::from_millis(400));
    assert!(removals(&http).is_empty());
}

#[test]
fn nothing_is_removed_in_a_shell_that_turned_it_off() {
    let (_dir, core, c, http) = setup(true);
    core.disable_feed_pruning();
    push_read(&http);
    http.push_for("app/version", 200, "v5.0.0");
    http.push_for("/subscriptions/remove", 200, "{}");
    read(&core, &c);
    std::thread::sleep(Duration::from_millis(400));
    assert!(removals(&http).is_empty());
}

#[test]
fn nothing_is_removed_while_the_torrent_client_is_down() {
    let (_dir, core, c, http) = setup(true);
    push_read(&http);
    http.push_for("/subscriptions/remove", 200, "{}");
    // No answer queued for the client: the probe gets a 500
    read(&core, &c);
    settle(&http, "app/version");
    assert!(removals(&http).is_empty());

    // The client comes back and the next read, not yet a day on, tries again
    push_read(&http);
    http.push_for("app/version", 200, "v5.0.0");
    http.push_for(
        "/subscriptions/remove",
        200,
        json!({"dry_run": false, "plans": []}).to_string(),
    );
    http.push_for(
        "/subscriptions/remove",
        200,
        json!({"dry_run": false, "plans": []}).to_string(),
    );
    read(&core, &c);
    common::wait_for(
        &c,
        |e| matches!(&e.body, EventBody::SubscriptionRemoved { name } if name == "Show 1"),
        Duration::from_secs(10),
    );
}
