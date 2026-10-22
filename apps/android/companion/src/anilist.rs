//! Airing times straight from AniList, for while the host is away.
//!
//! One GraphQL request per 50 shows (AniList's page size), no login. The caller keeps
//! it to once an hour; this module only asks and reads the answer.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::api::Airing;

pub const ENDPOINT: &str = "https://graphql.anilist.co";
/// AniList pages hold at most 50.
const PAGE: usize = 50;
/// How many scheduled episodes to ask for per show. Enough for weeks of countdown.
const AHEAD: usize = 10;

const QUERY: &str = "query ($ids: [Int], $perPage: Int, $ahead: Int) { \
    Page(perPage: $perPage) { media(id_in: $ids, type: ANIME) { id status \
    nextAiringEpisode { episode airingAt } \
    airingSchedule(notYetAired: true, perPage: $ahead) { nodes { episode airingAt } } } } }";

/// What AniList said about one show.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    /// AniList's word: `RELEASING`, `FINISHED`, `NOT_YET_RELEASED`, ...
    pub status: Option<String>,
    /// The episodes still to air, soonest first.
    pub upcoming: Vec<Airing>,
}

#[derive(Deserialize)]
struct Reply {
    data: Option<Data>,
    #[serde(default)]
    errors: Vec<ReplyError>,
}
#[derive(Deserialize)]
struct ReplyError {
    message: String,
}
#[derive(Deserialize)]
struct Data {
    #[serde(rename = "Page")]
    page: Page,
}
#[derive(Deserialize)]
struct Page {
    media: Vec<Media>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Media {
    id: u64,
    status: Option<String>,
    next_airing_episode: Option<Node>,
    airing_schedule: Option<Nodes>,
}
#[derive(Deserialize)]
struct Nodes {
    nodes: Vec<Node>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Node {
    episode: u32,
    airing_at: i64,
}

fn read(body: &str) -> Result<BTreeMap<u64, Schedule>, String> {
    let reply: Reply = serde_json::from_str(body).map_err(|e| format!("AniList's answer did not read: {e}"))?;
    let Some(data) = reply.data else {
        let why = reply.errors.first().map(|e| e.message.as_str()).unwrap_or("no data");
        return Err(format!("AniList said: {why}"));
    };
    let mut out = BTreeMap::new();
    for media in data.page.media {
        let mut upcoming: Vec<Airing> = media
            .airing_schedule
            .map(|s| s.nodes)
            .unwrap_or_default()
            .into_iter()
            .chain(media.next_airing_episode)
            .map(|n| Airing { episode: n.episode, season: None, at: n.airing_at })
            .collect();
        upcoming.sort_by_key(|a| (a.at, a.episode));
        upcoming.dedup_by_key(|a| a.episode);
        out.insert(media.id, Schedule { status: media.status, upcoming });
    }
    Ok(out)
}

/// Asks AniList for these shows' schedules. Ids AniList does not know are absent.
pub fn fetch(agent: &ureq::Agent, ids: &[u64]) -> Result<BTreeMap<u64, Schedule>, String> {
    let mut out = BTreeMap::new();
    for chunk in ids.chunks(PAGE) {
        let body = serde_json::json!({
            "query": QUERY,
            "variables": { "ids": chunk, "perPage": PAGE, "ahead": AHEAD },
        });
        let mut response = agent
            .post(ENDPOINT)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .send(body.to_string())
            .map_err(|e| crate::client::describe(&e, "graphql.anilist.co"))?;
        let status = response.status().as_u16();
        let text = response
            .body_mut()
            .read_to_string()
            .map_err(|e| format!("AniList's answer broke off: {e}"))?;
        if status == 429 {
            return Err("AniList asked to slow down; trying again in an hour".into());
        }
        if status != 200 && !text.trim_start().starts_with('{') {
            return Err(format!("AniList answered {status}"));
        }
        out.extend(read(&text)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_page_and_folds_in_the_next_episode() {
        let body = r#"{"data":{"Page":{"media":[
            {"id":182255,"status":"RELEASING","nextAiringEpisode":{"episode":8,"airingAt":200},
             "airingSchedule":{"nodes":[{"episode":9,"airingAt":300},{"episode":8,"airingAt":200}]}},
            {"id":457,"status":"FINISHED","nextAiringEpisode":null,"airingSchedule":{"nodes":[]}}
        ]}}}"#;
        let got = read(body).unwrap();
        let frieren = &got[&182255];
        assert_eq!(frieren.status.as_deref(), Some("RELEASING"));
        assert_eq!(
            frieren.upcoming.iter().map(|a| (a.episode, a.at)).collect::<Vec<_>>(),
            vec![(8, 200), (9, 300)]
        );
        assert!(got[&457].upcoming.is_empty());
    }

    #[test]
    fn an_error_reply_says_why() {
        let body = r#"{"data":null,"errors":[{"message":"Too Many Requests.","status":429}]}"#;
        assert_eq!(read(body).unwrap_err(), "AniList said: Too Many Requests.");
    }
}
