//! A subscription's feed, read by the core itself so the page can show
//! what it holds without sending the person to the feed's own page. Two
//! shapes reach here: nyaa's RSS, with its `nyaa:` fields beside each
//! item, and torznab (Jackett, Prowlarr), with `torznab:attr` name/value
//! pairs. Anything else is plain RSS 2.0 and gives what it gives.

use std::time::Duration;

use crate::contract::*;
use crate::net::{Http, HttpRequest, Method};

const FEED_TIMEOUT: Duration = Duration::from_secs(30);
/// More than any feed anirss registers holds; a broad one is cut here.
const MAX_ENTRIES: usize = 200;

/// Fetches and reads one feed. A non-2xx answer or a body that is not
/// XML is a `Service` error naming the feed's host.
pub async fn read(http: &dyn Http, url: &str) -> Result<Vec<FeedEntry>, CoreError> {
    let host = host_of(url);
    let response = http
        .send(HttpRequest {
            method: Method::Get,
            url: url.to_string(),
            headers: vec![(
                "Accept".to_string(),
                "application/rss+xml, application/xml, text/xml".to_string(),
            )],
            body: None,
            timeout: Some(FEED_TIMEOUT),
        })
        .await
        .map_err(|e| CoreError::Service {
            service: host.clone(),
            code: "unreachable".to_string(),
            message: e.message,
        })?;
    if !response.is_success() {
        return Err(CoreError::Service {
            service: host,
            code: format!("http_{}", response.status),
            message: format!("the feed answered {}", response.status),
        });
    }
    let text = String::from_utf8_lossy(&response.body);
    parse(&text).map_err(|message| CoreError::Service {
        service: host,
        code: "unreadable".to_string(),
        message,
    })
}

/// The host a URL names, for an error line: `nyaa.si`, `127.0.0.1:9117`.
pub fn host_of(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    rest.split(['/', '?', '#'])
        .next()
        .unwrap_or(rest)
        .to_string()
}

pub fn parse(text: &str) -> Result<Vec<FeedEntry>, String> {
    let doc = roxmltree::Document::parse(text).map_err(|e| format!("not a feed: {e}"))?;
    let mut out = Vec::new();
    for item in doc.descendants().filter(|n| n.has_tag_name("item")) {
        out.push(entry(item));
        if out.len() >= MAX_ENTRIES {
            break;
        }
    }
    Ok(out)
}

fn entry(item: roxmltree::Node) -> FeedEntry {
    // The child elements by local name, namespace aside: nyaa's
    // `nyaa:seeders` and a plain `seeders` read the same.
    let child = |name: &str| -> Option<String> {
        item.children()
            .find(|c| c.is_element() && c.tag_name().name() == name)
            .and_then(|c| c.text())
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
    };
    // torznab's `<torznab:attr name="seeders" value="12"/>`
    let attr = |name: &str| -> Option<String> {
        item.children()
            .find(|c| {
                c.is_element()
                    && c.tag_name().name() == "attr"
                    && c.attribute("name")
                        .is_some_and(|n| n.eq_ignore_ascii_case(name))
            })
            .and_then(|c| c.attribute("value"))
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let enclosure = item
        .children()
        .find(|c| c.is_element() && c.tag_name().name() == "enclosure");

    let title = child("title").unwrap_or_default();
    let guid = child("guid");
    let comments = child("comments");
    let link = child("link")
        .or_else(|| attr("magneturl"))
        .or_else(|| {
            enclosure
                .and_then(|e| e.attribute("url"))
                .map(str::to_string)
        })
        .unwrap_or_default();
    // The release's page: nyaa's guid is its view page; torznab names it
    // in comments, or in a guid that is a URL.
    let page = [guid, comments]
        .into_iter()
        .flatten()
        .find(|u| u.starts_with("http") && *u != link)
        .unwrap_or_default();
    let size_bytes = child("size")
        .and_then(|s| s.parse::<u64>().ok())
        .or_else(|| attr("size").and_then(|s| s.parse().ok()))
        .or_else(|| {
            enclosure
                .and_then(|e| e.attribute("length"))
                .and_then(|s| s.parse().ok())
        })
        .filter(|n| *n > 0);
    let size = child("size")
        .filter(|s| s.parse::<u64>().is_err())
        .or_else(|| size_bytes.map(human))
        .unwrap_or_default();
    let count = |name: &str| -> Option<i64> {
        child(name)
            .or_else(|| attr(name))
            .and_then(|s| s.parse::<i64>().ok())
    };
    let leechers = count("leechers").or_else(|| {
        // torznab carries peers (seeders and leechers together)
        let peers = attr("peers").and_then(|s| s.parse::<i64>().ok())?;
        let seeders = attr("seeders").and_then(|s| s.parse::<i64>().ok())?;
        Some((peers - seeders).max(0))
    });
    let info_hash = child("infoHash")
        .or_else(|| attr("infohash"))
        .or_else(|| hash_in(&link))
        .unwrap_or_default()
        .to_ascii_lowercase();
    FeedEntry {
        title,
        link,
        page,
        published: child("pubDate").and_then(|d| date(&d)),
        size,
        size_bytes,
        seeders: count("seeders"),
        leechers,
        downloads: count("downloads").or_else(|| count("grabs")),
        info_hash,
    }
}

/// RFC 2822, as RSS writes it; nyaa writes `-0000` for UTC, which the
/// parser takes.
fn date(text: &str) -> Option<i64> {
    time::OffsetDateTime::parse(text, &time::format_description::well_known::Rfc2822)
        .ok()
        .map(|d| d.unix_timestamp())
}

/// The btih in a magnet link, hex only (a base32 one is left alone).
fn hash_in(link: &str) -> Option<String> {
    let at = link.find("urn:btih:")? + "urn:btih:".len();
    let hash: String = link[at..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    (hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit())).then_some(hash)
}

fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = bytes as f64;
    let mut unit = 0;
    while v >= 1024.0 && unit < UNITS.len() - 1 {
        v /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;

    const NYAA: &str = r#"<rss xmlns:atom="http://www.w3.org/2005/Atom" xmlns:nyaa="https://nyaa.si/xmlns/nyaa" version="2.0">
<channel><title>Nyaa</title>
<item>
  <title>[ToonsHub] Show S01E12 1080p WEB-DL</title>
  <link>https://nyaa.si/download/2164565.torrent</link>
  <guid isPermaLink="true">https://nyaa.si/view/2164565</guid>
  <pubDate>Mon, 21 Sep 2026 14:34:40 -0000</pubDate>
  <nyaa:seeders>37</nyaa:seeders>
  <nyaa:leechers>0</nyaa:leechers>
  <nyaa:downloads>648</nyaa:downloads>
  <nyaa:infoHash>3AF8C29C1F96F548C7B5134A39730C2A138D4B02</nyaa:infoHash>
  <nyaa:size>1.4 GiB</nyaa:size>
</item>
<item><title>Second</title><link>https://nyaa.si/download/2.torrent</link></item>
</channel></rss>"#;

    const TORZNAB: &str = r#"<rss version="2.0" xmlns:torznab="http://torznab.com/schemas/2015/feed">
<channel><item>
  <title>Avatar Seven Havens S01E03 1080p</title>
  <guid>https://1337x.to/torrent/1/avatar/</guid>
  <link>http://127.0.0.1:9117/dl/1337x/?jackett_apikey=k&amp;path=x</link>
  <comments>https://1337x.to/torrent/1/avatar/</comments>
  <pubDate>Sat, 03 Oct 2026 10:00:00 +0000</pubDate>
  <size>1610612736</size>
  <enclosure url="http://127.0.0.1:9117/dl/1337x/?x" length="1610612736" type="application/x-bittorrent"/>
  <torznab:attr name="seeders" value="120"/>
  <torznab:attr name="peers" value="150"/>
  <torznab:attr name="grabs" value="900"/>
  <torznab:attr name="infohash" value="ABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCD"/>
</item></channel></rss>"#;

    #[test]
    fn nyaa_reads_its_own_fields() {
        let e = parse(NYAA).unwrap();
        assert_eq!(e.len(), 2);
        let a = &e[0];
        assert_eq!(a.title, "[ToonsHub] Show S01E12 1080p WEB-DL");
        assert_eq!(a.link, "https://nyaa.si/download/2164565.torrent");
        assert_eq!(a.page, "https://nyaa.si/view/2164565");
        assert_eq!(a.published, Some(1_790_001_280));
        assert_eq!(a.size, "1.4 GiB");
        assert_eq!(a.size_bytes, None);
        assert_eq!(
            (a.seeders, a.leechers, a.downloads),
            (Some(37), Some(0), Some(648))
        );
        assert_eq!(a.info_hash, "3af8c29c1f96f548c7b5134a39730c2a138d4b02");
        let b = &e[1];
        assert_eq!(b.title, "Second");
        assert_eq!(
            (b.seeders, b.published, b.info_hash.as_str()),
            (None, None, "")
        );
    }

    #[test]
    fn torznab_reads_its_attributes() {
        let e = parse(TORZNAB).unwrap();
        let a = &e[0];
        assert_eq!(a.page, "https://1337x.to/torrent/1/avatar/");
        assert_eq!(a.size_bytes, Some(1_610_612_736));
        assert_eq!(a.size, "1.5 GiB");
        assert_eq!(a.seeders, Some(120));
        assert_eq!(a.leechers, Some(30), "peers less seeders");
        assert_eq!(a.downloads, Some(900));
        assert_eq!(a.info_hash, "abcdefabcdefabcdefabcdefabcdefabcdefabcd");
    }

    #[test]
    fn a_magnet_link_gives_its_hash() {
        let xml = r#"<rss><channel><item><title>m</title><link>magnet:?xt=urn:btih:0123456789abcdef0123456789ABCDEF01234567&amp;dn=x</link></item></channel></rss>"#;
        assert_eq!(
            parse(xml).unwrap()[0].info_hash,
            "0123456789abcdef0123456789abcdef01234567"
        );
    }

    #[test]
    fn not_xml_is_an_error() {
        assert!(parse("<html><body>oops").is_err());
    }

    #[tokio::test]
    async fn read_names_the_host_when_the_feed_refuses() {
        let http = FakeHttp::new();
        http.push_for("nyaa.si", 503, "down");
        let err = read(http.as_ref(), "https://nyaa.si/?page=rss&q=x")
            .await
            .unwrap_err();
        match err {
            CoreError::Service { service, code, .. } => {
                assert_eq!(service, "nyaa.si");
                assert_eq!(code, "http_503");
            }
            other => panic!("{other:?}"),
        }
        http.push_for("nyaa.si", 200, NYAA);
        assert_eq!(
            read(http.as_ref(), "https://nyaa.si/?page=rss&q=x")
                .await
                .unwrap()
                .len(),
            2
        );
    }
}
