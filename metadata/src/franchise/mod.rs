//! The franchise graph: the closure over the `relations` table, and the
//! crawl that fills that table in. `closure` is pure and reads one
//! connection; `crawl` is the job that asks AniList for the edges the
//! closure found were missing.

use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};

use crate::Metadata;
use anibeam_contract::*;
pub(crate) use anibeam_foundation::store::sql::{as_i64, as_u64};
use anibeam_foundation::{time, titles};

pub mod closure;
pub mod crawl;
pub mod layout;

use closure::{Closure, Node};

/// `GetFranchiseGraph`: the graph is drawn off the tables as they stand, and
/// the crawl behind it fills in whatever the walk found missing.
pub fn graph_call(metadata: &Arc<Metadata>, series: u64) -> Result<Reply, CoreError> {
    Ok(Reply::Graph {
        layout: graph(metadata, series)?,
    })
}

/// The franchise a series page draws: the closure around the series'
/// AniList id, laid out, with a card's worth of columns on every node.
///
/// Two shapes come back as no layout at all: a series with no AniList id
/// to close a graph around, and a node standing on no edge, since a graph
/// of one card is not a graph.
fn graph(metadata: &Arc<Metadata>, series: u64) -> Result<Option<FranchiseLayout>, CoreError> {
    let now = time::now_secs();
    let built = metadata.services.store.read(|conn| {
        let matched: Option<Option<i64>> = conn
            .query_row(
                "SELECT anilist_id FROM series WHERE id = ?1",
                params![as_i64(series)],
                |r| r.get(0),
            )
            .optional()?;
        let Some(matched) = matched else {
            return Err(CoreError::NotFound {
                what: Entity::Series,
                id: series,
            });
        };
        let Some(seed) = matched.map(as_u64) else {
            return Ok(None);
        };
        let closure = closure::close(conn, seed, closure::CAP, now)?;
        let layout = match closure.nodes.len() {
            0 | 1 => None,
            _ => Some(draw(conn, metadata, &closure, seed)?),
        };
        Ok(Some((seed, closure.root, layout)))
    })?;

    // The crawl runs behind whichever answer was given, the lone node
    // included: a node whose edges have never been fetched looks exactly
    // like a node that has none, and the crawl is the only thing that
    // tells the two apart. It takes a root at most once a minute, so
    // opening the same page again costs AniList nothing.
    let Some((seed, root, layout)) = built else {
        return Ok(None);
    };
    crawl::maybe_crawl_for_read(metadata, seed, root);
    Ok(layout)
}

/// The closure turned into cards. The layout settles the positions and the
/// labels; the rest is what the tables say about each node.
fn draw(
    conn: &Connection,
    metadata: &Metadata,
    closure: &Closure,
    current: u64,
) -> Result<FranchiseLayout, CoreError> {
    let layout::Plan {
        positions,
        edges,
        mut labels,
    } = layout::plan(closure, current);

    let mut nodes: Vec<GraphNode> = Vec::with_capacity(closure.nodes.len());
    for n in &closure.nodes {
        // The layout places every node it was handed, so a miss here is a
        // bug rather than a shape the data can take. Skipping the card is
        // still better than dropping the graph.
        let Some(&(x, y)) = positions.get(&n.anilist_id) else {
            continue;
        };
        nodes.push(GraphNode {
            anilist_id: n.anilist_id,
            x,
            y,
            w: layout::NODE_W,
            h: layout::NODE_H,
            title: title_of(n),
            poster: match n.cover_url.as_deref() {
                Some(url) => metadata.net.images.path_for(conn, url)?,
                None => None,
            },
            owned: owned_by(conn, n)?,
            // A status nobody recognises counts as released: a card the
            // library holds a file for is not an announcement.
            released: n.status.as_deref() != Some("NOT_YET_RELEASED"),
            format: n.format.clone(),
            year: n.year,
            relation: labels.remove(&n.anilist_id),
            list_status: list_status(conn, n.anilist_id)?,
            current: n.anilist_id == current,
            root: n.anilist_id == closure.root,
            pending: !n.relations_fetched || n.deferred_until.is_some(),
            site_url: n.site_url.clone(),
        });
    }

    Ok(FranchiseLayout {
        root: closure.root,
        nodes,
        edges,
        complete: closure.complete,
    })
}

/// What the card is called. The graph is a chart of AniList ids, so it
/// reads in AniList's own titles rather than the library's folder names,
/// and an id with no row at all still says which id it is.
fn title_of(n: &Node) -> String {
    n.title_romaji
        .clone()
        .or_else(|| n.title_english.clone())
        .unwrap_or_else(|| titles::fallback_title(n.anilist_id))
}

/// The series this node is, when the library holds it. Never a manga: the
/// library holds video, and a manga id sharing a number with a series id
/// is not the same thing at all.
fn owned_by(conn: &Connection, n: &Node) -> Result<Option<u64>, CoreError> {
    if n.media_type.as_deref() == Some("MANGA") {
        return Ok(None);
    }
    let mut stmt =
        conn.prepare_cached("SELECT id FROM series WHERE anilist_id = ?1 ORDER BY id LIMIT 1")?;
    let found: Option<i64> = stmt
        .query_row(params![as_i64(n.anilist_id)], |r| r.get(0))
        .optional()?;
    Ok(found.map(as_u64))
}

/// Where this node sits on the AniList list. The graph is AniList's own
/// map, so it reads AniList's row whichever tracker is the main one, and
/// the list kept on this device where AniList has none (#213).
fn list_status(conn: &Connection, anilist_id: u64) -> Result<Option<ListStatus>, CoreError> {
    let mut stmt = conn.prepare_cached(
        "SELECT COALESCE(
             (SELECT status FROM tracker_entries WHERE tracker = 'anilist' AND media_id = ?1 AND profile_id = 1),
             (SELECT l.status FROM local_entries l JOIN series ls ON ls.id = l.series_id WHERE ls.anilist_id = ?1 ORDER BY ls.id LIMIT 1))",
    )?;
    let found: Option<Option<String>> = stmt
        .query_row(params![as_i64(anilist_id)], |r| r.get(0))
        .optional()?;
    Ok(found.flatten().as_deref().and_then(ListStatus::from_column))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A friend's AniList entry is not the owner's, so a node only the
    /// friend has listed reads as off the list (#863).
    #[test]
    fn a_node_reads_the_owners_list_status_only() {
        let c = anibeam_foundation::store::testing::conn();
        c.execute_batch(
            "INSERT INTO profiles (id, created_at) VALUES (2, 0);
             INSERT INTO anilist_media (id, mal_id, media_type, title_romaji) VALUES (10, 110, 'ANIME', 'Owner show'), (20, 120, 'ANIME', 'Friend show');
             INSERT INTO tracker_entries (profile_id, tracker, media_id, status, fetched_at) VALUES
                 (1, 'anilist', 10, 'watching', 0), (2, 'anilist', 20, 'completed', 0);",
        )
        .unwrap();
        assert_eq!(list_status(&c, 10).unwrap(), Some(ListStatus::Watching));
        assert_eq!(list_status(&c, 20).unwrap(), None);
    }
}
