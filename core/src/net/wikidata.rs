//! Wikidata, for the one thing neither anime provider carries: the IMDb
//! title behind an AniList or a MAL id. One SPARQL query answers for every
//! item that holds an IMDb title beside either, about ten thousand rows,
//! and the data is CC0.
//!
//! Asked once a week at most and straight through `Http`, like the IMDb
//! dump: one query has nothing to pace, and an `Upstream` would report
//! Wikidata's failures as one of the contract's providers, which it is not.

use std::time::Duration;

use serde::Deserialize;

use super::imdb::title_id;
use super::{Fault, Http, HttpRequest, Method, null_to_default};
use crate::contract::Provider;
use crate::percent;

pub const SPARQL_API: &str = "https://query.wikidata.org/sparql";

/// P345 is the IMDb id, P8729 the AniList id and P4086 the MAL id. Only an
/// IMDb id starting "tt" is a title; the rest name people and companies.
pub const QUERY: &str = "SELECT ?anilist ?mal ?imdb WHERE { ?item wdt:P345 ?imdb . FILTER(STRSTARTS(?imdb, \"tt\")) { ?item wdt:P8729 ?anilist } UNION { ?item wdt:P4086 ?mal } }";

/// The query service gives up on its own at 60 seconds. A little past
/// that, so a slow query ends in the service's answer rather than ours.
pub const QUERY_TIMEOUT: Duration = Duration::from_secs(70);

/// One link: an AniList or MAL id, and an IMDb title Wikidata puts on the
/// same item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImdbLink {
    pub provider: Provider,
    pub provider_id: u64,
    pub imdb: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct Term {
    value: String,
}

/// One row of the answer. The query's UNION means a row carries the
/// AniList id or the MAL id, never both, but nothing here relies on it.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct Binding {
    anilist: Option<Term>,
    mal: Option<Term>,
    imdb: Option<Term>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct Results {
    #[serde(deserialize_with = "null_to_default")]
    bindings: Vec<Binding>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct Reply {
    #[serde(deserialize_with = "null_to_default")]
    results: Results,
}

/// Every AniList and MAL id Wikidata links to an IMDb title.
pub async fn imdb_links(http: &dyn Http) -> Result<Vec<ImdbLink>, Fault> {
    let response = http
        .send(HttpRequest {
            method: Method::Get,
            url: format!("{SPARQL_API}?query={}", percent::encode(QUERY)),
            headers: vec![
                (
                    "Accept".to_string(),
                    "application/sparql-results+json".to_string(),
                ),
                // Wikimedia asks every client to say what it is and where
                // it lives, and throttles the ones that do not.
                (
                    "User-Agent".to_string(),
                    format!(
                        "AniBeam/{} (https://github.com/marcusbandit/AniBeam)",
                        crate::VERSION
                    ),
                ),
            ],
            body: None,
            timeout: Some(QUERY_TIMEOUT),
        })
        .await?;
    if !response.is_success() {
        return Err(Fault::Refused(format!(
            "Wikidata {}: {}",
            response.status,
            response.text()
        )));
    }
    parse(&response.body)
}

/// The answer's bindings as links. A row whose IMDb id is not a title, or
/// whose AniList or MAL id is not a plain number, is skipped rather than
/// failing the rest.
pub fn parse(body: &[u8]) -> Result<Vec<ImdbLink>, Fault> {
    let reply: Reply = serde_json::from_slice(body)
        .map_err(|e| Fault::Refused(format!("Wikidata's answer did not read: {e}")))?;
    let mut links = Vec::new();
    for row in reply.results.bindings {
        let Some(imdb) = row.imdb.and_then(|t| title_id(t.value.as_bytes())) else {
            continue;
        };
        for (provider, term) in [(Provider::Anilist, row.anilist), (Provider::Mal, row.mal)] {
            if let Some(provider_id) = term.and_then(|t| plain_id(&t.value)) {
                links.push(ImdbLink {
                    provider,
                    provider_id,
                    imdb,
                });
            }
        }
    }
    Ok(links)
}

/// Digits and nothing else, above nought. `str::parse` alone would take a
/// leading `+`, which no id is written with.
fn plain_id(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse().ok().filter(|id| *id > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;
    use serde_json::json;

    fn term(value: &str) -> serde_json::Value {
        json!({ "type": "literal", "value": value })
    }

    fn link(provider: Provider, provider_id: u64, imdb: u64) -> ImdbLink {
        ImdbLink {
            provider,
            provider_id,
            imdb,
        }
    }

    /// The shape the query service answers with, captured from a live run.
    #[test]
    fn bindings_become_links_by_provider() {
        let body = json!({
            "head": { "vars": ["anilist", "mal", "imdb"] },
            "results": { "bindings": [
                { "anilist": term("1"), "imdb": term("tt0213338") },
                { "mal": term("1"), "imdb": term("tt0213338") },
                { "anilist": term("212"), "imdb": term("tt0091794") },
            ] }
        });
        assert_eq!(
            parse(body.to_string().as_bytes()).unwrap(),
            vec![
                link(Provider::Anilist, 1, 213338),
                link(Provider::Mal, 1, 213338),
                link(Provider::Anilist, 212, 91794),
            ]
        );
    }

    #[test]
    fn rows_that_are_not_a_title_and_a_plain_id_are_skipped() {
        let body = json!({ "results": { "bindings": [
            { "anilist": term("5"), "imdb": term("nm0000001") },
            { "anilist": term("Q42"), "imdb": term("tt0000001") },
            { "mal": term("+7"), "imdb": term("tt0000001") },
            { "mal": term(""), "imdb": term("tt0000001") },
            { "anilist": term("0"), "imdb": term("tt0000001") },
            { "anilist": term("6") },
            { "imdb": term("tt0000001") },
            { "anilist": term("8"), "imdb": term("tt12x") },
            { "anilist": term("9"), "mal": term("10"), "imdb": term("tt0000009") },
        ] } });
        assert_eq!(
            parse(body.to_string().as_bytes()).unwrap(),
            vec![link(Provider::Anilist, 9, 9), link(Provider::Mal, 10, 9)]
        );
    }

    #[test]
    fn a_null_answer_is_empty_and_garbage_is_refused() {
        assert!(
            parse(br#"{"results":{"bindings":null}}"#)
                .unwrap()
                .is_empty()
        );
        assert!(parse(br#"{"results":null}"#).unwrap().is_empty());
        assert!(matches!(parse(b"<html>"), Err(Fault::Refused(_))));
    }

    #[tokio::test]
    async fn the_query_names_anibeam_and_asks_for_json() {
        let http = FakeHttp::new();
        http.push(
            200,
            json!({ "results": { "bindings": [
                { "anilist": term("1"), "imdb": term("tt0213338") }
            ] } })
            .to_string(),
        );
        http.push(429, "Too Many Requests");
        let links = imdb_links(http.as_ref()).await.unwrap();
        assert_eq!(links, vec![link(Provider::Anilist, 1, 213338)]);
        assert!(matches!(
            imdb_links(http.as_ref()).await,
            Err(Fault::Refused(_))
        ));

        let request = &http.requests()[0];
        assert!(
            request
                .url
                .starts_with("https://query.wikidata.org/sparql?query=SELECT%20%3Fanilist")
        );
        assert!(!request.url.contains(' '));
        assert_eq!(request.timeout, Some(QUERY_TIMEOUT));
        let header = |name: &str| {
            request
                .headers
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
        };
        assert_eq!(
            header("Accept").as_deref(),
            Some("application/sparql-results+json")
        );
        assert_eq!(
            header("User-Agent"),
            Some(format!(
                "AniBeam/{} (https://github.com/marcusbandit/AniBeam)",
                crate::VERSION
            ))
        );
    }
}
