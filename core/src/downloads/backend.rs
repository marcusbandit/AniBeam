//! Where the desktop's search, refine, subscriptions and downloads run:
//! anirss, over HTTP, or the core's own port of it, as the downloads
//! settings' `engine` says. The port searches through `releases::native`
//! and writes rules and torrents to qBittorrent itself (`native`). Both
//! answer in the shapes the handlers already read from anirss, so the
//! facts, the library link and the dating that follow are the same for
//! either. The endpoint list stays anirss's whichever runs (#119).

use std::sync::Arc;

use crate::contract::*;
use crate::core::Core;
use crate::releases::native::Native;

use super::anirss::{Analysis, AnirssClient, SearchRaw};
use super::{ensure_client, native as on_qbt, settings};

/// Native carries the core so its torrent client is fetched only when a
/// call needs it: search, refine and preview never touch qBittorrent.
#[derive(Clone)]
pub enum DownloadsBackend {
    Anirss(AnirssClient),
    Native(Native, Arc<Core>),
}

/// The engine the settings pick. A store that cannot be read is an error,
/// not a quiet fall back to one or the other.
pub fn engine(core: &Core) -> Result<DownloadsEngine, CoreError> {
    Ok(core.store.read(settings::load_stored)?.engine)
}

impl DownloadsBackend {
    /// The backend the settings pick.
    pub async fn open(core: &Arc<Core>) -> Result<DownloadsBackend, CoreError> {
        DownloadsBackend::with(core, engine(core)?).await
    }

    /// The backend for `engine`. Native never starts the sidecar; anirss is
    /// `ensure_client`, the sidecar started when it is not up.
    pub async fn with(
        core: &Arc<Core>,
        engine: DownloadsEngine,
    ) -> Result<DownloadsBackend, CoreError> {
        match engine {
            DownloadsEngine::Anirss => ensure_client(core).await.map(DownloadsBackend::Anirss),
            DownloadsEngine::Native => {
                let native = Native::for_core(core, Vec::new());
                tracing::debug!(
                    "searching on the core's engine: {} endpoints",
                    native.endpoints.len()
                );
                Ok(DownloadsBackend::Native(native, core.clone()))
            }
        }
    }

    /// The endpoint a search with none named lands on, when this side
    /// knows it without asking: anirss's comes from its own list.
    pub fn first_endpoint(&self) -> Option<String> {
        match self {
            DownloadsBackend::Anirss(_) => None,
            DownloadsBackend::Native(native, _) => native.endpoints.first().map(|e| e.name.clone()),
        }
    }

    /// One search with fallback. Native fills in the seeders a feed left
    /// out, as anirss's own search does before it answers.
    pub async fn search(
        &self,
        query: &str,
        endpoint: Option<&str>,
        fallback: bool,
    ) -> Result<SearchRaw, CoreError> {
        match self {
            DownloadsBackend::Anirss(client) => client.search(query, endpoint, fallback).await,
            DownloadsBackend::Native(native, _) => {
                let found = native.search(query, endpoint, fallback).await?;
                for t in &found.tried {
                    tracing::debug!("{}: {} ({})", t.endpoint, t.status, t.count);
                }
                let fell_back = found.fell_back();
                let items = if found.items.iter().any(|i| i.seeders.is_none()) {
                    native.enrich(found.items).await
                } else {
                    found.items
                };
                Ok(SearchRaw {
                    query: found.query,
                    endpoint: found.endpoint,
                    items,
                    tried: found.tried,
                    fell_back,
                })
            }
        }
    }

    /// The groups and the best fit after auto-resolution.
    pub async fn analyze(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
    ) -> Result<Analysis, CoreError> {
        match self {
            DownloadsBackend::Anirss(client) => client.analyze(query, endpoint, items).await,
            DownloadsBackend::Native(native, _) => {
                let a = native.analyze(query, endpoint, items, true)?;
                Ok(Analysis {
                    query: a.query,
                    items: a.items,
                    groups: a.groups,
                    best_fit: a.best_fit,
                    client_side: a.client_side,
                    auto_resolution: a.auto_resolution,
                })
            }
        }
    }

    /// One refine step. The facts and the show are left for the caller.
    pub async fn apply(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
        op: &RefineOp,
    ) -> Result<Refinement, CoreError> {
        match self {
            DownloadsBackend::Anirss(client) => client.apply(query, endpoint, items, op).await,
            DownloadsBackend::Native(native, _) => native.apply(query, endpoint, items, op).await,
        }
    }

    /// What subscribing to (query, items) would register.
    pub async fn preview(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
    ) -> Result<SubscriptionSpec, CoreError> {
        match self {
            DownloadsBackend::Anirss(client) => client.preview(query, endpoint, items).await,
            DownloadsBackend::Native(native, _) => native.preview(query, endpoint, items).await,
        }
    }

    /// Subscribes to (query, items): a feed and a rule saving into the
    /// show's folder.
    pub async fn subscribe(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
        name: Option<&str>,
    ) -> Result<Subscribed, CoreError> {
        match self {
            DownloadsBackend::Anirss(client) => {
                client.subscribe(query, endpoint, items, name).await
            }
            DownloadsBackend::Native(native, core) => {
                on_qbt::subscribe(core, native, query, endpoint, items, name).await
            }
        }
    }

    /// Every subscription rule qBittorrent holds.
    pub async fn subscriptions(&self) -> Result<Vec<SubscriptionRule>, CoreError> {
        match self {
            DownloadsBackend::Anirss(client) => client.subscriptions().await,
            DownloadsBackend::Native(_, core) => on_qbt::subscriptions(core).await,
        }
    }

    /// Removes subscriptions, or with `dry_run` only says what would go.
    pub async fn remove(
        &self,
        names: &[String],
        torrents: bool,
        files: bool,
        dry_run: bool,
    ) -> Result<Vec<RemovePlan>, CoreError> {
        match self {
            DownloadsBackend::Anirss(client) => {
                client.remove(names, torrents, files, dry_run).await
            }
            DownloadsBackend::Native(_, core) => {
                on_qbt::remove(core, names, torrents, files, dry_run).await
            }
        }
    }

    /// Adds the releases now, into the show's folder.
    pub async fn download(
        &self,
        items: &[Release],
        name: Option<&str>,
    ) -> Result<AddedTorrents, CoreError> {
        match self {
            DownloadsBackend::Anirss(client) => client.download(items, name).await,
            DownloadsBackend::Native(_, core) => on_qbt::download(core, items, name).await,
        }
    }

    /// Adds one film into the films' folder.
    pub async fn movie(&self, link: &str) -> Result<AddedTorrents, CoreError> {
        match self {
            DownloadsBackend::Anirss(client) => client.movie(link).await,
            DownloadsBackend::Native(_, core) => on_qbt::movie(core, link).await,
        }
    }

    /// An endpoint whose subscription is narrowed on this side, where a
    /// refined query is no query the endpoint can answer: a generic RSS
    /// endpoint that does not already narrow its subscription in the rule.
    /// anirss's endpoints are the list the last connect read from it; the
    /// core's are its own.
    pub fn narrows_locally(&self, core: &Core, endpoint: &str) -> bool {
        match self {
            DownloadsBackend::Anirss(_) => core
                .downloads
                .state()
                .endpoints
                .iter()
                .find(|e| e.name == endpoint)
                .is_some_and(|e| e.kind != "nyaa" && !e.url.contains("erai-raws")),
            DownloadsBackend::Native(native, _) => native
                .endpoints
                .iter()
                .find(|e| e.name == endpoint)
                .is_some_and(|e| e.kind != "nyaa" && !crate::releases::erai::is_erai(e)),
        }
    }
}
