//! AniList's JSON into the recommender's own types. Every field AniList can
//! send as null is read as optional, since it does send null for most of
//! them somewhere.

use std::collections::HashSet;

use anibeam_recs::{
    AiringStatus, Credit, Entry, Format, LookRole, RelationKind, Show, Status, Tag,
};
use serde_json::{Value, json};

/// One show as the recommender reads it. The title is the English one, else
/// the romaji, which is only ever used inside the models; what a page shows
/// is resolved in the user's own title language.
pub fn show(m: &Value) -> Option<Show> {
    let id = m["id"].as_u64()?;
    let title = m["title"]["english"]
        .as_str()
        .or(m["title"]["romaji"].as_str())
        .unwrap_or("?")
        .to_owned();
    let tags = m["tags"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| {
                    Some(Tag {
                        name: t["name"].as_str()?.to_owned(),
                        rank: t["rank"].as_u64().unwrap_or(0).min(100) as u8,
                        technical: t["category"].as_str() == Some("Technical"),
                        spoiler: is_spoiler(t),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let credits = m["staff"]["edges"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|e| {
                    Some(Credit {
                        staff: e["node"]["id"].as_u64()?,
                        name: e["node"]["name"]["full"].as_str().unwrap_or("?").to_owned(),
                        role: LookRole::classify(e["role"].as_str()?)?,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let relations = m["relations"]["edges"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|e| e["node"]["type"].as_str() == Some("ANIME"))
                .filter_map(|e| {
                    Some((
                        e["node"]["id"].as_u64()?,
                        RelationKind::from_anilist(e["relationType"].as_str()?),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    Some(Show {
        id,
        title,
        format: Format::from_anilist(m["format"].as_str().unwrap_or("")),
        episodes: m["episodes"].as_u64().map(|x| x as u32),
        minutes: m["duration"].as_u64().map(|x| x as u32),
        year: m["seasonYear"].as_i64().map(|x| x as i32),
        genres: strings(&m["genres"]),
        tags,
        studios: m["studios"]["nodes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|s| s["name"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
        credits,
        community: m["averageScore"].as_u64().map(|x| x.min(100) as u8),
        popularity: m["popularity"].as_u64().map(|x| x as u32),
        relations,
        adult: m["isAdult"].as_bool().unwrap_or(false),
        status: m["status"].as_str().and_then(AiringStatus::from_anilist),
    })
}

/// AniList keeps two flags: a spoiler for this show, and one for anything
/// that carries the tag. Either is enough to never show it.
pub fn is_spoiler(tag: &Value) -> bool {
    tag["isMediaSpoiler"].as_bool() == Some(true) || tag["isGeneralSpoiler"].as_bool() == Some(true)
}

pub fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// The entries of another user's MediaListCollection. Custom lists repeat
/// entries from the status lists, so they are skipped, and a show seen twice
/// is kept once. Scores arrive out of 100; nought is unrated.
pub fn collection(collection: &Value) -> Vec<Entry> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for list in collection["lists"].as_array().into_iter().flatten() {
        if list["isCustomList"].as_bool() == Some(true) {
            continue;
        }
        for e in list["entries"].as_array().into_iter().flatten() {
            let (Some(show), Some(status)) = (
                e["mediaId"].as_u64(),
                e["status"].as_str().and_then(Status::from_anilist),
            ) else {
                continue;
            };
            if !seen.insert(show) {
                continue;
            }
            out.push(Entry {
                show,
                status,
                score: e["score"].as_f64().filter(|s| *s > 0.0).map(|s| s as f32),
                repeat: e["repeat"].as_u64().unwrap_or(0) as u32,
            });
        }
    }
    out
}

fn status_code(s: Status) -> &'static str {
    match s {
        Status::Completed => "C",
        Status::Watching => "W",
        Status::Paused => "P",
        Status::Dropped => "D",
        Status::Planning => "L",
        Status::Repeating => "R",
    }
}

fn status_of(code: &str) -> Option<Status> {
    Some(match code {
        "C" => Status::Completed,
        "W" => Status::Watching,
        "P" => Status::Paused,
        "D" => Status::Dropped,
        "L" => Status::Planning,
        "R" => Status::Repeating,
        _ => return None,
    })
}

/// A list as it is stored: `[[show, status, score, repeat], ...]`. A hundred
/// lists of hundreds of entries each add up, so the row stays this short.
pub fn pack(entries: &[Entry]) -> String {
    let rows: Vec<Value> = entries
        .iter()
        .map(|e| json!([e.show, status_code(e.status), e.score, e.repeat]))
        .collect();
    Value::Array(rows).to_string()
}

pub fn unpack(text: &str) -> Vec<Entry> {
    let Ok(Value::Array(rows)) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    rows.iter()
        .filter_map(|r| {
            Some(Entry {
                show: r[0].as_u64()?,
                status: status_of(r[1].as_str()?)?,
                score: r[2].as_f64().map(|s| s as f32),
                repeat: r[3].as_u64().unwrap_or(0) as u32,
            })
        })
        .collect()
}

/// A list of ids as it is stored.
pub fn ids(text: &str) -> Vec<u64> {
    serde_json::from_str::<Vec<u64>>(text).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_show_reads_staff_roles_spoilers_and_anime_relations() {
        let m = json!({
            "id": 5, "format": "TV", "episodes": 12, "duration": 24, "seasonYear": 2011,
            "averageScore": 84, "popularity": 10, "isAdult": false, "genres": ["Drama"],
            "title": {"romaji": "R", "english": null},
            "tags": [
                {"name": "CGI", "rank": 70, "category": "Technical"},
                {"name": "Time Loop", "rank": 90, "category": "Theme-Other", "isMediaSpoiler": true}
            ],
            "studios": {"nodes": [{"name": "Studio A"}]},
            "staff": {"edges": [
                {"role": "Director", "node": {"id": 1, "name": {"full": "A"}}},
                {"role": "Music", "node": {"id": 2, "name": {"full": "B"}}},
                {"role": "Animation Director (eps 3, 7)", "node": {"id": 3, "name": {"full": "C"}}}
            ]},
            "relations": {"edges": [
                {"relationType": "PREQUEL", "node": {"id": 4, "type": "ANIME"}},
                {"relationType": "ADAPTATION", "node": {"id": 9, "type": "MANGA"}}
            ]}
        });
        let s = show(&m).unwrap();
        assert_eq!(s.title, "R");
        assert_eq!(s.credits.len(), 2, "music is not a look role");
        assert_eq!(s.credits[1].role, LookRole::AnimationDirector);
        assert!(s.tags[0].technical && !s.tags[0].spoiler);
        assert!(s.tags[1].spoiler);
        assert_eq!(s.relations, vec![(4, RelationKind::Prequel)]);
    }

    #[test]
    fn a_list_packs_and_unpacks_to_the_same_entries() {
        let c = json!({"lists": [
            {"isCustomList": false, "entries": [{"mediaId": 1, "status": "COMPLETED", "score": 0, "repeat": 2}]},
            {"isCustomList": true, "entries": [{"mediaId": 1, "status": "COMPLETED", "score": 90, "repeat": 0}]},
            {"isCustomList": false, "entries": [{"mediaId": 2, "status": "DROPPED", "score": 40, "repeat": 0}]}
        ]});
        let entries = collection(&c);
        assert_eq!(entries.len(), 2, "a custom list repeats an entry");
        assert_eq!(entries[0].score, None, "nought is unrated");
        assert_eq!(unpack(&pack(&entries)), entries);
    }
}
