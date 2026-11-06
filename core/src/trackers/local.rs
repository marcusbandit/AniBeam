//! The list kept on this device (#213): a series' list status and score
//! for whoever has no tracker account, or for a series no connected tracker
//! carries (a TVmaze show, an unmatched folder). The status menu and the
//! score picker write here instead of refusing, and a card reads it where
//! no tracker entry says otherwise. Connecting a tracker later offers to
//! move it up; until the person confirms, nothing here leaves the device.
//! The snapshot and the export read the table with their own queries.

use rusqlite::{Connection, params};

use crate::contract::*;

/// A status on the list kept here. The score stays as it was.
pub fn set_status(
    conn: &Connection,
    series: u64,
    status: ListStatus,
    now: i64,
) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO local_entries (series_id, status, score, updated_at) VALUES (?1, ?2, NULL, ?3)
         ON CONFLICT(series_id) DO UPDATE SET status = excluded.status, updated_at = excluded.updated_at",
        params![series as i64, status.as_str(), now],
    )?;
    Ok(())
}

/// A score on the list kept here, None for unrated. The status stays.
pub fn set_score(
    conn: &Connection,
    series: u64,
    score: Option<f64>,
    now: i64,
) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO local_entries (series_id, status, score, updated_at) VALUES (?1, NULL, ?2, ?3)
         ON CONFLICT(series_id) DO UPDATE SET score = excluded.score, updated_at = excluded.updated_at",
        params![series as i64, score, now],
    )?;
    Ok(())
}
