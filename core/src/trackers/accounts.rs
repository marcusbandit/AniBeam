//! A tracker account: the credentials the core signs in with, the row that
//! says who is signed in, and the token every tracker call carries.
//!
//! Two things shape this file. Secrets never enter the database, so an
//! account is split in half: the `tracker_accounts` row holds the public
//! side and a `secret_store` column saying which store the private side
//! landed in, and the tokens themselves live behind `Secrets`. And every
//! keyring call is synchronous underneath, so a call arm runs one on the
//! shell's calling thread while a job runs one through `spawn_blocking`;
//! `with_secrets` below is that second path, and every `async fn` here
//! goes through it.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rusqlite::{params, Connection, OptionalExtension};

use crate::contract::*;
use crate::core::Core;
use crate::library::cards;
use crate::prefs;
use crate::time;
use crate::trackers::secrets::{Secrets, StoreKind};

/// The one url the code exchange and the refresh both post to.
pub(crate) const MAL_TOKEN_URL: &str = "https://myanimelist.net/v1/oauth2/token";

/// How long before an access token expires the refresh happens. MAL's
/// tokens last a month, so an hour is a wide margin and costs one request.
const REFRESH_WINDOW: Duration = Duration::from_secs(60 * 60);

/// The one line a user gets when the refresh did not work. Nothing the
/// core can do about it: the tokens are gone and the flow has to be run
/// again.
const MAL_EXPIRED: &str = "MAL auth expired, reconnect in Settings.";

/// MAL is a confidential client, so it needs the secret as well as the id.
/// AniList's implicit grant needs none.
const MAL_NEEDS_SECRET: &str = "MyAnimeList needs a client secret. Paste it in the Trackers tab.";

// Keys and bundled credentials ----------------------------------------------

/// Where a tracker's access token lives, in the keyring's flat key space.
pub(crate) fn access_key(t: Tracker) -> String {
    format!("{}.access_token", t.as_str())
}

pub(crate) fn refresh_key(t: Tracker) -> String {
    format!("{}.refresh_token", t.as_str())
}

pub(crate) fn secret_key(t: Tracker) -> String {
    format!("{}.client_secret", t.as_str())
}

/// The client id compiled into this build, if there was one to compile in.
/// Set at build time the way Electron's Vite env was, so a packaged
/// AniBeam signs in with the project's own app registration and nobody has
/// to make one; a build without them still works, the user just pastes
/// their own in the Trackers tab. An id set to nothing is no id.
pub fn bundled_client_id(t: Tracker) -> Option<&'static str> {
    match t {
        Tracker::Anilist => option_env!("ANIBEAM_ANILIST_CLIENT_ID"),
        Tracker::Mal => option_env!("ANIBEAM_MAL_CLIENT_ID"),
    }
    .filter(|id| !id.is_empty())
}

/// The same for the client secret, which only MAL has one of.
pub fn bundled_client_secret(t: Tracker) -> Option<&'static str> {
    match t {
        Tracker::Anilist => None,
        Tracker::Mal => option_env!("ANIBEAM_MAL_CLIENT_SECRET"),
    }
    .filter(|secret| !secret.is_empty())
}

// The row --------------------------------------------------------------------

/// The public half of an account, straight off `tracker_accounts`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    pub user_id: Option<u64>,
    pub username: Option<String>,
    pub client_id: Option<String>,
    pub expires_at: Option<SystemTime>,
    pub connected_at: Option<SystemTime>,
    pub synced_at: Option<SystemTime>,
    pub progress_fetched_at: Option<SystemTime>,
    /// Which store the last write of this tracker's secrets landed in, so
    /// the read after it goes straight to the right one.
    pub secret_store: Option<StoreKind>,
}

impl Row {
    /// The stored client id, treating an empty string as none: a row can
    /// exist with nothing in the column, and "" is not a credential.
    fn stored_client_id(&self) -> Option<String> {
        self.client_id.clone().filter(|id| !id.is_empty())
    }
}

pub fn load_row(conn: &Connection, t: Tracker) -> Result<Option<Row>, CoreError> {
    let row = conn
        .query_row(
            "SELECT user_id, username, client_id, expires_at, connected_at, synced_at, progress_fetched_at, secret_store
             FROM tracker_accounts WHERE tracker = ?1",
            params![t.as_str()],
            |r| {
                Ok(Row {
                    user_id: r.get::<_, Option<i64>>(0)?.map(|id| id as u64),
                    username: r.get(1)?,
                    client_id: r.get(2)?,
                    expires_at: time::opt_from_secs(r.get(3)?),
                    connected_at: time::opt_from_secs(r.get(4)?),
                    synced_at: time::opt_from_secs(r.get(5)?),
                    progress_fetched_at: time::opt_from_secs(r.get(6)?),
                    secret_store: r.get::<_, Option<String>>(7)?.as_deref().and_then(StoreKind::from_column),
                })
            },
        )
        .optional()?;
    Ok(row)
}

// What the shell is told -----------------------------------------------------

/// Both accounts and which of them is the main one. Reads the keyring for
/// the client secrets, so this belongs on a calling thread; a job asks
/// through `state_async`.
pub fn state(core: &Core) -> Result<TrackerState, CoreError> {
    let (main, anilist, mal) = core
        .store
        .read(|c| Ok((prefs::load_main_tracker(c)?, load_row(c, Tracker::Anilist)?, load_row(c, Tracker::Mal)?)))?;
    Ok(TrackerState {
        main,
        anilist: account(core, Tracker::Anilist, anilist),
        mal: account(core, Tracker::Mal, mal),
    })
}

/// `state` from inside a job. The keyring is synchronous underneath, so it
/// runs on the blocking pool rather than on a tokio worker.
pub(crate) async fn state_async(core: &Arc<Core>) -> Result<TrackerState, CoreError> {
    let owner = core.clone();
    joined(core.handle.spawn_blocking(move || state(&owner)).await)?
}

fn account(core: &Core, t: Tracker, row: Option<Row>) -> TrackerAccount {
    let row = row.unwrap_or_default();
    let bundled_id = bundled_client_id(t);
    let bundled_secret = bundled_client_secret(t);
    TrackerAccount {
        // The row survives a disconnect with its credentials, so what says
        // an account is connected is the instant it connected.
        connected: row.connected_at.is_some(),
        username: row.username.clone(),
        user_id: row.user_id,
        expires_at: row.expires_at,
        last_sync: row.synced_at,
        client_id: row.stored_client_id().or_else(|| bundled_id.map(str::to_string)).unwrap_or_default(),
        has_client_secret: core.secrets().has(&secret_key(t)) || bundled_secret.is_some(),
        // Whether this build can sign in with nothing pasted at all, which
        // for MAL means both halves.
        bundled_credentials: match t {
            Tracker::Anilist => bundled_id.is_some(),
            Tracker::Mal => bundled_id.is_some() && bundled_secret.is_some(),
        },
    }
}

/// What a flow actually signs in with: the pasted credentials if there are
/// any, the bundled ones otherwise. Fails on its own preconditions, so a
/// connect with nothing to connect with never becomes a job.
pub fn effective_credentials(core: &Core, t: Tracker) -> Result<(String, Option<String>), CoreError> {
    let row = core.store.read(|c| load_row(c, t))?.unwrap_or_default();
    let client_id = row
        .stored_client_id()
        .or_else(|| bundled_client_id(t).map(str::to_string))
        .ok_or_else(|| CoreError::invalid("client_id", format!("No client ID set for {}.", t.label())))?;
    let client_secret = core
        .secrets()
        .get(&secret_key(t), row.secret_store)?
        .map(|(value, _)| value)
        .or_else(|| bundled_client_secret(t).map(str::to_string));
    if t == Tracker::Mal && client_secret.is_none() {
        return Err(CoreError::invalid("client_secret", MAL_NEEDS_SECRET));
    }
    Ok((client_id, client_secret))
}

/// `effective_credentials` from inside a job, on the blocking pool for the
/// keyring's sake.
pub(crate) async fn credentials_async(core: &Arc<Core>, t: Tracker) -> Result<(String, Option<String>), CoreError> {
    let owner = core.clone();
    joined(core.handle.spawn_blocking(move || effective_credentials(&owner, t)).await)?
}

// The calls ------------------------------------------------------------------

/// The Trackers tab's Save. The id is stored whether or not the tracker is
/// connected; an empty or absent secret keeps whatever is already stored,
/// so a user re-saving an id does not have to paste the secret again.
pub fn set_credentials(core: &Core, t: Tracker, client_id: &str, client_secret: Option<&str>) -> Result<Reply, CoreError> {
    let client_id = client_id.trim().to_string();
    let secret = client_secret.map(str::trim).filter(|s| !s.is_empty());
    let store = match secret {
        Some(secret) => Some(core.secrets().set(&secret_key(t), secret)?),
        None => None,
    };
    core.store.tx(move |tx| {
        tx.execute(
            "INSERT INTO tracker_accounts (tracker, client_id) VALUES (?1, ?2)
             ON CONFLICT(tracker) DO UPDATE SET client_id = excluded.client_id",
            params![t.as_str(), client_id],
        )?;
        // Only when a secret was actually written: the column is the one
        // record of where this tracker's secrets are, and a save that
        // wrote none must not claim to have moved them.
        if let Some(store) = store {
            tx.execute("UPDATE tracker_accounts SET secret_store = ?2 WHERE tracker = ?1", params![t.as_str(), store.as_str()])?;
        }
        Ok(())
    })?;
    emit_trackers_changed(core)?;
    Ok(Reply::Ok)
}

/// Signs out. The credentials survive by decision: a user who disconnects
/// to fix a bad token should not have to paste their client id back in.
/// What goes is the account, the progress cache and the tokens.
pub fn disconnect(core: &Core, t: Tracker) -> Result<Reply, CoreError> {
    core.store.tx(move |tx| {
        tx.execute(
            "UPDATE tracker_accounts SET user_id = NULL, username = NULL, expires_at = NULL, connected_at = NULL,
                    synced_at = NULL, progress_fetched_at = NULL
             WHERE tracker = ?1",
            params![t.as_str()],
        )?;
        tx.execute("DELETE FROM tracker_entries WHERE tracker = ?1", params![t.as_str()])?;
        Ok(())
    })?;
    let secrets = core.secrets();
    // Both are attempted whatever the first one says: a token left behind
    // is a token that would be used again.
    let access = secrets.delete(&access_key(t));
    let refresh = secrets.delete(&refresh_key(t));
    access.and(refresh)?;
    emit_trackers_changed(core)?;
    // Every card whose match carries this tracker's id loses the numbers
    // that came off it.
    emit_series_changed(core, Some(t), format!("{} disconnected", t.label()))?;
    Ok(Reply::Ok)
}

/// Which tracker answers first. Every matched series can change what it
/// shows, since the numbers now come off the other account.
pub fn set_main(core: &Core, t: Tracker) -> Result<Reply, CoreError> {
    core.store.write(move |c| prefs::save_main_tracker(c, t))?;
    emit_trackers_changed(core)?;
    emit_series_changed(core, None, format!("main tracker is {}", t.label()))?;
    Ok(Reply::Ok)
}

// The token ------------------------------------------------------------------

/// The token every tracker call carries, or `None` when there is nothing
/// to carry: not connected, or a MAL session whose refresh has failed. The
/// caller turns that `None` into the refusal the shell sees.
///
/// Async because a job is the only caller: the keyring reads and writes go
/// through the blocking pool.
pub async fn access_token(core: &Arc<Core>, t: Tracker) -> Result<Option<String>, CoreError> {
    let row = core.store.write_async(move |c| load_row(c, t)).await?.unwrap_or_default();
    let hint = row.secret_store;
    let token = with_secrets(core, move |s| s.get(&access_key(t), hint)).await?.map(|(value, _)| value);
    let Some(token) = token else { return Ok(None) };
    // AniList's implicit-grant tokens cannot be refreshed at all; they are
    // good for a year and the flow is run again after that.
    if t != Tracker::Mal || !due_for_refresh(row.expires_at, time::now()) {
        return Ok(Some(token));
    }
    let refresh = with_secrets(core, move |s| s.get(&refresh_key(t), hint)).await?.map(|(value, _)| value);
    let Some(refresh) = refresh else { return Ok(Some(token)) };
    match refresh_mal(core, &refresh).await {
        Ok(fresh) => Ok(Some(fresh)),
        Err(e) => {
            tracing::debug!("the MAL refresh failed: {e}");
            core.bus.warn(Stage::Trackers, MAL_EXPIRED, EventBody::Notice);
            Ok(None)
        }
    }
}

/// Whether a token is close enough to its expiry to be worth replacing.
/// An unknown expiry is left alone: a token that still works is worth more
/// than a refresh nothing asked for.
fn due_for_refresh(expires_at: Option<SystemTime>, now: SystemTime) -> bool {
    expires_at.is_some_and(|at| at <= now + REFRESH_WINDOW)
}

/// The refresh grant. MAL hands back a whole new pair, so both tokens and
/// the new expiry are stored before the caller sees the access token.
async fn refresh_mal(core: &Arc<Core>, refresh: &str) -> Result<String, CoreError> {
    let (client_id, client_secret) = credentials_async(core, Tracker::Mal).await?;
    let form = vec![
        ("client_id".to_string(), client_id),
        ("client_secret".to_string(), client_secret.unwrap_or_default()),
        ("grant_type".to_string(), "refresh_token".to_string()),
        ("refresh_token".to_string(), refresh.to_string()),
    ];
    let response = core.mal.post_form(MAL_TOKEN_URL, form).await?;
    if !response.is_success() {
        return Err(CoreError::Provider {
            provider: Provider::Mal,
            status: Some(u32::from(response.status)),
            message: response.text(),
            retry_after: None,
        });
    }
    let tokens = parse_tokens(&response.body, time::now())?;
    let store = save_tokens(core, Tracker::Mal, &tokens).await?;
    let expires_at = time::opt_to_secs(tokens.expires_at);
    core.store
        .write_async(move |c| {
            c.execute(
                "UPDATE tracker_accounts SET expires_at = ?2, secret_store = ?3 WHERE tracker = ?1",
                params![Tracker::Mal.as_str(), expires_at, store.as_str()],
            )?;
            Ok(())
        })
        .await?;
    Ok(tokens.access_token)
}

// Tokens ---------------------------------------------------------------------

/// What a token endpoint hands back. AniList's implicit grant fills the
/// same shape from the callback's query.
pub(crate) struct Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<SystemTime>,
}

/// A token reply. `expires_in` is seconds from now, so it is turned into
/// the instant the row stores while the reply is still fresh.
pub(crate) fn parse_tokens(body: &[u8], now: SystemTime) -> Result<Tokens, CoreError> {
    let value: serde_json::Value = serde_json::from_slice(body).map_err(|e| tracker_error(Tracker::Mal, format!("unreadable token reply: {e}")))?;
    let access_token = value["access_token"]
        .as_str()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| tracker_error(Tracker::Mal, "the token reply carried no access token"))?
        .to_string();
    Ok(Tokens {
        access_token,
        refresh_token: value["refresh_token"].as_str().filter(|t| !t.is_empty()).map(str::to_string),
        expires_at: value["expires_in"].as_u64().map(|seconds| now + Duration::from_secs(seconds)),
    })
}

/// Both tokens into the keyring, and where the access token landed, which
/// is what the row's `secret_store` records.
pub(crate) async fn save_tokens(core: &Arc<Core>, t: Tracker, tokens: &Tokens) -> Result<StoreKind, CoreError> {
    let access = tokens.access_token.clone();
    let store = with_secrets(core, move |s| s.set(&access_key(t), &access)).await?;
    match tokens.refresh_token.clone() {
        Some(refresh) => {
            with_secrets(core, move |s| s.set(&refresh_key(t), &refresh)).await?;
        }
        // A flow with no refresh token must not leave the last one behind:
        // it belongs to a session that is over.
        None => with_secrets(core, move |s| s.delete(&refresh_key(t))).await?,
    }
    Ok(store)
}

/// The row a finished flow leaves: who signed in, with which id, until
/// when, and where their tokens went.
pub(crate) async fn save_connection(
    core: &Arc<Core>,
    t: Tracker,
    user_id: Option<u64>,
    username: &str,
    client_id: &str,
    expires_at: Option<SystemTime>,
    store: StoreKind,
) -> Result<(), CoreError> {
    let username = username.to_string();
    let client_id = client_id.to_string();
    let user_id = user_id.map(|id| id as i64);
    let expires_at = time::opt_to_secs(expires_at);
    let connected_at = time::now_secs();
    core.store
        .tx_async(move |tx| {
            tx.execute(
                "INSERT INTO tracker_accounts (tracker, user_id, username, client_id, expires_at, connected_at, secret_store)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(tracker) DO UPDATE SET user_id = excluded.user_id, username = excluded.username,
                        client_id = excluded.client_id, expires_at = excluded.expires_at,
                        connected_at = excluded.connected_at, secret_store = excluded.secret_store",
                params![t.as_str(), user_id, username, client_id, expires_at, connected_at, store.as_str()],
            )?;
            Ok(())
        })
        .await
}

// Shared helpers -------------------------------------------------------------

/// Every keyring call a job makes goes through here: the Secret Service is
/// synchronous zbus underneath, so a tokio worker must never block on it.
/// The facade is taken off the field rather than through `Core::secrets`,
/// which would run the D-Bus probe on the calling task; whoever started
/// the job has already been through the accessor.
pub(crate) async fn with_secrets<T, F>(core: &Arc<Core>, f: F) -> Result<T, CoreError>
where
    T: Send + 'static,
    F: FnOnce(&Secrets) -> Result<T, CoreError> + Send + 'static,
{
    let secrets = core.secrets.clone();
    joined(core.handle.spawn_blocking(move || f(&secrets)).await)?
}

/// A blocking task that was cancelled or panicked, reported rather than
/// unwrapped: nothing exported panics on anything.
fn joined<T>(joined: Result<T, tokio::task::JoinError>) -> Result<T, CoreError> {
    joined.map_err(|e| CoreError::internal(format!("keyring task: {e}")))
}

/// A tracker's own failure, carrying the provider it belongs to.
pub(crate) fn tracker_error(t: Tracker, message: impl Into<String>) -> CoreError {
    CoreError::Provider {
        provider: match t {
            Tracker::Anilist => Provider::Anilist,
            Tracker::Mal => Provider::Mal,
        },
        status: None,
        message: message.into(),
        retry_after: None,
    }
}

/// The whole tracker state, after anything that changed a part of it.
pub(crate) fn emit_trackers_changed(core: &Core) -> Result<(), CoreError> {
    let state = state(core)?;
    core.bus.debug(Stage::Trackers, "trackers changed", EventBody::TrackersChanged { state });
    Ok(())
}

/// The cards whose numbers came off a tracker, in one batch: `Some(t)` for
/// the series carrying that tracker's id, `None` for every matched series.
fn emit_series_changed(core: &Core, t: Option<Tracker>, message: String) -> Result<(), CoreError> {
    let images_dir = core.paths.images_dir();
    let cards = core.store.read(|c| {
        let ids = matched_ids(c, t)?;
        cards::cards_for(c, &images_dir, &ids)
    })?;
    if cards.is_empty() {
        return Ok(());
    }
    core.bus.debug(Stage::Trackers, message, EventBody::SeriesChanged { series: cards });
    Ok(())
}

fn matched_ids(conn: &Connection, t: Option<Tracker>) -> Result<Vec<u64>, CoreError> {
    let sql = match t {
        Some(Tracker::Anilist) => "SELECT id FROM series WHERE anilist_id IS NOT NULL ORDER BY id",
        Some(Tracker::Mal) => "SELECT id FROM series WHERE mal_id IS NOT NULL ORDER BY id",
        None => "SELECT id FROM series WHERE provider IS NOT NULL ORDER BY id",
    };
    let mut stmt = conn.prepare(sql)?;
    let ids = stmt.query_map([], |r| r.get::<_, i64>(0))?.collect::<Result<Vec<i64>, _>>()?;
    Ok(ids.into_iter().map(|id| id as u64).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    #[test]
    fn the_keys_are_the_whole_address_of_a_secret() {
        assert_eq!(access_key(Tracker::Anilist), "anilist.access_token");
        assert_eq!(refresh_key(Tracker::Mal), "mal.refresh_token");
        assert_eq!(secret_key(Tracker::Mal), "mal.client_secret");
    }

    #[test]
    fn a_token_reply_parses_into_an_instant() {
        let now = UNIX_EPOCH + Duration::from_secs(1_000_000);
        let body = br#"{"access_token":"mtok","refresh_token":"mref","expires_in":2415600}"#;
        let tokens = parse_tokens(body, now).unwrap();
        assert_eq!(tokens.access_token, "mtok");
        assert_eq!(tokens.refresh_token.as_deref(), Some("mref"));
        assert_eq!(tokens.expires_at, Some(now + Duration::from_secs(2_415_600)));
    }

    /// A reply with no token is the provider's failure, not a token of
    /// nothing: the row must never end up connected with an empty string.
    #[test]
    fn a_token_reply_without_a_token_is_an_error() {
        assert!(parse_tokens(br#"{"error":"invalid_request"}"#, time::now()).is_err());
        assert!(parse_tokens(br#"{"access_token":""}"#, time::now()).is_err());
        assert!(parse_tokens(b"not json", time::now()).is_err());
    }

    /// No refresh token in the reply means none is stored, so nothing here
    /// invents one from an empty string.
    #[test]
    fn a_reply_without_a_refresh_token_carries_none() {
        let tokens = parse_tokens(br#"{"access_token":"tok"}"#, time::now()).unwrap();
        assert_eq!(tokens.refresh_token, None);
        assert_eq!(tokens.expires_at, None);
    }

    #[test]
    fn a_token_inside_the_window_or_past_it_is_due_for_a_refresh() {
        let now = UNIX_EPOCH + Duration::from_secs(1_000_000);
        assert!(!due_for_refresh(None, now));
        assert!(!due_for_refresh(Some(now + Duration::from_secs(60 * 60 * 24)), now));
        assert!(due_for_refresh(Some(now + Duration::from_secs(59 * 60)), now));
        assert!(due_for_refresh(Some(now - Duration::from_secs(1)), now));
    }

    #[test]
    fn an_empty_stored_client_id_is_no_client_id() {
        let row = Row { client_id: Some(String::new()), ..Row::default() };
        assert_eq!(row.stored_client_id(), None);
        let row = Row { client_id: Some("123".into()), ..Row::default() };
        assert_eq!(row.stored_client_id().as_deref(), Some("123"));
    }
}
