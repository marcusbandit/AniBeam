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

/// One enum, every fallible call returns it, and no exported code panics on
/// shell input. Third-party errors arrive as the message string.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    #[error("{what:?} {id} not found")]
    NotFound { what: Entity, id: u64 },
    #[error("invalid {field}: {message}")]
    Invalid { field: String, message: String },
    #[error("source path unavailable: {path}")]
    Unavailable { path: String },
    #[error("{tracker:?} is not connected")]
    NotConnected { tracker: Tracker },
    #[error("refused: {reason:?}")]
    Refused { reason: Refusal },
    #[error("{provider:?}: {message}")]
    Provider { provider: Provider, status: Option<u32>, message: String, retry_after: Option<f64> },
    #[error("io: {message}")]
    Io { path: Option<String>, message: String },
    #[error("storage: {message}")]
    Storage { message: String },
    #[error("keyring: {message}")]
    Keyring { message: String },
    #[error("unsupported: {what}")]
    Unsupported { what: String },
    #[error("export version {found} is newer than the supported {supported}")]
    Version { found: u32, supported: u32 },
    #[error("internal: {message}")]
    Internal { message: String },
}

impl From<rusqlite::Error> for CoreError {
    fn from(e: rusqlite::Error) -> Self {
        CoreError::Storage { message: e.to_string() }
    }
}

impl From<rusqlite_migration::Error> for CoreError {
    fn from(e: rusqlite_migration::Error) -> Self {
        CoreError::Storage { message: e.to_string() }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io { path: None, message: e.to_string() }
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::Internal { message: format!("json: {e}") }
    }
}

impl CoreError {
    pub fn io_at(path: impl Into<String>, e: std::io::Error) -> CoreError {
        CoreError::Io { path: Some(path.into()), message: e.to_string() }
    }

    pub fn internal(message: impl Into<String>) -> CoreError {
        CoreError::Internal { message: message.into() }
    }

    pub fn invalid(field: &str, message: impl Into<String>) -> CoreError {
        CoreError::Invalid { field: field.to_string(), message: message.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_serialise_externally_tagged() {
        let e = CoreError::NotFound { what: Entity::Series, id: 7 };
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json, serde_json::json!({ "NotFound": { "what": "Series", "id": 7 } }));
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
        let e = CoreError::Refused { reason: Refusal::OnDisk };
        assert_eq!(e.to_string(), "refused: OnDisk");
    }
}
