//! Jikan, and only for one thing: the per-episode side-fetch by MAL id.
//! It is not a matching provider and never was on the native line; AniList
//! carries no episode titles, so this fills them in.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::limiter::ProviderClient;
use super::{HttpRequest, Method};
use crate::contract::{CoreError, Provider};

pub const JIKAN_API: &str = "https://api.jikan.moe/v4";

/// One episode as Jikan reports it. `mal_id` is the database row, never the
/// episode number, so `episode` is the only key worth having.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct RawEpisode {
    episode: i64,
    title: Option<String>,
    aired: Option<String>,
    synopsis: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct EpisodesReply {
    data: Vec<RawEpisode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JikanEpisode {
    pub number: u32,
    pub title: Option<String>,
    pub aired: Option<String>,
    pub synopsis: Option<String>,
}

pub struct JikanClient {
    client: ProviderClient,
}

impl JikanClient {
    pub fn new(client: ProviderClient) -> JikanClient {
        JikanClient { client }
    }

    /// Every episode Jikan holds for that MAL id, in episode order. A 404
    /// is a series it has no episode list for, which is an empty list
    /// rather than a failure.
    pub async fn episodes(&self, mal_id: u64) -> Result<Vec<JikanEpisode>, CoreError> {
        let response = self
            .client
            .send(HttpRequest {
                method: Method::Get,
                url: format!("{JIKAN_API}/anime/{mal_id}/episodes"),
                headers: vec![("Accept".to_string(), "application/json".to_string())],
                body: None,
            })
            .await?;
        if response.status == 404 {
            return Ok(Vec::new());
        }
        if !response.is_success() {
            return Err(CoreError::Provider {
                provider: Provider::Mal,
                status: Some(u32::from(response.status)),
                message: format!("Jikan {}: {}", response.status, response.text()),
                retry_after: None,
            });
        }
        let reply: EpisodesReply = response.json()?;
        // Keyed by episode number, so a repeated entry collapses and the
        // list comes back in order whatever order Jikan sent it in.
        let mut by_number: BTreeMap<u32, JikanEpisode> = BTreeMap::new();
        for raw in reply.data {
            if raw.episode <= 0 {
                continue;
            }
            let number = raw.episode as u32;
            by_number.insert(
                number,
                JikanEpisode {
                    number,
                    title: raw.title,
                    aired: raw.aired,
                    synopsis: raw.synopsis,
                },
            );
        }
        Ok(by_number.into_values().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::limiter::ProviderClient;
    use crate::net::{FakeHttp, Upstream};
    use std::sync::Arc;
    use std::time::Duration;

    fn client(http: Arc<FakeHttp>) -> JikanClient {
        JikanClient::new(ProviderClient::new(
            Upstream::Jikan,
            http,
            Duration::from_millis(1),
        ))
    }

    #[tokio::test]
    async fn episodes_are_keyed_by_number_and_non_positive_ones_are_dropped() {
        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": [
                { "mal_id": 1, "episode": 2, "title": "It Didn't Have to Be Magic", "aired": null, "synopsis": "a synopsis" },
                { "mal_id": 2, "episode": 0, "title": "not an episode", "aired": null, "synopsis": null },
                { "mal_id": 3, "episode": 1, "title": "The Journey's End", "aired": "2023-09-29T00:00:00+00:00", "synopsis": null }
            ] }),
        );
        let eps = client(http.clone()).episodes(52991).await.unwrap();
        assert_eq!(eps.len(), 2);
        assert_eq!(eps[0].number, 1);
        assert_eq!(eps[0].title.as_deref(), Some("The Journey's End"));
        assert_eq!(eps[0].aired.as_deref(), Some("2023-09-29T00:00:00+00:00"));
        assert_eq!(eps[1].number, 2);
        assert_eq!(eps[1].synopsis.as_deref(), Some("a synopsis"));
        assert_eq!(
            http.requests()[0].url,
            "https://api.jikan.moe/v4/anime/52991/episodes"
        );
    }

    #[tokio::test]
    async fn a_404_is_an_empty_list_and_a_500_is_a_provider_error() {
        let http = FakeHttp::new();
        http.push(404, "not found");
        assert!(client(http).episodes(1).await.unwrap().is_empty());

        let http = FakeHttp::new();
        http.push(500, "boom");
        let err = client(http).episodes(1).await.err().unwrap();
        assert!(
            matches!(
                err,
                CoreError::Provider {
                    provider: Provider::Mal,
                    status: Some(500),
                    ..
                }
            ),
            "{err:?}"
        );
    }
}
