//! Casting: an episode played on a Chromecast or a DLNA TV, with the phone or the desktop as
//! the remote. The receiver fetches the episode from a small server here; nothing is
//! re-encoded, so only what the device can decode goes, and a file it cannot gets a reason
//! instead of a black screen.
//!
//! A Chromecast gets HLS: the Matroska file is read once for its keyframes and subtitles,
//! then cut into fragmented MP4 segments as they are asked for, so the TV seeks anywhere. A
//! DLNA TV gets the file as it is, which most take, and the same fragments as one
//! progressive stream when it refuses. An episode another Anibeam converts as it plays is
//! passed on from here: as HLS to a Chromecast, and to a DLNA TV as its MPEG-TS pieces joined
//! into one progressive stream. The session's position comes back once a second as
//! `CastChanged`, which is what the shell ticks the playback session with.

#[cfg(feature = "cast")]
pub mod castv2;
#[cfg(feature = "cast")]
pub mod discover;
#[cfg(feature = "cast")]
pub mod dlna;
#[cfg(feature = "cast")]
pub mod ebml;
#[cfg(feature = "cast")]
pub mod fmp4;
#[cfg(feature = "cast")]
pub mod media;
#[cfg(feature = "cast")]
pub mod mkv;
#[cfg(feature = "cast")]
pub mod server;
#[cfg(feature = "cast")]
pub mod vtt;

use crate::Playback;
use anibeam_contract::*;

#[derive(Default)]
pub struct Cast {
    #[cfg(feature = "cast")]
    inner: std::sync::Arc<imp::Inner>,
}

/// Every `Cast*` call.
pub fn call(playback: &std::sync::Arc<Playback>, call: Call) -> Result<Reply, CoreError> {
    #[cfg(feature = "cast")]
    {
        imp::call(playback, call)
    }
    #[cfg(not(feature = "cast"))]
    {
        let _ = playback;
        match call {
            Call::CastDevices => Ok(Reply::CastDevices {
                devices: Vec::new(),
            }),
            Call::CastStatus => Ok(Reply::CastState { state: None }),
            _ => Err(CoreError::invalid(
                "cast",
                anibeam_contract::i18n::tr("core-cast-not-built", &[]),
            )),
        }
    }
}

#[cfg(feature = "cast")]
mod imp {
    use std::net::SocketAddr;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::discover::{self, Found};
    use super::media::{Prepared, Receiver, Subs};
    use super::server::{self as served, Piece, Registry, Served};
    use super::{castv2, dlna};
    use crate::Playback;
    use anibeam_contract::i18n::tr;
    use anibeam_contract::*;
    use anibeam_foundation::services::Services;
    use anibeam_foundation::sync::recover;

    #[derive(Default)]
    pub struct Inner {
        devices: Mutex<Vec<Found>>,
        looking: AtomicBool,
        registry: Arc<Registry>,
        server: tokio::sync::OnceCell<SocketAddr>,
        session: Mutex<Option<Session>>,
        next: AtomicU64,
    }

    enum Control {
        /// The start task running, which a stop aborts, and the TV once the start has
        /// handed it something, which a stop then tells to stop.
        Starting {
            task: Option<tokio::task::AbortHandle>,
            tv: Option<Arc<dlna::Client>>,
        },
        Cast(castv2::Controller),
        Dlna {
            client: Arc<dlna::Client>,
            poll: tokio::task::AbortHandle,
        },
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Mode {
        Hls,
        Direct,
        /// The fragments as one stream from the keyframe at `offset` seconds. Some TVs count
        /// their position from the stream's start, some read the media's own times.
        Progressive {
            offset: f64,
        },
    }

    struct Session {
        id: u64,
        token: String,
        /// The device's id, so a session replaced on the same device leaves it alone.
        device: String,
        state: CastState,
        control: Control,
        mode: Mode,
        base: String,
        title: String,
        prepared: Option<Arc<Prepared>>,
        /// The pieces of a stream another machine converts, when a TV takes them as one
        /// progressive stream from here.
        pieces: Option<Arc<Vec<Piece>>>,
        /// HEVC to a Chromecast whose model is not known to decode it: a refusal says so.
        hevc_doubt: bool,
    }

    fn inner(playback: &Playback) -> Arc<Inner> {
        playback.cast.inner.clone()
    }

    fn devices(inner: &Inner) -> Vec<CastDevice> {
        recover(&inner.devices)
            .iter()
            .map(|f| f.device.clone())
            .collect()
    }

    fn found(playback: &Playback, f: Found) {
        let inner = inner(playback);
        let list = {
            let mut d = recover(&inner.devices);
            match d.iter_mut().find(|x| x.device.id == f.device.id) {
                Some(x) if *x == f => return,
                Some(x) => *x = f,
                None => d.push(f),
            }
            d.iter().map(|x| x.device.clone()).collect::<Vec<_>>()
        };
        playback.services.bus.debug(
            Stage::Playback,
            "cast devices",
            EventBody::CastDevicesChanged { devices: list },
        );
    }

    /// Looks around the network for a few seconds, once at a time.
    fn look(playback: &Arc<Playback>) {
        let inner = inner(playback);
        if inner.looking.swap(true, Ordering::AcqRel) {
            return;
        }
        let cast = playback.clone();
        let chromecasts = playback.services.handle.spawn_blocking(move || {
            if let Err(e) = discover::chromecasts(Duration::from_secs(6), |f| found(&cast, f)) {
                tracing::warn!("cast: {e}");
            }
        });
        let dlna = playback.clone();
        playback.services.handle.spawn(async move {
            let report = |f| found(&dlna, f);
            let extra = discover::dev().renderers;
            if !extra.is_empty() {
                discover::describe(extra, &report).await;
            }
            if let Err(e) = discover::renderers(Duration::from_secs(4), report).await {
                tracing::warn!("cast: {e}");
            }
            let _ = chromecasts.await;
            inner.looking.store(false, Ordering::Release);
        });
    }

    /// The device `id` among those found.
    fn find_device(inner: &Inner, id: &str) -> Result<Found, CoreError> {
        recover(&inner.devices)
            .iter()
            .find(|f| f.device.id == id)
            .cloned()
            .ok_or_else(|| CoreError::invalid("device", tr("core-cast-device-gone", &[])))
    }

    fn file_path(services: &Services, file: u64) -> Result<PathBuf, CoreError> {
        services
            .store
            .read(|c| {
                c.query_row("SELECT path FROM files WHERE id = ?1", [file as i64], |r| {
                    r.get::<_, String>(0)
                })
                .map_err(|_| CoreError::NotFound {
                    what: Entity::File,
                    id: file,
                })
            })
            .map(PathBuf::from)
    }

    /// A file's extension, lowercased.
    fn extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
    }

    fn mime(path: &Path) -> &'static str {
        match extension(path).as_deref() {
            Some("mkv") => "video/x-matroska",
            Some("webm") => "video/webm",
            Some("mp4") | Some("m4v") => "video/mp4",
            Some("ts") | Some("m2ts") => "video/mp2t",
            Some("avi") => "video/x-msvideo",
            Some("ogm") => "video/ogg",
            _ => "video/mp4",
        }
    }

    fn is_matroska(path: &Path) -> bool {
        matches!(extension(path).as_deref(), Some("mkv") | Some("webm"))
    }

    fn emit(services: &Services, state: &CastState, moved_only: bool) {
        let body = EventBody::CastChanged {
            state: state.clone(),
        };
        if moved_only {
            services.bus.debug(Stage::Playback, "cast position", body);
        } else {
            let device = &state.device;
            let (said, what) = match state.state {
                CastPlayState::Connecting => (
                    Some(anibeam_contract::words!(
                        "core-cast-connecting",
                        device = device
                    )),
                    format!("Casting to {device}"),
                ),
                CastPlayState::Playing => (None, format!("Playing on {device}")),
                CastPlayState::Paused => (None, format!("Paused on {device}")),
                CastPlayState::Buffering => (None, format!("Buffering on {device}")),
                CastPlayState::Ended => (
                    Some(anibeam_contract::words!("core-cast-ended", device = device)),
                    format!("Finished on {device}"),
                ),
                CastPlayState::Stopped => (
                    Some(anibeam_contract::words!(
                        "core-cast-stopped",
                        device = device
                    )),
                    format!("Stopped casting to {device}"),
                ),
                CastPlayState::Failed => (
                    Some(anibeam_contract::words!(
                        "core-cast-failed",
                        device = device
                    )),
                    format!(
                        "Casting to {device} failed: {}",
                        state.error.as_deref().unwrap_or_default()
                    ),
                ),
            };
            // A failure is a problem the person can act on; the rest is news
            let level = if state.state == CastPlayState::Failed {
                Level::Warn
            } else {
                Level::Info
            };
            services
                .bus
                .emit_said(level, Stage::Playback, said, what, None, body);
        }
    }

    /// Changes the session `id`'s state, if it is still the session, and says so.
    fn update(playback: &Playback, id: u64, f: impl FnOnce(&mut Session)) {
        let inner = inner(playback);
        let out = {
            let mut s = recover(&inner.session);
            let Some(session) = s.as_mut().filter(|s| s.id == id) else {
                return;
            };
            let before = session.state.clone();
            f(session);
            let after = session.state.clone();
            (after != before).then(|| {
                let moved_only = after.state == before.state
                    && after.error == before.error
                    && after.duration == before.duration;
                (after, moved_only)
            })
        };
        if let Some((state, moved_only)) = out {
            emit(&playback.services, &state, moved_only);
        }
    }

    fn no_route() -> String {
        tr("core-cast-no-route", &[])
    }

    fn no_control() -> String {
        tr("core-cast-no-control", &[])
    }

    fn not_handed(e: reqwest::Error) -> String {
        tr("core-cast-not-handed", &[("error", e.to_string().into())])
    }

    fn still_starting() -> CoreError {
        CoreError::invalid("cast", tr("core-cast-still-starting", &[]))
    }

    fn nothing() -> CoreError {
        CoreError::invalid("cast", tr("core-cast-nothing", &[]))
    }

    fn fail(playback: &Playback, id: u64, error: String) {
        update(playback, id, |s| {
            s.state.state = CastPlayState::Failed;
            s.state.error = Some(error);
        });
    }

    async fn server(inner: &Inner) -> Result<SocketAddr, String> {
        let reg = inner.registry.clone();
        inner
            .server
            .get_or_try_init(|| async move {
                super::server::serve(reg, discover::dev().port.unwrap_or(0)).await
            })
            .await
            .copied()
            .map_err(|e| tr("core-cast-serve-failed", &[("error", e.to_string().into())]))
    }

    /// Where `found` fetches the cast `token` from: this machine's server, started first if
    /// need be, at its address on the route to the device.
    async fn base(inner: &Inner, found: &Found, token: &str) -> Result<String, String> {
        let port = server(inner).await?.port();
        let ip = discover::local_ip_for(found.addr.ip()).ok_or_else(no_route)?;
        let host = discover::dev()
            .advertise
            .unwrap_or_else(|| format!("{ip}:{port}"));
        Ok(format!("http://{host}/c/{token}"))
    }

    /// Serves `path` under `token`, as it is and, once prepared, cut into fragments.
    fn serve_file(inner: &Inner, token: &str, path: &Path, prepared: Option<Arc<Prepared>>) {
        let served = Served::file(path.to_path_buf(), mime(path), prepared);
        inner.registry.add(token.to_string(), served);
    }

    /// Ends any cast and starts a session on `found`, connecting: its id, the token its
    /// device fetches under, and its state as the call answers.
    fn new_session(
        playback: &Playback,
        found: &Found,
        file: u64,
        title: String,
        start: f64,
        duration: f64,
        mode: Mode,
    ) -> (u64, String, CastState) {
        let inner = inner(playback);
        let id = inner.next.fetch_add(1, Ordering::Relaxed) + 1;
        let token = format!("{:032x}", rand::random::<u128>());
        let state = CastState {
            device: found.device.name.clone(),
            kind: found.device.kind,
            file,
            state: CastPlayState::Connecting,
            position: start,
            duration,
            error: None,
        };
        // Swapped in under one lock, so a start that races another still retires the
        // session it replaces.
        let old = recover(&inner.session).replace(Session {
            id,
            token: token.clone(),
            device: found.device.id.clone(),
            state: state.clone(),
            control: Control::Starting {
                task: None,
                tv: None,
            },
            mode,
            base: String::new(),
            title,
            prepared: None,
            pieces: None,
            hevc_doubt: false,
        });
        if let Some(old) = old {
            let here = old.device == found.device.id;
            retire(playback, old, here);
        }
        emit(&playback.services, &state, false);
        (id, token, state)
    }

    pub fn call(playback: &Arc<Playback>, call: Call) -> Result<Reply, CoreError> {
        let inner = inner(playback);
        match call {
            Call::CastDevices => {
                look(playback);
                Ok(Reply::CastDevices {
                    devices: devices(&inner),
                })
            }
            Call::CastStatus => {
                let s = recover(&inner.session);
                Ok(Reply::CastState {
                    state: s.as_ref().map(|s| s.state.clone()),
                })
            }
            Call::CastStart {
                device,
                file,
                start,
                audio_track,
                subtitle,
            } => {
                let found = find_device(&inner, &device)?;
                let path = file_path(&playback.services, file)?;
                let title = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let (id, token, state) = new_session(
                    playback,
                    &found,
                    file,
                    title.clone(),
                    start,
                    0.0,
                    Mode::Direct,
                );
                let subs = match subtitle {
                    CastSubtitle::None => None,
                    CastSubtitle::Track { number } => Some(Subs::Track(number)),
                    CastSubtitle::File { path } => Some(Subs::File(PathBuf::from(path))),
                };
                let arc = playback.clone();
                spawn_start(playback, id, async move {
                    begin(
                        &arc,
                        id,
                        found,
                        path,
                        token,
                        title,
                        start,
                        audio_track,
                        subs,
                    )
                    .await
                });
                Ok(Reply::CastState { state: Some(state) })
            }
            Call::CastStartStream {
                device,
                url,
                title,
                start,
                duration,
            } => {
                let found = find_device(&inner, &device)?;
                let Some((dir, name)) = url.split('?').next().and_then(|u| u.rsplit_once('/'))
                else {
                    return Err(CoreError::invalid(
                        "url",
                        tr("core-cast-not-a-playlist", &[]),
                    ));
                };
                let dir = format!("{dir}/");
                let playlist = match url.split_once('?') {
                    Some((_, query)) if !query.is_empty() => format!("{name}?{query}"),
                    _ => name.to_string(),
                };
                let mode = match found.device.kind {
                    CastKind::Chromecast => Mode::Hls,
                    CastKind::Dlna => Mode::Progressive { offset: start },
                };
                let (id, token, state) =
                    new_session(playback, &found, 0, title.clone(), start, duration, mode);
                let arc = playback.clone();
                spawn_start(playback, id, async move {
                    match found.device.kind {
                        CastKind::Chromecast => {
                            relay(
                                &arc, id, found, dir, playlist, token, title, start, duration,
                            )
                            .await
                        }
                        CastKind::Dlna => {
                            relay_dlna(&arc, id, found, dir, playlist, token, title, start).await
                        }
                    }
                });
                Ok(Reply::CastState { state: Some(state) })
            }
            Call::CastPlay => control(playback, |c, id| match c {
                Ctl::Cast(c) => c.send(castv2::Command::Play),
                Ctl::Dlna(d) => {
                    spawn_or_fail(playback, id, async move { d.play().await });
                }
            }),
            Call::CastPause => control(playback, |c, id| match c {
                Ctl::Cast(c) => c.send(castv2::Command::Pause),
                Ctl::Dlna(d) => {
                    spawn_or_fail(playback, id, async move { d.pause().await });
                }
            }),
            Call::CastSeek { to } => seek(playback, to),
            Call::CastStop => {
                stop(playback);
                Ok(Reply::Ok)
            }
            _ => Err(CoreError::internal("not a cast call")),
        }
    }

    enum Ctl<'a> {
        Cast(&'a castv2::Controller),
        Dlna(Arc<dlna::Client>),
    }

    /// Runs `f` with the session's controller and id. The session lock is held for the
    /// call, so `f` must not take it again: a DLNA action only spawns its request.
    fn control(playback: &Playback, f: impl FnOnce(Ctl<'_>, u64)) -> Result<Reply, CoreError> {
        let inner = inner(playback);
        let s = recover(&inner.session);
        let session = s.as_ref().ok_or_else(nothing)?;
        match &session.control {
            Control::Cast(c) => f(Ctl::Cast(c), session.id),
            Control::Dlna { client, .. } => f(Ctl::Dlna(client.clone()), session.id),
            Control::Starting { .. } => return Err(still_starting()),
        }
        Ok(Reply::Ok)
    }

    /// Runs `f` on the core's runtime, and fails the session `id` with what it answers.
    fn spawn_or_fail(
        playback: &Arc<Playback>,
        id: u64,
        f: impl std::future::Future<Output = Result<(), String>> + Send + 'static,
    ) -> tokio::task::AbortHandle {
        let arc = playback.clone();
        playback
            .services
            .handle
            .spawn(async move {
                if let Err(e) = f.await {
                    fail(&arc, id, e);
                }
            })
            .abort_handle()
    }

    /// Runs the start of session `id`, which a stop or a new start aborts.
    fn spawn_start(
        playback: &Arc<Playback>,
        id: u64,
        f: impl std::future::Future<Output = Result<(), String>> + Send + 'static,
    ) {
        let handle = spawn_or_fail(playback, id, f);
        let inner = inner(playback);
        let mut s = recover(&inner.session);
        match s.as_mut().filter(|s| s.id == id).map(|s| &mut s.control) {
            Some(Control::Starting { task, .. }) => *task = Some(handle),
            // Started already, or stopped before the handle got here.
            Some(_) => {}
            None => handle.abort(),
        }
    }

    /// The start of session `id` is about to hand `tv` something: from here a stop tells
    /// it to stop. False when the session is gone, and the start goes no further.
    fn reaching(inner: &Inner, id: u64, tv: &Arc<dlna::Client>) -> bool {
        let mut s = recover(&inner.session);
        match s.as_mut().filter(|s| s.id == id).map(|s| &mut s.control) {
            Some(Control::Starting { tv: slot, .. }) => {
                *slot = Some(tv.clone());
                true
            }
            Some(_) => true,
            None => false,
        }
    }

    fn seek(playback: &std::sync::Arc<Playback>, to: f64) -> Result<Reply, CoreError> {
        let inner = inner(playback);
        let (mode, base, title, id, prepared, pieces) = {
            let mut s = recover(&inner.session);
            let s = s.as_mut().ok_or_else(nothing)?;
            // Refused before anything changes: a seek the start never hears must not
            // leave its offset on the session.
            if matches!(s.control, Control::Starting { .. }) {
                return Err(still_starting());
            }
            s.state.position = to;
            (
                s.mode,
                s.base.clone(),
                s.title.clone(),
                s.id,
                s.prepared.clone(),
                s.pieces.clone(),
            )
        };
        if !matches!(mode, Mode::Progressive { .. }) {
            return control(playback, |c, id| match c {
                Ctl::Cast(c) => c.send(castv2::Command::Seek(to)),
                Ctl::Dlna(d) => {
                    spawn_or_fail(playback, id, async move { d.seek(to).await });
                }
            });
        }
        // A stream with no length seeks by starting again from the new point.
        let (name, kind, offset) = match (&pieces, prepared) {
            (Some(pieces), _) => (
                "stream.ts",
                "video/mp2t",
                pieces
                    .get(served::piece_at(pieces, to))
                    .map_or(to, |p| p.start),
            ),
            (None, Some(p)) => ("stream.mp4", "video/mp4", p.stream_start(to)),
            (None, None) => ("stream.mp4", "video/mp4", to),
        };
        let url = format!("{base}/{name}?t={to:.3}");
        update(playback, id, |s| s.mode = Mode::Progressive { offset });
        control(playback, |c, id| {
            if let Ctl::Dlna(d) = c {
                let meta = dlna::didl(&url, kind, &title, None);
                spawn_or_fail(playback, id, async move {
                    d.set_uri(&url, &meta).await?;
                    d.play().await
                });
            }
        })
    }

    fn stop(playback: &Playback) {
        let taken = recover(&inner(playback).session).take();
        if let Some(session) = taken {
            retire(playback, session, false);
        }
    }

    /// A session that is no longer the one casting: its files come off the server, the
    /// shells hear it stopped, and its device is stopped, unless `replaced` on that same
    /// device by a start that has not reached it yet.
    fn retire(playback: &Playback, session: Session, replaced: bool) {
        inner(playback).registry.remove(&session.token);
        let mut state = session.state.clone();
        if !matches!(state.state, CastPlayState::Ended | CastPlayState::Failed) {
            state.state = CastPlayState::Stopped;
            emit(&playback.services, &state, false);
        }
        end(playback, session.control, replaced);
    }

    /// Stops what a controller drives: dropping a Chromecast's stops the media receiver on
    /// the TV, and a DLNA TV is told to stop. A start still running is aborted, and a TV
    /// it handed something is told to stop as well. With `keep`, the device is playing
    /// the cast that replaced this one, so the controller lets go of it untouched.
    fn end(playback: &Playback, control: Control, keep: bool) {
        let tv = match control {
            Control::Cast(c) if keep => {
                c.leave();
                None
            }
            Control::Cast(c) => {
                drop(c);
                None
            }
            Control::Dlna { client, poll } => {
                poll.abort();
                Some(client)
            }
            Control::Starting { task, tv } => {
                if let Some(task) = task {
                    task.abort();
                }
                tv
            }
        };
        if let Some(tv) = tv.filter(|_| !keep) {
            playback.services.handle.spawn(async move {
                let _ = tv.stop().await;
            });
        }
    }

    /// Hands session `id` the controller its cast started with, and the mode it plays in.
    /// A session stopped or replaced in the moment since its start last awaited, which
    /// the abort could not reach, is not there to take it: what it started is stopped
    /// here, unless a cast on the same device replaced it, and its files come off.
    fn take_control(
        playback: &Playback,
        id: u64,
        token: &str,
        device: &str,
        mode: Option<Mode>,
        control: Control,
    ) {
        let inner = inner(playback);
        let mut s = recover(&inner.session);
        match s.as_mut().filter(|s| s.id == id) {
            Some(s) => {
                if let Some(mode) = mode {
                    s.mode = mode;
                }
                s.control = control;
            }
            None => {
                let here = s.as_ref().is_some_and(|s| s.device == device);
                drop(s);
                inner.registry.remove(token);
                end(playback, control, here);
            }
        }
    }

    /// A Chromecast's controller for `load`, its status read into session `id`.
    fn cast_control(
        playback: &Arc<Playback>,
        id: u64,
        found: &Found,
        load: castv2::Load,
    ) -> Control {
        let status = playback.clone();
        Control::Cast(castv2::start(found.addr, load, move |st| {
            on_cast_status(&status, id, st)
        }))
    }

    /// A DLNA TV's controller, its status read into session `id` once a second.
    fn dlna_control(playback: &Arc<Playback>, id: u64, client: Arc<dlna::Client>) -> Control {
        let poll = playback
            .services
            .handle
            .spawn(poll_dlna(playback.clone(), id, client.clone()))
            .abort_handle();
        Control::Dlna { client, poll }
    }

    /// Starts a relayed stream on a Chromecast: the playlist under this machine's own
    /// server, every request passed on to `dir`.
    #[allow(clippy::too_many_arguments)]
    async fn relay(
        playback: &Arc<Playback>,
        id: u64,
        found: Found,
        dir: String,
        playlist: String,
        token: String,
        title: String,
        start: f64,
        duration: f64,
    ) -> Result<(), String> {
        let inner = inner(playback);
        let base = base(&inner, &found, &token).await?;
        let served = Served::relayed("application/vnd.apple.mpegurl", dir, Arc::default());
        inner.registry.add(token.clone(), served);
        update(playback, id, |s| s.base = base.clone());
        let load = castv2::Load {
            url: format!("{base}/{playlist}"),
            content_type: "application/x-mpegurl".to_string(),
            hls_fmp4: false,
            start,
            duration,
            title,
        };
        let control = cast_control(playback, id, &found, load);
        take_control(playback, id, &token, &found.device.id, None, control);
        Ok(())
    }

    /// Starts a stream another machine converts on a TV over DLNA. Such a TV reads no HLS,
    /// so this machine reads the playlist once and hands the TV its pieces from the one
    /// holding `start` as one progressive MPEG-TS stream, fetching each from `dir` as the
    /// TV reads. A seek starts that stream again (`seek`).
    #[allow(clippy::too_many_arguments)]
    async fn relay_dlna(
        playback: &Arc<Playback>,
        id: u64,
        found: Found,
        dir: String,
        playlist: String,
        token: String,
        title: String,
        start: f64,
    ) -> Result<(), String> {
        let inner = inner(playback);
        let client = Arc::new(dlna::Client::new(
            found.control.clone().ok_or_else(no_control)?,
        ));
        let list = served::relay_client()
            .get(format!("{dir}{playlist}"))
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(not_handed)?
            .text()
            .await
            .map_err(not_handed)?;
        // The fetch can take as long as the other computer needs to start converting; a
        // stop meanwhile aborts this, and the TV is left alone.
        let pieces = Arc::new(served::read_pieces(&list));
        if pieces.is_empty() {
            return Err(tr("core-cast-empty-playlist", &[]));
        }
        let base = base(&inner, &found, &token).await?;
        let served = Served::relayed("video/mp2t", dir, pieces.clone());
        inner.registry.add(token.clone(), served);
        let offset = pieces
            .get(served::piece_at(&pieces, start))
            .map_or(0.0, |p| p.start);
        update(playback, id, |s| {
            s.base = base.clone();
            s.pieces = Some(pieces.clone());
            s.mode = Mode::Progressive { offset };
        });
        let url = format!("{base}/stream.ts?t={start:.3}");
        let meta = dlna::didl(&url, "video/mp2t", &title, None);
        if !reaching(&inner, id, &client) {
            return Ok(());
        }
        client.set_uri(&url, &meta).await?;
        client.play().await?;
        let control = dlna_control(playback, id, client);
        take_control(playback, id, &token, &found.device.id, None, control);
        Ok(())
    }

    /// Reads a Matroska file through once, off the runtime's threads.
    async fn prepare(
        path: &Path,
        audio_track: Option<u64>,
        subs: Option<Subs>,
    ) -> Result<Arc<Prepared>, String> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || Prepared::new(&path, audio_track, subs))
            .await
            .map_err(|e| e.to_string())?
            .map(Arc::new)
    }

    #[allow(clippy::too_many_arguments)]
    async fn begin(
        playback: &Arc<Playback>,
        id: u64,
        found: Found,
        path: PathBuf,
        token: String,
        title: String,
        start: f64,
        audio_track: Option<u64>,
        subs: Option<Subs>,
    ) -> Result<(), String> {
        let inner = inner(playback);
        let base = base(&inner, &found, &token).await?;
        let mkv = is_matroska(&path);
        let receiver = match found.device.kind {
            CastKind::Chromecast => Receiver::Cast,
            CastKind::Dlna => Receiver::Dlna,
        };
        // A Matroska file is read through once: keyframes for HLS, subtitles for both.
        let prepared = if mkv && (receiver == Receiver::Cast || subs.is_some()) {
            Some(prepare(&path, audio_track, subs).await?)
        } else {
            None
        };
        let hevc_doubt = found.device.kind == CastKind::Chromecast
            && prepared
                .as_ref()
                .and_then(|p| p.probe.video.as_ref())
                .is_some_and(|v| matches!(v.codec, super::mkv::VideoCodec::Hevc { .. }))
            && !discover::decodes_hevc(found.device.model.as_deref());
        serve_file(&inner, &token, &path, prepared.clone());
        let duration = prepared
            .as_ref()
            .map(|p| p.duration_ms() as f64 / 1000.0)
            .unwrap_or(0.0);
        update(playback, id, |s| {
            s.base = base.clone();
            s.prepared = prepared.clone();
            s.hevc_doubt = hevc_doubt;
            s.state.duration = duration;
        });
        match found.device.kind {
            CastKind::Chromecast => {
                let (url, content_type, hls) = match &prepared {
                    Some(_) => (
                        format!("{base}/master.m3u8"),
                        "application/x-mpegurl".to_string(),
                        true,
                    ),
                    None => (format!("{base}/file"), mime(&path).to_string(), false),
                };
                let load = castv2::Load {
                    url,
                    content_type,
                    hls_fmp4: hls,
                    start,
                    duration,
                    title,
                };
                let mode = if hls { Mode::Hls } else { Mode::Direct };
                let control = cast_control(playback, id, &found, load);
                take_control(playback, id, &token, &found.device.id, Some(mode), control);
            }
            CastKind::Dlna => {
                let client = Arc::new(dlna::Client::new(
                    found.control.clone().ok_or_else(no_control)?,
                ));
                let srt = prepared
                    .as_ref()
                    .filter(|p| p.cues.is_some())
                    .map(|_| format!("{base}/subs.srt"));
                let direct = format!("{base}/file");
                let meta = dlna::didl(&direct, mime(&path), &title, srt.as_deref());
                if !reaching(&inner, id, &client) {
                    return Ok(());
                }
                let mode = match client.set_uri(&direct, &meta).await {
                    Ok(()) => {
                        client.play().await?;
                        if start > 2.0 {
                            // Some TVs take a seek only once playing; a refusal still plays from the start.
                            tokio::time::sleep(Duration::from_millis(1500)).await;
                            let _ = client.seek(start).await;
                        }
                        Mode::Direct
                    }
                    Err(first) => {
                        // The TV would not take the file as it is; the fragments as one MP4 it may.
                        if !mkv {
                            return Err(first);
                        }
                        let p = match prepared {
                            Some(p) => p,
                            None => {
                                let p = prepare(&path, audio_track, None).await?;
                                serve_file(&inner, &token, &path, Some(p.clone()));
                                // A seek restarts the stream at a keyframe it reads from here
                                update(playback, id, |s| s.prepared = Some(p.clone()));
                                p
                            }
                        };
                        let url = format!("{base}/stream.mp4?t={start:.3}");
                        let meta = dlna::didl(&url, "video/mp4", &title, srt.as_deref());
                        client.set_uri(&url, &meta).await.map_err(|e| {
                            tr(
                                "core-cast-refused-both",
                                &[("error", e.into()), ("first", first.into())],
                            )
                        })?;
                        client.play().await?;
                        Mode::Progressive {
                            offset: p.stream_start(start),
                        }
                    }
                };
                let control = dlna_control(playback, id, client);
                take_control(playback, id, &token, &found.device.id, Some(mode), control);
            }
        }
        Ok(())
    }

    fn on_cast_status(playback: &Playback, id: u64, st: castv2::Status) {
        update(playback, id, |s| {
            let next = match (st.player.as_str(), st.idle_reason.as_deref()) {
                ("PLAYING", _) => CastPlayState::Playing,
                ("PAUSED", _) => CastPlayState::Paused,
                ("BUFFERING", _) | ("LOADING", _) => CastPlayState::Buffering,
                ("IDLE", Some("FINISHED")) => CastPlayState::Ended,
                ("IDLE", Some("ERROR")) => CastPlayState::Failed,
                ("IDLE", Some(_)) => CastPlayState::Stopped,
                _ => s.state.state,
            };
            if next == CastPlayState::Failed {
                let mut e = st
                    .error
                    .clone()
                    .unwrap_or_else(|| tr("core-cast-chromecast-stopped", &[]));
                if s.hevc_doubt {
                    e = tr("core-cast-hevc", &[("error", e.into())]);
                }
                s.state.error = Some(e);
            }
            s.state.state = next;
            if matches!(
                next,
                CastPlayState::Playing | CastPlayState::Paused | CastPlayState::Buffering
            ) && st.position > 0.0
            {
                s.state.position = st.position;
            }
            if let Some(d) = st.duration.filter(|d| *d > 0.0) {
                s.state.duration = d;
            }
        });
    }

    async fn poll_dlna(playback: Arc<Playback>, id: u64, client: Arc<dlna::Client>) {
        let mut seen_playing = false;
        let mut misses = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            match client.status().await {
                Ok((state, at, duration)) => {
                    misses = 0;
                    let mut finished = false;
                    update(&playback, id, |s| {
                        let offset = match s.mode {
                            Mode::Progressive { offset } => offset,
                            _ => 0.0,
                        };
                        s.state.state = match state.as_str() {
                            "PLAYING" => {
                                seen_playing = true;
                                CastPlayState::Playing
                            }
                            "PAUSED_PLAYBACK" => CastPlayState::Paused,
                            "TRANSITIONING" => CastPlayState::Buffering,
                            // Stopped after it played: the end, or someone pressed stop on the TV.
                            "STOPPED" | "NO_MEDIA_PRESENT" if seen_playing => {
                                finished = true;
                                let near_end = s.state.duration > 0.0
                                    && s.state.position > s.state.duration - 30.0;
                                if near_end {
                                    CastPlayState::Ended
                                } else {
                                    CastPlayState::Stopped
                                }
                            }
                            _ => s.state.state,
                        };
                        if let Some(at) = at.filter(|a| *a > 0.0) {
                            s.state.position = if offset > 0.0 && at + 1.0 >= offset {
                                at
                            } else {
                                offset + at
                            };
                        }
                        if let Some(d) = duration
                            && s.state.duration <= 0.0
                            && matches!(s.mode, Mode::Direct)
                        {
                            s.state.duration = d;
                        }
                    });
                    if finished {
                        return;
                    }
                }
                Err(e) => {
                    misses += 1;
                    if misses >= 5 {
                        fail(&playback, id, e);
                        return;
                    }
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        //! A TV over DLNA and another computer, both faked on loopback: no discovery runs and
        //! nothing leaves this machine.

        use std::sync::{Arc, Mutex};
        use std::time::{Duration, Instant};

        use axum::Router;
        use axum::body::Bytes;
        use axum::extract::{Path, State};
        use axum::http::{HeaderMap, Uri};
        use axum::routing::{get, post};

        use super::*;

        use anibeam_foundation::testing::Running;

        /// The playback team over the teams below it, each built as
        /// `Core::open` builds it, and the services they run on. None of
        /// `Core::open`'s links: casting reads none of them.
        struct Teams {
            playback: Arc<Playback>,
            _running: Running,
        }

        impl Teams {
            fn open(dir: &std::path::Path) -> Teams {
                let running = Running::open(dir, anibeam_net::FakeHttp::new());
                let services = running.services.clone();
                let net = Arc::new(anibeam_net::Net::new(
                    services.clone(),
                    anibeam_library::cards::cards_for,
                ));
                let trackers = Arc::new(anibeam_trackers::Trackers::new(
                    services.clone(),
                    net.clone(),
                ));
                let downloads = Arc::new(anibeam_downloads::Downloads::new(
                    services.clone(),
                    net.clone(),
                ));
                let playback = Arc::new(Playback::new(services, net, trackers, downloads));
                Teams {
                    playback,
                    _running: running,
                }
            }

            /// A `Cast*` call, as the core hands it on.
            fn call(&self, c: Call) -> Result<Reply, CoreError> {
                call(&self.playback, c)
            }
        }

        /// What the fake TV was told and what it read.
        #[derive(Default)]
        struct Tv {
            uris: Vec<String>,
            meta: Vec<String>,
            read: Vec<Vec<u8>>,
            /// Every action but the two status reads, in order.
            actions: Vec<String>,
            /// Answers Play only after a while, as a TV that is slow to start does.
            slow_play: bool,
            /// Answers Play with an error, as a TV that started anyway may.
            refuse_play: bool,
        }

        fn piece(i: u8) -> Vec<u8> {
            let mut p = vec![0x47u8; 188];
            p[1] = i;
            p
        }

        fn tag(xml: &str, name: &str) -> String {
            roxmltree::Document::parse(xml)
                .ok()
                .and_then(|d| {
                    d.descendants()
                        .find(|n| n.has_tag_name(name))
                        .and_then(|n| n.text().map(str::to_string))
                })
                .unwrap_or_default()
        }

        async fn tv(
            State(tv): State<Arc<Mutex<Tv>>>,
            headers: HeaderMap,
            body: Bytes,
        ) -> (axum::http::StatusCode, String) {
            let ok = |body: &str| (axum::http::StatusCode::OK, body.to_string());
            let action = headers
                .get("SOAPAction")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string();
            let xml = String::from_utf8_lossy(&body).into_owned();
            if !action.contains("#Get") {
                tv.lock().unwrap().actions.push(action.clone());
            }
            let (slow, refuse) = {
                let t = tv.lock().unwrap();
                (t.slow_play, t.refuse_play)
            };
            if slow && action.contains("#Play") {
                tokio::time::sleep(Duration::from_millis(1500)).await;
            }
            if refuse && action.contains("#Play") {
                let fault = "<r><errorCode>701</errorCode></r>".to_string();
                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, fault);
            }
            if action.contains("#SetAVTransportURI") {
                let mut t = tv.lock().unwrap();
                t.uris.push(tag(&xml, "CurrentURI"));
                t.meta.push(tag(&xml, "CurrentURIMetadata"));
            } else if action.contains("#Play") {
                let url = tv.lock().unwrap().uris.last().cloned().unwrap_or_default();
                let tv = tv.clone();
                tokio::spawn(async move {
                    // A stream that breaks off still counts what arrived before it did.
                    let mut got = Vec::new();
                    if let Ok(mut r) = reqwest::get(&url).await {
                        while let Ok(Some(chunk)) = r.chunk().await {
                            got.extend_from_slice(&chunk);
                        }
                    }
                    tv.lock().unwrap().read.push(got);
                });
            } else if action.contains("#GetTransportInfo") {
                return ok("<r><CurrentTransportState>PLAYING</CurrentTransportState></r>");
            } else if action.contains("#GetPositionInfo") {
                // Counted from the start of the stream it was handed, as many TVs do.
                return ok(
                    "<r><RelTime>0:00:02</RelTime><TrackDuration>0:00:00</TrackDuration></r>",
                );
            }
            ok("<ok/>")
        }

        async fn computer(
            State(asked): State<Arc<Mutex<Vec<String>>>>,
            Path(name): Path<String>,
            uri: Uri,
        ) -> Vec<u8> {
            asked.lock().unwrap().push(uri.to_string());
            if name == "index.m3u8" {
                return anibeam_host_api::hls_playlist(18.0, "?burn=0")
                    .unwrap()
                    .into_bytes();
            }
            let i: u8 = name
                .strip_prefix("seg")
                .and_then(|n| n.strip_suffix(".ts"))
                .and_then(|n| n.parse().ok())
                .unwrap();
            piece(i)
        }

        fn until(what: &str, mut f: impl FnMut() -> bool) {
            let deadline = Instant::now() + Duration::from_secs(10);
            while !f() {
                assert!(Instant::now() < deadline, "timed out waiting for {what}");
                std::thread::sleep(Duration::from_millis(20));
            }
        }

        /// A fake TV and a playback team that knows it, on loopback, for the tests below.
        fn tv_and_teams(
            rt: &tokio::runtime::Runtime,
        ) -> (Arc<Mutex<Tv>>, tempfile::TempDir, Teams) {
            let tv_state = Arc::new(Mutex::new(Tv::default()));
            let tv_at = serve(
                rt,
                Router::new()
                    .route("/ctl", post(tv))
                    .with_state(tv_state.clone()),
            );
            let dir = tempfile::tempdir().unwrap();
            let teams = Teams::open(dir.path());
            inner(&teams.playback).devices.lock().unwrap().push(Found {
                device: CastDevice {
                    id: "dlna:fake".into(),
                    name: "Fake TV".into(),
                    kind: CastKind::Dlna,
                    model: None,
                    address: tv_at.to_string(),
                },
                addr: tv_at,
                control: Some(format!("http://{tv_at}/ctl")),
            });
            (tv_state, dir, teams)
        }

        fn serve(rt: &tokio::runtime::Runtime, app: Router) -> std::net::SocketAddr {
            rt.block_on(async {
                let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                let at = l.local_addr().unwrap();
                tokio::spawn(async move { axum::serve(l, app).await });
                at
            })
        }

        #[test]
        fn a_stop_while_the_playlist_comes_leaves_the_tv_alone() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (tv_state, _dir, teams) = tv_and_teams(&rt);
            // The other computer takes its time over the playlist, as it does
            // while it starts converting.
            let slow = Router::new().route(
                "/hls/{name}",
                get(|| async {
                    tokio::time::sleep(Duration::from_millis(800)).await;
                    anibeam_host_api::hls_playlist(18.0, "").unwrap()
                }),
            );
            let computer_at = serve(&rt, slow);
            teams
                .call(Call::CastStartStream {
                    device: "dlna:fake".into(),
                    url: format!("http://{computer_at}/hls/index.m3u8"),
                    title: "Show 1".into(),
                    start: 0.0,
                    duration: 18.0,
                })
                .unwrap();
            teams.call(Call::CastStop).unwrap();
            std::thread::sleep(Duration::from_millis(1500));
            let t = tv_state.lock().unwrap();
            assert!(t.uris.is_empty(), "the TV was handed {:?}", t.uris);
            assert!(t.actions.is_empty(), "the TV was told {:?}", t.actions);
        }

        /// A file cast to a TV that is stopped before the TV has started it: the TV is told
        /// to stop once it answers, rather than left playing with nothing to control it,
        /// whether it answers Play with yes or with an error.
        #[test]
        fn a_stop_while_the_tv_starts_stops_the_tv() {
            stop_while_starting(false);
            stop_while_starting(true);
        }

        /// A library holding one episode, file 10, an MP4, for a cast to start from.
        fn one_file(teams: &Teams) {
            teams
                .playback
                .services
                .store
                .tx(|tx| {
                    tx.execute_batch(
                        "INSERT INTO sources (id, path, available, added_at) VALUES (1, '/lib', 1, 0);
                         INSERT INTO series (id, source_id, kind, path, folder_name, added_at)
                             VALUES (1, 1, 'show', '/lib/Show', 'Show', 0);
                         INSERT INTO files (id, series_id, path, size, mtime, kind, number, label, episode_key, seen_at)
                             VALUES (10, 1, '/lib/Show/01.mp4', 1, 0, 'episode', 1, '1', '1', 0);",
                    )?;
                    Ok(())
                })
                .unwrap();
        }

        fn cast_file(teams: &Teams) {
            teams
                .call(Call::CastStart {
                    device: "dlna:fake".into(),
                    file: 10,
                    start: 0.0,
                    audio_track: None,
                    subtitle: CastSubtitle::None,
                })
                .unwrap();
        }

        fn plays(tv: &Mutex<Tv>) -> usize {
            let t = tv.lock().unwrap();
            t.actions.iter().filter(|a| a.contains("#Play")).count()
        }

        fn stops(tv: &Mutex<Tv>) -> usize {
            let t = tv.lock().unwrap();
            t.actions.iter().filter(|a| a.contains("#Stop")).count()
        }

        fn stop_while_starting(refuse_play: bool) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (tv_state, _dir, teams) = tv_and_teams(&rt);
            {
                let mut t = tv_state.lock().unwrap();
                t.slow_play = true;
                t.refuse_play = refuse_play;
            }
            one_file(&teams);
            cast_file(&teams);
            until("the TV to be told to play", || plays(&tv_state) == 1);
            teams.call(Call::CastStop).unwrap();
            until("the TV to be told to stop", || stops(&tv_state) == 1);
        }

        /// A second cast to the same TV while the first is still starting: the first's
        /// start is let go without a stop, which would land on the TV as it plays the
        /// second.
        #[test]
        fn a_new_cast_to_the_same_tv_is_not_stopped_by_the_one_it_replaced() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (tv_state, _dir, teams) = tv_and_teams(&rt);
            tv_state.lock().unwrap().slow_play = true;
            one_file(&teams);
            cast_file(&teams);
            until("the first Play", || plays(&tv_state) == 1);
            cast_file(&teams);
            until("the second cast to play", || {
                matches!(
                    teams.call(Call::CastStatus),
                    Ok(Reply::CastState { state: Some(s) }) if s.state == CastPlayState::Playing
                )
            });
            // Long enough for the first Play to answer, and a stray stop to follow it.
            std::thread::sleep(Duration::from_millis(2000));
            assert_eq!(plays(&tv_state), 2);
            assert_eq!(
                stops(&tv_state),
                0,
                "{:?}",
                tv_state.lock().unwrap().actions
            );
        }

        #[test]
        fn a_redirect_from_the_other_computer_is_not_followed() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (tv_state, _dir, teams) = tv_and_teams(&rt);
            let elsewhere = serve(
                &rt,
                Router::new().route("/secret", get(|| async { "not for the TV" })),
            );
            let computer = Router::new().route(
                "/hls/{name}",
                get(move |Path(name): Path<String>| async move {
                    if name == "index.m3u8" {
                        return (
                            axum::http::StatusCode::OK,
                            [(axum::http::header::LOCATION, String::new())],
                            anibeam_host_api::hls_playlist(12.0, "")
                                .unwrap()
                                .into_bytes(),
                        );
                    }
                    if name == "seg0.ts" {
                        return (
                            axum::http::StatusCode::OK,
                            [(axum::http::header::LOCATION, String::new())],
                            piece(0),
                        );
                    }
                    (
                        axum::http::StatusCode::FOUND,
                        [(
                            axum::http::header::LOCATION,
                            format!("http://{elsewhere}/secret"),
                        )],
                        Vec::new(),
                    )
                }),
            );
            let computer_at = serve(&rt, computer);
            teams
                .call(Call::CastStartStream {
                    device: "dlna:fake".into(),
                    url: format!("http://{computer_at}/hls/index.m3u8"),
                    title: "Show 1".into(),
                    start: 0.0,
                    duration: 12.0,
                })
                .unwrap();
            until("the TV to read the stream", || {
                !tv_state.lock().unwrap().read.is_empty()
            });
            let t = tv_state.lock().unwrap();
            // The first piece, and nothing the redirect pointed at.
            assert_eq!(t.read[0], piece(0));
        }

        #[test]
        fn a_dlna_tv_plays_another_computers_conversion_as_one_stream() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (tv_state, _dir, teams) = tv_and_teams(&rt);
            let asked = Arc::new(Mutex::new(Vec::new()));
            let computer_at = serve(
                &rt,
                Router::new()
                    .route("/hls/{name}", get(computer))
                    .with_state(asked.clone()),
            );

            teams
                .call(Call::CastStartStream {
                    device: "dlna:fake".into(),
                    url: format!("http://{computer_at}/hls/index.m3u8?burn=0"),
                    title: "Show 1".into(),
                    start: 7.0,
                    duration: 18.0,
                })
                .expect("a TV over DLNA takes another computer's episode");

            until("the TV to read the stream", || {
                !tv_state.lock().unwrap().read.is_empty()
            });
            {
                let t = tv_state.lock().unwrap();
                assert!(t.uris[0].ends_with("/stream.ts?t=7.000"), "{:?}", t.uris);
                assert!(t.meta[0].contains("video/mp2t"), "{:?}", t.meta);
                // Second 7 is in the piece from 6: that piece and every one after it.
                assert_eq!(t.read[0], [piece(1), piece(2)].concat());
            }
            // Each piece is asked for with the conversion's own query.
            assert!(
                asked
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|u| u == "/hls/seg1.ts?burn=0"),
                "{:?}",
                asked.lock().unwrap()
            );
            // The TV counts from the start of what it was handed: 6 plus its 2.
            until("the position from the TV", || {
                matches!(
                    teams.call(Call::CastStatus),
                    Ok(Reply::CastState { state: Some(s) }) if (s.position - 8.0).abs() < 0.01
                )
            });

            teams.call(Call::CastSeek { to: 13.5 }).unwrap();
            until("the TV to read the stream again", || {
                tv_state.lock().unwrap().read.len() == 2
            });
            {
                let t = tv_state.lock().unwrap();
                assert!(t.uris[1].ends_with("/stream.ts?t=13.500"), "{:?}", t.uris);
                assert!(t.meta[1].contains("video/mp2t"), "{:?}", t.meta);
                assert_eq!(t.read[1], piece(2));
            }
            until("the position after the seek", || {
                matches!(
                    teams.call(Call::CastStatus),
                    Ok(Reply::CastState { state: Some(s) }) if (s.position - 14.0).abs() < 0.01
                )
            });
            teams.call(Call::CastStop).unwrap();
        }
    }
}
