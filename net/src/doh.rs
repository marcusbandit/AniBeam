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

    /// What one resolver says about one record type of `name`.
    async fn ask(&self, resolver: &str, name: &str, kind: &str) -> Asked {
        let Ok(reply) = self
            .client
            .get(format!("{resolver}?name={name}&type={kind}"))
            .header("accept", "application/dns-json")
            .timeout(ASK_TIMEOUT)
            .send()
            .await
        else {
            return Asked::Unreachable;
        };
        if !reply.status().is_success() {
            return Asked::Failed;
        }
        match reply.json::<serde_json::Value>().await {
            Ok(body) => parse(&body).map_or(Asked::Failed, |(ips, ttl)| Asked::Records(ips, ttl)),
            Err(_) => Asked::Failed,
        }
    }

    /// The addresses for `name`, or None for the system's DNS to answer: a home-network
    /// name, a name no resolver has an address for, or no resolver answered. What
    /// [`Doh::find`] found, without why.
    pub async fn lookup(&self, name: &str) -> Option<Vec<IpAddr>> {
        match self.find(name).await {
            Found::Addresses(ips) => Some(ips),
            Found::NoRecord | Found::NoAnswer => None,
        }
    }

    /// What the resolvers say about `name`: from the cache, else the first resolver with
    /// addresses for it. A home-network name never goes to a public resolver: it has no
    /// answer for it, and the name would leave the network (#251).
    ///
    /// A resolver that replies with no record for both A and AAAA does not end the round:
    /// a resolver can answer a name it blocks with nothing, so the next is asked too. Only
    /// when no resolver has an address, and one replied with no record, is the name kept
    /// for the system's DNS, for as long as the reply says. A question that failed, or
    /// came back as an error (SERVFAIL, REFUSED), proves nothing about the name, so it is
    /// not kept. Only when no resolver could be reached at all are they left alone for a
    /// while (#458).
    pub async fn find(&self, name: &str) -> Found {
        if let Ok(ip) = name.parse::<IpAddr>() {
            return Found::Addresses(vec![ip]);
        }
        if is_home_name(name) {
            return Found::NoRecord;
        }
        if let Some((ips, until)) = self.cache.lock().unwrap().get(name)
            && *until > Instant::now()
        {
            return if ips.is_empty() {
                Found::NoRecord
            } else {
                Found::Addresses(ips.clone())
            };
        }
        if self
            .down_until
            .lock()
            .unwrap()
            .is_some_and(|until| until > Instant::now())
        {
            return Found::NoAnswer;
        }
        let mut reached = false;
        // The shortest TTL a resolver gave for having no record, once one has.
        let mut none_for: Option<u64> = None;
        for resolver in self.resolvers.iter().map(String::as_str) {
            let (v4, v6) = tokio::join!(
                self.ask(resolver, name, "A"),
                self.ask(resolver, name, "AAAA")
            );
            reached |= !matches!(v4, Asked::Unreachable) || !matches!(v6, Asked::Unreachable);
            // IPv4 first: a phone on mobile data often has IPv6 that goes nowhere.
            let mut ips = Vec::new();
            let mut ttl = u64::MAX;
            for asked in [&v4, &v6] {
                if let Asked::Records(found, t) = asked
                    && !found.is_empty()
                {
                    ips.extend(found.iter().copied());
                    ttl = ttl.min(*t);
                }
            }
            if !ips.is_empty() {
                self.keep(name, ips.clone(), ttl);
                return Found::Addresses(ips);
            }
            if let (Asked::Records(_, a), Asked::Records(_, b)) = (&v4, &v6) {
                let t = (*a).min(*b);
                none_for = Some(none_for.map_or(t, |n| n.min(t)));
            }
        }
        if let Some(ttl) = none_for {
            self.keep(name, Vec::new(), ttl);
            return Found::NoRecord;
        }
        if !reached {
            *self.down_until.lock().unwrap() = Some(Instant::now() + DOWN_FOR);
        }
        Found::NoAnswer
    }

    /// Keeps what was found for `name` for `ttl` seconds, within bounds. Stale entries go
    /// on each write, so names asked once do not pile up.
    fn keep(&self, name: &str, ips: Vec<IpAddr>, ttl: u64) {
        let now = Instant::now();
        let until = now + Duration::from_secs(ttl.clamp(30, MAX_TTL));
        let mut cache = self.cache.lock().unwrap();
        cache.retain(|_, (_, stale)| *stale > now);
        cache.insert(name.to_string(), (ips, until));
    }
}

/// What one resolver said about one record type.
enum Asked {
    /// A DNS reply that settles it: the addresses, none when there is no record, and the
    /// TTL that holds for.
    Records(Vec<IpAddr>, u64),
    /// It answered, but not with anything that settles the name: an HTTP error, a body
    /// that is no DNS reply, or a DNS error such as SERVFAIL or REFUSED.
    Failed,
    /// It could not be reached.
    Unreachable,
}

/// What the resolvers say about a name.
#[derive(Debug, PartialEq)]
pub enum Found {
    Addresses(Vec<IpAddr>),
    /// No public address: a home-network name, or one the resolvers have no record for.
    /// The system's DNS is asked for it.
    NoRecord,
    /// No resolver answered.
    NoAnswer,
}

/// The addresses and the shortest TTL in a DNS JSON reply (Cloudflare's and Google's
/// shape: a `Status`, and `Answer` records with `type` 1 for A and 28 for AAAA). A reply
/// with no address (NXDOMAIN, or NOERROR with no record of that type) gives none, for as
/// long as its SOA says: the lower of the record's TTL and its MINIMUM (RFC 2308). None
/// when `body` is no DNS reply, or one with another status, which settles nothing.
fn parse(body: &serde_json::Value) -> Option<(Vec<IpAddr>, u64)> {
    const NOERROR: u64 = 0;
    const NXDOMAIN: u64 = 3;
    let status = body.get("Status")?.as_u64()?;
    if status != NOERROR && status != NXDOMAIN {
        return None;
    }
    let records = |key: &str| {
        body.get(key)
            .and_then(|a| a.as_array())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    };
    let ttl_of = |a: &serde_json::Value| a.get("TTL").and_then(|t| t.as_u64());
    let mut ips = Vec::new();
    let mut ttl = u64::MAX;
    for a in records("Answer") {
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
            ttl = ttl.min(ttl_of(a).unwrap_or(300));
        }
    }
    if ips.is_empty() {
        // An SOA's data ends in its MINIMUM: `mname rname serial refresh retry expire min`.
        let minimum = |a: &serde_json::Value| {
            a.get("data")
                .and_then(|d| d.as_str())
                .and_then(|d| d.split_whitespace().last())
                .and_then(|m| m.parse::<u64>().ok())
        };
        ttl = records("Authority")
            .iter()
            .filter(|a| a.get("type").and_then(|t| t.as_u64()) == Some(6))
            .filter_map(|a| match (ttl_of(a), minimum(a)) {
                (Some(t), Some(m)) => Some(t.min(m)),
                (t, m) => t.or(m),
            })
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
            "tracker.example",
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
                { "name": "tracker.example", "type": 5, "TTL": 60, "data": "alias.example." },
                { "name": "tracker.example", "type": 1, "TTL": 300, "data": "186.2.163.20" },
                { "name": "tracker.example", "type": 1, "TTL": 120, "data": "186.2.163.21" }
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
        // Kept for the lower of the SOA's own TTL and its MINIMUM.
        let nxdomain = serde_json::json!({
            "Status": 3,
            "Authority": [{ "name": "example", "type": 6, "TTL": 900, "data": "a. b. 1 2 3 4 600" }]
        });
        assert_eq!(parse(&nxdomain), Some((Vec::new(), 600)));
        let empty = serde_json::json!({ "Status": 0 });
        assert_eq!(parse(&empty), Some((Vec::new(), NONE_FOR)));
        // SERVFAIL and REFUSED say nothing about the name.
        for status in [2, 5] {
            assert!(parse(&serde_json::json!({ "Status": status })).is_none());
        }
    }

    const NXDOMAIN: &str = r#"{"Status":3}"#;
    const SERVFAIL: &str = r#"{"Status":2}"#;
    const NO_AAAA: &str = r#"{"Status":0}"#;
    const ONE_A: &str =
        r#"{"Status":0,"Answer":[{"name":"a.example","type":1,"TTL":60,"data":"192.0.2.7"}]}"#;

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

    /// An address nothing answers on: the port stays bound, never listening, while the
    /// socket is held, so no other test can be given it.
    fn nobody() -> (String, tokio::net::TcpSocket) {
        let socket = tokio::net::TcpSocket::new_v4().unwrap();
        socket.bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let url = format!("http://{}/dns-query", socket.local_addr().unwrap());
        (url, socket)
    }

    fn local(resolvers: Vec<String>) -> Doh {
        // No proxy from the environment: the resolvers are local listeners.
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        Doh::with_resolvers(client, resolvers)
    }

    fn count(asked: &std::sync::atomic::AtomicUsize) -> usize {
        asked.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[tokio::test]
    async fn a_name_with_no_public_record_leaves_doh_on_for_other_names() {
        let (url, asked) = resolver(NXDOMAIN, NXDOMAIN).await;
        let doh = local(vec![url]);
        assert_eq!(doh.find("nas.example").await, Found::NoRecord);
        assert_eq!(doh.lookup("nas.example").await, None);
        assert!(doh.down_until.lock().unwrap().is_none());
        // The name is kept as one for the system's DNS, so it is not asked again.
        let after_first = count(&asked);
        assert_eq!(doh.find("nas.example").await, Found::NoRecord);
        assert_eq!(count(&asked), after_first);
        // Any other name is still asked over HTTPS.
        assert_eq!(doh.find("other.example").await, Found::NoRecord);
        assert!(count(&asked) > after_first);
    }

    #[tokio::test]
    async fn a_resolver_with_nothing_for_a_name_does_not_stop_the_next() {
        // A resolver can answer a name it blocks with no record.
        let (empty, _) = resolver(NO_AAAA, NO_AAAA).await;
        let (full, _) = resolver(ONE_A, NO_AAAA).await;
        let doh = local(vec![empty, full]);
        assert_eq!(
            doh.find("a.example").await,
            Found::Addresses(vec!["192.0.2.7".parse().unwrap()])
        );
    }

    #[tokio::test]
    async fn a_failed_question_proves_nothing_about_the_name() {
        // A fails, AAAA has no record: not kept, the next resolver is asked, DoH stays on.
        let (half, _) = resolver(SERVFAIL, NO_AAAA).await;
        let doh = local(vec![half.clone()]);
        assert_eq!(doh.find("a.example").await, Found::NoAnswer);
        assert!(doh.cache.lock().unwrap().is_empty());
        assert!(doh.down_until.lock().unwrap().is_none());
        let (full, _) = resolver(ONE_A, NO_AAAA).await;
        let doh = local(vec![half, full]);
        assert_eq!(
            doh.find("a.example").await,
            Found::Addresses(vec!["192.0.2.7".parse().unwrap()])
        );
    }

    #[tokio::test]
    async fn only_unreachable_resolvers_switch_doh_off() {
        let (one, _held) = nobody();
        let (two, _also) = nobody();
        let doh = local(vec![one, two]);
        assert_eq!(doh.find("a.example").await, Found::NoAnswer);
        assert!(
            doh.down_until
                .lock()
                .unwrap()
                .is_some_and(|until| until > Instant::now())
        );
    }

    #[tokio::test]
    async fn the_first_resolver_that_replies_is_the_answer() {
        let (dead, _held) = nobody();
        let (url, _) = resolver(ONE_A, NO_AAAA).await;
        let doh = local(vec![dead, url]);
        assert_eq!(
            doh.lookup("a.example").await,
            Some(vec!["192.0.2.7".parse::<IpAddr>().unwrap()])
        );
        assert!(doh.down_until.lock().unwrap().is_none());
    }
}
