//! The phone's search: one source's feed, read by the core, its items turned into ranked
//! releases. The same shapes anirss gives the desktop, without anirss: nyaa's RSS with its
//! `q`, `c` and `f` parameters, and any feed whose URL holds `{query}` (erai's).

use crate::contract::*;
use crate::downloads::facts;

/// Release groups whose encodes are known to be clean; the desktop's best fit trusts the
/// same names.
const TRUSTED: [&str; 8] = [
    "SubsPlease",
    "Erai-raws",
    "EMBER",
    "ASW",
    "Judas",
    "Yameii",
    "ToonsHub",
    "VARYG",
];

/// The URL that asks `source` for `query`.
pub fn query_url(source: &PhoneSource, query: &str) -> String {
    match source.kind.as_str() {
        "nyaa" => {
            let base = source.url.trim_end_matches('/');
            format!("{base}/?page=rss&q={}&c=1_0&f=0", encode(query))
        }
        _ => {
            // erai's `s` is a word AND over the post text, which never holds the quality
            // tokens; they go, and the results are what the feed's own filters keep.
            let query = if source.url.contains("erai-raws") {
                without_quality(query)
            } else {
                query.to_string()
            };
            source.url.replace("{query}", &encode(&query))
        }
    }
}

/// Form encoding: letters, digits and `-_.~` stay, a space is `+`, the rest `%XX`.
pub fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.trim().bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            b => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn without_quality(query: &str) -> String {
    query
        .split_whitespace()
        .filter(|w| {
            let w = w.to_ascii_lowercase();
            !(w.ends_with('p')
                && w[..w.len() - 1].chars().all(|c| c.is_ascii_digit())
                && w.len() > 3
                || matches!(
                    w.as_str(),
                    "hevc" | "x265" | "x264" | "avc" | "web-dl" | "webrip" | "bluray" | "aac"
                ))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A feed item as a release from `source`. erai's titles carry a `[Torrent] ` lead and its
/// links hold spaces and brackets.
pub fn to_release(entry: &FeedEntry, source: &str) -> Release {
    let title = entry
        .title
        .strip_prefix("[Torrent] ")
        .unwrap_or(&entry.title)
        .trim()
        .to_string();
    Release {
        title,
        link: crate::downloads::anirss::url_safe(&entry.link),
        seeders: entry.seeders,
        leechers: entry.leechers,
        downloads: entry.downloads,
        size: entry.size.clone(),
        category: String::new(),
        page: entry.page.clone(),
        source: source.to_string(),
        resolution: String::new(),
        infomkv: String::new(),
        linktype: if entry.link.starts_with("magnet:") {
            "magnet"
        } else {
            "torrent"
        }
        .to_string(),
    }
}

fn resolution_points(resolution: &str) -> i64 {
    match resolution.to_ascii_lowercase().as_str() {
        "1080p" => 30,
        "720p" => 20,
        "2160p" | "4k" => 12,
        "480p" => 5,
        _ => 0,
    }
}

/// How good a pick a release is for a phone: 1080p from a trusted group with seeders,
/// one episode at a size a phone can hold.
pub fn score(release: &Release, facts: &ReleaseFacts, size: Option<u64>) -> i64 {
    let mut score = resolution_points(&facts.resolution);
    if TRUSTED.iter().any(|g| g.eq_ignore_ascii_case(&facts.group)) {
        score += 20;
    }
    let seeders = release.seeders.unwrap_or(0).max(0) as f64;
    score += ((seeders + 1.0).log2() * 5.0).min(35.0) as i64;
    if release.seeders == Some(0) {
        score -= 40;
    }
    // Phones decode HEVC in hardware almost everywhere now, but AV1 not yet.
    if release.title.to_ascii_uppercase().contains("AV1") {
        score -= 10;
    }
    // One episode past 4 GB is a remux or a raw: too much for a phone.
    if let (Some(bytes), false) = (size, facts.batch)
        && bytes > 4_000_000_000
    {
        score -= 15;
    }
    score
}

/// The releases with their facts and scores, in the feed's order, and which one ranks
/// highest.
pub fn rank(releases: Vec<Release>) -> (Vec<PhoneRelease>, Option<u64>) {
    let results: Vec<PhoneRelease> = releases
        .into_iter()
        .map(|release| {
            let facts = facts::read(&release);
            let size_bytes = facts
                .size_bytes
                .or_else(|| facts::size_bytes(&release.size));
            let score = score(&release, &facts, size_bytes);
            PhoneRelease {
                release,
                facts,
                size_bytes,
                score,
            }
        })
        .collect();
    let best = results
        .iter()
        .enumerate()
        .max_by_key(|(_, r)| r.score)
        .map(|(i, _)| i as u64);
    (results, best)
}

/// The show's own name from a release title, for its folder in the library:
/// `[SubsPlease] Sousou no Frieren - 01 (1080p) [ABCD].mkv` is `Sousou no Frieren`.
pub fn show_name(title: &str) -> String {
    let mut t = title.trim();
    // Leading groups: `[SubsPlease] `, `[Erai-raws] `.
    while let Some(rest) = t
        .strip_prefix('[')
        .and_then(|r| r.split_once(']').map(|(_, rest)| rest.trim_start()))
    {
        t = rest;
    }
    let mut name = t.to_string();
    // The episode and everything after it.
    for pattern in [
        r" - \d",
        r" S\d+E\d+",
        r" E\d+\b",
        r" \(",
        r" \[",
        r"\.mkv$",
        r"\.mp4$",
    ] {
        if let Ok(re) = regex::Regex::new(pattern)
            && let Some(m) = re.find(&name)
        {
            name.truncate(m.start());
        }
    }
    let clean: String = name
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                ' '
            } else {
                c
            }
        })
        .collect();
    let clean = clean.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.is_empty() {
        "Downloads".to_string()
    } else {
        clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(kind: &str, url: &str) -> PhoneSource {
        PhoneSource {
            name: "x".into(),
            kind: kind.into(),
            url: url.into(),
            enabled: true,
        }
    }

    #[test]
    fn nyaa_and_template_sources_build_their_search_urls() {
        assert_eq!(
            query_url(&source("nyaa", "https://nyaa.si/"), "Frieren 1080p"),
            "https://nyaa.si/?page=rss&q=Frieren+1080p&c=1_0&f=0"
        );
        assert_eq!(
            query_url(
                &source(
                    "rss",
                    "https://www.erai-raws.info/episodes/feed/?res=1080p&s={query}&token=T"
                ),
                "Frieren 1080p HEVC"
            ),
            "https://www.erai-raws.info/episodes/feed/?res=1080p&s=Frieren&token=T"
        );
    }

    #[test]
    fn a_release_title_gives_its_show_folder() {
        assert_eq!(
            show_name("[SubsPlease] Sousou no Frieren - 01 (1080p) [ABCD1234].mkv"),
            "Sousou no Frieren"
        );
        assert_eq!(
            show_name(
                "[Erai-raws] Kusuriya no Hitorigoto 3rd Season - 04 [1080p][Multiple Subtitle]"
            ),
            "Kusuriya no Hitorigoto 3rd Season"
        );
        assert_eq!(show_name("Show.Name S01E02 1080p"), "Show.Name");
        assert_eq!(show_name("[Group] Re:Zero"), "Re Zero");
    }

    #[test]
    fn a_trusted_1080p_release_with_seeders_ranks_first() {
        let r = |title: &str, seeders: i64| Release {
            title: title.into(),
            link: "magnet:?xt=urn:btih:00".into(),
            seeders: Some(seeders),
            leechers: None,
            downloads: None,
            size: "1.4 GiB".into(),
            category: String::new(),
            page: String::new(),
            source: "nyaa".into(),
            resolution: String::new(),
            infomkv: String::new(),
            linktype: "magnet".into(),
        };
        let (results, best) = rank(vec![
            r("[Nobody] Frieren - 01 (720p)", 3),
            r("[SubsPlease] Frieren - 01 (1080p)", 120),
            r("[SubsPlease] Frieren - 01 (1080p)", 0),
        ]);
        assert_eq!(best, Some(1));
        assert!(results[2].score < results[0].score, "no seeders ranks last");
    }
}
