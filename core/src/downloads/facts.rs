//! What a release title says that anirss does not split out: the episode
//! (and season), whether it is a batch, the release group, the resolution,
//! and the size in bytes for sorting. Release names follow a few house
//! styles, and this reads the common ones and nothing cleverer:
//!
//! - fansub: `[SubsPlease] Sousou no Frieren - 01 (1080p) [ABCD1234].mkv`
//! - Erai: `[Erai-raws] Sousou no Frieren - 28 [1080p CR WEB-DL AVC AAC][MultiSub]`
//! - scene: `Avatar Seven Havens S01E03 1080p WEB h264-ETHEL`
//! - batches: `[SubsPlease] Frieren (01-28) (1080p) [Batch]`, `Show S01 1080p`
//!
//! The library's file classifier is not used here: a file with no number
//! in it is episode 1 to the library, which is right for a film on disk
//! and wrong for a release list, where it would mark a batch or a film as
//! episode 1. A title this cannot read gets no episode at all.

use std::sync::LazyLock;

use regex::Regex;

use crate::contract::*;

static GROUP_LEAD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*\[([^\]]+)\]").unwrap());
/// A scene group at the end: `h264-ETHEL`, `x265-NeoNoir[EZTVx.to]`
static GROUP_TAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-([A-Za-z0-9]{2,20})(?:\[[^\]]*\])?(?:\.[a-z0-9]{2,4})?\s*$").unwrap()
});
static RESOLUTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(4320|2160|1440|1080|720|576|540|480|360)[pi]\b").unwrap());
static FOUR_K: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(4k|uhd)\b").unwrap());
static SEASON_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bS(\d{1,2})\s?E(\d{1,4}(?:\.\d)?)(?:v\d)?\b").unwrap());
/// The fansub convention: ` - 01`, ` - 12.5`, ` - 03v2`
static DASH_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s-\s(\d{1,4}(?:\.\d)?)(?:v\d)?(?:\s|$|[\[(.])").unwrap());
static WORD_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(?:ep|episode|e)\s?\.?\s?(\d{1,4})\b").unwrap());
/// `(01-28)`, `01 ~ 12`, `E01-E12`
static RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:^|[\s(\[_])(?:e|ep)?(\d{1,4})\s?[-~]\s?(?:e|ep)?(\d{1,4})(?:$|[\s)\]_v])")
        .unwrap()
});
static BATCH_WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(batch|complete series|complete season|season pack)\b").unwrap()
});
/// A season with no episode after it: `S01 1080p`, `S02.1080p`
static SEASON_PACK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bS(\d{1,2})(?:[\s._\[(]|$)").unwrap());
/// Where the video came from: `WEB-DL`, `WEBRip`, `WEB`, `BluRay`, `BDRip`, `HDTV`, `DVD`
static RIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:^|[^a-z0-9])(web[-. ]?dl|web[-. ]?rip|web|blu[-. ]?ray|bd[-. ]?rip|bd|hdtv|dvd[-. ]?rip|dvd)(?:$|[^a-z0-9])").unwrap()
});
/// A version after the episode number: ` - 03v2`, `S01E03v2`
static VERSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:\bS\d{1,2}\s?E|\s-\s)\d{1,4}(?:\.\d)?v(\d)\b").unwrap());
/// The CRC32 a fansub group tags its file with: `[80977B2B]`
static CRC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([0-9A-Fa-f]{8})\]").unwrap());
static SIZE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*([\d.,]+)\s*(B|KB|KIB|MB|MIB|GB|GIB|TB|TIB)\s*$").unwrap()
});

/// The facts of a bare name, a file's or a torrent's, read as a release
/// title is; a path counts by its last part.
pub fn read_name(name: &str) -> ReleaseFacts {
    let base = name.rsplit('/').next().unwrap_or(name);
    read(&Release {
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
    })
}

pub fn read(item: &Release) -> ReleaseFacts {
    let title = item.title.as_str();
    let group = GROUP_LEAD
        .captures(title)
        .map(|c| c[1].trim().to_string())
        .or_else(|| {
            // A scene name has no brackets up front; its group trails it
            (!title.trim_start().starts_with('[') && SEASON_EPISODE.is_match(title)
                || SEASON_PACK.is_match(title))
            .then(|| GROUP_TAIL.captures(title).map(|c| c[1].to_string()))
            .flatten()
        })
        .unwrap_or_default();
    let resolution = if !item.resolution.trim().is_empty() {
        item.resolution.trim().to_string()
    } else if let Some(c) = RESOLUTION.captures(title) {
        format!("{}p", &c[1])
    } else if FOUR_K.is_match(title) {
        "2160p".to_string()
    } else {
        String::new()
    };

    let mut season = None;
    let mut episode = None;
    if let Some(c) = SEASON_EPISODE.captures(title) {
        season = c[1].parse().ok();
        episode = c[2].parse().ok();
    }
    let batch = episode.is_none()
        && (BATCH_WORD.is_match(title)
            || ranged(title)
            // "Show S2 - 03" names a season and then its episode: not a pack
            || (!DASH_EPISODE.is_match(title)
                && SEASON_PACK
                    .captures(title)
                    .inspect(|c| season = c[1].parse().ok())
                    .is_some()));
    if !batch && episode.is_none() {
        episode = DASH_EPISODE
            .captures(title)
            .and_then(|c| c[1].parse().ok())
            .or_else(|| WORD_EPISODE.captures(title).and_then(|c| c[1].parse().ok()))
            .filter(|n: &f64| *n < 2000.0 || *n > 2099.0);
    }
    let span = if batch { span(title) } else { None };
    ReleaseFacts {
        episode,
        season,
        batch,
        first_episode: span.map(|s| s.0),
        last_episode: span.map(|s| s.1),
        group,
        resolution,
        size_bytes: size_bytes(&item.size),
        rip: rip(title),
        on_disk: false,
        other_on_disk: false,
    }
}

/// The rip a title names, in its usual spelling; empty when none.
pub fn rip(title: &str) -> String {
    let Some(c) = RIP.captures(title) else {
        return String::new();
    };
    let word: String = c[1]
        .to_ascii_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();
    match word.as_str() {
        "webdl" => "WEB-DL",
        "webrip" => "WEBRip",
        "web" => "WEB",
        "bluray" | "bd" => "BluRay",
        "bdrip" => "BDRip",
        "hdtv" => "HDTV",
        "dvdrip" => "DVDRip",
        _ => "DVD",
    }
    .to_string()
}

/// The release's version of its episode: 2 for ` - 03v2`, 1 when it names none.
pub fn version(title: &str) -> u32 {
    VERSION
        .captures(title)
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(1)
}

/// A name as compared: no folders, no video extension, lower case, and dots,
/// underscores and runs of spaces as one space.
fn plain(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let lower = base.to_lowercase();
    let stem = [".mkv", ".mp4", ".avi", ".webm", ".m4v", ".ts"]
        .iter()
        .find_map(|e| lower.strip_suffix(e))
        .unwrap_or(&lower);
    stem.replace(['.', '_'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether the file `file` is the release `title` (#237): the same name, or,
/// when a torrent named itself apart from its file, the same group, and the
/// same resolution, rip and version wherever both name one. Two CRC tags
/// that differ settle it the other way. A release or a file naming no group
/// is only ever its own name.
pub fn same_release(title: &str, file: &str) -> bool {
    if plain(title) == plain(file) {
        return true;
    }
    let crc = |s: &str| {
        CRC.captures_iter(s)
            .last()
            .map(|c| c[1].to_ascii_uppercase())
    };
    if let (Some(a), Some(b)) = (crc(title), crc(file)) {
        return a == b;
    }
    let (r, f) = (read_name(title), read_name(file));
    let agree = |a: &str, b: &str| a.is_empty() || b.is_empty() || a.eq_ignore_ascii_case(b);
    !r.group.is_empty()
        && r.group.eq_ignore_ascii_case(&f.group)
        && agree(&r.resolution, &f.resolution)
        && agree(&r.rip, &f.rip)
        && version(title) == version(file)
}

/// A range of episodes, not a year span or a resolution pair.
fn ranged(title: &str) -> bool {
    span(title).is_some()
}

/// The first range of episodes a title names, as first and last.
fn span(title: &str) -> Option<(u32, u32)> {
    RANGE.captures_iter(title).find_map(|c| {
        let (a, b) = (
            c[1].parse::<u32>().unwrap_or(0),
            c[2].parse::<u32>().unwrap_or(0),
        );
        (a < b && b < 1900 && b - a < 500).then_some((a, b))
    })
}

/// "1.4 GiB", "700 MB", "1,234.5 MiB" as bytes; a decimal unit is read as
/// the binary one, since every feed that writes "GB" means GiB.
pub fn size_bytes(text: &str) -> Option<u64> {
    let c = SIZE.captures(text)?;
    let n: f64 = c[1].replace(',', "").parse().ok()?;
    let unit = c[2].to_ascii_uppercase();
    let scale: f64 = match unit.trim_end_matches('B').trim_end_matches('I') {
        "" => 1.0,
        "K" => 1024.0,
        "M" => 1024.0 * 1024.0,
        "G" => 1024.0 * 1024.0 * 1024.0,
        "T" => 1024.0_f64.powi(4),
        _ => return None,
    };
    Some((n * scale).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(title: &str, size: &str) -> Release {
        serde_json::from_value(serde_json::json!({"title": title, "link": "x", "size": size}))
            .unwrap()
    }

    #[test]
    fn fansub_erai_and_scene_names_read() {
        let f = read(&release(
            "[SubsPlease] Sousou no Frieren - 01 (1080p) [ABCD1234].mkv",
            "1.4 GiB",
        ));
        assert_eq!((f.episode, f.season, f.batch), (Some(1.0), None, false));
        assert_eq!(
            (f.group.as_str(), f.resolution.as_str()),
            ("SubsPlease", "1080p")
        );
        assert_eq!(f.size_bytes, Some(1_503_238_554));

        let f = read(&release(
            "[Erai-raws] Sousou no Frieren - 28 [1080p CR WEB-DL AVC AAC][MultiSub][D2E8BC01]",
            "",
        ));
        assert_eq!(
            (f.episode, f.group.as_str(), f.resolution.as_str()),
            (Some(28.0), "Erai-raws", "1080p")
        );

        let f = read(&release(
            "[SubsPlease] Kusuriya no Hitorigoto - 12.5 (720p) [F00].mkv",
            "",
        ));
        assert_eq!(f.episode, Some(12.5));

        let f = read(&release("[ASW] Show - 03v2 [1080p HEVC][AAAA].mkv", ""));
        assert_eq!(f.episode, Some(3.0));

        let f = read(&release(
            "Avatar Seven Havens S01E03 1080p WEB h264-ETHEL",
            "1.5 GB",
        ));
        assert_eq!((f.season, f.episode, f.batch), (Some(1), Some(3.0), false));
        assert_eq!(
            (f.group.as_str(), f.resolution.as_str()),
            ("ETHEL", "1080p")
        );
    }

    #[test]
    fn a_season_then_a_dashed_episode_is_one_episode() {
        let f = read(&release(
            "[Naruto-Kun.Hu] Black Clover S2 - 03 [1080p].mkv",
            "",
        ));
        assert_eq!((f.episode, f.batch), (Some(3.0), false));
    }

    #[test]
    fn batches_carry_no_episode() {
        let f = read(&release(
            "[SubsPlease] Sousou no Frieren (01-28) (1080p) [Batch]",
            "",
        ));
        assert!(f.batch);
        assert_eq!(f.episode, None);
        assert_eq!((f.first_episode, f.last_episode), (Some(1), Some(28)));
        let f = read(&release(
            "[Erai-raws] Kino no Tabi: The Beautiful World - The Animated Series - 01 ~ 12 [1080p][Batch]",
            "",
        ));
        assert!(f.batch);
        assert_eq!((f.first_episode, f.last_episode), (Some(1), Some(12)));
        let f = read(&release(
            "Avatar Seven Havens S01 1080p WEB-DL x264-GROUP",
            "",
        ));
        assert!(f.batch);
        assert_eq!((f.season, f.episode), (Some(1), None));
        assert_eq!(f.first_episode, None);
        let f = read(&release("[Judas] Frieren - Complete Season [1080p]", ""));
        assert!(f.batch);
    }

    #[test]
    fn a_title_with_no_episode_gets_none_not_one() {
        let f = read(&release("Perfect Blue (1997) [BD 1080p HEVC]", ""));
        assert_eq!((f.episode, f.batch), (None, false));
        assert_eq!(f.resolution, "1080p");
        let f = read(&release("Some Film 2160p UHD BluRay", ""));
        assert_eq!(f.resolution, "2160p");
    }

    #[test]
    fn sizes_read_in_any_unit() {
        assert_eq!(size_bytes("700 MiB"), Some(734_003_200));
        assert_eq!(size_bytes("700 MB"), Some(734_003_200));
        assert_eq!(size_bytes("1,024 KiB"), Some(1_048_576));
        assert_eq!(size_bytes("12 B"), Some(12));
        assert_eq!(size_bytes(""), None);
        assert_eq!(size_bytes("big"), None);
    }

    #[test]
    fn rips_read_in_their_usual_spelling() {
        assert_eq!(
            rip("[Erai-raws] Show - 28 [1080p CR WEB-DL AVC AAC]"),
            "WEB-DL"
        );
        assert_eq!(rip("Show S01E03 1080p WEBRip x265-Group"), "WEBRip");
        assert_eq!(rip("Show.S01E03.1080p.WEB.h264-ETHEL"), "WEB");
        assert_eq!(rip("[Group] Show - 01 (BD 1080p)"), "BluRay");
        assert_eq!(rip("[SubsPlease] Show - 01 (1080p) [ABCD1234].mkv"), "");
    }

    #[test]
    fn only_the_same_release_is_the_file_on_disk() {
        let file = "/lib/Show/[SubsPlease] Show - 05 (1080p) [ABCD1234].mkv";
        assert!(same_release(
            "[SubsPlease] Show - 05 (1080p) [ABCD1234].mkv",
            file
        ));
        // The same group and resolution, named apart: still this release
        assert!(same_release("[SubsPlease] Show - 05 (1080p)", file));
        // Another group, another resolution, another version: not it
        assert!(!same_release(
            "[Erai-raws] Show - 05 [1080p CR WEB-DL AVC AAC]",
            file
        ));
        assert!(!same_release(
            "[SubsPlease] Show - 05 (720p) [EEEE0000].mkv",
            file
        ));
        assert!(!same_release(
            "[SubsPlease] Show - 05v2 (1080p) [FFFF0000].mkv",
            file
        ));
        assert!(same_release(
            "Show.S01E05.1080p.WEB.h264-ETHEL",
            "Show S01E05 1080p WEB h264-ETHEL.mkv"
        ));
        assert!(!same_release(
            "Show S01E05 1080p WEBRip x265-Other",
            "Show S01E05 1080p WEB h264-ETHEL.mkv"
        ));
    }

    #[test]
    fn erai_releases_of_one_episode_are_told_apart() {
        // The owner's file, and what nyaa listed for that episode (#237)
        let file = "/a/[Erai-raws] Katainaka no Ossan Kensei ni Naru II - 06 [1080p AMZN WEB-DL AVC EAC3][MultiSub][80977B2B].mkv";
        let listed = [
            (
                "[Erai-raws] Katainaka no Ossan, Kensei ni Naru II - 06 [1080p AMZN WEBRip HEVC EAC3][MultiSub][C781A1C2]",
                false,
            ),
            (
                "[Erai-raws] Katainaka no Ossan, Kensei ni Naru II - 06 [540p AMZN WEB-DL AVC EAC3][MultiSub][8B85B88D]",
                false,
            ),
            (
                "[Erai-raws] Katainaka no Ossan, Kensei ni Naru II - 06 [720p AMZN WEB-DL AVC EAC3][MultiSub][19163F35]",
                false,
            ),
            (
                "[Erai-raws] Katainaka no Ossan, Kensei ni Naru II - 06 [1080p AMZN WEB-DL AVC EAC3][MultiSub][80977B2B]",
                true,
            ),
        ];
        for (title, here) in listed {
            assert_eq!(same_release(title, file), here, "{title}");
        }
    }
}
