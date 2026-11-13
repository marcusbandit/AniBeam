//! HTTP to the host and to AniList.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use anibeam_host_api::proof::{self, DeviceKey};

use crate::address::HostAddress;
use crate::api::{self, Hello, Snapshot};

/// A poster bigger than this is not a poster.
const POSTER_LIMIT: u64 = 8 * 1024 * 1024;
/// A snapshot of a few thousand episodes is a few MB of JSON; leave room.
const SNAPSHOT_LIMIT: u64 = 64 * 1024 * 1024;

/// One agent for the life of the app, so connections and TLS sessions are reused.
pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        // A tailnet host that is off answers nothing at all, so the connect timeout is
        // what decides how long "offline" takes to show.
        .timeout_connect(Some(Duration::from_secs(4)))
        .timeout_global(Some(Duration::from_secs(20)))
        .http_status_as_error(false)
        .user_agent(concat!("AniBeam-Android/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostError {
    /// Nothing answered: off, asleep, not sharing, or the tailnet is down.
    Unreachable(String),
    /// Something answered, but it is not AniBeam. Holds the whole sentence, worded in the
    /// language the app spoke when it happened.
    NotAniBeam(String),
    /// An AniBeam that speaks a newer API than this app reads.
    TooNew(u32),
    /// AniBeam answered with an error or something unreadable.
    Bad(String),
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostError::Unreachable(why) => write!(f, "{why}"),
            HostError::NotAniBeam(what) => write!(f, "{what}"),
            HostError::TooNew(api) => {
                write!(
                    f,
                    "{}",
                    tr!(
                        "client-too-new",
                        host_api = *api,
                        app_api = api::API_VERSION
                    )
                )
            }
            HostError::Bad(why) => write!(f, "{why}"),
        }
    }
}

/// Plain words for a failed request.
pub fn describe(error: &ureq::Error, host: &str) -> String {
    match error {
        ureq::Error::HostNotFound => tr!("client-not-found", host = host),
        ureq::Error::Timeout(_) => tr!("client-timeout", host = host),
        ureq::Error::ConnectionFailed => tr!("client-connect-failed", host = host),
        ureq::Error::Io(io) => match io.kind() {
            std::io::ErrorKind::ConnectionRefused => tr!("client-refused", host = host),
            std::io::ErrorKind::TimedOut => tr!("client-timeout", host = host),
            std::io::ErrorKind::NetworkUnreachable | std::io::ErrorKind::HostUnreachable => {
                tr!("client-no-route", host = host)
            }
            // What connect() says on Android for an address it will not even try.
            std::io::ErrorKind::InvalidInput => tr!("client-invalid-input", host = host),
            _ => tr!("client-unreachable", host = host, why = io.to_string()),
        },
        other => tr!("client-unreachable", host = host, why = other.to_string()),
    }
}

/// The phone's key, set once at start ([`use_device_key`]), that every client signs with.
static DEVICE: OnceLock<Arc<DeviceKey>> = OnceLock::new();

/// Signs every request to a host that reads proofs with `key` from now on (#555).
pub fn use_device_key(key: DeviceKey) {
    let _ = DEVICE.set(Arc::new(key));
}

/// Each host's key and how far its clock is ahead of the phone's, by base URL, once its
/// hello said it reads proofs. Shared, so a client made for one fetch signs too.
static HOSTS: OnceLock<Mutex<HashMap<String, (String, i64)>>> = OnceLock::new();

fn hosts() -> std::sync::MutexGuard<'static, HashMap<String, (String, i64)>> {
    HOSTS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

pub struct HostClient {
    agent: ureq::Agent,
    address: HostAddress,
    key: Option<Arc<DeviceKey>>,
}

impl HostClient {
    pub fn new(agent: ureq::Agent, address: HostAddress) -> Self {
        HostClient {
            agent,
            address,
            key: DEVICE.get().cloned(),
        }
    }

    /// Signs with `key` instead of the phone's.
    pub fn with_key(mut self, key: DeviceKey) -> Self {
        self.key = Some(Arc::new(key));
        self
    }

    /// The proof header for one request, when the host reads proofs and the phone has a
    /// key. Made afresh for every try: a nonce is good once. The hello itself goes
    /// unsigned: it is how the phone learns the host's key, and a host whose key changed
    /// would refuse a hello signed to the old one.
    fn proof(&self, method: &str, path: &str) -> Option<String> {
        if path == api::HELLO {
            return None;
        }
        let key = self.key.as_ref()?;
        let (host, ahead) = hosts().get(&self.address.base_url())?.clone();
        let mut nonce = [0u8; 16];
        getrandom::fill(&mut nonce).ok()?;
        Some(key.sign(
            &host,
            crate::now() + ahead,
            &proof::hex(&nonce),
            method,
            path,
        ))
    }

    pub fn address(&self) -> &HostAddress {
        &self.address
    }

    /// One `GET`, whatever status came back: the status, the body and the content type.
    fn fetch(&self, path: &str, limit: u64) -> Result<(u16, Vec<u8>, Option<String>), HostError> {
        let url = format!("{}{}", self.address.base_url(), path);
        // A connection that drops before the answer gets one more try: every route is a
        // GET that changes nothing.
        let dropped = |e: &ureq::Error| {
            matches!(e, ureq::Error::Io(io) if matches!(io.kind(),
                std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted))
        };
        let call = || {
            let mut request = self.agent.get(&url);
            if let Some(p) = self.proof("GET", path) {
                request = request.header(proof::HEADER, p);
            }
            request.call()
        };
        let mut response = match call() {
            Err(e) if dropped(&e) => call(),
            other => other,
        }
        .map_err(|e| HostError::Unreachable(describe(&e, &self.address.display())))?;
        let status = response.status().as_u16();
        let kind = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let body = response
            .body_mut()
            .with_config()
            .limit(limit)
            .read_to_vec()
            .map_err(|e| {
                HostError::Bad(tr!(
                    "client-broke-off",
                    host = self.address.display(),
                    why = e.to_string()
                ))
            })?;
        Ok((status, body, kind))
    }

    /// The answer to a `GET` that came back 200.
    fn get(&self, path: &str, limit: u64) -> Result<(Vec<u8>, Option<String>), HostError> {
        let (status, body, kind) = self.fetch(path, limit)?;
        if status != 200 {
            if let Some(why) = refusal(status, &body) {
                return Err(HostError::Bad(why));
            }
            return Err(HostError::Bad(tr!(
                "client-answered",
                host = self.address.display(),
                status = status.to_string(),
                path = path
            )));
        }
        Ok((body, kind))
    }

    /// Looks the name up first, because a failed lookup reaches ureq as a bare
    /// "Invalid argument" on Android.
    fn resolve(&self) -> Result<(), HostError> {
        use std::net::ToSocketAddrs;
        let host = self
            .address
            .host
            .trim_start_matches('[')
            .trim_end_matches(']');
        let found = (host, self.address.port)
            .to_socket_addrs()
            .map(|mut all| all.next().is_some());
        match found {
            Ok(true) => Ok(()),
            _ => Err(HostError::Unreachable(tr!(
                "client-cannot-find",
                host = self.address.host.as_str()
            ))),
        }
    }

    /// Asks who is there. Cheap, and the first thing every sync does.
    pub fn hello(&self) -> Result<Hello, HostError> {
        self.resolve()?;
        let (status, body, _) = self.fetch(api::HELLO, 64 * 1024)?;
        if status != 200 {
            // A refusal is the computer's own word; any other status on the hello route is
            // a web server that is not AniBeam.
            return Err(match refusal(status, &body) {
                Some(why) => HostError::Bad(why),
                None => HostError::NotAniBeam(tr!(
                    "client-not-anibeam-status",
                    host = self.address.display(),
                    status = status.to_string(),
                    path = api::HELLO
                )),
            });
        }
        let hello: Hello = serde_json::from_slice(&body)
            .map_err(|_| HostError::NotAniBeam(tr!("client-not-anibeam-hello")))?;
        if hello.app != api::APP {
            return Err(HostError::NotAniBeam(tr!(
                "client-not-anibeam-name",
                name = hello.app.as_str()
            )));
        }
        if hello.api > api::API_VERSION {
            return Err(HostError::TooNew(hello.api));
        }
        let base = self.address.base_url();
        if hello.features.iter().any(|f| f == api::FEATURE_PROOF) && !hello.key.is_empty() {
            hosts().insert(base, (hello.key.clone(), hello.now - crate::now()));
        } else {
            hosts().remove(&base);
        }
        Ok(hello)
    }

    pub fn snapshot(&self) -> Result<Snapshot, HostError> {
        let (body, _) = self.get(api::SNAPSHOT, SNAPSHOT_LIMIT)?;
        serde_json::from_slice(&body)
            .map_err(|e| HostError::Bad(tr!("client-snapshot-unreadable", why = e.to_string())))
    }

    /// The body of any `GET` on the host, up to 1 MB.
    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>, HostError> {
        self.get(path, 1024 * 1024).map(|(body, _)| body)
    }

    /// What is in one of the host's episode files.
    pub fn probe(&self, file: u64) -> Result<api::Probe, HostError> {
        let (body, _) = self.get(&api::probe_path(file), 1024 * 1024)?;
        serde_json::from_slice(&body)
            .map_err(|e| HostError::Bad(tr!("client-probe-unreadable", why = e.to_string())))
    }

    /// The host's address with the name looked up, for a player that resolves names on
    /// its own and may not ask the tailnet's DNS: `http://100.94.40.21:8790`.
    pub fn stream_base(&self) -> Result<String, HostError> {
        use std::net::ToSocketAddrs;
        let host = self
            .address
            .host
            .trim_start_matches('[')
            .trim_end_matches(']');
        let found = (host, self.address.port)
            .to_socket_addrs()
            .ok()
            .and_then(|mut all| all.next());
        match found {
            Some(std::net::SocketAddr::V4(a)) => Ok(format!("http://{}:{}", a.ip(), a.port())),
            Some(std::net::SocketAddr::V6(a)) => Ok(format!("http://[{}]:{}", a.ip(), a.port())),
            None => Err(HostError::Unreachable(tr!(
                "client-cannot-find",
                host = self.address.host.as_str()
            ))),
        }
    }

    /// `POST`s JSON and hands back the status and the answer, whatever the status.
    pub fn post(&self, path: &str, body: &[u8]) -> Result<(u16, Vec<u8>), HostError> {
        let url = format!("{}{}", self.address.base_url(), path);
        let mut request = self
            .agent
            .post(&url)
            .header("content-type", "application/json");
        if let Some(p) = self.proof("POST", path) {
            request = request.header(proof::HEADER, p);
        }
        let mut response = request
            .send(body)
            .map_err(|e| HostError::Unreachable(describe(&e, &self.address.display())))?;
        let status = response.status().as_u16();
        let body = response
            .body_mut()
            .with_config()
            .limit(4 * 1024 * 1024)
            .read_to_vec()
            .map_err(|e| {
                HostError::Bad(tr!(
                    "client-broke-off",
                    host = self.address.display(),
                    why = e.to_string()
                ))
            })?;
        Ok((status, body))
    }

    /// The other AniBeams the host sees on the tailnet.
    pub fn peers(&self) -> Result<Vec<api::PeerHost>, HostError> {
        let (body, _) = self.get(api::PEERS, 256 * 1024)?;
        serde_json::from_slice::<api::Peers>(&body)
            .map(|p| p.hosts)
            .map_err(|e| HostError::Bad(tr!("client-peers-unreadable", why = e.to_string())))
    }

    /// The image bytes and their content type.
    pub fn poster(&self, series_id: u64) -> Result<(Vec<u8>, Option<String>), HostError> {
        self.get(&api::poster_path(series_id), POSTER_LIMIT)
    }
}

/// What a computer that refuses this device says: its `{"error": ...}` on a 403, the
/// answer every route gives a device switched off there.
pub fn refusal(status: u16, body: &[u8]) -> Option<String> {
    if status != 403 {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(body).ok()?;
    v["error"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod refusal_tests {
    use super::refusal;

    #[test]
    fn a_refusal_reads_as_what_the_computer_said() {
        let body = br#"{"error":"This device is switched off on that computer."}"#;
        assert_eq!(
            refusal(403, body).as_deref(),
            Some("This device is switched off on that computer.")
        );
        assert_eq!(refusal(404, body), None);
        assert_eq!(refusal(403, b"<html>"), None);
        assert_eq!(refusal(403, br#"{"error":""}"#), None);
    }
}

#[cfg(test)]
mod proof_tests {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;

    use anibeam_host_api::proof::{self, DeviceKey};

    use super::*;

    /// A host on loopback that answers `hello` for `count` requests, each on its own
    /// connection, and hands back each request's line and proof header.
    fn host(
        hello: String,
        count: usize,
    ) -> (HostAddress, mpsc::Receiver<(String, Option<String>)>) {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = HostAddress::parse(&l.local_addr().unwrap().to_string()).unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for stream in l.incoming().take(count) {
                let mut stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let mut proof = None;
                loop {
                    let mut header = String::new();
                    reader.read_line(&mut header).unwrap();
                    let header = header.trim_end();
                    if header.is_empty() {
                        break;
                    }
                    if let Some((name, value)) = header.split_once(": ")
                        && name.eq_ignore_ascii_case(proof::HEADER)
                    {
                        proof = Some(value.to_string());
                    }
                }
                tx.send((line.trim_end().to_string(), proof)).unwrap();
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{hello}",
                    hello.len()
                );
            }
        });
        (address, rx)
    }

    fn hello(features: &[&str], key: &str, now: i64) -> String {
        serde_json::json!({
            "app": "anibeam", "api": 1, "version": "2.5.0", "host": "banditbox",
            "now": now, "features": features, "key": key,
        })
        .to_string()
    }

    #[test]
    fn requests_carry_a_proof_the_host_can_check_once_it_reads_them() {
        let host_key = DeviceKey::from_seed([9; 32]).public();
        let phone = DeviceKey::from_seed([7; 32]);
        // The host's clock is an hour ahead: the proof is made on the host's time.
        let ahead = crate::now() + 3600;
        let (address, seen) = host(hello(&[api::FEATURE_PROOF], &host_key, ahead), 5);
        let client = HostClient::new(agent(), address.clone()).with_key(phone.clone());

        client.hello().unwrap();
        assert_eq!(
            seen.recv().unwrap().1,
            None,
            "nothing to sign to before the hello"
        );

        client.get_bytes("/api/v1/posters/3?x=1").unwrap();
        let (line, header) = seen.recv().unwrap();
        assert!(line.starts_with("GET /api/v1/posters/3?x=1 "), "{line}");
        let proven = proof::check(
            &header.unwrap(),
            &host_key,
            "GET",
            "/api/v1/posters/3?x=1",
            ahead,
        )
        .unwrap();
        assert_eq!(proven.key, phone.public());

        client.post(api::CALL, b"{}").unwrap();
        let (_, header) = seen.recv().unwrap();
        let again = proof::check(&header.unwrap(), &host_key, "POST", api::CALL, ahead).unwrap();
        assert_ne!(again.nonce, proven.nonce, "a nonce is used once");

        // A client made for one fetch, which never says hello itself, signs too.
        let fresh = HostClient::new(agent(), address).with_key(phone.clone());
        fresh.get_bytes("/api/v1/posters/4").unwrap();
        let (_, header) = seen.recv().unwrap();
        assert!(
            proof::check(
                &header.unwrap(),
                &host_key,
                "GET",
                "/api/v1/posters/4",
                ahead
            )
            .is_ok()
        );

        // Another hello goes unsigned, so a host whose key changed is heard again.
        client.hello().unwrap();
        assert_eq!(seen.recv().unwrap().1, None);
    }

    #[test]
    fn a_host_that_does_not_read_proofs_gets_none() {
        let (address, seen) = host(hello(&[], "", crate::now()), 2);
        let client = HostClient::new(agent(), address).with_key(DeviceKey::from_seed([7; 32]));
        client.hello().unwrap();
        client.get_bytes("/api/v1/snapshot").unwrap();
        assert_eq!(seen.recv().unwrap().1, None);
        assert_eq!(seen.recv().unwrap().1, None);
    }
}
