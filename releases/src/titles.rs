//! Best-effort title parsing for torrent names. Ported from anirss's `titles.py`: same
//! patterns, same cuts, same token rules.

use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::sync::LazyLock;

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("title pattern compiles")
}

fn fancy(pattern: &str) -> FancyRegex {
    FancyRegex::new(pattern).expect("title pattern compiles")
}

static POSTER_RE: LazyLock<Regex> = LazyLock::new(|| re(r"^\[([^\]]+)\]"));
static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| re(r"[\s\[\]\(\)]+"));
static NUMERIC_RE: LazyLock<Regex> = LazyLock::new(|| re(r"\A\d+\z"));
static HEX_RE: LazyLock<Regex> = LazyLock::new(|| re(r"\A[0-9A-Fa-f]{6,}\z"));
static NOT_HEX_LETTER_RE: LazyLock<Regex> = LazyLock::new(|| re(r"[g-zG-Z]"));
/// Python's `$` also matches before one trailing newline, so the end is a lookahead.
static EXT_RE: LazyLock<FancyRegex> = LazyLock::new(|| fancy(r"\.\w{2,4}(?=\n?\z)"));
/// `<n>p`, as in `1080p`.
pub static RES_RE: LazyLock<Regex> = LazyLock::new(|| re(r"(?i)\b(\d{3,4})p\b"));
/// " - 04" or " - 04v2" followed by space, end, or bracket; not " - 1080p".
pub static EPISODE_RE: LazyLock<FancyRegex> =
    LazyLock::new(|| fancy(r"\s+-\s+\d{1,4}(?:v\d+)?(?=\s|\z|\[)"));
/// An episode range, " - 02-03", " - E01~E12" or "(01-12)", which marks a batch.
/// Kept apart from `EPISODE_RE`, which stays a single-episode marker for `bestfit`.
/// In parentheses a number has at most three digits, so "(2005-2010)" stays a span of
/// years, as `bestfit::EP_RANGE_RE` reads it.
static EPISODE_RANGE_RE: LazyLock<FancyRegex> = LazyLock::new(|| {
    fancy(concat!(
        r"\s+-\s+[Ee]?\d{1,4}(?:v\d+)?\s*[-~]\s*[Ee]?\d{1,4}(?:v\d+)?(?=\s|\z|\[|\()",
        r"|\s*\([Ee]?\d{1,3}(?:v\d+)?\s*[-~]\s*[Ee]?\d{1,3}(?:v\d+)?\)",
    ))
});
/// "S01E02" / "s1e2" / "S01E02v2" season and episode markers. The season half is a
/// legitimate search facet; the episode half must never leak into queries or refine tags
/// (it would pin a single episode).
pub static SEASON_EP_RE: LazyLock<Regex> =
    LazyLock::new(|| re(r"\b[Ss](\d{1,2})[Ee]\d{1,4}(?:v\d+)?\b"));
static SEASON_EP_FULL_RE: LazyLock<Regex> =
    LazyLock::new(|| re(r"\A\b[Ss](\d{1,2})[Ee]\d{1,4}(?:v\d+)?\b\z"));
static META_RE: LazyLock<Regex> = LazyLock::new(|| re(r"\s*\["));

/// `title` without a trailing `.ext` of two to four word characters.
fn strip_ext(title: &str) -> String {
    match EXT_RE.find(title) {
        Ok(Some(m)) => format!("{}{}", &title[..m.start()], &title[m.end()..]),
        _ => title.to_string(),
    }
}

/// The release group in brackets at the front of a title, brackets kept: `[Lumen]`.
pub fn poster_of(title: &str) -> Option<String> {
    POSTER_RE.captures(title).map(|c| format!("[{}]", &c[1]))
}

/// Best-effort show name from a torrent title: strip poster, episode, metadata, extension.
pub fn show_name(title: &str) -> String {
    let name = strip_ext(title);
    let name = POSTER_RE.replace(&name, "").trim().to_string();
    let mut cuts: Vec<usize> = Vec::new();
    if let Ok(Some(m)) = EPISODE_RE.find(&name) {
        cuts.push(m.start());
    }
    if let Ok(Some(m)) = EPISODE_RANGE_RE.find(&name) {
        cuts.push(m.start());
    }
    if let Some(m) = SEASON_EP_RE.find(&name) {
        cuts.push(m.start());
    }
    if let Some(m) = META_RE.find(&name)
        && m.start() > 0
    {
        cuts.push(m.start());
    }
    let name = match cuts.iter().min() {
        Some(&cut) => &name[..cut],
        None => name.as_str(),
    };
    name.trim().trim_matches([' ', '-']).to_string()
}

/// The folder a release downloads into: its show name, and for a season pack the name
/// before its bare season too, so "Show S01 [1080p]" lands in `Show` beside "Show - 01".
/// The season stays when an episode follows it ("Show S2 - 03"), since that file's name
/// carries no other season, and when the name starts with it.
pub fn folder_name(title: &str) -> String {
    let name = show_name(title);
    let facts = super::facts::read_name(title);
    if !facts.batch || facts.episode.is_some() {
        return name;
    }
    match super::facts::SEASON_PACK.find(&name) {
        Some(m) if m.start() > 0 => name[..m.start()]
            .trim_end_matches([' ', '-', '.', '_', '('])
            .to_string(),
        _ => name,
    }
}

/// Canonical `Sxx` from the first SxxEyy marker in the title, else None.
pub fn season_of(title: &str) -> Option<String> {
    SEASON_EP_RE.captures(title).map(|c| format!("S{}", &c[1]))
}

/// The words of a title that can narrow a search: poster, extension and episode numbers
/// out, a SxxEyy marker reduced to its season, hashes and one-letter bits dropped.
pub fn title_tokens(title: &str) -> Vec<String> {
    let base = strip_ext(title);
    let base = POSTER_RE.replace(&base, "");
    let mut tokens = Vec::new();
    for token in TOKEN_RE.split(&base) {
        if token.is_empty() || token == "-" {
            continue;
        }
        if let Some(c) = SEASON_EP_FULL_RE.captures(token) {
            tokens.push(format!("S{}", &c[1]));
            continue;
        }
        if NUMERIC_RE.is_match(token) {
            continue;
        }
        if HEX_RE.is_match(token) && !NOT_HEX_LETTER_RE.is_match(token) {
            continue;
        }
        if token.chars().count() < 2 {
            continue;
        }
        tokens.push(token.to_string());
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poster_is_the_leading_bracket() {
        assert_eq!(poster_of("[Lumen] Frieren - 01"), Some("[Lumen]".into()));
        assert_eq!(poster_of("Frieren [Lumen]"), None);
        assert_eq!(poster_of("[] Show"), None);
    }

    #[test]
    fn show_name_cuts_at_episode_season_marker_or_metadata() {
        assert_eq!(
            show_name("[Lumen] Frieren - 01 (1080p) [ABCD0001].mkv"),
            "Frieren"
        );
        assert_eq!(
            show_name("[Kumo-Subs] Show Name - 04v2 [1080p][HEVC].mkv"),
            "Show Name"
        );
        assert_eq!(show_name("[Grp] Some Show S02E05 [1080p]"), "Some Show");
        assert_eq!(
            show_name("[Grp] Some Show (Batch) [1080p]"),
            "Some Show (Batch)"
        );
        assert_eq!(show_name("[Grp] Some Show [1080p]"), "Some Show");
        assert_eq!(show_name("Plain Title.mkv"), "Plain Title");
        // " - 1080p" is a resolution, not an episode.
        assert_eq!(show_name("[Grp] Show - 1080p x265"), "Show - 1080p x265");
        // A leading bracket after the poster is not a cut at zero.
        assert_eq!(show_name("[Grp] [Tag] Show - 02"), "[Tag] Show");
        assert_eq!(show_name("- Show -"), "Show");
    }

    #[test]
    fn show_name_cuts_at_an_episode_range() {
        assert_eq!(
            show_name("[Demo] Cowboy Bebop - 02-03 [1080p]"),
            "Cowboy Bebop"
        );
        assert_eq!(show_name("[Grp] Show - 01~12 [1080p]"), "Show");
        assert_eq!(show_name("[Grp] Show - 01 ~ 12"), "Show");
        assert_eq!(show_name("[Grp] Show - 01v2-03v2.mkv"), "Show");
        assert_eq!(show_name("[Grp] Show (01-12) [1080p]"), "Show");
        assert_eq!(show_name("[Grp] Show (01~12)"), "Show");
        assert_eq!(show_name("[Grp] Show - E01-E12 [1080p]"), "Show");
        assert_eq!(show_name("[Grp] Show (01v2-12) [1080p]"), "Show");
        // A span of years is part of the name.
        assert_eq!(
            show_name("[Grp] Doctor Who (2005-2010) [1080p]"),
            "Doctor Who (2005-2010)"
        );
        // A hyphenated name is not a range.
        assert_eq!(show_name("[Grp] Re-Kan - 03 [720p]"), "Re-Kan");
        // A parenthesised word is still part of the name.
        assert_eq!(show_name("[Grp] Show (Batch) [1080p]"), "Show (Batch)");
    }

    #[test]
    fn a_season_packs_folder_drops_its_bare_season() {
        assert_eq!(folder_name("[Grp] Some Show S01 [1080p]"), "Some Show");
        assert_eq!(
            folder_name("[Grp] Some Show S2 (Batch) [1080p]"),
            "Some Show"
        );
        assert_eq!(folder_name("[Grp] Some Show s03"), "Some Show");
        assert_eq!(folder_name("[Grp] Some Show-S01 [1080p]"), "Some Show");
        assert_eq!(folder_name("Some.Show.S01.1080p.WEB-DL"), "Some.Show");
        assert_eq!(
            folder_name("[Grp] Some Show S01 - 01-12 [1080p]"),
            "Some Show"
        );
        // An episode after the season keeps it: the file says no season of its own.
        assert_eq!(
            folder_name("[Grp] Some Show S2 - 03 [1080p]"),
            "Some Show S2"
        );
        assert_eq!(folder_name("[Grp] Some Show S02E05 [1080p]"), "Some Show");
        assert_eq!(folder_name("[Grp] Some Show - 04 [1080p]"), "Some Show");
        // A name that starts with one, or a longer number, is part of the name.
        assert_eq!(
            folder_name("[Grp] S01 Project (Batch) [1080p]"),
            "S01 Project (Batch)"
        );
        assert_eq!(
            folder_name("[Grp] Some Show S100 [1080p]"),
            "Some Show S100"
        );
        // show_name itself still keeps the season, for matching and search.
        assert_eq!(show_name("[Grp] Some Show S01 [1080p]"), "Some Show S01");
    }

    #[test]
    fn season_of_reads_the_first_marker() {
        assert_eq!(season_of("Show S02E05 [1080p]"), Some("S02".into()));
        assert_eq!(season_of("Show s1e2"), Some("S1".into()));
        assert_eq!(season_of("Show - 05"), None);
    }

    #[test]
    fn tokens_drop_noise_and_keep_the_season() {
        // ABCD0001 is six or more hex digits, so it reads as a hash and goes.
        let t = title_tokens("[Lumen] Frieren - 01 (1080p) [ABCD0001].mkv");
        assert_eq!(t, vec!["Frieren", "1080p"]);
        // Six or more hex digits are a hash; a hash with a letter past f is a word.
        let t = title_tokens("[G] Show S01E03 deadbeef 12345 x a [Multi-Subs]");
        assert_eq!(t, vec!["Show", "S01", "Multi-Subs"]);
    }

    #[test]
    fn extension_needs_two_to_four_word_characters() {
        assert_eq!(strip_ext("a.mkv"), "a");
        assert_eq!(strip_ext("a.mkv\n"), "a\n");
        assert_eq!(strip_ext("a.b"), "a.b");
        assert_eq!(strip_ext("a.torrent"), "a.torrent");
    }
}
