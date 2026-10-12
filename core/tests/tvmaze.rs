//! TVmaze end to end: the auto-match falling through to it in either
//! order, the manual search, link and match, the refresh, what a TVmaze
//! series draws on every page, and that it never reaches a tracker.
//!
//! The replies are TVmaze's own, captured with curl on 2026-10-12 for
//! Avatar: Seven Havens, show 83073. Its air dates are moved to sit around
//! the moment the test runs, so the schedule reads the same any day.

mod common;
mod fixtures;

use std::sync::Arc;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;
use rusqlite::params;
use serde_json::{Value, json};

const SHOW: &str = include_str!("fixtures/tvmaze/show-83073.json");
const SEARCH: &str = include_str!("fixtures/tvmaze/search-avatar-seven-havens.json");
const SEARCH_AVATAR: &str = include_str!("fixtures/tvmaze/search-avatar.json");

const POSTER: &str = "https://static.tvmaze.com/uploads/images/original_untouched/645/1613483.jpg";
const BACKGROUND: &str =
    "https://static.tvmaze.com/uploads/images/original_untouched/579/1449291.jpg";
const DAY: i64 = 86_400;

/// The show with its first three episodes aired over the last three days
/// and the rest a week apart from tomorrow on.
fn show_json() -> String {
    let mut show: Value = serde_json::from_str(SHOW).unwrap();
    let now = time::now_secs();
    for e in show["_embedded"]["episodes"].as_array_mut().unwrap() {
        let n = e["number"].as_i64().unwrap();
        let at = if n <= 3 {
            now - (4 - n) * DAY
        } else {
            now + (n - 3) * 7 * DAY
        };
        let stamp = ::time::OffsetDateTime::from_unix_timestamp(at)
            .unwrap()
            .format(&::time::format_description::well_known::Rfc3339)
            .unwrap();
        e["airstamp"] = Value::String(stamp);
    }
    show.to_string()
}

/// The two pictures the fetch waits for, answered once each.
fn push_images(http: &FakeHttp) {
    http.push_for("1613483.jpg", 200, vec![0xFF, 0xD8, 0xFF, 0xE0]);
    http.push_for("1449291.jpg", 200, vec![0xFF, 0xD8, 0xFF, 0xE1]);
}

/// The whole of a TVmaze match: the show and its two pictures.
fn push_show(http: &FakeHttp) {
    http.push_for("api.tvmaze.com/shows/83073?embed", 200, show_json());
    push_images(http);
}

fn anilist_search(media: Value) -> String {
    json!({ "data": { "Page": { "media": media } } }).to_string()
}

fn started(core: &Core, call: Call) -> u64 {
    match core.call(call).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

fn card(core: &Core, series: u64) -> SeriesCard {
    detail(core, series).card
}

fn detail(core: &Core, series: u64) -> SeriesDetail {
    match core.call(Call::GetSeries { series }).unwrap() {
        Reply::SeriesDetail { detail } => *detail,
        other => panic!("{other:?}"),
    }
}

fn match_row(core: &Core, series: u64) -> (Option<String>, Option<i64>, Option<i64>, i64) {
    core.store()
        .read(|c| {
            Ok(c.query_row(
                "SELECT provider, tvmaze_id, anilist_id, confirmed FROM series WHERE id = ?1",
                params![series as i64],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )?)
        })
        .unwrap()
}

fn attempted_at(core: &Core, series: u64) -> Option<i64> {
    core.store()
        .read(|c| {
            Ok(c.query_row(
                "SELECT attempted_at FROM series WHERE id = ?1",
                params![series as i64],
                |r| r.get(0),
            )?)
        })
        .unwrap()
}

/// The Avatar folder as the real library holds it: three episodes of
/// season 1, named the way the release named them.
fn avatar(core: &Core) -> (u64, Vec<u64>) {
    let src = fixtures::insert_source(core, "/tv");
    let series = fixtures::insert_series(
        core,
        src,
        SeriesKind::Show,
        "/tv/Avatar Seven Havens",
        "Avatar Seven Havens",
    );
    let files = (1..=3)
        .map(|n| {
            fixtures::insert_file(
                core,
                series,
                &format!("/tv/Avatar Seven Havens/Avatar.Seven.Havens.S01E0{n}.mkv"),
                f64::from(n),
                Some(1),
                "episode",
                1,
            )
        })
        .collect();
    (series, files)
}

/// What a matched Avatar draws, on its card and its page.
fn assert_avatar_drawn(core: &Core, series: u64) {
    let d = detail(core, series);
    let card = &d.card;
    assert_eq!(card.title, "Avatar: Seven Havens");
    assert_eq!(
        card.match_info
            .as_ref()
            .map(|m| (m.provider, m.tvmaze_id, m.anilist_id, m.mal_id)),
        Some((Provider::Tvmaze, Some(83073), None, None))
    );
    assert!(card.poster.is_some(), "{card:?}");
    assert_eq!(card.format.as_deref(), Some("TV"));
    assert_eq!(card.status, Some(AiringStatus::Releasing));
    // Three aired of a running show: an estimate, never a total.
    assert_eq!(
        (card.total_episodes, card.total_is_estimate),
        (Some(3), true)
    );
    let next = card.next_airing.as_ref().unwrap();
    assert_eq!(next.episode, 4);
    assert!(d.banner.is_some(), "{d:?}");
    assert!(
        d.synopsis
            .starts_with("In a world shattered by a devastating cataclysm"),
        "{}",
        d.synopsis
    );
    assert!(!d.synopsis.contains('<'), "{}", d.synopsis);
    assert_eq!(d.year, Some(2026));
    assert_eq!(d.studio.as_deref(), Some("Paramount+"));
    assert_eq!(
        d.genres,
        vec!["Action".to_string(), "Adventure".to_string()]
    );
    assert_eq!(
        d.site_url.as_deref(),
        Some("https://www.tvmaze.com/shows/83073/avatar-seven-havens")
    );
    assert!(!d.has_graph);
    assert!(d.characters.is_empty() && d.recommendations.is_empty());
    let titles: Vec<Option<&str>> = d.episodes.iter().map(|e| e.title.as_deref()).collect();
    assert_eq!(
        titles,
        vec![
            Some("On the Horizon"),
            Some("Into the Storm"),
            Some("The Order")
        ]
    );
    assert!(d.episodes.iter().all(|e| e.air_date.is_some()));
}

/// The default order: AniList is asked first and misses, TVmaze is asked
/// next and matches. The match is the whole record, drawn everywhere.
#[test]
fn the_auto_match_falls_through_to_tvmaze_when_anilist_misses() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, _) = avatar(&core);

    http.push_for(
        "graphql.anilist.co",
        200,
        anilist_search(json!([{
            "id": 1, "title": { "romaji": "Avatar: The Last Airbender" },
            "coverImage": { "large": "https://img/1.jpg" }
        }])),
    );
    http.push_for("search/shows?q=Avatar%20Seven%20Havens", 200, SEARCH);
    push_show(&http);

    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::AutoMatchFinished {
                matched: 1,
                unmatched: 0,
                ..
            }
        ),
        "{done:?}"
    );
    assert!(
        c.events().iter().any(
            |e| e.message == "match (TVmaze 1.00): Avatar Seven Havens -> Avatar: Seven Havens"
        ),
        "{:#?}",
        c.events()
    );
    let urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
    assert!(urls[0].contains("graphql.anilist.co"), "{urls:#?}");
    assert!(urls[1].contains("search/shows"), "{urls:#?}");
    assert_eq!(
        match_row(&core, series),
        (Some("tvmaze".into()), Some(83073), None, 0)
    );
    assert_avatar_drawn(&core, series);

    // The metadata table names the provider.
    match core
        .call(Call::ListMetadata {
            filter: MetadataFilter::All,
            query: String::new(),
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Metadata { rows, .. } => {
            assert_eq!(rows[0].provider, Some(Provider::Tvmaze));
            assert_eq!(rows[0].expected, Some(13));
        }
        other => panic!("{other:?}"),
    }
    core.shutdown();
}

/// TVmaze first: an anime TVmaze also carries is matched there and AniList
/// is never asked. A folder TVmaze misses still reaches AniList.
#[test]
fn with_tvmaze_first_a_tvmaze_hit_never_asks_anilist_and_a_miss_still_does() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    core.call(Call::SetProviderOrder {
        order: vec![Provider::Tvmaze, Provider::Anilist],
    })
    .unwrap();
    let (avatar, _) = avatar(&core);
    let src = fixtures::insert_source(&core, "/anime");
    let frieren = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        "/anime/Sousou no Frieren",
        "Sousou no Frieren",
    );
    fixtures::insert_file(
        &core,
        frieren,
        "/anime/Sousou no Frieren/01.mkv",
        1.0,
        None,
        "episode",
        1,
    );

    // Avatar, the lower id, goes first: TVmaze hits.
    http.push_for("search/shows?q=Avatar%20Seven%20Havens", 200, SEARCH);
    push_show(&http);
    // Frieren: TVmaze has only shows that score under the bar, so AniList
    // gets its turn and matches.
    http.push_for("search/shows?q=Sousou%20no%20Frieren", 200, SEARCH_AVATAR);
    http.push_for(
        "graphql.anilist.co",
        200,
        anilist_search(json!([{
            "id": 154587, "idMal": 52991,
            "title": { "romaji": "Sousou no Frieren" },
            "coverImage": { "extraLarge": "https://img/frieren.jpg" }
        }])),
    );
    http.push_for(
        "graphql.anilist.co",
        200,
        json!({ "data": { "Media": { "id": 154587, "idMal": 52991, "type": "ANIME", "streamingEpisodes": [], "tags": [], "studios": { "edges": [] }, "characters": { "edges": [] }, "recommendations": { "edges": [] }, "relations": { "edges": [] } } } }).to_string(),
    );
    http.push_for(
        "graphql.anilist.co",
        200,
        json!({ "data": { "Media": { "id": 154587, "nextAiringEpisode": null, "airingSchedule": { "nodes": [] } } } }).to_string(),
    );
    http.push_for("jikan.moe", 200, json!({ "data": [] }).to_string());
    http.push_for("img/frieren.jpg", 200, vec![1, 2, 3]);

    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::AutoMatchFinished {
                matched: 2,
                unmatched: 0,
                ..
            }
        ),
        "{done:?}"
    );
    let urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
    // Avatar's turn was TVmaze's alone: the first AniList request comes
    // after Frieren's TVmaze search.
    let first_anilist = urls.iter().position(|u| u.contains("anilist")).unwrap();
    let frieren_search = urls
        .iter()
        .position(|u| u.contains("q=Sousou%20no%20Frieren"))
        .unwrap();
    assert!(frieren_search < first_anilist, "{urls:#?}");
    assert_eq!(match_row(&core, avatar).0.as_deref(), Some("tvmaze"));
    assert_eq!(
        match_row(&core, frieren),
        (Some("anilist".into()), None, Some(154587), 0)
    );
    core.shutdown();
}

/// Both providers missing is the one outcome that stamps a series.
#[test]
fn a_miss_on_every_provider_stamps_the_series_once() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, _) = avatar(&core);
    http.push_for("graphql.anilist.co", 200, anilist_search(json!([])));
    http.push_for("search/shows", 200, "[]");

    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::AutoMatchFinished { unmatched: 1, .. }),
        "{done:?}"
    );
    assert!(attempted_at(&core, series).is_some());
    assert_eq!(http.requests().len(), 2);
    core.shutdown();
}

/// TVmaze saying stop, after AniList missed, is no more the series' fault
/// than AniList saying it: the job fails and nothing is stamped, so the
/// series is still a candidate next run.
#[test]
fn a_tvmaze_rate_limit_fails_the_job_and_stamps_nothing() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, _) = avatar(&core);
    http.push_for("graphql.anilist.co", 200, anilist_search(json!([])));
    for _ in 0..7 {
        http.push_for_with_headers(
            "search/shows",
            429,
            "",
            vec![("Retry-After".into(), "0".into())],
        );
    }

    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            &done.body,
            EventBody::JobFailed {
                error: CoreError::Provider {
                    provider: Provider::Tvmaze,
                    status: Some(429),
                    ..
                }
            }
        ),
        "{done:?}"
    );
    assert!(attempted_at(&core, series).is_none());

    // And a connection that never opened is the same.
    http.push_for("graphql.anilist.co", 200, anilist_search(json!([])));
    http.fail_next("connection refused");
    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    // The AniList search took the failure: nothing answered at all.
    assert!(
        matches!(
            &done.body,
            EventBody::JobFailed {
                error: CoreError::Provider { status: None, .. }
            }
        ),
        "{done:?}"
    );
    assert!(attempted_at(&core, series).is_none());
    core.shutdown();
}

/// The version bump: a series AniList alone missed under version 1 is
/// tried again under 2, which is what puts Avatar in front of TVmaze.
#[test]
fn a_series_stamped_under_version_1_is_tried_again() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, _) = avatar(&core);
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE series SET attempted_at = 100, attempt_version = 1 WHERE id = ?1",
                params![series as i64],
            )?;
            c.execute(
                "INSERT INTO settings (key, value) VALUES ('auto_match_version', '1')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    http.push_for("graphql.anilist.co", 200, anilist_search(json!([])));
    http.push_for("search/shows?q=Avatar%20Seven%20Havens", 200, SEARCH);
    push_show(&http);

    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(done.body, EventBody::AutoMatchFinished { matched: 1, .. }),
        "{done:?}"
    );
    assert!(
        c.events()
            .iter()
            .any(|e| e.message == "auto-match v2: re-attempting 1 unmatched series"),
        "{:#?}",
        c.events()
    );
    assert_eq!(match_row(&core, series).1, Some(83073));
    core.shutdown();
}

/// The manual half: a TVmaze search lists shows with their network and
/// year, a TVmaze or an IMDb link resolves to a TVmaze target, and an
/// anime matched to AniList can be matched again through TVmaze, which
/// clears every AniList trace off the row.
#[test]
fn a_series_is_searched_linked_and_rematched_through_tvmaze() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, _) = avatar(&core);
    fixtures::insert_media(
        &core,
        1,
        Some("Avatar: The Last Airbender"),
        None,
        Some(61),
        "FINISHED",
        "TV",
        Some(90),
    );
    fixtures::match_series(&core, series, Some(1), Some(11));

    http.push_for("search/shows?q=avatar", 200, SEARCH_AVATAR);
    let job = started(
        &core,
        Call::SearchProvider {
            provider: Provider::Tvmaze,
            query: "avatar".into(),
            limit: 4,
        },
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.message, "search: 4 TVmaze results for \"avatar\"");
    let EventBody::SearchFinished { results } = done.body else {
        panic!("{done:?}")
    };
    assert_eq!(results.len(), 4);
    assert_eq!(results[0].provider, Provider::Tvmaze);
    assert_eq!(results[0].id, 83073);
    assert_eq!(results[0].title, "Avatar: Seven Havens");
    assert_eq!(results[0].format.as_deref(), Some("Paramount+"));
    assert_eq!(results[0].year, Some(2026));
    assert!(
        results[0]
            .cover_url
            .as_deref()
            .unwrap()
            .contains("medium_portrait")
    );

    // A TVmaze link needs no request; an IMDb link is one lookup.
    let before = http.requests().len();
    let job = started(
        &core,
        Call::ResolveLink {
            url: "https://www.tvmaze.com/shows/83073/avatar-seven-havens".into(),
        },
    );
    assert_eq!(
        common::wait_job(&c, job).body,
        EventBody::LinkResolved {
            target: MatchTarget::Tvmaze { id: 83073 }
        }
    );
    assert_eq!(http.requests().len(), before);
    http.push_for("lookup/shows?imdb=tt35854736", 200, show_json());
    let job = started(
        &core,
        Call::ResolveLink {
            url: "https://www.imdb.com/title/tt35854736/".into(),
        },
    );
    assert_eq!(
        common::wait_job(&c, job).body,
        EventBody::LinkResolved {
            target: MatchTarget::Tvmaze { id: 83073 }
        }
    );
    http.push_for("lookup/shows?imdb=tt0000001", 404, "");
    let job = started(
        &core,
        Call::ResolveLink {
            url: "imdb.com/title/tt0000001".into(),
        },
    );
    let done = common::wait_job(&c, job);
    assert!(
        matches!(&done.body, EventBody::JobFailed { error: CoreError::Provider { provider: Provider::Tvmaze, status: Some(404), message, .. } }
            if message == "TVmaze has no show for that IMDb title."),
        "{done:?}"
    );

    // The re-match.
    push_show(&http);
    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Tvmaze { id: 83073 },
        },
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.body, EventBody::MatchApplied { series });
    assert_eq!(done.message, "matched Avatar Seven Havens to TVmaze 83073");
    assert_eq!(
        match_row(&core, series),
        (Some("tvmaze".into()), Some(83073), None, 1)
    );
    let mal: Option<i64> = core
        .store()
        .read(|c| {
            Ok(c.query_row(
                "SELECT mal_id FROM series WHERE id = ?1",
                params![series as i64],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(mal, None);
    assert_avatar_drawn(&core, series);

    // The refresh asks TVmaze again, for the one show.
    http.push_for("api.tvmaze.com/shows/83073?embed", 200, show_json());
    let job = started(&core, Call::RefreshSeries { series });
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::RefreshFinished {
                refreshed: 1,
                failed: 0
            }
        ),
        "{done:?}"
    );
    http.push_for("api.tvmaze.com/shows/83073?embed", 200, show_json());
    let job = started(&core, Call::RefreshAll);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::RefreshFinished {
                refreshed: 1,
                failed: 0
            }
        ),
        "{done:?}"
    );
    // Still confirmed: a refresh keeps the flag it found.
    assert_eq!(match_row(&core, series).3, 1);
    core.shutdown();
}

/// A TVmaze-only series has nothing to report to: every tracker write is
/// refused as unmatched, and a session's mark rule never fires, while the
/// view and the episode title still work.
#[test]
fn a_tvmaze_series_never_reaches_a_tracker_but_plays_with_its_titles() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, files) = avatar(&core);
    fixtures::connect_tracker(&core, Tracker::Anilist, 1, "token");
    push_show(&http);
    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Tvmaze { id: 83073 },
        },
    );
    common::wait_job(&c, job);

    for call in [
        Call::MarkEpisode {
            series,
            episode: 1.0,
        },
        Call::SetProgress {
            series,
            progress: 1,
        },
        Call::SetScore {
            series,
            score: Some(8.0),
        },
    ] {
        assert!(
            matches!(
                core.call(call.clone()),
                Err(CoreError::Refused {
                    reason: Refusal::NoMatch
                })
            ),
            "{call:?}"
        );
    }

    let session = match core.call(Call::OpenPlayback { file: files[1] }).unwrap() {
        Reply::Playback { session } => *session,
        other => panic!("{other:?}"),
    };
    assert_eq!(session.series_title, "Avatar: Seven Havens");
    assert_eq!(session.episode_title.as_deref(), Some("Into the Storm"));
    assert!(session.artwork.is_some());
    let requests = http.requests().len();
    // A whole episode's worth of ticks, past the 85 percent line.
    core.call(Call::ReportChapters {
        session: session.session,
        chapters: vec![],
        duration: 100.0,
    })
    .unwrap();
    for p in 0..=95 {
        core.call(Call::Tick {
            session: session.session,
            position: f64::from(p),
            paused: false,
        })
        .unwrap();
    }
    core.call(Call::ClosePlayback {
        session: session.session,
        position: 95.0,
        reason: CloseReason::Stopped,
    })
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert!(
        !c.bodies().iter().any(|b| matches!(
            b,
            EventBody::JobStarted {
                kind: JobKind::Mark
            }
        )),
        "{:#?}",
        c.bodies()
    );
    // Nothing went out to AniList, AniSkip or anyone else.
    assert_eq!(http.requests().len(), requests, "{:#?}", http.requests());
    core.shutdown();
}

/// The feed reads a TVmaze series' schedule the way it reads AniList's:
/// the newest aired episode on disk is why it is recent, and the next
/// announced one is what is coming.
#[test]
fn the_feed_reads_a_tvmaze_schedule() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, _) = avatar(&core);
    push_show(&http);
    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Tvmaze { id: 83073 },
        },
    );
    common::wait_job(&c, job);

    let feed = |sort| match core.call(Call::ListFeed { sort }).unwrap() {
        Reply::Feed { cards } => cards,
        other => panic!("{other:?}"),
    };
    let recent = feed(FeedSort::Recent);
    assert!(
        matches!(recent[0].reason, FeedReason::Aired { episode: 3, .. }),
        "{recent:#?}"
    );
    let upcoming = feed(FeedSort::Upcoming);
    assert!(
        matches!(upcoming[0].reason, FeedReason::Scheduled { episode: 4, .. }),
        "{upcoming:#?}"
    );
    // Releasing with files on disk: it sits on the airing rail.
    match core
        .call(Call::ListAiring {
            offset: 0,
            limit: 10,
        })
        .unwrap()
    {
        Reply::Series { series: cards } => assert_eq!(cards[0].id, series),
        other => panic!("{other:?}"),
    }
    core.shutdown();
}

/// A TVmaze match whose show row has no poster on disk yet is a gap like
/// any other: a read starts the fill and the fill brings the picture in.
#[test]
fn a_missing_tvmaze_poster_is_filled_like_any_other() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, _) = avatar(&core);
    http.push_for("api.tvmaze.com/shows/83073?embed", 200, show_json());
    // Both pictures fail on the match, so the show row names them and the
    // cache holds neither.
    http.push_for("1613483.jpg", 500, "");
    http.push_for("1449291.jpg", 500, "");
    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Tvmaze { id: 83073 },
        },
    );
    common::wait_job(&c, job);
    push_images(&http);

    assert!(card(&core, series).poster.is_none());
    let fill = common::wait_for(
        &c,
        |e| {
            matches!(
                e.body,
                EventBody::JobStarted {
                    kind: JobKind::FillImages
                }
            )
        },
        std::time::Duration::from_secs(5),
    )
    .job
    .unwrap()
    .id;
    common::wait_job(&c, fill);
    let d = detail(&core, series);
    assert!(d.card.poster.is_some(), "{d:?}");
    assert!(d.banner.is_some(), "{d:?}");
    let fetched: Vec<String> = http
        .requests()
        .into_iter()
        .map(|r| r.url)
        .filter(|u| u.contains("static.tvmaze.com"))
        .collect();
    assert!(fetched.iter().any(|u| u == POSTER), "{fetched:#?}");
    assert!(fetched.iter().any(|u| u == BACKGROUND), "{fetched:#?}");
    core.shutdown();
}

/// Export carries a TVmaze match as its own shape; a fresh core takes it
/// back confirmed with the id alone, and the launch's backfill fetches the
/// show behind it.
#[test]
fn a_tvmaze_match_round_trips_through_export_and_the_backfill_fills_it() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    let lib = dir.path().join("tv");
    std::fs::create_dir_all(lib.join("Avatar Seven Havens")).unwrap();
    let lib_path = lib.to_string_lossy().into_owned();
    let src = fixtures::insert_source(&core, &lib_path);
    let series = fixtures::insert_series(
        &core,
        src,
        SeriesKind::Show,
        &format!("{lib_path}/Avatar Seven Havens"),
        "Avatar Seven Havens",
    );
    push_show(&http);
    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Tvmaze { id: 83073 },
        },
    );
    common::wait_job(&c, job);

    let out = dir
        .path()
        .join("export.json")
        .to_string_lossy()
        .into_owned();
    let job = started(
        &core,
        Call::Export {
            path: out.clone(),
            private: false,
        },
    );
    common::wait_job(&c, job);
    let doc: Value = serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
    let entry = &doc["series"][0];
    assert_eq!(
        entry["match"],
        json!({ "provider": "tvmaze", "tvmazeId": 83073 })
    );
    assert_eq!(entry["title"], "Avatar: Seven Havens");
    core.shutdown();

    let http2: Arc<FakeHttp> = FakeHttp::new();
    let (_dir2, fresh, c2) = common::open_core_with_http(http2.clone());
    let job = started(&fresh, Call::Import { path: out.clone() });
    let done = common::wait_job(&c2, job);
    let EventBody::ImportFinished { summary } = done.body else {
        panic!("{done:?}")
    };
    assert_eq!(summary.matches_applied, 1);
    assert!(
        summary.fields_ignored.is_empty(),
        "{:?}",
        summary.fields_ignored
    );
    let imported: u64 = fresh
        .store()
        .read(|c| Ok(c.query_row("SELECT id FROM series", [], |r| r.get::<_, i64>(0))? as u64))
        .unwrap();
    assert_eq!(
        match_row(&fresh, imported),
        (Some("tvmaze".into()), Some(83073), None, 1)
    );
    // The id with nothing behind it yet: no show row, so no poster and no
    // synopsis until the backfill comes for it.
    let empty = detail(&fresh, imported);
    assert!(
        empty.card.poster.is_none() && empty.synopsis.is_empty(),
        "{empty:?}"
    );

    push_show(&http2);
    let job = anibeam_core::metadata::apply::backfill_stubs(&fresh);
    let done = common::wait_job(&c2, job);
    assert!(
        matches!(done.body, EventBody::RefreshFinished { refreshed: 1, .. }),
        "{done:?}"
    );
    let filled = detail(&fresh, imported);
    assert_eq!(filled.card.title, "Avatar: Seven Havens");
    assert!(
        filled.card.poster.is_some() && !filled.synopsis.is_empty(),
        "{filled:?}"
    );
    // Importing the same file again changes nothing.
    let job = started(&fresh, Call::Import { path: out });
    let done = common::wait_job(&c2, job);
    let EventBody::ImportFinished { summary } = done.body else {
        panic!("{done:?}")
    };
    assert_eq!(summary.matches_applied, 0);
    fresh.shutdown();
}

/// The airing refresh covers a running TVmaze show past its window, and
/// a finished one never.
#[test]
fn the_airing_refresh_covers_a_running_tvmaze_show() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, _) = avatar(&core);
    push_show(&http);
    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Tvmaze { id: 83073 },
        },
    );
    common::wait_job(&c, job);

    // Just fetched, so inside the window: nothing to ask.
    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::AiringRefreshed {
            series,
            updated: false
        }
    );
    let before = http.requests().len();

    // Seven hours on, with episode 4 renamed upstream: one request, and
    // the change is reported.
    core.store()
        .write(|c| {
            c.execute(
                "UPDATE tvmaze_shows SET fetched_at = fetched_at - 7 * 3600",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    let mut show: Value = serde_json::from_str(&show_json()).unwrap();
    show["_embedded"]["episodes"][3]["name"] = json!("The Faceless, Renamed");
    http.push_for("api.tvmaze.com/shows/83073?embed", 200, show.to_string());
    let job = started(&core, Call::RefreshAiring { series });
    let done = common::wait_job(&c, job);
    assert_eq!(
        done.body,
        EventBody::AiringRefreshed {
            series,
            updated: true
        }
    );
    assert_eq!(
        http.requests()
            .iter()
            .skip(before)
            .filter(|r| r.url.contains("api.tvmaze.com"))
            .count(),
        1
    );

    // Ended, it is no candidate at all.
    core.store()
        .write(|c| {
            c.execute(
                "UPDATE tvmaze_shows SET status = 'Ended', fetched_at = 0",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    let now = time::now_secs();
    let work = core
        .store()
        .read(|conn| anibeam_core::metadata::airing::candidates(conn, now))
        .unwrap();
    assert!(work.is_empty(), "{work:?}");
    // And ended, its season is complete: the total is the season's length.
    let card = card(&core, series);
    assert_eq!(card.status, Some(AiringStatus::Finished));
    assert_eq!(
        (card.total_episodes, card.total_is_estimate),
        (Some(13), false)
    );
    core.shutdown();
}

// ---------------------------------------------------------------------------
// Shows with more than one season, written straight into the store
// ---------------------------------------------------------------------------

/// A show row and its episodes with no fetch behind it. `seasons` is each
/// season's number and its episodes' airstamps, in episode order.
fn insert_show(core: &Core, id: u64, status: &str, seasons: &[(u32, Vec<i64>)]) {
    let status = status.to_string();
    let seasons = seasons.to_vec();
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT INTO tvmaze_shows (id, name, status, premiered, fetched_at) VALUES (?1, 'Two Seasons', ?2, '2024-01-01', ?3)",
                params![id as i64, status, time::now_secs()],
            )?;
            for (season, stamps) in &seasons {
                for (i, at) in stamps.iter().enumerate() {
                    c.execute(
                        "INSERT INTO tvmaze_episodes (show_id, season, number, name, airstamp) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![id as i64, season, i as i64 + 1, format!("S{season}E{}", i + 1), at],
                    )?;
                }
            }
            Ok(())
        })
        .unwrap();
}

/// A folder holding `files` as (season, episode), matched to `show`. Each
/// file's mtime is its episode's position, so none of them reads as new.
fn tv_series(core: &Core, show: u64, files: &[(u32, u32)]) -> u64 {
    let src = fixtures::insert_source(core, "/tv");
    let series = fixtures::insert_series(
        core,
        src,
        SeriesKind::Show,
        "/tv/Two Seasons",
        "Two Seasons",
    );
    for (season, number) in files {
        fixtures::insert_file(
            core,
            series,
            &format!("/tv/Two Seasons/Two.Seasons.S{season:02}E{number:02}.mkv"),
            f64::from(*number),
            Some(*season),
            "episode",
            1,
        );
    }
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE series SET provider = 'tvmaze', tvmaze_id = ?2 WHERE id = ?1",
                params![series as i64, show as i64],
            )?;
            Ok(())
        })
        .unwrap();
    series
}

fn metadata_row(core: &Core, series: u64) -> MetadataRow {
    match core
        .call(Call::ListMetadata {
            filter: MetadataFilter::All,
            query: String::new(),
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Metadata { rows, .. } => rows.into_iter().find(|r| r.series.id == series).unwrap(),
        other => panic!("{other:?}"),
    }
}

fn feed(core: &Core, sort: FeedSort) -> Vec<FeedCard> {
    match core.call(Call::ListFeed { sort }).unwrap() {
        Reply::Feed { cards } => cards,
        other => panic!("{other:?}"),
    }
}

/// Every one of `n` episodes aired, a week apart, a year ago.
fn long_ago(n: u32) -> Vec<i64> {
    let start = time::now_secs() - 365 * DAY;
    (0..i64::from(n)).map(|i| start + i * 7 * DAY).collect()
}

/// An ended show with a ten-episode season 1 and an eight-episode season
/// 2, both whole on disk: every file is judged against its own season, so
/// none of season 1's last two is flagged, and what is on disk and what is
/// expected are both counted over the two seasons. A stray S01E11 is past
/// its season's end and is the one file flagged.
#[test]
fn an_ended_multi_season_show_judges_each_file_against_its_own_season() {
    let (_dir, core, _c) = common::open_core();
    insert_show(&core, 900, "Ended", &[(1, long_ago(10)), (2, long_ago(8))]);
    let mut files: Vec<(u32, u32)> = (1..=10).map(|n| (1, n)).collect();
    files.extend((1..=8).map(|n| (2, n)));
    let series = tv_series(&core, 900, &files);

    let d = detail(&core, series);
    assert!(d.unmatched_files.is_empty(), "{:#?}", d.unmatched_files);
    assert_eq!(d.episodes.len(), 18);
    assert_eq!(
        (d.card.total_episodes, d.card.total_is_estimate),
        (Some(18), false)
    );
    assert_eq!((d.progress.total, d.progress.on_disk), (Some(18), 18));
    let row = metadata_row(&core, series);
    assert_eq!(
        (row.have, row.expected, row.extra_on_disk),
        (18, Some(18), 0)
    );

    // A stray past season 1's end.
    fixtures::insert_file(
        &core,
        series,
        "/tv/Two Seasons/Two.Seasons.S01E11.mkv",
        11.0,
        Some(1),
        "episode",
        1,
    );
    let d = detail(&core, series);
    let flagged: Vec<(Option<u32>, f64)> = d
        .unmatched_files
        .iter()
        .map(|e| (e.season, e.number))
        .collect();
    assert_eq!(flagged, vec![(Some(1), 11.0)]);
    let row = metadata_row(&core, series);
    assert_eq!(
        (row.have, row.expected, row.extra_on_disk),
        (19, Some(18), 1)
    );
    core.shutdown();
}

/// A running show with season 1 whole on disk and the first three of
/// season 2's eleven aired episodes: the estimate counts what has aired in
/// both seasons, what is expected is both seasons' announced length, and
/// the feed and the card's activity read season 2's own episodes, never
/// season 1's numbers against season 2's dates.
#[test]
fn a_running_multi_season_show_counts_every_season_on_disk() {
    let (_dir, core, _c) = common::open_core();
    let now = time::now_secs();
    // Season 2: eleven aired a day apart up to yesterday, the twelfth next
    // week.
    let mut s2: Vec<i64> = (0..11).map(|i| now - (11 - i) * DAY).collect();
    s2.push(now + 7 * DAY);
    insert_show(&core, 901, "Running", &[(1, long_ago(10)), (2, s2.clone())]);
    let mut files: Vec<(u32, u32)> = (1..=10).map(|n| (1, n)).collect();
    files.extend((1..=3).map(|n| (2, n)));
    let series = tv_series(&core, 901, &files);

    let d = detail(&core, series);
    assert!(d.unmatched_files.is_empty());
    assert_eq!(
        (d.card.total_episodes, d.card.total_is_estimate),
        (Some(21), true)
    );
    assert_eq!((d.progress.total, d.progress.estimate), (Some(21), true));
    let row = metadata_row(&core, series);
    assert_eq!((row.have, row.expected), (13, Some(22)));
    let next = d.card.next_airing.as_ref().unwrap();
    assert_eq!((next.episode, time::to_secs(next.at)), (12, s2[11]));
    // The newest thing on disk is S02E03, aired eight days ago.
    assert_eq!(time::to_secs(d.card.latest_activity_at), s2[2]);

    let recent = feed(&core, FeedSort::Recent);
    assert!(
        matches!(recent[0].reason, FeedReason::Aired { episode: 3, at } if time::to_secs(at) == s2[2]),
        "{recent:#?}"
    );
    assert_eq!(recent[0].highest_on_disk, Some(3.0));
    core.shutdown();
}

/// Season 1 whole on disk and over, season 2 announced and not yet begun:
/// the season to come is what is next, on the card and in the feed.
#[test]
fn an_announced_next_season_is_upcoming() {
    let (_dir, core, _c) = common::open_core();
    let now = time::now_secs();
    let s2: Vec<i64> = (0..8).map(|i| now + (i + 1) * 7 * DAY).collect();
    insert_show(&core, 902, "Running", &[(1, long_ago(10)), (2, s2.clone())]);
    let files: Vec<(u32, u32)> = (1..=10).map(|n| (1, n)).collect();
    let series = tv_series(&core, 902, &files);

    let c = card(&core, series);
    let next = c.next_airing.as_ref().expect("season 2 is announced");
    assert_eq!((next.episode, time::to_secs(next.at)), (1, s2[0]));
    let upcoming = feed(&core, FeedSort::Upcoming);
    assert!(
        matches!(upcoming[0].reason, FeedReason::Scheduled { episode: 1, at } if time::to_secs(at) == s2[0]),
        "{upcoming:#?}"
    );
    core.shutdown();
}

/// A pasted link to a show TVmaze does not have leaves the series' match
/// as it was: the fetch comes first, and only a show that arrived is
/// written.
#[test]
fn a_tvmaze_match_to_a_missing_show_keeps_the_old_match() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (series, _) = avatar(&core);
    push_show(&http);
    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Tvmaze { id: 83073 },
        },
    );
    common::wait_job(&c, job);
    assert_eq!(
        match_row(&core, series),
        (Some("tvmaze".into()), Some(83073), None, 1)
    );

    http.push_for("api.tvmaze.com/shows/999999?embed", 404, "");
    let job = started(
        &core,
        Call::ApplyMatch {
            series,
            target: MatchTarget::Tvmaze { id: 999_999 },
        },
    );
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            &done.body,
            EventBody::JobFailed {
                error: CoreError::Provider {
                    provider: Provider::Tvmaze,
                    status: Some(404),
                    ..
                }
            }
        ),
        "{done:?}"
    );
    assert_eq!(
        match_row(&core, series),
        (Some("tvmaze".into()), Some(83073), None, 1)
    );
    assert_avatar_drawn(&core, series);
    core.shutdown();
}

/// A match the user cleared stays out of the auto-match across a version
/// bump, while a real miss from the older version is tried again.
#[test]
fn a_cleared_match_survives_a_version_bump_and_a_v1_miss_does_not() {
    let http = FakeHttp::new();
    let (_dir, core, c) = common::open_core_with_http(http.clone());
    let (cleared, _) = avatar(&core);
    push_show(&http);
    let job = started(
        &core,
        Call::ApplyMatch {
            series: cleared,
            target: MatchTarget::Tvmaze { id: 83073 },
        },
    );
    common::wait_job(&c, job);
    core.call(Call::ClearMatch { series: cleared }).unwrap();

    let src = fixtures::insert_source(&core, "/tv2");
    let missed = fixtures::insert_series(&core, src, SeriesKind::Show, "/tv2/Zzz", "Zzz");
    fixtures::insert_file(&core, missed, "/tv2/Zzz/01.mkv", 1.0, None, "episode", 1);
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE series SET attempted_at = 100, attempt_version = 1 WHERE id = ?1",
                params![missed as i64],
            )?;
            c.execute(
                "INSERT INTO settings (key, value) VALUES ('auto_match_version', '1')
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    let before = http.requests().len();
    http.push_for("graphql.anilist.co", 200, anilist_search(json!([])));
    http.push_for("search/shows?q=Zzz", 200, "[]");

    let job = started(&core, Call::AutoMatch);
    let done = common::wait_job(&c, job);
    assert!(
        matches!(
            done.body,
            EventBody::AutoMatchFinished {
                matched: 0,
                unmatched: 1,
                ..
            }
        ),
        "{done:?}"
    );
    let urls: Vec<String> = http
        .requests()
        .into_iter()
        .skip(before)
        .map(|r| r.url)
        .collect();
    assert_eq!(urls.len(), 2, "{urls:#?}");
    assert!(urls.iter().all(|u| !u.contains("Avatar")), "{urls:#?}");
    assert_eq!(match_row(&core, cleared).0, None);
    assert!(attempted_at(&core, cleared).is_some());
    core.shutdown();
}

/// TVmaze summaries carry numeric and named entities beyond the five XML
/// ones, and text with a bare `<` in it is text, not a tag.
#[test]
fn a_summary_decodes_every_entity_and_strips_only_real_tags() {
    use anibeam_core::net::tvmaze::plain_summary;
    assert_eq!(
        plain_summary(
            "<p>Wait&hellip; it&rsquo;s &ldquo;back&rdquo; &mdash; 2&ndash;3 &lsquo;x&rsquo;</p>"
        ),
        "Wait\u{2026} it\u{2019}s \u{201c}back\u{201d} \u{2014} 2\u{2013}3 \u{2018}x\u{2019}"
    );
    assert_eq!(
        plain_summary("Caf&eacute; &#233; &#xE9; &#XE9;"),
        "Café é é é"
    );
    assert_eq!(plain_summary("Tom&#8217;s"), "Tom\u{2019}s");
    assert_eq!(plain_summary("a < b and c > d"), "a < b and c > d");
    assert_eq!(plain_summary("1 <2 and 3> 0"), "1 <2 and 3> 0");
    assert_eq!(plain_summary("<!-- note --><i>One</i></p>"), "One");
    // An entity it cannot name, or a malformed one, stays as written.
    assert_eq!(
        plain_summary("&bogus; & &#; &#xZZ; AT&T"),
        "&bogus; & &#; &#xZZ; AT&T"
    );
    // Decoding happens once: an escaped entity is not decoded twice.
    assert_eq!(plain_summary("&amp;lt;b&amp;gt;"), "&lt;b&gt;");
    assert_eq!(plain_summary("&lt;b&gt;bold&lt;/b&gt;"), "<b>bold</b>");
}
