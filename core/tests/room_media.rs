//! Get media (#889) end to end: the native relay runs in this process, and a host
//! and a member each have a core with a torrent client faked over `FakeHttp`. The
//! member's library lacks the room's episode; it asks, the host's app answers
//! sealed, and the member downloads either the host's own torrent or, when the host
//! only has the file, the same release from the member's own source. The host sees
//! the progress, and the member ends with the episode found in its library.
#![cfg(unix)]

mod common;
mod fixtures;

use std::sync::Arc;
use std::time::Duration;

use anibeam_core::events::Collector;
use anibeam_core::net::{Body, FakeHttp, HttpRequest};
use anibeam_core::*;
use rusqlite::params;
use serde_json::json;

const WAIT: Duration = Duration::from_secs(30);
const QBT: &str = "http://localhost:9090";
const FEED: &str = "https://feed.example/rss?q={query}";

/// An invented release of episode 5, and the torrent it came in.
const NAME: &str = "[Lantern] Cowboy Bebop - 05 (1080p)";
const HASH: &str = "4f1c0e5a6b7d8e9f00112233445566778899aabb";

struct Relay {
    _rt: tokio::runtime::Runtime,
    url: String,
}

fn relay() -> Relay {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let listener = rt
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let addr = listener.local_addr().unwrap();
    rt.spawn(async move {
        let _ = anibeam_relay::serve(listener).await;
    });
    Relay {
        _rt: rt,
        url: format!("http://{addr}"),
    }
}

struct Person {
    _dir: common::Dir,
    core: Arc<Core>,
    c: Arc<Collector>,
    http: Arc<FakeHttp>,
    series: u64,
    files: Vec<u64>,
}

fn settings() -> DownloadSettings {
    DownloadSettings {
        anirss_mode: AnirssMode::Remote,
        anirss_command: "anirss".to_string(),
        jackett_command: String::new(),
        anirss_url: "http://anirss.test".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_mode: TorrentMode::Existing,
        torrent_url: QBT.to_string(),
        torrent_username: "me".to_string(),
        has_torrent_password: false,
        saved_clients: Vec::new(),
        active_client: String::new(),
        prune_finished_feeds: true,
        rolling: Default::default(),
        engine: DownloadsEngine::Native,
    }
}

fn file_path(n: f64) -> String {
    format!("/lib/Bebop/[Lantern] Cowboy Bebop - {n:02} (1080p).mkv")
}

/// A core whose library has Cowboy Bebop with `episodes`, and whose downloads go
/// to a fake qBittorrent on the core's own engine, one source added by hand.
fn person(relay: &Relay, name: &str, episodes: &[f64]) -> Person {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    // No case reads the machine's own anirss config or saved password
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", dir.path());
        std::env::set_var("XDG_STATE_HOME", dir.path());
    }
    let reply = core
        .call(Call::SetDownloadSettings {
            settings: settings(),
            anirss_token: None,
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
    http.push_for("app/defaultSavePath", 200, "/srv/TV");
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(&core, "/lib");
    let series =
        fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    let files = episodes
        .iter()
        .map(|n| fixtures::insert_episode_file(&core, series, &file_path(*n), *n, now))
        .collect();
    fixtures::media(1, "Cowboy Bebop")
        .episodes(26)
        .insert(&core);
    fixtures::match_series(&core, series, Some(1), Some(21));
    let mut p = match core.call(Call::GetPreferences).unwrap() {
        Reply::Preferences { preferences } => preferences,
        other => panic!("{other:?}"),
    };
    p.room_name = name.into();
    p.room_relay = relay.url.clone();
    core.call(Call::SetPreferences { preferences: p }).unwrap();
    Person {
        _dir: dir,
        core,
        c,
        http,
        series,
        files,
    }
}

fn open(p: &Person, file: u64) -> u64 {
    match p.core.call(Call::OpenPlayback { file }).unwrap() {
        Reply::Playback { session } => session.session,
        other => panic!("{other:?}"),
    }
}

fn room(reply: Result<Reply, CoreError>) -> Option<RoomInfo> {
    match reply.unwrap() {
        Reply::Room { room } => room,
        other => panic!("{other:?}"),
    }
}

/// The torrent client's view of one torrent, as qBittorrent writes it.
fn torrent(hash: &str, name: &str, progress: f64, eta: i64) -> String {
    json!([{
        "hash": hash, "name": name, "state": "downloading", "progress": progress, "eta": eta,
        "save_path": "/lib/Bebop", "content_path": file_path(5.0),
        "magnet_uri": format!("magnet:?xt=urn:btih:{hash}&dn=x&tr=udp%3A%2F%2Ftracker.example%3A1337"),
    }])
    .to_string()
}

fn form(r: &HttpRequest) -> Vec<(String, String)> {
    match &r.body {
        Some(Body::Form(pairs)) => pairs.clone(),
        other => panic!("not a form: {other:?}"),
    }
}

/// The urls the member's client was handed.
fn added_urls(p: &Person) -> Vec<String> {
    p.http
        .requests()
        .iter()
        .filter(|r| r.url.ends_with("api/v2/torrents/add"))
        .flat_map(form)
        .filter(|(k, _)| k == "urls")
        .map(|(_, v)| v)
        .collect()
}

fn ben_in(room: &RoomInfo, you: u32) -> Option<&RoomMember> {
    room.members.iter().find(|m| m.id == you)
}

/// Ann hosts episode 5, Ben joins without it and asks for it. Returns Ben's id.
fn room_with_ben_missing(ann: &Person, ben: &Person) -> u32 {
    let ann_s = open(ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate { session: ann_s })).unwrap();
    let joined = room(ben.core.call(Call::RoomJoin {
        code: made.code.clone(),
    }))
    .unwrap();
    common::wait_for(
        &ben.c,
        |e| {
            matches!(&e.body, EventBody::RoomEpisode { series, file: None, .. }
                if *series == Some(ben.series))
        },
        WAIT,
    );
    // The host's key has reached Ben's member list, and Ben lacks the episode
    common::wait_for(
        &ben.c,
        |e| matches!(&e.body, EventBody::RoomChanged { room: Some(r) } if r.can_fetch),
        WAIT,
    );
    joined.you
}

/// Ben's download runs: Ann sees it at 40 percent with two minutes left.
fn ann_sees_progress(ann: &Person, ben_id: u32) {
    common::wait_for(
        &ann.c,
        |e| match &e.body {
            EventBody::RoomChanged { room: Some(r) } => ben_in(r, ben_id).is_some_and(|m| {
                m.ready == RoomReady::Downloading && m.progress == Some(40.0) && m.eta == Some(120)
            }),
            _ => false,
        },
        WAIT,
    );
}

/// The download finishes and the library takes the file in: Ben's core finds it,
/// says so, and Ben's player attaching makes him ready for Ann too.
fn ben_gets_it(ann: &Person, ben: &Person, ben_id: u32, hash: &str) {
    ben.http.push_for(
        &format!("torrents/info?hashes={hash}"),
        200,
        torrent(hash, NAME, 1.0, 0),
    );
    let file = fixtures::insert_episode_file(
        &ben.core,
        ben.series,
        &file_path(5.0),
        5.0,
        anibeam_core::time::now_secs(),
    );
    common::wait_for(
        &ben.c,
        |e| matches!(&e.body, EventBody::RoomEpisode { file: Some(f), .. } if *f == file),
        WAIT,
    );
    common::wait_for(
        &ben.c,
        |e| {
            e.plain_words
                .as_ref()
                .is_some_and(|w| w.id == "core-room-media-here")
        },
        WAIT,
    );
    let ben_s = open(ben, file);
    let attached = room(ben.core.call(Call::RoomAttach { session: ben_s })).unwrap();
    assert_eq!(attached.fetch, RoomFetch::Idle);
    common::wait_for(
        &ann.c,
        |e| match &e.body {
            EventBody::RoomChanged { room: Some(r) } => ben_in(r, ben_id)
                .is_some_and(|m| m.ready == RoomReady::Ready && m.progress.is_none()),
            _ => false,
        },
        WAIT,
    );
}

#[test]
fn a_member_downloads_the_host_s_torrent() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[5.0]);
    let ben = person(&relay, "Ben", &[4.0]);
    // Ann's file came from a torrent her client still holds
    let path = file_path(5.0);
    ann.core
        .store()
        .write(move |c| {
            c.execute(
                "INSERT INTO file_origins (path, info_hash, torrent_name, recorded_at) VALUES (?1, ?2, ?3, ?4)",
                params![path, HASH, NAME, anibeam_core::time::now_secs()],
            )?;
            Ok(())
        })
        .unwrap();
    ann.http.push_for(
        &format!("torrents/info?hashes={HASH}"),
        200,
        torrent(HASH, NAME, 1.0, 0),
    );
    ben.http.push_for("torrents/add", 200, "Ok.");
    for _ in 0..3 {
        ben.http.push_for(
            &format!("torrents/info?hashes={HASH}"),
            200,
            torrent(HASH, NAME, 0.4, 120),
        );
    }

    let ben_id = room_with_ben_missing(&ann, &ben);
    let asked = room(ben.core.call(Call::RoomGetMedia)).unwrap();
    assert_eq!(asked.fetch, RoomFetch::Asking);
    assert!(!asked.can_fetch);

    // Ann's app answers with the torrent, and Ben's client is handed its magnet
    common::wait_for(
        &ann.c,
        |e| {
            e.plain_words
                .as_ref()
                .is_some_and(|w| w.id == "core-room-media-sent")
        },
        WAIT,
    );
    common::wait_for(
        &ben.c,
        |e| matches!(&e.body, EventBody::RoomChanged { room: Some(r) } if r.fetch == RoomFetch::Downloading),
        WAIT,
    );
    ann_sees_progress(&ann, ben_id);
    let urls = added_urls(&ben);
    assert_eq!(urls.len(), 1, "{urls:?}");
    assert!(urls[0].starts_with(&format!("magnet:?xt=urn:btih:{HASH}")));
    assert!(urls[0].contains("tracker.example"), "trackers come along");
    // The relay never saw the hash or the release: only Ann's and Ben's apps did
    assert!(
        ben.http
            .requests()
            .iter()
            .all(|r| !r.url.contains("feed.example")),
        "the torrent route searches nothing"
    );

    ben_gets_it(&ann, &ben, ben_id, HASH);
}

#[test]
fn a_member_finds_the_host_s_release_by_its_name() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[5.0]);
    let ben = person(&relay, "Ben", &[4.0]);
    // Ann's file has no torrent on record: she offers its name. Ben's source has
    // another group's copy, the same group at another resolution, a batch, and the
    // one release that is Ann's.
    let same = "1111111111111111111111111111111111111111";
    let item = |title: &str, hash: &str| {
        format!("<item><title>{title}</title><link>magnet:?xt=urn:btih:{hash}</link></item>")
    };
    let feed = format!(
        "<?xml version=\"1.0\"?><rss version=\"2.0\"><channel><title>f</title>{}{}{}{}</channel></rss>",
        item(
            "[Other] Cowboy Bebop - 05 (1080p)",
            "2222222222222222222222222222222222222222"
        ),
        item(
            "[Lantern] Cowboy Bebop - 05 (720p)",
            "3333333333333333333333333333333333333333"
        ),
        item(
            "[Lantern] Cowboy Bebop (01-26) (1080p) [Batch]",
            "4444444444444444444444444444444444444444"
        ),
        item(NAME, same),
    );
    for _ in 0..4 {
        ben.http.push_for("feed.example", 200, feed.clone());
    }
    ben.http.push_for("torrents/add", 200, "Ok.");
    for _ in 0..3 {
        ben.http.push_for(
            &format!("torrents/info?hashes={same}"),
            200,
            torrent(same, NAME, 0.4, 120),
        );
    }

    let ben_id = room_with_ben_missing(&ann, &ben);
    room(ben.core.call(Call::RoomGetMedia)).unwrap();
    common::wait_for(
        &ann.c,
        |e| {
            e.plain_words
                .as_ref()
                .is_some_and(|w| w.id == "core-room-media-sent-name")
        },
        WAIT,
    );
    common::wait_for(
        &ben.c,
        |e| {
            e.plain_words
                .as_ref()
                .is_some_and(|w| w.id == "core-room-media-searching")
        },
        WAIT,
    );
    ann_sees_progress(&ann, ben_id);
    let urls = added_urls(&ben);
    assert_eq!(urls, vec![format!("magnet:?xt=urn:btih:{same}")]);
    let searched = ben
        .http
        .requests()
        .iter()
        .find(|r| r.url.contains("feed.example"))
        .map(|r| r.url.clone())
        .unwrap();
    assert!(searched.contains("Lantern"), "{searched}");

    ben_gets_it(&ann, &ben, ben_id, same);
}

#[test]
fn nothing_found_fails_and_get_media_can_be_tried_again() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[5.0]);
    let ben = person(&relay, "Ben", &[4.0]);
    let empty =
        "<?xml version=\"1.0\"?><rss version=\"2.0\"><channel><title>f</title></channel></rss>";
    for _ in 0..6 {
        ben.http.push_for("feed.example", 200, empty);
    }
    room_with_ben_missing(&ann, &ben);
    room(ben.core.call(Call::RoomGetMedia)).unwrap();
    common::wait_for(
        &ben.c,
        |e| {
            e.plain_words
                .as_ref()
                .is_some_and(|w| w.id == "core-room-media-not-found")
        },
        WAIT,
    );
    let now = room(ben.core.call(Call::RoomStatus)).unwrap();
    assert_eq!(now.fetch, RoomFetch::Failed);
    assert!(now.can_fetch, "Get media can be pressed again");
    assert!(added_urls(&ben).is_empty());
}

#[test]
fn the_host_cannot_ask_itself_and_a_member_with_the_file_cannot_ask() {
    let relay = relay();
    let ann = person(&relay, "Ann", &[5.0]);
    let ben = person(&relay, "Ben", &[5.0]);
    let ann_s = open(&ann, ann.files[0]);
    let made = room(ann.core.call(Call::RoomCreate { session: ann_s })).unwrap();
    assert!(!made.can_fetch);
    room(ben.core.call(Call::RoomJoin { code: made.code })).unwrap();
    common::wait_for(
        &ben.c,
        |e| matches!(&e.body, EventBody::RoomEpisode { file: Some(_), .. }),
        WAIT,
    );
    for p in [&ann, &ben] {
        match p.core.call(Call::RoomGetMedia) {
            Err(CoreError::Service { code, .. }) => assert_eq!(code, "cannot_fetch"),
            other => panic!("{other:?}"),
        }
    }
}
