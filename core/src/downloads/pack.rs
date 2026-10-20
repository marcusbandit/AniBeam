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
    fn a_written_pack_reads_back() {
        let p = pack("mine", vec![ep("a", "nyaa", "https://a.example/")], None);
        let text = serde_json::to_string(&p).unwrap();
        assert_eq!(parse(&text).unwrap(), p);
    }
}
