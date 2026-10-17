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
pub mod facts;
pub mod feed;
pub mod link;
pub mod managed;
pub mod qbt;
pub mod rewire;
pub mod settings;
pub mod trash;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
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
}

impl Default for Downloads {
    fn default() -> Self {
        Downloads {
            state: Mutex::new(State::default()),
            connecting: tokio::sync::Mutex::new(()),
            torrent_setup: tokio::sync::Mutex::new(()),
        }
    }
}

impl Downloads {
    fn state(&self) -> MutexGuard<'_, State> {
        recover(&self.state)
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
    let (has_token, has_password) = with_secrets(core, |s| {
        Ok((
            s.has(settings::ANIRSS_TOKEN),
            s.has(settings::TORRENT_PASSWORD),
        ))
    })
    .await?;
    let mut s = core.store.read(settings::load_stored)?;
    s.has_anirss_token = has_token;
    s.has_torrent_password = has_password;
    Ok(s)
}

// The clients ----------------------------------------------------------------

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
    let _one_at_a_time = core.downloads.connecting.lock().await;
    {
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
    }
    // anirss is pointed at the client the core uses, so that has to be
    // settled first. It still starts without one: the page then says why
    // the torrent half is down.
    let s = match ensure_torrent(core).await {
        Ok(settled) => settled,
        Err(_) => load_settings(core).await?,
    };
    let (password, _) = resolve_password(core).await?;
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
        Ok(version) => {
            let mut st = core.downloads.state();
            st.anirss.state = AnirssState::Ready;
            st.anirss.version = version;
            st.anirss.url = sidecar.server();
            st.anirss.message.clear();
            st.sidecar = Some(sidecar);
            Ok(client)
        }
        Err(message) => {
            // An anirss from before the server mode exits on `--serve`
            // with a usage line that says nothing about versions.
            let message = match anirss::installed_version(&command)
                .await
                .and_then(|text| anirss::too_old(&text))
            {
                Some(version) => format!(
                    "anirss {version} is too old for AniBeam, which needs {} or newer. {}.",
                    anirss::MIN_VERSION,
                    anirss::update_hint()
                ),
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

/// The torrent client's password and where it came from: the one saved
/// here, else the one anirss's CLI saved, else none.
async fn resolve_password(core: &Arc<Core>) -> Result<(Option<String>, &'static str), CoreError> {
    let saved = with_secrets(core, |s| {
        Ok(s.get(settings::TORRENT_PASSWORD, None)?.map(|(v, _)| v))
    })
    .await?;
    if let Some(pw) = saved {
        return Ok((Some(pw), "anibeam"));
    }
    match settings::anirss_saved_password() {
        Some(pw) => Ok((Some(pw), "anirss")),
        None => Ok((None, "none")),
    }
}

/// The torrent client, built on first use and kept with its session.
async fn qbt_client(core: &Arc<Core>) -> Result<Arc<QbtClient>, CoreError> {
    if let Some(c) = core.downloads.state().qbt.clone() {
        return Ok(c);
    }
    let s = ensure_torrent(core).await?;
    let (password, source) = resolve_password(core).await?;
    let client = Arc::new(QbtClient::new(
        &s.torrent_url,
        &s.torrent_username,
        password.as_deref().unwrap_or(""),
        core.http.clone(),
    ));
    let mut st = core.downloads.state();
    st.torrent.password_source = source.to_string();
    st.qbt = Some(client.clone());
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
    let (password, _) = resolve_password(core).await?;
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
    let message = if answered {
        s.torrent_mode = TorrentMode::Existing;
        format!("Downloads use the qBittorrent at {}", s.torrent_url)
    } else if managed::binary().is_some() {
        let port = anirss::free_port()
            .map_err(|e| service(qbt::SERVICE, "failed", format!("no free port: {e}")))?;
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

/// Health, endpoints and anirss's view of qBittorrent, into the state.
async fn read_anirss(core: &Arc<Core>, client: &AnirssClient) -> Result<(), CoreError> {
    let version = client.health().await?;
    let mut endpoints = client.endpoints().await?;
    if let Some(path) = settings::anirss_config_path() {
        for e in &mut endpoints {
            e.on_demand = settings::anirss_ondemand(&path, &e.name).map(|o| o.timeout);
        }
    }
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

pub fn search(core: &Core, query: String, endpoint: Option<String>) -> Result<u64, CoreError> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err(CoreError::invalid("query", "type something to search for"));
    }
    let owner = core.owner()?;
    // The endpoint the search starts on, for the progress line: the one
    // asked for, else the first anirss lists
    let first = endpoint.clone().or_else(|| {
        owner
            .downloads
            .state()
            .endpoints
            .first()
            .map(|e| e.name.clone())
    });
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
                phase.store(PHASE_FETCHING, Ordering::Relaxed);
                let raw = client.search(&query, endpoint.as_deref(), true).await?;
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
fn annotate(core: &Core, query: &str, items: &[Release]) -> (Vec<ReleaseFacts>, Option<ShowLink>) {
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
            for f in &mut facts {
                if let Some(n) = f.episode {
                    f.on_disk = link::has_episode(&files, f.season, n);
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
        .start(JobKind::Subscribe, move |_| async move {
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
                rewire_subscription(
                    &owner,
                    &client,
                    &mut result,
                    &wire,
                    &endpoint,
                    &items,
                    &terms,
                )
                .await?;
                format!(
                    "Subscribed to {}; {endpoint} is searched for \"{wire}\" and the rule keeps {}",
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

/// Points anirss's new rule at the feed the search read and puts the
/// narrowing words in the rule. In this order, so qBittorrent never holds
/// the rule unnarrowed beside a feed with articles: the rule first (its old
/// feed found nothing, and its new one is not there yet), then the empty
/// feed out (unless another rule reads it), then the search's feed in.
async fn rewire_subscription(
    core: &Arc<Core>,
    client: &AnirssClient,
    result: &mut Subscribed,
    wire: &str,
    endpoint: &str,
    items: &[Release],
    terms: &rewire::LocalTerms,
) -> Result<(), CoreError> {
    let spec = client.preview(wire, endpoint, items).await?;
    if spec.feed_url.is_empty() || spec.feed_url == result.feed_url {
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
    let def = rewire::narrowed(def, terms, &spec.feed_url);
    qbt.set_rule(&result.name, &def).await?;
    let feeds = qbt.feeds().await?;
    let old = feeds
        .iter()
        .find(|(_, url)| *url == result.feed_url)
        .map(|(path, _)| path.clone());
    if let Some(path) = &old
        && !rewire::feed_shared(&rules, &result.name, &result.feed_url)
    {
        qbt.remove_item(path).await?;
    }
    qbt.add_feed(&spec.feed_url, old.as_deref().unwrap_or(&result.name))
        .await?;
    result.feed_url = spec.feed_url;
    result.must_contain = def["mustContain"].as_str().unwrap_or("").to_string();
    result.must_not_contain = def["mustNotContain"].as_str().unwrap_or("").to_string();
    Ok(())
}

pub fn download(core: &Core, items: Vec<Release>, name: Option<String>) -> Result<u64, CoreError> {
    if items.is_empty() {
        return Err(CoreError::invalid("items", "nothing selected to download"));
    }
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::Download, move |_| async move {
            let client = ensure_client(&owner).await?;
            let result = client.download(&items, name.as_deref()).await?;
            Ok(Finished {
                level: Level::Info,
                message: format!(
                    "Added {} to {} under {}",
                    plural(result.count, "torrent", "torrents"),
                    owner.downloads.torrent_name(),
                    result.save_path
                ),
                body: EventBody::TorrentsAdded { result },
            })
        }))
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
            // What the torrents wrote, read before they go, and the series' folder
            let mut candidates = Vec::new();
            if takes_files && !hashes.is_empty() {
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
            let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
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
                // Never the client's own delete-with-files: what goes, goes to the trash
                qbt_client(&owner).await?.delete(&hashes, false).await?;
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
            if !targets.is_empty() {
                parts.push(format!(
                    "{} to the trash",
                    plural(targets.len() as u64, "path", "paths")
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
            Ok(client) => client.subscriptions().await,
            Err(e) => Err(e),
        };
        match outcome {
            Ok(mut rules) => {
                link_rules(&owner, &mut rules);
                owner.bus.debug(
                    Stage::Downloads,
                    format!(
                        "{} read",
                        plural(rules.len() as u64, "subscription", "subscriptions")
                    ),
                    EventBody::SubscriptionsRead { rules },
                )
            }
            Err(error) => owner.bus.debug(
                Stage::Downloads,
                format!("subscriptions not read: {}", plain(&error)),
                EventBody::DownloadsUnavailable {
                    what: "subscriptions".to_string(),
                    error,
                },
            ),
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
            let torrents = client.torrents().await?;
            let transfer = client.transfer().await?;
            Ok::<_, CoreError>((torrents, transfer))
        }
        .await;
        match outcome {
            Ok((mut torrents, transfer)) => {
                let shows = link_torrents(&owner, &mut torrents);
                {
                    let mut st = owner.downloads.state();
                    st.torrent.connected = true;
                    st.torrent.message.clear();
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
                qbt.set_rule(&name, &narrowed).await?;
                let feeds = qbt.feeds().await?;
                let old = feeds.iter().find(|(_, u)| *u == url).map(|(p, _)| p.clone());
                if let Some(path) = &old
                    && !rewire::feed_shared(&rules, &name, &url)
                {
                    qbt.remove_item(path).await?;
                }
                qbt.add_feed(&next, old.as_deref().unwrap_or(&name)).await?;
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
    settings::validate(&mut settings)?;
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
    // it is not trimmed; only an empty one is a removal.
    match torrent_password {
        Some("") => {
            core.secrets().delete(settings::TORRENT_PASSWORD)?;
        }
        Some(pw) => {
            core.secrets().set(settings::TORRENT_PASSWORD, pw)?;
        }
        None => {}
    }
    let stored = settings.clone();
    core.store.write(move |c| settings::save(c, &stored))?;
    {
        let mut st = core.downloads.state();
        if let Some(mut sc) = st.sidecar.take() {
            sc.stop();
        }
        st.anirss = AnirssStatus::default();
        st.torrent = TorrentClientStatus::default();
        st.endpoints.clear();
        st.qbt = None;
    }
    let status = status(core)?;
    core.bus.debug(
        Stage::Downloads,
        "downloads settings changed",
        EventBody::DownloadsStatusChanged { status },
    );
    Ok(Reply::Ok)
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

#[cfg(test)]
mod tests {
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
