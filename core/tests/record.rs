//! Provider replies become rows. One `Media` and one `Enrichment` go in;
//! a media row, a stub per neighbour, the recommendations, the relations
//! and the episode rows come out.

use std::sync::Arc;

use anibeam_core::metadata::record::{self, EpisodeRow, StubWrite};
use anibeam_core::net::anilist::{
    CharacterEdge, CharacterEdges, CharacterName, CharacterNode, CoverImage, CoverLarge, Enrichment, FuzzyDate, Image,
    Media, RecommendationEdge, RecommendationEdges, RecommendationNode, RelatedNode, RelationEdge, RelationEdges,
    StudioEdge, StudioEdges, StudioNode, TagNode, Title,
};
use anibeam_core::store::Store;
use rusqlite::Connection;

const NOW: i64 = 1_700_000_000;

fn open() -> (tempfile::TempDir, Arc<Store>) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("anibeam.db")).unwrap();
    (dir, store)
}

fn node(id: u64, romaji: &str, cover: Option<&str>) -> RelatedNode {
    RelatedNode {
        id,
        id_mal: Some(id + 1000),
        type_: Some("ANIME".into()),
        format: Some("TV".into()),
        status: Some("FINISHED".into()),
        season_year: Some(2020),
        site_url: Some(format!("https://anilist.co/anime/{id}")),
        title: Some(Title { romaji: Some(romaji.into()), english: Some(format!("{romaji} EN")), native: None }),
        cover_image: Some(CoverLarge { large: cover.map(str::to_string) }),
        ..Default::default()
    }
}

fn canned_media() -> Media {
    Media {
        id: 154_587,
        id_mal: Some(52_991),
        title: Title {
            romaji: Some("Sousou no Frieren".into()),
            english: Some("Frieren: Beyond Journey's End".into()),
            native: Some("Sousou no Frieren JP".into()),
        },
        synonyms: vec!["Frieren".into()],
        description: Some("A mage outlives her party.".into()),
        genres: vec!["Adventure".into(), "Drama".into()],
        cover_image: Some(CoverImage {
            large: Some("https://img/l.jpg".into()),
            extra_large: Some("https://img/xl.jpg".into()),
        }),
        banner_image: Some("https://img/banner.jpg".into()),
        episodes: Some(28),
        duration: Some(24),
        season: Some("FALL".into()),
        season_year: Some(2023),
        status: Some("FINISHED".into()),
        format: Some("TV".into()),
        start_date: Some(FuzzyDate { year: Some(2023), month: Some(9), day: Some(29) }),
        end_date: Some(FuzzyDate { year: Some(2024), month: Some(3), day: None }),
        average_score: Some(91),
        studios: None,
    }
}

fn canned_enrichment() -> Enrichment {
    let characters = (1..=12u64)
        .map(|i| CharacterEdge {
            role: Some(if i == 1 { "MAIN".into() } else { "SUPPORTING".into() }),
            node: CharacterNode {
                id: i,
                name: Some(CharacterName { full: Some(format!("Character {i}")) }),
                image: Some(Image {
                    large: Some(format!("https://img/c{i}-l.jpg")),
                    medium: Some(format!("https://img/c{i}-m.jpg")),
                }),
                site_url: None,
            },
        })
        .collect();
    let recommendation = |id: u64, rating: i64| RecommendationEdge {
        node: RecommendationNode {
            rating: Some(rating),
            media_recommendation: Some(node(id, &format!("Rec {id}"), Some(&format!("https://img/rec{id}.jpg")))),
        },
    };
    Enrichment {
        id: 154_587,
        id_mal: Some(52_991),
        type_: Some("ANIME".into()),
        site_url: Some("https://anilist.co/anime/154587".into()),
        tags: vec![
            TagNode {
                name: "Adventure".into(),
                rank: Some(90),
                is_media_spoiler: false,
                is_general_spoiler: false,
                is_adult: false,
                category: Some("Theme".into()),
            },
            TagNode {
                name: "Twist".into(),
                rank: Some(40),
                is_media_spoiler: false,
                is_general_spoiler: true,
                is_adult: false,
                category: None,
            },
        ],
        studios: Some(StudioEdges {
            edges: vec![
                StudioEdge { is_main: false, node: StudioNode { id: 1, name: "Aniplex".into(), is_animation_studio: false } },
                StudioEdge { is_main: true, node: StudioNode { id: 2, name: "Madhouse".into(), is_animation_studio: true } },
            ],
        }),
        characters: Some(CharacterEdges { edges: characters }),
        // Deliberately out of rating order: the top eight are the top eight
        // by rating whatever order AniList sent them in.
        recommendations: Some(RecommendationEdges {
            edges: vec![recommendation(11, 100), recommendation(12, 300), recommendation(13, 200)],
        }),
        // A CHARACTER edge is kept like any other. The crawl is what
        // refuses to walk one, not the write.
        relations: Some(RelationEdges {
            edges: vec![
                RelationEdge { relation_type: "SEQUEL".into(), node: node(2, "Sequel", Some("https://img/seq.jpg")) },
                RelationEdge { relation_type: "CHARACTER".into(), node: node(3, "Cameo", None) },
            ],
        }),
        ..Default::default()
    }
}

fn text(conn: &Connection, sql: &str) -> Option<String> {
    conn.query_row(sql, [], |r| r.get::<_, Option<String>>(0)).unwrap()
}

fn number(conn: &Connection, sql: &str) -> Option<i64> {
    conn.query_row(sql, [], |r| r.get::<_, Option<i64>>(0)).unwrap()
}

fn json(conn: &Connection, sql: &str) -> serde_json::Value {
    serde_json::from_str(&text(conn, sql).expect("a column that is never null")).unwrap()
}

#[test]
fn a_media_and_its_enrichment_become_a_row_its_stubs_its_recommendations_and_its_relations() {
    let (_dir, store) = open();
    let w = record::build(&canned_media(), Some(&canned_enrichment()));

    assert_eq!(w.studio.as_deref(), Some("Madhouse"));
    assert_eq!(w.cover_url.as_deref(), Some("https://img/xl.jpg"));
    assert_eq!(w.characters.len(), 10, "the top ten of twelve");
    assert_eq!(w.recommendations.iter().map(|(_, n, _)| n.id).collect::<Vec<_>>(), vec![12, 13, 11]);
    assert_eq!(w.relations.len(), 2);

    // Every picture the row names, in one list for the image cache.
    let urls = record::image_urls(&w);
    assert!(urls.contains(&"https://img/xl.jpg".to_string()));
    assert!(urls.contains(&"https://img/banner.jpg".to_string()));
    assert!(urls.contains(&"https://img/c1-l.jpg".to_string()));
    assert!(urls.contains(&"https://img/rec12.jpg".to_string()));

    let raw = record::raw_bundle(Some(&serde_json::json!({ "id": 154587 })), None, None, Some(&serde_json::json!([])));
    store.tx(move |tx| record::write_media(tx, &w, &raw, NOW)).unwrap();

    store
        .read(|c| {
            assert_eq!(text(c, "SELECT title_romaji FROM anilist_media WHERE id = 154587").as_deref(), Some("Sousou no Frieren"));
            assert_eq!(
                text(c, "SELECT title_english FROM anilist_media WHERE id = 154587").as_deref(),
                Some("Frieren: Beyond Journey's End")
            );
            assert_eq!(text(c, "SELECT media_type FROM anilist_media WHERE id = 154587").as_deref(), Some("ANIME"));
            assert_eq!(text(c, "SELECT synonyms FROM anilist_media WHERE id = 154587").as_deref(), Some(r#"["Frieren"]"#));
            assert_eq!(text(c, "SELECT start_date FROM anilist_media WHERE id = 154587").as_deref(), Some("2023-09-29"));
            assert_eq!(text(c, "SELECT end_date FROM anilist_media WHERE id = 154587").as_deref(), Some("2024-03-01"));
            assert_eq!(text(c, "SELECT studio FROM anilist_media WHERE id = 154587").as_deref(), Some("Madhouse"));
            assert_eq!(text(c, "SELECT site_url FROM anilist_media WHERE id = 154587").as_deref(), Some("https://anilist.co/anime/154587"));
            assert_eq!(number(c, "SELECT year FROM anilist_media WHERE id = 154587"), Some(2023));
            assert_eq!(number(c, "SELECT episodes FROM anilist_media WHERE id = 154587"), Some(28));
            assert_eq!(number(c, "SELECT duration FROM anilist_media WHERE id = 154587"), Some(24));
            assert_eq!(number(c, "SELECT average_score FROM anilist_media WHERE id = 154587"), Some(91));
            assert_eq!(number(c, "SELECT fetched_at FROM anilist_media WHERE id = 154587"), Some(NOW));
            assert_eq!(number(c, "SELECT relations_fetched_at FROM anilist_media WHERE id = 154587"), Some(NOW));
            assert_eq!(number(c, "SELECT crawl_deferred_until FROM anilist_media WHERE id = 154587"), None);

            // The two spoiler flags collapse into one, and the studio list
            // keeps everything the flat string threw away.
            let tags = json(c, "SELECT tags FROM anilist_media WHERE id = 154587");
            assert_eq!(tags[0]["name"], "Adventure");
            assert_eq!(tags[0]["spoiler"], false);
            assert_eq!(tags[1]["spoiler"], true);
            let studios = json(c, "SELECT studios FROM anilist_media WHERE id = 154587");
            assert_eq!(studios[0]["name"], "Madhouse");
            assert_eq!(studios[0]["animation"], true);
            assert_eq!(studios[1]["name"], "Aniplex");
            let characters = json(c, "SELECT characters FROM anilist_media WHERE id = 154587");
            assert_eq!(characters.as_array().unwrap().len(), 10);
            assert_eq!(characters[0]["image_url"], "https://img/c1-l.jpg");

            // The reply as fetched, all four parts, so a later migration
            // can mine it without asking AniList again.
            let raw = json(c, "SELECT raw FROM anilist_media WHERE id = 154587");
            assert_eq!(raw["media"]["id"], 154_587);
            assert!(raw["enrichment"].is_null());
            assert!(raw["schedule"].is_null());
            assert!(raw["jikan"].is_array());

            // A neighbour and a recommendation target are known by name
            // only: a stub, which the crawl and the refresh still owe a
            // fetch.
            let stubs: i64 = c
                .query_row(
                    "SELECT count(*) FROM anilist_media WHERE fetched_at IS NULL AND relations_fetched_at IS NULL",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(stubs, 5, "two relation neighbours and three recommendation targets");
            assert_eq!(text(c, "SELECT title_romaji FROM anilist_media WHERE id = 2").as_deref(), Some("Sequel"));
            assert_eq!(text(c, "SELECT title_romaji FROM anilist_media WHERE id = 3").as_deref(), Some("Cameo"));
            assert_eq!(number(c, "SELECT year FROM anilist_media WHERE id = 3"), Some(2020));
            assert_eq!(number(c, "SELECT episodes FROM anilist_media WHERE id = 3"), None);

            let recs: Vec<(i64, i64)> = c
                .prepare("SELECT recommended_id, rating FROM recommendations WHERE anilist_id = 154587 ORDER BY rank")
                .unwrap()
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            assert_eq!(recs, vec![(12, 300), (13, 200), (11, 100)]);

            let relations: Vec<(i64, String)> = c
                .prepare("SELECT to_id, relation FROM relations WHERE from_id = 154587 ORDER BY to_id")
                .unwrap()
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            assert_eq!(relations, vec![(2, "SEQUEL".to_string()), (3, "CHARACTER".to_string())]);
            Ok(())
        })
        .unwrap();
}

#[test]
fn a_stub_fills_blanks_and_never_overwrites_a_value_with_null() {
    let (_dir, store) = open();
    let w = record::build(&canned_media(), Some(&canned_enrichment()));
    let raw = record::raw_bundle(None, None, None, None);
    store.tx(move |tx| record::write_media(tx, &w, &raw, NOW)).unwrap();

    let stub = StubWrite {
        id: 2,
        mal_id: None,
        media_type: None,
        title_romaji: None,
        title_english: Some("Later English".into()),
        format: None,
        status: None,
        year: None,
        cover_url: None,
        site_url: None,
        episodes: Some(12),
        average_score: Some(77),
    };
    store.tx(move |tx| record::write_stub(tx, &stub)).unwrap();

    store
        .read(|c| {
            assert_eq!(text(c, "SELECT title_romaji FROM anilist_media WHERE id = 2").as_deref(), Some("Sequel"));
            assert_eq!(text(c, "SELECT title_english FROM anilist_media WHERE id = 2").as_deref(), Some("Sequel EN"));
            assert_eq!(number(c, "SELECT episodes FROM anilist_media WHERE id = 2"), Some(12));
            assert_eq!(number(c, "SELECT average_score FROM anilist_media WHERE id = 2"), Some(77));
            assert_eq!(number(c, "SELECT fetched_at FROM anilist_media WHERE id = 2"), None);
            Ok(())
        })
        .unwrap();
}

#[test]
fn a_refresh_promotes_a_deferred_stub_and_replaces_the_edges_it_used_to_have() {
    let (_dir, store) = open();
    let w = record::build(&canned_media(), Some(&canned_enrichment()));
    let raw = record::raw_bundle(None, None, None, None);
    store.tx(move |tx| record::write_media(tx, &w, &raw, NOW)).unwrap();

    // The crawl had reached this node as a neighbour, been rate limited on
    // it, and put it off until later.
    store
        .tx(|tx| {
            tx.execute("UPDATE anilist_media SET crawl_deferred_until = ?1 WHERE id = 2", [NOW + 3600])?;
            Ok(())
        })
        .unwrap();

    // Now the refresh fetches it properly: one relation where there were
    // two, and a recommendation that has since been voted off.
    let mut media = canned_media();
    media.id = 2;
    media.title.romaji = Some("Sequel, fetched".into());
    let mut enrichment = canned_enrichment();
    enrichment.id = 2;
    enrichment.relations = Some(RelationEdges {
        edges: vec![RelationEdge { relation_type: "PREQUEL".into(), node: node(154_587, "Frieren", None) }],
    });
    let w = record::build(&media, Some(&enrichment));
    let raw = record::raw_bundle(None, None, None, None);
    store.tx(move |tx| record::write_media(tx, &w, &raw, NOW + 60)).unwrap();

    store
        .read(|c| {
            assert_eq!(text(c, "SELECT title_romaji FROM anilist_media WHERE id = 2").as_deref(), Some("Sequel, fetched"));
            assert_eq!(number(c, "SELECT fetched_at FROM anilist_media WHERE id = 2"), Some(NOW + 60));
            assert_eq!(number(c, "SELECT relations_fetched_at FROM anilist_media WHERE id = 2"), Some(NOW + 60));
            assert_eq!(number(c, "SELECT crawl_deferred_until FROM anilist_media WHERE id = 2"), None, "the deferral is spent");

            let relations: Vec<(i64, String)> = c
                .prepare("SELECT to_id, relation FROM relations WHERE from_id = 2")
                .unwrap()
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            assert_eq!(relations, vec![(154_587, "PREQUEL".to_string())]);

            // The edges the other series drew to this one are that series'
            // rows, and this write leaves them alone.
            let inbound: i64 =
                c.query_row("SELECT count(*) FROM relations WHERE from_id = 154587", [], |r| r.get(0)).unwrap();
            assert_eq!(inbound, 2);
            Ok(())
        })
        .unwrap();
}

#[test]
fn episodes_upsert_and_the_airing_refresh_keeps_the_titles_it_has() {
    let (_dir, store) = open();
    let w = record::build(&canned_media(), Some(&canned_enrichment()));
    let raw = record::raw_bundle(None, None, None, None);
    store.tx(move |tx| record::write_media(tx, &w, &raw, NOW)).unwrap();

    let first = vec![
        EpisodeRow { number: 1, title: Some("The Journey's End".into()), aired_at: Some(NOW - 1000) },
        EpisodeRow { number: 2, title: Some("It Didn't Have to Be Magic".into()), aired_at: Some(NOW - 500) },
        EpisodeRow { number: 3, title: None, aired_at: Some(NOW + 500) },
        EpisodeRow { number: 4, title: None, aired_at: Some(NOW + 1000) },
    ];
    store.tx(move |tx| record::write_episodes(tx, 154_587, &first, false, NOW)).unwrap();

    // The airing refresh: fresh dates, no titles, and episode 4 has slipped
    // off the schedule page. A future row nobody claims goes; a past row is
    // never touched by the rewrite.
    let second = vec![
        EpisodeRow { number: 1, title: None, aired_at: Some(NOW - 900) },
        EpisodeRow { number: 2, title: None, aired_at: None },
        EpisodeRow { number: 3, title: None, aired_at: Some(NOW + 600) },
    ];
    store.tx(move |tx| record::write_episodes(tx, 154_587, &second, true, NOW)).unwrap();

    store
        .read(|c| {
            let rows: Vec<(i64, Option<String>, Option<i64>)> = c
                .prepare("SELECT number, title, aired_at FROM anilist_episodes WHERE anilist_id = 154587 ORDER BY number")
                .unwrap()
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            assert_eq!(
                rows,
                vec![
                    (1, Some("The Journey's End".to_string()), Some(NOW - 900)),
                    (2, Some("It Didn't Have to Be Magic".to_string()), Some(NOW - 500)),
                    (3, None, Some(NOW + 600)),
                ],
                "episode 4 was future and unclaimed, so it went"
            );
            Ok(())
        })
        .unwrap();

    // An empty list is a fetch that found nothing, never an instruction to
    // delete the schedule.
    store.tx(move |tx| record::write_episodes(tx, 154_587, &[], true, NOW)).unwrap();
    let kept: i64 = store.read(|c| Ok(c.query_row("SELECT count(*) FROM anilist_episodes", [], |r| r.get(0))?)).unwrap();
    assert_eq!(kept, 3);

    // Without keep_titles a fresh title wins.
    let third = vec![EpisodeRow { number: 1, title: Some("A better title".into()), aired_at: None }];
    store.tx(move |tx| record::write_episodes(tx, 154_587, &third, false, NOW)).unwrap();
    store
        .read(|c| {
            assert_eq!(
                text(c, "SELECT title FROM anilist_episodes WHERE anilist_id = 154587 AND number = 1").as_deref(),
                Some("A better title")
            );
            assert_eq!(number(c, "SELECT aired_at FROM anilist_episodes WHERE anilist_id = 154587 AND number = 1"), Some(NOW - 900));
            Ok(())
        })
        .unwrap();
}
