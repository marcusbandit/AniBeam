//! Small wording helpers for the lines the core writes for a person.

/// `one` for exactly one, `many` for any other count: the word that follows
/// or agrees with a number ("1 show", "2 shows"; "1 show needs", "2 shows
/// need").
pub fn pick<'a>(n: impl TryInto<u64>, one: &'a str, many: &'a str) -> &'a str {
    if n.try_into().is_ok_and(|n| n == 1) {
        one
    } else {
        many
    }
}

/// `text` cut to at most `max` characters, the cut marked with "..." inside that limit.
/// Space left before the marker is dropped, so a cut at a word break comes out shorter.
/// A limit too small for the marker gets a plain cut.
pub fn cut(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    if max < 3 {
        return text.chars().take(max).collect();
    }
    let kept: String = text.chars().take(max - 3).collect();
    format!("{}...", kept.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cut_keeps_short_text_and_marks_a_long_one_within_the_limit() {
        assert_eq!(cut("Bebop", 10), "Bebop");
        assert_eq!(cut("Cowboy Bebop", 10), "Cowboy...");
        assert_eq!(cut("åäöåäöåäöå", 10), "åäöåäöåäöå");
        assert_eq!(cut("åäöåäöåäöåä", 10).chars().count(), 10);
        assert_eq!(cut("abcd", 2), "ab");
        assert_eq!(cut("abcd", 3), "...");
    }

    #[test]
    fn one_is_singular_and_every_other_count_is_not() {
        assert_eq!(pick(1u64, "show", "shows"), "show");
        assert_eq!(pick(0usize, "show", "shows"), "shows");
        assert_eq!(pick(2i64, "show", "shows"), "shows");
        assert_eq!(pick(-1i64, "show", "shows"), "shows");
    }
}
