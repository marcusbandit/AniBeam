//! TsukiHime, a torrent index with a JSON search API and no RSS feed. Not an anirss endpoint
//! kind: the `tsukihime` kind a repo definition can name. The API takes nyaa's `-word`,
//! `!word`, `a | b` and "exact phrase" syntax itself, so the query goes through whole. Its
//! results carry an info-hash rather than a link, which becomes a magnet here.

use std::time::Duration;

use crate::contract::*;
use crate::net::{Http, HttpRequest, Method};

/// The search reply, as far as releases need.
#[derive(serde::Deserialize)]
struct Answer {
    #[serde(default)]
    error: bool,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    results: Vec<Entry>,
}

/// One search result.
#[derive(serde::Deserialize)]
struct Entry {
    #[serde(default)]
    name: String,
    #[serde(default)]
    btih: String,
    #[serde(default)]
    totalsize: Option<u64>,
    #[serde(default)]
    nyaa_id: Option<u64>,
    /// `0` or `1`; a boolean is read the same way.
    #[serde(default)]
    is_adult: serde_json::Value,
}

impl Entry {
    fn adult(&self) -> bool {
        match &self.is_adult {
            serde_json::Value::Bool(b) => *b,
            v => v.as_u64().is_some_and(|n| n != 0),
        }
    }
}

/// The magnet for an info-hash, named after the torrent.
fn magnet(btih: &str, name: &str) -> String {
    format!(
        "magnet:?xt=urn:btih:{btih}&dn={}",
        super::endpoints::quote(name.as_bytes(), "")
    )
}

/// The reply's results as releases. Adult results are left out: a general search should not
/// surface them, and whether a source is nsfw is its definition's call. Counts stay unknown,
/// since the API has none.
pub fn parse(body: &[u8], source: &str) -> Result<Vec<Release>, String> {
    let answer: Answer =
        serde_json::from_slice(body).map_err(|e| format!("not the search API's JSON: {e}"))?;
    if answer.error {
        return Err(match answer.message.filter(|m| !m.trim().is_empty()) {
            Some(m) => format!("the search answered with an error: {m}"),
            None => "the search answered with an error".into(),
        });
    }
    Ok(answer
        .results
        .into_iter()
        .filter(|e| !e.adult())
        .filter_map(|e| {
            let btih = e.btih.trim();
            if btih.is_empty() {
                return None;
            }
            Some(Release {
                link: magnet(btih, &e.name),
                title: e.name,
                seeders: None,
                leechers: None,
                downloads: None,
                size: e.totalsize.map(super::tosho::size_text).unwrap_or_default(),
                category: String::new(),
                page: e
                    .nyaa_id
                    .filter(|id| *id > 0)
                    .map(|id| format!("https://nyaa.si/view/{id}"))
                    .unwrap_or_default(),
                source: source.to_string(),
                resolution: String::new(),
                infomkv: String::new(),
                linktype: String::new(),
            })
        })
        .collect())
}

/// Searches TsukiHime.
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
            message: format!("the search answered {}", response.status),
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

    fn fixture() -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tsukihime.json"),
        )
        .unwrap()
    }

    #[test]
    fn a_reply_gives_releases_named_after_the_torrent_with_unknown_counts() {
        let items = parse(&fixture(), "tsukihime").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].title,
            "[BlackRabbit] Frieren - Beyond Journey's End (2023) - S01 v2 [Bluray-1080p][Opus 2.0][Dual Audio][AV1]"
        );
        assert_eq!(items[0].size, "7.8 GiB");
        assert_eq!(items[0].source, "tsukihime");
        assert_eq!(
            (items[0].seeders, items[0].leechers, items[0].downloads),
            (None, None, None)
        );
    }

    #[test]
    fn adult_results_are_left_out() {
        let items = parse(&fixture(), "tsukihime").unwrap();
        assert!(items.iter().all(|i| !i.title.contains("Adult Release")));
        let one = br#"{"error":false,"results":[{"name":"x","btih":"ab","is_adult":true}]}"#;
        assert!(parse(one, "t").unwrap().is_empty());
    }

    #[test]
    fn the_link_is_a_magnet_from_the_info_hash_and_the_name() {
        let items = parse(&fixture(), "tsukihime").unwrap();
        assert_eq!(
            items[0].link,
            "magnet:?xt=urn:btih:05bd3a45348e1338117e89cf1e25fd76b7a30de3&dn=%5BBlackRabbit%5D%20Frieren%20-%20Beyond%20Journey%27s%20End%20%282023%29%20-%20S01%20v2%20%5BBluray-1080p%5D%5BOpus%202.0%5D%5BDual%20Audio%5D%5BAV1%5D"
        );
        let none = br#"{"error":false,"results":[{"name":"no hash","btih":""}]}"#;
        assert!(
            parse(none, "t").unwrap().is_empty(),
            "a result without a hash is left out"
        );
    }

    #[test]
    fn the_page_is_nyaas_when_the_release_is_on_nyaa() {
        let items = parse(&fixture(), "tsukihime").unwrap();
        assert_eq!(items[0].page, "", "nyaa_id 0 has no page");
        assert_eq!(items[1].page, "https://nyaa.si/view/2154329");
    }

    #[test]
    fn an_error_reply_is_an_error() {
        let body =
            br#"{"total":0,"start":0,"limit":0,"error":true,"message":"bad query","results":[]}"#;
        let e = parse(body, "tsukihime").unwrap_err();
        assert!(e.contains("bad query"), "{e}");
        assert!(parse(br#"{"error":true}"#, "t").is_err());
        assert!(parse(b"<html>", "t").is_err());
    }
}
