//! The tailnet host: with "Share over Tailscale" on, the core answers a
//! read-only JSON API (`anibeam-host-api`) on this machine's Tailscale
//! addresses and nowhere else, for the phone app. Every device on the tailnet
//! is trusted unless the person switched it off (`trust`), and the host asks
//! who is calling before every request.
//!
//! A supervisor task owns the listening. It looks at the interfaces, binds
//! the Tailscale addresses it finds, and looks again every minute, so a
//! tailnet that comes up after the core (or an address that changes) is
//! picked up without a restart. Turning the preference off, or shutting the
//! core down, ends the task and the servers with it. `ANIBEAM_HOST_BIND`
//! (`ip:port`) replaces the address search, for development and tests.

pub mod addr;
pub mod build;
pub mod convert;
pub mod history;
pub mod keys;
pub mod peers;
pub mod server;
pub mod stream;
pub mod trust;

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use anibeam_host_api::DEFAULT_PORT;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::contract::*;
use crate::core::Core;
use crate::events::EventBus;
use crate::foundation::trust::{Asker, TailnetUser};
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
/// The env var that names a file read in place of `tailscale status --json`,
/// for development and captures.
pub const STATUS_ENV: &str = "ANIBEAM_TAILSCALE_STATUS";
/// How long a read of the tailnet answers every request.
const TAILNET_FOR: Duration = Duration::from_secs(30);
/// An address the last read could not place reads the tailnet again, but not
/// more often than this.
const TAILNET_AGAIN: Duration = Duration::from_secs(5);

pub struct Host {
    status: Arc<Status>,
    run: Mutex<Option<Run>>,
    /// A bind override a test sets on one core, where the env var would be
    /// shared by every test in the process.
    bind: Mutex<Option<String>>,
    /// Set by a shell that serves other devices whatever the preference
    /// says: the headless host service, and the desktop app while that
    /// service has stepped aside for it.
    held: AtomicBool,
    /// Whether the episodes themselves are served, from the preference.
    streaming: AtomicBool,
    /// The ids of the tailnet devices the person switched off.
    off: Mutex<Vec<String>>,
    /// When the tailnet was last read, and the last read that worked: a
    /// failed read keeps the one before it, so one hiccup of `tailscale`
    /// does not make every device unplaceable.
    tailnet: Mutex<(Option<Instant>, Option<Arc<trust::Tailnet>>)>,
    /// Held across a read, so a burst of requests runs `tailscale` once.
    reading: Mutex<()>,
    /// A status document a test sets on one core, read in place of the CLI.
    fixed_status: Mutex<Option<Vec<u8>>>,
    /// When each other computer's history was last pulled, by its base URL.
    pub(crate) pulled: Mutex<std::collections::HashMap<String, Instant>>,
    /// This machine's key and the keys it is paired with (#555).
    pub keys: keys::Keys,
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
        message: crate::i18n::tr("core-host-off", &[]),
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
            held: AtomicBool::new(false),
            streaming: AtomicBool::new(true),
            off: Mutex::new(Vec::new()),
            tailnet: Mutex::new((None, None)),
            reading: Mutex::new(()),
            fixed_status: Mutex::new(None),
            pulled: Mutex::new(std::collections::HashMap::new()),
            keys: keys::Keys::new(),
        }
    }

    /// Whether the host serves the episodes, as "Stream episodes" says.
    pub fn streaming(&self) -> bool {
        self.streaming.load(Ordering::SeqCst)
    }

    pub fn set_streaming(&self, on: bool) {
        self.streaming.store(on, Ordering::SeqCst);
    }

    /// The tailnet devices the person switched off, by Tailscale id.
    pub fn set_switched_off(&self, ids: Vec<String>) {
        *recover(&self.off) = ids;
    }

    /// For a test: read this status document instead of asking `tailscale`.
    pub fn set_status(&self, json: Option<Vec<u8>>) {
        *recover(&self.fixed_status) = json;
        *recover(&self.tailnet) = (None, None);
    }

    /// The tailnet as last read, read again when the read is older than
    /// `TAILNET_FOR`, or than `TAILNET_AGAIN` when `again` is set. Blocks
    /// while it runs `tailscale`, at most `NAME_LIMIT` per program.
    pub fn tailnet(&self, again: bool) -> Option<Arc<trust::Tailnet>> {
        let fresh_for = if again { TAILNET_AGAIN } else { TAILNET_FOR };
        let fresh = |slot: &(Option<Instant>, Option<Arc<trust::Tailnet>>)| {
            slot.0.is_some_and(|at| at.elapsed() < fresh_for)
        };
        {
            let slot = recover(&self.tailnet);
            if fresh(&slot) {
                return slot.1.clone();
            }
        }
        let _reading = recover(&self.reading);
        // Another request may have read it while this one waited.
        {
            let slot = recover(&self.tailnet);
            if fresh(&slot) {
                return slot.1.clone();
            }
        }
        let json = recover(&self.fixed_status).clone().or_else(status_json);
        let read = json.as_deref().and_then(trust::read_status).map(Arc::new);
        let mut slot = recover(&self.tailnet);
        slot.0 = Some(Instant::now());
        if read.is_some() {
            slot.1 = read;
        }
        slot.1.clone()
    }

    /// The tailnet as last read, without reading it: for a caller that must
    /// not wait, such as the Settings list.
    pub fn tailnet_now(&self) -> Option<Arc<trust::Tailnet>> {
        recover(&self.tailnet).1.clone()
    }

    /// The switched-off ids, for the peer list.
    pub fn switched_off(&self) -> Vec<String> {
        recover(&self.off).clone()
    }

    /// Whether to answer a request from `from` (`trust::decide`). An address
    /// the last read cannot place, while some device is switched off, reads
    /// the tailnet again first: it may have just joined.
    pub fn verdict(&self, from: std::net::IpAddr) -> trust::Verdict {
        let off = self.switched_off();
        // This machine and anything off the tailnet are decided by the
        // address alone, so they never wait on `tailscale`.
        if !trust::needs_tailnet(from, &off) {
            return trust::decide(from, &off, None);
        }
        // Before `tailscale` has ever answered, this machine still knows its own
        // tailnet addresses from its interfaces.
        let known = |t: Option<Arc<trust::Tailnet>>| {
            t.unwrap_or_else(|| {
                Arc::new(trust::local_tailnet(
                    &crate::foundation::machine::local_addresses(),
                ))
            })
        };
        let first = trust::decide(from, &off, Some(&known(self.tailnet(false))));
        if first != trust::Verdict::Unknown {
            return first;
        }
        trust::decide(from, &off, Some(&known(self.tailnet(true))))
    }

    /// Whose request it is (`trust::asker`), and this machine's own Tailscale user when
    /// the tailnet names one. A device the last read cannot place reads the tailnet
    /// again first: it may have just joined. This machine is the owner without a read.
    pub fn asker(&self, from: std::net::IpAddr) -> (Asker, Option<TailnetUser>) {
        let me = |t: &Option<Arc<trust::Tailnet>>| t.as_ref().and_then(|t| t.owner().cloned());
        if from.is_loopback() {
            return (Asker::Owner, me(&self.tailnet_now()));
        }
        let first = self.tailnet(false);
        let asked = trust::asker(from, first.as_deref());
        if asked != Asker::Unknown {
            return (asked, me(&first));
        }
        let again = self.tailnet(true);
        (trust::asker(from, again.as_deref()), me(&again))
    }

    pub fn status(&self) -> HostStatus {
        recover(&self.status.now).clone()
    }

    /// For a test: listen where the env var would say, for this core alone.
    pub fn set_bind(&self, bind: Option<String>) {
        *recover(&self.bind) = bind;
    }

    /// Keeps the host on whatever the preference says, or hands it back to
    /// the preference. Takes effect at the next `follow`.
    pub fn hold(&self, held: bool) {
        self.held.store(held, Ordering::SeqCst);
    }

    /// Starts or stops the supervisor to match the preference, or keeps it
    /// running while the host is held.
    pub fn follow(&self, core: &Core, preference: bool) {
        self.set_enabled(core, preference || self.held.load(Ordering::SeqCst));
    }

    /// Starts or stops the supervisor. Calling it with what already holds
    /// does nothing.
    fn set_enabled(&self, core: &Core, on: bool) {
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
                message: crate::i18n::tr("core-host-starting", &[]),
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
/// the core starts, and when a shell holds the host or lets it go.
pub fn sync(core: &Core) {
    keys::load(core);
    let on = match core.store.read(prefs::load_preferences) {
        Ok(p) => {
            core.host.set_streaming(p.stream_over_tailscale);
            core.host.set_switched_off(p.tailnet_switched_off);
            p.share_over_tailscale
        }
        Err(e) => {
            // A held host serves on through this, so streaming and the devices
            // switched off stay as they were rather than falling back to on and
            // everyone trusted.
            tracing::warn!(
                "host: the preferences could not be read, so the switches stay as they were: {e}"
            );
            false
        }
    };
    core.host.follow(core, on);
}

/// The devices on the tailnet for Settings, each with whether it is trusted and
/// whose profile it gets, and the people they belong to, as last read. It never
/// waits: a fresh read runs on a worker and arrives as `TailnetDevicesChanged`.
pub fn devices(core: &Core) -> (Vec<TailnetDevice>, Vec<TailnetPerson>) {
    if let Some(arc) = core.arc() {
        core.handle.spawn_blocking(move || {
            arc.host.tailnet(true);
            let (devices, people) = listed(&arc);
            arc.bus.debug(
                Stage::System,
                "tailnet devices",
                EventBody::TailnetDevicesChanged { devices, people },
            );
        });
    }
    listed(core)
}

/// `GetHost`: the host's state as last known.
pub(crate) fn status_call(core: &Core) -> Result<Reply, CoreError> {
    Ok(Reply::Host {
        status: core.host.status(),
    })
}

/// `ListTailnetDevices`: [`devices`] as the call's reply.
pub(crate) fn devices_call(core: &Core) -> Result<Reply, CoreError> {
    let (devices, people) = devices(core);
    Ok(Reply::TailnetDevices { devices, people })
}

/// `SetTailnetDeviceTrust`: switches one device off or back on, the only
/// write that changes which devices are off.
pub(crate) fn set_device_trust(
    core: &Core,
    device: String,
    trusted: bool,
) -> Result<Reply, CoreError> {
    let preferences = core.store.write(move |c| {
        let mut p = prefs::load_preferences(c)?;
        p.tailnet_switched_off.retain(|d| *d != device);
        if !trusted {
            p.tailnet_switched_off.push(device);
        }
        prefs::save_preferences(c, &p)?;
        Ok(p)
    })?;
    core.host
        .set_switched_off(preferences.tailnet_switched_off.clone());
    crate::app::preferences_changed(core, preferences)
}

/// Whether Tailscale carries the path to `from` through a relay, as the last
/// read says. Hello asks it, so it never waits on `tailscale`: a read older
/// than `TAILNET_AGAIN` is made on a worker for the next ask, and before any
/// read the answer is no, which plays the file as a host without this would.
pub fn relayed(core: &Core, from: std::net::IpAddr) -> bool {
    let from = trust::canonical(from);
    // Only a tailnet address has a path Tailscale can relay.
    if !addr::is_tailscale(&from) {
        return false;
    }
    if let Some(arc) = core.arc() {
        core.handle.spawn_blocking(move || {
            arc.host.tailnet(true);
        });
    }
    core.host
        .tailnet_now()
        .and_then(|t| t.peer(from).map(|p| p.relayed))
        .unwrap_or(false)
}

fn listed(core: &Core) -> (Vec<TailnetDevice>, Vec<TailnetPerson>) {
    let off = recover(&core.host.off).clone();
    core.host
        .tailnet_now()
        .map(|t| listing(&t, &off))
        .unwrap_or_default()
}

/// Every device with whose profile it gets, and every person the host answers with a
/// profile of their own (#633): the owner first, then the rest by name. A tagged device
/// is the owner's, as it is answered; one the tailnet ties to nobody is refused, and says
/// so. A person's count is of the devices the host serves, switched-off ones left out. A
/// tailnet that names no person for this machine lists no people, since the host then
/// cannot tell anyone apart.
fn listing(t: &trust::Tailnet, off: &[String]) -> (Vec<TailnetDevice>, Vec<TailnetPerson>) {
    // The owner, also when the status has no entry for their user.
    let owner = t.me_user.as_deref().map(|id| {
        t.user(id).cloned().unwrap_or_else(|| TailnetUser {
            id: id.to_string(),
            ..TailnetUser::default()
        })
    });
    let mut people: Vec<TailnetPerson> = owner.iter().map(|me| person(me, true)).collect();
    let devices = t
        .peers
        .iter()
        .map(|p| {
            let asker = trust::whose(t, Some(p));
            let at = match &asker {
                Asker::Owner if owner.is_some() => Some(0),
                Asker::Person(user) => Some(match people.iter().position(|q| q.id == user.id) {
                    Some(i) => i,
                    None => {
                        people.push(person(user, false));
                        people.len() - 1
                    }
                }),
                _ => None,
            };
            let trusted = !off.contains(&p.id);
            if let Some(i) = at
                && trusted
            {
                people[i].devices += 1;
            }
            TailnetDevice {
                id: p.id.clone(),
                name: p.name.clone(),
                os: p.os.clone(),
                online: p.online,
                trusted,
                person: at.map(|i| people[i].name.clone()).unwrap_or_default(),
                nobody: asker == Asker::Unknown,
            }
        })
        .collect();
    let first = usize::from(owner.is_some());
    people[first..].sort_by_key(|p| p.name.to_lowercase());
    (devices, people)
}

fn person(user: &TailnetUser, owner: bool) -> TailnetPerson {
    TailnetPerson {
        id: user.id.clone(),
        name: user.shown().map_or_else(
            || crate::i18n::tr("tailnet-person-no-name", &[]),
            str::to_string,
        ),
        login: user.login.clone(),
        owner,
        devices: 0,
    }
}

/// `tailscale status --json`, from the file `STATUS_ENV` names when it is
/// set, else from the CLI, waiting at most `NAME_LIMIT` for each program.
fn status_json() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var(STATUS_ENV) {
        return std::fs::read(path).ok();
    }
    for program in [
        "tailscale",
        "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
    ] {
        let Ok(mut child) = std::process::Command::new(program)
            .args(["status", "--json"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        else {
            continue;
        };
        // The document is read on its own thread, so a big tailnet cannot
        // fill the pipe and stall the CLI while this one waits for it.
        let reader = child.stdout.take().map(|mut out| {
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut out, &mut buf).map(|_| buf)
            })
        });
        let deadline = Instant::now() + NAME_LIMIT;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                _ => {
                    // Stuck on its daemon: ended through its own handle, which
                    // is not reaped yet, so no other process can be hit.
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
            }
        };
        let out = reader.and_then(|r| r.join().ok()).and_then(Result::ok);
        if let (Some(status), Some(out)) = (status, out)
            && status.success()
        {
            return Some(out);
        }
    }
    None
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
            message: crate::i18n::tr(
                "core-host-serving",
                &[("addresses", shown.join(", ").into())],
            ),
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
        match addr::wanted(
            bind.as_deref(),
            &crate::foundation::machine::local_addresses(),
        ) {
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
                status.put(
                    Some(&cancel),
                    waiting(&crate::i18n::tr("core-host-no-tailscale", &[])),
                );
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
                            // Serving on the tailnet: catch up with every
                            // trusted AniBeam that is up. Not on an address a
                            // test or the env var chose, which is no tailnet.
                            if bind.is_none()
                                && let Some(core) = core.upgrade()
                            {
                                history::pull_all(&core);
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
        return Err(failure.unwrap_or_else(|| crate::i18n::tr("core-host-cannot-listen", &[])));
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
                        let app = app.into_make_service_with_connect_info::<SocketAddr>();
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
                    failure = Some(crate::i18n::tr(
                        "core-host-port-taken",
                        &[("port", want.port().to_string().into())],
                    ));
                }
                Err(e) => {
                    failure = Some(crate::i18n::tr(
                        "core-host-cannot-listen-on",
                        &[
                            ("address", want.to_string().into()),
                            ("error", e.to_string().into()),
                        ],
                    ));
                }
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

    /// The owner has this machine, a phone and a tagged server's worth of trust; Sam has
    /// a tablet and a laptop, Alex a phone with no name in Tailscale, and one device
    /// names nobody.
    const STATUS: &[u8] = br#"{
        "Self": {"ID": "nSelf", "HostName": "desk", "UserID": 1, "TailscaleIPs": ["100.64.0.1"]},
        "Peer": {
            "a": {"ID": "nPhone", "HostName": "phone", "UserID": 1, "TailscaleIPs": ["100.64.0.2"]},
            "b": {"ID": "nTab", "HostName": "tab", "UserID": 3, "TailscaleIPs": ["100.64.0.3"]},
            "c": {"ID": "nLap", "HostName": "lap", "UserID": 3, "TailscaleIPs": ["100.64.0.4"]},
            "d": {"ID": "nAlex", "HostName": "alexphone", "UserID": 2, "TailscaleIPs": ["100.64.0.5"]},
            "e": {"ID": "nNas", "HostName": "nas", "UserID": 9, "Tags": ["tag:server"], "TailscaleIPs": ["100.64.0.6"]},
            "f": {"ID": "nOdd", "HostName": "odd", "TailscaleIPs": ["100.64.0.7"]}
        },
        "User": {
            "1": {"ID": 1, "LoginName": "owner@example.com", "DisplayName": "Owner"},
            "2": {"ID": 2, "LoginName": "alex@example.com", "DisplayName": ""},
            "3": {"ID": 3, "LoginName": "sam@example.com", "DisplayName": "Sam"},
            "9": {"ID": 9, "LoginName": "tagged-devices", "DisplayName": "Tagged Devices"}
        }
    }"#;

    #[test]
    fn each_device_is_listed_under_the_person_whose_profile_it_gets() {
        let t = trust::read_status(STATUS).unwrap();
        let (devices, people) = listing(&t, &["nLap".to_string()]);
        let whose: Vec<(&str, &str, bool, bool)> = devices
            .iter()
            .map(|d| (d.name.as_str(), d.person.as_str(), d.trusted, d.nobody))
            .collect();
        assert_eq!(
            whose,
            [
                ("alexphone", "alex@example.com", true, false),
                ("lap", "Sam", false, false),
                ("nas", "Owner", true, false),
                ("odd", "", true, true),
                ("phone", "Owner", true, false),
                ("tab", "Sam", true, false),
            ]
        );
        let listed: Vec<(&str, bool, u32)> = people
            .iter()
            .map(|p| (p.name.as_str(), p.owner, p.devices))
            .collect();
        assert_eq!(
            listed,
            [
                ("Owner", true, 2),
                ("alex@example.com", false, 1),
                ("Sam", false, 1),
            ],
            "the owner first, then the rest by name, each counting the devices served"
        );
    }

    #[test]
    fn the_owner_is_listed_without_an_entry_and_nobody_shows_a_bare_id() {
        // The status names this machine's user and the tablet's, but has no entry for
        // either: the owner is still first, and the friend is not called "3".
        let bare = String::from_utf8(STATUS.to_vec())
            .unwrap()
            .replace(r#""1": {"ID": 1"#, r#""101": {"ID": 101"#)
            .replace(r#""3": {"ID": 3"#, r#""103": {"ID": 103"#);
        let t = trust::read_status(bare.as_bytes()).unwrap();
        let (devices, people) = listing(&t, &[]);
        let no_name = crate::i18n::tr("tailnet-person-no-name", &[]);
        let listed: Vec<(&str, bool)> = people.iter().map(|p| (p.name.as_str(), p.owner)).collect();
        assert_eq!(
            listed,
            [
                (no_name.as_str(), true),
                ("alex@example.com", false),
                (no_name.as_str(), false),
            ]
        );
        let phone = devices.iter().find(|d| d.name == "phone").unwrap();
        assert_eq!(phone.person, no_name);
    }

    #[test]
    fn a_tailnet_that_names_nobody_for_this_machine_lists_no_people() {
        let tagged = String::from_utf8(STATUS.to_vec()).unwrap().replace(
            r#""UserID": 1, "TailscaleIPs": ["100.64.0.1"]"#,
            r#""Tags": ["tag:desk"], "TailscaleIPs": ["100.64.0.1"]"#,
        );
        let t = trust::read_status(tagged.as_bytes()).unwrap();
        assert_eq!(t.me_user, None);
        let (devices, people) = listing(&t, &[]);
        assert!(people.is_empty());
        assert!(devices.iter().all(|d| d.person.is_empty()));
    }

    #[test]
    fn preferences_that_cannot_be_read_leave_streaming_as_it_was() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::CorePaths::under(dir.path());
        let secrets = crate::trackers::Secrets::file_only(paths.secrets_path());
        let core = Core::open_with_secrets(paths, secrets).unwrap();
        let mut p = core.store.read(prefs::load_preferences).unwrap();
        p.stream_over_tailscale = false;
        core.call(Call::SetPreferences { preferences: p }).unwrap();
        assert!(!core.host.streaming());
        core.store
            .write(|c| {
                c.execute(
                    "UPDATE settings SET value = 'not json' WHERE key = 'preferences'",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        sync(&core);
        assert!(
            !core.host.streaming(),
            "a failed read turned streaming back on"
        );
    }

    #[test]
    fn preferences_that_cannot_be_read_leave_the_switched_off_list_alone() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::CorePaths::under(dir.path());
        let secrets = crate::trackers::Secrets::file_only(paths.secrets_path());
        let core = Core::open_with_secrets(paths, secrets).unwrap();
        core.call(Call::SetTailnetDeviceTrust {
            device: "nPhone".into(),
            trusted: false,
        })
        .unwrap();
        core.store
            .write(|c| {
                c.execute(
                    "UPDATE settings SET value = 'not json' WHERE key = 'preferences'",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        sync(&core);
        // A held host keeps serving through a failed read, so the list it
        // had must still stand: never fall back to trusting everyone.
        assert_eq!(*recover(&core.host.off), ["nPhone"]);
    }

    #[test]
    fn the_dns_name_comes_out_of_the_status_document() {
        let doc = br#"{"Version":"1.80","Self":{"DNSName":"desk.example-tailnet.ts.net.","HostName":"desk"}}"#;
        assert_eq!(
            dns_name_of(doc).as_deref(),
            Some("desk.example-tailnet.ts.net")
        );
        assert_eq!(dns_name_of(br#"{"Self":{"DNSName":""}}"#), None);
        assert_eq!(dns_name_of(br#"{"Self":{}}"#), None);
        assert_eq!(dns_name_of(b"not json"), None);
    }
}
