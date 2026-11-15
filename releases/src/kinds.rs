//! Endpoint kinds are named for the shape of what they read, never for a site (#277). Two
//! kinds had a site's name before #602, and anirss, whose own names stay as they are, still
//! knows one of them by it. This is the one module that holds those names: an installed
//! endpoint, an index or anirss's list that carries one is read through `shape_of`, and what
//! goes to anirss through `anirss_kind`.

/// A base address the core turns into an RSS search with a category and an uploader filter,
/// asked again unfiltered when the filter finds nothing (`endpoints::search_url`).
pub const FILTERED_RSS: &str = "filtered-rss";

/// A JSON feed of torrent entries in one fixed layout (`json_feed.rs`).
pub const JSON_FEED: &str = "json-feed";

/// Each kind's name from before #602, and the shape name it has now.
pub const RENAMED: [(&str, &str); 2] = [("nyaa", FILTERED_RSS), ("tosho", JSON_FEED)];

/// `kind` under its shape name: a name from before #602 becomes the kind it was renamed to,
/// and every other kind stays as it is.
pub fn shape_of(kind: &str) -> &str {
    RENAMED
        .iter()
        .find(|(old, _)| *old == kind)
        .map_or(kind, |(_, now)| now)
}

/// The kind anirss searches `kind` with. anirss knows filtered-rss by its old name and reads
/// a torznab feed through its `rss` kind; every other kind goes over as itself.
pub fn anirss_kind(kind: &str) -> &str {
    match kind {
        FILTERED_RSS => RENAMED[0].0,
        "torznab" => "rss",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_old_name_reads_as_its_shape_and_every_other_kind_stays() {
        for (old, now) in RENAMED {
            assert_eq!(shape_of(old), now);
            assert_eq!(shape_of(now), now);
        }
        for kind in ["rss", "torznab", "json", "anything"] {
            assert_eq!(shape_of(kind), kind);
        }
    }

    #[test]
    fn anirss_gets_its_own_names() {
        assert_eq!(anirss_kind(FILTERED_RSS), RENAMED[0].0);
        assert_eq!(anirss_kind("torznab"), "rss");
        assert_eq!(anirss_kind("rss"), "rss");
        // What anirss says back reads as the kind it was sent as.
        assert_eq!(shape_of(anirss_kind(FILTERED_RSS)), FILTERED_RSS);
    }
}
