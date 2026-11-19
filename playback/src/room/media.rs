//! Get media (#889): a member whose library lacks the room's episode asks the
//! host's app for it, and downloads what comes back.
//!
//! Everything that leads to the video travels sealed (`seal`): a relay that only
//! passes the payloads on reads none of them (an active one could swap the keys
//! it lists, #982). The member asks (`Secret::Ask`); the
//! host's app looks up where its own file came from and answers with an
//! `Offer`. When the torrent client still holds the torrent the file came from
//! (`downloads::origins`) and it is not a private tracker's, the offer carries
//! its magnet with the public trackers alone, and the member hands that to its
//! own torrent client: the host is a peer. When it does not, the offer carries the release's name, group and resolution, and the
//! member searches its own sources for the same release and downloads the one
//! that fits best (`pick`). Either way the download goes through the downloads
//! engine (`downloads::download_telling`), saved into the member's copy of the
//! show when the library has it, else into a folder named for the show.
//!
//! While it downloads, the member's status carries its progress and time left,
//! which is all the relay and the others see. Once done, the playback waits for the
//! library to take the file in, points the show at the room's match unless the
//! user matched that show themselves, and says `RoomEpisode` with the
//! file, so the shell opens it and the member is ready.
//!
//! A member can ask for the whole show with the episode (#922). The episode
//! goes first as above; once its torrent is in the client, the member's sources
//! are searched for the rest of the host's release, the same group and
//! resolution, and every episode of the room's season this library lacks goes
//! to the downloads engine in one pick, which rolls a long show the way any
//! pick does. That search runs once an episode, and stops with the room or when
//! the host moves on; when it finds nothing, the member is told. A film has no
//! rest to fetch, and an episode that came as a batch keeps the batch's other
//! files, with the episode's own file first.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anibeam_room::proto::{self, ClientMsg, MemberId, Progress};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, oneshot};
use tokio_util::sync::CancellationToken;

use super::{Active, Rooms, Say, emit, episode, error, not_in_room};
use anibeam_contract::*;
use anibeam_downloads::{self as downloads, backend::DownloadsBackend, origins, replace};
use anibeam_foundation::events::Listening;
use anibeam_library::scan::ScanScope;
use anibeam_releases::{facts, titles};

/// How long the host's app has to answer before Get media stops waiting.
const ASK_WAIT: Duration = Duration::from_secs(30);

/// How often the member reads its download's progress from the torrent client.
const POLL: Duration = Duration::from_secs(2);

/// How long the torrent client may go without listing a download that was just
/// handed to it, before it counts as lost.
const LIST_WAIT: Duration = Duration::from_secs(120);

/// How long a download may go without moving before Get media gives up on it:
/// nobody seeds it, or the client holds it back. The torrent stays with the
/// client, and Get media can be pressed again.
const STALL: Duration = Duration::from_secs(10 * 60);

/// How long a finished download has to show in the library.
const ARRIVE_WAIT: Duration = Duration::from_secs(300);

/// When a finished download is still not in the library, its own path is
/// scanned once, this long after it finished: the watcher usually brings it
/// sooner.
const SCAN_AFTER: Duration = Duration::from_secs(10);

/// qBittorrent's highest file priority: the room's episode in a batch kept
/// whole, so the member's ready state waits on that file and not the batch.
const FIRST: i64 = 7;

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
    /// The torrent while the host's client still holds it: its hash, its name
    /// and its public trackers (`foundation::magnet::public_only`), never a
    /// private torrent's.
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
    /// The host's app never answered, or the relay could not pass the ask on.
    NoAnswer,
    NotFound,
    NoSources,
    Start(String),
    Lost,
    Stalled,
    NotInLibrary,
}

impl Why {
    fn words(&self) -> Words {
        match self {
            Why::Nothing => anibeam_contract::words!("core-room-media-nothing"),
            Why::NoAnswer => anibeam_contract::words!("core-room-media-no-answer"),
            Why::NotFound => anibeam_contract::words!("core-room-media-not-found"),
            Why::NoSources => anibeam_contract::words!("core-room-media-no-sources"),
            Why::Start(message) => {
                anibeam_contract::words!("core-room-media-no-start", message = message.as_str())
            }
            Why::Lost => anibeam_contract::words!("core-room-media-lost"),
            Why::Stalled => anibeam_contract::words!("core-room-media-stalled"),
            Why::NotInLibrary => anibeam_contract::words!("core-room-media-not-in-library"),
        }
    }
}

// ---------------------------------------------------------------------------
// The member asks
// ---------------------------------------------------------------------------

/// `RoomGetMedia`: seals an ask to the host's app and waits for its answer.
pub(super) fn get(rooms: &Rooms, whole_show: bool) -> Result<Option<RoomInfo>, CoreError> {
    let mut out = Vec::new();
    let info = {
        let mut slot = rooms.lock();
        let a = slot.as_mut().ok_or_else(not_in_room)?;
        if !a.can_fetch() {
            return Err(cannot_fetch());
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
            .ok_or_else(cannot_fetch)?;
        a.send(ClientMsg::Seal { to: host, data });
        let id = begin(a, RoomFetch::Asking);
        a.whole_show = whole_show && !e.film;
        // An app from before Get media, or one gone quiet, never answers
        let (owner, serial) = (a.rooms.clone(), a.serial);
        rooms.services.handle.spawn(async move {
            tokio::time::sleep(ASK_WAIT).await;
            if let Some(rooms) = owner.upgrade() {
                let late = |a: &Active| a.fetch == RoomFetch::Asking;
                stop(&rooms, serial, id, late, Why::NoAnswer);
            }
        });
        out.push(Say::said(
            Level::Info,
            anibeam_contract::words!("core-room-media-asked"),
            format!(
                "Room {}: asked the host for {} {}",
                a.code, e.series_title, e.episode_key
            ),
            a.changed(),
        ));
        a.info()
    };
    emit(&rooms.services, out);
    Ok(info)
}

fn cannot_fetch() -> CoreError {
    error(
        "cannot_fetch",
        anibeam_contract::words!("core-room-media-cannot"),
    )
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

/// The room team, held for a task that outlives the call, while the room
/// `serial` is still the one here.
fn owned(rooms: &Rooms, serial: u64) -> Option<Arc<Rooms>> {
    rooms
        .lock()
        .as_ref()
        .filter(|a| a.serial == serial)
        .and_then(|a| a.rooms.upgrade())
}

/// A sealed payload opened: what it asks of this member.
pub(super) fn on_secret(rooms: &Rooms, serial: u64, from: MemberId, secret: Secret) {
    match secret {
        Secret::Ask { episode_key } => {
            // Only the host's app answers: a member asks the host, never another
            // member, whose torrents are its own
            let hosting = rooms
                .lock()
                .as_ref()
                .is_some_and(|a| a.serial == serial && a.is_host());
            if !hosting {
                tracing::debug!("room: an ask to a member who is not the host, dropped");
                return;
            }
            let Some(owner) = owned(rooms, serial) else {
                return;
            };
            rooms
                .services
                .handle
                .spawn(async move { answer(owner, serial, from, episode_key).await });
        }
        Secret::Offer(offer) => take_offer(rooms, serial, from, offer),
        Secret::Nothing { episode_key } => {
            let fits = |a: &Active| {
                a.fetch == RoomFetch::Asking
                    && a.episode
                        .as_ref()
                        .is_some_and(|e| e.episode_key == episode_key)
                    && a.host_key().is_some_and(|(h, _)| h == from)
            };
            stop_current(rooms, serial, fits, Why::Nothing);
        }
    }
}

/// The relay refused to pass a sealed payload on: nobody by that id is
/// connected, the host having just dropped. An ask waiting on it stops now
/// rather than when its timer runs out.
pub(super) fn seal_refused(rooms: &Rooms, serial: u64) {
    let asking = |a: &Active| a.fetch == RoomFetch::Asking;
    stop_current(rooms, serial, asking, Why::NoAnswer);
}

/// The host's offer, when it answers this member's ask: the download starts.
fn take_offer(rooms: &Rooms, serial: u64, from: MemberId, mut offer: Offer) {
    let Some(owner) = owned(rooms, serial) else {
        return;
    };
    // Only a magnet with a hash reaches the torrent client, rebuilt from its hash,
    // name and public trackers: any other link would have this member's machine
    // fetch an address of the sender's choosing. Without one the release's name
    // is still searched for on this member's own sources.
    if let Some(link) = offer.magnet.take() {
        offer.magnet = anibeam_foundation::magnet::public_only(&link);
        if offer.magnet.is_none() {
            tracing::warn!("room: an offer's link is not a magnet; its name is searched for");
        }
    }
    offer.hash = offer
        .hash
        .take()
        .filter(|h| anibeam_foundation::magnet::is_hash(h));
    let started = {
        let mut slot = rooms.lock();
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
        let whole_show = a.whole_show;
        let (fetch, words, what) = match offer.magnet {
            Some(_) => (
                RoomFetch::Downloading,
                anibeam_contract::words!("core-room-media-torrent"),
                "downloading the host's torrent",
            ),
            None => (
                RoomFetch::Searching,
                anibeam_contract::words!("core-room-media-searching"),
                "searching the sources for the host's release",
            ),
        };
        a.fetch = fetch;
        let say = Say::said(
            Level::Info,
            words,
            format!("Room {}: {what}", a.code),
            a.changed(),
        );
        Some((id, token, e, say, whole_show))
    };
    let Some((id, token, e, say, whole_show)) = started else {
        return;
    };
    emit(&rooms.services, vec![say]);
    rooms.services.handle.spawn(async move {
        let run = fetch(&owner, serial, id, &offer, &e, whole_show);
        let result = tokio::select! {
            _ = token.cancelled() => return,
            r = run => r,
        };
        match result {
            Ok(here) => arrived(&owner, serial, id, e, here),
            Err(why) => stop(&owner, serial, id, |_| true, why),
        }
    });
}

/// Changes the fetch `id`, if it still runs, and tells the relay and the shell.
fn update(rooms: &Rooms, serial: u64, id: u64, f: impl FnOnce(&mut Active) -> bool) {
    let say = {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.is_fetch(serial, id)) else {
            return;
        };
        if !f(a) {
            return;
        }
        a.send(a.status());
        Say::debug(format!("Room {}: get media", a.code), a.changed())
    };
    emit(&rooms.services, vec![say]);
}

/// Stops the fetch `id` short, when `fits` its state, and says why.
fn stop(rooms: &Rooms, serial: u64, id: u64, fits: impl FnOnce(&Active) -> bool, why: Why) {
    stop_where(rooms, |a| a.is_fetch(serial, id) && fits(a), why);
}

/// Stops the fetch running in room `serial` now, when `fits` its state.
fn stop_current(rooms: &Rooms, serial: u64, fits: impl FnOnce(&Active) -> bool, why: Why) {
    stop_where(
        rooms,
        |a| a.serial == serial && a.fetching.is_some() && fits(a),
        why,
    );
}

/// Stops the room's fetch short, under one lock, when `stops` says so, and says why.
fn stop_where(rooms: &Rooms, stops: impl FnOnce(&Active) -> bool, why: Why) {
    let say = {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| stops(a)) else {
            return;
        };
        a.stop_fetch();
        a.fetch = RoomFetch::Failed;
        a.fetch_error = Some(why.words());
        a.send(a.status());
        Say::said(
            Level::Warn,
            why.words(),
            format!("Room {}: get media stopped: {why:?}", a.code),
            a.changed(),
        )
    };
    emit(&rooms.services, vec![say]);
}

/// The episode is in the library: the fetch ends, and the room's episode is
/// found again, which says `RoomEpisode` with the file for the shell to open.
/// `here` is the file where it landed, in a show the user matched themselves,
/// which the room's ids do not find.
fn arrived(
    rooms: &Rooms,
    serial: u64,
    id: u64,
    e: proto::EpisodeRef,
    here: Option<episode::Local>,
) {
    let say = {
        let mut slot = rooms.lock();
        let Some(a) = slot.as_mut().filter(|a| a.is_fetch(serial, id)) else {
            return;
        };
        a.stop_fetch();
        Say::said(
            Level::Info,
            anibeam_contract::words!("core-room-media-here"),
            format!("Room {}: the episode is in the library", a.code),
            a.changed(),
        )
    };
    emit(&rooms.services, vec![say]);
    match here {
        Some(local) => super::show(rooms, serial, e, local),
        None => super::find(rooms, serial, e),
    }
}

/// The member's side: the release to download, the download, its progress,
/// and the wait for the library to take it in.
async fn fetch(
    rooms: &Arc<Rooms>,
    serial: u64,
    id: u64,
    offer: &Offer,
    e: &proto::EpisodeRef,
    whole_show: bool,
) -> Result<Option<episode::Local>, Why> {
    let local = rooms
        .services
        .store
        .read(|c| episode::find_local(c, e))
        .unwrap_or_default();
    let release = match &offer.magnet {
        Some(magnet) => release(&offer.name, magnet),
        None => {
            let found = search(rooms, offer, e).await?;
            update(rooms, serial, id, |a| {
                a.fetch = RoomFetch::Downloading;
                true
            });
            found
        }
    };
    let known = anibeam_foundation::magnet::hash(&release.link);
    let title = release.title.clone();
    // A batch brings the rest of the show with it
    let rest = whole_show && !facts::read(&release).batch;
    let since = anibeam_foundation::time::now_secs();
    update(rooms, serial, id, |a| {
        a.progress = Some(Progress {
            percent: 0.0,
            eta: None,
        });
        true
    });
    // A Get media tried again finds the torrent the last one handed over:
    // qBittorrent refuses to add it twice, so it is followed where it is
    let held = match &known {
        Some(hash) => match downloads::qbt_client(&rooms.downloads).await {
            Ok(qbt) => qbt
                .torrents_named(std::slice::from_ref(hash))
                .await
                .is_ok_and(|t| !t.is_empty()),
            Err(_) => false,
        },
        None => false,
    };
    let hash = if held {
        known
    } else {
        let (tell, told) = oneshot::channel();
        // The episode alone, now: never held to a rolling window
        let options = DownloadOptions {
            series: local.series,
            provider: release.source.clone(),
            rolling: Some(false),
            ..Default::default()
        };
        downloads::download_telling(
            &rooms.downloads,
            vec![release],
            folder(&e.series_title),
            options,
            Some(tell),
        )
        .map_err(|err| Why::Start(err.to_string()))?;
        let added = told
            .await
            .map_err(|_| Why::Start(anibeam_contract::i18n::tr("core-room-media-client", &[])))?;
        // A link the client fetches itself names no hash up front: the torrent is
        // then found by its name among those added since
        known.or_else(|| added.hashes.first().map(|h| h.to_ascii_lowercase()))
    };
    if rest && let Some(token) = start_rest(rooms, serial, id) {
        let (owner, offer, e) = (rooms.clone(), offer.clone(), e.clone());
        rooms.services.handle.spawn(async move {
            tokio::select! {
                _ = token.cancelled() => {}
                _ = rest_of_show(&owner, serial, &offer, &e, local.series) => {}
            }
        });
    }
    let content = follow(
        rooms,
        serial,
        id,
        hash.as_deref(),
        &title,
        since,
        offer,
        e,
        whole_show,
    )
    .await?;
    settle_in(rooms, e, offer, &content).await
}

/// Reads the download's progress until it is done; the path of the episode's
/// file then. A torrent of several files (the host's batch) downloads the
/// episode's file alone, once the client knows the files: the rest are set to
/// skip, and that file's progress is the one followed. With the whole show the
/// rest stay, and the episode's file goes first (`FIRST`).
#[allow(clippy::too_many_arguments)]
async fn follow(
    rooms: &Arc<Rooms>,
    serial: u64,
    id: u64,
    hash: Option<&str>,
    title: &str,
    since: i64,
    offer: &Offer,
    e: &proto::EpisodeRef,
    whole: bool,
) -> Result<String, Why> {
    let mut last_seen = Instant::now();
    let mut stall = Stall::new(Instant::now());
    // None until the client lists the torrent's files; then the one file
    // followed, or Whole for a torrent of one file or none that is the episode
    let mut unit: Option<Unit> = None;
    loop {
        tokio::time::sleep(POLL).await;
        let qbt = downloads::qbt_client(&rooms.downloads).await.ok();
        let listed = match (&qbt, hash) {
            (Some(qbt), Some(hash)) => qbt
                .torrents_named(&[hash.to_string()])
                .await
                .ok()
                .and_then(|t| t.into_iter().next()),
            (Some(qbt), None) => qbt.torrents().await.ok().and_then(|all| {
                all.into_iter()
                    .find(|t| t.added_on >= since - 60 && replace::names_match(&t.name, title))
            }),
            (None, _) => None,
        };
        let (Some(qbt), Some(t)) = (qbt, listed) else {
            if last_seen.elapsed() >= LIST_WAIT {
                return Err(Why::Lost);
            }
            continue;
        };
        last_seen = Instant::now();
        let files = match unit {
            Some(Unit::Whole) => None,
            _ => qbt.files(&t.hash).await.ok().filter(|f| !f.is_empty()),
        };
        if unit.is_none()
            && let Some(files) = &files
        {
            let one = episode_file(files, offer, e);
            if let Some(i) = one {
                // Whole show keeps the batch's other episodes, behind the room's
                if whole {
                    if let Err(err) = qbt.set_file_priority(&t.hash, &[i], FIRST).await {
                        tracing::warn!("room: the episode's file was not put first: {err}");
                    }
                } else {
                    let others: Vec<usize> = (0..files.len()).filter(|j| *j != i).collect();
                    if let Err(err) = qbt.set_file_priority(&t.hash, &others, 0).await {
                        tracing::warn!(
                            "room: the other files of the torrent were not skipped: {err}"
                        );
                    }
                }
            }
            unit = Some(one.map_or(Unit::Whole, Unit::File));
        }
        let followed = match (unit, &files) {
            (Some(Unit::File(i)), Some(files)) => files.get(i).map(|f| {
                let path = anibeam_downloads::rolling::file_path(&t.save_path, &f.name)
                    .map(|p| p.to_string_lossy().into_owned());
                (f.progress, path)
            }),
            // The files could not be read this time round
            (Some(Unit::File(_)), None) => continue,
            _ => None,
        };
        let (fraction, path) = followed.unwrap_or((t.progress, Some(t.content_path.clone())));
        let progress = progress_at(fraction, t.eta);
        update(rooms, serial, id, |a| {
            let changed = a.progress != progress;
            a.progress = progress;
            changed
        });
        if fraction >= 1.0 {
            return path.ok_or(Why::Lost);
        }
        if stall.stuck(fraction, Instant::now()) {
            return Err(Why::Stalled);
        }
    }
}

/// What of a torrent Get media follows.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Unit {
    Whole,
    File(usize),
}

/// The file of a torrent of several that is the room's episode: the host's
/// own file by its name, else the one file whose name carries the episode's
/// number. None for a torrent of one file, or when no single file fits.
fn episode_file(files: &[TorrentFile], offer: &Offer, e: &proto::EpisodeRef) -> Option<usize> {
    if files.len() < 2 {
        return None;
    }
    let base = |name: &str| name.rsplit(['/', '\\']).next().unwrap_or(name).to_string();
    if !offer.file.is_empty()
        && let Some(i) = files
            .iter()
            .position(|f| base(&f.name).eq_ignore_ascii_case(&offer.file))
    {
        return Some(i);
    }
    let want = e.number?;
    let fits: Vec<usize> = files
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            let facts = facts::read_name(&f.name);
            !facts.batch && facts.episode.is_some_and(|n| (n - want).abs() < 0.01)
        })
        .map(|(i, _)| i)
        .collect();
    match fits.as_slice() {
        [one] => Some(*one),
        _ => None,
    }
}

/// Whether a download has stopped moving: the furthest it got, and when.
struct Stall {
    best: f64,
    since: Instant,
}

impl Stall {
    fn new(now: Instant) -> Stall {
        Stall {
            best: 0.0,
            since: now,
        }
    }

    /// Notes how far the download is at `now`: true once it has gone `STALL`
    /// without getting any further.
    fn stuck(&mut self, fraction: f64, now: Instant) -> bool {
        if fraction > self.best {
            self.best = fraction;
            self.since = now;
        }
        now.duration_since(self.since) >= STALL
    }
}

/// A torrent's progress as the status carries it: a tenth of a percent at most,
/// and no time left when the client has none (qBittorrent says 8640000).
pub fn progress_of(t: &Torrent) -> Option<Progress> {
    progress_at(t.progress, t.eta)
}

/// `progress_of` for a fraction done and the client's seconds left.
fn progress_at(fraction: f64, eta: i64) -> Option<Progress> {
    Progress {
        percent: (fraction * 1000.0).floor() / 10.0,
        eta: u64::try_from(eta).ok(),
    }
    .clean()
}

/// Waits for the library to take the download in, and points the show it came
/// in as at the room's match. A show the user matched themselves keeps its
/// match, and the file is then the one where the download landed, which comes
/// back since the room's ids do not find it. Nothing is polled: the library is
/// read again when a scan, a series or a match says it changed, and when the
/// watcher has not brought the file in by `SCAN_AFTER`, the download's own
/// path is scanned, once.
async fn settle_in(
    rooms: &Arc<Rooms>,
    e: &proto::EpisodeRef,
    offer: &Offer,
    content: &str,
) -> Result<Option<episode::Local>, Why> {
    let woken = Arc::new(Notify::new());
    let failed = Failed::default();
    let _listening = Listening::new(
        rooms.services.bus.clone(),
        Arc::new(Arrivals {
            woken: woken.clone(),
            failed: failed.clone(),
        }),
    );
    let started = tokio::time::Instant::now();
    let (scan_at, deadline) = (started + SCAN_AFTER, started + ARRIVE_WAIT);
    let (mut matched, mut scanned) = (ShowMatch::default(), false);
    loop {
        let local = rooms
            .services
            .store
            .read(|c| episode::find_local(c, e))
            .unwrap_or_default();
        if local.file.is_some() {
            return Ok(None);
        }
        if let Some(here) = matched.try_apply(rooms, content, offer, e, &failed) {
            return Ok(Some(here));
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(Why::NotInLibrary);
        }
        let until = if scanned { deadline } else { scan_at };
        tokio::select! {
            _ = woken.notified() => {}
            _ = tokio::time::sleep_until(until) => {
                if !scanned {
                    scanned = true;
                    let scope = ScanScope::Paths(vec![content.to_string()]);
                    anibeam_library::scan::start(&rooms.library, scope);
                }
            }
        }
    }
}

/// How many times `settle_in` starts the room's match before it lets the
/// show be, so a match that keeps failing (the provider down, say) is not
/// started again on every wake.
const MATCH_TRIES: u8 = 3;

/// The match jobs that failed or were cancelled while `settle_in` waits.
type Failed = Arc<Mutex<HashSet<u64>>>;

/// Where `settle_in` is with pointing the download's show at the room's
/// match. A match that could not start (the series' row not readable yet,
/// say) or whose job failed is tried again on the next wake, up to
/// `MATCH_TRIES` times (#1067).
#[derive(Default)]
struct ShowMatch {
    /// The match job started last, while it may still land.
    job: Option<u64>,
    tries: u8,
    /// Each series the download is in, and whether the user matched it
    /// themselves, read when it is first seen: the room's own match writes a
    /// confirmed match too, and a retry must not take that for the user's.
    users: HashMap<u64, bool>,
}

impl ShowMatch {
    /// Starts the room's match on the series that holds the download, unless
    /// the user matched it themselves (#1116). Such a series keeps its match,
    /// and once the library holds the download's episode file, that comes back.
    fn try_apply(
        &mut self,
        rooms: &Rooms,
        content: &str,
        offer: &Offer,
        e: &proto::EpisodeRef,
        failed: &Failed,
    ) -> Option<episode::Local> {
        let here = rooms
            .services
            .store
            .read(|c| episode::at_path(c, content, e))
            .ok()
            .flatten()?;
        let series = here.series?;
        let users = *self
            .users
            .entry(series)
            .or_insert_with(|| user_matched(rooms, series));
        if users {
            return here.file.is_some().then_some(here);
        }
        if let Some(job) = self.job {
            if !anibeam_foundation::sync::recover(failed).contains(&job) {
                return None;
            }
            self.job = None;
        }
        if self.tries >= MATCH_TRIES {
            return None;
        }
        let target = target(offer, e)?;
        self.tries += 1;
        match anibeam_metadata::apply::apply_match(&rooms.metadata, series, target) {
            Ok(job) => {
                self.job = Some(job);
                tracing::info!("room: matching series {series} to the room's show");
            }
            Err(err) => tracing::warn!("room: could not match series {series}: {err}"),
        }
        None
    }
}

/// Wakes `settle_in` when the library or a show's match moved, and notes
/// the match jobs that ended without matching.
struct Arrivals {
    woken: Arc<Notify>,
    failed: Failed,
}

impl EventListener for Arrivals {
    fn on_event(&self, event: Event) {
        let match_ended = matches!(
            event.job,
            Some(JobRef {
                kind: JobKind::ApplyMatch,
                phase: JobPhase::Finished,
                ..
            })
        ) && matches!(
            event.body,
            EventBody::JobFailed { .. } | EventBody::JobCancelled
        );
        if match_ended && let Some(job) = &event.job {
            anibeam_foundation::sync::recover(&self.failed).insert(job.id);
        }
        if match_ended
            || matches!(
                event.body,
                EventBody::ScanFinished { .. }
                    | EventBody::SeriesChanged { .. }
                    | EventBody::MatchApplied { .. }
            )
        {
            self.woken.notify_one();
        }
    }
}

/// Whether the user matched the series themselves, or an import brought the
/// match: the match the auto-match never replaces either. A match the
/// auto-match guessed is the library's own and takes the room's, since the
/// auto-match can reach a new folder while the episode is still coming in.
fn user_matched(rooms: &Rooms, series: u64) -> bool {
    rooms
        .services
        .store
        .read(move |c| {
            Ok(c.query_row(
                "SELECT confirmed FROM series WHERE id = ?1",
                [series as i64],
                |r| r.get::<_, i64>(0),
            )
            .optional()?)
        })
        .ok()
        .flatten()
        .is_some_and(|confirmed| confirmed == 1)
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
async fn search(rooms: &Arc<Rooms>, offer: &Offer, e: &proto::EpisodeRef) -> Result<Release, Why> {
    let backend = DownloadsBackend::open(&rooms.downloads)
        .await
        .map_err(|err| Why::Start(err.to_string()))?;
    if backend.has_no_sources() {
        return Err(Why::NoSources);
    }
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

/// The folder a download lands in, named for the show: a title with a slash in it
/// ("Ranma1/2") would be refused as a path, so a slash reads as a hyphen. None when
/// nothing is left, which names the folder after the release instead.
fn folder(title: &str) -> Option<String> {
    let name: String = title
        .chars()
        .map(|c| match c {
            '/' | '\\' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let name = name.trim();
    (!name.is_empty() && name != "." && name != "..").then(|| name.to_string())
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
    let show = offer_show(offer);
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
    distinct(out)
}

/// Each query once, ignoring case, and none that is empty.
fn distinct(queries: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    queries
        .into_iter()
        .map(|q| q.trim().to_string())
        .filter(|q| !q.is_empty() && seen.insert(q.to_lowercase()))
        .collect()
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
    if let (Some(hash), Some(theirs)) = (&offer.hash, anibeam_foundation::magnet::hash(&r.link))
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
// The rest of the show (#922)
// ---------------------------------------------------------------------------

/// Marks the search for the rest of the show as started for the fetch `id`:
/// the token that stops it, a child of the room's, or None when the fetch is
/// gone or the rest was already asked for with this episode.
fn start_rest(rooms: &Rooms, serial: u64, id: u64) -> Option<CancellationToken> {
    let mut slot = rooms.lock();
    let a = slot.as_mut().filter(|a| a.is_fetch(serial, id))?;
    if a.rest.is_some() {
        return None;
    }
    let token = a.cancel.child_token();
    a.rest = Some(token.clone());
    Some(token)
}

/// Searches the member's sources for the rest of the host's release and hands
/// every episode this library lacks of the room's season to the downloads
/// engine. When nothing turns up or it cannot start, the member is told: the
/// room's episode still comes.
async fn rest_of_show(
    rooms: &Arc<Rooms>,
    serial: u64,
    offer: &Offer,
    e: &proto::EpisodeRef,
    series: Option<u64>,
) {
    let have = series
        .and_then(|s| {
            let e = e.clone();
            rooms.services.store.read(move |c| held(c, s, &e)).ok()
        })
        .unwrap_or_default();
    let backend = match DownloadsBackend::open(&rooms.downloads).await {
        Ok(b) => b,
        Err(err) => {
            no_rest(rooms, serial, &format!("no sources: {err}"));
            return;
        }
    };
    let mut found = Vec::new();
    for query in show_queries(offer, e) {
        match backend.search(&query, None, true).await {
            Ok(raw) => {
                found = rest(&raw.items, offer, e, &have);
                if !found.is_empty() {
                    tracing::info!("room: the rest of the show with {query:?}");
                    break;
                }
            }
            Err(err) => tracing::debug!("room: search {query:?} failed: {err}"),
        }
    }
    if found.is_empty() {
        no_rest(
            rooms,
            serial,
            "no more of the host's release in the sources",
        );
        return;
    }
    let count = found.len();
    let options = DownloadOptions {
        series,
        provider: found[0].source.clone(),
        ..Default::default()
    };
    match downloads::download_telling(
        &rooms.downloads,
        found,
        folder(&e.series_title),
        options,
        None,
    ) {
        Ok(_) => tracing::info!("room: {count} more episodes of the show handed on"),
        Err(err) => no_rest(rooms, serial, &format!("did not start: {err}")),
    }
}

/// Tells the member the rest of the show is not coming, while the room is
/// still the one it was asked in.
fn no_rest(rooms: &Rooms, serial: u64, why: &str) {
    let say = {
        let slot = rooms.lock();
        let Some(a) = slot.as_ref().filter(|a| a.serial == serial) else {
            return;
        };
        Say::said(
            Level::Warn,
            anibeam_contract::words!("core-room-media-rest-missing"),
            format!("Room {}: the rest of the show: {why}", a.code),
            a.changed(),
        )
    };
    emit(&rooms.services, vec![say]);
}

/// The episode numbers a library series holds of the room's season. A key
/// such as `S02E05` names the season; a bare number names none, and is the
/// first season of a TVmaze show, which keeps every season in one series. An
/// AniList or MAL match is one season already, so a file there that names no
/// season counts whatever the room's key.
fn held(
    c: &rusqlite::Connection,
    series: u64,
    e: &proto::EpisodeRef,
) -> Result<Vec<f64>, CoreError> {
    let tvmaze = c
        .query_row(
            "SELECT provider = 'tvmaze' FROM series WHERE id = ?1",
            [series as i64],
            |r| r.get::<_, bool>(0),
        )
        .optional()?
        .unwrap_or(false);
    let want = episode::season_of(&e.episode_key);
    let mut stmt = c.prepare(
        "SELECT number, season FROM files WHERE series_id = ?1 AND kind = 'episode' AND number IS NOT NULL",
    )?;
    let rows = stmt
        .query_map([series as i64], |r| {
            Ok((r.get::<_, f64>(0)?, r.get::<_, Option<i64>>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let of_room = |season: Option<i64>| match (want, season) {
        (Some(w), Some(s)) => w == s,
        (Some(w), None) => w == 1 || !tvmaze,
        (None, Some(s)) => s == 1 || !tvmaze,
        (None, None) => true,
    };
    Ok(rows
        .into_iter()
        .filter(|(_, season)| of_room(*season))
        .map(|(n, _)| n)
        .collect())
}

/// The show as the host's release names it, else as its file does.
fn offer_show(offer: &Offer) -> String {
    let show = titles::show_name(&offer.name);
    if show.is_empty() {
        titles::show_name(&offer.file)
    } else {
        show
    }
}

/// What to search for the whole show: the group and the show as the release
/// names it, then the show alone, then the room's title. No episode number.
pub fn show_queries(offer: &Offer, e: &proto::EpisodeRef) -> Vec<String> {
    let show = offer_show(offer);
    let mut out = Vec::new();
    if !show.is_empty() {
        if !offer.group.is_empty() {
            out.push(format!("{} {show}", offer.group.trim()));
        }
        out.push(show);
    }
    out.push(e.series_title.trim().to_string());
    distinct(out)
}

/// The other episodes of the host's release among search results: single
/// episodes of the same show, group and resolution (each where the host's
/// release names one), the room's episode and those already `have` left out,
/// one release per episode, the best seeded, in episode order.
pub fn rest(items: &[Release], offer: &Offer, e: &proto::EpisodeRef, have: &[f64]) -> Vec<Release> {
    let show = offer_show(offer);
    let season = facts::read_name(&offer.name).season;
    let near = |a: f64, b: f64| (a - b).abs() < 0.01;
    let mut best: std::collections::BTreeMap<i64, &Release> = std::collections::BTreeMap::new();
    for r in items {
        let f = facts::read(r);
        let Some(n) = f.episode.filter(|_| !f.batch) else {
            continue;
        };
        if e.number.is_some_and(|w| near(w, n)) || have.iter().any(|h| near(*h, n)) {
            continue;
        }
        if !offer.group.is_empty() && !f.group.eq_ignore_ascii_case(&offer.group) {
            continue;
        }
        if !offer.resolution.is_empty() && !f.resolution.eq_ignore_ascii_case(&offer.resolution) {
            continue;
        }
        if !show.is_empty() && !titles::show_name(&r.title).eq_ignore_ascii_case(&show) {
            continue;
        }
        if let (Some(a), Some(b)) = (season, f.season)
            && a != b
        {
            continue;
        }
        let key = (n * 100.0).round() as i64;
        let better = best
            .get(&key)
            .is_none_or(|old| r.seeders.unwrap_or(0) > old.seeders.unwrap_or(0));
        if better {
            best.insert(key, r);
        }
    }
    best.into_values().cloned().collect()
}

// ---------------------------------------------------------------------------
// The host answers
// ---------------------------------------------------------------------------

/// The host's app answers a member's ask with what it has of the episode.
async fn answer(rooms: Arc<Rooms>, serial: u64, to: MemberId, episode_key: String) {
    let asked = {
        let slot = rooms.lock();
        let Some(a) = slot.as_ref().filter(|a| a.serial == serial && a.is_host()) else {
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
        None => rooms
            .services
            .store
            .read(|c| episode::find_local(c, &e))
            .ok()
            .and_then(|l| l.file),
    };
    let offer = match file {
        Some(f) => offer_for(&rooms, f, &e).await,
        None => None,
    };
    // Whether the torrent went is read off what was sealed: a magnet too long for the
    // relay is dropped there, and only the name goes.
    let sealed = match offer {
        Some(offer) => seal_offer(&keys, &code, &theirs, offer),
        None => {
            let nothing = Secret::Nothing {
                episode_key: e.episode_key.clone(),
            };
            seal_one(&keys, &code, &theirs, &nothing).map(|data| (data, false))
        }
    };
    let Some((data, torrent)) = sealed else {
        tracing::warn!("Room {code}: the answer to {name} could not be sealed");
        return;
    };
    let said = {
        let slot = rooms.lock();
        let Some(a) = slot.as_ref().filter(|a| a.serial == serial) else {
            return;
        };
        a.send(ClientMsg::Seal { to, data });
        Say::said(
            Level::Info,
            if torrent {
                anibeam_contract::words!("core-room-media-sent", name = name.as_str())
            } else {
                anibeam_contract::words!("core-room-media-sent-name", name = name.as_str())
            },
            format!(
                "Room {code}: answered {name}'s ask for {} {}",
                e.series_title, e.episode_key
            ),
            EventBody::RoomChanged { room: a.info() },
        )
    };
    emit(&rooms.services, vec![said]);
}

fn seal_one(keys: &super::seal::Keys, code: &str, theirs: &str, s: &Secret) -> Option<String> {
    let plain = serde_json::to_vec(s).ok()?;
    keys.seal(code, theirs, &plain)
        .filter(|d| d.len() <= proto::MAX_SEALED)
}

/// Seals an offer under the relay's cap, and says whether its magnet went with
/// it: a magnet too long sheds its trackers from the end, and one still too long
/// goes, leaving the release's name. How long a payload seals to follows from its
/// own length, so the trackers to shed are counted first and only the offer that
/// fits is sealed.
fn seal_offer(
    keys: &super::seal::Keys,
    code: &str,
    theirs: &str,
    mut offer: Offer,
) -> Option<(String, bool)> {
    loop {
        let plain = serde_json::to_vec(&Secret::Offer(offer.clone())).ok()?;
        let mut need = plain
            .len()
            .saturating_sub(super::seal::plain_max(proto::MAX_SEALED));
        if need == 0 {
            let data = keys.seal(code, theirs, &plain)?;
            return Some((data, offer.magnet.is_some()));
        }
        // Names past 300 bytes go first: a name that long says no more than its start,
        // and the torrent is worth more than its tail
        if offer.name.len() > 300 || offer.file.len() > 300 {
            offer.name.truncate(offer.name.floor_char_boundary(300));
            offer.file.truncate(offer.file.floor_char_boundary(300));
            continue;
        }
        let mut m = offer.magnet.take()?;
        while need > 0
            && let Some(cut) = m.rfind("&tr=")
        {
            // Counted as the payload writes it, escapes and all
            let shed = serde_json::to_string(&m[cut..]).map_or(0, |j| j.len() - 2);
            need = need.saturating_sub(shed);
            m.truncate(cut);
        }
        offer.magnet = (need == 0).then_some(m);
    }
}

/// What the host can offer for its file `file`: the torrent while the client
/// holds it, and the release's name always.
async fn offer_for(rooms: &Arc<Rooms>, file: u64, e: &proto::EpisodeRef) -> Option<Offer> {
    let path: String = rooms
        .services
        .store
        .read(move |c| {
            Ok(
                c.query_row("SELECT path FROM files WHERE id = ?1", [file as i64], |r| {
                    r.get(0)
                })?,
            )
        })
        .ok()?;
    let read = |rooms: &Arc<Rooms>, path: &str| {
        let p = path.to_string();
        rooms
            .services
            .store
            .read(move |c| origins::of(c, &[p.as_str()]))
            .ok()
            .and_then(|mut m| m.remove(path))
    };
    let mut origin = read(rooms, &path);
    // A file the client brought in that nobody has read the client for yet
    if origin.is_none()
        && let Some(qbt) = downloads::running_qbt(&rooms.downloads)
        && let Ok(torrents) = qbt.torrents().await
    {
        origins::logged(
            origins::record_from(&rooms.services, Some(&qbt), &torrents).await,
            "for a room",
        );
        origin = read(rooms, &path);
    }
    let magnet = match &origin {
        Some(o) => match downloads::qbt_client(&rooms.downloads).await {
            // Never a private torrent, and only the trackers that carry nothing
            // of the host's own
            Ok(qbt) => qbt
                .magnet(&o.hash)
                .await
                .ok()
                .flatten()
                .and_then(|m| anibeam_foundation::magnet::public_only(&m)),
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

    use anibeam_foundation::testing::Running;

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
            film: false,
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
    fn a_title_with_a_slash_still_names_a_folder() {
        assert_eq!(
            folder("Ranma1/2 (2024) Season 3").as_deref(),
            Some("Ranma1-2 (2024) Season 3")
        );
        assert_eq!(folder("A\\B").as_deref(), Some("A-B"));
        assert_eq!(folder("  "), None);
        assert_eq!(folder(".."), None);
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
    fn the_rest_of_the_show_is_the_host_s_release_without_what_is_here() {
        let host = offer("[Lantern] Cowboy Bebop - 05 (1080p)");
        let items = vec![
            release("[Lantern] Cowboy Bebop - 01 (1080p)", &magnet(1)),
            Release {
                seeders: Some(40),
                ..release("[Lantern] Cowboy Bebop - 02 (1080p)", &magnet(2))
            },
            Release {
                seeders: Some(3),
                ..release("[Lantern] Cowboy Bebop - 02 (1080p) [v2]", &magnet(3))
            },
            release("[Lantern] Cowboy Bebop - 03 (1080p)", &magnet(4)),
            release("[Lantern] Cowboy Bebop - 05 (1080p)", &magnet(5)),
            release("[Lantern] Cowboy Bebop - 06 (720p)", &magnet(6)),
            release("[Other] Cowboy Bebop - 07 (1080p)", &magnet(7)),
            release("[Lantern] Cowboy Bebop (01-26) (1080p) [Batch]", &magnet(8)),
            release("[Lantern] Trigun - 08 (1080p)", &magnet(9)),
            release("[Lantern] Cowboy Bebop - 09 (1080p)", &magnet(10)),
        ];
        let got: Vec<String> = rest(&items, &host, &ep(5.0), &[3.0])
            .into_iter()
            .map(|r| r.link)
            .collect();
        assert_eq!(got, vec![magnet(1), magnet(2), magnet(10)]);
        assert_eq!(
            show_queries(&host, &ep(5.0)),
            vec!["Lantern Cowboy Bebop", "Cowboy Bebop"]
        );
    }

    /// A library with series 1, matched by `provider`, holding `files` as
    /// (number, season) pairs.
    fn library(provider: &str, files: &[(i64, Option<i64>)]) -> rusqlite::Connection {
        let mut c = rusqlite::Connection::open_in_memory().unwrap();
        c.pragma_update(None, "foreign_keys", "ON").unwrap();
        anibeam_foundation::store::migrations::apply(&mut c).unwrap();
        c.execute(
            "INSERT INTO sources (id, path, available, added_at) VALUES (1, '/lib', 1, 0)",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO series (id, source_id, kind, path, folder_name, added_at, provider)
             VALUES (1, 1, 'show', '/lib/1', 'Show', 0, ?1)",
            [provider],
        )
        .unwrap();
        for (i, (number, season)) in files.iter().enumerate() {
            let key = match season {
                Some(s) => format!("S{s:02}E{number:02}"),
                None => number.to_string(),
            };
            c.execute(
                "INSERT INTO files (id, series_id, path, size, mtime, kind, number, season, label, episode_key, sidecars, seen_at)
                 VALUES (?1, 1, ?2, 0, 0, 'episode', ?3, ?4, 'EP', ?5, '[]', 0)",
                rusqlite::params![i as i64 + 10, format!("/lib/1/{i}.mkv"), number, season, key],
            )
            .unwrap();
        }
        c
    }

    #[test]
    fn the_rest_counts_only_what_is_here_of_the_room_s_season() {
        let key = |k: &str| proto::EpisodeRef {
            episode_key: k.into(),
            ..ep(5.0)
        };
        let have = |c: &rusqlite::Connection, k: &str| {
            let mut got = held(c, 1, &key(k)).unwrap();
            got.sort_by(f64::total_cmp);
            got
        };
        // A TVmaze show keeps its seasons in one series: season one is here,
        // the room is on season two, and none of it counts
        let tv = library(
            "tvmaze",
            &[(1, Some(1)), (2, Some(1)), (3, None), (1, Some(2))],
        );
        assert_eq!(have(&tv, "S02E05"), vec![1.0]);
        assert_eq!(have(&tv, "S01E05"), vec![1.0, 2.0, 3.0]);
        assert_eq!(have(&tv, "5"), vec![1.0, 2.0, 3.0]);
        // An AniList match is one season: a file that names none is of it
        let one = library("anilist", &[(1, None), (2, None), (4, Some(3))]);
        assert_eq!(have(&one, "S02E05"), vec![1.0, 2.0]);
        assert_eq!(have(&one, "5"), vec![1.0, 2.0, 4.0]);
    }

    /// The rooms over the teams below them, each built as `Core::open`
    /// builds it, and the services they run on. None of `Core::open`'s
    /// links: the match reads none of them.
    fn rooms_over(dir: &std::path::Path, http: Arc<anibeam_net::FakeHttp>) -> (Running, Rooms) {
        let running = Running::open(dir, http);
        let services = running.services.clone();
        let net = Arc::new(anibeam_net::Net::new(
            services.clone(),
            anibeam_library::cards::cards_for,
        ));
        let closed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let library = Arc::new(anibeam_library::Library::new(
            services.clone(),
            net.clone(),
            closed,
        ));
        let metadata = Arc::new(anibeam_metadata::Metadata::new(
            services.clone(),
            net.clone(),
            library.clone(),
        ));
        let trackers = Arc::new(anibeam_trackers::Trackers::new(
            services.clone(),
            net.clone(),
        ));
        let downloads = Arc::new(anibeam_downloads::Downloads::new(
            services.clone(),
            net.clone(),
        ));
        let playback = Arc::new(crate::Playback::new(
            services.clone(),
            net,
            trackers,
            downloads.clone(),
        ));
        let rooms = Rooms::new(services, playback, downloads, library, metadata);
        (running, rooms)
    }

    #[test]
    fn a_room_match_whose_job_failed_is_started_again() {
        let dir = tempfile::tempdir().unwrap();
        // TVmaze has no such show, so every match job fails.
        let http = anibeam_net::FakeHttp::new();
        for _ in 0..MATCH_TRIES {
            http.push_for("tvmaze", 404, "");
        }
        let (running, rooms) = rooms_over(dir.path(), http);
        let services = running.services.clone();
        services
            .store
            .tx(|tx| {
                tx.execute_batch(
                    "INSERT INTO sources (id, path, available, added_at) VALUES (1, '/lib', 1, 0);
                     INSERT INTO series (id, source_id, kind, path, folder_name, added_at)
                         VALUES (7, 1, 'show', '/lib/Show', 'Show', 0);
                     INSERT INTO files (series_id, path, size, mtime, kind, number, label, episode_key, seen_at)
                         VALUES (7, '/lib/Show/05.mkv', 1, 0, 'episode', 5, '5', '5', 0);",
                )?;
                Ok(())
            })
            .unwrap();
        let woken = Arc::new(Notify::new());
        let failed = Failed::default();
        let listening = Listening::new(
            services.bus.clone(),
            Arc::new(Arrivals {
                woken: woken.clone(),
                failed: failed.clone(),
            }),
        );
        let offer = Offer {
            anilist_id: None,
            tvmaze_id: Some(9),
            ..offer("[Group] Show - 05 [1080p]")
        };
        let e = proto::EpisodeRef {
            anilist_id: None,
            ..ep(5.0)
        };
        let mut matched = ShowMatch::default();
        let mut started = Vec::new();
        for _ in 0..MATCH_TRIES {
            matched.try_apply(&rooms, "/lib/Show", &offer, &e, &failed);
            let job = matched.job.expect("a match job started");
            assert!(!started.contains(&job), "a new job each try");
            started.push(job);
            let gone = async {
                while !anibeam_foundation::sync::recover(&failed).contains(&job) {
                    woken.notified().await;
                }
            };
            let ended = services
                .handle
                .block_on(async { tokio::time::timeout(Duration::from_secs(30), gone).await });
            assert!(ended.is_ok(), "match job {job} never failed");
        }
        // The tries are spent: a later wake starts nothing.
        matched.try_apply(&rooms, "/lib/Show", &offer, &e, &failed);
        assert_eq!(matched.job, None);
        assert_eq!(started.len(), usize::from(MATCH_TRIES));
        drop(listening);
        drop(rooms);
        running.shutdown();
    }

    /// A library with series 7 at `/lib/Show`, matched to TVmaze show 3 and
    /// `confirmed` as given, holding episode 5. The offer and the episode are
    /// the room's, on TVmaze show 9.
    fn matched_show(
        services: &anibeam_foundation::services::Services,
        confirmed: bool,
    ) -> (Offer, proto::EpisodeRef) {
        services
            .store
            .tx(move |tx| {
                tx.execute_batch(
                    "INSERT INTO sources (id, path, available, added_at) VALUES (1, '/lib', 1, 0);
                     INSERT INTO series (id, source_id, kind, path, folder_name, added_at, provider, tvmaze_id)
                         VALUES (7, 1, 'show', '/lib/Show', 'Show', 0, 'tvmaze', 3);
                     INSERT INTO files (id, series_id, path, size, mtime, kind, number, label, episode_key, seen_at)
                         VALUES (70, 7, '/lib/Show/05.mkv', 1, 0, 'episode', 5, '5', '5', 0);",
                )?;
                tx.execute(
                    "UPDATE series SET confirmed = ?1 WHERE id = 7",
                    [i64::from(confirmed)],
                )?;
                Ok(())
            })
            .unwrap();
        let offer = Offer {
            anilist_id: None,
            tvmaze_id: Some(9),
            ..offer("[Group] Show - 05 [1080p]")
        };
        let e = proto::EpisodeRef {
            anilist_id: None,
            ..ep(5.0)
        };
        (offer, e)
    }

    fn match_of(services: &anibeam_foundation::services::Services) -> (i64, i64) {
        services
            .store
            .read(|c| {
                Ok(c.query_row(
                    "SELECT tvmaze_id, confirmed FROM series WHERE id = 7",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?)
            })
            .unwrap()
    }

    #[test]
    fn a_show_the_user_matched_keeps_its_match_and_its_file_opens() {
        let dir = tempfile::tempdir().unwrap();
        let (running, rooms) = rooms_over(dir.path(), anibeam_net::FakeHttp::new());
        let services = running.services.clone();
        // The download lands in a folder the user matched to another show,
        // season two, say, while the room plays season one.
        let (offer, e) = matched_show(&services, true);
        let failed = Failed::default();
        let mut matched = ShowMatch::default();
        let here = matched.try_apply(&rooms, "/lib/Show", &offer, &e, &failed);
        assert_eq!(matched.job, None, "the user's match is left alone");
        assert_eq!(matched.tries, 0);
        assert_eq!(match_of(&services), (3, 1));
        assert_eq!(here.map(|l| (l.series, l.file)), Some((Some(7), Some(70))));
        // The room's ids find nothing here, so the wait ends on the file
        // where the download landed, never on the timeout.
        let rooms = Arc::new(rooms);
        let settled = services.handle.block_on(async {
            tokio::time::timeout(
                Duration::from_secs(30),
                settle_in(&rooms, &e, &offer, "/lib/Show/05.mkv"),
            )
            .await
        });
        let local = settled.expect("settle_in waited").expect("settled in");
        assert_eq!(local.map(|l| l.file), Some(Some(70)));
        assert_eq!(match_of(&services), (3, 1));
        drop(rooms);
        running.shutdown();
    }

    #[test]
    fn an_auto_match_takes_the_room_s_and_a_retry_is_not_put_off_by_it() {
        let dir = tempfile::tempdir().unwrap();
        // Each match job asks TVmaze, which has no such show.
        let http = anibeam_net::FakeHttp::new();
        http.push_for("tvmaze", 404, "");
        http.push_for("tvmaze", 404, "");
        let (running, rooms) = rooms_over(dir.path(), http);
        let services = running.services.clone();
        let (offer, e) = matched_show(&services, false);
        let failed = Failed::default();
        let mut matched = ShowMatch::default();
        let here = matched.try_apply(&rooms, "/lib/Show", &offer, &e, &failed);
        assert_eq!(here, None);
        let first = matched.job.expect("an auto-match gets the room's match");
        // The room's match writes a confirmed match before its fetch, and the
        // fetch fails: that is the room's own match, so it is tried again.
        services
            .store
            .tx(|tx| {
                tx.execute("UPDATE series SET confirmed = 1 WHERE id = 7", [])?;
                Ok(())
            })
            .unwrap();
        anibeam_foundation::sync::recover(&failed).insert(first);
        matched.try_apply(&rooms, "/lib/Show", &offer, &e, &failed);
        let second = matched.job.expect("the failed match is started again");
        assert_ne!(first, second);
        assert_eq!(matched.tries, 2);
        drop(rooms);
        running.shutdown();
    }

    #[test]
    fn progress_reads_the_client_s_numbers() {
        let mut t = anibeam_downloads::qbt::tests_torrent();
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

    fn files(names: &[&str]) -> Vec<TorrentFile> {
        names
            .iter()
            .map(|n| TorrentFile {
                name: (*n).into(),
                size: 1,
                progress: 0.0,
                priority: 1,
                episode: None,
            })
            .collect()
    }

    #[test]
    fn a_batch_downloads_the_episode_s_file_alone() {
        let batch = files(&[
            "Cowboy Bebop (01-26)/[Lantern] Cowboy Bebop - 04 (1080p).mkv",
            "Cowboy Bebop (01-26)/[Lantern] Cowboy Bebop - 05 (1080p).mkv",
            "Cowboy Bebop (01-26)/[Lantern] Cowboy Bebop - 06 (1080p).mkv",
        ]);
        // The host's own file by its name
        let host = Offer {
            file: "[Lantern] Cowboy Bebop - 05 (1080p).mkv".into(),
            ..offer("[Lantern] Cowboy Bebop (01-26) (1080p) [Batch]")
        };
        assert_eq!(episode_file(&batch, &host, &ep(5.0)), Some(1));
        // A file renamed on the host's side is found by its number
        let renamed = Offer {
            file: "bebop five.mkv".into(),
            ..host.clone()
        };
        assert_eq!(episode_file(&batch, &renamed, &ep(6.0)), Some(2));
        // Two files of that number fit, so neither is picked
        let two = files(&["A - 05.mkv", "B - 05.mkv"]);
        assert_eq!(episode_file(&two, &renamed, &ep(5.0)), None);
        // One file is the whole torrent
        assert_eq!(episode_file(&batch[..1], &host, &ep(4.0)), None);
    }

    #[test]
    fn a_download_that_stops_moving_is_stuck_after_the_stall_time() {
        let t0 = Instant::now();
        let mut stall = Stall::new(t0);
        assert!(!stall.stuck(0.0, t0 + STALL / 2));
        // It moved, so the clock starts again from there
        assert!(!stall.stuck(0.1, t0 + STALL - Duration::from_secs(1)));
        assert!(!stall.stuck(0.1, t0 + STALL + Duration::from_secs(1)));
        assert!(stall.stuck(0.1, t0 + STALL * 2));
        // Metadata never came: no progress from the start
        let mut never = Stall::new(t0);
        assert!(never.stuck(0.0, t0 + STALL));
    }

    #[test]
    fn a_long_magnet_sheds_trackers_to_fit_the_cap() {
        let (host, member) = (
            super::super::seal::Keys::new(),
            super::super::seal::Keys::new(),
        );
        let opened = |sealed: &str| {
            let plain = member.open("K7QM2X", &host.public(), sealed).unwrap();
            match serde_json::from_slice::<Secret>(&plain).unwrap() {
                Secret::Offer(o) => o,
                other => panic!("{other:?}"),
            }
        };
        let trackers: String = (0..400)
            .map(|i| format!("&tr=udp%3A%2F%2Ftracker{i}.example%3A1337"))
            .collect();
        let long = Offer {
            magnet: Some(format!("{}{trackers}", magnet(1))),
            ..offer("[Lantern] Cowboy Bebop - 05 (1080p)")
        };
        let (sealed, torrent) =
            seal_offer(&host, "K7QM2X", &member.public(), long.clone()).unwrap();
        assert!(sealed.len() <= proto::MAX_SEALED);
        assert!(torrent, "the magnet went, with fewer trackers");
        // Every tracker that fits stays, from the front: one more would not seal
        let kept = opened(&sealed).magnet.unwrap();
        assert!(kept.starts_with(&format!("{}&tr=", magnet(1))), "{kept}");
        let n = kept.matches("&tr=").count();
        assert!(n > 0 && n < 400, "{n} trackers kept");
        let one_more = Offer {
            magnet: Some(format!("{kept}&tr=udp%3A%2F%2Ftracker{n}.example%3A1337")),
            ..long
        };
        let plain = serde_json::to_vec(&Secret::Offer(one_more)).unwrap();
        assert!(super::super::seal::sealed_len(plain.len()) > proto::MAX_SEALED);
        // A magnet with no trackers to shed goes, and the name is sent alone
        let bare = Offer {
            magnet: Some(format!("{}{}", magnet(1), "x".repeat(10_000))),
            ..offer("[Lantern] Cowboy Bebop - 05 (1080p)")
        };
        let (sealed, torrent) = seal_offer(&host, "K7QM2X", &member.public(), bare).unwrap();
        assert!(!torrent, "a name alone is no torrent");
        assert_eq!(opened(&sealed).magnet, None);
        // Names too long for the cap are cut
        let named = Offer {
            name: "x".repeat(20_000),
            ..offer("[Lantern] Cowboy Bebop - 05 (1080p)")
        };
        let (sealed, _) = seal_offer(&host, "K7QM2X", &member.public(), named).unwrap();
        assert_eq!(opened(&sealed).name.len(), 300);
        // A name too long beside a short magnet costs the name its tail, not the torrent
        let both = Offer {
            name: "x".repeat(7_000),
            magnet: Some(format!(
                "{}&tr=udp%3A%2F%2Ftracker.example%3A1337",
                magnet(1)
            )),
            ..offer("[Lantern] Cowboy Bebop - 05 (1080p)")
        };
        let (sealed, torrent) = seal_offer(&host, "K7QM2X", &member.public(), both).unwrap();
        assert!(torrent, "the torrent went");
        let got = opened(&sealed);
        assert_eq!(got.name.len(), 300);
        assert!(got.magnet.unwrap().contains("&tr="), "its tracker kept");
    }
}
