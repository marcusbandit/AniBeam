//! Stopping a phone subscription with its files (#654): only the torrents the
//! subscription added go, and a download got by hand into the show's folder
//! stays. A subscription saved before it noted what it added removes nothing,
//! and its answer says so. The engine is the phone's own, its torrents made
//! here and their files already whole in the folder, so nothing touches the
//! network.
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

fn hash_of(torrents: &[Torrent], episode: &str) -> String {
    torrents
        .iter()
        .find(|t| t.name.contains(episode))
        .map(|t| t.hash.clone())
        .unwrap()
}

fn sorted(mut hashes: Vec<String>) -> Vec<String> {
    hashes.sort();
    hashes
}

/// Gets `title` by hand into the library folder `name`, as Get does.
fn by_hand(core: &Core, title: &str, link: &str, name: &str) {
    core.call(Call::PhoneDownload {
        items: vec![release(title, link)],
        name: Some(name.to_string()),
        limit_bytes: None,
        rolling: Some(false),
    })
    .unwrap();
}

fn unsubscribe(core: &Core, name: &str) -> u64 {
    match core
        .call(Call::PhoneUnsubscribe {
            names: vec![name.to_string()],
            files: true,
        })
        .unwrap()
    {
        Reply::PhoneUnsubscribed { removed } => removed,
        other => panic!("{other:?}"),
    }
}

#[test]
fn unsubscribing_with_files_leaves_a_download_got_by_hand_in_its_folder() {
    let dir = tempfile::tempdir().unwrap();
    // The phone's layout: the library sits beside the core's own folders.
    let paths = CorePaths::under(&dir.path().join("files").join("core"));
    let anime = dir.path().join("files").join("Anime");
    let show = anime.join("Show");
    let old = anime.join("Old");
    let made = dir.path().join("made");
    for folder in [&show, &old, &made] {
        std::fs::create_dir_all(folder).unwrap();
    }
    std::fs::create_dir_all(&paths.config_dir).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let got_by_hand = "[G] Show - 01 (1080p).mkv";
    let followed = "[G] Show - 02 (1080p).mkv";
    let older = "[G] Old - 03 (1080p).mkv";
    let hand_torrent = episode(&rt, &made, &show, got_by_hand, 1);
    let followed_torrent = episode(&rt, &made, &show, followed, 2);
    let older_torrent = episode(&rt, &made, &old, older, 3);

    // "Show" saves into its folder and its feed has episode 2. "Old" was saved before
    // subscriptions noted what they added: it took episode 3 and has no record of it.
    std::fs::write(
        Path::new(&paths.config_dir).join("phone-rules.json"),
        serde_json::to_vec(&serde_json::json!([
            {
                "name": "Show",
                "feed_url": "https://feed.test/rss?q=Show",
                "save_path": show.to_string_lossy(),
            },
            {
                "name": "Old",
                "feed_url": "https://feed.test/rss?q=Old",
                "save_path": old.to_string_lossy(),
                "taken": ["https://dl.test/03.torrent"],
            },
        ]))
        .unwrap(),
    )
    .unwrap();
    let http = FakeHttp::new();
    http.push_for(
        "feed.test/rss?q=Show",
        200,
        "<rss version=\"2.0\"><channel><title>Show</title>\
         <item><title>[G] Show - 02 (1080p)</title>\
         <link>https://dl.test/02.torrent</link></item></channel></rss>",
    );
    http.push_for("dl.test/02.torrent", 200, followed_torrent);
    http.push_for("dl.test/01.torrent", 200, hand_torrent);
    http.push_for("dl.test/03.torrent", 200, older_torrent);

    let secrets = Secrets::file_only(paths.secrets_path());
    let core = Core::open_with_http_and_secrets(paths, http, secrets).unwrap();
    let c = Arc::new(Collector::default());
    let _sub = core.subscribe(c.clone());

    // Episode 1 got by hand into Show's folder; opening the engine for it starts the
    // subscriptions' first round, which takes episode 2.
    by_hand(
        &core,
        "[G] Show - 01 (1080p)",
        "https://dl.test/01.torrent",
        "Show",
    );
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::Notice) && e.message.contains("[G] Show - 02"),
        Duration::from_secs(30),
    );
    // Episode 3 stands in for what Old downloaded before: in the engine, in its folder.
    by_hand(
        &core,
        "[G] Old - 03 (1080p)",
        "https://dl.test/03.torrent",
        "Old",
    );
    let before = torrents(&core);
    assert_eq!(before.len(), 3, "{before:?}");
    let hand_hash = hash_of(&before, "Show - 01");
    let old_hash = hash_of(&before, "Old - 03");

    // Old has nothing on record, so nothing goes, and the answer says none went.
    assert_eq!(
        unsubscribe(&core, "Old"),
        0,
        "an old subscription removes nothing"
    );
    assert_eq!(torrents(&core).len(), 3);
    assert!(
        old.join(older).exists(),
        "the old subscription's episode stays"
    );

    assert_eq!(
        unsubscribe(&core, "Show"),
        1,
        "the subscription's one torrent"
    );
    let after: Vec<String> = torrents(&core).into_iter().map(|t| t.hash).collect();
    assert_eq!(
        sorted(after),
        sorted(vec![hand_hash, old_hash]),
        "only the subscription's torrent goes"
    );
    assert!(
        show.join(got_by_hand).exists(),
        "the hand download's file stays"
    );
    assert!(
        !show.join(followed).exists(),
        "the subscription's episode goes with it"
    );
}
