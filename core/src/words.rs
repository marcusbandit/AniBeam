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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_is_singular_and_every_other_count_is_not() {
        assert_eq!(pick(1u64, "show", "shows"), "show");
        assert_eq!(pick(0usize, "show", "shows"), "shows");
        assert_eq!(pick(2i64, "show", "shows"), "shows");
        assert_eq!(pick(-1i64, "show", "shows"), "shows");
    }
}
