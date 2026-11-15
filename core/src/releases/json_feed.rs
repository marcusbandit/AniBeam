//! The `json-feed` kind: a JSON array of torrent entries in one fixed layout, `title` and
//! `torrent_name`, a `magnet_uri` or a `torrent_url`, `seeders`, `leechers`,
//! `torrent_downloaded_count`, `total_size` and the entry's `link`. Not an anirss kind. A
//! `json` endpoint reads any layout through the field map its definition gives; this one
//! needs none.

use std::time::Duration;

use crate::contract::*;
use crate::net::{Http, HttpRequest};

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

/// The feed's entries as releases. The torrent's own name is the title: the feed's title adds
/// an English name and tags after ` | `, which the title parsing would read as part of it.
pub fn parse(body: &[u8], source: &str) -> Result<Vec<Release>, String> {
    let entries: Vec<Entry> = serde_json::from_slice(body).map_err(|e| {
        crate::i18n::tr(
            "core-releases-not-feed-json",
            &[("error", e.to_string().into())],
        )
    })?;
    Ok(entries
        .into_iter()
        .filter_map(|e| {
            // A magnet first: a network that blocks the tracker may block its .torrent links too.
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
                size: e
                    .total_size
                    .map(super::fetch::size_text)
                    .unwrap_or_default(),
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

/// Searches a json-feed endpoint.
pub async fn fetch(
    http: &dyn Http,
    url: &str,
    source: &str,
    timeout: Duration,
) -> Result<Vec<Release>, CoreError> {
    let response = http
        .send(
            HttpRequest::get(url)
                .header("User-Agent", super::fetch::USER_AGENT)
                .header("Accept", "application/json")
                .timeout(timeout),
        )
        .await
        .map_err(|e| CoreError::service(source, "unreachable", e.message))?;
    if !response.is_success() {
        return Err(CoreError::service(
            source,
            format!("http_{}", response.status),
            crate::i18n::tr(
                "core-releases-feed-answered",
                &[("status", response.status.to_string().into())],
            ),
        ));
    }
    parse(&response.body, source)
        .map_err(|message| CoreError::service(source, "unreadable", message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_json_feed_gives_releases_with_magnets_and_the_torrent_name() {
        let body = br#"[{"id":1,"title":"[9volt] Frieren - S2 (WEB 1080p) | Frieren: Beyond (2026)",
            "torrent_name":"[9volt] Frieren - S2 (WEB 1080p)","link":"https://feed.example/view/x",
            "magnet_uri":"magnet:?xt=urn:btih:F7JB","torrent_url":"https://tracker.example/download/1.torrent",
            "seeders":12,"leechers":3,"torrent_downloaded_count":0,"total_size":1503238553},
            {"id":2,"title":"no links"}]"#;
        let items = parse(body, "feed").unwrap();
        assert_eq!(items.len(), 1, "an entry with no link is left out");
        assert_eq!(items[0].title, "[9volt] Frieren - S2 (WEB 1080p)");
        assert_eq!(items[0].link, "magnet:?xt=urn:btih:F7JB");
        assert_eq!(items[0].size, "1.4 GiB");
        assert_eq!(items[0].seeders, Some(12));
        assert_eq!(items[0].downloads, None);
        assert_eq!(items[0].source, "feed");
    }
}
