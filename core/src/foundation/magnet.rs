//! Reading a magnet link's info-hash. Adding a torrent, matching a search
//! result to a torrent, reading a feed and scraping a tracker all read it the
//! same way, through here.

/// Whether `hash` is an info-hash: 40 hex digits or 32 base32 ones, in either case.
pub fn is_hash(hash: &str) -> bool {
    match hash.len() {
        40 => hash.bytes().all(|b| b.is_ascii_hexdigit()),
        32 => hash
            .bytes()
            .all(|b| b.is_ascii_alphabetic() || (b'2'..=b'7').contains(&b)),
        _ => false,
    }
}

/// The info-hash a magnet link names, as lowercase hex (the base32 form is
/// converted). None for anything that is not a magnet with a hash.
pub fn hash(link: &str) -> Option<String> {
    let query = link.trim().strip_prefix("magnet:?")?;
    for part in query.split('&') {
        let Some(v) = part
            .get(..3)
            .filter(|key| key.eq_ignore_ascii_case("xt="))
            .map(|_| &part[3..])
        else {
            continue;
        };
        let lower = v.to_ascii_lowercase();
        let Some(h) = lower
            .strip_prefix("urn:btih:")
            .or_else(|| lower.strip_prefix("urn%3abtih%3a"))
        else {
            continue;
        };
        if h.len() == 40 && h.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(h.to_string());
        }
        if h.len() == 32 {
            return base32_hex(h);
        }
    }
    None
}

/// [`hash`] as the 20 bytes a tracker is asked about.
pub fn hash_bytes(link: &str) -> Option<[u8; 20]> {
    let hex = hash(link)?;
    let mut out = [0u8; 20];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn base32_hex(s: &str) -> Option<String> {
    let mut bits: u64 = 0;
    let mut n = 0;
    let mut out = String::new();
    for c in s.chars() {
        let v = match c {
            'a'..='z' => c as u64 - 'a' as u64,
            '2'..='7' => c as u64 - '2' as u64 + 26,
            _ => return None,
        };
        bits = (bits << 5) | v;
        n += 5;
        if n >= 8 {
            n -= 8;
            out.push_str(&format!("{:02x}", (bits >> n) & 0xff));
            bits &= (1 << n) - 1;
        }
    }
    (out.len() == 40).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_magnet_gives_its_hash_in_hex_or_base32() {
        let hex = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            hash(&format!("magnet:?xt=urn:btih:{}&dn=x", hex.to_uppercase())).as_deref(),
            Some(hex)
        );
        // 20 bytes of 0x00 .. 0x13 in base32
        assert_eq!(
            hash("magnet:?dn=a&xt=urn:btih:AAAQEAYEAUDAOCAJBIFQYDIOB4IBCEQT&tr=x").as_deref(),
            Some("000102030405060708090a0b0c0d0e0f10111213")
        );
        assert_eq!(hash("https://tracker.example/download/1.torrent"), None);
        assert_eq!(hash("magnet:?xt=urn:btih:short"), None);
    }

    #[test]
    fn a_hash_is_hex_or_base32_of_the_right_length() {
        assert!(is_hash("0123456789abcdef0123456789ABCDEF01234567"));
        assert!(is_hash("AAAQEAYEAUDAOCAJBIFQYDIOB4IBCEQT"));
        assert!(!is_hash("AAAQEAYEAUDAOCAJBIFQYDIOB4IBCEQ1"));
        assert!(!is_hash("abc"));
    }

    #[test]
    fn the_bytes_are_the_hex_read_two_digits_at_a_time() {
        let bytes = hash_bytes("magnet:?xt=urn:btih:AAAQEAYEAUDAOCAJBIFQYDIOB4IBCEQT").unwrap();
        assert_eq!(bytes, std::array::from_fn::<u8, 20, _>(|i| i as u8));
        assert_eq!(hash_bytes("magnet:?xt=urn:btih:short"), None);
    }
}
