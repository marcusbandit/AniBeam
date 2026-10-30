//! The connection check: what stands between the phone and each source, one layer at a
//! time, so a person (or whoever helps them) sees where a search stops. For each host: the
//! network's own DNS, DNS over HTTPS, a plain TCP connection to each address, and a whole
//! HTTPS request. The hosts are the ones this phone's searches go to; a site no network
//! blocks is checked the same way, as the yardstick.

use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use crate::contract::*;
use crate::core::Core;
use crate::net::{HttpRequest, Method};

const STEP: Duration = Duration::from_secs(6);

/// Checked beside the sources: the DNS-over-HTTPS resolver, and a site no network blocks as
/// the yardstick.
const ALWAYS: [(&str, &str); 2] = [
    (
        "one.one.one.one",
        "https://1.1.1.1/dns-query?name=example.com&type=A",
    ),
    ("example.com", "https://example.com/"),
];

/// The hosts a search on this phone goes to, one each, with the site's front page to fetch.
fn searched_hosts(core: &Core) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for s in super::search_sources(core) {
        let host = crate::repos::index::host_of(&s.url);
        if host.is_empty() || out.iter().any(|(h, _)| *h == host) {
            continue;
        }
        let scheme = if s.url.starts_with("http://") {
            "http"
        } else {
            "https"
        };
        out.push((host.clone(), format!("{scheme}://{host}/")));
    }
    out
}

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

/// A name or address that only means something on this network: an IP literal, a private,
/// loopback or link-local address, a single-label name, or a home-network suffix. Public
/// resolvers have no answer for these, and asking one sends the name off the network.
fn is_private(name: &str) -> bool {
    let name = name.trim_end_matches('.').to_lowercase();
    let bare = name.trim_start_matches('[').trim_end_matches(']');
    if bare.parse::<IpAddr>().is_ok() {
        return true;
    }
    !name.contains('.')
        || [".lan", ".local", ".home.arpa", ".internal", ".localhost"]
            .iter()
            .any(|suffix| name.ends_with(suffix))
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
    if is_private(name) {
        out.push(line(
            format!("{name}: DNS over HTTPS"),
            true,
            "skipped, private host".into(),
            0,
        ));
    } else {
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
    let Ok(owner) = core.owner() else {
        return Vec::new();
    };
    let mut hosts = searched_hosts(core);
    hosts.extend(ALWAYS.iter().map(|(h, u)| (h.to_string(), u.to_string())));
    // All at once, as before: a dropped host costs one timeout, not one each.
    let tasks: Vec<_> = hosts
        .into_iter()
        .map(|(name, url)| {
            let (owner, doh) = (owner.clone(), doh.clone());
            core.handle
                .spawn(async move { host(&owner, &doh, &name, &url).await })
        })
        .collect();
    let mut out = Vec::new();
    for t in tasks {
        out.extend(t.await.unwrap_or_default());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::is_private;

    #[test]
    fn home_network_names_and_addresses_are_private() {
        for name in [
            "prowlarr.lan",
            "jackett.local",
            "nas.home.arpa",
            "box.internal",
            "prowlarr",
            "localhost",
            "192.168.1.20",
            "10.0.0.5",
            "172.16.4.1",
            "169.254.3.3",
            "127.0.0.1",
            "[fd00::1]",
            "fe80::1",
            "NAS.LAN.",
        ] {
            assert!(is_private(name), "{name} should be private");
        }
    }

    #[test]
    fn public_names_are_not_private() {
        for name in [
            "nyaa.si",
            "example.com",
            "one.one.one.one",
            "lan.example.com",
        ] {
            assert!(!is_private(name), "{name} should be public");
        }
    }
}
