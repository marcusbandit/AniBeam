//! Single source of truth for what kind of file a name is: a real episode,
//! an opening, ending, promo or special, or something else. Pure string in,
//! kind and numbers out. Carried from src/shared/episodeClassifier.ts.

use std::path::Path;
use std::sync::LazyLock;

use regex::{Captures, Regex, RegexSet};

use anibeam_contract::ExtraKind;

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

/// The extras token patterns, tried in this order, each anchored at both ends
/// so "Operations" and "Edge" never fire. A variant letter only follows a
/// number, so "Spy" is no `SP` plus a "y" (#426); a plural is spelled out.
/// A release revision at the end of a token is cut off before these run, as
/// for an episode, so `SP01v2`, `NCOP1v2` and `SPv2` read as their extra (#442).
/// Each pattern says whether its marker is also a plain word a series title can hold, a
/// preview or a special with no number (Trailer Park Boys, Special A); see `find_extra_token`.
/// An extras pattern: the kind it reads, its regex, and whether a bare one with no index
/// may be a word of a title.
type Pattern = (ExtraKind, Regex, bool);
/// The pattern a text reads as, with what it captured.
type Reading<'a> = (&'static Pattern, Captures<'a>);

static EXTRA_PATTERNS: LazyLock<Vec<Pattern>> = LazyLock::new(|| {
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
/// The same patterns as one set, to find in one pass which of them reads a token. Where
/// more than one could, the set lists its matches in pattern order and the first is taken,
/// the pattern the loop over `EXTRA_PATTERNS` would have reached first.
static EXTRA_SET: LazyLock<RegexSet> =
    LazyLock::new(|| RegexSet::new(EXTRA_PATTERNS.iter().map(|(_, re, _)| re.as_str())).unwrap());
const OTHER_TOKENS: [&str; 5] = ["menu", "cm", "bonus", "extra", "extras"];

/// The extras pattern a text reads as, with what it captured.
fn extra_reading(text: &str) -> Option<Reading<'_>> {
    let pattern = &EXTRA_PATTERNS[EXTRA_SET.matches(text).into_iter().next()?];
    Some((pattern, pattern.1.captures(text)?))
}

/// The extras pattern a token reads as, and the stem it was read from when only the
/// token with its release revision cut reads as one. The token as written first, so
/// `PV01` is never `P` plus a `V01`. No token a pattern reads has a stem another pattern
/// reads, so this agrees with trying each pattern on the token and then its stem in
/// turn; a test holds every pattern to that.
fn token_reading(token: &str) -> Option<(Reading<'_>, Option<&str>)> {
    extra_reading(token)
        .map(|reading| (reading, None))
        .or_else(|| {
            let stem = TOKEN_REVISION.captures(token)?.get(1)?.as_str();
            Some((extra_reading(stem)?, Some(stem)))
        })
}

/// A word that makes a file an extra of kind Other. The words are ASCII, so an ASCII
/// case fold reads them as a full lowercase would.
fn is_other(token: &str) -> bool {
    OTHER_TOKENS.iter().any(|w| w.eq_ignore_ascii_case(token))
}

/// A token that marks an extra of any kind.
fn is_marker(token: &str) -> bool {
    EXTRA_SET.is_match(token) || is_other(token)
}

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
        .unwrap_or_default()
        .to_string_lossy();
    let no_brackets = BRACKETS.replace_all(&stem, " ");
    SPACES.replace_all(&no_brackets, " ").trim().to_string()
}

/// The words of a stripped name, split at whitespace and at `_` and `.` too, so the
/// token scan splits `Bakemonogatari_ED1_...` and `Show.Name.OP1.mkv`.
fn tokens_of(base: &str) -> Vec<&str> {
    base.split(|c: char| c.is_whitespace() || c == '_' || c == '.')
        .filter(|t| !t.is_empty())
        .collect()
}

struct ExtraHit {
    kind: ExtraKind,
    index: Option<u32>,
    variant: Option<String>,
    raw_label: String,
    /// The token less its release revision, `SP01` out of `SP01v2`, when only that
    /// reads as the extra; None when the token as written does.
    unrevised: Option<String>,
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

/// A zero-padded episode number standing alone, `07` or `012`, a revision allowed. A
/// series title's own number is rarely padded (`Mob Psycho 100`, `Season 3`, a codec's
/// `264`), and `00` is no episode (`Gundam 00`).
static PADDED_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(0[0-9]{1,2})(?:v[0-9]+)?$").unwrap());
static SEASON_WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(?:season|part|cour|vol|volume)$").unwrap());

/// Where a name with no dash before its number has its episode reading: the first
/// zero-padded number after the first word and before any dash (`Show 07 Title`,
/// `Show.07.Special.Delivery`), not right after a season word, and with no marker before
/// it, so a marker in the series title keeps its own reading (`Show Trailer Collection
/// 02`) (#497). Its number, for `extract_episode`.
/// [`number_reading`] of one name's tokens, worked out the first time a reader asks: most
/// names are read by a dash or a pattern long before it would matter.
struct Number<'a> {
    tokens: &'a [&'a str],
    reading: std::cell::OnceCell<Option<(usize, u32)>>,
}

impl<'a> Number<'a> {
    fn of(tokens: &'a [&'a str]) -> Self {
        Number {
            tokens,
            reading: Default::default(),
        }
    }

    fn get(&self) -> Option<(usize, u32)> {
        *self.reading.get_or_init(|| number_reading(self.tokens))
    }
}

fn number_reading(tokens: &[&str]) -> Option<(usize, u32)> {
    let before_dash = tokens
        .iter()
        .position(|t| *t == "-")
        .unwrap_or(tokens.len());
    // A number at or before the first marker has none before it.
    let end = tokens[..before_dash]
        .iter()
        .position(|t| is_marker(t))
        .map_or(before_dash, |marker| marker + 1);
    (1..end).find_map(|at| {
        let n: u32 = PADDED_NUMBER.captures(tokens[at])?[1].parse().ok()?;
        (n > 0 && !SEASON_WORD.is_match(tokens[at - 1])).then_some((at, n))
    })
}

/// Where the series title ends among the tokens, and where the episode reading starts,
/// when one does. The title ends at the first lone dash, else at the reading, else at
/// the end. The reading is the first lone dash with an episode number or code right
/// after it, which skips the dashes of a series title (`Sword Art Online - Alicization -
/// 05`); else the first episode token or [`number_reading`], whichever comes first
/// (#497).
fn title_end(tokens: &[&str], number: &Number) -> (usize, Option<usize>) {
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
    let reading = dashes()
        .find(|&dash| {
            tokens.get(dash + 1).is_some_and(|n| BARE_INDEX.is_match(n)) || starts_episode(dash + 1)
        })
        .or_else(|| {
            let token = (0..tokens.len()).find(|&at| starts_episode(at));
            token
                .into_iter()
                .chain(number.get().map(|(at, _)| at))
                .min()
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
        && !is_marker(token)
}

/// The extra a name's tokens make it, if any.
fn find_extra_token(tokens: &[&str], number: &Number) -> Option<ExtraHit> {
    let (title_end, reading) = title_end(tokens, number);
    // A bare preview or special marker inside the series title, with another title word
    // after it, may be a word of the title (Trailer Park Boys, Special A). It loses to an
    // episode reading anywhere in the name and to any later marker, and is the reading
    // only when neither comes (#443). One after the episode reading, with another word after
    // it, is a word of the episode's title (`S01E05 - Special Delivery`) and is no
    // reading at all (#466).
    let mut title_word = None;
    for (at, token) in tokens.iter().enumerate() {
        let read = token_reading(token);
        if let Some((((kind, _, word), m), unrevised)) = read {
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
                unrevised: unrevised.map(str::to_string),
            };
            let bare = *word && hit.index.is_none();
            if bare && reading.is_some_and(|r| at > r) && title_word_at(tokens, at + 1) {
                continue;
            }
            let in_title = bare && at + 1 < title_end;
            if !in_title {
                return Some(hit);
            }
            title_word.get_or_insert(hit);
        } else if is_other(token) {
            return Some(ExtraHit {
                kind: ExtraKind::Other,
                index: None,
                variant: None,
                raw_label: token.to_string(),
                unrevised: None,
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

/// The season and episode a name with no extra in it reads as, from its stripped `base`
/// and its tokens' number reading.
fn extract_episode(base: &str, number: &Number) -> (Option<u32>, f64) {
    if let Some(m) = SEASON_EPISODE.captures(base) {
        return finalize(m[1].parse().ok(), m[2].parse().unwrap_or(1.0));
    }
    if let Some(m) = DECIMAL_EPISODE.captures(base) {
        let whole: f64 = m[1].parse().unwrap_or(0.0);
        let decimal: f64 = m[2].parse().unwrap_or(0.0);
        return finalize(None, whole + decimal / 10.0);
    }
    for re in EPISODE_PATTERNS.iter() {
        if let Some(m) = re.captures(base) {
            return finalize(None, m[1].parse().unwrap_or(1.0));
        }
    }
    if let Some(m) = SEASON_X_EPISODE.captures(base) {
        return finalize(m[1].parse().ok(), m[2].parse().unwrap_or(1.0));
    }
    if let Some(m) = DASH_DECIMAL.captures(base) {
        return finalize(None, m[1].parse().unwrap_or(1.0));
    }
    // A dotted name has no spaces for the patterns above, so its padded number is read
    // from the tokens, the way `title_end` read it, before the last number in the name
    // (a codec's `264`) can stand in.
    if let Some((_, n)) = number.get() {
        return finalize(None, n as f64);
    }
    let unrevised = REVISION.replace_all(base, "$1");
    let last = ANY_NUMBER
        .find_iter(&unrevised)
        .filter_map(|m| m.as_str().parse::<u64>().ok())
        .filter(|n| !(1900..=2099).contains(n) && *n < 1000)
        .last();
    if let Some(last) = last {
        return finalize(None, last as f64);
    }
    (None, 1.0)
}

/// What kind of file a name is, and its numbers. The name is stripped and split once,
/// for the extras scan and the episode reading both.
pub fn classify(file_name: &str) -> Classified {
    let base = strip_brackets_and_ext(file_name);
    let tokens = tokens_of(&base);
    let reading = Number::of(&tokens);
    if let Some(hit) = find_extra_token(&tokens, &reading) {
        let number = hit.index.unwrap_or(0) as f64;
        return Classified {
            extra: Some(hit.kind),
            number,
            season: (hit.kind == ExtraKind::Sp).then_some(0),
            extra_index: hit.index,
            extra_variant: hit.variant,
            raw_label: Some(hit.raw_label),
        };
    }
    let (season, number) = extract_episode(&base, &reading);
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
        Some(_) => extra_key(file_name),
    }
}

/// An extra's key: its file name less what tells one release of it from another, so
/// `SP01.mkv` and `SP01v2.mkv` are two releases of one extra with one history (#468): a
/// release revision after its marker (`SP01v2`, `OP1av2`) or its index (`OVA 01v2`), and a
/// checksum tag (`[A1B2C3D4]`), which differs between any two encodes. A name with none
/// of those is its own key, as every extra's was.
fn extra_key(file_name: &str) -> String {
    let mut name = file_name.to_string();
    let base = strip_brackets_and_ext(file_name);
    let tokens = tokens_of(&base);
    if let Some(ExtraHit {
        raw_label,
        unrevised: Some(stem),
        ..
    }) = find_extra_token(&tokens, &Number::of(&tokens))
        && let Some(at) = name.rfind(&raw_label)
    {
        name = format!("{}{stem}{}", &name[..at], &name[at + raw_label.len()..]);
    }
    let name = REVISION.replace_all(&name, "$1");
    CHECKSUM.replace_all(&name, "").into_owned()
}

/// A checksum tag in a release's name, `[A1B2C3D4]` or `(A1B2C3D4)`, with the space before.
static CHECKSUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*[\[(][0-9A-Fa-f]{8}[\])]").unwrap());

/// The history key a file of this name carries when it is an extra, for a document that
/// names the file rather than the key (an import): the name itself for anything else.
pub fn key_of_file_name(file_name: &str) -> String {
    match classify(file_name).extra {
        Some(_) => extra_key(file_name),
        None => file_name.to_string(),
    }
}

/// The folders between a series and one of its files, joined with `/`:
/// `Season 2` for `Show/Season 2/NCOP1.mkv`. Empty for a file in the series'
/// own folder, and for one outside it.
pub fn folder_below(series: &str, file: &str) -> String {
    let root = series.trim_end_matches(std::path::is_separator);
    let Some(rest) = file.strip_prefix(root) else {
        return String::new();
    };
    if !rest.starts_with(std::path::is_separator) {
        return String::new();
    }
    let parts: Vec<&str> = rest
        .split(std::path::is_separator)
        .filter(|p| !p.is_empty())
        .collect();
    parts[..parts.len().saturating_sub(1)].join("/")
}

/// An extra's key with the folder it sits in below its show:
/// `Season 2/NCOP1.mkv`, or the key alone for the show's own folder. Two
/// extras of one name in two folders of a show key apart this way, or they
/// would share one history (#1085).
pub fn key_in_folder(folder: &str, key: &str) -> String {
    if folder.is_empty() {
        key.to_string()
    } else {
        format!("{folder}/{key}")
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
    stripped
        .strip_suffix(['-', '_'])
        .unwrap_or(&stripped)
        .trim()
        .to_string()
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
    // A run of separators turns into a run of spaces, which the split below reads as one.
    let separators: &[char] = if dotted { &['_', '.'] } else { &['_'] };
    let rest = rest.replace(separators, " ");
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
    use anibeam_contract::ExtraKind::{Ed, Op, Other, Pv, Sp};

    /// Each name classified and read by `read`, against what it should read as.
    fn check<T: PartialEq + std::fmt::Debug>(read: impl Fn(Classified) -> T, cases: &[(&str, T)]) {
        for (name, want) in cases {
            assert_eq!(read(classify(name)), *want, "{name}");
        }
    }

    fn text(s: &str) -> Option<String> {
        Some(s.to_string())
    }

    /// The one-pass reading picks the pattern the old loop did, which tried each pattern
    /// on the token and then on its revision-cut stem before the next pattern, for every
    /// marker with every index and revision shape.
    #[test]
    fn the_extras_set_reads_a_token_as_the_pattern_loop_did() {
        let by_loop = |token: &str| {
            let stem = TOKEN_REVISION.captures(token).map(|m| m[1].to_string());
            EXTRA_PATTERNS.iter().position(|(_, re, _)| {
                re.is_match(token) || stem.as_deref().is_some_and(|s| re.is_match(s))
            })
        };
        let by_set = |token: &str| {
            let ((pattern, _), _) = token_reading(token)?;
            EXTRA_PATTERNS.iter().position(|p| std::ptr::eq(p, pattern))
        };
        let markers = [
            "NCOP", "NCED", "OP", "ED", "PV", "CM", "SP", "OVA", "OAD", "OAV", "Trailer", "Teaser",
            "Special", "Specials", "S05", "S05E", "S05EEX", "Preview", "Menu",
        ];
        for marker in markers {
            for index in ["", "1", "01", "1a", "12b", "2v"] {
                for revision in ["", "v2", "v10"] {
                    for token in [
                        format!("{marker}{index}{revision}"),
                        format!("{}{index}{revision}", marker.to_lowercase()),
                    ] {
                        assert_eq!(by_set(&token), by_loop(&token), "{token}");
                    }
                }
            }
        }
    }

    #[test]
    fn bakemonogatari_episodes_and_extras() {
        check(
            |c| (c.extra, c.number),
            &[
                (
                    "[Coalgirls]_Bakemonogatari_01_(1920x1080_Blu-ray_FLAC)_[9787055F].mkv",
                    (None, 1.0),
                ),
                (
                    "[Coalgirls]_Bakemonogatari_15_(1920x1080_Blu-ray_FLAC)_[256D3923].mkv",
                    (None, 15.0),
                ),
            ],
        );
        check(
            |c| (c.extra, c.extra_index),
            &[
                (
                    "[Coalgirls]_Bakemonogatari_ED3_(1920x1080_Blu-ray_FLAC)_[8F8AC7AF].mkv",
                    (Some(Ed), Some(3)),
                ),
                (
                    "[Coalgirls]_Bakemonogatari_OP2_(1920x1080_Blu-ray_FLAC)_[57D95944].mkv",
                    (Some(Op), Some(2)),
                ),
                (
                    "[Coalgirls]_Bakemonogatari_PV01_(1920x1080_Blu-ray_FLAC)_[8924213A].mkv",
                    (Some(Pv), Some(1)),
                ),
                (
                    "[Coalgirls]_Bakemonogatari_PV12_(1920x1080_Blu-ray_FLAC)_[17C508BF].mkv",
                    (Some(Pv), Some(12)),
                ),
            ],
        );
        check(
            |c| (c.extra, c.extra_index, c.extra_variant),
            &[
                (
                    "[Coalgirls]_Bakemonogatari_OP4b_(1920x1080_Blu-ray_FLAC)_[63162685].mkv",
                    (Some(Op), Some(4), text("b")),
                ),
                (
                    "[Coalgirls]_Bakemonogatari_OP5b_(1920x1080_Blu-ray_FLAC)_[7B7B859A].mkv",
                    (Some(Op), Some(5), text("b")),
                ),
            ],
        );
        check(
            |c| (c.extra, c.extra_index, c.extra_variant, c.raw_label),
            &[
                (
                    "[Coalgirls]_Bakemonogatari_ED1_(1920x1080_Blu-ray_FLAC)_[7EE4E478].mkv",
                    (Some(Ed), Some(1), None, text("ED1")),
                ),
                (
                    "[Coalgirls]_Bakemonogatari_OP4a_(1920x1080_Blu-ray_FLAC)_[AF4FF3CC].mkv",
                    (Some(Op), Some(4), text("a"), text("OP4a")),
                ),
            ],
        );
    }

    #[test]
    fn standard_release_patterns() {
        check(
            |c| c.number,
            &[
                ("[Kumo-Subs] Show Name - 01 [1080p].mkv", 1.0),
                ("Show Name - Episode 12.mkv", 12.0),
                ("Show Name Episode 6.5.mkv", 6.5),
            ],
        );
        check(
            |c| (c.number, c.season),
            &[("Show.Name.S02E07.1080p.WEB.mkv", (7.0, Some(2)))],
        );
        check(
            |c| (c.extra, c.extra_index),
            &[
                ("Show Name NCOP1 [1080p].mkv", (Some(Op), Some(1))),
                ("Show Name NCED2 [1080p].mkv", (Some(Ed), Some(2))),
                ("Show Name SP1 [1080p].mkv", (Some(Sp), Some(1))),
                ("Show Name Trailer1.mkv", (Some(Pv), Some(1))),
                ("Show Name Trailer.mkv", (Some(Pv), None)),
            ],
        );
        check(
            |c| (c.extra, c.extra_index, c.number, c.season),
            &[(
                "Show Name Special [1080p].mkv",
                (Some(Sp), None, 0.0, Some(0)),
            )],
        );
        check(
            |c| (c.extra, c.number),
            &[
                ("Show Name - 03 - Operations of Hope.mkv", (None, 3.0)),
                ("Show Name - 04 - Edge of Tomorrow.mkv", (None, 4.0)),
            ],
        );
    }

    #[test]
    fn season_x_episode_beats_a_number_in_the_title() {
        check(
            |c| (c.extra, c.number, c.season),
            &[("Death Note - 01x36 - 1.28.mkv", (None, 36.0, Some(1)))],
        );
        check(
            |c| (c.number, c.season),
            &[
                ("Death Note - 01x30 - Justice.mkv", (30.0, Some(1))),
                ("Show.2x05.The.Title.720p.mkv", (5.0, Some(2))),
            ],
        );
        check(
            |c| c.number,
            &[
                // A series name that opens the file is not a season and episode.
                ("3x3 Eyes - 01.mkv", 1.0),
                ("3x3.Eyes.01.mkv", 1.0),
                // Nor is a resolution.
                ("Show 1920x1080 - 07.mkv", 7.0),
            ],
        );
        // Season 1 keys by the bare number, as a name with no season does.
        let name = "Death Note - 01x36 - 1.28.mkv";
        assert_eq!(episode_key(&classify(name), name), "36");
    }

    #[test]
    fn an_ova_and_a_season_extra_are_specials() {
        check(
            |c| (c.extra, c.extra_index, c.season),
            &[("Dorohedoro - OVA .mkv", (Some(Sp), None, Some(0)))],
        );
        check(
            |c| (c.extra, c.extra_index),
            &[
                ("Show OVA2 [1080p].mkv", (Some(Sp), Some(2))),
                ("Monogatari Series S05EEX.mkv", (Some(Sp), None)),
                ("Show S01ESP2.mkv", (Some(Sp), Some(2))),
            ],
        );
        check(|c| c.extra, &[("Show - OAD.mkv", Some(Sp))]);
        // A word that only starts like one is not.
        check(
            |c| (c.extra, c.number),
            &[("Show - 03 - Ovation.mkv", (None, 3.0))],
        );
        check(
            |c| (c.extra, c.number, c.season),
            &[("Monogatari Series - S05E05.mkv", (None, 5.0, Some(5)))],
        );
    }

    #[test]
    fn a_title_word_that_starts_like_a_marker_is_not_one() {
        // A variant letter only follows a number, so "Spy" is no `SP` plus
        // a "y" (#426).
        check(
            |c| (c.extra, c.number, c.season),
            &[("Spy x Family - 01.mkv", (None, 1.0, None))],
        );
        check(
            |c| (c.extra, c.number),
            &[
                (
                    "[Lumen] Spy x Family Season 2 - 05 (1080p).mkv",
                    (None, 5.0),
                ),
                ("Spa Show - 02.mkv", (None, 2.0)),
            ],
        );
        // A plural is still the marker, with no variant.
        check(
            |c| (c.extra, c.extra_variant),
            &[
                ("Show Specials.mkv", (Some(Sp), None)),
                ("Show SPs.mkv", (Some(Sp), None)),
                ("Show PVs.mkv", (Some(Pv), None)),
                ("Show Trailers.mkv", (Some(Pv), None)),
                ("Show - Teasers 01.mkv", (Some(Pv), None)),
            ],
        );
        check(|c| c.extra_index, &[("Show - Teasers 01.mkv", Some(1))]);
        // A variant glued on after a number still reads.
        check(
            |c| (c.extra, c.extra_index, c.extra_variant),
            &[
                ("Show SP2b.mkv", (Some(Sp), Some(2), text("b"))),
                ("Show PV1a.mkv", (Some(Pv), Some(1), text("a"))),
            ],
        );
    }

    #[test]
    fn a_revision_after_an_extras_index_is_that_extra() {
        check(
            |c| (c.extra, c.extra_index, c.extra_variant),
            &[
                // `v2` after the number is the release's revision, as it is for an
                // episode, so the file is still the extra and not episode 1 (#442).
                ("[Group] Show - SP01v2.mkv", (Some(Sp), Some(1), None)),
                ("[Group] Show - NCOP1v2.mkv", (Some(Op), Some(1), None)),
                (
                    "[Group] Show - NCED2v3 [1080p].mkv",
                    (Some(Ed), Some(2), None),
                ),
                ("[Group] Show - PV1v2.mkv", (Some(Pv), Some(1), None)),
                ("[Group] Show - OVA3v2.mkv", (Some(Sp), Some(3), None)),
                ("[Group] Show - S01ESP2v2.mkv", (Some(Sp), Some(2), None)),
                // A variant letter and a revision together keep both readings, in
                // either case, and a bare `v` is still a variant.
                ("Show OP1av2.mkv", (Some(Op), Some(1), text("a"))),
                ("Show OP1V2.mkv", (Some(Op), Some(1), None)),
                ("Show OP1v.mkv", (Some(Op), Some(1), text("v"))),
            ],
        );
        // An extra with no number of its own takes a revision too.
        check(
            |c| (c.extra, c.extra_index),
            &[
                ("[Group] Show - SPv2.mkv", (Some(Sp), None)),
                ("[Group] Show - PVv2.mkv", (Some(Pv), None)),
                ("[Group] Show - OVAv2.mkv", (Some(Sp), None)),
                ("[Group] Show - S05EEXv2.mkv", (Some(Sp), None)),
            ],
        );
        // The label is still the token as written.
        check(|c| c.raw_label, &[("Show - SP01v2.mkv", text("SP01v2"))]);
    }

    #[test]
    fn a_title_word_that_is_a_bare_marker_does_not_beat_the_episode() {
        check(
            |c| (c.extra, c.number),
            &[
                // The title's own word is the marker, with no number of its own,
                // and an episode number follows it (#443).
                ("Trailer Park Boys - 01.mkv", (None, 1.0)),
                ("[Group] Special A - 07 [1080p].mkv", (None, 7.0)),
                ("Trailer.Park.Boys.S02E03.mkv", (None, 3.0)),
                ("Special A Episode 12.mkv", (None, 12.0)),
                // With no episode after it, the bare marker is the file's reading.
                ("Show - Special.mkv", (Some(Sp), 0.0)),
            ],
        );
        check(
            |c| (c.extra, c.extra_index),
            &[
                // A marker later in the name still makes the file that extra.
                ("Special A - SP01.mkv", (Some(Sp), Some(1))),
                ("Trailer Park Boys - NCOP2.mkv", (Some(Op), Some(2))),
                ("Trailer Park Boys - Trailer.mkv", (Some(Pv), None)),
            ],
        );
        // A marker with its own index keeps it, as before.
        check(
            |c| c.extra_index,
            &[
                ("Show Special - 04.mkv", Some(4)),
                ("Show - Trailer 2.mkv", Some(2)),
            ],
        );
        check(
            |c| c.extra,
            &[
                // A later bare marker beats the title's own.
                ("Trailer Park Boys - Special.mkv", Some(Sp)),
                ("Special A - Trailer.mkv", Some(Pv)),
                // A marker after the title's dash is the file's, whatever follows it:
                // a year, another word and a number, or an episode word.
                ("Show - Special - 2019.mkv", Some(Sp)),
                ("Show - Trailer - 2024.mkv", Some(Pv)),
                ("Show - Trailer Collection - 02.mkv", Some(Pv)),
                ("Show - Special Ending - 01.mkv", Some(Sp)),
                ("[Group] Show - SP Ep 01.mkv", Some(Sp)),
                ("Show - Special - Step 1.mkv", Some(Sp)),
                // A marker with no title word after it is no title word.
                ("Show Special Episode 3.mkv", Some(Sp)),
            ],
        );
    }

    #[test]
    fn a_bare_marker_in_the_episode_title_after_the_number_is_a_title_word() {
        // After the episode reading, a bare marker with a title word after it is a word
        // of the episode's title (#466).
        check(
            |c| (c.extra, c.season, c.number),
            &[
                ("Show - S01E05 - Special Delivery.mkv", (None, Some(1), 5.0)),
                ("Show - 07 - Trailer Trouble.mkv", (None, None, 7.0)),
                (
                    "[Group] Show - 12 - Special Ending [1080p].mkv",
                    (None, None, 12.0),
                ),
                ("Show.S02E03.Special.Delivery.mkv", (None, Some(2), 3.0)),
                ("Show Episode 4 Teaser Night.mkv", (None, None, 4.0)),
            ],
        );
        check(
            |c| c.extra,
            &[
                // One with nothing after it stays the file's reading: the name says the
                // episode is a special, not what it is called.
                ("Show - 01 - Special.mkv", Some(Sp)),
                // What follows has to be a title word: a release tag, a dash, an end word
                // or another marker is not one, and the file stays the extra it names.
                ("Show - 01 - Special 1080p.mkv", Some(Sp)),
                ("Show.S01E00.Special.1080p.WEB-DL.mkv", Some(Sp)),
                ("Show - 13 - Special - Recap.mkv", Some(Sp)),
                ("Show - 07 - Special Trailer.mkv", Some(Sp)),
                ("Show - 07 - Special Bonus.mkv", Some(Sp)),
                ("Show - 07 - Trailer END.mkv", Some(Pv)),
            ],
        );
        // A marker with its own number or glued to one is still the extra.
        check(
            |c| (c.extra, c.extra_index),
            &[
                ("Show - 07 - Special 2 Night.mkv", (Some(Sp), Some(2))),
                ("Show - 07 - SP01 Delivery.mkv", (Some(Sp), Some(1))),
                ("Show - 07 - NCOP1 Clean.mkv", (Some(Op), Some(1))),
            ],
        );
    }

    #[test]
    fn a_bare_marker_in_the_episode_title_reads_with_no_dash_or_a_dashed_series_title() {
        check(
            |c| (c.extra, c.number),
            &[
                // A bare number with no dash before it starts the episode reading too, and
                // so does a dash after a series title that has dashes of its own (#497).
                ("Show 07 Trailer Trouble.mkv", (None, 7.0)),
                ("Show.07.Special.Delivery.mkv", (None, 7.0)),
                ("[Group] Show 07 Special Delivery [1080p].mkv", (None, 7.0)),
                (
                    "Sword Art Online - Alicization - 05 - Special Delivery.mkv",
                    (None, 5.0),
                ),
                ("Show - Season 2 - 05 - Special Delivery.mkv", (None, 5.0)),
                ("Show S01E05 - Special Delivery.mkv", (None, 5.0)),
                // A dotted name reads its padded number, not a codec's at the end.
                (
                    "Show.07.Special.Delivery.1080p.WEB.DDP5.1.H.264-GRP.mkv",
                    (None, 7.0),
                ),
            ],
        );
        check(
            |c| c.extra,
            &[
                // A number in the series title before the marker still lets the marker
                // read, and a marker with its own number is still the extra.
                ("Mob Psycho 100 Special.mkv", Some(Sp)),
                ("Show - Alicization - 05 - Special.mkv", Some(Sp)),
                // A number that is the series title's, a season's or a codec's is no
                // reading, and a marker before the number keeps the file its extra, as
                // before.
                ("Mobile Suit Gundam 00 Special Edition I.mkv", Some(Sp)),
                ("Mobile Suit Gundam 00 - Special Edition I.mkv", Some(Sp)),
                ("Mob Psycho 100 - Special Reigen.mkv", Some(Sp)),
                ("Attack on Titan Season 3 Special Lost Girls.mkv", Some(Sp)),
                ("Attack on Titan Season 03 Special Lost Girls.mkv", Some(Sp)),
                (
                    "Show.Special.Holiday.1080p.WEB.DDP5.1.H.264-GRP.mkv",
                    Some(Sp),
                ),
                ("Show Trailer Collection 02.mkv", Some(Pv)),
                ("Show.Trailer.Collection.02.mkv", Some(Pv)),
                ("Show Special Edition 01.mkv", Some(Sp)),
            ],
        );
        check(
            |c| (c.extra, c.extra_index),
            &[("Show 07 Special 2 Night.mkv", (Some(Sp), Some(2)))],
        );
    }

    #[test]
    fn a_number_apart_from_the_token_is_its_index() {
        // The Girls und Panzer OVA Series files (#167).
        check(
            |c| (c.extra, c.extra_index, c.number, c.season),
            &[(
                "[ak-Submarines] Girls und Panzer - OVA 1 [BD 1080p][F37D267C].mkv",
                (Some(Sp), Some(1), 1.0, Some(0)),
            )],
        );
        check(
            |c| c.extra_index,
            &[
                ("Show - OVA 6 [1080p].mkv", Some(6)),
                ("Show SP 2.mkv", Some(2)),
                ("Show - Special 03 - The Title.mkv", Some(3)),
                ("Show OAD 2v2.mkv", Some(2)),
                ("Show PV 2.mkv", Some(2)),
                // A glued number still wins, and a year or a word is no index.
                ("Show OVA2 3.mkv", Some(2)),
                ("Show Special 2019.mkv", None),
                ("Show OVA Recap.mkv", None),
                // A season and episode code after the token: a folder of OVAs that
                // numbers them as a season's episodes (LiamPalmqvist's layout, #177).
                ("Seitokai Yakuindomo OVA - S02E10.mkv", Some(10)),
                ("Show OVA S01E02v2.mkv", Some(2)),
            ],
        );
        // An opening needs its number glued on, as before.
        check(|c| c.extra, &[("Show - OP 2.mkv", None)]);
        check(
            |c| (c.extra, c.extra_index),
            &[("Seitokai Yakuindomo OVA - S01E03.mkv", (Some(Sp), Some(3)))],
        );
        // The series' own episodes beside them are untouched.
        check(
            |c| (c.extra, c.number, c.season),
            &[("Seitokai Yakuindomo S1 - S01E01.mkv", (None, 1.0, Some(1)))],
        );
    }

    #[test]
    fn a_decimal_after_the_dash_is_the_episode() {
        check(
            |c| (c.extra, c.number, c.season),
            &[(
                "Kara no Kyoukai - 6.5 - Remix - Gate of Seventh Heaven (recap).mkv",
                (None, 6.5, None),
            )],
        );
        check(
            |c| c.number,
            &[
                ("[Group] Show - 12.5 - Recap Title [1080p].mkv", 12.5),
                ("Show - 07.5v2 [1080p].mkv", 7.5),
            ],
        );
    }

    #[test]
    fn the_fallback_drops_years_and_big_numbers_and_defaults_to_one() {
        check(
            |c| c.number,
            &[
                ("Show.Name.2019.1080p.Part7.mkv", 7.0),
                ("Show Name.mkv", 1.0),
                ("Show - E07.mkv", 7.0),
                ("Show Ep. 9.mkv", 9.0),
            ],
        );
        check(
            |c| (c.number, c.season),
            &[("Show Name - 00.mkv", (0.0, Some(0)))],
        );
    }

    #[test]
    fn a_revision_suffix_is_not_the_episode_number() {
        check(
            |c| (c.extra, c.number),
            &[(
                "[Kumo-Subs] Honzuki no Gekokujou S4 - 17v2 [1080p CR WEB-DL AVC AAC][MultiSub][BC7A5FC1].mkv",
                (None, 17.0),
            )],
        );
        check(
            |c| c.number,
            &[
                ("[Kumo-Subs] Neko to Ryuu - 07v2 [1080p].mkv", 7.0),
                ("Show 05v3.mkv", 5.0),
                ("Show.Name.12v2.mkv", 12.0),
                ("[Kumo-Subs] Show Name - 02 [1080p].mkv", 2.0),
            ],
        );
    }

    #[test]
    fn other_tokens_are_extras_of_kind_other() {
        check(
            |c| (c.extra, c.raw_label, c.number),
            &[("Show Name Menu.mkv", (Some(Other), text("Menu"), 0.0))],
        );
        check(
            |c| (c.extra, c.raw_label),
            &[("Show_Name_CM_01.mkv", (Some(Other), text("CM")))],
        );
    }

    #[test]
    fn keys_titles_and_numbers() {
        for (name, key) in [
            ("Show Name Episode 6.5.mkv", "6.5"),
            ("Show - 12.mkv", "12"),
            ("Show OP1.mkv", "Show OP1.mkv"),
            // Two revisions of one extra share a key, the name without the
            // revision, so they are one extra's releases with one history (#468).
            (
                "[G] Show - SP01v2 [1080p].mkv",
                "[G] Show - SP01 [1080p].mkv",
            ),
            (
                "[G] Show - SP01v3 [1080p].mkv",
                "[G] Show - SP01 [1080p].mkv",
            ),
            ("[G] Show - SP01 [1080p].mkv", "[G] Show - SP01 [1080p].mkv"),
            // An index written apart takes its revision too, and a checksum differs
            // between any two encodes, so neither keeps two releases apart.
            ("Show - OVA 01v2.mkv", "Show - OVA 01.mkv"),
            ("Show - OVA 01.mkv", "Show - OVA 01.mkv"),
            (
                "[G] Show - SP01 [1080p][A1B2C3D4].mkv",
                "[G] Show - SP01 [1080p].mkv",
            ),
            (
                "[G] Show - SP01v2 [1080p][E5F60718].mkv",
                "[G] Show - SP01 [1080p].mkv",
            ),
            ("Show_NCOP1v2.mkv", "Show_NCOP1.mkv"),
            // A number that only looks like a revision stays: PV2 is preview 2.
            ("Show PV2.mkv", "Show PV2.mkv"),
            ("Show OP1v.mkv", "Show OP1v.mkv"),
            // Season 1 keys by the bare number, as every key did before
            // seasons; a later season is prefixed so S01E01 and S02E01 never
            // share a history row.
            ("Show.Name.S01E01.mkv", "1"),
            ("Show.Name.S02E01.mkv", "S02E01"),
        ] {
            assert_eq!(episode_key(&classify(name), name), key, "{name}");
        }
        // A document that names the file finds the same key; an episode's name stays.
        assert_eq!(
            key_of_file_name("[G] Show - SP01v2 [1080p][E5F60718].mkv"),
            "[G] Show - SP01 [1080p].mkv"
        );
        assert_eq!(key_of_file_name("Show - 07v2.mkv"), "Show - 07v2.mkv");
        assert_eq!(number_key(Some(2), 12.5), "S02E12.5");
        assert_eq!(number_key(Some(0), 3.0), "S00E03");
        assert_eq!(number_key(None, 7.0), "7");
        assert_eq!(parse_episode_key("12"), Some((1, 12.0)));
        assert_eq!(parse_episode_key("12.5"), Some((1, 12.5)));
        assert_eq!(parse_episode_key("S02E01"), Some((2, 1.0)));
        assert_eq!(parse_episode_key("S02E12.5"), Some((2, 12.5)));
        assert_eq!(parse_episode_key("Show OP1.mkv"), None);
        assert_eq!(parse_episode_key("Season 2/NCOP1.mkv"), None);
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
        for (name, title) in [
            (
                "Helluva.Boss.S03E04.A.Trial.of.Errors.1080p.AMZN.WEB-DL.ENG.ITA.DDP5.1.H264-TBK.mkv",
                Some("A Trial of Errors"),
            ),
            (
                "Helluva.Boss.S03E01.Turtle.Emoji.1080p.AMZN.WEB-DL.mkv",
                Some("Turtle Emoji"),
            ),
            (
                "[Group] Show - 03 - The Heroine Arrives [1080p].mkv",
                Some("The Heroine Arrives"),
            ),
            (
                "Show S01E02 Mr. Smith Goes (1080p).mkv",
                Some("Mr. Smith Goes"),
            ),
            ("[Lumen] Show - 01 (1080p) [ABCD1234].mkv", None),
            ("Show.S01E01.720p.HDTV.x264.mkv", None),
            ("Show Episode 3.mkv", None),
            ("Show - 05v2.mkv", None),
            ("Show OP1.mkv", None),
            // From the #108 review.
            (
                "Show.S01E02.Some.Title.(2019).1080p.mkv",
                Some("Some Title"),
            ),
            (
                "[Group] Show - 12.5 - Recap Title [1080p].mkv",
                Some("Recap Title"),
            ),
            ("Show.S01E05.Magnum.Opus.1080p.mkv", Some("Magnum Opus")),
            ("Show.S02E03.The.Sub.Plot.720p.mkv", Some("The Sub Plot")),
            ("Show.S01E02.Pilot.HDTV-LOL.mkv", Some("Pilot")),
            ("Show.S01E02.Title.HDRip.XviD.mkv", Some("Title")),
            ("Show.S01E02.Title.BD1080p.Hi10P.mkv", Some("Title")),
            ("Show Ep 03 - The Title.mkv", Some("The Title")),
            ("Show S01E03 Ep 3.mkv", None),
            ("Show S01E03 Episode 03v2.mkv", None),
            ("[G] Show - 12 END [1080p].mkv", None),
            ("Death Note - 01x30 - Justice.mkv", Some("Justice")),
            ("Death Note - 01x36 - 1.28.mkv", Some("1.28")),
        ] {
            assert_eq!(file_episode_title(name).as_deref(), title, "{name}");
        }
    }

    #[test]
    fn release_labels() {
        for (path, label) in [
            (
                "/x/Helluva.Boss.S03E04.1080p.AMZN.WEB-DL.ENG.ITA.DDP5.1.H264-TBK.mkv",
                "TBK, 1080p WEB-DL H264",
            ),
            (
                "Avatar.Seven.Havens.S01E01.1080p.x265-ELiTE.mkv",
                "ELiTE, 1080p x265",
            ),
            (
                "[Kumo-Subs] Show - 17v2 [1080p CR WEB-DL AVC AAC].mkv",
                "Kumo-Subs, v2 1080p WEB-DL AVC",
            ),
            ("Show.S01E01.1080p-x265.mkv", "1080p x265"),
            ("plain name.mkv", "plain name"),
            // ASCII word boundaries, as in the old QML rule: CJK letters are not word
            // characters
            ("番組第01話1080p.mkv", "1080p"),
        ] {
            assert_eq!(release_label(path), label, "{path}");
        }
    }

    /// The folder a file sits in below its show, for a key that has to tell
    /// two of one name apart (#1085).
    #[test]
    fn the_folder_below_a_show_and_a_key_in_it() {
        for (series, file, folder) in [
            ("/lib/Show", "/lib/Show/Season 2/NCOP1.mkv", "Season 2"),
            (
                "/lib/Show/",
                "/lib/Show/Season 2/Extras/NCOP1.mkv",
                "Season 2/Extras",
            ),
            ("/lib/Show", "/lib/Show/NCOP1.mkv", ""),
            ("/lib/Show", "/lib/Show2/NCOP1.mkv", ""),
            ("/lib/Show", "/elsewhere/NCOP1.mkv", ""),
        ] {
            assert_eq!(folder_below(series, file), folder, "{file}");
        }
        assert_eq!(key_in_folder("Season 2", "NCOP1.mkv"), "Season 2/NCOP1.mkv");
        assert_eq!(key_in_folder("", "NCOP1.mkv"), "NCOP1.mkv");
    }
}
