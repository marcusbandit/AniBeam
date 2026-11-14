use serde::{Deserialize, Serialize};

use super::{Provider, Tracker, Words};

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

pub fn tracker_name(tracker: Tracker) -> &'static str {
    match tracker {
        Tracker::Anilist => "AniList",
        Tracker::Mal => "MyAnimeList",
    }
}

pub fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Anilist => "AniList",
        Provider::Mal => "MyAnimeList",
        Provider::Tmdb => "TMDB",
        Provider::Tvmaze => "TVmaze",
    }
}

/// One enum, every fallible call returns it, and no exported code panics on
/// shell input. Third-party errors arrive as the message string. Its words for a person
/// come from the catalog (`words`, #792); `Display` is them in the current language.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, uniffi::Error)]
pub enum CoreError {
    NotFound {
        what: Entity,
        id: u64,
    },
    Invalid {
        field: String,
        message: String,
    },
    Unavailable {
        path: String,
    },
    NotConnected {
        tracker: Tracker,
    },
    Refused {
        reason: Refusal,
    },
    Provider {
        provider: Provider,
        status: Option<u32>,
        message: String,
        retry_after: Option<f64>,
    },
    Io {
        path: Option<String>,
        message: String,
    },
    Storage {
        message: String,
    },
    Keyring {
        message: String,
    },
    Unsupported {
        what: String,
    },
    Version {
        found: u32,
        supported: u32,
    },
    Internal {
        message: String,
    },
    /// A service outside the metadata providers would not do what was
    /// asked: anirss or the torrent client. `code` is a fixed word a shell
    /// branches on (anirss's `error.type`, or `unreachable`, `unauthorized`,
    /// `not_running`, `missing`), `message` is for people.
    Service {
        service: String,
        code: String,
        message: String,
    },
}

impl CoreError {
    /// The error's words: a catalog ID and its arguments, for a shell to show in the
    /// language it speaks.
    pub fn words(&self) -> Words {
        use crate::words;
        match self {
            CoreError::NotFound { what, id } => Words::new(match what {
                Entity::Source => "core-error-not-found-source",
                Entity::Series => "core-error-not-found-series",
                Entity::File => "core-error-not-found-file",
                Entity::Session => "core-error-not-found-session",
                Entity::Job => "core-error-not-found-job",
            })
            .arg("id", id.to_string()),
            // A message that ends a sentence keeps its own mark; any other gets a full stop
            CoreError::Invalid { field, message } => Words::new(
                if message
                    .trim_end()
                    .ends_with(['.', '!', '?', '。', '！', '？', '…'])
                {
                    "core-error-invalid-ended"
                } else {
                    "core-error-invalid"
                },
            )
            .arg("field", field.replace('_', " "))
            .arg("message", message.trim_end()),
            CoreError::Unavailable { path } => words!("core-error-unavailable", path = path),
            CoreError::NotConnected { tracker } => {
                words!("core-error-not-connected", tracker = tracker_name(*tracker))
            }
            CoreError::Refused { reason } => Words::new(match reason {
                Refusal::Hidden => "core-error-refused-hidden",
                Refusal::NoMatch => "core-error-refused-no-match",
                Refusal::NotNewer => "core-error-refused-not-newer",
                Refusal::Extra => "core-error-refused-extra",
                Refusal::Unmatched => "core-error-refused-unmatched",
                Refusal::OnDisk => "core-error-refused-on-disk",
            }),
            CoreError::Provider {
                provider, message, ..
            } => words!(
                "core-error-provider",
                provider = provider_name(*provider),
                message = message
            ),
            CoreError::Io { message, .. } => words!("core-error-io", message = message),
            CoreError::Storage { message } => words!("core-error-storage", message = message),
            CoreError::Keyring { message } => words!("core-error-keyring", message = message),
            CoreError::Unsupported { what } => words!("core-error-unsupported", what = what),
            CoreError::Version { found, .. } => words!("core-error-version", found = *found),
            CoreError::Internal { message } => words!("core-error-internal", message = message),
            CoreError::Service {
                service, message, ..
            } => words!("core-error-service", service = service, message = message),
        }
    }
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.words().render())
    }
}

impl std::error::Error for CoreError {}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for CoreError {
    fn from(e: rusqlite::Error) -> Self {
        CoreError::Storage {
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "sqlite")]
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

    /// True for a service or provider that could not be reached at all: a timeout, a
    /// refused connection, no route. A shell picks its explanation from this, never from
    /// the words, which are in the reader's language (#792).
    pub fn is_unreachable(&self) -> bool {
        match self {
            CoreError::Service { code, .. } => matches!(code.as_str(), "unreachable" | "timeout"),
            CoreError::Provider { status: None, .. } => true,
            _ => false,
        }
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
    fn display_names_the_thing() {
        let e = CoreError::Refused {
            reason: Refusal::OnDisk,
        };
        assert_eq!(e.to_string(), "Cannot do that: the file is only on disk.");
    }

    #[test]
    fn unreachable_comes_from_the_shape_never_the_words() {
        let service = |code: &str| CoreError::Service {
            service: "Feed".into(),
            code: code.into(),
            message: "anything at all".into(),
        };
        assert!(service("unreachable").is_unreachable());
        assert!(service("timeout").is_unreachable());
        assert!(!service("unauthorized").is_unreachable());
        let provider = |status| CoreError::Provider {
            provider: Provider::Anilist,
            status,
            message: "timed out".into(),
            retry_after: None,
        };
        assert!(provider(None).is_unreachable());
        assert!(!provider(Some(500)).is_unreachable());
    }

    #[test]
    fn an_invalid_message_keeps_its_own_ending() {
        let invalid = |m: &str| CoreError::invalid("folder_path", m).to_string();
        assert_eq!(invalid("is empty"), "Invalid folder path: is empty.");
        assert_eq!(invalid("is empty."), "Invalid folder path: is empty.");
        assert_eq!(
            invalid("is that the right folder?"),
            "Invalid folder path: is that the right folder?"
        );
        assert_eq!(
            invalid("still reading..."),
            "Invalid folder path: still reading..."
        );
    }

    #[test]
    fn refusals_speak_plain_words() {
        for reason in [Refusal::NoMatch, Refusal::NotNewer] {
            let shown = CoreError::Refused { reason }.to_string();
            assert!(!shown.contains("tracker"), "{shown}");
        }
    }
}
