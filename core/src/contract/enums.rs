use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Provider {
    Anilist,
    Mal,
    Tmdb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Enum)]
pub enum Tracker {
    Anilist,
    Mal,
}

impl Tracker {
    /// The value every `tracker` column holds.
    pub fn as_str(self) -> &'static str {
        match self {
            Tracker::Anilist => "anilist",
            Tracker::Mal => "mal",
        }
    }

    pub fn from_column(s: &str) -> Option<Tracker> {
        match s {
            "anilist" => Some(Tracker::Anilist),
            "mal" => Some(Tracker::Mal),
            _ => None,
        }
    }

    /// The label a user reads: AniList or MAL.
    pub fn label(self) -> &'static str {
        match self {
            Tracker::Anilist => "AniList",
            Tracker::Mal => "MAL",
        }
    }
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Anilist => "anilist",
            Provider::Mal => "mal",
            Provider::Tmdb => "tmdb",
        }
    }

    pub fn from_column(s: &str) -> Option<Provider> {
        match s {
            "anilist" => Some(Provider::Anilist),
            "mal" => Some(Provider::Mal),
            "tmdb" => Some(Provider::Tmdb),
            _ => None,
        }
    }
}
