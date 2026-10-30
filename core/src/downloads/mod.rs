//! The downloads domain: searching release feeds, narrowing a result set,
//! subscribing and downloading through anirss, and watching what the
//! torrent client is doing. anirss is the tool that owns every rule about
//! feeds and qBittorrent; this module is its client, and the torrent
//! client's for what anirss leaves out (live transfer, stop, start,
//! delete).
//!
//! The core runs anirss as a sidecar by default (`anirss --serve` on a
//! port and token it picked), started on first use rather than at launch,
//! and stopped with the core. A sidecar that dies is started again on the
//! next call. The state here is the connection's: the sidecar, the last
//! status read, the endpoints, the torrent client's session. Nothing about
//! a search: anirss keeps no search state and neither does the core; the
//! shell holds `{endpoint, query, items}` and its own history.

pub mod anirss;
pub mod contents;
pub mod facts;
pub mod feed;
pub mod jackett;
pub mod link;
pub mod managed;
pub mod prune;
pub mod qbt;
pub mod replace;
pub mod rewire;
pub mod rolling;
pub mod settings;
pub mod trash;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use std::time::Duration;

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::net::{Http, HttpRequest, Method};
use crate::sync::recover;
use crate::trackers::accounts::with_secrets;
use anirss::{AnirssClient, Sidecar, SpawnFailure};
use qbt::QbtClient;

pub struct Downloads {
    state: Mutex<State>,
    /// One sidecar start at a time: two jobs that both find it down must
    /// not each spawn one.
    connecting: tokio::sync::Mutex<()>,
    /// The same for settling the torrent client and starting AniBeam's own.
    torrent_setup: tokio::sync::Mutex<()>,
    /// Set by a shell that runs against a copy of the library but the
    /// owner's real anirss (a drive script, a screenshot, a sandbox root):
    /// finished feeds are never pruned there.
    unattended_off: AtomicBool,
}

#[derive(Default)]
struct State {
    sidecar: Option<Sidecar>,
    anirss: AnirssStatus,
    torrent: TorrentClientStatus,
    endpoints: Vec<SearchEndpoint>,
    /// The torrent client with its session, built from the settings on
    /// first use and dropped when they change or a test is asked for.
    qbt: Option<Arc<QbtClient>>,
    /// Bumped by every settings change. A sidecar or a session built from
    /// settings read before the bump belongs to the old client and is
    /// thrown away rather than kept.
    generation: u64,
    /// The episodes an unfinished torrent is still writing, by series, with
    /// how far each is (0 to 1), as of the last torrent read: on disk is not
    /// in the library until the torrent is done with them (`overlay`).
    downloading: HashMap<u64, Downloading>,
    /// The files an unfinished torrent is still writing, by `path_key`: what
    /// Play refuses. An episode can be downloading while an older, whole
    /// file of it (one being replaced) stays playable.
    writing: std::collections::HashSet<std::path::PathBuf>,
    /// What each series' downloads looked like when its last `SeriesChanged`
    /// went out, and when: a progress tick only goes out again after
    /// `PROGRESS_EVERY`, a change of which episodes are downloading at once.
    announced: HashMap<u64, (std::time::Instant, BTreeMap<u32, u8>)>,
    /// A re-read of the torrents is already waiting its turn: while anything
    /// downloads the core looks again every `REREAD_EVERY` by itself, so
    /// the pages that show progress need no poll of their own.
    rereading: bool,
    /// When the torrents were last read, so a library read can ask for a
    /// fresh look without asking on every read.
    torrents_at: Option<std::time::Instant>,
}

impl Default for Downloads {
    fn default() -> Self {
        Downloads {
            state: Mutex::new(State::default()),
            connecting: tokio::sync::Mutex::new(()),
            torrent_setup: tokio::sync::Mutex::new(()),
            unattended_off: AtomicBool::new(false),
        }
    }
}

impl Downloads {
    fn state(&self) -> MutexGuard<'_, State> {
        recover(&self.state)
    }

    pub(crate) fn disable_unattended(&self) {
        self.unattended_off.store(true, Ordering::Relaxed);
    }

    fn unattended_allowed(&self) -> bool {
        !self.unattended_off.load(Ordering::Relaxed)
    }

    /// Kills the sidecar. `Core::shutdown` calls it; `kill_on_drop` is the
    /// backstop for a core dropped without one.
    pub fn stop(&self) {
        if let Some(mut sc) = self.state().sidecar.take() {
            sc.stop();
        }
    }
}

// The reads ------------------------------------------------------------------

/// The status off the state as it stands: settings from the store, the
/// rest from the last connect. No network.
pub fn status(core: &Core) -> Result<DownloadsStatus, CoreError> {
    let secrets = core.secrets().clone();
    let settings = core.store.read(move |c| settings::load(c, &secrets))?;
    Ok(assemble(core, settings))
}

/// `status` for a job: the keyring look goes to a blocking thread.
async fn status_async(core: &Arc<Core>) -> Result<DownloadsStatus, CoreError> {
    let settings = load_settings(core).await?;
    Ok(assemble(core, settings))
}

fn assemble(core: &Core, settings: DownloadSettings) -> DownloadsStatus {
    let st = core.downloads.state();
    let mut anirss = st.anirss.clone();
    anirss.config_path = settings::anirss_config_path()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    DownloadsStatus {
        settings,
        anirss,
        torrent: st.torrent.clone(),
        endpoints: st.endpoints.clone(),
    }
}

async fn load_settings(core: &Arc<Core>) -> Result<DownloadSettings, CoreError> {
    let mut s = core.store.read(settings::load_stored)?;
    let mut keys = vec![settings::ANIRSS_TOKEN.to_string()];
    keys.extend(settings::active_password_key(&s));
    keys.extend(
        s.saved_clients
            .iter()
            .map(|c| settings::password_key(&c.id)),
    );
    let held = with_secrets(core, move |sec| {
        Ok(keys.into_iter().filter(|k| sec.has(k)).collect::<Vec<_>>())
    })
    .await?;
    settings::fill_secret_flags(&mut s, |key| held.iter().any(|h| h == key));
    Ok(s)
}

// The clients ----------------------------------------------------------------

/// The anirss client when one is already there: the remote server, or a
/// sidecar that is alive. Starts nothing.
async fn running_client(core: &Arc<Core>) -> Result<Option<AnirssClient>, CoreError> {
    let s = load_settings(core).await?;
    if s.anirss_mode == AnirssMode::Remote {
        return ensure_client(core).await.map(Some);
    }
    let mut st = core.downloads.state();
    let Some(sc) = st.sidecar.as_mut() else {
        return Ok(None);
    };
    Ok(sc
        .alive()
        .then(|| AnirssClient::new(&sc.server(), &sc.token, core.http.clone())))
}

/// The anirss client, starting the sidecar when that is the mode and it is
/// not up. The status in the state follows every step, so a shell that
/// reads `GetDownloads` after a failure sees why.
async fn ensure_client(core: &Arc<Core>) -> Result<AnirssClient, CoreError> {
    let s = load_settings(core).await?;
    if s.anirss_mode == AnirssMode::Remote {
        let token = with_secrets(core, |sec| {
            Ok(sec
                .get(settings::ANIRSS_TOKEN, None)?
                .map(|(v, _)| v)
                .unwrap_or_default())
        })
        .await?;
        {
            let mut st = core.downloads.state();
            st.anirss.url = s.anirss_url.clone();
        }
        return Ok(AnirssClient::new(&s.anirss_url, &token, core.http.clone()));
    }
    let one_at_a_time = core.downloads.connecting.lock().await;
    let generation = {
        let mut st = core.downloads.state();
        if let Some(sc) = st.sidecar.as_mut() {
            if sc.alive() {
                return Ok(AnirssClient::new(
                    &sc.server(),
                    &sc.token,
                    core.http.clone(),
                ));
            }
            st.sidecar = None;
            st.anirss.state = AnirssState::Failed;
            st.anirss.message = "anirss stopped".to_string();
        }
        st.anirss.state = AnirssState::Starting;
        st.anirss.message.clear();
        st.anirss.url.clear();
        st.generation
    };
    // anirss is pointed at the client the core uses, so that has to be
    // settled first. It still starts without one: the page then says why
    // the torrent half is down.
    let s = match ensure_torrent(core).await {
        Ok(settled) => settled,
        Err(_) => load_settings(core).await?,
    };
    let (password, _) = resolve_password(core, &s).await?;
    let command = anirss::sidecar_command(&s.anirss_command);
    let login = anirss::QbtLogin {
        url: &s.torrent_url,
        username: &s.torrent_username,
        password: password.as_deref(),
    };
    let mut sidecar = match Sidecar::spawn(&command, &login) {
        Ok(sc) => sc,
        Err(SpawnFailure::Missing) => {
            let hint = format!(
                "{}. Or point Settings at a running server.",
                anirss::install_hint()
            );
            // The shell already says anirss is not installed; a command
            // the user typed in is worth naming, the default is not.
            let message = if s.anirss_command == "anirss" {
                hint
            } else {
                format!("{} was not found. {hint}", s.anirss_command)
            };
            set_anirss_failure(core, AnirssState::Missing, &message);
            return Err(service(anirss::SERVICE, "missing", message));
        }
        Err(SpawnFailure::Io(e)) => {
            let message = format!("{} could not start: {e}", s.anirss_command);
            set_anirss_failure(core, AnirssState::Failed, &message);
            return Err(service(anirss::SERVICE, "failed", message));
        }
    };
    let client = AnirssClient::new(&sidecar.server(), &sidecar.token, core.http.clone());
    match sidecar.wait_ready(&client).await {
        // One that serves but does not say it keeps each client's login apart
        // is not used: it would carry one client's login to another. Its
        // version is not the question; a fork or a checkout can say anything.
        Ok(health) if !health.has(anirss::SIDECAR_FEATURE) => {
            sidecar.stop();
            let message = anirss::lacks_feature_message(&health.version);
            set_anirss_failure(core, AnirssState::Failed, &message);
            Err(service(anirss::SERVICE, "failed", message))
        }
        Ok(health) => {
            let kept = {
                let mut st = core.downloads.state();
                let current = st.generation == generation;
                if current {
                    st.anirss.state = AnirssState::Ready;
                    st.anirss.version = health.version;
                    st.anirss.url = sidecar.server();
                    st.anirss.message.clear();
                    st.sidecar = Some(sidecar);
                } else {
                    // The client was switched while this one started: it
                    // points at the old one.
                    sidecar.stop();
                }
                current
            };
            if !kept {
                // Start again for the one now in use
                drop(one_at_a_time);
                return Box::pin(ensure_client(core)).await;
            }
            Ok(client)
        }
        Err(message) => {
            // An anirss from before the server mode exits on `--serve`
            // with a usage line that says nothing about versions.
            let message = match anirss::installed_version(&command)
                .await
                .and_then(|text| anirss::too_old(&text))
            {
                Some(version) => anirss::too_old_message(&version),
                None => message,
            };
            set_anirss_failure(core, AnirssState::Failed, &message);
            Err(service(anirss::SERVICE, "failed", message))
        }
    }
}

fn set_anirss_failure(core: &Core, state: AnirssState, message: &str) {
    let mut st = core.downloads.state();
    st.anirss.state = state;
    st.anirss.message = message.to_string();
    st.anirss.version.clear();
    st.anirss.url.clear();
    st.endpoints.clear();
}

/// The password of the client in use and where it came from: the one saved
/// here for it, else the one anirss's CLI saved, else none. anirss's is only
/// borrowed for the client at the address anirss's own config names: it is
/// that server's password, and no other client's business.
async fn resolve_password(
    core: &Arc<Core>,
    s: &DownloadSettings,
) -> Result<(Option<String>, &'static str), CoreError> {
    let Some(key) = settings::active_password_key(s) else {
        return Ok((None, "none"));
    };
    let saved = with_secrets(core, move |sec| Ok(sec.get(&key, None)?.map(|(v, _)| v))).await?;
    if let Some(pw) = saved {
        return Ok((Some(pw), "anibeam"));
    }
    let anirss_url = settings::anirss_config_path().and_then(|p| settings::anirss_config_qbt(&p));
    let borrowed = anirss_url
        .filter(|(url, _)| settings::same_address(url, &s.torrent_url))
        .and_then(|_| settings::anirss_saved_password());
    match borrowed {
        Some(pw) => Ok((Some(pw), "anirss")),
        None => Ok((None, "none")),
    }
}

/// The torrent client, built on first use and kept with its session.
async fn qbt_client(core: &Arc<Core>) -> Result<Arc<QbtClient>, CoreError> {
    let generation = {
        let st = core.downloads.state();
        if let Some(c) = st.qbt.clone() {
            return Ok(c);
        }
        st.generation
    };
    let s = ensure_torrent(core).await?;
    let (password, source) = resolve_password(core, &s).await?;
    let client = Arc::new(QbtClient::new(
        &s.torrent_url,
        &s.torrent_username,
        password.as_deref().unwrap_or(""),
        core.http.clone(),
    ));
    let kept = {
        let mut st = core.downloads.state();
        // One built from settings a switch has since replaced is not kept
        let current = st.generation == generation;
        if current {
            st.torrent.password_source = source.to_string();
            st.qbt = Some(client.clone());
        }
        current
    };
    if !kept {
        return Box::pin(qbt_client(core)).await;
    }
    Ok(client)
}

/// How long a client gets to answer when the core is only asking whether
/// one is there.
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Settles whose torrent client the downloads use and, when it is AniBeam's
/// own, has it running. A fresh install decides here, once: a qBittorrent
/// that answers at the address wins, else AniBeam sets up its own, and the
/// choice is saved.
async fn ensure_torrent(core: &Arc<Core>) -> Result<DownloadSettings, CoreError> {
    let _one_at_a_time = core.downloads.torrent_setup.lock().await;
    let mut s = load_settings(core).await?;
    if s.torrent_mode == TorrentMode::Auto {
        s = decide_torrent(core, s).await?;
    }
    if s.torrent_mode == TorrentMode::Managed {
        start_managed(core, &s).await?;
    }
    Ok(s)
}

async fn decide_torrent(
    core: &Arc<Core>,
    mut s: DownloadSettings,
) -> Result<DownloadSettings, CoreError> {
    // AniBeam's own, set up on an earlier day, is still AniBeam's: asked to
    // pick again, it must not mistake its own Web UI for the user's client.
    let own_port = managed::configured_port(&managed::profile_dir(&core.paths.data_dir));
    if let Some(port) = own_port {
        let own = format!("http://127.0.0.1:{port}");
        let probe = QbtClient::new(&own, managed::USERNAME, "", core.http.clone());
        let running = matches!(
            tokio::time::timeout(PROBE_TIMEOUT, probe.version()).await,
            Ok(Ok(_))
        );
        if running || s.torrent_url == own {
            s.torrent_url = own;
        }
    }
    let (password, _) = resolve_password(core, &s).await?;
    let probe = QbtClient::new(
        &s.torrent_url,
        &s.torrent_username,
        password.as_deref().unwrap_or(""),
        core.http.clone(),
    );
    let answered = match tokio::time::timeout(PROBE_TIMEOUT, probe.version()).await {
        Ok(Ok(_)) => true,
        // A login it refused still came from qBittorrent
        Ok(Err(CoreError::Service { code, .. })) => code == "unauthorized" || code == "banned",
        _ => false,
    };
    let is_own = own_port.is_some_and(|port| s.torrent_url == format!("http://127.0.0.1:{port}"));
    let message = if is_own && (answered || managed::binary().is_some()) {
        s.torrent_mode = TorrentMode::Managed;
        s.torrent_username = managed::USERNAME.to_string();
        "Downloads use AniBeam's own qBittorrent".to_string()
    } else if answered {
        s.torrent_mode = TorrentMode::Existing;
        // The client it found becomes a saved one, the first on the list
        settings::settle_clients(&mut s, &[])?;
        format!("Downloads use the qBittorrent at {}", s.torrent_url)
    } else if managed::binary().is_some() {
        // The port an earlier setup chose, so the profile and the address agree
        let port = match own_port {
            Some(port) => port,
            None => anirss::free_port()
                .map_err(|e| service(qbt::SERVICE, "failed", format!("no free port: {e}")))?,
        };
        s.torrent_mode = TorrentMode::Managed;
        s.torrent_url = format!("http://127.0.0.1:{port}");
        s.torrent_username = managed::USERNAME.to_string();
        "qBittorrent is a dependency of Downloads and none was running, so AniBeam set up its own"
            .to_string()
    } else {
        return Err(service(
            qbt::SERVICE,
            "missing",
            format!(
                "No qBittorrent answers at {}, and AniBeam has none of its own to run. Install \
                 qBittorrent and turn on its Web UI, or install qbittorrent-nox, which AniBeam \
                 then runs for you",
                s.torrent_url
            ),
        ));
    };
    let stored = s.clone();
    core.store.write(move |c| settings::save(c, &stored))?;
    let status = assemble(core, s.clone());
    core.bus.info(
        Stage::Downloads,
        message,
        EventBody::DownloadsStatusChanged { status },
    );
    Ok(s)
}

/// Where AniBeam's own qBittorrent listens: the port an earlier setup chose,
/// else a free one.
fn own_address(core: &Core) -> Result<String, CoreError> {
    let port = match managed::configured_port(&managed::profile_dir(&core.paths.data_dir)) {
        Some(port) => port,
        None => anirss::free_port()
            .map_err(|e| service(qbt::SERVICE, "failed", format!("no free port: {e}")))?,
    };
    Ok(format!("http://127.0.0.1:{port}"))
}

/// AniBeam's own qBittorrent, started when its Web UI does not answer.
async fn start_managed(core: &Arc<Core>, s: &DownloadSettings) -> Result<(), CoreError> {
    let client = QbtClient::new(&s.torrent_url, managed::USERNAME, "", core.http.clone());
    let answers = || async {
        matches!(
            tokio::time::timeout(PROBE_TIMEOUT, client.version()).await,
            Ok(Ok(_))
        )
    };
    if answers().await {
        return Ok(());
    }
    let failed = |message: String| service(qbt::SERVICE, "failed", message);
    let binary = managed::binary().ok_or_else(|| {
        service(
            qbt::SERVICE,
            "missing",
            "AniBeam's qBittorrent is gone: qbittorrent-nox is not installed. Reinstall \
             AniBeam, or point Settings at a qBittorrent of your own"
                .to_string(),
        )
    })?;
    let port = s
        .torrent_url
        .rsplit(':')
        .next()
        .and_then(|p| p.trim_end_matches('/').parse::<u16>().ok())
        .ok_or_else(|| failed(format!("no port in {}", s.torrent_url)))?;
    let profile = managed::profile_dir(&core.paths.data_dir);
    managed::write_config(&profile, port, managed::save_dir().as_deref())
        .map_err(|e| failed(format!("cannot write qBittorrent's settings: {e}")))?;
    managed::spawn(&binary, &profile, port)
        .map_err(|e| failed(format!("{} could not start: {e}", binary.display())))?;
    let started = tokio::time::Instant::now();
    loop {
        tokio::time::sleep(managed::READY_POLL).await;
        if answers().await {
            tracing::info!("started AniBeam's qBittorrent on port {port}");
            return Ok(());
        }
        if started.elapsed() > managed::READY_TIMEOUT {
            return Err(failed(format!(
                "AniBeam's qBittorrent did not start{}",
                managed::log_tail(&profile)
            )));
        }
    }
}

/// Tries the torrent client afresh and writes what it said into the state.
async fn probe_torrent(core: &Arc<Core>) {
    core.downloads.state().qbt = None;
    let outcome = match qbt_client(core).await {
        Ok(c) => c.version().await,
        Err(e) => Err(e),
    };
    let mut st = core.downloads.state();
    match outcome {
        Ok(version) => {
            st.torrent.connected = true;
            st.torrent.version = version;
            st.torrent.message.clear();
        }
        Err(e) => {
            st.torrent.connected = false;
            st.torrent.version.clear();
            st.torrent.message = plain(&e);
        }
    }
}

fn service(name: &str, code: &str, message: String) -> CoreError {
    CoreError::Service {
        service: name.to_string(),
        code: code.to_string(),
        message,
    }
}

/// A service error's own words, without the service's name in front.
fn plain(e: &CoreError) -> String {
    match e {
        CoreError::Service { message, .. } => message.clone(),
        other => other.to_string(),
    }
}

// The calls ------------------------------------------------------------------

pub fn connect(core: &Core) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::ConnectDownloads, move |_| async move {
            run_connect(owner).await
        }))
}

async fn run_connect(core: Arc<Core>) -> Result<Finished, CoreError> {
    let before = {
        let st = core.downloads.state();
        (st.anirss.state, st.torrent.connected)
    };
    let anirss_failure = match ensure_client(&core).await {
        Ok(client) => match read_anirss(&core, &client).await {
            Ok(()) => None,
            Err(e) => {
                set_anirss_failure(&core, AnirssState::Failed, &plain(&e));
                Some(plain(&e))
            }
        },
        Err(e) => Some(plain(&e)),
    };
    probe_torrent(&core).await;
    // Any other call may have started anirss again while the torrent half
    // was probed. Its outcome is the one to report: a status read in the
    // middle says Starting, and nothing would follow it to the shell.
    let status = {
        let _settled = core.downloads.connecting.lock().await;
        status_async(&core).await?
    };
    let anirss_failure = match status.anirss.state {
        AnirssState::Ready => None,
        _ => anirss_failure.or_else(|| Some(status.anirss.message.clone())),
    };
    let after = (status.anirss.state, status.torrent.connected);
    let level = if after == before {
        Level::Debug
    } else if anirss_failure.is_some() || !status.torrent.connected {
        Level::Warn
    } else {
        Level::Info
    };
    Ok(Finished {
        level,
        message: describe(&status, anirss_failure.as_deref()),
        body: EventBody::DownloadsStatusChanged { status },
    })
}

/// The anirss this core last read: its version and the names of its endpoints. None until
/// one has been read, and always on the phone, which has none.
pub(crate) fn anirss_view(core: &Core) -> Option<(String, Vec<String>)> {
    let st = core.downloads.state();
    (st.anirss.state == AnirssState::Ready).then(|| {
        (
            st.anirss.version.clone(),
            st.endpoints.iter().map(|e| e.name.clone()).collect(),
        )
    })
}

/// Whether the endpoints installed from repos go into this anirss: AniBeam's own sidecar,
/// in a run that may change the owner's setup. A server the person runs themselves keeps
/// its own config, and a sandbox root or a drive never writes into the real one.
async fn syncs_endpoints(core: &Arc<Core>) -> bool {
    core.downloads.unattended_allowed()
        && load_settings(core)
            .await
            .is_ok_and(|s| s.anirss_mode != AnirssMode::Remote)
}

/// What is installed from repos changed (#143): a running anirss gets it, and the page
/// reads the endpoint list again. Nothing is started for this; the next connect syncs.
pub(crate) fn repo_endpoints_changed(core: &Core) {
    let Ok(owner) = core.owner() else { return };
    core.handle.spawn(async move {
        if !syncs_endpoints(&owner).await {
            return;
        }
        let Ok(Some(client)) = running_client(&owner).await else {
            return;
        };
        if read_anirss(&owner, &client).await.is_ok()
            && let Ok(status) = status_async(&owner).await
        {
            owner.bus.debug(
                Stage::Downloads,
                "Search endpoints synced with anirss",
                EventBody::DownloadsStatusChanged { status },
            );
        }
    });
}

/// Health, endpoints and anirss's view of qBittorrent, into the state.
async fn read_anirss(core: &Arc<Core>, client: &AnirssClient) -> Result<(), CoreError> {
    // No feature is asked of a server the user runs: AniBeam never points it
    // at a client, so it reads its own config and keeps its own logins.
    let version = client.health().await?.version;
    let mut endpoints = client.endpoints().await?;
    if syncs_endpoints(core).await
        && crate::repos::sync_anirss(core, client, &version, &endpoints).await
    {
        endpoints = client.endpoints().await?;
    }
    if let Some(path) = settings::anirss_config_path() {
        for e in &mut endpoints {
            e.on_demand = settings::anirss_ondemand(&path, &e.name).map(|o| o.timeout);
        }
    }
    probe_endpoints(core.http.as_ref(), &mut endpoints).await;
    let view = client.qbt_status().await?;
    let mut st = core.downloads.state();
    st.anirss.state = AnirssState::Ready;
    st.anirss.version = version;
    st.anirss.url = client.server();
    st.anirss.message.clear();
    st.anirss.qbt_url = view.url;
    st.anirss.qbt_reachable = view.reachable;
    st.anirss.qbt_authenticated = view.authenticated;
    st.anirss.qbt_login_failed = view.login_failed;
    st.endpoints = endpoints;
    Ok(())
}

/// One line for the log: both halves, as they are.
fn describe(status: &DownloadsStatus, anirss_failure: Option<&str>) -> String {
    let anirss = match anirss_failure {
        Some(m) => format!("anirss is not available: {m}"),
        None => format!("anirss {} ready", status.anirss.version),
    };
    let client = status.settings.torrent_client.as_str();
    let torrent = if status.torrent.connected {
        format!("{client} {} connected", status.torrent.version)
    } else {
        format!("{client} not connected: {}", status.torrent.message)
    };
    format!("{anirss}; {torrent}")
}

pub fn test_torrent_client(core: &Core) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::TestTorrentClient, move |_| async move {
            let before = owner.downloads.state().torrent.connected;
            probe_torrent(&owner).await;
            let status = status_async(&owner).await?;
            let client = status.settings.torrent_client.as_str();
            let (level, message) = if status.torrent.connected {
                (
                    if before { Level::Debug } else { Level::Info },
                    format!("{client} {} connected", status.torrent.version),
                )
            } else {
                (
                    Level::Warn,
                    format!("{client} not connected: {}", status.torrent.message),
                )
            };
            Ok(Finished {
                level,
                message,
                body: EventBody::DownloadsStatusChanged { status },
            })
        }))
}

/// Whether each on-demand endpoint is up, from its ready URL. A GET and
/// nothing more: only `start_endpoint`, after a confirm, starts a service
/// (anirss does too when a search reaches one, which is why `search` checks first).
async fn probe_endpoints(http: &dyn Http, endpoints: &mut [SearchEndpoint]) {
    let path = settings::anirss_config_path();
    for e in endpoints.iter_mut() {
        let ready = path
            .as_deref()
            .and_then(|p| settings::anirss_ondemand(p, &e.name))
            .map(|o| o.ready)
            .filter(|r| !r.is_empty());
        e.running = match ready {
            Some(url) if e.on_demand.is_some() => Some(probe(http, &url).await),
            _ => None,
        };
    }
}

/// One ready probe, short: a local service either answers at once or is
/// not there.
async fn probe(http: &dyn Http, url: &str) -> bool {
    http.send(HttpRequest {
        method: Method::Get,
        url: url.to_string(),
        headers: Vec::new(),
        body: None,
        timeout: Some(std::time::Duration::from_millis(1500)),
    })
    .await
    .map(|r| r.is_success())
    .unwrap_or(false)
}

/// Re-probes the on-demand endpoints the status already lists. Starts
/// nothing, anirss included: with no endpoint list there is nothing to ask.
pub fn check_endpoints(core: &Core) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    forget_probes();
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::CheckEndpoints, move |_| async move {
            let mut endpoints = owner.downloads.state().endpoints.clone();
            probe_endpoints(owner.http.as_ref(), &mut endpoints).await;
            let up: Vec<String> = endpoints
                .iter()
                .filter(|e| e.running == Some(true))
                .map(|e| e.name.clone())
                .collect();
            owner.downloads.state().endpoints = endpoints;
            let status = status_async(&owner).await?;
            Ok(Finished {
                level: Level::Debug,
                message: if up.is_empty() {
                    "No on-demand endpoint is running".to_string()
                } else {
                    format!("Running: {}", up.join(", "))
                },
                body: EventBody::DownloadsStatusChanged { status },
            })
        }))
}

/// Starts the service behind an on-demand endpoint (Jackett for 1337x) and
/// probes the endpoints again. The shell calls this only after the user
/// confirmed; nothing else in the core starts one. The command is the
/// `jackett_command` setting, else anirss's `start` for the endpoint, else
/// `jackett`, and it is detached (see `jackett`).
pub fn start_endpoint(core: &Core, name: String) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    forget_probes();
    let config = settings::anirss_config_path();
    let ondemand = config
        .as_deref()
        .and_then(|p| settings::anirss_ondemand(p, &name))
        .ok_or_else(|| {
            CoreError::invalid("name", format!("{name} is not an on-demand endpoint"))
        })?;
    let log = std::path::Path::new(&core.paths.state_dir).join("jackett-start.log");
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::StartEndpoint, move |ctx| async move {
            let s = load_settings(&owner).await?;
            let command = jackett::resolve_command(&s.jackett_command, config.as_deref(), &name);
            ctx.progress(0, None, &format!("Starting {name}"));
            jackett::start(
                owner.http.as_ref(),
                &command,
                &ondemand.ready,
                std::time::Duration::from_secs(ondemand.timeout),
                std::time::Duration::from_millis(500),
                &log,
            )
            .await
            .map_err(|failure| {
                let (code, message) = jackett::explain(&name, &failure);
                service(&name, code, message)
            })?;
            forget_probes();
            let mut endpoints = owner.downloads.state().endpoints.clone();
            probe_endpoints(owner.http.as_ref(), &mut endpoints).await;
            owner.downloads.state().endpoints = endpoints;
            let status = status_async(&owner).await?;
            Ok(Finished {
                level: Level::Info,
                message: format!("{name} is running"),
                body: EventBody::DownloadsStatusChanged { status },
            })
        }))
}

/// A probe's answer kept for half a minute, so a burst of searches (one per
/// source, then the show view's) asks each on-demand endpoint once.
static KEPT: LazyLock<Mutex<HashMap<String, (std::time::Instant, bool)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Drops every kept probe, for a check or a start that must see the
/// endpoint as it is now.
fn forget_probes() {
    recover(&KEPT).clear();
}

async fn probe_kept(http: &dyn Http, url: &str) -> bool {
    if let Some((at, up)) = recover(&KEPT).get(url).copied()
        && at.elapsed() < std::time::Duration::from_secs(30)
    {
        return up;
    }
    let up = probe(http, url).await;
    recover(&KEPT).insert(url.to_string(), (std::time::Instant::now(), up));
    up
}

/// The endpoint a search asked for, else the first anirss lists that is not
/// started on demand: a default search never lands on a stopped 1337x.
fn default_target(endpoint: Option<String>, listed: &[SearchEndpoint]) -> Option<String> {
    endpoint.or_else(|| {
        listed
            .iter()
            .find(|e| e.on_demand.is_none())
            .or_else(|| listed.first())
            .map(|e| e.name.clone())
    })
}

/// The search guard. anirss boots an on-demand endpoint whenever a search
/// reaches it, so the core makes sure none does unless it already runs:
/// a search aimed at a stopped one is refused, and anirss's fallback is
/// switched off while any on-demand endpoint is down. Returns whether the
/// fallback may stay on.
async fn guard_search(
    http: &dyn Http,
    endpoints: &[SearchEndpoint],
    target: Option<&str>,
) -> Result<bool, CoreError> {
    let path = settings::anirss_config_path();
    let mut all_up = true;
    for e in endpoints.iter().filter(|e| e.on_demand.is_some()) {
        let ready = path
            .as_deref()
            .and_then(|p| settings::anirss_ondemand(p, &e.name))
            .map(|o| o.ready)
            .unwrap_or_default();
        let up = !ready.is_empty() && probe_kept(http, &ready).await;
        if !up && target == Some(e.name.as_str()) {
            return Err(CoreError::Service {
                service: e.name.clone(),
                code: "not_running".to_string(),
                message: format!(
                    "{} is not running. Switch it on in the sources and confirm to start it.",
                    e.name
                ),
            });
        }
        all_up &= up;
    }
    Ok(all_up)
}

pub fn search(core: &Core, query: String, endpoint: Option<String>) -> Result<u64, CoreError> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err(CoreError::invalid("query", "type something to search for"));
    }
    let owner = core.owner()?;
    // The endpoint the search starts on, for the progress line: the one
    // asked for, else the first anirss lists
    let first = default_target(endpoint.clone(), &owner.downloads.state().endpoints);
    let ondemand = first.as_deref().and_then(|name| {
        settings::anirss_config_path().and_then(|p| settings::anirss_ondemand(&p, name))
    });
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::SearchReleases, move |ctx| async move {
            let phase = Arc::new(AtomicU8::new(PHASE_CONNECTING));
            let watch = watch_search(
                ctx.clone(),
                owner.http.clone(),
                phase.clone(),
                first
                    .clone()
                    .unwrap_or_else(|| "the first endpoint".to_string()),
                ondemand,
            );
            let work = async {
                let client = ensure_client(&owner).await?;
                let listed = owner.downloads.state().endpoints.clone();
                let target = default_target(endpoint.clone(), &listed);
                let fallback =
                    guard_search(owner.http.as_ref(), &listed, target.as_deref()).await?;
                phase.store(PHASE_FETCHING, Ordering::Relaxed);
                let raw = client.search(&query, target.as_deref(), fallback).await?;
                phase.store(PHASE_READING, Ordering::Relaxed);
                let fetched = raw.query.clone();
                let mut search = if raw.items.is_empty() {
                    ReleaseSearch {
                        query: raw.query,
                        endpoint: raw.endpoint,
                        items: Vec::new(),
                        groups: Vec::new(),
                        best_fit: BestFit {
                            available: false,
                            query: String::new(),
                            item: None,
                            profile: Vec::new(),
                        },
                        tried: raw.tried,
                        fell_back: raw.fell_back,
                        client_side: false,
                        auto_resolution: None,
                        facts: Vec::new(),
                        show: None,
                        fetched_query: String::new(),
                    }
                } else {
                    let a = client
                        .analyze(&raw.query, &raw.endpoint, &raw.items)
                        .await?;
                    ReleaseSearch {
                        query: a.query,
                        endpoint: raw.endpoint,
                        items: a.items,
                        groups: a.groups,
                        best_fit: a.best_fit,
                        tried: raw.tried,
                        fell_back: raw.fell_back,
                        client_side: a.client_side,
                        auto_resolution: a.auto_resolution,
                        facts: Vec::new(),
                        show: None,
                        fetched_query: String::new(),
                    }
                };
                let (facts, show) = annotate(&owner, &search.query, &search.items);
                search.facts = facts;
                search.show = show;
                search.fetched_query = fetched;
                if !search.items.is_empty() {
                    enrich(
                        &owner,
                        client,
                        &search.query,
                        &search.endpoint,
                        &search.items,
                    );
                }
                Ok::<_, CoreError>(search)
            };
            let search = tokio::select! {
                r = work => r?,
                _ = watch => unreachable!("the watch never ends"),
            };
            let message = if search.fell_back {
                let asked = endpoint
                    .clone()
                    .or_else(|| search.tried.first().map(|t| t.endpoint.clone()))
                    .unwrap_or_default();
                format!(
                    "{asked}: no results, switched to {} ({})",
                    search.endpoint,
                    search.items.len()
                )
            } else {
                format!(
                    "{} releases for \"{}\" on {}",
                    search.items.len(),
                    search.query,
                    search.endpoint
                )
            };
            Ok(Finished {
                level: Level::Debug,
                message,
                body: EventBody::ReleasesFound { search },
            })
        }))
}

const PHASE_CONNECTING: u8 = 0;
const PHASE_FETCHING: u8 = 1;
const PHASE_READING: u8 = 2;

/// The search's progress line, once a second until the search answers:
/// what it is waiting on, and for how long. An on-demand endpoint (one
/// anirss starts, a podman pod behind Jackett) is "Starting" until its
/// ready URL answers, then "Searching"; `done` counts seconds. A bounded
/// poll that lives only as long as the search does.
async fn watch_search(
    ctx: Arc<JobCtx>,
    http: Arc<dyn Http>,
    phase: Arc<AtomicU8>,
    endpoint: String,
    ondemand: Option<settings::OnDemand>,
) {
    let started = std::time::Instant::now();
    loop {
        let label = match phase.load(Ordering::Relaxed) {
            PHASE_CONNECTING => "Starting anirss".to_string(),
            PHASE_READING => "Reading the results".to_string(),
            _ => match &ondemand {
                Some(od) if !od.ready.is_empty() && !answers(http.as_ref(), &od.ready).await => {
                    format!("Starting {endpoint}")
                }
                _ => format!("Searching {endpoint}"),
            },
        };
        // No total: the seconds are not a count of anything, and the page
        // knows an on-demand endpoint's ceiling from the endpoint list
        ctx.progress(started.elapsed().as_secs(), None, &label);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

async fn answers(http: &dyn Http, url: &str) -> bool {
    http.send(HttpRequest {
        method: Method::Get,
        url: url.to_string(),
        headers: Vec::new(),
        body: None,
        timeout: Some(std::time::Duration::from_secs(2)),
    })
    .await
    .map(|r| r.is_success())
    .unwrap_or(false)
}

/// The facts read off each release, and the library series the query
/// names with each release's episode checked against its files. A library
/// that cannot be read leaves the show out and every `on_disk` false.
pub(crate) fn annotate(
    core: &Core,
    query: &str,
    items: &[Release],
) -> (Vec<ReleaseFacts>, Option<ShowLink>) {
    let mut facts: Vec<ReleaseFacts> = items.iter().map(facts::read).collect();
    if items.is_empty() {
        return (facts, None);
    }
    let images = core.paths.images_dir();
    let found = core
        .store
        .read(|c| match link::series_for_query(c, query)? {
            Some(id) => link::show_with_numbers(c, &images, crate::time::now(), id),
            None => Ok(None),
        });
    match found {
        Ok(Some((show, files))) => {
            // On disk means this very release is the file (#237); the episode
            // from another release is only other_on_disk
            for (f, item) in facts.iter_mut().zip(items) {
                if let Some(n) = f.episode {
                    let mut paths = link::episode_files(&files, f.season, n).peekable();
                    let any = paths.peek().is_some();
                    f.on_disk = paths.any(|p| facts::same_release(&item.title, f, p));
                    f.other_on_disk = any && !f.on_disk;
                }
            }
            (facts, Some(show))
        }
        Ok(None) => (facts, None),
        Err(e) => {
            tracing::warn!("search not matched to the library: {e}");
            (facts, None)
        }
    }
}

/// After a search answers: the feed it read, read again by the core for
/// what anirss drops (when each release was posted, a torznab feed's
/// counts). The address is anirss's own (`/subscriptions/preview` gives the
/// feed a search on this query reads), and the read is cheap: nyaa and
/// Erai answer at once and Jackett serves a repeat query from its cache.
/// Not a job; a failure is a debug line and the page shows no dates.
fn enrich(core: &Arc<Core>, client: AnirssClient, query: &str, endpoint: &str, items: &[Release]) {
    let owner = core.clone();
    let (query, endpoint, items) = (query.to_string(), endpoint.to_string(), items.to_vec());
    core.handle.spawn(async move {
        let outcome = async {
            let spec = client.preview(&query, &endpoint, &items).await?;
            if spec.feed_url.is_empty() {
                return Err(CoreError::internal("anirss named no feed for this search"));
            }
            feed::read(owner.http.as_ref(), &spec.feed_url).await
        }
        .await;
        match outcome {
            Ok(entries) => owner.bus.debug(
                Stage::Downloads,
                format!(
                    "{} dated on {endpoint}",
                    plural(entries.len() as u64, "release", "releases")
                ),
                EventBody::ReleasesEnriched {
                    query,
                    endpoint,
                    entries,
                },
            ),
            Err(error) => owner.bus.debug(
                Stage::Downloads,
                format!("releases not dated: {}", plain(&error)),
                EventBody::DownloadsUnavailable {
                    what: "dates".to_string(),
                    error,
                },
            ),
        };
    });
}

pub fn refine(
    core: &Core,
    query: String,
    endpoint: String,
    items: Vec<Release>,
    op: RefineOp,
) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::RefineReleases, move |_| async move {
            let client = ensure_client(&owner).await?;
            let mut refinement = client.apply(&query, &endpoint, &items, &op).await?;
            if refinement.applied {
                let (facts, show) = annotate(&owner, &refinement.query, &refinement.items);
                refinement.facts = facts;
                refinement.show = show;
                // A refetch read a new feed; a client-side step kept releases already dated
                if refinement.mode == "refetch" && !refinement.items.is_empty() {
                    enrich(
                        &owner,
                        client,
                        &refinement.query,
                        &refinement.endpoint,
                        &refinement.items,
                    );
                }
            }
            let message = if refinement.applied {
                format!(
                    "Narrowed to {} releases ({:+})",
                    refinement.items.len(),
                    refinement.delta
                )
            } else {
                format!(
                    "Nothing changed: {}",
                    refinement.reason.as_deref().unwrap_or("no reason given")
                )
            };
            Ok(Finished {
                level: Level::Debug,
                message,
                body: EventBody::ReleasesRefined { refinement },
            })
        }))
}

pub fn preview(
    core: &Core,
    query: String,
    endpoint: String,
    items: Vec<Release>,
) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::Subscribe, move |_| async move {
            let client = ensure_client(&owner).await?;
            let spec = client.preview(&query, &endpoint, &items).await?;
            Ok(Finished {
                level: Level::Debug,
                message: format!("Subscription preview for \"{query}\""),
                body: EventBody::SubscriptionPreviewed { spec },
            })
        }))
}

pub fn subscribe(
    core: &Core,
    query: String,
    endpoint: String,
    items: Vec<Release>,
    name: Option<String>,
    wire_query: Option<String>,
) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::Subscribe, move |ctx| async move {
            let client = ensure_client(&owner).await?;
            let mut result = client
                .subscribe(&query, &endpoint, &items, name.as_deref())
                .await?;
            let terms = match wire_query.as_deref() {
                Some(wire) if narrows_locally(&owner, &endpoint) => {
                    rewire::local_terms(&query, wire)
                }
                _ => rewire::LocalTerms::default(),
            };
            let message = if terms.is_empty() {
                format!(
                    "Subscribed to {}; new episodes save to {}",
                    result.name, result.save_path
                )
            } else {
                let wire = wire_query.clone().unwrap_or_default();
                let wide = client.preview(&wire, &endpoint, &items).await?.feed_url;
                rewire_subscription(&owner, &ctx, &mut result, &wide, &items, &terms).await?;
                let searched = rewire::search_param(&result.feed_url).map_or(wire, |(_, v)| v);
                format!(
                    "Subscribed to {}; {endpoint} is searched for \"{searched}\" and the rule keeps {}",
                    result.name,
                    describe_terms(&terms)
                )
            };
            Ok(Finished {
                level: Level::Info,
                message,
                body: EventBody::Subscribed { result },
            })
        }))
}

/// An endpoint anirss narrows on its own side, where a refined query is no
/// query the endpoint can answer: a generic RSS endpoint that is not Erai
/// (whose subscription anirss already narrows in the rule).
fn narrows_locally(core: &Core, endpoint: &str) -> bool {
    core.downloads
        .state()
        .endpoints
        .iter()
        .find(|e| e.name == endpoint)
        .is_some_and(|e| e.kind != "nyaa" && !e.url.contains("erai-raws"))
}

fn describe_terms(terms: &rewire::LocalTerms) -> String {
    let mut parts = Vec::new();
    if !terms.must.is_empty() {
        parts.push(format!("releases with \"{}\"", terms.must.join(" ")));
    }
    if !terms.not.is_empty() {
        parts.push(format!("none with \"{}\"", terms.not.join("\", \"")));
    }
    parts.join(" and ")
}

/// Points anirss's new rule at the feed the search read, narrowed by the
/// words the endpoint can answer, and puts every narrowing word in the
/// rule.
/// `searched` is the feed anirss would make for the query the endpoint was
/// asked.
async fn rewire_subscription(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    result: &mut Subscribed,
    searched: &str,
    items: &[Release],
    terms: &rewire::LocalTerms,
) -> Result<(), CoreError> {
    if searched.is_empty() || searched == result.feed_url {
        return Ok(());
    }
    let feed_url = tightest_feed(core, ctx, searched, &terms.must, items).await;
    if feed_url == result.feed_url {
        return Ok(());
    }
    let qbt = qbt_client(core).await?;
    let rules = qbt.rules().await?;
    let Some(def) = rules.get(&result.name) else {
        return Err(service(
            "anirss",
            "rule_missing",
            format!("qBittorrent holds no rule named {}", result.name),
        ));
    };
    let def = rewire::narrowed(def, terms, &feed_url);
    swap_feed(
        &qbt,
        &rules,
        &result.name,
        &def,
        &result.feed_url,
        &feed_url,
    )
    .await?;
    result.feed_url = feed_url;
    result.must_contain = def["mustContain"].as_str().unwrap_or("").to_string();
    result.must_not_contain = def["mustNotContain"].as_str().unwrap_or("").to_string();
    Ok(())
}

/// Moves rule `name` from `old_url` to `new_url`, `def` being the rule as
/// it should read. In this order, so qBittorrent never holds the rule
/// unnarrowed beside a feed with articles: the rule first, then the old
/// feed out (unless another rule reads it), then the new feed in, in the
/// old one's place.
async fn swap_feed(
    qbt: &QbtClient,
    rules: &serde_json::Value,
    name: &str,
    def: &serde_json::Value,
    old_url: &str,
    new_url: &str,
) -> Result<(), CoreError> {
    qbt.set_rule(name, def).await?;
    let feeds = qbt.feeds().await?;
    let old = feeds
        .iter()
        .find(|(_, url)| *url == old_url)
        .map(|(path, _)| path.clone());
    if let Some(path) = &old
        && !rewire::feed_shared(rules, name, old_url)
    {
        qbt.remove_item(path).await?;
    }
    qbt.add_feed(new_url, old.as_deref().unwrap_or(name))
        .await?;
    Ok(())
}

/// The narrowest feed the endpoint itself can answer. Starting from the
/// search the feed carries, each narrowing word is tried in it, and a word
/// stays when the feed still holds every picked release and holds fewer
/// than before ("Helluva Boss TBK" works on 1337x where "Helluva Boss S03
/// TBK" finds nothing). Only what the feed returns decides, so it works the
/// same for any endpoint whose feed carries a search. A feed that cannot be
/// read rejects that word; the rule keeps every word either way.
async fn tightest_feed(
    core: &Arc<Core>,
    ctx: &Arc<JobCtx>,
    feed_url: &str,
    words: &[String],
    items: &[Release],
) -> String {
    const MAX_PROBES: usize = 4;
    let Some((key, base)) = rewire::search_param(feed_url) else {
        return feed_url.to_string();
    };
    if words.is_empty() || items.is_empty() {
        return feed_url.to_string();
    }
    let http = core.http.as_ref();
    let Ok(now) = feed::read(http, feed_url).await else {
        return feed_url.to_string();
    };
    let words: Vec<&String> = words.iter().take(MAX_PROBES).collect();
    let mut best = (feed_url.to_string(), now.len());
    let mut search = base.trim().to_string();
    for (n, word) in words.iter().enumerate() {
        let field = if word.contains(' ') {
            format!("\"{word}\"")
        } else {
            word.to_string()
        };
        let candidate = format!("{search} {field}");
        ctx.progress(
            n as u64,
            Some(words.len() as u64),
            &format!("Trying \"{candidate}\""),
        );
        let url = rewire::with_search(feed_url, &key, &candidate);
        let Ok(entries) = feed::read(http, &url).await else {
            continue;
        };
        if entries.len() < best.1 && rewire::holds_all(&entries, items) {
            best = (url, entries.len());
            search = candidate;
        }
    }
    best.0
}

pub fn download(core: &Core, items: Vec<Release>, name: Option<String>) -> Result<u64, CoreError> {
    download_with(core, items, name, DownloadOptions::default())
}

/// `download` with what anirss cannot be asked for: the new torrents read in
/// order, and episodes they replace (`downloads::replace`). anirss adds the
/// torrents; the rest is applied once qBittorrent lists them. The add
/// itself is the same call, so a failure there changes nothing: nothing is
/// cancelled and nothing is recorded.
pub fn download_with(
    core: &Core,
    items: Vec<Release>,
    name: Option<String>,
    options: DownloadOptions,
) -> Result<u64, CoreError> {
    if items.is_empty() {
        return Err(CoreError::invalid("items", "nothing selected to download"));
    }
    let owner = core.owner()?;
    // What rolls is held back before anything else is asked of it, so a
    // held episode gets no followup that would wait days for its torrent
    let roll_settings = owner.store.read(settings::load_stored)?.rolling;
    let plan = rolling::plan(
        &items,
        &options,
        &roll_settings,
        name.as_deref(),
        crate::time::now_secs(),
    );
    let followups = replace::prepare(&owner.store, &plan.now, &options)?;
    let home = show_folder(&owner, options.series)?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::Download, move |_| async move {
            let client = ensure_client(&owner).await?;
            let mut result = client.download(&plan.now, name.as_deref()).await?;
            move_home(&owner, &mut result, home.as_deref()).await;
            let mut extra = replace::describe(&options, &followups);
            extra.push_str(&rolling::describe(&plan));
            rolling::after_add(&owner, plan.rolls, &plan.now, &result.hashes).await;
            if options != DownloadOptions::default() || !followups.is_empty() {
                match qbt_client(&owner).await {
                    Ok(qbt) => replace::after_add(&owner, &qbt, &items, &options, followups).await,
                    Err(e) => tracing::warn!("download not followed: {e}"),
                }
            }
            Ok(Finished {
                level: Level::Info,
                message: format!(
                    "Added {} to {} under {}{extra}",
                    plural(result.count, "torrent", "torrents"),
                    owner.downloads.torrent_name(),
                    result.save_path
                ),
                body: EventBody::TorrentsAdded { result },
            })
        }))
}

/// The folder of the library show a download is for. anirss saves under
/// its own root plus the show's name, which is another folder whenever the
/// show lives in a different source: the episodes would scan in as a show
/// of their own beside it.
fn show_folder(core: &Core, series: Option<u64>) -> Result<Option<String>, CoreError> {
    let Some(id) = series else { return Ok(None) };
    core.store.read(|c| {
        Ok(c.query_row(
            "SELECT path FROM series WHERE id = ?1 AND kind = 'show' AND missing_since IS NULL",
            [id as i64],
            |r| r.get::<_, String>(0),
        )
        .ok())
    })
}

/// Moves new torrents into their show's folder when anirss put them
/// elsewhere. Only when this machine sees both folders, so a client on
/// another host, which names its paths its own way, is left alone.
async fn move_home(core: &Arc<Core>, result: &mut AddedTorrents, home: Option<&str>) {
    let Some(home) = home else { return };
    let saved = result.save_path.trim_end_matches(['/', '\\']);
    if result.hashes.is_empty() || saved == home.trim_end_matches(['/', '\\']) {
        return;
    }
    let local = std::path::Path::new(saved)
        .parent()
        .is_some_and(|base| base.is_dir());
    if !local || !std::path::Path::new(home).is_dir() {
        return;
    }
    let moved = match qbt_client(core).await {
        Ok(qbt) => qbt.set_location(&result.hashes, home).await,
        Err(e) => Err(e),
    };
    match moved {
        Ok(()) => result.save_path = home.to_string(),
        Err(e) => tracing::warn!("download not moved into {home}: {e}"),
    }
}

pub fn movie(core: &Core, link: String) -> Result<u64, CoreError> {
    let link = link.trim().to_string();
    if !(link.starts_with("magnet:") || link.starts_with("http://") || link.starts_with("https://"))
    {
        return Err(CoreError::invalid(
            "link",
            "a movie link must be a magnet or an http(s) URL",
        ));
    }
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::Download, move |_| async move {
            let client = ensure_client(&owner).await?;
            let result = client.movie(&link).await?;
            Ok(Finished {
                level: Level::Info,
                message: format!("Added a movie to {}", result.save_path),
                body: EventBody::TorrentsAdded { result },
            })
        }))
}

pub fn plan_remove(
    core: &Core,
    name: String,
    torrents: bool,
    files: bool,
) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::RemoveSubscription, move |_| async move {
            let client = ensure_client(&owner).await?;
            let plans = client
                .remove(std::slice::from_ref(&name), torrents, files, true)
                .await?;
            Ok(Finished {
                level: Level::Debug,
                message: format!("Planned the removal of {name}"),
                body: EventBody::RemovalPlanned { plans },
            })
        }))
}

pub fn remove(core: &Core, name: String, torrents: bool, files: bool) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::RemoveSubscription, move |_| async move {
            let client = ensure_client(&owner).await?;
            client
                .remove(std::slice::from_ref(&name), torrents, files, false)
                .await?;
            let extra = match (torrents, files) {
                (true, true) => " with its torrents and files",
                (true, false) => " with its torrents",
                (false, true) => " with its files",
                (false, false) => "",
            };
            Ok(Finished {
                level: Level::Info,
                message: format!("Removed the subscription {name}{extra}"),
                body: EventBody::SubscriptionRemoved { name },
            })
        }))
}

pub fn control(core: &Core, hashes: Vec<String>, action: TorrentAction) -> Result<u64, CoreError> {
    let hashes: Vec<String> = hashes
        .into_iter()
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty())
        .collect();
    if hashes.is_empty() {
        return Err(CoreError::invalid("hashes", "no torrents named"));
    }
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::ControlTorrents, move |_| async move {
            let client = qbt_client(&owner).await?;
            let verb = match &action {
                TorrentAction::Stop => {
                    client.stop(&hashes).await?;
                    "Stopped"
                }
                TorrentAction::Start => {
                    client.start(&hashes).await?;
                    "Started"
                }
                TorrentAction::Delete { files } => {
                    client.delete(&hashes, *files).await?;
                    if *files {
                        "Deleted, files included,"
                    } else {
                        "Removed"
                    }
                }
                TorrentAction::Sequential { on } => {
                    client.set_sequential(&hashes, *on).await?;
                    if *on {
                        "Set download in order on for"
                    } else {
                        "Set download in order off for"
                    }
                }
                TorrentAction::FirstLastPieces { on } => {
                    client.set_first_last(&hashes, *on).await?;
                    if *on {
                        "Set first and last pieces first on for"
                    } else {
                        "Set first and last pieces first off for"
                    }
                }
            };
            Ok(Finished {
                level: Level::Info,
                message: format!(
                    "{verb} {}",
                    plural(hashes.len() as u64, "torrent", "torrents")
                ),
                body: EventBody::TorrentsControlled { hashes, action },
            })
        }))
}

/// `Call::RemoveShow`: the subscription, the torrents, the files, as far
/// as asked, each step only once the one before it held. The file targets
/// are read and checked first, so a refusal removes nothing at all.
#[allow(clippy::too_many_arguments)]
pub fn remove_show(
    core: &Core,
    title: String,
    rule: Option<String>,
    hashes: Vec<String>,
    torrents: bool,
    files: bool,
    series: Option<u64>,
) -> Result<u64, CoreError> {
    let rule = rule.map(|r| r.trim().to_string()).filter(|r| !r.is_empty());
    let hashes: Vec<String> = hashes
        .into_iter()
        .map(|h| h.trim().to_ascii_lowercase())
        .filter(|h| !h.is_empty())
        .collect();
    let torrents = torrents || files;
    let takes_torrents = torrents && !hashes.is_empty();
    let takes_files = files && (!hashes.is_empty() || series.is_some());
    if rule.is_none() && !takes_torrents && !takes_files {
        return Err(CoreError::invalid("show", "nothing to remove"));
    }
    let title = if title.trim().is_empty() {
        rule.clone().unwrap_or_else(|| "the show".to_string())
    } else {
        title.trim().to_string()
    };
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::RemoveShow, move |_| async move {
            // A client on another machine wrote its files there: they are out
            // of the trash's reach, and its own delete takes them instead.
            let here = settings::files_here(&load_settings(&owner).await?);
            let client_deletes_files = takes_files && takes_torrents && !here;
            // What the torrents wrote, read before they go, and the series' folder
            let mut candidates = Vec::new();
            if takes_files && here && !hashes.is_empty() {
                let client = qbt_client(&owner).await?;
                for t in client.torrents().await? {
                    if hashes.contains(&t.hash.to_ascii_lowercase()) {
                        candidates.push(trash::Candidate::Torrent {
                            content: t.content_path,
                            save_path: t.save_path,
                        });
                    }
                }
            }
            let mut sources = Vec::new();
            if takes_files {
                let (series_path, all_sources) = owner.store.read(|c| {
                    let path: Option<String> = match series {
                        Some(id) => c
                            .query_row("SELECT path FROM series WHERE id = ?1", [id as i64], |r| {
                                r.get(0)
                            })
                            .ok(),
                        None => None,
                    };
                    let mut stmt = c.prepare("SELECT path FROM sources")?;
                    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
                    let mut all = Vec::new();
                    for row in rows {
                        all.push(row?);
                    }
                    Ok((path, all))
                })?;
                if let Some(path) = series_path {
                    candidates.push(trash::Candidate::Series { path });
                }
                sources = all_sources;
            }
            let home = std::env::home_dir();
            let targets = if takes_files {
                trash::targets(&candidates, &sources, home.as_deref())?
            } else {
                Vec::new()
            };

            if let Some(name) = &rule {
                let client = ensure_client(&owner).await?;
                client
                    .remove(std::slice::from_ref(name), false, false, false)
                    .await?;
            }
            if takes_torrents {
                // Never the client's own delete-with-files on this machine:
                // what goes, goes to the trash
                qbt_client(&owner)
                    .await?
                    .delete(&hashes, client_deletes_files)
                    .await?;
            }
            if !targets.is_empty() {
                let paths = targets.clone();
                tokio::task::spawn_blocking(move || trash::move_to_trash(&paths))
                    .await
                    .map_err(|e| CoreError::Internal {
                        message: format!("trash: {e}"),
                    })??;
            }

            let mut parts = Vec::new();
            if rule.is_some() {
                parts.push("the subscription".to_string());
            }
            if takes_torrents {
                parts.push(plural(hashes.len() as u64, "torrent", "torrents"));
            }
            if client_deletes_files {
                parts.push("their files on the torrent client".to_string());
            }
            if !targets.is_empty() {
                parts.push(format!(
                    "{} {}",
                    plural(targets.len() as u64, "path", "paths"),
                    trash::WHERE
                ));
            }
            let what = match parts.len() {
                0 => "nothing".to_string(),
                1 => parts[0].clone(),
                _ => format!(
                    "{} and {}",
                    parts[..parts.len() - 1].join(", "),
                    parts[parts.len() - 1]
                ),
            };
            Ok(Finished {
                level: Level::Info,
                message: format!("Removed {title}: {what}"),
                body: EventBody::ShowRemoved {
                    title,
                    rule,
                    hashes: if takes_torrents { hashes } else { Vec::new() },
                    trashed: targets
                        .iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect(),
                },
            })
        }))
}

/// Not a job: the page asks on open and after every change. The list
/// arrives as `SubscriptionsRead`, a failure as `DownloadsUnavailable`,
/// both at debug level so a page polling an absent server writes nothing
/// into the log.
pub fn list_rules(core: &Core) -> Result<Reply, CoreError> {
    let owner = core.owner()?;
    core.handle.spawn(async move {
        let outcome = match ensure_client(&owner).await {
            Ok(client) => client.subscriptions().await.map(|rules| (client, rules)),
            Err(e) => Err(e),
        };
        match outcome {
            Ok((client, mut rules)) => {
                link_rules(&owner, &mut rules);
                owner.bus.debug(
                    Stage::Downloads,
                    format!(
                        "{} read",
                        plural(rules.len() as u64, "subscription", "subscriptions")
                    ),
                    EventBody::SubscriptionsRead {
                        rules: rules.clone(),
                    },
                );
                prune::after_read(&owner, &client, &rules).await;
            }
            Err(error) => {
                owner.bus.debug(
                    Stage::Downloads,
                    format!("subscriptions not read: {}", plain(&error)),
                    EventBody::DownloadsUnavailable {
                        what: "subscriptions".to_string(),
                        error,
                    },
                );
            }
        };
    });
    Ok(Reply::Ok)
}

/// Not a job either: the page polls this while it is open.
pub fn list_torrents(core: &Core) -> Result<Reply, CoreError> {
    let owner = core.owner()?;
    core.handle.spawn(async move {
        let outcome = async {
            let client = qbt_client(&owner).await?;
            let mut torrents = client.torrents().await?;
            // The files of each unfinished torrent, so a batch shows each
            // episode's progress; a single file adds nothing to the torrent's
            for t in torrents.iter_mut().filter(|t| t.progress < 1.0) {
                match client.files(&t.hash).await {
                    Ok(files) if files.len() > 1 => t.files = files,
                    Ok(_) => {}
                    Err(e) => tracing::debug!("files of {} not read: {e}", t.name),
                }
            }
            let transfer = client.transfer().await?;
            if owner.downloads.unattended_allowed() {
                replace::tick(&owner, &client, &torrents).await;
                rolling::tick(&owner, &client, &torrents).await;
            }
            Ok::<_, CoreError>((torrents, transfer))
        }
        .await;
        match outcome {
            Ok((mut torrents, transfer)) => {
                let mut shows = link_torrents(&owner, &mut torrents);
                replace::annotate(&owner, &mut torrents);
                rolling::annotate(&owner, &mut torrents);
                let downloading = downloading_of(&torrents);
                let moved: Vec<u64> = {
                    let mut st = owner.downloads.state();
                    st.torrent.connected = true;
                    st.torrent.message.clear();
                    st.torrents_at = Some(std::time::Instant::now());
                    let due = due_for_announcing(
                        &mut st.announced,
                        &downloading,
                        std::time::Instant::now(),
                    );
                    st.downloading = downloading;
                    st.writing = writing_of(&torrents);
                    due
                };
                schedule_reread(&owner);
                for show in shows.iter_mut() {
                    overlay_cubes(&owner, show.series_id, &mut show.cubes, None);
                }
                // A series whose downloading episodes (or, every so often,
                // their progress) changed reads anew on every page that shows it
                if !moved.is_empty() {
                    let images = owner.paths.images_dir();
                    if let Ok(mut cards) = owner.store.read(|c| {
                        let snap = crate::library::snapshot::Snapshot::load(
                            c,
                            &images,
                            crate::time::now(),
                            Some(&moved),
                        )?;
                        Ok(moved
                            .iter()
                            .filter_map(|id| snap.card(*id))
                            .collect::<Vec<SeriesCard>>())
                    }) {
                        overlay(&owner, &mut cards);
                        owner.bus.debug(
                            Stage::Downloads,
                            format!(
                                "{} downloading changed",
                                plural(cards.len() as u64, "series", "series")
                            ),
                            EventBody::SeriesChanged { series: cards },
                        );
                    }
                }
                owner.bus.debug(
                    Stage::Downloads,
                    format!(
                        "{} read",
                        plural(torrents.len() as u64, "torrent", "torrents")
                    ),
                    EventBody::TorrentsRead {
                        torrents,
                        transfer,
                        shows,
                    },
                )
            }
            Err(error) => {
                {
                    let mut st = owner.downloads.state();
                    st.torrent.connected = false;
                    st.torrent.message = plain(&error);
                }
                owner.bus.debug(
                    Stage::Downloads,
                    format!("torrents not read: {}", plain(&error)),
                    EventBody::DownloadsUnavailable {
                        what: "torrents".to_string(),
                        error,
                    },
                )
            }
        };
    });
    Ok(Reply::Ok)
}

/// The episodes of one series that a torrent is still writing, by number,
/// with how far each is (0 to 1).
type Downloading = BTreeMap<u32, f64>;

/// How often the progress of an episode that keeps downloading is announced
/// as a change of its series, so a page that shows it moves without a
/// reload on every torrent read.
const PROGRESS_EVERY: std::time::Duration = std::time::Duration::from_secs(10);

/// How often the core looks at the torrents again by itself while anything
/// downloads, so an episode that just finished plays within seconds.
const REREAD_EVERY: std::time::Duration = std::time::Duration::from_secs(5);

/// After a torrent read: while anything still downloads, one more read in
/// `REREAD_EVERY`, and only one at a time however many reads land meanwhile.
fn schedule_reread(owner: &Arc<Core>) {
    let again = {
        let mut st = owner.downloads.state();
        let want = !st.downloading.is_empty() && !st.rereading;
        st.rereading |= want;
        want
    };
    if again {
        let later = owner.clone();
        owner.handle.spawn(async move {
            tokio::time::sleep(REREAD_EVERY).await;
            later.downloads.state().rereading = false;
            let _ = list_torrents(&later);
        });
    }
}

/// The episodes each linked series has in an unfinished torrent, and how far
/// each is: a file of a batch that is not done (and not skipped) with the
/// file's progress, or a single-episode torrent that is not done with the
/// torrent's. An episode two torrents carry counts at the further one.
fn downloading_of(torrents: &[Torrent]) -> HashMap<u64, Downloading> {
    let mut out: HashMap<u64, Downloading> = HashMap::new();
    for t in torrents.iter().filter(|t| t.progress < 1.0) {
        let Some(id) = t.series_id else { continue };
        let whole = |n: f64| (n.is_finite() && n >= 1.0 && n.fract() == 0.0).then_some(n as u32);
        let eps: Vec<(u32, f64)> = if t.files.is_empty() {
            t.episode
                .and_then(whole)
                .map(|n| (n, t.progress))
                .into_iter()
                .collect()
        } else {
            t.files
                .iter()
                .filter(|f| f.progress < 1.0 && f.priority != 0)
                .filter_map(|f| f.episode.and_then(whole).map(|n| (n, f.progress)))
                .collect()
        };
        for (n, progress) in eps {
            let progress = if progress.is_finite() {
                progress.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let at = out.entry(id).or_default().entry(n).or_insert(progress);
            *at = at.max(progress);
        }
    }
    out
}

/// A path as one key whatever string reaches it: resolved on disk when it
/// exists (a symlinked source, a doubled slash), else tidied as text.
fn path_key(p: &str) -> std::path::PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| std::path::Path::new(p).components().collect())
}

/// Every file an unfinished torrent is still writing: each unfinished,
/// wanted file of a batch, or a single file's content path.
fn writing_of(torrents: &[Torrent]) -> std::collections::HashSet<std::path::PathBuf> {
    let mut out = std::collections::HashSet::new();
    for t in torrents.iter().filter(|t| t.progress < 1.0) {
        if t.files.is_empty() {
            if !t.content_path.is_empty() {
                out.insert(path_key(&t.content_path));
            }
            continue;
        }
        for f in t
            .files
            .iter()
            .filter(|f| f.progress < 1.0 && f.priority != 0)
        {
            out.insert(path_key(&format!(
                "{}/{}",
                t.save_path.trim_end_matches('/'),
                f.name
            )));
        }
    }
    out
}

/// Whether a torrent is still writing this file.
pub fn is_writing(core: &Core, path: &str) -> bool {
    let st = core.downloads.state();
    !st.writing.is_empty() && st.writing.contains(&path_key(path))
}

/// The series whose downloads are worth a `SeriesChanged` now, and the
/// record of what went out updated to match: one whose set of downloading
/// episodes differs from the last announcement (or that finished), at once;
/// one whose only change is progress in whole percent, once `PROGRESS_EVERY`
/// has passed since the last.
fn due_for_announcing(
    announced: &mut HashMap<u64, (std::time::Instant, BTreeMap<u32, u8>)>,
    now_downloading: &HashMap<u64, Downloading>,
    at: std::time::Instant,
) -> Vec<u64> {
    let mut due: BTreeSet<u64> = BTreeSet::new();
    announced.retain(|id, _| {
        let kept = now_downloading.contains_key(id);
        if !kept {
            due.insert(*id);
        }
        kept
    });
    for (id, eps) in now_downloading {
        let pct: BTreeMap<u32, u8> = eps
            .iter()
            .map(|(n, p)| (*n, (p * 100.0).floor() as u8))
            .collect();
        let go = match announced.get(id) {
            None => true,
            Some((_, last)) if last.keys().ne(pct.keys()) => true,
            Some((sent, last)) => {
                *last != pct && at.saturating_duration_since(*sent) >= PROGRESS_EVERY
            }
        };
        if go {
            announced.insert(*id, (at, pct));
            due.insert(*id);
        }
    }
    due.into_iter().collect()
}

/// Lays the downloading episodes over a cube row: an episode a torrent is
/// still writing is `t`, not on disk, whatever the scan found. Watched
/// episodes keep their mark. Returns how many on-disk cubes it turned.
fn overlay_cubes(
    core: &Core,
    id: u64,
    cubes: &mut String,
    known: Option<&HashMap<u64, Downloading>>,
) -> u64 {
    let st;
    let map = match known {
        Some(m) => m,
        None => {
            st = core.downloads.state();
            &st.downloading
        }
    };
    match map.get(&id) {
        Some(eps) => lay(eps, cubes),
        None => 0,
    }
}

fn lay(eps: &Downloading, cubes: &mut String) -> u64 {
    let mut codes: Vec<char> = cubes.chars().collect();
    let mut turned = 0;
    for n in eps.keys() {
        if let Some(c) = codes.get_mut((*n as usize).wrapping_sub(1))
            && matches!(*c, 'd' | 'g' | 'm' | 'f')
        {
            if *c == 'd' {
                turned += 1;
            }
            *c = 't';
        }
    }
    *cubes = codes.into_iter().collect();
    turned
}

/// The library's cards with what is still downloading laid over them, and
/// a fresh torrent read asked for when the last one is stale and something
/// was downloading then. With nothing downloading it asks for nothing: the
/// Downloads page's poll is what starts the overlay.
pub fn overlay(core: &Core, cards: &mut [SeriesCard]) {
    let stale = {
        let st = core.downloads.state();
        for card in cards.iter_mut() {
            let turned = overlay_cubes(core, card.id, &mut card.cubes, Some(&st.downloading));
            card.episodes_on_disk = card.episodes_on_disk.saturating_sub(turned);
            card.downloading = st.downloading.get(&card.id).map_or_else(Vec::new, |eps| {
                eps.iter()
                    .map(|(n, p)| EpisodeDownload {
                        number: *n,
                        progress: *p,
                    })
                    .collect()
            });
        }
        !st.downloading.is_empty()
            && st
                .torrents_at
                .is_none_or(|at| at.elapsed() > std::time::Duration::from_secs(10))
    };
    if stale {
        core.downloads.state().torrents_at = Some(std::time::Instant::now());
        let _ = list_torrents(core);
    }
}

/// How far a torrent has written an episode of a series, while one is still
/// on it; None for an episode that is whole (or that no torrent carries).
pub fn episode_progress(core: &Core, series: u64, number: f64) -> Option<f64> {
    if !(number >= 1.0 && number.fract() == 0.0) {
        return None;
    }
    core.downloads
        .state()
        .downloading
        .get(&series)?
        .get(&(number as u32))
        .copied()
}

/// A series' detail with what is still downloading laid over it: the card as
/// `overlay` leaves it, each downloading episode marked with its progress
/// (a file being written is on disk and still not playable), and next up
/// moved past any of them to the next episode that is whole, else none.
pub fn overlay_detail(core: &Core, detail: &mut SeriesDetail) {
    overlay(core, std::slice::from_mut(&mut detail.card));
    mark_downloading(detail, &|path| is_writing(core, path));
}

/// Marks the episodes whose file a torrent is still writing. An episode
/// whose new copy is coming but whose file here is whole (a replacement)
/// stays playable.
fn mark_downloading(detail: &mut SeriesDetail, writing: &dyn Fn(&str) -> bool) {
    if detail.card.downloading.is_empty() {
        return;
    }
    let progress_of = |n: f64| {
        detail
            .card
            .downloading
            .iter()
            .find(|d| f64::from(d.number) == n)
            .map(|d| d.progress)
    };
    let marks: Vec<Option<f64>> = detail
        .episodes
        .iter()
        .map(|e| progress_of(e.number).filter(|_| writing(&e.path)))
        .collect();
    for (e, m) in detail.episodes.iter_mut().zip(&marks) {
        e.downloading = *m;
        if m.is_some() {
            e.next_up = false;
        }
    }
    let Some(next) = detail.next_up else { return };
    let Some(at) = detail.episodes.iter().position(|e| e.file == next) else {
        return;
    };
    if marks[at].is_none() {
        return;
    }
    let after = detail.episodes[at + 1..]
        .iter()
        .zip(&marks[at + 1..])
        .find(|(_, m)| m.is_none())
        .map(|(e, _)| e.file);
    detail.next_up = after;
    for e in detail.episodes.iter_mut() {
        e.next_up = Some(e.file) == after;
    }
}

/// Not a job: the show view asks once per batch that names no range.
pub fn inspect_release(core: &Core, link: String) -> Result<Reply, CoreError> {
    let link = link.trim().to_string();
    let owner = core.owner()?;
    core.handle.spawn(async move {
        let (episodes, files) = match contents::inspect(owner.http.as_ref(), &link).await {
            Ok(found) => found,
            Err(e) => {
                tracing::debug!("release not inspected: {}", plain(&e));
                (Vec::new(), 0)
            }
        };
        owner.bus.debug(
            Stage::Downloads,
            format!(
                "{} in a release",
                plural(episodes.len() as u64, "episode", "episodes")
            ),
            EventBody::ReleaseInspected {
                link,
                episodes,
                files,
            },
        );
    });
    Ok(Reply::Ok)
}

/// Not a job: opening a show reads its feed. The core fetches the feed
/// itself rather than through anirss, which has no route for it; the URL
/// is the one anirss registered, API key and all, and never leaves this
/// machine but to its own host.
pub fn read_feed(core: &Core, url: String) -> Result<Reply, CoreError> {
    let url = url.trim().to_string();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(CoreError::invalid("url", "a feed is an http(s) URL"));
    }
    let owner = core.owner()?;
    core.handle.spawn(async move {
        match feed::read(owner.http.as_ref(), &url).await {
            Ok(entries) => owner.bus.debug(
                Stage::Downloads,
                format!(
                    "{} read from {}",
                    plural(entries.len() as u64, "feed entry", "feed entries"),
                    feed::host_of(&url)
                ),
                EventBody::FeedRead { url, entries },
            ),
            Err(error) => owner.bus.debug(
                Stage::Downloads,
                format!("feed not read: {}", plain(&error)),
                EventBody::DownloadsUnavailable {
                    what: "feed".to_string(),
                    error,
                },
            ),
        };
    });
    Ok(Reply::Ok)
}

/// `Call::RepairSubscription`: a shorter search for the rule's feed, word
/// by word from the end, until the endpoint answers with releases; the
/// dropped words go into the rule. Nothing changes when the feed already
/// finds something, or when no shorter search does.
pub fn repair_subscription(core: &Core, name: String) -> Result<u64, CoreError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(CoreError::invalid("name", "a rule has a name"));
    }
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::RepairSubscription, move |ctx| async move {
            let qbt = qbt_client(&owner).await?;
            let rules = qbt.rules().await?;
            let def = rules.get(&name).cloned().ok_or_else(|| {
                service(qbt::SERVICE, "rule_missing", format!("no rule named {name}"))
            })?;
            let url = def["affectedFeeds"][0].as_str().unwrap_or("").to_string();
            let host = feed::host_of(&url);
            let (key, value) = rewire::search_param(&url).ok_or_else(|| {
                service(&host, "no_search", "the feed carries no search to shorten".to_string())
            })?;
            let now = feed::read(owner.http.as_ref(), &url).await?;
            if !now.is_empty() {
                return Ok(Finished {
                    level: Level::Info,
                    message: format!(
                        "{name} needs no repair: its feed finds {}",
                        plural(now.len() as u64, "release", "releases")
                    ),
                    body: EventBody::SubscriptionRepaired {
                        name,
                        feed_url: url,
                        must_contain: def["mustContain"].as_str().unwrap_or("").to_string(),
                        found: now.len() as u64,
                    },
                });
            }
            let fields = rewire::positive_fields(&value);
            let tries: Vec<usize> = (1..fields.len()).rev().take(4).collect();
            for (n, k) in tries.iter().enumerate() {
                let candidate = fields[..*k].join(" ");
                ctx.progress(n as u64, Some(tries.len() as u64), &format!("Trying \"{candidate}\""));
                let next = rewire::with_search(&url, &key, &candidate);
                let entries = feed::read(owner.http.as_ref(), &next).await?;
                if entries.is_empty() {
                    continue;
                }
                let terms = rewire::local_terms(&value, &candidate);
                let narrowed = rewire::narrowed(&def, &terms, &next);
                swap_feed(&qbt, &rules, &name, &narrowed, &url, &next).await?;
                let must = narrowed["mustContain"].as_str().unwrap_or("").to_string();
                return Ok(Finished {
                    level: Level::Info,
                    message: format!(
                        "Repaired {name}: {host} is searched for \"{candidate}\" ({}) and the rule keeps {}",
                        plural(entries.len() as u64, "release", "releases"),
                        describe_terms(&terms)
                    ),
                    body: EventBody::SubscriptionRepaired {
                        name,
                        feed_url: next,
                        must_contain: must,
                        found: entries.len() as u64,
                    },
                });
            }
            Err(service(
                &host,
                "repair_failed",
                format!(
                    "no shorter search finds anything; even \"{}\" comes back empty",
                    fields.first().cloned().unwrap_or_default()
                ),
            ))
        }))
}

/// Not a job: the show view reads its rule on open and while a fresh
/// subscription settles.
pub fn read_rule(core: &Core, name: String) -> Result<Reply, CoreError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(CoreError::invalid("name", "a rule has a name"));
    }
    let owner = core.owner()?;
    core.handle.spawn(async move {
        let outcome = async {
            let qbt = qbt_client(&owner).await?;
            let rules = qbt.rules().await?;
            let def = rules.get(&name).cloned().ok_or_else(|| {
                service(
                    qbt::SERVICE,
                    "rule_missing",
                    format!("no rule named {name}"),
                )
            })?;
            let matching = qbt.matching_articles(&name).await?;
            let text = |k: &str| def[k].as_str().unwrap_or("").to_string();
            Ok::<_, CoreError>(RuleView {
                name: name.clone(),
                enabled: def["enabled"].as_bool().unwrap_or(true),
                must_contain: text("mustContain"),
                must_not_contain: text("mustNotContain"),
                save_path: def["savePath"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .or_else(|| def["torrentParams"]["save_path"].as_str())
                    .unwrap_or("")
                    .to_string(),
                feeds: def["affectedFeeds"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|u| u.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                last_match: text("lastMatch"),
                matching,
            })
        }
        .await;
        match outcome {
            Ok(rule) => owner.bus.debug(
                Stage::Downloads,
                format!(
                    "rule {} read: {} accepted",
                    rule.name,
                    plural(rule.matching.len() as u64, "article", "articles")
                ),
                EventBody::RuleRead { rule },
            ),
            Err(error) => owner.bus.debug(
                Stage::Downloads,
                format!("rule not read: {}", plain(&error)),
                EventBody::DownloadsUnavailable {
                    what: "rule".to_string(),
                    error,
                },
            ),
        };
    });
    Ok(Reply::Ok)
}

/// Fills each rule's `show` from the library. A library that cannot be
/// read leaves them empty: the rules still stand on their own.
fn link_rules(core: &Core, rules: &mut [SubscriptionRule]) {
    let images = core.paths.images_dir();
    let linked = core.store.read(|c| {
        let linker = link::Linker::load(c)?;
        let ids: Vec<Option<u64>> = rules.iter().map(|r| linker.find(&r.save_path)).collect();
        let wanted: Vec<u64> = ids.iter().flatten().copied().collect();
        let shows = link::shows(c, &images, crate::time::now(), &wanted)?;
        Ok((ids, shows))
    });
    match linked {
        Ok((ids, shows)) => {
            for (rule, id) in rules.iter_mut().zip(ids) {
                rule.show = id.and_then(|id| shows.get(&id).cloned());
            }
        }
        Err(e) => tracing::warn!("subscriptions not linked to the library: {e}"),
    }
}

/// Sets each torrent's `series_id` and returns those series, once each.
fn link_torrents(core: &Core, torrents: &mut [Torrent]) -> Vec<ShowLink> {
    let images = core.paths.images_dir();
    let linked = core.store.read(|c| {
        let linker = link::Linker::load(c)?;
        let ids: Vec<Option<u64>> = torrents.iter().map(|t| linker.find_torrent(t)).collect();
        let wanted: Vec<u64> = ids.iter().flatten().copied().collect();
        let shows = link::shows(c, &images, crate::time::now(), &wanted)?;
        Ok((ids, shows))
    });
    match linked {
        Ok((ids, shows)) => {
            for (t, id) in torrents.iter_mut().zip(ids) {
                t.series_id = id;
                let f = facts::read_name(&t.name);
                t.batch = f.batch || !t.files.is_empty();
                t.episode = if t.batch { None } else { f.episode };
                for file in t.files.iter_mut() {
                    file.episode = facts::read_name(&file.name).episode;
                }
            }
            let mut out: Vec<ShowLink> = shows.into_values().collect();
            out.sort_by_key(|s| s.series_id);
            out
        }
        Err(e) => {
            tracing::warn!("torrents not linked to the library: {e}");
            Vec::new()
        }
    }
}

/// Saves the settings and the two secrets, each secret three-state as
/// `SetTrackerCredentials` has it, then drops everything built from the
/// old ones: the sidecar (its environment carried the password), the
/// torrent client's session, the last status.
pub fn set_settings(
    core: &Core,
    mut settings: DownloadSettings,
    anirss_token: Option<&str>,
    torrent_password: Option<&str>,
) -> Result<Reply, CoreError> {
    // AniBeam's own listens on this machine; an address left over from a
    // client elsewhere would have it started at somebody else's port.
    // The same for one of the user's own on this machine: its port is theirs.
    if settings.torrent_mode == TorrentMode::Managed && !settings::fits_managed(&settings) {
        settings.torrent_url = own_address(core)?;
        settings.torrent_username = managed::USERNAME.to_string();
    }
    let before = core.store.read(settings::load_stored)?;
    settings::validate(&mut settings, &before.saved_clients)?;
    match anirss_token.map(str::trim) {
        Some("") => {
            core.secrets().delete(settings::ANIRSS_TOKEN)?;
        }
        Some(token) => {
            core.secrets().set(settings::ANIRSS_TOKEN, token)?;
        }
        None => {}
    }
    // A password is the one secret where whitespace can be the point, so
    // it is not trimmed; only an empty one is a removal. It is the password
    // of the client in use; AniBeam's own has none to set.
    if let Some(key) = settings::active_password_key(&settings) {
        match torrent_password {
            Some("") => {
                core.secrets().delete(&key)?;
            }
            Some(pw) => {
                core.secrets().set(&key, pw)?;
            }
            None => {}
        }
    }
    let gone: Vec<String> = before
        .saved_clients
        .iter()
        .filter(|old| !settings.saved_clients.iter().any(|c| c.id == old.id))
        .map(|old| settings::password_key(&old.id))
        .collect();
    restart(core, settings)?;
    // A client taken off the list takes its password with it, once the
    // list without it is saved: a failed save keeps both.
    for key in gone {
        core.secrets().delete(&key)?;
    }
    let status = status(core)?;
    core.bus.debug(
        Stage::Downloads,
        "downloads settings changed",
        EventBody::DownloadsStatusChanged { status },
    );
    Ok(Reply::Ok)
}

/// Switches the downloads to the saved client `id`, or to AniBeam's own
/// when there is none. The rest of the settings stay as they are.
pub fn use_client(core: &Core, id: Option<String>) -> Result<Reply, CoreError> {
    let mut s = core.store.read(settings::load_stored)?;
    let in_use = match id.as_deref() {
        Some(id) => s.torrent_mode == TorrentMode::Existing && s.active_client == id,
        None => s.torrent_mode == TorrentMode::Managed,
    };
    if in_use {
        // Picking the one in use keeps its sidecar and its session
        return Ok(Reply::Ok);
    }
    let name = match id.as_deref() {
        Some(id) => {
            let c = s.saved_clients.iter().find(|c| c.id == id).ok_or_else(|| {
                CoreError::invalid("id", format!("no saved torrent client {id:?}"))
            })?;
            let name = c.name.clone();
            s.torrent_url = c.url.clone();
            s.torrent_username = c.username.clone();
            s.torrent_mode = TorrentMode::Existing;
            s.active_client = id.to_string();
            name
        }
        None => {
            s.torrent_url = own_address(core)?;
            s.torrent_username = managed::USERNAME.to_string();
            s.torrent_mode = TorrentMode::Managed;
            "AniBeam's own qBittorrent".to_string()
        }
    };
    settings::validate(&mut s, &[])?;
    restart(core, s)?;
    let status = status(core)?;
    core.bus.info(
        Stage::Downloads,
        format!("Downloads use {name}"),
        EventBody::DownloadsStatusChanged { status },
    );
    Ok(Reply::Ok)
}

/// Saves the settings and drops everything built from the old ones: the
/// sidecar (its environment names the client), both statuses, the endpoints
/// and the torrent client's session.
fn restart(core: &Core, settings: DownloadSettings) -> Result<(), CoreError> {
    core.store.write(move |c| settings::save(c, &settings))?;
    let mut st = core.downloads.state();
    if let Some(mut sc) = st.sidecar.take() {
        sc.stop();
    }
    st.anirss = AnirssStatus::default();
    st.torrent = TorrentClientStatus::default();
    st.endpoints.clear();
    st.qbt = None;
    st.generation += 1;
    Ok(())
}

impl Downloads {
    /// The torrent client's name for a log line, off the last status.
    fn torrent_name(&self) -> &'static str {
        TorrentClient::Qbittorrent.as_str()
    }
}

fn plural(n: u64, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// The two reads the tailnet host serves, each with the library's links
/// filled in and each held to `limit`, so a stopped anirss or a hung
/// torrent client costs a snapshot one half and some seconds, never the
/// whole. A timeout is a `Service` error with the code `timeout`.
pub(crate) async fn host_read(
    core: &Arc<Core>,
    limit: std::time::Duration,
) -> (
    Result<Vec<SubscriptionRule>, CoreError>,
    Result<Vec<Torrent>, CoreError>,
) {
    // The host only reads: it uses a client that is already up and never
    // starts, decides or saves anything for a device that asked.
    let idle = |what: &str| {
        service(
            what,
            "not_running",
            "is not running on the host".to_string(),
        )
    };
    let rules = async {
        let client = running_client(core).await?.ok_or_else(|| idle("anirss"))?;
        let mut rules = client.subscriptions().await?;
        link_rules(core, &mut rules);
        Ok(rules)
    };
    let torrents = async {
        // A client the user runs is only asked; AniBeam's own, or a mode
        // not decided yet, waits for the user to open Downloads here.
        let held = core.downloads.state().qbt.clone();
        let client = match held {
            Some(c) => c,
            None if load_settings(core).await?.torrent_mode == TorrentMode::Existing => {
                qbt_client(core).await?
            }
            None => return Err(idle("qBittorrent")),
        };
        let mut torrents = client.torrents().await?;
        link_torrents(core, &mut torrents);
        Ok(torrents)
    };
    let late = |what: &str| service(what, "timeout", "did not answer in time".to_string());
    let (rules, torrents) = tokio::join!(
        tokio::time::timeout(limit, rules),
        tokio::time::timeout(limit, torrents)
    );
    (
        rules.unwrap_or_else(|_| Err(late("anirss"))),
        torrents.unwrap_or_else(|_| Err(late("qBittorrent"))),
    )
}

#[cfg(test)]
mod tests {

    #[test]
    fn a_batch_downloads_its_unfinished_files_and_lays_them_over_the_cubes() {
        let file = |name: &str, progress: f64| TorrentFile {
            name: name.to_string(),
            size: 1,
            progress,
            priority: 1,
            episode: super::facts::read_name(name).episode,
        };
        let dir = "[Erai-raws] Kino no Tabi The Beautiful World - The Animated Series - 01~12 [1080p][Multiple Subtitle]";
        let mut t = super::qbt::tests_torrent();
        t.series_id = Some(7);
        t.progress = 0.4;
        t.files = vec![
            file(
                &format!(
                    "{dir}/[Erai-raws] Kino no Tabi The Beautiful World - The Animated Series - 01 [1080p][Multiple Subtitle].mkv"
                ),
                1.0,
            ),
            file(
                &format!(
                    "{dir}/[Erai-raws] Kino no Tabi The Beautiful World - The Animated Series - 02 [1080p][Multiple Subtitle].mkv"
                ),
                0.3,
            ),
            file(
                &format!(
                    "{dir}/[Erai-raws] Kino no Tabi The Beautiful World - The Animated Series - 03 [1080p][Multiple Subtitle].mkv"
                ),
                0.0,
            ),
        ];
        assert_eq!(t.files[1].episode, Some(2.0));
        let map = downloading_of(&[t]);
        assert_eq!(map[&7], Downloading::from([(2, 0.3), (3, 0.0)]));
        let mut cubes = "dddgw".to_string();
        let eps: Downloading = [2, 3, 5, 9].into_iter().map(|n| (n, 0.5)).collect();
        assert_eq!(lay(&eps, &mut cubes), 2);
        assert_eq!(
            cubes, "dttgw",
            "a watched cube keeps its mark, one past the row is ignored"
        );
    }

    #[test]
    fn a_single_episode_torrent_downloads_at_the_torrents_progress() {
        let mut t = super::qbt::tests_torrent();
        t.series_id = Some(4);
        t.episode = Some(6.0);
        t.progress = 0.34;
        let mut twin = t.clone();
        twin.progress = 0.5;
        let mut done = t.clone();
        done.episode = Some(7.0);
        done.progress = 1.0;
        let map = downloading_of(&[t, twin, done]);
        assert_eq!(
            map[&4],
            Downloading::from([(6, 0.5)]),
            "the further torrent counts, a finished one does not"
        );
    }

    #[test]
    fn announcing_follows_the_set_at_once_and_progress_every_ten_seconds() {
        let t0 = std::time::Instant::now();
        let mut sent = HashMap::new();
        let at = |eps: &[(u32, f64)]| {
            HashMap::from([(1u64, eps.iter().copied().collect::<Downloading>())])
        };
        assert_eq!(
            due_for_announcing(&mut sent, &at(&[(2, 0.10)]), t0),
            vec![1],
            "first sight"
        );
        let s = std::time::Duration::from_secs;
        assert!(
            due_for_announcing(&mut sent, &at(&[(2, 0.12)]), t0 + s(3)).is_empty(),
            "too soon"
        );
        assert!(
            due_for_announcing(&mut sent, &at(&[(2, 0.109)]), t0 + s(11)).is_empty(),
            "the same percent"
        );
        assert_eq!(
            due_for_announcing(&mut sent, &at(&[(2, 0.20)]), t0 + s(12)),
            vec![1],
            "ten seconds on"
        );
        assert!(due_for_announcing(&mut sent, &at(&[(2, 0.30)]), t0 + s(14)).is_empty());
        assert_eq!(
            due_for_announcing(&mut sent, &at(&[(2, 0.30), (3, 0.0)]), t0 + s(14)),
            vec![1],
            "a new episode is not throttled"
        );
        assert_eq!(
            due_for_announcing(&mut sent, &HashMap::new(), t0 + s(15)),
            vec![1],
            "finished"
        );
        assert!(sent.is_empty());
    }

    fn detail_of(card_downloading: &[(u32, f64)], next_up: Option<u64>) -> SeriesDetail {
        let json = serde_json::json!({
            "card": {
                "id": 1, "kind": "Show", "path": "", "title": "", "titles": {"folder": ""},
                "genres": [], "hidden": false, "missing": false, "episodes_on_disk": 0,
                "extras_on_disk": 0, "total_is_estimate": false, "watched_state": "Unknown",
                "strip": {"watched": 0.0, "aired_unwatched": 0.0, "unknown": 0.0},
                "latest_activity_at": {"secs_since_epoch": 0, "nanos_since_epoch": 0}
            },
            "genres": [], "tags": [], "synopsis": "",
            "progress": {"estimate": false, "on_disk": 0},
            "next_up": next_up,
            "episodes": (1..=4).map(|n| serde_json::json!({
                "file": 100 + n, "number": n as f64, "code": "", "path": "", "sidecars": [],
                "stopped_short": false, "watched": false, "next_up": Some(100 + n as u64) == next_up
            })).collect::<Vec<_>>(),
            "extras": [], "unmatched_files": [], "characters": [], "recommendations": [],
            "has_graph": false
        });
        let mut d: SeriesDetail = serde_json::from_value(json).expect("a detail");
        d.card.downloading = card_downloading
            .iter()
            .map(|(number, progress)| EpisodeDownload {
                number: *number,
                progress: *progress,
            })
            .collect();
        d
    }

    #[test]
    fn a_downloading_episode_is_marked_and_never_next_up() {
        let mut d = detail_of(&[(2, 0.34), (3, 0.0)], Some(102));
        mark_downloading(&mut d, &|_| true);
        let marks: Vec<Option<f64>> = d.episodes.iter().map(|e| e.downloading).collect();
        assert_eq!(marks, vec![None, Some(0.34), Some(0.0), None]);
        assert_eq!(
            d.next_up,
            Some(104),
            "next up moves past the downloading run"
        );
        assert_eq!(
            d.episodes.iter().map(|e| e.next_up).collect::<Vec<_>>(),
            vec![false, false, false, true]
        );
        let mut d = detail_of(&[(4, 0.5)], Some(104));
        mark_downloading(&mut d, &|_| true);
        assert_eq!(d.next_up, None, "nothing whole after it");
        assert!(d.episodes.iter().all(|e| !e.next_up));
        // A replacement: episode 2 has a new copy coming, but the file here is whole
        let mut d = detail_of(&[(2, 0.34)], Some(102));
        mark_downloading(&mut d, &|_| false);
        assert!(
            d.episodes.iter().all(|e| e.downloading.is_none()),
            "the old file plays"
        );
        assert_eq!(d.next_up, Some(102));
        let mut d = detail_of(&[(4, 0.5)], Some(101));
        mark_downloading(&mut d, &|_| true);
        assert_eq!(d.next_up, Some(101), "a whole next up stays");
        let mut d = detail_of(&[], Some(102));
        mark_downloading(&mut d, &|_| true);
        assert!(d.episodes.iter().all(|e| e.downloading.is_none()) && d.next_up == Some(102));
    }
    use super::*;

    #[test]
    fn describe_names_both_halves() {
        let mut status = DownloadsStatus {
            settings: settings::defaults(),
            anirss: AnirssStatus::default(),
            torrent: TorrentClientStatus::default(),
            endpoints: Vec::new(),
        };
        status.anirss.version = "0.3.4".to_string();
        status.torrent.message = "unreachable".to_string();
        assert_eq!(
            describe(&status, None),
            "anirss 0.3.4 ready; qBittorrent not connected: unreachable"
        );
        status.torrent.connected = true;
        status.torrent.version = "v5.2.4".to_string();
        assert_eq!(
            describe(&status, Some("anirss exited")),
            "anirss is not available: anirss exited; qBittorrent v5.2.4 connected"
        );
    }

    #[test]
    fn plural_reads_right() {
        assert_eq!(plural(1, "torrent", "torrents"), "1 torrent");
        assert_eq!(plural(0, "torrent", "torrents"), "0 torrents");
    }
}
