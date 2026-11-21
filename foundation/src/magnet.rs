//! Reading a magnet link's info-hash. Adding a torrent, matching a search
//! result to a torrent, reading a feed and scraping a tracker all read it the
//! same way, through here.

/// Whether `hash` is an info-hash: 40 hex digits or 32 base32 ones, in either case. Anything
/// else is left out rather than put in a magnet, where it could add parameters of its own.
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
/// converted). The scheme reads in any case. None for anything that is not a
/// magnet with a hash.
pub fn hash(link: &str) -> Option<String> {
    let link = link.trim();
    let query = link
        .get(..8)
        .filter(|head| head.eq_ignore_ascii_case("magnet:?"))
        .map(|_| &link[8..])?;
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
        match h.len() {
            40 if is_hash(h) => return Some(h.to_string()),
            32 => return base32_hex(h),
            _ => {}
        }
    }
    None
}

/// `link` trimmed, with a magnet scheme written in any case (`MAGNET:?`, as a browser may
/// hand it over) as `magnet:?`, the way torrent clients take it. Anything else comes back
/// trimmed and otherwise as it was.
pub fn normalize(link: &str) -> String {
    let link = link.trim();
    match link.get(..8) {
        Some(head) if head.eq_ignore_ascii_case("magnet:?") => format!("magnet:?{}", &link[8..]),
        _ => link.to_string(),
    }
}

/// The name a magnet link carries in its first `dn`, decoded (`+` reads as a space, as a
/// browser writes it in a query). None for anything that is not a magnet, or a magnet
/// with no name.
pub fn name(link: &str) -> Option<String> {
    let link = link.trim();
    let query = link
        .get(..8)
        .filter(|head| head.eq_ignore_ascii_case("magnet:?"))
        .map(|_| &link[8..])?;
    query.split('&').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        if !key.eq_ignore_ascii_case("dn") {
            return None;
        }
        let name = crate::percent::decode(value);
        let name = name.trim();
        (!name.is_empty()).then(|| name.to_string())
    })
}

/// The magnet rebuilt from what another peer needs and nothing else: its
/// hash, its name, and the trackers whose address carries nothing personal
/// (`is_public_tracker`). Every other parameter goes, a web seed or an exact
/// source among them, since each is an address a client would fetch. None
/// for anything that is not a magnet with a hash.
pub fn public_only(link: &str) -> Option<String> {
    let hash = hash(link)?;
    let query = &link.trim()[8..];
    let mut out = format!("magnet:?xt=urn:btih:{hash}");
    let mut named = false;
    for part in query.split('&') {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        if key.eq_ignore_ascii_case("dn") && !named {
            let name = crate::percent::decode(value);
            if !name.trim().is_empty() {
                named = true;
                out.push_str("&dn=");
                out.push_str(&crate::percent::encode(&name));
            }
        } else if key.eq_ignore_ascii_case("tr") {
            let tracker = crate::percent::decode(value);
            if is_public_tracker(&tracker) {
                out.push_str("&tr=");
                out.push_str(&crate::percent::encode(&tracker));
            }
        }
    }
    Some(out)
}

/// Whether a tracker's announce address is one anyone may use: a plain
/// `udp`, `http(s)` or `ws(s)` address of a host whose path is the bare
/// announce. A private tracker puts the member's passkey in the path, the
/// query or the user part, so anything else is left out.
pub fn is_public_tracker(url: &str) -> bool {
    let url = url.trim();
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if !["udp", "http", "https", "ws", "wss"]
        .iter()
        .any(|s| scheme.eq_ignore_ascii_case(s))
    {
        return false;
    }
    let cut = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (host, tail) = rest.split_at(cut);
    if host.is_empty() || host.contains('@') || tail.contains(['?', '#']) {
        return false;
    }
    ["", "/", "/announce", "/announce.php"]
        .iter()
        .any(|p| tail.eq_ignore_ascii_case(p))
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
        // The scheme in any case, as a feed or a search may write it.
        assert_eq!(
            hash(&format!("Magnet:?xt=urn:btih:{hex}")).as_deref(),
            Some(hex)
        );
    }

    #[test]
    fn a_magnet_scheme_in_any_case_is_written_lowercase() {
        assert_eq!(
            normalize(" MAGNET:?xt=urn:btih:ABC&dn=Example "),
            "magnet:?xt=urn:btih:ABC&dn=Example"
        );
        assert_eq!(normalize("Magnet:?dn=x"), "magnet:?dn=x");
        assert_eq!(normalize("magnet:?dn=x"), "magnet:?dn=x");
        assert_eq!(
            normalize("HTTPS://example.com/a.torrent"),
            "HTTPS://example.com/a.torrent"
        );
        assert_eq!(normalize("magnet:"), "magnet:");
    }

    #[test]
    fn a_magnet_gives_its_name_decoded() {
        let hex = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            name(&format!(
                "magnet:?xt=urn:btih:{hex}&dn=Example%20Show%20-%2003%20%5B1080p%5D&tr=x"
            ))
            .as_deref(),
            Some("Example Show - 03 [1080p]")
        );
        assert_eq!(
            name("MAGNET:?DN=Example+Film+2001&xt=urn:btih:x").as_deref(),
            Some("Example Film 2001")
        );
        assert_eq!(name(&format!("magnet:?xt=urn:btih:{hex}")), None);
        assert_eq!(name(&format!("magnet:?xt=urn:btih:{hex}&dn=")), None);
        assert_eq!(name("https://example.com/?dn=x"), None);
    }

    #[test]
    fn a_public_magnet_keeps_the_hash_the_name_and_open_trackers() {
        let hex = "0123456789abcdef0123456789abcdef01234567";
        let link = format!(
            "magnet:?xt=urn:btih:{}&dn=Some%20Show%20-%2005&tr=udp%3A%2F%2Ftracker.example%3A1337\
             &tr=https%3A%2F%2Fprivate.example%2F0a1b2c3d4e5f6a7b8c9d%2Fannounce\
             &tr=http%3A%2F%2Ftracker.example%2Fannounce.php%3Fpasskey%3Dabc\
             &tr=http%3A%2F%2Fme%3Asecret%40tracker.example%2Fannounce\
             &tr=http%3A%2F%2Fopen.example%3A6969%2Fannounce\
             &xs=http%3A%2F%2F198.51.100.7%2Fx.torrent&ws=http%3A%2F%2F198.51.100.7%2F",
            hex.to_uppercase()
        );
        assert_eq!(
            public_only(&link).as_deref(),
            Some(
                "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=Some%20Show%20-%2005\
                 &tr=udp%3A%2F%2Ftracker.example%3A1337&tr=http%3A%2F%2Fopen.example%3A6969%2Fannounce"
            )
        );
        assert_eq!(public_only("http://198.51.100.7/a.torrent"), None);
        assert_eq!(public_only("magnet:?dn=no-hash"), None);
    }

    #[test]
    fn a_tracker_with_anything_personal_is_not_public() {
        assert!(is_public_tracker("udp://tracker.example:1337"));
        assert!(is_public_tracker("udp://tracker.example:1337/announce"));
        assert!(is_public_tracker("wss://tracker.example"));
        assert!(!is_public_tracker(
            "https://tracker.example/0a1b2c3d4e5f/announce"
        ));
        assert!(!is_public_tracker("https://tracker.example/announce?pk=1"));
        assert!(!is_public_tracker("http://u:p@tracker.example/announce"));
        assert!(!is_public_tracker("ftp://tracker.example/announce"));
        assert!(!is_public_tracker("tracker.example/announce"));
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
