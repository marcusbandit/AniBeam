//! What a batch holds, read from its `.torrent`: a release title that names
//! no range ("Complete Season", "[Batch]") says nothing about which
//! episodes are inside, so the show view asks for the torrent file and the
//! core counts the episodes in its file list. Only the metadata is read;
//! nothing is added to the client. A magnet link carries no file list and
//! answers with nothing.

use std::collections::BTreeSet;

use crate::contract::*;
use crate::net::{Http, HttpRequest, Method};

/// The files a `.torrent` lists, path and size; empty when the bytes are
/// not a torrent.
pub fn files_of(bytes: &[u8]) -> Vec<(String, u64)> {
    let Some((root, _)) = parse(bytes, 0) else { return Vec::new() };
    let Some(info) = root.get(b"info") else { return Vec::new() };
    let name = info.get(b"name").and_then(Value::text).unwrap_or_default();
    match info.get(b"files").and_then(Value::list) {
        Some(files) => files
            .iter()
            .filter_map(|f| {
                let parts = f.get(b"path").and_then(Value::list)?;
                let path: Vec<String> = parts.iter().filter_map(Value::text).collect();
                let size = f.get(b"length").and_then(Value::int).unwrap_or(0).max(0) as u64;
                Some((format!("{name}/{}", path.join("/")), size))
            })
            .collect(),
        None => {
            let size = info.get(b"length").and_then(Value::int).unwrap_or(0).max(0) as u64;
            if name.is_empty() { Vec::new() } else { vec![(name, size)] }
        }
    }
}

const VIDEO: [&str; 7] = ["mkv", "mp4", "avi", "m4v", "webm", "ts", "wmv"];

/// The whole episode numbers among the video files, each read as a release
/// title is.
pub fn episodes_of(files: &[(String, u64)]) -> Vec<u32> {
    let mut out = BTreeSet::new();
    for (path, _) in files {
        let base = path.rsplit('/').next().unwrap_or(path);
        let ext = base.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        if !VIDEO.contains(&ext.as_str()) {
            continue;
        }
        let facts = super::facts::read(&Release {
            title: base.to_string(),
            link: String::new(),
            seeders: None,
            leechers: None,
            downloads: None,
            size: String::new(),
            category: String::new(),
            page: String::new(),
            source: String::new(),
            resolution: String::new(),
            infomkv: String::new(),
            linktype: String::new(),
        });
        if let Some(n) = facts.episode.filter(|n| n.is_finite() && *n >= 1.0 && n.fract() == 0.0) {
            out.insert(n as u32);
        }
    }
    out.into_iter().collect()
}

/// Fetches the torrent file behind `link` and counts its episodes. A link
/// that is not http(s), a magnet among them, answers with no files.
pub async fn inspect(http: &dyn Http, link: &str) -> Result<(Vec<u32>, u64), CoreError> {
    if !(link.starts_with("http://") || link.starts_with("https://")) {
        return Ok((Vec::new(), 0));
    }
    let r = http
        .send(HttpRequest {
            method: Method::Get,
            url: link.to_string(),
            headers: Vec::new(),
            body: None,
            timeout: None,
        })
        .await
        .map_err(|e| CoreError::Service {
            service: "torrent".to_string(),
            code: "unreachable".to_string(),
            message: format!("the torrent file did not come: {}", e.message),
        })?;
    if r.status >= 400 {
        return Err(CoreError::Service {
            service: "torrent".to_string(),
            code: format!("http_{}", r.status),
            message: format!("the torrent file answered {}", r.status),
        });
    }
    let files = files_of(&r.body);
    Ok((episodes_of(&files), files.len() as u64))
}

// A bencode reader, just enough for a torrent's info dictionary ----------

#[derive(Debug)]
enum Value {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Dict(Vec<(Vec<u8>, Value)>),
}

impl Value {
    fn get(&self, key: &[u8]) -> Option<&Value> {
        match self {
            Value::Dict(d) => d.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    fn list(&self) -> Option<&Vec<Value>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }
    fn int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }
    fn text(&self) -> Option<String> {
        match self {
            Value::Bytes(b) => Some(String::from_utf8_lossy(b).into_owned()),
            _ => None,
        }
    }
}

/// One value from `at`, and where the next one starts. Depth is bounded by
/// the input, so a hostile file can at worst be refused.
fn parse(b: &[u8], at: usize) -> Option<(Value, usize)> {
    match *b.get(at)? {
        b'i' => {
            let end = at + 1 + b[at + 1..].iter().position(|c| *c == b'e')?;
            let n = std::str::from_utf8(&b[at + 1..end]).ok()?.parse().ok()?;
            Some((Value::Int(n), end + 1))
        }
        b'l' => {
            let mut out = Vec::new();
            let mut i = at + 1;
            while *b.get(i)? != b'e' {
                let (v, next) = parse(b, i)?;
                out.push(v);
                i = next;
            }
            Some((Value::List(out), i + 1))
        }
        b'd' => {
            let mut out = Vec::new();
            let mut i = at + 1;
            while *b.get(i)? != b'e' {
                let (k, next) = parse(b, i)?;
                let Value::Bytes(k) = k else { return None };
                let (v, after) = parse(b, next)?;
                out.push((k, v));
                i = after;
            }
            Some((Value::Dict(out), i + 1))
        }
        b'0'..=b'9' => {
            let colon = at + b[at..].iter().position(|c| *c == b':')?;
            let len: usize = std::str::from_utf8(&b[at..colon]).ok()?.parse().ok()?;
            let start = colon + 1;
            let end = start.checked_add(len)?;
            Some((Value::Bytes(b.get(start..end)?.to_vec()), end))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bstr(s: &str) -> String {
        format!("{}:{s}", s.len())
    }

    fn torrent(name: &str, files: &[&str]) -> Vec<u8> {
        let list: String = files
            .iter()
            .map(|f| format!("d6:lengthi100e4:pathl{}ee", bstr(f)))
            .collect();
        format!("d8:announce{}4:infod5:filesl{list}e4:name{}12:piece lengthi16384eee", bstr("http://t"), bstr(name)).into_bytes()
    }

    #[test]
    fn a_batch_without_a_range_counts_its_episode_files() {
        let t = torrent(
            "[Group] Show (Complete Season) [1080p]",
            &[
                "[Group] Show - 01 [1080p].mkv",
                "[Group] Show - 02 [1080p].mkv",
                "[Group] Show - 03 [1080p].mkv",
                "[Group] Show - 05 [1080p].mkv",
                "Extras/NCOP.mkv",
                "readme.txt",
            ],
        );
        let files = files_of(&t);
        assert_eq!(files.len(), 6);
        assert_eq!(episodes_of(&files), vec![1, 2, 3, 5]);
    }

    #[test]
    fn three_digit_episodes_and_a_single_file() {
        let t = torrent("Naruto", &["Naruto - 001.mkv", "Naruto - 219.mkv", "Naruto - 220.mkv"]);
        assert_eq!(episodes_of(&files_of(&t)), vec![1, 219, 220]);
        let single = format!("d4:infod6:lengthi5e4:name{}ee", bstr("[G] Film - 07 [720p].mkv")).into_bytes();
        assert_eq!(files_of(&single), vec![("[G] Film - 07 [720p].mkv".to_string(), 5)]);
    }

    #[test]
    fn not_a_torrent_is_no_files() {
        assert!(files_of(b"<html>nope</html>").is_empty());
        assert!(files_of(b"d4:infod5:filesl").is_empty());
        assert!(files_of(b"99999999999999999999:x").is_empty());
    }
}
