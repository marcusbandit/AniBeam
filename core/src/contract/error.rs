use serde::{Deserialize, Serialize};

use super::{Provider, Tracker};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum Entity {
    Source,
    Series,
    File,
    Session,
    Job,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum Refusal {
    Hidden,
    NoMatch,
    NotNewer,
    Extra,
    Unmatched,
    OnDisk,
}

/// The words a toast uses for each entity, tracker, provider and refusal; the
/// Debug names ("Anilist", "NoMatch") are for logs, not for people.
fn entity_words(what: Entity) -> &'static str {
    match what {
        Entity::Source => "folder",
        Entity::Series => "series",
        Entity::File => "file",
        Entity::Session => "playback session",
        Entity::Job => "job",
    }
}

fn tracker_name(tracker: Tracker) -> &'static str {
    match tracker {
        Tracker::Anilist => "AniList",
        Tracker::Mal => "MyAnimeList",
    }
}

fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Anilist => "AniList",
        Provider::Mal => "MyAnimeList",
        Provider::Tmdb => "TMDB",
        Provider::Tvmaze => "TVmaze",
    }
}

/// A message as a sentence: a terminal full stop, unless it already has one.
fn sentence(message: &str) -> String {
    if message.ends_with(['.', '!', '?']) {
        message.to_string()
    } else {
        format!("{message}.")
    }
}

/// The same sentences the player's `refusalWords` uses.
fn refusal_words(reason: Refusal) -> &'static str {
    match reason {
        Refusal::Hidden => "the show is hidden",
        Refusal::NoMatch => "the show has no tracker match",
        Refusal::NotNewer => "the tracker is already past this episode",
        Refusal::Extra => "this is an extra",
        Refusal::Unmatched => "the file is not matched to an episode",
        Refusal::OnDisk => "the file is only on disk",
    }
}

/// One enum, every fallible call returns it, and no exported code panics on
/// shell input. Third-party errors arrive as the message string.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    #[error("Could not find {} {id}.", entity_words(*.what))]
    NotFound { what: Entity, id: u64 },
    #[error("Invalid {}: {}", .field.replace('_', " "), sentence(.message))]
    Invalid { field: String, message: String },
    #[error("This folder is not available: {path}")]
    Unavailable { path: String },
    #[error("{} is not connected.", tracker_name(*.tracker))]
    NotConnected { tracker: Tracker },
    #[error("Cannot do that: {}.", refusal_words(*.reason))]
    Refused { reason: Refusal },
    #[error("{} did not respond: {message}", provider_name(*.provider))]
    Provider {
        provider: Provider,
        status: Option<u32>,
        message: String,
        retry_after: Option<f64>,
    },
    #[error("Could not read or write a file: {message}")]
    Io {
        path: Option<String>,
        message: String,
    },
    #[error("Could not save your library: {message}")]
    Storage { message: String },
    #[error("Could not reach the system password store: {message}")]
    Keyring { message: String },
    #[error("{what} is not supported yet.")]
    Unsupported { what: String },
    #[error("This export is from a newer AniBeam (format {found}). Update AniBeam to import it.")]
    Version { found: u32, supported: u32 },
    #[error("AniBeam hit an internal error: {message}")]
    Internal { message: String },
}

impl From<rusqlite::Error> for CoreError {
    fn from(e: rusqlite::Error) -> Self {
        CoreError::Storage {
            message: e.to_string(),
        }
    }
}

impl From<rusqlite_migration::Error> for CoreError {
    fn from(e: rusqlite_migration::Error) -> Self {
        CoreError::Storage {
            message: e.to_string(),
        }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io {
            path: None,
            message: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::Internal {
            message: format!("json: {e}"),
        }
    }
}

impl CoreError {
    /// True for the `Internal` a `serde_json` failure converts into: a
    /// reply, a row or a file whose shape was not what the code expected.
    /// The auto-match reads it to tell a malformed provider reply, which
    /// is that one series' problem, from a storage failure, which is the
    /// job's.
    pub fn is_decode_failure(&self) -> bool {
        matches!(self, CoreError::Internal { message } if message.starts_with("json: "))
    }

    /// True for a provider that answered 404. An entry the user has never
    /// added reads as one, so a first mark and a link that resolves to
    /// nothing both go through this rather than through a failure.
    pub fn is_provider_not_found(&self) -> bool {
        matches!(
            self,
            CoreError::Provider {
                status: Some(404),
                ..
            }
        )
    }

    pub fn io_at(path: impl Into<String>, e: std::io::Error) -> CoreError {
        CoreError::Io {
            path: Some(path.into()),
            message: e.to_string(),
        }
    }

    pub fn internal(message: impl Into<String>) -> CoreError {
        CoreError::Internal {
            message: message.into(),
        }
    }

    pub fn invalid(field: &str, message: impl Into<String>) -> CoreError {
        CoreError::Invalid {
            field: field.to_string(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_serialise_externally_tagged() {
        let e = CoreError::NotFound {
            what: Entity::Series,
            id: 7,
        };
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "NotFound": { "what": "Series", "id": 7 } })
        );
        let back: CoreError = serde_json::from_value(json).unwrap();
        assert_eq!(back, e);
    }

    #[test]
    fn storage_errors_convert_from_rusqlite() {
        let e: CoreError = rusqlite::Error::QueryReturnedNoRows.into();
        assert!(matches!(e, CoreError::Storage { .. }));
    }

    #[test]
    fn display_names_the_thing() {
        let e = CoreError::Refused {
            reason: Refusal::OnDisk,
        };
        assert_eq!(e.to_string(), "Cannot do that: the file is only on disk.");
    }
}
