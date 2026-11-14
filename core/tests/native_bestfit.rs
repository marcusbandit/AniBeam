//! The desktop's native engine ranks best fit by what the person saved in anirss's
//! `config.toml` (#545), so a search there picks the release anirss would. The config is
//! the one in the home the core's root gives anirss.
#![cfg(unix)]

mod common;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;

const FEED_URL: &str = "https://feed.example/rss?q={query}";

fn feed(titles: &[&str]) -> String {
    let items: String = titles
        .iter()
        .enumerate()
        .map(|(n, t)| {
            format!(
                "<item><title>{t}</title><link>magnet:?xt=urn:btih:{n:040}</link>\
                 <torznab:attr name=\"seeders\" value=\"10\"/>\
                 <torznab:attr name=\"peers\" value=\"10\"/>\
                 <torznab:attr name=\"size\" value=\"1503238553\"/></item>"
            )
        })
        .collect();
    format!(
        "<?xml version=\"1.0\"?><rss version=\"2.0\" xmlns:torznab=\"http://torznab.com/schemas/2015/feed\">\
         <channel><title>feed</title>{items}</channel></rss>"
    )
}

/// Anibeam runs anirss itself (`mode` Sidecar), so its config is the one on this machine.
/// The native engine never starts it.
fn settings(mode: AnirssMode) -> DownloadSettings {
    DownloadSettings {
        anirss_mode: mode,
        anirss_command: "anirss".to_string(),
        jackett_command: String::new(),
        anirss_url: "http://anirss.test".to_string(),
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
        engine: DownloadsEngine::Native,
    }
}

fn best_fit_of(core: &Core, c: &anibeam_core::events::Collector) -> BestFit {
    let job = match core
        .call(Call::SearchReleases {
            query: "Show".into(),
            endpoint: None,
        })
        .unwrap()
    {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    match common::wait_job(c, job).body {
        EventBody::ReleasesFound { search } => search.best_fit,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_search_on_the_native_engine_picks_by_the_groups_saved_in_anirss_s_config() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    let config = dir.path().join("state/home/.config");
    std::fs::create_dir_all(config.join("anirss")).unwrap();
    // The group the core trusts out of the box is not the one the person saved.
    std::fs::write(
        config.join("anirss").join("config.toml"),
        "[bestfit]\npreferred_groups = [\"Beta\", \"Alpha\"]\n",
    )
    .unwrap();
    let reply = core
        .call(Call::SetDownloadSettings {
            settings: settings(AnirssMode::Sidecar),
            anirss_token: Some("tok".to_string()),
            torrent_password: None,
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
    core.call(Call::AddCustomEndpoint {
        name: "feed".into(),
        url: FEED_URL.into(),
        credentials: vec![],
    })
    .unwrap();
    let titles = [
        "[Alpha] Show - 01 (1080p).mkv",
        "[Beta] Show - 01 (1080p).mkv",
    ];
    for _ in 0..6 {
        http.push_for("feed.example", 200, feed(&titles));
    }
    let best = best_fit_of(&core, &c);
    assert!(best.available, "{best:?}");
    assert_eq!(
        best.item.map(|i| i.title).as_deref(),
        Some("[Beta] Show - 01 (1080p).mkv")
    );
    assert!(best.query.contains("[Beta]"), "{}", best.query);

    // An anirss the person runs elsewhere keeps its config there: this machine's file is not
    // its preferences, so the core ranks as it does out of the box (neither group trusted,
    // the first of equals).
    core.call(Call::SetDownloadSettings {
        settings: settings(AnirssMode::Remote),
        anirss_token: Some("tok".to_string()),
        torrent_password: None,
    })
    .unwrap();
    let best = best_fit_of(&core, &c);
    assert_eq!(
        best.item.map(|i| i.title).as_deref(),
        Some("[Alpha] Show - 01 (1080p).mkv")
    );
}
