//! A subscription's search, read back from its feed URL.

use crate::percent;

/// A feed's `q` parameter, percent-decoded with `+` read as a
/// space. `None` for a URL with no query, no `q` in it, or a `q` that
/// decodes to nothing.
pub fn decode_search_query(url: &str) -> Option<String> {
    let (_, query) = url.split_once('?')?;
    let query = query.split('#').next().unwrap_or(query);
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == "q" {
            let decoded = percent::decode(value);
            return if decoded.is_empty() {
                None
            } else {
                Some(decoded)
            };
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_search_query_reads_the_q_parameter() {
        assert_eq!(
            decode_search_query("https://tracker.example/?page=rss&q=Frieren+1080p&c=1_2"),
            Some("Frieren 1080p".to_string())
        );
    }

    #[test]
    fn decode_search_query_is_none_without_one() {
        assert_eq!(
            decode_search_query("https://tracker.example/?page=rss&c=1_2"),
            None
        );
    }
}
