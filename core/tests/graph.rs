//! The franchise graph a series page reads: the closure the walk found,
//! turned into cards with positions on it, and the two answers a page can
//! get instead of a graph.

mod common;
mod fixtures;

use anibeam_core::*;
use rusqlite::params;

/// One media row with everything the graph draws with, and its edges
/// already taken, so nothing in these tests is owed or pending.
fn media(core: &Core, id: u64, media_type: &str, format: &str, status: &str, year: i64) {
    let (media_type, format, status) = (
        media_type.to_string(),
        format.to_string(),
        status.to_string(),
    );
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO anilist_media
                     (id, media_type, title_romaji, format, status, year, site_url, relations_fetched_at, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
                params![
                    id as i64,
                    media_type,
                    format!("T{id}"),
                    format,
                    status,
                    year,
                    format!("https://anilist.co/anime/{id}"),
                    time::now_secs(),
                ],
            )?;
            Ok(())
        })
        .unwrap();
}

fn edge(core: &Core, from: u64, to: u64, relation: &str) {
    let relation = relation.to_string();
    core.store()
        .write(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO relations (from_id, to_id, relation) VALUES (?1, ?2, ?3)",
                params![from as i64, to as i64, relation],
            )?;
            Ok(())
        })
        .unwrap();
}

fn layout_of(core: &Core, series: u64) -> Option<FranchiseLayout> {
    match core.call(Call::GetFranchiseGraph { series }).unwrap() {
        Reply::Graph { layout } => layout,
        other => panic!("expected a graph, got {other:?}"),
    }
}

fn node_of(layout: &FranchiseLayout, id: u64) -> &GraphNode {
    layout
        .nodes
        .iter()
        .find(|n| n.anilist_id == id)
        .unwrap_or_else(|| panic!("no node {id} in the layout"))
}

/// The edges the layout drew, as `from->to:relation`, smallest first.
fn edges_of(layout: &FranchiseLayout) -> Vec<String> {
    let mut v: Vec<String> = layout
        .edges
        .iter()
        .map(|e| format!("{}->{}:{}", e.from, e.to, e.relation))
        .collect();
    v.sort();
    v
}

/// A three node chain the library owns the first of, plus a cameo the walk
/// draws and refuses to follow. One row for the chain, one for the cameo.
#[test]
fn a_series_page_reads_its_chain_on_one_row_and_its_cameo_on_another() {
    let (_dir, core, _c) = common::open_core_with_http(net::FakeHttp::new());
    for (id, year) in [(1, 2001), (2, 2002), (3, 2003), (50, 1999)] {
        media(&core, id, "ANIME", "TV", "FINISHED", year);
    }
    edge(&core, 1, 2, "SEQUEL");
    edge(&core, 2, 3, "SEQUEL");
    edge(&core, 1, 50, "CHARACTER");

    let src = fixtures::insert_source(&core, "/lib");
    let series = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/one", "One");
    fixtures::match_series(&core, series, Some(1), None);
    fixtures::insert_tracker_entry(&core, "anilist", 1, 3, "completed", Some(8.0));

    let layout = layout_of(&core, series).expect("a four node graph");
    assert_eq!(layout.root, 1);
    assert_eq!(layout.nodes.len(), 4);
    assert!(layout.complete);
    assert_eq!(
        edges_of(&layout),
        vec!["1->2:SEQUEL", "1->50:CHARACTER", "2->3:SEQUEL"],
        "the cameo is drawn, never walked"
    );

    let one = node_of(&layout, 1);
    assert!(one.current, "the series' own node is the current one");
    assert!(one.root, "and the smallest member is the root");
    assert_eq!(one.owned, Some(series));
    assert_eq!(one.title, "T1");
    assert_eq!(one.list_status, Some(ListStatus::Completed));
    assert!(one.released);
    assert!(!one.pending);
    assert_eq!(one.w, 180.0);
    assert_eq!(one.h, 420.0);
    assert_eq!(one.site_url.as_deref(), Some("https://anilist.co/anime/1"));

    let cameo = node_of(&layout, 50);
    assert!(
        !cameo.pending,
        "its edges are in the table, so nothing is owed"
    );
    assert_eq!(cameo.owned, None);
    assert_eq!(cameo.relation.as_deref(), Some("Shared characters"));

    // The chain sits on one row, stepping by a column; the cameo hangs off
    // the end of the graph on a row of its own.
    let row = |id: u64| node_of(&layout, id).y;
    assert_eq!((row(1), row(2), row(3)), (0.0, 0.0, 0.0));
    assert_eq!(
        (
            node_of(&layout, 1).x,
            node_of(&layout, 2).x,
            node_of(&layout, 3).x
        ),
        (0.0, 320.0, 640.0)
    );
    assert_ne!(row(50), 0.0, "a node in no chain takes a row of its own");

    assert_eq!(node_of(&layout, 2).relation.as_deref(), Some("Sequel"));
    assert_eq!(node_of(&layout, 3).relation.as_deref(), Some("Sequel"));

    // The detail page's own flag agrees with the graph it links to.
    let Reply::SeriesDetail { detail } = core.call(Call::GetSeries { series }).unwrap() else {
        panic!("expected a detail")
    };
    assert!(detail.has_graph);
}

/// A matched series whose node sits on no edge at all: there is no graph
/// to draw, and the page is told so rather than handed a single card.
#[test]
fn a_series_whose_node_stands_alone_has_no_layout() {
    let (_dir, core, _c) = common::open_core_with_http(net::FakeHttp::new());
    media(&core, 7, "ANIME", "TV", "FINISHED", 2004);

    let src = fixtures::insert_source(&core, "/lib");
    let series = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/alone", "Alone");
    fixtures::match_series(&core, series, Some(7), None);

    assert!(layout_of(&core, series).is_none());

    let Reply::SeriesDetail { detail } = core.call(Call::GetSeries { series }).unwrap() else {
        panic!("expected a detail")
    };
    assert!(
        !detail.has_graph,
        "the detail page agrees there is nothing to open"
    );
}

/// An unmatched series has no AniList id to close a graph around, and a
/// series that is not there at all is a different answer again.
#[test]
fn an_unmatched_series_has_no_layout_and_an_unknown_one_is_not_found() {
    let (_dir, core, _c) = common::open_core_with_http(net::FakeHttp::new());
    let src = fixtures::insert_source(&core, "/lib");
    let series = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/new", "New");

    assert!(layout_of(&core, series).is_none());
    assert!(matches!(
        core.call(Call::GetFranchiseGraph { series: 9_999 }),
        Err(CoreError::NotFound {
            what: Entity::Series,
            id: 9_999
        })
    ));
}
