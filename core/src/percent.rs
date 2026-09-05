//! The form-encoded reading of a query string value, by hand.
//!
//! Two callers want it, the OAuth callback's query and a nyaa feed url's
//! search term, and neither wants a dependency for thirty lines.

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
    fn a_plus_is_a_space_and_a_pair_of_hex_digits_is_a_byte() {
        assert_eq!(decode("one+two"), "one two");
        assert_eq!(decode("%5Bsubs%5D+show"), "[subs] show");
        assert_eq!(decode("plain"), "plain");
    }
}
