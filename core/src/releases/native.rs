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

    /// The endpoint a step after a search names: the analysis, a refine, a preview. The
    /// list is read again on every call, so one removed, switched off or left without its
    /// key since the search is not in it, even when it was the only one.
    fn step_endpoint(&self, name: &str) -> Result<&Endpoint, CoreError> {
        self.endpoints
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| CoreError::invalid("endpoint", gone(name)))
    }

    /// Whether `endpoint`'s feed is RSS, so reading it again after a search can date the
    /// releases. False for a JSON answer and for an endpoint the core no longer has.
    pub fn feed_is_rss(&self, endpoint: &str) -> bool {
        self.endpoint(Some(endpoint))
            .is_ok_and(|e| endpoints::answers_rss(&e.kind))
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
    /// was found and the picked endpoint failed, so a bad key is not read as no results
    /// just because a fallback answered empty (#255).
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
        let mut errors: Vec<CoreError> = Vec::new();
        let mut active_failed = false;
        for (i, ep) in order.iter().enumerate() {
            match endpoints::fetch_items(self.http.as_ref(), ep, query).await {
                Err(e) => {
                    tried.push(EndpointTry {
                        endpoint: ep.name.clone(),
                        status: "unreachable".into(),
                        count: 0,
                        error: Some(e.to_string()),
                    });
                    active_failed |= i == 0;
                    errors.push(e);
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
                            for it in items.iter_mut().filter(|it| it.source.is_empty()) {
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
        if active_failed {
            // One endpoint failed: its error as it came. Several: the active endpoint's
            // code, since that is the one the user picked and can act on (a bad key is
            // `refused`, not `unreachable`), with every endpoint's message joined.
            if errors.len() == 1 {
                return Err(errors.remove(0));
            }
            let rest = errors.drain(1..).map(|e| e.to_string()).collect::<Vec<_>>();
            return Err(match errors.remove(0) {
                // The active endpoint's own message leads, so its name is not said twice.
                CoreError::Service {
                    service,
                    code,
                    message,
                } => CoreError::Service {
                    service,
                    code,
                    message: [message]
                        .into_iter()
                        .chain(rest)
                        .collect::<Vec<_>>()
                        .join("; "),
                },
                other => other,
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
        let ep = self.step_endpoint(endpoint)?;
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
        let ep = self.step_endpoint(endpoint)?.clone();
        let http = self.http.clone();
        let all_uploads = endpoints::found_unfiltered(items);
        // Typed as the callback so the closure is generic over the borrows it is given.
        let fetch: Box<refinement::FetchFn<'_>> = Box::new(move |ep: &Endpoint, q: &str| {
            let http = http.clone();
            let (ep, q) = (ep.clone(), q.to_string());
            Box::pin(async move { endpoints::refetch(http.as_ref(), &ep, &q, all_uploads).await })
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
        let ep = self.step_endpoint(endpoint)?;
        Ok(endpoints::build_subscription(self.http.as_ref(), ep, query, items).await)
    }

    /// The release best fit would pick.
    pub fn best_item(&self, items: &[Release]) -> Option<Release> {
        bestfit::best_item(items, &self.bestfit).cloned()
    }
}

/// Why a step after a search cannot go on: its endpoint left the list since.
fn gone(name: &str) -> String {
    format!("{name} is not available any more: removed, switched off or missing its key")
}

/// Query parameters that hold a secret, with the label a person reads for each. A source
/// added by hand has these lifted out of its address, a listed address has them masked, and
/// the tailnet snapshot leaves them out. Names match case-insensitively.
pub const SECRET_PARAMS: [(&str, &str); 15] = [
    ("apikey", "API key"),
    ("api_key", "API key"),
    ("jackett_apikey", "API key"),
    ("token", "Token"),
    ("passkey", "Passkey"),
    ("torrent_pass", "Passkey"),
    ("rsskey", "RSS key"),
    ("authkey", "Auth key"),
    ("auth", "Auth key"),
    ("key", "Key"),
    ("secret", "Secret"),
    ("sig", "Signature"),
    ("pass", "Password"),
    ("password", "Password"),
    ("uid", "User id"),
];

/// The label of a query parameter that holds a secret; None for any other.
pub fn secret_param(name: &str) -> Option<&'static str> {
    SECRET_PARAMS
        .iter()
        .find(|(p, _)| p.eq_ignore_ascii_case(name))
        .map(|(_, label)| *label)
}

/// A URL with any key in it cut short, for a list a person reads: the value of each secret
/// query parameter, and the password of a `user:password@` login.
pub fn mask(url: &str) -> String {
    let (base, query) = match url.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (url, None),
    };
    let hidden = |part: &str| part.is_empty() || part.starts_with('{');
    let mut out = match crate::repos::index::split_login(base) {
        Some((scheme, login, host)) => {
            let login = match login.split_once(':') {
                Some((user, pw)) if !hidden(pw) => format!("{user}:..."),
                Some((user, pw)) if !hidden(user) => format!("...:{pw}"),
                None if !hidden(login) => "...".to_string(),
                _ => login.to_string(),
            };
            format!("{scheme}{login}@{host}")
        }
        None => base.to_string(),
    };
    if let Some(query) = query {
        let pairs: Vec<String> = query
            .split('&')
            .map(|pair| match pair.split_once('=') {
                Some((k, v)) if secret_param(k).is_some() && !v.starts_with('{') => {
                    format!("{k}=...")
                }
                _ => pair.to_string(),
            })
            .collect();
        out = format!("{out}?{}", pairs.join("&"));
    }
    out
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

    #[test]
    fn only_an_endpoint_whose_feed_is_rss_is_read_again_for_dates() {
        let native = Native::new(
            FakeHttp::new(),
            vec![
                Endpoint::new("feed", "rss", "https://a/?q={query}"),
                Endpoint::new("indexer", "torznab", "https://b/?q={query}"),
                Endpoint::new("search", "nyaa", "https://e/"),
                Endpoint::new("api", "json", "https://c/?q={query}"),
                Endpoint::new("anime", "tosho", "https://d/?q={query}"),
            ],
        );
        assert!(native.feed_is_rss("feed"));
        assert!(native.feed_is_rss("indexer"));
        assert!(native.feed_is_rss("search"));
        assert!(!native.feed_is_rss("api"));
        assert!(!native.feed_is_rss("anime"));
        // One the core no longer has: nothing to read.
        assert!(!native.feed_is_rss("gone"));
    }

    #[tokio::test]
    async fn a_step_on_an_endpoint_gone_since_the_search_says_so() {
        let native = Native::new(
            FakeHttp::new(),
            vec![Endpoint::new("feed", "rss", "https://a/?q={query}")],
        );
        let refine = native
            .apply("show", "gone", &[], &RefineOp::BestFit)
            .await
            .unwrap_err();
        let preview = native.preview("show", "gone", &[]).await.unwrap_err();
        let analyze = native.analyze("show", "gone", &[], false).unwrap_err();
        // The only endpoint removed: still named, not "no endpoints yet".
        let alone = Native::new(FakeHttp::new(), Vec::new())
            .preview("show", "gone", &[])
            .await
            .unwrap_err();
        for err in [refine, preview, analyze, alone] {
            let text = err.to_string();
            assert!(text.contains("gone is not available any more"), "{text}");
        }
        // A search naming one it never had keeps its own answer.
        let search = native
            .search("show", Some("gone"), false)
            .await
            .unwrap_err();
        assert!(
            search.to_string().contains("no source named gone"),
            "{search}"
        );
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

    #[tokio::test]
    async fn a_failed_pick_with_an_empty_fallback_keeps_the_picks_code() {
        let http = FakeHttp::new();
        http.push_for("https://a/", 401, "no");
        http.push_for("https://b/", 200, "<rss><channel></channel></rss>");
        let native = Native::new(
            http,
            vec![
                Endpoint::new("first", "rss", "https://a/?q={query}"),
                Endpoint::new("second", "rss", "https://b/?q={query}"),
            ],
        );
        let err = native.search("show", None, true).await.unwrap_err();
        assert_eq!(code_of(err), ("first".into(), "http_401".into()));
    }

    #[tokio::test]
    async fn a_reachable_pick_with_nothing_found_is_no_results() {
        let http = FakeHttp::new();
        http.push_for("https://a/", 200, "<rss><channel></channel></rss>");
        http.push_for("https://b/", 503, "busy");
        let native = Native::new(
            http,
            vec![
                Endpoint::new("first", "rss", "https://a/?q={query}"),
                Endpoint::new("second", "rss", "https://b/?q={query}"),
            ],
        );
        let found = native.search("show", None, true).await.unwrap();
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

    #[test]
    fn every_key_name_and_a_login_are_masked_whatever_their_case() {
        assert_eq!(
            mask("https://me:pw@x/rss?q={query}&RssKey=a&authkey=b&key=c&monkey=d"),
            "https://me:...@x/rss?q={query}&RssKey=...&authkey=...&key=...&monkey=d"
        );
        assert_eq!(mask("https://tok@x/rss"), "https://...@x/rss");
        assert_eq!(mask("https://tok:@x/rss"), "https://...:@x/rss");
        assert_eq!(
            mask("https://x/rss?uid=1&PASS=2&q={query}"),
            "https://x/rss?uid=...&PASS=...&q={query}"
        );
        assert_eq!(
            mask("https://x/rss?q={query}&passkey={passkey}"),
            "https://x/rss?q={query}&passkey={passkey}"
        );
    }
}
