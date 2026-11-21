//! A body sent gzip'd because its request asked for it with `Accept-Encoding: gzip`. The
//! shared client is built without reqwest's decompression, so every other request goes out
//! and comes back as it always has; a caller that asks for a compressed body for itself, the
//! update checks' release list (#809), opens it here.

use std::io::Read;

use crate::HttpResponse;

/// The header a request sets to ask for a gzip'd body.
pub const ACCEPT: (&str, &str) = ("Accept-Encoding", "gzip");

/// `res`'s body, unpacked when the server says it sent it gzip'd, else as it came.
pub fn body(res: HttpResponse) -> std::io::Result<Vec<u8>> {
    match res.header("content-encoding") {
        Some(encoding) if encoding.trim().eq_ignore_ascii_case("gzip") => {
            let mut out = Vec::new();
            flate2::read::GzDecoder::new(res.body.as_slice()).read_to_end(&mut out)?;
            Ok(out)
        }
        _ => Ok(res.body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    fn reply(headers: Vec<(String, String)>, body: Vec<u8>) -> HttpResponse {
        HttpResponse {
            status: 200,
            headers,
            body,
        }
    }

    #[test]
    fn a_gzipped_body_is_unpacked_and_a_plain_one_is_left() {
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(b"[1, 2, 3]").unwrap();
        let packed = gz.finish().unwrap();
        let sent = reply(vec![("Content-Encoding".into(), "gzip".into())], packed);
        assert_eq!(body(sent).unwrap(), b"[1, 2, 3]");
        assert_eq!(body(reply(vec![], b"[4]".to_vec())).unwrap(), b"[4]");
    }

    #[test]
    fn a_body_that_says_gzip_but_is_not_is_an_error() {
        let sent = reply(
            vec![("content-encoding".into(), "gzip".into())],
            b"not gzip".to_vec(),
        );
        assert!(body(sent).is_err());
    }
}
