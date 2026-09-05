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

use rusqlite::{params, Connection};

use crate::contract::CoreError;

pub mod closure;
pub mod crawl;

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
