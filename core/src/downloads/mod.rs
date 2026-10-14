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
pub mod feed;
pub mod link;
pub mod qbt;
pub mod settings;

use std::sync::{Arc, Mutex, MutexGuard};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::Finished;
use crate::sync::recover;
use crate::trackers::accounts::with_secrets;
use anirss::{AnirssClient, Sidecar, SpawnFailure};
use qbt::QbtClient;

pub struct Downloads {
    state: Mutex<State>,
    /// One sidecar start at a time: two jobs that both find it down must
    /// not each spawn one.
    connecting: tokio::sync::Mutex<()>,
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
    let (password, _) = resolve_password(core).await?;
    let mut sidecar = match Sidecar::spawn(&s.anirss_command, password.as_deref()) {
        Ok(sc) => sc,
        Err(SpawnFailure::Missing) => {
            let message = format!(
                "{} was not found. Install anirss, or point Settings at a running server.",
                s.anirss_command
            );
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
    let s = load_settings(core).await?;
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
    let status = status_async(&core).await?;
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
    let endpoints = client.endpoints().await?;
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
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::SearchReleases, move |_| async move {
            let client = ensure_client(&owner).await?;
            let raw = client.search(&query, endpoint.as_deref(), true).await?;
            let search = if raw.items.is_empty() {
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
                }
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
            let refinement = client.apply(&query, &endpoint, &items, &op).await?;
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
) -> Result<u64, CoreError> {
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::Subscribe, move |_| async move {
            let client = ensure_client(&owner).await?;
            let result = client
                .subscribe(&query, &endpoint, &items, name.as_deref())
                .await?;
            Ok(Finished {
                level: Level::Info,
                message: format!(
                    "Subscribed to {}; new episodes save to {}",
                    result.name, result.save_path
                ),
                body: EventBody::Subscribed { result },
            })
        }))
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
