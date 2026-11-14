//! Get media (#889): a member whose library lacks the room's episode asks the
//! host's app for it, and downloads what comes back.
//!
//! Everything that leads to the video travels sealed (`seal`): the relay passes
//! the payloads on and reads none of them. The member asks (`Secret::Ask`); the
//! host's app looks up where its own file came from and answers with an
//! `Offer`. When the torrent client still holds the torrent the file came from
//! (`downloads::origins`), the offer carries its magnet, trackers included, and
//! the member hands that to its own torrent client: the host is a peer. When it
//! does not, the offer carries the release's name, group and resolution, and the
//! member searches its own sources for the same release and downloads the one
//! that fits best (`pick`). Either way the download goes through the downloads
//! engine (`downloads::download_telling`), saved into the member's copy of the
//! show when the library has it, else into a folder named for the show.
//!
//! While it downloads, the member's status carries its progress and time left,
//! which is all the relay and the others see. Once done, the core waits for the
//! library to take the file in, points the show at the room's match when it
//! arrived as a show the library did not know, and says `RoomEpisode` with the
//! file, so the shell opens it and the member is ready.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anibeam_room::proto::{self, ClientMsg, MemberId, Progress};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use super::{Active, Say, emit, episode, error, not_in_room};
use crate::contract::*;
use crate::core::Core;
use crate::downloads::{self, backend::DownloadsBackend, facts, origins, replace};
use crate::releases::titles;

/// How long the host's app has to answer before Get media stops waiting.
const ASK_WAIT: Duration = Duration::from_secs(30);

/// How often the member reads its download's progress from the torrent client.
const POLL: Duration = Duration::from_secs(2);

/// How long the torrent client may go without listing a download that was just
/// handed to it, before it counts as lost.
const LIST_WAIT: Duration = Duration::from_secs(120);

/// How long a finished download has to show in the library.
const ARRIVE_WAIT: Duration = Duration::from_secs(300);

/// When a finished download is still not in the library, a scan is asked for
/// once, this long after it finished: the watcher usually brings it sooner.
const SCAN_AFTER: Duration = Duration::from_secs(10);

/// What two members seal to each other.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "k", rename_all = "snake_case")]
pub enum Secret {
    /// A member without the episode asks the host's app for it.
    Ask { episode_key: String },
    /// What the host has of it.
    Offer(Offer),
    /// The host has no copy to point at.
    Nothing { episode_key: String },
}

/// Where the host's copy came from. Only ever sealed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    /// The room episode this is for, so an answer to an old ask is dropped.
    pub episode_key: String,
    /// The torrent, trackers included, while the host's client still holds it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magnet: Option<String>,
    /// The info hash the file came from, when known: a search result with the
    /// same hash is the same release whatever it is called.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// The release's name: the torrent's, else the file's.
    pub name: String,
    /// The file's own name.
    pub file: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub resolution: String,
    /// The host's match for the show, so the member's copy lands matched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anilist_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mal_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tvmaze_id: Option<i64>,
}

/// Why a Get media stopped short.
#[derive(Debug)]
enum Why {
    Nothing,
    NotFound,
    Start(String),
    Lost,
    NotInLibrary,
}

impl Why {
    fn words(&self) -> Words {
        match self {
            Why::Nothing => crate::words!("core-room-media-nothing"),
            Why::NotFound => crate::words!("core-room-media-not-found"),
            Why::Start(message) => {
                crate::words!("core-room-media-no-start", message = message.as_str())
            }
            Why::Lost => crate::words!("core-room-media-lost"),
            Why::NotInLibrary => crate::words!("core-room-media-not-in-library"),
        }
    }
}

// ---------------------------------------------------------------------------
// The member asks
// ---------------------------------------------------------------------------

/// `RoomGetMedia`: seals an ask to the host's app and waits for its answer.
pub(super) fn get(core: &Core) -> Result<Option<RoomInfo>, CoreError> {
    let mut out = Vec::new();
    let info = {
        let mut slot = core.rooms.lock();
        let a = slot.as_mut().ok_or_else(not_in_room)?;
        if !a.can_fetch() {
            return Err(error(
                "cannot_fetch",
                crate::words!("core-room-media-cannot"),
            ));
        }
        let (host, theirs) = a.host_key().ok_or_else(not_in_room)?;
        let e = a.episode.clone().ok_or_else(not_in_room)?;
        let ask = Secret::Ask {
            episode_key: e.episode_key.clone(),
        };
        let plain = serde_json::to_vec(&ask).map_err(|e| CoreError::internal(e.to_string()))?;
        let data = a
            .keys
            .seal(&a.code, &theirs, &plain)
            .ok_or_else(|| error("cannot_fetch", crate::words!("core-room-media-cannot")))?;
        a.send(ClientMsg::Seal { to: host, data });
        let id = begin(a, RoomFetch::Asking);
        // An app from before Get media, or one gone quiet, never answers
        let (owner, serial) = (a.core.clone(), a.serial);
        core.handle.spawn(async move {
            tokio::time::sleep(ASK_WAIT).await;
            if let Some(core) = owner.upgrade() {
                let late = |a: &Active| a.fetch == RoomFetch::Asking;
                stop(&core, serial, id, late, Why::Nothing);
            }
        });
        out.push(Say::said(
            Level::Info,
            crate::words!("core-room-media-asked"),
            format!(
                "Room {}: asked the host for {} {}",
                a.code, e.series_title, e.episode_key
            ),
            a.changed(),
        ));
        a.info()
    };
    emit(core, out);
    Ok(info)
}

/// Numbers each fetch, so a task that outlived its fetch finds it gone.
static NEXT: AtomicU64 = AtomicU64::new(1);

/// Starts a new fetch in `a`, stopping the one before: its number.
fn begin(a: &mut Active, fetch: RoomFetch) -> u64 {
    a.stop_fetch();
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    a.fetching = Some((id, CancellationToken::new()));
    a.fetch = fetch;
    id
}

impl Active {
    /// The fetch `id` is the one running in this room.
    fn is_fetch(&self, serial: u64, id: u64) -> bool {
        self.serial == serial && self.fetching.as_ref().is_some_and(|(f, _)| *f == id)
    }
}

/// A sealed payload opened: what it asks of this member.
pub(super) fn on_secret(core: &Core, serial: u64, from: MemberId, secret: Secret) {
    match secret {
        Secret::Ask { episode_key } => {
            let Ok(owner) = core.owner() else { return };
            core.handle
                .spawn(async move { answer(owner, serial, from, episode_key).await });
        }
        Secret::Offer(offer) => take_offer(core, serial, from, offer),
        Secret::Nothing { episode_key } => {
            let fits = |a: &Active| {
                a.fetch == RoomFetch::Asking
                    && a.episode
                        .as_ref()
                        .is_some_and(|e| e.episode_key == episode_key)
                    && a.host_key().is_some_and(|(h, _)| h == from)
            };
            let id = core
                .rooms
                .lock()
                .as_ref()
                .and_then(|a| a.fetching.as_ref().map(|(id, _)| *id));
            if let Some(id) = id {
                stop(core, serial, id, fits, Why::Nothing);
            }
        }
    }
}

/// The host's offer, when it answers this member's ask: the download starts.
fn take_offer(core: &Core, serial: u64, from: MemberId, offer: Offer) {
    let Ok(owner) = core.owner() else { return };
    let started = {
        let mut slot = core.rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.serial == serial) else {
            return;
        };
        let Some(e) = a.episode.clone() else { return };
        let from_host = a.host_key().is_some_and(|(h, _)| h == from);
        if a.fetch != RoomFetch::Asking || !from_host || e.episode_key != offer.episode_key {
            tracing::debug!("Room {}: an offer nobody asked for, dropped", a.code);
            return;
        }
        let Some((id, token)) = a.fetching.clone() else {
            return;
        };
        a.fetch = if offer.magnet.is_some() {
            RoomFetch::Downloading
        } else {
            RoomFetch::Searching
        };
        let say = if offer.magnet.is_some() {
            Say::said(
                Level::Info,
                crate::words!("core-room-media-torrent"),
                format!("Room {}: downloading the host's torrent", a.code),
                a.changed(),
            )
        } else {
            Say::said(
                Level::Info,
                crate::words!("core-room-media-searching"),
                format!(
                    "Room {}: searching the sources for the host's release",
                    a.code
                ),
                a.changed(),
            )
        };
        Some((id, token, e, say))
    };
    let Some((id, token, e, say)) = started else {
        return;
    };
    emit(core, vec![say]);
    core.handle.spawn(async move {
        let run = fetch(&owner, serial, id, &offer, &e);
        let result = tokio::select! {
            _ = token.cancelled() => return,
            r = run => r,
        };
        match result {
            Ok(()) => arrived(&owner, serial, id, e),
            Err(why) => stop(&owner, serial, id, |_| true, why),
        }
    });
}

/// Changes the fetch `id`, if it still runs, and tells the relay and the shell.
fn update(core: &Core, serial: u64, id: u64, f: impl FnOnce(&mut Active) -> bool) {
    let say = {
        let mut slot = core.rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.is_fetch(serial, id)) else {
            return;
        };
        if !f(a) {
            return;
        }
        a.send(a.status());
        Say::debug(format!("Room {}: get media", a.code), a.changed())
    };
    emit(core, vec![say]);
}

/// Stops the fetch `id` short, when `fits` its state, and says why.
fn stop(core: &Core, serial: u64, id: u64, fits: impl FnOnce(&Active) -> bool, why: Why) {
    let say = {
        let mut slot = core.rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.is_fetch(serial, id)) else {
            return;
        };
        if !fits(a) {
            return;
        }
        a.stop_fetch();
        a.fetch = RoomFetch::Failed;
        a.send(a.status());
        Say::said(
            Level::Warn,
            why.words(),
            format!("Room {}: get media stopped: {why:?}", a.code),
            a.changed(),
        )
    };
    emit(core, vec![say]);
}

/// The episode is in the library: the fetch ends, and the room's episode is
/// found again, which says `RoomEpisode` with the file for the shell to open.
fn arrived(core: &Core, serial: u64, id: u64, e: proto::EpisodeRef) {
    let say = {
        let mut slot = core.rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.is_fetch(serial, id)) else {
            return;
        };
        a.stop_fetch();
        Say::said(
            Level::Info,
            crate::words!("core-room-media-here"),
            format!("Room {}: the episode is in the library", a.code),
            a.changed(),
        )
    };
    emit(core, vec![say]);
    super::find(core, serial, e);
}

/// The member's side: the release to download, the download, its progress,
/// and the wait for the library to take it in.
async fn fetch(
    core: &Arc<Core>,
    serial: u64,
    id: u64,
    offer: &Offer,
    e: &proto::EpisodeRef,
) -> Result<(), Why> {
    let local = core
        .store
        .read(|c| episode::find_local(c, e))
        .unwrap_or_default();
    let release = match &offer.magnet {
        Some(magnet) => release(&offer.name, magnet),
        None => {
            let found = search(core, offer, e).await?;
            update(core, serial, id, |a| {
                a.fetch = RoomFetch::Downloading;
                true
            });
            found
        }
    };
    let known = replace::magnet_hash(&release.link);
    let title = release.title.clone();
    let since = crate::time::now_secs();
    let (tell, told) = oneshot::channel();
    let options = DownloadOptions {
        series: local.series,
        provider: release.source.clone(),
        ..Default::default()
    };
    downloads::download_telling(
        core,
        vec![release],
        Some(e.series_title.clone()),
        options,
        Some(tell),
    )
    .map_err(|err| Why::Start(err.to_string()))?;
    update(core, serial, id, |a| {
        a.progress = Some(Progress {
            percent: 0.0,
            eta: None,
        });
        true
    });
    let added = told
        .await
        .map_err(|_| Why::Start(crate::i18n::tr("core-room-media-client", &[])))?;
    // A link the client fetches itself names no hash up front: the torrent is
    // then found by its name among those added since
    let hash = known.or_else(|| added.hashes.first().map(|h| h.to_ascii_lowercase()));
    let content = follow(core, serial, id, hash.as_deref(), &title, since).await?;
    settle_in(core, e, offer, &content).await
}

/// Reads the download's progress until it is done; its content path then.
async fn follow(
    core: &Arc<Core>,
    serial: u64,
    id: u64,
    hash: Option<&str>,
    title: &str,
    since: i64,
) -> Result<String, Why> {
    let mut last_seen = Instant::now();
    loop {
        tokio::time::sleep(POLL).await;
        let listed = match downloads::qbt_client(core).await {
            Ok(qbt) => match hash {
                Some(hash) => qbt
                    .torrents_named(&[hash.to_string()])
                    .await
                    .ok()
                    .and_then(|t| t.into_iter().next()),
                None => qbt.torrents().await.ok().and_then(|all| {
                    all.into_iter()
                        .find(|t| t.added_on >= since - 60 && replace::names_match(&t.name, title))
                }),
            },
            Err(_) => None,
        };
        let Some(t) = listed else {
            if last_seen.elapsed() >= LIST_WAIT {
                return Err(Why::Lost);
            }
            continue;
        };
        last_seen = Instant::now();
        let progress = progress_of(&t);
        update(core, serial, id, |a| {
            let changed = a.progress != progress;
            a.progress = progress;
            changed
        });
        if t.progress >= 1.0 {
            return Ok(t.content_path);
        }
    }
}

/// A torrent's progress as the status carries it: a tenth of a percent at most,
/// and no time left when the client has none (qBittorrent says 8640000).
pub fn progress_of(t: &Torrent) -> Option<Progress> {
    Progress {
        percent: (t.progress * 1000.0).floor() / 10.0,
        eta: u64::try_from(t.eta).ok(),
    }
    .clean()
}

/// Waits for the library to take the download in, and points the show it came
/// in as at the room's match when the library did not have that show.
async fn settle_in(
    core: &Arc<Core>,
    e: &proto::EpisodeRef,
    offer: &Offer,
    content: &str,
) -> Result<(), Why> {
    let started = Instant::now();
    let (mut matched, mut scanned) = (false, false);
    loop {
        let local = core
            .store
            .read(|c| episode::find_local(c, e))
            .unwrap_or_default();
        if local.file.is_some() {
            return Ok(());
        }
        if !matched
            && let Some(series) = series_holding(core, content)
            && let Some(target) = target(offer, e)
        {
            matched = true;
            match crate::metadata::apply::apply_match(core, series, target) {
                Ok(_) => tracing::info!("room: matched series {series} to the room's show"),
                Err(err) => tracing::warn!("room: could not match series {series}: {err}"),
            }
        }
        if !scanned && started.elapsed() >= SCAN_AFTER {
            scanned = true;
            if let Err(err) = crate::library::scan::scan(core, None) {
                tracing::debug!("room: scan after get media not started: {err}");
            }
        }
        if started.elapsed() >= ARRIVE_WAIT {
            return Err(Why::NotInLibrary);
        }
        tokio::time::sleep(POLL).await;
    }
}

/// The library series that holds the download, by its content path: the file
/// itself, or a file in the folder it made.
fn series_holding(core: &Core, content: &str) -> Option<u64> {
    let path = content.replace('\\', "/");
    let folder = format!("{}/", path.trim_end_matches('/'));
    core.store
        .read(move |c| {
            Ok(c.query_row(
                "SELECT series_id FROM files WHERE path = ?1 OR substr(path, 1, length(?2)) = ?2
                 ORDER BY id LIMIT 1",
                rusqlite::params![path, folder],
                |r| r.get::<_, i64>(0),
            )
            .ok())
        })
        .ok()
        .flatten()
        .map(|s| s as u64)
}

/// The match the host's show has, as `ApplyMatch` takes it.
fn target(offer: &Offer, e: &proto::EpisodeRef) -> Option<MatchTarget> {
    let id = |v: Option<i64>| v.and_then(|v| u64::try_from(v).ok());
    if let Some(id) = id(offer.anilist_id.or(e.anilist_id)) {
        return Some(MatchTarget::Anilist { id, season: None });
    }
    if let Some(id) = id(offer.mal_id.or(e.mal_id)) {
        return Some(MatchTarget::Mal { id });
    }
    id(offer.tvmaze_id.or(e.tvmaze_id)).map(|id| MatchTarget::Tvmaze { id })
}

/// A release from a name and a link, the rest unknown.
fn release(title: &str, link: &str) -> Release {
    Release {
        title: title.to_string(),
        link: link.to_string(),
        seeders: None,
        leechers: None,
        downloads: None,
        size: String::new(),
        category: String::new(),
        page: String::new(),
        source: String::new(),
        resolution: String::new(),
        infomkv: String::new(),
        linktype: String::new(),
    }
}

// ---------------------------------------------------------------------------
// Plan B: the release by its name, from the member's own sources
// ---------------------------------------------------------------------------

/// Searches the member's sources for the host's release. When no result is
/// the host's own (a renamed file, a release no source carries), the best fit
/// of the same episode from the closest search stands in: the room's offset
/// covers a cut that differs.
async fn search(core: &Arc<Core>, offer: &Offer, e: &proto::EpisodeRef) -> Result<Release, Why> {
    let backend = DownloadsBackend::open(core)
        .await
        .map_err(|err| Why::Start(err.to_string()))?;
    let mut found_episode = Vec::new();
    for query in queries(offer, e) {
        match backend.search(&query, None, true).await {
            Ok(raw) => {
                if let Some(found) = pick(&raw.items, offer, e) {
                    tracing::info!("room: found the host's release with {query:?}");
                    return Ok(found);
                }
                let fits: Vec<Release> = raw
                    .items
                    .into_iter()
                    .filter(|r| episode_fits(&facts::read(r), e))
                    .collect();
                if !fits.is_empty() {
                    found_episode.push((raw.query, raw.endpoint, fits));
                }
            }
            Err(err) => tracing::debug!("room: search {query:?} failed: {err}"),
        }
    }
    for (query, endpoint, items) in found_episode {
        match backend.analyze(&query, &endpoint, &items).await {
            Ok(a) if a.best_fit.available => {
                if let Some(item) = a.best_fit.item {
                    tracing::info!(
                        "room: no source has the host's release; best fit from {query:?}"
                    );
                    return Ok(item);
                }
            }
            Ok(_) => {}
            Err(err) => tracing::debug!("room: best fit for {query:?} failed: {err}"),
        }
    }
    Err(Why::NotFound)
}

/// A release that holds this episode alone: never a batch, and one with no
/// number only for the first episode, which is how a film is named.
fn episode_fits(f: &ReleaseFacts, e: &proto::EpisodeRef) -> bool {
    if f.batch {
        return false;
    }
    match (e.number, f.episode) {
        (Some(want), Some(got)) => (want - got).abs() < 0.01,
        (Some(want), None) => (want - 1.0).abs() < 0.01,
        (None, _) => true,
    }
}

/// What to search for, the closest first: the group, the show as the release
/// names it and the episode; then without the group; then the room's title.
pub fn queries(offer: &Offer, e: &proto::EpisodeRef) -> Vec<String> {
    let ep = e
        .number
        .filter(|n| *n >= 0.0 && n.fract() == 0.0)
        .map(|n| format!("{:02}", n as u64));
    let mut show = titles::show_name(&offer.name);
    if show.is_empty() {
        show = titles::show_name(&offer.file);
    }
    let join = |parts: &[&str]| {
        parts
            .iter()
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    };
    let ep = ep.as_deref().unwrap_or("");
    let mut out = Vec::new();
    if !show.is_empty() {
        if !offer.group.is_empty() {
            out.push(join(&[&offer.group, &show, ep]));
        }
        out.push(join(&[&show, ep]));
    }
    out.push(join(&[&e.series_title, ep]));
    // A first episode may be a film, whose releases carry no number
    if ep == "01" {
        let bare = untagged(&show);
        if !bare.is_empty() {
            out.push(bare);
        }
        out.push(e.series_title.trim().to_string());
    }
    let mut seen = std::collections::HashSet::new();
    out.retain(|q| !q.is_empty() && seen.insert(q.to_lowercase()));
    out
}

/// A name without its bracketed tags: "Film (BD 1080p) [ABCD1234]" reads "Film".
fn untagged(name: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in name.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The search result that is the host's release, or none: the same torrent by
/// its hash, else the same episode of the same release by its name, or by its
/// group and resolution. A batch never fits one episode.
pub fn pick(items: &[Release], offer: &Offer, e: &proto::EpisodeRef) -> Option<Release> {
    items
        .iter()
        .filter_map(|r| score(r, offer, e).map(|s| (s, r)))
        .max_by_key(|(s, _)| *s)
        .map(|(_, r)| r.clone())
}

fn score(r: &Release, offer: &Offer, e: &proto::EpisodeRef) -> Option<i64> {
    if let (Some(hash), Some(theirs)) = (&offer.hash, replace::magnet_hash(&r.link))
        && hash.eq_ignore_ascii_case(&theirs)
    {
        return Some(100_000);
    }
    let f = facts::read(r);
    if !episode_fits(&f, e) {
        return None;
    }
    let same = facts::same_release(&r.title, &f, &offer.name)
        || facts::same_release(&r.title, &f, &offer.file);
    let group = !offer.group.is_empty() && f.group.eq_ignore_ascii_case(&offer.group);
    let resolution =
        !offer.resolution.is_empty() && f.resolution.eq_ignore_ascii_case(&offer.resolution);
    if !(same || (group && resolution)) {
        return None;
    }
    let seeders = r.seeders.unwrap_or(0).clamp(0, 999);
    Some(
        i64::from(same) * 10_000
            + i64::from(group) * 2_000
            + i64::from(resolution) * 1_000
            + seeders,
    )
}

// ---------------------------------------------------------------------------
// The host answers
// ---------------------------------------------------------------------------

/// The host's app answers a member's ask with what it has of the episode.
async fn answer(core: Arc<Core>, serial: u64, to: MemberId, episode_key: String) {
    let asked = {
        let slot = core.rooms.lock();
        let Some(a) = slot.as_ref().filter(|a| a.serial == serial) else {
            return;
        };
        let Some(e) = a.episode.clone().filter(|e| e.episode_key == episode_key) else {
            return;
        };
        let Some(theirs) = a
            .members
            .iter()
            .find(|m| m.id == to)
            .and_then(|m| Some((m.name.clone(), m.key.clone()?)))
        else {
            return;
        };
        // The attached file, when it is the room's episode
        let file = a
            .file
            .filter(|_| a.mine.as_ref().is_some_and(|m| episode::same(m, &e)));
        (e, file, theirs, a.keys.clone(), a.code.clone())
    };
    let (e, file, (name, theirs), keys, code) = asked;
    let file = match file {
        Some(f) => Some(f),
        None => core
            .store
            .read(|c| episode::find_local(c, &e))
            .ok()
            .and_then(|l| l.file),
    };
    let offer = match file {
        Some(f) => offer_for(&core, f, &e).await,
        None => None,
    };
    let torrent = offer.as_ref().is_some_and(|o| o.magnet.is_some());
    let sealed = match offer {
        Some(offer) => seal_offer(&keys, &code, &theirs, offer),
        None => seal_one(
            &keys,
            &code,
            &theirs,
            &Secret::Nothing {
                episode_key: e.episode_key.clone(),
            },
        ),
    };
    let Some(data) = sealed else {
        tracing::warn!("Room {code}: the answer to {name} could not be sealed");
        return;
    };
    let said = {
        let slot = core.rooms.lock();
        let Some(a) = slot.as_ref().filter(|a| a.serial == serial) else {
            return;
        };
        a.send(ClientMsg::Seal { to, data });
        Say::said(
            Level::Info,
            if torrent {
                crate::words!("core-room-media-sent", name = name.as_str())
            } else {
                crate::words!("core-room-media-sent-name", name = name.as_str())
            },
            format!(
                "Room {code}: answered {name}'s ask for {} {}",
                e.series_title, e.episode_key
            ),
            EventBody::RoomChanged { room: a.info() },
        )
    };
    emit(&core, vec![said]);
}

fn seal_one(keys: &super::seal::Keys, code: &str, theirs: &str, s: &Secret) -> Option<String> {
    let plain = serde_json::to_vec(s).ok()?;
    keys.seal(code, theirs, &plain)
        .filter(|d| d.len() <= proto::MAX_SEALED)
}

/// Seals an offer under the relay's cap: a magnet too long sheds its trackers
/// from the end, and one still too long goes, leaving the release's name.
fn seal_offer(
    keys: &super::seal::Keys,
    code: &str,
    theirs: &str,
    mut offer: Offer,
) -> Option<String> {
    loop {
        if let Some(sealed) = seal_one(keys, code, theirs, &Secret::Offer(offer.clone())) {
            return Some(sealed);
        }
        if let Some(m) = offer.magnet.take() {
            if let Some(cut) = m.rfind("&tr=") {
                offer.magnet = Some(m[..cut].to_string());
            }
            continue;
        }
        // The names alone are too long: cut them
        if offer.name.len() <= 300 && offer.file.len() <= 300 {
            return None;
        }
        offer.name.truncate(offer.name.floor_char_boundary(300));
        offer.file.truncate(offer.file.floor_char_boundary(300));
    }
}

/// What the host can offer for its file `file`: the torrent while the client
/// holds it, and the release's name always.
async fn offer_for(core: &Arc<Core>, file: u64, e: &proto::EpisodeRef) -> Option<Offer> {
    let path: String = core
        .store
        .read(move |c| {
            Ok(
                c.query_row("SELECT path FROM files WHERE id = ?1", [file as i64], |r| {
                    r.get(0)
                })?,
            )
        })
        .ok()?;
    let read = |core: &Arc<Core>, path: &str| {
        let p = path.to_string();
        core.store
            .read(move |c| origins::of(c, &[p.as_str()]))
            .ok()
            .and_then(|mut m| m.remove(path))
    };
    let mut origin = read(core, &path);
    // A file the client brought in that nobody has read the client for yet
    if origin.is_none()
        && let Some(qbt) = downloads::running_qbt(core)
        && let Ok(torrents) = qbt.torrents().await
    {
        origins::logged(
            origins::record_from(core, Some(&qbt), &torrents).await,
            "for a room",
        );
        origin = read(core, &path);
    }
    let magnet = match &origin {
        Some(o) => match downloads::qbt_client(core).await {
            Ok(qbt) => qbt.magnet(&o.hash).await.ok().flatten(),
            Err(err) => {
                tracing::debug!("room: the torrent client was not reached: {err}");
                None
            }
        },
        None => None,
    };
    let file_name = path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(path.as_str())
        .to_string();
    let name = origin
        .as_ref()
        .map(|o| o.name.clone())
        .unwrap_or_else(|| file_name.clone());
    let (of_name, of_file) = (facts::read_name(&name), facts::read_name(&file_name));
    let or = |a: String, b: String| if a.is_empty() { b } else { a };
    Some(Offer {
        episode_key: e.episode_key.clone(),
        magnet,
        hash: origin.map(|o| o.hash),
        name,
        file: file_name,
        group: or(of_name.group, of_file.group),
        resolution: or(of_name.resolution, of_file.resolution),
        anilist_id: e.anilist_id,
        mal_id: e.mal_id,
        tvmaze_id: e.tvmaze_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(number: f64) -> proto::EpisodeRef {
        proto::EpisodeRef {
            anilist_id: Some(1),
            mal_id: None,
            tvmaze_id: None,
            episode_key: format!("{number}"),
            number: Some(number),
            series_title: "Cowboy Bebop".into(),
            episode_title: None,
            duration: None,
            opening_start: None,
        }
    }

    fn offer(name: &str) -> Offer {
        let f = facts::read_name(name);
        Offer {
            episode_key: "5".into(),
            name: name.into(),
            file: format!("{name}.mkv"),
            group: f.group,
            resolution: f.resolution,
            anilist_id: Some(1),
            ..Default::default()
        }
    }

    fn magnet(n: u32) -> String {
        format!("magnet:?xt=urn:btih:{n:040x}&dn=x")
    }

    #[test]
    fn an_offer_round_trips_sealed_shape() {
        let o = Secret::Offer(Offer {
            magnet: Some(magnet(1)),
            hash: Some(format!("{:040x}", 1)),
            ..offer("[Lantern] Cowboy Bebop - 05 (1080p)")
        });
        let json = serde_json::to_string(&o).unwrap();
        assert!(json.contains(r#""k":"offer""#));
        assert_eq!(serde_json::from_str::<Secret>(&json).unwrap(), o);
        let ask = serde_json::to_string(&Secret::Ask {
            episode_key: "5".into(),
        })
        .unwrap();
        assert_eq!(ask, r#"{"k":"ask","episode_key":"5"}"#);
    }

    #[test]
    fn queries_go_from_the_release_to_the_show() {
        let q = queries(&offer("[Lantern] Cowboy Bebop - 05 (1080p)"), &ep(5.0));
        assert_eq!(q, vec!["Lantern Cowboy Bebop 05", "Cowboy Bebop 05"]);
        let q = queries(&offer("[Lantern] Kaubōi Bibappu - 05 (720p)"), &ep(5.0));
        assert_eq!(
            q,
            vec![
                "Lantern Kaubōi Bibappu 05",
                "Kaubōi Bibappu 05",
                "Cowboy Bebop 05"
            ]
        );
    }

    #[test]
    fn the_same_release_is_picked_over_others() {
        let host = offer("[Lantern] Cowboy Bebop - 05 (1080p) [ABCD1234]");
        let items = vec![
            Release {
                seeders: Some(500),
                ..release("[Other] Cowboy Bebop - 05 (1080p)", &magnet(1))
            },
            release("[Lantern] Cowboy Bebop - 05 (720p)", &magnet(2)),
            release("[Lantern] Cowboy Bebop - 06 (1080p)", &magnet(3)),
            release("[Lantern] Cowboy Bebop (01-26) (1080p) [Batch]", &magnet(4)),
            Release {
                seeders: Some(12),
                ..release("[Lantern] Cowboy Bebop - 05 (1080p)", &magnet(5))
            },
        ];
        assert_eq!(pick(&items, &host, &ep(5.0)).unwrap().link, magnet(5));
    }

    #[test]
    fn the_same_hash_wins_whatever_its_name() {
        let host = Offer {
            hash: Some(format!("{:040x}", 7)),
            ..offer("[Lantern] Cowboy Bebop - 05 (1080p)")
        };
        let items = vec![
            release("[Lantern] Cowboy Bebop - 05 (1080p)", &magnet(5)),
            release("Cowboy.Bebop.S01E05.renamed", &magnet(7)),
        ];
        assert_eq!(pick(&items, &host, &ep(5.0)).unwrap().link, magnet(7));
    }

    #[test]
    fn a_film_s_release_fits_its_one_episode() {
        let host = offer("[MTBB] The Disappearance of Haruhi Suzumiya (v2) (BD 1080p) [8D032564]");
        let items = vec![release(
            "[MTBB] The Disappearance of Haruhi Suzumiya (v2) (BD 1080p) [8D032564]",
            &magnet(1),
        )];
        assert_eq!(pick(&items, &host, &ep(1.0)).unwrap().link, magnet(1));
        assert_eq!(pick(&items, &host, &ep(2.0)), None);
    }

    #[test]
    fn a_first_episode_also_searches_without_its_number() {
        let q = queries(
            &offer("[MTBB] The Disappearance of Haruhi Suzumiya (BD 1080p)"),
            &ep(1.0),
        );
        assert!(
            q.iter()
                .any(|q| q == "The Disappearance of Haruhi Suzumiya"),
            "{q:?}"
        );
        assert!(q.iter().any(|q| q == "Cowboy Bebop"), "{q:?}");
        let q = queries(&offer("[Lantern] Cowboy Bebop - 05 (1080p)"), &ep(5.0));
        assert!(!q.iter().any(|q| q == "Cowboy Bebop"), "{q:?}");
    }

    #[test]
    fn nothing_fits_when_no_release_is_the_host_s() {
        let host = offer("[Lantern] Cowboy Bebop - 05 (1080p)");
        let items = vec![
            release("[Other] Cowboy Bebop - 05 (1080p)", &magnet(1)),
            release("[Lantern] Cowboy Bebop - 05 (480p)", &magnet(2)),
        ];
        assert_eq!(pick(&items, &host, &ep(5.0)), None);
    }

    #[test]
    fn progress_reads_the_client_s_numbers() {
        let mut t = crate::downloads::qbt::tests_torrent();
        t.progress = 0.4237;
        t.eta = 95;
        assert_eq!(
            progress_of(&t),
            Some(Progress {
                percent: 42.3,
                eta: Some(95)
            })
        );
        t.eta = 8_640_000;
        assert_eq!(progress_of(&t).unwrap().eta, None);
        t.eta = -1;
        assert_eq!(progress_of(&t).unwrap().eta, None);
    }

    #[test]
    fn a_long_magnet_sheds_trackers_to_fit_the_cap() {
        let keys = super::super::seal::Keys::new();
        let theirs = super::super::seal::Keys::new().public();
        let trackers: String = (0..400)
            .map(|i| format!("&tr=udp%3A%2F%2Ftracker{i}.example%3A1337"))
            .collect();
        let long = Offer {
            magnet: Some(format!("{}{trackers}", magnet(1))),
            ..offer("[Lantern] Cowboy Bebop - 05 (1080p)")
        };
        let sealed = seal_offer(&keys, "K7QM2X", &theirs, long).unwrap();
        assert!(sealed.len() <= proto::MAX_SEALED);
    }
}
