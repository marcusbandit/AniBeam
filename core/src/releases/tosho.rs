//! AnimeTosho, a mirror of nyaa's (and other trackers') releases with a JSON feed. Not an
//! anirss endpoint kind: the phone carries it as a second built-in source, for networks that
//! block nyaa. Its releases come with magnet links, so nothing is fetched from nyaa itself.

use std::time::Duration;

use crate::contract::*;
use crate::net::{Http, HttpRequest, Method};

/// The JSON feed's search URL. `feed.animetosho.org` does not resolve; the site's own path
/// serves the same feed.
pub const SEARCH_URL: &str = "https://animetosho.org/feed/json?q={query}";

/// One entry of the JSON feed, as far as a release needs.
#[derive(serde::Deserialize)]
struct Entry {
    #[serde(default)]
    title: String,
    #[serde(default)]
    torrent_name: Option<String>,
    #[serde(default)]
    link: Option<String>,
    #[serde(default)]
    magnet_uri: Option<String>,
    #[serde(default)]
    torrent_url: Option<String>,
    #[serde(default)]
    seeders: Option<i64>,
    #[serde(default)]
    leechers: Option<i64>,
    #[serde(default)]
    torrent_downloaded_count: Option<i64>,
    #[serde(default)]
    total_size: Option<u64>,
}

/// A size as nyaa writes it: `1.4 GiB`.
fn size_text(bytes: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u < units.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", units[u])
    }
}

/// The feed's entries as releases. The torrent's own name is the title: the feed's title adds
/// an English name and tags after ` | `, which the title parsing would read as part of it.
pub fn parse(body: &[u8], source: &str) -> Result<Vec<Release>, String> {
    let entries: Vec<Entry> =
        serde_json::from_slice(body).map_err(|e| format!("not the feed's JSON: {e}"))?;
    Ok(entries
        .into_iter()
        .filter_map(|e| {
            // A magnet first: the .torrent link points at nyaa, which a network may block.
            let link = e.magnet_uri.or(e.torrent_url)?;
            let title = e
                .torrent_name
                .filter(|t| !t.trim().is_empty())
                .unwrap_or(e.title);
            Some(Release {
                title,
                link,
                seeders: e.seeders,
                leechers: e.leechers,
                downloads: e.torrent_downloaded_count.filter(|n| *n > 0),
                size: e.total_size.map(size_text).unwrap_or_default(),
                category: String::new(),
                page: e.link.unwrap_or_default(),
                source: source.to_string(),
                resolution: String::new(),
                infomkv: String::new(),
                linktype: String::new(),
            })
        })
        .collect())
}

/// Searches AnimeTosho.
pub async fn fetch(
    http: &dyn Http,
    url: &str,
    source: &str,
    timeout: Duration,
) -> Result<Vec<Release>, CoreError> {
    let response = http
        .send(HttpRequest {
            method: Method::Get,
            url: url.to_string(),
            headers: vec![
                (
                    "User-Agent".to_string(),
                    super::fetch::USER_AGENT.to_string(),
                ),
                ("Accept".to_string(), "application/json".to_string()),
            ],
            body: None,
            timeout: Some(timeout),
        })
        .await
        .map_err(|e| CoreError::Service {
            service: source.to_string(),
            code: "unreachable".into(),
            message: e.message,
        })?;
    if !response.is_success() {
        return Err(CoreError::Service {
            service: source.to_string(),
            code: format!("http_{}", response.status),
            message: format!("the feed answered {}", response.status),
        });
    }
    parse(&response.body, source).map_err(|message| CoreError::Service {
        service: source.to_string(),
        code: "unreadable".into(),
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_json_feed_gives_releases_with_magnets_and_the_torrent_name() {
        let body = br#"[{"id":1,"title":"[9volt] Frieren - S2 (WEB 1080p) | Frieren: Beyond (2026)",
            "torrent_name":"[9volt] Frieren - S2 (WEB 1080p)","link":"https://animetosho.org/view/x",
            "magnet_uri":"magnet:?xt=urn:btih:F7JB","torrent_url":"https://nyaa.si/download/1.torrent",
            "seeders":12,"leechers":3,"torrent_downloaded_count":0,"total_size":1503238553},
            {"id":2,"title":"no links"}]"#;
        let items = parse(body, "animetosho").unwrap();
        assert_eq!(items.len(), 1, "an entry with no link is left out");
        assert_eq!(items[0].title, "[9volt] Frieren - S2 (WEB 1080p)");
        assert_eq!(items[0].link, "magnet:?xt=urn:btih:F7JB");
        assert_eq!(items[0].size, "1.4 GiB");
        assert_eq!(items[0].seeders, Some(12));
        assert_eq!(items[0].downloads, None);
        assert_eq!(items[0].source, "animetosho");
    }
}
