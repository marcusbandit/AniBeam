//! A feed's search, read back from its feed URL.

/// A feed's `q` parameter, read as `rewire::search_param` reads a feed's
/// search. `None` for a URL with no query, no `q` in it, or a `q` that
/// decodes to nothing but spaces.
pub fn decode_search_query(url: &str) -> Option<String> {
    crate::rewire::search_param(url)
        .filter(|(key, _)| key == "q")
        .map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_search_query_reads_the_q_parameter_and_nothing_else() {
        for (url, want) in [
            (
                "https://t.example/?page=rss&q=Frieren+1080p&c=1_2",
                Some("Frieren 1080p"),
            ),
            (
                "https://t.example/?q=Frieren%201080p#top",
                Some("Frieren 1080p"),
            ),
            ("https://t.example/#/feed?q=Frieren", Some("Frieren")),
            ("https://t.example/?page=rss&c=1_2", None),
            ("https://t.example/?q=&s=Frieren", None),
            ("https://t.example/feed", None),
        ] {
            assert_eq!(decode_search_query(url).as_deref(), want, "{url}");
        }
    }
}
