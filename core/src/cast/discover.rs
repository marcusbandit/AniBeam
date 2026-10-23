//! Finding devices: Chromecasts answer mDNS for `_googlecast._tcp`, DLNA renderers answer an
//! SSDP search for AVTransport. Each look lasts a few seconds and reports what it finds as
//! it goes.

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use super::dlna;
use crate::contract::{CastDevice, CastKind};

/// A found device with what it takes to talk to it.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub device: CastDevice,
    /// The Chromecast's control port.
    pub addr: SocketAddr,
    /// The renderer's AVTransport control URL.
    pub control: Option<String>,
}

/// Looks for Chromecasts for `wait`, calling `found` for each.
pub fn chromecasts(wait: Duration, found: impl Fn(Found)) -> Result<(), String> {
    let daemon = mdns_sd::ServiceDaemon::new().map_err(|e| format!("mDNS would not start: {e}"))?;
    let rx = daemon
        .browse("_googlecast._tcp.local.")
        .map_err(|e| format!("mDNS would not browse: {e}"))?;
    let until = Instant::now() + wait;
    while let Some(left) = until.checked_duration_since(Instant::now()) {
        let Ok(event) = rx.recv_timeout(left) else {
            break;
        };
        if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
            let Some(ip) = info.get_addresses_v4().into_iter().next() else {
                continue;
            };
            let name = info
                .get_property_val_str("fn")
                .map(str::to_string)
                .unwrap_or_else(|| info.get_fullname().to_string());
            let id = info
                .get_property_val_str("id")
                .map(str::to_string)
                .unwrap_or_else(|| info.get_fullname().to_string());
            let addr = SocketAddr::new(IpAddr::V4(ip), info.get_port());
            found(Found {
                device: CastDevice {
                    id: format!("cast:{id}"),
                    name,
                    kind: CastKind::Chromecast,
                    model: info.get_property_val_str("md").map(str::to_string),
                    address: addr.to_string(),
                },
                addr,
                control: None,
            });
        }
    }
    let _ = daemon.shutdown();
    Ok(())
}

const SEARCH: &str = "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\nST: urn:schemas-upnp-org:service:AVTransport:1\r\n\r\n";

/// The LOCATION header of an SSDP answer.
pub fn location(answer: &str) -> Option<String> {
    answer.lines().find_map(|l| {
        l.split_once(':')
            .filter(|(k, _)| k.trim().eq_ignore_ascii_case("location"))
            .map(|(_, v)| v.trim().to_string())
    })
}

/// Looks for DLNA renderers for `wait`: the SSDP search goes out twice, as UDP is lossy, and
/// each answering description is read once.
pub async fn renderers(wait: Duration, found: impl Fn(Found)) -> Result<(), String> {
    let locations = tokio::task::spawn_blocking(move || -> Result<Vec<String>, String> {
        let socket =
            UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("SSDP would not start: {e}"))?;
        socket
            .set_read_timeout(Some(Duration::from_millis(300)))
            .ok();
        let mut seen: Vec<String> = Vec::new();
        let until = Instant::now() + wait;
        let mut sent = 0;
        let mut buf = [0u8; 4096];
        while Instant::now() < until {
            if sent < 2 {
                let _ = socket.send_to(SEARCH.as_bytes(), "239.255.255.250:1900");
                sent += 1;
            }
            if let Ok((n, _)) = socket.recv_from(&mut buf)
                && let Some(loc) = location(&String::from_utf8_lossy(&buf[..n]))
                && !seen.contains(&loc)
            {
                seen.push(loc);
            }
        }
        Ok(seen)
    })
    .await
    .map_err(|e| e.to_string())??;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;
    for loc in locations {
        let Ok(r) = http.get(&loc).send().await else {
            continue;
        };
        let Ok(xml) = r.text().await else { continue };
        let Some(renderer) = dlna::parse_description(&xml, &loc) else {
            continue;
        };
        let host = loc
            .split("://")
            .nth(1)
            .and_then(|r| r.split('/').next())
            .unwrap_or_default()
            .to_string();
        let addr: SocketAddr = host
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));
        found(Found {
            device: CastDevice {
                id: format!("dlna:{loc}"),
                name: renderer.name,
                kind: CastKind::Dlna,
                model: renderer.model,
                address: host,
            },
            addr,
            control: Some(renderer.control),
        });
    }
    Ok(())
}

/// Whether a Chromecast model is known to decode HEVC. A plain "Chromecast" may be an old
/// one that does not, or a Chromecast with Google TV 4K that does; it is not on this list.
pub fn decodes_hevc(model: Option<&str>) -> bool {
    match model {
        Some(m) => {
            let m = m.to_lowercase();
            m.contains("ultra")
                || m.contains("google tv")
                || m.contains("streamer")
                || m.contains("hd")
                || m.contains("nest hub max")
        }
        None => false,
    }
}

/// This machine's address on the route to `peer`: no packet is sent, the system only picks
/// the interface it would use.
pub fn local_ip_for(peer: IpAddr) -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect(SocketAddr::new(peer, 9)).ok()?;
    socket.local_addr().ok().map(|a| a.ip())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ssdp_answer_gives_its_location() {
        let a = "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=1800\r\nLocation: http://192.168.1.40:9197/dmr\r\nST: urn:schemas-upnp-org:service:AVTransport:1\r\n\r\n";
        assert_eq!(location(a).as_deref(), Some("http://192.168.1.40:9197/dmr"));
        assert_eq!(location("HTTP/1.1 200 OK\r\n\r\n"), None);
    }

    #[test]
    fn hevc_by_model() {
        assert!(decodes_hevc(Some("Chromecast Ultra")));
        assert!(decodes_hevc(Some("Chromecast HD")));
        assert!(decodes_hevc(Some("Google TV Streamer")));
        assert!(!decodes_hevc(Some("Chromecast")));
        assert!(!decodes_hevc(None));
    }
}
