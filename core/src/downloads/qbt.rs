//! qBittorrent over its WebUI API (`/api/v2`), reached directly for what
//! anirss does not offer: the live transfer, the client's version as proof
//! that the saved settings work, and stopping, starting or deleting a
//! torrent. anirss owns everything that creates a torrent or a rule.
//!
//! The session is a cookie from `auth/login` (`SID` on 4.x, `QBT_SID_<port>`
//! on 5.x, so the name is whatever the server said); reqwest here has no
//! cookie store, so the cookie is carried by hand as the `name=value` pair. A 403 mid-call means
//! the session expired: one re-login, one retry. qBittorrent bans an
//! address after a few failed logins, so a rejected password is not
//! retried until the client is rebuilt from new settings. Version 5
//! renamed `pause`/`resume` to `stop`/`start`; both names are tried, the
//! new one first, so a 4.x server still works.

use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::contract::*;
use crate::net::{Body, Http, HttpRequest, HttpResponse, Method};

/// How `CoreError::Service` names this service.
pub const SERVICE: &str = "qBittorrent";

pub struct QbtClient {
    /// `http://host:port`, no trailing slash.
    base: String,
    username: String,
    password: String,
    http: Arc<dyn Http>,
    sid: Mutex<Option<String>>,
    /// Set once the server said the password is wrong, so no later call
    /// walks into the ban.
    rejected: Mutex<bool>,
}

impl QbtClient {
    pub fn new(base: &str, username: &str, password: &str, http: Arc<dyn Http>) -> QbtClient {
        QbtClient {
            base: base.trim_end_matches('/').to_string(),
            username: username.to_string(),
            password: password.to_string(),
            http,
            sid: Mutex::new(None),
            rejected: Mutex::new(false),
        }
    }

    pub async fn version(&self) -> Result<String, CoreError> {
        let r = self.call("app/version", None).await?;
        Ok(String::from_utf8_lossy(&r.body).trim().to_string())
    }

    pub async fn transfer(&self) -> Result<TransferRate, CoreError> {
        let r = self.call("transfer/info", None).await?;
        let v: Value = r.json()?;
        Ok(TransferRate {
            dl_speed: num(&v["dl_info_speed"]),
            up_speed: num(&v["up_info_speed"]),
            connection_status: v["connection_status"].as_str().unwrap_or("").to_string(),
            dht_nodes: num(&v["dht_nodes"]),
        })
    }

    pub async fn torrents(&self) -> Result<Vec<Torrent>, CoreError> {
        let r = self.call("torrents/info", None).await?;
        let v: Value = r.json()?;
        Ok(v.as_array()
            .map(|a| a.iter().map(torrent).collect())
            .unwrap_or_default())
    }

    /// The files inside one torrent, in qBittorrent's order.
    pub async fn files(&self, hash: &str) -> Result<Vec<TorrentFile>, CoreError> {
        let r = self.call(&format!("torrents/files?hash={hash}"), None).await?;
        let v: Value = r.json()?;
        Ok(v.as_array()
            .map(|a| {
                a.iter()
                    .map(|f| TorrentFile {
                        name: text(&f["name"]),
                        size: num(&f["size"]),
                        progress: f["progress"].as_f64().unwrap_or(0.0).clamp(0.0, 1.0),
                        priority: int(&f["priority"]),
                        episode: None,
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    // ---- RSS: anirss creates the feeds and rules; the core reads them, and
    // rewires one when anirss's feed would come back empty (see
    // `downloads::rewire`)

    /// Every auto-download rule by name, as qBittorrent writes them.
    pub async fn rules(&self) -> Result<Value, CoreError> {
        self.call("rss/rules", None).await?.json()
    }

    /// Replaces the named rule with `def` whole.
    pub async fn set_rule(&self, name: &str, def: &Value) -> Result<(), CoreError> {
        let form = vec![
            ("ruleName".to_string(), name.to_string()),
            ("ruleDef".to_string(), def.to_string()),
        ];
        self.call("rss/setRule", Some(form)).await.map(|_| ())
    }

    /// Every feed as (path, url): the tree flattened, folders joined with
    /// qBittorrent's own `\`.
    pub async fn feeds(&self) -> Result<Vec<(String, String)>, CoreError> {
        let v: Value = self.call("rss/items", None).await?.json()?;
        let mut out = Vec::new();
        flatten_feeds(&v, "", &mut out);
        Ok(out)
    }

    /// Adds a feed at `path`. False when qBittorrent already has a feed at
    /// that path or with that URL (409), which is no failure here.
    pub async fn add_feed(&self, url: &str, path: &str) -> Result<bool, CoreError> {
        let form = vec![
            ("url".to_string(), url.to_string()),
            ("path".to_string(), path.to_string()),
        ];
        match self.call("rss/addFeed", Some(form)).await {
            Ok(_) => Ok(true),
            Err(CoreError::Service { code, .. }) if code == "http_409" => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub async fn remove_item(&self, path: &str) -> Result<(), CoreError> {
        let form = vec![("path".to_string(), path.to_string())];
        self.call("rss/removeItem", Some(form)).await.map(|_| ())
    }

    pub async fn remove_rule(&self, name: &str) -> Result<(), CoreError> {
        let form = vec![("ruleName".to_string(), name.to_string())];
        self.call("rss/removeRule", Some(form)).await.map(|_| ())
    }

    /// The titles of the articles in the rule's feeds that the rule accepts,
    /// as qBittorrent judges them now.
    pub async fn matching_articles(&self, rule: &str) -> Result<Vec<String>, CoreError> {
        let path = format!(
            "rss/matchingArticles?ruleName={}",
            crate::percent::encode(rule)
        );
        let v: Value = self.call(&path, None).await?.json()?;
        let mut out = Vec::new();
        if let Some(feeds) = v.as_object() {
            for titles in feeds.values() {
                for t in titles.as_array().into_iter().flatten() {
                    if let Some(t) = t.as_str() {
                        out.push(t.to_string());
                    }
                }
            }
        }
        Ok(out)
    }

    pub async fn stop(&self, hashes: &[String]) -> Result<(), CoreError> {
        self.either("torrents/stop", "torrents/pause", hashes).await
    }

    pub async fn start(&self, hashes: &[String]) -> Result<(), CoreError> {
        self.either("torrents/start", "torrents/resume", hashes)
            .await
    }

    pub async fn delete(&self, hashes: &[String], files: bool) -> Result<(), CoreError> {
        let form = vec![
            ("hashes".to_string(), hashes.join("|")),
            ("deleteFiles".to_string(), files.to_string()),
        ];
        self.call("torrents/delete", Some(form)).await.map(|_| ())
    }

    /// The 5.x route, then the 4.x one when the server has no such path.
    async fn either(&self, new: &str, old: &str, hashes: &[String]) -> Result<(), CoreError> {
        let form = vec![("hashes".to_string(), hashes.join("|"))];
        match self.call(new, Some(form.clone())).await {
            Err(CoreError::Service { code, .. }) if code == "http_404" => {
                self.call(old, Some(form)).await.map(|_| ())
            }
            other => other.map(|_| ()),
        }
    }

    // The wire --------------------------------------------------------------

    /// One API call, logged in as needed. Reads first without a session:
    /// a WebUI that bypasses authentication for this address answers as
    /// it is, and only a 403 sends it to the login.
    async fn call(
        &self,
        path: &str,
        form: Option<Vec<(String, String)>>,
    ) -> Result<HttpResponse, CoreError> {
        let first = self.send(path, form.clone()).await?;
        if first.status != 403 {
            return judge(first);
        }
        self.login().await?;
        judge(self.send(path, form).await?)
    }

    async fn send(
        &self,
        path: &str,
        form: Option<Vec<(String, String)>>,
    ) -> Result<HttpResponse, CoreError> {
        let mut headers = vec![("Referer".to_string(), format!("{}/", self.base))];
        if let Some(cookie) = self.sid.lock().unwrap_or_else(|e| e.into_inner()).clone() {
            headers.push(("Cookie".to_string(), cookie));
        }
        let (method, body) = match form {
            Some(f) => (Method::Post, Some(Body::Form(f))),
            None => (Method::Get, None),
        };
        self.http
            .send(HttpRequest {
                method,
                url: format!("{}/api/v2/{path}", self.base),
                headers,
                body,
                timeout: None,
            })
            .await
            .map_err(|e| CoreError::Service {
                service: SERVICE.to_string(),
                code: "unreachable".to_string(),
                message: format!("qBittorrent did not answer at {}: {}", self.base, e.message),
            })
    }

    async fn login(&self) -> Result<(), CoreError> {
        if *self.rejected.lock().unwrap_or_else(|e| e.into_inner()) {
            return Err(unauthorized(
                "qBittorrent rejected the password; fix it in Settings",
            ));
        }
        let form = vec![
            ("username".to_string(), self.username.clone()),
            ("password".to_string(), self.password.clone()),
        ];
        let r = self
            .http
            .send(HttpRequest {
                method: Method::Post,
                url: format!("{}/api/v2/auth/login", self.base),
                headers: vec![("Referer".to_string(), format!("{}/", self.base))],
                body: Some(Body::Form(form)),
                timeout: None,
            })
            .await
            .map_err(|e| CoreError::Service {
                service: SERVICE.to_string(),
                code: "unreachable".to_string(),
                message: format!("qBittorrent did not answer at {}: {}", self.base, e.message),
            })?;
        if r.status == 403 {
            return Err(CoreError::Service {
                service: SERVICE.to_string(),
                code: "banned".to_string(),
                message: "qBittorrent has banned this address after failed logins; wait, or restart qBittorrent".to_string(),
            });
        }
        // 4.x answers a good login with `Ok.` and a bad one with `Fails.`, both 200;
        // 5.x answers a good one with 204 and nothing. Only a refusal is a rejection.
        let text = String::from_utf8_lossy(&r.body).trim().to_string();
        if !r.is_success() || text == "Fails." {
            *self.rejected.lock().unwrap_or_else(|e| e.into_inner()) = true;
            return Err(unauthorized(
                "qBittorrent rejected the username or password",
            ));
        }
        // The cookie's own `name=value`, the first part before the attributes
        let sid = r
            .headers
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
            .find_map(|(_, v)| {
                let pair = v.split(';').next().unwrap_or("").trim();
                let (name, value) = pair.split_once('=')?;
                (name.to_ascii_uppercase().contains("SID") && !value.is_empty())
                    .then(|| pair.to_string())
            });
        match sid {
            Some(sid) => {
                *self.sid.lock().unwrap_or_else(|e| e.into_inner()) = Some(sid);
                Ok(())
            }
            // Logged in without a cookie: an auth bypass answered the
            // login too. Nothing to carry; later calls go as they are.
            None => Ok(()),
        }
    }
}

fn unauthorized(message: &str) -> CoreError {
    CoreError::Service {
        service: SERVICE.to_string(),
        code: "unauthorized".to_string(),
        message: message.to_string(),
    }
}

fn flatten_feeds(v: &Value, prefix: &str, out: &mut Vec<(String, String)>) {
    let Some(map) = v.as_object() else { return };
    for (name, item) in map {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}\\{name}")
        };
        match item.get("url").and_then(|u| u.as_str()) {
            Some(url) => out.push((path, url.to_string())),
            None => flatten_feeds(item, &path, out),
        }
    }
}

fn judge(r: HttpResponse) -> Result<HttpResponse, CoreError> {
    if r.is_success() {
        return Ok(r);
    }
    let code = match r.status {
        401 | 403 => "unauthorized".to_string(),
        s => format!("http_{s}"),
    };
    Err(CoreError::Service {
        service: SERVICE.to_string(),
        code,
        message: format!("qBittorrent answered {}: {}", r.status, r.text()),
    })
}

/// A count off qBittorrent's JSON: negative (its "unknown") and absent
/// both read as nought.
fn num(v: &Value) -> u64 {
    v.as_u64()
        .or_else(|| v.as_i64().map(|i| i.max(0) as u64))
        .or_else(|| v.as_f64().map(|f| f.max(0.0) as u64))
        .unwrap_or(0)
}

fn int(v: &Value) -> i64 {
    v.as_i64()
        .or_else(|| v.as_f64().map(|f| f as i64))
        .unwrap_or(0)
}

fn text(v: &Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

fn torrent(v: &Value) -> Torrent {
    Torrent {
        hash: text(&v["hash"]),
        name: text(&v["name"]),
        state: text(&v["state"]),
        progress: v["progress"].as_f64().unwrap_or(0.0).clamp(0.0, 1.0),
        size: num(&v["size"]),
        dlspeed: num(&v["dlspeed"]),
        upspeed: num(&v["upspeed"]),
        eta: int(&v["eta"]),
        num_seeds: num(&v["num_seeds"]),
        num_leechs: num(&v["num_leechs"]),
        ratio: v["ratio"].as_f64().unwrap_or(0.0).max(0.0),
        save_path: text(&v["save_path"]),
        content_path: text(&v["content_path"]),
        added_on: int(&v["added_on"]),
        completion_on: int(&v["completion_on"]),
        tags: text(&v["tags"]),
        category: text(&v["category"]),
        series_id: None,
        episode: None,
        batch: false,
        files: Vec::new(),
    }
}

/// A blank torrent for other modules' tests.
#[cfg(test)]
pub(crate) fn tests_torrent() -> Torrent {
    torrent(&Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;
    use serde_json::json;

    fn client(http: &Arc<FakeHttp>) -> QbtClient {
        QbtClient::new("http://localhost:9090/", "me", "pw", http.clone())
    }

    #[tokio::test]
    async fn a_403_logs_in_once_and_carries_the_cookie() {
        let http = Arc::new(FakeHttp::new());
        http.push_for("app/version", 403, "Forbidden");
        http.push_for_with_headers(
            "auth/login",
            200,
            "Ok.",
            vec![(
                "set-cookie".to_string(),
                "SID=abc123; HttpOnly; path=/".to_string(),
            )],
        );
        http.push_for("app/version", 200, "v5.2.4");
        let c = client(&http);
        assert_eq!(c.version().await.unwrap(), "v5.2.4");
        let sent = http.requests();
        assert_eq!(sent.len(), 3);
        assert!(
            matches!(&sent[1].body, Some(Body::Form(f)) if f[0] == ("username".to_string(), "me".to_string()))
        );
        assert!(
            sent[2]
                .headers
                .iter()
                .any(|(k, v)| k == "Cookie" && v == "SID=abc123")
        );
    }

    #[tokio::test]
    async fn a_5x_login_answers_204_with_only_the_cookie() {
        let http = Arc::new(FakeHttp::new());
        http.push_for("app/version", 403, "");
        http.push_for_with_headers(
            "auth/login",
            204,
            "",
            vec![(
                "set-cookie".to_string(),
                "QBT_SID_9090=9R//Ql; HttpOnly; SameSite=Lax; path=/".to_string(),
            )],
        );
        http.push_for("app/version", 200, "v5.2.4");
        assert_eq!(client(&http).version().await.unwrap(), "v5.2.4");
        assert!(
            http.requests()[2]
                .headers
                .iter()
                .any(|(k, v)| k == "Cookie" && v == "QBT_SID_9090=9R//Ql")
        );
    }

    #[tokio::test]
    async fn a_rejected_password_is_not_retried() {
        let http = Arc::new(FakeHttp::new());
        http.push_for("app/version", 403, "");
        http.push_for("auth/login", 200, "Fails.");
        let c = client(&http);
        let e = c.version().await.unwrap_err();
        assert!(matches!(&e, CoreError::Service { code, .. } if code == "unauthorized"));
        // The next call still tries without a session (a bypass could answer),
        // but a 403 must not reach the login again
        http.push_for("transfer/info", 403, "");
        let e2 = c.transfer().await.unwrap_err();
        assert!(matches!(&e2, CoreError::Service { code, .. } if code == "unauthorized"));
        assert_eq!(
            http.requests().len(),
            3,
            "version, login, transfer: no second login"
        );
    }

    #[tokio::test]
    async fn stop_falls_back_to_pause_on_a_4x_server() {
        let http = Arc::new(FakeHttp::new());
        http.push_for("torrents/stop", 404, "Not Found");
        http.push_for("torrents/pause", 200, "");
        client(&http)
            .stop(&["h1".into(), "h2".into()])
            .await
            .unwrap();
        let sent = http.requests();
        assert!(sent[1].url.ends_with("torrents/pause"));
        assert!(matches!(&sent[1].body, Some(Body::Form(f)) if f[0].1 == "h1|h2"));
    }

    #[test]
    fn a_torrent_reads_leniently() {
        let t = torrent(
            &json!({"hash": "h", "name": "n", "state": "downloading", "progress": 0.5,
                                "size": 100, "dlspeed": -1, "eta": 8640000, "num_seeds": 3,
                                "ratio": 0.4, "tags": "Anime"}),
        );
        assert_eq!(t.dlspeed, 0);
        assert_eq!(t.eta, 8640000);
        assert_eq!(t.num_seeds, 3);
        assert_eq!(t.content_path, "");
    }

    #[tokio::test]
    async fn feeds_flatten_folders_and_a_taken_url_is_no_failure() {
        let http = FakeHttp::new();
        let client = client(&http);
        http.push(
            200,
            json!({"Frieren": {"uid": "a", "url": "https://nyaa.si/?q=f"},
                   "TV": {"Avatar": {"uid": "b", "url": "http://127.0.0.1:9117/x"}}})
            .to_string(),
        );
        let mut feeds = client.feeds().await.unwrap();
        feeds.sort();
        assert_eq!(
            feeds,
            vec![
                ("Frieren".to_string(), "https://nyaa.si/?q=f".to_string()),
                (
                    "TV\\Avatar".to_string(),
                    "http://127.0.0.1:9117/x".to_string()
                ),
            ]
        );
        http.push(409, "Conflict");
        assert!(
            !client
                .add_feed("https://nyaa.si/?q=f", "Frieren")
                .await
                .unwrap()
        );
        http.push(200, "");
        assert!(client.add_feed("https://nyaa.si/?q=g", "G").await.unwrap());
    }

    #[tokio::test]
    async fn matching_articles_gather_every_feed() {
        let http = FakeHttp::new();
        let client = client(&http);
        http.push(
            200,
            json!({"A": ["one", "two"], "B": ["three"]}).to_string(),
        );
        let mut got = client.matching_articles("Helluva Boss").await.unwrap();
        got.sort();
        assert_eq!(got, vec!["one", "three", "two"]);
        assert!(
            http.requests()[0]
                .url
                .ends_with("rss/matchingArticles?ruleName=Helluva%20Boss")
                || http.requests()[0]
                    .url
                    .ends_with("rss/matchingArticles?ruleName=Helluva+Boss")
        );
    }
}
