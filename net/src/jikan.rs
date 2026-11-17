//! Jikan, and only for one thing: the per-episode side-fetch by MAL id.
//! It is not a matching provider and never was on the native line; AniList
//! carries no episode titles, so this fills them in.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

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

/// The most pages one read asks for from where it starts. Jikan sends 100
/// episodes a page, so a read from the first page reaches episode 2,000,
/// past the longest show it lists; a reply that says there is a next page
/// forever stops here instead of looping.
pub const JIKAN_MAX_PAGES: u32 = 20;

/// How long one read may spend before it stops asking for more pages. A
/// match waits on the read, so a long show on a slow Jikan stops here with
/// what it has rather than holding the match for minutes. A quick Jikan
/// gets through all of `JIKAN_MAX_PAGES` well inside it at the limiter's
/// pace.
pub const JIKAN_READ_BUDGET: Duration = Duration::from_secs(30);

/// How many episodes Jikan sends a page. A read that starts past the
/// first page finds its page from this.
pub const JIKAN_PAGE_SIZE: u32 = 100;

/// Why a read stopped before Jikan said there was nothing more.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JikanCap {
    /// It asked for `JIKAN_MAX_PAGES` pages and Jikan still said another
    /// followed.
    Pages,
    /// It ran past its budget before the next page was asked for.
    Budget,
}

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
    /// first failed, or the read stopped at `JIKAN_MAX_PAGES` or at
    /// `JIKAN_READ_BUDGET`, so a caller
    /// may add these episodes but must not delete a row they leave out.
    pub whole: bool,
    /// The failure that stopped the read on a page after the first.
    pub stopped: Option<CoreError>,
    /// Set when the read stopped at `JIKAN_MAX_PAGES` or at its budget,
    /// which is what the one log line about it says (#1105).
    pub capped: Option<JikanCap>,
}

impl JikanEpisodes {
    /// A list that is everything Jikan holds.
    pub fn whole(episodes: Vec<JikanEpisode>) -> JikanEpisodes {
        JikanEpisodes {
            episodes,
            whole: true,
            stopped: None,
            capped: None,
        }
    }
}

pub struct JikanClient {
    client: ProviderClient,
    /// The read budget in milliseconds: `JIKAN_READ_BUDGET`, which a test
    /// shortens.
    budget_ms: AtomicU64,
}

impl JikanClient {
    pub fn new(client: ProviderClient) -> JikanClient {
        JikanClient::with_budget(client, JIKAN_READ_BUDGET)
    }

    /// A client whose reads stop asking for pages after `budget`, for a
    /// test that cannot wait out the real one.
    pub fn with_budget(client: ProviderClient, budget: Duration) -> JikanClient {
        JikanClient {
            client,
            budget_ms: AtomicU64::new(budget.as_millis() as u64),
        }
    }

    /// Changes the read budget, for a test that holds the client already.
    pub fn set_budget(&self, budget: Duration) {
        self.budget_ms
            .store(budget.as_millis() as u64, Ordering::SeqCst);
    }

    fn budget(&self) -> Duration {
        Duration::from_millis(self.budget_ms.load(Ordering::SeqCst))
    }

    /// Every episode Jikan holds for that MAL id, in episode order, read a
    /// page at a time while Jikan says another follows (#1040). Each page
    /// is its own request through the limiter. A 404 is a series it has no
    /// episode list for, which is an empty list rather than a failure.
    ///
    /// A failure on the first page is the read's failure. One on a later
    /// page keeps the pages before it, as a list that is not whole, with
    /// the failure in `stopped`. So does running past the read's budget
    /// before a later page is asked for, with nothing in `stopped`.
    pub async fn episodes(&self, mal_id: u64) -> Result<JikanEpisodes, CoreError> {
        self.episodes_from(mal_id, 1).await
    }

    /// The same read, from the page that holds episode `from` on (#1105).
    /// A caller that already holds everything before `from` asks for this
    /// rather than the whole list: a long show is a request per hundred
    /// episodes. `whole` then means Jikan holds nothing past what came
    /// back, and the episodes before the start page are simply not in it.
    ///
    /// The page is found by counting a hundred episodes a page from one.
    /// When Jikan's own list does not count that way, the start page
    /// begins past `from`, is missing, or is refused; the read then tries
    /// the page before it, and after that the first page, so it never
    /// skips the episode asked for. A list that ends on a page boundary
    /// costs one page more, not all of them. Each try has its own budget.
    pub async fn episodes_from(&self, mal_id: u64, from: u32) -> Result<JikanEpisodes, CoreError> {
        let start = from.saturating_sub(1) / JIKAN_PAGE_SIZE + 1;
        let mut tries = vec![start];
        if start > 2 {
            tries.push(start - 1);
        }
        if start > 1 {
            tries.push(1);
        }
        for first in tries {
            if let Read::Pages(read) = self.read(mal_id, first, from).await? {
                return Ok(read);
            }
        }
        // Even the first page is a 404: a series with no episode list.
        Ok(JikanEpisodes::whole(Vec::new()))
    }

    /// Pages from `first` on, while Jikan says another follows. A 404 on
    /// `first` is `Read::Missing`. So is a start page past the first that
    /// Jikan refuses or that begins past episode `from`, which ends the
    /// read after that one request.
    async fn read(&self, mal_id: u64, first: u32, from: u32) -> Result<Read, CoreError> {
        // Keyed by episode number, so a repeated entry collapses and the
        // list comes back in order whatever order Jikan sent it in.
        let mut by_number: BTreeMap<u32, JikanEpisode> = BTreeMap::new();
        let mut whole = false;
        let mut stopped = None;
        let mut capped = Some(JikanCap::Pages);
        let started = Instant::now();
        let budget = self.budget();
        for page in first..first + JIKAN_MAX_PAGES {
            if page > first && started.elapsed() >= budget {
                capped = Some(JikanCap::Budget);
                break;
            }
            let reply = match self.page(mal_id, page).await {
                Ok(Some(reply)) => reply,
                // A 404 on the first page is a series with no episode
                // list. On a later one it is a reply that contradicts the
                // page before it, so what was read is kept but not whole.
                Ok(None) if page == first => return Ok(Read::Missing),
                Ok(None) => {
                    capped = None;
                    break;
                }
                // A start page Jikan refuses outright may be one past its
                // last; an earlier page may still answer.
                Err(e) if page == first && first > 1 && refused(&e) => return Ok(Read::Missing),
                Err(e) if page == first => return Err(e),
                Err(e) => {
                    stopped = Some(e);
                    capped = None;
                    break;
                }
            };
            let episodes: Vec<RawEpisode> = reply
                .data
                .into_iter()
                .filter(|raw| raw.episode > 0)
                .collect();
            if page == first
                && first > 1
                && episodes
                    .iter()
                    .map(|raw| raw.episode)
                    .min()
                    .is_some_and(|n| n > i64::from(from))
            {
                return Ok(Read::Missing);
            }
            for raw in episodes {
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
                capped = None;
                break;
            }
        }
        Ok(Read::Pages(JikanEpisodes {
            episodes: by_number.into_values().collect(),
            whole,
            stopped,
            capped,
        }))
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

/// What a run of pages came back with.
enum Read {
    /// The pages read.
    Pages(JikanEpisodes),
    /// The first page asked for was a 404, or a start page that does not
    /// hold the episode asked for.
    Missing,
}

/// A client error that is not a rate limit: Jikan refusing the request
/// itself, which asking for a different page may not repeat.
fn refused(e: &CoreError) -> bool {
    matches!(e, CoreError::Provider { status: Some(s), .. } if (400..500).contains(s) && *s != 429)
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
        assert_eq!(read.capped, None);
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
        assert_eq!(read.capped, None);
        assert_eq!(read.episodes.len(), 100);
    }

    /// A read from a late episode asks only from the page that holds it
    /// (#1105): a show past a thousand episodes is one request, not
    /// eleven, and what it read is whole from there on.
    #[tokio::test]
    async fn a_read_from_a_late_episode_asks_only_from_the_page_holding_it() {
        let http = FakeHttp::new();
        http.push_json(200, page(1001, 1060, false));
        let read = client(http.clone()).episodes_from(21, 1051).await.unwrap();
        assert!(read.whole);
        assert_eq!(read.capped, None);
        assert_eq!(read.episodes.len(), 60);
        assert_eq!(read.episodes[0].number, 1001);
        let urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
        assert_eq!(
            urls,
            vec!["https://api.jikan.moe/v4/anime/21/episodes?page=11".to_string()]
        );
    }

    /// Episode 100 is still on the first page, and 101 on the second.
    #[tokio::test]
    async fn the_start_page_counts_a_hundred_episodes_a_page() {
        let http = FakeHttp::new();
        http.push_json(200, page(1, 100, false));
        client(http.clone()).episodes_from(21, 100).await.unwrap();
        http.push_json(200, page(101, 120, false));
        client(http.clone()).episodes_from(21, 101).await.unwrap();
        let urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
        assert_eq!(
            urls,
            vec![
                "https://api.jikan.moe/v4/anime/21/episodes".to_string(),
                "https://api.jikan.moe/v4/anime/21/episodes?page=2".to_string(),
            ]
        );
    }

    /// A Jikan list that skips numbers puts an episode on an earlier page
    /// than counting says. A start page that begins past the episode
    /// asked for starts the read over from the first page, so the
    /// episode is not skipped.
    #[tokio::test]
    async fn a_start_page_past_the_episode_asked_for_starts_over() {
        let http = FakeHttp::new();
        http.push_json(200, page(160, 200, false));
        http.push_json(200, page(1, 100, true));
        http.push_json(200, page(101, 200, false));
        let read = client(http.clone()).episodes_from(21, 150).await.unwrap();
        assert!(read.whole);
        assert_eq!(read.episodes.len(), 200);
        let urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
        assert_eq!(
            urls,
            vec![
                "https://api.jikan.moe/v4/anime/21/episodes?page=2".to_string(),
                "https://api.jikan.moe/v4/anime/21/episodes".to_string(),
                "https://api.jikan.moe/v4/anime/21/episodes?page=2".to_string(),
            ]
        );
    }

    /// A 404 on the start page is a page that does not exist, so the read
    /// starts over from the first.
    #[tokio::test]
    async fn a_missing_start_page_starts_over() {
        let http = FakeHttp::new();
        http.push(404, "not found");
        http.push_json(200, page(1, 40, false));
        let read = client(http.clone()).episodes_from(21, 150).await.unwrap();
        assert!(read.whole);
        assert_eq!(read.episodes.len(), 40);
        assert_eq!(http.requests().len(), 2);
    }

    /// A list that ends on a page boundary has no page for the next
    /// episode yet. The read tries the page before it, which is whole,
    /// rather than every page from the first.
    #[tokio::test]
    async fn a_list_ending_on_a_page_boundary_costs_one_page_more() {
        let http = FakeHttp::new();
        http.push(404, "not found");
        http.push_json(200, page(901, 1000, false));
        let read = client(http.clone()).episodes_from(21, 1001).await.unwrap();
        assert!(read.whole);
        assert_eq!(read.episodes.len(), 100);
        let urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
        assert_eq!(
            urls,
            vec![
                "https://api.jikan.moe/v4/anime/21/episodes?page=11".to_string(),
                "https://api.jikan.moe/v4/anime/21/episodes?page=10".to_string(),
            ]
        );
    }

    /// A start page Jikan refuses is tried again from an earlier page; an
    /// outage on it is the read's failure, with no second request.
    #[tokio::test]
    async fn a_refused_start_page_falls_back_and_an_outage_does_not() {
        let http = FakeHttp::new();
        http.push(400, "bad request");
        http.push_json(200, page(101, 150, false));
        let read = client(http.clone()).episodes_from(21, 120).await;
        assert!(read.unwrap().whole);
        assert_eq!(http.requests().len(), 2);

        let http = FakeHttp::new();
        http.push(504, "gateway timeout");
        assert!(client(http.clone()).episodes_from(21, 1051).await.is_err());
        assert_eq!(http.requests().len(), 1);
    }

    /// A read past its budget asks for no further page, and what it read
    /// is not whole.
    #[tokio::test]
    async fn a_read_past_its_budget_stops_with_what_it_has() {
        let http = FakeHttp::new();
        http.push_json(200, page(1, 100, true));
        http.push_json(200, page(101, 150, false));
        let jikan = JikanClient::with_budget(
            ProviderClient::new(Upstream::Jikan, http.clone(), Duration::from_millis(1)),
            Duration::ZERO,
        );
        let read = jikan.episodes(21).await.unwrap();
        assert!(!read.whole);
        assert!(read.stopped.is_none());
        assert_eq!(read.capped, Some(JikanCap::Budget));
        assert_eq!(read.episodes.len(), 100);
        assert_eq!(http.requests().len(), 1);
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
        assert_eq!(read.capped, Some(JikanCap::Pages));
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
