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
/// How long the resolvers are left alone after none could be reached.
const DOWN_FOR: Duration = Duration::from_secs(300);
/// How long a name the resolvers have no address for goes straight to the system's DNS,
/// when the reply says nothing shorter.
const NONE_FOR: u64 = 300;

/// Endings that only a home network, a router or a tailnet answers for.
const HOME_SUFFIXES: [&str; 10] = [
    ".lan",
    ".local",
    ".localdomain",
    ".localhost",
    ".home",
    ".home.arpa",
    ".internal",
    ".corp",
    ".fritz.box",
    ".ts.net",
];

/// A name only this network can answer: a single label (`prowlarr`), or one under a home,
/// router or tailnet ending (`prowlarr.lan`, `nas.fritz.box`, `box.tail1234.ts.net`). A
/// public resolver has no answer for these, and asking one sends the name off the network.
pub fn is_home_name(name: &str) -> bool {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    !name.contains('.') || HOME_SUFFIXES.iter().any(|end| name.ends_with(end))
}

/// Name to its addresses and when they go stale. No addresses means the resolvers have
/// none for it, so the system's DNS is asked for that name alone.
type Cache = HashMap<String, (Vec<IpAddr>, Instant)>;

#[derive(Clone)]
pub struct Doh {
    client: reqwest::Client,
    /// `RESOLVERS`, or a test's own.
    resolvers: Arc<[String]>,
    cache: Arc<Mutex<Cache>>,
    /// Until when no resolver is asked: set when none could be reached, so a network that
    /// drops them costs one wait, not one per host. A resolver that replies, whatever it
    /// says, never sets it: a name with no public record goes to the system's DNS alone.
    down_until: Arc<Mutex<Option<Instant>>>,
}

impl Doh {
    /// `client` asks the resolvers; it must not use this resolver itself.
    pub fn new(client: reqwest::Client) -> Doh {
        Doh::with_resolvers(client, RESOLVERS.iter().map(|r| r.to_string()).collect())
    }

    fn with_resolvers(client: reqwest::Client, resolvers: Vec<String>) -> Doh {
        Doh {
            client,
            resolvers: resolvers.into(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            down_until: Arc::new(Mutex::new(None)),
        }
    }

    /// The resolver's reply: its addresses, none when it has no record, and its TTL. None
    /// when it could not be reached or what came back is no DNS reply.
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

    /// The addresses for `name`: from the cache, else the first resolver that answers. None
    /// for a home-network name, which never goes to a public resolver: it has no answer for
    /// it, and the name would leave the network (#251). None too when the resolver that
    /// replied has no address for it, or none could be reached. The caller asks the
    /// system's DNS.
    pub async fn lookup(&self, name: &str) -> Option<Vec<IpAddr>> {
        if let Ok(ip) = name.parse::<IpAddr>() {
            return Some(vec![ip]);
        }
        if is_home_name(name) {
            return None;
        }
        if let Some((ips, until)) = self.cache.lock().unwrap().get(name)
            && *until > Instant::now()
        {
            return (!ips.is_empty()).then(|| ips.clone());
        }
        if self
            .down_until
            .lock()
            .unwrap()
            .is_some_and(|until| until > Instant::now())
        {
            return None;
        }
        for resolver in self.resolvers.iter().map(String::as_str) {
            let (v4, v6) = tokio::join!(
                self.ask(resolver, name, "A"),
                self.ask(resolver, name, "AAAA")
            );
            if v4.is_none() && v6.is_none() {
                continue;
            }
            // IPv4 first: a phone on mobile data often has IPv6 that goes nowhere.
            let replies = [v4, v6];
            let ips: Vec<IpAddr> = replies
                .iter()
                .flatten()
                .flat_map(|(a, _)| a.iter().copied())
                .collect();
            // The shortest TTL of a reply with addresses, or with none, of any reply.
            let ttl = replies
                .iter()
                .flatten()
                .filter(|(a, _)| a.is_empty() == ips.is_empty())
                .map(|(_, t)| *t)
                .min()
                .unwrap_or(NONE_FOR);
            // A reply with no address is still a reply: the resolver is there, and this
            // name alone goes to the system's DNS (#458).
            let until = Instant::now() + Duration::from_secs(ttl.clamp(30, MAX_TTL));
            self.cache
                .lock()
                .unwrap()
                .insert(name.to_string(), (ips.clone(), until));
            return (!ips.is_empty()).then_some(ips);
        }
        *self.down_until.lock().unwrap() = Some(Instant::now() + DOWN_FOR);
        None
    }
}

/// The addresses and the shortest TTL in a DNS JSON reply (Cloudflare's and Google's
/// shape: a `Status`, and `Answer` records with `type` 1 for A and 28 for AAAA). A reply
/// with no address (NXDOMAIN, or a name with no record of that type) gives none, with the
/// TTL its `Authority` records give for that, if any. None when `body` is no DNS reply.
fn parse(body: &serde_json::Value) -> Option<(Vec<IpAddr>, u64)> {
    body.get("Status")?.as_u64()?;
    let ttl_of = |a: &serde_json::Value| a.get("TTL").and_then(|t| t.as_u64()).unwrap_or(NONE_FOR);
    let records = |key: &str| {
        body.get(key)
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default()
    };
    let mut ips = Vec::new();
    let mut ttl = u64::MAX;
    for a in &records("Answer") {
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
            ttl = ttl.min(ttl_of(a));
        }
    }
    if ips.is_empty() {
        ttl = records("Authority")
            .iter()
            .map(ttl_of)
            .min()
            .unwrap_or(NONE_FOR);
    }
    Some((ips, ttl))
}

impl Resolve for Doh {
    fn resolve(&self, name: Name) -> Resolving {
        let doh = self.clone();
        let host = name.as_str().to_string();
        Box::pin(async move {
            let ips = match doh.lookup(&host).await {
                Some(ips) => ips,
                // A home name, a name the resolvers have no address for, or no resolver
                // answered: the system's DNS, for this name.
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
    fn home_network_names_are_home_names() {
        for name in [
            "prowlarr",
            "localhost",
            "prowlarr.lan",
            "jackett.local",
            "nas.localdomain",
            "nas.home",
            "nas.home.arpa",
            "box.internal",
            "files.corp",
            "nas.fritz.box",
            "nas.tail1234.ts.net",
            "NAS.LAN.",
        ] {
            assert!(is_home_name(name), "{name} should be a home name");
        }
    }

    /// A resolver that takes connections and never answers, counting them.
    async fn silent_resolver() -> (String, Arc<std::sync::atomic::AtomicUsize>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/dns-query", listener.local_addr().unwrap());
        let asked = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = asked.clone();
        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                drop(socket);
            }
        });
        (url, asked)
    }

    #[tokio::test]
    async fn the_client_resolves_a_home_name_without_the_public_resolvers() {
        let (url, asked) = silent_resolver().await;
        // No proxy from the environment: the resolver is the local listener.
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let doh = Doh::with_resolvers(client, vec![url]);
        let asked = || asked.load(std::sync::atomic::Ordering::SeqCst);

        // The system's DNS answers it.
        let name: Name = "localhost".parse().unwrap();
        let addrs: Vec<SocketAddr> = doh.resolve(name).await.unwrap().collect();
        assert!(!addrs.is_empty());
        assert!(addrs.iter().all(|a| a.ip().is_loopback()), "{addrs:?}");
        assert_eq!(asked(), 0);

        // A public name does go to them first. `.invalid` never resolves anywhere, so the
        // system's DNS after them answers nothing either.
        let name: Name = "public.example.invalid".parse().unwrap();
        let _ = doh.resolve(name).await;
        assert!(asked() > 0);
    }

    #[test]
    fn public_names_are_not_home_names() {
        for name in [
            "nyaa.si",
            "example.com",
            "one.one.one.one",
            "lan.example.com",
            "fritz.box.example.com",
        ] {
            assert!(!is_home_name(name), "{name} should be public");
        }
    }

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
        // Something that is no DNS reply at all.
        assert!(parse(&serde_json::json!({ "error": "blocked" })).is_none());
    }

    #[test]
    fn a_reply_with_no_record_is_a_reply_with_no_address() {
        let nxdomain = serde_json::json!({
            "Status": 3,
            "Authority": [{ "name": "example", "type": 6, "TTL": 900, "data": "a. b. 1 2 3 4 5" }]
        });
        assert_eq!(parse(&nxdomain), Some((Vec::new(), 900)));
        let empty = serde_json::json!({ "Status": 0 });
        assert_eq!(parse(&empty), Some((Vec::new(), NONE_FOR)));
    }

    /// A resolver on this machine that gives an A question `a` and any other `rest`, and
    /// how many questions it got.
    async fn resolver(
        a: &'static str,
        rest: &'static str,
    ) -> (String, Arc<std::sync::atomic::AtomicUsize>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/dns-query", listener.local_addr().unwrap());
        let asked = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = asked.clone();
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let body = if String::from_utf8_lossy(&buf[..n]).contains("type=A ") {
                    a
                } else {
                    rest
                };
                let reply = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/dns-json\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(reply.as_bytes()).await;
            }
        });
        (url, asked)
    }

    /// An address nothing listens on.
    fn nobody() -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}/dns-query", listener.local_addr().unwrap())
    }

    fn local(resolvers: Vec<String>) -> Doh {
        // No proxy from the environment: the resolvers are local listeners.
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        Doh::with_resolvers(client, resolvers)
    }

    #[tokio::test]
    async fn a_name_with_no_public_record_leaves_doh_on_for_other_names() {
        let (url, asked) = resolver(r#"{"Status":3}"#, r#"{"Status":3}"#).await;
        let doh = local(vec![url]);
        let asked = || asked.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(doh.lookup("nas.example").await, None);
        assert!(doh.down_until.lock().unwrap().is_none());
        // The name is kept as one for the system's DNS, so it is not asked again.
        let after_first = asked();
        assert_eq!(doh.lookup("nas.example").await, None);
        assert_eq!(asked(), after_first);
        // Any other name is still asked over HTTPS.
        assert_eq!(doh.lookup("other.example").await, None);
        assert!(asked() > after_first);
    }

    #[tokio::test]
    async fn only_unreachable_resolvers_switch_doh_off() {
        let doh = local(vec![nobody(), nobody()]);
        assert_eq!(doh.lookup("a.example").await, None);
        assert!(
            doh.down_until
                .lock()
                .unwrap()
                .is_some_and(|until| until > Instant::now())
        );
    }

    #[tokio::test]
    async fn the_first_resolver_that_replies_is_the_answer() {
        let (url, _) = resolver(
            r#"{"Status":0,"Answer":[{"name":"a.example","type":1,"TTL":60,"data":"192.0.2.7"}]}"#,
            r#"{"Status":0}"#,
        )
        .await;
        let doh = local(vec![nobody(), url]);
        assert_eq!(
            doh.lookup("a.example").await,
            Some(vec!["192.0.2.7".parse::<IpAddr>().unwrap()])
        );
        assert!(doh.down_until.lock().unwrap().is_none());
    }
}
