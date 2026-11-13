//! Casting: an episode played on a Chromecast or a DLNA TV, with the phone or the desktop as
//! the remote. The receiver fetches the episode from a small server here; nothing is
//! re-encoded, so only what the device can decode goes, and a file it cannot gets a reason
//! instead of a black screen.
//!
//! A Chromecast gets HLS: the Matroska file is read once for its keyframes and subtitles,
//! then cut into fragmented MP4 segments as they are asked for, so the TV seeks anywhere. A
//! DLNA TV gets the file as it is, which most take, and the same fragments as one
//! progressive stream when it refuses. An episode another AniBeam converts as it plays is
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

use crate::contract::*;
use crate::core::Core;

#[derive(Default)]
pub struct Cast {
    #[cfg(feature = "cast")]
    inner: std::sync::Arc<imp::Inner>,
}

/// Every `Cast*` call.
pub fn call(core: &Core, call: Call) -> Result<Reply, CoreError> {
    #[cfg(feature = "cast")]
    {
        imp::call(core, call)
    }
    #[cfg(not(feature = "cast"))]
    {
        let _ = core;
        match call {
            Call::CastDevices => Ok(Reply::CastDevices {
                devices: Vec::new(),
            }),
            Call::CastStatus => Ok(Reply::CastState { state: None }),
            _ => Err(CoreError::invalid(
                "cast",
                crate::i18n::tr("core-cast-not-built", &[]),
            )),
        }
    }
}

#[cfg(feature = "cast")]
mod imp {
    use std::net::{IpAddr, SocketAddr};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::discover::{self, Found};
    use super::media::{Prepared, Receiver, Subs};
    use super::server::{self as served, Piece, Registry, Relay, Served};
    use super::{castv2, dlna};
    use crate::contract::*;
    use crate::core::Core;

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
        Starting,
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

    fn inner(core: &Core) -> Arc<Inner> {
        core.cast.inner.clone()
    }

    fn devices(inner: &Inner) -> Vec<CastDevice> {
        inner
            .devices
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .map(|f| f.device.clone())
            .collect()
    }

    fn found(core: &Arc<Core>, f: Found) {
        let inner = inner(core);
        let list = {
            let mut d = inner.devices.lock().unwrap_or_else(|e| e.into_inner());
            match d.iter_mut().find(|x| x.device.id == f.device.id) {
                Some(x) if *x == f => return,
                Some(x) => *x = f,
                None => d.push(f),
            }
            d.iter().map(|x| x.device.clone()).collect::<Vec<_>>()
        };
        core.bus.debug(
            Stage::Playback,
            "cast devices",
            EventBody::CastDevicesChanged { devices: list },
        );
    }

    /// Looks around the network for a few seconds, once at a time.
    fn look(core: &Core) {
        let inner = inner(core);
        if inner.looking.swap(true, Ordering::AcqRel) {
            return;
        }
        let Some(arc) = core.arc() else { return };
        let cast_core = arc.clone();
        let chromecasts = core.handle.spawn_blocking(move || {
            let c = cast_core.clone();
            if let Err(e) = discover::chromecasts(Duration::from_secs(6), move |f| found(&c, f)) {
                tracing::warn!("cast: {e}");
            }
        });
        let dlna_core = arc.clone();
        core.handle.spawn(async move {
            let c = dlna_core.clone();
            let extra = discover::dev().renderers;
            if !extra.is_empty() {
                let c = c.clone();
                discover::describe(extra, &move |f| found(&c, f)).await;
            }
            if let Err(e) = discover::renderers(Duration::from_secs(4), move |f| found(&c, f)).await
            {
                tracing::warn!("cast: {e}");
            }
            let _ = chromecasts.await;
            inner.looking.store(false, Ordering::Release);
        });
    }

    fn file_path(core: &Core, file: u64) -> Result<PathBuf, CoreError> {
        core.store
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

    fn mime(path: &Path) -> &'static str {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("mkv") => "video/x-matroska",
            Some("webm") => "video/webm",
            Some("mp4") | Some("m4v") => "video/mp4",
            Some("ts") | Some("m2ts") => "video/mp2t",
            Some("avi") => "video/x-msvideo",
            _ => "video/mp4",
        }
    }

    fn is_matroska(path: &Path) -> bool {
        matches!(
            path.extension()
                .and_then(|e| e.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("mkv") | Some("webm")
        )
    }

    fn emit(core: &Core, state: &CastState, moved_only: bool) {
        let body = EventBody::CastChanged {
            state: state.clone(),
        };
        if moved_only {
            core.bus.debug(Stage::Playback, "cast position", body);
        } else {
            let device = &state.device;
            let said = match state.state {
                CastPlayState::Connecting => {
                    Some(crate::words!("core-cast-connecting", device = device))
                }
                CastPlayState::Ended => Some(crate::words!("core-cast-ended", device = device)),
                CastPlayState::Stopped => Some(crate::words!("core-cast-stopped", device = device)),
                CastPlayState::Failed => Some(crate::words!("core-cast-failed", device = device)),
                CastPlayState::Playing | CastPlayState::Paused | CastPlayState::Buffering => None,
            };
            let what = match state.state {
                CastPlayState::Connecting => format!("Casting to {}", state.device),
                CastPlayState::Playing => format!("Playing on {}", state.device),
                CastPlayState::Paused => format!("Paused on {}", state.device),
                CastPlayState::Buffering => format!("Buffering on {}", state.device),
                CastPlayState::Ended => format!("Finished on {}", state.device),
                CastPlayState::Stopped => format!("Stopped casting to {}", state.device),
                CastPlayState::Failed => format!(
                    "Casting to {} failed: {}",
                    state.device,
                    state.error.clone().unwrap_or_default()
                ),
            };
            // A failure is a problem the person can act on; the rest is news
            let level = if state.state == CastPlayState::Failed {
                Level::Warn
            } else {
                Level::Info
            };
            core.bus
                .emit_said(level, Stage::Playback, said, what, None, body);
        }
    }

    /// Changes the session `id`'s state, if it is still the session, and says so.
    fn update(core: &Core, id: u64, f: impl FnOnce(&mut Session)) {
        let inner = inner(core);
        let out = {
            let mut s = inner.session.lock().unwrap_or_else(|e| e.into_inner());
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
            emit(core, &state, moved_only);
        }
    }

    fn no_route() -> String {
        crate::i18n::tr("core-cast-no-route", &[])
    }

    fn no_control() -> String {
        crate::i18n::tr("core-cast-no-control", &[])
    }

    fn not_handed(e: reqwest::Error) -> String {
        crate::i18n::tr("core-cast-not-handed", &[("error", e.to_string().into())])
    }

    fn fail(core: &Core, id: u64, error: String) {
        update(core, id, |s| {
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
            .map_err(|e| {
                crate::i18n::tr("core-cast-serve-failed", &[("error", e.to_string().into())])
            })
    }

    pub fn call(core: &Core, call: Call) -> Result<Reply, CoreError> {
        let inner = inner(core);
        match call {
            Call::CastDevices => {
                look(core);
                Ok(Reply::CastDevices {
                    devices: devices(&inner),
                })
            }
            Call::CastStatus => {
                let s = inner.session.lock().unwrap_or_else(|e| e.into_inner());
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
                let found = inner
                    .devices
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .iter()
                    .find(|f| f.device.id == device)
                    .cloned()
                    .ok_or_else(|| {
                        CoreError::invalid("device", crate::i18n::tr("core-cast-device-gone", &[]))
                    })?;
                let path = file_path(core, file)?;
                stop(core);
                let id = inner.next.fetch_add(1, Ordering::Relaxed) + 1;
                let token = format!(
                    "{:016x}{:016x}",
                    rand::random::<u64>(),
                    rand::random::<u64>()
                );
                let title = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let state = CastState {
                    device: found.device.name.clone(),
                    kind: found.device.kind,
                    file,
                    state: CastPlayState::Connecting,
                    position: start,
                    duration: 0.0,
                    error: None,
                };
                *inner.session.lock().unwrap_or_else(|e| e.into_inner()) = Some(Session {
                    id,
                    token: token.clone(),
                    state: state.clone(),
                    control: Control::Starting,
                    mode: Mode::Direct,
                    base: String::new(),
                    title: title.clone(),
                    prepared: None,
                    pieces: None,
                    hevc_doubt: false,
                });
                emit(core, &state, false);
                let subs = match subtitle {
                    CastSubtitle::None => None,
                    CastSubtitle::Track { number } => Some(Subs::Track(number)),
                    CastSubtitle::File { path } => Some(Subs::File(PathBuf::from(path))),
                };
                let arc = core
                    .arc()
                    .ok_or_else(|| CoreError::internal("the core is closing"))?;
                core.handle.spawn(async move {
                    if let Err(e) = begin(
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
                    {
                        fail(&arc, id, e);
                    }
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
                let found = inner
                    .devices
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .iter()
                    .find(|f| f.device.id == device)
                    .cloned()
                    .ok_or_else(|| {
                        CoreError::invalid("device", crate::i18n::tr("core-cast-device-gone", &[]))
                    })?;
                let Some((dir, name)) = url.split('?').next().and_then(|u| u.rsplit_once('/'))
                else {
                    return Err(CoreError::invalid(
                        "url",
                        crate::i18n::tr("core-cast-not-a-playlist", &[]),
                    ));
                };
                let dir = format!("{dir}/");
                let query = url.split_once('?').map(|(_, q)| q).unwrap_or_default();
                let playlist = if query.is_empty() {
                    name.to_string()
                } else {
                    format!("{name}?{query}")
                };
                stop(core);
                let id = inner.next.fetch_add(1, Ordering::Relaxed) + 1;
                let token = format!(
                    "{:016x}{:016x}",
                    rand::random::<u64>(),
                    rand::random::<u64>()
                );
                let state = CastState {
                    device: found.device.name.clone(),
                    kind: found.device.kind,
                    file: 0,
                    state: CastPlayState::Connecting,
                    position: start,
                    duration,
                    error: None,
                };
                *inner.session.lock().unwrap_or_else(|e| e.into_inner()) = Some(Session {
                    id,
                    token: token.clone(),
                    state: state.clone(),
                    control: Control::Starting,
                    mode: match found.device.kind {
                        CastKind::Chromecast => Mode::Hls,
                        CastKind::Dlna => Mode::Progressive { offset: start },
                    },
                    base: String::new(),
                    title: title.clone(),
                    prepared: None,
                    pieces: None,
                    hevc_doubt: false,
                });
                emit(core, &state, false);
                let arc = core
                    .arc()
                    .ok_or_else(|| CoreError::internal("the core is closing"))?;
                core.handle.spawn(async move {
                    let begun = match found.device.kind {
                        CastKind::Chromecast => {
                            relay(
                                &arc, id, found, dir, playlist, token, title, start, duration,
                            )
                            .await
                        }
                        CastKind::Dlna => {
                            relay_dlna(&arc, id, found, dir, playlist, token, title, start).await
                        }
                    };
                    if let Err(e) = begun {
                        fail(&arc, id, e);
                    }
                });
                Ok(Reply::CastState { state: Some(state) })
            }
            Call::CastPlay => control(core, |c, id| match c {
                Ctl::Cast(c) => c.send(castv2::Command::Play),
                Ctl::Dlna(d) => spawn_dlna(core, id, async move { d.play().await }),
            }),
            Call::CastPause => control(core, |c, id| match c {
                Ctl::Cast(c) => c.send(castv2::Command::Pause),
                Ctl::Dlna(d) => spawn_dlna(core, id, async move { d.pause().await }),
            }),
            Call::CastSeek { to } => seek(core, to),
            Call::CastStop => {
                stop(core);
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
    fn control(core: &Core, f: impl FnOnce(Ctl<'_>, u64)) -> Result<Reply, CoreError> {
        let inner = inner(core);
        let s = inner.session.lock().unwrap_or_else(|e| e.into_inner());
        let Some(session) = s.as_ref() else {
            return Err(CoreError::invalid(
                "cast",
                crate::i18n::tr("core-cast-nothing", &[]),
            ));
        };
        match &session.control {
            Control::Cast(c) => f(Ctl::Cast(c), session.id),
            Control::Dlna { client, .. } => f(Ctl::Dlna(client.clone()), session.id),
            Control::Starting => {
                return Err(CoreError::invalid(
                    "cast",
                    crate::i18n::tr("core-cast-still-starting", &[]),
                ));
            }
        }
        Ok(Reply::Ok)
    }

    fn spawn_dlna(
        core: &Core,
        id: u64,
        f: impl std::future::Future<Output = Result<(), String>> + Send + 'static,
    ) {
        let Some(arc) = core.arc() else { return };
        core.handle.spawn(async move {
            if let Err(e) = f.await {
                fail(&arc, id, e);
            }
        });
    }

    fn seek(core: &Core, to: f64) -> Result<Reply, CoreError> {
        let inner = inner(core);
        let (mode, base, title, id, prepared, pieces) = {
            let mut s = inner.session.lock().unwrap_or_else(|e| e.into_inner());
            let Some(s) = s.as_mut() else {
                return Err(CoreError::invalid(
                    "cast",
                    crate::i18n::tr("core-cast-nothing", &[]),
                ));
            };
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
        match mode {
            Mode::Progressive { .. } => {
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
                update(core, id, |s| s.mode = Mode::Progressive { offset });
                control(core, |c, id| {
                    if let Ctl::Dlna(d) = c {
                        let meta = dlna::didl(&url, kind, &title, None);
                        spawn_dlna(core, id, async move {
                            d.set_uri(&url, &meta).await?;
                            d.play().await
                        });
                    }
                })
            }
            _ => control(core, |c, id| match c {
                Ctl::Cast(c) => c.send(castv2::Command::Seek(to)),
                Ctl::Dlna(d) => spawn_dlna(core, id, async move { d.seek(to).await }),
            }),
        }
    }

    fn stop(core: &Core) {
        let inner = inner(core);
        let Some(session) = inner
            .session
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
        else {
            return;
        };
        inner.registry.remove(&session.token);
        let mut state = session.state.clone();
        if !matches!(state.state, CastPlayState::Ended | CastPlayState::Failed) {
            state.state = CastPlayState::Stopped;
            emit(core, &state, false);
        }
        match session.control {
            // Dropping the controller stops the media receiver on the TV.
            Control::Cast(c) => drop(c),
            Control::Dlna { client, poll } => {
                poll.abort();
                core.handle.spawn(async move {
                    let _ = client.stop().await;
                });
            }
            Control::Starting => {}
        }
    }

    /// Starts a relayed stream on a Chromecast: the playlist under this machine's own
    /// server, every request passed on to `dir`.
    #[allow(clippy::too_many_arguments)]
    async fn relay(
        core: &Arc<Core>,
        id: u64,
        found: Found,
        dir: String,
        playlist: String,
        token: String,
        title: String,
        start: f64,
        duration: f64,
    ) -> Result<(), String> {
        let inner = inner(core);
        let port = server(&inner).await?.port();
        let ip = discover::local_ip_for(found.addr.ip()).ok_or_else(no_route)?;
        let host = discover::dev()
            .advertise
            .unwrap_or_else(|| format!("{ip}:{port}"));
        let base = format!("http://{host}/c/{token}");
        inner.registry.add(
            token.clone(),
            Served {
                path: PathBuf::new(),
                mime: "application/vnd.apple.mpegurl".to_string(),
                prepared: None,
                relay: Some(Relay {
                    dir,
                    pieces: Arc::new(Vec::new()),
                }),
            },
        );
        update(core, id, |s| s.base = base.clone());
        let load = castv2::Load {
            url: format!("{base}/{playlist}"),
            content_type: "application/x-mpegurl".to_string(),
            hls_fmp4: false,
            start,
            duration,
            title,
            subtitles: None,
        };
        let status_core = core.clone();
        let controller = castv2::start(found.addr, load, move |st| {
            on_cast_status(&status_core, id, st)
        });
        let mut s = inner.session.lock().unwrap_or_else(|e| e.into_inner());
        match s.as_mut().filter(|s| s.id == id) {
            Some(s) => s.control = Control::Cast(controller),
            None => drop(controller),
        }
        Ok(())
    }

    /// Starts a stream another machine converts on a TV over DLNA. Such a TV reads no HLS,
    /// so this machine reads the playlist once and hands the TV its pieces from the one
    /// holding `start` as one progressive MPEG-TS stream, fetching each from `dir` as the
    /// TV reads. A seek starts that stream again (`seek`).
    #[allow(clippy::too_many_arguments)]
    async fn relay_dlna(
        core: &Arc<Core>,
        id: u64,
        found: Found,
        dir: String,
        playlist: String,
        token: String,
        title: String,
        start: f64,
    ) -> Result<(), String> {
        let inner = inner(core);
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
        // cast stopped or replaced meanwhile is left alone, and so is the TV.
        if !current(&inner, id) {
            return Ok(());
        }
        let pieces = Arc::new(served::read_pieces(&list));
        if pieces.is_empty() {
            return Err(crate::i18n::tr("core-cast-empty-playlist", &[]));
        }
        let port = server(&inner).await?.port();
        let ip = discover::local_ip_for(found.addr.ip()).ok_or_else(no_route)?;
        let host = discover::dev()
            .advertise
            .unwrap_or_else(|| format!("{ip}:{port}"));
        let base = format!("http://{host}/c/{token}");
        inner.registry.add(
            token,
            Served {
                path: PathBuf::new(),
                mime: "video/mp2t".to_string(),
                prepared: None,
                relay: Some(Relay {
                    dir,
                    pieces: pieces.clone(),
                }),
            },
        );
        let offset = pieces
            .get(served::piece_at(&pieces, start))
            .map_or(0.0, |p| p.start);
        update(core, id, |s| {
            s.base = base.clone();
            s.pieces = Some(pieces.clone());
            s.mode = Mode::Progressive { offset };
        });
        let url = format!("{base}/stream.ts?t={start:.3}");
        let meta = dlna::didl(&url, "video/mp2t", &title, None);
        client.set_uri(&url, &meta).await?;
        client.play().await?;
        let poll_core = core.clone();
        let poll_client = client.clone();
        let poll = core
            .handle
            .spawn(async move { poll_dlna(poll_core, id, poll_client).await })
            .abort_handle();
        let mut s = inner.session.lock().unwrap_or_else(|e| e.into_inner());
        match s.as_mut().filter(|s| s.id == id) {
            Some(s) => s.control = Control::Dlna { client, poll },
            None => {
                poll.abort();
                drop(s);
                core.handle.spawn(async move {
                    let _ = client.stop().await;
                });
            }
        }
        Ok(())
    }

    /// Whether session `id` is still the one casting.
    fn current(inner: &Inner, id: u64) -> bool {
        inner
            .session
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .is_some_and(|s| s.id == id)
    }

    #[allow(clippy::too_many_arguments)]
    async fn begin(
        core: &Arc<Core>,
        id: u64,
        found: Found,
        path: PathBuf,
        token: String,
        title: String,
        start: f64,
        audio_track: Option<u64>,
        subs: Option<Subs>,
    ) -> Result<(), String> {
        let inner = inner(core);
        let port = server(&inner).await?.port();
        let peer: IpAddr = found.addr.ip();
        let ip = discover::local_ip_for(peer).ok_or_else(no_route)?;
        let host = discover::dev()
            .advertise
            .unwrap_or_else(|| format!("{ip}:{port}"));
        let base = format!("http://{host}/c/{token}");
        let mkv = is_matroska(&path);
        let receiver = match found.device.kind {
            CastKind::Chromecast => Receiver::Cast,
            CastKind::Dlna => Receiver::Dlna,
        };
        // A Matroska file is read through once: keyframes for HLS, subtitles for both.
        let prepared = if mkv && (receiver == Receiver::Cast || subs.is_some()) {
            let p = path.clone();
            let s = subs.clone();
            let prepared = tokio::task::spawn_blocking(move || Prepared::new(&p, audio_track, s))
                .await
                .map_err(|e| e.to_string())??;
            Some(Arc::new(prepared))
        } else {
            None
        };
        let hevc_doubt = found.device.kind == CastKind::Chromecast
            && prepared
                .as_ref()
                .and_then(|p| p.probe.video.as_ref())
                .is_some_and(|v| matches!(v.codec, super::mkv::VideoCodec::Hevc { .. }))
            && !discover::decodes_hevc(found.device.model.as_deref());
        inner.registry.add(
            token.clone(),
            Served {
                path: path.clone(),
                mime: mime(&path).to_string(),
                prepared: prepared.clone(),
                relay: None,
            },
        );
        let duration = prepared
            .as_ref()
            .map(|p| p.duration_ms() as f64 / 1000.0)
            .unwrap_or(0.0);
        update(core, id, |s| {
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
                    subtitles: None,
                };
                let status_core = core.clone();
                let controller = castv2::start(found.addr, load, move |st| {
                    on_cast_status(&status_core, id, st)
                });
                update(core, id, |s| {
                    s.mode = if hls { Mode::Hls } else { Mode::Direct }
                });
                let mut s = inner.session.lock().unwrap_or_else(|e| e.into_inner());
                match s.as_mut().filter(|s| s.id == id) {
                    Some(s) => s.control = Control::Cast(controller),
                    // Stopped while connecting: the controller drops here and stops the TV.
                    None => drop(controller),
                }
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
                        let p = match prepared.clone() {
                            Some(p) => p,
                            None => {
                                let pp = path.clone();
                                let p = tokio::task::spawn_blocking(move || {
                                    Prepared::new(&pp, audio_track, None)
                                })
                                .await
                                .map_err(|e| e.to_string())??;
                                let p = Arc::new(p);
                                inner.registry.add(
                                    token.clone(),
                                    Served {
                                        path: path.clone(),
                                        mime: mime(&path).to_string(),
                                        prepared: Some(p.clone()),
                                        relay: None,
                                    },
                                );
                                p
                            }
                        };
                        let url = format!("{base}/stream.mp4?t={start:.3}");
                        let meta = dlna::didl(&url, "video/mp4", &title, srt.as_deref());
                        client.set_uri(&url, &meta).await.map_err(|e| {
                            crate::i18n::tr(
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
                let poll_core = core.clone();
                let poll_client = client.clone();
                let poll = core
                    .handle
                    .spawn(async move { poll_dlna(poll_core, id, poll_client).await })
                    .abort_handle();
                let mut s = inner.session.lock().unwrap_or_else(|e| e.into_inner());
                match s.as_mut().filter(|s| s.id == id) {
                    Some(s) => {
                        s.mode = mode;
                        s.control = Control::Dlna { client, poll };
                    }
                    None => poll.abort(),
                }
            }
        }
        Ok(())
    }

    fn on_cast_status(core: &Core, id: u64, st: castv2::Status) {
        update(core, id, |s| {
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
                    .unwrap_or_else(|| crate::i18n::tr("core-cast-chromecast-stopped", &[]));
                if s.hevc_doubt {
                    e = crate::i18n::tr("core-cast-hevc", &[("error", e.into())]);
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

    async fn poll_dlna(core: Arc<Core>, id: u64, client: Arc<dlna::Client>) {
        let mut seen_playing = false;
        let mut misses = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            match client.status().await {
                Ok((state, at, duration)) => {
                    misses = 0;
                    let mut finished = false;
                    update(&core, id, |s| {
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
                        fail(&core, id, e);
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
        use crate::CorePaths;
        use crate::trackers::Secrets;

        /// What the fake TV was told and what it read.
        #[derive(Default)]
        struct Tv {
            uris: Vec<String>,
            meta: Vec<String>,
            read: Vec<Vec<u8>>,
            /// Every action but the two status reads, in order.
            actions: Vec<String>,
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

        async fn tv(State(tv): State<Arc<Mutex<Tv>>>, headers: HeaderMap, body: Bytes) -> String {
            let action = headers
                .get("SOAPAction")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string();
            let xml = String::from_utf8_lossy(&body).into_owned();
            if !action.contains("#Get") {
                tv.lock().unwrap().actions.push(action.clone());
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
                return "<r><CurrentTransportState>PLAYING</CurrentTransportState></r>".into();
            } else if action.contains("#GetPositionInfo") {
                // Counted from the start of the stream it was handed, as many TVs do.
                return "<r><RelTime>0:00:02</RelTime><TrackDuration>0:00:00</TrackDuration></r>"
                    .into();
            }
            "<ok/>".into()
        }

        async fn computer(
            State(asked): State<Arc<Mutex<Vec<String>>>>,
            Path(name): Path<String>,
            uri: Uri,
        ) -> Vec<u8> {
            asked.lock().unwrap().push(uri.to_string());
            if name == "index.m3u8" {
                return crate::host::convert::playlist(18.0, "?burn=0").into_bytes();
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

        /// A fake TV and a core that knows it, on loopback, for the tests below.
        fn tv_and_core(
            rt: &tokio::runtime::Runtime,
        ) -> (Arc<Mutex<Tv>>, tempfile::TempDir, Arc<crate::Core>) {
            let tv_state = Arc::new(Mutex::new(Tv::default()));
            let tv_at = rt.block_on(async {
                let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                let at = l.local_addr().unwrap();
                let app = Router::new()
                    .route("/ctl", post(tv))
                    .with_state(tv_state.clone());
                tokio::spawn(async move { axum::serve(l, app).await });
                at
            });
            let dir = tempfile::tempdir().unwrap();
            let paths = CorePaths::under(dir.path());
            let secrets = Secrets::file_only(paths.secrets_path());
            let core = crate::Core::open_with_secrets(paths, secrets).unwrap();
            inner(&core).devices.lock().unwrap().push(Found {
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
            (tv_state, dir, core)
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
            let (tv_state, _dir, core) = tv_and_core(&rt);
            // The other computer takes its time over the playlist, as it does
            // while it starts converting.
            let slow = Router::new().route(
                "/hls/{name}",
                get(|| async {
                    tokio::time::sleep(Duration::from_millis(800)).await;
                    crate::host::convert::playlist(18.0, "")
                }),
            );
            let computer_at = serve(&rt, slow);
            core.call(Call::CastStartStream {
                device: "dlna:fake".into(),
                url: format!("http://{computer_at}/hls/index.m3u8"),
                title: "Show 1".into(),
                start: 0.0,
                duration: 18.0,
            })
            .unwrap();
            core.call(Call::CastStop).unwrap();
            std::thread::sleep(Duration::from_millis(1500));
            let t = tv_state.lock().unwrap();
            assert!(t.uris.is_empty(), "the TV was handed {:?}", t.uris);
            assert!(t.actions.is_empty(), "the TV was told {:?}", t.actions);
        }

        #[test]
        fn a_redirect_from_the_other_computer_is_not_followed() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (tv_state, _dir, core) = tv_and_core(&rt);
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
                            crate::host::convert::playlist(12.0, "").into_bytes(),
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
            core.call(Call::CastStartStream {
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
            let tv_state = Arc::new(Mutex::new(Tv::default()));
            let asked = Arc::new(Mutex::new(Vec::new()));
            let (tv_at, computer_at) = rt.block_on(async {
                let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                let tv_at = l.local_addr().unwrap();
                let app = Router::new()
                    .route("/ctl", post(tv))
                    .with_state(tv_state.clone());
                tokio::spawn(async move { axum::serve(l, app).await });
                let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                let computer_at = l.local_addr().unwrap();
                let app = Router::new()
                    .route("/hls/{name}", get(computer))
                    .with_state(asked.clone());
                tokio::spawn(async move { axum::serve(l, app).await });
                (tv_at, computer_at)
            });

            let dir = tempfile::tempdir().unwrap();
            let paths = CorePaths::under(dir.path());
            let secrets = Secrets::file_only(paths.secrets_path());
            let core = crate::Core::open_with_secrets(paths, secrets).unwrap();
            inner(&core).devices.lock().unwrap().push(Found {
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

            core.call(Call::CastStartStream {
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
                    core.call(Call::CastStatus),
                    Ok(Reply::CastState { state: Some(s) }) if (s.position - 8.0).abs() < 0.01
                )
            });

            core.call(Call::CastSeek { to: 13.5 }).unwrap();
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
                    core.call(Call::CastStatus),
                    Ok(Reply::CastState { state: Some(s) }) if (s.position - 14.0).abs() < 0.01
                )
            });
            core.call(Call::CastStop).unwrap();
        }
    }
}
