mod common;
mod fixtures;
use anibeam_core::*;

/// One AniList search hit or `Media` reply, with everything a match writes.
fn media_json(id: u64, romaji: &str, id_mal: Option<u64>, cover: bool) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "idMal": id_mal,
        "title": { "romaji": romaji, "english": null, "native": null },
        "synonyms": [],
        "episodes": 12,
        "status": "FINISHED",
        "format": "TV",
        "seasonYear": 2020,
        "startDate": { "year": 2020, "month": 1, "day": 1 },
        "averageScore": 80,
        "coverImage": if cover {
            serde_json::json!({ "large": format!("https://img/{id}-l.jpg"), "extraLarge": format!("https://img/{id}-xl.jpg") })
        } else {
            serde_json::Value::Null
        },
        "studios": { "nodes": [] }
    })
}

/// The enrichment every fetch asks for, with nothing on it: no relations,
/// no recommendations, no characters, so the test's request count is the
/// three AniList calls and nothing more.
fn bare_enrichment(id: u64, id_mal: Option<u64>) -> serde_json::Value {
    serde_json::json!({ "data": { "Media": {
        "id": id,
        "idMal": id_mal,
        "type": "ANIME",
        "streamingEpisodes": [],
        "tags": [],
        "studios": { "edges": [] },
        "characters": { "edges": [] },
        "recommendations": { "edges": [] },
        "relations": { "edges": [] }
    } } })
}

fn empty_schedule(id: u64) -> serde_json::Value {
    serde_json::json!({ "data": { "Media": { "id": id, "nextAiringEpisode": null, "airingSchedule": { "nodes": [] } } } })
}

fn not_found() -> serde_json::Value {
    serde_json::json!({ "data": null, "errors": [ { "message": "Not Found.", "status": 404 } ] })
}

fn started(core: &Core, call: Call) -> u64 {
    match core.call(call).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

fn list_cards(core: &Core) -> Vec<SeriesCard> {
    match core
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
    }
}

/// The `perPage` the search actually asked AniList for.
fn per_page(req: &anibeam_core::net::HttpRequest) -> u64 {
    match &req.body {
        Some(anibeam_core::net::Body::Json(v)) => v["variables"]["perPage"].as_u64().unwrap(),
        other => panic!("expected a json body, got {other:?}"),
    }
}

/// AniList is the only provider the native line can search: TMDB has no
/// client at all, and MAL is a tracker rather than a matching provider.
#[test]
fn a_search_answers_from_anilist_and_refuses_every_other_provider() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());

    let err = core
        .call(Call::SearchProvider {
            provider: Provider::Tmdb,
            query: "Frieren".into(),
            limit: 12,
        })
        .err()
        .unwrap();
    assert!(
        matches!(&err, CoreError::Unsupported { what } if what == "search on tmdb"),
        "{err:?}"
    );
    let err = core
        .call(Call::SearchProvider {
            provider: Provider::Mal,
            query: "Frieren".into(),
            limit: 12,
        })
        .err()
        .unwrap();
    assert!(
        matches!(&err, CoreError::Unsupported { what } if what == "search on mal"),
        "{err:?}"
    );

    // A query too short to be worth a request never becomes a job.
    let err = core
        .call(Call::SearchProvider {
            provider: Provider::Anilist,
            query: " a ".into(),
            limit: 12,
        })
        .err()
        .unwrap();
    assert!(
        matches!(&err, CoreError::Invalid { field, .. } if field == "query"),
        "{err:?}"
    );
    assert!(http.requests().is_empty());

    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "media": [
            {
                "id": 1,
                "title": { "romaji": "Sousou no Frieren", "english": "Frieren: Beyond Journey's End", "native": "\u{846c}\u{9001}\u{306e}\u{30d5}\u{30ea}\u{30fc}\u{30ec}\u{30f3}" },
                "format": "TV",
                "seasonYear": 2023,
                "episodes": 28,
                "coverImage": { "large": "https://img/1-l.jpg", "extraLarge": "https://img/1-xl.jpg" }
            },
            {
                "id": 2,
                "title": { "romaji": null, "english": null, "native": "\u{30ca}\u{30cb}\u{30ab}" },
                "format": "MOVIE",
                "seasonYear": null,
                "startDate": { "year": 2019 },
                "episodes": null,
                "coverImage": { "large": "https://img/2-l.jpg", "extraLarge": null }
            }
        ] } } }),
    );
    let job = started(
        &core,
        Call::SearchProvider {
            provider: Provider::Anilist,
            query: "Frieren".into(),
            limit: 12,
        },
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.message, "search: 2 results for \"Frieren\"");
    assert_eq!(done.level, Level::Debug);
    let EventBody::SearchFinished { results } = done.body else {
        panic!("{done:?}")
    };
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].provider, Provider::Anilist);
    assert_eq!(results[0].id, 1);
    assert_eq!(results[0].title, "Sousou no Frieren");
    assert_eq!(
        results[0].alt_title.as_deref(),
        Some("Frieren: Beyond Journey's End")
    );
    assert_eq!(results[0].format.as_deref(), Some("TV"));
    assert_eq!(results[0].year, Some(2023));
    assert_eq!(results[0].episodes, Some(28));
    assert_eq!(
        results[0].cover_url.as_deref(),
        Some("https://img/1-xl.jpg")
    );
    // No romaji and no english: the native title is the fallback, the year
    // comes off the start date, and the large cover stands in for the
    // extra large one.
    assert_eq!(results[1].title, "\u{30ca}\u{30cb}\u{30ab}");
    assert_eq!(results[1].alt_title, None);
    assert_eq!(results[1].year, Some(2019));
    assert_eq!(results[1].episodes, None);
    assert_eq!(results[1].cover_url.as_deref(), Some("https://img/2-l.jpg"));
    assert_eq!(per_page(&http.requests()[0]), 12);

    // The shell asks for what it wants, inside the provider's bounds.
    http.push_json(
        200,
        serde_json::json!({ "data": { "Page": { "media": [] } } }),
    );
    let job = started(
        &core,
        Call::SearchProvider {
            provider: Provider::Anilist,
            query: "Frieren".into(),
            limit: 0,
        },
    );
    common::wait_job(&c, job);
    assert_eq!(per_page(&http.requests()[1]), 1);

    core.shutdown();
}

/// A MyAnimeList link is resolved through AniList, because a MAL id is not
/// something the core can draw a series page from. AniList never having
/// heard of it is the one case the modal has to say out loud.
#[test]
fn a_mal_link_anilist_cannot_resolve_fails_the_job_with_a_message() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());

    // Not a link at all, and a link with nothing behind it.
    let err = core
        .call(Call::ResolveLink {
            url: "Frieren".into(),
        })
        .err()
        .unwrap();
    assert!(
        matches!(&err, CoreError::Invalid { field, message } if field == "url" && message == "not a link"),
        "{err:?}"
    );
    let err = core
        .call(Call::ResolveLink {
            url: "https://www.themoviedb.org/movie/550".into(),
        })
        .err()
        .unwrap();
    assert!(
        matches!(&err, CoreError::Invalid { field, message }
            if field == "url" && message == "Couldn't read that link. Paste an AniList or MyAnimeList page URL."),
        "{err:?}"
    );

    // An AniList link needs no request at all.
    let job = started(
        &core,
        Call::ResolveLink {
            url: "https://anilist.co/anime/21/One-Piece".into(),
        },
    );
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::LinkResolved {
            target: MatchTarget::Anilist {
                id: 21,
                season: None
            }
        }
    );
    assert!(http.requests().is_empty());

    http.push_json(200, not_found());
    let job = started(
        &core,
        Call::ResolveLink {
            url: "https://myanimelist.net/anime/9999/Nothing".into(),
        },
    );
    let done = common::wait_job(&c, job);
    let EventBody::JobFailed { error } = done.body else {
        panic!("{done:?}")
    };
    assert!(
        matches!(&error, CoreError::Provider { message, .. } if message == "AniList has no entry for that MyAnimeList id."),
        "{error:?}"
    );

    core.shutdown();
}

/// The match is written and reported before the record is fetched, so the
/// modal can close on a card that already says matched while the four
/// provider calls run behind it.
#[test]
fn apply_match_writes_the_match_first_and_the_record_after() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/lib/Sousou no Frieren",
        "Sousou no Frieren",
    );
    fixtures::insert_file(
        &core,
        series,
        "/lib/Sousou no Frieren/01.mkv",
        1.0,
        None,
        "episode",
        1,
    );

    assert!(matches!(
        core.call(Call::ApplyMatch {
            series: 9999,
            target: MatchTarget::Anilist {
                id: 1,
                season: None
            }
        }),
        Err(CoreError::NotFound {
            what: Entity::Series,
            id: 9999
        })
    ));

    // Media by id, enrichment, schedule, Jikan, then the cover.
    http.push_json(200, serde_json::json!({ "data": { "Media": media_json(1, "Sousou no Frieren", Some(1001), true) } }));
    http.push_json(200, bare_enrichment(1, Some(1001)));
    http.push_json(200, empty_schedule(1));
    http.push_for(
        "jikan.moe",
        200,
        serde_json::json!({ "data": [] }).to_string(),
    );
    http.push_for("img/1-xl.jpg", 200, vec![1, 2, 3]);

    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Anilist {
                id: 1,
                season: None,
            },
        },
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.level, Level::Info, "{done:#?}");
    assert_eq!(done.message, "matched Sousou no Frieren to AniList 1");
    assert_eq!(done.body, EventBody::MatchApplied { series });

    // The first card out is confirmed and has no poster yet: it was
    // written before the fetch, which is the whole point of the step.
    let events = c.events();
    let first = events
        .iter()
        .position(|e| matches!(&e.body, EventBody::SeriesChanged { series } if series.iter().any(|s| s.match_info.as_ref().is_some_and(|m| m.confirmed))))
        .expect("a confirmed card before the fetch");
    let applied = events
        .iter()
        .position(|e| matches!(e.body, EventBody::MatchApplied { .. }))
        .unwrap();
    assert!(first < applied, "{:#?}", events);
    let EventBody::SeriesChanged { series: early } = &events[first].body else {
        unreachable!()
    };
    assert!(early[0].poster.is_none(), "{early:?}");

    let cards = list_cards(&core);
    let card = cards.iter().find(|s| s.id == series).unwrap();
    assert_eq!(
        card.match_info
            .as_ref()
            .map(|m| (m.provider, m.anilist_id, m.mal_id, m.confirmed)),
        Some((Provider::Anilist, Some(1), Some(1001), true))
    );
    assert!(card.poster.is_some(), "{card:?}");

    // A confirmed match takes the series out of the auto-match's way for
    // good: nothing left to search for.
    let before = http.requests().len();
    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::AutoMatchFinished {
                matched: 0,
                unmatched: 0,
                ..
            }
        ),
        "{done:?}"
    );
    assert_eq!(http.requests().len(), before);

    core.shutdown();
}

/// A refresh re-runs the same fetch against the id the series already
/// carries. An unmatched series has nothing to refresh, and the Match
/// button is the recovery.
#[test]
fn refresh_refuses_an_unmatched_series_and_refetches_a_matched_one() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let bare = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Unmatched", "Unmatched");
    let matched = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Matched", "Matched");
    fixtures::insert_media(
        &core,
        7,
        Some("Old title"),
        None,
        Some(12),
        "FINISHED",
        "TV",
        Some(80),
    );
    fixtures::match_series(&core, matched, Some(7), None);

    assert!(matches!(
        core.call(Call::RefreshSeries { series: 9999 }),
        Err(CoreError::NotFound {
            what: Entity::Series,
            id: 9999
        })
    ));
    assert!(matches!(
        core.call(Call::RefreshSeries { series: bare }),
        Err(CoreError::Refused {
            reason: Refusal::Unmatched
        })
    ));
    assert!(http.requests().is_empty());

    // No MAL id and no cover, so the fetch is exactly the three AniList
    // calls: no Jikan side-fetch and no image to bring in.
    http.push_json(
        200,
        serde_json::json!({ "data": { "Media": {
            "id": 7,
            "idMal": null,
            "title": { "romaji": "New title", "english": null, "native": null },
            "synonyms": [],
            "episodes": 24,
            "status": "RELEASING",
            "format": "TV",
            "coverImage": null,
            "studios": { "nodes": [] }
        } } }),
    );
    http.push_json(200, bare_enrichment(7, None));
    http.push_json(200, empty_schedule(7));

    let job = started(&core, Call::RefreshSeries { series: matched });
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::RefreshFinished {
            refreshed: 1,
            failed: 0
        }
    );
    assert_eq!(done.level, Level::Info);
    assert_eq!(http.requests().len(), 3);

    let cards = list_cards(&core);
    assert_eq!(
        cards.iter().find(|s| s.id == matched).unwrap().title,
        "New title"
    );

    core.shutdown();
}

/// A MAL id AniList has never heard of is still a match the user asked
/// for: it is written MAL-only, titled by the folder, with no record and
/// no poster behind it, and the auto-match leaves it alone.
#[test]
fn a_mal_target_anilist_cannot_resolve_becomes_a_mal_only_match() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let series = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/lib/Obscure OVA",
        "Obscure OVA",
    );
    fixtures::insert_file(
        &core,
        series,
        "/lib/Obscure OVA/01.mkv",
        1.0,
        None,
        "episode",
        1,
    );

    http.push_json(200, not_found());
    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Mal { id: 5 },
        },
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.body, EventBody::MatchApplied { series });
    assert_eq!(http.requests().len(), 1);

    let cards = list_cards(&core);
    let card = cards.iter().find(|s| s.id == series).unwrap();
    assert_eq!(
        card.match_info
            .as_ref()
            .map(|m| (m.provider, m.anilist_id, m.mal_id, m.confirmed)),
        Some((Provider::Mal, None, Some(5), true))
    );
    assert_eq!(card.title, "Obscure OVA");
    assert!(card.poster.is_none(), "{card:?}");

    // There is no AniList id to refetch, so a refresh says so rather than
    // guessing at one.
    let err = core.call(Call::RefreshSeries { series }).err().unwrap();
    assert!(
        matches!(&err, CoreError::Unsupported { what } if what == "refresh of a MAL-only or TMDB match"),
        "{err:?}"
    );

    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::AutoMatchFinished {
                matched: 0,
                unmatched: 0,
                ..
            }
        ),
        "{done:?}"
    );
    assert_eq!(http.requests().len(), 1);

    core.shutdown();
}

/// The whole-library refresh and the stub backfill are the same walk over
/// two different lists: everything carrying an AniList id, and everything
/// whose media row was only ever known of.
#[test]
fn the_backfill_walks_the_stubs_and_refresh_all_walks_every_match() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let fetched = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Fetched", "Fetched");
    let stub = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Stub", "Stub");
    fixtures::insert_media(
        &core,
        10,
        Some("Fetched"),
        None,
        Some(12),
        "FINISHED",
        "TV",
        Some(80),
    );
    // A row known from an edge, a list or an import: an id and a title,
    // and `fetched_at` still NULL.
    core.store()
        .write(|conn| {
            conn.execute("INSERT INTO anilist_media (id, media_type, title_romaji) VALUES (11, 'ANIME', 'Stub')", [])?;
            Ok(())
        })
        .unwrap();
    fixtures::match_series(&core, fetched, Some(10), None);
    fixtures::match_series(&core, stub, Some(11), None);

    // The backfill takes the stub and nothing else.
    http.push_json(
        200,
        serde_json::json!({ "data": { "Media": media_json(11, "Stub filled in", None, false) } }),
    );
    http.push_json(200, bare_enrichment(11, None));
    http.push_json(200, empty_schedule(11));
    let job = anibeam_core::metadata::apply::backfill_stubs(&core);
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::RefreshFinished {
            refreshed: 1,
            failed: 0
        }
    );
    assert_eq!(done.message, "backfill: 1 refreshed, 0 failed");
    assert_eq!(http.requests().len(), 3);
    assert_eq!(
        list_cards(&core)
            .iter()
            .find(|s| s.id == stub)
            .unwrap()
            .title,
        "Stub filled in"
    );

    // Nothing is a stub any more, so a second backfill has no work at all.
    let job = anibeam_core::metadata::apply::backfill_stubs(&core);
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::RefreshFinished {
            refreshed: 0,
            failed: 0
        }
    );
    assert_eq!(http.requests().len(), 3);

    // RefreshAll takes both, in id order.
    for (anilist_id, romaji) in [(10, "Fetched again"), (11, "Stub again")] {
        http.push_json(
            200,
            serde_json::json!({ "data": { "Media": media_json(anilist_id, romaji, None, false) } }),
        );
        http.push_json(200, bare_enrichment(anilist_id, None));
        http.push_json(200, empty_schedule(anilist_id));
    }
    let job = started(&core, Call::RefreshAll);
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::RefreshFinished {
            refreshed: 2,
            failed: 0
        }
    );
    assert_eq!(done.message, "refresh: 2 refreshed, 0 failed");
    assert_eq!(http.requests().len(), 9);
    let cards = list_cards(&core);
    assert_eq!(
        cards.iter().find(|s| s.id == fetched).unwrap().title,
        "Fetched again"
    );
    assert_eq!(
        cards.iter().find(|s| s.id == stub).unwrap().title,
        "Stub again"
    );

    core.shutdown();
}

/// AniList not answering at all ends the walk. Every series after the
/// first keeps its record and its place in the next run's list, and the
/// terminal is the failure rather than a count of series that never had
/// their turn.
#[test]
fn a_transport_failure_ends_the_refresh_walk() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    for (n, name) in [(10u64, "Aaa"), (11, "Bbb")] {
        let series =
            fixtures::insert_series(&core, src, SeriesKind::Show, &format!("/lib/{name}"), name);
        fixtures::insert_media(
            &core,
            n,
            Some(name),
            None,
            Some(12),
            "FINISHED",
            "TV",
            Some(80),
        );
        fixtures::match_series(&core, series, Some(n), None);
    }

    // A connection that never opened. The limiter does not retry one, so
    // this is a single request and the second series is never asked about.
    http.fail_next("connection refused");

    let job = started(&core, Call::RefreshAll);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::JobFailed {
                error: CoreError::Provider { status: None, .. }
            }
        ),
        "{done:?}"
    );
    assert_eq!(
        http.requests().len(),
        1,
        "the walk carried on past an outage"
    );
    core.shutdown();
}

/// The backfill writes records, so it brings covers and banners in, so it
/// owes the sweep that keeps the image directory from only ever growing.
/// It used to run the sweep for RefreshAll alone.
#[test]
fn the_backfill_sweeps_the_image_cache_like_every_other_refresh() {
    let http = anibeam_core::net::FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let src = fixtures::insert_source(&core, "/lib");
    let stub = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Stub", "Stub");
    core.store()
        .write(|conn| {
            conn.execute(
                "INSERT INTO anilist_media (id, media_type, title_romaji) VALUES (11, 'ANIME', 'Stub')",
                [],
            )?;
            // A row whose file is not there: a lie the sweep exists to
            // clear out.
            conn.execute(
                "INSERT INTO images (url, path, bytes, fetched_at, used_at) VALUES ('https://img/gone.jpg', 'ab/gone.jpg', 2, 1, 1)",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    fixtures::match_series(&core, stub, Some(11), None);

    http.push_json(
        200,
        serde_json::json!({ "data": { "Media": media_json(11, "Stub filled in", None, false) } }),
    );
    http.push_json(200, bare_enrichment(11, None));
    http.push_json(200, empty_schedule(11));
    let job = anibeam_core::metadata::apply::backfill_stubs(&core);
    common::wait_job(&c, job);

    let rows: i64 = core
        .store()
        .read(|conn| Ok(conn.query_row("SELECT count(*) FROM images", [], |r| r.get(0))?))
        .unwrap();
    assert_eq!(
        rows, 0,
        "the backfill left a row naming a file that is gone"
    );
}
