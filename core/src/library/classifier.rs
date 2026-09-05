//! Single source of truth for what kind of file a name is: a real episode,
//! an opening, ending, promo or special, or something else. Pure string in,
//! kind and numbers out. Carried from src/shared/episodeClassifier.ts.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::contract::ExtraKind;

use super::{BRACKETS, SPACES};

#[derive(Clone, Debug, PartialEq)]
pub struct Classified {
    /// None for an episode.
    pub extra: Option<ExtraKind>,
    /// The episode number, decimals kept; an extra's index, or 0 without one.
    pub number: f64,
    pub season: Option<u32>,
    pub extra_index: Option<u32>,
    pub extra_variant: Option<String>,
    pub raw_label: Option<String>,
}

static SEPARATORS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[_.]+").unwrap());
static TRAILING_DASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*[-_]\s*$").unwrap());

/// The extras token patterns, tried in this order, each anchored at both ends
/// so "Operations" and "Edge" never fire.
static EXTRA_PATTERNS: LazyLock<Vec<(ExtraKind, Regex)>> = LazyLock::new(|| {
    vec![
        (
            ExtraKind::Op,
            Regex::new(r"(?i)^(?:NCOP|OP)([0-9]+)([a-z])?$").unwrap(),
        ),
        (
            ExtraKind::Ed,
            Regex::new(r"(?i)^(?:NCED|ED)([0-9]+)([a-z])?$").unwrap(),
        ),
        (
            ExtraKind::Pv,
            Regex::new(r"(?i)^(?:PV|Trailer|Teaser)([0-9]+)?([a-z])?$").unwrap(),
        ),
        (
            ExtraKind::Sp,
            Regex::new(r"(?i)^(?:SP|Special|Specials)([0-9]+)?([a-z])?$").unwrap(),
        ),
    ]
});
const OTHER_TOKENS: [&str; 5] = ["menu", "cm", "bonus", "extra", "extras"];

static SEASON_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bS([0-9]+)E([0-9]+)\b").unwrap());
static DECIMAL_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Episode\s*([0-9]+)\.([0-9]+)").unwrap());
static EPISODE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)Episode\s*([0-9]+)").unwrap(),
        Regex::new(r"(?i)Ep\.?\s*([0-9]+)").unwrap(),
        Regex::new(r"(?i)\bE([0-9]{2,})\b").unwrap(),
        Regex::new(r"\s-\s*([0-9]+)(?:\s|$)").unwrap(),
        Regex::new(r"\s([0-9]{1,3})(?:\s|$)").unwrap(),
    ]
});
static ANY_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[0-9]+").unwrap());

/// Extension and every [bracketed] group gone, whitespace collapsed,
/// separators kept so `Episode 6.5` and `Show.Name.S02E07` still anchor.
pub fn strip_brackets_and_ext(file_name: &str) -> String {
    let stem = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let no_brackets = BRACKETS.replace_all(&stem, " ");
    SPACES.replace_all(&no_brackets, " ").trim().to_string()
}

/// The same, then `_` and `.` become spaces so the token scan splits
/// `Bakemonogatari_ED1_...` and `Show.Name.OP1.mkv`.
fn normalise_for_tokens(file_name: &str) -> String {
    let stripped = strip_brackets_and_ext(file_name);
    let flat = SEPARATORS.replace_all(&stripped, " ");
    SPACES.replace_all(&flat, " ").trim().to_string()
}

struct ExtraHit {
    kind: ExtraKind,
    index: Option<u32>,
    variant: Option<String>,
    raw_label: String,
}

fn find_extra_token(file_name: &str) -> Option<ExtraHit> {
    for token in normalise_for_tokens(file_name).split_whitespace() {
        for (kind, re) in EXTRA_PATTERNS.iter() {
            if let Some(m) = re.captures(token) {
                return Some(ExtraHit {
                    kind: *kind,
                    index: m.get(1).and_then(|g| g.as_str().parse().ok()),
                    variant: m.get(2).map(|g| g.as_str().to_lowercase()),
                    raw_label: token.to_string(),
                });
            }
        }
        if OTHER_TOKENS.contains(&token.to_lowercase().as_str()) {
            return Some(ExtraHit {
                kind: ExtraKind::Other,
                index: None,
                variant: None,
                raw_label: token.to_string(),
            });
        }
    }
    None
}

fn finalize(season: Option<u32>, episode: f64) -> (Option<u32>, f64) {
    if episode == 0.0 {
        (Some(0), 0.0)
    } else {
        (season, episode)
    }
}

fn extract_episode(file_name: &str) -> (Option<u32>, f64) {
    let base = strip_brackets_and_ext(file_name);
    if let Some(m) = SEASON_EPISODE.captures(&base) {
        return finalize(m[1].parse().ok(), m[2].parse().unwrap_or(1.0));
    }
    if let Some(m) = DECIMAL_EPISODE.captures(&base) {
        let whole: f64 = m[1].parse().unwrap_or(0.0);
        let decimal: f64 = m[2].parse().unwrap_or(0.0);
        return finalize(None, whole + decimal / 10.0);
    }
    for re in EPISODE_PATTERNS.iter() {
        if let Some(m) = re.captures(&base) {
            return finalize(None, m[1].parse().unwrap_or(1.0));
        }
    }
    let survivors: Vec<u64> = ANY_NUMBER
        .find_iter(&base)
        .filter_map(|m| m.as_str().parse::<u64>().ok())
        .filter(|n| !(1900..=2099).contains(n) && *n < 1000)
        .collect();
    if let Some(last) = survivors.last() {
        return finalize(None, *last as f64);
    }
    (None, 1.0)
}

pub fn classify(file_name: &str) -> Classified {
    if let Some(hit) = find_extra_token(file_name) {
        let number = hit.index.unwrap_or(0) as f64;
        return Classified {
            extra: Some(hit.kind),
            number,
            season: if hit.kind == ExtraKind::Sp {
                Some(0)
            } else {
                None
            },
            extra_index: hit.index,
            extra_variant: hit.variant,
            raw_label: Some(hit.raw_label),
        };
    }
    let (season, number) = extract_episode(file_name);
    Classified {
        extra: None,
        number,
        season,
        extra_index: None,
        extra_variant: None,
        raw_label: None,
    }
}

/// `12` or `12.5`, never `12.0`.
pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// The history key: the number for an episode, the file name for an extra.
pub fn episode_key(c: &Classified, file_name: &str) -> String {
    match c.extra {
        None => format_number(c.number),
        Some(_) => file_name.to_string(),
    }
}

/// The title of last resort: bracket-stripped, a trailing `-` or `_` removed.
pub fn clean_episode_title(file_name: &str) -> String {
    let stripped = strip_brackets_and_ext(file_name);
    TRAILING_DASH.replace(&stripped, "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::ExtraKind;

    fn ep(name: &str) -> Classified {
        classify(name)
    }

    #[test]
    fn bakemonogatari_episodes_and_extras() {
        let c = ep("[Coalgirls]_Bakemonogatari_01_(1920x1080_Blu-ray_FLAC)_[9787055F].mkv");
        assert_eq!((c.extra, c.number), (None, 1.0));
        let c = ep("[Coalgirls]_Bakemonogatari_15_(1920x1080_Blu-ray_FLAC)_[256D3923].mkv");
        assert_eq!((c.extra, c.number), (None, 15.0));
        let c = ep("[Coalgirls]_Bakemonogatari_ED1_(1920x1080_Blu-ray_FLAC)_[7EE4E478].mkv");
        assert_eq!(
            (
                c.extra,
                c.extra_index,
                c.extra_variant.as_deref(),
                c.raw_label.as_deref()
            ),
            (Some(ExtraKind::Ed), Some(1), None, Some("ED1"))
        );
        let c = ep("[Coalgirls]_Bakemonogatari_ED3_(1920x1080_Blu-ray_FLAC)_[8F8AC7AF].mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Ed), Some(3)));
        let c = ep("[Coalgirls]_Bakemonogatari_OP2_(1920x1080_Blu-ray_FLAC)_[57D95944].mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Op), Some(2)));
        let c = ep("[Coalgirls]_Bakemonogatari_OP4a_(1920x1080_Blu-ray_FLAC)_[AF4FF3CC].mkv");
        assert_eq!(
            (
                c.extra,
                c.extra_index,
                c.extra_variant.as_deref(),
                c.raw_label.as_deref()
            ),
            (Some(ExtraKind::Op), Some(4), Some("a"), Some("OP4a"))
        );
        let c = ep("[Coalgirls]_Bakemonogatari_OP4b_(1920x1080_Blu-ray_FLAC)_[63162685].mkv");
        assert_eq!(
            (c.extra, c.extra_index, c.extra_variant.as_deref()),
            (Some(ExtraKind::Op), Some(4), Some("b"))
        );
        let c = ep("[Coalgirls]_Bakemonogatari_OP5b_(1920x1080_Blu-ray_FLAC)_[7B7B859A].mkv");
        assert_eq!(
            (c.extra, c.extra_index, c.extra_variant.as_deref()),
            (Some(ExtraKind::Op), Some(5), Some("b"))
        );
        let c = ep("[Coalgirls]_Bakemonogatari_PV01_(1920x1080_Blu-ray_FLAC)_[8924213A].mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Pv), Some(1)));
        let c = ep("[Coalgirls]_Bakemonogatari_PV12_(1920x1080_Blu-ray_FLAC)_[17C508BF].mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Pv), Some(12)));
    }

    #[test]
    fn standard_release_patterns() {
        assert_eq!(ep("[Erai-raws] Show Name - 01 [1080p].mkv").number, 1.0);
        let c = ep("Show.Name.S02E07.1080p.WEB.mkv");
        assert_eq!((c.number, c.season), (7.0, Some(2)));
        assert_eq!(ep("Show Name - Episode 12.mkv").number, 12.0);
        assert_eq!(ep("Show Name Episode 6.5.mkv").number, 6.5);
        assert_eq!(
            (
                ep("Show Name NCOP1 [1080p].mkv").extra,
                ep("Show Name NCOP1 [1080p].mkv").extra_index
            ),
            (Some(ExtraKind::Op), Some(1))
        );
        assert_eq!(
            (
                ep("Show Name NCED2 [1080p].mkv").extra,
                ep("Show Name NCED2 [1080p].mkv").extra_index
            ),
            (Some(ExtraKind::Ed), Some(2))
        );
        let c = ep("Show Name SP1 [1080p].mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Sp), Some(1)));
        let c = ep("Show Name Special [1080p].mkv");
        assert_eq!(
            (c.extra, c.extra_index, c.number, c.season),
            (Some(ExtraKind::Sp), None, 0.0, Some(0))
        );
        assert_eq!(
            (
                ep("Show Name Trailer1.mkv").extra,
                ep("Show Name Trailer1.mkv").extra_index
            ),
            (Some(ExtraKind::Pv), Some(1))
        );
        assert_eq!(
            (
                ep("Show Name Trailer.mkv").extra,
                ep("Show Name Trailer.mkv").extra_index
            ),
            (Some(ExtraKind::Pv), None)
        );
        assert_eq!(
            (
                ep("Show Name - 03 - Operations of Hope.mkv").extra,
                ep("Show Name - 03 - Operations of Hope.mkv").number
            ),
            (None, 3.0)
        );
        assert_eq!(
            (
                ep("Show Name - 04 - Edge of Tomorrow.mkv").extra,
                ep("Show Name - 04 - Edge of Tomorrow.mkv").number
            ),
            (None, 4.0)
        );
    }

    #[test]
    fn the_fallback_drops_years_and_big_numbers_and_defaults_to_one() {
        assert_eq!(ep("Show.Name.2019.1080p.Part7.mkv").number, 7.0);
        assert_eq!(ep("Show Name.mkv").number, 1.0);
        let c = ep("Show Name - 00.mkv");
        assert_eq!((c.number, c.season), (0.0, Some(0)));
        assert_eq!(ep("Show - E07.mkv").number, 7.0);
        assert_eq!(ep("Show Ep. 9.mkv").number, 9.0);
    }

    #[test]
    fn other_tokens_are_extras_of_kind_other() {
        let c = ep("Show Name Menu.mkv");
        assert_eq!(
            (c.extra, c.raw_label.as_deref(), c.number),
            (Some(ExtraKind::Other), Some("Menu"), 0.0)
        );
        let c = ep("Show_Name_CM_01.mkv");
        assert_eq!(
            (c.extra, c.raw_label.as_deref()),
            (Some(ExtraKind::Other), Some("CM"))
        );
    }

    #[test]
    fn keys_titles_and_numbers() {
        let c = ep("Show Name Episode 6.5.mkv");
        assert_eq!(episode_key(&c, "Show Name Episode 6.5.mkv"), "6.5");
        let c = ep("Show - 12.mkv");
        assert_eq!(episode_key(&c, "Show - 12.mkv"), "12");
        let c = ep("Show OP1.mkv");
        assert_eq!(episode_key(&c, "Show OP1.mkv"), "Show OP1.mkv");
        assert_eq!(format_number(12.0), "12");
        assert_eq!(format_number(12.5), "12.5");
        assert_eq!(
            clean_episode_title("[Group] Show - 01 - [ABCD1234].mkv"),
            "Show - 01"
        );
        assert_eq!(clean_episode_title("Show_Name_03_.mkv"), "Show_Name_03");
        assert_eq!(strip_brackets_and_ext("[A] Show [B] 01 [C].mkv"), "Show 01");
    }
}
