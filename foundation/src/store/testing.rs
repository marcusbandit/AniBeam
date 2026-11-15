//! A migrated database in memory and the two rows the relation graph is made
//! of, for the tests of every team that reads those tables. Not part of the
//! contract and not for the app: a crate's own `#[cfg(test)]` code is all
//! that calls it, which is why it is compiled into the crate at all.

use rusqlite::{Connection, params};

use super::sql::as_i64;

/// A migrated database in memory. The store's own WAL pragma is a file
/// mode and does not apply here, but foreign keys do: a relation's other end
/// must have a row, which is what makes a missing node the defensive case it
/// is.
pub fn conn() -> Connection {
    let mut c = Connection::open_in_memory().unwrap();
    c.pragma_update(None, "foreign_keys", "ON").unwrap();
    super::migrations::apply(&mut c).unwrap();
    c
}

/// One media row. `fetched` is whether the crawl has already taken this
/// node's edges, which is what keeps it out of `owed`.
pub fn media(c: &Connection, id: u64, fetched: bool) {
    c.execute(
        "INSERT OR REPLACE INTO anilist_media (id, media_type, title_romaji, relations_fetched_at)
         VALUES (?1, 'ANIME', ?2, ?3)",
        params![as_i64(id), format!("T{id}"), fetched.then_some(1_000i64)],
    )
    .unwrap();
}

pub fn edge(c: &Connection, from: u64, to: u64, relation: &str) {
    c.execute(
        "INSERT OR REPLACE INTO relations (from_id, to_id, relation) VALUES (?1, ?2, ?3)",
        params![as_i64(from), as_i64(to), relation],
    )
    .unwrap();
}
