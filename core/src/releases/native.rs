//! anirss's API (`api.py`), in the core: search across endpoints with fallback, the
//! analysis the refine screen shows, one refine step, and a subscription's feed and rule.
//! The answers are the same contract types the desktop reads from anirss over HTTP, so a
//! shell shows them the way it shows anirss's.

use std::sync::Arc;

use crate::contract::*;
use crate::core::Core;
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

#[derive(Clone)]
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

    /// The core's endpoints, each host held to the pace its repo definition asks for:
    /// `first` in its order, then every enabled installed endpoint (`repos::runnable`, the
    /// repos' and the ones added by hand) that `first` does not already name. The phone
    /// passes the sources it searches; the desktop passes none.
    ///
    /// The desktop's list is the core's own, not anirss's `config.toml`: whether it should
    /// match anirss's endpoints before the native engine is the default is the owner's call
    /// (#119, decision B).
    pub fn for_core(core: &Core, first: Vec<Endpoint>) -> Native {
        let installed = crate::repos::runnable(core);
        let http = crate::repos::paced_http(core, &installed);
        let mut endpoints = first;
        for r in installed {
            if !endpoints.iter().any(|e| e.name == r.endpoint.name) {
                endpoints.push(r.endpoint);
            }
        }
        Native::new(http, endpoints)
    }

    /// The endpoint by name, else the first (anirss `endpoint`).
    pub fn endpoint(&self, name: Option<&str>) -> Result<&Endpoint, CoreError> {
        if self.endpoints.is_empty() {
            return Err(CoreError::invalid(
                "endpoint",
                "no endpoints yet: add a repo, or one by hand, under Endpoints in More",
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
    /// others in order, first hit wins and its items carry `source`. Fails when nothing
    /// found anything and the picked endpoint failed, with that endpoint's error (anirss
    /// `search`).
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
        let mut picked_error: Option<CoreError> = None;
        let mut other_errors: Vec<String> = Vec::new();
        for (i, ep) in order.iter().enumerate() {
            match endpoints::fetch_items(self.http.as_ref(), ep, query).await {
                Err(e) => {
                    tried.push(EndpointTry {
                        endpoint: ep.name.clone(),
                        status: match &e {
                            CoreError::Service { code, .. } => code.clone(),
                            _ => "unreachable".into(),
                        },
                        count: 0,
                        error: Some(e.to_string()),
                    });
                    if i == 0 {
                        picked_error = Some(e);
                    } else {
                        other_errors.push(e.to_string());
                    }
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
        // The picked endpoint is the one the user can act on (a bad key is `refused`), so
        // its failure is not hidden behind another endpoint answering empty.
        if let Some(e) = picked_error {
            return Err(with_notes(e, other_errors));
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
pub fn mask(url: &str) -> String {
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

/// `lead` with `notes` joined after its message, keeping its service and code. Its own
/// message leads, so its endpoint's name is not said twice.
fn with_notes(lead: CoreError, notes: Vec<String>) -> CoreError {
    match lead {
        CoreError::Service {
            service,
            code,
            message,
        } => CoreError::Service {
            service,
            code,
            message: [message]
                .into_iter()
                .chain(notes)
                .collect::<Vec<_>>()
                .join("; "),
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;

    fn code_of(err: CoreError) -> (String, String) {
        match err {
            CoreError::Service { service, code, .. } => (service, code),
            other => panic!("{other:?}"),
        }
    }

    #[tokio::test]
    async fn one_endpoint_that_fails_keeps_its_own_code() {
        let http = FakeHttp::new();
        http.push(401, "no");
        let native = Native::new(
            http,
            vec![Endpoint::new("feed", "rss", "https://x/?q={query}")],
        );
        let err = native.search("show", None, false).await.unwrap_err();
        assert_eq!(code_of(err), ("feed".into(), "http_401".into()));
    }

    #[tokio::test]
    async fn every_endpoint_failing_keeps_the_active_endpoints_code() {
        let http = FakeHttp::new();
        http.push_for("https://a/", 200, "not a feed <");
        http.push_for("https://b/", 503, "busy");
        let native = Native::new(
            http,
            vec![
                Endpoint::new("first", "rss", "https://a/?q={query}"),
                Endpoint::new("second", "rss", "https://b/?q={query}"),
            ],
        );
        let err = native.search("show", None, true).await.unwrap_err();
        let text = err.to_string();
        assert_eq!(text.matches("first").count(), 1, "{text}");
        assert!(text.contains("second"), "{text}");
        assert_eq!(code_of(err), ("first".into(), "unreadable".into()));
    }

    const EMPTY_FEED: &str = r#"<rss version="2.0"><channel></channel></rss>"#;
    const ONE_ITEM_FEED: &str = r#"<rss version="2.0"><channel><item><title>Show 01</title><link>https://b/1.torrent</link></item></channel></rss>"#;

    fn two_endpoints(http: Arc<FakeHttp>) -> Native {
        Native::new(
            http,
            vec![
                Endpoint::new("first", "rss", "https://a/?q={query}"),
                Endpoint::new("second", "rss", "https://b/?q={query}"),
            ],
        )
    }

    #[tokio::test]
    async fn a_picked_endpoint_that_fails_is_not_hidden_by_an_empty_fallback() {
        let http = FakeHttp::new();
        http.push_for("https://a/", 401, "no");
        http.push_for("https://b/", 200, EMPTY_FEED);
        let err = two_endpoints(http)
            .search("show", None, true)
            .await
            .unwrap_err();
        assert_eq!(code_of(err), ("first".into(), "http_401".into()));
    }

    #[tokio::test]
    async fn a_picked_endpoint_that_fails_falls_back_to_one_with_items() {
        let http = FakeHttp::new();
        http.push_for("https://a/", 401, "no");
        http.push_for("https://b/", 200, ONE_ITEM_FEED);
        let found = two_endpoints(http)
            .search("show", None, true)
            .await
            .unwrap();
        assert_eq!(found.endpoint, "second");
        assert_eq!(found.items.len(), 1);
        assert_eq!(found.items[0].source, "second");
        assert_eq!(found.tried[0].status, "http_401");
    }

    #[tokio::test]
    async fn a_picked_endpoint_that_is_empty_stays_empty_when_the_fallback_fails() {
        let http = FakeHttp::new();
        http.push_for("https://a/", 200, EMPTY_FEED);
        http.push_for("https://b/", 503, "busy");
        let found = two_endpoints(http)
            .search("show", None, true)
            .await
            .unwrap();
        assert_eq!(found.endpoint, "first");
        assert!(found.items.is_empty());
        assert_eq!(found.tried.len(), 2);
    }

    #[test]
    fn a_key_in_a_feed_address_is_masked() {
        assert_eq!(
            mask("https://x/feed/?s={query}&token=abc&res=1080p"),
            "https://x/feed/?s={query}&token=...&res=1080p"
        );
    }
}
