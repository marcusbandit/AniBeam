//! Episodes from the other device, played here: the host serves the file, or the same
//! converted when the file is too big for the link, and its core keeps the session, so
//! the place, the history and the tracker move on there as if it had played there.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anibeam_companion::address::HostAddress;
use anibeam_companion::api::{self, Probe};
use anibeam_companion::client::{self, HostClient};
use anibeam_companion::store::Store;
use anibeam_core::{Call, CoreError, PlaybackSession, Reply, Sidecar};
use serde::{Deserialize, Serialize};

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Called once at startup, so a stream can read the host's address.
pub fn init(data_dir: PathBuf) {
    let _ = DATA_DIR.set(data_dir);
}

/// The quality picked in More.
pub fn quality() -> Quality {
    DATA_DIR.get().map(|d| crate::prefs::load(d).stream_quality).unwrap_or_default()
}

/// How a stream from the other device plays. Auto plays the file as it is on Wi-Fi and a
/// 720p conversion on mobile data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quality {
    #[default]
    Auto,
    Original,
    P1080,
    P720,
    P480,
}

impl Quality {
    pub const ALL: [Quality; 5] = [Quality::Auto, Quality::Original, Quality::P1080, Quality::P720, Quality::P480];

    pub fn label(self) -> &'static str {
        match self {
            Quality::Auto => "Auto",
            Quality::Original => "Original",
            Quality::P1080 => "1080p",
            Quality::P720 => "720p",
            Quality::P480 => "480p",
        }
    }
}

/// A file bigger than this a second plays converted even on Wi-Fi under Auto, since a
/// tailnet relay rarely carries more.
const AUTO_WIFI_LIMIT: u64 = 25_000_000;
/// Auto on mobile data.
const AUTO_MOBILE_HEIGHT: u32 = 720;

/// The height to convert to, or None for the file as it is.
pub fn height(quality: Quality, probe: Option<&Probe>, unmetered: bool, can_convert: bool) -> Option<u32> {
    if !can_convert {
        return None;
    }
    let source = probe.and_then(|p| p.height).unwrap_or(u32::MAX);
    let wanted = match quality {
        Quality::Original => return None,
        Quality::P1080 => 1080,
        Quality::P720 => 720,
        Quality::P480 => 480,
        Quality::Auto if !unmetered => AUTO_MOBILE_HEIGHT,
        Quality::Auto => {
            let heavy = probe.is_some_and(|p| p.bitrate > AUTO_WIFI_LIMIT);
            if !heavy {
                return None;
            }
            1080
        }
    };
    // A conversion no smaller than the file gains nothing but lost quality, unless the
    // file is too heavy for the link anyway.
    (wanted < source || probe.is_some_and(|p| p.bitrate > AUTO_WIFI_LIMIT)).then_some(wanted)
}

/// The device a stream plays from.
pub struct Host {
    client: Arc<HostClient>,
    /// The calls sent without waiting, in order, through one thread.
    queue: std::sync::Mutex<Option<std::sync::mpsc::Sender<Call>>>,
    /// `http://100.x.y.z:8790`, the name already looked up.
    base: String,
    pub can_convert: bool,
    pub name: String,
}

impl Host {
    /// Reaches the device set in More, checks it streams, and looks its name up.
    pub fn connect() -> Result<Host, String> {
        let dir = DATA_DIR.get().ok_or("The app is still starting")?;
        let store = Store::open(dir).map_err(|e| e.to_string())?;
        let address = HostAddress::parse(&store.settings().address)?;
        let client = HostClient::new(client::agent(), address);
        let hello = client.hello().map_err(|e| e.to_string())?;
        if !hello.features.iter().any(|f| f == api::FEATURE_STREAM) {
            return Err(format!("{} runs AniBeam {}, which cannot stream yet. Update it there.", hello.host, hello.version));
        }
        let base = client.stream_base().map_err(|e| e.to_string())?;
        Ok(Host {
            can_convert: hello.features.iter().any(|f| f == api::FEATURE_CONVERT),
            name: hello.host,
            client: Arc::new(client),
            queue: std::sync::Mutex::new(None),
            base,
        })
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    pub fn probe(&self, file: u64) -> Option<Probe> {
        self.client.probe(file).ok()
    }

    /// One playback call, run on the host's core. Blocking.
    pub fn call(&self, call: &Call) -> Result<Reply, CoreError> {
        call_on(&self.client, &self.name, call)
    }

    /// Sends a call without waiting for it. Every call sent this way reaches the host in
    /// the order it was sent, so a tick never lands after its session's close.
    pub fn send(&self, call: Call) {
        let mut queue = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        let tx = queue.get_or_insert_with(|| {
            let (tx, rx) = std::sync::mpsc::channel::<Call>();
            let (client, name) = (self.client.clone(), self.name.clone());
            std::thread::spawn(move || {
                for call in rx {
                    if let Err(e) = call_on(&client, &name, &call) {
                        crate::local::log(&format!("player: the other device refused a call: {e}"));
                    }
                }
            });
            tx
        });
        let _ = tx.send(call);
    }

    /// The skip windows the host has found for its session so far. Blocking.
    pub fn skip_windows(&self, session: u64) -> Option<Vec<anibeam_core::SkipWindow>> {
        let body = self.client.get_bytes(&api::skip_path(session)).ok()?;
        serde_json::from_slice(&body).ok()
    }
}

pub(crate) fn call_on(client: &HostClient, name: &str, call: &Call) -> Result<Reply, CoreError> {
    let body = serde_json::to_vec(call).map_err(|e| CoreError::internal(e.to_string()))?;
    let (status, answer) = client.post(api::CALL, &body).map_err(|e| CoreError::internal(e.to_string()))?;
    if status == 200 {
        return serde_json::from_slice(&answer).map_err(|e| CoreError::internal(format!("The answer did not read: {e}")));
    }
    #[derive(Deserialize)]
    struct Refused {
        error: String,
        #[serde(default)]
        core: Option<CoreError>,
    }
    match serde_json::from_slice::<Refused>(&answer) {
        Ok(Refused { core: Some(e), .. }) => Err(e),
        Ok(Refused { error, .. }) => Err(CoreError::internal(error)),
        Err(_) => Err(CoreError::internal(format!("{name} answered {status}"))),
    }
}

/// The file's audio track the series was last watched with, by language and then title,
/// as an index of the probe's audio tracks.
fn saved_audio(session: &PlaybackSession, probe: &Probe) -> Option<usize> {
    let want = session.track_choice.audio.as_ref()?;
    let lang = |t: &anibeam_companion::api::Track| t.language.as_deref() == want.language.as_deref();
    let title = |t: &anibeam_companion::api::Track| t.title.as_deref() == want.title.as_deref();
    probe.audio.iter().position(|t| lang(t) && title(t)).or_else(|| probe.audio.iter().position(lang))
}

/// Pictures, not text: these cannot be read out as a subtitle file.
fn is_bitmap(codec: &str) -> bool {
    matches!(codec, "hdmv_pgs_subtitle" | "dvd_subtitle" | "dvb_subtitle" | "xsub")
}

/// A session opened on the host, ready for the player: `path` is what mpv loads, and the
/// sidecars are addresses too.
pub struct Opened {
    pub session: PlaybackSession,
    pub host: Arc<Host>,
    /// Set when the stream is a conversion, for the page to say so.
    pub converted: Option<u32>,
}

/// Opens `file` of the host on the host's core and picks how it streams. Blocking: run it
/// off the UI thread.
pub fn open(file: u64, quality: Quality, unmetered: bool) -> Result<Opened, String> {
    let host = Arc::new(Host::connect()?);
    let mut session = match host.call(&Call::OpenPlayback { file }) {
        Ok(Reply::Playback { session }) => *session,
        Ok(_) => return Err("The other device gave no session".into()),
        Err(e) => return Err(e.to_string()),
    };
    let probe = host.can_convert.then(|| host.probe(file)).flatten();
    let converted = height(quality, probe.as_ref(), unmetered, host.can_convert);
    session.path = match converted {
        // A conversion carries one audio track, so it is the one the series was last
        // watched with; the subtitles come beside it, below.
        Some(h) => {
            let audio = probe.as_ref().and_then(|p| saved_audio(&session, p));
            host.url(&api::hls_path(file, h, audio, None))
        }
        None => host.url(&api::file_path(file)),
    };
    for (i, s) in session.sidecars.iter_mut().enumerate() {
        s.path = host.url(&api::sidecar_path(file, i));
    }
    // A conversion carries no subtitles, so the file's own text tracks come beside it.
    if converted.is_some()
        && let Some(p) = &probe
    {
        let own = p.subtitles.iter().enumerate().filter(|(_, t)| !is_bitmap(&t.codec));
        let tracks: Vec<Sidecar> = own
            .map(|(i, t)| Sidecar { path: host.url(&api::subtitle_path(file, i)), language: t.language.clone(), title: t.title.clone() })
            .collect();
        session.sidecars.splice(0..0, tracks);
    }
    Ok(Opened { session, host, converted })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(height: u32, bitrate: u64) -> Probe {
        Probe { height: Some(height), bitrate, ..Default::default() }
    }

    #[test]
    fn auto_plays_the_file_on_wifi_and_720p_on_mobile_data() {
        let p = probe(1080, 4_000_000);
        assert_eq!(height(Quality::Auto, Some(&p), true, true), None);
        assert_eq!(height(Quality::Auto, Some(&p), false, true), Some(720));
        let heavy = probe(1080, 40_000_000);
        assert_eq!(height(Quality::Auto, Some(&heavy), true, true), Some(1080));
    }

    #[test]
    fn a_pick_never_converts_up_and_nothing_converts_without_a_converter() {
        let p = probe(720, 3_000_000);
        assert_eq!(height(Quality::P1080, Some(&p), true, true), None);
        assert_eq!(height(Quality::P480, Some(&p), true, true), Some(480));
        assert_eq!(height(Quality::P480, Some(&p), true, false), None);
        assert_eq!(height(Quality::Original, Some(&p), false, true), None);
    }
}
