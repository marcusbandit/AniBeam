//! The connection check: what stands between the phone and each source, one layer at a
//! time, so a person (or whoever helps them) sees where a search stops. For each host: the
//! network's own DNS, DNS over HTTPS, a plain TCP connection to each address, and a whole
//! HTTPS request. A site no network blocks is checked the same way, as the yardstick.

use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use crate::contract::*;
use crate::core::Core;
use crate::net::{HttpRequest, Method};

const STEP: Duration = Duration::from_secs(6);

/// The hosts checked: the sources, the DNS-over-HTTPS resolvers, and the yardstick.
const HOSTS: [(&str, &str); 5] = [
    ("nyaa.si", "https://nyaa.si/?page=rss&q=frieren"),
    (
        "animetosho.org",
        "https://animetosho.org/feed/json?q=frieren",
    ),
    ("www.erai-raws.info", "https://www.erai-raws.info/"),
    (
        "one.one.one.one",
        "https://1.1.1.1/dns-query?name=nyaa.si&type=A",
    ),
    ("example.com", "https://example.com/"),
];

fn ms(since: Instant) -> u64 {
    since.elapsed().as_millis() as u64
}

fn line(name: String, ok: bool, detail: String, ms: u64) -> NetCheck {
    NetCheck {
        name,
        ok,
        detail,
        ms,
    }
}

async fn tcp(ip: IpAddr) -> (bool, String, u64) {
    let start = Instant::now();
    match tokio::time::timeout(
        STEP,
        tokio::net::TcpStream::connect(SocketAddr::new(ip, 443)),
    )
    .await
    {
        Ok(Ok(_)) => (true, "connected".into(), ms(start)),
        Ok(Err(e)) => (false, format!("refused: {e}"), ms(start)),
        Err(_) => (false, "no answer (dropped)".into(), ms(start)),
    }
}

async fn host(core: &Core, doh: &crate::net::doh::Doh, name: &str, url: &str) -> Vec<NetCheck> {
    let mut out = Vec::new();
    let mut ips: Vec<IpAddr> = Vec::new();

    let start = Instant::now();
    let system = tokio::time::timeout(STEP, tokio::net::lookup_host((name, 443))).await;
    match system {
        Ok(Ok(addrs)) => {
            let found: Vec<IpAddr> = addrs.map(|a| a.ip()).collect();
            let shown = found
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            out.push(line(
                format!("{name}: network DNS"),
                !found.is_empty(),
                shown,
                ms(start),
            ));
            ips.extend(found);
        }
        Ok(Err(e)) => out.push(line(
            format!("{name}: network DNS"),
            false,
            format!("failed: {e}"),
            ms(start),
        )),
        Err(_) => out.push(line(
            format!("{name}: network DNS"),
            false,
            "no answer".into(),
            ms(start),
        )),
    }

    let start = Instant::now();
    match doh.lookup(name).await {
        Some(found) => {
            let shown = found
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            out.push(line(
                format!("{name}: DNS over HTTPS"),
                true,
                shown,
                ms(start),
            ));
            for ip in found {
                if !ips.contains(&ip) {
                    ips.push(ip);
                }
            }
        }
        None => out.push(line(
            format!("{name}: DNS over HTTPS"),
            false,
            "no resolver answered".into(),
            ms(start),
        )),
    }

    for ip in ips.iter().filter(|ip| ip.is_ipv4()).take(3) {
        let (ok, detail, t) = tcp(*ip).await;
        out.push(line(format!("{name}: connect {ip}"), ok, detail, t));
    }

    let start = Instant::now();
    let answer = core
        .http
        .send(HttpRequest {
            method: Method::Get,
            url: url.to_string(),
            headers: vec![
                (
                    "User-Agent".into(),
                    crate::releases::fetch::USER_AGENT.into(),
                ),
                // Cloudflare's resolver refuses anything but exactly its own type.
                (
                    "Accept".into(),
                    if url.contains("dns-query") {
                        "application/dns-json"
                    } else {
                        "*/*"
                    }
                    .into(),
                ),
            ],
            body: None,
            timeout: Some(STEP * 2),
        })
        .await;
    match answer {
        Ok(r) => out.push(line(
            format!("{name}: HTTPS"),
            r.is_success(),
            format!("HTTP {}, {} bytes", r.status, r.body.len()),
            ms(start),
        )),
        Err(e) => out.push(line(format!("{name}: HTTPS"), false, e.message, ms(start))),
    }
    out
}

/// Every check, hosts side by side so the whole run takes about as long as the slowest.
pub async fn run(core: &Core) -> Vec<NetCheck> {
    let client = match crate::net::resolver_client() {
        Ok(c) => c,
        Err(e) => return vec![line("setup".into(), false, e.to_string(), 0)],
    };
    let doh = crate::net::doh::Doh::new(client);
    let [a, b, c, d, e] = HOSTS;
    let (a, b, c, d, e) = tokio::join!(
        host(core, &doh, a.0, a.1),
        host(core, &doh, b.0, b.1),
        host(core, &doh, c.0, c.1),
        host(core, &doh, d.0, d.1),
        host(core, &doh, e.0, e.1),
    );
    [a, b, c, d, e].concat()
}
