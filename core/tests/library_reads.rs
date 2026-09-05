mod common;
mod fixtures;
use anibeam_core::*;
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn list_series_tabs_cards_detail_and_metadata() {
    let (dir, core, c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    let now = anibeam_core::time::now_secs();
    let frieren = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Sousou no Frieren", "Sousou no Frieren");
    for n in 1..=8 {
        fixtures::insert_file(&core, frieren, &format!("/lib/Sousou no Frieren/{n:02}.mkv"), n as f64, None, "episode", now - 100 * n);
    }
    fixtures::insert_file(&core, frieren, "/lib/Sousou no Frieren/NCOP1.mkv", 1.0, None, "extra", now);
    fixtures::insert_media(&core, 154587, Some("Sousou no Frieren"), Some("Frieren: Beyond Journey's End"), Some(28), "RELEASING", "TV", Some(91));
    fixtures::match_series(&core, frieren, Some(154587), Some(52991));
    for n in 1..=9 {
        fixtures::insert_airing(&core, 154587, n, if n <= 8 { now - 86400 * (9 - n) } else { now + 86400 });
    }
    fixtures::insert_tracker_entry(&core, "anilist", 154587, 5, "watching", Some(8.5));
    fixtures::insert_completed(&core, frieren, "5", now - 10);
    fixtures::insert_view(&core, frieren, "5", now - 10);

    let film = fixtures::insert_series(&core, src, SeriesKind::Movie, "/lib/Movies/Perfect Blue (1997).mkv", "Perfect Blue");
    fixtures::insert_file(&core, film, "/lib/Movies/Perfect Blue (1997).mkv", 1.0, None, "episode", now);
    let hidden = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Secret", "Secret");
    fixtures::insert_file(&core, hidden, "/lib/Secret/01.mkv", 1.0, None, "episode", now);
    core.call(Call::SetHidden { series: hidden, hidden: true }).unwrap();
    let missing = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Gone", "Gone");
    fixtures::mark_missing(&core, missing);

    let list = |tab: Tab| match core
        .call(Call::ListSeries { tab, query: String::new(), sort: Sort::Alpha, direction: Direction::Asc, reveal_hidden: false })
        .unwrap()
    {
        Reply::Series { series } => series,
        other => panic!("{other:?}"),
    };
    let all = list(Tab::All);
    assert_eq!(all.iter().map(|s| s.id).collect::<Vec<_>>(), vec![film, frieren]);
    assert_eq!(list(Tab::Series).len(), 1);
    assert_eq!(list(Tab::Movies)[0].id, film);
    assert_eq!(list(Tab::Hidden)[0].id, hidden);

    let card = all.iter().find(|s| s.id == frieren).unwrap();
    assert_eq!(card.title, "Sousou no Frieren");
    assert_eq!(card.code.as_deref(), Some("EP 8"));
    assert_eq!(card.episodes_on_disk, 8);
    assert_eq!(card.extras_on_disk, 1);
    assert_eq!(card.total_episodes, Some(28));
    assert_eq!(card.watched, Some(5));
    assert_eq!(card.watched_state, WatchedState::Behind);
    assert_eq!(card.community_score, Some(9.1));
    assert_eq!(card.my_score, Some(8.5));
    assert_eq!(card.list_status, Some(ListStatus::Watching));
    assert_eq!(card.next_airing.as_ref().map(|a| a.episode), Some(9));
    assert_eq!(card.status, Some(AiringStatus::Releasing));
    assert!(card.last_viewed_at.is_some());
    assert_eq!(card.match_info.as_ref().unwrap().mal_id, Some(52991));
    assert!(card.poster.is_none());
    assert_eq!(card.latest_activity_at, UNIX_EPOCH + Duration::from_secs((now - 86400) as u64));

    core.call(Call::SetPreferences { preferences: Preferences { title_language: TitleLanguage::English, ..Preferences::default() } }).unwrap();
    assert_eq!(list(Tab::All).iter().find(|s| s.id == frieren).unwrap().title, "Frieren: Beyond Journey's End");

    match core.call(Call::GetSeries { series: frieren }).unwrap() {
        Reply::SeriesDetail { detail } => {
            assert_eq!(detail.progress, ProgressLine { watched: Some(5), total: Some(28), estimate: false, on_disk: 8 });
            let six = detail.episodes.iter().find(|e| e.number == 6.0).unwrap();
            assert_eq!(detail.next_up, Some(six.file));
            assert!(six.next_up);
            assert!(detail.episodes.iter().find(|e| e.number == 5.0).unwrap().watched);
            assert!(!six.watched);
            assert_eq!(detail.episodes[0].code, "EP 1");
            assert_eq!(detail.extras.len(), 1);
            assert_eq!(detail.extras[0].code, "OP1");
            assert!(!detail.has_graph);
            assert!(detail.unmatched_files.is_empty());
        }
        other => panic!("{other:?}"),
    }

    match core.call(Call::ListMetadata { filter: MetadataFilter::All, query: String::new(), reveal_hidden: false }).unwrap() {
        Reply::Metadata { rows, counts } => {
            assert_eq!(counts, FilterCounts { all: 3, series: 2, movies: 1, missing_files: 1 });
            let r = rows.iter().find(|r| r.series.id == frieren).unwrap();
            assert_eq!((r.have, r.expected, r.extra_on_disk, r.provider), (8, Some(28), 0, Some(Provider::Anilist)));
            assert_eq!(r.alt_title.as_deref(), Some("Sousou no Frieren"));
            assert!(rows.iter().any(|r| r.series.id == missing && r.series.missing));
        }
        other => panic!("{other:?}"),
    }
    match core.call(Call::ListMetadata { filter: MetadataFilter::MissingFiles, query: String::new(), reveal_hidden: true }).unwrap() {
        Reply::Metadata { rows, counts } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(counts.all, 4);
        }
        other => panic!("{other:?}"),
    }
    match core.call(Call::ListAiring { offset: 0, limit: 10 }).unwrap() {
        Reply::Series { series } => assert_eq!(series.iter().map(|s| s.id).collect::<Vec<_>>(), vec![frieren]),
        other => panic!("{other:?}"),
    }
    assert!(matches!(core.call(Call::Lookup { path: "/lib/Sousou no Frieren".into() }).unwrap(), Reply::Lookup { series: Some(s), file: None } if s == frieren));
    assert!(matches!(core.call(Call::Lookup { path: "/lib/Sousou no Frieren/03.mkv".into() }).unwrap(), Reply::Lookup { series: Some(_), file: Some(_) }));
    assert!(matches!(core.call(Call::GetSeries { series: 9999 }), Err(CoreError::NotFound { what: Entity::Series, id: 9999 })));
    assert!(c.bodies().iter().any(|b| matches!(b, EventBody::SeriesChanged { series } if series[0].id == hidden && series[0].hidden)));
    drop(dir);
}
