use rusqlite::{Connection, OptionalExtension, params};
use serde::{Serialize, de::DeserializeOwned};

use crate::contract::CoreError;

pub const PREFERENCES: &str = "preferences";
pub const SUBTITLE_DEFAULTS: &str = "subtitle_defaults";
pub const AUTO_SKIP: &str = "auto_skip";
pub const MAIN_TRACKER: &str = "main_tracker";
pub const AUTO_MATCH_VERSION: &str = "auto_match_version";
pub const WATCHING_FETCHED_AT: &str = "watching_fetched_at";
pub const PROVIDER_ORDER: &str = "provider_order";
/// When the IMDb dump was last fetched or confirmed unchanged, in Unix
/// seconds, and the `Last-Modified` it came with, sent back as
/// `If-Modified-Since` so an unchanged dump costs a 304 rather than 8 MB.
pub const IMDB_RATINGS_FETCHED_AT: &str = "imdb_ratings_fetched_at";
pub const IMDB_RATINGS_LAST_MODIFIED: &str = "imdb_ratings_last_modified";
/// When Wikidata's AniList and MAL to IMDb links were last fetched.
pub const IMDB_LINKS_FETCHED_AT: &str = "imdb_links_fetched_at";
/// The downloads settings (`downloads::settings`), secrets left out.
pub const DOWNLOADS: &str = "downloads";

pub fn get<T: DeserializeOwned>(conn: &Connection, key: &str) -> Result<Option<T>, CoreError> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get(0),
        )
        .optional()?;
    match raw {
        Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        None => Ok(None),
    }
}

pub fn set<T: Serialize>(conn: &Connection, key: &str, value: &T) -> Result<(), CoreError> {
    let json = serde_json::to_string(value)?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, json],
    )?;
    Ok(())
}

pub fn remove(conn: &Connection, key: &str) -> Result<(), CoreError> {
    conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
    Ok(())
}

pub fn get_or_default<T: DeserializeOwned + Default>(
    conn: &Connection,
    key: &str,
) -> Result<T, CoreError> {
    Ok(get(conn, key)?.unwrap_or_default())
}
