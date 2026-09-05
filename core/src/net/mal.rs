//! MyAnimeList's own REST API, the tracker half. MAL is not a matching
//! provider: this carries the account, the list reads and the list writes,
//! and the token exchange that has no bearer token yet.

use super::limiter::ProviderClient;
use super::{Body, HttpRequest, HttpResponse, Method};
use crate::contract::CoreError;

pub struct MalClient {
    client: ProviderClient,
}

impl MalClient {
    pub fn new(client: ProviderClient) -> MalClient {
        MalClient { client }
    }

    /// A read, with the bearer token. The status comes back untouched: the
    /// tracker decides what a 401 means, since only it knows whether a
    /// refresh is worth trying.
    pub async fn get(&self, url: &str, token: &str) -> Result<HttpResponse, CoreError> {
        self.client
            .send(HttpRequest {
                method: Method::Get,
                url: url.to_string(),
                headers: bearer(token),
                body: None,
            })
            .await
    }

    /// A list write. MAL takes these as a form, never as JSON.
    pub async fn patch_form(
        &self,
        url: &str,
        token: &str,
        form: Vec<(String, String)>,
    ) -> Result<HttpResponse, CoreError> {
        self.client
            .send(HttpRequest {
                method: Method::Patch,
                url: url.to_string(),
                headers: bearer(token),
                body: Some(Body::Form(form)),
            })
            .await
    }

    /// The token exchange and the refresh, which carry the client id in the
    /// form and have no token to send.
    pub async fn post_form(&self, url: &str, form: Vec<(String, String)>) -> Result<HttpResponse, CoreError> {
        self.client
            .send(HttpRequest {
                method: Method::Post,
                url: url.to_string(),
                headers: vec![],
                body: Some(Body::Form(form)),
            })
            .await
    }
}

fn bearer(token: &str) -> Vec<(String, String)> {
    vec![("Authorization".to_string(), format!("Bearer {token}"))]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::limiter::ProviderClient;
    use crate::net::{Body, FakeHttp, Method, Upstream};
    use std::sync::Arc;
    use std::time::Duration;

    fn client(http: Arc<FakeHttp>) -> MalClient {
        MalClient::new(ProviderClient::new(Upstream::Mal, http, Duration::from_millis(1)))
    }

    #[tokio::test]
    async fn a_get_carries_the_bearer_token() {
        let http = FakeHttp::new();
        http.push_json(200, serde_json::json!({ "id": 1, "name": "bandit" }));
        let r = client(http.clone()).get("https://api.myanimelist.net/v2/users/@me", "t0ken").await.unwrap();
        assert_eq!(r.status, 200);
        let req = &http.requests()[0];
        assert_eq!(req.method, Method::Get);
        assert!(req.headers.contains(&("Authorization".to_string(), "Bearer t0ken".to_string())));
    }

    #[tokio::test]
    async fn a_patch_is_a_form_with_the_token_and_a_post_is_a_form_without_one() {
        let http = FakeHttp::new();
        http.push(200, "{}");
        http.push(200, "{}");
        let c = client(http.clone());
        let form = vec![("num_watched_episodes".to_string(), "3".to_string())];
        c.patch_form("https://api.myanimelist.net/v2/anime/1/my_list_status", "t0ken", form.clone()).await.unwrap();
        c.post_form("https://myanimelist.net/v1/oauth2/token", form.clone()).await.unwrap();
        let reqs = http.requests();
        assert_eq!(reqs[0].method, Method::Patch);
        assert!(reqs[0].headers.contains(&("Authorization".to_string(), "Bearer t0ken".to_string())));
        assert_eq!(reqs[0].body, Some(Body::Form(form.clone())));
        assert_eq!(reqs[1].method, Method::Post);
        assert!(!reqs[1].headers.iter().any(|(k, _)| k == "Authorization"));
        assert_eq!(reqs[1].body, Some(Body::Form(form)));
    }
}
