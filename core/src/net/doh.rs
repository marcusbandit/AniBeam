//! Names looked up over HTTPS (DNS over HTTPS), for the phone. Some networks, and whole
//! countries, block release sites by answering their names wrongly or not at all; the
//! address itself still answers. Asking a public resolver over HTTPS, on the web's own
//! port and by its IP address, gets the real answer where plain DNS and Android's Private
//! DNS (port 853) are blocked. When no resolver answers, the system's DNS is asked.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

/// The resolvers, asked in order, by address so no lookup is needed to reach them: their
/// certificates name these addresses.
const RESOLVERS: [&str; 2] = ["https://1.1.1.1/dns-query", "https://8.8.8.8/resolve"];
const ASK_TIMEOUT: Duration = Duration::from_secs(5);
/// The longest an answer is kept, whatever its TTL says.
const MAX_TTL: u64 = 3600;

/// Name to its addresses and when they go stale.
type Cache = HashMap<String, (Vec<IpAddr>, Instant)>;

#[derive(Clone)]
pub struct Doh {
    client: reqwest::Client,
    cache: Arc<Mutex<Cache>>,
}

impl Doh {
    /// `client` asks the resolvers; it must not use this resolver itself.
    pub fn new(client: reqwest::Client) -> Doh {
        Doh {
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn ask(&self, resolver: &str, name: &str, kind: &str) -> Option<(Vec<IpAddr>, u64)> {
        let reply = self
            .client
            .get(format!("{resolver}?name={name}&type={kind}"))
            .header("accept", "application/dns-json")
            .timeout(ASK_TIMEOUT)
            .send()
            .await
            .ok()?;
        if !reply.status().is_success() {
            return None;
        }
        let body: serde_json::Value = reply.json().await.ok()?;
        parse(&body)
    }

    /// The addresses for `name`: from the cache, else the first resolver that answers.
    pub async fn lookup(&self, name: &str) -> Option<Vec<IpAddr>> {
        if let Ok(ip) = name.parse::<IpAddr>() {
            return Some(vec![ip]);
        }
        if let Some((ips, until)) = self.cache.lock().unwrap().get(name)
            && *until > Instant::now()
        {
            return Some(ips.clone());
        }
        for resolver in RESOLVERS {
            let (v4, v6) = tokio::join!(
                self.ask(resolver, name, "A"),
                self.ask(resolver, name, "AAAA")
            );
            let ttl = v4
                .as_ref()
                .map(|(_, t)| *t)
                .or(v6.as_ref().map(|(_, t)| *t))
                .unwrap_or(300);
            // IPv4 first: a phone on mobile data often has IPv6 that goes nowhere.
            let ips: Vec<IpAddr> = v4
                .map(|(a, _)| a)
                .unwrap_or_default()
                .into_iter()
                .chain(v6.map(|(a, _)| a).unwrap_or_default())
                .collect();
            if !ips.is_empty() {
                let until = Instant::now() + Duration::from_secs(ttl.clamp(30, MAX_TTL));
                self.cache
                    .lock()
                    .unwrap()
                    .insert(name.to_string(), (ips.clone(), until));
                return Some(ips);
            }
        }
        None
    }
}

/// The addresses and the shortest TTL in a DNS JSON answer (Cloudflare's and Google's
/// shape: `Answer` records with `type` 1 for A and 28 for AAAA).
fn parse(body: &serde_json::Value) -> Option<(Vec<IpAddr>, u64)> {
    let answers = body.get("Answer")?.as_array()?;
    let mut ips = Vec::new();
    let mut ttl = u64::MAX;
    for a in answers {
        let kind = a.get("type").and_then(|t| t.as_u64()).unwrap_or(0);
        if kind != 1 && kind != 28 {
            continue;
        }
        if let Some(ip) = a
            .get("data")
            .and_then(|d| d.as_str())
            .and_then(|d| d.parse::<IpAddr>().ok())
        {
            ips.push(ip);
            ttl = ttl.min(a.get("TTL").and_then(|t| t.as_u64()).unwrap_or(300));
        }
    }
    (!ips.is_empty()).then_some((ips, ttl))
}

impl Resolve for Doh {
    fn resolve(&self, name: Name) -> Resolving {
        let doh = self.clone();
        let host = name.as_str().to_string();
        Box::pin(async move {
            let ips = match doh.lookup(&host).await {
                Some(ips) => ips,
                // No resolver answered: the system's DNS, as before.
                None => tokio::net::lookup_host((host.as_str(), 0))
                    .await?
                    .map(|a| a.ip())
                    .collect(),
            };
            let addrs: Addrs = Box::new(ips.into_iter().map(|ip| SocketAddr::new(ip, 0)));
            Ok(addrs)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dns_json_answer_gives_its_addresses_and_shortest_ttl() {
        let body = serde_json::json!({
            "Status": 0,
            "Answer": [
                { "name": "nyaa.si", "type": 5, "TTL": 60, "data": "alias.example." },
                { "name": "nyaa.si", "type": 1, "TTL": 300, "data": "186.2.163.20" },
                { "name": "nyaa.si", "type": 1, "TTL": 120, "data": "186.2.163.21" }
            ]
        });
        let (ips, ttl) = parse(&body).unwrap();
        assert_eq!(
            ips,
            vec![
                "186.2.163.20".parse::<IpAddr>().unwrap(),
                "186.2.163.21".parse().unwrap()
            ]
        );
        assert_eq!(ttl, 120);
        assert!(parse(&serde_json::json!({ "Status": 3 })).is_none());
    }
}
