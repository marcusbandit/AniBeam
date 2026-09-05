//! The watching list: the cached list a page paints at once, the refresh
//! that runs behind it, and what each of them keeps. AniList is connected
//! through the fixtures rather than through a flow, so these tests are
//! about the list alone.

mod common;
mod fixtures;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;

/// A one pixel JPEG, so the cover fetches land a real file the read can
/// hand back as a poster.
const JPEG: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xE0];

fn watching(core: &Core) -> (WatchingList, Option<u64>) {
    match core.call(Call::ListWatching).unwrap() {
        Reply::Watching { list, refreshing } => (list, refreshing),
        other => panic!("{other:?}"),
    }
}

/// AniList's answer, spread over two custom lists: three entries the page
/// wants, one it does not, and the first entry again under a second list
/// with different numbers, which is how AniList reports a media that sits
/// on two lists at once.
fn reply(frieren_airs: i64, one_piece_airs: i64) -> serde_json::Value {
    serde_json::json!({ "data": { "MediaListCollection": { "lists": [
        { "entries": [
            {
                "progress": 5, "status": "CURRENT", "score": 8.5, "updatedAt": 2000,
                "media": {
                    "id": 154587, "idMal": 52991, "siteUrl": "https://anilist.co/anime/154587",
                    "episodes": 30, "averageScore": 91,
                    "title": { "romaji": "Frieren", "english": "Frieren: Beyond Journey's End" },
                    "coverImage": { "large": "https://img/frieren.jpg" },
                    "nextAiringEpisode": { "episode": 29, "airingAt": frieren_airs }
                }
            },
            {
                "progress": 1100, "status": "REPEATING", "score": 0, "updatedAt": 1000,
                "media": {
                    "id": 21, "idMal": 21, "siteUrl": "https://anilist.co/anime/21",
                    "episodes": 1122, "averageScore": 88,
                    "title": { "romaji": "One Piece", "english": null },
                    "coverImage": { "large": "https://img/onepiece.jpg" },
                    "nextAiringEpisode": { "episode": 1124, "airingAt": one_piece_airs }
                }
            },
            {
                "progress": 3, "status": "CURRENT", "score": null, "updatedAt": 3000,
                "media": {
                    "id": 1, "idMal": 1, "siteUrl": "https://anilist.co/anime/1",
                    "episodes": 26, "averageScore": 86,
                    "title": { "romaji": "Cowboy Bebop", "english": null },
                    "coverImage": { "large": "https://img/bebop.jpg" },
                    "nextAiringEpisode": null
                }
            },
            {
                "progress": 64, "status": "COMPLETED", "score": 9.0, "updatedAt": 4000,
                "media": {
                    "id": 5114, "siteUrl": "https://anilist.co/anime/5114", "episodes": 64, "averageScore": 90,
                    "title": { "romaji": "Fullmetal Alchemist: Brotherhood", "english": null },
                    "coverImage": { "large": "https://img/fmab.jpg" },
                    "nextAiringEpisode": null
                }
            }
        ] },
        { "entries": [
            {
                "progress": 99, "status": "CURRENT", "score": 1.0, "updatedAt": 9999,
                "media": {
                    "id": 154587, "idMal": 52991, "siteUrl": "https://anilist.co/anime/154587",
                    "episodes": 30, "averageScore": 91,
                    "title": { "romaji": "Frieren", "english": null },
                    "coverImage": { "large": "https://img/frieren.jpg" },
                    "nextAiringEpisode": null
                }
            }
        ] }
    ] } } })
}

/// One media the library owns, one it owns and hides, and the episode row
/// a full fetch would already have left behind for the owned one.
fn library(core: &Core) -> u64 {
    let src = fixtures::insert_source(core, "/lib");
    let frieren = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Frieren", "Sousou no Frieren");
    fixtures::insert_media(core, 154587, Some("Sousou no Frieren"), None, Some(28), "RELEASING", "TV", Some(91));
    fixtures::match_series(core, frieren, Some(154587), Some(52991));
    fixtures::insert_episode(core, 154587, 29, Some("Aureole"), None);

    let bebop = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    fixtures::insert_media(core, 1, Some("Cowboy Bebop"), None, Some(26), "FINISHED", "TV", Some(86));
    fixtures::match_series(core, bebop, Some(1), Some(1));
    core.call(Call::SetHidden { series: bebop, hidden: true }).unwrap();
    frieren
}

#[test]
fn the_page_paints_the_cache_and_the_refresh_fills_it_behind() {
    let now = anibeam_core::time::now_secs();
    let (frieren_airs, one_piece_airs) = (now + 86_400, now + 3_600);
    let http = FakeHttp::new();
    http.push_json(200, reply(frieren_airs, one_piece_airs));
    for cover in ["frieren.jpg", "onepiece.jpg", "bebop.jpg"] {
        http.push_for(cover, 200, JPEG.to_vec());
    }
    let (dir, core, c) = common::open_core_with_http(http.clone());
    let frieren = library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    // Nothing cached yet, so the page paints an empty list and the refresh
    // it started is what fills it.
    let (empty, refreshing) = watching(&core);
    assert!(empty.entries.is_empty());
    assert_eq!(empty.fetched_at, None);
    let job = refreshing.expect("a connected AniList always starts a refresh");
    let done = common::wait_job(&c, job);

    let query = &http.requests()[0];
    assert_eq!(query.url, "https://graphql.anilist.co");
    assert!(query.headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer tok"), "{query:?}");
    let sent = format!("{:?}", query.body);
    assert!(sent.contains("userId") && sent.contains("42"), "{sent}");
    assert!(sent.contains("nextAiringEpisode"), "{sent}");

    // The terminal event carries the same list the page reads back.
    let EventBody::WatchingRefreshed { list } = done.body.clone() else { panic!("{done:?}") };
    assert_eq!(done.level, Level::Debug);
    assert_eq!(done.message, "watching list refreshed: 2 entries");
    let (cached, again) = watching(&core);
    assert_eq!(cached.entries, list.entries);
    assert!(cached.fetched_at.is_some());
    assert!(again.is_some());

    // Two of the three watched entries: the third is owned and hidden, and
    // the completed one was never on this page at all.
    assert_eq!(list.entries.len(), 2, "{:#?}", list.entries);
    let frieren_entry = &list.entries[0];
    let one_piece = &list.entries[1];
    assert_eq!(frieren_entry.anilist_id, 154587);
    assert_eq!(one_piece.anilist_id, 21);

    // The stored row wins over the list's thinner copy of it, and the
    // duplicate under the second list never displaces the first.
    assert_eq!(frieren_entry.title, "Sousou no Frieren");
    assert_eq!(frieren_entry.total, Some(28));
    assert_eq!(frieren_entry.progress, 5);
    assert_eq!(frieren_entry.score, Some(8.5));
    assert!(!frieren_entry.repeating);
    assert_eq!(frieren_entry.updated_at, anibeam_core::time::from_secs(2000));
    assert_eq!(frieren_entry.owned, Some(frieren));
    assert_eq!(frieren_entry.site_url.as_deref(), Some("https://anilist.co/anime/154587"));
    assert_eq!(frieren_entry.next_airing, Some(Airing { episode: 29, at: anibeam_core::time::from_secs(frieren_airs) }));

    // A media the store had never heard of arrives as a stub, and a
    // rewatch is on the page like anything else being watched.
    assert_eq!(one_piece.title, "One Piece");
    assert_eq!(one_piece.total, Some(1122));
    assert_eq!(one_piece.progress, 1100);
    // Nought is AniList's unrated rather than a rating.
    assert_eq!(one_piece.score, None);
    assert!(one_piece.repeating);
    assert_eq!(one_piece.owned, None);
    assert_eq!(one_piece.site_url.as_deref(), Some("https://anilist.co/anime/21"));
    assert_eq!(one_piece.next_airing, Some(Airing { episode: 1124, at: anibeam_core::time::from_secs(one_piece_airs) }));

    // Every cover was fetched before the job reported, so a poster is a
    // local file rather than a url the shell has to go and get.
    let images = dir.path().join("cache").join("images");
    for entry in &list.entries {
        let poster = entry.poster.clone().unwrap_or_else(|| panic!("no poster on {}", entry.title));
        assert!(poster.starts_with(images.to_str().unwrap()), "{poster}");
        assert!(std::path::Path::new(&poster).exists(), "{poster}");
    }
    let urls: Vec<String> = http.requests().iter().map(|r| r.url.clone()).collect();
    for cover in ["https://img/frieren.jpg", "https://img/onepiece.jpg", "https://img/bebop.jpg"] {
        assert!(urls.iter().any(|u| u == cover), "{urls:?}");
    }
    // The completed entry never became a row, so its cover was never asked
    // for either.
    assert!(!urls.iter().any(|u| u.contains("fmab")), "{urls:?}");

    // What the refresh wrote under the entries: a stub that says it was
    // never fetched, and an episode row whose stored title survived a
    // schedule that carries none.
    let (stub_score, stub_fetched, episode_title, episode_at) = core
        .store()
        .write(|conn| {
            let stub: (Option<i64>, Option<i64>) =
                conn.query_row("SELECT average_score, fetched_at FROM anilist_media WHERE id = 21", [], |r| Ok((r.get(0)?, r.get(1)?)))?;
            let episode: (Option<String>, Option<i64>) = conn.query_row(
                "SELECT title, aired_at FROM anilist_episodes WHERE anilist_id = 154587 AND number = 29",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            Ok((stub.0, stub.1, episode.0, episode.1))
        })
        .unwrap();
    assert_eq!(stub_score, Some(88));
    assert_eq!(stub_fetched, None);
    assert_eq!(episode_title.as_deref(), Some("Aureole"));
    assert_eq!(episode_at, Some(frieren_airs));
}

/// A refresh that fails says so in one sanitised line and changes nothing:
/// yesterday's list reads better than no list at all.
#[test]
fn a_failed_refresh_leaves_the_cached_list_standing() {
    let now = anibeam_core::time::now_secs();
    let http = FakeHttp::new();
    http.push_json(200, reply(now + 86_400, now + 3_600));
    for cover in ["frieren.jpg", "onepiece.jpg", "bebop.jpg"] {
        http.push_for(cover, 200, JPEG.to_vec());
    }
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    library(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");

    let (_, refreshing) = watching(&core);
    common::wait_job(&c, refreshing.unwrap());

    http.fail_next("connection refused");
    let (cached, refreshing) = watching(&core);
    assert_eq!(cached.entries.len(), 2);
    let failed = common::wait_job(&c, refreshing.unwrap());
    match &failed.body {
        EventBody::JobFailed { error: CoreError::Provider { provider: Provider::Anilist, message, .. } } => {
            assert_eq!(message, "connection refused");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(watching(&core).0.entries.len(), 2);
}

/// No AniList, no refresh: the page reads what it has and is told nothing
/// is on its way.
#[test]
fn a_disconnected_anilist_starts_no_refresh() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    library(&core);

    let (list, refreshing) = watching(&core);
    assert!(list.entries.is_empty());
    assert_eq!(list.fetched_at, None);
    assert_eq!(refreshing, None);
    assert!(http.requests().is_empty());
}
