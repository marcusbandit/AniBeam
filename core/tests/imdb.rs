//! The IMDb refresh: IMDb's own dump and Wikidata's links into the store
//! once a week at launch, and the rating each card resolves to from its
//! current match.

mod common;
mod fixtures;

use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use anibeam_core::events::Collector;
use anibeam_core::library::cards;
use anibeam_core::metadata::imdb;
use anibeam_core::net::FakeHttp;
use anibeam_core::*;
use flate2::Compression;
use flate2::write::GzEncoder;
use rusqlite::params;
use serde_json::json;

const DUMP_HOST: &str = "datasets.imdbws.com";
const WIKIDATA_HOST: &str = "query.wikidata.org";
const LAST_MODIFIED: &str = "Wed, 14 Oct 2026 00:44:30 GMT";
const WEEK: i64 = 7 * 24 * 60 * 60;

/// The dump's own rows the tests care about: tconst, rating, votes.
const RATED: [(&str, &str, u64); 8] = [
    ("tt0213338", "8.9", 174_384), // Cowboy Bebop, AniList 1
    ("tt35854736", "7.7", 1_200),  // the TVmaze show's own
    ("tt0000100", "6.0", 5),       // MAL 33's lesser title
    ("tt0000101", "8.1", 900),     // MAL 33's most voted
    ("tt0000200", "7.0", 10),      // MAL 22, AniList 2's fallback
    ("tt0000300", "5.0", 10),      // AniList 3's own link
    ("tt0000301", "9.5", 99_999),  // MAL 44, more votes but only a fallback
    ("tt0000500", "6.6", 66),      // MAL 55, behind a TMDB match
];

/// A real dump holds 1.7 million titles and the refresh refuses fewer than
/// 100,000, so every dump that is meant to land carries this much filler.
const FILLER: u64 = 100_000;

fn gz(text: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(text.as_bytes()).unwrap();
    encoder.finish().unwrap()
}

/// IMDb's header, `rows`, then `filler` titles nobody owns.
fn dump(rows: &[(&str, &str, u64)], filler: u64) -> Vec<u8> {
    let mut text = String::from("tconst\taverageRating\tnumVotes\n");
    for (tconst, rating, votes) in rows {
        text.push_str(&format!("{tconst}\t{rating}\t{votes}\n"));
    }
    for i in 0..filler {
        text.push_str(&format!("tt{}\t7.0\t1\n", 5_000_000 + i));
    }
    gz(&text)
}

/// The links: AniList 1, 3 and 4, MAL 22, 33 twice, 44 and 55, and enough
/// filler to clear the refresh's floor of a thousand.
fn links() -> String {
    let mut bindings = vec![
        json!({ "anilist": term("1"), "imdb": term("tt0213338") }),
        json!({ "mal": term("1"), "imdb": term("tt0213338") }),
        json!({ "mal": term("22"), "imdb": term("tt0000200") }),
        json!({ "mal": term("33"), "imdb": term("tt0000100") }),
        json!({ "mal": term("33"), "imdb": term("tt0000101") }),
        json!({ "anilist": term("3"), "imdb": term("tt0000300") }),
        json!({ "mal": term("44"), "imdb": term("tt0000301") }),
        json!({ "anilist": term("4"), "imdb": term("tt0000999") }),
        json!({ "mal": term("55"), "imdb": term("tt0000500") }),
    ];
    for i in 0..1_000u64 {
        bindings.push(json!({ "anilist": term(&(900_000 + i).to_string()), "imdb": term(&format!("tt{}", 9_000_000 + i)) }));
    }
    json!({ "head": { "vars": ["anilist", "mal", "imdb"] }, "results": { "bindings": bindings } })
        .to_string()
}

fn term(value: &str) -> serde_json::Value {
    json!({ "type": "literal", "value": value })
}

/// One series for every way a match can lead to IMDb, or not.
struct Library {
    tvmaze: u64,
    bebop: u64,
    fallback: u64,
    mal_only: u64,
    own_link_wins: u64,
    unrated: u64,
    tmdb: u64,
    unmatched: u64,
}

fn library(core: &Core) -> Library {
    let src = fixtures::insert_source(core, "/lib");
    let series = |name: &str| {
        fixtures::insert_series(core, src, SeriesKind::Show, &format!("/lib/{name}"), name)
    };
    let anilist = |name: &str, anilist_id: u64, mal_id: Option<u64>| {
        let id = series(name);
        fixtures::insert_media(
            core,
            anilist_id,
            Some(name),
            None,
            Some(12),
            "FINISHED",
            "TV",
            Some(80),
        );
        fixtures::match_series(core, id, Some(anilist_id), mal_id);
        id
    };

    let tvmaze = series("Avatar Seven Havens");
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT INTO tvmaze_shows (id, name, imdb, fetched_at) VALUES (83073, 'Avatar: Seven Havens', 'tt35854736', ?1)",
                params![time::now_secs()],
            )?;
            c.execute(
                "UPDATE series SET provider = 'tvmaze', tvmaze_id = 83073 WHERE id = ?1",
                params![tvmaze as i64],
            )?;
            Ok(())
        })
        .unwrap();

    let bebop = anilist("Cowboy Bebop", 1, Some(1));
    let fallback = anilist("Two", 2, Some(22));
    let mal_only = series("Thirty Three");
    fixtures::match_series(core, mal_only, None, Some(33));
    let own_link_wins = anilist("Three", 3, Some(44));
    let unrated = anilist("Four", 4, None);

    // A TMDB match carrying a MAL id Wikidata links: the match is TMDB, so
    // the MAL id is nothing to go on.
    let tmdb = series("A Film");
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE series SET provider = 'tmdb', tmdb_id = 5, tmdb_kind = 'movie', mal_id = 55 WHERE id = ?1",
                params![tmdb as i64],
            )?;
            Ok(())
        })
        .unwrap();
    let unmatched = series("Nobody Knows");

    Library {
        tvmaze,
        bebop,
        fallback,
        mal_only,
        own_link_wins,
        unrated,
        tmdb,
        unmatched,
    }
}

fn card(core: &Core, series: u64) -> SeriesCard {
    core.store()
        .read(move |c| cards::cards_for(c, Path::new("/images"), &[series]))
        .unwrap()
        .remove(0)
}

fn imdb_of(core: &Core, series: u64) -> (Option<String>, Option<f64>, Option<u64>) {
    let card = card(core, series);
    (card.imdb_id, card.imdb_score, card.imdb_votes)
}

fn count(core: &Core, table: &'static str) -> i64 {
    core.store()
        .read(move |c| Ok(c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))?))
        .unwrap()
}

fn setting(core: &Core, key: &'static str) -> Option<String> {
    core.store()
        .read(move |c| {
            Ok(c.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .ok())
        })
        .unwrap()
}

fn stamp(core: &Core, key: &'static str) -> Option<i64> {
    setting(core, key).map(|v| v.parse().unwrap())
}

/// Moves a download's clock back, as if it happened that long ago.
fn age(core: &Core, key: &'static str, secs: i64) {
    let at = time::now_secs() - secs;
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE settings SET value = ?2 WHERE key = ?1",
                params![key, at.to_string()],
            )?;
            Ok(())
        })
        .unwrap();
}

/// The requests that went to IMDb or Wikidata, by url.
fn upstream_requests(http: &FakeHttp) -> Vec<String> {
    http.requests()
        .into_iter()
        .map(|r| r.url)
        .filter(|u| u.contains(DUMP_HOST) || u.contains(WIKIDATA_HOST))
        .collect()
}

/// Runs the refresh to its end and hands back the terminal event.
fn run(core: &Arc<Core>, c: &Collector) -> (u64, Event) {
    let job = imdb::start_refresh(core);
    (job, common::wait_job(c, job))
}

/// Every series a job's `SeriesChanged` events named.
fn changed(c: &Collector, job: u64) -> BTreeSet<u64> {
    c.events()
        .into_iter()
        .filter(|e| e.job.as_ref().is_some_and(|j| j.id == job))
        .filter_map(|e| match e.body {
            EventBody::SeriesChanged { series } => Some(series),
            _ => None,
        })
        .flatten()
        .map(|card| card.id)
        .collect()
}

/// A first run downloads both, fills both tables whole, stamps both, and
/// tells the shell about exactly the cards that gained an IMDb title.
fn first_run(http: &Arc<FakeHttp>, core: &Arc<Core>, c: &Collector) -> u64 {
    http.push_for(WIKIDATA_HOST, 200, links());
    http.push_for_with_headers(
        DUMP_HOST,
        200,
        dump(&RATED, FILLER),
        vec![("Last-Modified".into(), LAST_MODIFIED.into())],
    );
    let (job, done) = run(core, c);
    assert_eq!(done.level, Level::Info, "{done:?}");
    assert!(
        done.message
            .starts_with("IMDb ratings updated, 100008 rows; IMDb links updated, 1009 rows"),
        "{}",
        done.message
    );
    job
}

#[test]
fn a_first_run_fills_both_tables_stamps_them_and_resolves_every_card() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    let before = time::now_secs();
    let job = first_run(&http, &core, &c);

    assert_eq!(count(&core, "imdb_ratings"), 100_008);
    assert_eq!(count(&core, "imdb_links"), 1_009);
    assert!(stamp(&core, "imdb_ratings_fetched_at").unwrap() >= before);
    assert!(stamp(&core, "imdb_links_fetched_at").unwrap() >= before);
    assert_eq!(
        setting(&core, "imdb_ratings_last_modified").as_deref(),
        Some(format!("\"{LAST_MODIFIED}\"").as_str())
    );

    // Each host asked once, the dump with no date to ask against yet.
    let requests = http.requests();
    let dump_request = requests.iter().find(|r| r.url.contains(DUMP_HOST)).unwrap();
    assert!(
        dump_request.headers.is_empty(),
        "{:?}",
        dump_request.headers
    );
    assert_eq!(upstream_requests(&http).len(), 2);

    // A TVmaze match names its own title.
    assert_eq!(
        imdb_of(&core, lib.tvmaze),
        (Some("tt35854736".into()), Some(7.7), Some(1_200))
    );
    // An AniList match goes through its AniList id's link.
    assert_eq!(
        imdb_of(&core, lib.bebop),
        (Some("tt0213338".into()), Some(8.9), Some(174_384))
    );
    // One with no AniList link falls back to its MAL id's.
    assert_eq!(
        imdb_of(&core, lib.fallback),
        (Some("tt0000200".into()), Some(7.0), Some(10))
    );
    // A MAL match takes its MAL id's, and of two titles the most voted.
    assert_eq!(
        imdb_of(&core, lib.mal_only),
        (Some("tt0000101".into()), Some(8.1), Some(900))
    );
    // An AniList link outranks the MAL one, however many votes that has.
    assert_eq!(
        imdb_of(&core, lib.own_link_wins),
        (Some("tt0000300".into()), Some(5.0), Some(10))
    );
    // A linked title the dump has no rating for is still the title.
    assert_eq!(
        imdb_of(&core, lib.unrated),
        (Some("tt0000999".into()), None, None)
    );
    // TMDB and no match at all have nothing to go on.
    assert_eq!(imdb_of(&core, lib.tmdb), (None, None, None));
    assert_eq!(imdb_of(&core, lib.unmatched), (None, None, None));

    assert_eq!(
        changed(&c, job),
        BTreeSet::from([
            lib.tvmaze,
            lib.bebop,
            lib.fallback,
            lib.mal_only,
            lib.own_link_wins,
            lib.unrated,
        ])
    );

    // The library read carries the same fields the job's cards did.
    let listed = match core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Series { series } => series,
        other => panic!("{other:?}"),
    };
    let bebop = listed.iter().find(|s| s.id == lib.bebop).unwrap();
    assert_eq!(bebop.imdb_id.as_deref(), Some("tt0213338"));
    assert_eq!(bebop.imdb_score, Some(8.9));
}

#[test]
fn a_second_run_inside_the_week_asks_nobody() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    library(&core);
    first_run(&http, &core, &c);
    assert_eq!(upstream_requests(&http).len(), 2);

    // Six days on, still inside the week.
    age(&core, "imdb_ratings_fetched_at", WEEK - 86_400);
    age(&core, "imdb_links_fetched_at", WEEK - 86_400);
    let (job, done) = run(&core, &c);
    assert_eq!(done.level, Level::Debug);
    assert_eq!(done.message, "IMDb ratings are current");
    assert_eq!(upstream_requests(&http).len(), 2);
    assert!(changed(&c, job).is_empty());
}

/// A week on, the dump is asked for by date; IMDb's 304 moves the clock
/// and nothing else, and the table is left exactly as it was.
#[test]
fn a_304_stamps_the_clock_without_rewriting_the_table() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    first_run(&http, &core, &c);

    // A mark the dump would wipe if it were written again.
    core.store()
        .write(|c| {
            c.execute("UPDATE imdb_ratings SET votes = 1 WHERE id = 213338", [])?;
            Ok(())
        })
        .unwrap();
    age(&core, "imdb_ratings_fetched_at", WEEK + 60);
    let aged = stamp(&core, "imdb_ratings_fetched_at").unwrap();
    http.push_for(DUMP_HOST, 304, "");

    let (job, done) = run(&core, &c);
    assert_eq!(done.level, Level::Debug, "{done:?}");
    assert_eq!(done.message, "IMDb ratings unchanged");
    let dump_request = http
        .requests()
        .into_iter()
        .rfind(|r| r.url.contains(DUMP_HOST))
        .unwrap();
    assert_eq!(
        dump_request.headers,
        vec![("If-Modified-Since".to_string(), LAST_MODIFIED.to_string())]
    );
    // The links were fresh, so only the dump was asked about.
    assert_eq!(upstream_requests(&http).len(), 3);
    assert!(stamp(&core, "imdb_ratings_fetched_at").unwrap() > aged);
    assert_eq!(count(&core, "imdb_ratings"), 100_008);
    assert_eq!(imdb_of(&core, lib.bebop).2, Some(1));
    assert!(changed(&c, job).is_empty());
}

/// A dump that does not read whole is not written over the one stored:
/// cut short, too small to be IMDb's, or not there at all. The old rows
/// stand, the clock stays where it was, and the next launch asks again.
#[test]
fn a_bad_dump_keeps_the_old_table_and_its_stamp() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    first_run(&http, &core, &c);
    age(&core, "imdb_ratings_fetched_at", WEEK + 60);
    let aged = stamp(&core, "imdb_ratings_fetched_at").unwrap();

    let mut cut = dump(&[("tt0213338", "1.0", 3)], FILLER);
    cut.truncate(cut.len() / 2);
    let short = dump(&[("tt0213338", "1.0", 3)], 10);
    let wrong_header = gz("tconst\tnumVotes\ntt0213338\t3\n");
    for body in [cut, short, wrong_header] {
        http.push_for_with_headers(
            DUMP_HOST,
            200,
            body,
            vec![(
                "Last-Modified".into(),
                "Thu, 15 Oct 2026 00:44:30 GMT".into(),
            )],
        );
        let (job, done) = run(&core, &c);
        assert_eq!(done.level, Level::Warn, "{done:?}");
        assert!(
            done.message.starts_with("IMDb ratings not refreshed: "),
            "{}",
            done.message
        );
        assert_eq!(count(&core, "imdb_ratings"), 100_008);
        assert_eq!(
            imdb_of(&core, lib.bebop),
            (Some("tt0213338".into()), Some(8.9), Some(174_384))
        );
        assert_eq!(stamp(&core, "imdb_ratings_fetched_at"), Some(aged));
        assert_eq!(
            setting(&core, "imdb_ratings_last_modified").as_deref(),
            Some(format!("\"{LAST_MODIFIED}\"").as_str())
        );
        assert!(changed(&c, job).is_empty());
    }

    // Offline is the same: nothing written, nothing stamped.
    http.fail_next("dns error");
    let (_, done) = run(&core, &c);
    assert_eq!(done.level, Level::Warn);
    assert_eq!(
        done.message,
        "IMDb ratings not refreshed: unreachable: dns error"
    );
    assert_eq!(stamp(&core, "imdb_ratings_fetched_at"), Some(aged));

    // And the next good dump lands, telling the shell about the one card
    // whose rating moved and no other.
    http.push_for_with_headers(
        DUMP_HOST,
        200,
        dump(
            &[
                ("tt0213338", "9.0", 174_500),
                ("tt35854736", "7.7", 1_200),
                ("tt0000100", "6.0", 5),
                ("tt0000101", "8.1", 900),
                ("tt0000200", "7.0", 10),
                ("tt0000300", "5.0", 10),
                ("tt0000301", "9.5", 99_999),
                ("tt0000500", "6.6", 66),
            ],
            FILLER,
        ),
        vec![(
            "Last-Modified".into(),
            "Thu, 15 Oct 2026 00:44:30 GMT".into(),
        )],
    );
    let (job, done) = run(&core, &c);
    assert_eq!(done.level, Level::Info, "{done:?}");
    assert_eq!(changed(&c, job), BTreeSet::from([lib.bebop]));
    assert_eq!(
        imdb_of(&core, lib.bebop),
        (Some("tt0213338".into()), Some(9.0), Some(174_500))
    );
    assert!(stamp(&core, "imdb_ratings_fetched_at").unwrap() > aged);
    assert_eq!(
        setting(&core, "imdb_ratings_last_modified").as_deref(),
        Some("\"Thu, 15 Oct 2026 00:44:30 GMT\"")
    );
}

/// Wikidata answering with a handful of rows is a query gone wrong rather
/// than the links shrinking to nothing, so the old links stand.
#[test]
fn a_suspiciously_small_link_answer_keeps_the_old_links() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let lib = library(&core);
    first_run(&http, &core, &c);
    age(&core, "imdb_links_fetched_at", WEEK + 60);
    let aged = stamp(&core, "imdb_links_fetched_at").unwrap();

    http.push_for(
        WIKIDATA_HOST,
        200,
        json!({ "results": { "bindings": [
            { "anilist": term("1"), "imdb": term("tt0000100") }
        ] } })
        .to_string(),
    );
    let (job, done) = run(&core, &c);
    assert_eq!(done.level, Level::Warn, "{done:?}");
    assert!(
        done.message.starts_with("IMDb links not refreshed: "),
        "{}",
        done.message
    );
    assert_eq!(count(&core, "imdb_links"), 1_009);
    assert_eq!(stamp(&core, "imdb_links_fetched_at"), Some(aged));
    assert_eq!(imdb_of(&core, lib.bebop).0.as_deref(), Some("tt0213338"));
    assert!(changed(&c, job).is_empty());
}
