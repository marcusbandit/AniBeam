//! The `anibeam-export` document: what Electron wrote at version 1, what
//! the core writes at version 2, and the reader that takes both.
//!
//! Version 2 is version 1 plus two fields the native line owns:
//! `series[].trackChoice`, which describes the files rather than the match
//! and so survives a cleared one, and `preferences.autoSkip`. A version 1
//! document migrates in memory by leaving both None.
//!
//! Every struct carries a flattened `extra` map, so a field this version
//! has never heard of survives the read and is reported by name rather
//! than dropped in silence. The two untagged enums carry none: an untagged
//! variant with a catch-all map would swallow every document.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contract::{AutoSkip, CoreError, TrackChoice};

/// The `format` field, the only value this reader takes.
pub const FORMAT: &str = "anibeam-export";

/// The version the core writes, and the highest it reads.
pub const VERSION: u32 = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub format: String,
    pub version: u32,
    #[serde(default)]
    pub exported_at: String,
    #[serde(default)]
    pub exported_by: ExportedBy,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub sources: Vec<SourceEntry>,
    #[serde(default)]
    pub series: Vec<SeriesEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Accounts>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<History>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferences: Option<PrefsEntry>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedBy {
    #[serde(default)]
    pub app: String,
    /// `electron` from Electron, `native` from the core.
    #[serde(default)]
    pub line: String,
    #[serde(default)]
    pub version: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceEntry {
    pub path: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesEntry {
    /// `series` or `movie`; the table's `show` is the file's `series`.
    #[serde(default)]
    pub kind: String,
    /// Identity, with `kind`: a show is its folder, a film is its file.
    pub path: String,
    /// Electron's name slug, the row id here. For a human reading the
    /// file; nothing matches on it.
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub hidden: bool,
    #[serde(rename = "match")]
    pub match_: Option<MatchEntry>,
    /// Version 2. Absent in a version 1 document, and absent rather than
    /// null when the series has never had one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_choice: Option<TrackChoice>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// The one provider record a series carries, or none at all. Untagged, so
/// the variants are told apart by the `provider` string itself.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MatchEntry {
    Tracker {
        provider: TrackerName,
        #[serde(default, rename = "anilistId")]
        anilist_id: Option<u64>,
        #[serde(default, rename = "malId")]
        mal_id: Option<u64>,
    },
    Tmdb {
        provider: TmdbName,
        #[serde(rename = "tmdbId")]
        tmdb_id: u64,
        #[serde(rename = "tmdbKind")]
        tmdb_kind: String,
    },
}

/// The two tracker names, spelled as the document spells them. A typed
/// literal rather than a `String`, so the untagged enum above can tell a
/// tracker match from a TMDB one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackerName {
    #[serde(rename = "anilist")]
    Anilist,
    #[serde(rename = "mal")]
    Mal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TmdbName {
    #[serde(rename = "tmdb")]
    Tmdb,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accounts {
    /// `anilist` or `mal`: which tracker answers first.
    #[serde(default)]
    pub main: String,
    pub anilist: Option<Account>,
    pub mal: Option<Account>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub user_id: Option<u64>,
    pub username: Option<String>,
    #[serde(default)]
    pub client_id: String,
    pub client_secret: Option<String>,
    /// Empty when the store held none, which is Electron's own answer.
    #[serde(default)]
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// Electron writes a millisecond epoch number or null and the native
    /// export writes an ISO string, so this stays a value and
    /// `parse_instant` takes both.
    pub expires_at: Option<Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct History {
    #[serde(default)]
    pub views: Vec<View>,
    #[serde(default)]
    pub completed: Vec<Completed>,
    #[serde(default)]
    pub resume_points: Vec<ResumeEntry>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    /// The series' path, which is what identifies it in this document.
    pub series: String,
    /// The number of the latest episode watched, 0 for a film.
    #[serde(default)]
    pub last_episode: f64,
    pub at: Value,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Completed {
    pub series: String,
    #[serde(default)]
    pub episode: f64,
    pub at: Value,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// A resume point is keyed by series and episode when the entry has a
/// number, and by file when it has not: a film, an OP, an ED, an SP.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResumeEntry {
    Series {
        series: String,
        episode: f64,
        position: f64,
        duration: f64,
        at: Value,
    },
    File {
        file: String,
        position: f64,
        duration: f64,
        at: Value,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefsEntry {
    #[serde(default)]
    pub title_language: String,
    #[serde(default)]
    pub library_tab: String,
    #[serde(default)]
    pub library_sort: SortEntry,
    #[serde(default)]
    pub feed_sort: String,
    /// Version 2, and core-owned because the import carries it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_skip: Option<AutoSkip>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortEntry {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub direction: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// The document and every field name this version does not know, in the
/// order they appear. A file that is not this format at all, or one from a
/// newer core, fails here rather than half way through the merge.
///
/// The version check reads the raw value first, so a document from a later
/// core is refused by version rather than by whichever field it added.
pub fn parse(bytes: &[u8]) -> Result<(Document, Vec<String>), CoreError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|e| CoreError::invalid("format", format!("not JSON: {e}")))?;
    if value.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(CoreError::invalid(
            "format",
            format!("not an {FORMAT} document"),
        ));
    }
    let found = value.get("version").and_then(Value::as_u64).unwrap_or(0);
    let found = u32::try_from(found).unwrap_or(u32::MAX);
    if found > VERSION {
        return Err(CoreError::Version {
            found,
            supported: VERSION,
        });
    }
    // Version 1 migrates here by doing nothing: the fields version 2 added
    // are absent, so they read as None and the merge leaves them alone.
    let doc: Document =
        serde_json::from_value(value).map_err(|e| CoreError::invalid("format", e.to_string()))?;
    let ignored = doc.unknown_fields();
    Ok((doc, ignored))
}

/// An instant in either of the two shapes the format carries: an RFC 3339
/// string, or the millisecond epoch number `trackers.json` held. Anything
/// else is no instant rather than a failure.
pub fn parse_instant(value: &Value) -> Option<i64> {
    match value {
        Value::String(s) => ::time::OffsetDateTime::parse(
            s.trim(),
            &::time::format_description::well_known::Rfc3339,
        )
        .ok()
        .map(|t| t.unix_timestamp()),
        Value::Number(_) => value.as_f64().map(|ms| (ms / 1000.0) as i64),
        _ => None,
    }
}

/// The instant every field of this document is written with: RFC 3339, in
/// UTC, which is what the format calls an ISO 8601 string. An instant no
/// calendar can hold is the epoch rather than a failed export.
pub fn format_instant(secs: i64) -> String {
    ::time::OffsetDateTime::from_unix_timestamp(secs)
        .unwrap_or(::time::OffsetDateTime::UNIX_EPOCH)
        .format(&::time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// Every unknown field's name, one container at a time, so a name reads as
/// the path to the field: `foo`, `series[2].bar`, `accounts.anilist.baz`.
/// The two untagged enums carry no `extra` map, so an unknown field inside
/// a match or a resume point is dropped rather than named.
fn names(prefix: &str, extra: &HashMap<String, Value>, out: &mut Vec<String>) {
    let mut keys: Vec<&str> = extra.keys().map(String::as_str).collect();
    keys.sort_unstable();
    for key in keys {
        out.push(if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{prefix}.{key}")
        });
    }
}

impl Document {
    fn unknown_fields(&self) -> Vec<String> {
        let mut out = Vec::new();
        names("", &self.extra, &mut out);
        names("exportedBy", &self.exported_by.extra, &mut out);
        for (i, s) in self.sources.iter().enumerate() {
            names(&format!("sources[{i}]"), &s.extra, &mut out);
        }
        for (i, s) in self.series.iter().enumerate() {
            names(&format!("series[{i}]"), &s.extra, &mut out);
        }
        if let Some(accounts) = &self.accounts {
            names("accounts", &accounts.extra, &mut out);
            for (name, account) in [("anilist", &accounts.anilist), ("mal", &accounts.mal)] {
                if let Some(account) = account {
                    names(&format!("accounts.{name}"), &account.extra, &mut out);
                }
            }
        }
        if let Some(history) = &self.history {
            names("history", &history.extra, &mut out);
            for (i, v) in history.views.iter().enumerate() {
                names(&format!("history.views[{i}]"), &v.extra, &mut out);
            }
            for (i, v) in history.completed.iter().enumerate() {
                names(&format!("history.completed[{i}]"), &v.extra, &mut out);
            }
        }
        if let Some(preferences) = &self.preferences {
            names("preferences", &preferences.extra, &mut out);
            names(
                "preferences.librarySort",
                &preferences.library_sort.extra,
                &mut out,
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/anibeam-export-v1.json");

    #[test]
    fn the_version_1_document_parses_with_its_keys_and_no_track_choice() {
        let (doc, ignored) = parse(FIXTURE.as_bytes()).unwrap();
        assert_eq!((doc.format.as_str(), doc.version), (FORMAT, 1));
        assert_eq!(doc.exported_by.line, "electron");
        assert!(doc.private);
        assert_eq!(doc.sources.len(), 1);
        assert_eq!(doc.series.len(), 3);
        assert!(doc.series.iter().all(|s| s.track_choice.is_none()));
        assert_eq!(
            doc.keys
                .as_ref()
                .and_then(|k| k.get("tmdb"))
                .and_then(Value::as_str),
            Some("0123abcd")
        );
        let history = doc.history.as_ref().unwrap();
        assert_eq!(
            (
                history.views.len(),
                history.completed.len(),
                history.resume_points.len()
            ),
            (1, 1, 2)
        );
        assert!(doc.preferences.as_ref().unwrap().auto_skip.is_none());
        assert!(ignored.is_empty(), "{ignored:?}");

        // The three matches, one of each shape the format has.
        assert!(matches!(
            doc.series[0].match_,
            Some(MatchEntry::Tracker {
                provider: TrackerName::Anilist,
                anilist_id: Some(154587),
                mal_id: Some(52991)
            })
        ));
        assert!(matches!(
            doc.series[1].match_,
            Some(MatchEntry::Tmdb { tmdb_id: 10494, .. })
        ));
        assert!(doc.series[2].match_.is_none());
        // And the two resume points, one of each shape.
        assert!(
            matches!(history.resume_points[0], ResumeEntry::Series { episode, .. } if episode == 13.0)
        );
        assert!(
            matches!(&history.resume_points[1], ResumeEntry::File { file, .. } if file.ends_with("NCOP1.mkv"))
        );
    }

    #[test]
    fn a_newer_document_is_refused_by_version() {
        let text = FIXTURE.replace("\"version\": 1", "\"version\": 3");
        assert!(matches!(
            parse(text.as_bytes()),
            Err(CoreError::Version {
                found: 3,
                supported: VERSION
            })
        ));
    }

    #[test]
    fn a_document_of_another_format_is_invalid() {
        let text = FIXTURE.replace("anibeam-export", "something-else");
        assert!(
            matches!(parse(text.as_bytes()), Err(CoreError::Invalid { field, .. }) if field == "format")
        );
        assert!(
            matches!(parse(b"not json at all"), Err(CoreError::Invalid { field, .. }) if field == "format")
        );
    }

    #[test]
    fn unknown_fields_come_back_by_name() {
        let text = FIXTURE
            .replace(
                "\"format\": \"anibeam-export\",",
                "\"format\": \"anibeam-export\",\n  \"foo\": 1,",
            )
            .replace(
                "\"kind\": \"series\",\n      \"path\"",
                "\"kind\": \"series\",\n      \"bar\": true,\n      \"path\"",
            );
        let (_, ignored) = parse(text.as_bytes()).unwrap();
        assert!(ignored.contains(&"foo".to_string()), "{ignored:?}");
        assert!(
            ignored.contains(&"series[0].bar".to_string()),
            "{ignored:?}"
        );
    }

    #[test]
    fn an_instant_reads_from_a_string_or_a_millisecond_number() {
        assert_eq!(
            parse_instant(&Value::from("1970-01-01T00:00:10Z")),
            Some(10)
        );
        assert_eq!(
            parse_instant(&Value::from("2026-08-30T21:04:11Z")),
            Some(1_788_123_851)
        );
        assert_eq!(
            parse_instant(&Value::from(1_788_123_851_000i64)),
            Some(1_788_123_851)
        );
        assert_eq!(parse_instant(&Value::Null), None);
        assert_eq!(parse_instant(&Value::from("not a date")), None);
    }
}
