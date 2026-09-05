//! The franchise graph: the closure over the `relations` table, and the
//! crawl that fills that table in. `closure` is pure and reads one
//! connection; `crawl` is the job that asks AniList for the edges the
//! closure found were missing.
//!
//! The rules are Electron's, carried over one for one: the traversal set
//! and the boundary rule of `src/shared/franchise.ts`'s `closeGraph`, the
//! minute throttle and the deferral of
//! `src/main/services/franchiseCrawler.ts`, and the crawl bar's two
//! numbers from `franchiseGraph.ts`'s `getFranchiseCrawlProgress`.

use std::collections::HashMap;
use std::sync::Arc;

use rusqlite::{params, Connection, OptionalExtension};

use crate::contract::*;
use crate::core::Core;
use crate::time;

pub mod closure;
pub mod crawl;
pub mod layout;

use closure::{Closure, Node};

/// The franchise a series page draws: the closure around the series'
/// AniList id, laid out, with a card's worth of columns on every node.
///
/// Two shapes come back as no layout at all: a series with no AniList id
/// to close a graph around, and a node standing on no edge, since a graph
/// of one card is not a graph.
pub fn graph(core: &Arc<Core>, series: u64) -> Result<Option<FranchiseLayout>, CoreError> {
    let now = time::now_secs();
    let built = core.store.read(|conn| {
        let matched: Option<Option<i64>> =
            conn.query_row("SELECT anilist_id FROM series WHERE id = ?1", params![as_i64(series)], |r| r.get(0)).optional()?;
        let Some(matched) = matched else { return Err(CoreError::NotFound { what: Entity::Series, id: series }) };
        let Some(seed) = matched.map(as_u64) else { return Ok(None) };
        let closure = closure::close(conn, seed, closure::CAP, now)?;
        let layout = match closure.nodes.len() {
            0 | 1 => None,
            _ => Some(draw(conn, core, &closure, seed)?),
        };
        Ok(Some((seed, closure.root, layout)))
    })?;

    // The crawl runs behind whichever answer was given, the lone node
    // included: a node whose edges have never been fetched looks exactly
    // like a node that has none, and the crawl is the only thing that
    // tells the two apart. It takes a root at most once a minute, so
    // opening the same page again costs AniList nothing.
    let Some((seed, root, layout)) = built else { return Ok(None) };
    crawl::maybe_crawl_for_read(core, seed, root);
    Ok(layout)
}

/// The closure turned into cards. The layout settles the positions and the
/// labels; the rest is what the tables say about each node.
fn draw(conn: &Connection, core: &Core, closure: &Closure, current: u64) -> Result<FranchiseLayout, CoreError> {
    let plan = layout::plan(closure, current);
    let positions: HashMap<u64, (f64, f64)> = plan.positions.iter().map(|(id, x, y)| (*id, (*x, *y))).collect();

    let mut nodes: Vec<GraphNode> = Vec::with_capacity(closure.nodes.len());
    for n in &closure.nodes {
        // The layout places every node it was handed, so a miss here is a
        // bug rather than a shape the data can take. Skipping the card is
        // still better than dropping the graph.
        let Some(&(x, y)) = positions.get(&n.anilist_id) else { continue };
        nodes.push(GraphNode {
            anilist_id: n.anilist_id,
            x,
            y,
            w: layout::NODE_W,
            h: layout::NODE_H,
            title: title_of(n),
            poster: match n.cover_url.as_deref() {
                Some(url) => core.images.path_for(conn, url)?,
                None => None,
            },
            owned: owned_by(conn, n)?,
            // A status nobody recognises counts as released: a card the
            // library holds a file for is not an announcement.
            released: n.status.as_deref() != Some("NOT_YET_RELEASED"),
            format: n.format.clone(),
            year: n.year,
            relation: plan.labels.get(&n.anilist_id).cloned(),
            list_status: list_status(conn, n.anilist_id)?,
            current: n.anilist_id == current,
            root: n.anilist_id == closure.root,
            pending: !n.relations_fetched || n.deferred_until.is_some(),
            site_url: n.site_url.clone(),
        });
    }

    let edges = plan.edges.iter().map(|e| GraphEdge { from: e.from, to: e.to, relation: e.relation.clone() }).collect();
    Ok(FranchiseLayout { root: closure.root, nodes, edges, complete: closure.complete })
}

/// What the card is called. The graph is a chart of AniList ids, so it
/// reads in AniList's own titles rather than the library's folder names,
/// and an id with no row at all still says which id it is.
fn title_of(n: &Node) -> String {
    n.title_romaji.clone().or_else(|| n.title_english.clone()).unwrap_or_else(|| format!("AniList {}", n.anilist_id))
}

/// The series this node is, when the library holds it. Never a manga: the
/// library holds video, and a manga id sharing a number with a series id
/// is not the same thing at all.
fn owned_by(conn: &Connection, n: &Node) -> Result<Option<u64>, CoreError> {
    if n.media_type.as_deref() == Some("MANGA") {
        return Ok(None);
    }
    let mut stmt = conn.prepare_cached("SELECT id FROM series WHERE anilist_id = ?1 ORDER BY id LIMIT 1")?;
    let found: Option<i64> = stmt.query_row(params![as_i64(n.anilist_id)], |r| r.get(0)).optional()?;
    Ok(found.map(as_u64))
}

/// Where this node sits on the AniList list. The graph is AniList's own
/// map, so it reads AniList's row whichever tracker is the main one.
fn list_status(conn: &Connection, anilist_id: u64) -> Result<Option<ListStatus>, CoreError> {
    let mut stmt = conn.prepare_cached("SELECT status FROM tracker_entries WHERE tracker = 'anilist' AND media_id = ?1")?;
    let found: Option<Option<String>> = stmt.query_row(params![as_i64(anilist_id)], |r| r.get(0)).optional()?;
    Ok(found.flatten().as_deref().and_then(ListStatus::from_column))
}

/// Whether this media id sits on any edge at all, in either direction.
/// The single-id form of the set Task 10's card snapshot computes in one
/// query; a series page asks about one id, so it asks here.
pub fn has_graph(conn: &Connection, anilist_id: u64) -> Result<bool, CoreError> {
    let id = as_i64(anilist_id);
    let found: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM relations WHERE from_id = ?1 OR to_id = ?1)", params![id], |r| r.get(0))?;
    Ok(found)
}

/// An AniList id as SQLite stores it. Ids never come near the ceiling, so
/// the saturating fallback is unreachable; it is here because a cast that
/// can lose a value should say what it does with it.
pub(crate) fn as_i64(v: u64) -> i64 {
    i64::try_from(v).unwrap_or(i64::MAX)
}

/// An id back out of a column. A negative id is not a thing the schema can
/// hold, so it reads as nought rather than wrapping around.
pub(crate) fn as_u64(v: i64) -> u64 {
    u64::try_from(v).unwrap_or(0)
}

#[cfg(test)]
pub(crate) mod testing {
    use rusqlite::{params, Connection};

    /// A migrated database in memory. The store's own WAL pragma is a file
    /// mode and does not apply here, but foreign keys do: a relation's
    /// other end must have a row, which is what makes a missing node the
    /// defensive case it is.
    pub fn conn() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        c.pragma_update(None, "foreign_keys", "ON").unwrap();
        crate::store::migrations::apply(&mut c).unwrap();
        c
    }

    /// One media row. `fetched` is whether the crawl has already taken this
    /// node's edges, which is what keeps it out of `owed`.
    pub fn media(c: &Connection, id: u64, fetched: bool) {
        c.execute(
            "INSERT OR REPLACE INTO anilist_media (id, media_type, title_romaji, relations_fetched_at)
             VALUES (?1, 'ANIME', ?2, ?3)",
            params![super::as_i64(id), format!("T{id}"), fetched.then_some(1_000i64)],
        )
        .unwrap();
    }

    pub fn edge(c: &Connection, from: u64, to: u64, relation: &str) {
        c.execute(
            "INSERT OR REPLACE INTO relations (from_id, to_id, relation) VALUES (?1, ?2, ?3)",
            params![super::as_i64(from), super::as_i64(to), relation],
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;

    #[test]
    fn has_graph_answers_for_either_end_of_an_edge() {
        let c = conn();
        media(&c, 1, true);
        media(&c, 2, true);
        media(&c, 3, true);
        edge(&c, 1, 2, "SEQUEL");

        assert!(has_graph(&c, 1).unwrap());
        assert!(has_graph(&c, 2).unwrap(), "the far end of an edge has a graph too");
        assert!(!has_graph(&c, 3).unwrap());
        assert!(!has_graph(&c, 999).unwrap(), "an id with no row at all has no graph");
    }
}
