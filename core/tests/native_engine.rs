//! The desktop's search, refine and subscription preview on the core's own
//! engine (#119), against `FakeHttp`: the setting that picks it and reads
//! anything it does not know as anirss, the three calls answered without
//! anirss, and one feed through both engines giving the same releases and
//! groups.

mod common;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;
use serde_json::{Value, json};

/// anirss reached as a remote server, so a request to it would show in
/// `FakeHttp` rather than start a process.
fn settings(engine: DownloadsEngine) -> DownloadSettings {
    DownloadSettings {
        anirss_mode: AnirssMode::Remote,
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
        engine,
    }
}

fn save(core: &Core, settings: DownloadSettings) {
    let reply = core
        .call(Call::SetDownloadSettings {
            settings,
            anirss_token: Some("tok".to_string()),
            torrent_password: None,
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

fn stored_engine(core: &Core) -> DownloadsEngine {
    match core.call(Call::GetDownloads).unwrap() {
        Reply::Downloads { status } => status.settings.engine,
        other => panic!("{other:?}"),
    }
}

// The setting ------------------------------------------------------------------------------

#[test]
fn settings_saved_before_the_engine_existed_read_as_anirss() {
    let mut blob = serde_json::to_value(settings(DownloadsEngine::Native)).unwrap();
    blob.as_object_mut().unwrap().remove("engine");
    let read: DownloadSettings = serde_json::from_value(blob).unwrap();
    assert_eq!(read.engine, DownloadsEngine::Anirss);
}

#[test]
fn an_unknown_engine_reads_as_anirss_and_keeps_the_rest() {
    let mut blob = serde_json::to_value(settings(DownloadsEngine::Native)).unwrap();
    blob["engine"] = json!("Telepathy");
    blob["prune_finished_feeds"] = json!(false);
    let read: DownloadSettings = serde_json::from_value(blob).unwrap();
    assert_eq!(read.engine, DownloadsEngine::Anirss);
    assert!(!read.prune_finished_feeds);
}

#[test]
fn the_native_engine_round_trips_through_the_settings() {
    let (_dir, core, c) = common::open_core();
    assert_eq!(stored_engine(&core), DownloadsEngine::Anirss);
    save(&core, settings(DownloadsEngine::Native));
    assert_eq!(stored_engine(&core), DownloadsEngine::Native);
    let json = serde_json::to_value(settings(DownloadsEngine::Native)).unwrap();
    assert_eq!(json["engine"], json!("Native"));
    // The switch is a state change, so it reaches the activity log.
    common::wait_for(
        &c,
        |e| e.level == Level::Info && e.message.contains("AniBeam's own engine"),
        std::time::Duration::from_secs(5),
    );
    save(&core, settings(DownloadsEngine::Anirss));
    assert_eq!(stored_engine(&core), DownloadsEngine::Anirss);
}

// One feed ---------------------------------------------------------------------------------

const FEED_URL: &str = "https://feed.example/rss?q={query}";

/// Two posters, two episodes each, every one at 1080p and with its counts,
/// so nothing is fetched for seeders.
const TITLES: [&str; 4] = [
    "[SubsPlease] Sousou no Frieren - 01 (1080p) [ABCDEF12].mkv",
    "[SubsPlease] Sousou no Frieren - 02 (1080p) [ABCDEF13].mkv",
    "[Erai-raws] Sousou no Frieren - 01 [1080p][Multiple Subtitle]",
    "[Erai-raws] Sousou no Frieren - 02 [1080p][Multiple Subtitle]",
];

fn link(n: usize) -> String {
    format!("magnet:?xt=urn:btih:{n:040}")
}

fn feed() -> String {
    let items: String = TITLES
        .iter()
        .enumerate()
        .map(|(n, t)| {
            format!(
                "<item><title>{t}</title><link>{}</link>\
                 <nyaa:seeders>{}</nyaa:seeders><nyaa:leechers>1</nyaa:leechers>\
                 <nyaa:size>1.4 GiB</nyaa:size></item>",
                link(n).replace('&', "&amp;"),
                10 + n
            )
        })
        .collect();
    format!(
        "<?xml version=\"1.0\"?><rss version=\"2.0\" xmlns:nyaa=\"https://nyaa.si/xmlns/nyaa\">\
         <channel><title>feed</title>{items}</channel></rss>"
    )
}

/// The core's engine on one endpoint added by hand, serving `feed()`.
fn native_core() -> (
    common::Dir,
    std::sync::Arc<Core>,
    std::sync::Arc<anibeam_core::events::Collector>,
    std::sync::Arc<FakeHttp>,
) {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    save(&core, settings(DownloadsEngine::Native));
    core.call(Call::AddCustomEndpoint {
        name: "feed".into(),
        url: FEED_URL.into(),
        credentials: vec![],
    })
    .unwrap();
    for _ in 0..3 {
        http.push_for("feed.example", 200, feed());
    }
    (dir, core, c, http)
}

fn search(core: &Core, c: &anibeam_core::events::Collector, query: &str) -> ReleaseSearch {
    let job = started(
        core,
        Call::SearchReleases {
            query: query.into(),
            endpoint: None,
        },
    );
    let done = common::wait_job(c, job);
    match done.body {
        EventBody::ReleasesFound { search } => search,
        other => panic!("{other:?}"),
    }
}

fn titles(items: &[Release]) -> Vec<&str> {
    items.iter().map(|i| i.title.as_str()).collect()
}

/// Every request went to the feed: anirss was never asked, and no sidecar
/// or torrent client was started for it.
fn only_the_feed(http: &FakeHttp) {
    let sent: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
    assert!(!sent.is_empty());
    assert!(
        sent.iter().all(|u| u.starts_with("https://feed.example/")),
        "{sent:?}"
    );
}

#[test]
fn the_native_engine_searches_refines_and_previews_without_anirss() {
    let (_dir, core, c, http) = native_core();

    let found = search(&core, &c, "Frieren");
    assert_eq!(found.endpoint, "feed");
    assert_eq!(titles(&found.items), TITLES);
    assert_eq!(found.query, "Frieren 1080p");
    assert_eq!(found.fetched_query, "Frieren");
    assert_eq!(found.auto_resolution.as_deref(), Some("1080p"));
    assert!(found.client_side);
    assert!(!found.fell_back);
    assert_eq!(found.tried.len(), 1);
    assert_eq!(found.tried[0].status, "ok");
    assert_eq!(found.facts.len(), 4);
    assert_eq!(found.facts[1].episode, Some(2.0));
    assert!(
        http.requests()[0].url.contains("q=Frieren"),
        "{:?}",
        http.requests()[0].url
    );

    let job = started(
        &core,
        Call::RefineReleases {
            query: found.query.clone(),
            endpoint: found.endpoint.clone(),
            items: found.items.clone(),
            op: RefineOp::Tokens {
                tokens: vec!["[SubsPlease]".into()],
            },
        },
    );
    let done = common::wait_job(&c, job);
    let EventBody::ReleasesRefined { refinement } = done.body else {
        panic!("{:?}", done.body)
    };
    assert!(refinement.applied);
    assert_eq!(titles(&refinement.items), &TITLES[..2]);
    assert_eq!(refinement.previous_count, 4);
    assert_eq!(refinement.delta, -2);
    assert_eq!(refinement.facts.len(), 2);

    let job = started(
        &core,
        Call::PreviewSubscription {
            query: refinement.query.clone(),
            endpoint: refinement.endpoint.clone(),
            items: refinement.items.clone(),
        },
    );
    let done = common::wait_job(&c, job);
    let EventBody::SubscriptionPreviewed { spec } = done.body else {
        panic!("{:?}", done.body)
    };
    assert_eq!(spec.endpoint_name, "feed");
    assert!(
        spec.feed_url.starts_with("https://feed.example/rss?q="),
        "{}",
        spec.feed_url
    );

    only_the_feed(&http);
    match core.call(Call::GetDownloads).unwrap() {
        Reply::Downloads { status } => assert_ne!(status.anirss.state, AnirssState::Ready),
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_endpoint_the_core_does_not_hold_is_refused_on_the_native_engine() {
    let (_dir, core, c, http) = native_core();
    let job = started(
        &core,
        Call::SearchReleases {
            query: "Frieren".into(),
            endpoint: Some("1337x".into()),
        },
    );
    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::JobFailed { .. }),
        "{:?}",
        done.body
    );
    assert!(http.requests().is_empty(), "{:?}", http.requests());
}

/// The same four releases as anirss's feed read gives them.
fn release_json(n: usize) -> Value {
    json!({
        "title": TITLES[n],
        "link": link(n),
        "seeders": 10 + n,
        "leechers": 1,
        "size": "1.4 GiB",
        "source": "feed",
        "resolution": "1080p"
    })
}

/// The feed through the fake anirss and through the core's engine: the
/// same releases in the same order, the same auto-resolution and the same
/// groups. The anirss side's replies are written out by hand from anirss's
/// rules (`titles.title_tokens`, `refinement.compute_groups` and
/// `auto_resolution`): the poster is a token of its own, a token every
/// release shares is no group, and releases with the same tokens share one
/// group, labelled poster first, then longest. Best fit is left out: it
/// follows each side's own trusted groups.
#[test]
fn the_same_feed_gives_the_same_releases_and_groups_on_either_engine() {
    let (_ndir, ncore, nc, _nhttp) = native_core();
    let native = search(&ncore, &nc, "Frieren");

    let http = FakeHttp::new();
    let (_adir, acore, ac) = common::open_core_with_http(http.clone());
    save(&acore, settings(DownloadsEngine::Anirss));
    let items: Vec<Value> = (0..4).map(release_json).collect();
    http.push_for(
        "/search",
        200,
        json!({
            "query": "Frieren",
            "endpoint": "feed",
            "fell_back": false,
            "tried": [{"endpoint": "feed", "status": "ok", "count": 4, "error": null}],
            "items": items
        })
        .to_string(),
    );
    http.push_for(
        "/refine/analyze",
        200,
        json!({
            "query": "Frieren 1080p",
            "client_side": true,
            "auto_resolution": {"applied": true, "token": "1080p"},
            "items": items,
            "groups": [
                {"label": "[Erai-raws]+Multiple+Subtitle",
                 "tokens": ["[Erai-raws]", "Multiple", "Subtitle"],
                 "member_count": 2, "has_poster": true, "members": [2, 3]},
                {"label": "[SubsPlease]", "tokens": ["[SubsPlease]"],
                 "member_count": 2, "has_poster": true, "members": [0, 1]}
            ],
            "best_fit": {"available": false, "query": "", "item": null, "profile": []}
        })
        .to_string(),
    );
    let anirss = search(&acore, &ac, "Frieren");
    assert!(
        http.requests()[0].url.starts_with("http://anirss.test/"),
        "{:?}",
        http.requests()[0].url
    );

    assert_eq!(titles(&native.items), titles(&anirss.items));
    assert_eq!(native.endpoint, anirss.endpoint);
    assert_eq!(native.query, anirss.query);
    assert_eq!(native.fetched_query, anirss.fetched_query);
    assert_eq!(native.auto_resolution, anirss.auto_resolution);
    assert_eq!(native.client_side, anirss.client_side);
    assert_eq!(native.tried, anirss.tried);
    assert_eq!(native.fell_back, anirss.fell_back);
    assert_eq!(native.groups, anirss.groups);
    assert_eq!(native.facts, anirss.facts);
}
