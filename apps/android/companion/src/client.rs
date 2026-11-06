//! HTTP to the host and to AniList.

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
    /// Something answered, but it is not AniBeam.
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
            HostError::NotAniBeam(what) => {
                write!(f, "Something answered, but not AniBeam ({what})")
            }
            HostError::TooNew(api) => {
                write!(
                    f,
                    "The host speaks API {api}; this app reads {}. Update the app.",
                    api::API_VERSION
                )
            }
            HostError::Bad(why) => write!(f, "{why}"),
        }
    }
}

/// Plain words for a failed request.
pub fn describe(error: &ureq::Error, host: &str) -> String {
    match error {
        ureq::Error::HostNotFound => {
            format!("{host} is not a name the phone can find. Is Tailscale connected?")
        }
        ureq::Error::Timeout(_) => {
            format!("{host} did not answer. Is it on, and is Tailscale connected on both?")
        }
        ureq::Error::ConnectionFailed => {
            format!("Could not reach {host}. Is it on, and is Tailscale connected on both?")
        }
        ureq::Error::Io(io) => match io.kind() {
            std::io::ErrorKind::ConnectionRefused => {
                format!(
                    "{host} is on, but AniBeam there is closed or not sharing. Open AniBeam on it and switch on Share over Tailscale in Settings, Phone app."
                )
            }
            std::io::ErrorKind::TimedOut => {
                format!("{host} did not answer. Is it on, and is Tailscale connected on both?")
            }
            std::io::ErrorKind::NetworkUnreachable | std::io::ErrorKind::HostUnreachable => {
                format!("No route to {host}. Is Tailscale connected?")
            }
            // What connect() says on Android for an address it will not even try.
            std::io::ErrorKind::InvalidInput => format!("Could not connect to {host}"),
            _ => format!("Could not reach {host}: {io}"),
        },
        other => format!("Could not reach {host}: {other}"),
    }
}

/// The phone's key, set once at start ([`use_device_key`]), that every client signs with.
static DEVICE: OnceLock<Arc<DeviceKey>> = OnceLock::new();

/// Signs every request to a host that reads proofs with `key` from now on (#555).
pub fn use_device_key(key: DeviceKey) {
    let _ = DEVICE.set(Arc::new(key));
}

pub struct HostClient {
    agent: ureq::Agent,
    address: HostAddress,
    key: Option<Arc<DeviceKey>>,
    /// The host's key and how far its clock is ahead of the phone's, once its hello said
    /// it reads proofs.
    host: Mutex<Option<(String, i64)>>,
}

impl HostClient {
    pub fn new(agent: ureq::Agent, address: HostAddress) -> Self {
        HostClient {
            agent,
            address,
            key: DEVICE.get().cloned(),
            host: Mutex::new(None),
        }
    }

    /// Signs with `key` instead of the phone's.
    pub fn with_key(mut self, key: DeviceKey) -> Self {
        self.key = Some(Arc::new(key));
        self
    }

    /// The proof header for one request, when the host reads proofs and the phone has a
    /// key. Made afresh for every try: a nonce is good once.
    fn proof(&self, method: &str, path: &str) -> Option<String> {
        let key = self.key.as_ref()?;
        let (host, ahead) = self.host.lock().ok()?.clone()?;
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

    fn get(&self, path: &str, limit: u64) -> Result<(Vec<u8>, Option<String>), HostError> {
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
                HostError::Bad(format!(
                    "The answer from {} broke off: {e}",
                    self.address.display()
                ))
            })?;
        if status != 200 {
            if let Some(why) = refusal(status, &body) {
                return Err(HostError::Bad(why));
            }
            return Err(HostError::Bad(format!(
                "{} answered {status} for {path}",
                self.address.display()
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
            _ => Err(HostError::Unreachable(format!(
                "The phone cannot find {}. Is Tailscale connected?",
                self.address.host
            ))),
        }
    }

    /// Asks who is there. Cheap, and the first thing every sync does.
    pub fn hello(&self) -> Result<Hello, HostError> {
        self.resolve()?;
        let (body, _) = self.get(api::HELLO, 64 * 1024).map_err(|e| match e {
            // A 404 on the hello route is a web server that is not AniBeam.
            HostError::Bad(why) if why.contains(" answered ") => HostError::NotAniBeam(why),
            other => other,
        })?;
        let hello: Hello = serde_json::from_slice(&body)
            .map_err(|_| HostError::NotAniBeam("its answer is not AniBeam's hello".into()))?;
        if hello.app != api::APP {
            return Err(HostError::NotAniBeam(format!(
                "it calls itself {}",
                hello.app
            )));
        }
        if hello.api > api::API_VERSION {
            return Err(HostError::TooNew(hello.api));
        }
        if let Ok(mut host) = self.host.lock() {
            *host = (hello.features.iter().any(|f| f == api::FEATURE_PROOF)
                && !hello.key.is_empty())
            .then(|| (hello.key.clone(), hello.now - crate::now()));
        }
        Ok(hello)
    }

    pub fn snapshot(&self) -> Result<Snapshot, HostError> {
        let (body, _) = self.get(api::SNAPSHOT, SNAPSHOT_LIMIT)?;
        serde_json::from_slice(&body)
            .map_err(|e| HostError::Bad(format!("The snapshot did not read: {e}")))
    }

    /// The body of any `GET` on the host, up to 1 MB.
    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>, HostError> {
        self.get(path, 1024 * 1024).map(|(body, _)| body)
    }

    /// What is in one of the host's episode files.
    pub fn probe(&self, file: u64) -> Result<api::Probe, HostError> {
        let (body, _) = self.get(&api::probe_path(file), 1024 * 1024)?;
        serde_json::from_slice(&body)
            .map_err(|e| HostError::Bad(format!("The file's details did not read: {e}")))
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
            None => Err(HostError::Unreachable(format!(
                "The phone cannot find {}. Is Tailscale connected?",
                self.address.host
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
                HostError::Bad(format!(
                    "The answer from {} broke off: {e}",
                    self.address.display()
                ))
            })?;
        Ok((status, body))
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
        let (address, seen) = host(hello(&[api::FEATURE_PROOF], &host_key, ahead), 3);
        let client = HostClient::new(agent(), address).with_key(phone.clone());

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
