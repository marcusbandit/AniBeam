//! The tailnet host: with "Share over Tailscale" on, the core answers a
//! read-only JSON API (`anibeam-host-api`) on this machine's Tailscale
//! addresses and nowhere else, for the phone app. The tailnet is the access
//! control, so nothing here asks who is calling.
//!
//! A supervisor task owns the listening. It looks at the interfaces, binds
//! the Tailscale addresses it finds, and looks again every minute, so a
//! tailnet that comes up after the core (or an address that changes) is
//! picked up without a restart. Turning the preference off, or shutting the
//! core down, ends the task and the servers with it. `ANIBEAM_HOST_BIND`
//! (`ip:port`) replaces the address search, for development and tests.

pub mod addr;
pub mod build;
pub mod server;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use anibeam_host_api::DEFAULT_PORT;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::contract::*;
use crate::core::Core;
use crate::events::EventBus;
use crate::prefs;
use crate::sync::recover;

/// How often the supervisor looks again: for Tailscale when it was not
/// there, for a changed address when it was.
const RECHECK: Duration = Duration::from_secs(60);
/// How long a stopping server gets to finish its answers before it is cut.
const GRACE: Duration = Duration::from_secs(2);
/// A port the stopped servers still hold is tried this often, this far
/// apart: a little past `GRACE` in all.
const BIND_TRIES: u32 = 6;
const BIND_PAUSE: Duration = Duration::from_millis(500);
/// How long `tailscale status` gets to name this machine.
const NAME_LIMIT: Duration = Duration::from_secs(2);

/// The env var that names the one address to listen on.
pub const BIND_ENV: &str = "ANIBEAM_HOST_BIND";

pub struct Host {
    status: Arc<Status>,
    run: Mutex<Option<Run>>,
    /// A bind override a test sets on one core, where the env var would be
    /// shared by every test in the process.
    bind: Mutex<Option<String>>,
}

struct Run {
    cancel: CancellationToken,
    task: JoinHandle<()>,
}

/// The status and the bus that hears it change.
struct Status {
    now: Mutex<HostStatus>,
    bus: Arc<EventBus>,
}

impl Status {
    /// Records `next` and says so, unless it is what already stands or the
    /// run that wants to say it has been cancelled. The check is made under
    /// the lock `Host::set_enabled` takes to switch it off, so a task that
    /// is being stopped cannot write over "Off".
    fn put(&self, cancel: Option<&CancellationToken>, next: HostStatus) {
        let mut now = recover(&self.now);
        if cancel.is_some_and(CancellationToken::is_cancelled) || *now == next {
            return;
        }
        *now = next.clone();
        drop(now);
        self.bus.info(
            Stage::System,
            format!("Tailnet host: {}", next.message),
            EventBody::HostChanged { status: next },
        );
    }
}

fn off() -> HostStatus {
    HostStatus {
        enabled: false,
        urls: Vec::new(),
        message: "Off".to_string(),
    }
}

impl Host {
    pub fn new(bus: Arc<EventBus>) -> Host {
        Host {
            status: Arc::new(Status {
                now: Mutex::new(off()),
                bus,
            }),
            run: Mutex::new(None),
            bind: Mutex::new(None),
        }
    }

    pub fn status(&self) -> HostStatus {
        recover(&self.status.now).clone()
    }

    /// For a test: listen where the env var would say, for this core alone.
    pub fn set_bind(&self, bind: Option<String>) {
        *recover(&self.bind) = bind;
    }

    /// Starts or stops the supervisor to match the preference. Calling it
    /// with what already holds does nothing.
    pub fn set_enabled(&self, core: &Core, on: bool) {
        let mut run = recover(&self.run);
        if on == run.is_some() {
            return;
        }
        if !on {
            if let Some(run) = run.take() {
                run.cancel.cancel();
                // The task winds its servers down by itself, and the runtime
                // reaps it; nothing here waits on it.
                drop(run.task);
            }
            self.status.put(None, off());
            return;
        }
        let Some(owner) = core.arc() else { return };
        let cancel = CancellationToken::new();
        self.status.put(
            Some(&cancel),
            HostStatus {
                enabled: true,
                urls: Vec::new(),
                message: "Starting".to_string(),
            },
        );
        let bind = recover(&self.bind)
            .clone()
            .or_else(|| std::env::var(BIND_ENV).ok());
        let task = core.handle.spawn(supervise(
            Arc::downgrade(&owner),
            self.status.clone(),
            cancel.clone(),
            bind,
        ));
        *run = Some(Run { cancel, task });
    }

    /// The core is going away: end the supervisor and its servers.
    pub fn stop(&self) {
        if let Some(run) = recover(&self.run).take() {
            run.cancel.cancel();
            drop(run.task);
        }
    }
}

/// Reads the preference and starts or stops the host to match. Called when
/// the core starts.
pub fn sync(core: &Core) {
    let on = core
        .store
        .read(prefs::load_preferences)
        .map(|p| p.share_over_tailscale)
        .unwrap_or(false);
    core.host.set_enabled(core, on);
}

/// What the supervisor is serving right now.
struct Serving {
    wanted: Vec<SocketAddr>,
    /// The wanted addresses that took; the rest are tried again each check.
    took: Vec<SocketAddr>,
    bound: Vec<SocketAddr>,
    router: axum::Router,
    name: Option<String>,
    token: CancellationToken,
    tasks: Vec<JoinHandle<()>>,
}

impl Serving {
    async fn stop(self) {
        self.token.cancel();
        for task in self.tasks {
            let abort = task.abort_handle();
            if tokio::time::timeout(GRACE, task).await.is_err() {
                abort.abort();
            }
        }
    }

    fn status(&self) -> HostStatus {
        let mut urls: Vec<String> = self.bound.iter().map(addr::url).collect();
        if let Some(name) = &self.name {
            let port = self.bound.first().map_or(DEFAULT_PORT, |a| a.port());
            urls.push(addr::name_url(name, port));
        }
        let shown: Vec<String> = self.bound.iter().map(addr::shown).collect();
        HostStatus {
            enabled: true,
            urls,
            message: format!("Serving on {}", shown.join(", ")),
        }
    }
}

async fn supervise(
    core: Weak<Core>,
    status: Arc<Status>,
    cancel: CancellationToken,
    bind: Option<String>,
) {
    let waiting = |message: &str| HostStatus {
        enabled: true,
        urls: Vec::new(),
        message: message.to_string(),
    };
    let mut serving: Option<Serving> = None;
    loop {
        match addr::wanted(bind.as_deref(), &addr::local_addresses()) {
            Err(message) => {
                if let Some(old) = serving.take() {
                    old.stop().await;
                }
                status.put(Some(&cancel), waiting(&message));
            }
            Ok(wanted) if wanted.is_empty() => {
                if let Some(old) = serving.take() {
                    old.stop().await;
                }
                status.put(Some(&cancel), waiting("Tailscale is not connected"));
            }
            Ok(wanted) => {
                if let Some(s) = serving.as_mut()
                    && s.wanted == wanted
                    && s.took.len() < wanted.len()
                {
                    let missing: Vec<SocketAddr> = wanted
                        .iter()
                        .filter(|w| !s.took.contains(w))
                        .copied()
                        .collect();
                    if s.bind(&missing).await.0 > 0 {
                        status.put(Some(&cancel), s.status());
                    }
                }
                if serving.as_ref().is_none_or(|s| s.wanted != wanted) {
                    if let Some(old) = serving.take() {
                        old.stop().await;
                    }
                    match listen(&core, wanted).await {
                        Ok(now) => {
                            status.put(Some(&cancel), now.status());
                            serving = Some(now);
                            if let Some(s) = serving.as_mut()
                                && bind.is_none()
                                && let Some(name) = magic_dns_name().await
                            {
                                s.name = Some(name);
                                status.put(Some(&cancel), s.status());
                            }
                        }
                        Err(message) => status.put(Some(&cancel), waiting(&message)),
                    }
                }
            }
        }
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(RECHECK) => {}
        }
    }
    if let Some(old) = serving.take() {
        old.stop().await;
    }
}

/// Binds every wanted address and serves on each that took. Fails with the
/// sentence for the status only when none did.
async fn listen(core: &Weak<Core>, wanted: Vec<SocketAddr>) -> Result<Serving, String> {
    let mut serving = Serving {
        wanted: wanted.clone(),
        took: Vec::new(),
        bound: Vec::new(),
        router: server::router(core.clone()),
        name: None,
        token: CancellationToken::new(),
        tasks: Vec::new(),
    };
    let (_, failure) = serving.bind(&wanted).await;
    if serving.bound.is_empty() {
        return Err(failure.unwrap_or_else(|| "Could not listen".to_string()));
    }
    Ok(serving)
}

impl Serving {
    /// Binds each of `wants` and serves on it. A port still held by the
    /// servers just stopped is tried again for a little longer than their
    /// grace, so switching sharing off and on does not wait a whole check.
    /// Returns how many took, and the sentence for the last that did not.
    async fn bind(&mut self, wants: &[SocketAddr]) -> (usize, Option<String>) {
        let mut failure = None;
        let mut took = 0;
        for want in wants {
            let mut tries = 0;
            let listener = loop {
                match TcpListener::bind(want).await {
                    Ok(l) => break Ok(l),
                    Err(e) if e.kind() == std::io::ErrorKind::AddrInUse && tries < BIND_TRIES => {
                        tries += 1;
                        tokio::time::sleep(BIND_PAUSE).await;
                    }
                    Err(e) => break Err(e),
                }
            };
            match listener {
                Ok(listener) => {
                    let at = listener.local_addr().unwrap_or(*want);
                    let stop = self.token.clone();
                    let app = self.router.clone();
                    self.tasks.push(tokio::spawn(async move {
                        let served = axum::serve(listener, app)
                            .with_graceful_shutdown(async move { stop.cancelled().await })
                            .await;
                        if let Err(e) = served {
                            tracing::warn!("host server on {at} ended: {e}");
                        }
                    }));
                    self.took.push(*want);
                    self.bound.push(at);
                    took += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                    failure = Some(format!("Port {} is taken", want.port()));
                }
                Err(e) => failure = Some(format!("Could not listen on {want}: {e}")),
            }
        }
        (took, failure)
    }
}

/// This machine's MagicDNS name from `tailscale status --json`, when the
/// CLI is there and answers quickly. The macOS app keeps its CLI inside the
/// bundle. Any failure is simply no name.
async fn magic_dns_name() -> Option<String> {
    for program in [
        "tailscale",
        "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
    ] {
        let run = tokio::process::Command::new(program)
            .args(["status", "--json"])
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .output();
        let Ok(Ok(out)) = tokio::time::timeout(NAME_LIMIT, run).await else {
            continue;
        };
        if let Some(name) = dns_name_of(&out.stdout) {
            return Some(name);
        }
    }
    None
}

/// `Self.DNSName` out of the status document, without its trailing dot.
fn dns_name_of(json: &[u8]) -> Option<String> {
    let doc: serde_json::Value = serde_json::from_slice(json).ok()?;
    let name = doc.get("Self")?.get("DNSName")?.as_str()?;
    let name = name.trim_end_matches('.');
    (!name.is_empty()).then(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dns_name_comes_out_of_the_status_document() {
        let doc = br#"{"Version":"1.80","Self":{"DNSName":"banditbox.tail24c6a9.ts.net.","HostName":"banditbox"}}"#;
        assert_eq!(
            dns_name_of(doc).as_deref(),
            Some("banditbox.tail24c6a9.ts.net")
        );
        assert_eq!(dns_name_of(br#"{"Self":{"DNSName":""}}"#), None);
        assert_eq!(dns_name_of(br#"{"Self":{}}"#), None);
        assert_eq!(dns_name_of(b"not json"), None);
    }
}
