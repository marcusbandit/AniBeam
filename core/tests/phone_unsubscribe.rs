//! Stopping a phone subscription with its files (#654): only the torrents the
//! subscription added go, and a download got by hand into the show's folder
//! stays. The engine is the phone's own, its torrents made here and their
//! files already whole in the folder, so nothing touches the network.
#![cfg(feature = "phone")]

mod common;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anibeam_core::events::Collector;
use anibeam_core::net::FakeHttp;
use anibeam_core::trackers::Secrets;
use anibeam_core::*;

/// A single-file torrent of `bytes`, made in `made` and also put whole in
/// `folder`, as an episode already downloaded there.
fn episode(
    rt: &tokio::runtime::Runtime,
    made: &Path,
    folder: &Path,
    name: &str,
    byte: u8,
) -> Vec<u8> {
    let file = made.join(name);
    std::fs::write(&file, vec![byte; 50_000]).unwrap();
    std::fs::copy(&file, folder.join(name)).unwrap();
    rt.block_on(async {
        librqbit::create_torrent(
            &file,
            librqbit::CreateTorrentOptions {
                name: None,
                trackers: Vec::new(),
                piece_length: Some(16_384),
            },
            &librqbit::spawn_utils::BlockingSpawner::new(1),
        )
        .await
        .unwrap()
        .as_bytes()
        .unwrap()
        .to_vec()
    })
}

fn release(title: &str, link: &str) -> Release {
    serde_json::from_value(serde_json::json!({ "title": title, "link": link })).unwrap()
}

fn torrents(core: &Core) -> Vec<Torrent> {
    match core.call(Call::PhoneTorrents).unwrap() {
        Reply::PhoneTorrents { torrents, .. } => torrents,
        other => panic!("{other:?}"),
    }
}

#[test]
fn unsubscribing_with_files_leaves_a_download_got_by_hand_in_its_folder() {
    let dir = tempfile::tempdir().unwrap();
    // The phone's layout: the library sits beside the core's own folders.
    let paths = CorePaths::under(&dir.path().join("files").join("core"));
    let show = dir.path().join("files").join("Anime").join("Show");
    let made = dir.path().join("made");
    std::fs::create_dir_all(&show).unwrap();
    std::fs::create_dir_all(&made).unwrap();
    std::fs::create_dir_all(&paths.config_dir).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let by_hand = "[G] Show - 01 (1080p).mkv";
    let followed = "[G] Show - 02 (1080p).mkv";
    let hand_torrent = episode(&rt, &made, &show, by_hand, 1);
    let followed_torrent = episode(&rt, &made, &show, followed, 2);

    // The subscription saves into the show's folder; its feed has episode 2.
    std::fs::write(
        Path::new(&paths.config_dir).join("phone-rules.json"),
        serde_json::to_vec(&serde_json::json!([{
            "name": "Show",
            "feed_url": "https://feed.test/rss?q=Show",
            "save_path": show.to_string_lossy(),
        }]))
        .unwrap(),
    )
    .unwrap();
    let http = FakeHttp::new();
    http.push_for(
        "feed.test/rss",
        200,
        "<rss version=\"2.0\"><channel><title>Show</title>\
         <item><title>[G] Show - 02 (1080p)</title>\
         <link>https://dl.test/02.torrent</link></item></channel></rss>",
    );
    http.push_for("dl.test/02.torrent", 200, followed_torrent);
    http.push_for("dl.test/01.torrent", 200, hand_torrent);

    let secrets = Secrets::file_only(paths.secrets_path());
    let core = Core::open_with_http_and_secrets(paths, http, secrets).unwrap();
    let c = Arc::new(Collector::default());
    let _sub = core.subscribe(c.clone());

    // Episode 1 got by hand into the same folder; opening the engine for it starts the
    // subscription's first round, which takes episode 2.
    core.call(Call::PhoneDownload {
        items: vec![release(
            "[G] Show - 01 (1080p)",
            "https://dl.test/01.torrent",
        )],
        name: Some("Show".to_string()),
        limit_bytes: None,
        rolling: Some(false),
    })
    .unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::Notice) && e.message.contains("[G] Show - 02"),
        Duration::from_secs(30),
    );
    let before = torrents(&core);
    assert_eq!(before.len(), 2, "{before:?}");
    let hand_hash = before
        .iter()
        .find(|t| t.name.contains("- 01"))
        .map(|t| t.hash.clone())
        .unwrap();

    core.call(Call::PhoneUnsubscribe {
        names: vec!["Show".to_string()],
        files: true,
    })
    .unwrap();

    let after: Vec<String> = torrents(&core).into_iter().map(|t| t.hash).collect();
    assert_eq!(after, [hand_hash], "only the subscription's torrent goes");
    assert!(
        show.join(by_hand).exists(),
        "the hand download's file stays"
    );
    assert!(
        !show.join(followed).exists(),
        "the subscription's episode goes with it"
    );
}
