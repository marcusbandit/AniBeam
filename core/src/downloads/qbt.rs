//! qBittorrent over its WebUI API (`/api/v2`), reached directly for what
//! anirss does not offer: the live transfer, the client's version as proof
//! that the saved settings work, and stopping, starting or deleting a
//! torrent. On anirss's engine anirss creates every torrent and rule; on
//! the core's own (`backend`) the core adds them here, the same way.
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
use crate::net::{Body, FilePart, Http, HttpRequest, HttpResponse, Method};

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

    /// The torrents with these info-hashes, and no others.
    pub async fn torrents_named(&self, hashes: &[String]) -> Result<Vec<Torrent>, CoreError> {
        if hashes.is_empty() {
            return Ok(Vec::new());
        }
        let r = self
            .call(&format!("torrents/info?hashes={}", hashes.join("|")), None)
            .await?;
        let v: Value = r.json()?;
        Ok(v.as_array()
            .map(|a| a.iter().map(torrent).collect())
            .unwrap_or_default())
    }

    /// The files inside one torrent, in qBittorrent's order.
    pub async fn files(&self, hash: &str) -> Result<Vec<TorrentFile>, CoreError> {
        let r = self
            .call(&format!("torrents/files?hash={hash}"), None)
            .await?;
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

    /// Where qBittorrent saves a torrent added without a path.
    pub async fn default_save_path(&self) -> Result<String, CoreError> {
        let r = self.call("app/defaultSavePath", None).await?;
        Ok(String::from_utf8_lossy(&r.body).trim().to_string())
    }

    /// Adds magnet and http(s) links, saved into `save_path`. qBittorrent
    /// fetches a web link itself. Tags go as one comma-separated field, and
    /// none at all when there are none, so the client's own defaults apply.
    pub async fn add_urls(
        &self,
        urls: &[String],
        save_path: &str,
        tags: &[String],
    ) -> Result<(), CoreError> {
        let mut form = vec![
            ("urls".to_string(), urls.join("\n")),
            ("savepath".to_string(), save_path.to_string()),
        ];
        form.extend(tags_field(tags));
        let r = self.call("torrents/add", Some(form)).await?;
        added_or_not(&r)
    }

    /// Uploads one .torrent file's bytes, saved into `save_path`: a URL
    /// only works for a link qBittorrent can fetch itself. A torrent it
    /// already has answers 409, which is no failure here.
    pub async fn add_file(
        &self,
        bytes: Vec<u8>,
        file_name: &str,
        save_path: &str,
        tags: &[String],
    ) -> Result<(), CoreError> {
        let mut fields = vec![("savepath".to_string(), save_path.to_string())];
        fields.extend(tags_field(tags));
        let file_name = if file_name.is_empty() {
            "file.torrent"
        } else {
            file_name
        };
        let body = Body::Multipart {
            fields,
            files: vec![FilePart {
                field: "torrents".to_string(),
                file_name: file_name.to_string(),
                content_type: "application/x-bittorrent".to_string(),
                bytes,
            }],
        };
        match self.call_with("torrents/add", Some(body)).await {
            Ok(r) => added_or_not(&r),
            Err(CoreError::Service { code, .. }) if code == "http_409" => Ok(()),
            Err(e) => Err(e),
        }
    }

    // ---- RSS: the engine that subscribes creates the feeds and rules; the
    // core reads them, and rewires one when the feed would come back empty
    // (see `downloads::rewire`)

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

    /// Download in order on or off. qBittorrent only has a toggle, so the
    /// current setting is read first and the toggle pressed for the torrents
    /// that differ; asking for what is already so changes nothing.
    pub async fn set_sequential(&self, hashes: &[String], on: bool) -> Result<(), CoreError> {
        self.set_flag("torrents/toggleSequentialDownload", hashes, on, |t| {
            t.seq_dl
        })
        .await
    }

    /// First and last pieces first on or off, set the same way.
    pub async fn set_first_last(&self, hashes: &[String], on: bool) -> Result<(), CoreError> {
        self.set_flag("torrents/toggleFirstLastPiecePrio", hashes, on, |t| {
            t.f_l_piece_prio
        })
        .await
    }

    async fn set_flag(
        &self,
        toggle: &str,
        hashes: &[String],
        on: bool,
        current: fn(&Torrent) -> bool,
    ) -> Result<(), CoreError> {
        let known = self.torrents_named(hashes).await?;
        let differ: Vec<String> = known
            .iter()
            .filter(|t| current(t) != on)
            .map(|t| t.hash.clone())
            .collect();
        if differ.is_empty() {
            return Ok(());
        }
        let form = vec![("hashes".to_string(), differ.join("|"))];
        self.call(toggle, Some(form)).await.map(|_| ())
    }

    /// Sets the priority of files inside one torrent by index (0 skips them).
    pub async fn set_file_priority(
        &self,
        hash: &str,
        ids: &[usize],
        priority: i64,
    ) -> Result<(), CoreError> {
        if ids.is_empty() {
            return Ok(());
        }
        let ids: Vec<String> = ids.iter().map(|i| i.to_string()).collect();
        let form = vec![
            ("hash".to_string(), hash.to_string()),
            ("id".to_string(), ids.join("|")),
            ("priority".to_string(), priority.to_string()),
        ];
        self.call("torrents/filePrio", Some(form)).await.map(|_| ())
    }

    /// Has the client check the torrents' data against their hashes again.
    pub async fn recheck(&self, hashes: &[String]) -> Result<(), CoreError> {
        let form = vec![("hashes".to_string(), hashes.join("|"))];
        self.call("torrents/recheck", Some(form)).await.map(|_| ())
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
        self.call_with(path, form.map(Body::Form)).await
    }

    /// `call` with any body: a POST, or a GET without one.
    async fn call_with(&self, path: &str, body: Option<Body>) -> Result<HttpResponse, CoreError> {
        let first = self.send(path, body.clone()).await?;
        if first.status != 403 {
            return judge(&self.base, first);
        }
        self.login().await?;
        judge(&self.base, self.send(path, body).await?)
    }

    async fn send(&self, path: &str, body: Option<Body>) -> Result<HttpResponse, CoreError> {
        let mut headers = vec![("Referer".to_string(), format!("{}/", self.base))];
        if let Some(cookie) = self.sid.lock().unwrap_or_else(|e| e.into_inner()).clone() {
            headers.push(("Cookie".to_string(), cookie));
        }
        let method = if body.is_some() {
            Method::Post
        } else {
            Method::Get
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

/// Tags trimmed, blanks dropped and repeats dropped in order. qBittorrent
/// splits tags on commas itself, so a comma inside one becomes two here.
pub fn clean_tags(tags: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tag in tags {
        for piece in tag.split(',') {
            let piece = piece.trim();
            if !piece.is_empty() && !out.iter().any(|t| t == piece) {
                out.push(piece.to_string());
            }
        }
    }
    out
}

/// `torrents/add`'s tags field, absent when there are no tags.
fn tags_field(tags: &[String]) -> Option<(String, String)> {
    let clean = clean_tags(tags);
    (!clean.is_empty()).then(|| ("tags".to_string(), clean.join(",")))
}

/// Whether `torrents/add` took the torrents. 4.x answers `Ok.`; 5.x a
/// JSON count, where a pending add is one still landing, not a failure.
fn added_or_not(r: &HttpResponse) -> Result<(), CoreError> {
    let body = String::from_utf8_lossy(&r.body);
    let body = body.trim();
    if body == "Ok." {
        return Ok(());
    }
    let failed = |message: String| {
        Err(CoreError::Service {
            service: SERVICE.to_string(),
            code: "add_failed".to_string(),
            message,
        })
    };
    let Ok(v) = serde_json::from_str::<Value>(body) else {
        return failed(format!("qBittorrent did not add the torrents: {body:?}"));
    };
    if !v.is_object() {
        return failed(format!("qBittorrent did not add the torrents: {body:?}"));
    }
    match num(&v["failure_count"]) {
        0 => Ok(()),
        n => failed(format!(
            "qBittorrent could not add {n} of the torrents: {body:?}"
        )),
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

fn judge(base: &str, r: HttpResponse) -> Result<HttpResponse, CoreError> {
    if r.is_success() {
        return Ok(r);
    }
    let code = match r.status {
        401 | 403 => "unauthorized".to_string(),
        s => format!("http_{s}"),
    };
    // qBittorrent's API errors are a line of plain text. A page of HTML is
    // some other program on the port, and 8080 is a popular one; reading
    // that page out to the user helps nobody.
    let text = r.text();
    let text = text.trim();
    let message = if text.starts_with('<') {
        format!(
            "{base} answered {}, but not as qBittorrent's Web UI. Check the address in Settings, \
             and that the Web UI is turned on in qBittorrent",
            r.status
        )
    } else {
        let line: String = text
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(160)
            .collect();
        format!("qBittorrent answered {}: {line}", r.status)
    };
    Err(CoreError::Service {
        service: SERVICE.to_string(),
        code,
        message,
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
        seq_dl: v["seq_dl"].as_bool().unwrap_or(false),
        f_l_piece_prio: v["f_l_piece_prio"].as_bool().unwrap_or(false),
        tracker: tracker_host(&text(&v["tracker"])),
        group: String::new(),
        resolution: String::new(),
        provider: String::new(),
        roll: None,
    }
}

/// `http://nyaa.tracker.wf:7777/announce` to `nyaa.tracker.wf`.
fn tracker_host(url: &str) -> String {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let host = rest.split(['/', '?']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or(host);
    match host.strip_prefix('[') {
        Some(v6) => v6.split(']').next().unwrap_or("").to_string(),
        None => host.split(':').next().unwrap_or("").to_string(),
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
    async fn a_web_page_on_the_port_is_not_read_out_as_qbittorrents_answer() {
        let http = Arc::new(FakeHttp::new());
        http.push_for(
            "app/version",
            404,
            "<!DOCTYPE html>\n<html lang=\"en\">\n<head><title>SearXNG</title>",
        );
        let err = client(&http).version().await.unwrap_err();
        let CoreError::Service { code, message, .. } = err else {
            panic!("{err:?}")
        };
        assert_eq!(code, "http_404");
        assert_eq!(
            message,
            "http://localhost:9090 answered 404, but not as qBittorrent's Web UI. Check the address \
             in Settings, and that the Web UI is turned on in qBittorrent"
        );
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

    #[test]
    fn a_torrent_reads_its_order_flags_and_tracker() {
        let t = torrent(
            &json!({"hash": "h", "seq_dl": true, "f_l_piece_prio": false,
                                "tracker": "http://nyaa.tracker.wf:7777/announce"}),
        );
        assert!(t.seq_dl);
        assert!(!t.f_l_piece_prio);
        assert_eq!(t.tracker, "nyaa.tracker.wf");
        assert_eq!(tracker_host("udp://[::1]:80/announce"), "::1");
        assert_eq!(tracker_host(""), "");
        assert!(!torrent(&Value::Null).seq_dl);
    }

    #[tokio::test]
    async fn sequential_presses_the_toggle_only_for_torrents_that_differ() {
        let http = Arc::new(FakeHttp::new());
        http.push_for(
            "torrents/info",
            200,
            json!([{"hash": "aa", "seq_dl": true}, {"hash": "bb", "seq_dl": false}]).to_string(),
        );
        http.push_for("torrents/toggleSequentialDownload", 200, "");
        client(&http)
            .set_sequential(&["aa".into(), "bb".into()], true)
            .await
            .unwrap();
        let sent = http.requests();
        assert_eq!(sent.len(), 2);
        assert!(sent[0].url.ends_with("torrents/info?hashes=aa|bb"));
        assert!(sent[1].url.ends_with("torrents/toggleSequentialDownload"));
        assert!(matches!(&sent[1].body, Some(Body::Form(f)) if f[0].1 == "bb"));
    }

    #[tokio::test]
    async fn sequential_already_so_sends_no_toggle() {
        let http = Arc::new(FakeHttp::new());
        http.push_for(
            "torrents/info",
            200,
            json!([{"hash": "aa", "seq_dl": true}]).to_string(),
        );
        client(&http)
            .set_sequential(&["aa".into()], true)
            .await
            .unwrap();
        assert_eq!(http.requests().len(), 1, "only the read");
        // And a torrent the client does not know is left alone
        let http = Arc::new(FakeHttp::new());
        http.push_for("torrents/info", 200, "[]");
        client(&http)
            .set_sequential(&["zz".into()], false)
            .await
            .unwrap();
        assert_eq!(http.requests().len(), 1);
    }

    #[tokio::test]
    async fn first_last_pieces_toggle_their_own_route() {
        let http = Arc::new(FakeHttp::new());
        http.push_for(
            "torrents/info",
            200,
            json!([{"hash": "aa", "f_l_piece_prio": false}]).to_string(),
        );
        http.push_for("torrents/toggleFirstLastPiecePrio", 200, "");
        client(&http)
            .set_first_last(&["aa".into()], true)
            .await
            .unwrap();
        assert!(
            http.requests()[1]
                .url
                .ends_with("torrents/toggleFirstLastPiecePrio")
        );
    }

    #[tokio::test]
    async fn file_priority_names_the_files_by_index() {
        let http = Arc::new(FakeHttp::new());
        http.push_for("torrents/filePrio", 200, "");
        client(&http)
            .set_file_priority("aa", &[0, 3], 0)
            .await
            .unwrap();
        let sent = http.requests();
        assert!(matches!(&sent[0].body, Some(Body::Form(f))
            if f.contains(&("id".to_string(), "0|3".to_string())) && f.contains(&("priority".to_string(), "0".to_string()))));
        client(&http).set_file_priority("aa", &[], 0).await.unwrap();
        assert_eq!(http.requests().len(), 1, "no files, no call");
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
    async fn add_urls_sends_the_links_a_line_each_with_the_folder_and_tags() {
        let http = Arc::new(FakeHttp::new());
        http.push_for("torrents/add", 200, "Ok.");
        let links = vec![
            "magnet:?xt=urn:btih:aa".to_string(),
            "https://dl.example/b.torrent".to_string(),
        ];
        client(&http)
            .add_urls(&links, "/srv/TV/Show", &[" TV, New ".into(), "TV".into()])
            .await
            .unwrap();
        let sent = http.requests();
        assert!(sent[0].url.ends_with("api/v2/torrents/add"));
        assert_eq!(
            sent[0].body,
            Some(Body::Form(vec![
                (
                    "urls".to_string(),
                    "magnet:?xt=urn:btih:aa\nhttps://dl.example/b.torrent".to_string()
                ),
                ("savepath".to_string(), "/srv/TV/Show".to_string()),
                ("tags".to_string(), "TV,New".to_string()),
            ]))
        );
        // No tags, no tags field: qBittorrent's own defaults apply
        http.push_for("torrents/add", 200, "Ok.");
        client(&http)
            .add_urls(&links[..1], "/srv/TV/Show", &[" ".into()])
            .await
            .unwrap();
        assert!(matches!(&http.requests()[1].body, Some(Body::Form(f)) if f.len() == 2));
    }

    #[tokio::test]
    async fn an_add_is_judged_by_its_answer() {
        let http = Arc::new(FakeHttp::new());
        let c = client(&http);
        let links = vec!["magnet:?xt=urn:btih:aa".to_string()];
        // 5.x counts; a pending add is still landing, not a failure
        http.push_for(
            "torrents/add",
            200,
            json!({"success_count": 0, "pending_count": 1, "failure_count": 0}).to_string(),
        );
        c.add_urls(&links, "/x", &[]).await.unwrap();
        http.push_for(
            "torrents/add",
            200,
            json!({"success_count": 0, "failure_count": 1}).to_string(),
        );
        let e = c.add_urls(&links, "/x", &[]).await.unwrap_err();
        assert!(
            matches!(&e, CoreError::Service { code, .. } if code == "add_failed"),
            "{e:?}"
        );
        http.push_for("torrents/add", 200, "Fails.");
        let e = c.add_urls(&links, "/x", &[]).await.unwrap_err();
        assert!(
            matches!(&e, CoreError::Service { code, .. } if code == "add_failed"),
            "{e:?}"
        );
    }

    #[tokio::test]
    async fn a_file_is_uploaded_as_a_multipart_form_and_a_duplicate_is_no_failure() {
        let http = Arc::new(FakeHttp::new());
        http.push_for("torrents/add", 200, "Ok.");
        let c = client(&http);
        c.add_file(
            b"d4:infoe".to_vec(),
            "a.torrent",
            "/srv/TV/Show",
            &["Anime".into()],
        )
        .await
        .unwrap();
        let sent = http.requests();
        assert_eq!(sent[0].method, Method::Post);
        assert_eq!(
            sent[0].body,
            Some(Body::Multipart {
                fields: vec![
                    ("savepath".to_string(), "/srv/TV/Show".to_string()),
                    ("tags".to_string(), "Anime".to_string()),
                ],
                files: vec![FilePart {
                    field: "torrents".to_string(),
                    file_name: "a.torrent".to_string(),
                    content_type: "application/x-bittorrent".to_string(),
                    bytes: b"d4:infoe".to_vec(),
                }],
            })
        );
        http.push_for("torrents/add", 409, "Conflict");
        c.add_file(b"d4:infoe".to_vec(), "", "/x", &[])
            .await
            .unwrap();
        assert!(matches!(&http.requests()[1].body,
            Some(Body::Multipart { files, .. }) if files[0].file_name == "file.torrent"));
    }

    #[tokio::test]
    async fn the_default_save_path_is_read_as_text() {
        let http = Arc::new(FakeHttp::new());
        http.push_for("app/defaultSavePath", 200, "/home/me/Downloads\n");
        assert_eq!(
            client(&http).default_save_path().await.unwrap(),
            "/home/me/Downloads"
        );
    }

    #[test]
    fn tags_are_trimmed_split_and_kept_once_in_order() {
        let tags: Vec<String> = vec!["Anime".into(), " a, b ,,".into(), "b".into(), "".into()];
        assert_eq!(clean_tags(&tags), vec!["Anime", "a", "b"]);
        assert_eq!(tags_field(&[" ".into()]), None);
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
