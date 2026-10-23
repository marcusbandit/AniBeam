//! RSS fetching and parsing: nyaa's stat extensions, erai's metadata and the generic-feed
//! fallbacks. Ported from anirss `nyaa.py`. The core's own feed reader (`downloads::feed`)
//! reads the same feeds for a subscription's page and stays as it is; this one produces
//! anirss's `Item`, field for field.

use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use roxmltree::Node;

use crate::contract::{CoreError, Release};
use crate::downloads::feed::host_of;
use crate::net::{Http, HttpRequest, Method};

/// anirss `fetch_rss` default: a blackholed endpoint must fail fast so the fallback probe
/// can move on instead of feeling like a hang.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

const NYAA_NS: &str = "https://nyaa.si/xmlns/nyaa";
const ERAI_NS: &str = "https://www.erai-raws.info/rss-page/";
const TORZNAB_NS: &str = "http://torznab.com/schemas/2015/feed";

/// erai-raws (Cloudflare) 502s non-browser User-Agents, so present as a stock Firefox.
pub const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0";

/// Newznab top-level category ids (Jackett and Prowlarr torznab feeds).
fn torznab_category(id: &str) -> &'static str {
    match id {
        "1" => "Console",
        "2" => "Movies",
        "3" => "Audio",
        "4" => "PC",
        "5" => "TV",
        "6" => "XXX",
        "7" => "Books",
        "8" => "Other",
        _ => "",
    }
}

/// The first direct child element with this namespace and local name (ElementTree's
/// `find("ns:name")`; a `None` namespace is ElementTree's bare `find("name")`).
fn child<'a, 'i>(node: Node<'a, 'i>, ns: Option<&str>, name: &str) -> Option<Node<'a, 'i>> {
    node.children()
        .find(|c| c.is_element() && c.tag_name().name() == name && c.tag_name().namespace() == ns)
}

/// An element's own text before its first child element, `None` when there is none.
fn text<'a>(elem: Option<Node<'a, '_>>) -> Option<&'a str> {
    let first = elem?.first_child()?;
    if first.is_text() { first.text() } else { None }
}

/// Stat from a namespaced element, or `None` when the feed does not carry it. `None` is
/// "unknown" and must never render as 0 (a real zero seed count is dead; an absent count
/// is just absent).
fn opt_int_text(elem: Option<Node>) -> Option<i64> {
    let t = text(elem).filter(|t| !t.is_empty())?;
    t.trim().parse().ok()
}

fn str_text(elem: Option<Node>) -> String {
    text(elem).map(|t| t.trim().to_string()).unwrap_or_default()
}

/// Bytes in nyaa's own notation ("284.7 MiB").
fn human_size(n: f64) -> String {
    let mut f = n;
    for unit in ["B", "KiB", "MiB", "GiB", "TiB"] {
        if f < 1024.0 || unit == "TiB" {
            return if unit == "B" {
                format!("{} B", f as u64)
            } else {
                format!("{f:.1} {unit}")
            };
        }
        f /= 1024.0;
    }
    format!("{n} B")
}

struct Torznab {
    seeders: Option<i64>,
    leechers: Option<i64>,
    grabs: Option<i64>,
    size: String,
    category: String,
}

/// (seeders, leechers, grabs, size, category) from a torznab item, all empty or `None`
/// when the item has no `torznab:attr` elements. Torznab reports peers as seeders plus
/// leechers.
fn torznab(entry: Node) -> Torznab {
    let mut attrs: Vec<(String, String)> = Vec::new();
    for attr in entry.children().filter(|c| {
        c.is_element()
            && c.tag_name().name() == "attr"
            && c.tag_name().namespace() == Some(TORZNAB_NS)
    }) {
        if let (Some(name), Some(value)) = (attr.attribute("name"), attr.attribute("value"))
            && !name.is_empty()
            && !attrs.iter().any(|(k, _)| k == name)
        {
            attrs.push((name.to_string(), value.to_string()));
        }
    }
    if attrs.is_empty() {
        return Torznab {
            seeders: None,
            leechers: None,
            grabs: None,
            size: String::new(),
            category: String::new(),
        };
    }
    let get = |key: &str| {
        attrs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    };
    let num = |key: &str| get(key).and_then(|v| v.trim().parse::<i64>().ok());
    let (seeders, peers) = (num("seeders"), num("peers"));
    let leechers = match (peers, seeders) {
        (Some(p), Some(s)) => Some((p - s).max(0)),
        _ => None,
    };
    let mut size = String::new();
    let raw = Some(str_text(child(entry, None, "size")))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| get("size").unwrap_or("").to_string());
    if !raw.is_empty()
        && raw.bytes().all(|b| b.is_ascii_digit())
        && let Ok(n) = raw.parse::<f64>()
    {
        size = human_size(n);
    }
    let cat = get("category").unwrap_or("");
    let category = match cat.chars().next() {
        Some(first) if cat.chars().count() == 4 => torznab_category(&first.to_string()).to_string(),
        _ => String::new(),
    };
    Torznab {
        seeders,
        leechers,
        grabs: num("grabs"),
        size,
        category,
    }
}

static DESC_SIZE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Size:\s*([\d.,]+\s*[KMGT]?i?B)").unwrap());
static DESC_CAT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Category:\s*([^|]+)").unwrap());
static HREF_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"href="([^"]+)""#).unwrap());

/// Prefer a torrent-file enclosure over `<link>`: on generic trackers the link is a web
/// page, while the enclosure is the actual .torrent.
fn item_link(entry: Node, link_text: &str) -> String {
    if let Some(enclosure) = child(entry, None, "enclosure") {
        let url = enclosure.attribute("url").unwrap_or("").trim();
        let kind = enclosure.attribute("type").unwrap_or("").to_lowercase();
        if !url.is_empty()
            && (kind.contains("bittorrent") || url.to_lowercase().ends_with(".torrent"))
        {
            return url.to_string();
        }
    }
    link_text.to_string()
}

#[derive(Default)]
struct Erai {
    title: String,
    size: String,
    category: String,
    page: String,
    resolution: String,
    infomkv: String,
    linktype: String,
}

/// The title and extras of an Erai-raws feed item, or the title untouched with empty extras
/// when the item has no erai: extensions.
///
/// Erai titles read "[Torrent] Show - 02 [tags][us][br]...[Airing]": the linktype prefix
/// and the subtitle-language run are noise (the languages live in erai:subtitles), while
/// the trailing [Airing], [Encoded] or [Batch] tag is kept as a real refine facet. The post
/// URL from the description becomes `page` so the per-show feed slug can be derived from
/// it. resolution, infomkv and linktype are the structured quality facets: they exist
/// because the `s=` search cannot match them (see `erai::split_noise`).
fn erai_polish(entry: Node, title: &str) -> Erai {
    let e = Some(ERAI_NS);
    let linktype = str_text(child(entry, e, "linktype"));
    if linktype.is_empty() {
        return Erai {
            title: title.to_string(),
            ..Erai::default()
        };
    }
    let mut title = title.to_string();
    let lead = format!("[{linktype}] ");
    if let Some(rest) = title.strip_prefix(&lead) {
        title = rest.to_string();
    }
    let subs_tags = str_text(child(entry, e, "subtitles"));
    if !subs_tags.is_empty() {
        title = title.replace(&subs_tags, "");
    }
    let description = str_text(child(entry, None, "description"));
    Erai {
        title,
        size: str_text(child(entry, e, "size")),
        category: str_text(child(entry, e, "category"))
            .trim_matches(['[', ']'])
            .to_string(),
        page: HREF_RE
            .captures(&description)
            .map(|m| m[1].to_string())
            .unwrap_or_default(),
        resolution: str_text(child(entry, e, "resolution")),
        infomkv: str_text(child(entry, e, "infomkv")),
        linktype,
    }
}

/// Parse an RSS feed into releases. Reads nyaa's stat extensions when present; otherwise
/// falls back to Size: and Category: hints in `<description>`. Errors on malformed XML.
pub fn parse_rss(data: &str, endpoint_name: &str) -> Result<Vec<Release>, CoreError> {
    let data = data.strip_prefix('\u{feff}').unwrap_or(data);
    let doc = roxmltree::Document::parse(data).map_err(|e| CoreError::Service {
        service: endpoint_name.to_string(),
        code: "unreadable".to_string(),
        message: format!("bad feed: {e}"),
    })?;
    let n = Some(NYAA_NS);
    let mut out = Vec::new();
    for entry in doc.descendants().filter(|d| {
        d.is_element() && d.tag_name().name() == "item" && d.tag_name().namespace().is_none()
    }) {
        let title = child(entry, None, "title");
        let link = child(entry, None, "link");
        let (Some(title_raw), Some(link_raw)) = (
            text(title).filter(|t| !t.is_empty()),
            text(link).filter(|t| !t.is_empty()),
        ) else {
            continue;
        };
        let erai = erai_polish(entry, title_raw.trim());
        let mut title_text = erai.title;
        let tz = torznab(entry);
        let first = |a: String, b: String, c: String| {
            [a, b, c]
                .into_iter()
                .find(|s| !s.is_empty())
                .unwrap_or_default()
        };
        let mut size = first(str_text(child(entry, n, "size")), erai.size, tz.size);
        let mut category = first(
            str_text(child(entry, n, "category")),
            erai.category,
            tz.category,
        );
        if size.is_empty() || category.is_empty() {
            // CDATA descriptions arrive HTML-escaped (AniRena writes "Anime &gt; Subs"), so
            // unescape what the regexes pull out.
            let desc = html_unescape(&str_text(child(entry, None, "description")));
            if !desc.is_empty() {
                if size.is_empty()
                    && let Some(m) = DESC_SIZE_RE.captures(&desc)
                {
                    size = m[1].trim().to_string();
                }
                if category.is_empty()
                    && let Some(m) = DESC_CAT_RE.captures(&desc)
                {
                    category = m[1].trim().to_string();
                }
            }
        }
        // Generic trackers prefix titles with "[<category>] ": strip it so release-group
        // detection does not mistake the category for a poster.
        if !category.is_empty()
            && let Some(rest) = title_text.strip_prefix(&format!("[{category}] "))
        {
            title_text = rest.to_string();
        }
        out.push(Release {
            title: title_text,
            link: item_link(entry, link_raw.trim()),
            seeders: tz
                .seeders
                .or_else(|| opt_int_text(child(entry, n, "seeders"))),
            leechers: tz
                .leechers
                .or_else(|| opt_int_text(child(entry, n, "leechers"))),
            downloads: tz
                .grabs
                .or_else(|| opt_int_text(child(entry, n, "downloads"))),
            size,
            category,
            page: erai.page,
            source: String::new(),
            resolution: erai.resolution,
            infomkv: erai.infomkv,
            linktype: erai.linktype,
        });
    }
    Ok(out)
}

/// GET `url` and parse it as an RSS feed. `source_name` names the endpoint in errors; the
/// URL stays out of them, since a torznab URL carries the indexer's API key.
pub async fn fetch_rss(
    http: &dyn Http,
    url: &str,
    source_name: &str,
    timeout: Duration,
) -> Result<Vec<Release>, CoreError> {
    tracing::debug!("GET {}", host_of(url));
    let response = http
        .send(HttpRequest {
            method: Method::Get,
            url: url.to_string(),
            headers: vec![
                ("User-Agent".to_string(), USER_AGENT.to_string()),
                (
                    "Accept".to_string(),
                    "application/rss+xml, application/xml, text/xml, */*".to_string(),
                ),
            ],
            body: None,
            timeout: Some(timeout),
        })
        .await
        .map_err(|e| CoreError::Service {
            service: source_name.to_string(),
            code: "unreachable".to_string(),
            message: e.message,
        })?;
    if !response.is_success() {
        return Err(CoreError::Service {
            service: source_name.to_string(),
            code: format!("http_{}", response.status),
            message: format!("the feed answered {}", response.status),
        });
    }
    let body = String::from_utf8_lossy(&response.body);
    let items = parse_rss(&body, source_name)?;
    tracing::debug!("  -> {} items", items.len());
    Ok(items)
}

// Python's html.unescape, for the descriptions the Size: and Category: regexes read ----

/// `html.unescape`: numeric references exactly as Python resolves them, and the named
/// references feeds actually use. Python knows the whole HTML5 table; this one knows the
/// common names, with and without the trailing semicolon where HTML allows it.
fn html_unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    static REF: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"&(#[0-9]+;?|#[xX][0-9a-fA-F]+;?|[^\t\n\x0c <&#;]{1,32};?)").unwrap()
    });
    REF.replace_all(s, |caps: &regex::Captures| replace_ref(&caps[1], &caps[0]))
        .into_owned()
}

fn replace_ref(body: &str, whole: &str) -> String {
    if let Some(num) = body.strip_prefix('#') {
        let num = num.trim_end_matches(';');
        let value = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16),
            None => num.parse::<u32>(),
        };
        // An overflowing number is past U+10FFFF either way.
        return charref(value.unwrap_or(u32::MAX));
    }
    if let Some(c) = named(body) {
        return c.to_string();
    }
    // Longest prefix that is a reference HTML allows without its semicolon.
    for end in (2..body.len()).rev() {
        if body.is_char_boundary(end)
            && let Some(c) = named_legacy(&body[..end])
        {
            return format!("{c}{}", &body[end..]);
        }
    }
    whole.to_string()
}

fn named_legacy(name: &str) -> Option<char> {
    match name {
        "amp" | "AMP" => Some('&'),
        "lt" | "LT" => Some('<'),
        "gt" | "GT" => Some('>'),
        "quot" | "QUOT" => Some('"'),
        "nbsp" => Some('\u{a0}'),
        "copy" | "COPY" => Some('\u{a9}'),
        "reg" | "REG" => Some('\u{ae}'),
        _ => None,
    }
}

fn named(body: &str) -> Option<char> {
    if let Some(name) = body.strip_suffix(';') {
        let c = match name {
            "apos" => '\'',
            "hellip" => '\u{2026}',
            "ndash" => '\u{2013}',
            "mdash" => '\u{2014}',
            "lsquo" => '\u{2018}',
            "rsquo" => '\u{2019}',
            "ldquo" => '\u{201c}',
            "rdquo" => '\u{201d}',
            "bull" => '\u{2022}',
            "middot" => '\u{b7}',
            "times" => '\u{d7}',
            "deg" => '\u{b0}',
            "plusmn" => '\u{b1}',
            "laquo" => '\u{ab}',
            "raquo" => '\u{bb}',
            "euro" => '\u{20ac}',
            "trade" => '\u{2122}',
            "hearts" => '\u{2665}',
            _ => return named_legacy(name),
        };
        return Some(c);
    }
    named_legacy(body)
}

/// Python's `_replace_charref` for a numeric reference.
fn charref(num: u32) -> String {
    const CP1252: [char; 32] = [
        '\u{20ac}', '\u{81}', '\u{201a}', '\u{192}', '\u{201e}', '\u{2026}', '\u{2020}',
        '\u{2021}', '\u{2c6}', '\u{2030}', '\u{160}', '\u{2039}', '\u{152}', '\u{8d}', '\u{17d}',
        '\u{8f}', '\u{90}', '\u{2018}', '\u{2019}', '\u{201c}', '\u{201d}', '\u{2022}', '\u{2013}',
        '\u{2014}', '\u{2dc}', '\u{2122}', '\u{161}', '\u{203a}', '\u{153}', '\u{9d}', '\u{17e}',
        '\u{178}',
    ];
    match num {
        0x00 => return '\u{fffd}'.to_string(),
        0x0d => return "\r".to_string(),
        0x80..=0x9f => return CP1252[(num - 0x80) as usize].to_string(),
        _ => {}
    }
    if (0xd800..=0xdfff).contains(&num) || num > 0x10ffff {
        return '\u{fffd}'.to_string();
    }
    let invalid = matches!(num, 0x1..=0x8 | 0xb | 0xe..=0x1f | 0x7f..=0x9f | 0xfdd0..=0xfdef)
        || (num & 0xfffe) == 0xfffe;
    if invalid {
        return String::new();
    }
    char::from_u32(num).map(String::from).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;

    const NYAA_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:nyaa="https://nyaa.si/xmlns/nyaa">
<channel>
  <item>
    <title>[Erai-raws] Show - 05 [1080p][Multiple Subtitle]</title>
    <link>https://nyaa.si/download/1837471.torrent</link>
    <nyaa:seeders>923</nyaa:seeders>
    <nyaa:leechers>12</nyaa:leechers>
    <nyaa:downloads>4051</nyaa:downloads>
    <nyaa:size>1.4 GiB</nyaa:size>
    <nyaa:category>Anime - English-translated</nyaa:category>
  </item>
</channel>
</rss>"#;

    const ANIRENA_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
  <title>AniRena</title>
  <item>
    <title>[Anime &gt; Subs] Sayonara Lara - 02 [1080p CR WEBRip][768DB037]</title>
    <link>https://www.anirena.com/torrents/019f57b3</link>
    <description><![CDATA[Size: 485.1 MB | Uploader: Erai-raws | Category: Anime &gt; Subs]]></description>
    <enclosure url="https://www.anirena.com/torrents/019f57b3.torrent" type="application/x-bittorrent" length="0"/>
  </item>
  <item>
    <title>Bare Title Without Prefix - 03</title>
    <link>https://www.anirena.com/torrents/019f57a5</link>
  </item>
</channel>
</rss>"#;

    const ERAI_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:erai="https://www.erai-raws.info/rss-page/">
<channel>
  <title>Episodes &#8211; Erai-raws Torrent RSS</title>
  <item>
    <title>[Torrent] Gaikotsu Kishi-sama Tadaima Isekai e Odekake-chuu II - 02 [1080p CR WEB-DL AVC AAC][us][br][mx][Airing]</title>
    <link>https://t.erai-raws.info/Torrent/2026/Summer/Gaikotsu Kishi-sama/file.mkv.torrent</link>
    <erai:resolution>1080p</erai:resolution>
    <erai:infomkv>CR WEB-DL AVC AAC</erai:infomkv>
    <erai:linktype>Torrent</erai:linktype>
    <erai:size>1.36GB</erai:size>
    <erai:infohash>88b0c4737b638b6c5bc38c458e812e2dc635f0dc</erai:infohash>
    <erai:subtitles>[us][br][mx]</erai:subtitles>
    <erai:category>[Airing]</erai:category>
    <description><![CDATA[<a href="https://www.erai-raws.info/episodes/gaikotsu-kishi-sama-tadaima-isekai-e-odekake-chuu-ii-02/">[Erai-raws] Gaikotsu Kishi-sama Tadaima Isekai e Odekake-chuu II - 02 [1080p CR WEB-DL AVC AAC][MultiSub][B8B9B54A].mkv</a> | Subtitles: [us][br][mx] | Size: 1.36GB | InfoHash: 88b0c4737b638b6c5bc38c458e812e2dc635f0dc | Categories: [Airing] ]]></description>
  </item>
</channel>
</rss>"#;

    const TORZNAB_XML: &str = r#"<?xml version="1.0"?>
<rss version="2.0" xmlns:torznab="http://torznab.com/schemas/2015/feed"><channel>
<item><title>Avatar 2160p</title><link>https://jackett/dl/1</link><size>1610612736</size>
<enclosure url="https://jackett/dl/1.torrent" length="1610612736" type="application/x-bittorrent"/>
<torznab:attr name="category" value="2040"/><torznab:attr name="category" value="5000"/>
<torznab:attr name="seeders" value="120"/><torznab:attr name="peers" value="150"/><torznab:attr name="grabs" value="900"/>
</item>
<item><title>Peers behind seeders</title><link>l2</link>
<torznab:attr name="seeders" value="9"/><torznab:attr name="peers" value="3"/><torznab:attr name="category" value="123"/><torznab:attr name="size" value="2048"/>
</item>
</channel></rss>"#;

    fn parse(xml: &str) -> Vec<Release> {
        parse_rss(xml, "test").unwrap()
    }

    #[test]
    fn parse_nyaa_feed_unchanged() {
        let items = parse(NYAA_XML);
        assert_eq!(items.len(), 1);
        let it = &items[0];
        assert_eq!(it.title, "[Erai-raws] Show - 05 [1080p][Multiple Subtitle]");
        assert_eq!(it.link, "https://nyaa.si/download/1837471.torrent");
        assert_eq!(
            (it.seeders, it.leechers, it.downloads),
            (Some(923), Some(12), Some(4051))
        );
        assert_eq!(it.size, "1.4 GiB");
        assert_eq!(it.category, "Anime - English-translated");
        assert_eq!(it.source, "");
    }

    #[test]
    fn parse_generic_feed_prefers_torrent_enclosure() {
        assert_eq!(
            parse(ANIRENA_XML)[0].link,
            "https://www.anirena.com/torrents/019f57b3.torrent"
        );
    }

    #[test]
    fn parse_generic_feed_description_fallbacks() {
        let it = &parse(ANIRENA_XML)[0];
        assert_eq!(it.size, "485.1 MB");
        assert_eq!(it.category, "Anime > Subs");
        // AniRena carries no stat extensions: unknown, not zero.
        assert_eq!((it.seeders, it.leechers, it.downloads), (None, None, None));
    }

    #[test]
    fn parse_generic_feed_strips_category_title_prefix() {
        assert_eq!(
            parse(ANIRENA_XML)[0].title,
            "Sayonara Lara - 02 [1080p CR WEBRip][768DB037]"
        );
    }

    #[test]
    fn parse_generic_feed_item_without_extras() {
        let it = &parse(ANIRENA_XML)[1];
        assert_eq!(it.title, "Bare Title Without Prefix - 03");
        assert_eq!(it.link, "https://www.anirena.com/torrents/019f57a5");
        assert!(it.size.is_empty() && it.category.is_empty());
    }

    #[test]
    fn parse_erai_feed_cleans_title() {
        // The linktype prefix and the subtitle-language run go; the category tag stays (a
        // real refine facet: [Airing] vs [Encoded] vs [Batch]).
        assert_eq!(
            parse(ERAI_XML)[0].title,
            "Gaikotsu Kishi-sama Tadaima Isekai e Odekake-chuu II - 02 [1080p CR WEB-DL AVC AAC][Airing]"
        );
    }

    #[test]
    fn parse_erai_feed_reads_erai_extensions() {
        let it = &parse(ERAI_XML)[0];
        assert_eq!(it.size, "1.36GB");
        assert_eq!(it.category, "Airing");
        assert!(it.link.ends_with(".torrent"));
    }

    #[test]
    fn parse_erai_feed_records_post_page() {
        assert_eq!(
            parse(ERAI_XML)[0].page,
            "https://www.erai-raws.info/episodes/gaikotsu-kishi-sama-tadaima-isekai-e-odekake-chuu-ii-02/"
        );
    }

    #[test]
    fn parse_erai_feed_captures_quality_metadata() {
        let it = &parse(ERAI_XML)[0];
        assert_eq!(it.resolution, "1080p");
        assert_eq!(it.infomkv, "CR WEB-DL AVC AAC");
        assert_eq!(it.linktype, "Torrent");
    }

    #[test]
    fn parse_generic_feed_has_no_erai_metadata() {
        for it in parse(ANIRENA_XML) {
            assert!(it.resolution.is_empty() && it.infomkv.is_empty() && it.linktype.is_empty());
        }
    }

    #[test]
    fn parse_erai_feed_stats_are_unknown_not_zero() {
        // The erai: namespace has no seed, peer or download stats; they must stay None so
        // 0 never masquerades as a dead torrent.
        let it = &parse(ERAI_XML)[0];
        assert!(it.seeders.is_none() && it.leechers.is_none() && it.downloads.is_none());
    }

    #[test]
    fn parse_rss_zero_stats_are_real_zeros() {
        let xml = NYAA_XML.replace(
            "<nyaa:seeders>923</nyaa:seeders>",
            "<nyaa:seeders>0</nyaa:seeders>",
        );
        let it = &parse(&xml)[0];
        assert_eq!(
            (it.seeders, it.leechers, it.downloads),
            (Some(0), Some(12), Some(4051))
        );
    }

    #[test]
    fn parse_rss_bad_xml_is_an_error_naming_the_endpoint() {
        match parse_rss("<not-xml", "anirena").unwrap_err() {
            CoreError::Service { service, code, .. } => {
                assert_eq!((service.as_str(), code.as_str()), ("anirena", "unreadable"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn parse_rss_skips_items_without_a_title_or_link() {
        let xml = "<rss><channel><item><title>a</title></item><item><link>l</link></item>\
                   <item><title></title><link>l</link></item><item><title>ok</title><link> l </link></item></channel></rss>";
        let items = parse(xml);
        assert_eq!(items.len(), 1);
        assert_eq!(
            (items[0].title.as_str(), items[0].link.as_str()),
            ("ok", "l")
        );
    }

    #[test]
    fn parse_torznab_reads_stats_size_and_category() {
        let items = parse(TORZNAB_XML);
        let a = &items[0];
        assert_eq!(
            (a.seeders, a.leechers, a.downloads),
            (Some(120), Some(30), Some(900))
        );
        assert_eq!(a.size, "1.5 GiB");
        assert_eq!(a.category, "Movies");
        assert_eq!(a.link, "https://jackett/dl/1.torrent");
        // Peers below seeders clamp to zero; a category id of the wrong length gives none;
        // the size attribute is used when there is no size element.
        let b = &items[1];
        assert_eq!(
            (b.seeders, b.leechers, b.downloads),
            (Some(9), Some(0), None)
        );
        assert_eq!(b.category, "");
        assert_eq!(b.size, "2.0 KiB");
    }

    #[test]
    fn human_size_uses_nyaas_notation() {
        assert_eq!(human_size(0.0), "0 B");
        assert_eq!(human_size(1023.0), "1023 B");
        assert_eq!(human_size(1024.0), "1.0 KiB");
        assert_eq!(human_size(298_513_817.0), "284.7 MiB");
        assert_eq!(human_size(2.0 * 1024f64.powi(5)), "2048.0 TiB");
    }

    #[test]
    fn html_unescape_matches_python() {
        assert_eq!(
            html_unescape("Anime &gt; Subs &amp; more"),
            "Anime > Subs & more"
        );
        assert_eq!(html_unescape("&#65;&#x42;&#X43;"), "ABC");
        assert_eq!(
            html_unescape("&#0;|&#128;|&#xD800;|&#1114112;|&#1;"),
            "\u{fffd}|\u{20ac}|\u{fffd}|\u{fffd}|"
        );
        // The legacy names work without their semicolon, and a longer name keeps its tail.
        assert_eq!(html_unescape("a &amp b &ampx c"), "a & b &x c");
        assert_eq!(html_unescape("&nope; & &"), "&nope; & &");
        assert_eq!(html_unescape("plain"), "plain");
    }

    #[test]
    fn description_size_regex_is_case_insensitive_and_takes_units() {
        let xml = r#"<rss><channel><item><title>t</title><link>l</link>
<description>size: 1,5 gib | Category: Anime &amp;gt; Raw | x</description></item></channel></rss>"#;
        let it = &parse(xml)[0];
        // The CDATA-less description is unescaped once by the parser, then once by us.
        assert_eq!(it.size, "1,5 gib");
        assert_eq!(it.category, "Anime > Raw");
    }

    #[tokio::test]
    async fn fetch_rss_sends_firefox_and_parses() {
        let http = FakeHttp::new();
        http.push(200, NYAA_XML);
        let items = fetch_rss(
            http.as_ref(),
            "https://nyaa.si/?page=rss",
            "nyaa",
            DEFAULT_TIMEOUT,
        )
        .await
        .unwrap();
        assert_eq!(items.len(), 1);
        let req = &http.requests()[0];
        assert!(
            req.headers
                .iter()
                .any(|(k, v)| k == "User-Agent" && v.contains("Firefox/130.0"))
        );
        assert_eq!(req.timeout, Some(DEFAULT_TIMEOUT));
    }

    #[tokio::test]
    async fn fetch_rss_transport_error_names_the_endpoint_not_the_url() {
        let http = FakeHttp::new();
        http.fail_next("boom");
        let err = fetch_rss(
            http.as_ref(),
            "https://example.invalid/rss?apikey=SECRET",
            "anirena",
            DEFAULT_TIMEOUT,
        )
        .await
        .unwrap_err();
        let text = err.to_string();
        assert!(text.contains("anirena") && text.contains("boom"), "{text}");
        assert!(!text.contains("SECRET"));
    }

    #[tokio::test]
    async fn fetch_rss_http_error_is_a_service_error() {
        let http = FakeHttp::new();
        http.push(502, "bad gateway");
        match fetch_rss(http.as_ref(), "https://erai/x", "erai", DEFAULT_TIMEOUT)
            .await
            .unwrap_err()
        {
            CoreError::Service { service, code, .. } => {
                assert_eq!((service.as_str(), code.as_str()), ("erai", "http_502"));
            }
            other => panic!("{other:?}"),
        }
    }
}
