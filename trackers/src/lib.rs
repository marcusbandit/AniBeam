//! The trackers: AniList and MAL, where their credentials live and what
//! the core does with them.
//!
//! The trackers team (#854) is this crate. It reads the metadata team below
//! it for a card after a write and the stub record a watched show gets, and
//! says a list was read again on the bus. The core hands it on as
//! `anibeam_core::trackers`.

pub mod accounts;
pub mod cache;
pub mod local;
mod mal_lookup;
pub mod move_up;
pub mod oauth;
pub mod on_hold;
pub mod plan_to_watch;
#[cfg(test)]
mod profile_scope;
mod team;
pub mod watching;
pub mod writes;

pub use team::Trackers;

use anibeam_contract::{CoreError, Provider};
use anibeam_net::HttpResponse;

/// Where the tokens live moved down to the foundation with `Services`, which
/// holds it (#854); the shells and tests still name it here.
pub use anibeam_foundation::secrets::{Secrets, StoreKind};

/// The tracker timeout and the number readers moved down to the network team,
/// whose MAL client and Airing now read them too (#854).
pub(crate) use anibeam_net::{TRACKER_TIMEOUT, as_count, as_u32};

/// The account says it is connected but there is no token behind it: an
/// AniList keyring entry the store lost, or a MAL session whose refresh has
/// failed and already said so.
pub(crate) const NO_TOKEN: &str = "no access token stored, reconnect in Settings";

/// A job's end that only the developer log keeps.
pub(crate) fn quiet(message: impl Into<String>) -> anibeam_foundation::jobs::Finished {
    anibeam_foundation::jobs::Finished {
        plain: None,
        level: anibeam_contract::Level::Debug,
        message: message.into(),
        body: anibeam_contract::EventBody::Notice,
    }
}

/// [`NO_TOKEN`] in the current language, for an error a person reads.
pub(crate) fn no_token() -> String {
    anibeam_contract::i18n::tr("core-trackers-no-token", &[])
}

/// A MAL reply's body, or the failure [`mal_failure`] makes of it. MAL's
/// replies are read here for every module that asks MAL something.
pub(crate) fn mal_json(response: &HttpResponse) -> Result<serde_json::Value, CoreError> {
    if !response.is_success() {
        return Err(mal_failure(response));
    }
    response.json()
}

/// MAL's failure with the `message` field it usually carries lifted out of
/// the body: the body is JSON, and a truncated blob of it reads as nothing
/// at all in a toast.
pub(crate) fn mal_failure(response: &HttpResponse) -> CoreError {
    let message = response
        .json::<serde_json::Value>()
        .ok()
        .and_then(|body| body["message"].as_str().map(str::to_string))
        .filter(|message| !message.is_empty())
        .unwrap_or_else(|| response.text());
    CoreError::provider(Provider::Mal, Some(u32::from(response.status)), message)
}
