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

/// The release group in brackets at the front of a title, brackets kept: `[SubsPlease]`.
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
        assert_eq!(
            poster_of("[SubsPlease] Frieren - 01"),
            Some("[SubsPlease]".into())
        );
        assert_eq!(poster_of("Frieren [SubsPlease]"), None);
        assert_eq!(poster_of("[] Show"), None);
    }

    #[test]
    fn show_name_cuts_at_episode_season_marker_or_metadata() {
        assert_eq!(
            show_name("[SubsPlease] Frieren - 01 (1080p) [ABCD0001].mkv"),
            "Frieren"
        );
        assert_eq!(
            show_name("[Erai-raws] Show Name - 04v2 [1080p][HEVC].mkv"),
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
    fn season_of_reads_the_first_marker() {
        assert_eq!(season_of("Show S02E05 [1080p]"), Some("S02".into()));
        assert_eq!(season_of("Show s1e2"), Some("S1".into()));
        assert_eq!(season_of("Show - 05"), None);
    }

    #[test]
    fn tokens_drop_noise_and_keep_the_season() {
        // ABCD0001 is six or more hex digits, so it reads as a hash and goes.
        let t = title_tokens("[SubsPlease] Frieren - 01 (1080p) [ABCD0001].mkv");
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
