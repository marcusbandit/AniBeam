//! Where downloads save (#1005): Anibeam's own qBittorrent saves a new show into the first
//! library folder rather than a folder in Downloads, a show already in the library into its
//! own folder, and a folder picked in Settings wins over both. The user's own qBittorrent
//! keeps its own default. Against `FakeHttp`, on the core's own engine.
#![cfg(unix)]

mod common;
use common::started;

use std::path::Path;
use std::sync::Arc;

use anibeam_core::events::Collector;
use anibeam_core::net::{Body, FakeHttp, HttpRequest};
use anibeam_core::*;
use serde_json::json;

fn settings(mode: TorrentMode, torrent_url: &str) -> DownloadSettings {
    DownloadSettings {
        anirss_mode: AnirssMode::Remote,
        anirss_command: "anirss".to_string(),
        jackett_command: String::new(),
        anirss_url: "http://anirss.test".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_mode: mode,
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

struct Rig {
    dir: common::Dir,
    core: Arc<Core>,
    c: Arc<Collector>,
    http: Arc<FakeHttp>,
}

/// A core whose downloads use Anibeam's own qBittorrent, already up: its Web UI answers.
fn managed() -> Rig {
    let mut s = settings(TorrentMode::Managed, "http://127.0.0.1:47321");
    s.torrent_username = "anibeam".to_string();
    let r = rig(s);
    assert_eq!(status(&r.core).settings.torrent_mode, TorrentMode::Managed);
    r
}

fn rig(s: DownloadSettings) -> Rig {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    let reply = core
        .call(Call::SetDownloadSettings {
            settings: s,
            anirss_token: None,
            torrent_password: None,
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
    Rig { dir, core, c, http }
}

fn status(core: &Core) -> DownloadsStatus {
    match core.call(Call::GetDownloads).unwrap() {
        Reply::Downloads { status } => status,
        other => panic!("{other:?}"),
    }
}

/// An empty library folder, added and scanned.
fn add_library(r: &Rig, name: &str) -> String {
    let lib = r.dir.path().join(name);
    std::fs::create_dir_all(&lib).unwrap();
    let path = lib.to_string_lossy().into_owned();
    r.core.call(Call::AddSource { path: path.clone() }).unwrap();
    path
}

fn episode(n: u32) -> Release {
    serde_json::from_value(json!({
        "title": format!("[Group] Show - {n:02} (1080p)"),
        "link": format!("magnet:?xt=urn:btih:{n:040x}"),
        "source": "feed",
    }))
    .unwrap()
}

/// Downloads one episode and returns the folder the add named.
fn download(r: &Rig, series: Option<u64>) -> String {
    r.http.push_for("app/version", 200, "v5.0.0");
    r.http.push_for("torrents/add", 200, "Ok.");
    let job = started(
        &r.core,
        Call::DownloadReleasesWith {
            items: vec![episode(1)],
            name: None,
            options: DownloadOptions {
                series,
                ..Default::default()
            },
        },
    );
    let done = common::wait_job(&r.c, job);
    let EventBody::TorrentsAdded { result } = &done.body else {
        panic!("{done:?}");
    };
    let sent = add_field(&r.http, "savepath");
    assert_eq!(sent, result.save_path);
    sent
}

fn add_field(http: &FakeHttp, key: &str) -> String {
    let add: HttpRequest = http
        .requests()
        .into_iter()
        .rfind(|r| r.url.ends_with("/api/v2/torrents/add"))
        .expect("no add");
    match &add.body {
        Some(Body::Form(pairs)) => pairs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default(),
        other => panic!("not a form: {other:?}"),
    }
}

fn asked_default(http: &FakeHttp) -> bool {
    http.requests()
        .iter()
        .any(|r| r.url.contains("app/defaultSavePath"))
}

#[test]
fn anibeams_own_saves_a_new_show_into_the_library_folder() {
    let r = managed();
    let lib = add_library(&r, "Anime");
    let saved = download(&r, None);
    assert_eq!(saved, format!("{lib}/Show"));
    // The client's own folder is never asked for: the library answers first
    assert!(!asked_default(&r.http));
    assert_eq!(
        status(&r.core).save_folder,
        SaveFolder {
            path: lib,
            from: SaveFolderFrom::Library,
            here: true,
        }
    );
}

#[test]
fn the_first_library_folder_added_wins_and_a_missing_one_is_skipped() {
    let r = managed();
    let first = add_library(&r, "First");
    let second = add_library(&r, "Second");
    assert_eq!(status(&r.core).save_folder.path, first);
    std::fs::remove_dir_all(&first).unwrap();
    assert_eq!(status(&r.core).save_folder.path, second);
}

#[test]
fn with_no_library_folder_the_client_decides_until_one_is_added() {
    let r = managed();
    let before = status(&r.core).save_folder;
    assert_eq!(before.from, SaveFolderFrom::Client);
    assert_eq!(before.path, "");
    let fallback = r.dir.path().join("fallback").to_string_lossy().into_owned();
    r.http
        .push_for("app/defaultSavePath", 200, fallback.clone());
    assert_eq!(download(&r, None), format!("{fallback}/Show"));

    // Adding the first library folder moves the default there
    let lib = add_library(&r, "Anime");
    assert_eq!(status(&r.core).save_folder.from, SaveFolderFrom::Library);
    assert_eq!(download(&r, None), format!("{lib}/Show"));
}

#[test]
fn a_show_already_in_the_library_saves_into_its_own_folder() {
    let r = managed();
    let show = common::library_show(r.dir.path(), &r.core, &r.c);
    assert_eq!(download(&r, Some(1)), show);
    // A show not in the library yet gets a folder of its own beside it
    let lib = Path::new(&show)
        .parent()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert_eq!(download(&r, None), format!("{lib}/Show"));
}

#[test]
fn a_folder_picked_in_settings_wins_and_reset_goes_back_to_the_library() {
    let r = managed();
    let lib = add_library(&r, "Anime");
    let picked = r.dir.path().join("Picked").to_string_lossy().into_owned();
    std::fs::create_dir_all(&picked).unwrap();
    let reply = r
        .core
        .call(Call::SetSaveFolder {
            folder: Some(picked.clone()),
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
    let st = status(&r.core).save_folder;
    assert_eq!(
        (st.path.as_str(), st.from),
        (picked.as_str(), SaveFolderFrom::Chosen)
    );
    assert_eq!(download(&r, None), format!("{picked}/Show"));
    // Films follow the picked folder too, into its Movies folder
    r.http.push_for("app/version", 200, "v5.0.0");
    r.http.push_for("torrents/add", 200, "Ok.");
    let job = started(
        &r.core,
        Call::DownloadMovie {
            link: format!("magnet:?xt=urn:btih:{:040x}", 7),
            needs_video: false,
        },
    );
    common::wait_job(&r.c, job);
    assert_eq!(add_field(&r.http, "savepath"), format!("{picked}/Movies"));

    // An unmounted drive falls back instead of being recreated under its mount point
    std::fs::remove_dir_all(&picked).unwrap();
    assert_eq!(status(&r.core).save_folder.from, SaveFolderFrom::Library);
    std::fs::create_dir_all(&picked).unwrap();

    r.core.call(Call::SetSaveFolder { folder: None }).unwrap();
    let st = status(&r.core).save_folder;
    assert_eq!(
        (st.path.as_str(), st.from),
        (lib.as_str(), SaveFolderFrom::Library)
    );
}

#[test]
fn a_folder_that_is_not_a_full_path_missing_or_the_root_is_refused() {
    let r = managed();
    let missing = r.dir.path().join("Gone").to_string_lossy().into_owned();
    for folder in ["Anime".to_string(), missing, "/".to_string()] {
        let refused = r.core.call(Call::SetSaveFolder {
            folder: Some(folder),
        });
        assert!(
            matches!(refused, Err(CoreError::Invalid { ref field, .. }) if field == "folder"),
            "{refused:?}"
        );
    }
    assert_eq!(status(&r.core).save_folder.from, SaveFolderFrom::Client);
}

#[test]
fn the_users_own_client_keeps_its_own_default_folder() {
    let r = rig(settings(TorrentMode::Existing, "http://192.0.2.10:9090"));
    add_library(&r, "Anime");
    let st = status(&r.core).save_folder;
    assert_eq!(st.from, SaveFolderFrom::Client);
    assert!(!st.here);
    r.http.push_for("app/defaultSavePath", 200, "/srv/TV");
    assert_eq!(download(&r, None), "/srv/TV/Show");
    // A folder picked on this machine is no path on a client elsewhere
    let picked = r.dir.path().join("Picked").to_string_lossy().into_owned();
    std::fs::create_dir_all(&picked).unwrap();
    r.core
        .call(Call::SetSaveFolder {
            folder: Some(picked),
        })
        .unwrap();
    assert_eq!(status(&r.core).save_folder.from, SaveFolderFrom::Client);
}
