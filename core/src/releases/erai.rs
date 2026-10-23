//! Erai-raws specifics: search-term hygiene and per-show feed resolution. Ported from
//! anirss `erai.py`, without `render_query` (terminal colours).
//!
//! Erai's search endpoint (WordPress `s=` on the tokenized feed) is a plain word-AND over
//! post text, and the posts are titled "Show - 02 (NF)" with no quality tags at all. A query
//! like "Show 1080p WEB-DL" therefore zeroes the search out: the quality facets live in the
//! feed's erai: metadata instead (resolution, infomkv, linktype). `split_noise` strips those
//! tokens from a query before it goes on the wire and `matches_token` applies them to the
//! fetched results client-side.
//!
//! What Erai adds over a generic feed is a per-show feed at `/anime-list/<slug>/feed/`
//! that carries only that show's current offerings. The show slug is derived from the
//! release post URL each feed item links to, then validated with a single fetch: a wrong
//! slug returns HTTP 200 with an empty feed, so an empty probe means "fall back to the
//! search feed", never a hard failure.

use std::future::Future;
use std::sync::LazyLock;

use regex::Regex;

use super::Endpoint;
use super::endpoints::{FIELD_RE, parse_qsl, urlencode};
use crate::contract::{CoreError, Release};

const HOST: &str = "erai-raws.info";

/// Release post URL inside a feed item, for example
/// `https://www.erai-raws.info/episodes/<show-slug>-02/`, `.../encodes/<show-slug>-01-hevc/`
/// or `.../batches/<show-slug>-01-11/`.
static POST_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"https://www\.erai-raws\.info/(episodes|encodes|batches|specials|raws)/([a-z0-9-]+)/",
    )
    .expect("POST_RE compiles")
});

/// The suffix stripped from a post slug to recover the show slug. Only the section's own
/// suffix is removed so shows whose slug ends in a number (mob-psycho-100) survive:
/// episodes, specials and raws end in one episode number, encodes add -hevc after it,
/// batches end in a range.
static POST_SUFFIX_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-\d+(?:v\d+)?$").unwrap());
static POST_SUFFIX_ENCODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-\d+(?:v\d+)?-hevc$").unwrap());
static POST_SUFFIX_BATCH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-\d+-\d+$").unwrap());

fn post_suffix(section: &str) -> Option<&'static Regex> {
    match section {
        "episodes" | "specials" | "raws" => Some(&POST_SUFFIX_EPISODE),
        "encodes" => Some(&POST_SUFFIX_ENCODE),
        "batches" => Some(&POST_SUFFIX_BATCH),
        _ => None,
    }
}

pub fn is_erai(ep: &Endpoint) -> bool {
    ep.kind == "rss" && ep.url.contains(HOST)
}

/// Quality facets Erai keeps in its erai:resolution, infomkv, linktype and category
/// metadata instead of the post text its `s=` search matches. Whole-token only: "1080p" is
/// stripped from a query, "NF" (a real post-text platform tag) is not.
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

/// Erai posts every episode twice into the same feed: the AVC WEB-DL as it airs
/// ([Airing]) and an HEVC re-encode hours later ([Encoded]). No feed parameter separates
/// them, so a qB rule that names neither variant matches both and downloads each episode
/// twice.
static VARIANT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(?:avc|hevc|x[\s._-]?26[45]|h[\s._-]?26[45]|web[\s._-]?(?:dl|rip)|airing|encoded)$",
    )
    .expect("VARIANT_RE compiles")
});

/// Rule exclusions that keep a subscription to one release per episode: `["HEVC"]` (drop
/// the re-encode, keep the airing WEB-DL) unless a field of `query`, positive or
/// `-`excluded, already picks a variant.
pub fn variant_exclusions(query: &str) -> Vec<String> {
    let picked = FIELD_RE
        .find_iter(query)
        .any(|m| VARIANT_RE.is_match(field_core(m.as_str())));
    if picked {
        Vec::new()
    } else {
        vec!["HEVC".to_string()]
    }
}

/// (searchable terms, quality tokens) from a positive-terms query.
///
/// The quality tokens are the facets Erai exposes as erai: metadata; the `s=` search only
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

/// Does `token` match the item's structured erai fields? resolution, category and linktype
/// by equality, infomkv by substring. False for items without erai metadata (all fields
/// empty), so nyaa and generic feeds are unaffected.
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

/// Client-side facet match for one item: title text first, then the structured erai
/// fields. The polished title drops the linktype prefix, so "Torrent" only exists there.
pub fn matches_token(item: &Release, token: &str) -> bool {
    let lc = token.to_lowercase();
    item.title.to_lowercase().contains(&lc) || metadata_hit(item, &lc)
}

/// Every token matches (see `matches_token`).
pub fn matches_all(item: &Release, tokens: &[String]) -> bool {
    tokens.iter().all(|t| matches_token(item, t))
}

/// Show slug from a release post slug, `None` when nothing was stripped (an unrecognized
/// shape is not trusted as a show slug).
pub(crate) fn show_slug(section: &str, post_slug: &str) -> Option<String> {
    let suffix = post_suffix(section)?;
    let slug = suffix.replace_all(post_slug, "");
    (!slug.is_empty() && slug != post_slug).then(|| slug.into_owned())
}

/// The anime-list feed for `slug`, carrying over the endpoint URL's res, type, subs and
/// token params and dropping the `s=` search placeholder.
pub fn per_show_feed_url(ep: &Endpoint, slug: &str) -> String {
    let (base, qs) = ep.url.split_once('?').unwrap_or((&ep.url, ""));
    let params: Vec<(String, String)> = parse_qsl(qs)
        .into_iter()
        .filter(|(k, _)| k != "s")
        .collect();
    let origin = format!("{}{HOST}", base.split(HOST).next().unwrap_or(""));
    format!("{origin}/anime-list/{slug}/feed/?{}", urlencode(&params))
}

/// Per-show feed URL for the show `items` belong to, or `None` to make the caller fall
/// back to the search feed. Costs at most one request: the first item with a recognizable
/// post URL decides. `fetch` fetches and reads one feed URL.
pub async fn resolve_show_feed<F, Fut>(ep: &Endpoint, items: &[Release], fetch: F) -> Option<String>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Result<Vec<Release>, CoreError>>,
{
    for item in items {
        let Some(m) = POST_RE.captures(&item.page) else {
            continue;
        };
        let Some(slug) = show_slug(&m[1], &m[2]) else {
            continue;
        };
        let url = per_show_feed_url(ep, &slug);
        match fetch(url.clone()).await {
            Err(e) => {
                tracing::warn!("erai per-show probe failed: {e}");
                return None;
            }
            Ok(found) if !found.is_empty() => {
                tracing::info!("erai per-show feed for {slug:?}: {} item(s)", found.len());
                return Some(url);
            }
            Ok(_) => {
                tracing::warn!("erai slug {slug:?}: empty probe, using the search feed");
                return None;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::super::endpoints::tests::{erai_ep, rel};
    use super::*;
    use std::sync::Mutex;

    fn nyaa_ep() -> Endpoint {
        Endpoint::new("nyaa", "nyaa", "https://nyaa.si/")
    }

    const GAIKOTSU_PAGE: &str = "https://www.erai-raws.info/episodes/gaikotsu-kishi-sama-tadaima-isekai-e-odekake-chuu-ii-02/";

    fn erai_items() -> Vec<Release> {
        let mut item = rel(
            "Gaikotsu Kishi-sama Tadaima Isekai e Odekake-chuu II - 02 [1080p CR WEB-DL AVC AAC][Airing]",
            "l",
        );
        item.page = GAIKOTSU_PAGE.to_string();
        vec![item]
    }

    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn is_erai_needs_an_rss_endpoint_on_the_host() {
        assert!(is_erai(&erai_ep()));
        assert!(!is_erai(&nyaa_ep()));
        assert!(!is_erai(&Endpoint::new(
            "n",
            "nyaa",
            "https://erai-raws.info/"
        )));
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
    fn metadata_hit_ignores_items_without_erai_fields() {
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
        assert_eq!(
            show_slug("episodes", "gaikotsu-kishi-sama-02").unwrap(),
            "gaikotsu-kishi-sama"
        );
        // A show whose own slug ends in a number must survive the strip.
        assert_eq!(
            show_slug("episodes", "mob-psycho-100-05").unwrap(),
            "mob-psycho-100"
        );
        assert_eq!(show_slug("episodes", "show-02v2").unwrap(), "show");
        assert_eq!(
            show_slug("encodes", "gaikotsu-kishi-sama-01-hevc").unwrap(),
            "gaikotsu-kishi-sama"
        );
        assert_eq!(
            show_slug("batches", "86-eighty-six-01-11").unwrap(),
            "86-eighty-six"
        );
        // Nothing stripped, nothing left, or an unknown section is not trusted.
        assert_eq!(show_slug("episodes", "no-number-here"), None);
        assert_eq!(show_slug("episodes", "02"), None);
        assert_eq!(show_slug("elsewhere", "show-02"), None);
    }

    #[test]
    fn per_show_feed_url_carries_params_drops_search() {
        let url = per_show_feed_url(&erai_ep(), "86-eighty-six");
        assert!(url.starts_with("https://www.erai-raws.info/anime-list/86-eighty-six/feed/?"));
        assert!(url.contains("res=1080p") && url.contains("type=torrent"));
        assert!(url.contains("subs%5B0%5D=us") && url.contains("token=TOK"));
        assert!(!url.contains("{query}") && !url.contains("&s=") && !url.contains("?s="));
    }

    async fn resolve(
        items: &[Release],
        reply: Result<Vec<Release>, CoreError>,
    ) -> (Option<String>, Vec<String>) {
        let calls = Mutex::new(Vec::new());
        let url = resolve_show_feed(&erai_ep(), items, |url: String| {
            calls.lock().unwrap().push(url);
            let reply = reply.clone();
            async move { reply }
        })
        .await;
        (url, calls.into_inner().unwrap())
    }

    #[tokio::test]
    async fn resolve_show_feed_success() {
        let (url, calls) = resolve(&erai_items(), Ok(vec![rel("x", "l")])).await;
        let url = url.unwrap();
        assert!(
            url.contains("/anime-list/gaikotsu-kishi-sama-tadaima-isekai-e-odekake-chuu-ii/feed/")
        );
        assert_eq!(calls, vec![url]);
    }

    #[tokio::test]
    async fn resolve_show_feed_empty_probe_falls_back() {
        assert_eq!(resolve(&erai_items(), Ok(vec![])).await.0, None);
    }

    #[tokio::test]
    async fn resolve_show_feed_fetch_error_falls_back() {
        assert_eq!(
            resolve(&erai_items(), Err(CoreError::internal("boom")))
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
    async fn resolve_show_feed_skips_unrecognised_posts_to_the_next_item() {
        let mut odd = rel("t", "l");
        odd.page = "https://www.erai-raws.info/episodes/no-number-here/".to_string();
        let mut items = vec![odd];
        items.extend(erai_items());
        let (url, calls) = resolve(&items, Ok(vec![rel("x", "l")])).await;
        assert!(url.is_some());
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn variant_exclusions_neutral_query_drops_the_encode() {
        assert_eq!(variant_exclusions("Kyoufu 1080p"), strings(&["HEVC"]));
        // WEB alone matches both WEB-DL and WEBRip, so it picks nothing.
        assert_eq!(variant_exclusions("Show WEB AAC"), strings(&["HEVC"]));
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
            assert!(variant_exclusions(query).is_empty(), "{query}");
        }
    }
}
