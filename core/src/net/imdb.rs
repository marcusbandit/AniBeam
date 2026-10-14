//! IMDb's ratings, from the dump IMDb publishes every day for personal and
//! non-commercial use: every rated title with its average and its vote
//! count, one gzipped TSV of about 9 MB and 1.7 million lines.
//!
//! The dump is a static file on a CDN, the way a poster is, and it is asked
//! for once a week at most, so it goes straight through `Http` like the
//! image cache's fetches rather than through a `ProviderClient`. There is
//! nothing to pace, and an `Upstream` reports its failures as one of the
//! contract's providers, which IMDb is not; a failure here is a `Fault`.

use std::io::{BufRead, BufReader, Read};
use std::time::Duration;

use flate2::read::GzDecoder;

use super::{Fault, Http, HttpRequest, Method};

pub const RATINGS_URL: &str = "https://datasets.imdbws.com/title.ratings.tsv.gz";

/// The dump's ceiling, in place of the client's 30 seconds: that covers
/// the whole body, and 9 MB over a slow line takes longer than a poster.
pub const DUMP_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// The dump's first line, verbatim. Any other first line is some other
/// file, and nothing after it is read.
const HEADER: &[u8] = b"tconst\taverageRating\tnumVotes";

/// What a fetch of the dump came back with.
#[derive(Debug)]
pub enum Dump {
    /// A 304: the dump stored last time is still IMDb's latest.
    Unchanged,
    /// A 200: the gzipped dump, and the `Last-Modified` to ask with next
    /// time.
    Fresh {
        gz: Vec<u8>,
        last_modified: Option<String>,
    },
}

/// The dump, unless it has not changed since `since`, which is the
/// `Last-Modified` the last download came with.
pub async fn fetch(http: &dyn Http, since: Option<&str>) -> Result<Dump, Fault> {
    let headers = since
        .map(|at| vec![("If-Modified-Since".to_string(), at.to_string())])
        .unwrap_or_default();
    let response = http
        .send(HttpRequest {
            method: Method::Get,
            url: RATINGS_URL.to_string(),
            headers,
            body: None,
            timeout: Some(DUMP_TIMEOUT),
        })
        .await?;
    match response.status {
        304 => Ok(Dump::Unchanged),
        200 => {
            let last_modified = response.header("last-modified").map(str::to_string);
            Ok(Dump::Fresh {
                gz: response.body,
                last_modified,
            })
        }
        status => Err(Fault::Refused(format!(
            "IMDb {status}: {}",
            response.text()
        ))),
    }
}

/// One title's rating: the number after "tt", the average in tenths, and
/// how many votes it stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rating {
    pub id: u64,
    pub tenths: u32,
    pub votes: u64,
}

/// The dump read a line at a time straight out of the gzip, so the table
/// is filled as it decompresses and 1.7 million titles are never held in
/// memory together. A line that does not read as a rating is skipped. A
/// stream that stops decompressing, a download cut short, yields one
/// `Refused` and then ends, and the caller keeps what it had.
pub struct Ratings<R: Read> {
    lines: BufReader<GzDecoder<R>>,
    line: Vec<u8>,
    done: bool,
}

impl<R: Read> Ratings<R> {
    /// Opens the dump and checks its header before a single row is read.
    pub fn new(gz: R) -> Result<Ratings<R>, Fault> {
        let mut ratings = Ratings {
            lines: BufReader::new(GzDecoder::new(gz)),
            line: Vec::new(),
            done: false,
        };
        match ratings.next_line()? {
            Some(header) if header == HEADER => Ok(ratings),
            Some(_) => Err(Fault::Refused(
                "the IMDb dump's header is not tconst, averageRating, numVotes".to_string(),
            )),
            None => Err(Fault::Refused("the IMDb dump is empty".to_string())),
        }
    }

    /// The next line without its line ending, `None` at the end.
    fn next_line(&mut self) -> Result<Option<&[u8]>, Fault> {
        self.line.clear();
        match self.lines.read_until(b'\n', &mut self.line) {
            Ok(0) => Ok(None),
            Ok(_) => {
                let line = self.line.as_slice();
                let line = line.strip_suffix(b"\n").unwrap_or(line);
                Ok(Some(line.strip_suffix(b"\r").unwrap_or(line)))
            }
            Err(e) => Err(Fault::Refused(format!(
                "the IMDb dump did not decompress: {e}"
            ))),
        }
    }
}

impl<R: Read> Iterator for Ratings<R> {
    type Item = Result<Rating, Fault>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.done {
            match self.next_line() {
                Ok(Some(line)) => {
                    if let Some(rating) = parse_line(line) {
                        return Some(Ok(rating));
                    }
                }
                Ok(None) => self.done = true,
                Err(fault) => {
                    self.done = true;
                    return Some(Err(fault));
                }
            }
        }
        None
    }
}

/// `tt0213338\t8.9\t174384`, and nothing else: three fields, every one of
/// them read whole.
fn parse_line(line: &[u8]) -> Option<Rating> {
    let mut fields = line.split(|b| *b == b'\t');
    let id = title_id(fields.next()?)?;
    let tenths = tenths(fields.next()?)?;
    let votes = digits(fields.next()?)?;
    if fields.next().is_some() {
        return None;
    }
    Some(Rating { id, tenths, votes })
}

/// `tt0213338` as 213338. The "tt" is what makes it a title rather than a
/// person (`nm`) or a company (`co`).
pub fn title_id(field: &[u8]) -> Option<u64> {
    digits(field.strip_prefix(b"tt")?).filter(|id| *id > 0)
}

/// "8.9" as 89 and "10.0" as 100. Split on the point rather than read as a
/// float, so no rating is ever a rounding away from what IMDb wrote. IMDb
/// writes one digit after the point; anything else is not its format.
fn tenths(field: &[u8]) -> Option<u32> {
    let (whole, tenth) = match field.iter().position(|b| *b == b'.') {
        Some(at) => (&field[..at], &field[at + 1..]),
        None => (field, &b"0"[..]),
    };
    if tenth.len() != 1 {
        return None;
    }
    let value = digits(whole)?
        .checked_mul(10)?
        .checked_add(digits(tenth)?)?;
    u32::try_from(value).ok().filter(|v| *v <= 100)
}

/// ASCII digits as a number. Nothing at all, a sign, a space or a number
/// too big for a `u64` is not one.
fn digits(field: &[u8]) -> Option<u64> {
    if field.is_empty() {
        return None;
    }
    field.iter().try_fold(0u64, |n, b| {
        b.is_ascii_digit()
            .then(|| n.checked_mul(10)?.checked_add(u64::from(b - b'0')))
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    fn gz(text: &str) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(text.as_bytes()).unwrap();
        encoder.finish().unwrap()
    }

    fn read_all(gz: &[u8]) -> Result<Vec<Rating>, Fault> {
        Ratings::new(gz)?.collect()
    }

    fn rating(id: u64, tenths: u32, votes: u64) -> Rating {
        Rating { id, tenths, votes }
    }

    #[test]
    fn rows_read_into_ids_tenths_and_votes() {
        let dump = gz("tconst\taverageRating\tnumVotes\n\
                       tt0213338\t8.9\t174384\n\
                       tt0000001\t5.7\t2234\n\
                       tt10000000\t10.0\t5\n\
                       tt0000002\t1.0\t0\n");
        assert_eq!(
            read_all(&dump).unwrap(),
            vec![
                rating(213338, 89, 174384),
                rating(1, 57, 2234),
                rating(10_000_000, 100, 5),
                rating(2, 10, 0),
            ]
        );
    }

    /// The tenths come from the digits, never from a float: 0.1 has no
    /// exact float, and a parse that went through one could land 8.3 on
    /// 82 after a truncation.
    #[test]
    fn a_rating_is_split_on_the_point_rather_than_read_as_a_float() {
        assert_eq!(tenths(b"8.3"), Some(83));
        assert_eq!(tenths(b"0.1"), Some(1));
        assert_eq!(tenths(b"7"), Some(70));
        assert_eq!(tenths(b"10.0"), Some(100));
        assert_eq!(tenths(b"10.1"), None, "past the top of the scale");
        assert_eq!(tenths(b"8.95"), None, "IMDb writes one digit");
        assert_eq!(tenths(b"8."), None);
        assert_eq!(tenths(b".9"), None);
        assert_eq!(tenths(b"-8.9"), None);
        assert_eq!(tenths(b"8,9"), None);
    }

    #[test]
    fn malformed_lines_are_skipped_and_the_rest_still_read() {
        let dump = gz("tconst\taverageRating\tnumVotes\r\n\
                       tt0000001\t5.7\t2234\r\n\
                       nm0000001\t5.7\t10\n\
                       tt\t5.7\t10\n\
                       tt0000003\t\\N\t10\n\
                       tt0000004\t6.1\n\
                       tt0000005\t6.1\t10\textra\n\
                       tt0000006\t6.1\t-4\n\
                       \n\
                       tt0000007\t6.2\t99");
        assert_eq!(
            read_all(&dump).unwrap(),
            vec![rating(1, 57, 2234), rating(7, 62, 99)]
        );
    }

    #[test]
    fn a_wrong_header_or_an_empty_dump_is_refused_before_any_row() {
        let wrong = gz("tconst\tnumVotes\taverageRating\ntt0000001\t5.7\t2234\n");
        assert!(matches!(
            Ratings::new(wrong.as_slice()),
            Err(Fault::Refused(_))
        ));
        let empty = gz("");
        assert!(matches!(
            Ratings::new(empty.as_slice()),
            Err(Fault::Refused(_))
        ));
        // Not gzip at all: an HTML error page served with a 200.
        assert!(matches!(
            Ratings::new(&b"<html>nope</html>"[..]),
            Err(Fault::Refused(_))
        ));
    }

    /// A download cut short ends in a `Refused` rather than passing for a
    /// shorter dump: the gzip trailer carries the length and the checksum,
    /// and a stream without it does not decompress cleanly.
    #[test]
    fn a_truncated_dump_ends_in_a_refusal() {
        let mut text = String::from("tconst\taverageRating\tnumVotes\n");
        for id in 1..=5_000u64 {
            text.push_str(&format!(
                "tt{id:07}\t{}.{}\t{}\n",
                id % 9 + 1,
                id % 10,
                id * 7
            ));
        }
        let mut dump = gz(&text);
        dump.truncate(dump.len() * 2 / 3);
        let rows: Vec<Result<Rating, Fault>> = Ratings::new(dump.as_slice()).unwrap().collect();
        assert!(rows.len() < 5_000, "{} rows", rows.len());
        assert!(
            matches!(rows.last(), Some(Err(Fault::Refused(_)))),
            "{:?}",
            rows.last()
        );
        assert_eq!(rows.iter().filter(|r| r.is_err()).count(), 1);
    }

    #[tokio::test]
    async fn the_fetch_is_conditional_and_carries_the_long_timeout() {
        let http = FakeHttp::new();
        http.push_with_headers(
            200,
            gz("tconst\taverageRating\tnumVotes\n"),
            vec![(
                "Last-Modified".into(),
                "Wed, 14 Oct 2026 00:44:30 GMT".into(),
            )],
        );
        http.push(304, "");
        http.push(503, "Slow Down");

        match fetch(http.as_ref(), None).await.unwrap() {
            Dump::Fresh { last_modified, gz } => {
                assert_eq!(
                    last_modified.as_deref(),
                    Some("Wed, 14 Oct 2026 00:44:30 GMT")
                );
                assert!(!gz.is_empty());
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            fetch(http.as_ref(), Some("Wed, 14 Oct 2026 00:44:30 GMT"))
                .await
                .unwrap(),
            Dump::Unchanged
        ));
        assert!(matches!(
            fetch(http.as_ref(), None).await,
            Err(Fault::Refused(_))
        ));
        http.fail_next("connection refused");
        assert_eq!(
            fetch(http.as_ref(), None).await.unwrap_err(),
            Fault::Unreachable("connection refused".into())
        );

        let requests = http.requests();
        assert_eq!(requests[0].url, RATINGS_URL);
        assert_eq!(requests[0].timeout, Some(DUMP_TIMEOUT));
        assert!(requests[0].headers.is_empty());
        assert_eq!(
            requests[1].headers,
            vec![(
                "If-Modified-Since".to_string(),
                "Wed, 14 Oct 2026 00:44:30 GMT".to_string()
            )]
        );
    }
}
