//! AniSkip, the crowd-sourced intro and outro times, keyed by MAL id. The
//! file's own chapters win when it has them; this is the fallback.

use serde::Deserialize;

use super::limiter::ProviderClient;
use super::{HttpRequest, Method};
use crate::contract::{CoreError, Provider, SkipKind, SkipSource, SkipWindow};

pub const ANISKIP_API: &str = "https://api.aniskip.com/v2";

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Interval {
    start_time: f64,
    end_time: f64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SkipResult {
    interval: Interval,
    skip_type: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SkipReply {
    results: Vec<SkipResult>,
}

pub struct AniSkipClient {
    client: ProviderClient,
}

impl AniSkipClient {
    pub fn new(client: ProviderClient) -> AniSkipClient {
        AniSkipClient { client }
    }

    /// The windows AniSkip holds for one episode. `Ok(None)` is a miss, so
    /// the caller can cache "nothing here" and retry it later; an `Err` is
    /// a failure to ask, and the caller caches nothing.
    pub async fn skip_times(
        &self,
        mal_id: u64,
        episode: u32,
        duration_secs: u64,
    ) -> Result<Option<Vec<SkipWindow>>, CoreError> {
        let url = format!(
            "{ANISKIP_API}/skip-times/{mal_id}/{episode}?types[]=op&types[]=ed&episodeLength={duration_secs}"
        );
        let response = self
            .client
            .send(HttpRequest {
                method: Method::Get,
                url,
                headers: vec![("Accept".to_string(), "application/json".to_string())],
                body: None,
            })
            .await?;
        if response.status == 404 {
            return Ok(None);
        }
        if !response.is_success() {
            return Err(CoreError::Provider {
                provider: Provider::Mal,
                status: Some(u32::from(response.status)),
                message: format!("AniSkip {}: {}", response.status, response.text()),
                retry_after: None,
            });
        }
        let reply: SkipReply = response.json()?;
        let mut windows = Vec::new();
        for result in reply.results {
            // A mixed opening or ending is still the opening or the ending;
            // a recap is neither, and is dropped.
            let kind = match result.skip_type.as_str() {
                "op" | "mixed-op" => SkipKind::Intro,
                "ed" | "mixed-ed" => SkipKind::Outro,
                _ => continue,
            };
            windows.push(SkipWindow {
                kind,
                start: result.interval.start_time,
                end: result.interval.end_time,
                source: SkipSource::AniSkip,
            });
        }
        Ok(Some(windows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::limiter::ProviderClient;
    use crate::net::{FakeHttp, Upstream};
    use std::sync::Arc;
    use std::time::Duration;

    fn client(http: Arc<FakeHttp>) -> AniSkipClient {
        AniSkipClient::new(ProviderClient::new(
            Upstream::AniSkip,
            http,
            Duration::from_millis(1),
        ))
    }

    #[tokio::test]
    async fn op_and_ed_become_windows_and_recap_is_dropped() {
        let http = FakeHttp::new();
        http.push_json(
            200,
            serde_json::json!({ "found": true, "results": [
                { "interval": { "startTime": 85.0, "endTime": 175.0 }, "skipType": "op" },
                { "interval": { "startTime": 1320.0, "endTime": 1410.0 }, "skipType": "mixed-ed" },
                { "interval": { "startTime": 0.0, "endTime": 60.0 }, "skipType": "recap" }
            ] }),
        );
        let windows = client(http.clone())
            .skip_times(52991, 3, 1440)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            windows,
            vec![
                SkipWindow {
                    kind: SkipKind::Intro,
                    start: 85.0,
                    end: 175.0,
                    source: SkipSource::AniSkip
                },
                SkipWindow {
                    kind: SkipKind::Outro,
                    start: 1320.0,
                    end: 1410.0,
                    source: SkipSource::AniSkip
                },
            ]
        );
        assert_eq!(
            http.requests()[0].url,
            "https://api.aniskip.com/v2/skip-times/52991/3?types[]=op&types[]=ed&episodeLength=1440"
        );
    }

    #[tokio::test]
    async fn a_404_is_a_miss_and_a_500_is_an_error() {
        let http = FakeHttp::new();
        http.push(404, r#"{"found":false,"message":"Not Found"}"#);
        assert_eq!(client(http).skip_times(1, 1, 1440).await.unwrap(), None);

        let http = FakeHttp::new();
        http.fail_next("connection refused");
        assert!(client(http).skip_times(1, 1, 1440).await.is_err());
    }
}
