//! Sealed payloads between two members of a room (#889). The relay passes them
//! on and can read none of them: not a torrent link, an info hash, a file name
//! or a match id.
//!
//! Each member makes a key pair when it joins a room and says the public half in
//! its hello, so the room's member list carries everyone's. Two members agree on
//! a shared secret with X25519; HKDF-SHA256 draws the key from it, salted with
//! the room's code and bound to both public keys, so a key is good for one pair
//! in one room and nowhere else. Each payload is XChaCha20-Poly1305 under a
//! fresh random 24 byte nonce, its additional data the sender's public key then
//! the recipient's, so a payload sent one way cannot be passed back the other
//! way as if the other member had sealed it. On the wire it is the nonce and the
//! ciphertext together, in base64.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

/// What HKDF's info starts with, so these keys are for this and nothing else.
const INFO: &[u8] = b"anibeam room seal v1";

const NONCE: usize = 24;

/// This member's key pair for one room.
pub struct Keys {
    secret: StaticSecret,
    public: [u8; 32],
}

impl Keys {
    /// A fresh pair, from the system's randomness.
    pub fn new() -> Keys {
        Keys::from_secret(rand::random::<[u8; 32]>())
    }

    /// A pair from fixed secret bytes, for tests.
    pub fn from_secret(bytes: [u8; 32]) -> Keys {
        let secret = StaticSecret::from(bytes);
        let public = PublicKey::from(&secret).to_bytes();
        Keys { secret, public }
    }

    /// The public half as the hello carries it.
    pub fn public(&self) -> String {
        B64.encode(self.public)
    }

    /// The key this member shares with the holder of `theirs` in the room
    /// `code`. None for a key that is not one, or one that agrees on nothing
    /// (a low order point, which would make the secret known to anyone).
    fn shared(&self, code: &str, theirs: &[u8; 32]) -> Option<XChaCha20Poly1305> {
        let agreed = self.secret.diffie_hellman(&PublicKey::from(*theirs));
        if !agreed.was_contributory() {
            return None;
        }
        // Both ends must build the same info: the two keys in a fixed order
        let (a, b) = if self.public <= *theirs {
            (&self.public, theirs)
        } else {
            (theirs, &self.public)
        };
        let mut info = Vec::with_capacity(INFO.len() + 64);
        info.extend_from_slice(INFO);
        info.extend_from_slice(a);
        info.extend_from_slice(b);
        let mut key = [0u8; 32];
        Hkdf::<Sha256>::new(Some(code.as_bytes()), agreed.as_bytes())
            .expand(&info, &mut key)
            .ok()?;
        XChaCha20Poly1305::new_from_slice(&key).ok()
    }

    /// `plain` sealed for the holder of `theirs` (base64), in the room `code`.
    pub fn seal(&self, code: &str, theirs: &str, plain: &[u8]) -> Option<String> {
        let theirs = key_bytes(theirs)?;
        let cipher = self.shared(code, &theirs)?;
        let nonce: [u8; NONCE] = rand::random();
        let aad = pair(&self.public, &theirs);
        let sealed = cipher
            .encrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: plain,
                    aad: &aad,
                },
            )
            .ok()?;
        let mut out = Vec::with_capacity(NONCE + sealed.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&sealed);
        Some(B64.encode(out))
    }

    /// A payload the holder of `theirs` sealed for this member, opened. None
    /// when it was not sealed for this pair in this room, or was changed on the
    /// way.
    pub fn open(&self, code: &str, theirs: &str, data: &str) -> Option<Vec<u8>> {
        let theirs = key_bytes(theirs)?;
        let bytes = B64.decode(data.trim()).ok()?;
        if bytes.len() < NONCE {
            return None;
        }
        let (nonce, sealed) = bytes.split_at(NONCE);
        let nonce: [u8; NONCE] = nonce.try_into().ok()?;
        let cipher = self.shared(code, &theirs)?;
        let aad = pair(&theirs, &self.public);
        cipher
            .decrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: sealed,
                    aad: &aad,
                },
            )
            .ok()
    }
}

impl Default for Keys {
    fn default() -> Self {
        Keys::new()
    }
}

/// The additional data: who sealed it, then for whom.
fn pair(from: &[u8; 32], to: &[u8; 32]) -> [u8; 64] {
    let mut out = [0u8; 64];
    out[..32].copy_from_slice(from);
    out[32..].copy_from_slice(to);
    out
}

fn key_bytes(b64: &str) -> Option<[u8; 32]> {
    B64.decode(b64.trim()).ok()?.try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: &str = "K7QM2X";

    #[test]
    fn the_other_member_opens_it() {
        let (host, member) = (Keys::new(), Keys::new());
        let sealed = host
            .seal(CODE, &member.public(), b"magnet and match")
            .unwrap();
        assert_eq!(
            member.open(CODE, &host.public(), &sealed).as_deref(),
            Some(&b"magnet and match"[..])
        );
        // And the other way
        let back = member.seal(CODE, &host.public(), b"ask").unwrap();
        assert_eq!(
            host.open(CODE, &member.public(), &back).as_deref(),
            Some(&b"ask"[..])
        );
    }

    #[test]
    fn a_third_member_cannot_open_it() {
        let (host, member, third) = (Keys::new(), Keys::new(), Keys::new());
        let sealed = host.seal(CODE, &member.public(), b"secret").unwrap();
        assert_eq!(third.open(CODE, &host.public(), &sealed), None);
        assert_eq!(third.open(CODE, &member.public(), &sealed), None);
    }

    #[test]
    fn the_relay_sees_none_of_it() {
        let (host, member) = (Keys::new(), Keys::new());
        let plain = b"magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567";
        let sealed = host.seal(CODE, &member.public(), plain).unwrap();
        let wire = B64.decode(&sealed).unwrap();
        assert!(!wire.windows(8).any(|w| plain.windows(8).any(|p| p == w)));
        assert!(!sealed.contains("btih"));
        // A fresh nonce each time: the same payload never looks the same twice
        assert_ne!(sealed, host.seal(CODE, &member.public(), plain).unwrap());
    }

    #[test]
    fn another_room_or_a_changed_byte_does_not_open() {
        let (host, member) = (Keys::new(), Keys::new());
        let sealed = host.seal(CODE, &member.public(), b"x").unwrap();
        assert_eq!(member.open("ZZZZZZ", &host.public(), &sealed), None);
        let mut bytes = B64.decode(&sealed).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        assert_eq!(member.open(CODE, &host.public(), &B64.encode(bytes)), None);
        assert_eq!(member.open(CODE, &host.public(), "not base64!"), None);
        assert_eq!(member.open(CODE, &host.public(), "AAAA"), None);
    }

    #[test]
    fn a_payload_cannot_be_passed_back_as_the_other_side_s() {
        let (host, member) = (Keys::new(), Keys::new());
        let to_member = host.seal(CODE, &member.public(), b"offer").unwrap();
        // The host opening its own payload as if the member had sent it
        assert_eq!(host.open(CODE, &member.public(), &to_member), None);
    }

    #[test]
    fn a_key_that_is_no_key_seals_nothing() {
        let host = Keys::new();
        assert_eq!(host.seal(CODE, "short", b"x"), None);
        // The identity point agrees on nothing
        assert_eq!(host.seal(CODE, &B64.encode([0u8; 32]), b"x"), None);
    }
}
