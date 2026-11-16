//! Jikan, and only for one thing: the per-episode side-fetch by MAL id.
//! It is not a matching provider and never was on the native line; AniList
//! carries no episode titles, so this fills them in.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;

use super::limiter::ProviderClient;
use super::{HttpRequest, null_to_default};
use anibeam_contract::{CoreError, Provider};

pub const JIKAN_API: &str = "https://api.jikan.moe/v4";

/// How long one Jikan request may take. Jikan answers in a second or two
/// when it is up, and a match waits on it, so a request that has not
/// answered by now is an outage rather than a slow reply.
pub const JIKAN_TIMEOUT: Duration = Duration::from_secs(10);

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
    #[serde(default, deserialize_with = "null_to_default")]
    data: Vec<RawEpisode>,
    #[serde(default, deserialize_with = "null_to_default")]
    pagination: Pagination,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct Pagination {
    has_next_page: bool,
}

/// The most pages one read asks for. Jikan sends 100 episodes a page, so
/// this reaches episode 2,000, past the longest show it lists; a reply that
/// says there is a next page forever stops here instead of looping.
pub const JIKAN_MAX_PAGES: u32 = 20;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JikanEpisode {
    pub number: u32,
    pub title: Option<String>,
    pub aired: Option<String>,
    pub synopsis: Option<String>,
}

/// What one read of a series' episode list came back with.
#[derive(Debug, Default)]
pub struct JikanEpisodes {
    /// Every episode read, keyed by number and in episode order.
    pub episodes: Vec<JikanEpisode>,
    /// Whether this is all Jikan holds. It is not when a page after the
    /// first failed, or the read stopped at `JIKAN_MAX_PAGES`, so a caller
    /// may add these episodes but must not delete a row they leave out.
    pub whole: bool,
    /// The failure that stopped the read on a page after the first.
    pub stopped: Option<CoreError>,
}

impl JikanEpisodes {
    /// A list that is everything Jikan holds.
    pub fn whole(episodes: Vec<JikanEpisode>) -> JikanEpisodes {
        JikanEpisodes {
            episodes,
            whole: true,
            stopped: None,
        }
    }
}

pub struct JikanClient {
    client: ProviderClient,
}

impl JikanClient {
    pub fn new(client: ProviderClient) -> JikanClient {
        JikanClient { client }
    }

    /// Every episode Jikan holds for that MAL id, in episode order, read a
    /// page at a time while Jikan says another follows (#1040). Each page
    /// is its own request through the limiter. A 404 is a series it has no
    /// episode list for, which is an empty list rather than a failure.
    ///
    /// A failure on the first page is the read's failure. One on a later
    /// page keeps the pages before it, as a list that is not whole, with
    /// the failure in `stopped`.
    pub async fn episodes(&self, mal_id: u64) -> Result<JikanEpisodes, CoreError> {
        // Keyed by episode number, so a repeated entry collapses and the
        // list comes back in order whatever order Jikan sent it in.
        let mut by_number: BTreeMap<u32, JikanEpisode> = BTreeMap::new();
        let mut whole = false;
        let mut stopped = None;
        for page in 1..=JIKAN_MAX_PAGES {
            let reply = match self.page(mal_id, page).await {
                Ok(Some(reply)) => reply,
                // A 404 on the first page is a series with no episode
                // list. On a later one it is a reply that contradicts the
                // page before it, so what was read is kept but not whole.
                Ok(None) => {
                    whole = page == 1;
                    break;
                }
                Err(e) if page == 1 => return Err(e),
                Err(e) => {
                    stopped = Some(e);
                    break;
                }
            };
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
            if !reply.pagination.has_next_page {
                whole = true;
                break;
            }
        }
        Ok(JikanEpisodes {
            episodes: by_number.into_values().collect(),
            whole,
            stopped,
        })
    }

    /// One page of the episode list, or `None` for a 404. The first page
    /// is asked for without a page number, the way Jikan's own links do.
    async fn page(&self, mal_id: u64, page: u32) -> Result<Option<EpisodesReply>, CoreError> {
        let mut url = format!("{JIKAN_API}/anime/{mal_id}/episodes");
        if page > 1 {
            url.push_str(&format!("?page={page}"));
        }
        let response = self
            .client
            .send(HttpRequest::get(url).header("Accept", "application/json"))
            .await?;
        if response.status == 404 {
            return Ok(None);
        }
        if !response.is_success() {
            return Err(CoreError::provider(
                Provider::Mal,
                Some(u32::from(response.status)),
                format!("Jikan {}: {}", response.status, response.text()),
            ));
        }
        Ok(Some(response.json()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limiter::ProviderClient;
    use crate::{FakeHttp, Upstream};
    use std::sync::Arc;

    fn client(http: Arc<FakeHttp>) -> JikanClient {
        JikanClient::new(ProviderClient::new(
            Upstream::Jikan,
            http,
            Duration::from_millis(1),
        ))
    }

    /// Jikan answering with a null `data` is a series it holds no episodes
    /// for, which is an empty list rather than a reply that failed to
    /// parse.
    #[tokio::test]
    async fn a_null_data_list_is_an_empty_answer_not_a_failure() {
        let http = FakeHttp::new();
        http.push_json(200, serde_json::json!({ "data": null }));
        let read = client(http).episodes(1).await.unwrap();
        assert!(read.episodes.is_empty());
        assert!(read.whole);
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
        let eps = client(http.clone()).episodes(52991).await.unwrap().episodes;
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

    /// One page of Jikan's episode list: episodes `first..=last`, saying
    /// whether another page follows.
    fn page(first: u32, last: u32, has_next: bool) -> serde_json::Value {
        let data: Vec<serde_json::Value> = (first..=last)
            .map(|n| serde_json::json!({ "mal_id": n, "episode": n, "title": format!("Episode {n}"), "aired": null, "synopsis": null }))
            .collect();
        serde_json::json!({
            "pagination": { "last_visible_page": 2, "has_next_page": has_next },
            "data": data,
        })
    }

    /// A show past 100 episodes comes back over two pages, and the fetch
    /// asks for the second one.
    #[tokio::test]
    async fn a_long_show_is_read_across_every_page() {
        let http = FakeHttp::new();
        http.push_json(200, page(1, 100, true));
        http.push_json(200, page(101, 150, false));
        let read = client(http.clone()).episodes(21).await.unwrap();
        assert!(read.whole);
        assert!(read.stopped.is_none());
        assert_eq!(read.episodes.len(), 150);
        assert_eq!(read.episodes[149].number, 150);
        let urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
        assert_eq!(
            urls,
            vec![
                "https://api.jikan.moe/v4/anime/21/episodes".to_string(),
                "https://api.jikan.moe/v4/anime/21/episodes?page=2".to_string(),
            ]
        );
    }

    /// A failure on page two keeps page one, as a list that is not whole,
    /// and hands the failure back beside it.
    #[tokio::test]
    async fn a_failure_on_a_later_page_keeps_the_pages_before_it() {
        let http = FakeHttp::new();
        http.push_json(200, page(1, 100, true));
        http.push(504, "gateway timeout");
        let read = client(http.clone()).episodes(21).await.unwrap();
        assert!(!read.whole);
        assert_eq!(read.episodes.len(), 100);
        assert!(
            matches!(
                read.stopped,
                Some(CoreError::Provider {
                    status: Some(504),
                    ..
                })
            ),
            "{:?}",
            read.stopped
        );
        assert_eq!(http.requests().len(), 2);
    }

    /// A 404 past the first page contradicts the page before it, so what
    /// was read is kept but not whole.
    #[tokio::test]
    async fn a_404_on_a_later_page_is_not_a_whole_list() {
        let http = FakeHttp::new();
        http.push_json(200, page(1, 100, true));
        http.push(404, "not found");
        let read = client(http).episodes(21).await.unwrap();
        assert!(!read.whole);
        assert!(read.stopped.is_none());
        assert_eq!(read.episodes.len(), 100);
    }

    /// A reply that always says another page follows stops at the cap,
    /// and what it read is not whole.
    #[tokio::test]
    async fn a_next_page_that_never_ends_stops_at_the_cap() {
        let http = FakeHttp::new();
        for n in 0..JIKAN_MAX_PAGES + 5 {
            http.push_json(200, page(n * 100 + 1, n * 100 + 100, true));
        }
        let read = client(http.clone()).episodes(21).await.unwrap();
        assert!(!read.whole);
        assert_eq!(http.requests().len(), JIKAN_MAX_PAGES as usize);
        assert_eq!(read.episodes.len(), JIKAN_MAX_PAGES as usize * 100);
    }

    #[tokio::test]
    async fn a_404_is_an_empty_list_and_a_500_is_a_provider_error() {
        let http = FakeHttp::new();
        http.push(404, "not found");
        assert!(client(http).episodes(1).await.unwrap().episodes.is_empty());

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
