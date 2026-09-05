use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use std::sync::LazyLock;

use crate::contract::CoreError;

/// Each migration is one SQL file under sql/, listed in order. A migration
/// that has reached main is never edited; a fix is the next migration.
pub static MIGRATIONS: LazyLock<Migrations<'static>> = LazyLock::new(|| {
    Migrations::new(vec![
        M::up(include_str!("sql/0001_initial.sql")).foreign_key_check(),
    ])
});

/// The count. Private to the crate so this `usize` never crosses the
/// contract boundary (constraint 10 forbids `usize` in any exported type).
pub(crate) const SCHEMA_VERSION: usize = 1;

/// Refuses a database written by a newer build, then applies what is pending
/// inside rusqlite_migration's own transaction.
pub fn apply(conn: &mut Connection) -> Result<(), CoreError> {
    let found: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if found as usize > SCHEMA_VERSION {
        return Err(CoreError::Storage {
            message: format!(
                "database schema version {found} is newer than this build's {SCHEMA_VERSION}; refusing to open it"
            ),
        });
    }
    MIGRATIONS.to_latest(conn)?;
    Ok(())
}
