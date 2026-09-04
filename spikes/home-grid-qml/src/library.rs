//! Card records for the grid, read from the Electron app's config directory. Read only.

use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn config_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home().join(".config"));
    base.join("anibeam")
}

pub fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

fn read_json(path: &Path) -> Value {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(Value::Null)
}

/// `YYYY-MM-DDTHH:MM:SS(.sss)Z` to unix milliseconds; None for anything else.
fn parse_iso_ms(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 19 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let num = |from: usize, to: usize| s.get(from..to)?.parse::<i64>().ok();
    let (y, m, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (hh, mm, ss) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    let millis = if b.len() > 20 && b[19] == b'.' {
        let frac: String = s[20..].chars().take_while(|c| c.is_ascii_digit()).collect();
        let mut v = frac.parse::<i64>().ok()?;
        for _ in frac.len()..3 {
            v *= 10;
        }
        for _ in 3..frac.len() {
            v /= 10;
        }
        v
    } else {
        0
    };
    // Days from civil, Howard Hinnant's algorithm.
    let (y2, m2) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    let era = if y2 >= 0 { y2 } else { y2 - 399 } / 400;
    let yoe = y2 - era * 400;
    let doy = (153 * m2 + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(((days * 86400 + hh * 3600 + mm * 60 + ss) * 1000) + millis)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn str_or_null(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(s.clone()),
        _ => Value::Null,
    }
}

fn num_or_null(v: &Value) -> Value {
    match v {
        Value::Number(n) => Value::Number(n.clone()),
        _ => Value::Null,
    }
}

/// Whole numbers print as integers, so QML sees `15` rather than `15.0`.
fn tidy(n: Option<f64>) -> Value {
    match n {
        Some(v) if v.fract() == 0.0 => json!(v as i64),
        Some(v) => json!(v),
        None => Value::Null,
    }
}

fn provider_key(v: &Value) -> Option<String> {
    match v {
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

pub fn load() -> Value {
    let dir = config_dir();
    let metadata = read_json(&dir.join("metadata.json"));
    let trackers = read_json(&dir.join("trackers.json"));
    let history = read_json(&dir.join("view-history.json"));

    let empty = Map::new();
    let anilist_progress = trackers["progress"]["anilist"].as_object().unwrap_or(&empty);
    let mal_progress = trackers["progress"]["mal"].as_object().unwrap_or(&empty);
    let history = history["history"].as_object().unwrap_or(&empty);
    let now = now_ms();

    let Some(entries) = metadata.as_object() else {
        return Value::Array(vec![]);
    };

    let records = entries
        .iter()
        .map(|(key, s)| {
            let id = s["seriesId"].as_str().unwrap_or(key).to_string();
            let folder_name = s["folderPath"]
                .as_str()
                .and_then(|p| Path::new(p).file_name())
                .map(|n| Value::String(n.to_string_lossy().into_owned()))
                .unwrap_or(Value::Null);
            let poster = s["posterLocal"]
                .as_str()
                .filter(|p| Path::new(p).is_file())
                .map(|p| Value::String(p.to_string()))
                .unwrap_or(Value::Null);
            let is_movie = s["type"].as_str() == Some("movie");

            let files = s["fileEpisodes"].as_array();
            let file_count = files.map(|f| f.len()).unwrap_or(0);
            let latest_file = files
                .map(|f| {
                    f.iter()
                        .filter(|e| e["kind"].as_str() == Some("episode"))
                        .filter_map(|e| e["episodeNumber"].as_f64())
                        .fold(None::<f64>, |acc, n| Some(acc.map_or(n, |a| a.max(n))))
                })
                .flatten();

            let progress = provider_key(&s["anilistId"])
                .and_then(|k| anilist_progress.get(&k))
                .or_else(|| provider_key(&s["malId"]).and_then(|k| mal_progress.get(&k)));
            let (watched, my_score, list_status) = match progress {
                Some(p) => (num_or_null(&p["progress"]), num_or_null(&p["score"]), str_or_null(&p["status"])),
                None => (Value::Null, Value::Null, Value::Null),
            };

            let mut latest_aired: Option<f64> = None;
            let mut next_air: Option<i64> = None;
            if let Some(eps) = s["episodes"].as_array() {
                for e in eps {
                    let Some(air) = e["airDate"].as_str().and_then(parse_iso_ms) else { continue };
                    let Some(n) = e["episodeNumber"].as_f64() else { continue };
                    if air <= now {
                        latest_aired = Some(latest_aired.map_or(n, |a| a.max(n)));
                    } else {
                        next_air = Some(next_air.map_or(air, |a| a.min(air)));
                    }
                }
            }

            let source = s["source"].as_str();
            let score = s["averageScore"].as_f64().map(|v| {
                let n = if source == Some("anilist") { v / 10.0 } else { v };
                json!((n * 10.0).round() / 10.0)
            });

            let last_viewed = history
                .get(&id)
                .and_then(|h| h["lastViewedAt"].as_i64())
                .map(Value::from)
                .unwrap_or(Value::Null);

            json!({
                "id": id,
                "folderName": folder_name,
                "titleRomaji": str_or_null(&s["titleRomaji"]),
                "titleEnglish": str_or_null(&s["titleEnglish"]),
                "matchedTitle": str_or_null(&s["matchedTitle"]),
                "title": str_or_null(&s["title"]),
                "poster": poster,
                "isMovie": is_movie,
                "fileCount": file_count,
                "latestFile": tidy(latest_file),
                "watched": watched,
                "myScore": my_score,
                "listStatus": list_status,
                "total": num_or_null(&s["totalEpisodes"]),
                "latestAired": tidy(latest_aired),
                "nextAirMs": next_air,
                "status": str_or_null(&s["status"]),
                "format": str_or_null(&s["format"]),
                "source": str_or_null(&s["source"]),
                "year": num_or_null(&s["seasonYear"]),
                "score": score.unwrap_or(Value::Null),
                "lastViewedAt": last_viewed,
                "hidden": false,
            })
        })
        .collect();

    Value::Array(records)
}
