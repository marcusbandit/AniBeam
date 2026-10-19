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
static SIZE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*([\d.,]+)\s*(B|KB|KIB|MB|MIB|GB|GIB|TB|TIB)\s*$").unwrap()
});

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
        on_disk: false,
    }
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
        let f = read(&release("[Naruto-Kun.Hu] Black Clover S2 - 03 [1080p].mkv", ""));
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
}
