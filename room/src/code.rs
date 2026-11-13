//! Room codes and the keys a relay hands out.
//!
//! A code is 6 characters from [`ALPHABET`], which leaves out I, O, 0 and 1 so a
//! code read aloud or off a screen cannot be mistyped. The caller supplies the
//! randomness, so this works the same in the native relay and in the Worker.

/// The characters a code is made of: no I, O, 0 or 1.
pub const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// How many characters a code has.
pub const LEN: usize = 6;

/// A code from 6 random bytes. The alphabet has 32 characters, so the low 5 bits
/// of each byte pick one with no bias.
pub fn generate(random: [u8; LEN]) -> String {
    random
        .iter()
        .map(|b| ALPHABET[(b & 31) as usize] as char)
        .collect()
}

/// A code as a person typed or pasted it, made canonical: uppercase, with spaces
/// and dashes dropped. `None` when anything else is left or the length is wrong.
pub fn normalize(input: &str) -> Option<String> {
    let mut out = String::with_capacity(LEN);
    for c in input.chars() {
        if c == ' ' || c == '-' {
            continue;
        }
        let c = c.to_ascii_uppercase();
        if !c.is_ascii() || !ALPHABET.contains(&(c as u8)) {
            return None;
        }
        out.push(c);
    }
    (out.len() == LEN).then_some(out)
}

/// A 32 character lowercase hex key from 16 random bytes: the host key a relay
/// returns when it creates a room.
pub fn key(random: [u8; 16]) -> String {
    hex(&random)
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_uses_the_alphabet_only() {
        let code = generate([0, 31, 32, 255, 8, 100]);
        assert_eq!(code.len(), LEN);
        assert_eq!(code, "A9A9JE");
        for b in 0..=255u8 {
            let code = generate([b; LEN]);
            assert!(code.bytes().all(|c| ALPHABET.contains(&c)), "{code}");
        }
    }

    #[test]
    fn generate_never_makes_lookalikes() {
        for b in 0..=255u8 {
            let code = generate([b, b.wrapping_add(1), 0, 0, 0, 0]);
            assert!(!code.contains(['I', 'O', '0', '1']), "{code}");
        }
    }

    #[test]
    fn every_code_normalizes_to_itself() {
        for b in 0..=255u8 {
            let code = generate([b, b ^ 7, b ^ 13, b ^ 21, b ^ 29, b ^ 3]);
            assert_eq!(normalize(&code).as_deref(), Some(code.as_str()));
        }
    }

    #[test]
    fn normalize_uppercases_and_drops_spaces_and_dashes() {
        assert_eq!(normalize("k7qm2x").as_deref(), Some("K7QM2X"));
        assert_eq!(normalize("K7Q-M2X").as_deref(), Some("K7QM2X"));
        assert_eq!(normalize(" k7q m2x ").as_deref(), Some("K7QM2X"));
        assert_eq!(normalize("k-7-q-m-2-x").as_deref(), Some("K7QM2X"));
    }

    #[test]
    fn normalize_refuses_anything_else() {
        assert_eq!(normalize(""), None);
        assert_eq!(normalize("K7QM2"), None);
        assert_eq!(normalize("K7QM2XA"), None);
        // I, O, 0 and 1 are not in the alphabet.
        assert_eq!(normalize("K7QM2I"), None);
        assert_eq!(normalize("K7QM20"), None);
        assert_eq!(normalize("K7QM2_"), None);
        assert_eq!(normalize("K7QM2\u{0130}"), None);
        assert_eq!(normalize("K7QM2\u{00e9}"), None);
        assert_eq!(normalize("K7\tQM2X"), None);
    }

    #[test]
    fn key_is_32_lowercase_hex() {
        let k = key([0xab; 16]);
        assert_eq!(k, "ab".repeat(16));
        let k = key([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255]);
        assert_eq!(k, "000102030405060708090a0b0c0d0eff");
    }
}
