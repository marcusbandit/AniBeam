//! Rank releases for the best-fit step. Ported from anirss's `bestfit.py`.
//!
//! Given the current results, find the single best release from a trusted group at the
//! best available quality, then expose the (group + resolution + source) tokens that
//! define its profile so the refine loop can refetch every matching release.
//!
//! Everything here is pure: parsing helpers plus a `score()` that turns a release and a
//! config into a comparable tuple. The tuple's field order is the policy:
//!
//! ```text
//! (is_trusted, size_rank, source_rank, resolution, has_subs, audio_rank,
//!  group_rank, seeders, downloads)
//! ```
//!
//! Trusted groups win first. Then comes size adequacy: a release starved of bitrate is
//! the one defect no other virtue makes up for, so it outranks even the source type (a
//! properly encoded WEB-DL beats a BDRip squeezed to half the bits it needed). Then the
//! source type (BluRay over WEB-DL over WebRip), resolution, subtitle presence, the audio
//! variant (Dual-Audio beats Multi-Audio: JP+EN is all we play, the extra dubs just cost
//! disk), which trusted group it is, and finally raw popularity. Group and source
//! priority and the size target come from config, so the ranking is data-driven.

use super::BestfitConfig;
use super::titles::{EPISODE_RE, RES_RE, SEASON_EP_RE, poster_of, season_of, show_name};
use crate::contract::Release;
use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::sync::LazyLock;

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("best fit pattern compiles")
}

fn fancy(pattern: &str) -> FancyRegex {
    FancyRegex::new(pattern).expect("best fit pattern compiles")
}

/// Punctuation that is noise in a search query (and that nyaa tokenises away anyway).
/// Stripped from the extracted show name so the rebuilt query reads clean:
/// "Heroine? Seijo? Iie, ..." becomes "Heroine Seijo Iie ...". Colons, dots, parentheses
/// and the like are kept since they can carry meaning (Re:Zero).
static TITLE_NOISE_RE: LazyLock<Regex> = LazyLock::new(|| re(r"[?!,]+"));

/// Source-type detection. List order is detection precedence: WEB-DL and WEBRip are tried
/// before bare WEB so "WEB-DL" never reads as a plain "WEB". Which source is preferred is
/// a separate question, decided by the config's source order.
static SOURCE_PATTERNS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    vec![
        ("WEB-DL", re(r"(?i)WEB[\s._-]?DL")),
        ("WEBRip", re(r"(?i)WEB[\s._-]?Rip")),
        ("BluRay", re(r"(?i)Blu[\s._-]?Ray|BD[\s._-]?Rip|\bBD\b")),
        ("WEB", re(r"(?i)\bWEB\b")),
        ("HDTV", re(r"(?i)\bHDTV\b")),
    ]
});
static SUBS_RE: LazyLock<Regex> = LazyLock::new(|| re(r"(?i)Multi[\s._-]?Subs?|Multiple Subtitle"));

/// Audio-variant detection, same shape as the source patterns. List order is both
/// detection precedence and preference: Dual-Audio first. The bare "Multi" form must not
/// swallow "Multi Subs", hence the lookahead.
static AUDIO_PATTERNS: LazyLock<Vec<(&'static str, FancyRegex)>> = LazyLock::new(|| {
    vec![
        ("Dual", fancy(r"(?i)Dual[\s._-]?Audio|\bDual\b")),
        (
            "Multi",
            fancy(r"(?i)Multi[\s._-]?Audio|\bMulti\b(?![\s._-]?Sub)"),
        ),
    ]
});

// Size adequacy. "12 episodes in 6 GiB" is the complaint this section answers. Raw bytes
// cannot be compared across releases directly: HEVC needs about a third fewer bits than
// x264 for the same result, and 720p needs a fraction of what 1080p does. So a release's
// per-episode size is normalised to a "1080p x264-equivalent" figure and compared against
// one config target.

static SIZE_RE: LazyLock<Regex> = LazyLock::new(|| re(r"(?i)(\d[\d.,]*)\s*([KMGT])?i?B\b"));

fn unit_mib(unit: &str) -> f64 {
    match unit {
        "" => 1.0 / (1024.0 * 1024.0),
        "k" => 1.0 / 1024.0,
        "m" => 1.0,
        "g" => 1024.0,
        _ => 1024.0 * 1024.0,
    }
}

/// Codec efficiency relative to AVC/x264 (1.0): how many times smaller a file in this
/// codec can be while still looking the same. List order is detection precedence. An
/// untagged title falls back to 1.0, the least efficient codec, which reads its size as
/// generously as possible.
static CODEC_EFFICIENCY: LazyLock<Vec<(&'static str, f64, Regex)>> = LazyLock::new(|| {
    vec![
        ("AV1", 1.8, re(r"(?i)\bAV1\b")),
        ("HEVC", 1.5, re(r"(?i)\bHEVC\b|\b[xh][\s._-]?265\b")),
        ("AVC", 1.0, re(r"(?i)\bAVC\b|\b[xh][\s._-]?264\b")),
    ]
});
const DEFAULT_CODEC_EFFICIENCY: f64 = 1.0;

/// Bitrate scales with pixel count to roughly the 0.75 power, and pixel count scales with
/// the square of the vertical resolution, so the per-episode target scales as
/// `(res / 1080) ** 1.5`. That puts 720p at 0.54x the 1080p target and 2160p at 2.83x.
const RES_TARGET_EXPONENT: f64 = 1.5;
const TARGET_BASE_RES: f64 = 1080.0;

/// Adequacy bands for the normalised per-episode size, as (min_ratio, rank), checked
/// top-down. Unknown size sits at 0 so a release whose size or episode count cannot be
/// parsed neither gains nor loses ground against one that can.
const SIZE_BANDS: [(f64, i64); 3] = [
    (0.85, 1),  // adequate: enough bits for its resolution and codec
    (0.60, -1), // thin: under-provisioned, banding on gradients and dark scenes
    (0.00, -2), // starved
];
const UNKNOWN_SIZE_RANK: i64 = 0;

/// Episode spans: "01-12", "(1-12)", "07~12", "S01E01-E12". The lookbehind rejects date-
/// and version-like neighbours ("2021-07-12", "x264-2"), which would otherwise read as an
/// episode span.
static EP_RANGE_RE: LazyLock<FancyRegex> =
    LazyLock::new(|| fancy(r"(?<![\d\-./~])[Ee]?(\d{1,3})\s*[-~]\s*[Ee]?(\d{1,3})(?![\d.])"));

/// Marks a release as covering a whole season rather than one episode or a movie. Only
/// these may borrow the result set's consensus episode count; without it a countless
/// title (typically a film) would be divided by 12 and look starved.
static BATCH_RE: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)\bBatch\b|\bSeason\b|\bS\d{1,2}\b|\bComplete\b|\bFin\b|合集|全集"));

/// The ranking tuple: higher sorts as better. Field order is the policy.
pub type Score = (i64, i64, i64, i64, i64, i64, i64, i64, i64);

/// (canonical name, literal match) of the first source tag found, else None.
///
/// The literal match is what actually appears in the title (`WEB-DL` against `WEBDL`), so
/// it can be pinned back into a nyaa query and still match.
pub fn source_of(title: &str) -> Option<(&'static str, String)> {
    SOURCE_PATTERNS
        .iter()
        .find_map(|(name, pattern)| pattern.find(title).map(|m| (*name, m.as_str().to_string())))
}

/// (canonical name, literal match) of the first audio-variant tag found, else None. Same
/// contract as `source_of`: the literal is what the title actually says (`DUAL` against
/// `Dual-Audio`) so it can be pinned back into a query and still match.
pub fn audio_of(title: &str) -> Option<(&'static str, String)> {
    AUDIO_PATTERNS.iter().find_map(|(name, pattern)| {
        pattern
            .find(title)
            .ok()
            .flatten()
            .map(|m| (*name, m.as_str().to_string()))
    })
}

/// Rank of the title's audio variant in the pattern order (first is highest), 0 when the
/// title carries no audio tag.
fn audio_rank(title: &str) -> i64 {
    let Some((found, _)) = audio_of(title) else {
        return 0;
    };
    let index = AUDIO_PATTERNS
        .iter()
        .position(|(name, _)| *name == found)
        .unwrap_or(0);
    (AUDIO_PATTERNS.len() - index) as i64
}

/// Highest `<n>p` resolution in the title (1080, say), or 0 if none.
pub fn resolution_of(title: &str) -> i64 {
    RES_RE
        .captures_iter(title)
        .filter_map(|c| c[1].parse::<i64>().ok())
        .max()
        .unwrap_or(0)
}

pub fn has_subs(title: &str) -> bool {
    SUBS_RE.is_match(title)
}

/// Rank of the title's release group in `preferred` (first listed is highest), or 0 when
/// the group is absent or not in the trusted list.
fn group_rank(title: &str, preferred: &[String]) -> i64 {
    let Some(poster) = poster_of(title) else {
        return 0;
    };
    let core = poster.trim_matches(['[', ']']).to_lowercase();
    let lowered: Vec<String> = preferred.iter().map(|g| g.to_lowercase()).collect();
    match lowered.iter().position(|g| *g == core) {
        Some(index) => (lowered.len() - index) as i64,
        None => 0,
    }
}

/// Rank of the title's source type in `source_order` (first is highest), 0 when the type
/// is absent and no `assumed` fallback places it.
///
/// Plenty of groups (SubsPlease, Erai-raws) never write a source tag at all. Scoring those
/// as 0 buried them below every tagged release no matter how trusted they were, so
/// `assumed` lets config name what an untagged release most likely is (in practice a
/// simulcast "WEB") instead of assuming the worst.
fn source_rank(title: &str, source_order: &[String], assumed: &str) -> i64 {
    let name = match source_of(title) {
        Some((name, _)) => name.to_lowercase(),
        None => assumed.to_lowercase(),
    };
    if name.is_empty() {
        return 0;
    }
    let lowered: Vec<String> = source_order.iter().map(|s| s.to_lowercase()).collect();
    match lowered.iter().position(|s| *s == name) {
        Some(index) => (lowered.len() - index) as i64,
        None => 0,
    }
}

/// Score a resolution. "highest" rewards bigger numbers; a target like "1080" rewards
/// closeness to that target (so 4K loses to the sweet spot).
fn resolution_score(res: i64, preferred_resolution: &str) -> i64 {
    if preferred_resolution != "highest"
        && let Ok(target) = preferred_resolution
            .trim()
            .trim_end_matches(['p', 'P'])
            .parse::<i64>()
    {
        return -(res - target).abs();
    }
    res
}

/// Parse a size number that may use '.' or ',' as its decimal mark, and ',' as a
/// thousands separator ("1,024" is 1024.0, "6,2" is 6.2).
fn to_float(raw: &str) -> Option<f64> {
    let text = raw.trim();
    let text = if text.contains('.') && text.contains(',') {
        text.replace(',', "")
    } else if text.contains(',') {
        let (head, tail) = text.rsplit_once(',').unwrap_or(("", text));
        let joiner = if tail.chars().count() == 3 { "" } else { "." };
        format!("{}{joiner}{tail}", head.replace(',', ""))
    } else {
        text.to_string()
    };
    text.parse::<f64>().ok()
}

/// A feed's size string ("6.2 GiB", "301.4 MiB") in MiB, or None if unreadable.
pub fn size_mib(size: &str) -> Option<f64> {
    let m = SIZE_RE.captures(size)?;
    let value = to_float(&m[1])?;
    let unit = m
        .get(2)
        .map(|u| u.as_str().to_lowercase())
        .unwrap_or_default();
    Some(value * unit_mib(&unit))
}

/// How many times smaller this title's codec can be than x264 for equal quality. 1.0
/// when no codec is tagged.
pub fn codec_efficiency(title: &str) -> f64 {
    CODEC_EFFICIENCY
        .iter()
        .find(|(_, _, pattern)| pattern.is_match(title))
        .map(|(_, factor, _)| *factor)
        .unwrap_or(DEFAULT_CODEC_EFFICIENCY)
}

/// How many episodes the title says it contains: the widest declared range ("01-12" is
/// 12, "07~12" is 6), 1 for a single-episode marker, else None.
///
/// None means the title does not say, which is the common case for season batches ("S01",
/// "(Batch)"); `consensus_episode_count` fills those in.
pub fn episode_span(title: &str) -> Option<i64> {
    let widest = EP_RANGE_RE
        .captures_iter(title)
        .filter_map(|c| c.ok())
        .filter_map(|c| {
            let low = c[1].parse::<i64>().ok()?;
            let high = c[2].parse::<i64>().ok()?;
            (0 <= low && low < high).then_some(high - low + 1)
        })
        .max();
    if widest.is_some() {
        return widest;
    }
    if SEASON_EP_RE.is_match(title) || EPISODE_RE.is_match(title).unwrap_or(false) {
        return Some(1);
    }
    None
}

/// The episode count the result set agrees on, or None if none declares one.
///
/// Episode count is a property of the show, not of any one release, so the batches that
/// spell out "01-12" tell us what the ones that only say "S01" contain. Most-declared
/// wins; a tie goes to the wider span, since a full season is likelier to be the shared
/// truth than a half-batch.
pub fn consensus_episode_count(items: &[Release]) -> Option<i64> {
    let spans: Vec<i64> = items
        .iter()
        .filter_map(|i| episode_span(&i.title))
        .filter(|&s| s > 1)
        .collect();
    spans
        .iter()
        .copied()
        .max_by_key(|&s| (spans.iter().filter(|&&x| x == s).count(), s))
}

/// The release's MiB per episode, or None when size or episode count is unknown.
///
/// The title's own declared span wins; `fallback_episodes` (the result set's consensus)
/// is used only for titles that look like season batches.
pub fn per_episode_mib(item: &Release, fallback_episodes: Option<i64>) -> Option<f64> {
    let total = size_mib(&item.size)?;
    let mut episodes = episode_span(&item.title);
    if episodes.is_none() && BATCH_RE.is_match(&item.title) {
        episodes = fallback_episodes;
    }
    let episodes = episodes.filter(|&e| e != 0)?;
    Some(total / episodes as f64)
}

/// The per-episode MiB a release at this resolution should carry, scaled off the config's
/// 1080p baseline.
fn size_target_mib(res: i64, cfg: &BestfitConfig) -> f64 {
    let base = cfg.target_mib_per_episode;
    if res <= 0 {
        return base;
    }
    base * (res as f64 / TARGET_BASE_RES).powf(RES_TARGET_EXPONENT)
}

/// How the release's per-episode size compares to the target for its resolution,
/// normalised for codec efficiency. 1.0 is on target, 0.5 is half the bits it wanted.
/// None when it cannot be determined.
pub fn size_ratio(
    item: &Release,
    cfg: &BestfitConfig,
    fallback_episodes: Option<i64>,
) -> Option<f64> {
    let per_ep = per_episode_mib(item, fallback_episodes)?;
    let target = size_target_mib(resolution_of(&item.title), cfg);
    if target <= 0.0 {
        return None;
    }
    Some((per_ep * codec_efficiency(&item.title)) / target)
}

/// Adequacy band of the release's bitrate budget.
///
/// Past `max_size_ratio` the extra bits stop buying visible quality, so remuxes and other
/// giants drop back to the unknown-size rank rather than winning on bulk alone: this
/// picks the best fit, not the biggest file.
fn size_rank(item: &Release, cfg: &BestfitConfig, fallback_episodes: Option<i64>) -> i64 {
    let Some(ratio) = size_ratio(item, cfg, fallback_episodes) else {
        return UNKNOWN_SIZE_RANK;
    };
    if ratio > cfg.max_size_ratio {
        return UNKNOWN_SIZE_RANK;
    }
    for (minimum, rank) in SIZE_BANDS {
        if ratio >= minimum {
            return rank;
        }
    }
    SIZE_BANDS[SIZE_BANDS.len() - 1].1
}

/// Comparable ranking tuple; higher sorts as better. See the module docs.
///
/// `fallback_episodes` is the result set's consensus episode count, which `best_item`
/// derives; without it, season batches that do not spell out their episode range simply
/// score as unknown-size.
pub fn score(item: &Release, cfg: &BestfitConfig, fallback_episodes: Option<i64>) -> Score {
    let title = &item.title;
    let grank = group_rank(title, &cfg.preferred_groups);
    (
        i64::from(grank > 0),                                       // trusted-group gate
        size_rank(item, cfg, fallback_episodes),                    // enough bits?
        source_rank(title, &cfg.source_order, &cfg.assumed_source), // BluRay > WEB-DL
        resolution_score(resolution_of(title), &cfg.preferred_resolution),
        i64::from(has_subs(title)),
        audio_rank(title),         // Dual > Multi
        grank,                     // which trusted group
        item.seeders.unwrap_or(0), // unknown is 0 here
        item.downloads.unwrap_or(0),
    )
}

/// The single highest-ranked release, or None for an empty list. On a tie the first of
/// the tied releases wins, as Python's `max` does.
pub fn best_item<'a>(items: &'a [Release], cfg: &BestfitConfig) -> Option<&'a Release> {
    let episodes = consensus_episode_count(items);
    let mut best: Option<(&Release, Score)> = None;
    for item in items {
        let s = score(item, cfg, episodes);
        if best.as_ref().is_none_or(|(_, top)| s > *top) {
            best = Some((item, s));
        }
    }
    best.map(|(item, _)| item)
}

/// The facets `best_fit_query` would pin (poster, season, resolution, source, audio) as
/// the literals that actually appear in the title.
pub fn profile_parts(item: &Release) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(poster) = poster_of(&item.title) {
        parts.push(poster);
    }
    if let Some(season) = season_of(&item.title) {
        parts.push(season);
    }
    let res = resolution_of(&item.title);
    if res != 0 {
        parts.push(format!("{res}p"));
    }
    if let Some((_, literal)) = source_of(&item.title) {
        parts.push(literal);
    }
    if let Some((_, literal)) = audio_of(&item.title) {
        parts.push(literal);
    }
    parts
}

/// Does a title carry every pinned facet? Case-insensitive substrings; the poster part
/// matches literally like the others.
pub fn matches_profile(title: &str, parts: &[String]) -> bool {
    let lowered = title.to_lowercase();
    parts
        .iter()
        .all(|part| lowered.contains(&part.to_lowercase()))
}

/// The real show name from a release title, with search-noise punctuation removed and
/// whitespace collapsed ("Heroine? Seijo? Iie, ... desu (Hokori)!" becomes "Heroine Seijo
/// Iie ... desu (Hokori)").
pub fn clean_show_name(title: &str) -> String {
    let name = TITLE_NOISE_RE
        .replace_all(&show_name(title), " ")
        .into_owned();
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Rebuild a clean nyaa query from the best-matched release: the group, the real show
/// name pulled from its title, then season, resolution and source.
///
/// This replaces whatever (often hand-truncated) terms the user searched with the
/// canonical title the result actually has, so a search narrowed down to "... Maid des"
/// becomes "... Maid desu (Hokori)". A SxxEyy marker keeps only its season half (pinning
/// the episode would drop every other episode from the refetch). The audio variant is
/// pinned: groups that post a DUAL and a MULTi copy of every episode would otherwise come
/// back as duplicate pairs. Subtitles are left out so the refetch is not over-narrowed.
pub fn best_fit_query(item: &Release) -> String {
    let mut parts = Vec::new();
    if let Some(poster) = poster_of(&item.title) {
        parts.push(poster);
    }
    let name = clean_show_name(&item.title);
    if !name.is_empty() {
        parts.push(name);
    }
    if let Some(season) = season_of(&item.title) {
        parts.push(season);
    }
    let res = resolution_of(&item.title);
    if res != 0 {
        parts.push(format!("{res}p"));
    }
    if let Some((_, literal)) = source_of(&item.title) {
        parts.push(literal);
    }
    if let Some((_, literal)) = audio_of(&item.title) {
        parts.push(literal);
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(title: &str) -> Release {
        Release {
            title: title.to_string(),
            link: "l".to_string(),
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
        }
    }

    fn sized(title: &str, size: &str) -> Release {
        Release {
            size: size.to_string(),
            ..item(title)
        }
    }

    fn cfg() -> BestfitConfig {
        BestfitConfig::default()
    }

    #[test]
    fn best_item_of_nothing_is_none() {
        assert!(best_item(&[], &cfg()).is_none());
        assert!(
            best_item(
                &[item("[SubsPlease] Frieren - 01 (1080p) [ABCD0001].mkv")],
                &cfg()
            )
            .is_some()
        );
    }

    #[test]
    fn best_item_ranks_downloads_and_keeps_the_first_of_a_tie() {
        let a = Release {
            downloads: Some(1),
            ..item("a")
        };
        let b = Release {
            downloads: Some(50),
            ..item("b")
        };
        let c = item("c");
        let items = [a, b, c];
        assert_eq!(best_item(&items, &cfg()).unwrap().title, "b");
        let tied = [item("x"), item("y")];
        assert_eq!(best_item(&tied, &cfg()).unwrap().title, "x");
    }

    #[test]
    fn source_detection_order_and_literal() {
        assert_eq!(
            source_of("Show [WEB-DL 1080p]"),
            Some(("WEB-DL", "WEB-DL".into()))
        );
        assert_eq!(source_of("Show WEBDL"), Some(("WEB-DL", "WEBDL".into())));
        assert_eq!(
            source_of("Show WEB Rip"),
            Some(("WEBRip", "WEB Rip".into()))
        );
        assert_eq!(
            source_of("Show Blu-Ray"),
            Some(("BluRay", "Blu-Ray".into()))
        );
        assert_eq!(source_of("Show BDRip"), Some(("BluRay", "BDRip".into())));
        assert_eq!(source_of("Show BD 1080p"), Some(("BluRay", "BD".into())));
        assert_eq!(source_of("Show [WEB]"), Some(("WEB", "WEB".into())));
        assert_eq!(source_of("Show HDTV"), Some(("HDTV", "HDTV".into())));
        assert_eq!(source_of("Show - 01 [1080p]"), None);
        // A bare BD inside a word is not a source.
        assert_eq!(source_of("Show ABD 1080p"), None);
    }

    #[test]
    fn audio_detection_does_not_swallow_multi_subs() {
        assert_eq!(
            audio_of("Show [Dual-Audio]"),
            Some(("Dual", "Dual-Audio".into()))
        );
        assert_eq!(audio_of("Show DUAL"), Some(("Dual", "DUAL".into())));
        assert_eq!(
            audio_of("Show Multi Audio"),
            Some(("Multi", "Multi Audio".into()))
        );
        assert_eq!(audio_of("Show MULTi"), Some(("Multi", "MULTi".into())));
        assert_eq!(audio_of("Show [Multi-Subs]"), None);
        assert_eq!(audio_of("Show Multi Subs"), None);
        assert_eq!(audio_rank("Show Dual"), 2);
        assert_eq!(audio_rank("Show Multi"), 1);
        assert_eq!(audio_rank("Show"), 0);
    }

    #[test]
    fn resolution_takes_the_highest() {
        assert_eq!(resolution_of("Show [720p] [1080p]"), 1080);
        assert_eq!(resolution_of("Show x264"), 0);
        assert_eq!(resolution_of("Show 2160P"), 2160);
    }

    #[test]
    fn subs_detection() {
        assert!(has_subs("Show [Multi-Subs]"));
        assert!(has_subs("Show MultiSub"));
        assert!(has_subs("Show Multiple Subtitle"));
        assert!(!has_subs("Show [Dual]"));
    }

    #[test]
    fn group_rank_follows_the_preferred_order() {
        let preferred = vec!["SubsPlease".to_string(), "Erai-raws".to_string()];
        assert_eq!(group_rank("[SubsPlease] x", &preferred), 2);
        assert_eq!(group_rank("[erai-raws] x", &preferred), 1);
        assert_eq!(group_rank("[Judas] x", &preferred), 0);
        assert_eq!(group_rank("no poster", &preferred), 0);
    }

    #[test]
    fn source_rank_uses_the_assumed_source_for_untagged_titles() {
        let order = cfg().source_order;
        assert_eq!(source_rank("Show BluRay", &order, "WEB"), 5);
        assert_eq!(source_rank("Show", &order, "WEB"), 3);
        assert_eq!(source_rank("Show", &order, ""), 0);
        assert_eq!(source_rank("Show", &order, "LaserDisc"), 0);
    }

    #[test]
    fn resolution_score_modes() {
        assert_eq!(resolution_score(2160, "highest"), 2160);
        assert_eq!(resolution_score(2160, "1080"), -1080);
        assert_eq!(resolution_score(1080, "1080p"), 0);
        assert_eq!(resolution_score(720, "1080P"), -360);
        assert_eq!(resolution_score(720, "bogus"), 720);
    }

    #[test]
    fn to_float_decimal_and_thousands_marks() {
        assert_eq!(to_float("1,024"), Some(1024.0));
        assert_eq!(to_float("6,2"), Some(6.2));
        assert_eq!(to_float("1,024.5"), Some(1024.5));
        assert_eq!(to_float("6.2"), Some(6.2));
        assert_eq!(to_float("1,234,567"), Some(1234567.0));
        assert_eq!(to_float("1.2.3"), None);
    }

    #[test]
    fn size_mib_units() {
        assert_eq!(size_mib("6.2 GiB"), Some(6.2 * 1024.0));
        assert_eq!(size_mib("301.4 MiB"), Some(301.4));
        assert_eq!(size_mib("1.36GB"), Some(1.36 * 1024.0));
        assert_eq!(size_mib("2 TiB"), Some(2.0 * 1024.0 * 1024.0));
        assert_eq!(size_mib("512 KiB"), Some(0.5));
        assert_eq!(size_mib("1048576 B"), Some(1.0));
        assert_eq!(size_mib("1,024 MiB"), Some(1024.0));
        assert_eq!(size_mib(""), None);
        assert_eq!(size_mib("big"), None);
    }

    #[test]
    fn codec_efficiency_by_tag() {
        assert_eq!(codec_efficiency("Show AV1"), 1.8);
        assert_eq!(codec_efficiency("Show HEVC"), 1.5);
        assert_eq!(codec_efficiency("Show x265"), 1.5);
        assert_eq!(codec_efficiency("Show H.265"), 1.5);
        assert_eq!(codec_efficiency("Show x 265"), 1.5);
        assert_eq!(codec_efficiency("Show x264"), 1.0);
        assert_eq!(codec_efficiency("Show"), 1.0);
    }

    #[test]
    fn episode_span_ranges_and_single_markers() {
        assert_eq!(episode_span("Show (01-12) [1080p]"), Some(12));
        assert_eq!(episode_span("Show 07~12"), Some(6));
        assert_eq!(episode_span("Show S01E01-E12"), Some(12));
        assert_eq!(episode_span("Show - 05 [1080p]"), Some(1));
        assert_eq!(episode_span("Show S02E05"), Some(1));
        assert_eq!(episode_span("Show S01 (Batch)"), None);
        assert_eq!(episode_span("Show Movie"), None);
        // Date and version neighbours are not spans.
        assert_eq!(episode_span("Show 2021-07-12"), None);
        assert_eq!(episode_span("Show x264-2"), None);
        // The widest range wins; a reversed or equal one does not count.
        assert_eq!(episode_span("Show 01-12 and 13-14"), Some(12));
        assert_eq!(episode_span("Show 12-01"), None);
        assert_eq!(episode_span("Show 05-05"), None);
    }

    #[test]
    fn consensus_takes_the_most_declared_then_the_wider_span() {
        let items = [
            item("Show (01-12)"),
            item("Show (01-12) [x]"),
            item("Show (01-24)"),
            item("Show S01"),
            item("Show - 03"),
        ];
        assert_eq!(consensus_episode_count(&items), Some(12));
        let tie = [item("Show (01-12)"), item("Show (01-24)")];
        assert_eq!(consensus_episode_count(&tie), Some(24));
        assert_eq!(
            consensus_episode_count(&[item("Show - 03"), item("Show")]),
            None
        );
        assert_eq!(consensus_episode_count(&[]), None);
    }

    #[test]
    fn per_episode_size_borrows_the_consensus_only_for_batches() {
        let own = sized("Show (01-12) [1080p]", "12 GiB");
        assert_eq!(per_episode_mib(&own, Some(24)), Some(1024.0));
        let batch = sized("Show S01 (Batch) [1080p]", "12 GiB");
        assert_eq!(per_episode_mib(&batch, Some(12)), Some(1024.0));
        assert_eq!(per_episode_mib(&batch, None), None);
        // A countless title that is not a batch (a film) is never divided.
        let film = sized("Some Film [1080p]", "4 GiB");
        assert_eq!(per_episode_mib(&film, Some(12)), None);
        assert_eq!(per_episode_mib(&item("Show - 01"), None), None);
    }

    #[test]
    fn size_ratio_normalises_for_codec_and_resolution() {
        let c = cfg();
        let on_target = sized("Show - 01 [1080p x264]", "1000 MiB");
        assert!((size_ratio(&on_target, &c, None).unwrap() - 1.0).abs() < 1e-9);
        let hevc = sized("Show - 01 [1080p HEVC]", "1000 MiB");
        assert!((size_ratio(&hevc, &c, None).unwrap() - 1.5).abs() < 1e-9);
        let p720 = sized("Show - 01 [720p x264]", "540 MiB");
        let want = 540.0 / (1000.0 * (720.0f64 / 1080.0).powf(1.5));
        assert!((size_ratio(&p720, &c, None).unwrap() - want).abs() < 1e-9);
        // No resolution: the plain 1080p target.
        let bare = sized("Show - 01", "500 MiB");
        assert!((size_ratio(&bare, &c, None).unwrap() - 0.5).abs() < 1e-9);
        assert_eq!(size_ratio(&item("Show - 01"), &c, None), None);
        let zero = BestfitConfig {
            target_mib_per_episode: 0.0,
            ..cfg()
        };
        assert_eq!(size_ratio(&on_target, &zero, None), None);
    }

    #[test]
    fn size_rank_bands_and_the_giant_cap() {
        let c = cfg();
        let rank = |size: &str| size_rank(&sized("Show - 01 [1080p x264]", size), &c, None);
        assert_eq!(rank("1000 MiB"), 1);
        assert_eq!(rank("850 MiB"), 1);
        assert_eq!(rank("849 MiB"), -1);
        assert_eq!(rank("600 MiB"), -1);
        assert_eq!(rank("599 MiB"), -2);
        assert_eq!(rank("3000 MiB"), 1);
        assert_eq!(rank("3001 MiB"), 0, "past max_size_ratio drops to unknown");
        assert_eq!(size_rank(&item("Show - 01"), &c, None), 0);
    }

    #[test]
    fn score_orders_trust_then_size_then_source() {
        let c = BestfitConfig {
            preferred_groups: vec!["Erai-raws".into()],
            ..cfg()
        };
        let trusted = item("[Erai-raws] Show - 01 [720p]");
        let other = sized("[Judas] Show - 01 [1080p BluRay]", "1000 MiB");
        assert!(score(&trusted, &c, None) > score(&other, &c, None));
        // Size outranks source: a WEB-DL with enough bits beats a starved BluRay.
        let web = sized("[A] Show - 01 [1080p WEB-DL x264]", "1000 MiB");
        let bd = sized("[B] Show - 01 [1080p BluRay x264]", "400 MiB");
        assert!(
            score(&web, &BestfitConfig::default(), None)
                > score(&bd, &BestfitConfig::default(), None)
        );
        let s = score(&web, &BestfitConfig::default(), None);
        assert_eq!(s, (0, 1, 4, 1080, 0, 0, 0, 0, 0));
    }

    #[test]
    fn score_counts_seeders_and_downloads_last() {
        let a = Release {
            seeders: Some(5),
            downloads: Some(1),
            ..item("[A] Show - 01")
        };
        let b = Release {
            seeders: Some(5),
            downloads: Some(9),
            ..item("[A] Show - 01")
        };
        assert!(score(&b, &cfg(), None) > score(&a, &cfg(), None));
    }

    #[test]
    fn best_item_uses_the_consensus_for_season_batches() {
        // The batch spells out no span; the consensus (12) makes it adequate. The other
        // names its range but is starved.
        let batch = sized("[A] Show S01 (Batch) [1080p x264]", "12 GiB");
        let ranged = sized("[B] Show (01-12) [1080p x264]", "4 GiB");
        let items = [ranged, batch];
        assert_eq!(
            best_item(&items, &cfg()).unwrap().title,
            "[A] Show S01 (Batch) [1080p x264]"
        );
    }

    #[test]
    fn profile_parts_are_the_literals_in_the_title() {
        let it = item("[Erai-raws] Show S02E05 [1080p][WEB-DL][Dual-Audio]");
        assert_eq!(
            profile_parts(&it),
            vec!["[Erai-raws]", "S02", "1080p", "WEB-DL", "Dual-Audio"]
        );
        assert!(profile_parts(&item("Show")).is_empty());
    }

    #[test]
    fn matches_profile_is_case_insensitive_substrings() {
        let parts = vec!["[Erai-raws]".to_string(), "1080p".to_string()];
        assert!(matches_profile("[erai-raws] Show [1080P]", &parts));
        assert!(!matches_profile("[Other] Show [1080p]", &parts));
        assert!(matches_profile("anything", &[]));
    }

    #[test]
    fn clean_show_name_drops_search_noise() {
        assert_eq!(
            clean_show_name("[Grp] Heroine? Seijo? Iie, Kaettekudasai desu (Hokori)! - 01 [1080p]"),
            "Heroine Seijo Iie Kaettekudasai desu (Hokori)"
        );
        assert_eq!(clean_show_name("[Grp] Re:Zero - 03 [1080p]"), "Re:Zero");
    }

    #[test]
    fn best_fit_query_pins_group_name_season_resolution_source_audio() {
        let it = item("[Erai-raws] Frieren S01E02 [1080p][WEB-DL][Dual-Audio].mkv");
        assert_eq!(
            best_fit_query(&it),
            "[Erai-raws] Frieren S01 1080p WEB-DL Dual-Audio"
        );
        let plain = item("[SubsPlease] Frieren - 01 (1080p) [ABCD0001].mkv");
        assert_eq!(best_fit_query(&plain), "[SubsPlease] Frieren 1080p");
        assert_eq!(best_fit_query(&item("")), "");
    }
}
