//! Where the host listens: the addresses Tailscale gave this machine, picked
//! out of everything its interfaces hold, and the URLs they make.

use std::net::{IpAddr, SocketAddr};

use anibeam_host_api::DEFAULT_PORT;

/// The addresses Tailscale hands out: IPv4 in the carrier-grade NAT block
/// 100.64.0.0/10, IPv6 in the unique-local prefix fd7a:115c:a1e0::/48.
pub fn is_tailscale(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            a == 100 && (64..=127).contains(&b)
        }
        IpAddr::V6(v6) => v6.segments()[..3] == [0xfd7a, 0x115c, 0xa1e0],
    }
}

/// The Tailscale addresses among `all`: IPv4 first, each once, in order.
/// Pure, so the choice is tested without a network.
pub fn pick(all: &[IpAddr]) -> Vec<IpAddr> {
    let mut mine: Vec<IpAddr> = all.iter().copied().filter(is_tailscale).collect();
    mine.sort_by_key(|ip| (ip.is_ipv6(), *ip));
    mine.dedup();
    mine
}

/// Every address on this machine's interfaces. A machine whose interfaces
/// cannot be listed has none to offer.
pub fn local_addresses() -> Vec<IpAddr> {
    if_addrs::get_if_addrs()
        .map(|all| all.into_iter().map(|i| i.ip()).collect())
        .unwrap_or_default()
}

/// The sockets to listen on: the override when there is one
/// (`ANIBEAM_HOST_BIND`, an `ip:port`), else each Tailscale address on the
/// default port. Empty when Tailscale is not up. Err is the override's own
/// text when it is not an address.
pub fn wanted(bind_override: Option<&str>, all: &[IpAddr]) -> Result<Vec<SocketAddr>, String> {
    match bind_override.map(str::trim).filter(|s| !s.is_empty()) {
        Some(text) => text
            .parse::<SocketAddr>()
            .map(|a| vec![a])
            .map_err(|_| format!("{text} is not an address and port")),
        None => Ok(pick(all)
            .into_iter()
            .map(|ip| SocketAddr::new(ip, DEFAULT_PORT))
            .collect()),
    }
}

/// `http://100.65.105.9:8790`, an IPv6 address in brackets.
pub fn url(addr: &SocketAddr) -> String {
    format!("http://{addr}")
}

/// `http://banditbox.tail24c6a9.ts.net:8790`.
pub fn name_url(dns_name: &str, port: u16) -> String {
    format!("http://{}:{port}", dns_name.trim_end_matches('.'))
}

/// The address as a person reads it in "Serving on ...": the port only when
/// it is not the usual one.
pub fn shown(addr: &SocketAddr) -> String {
    if addr.port() == DEFAULT_PORT {
        addr.ip().to_string()
    } else {
        addr.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn the_cgnat_block_is_tailscale_and_nothing_next_to_it() {
        assert!(is_tailscale(&ip("100.64.0.0")));
        assert!(is_tailscale(&ip("100.65.105.9")));
        assert!(is_tailscale(&ip("100.127.255.255")));
        assert!(!is_tailscale(&ip("100.63.255.255")));
        assert!(!is_tailscale(&ip("100.128.0.0")));
        assert!(!is_tailscale(&ip("101.65.0.1")));
        assert!(!is_tailscale(&ip("192.168.1.20")));
        assert!(!is_tailscale(&ip("127.0.0.1")));
    }

    #[test]
    fn the_tailscale_ipv6_prefix_is_exactly_the_48() {
        assert!(is_tailscale(&ip("fd7a:115c:a1e0::1")));
        assert!(is_tailscale(&ip("fd7a:115c:a1e0:ab12:1234::5")));
        assert!(!is_tailscale(&ip("fd7a:115c:a1e1::1")));
        assert!(!is_tailscale(&ip("fd7a:115d:a1e0::1")));
        assert!(!is_tailscale(&ip("fe80::1")));
        assert!(!is_tailscale(&ip("::1")));
    }

    #[test]
    fn pick_keeps_only_tailscale_addresses_ipv4_first_each_once() {
        let all = [
            ip("127.0.0.1"),
            ip("fd7a:115c:a1e0::5"),
            ip("192.168.1.20"),
            ip("100.65.105.9"),
            ip("fe80::1"),
            ip("100.65.105.9"),
            ip("100.100.1.1"),
        ];
        assert_eq!(
            pick(&all),
            vec![
                ip("100.65.105.9"),
                ip("100.100.1.1"),
                ip("fd7a:115c:a1e0::5")
            ]
        );
        assert!(pick(&[ip("10.0.0.2"), ip("::1")]).is_empty());
        assert!(pick(&[]).is_empty());
    }

    #[test]
    fn wanted_listens_on_the_default_port_or_where_it_is_told() {
        let all = [ip("100.65.105.9"), ip("192.168.1.20")];
        assert_eq!(
            wanted(None, &all).unwrap(),
            vec!["100.65.105.9:8790".parse().unwrap()]
        );
        assert_eq!(wanted(Some(""), &all).unwrap().len(), 1);
        assert!(wanted(None, &[ip("10.0.0.2")]).unwrap().is_empty());
        // The override ignores the interfaces altogether.
        assert_eq!(
            wanted(Some(" 127.0.0.1:0 "), &[]).unwrap(),
            vec!["127.0.0.1:0".parse().unwrap()]
        );
        assert_eq!(
            wanted(Some("0.0.0.0:8790"), &all).unwrap(),
            vec!["0.0.0.0:8790".parse().unwrap()]
        );
        assert!(wanted(Some("not an address"), &all).is_err());
    }

    #[test]
    fn urls_bracket_ipv6_and_trim_the_dns_dot() {
        assert_eq!(
            url(&"100.65.105.9:8790".parse().unwrap()),
            "http://100.65.105.9:8790"
        );
        assert_eq!(
            url(&"[fd7a:115c:a1e0::5]:8790".parse().unwrap()),
            "http://[fd7a:115c:a1e0::5]:8790"
        );
        assert_eq!(
            name_url("banditbox.tail24c6a9.ts.net.", 8790),
            "http://banditbox.tail24c6a9.ts.net:8790"
        );
        assert_eq!(shown(&"100.65.105.9:8790".parse().unwrap()), "100.65.105.9");
        assert_eq!(
            shown(&"127.0.0.1:41000".parse().unwrap()),
            "127.0.0.1:41000"
        );
    }
}
