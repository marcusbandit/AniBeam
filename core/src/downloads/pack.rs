//! Endpoint packs: the JSON file that carries a list of endpoints from one
//! person to another. AniBeam ships with no endpoints at all; a user pastes
//! one search URL or imports a pack someone gave them. This module reads,
//! checks and merges packs and writes them back out. It never fetches one
//! and knows no site by name: where a pack comes from is the user's
//! business.

use crate::contract::*;

pub const FORMAT: &str = "anibeam-endpoints";
pub const VERSION: u32 = 1;

/// Reads a pack's JSON and checks it, trimming every field on the way.
pub fn parse(text: &str) -> Result<EndpointPack, CoreError> {
    let mut pack: EndpointPack = serde_json::from_str(text)
        .map_err(|e| CoreError::invalid("text", format!("not an endpoint pack: {e}")))?;
    if pack.format != FORMAT {
        return Err(CoreError::invalid(
            "format",
            format!("expected \"{FORMAT}\", found \"{}\"", pack.format),
        ));
    }
    if pack.version != VERSION {
        return Err(CoreError::Version {
            found: pack.version,
            supported: VERSION,
        });
    }
    validate(&mut pack)?;
    Ok(pack)
}

/// Refuses a pack anirss could not use: no endpoints, an empty or repeated
/// name, a kind anirss does not know, a URL that is not http(s), an `rss`
/// URL with nowhere to put the query.
pub fn validate(pack: &mut EndpointPack) -> Result<(), CoreError> {
    pack.name = pack.name.trim().to_string();
    if pack.endpoints.is_empty() {
        return Err(CoreError::invalid(
            "endpoints",
            "the pack lists no endpoints",
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for ep in &mut pack.endpoints {
        ep.name = ep.name.trim().to_string();
        ep.kind = ep.kind.trim().to_ascii_lowercase();
        ep.url = ep.url.trim().to_string();
        if ep.name.is_empty() {
            return Err(CoreError::invalid("name", "an endpoint has no name"));
        }
        // anirss's own rule for an [[endpoint]] name.
        if !ep
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        {
            return Err(CoreError::invalid(
                "name",
                format!("{}: use only letters, digits, '.', '_' and '-'", ep.name),
            ));
        }
        if !seen.insert(ep.name.to_lowercase()) {
            return Err(CoreError::invalid(
                "name",
                format!("{} is listed twice", ep.name),
            ));
        }
        if !(ep.url.starts_with("https://") || ep.url.starts_with("http://")) {
            return Err(CoreError::invalid(
                "url",
                format!("{}: the URL must start with http:// or https://", ep.name),
            ));
        }
        match ep.kind.as_str() {
            "nyaa" => {}
            "rss" if ep.url.contains("{query}") => {}
            "rss" => {
                return Err(CoreError::invalid(
                    "url",
                    format!("{}: an rss endpoint needs {{query}} in its URL", ep.name),
                ));
            }
            other => {
                return Err(CoreError::invalid(
                    "kind",
                    format!(
                        "{}: unknown kind \"{other}\", expected nyaa or rss",
                        ep.name
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// The list after an import: what was there keeps its place, an incoming
/// endpoint with a name already in use replaces that one in place, and the
/// rest go on the end in the pack's order. Names compare without case.
pub fn merge(existing: &[PackEndpoint], incoming: &[PackEndpoint]) -> Vec<PackEndpoint> {
    let mut out = existing.to_vec();
    for ep in incoming {
        match out
            .iter_mut()
            .find(|e| e.name.eq_ignore_ascii_case(&ep.name))
        {
            Some(slot) => *slot = ep.clone(),
            None => out.push(ep.clone()),
        }
    }
    out
}

/// One endpoint from a URL someone pasted, so setting up takes a single
/// paste. A URL with `{query}` is used as it is. Otherwise the URL's shape
/// decides: a nyaa-style site becomes a `nyaa` endpoint at its root, a feed
/// gets `{query}` in its search parameter, a torznab API gets `t=search`.
/// No site is named anywhere a user sees; these are shapes, not a list.
/// `name` empty takes one from the host.
pub fn from_url(url: &str, name: &str) -> Result<PackEndpoint, CoreError> {
    let url = url.trim();
    let (scheme, rest) = url
        .split_once("://")
        .filter(|(s, _)| matches!(s.to_ascii_lowercase().as_str(), "http" | "https"))
        .ok_or_else(|| CoreError::invalid("url", "the URL must start with http:// or https://"))?;
    let (before_query, query) = rest.split_once('?').unwrap_or((rest, ""));
    let (host, path) = before_query.split_once('/').unwrap_or((before_query, ""));
    let host = host.to_ascii_lowercase();
    if host.is_empty() {
        return Err(CoreError::invalid("url", "the URL has no host"));
    }
    let params: Vec<(&str, &str)> = query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| p.split_once('=').unwrap_or((p, "")))
        .collect();
    let param = |k: &str| {
        params
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(k))
            .map(|(_, v)| v.to_string())
            .filter(|v| !v.is_empty())
    };
    let label = host
        .split(':')
        .next()
        .unwrap_or("")
        .trim_start_matches("www.")
        .split('.')
        .next()
        .unwrap_or("")
        .to_string();

    let (kind, url, category, filter, guessed) = if url.contains("{query}") {
        ("rss", url.to_string(), None, None, label.clone())
    } else if label.contains("nyaa") {
        // A nyaa-style site: anirss builds the search from the root.
        (
            "nyaa",
            format!("{scheme}://{host}/"),
            Some(param("c").unwrap_or_else(|| "1_0".to_string())),
            Some(param("f").unwrap_or_else(|| "0".to_string())),
            label.clone(),
        )
    } else if path.contains("torznab") || param("t").is_some() && param("apikey").is_some() {
        // A torznab API (Jackett, Prowlarr): search mode with the words in q.
        let mut kept: Vec<String> = params
            .iter()
            .filter(|(k, _)| !matches!(k.to_ascii_lowercase().as_str(), "t" | "q"))
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        kept.push("t=search".to_string());
        kept.push("q={query}".to_string());
        let indexer = path
            .split('/')
            .skip_while(|s| *s != "indexers")
            .nth(1)
            .filter(|s| !s.is_empty() && *s != "all")
            .map(String::from)
            .unwrap_or(label.clone());
        (
            "rss",
            format!("{scheme}://{before_query}?{}", kept.join("&")),
            None,
            None,
            indexer,
        )
    } else {
        // A feed: the search words go in its search parameter, `s` (the
        // WordPress one) when it has none.
        let key = ["q", "s", "search", "query", "term"]
            .into_iter()
            .find(|k| params.iter().any(|(p, _)| p.eq_ignore_ascii_case(k)))
            .unwrap_or("s");
        let mut kept: Vec<String> = params
            .iter()
            .filter(|(k, _)| !k.eq_ignore_ascii_case(key))
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        kept.push(format!("{key}={{query}}"));
        let guessed = label.split('-').next().unwrap_or(&label).to_string();
        (
            "rss",
            format!("{scheme}://{before_query}?{}", kept.join("&")),
            None,
            None,
            guessed,
        )
    };
    let name = match name.trim() {
        "" => guessed
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
            .collect::<String>(),
        given => given.to_string(),
    };
    let mut one = pack(
        "",
        vec![PackEndpoint {
            name: if name.is_empty() {
                "search".into()
            } else {
                name
            },
            kind: kind.to_string(),
            url,
            category,
            filter,
        }],
        None,
    );
    validate(&mut one)?;
    Ok(one.endpoints.remove(0))
}

/// The endpoints in an anirss `config.toml`, as a pack, with its best-fit
/// ranking. On-demand endpoints (`start`, `stop`, `ready`) are left out:
/// anirss takes no commands over its API, so they stay hand-written.
pub fn from_anirss_config(text: &str, name: &str) -> Result<EndpointPack, CoreError> {
    let doc: toml::Table = toml::from_str(text)
        .map_err(|e| CoreError::invalid("config", format!("anirss's config.toml: {e}")))?;
    let field = |t: &toml::Table, k: &str| {
        t.get(k)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let endpoints: Vec<PackEndpoint> = doc
        .get("endpoint")
        .and_then(|v| v.as_array())
        .map(|a| a.as_slice())
        .unwrap_or_default()
        .iter()
        .filter_map(|v| v.as_table())
        .filter(|t| {
            !["start", "stop", "ready"]
                .iter()
                .any(|k| t.contains_key(*k))
        })
        .filter_map(|t| {
            Some(PackEndpoint {
                name: field(t, "name")?,
                kind: field(t, "kind").unwrap_or_else(|| "nyaa".to_string()),
                url: field(t, "url")?,
                category: field(t, "category"),
                filter: field(t, "filter"),
            })
        })
        .collect();
    let best_fit = doc
        .get("bestfit")
        .and_then(|v| v.as_table())
        .map(|b| BestFitPrefs {
            preferred_groups: b
                .get("preferred_groups")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|g| g.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            preferred_resolution: field(b, "preferred_resolution"),
        });
    let mut out = pack(name, endpoints, best_fit);
    validate(&mut out)?;
    Ok(out)
}

/// A pack of the given endpoints, ready to write out.
pub fn pack(
    name: &str,
    endpoints: Vec<PackEndpoint>,
    best_fit: Option<BestFitPrefs>,
) -> EndpointPack {
    EndpointPack {
        format: FORMAT.to_string(),
        version: VERSION,
        name: name.to_string(),
        endpoints,
        best_fit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(name: &str, kind: &str, url: &str) -> PackEndpoint {
        PackEndpoint {
            name: name.into(),
            kind: kind.into(),
            url: url.into(),
            category: None,
            filter: None,
        }
    }

    const PACK: &str = r#"{
        "format": "anibeam-endpoints",
        "version": 1,
        "name": " Example pack ",
        "endpoints": [
            {"name": "first", "kind": "NYAA", "url": "https://one.example/", "category": "1_0"},
            {"name": "second", "kind": "rss", "url": "https://two.example/rss?q={query}"}
        ],
        "best_fit": {"preferred_groups": ["GroupA"], "preferred_resolution": "1080"}
    }"#;

    #[test]
    fn parse_reads_and_trims_a_pack() {
        let p = parse(PACK).unwrap();
        assert_eq!(p.name, "Example pack");
        assert_eq!(p.endpoints.len(), 2);
        assert_eq!(p.endpoints[0].kind, "nyaa");
        assert_eq!(p.endpoints[0].category.as_deref(), Some("1_0"));
        assert_eq!(p.best_fit.unwrap().preferred_groups, vec!["GroupA"]);
    }

    #[test]
    fn parse_refuses_another_format_or_version() {
        let other = PACK.replace("anibeam-endpoints", "anibeam-export");
        assert!(matches!(parse(&other), Err(CoreError::Invalid { .. })));
        let newer = PACK.replace("\"version\": 1", "\"version\": 2");
        assert!(matches!(
            parse(&newer),
            Err(CoreError::Version {
                found: 2,
                supported: 1
            })
        ));
    }

    #[test]
    fn validate_refuses_what_anirss_cannot_use() {
        let bad = [
            vec![],
            vec![ep("", "nyaa", "https://a.example/")],
            vec![ep("a b", "nyaa", "https://a.example/")],
            vec![
                ep("a", "nyaa", "https://a.example/"),
                ep("A", "nyaa", "https://b.example/"),
            ],
            vec![ep("a", "torznab", "https://a.example/")],
            vec![ep("a", "rss", "https://a.example/rss")],
            vec![ep("a", "nyaa", "ftp://a.example/")],
        ];
        for endpoints in bad {
            let mut p = pack("x", endpoints.clone(), None);
            assert!(validate(&mut p).is_err(), "{endpoints:?} passed");
        }
    }

    #[test]
    fn merge_keeps_order_and_replaces_by_name() {
        let existing = vec![
            ep("a", "nyaa", "https://a.example/"),
            ep("b", "nyaa", "https://b.example/"),
        ];
        let incoming = vec![
            ep("c", "nyaa", "https://c.example/"),
            ep("B", "nyaa", "https://b2.example/"),
        ];
        let out = merge(&existing, &incoming);
        let names: Vec<_> = out.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["a", "B", "c"]);
        assert_eq!(out[1].url, "https://b2.example/");
    }

    #[test]
    fn a_pasted_url_becomes_an_endpoint_by_its_shape() {
        let e = from_url("https://nyaa.si/?page=rss&q=x&c=1_2&f=2", "").unwrap();
        assert_eq!((e.name.as_str(), e.kind.as_str()), ("nyaa", "nyaa"));
        assert_eq!(e.url, "https://nyaa.si/");
        assert_eq!(e.category.as_deref(), Some("1_2"));
        assert_eq!(e.filter.as_deref(), Some("2"));

        let e = from_url("https://nyaa.si", "").unwrap();
        assert_eq!(e.category.as_deref(), Some("1_0"));

        let e = from_url(
            "https://www.erai-raws.info/episodes/feed/?res=1080p&type=torrent&token=abc",
            "",
        )
        .unwrap();
        assert_eq!((e.name.as_str(), e.kind.as_str()), ("erai", "rss"));
        assert_eq!(
            e.url,
            "https://www.erai-raws.info/episodes/feed/?res=1080p&type=torrent&token=abc&s={query}"
        );

        let e = from_url(
            "http://127.0.0.1:9117/api/v2.0/indexers/1337x/results/torznab/api?apikey=k&t=caps",
            "",
        )
        .unwrap();
        assert_eq!(e.name, "1337x");
        assert_eq!(
            e.url,
            "http://127.0.0.1:9117/api/v2.0/indexers/1337x/results/torznab/api?apikey=k&t=search&q={query}"
        );

        let e = from_url("https://feed.example/rss?q=old&x=1", "mine").unwrap();
        assert_eq!(e.name, "mine");
        assert_eq!(e.url, "https://feed.example/rss?x=1&q={query}");

        let given = "https://a.example/s/{query}/rss";
        assert_eq!(from_url(given, "").unwrap().url, given);
        assert!(from_url("ftp://a.example/", "").is_err());
        assert!(from_url("https://a.example/", "bad name").is_err());
    }

    #[test]
    fn an_anirss_config_exports_without_its_on_demand_endpoints() {
        let config = r#"
            [[endpoint]]
            name = "feed"
            kind = "rss"
            url = "https://a.example/rss?s={query}&token=t"

            [[endpoint]]
            name = "site"
            kind = "nyaa"
            url = "https://b.example/"
            category = "1_0"
            filter = "0"

            [[endpoint]]
            name = "pod"
            kind = "rss"
            url = "http://127.0.0.1:9117/api?q={query}"
            start = "podman pod start x"

            [bestfit]
            preferred_groups = ["GroupA", "GroupB"]
            preferred_resolution = "highest"
            source_order = ["WEB"]
        "#;
        let p = from_anirss_config(config, "mine").unwrap();
        let names: Vec<_> = p.endpoints.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["feed", "site"]);
        assert!(p.endpoints[0].url.contains("token=t"));
        assert_eq!(p.endpoints[1].filter.as_deref(), Some("0"));
        let bf = p.best_fit.unwrap();
        assert_eq!(bf.preferred_groups, ["GroupA", "GroupB"]);
        assert_eq!(bf.preferred_resolution.as_deref(), Some("highest"));
    }

    #[test]
    fn a_written_pack_reads_back() {
        let p = pack("mine", vec![ep("a", "nyaa", "https://a.example/")], None);
        let text = serde_json::to_string(&p).unwrap();
        assert_eq!(parse(&text).unwrap(), p);
    }
}
