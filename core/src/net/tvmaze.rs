//! TVmaze, the second match provider: the one that knows the series AniList
//! has no entry for at all, western animation and live-action television.
//! Free and keyless, so it needs no per-user setup the way TMDB would.
//!
//! Three requests and no more. A search answers with whole shows in
//! relevance order, which is all the auto-match scores. A fetch is one call
//! with the episodes and the images embedded, so a match costs two
//! requests against a 20-per-10-seconds limit. The IMDb lookup is how a
//! pasted IMDb link becomes a show id.

use serde::Deserialize;

use super::limiter::ProviderClient;
use super::{HttpRequest, HttpResponse, Method, null_to_default};
use crate::contract::{AiringStatus, CoreError, Provider};
use crate::percent;

pub const TVMAZE_API: &str = "https://api.tvmaze.com";

/// A network or a streaming service. Only the name is ever drawn.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Channel {
    pub name: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Rating {
    /// Already 0 to 10, the scale the cards draw.
    pub average: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Externals {
    pub imdb: Option<String>,
    pub thetvdb: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ImageSet {
    pub medium: Option<String>,
    pub original: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Resolution {
    pub url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Resolutions {
    pub original: Option<Resolution>,
}

/// One entry of the embedded image list: a poster, a background, a banner
/// and so on, one of each kind flagged as the main one.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct RawImage {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    #[serde(deserialize_with = "null_to_default")]
    pub main: bool,
    #[serde(deserialize_with = "null_to_default")]
    pub resolutions: Resolutions,
}

/// One entry of the embedded episode list. `number` is null for a special,
/// which has no place in a running order a file could be numbered by.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct RawEpisode {
    pub name: Option<String>,
    pub season: Option<i64>,
    pub number: Option<i64>,
    pub airstamp: Option<String>,
    pub runtime: Option<u32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Embedded {
    #[serde(deserialize_with = "null_to_default")]
    pub episodes: Vec<RawEpisode>,
    #[serde(deserialize_with = "null_to_default")]
    pub images: Vec<RawImage>,
}

/// A show as TVmaze answers for it, from a search or a fetch. Only the
/// fetch carries `_embedded`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Show {
    pub id: u64,
    pub name: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub language: Option<String>,
    #[serde(deserialize_with = "null_to_default")]
    pub genres: Vec<String>,
    pub status: Option<String>,
    pub premiered: Option<String>,
    pub ended: Option<String>,
    #[serde(rename = "averageRuntime")]
    pub average_runtime: Option<u32>,
    #[serde(deserialize_with = "null_to_default")]
    pub rating: Rating,
    pub network: Option<Channel>,
    #[serde(rename = "webChannel")]
    pub web_channel: Option<Channel>,
    #[serde(deserialize_with = "null_to_default")]
    pub externals: Externals,
    pub image: Option<ImageSet>,
    pub summary: Option<String>,
    #[serde(rename = "_embedded")]
    pub embedded: Option<Embedded>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct SearchHit {
    show: Show,
}

/// One numbered episode, the shape `tvmaze_episodes` holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TvmazeEpisode {
    pub season: u32,
    pub number: u32,
    pub name: Option<String>,
    pub airstamp: Option<i64>,
    pub runtime: Option<u32>,
}

impl Show {
    /// The network, or the streaming service for a show that never had one.
    pub fn network_name(&self) -> Option<String> {
        [&self.network, &self.web_channel]
            .into_iter()
            .flatten()
            .find_map(|c| c.name.clone())
    }

    /// The premiere's year, the one a search row and a page print.
    pub fn year(&self) -> Option<u32> {
        year_of(self.premiered.as_deref())
    }

    /// The poster flagged main, else the show's own image. The embedded
    /// list is only there on a fetch; a search hit has the image alone.
    pub fn poster_url(&self) -> Option<String> {
        self.main_image("poster", false)
            .or_else(|| self.image.as_ref().and_then(|i| i.original.clone()))
    }

    /// The background flagged main, else the first background there is.
    pub fn background_url(&self) -> Option<String> {
        self.main_image("background", true)
    }

    /// The cover a search row draws: the small one, since a picker of ten
    /// results has no use for ten originals.
    pub fn thumbnail_url(&self) -> Option<String> {
        self.image
            .as_ref()
            .and_then(|i| i.medium.clone().or_else(|| i.original.clone()))
    }

    fn main_image(&self, kind: &str, any_will_do: bool) -> Option<String> {
        let images = &self.embedded.as_ref()?.images;
        let of_kind = || {
            images
                .iter()
                .filter(move |i| i.kind.as_deref() == Some(kind))
                .filter_map(|i| Some((i.main, i.resolutions.original.as_ref()?.url.clone()?)))
        };
        of_kind()
            .find(|(main, _)| *main)
            .or_else(|| if any_will_do { of_kind().next() } else { None })
            .map(|(_, url)| url)
    }

    /// The numbered episodes, in season and episode order. A special
    /// carries no number and is left out: no file is numbered after it.
    pub fn episodes(&self) -> Vec<TvmazeEpisode> {
        let mut out: Vec<TvmazeEpisode> = self
            .embedded
            .iter()
            .flat_map(|e| e.episodes.iter())
            .filter_map(|e| {
                let season = u32::try_from(e.season?).ok()?;
                let number = u32::try_from(e.number?).ok().filter(|n| *n > 0)?;
                Some(TvmazeEpisode {
                    season,
                    number,
                    name: e.name.clone().filter(|n| !n.trim().is_empty()),
                    airstamp: e.airstamp.as_deref().and_then(parse_airstamp),
                    runtime: e.runtime,
                })
            })
            .collect();
        out.sort_by_key(|e| (e.season, e.number));
        // The table's key is the pair, so a repeat keeps the first.
        out.dedup_by_key(|e| (e.season, e.number));
        out
    }
}

/// The first four digits of a `YYYY-MM-DD`, when they are a year.
pub fn year_of(date: Option<&str>) -> Option<u32> {
    date.and_then(|d| d.get(..4))
        .and_then(|y| y.parse().ok())
        .filter(|y| *y > 0)
}

/// An RFC 3339 instant as Unix seconds, `None` for anything else.
fn parse_airstamp(s: &str) -> Option<i64> {
    time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
        .ok()
        .map(|t| t.unix_timestamp())
}

/// TVmaze's status in the card's vocabulary. Running is releasing, Ended is
/// finished, and the two states before a premiere are not yet released.
pub fn airing_status(status: &str) -> Option<AiringStatus> {
    match status.trim() {
        "Running" => Some(AiringStatus::Releasing),
        "Ended" => Some(AiringStatus::Finished),
        "To Be Determined" | "In Development" => Some(AiringStatus::NotYetReleased),
        _ => None,
    }
}

/// The summary as plain text: tags gone, entities decoded, and one
/// paragraph per `<p>` or `<br>`.
///
/// Only a `<` followed by a letter, a `/` or a `!` opens a tag, so "a < b"
/// and "1 <2" are text. Entities are decoded in the same single pass, so
/// an escaped one (`&amp;lt;`) is decoded once and never twice, and one
/// this cannot name is left as written.
pub fn plain_summary(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(at) = rest.find(['<', '&']) {
        text.push_str(&rest[..at]);
        rest = &rest[at..];
        if rest.starts_with('<') {
            let opens_tag = rest[1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!');
            match rest.find('>').filter(|_| opens_tag) {
                Some(close) => {
                    let name = rest[1..close]
                        .trim_start_matches('/')
                        .split(|c: char| c.is_whitespace() || c == '/')
                        .next()
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    if name == "p" || name == "br" {
                        text.push('\n');
                    }
                    rest = &rest[close + 1..];
                }
                None => {
                    text.push('<');
                    rest = &rest[1..];
                }
            }
        } else {
            match decode_entity(rest) {
                Some((c, len)) => {
                    text.push(c);
                    rest = &rest[len..];
                }
                None => {
                    text.push('&');
                    rest = &rest[1..];
                }
            }
        }
    }
    text.push_str(rest);
    text.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The entity `s` starts with, as its character and the bytes it spans:
/// `&#NNN;`, `&#xHH;`, or one of the names TVmaze's summaries use.
fn decode_entity(s: &str) -> Option<(char, usize)> {
    // The longest name below is eight letters; a numeric one is at most
    // `#x10FFFF`.
    let end = s
        .get(1..)?
        .char_indices()
        .take(10)
        .find(|(_, c)| *c == ';')?
        .0
        + 1;
    let body = &s[1..end];
    let c = if let Some(num) = body.strip_prefix('#') {
        let value = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse::<u32>().ok()?,
        };
        char::from_u32(value).filter(|c| *c != '\0')?
    } else {
        named_entity(body)?
    };
    Some((c, end + 1))
}

fn named_entity(name: &str) -> Option<char> {
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => ' ',
        "hellip" => '\u{2026}',
        "mdash" => '\u{2014}',
        "ndash" => '\u{2013}',
        "lsquo" => '\u{2018}',
        "rsquo" => '\u{2019}',
        "sbquo" => '\u{201a}',
        "ldquo" => '\u{201c}',
        "rdquo" => '\u{201d}',
        "bdquo" => '\u{201e}',
        "laquo" => '\u{ab}',
        "raquo" => '\u{bb}',
        "bull" => '\u{2022}',
        "middot" => '\u{b7}',
        "deg" => '\u{b0}',
        "copy" => '\u{a9}',
        "reg" => '\u{ae}',
        "trade" => '\u{2122}',
        "times" => '\u{d7}',
        "eacute" => 'é',
        "Eacute" => 'É',
        "egrave" => 'è',
        "Egrave" => 'È',
        "ecirc" => 'ê',
        "euml" => 'ë',
        "aacute" => 'á',
        "Aacute" => 'Á',
        "agrave" => 'à',
        "acirc" => 'â',
        "auml" => 'ä',
        "Auml" => 'Ä',
        "aring" => 'å',
        "atilde" => 'ã',
        "iacute" => 'í',
        "igrave" => 'ì',
        "icirc" => 'î',
        "iuml" => 'ï',
        "oacute" => 'ó',
        "Oacute" => 'Ó',
        "ograve" => 'ò',
        "ocirc" => 'ô',
        "ouml" => 'ö',
        "Ouml" => 'Ö',
        "otilde" => 'õ',
        "oslash" => 'ø',
        "uacute" => 'ú',
        "ugrave" => 'ù',
        "ucirc" => 'û',
        "uuml" => 'ü',
        "Uuml" => 'Ü',
        "ntilde" => 'ñ',
        "Ntilde" => 'Ñ',
        "ccedil" => 'ç',
        "Ccedil" => 'Ç',
        "szlig" => 'ß',
        "aelig" => 'æ',
        _ => return None,
    })
}

pub struct TvmazeClient {
    client: ProviderClient,
}

impl TvmazeClient {
    pub fn new(client: ProviderClient) -> TvmazeClient {
        TvmazeClient { client }
    }

    async fn get(&self, url: String) -> Result<HttpResponse, CoreError> {
        self.client
            .send(HttpRequest {
                method: Method::Get,
                url,
                headers: vec![("Accept".to_string(), "application/json".to_string())],
                body: None,
                timeout: None,
            })
            .await
    }

    /// Every show TVmaze finds for the query, in its own relevance order.
    pub async fn search(&self, query: &str) -> Result<Vec<Show>, CoreError> {
        let response = self
            .get(format!(
                "{TVMAZE_API}/search/shows?q={}",
                percent::encode(query)
            ))
            .await?;
        check(&response)?;
        let hits: Option<Vec<SearchHit>> = response.json()?;
        Ok(hits
            .unwrap_or_default()
            .into_iter()
            .map(|h| h.show)
            .collect())
    }

    /// One show with its episodes and its images. A 404 is a show TVmaze
    /// does not have, which is an answer rather than a failure.
    pub async fn show(&self, id: u64) -> Result<Option<Show>, CoreError> {
        let response = self
            .get(format!(
                "{TVMAZE_API}/shows/{id}?embed[]=episodes&embed[]=images"
            ))
            .await?;
        if response.status == 404 {
            return Ok(None);
        }
        check(&response)?;
        Ok(Some(response.json()?))
    }

    /// The show behind an IMDb title id. TVmaze answers with a redirect to
    /// the show, which the transport follows, so the reply is the show
    /// itself; a 404 is a title it has no show for.
    pub async fn lookup_imdb(&self, imdb: &str) -> Result<Option<u64>, CoreError> {
        let response = self
            .get(format!(
                "{TVMAZE_API}/lookup/shows?imdb={}",
                percent::encode(imdb)
            ))
            .await?;
        if response.status == 404 {
            return Ok(None);
        }
        check(&response)?;
        let show: Option<Show> = response.json()?;
        Ok(show.map(|s| s.id).filter(|id| *id > 0))
    }
}

/// Anything but a success is TVmaze's own answer, carried with its status
/// so the jobs can tell it from a transport failure.
fn check(response: &HttpResponse) -> Result<(), CoreError> {
    if response.is_success() {
        return Ok(());
    }
    Err(CoreError::Provider {
        provider: Provider::Tvmaze,
        status: Some(u32::from(response.status)),
        message: format!("TVmaze {}: {}", response.status, response.text()),
        retry_after: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::{FakeHttp, Upstream};
    use std::sync::Arc;
    use std::time::Duration;

    const SHOW: &str = include_str!("../../tests/fixtures/tvmaze/show-83073.json");
    const SEARCH: &str = include_str!("../../tests/fixtures/tvmaze/search-avatar.json");

    fn client(http: Arc<FakeHttp>) -> TvmazeClient {
        TvmazeClient::new(ProviderClient::new(
            Upstream::Tvmaze,
            http,
            Duration::from_millis(1),
        ))
    }

    /// The real reply for Avatar: Seven Havens, captured with curl. The
    /// main poster wins over the show's image, and with no background
    /// flagged main the first one stands in.
    #[tokio::test]
    async fn a_fetch_reads_the_show_its_episodes_and_its_main_images() {
        let http = FakeHttp::new();
        http.push(200, SHOW);
        let show = client(http.clone()).show(83073).await.unwrap().unwrap();
        assert_eq!(
            http.requests()[0].url,
            "https://api.tvmaze.com/shows/83073?embed[]=episodes&embed[]=images"
        );
        assert_eq!(show.id, 83073);
        assert_eq!(show.name.as_deref(), Some("Avatar: Seven Havens"));
        assert_eq!(show.status.as_deref(), Some("Running"));
        assert_eq!(show.network_name().as_deref(), Some("Paramount+"));
        assert_eq!(show.year(), Some(2026));
        assert_eq!(show.externals.imdb.as_deref(), Some("tt35854736"));
        assert_eq!(show.externals.thetvdb, Some(460458));
        assert_eq!(
            show.poster_url().as_deref(),
            Some("https://static.tvmaze.com/uploads/images/original_untouched/645/1613483.jpg")
        );
        assert_eq!(
            show.background_url().as_deref(),
            Some("https://static.tvmaze.com/uploads/images/original_untouched/579/1449291.jpg")
        );
        let episodes = show.episodes();
        assert_eq!(episodes.len(), 13);
        assert_eq!(
            episodes[0],
            TvmazeEpisode {
                season: 1,
                number: 1,
                name: Some("On the Horizon".into()),
                airstamp: Some(1_791_547_200),
                runtime: Some(22),
            }
        );
        assert_eq!(episodes[2].name.as_deref(), Some("The Order"));
    }

    #[tokio::test]
    async fn a_search_keeps_tvmazes_order_and_a_404_fetch_is_no_show() {
        let http = FakeHttp::new();
        http.push(200, SEARCH);
        http.push(404, "null");
        let c = client(http.clone());
        let shows = c.search("avatar seven").await.unwrap();
        assert_eq!(
            http.requests()[0].url,
            "https://api.tvmaze.com/search/shows?q=avatar%20seven"
        );
        assert_eq!(shows.len(), 10);
        assert_eq!(shows[0].id, 83073);
        assert_eq!(shows[1].name.as_deref(), Some("The King's Avatar"));
        // A search hit has no embedded list, so its poster is the image.
        assert!(
            shows[0]
                .poster_url()
                .unwrap()
                .contains("original_untouched")
        );
        assert!(
            shows[0]
                .thumbnail_url()
                .unwrap()
                .contains("medium_portrait")
        );
        // Avatar Kuruk has no image at all.
        assert!(
            shows
                .iter()
                .find(|s| s.id == 94958)
                .unwrap()
                .poster_url()
                .is_none()
        );
        assert!(c.show(1).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn an_imdb_lookup_follows_to_the_show_and_a_miss_is_none() {
        let http = FakeHttp::new();
        http.push_for("lookup/shows?imdb=tt35854736", 200, SHOW);
        http.push_for("lookup/shows?imdb=tt0000001", 404, "");
        let c = client(http.clone());
        assert_eq!(c.lookup_imdb("tt35854736").await.unwrap(), Some(83073));
        assert_eq!(c.lookup_imdb("tt0000001").await.unwrap(), None);
    }

    #[tokio::test]
    async fn an_error_status_names_tvmaze_and_a_null_list_is_empty() {
        let http = FakeHttp::new();
        http.push(500, "boom");
        http.push(200, "null");
        let c = client(http);
        let err = c.search("x").await.err().unwrap();
        assert!(
            matches!(
                err,
                CoreError::Provider {
                    provider: Provider::Tvmaze,
                    status: Some(500),
                    ..
                }
            ),
            "{err:?}"
        );
        assert!(c.search("x").await.unwrap().is_empty());
    }

    #[test]
    fn the_statuses_map_onto_the_cards_vocabulary() {
        assert_eq!(airing_status("Running"), Some(AiringStatus::Releasing));
        assert_eq!(airing_status("Ended"), Some(AiringStatus::Finished));
        assert_eq!(
            airing_status("To Be Determined"),
            Some(AiringStatus::NotYetReleased)
        );
        assert_eq!(
            airing_status("In Development"),
            Some(AiringStatus::NotYetReleased)
        );
        assert_eq!(airing_status("Something Else"), None);
    }

    #[test]
    fn a_summary_loses_its_tags_and_keeps_its_paragraphs() {
        assert_eq!(
            plain_summary("<p>One <b>bold</b> &amp; plain.</p><p>Two<br/>Three</p>"),
            "One bold & plain.\n\nTwo\n\nThree"
        );
        assert_eq!(plain_summary(""), "");
        assert_eq!(plain_summary("a < b"), "a < b");
        assert_eq!(
            plain_summary("Tom&#39;s &quot;show&quot;"),
            "Tom's \"show\""
        );
    }
}
