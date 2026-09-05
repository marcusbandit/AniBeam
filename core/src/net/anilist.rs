//! AniList, the one matching provider. Every query is Electron's, copied
//! character for character from `src/main/handlers/anilistHandler.ts` and
//! `src/main/handlers/trackerHandler.ts`, so the native line asks AniList
//! exactly what the Electron line asked it.

use serde::Deserialize;

use super::limiter::ProviderClient;
use super::{Body, HttpRequest, Method, null_to_default};
use crate::contract::{CoreError, Provider};

pub const ANILIST_API: &str = "https://graphql.anilist.co";

/// Electron's `SEARCH_MULTIPLE_QUERY`, anilistHandler.ts:315. Its
/// single-`Media` search query is dead code there and is not carried.
pub const SEARCH_QUERY: &str = r"
  query ($search: String, $page: Int, $perPage: Int) {
    Page(page: $page, perPage: $perPage) {
      media(search: $search, type: ANIME) {
        id
        idMal
        title {
          romaji
          english
          native
        }
        synonyms
        description
        genres
        coverImage {
          large
          extraLarge
        }
        bannerImage
        episodes
        duration
        season
        seasonYear
        status
        format
        startDate {
          year
          month
          day
        }
        endDate {
          year
          month
          day
        }
        averageScore
        studios {
          nodes {
            name
          }
        }
      }
    }
  }
";

/// Electron's `MEDIA_BY_ID_QUERY`, anilistHandler.ts:543.
pub const MEDIA_BY_ID_QUERY: &str = r"
  query ($id: Int) {
    Media(id: $id, type: ANIME) {
      id
      idMal
      title {
        romaji
        english
        native
      }
      description
      genres
      coverImage {
        large
        extraLarge
      }
      bannerImage
      episodes
      duration
      season
      seasonYear
      status
      format
      startDate {
        year
        month
        day
      }
      endDate {
        year
        month
        day
      }
      averageScore
      studios {
        nodes {
          name
        }
      }
    }
  }
";

/// Electron's `RESOLVE_ID_BY_MAL_QUERY`, anilistHandler.ts:364. The cheap
/// one-shot that maps a MAL id to its AniList id.
pub const RESOLVE_ID_BY_MAL_QUERY: &str = r"
  query ($idMal: Int) {
    Media(idMal: $idMal, type: ANIME) {
      id
    }
  }
";

/// Electron's `AIRING_SCHEDULE_QUERY`, anilistHandler.ts:386.
/// `nextAiringEpisode` rides along because `airingSchedule` is paginated,
/// so a long-runner's upcoming episode is simply absent from its nodes.
pub const AIRING_SCHEDULE_QUERY: &str = r"
  query ($id: Int, $idMal: Int) {
    Media(id: $id, idMal: $idMal, type: ANIME) {
      id
      nextAiringEpisode {
        episode
        airingAt
      }
      airingSchedule {
        nodes {
          episode
          airingAt
        }
      }
    }
  }
";

/// Electron's `ENRICHMENT_QUERY`, anilistHandler.ts:418. One request for
/// the whole series page: tags, studios, characters, recommendations and
/// relations.
pub const ENRICHMENT_QUERY: &str = r"
  query ($id: Int, $idMal: Int) {
    Media(id: $id, idMal: $idMal) {
      id
      idMal
      type
      format
      status
      seasonYear
      startDate {
        year
      }
      siteUrl
      title {
        romaji
        english
      }
      coverImage {
        large
      }
      streamingEpisodes {
        title
        thumbnail
        url
        site
      }
      tags {
        name
        rank
        isMediaSpoiler
        isGeneralSpoiler
        isAdult
        category
      }
      studios {
        edges {
          isMain
          node {
            id
            name
            isAnimationStudio
          }
        }
      }
      characters(perPage: 12, sort: [ROLE, RELEVANCE, ID]) {
        edges {
          role
          node {
            id
            name {
              full
            }
            image {
              large
              medium
            }
            siteUrl
          }
        }
      }
      recommendations(perPage: 12, sort: RATING_DESC) {
        edges {
          node {
            rating
            mediaRecommendation {
              id
              idMal
              type
              format
              status
              seasonYear
              siteUrl
              title {
                romaji
                english
              }
              coverImage {
                large
              }
            }
          }
        }
      }
      relations {
        edges {
          relationType
          node {
            id
            idMal
            type
            format
            status
            seasonYear
            startDate {
              year
            }
            siteUrl
            title {
              romaji
              english
            }
            coverImage {
              large
            }
          }
        }
      }
    }
  }
";

/// Electron's viewer query, trackerHandler.ts:307-313. Who the token
/// belongs to, asked once at connect.
pub const VIEWER_QUERY: &str = r"query { Viewer { id name } }";

/// Electron's current-entry query, trackerHandler.ts:430. `userId` is not
/// optional in practice: `MediaList(mediaId)` without it ignores the
/// bearer token and answers with some other user's entry.
pub const MEDIA_LIST_ENTRY_QUERY: &str = r"query ($userId: Int, $mediaId: Int) { MediaList(userId: $userId, mediaId: $mediaId) { progress status } }";

/// Electron's progress mutation, trackerHandler.ts:445-449.
pub const SAVE_PROGRESS_MUTATION: &str = r"mutation ($mediaId: Int, $progress: Int, $status: MediaListStatus) {
      SaveMediaListEntry(mediaId: $mediaId, progress: $progress, status: $status) {
        id progress status
      }
    }";

/// Electron's score mutation, trackerHandler.ts:693-697. `scoreRaw` is the
/// 0 to 100 form AniList converts to the user's own scale, so one number
/// works whatever their list format is.
pub const SAVE_SCORE_MUTATION: &str = r"mutation ($mediaId: Int, $scoreRaw: Int, $status: MediaListStatus) {
      SaveMediaListEntry(mediaId: $mediaId, scoreRaw: $scoreRaw, status: $status) {
        id score status
      }
    }";

/// Electron's progress sweep, trackerHandler.ts:775-779.
pub const MEDIA_LIST_COLLECTION_QUERY: &str = r"query ($userId: Int) {
        MediaListCollection(userId: $userId, type: ANIME) {
          lists { entries { progress status score(format: POINT_10_DECIMAL) repeat media { id } } }
        }
      }";

/// Electron's watching list, trackerHandler.ts:936-957. The same
/// collection, with everything a card needs on the media.
pub const WATCHING_LIST_QUERY: &str = r"query ($userId: Int) {
          MediaListCollection(userId: $userId, type: ANIME) {
            lists {
              entries {
                progress
                status
                score(format: POINT_10_DECIMAL)
                updatedAt
                media {
                  id
                  idMal
                  siteUrl
                  episodes
                  averageScore
                  title { romaji english }
                  coverImage { large }
                  nextAiringEpisode { episode airingAt }
                }
              }
            }
          }
        }";

/// AniList's schema marks a good many things nullable that read as
/// required: a tag's spoiler flags, a relation's type, a studio edge's
/// `isMain`, and every list and edge object on a series page. A bare field
/// would fail the whole reply on one of those nulls, so nothing here that
/// is not an `Option` is read without `null_to_default`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CoverImage {
    pub large: Option<String>,
    pub extra_large: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CoverLarge {
    pub large: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FuzzyDate {
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Named {
    #[serde(default, deserialize_with = "null_to_default")]
    pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StudioNodes {
    #[serde(default, deserialize_with = "null_to_default")]
    pub nodes: Vec<Named>,
}

/// One search hit or one `Media` by id: everything a match writes to the
/// series row.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Media {
    pub id: u64,
    pub id_mal: Option<u64>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub title: Title,
    #[serde(default, deserialize_with = "null_to_default")]
    pub synonyms: Vec<String>,
    pub description: Option<String>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub genres: Vec<String>,
    pub cover_image: Option<CoverImage>,
    pub banner_image: Option<String>,
    pub episodes: Option<u32>,
    pub duration: Option<u32>,
    pub season: Option<String>,
    pub season_year: Option<u32>,
    pub status: Option<String>,
    pub format: Option<String>,
    pub start_date: Option<FuzzyDate>,
    pub end_date: Option<FuzzyDate>,
    /// AniList's own 0 to 100 scale. Never normalised here.
    pub average_score: Option<u32>,
    pub studios: Option<StudioNodes>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StreamingEpisode {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub url: Option<String>,
    pub site: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TagNode {
    #[serde(default, deserialize_with = "null_to_default")]
    pub name: String,
    pub rank: Option<u32>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub is_media_spoiler: bool,
    #[serde(default, deserialize_with = "null_to_default")]
    pub is_general_spoiler: bool,
    #[serde(default, deserialize_with = "null_to_default")]
    pub is_adult: bool,
    pub category: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StudioNode {
    pub id: u64,
    #[serde(default, deserialize_with = "null_to_default")]
    pub name: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub is_animation_studio: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StudioEdge {
    #[serde(default, deserialize_with = "null_to_default")]
    pub is_main: bool,
    #[serde(default, deserialize_with = "null_to_default")]
    pub node: StudioNode,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StudioEdges {
    #[serde(default, deserialize_with = "null_to_default")]
    pub edges: Vec<StudioEdge>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CharacterName {
    pub full: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Image {
    pub large: Option<String>,
    pub medium: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CharacterNode {
    pub id: u64,
    pub name: Option<CharacterName>,
    pub image: Option<Image>,
    pub site_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CharacterEdge {
    pub role: Option<String>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub node: CharacterNode,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CharacterEdges {
    #[serde(default, deserialize_with = "null_to_default")]
    pub edges: Vec<CharacterEdge>,
}

/// A neighbour on the graph: a relation's other end, or a recommendation.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RelatedNode {
    pub id: u64,
    pub id_mal: Option<u64>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub season_year: Option<u32>,
    pub start_date: Option<FuzzyDate>,
    pub site_url: Option<String>,
    pub title: Option<Title>,
    pub cover_image: Option<CoverLarge>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RecommendationNode {
    pub rating: Option<i64>,
    pub media_recommendation: Option<RelatedNode>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RecommendationEdge {
    #[serde(default, deserialize_with = "null_to_default")]
    pub node: RecommendationNode,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RecommendationEdges {
    #[serde(default, deserialize_with = "null_to_default")]
    pub edges: Vec<RecommendationEdge>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RelationEdge {
    #[serde(default, deserialize_with = "null_to_default")]
    pub relation_type: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub node: RelatedNode,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RelationEdges {
    #[serde(default, deserialize_with = "null_to_default")]
    pub edges: Vec<RelationEdge>,
}

/// The series page in one reply.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Enrichment {
    pub id: u64,
    pub id_mal: Option<u64>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub season_year: Option<u32>,
    pub start_date: Option<FuzzyDate>,
    pub site_url: Option<String>,
    pub title: Option<Title>,
    pub cover_image: Option<CoverLarge>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub streaming_episodes: Vec<StreamingEpisode>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub tags: Vec<TagNode>,
    pub studios: Option<StudioEdges>,
    pub characters: Option<CharacterEdges>,
    pub recommendations: Option<RecommendationEdges>,
    pub relations: Option<RelationEdges>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AiringNode {
    pub episode: u32,
    pub airing_at: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AiringNodes {
    #[serde(default, deserialize_with = "null_to_default")]
    pub nodes: Vec<AiringNode>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Schedule {
    pub next_airing_episode: Option<AiringNode>,
    pub airing_schedule: Option<AiringNodes>,
}

pub struct AnilistClient {
    client: ProviderClient,
}

impl AnilistClient {
    pub fn new(client: ProviderClient) -> AnilistClient {
        AnilistClient { client }
    }

    /// Every AniList request goes through here: one POST, the errors rule,
    /// and `data` back. The tracker calls pass their bearer token.
    pub async fn graphql(
        &self,
        query: &'static str,
        variables: serde_json::Value,
        token: Option<&str>,
    ) -> Result<serde_json::Value, CoreError> {
        self.post_query(query, variables, token, None).await
    }

    /// The same, with a cap on each request. The tracker calls take one: a
    /// user is waiting on those, and the client's own 30 second ceiling is
    /// a long time to watch a spinner. The cap is per request, so the 429
    /// schedule underneath keeps its own bounds.
    pub async fn graphql_within(
        &self,
        query: &'static str,
        variables: serde_json::Value,
        token: Option<&str>,
        timeout: std::time::Duration,
    ) -> Result<serde_json::Value, CoreError> {
        self.post_query(query, variables, token, Some(timeout))
            .await
    }

    async fn post_query(
        &self,
        query: &'static str,
        variables: serde_json::Value,
        token: Option<&str>,
        timeout: Option<std::time::Duration>,
    ) -> Result<serde_json::Value, CoreError> {
        let mut headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Accept".to_string(), "application/json".to_string()),
        ];
        if let Some(token) = token {
            headers.push(("Authorization".to_string(), format!("Bearer {token}")));
        }
        let request = HttpRequest {
            method: Method::Post,
            url: ANILIST_API.to_string(),
            headers,
            body: Some(Body::Json(
                serde_json::json!({ "query": query, "variables": variables }),
            )),
        };
        let response = match timeout {
            Some(d) => self.client.send_within(request, d).await?,
            None => self.client.send(request).await?,
        };
        // AniList answers a GraphQL failure with a 200 and an `errors`
        // array as often as it answers with the status, so the body is
        // read first and the status is only the fallback.
        let value = match response.json::<serde_json::Value>() {
            Ok(value) => value,
            Err(e) => {
                if response.is_success() {
                    return Err(e);
                }
                return Err(provider_error(
                    Some(u32::from(response.status)),
                    response.text(),
                ));
            }
        };
        if let Some(first) = value["errors"].as_array().and_then(|errors| errors.first()) {
            let status = first["status"].as_u64().map(|s| s as u32);
            let message = first["message"]
                .as_str()
                .unwrap_or("AniList error")
                .to_string();
            return Err(provider_error(status, message));
        }
        if !response.is_success() {
            return Err(provider_error(
                Some(u32::from(response.status)),
                response.text(),
            ));
        }
        match value.get("data") {
            Some(data) if !data.is_null() => Ok(data.clone()),
            _ => Err(provider_error(None, "no data")),
        }
    }

    pub async fn search(&self, query: &str, per_page: u32) -> Result<Vec<Media>, CoreError> {
        let data = self
            .graphql(
                SEARCH_QUERY,
                serde_json::json!({ "search": query, "page": 1, "perPage": per_page }),
                None,
            )
            .await?;
        let media = &data["Page"]["media"];
        if media.is_null() {
            return Ok(Vec::new());
        }
        Ok(serde_json::from_value(media.clone())?)
    }

    pub async fn media_by_id(&self, id: u64) -> Result<Option<Media>, CoreError> {
        Ok(self.media_by_id_raw(id).await?.0)
    }

    /// The typed reply and the JSON it was parsed from, together. A match
    /// keeps the second half verbatim in `anilist_media.raw`, so a later
    /// migration can mine a field this schema has no column for without
    /// asking AniList again.
    pub async fn media_by_id_raw(
        &self,
        id: u64,
    ) -> Result<(Option<Media>, serde_json::Value), CoreError> {
        let data = self
            .graphql(MEDIA_BY_ID_QUERY, serde_json::json!({ "id": id }), None)
            .await?;
        let raw = data["Media"].clone();
        Ok((parse_media(&raw)?, raw))
    }

    /// The MAL id to its AniList id. AniList answers a MAL id it has never
    /// heard of with a 404 error, which is a miss rather than a failure.
    pub async fn resolve_by_mal(&self, mal_id: u64) -> Result<Option<u64>, CoreError> {
        match self
            .graphql(
                RESOLVE_ID_BY_MAL_QUERY,
                serde_json::json!({ "idMal": mal_id }),
                None,
            )
            .await
        {
            Ok(data) => Ok(data["Media"]["id"].as_u64()),
            Err(e) if is_not_found(&e) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// The paginated schedule page and the always-present next broadcast.
    /// A series AniList carries no schedule for is an empty schedule.
    pub async fn schedule(&self, id: u64) -> Result<Schedule, CoreError> {
        Ok(self.schedule_raw(id).await?.0)
    }

    /// The schedule and the JSON behind it, for the same reason
    /// `media_by_id_raw` exists.
    pub async fn schedule_raw(&self, id: u64) -> Result<(Schedule, serde_json::Value), CoreError> {
        let data = self
            .graphql(AIRING_SCHEDULE_QUERY, serde_json::json!({ "id": id }), None)
            .await?;
        let raw = data["Media"].clone();
        if raw.is_null() {
            return Ok((Schedule::default(), raw));
        }
        Ok((serde_json::from_value(raw.clone())?, raw))
    }

    pub async fn enrichment(&self, id: u64) -> Result<Option<Enrichment>, CoreError> {
        Ok(self.enrichment_raw(id).await?.0)
    }

    /// The series page and the JSON behind it, for the same reason
    /// `media_by_id_raw` exists.
    pub async fn enrichment_raw(
        &self,
        id: u64,
    ) -> Result<(Option<Enrichment>, serde_json::Value), CoreError> {
        let data = self
            .graphql(ENRICHMENT_QUERY, serde_json::json!({ "id": id }), None)
            .await?;
        let raw = data["Media"].clone();
        if raw.is_null() {
            return Ok((None, raw));
        }
        Ok((Some(serde_json::from_value(raw.clone())?), raw))
    }
}

fn provider_error(status: Option<u32>, message: impl Into<String>) -> CoreError {
    CoreError::Provider {
        provider: Provider::Anilist,
        status,
        message: message.into(),
        retry_after: None,
    }
}

fn is_not_found(e: &CoreError) -> bool {
    matches!(
        e,
        CoreError::Provider {
            status: Some(404),
            ..
        }
    )
}

fn parse_media(value: &serde_json::Value) -> Result<Option<Media>, CoreError> {
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_value(value.clone())?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::limiter::ProviderClient;
    use crate::net::{Body, FakeHttp, Upstream};
    use std::sync::Arc;
    use std::time::Duration;

    fn client(http: Arc<FakeHttp>) -> AnilistClient {
        AnilistClient::new(ProviderClient::new(
            Upstream::Anilist,
            http,
            Duration::from_millis(1),
        ))
    }

    #[tokio::test]
    async fn search_posts_the_query_and_parses_media() {
        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": { "Page": { "media": [ {
                "id": 154587,
                "idMal": 52991,
                "title": { "romaji": "Sousou no Frieren", "english": "Frieren: Beyond Journey's End", "native": null },
                "synonyms": ["Frieren"],
                "episodes": 28,
                "status": "FINISHED",
                "format": "TV",
                "seasonYear": 2023,
                "startDate": { "year": 2023, "month": 9, "day": 29 },
                "averageScore": 91,
                "coverImage": { "large": "https://img/l.jpg", "extraLarge": "https://img/xl.jpg" },
                "studios": { "nodes": [ { "name": "Madhouse" } ] }
            } ] } } }),
        );
        let media = client(http.clone()).search("Frieren", 10).await.unwrap();
        assert_eq!(media.len(), 1);
        assert_eq!(media[0].id, 154587);
        assert_eq!(media[0].id_mal, Some(52991));
        assert_eq!(media[0].synonyms, vec!["Frieren"]);
        assert_eq!(media[0].studios.as_ref().unwrap().nodes[0].name, "Madhouse");
        let req = &http.requests()[0];
        assert_eq!(req.url, "https://graphql.anilist.co");
        let body = match &req.body {
            Some(Body::Json(v)) => v.clone(),
            other => panic!("expected a json body, got {other:?}"),
        };
        assert_eq!(
            body["variables"],
            serde_json::json!({ "search": "Frieren", "page": 1, "perPage": 10 })
        );
        assert!(
            body["query"]
                .as_str()
                .unwrap()
                .contains("Page(page: $page, perPage: $perPage)")
        );
    }

    /// AniList sends null where its own schema promises a list or an
    /// object, and a struct-level `#[serde(default)]` covers a missing
    /// field, never a null one. Every list and every nested object on
    /// these replies reads a null as its default, so one null costs that
    /// value rather than the whole reply.
    #[tokio::test]
    async fn an_explicit_null_reads_as_the_default_rather_than_failing() {
        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": { "Page": { "media": [ {
                "id": 1,
                "title": null,
                "synonyms": null,
                "genres": null,
                "studios": { "nodes": null }
            } ] } } }),
        );
        let media = client(http).search("x", 10).await.unwrap();
        assert_eq!(media[0].id, 1);
        assert_eq!(media[0].title.romaji, None);
        assert!(media[0].synonyms.is_empty());
        assert!(media[0].genres.is_empty());
        assert!(media[0].studios.as_ref().unwrap().nodes.is_empty());

        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": { "Media": {
                "id": 2,
                "streamingEpisodes": null,
                "tags": null,
                "studios": { "edges": null },
                "characters": { "edges": null },
                "recommendations": { "edges": null },
                "relations": { "edges": [ { "relationType": null, "node": null } ] }
            } } }),
        );
        let enrichment = client(http).enrichment(2).await.unwrap().unwrap();
        assert!(enrichment.streaming_episodes.is_empty());
        assert!(enrichment.tags.is_empty());
        assert!(enrichment.studios.unwrap().edges.is_empty());
        assert!(enrichment.characters.unwrap().edges.is_empty());
        assert!(enrichment.recommendations.unwrap().edges.is_empty());
        let edges = enrichment.relations.unwrap().edges;
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].relation_type, "");
        assert_eq!(edges[0].node.id, 0);

        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": { "Media": { "id": 3, "nextAiringEpisode": null, "airingSchedule": { "nodes": null } } } }),
        );
        let schedule = client(http).schedule(3).await.unwrap();
        assert!(schedule.airing_schedule.unwrap().nodes.is_empty());
    }

    #[tokio::test]
    async fn graphql_errors_become_provider_errors() {
        let http = FakeHttp::new();
        http.push_json(200, serde_json::json!({ "data": null, "errors": [ { "message": "Not Found.", "status": 404 } ] }));
        let err = client(http).media_by_id(1).await.err().unwrap();
        assert!(
            matches!(
                err,
                CoreError::Provider {
                    status: Some(404),
                    ..
                }
            ),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn a_reply_without_data_is_a_provider_error() {
        let http = FakeHttp::new();
        http.push_json(200, serde_json::json!({ "nothing": true }));
        let err = client(http).media_by_id(1).await.err().unwrap();
        assert!(
            matches!(&err, CoreError::Provider { status: None, message, .. } if message == "no data"),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn resolve_by_mal_returns_the_id_or_none() {
        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": { "Media": { "id": 21 } } }),
        );
        http.push_json(200, serde_json::json!({ "data": null, "errors": [ { "message": "Not Found.", "status": 404 } ] }));
        let c = client(http);
        assert_eq!(c.resolve_by_mal(21).await.unwrap(), Some(21));
        assert_eq!(c.resolve_by_mal(99).await.unwrap(), None);
    }

    #[tokio::test]
    async fn a_null_media_is_none_and_a_bearer_token_is_sent() {
        let http = FakeHttp::new();
        http.push_json(200, serde_json::json!({ "data": { "Media": null } }));
        let c = client(http.clone());
        assert!(c.media_by_id(7).await.unwrap().is_none());
        c.graphql(VIEWER_QUERY, serde_json::json!({}), Some("t0ken"))
            .await
            .ok();
        let auth = http.requests()[1]
            .headers
            .iter()
            .find(|(k, _)| k == "Authorization")
            .cloned();
        assert_eq!(
            auth,
            Some(("Authorization".to_string(), "Bearer t0ken".to_string()))
        );
    }

    #[tokio::test]
    async fn the_schedule_and_the_enrichment_parse() {
        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": { "Media": {
                "id": 21,
                "nextAiringEpisode": { "episode": 1124, "airingAt": 1_725_000_000i64 },
                "airingSchedule": { "nodes": [ { "episode": 1123, "airingAt": 1_724_400_000i64 } ] }
            } } }),
        );
        let s = client(http.clone()).schedule(21).await.unwrap();
        assert_eq!(s.next_airing_episode.unwrap().episode, 1124);
        assert_eq!(s.airing_schedule.unwrap().nodes.len(), 1);

        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": { "Media": {
                "id": 154587,
                "idMal": 52991,
                "type": "ANIME",
                "siteUrl": "https://anilist.co/anime/154587",
                "tags": [ { "name": "Adventure", "rank": 90, "isMediaSpoiler": false, "isGeneralSpoiler": false, "isAdult": false, "category": "Theme" } ],
                "studios": { "edges": [ { "isMain": true, "node": { "id": 11, "name": "Madhouse", "isAnimationStudio": true } } ] },
                "characters": { "edges": [ { "role": "MAIN", "node": { "id": 1, "name": { "full": "Frieren" }, "image": { "large": "l", "medium": "m" }, "siteUrl": "u" } } ] },
                "recommendations": { "edges": [ { "node": { "rating": 42, "mediaRecommendation": { "id": 1, "title": { "romaji": "x" } } } } ] },
                "relations": { "edges": [ { "relationType": "SEQUEL", "node": { "id": 2, "title": { "romaji": "y" } } } ] },
                "streamingEpisodes": [ { "title": "Episode 1", "thumbnail": "t", "url": "u", "site": "Crunchyroll" } ]
            } } }),
        );
        let e = client(http).enrichment(154587).await.unwrap().unwrap();
        assert_eq!(e.type_.as_deref(), Some("ANIME"));
        assert_eq!(e.tags[0].rank, Some(90));
        assert!(e.studios.unwrap().edges[0].is_main);
        assert_eq!(
            e.characters.unwrap().edges[0]
                .node
                .name
                .as_ref()
                .unwrap()
                .full
                .as_deref(),
            Some("Frieren")
        );
        assert_eq!(e.recommendations.unwrap().edges[0].node.rating, Some(42));
        assert_eq!(e.relations.unwrap().edges[0].relation_type, "SEQUEL");
        assert_eq!(e.streaming_episodes[0].site.as_deref(), Some("Crunchyroll"));
    }

    /// AniList marks most of these scalars nullable, so one explicit null
    /// must cost that one field its value and nothing else.
    #[tokio::test]
    async fn explicit_nulls_in_an_enrichment_fall_back_to_defaults() {
        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": { "Media": {
                "id": 154587,
                "tags": [ { "name": "Adventure", "rank": null, "isMediaSpoiler": null, "isGeneralSpoiler": null, "isAdult": null, "category": null } ],
                "studios": { "edges": [ { "isMain": null, "node": { "id": 11, "name": null, "isAnimationStudio": null } } ] },
                "characters": { "edges": [ { "role": null, "node": { "id": 1, "name": { "full": null }, "image": null, "siteUrl": null } } ] },
                "recommendations": { "edges": [ { "node": { "rating": null, "mediaRecommendation": null } } ] },
                "relations": { "edges": [ { "relationType": null, "node": { "id": 2 } } ] }
            } } }),
        );
        let e = client(http).enrichment(154587).await.unwrap().unwrap();
        assert_eq!(e.id, 154587);
        assert_eq!(e.tags[0].name, "Adventure");
        assert_eq!(e.tags[0].rank, None);
        assert!(!e.tags[0].is_media_spoiler);
        assert!(!e.tags[0].is_general_spoiler);
        assert!(!e.tags[0].is_adult);
        let studios = e.studios.unwrap();
        assert!(!studios.edges[0].is_main);
        assert_eq!(studios.edges[0].node.name, "");
        assert!(!studios.edges[0].node.is_animation_studio);
        let characters = e.characters.unwrap();
        assert_eq!(characters.edges[0].role, None);
        assert_eq!(characters.edges[0].node.name.as_ref().unwrap().full, None);
        assert_eq!(e.recommendations.unwrap().edges[0].node.rating, None);
        assert_eq!(e.relations.unwrap().edges[0].relation_type, "");
    }

    /// The same for a `Media`: a studio node with a null name is one empty
    /// name, not a failed search.
    #[tokio::test]
    async fn a_null_studio_name_does_not_fail_a_search() {
        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "data": { "Page": { "media": [ {
                "id": 1,
                "title": { "romaji": "x" },
                "studios": { "nodes": [ { "name": null } ] }
            } ] } } }),
        );
        let media = client(http).search("x", 1).await.unwrap();
        assert_eq!(media[0].studios.as_ref().unwrap().nodes[0].name, "");
    }
}
