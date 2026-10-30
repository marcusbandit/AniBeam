use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use std::sync::LazyLock;

use crate::contract::CoreError;

/// Each migration is one SQL file under sql/, listed in order. A migration
/// that has reached main is never edited; a fix is the next migration.
pub static MIGRATIONS: LazyLock<Migrations<'static>> = LazyLock::new(|| {
    Migrations::new(vec![
        M::up(include_str!("sql/0001_initial.sql")).foreign_key_check(),
        M::up(include_str!("sql/0002_tvmaze.sql")).foreign_key_check(),
        M::up(include_str!("sql/0003_season_keys.sql")).foreign_key_check(),
        M::up(include_str!("sql/0004_imdb.sql")).foreign_key_check(),
        M::up(include_str!("sql/0005_events_without_keys.sql")).foreign_key_check(),
        M::up(include_str!("sql/0006_unmatch_imported_shorts.sql")).foreign_key_check(),
        M::up(include_str!("sql/0007_endpoint_repos.sql")).foreign_key_check(),
        M::up(include_str!("sql/0008_file_fixes.sql")).foreign_key_check(),
        M::up(include_str!("sql/0009_airing_now.sql")).foreign_key_check(),
        M::up(include_str!("sql/0010_episodes_set.sql")).foreign_key_check(),
    ])
});

/// The count. Private to the crate so this `usize` never crosses the
/// contract boundary (constraint 10 forbids `usize` in any exported type).
pub(crate) const SCHEMA_VERSION: usize = 10;

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
