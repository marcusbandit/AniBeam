//! The `Range` header a player sends, read against a file's length. The host's
//! episodes and the cast server both answer byte ranges, so it sits below both
//! teams (#854).

/// What a request's `Range` asks of a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Range {
    /// No range: the whole file.
    Whole,
    /// The first and the last byte.
    Part(u64, u64),
    /// A range the file cannot meet.
    Unmet,
}

/// The one range a request asks for of a `len` byte file.
pub fn byte_range(header: Option<&str>, len: u64) -> Range {
    let Some(header) = header else {
        return Range::Whole;
    };
    let parsed = header
        .strip_prefix("bytes=")
        .and_then(|v| v.split(',').next())
        .and_then(|v| {
            let (a, b) = v.split_once('-')?;
            let (start, end) = match (a.trim(), b.trim()) {
                ("", suffix) => (
                    len.saturating_sub(suffix.parse().ok()?),
                    len.saturating_sub(1),
                ),
                (a, "") => (a.parse().ok()?, len.saturating_sub(1)),
                (a, b) => (
                    a.parse().ok()?,
                    b.parse::<u64>().ok()?.min(len.saturating_sub(1)),
                ),
            };
            (start <= end && start < len).then_some((start, end))
        });
    parsed.map_or(Range::Unmet, |(s, e)| Range::Part(s, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_range_reads_like_a_browser_sends_it() {
        assert_eq!(byte_range(None, 100), Range::Whole);
        assert_eq!(byte_range(Some("bytes=0-"), 100), Range::Part(0, 99));
        assert_eq!(byte_range(Some("bytes=10-19"), 100), Range::Part(10, 19));
        assert_eq!(byte_range(Some("bytes=-10"), 100), Range::Part(90, 99));
        assert_eq!(byte_range(Some("bytes=50-500"), 100), Range::Part(50, 99));
        assert_eq!(byte_range(Some("bytes=200-"), 100), Range::Unmet);
        assert_eq!(byte_range(Some("pages=1"), 100), Range::Unmet);
    }
}
