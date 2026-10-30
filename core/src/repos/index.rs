//! A repo's file, read and checked: the envelope, the ed25519 signature over the payload, the
//! index inside it, and each endpoint definition. Nothing here touches the store or the
//! network, so every check is a plain function a test can call.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::contract::{CoreError, EndpointField};
use crate::releases::endpoints::VALID_KINDS;

/// The envelope's `format`.
pub const FORMAT: &str = "anibeam-endpoint-repo";
/// The envelope versions this core reads.
pub const VERSION: u32 = 1;
/// A repo file larger than this is refused before it is parsed.
pub const MAX_BYTES: usize = 1 << 20;

/// What went wrong with a repo, as a fixed word a shell can branch on and a sentence for people.
#[derive(Clone, Debug, PartialEq)]
pub struct RepoFault {
    pub code: &'static str,
    pub message: String,
}

impl RepoFault {
    pub fn new(code: &'static str, message: impl Into<String>) -> RepoFault {
        RepoFault {
            code,
            message: message.into(),
        }
    }
}

impl From<RepoFault> for CoreError {
    fn from(f: RepoFault) -> CoreError {
        CoreError::Service {
            service: "Endpoint repo".into(),
            code: f.code.into(),
            message: f.message,
        }
    }
}

#[derive(Deserialize)]
struct Envelope {
    format: String,
    version: u32,
    payload: String,
    signature: String,
}

#[derive(Deserialize)]
struct RawIndex {
    name: String,
    #[serde(default)]
    nsfw: bool,
    key: String,
    serial: u64,
    #[serde(default)]
    endpoints: Vec<serde_json::Value>,
}

/// An index whose signature checked out against its own key. Whether that key is the one the
/// person trusted is the caller's question.
#[derive(Clone, Debug, PartialEq)]
pub struct Index {
    pub name: String,
    pub nsfw: bool,
    /// `ed25519:<base64>`, as the index states it.
    pub key: String,
    pub serial: u64,
    /// The definitions this core can run. One that is malformed, of a kind this core does not
    /// know, or meant for a newer core is left out rather than sinking the whole index.
    pub endpoints: Vec<Definition>,
    /// The payload exactly as it was signed, for the store.
    pub payload: Vec<u8>,
}

/// One endpoint as a repo defines it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Definition {
    pub id: String,
    pub name: String,
    pub version: u32,
    #[serde(default)]
    pub lang: String,
    pub kind: String,
    /// `{query}` plus one placeholder per `auth` key; filled at request time.
    pub url: String,
    #[serde(default)]
    pub auth: Vec<AuthField>,
    #[serde(default)]
    pub front: Option<String>,
    #[serde(default)]
    pub rate: Option<Rate>,
    #[serde(default)]
    pub nsfw: bool,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub min_core: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthField {
    pub key: String,
    pub label: String,
    #[serde(default = "yes")]
    pub secret: bool,
    #[serde(default)]
    pub help: String,
    #[serde(default)]
    pub help_url: Option<String>,
}

fn yes() -> bool {
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rate {
    pub per_minute: u32,
}

impl Definition {
    /// The host its requests go to.
    pub fn host(&self) -> String {
        host_of(&self.url)
    }

    pub fn fields(&self) -> Vec<EndpointField> {
        self.auth
            .iter()
            .map(|a| EndpointField {
                key: a.key.clone(),
                label: a.label.clone(),
                secret: a.secret,
                help: a.help.clone(),
                help_url: a.help_url.clone(),
            })
            .collect()
    }

    /// The `auth` keys, sorted, for comparing two versions.
    pub fn auth_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.auth.iter().map(|a| a.key.clone()).collect();
        keys.sort();
        keys
    }

    /// Whether credentials stored for `self` may go with `next` unasked: the same fields, and
    /// the same hosts for the address and the front.
    pub fn keeps_credentials(&self, next: &Definition) -> bool {
        self.auth_keys() == next.auth_keys()
            && self.host() == next.host()
            && self.front == next.front
    }

    /// The address with every `auth` placeholder filled, percent-encoded; `{query}` stays for
    /// the search to fill. None when a value is missing.
    pub fn filled_url(&self, value: impl Fn(&str) -> Option<String>) -> Option<String> {
        let mut url = self.url.clone();
        for field in &self.auth {
            let v = value(&field.key)?;
            url = url.replace(
                &format!("{{{}}}", field.key),
                &crate::releases::endpoints::quote(v.as_bytes(), ""),
            );
        }
        Some(url)
    }
}

/// The host of an http(s) address, lower case; empty when there is none.
pub fn host_of(url: &str) -> String {
    let rest = url.split_once("://").map_or("", |(_, r)| r);
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = &rest[..end];
    let host = host.rsplit_once('@').map_or(host, |(_, h)| h);
    host.split(':').next().unwrap_or("").to_lowercase()
}

/// A short hash of a key for people to compare: the first 16 bytes of its SHA-256, in groups.
pub fn fingerprint(key: &str) -> String {
    let raw = key.strip_prefix("ed25519:").unwrap_or(key);
    let bytes = B64.decode(raw).unwrap_or_else(|_| raw.as_bytes().to_vec());
    let digest = Sha256::digest(&bytes);
    digest[..16]
        .chunks(2)
        .map(|c| format!("{:02X}{:02X}", c[0], c[1]))
        .collect::<Vec<_>>()
        .join(" ")
}

fn verifying_key(key: &str) -> Result<VerifyingKey, RepoFault> {
    let raw = key
        .strip_prefix("ed25519:")
        .ok_or_else(|| RepoFault::new("format", "the index's key is not an ed25519 key"))?;
    let bytes: [u8; 32] = B64
        .decode(raw)
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| RepoFault::new("format", "the index's key is not 32 bytes of base64"))?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|_| RepoFault::new("format", "the index's key is not a valid ed25519 key"))
}

/// Reads a repo file: the envelope, then the payload's signature against the key the payload
/// itself names, then the index.
pub fn parse(bytes: &[u8], core_version: &str) -> Result<Index, RepoFault> {
    if bytes.len() > MAX_BYTES {
        return Err(RepoFault::new(
            "format",
            "the file is too large to be a repo",
        ));
    }
    let env: Envelope = serde_json::from_slice(bytes)
        .map_err(|_| RepoFault::new("format", "this is not an AniBeam endpoint repo"))?;
    if env.format != FORMAT {
        return Err(RepoFault::new(
            "format",
            "this is not an AniBeam endpoint repo",
        ));
    }
    if env.version != VERSION {
        return Err(RepoFault::new(
            "version",
            format!(
                "the repo is format version {}; this AniBeam reads version {VERSION}. Update AniBeam.",
                env.version
            ),
        ));
    }
    let payload = B64
        .decode(env.payload.trim())
        .map_err(|_| RepoFault::new("format", "the repo's payload is not base64"))?;
    let signature = B64
        .decode(env.signature.trim())
        .ok()
        .and_then(|b| Signature::from_slice(&b).ok())
        .ok_or_else(|| RepoFault::new("signature", "the repo's signature is malformed"))?;
    let raw: RawIndex = serde_json::from_slice(&payload)
        .map_err(|e| RepoFault::new("format", format!("the repo's index does not read: {e}")))?;
    let key = verifying_key(&raw.key)?;
    key.verify_strict(&payload, &signature).map_err(|_| {
        RepoFault::new(
            "signature",
            "the repo's signature does not match its contents",
        )
    })?;
    let mut endpoints: Vec<Definition> = Vec::new();
    for value in raw.endpoints {
        let Ok(def) = serde_json::from_value::<Definition>(value) else {
            continue;
        };
        if check(&def).is_err() || !runs_on(&def, core_version) {
            continue;
        }
        if endpoints.iter().any(|d| d.id == def.id) {
            continue;
        }
        endpoints.push(def);
    }
    Ok(Index {
        name: raw.name,
        nsfw: raw.nsfw,
        key: raw.key,
        serial: raw.serial,
        endpoints,
        payload,
    })
}

/// Reads a payload the store kept, which passed its checks when it was fetched.
pub fn reread(payload: &[u8], core_version: &str) -> Vec<Definition> {
    let Ok(raw) = serde_json::from_slice::<RawIndex>(payload) else {
        return Vec::new();
    };
    raw.endpoints
        .into_iter()
        .filter_map(|v| serde_json::from_value::<Definition>(v).ok())
        .filter(|d| check(d).is_ok() && runs_on(d, core_version))
        .collect()
}

fn plain_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

/// The placeholders in a URL template, in order.
fn placeholders(url: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = url;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else { break };
        out.push(&after[..close]);
        rest = &after[close + 1..];
    }
    out
}

/// A definition this core can run safely: a known kind, an https address on a real host,
/// and placeholders that are exactly `{query}` and its own `auth` keys.
pub fn check(def: &Definition) -> Result<(), String> {
    check_with(def, false)
}

/// The same for an endpoint the person typed in, which may be plain http: a Jackett or a
/// Prowlarr on this machine or the home network.
pub fn check_custom(def: &Definition) -> Result<(), String> {
    check_with(def, true)
}

fn check_with(def: &Definition, http: bool) -> Result<(), String> {
    if !plain_id(&def.id) {
        return Err(format!(
            "{:?}: the id may hold a-z, 0-9, '.', '_', '-'",
            def.id
        ));
    }
    if def.name.trim().is_empty() {
        return Err(format!("{}: no name", def.id));
    }
    if !VALID_KINDS.contains(&def.kind.as_str()) {
        return Err(format!("{}: unknown kind {:?}", def.id, def.kind));
    }
    let scheme_ok = def.url.starts_with("https://") || (http && def.url.starts_with("http://"));
    if !scheme_ok || def.host().is_empty() {
        return Err(format!(
            "{}: the address must start with {}",
            def.id,
            if http {
                "http:// or https://"
            } else {
                "https://"
            }
        ));
    }
    let mut keys = Vec::new();
    for field in &def.auth {
        if !plain_id(&field.key) || field.key == "query" || keys.contains(&field.key.as_str()) {
            return Err(format!("{}: bad auth key {:?}", def.id, field.key));
        }
        if !def.url.contains(&format!("{{{}}}", field.key)) {
            return Err(format!(
                "{}: {{{}}} is not in the address",
                def.id, field.key
            ));
        }
        keys.push(field.key.as_str());
    }
    for p in placeholders(&def.url) {
        if p != "query" && !keys.contains(&p) {
            return Err(format!("{}: unknown placeholder {{{p}}}", def.id));
        }
    }
    // The host must be literal: a placeholder there would let a credential pick where it goes.
    if def.host().contains('{') {
        return Err(format!("{}: a placeholder in the host", def.id));
    }
    let has_query = def.url.contains("{query}");
    if def.kind == "nyaa" && has_query {
        return Err(format!("{}: nyaa adds the query itself", def.id));
    }
    if crate::releases::endpoints::needs_query(&def.kind) && !has_query {
        return Err(format!("{}: the address needs {{query}}", def.id));
    }
    if let Some(front) = &def.front
        && (front.is_empty() || front.contains(['/', ':', '{', '@']))
    {
        return Err(format!("{}: the front must be a bare host", def.id));
    }
    if def.rate.is_some_and(|r| r.per_minute == 0) {
        return Err(format!("{}: a rate of zero", def.id));
    }
    Ok(())
}

/// Whether a definition's `min_core` is at most this core's version.
pub fn runs_on(def: &Definition, core_version: &str) -> bool {
    let Some(min) = &def.min_core else {
        return true;
    };
    match (semver(min), semver(core_version)) {
        (Some(min), Some(have)) => min <= have,
        _ => false,
    }
}

/// A version as numbers, "v" and any pre-release part dropped.
pub fn semver_of(v: &str) -> Option<(u64, u64, u64)> {
    semver(v)
}

fn semver(v: &str) -> Option<(u64, u64, u64)> {
    let v = v.trim().trim_start_matches('v');
    let v = v.split(['-', '+']).next()?;
    let mut parts = v.split('.').map(|p| p.parse::<u64>().ok());
    Some((
        parts.next()??,
        parts.next().flatten().unwrap_or(0),
        parts.next().flatten().unwrap_or(0),
    ))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    pub fn signing_key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    pub fn key_text(k: &SigningKey) -> String {
        format!("ed25519:{}", B64.encode(k.verifying_key().to_bytes()))
    }

    /// A repo file of `index`, signed by `k`.
    pub fn sign(index: &serde_json::Value, k: &SigningKey) -> Vec<u8> {
        let payload = serde_json::to_vec(index).unwrap();
        let sig = k.sign(&payload);
        serde_json::to_vec(&serde_json::json!({
            "format": FORMAT,
            "version": 1,
            "payload": B64.encode(&payload),
            "signature": B64.encode(sig.to_bytes()),
        }))
        .unwrap()
    }

    fn def(id: &str, kind: &str, url: &str) -> serde_json::Value {
        serde_json::json!({"id": id, "name": id, "version": 1, "kind": kind, "url": url})
    }

    fn index(k: &SigningKey, endpoints: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({"name": "Test", "key": key_text(k), "serial": 10, "endpoints": endpoints})
    }

    #[test]
    fn a_signed_index_reads() {
        let k = signing_key(1);
        let file = sign(
            &index(&k, vec![def("nyaa", "nyaa", "https://nyaa.si/")]),
            &k,
        );
        let idx = parse(&file, "2.1.0").unwrap();
        assert_eq!(idx.name, "Test");
        assert_eq!(idx.serial, 10);
        assert_eq!(idx.endpoints.len(), 1);
        assert_eq!(idx.key, key_text(&k));
    }

    #[test]
    fn a_changed_payload_fails_the_signature() {
        let k = signing_key(1);
        let file = sign(&index(&k, vec![]), &k);
        let mut env: serde_json::Value = serde_json::from_slice(&file).unwrap();
        let mut payload = B64.decode(env["payload"].as_str().unwrap()).unwrap();
        let at = payload.windows(2).position(|w| w == b"10").unwrap();
        payload[at] = b'9';
        env["payload"] = B64.encode(&payload).into();
        let err = parse(&serde_json::to_vec(&env).unwrap(), "2.1.0").unwrap_err();
        assert_eq!(err.code, "signature");
    }

    #[test]
    fn an_index_signed_by_another_key_than_it_names_fails() {
        let named = signing_key(1);
        let signer = signing_key(2);
        let file = sign(&index(&named, vec![]), &signer);
        assert_eq!(parse(&file, "2.1.0").unwrap_err().code, "signature");
    }

    #[test]
    fn a_newer_envelope_asks_for_an_update() {
        let env =
            serde_json::json!({"format": FORMAT, "version": 2, "payload": "", "signature": ""});
        let err = parse(&serde_json::to_vec(&env).unwrap(), "2.1.0").unwrap_err();
        assert_eq!(err.code, "version");
    }

    #[test]
    fn something_else_is_not_a_repo() {
        assert_eq!(parse(b"<html>", "2.1.0").unwrap_err().code, "format");
        let other =
            serde_json::json!({"format": "mihon", "version": 1, "payload": "", "signature": ""});
        assert_eq!(
            parse(&serde_json::to_vec(&other).unwrap(), "2.1.0")
                .unwrap_err()
                .code,
            "format"
        );
    }

    #[test]
    fn a_bad_definition_is_left_out_and_the_rest_stays() {
        let k = signing_key(1);
        let mut too_new = def("later", "rss", "https://a.example/{query}");
        too_new["min_core"] = "9.0.0".into();
        let file = sign(
            &index(
                &k,
                vec![
                    def("ok", "rss", "https://a.example/?q={query}"),
                    def("plain-http", "rss", "http://a.example/?q={query}"),
                    def("noquery", "rss", "https://a.example/feed"),
                    def("weird", "torznab", "https://a.example/{query}"),
                    def("stray", "rss", "https://a.example/{query}&k={key}"),
                    def("Upper", "rss", "https://a.example/{query}"),
                    too_new,
                    serde_json::json!({"id": "broken"}),
                ],
            ),
            &k,
        );
        let ids: Vec<String> = parse(&file, "2.1.0")
            .unwrap()
            .endpoints
            .into_iter()
            .map(|d| d.id)
            .collect();
        assert_eq!(ids, vec!["ok"]);
    }

    #[test]
    fn auth_keys_must_sit_in_the_address_but_not_in_the_host() {
        let mut d: Definition = serde_json::from_value(def(
            "erai",
            "rss",
            "https://erai.example/feed?s={query}&token={token}",
        ))
        .unwrap();
        d.auth = vec![AuthField {
            key: "token".into(),
            label: "Token".into(),
            secret: true,
            help: String::new(),
            help_url: None,
        }];
        assert!(check(&d).is_ok());
        d.url = "https://erai.example/feed?s={query}".into();
        assert!(check(&d).is_err());
        d.url = "https://{token}.example/feed?s={query}".into();
        assert!(check(&d).is_err());
    }

    #[test]
    fn a_tosho_or_rss_address_needs_the_query() {
        for kind in ["tosho", "rss"] {
            let mut d: Definition =
                serde_json::from_value(def("t", kind, "https://feed.example/json")).unwrap();
            assert!(check(&d).is_err(), "{kind}");
            assert!(check_custom(&d).is_err(), "{kind}");
            d.url = "https://feed.example/json?q={query}".into();
            assert!(check(&d).is_ok(), "{kind}");
            assert!(check_custom(&d).is_ok(), "{kind}");
        }
    }

    #[test]
    fn filling_encodes_the_value_and_leaves_the_query() {
        let mut d: Definition = serde_json::from_value(def(
            "erai",
            "rss",
            "https://erai.example/feed?s={query}&token={token}",
        ))
        .unwrap();
        d.auth = vec![AuthField {
            key: "token".into(),
            label: "Token".into(),
            secret: true,
            help: String::new(),
            help_url: None,
        }];
        assert_eq!(
            d.filled_url(|_| Some("a b&c".into())).unwrap(),
            "https://erai.example/feed?s={query}&token=a%20b%26c"
        );
        assert_eq!(d.filled_url(|_| None), None);
    }

    #[test]
    fn credentials_follow_an_update_only_to_the_same_host_and_fields() {
        let a: Definition =
            serde_json::from_value(def("x", "rss", "https://a.example/?q={query}")).unwrap();
        let mut b = a.clone();
        b.version = 2;
        assert!(a.keeps_credentials(&b));
        b.url = "https://b.example/?q={query}".into();
        assert!(!a.keeps_credentials(&b));
        let mut c = a.clone();
        c.front = Some("front.example".into());
        assert!(!a.keeps_credentials(&c));
    }

    #[test]
    fn min_core_compares_as_versions() {
        let mut d: Definition =
            serde_json::from_value(def("x", "rss", "https://a.example/?q={query}")).unwrap();
        d.min_core = Some("2.1.0".into());
        assert!(runs_on(&d, "2.1.0"));
        assert!(runs_on(&d, "2.10.0"));
        assert!(!runs_on(&d, "2.0.9"));
        d.min_core = Some("nonsense".into());
        assert!(!runs_on(&d, "2.1.0"));
    }

    #[test]
    fn the_fingerprint_is_eight_groups() {
        let f = fingerprint(&key_text(&signing_key(1)));
        assert_eq!(f.split(' ').count(), 8);
        assert_ne!(f, fingerprint(&key_text(&signing_key(2))));
    }
}
