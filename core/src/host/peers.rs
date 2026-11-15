//! The other Anibeams on the tailnet, for a phone that cannot list tailnet
//! devices itself (#313): every trusted, online device the status names is
//! asked for its hello on the usual port, all at once and briefly, and the
//! ones that answer as Anibeam are handed over.

use std::time::Duration;

use anibeam_host_api as wire;

use super::addr;
use super::trust::Tailnet;

/// How long one device gets to say hello.
pub const ASK_FOR: Duration = Duration::from_secs(2);

/// Each device worth asking, by name and the base URL it would answer on:
/// online, not switched off, at its first tailnet IPv4 address or else its
/// first tailnet address.
pub fn candidates(t: &Tailnet, off: &[String], port: u16) -> Vec<(String, String)> {
    t.peers
        .iter()
        .filter(|p| p.online && !off.contains(&p.id))
        .filter_map(|p| {
            // Only tailnet addresses are asked, whatever the status names.
            let ip = addr::pick(&p.ips).first().copied()?;
            let at = std::net::SocketAddr::new(ip, port);
            Some((p.name.clone(), format!("http://{at}")))
        })
        .collect()
}

/// The candidates that answer their hello as Anibeam, in the order given.
pub async fn find(candidates: Vec<(String, String)>) -> Vec<wire::PeerHost> {
    // A device answers for itself only: no redirect is followed, so none can
    // point this machine at its own or its network's services, and no proxy
    // stands between two tailnet devices.
    let Ok(http) = reqwest::Client::builder()
        .timeout(ASK_FOR)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
    else {
        return Vec::new();
    };
    let mut asks = tokio::task::JoinSet::new();
    for (i, (name, url)) in candidates.into_iter().enumerate() {
        let http = http.clone();
        asks.spawn(async move {
            let hello: Option<wire::Hello> =
                match http.get(format!("{url}{}", wire::HELLO)).send().await {
                    Ok(r) if r.status().is_success() => r.json().await.ok(),
                    _ => None,
                };
            let found = hello
                .filter(|h| h.app == wire::APP)
                .map(|h| wire::PeerHost {
                    name: if h.host.is_empty() { name } else { h.host },
                    url,
                    version: h.version,
                });
            (i, found)
        });
    }
    let mut out: Vec<(usize, wire::PeerHost)> = Vec::new();
    while let Some(done) = asks.join_next().await {
        if let Ok((i, Some(host))) = done {
            out.push((i, host));
        }
    }
    out.sort_by_key(|(i, _)| *i);
    out.into_iter().map(|(_, h)| h).collect()
}

/// The list as handed to the device at `from`, which is never handed itself.
pub fn others(hosts: &[wire::PeerHost], from: std::net::IpAddr) -> Vec<wire::PeerHost> {
    let from = super::trust::canonical(from);
    hosts
        .iter()
        .filter(|h| {
            let at = h
                .url
                .strip_prefix("http://")
                .and_then(|a| a.parse::<std::net::SocketAddr>().ok());
            at.is_none_or(|at| super::trust::canonical(at.ip()) != from)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::trust::read_status;

    const STATUS: &[u8] = br#"{
        "Self": {"ID": "nSelf", "TailscaleIPs": ["100.64.0.1"]},
        "Peer": {
            "a": {"ID": "nPhone", "HostName": "pixel", "Online": true, "TailscaleIPs": ["fd7a:115c:a1e0::2", "100.70.1.2"]},
            "b": {"ID": "nLaptop", "HostName": "laptop", "Online": true, "TailscaleIPs": ["fd7a:115c:a1e0::3"]},
            "c": {"ID": "nAway", "HostName": "away", "Online": false, "TailscaleIPs": ["100.64.0.4"]},
            "d": {"ID": "nOff", "HostName": "hub", "Online": true, "TailscaleIPs": ["100.64.0.5"]},
            "e": {"ID": "nLan", "HostName": "printer", "Online": true, "TailscaleIPs": ["192.168.1.9"]}
        }
    }"#;

    #[test]
    fn only_trusted_online_devices_are_asked_at_an_ipv4_address_first() {
        let t = read_status(STATUS).unwrap();
        let off = vec!["nOff".to_string()];
        assert_eq!(
            candidates(&t, &off, 8790),
            vec![
                (
                    "laptop".to_string(),
                    "http://[fd7a:115c:a1e0::3]:8790".to_string()
                ),
                ("pixel".to_string(), "http://100.70.1.2:8790".to_string()),
            ]
        );
    }

    async fn answer(body: &'static str) -> String {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let at = l.local_addr().unwrap();
        let app =
            axum::Router::new().route(wire::HELLO, axum::routing::get(move || async move { body }));
        tokio::spawn(async move { axum::serve(l, app).await });
        format!("http://{at}")
    }

    #[test]
    fn only_what_answers_as_anibeam_is_handed_over() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let anibeam =
                answer(r#"{"app":"anibeam","api":1,"version":"2.4.0","host":"laptop","now":0}"#)
                    .await;
            let other =
                answer(r#"{"app":"something-else","api":1,"version":"1","host":"x","now":0}"#)
                    .await;
            let junk = answer("not json").await;
            // A port nothing listens on: bound, then let go.
            let closed = {
                let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
                format!("http://{}", l.local_addr().unwrap())
            };
            let found = find(vec![
                ("a".into(), closed),
                ("b".into(), other),
                ("laptop-name".into(), anibeam.clone()),
                ("d".into(), junk),
            ])
            .await;
            assert_eq!(
                found,
                vec![wire::PeerHost {
                    name: "laptop".into(),
                    url: anibeam,
                    version: "2.4.0".into(),
                }]
            );
        });
    }

    #[test]
    fn a_device_that_sends_the_host_elsewhere_is_not_followed() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let anibeam =
                answer(r#"{"app":"anibeam","api":1,"version":"2.4.0","host":"laptop","now":0}"#)
                    .await;
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let at = l.local_addr().unwrap();
            let to = format!("{anibeam}{}", wire::HELLO);
            let app = axum::Router::new().route(
                wire::HELLO,
                axum::routing::get(move || async move { axum::response::Redirect::to(&to) }),
            );
            tokio::spawn(async move { axum::serve(l, app).await });
            let found = find(vec![("sly".into(), format!("http://{at}"))]).await;
            assert!(found.is_empty(), "{found:?}");
        });
    }

    #[test]
    fn the_device_that_asks_is_not_handed_itself() {
        let host = |name: &str, url: &str| wire::PeerHost {
            name: name.into(),
            url: url.into(),
            version: "2.4.0".into(),
        };
        let all = vec![
            host("pixel", "http://100.70.1.2:8790"),
            host("laptop", "http://[fd7a:115c:a1e0::3]:8790"),
        ];
        assert_eq!(
            others(&all, "100.70.1.2".parse().unwrap()),
            vec![all[1].clone()]
        );
        assert_eq!(
            others(&all, "::ffff:100.70.1.2".parse().unwrap()),
            vec![all[1].clone()],
            "an IPv4 asker seen through an IPv6 socket"
        );
        assert_eq!(
            others(&all, "fd7a:115c:a1e0::3".parse().unwrap()),
            vec![all[0].clone()]
        );
        assert_eq!(others(&all, "127.0.0.1".parse().unwrap()), all);
    }
}
