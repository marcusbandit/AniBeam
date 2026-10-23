//! anirss's API (`api.py`), in the core: search across endpoints with fallback, the
//! analysis the refine screen shows, one refine step, and a subscription's feed and rule.
//! The answers are the same contract types the desktop reads from anirss over HTTP, so a
//! shell shows them the way it shows anirss's.

use std::sync::Arc;

use crate::contract::*;
use crate::net::Http;

use super::{BestfitConfig, Endpoint, bestfit, endpoints, refinement, stats};

/// The groups the desktop's best fit trusts out of the box: the owner's anirss config.
pub const TRUSTED_GROUPS: [&str; 9] = [
    "Erai-raws",
    "SubsPlease",
    "ASW",
    "EMBER",
    "Judas",
    "Osiris",
    "Moozzi2",
    "VCB-Studio",
    "Kawaiika-Raws",
];

/// What a search found before its analysis (anirss `SearchResult`).
#[derive(Clone, Debug)]
pub struct Found {
    pub query: String,
    pub endpoint: String,
    pub items: Vec<Release>,
    pub tried: Vec<EndpointTry>,
}

impl Found {
    /// anirss: more than one endpoint asked, and the last one had something.
    pub fn fell_back(&self) -> bool {
        self.tried.len() > 1 && !self.items.is_empty()
    }
}

/// The refine screen's view of a result set (anirss `analyze`).
#[derive(Clone, Debug)]
pub struct Analysis {
    pub query: String,
    pub items: Vec<Release>,
    pub groups: Vec<ReleaseGroup>,
    pub best_fit: BestFit,
    pub client_side: bool,
    pub auto_resolution: Option<String>,
}

pub struct Native {
    pub http: Arc<dyn Http>,
    pub endpoints: Vec<Endpoint>,
    pub bestfit: BestfitConfig,
}

impl Native {
    pub fn new(http: Arc<dyn Http>, endpoints: Vec<Endpoint>) -> Native {
        let bestfit = BestfitConfig {
            preferred_groups: TRUSTED_GROUPS.map(String::from).to_vec(),
            ..BestfitConfig::default()
        };
        Native {
            http,
            endpoints,
            bestfit,
        }
    }

    /// The endpoint by name, else the first (anirss `endpoint`).
    pub fn endpoint(&self, name: Option<&str>) -> Result<&Endpoint, CoreError> {
        if self.endpoints.is_empty() {
            return Err(CoreError::invalid(
                "endpoint",
                "no sources to search; add one first",
            ));
        }
        match name {
            None => Ok(&self.endpoints[0]),
            Some(name) => self
                .endpoints
                .iter()
                .find(|e| e.name == name)
                .ok_or_else(|| CoreError::invalid("endpoint", format!("no source named {name}"))),
        }
    }

    /// The endpoints as the downloads page lists them, keys masked.
    pub fn list(&self) -> Vec<SearchEndpoint> {
        self.endpoints
            .iter()
            .map(|e| SearchEndpoint {
                name: e.name.clone(),
                kind: e.kind.clone(),
                url: mask(&e.url),
                client_side_refine: refinement::is_client_side(e),
                on_demand: None,
                running: None,
            })
            .collect()
    }

    /// Searches one endpoint; when it is empty or unreachable and `fallback` is on, the
    /// others in order, first hit wins and its items carry `source`. Fails only when
    /// nothing was reachable at all (anirss `search`).
    pub async fn search(
        &self,
        query: &str,
        endpoint: Option<&str>,
        fallback: bool,
    ) -> Result<Found, CoreError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(CoreError::invalid("query", "type something to search for"));
        }
        let active = self.endpoint(endpoint)?;
        let mut order = vec![active];
        if fallback {
            order.extend(self.endpoints.iter().filter(|e| e.name != active.name));
        }
        let mut tried = Vec::new();
        let mut errors = Vec::new();
        for (i, ep) in order.iter().enumerate() {
            match endpoints::fetch_items(self.http.as_ref(), ep, query).await {
                Err(e) => {
                    tried.push(EndpointTry {
                        endpoint: ep.name.clone(),
                        status: "unreachable".into(),
                        count: 0,
                        error: Some(e.to_string()),
                    });
                    errors.push(e.to_string());
                }
                Ok(mut items) => {
                    tried.push(EndpointTry {
                        endpoint: ep.name.clone(),
                        status: if items.is_empty() { "empty" } else { "ok" }.into(),
                        count: items.len() as u64,
                        error: None,
                    });
                    if !items.is_empty() {
                        if i > 0 {
                            for it in &mut items {
                                it.source = ep.name.clone();
                            }
                        }
                        return Ok(Found {
                            query: query.to_string(),
                            endpoint: ep.name.clone(),
                            items,
                            tried,
                        });
                    }
                }
            }
        }
        if errors.len() == order.len() {
            return Err(CoreError::Service {
                service: active.name.clone(),
                code: "unreachable".into(),
                message: errors.join("; "),
            });
        }
        Ok(Found {
            query: query.to_string(),
            endpoint: active.name.clone(),
            items: Vec::new(),
            tried,
        })
    }

    /// Fills seeders and the like where the feed left them out (anirss `enrich`).
    pub async fn enrich(&self, items: Vec<Release>) -> Vec<Release> {
        stats::enrich(self.http.as_ref(), items, stats::DEFAULT_DEADLINE).await
    }

    /// The auto-resolution step, the groups to narrow by and what best fit would do
    /// (anirss `analyze`).
    pub fn analyze(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
        auto_resolution: bool,
    ) -> Result<Analysis, CoreError> {
        let ep = self.endpoint(Some(endpoint))?;
        let auto = if auto_resolution {
            refinement::auto_resolution(query.trim(), items)
        } else {
            refinement::AutoResolution {
                query: query.trim().to_string(),
                items: items.to_vec(),
                token: String::new(),
                skipped_missing: 0,
            }
        };
        let groups = refinement::compute_groups(&auto.items);
        let best_fit = refinement::best_fit_info(&auto.query, &auto.items, &self.bestfit);
        Ok(Analysis {
            client_side: refinement::is_client_side(ep),
            auto_resolution: (!auto.token.is_empty()).then(|| auto.token.clone()),
            query: auto.query,
            items: auto.items,
            groups,
            best_fit,
        })
    }

    /// One refine step and the analysis of where it leaves the results (anirss `refine`,
    /// as its server answers it).
    pub async fn apply(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
        op: &RefineOp,
    ) -> Result<Refinement, CoreError> {
        let ep = self.endpoint(Some(endpoint))?.clone();
        let http = self.http.clone();
        // Typed as the callback so the closure is generic over the borrows it is given.
        let fetch: Box<refinement::FetchFn<'_>> = Box::new(move |ep: &Endpoint, q: &str| {
            let http = http.clone();
            let (ep, q) = (ep.clone(), q.to_string());
            Box::pin(async move { endpoints::fetch_items(http.as_ref(), &ep, &q).await })
        });
        let outcome =
            refinement::apply(&ep, query, items, op, &self.bestfit, fetch.as_ref(), None).await?;
        let groups = refinement::compute_groups(&outcome.items);
        let best_fit = refinement::best_fit_info(&outcome.query, &outcome.items, &self.bestfit);
        let count = outcome.items.len() as i64;
        Ok(Refinement {
            applied: outcome.applied,
            reason: outcome.reason.clone(),
            mode: outcome.mode.clone(),
            query: outcome.query.clone(),
            endpoint: ep.name.clone(),
            groups,
            best_fit,
            previous_count: outcome.previous_count as u64,
            delta: count - outcome.previous_count as i64,
            items: outcome.items,
            facts: Vec::new(),
            show: None,
        })
    }

    /// What subscribing to (query, items) would register (anirss `subscription_spec`).
    pub async fn preview(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
    ) -> Result<SubscriptionSpec, CoreError> {
        let ep = self.endpoint(Some(endpoint))?;
        Ok(endpoints::build_subscription(self.http.as_ref(), ep, query, items).await)
    }

    /// The release best fit would pick.
    pub fn best_item(&self, items: &[Release]) -> Option<Release> {
        bestfit::best_item(items, &self.bestfit).cloned()
    }
}

/// A URL with any key in it cut short, for a list a person reads.
fn mask(url: &str) -> String {
    let mut out = url.to_string();
    for key in ["token=", "apikey=", "passkey=", "api_key="] {
        let mut from = 0;
        while let Some(at) = out[from..].find(key) {
            let start = from + at + key.len();
            let end = out[start..].find('&').map_or(out.len(), |e| start + e);
            out.replace_range(start..end, "...");
            from = start + 3;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_in_a_feed_address_is_masked() {
        assert_eq!(
            mask("https://x/feed/?s={query}&token=abc&res=1080p"),
            "https://x/feed/?s={query}&token=...&res=1080p"
        );
    }
}
