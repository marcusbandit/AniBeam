//! HTTP to the host and to AniList.

use std::time::Duration;

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
            HostError::NotAniBeam(what) => write!(f, "Something answered, but not AniBeam ({what})"),
            HostError::TooNew(api) => {
                write!(f, "The host speaks API {api}; this app reads {}. Update the app.", api::API_VERSION)
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
        ureq::Error::Timeout(_) => format!("{host} did not answer. Is it on, and is Tailscale connected on both?"),
        ureq::Error::ConnectionFailed => format!("Could not reach {host}. Is it on, and is Tailscale connected on both?"),
        ureq::Error::Io(io) => match io.kind() {
            std::io::ErrorKind::ConnectionRefused => {
                format!(
                    "{host} is on, but AniBeam there is closed or not sharing. Open AniBeam on it and switch on Share over Tailscale in Settings, Phone app."
                )
            }
            std::io::ErrorKind::TimedOut => format!("{host} did not answer in time"),
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

pub struct HostClient {
    agent: ureq::Agent,
    address: HostAddress,
}

impl HostClient {
    pub fn new(agent: ureq::Agent, address: HostAddress) -> Self {
        HostClient { agent, address }
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
        let mut response = match self.agent.get(&url).call() {
            Err(e) if dropped(&e) => self.agent.get(&url).call(),
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
            .map_err(|e| HostError::Bad(format!("The answer from {} broke off: {e}", self.address.display())))?;
        if status != 200 {
            return Err(HostError::Bad(format!("{} answered {status} for {path}", self.address.display())));
        }
        Ok((body, kind))
    }

    /// Looks the name up first, because a failed lookup reaches ureq as a bare
    /// "Invalid argument" on Android.
    fn resolve(&self) -> Result<(), HostError> {
        use std::net::ToSocketAddrs;
        let host = self.address.host.trim_start_matches('[').trim_end_matches(']');
        let found = (host, self.address.port).to_socket_addrs().map(|mut all| all.next().is_some());
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
            return Err(HostError::NotAniBeam(format!("it calls itself {}", hello.app)));
        }
        if hello.api > api::API_VERSION {
            return Err(HostError::TooNew(hello.api));
        }
        Ok(hello)
    }

    pub fn snapshot(&self) -> Result<Snapshot, HostError> {
        let (body, _) = self.get(api::SNAPSHOT, SNAPSHOT_LIMIT)?;
        serde_json::from_slice(&body).map_err(|e| HostError::Bad(format!("The snapshot did not read: {e}")))
    }

    /// The body of any `GET` on the host, up to 1 MB.
    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>, HostError> {
        self.get(path, 1024 * 1024).map(|(body, _)| body)
    }

    /// What is in one of the host's episode files.
    pub fn probe(&self, file: u64) -> Result<api::Probe, HostError> {
        let (body, _) = self.get(&api::probe_path(file), 1024 * 1024)?;
        serde_json::from_slice(&body).map_err(|e| HostError::Bad(format!("The file's details did not read: {e}")))
    }

    /// The host's address with the name looked up, for a player that resolves names on
    /// its own and may not ask the tailnet's DNS: `http://100.94.40.21:8790`.
    pub fn stream_base(&self) -> Result<String, HostError> {
        use std::net::ToSocketAddrs;
        let host = self.address.host.trim_start_matches('[').trim_end_matches(']');
        let found = (host, self.address.port).to_socket_addrs().ok().and_then(|mut all| all.next());
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
        let mut response = self
            .agent
            .post(&url)
            .header("content-type", "application/json")
            .send(body)
            .map_err(|e| HostError::Unreachable(describe(&e, &self.address.display())))?;
        let status = response.status().as_u16();
        let body = response
            .body_mut()
            .with_config()
            .limit(4 * 1024 * 1024)
            .read_to_vec()
            .map_err(|e| HostError::Bad(format!("The answer from {} broke off: {e}", self.address.display())))?;
        Ok((status, body))
    }

    /// The image bytes and their content type.
    pub fn poster(&self, series_id: u64) -> Result<(Vec<u8>, Option<String>), HostError> {
        self.get(&api::poster_path(series_id), POSTER_LIMIT)
    }
}
