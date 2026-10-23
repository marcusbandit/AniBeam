//! Google Cast's sender protocol, the part a remote needs: TLS to port 8009, length-prefixed
//! protobuf frames carrying JSON, the Default Media Receiver launched on the device, then
//! LOAD, PLAY, PAUSE, SEEK and the media status read back once a second. One thread owns
//! the connection, so there is never a reply raced for by two readers.

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const NS_CONNECTION: &str = "urn:x-cast:com.google.cast.tp.connection";
const NS_HEARTBEAT: &str = "urn:x-cast:com.google.cast.tp.heartbeat";
const NS_RECEIVER: &str = "urn:x-cast:com.google.cast.receiver";
const NS_MEDIA: &str = "urn:x-cast:com.google.cast.media";
const DEFAULT_MEDIA_RECEIVER: &str = "CC1AD845";
const SENDER: &str = "sender-anibeam";

/// One frame: the protobuf CastMessage with a string payload.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub source: String,
    pub destination: String,
    pub namespace: String,
    pub payload: String,
}

fn varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

fn field_str(out: &mut Vec<u8>, field: u64, s: &str) {
    varint(out, (field << 3) | 2);
    varint(out, s.len() as u64);
    out.extend_from_slice(s.as_bytes());
}

pub fn encode(m: &Message) -> Vec<u8> {
    let mut body = Vec::new();
    varint(&mut body, 1 << 3); // protocol_version
    varint(&mut body, 0); // CASTV2_1_0
    field_str(&mut body, 2, &m.source);
    field_str(&mut body, 3, &m.destination);
    field_str(&mut body, 4, &m.namespace);
    varint(&mut body, 5 << 3); // payload_type
    varint(&mut body, 0); // STRING
    field_str(&mut body, 6, &m.payload);
    let mut out = (body.len() as u32).to_be_bytes().to_vec();
    out.extend_from_slice(&body);
    out
}

fn read_varint(b: &[u8], i: &mut usize) -> Option<u64> {
    let mut v = 0u64;
    for shift in (0..64).step_by(7) {
        let byte = *b.get(*i)?;
        *i += 1;
        v |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some(v);
        }
    }
    None
}

/// A frame's body, without the length prefix. Binary payloads (device auth) are ignored.
pub fn decode(body: &[u8]) -> Option<Message> {
    let mut m = Message {
        source: String::new(),
        destination: String::new(),
        namespace: String::new(),
        payload: String::new(),
    };
    let mut i = 0;
    while i < body.len() {
        let key = read_varint(body, &mut i)?;
        match key & 7 {
            0 => {
                read_varint(body, &mut i)?;
            }
            2 => {
                let len = read_varint(body, &mut i)? as usize;
                let bytes = body.get(i..i + len)?;
                i += len;
                let s = || String::from_utf8_lossy(bytes).into_owned();
                match key >> 3 {
                    2 => m.source = s(),
                    3 => m.destination = s(),
                    4 => m.namespace = s(),
                    6 => m.payload = s(),
                    _ => {}
                }
            }
            _ => return None,
        }
    }
    Some(m)
}

/// Chromecasts present a certificate of their own making; the network is the trust here,
/// the same as every Cast sender.
#[derive(Debug)]
struct AnyCert(Arc<rustls::crypto::CryptoProvider>);

impl rustls::client::danger::ServerCertVerifier for AnyCert {
    fn verify_server_cert(
        &self,
        _: &rustls::pki_types::CertificateDer<'_>,
        _: &[rustls::pki_types::CertificateDer<'_>],
        _: &rustls::pki_types::ServerName<'_>,
        _: &[u8],
        _: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

type Tls = rustls::StreamOwned<rustls::ClientConnection, TcpStream>;

fn connect(addr: SocketAddr) -> io::Result<Tls> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(io::Error::other)?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AnyCert(provider)))
        .with_no_client_auth();
    let name = rustls::pki_types::ServerName::IpAddress(addr.ip().into());
    let conn = rustls::ClientConnection::new(Arc::new(config), name).map_err(io::Error::other)?;
    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
    tcp.set_nodelay(true)?;
    Ok(rustls::StreamOwned::new(conn, tcp))
}

/// What to play: the media URL, its type, where to start, and a WebVTT URL to load as a
/// side track when the media is not HLS (HLS carries its subtitles in the playlist).
#[derive(Clone, Debug)]
pub struct Load {
    pub url: String,
    pub content_type: String,
    pub hls_fmp4: bool,
    pub start: f64,
    pub duration: f64,
    pub title: String,
    pub subtitles: Option<String>,
}

/// The receiver's state, as its media status said.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    /// PLAYING, PAUSED, BUFFERING, IDLE, or LOADING before the first status.
    pub player: String,
    pub position: f64,
    pub duration: Option<f64>,
    /// FINISHED, CANCELLED, INTERRUPTED or ERROR once IDLE.
    pub idle_reason: Option<String>,
    pub error: Option<String>,
}

pub enum Command {
    Play,
    Pause,
    Seek(f64),
    Stop,
}

pub struct Controller {
    tx: Sender<Command>,
}

impl Controller {
    pub fn send(&self, c: Command) {
        let _ = self.tx.send(c);
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Stop);
    }
}

/// Connects, launches the media receiver, loads, and reports every status to `on_status`
/// until the session ends. The thread stops when the controller is dropped.
pub fn start(
    addr: SocketAddr,
    load: Load,
    on_status: impl Fn(Status) + Send + 'static,
) -> Controller {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("cast-chromecast".into())
        .spawn(move || {
            if let Err(e) = run(addr, &load, &rx, &on_status) {
                on_status(Status {
                    player: "IDLE".into(),
                    position: 0.0,
                    duration: None,
                    idle_reason: Some("ERROR".into()),
                    error: Some(e),
                });
            }
        })
        .expect("a thread for the Chromecast");
    Controller { tx }
}

struct Link {
    tls: Tls,
    buf: Vec<u8>,
    request: u64,
}

impl Link {
    fn send(&mut self, destination: &str, namespace: &str, payload: Value) -> io::Result<()> {
        let m = Message {
            source: SENDER.into(),
            destination: destination.into(),
            namespace: namespace.into(),
            payload: payload.to_string(),
        };
        self.tls.write_all(&encode(&m))?;
        self.tls.flush()
    }

    fn next_id(&mut self) -> u64 {
        self.request += 1;
        self.request
    }

    /// The next whole frame, or None when nothing came within the socket's timeout.
    fn read(&mut self) -> io::Result<Option<Message>> {
        loop {
            if self.buf.len() >= 4 {
                let len = u32::from_be_bytes(self.buf[..4].try_into().unwrap()) as usize;
                if len > 1 << 20 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "a frame too large to be Cast",
                    ));
                }
                if self.buf.len() >= 4 + len {
                    let body: Vec<u8> = self.buf.drain(..4 + len).skip(4).collect();
                    return Ok(decode(&body));
                }
            }
            let mut chunk = [0u8; 16 * 1024];
            match self.tls.read(&mut chunk) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "the device closed the connection",
                    ));
                }
                Ok(n) => self.buf.extend_from_slice(&chunk[..n]),
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    return Ok(None);
                }
                Err(e) => return Err(e),
            }
        }
    }
}

fn media_info(load: &Load) -> Value {
    let mut media = json!({
        "contentId": load.url,
        "contentUrl": load.url,
        "contentType": load.content_type,
        "streamType": "BUFFERED",
        "metadata": { "metadataType": 0, "title": load.title },
    });
    if load.duration > 0.0 {
        media["duration"] = json!(load.duration);
    }
    if load.hls_fmp4 {
        media["hlsSegmentFormat"] = json!("fmp4");
        media["hlsVideoSegmentFormat"] = json!("fmp4");
    }
    if let Some(vtt) = &load.subtitles {
        media["tracks"] = json!([{
            "trackId": 1, "type": "TEXT", "subtype": "SUBTITLES",
            "trackContentId": vtt, "trackContentType": "text/vtt", "name": "Subtitles", "language": "und",
        }]);
    }
    media
}

fn run(
    addr: SocketAddr,
    load: &Load,
    rx: &Receiver<Command>,
    on_status: &impl Fn(Status),
) -> Result<(), String> {
    let tls = connect(addr).map_err(|e| format!("could not reach the Chromecast: {e}"))?;
    tls.sock
        .set_read_timeout(Some(Duration::from_millis(200)))
        .map_err(|e| e.to_string())?;
    let mut link = Link {
        tls,
        buf: Vec::new(),
        request: 0,
    };
    let io = |e: io::Error| format!("the Chromecast connection broke: {e}");
    link.send("receiver-0", NS_CONNECTION, json!({"type": "CONNECT"}))
        .map_err(io)?;
    let id = link.next_id();
    link.send(
        "receiver-0",
        NS_RECEIVER,
        json!({"type": "LAUNCH", "appId": DEFAULT_MEDIA_RECEIVER, "requestId": id}),
    )
    .map_err(io)?;

    let mut transport: Option<String> = None;
    let mut session_id: Option<String> = None;
    let mut media_session: Option<i64> = None;
    let mut loaded = false;
    let mut text_tracks_set = load.subtitles.is_some();
    let mut last_poll = Instant::now();
    let mut last_ping = Instant::now();
    let launched_at = Instant::now();
    let mut ended = false;

    loop {
        // Commands first, so a pause never waits behind a status read.
        loop {
            match rx.recv_timeout(Duration::ZERO) {
                Ok(cmd) => {
                    let (Some(t), Some(ms)) = (transport.clone(), media_session) else {
                        if matches!(cmd, Command::Stop) {
                            stop(&mut link, session_id.as_deref());
                            return Ok(());
                        }
                        continue;
                    };
                    let id = link.next_id();
                    let msg = match cmd {
                        Command::Play => {
                            json!({"type": "PLAY", "mediaSessionId": ms, "requestId": id})
                        }
                        Command::Pause => {
                            json!({"type": "PAUSE", "mediaSessionId": ms, "requestId": id})
                        }
                        Command::Seek(to) => {
                            json!({"type": "SEEK", "mediaSessionId": ms, "currentTime": to, "requestId": id})
                        }
                        Command::Stop => {
                            stop(&mut link, session_id.as_deref());
                            return Ok(());
                        }
                    };
                    link.send(&t, NS_MEDIA, msg).map_err(io)?;
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => {
                    stop(&mut link, session_id.as_deref());
                    return Ok(());
                }
            }
        }

        if last_ping.elapsed() > Duration::from_secs(5) {
            link.send("receiver-0", NS_HEARTBEAT, json!({"type": "PING"}))
                .map_err(io)?;
            last_ping = Instant::now();
        }
        if let (Some(t), Some(ms)) = (&transport, media_session)
            && last_poll.elapsed() > Duration::from_secs(1)
            && !ended
        {
            let t = t.clone();
            let id = link.next_id();
            link.send(
                &t,
                NS_MEDIA,
                json!({"type": "GET_STATUS", "mediaSessionId": ms, "requestId": id}),
            )
            .map_err(io)?;
            last_poll = Instant::now();
        }
        if transport.is_none() && launched_at.elapsed() > Duration::from_secs(20) {
            return Err("the Chromecast did not start its media player".into());
        }

        let Some(m) = link.read().map_err(io)? else {
            continue;
        };
        let Ok(p) = serde_json::from_str::<Value>(&m.payload) else {
            continue;
        };
        let kind = p["type"].as_str().unwrap_or_default();
        match (m.namespace.as_str(), kind) {
            (NS_HEARTBEAT, "PING") => {
                link.send(&m.source, NS_HEARTBEAT, json!({"type": "PONG"}))
                    .map_err(io)?;
            }
            (NS_RECEIVER, "RECEIVER_STATUS") => {
                let app = p["status"]["applications"]
                    .as_array()
                    .and_then(|a| a.iter().find(|x| x["appId"] == DEFAULT_MEDIA_RECEIVER));
                match app {
                    Some(app) if transport.is_none() => {
                        let t = app["transportId"].as_str().unwrap_or_default().to_string();
                        session_id = app["sessionId"].as_str().map(str::to_string);
                        link.send(&t, NS_CONNECTION, json!({"type": "CONNECT"}))
                            .map_err(io)?;
                        let id = link.next_id();
                        let mut req = json!({
                            "type": "LOAD", "requestId": id, "sessionId": session_id,
                            "media": media_info(load), "autoplay": true, "currentTime": load.start,
                        });
                        if load.subtitles.is_some() {
                            req["activeTrackIds"] = json!([1]);
                        }
                        link.send(&t, NS_MEDIA, req).map_err(io)?;
                        transport = Some(t);
                        loaded = true;
                    }
                    // Someone else took the TV over: the media receiver is gone.
                    None if loaded => {
                        on_status(Status {
                            player: "IDLE".into(),
                            position: 0.0,
                            duration: None,
                            idle_reason: Some("CANCELLED".into()),
                            error: None,
                        });
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (NS_RECEIVER, "LAUNCH_ERROR") => {
                return Err(format!(
                    "the Chromecast would not start its media player ({})",
                    p["reason"].as_str().unwrap_or("no reason")
                ));
            }
            (NS_MEDIA, "LOAD_FAILED")
            | (NS_MEDIA, "INVALID_REQUEST")
            | (NS_MEDIA, "LOAD_CANCELLED") => {
                return Err(format!("the Chromecast could not play this ({kind})"));
            }
            (NS_MEDIA, "MEDIA_STATUS") => {
                let Some(s) = p["status"].as_array().and_then(|a| a.first()) else {
                    continue;
                };
                if let Some(ms) = s["mediaSessionId"].as_i64() {
                    media_session = Some(ms);
                }
                // An HLS stream's text track comes from its playlist; turn the first on.
                if !text_tracks_set
                    && let (Some(t), Some(ms)) = (&transport, media_session)
                    && let Some(id) = s["media"]["tracks"]
                        .as_array()
                        .and_then(|tr| tr.iter().find(|x| x["type"] == "TEXT"))
                        .and_then(|x| x["trackId"].as_i64())
                {
                    let t = t.clone();
                    let rid = link.next_id();
                    link.send(&t, NS_MEDIA, json!({"type": "EDIT_TRACKS_INFO", "mediaSessionId": ms, "activeTrackIds": [id], "requestId": rid}))
                        .map_err(io)?;
                    text_tracks_set = true;
                }
                let status = parse_status(s);
                ended = status.player == "IDLE" && status.idle_reason.is_some();
                let finished = ended;
                on_status(status);
                if finished {
                    return Ok(());
                }
            }
            (NS_CONNECTION, "CLOSE") if Some(&m.source) == transport.as_ref() => {
                on_status(Status {
                    player: "IDLE".into(),
                    position: 0.0,
                    duration: None,
                    idle_reason: Some("CANCELLED".into()),
                    error: None,
                });
                return Ok(());
            }
            _ => {}
        }
    }
}

pub fn parse_status(s: &Value) -> Status {
    let player = s["playerState"].as_str().unwrap_or("IDLE").to_string();
    let idle_reason = if player == "IDLE" {
        s["idleReason"].as_str().map(str::to_string)
    } else {
        None
    };
    let error = (idle_reason.as_deref() == Some("ERROR"))
        .then(|| "the Chromecast stopped with an error playing this".to_string());
    Status {
        position: s["currentTime"].as_f64().unwrap_or(0.0),
        duration: s["media"]["duration"].as_f64(),
        player,
        idle_reason,
        error,
    }
}

/// Stops the media receiver on the TV so it goes back to its own screen.
fn stop(link: &mut Link, session_id: Option<&str>) {
    if let Some(s) = session_id {
        let id = link.next_id();
        let _ = link.send(
            "receiver-0",
            NS_RECEIVER,
            json!({"type": "STOP", "sessionId": s, "requestId": id}),
        );
    }
    let _ = link.send("receiver-0", NS_CONNECTION, json!({"type": "CLOSE"}));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip() {
        let m = Message {
            source: "sender-0".into(),
            destination: "receiver-0".into(),
            namespace: NS_HEARTBEAT.into(),
            payload: r#"{"type":"PING"}"#.into(),
        };
        let bytes = encode(&m);
        let len = u32::from_be_bytes(bytes[..4].try_into().unwrap()) as usize;
        assert_eq!(len, bytes.len() - 4);
        assert_eq!(decode(&bytes[4..]), Some(m));
    }

    #[test]
    fn media_status_reads_position_and_end() {
        let s = json!({"mediaSessionId": 3, "playerState": "PLAYING", "currentTime": 61.5, "media": {"duration": 1420.0}});
        assert_eq!(
            parse_status(&s),
            Status {
                player: "PLAYING".into(),
                position: 61.5,
                duration: Some(1420.0),
                idle_reason: None,
                error: None
            }
        );
        let done = json!({"playerState": "IDLE", "idleReason": "FINISHED", "currentTime": 0});
        assert_eq!(parse_status(&done).idle_reason.as_deref(), Some("FINISHED"));
    }

    #[test]
    fn a_load_for_hls_names_the_fmp4_segments_and_no_side_track() {
        let l = Load {
            url: "http://192.168.1.2:8000/c/x/master.m3u8".into(),
            content_type: "application/x-mpegurl".into(),
            hls_fmp4: true,
            start: 0.0,
            duration: 1420.0,
            title: "EP 1".into(),
            subtitles: None,
        };
        let m = media_info(&l);
        assert_eq!(m["hlsSegmentFormat"], "fmp4");
        assert!(m.get("tracks").is_none());
    }
}
