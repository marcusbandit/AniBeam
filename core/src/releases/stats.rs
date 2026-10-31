//! Tracker stats for feeds that do not carry them (Erai-raws and others). Ported from
//! anirss `stats.py`.
//!
//! A feed item without nyaa-style stat extensions has unknown counts, but the numbers do
//! exist: they live at the torrent's tracker, which is where qBittorrent reads them from
//! after adding. This module does the same without adding the torrent:
//!
//! 1. download the small .torrent file,
//! 2. pull the info-hash and announce list out of its bencode,
//! 3. scrape the trackers (HTTP batch first, UDP fallback), which answers seeders,
//!    leechers and completed per info-hash in one tiny request.
//!
//! A magnet link skips the download: its `xt=urn:btih:` is the info-hash and its `tr=`
//! parameters are the trackers.
//!
//! `enrich` is the only entry point the flows call. It never fails: any failure leaves the
//! stats unknown and logs. Results are cached in memory with a TTL, so keystroke-driven
//! live previews and refine refetches mostly cost nothing. anirss also keeps that cache in
//! a `stats.json` between runs; the core has no state directory for it, so this port
//! keeps it for the life of the process only.

use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex};
use std::task::Poll;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::net::UdpSocket;

use super::endpoints::{quote, split_url, unquote_plus};
use crate::contract::Release;
use crate::downloads::replace::magnet_hash;
use crate::net::{BoxFuture, Http, HttpRequest, Method};

/// Tracker counts go stale quickly.
pub const CACHE_TTL: Duration = Duration::from_secs(2 * 60 * 60);
/// Per HTTP scrape request.
pub const HTTP_SCRAPE_TIMEOUT: Duration = Duration::from_secs(8);
/// For each step of the UDP connect and scrape.
pub const UDP_SCRAPE_TIMEOUT: Duration = Duration::from_secs(5);
/// Per .torrent download.
pub const TORRENT_TIMEOUT: Duration = Duration::from_secs(10);
/// Info-hashes per HTTP scrape.
pub const MAX_HTTP_BATCH: usize = 50;
/// Keeps the datagram well under the MTU.
pub const MAX_UDP_BATCH: usize = 60;
pub const MAX_TRACKERS_TRIED: usize = 4;
pub const DOWNLOAD_WORKERS: usize = 8;
/// anirss `enrich(deadline_s=20.0)`.
pub const DEFAULT_DEADLINE: Duration = Duration::from_secs(20);

const USER_AGENT: &str = "anirss/1.0";
/// BEP 15 protocol id.
const UDP_MAGIC: u64 = 0x41727101980;

/// An info-hash: the SHA-1 of the torrent's info dictionary.
pub type Hash = [u8; 20];
/// Stats as stored: (seeders, leechers, downloads).
pub type Stats = (i64, i64, i64);

// SHA-1 (RFC 3174), for the info-hash --------------------------------------------------

pub(crate) fn sha1(data: &[u8]) -> Hash {
    let mut h: [u32; 5] = [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for block in msg.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in block.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = h;
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let t = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = t;
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e]) {
            *x = x.wrapping_add(y);
        }
    }
    let mut out = [0u8; 20];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

// Minimal bencode (read-only) ----------------------------------------------------------

#[derive(Debug, PartialEq)]
enum Value {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Dict(Vec<(Value, Value)>),
}

/// Anything that does not parse (anirss's `ValueError`).
#[derive(Debug, PartialEq)]
pub struct BadBencode;

/// Deeper than any real torrent; Python would hit its recursion limit here.
const MAX_DEPTH: usize = 64;

fn index_of(data: &[u8], needle: u8, from: usize) -> Result<usize, BadBencode> {
    data.get(from..)
        .and_then(|rest| rest.iter().position(|b| *b == needle))
        .map(|i| from + i)
        .ok_or(BadBencode)
}

fn find(data: &[u8], needle: &[u8]) -> Result<usize, BadBencode> {
    data.windows(needle.len())
        .position(|w| w == needle)
        .ok_or(BadBencode)
}

fn parse_int(raw: &[u8]) -> Result<i64, BadBencode> {
    std::str::from_utf8(raw)
        .map_err(|_| BadBencode)?
        .trim()
        .parse()
        .map_err(|_| BadBencode)
}

/// Decode one bencode value at `pos`; return (value, offset past it).
fn decode(data: &[u8], pos: usize, depth: usize) -> Result<(Value, usize), BadBencode> {
    if depth > MAX_DEPTH {
        return Err(BadBencode);
    }
    match data.get(pos) {
        Some(b'd') => {
            let mut pos = pos + 1;
            let mut out = Vec::new();
            while data.get(pos) != Some(&b'e') {
                let (key, next) = decode(data, pos, depth + 1)?;
                if !matches!(key, Value::Int(_) | Value::Bytes(_)) {
                    return Err(BadBencode);
                }
                let (val, next) = decode(data, next, depth + 1)?;
                out.push((key, val));
                pos = next;
            }
            Ok((Value::Dict(out), pos + 1))
        }
        Some(b'l') => {
            let mut pos = pos + 1;
            let mut out = Vec::new();
            while data.get(pos) != Some(&b'e') {
                let (val, next) = decode(data, pos, depth + 1)?;
                out.push(val);
                pos = next;
            }
            Ok((Value::List(out), pos + 1))
        }
        Some(b'i') => {
            let end = index_of(data, b'e', pos)?;
            Ok((Value::Int(parse_int(&data[pos + 1..end])?), end + 1))
        }
        _ => {
            let colon = index_of(data, b':', pos)?;
            let n: usize = parse_int(&data[pos..colon])?
                .try_into()
                .map_err(|_| BadBencode)?;
            let end = (colon + 1).checked_add(n).ok_or(BadBencode)?;
            let bytes = data.get(colon + 1..end).ok_or(BadBencode)?;
            Ok((Value::Bytes(bytes.to_vec()), end))
        }
    }
}

/// SHA-1 info-hash of .torrent bytes (trackerless torrents too).
pub fn info_hash(data: &[u8]) -> Result<Hash, BadBencode> {
    let start = find(data, b"4:info")? + b"4:info".len();
    let (_info, end) = decode(data, start, 0)?;
    Ok(sha1(&data[start..end]))
}

/// (info-hash bytes, announce URLs in priority order) from .torrent bytes. An error means
/// "can't scrape this one".
pub fn torrent_facts(data: &[u8]) -> Result<(Hash, Vec<String>), BadBencode> {
    let start = find(data, b"4:info")? + b"4:info".len();
    let (_info, end) = decode(data, start, 0)?;
    let infohash = sha1(&data[start..end]);

    let mut single: Option<String> = None;
    let mut from_list: Vec<String> = Vec::new();
    let mut pos = 1; // skip the leading dict marker
    while pos + 1 < data.len() && data[pos] != b'e' {
        let (key, next) = decode(data, pos, 0)?;
        let val_start = next;
        let (val, next) = decode(data, next, 0)?;
        pos = next;
        let key = match &key {
            Value::Bytes(b) => b.as_slice(),
            _ => &[],
        };
        if key == b"announce-list" && from_list.is_empty() {
            let entries: Vec<&Value> = match &val {
                Value::List(l) => l.iter().collect(),
                other => vec![other],
            };
            for entry in entries {
                let urls: Vec<&Value> = match entry {
                    Value::List(l) => l.iter().collect(),
                    other => vec![other],
                };
                for url in urls {
                    let text = match url {
                        Value::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
                        _ => String::new(),
                    };
                    if ["http://", "https://", "udp://"]
                        .iter()
                        .any(|p| text.starts_with(p))
                    {
                        from_list.push(text);
                    }
                }
            }
        } else if key == b"announce" && single.is_none() {
            single = Some(match &val {
                Value::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
                _ => String::new(),
            });
        }
        if val_start >= start {
            break; // inside or past the info dict: the top-level walk is done
        }
    }
    let announces = if !from_list.is_empty() {
        from_list
    } else {
        single.filter(|s| !s.is_empty()).into_iter().collect()
    };
    if announces.is_empty() {
        return Err(BadBencode); // no announce URL
    }
    Ok((infohash, announces))
}

// Magnet links --------------------------------------------------------------------------

/// (info-hash, trackers in the order given) from a magnet link's `xt=urn:btih:` and `tr=`
/// parameters, keeping the http, https and udp trackers. `None` when the link is not a
/// magnet, its hash is malformed or it names no tracker to scrape.
pub fn magnet_facts(link: &str) -> Option<(Hash, Vec<String>)> {
    let query = link.trim().strip_prefix("magnet:?")?;
    // The hex or base32 hash, read the way a download reads it, as 20 bytes.
    let hex = magnet_hash(link)?;
    let mut hash = [0u8; 20];
    for (i, byte) in hash.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    let mut trackers: Vec<String> = Vec::new();
    for (key, value) in query.split('&').filter_map(|p| p.split_once('=')) {
        // A magnet's values are percent-encoded; a `+` in one is a plus, not a space.
        let value = unquote_plus(&value.replace('+', "%2B"));
        if key.eq_ignore_ascii_case("tr")
            && ["http://", "https://", "udp://"]
                .iter()
                .any(|p| value.starts_with(p))
            && !trackers.contains(&value)
        {
            trackers.push(value);
        }
    }
    (!trackers.is_empty()).then_some((hash, trackers))
}

// Scraping ------------------------------------------------------------------------------

/// Byte offset just past the nested value starting at `pos`.
fn bencode_nested_end(data: &[u8], mut pos: usize) -> Result<usize, BadBencode> {
    let mut depth: i64 = 0;
    loop {
        match data.get(pos) {
            Some(b'd' | b'l') => {
                depth += 1;
                pos += 1;
            }
            Some(b'i') => pos = index_of(data, b'e', pos)? + 1,
            Some(b'e') => {
                depth -= 1;
                pos += 1;
                if depth == 0 {
                    return Ok(pos);
                }
            }
            _ => {
                let colon = index_of(data, b':', pos)?;
                let n: usize = parse_int(&data[pos..colon])?
                    .try_into()
                    .map_err(|_| BadBencode)?;
                pos = (colon + 1).checked_add(n).ok_or(BadBencode)?;
            }
        }
    }
}

/// The integer after `key` in a scrape entry, 0 when it is not there. anirss's
/// `re.search(key + rb"i(-?\d+)e", seg)`.
fn bkey_int(seg: &[u8], key: &[u8]) -> i64 {
    let mut from = 0;
    while let Some(i) = seg
        .get(from..)
        .and_then(|r| r.windows(key.len()).position(|w| w == key))
    {
        let at = from + i + key.len();
        if seg.get(at) == Some(&b'i') {
            let mut end = at + 1;
            if seg.get(end) == Some(&b'-') {
                end += 1;
            }
            let digits = end;
            while seg.get(end).is_some_and(u8::is_ascii_digit) {
                end += 1;
            }
            if end > digits && seg.get(end) == Some(&b'e') {
                return std::str::from_utf8(&seg[at + 1..end])
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        from += i + 1;
    }
    0
}

/// Parse `d5:filesd20:<hash>d8:completei N e10:incompletei N e10:downloadedi N e ... ee`.
fn parse_http_scrape(body: &[u8]) -> Result<HashMap<Hash, Stats>, BadBencode> {
    let files_at = find(body, b"d5:filesd")? + b"d5:filesd".len();
    let mut out = HashMap::new();
    let mut pos = files_at;
    while body.get(pos..pos + 3) == Some(b"20:") {
        let h: Hash = body
            .get(pos + 3..pos + 23)
            .ok_or(BadBencode)?
            .try_into()
            .map_err(|_| BadBencode)?;
        let seg_end = bencode_nested_end(body, pos + 23)?;
        let seg = &body[pos + 23..seg_end];
        out.insert(
            h,
            (
                bkey_int(seg, b"8:complete"),
                bkey_int(seg, b"10:incomplete"),
                bkey_int(seg, b"10:downloaded"),
            ),
        );
        pos = seg_end;
    }
    Ok(out)
}

/// Scrape an http(s) tracker (announce becomes scrape, BEP 48 batching).
async fn http_scrape(http: &dyn Http, announce: &str, hashes: &[Hash]) -> HashMap<Hash, Stats> {
    let Ok(mut parts) = split_url(announce) else {
        return HashMap::new();
    };
    let path = if parts.path.is_empty() {
        "/announce".to_string()
    } else {
        parts.path.clone()
    };
    if !matches!(parts.scheme.as_str(), "http" | "https") || !path.ends_with("/announce") {
        return HashMap::new();
    }
    parts.path = format!("{}scrape", &path[..path.len() - "announce".len()]);
    let scrape_url = parts.geturl();
    let shown = scrape_url.split('?').next().unwrap_or("").to_string();
    let mut out = HashMap::new();
    for batch in hashes.chunks(MAX_HTTP_BATCH) {
        let qs = batch
            .iter()
            .map(|h| format!("info_hash={}", quote(h, "/")))
            .collect::<Vec<_>>()
            .join("&");
        let request = HttpRequest {
            method: Method::Get,
            url: format!("{scrape_url}?{qs}"),
            headers: vec![("User-Agent".to_string(), USER_AGENT.to_string())],
            body: None,
            timeout: Some(HTTP_SCRAPE_TIMEOUT),
        };
        let body = match http.send(request).await {
            Ok(r) if r.is_success() => r.body,
            Ok(r) => {
                tracing::warn!("scrape {shown}: HTTP {}", r.status);
                continue;
            }
            Err(e) => {
                tracing::warn!("scrape {shown}: {}", e.message);
                continue;
            }
        };
        match parse_http_scrape(&body) {
            Ok(found) => out.extend(found),
            Err(_) => tracing::warn!("scrape {shown}: unreadable answer"),
        }
    }
    out
}

/// UDP tracker scrape (BEP 15): connect for a lease, then action 2.
///
/// The UDP scrape response's per-hash order is (seeders, completed, leechers): completed
/// and leechers swap places against the HTTP answer.
async fn udp_scrape(announce: &str, hashes: &[Hash]) -> HashMap<Hash, Stats> {
    let Ok(parts) = split_url(announce) else {
        return HashMap::new();
    };
    let Some(host) = parts.hostname().filter(|_| parts.scheme == "udp") else {
        return HashMap::new();
    };
    let port = parts.port().filter(|p| *p != 0).unwrap_or(80);
    let hashes = &hashes[..hashes.len().min(MAX_UDP_BATCH)];
    match udp_exchange(&host, port, hashes).await {
        Ok(out) => out,
        Err(e) => {
            tracing::warn!("udp scrape {host}:{port}: {e}");
            HashMap::new()
        }
    }
}

/// The datagram exchange. `Ok` with an empty map is a tracker that answered wrongly;
/// `Err` is anirss's `OSError` and `struct.error` cases.
async fn udp_exchange(
    host: &str,
    port: u16,
    hashes: &[Hash],
) -> Result<HashMap<Hash, Stats>, String> {
    let mut out = HashMap::new();
    let addr = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| e.to_string())?
        .find(|a| a.is_ipv4())
        .ok_or_else(|| "no IPv4 address".to_string())?;
    let sock = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| e.to_string())?;
    let tid: u32 = rand::random();
    let timed = |what: &'static str| move |_| format!("{what} timed out");

    let mut hello = Vec::with_capacity(16);
    hello.extend_from_slice(&UDP_MAGIC.to_be_bytes());
    hello.extend_from_slice(&0u32.to_be_bytes());
    hello.extend_from_slice(&tid.to_be_bytes());
    tokio::time::timeout(UDP_SCRAPE_TIMEOUT, sock.send_to(&hello, addr))
        .await
        .map_err(timed("send"))?
        .map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 2048];
    let (n, _) = tokio::time::timeout(UDP_SCRAPE_TIMEOUT, sock.recv_from(&mut buf))
        .await
        .map_err(timed("receive"))?
        .map_err(|e| e.to_string())?;
    let resp = &buf[..n];
    if resp.len() < 16 {
        return Ok(out);
    }
    let action = u32::from_be_bytes(resp[0..4].try_into().unwrap());
    let conn_id = u64::from_be_bytes(resp[8..16].try_into().unwrap());
    if action != 0 {
        return Ok(out);
    }
    let next_tid = tid
        .checked_add(1)
        .ok_or_else(|| "transaction id out of range".to_string())?;
    let mut payload = Vec::with_capacity(16 + 20 * hashes.len());
    payload.extend_from_slice(&conn_id.to_be_bytes());
    payload.extend_from_slice(&2u32.to_be_bytes());
    payload.extend_from_slice(&next_tid.to_be_bytes());
    for h in hashes {
        payload.extend_from_slice(h);
    }
    tokio::time::timeout(UDP_SCRAPE_TIMEOUT, sock.send_to(&payload, addr))
        .await
        .map_err(timed("send"))?
        .map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 8 + 12 * hashes.len()];
    let (n, _) = tokio::time::timeout(UDP_SCRAPE_TIMEOUT, sock.recv_from(&mut buf))
        .await
        .map_err(timed("receive"))?
        .map_err(|e| e.to_string())?;
    let resp = &buf[..n];
    if resp.len() < 8 {
        return Ok(out);
    }
    if u32::from_be_bytes(resp[0..4].try_into().unwrap()) != 2 {
        return Ok(out);
    }
    for (i, h) in hashes.iter().enumerate() {
        let off = 8 + 12 * i;
        if off + 12 > resp.len() {
            break;
        }
        let word =
            |at: usize| u32::from_be_bytes(resp[off + at..off + at + 4].try_into().unwrap()) as i64;
        let (seeders, completed, leechers) = (word(0), word(4), word(8));
        out.insert(*h, (seeders, leechers, completed));
    }
    Ok(out)
}

async fn scrape_tracker_http(
    http: &dyn Http,
    announce: &str,
    hashes: &[Hash],
) -> HashMap<Hash, Stats> {
    let found = http_scrape(http, announce, hashes).await;
    if !found.is_empty() {
        return found;
    }
    udp_scrape(announce, hashes).await
}

// .torrent downloads --------------------------------------------------------------------

/// What a URL path may hold raw: RFC 3986 pchar, plus `/` and `%` so an already-encoded
/// URL passes through unchanged. A query may also hold `?`.
const PATH_SAFE: &str = "/%:@!$&'()*+,;=";
const QUERY_SAFE: &str = "/%:@!$&'()*+,;=?";

/// Percent-encode what an http(s) URL cannot hold raw.
///
/// Feed links carry raw spaces and brackets (t.erai-raws.info does). The host is left alone
/// so an IPv6 literal keeps its brackets. Errors on a malformed host, for example an
/// unclosed `[`.
pub fn encode_http_link(link: &str) -> Result<String, BadBencode> {
    let mut parts = split_url(link).map_err(|_| BadBencode)?;
    parts.path = quote(parts.path.as_bytes(), PATH_SAFE);
    parts.query = quote(parts.query.as_bytes(), QUERY_SAFE);
    parts.fragment = quote(parts.fragment.as_bytes(), QUERY_SAFE);
    Ok(parts.geturl())
}

/// Where the network is reached, so tests can stand in for it as anirss's do.
trait Backend: Sync {
    fn fetch_torrent<'a>(
        &'a self,
        url: &'a str,
        timeout: Duration,
    ) -> BoxFuture<'a, Option<Vec<u8>>>;
    fn scrape_tracker<'a>(
        &'a self,
        announce: &'a str,
        hashes: &'a [Hash],
    ) -> BoxFuture<'a, HashMap<Hash, Stats>>;
}

struct Net<'h>(&'h dyn Http);

impl Backend for Net<'_> {
    fn fetch_torrent<'a>(
        &'a self,
        url: &'a str,
        timeout: Duration,
    ) -> BoxFuture<'a, Option<Vec<u8>>> {
        Box::pin(async move {
            let shown = url.split('?').next().unwrap_or("");
            let Ok(encoded) = encode_http_link(url) else {
                tracing::warn!("torrent download failed ({shown}): malformed URL");
                return None;
            };
            let request = HttpRequest {
                method: Method::Get,
                url: encoded,
                headers: vec![("User-Agent".to_string(), USER_AGENT.to_string())],
                body: None,
                timeout: Some(timeout),
            };
            match self.0.send(request).await {
                Ok(r) if r.is_success() => Some(r.body),
                Ok(r) => {
                    tracing::warn!("torrent download failed ({shown}): HTTP {}", r.status);
                    None
                }
                Err(e) => {
                    tracing::warn!("torrent download failed ({shown}): {}", e.message);
                    None
                }
            }
        })
    }

    fn scrape_tracker<'a>(
        &'a self,
        announce: &'a str,
        hashes: &'a [Hash],
    ) -> BoxFuture<'a, HashMap<Hash, Stats>> {
        Box::pin(scrape_tracker_http(self.0, announce, hashes))
    }
}

/// Drive `pending` with at most `limit` in flight, until all finish or `wait` runs out.
/// What did not finish by then is dropped.
async fn run_bounded<'a, T>(
    mut pending: VecDeque<BoxFuture<'a, T>>,
    limit: usize,
    wait: Duration,
) -> Vec<T> {
    let mut running: Vec<BoxFuture<'a, T>> = Vec::new();
    let mut done: Vec<T> = Vec::new();
    let all = std::future::poll_fn(|cx| {
        loop {
            while running.len() < limit {
                match pending.pop_front() {
                    Some(f) => running.push(f),
                    None => break,
                }
            }
            if running.is_empty() {
                return Poll::Ready(());
            }
            let mut progressed = false;
            let mut i = 0;
            while i < running.len() {
                match running[i].as_mut().poll(cx) {
                    Poll::Ready(v) => {
                        done.push(v);
                        drop(running.swap_remove(i));
                        progressed = true;
                    }
                    Poll::Pending => i += 1,
                }
            }
            if !progressed {
                return Poll::Pending;
            }
        }
    });
    let _ = tokio::time::timeout(wait, all).await;
    done
}

/// A URL with the torrent bytes it gave, if any.
type Fetched<'a> = (&'a str, Option<Vec<u8>>);

/// The .torrent bytes behind each URL, `DOWNLOAD_WORKERS` at a time.
///
/// A URL that fails, misses the deadline or is not a torrent is left out. Each attempt's own
/// timeout shrinks with the time that is left, so even a fully stalled server can only cost
/// about `deadline` in total.
async fn fetch_torrents_with(
    backend: &dyn Backend,
    urls: &[String],
    deadline: Duration,
) -> HashMap<String, Vec<u8>> {
    let deadline_at = Instant::now() + deadline;
    let left = || deadline_at.saturating_duration_since(Instant::now());
    let mut seen: Vec<&str> = Vec::new();
    for u in urls {
        if !seen.contains(&u.as_str()) {
            seen.push(u);
        }
    }
    let attempt = left().max(Duration::from_millis(500));
    let pending: VecDeque<BoxFuture<'_, Fetched<'_>>> = seen
        .into_iter()
        .map(|url| {
            let f: BoxFuture<'_, Fetched<'_>> =
                Box::pin(async move { (url, backend.fetch_torrent(url, attempt).await) });
            f
        })
        .collect();
    let finished = run_bounded(
        pending,
        DOWNLOAD_WORKERS,
        left().max(Duration::from_secs(1)),
    )
    .await;
    let mut out = HashMap::new();
    for (url, data) in finished {
        if let Some(data) = data.filter(|d| !d.is_empty())
            && info_hash(&data).is_ok()
        {
            out.insert(url.to_string(), data);
        }
    }
    out
}

/// `fetch_torrents` over the real network.
pub async fn fetch_torrents(
    http: &dyn Http,
    urls: &[String],
    deadline: Duration,
) -> HashMap<String, Vec<u8>> {
    fetch_torrents_with(&Net(http), urls, deadline).await
}

// Cache ---------------------------------------------------------------------------------

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

/// URL to (when stored, stats), `None` stats for a dead link or a silent tracker.
#[derive(Default)]
struct StatsCache {
    map: Mutex<HashMap<String, (f64, Option<Stats>)>>,
}

enum Cached {
    Ok(Stats),
    Fail,
}

impl StatsCache {
    /// `Ok` or `Fail` for a fresh entry, `None` for a miss.
    fn get(&self, url: &str) -> Option<Cached> {
        let map = self.map.lock().unwrap();
        let (ts, stats) = map.get(url)?;
        if now_secs() - ts >= CACHE_TTL.as_secs_f64() {
            return None;
        }
        Some(match stats {
            Some(s) => Cached::Ok(*s),
            None => Cached::Fail,
        })
    }

    fn put(&self, url: &str, ts: f64, stats: Option<Stats>) {
        let mut map = self.map.lock().unwrap();
        map.retain(|_, (t, _)| now_secs() - *t < CACHE_TTL.as_secs_f64());
        map.insert(url.to_string(), (ts, stats));
    }
}

static CACHE: LazyLock<StatsCache> = LazyLock::new(StatsCache::default);

// Entry point ---------------------------------------------------------------------------

fn with_stats(mut item: Release, stats: Stats) -> Release {
    item.seeders = Some(stats.0);
    item.leechers = Some(stats.1);
    item.downloads = Some(stats.2);
    item
}

/// Fill in unknown seed, peer and download counts from the items' trackers.
///
/// Items whose feed already carries stats (nyaa) pass through untouched. For the rest,
/// stats come from scraping the .torrent's trackers, or for a magnet link the trackers its
/// `tr=` parameters name: the same numbers qBittorrent shows after adding. Never fails; on
/// any failure the item keeps its unknown stats.
pub async fn enrich(http: &dyn Http, items: Vec<Release>, deadline: Duration) -> Vec<Release> {
    enrich_with(&Net(http), &CACHE, items, deadline).await
}

async fn enrich_with(
    backend: &dyn Backend,
    cache: &StatsCache,
    items: Vec<Release>,
    deadline_s: Duration,
) -> Vec<Release> {
    if items.is_empty() {
        return items;
    }
    let deadline = Instant::now() + deadline_s;

    // Torrent URL or magnet link to item indexes, in first-seen order.
    let mut by_url: Vec<(String, Vec<usize>)> = Vec::new();
    let mut resolved: HashMap<usize, Stats> = HashMap::new();
    for (i, item) in items.iter().enumerate() {
        // Torznab feeds carry live seeders and leechers but often no download count;
        // scraping would only re-fetch numbers already in hand.
        if item.seeders.is_some() && item.leechers.is_some() {
            continue;
        }
        // A magnet is scraped only at the trackers its own `tr=` parameters name. There is
        // no fallback tracker list: the app ships no sources and names no tracker, so a
        // magnet without `tr=` (or with a malformed info-hash) stays unscraped.
        let scrapable = item.link.starts_with("http://")
            || item.link.starts_with("https://")
            || magnet_facts(&item.link).is_some();
        if scrapable {
            match cache.get(&item.link) {
                None => match by_url.iter_mut().find(|(u, _)| *u == item.link) {
                    Some((_, idxs)) => idxs.push(i),
                    None => by_url.push((item.link.clone(), vec![i])),
                },
                Some(Cached::Ok(stats)) => {
                    resolved.insert(i, stats);
                }
                // A known-bad URL: leave it unknown without fetching it again.
                Some(Cached::Fail) => {}
            }
        }
    }
    let apply = |items: Vec<Release>, resolved: &HashMap<usize, Stats>| -> Vec<Release> {
        items
            .into_iter()
            .enumerate()
            .map(|(i, it)| match resolved.get(&i) {
                Some(s) => with_stats(it, *s),
                None => it,
            })
            .collect()
    };
    if by_url.is_empty() {
        return apply(items, &resolved);
    }

    // Download the missing .torrent files in parallel, honouring the deadline. A magnet
    // carries its info-hash and trackers itself, so it needs no download.
    let urls: Vec<String> = by_url.iter().map(|(u, _)| u.clone()).collect();
    let torrent_urls: Vec<String> = urls
        .iter()
        .filter(|u| !u.starts_with("magnet:"))
        .cloned()
        .collect();
    let fetched = fetch_torrents_with(
        backend,
        &torrent_urls,
        deadline.saturating_duration_since(Instant::now()),
    )
    .await;
    let mut facts: HashMap<String, (Hash, Vec<String>)> = HashMap::new();
    for url in &urls {
        let fact = match magnet_facts(url) {
            Some(fact) => Some(fact),
            None => fetched.get(url).and_then(|data| torrent_facts(data).ok()),
        };
        if let Some(fact) = fact {
            facts.insert(url.clone(), fact);
        }
    }

    // Scrape per tracker in announce-priority order until everything lands.
    let mut pending: Vec<(Hash, Vec<String>)> = Vec::new(); // info-hash to the announces to try
    for url in &urls {
        if let Some((hash, announces)) = facts.get(url)
            && !pending.iter().any(|(h, _)| h == hash)
        {
            pending.push((*hash, announces.clone()));
        }
    }
    let mut tracker_order: Vec<&String> = Vec::new();
    for (_, announces) in &pending {
        for a in announces {
            if !tracker_order.contains(&a) {
                tracker_order.push(a);
            }
        }
    }

    let mut found: HashMap<Hash, Stats> = HashMap::new();
    let unknown =
        |found: &HashMap<Hash, Stats>, h: &Hash| found.get(h).is_none_or(|s| *s == (0, 0, 0));
    for announce in tracker_order.into_iter().take(MAX_TRACKERS_TRIED) {
        if Instant::now() > deadline {
            break;
        }
        let todo: Vec<Hash> = pending
            .iter()
            .filter(|(h, urls)| urls.contains(announce) && unknown(&found, h))
            .map(|(h, _)| *h)
            .collect();
        if todo.is_empty() {
            continue;
        }
        for (h, stats) in backend.scrape_tracker(announce, &todo).await {
            if unknown(&found, &h) {
                // A tracker that does not know the hash answers all zeros; prefer any later
                // tracker that knows better.
                found.insert(h, stats);
            }
        }
    }

    let now = now_secs();
    for (url, idxs) in &by_url {
        let stats = facts.get(url).and_then(|(h, _)| found.get(h)).copied();
        // Negative-cache: a dead link or silent tracker stays unknown until the TTL,
        // instead of being fetched again on every render.
        cache.put(url, now, stats);
        if let Some(stats) = stats {
            for i in idxs {
                resolved.insert(*i, stats);
            }
        }
    }
    apply(items, &resolved)
}

/// Top-N by downloads, clamped to the item count. Stable for ties. Unknown downloads
/// (feeds without stats) sort last, like -1, with seeders breaking the tie.
pub fn top_by_downloads(items: &[Release], n: usize) -> Vec<Release> {
    let mut sorted: Vec<Release> = items.to_vec();
    sorted.sort_by_key(|i| std::cmp::Reverse((i.downloads.unwrap_or(-1), i.seeders.unwrap_or(-1))));
    sorted.truncate(n);
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;
    use std::sync::Arc;

    // A tiny bencode encoder for fixtures.
    enum B<'a> {
        I(i64),
        S(&'a [u8]),
        L(Vec<B<'a>>),
        D(Vec<(&'a str, B<'a>)>),
    }

    fn benc(b: &B) -> Vec<u8> {
        match b {
            B::I(n) => format!("i{n}e").into_bytes(),
            B::S(s) => [format!("{}:", s.len()).as_bytes(), s].concat(),
            B::L(l) => [
                b"l".to_vec(),
                l.iter().flat_map(benc).collect(),
                b"e".to_vec(),
            ]
            .concat(),
            B::D(d) => [
                b"d".to_vec(),
                d.iter()
                    .flat_map(|(k, v)| [benc(&B::S(k.as_bytes())), benc(v)].concat())
                    .collect(),
                b"e".to_vec(),
            ]
            .concat(),
        }
    }

    fn info() -> B<'static> {
        B::D(vec![
            ("name", B::S(b"Show - 01 [1080p].mkv")),
            ("length", B::I(12345)),
            ("piece length", B::I(16384)),
            ("pieces", B::S(&[0u8; 20])),
        ])
    }

    fn make_torrent(announces: &[&[&str]]) -> Vec<u8> {
        benc(&B::D(vec![
            ("announce", B::S(announces[0][0].as_bytes())),
            (
                "announce-list",
                B::L(
                    announces
                        .iter()
                        .map(|tier| B::L(tier.iter().map(|u| B::S(u.as_bytes())).collect()))
                        .collect(),
                ),
            ),
            ("comment", B::S(b"test")),
            ("info", info()),
        ]))
    }

    fn default_torrent() -> Vec<u8> {
        make_torrent(&[
            &["http://t1.example/announce"],
            &["udp://t2.example:1337/announce"],
        ])
    }

    fn hex(h: &Hash) -> String {
        h.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn rel(link: &str) -> Release {
        Release {
            title: "Show - 01".to_string(),
            link: link.to_string(),
            seeders: None,
            leechers: None,
            downloads: None,
            size: String::new(),
            category: String::new(),
            page: String::new(),
            source: String::new(),
            resolution: String::new(),
            infomkv: String::new(),
            linktype: String::new(),
        }
    }

    fn counts(r: &Release) -> (Option<i64>, Option<i64>, Option<i64>) {
        (r.seeders, r.leechers, r.downloads)
    }

    // sha1 and bencode ---------------------------------------------------------------

    #[test]
    fn sha1_matches_known_vectors() {
        assert_eq!(
            hex(&sha1(b"abc")),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(hex(&sha1(b"")), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(
            hex(&sha1(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
        assert_eq!(
            hex(&sha1(&[b'a'; 1000])),
            "291e9a6c66994949b57ba5e650361e98fc36b1ba"
        );
    }

    #[test]
    fn torrent_facts_infohash_matches_sha1_of_info_dict() {
        let data = default_torrent();
        let (infohash, _) = torrent_facts(&data).unwrap();
        // sha1 of the encoded info dict, computed with Python's hashlib.
        assert_eq!(hex(&infohash), "74cec52d333abf05038b8afd98668e51d6050c5c");
        assert_eq!(info_hash(&data).unwrap(), infohash);
    }

    #[test]
    fn torrent_facts_announce_list_in_priority_order() {
        let (_, announces) = torrent_facts(&default_torrent()).unwrap();
        assert_eq!(
            announces,
            [
                "http://t1.example/announce",
                "udp://t2.example:1337/announce"
            ]
        );
    }

    #[test]
    fn torrent_facts_falls_back_to_single_announce() {
        let data = benc(&B::D(vec![
            ("announce", B::S(b"http://only.example/announce")),
            ("info", info()),
        ]));
        let (_, announces) = torrent_facts(&data).unwrap();
        assert_eq!(announces, ["http://only.example/announce"]);
    }

    #[test]
    fn torrent_facts_ignores_list_entries_that_are_not_tracker_urls() {
        let data = benc(&B::D(vec![
            ("announce", B::S(b"http://one/announce")),
            (
                "announce-list",
                B::L(vec![B::L(vec![
                    B::S(b"dht://x"),
                    B::S(b"https://two/announce"),
                ])]),
            ),
            ("info", info()),
        ]));
        assert_eq!(torrent_facts(&data).unwrap().1, ["https://two/announce"]);
    }

    #[test]
    fn torrent_facts_rejects_no_announce_and_garbage() {
        assert!(torrent_facts(&benc(&B::D(vec![("comment", B::S(b"nothing here"))]))).is_err());
        assert!(torrent_facts(b"<html>checking your browser</html>").is_err());
        assert!(torrent_facts(b"d4:infod1:xi1e").is_err());
        assert!(info_hash(b"").is_err());
    }

    // Magnet links ------------------------------------------------------------------------

    /// One info-hash written both ways a magnet may carry it.
    const HEX: &str = "c12fe1c06bba254a9dc9f519b335aa7c1367a88a";
    const B32: &str = "YEX6DQDLXISUVHOJ6UM3GNNKPQJWPKEK";

    fn facts_hash(xt: &str) -> Option<Hash> {
        magnet_facts(&format!(
            "magnet:?xt=urn:btih:{xt}&tr=http%3A%2F%2Ft.example"
        ))
        .map(|f| f.0)
    }

    #[test]
    fn magnet_facts_decodes_hex_and_base32_hashes_in_either_case() {
        let h = facts_hash(HEX).unwrap();
        assert_eq!(hex(&h), HEX);
        assert_eq!(facts_hash(&HEX.to_uppercase()), Some(h));
        assert_eq!(facts_hash(B32), Some(h));
        assert_eq!(facts_hash(&B32.to_lowercase()), Some(h));
    }

    #[test]
    fn magnet_facts_rejects_malformed_hashes() {
        assert_eq!(facts_hash(""), None);
        assert_eq!(facts_hash("abc"), None);
        assert_eq!(facts_hash(&HEX[..39]), None);
        assert_eq!(facts_hash(&format!("{HEX}0")), None);
        assert_eq!(facts_hash(&format!("{}g", &HEX[..39])), None);
        // 0, 1, 8 and 9 are not base32 letters.
        assert_eq!(facts_hash(&format!("{}1", &B32[..31])), None);
        assert_eq!(facts_hash(&format!("{}8", &B32[..31])), None);
        assert_eq!(facts_hash(&format!("{}=", &B32[..31])), None);
    }

    #[test]
    fn magnet_facts_reads_the_hash_and_decoded_trackers() {
        let link = format!(
            "magnet:?xt=urn:btih:{HEX}&dn=%5BGroup%5D+Show+-+01+%281080p%29\
             &tr=http%3A%2F%2Ft1.example%2Fannounce%3Fk%3Da+b\
             &tr=udp%3A%2F%2Ft2.example%3A1337%2Fannounce\
             &TR=https://t3.example/announce\
             &tr=wss%3A%2F%2Ft4.example\
             &tr=udp%3A%2F%2Ft2.example%3A1337%2Fannounce"
        );
        let (h, trackers) = magnet_facts(&link).unwrap();
        assert_eq!(hex(&h), HEX);
        assert_eq!(
            trackers,
            [
                "http://t1.example/announce?k=a+b",
                "udp://t2.example:1337/announce",
                "https://t3.example/announce",
            ]
        );
        let upper = format!("magnet:?xt=URN:BTIH:{B32}&tr=udp%3A%2F%2Ft2.example%3A1337");
        assert_eq!(magnet_facts(&upper).unwrap().0, h);
        // Padding a download's hash reader accepts is accepted here too.
        let padded = format!("  magnet:?xt=urn:btih:{HEX}&tr=udp%3A%2F%2Ft2.example%3A1337\n");
        assert_eq!(
            magnet_facts(&padded),
            Some((h, vec!["udp://t2.example:1337".to_string()]))
        );
    }

    #[test]
    fn magnet_facts_needs_a_good_hash_and_a_tracker() {
        assert_eq!(magnet_facts(&format!("magnet:?xt=urn:btih:{HEX}")), None);
        assert_eq!(
            magnet_facts(&format!(
                "magnet:?xt=urn:btih:{HEX}&tr=wss%3A%2F%2Ft.example"
            )),
            None
        );
        assert_eq!(
            magnet_facts("magnet:?xt=urn:btih:abc&tr=http%3A%2F%2Ft.example"),
            None
        );
        assert_eq!(
            magnet_facts(&format!(
                "magnet:?xt=urn:sha1:{HEX}&tr=http%3A%2F%2Ft.example"
            )),
            None
        );
        assert_eq!(magnet_facts("http://example.org/a.torrent"), None);
    }

    // HTTP scrape --------------------------------------------------------------------

    #[test]
    fn parse_http_scrape_reads_all_files() {
        let (h1, h2) = ([1u8; 20], [2u8; 20]);
        let body = [
            b"d5:filesd20:".as_slice(),
            &h1,
            b"d8:completei310e10:incompletei2e10:downloadedi4576ee20:",
            &h2,
            b"d8:completei0e10:incompletei0e10:downloadedi0eee",
        ]
        .concat();
        let got = parse_http_scrape(&body).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[&h1], (310, 2, 4576));
        assert_eq!(got[&h2], (0, 0, 0));
    }

    #[test]
    fn parse_http_scrape_rejects_other_bodies() {
        assert!(parse_http_scrape(b"<html>nope</html>").is_err());
        assert!(parse_http_scrape(b"d5:filesd20:short").is_err());
    }

    fn scrape_body(h: &Hash, seeders: i64, leechers: i64, done: i64) -> Vec<u8> {
        [
            b"d5:filesd20:".as_slice(),
            h,
            format!("d8:completei{seeders}e10:incompletei{leechers}e10:downloadedi{done}eee")
                .as_bytes(),
            b"e",
        ]
        .concat()
    }

    #[tokio::test]
    async fn http_scrape_replaces_announce_with_scrape() {
        let h = [1u8; 20];
        let http = FakeHttp::new();
        http.push(200, scrape_body(&h, 5, 1, 9));
        let out = http_scrape(http.as_ref(), "http://nyaa.tracker.wf:7777/announce", &[h]).await;
        let req = &http.requests()[0];
        assert!(
            req.url.starts_with("http://nyaa.tracker.wf:7777/scrape?"),
            "{}",
            req.url
        );
        assert!(
            req.url.ends_with(
                "info_hash=%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01%01"
            )
        );
        assert!(
            req.headers
                .iter()
                .any(|(k, v)| k == "User-Agent" && v == "anirss/1.0")
        );
        assert_eq!(req.timeout, Some(HTTP_SCRAPE_TIMEOUT));
        assert_eq!(out[&h], (5, 1, 9));
    }

    #[tokio::test]
    async fn http_scrape_batches_fifty_hashes_a_request() {
        let hashes: Vec<Hash> = (0..120u8).map(|i| [i; 20]).collect();
        let http = FakeHttp::new();
        for _ in 0..3 {
            http.push(200, scrape_body(&[0u8; 20], 1, 1, 1));
        }
        http_scrape(http.as_ref(), "https://t.example/x/announce", &hashes).await;
        let urls: Vec<String> = http.requests().into_iter().map(|r| r.url).collect();
        assert_eq!(urls.len(), 3);
        assert!(urls[0].starts_with("https://t.example/x/scrape?"));
        let counts: Vec<usize> = urls
            .iter()
            .map(|u| u.matches("info_hash=").count())
            .collect();
        assert_eq!(counts, [50, 50, 20]);
    }

    #[tokio::test]
    async fn http_scrape_survives_a_bad_answer_and_keeps_going() {
        let hashes: Vec<Hash> = (0..60u8).map(|i| [i; 20]).collect();
        let http = FakeHttp::new();
        http.push(503, "down");
        http.push(200, scrape_body(&hashes[55], 4, 3, 2));
        let out = http_scrape(http.as_ref(), "http://t.example/announce", &hashes).await;
        assert_eq!(out.len(), 1);
        assert_eq!(out[&hashes[55]], (4, 3, 2));
    }

    #[tokio::test]
    async fn http_scrape_skips_non_http_trackers_and_odd_paths() {
        let http = FakeHttp::new();
        assert!(
            http_scrape(
                http.as_ref(),
                "udp://tracker.example:1337/announce",
                &[[1u8; 20]]
            )
            .await
            .is_empty()
        );
        assert!(
            http_scrape(http.as_ref(), "http://tracker.example/other", &[[1u8; 20]])
                .await
                .is_empty()
        );
        assert!(http.requests().is_empty());
    }

    #[tokio::test]
    async fn a_bare_http_tracker_url_is_scraped_at_slash_scrape() {
        let http = FakeHttp::new();
        http.push(200, scrape_body(&[1u8; 20], 1, 2, 3));
        http_scrape(http.as_ref(), "http://tracker.example", &[[1u8; 20]]).await;
        assert!(
            http.requests()[0]
                .url
                .starts_with("http://tracker.example/scrape?")
        );
    }

    // UDP scrape ---------------------------------------------------------------------

    #[tokio::test]
    async fn udp_scrape_parses_seeders_completed_leechers() {
        let h = [3u8; 20];
        let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = server.local_addr().unwrap().port();
        let tracker = tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            let (n, from) = server.recv_from(&mut buf).await.unwrap();
            assert_eq!(n, 16);
            assert_eq!(u64::from_be_bytes(buf[0..8].try_into().unwrap()), UDP_MAGIC);
            assert_eq!(u32::from_be_bytes(buf[8..12].try_into().unwrap()), 0);
            let tid = &buf[12..16];
            let conn_id = 0x1111_1111_1111_1111u64;
            server
                .send_to(
                    &[&0u32.to_be_bytes()[..], tid, &conn_id.to_be_bytes()].concat(),
                    from,
                )
                .await
                .unwrap();
            let (n, from) = server.recv_from(&mut buf).await.unwrap();
            assert_eq!(n, 16 + 20);
            assert_eq!(u64::from_be_bytes(buf[0..8].try_into().unwrap()), conn_id);
            assert_eq!(u32::from_be_bytes(buf[8..12].try_into().unwrap()), 2);
            assert_eq!(&buf[16..36], &[3u8; 20]);
            // action 2, transaction id, then (seeders, completed, leechers)
            let reply = [
                2u32.to_be_bytes(),
                0u32.to_be_bytes(),
                99u32.to_be_bytes(),  // seeders
                777u32.to_be_bytes(), // completed
                11u32.to_be_bytes(),  // leechers
            ]
            .concat();
            server.send_to(&reply, from).await.unwrap();
        });
        let out = udp_scrape(&format!("udp://127.0.0.1:{port}/announce"), &[h]).await;
        tracker.await.unwrap();
        // UDP order is (seeders, completed, leechers), mapped to (seeders, leechers,
        // downloads) like the HTTP answer.
        assert_eq!(out[&h], (99, 11, 777));
    }

    #[tokio::test]
    async fn udp_scrape_ignores_a_tracker_that_refuses_the_connect() {
        let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = server.local_addr().unwrap().port();
        let tracker = tokio::spawn(async move {
            let mut buf = [0u8; 64];
            let (_, from) = server.recv_from(&mut buf).await.unwrap();
            // action 3 is an error
            server
                .send_to(&[3u32.to_be_bytes(), [0; 4], [0; 4], [0; 4]].concat(), from)
                .await
                .unwrap();
        });
        let out = udp_scrape(&format!("udp://127.0.0.1:{port}/announce"), &[[3u8; 20]]).await;
        tracker.await.unwrap();
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn udp_scrape_skips_other_schemes() {
        assert!(
            udp_scrape("http://t.example/announce", &[[1u8; 20]])
                .await
                .is_empty()
        );
    }

    // URLs ----------------------------------------------------------------------------

    #[test]
    fn encode_http_link_encodes_the_path_but_keeps_the_query() {
        assert_eq!(
            encode_http_link("https://h.example/[G] a b.torrent?token=1&x=y").unwrap(),
            "https://h.example/%5BG%5D%20a%20b.torrent?token=1&x=y"
        );
        // Already-encoded input passes through, and an IPv6 host keeps its brackets.
        assert_eq!(
            encode_http_link("http://[::1]:80/a%20b").unwrap(),
            "http://[::1]:80/a%20b"
        );
        assert!(encode_http_link("http://[::1/x").is_err());
    }

    #[tokio::test]
    async fn fetch_torrent_encodes_the_link_and_sends_the_anirss_agent() {
        let http = FakeHttp::new();
        http.push(200, b"d4:infod1:xi1eee".to_vec());
        let got = Net(http.as_ref())
            .fetch_torrent(
                "https://h.example/[G] a b.torrent?token=1&x=y",
                TORRENT_TIMEOUT,
            )
            .await;
        assert!(got.is_some());
        let req = &http.requests()[0];
        assert_eq!(
            req.url,
            "https://h.example/%5BG%5D%20a%20b.torrent?token=1&x=y"
        );
        assert!(
            req.headers
                .iter()
                .any(|(k, v)| k == "User-Agent" && v == "anirss/1.0")
        );
        assert_eq!(req.timeout, Some(TORRENT_TIMEOUT));
    }

    #[tokio::test]
    async fn fetch_torrent_gives_none_on_an_error_status() {
        let http = FakeHttp::new();
        http.push(404, "gone");
        assert!(
            Net(http.as_ref())
                .fetch_torrent("http://h/a.torrent", TORRENT_TIMEOUT)
                .await
                .is_none()
        );
    }

    // enrich, against a stand-in network ------------------------------------------------

    type ScrapeFn = Box<dyn Fn(&str, &[Hash]) -> HashMap<Hash, Stats> + Send + Sync>;

    struct Mock {
        torrents: HashMap<String, Vec<u8>>,
        /// Sleeps as long as its timeout allows, like a stalled server.
        stall: bool,
        fetched: Mutex<Vec<String>>,
        scrape: ScrapeFn,
    }

    impl Mock {
        fn new(torrents: &[(&str, Vec<u8>)]) -> Mock {
            Mock {
                torrents: torrents
                    .iter()
                    .map(|(u, d)| (u.to_string(), d.clone()))
                    .collect(),
                stall: false,
                fetched: Mutex::new(Vec::new()),
                scrape: Box::new(|_, _| HashMap::new()),
            }
        }

        fn scraping(
            mut self,
            f: impl Fn(&str, &[Hash]) -> HashMap<Hash, Stats> + Send + Sync + 'static,
        ) -> Mock {
            self.scrape = Box::new(f);
            self
        }

        fn fetched(&self) -> Vec<String> {
            self.fetched.lock().unwrap().clone()
        }
    }

    impl Backend for Mock {
        fn fetch_torrent<'a>(
            &'a self,
            url: &'a str,
            timeout: Duration,
        ) -> BoxFuture<'a, Option<Vec<u8>>> {
            Box::pin(async move {
                self.fetched.lock().unwrap().push(url.to_string());
                if self.stall {
                    tokio::time::sleep(timeout).await;
                    return None;
                }
                self.torrents.get(url).cloned()
            })
        }

        fn scrape_tracker<'a>(
            &'a self,
            announce: &'a str,
            hashes: &'a [Hash],
        ) -> BoxFuture<'a, HashMap<Hash, Stats>> {
            Box::pin(async move { (self.scrape)(announce, hashes) })
        }
    }

    const T: &str = "http://erai/t.torrent";

    fn infohash() -> Hash {
        torrent_facts(&default_torrent()).unwrap().0
    }

    async fn enrich_one(mock: &Mock, cache: &StatsCache, items: Vec<Release>) -> Vec<Release> {
        enrich_with(mock, cache, items, DEFAULT_DEADLINE).await
    }

    #[tokio::test]
    async fn enrich_passes_through_known_stats_and_unscrapable_links() {
        let asked = Arc::new(Mutex::new(Vec::new()));
        let log = asked.clone();
        let mock = Mock::new(&[]).scraping(move |announce, _| {
            log.lock().unwrap().push(announce.to_string());
            HashMap::from([(infohash(), (1, 1, 1))])
        });
        let mut known = rel(T);
        known.seeders = Some(5);
        known.leechers = Some(1);
        known.downloads = Some(9);
        let mut known_magnet = rel(&format!("magnet:?xt=urn:btih:{HEX}&tr=http%3A%2F%2Ft1"));
        known_magnet.seeders = Some(3);
        known_magnet.leechers = Some(0);
        // No `tr=`, a malformed hash, and no scheme the scrape code speaks.
        let bare_magnet = rel(&format!("magnet:?xt=urn:btih:{HEX}"));
        let bad_hash = rel("magnet:?xt=urn:btih:abc&tr=http%3A%2F%2Ft1");
        let other = rel("ftp://example.org/a.torrent");
        let items = vec![known, known_magnet, bare_magnet, bad_hash, other];
        assert_eq!(
            enrich_one(&mock, &StatsCache::default(), items.clone()).await,
            items
        );
        assert!(mock.fetched().is_empty());
        assert!(asked.lock().unwrap().is_empty());
        assert!(
            enrich_one(&mock, &StatsCache::default(), vec![])
                .await
                .is_empty()
        );
    }

    #[tokio::test]
    async fn enrich_scrapes_a_magnet_at_its_own_trackers() {
        let h = facts_hash(HEX).unwrap();
        let asked = Arc::new(Mutex::new(Vec::new()));
        let log = asked.clone();
        let mock = Mock::new(&[]).scraping(move |announce, hashes| {
            log.lock()
                .unwrap()
                .push((announce.to_string(), hashes.to_vec()));
            if announce.starts_with("udp://") {
                HashMap::from([(h, (12, 3, 40))])
            } else {
                HashMap::new()
            }
        });
        let link = format!(
            "magnet:?xt=urn:btih:{}&dn=Show&tr=http%3A%2F%2Ft1.example%2Fannounce\
             &tr=udp%3A%2F%2Ft2.example%3A1337%2Fannounce",
            B32.to_lowercase()
        );
        let cache = StatsCache::default();
        let out = enrich_one(&mock, &cache, vec![rel(&link)]).await;
        assert_eq!(counts(&out[0]), (Some(12), Some(3), Some(40)));
        assert!(mock.fetched().is_empty(), "a magnet needs no download");
        assert_eq!(
            *asked.lock().unwrap(),
            [
                ("http://t1.example/announce".to_string(), vec![h]),
                ("udp://t2.example:1337/announce".to_string(), vec![h]),
            ]
        );
        // Cached by link like a .torrent URL.
        let again = Mock::new(&[]);
        let out = enrich_one(&again, &cache, vec![rel(&link)]).await;
        assert_eq!(counts(&out[0]), (Some(12), Some(3), Some(40)));
    }

    #[tokio::test]
    async fn enrich_fills_stats_from_scrape() {
        let h = infohash();
        let mock = Mock::new(&[(T, default_torrent())])
            .scraping(move |_, _| HashMap::from([(h, (310, 2, 4576))]));
        let out = enrich_one(&mock, &StatsCache::default(), vec![rel(T)]).await;
        assert_eq!(counts(&out[0]), (Some(310), Some(2), Some(4576)));
    }

    #[tokio::test]
    async fn enrich_failure_leaves_unknown_and_negative_caches() {
        let cache = StatsCache::default();
        let items = vec![rel("http://erai/broken.torrent")];
        let mock = Mock::new(&[]);
        assert_eq!(
            counts(&enrich_one(&mock, &cache, items.clone()).await[0]),
            (None, None, None)
        );
        assert_eq!(mock.fetched().len(), 1);
        // The second run must not even try to download.
        let again = Mock::new(&[]);
        assert_eq!(
            counts(&enrich_one(&again, &cache, items).await[0]),
            (None, None, None)
        );
        assert!(again.fetched().is_empty());
    }

    #[tokio::test]
    async fn enrich_prefers_nonzero_tracker_results() {
        // A tracker that does not know the hash answers zeros; a later tracker that knows
        // better wins.
        let h = infohash();
        let mock = Mock::new(&[(T, default_torrent())]).scraping(move |announce, _| {
            if announce.contains("t1") {
                HashMap::from([(h, (0, 0, 0))])
            } else {
                HashMap::from([(h, (7, 1, 42))])
            }
        });
        let out = enrich_one(&mock, &StatsCache::default(), vec![rel(T)]).await;
        assert_eq!(counts(&out[0]), (Some(7), Some(1), Some(42)));
    }

    #[tokio::test]
    async fn enrich_stops_asking_trackers_once_the_hash_is_known() {
        let h = infohash();
        let asked = Arc::new(Mutex::new(Vec::new()));
        let log = asked.clone();
        let mock = Mock::new(&[(T, default_torrent())]).scraping(move |announce, _| {
            log.lock().unwrap().push(announce.to_string());
            HashMap::from([(h, (1, 1, 1))])
        });
        enrich_one(&mock, &StatsCache::default(), vec![rel(T)]).await;
        assert_eq!(*asked.lock().unwrap(), ["http://t1.example/announce"]);
    }

    #[tokio::test]
    async fn enrich_asks_at_most_four_trackers() {
        let data = make_torrent(&[
            &[
                "http://a/announce",
                "http://b/announce",
                "http://c/announce",
            ],
            &["http://d/announce", "http://e/announce"],
        ]);
        let asked = Arc::new(Mutex::new(Vec::new()));
        let log = asked.clone();
        let mock = Mock::new(&[(T, data)]).scraping(move |announce, _| {
            log.lock().unwrap().push(announce.to_string());
            HashMap::new()
        });
        enrich_one(&mock, &StatsCache::default(), vec![rel(T)]).await;
        assert_eq!(asked.lock().unwrap().len(), MAX_TRACKERS_TRIED);
    }

    #[tokio::test]
    async fn enrich_uses_cache_within_ttl_and_refetches_after_it() {
        let h = infohash();
        let cache = StatsCache::default();
        let make = || {
            Mock::new(&[(T, default_torrent())])
                .scraping(move |_, _| HashMap::from([(h, (1, 2, 3))]))
        };
        let first = make();
        enrich_one(&first, &cache, vec![rel(T)]).await;
        let second = make();
        let out = enrich_one(&second, &cache, vec![rel(T)]).await;
        assert_eq!(first.fetched(), [T]);
        assert!(second.fetched().is_empty());
        assert_eq!(counts(&out[0]), (Some(1), Some(2), Some(3)));
        // An entry older than the TTL counts as a miss.
        cache.put(
            T,
            now_secs() - CACHE_TTL.as_secs_f64() - 1.0,
            Some((1, 2, 3)),
        );
        let third = make();
        enrich_one(&third, &cache, vec![rel(T)]).await;
        assert_eq!(third.fetched(), [T]);
    }

    #[tokio::test]
    async fn enrich_fetches_a_shared_link_once_and_fills_every_item() {
        let h = infohash();
        let mock = Mock::new(&[(T, default_torrent())])
            .scraping(move |_, _| HashMap::from([(h, (4, 5, 6))]));
        let out = enrich_one(&mock, &StatsCache::default(), vec![rel(T), rel(T)]).await;
        assert_eq!(mock.fetched().len(), 1);
        assert!(out.iter().all(|r| counts(r) == (Some(4), Some(5), Some(6))));
    }

    #[tokio::test]
    async fn enrich_respects_deadline() {
        // A stalled server: the fetch sleeps exactly as long as its timeout allows, so the
        // deadline must bound the total.
        let mut mock = Mock::new(&[]);
        mock.stall = true;
        let items: Vec<Release> = (0..9)
            .map(|i| rel(&format!("http://erai/{i}.torrent")))
            .collect();
        let started = Instant::now();
        enrich_with(&mock, &StatsCache::default(), items, Duration::from_secs(1)).await;
        assert!(
            started.elapsed() < Duration::from_millis(3500),
            "{:?}",
            started.elapsed()
        );
    }

    #[tokio::test]
    async fn fetch_torrents_keeps_only_real_torrents() {
        let good = default_torrent();
        let mock = Mock::new(&[
            ("http://h/a", good.clone()),
            ("http://h/c", b"<html>checking your browser</html>".to_vec()),
        ]);
        let urls: Vec<String> = ["http://h/a", "http://h/b", "http://h/c", "http://h/a"]
            .map(String::from)
            .to_vec();
        let got = fetch_torrents_with(&mock, &urls, Duration::from_secs(5)).await;
        assert_eq!(got, HashMap::from([("http://h/a".to_string(), good)]));
        let mut calls = mock.fetched();
        calls.sort();
        assert_eq!(calls, ["http://h/a", "http://h/b", "http://h/c"]);
    }

    #[tokio::test]
    async fn fetch_torrents_runs_at_most_eight_at_a_time() {
        struct Gauge {
            now: Mutex<(usize, usize)>,
        }
        impl Backend for Gauge {
            fn fetch_torrent<'a>(
                &'a self,
                _: &'a str,
                _: Duration,
            ) -> BoxFuture<'a, Option<Vec<u8>>> {
                Box::pin(async move {
                    {
                        let mut g = self.now.lock().unwrap();
                        g.0 += 1;
                        g.1 = g.1.max(g.0);
                    }
                    tokio::time::sleep(Duration::from_millis(30)).await;
                    self.now.lock().unwrap().0 -= 1;
                    Some(default_torrent())
                })
            }
            fn scrape_tracker<'a>(
                &'a self,
                _: &'a str,
                _: &'a [Hash],
            ) -> BoxFuture<'a, HashMap<Hash, Stats>> {
                Box::pin(async { HashMap::new() })
            }
        }
        let gauge = Gauge {
            now: Mutex::new((0, 0)),
        };
        let urls: Vec<String> = (0..20).map(|i| format!("http://h/{i}")).collect();
        let got = fetch_torrents_with(&gauge, &urls, Duration::from_secs(5)).await;
        assert_eq!(got.len(), 20);
        assert_eq!(gauge.now.lock().unwrap().1, DOWNLOAD_WORKERS);
    }

    #[test]
    fn top_by_downloads_sorts_unknown_last_and_clamps() {
        let mk = |link: &str, d: Option<i64>, s: Option<i64>| {
            let mut r = rel(link);
            r.downloads = d;
            r.seeders = s;
            r
        };
        let items = vec![
            mk("none", None, None),
            mk("few", Some(5), Some(1)),
            mk("many", Some(50), Some(1)),
            mk("tie", Some(5), Some(9)),
            mk("noseed", None, Some(3)),
        ];
        let top: Vec<String> = top_by_downloads(&items, 4)
            .into_iter()
            .map(|r| r.link)
            .collect();
        assert_eq!(top, ["many", "tie", "few", "noseed"]);
        assert_eq!(top_by_downloads(&items, 99).len(), 5);
    }
}
