//! A pasted URL turned into a provider id. The match modal takes links in
//! the same box as a search query, so the first question is always whether
//! the text is a link at all: `None` means "search for it".
//!
//! Carried from Electron's `src/shared/metadataLink.ts` with TMDB dropped:
//! the native line has no TMDB client, so a themoviedb.org link is a known
//! host with nothing behind it rather than a match target. TVmaze links
//! are, and so are IMDb title links, which TVmaze can look up.

use std::sync::LazyLock;

use regex::Regex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Link {
    Anilist {
        id: u64,
    },
    Mal {
        id: u64,
    },
    Tvmaze {
        id: u64,
    },
    /// An IMDb title id, `tt` and its digits, kept as the string IMDb
    /// writes it so the leading zeros survive.
    Imdb {
        id: String,
    },
    Unknown,
}

static HAS_SCHEME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^https?://").unwrap());

/// People paste from the address bar, which drops the scheme. Only the
/// hosts that can be looked up get that leniency; `example.com/anime/21`
/// stays a search.
static KNOWN_HOST_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(www\.|m\.)?(anilist\.co|myanimelist\.net|themoviedb\.org|tvmaze\.com|imdb\.com)/",
    )
    .unwrap()
});

/// Digits only and above zero. `21abc`, `-21` and `0` are all links with
/// nothing behind them rather than search queries, so the caller reports
/// `Unknown`.
fn positive_int(s: Option<&str>) -> Option<u64> {
    let s = s?;
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse::<u64>().ok().filter(|n| *n > 0)
}

/// None when the text is not a link at all: the caller searches for it.
pub fn parse(text: &str) -> Option<Link> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let has_scheme = HAS_SCHEME.is_match(trimmed);
    if !has_scheme && !KNOWN_HOST_PREFIX.is_match(trimmed) {
        return None;
    }
    let full = if has_scheme {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let rest = full.split_once("://")?.1;
    let (host_port, path_query) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let host = host_port.split(':').next().unwrap_or("").to_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host).to_string();
    let host = host.strip_prefix("m.").unwrap_or(&host).to_string();
    let (path, query) = match path_query.find('?') {
        Some(i) => (&path_query[..i], &path_query[i + 1..]),
        None => (path_query, ""),
    };
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let query_id = query.split('&').find_map(|kv| kv.strip_prefix("id="));
    Some(match host.as_str() {
        "anilist.co" => match (segments.first(), positive_int(segments.get(1).copied())) {
            (Some(&"anime"), Some(id)) => Link::Anilist { id },
            _ => Link::Unknown,
        },
        // Two shapes: /anime/{id}/Slug and the legacy /anime.php?id={id}.
        "myanimelist.net" => {
            let id = match segments.first() {
                Some(&"anime") => positive_int(segments.get(1).copied()),
                Some(&"anime.php") => positive_int(query_id),
                _ => None,
            };
            id.map_or(Link::Unknown, |id| Link::Mal { id })
        }
        // /shows/{id}/slug, the page a user copies the address of.
        "tvmaze.com" => match (segments.first(), positive_int(segments.get(1).copied())) {
            (Some(&"shows"), Some(id)) => Link::Tvmaze { id },
            _ => Link::Unknown,
        },
        // /title/tt1234567/, with or without a locale in front of it.
        "imdb.com" => {
            let at = segments.iter().position(|s| *s == "title");
            match at.and_then(|i| segments.get(i + 1)) {
                Some(id) if is_imdb_title(id) => Link::Imdb {
                    id: id.to_ascii_lowercase(),
                },
                _ => Link::Unknown,
            }
        }
        _ => Link::Unknown,
    })
}

/// `tt` and at least one digit, digits only after it.
fn is_imdb_title(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower
        .strip_prefix("tt")
        .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_anilist_link_carries_its_id_with_or_without_the_scheme() {
        assert_eq!(
            parse("https://anilist.co/anime/21/One-Piece"),
            Some(Link::Anilist { id: 21 })
        );
        assert_eq!(parse("anilist.co/anime/21"), Some(Link::Anilist { id: 21 }));
        assert_eq!(
            parse("www.anilist.co/anime/21"),
            Some(Link::Anilist { id: 21 })
        );
        assert_eq!(
            parse("  https://ANILIST.co/anime/21  "),
            Some(Link::Anilist { id: 21 })
        );
    }

    #[test]
    fn both_myanimelist_shapes_carry_their_id() {
        assert_eq!(
            parse("https://myanimelist.net/anime/21/One_Piece"),
            Some(Link::Mal { id: 21 })
        );
        assert_eq!(
            parse("https://myanimelist.net/anime.php?id=21"),
            Some(Link::Mal { id: 21 })
        );
    }

    /// A known host with nothing usable behind it is a link the core
    /// cannot follow, which is a different answer from "that was a search
    /// query": the modal says so rather than searching for the URL.
    #[test]
    fn a_known_host_with_no_id_behind_it_is_unknown() {
        assert_eq!(parse("https://anilist.co/anime/21abc"), Some(Link::Unknown));
        assert_eq!(parse("https://anilist.co/anime/0"), Some(Link::Unknown));
        assert_eq!(parse("https://anilist.co/user/x"), Some(Link::Unknown));
        assert_eq!(
            parse("https://www.themoviedb.org/movie/550"),
            Some(Link::Unknown)
        );
    }

    #[test]
    fn a_tvmaze_show_link_carries_its_id() {
        assert_eq!(
            parse("https://www.tvmaze.com/shows/83073/avatar-seven-havens"),
            Some(Link::Tvmaze { id: 83073 })
        );
        assert_eq!(
            parse("tvmaze.com/shows/83073"),
            Some(Link::Tvmaze { id: 83073 })
        );
        assert_eq!(
            parse("https://www.tvmaze.com/episodes/3682283/x"),
            Some(Link::Unknown)
        );
        assert_eq!(parse("https://tvmaze.com/shows/abc"), Some(Link::Unknown));
    }

    #[test]
    fn an_imdb_title_link_carries_its_title_id() {
        assert_eq!(
            parse("https://www.imdb.com/title/tt35854736/"),
            Some(Link::Imdb {
                id: "tt35854736".into()
            })
        );
        assert_eq!(
            parse("imdb.com/title/tt0000001/?ref_=nv_sr_srsg_0"),
            Some(Link::Imdb {
                id: "tt0000001".into()
            })
        );
        assert_eq!(
            parse("https://m.imdb.com/de/title/TT35854736"),
            Some(Link::Imdb {
                id: "tt35854736".into()
            })
        );
        assert_eq!(
            parse("https://www.imdb.com/name/nm0000001/"),
            Some(Link::Unknown)
        );
        assert_eq!(parse("https://www.imdb.com/title/tt/"), Some(Link::Unknown));
    }

    #[test]
    fn anything_that_is_not_a_link_is_a_search_query() {
        assert_eq!(parse("example.com/anime/21"), None);
        assert_eq!(parse("Frieren"), None);
        assert_eq!(parse(""), None);
        assert_eq!(parse("   "), None);
    }
}
