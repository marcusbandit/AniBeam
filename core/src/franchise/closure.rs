//! The closure: one connection, one seed, and the connected franchise
//! around it. Nothing here fetches anything, so a page can draw whatever
//! the tables hold while the crawl is still filling them in.
//!
//! Electron's `closeGraph` closed over relations it carried in memory and
//! fetched what it lacked. Here the `relations` table is the memory and
//! the crawl is the fetch, so this half is pure: a member whose edges are
//! not in the table yet is reported as owed, and the crawl is what turns
//! that into a request.

use std::collections::{HashMap, HashSet, VecDeque};

use rusqlite::{params, Connection, OptionalExtension};

use super::{as_i64, as_u64};
use crate::contract::CoreError;

/// The relations the walk follows. CHARACTER and OTHER are deliberately
/// missing: a shared character or a loose link is drawn, never walked, or
/// one cameo would glue every franchise it touches into a single blob.
pub const TRAVERSABLE: [&str; 11] = [
    "PREQUEL",
    "SEQUEL",
    "SIDE_STORY",
    "SPIN_OFF",
    "ALTERNATIVE",
    "PARENT",
    "CONTAINS",
    "SUMMARY",
    "COMPILATION",
    "SOURCE",
    "ADAPTATION",
];

/// How many nodes one closure will discover. Electron's number.
pub const CAP: usize = 150;

pub fn is_traversable(relation: &str) -> bool {
    TRAVERSABLE.contains(&relation)
}

/// One node as the tables hold it. A node with no `anilist_media` row at
/// all is this with nothing but its id filled in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Node {
    pub anilist_id: u64,
    pub mal_id: Option<u64>,
    pub media_type: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub year: Option<u32>,
    pub site_url: Option<String>,
    pub title_romaji: Option<String>,
    pub title_english: Option<String>,
    pub cover_url: Option<String>,
    /// Whether the crawl has taken this node's edges. False means the row
    /// carries no edges rather than that the node has none.
    pub relations_fetched: bool,
    /// A rate limit this node is still sitting out, if it is still in the
    /// future at the `now` the closure was asked for. A deferral already
    /// spent reads as none, so the crawl's fetch list is simply the members
    /// that owe their edges and carry no deferral.
    pub deferred_until: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: u64,
    pub to: u64,
    pub relation: String,
}

/// The franchise around one seed.
pub struct Closure {
    /// The smallest member id: the franchise's identity, and the node the
    /// shell anchors its rows on. Never a boundary id, however small: a
    /// crossover must not re-key the franchise it wandered into.
    pub root: u64,
    /// Every node drawn, in the order the walk found them.
    pub nodes: Vec<Node>,
    /// Every edge drawn, once per `(from, to, relation)`.
    pub edges: Vec<Edge>,
    /// The seed and everything reached across a traversable edge from an
    /// expanded member.
    pub members: HashSet<u64>,
    /// Everything else in `nodes`: reached only across a CHARACTER or an
    /// OTHER edge, drawn so the connection shows, never expanded.
    pub boundary: HashSet<u64>,
    /// Nothing owed and the cap never hit.
    pub complete: bool,
    /// The members whose edges are not in the table yet, in the order the
    /// walk reached them. A boundary node is never here: it is never
    /// expanded, so nothing is owed on its behalf.
    pub owed: Vec<u64>,
}

/// BFS from `seed` over `relations`. `now` is what a deferral is measured
/// against; `cap` stops discovery, not the walk, so the nodes already found
/// keep every edge they have.
pub fn close(conn: &Connection, seed: u64, cap: usize, now: i64) -> Result<Closure, CoreError> {
    let mut nodes: HashMap<u64, Node> = HashMap::new();
    let mut order: Vec<u64> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut seen_edges: HashSet<(u64, u64, String)> = HashSet::new();
    let mut members: HashSet<u64> = HashSet::new();
    let mut expanded: HashSet<u64> = HashSet::new();
    let mut owed: Vec<u64> = Vec::new();
    let mut queue: VecDeque<u64> = VecDeque::new();
    let mut hit_cap = false;

    // The seed is a member and is always drawn, whether or not it has a
    // row and whether or not it has a single edge.
    nodes.insert(seed, load(conn, seed, now)?);
    order.push(seed);
    members.insert(seed);
    queue.push_back(seed);

    while let Some(id) = queue.pop_front() {
        if !expanded.insert(id) {
            continue;
        }
        if !nodes[&id].relations_fetched {
            owed.push(id);
        }
        let mut stmt = conn.prepare_cached("SELECT to_id, relation FROM relations WHERE from_id = ?1 ORDER BY to_id")?;
        let rows = stmt.query_map(params![as_i64(id)], |r| Ok((as_u64(r.get::<_, i64>(0)?), r.get::<_, String>(1)?)))?;
        let rels: Vec<(u64, String)> = rows.collect::<Result<Vec<_>, _>>()?;
        for (to, relation) in rels {
            if !nodes.contains_key(&to) {
                if nodes.len() >= cap {
                    hit_cap = true;
                    continue;
                }
                nodes.insert(to, load(conn, to, now)?);
                order.push(to);
            }
            if seen_edges.insert((id, to, relation.clone())) {
                edges.push(Edge { from: id, to, relation: relation.clone() });
            }
            // A traversable edge makes the target a member even if it was
            // already drawn as a boundary node; only a member not yet
            // expanded is worth queueing.
            if is_traversable(&relation) {
                members.insert(to);
                if !expanded.contains(&to) {
                    queue.push_back(to);
                }
            }
        }
    }

    let boundary: HashSet<u64> = nodes.keys().filter(|id| !members.contains(id)).copied().collect();
    let root = members.iter().copied().min().unwrap_or(seed);
    let complete = !hit_cap && owed.is_empty();
    let nodes: Vec<Node> = order.iter().map(|id| nodes[id].clone()).collect();
    Ok(Closure { root, nodes, edges, members, boundary, complete, owed })
}

/// One node off `anilist_media`. A row that is not there at all is a node
/// the library knows only as an id, which is still worth drawing.
fn load(conn: &Connection, id: u64, now: i64) -> Result<Node, CoreError> {
    let row = conn
        .query_row(
            "SELECT mal_id, media_type, format, status, year, site_url, title_romaji, title_english, cover_url,
                    relations_fetched_at, crawl_deferred_until
             FROM anilist_media WHERE id = ?1",
            params![as_i64(id)],
            |r| {
                Ok(Node {
                    anilist_id: id,
                    mal_id: r.get::<_, Option<i64>>(0)?.map(as_u64),
                    media_type: r.get(1)?,
                    format: r.get(2)?,
                    status: r.get(3)?,
                    year: r.get::<_, Option<i64>>(4)?.and_then(|v| u32::try_from(v).ok()),
                    site_url: r.get(5)?,
                    title_romaji: r.get(6)?,
                    title_english: r.get(7)?,
                    cover_url: r.get(8)?,
                    relations_fetched: r.get::<_, Option<i64>>(9)?.is_some(),
                    deferred_until: r.get::<_, Option<i64>>(10)?.filter(|until| *until > now),
                })
            },
        )
        .optional()?;
    Ok(row.unwrap_or(Node { anilist_id: id, ..Node::default() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::franchise::testing::*;

    /// The nodes a closure drew, smallest id first.
    fn ids(closure: &Closure) -> Vec<u64> {
        let mut v: Vec<u64> = closure.nodes.iter().map(|n| n.anilist_id).collect();
        v.sort_unstable();
        v
    }

    fn sorted(set: &std::collections::HashSet<u64>) -> Vec<u64> {
        let mut v: Vec<u64> = set.iter().copied().collect();
        v.sort_unstable();
        v
    }

    /// The relation of an edge, as `from->to:relation`.
    fn edges(closure: &Closure) -> Vec<String> {
        let mut v: Vec<String> = closure.edges.iter().map(|e| format!("{}->{}:{}", e.from, e.to, e.relation)).collect();
        v.sort();
        v
    }

    #[test]
    fn character_and_other_are_the_two_relations_never_traversed() {
        assert!(is_traversable("SEQUEL"));
        assert!(is_traversable("SIDE_STORY"));
        assert!(!is_traversable("CHARACTER"));
        assert!(!is_traversable("OTHER"));
        assert!(!is_traversable("NOT_A_RELATION"));
        assert_eq!(TRAVERSABLE.len(), 11);
    }

    /// A cameo is drawn and then stopped at: node 3 hangs off a CHARACTER
    /// edge, so its own SEQUEL to 4 is never read and 4 never joins the
    /// graph. This is what keeps a crossover from gluing two franchises
    /// into one.
    #[test]
    fn a_character_neighbour_is_a_boundary_node_and_is_never_expanded() {
        let c = conn();
        for id in [1, 2, 3, 4] {
            media(&c, id, true);
        }
        edge(&c, 1, 2, "SEQUEL");
        edge(&c, 2, 3, "CHARACTER");
        edge(&c, 3, 4, "SEQUEL");

        let g = close(&c, 1, CAP, 0).unwrap();
        assert_eq!(ids(&g), vec![1, 2, 3]);
        assert_eq!(sorted(&g.members), vec![1, 2]);
        assert_eq!(sorted(&g.boundary), vec![3]);
        assert_eq!(g.root, 1);
        assert!(g.complete);
        assert!(g.owed.is_empty(), "a boundary node is never owed");
    }

    /// The same rule for OTHER, and the root taken from the members alone:
    /// the boundary node has the smaller id and must not re-key the
    /// franchise.
    #[test]
    fn an_other_neighbour_is_a_boundary_node_and_never_the_root() {
        let c = conn();
        for id in [5, 10, 11] {
            media(&c, id, true);
        }
        edge(&c, 10, 5, "OTHER");
        edge(&c, 10, 11, "SEQUEL");

        let g = close(&c, 10, CAP, 0).unwrap();
        assert_eq!(ids(&g), vec![5, 10, 11]);
        assert_eq!(g.root, 10, "the root is the smallest member, not the smaller boundary id");
        assert_eq!(sorted(&g.boundary), vec![5]);
    }

    #[test]
    fn a_smaller_character_neighbour_is_drawn_but_never_becomes_the_root() {
        let c = conn();
        for id in [2, 100, 101] {
            media(&c, id, true);
        }
        edge(&c, 100, 101, "SEQUEL");
        edge(&c, 100, 2, "CHARACTER");

        let g = close(&c, 100, CAP, 0).unwrap();
        assert_eq!(ids(&g), vec![2, 100, 101]);
        assert_eq!(g.root, 100);
        assert_eq!(sorted(&g.boundary), vec![2]);
    }

    #[test]
    fn a_fully_traversable_graph_has_no_boundary_at_all() {
        let c = conn();
        for id in [1, 2, 3] {
            media(&c, id, true);
        }
        edge(&c, 1, 2, "SEQUEL");
        edge(&c, 2, 3, "SIDE_STORY");

        let g = close(&c, 1, CAP, 0).unwrap();
        assert_eq!(ids(&g), vec![1, 2, 3]);
        assert!(g.boundary.is_empty());
        assert!(g.complete);
    }

    /// The cap stops discovery rather than the walk: the nodes already
    /// found keep their edges, and the closure says it is not complete.
    #[test]
    fn the_cap_stops_discovery_and_leaves_the_closure_incomplete() {
        let c = conn();
        for id in [1, 2, 3, 4] {
            media(&c, id, true);
        }
        edge(&c, 1, 2, "SEQUEL");
        edge(&c, 1, 3, "SEQUEL");
        edge(&c, 1, 4, "SEQUEL");

        let g = close(&c, 1, 2, 0).unwrap();
        assert_eq!(ids(&g), vec![1, 2]);
        assert!(!g.complete);
        assert!(g.owed.is_empty(), "nothing owes its edges; the cap alone is why this is incomplete");
    }

    /// The primary key already forbids a duplicate row, so what this pins
    /// is that a triple written from two seeds' walks is still one edge in
    /// the closure, and that a reciprocal edge is its own.
    #[test]
    fn an_edge_is_drawn_once_per_triple() {
        let c = conn();
        media(&c, 1, true);
        media(&c, 2, true);
        edge(&c, 1, 2, "SEQUEL");
        edge(&c, 1, 2, "SEQUEL");
        edge(&c, 2, 1, "PREQUEL");

        let g = close(&c, 1, CAP, 0).unwrap();
        assert_eq!(ids(&g), vec![1, 2]);
        assert_eq!(edges(&g), vec!["1->2:SEQUEL".to_string(), "2->1:PREQUEL".to_string()]);
    }

    /// A member whose edges the crawl has never taken is owed, and a
    /// closure with anything owed is not complete however well the walk
    /// itself went.
    #[test]
    fn a_member_that_owes_its_edges_is_owed_and_the_closure_is_not_complete() {
        let c = conn();
        media(&c, 1, true);
        media(&c, 2, false);
        media(&c, 3, true);
        edge(&c, 1, 2, "SEQUEL");
        edge(&c, 1, 3, "CHARACTER");

        let g = close(&c, 1, CAP, 0).unwrap();
        assert_eq!(g.owed, vec![2]);
        assert!(!g.complete);
        assert_eq!(sorted(&g.boundary), vec![3], "the boundary node owes nothing: it is never expanded");
    }

    /// A deferral in the future rides along on the node so the crawl can
    /// leave it alone; the member still owes its edges, so the closure is
    /// still incomplete. A deferral already spent reads as none.
    #[test]
    fn a_deferral_rides_along_only_while_it_is_still_in_the_future() {
        let c = conn();
        media(&c, 1, true);
        media(&c, 2, false);
        edge(&c, 1, 2, "SEQUEL");
        c.execute("UPDATE anilist_media SET crawl_deferred_until = 500 WHERE id = 2", []).unwrap();

        let g = close(&c, 1, CAP, 100).unwrap();
        let two = g.nodes.iter().find(|n| n.anilist_id == 2).unwrap();
        assert_eq!(two.deferred_until, Some(500));
        assert!(!two.relations_fetched);
        assert_eq!(g.owed, vec![2], "a deferred member is still owed");
        assert!(!g.complete);

        let g = close(&c, 1, CAP, 900).unwrap();
        let two = g.nodes.iter().find(|n| n.anilist_id == 2).unwrap();
        assert_eq!(two.deferred_until, None, "a deferral already past is no deferral");
    }

    /// A seed with no media row at all is still drawn, as a stub carrying
    /// nothing but its id, and it owes its edges.
    #[test]
    fn a_node_with_no_row_is_drawn_as_a_bare_stub() {
        let c = conn();
        let g = close(&c, 999, CAP, 0).unwrap();
        assert_eq!(ids(&g), vec![999]);
        assert_eq!(g.root, 999);
        assert_eq!(g.nodes[0], Node { anilist_id: 999, ..Node::default() });
        assert_eq!(g.owed, vec![999]);
        assert!(!g.complete);
    }

    /// The node's own row is what the closure draws with: the titles, the
    /// year, the cover and the AniList address all come off `anilist_media`.
    #[test]
    fn a_node_carries_the_columns_a_card_draws_with() {
        let c = conn();
        media(&c, 1, true);
        c.execute(
            "UPDATE anilist_media SET mal_id = 77, format = 'TV', status = 'FINISHED', year = 2020,
                    site_url = 'https://anilist.co/anime/1', title_english = 'One', cover_url = 'https://img/1.jpg'
             WHERE id = 1",
            [],
        )
        .unwrap();

        let g = close(&c, 1, CAP, 0).unwrap();
        assert_eq!(
            g.nodes[0],
            Node {
                anilist_id: 1,
                mal_id: Some(77),
                media_type: Some("ANIME".to_string()),
                format: Some("TV".to_string()),
                status: Some("FINISHED".to_string()),
                year: Some(2020),
                site_url: Some("https://anilist.co/anime/1".to_string()),
                title_romaji: Some("T1".to_string()),
                title_english: Some("One".to_string()),
                cover_url: Some("https://img/1.jpg".to_string()),
                relations_fetched: true,
                deferred_until: None,
            }
        );
    }
}
