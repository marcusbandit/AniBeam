//! The connection check: what stands between the phone and each source, one layer at a
//! time, so a person (or whoever helps them) sees where a search stops. For each host: the
//! network's own DNS, DNS over HTTPS, a plain TCP connection to each address, and a whole
//! HTTPS request. The hosts are the ones this phone's searches go to; a site no network
//! blocks is checked the same way, as the yardstick.

use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use crate::Downloads;
use anibeam_contract::i18n::tr;
use anibeam_contract::*;
use anibeam_foundation::services::Services;
use anibeam_net::HttpRequest;

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
fn searched_hosts(services: &Services) -> Vec<(String, String, u16)> {
    let mut out: Vec<(String, String, u16)> = Vec::new();
    // The runnable addresses, not the shown ones: a torznab source's `{host}` is filled there.
    for (_, e) in super::searched(services) {
        let host = anibeam_releases::repos::index::host_of(&e.url);
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
        out.push((
            host.clone(),
            format!("{scheme}://{}/", server_of(&e.url)),
            port,
        ));
    }
    out
}

/// The host and port of `url`, without any login in front of them.
fn server_of(url: &str) -> &str {
    let authority = anibeam_releases::repos::index::authority_of(url);
    authority.rsplit_once('@').map_or(authority, |(_, a)| a)
}

/// The port a connection to `url` goes to: the one in the address, else the scheme's own.
fn port_of(url: &str) -> u16 {
    // A ':' inside an IPv6 literal's brackets is not a port.
    server_of(url)
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

fn shown(ips: &[IpAddr]) -> String {
    ips.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

async fn tcp(ip: IpAddr, port: u16) -> (bool, String, u64) {
    let start = Instant::now();
    match tokio::time::timeout(
        STEP,
        tokio::net::TcpStream::connect(SocketAddr::new(ip, port)),
    )
    .await
    {
        Ok(Ok(_)) => (true, tr("core-phone-check-connected", &[]), ms(start)),
        Ok(Err(e)) => (
            false,
            tr(
                "core-phone-check-refused",
                &[("error", e.to_string().into())],
            ),
            ms(start),
        ),
        Err(_) => (false, tr("core-phone-check-dropped", &[]), ms(start)),
    }
}

async fn host(
    services: &Services,
    doh: &anibeam_net::doh::Doh,
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
    let (ok, detail) = match system {
        Ok(Ok(addrs)) => {
            let found: Vec<IpAddr> = addrs.map(|a| a.ip()).collect();
            let answer = (!found.is_empty(), shown(&found));
            ips.extend(found);
            answer
        }
        Ok(Err(e)) => (
            false,
            tr(
                "core-phone-check-failed",
                &[("error", e.to_string().into())],
            ),
        ),
        Err(_) => (false, tr("core-phone-check-no-answer", &[])),
    };
    let finding = tr(
        "core-phone-check-finding",
        &[("label", label.as_str().into())],
    );
    out.push(line(finding, ok, detail, ms(start)));

    let doh_line = tr(
        "core-phone-check-private",
        &[("label", label.as_str().into())],
    );
    let skipped = if name.parse::<IpAddr>().is_ok() {
        Some(tr("core-phone-check-skipped-address", &[]))
    } else if anibeam_net::doh::is_home_name(name) {
        Some(tr("core-phone-check-skipped-home", &[]))
    } else {
        None
    };
    if let Some(why) = skipped {
        out.push(line(doh_line, true, why, 0));
    } else {
        let start = Instant::now();
        let (ok, detail) = match doh.find(name).await {
            anibeam_net::doh::Found::Addresses(found) => {
                let detail = shown(&found);
                for ip in found {
                    if !ips.contains(&ip) {
                        ips.push(ip);
                    }
                }
                (true, detail)
            }
            // The resolvers answered: the name has no public address, and this network's
            // DNS is the one to ask.
            anibeam_net::doh::Found::NoRecord => (true, tr("core-phone-check-no-record", &[])),
            anibeam_net::doh::Found::NoAnswer => (false, tr("core-phone-check-no-resolver", &[])),
        };
        out.push(line(doh_line, ok, detail, ms(start)));
    }

    for ip in ips.iter().filter(|ip| ip.is_ipv4()).take(3) {
        let (ok, detail, t) = tcp(*ip, port).await;
        let step = tr(
            "core-phone-check-reaching",
            &[
                ("label", label.as_str().into()),
                ("ip", ip.to_string().into()),
            ],
        );
        out.push(line(step, ok, detail, t));
    }

    let start = Instant::now();
    let answer = services
        .http
        .send(
            HttpRequest::get(url)
                .header("User-Agent", anibeam_releases::fetch::USER_AGENT)
                // Cloudflare's resolver refuses anything but exactly its own type.
                .header(
                    "Accept",
                    if url.contains("dns-query") {
                        "application/dns-json"
                    } else {
                        "*/*"
                    },
                )
                .timeout(STEP * 2),
        )
        .await;
    let (ok, detail) = match answer {
        Ok(r) => (
            r.is_success(),
            tr(
                "core-phone-check-http",
                &[
                    ("status", r.status.to_string().into()),
                    ("bytes", r.body.len().into()),
                ],
            ),
        ),
        Err(e) => (false, e.message),
    };
    out.push(line(format!("{label}: {fetch}"), ok, detail, ms(start)));
    out
}

/// `PhoneNetCheck`: [`run`], waited for on the core's runtime.
pub fn run_call(downloads: &std::sync::Arc<Downloads>) -> Result<Reply, CoreError> {
    Ok(Reply::PhoneNetCheck {
        checks: downloads.services.handle.block_on(run(downloads)),
    })
}

/// Every check, hosts side by side so the whole run takes about as long as the slowest.
pub async fn run(downloads: &std::sync::Arc<Downloads>) -> Vec<NetCheck> {
    let client = match anibeam_net::resolver_client() {
        Ok(c) => c,
        Err(e) => {
            return vec![line(
                tr("core-phone-check-setup", &[]),
                false,
                e.to_string(),
                0,
            )];
        }
    };
    let doh = anibeam_net::doh::Doh::new(client);
    let owner = downloads.clone();
    let mut hosts = searched_hosts(&downloads.services);
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
            downloads
                .services
                .handle
                .spawn(async move { host(&owner.services, &doh, &name, &url, port).await })
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
