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

/// Whether a value is set: anything but null, false, zero or empty.
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Whether a `skip` or `error` value says yes: as `truthy`, except that a string reading
/// "false", "0" or "no" (trimmed, in any case), or only blank, says no.
fn flag(v: &Value) -> bool {
    match v {
        Value::String(s) => {
            let s = s.trim();
            !s.is_empty()
                && !["false", "0", "no"]
                    .iter()
                    .any(|f| s.eq_ignore_ascii_case(f))
        }
        _ => truthy(v),
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

/// A size in bytes, as a number, a numeric string, or a string with a unit as feeds write it
/// ("1.36GB", "485.1 MB"). None when negative, not finite or past what a u64 holds.
fn bytes(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64().or_else(|| whole(n.as_f64()?)),
        Value::String(s) => {
            let s = s.trim();
            if s.starts_with('-') {
                return None;
            }
            whole(
                s.parse::<f64>()
                    .ok()
                    .or_else(|| super::bestfit::size_mib(s).map(|mib| mib * 1024.0 * 1024.0))?,
            )
        }
        _ => None,
    }
}

/// `n` rounded to a whole u64, or None when it is negative, not finite or too big for one.
fn whole(n: f64) -> Option<u64> {
    let n = n.round();
    (n.is_finite() && n >= 0.0 && n < u64::MAX as f64).then_some(n as u64)
}

/// Whether `link` starts with `prefix`, in any case.
fn starts_with_any_case(link: &str, prefix: &str) -> bool {
    link.get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

/// Whether a ready link can be handed on: `https://`, or a `magnet:` that names an
/// info-hash, both in any case.
fn usable_link(link: &str) -> bool {
    starts_with_any_case(link, "https://")
        || (starts_with_any_case(link, "magnet:")
            && link.to_ascii_lowercase().contains("xt=urn:btih:"))
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
    if field(&map.skip).is_some_and(flag) {
        return None;
    }
    let title = at(item, &map.title).and_then(text)?;
    let link = match field(&map.hash).and_then(text).filter(|h| is_hash(h)) {
        Some(hash) => magnet(&hash, &title),
        None => field(&map.link).and_then(text).filter(|l| usable_link(l))?,
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
/// has no field for them. A null or empty results list is no results; a reply missing the
/// list, or whose list is something else, does not read, so a renamed field shows as an
/// error rather than as no results.
pub fn parse(body: &[u8], map: &JsonMap, source: &str) -> Result<Vec<Release>, String> {
    let reply: Value = serde_json::from_slice(body).map_err(|e| format!("not JSON: {e}"))?;
    if let Some(v) = map.error.as_deref().and_then(|p| at(&reply, p))
        && flag(v)
    {
        let said = v.as_str().map(super::fetch::feed_message);
        return Err(match said.filter(|m| !m.is_empty()) {
            Some(m) => format!("the search answered with an error: {m}"),
            None => "the search answered with an error".into(),
        });
    }
    let items = match at(&reply, &map.items) {
        None => return Err(format!("the reply has no {:?} results list", map.items)),
        Some(Value::Null) => return Ok(Vec::new()),
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
    fn a_ready_link_is_matched_in_any_case_and_a_magnet_needs_its_hash() {
        let links = small(json!({"link": "url"}));
        let body = json!([
            {"t": "a", "url": "HTTPS://files.example.org/a.torrent"},
            {"t": "b", "url": "Magnet:?XT=URN:BTIH:00112233445566778899aabbccddeeff00112233"},
            {"t": "c", "url": "magnet:?dn=no-hash"},
            {"t": "d", "url": "magnet:?xt=urn:sha1:abc"},
        ])
        .to_string();
        let titles: Vec<String> = parse(body.as_bytes(), &links, "s")
            .unwrap()
            .into_iter()
            .map(|i| i.title)
            .collect();
        assert_eq!(titles, ["a", "b"]);
    }

    #[test]
    fn a_size_reads_numbers_numeric_strings_and_units_and_never_saturates() {
        let sizes = small(json!({"hash": "h", "size": "z"}));
        let size = |z: Value| {
            let body = json!([{"t": "x", "h": HASH, "z": z}]).to_string();
            parse(body.as_bytes(), &sizes, "s").unwrap().remove(0).size
        };
        assert_eq!(size(json!(1536)), "1.5 KiB");
        assert_eq!(size(json!(1536.4)), "1.5 KiB");
        assert_eq!(size(json!("1536")), "1.5 KiB");
        assert_eq!(size(json!(" 1.5 ")), "2 B");
        assert_eq!(size(json!("1.36GB")), "1.4 GiB");
        assert_eq!(size(json!("485.1 MB")), "485.1 MiB");
        for unknown in [
            json!(-1),
            json!(-1.5),
            json!(1e30),
            json!("-1"),
            json!("-5 GB"),
            json!("NaN"),
            json!("inf"),
            json!("1e30"),
            json!("big"),
            json!(true),
        ] {
            assert_eq!(size(unknown.clone()), "", "{unknown}");
        }
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
        let e = parse(br#"{"error": " \u0000 "}"#, &errs, "s").unwrap_err();
        assert_eq!(e, "the search answered with an error");
        let e = parse(br#"{"error": "bad\n\u001b query"}"#, &errs, "s").unwrap_err();
        assert!(e.ends_with(": bad query"), "{e}");
        let long = json!({"error": "x".repeat(500)}).to_string();
        let e = parse(long.as_bytes(), &errs, "s").unwrap_err();
        assert!(e.ends_with(&format!("{}...", "x".repeat(200))), "{e}");
        assert!(parse(br#"{"error": false, "results": []}"#, &errs, "s").is_ok());
        assert!(
            parse(br#"{"error": 0, "results": []}"#, &errs, "s")
                .unwrap()
                .is_empty()
        );
        assert!(parse(b"<html>", &errs, "s").is_err());
        assert!(parse(br#"{"results": {"a": 1}}"#, &errs, "s").is_err());
    }

    #[test]
    fn skip_and_error_read_false_0_and_no_as_unset() {
        let errs = small(json!({"hash": "h", "items": "results", "error": "error"}));
        let skips = small(json!({"hash": "h", "skip": "s"}));
        let unset = [
            json!(false),
            json!(0),
            json!(0.0),
            json!(null),
            json!(""),
            json!("  "),
            json!("false"),
            json!(" FALSE "),
            json!("0"),
            json!("No"),
        ];
        let set = [
            json!(true),
            json!(1),
            json!(-1),
            json!("true"),
            json!("yes"),
            json!("1"),
            json!("nope"),
            json!("0.0"),
        ];
        for (value, says) in unset
            .iter()
            .map(|v| (v, false))
            .chain(set.iter().map(|v| (v, true)))
        {
            let reply = json!({"error": value, "results": []}).to_string();
            assert_eq!(
                parse(reply.as_bytes(), &errs, "x").is_err(),
                says,
                "error {value}"
            );
            let reply = json!([{"t": "x", "h": HASH, "s": value}]).to_string();
            assert_eq!(
                parse(reply.as_bytes(), &skips, "x").unwrap().is_empty(),
                says,
                "skip {value}"
            );
        }
    }

    #[test]
    fn a_missing_results_list_is_an_error_and_a_null_or_empty_one_is_no_results() {
        let deep = small(json!({"hash": "h", "items": "data.hits"}));
        for empty in [r#"{"data": {"hits": null}}"#, r#"{"data": {"hits": []}}"#] {
            assert!(
                parse(empty.as_bytes(), &deep, "s").unwrap().is_empty(),
                "{empty}"
            );
        }
        for missing in [r#"{"data": {"results": []}}"#, r#"{"other": 1}"#, "[]"] {
            let e = parse(missing.as_bytes(), &deep, "s").unwrap_err();
            assert!(e.contains("data.hits"), "{missing}: {e}");
        }
    }

    #[tokio::test]
    async fn a_missing_results_list_reaches_the_caller_as_unreadable() {
        let http = crate::net::FakeHttp::new();
        http.push_json(200, json!({"renamed": []}));
        let deep = small(json!({"hash": "h", "items": "hits"}));
        let e = fetch(
            http.as_ref(),
            "https://example.org/s",
            &deep,
            "s",
            Duration::from_secs(5),
        )
        .await
        .unwrap_err();
        assert!(
            matches!(&e, CoreError::Service { code, .. } if code == "unreadable"),
            "{e:?}"
        );
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
