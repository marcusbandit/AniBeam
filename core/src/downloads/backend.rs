//! Where the desktop's search, refine and subscription preview run: anirss,
//! over HTTP, or the core's own port of it (`releases::native`), as the
//! downloads settings' `engine` says. Both answer in the shapes the
//! handlers already read from anirss, so the facts, the library link and
//! the dating that follow are the same for either. Subscribing, the
//! downloads and the endpoint list stay on anirss whichever runs (#119).

use std::sync::Arc;

use crate::contract::*;
use crate::core::Core;
use crate::releases::native::Native;

use super::anirss::{Analysis, AnirssClient, SearchRaw};
use super::{ensure_client, settings};

#[derive(Clone)]
pub enum DownloadsBackend {
    Anirss(AnirssClient),
    Native(Native),
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
                Ok(DownloadsBackend::Native(native))
            }
        }
    }

    /// The endpoint a search with none named lands on, when this side
    /// knows it without asking: anirss's comes from its own list.
    pub fn first_endpoint(&self) -> Option<String> {
        match self {
            DownloadsBackend::Anirss(_) => None,
            DownloadsBackend::Native(native) => native.endpoints.first().map(|e| e.name.clone()),
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
            DownloadsBackend::Native(native) => {
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
            DownloadsBackend::Native(native) => {
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
            DownloadsBackend::Native(native) => native.apply(query, endpoint, items, op).await,
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
            DownloadsBackend::Native(native) => native.preview(query, endpoint, items).await,
        }
    }
}
