//! anirss over its JSON HTTP API (`anirss --serve`), and the sidecar that
//! runs it. The API is `docs/agents/api-reference.md` in anirss's repo:
//! every route under `/api/v1`, a bearer token on all but `/health`, one
//! error envelope `{"error": {"type", "message"}}` whose `type` is the word
//! a client branches on. anirss keeps no search state, so every refine and
//! subscribe call sends the whole `{endpoint, query, items}` back.
//!
//! As a sidecar the core picks the port and the token itself and hands
//! them over in the environment, as the API docs ask of a client that
//! launches the server: the startup banner is coloured prose, not a thing
//! to parse. The qBittorrent password rides along as `ANIRSS_QBT_PASSWORD`
//! so the one saved in AniBeam is the one anirss logs in with.

use std::collections::VecDeque;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use crate::contract::*;
use crate::net::{Body, Http, HttpRequest, HttpResponse, Method};

/// How `CoreError::Service` names this service.
pub const SERVICE: &str = "anirss";

/// A search can start an on-demand endpoint (a Jackett pod) and wait a
/// minute on it, then enrich for twenty seconds more.
const SEARCH_TIMEOUT: Duration = Duration::from_secs(120);
/// A refine on a nyaa endpoint searches again; a subscribe may probe Erai.
const STEP_TIMEOUT: Duration = Duration::from_secs(60);
/// A remove with `files` walks the download roots.
const REMOVE_TIMEOUT: Duration = Duration::from_secs(60);
/// How long a fresh sidecar gets to answer `/health`.
const READY_TIMEOUT: Duration = Duration::from_secs(20);
const READY_POLL: Duration = Duration::from_millis(200);
/// Lines of the sidecar's stderr kept for the failure message.
const STDERR_TAIL: usize = 40;

/// One server, one token.
#[derive(Clone)]
pub struct AnirssClient {
    /// `http://host:port/api/v1`, no trailing slash.
    base: String,
    token: String,
    http: Arc<dyn Http>,
}

/// qBittorrent as anirss sees it (`GET /qbt/status`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QbtView {
    pub url: String,
    pub reachable: bool,
    pub authenticated: bool,
    pub login_failed: bool,
}

/// `GET /search` before the analysis is folded in.
#[derive(Debug)]
pub struct SearchRaw {
    pub query: String,
    pub endpoint: String,
    pub items: Vec<Release>,
    pub tried: Vec<EndpointTry>,
    pub fell_back: bool,
}

/// `POST /refine/analyze`.
#[derive(Debug)]
pub struct Analysis {
    pub query: String,
    pub items: Vec<Release>,
    pub groups: Vec<ReleaseGroup>,
    pub best_fit: BestFit,
    pub client_side: bool,
    pub auto_resolution: Option<String>,
}

impl AnirssClient {
    /// `server` is the base URL without `/api/v1`, with or without a
    /// trailing slash.
    pub fn new(server: &str, token: &str, http: Arc<dyn Http>) -> AnirssClient {
        AnirssClient {
            base: format!("{}/api/v1", server.trim_end_matches('/')),
            token: token.to_string(),
            http,
        }
    }

    pub fn server(&self) -> String {
        self.base.trim_end_matches("/api/v1").to_string()
    }

    /// `GET /health`: the version. No token needed.
    pub async fn health(&self) -> Result<String, CoreError> {
        let v = self.get("/health", &[], None).await?;
        Ok(v["version"].as_str().unwrap_or("").to_string())
    }

    pub async fn endpoints(&self) -> Result<Vec<SearchEndpoint>, CoreError> {
        let v = self.get("/endpoints", &[], None).await?;
        Ok(serde_json::from_value(v["endpoints"].clone()).unwrap_or_default())
    }

    pub async fn qbt_status(&self) -> Result<QbtView, CoreError> {
        let v = self.get("/qbt/status", &[], None).await?;
        Ok(QbtView {
            url: v["url"].as_str().unwrap_or("").to_string(),
            reachable: v["reachable"].as_bool().unwrap_or(false),
            authenticated: v["authenticated"].as_bool().unwrap_or(false),
            login_failed: v["login_failed"].as_bool().unwrap_or(false),
        })
    }

    /// `POST /search`. With `fallback`, an empty or unreachable endpoint
    /// falls through to the others in anirss's order; zero results
    /// everywhere is an empty list, not an error.
    pub async fn search(
        &self,
        query: &str,
        endpoint: Option<&str>,
        fallback: bool,
    ) -> Result<SearchRaw, CoreError> {
        let mut body = json!({ "query": query, "fallback": fallback });
        if let Some(e) = endpoint {
            body["endpoint"] = Value::String(e.to_string());
        }
        let v = self.post("/search", body, Some(SEARCH_TIMEOUT)).await?;
        Ok(SearchRaw {
            query: v["query"].as_str().unwrap_or(query).to_string(),
            endpoint: v["endpoint"].as_str().unwrap_or("").to_string(),
            items: releases(&v["items"]),
            tried: serde_json::from_value(v["tried"].clone()).unwrap_or_default(),
            fell_back: v["fell_back"].as_bool().unwrap_or(false),
        })
    }

    /// `POST /refine/analyze`: the groups and the best fit, after
    /// auto-resolution.
    pub async fn analyze(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
    ) -> Result<Analysis, CoreError> {
        let body = json!({ "query": query, "endpoint": endpoint, "items": items });
        let v = self
            .post("/refine/analyze", body, Some(STEP_TIMEOUT))
            .await?;
        let auto = &v["auto_resolution"];
        Ok(Analysis {
            query: v["query"].as_str().unwrap_or(query).to_string(),
            items: releases(&v["items"]),
            groups: groups(&v["groups"]),
            best_fit: best_fit(&v["best_fit"]),
            client_side: v["client_side"].as_bool().unwrap_or(false),
            auto_resolution: auto["applied"]
                .as_bool()
                .unwrap_or(false)
                .then(|| auto["token"].as_str().unwrap_or("").to_string())
                .filter(|t| !t.is_empty()),
        })
    }

    /// `POST /refine/apply`, one step.
    pub async fn apply(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
        op: &RefineOp,
    ) -> Result<Refinement, CoreError> {
        let op = match op {
            RefineOp::Tokens { tokens } => json!({ "type": "tokens", "tokens": tokens }),
            RefineOp::Term { term } => json!({ "type": "term", "term": term }),
            RefineOp::Exclude { term } => json!({ "type": "exclude", "term": term }),
            RefineOp::BestFit => json!({ "type": "best_fit" }),
        };
        let body = json!({ "query": query, "endpoint": endpoint, "items": items, "op": op });
        let v = self.post("/refine/apply", body, Some(STEP_TIMEOUT)).await?;
        let count = v["items"].as_array().map(|a| a.len()).unwrap_or(0) as i64;
        let previous = v["previous_count"].as_u64().unwrap_or(items.len() as u64);
        Ok(Refinement {
            applied: v["applied"].as_bool().unwrap_or(false),
            reason: v["reason"].as_str().map(String::from),
            mode: v["mode"].as_str().unwrap_or("").to_string(),
            query: v["query"].as_str().unwrap_or(query).to_string(),
            endpoint: v["endpoint"].as_str().unwrap_or(endpoint).to_string(),
            items: releases(&v["items"]),
            groups: groups(&v["groups"]),
            best_fit: best_fit(&v["best_fit"]),
            previous_count: previous,
            delta: v["delta"].as_i64().unwrap_or(count - previous as i64),
            facts: Vec::new(),
            show: None,
        })
    }

    pub async fn preview(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
    ) -> Result<SubscriptionSpec, CoreError> {
        let body = json!({ "query": query, "endpoint": endpoint, "items": items });
        let v = self
            .post("/subscriptions/preview", body, Some(STEP_TIMEOUT))
            .await?;
        Ok(SubscriptionSpec {
            feed_url: v["feed_url"].as_str().unwrap_or("").to_string(),
            must_contain: v["must_contain"].as_str().unwrap_or("").to_string(),
            must_not_contain: v["must_not_contain"].as_str().unwrap_or("").to_string(),
            endpoint_name: v["endpoint_name"].as_str().unwrap_or("").to_string(),
        })
    }

    pub async fn subscribe(
        &self,
        query: &str,
        endpoint: &str,
        items: &[Release],
        name: Option<&str>,
    ) -> Result<Subscribed, CoreError> {
        let mut body = json!({ "query": query, "endpoint": endpoint, "items": items });
        if let Some(n) = name.map(str::trim).filter(|n| !n.is_empty()) {
            body["name"] = Value::String(n.to_string());
        }
        let v = self
            .post("/subscriptions", body, Some(STEP_TIMEOUT))
            .await?;
        Ok(Subscribed {
            name: v["name"].as_str().unwrap_or("").to_string(),
            feed_url: v["feed_url"].as_str().unwrap_or("").to_string(),
            save_path: v["save_path"].as_str().unwrap_or("").to_string(),
            must_contain: v["must_contain"].as_str().unwrap_or("").to_string(),
            must_not_contain: v["must_not_contain"].as_str().unwrap_or("").to_string(),
            tags: strings(&v["tags"]),
        })
    }

    pub async fn download(
        &self,
        items: &[Release],
        name: Option<&str>,
    ) -> Result<AddedTorrents, CoreError> {
        let mut body = json!({ "items": items });
        if let Some(n) = name.map(str::trim).filter(|n| !n.is_empty()) {
            body["name"] = Value::String(n.to_string());
        }
        let v = self.post("/downloads", body, Some(STEP_TIMEOUT)).await?;
        Ok(added(&v))
    }

    pub async fn movie(&self, link: &str) -> Result<AddedTorrents, CoreError> {
        let v = self
            .post("/movie", json!({ "link": link }), Some(STEP_TIMEOUT))
            .await?;
        Ok(added(&v))
    }

    pub async fn subscriptions(&self) -> Result<Vec<SubscriptionRule>, CoreError> {
        let v = self.get("/subscriptions", &[], Some(STEP_TIMEOUT)).await?;
        let mut rules: Vec<SubscriptionRule> =
            serde_json::from_value(v["subscriptions"].clone()).unwrap_or_default();
        for r in &mut rules {
            r.query = crate::subscriptions::decode_nyaa_query(&r.feed_url).unwrap_or_default();
        }
        Ok(rules)
    }

    /// `POST /subscriptions/remove`; with `dry_run` the plan and nothing
    /// else. The same route serves the plan and the removal, so the two
    /// can never disagree about what goes.
    pub async fn remove(
        &self,
        names: &[String],
        torrents: bool,
        files: bool,
        dry_run: bool,
    ) -> Result<Vec<RemovePlan>, CoreError> {
        let body =
            json!({ "names": names, "torrents": torrents, "files": files, "dry_run": dry_run });
        let v = self
            .post("/subscriptions/remove", body, Some(REMOVE_TIMEOUT))
            .await?;
        let plans = v["plans"].as_array().cloned().unwrap_or_default();
        Ok(plans
            .iter()
            .map(|p| RemovePlan {
                name: p["name"].as_str().unwrap_or("").to_string(),
                feed_url: p["feed_url"].as_str().unwrap_or("").to_string(),
                save_path: p["save_path"].as_str().unwrap_or("").to_string(),
                torrent_count_in_qb: p["torrent_count_in_qb"].as_u64().unwrap_or(0),
                size_bytes: p["size_bytes"].as_u64().unwrap_or(0),
                torrents: p["torrents"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|t| t["name"].as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
            .collect())
    }

    // The wire --------------------------------------------------------------

    async fn get(
        &self,
        path: &str,
        query: &[(&str, &str)],
        timeout: Option<Duration>,
    ) -> Result<Value, CoreError> {
        let mut url = format!("{}{}", self.base, path);
        if !query.is_empty() {
            let q: Vec<String> = query
                .iter()
                .map(|(k, v)| format!("{k}={}", encode(v)))
                .collect();
            url = format!("{url}?{}", q.join("&"));
        }
        let response = self
            .send(HttpRequest {
                method: Method::Get,
                url,
                headers: self.headers(),
                body: None,
                timeout,
            })
            .await?;
        judge(response)
    }

    async fn post(
        &self,
        path: &str,
        body: Value,
        timeout: Option<Duration>,
    ) -> Result<Value, CoreError> {
        let mut headers = self.headers();
        headers.push(("Content-Type".to_string(), "application/json".to_string()));
        let response = self
            .send(HttpRequest {
                method: Method::Post,
                url: format!("{}{}", self.base, path),
                headers,
                body: Some(Body::Json(body)),
                timeout,
            })
            .await?;
        judge(response)
    }

    fn headers(&self) -> Vec<(String, String)> {
        vec![
            ("Accept".to_string(), "application/json".to_string()),
            (
                "Authorization".to_string(),
                format!("Bearer {}", self.token),
            ),
        ]
    }

    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, CoreError> {
        self.http.send(req).await.map_err(|e| CoreError::Service {
            service: SERVICE.to_string(),
            code: "unreachable".to_string(),
            message: format!("anirss did not answer: {}", e.message),
        })
    }
}

/// A reply into its JSON, or the envelope's error. A success without a
/// JSON body (there is none in the API) reads as an empty object.
fn judge(response: HttpResponse) -> Result<Value, CoreError> {
    if response.is_success() {
        if response.body.is_empty() {
            return Ok(json!({}));
        }
        return response.json();
    }
    let envelope: Value = serde_json::from_slice(&response.body).unwrap_or(Value::Null);
    let code = envelope["error"]["type"]
        .as_str()
        .map(String::from)
        .unwrap_or_else(|| format!("http_{}", response.status));
    let message = envelope["error"]["message"]
        .as_str()
        .map(String::from)
        .unwrap_or_else(|| format!("anirss answered {}: {}", response.status, response.text()));
    Err(CoreError::Service {
        service: SERVICE.to_string(),
        code,
        message,
    })
}

/// Percent-encodes a query value the way a form does.
fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// An item list, one malformed item dropped rather than the whole reply.
fn releases(v: &Value) -> Vec<Release> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|i| serde_json::from_value(i.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn groups(v: &Value) -> Vec<ReleaseGroup> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|g| serde_json::from_value(g.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn best_fit(v: &Value) -> BestFit {
    BestFit {
        available: v["available"].as_bool().unwrap_or(false),
        query: v["query"].as_str().unwrap_or("").to_string(),
        item: serde_json::from_value(v["item"].clone()).ok(),
        profile: strings(&v["profile"]),
    }
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn added(v: &Value) -> AddedTorrents {
    AddedTorrents {
        count: v["count"].as_u64().unwrap_or(0),
        save_path: v["save_path"].as_str().unwrap_or("").to_string(),
        tags: strings(&v["tags"]),
        hashes: strings(&v["hashes"]),
    }
}

// The sidecar ----------------------------------------------------------------

/// `anirss --serve` as a child of the core.
pub struct Sidecar {
    child: Child,
    pub port: u16,
    pub token: String,
    stderr: Arc<Mutex<VecDeque<String>>>,
}

/// Why a spawn did not happen.
#[derive(Debug)]
pub enum SpawnFailure {
    /// The command is not on PATH: the "go install anirss" story.
    Missing,
    /// Anything else the OS said.
    Io(String),
}

impl Sidecar {
    /// Starts `command --serve --port <free port>` with the token and the
    /// qBittorrent password in its environment. Returns as soon as the
    /// process exists; `wait_ready` is the half that waits on `/health`.
    pub fn spawn(command: &str, qbt_password: Option<&str>) -> Result<Sidecar, SpawnFailure> {
        let port = free_port().map_err(|e| SpawnFailure::Io(format!("no free port: {e}")))?;
        let token = random_token();
        let mut cmd = Command::new(command);
        cmd.arg("--serve")
            .arg("--port")
            .arg(port.to_string())
            .env("ANIRSS_API_TOKEN", &token)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        match qbt_password {
            Some(pw) => {
                cmd.env("ANIRSS_QBT_PASSWORD", pw);
            }
            None => {
                cmd.env_remove("ANIRSS_QBT_PASSWORD");
            }
        }
        if let Some(path) = child_path() {
            cmd.env("PATH", path);
        }
        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SpawnFailure::Missing
            } else {
                SpawnFailure::Io(e.to_string())
            }
        })?;
        // Python's http.server logs every request to stderr, so the pipe
        // has to be drained or the child blocks once it fills. The last
        // lines are kept for a failure message; the rest is dropped.
        let stderr = Arc::new(Mutex::new(VecDeque::with_capacity(STDERR_TAIL)));
        if let Some(pipe) = child.stderr.take() {
            let tail = stderr.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(pipe).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let mut t = tail.lock().unwrap_or_else(|e| e.into_inner());
                    if t.len() >= STDERR_TAIL {
                        t.pop_front();
                    }
                    t.push_back(line);
                }
            });
        }
        Ok(Sidecar {
            child,
            port,
            token,
            stderr,
        })
    }

    pub fn server(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Polls `/health` until the server answers, the child exits, or the
    /// ready timeout passes. The version on success; what stderr said on
    /// failure.
    pub async fn wait_ready(&mut self, client: &AnirssClient) -> Result<String, String> {
        let started = tokio::time::Instant::now();
        loop {
            if let Ok(Some(status)) = self.child.try_wait() {
                return Err(format!("anirss exited ({status}){}", self.stderr_tail()));
            }
            if let Ok(version) = client.health().await {
                return Ok(version);
            }
            if started.elapsed() > READY_TIMEOUT {
                let _ = self.child.start_kill();
                return Err(format!(
                    "anirss did not answer on port {} within {} seconds{}",
                    self.port,
                    READY_TIMEOUT.as_secs(),
                    self.stderr_tail()
                ));
            }
            tokio::time::sleep(READY_POLL).await;
        }
    }

    /// True while the child is still running.
    pub fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn stop(&mut self) {
        let _ = self.child.start_kill();
    }

    fn stderr_tail(&self) -> String {
        let t = self.stderr.lock().unwrap_or_else(|e| e.into_inner());
        let lines: Vec<&str> = t
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if lines.is_empty() {
            String::new()
        } else {
            format!(": {}", lines[lines.len().saturating_sub(3)..].join(" / "))
        }
    }
}

/// A port nobody is listening on right now. Bound and released, so there
/// is a window in which something else could take it; a sidecar that then
/// fails to bind reports it through `wait_ready`, and the next connect
/// picks another.
fn free_port() -> std::io::Result<u16> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    Ok(listener.local_addr()?.port())
}

/// 32 hex characters off the OS's randomness.
fn random_token() -> String {
    use rand::Rng;
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `~/.local/bin` ahead of `PATH` when it is not already on it, so a
/// launch from a `.desktop` entry with a sparse `PATH` still finds a
/// user-installed anirss. `None` when nothing needs changing.
pub(super) fn child_path() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let user_bin = format!("{home}/.local/bin");
    let path = std::env::var("PATH").unwrap_or_default();
    if path.split(':').any(|segment| segment == user_bin) {
        return None;
    }
    Some(if path.is_empty() {
        user_bin
    } else {
        format!("{user_bin}:{path}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;

    fn client(http: &Arc<FakeHttp>) -> AnirssClient {
        AnirssClient::new("http://127.0.0.1:8787/", "tok", http.clone())
    }

    #[tokio::test]
    async fn health_reads_the_version_and_sends_the_bearer() {
        let http = Arc::new(FakeHttp::new());
        http.push_json(200, json!({"ok": true, "version": "0.3.4"}));
        let v = client(&http).health().await.unwrap();
        assert_eq!(v, "0.3.4");
        let sent = http.requests();
        assert_eq!(sent[0].url, "http://127.0.0.1:8787/api/v1/health");
        assert!(
            sent[0]
                .headers
                .iter()
                .any(|(k, v)| k == "Authorization" && v == "Bearer tok")
        );
    }

    #[tokio::test]
    async fn an_error_envelope_becomes_a_service_error_with_its_type() {
        let http = Arc::new(FakeHttp::new());
        http.push_json(
            503,
            json!({"error": {"type": "qbt_auth_required", "message": "fix the password"}}),
        );
        let err = client(&http).subscriptions().await.unwrap_err();
        match err {
            CoreError::Service {
                service,
                code,
                message,
            } => {
                assert_eq!(service, "anirss");
                assert_eq!(code, "qbt_auth_required");
                assert_eq!(message, "fix the password");
            }
            other => panic!("{other:?}"),
        }
    }

    #[tokio::test]
    async fn a_transport_failure_is_unreachable() {
        let http = Arc::new(FakeHttp::new());
        http.fail_next("connection refused");
        let err = client(&http).health().await.unwrap_err();
        assert!(matches!(err, CoreError::Service { code, .. } if code == "unreachable"));
    }

    #[tokio::test]
    async fn search_keeps_null_counts_as_unknown() {
        let http = Arc::new(FakeHttp::new());
        http.push_json(
            200,
            json!({"query": "Frieren", "endpoint": "erai", "fell_back": true,
                   "tried": [{"endpoint": "nyaa", "status": "empty", "count": 0, "error": null}],
                   "items": [{"title": "[Erai-raws] Frieren - 01", "link": "magnet:?x", "seeders": null,
                              "leechers": null, "downloads": null, "size": "", "category": "",
                              "page": "", "source": "erai", "resolution": "1080p", "infomkv": "", "linktype": ""}]}),
        );
        let s = client(&http).search("Frieren", None, true).await.unwrap();
        assert!(s.fell_back);
        assert_eq!(s.endpoint, "erai");
        assert_eq!(s.items[0].seeders, None);
        assert_eq!(s.tried[0].status, "empty");
    }

    #[tokio::test]
    async fn apply_sends_the_op_and_reads_the_outcome() {
        let http = Arc::new(FakeHttp::new());
        http.push_json(
            200,
            json!({"applied": false, "reason": "noop", "mode": "client", "query": "q", "endpoint": "erai",
                   "items": [], "previous_count": 0, "delta": 0, "groups": [],
                   "best_fit": {"available": false, "query": "", "item": null, "profile": []}}),
        );
        let r = client(&http)
            .apply(
                "q",
                "erai",
                &[],
                &RefineOp::Tokens {
                    tokens: vec!["[SubsPlease]".into()],
                },
            )
            .await
            .unwrap();
        assert!(!r.applied);
        assert_eq!(r.reason.as_deref(), Some("noop"));
        let body = match &http.requests()[0].body {
            Some(Body::Json(v)) => v.clone(),
            _ => panic!("json body"),
        };
        assert_eq!(body["op"]["type"], "tokens");
        assert_eq!(body["op"]["tokens"][0], "[SubsPlease]");
    }

    #[test]
    fn encode_is_form_style() {
        assert_eq!(encode("Frieren 1080p"), "Frieren+1080p");
        assert_eq!(encode("a&b"), "a%26b");
    }

    #[test]
    fn tokens_are_32_hex() {
        let t = random_token();
        assert_eq!(t.len(), 32);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(t, random_token());
    }
}
