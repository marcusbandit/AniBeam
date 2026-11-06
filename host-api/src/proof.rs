//! Proof that a request comes from the device holding a key (#555).
//!
//! Every AniBeam makes an ed25519 key pair once. A host that offers
//! [`crate::FEATURE_PROOF`] says its public key in [`crate::Hello::key`]. A
//! client then signs each request it makes there and sends the proof in
//! [`HEADER`]:
//!
//! ```text
//! anibeam-proof: v1 <client key> <time> <nonce> <signature>
//! ```
//!
//! The key and the signature are lowercase hex. The signature covers
//! [`SCHEME`], the host's key, the time, the nonce, the method and the path
//! with its query, so a proof is good for one request to one host, only
//! around the time it was made, and only once: the host keeps the nonces it
//! has seen for as long as a proof stays fresh. A new scheme gets a new first
//! word, and a host that does not know it treats the proof as unreadable.
//!
//! A request without a proof is judged as before, by where it comes from; a
//! proof only adds: a request whose proof does not check out is refused, and
//! one from a key the host paired with is known to be that device.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// The request header that carries the proof.
pub const HEADER: &str = "anibeam-proof";
/// The first word of a proof in this version of the scheme.
pub const SCHEME: &str = "v1";
/// How far a proof's time may be from the host's clock, in seconds, either
/// way. A client signs with the host's time from its hello, so only the drift
/// since then counts.
pub const FRESH_FOR: i64 = 300;
/// The longest nonce a host reads.
const NONCE_MAX: usize = 64;

/// This device's key pair.
#[derive(Clone)]
pub struct DeviceKey(SigningKey);

impl DeviceKey {
    /// The key made from 32 random bytes, which are all a device stores.
    pub fn from_seed(seed: [u8; 32]) -> DeviceKey {
        DeviceKey(SigningKey::from_bytes(&seed))
    }

    /// The 32 bytes to store.
    pub fn seed(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// The public key, in hex, as a host stores it once paired.
    pub fn public(&self) -> String {
        hex(self.0.verifying_key().as_bytes())
    }

    /// The [`HEADER`] value for one request to the host whose key is
    /// `host_key`, made at the host's `time`, with a `nonce` used only once.
    pub fn sign(&self, host_key: &str, time: i64, nonce: &str, method: &str, path: &str) -> String {
        let me = self.public();
        let signature = self
            .0
            .sign(&message(host_key, &me, time, nonce, method, path));
        format!(
            "{SCHEME} {me} {time} {nonce} {}",
            hex(&signature.to_bytes())
        )
    }
}

/// A proof that checked out.
#[derive(Clone, Debug, PartialEq)]
pub struct Proven {
    /// The client's public key, in hex.
    pub key: String,
    pub time: i64,
    pub nonce: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProofError {
    /// Not a proof this host can read: the wrong number of parts, a scheme
    /// it does not know, a key or signature that is not hex of the right size.
    Unreadable,
    /// Signed by another key, or for another request or host.
    Forged,
    /// Made too far from the host's time.
    Stale,
}

/// Checks the [`HEADER`] value of a `method` request for `path` (with its
/// query) to the host whose key is `host_key`, at the host's time `now`.
/// Whether the nonce was seen before is the host's to check.
pub fn check(
    header: &str,
    host_key: &str,
    method: &str,
    path: &str,
    now: i64,
) -> Result<Proven, ProofError> {
    let parts: Vec<&str> = header.split(' ').collect();
    let [scheme, key, time, nonce, signature] = parts[..] else {
        return Err(ProofError::Unreadable);
    };
    if scheme != SCHEME
        || nonce.is_empty()
        || nonce.len() > NONCE_MAX
        || !nonce.bytes().all(|b| b.is_ascii_alphanumeric())
    {
        return Err(ProofError::Unreadable);
    }
    let time: i64 = time.parse().map_err(|_| ProofError::Unreadable)?;
    let public: [u8; 32] = unhex(key).ok_or(ProofError::Unreadable)?;
    let signature: [u8; 64] = unhex(signature).ok_or(ProofError::Unreadable)?;
    let public = VerifyingKey::from_bytes(&public).map_err(|_| ProofError::Unreadable)?;
    public
        .verify(
            &message(host_key, key, time, nonce, method, path),
            &Signature::from_bytes(&signature),
        )
        .map_err(|_| ProofError::Forged)?;
    if (now - time).abs() > FRESH_FOR {
        return Err(ProofError::Stale);
    }
    Ok(Proven {
        key: key.to_string(),
        time,
        nonce: nonce.to_string(),
    })
}

fn message(host: &str, me: &str, time: i64, nonce: &str, method: &str, path: &str) -> Vec<u8> {
    format!(
        "anibeam-proof {SCHEME}\n{host}\n{me}\n{time}\n{nonce}\n{}\n{path}",
        method.to_ascii_uppercase()
    )
    .into_bytes()
}

/// Lowercase hex.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Exactly `N` bytes out of lowercase or uppercase hex.
pub fn unhex<const N: usize>(text: &str) -> Option<[u8; N]> {
    if text.len() != N * 2 || !text.is_ascii() {
        return None;
    }
    let mut out = [0u8; N];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn phone() -> DeviceKey {
        DeviceKey::from_seed([7; 32])
    }

    fn host() -> String {
        DeviceKey::from_seed([9; 32]).public()
    }

    #[test]
    fn a_good_proof_names_the_key_that_made_it() {
        let header = phone().sign(&host(), NOW, "n1", "get", "/api/v1/snapshot");
        assert_eq!(
            check(&header, &host(), "GET", "/api/v1/snapshot", NOW + 10),
            Ok(Proven {
                key: phone().public(),
                time: NOW,
                nonce: "n1".into(),
            })
        );
    }

    #[test]
    fn a_seed_makes_the_same_key_again() {
        let again = DeviceKey::from_seed(phone().seed());
        assert_eq!(again.public(), phone().public());
        assert_eq!(phone().public().len(), 64);
    }

    #[test]
    fn a_proof_under_another_key_is_forged() {
        // The header names the phone's key, but another key signed it.
        let other = DeviceKey::from_seed([8; 32]);
        let signed = other.sign(&host(), NOW, "n1", "GET", "/api/v1/snapshot");
        let signature = signed.rsplit(' ').next().unwrap();
        let header = format!("v1 {} {NOW} n1 {signature}", phone().public());
        assert_eq!(
            check(&header, &host(), "GET", "/api/v1/snapshot", NOW),
            Err(ProofError::Forged)
        );
    }

    #[test]
    fn a_proof_is_good_for_one_request_to_one_host() {
        let header = phone().sign(&host(), NOW, "n1", "GET", "/api/v1/snapshot");
        let other_host = DeviceKey::from_seed([1; 32]).public();
        for (host_key, method, path) in [
            (host(), "GET", "/api/v1/posters/3"),
            (host(), "POST", "/api/v1/snapshot"),
            (host(), "GET", "/api/v1/snapshot?x=1"),
            (other_host, "GET", "/api/v1/snapshot"),
        ] {
            assert_eq!(
                check(&header, &host_key, method, path, NOW),
                Err(ProofError::Forged),
                "{method} {path}"
            );
        }
    }

    #[test]
    fn a_proof_from_too_long_ago_or_ahead_is_stale() {
        let old = phone().sign(&host(), NOW - FRESH_FOR - 1, "n1", "GET", "/");
        assert_eq!(
            check(&old, &host(), "GET", "/", NOW),
            Err(ProofError::Stale)
        );
        let ahead = phone().sign(&host(), NOW + FRESH_FOR + 1, "n1", "GET", "/");
        assert_eq!(
            check(&ahead, &host(), "GET", "/", NOW),
            Err(ProofError::Stale)
        );
        let edge = phone().sign(&host(), NOW - FRESH_FOR, "n1", "GET", "/");
        assert!(check(&edge, &host(), "GET", "/", NOW).is_ok());
    }

    #[test]
    fn what_is_not_a_proof_is_unreadable() {
        let good = phone().sign(&host(), NOW, "n1", "GET", "/");
        let parts: Vec<&str> = good.split(' ').collect();
        let with = |i: usize, v: &str| {
            let mut p = parts.clone();
            p[i] = v;
            p.join(" ")
        };
        for header in [
            String::new(),
            "v1".to_string(),
            with(0, "v2"),
            with(1, "zz"),
            with(2, "soon"),
            with(3, ""),
            with(3, "a-b"),
            with(3, &"n".repeat(65)),
            with(4, "00"),
            format!("{good} extra"),
        ] {
            assert_eq!(
                check(&header, &host(), "GET", "/", NOW),
                Err(ProofError::Unreadable),
                "{header:?}"
            );
        }
    }

    #[test]
    fn hex_goes_both_ways() {
        assert_eq!(hex(&[0, 15, 255]), "000fff");
        assert_eq!(unhex::<3>("000fff"), Some([0, 15, 255]));
        assert_eq!(unhex::<3>("000FFF"), Some([0, 15, 255]));
        assert_eq!(unhex::<3>("000ff"), None);
        assert_eq!(unhex::<3>("000fgf"), None);
    }
}
