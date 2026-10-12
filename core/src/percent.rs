//! The form-encoded reading of a query string value, by hand, and the
//! writing of one.
//!
//! Two callers want the reading, the OAuth callback's query and a nyaa
//! feed url's search term, and one the writing, TVmaze's search term;
//! none of them wants a dependency for thirty lines.

/// `+` is a space, `%XX` is a byte, and anything that is not a complete
/// escape stands for itself rather than failing. Bytes that do not make
/// UTF-8 become the replacement character, so a mangled query is still a
/// string rather than a panic.
pub(crate) fn decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => match (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                (Some(high), Some(low)) => {
                    out.push((high << 4) | low);
                    i += 3;
                }
                _ => {
                    out.push(b'%');
                    i += 1;
                }
            },
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A query string value, percent-encoded: the unreserved characters stand
/// for themselves and every other byte of the UTF-8 is `%XX`. A space is
/// `%20` rather than `+`, which every server reads the same way.
pub(crate) fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_incomplete_escape_stands_for_itself() {
        assert_eq!(decode("100%"), "100%");
        assert_eq!(decode("%zz"), "%zz");
        assert_eq!(decode("%2"), "%2");
        assert_eq!(decode("%41%42"), "AB");
    }

    #[test]
    fn encoding_escapes_all_but_the_unreserved_and_decodes_back() {
        assert_eq!(encode("Avatar: Seven Havens"), "Avatar%3A%20Seven%20Havens");
        assert_eq!(encode("a-b_c.d~e"), "a-b_c.d~e");
        assert_eq!(encode("Shōgun & co"), "Sh%C5%8Dgun%20%26%20co");
        assert_eq!(decode(&encode("Shōgun & co+1")), "Shōgun & co+1");
    }

    #[test]
    fn a_plus_is_a_space_and_a_pair_of_hex_digits_is_a_byte() {
        assert_eq!(decode("one+two"), "one two");
        assert_eq!(decode("%5Bsubs%5D+show"), "[subs] show");
        assert_eq!(decode("plain"), "plain");
    }
}
