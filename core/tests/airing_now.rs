//! Airing now (#202): every show on air, read once and kept, with the
//! shows marked not interested left out, one AniList entry at a time.

mod common;
mod fixtures;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;

const JPEG: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xE0];

fn airing(core: &Core, season: SeasonPick) -> (AiringNow, Option<u64>) {
    match core.call(Call::ListAiringNow { season }).unwrap() {
        Reply::AiringNow { now, refreshing } => (now, refreshing),
        other => panic!("{other:?}"),
    }
}

fn refresh(core: &Core, season: SeasonPick, force: bool) -> Option<u64> {
    match core.call(Call::RefreshAiringNow { season, force }).unwrap() {
        Reply::Started { job } => Some(job),
        Reply::Ok => None,
        other => panic!("{other:?}"),
    }
}

fn mark(core: &Core, anilist_id: u64, on: bool) {
    let reply = core
        .call(Call::SetNotInterested {
            anilist_id,
            not_interested: on,
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
}

fn ids(now: &AiringNow) -> Vec<u64> {
    now.shows.iter().map(|s| s.anilist_id).collect()
}

fn show(id: u64, romaji: &str, next: Option<(u32, i64)>) -> serde_json::Value {
    serde_json::json!({
        "id": id, "idMal": id + 1000, "siteUrl": format!("https://anilist.co/anime/{id}"),
        "format": "TV", "episodes": 12, "averageScore": 80,
        "title": { "romaji": romaji, "english": null, "native": null },
        "coverImage": { "large": format!("https://img/{id}.jpg") },
        "nextAiringEpisode": next.map(|(e, at)| serde_json::json!({ "episode": e, "airingAt": at })),
    })
}

/// Three shows on air over two pages, most popular first, and a strip of
/// three episodes: two from shows on the page, one from a show that is not
/// (a film, say), which the strip must leave out.
fn queue_now(http: &FakeHttp, now: i64) {
    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "pageInfo": { "hasNextPage": true }, "media": [
            show(235, "Detective Conan", Some((1180, now + 3 * 86_400))),
            show(154587, "Sousou no Frieren", Some((8, now + 6 * 3_600))),
        ] } } }),
    );
    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "pageInfo": { "hasNextPage": false }, "media": [
            show(171018, "Dandadan", Some((6, now + 5 * 86_400))),
            // AniList moved it a page down mid-read: listed once
            show(154587, "Sousou no Frieren", Some((8, now + 6 * 3_600))),
        ] } } }),
    );
    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "pageInfo": { "hasNextPage": false }, "airingSchedules": [
            { "episode": 7, "airingAt": now - 3_600, "mediaId": 154587 },
            { "episode": 5, "airingAt": now - 7_200, "mediaId": 171018 },
            { "episode": 1, "airingAt": now - 9_000, "mediaId": 99999 },
        ] } } }),
    );
    for id in [235, 154587, 171018] {
        http.push_for(&format!("{id}.jpg"), 200, JPEG.to_vec());
    }
}

#[test]
fn the_page_reads_the_cache_and_a_refresh_reads_every_show_on_air() {
    let now = anibeam_core::time::now_secs();
    let http = FakeHttp::new();
    queue_now(&http, now);
    let (_dir, core, c) = common::open_core_with_http(http.clone());

    let (empty, refreshing) = airing(&core, SeasonPick::Now);
    assert!(empty.shows.is_empty() && empty.aired.is_empty());
    assert_eq!(empty.label, "Airing now");
    assert_eq!(empty.fetched_at, None);
    assert_eq!(refreshing, None, "a read starts nothing");
    assert!(http.requests().is_empty(), "and asks AniList nothing");

    let job = refresh(&core, SeasonPick::Now, false).expect("never read is stale");
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::AiringNowRefreshed {
            season: SeasonPick::Now
        }
    );

    // Two season pages, no token, everything releasing; then the strip.
    let requests = http.requests();
    let first = format!("{:?}", requests[0].body);
    assert!(first.contains("RELEASING"), "{first}");
    assert!(
        !first.contains("seasonYear\":"),
        "no null filter is sent: {first}"
    );
    assert!(
        !requests[0]
            .headers
            .iter()
            .any(|(k, _)| k == "Authorization"),
        "a public read sends no token"
    );
    assert!(format!("{:?}", requests[2].body).contains("airingSchedules"));

    let (now_page, _) = airing(&core, SeasonPick::Now);
    assert_eq!(
        ids(&now_page),
        vec![235, 154587, 171018],
        "popularity order"
    );
    assert!(now_page.fetched_at.is_some());
    let frieren = &now_page.shows[1];
    assert_eq!(frieren.title, "Sousou no Frieren");
    assert_eq!(frieren.mal_id, Some(155587));
    assert_eq!(frieren.next_airing.as_ref().map(|a| a.episode), Some(8));
    assert!(frieren.poster.is_some(), "the cover is a local file");
    assert_eq!(frieren.latest.as_ref().map(|a| a.episode), Some(7));
    assert_eq!(frieren.owned, None);
    assert_eq!(frieren.list_status, None);

    // Newest first, and the show not on the page is not in the strip.
    let strip: Vec<(u64, u32)> = now_page
        .aired
        .iter()
        .map(|a| (a.anilist_id, a.episode))
        .collect();
    assert_eq!(strip, vec![(154587, 7), (171018, 5)]);
    assert_eq!(now_page.aired[0].title, "Sousou no Frieren");

    // Fresh: no second read unless forced.
    let asked = http.requests().len();
    assert_eq!(refresh(&core, SeasonPick::Now, false), None);
    assert_eq!(http.requests().len(), asked);
}

#[test]
fn not_interested_is_one_entry_and_nothing_related_to_it() {
    let now = anibeam_core::time::now_secs();
    let http = FakeHttp::new();
    queue_now(&http, now);
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    common::wait_job(&c, refresh(&core, SeasonPick::Now, true).unwrap());

    // Frieren's earlier season sits in the library's tables as a prequel of
    // the one on air. Marking that one must not touch this one, and the
    // other way round.
    fixtures::insert_media(
        &core,
        154588,
        Some("Sousou no Frieren 2"),
        None,
        Some(12),
        "NOT_YET_RELEASED",
        "TV",
        None,
    );
    fixtures::insert_media(
        &core,
        154587,
        Some("Sousou no Frieren"),
        None,
        Some(28),
        "RELEASING",
        "TV",
        Some(91),
    );
    core.store()
        .write(|c| {
            c.execute(
                "INSERT INTO relations (from_id, to_id, relation) VALUES (154588, 154587, 'PREQUEL'), (154587, 154588, 'SEQUEL')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    mark(&core, 154588, true);
    let (page, _) = airing(&core, SeasonPick::Now);
    assert_eq!(ids(&page), vec![235, 154587, 171018]);
    assert_eq!(page.not_interested, 0, "the marked show is not on air");

    mark(&core, 235, true);
    mark(&core, 235, true);
    let (page, _) = airing(&core, SeasonPick::Now);
    assert_eq!(ids(&page), vec![154587, 171018]);
    assert_eq!(page.not_interested, 1);

    mark(&core, 154587, true);
    let (page, _) = airing(&core, SeasonPick::Now);
    assert_eq!(ids(&page), vec![171018]);
    assert!(
        page.aired.iter().all(|a| a.anilist_id != 154587),
        "its episodes leave the strip too"
    );

    let Reply::NotInterested { shows } = core.call(Call::ListNotInterested).unwrap() else {
        panic!()
    };
    let named: Vec<(u64, &str)> = shows
        .iter()
        .map(|s| (s.anilist_id, s.title.as_str()))
        .collect();
    assert_eq!(named.len(), 3);
    assert!(named.contains(&(235, "Detective Conan")));
    assert!(named.contains(&(154588, "Sousou no Frieren 2")));

    // Undo brings it back where it was.
    mark(&core, 235, false);
    let (page, _) = airing(&core, SeasonPick::Now);
    assert_eq!(ids(&page), vec![235, 171018]);
}

#[test]
fn a_show_reads_owned_and_listed_and_a_hidden_series_is_left_out() {
    let now = anibeam_core::time::now_secs();
    let http = FakeHttp::new();
    queue_now(&http, now);
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    common::wait_job(&c, refresh(&core, SeasonPick::Now, true).unwrap());

    let src = fixtures::insert_source(&core, "/lib");
    let frieren = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Frieren", "Frieren");
    fixtures::insert_media(
        &core,
        154587,
        Some("Sousou no Frieren"),
        None,
        Some(28),
        "RELEASING",
        "TV",
        Some(91),
    );
    fixtures::match_series(&core, frieren, Some(154587), None);
    let conan = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Conan", "Conan");
    fixtures::insert_media(
        &core,
        235,
        Some("Detective Conan"),
        None,
        None,
        "RELEASING",
        "TV",
        None,
    );
    fixtures::match_series(&core, conan, Some(235), None);
    core.call(Call::SetHidden {
        series: conan,
        hidden: true,
    })
    .unwrap();
    // On MAL's list by MAL id, with MAL connected
    fixtures::connect_tracker(&core, Tracker::Mal, 7, "mal-token");
    fixtures::insert_tracker_entry(&core, "mal", 172018, 0, "planning", None);

    let (page, _) = airing(&core, SeasonPick::Now);
    assert_eq!(ids(&page), vec![154587, 171018]);
    assert_eq!(page.shows[0].owned, Some(frieren));
    assert_eq!(page.shows[1].list_status, Some(ListStatus::Planning));
}

#[test]
fn next_season_is_its_own_read_with_no_strip() {
    let http = FakeHttp::new();
    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "pageInfo": { "hasNextPage": false }, "media": [
            show(500, "Next Thing", None),
        ] } } }),
    );
    http.push_for("500.jpg", 200, JPEG.to_vec());
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    common::wait_job(&c, refresh(&core, SeasonPick::Next, false).unwrap());

    let requests = http.requests();
    let sent = format!("{:?}", requests[0].body);
    assert!(
        sent.contains("\"seasonYear\": Number") && !sent.contains("RELEASING"),
        "{sent}"
    );
    assert!(
        requests
            .iter()
            .all(|r| !format!("{:?}", r.body).contains("airingSchedules")),
        "the strip is what is on air"
    );
    let (page, _) = airing(&core, SeasonPick::Next);
    assert_eq!(ids(&page), vec![500]);
    assert_ne!(page.label, "Airing now");
    assert!(airing(&core, SeasonPick::Now).0.shows.is_empty());
}

#[test]
fn plan_to_watch_needs_a_tracker_and_never_moves_a_listed_show() {
    let now = anibeam_core::time::now_secs();
    let http = FakeHttp::new();
    queue_now(&http, now);
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    common::wait_job(&c, refresh(&core, SeasonPick::Now, true).unwrap());

    let err = core
        .call(Call::PlanToWatch { anilist_id: 154587 })
        .unwrap_err();
    assert!(matches!(err, CoreError::NotConnected { .. }), "{err:?}");

    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "tok");
    // Already on the list: left as it is, nothing asked.
    fixtures::insert_tracker_entry(&core, "anilist", 235, 900, "dropped", None);
    let asked = http.requests().len();
    assert_eq!(
        core.call(Call::PlanToWatch { anilist_id: 235 }).unwrap(),
        Reply::Ok
    );
    assert_eq!(http.requests().len(), asked);

    // Not on it: read live (AniList answers 404 for no entry), then saved.
    http.push_json(
        200,
        serde_json::json!({ "errors": [{ "status": 404, "message": "Not Found." }], "data": { "MediaList": null } }),
    );
    http.push_json(
        200,
        serde_json::json!({ "data": { "SaveMediaListEntry": { "id": 1, "status": "PLANNING", "updatedAt": 1234 } } }),
    );
    let Reply::Started { job } = core.call(Call::PlanToWatch { anilist_id: 154587 }).unwrap()
    else {
        panic!()
    };
    let done = common::wait_job(&c, job);
    assert_eq!(done.level, Level::Info, "{done:?}");
    assert!(
        done.message.contains("Planning on AniList"),
        "{}",
        done.message
    );
    let sent = format!("{:?}", http.requests().last().unwrap().body);
    assert!(
        sent.contains("PLANNING") && sent.contains("154587"),
        "{sent}"
    );

    let (page, _) = airing(&core, SeasonPick::Now);
    let frieren = page.shows.iter().find(|s| s.anilist_id == 154587).unwrap();
    assert_eq!(frieren.list_status, Some(ListStatus::Planning));
}
