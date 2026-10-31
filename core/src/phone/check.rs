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

/// The hosts a search on this phone goes to, one each, with the site's front page to fetch
/// and the port its searches connect to.
fn searched_hosts(core: &Core) -> Vec<(String, String, u16)> {
    let mut out: Vec<(String, String, u16)> = Vec::new();
    // The runnable addresses, not the shown ones: a torznab source's `{host}` is filled there.
    for (_, e) in super::searched(core) {
        let host = crate::repos::index::host_of(&e.url);
        let port = port_of(&e.url);
        // By host and port: a Prowlarr and a Jackett on one machine are two servers.
        if host.is_empty() || out.iter().any(|(h, _, p)| *h == host && *p == port) {
            continue;
        }
        let scheme = if e.url.starts_with("http://") {
            "http"
        } else {
            "https"
        };
        // The port stays (a Prowlarr listens on 9696); a login in the address never goes out.
        let authority = crate::repos::index::authority_of(&e.url);
        let authority = authority.rsplit_once('@').map_or(authority, |(_, a)| a);
        out.push((host.clone(), format!("{scheme}://{authority}/"), port));
    }
    out
}

/// The port a connection to `url` goes to: the one in the address, else the scheme's own.
fn port_of(url: &str) -> u16 {
    let authority = crate::repos::index::authority_of(url);
    let authority = authority.rsplit_once('@').map_or(authority, |(_, a)| a);
    // A ':' inside an IPv6 literal's brackets is not a port.
    authority
        .rsplit_once(':')
        .filter(|(_, port)| !port.contains(']'))
        .and_then(|(_, port)| port.parse().ok())
        .unwrap_or(if url.starts_with("http://") { 80 } else { 443 })
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

async fn tcp(ip: IpAddr, port: u16) -> (bool, String, u64) {
    let start = Instant::now();
    match tokio::time::timeout(
        STEP,
        tokio::net::TcpStream::connect(SocketAddr::new(ip, port)),
    )
    .await
    {
        Ok(Ok(_)) => (true, "connected".into(), ms(start)),
        Ok(Err(e)) => (false, format!("refused: {e}"), ms(start)),
        Err(_) => (false, "no answer (dropped)".into(), ms(start)),
    }
}

async fn host(
    core: &Core,
    doh: &crate::net::doh::Doh,
    name: &str,
    url: &str,
    port: u16,
) -> Vec<NetCheck> {
    let mut out = Vec::new();
    let mut ips: Vec<IpAddr> = Vec::new();
    // Named with the port when it is not the scheme's own, so two servers on one machine read
    // apart, and the fetch says HTTP when that is what it is.
    let plain = url.starts_with("http://");
    let label = if port == if plain { 80 } else { 443 } {
        name.to_string()
    } else {
        format!("{name}:{port}")
    };
    let fetch = if plain { "HTTP" } else { "HTTPS" };

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
                format!("{label}: network DNS"),
                !found.is_empty(),
                shown,
                ms(start),
            ));
            ips.extend(found);
        }
        Ok(Err(e)) => out.push(line(
            format!("{label}: network DNS"),
            false,
            format!("failed: {e}"),
            ms(start),
        )),
        Err(_) => out.push(line(
            format!("{label}: network DNS"),
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
                format!("{label}: DNS over HTTPS"),
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
            format!("{label}: DNS over HTTPS"),
            false,
            "no resolver answered".into(),
            ms(start),
        )),
    }

    for ip in ips.iter().filter(|ip| ip.is_ipv4()).take(3) {
        let (ok, detail, t) = tcp(*ip, port).await;
        out.push(line(format!("{label}: connect {ip}"), ok, detail, t));
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
            format!("{label}: {fetch}"),
            r.is_success(),
            format!("HTTP {}, {} bytes", r.status, r.body.len()),
            ms(start),
        )),
        Err(e) => out.push(line(
            format!("{label}: {fetch}"),
            false,
            e.message,
            ms(start),
        )),
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
    hosts.extend(
        ALWAYS
            .iter()
            .map(|(h, u)| (h.to_string(), u.to_string(), port_of(u))),
    );
    // All at once, as before: a dropped host costs one timeout, not one each.
    let tasks: Vec<_> = hosts
        .into_iter()
        .map(|(name, url, port)| {
            let (owner, doh) = (owner.clone(), doh.clone());
            core.handle
                .spawn(async move { host(&owner, &doh, &name, &url, port).await })
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
    use super::*;

    #[test]
    fn the_connect_step_takes_the_address_port() {
        assert_eq!(port_of("https://example.org/?q={query}"), 443);
        assert_eq!(port_of("http://jackett.lan/api?q={query}"), 80);
        assert_eq!(port_of("http://192.168.1.10:9696/1/api?q={query}"), 9696);
        assert_eq!(port_of("https://me:pw@idx.example:8443/api"), 8443);
        assert_eq!(port_of("http://[::1]:9117/api"), 9117);
        assert_eq!(port_of("https://[::1]/api"), 443);
    }
}
