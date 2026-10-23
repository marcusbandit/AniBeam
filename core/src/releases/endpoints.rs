//! Search endpoints: validation, URL building, fetch dispatch and the fallback probe.
//! Ported from anirss `endpoints.py`. The configured-list state (`EndpointState`) and the
//! on-demand start and stop commands stay out: every endpoint counts as usable here, so
//! the "off" note of `probe_fallback` never appears.

use std::future::Future;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;

use super::Endpoint;
use super::erai;
use super::fetch::{self, DEFAULT_TIMEOUT};
use crate::contract::{CoreError, Release, SubscriptionSpec};
use crate::net::{BoxFuture, Http};

/// anirss `endpoints.VALID_KINDS`.
pub const VALID_KINDS: [&str; 2] = ["nyaa", "rss"];

/// Splits a query into fields, keeping `-"quoted phrase"` and `"quoted"` intact
/// (anirss `endpoints.FIELD_RE`, shared with refine's positional-insert logic).
pub static FIELD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"-?"[^"]*"|\S+"#).expect("FIELD_RE compiles"));

/// A fetch timeout from an endpoint's seconds, clamped so a bad config value cannot panic.
fn seconds(s: f64) -> Duration {
    if s.is_nan() {
        return DEFAULT_TIMEOUT;
    }
    Duration::from_secs_f64(s.clamp(0.0, 3600.0))
}

/// Endpoint checks from anirss `load_endpoints`, minus the parts that need the whole list
/// (duplicate names) or the on-demand fields. Values are judged after trimming, like
/// anirss does before it builds the `Endpoint`.
pub fn validate(name: &str, kind: &str, url: &str) -> Result<(), CoreError> {
    let (name, kind, url) = (name.trim(), kind.trim(), url.trim());
    if name.is_empty() {
        return Err(CoreError::invalid("endpoint name", "missing `name`"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(CoreError::invalid(
            "endpoint name",
            format!("{name:?}: use only letters, digits, '.', '_', '-'"),
        ));
    }
    if !VALID_KINDS.contains(&kind) {
        return Err(CoreError::invalid(
            "endpoint kind",
            format!(
                "{name:?}: unknown kind {kind:?} (valid: {})",
                VALID_KINDS.join(", ")
            ),
        ));
    }
    if url.is_empty() {
        return Err(CoreError::invalid(
            "endpoint url",
            format!("{name:?}: missing `url`"),
        ));
    }
    if kind == "rss" && !url.contains("{query}") {
        return Err(CoreError::invalid(
            "endpoint url",
            format!("{name:?}: rss url must contain a {{query}} placeholder"),
        ));
    }
    Ok(())
}

// Python urllib.parse helpers the URL code leans on, kept byte for byte --------------

/// `urllib.parse.quote` over UTF-8 bytes: letters, digits and `_.-~` stay, as do the bytes
/// of `safe`; everything else becomes `%XX` in upper case.
pub(crate) fn quote(bytes: &[u8], safe: &str) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if b.is_ascii_alphanumeric()
            || matches!(b, b'_' | b'.' | b'-' | b'~')
            || safe.as_bytes().contains(&b)
        {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `urllib.parse.quote_plus`.
pub(crate) fn quote_plus(s: &str) -> String {
    quote(s.as_bytes(), " ").replace(' ', "+")
}

/// `urllib.parse.urlencode` for a list of pairs.
pub(crate) fn urlencode(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", quote_plus(k), quote_plus(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// `urllib.parse.unquote_plus`: `+` is a space, bad escapes stay, bad UTF-8 becomes U+FFFD.
pub(crate) fn unquote_plus(s: &str) -> String {
    let raw = s.replace('+', " ");
    let b = raw.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let digit = |c: u8| (c as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (digit(b[i + 1]), digit(b[i + 2])) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `urllib.parse.parse_qsl` with its defaults: pairs without `=` or with an empty value
/// are dropped.
pub(crate) fn parse_qsl(qs: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for chunk in qs.split('&') {
        if chunk.is_empty() {
            continue;
        }
        let Some((k, v)) = chunk.split_once('=') else {
            continue;
        };
        if v.is_empty() {
            continue;
        }
        out.push((unquote_plus(k), unquote_plus(v)));
    }
    out
}

/// What `urllib.parse.urlsplit` returns.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UrlParts {
    pub scheme: String,
    pub netloc: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

impl UrlParts {
    /// `urlunsplit` (`geturl`).
    pub fn geturl(&self) -> String {
        let mut url = self.path.clone();
        if !self.netloc.is_empty() {
            if !url.is_empty() && !url.starts_with('/') {
                url.insert(0, '/');
            }
            url = format!("//{}{url}", self.netloc);
        }
        if !self.scheme.is_empty() {
            url = format!("{}:{url}", self.scheme);
        }
        if !self.query.is_empty() {
            url = format!("{url}?{}", self.query);
        }
        if !self.fragment.is_empty() {
            url = format!("{url}#{}", self.fragment);
        }
        url
    }

    /// `parts.hostname`: lower case, userinfo and brackets gone.
    pub fn hostname(&self) -> Option<String> {
        let host = self
            .netloc
            .rsplit_once('@')
            .map_or(self.netloc.as_str(), |(_, h)| h);
        let host = if let Some(rest) = host.strip_prefix('[') {
            rest.split(']').next().unwrap_or("")
        } else {
            host.split(':').next().unwrap_or("")
        };
        (!host.is_empty()).then(|| host.to_lowercase())
    }

    /// `parts.port`: `None` when absent or not a valid port.
    pub fn port(&self) -> Option<u16> {
        let host = self
            .netloc
            .rsplit_once('@')
            .map_or(self.netloc.as_str(), |(_, h)| h);
        let after = if host.starts_with('[') {
            host.split_once(']')?.1
        } else {
            host.split_once(':').map_or("", |(_, p)| p)
        };
        let port = after.strip_prefix(':').unwrap_or(after);
        port.parse().ok()
    }
}

/// `urllib.parse.urlsplit`. Errors on an unbalanced `[` or `]` in the host, as Python does.
pub(crate) fn split_url(url: &str) -> Result<UrlParts, ()> {
    let cleaned: String = url
        .chars()
        .filter(|c| !matches!(c, '\t' | '\r' | '\n'))
        .collect();
    let mut rest = cleaned.trim_start_matches(|c: char| c <= ' ');
    let mut scheme = String::new();
    if let Some(i) = rest.find(':') {
        let head = &rest[..i];
        if i > 0
            && head.starts_with(|c: char| c.is_ascii_alphabetic())
            && head
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        {
            scheme = head.to_lowercase();
            rest = &rest[i + 1..];
        }
    }
    let mut netloc = "";
    if let Some(after) = rest.strip_prefix("//") {
        let end = after.find(['/', '?', '#']).unwrap_or(after.len());
        netloc = &after[..end];
        rest = &after[end..];
    }
    if (netloc.contains('[') && !netloc.contains(']'))
        || (netloc.contains(']') && !netloc.contains('['))
    {
        return Err(());
    }
    let (rest, fragment) = rest.split_once('#').unwrap_or((rest, ""));
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    Ok(UrlParts {
        scheme,
        netloc: netloc.to_string(),
        path: path.to_string(),
        query: query.to_string(),
        fragment: fragment.to_string(),
    })
}

// URLs and queries ---------------------------------------------------------------------

pub fn search_url(ep: &Endpoint, query: &str) -> String {
    if ep.kind == "nyaa" {
        let qs = urlencode(&[
            ("page".to_string(), "rss".to_string()),
            ("q".to_string(), query.to_string()),
            ("c".to_string(), ep.category.clone()),
            ("f".to_string(), ep.filter.clone()),
        ]);
        return format!("{}?{qs}", ep.url);
    }
    ep.url.replace("{query}", &quote_plus(query))
}

/// Split `query` into (positive-terms query, exclusion terms). An exclusion is a `-tag` or
/// `-"quoted phrase"` field (nyaa syntax). Generic RSS endpoints do not understand `-tag`,
/// so exclusions are applied client-side after the fetch instead.
pub fn split_exclusions(query: &str) -> (String, Vec<String>) {
    let mut positive: Vec<&str> = Vec::new();
    let mut excluded: Vec<String> = Vec::new();
    for field in FIELD_RE.find_iter(query).map(|m| m.as_str()) {
        if field.starts_with('-') && field.chars().count() > 1 {
            let mut core = &field[1..];
            if core.chars().count() >= 2 && core.starts_with('"') && core.ends_with('"') {
                core = &core[1..core.len() - 1];
            }
            if !core.is_empty() {
                excluded.push(core.to_string());
                continue;
            }
        }
        positive.push(field);
    }
    (positive.join(" "), excluded)
}

/// Erai's `s=` search matches post text only; the quality facets (1080p, WEB-DL, and so on)
/// live in the erai: metadata and would zero the search out, so they are stripped from the
/// terms here and returned for client-side filtering (`erai::split_noise`). Other endpoints
/// pass through untouched.
fn erai_split(ep: &Endpoint, positive: &str) -> (String, Vec<String>) {
    if ep.kind == "rss" && erai::is_erai(ep) {
        return erai::split_noise(positive);
    }
    (positive.to_string(), Vec::new())
}

/// URL to persist as a qBittorrent RSS feed for `query`, plus the exclusion terms that
/// could not be encoded into it. nyaa understands `-tag` exclusions server-side, so they
/// stay in the URL; generic rss endpoints get only the positive terms and the exclusions
/// come back for the caller to enforce through the qB rule. erai also loses its quality
/// tokens.
pub fn feed_url(ep: &Endpoint, query: &str) -> (String, Vec<String>) {
    if ep.kind == "nyaa" {
        return (search_url(ep, query), Vec::new());
    }
    let (positive, excluded) = split_exclusions(query);
    let (positive, _) = erai_split(ep, &positive);
    (search_url(ep, &positive), excluded)
}

/// The synchronous half of `subscription_feed`: everything once the per-show feed (the one
/// request erai may cost) is known. Returns (feed_url, mustContain, excluded).
///
/// Erai endpoints subscribe to the per-show anime-list feed when the slug resolved; the
/// positive query terms then move into the rule's mustContain since the per-show URL no
/// longer encodes the search. Quoted phrases are unwrapped and poster fields dropped: qB
/// matches whitespace-separated tokens against the raw feed titles, which carry no
/// `[group]` poster. When the slug did not resolve, the search feed is used with only the
/// searchable terms on the wire and the quality tokens moved into mustContain. Either way a
/// query that picks no AVC/HEVC variant gets the HEVC encode excluded. Every other endpoint
/// keeps the `feed_url` behaviour with an empty mustContain.
pub fn subscription_feed_resolved(
    ep: &Endpoint,
    query: &str,
    per_show: Option<String>,
) -> (String, String, Vec<String>) {
    if erai::is_erai(ep) {
        let (positive, mut excluded) = split_exclusions(query);
        excluded.extend(erai::variant_exclusions(query));
        if let Some(per_show) = per_show.filter(|u| !u.is_empty()) {
            let must = FIELD_RE
                .find_iter(&positive)
                .map(|m| m.as_str())
                .filter(|f| !f.starts_with('['))
                .map(|f| f.trim_matches('"'))
                .collect::<Vec<_>>()
                .join(" ");
            return (per_show, must, excluded);
        }
        let (positive, quality) = erai_split(ep, &positive);
        return (search_url(ep, &positive), quality.join(" "), excluded);
    }
    let (url, excluded) = feed_url(ep, query);
    (url, String::new(), excluded)
}

/// (feed_url, mustContain, excluded) to persist as a qB subscription. Erai endpoints cost
/// one probe request here, to resolve the per-show feed from `items`.
pub async fn subscription_feed(
    http: &dyn Http,
    ep: &Endpoint,
    query: &str,
    items: &[Release],
) -> (String, String, Vec<String>) {
    let per_show = if erai::is_erai(ep) {
        erai::resolve_show_feed(ep, items, |url: String| async move {
            fetch::fetch_rss(http, &url, "erai per-show probe", DEFAULT_TIMEOUT).await
        })
        .await
    } else {
        None
    };
    subscription_feed_resolved(ep, query, per_show)
}

/// `subscription_feed` packaged as what the subscribe call takes.
pub async fn build_subscription(
    http: &dyn Http,
    ep: &Endpoint,
    query: &str,
    items: &[Release],
) -> SubscriptionSpec {
    let (feed_url, must_contain, excluded) = subscription_feed(http, ep, query, items).await;
    SubscriptionSpec {
        feed_url,
        must_contain,
        must_not_contain: excluded.join("|"),
        endpoint_name: ep.name.clone(),
    }
}

/// Drop items whose title contains any excluded term (case-insensitive).
pub fn filter_excluded(items: Vec<Release>, terms: &[String]) -> Vec<Release> {
    if terms.is_empty() {
        return items;
    }
    let lc: Vec<String> = terms.iter().map(|t| t.to_lowercase()).collect();
    items
        .into_iter()
        .filter(|it| {
            let title = it.title.to_lowercase();
            !lc.iter().any(|t| title.contains(t.as_str()))
        })
        .collect()
}

/// Fetch `query` from `ep`. Errors are the feed's: unreachable, refused or unreadable.
pub async fn fetch_items(
    http: &dyn Http,
    ep: &Endpoint,
    query: &str,
) -> Result<Vec<Release>, CoreError> {
    let timeout = seconds(ep.timeout);
    if ep.kind == "nyaa" {
        return fetch::fetch_rss(http, &search_url(ep, query), &ep.name, timeout).await;
    }
    let (positive, excluded) = split_exclusions(query);
    let (positive, quality) = erai_split(ep, &positive);
    let items = fetch::fetch_rss(http, &search_url(ep, &positive), &ep.name, timeout).await?;
    let mut items = filter_excluded(items, &excluded);
    if !quality.is_empty() {
        // The quality facets cannot be searched server-side (they would zero the s= search
        // out); they narrow the results here instead.
        items.retain(|it| erai::matches_all(it, &quality));
    }
    Ok(items)
}

/// After the active endpoint came up empty or unreachable, try the other endpoints in
/// priority order. The first hit wins: its items come back tagged with its name in
/// `source`, so the winner is `items[0].source`. Returns (items, per-endpoint notes); empty
/// items means nothing anywhere.
pub async fn probe_fallback(
    http: &dyn Http,
    endpoints: &[Endpoint],
    active: &str,
    query: &str,
) -> (Vec<Release>, Vec<String>) {
    probe_with(
        endpoints,
        active,
        |ep| -> BoxFuture<'_, Result<Vec<Release>, CoreError>> {
            Box::pin(fetch_items(http, ep, query))
        },
    )
    .await
}

/// `probe_fallback` with the fetch injected, as anirss's `fetch=` argument is.
pub(crate) async fn probe_with<'a, F, Fut>(
    endpoints: &'a [Endpoint],
    active: &str,
    fetch: F,
) -> (Vec<Release>, Vec<String>)
where
    F: Fn(&'a Endpoint) -> Fut,
    Fut: Future<Output = Result<Vec<Release>, CoreError>>,
{
    let mut notes = Vec::new();
    for ep in endpoints {
        if ep.name == active {
            continue;
        }
        let items = match fetch(ep).await {
            Ok(items) => items,
            Err(_) => {
                notes.push(format!("{}: unreachable", ep.name));
                continue;
            }
        };
        if !items.is_empty() {
            notes.push(format!("{}: {}", ep.name, items.len()));
            // Tag where these came from so the results list can render a source header
            // above them: the search did not ask for this endpoint, it fell back to it.
            let tagged = items
                .into_iter()
                .map(|mut it| {
                    it.source = ep.name.clone();
                    it
                })
                .collect();
            return (tagged, notes);
        }
        notes.push(format!("{}: 0", ep.name));
    }
    (Vec::new(), notes)
}

/// The configured endpoint whose feed `url` is, if any.
pub fn owner_of<'a>(endpoints: &'a [Endpoint], url: &str) -> Option<&'a Endpoint> {
    endpoints.iter().find(|e| e.owns_url(url))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::net::FakeHttp;

    fn nyaa() -> Endpoint {
        Endpoint {
            category: "1_2".to_string(),
            filter: "1".to_string(),
            ..Endpoint::new("nyaa", "nyaa", "https://nyaa.si/")
        }
    }

    fn anirena() -> Endpoint {
        Endpoint::new(
            "anirena",
            "rss",
            "https://www.anirena.com/rss?q={query}&adult=1",
        )
    }

    pub(crate) fn erai_ep() -> Endpoint {
        Endpoint::new(
            "erai",
            "rss",
            "https://www.erai-raws.info/episodes/feed/?res=1080p&type=torrent&subs%5B0%5D=us&s={query}&token=TOK",
        )
    }

    pub(crate) fn rel(title: &str, link: &str) -> Release {
        Release {
            title: title.to_string(),
            link: link.to_string(),
            seeders: None,
            leechers: None,
            downloads: None,
            size: String::new(),
            category: String::new(),
            page: String::new(),
            source: String::new(),
            resolution: String::new(),
            infomkv: String::new(),
            linktype: String::new(),
        }
    }

    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    /// The value of the `s=` search parameter, the only part a query can pollute.
    fn s_param(url: &str) -> String {
        let qs = url.split_once('?').map_or("", |(_, q)| q);
        parse_qsl(qs)
            .into_iter()
            .find(|(k, _)| k == "s")
            .map(|(_, v)| v)
            .unwrap_or_default()
    }

    fn feed(items: &[(&str, &str)]) -> String {
        let body: String = items
            .iter()
            .map(|(t, l)| format!("<item><title>{t}</title><link>{l}</link></item>"))
            .collect();
        format!(r#"<rss version="2.0"><channel>{body}</channel></rss>"#)
    }

    #[test]
    fn validate_accepts_good_endpoints() {
        validate("nyaa", "nyaa", "https://nyaa.si/").unwrap();
        validate(
            "anirena",
            "rss",
            "https://www.anirena.com/rss?q={query}&adult=1",
        )
        .unwrap();
        validate(" a.b_c-1 ", " rss ", " https://x/?q={query} ").unwrap();
    }

    #[test]
    fn validate_rejects_bad_endpoints() {
        for (name, kind, url) in [
            ("", "nyaa", "https://x/"),          // no name
            ("a", "html", "https://x/"),         // bad kind
            ("a", "rss", "https://x/rss"),       // no {query}
            ("a", "nyaa", ""),                   // no url
            ("bad name)", "nyaa", "https://x/"), // bad name characters
        ] {
            let err = validate(name, kind, url).unwrap_err();
            assert!(
                matches!(err, CoreError::Invalid { .. }),
                "{name:?} {kind:?} {url:?}"
            );
        }
    }

    #[test]
    fn search_url_nyaa_kind() {
        let url = search_url(&nyaa(), "one piece");
        assert!(url.starts_with("https://nyaa.si/?"));
        assert!(url.contains("page=rss") && url.contains("q=one+piece"));
        assert!(url.contains("c=1_2") && url.contains("f=1"));
        assert_eq!(url, "https://nyaa.si/?page=rss&q=one+piece&c=1_2&f=1");
    }

    #[test]
    fn search_url_rss_kind_fills_template() {
        assert_eq!(
            search_url(&anirena(), "shin chan"),
            "https://www.anirena.com/rss?q=shin+chan&adult=1"
        );
    }

    #[test]
    fn search_url_quotes_like_python() {
        // quote_plus keeps `_.-~`, turns a space into `+` and escapes the rest as UTF-8.
        assert_eq!(quote_plus("a b_c.d-e~f"), "a+b_c.d-e~f");
        assert_eq!(
            quote_plus("[x] \"y\" & z/\u{e9}"),
            "%5Bx%5D+%22y%22+%26+z%2F%C3%A9"
        );
        assert_eq!(quote(b"a/b c", "/"), "a/b%20c");
    }

    #[test]
    fn unquote_and_parse_qsl_like_python() {
        assert_eq!(unquote_plus("a+b%20c%5B0%5D%zz%4"), "a b c[0]%zz%4");
        assert_eq!(unquote_plus("%C3%A9"), "\u{e9}");
        // Pairs without `=` or with an empty value are dropped.
        assert_eq!(
            parse_qsl("a=1&b&c=&&d=x%20y&=z"),
            vec![
                ("a".to_string(), "1".to_string()),
                ("d".to_string(), "x y".to_string()),
                ("".to_string(), "z".to_string()),
            ]
        );
    }

    #[test]
    fn split_url_reads_the_parts() {
        let p = split_url("HTTP://user@Host.example:8080/a b?x=1#frag").unwrap();
        assert_eq!(p.scheme, "http");
        assert_eq!(p.netloc, "user@Host.example:8080");
        assert_eq!(
            (p.path.as_str(), p.query.as_str(), p.fragment.as_str()),
            ("/a b", "x=1", "frag")
        );
        assert_eq!(p.hostname().as_deref(), Some("host.example"));
        assert_eq!(p.port(), Some(8080));
        assert_eq!(p.geturl(), "http://user@Host.example:8080/a b?x=1#frag");
        assert!(split_url("http://[::1/x").is_err());
        let v6 = split_url("udp://[2001:db8::1]:1337/announce").unwrap();
        assert_eq!(v6.hostname().as_deref(), Some("2001:db8::1"));
        assert_eq!(v6.port(), Some(1337));
    }

    #[test]
    fn feed_url_nyaa_kind_keeps_exclusions_in_url() {
        let (url, excluded) = feed_url(&nyaa(), "one piece -HEVC");
        assert_eq!(url, search_url(&nyaa(), "one piece -HEVC"));
        assert!(excluded.is_empty());
    }

    #[test]
    fn feed_url_rss_kind_strips_exclusions() {
        let (url, excluded) = feed_url(&anirena(), r#"show 1080p -HEVC -"dual audio""#);
        assert!(!url.contains("HEVC") && !url.contains("dual"));
        assert_eq!(url, search_url(&anirena(), "show 1080p"));
        assert_eq!(excluded, strings(&["HEVC", "dual audio"]));
    }

    #[test]
    fn feed_url_rss_kind_no_exclusions_roundtrips() {
        let (url, excluded) = feed_url(&anirena(), "shin chan");
        assert_eq!(url, search_url(&anirena(), "shin chan"));
        assert!(excluded.is_empty());
    }

    #[test]
    fn split_exclusions_splits_positive_from_negative() {
        let (q, excl) = split_exclusions(r#"show 1080p -HEVC -"dual audio""#);
        assert_eq!(q, "show 1080p");
        assert_eq!(excl, strings(&["HEVC", "dual audio"]));
    }

    #[test]
    fn split_exclusions_no_exclusions_roundtrip() {
        let (q, excl) = split_exclusions("[Erai-raws] show 1080p");
        assert_eq!(q, "[Erai-raws] show 1080p");
        assert!(excl.is_empty());
    }

    #[test]
    fn split_exclusions_edge_fields() {
        // A bare dash and an empty quoted exclusion stay positive; an unterminated quote
        // splits at the space and keeps its half as typed.
        let (q, excl) = split_exclusions(r#"a - -"" -"b c"#);
        assert_eq!(q, r#"a - -"" c"#);
        assert_eq!(excl, strings(&["\"b"]));
    }

    #[test]
    fn filter_excluded_case_insensitive() {
        let items = vec![rel("Show 05 HEVC x265", "l1"), rel("Show 05 AVC", "l2")];
        let kept = filter_excluded(items, &strings(&["hevc"]));
        assert_eq!(
            kept.iter().map(|i| i.link.as_str()).collect::<Vec<_>>(),
            ["l2"]
        );
    }

    #[tokio::test]
    async fn fetch_items_rss_kind_applies_exclusions() {
        let http = FakeHttp::new();
        http.push(200, feed(&[("Show 05 HEVC", "l1"), ("Show 05 AVC", "l2")]));
        let items = fetch_items(http.as_ref(), &anirena(), "show -HEVC")
            .await
            .unwrap();
        assert_eq!(
            items.iter().map(|i| i.link.as_str()).collect::<Vec<_>>(),
            ["l2"]
        );
        // The exclusion never reaches the wire; only positive terms are sent.
        let url = &http.requests()[0].url;
        assert!(!url.contains("HEVC"), "{url}");
        assert_eq!(url, "https://www.anirena.com/rss?q=show&adult=1");
    }

    #[tokio::test]
    async fn fetch_items_nyaa_kind_sends_exclusions() {
        let http = FakeHttp::new();
        http.push(200, feed(&[("t", "l")]));
        let items = fetch_items(http.as_ref(), &nyaa(), "show -HEVC")
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert!(unquote_plus(&http.requests()[0].url).contains("-HEVC"));
    }

    #[tokio::test]
    async fn fetch_items_sends_a_browser_user_agent_and_the_endpoint_timeout() {
        let http = FakeHttp::new();
        http.push(200, feed(&[]));
        let mut ep = nyaa();
        ep.timeout = 60.0;
        fetch_items(http.as_ref(), &ep, "x").await.unwrap();
        let req = &http.requests()[0];
        assert_eq!(req.timeout, Some(Duration::from_secs(60)));
        assert!(
            req.headers
                .iter()
                .any(|(k, v)| k == "User-Agent" && v.contains("Firefox"))
        );
    }

    #[tokio::test]
    async fn fetch_items_erai_searches_clean_and_filters_quality() {
        let xml = r#"<rss version="2.0" xmlns:erai="https://www.erai-raws.info/rss-page/"><channel>
<item><title>[Torrent] Ranma - 01 [1080p NF WEB-DL AVC AAC][Airing]</title><link>dl</link>
<erai:resolution>1080p</erai:resolution><erai:infomkv>NF WEB-DL AVC AAC</erai:infomkv><erai:linktype>Torrent</erai:linktype></item>
<item><title>[Torrent] Ranma - 01 (HEVC) [1080p NF WEBRip HEVC AAC][Encoded]</title><link>rip</link>
<erai:resolution>1080p</erai:resolution><erai:infomkv>NF WEBRip HEVC AAC</erai:infomkv><erai:linktype>Torrent</erai:linktype></item>
</channel></rss>"#;
        let http = FakeHttp::new();
        http.push(200, xml);
        let items = fetch_items(http.as_ref(), &erai_ep(), "Ranma 1080p WEB-DL")
            .await
            .unwrap();
        // The search went out with the show name only, since the quality tokens would zero
        // it out, and the facet was applied to the results client-side.
        assert_eq!(s_param(&http.requests()[0].url), "Ranma");
        assert_eq!(
            items.iter().map(|i| i.link.as_str()).collect::<Vec<_>>(),
            ["dl"]
        );
    }

    #[tokio::test]
    async fn fetch_items_failure_names_the_endpoint() {
        let http = FakeHttp::new();
        http.fail_next("boom");
        let err = fetch_items(http.as_ref(), &anirena(), "q")
            .await
            .unwrap_err();
        match err {
            CoreError::Service {
                service,
                code,
                message,
            } => {
                assert_eq!(
                    (service.as_str(), code.as_str(), message.as_str()),
                    ("anirena", "unreachable", "boom")
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[tokio::test]
    async fn probe_fallback_switches_to_first_hit() {
        let eps = [nyaa(), anirena()];
        let (items, notes) = probe_with(&eps, "nyaa", |ep| async move {
            Ok(if ep.name == "anirena" {
                vec![rel("t", "l")]
            } else {
                vec![]
            })
        })
        .await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].source, "anirena");
        assert_eq!(notes, strings(&["anirena: 1"]));
    }

    #[tokio::test]
    async fn probe_fallback_tags_items_with_source() {
        let eps = [nyaa(), anirena()];
        let (items, _) = probe_with(&eps, "nyaa", |ep| async move {
            Ok(if ep.name == "anirena" {
                vec![rel("t", "l"), rel("t2", "l2")]
            } else {
                vec![]
            })
        })
        .await;
        assert_eq!(
            items.iter().map(|i| i.source.as_str()).collect::<Vec<_>>(),
            ["anirena", "anirena"]
        );
    }

    #[tokio::test]
    async fn probe_fallback_all_fail_returns_nothing() {
        let third = Endpoint::new("tosho", "rss", "https://x/?q={query}");
        let eps = [nyaa(), anirena(), third];
        let (items, notes) = probe_with(&eps, "nyaa", |ep| async move {
            if ep.name == "anirena" {
                Err(CoreError::internal("can't reach anirena: boom"))
            } else {
                Ok(vec![])
            }
        })
        .await;
        assert!(items.is_empty());
        assert_eq!(notes, strings(&["anirena: unreachable", "tosho: 0"]));
    }

    #[tokio::test]
    async fn probe_fallback_over_http_skips_the_active_endpoint() {
        let eps = [nyaa(), anirena()];
        let http = FakeHttp::new();
        http.push_for("anirena.com", 200, feed(&[("Show", "l")]));
        let (items, notes) = probe_fallback(http.as_ref(), &eps, "nyaa", "show").await;
        assert_eq!(items[0].source, "anirena");
        assert_eq!(notes, strings(&["anirena: 1"]));
        assert_eq!(http.requests().len(), 1);
    }

    #[test]
    fn owner_of_finds_the_endpoint_by_url_prefix() {
        let eps = [nyaa(), anirena()];
        let owner = owner_of(&eps, "https://www.anirena.com/rss?q=x&adult=1").unwrap();
        assert_eq!(owner.name, "anirena");
        assert!(owner_of(&eps, "https://elsewhere.example/").is_none());
    }

    // The erai subscription rules (anirss test_erai.py) ----------------------------------

    #[test]
    fn feed_url_erai_strips_quality_tokens() {
        let (url, excluded) = feed_url(&erai_ep(), "Ranma 1080p WEB-DL -hevc");
        assert_eq!(s_param(&url), "Ranma");
        assert_eq!(excluded, strings(&["hevc"]));
    }

    #[test]
    fn subscription_feed_erai_fallback_puts_quality_in_must() {
        let (url, must, excluded) =
            subscription_feed_resolved(&erai_ep(), "Ranma 1080p WEB-DL -hevc", None);
        assert_eq!(s_param(&url), "Ranma");
        // qB enforces the quality facets against the raw feed titles instead.
        assert_eq!(must, "1080p WEB-DL");
        assert_eq!(excluded, strings(&["hevc"]));
    }

    #[test]
    fn subscription_feed_erai_uses_per_show_and_must_contain() {
        let per_show = "https://www.erai-raws.info/anime-list/x/feed/?res=1080p".to_string();
        let (url, must, excluded) = subscription_feed_resolved(
            &erai_ep(),
            "Gaikotsu 1080p CR WEB-DL -hevc",
            Some(per_show.clone()),
        );
        assert_eq!(url, per_show);
        assert_eq!(must, "Gaikotsu 1080p CR WEB-DL");
        assert_eq!(excluded, strings(&["hevc"]));
    }

    #[test]
    fn subscription_feed_must_contain_drops_poster_and_quotes() {
        let (_, must, _) = subscription_feed_resolved(
            &erai_ep(),
            r#"[Erai-raws] Show "Multi Subs" 1080p"#,
            Some("https://e/f".to_string()),
        );
        assert_eq!(must, "Show Multi Subs 1080p");
    }

    #[test]
    fn subscription_feed_erai_falls_back_to_search_feed() {
        let (url, must, excluded) = subscription_feed_resolved(&erai_ep(), "gaikotsu -hevc", None);
        assert!(url.contains("s=gaikotsu"));
        assert_eq!(must, "");
        assert_eq!(excluded, strings(&["hevc"]));
    }

    #[test]
    fn subscription_feed_non_erai_unchanged() {
        let (url, must, excluded) = subscription_feed_resolved(&nyaa(), "show -bad", None);
        assert_eq!(url, feed_url(&nyaa(), "show -bad").0);
        assert_eq!(must, "");
        assert!(excluded.is_empty());
    }

    // The two releases Erai's feed carries for every episode (Kyoufu Collector, whose
    // `kyoufu 1080p` rule downloaded both).
    const KYOUFU_AVC: &str = "[Torrent] Kyoufu Collector - 01 [1080p CR WEB-DL AVC AAC][us][br][mx][es][sa][fr][de][it][ru][pl][Airing]";
    const KYOUFU_HEVC: &str = "[Torrent] Kyoufu Collector - 01 (HEVC) [1080p CR WEBRip HEVC AAC][us][br][mx][es][sa][fr][de][it][ru][pl][Encoded]";

    /// qB's non-regex rule match: every mustContain token and none of the mustNotContain
    /// expressions, as case-insensitive substrings.
    fn qb_accepts(title: &str, must: &str, excluded: &[String]) -> bool {
        let lc = title.to_lowercase();
        must.split_whitespace()
            .all(|t| lc.contains(&t.to_lowercase()))
            && !excluded.iter().any(|x| lc.contains(&x.to_lowercase()))
    }

    fn kept(query: &str) -> (Vec<&'static str>, Vec<String>) {
        let (_, must, excluded) =
            subscription_feed_resolved(&erai_ep(), query, Some("https://e/f".to_string()));
        let kept = [KYOUFU_AVC, KYOUFU_HEVC]
            .into_iter()
            .filter(|t| qb_accepts(t, &must, &excluded))
            .collect();
        (kept, excluded)
    }

    #[test]
    fn subscription_feed_per_show_downloads_one_release_per_episode() {
        let (kept, excluded) = kept("kyoufu 1080p");
        assert_eq!(excluded, strings(&["HEVC"]));
        assert_eq!(kept, [KYOUFU_AVC]);
    }

    #[test]
    fn subscription_feed_hevc_query_keeps_only_the_encode() {
        let (kept, excluded) = kept("kyoufu 1080p HEVC");
        assert!(excluded.is_empty());
        assert_eq!(kept, [KYOUFU_HEVC]);
    }

    #[test]
    fn subscription_feed_search_fallback_also_drops_the_encode() {
        let (_, _, excluded) = subscription_feed_resolved(&erai_ep(), "Ranma 1080p", None);
        assert_eq!(excluded, strings(&["HEVC"]));
    }

    #[tokio::test]
    async fn build_subscription_probes_the_per_show_feed_once() {
        let mut page_item = rel("Gaikotsu - 02 [1080p][Airing]", "l");
        page_item.page = "https://www.erai-raws.info/episodes/gaikotsu-kishi-sama-02/".to_string();
        let http = FakeHttp::new();
        http.push(200, feed(&[("x", "l")]));
        let spec = build_subscription(
            http.as_ref(),
            &erai_ep(),
            "Gaikotsu 1080p -dub",
            &[page_item],
        )
        .await;
        assert!(
            spec.feed_url
                .starts_with("https://www.erai-raws.info/anime-list/gaikotsu-kishi-sama/feed/?")
        );
        assert_eq!(spec.must_contain, "Gaikotsu 1080p");
        assert_eq!(spec.must_not_contain, "dub|HEVC");
        assert_eq!(spec.endpoint_name, "erai");
        assert_eq!(http.requests().len(), 1);
    }

    #[tokio::test]
    async fn build_subscription_for_a_plain_endpoint_makes_no_request() {
        let http = FakeHttp::new();
        let spec = build_subscription(http.as_ref(), &anirena(), "show -bad -worse", &[]).await;
        assert_eq!(spec.feed_url, "https://www.anirena.com/rss?q=show&adult=1");
        assert_eq!(spec.must_not_contain, "bad|worse");
        assert!(http.requests().is_empty());
    }
}
