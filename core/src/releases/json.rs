//! The `json` kind: a search API that answers in JSON, giving per result a title, an
//! info-hash or a ready link, a size and maybe a page. Not an anirss endpoint kind. It knows
//! no field names: the repo definition's `JsonMap` says where each one sits, so one kind reads
//! any API of that shape. An info-hash becomes a magnet here.

use std::time::Duration;

use serde_json::Value;

use crate::contract::*;
use crate::net::{Http, HttpRequest, Method};
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

/// A size in bytes, as a number or a string of digits.
fn bytes(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64().or_else(|| {
            n.as_f64()
                .filter(|f| f.is_finite() && *f >= 0.0)
                .map(|f| f as u64)
        }),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

/// Whether `hash` is an info-hash: 40 hex digits or 32 base32 ones, in either case. Anything
/// else is left out rather than put in a magnet, where it could add parameters of its own.
fn is_hash(hash: &str) -> bool {
    match hash.len() {
        40 => hash.bytes().all(|b| b.is_ascii_hexdigit()),
        32 => hash
            .bytes()
            .all(|b| b.is_ascii_alphabetic() || (b'2'..=b'7').contains(&b)),
        _ => false,
    }
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
            .filter(|l| l.starts_with("magnet:") || l.starts_with("https://"))?,
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
    let reply: Value = serde_json::from_slice(body).map_err(|e| format!("not JSON: {e}"))?;
    if let Some(v) = map.error.as_deref().and_then(|p| at(&reply, p))
        && truthy(v)
    {
        return Err(
            match v
                .as_str()
                .map(super::fetch::their_message)
                .filter(|m| !m.is_empty())
            {
                Some(m) => format!("the search answered with an error: {m}"),
                None => "the search answered with an error".into(),
            },
        );
    }
    let items = match at(&reply, &map.items) {
        Some(Value::Null) => return Ok(Vec::new()),
        None => return Err(format!("the reply has no results at {:?}", map.items)),
        Some(Value::Array(items)) => items,
        Some(_) => return Err("the reply's results are not a list".into()),
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
        .send(HttpRequest {
            method: Method::Get,
            url: url.to_string(),
            headers: vec![
                (
                    "User-Agent".to_string(),
                    super::fetch::USER_AGENT.to_string(),
                ),
                ("Accept".to_string(), "application/json".to_string()),
            ],
            body: None,
            timeout: Some(timeout),
        })
        .await
        .map_err(|e| CoreError::Service {
            service: source.to_string(),
            code: "unreachable".into(),
            message: e.message,
        })?;
    if !response.is_success() {
        return Err(CoreError::Service {
            service: source.to_string(),
            code: format!("http_{}", response.status),
            message: format!("the search answered {}", response.status),
        });
    }
    parse(&response.body, map, source).map_err(|message| CoreError::Service {
        service: source.to_string(),
        code: "unreadable".into(),
        message,
    })
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
        let e = parse(br#"{"error": "  "}"#, &errs, "s").unwrap_err();
        assert_eq!(e, "the search answered with an error");
        assert!(parse(br#"{"error": false, "results": []}"#, &errs, "s").is_ok());
        assert!(parse(br#"{"error": 0}"#, &errs, "s").unwrap().is_empty());
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
