//! The `json` kind: a search API that answers in JSON, giving per result a title, an
//! info-hash or a ready link, a size and maybe a page. Not an anirss endpoint kind. It knows
//! no field names: the repo definition's `JsonMap` says where each one sits, so one kind reads
//! any API of that shape. An info-hash becomes a magnet here.

use std::time::Duration;

use serde_json::Value;

use crate::contract::*;
use crate::foundation::magnet::is_hash;
use crate::net::{Http, HttpRequest};
use crate::repos::index::JsonMap;

/// The value at a dot path of object keys; the value itself for an empty path.
fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(value);
    }
    path.split('.')
        .try_fold(value, |v, key| v.as_object()?.get(key))
}

/// Whether a value is set: anything but null, false, zero or empty. A string reads the way
/// an API that stringifies its flags means it, so "false", "0" and "no" are unset too.
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => {
            let s = s.trim();
            !s.is_empty()
                && !["false", "0", "no"]
                    .iter()
                    .any(|f| s.eq_ignore_ascii_case(f))
        }
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// A string or a number as text, trimmed; None when empty or anything else.
fn text(v: &Value) -> Option<String> {
    let s = match v {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    (!s.is_empty()).then_some(s)
}

/// A size in bytes: a number, a string of digits ("1500" or "1.5"), or one with a unit
/// ("1.4 GiB", "700 MB"). None when unreadable, or past what a u64 holds rather than a
/// saturated 16 EiB.
fn bytes(v: &Value) -> Option<u64> {
    fn whole(f: f64) -> Option<u64> {
        (f.is_finite() && f >= 0.0 && f < u64::MAX as f64).then_some(f as u64)
    }
    match v {
        Value::Number(n) => n.as_u64().or_else(|| n.as_f64().and_then(whole)),
        Value::String(s) => {
            let s = s.trim();
            s.parse()
                .ok()
                .or_else(|| s.parse::<f64>().ok().and_then(whole))
                .or_else(|| crate::foundation::size::size_bytes(s))
        }
        _ => None,
    }
}

/// A ready link as the release keeps it: an `https://` address, its scheme in any case, or a
/// magnet that names an info-hash. Anything else is left out.
fn ready_link(link: &str) -> Option<String> {
    let scheme = link.get(..8).filter(|s| s.eq_ignore_ascii_case("https://"));
    if scheme.is_some() && link.len() > 8 {
        return Some(format!("https://{}", &link[8..]));
    }
    let magnet = link
        .get(..7)
        .is_some_and(|s| s.eq_ignore_ascii_case("magnet:"));
    let hashed = link
        .split(['?', '&'])
        .filter_map(|p| p.split_once('='))
        .any(|(k, v)| {
            k.eq_ignore_ascii_case("xt")
                && v.get(..9)
                    .is_some_and(|u| u.eq_ignore_ascii_case("urn:btih:"))
                && is_hash(&v[9..])
        });
    (magnet && hashed).then(|| link.to_string())
}

/// The magnet for an info-hash, named after the release.
fn magnet(hash: &str, title: &str) -> String {
    format!(
        "magnet:?xt=urn:btih:{hash}&dn={}",
        super::endpoints::quote(title.as_bytes(), "")
    )
}

/// The page template with every placeholder filled, percent-encoded, from `item`; empty when
/// any value is missing or unset, so a result never links to a page that is not there.
fn fill_page(template: &str, item: &Value) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            return String::new();
        };
        let Some(value) = at(item, &after[..close])
            .filter(|v| truthy(v))
            .and_then(text)
        else {
            return String::new();
        };
        out.push_str(&rest[..open]);
        out.push_str(&super::endpoints::quote(value.as_bytes(), ""));
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

/// One result as a release, or None when the map says to skip it or it has no title, or
/// neither a usable hash nor a ready `magnet:` or `https://` link.
fn release(item: &Value, map: &JsonMap, source: &str) -> Option<Release> {
    let field = |path: &Option<String>| path.as_deref().and_then(|p| at(item, p));
    if field(&map.skip).is_some_and(truthy) {
        return None;
    }
    let title = at(item, &map.title).and_then(text)?;
    let link = match field(&map.hash).and_then(text).filter(|h| is_hash(h)) {
        Some(hash) => magnet(&hash, &title),
        None => field(&map.link)
            .and_then(text)
            .and_then(|l| ready_link(&l))?,
    };
    Some(Release {
        link,
        size: field(&map.size)
            .and_then(bytes)
            .map(super::fetch::size_text)
            .unwrap_or_default(),
        page: map
            .page
            .as_deref()
            .map(|p| fill_page(p, item))
            .unwrap_or_default(),
        title,
        seeders: None,
        leechers: None,
        downloads: None,
        category: String::new(),
        source: source.to_string(),
        resolution: String::new(),
        infomkv: String::new(),
        linktype: String::new(),
    })
}

/// The reply's results as releases, read through `map`. Counts stay unknown, since the kind
/// has no field for them. A reply whose results list is null has none; one without the list
/// at all, or whose list is something else, does not read, so an API that renames the field
/// shows an error rather than "no results" forever.
pub fn parse(body: &[u8], map: &JsonMap, source: &str) -> Result<Vec<Release>, String> {
    let reply: Value = serde_json::from_slice(body).map_err(|e| {
        crate::i18n::tr("core-releases-not-json", &[("error", e.to_string().into())])
    })?;
    if let Some(v) = map.error.as_deref().and_then(|p| at(&reply, p))
        && truthy(v)
    {
        return Err(
            match v
                .as_str()
                .map(super::fetch::their_message)
                .filter(|m| !m.is_empty())
            {
                Some(m) => {
                    crate::i18n::tr("core-releases-search-error-said", &[("message", m.into())])
                }
                None => crate::i18n::tr("core-releases-search-error", &[]),
            },
        );
    }
    let items = match at(&reply, &map.items) {
        Some(Value::Null) => return Ok(Vec::new()),
        None => {
            return Err(crate::i18n::tr(
                "core-releases-no-results-field",
                &[("path", format!("{:?}", map.items).into())],
            ));
        }
        Some(Value::Array(items)) => items,
        Some(_) => return Err(crate::i18n::tr("core-releases-results-not-list", &[])),
    };
    Ok(items
        .iter()
        .filter_map(|item| release(item, map, source))
        .collect())
}

/// Searches a JSON API at `url`, reading its reply through `map`.
pub async fn fetch(
    http: &dyn Http,
    url: &str,
    map: &JsonMap,
    source: &str,
    timeout: Duration,
) -> Result<Vec<Release>, CoreError> {
    let response = http
        .send(
            HttpRequest::get(url.to_string())
                .header("User-Agent", super::fetch::USER_AGENT)
                .header("Accept", "application/json")
                .timeout(timeout),
        )
        .await
        .map_err(|e| CoreError::service(source.to_string(), "unreachable", e.message))?;
    if !response.is_success() {
        return Err(CoreError::service(
            source.to_string(),
            format!("http_{}", response.status),
            crate::i18n::tr(
                "core-releases-search-answered",
                &[("status", response.status.to_string().into())],
            ),
        ));
    }
    parse(&response.body, map, source)
        .map_err(|message| CoreError::service(source.to_string(), "unreadable", message))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/json-search.json"),
        )
        .unwrap()
    }

    /// The fixture's map.
    fn map() -> JsonMap {
        serde_json::from_value(json!({
            "items": "data.hits",
            "title": "label",
            "hash": "torrent.infohash",
            "size": "torrent.bytes",
            "page": "https://example.org/view/{ref.post}",
            "skip": "flags.hidden",
            "error": "status.failed",
        }))
        .unwrap()
    }

    /// A map of just `fields`, plus a title at `t`.
    fn small(fields: Value) -> JsonMap {
        let mut m = json!({"items": "", "title": "t"});
        m.as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        serde_json::from_value(m).unwrap()
    }

    const HASH: &str = "00112233445566778899aabbccddeeff00112233";

    #[test]
    fn each_field_comes_from_where_the_map_says() {
        let items = parse(&fixture(), &map(), "search").unwrap();
        let titles: Vec<&str> = items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "[Group] Show - 01 (1080p)",
                "[Group] Show - 01 (720p)",
                "[Other] Show - 01 (1080p HEVC)",
            ]
        );
        assert_eq!(items[0].size, "1.4 GiB");
        assert_eq!(items[1].size, "700.0 MiB");
        assert_eq!(items[0].source, "search");
        assert_eq!(
            (items[0].seeders, items[0].leechers, items[0].downloads),
            (None, None, None)
        );
    }

    #[test]
    fn the_items_path_reaches_down_or_takes_the_reply_itself() {
        let one = json!({"t": "x", "h": HASH});
        let deep = small(json!({"items": "a.b.c", "hash": "h"}));
        let body = json!({"a": {"b": {"c": [one]}}}).to_string();
        assert_eq!(parse(body.as_bytes(), &deep, "s").unwrap().len(), 1);
        let bare = small(json!({"hash": "h"}));
        let body = json!([one, one]).to_string();
        assert_eq!(parse(body.as_bytes(), &bare, "s").unwrap().len(), 2);
    }

    #[test]
    fn a_missing_results_list_is_an_error_and_a_null_one_is_empty() {
        let deep = small(json!({"items": "data.hits", "hash": "h"}));
        let err = parse(br#"{"data": {"results": []}}"#, &deep, "s").unwrap_err();
        assert!(err.contains("data.hits"), "{err}");
        assert!(
            parse(br#"{"data": {"hits": null}}"#, &deep, "s")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_flag_written_as_text_reads_the_way_it_means() {
        for unset in ["false", "FALSE", "0", "no", " No ", ""] {
            assert!(!truthy(&json!(unset)), "{unset:?}");
        }
        for set in ["true", "1", "yes", "x"] {
            assert!(truthy(&json!(set)), "{set:?}");
        }
    }

    #[test]
    fn an_error_text_is_cut_short_and_cleaned() {
        let errs = small(json!({"hash": "h", "error": "e"}));
        let long = format!("bad\u{7}{}", "x".repeat(500));
        let body = json!({"e": long}).to_string();
        let err = parse(body.as_bytes(), &errs, "s").unwrap_err();
        assert!(!err.contains('\u{7}'), "{err}");
        assert!(err.chars().count() < 260, "{}", err.len());
    }

    #[test]
    fn the_link_is_a_magnet_from_the_hash_and_the_title() {
        let items = parse(&fixture(), &map(), "s").unwrap();
        assert_eq!(
            items[0].link,
            format!("magnet:?xt=urn:btih:{HASH}&dn=%5BGroup%5D%20Show%20-%2001%20%281080p%29")
        );
        assert!(
            !items.iter().any(|i| i.title.contains("No Hash")),
            "a result without a hash is left out"
        );
    }

    #[test]
    fn only_a_hex_or_base32_hash_reaches_the_magnet() {
        let one = |hash: &str| {
            let body = json!([{"t": "x", "h": hash}]).to_string();
            parse(body.as_bytes(), &small(json!({"hash": "h"})), "s").unwrap()
        };
        let hex = one("05BD3A45348E1338117E89CF1E25FD76B7A30DE3");
        assert_eq!(hex.len(), 1);
        assert!(
            hex[0]
                .link
                .starts_with("magnet:?xt=urn:btih:05BD3A45348E1338117E89CF1E25FD76B7A30DE3&dn=")
        );
        assert_eq!(one("AW6TURJURYJTQEL6RHHR4JP5O23KGDPD").len(), 1);
        assert_eq!(one("aw6turjuryjtqel6rhhr4jp5o23kgdpd").len(), 1);
        assert!(
            one(&format!("{HASH}&tr=https://x")).is_empty(),
            "an injected parameter leaves the result out"
        );
        assert!(one("abc&tr=https://x").is_empty());
        assert!(one("05bd3a45348e1338117e89cf1e25fd76b7a30dzz").is_empty());
        assert!(one("AW6TURJURYJTQEL6RHHR4JP5O23KGDP1").is_empty());
    }

    #[test]
    fn a_ready_link_stands_in_for_a_hash() {
        let links = small(json!({"link": "url"}));
        let body = json!([
            {"t": "a", "url": "magnet:?xt=urn:btih:00112233445566778899aabbccddeeff00112233"},
            {"t": "b", "url": "https://files.example.org/b.torrent"},
            {"t": "c", "url": "http://files.example.org/c.torrent"},
            {"t": "d", "url": "javascript:alert(1)"},
            {"t": "e"},
        ])
        .to_string();
        let items = parse(body.as_bytes(), &links, "s").unwrap();
        let got: Vec<(&str, &str)> = items
            .iter()
            .map(|i| (i.title.as_str(), i.link.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (
                    "a",
                    "magnet:?xt=urn:btih:00112233445566778899aabbccddeeff00112233"
                ),
                ("b", "https://files.example.org/b.torrent"),
            ]
        );
        let both = small(json!({"hash": "h", "link": "url"}));
        let body = json!([{"t": "x", "h": "nope", "url": "https://files.example.org/x"}]);
        let items = parse(body.to_string().as_bytes(), &both, "s").unwrap();
        assert_eq!(items[0].link, "https://files.example.org/x");
    }

    #[test]
    fn a_link_takes_https_in_any_case_and_a_magnet_needs_a_hash() {
        let links = small(json!({"link": "url"}));
        let body = json!([
            {"t": "a", "url": "HTTPS://files.example.org/a.torrent"},
            {"t": "b", "url": "MAGNET:?dn=x&XT=URN:BTIH:00112233445566778899AABBCCDDEEFF00112233"},
            {"t": "c", "url": "magnet:?dn=no-hash"},
            {"t": "d", "url": "magnet:?xt=urn:btih:nope"},
            {"t": "e", "url": "https://"},
        ])
        .to_string();
        let items = parse(body.as_bytes(), &links, "s").unwrap();
        let got: Vec<(&str, &str)> = items
            .iter()
            .map(|i| (i.title.as_str(), i.link.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                ("a", "https://files.example.org/a.torrent"),
                (
                    "b",
                    "MAGNET:?dn=x&XT=URN:BTIH:00112233445566778899AABBCCDDEEFF00112233"
                ),
            ]
        );
    }

    #[test]
    fn a_size_reads_decimals_and_units_and_never_saturates() {
        assert_eq!(bytes(&json!("1.5")), Some(1));
        assert_eq!(bytes(&json!(" 1500 ")), Some(1500));
        assert_eq!(bytes(&json!("1 KiB")), Some(1024));
        assert_eq!(bytes(&json!("1.5 GB")), Some(1_610_612_736));
        assert_eq!(bytes(&json!(1.0e30)), None);
        assert_eq!(bytes(&json!("1e30")), None);
        assert_eq!(bytes(&json!("99999999999 TiB")), None);
        assert_eq!(bytes(&json!(-1)), None);
        assert_eq!(bytes(&json!("big")), None);
    }

    #[test]
    fn the_page_is_filled_only_when_every_value_is_there() {
        let items = parse(&fixture(), &map(), "s").unwrap();
        assert_eq!(items[0].page, "", "a zero leaves the page empty");
        assert_eq!(items[1].page, "https://example.org/view/4821");
        let page = small(json!({
            "hash": "h",
            "page": "https://example.org/g/{group}/v?id={id}",
        }));
        let one = |item: Value| {
            let body = json!([item]).to_string();
            parse(body.as_bytes(), &page, "s").unwrap().remove(0).page
        };
        assert_eq!(
            one(json!({"t": "x", "h": HASH, "group": "a/b c", "id": 7})),
            "https://example.org/g/a%2Fb%20c/v?id=7"
        );
        for missing in [
            json!({"t": "x", "h": HASH, "id": 7}),
            json!({"t": "x", "h": HASH, "group": "", "id": 7}),
            json!({"t": "x", "h": HASH, "group": null, "id": 7}),
            json!({"t": "x", "h": HASH, "group": false, "id": 7}),
            json!({"t": "x", "h": HASH, "group": "g", "id": 0}),
        ] {
            assert_eq!(one(missing.clone()), "", "{missing}");
        }
    }

    #[test]
    fn a_result_the_map_skips_is_left_out() {
        let items = parse(&fixture(), &map(), "s").unwrap();
        assert!(items.iter().all(|i| !i.title.contains("Hidden")));
        assert!(
            items.iter().any(|i| i.title.contains("HEVC")),
            "false does not skip"
        );
    }

    #[test]
    fn an_error_reply_is_an_error() {
        let errs = small(json!({"hash": "h", "items": "results", "error": "error"}));
        let e = parse(br#"{"error": "bad query", "results": []}"#, &errs, "s").unwrap_err();
        assert!(e.contains("bad query"), "{e}");
        let e = parse(br#"{"error": true}"#, &errs, "s").unwrap_err();
        assert_eq!(e, "the search answered with an error");
        let e = parse(br#"{"error": 1}"#, &errs, "s").unwrap_err();
        assert_eq!(e, "the search answered with an error");
        assert!(parse(br#"{"error": false, "results": []}"#, &errs, "s").is_ok());
        assert!(parse(br#"{"error": "  ", "results": []}"#, &errs, "s").is_ok());
        assert!(
            parse(br#"{"error": 0, "results": []}"#, &errs, "s")
                .unwrap()
                .is_empty()
        );
        assert!(parse(b"<html>", &errs, "s").is_err());
        assert!(parse(br#"{"results": {"a": 1}}"#, &errs, "s").is_err());
    }

    #[test]
    fn a_result_without_its_title_is_left_out_and_missing_extras_stay_empty() {
        let full = map();
        let body = json!({"data": {"hits": [
            {"torrent": {"infohash": HASH}},
            {"label": "  ", "torrent": {"infohash": HASH}},
            {"label": "[Group] Show - 02 (1080p)", "torrent": {"infohash": HASH}},
        ]}})
        .to_string();
        let items = parse(body.as_bytes(), &full, "s").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "[Group] Show - 02 (1080p)");
        assert_eq!((items[0].size.as_str(), items[0].page.as_str()), ("", ""));
    }
}
