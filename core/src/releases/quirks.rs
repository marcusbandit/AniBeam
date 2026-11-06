//! What sets an RSS source apart from a plain feed, as its endpoint definition says (#604).
//! Ported from anirss's search-term rules, without their terminal colours; the core knows no
//! site, and a definition turns each part on as data:
//!
//! - `quality_local`: the source's search is a plain word-AND over post text whose posts
//!   carry no quality tags, so a query like "Show 1080p WEB-DL" zeroes it out. The quality
//!   words are taken out of the search (`split_noise`) and narrow the results here, matched
//!   against the title and the feed's extension fields (`matches_all`).
//! - `one_variant`: the source posts every episode twice, as it airs and as a re-encode, and
//!   no feed parameter separates them. A subscription whose query picks no variant excludes
//!   this word (`variant_exclusions`), so each episode comes down once.
//! - `show_feed`: the source has a feed per show. Its slug is read from the post page a
//!   result links to, through the definition's patterns, and checked with one fetch: a wrong
//!   slug answers an empty feed, which means "keep the search feed", never a failure.

use std::future::Future;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::Endpoint;
use super::endpoints::{FIELD_RE, parse_qsl, urlencode};
use crate::contract::{CoreError, Release};

/// The parts of a definition that make a source more than a plain RSS feed. All off, the
/// source is read as any feed is.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Quirks {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub quality_local: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub one_variant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_feed: Option<ShowFeed>,
}

/// Where a show's own feed is, and how its slug is read from a result's post page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShowFeed {
    /// Patterns over a result's `page`, each with a group named `slug`; the first that
    /// matches gives the show's slug.
    pub pages: Vec<String>,
    /// The show's feed with `{slug}` in it. The search address's own parameters, all but the
    /// one holding the search, are carried over.
    pub url: String,
}

/// The longest a `pages` pattern may be, and how many a definition may have.
const MAX_PATTERN: usize = 400;
const MAX_PATTERNS: usize = 8;

impl Quirks {
    /// Whether a subscription on this source is narrowed in its rule here rather than by the
    /// feed alone.
    pub fn narrows_in_rule(&self) -> bool {
        self.quality_local || self.one_variant.is_some() || self.show_feed.is_some()
    }

    /// What a definition's quirks must be: a short plain variant word, and a show feed on
    /// `host` whose patterns compile and name their slug.
    pub fn check(&self, host: &str) -> Result<(), String> {
        if let Some(word) = &self.one_variant
            && (word.is_empty()
                || word.len() > 32
                || !word.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
        {
            return Err(format!("one_variant {word:?} must be one short word"));
        }
        let Some(feed) = &self.show_feed else {
            return Ok(());
        };
        if feed.pages.is_empty() || feed.pages.len() > MAX_PATTERNS {
            return Err(format!("show_feed takes 1 to {MAX_PATTERNS} page patterns"));
        }
        for p in &feed.pages {
            if p.len() > MAX_PATTERN {
                return Err("a show_feed page pattern is too long".into());
            }
            let re = compile(p).map_err(|e| format!("show_feed page pattern: {e}"))?;
            if !re.capture_names().any(|n| n == Some("slug")) {
                return Err("a show_feed page pattern needs a group named slug".into());
            }
        }
        // The show's feed carries the search address's parameters, a token among them, so it
        // must go to the same host.
        let rest = feed
            .url
            .strip_prefix("https://")
            .ok_or("show_feed url must be https")?;
        let feed_host = rest.split(['/', '?', '#']).next().unwrap_or("");
        if !feed_host.eq_ignore_ascii_case(host) {
            return Err("show_feed url must be on the endpoint's own host".into());
        }
        if feed.url.matches("{slug}").count() != 1 || feed.url.replace("{slug}", "").contains('{') {
            return Err("show_feed url takes {slug} once and no other placeholder".into());
        }
        Ok(())
    }
}

fn compile(pattern: &str) -> Result<Regex, regex::Error> {
    regex::RegexBuilder::new(pattern)
        .size_limit(1 << 16)
        .build()
}

/// Quality facets a `quality_local` source keeps in its feed's resolution, infomkv, linktype
/// and category fields instead of the post text its search matches. Whole-token only:
/// "1080p" is stripped from a query, "NF" (a real post-text platform tag) is not.
static QUALITY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(?:\d{3,4}p|web[\s._-]?(?:dl|rip)?|blu[\s._-]?ray|bd[\s._-]?(?:rip|mv)?|hdtv|dvd|avc|hevc|x[\s._-]?26[45]|h[\s._-]?26[45]|av1|aac|flac|mp3|e?ac3|dts|airing|encoded|batch)$",
    )
    .expect("QUALITY_RE compiles")
});

/// A query field without its leading `-` and surrounding quotes.
pub(crate) fn field_core(field: &str) -> &str {
    let core = field.strip_prefix('-').unwrap_or(field);
    if core.chars().count() >= 2 && core.starts_with('"') && core.ends_with('"') {
        return &core[1..core.len() - 1];
    }
    core
}

/// Is this query field a quality facet? A leading `-` and quotes are stripped before the
/// whole-token check.
pub fn is_quality_token(field: &str) -> bool {
    let core = field_core(field);
    !core.is_empty() && QUALITY_RE.is_match(core)
}

/// The words that pick a variant: a source that posts the WEB-DL as it airs and a re-encode
/// hours later matches both with a qB rule that names neither, and downloads each episode
/// twice.
static VARIANT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(?:avc|hevc|x[\s._-]?26[45]|h[\s._-]?26[45]|web[\s._-]?(?:dl|rip)|airing|encoded)$",
    )
    .expect("VARIANT_RE compiles")
});

/// Rule exclusions that keep a subscription to one release per episode: `[word]`, the
/// definition's `one_variant` (drop the re-encode, keep the one that airs), unless a field
/// of `query`, positive or `-`excluded, already picks a variant.
pub fn variant_exclusions(query: &str, word: &str) -> Vec<String> {
    let picked = FIELD_RE
        .find_iter(query)
        .any(|m| VARIANT_RE.is_match(field_core(m.as_str())));
    if picked {
        Vec::new()
    } else {
        vec![word.to_string()]
    }
}

/// (searchable terms, quality tokens) from a positive-terms query.
///
/// The quality tokens are the facets the feed carries as extension fields; the search only
/// matches post text, so sending them zeroes it out. They come back for client-side
/// filtering (`matches_all`) instead. Quoted phrases survive: they never match the
/// whole-token pattern.
pub fn split_noise(query: &str) -> (String, Vec<String>) {
    let mut terms: Vec<&str> = Vec::new();
    let mut quality: Vec<String> = Vec::new();
    for field in FIELD_RE.find_iter(query).map(|m| m.as_str()) {
        if is_quality_token(field) {
            quality.push(field.trim_matches('"').to_string());
            continue;
        }
        terms.push(field);
    }
    (terms.join(" "), quality)
}

/// Does `token` match the item's structured extension fields? resolution, category and
/// linktype by equality, infomkv by substring. False for items without them (all fields
/// empty), so other feeds are unaffected.
pub fn metadata_hit(item: &Release, token: &str) -> bool {
    if item.resolution.is_empty() && item.infomkv.is_empty() && item.linktype.is_empty() {
        return false;
    }
    let lc = token.to_lowercase();
    lc == item.resolution.to_lowercase()
        || item.infomkv.to_lowercase().contains(&lc)
        || lc == item.linktype.to_lowercase()
        || lc == item.category.to_lowercase()
}

/// Client-side facet match for one item: title text first, then the structured extension
/// fields. The polished title drops the linktype prefix, so "Torrent" only exists there.
pub fn matches_token(item: &Release, token: &str) -> bool {
    let lc = token.to_lowercase();
    item.title.to_lowercase().contains(&lc) || metadata_hit(item, &lc)
}

/// Every token matches (see `matches_token`).
pub fn matches_all(item: &Release, tokens: &[String]) -> bool {
    tokens.iter().all(|t| matches_token(item, t))
}

/// The show's slug in `page`, by the first of `feed`'s patterns that matches it. Only a plain
/// slug counts: letters, digits, `.`, `_` and `-`.
pub(crate) fn show_slug(feed: &ShowFeed, page: &str) -> Option<String> {
    feed.pages
        .iter()
        .filter_map(|p| compile(p).ok())
        .find_map(|re| {
            re.captures(page)?
                .name("slug")
                .map(|m| m.as_str().to_string())
        })
        .filter(|slug| {
            !slug.is_empty()
                && slug
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        })
}

/// The show's own feed for `slug`, carrying over every parameter of the endpoint's address
/// but the one that holds the search.
pub fn per_show_feed_url(ep: &Endpoint, feed: &ShowFeed, slug: &str) -> String {
    let qs = ep.url.split_once('?').map_or("", |(_, qs)| qs);
    let params: Vec<(String, String)> = parse_qsl(qs)
        .into_iter()
        .filter(|(_, v)| !v.contains("{query}"))
        .collect();
    let base = feed.url.replace("{slug}", slug);
    if params.is_empty() {
        return base;
    }
    let join = if base.contains('?') { '&' } else { '?' };
    format!("{base}{join}{}", urlencode(&params))
}

/// The show's own feed for the show `items` belong to, or `None` to make the caller fall
/// back to the search feed. Costs at most one request: the first item with a recognizable
/// post page decides. `fetch` fetches and reads one feed URL.
pub async fn resolve_show_feed<F, Fut>(ep: &Endpoint, items: &[Release], fetch: F) -> Option<String>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Result<Vec<Release>, CoreError>>,
{
    let feed = ep.quirks.show_feed.as_ref()?;
    for item in items {
        let Some(slug) = show_slug(feed, &item.page) else {
            continue;
        };
        let url = per_show_feed_url(ep, feed, &slug);
        match fetch(url.clone()).await {
            Err(e) => {
                tracing::warn!("{}: show feed probe failed: {e}", ep.name);
                return None;
            }
            Ok(found) if !found.is_empty() => {
                tracing::info!(
                    "{}: show feed for {slug:?}: {} item(s)",
                    ep.name,
                    found.len()
                );
                return Some(url);
            }
            Ok(_) => {
                tracing::warn!(
                    "{}: show {slug:?}: empty probe, using the search feed",
                    ep.name
                );
                return None;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::super::endpoints::tests::{keyed_ep, rel};
    use super::*;
    use std::sync::Mutex;

    const SHOW_PAGE: &str = "https://www.keyed.example/episodes/gaikotsu-kishi-sama-tadaima-isekai-e-odekake-chuu-ii-02/";

    fn keyed_items() -> Vec<Release> {
        let mut item = rel(
            "Gaikotsu Kishi-sama Tadaima Isekai e Odekake-chuu II - 02 [1080p CR WEB-DL AVC AAC][Airing]",
            "l",
        );
        item.page = SHOW_PAGE.to_string();
        vec![item]
    }

    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn feed() -> ShowFeed {
        keyed_ep().quirks.show_feed.unwrap()
    }

    #[test]
    fn a_plain_source_narrows_nothing_in_its_rule() {
        assert!(!Quirks::default().narrows_in_rule());
        assert!(keyed_ep().quirks.narrows_in_rule());
    }

    #[test]
    fn a_definitions_quirks_are_checked() {
        let host = "www.keyed.example";
        assert!(keyed_ep().quirks.check(host).is_ok());
        let with = |f: &dyn Fn(&mut Quirks)| {
            let mut q = keyed_ep().quirks;
            f(&mut q);
            q.check(host)
        };
        assert!(with(&|q| q.one_variant = Some("two words".into())).is_err());
        assert!(with(&|q| q.show_feed.as_mut().unwrap().pages.clear()).is_err());
        assert!(with(&|q| q.show_feed.as_mut().unwrap().pages = vec!["(unclosed".into()]).is_err());
        assert!(
            with(&|q| q.show_feed.as_mut().unwrap().pages = vec!["/episodes/([a-z]+)/".into()])
                .is_err(),
            "no slug group"
        );
        assert!(
            with(&|q| q.show_feed.as_mut().unwrap().url =
                "https://elsewhere.example/{slug}/feed/".into())
            .is_err(),
            "the token would go to another host"
        );
        assert!(
            with(&|q| q.show_feed.as_mut().unwrap().url =
                "http://www.keyed.example/{slug}/".into())
            .is_err()
        );
        assert!(
            with(&|q| q.show_feed.as_mut().unwrap().url =
                "https://www.keyed.example/{slug}/{token}/".into())
            .is_err()
        );
    }

    #[test]
    fn split_noise_strips_quality_tokens() {
        let (terms, quality) = split_noise("Ranma 3rd Season 1080p WEB-DL AVC AAC");
        assert_eq!(terms, "Ranma 3rd Season");
        assert_eq!(quality, strings(&["1080p", "WEB-DL", "AVC", "AAC"]));
    }

    #[test]
    fn split_noise_keeps_searchable_text() {
        // Platform tags ("NF") are real post text; a year is not a resolution; quoted
        // phrases never match the whole-token pattern.
        let (terms, quality) = split_noise(r#"Ranma NF 2024 "Multi Subs""#);
        assert_eq!(terms, r#"Ranma NF 2024 "Multi Subs""#);
        assert!(quality.is_empty());
    }

    #[test]
    fn split_noise_strips_category_tokens() {
        let (terms, quality) = split_noise("Ranma Airing");
        assert_eq!(terms, "Ranma");
        assert_eq!(quality, strings(&["Airing"]));
    }

    #[test]
    fn quality_tokens_are_whole_tokens_after_dash_and_quotes() {
        for field in [
            "1080p",
            "720P",
            "WEB",
            "web-dl",
            "WEBRip",
            "web.dl",
            "BD",
            "bdrip",
            "blu-ray",
            "x265",
            "H.264",
            "AV1",
            "EAC3",
            "-hevc",
            "\"web dl\"",
            "-\"1080p\"",
        ] {
            assert!(is_quality_token(field), "{field}");
        }
        for field in [
            "NF",
            "2024",
            "Ranma",
            "10800p",
            "web-dl2",
            "\"1080p x\"",
            "-",
            "\"\"",
        ] {
            assert!(!is_quality_token(field), "{field}");
        }
    }

    #[test]
    fn matches_token_uses_metadata() {
        let mut item = rel("Ranma - 01 [Airing]", "l");
        item.resolution = "1080p".to_string();
        item.infomkv = "NF WEB-DL AVC AAC".to_string();
        item.linktype = "Torrent".to_string();
        assert!(matches_token(&item, "torrent")); // linktype, stripped from titles
        assert!(matches_token(&item, "web-dl")); // infomkv
        assert!(matches_token(&item, "1080p")); // resolution
        assert!(matches_token(&item, "airing")); // title text
        assert!(!matches_token(&item, "hevc"));
        assert!(matches_all(&item, &strings(&["torrent", "WEB-DL"])));
        assert!(!matches_all(&item, &strings(&["torrent", "hevc"])));
    }

    #[test]
    fn metadata_hit_ignores_items_without_extension_fields() {
        assert!(!metadata_hit(&rel("Show 1080p WEB-DL", "l"), "1080p"));
    }

    #[test]
    fn metadata_hit_matches_the_category_once_there_is_metadata() {
        let mut item = rel("t", "l");
        item.linktype = "Torrent".to_string();
        item.category = "Airing".to_string();
        assert!(metadata_hit(&item, "airing"));
    }

    #[test]
    fn show_slug_per_section() {
        let page =
            |section: &str, slug: &str| format!("https://www.keyed.example/{section}/{slug}/");
        let slug = |section: &str, post: &str| show_slug(&feed(), &page(section, post));
        assert_eq!(
            slug("episodes", "gaikotsu-kishi-sama-02").unwrap(),
            "gaikotsu-kishi-sama"
        );
        // A show whose own slug ends in a number must survive the strip.
        assert_eq!(
            slug("episodes", "mob-psycho-100-05").unwrap(),
            "mob-psycho-100"
        );
        assert_eq!(slug("episodes", "show-02v2").unwrap(), "show");
        assert_eq!(
            slug("encodes", "gaikotsu-kishi-sama-01-hevc").unwrap(),
            "gaikotsu-kishi-sama"
        );
        assert_eq!(
            slug("batches", "86-eighty-six-01-11").unwrap(),
            "86-eighty-six"
        );
        // Nothing stripped, nothing left, or an unknown section is not trusted.
        assert_eq!(slug("episodes", "no-number-here"), None);
        assert_eq!(slug("episodes", "02"), None);
        assert_eq!(slug("elsewhere", "show-02"), None);
    }

    #[test]
    fn per_show_feed_url_carries_params_drops_search() {
        let url = per_show_feed_url(&keyed_ep(), &feed(), "86-eighty-six");
        assert!(url.starts_with("https://www.keyed.example/anime-list/86-eighty-six/feed/?"));
        assert!(url.contains("res=1080p") && url.contains("type=torrent"));
        assert!(url.contains("subs%5B0%5D=us") && url.contains("token=TOK"));
        assert!(!url.contains("{query}") && !url.contains("&s=") && !url.contains("?s="));
    }

    async fn resolve(
        items: &[Release],
        reply: Result<Vec<Release>, CoreError>,
    ) -> (Option<String>, Vec<String>) {
        let calls = Mutex::new(Vec::new());
        let url = resolve_show_feed(&keyed_ep(), items, |url: String| {
            calls.lock().unwrap().push(url);
            let reply = reply.clone();
            async move { reply }
        })
        .await;
        (url, calls.into_inner().unwrap())
    }

    #[tokio::test]
    async fn resolve_show_feed_success() {
        let (url, calls) = resolve(&keyed_items(), Ok(vec![rel("x", "l")])).await;
        let url = url.unwrap();
        assert!(
            url.contains("/anime-list/gaikotsu-kishi-sama-tadaima-isekai-e-odekake-chuu-ii/feed/")
        );
        assert_eq!(calls, vec![url]);
    }

    #[tokio::test]
    async fn resolve_show_feed_empty_probe_falls_back() {
        assert_eq!(resolve(&keyed_items(), Ok(vec![])).await.0, None);
    }

    #[tokio::test]
    async fn resolve_show_feed_fetch_error_falls_back() {
        assert_eq!(
            resolve(&keyed_items(), Err(CoreError::internal("boom")))
                .await
                .0,
            None
        );
    }

    #[tokio::test]
    async fn resolve_show_feed_without_page_makes_no_request() {
        let (url, calls) =
            resolve(&[rel("t", "l")], Err(CoreError::internal("must not fetch"))).await;
        assert_eq!(url, None);
        assert!(calls.is_empty());
    }

    #[tokio::test]
    async fn a_source_with_no_show_feed_makes_no_request() {
        let calls = Mutex::new(0);
        let plain = Endpoint::new("feed", "rss", "https://feed.example/?q={query}");
        let url = resolve_show_feed(&plain, &keyed_items(), |_| {
            *calls.lock().unwrap() += 1;
            async { Ok(vec![rel("x", "l")]) }
        })
        .await;
        assert_eq!((url, *calls.lock().unwrap()), (None, 0));
    }

    #[tokio::test]
    async fn resolve_show_feed_skips_unrecognised_posts_to_the_next_item() {
        let mut odd = rel("t", "l");
        odd.page = "https://www.keyed.example/episodes/no-number-here/".to_string();
        let mut items = vec![odd];
        items.extend(keyed_items());
        let (url, calls) = resolve(&items, Ok(vec![rel("x", "l")])).await;
        assert!(url.is_some());
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn variant_exclusions_neutral_query_drops_the_encode() {
        assert_eq!(
            variant_exclusions("Kyoufu 1080p", "HEVC"),
            strings(&["HEVC"])
        );
        // WEB alone matches both WEB-DL and WEBRip, so it picks nothing.
        assert_eq!(
            variant_exclusions("Show WEB AAC", "HEVC"),
            strings(&["HEVC"])
        );
    }

    #[test]
    fn variant_exclusions_respects_a_picked_variant() {
        for query in [
            "Show 1080p WEB-DL",
            "Show HEVC",
            "Show webrip",
            "Show -hevc",
            "Show -\"WEB-DL\"",
            "Show x265",
            "Show AVC",
        ] {
            assert!(variant_exclusions(query, "HEVC").is_empty(), "{query}");
        }
    }
}
