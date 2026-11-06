//! Search endpoints: validation, URL building, fetch dispatch and the fallback probe.
//! Ported from anirss `endpoints.py`. The configured-list state (`EndpointState`) and the
//! on-demand start and stop commands stay out: every endpoint counts as usable here, so
//! the "off" note of `probe_fallback` never appears.

use std::future::Future;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;

use super::Endpoint;
use super::fetch::{self, DEFAULT_TIMEOUT};
use super::kinds::{FILTERED_RSS, JSON_FEED};
use super::quirks;
use crate::contract::{CoreError, Release, SubscriptionSpec};
use crate::net::{BoxFuture, Http};

/// The kinds an endpoint can be, each named for the shape it reads (`kinds.rs`).
/// `filtered-rss` and `rss` are anirss's kinds under shape names. `json-feed` is AniBeam's
/// own: a JSON feed in one fixed layout (json_feed.rs). So is `json`: any JSON search API,
/// read through the field map its repo definition supplies (json.rs). `torznab` is an
/// indexer's torznab search, read as RSS; it may name the person's own server as `{host}`.
pub const VALID_KINDS: [&str; 5] = ["filtered-rss", "json", "json-feed", "rss", "torznab"];

/// Whether an endpoint of `kind` searches by putting the query into its address at
/// `{query}`. filtered-rss builds its own search; rss and json-feed have only the address, so without
/// the placeholder every search would fetch the same fixed page (#248). Every check of an
/// endpoint or a phone source asks this. A json endpoint has only the address too, and
/// so does a torznab search.
pub fn needs_query(kind: &str) -> bool {
    matches!(kind, "json" | "json-feed" | "rss" | "torznab")
}

/// Whether what an endpoint of `kind` answers at its search address is an RSS feed, which
/// `downloads::feed::read` can read. json and json-feed answer JSON; every other kind is read
/// as RSS, the way `fetch_exact` falls through to RSS for a kind it has no branch for.
pub fn answers_rss(kind: &str) -> bool {
    !matches!(kind, "json" | "json-feed")
}

/// Why a json source cannot be added by hand: its field map only comes from a definition.
pub const JSON_NEEDS_REPO: &str =
    "a JSON search source comes from an endpoint repo whose definition maps its fields";

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
/// anirss does before it builds the `Endpoint`. A phone source gets the same (#260).
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
    check_shape(name, kind, url)
}

/// The part of `validate` a source saved earlier is held to before it is searched: a known
/// kind other than json (its field map only comes from a repo), an address, and `{query}`
/// exactly where the kind wants it. A kind that builds its
/// own search takes none, or the search would go in twice.
pub fn check_shape(name: &str, kind: &str, url: &str) -> Result<(), CoreError> {
    let (name, kind, url) = (name.trim(), kind.trim(), url.trim());
    if kind == "json" {
        return Err(CoreError::invalid(
            "endpoint kind",
            format!("{name:?}: {JSON_NEEDS_REPO}"),
        ));
    }
    if !VALID_KINDS.contains(&kind) {
        return Err(CoreError::invalid(
            "endpoint kind",
            format!(
                "{name:?}: unknown kind {kind:?} (valid: {})",
                VALID_KINDS
                    .iter()
                    .filter(|k| **k != "json")
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    if url.is_empty() {
        return Err(CoreError::invalid(
            "endpoint url",
            format!("{name:?}: missing `url`"),
        ));
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(CoreError::invalid(
            "endpoint url",
            format!("{name:?}: the address must start with http:// or https://"),
        ));
    }
    if needs_query(kind) && !url.contains("{query}") {
        return Err(CoreError::invalid(
            "endpoint url",
            format!("{name:?}: {kind} url must contain a {{query}} placeholder"),
        ));
    }
    if !needs_query(kind) && url.contains("{query}") {
        return Err(CoreError::invalid(
            "endpoint url",
            format!("{name:?}: a {kind} url adds the search itself, so it takes no {{query}}"),
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

/// Tags results a filtered-rss endpoint found only with its filter off, after the endpoint's
/// name: the results list heads them with it, and a subscription to them keeps the filter off.
pub const ALL_UPLOADS: &str = ": all uploads";

/// `ep` with its filter off: every upload, trusted or not.
fn unfiltered(ep: &Endpoint) -> Endpoint {
    Endpoint {
        filter: "0".to_string(),
        ..ep.clone()
    }
}

/// Whether `items` are what an endpoint found only with its filter off.
pub fn found_unfiltered(items: &[Release]) -> bool {
    !items.is_empty() && items.iter().all(|it| it.source.ends_with(ALL_UPLOADS))
}

pub fn search_url(ep: &Endpoint, query: &str) -> String {
    if ep.kind == FILTERED_RSS {
        let qs = urlencode(&[
            ("page".to_string(), "rss".to_string()),
            ("q".to_string(), query.to_string()),
            ("c".to_string(), ep.category.clone()),
            ("f".to_string(), ep.filter.clone()),
        ]);
        // A nyaa address may carry fixed parameters of its own (a token), as anirss allows
        let sep = if ep.url.contains('?') { '&' } else { '?' };
        return format!("{}{sep}{qs}", ep.url);
    }
    ep.url.replace("{query}", &quote_plus(query))
}

/// Split `query` into (positive-terms query, exclusion terms). An exclusion is a `-tag` or
/// `-"quoted phrase"` field (filtered-rss syntax). Generic RSS endpoints do not understand `-tag`,
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

/// A `quality_local` source's search matches post text only; the quality facets (1080p,
/// WEB-DL, and so on) live in its feed's extension fields and would zero the search out, so
/// they are stripped from the terms here and returned for client-side filtering
/// (`quirks::split_noise`). Other endpoints pass through untouched.
fn quality_split(ep: &Endpoint, positive: &str) -> (String, Vec<String>) {
    if ep.kind == "rss" && ep.quirks.quality_local {
        return quirks::split_noise(positive);
    }
    (positive.to_string(), Vec::new())
}

/// URL to persist as a qBittorrent RSS feed for `query`, plus the exclusion terms that
/// could not be encoded into it. filtered-rss understands `-tag` exclusions server-side, so they
/// stay in the URL; generic rss endpoints get only the positive terms and the exclusions
/// come back for the caller to enforce through the qB rule. A `quality_local` source also
/// loses its quality tokens.
pub fn feed_url(ep: &Endpoint, query: &str) -> (String, Vec<String>) {
    if ep.kind == FILTERED_RSS {
        return (search_url(ep, query), Vec::new());
    }
    let (positive, excluded) = split_exclusions(query);
    let (positive, _) = quality_split(ep, &positive);
    (search_url(ep, &positive), excluded)
}

/// The synchronous half of `subscription_feed`: everything once the show's own feed (the one
/// request a `show_feed` source may cost) is known. Returns (feed_url, mustContain, excluded).
///
/// A source whose definition narrows its rule (`Quirks::narrows_in_rule`) subscribes to the
/// show's own feed when the slug resolved; the positive query terms then move into the
/// rule's mustContain since that feed's URL no longer encodes the search. Quoted phrases are
/// unwrapped and poster fields dropped: qB matches whitespace-separated tokens against the
/// raw feed titles, which carry no `[group]` poster. When the slug did not resolve, the
/// search feed is used with only the searchable terms on the wire and the quality tokens
/// moved into mustContain. Either way a query that picks no variant gets the definition's
/// `one_variant` excluded. Every other endpoint keeps the `feed_url` behaviour with an empty
/// mustContain.
pub fn subscription_feed_resolved(
    ep: &Endpoint,
    query: &str,
    per_show: Option<String>,
) -> (String, String, Vec<String>) {
    if ep.kind == "rss" && ep.quirks.narrows_in_rule() {
        let (positive, mut excluded) = split_exclusions(query);
        if let Some(word) = &ep.quirks.one_variant {
            excluded.extend(quirks::variant_exclusions(query, word));
        }
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
        let (positive, quality) = quality_split(ep, &positive);
        return (search_url(ep, &positive), quality.join(" "), excluded);
    }
    let (url, excluded) = feed_url(ep, query);
    (url, String::new(), excluded)
}

/// (feed_url, mustContain, excluded) to persist as a qB subscription. A `show_feed` source
/// costs one probe request here, to resolve the show's own feed from `items`.
pub async fn subscription_feed(
    http: &dyn Http,
    ep: &Endpoint,
    query: &str,
    items: &[Release],
) -> (String, String, Vec<String>) {
    let relaxed;
    let ep = if ep.kind == FILTERED_RSS && found_unfiltered(items) {
        relaxed = unfiltered(ep);
        &relaxed
    } else {
        ep
    };
    let per_show = if ep.kind == "rss" {
        quirks::resolve_show_feed(ep, items, |url: String| async move {
            fetch::fetch_rss(http, &url, &ep.name, DEFAULT_TIMEOUT).await
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

/// Fetch `query` from `ep`. Errors are the feed's: unreachable, refused or unreadable. A
/// filtered-rss endpoint whose filter finds nothing is asked again unfiltered, and those results carry
/// `source` "<name>: all uploads". When that second request fails, the answer stays the
/// first one: nothing found.
pub async fn fetch_items(
    http: &dyn Http,
    ep: &Endpoint,
    query: &str,
) -> Result<Vec<Release>, CoreError> {
    let items = fetch_exact(http, ep, query).await?;
    if ep.kind != FILTERED_RSS || !items.is_empty() || matches!(ep.filter.as_str(), "" | "0") {
        return Ok(items);
    }
    // A show nobody trusted has uploaded (or that trusted groups name differently)
    // would vanish behind "trusted only": ask again for every upload, and say so.
    Ok(fetch_all_uploads(http, ep, query).await.unwrap_or_default())
}

/// `query` from `ep` with its filter off, tagged "<name>: all uploads".
async fn fetch_all_uploads(
    http: &dyn Http,
    ep: &Endpoint,
    query: &str,
) -> Result<Vec<Release>, CoreError> {
    let items = fetch_exact(http, &unfiltered(ep), query).await?;
    let tag = format!("{}{ALL_UPLOADS}", ep.name);
    Ok(items
        .into_iter()
        .map(|mut it| {
            it.source = tag.clone();
            it
        })
        .collect())
}

/// What a refine refetches with: the filter the list it narrows was found with
/// (`all_uploads`, from `found_unfiltered`), and no unfiltered retry, so narrowing a trusted
/// list never comes back with untrusted uploads.
pub async fn refetch(
    http: &dyn Http,
    ep: &Endpoint,
    query: &str,
    all_uploads: bool,
) -> Result<Vec<Release>, CoreError> {
    if ep.kind == FILTERED_RSS && all_uploads {
        return fetch_all_uploads(http, ep, query).await;
    }
    fetch_exact(http, ep, query).await
}

/// One fetch of `query` from `ep`, exactly as configured.
async fn fetch_exact(
    http: &dyn Http,
    ep: &Endpoint,
    query: &str,
) -> Result<Vec<Release>, CoreError> {
    let timeout = seconds(ep.timeout);
    if ep.kind == FILTERED_RSS {
        return fetch::fetch_rss(http, &search_url(ep, query), &ep.name, timeout).await;
    }
    if ep.kind == JSON_FEED {
        // Exclusions are applied here: a json-feed search does not take `-tag`.
        let (positive, excluded) = split_exclusions(query);
        let items =
            super::json_feed::fetch(http, &search_url(ep, &positive), &ep.name, timeout).await?;
        return Ok(filter_excluded(items, &excluded));
    }
    if ep.kind == "json" {
        // Which search syntax an API takes is the site's business, so exclusions are applied
        // here rather than sent.
        let Some(map) = &ep.json else {
            return Err(CoreError::Service {
                service: ep.name.clone(),
                code: "unmapped".into(),
                message: JSON_NEEDS_REPO.into(),
            });
        };
        let (positive, excluded) = split_exclusions(query);
        let items =
            super::json::fetch(http, &search_url(ep, &positive), map, &ep.name, timeout).await?;
        return Ok(filter_excluded(items, &excluded));
    }
    if ep.kind == "torznab" {
        // Prowlarr and Jackett do not take `-tag` either, and keep every quality term.
        let (positive, excluded) = split_exclusions(query);
        let items = fetch::fetch_rss(http, &search_url(ep, &positive), &ep.name, timeout).await?;
        return Ok(filter_excluded(items, &excluded));
    }
    let (positive, excluded) = split_exclusions(query);
    let (positive, quality) = quality_split(ep, &positive);
    let items = fetch::fetch_rss(http, &search_url(ep, &positive), &ep.name, timeout).await?;
    let mut items = filter_excluded(items, &excluded);
    if !quality.is_empty() {
        // The quality facets cannot be searched server-side (they would zero the s= search
        // out); they narrow the results here instead.
        items.retain(|it| quirks::matches_all(it, &quality));
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
                    if it.source.is_empty() {
                        it.source = ep.name.clone();
                    }
                    it
                })
                .collect();
            return (tagged, notes);
        }
        notes.push(format!("{}: 0", ep.name));
    }
    (Vec::new(), notes)
}

/// The configured endpoint whose feed `url` is, if any: of those whose address the feed
/// starts with, the longest such, then the one whose template past `{query}` the feed also
/// ends with, then the first listed. Two sources on one host told apart only by a parameter
/// after the search (`&fmt=json`) are told apart by it, not by their order.
pub fn owner_of<'a>(endpoints: &'a [Endpoint], url: &str) -> Option<&'a Endpoint> {
    let score = |e: &Endpoint| {
        let (base, tail) = e.url.split_once("{query}").unwrap_or((e.url.as_str(), ""));
        let tail = if url.ends_with(tail) {
            1 + tail.len()
        } else {
            0
        };
        (base.len(), tail)
    };
    let mut best: Option<&Endpoint> = None;
    for e in endpoints.iter().filter(|e| e.owns_url(url)) {
        if best.is_none_or(|b| score(e) > score(b)) {
            best = Some(e);
        }
    }
    best
}

/// Whether a subscription on `feed_url` can fetch new episodes: false when its source
/// answers JSON, which no client reads as a feed (#507, #536). The source is `endpoint` when
/// the rule keeps its name and that source is still in `endpoints`, else the one that owns
/// the feed (`owner_of`). A feed no source owns claims nothing.
pub fn follows(endpoints: &[Endpoint], feed_url: &str, endpoint: Option<&str>) -> bool {
    source_of(endpoints, feed_url, endpoint).is_none_or(|e| answers_rss(&e.kind))
}

/// The source a subscription's feed comes from: `endpoint` when the rule keeps its name and
/// that source is still in `endpoints`, else the one that owns the feed (`owner_of`).
pub fn source_of<'a>(
    endpoints: &'a [Endpoint],
    feed_url: &str,
    endpoint: Option<&str>,
) -> Option<&'a Endpoint> {
    endpoint
        .filter(|n| !n.is_empty())
        .and_then(|n| endpoints.iter().find(|e| e.name == n))
        .or_else(|| owner_of(endpoints, feed_url))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::net::FakeHttp;

    fn nyaa() -> Endpoint {
        Endpoint {
            category: "1_2".to_string(),
            filter: "1".to_string(),
            ..Endpoint::new("nyaa", FILTERED_RSS, "https://nyaa.si/")
        }
    }

    fn anirena() -> Endpoint {
        Endpoint::new(
            "anirena",
            "rss",
            "https://www.anirena.com/rss?q={query}&adult=1",
        )
    }

    /// An RSS source that needs a token, with every quirk on, as its definition gives them.
    pub(crate) fn keyed_ep() -> Endpoint {
        let mut ep = Endpoint::new(
            "keyed",
            "rss",
            "https://www.keyed.example/episodes/feed/?res=1080p&type=torrent&subs%5B0%5D=us&s={query}&token=TOK",
        );
        ep.quirks = super::quirks::Quirks {
            quality_local: true,
            one_variant: Some("HEVC".into()),
            show_feed: Some(super::quirks::ShowFeed {
                pages: vec![
                    r"^https://www\.keyed\.example/(?:episodes|specials|raws)/(?P<slug>[a-z0-9-]+?)-\d+(?:v\d+)?/".into(),
                    r"^https://www\.keyed\.example/encodes/(?P<slug>[a-z0-9-]+?)-\d+(?:v\d+)?-hevc/".into(),
                    r"^https://www\.keyed\.example/batches/(?P<slug>[a-z0-9-]+?)-\d+-\d+/".into(),
                ],
                url: "https://www.keyed.example/anime-list/{slug}/feed/".into(),
            }),
        };
        ep
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
        param(url, "s")
    }

    /// The search's parameters, in order.
    fn params(url: &str) -> Vec<(String, String)> {
        let qs = url.split_once('?').map_or("", |(_, q)| q);
        parse_qsl(qs)
    }

    /// The value of the `key` parameter, empty when there is none.
    fn param(url: &str, key: &str) -> String {
        params(url)
            .into_iter()
            .find(|(k, _)| k == key)
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
        validate("nyaa", FILTERED_RSS, "https://nyaa.si/").unwrap();
        validate(
            "anirena",
            "rss",
            "https://www.anirena.com/rss?q={query}&adult=1",
        )
        .unwrap();
        validate(" a.b_c-1 ", " rss ", " https://x/?q={query} ").unwrap();
    }

    #[test]
    fn validate_refuses_a_hand_added_json_source() {
        let err = validate("a", "json", "https://search.example.org/api?q={query}").unwrap_err();
        assert!(
            matches!(&err, CoreError::Invalid { message, .. } if message.contains("endpoint repo")),
            "{err:?}"
        );
    }

    #[test]
    fn validate_rejects_bad_endpoints() {
        for (name, kind, url) in [
            ("", FILTERED_RSS, "https://x/"),            // no name
            ("a", "html", "https://x/"),                 // bad kind
            ("a", "rss", "https://x/rss"),               // no {query}
            ("a", "torznab", "http://x/api"),            // no {query}
            ("a", JSON_FEED, "https://x/json"),          // no {query}
            ("a", FILTERED_RSS, ""),                     // no url
            ("bad name)", FILTERED_RSS, "https://x/"),   // bad name characters
            ("a", FILTERED_RSS, "https://x/?q={query}"), // adds its own query
            ("a", "RSS", "https://x/?q={query}"),        // kinds are lower case
            ("a", "rss", "x/rss?q={query}"),             // no scheme
        ] {
            let err = validate(name, kind, url).unwrap_err();
            assert!(
                matches!(err, CoreError::Invalid { .. }),
                "{name:?} {kind:?} {url:?}"
            );
        }
    }

    #[test]
    fn search_url_filtered_rss_kind() {
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
        let (q, excl) = split_exclusions("[Kumo] show 1080p");
        assert_eq!(q, "[Kumo] show 1080p");
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
    async fn fetch_items_torznab_kind_applies_exclusions() {
        let http = FakeHttp::new();
        http.push(200, feed(&[("Show 05 HEVC", "l1"), ("Show 05 AVC", "l2")]));
        let ep = Endpoint::new(
            "indexer",
            "torznab",
            "http://10.0.0.2:9696/1/api?t=search&q={query}&apikey=k",
        );
        let items = fetch_items(http.as_ref(), &ep, "show -HEVC").await.unwrap();
        assert_eq!(
            items.iter().map(|i| i.link.as_str()).collect::<Vec<_>>(),
            ["l2"]
        );
        assert_eq!(
            http.requests()[0].url,
            "http://10.0.0.2:9696/1/api?t=search&q=show&apikey=k"
        );
    }

    #[test]
    fn a_rule_on_a_source_that_answers_json_cannot_follow_new_episodes() {
        let sources = [
            Endpoint::new("feed", "rss", "https://a/rss?q={query}"),
            Endpoint::new("api", "json", "https://c/api?q={query}"),
            Endpoint::new("anime", JSON_FEED, "https://d/feed/json?q={query}"),
            Endpoint::new("search", FILTERED_RSS, "https://e/"),
        ];
        let follows = |feed: &str, name: Option<&str>| follows(&sources, feed, name);
        // A rule written before #507 holds the JSON search address as its feed.
        assert!(!follows("https://c/api?q=Show", None));
        assert!(!follows("https://d/feed/json?q=Show&old=1", None));
        assert!(follows("https://a/rss?q=Show", None));
        assert!(follows("https://e/?page=rss&q=Show&c=1_0&f=0", None));
        // A rule that keeps its source's name is judged by that source, whatever its
        // feed holds (the phone keeps a key as a placeholder).
        assert!(!follows("https://c/api?q=Show&key={key}", Some("api")));
        assert!(follows("https://c/api?q=Show", Some("feed")));
        // A source gone from the list leaves the feed to say whose it is.
        assert!(!follows("https://c/api?q=Show", Some("gone")));
        // A feed no source owns claims nothing.
        assert!(follows("https://elsewhere/rss?q=Show", None));
        assert!(follows("https://elsewhere/rss?q=Show", Some("gone")));
    }

    #[test]
    fn two_sources_on_one_base_are_told_apart_by_what_follows_the_search() {
        let rss = Endpoint::new("rss", "rss", "https://x/?q={query}");
        let json = Endpoint::new("json", "json", "https://x/?q={query}&fmt=json");
        for list in [[rss.clone(), json.clone()], [json.clone(), rss.clone()]] {
            assert_eq!(
                owner_of(&list, "https://x/?q=Show&fmt=json").unwrap().name,
                "json"
            );
            assert_eq!(owner_of(&list, "https://x/?q=Show").unwrap().name, "rss");
            assert!(!follows(&list, "https://x/?q=Show&fmt=json", None));
            assert!(follows(&list, "https://x/?q=Show", None));
        }
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

    fn f_param(url: &str) -> String {
        param(url, "f")
    }

    #[test]
    fn a_filtered_rss_endpoint_named_with_no_category_or_filter_asks_for_everything() {
        // anirss's Endpoint: category "1_0" (every anime upload) and filter "0" (no filter)
        let ep = Endpoint::new("site", FILTERED_RSS, "https://tracker.example/");
        assert_eq!((ep.category.as_str(), ep.filter.as_str()), ("1_0", "0"));
        assert_eq!(
            search_url(&ep, "Show 1080p"),
            "https://tracker.example/?page=rss&q=Show+1080p&c=1_0&f=0"
        );
        // Trusted only is the definition's to ask for, and goes out as given.
        let trusted = Endpoint {
            category: "1_2".to_string(),
            filter: "2".to_string(),
            ..ep
        };
        let got = params(&search_url(&trusted, "Show"));
        assert_eq!(
            got,
            [("page", "rss"), ("q", "Show"), ("c", "1_2"), ("f", "2")]
                .map(|(k, v)| (k.to_string(), v.to_string()))
        );
    }

    #[test]
    fn a_filtered_rss_address_with_its_own_parameters_takes_the_search_after_them() {
        // anirss: f"{ep.url}{'&' if '?' in ep.url else '?'}{qs}"
        let ep = Endpoint::new("site", FILTERED_RSS, "https://tracker.example/?token=abc");
        assert_eq!(
            search_url(&ep, "Show"),
            "https://tracker.example/?token=abc&page=rss&q=Show&c=1_0&f=0"
        );
    }

    #[tokio::test]
    async fn a_trusted_only_subscription_keeps_its_filter() {
        // anirss's subscription_feed: unfiltered only when the results were found that way
        let http = FakeHttp::new();
        let trusted = Endpoint {
            filter: "2".to_string(),
            ..Endpoint::new("site", FILTERED_RSS, "https://tracker.example/")
        };
        let mut item = rel("[A] Show - 01", "l");
        item.source = "site".to_string();
        let spec = build_subscription(http.as_ref(), &trusted, "Show", &[item]).await;
        assert_eq!(f_param(&spec.feed_url), "2", "{}", spec.feed_url);
    }

    #[tokio::test]
    async fn a_filtered_search_with_nothing_asks_again_for_every_upload() {
        let trusted = Endpoint {
            filter: "2".to_string(),
            ..nyaa()
        };
        let http = FakeHttp::new();
        http.push(200, feed(&[]));
        http.push(200, feed(&[("[g] Seitokai ni mo Ana wa Aru! - 05", "l")]));
        let items = fetch_items(http.as_ref(), &trusted, "Seitokai ni mo Ana wa Aru")
            .await
            .unwrap();
        let sent: Vec<String> = http.requests().iter().map(|r| f_param(&r.url)).collect();
        assert_eq!(sent, ["2", "0"]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].source, "nyaa: all uploads");
        // Subscribing to those results keeps the filter off, or the rule finds nothing.
        let (url, _, _) = subscription_feed(http.as_ref(), &trusted, "Seitokai", &items).await;
        assert_eq!(f_param(&url), "0");
    }

    #[tokio::test]
    async fn a_filtered_search_with_results_or_no_filter_asks_once() {
        let trusted = Endpoint {
            filter: "2".to_string(),
            ..nyaa()
        };
        let http = FakeHttp::new();
        http.push(200, feed(&[("t", "l")]));
        let items = fetch_items(http.as_ref(), &trusted, "show").await.unwrap();
        assert_eq!(http.requests().len(), 1);
        assert_eq!(items[0].source, "");
        let (url, _, _) = subscription_feed(http.as_ref(), &trusted, "show", &items).await;
        assert_eq!(f_param(&url), "2");

        let open = Endpoint {
            filter: "0".to_string(),
            ..nyaa()
        };
        let http = FakeHttp::new();
        http.push(200, feed(&[]));
        assert!(
            fetch_items(http.as_ref(), &open, "x")
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(http.requests().len(), 1);
    }

    #[tokio::test]
    async fn a_failed_unfiltered_retry_still_answers_nothing_found() {
        let trusted = Endpoint {
            filter: "2".to_string(),
            ..nyaa()
        };
        let http = FakeHttp::new();
        http.push(200, feed(&[]));
        http.push(503, "busy");
        let items = fetch_items(http.as_ref(), &trusted, "show").await.unwrap();
        assert!(items.is_empty());
        assert_eq!(http.requests().len(), 2);
    }

    #[tokio::test]
    async fn a_refine_refetches_with_the_filter_its_list_was_found_with() {
        let trusted = Endpoint {
            filter: "2".to_string(),
            ..nyaa()
        };
        // A trusted list narrowed to nothing stays nothing: no unfiltered retry.
        let http = FakeHttp::new();
        http.push(200, feed(&[]));
        let items = refetch(http.as_ref(), &trusted, "show -HEVC", false)
            .await
            .unwrap();
        assert!(items.is_empty());
        let sent: Vec<String> = http.requests().iter().map(|r| f_param(&r.url)).collect();
        assert_eq!(sent, ["2"]);

        // An all-uploads list is narrowed among all uploads and keeps its marker.
        let http = FakeHttp::new();
        http.push(200, feed(&[("[g] show - 05", "l")]));
        let items = refetch(http.as_ref(), &trusted, "show -HEVC", true)
            .await
            .unwrap();
        assert_eq!(f_param(&http.requests()[0].url), "0");
        assert_eq!(items[0].source, "nyaa: all uploads");
    }

    #[tokio::test]
    async fn fetch_items_sends_a_browser_user_agent_and_the_endpoint_timeout() {
        let http = FakeHttp::new();
        http.push(200, feed(&[]));
        // Its filter finds nothing, so it asks again unfiltered
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
    async fn fetch_items_quality_local_searches_clean_and_filters_quality() {
        let xml = r#"<rss version="2.0" xmlns:x="https://www.keyed.example/rss-page/"><channel>
<item><title>[Torrent] Ranma - 01 [1080p NF WEB-DL AVC AAC][Airing]</title><link>dl</link>
<x:resolution>1080p</x:resolution><x:infomkv>NF WEB-DL AVC AAC</x:infomkv><x:linktype>Torrent</x:linktype></item>
<item><title>[Torrent] Ranma - 01 (HEVC) [1080p NF WEBRip HEVC AAC][Encoded]</title><link>rip</link>
<x:resolution>1080p</x:resolution><x:infomkv>NF WEBRip HEVC AAC</x:infomkv><x:linktype>Torrent</x:linktype></item>
</channel></rss>"#;
        let http = FakeHttp::new();
        http.push(200, xml);
        let items = fetch_items(http.as_ref(), &keyed_ep(), "Ranma 1080p WEB-DL")
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

    // A narrowing source's subscription rules, from anirss's own tests ----------------------

    #[test]
    fn feed_url_quality_local_strips_quality_tokens() {
        let (url, excluded) = feed_url(&keyed_ep(), "Ranma 1080p WEB-DL -hevc");
        assert_eq!(s_param(&url), "Ranma");
        assert_eq!(excluded, strings(&["hevc"]));
    }

    #[test]
    fn subscription_feed_keyed_fallback_puts_quality_in_must() {
        let (url, must, excluded) =
            subscription_feed_resolved(&keyed_ep(), "Ranma 1080p WEB-DL -hevc", None);
        assert_eq!(s_param(&url), "Ranma");
        // qB enforces the quality facets against the raw feed titles instead.
        assert_eq!(must, "1080p WEB-DL");
        assert_eq!(excluded, strings(&["hevc"]));
    }

    #[test]
    fn subscription_feed_keyed_uses_per_show_and_must_contain() {
        let per_show = "https://www.keyed.example/anime-list/x/feed/?res=1080p".to_string();
        let (url, must, excluded) = subscription_feed_resolved(
            &keyed_ep(),
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
            &keyed_ep(),
            r#"[Kumo] Show "Multi Subs" 1080p"#,
            Some("https://e/f".to_string()),
        );
        assert_eq!(must, "Show Multi Subs 1080p");
    }

    #[test]
    fn subscription_feed_keyed_falls_back_to_search_feed() {
        let (url, must, excluded) = subscription_feed_resolved(&keyed_ep(), "gaikotsu -hevc", None);
        assert!(url.contains("s=gaikotsu"));
        assert_eq!(must, "");
        assert_eq!(excluded, strings(&["hevc"]));
    }

    #[test]
    fn subscription_feed_of_a_plain_source_unchanged() {
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
            subscription_feed_resolved(&keyed_ep(), query, Some("https://e/f".to_string()));
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
        let (_, _, excluded) = subscription_feed_resolved(&keyed_ep(), "Ranma 1080p", None);
        assert_eq!(excluded, strings(&["HEVC"]));
    }

    #[tokio::test]
    async fn build_subscription_probes_the_per_show_feed_once() {
        let mut page_item = rel("Gaikotsu - 02 [1080p][Airing]", "l");
        page_item.page = "https://www.keyed.example/episodes/gaikotsu-kishi-sama-02/".to_string();
        let http = FakeHttp::new();
        http.push(200, feed(&[("x", "l")]));
        let spec = build_subscription(
            http.as_ref(),
            &keyed_ep(),
            "Gaikotsu 1080p -dub",
            &[page_item],
        )
        .await;
        assert!(
            spec.feed_url
                .starts_with("https://www.keyed.example/anime-list/gaikotsu-kishi-sama/feed/?")
        );
        assert_eq!(spec.must_contain, "Gaikotsu 1080p");
        assert_eq!(spec.must_not_contain, "dub|HEVC");
        assert_eq!(spec.endpoint_name, "keyed");
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
