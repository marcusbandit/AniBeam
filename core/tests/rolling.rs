//! Rolling download end to end (issue #195), against `FakeHttp`: a batch is
//! held to a window of files through qBittorrent's file priorities, a long
//! pick of single releases adds only its first window, `RollMore` and
//! `SetRoll` move or end the roll, and a watched episode leaves the disk
//! once a later one is watched.
//!
//! `FakeHttp` hands each canned reply out once, so every case queues more
//! than it expects to use. Where a later pass needs a different file list,
//! the case waits until the earlier pass is done before queuing it.

mod common;
mod fixtures;

use std::path::Path;
use std::time::{Duration, Instant};

use anibeam_core::events::Collector;
use anibeam_core::library::classifier;
use anibeam_core::net::{Body, FakeHttp, HttpRequest};
use anibeam_core::*;
use serde_json::{Value, json};

const WAIT: Duration = Duration::from_secs(25);

fn settings() -> DownloadSettings {
    DownloadSettings {
        anirss_mode: AnirssMode::Remote,
        anirss_url: "http://anirss.test".to_string(),
        anirss_command: "anirss".to_string(),
        jackett_command: String::new(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_mode: TorrentMode::Existing,
        torrent_url: "http://localhost:9090".to_string(),
        torrent_username: "me".to_string(),
        has_torrent_password: false,
        saved_clients: Vec::new(),
        active_client: String::new(),
        prune_finished_feeds: true,
        rolling: Default::default(),
    }
}

/// Points the XDG homes at `dir`, so no case reads the machine's own anirss
/// files.
fn isolate_xdg(dir: &Path) {
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", dir);
        std::env::set_var("XDG_STATE_HOME", dir);
    }
}

fn save_settings(core: &Core) {
    let reply = core
        .call(Call::SetDownloadSettings {
            settings: settings(),
            anirss_token: Some("tok".to_string()),
            torrent_password: Some("pw".to_string()),
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
}

fn started(core: &Core, call: Call) -> u64 {
    match core.call(call).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

fn release(title: &str, link: &str) -> Release {
    serde_json::from_value(json!({"title": title, "link": link, "source": "nyaa"})).unwrap()
}

fn batch_release(hash: &str) -> Release {
    release(
        "[Judas] Black Clover (01-40) [1080p][Batch]",
        &format!("magnet:?xt=urn:btih:{hash}&dn=Black+Clover"),
    )
}

const BATCH_DIR: &str = "Black Clover";

/// The file name of episode `n` (from 1) inside the batch.
fn episode_name(n: usize) -> String {
    format!("{BATCH_DIR}/Black Clover - {n:03}.mkv")
}

/// qBittorrent's file list for the 40-episode batch, each file at the
/// priority `priority` gives for its index.
fn files_json(priority: impl Fn(usize) -> i64) -> String {
    files_json_at(priority, |_| 0.0)
}

/// The same with a progress per file; a file with none cannot have been
/// watched, so a case about watching needs the files it watches to have come in.
fn files_json_at(priority: impl Fn(usize) -> i64, progress: impl Fn(usize) -> f64) -> String {
    let files: Vec<Value> = (0..40)
        .map(|i| {
            json!({"name": episode_name(i + 1), "size": 100, "progress": progress(i), "priority": priority(i)})
        })
        .collect();
    Value::Array(files).to_string()
}

fn torrent_json(hash: &str, save_path: &str) -> String {
    json!([{
        "hash": hash,
        "name": "[Judas] Black Clover (01-40) [1080p][Batch]",
        "state": "downloading",
        "progress": 0.5,
        "size": 4000,
        "save_path": save_path,
        "content_path": format!("{save_path}/{BATCH_DIR}"),
        "added_on": 1,
        "completion_on": -1,
        "seq_dl": false
    }])
    .to_string()
}

fn push_many(http: &FakeHttp, url: &str, body: &str, times: usize) {
    for _ in 0..times {
        http.push_for(url, 200, body);
    }
}

fn form_pairs(r: &HttpRequest) -> Vec<(String, String)> {
    match &r.body {
        Some(Body::Form(pairs)) => pairs.clone(),
        _ => Vec::new(),
    }
}

/// One `torrents/filePrio` call: its position among the requests, the ids
/// joined as sent, and the priority.
#[derive(Debug, PartialEq)]
struct Prio {
    at: usize,
    ids: String,
    priority: i64,
}

fn prio_calls(http: &FakeHttp) -> Vec<Prio> {
    http.requests()
        .iter()
        .enumerate()
        .filter(|(_, r)| r.url.contains("torrents/filePrio"))
        .map(|(at, r)| {
            let f = form_pairs(r);
            let get = |k: &str| {
                f.iter()
                    .find(|(name, _)| name == k)
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default()
            };
            Prio {
                at,
                ids: get("id"),
                priority: get("priority").parse().unwrap(),
            }
        })
        .collect()
}

/// Polls the recorded requests until `found` returns something, or fails.
fn wait_requests<T>(http: &FakeHttp, what: &str, found: impl Fn(&FakeHttp) -> Option<T>) -> T {
    let deadline = Instant::now() + WAIT;
    loop {
        if let Some(v) = found(http) {
            return v;
        }
        assert!(
            Instant::now() < deadline,
            "never saw {what}; requests: {:#?}; priorities: {:#?}",
            http.requests().iter().map(|r| &r.url).collect::<Vec<_>>(),
            prio_calls(http)
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn ids(range: impl Iterator<Item = usize>) -> String {
    range.map(|i| i.to_string()).collect::<Vec<_>>().join("|")
}

fn reads(c: &Collector) -> usize {
    c.events()
        .iter()
        .filter(|e| matches!(e.body, EventBody::TorrentsRead { .. }))
        .count()
}

/// Waits for a `TorrentsRead` beyond the first `skip` whose torrents satisfy
/// `pred`, and returns those torrents.
fn wait_read(c: &Collector, skip: usize, pred: impl Fn(&[Torrent]) -> bool) -> Vec<Torrent> {
    let seen = |events: &[Event]| {
        events
            .iter()
            .filter_map(|e| match &e.body {
                EventBody::TorrentsRead { torrents, .. } => Some(torrents.clone()),
                _ => None,
            })
            .skip(skip)
            .find(|t| pred(t))
    };
    assert!(
        c.wait_for(|events| seen(events).is_some(), WAIT),
        "no matching torrent read; saw {:#?}",
        c.events()
    );
    seen(&c.events()).unwrap()
}

/// A core with a 40-episode batch just added and its first pass done: the
/// window holds the first 12, so 28 files were skipped and the first set to
/// top priority.
struct Rig {
    http: std::sync::Arc<FakeHttp>,
    core: std::sync::Arc<Core>,
    c: std::sync::Arc<Collector>,
    _dir: common::Dir,
    hash: String,
}

fn batch_rig() -> Rig {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save_settings(&core);

    let hash = "ab".repeat(20);
    http.push_for(
        "/downloads",
        200,
        json!({"count": 1, "save_path": "/dl/Black Clover", "hashes": [hash]}).to_string(),
    );
    // The burst after the add: one read of the client, one file list, all
    // files at normal priority as a fresh batch has them.
    http.push_for("torrents/info", 200, torrent_json(&hash, "/dl/save"));
    http.push_for("torrents/files", 200, files_json(|_| 1));
    push_many(&http, "torrents/filePrio", "", 10);

    let job = started(
        &core,
        Call::DownloadReleasesWith {
            items: vec![batch_release(&hash)],
            name: Some("Black Clover".to_string()),
            options: DownloadOptions::default(),
        },
    );
    let done = common::wait_job(&c, job);
    assert!(
        matches!(&done.body, EventBody::TorrentsAdded { .. }),
        "{:?}",
        done.body
    );
    Rig {
        http,
        core,
        c,
        _dir: dir,
        hash,
    }
}

#[test]
fn a_batch_of_forty_episodes_rolls_twelve_ahead() {
    let rig = batch_rig();

    let skip = wait_requests(&rig.http, "the skip call", |h| {
        prio_calls(h).into_iter().find(|p| p.priority == 0)
    });
    assert_eq!(skip.ids, ids(12..40), "the 28 files past the window");
    let first = wait_requests(&rig.http, "the top priority call", |h| {
        prio_calls(h).into_iter().find(|p| p.priority == 7)
    });
    assert_eq!(first.ids, "0", "the next episode to watch goes first");
    let calls = prio_calls(&rig.http);
    assert_eq!(calls.len(), 2, "nothing else is changed: {calls:#?}");
    let hashes: Vec<String> = rig
        .http
        .requests()
        .iter()
        .filter(|r| r.url.contains("torrents/filePrio"))
        .filter_map(|r| {
            form_pairs(r)
                .into_iter()
                .find(|(k, _)| k == "hash")
                .map(|(_, v)| v)
        })
        .collect();
    assert!(hashes.iter().all(|h| *h == rig.hash), "{hashes:?}");
    rig.core.shutdown();
}

#[test]
fn a_pick_of_forty_single_episodes_adds_only_twelve_now() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save_settings(&core);

    // Handed over in reverse, so the order is the roll's own.
    let items: Vec<Release> = (1..=40)
        .rev()
        .map(|n| {
            release(
                &format!("[Erai-raws] Black Clover - {n:03} [1080p].mkv"),
                &format!("magnet:?xt=urn:btih:{n:040x}"),
            )
        })
        .collect();
    let hashes: Vec<String> = (1..=12).map(|n| format!("{n:040x}")).collect();
    http.push_for(
        "/downloads",
        200,
        json!({"count": 12, "save_path": "/dl/Black Clover", "hashes": hashes}).to_string(),
    );
    let job = started(
        &core,
        Call::DownloadReleasesWith {
            items,
            name: Some("Black Clover".to_string()),
            options: DownloadOptions::default(),
        },
    );
    let done = common::wait_job(&c, job);
    assert!(
        matches!(&done.body, EventBody::TorrentsAdded { .. }),
        "{:?}",
        done.body
    );

    let adds: Vec<HttpRequest> = http
        .requests()
        .into_iter()
        .filter(|r| r.url.ends_with("/downloads"))
        .collect();
    assert_eq!(adds.len(), 1, "one add call");
    let Some(Body::Json(body)) = &adds[0].body else {
        panic!("{:?}", adds[0].body)
    };
    let sent: Vec<&str> = body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["title"].as_str().unwrap())
        .collect();
    let want: Vec<String> = (1..=12)
        .map(|n| format!("[Erai-raws] Black Clover - {n:03} [1080p].mkv"))
        .collect();
    assert_eq!(sent, want, "episodes 1 to 12, in episode order");
    core.shutdown();
}

#[test]
fn roll_more_widens_the_window_and_set_roll_off_fetches_the_rest_and_ends_the_roll() {
    let rig = batch_rig();
    let (http, core, c) = (&rig.http, &rig.core, &rig.c);
    wait_requests(http, "the first pass", |h| {
        prio_calls(h).into_iter().find(|p| p.priority == 7)
    });

    // The client as the first pass left it: first file on top, the rest of
    // the window normal, everything past it skipped. Queued only now, since
    // the first pass read a list with every file at normal.
    let state = |i: usize| match i {
        0 => 7,
        1..=11 => 1,
        _ => 0,
    };
    push_many(
        http,
        "torrents/info",
        &torrent_json(&rig.hash, "/dl/save"),
        40,
    );
    push_many(http, "torrents/files", &files_json(state), 40);
    push_many(http, "torrents/filePrio", "", 40);
    push_many(http, "transfer/info", &json!({}).to_string(), 20);

    // The page's read finds the roll on the torrent.
    assert_eq!(core.call(Call::ListTorrents).unwrap(), Reply::Ok);
    let listed = wait_read(c, 0, |t| t.len() == 1 && t[0].roll.is_some());
    let roll = listed[0].roll.clone().unwrap();
    assert_eq!(
        (
            roll.ahead,
            roll.next,
            roll.through,
            roll.total,
            roll.watched
        ),
        (12, 1, 12, 40, 0)
    );
    assert!(roll.delete_watched);
    assert_eq!(
        prio_calls(http).len(),
        2,
        "a window already in place changes nothing"
    );

    // Four more: files 13 to 16 turn on, and only those.
    let mark = http.requests().len();
    let seen = reads(c);
    assert_eq!(
        core.call(Call::RollMore {
            id: roll.id,
            count: 4
        })
        .unwrap(),
        Reply::Ok
    );
    let more = wait_requests(http, "files 13 to 16 turning on", |h| {
        prio_calls(h)
            .into_iter()
            .find(|p| p.at >= mark && p.priority == 1)
    });
    assert_eq!(more.ids, ids(12..16));
    let widened = wait_read(c, seen, |t| {
        t.len() == 1 && t[0].roll.as_ref().is_some_and(|r| r.through == 16)
    });
    assert_eq!(widened[0].roll.as_ref().unwrap().total, 40);

    // Rolling off: everything not yet watched comes, and the roll goes.
    let mark = http.requests().len();
    let seen = reads(c);
    assert_eq!(
        core.call(Call::SetRoll {
            id: roll.id,
            ahead: None,
            on: Some(false),
            delete_watched: None,
        })
        .unwrap(),
        Reply::Ok
    );
    let rest = wait_requests(http, "every remaining file turning on", |h| {
        prio_calls(h)
            .into_iter()
            .find(|p| p.at >= mark && p.priority == 1 && p.ids.ends_with("|39"))
    });
    assert_eq!(
        rest.ids,
        format!("0|{}", ids(12..40)),
        "the first file leaves top priority too, nothing is skipped"
    );
    let ended = wait_read(c, seen, |t| t.len() == 1 && t[0].roll.is_none());
    assert_eq!(ended[0].hash, rig.hash);
    core.shutdown();
}

#[test]
fn a_watched_episode_moves_the_window_and_leaves_the_disk_once_two_later_ones_are_watched() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    isolate_xdg(dir.path());
    save_settings(&core);

    // The batch on disk: episodes 1 to 4, in the library.
    let save_path = dir.path().join("dl");
    let show = save_path.join(BATCH_DIR);
    std::fs::create_dir_all(&show).unwrap();
    let on_disk = |n: usize| save_path.join(episode_name(n));
    for n in 1..=4 {
        std::fs::write(on_disk(n), b"x").unwrap();
    }
    let save = save_path.to_string_lossy().into_owned();
    let src = fixtures::insert_source(&core, &save);
    let series = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        &show.to_string_lossy(),
        "Black Clover",
    );
    for n in 1..=4 {
        fixtures::insert_file(
            &core,
            series,
            &on_disk(n).to_string_lossy(),
            n as f64,
            None,
            "episode",
            1,
        );
    }
    let key = |n: usize| classifier::number_key(None, n as f64);
    // Episodes 1 to 3 finished after the roll begins; episode 4 was seen
    // long before it and must not move the window.
    let later = time::now_secs() + 3600;
    fixtures::insert_completed(&core, series, &key(1), later);
    fixtures::insert_completed(&core, series, &key(2), later + 1);
    fixtures::insert_completed(&core, series, &key(3), later + 2);
    fixtures::insert_completed(&core, series, &key(4), 1);

    let hash = "cd".repeat(20);
    http.push_for(
        "/downloads",
        200,
        json!({"count": 1, "save_path": save, "hashes": [hash]}).to_string(),
    );
    http.push_for("torrents/info", 200, torrent_json(&hash, &save));
    http.push_for(
        "torrents/files",
        200,
        files_json_at(|_| 1, |i| if i < 4 { 1.0 } else { 0.0 }),
    );
    push_many(&http, "torrents/filePrio", "", 10);
    push_many(&http, "torrents/recheck", "", 5);
    let job = started(
        &core,
        Call::DownloadReleasesWith {
            items: vec![batch_release(&hash)],
            name: Some("Black Clover".to_string()),
            options: DownloadOptions::default(),
        },
    );
    common::wait_job(&c, job);

    let recheck = wait_requests(&http, "the recheck", |h| {
        h.requests()
            .into_iter()
            .find(|r| r.url.contains("torrents/recheck"))
    });
    assert_eq!(
        form_pairs(&recheck),
        vec![("hashes".to_string(), hash.clone())]
    );
    assert!(
        !on_disk(1).exists(),
        "episode 1 is more than one behind episode 3"
    );
    assert!(
        on_disk(2).exists() && on_disk(3).exists(),
        "the one just finished and the one kept behind it stay"
    );
    assert!(
        on_disk(4).exists(),
        "an episode seen before the roll counts as not watched yet"
    );
    // Three watched: the window starts at episode 4 and reaches 12 ahead.
    let calls = prio_calls(&http);
    let top = calls.iter().find(|p| p.priority == 7).expect("a top call");
    assert_eq!(top.ids, "3", "episode 4 is the next to watch");
    core.shutdown();
}
