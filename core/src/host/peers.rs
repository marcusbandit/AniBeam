//! The other AniBeams on the tailnet, for a phone that cannot list tailnet
//! devices itself (#313): every trusted, online device the status names is
//! asked for its hello on the usual port, all at once and briefly, and the
//! ones that answer as AniBeam are handed over.

use std::time::Duration;

use anibeam_host_api as wire;

use super::addr::is_tailscale;
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
            let ips: Vec<_> = p.ips.iter().filter(|ip| is_tailscale(ip)).collect();
            let ip = ips
                .iter()
                .find(|ip| ip.is_ipv4())
                .or(ips.first())
                .copied()?;
            let at = std::net::SocketAddr::new(*ip, port);
            Some((p.name.clone(), format!("http://{at}")))
        })
        .collect()
}

/// The candidates that answer their hello as AniBeam, in the order given.
pub async fn find(candidates: Vec<(String, String)>) -> Vec<wire::PeerHost> {
    let Ok(http) = reqwest::Client::builder().timeout(ASK_FOR).build() else {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::trust::read_status;

    const STATUS: &[u8] = br#"{
        "Self": {"ID": "nSelf", "TailscaleIPs": ["100.65.105.9"]},
        "Peer": {
            "a": {"ID": "nPhone", "HostName": "pixel", "Online": true, "TailscaleIPs": ["fd7a:115c:a1e0::2", "100.70.1.2"]},
            "b": {"ID": "nLaptop", "HostName": "kangaeru", "Online": true, "TailscaleIPs": ["fd7a:115c:a1e0::3"]},
            "c": {"ID": "nAway", "HostName": "nemurenai", "Online": false, "TailscaleIPs": ["100.80.0.4"]},
            "d": {"ID": "nOff", "HostName": "homehub", "Online": true, "TailscaleIPs": ["100.80.0.5"]},
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
                    "kangaeru".to_string(),
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
                answer(r#"{"app":"anibeam","api":1,"version":"2.4.0","host":"kangaeru","now":0}"#)
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
                ("kangaeru-name".into(), anibeam.clone()),
                ("d".into(), junk),
            ])
            .await;
            assert_eq!(
                found,
                vec![wire::PeerHost {
                    name: "kangaeru".into(),
                    url: anibeam,
                    version: "2.4.0".into(),
                }]
            );
        });
    }
}
