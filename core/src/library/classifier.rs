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
/// so "Operations" and "Edge" never fire. A variant letter only follows a
/// number, so "Spy" is no `SP` plus a "y" (#426); a plural is spelled out.
/// A release revision at the end of a token is cut off before these run, as
/// for an episode, so `SP01v2`, `NCOP1v2` and `SPv2` read as their extra (#442).
/// Each pattern says whether its marker is also a plain word a series title can hold, a
/// preview or a special with no number (Trailer Park Boys, Special A); see `find_extra_token`.
static EXTRA_PATTERNS: LazyLock<Vec<(ExtraKind, Regex, bool)>> = LazyLock::new(|| {
    vec![
        (
            ExtraKind::Op,
            Regex::new(r"(?i)^(?:NCOP|OP)([0-9]+)([a-z])?$").unwrap(),
            false,
        ),
        (
            ExtraKind::Ed,
            Regex::new(r"(?i)^(?:NCED|ED)([0-9]+)([a-z])?$").unwrap(),
            false,
        ),
        (
            ExtraKind::Pv,
            Regex::new(r"(?i)^(?:PVs?|Trailers?|Teasers?)(?:([0-9]+)([a-z])?)?$").unwrap(),
            true,
        ),
        (
            ExtraKind::Sp,
            Regex::new(r"(?i)^(?:SPs?|Specials?)(?:([0-9]+)([a-z])?)?$").unwrap(),
            true,
        ),
        // An OVA is a special. A number glued on is its index; one apart
        // (`OVA 2`) is read by `index_apart`, as with `SP 2`.
        (
            ExtraKind::Sp,
            Regex::new(r"(?i)^(?:OVA|OAD|OAV)([0-9]+)?$").unwrap(),
            false,
        ),
        // A season's extra in an `SxxEyy` slot: `S05EEX`, `S01ESP2`.
        (
            ExtraKind::Sp,
            Regex::new(r"(?i)^S[0-9]+E(?:EX|SP)([0-9]+)?$").unwrap(),
            false,
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
        // A trailing `v2` is the release's revision, not a number: `- 17v2`
        // is episode 17.
        Regex::new(r"(?i)\s-\s*([0-9]+)(?:v[0-9]+)?(?:\s|$)").unwrap(),
        Regex::new(r"(?i)\s([0-9]{1,3})(?:v[0-9]+)?(?:\s|$)").unwrap(),
    ]
});
/// `01x36`, the `<season>x<episode>` form Sonarr and many scene releases
/// write. Tried after the patterns above and ahead of the fallback, which
/// would otherwise take the last number in the name, a title's included:
/// `Death Note - 01x36 - 1.28` is episode 36, not 28. A separator has to
/// come first, so a series name that opens the file (`3x3.Eyes.01`) is
/// never read as one, and two digits at most before the `x` keep a
/// resolution (`1920x1080`) out.
static SEASON_X_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)[\s._-]([0-9]{1,2})x([0-9]{1,3})\b").unwrap());
/// `- 6.5`, a recap or half episode after the dash, which the title anchor
/// already takes whole. Tried after `01x36`, so a title of `1.28` after one
/// never stands in for the episode, and ahead of the fallback, which would
/// otherwise take the `5`.
static DASH_DECIMAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\s-\s*([0-9]+\.[0-9]+)(?:v[0-9]+)?(?:\s|$)").unwrap());
static ANY_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[0-9]+").unwrap());
/// A revision suffix glued to a number (`07v2`), dropped before the
/// fallback counts numbers so the `2` never reads as the episode.
static REVISION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)([0-9])v[0-9]+\b").unwrap());

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

/// A bare number of up to three digits, a release revision allowed (`2v2`).
/// A year has four, so `Special 2019` keeps no index.
static BARE_INDEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^([0-9]{1,3})(?:v[0-9]+)?$").unwrap());

/// A season and episode code standing in for the number: `S01E03`, its
/// episode the index.
static CODE_INDEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^S[0-9]{1,2}E([0-9]{1,3})(?:v[0-9]+)?$").unwrap());

/// The index of a special or a preview whose number stands apart from its
/// token: `OVA 1`, `SP 2`, `Special - 03`, `PV 2`, and `OVA - S01E03`, where
/// a folder of OVAs numbers them as episodes of a season. One lone dash may
/// sit between the two (#167).
fn index_apart(rest: &[&str]) -> Option<u32> {
    let next = match rest {
        ["-", after, ..] => after,
        [next, ..] => next,
        [] => return None,
    };
    BARE_INDEX
        .captures(next)
        .or_else(|| CODE_INDEX.captures(next))?
        .get(1)?
        .as_str()
        .parse()
        .ok()
}

/// A release revision ending a token, `v2` in `SP01v2`, `OP1av2` or `S05EEXv2`.
/// A bare `v` stays, since `OP1v` is opening 1, variant v. Only tried when the
/// token as written is no extra.
static TOKEN_REVISION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(.+?)v[0-9]+$").unwrap());

/// A token that starts an episode reading: `S02E03`, `E07`, `01x36`, or `Episode` or `Ep`
/// with its number in the next token. A dash is not one; `title_end` reads it apart.
static EPISODE_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:S[0-9]+E[0-9]+(?:v[0-9]+)?|E[0-9]{2,}|[0-9]{1,2}x[0-9]{1,3})$").unwrap()
});
static EPISODE_WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(?:Episode|Ep)$").unwrap());

/// Where the series title ends among the tokens, and where the episode reading starts,
/// when one does. The title ends at the first lone dash, else at the reading, else at
/// the end. The reading is the first lone dash with an episode number or code right
/// after it, which skips the dashes of a series title (`Sword Art Online - Alicization -
/// 05`); else the first episode token, or a bare number after the first word and before
/// any dash (`Show 07 Title`) (#497).
fn title_end(tokens: &[&str]) -> (usize, Option<usize>) {
    let starts_episode = |at: usize| {
        let Some(token) = tokens.get(at) else {
            return false;
        };
        EPISODE_TOKEN.is_match(token)
            || (EPISODE_WORD.is_match(token)
                && tokens.get(at + 1).is_some_and(|n| BARE_INDEX.is_match(n)))
    };
    let dashes = || (0..tokens.len()).filter(|&at| tokens[at] == "-");
    let first_dash = dashes().next();
    let before_dash = first_dash.unwrap_or(tokens.len());
    let reading = dashes()
        .find(|&dash| {
            tokens.get(dash + 1).is_some_and(|n| BARE_INDEX.is_match(n)) || starts_episode(dash + 1)
        })
        .or_else(|| {
            (0..tokens.len()).find(|&at| {
                starts_episode(at)
                    || (at > 0 && at < before_dash && BARE_INDEX.is_match(tokens[at]))
            })
        });
    (first_dash.or(reading).unwrap_or(tokens.len()), reading)
}

/// Whether the token at `at` is a word of an episode's title: one with a letter in it
/// that is no dash, release tag, number, end word or marker of its own. `Special
/// Delivery` is a title; `Special 1080p`, `Special - Recap` and `Special Trailer` are not.
fn title_word_at(tokens: &[&str], at: usize) -> bool {
    let Some(token) = tokens.get(at) else {
        return false;
    };
    token.chars().any(char::is_alphabetic)
        && !RELEASE_TOKEN.is_match(token)
        && !NOT_A_TITLE.is_match(token)
        && !EXTRA_PATTERNS.iter().any(|(_, re, _)| re.is_match(token))
        && !OTHER_TOKENS.contains(&token.to_lowercase().as_str())
}

fn find_extra_token(file_name: &str) -> Option<ExtraHit> {
    let normalised = normalise_for_tokens(file_name);
    let tokens: Vec<&str> = normalised.split_whitespace().collect();
    let (title_end, reading) = title_end(&tokens);
    // A bare preview or special marker inside the series title, with another title word
    // after it, may be a word of the title (Trailer Park Boys, Special A). It loses to an
    // episode reading after the title and to any later marker, and is the reading only
    // when neither comes (#443). One after the episode reading, with another word after
    // it, is a word of the episode's title (`S01E05 - Special Delivery`) and is no
    // reading at all (#466).
    let mut title_word = None;
    for (at, token) in tokens.iter().enumerate() {
        // The token as written first, so `PV01` is never `P` plus a `V01`.
        let stem = TOKEN_REVISION
            .captures(token)
            .and_then(|m| m.get(1))
            .map(|m| m.as_str());
        for (kind, re, word) in EXTRA_PATTERNS.iter() {
            if let Some(m) = re.captures(token).or_else(|| re.captures(stem?)) {
                let glued = m.get(1).and_then(|g| g.as_str().parse().ok());
                let apart = || match kind {
                    ExtraKind::Sp | ExtraKind::Pv => index_apart(&tokens[at + 1..]),
                    _ => None,
                };
                let hit = ExtraHit {
                    kind: *kind,
                    index: glued.or_else(apart),
                    variant: m.get(2).map(|g| g.as_str().to_lowercase()),
                    raw_label: token.to_string(),
                };
                let bare = *word && hit.index.is_none();
                if bare && reading.is_some_and(|r| at > r) && title_word_at(&tokens, at + 1) {
                    break;
                }
                let in_title = bare && at + 1 < title_end;
                if !in_title {
                    return Some(hit);
                }
                title_word.get_or_insert(hit);
                break;
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
    title_word.filter(|_| reading.is_none())
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
    if let Some(m) = SEASON_X_EPISODE.captures(&base) {
        return finalize(m[1].parse().ok(), m[2].parse().unwrap_or(1.0));
    }
    if let Some(m) = DASH_DECIMAL.captures(&base) {
        return finalize(None, m[1].parse().unwrap_or(1.0));
    }
    let unrevised = REVISION.replace_all(&base, "$1");
    let survivors: Vec<u64> = ANY_NUMBER
        .find_iter(&unrevised)
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
        None => number_key(c.season, c.number),
        Some(_) => file_name.to_string(),
    }
}

/// An episode's history key. Season 1, and a file naming no season, key
/// by the bare number (`12`, `12.5`), which is what every key was before
/// seasons were part of it, so old history rows keep matching their files.
/// Any other season is prefixed the way the code reads: `S02E01`,
/// `S02E12.5`, `S00E03` for a special.
pub fn number_key(season: Option<u32>, number: f64) -> String {
    match season {
        None | Some(1) => format_number(number),
        Some(s) => format!("S{s:02}E{}", pad_number(number)),
    }
}

/// `01` or `12.5`: an integer padded to two digits, a decimal as is.
fn pad_number(number: f64) -> String {
    if number.fract() == 0.0 {
        format!("{:02}", number as i64)
    } else {
        format_number(number)
    }
}

static SEASON_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^S([0-9]+)E([0-9]+(?:\.[0-9]+)?)$").unwrap());

/// The season and number an episode key names, None for an extra's file
/// name. A bare number is season 1's.
pub fn parse_episode_key(key: &str) -> Option<(u32, f64)> {
    if let Some(m) = SEASON_KEY.captures(key) {
        return Some((m[1].parse().ok()?, m[2].parse().ok()?));
    }
    key.parse::<f64>().ok().map(|n| (1, n))
}

/// The title of last resort: bracket-stripped, a trailing `-` or `_` removed.
pub fn clean_episode_title(file_name: &str) -> String {
    let stripped = strip_brackets_and_ext(file_name);
    TRAILING_DASH.replace(&stripped, "").trim().to_string()
}

/// Where a release name says which episode it is. The title, when the
/// release carries one, follows it. The same tokens `extract_episode` reads,
/// with `- 12.5` and `- 05v2` taken whole so no part of the number is left
/// to read as the title.
static TITLE_ANCHOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\bS[0-9]+E[0-9]+\b|\b(?:Episode|Ep\.?)\s*[0-9]+(?:\.[0-9]+)?(?:v[0-9]+)?\b|\bE[0-9]{2,}\b|\s-\s*[0-9]+(?:\.[0-9]+)?(?:v[0-9]+)?\b|[\s._-][0-9]{1,2}x[0-9]{1,3}\b",
    )
    .unwrap()
});
static PARENS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\([^)]*\)\s*").unwrap());
/// A word that opens the release tail: resolution, source, codec, audio,
/// language or edition. The title ends where one starts. Only words no
/// title would use, so "Magnum Opus" and "The Sub Plot" survive.
static RELEASE_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(?:(?:bd|web)?[0-9]{3,4}[pi][0-9]*|[0-9]{3,4}x[0-9]{3,4}|4k|uhd|hdr(?:10)?|web(?:-?dl|-?rip)?|blu-?ray|bdrip|brrip|bdremux|remux|hdtv|hdrip|dvdrip|dvdscr|amzn|dsnp|hmax|atvp|x26[45]|h\.?26[45]|hevc|avc|av1|xvid|divx|hi10p?|10-?bit|8-?bit|aac[0-9]*|ddp?[0-9]*|e?ac3|dts(?:-?hd)?|truehd|flac|repack|eng|ita|ger|jpn|vostfr)$",
    )
    .unwrap()
});
/// What a fansub release puts where a title would go, and what a release
/// with no title repeats: `Ep 3`, `Episode 03v2`, `END`.
static NOT_A_TITLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(?:(?:Episode|Ep\.?)\s*[0-9]+(?:\.[0-9]+)?(?:v[0-9]+)?|end|final|fin|[0-9]+)$",
    )
    .unwrap()
});

static RELEASE_LEAD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\[([^\]]+)\]").unwrap());
static RELEASE_TAIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-([A-Za-z0-9]+)$").unwrap());
static RELEASE_TAIL_TAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i-u)^(?:DL|Rip|x26[45]|HEVC|AVC|AV1|\d{3,4}p|AAC|FLAC|DDP?\d?)$").unwrap()
});
static RELEASE_REVISION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i-u)(?:\b|\d)v(\d)\b").unwrap());
static RELEASE_RESOLUTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i-u)\b(?:2160|1080|720|480)p\b").unwrap());
static RELEASE_SOURCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i-u)\b(?:WEB-?DL|WEBRip|BluRay|BDRip|BD|DVD)\b").unwrap());
static RELEASE_CODEC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i-u)\b(?:x265|x264|HEVC|AVC|H\.?26[45]|AV1)\b").unwrap());

/// A path's last part, split on either separator, as the shells show it.
pub fn base_name(path: &str) -> &str {
    let trimmed = path.trim_end_matches(['/', '\\']);
    trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed)
}

/// One release told from another by its file name: the group and what it is,
/// `TBK, 1080p WEB-DL H264` or `ELiTE, 1080p x265`, else the name itself
/// (without its extension). The text is taken as written, so a release that
/// carries none of the tags still reads as its name.
pub fn release_label(path: &str) -> String {
    let base = base_name(path);
    let name = match base.rfind('.') {
        Some(i)
            if (3..=5).contains(&(base.len() - i))
                && base[i + 1..].chars().all(|c| c.is_ascii_alphanumeric()) =>
        {
            &base[..i]
        }
        _ => base,
    };
    let lead = RELEASE_LEAD.captures(name).map(|c| c[1].to_string());
    // A trailing "-DL" or "-x265" is a tag, not a group
    let tail = RELEASE_TAIL
        .captures(name)
        .map(|c| c[1].to_string())
        .filter(|t| !RELEASE_TAIL_TAG.is_match(t));
    let group = lead.or(tail).unwrap_or_default();
    let mut tags: Vec<String> = Vec::new();
    if let Some(c) = RELEASE_REVISION.captures(name) {
        tags.push(format!("v{}", &c[1]));
    }
    if let Some(m) = RELEASE_RESOLUTION.find(name) {
        tags.push(m.as_str().to_lowercase());
    }
    for re in [&*RELEASE_SOURCE, &*RELEASE_CODEC] {
        if let Some(m) = re.find(name) {
            tags.push(m.as_str().to_string());
        }
    }
    let what = tags.join(" ");
    match (group.is_empty(), what.is_empty()) {
        (false, false) => format!("{group}, {what}"),
        (false, true) => group,
        (true, false) => what,
        (true, true) => name.to_string(),
    }
}

/// The episode title a release name carries after its episode token, or
/// None. `Show.S03E04.A.Trial.of.Errors.1080p.WEB-DL.mkv` is "A Trial of
/// Errors"; `[Group] Show - 01 [1080p].mkv` carries none.
pub fn file_episode_title(file_name: &str) -> Option<String> {
    // A scene name separates with dots and underscores; a name with spaces
    // of its own keeps its dots, which belong to the title ("Mr. Smith").
    // Decided on the name as given, before the bracket and paren strips
    // put spaces of their own into it.
    let stem = Path::new(file_name).file_stem()?.to_string_lossy();
    let dotted = !stem.contains(' ');
    let base = strip_brackets_and_ext(file_name);
    let anchor = TITLE_ANCHOR.find(&base)?;
    let rest = &base[anchor.end()..];
    let rest = if dotted {
        SEPARATORS.replace_all(rest, " ").into_owned()
    } else {
        rest.replace('_', " ")
    };
    let rest = PARENS.replace_all(&rest, " ");
    // A scene group glued to the last release token (`HDTV-LOL`) is still
    // that token.
    let words: Vec<&str> = rest
        .split_whitespace()
        .take_while(|w| !RELEASE_TOKEN.is_match(w.split('-').next().unwrap_or(w)))
        .collect();
    let title = words.join(" ");
    let title = title.trim_matches(|c: char| c == '-' || c.is_whitespace());
    if title.is_empty() || NOT_A_TITLE.is_match(title) {
        return None;
    }
    Some(title.to_string())
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
    fn season_x_episode_beats_a_number_in_the_title() {
        let c = ep("Death Note - 01x36 - 1.28.mkv");
        assert_eq!((c.extra, c.number, c.season), (None, 36.0, Some(1)));
        let c = ep("Death Note - 01x30 - Justice.mkv");
        assert_eq!((c.number, c.season), (30.0, Some(1)));
        let c = ep("Show.2x05.The.Title.720p.mkv");
        assert_eq!((c.number, c.season), (5.0, Some(2)));
        // A series name that opens the file is not a season and episode.
        assert_eq!(ep("3x3 Eyes - 01.mkv").number, 1.0);
        assert_eq!(ep("3x3.Eyes.01.mkv").number, 1.0);
        // Nor is a resolution.
        assert_eq!(ep("Show 1920x1080 - 07.mkv").number, 7.0);
        // Season 1 keys by the bare number, as a name with no season does.
        let name = "Death Note - 01x36 - 1.28.mkv";
        assert_eq!(episode_key(&ep(name), name), "36");
    }

    #[test]
    fn an_ova_and_a_season_extra_are_specials() {
        let c = ep("Dorohedoro - OVA .mkv");
        assert_eq!(
            (c.extra, c.extra_index, c.season),
            (Some(ExtraKind::Sp), None, Some(0))
        );
        let c = ep("Show OVA2 [1080p].mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Sp), Some(2)));
        assert_eq!(ep("Show - OAD.mkv").extra, Some(ExtraKind::Sp));
        let c = ep("Monogatari Series S05EEX.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Sp), None));
        let c = ep("Show S01ESP2.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Sp), Some(2)));
        // A word that only starts like one is not.
        let c = ep("Show - 03 - Ovation.mkv");
        assert_eq!((c.extra, c.number), (None, 3.0));
        let c = ep("Monogatari Series - S05E05.mkv");
        assert_eq!((c.extra, c.number, c.season), (None, 5.0, Some(5)));
    }

    #[test]
    fn a_title_word_that_starts_like_a_marker_is_not_one() {
        // A variant letter only follows a number, so "Spy" is no `SP` plus
        // a "y" (#426).
        let c = ep("Spy x Family - 01.mkv");
        assert_eq!((c.extra, c.number, c.season), (None, 1.0, None));
        let c = ep("[SubsPlease] Spy x Family Season 2 - 05 (1080p).mkv");
        assert_eq!((c.extra, c.number), (None, 5.0));
        let c = ep("Spa Show - 02.mkv");
        assert_eq!((c.extra, c.number), (None, 2.0));
        // A plural is still the marker, with no variant.
        for (name, kind) in [
            ("Show Specials.mkv", ExtraKind::Sp),
            ("Show SPs.mkv", ExtraKind::Sp),
            ("Show PVs.mkv", ExtraKind::Pv),
            ("Show Trailers.mkv", ExtraKind::Pv),
            ("Show - Teasers 01.mkv", ExtraKind::Pv),
        ] {
            let c = ep(name);
            assert_eq!((c.extra, c.extra_variant), (Some(kind), None), "{name}");
        }
        assert_eq!(ep("Show - Teasers 01.mkv").extra_index, Some(1));
        // A variant glued on after a number still reads.
        let c = ep("Show SP2b.mkv");
        assert_eq!(
            (c.extra, c.extra_index, c.extra_variant.as_deref()),
            (Some(ExtraKind::Sp), Some(2), Some("b"))
        );
        let c = ep("Show PV1a.mkv");
        assert_eq!(
            (c.extra, c.extra_index, c.extra_variant.as_deref()),
            (Some(ExtraKind::Pv), Some(1), Some("a"))
        );
    }

    #[test]
    fn a_revision_after_an_extras_index_is_that_extra() {
        // `v2` after the number is the release's revision, as it is for an
        // episode, so the file is still the extra and not episode 1 (#442).
        for (name, kind, index) in [
            ("[Group] Show - SP01v2.mkv", ExtraKind::Sp, 1),
            ("[Group] Show - NCOP1v2.mkv", ExtraKind::Op, 1),
            ("[Group] Show - NCED2v3 [1080p].mkv", ExtraKind::Ed, 2),
            ("[Group] Show - PV1v2.mkv", ExtraKind::Pv, 1),
            ("[Group] Show - OVA3v2.mkv", ExtraKind::Sp, 3),
            ("[Group] Show - S01ESP2v2.mkv", ExtraKind::Sp, 2),
        ] {
            let c = ep(name);
            assert_eq!(
                (c.extra, c.extra_index, c.extra_variant),
                (Some(kind), Some(index), None),
                "{name}"
            );
        }
        // An extra with no number of its own takes a revision too.
        for (name, kind) in [
            ("[Group] Show - SPv2.mkv", ExtraKind::Sp),
            ("[Group] Show - PVv2.mkv", ExtraKind::Pv),
            ("[Group] Show - OVAv2.mkv", ExtraKind::Sp),
            ("[Group] Show - S05EEXv2.mkv", ExtraKind::Sp),
        ] {
            let c = ep(name);
            assert_eq!((c.extra, c.extra_index), (Some(kind), None), "{name}");
        }
        // A variant letter and a revision together keep both readings, in
        // either case, and a bare `v` is still a variant.
        for (name, variant) in [
            ("Show OP1av2.mkv", Some("a")),
            ("Show OP1V2.mkv", None),
            ("Show OP1v.mkv", Some("v")),
        ] {
            let c = ep(name);
            assert_eq!(
                (c.extra, c.extra_index, c.extra_variant.as_deref()),
                (Some(ExtraKind::Op), Some(1), variant),
                "{name}"
            );
        }
        // The label is still the token as written.
        assert_eq!(ep("Show - SP01v2.mkv").raw_label.as_deref(), Some("SP01v2"));
    }

    #[test]
    fn a_title_word_that_is_a_bare_marker_does_not_beat_the_episode() {
        // The title's own word is the marker, with no number of its own,
        // and an episode number follows it (#443).
        for (name, number) in [
            ("Trailer Park Boys - 01.mkv", 1.0),
            ("[Group] Special A - 07 [1080p].mkv", 7.0),
            ("Trailer.Park.Boys.S02E03.mkv", 3.0),
            ("Special A Episode 12.mkv", 12.0),
        ] {
            let c = ep(name);
            assert_eq!((c.extra, c.number), (None, number), "{name}");
        }
        // A marker later in the name still makes the file that extra.
        let c = ep("Special A - SP01.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Sp), Some(1)));
        let c = ep("Trailer Park Boys - NCOP2.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Op), Some(2)));
        // With no episode after it, the bare marker is the file's reading.
        let c = ep("Trailer Park Boys - Trailer.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Pv), None));
        let c = ep("Show - Special.mkv");
        assert_eq!((c.extra, c.number), (Some(ExtraKind::Sp), 0.0));
        // A marker with its own index keeps it, as before.
        assert_eq!(ep("Show Special - 04.mkv").extra_index, Some(4));
        assert_eq!(ep("Show - Trailer 2.mkv").extra_index, Some(2));
        // A later bare marker beats the title's own.
        let c = ep("Trailer Park Boys - Special.mkv");
        assert_eq!(c.extra, Some(ExtraKind::Sp));
        let c = ep("Special A - Trailer.mkv");
        assert_eq!(c.extra, Some(ExtraKind::Pv));
        // A marker after the title's dash is the file's, whatever follows it:
        // a year, another word and a number, or an episode word.
        for (name, kind) in [
            ("Show - Special - 2019.mkv", ExtraKind::Sp),
            ("Show - Trailer - 2024.mkv", ExtraKind::Pv),
            ("Show - Trailer Collection - 02.mkv", ExtraKind::Pv),
            ("Show - Special Ending - 01.mkv", ExtraKind::Sp),
            ("[Group] Show - SP Ep 01.mkv", ExtraKind::Sp),
            ("Show - Special - Step 1.mkv", ExtraKind::Sp),
        ] {
            assert_eq!(ep(name).extra, Some(kind), "{name}");
        }
        // A marker with no title word after it is no title word.
        assert_eq!(ep("Show Special Episode 3.mkv").extra, Some(ExtraKind::Sp));
    }

    #[test]
    fn a_bare_marker_in_the_episode_title_after_the_number_is_a_title_word() {
        // After the episode reading, a bare marker with a title word after it is a word
        // of the episode's title (#466).
        for (name, season, number) in [
            ("Show - S01E05 - Special Delivery.mkv", Some(1), 5.0),
            ("Show - 07 - Trailer Trouble.mkv", None, 7.0),
            ("[Group] Show - 12 - Special Ending [1080p].mkv", None, 12.0),
            ("Show.S02E03.Special.Delivery.mkv", Some(2), 3.0),
            ("Show Episode 4 Teaser Night.mkv", None, 4.0),
        ] {
            let c = ep(name);
            assert_eq!(
                (c.extra, c.season, c.number),
                (None, season, number),
                "{name}"
            );
        }
        // One with nothing after it stays the file's reading: the name says the episode
        // is a special, not what it is called.
        let c = ep("Show - 01 - Special.mkv");
        assert_eq!(c.extra, Some(ExtraKind::Sp));
        // A marker with its own number or glued to one is still the extra.
        let c = ep("Show - 07 - Special 2 Night.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Sp), Some(2)));
        let c = ep("Show - 07 - SP01 Delivery.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Sp), Some(1)));
        let c = ep("Show - 07 - NCOP1 Clean.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Op), Some(1)));
        // What follows has to be a title word: a release tag, a dash, an end word or
        // another marker is not one, and the file stays the extra it names.
        for (name, kind) in [
            ("Show - 01 - Special 1080p.mkv", ExtraKind::Sp),
            ("Show.S01E00.Special.1080p.WEB-DL.mkv", ExtraKind::Sp),
            ("Show - 13 - Special - Recap.mkv", ExtraKind::Sp),
            ("Show - 07 - Special Trailer.mkv", ExtraKind::Sp),
            ("Show - 07 - Special Bonus.mkv", ExtraKind::Sp),
            ("Show - 07 - Trailer END.mkv", ExtraKind::Pv),
        ] {
            assert_eq!(ep(name).extra, Some(kind), "{name}");
        }
    }

    #[test]
    fn a_bare_marker_in_the_episode_title_reads_with_no_dash_or_a_dashed_series_title() {
        // A bare number with no dash before it starts the episode reading too, and so
        // does a dash after a series title that has dashes of its own (#497).
        for (name, number) in [
            ("Show 07 Trailer Trouble.mkv", 7.0),
            ("Show.07.Special.Delivery.mkv", 7.0),
            ("[Group] Show 07 Special Delivery [1080p].mkv", 7.0),
            (
                "Sword Art Online - Alicization - 05 - Special Delivery.mkv",
                5.0,
            ),
            ("Show - Season 2 - 05 - Special Delivery.mkv", 5.0),
            ("Show S01E05 - Special Delivery.mkv", 5.0),
        ] {
            let c = ep(name);
            assert_eq!((c.extra, c.number), (None, number), "{name}");
        }
        // A number in the series title before the marker still lets the marker read, and
        // a marker with its own number is still the extra.
        assert_eq!(ep("Mob Psycho 100 Special.mkv").extra, Some(ExtraKind::Sp));
        let c = ep("Show 07 Special 2 Night.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Sp), Some(2)));
        let c = ep("Show - Alicization - 05 - Special.mkv");
        assert_eq!(c.extra, Some(ExtraKind::Sp));
        // A series title that is a marker word, numbered with no dash, is its episodes.
        let c = ep("Trailer Park Boys 03.mkv");
        assert_eq!((c.extra, c.number), (None, 3.0));
    }

    #[test]
    fn a_number_apart_from_the_token_is_its_index() {
        // The Girls und Panzer OVA Series files (#167).
        let c = ep("[ak-Submarines] Girls und Panzer - OVA 1 [BD 1080p][F37D267C].mkv");
        assert_eq!(
            (c.extra, c.extra_index, c.number, c.season),
            (Some(ExtraKind::Sp), Some(1), 1.0, Some(0))
        );
        assert_eq!(ep("Show - OVA 6 [1080p].mkv").extra_index, Some(6));
        assert_eq!(ep("Show SP 2.mkv").extra_index, Some(2));
        assert_eq!(ep("Show - Special 03 - The Title.mkv").extra_index, Some(3));
        assert_eq!(ep("Show Special - 04.mkv").extra_index, Some(4));
        assert_eq!(ep("Show OAD 2v2.mkv").extra_index, Some(2));
        assert_eq!(ep("Show PV 2.mkv").extra_index, Some(2));
        // A glued number still wins, and a year or a word is no index.
        assert_eq!(ep("Show OVA2 3.mkv").extra_index, Some(2));
        assert_eq!(ep("Show Special 2019.mkv").extra_index, None);
        assert_eq!(ep("Show OVA Recap.mkv").extra_index, None);
        // An opening needs its number glued on, as before.
        assert_eq!(ep("Show - OP 2.mkv").extra, None);
        // A season and episode code after the token: a folder of OVAs that
        // numbers them as a season's episodes (LiamPalmqvist's layout, #177).
        let c = ep("Seitokai Yakuindomo OVA - S01E03.mkv");
        assert_eq!((c.extra, c.extra_index), (Some(ExtraKind::Sp), Some(3)));
        assert_eq!(
            ep("Seitokai Yakuindomo OVA - S02E10.mkv").extra_index,
            Some(10)
        );
        assert_eq!(ep("Show OVA S01E02v2.mkv").extra_index, Some(2));
        // The series' own episodes beside them are untouched.
        let c = ep("Seitokai Yakuindomo S1 - S01E01.mkv");
        assert_eq!((c.extra, c.number, c.season), (None, 1.0, Some(1)));
    }

    #[test]
    fn a_decimal_after_the_dash_is_the_episode() {
        let c = ep("Kara no Kyoukai - 6.5 - Remix - Gate of Seventh Heaven (recap).mkv");
        assert_eq!((c.extra, c.number, c.season), (None, 6.5, None));
        assert_eq!(
            ep("[Group] Show - 12.5 - Recap Title [1080p].mkv").number,
            12.5
        );
        assert_eq!(ep("Show - 07.5v2 [1080p].mkv").number, 7.5);
        // `01x36` comes first, so the title after it stays a title.
        assert_eq!(ep("Death Note - 01x36 - 1.28.mkv").number, 36.0);
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
    fn a_revision_suffix_is_not_the_episode_number() {
        let c = ep(
            "[Erai-raws] Honzuki no Gekokujou S4 - 17v2 [1080p CR WEB-DL AVC AAC][MultiSub][BC7A5FC1].mkv",
        );
        assert_eq!((c.extra, c.number), (None, 17.0));
        assert_eq!(
            ep("[Erai-raws] Neko to Ryuu - 07v2 [1080p].mkv").number,
            7.0
        );
        assert_eq!(ep("Show 05v3.mkv").number, 5.0);
        assert_eq!(ep("Show.Name.12v2.mkv").number, 12.0);
        assert_eq!(ep("[Erai-raws] Show Name - 02 [1080p].mkv").number, 2.0);
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
        // Season 1 keys by the bare number, as every key did before
        // seasons; a later season is prefixed so S01E01 and S02E01 never
        // share a history row.
        let c = ep("Show.Name.S01E01.mkv");
        assert_eq!(episode_key(&c, "Show.Name.S01E01.mkv"), "1");
        let c = ep("Show.Name.S02E01.mkv");
        assert_eq!(episode_key(&c, "Show.Name.S02E01.mkv"), "S02E01");
        assert_eq!(number_key(Some(2), 12.5), "S02E12.5");
        assert_eq!(number_key(Some(0), 3.0), "S00E03");
        assert_eq!(number_key(None, 7.0), "7");
        assert_eq!(parse_episode_key("12"), Some((1, 12.0)));
        assert_eq!(parse_episode_key("12.5"), Some((1, 12.5)));
        assert_eq!(parse_episode_key("S02E01"), Some((2, 1.0)));
        assert_eq!(parse_episode_key("S02E12.5"), Some((2, 12.5)));
        assert_eq!(parse_episode_key("Show OP1.mkv"), None);
        assert_eq!(format_number(12.0), "12");
        assert_eq!(format_number(12.5), "12.5");
        assert_eq!(
            clean_episode_title("[Group] Show - 01 - [ABCD1234].mkv"),
            "Show - 01"
        );
        assert_eq!(clean_episode_title("Show_Name_03_.mkv"), "Show_Name_03");
        assert_eq!(strip_brackets_and_ext("[A] Show [B] 01 [C].mkv"), "Show 01");
    }

    #[test]
    fn titles_come_from_the_release_name_after_the_episode_token() {
        let t = |n: &str| file_episode_title(n);
        assert_eq!(
            t("Helluva.Boss.S03E04.A.Trial.of.Errors.1080p.AMZN.WEB-DL.ENG.ITA.DDP5.1.H264-TBK.mkv")
                .as_deref(),
            Some("A Trial of Errors")
        );
        assert_eq!(
            t("Helluva.Boss.S03E01.Turtle.Emoji.1080p.AMZN.WEB-DL.mkv").as_deref(),
            Some("Turtle Emoji")
        );
        assert_eq!(
            t("[Group] Show - 03 - The Heroine Arrives [1080p].mkv").as_deref(),
            Some("The Heroine Arrives")
        );
        assert_eq!(
            t("Show S01E02 Mr. Smith Goes (1080p).mkv").as_deref(),
            Some("Mr. Smith Goes")
        );
        assert_eq!(t("[SubsPlease] Show - 01 (1080p) [ABCD1234].mkv"), None);
        assert_eq!(t("Show.S01E01.720p.HDTV.x264.mkv"), None);
        assert_eq!(t("Show Episode 3.mkv"), None);
        assert_eq!(t("Show - 05v2.mkv"), None);
        assert_eq!(t("Show OP1.mkv"), None);
        // From the #108 review.
        assert_eq!(
            t("Show.S01E02.Some.Title.(2019).1080p.mkv").as_deref(),
            Some("Some Title")
        );
        assert_eq!(
            t("[Group] Show - 12.5 - Recap Title [1080p].mkv").as_deref(),
            Some("Recap Title")
        );
        assert_eq!(
            t("Show.S01E05.Magnum.Opus.1080p.mkv").as_deref(),
            Some("Magnum Opus")
        );
        assert_eq!(
            t("Show.S02E03.The.Sub.Plot.720p.mkv").as_deref(),
            Some("The Sub Plot")
        );
        assert_eq!(
            t("Show.S01E02.Pilot.HDTV-LOL.mkv").as_deref(),
            Some("Pilot")
        );
        assert_eq!(
            t("Show.S01E02.Title.HDRip.XviD.mkv").as_deref(),
            Some("Title")
        );
        assert_eq!(
            t("Show.S01E02.Title.BD1080p.Hi10P.mkv").as_deref(),
            Some("Title")
        );
        assert_eq!(
            t("Show Ep 03 - The Title.mkv").as_deref(),
            Some("The Title")
        );
        assert_eq!(t("Show S01E03 Ep 3.mkv"), None);
        assert_eq!(t("Show S01E03 Episode 03v2.mkv"), None);
        assert_eq!(t("[G] Show - 12 END [1080p].mkv"), None);
        assert_eq!(
            t("Death Note - 01x30 - Justice.mkv").as_deref(),
            Some("Justice")
        );
        assert_eq!(t("Death Note - 01x36 - 1.28.mkv").as_deref(), Some("1.28"));
    }

    #[test]
    fn release_labels() {
        assert_eq!(
            release_label("/x/Helluva.Boss.S03E04.1080p.AMZN.WEB-DL.ENG.ITA.DDP5.1.H264-TBK.mkv"),
            "TBK, 1080p WEB-DL H264"
        );
        assert_eq!(
            release_label("Avatar.Seven.Havens.S01E01.1080p.x265-ELiTE.mkv"),
            "ELiTE, 1080p x265"
        );
        assert_eq!(
            release_label("[Erai-raws] Show - 17v2 [1080p CR WEB-DL AVC AAC].mkv"),
            "Erai-raws, v2 1080p WEB-DL AVC"
        );
        assert_eq!(release_label("Show.S01E01.1080p-x265.mkv"), "1080p x265");
        assert_eq!(release_label("plain name.mkv"), "plain name");
        // ASCII word boundaries, as in the old QML rule: CJK letters are not word characters
        assert_eq!(release_label("番組第01話1080p.mkv"), "1080p");
    }
}
