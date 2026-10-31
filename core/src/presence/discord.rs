//! The wire to a running Discord client: its local IPC socket, a Unix socket
//! named `discord-ipc-N` on Linux and macOS and a named pipe of the same name
//! on Windows. Each frame is a little-endian opcode, a little-endian length
//! and that many bytes of JSON. A handshake names the Discord application,
//! Discord answers READY, and from then on each `SET_ACTIVITY` command gets a
//! reply carrying its nonce. Discord drops the activity by itself when the
//! socket closes.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value, json};

use super::{Activity, SendError, Transport};

pub const HANDSHAKE: u32 = 0;
pub const FRAME: u32 = 1;
pub const CLOSE: u32 = 2;
pub const PING: u32 = 3;
pub const PONG: u32 = 4;

/// Larger than any reply Discord sends to these commands. A length past it
/// is a stream that is not Discord's, read no further.
const MAX_FRAME: u32 = 1 << 20;
/// Frames read past the one awaited before giving up on it: a ping or a
/// stray dispatch is fine, an endless run of them is not a reply.
const MAX_SKIPPED: u32 = 16;
/// How long a read or a write on the socket may take. Discord answers in
/// milliseconds; one that does not is treated as gone.
const IO_TIMEOUT: Duration = Duration::from_secs(5);

/// One frame on the wire.
pub fn encode(op: u32, payload: &Value) -> Vec<u8> {
    let body = payload.to_string().into_bytes();
    let mut out = Vec::with_capacity(8 + body.len());
    out.extend_from_slice(&op.to_le_bytes());
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    out
}

/// The next frame off the wire, its opcode and its JSON.
pub fn read_frame(r: &mut impl Read) -> io::Result<(u32, Value)> {
    let mut head = [0u8; 8];
    r.read_exact(&mut head)?;
    let op = u32::from_le_bytes([head[0], head[1], head[2], head[3]]);
    let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]);
    if len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("a {len} byte frame"),
        ));
    }
    let mut body = vec![0u8; len as usize];
    r.read_exact(&mut body)?;
    let value =
        serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok((op, value))
}

/// An open, handshaken connection over any byte stream, so the tests can
/// speak to it from memory.
pub struct Connection<S> {
    stream: S,
    nonce: u64,
}

impl<S: Read + Write> Connection<S> {
    /// Says which application this is and waits for READY.
    pub fn handshake(mut stream: S, client_id: &str) -> Result<Connection<S>, SendError> {
        stream.write_all(&encode(HANDSHAKE, &json!({"v": 1, "client_id": client_id})))?;
        let reply = next_reply(&mut stream, |_| true)?;
        if reply["evt"] == "READY" {
            Ok(Connection { stream, nonce: 0 })
        } else {
            Err(SendError::Gone(format!("no READY: {reply}")))
        }
    }

    /// Sets the activity, or clears it with `None`, and reads the reply so
    /// the socket never fills. An ERROR reply is a rejection; anything else
    /// that goes wrong means the connection is gone.
    pub fn set_activity(&mut self, activity: Option<&Value>) -> Result<(), SendError> {
        self.nonce += 1;
        let nonce = format!("anibeam-{}-{}", std::process::id(), self.nonce);
        let command = json!({
            "cmd": "SET_ACTIVITY",
            "args": {"pid": std::process::id(), "activity": activity},
            "nonce": nonce,
        });
        self.stream.write_all(&encode(FRAME, &command))?;
        let reply = next_reply(&mut self.stream, |v| v["nonce"] == nonce.as_str())?;
        if reply["evt"] == "ERROR" {
            let message = reply["data"]["message"].as_str().unwrap_or("rejected");
            return Err(SendError::Rejected(message.to_string()));
        }
        Ok(())
    }
}

/// The first command frame `wanted` accepts, answering pings on the way. A
/// close frame, or one too many frames that are not it, is the end.
fn next_reply<S: Read + Write>(
    stream: &mut S,
    wanted: impl Fn(&Value) -> bool,
) -> Result<Value, SendError> {
    for _ in 0..MAX_SKIPPED {
        let (op, value) = read_frame(stream)?;
        match op {
            FRAME if wanted(&value) => return Ok(value),
            PING => stream.write_all(&encode(PONG, &value))?,
            CLOSE => return Err(SendError::Gone(format!("closed: {value}"))),
            _ => {}
        }
    }
    Err(SendError::Gone("no reply".to_string()))
}

pub trait Stream: Read + Write + Send {}
impl<T: Read + Write + Send> Stream for T {}

/// The real transport: finds the socket, handshakes, and holds the
/// connection until it breaks or is closed.
#[derive(Default)]
pub struct Discord {
    conn: Option<Connection<Box<dyn Stream>>>,
}

impl Transport for Discord {
    fn connect(&mut self, client_id: &str) -> Result<(), SendError> {
        self.conn = None;
        let mut last = SendError::Gone("no Discord socket".to_string());
        for path in candidates() {
            let Ok(stream) = open(&path) else { continue };
            match Connection::handshake(stream, client_id) {
                Ok(conn) => {
                    self.conn = Some(conn);
                    return Ok(());
                }
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    fn send(&mut self, activity: Option<&Activity>) -> Result<(), SendError> {
        let Some(conn) = self.conn.as_mut() else {
            return Err(SendError::Gone("not connected".to_string()));
        };
        let result = conn.set_activity(activity.map(Activity::to_json).as_ref());
        if matches!(result, Err(SendError::Gone(_))) {
            self.conn = None;
        }
        result
    }

    fn close(&mut self) {
        self.conn = None;
    }
}

/// Every place a Discord client may have put its socket, in the order the
/// official library looks: the runtime dir, the temp dirs, then `/tmp`, with
/// the Flatpak and Snap builds' own corners of the runtime dir.
#[cfg(unix)]
fn candidates() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(run) = std::env::var_os("XDG_RUNTIME_DIR") {
        let run = PathBuf::from(run);
        dirs.push(run.join("app/com.discordapp.Discord"));
        dirs.push(run.join("snap.discord"));
        dirs.insert(0, run);
    }
    for var in ["TMPDIR", "TMP", "TEMP"] {
        if let Some(dir) = std::env::var_os(var) {
            dirs.push(PathBuf::from(dir));
        }
    }
    dirs.push(PathBuf::from("/tmp"));
    let mut seen = Vec::new();
    dirs.retain(|d| {
        let fresh = !seen.contains(d);
        seen.push(d.clone());
        fresh
    });
    dirs.iter()
        .flat_map(|d| (0..10).map(move |i| d.join(format!("discord-ipc-{i}"))))
        .collect()
}

#[cfg(unix)]
fn open(path: &Path) -> io::Result<Box<dyn Stream>> {
    let stream = std::os::unix::net::UnixStream::connect(path)?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    Ok(Box::new(stream))
}

#[cfg(windows)]
fn candidates() -> Vec<PathBuf> {
    (0..10)
        .map(|i| PathBuf::from(format!(r"\\?\pipe\discord-ipc-{i}")))
        .collect()
}

#[cfg(windows)]
fn open(path: &Path) -> io::Result<Box<dyn Stream>> {
    let _ = IO_TIMEOUT;
    let pipe = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;
    Ok(Box::new(pipe))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Reads from canned frames, records what is written.
    struct Fake {
        input: Cursor<Vec<u8>>,
        output: Vec<u8>,
    }

    impl Fake {
        fn new(frames: &[(u32, Value)]) -> Fake {
            let input = frames.iter().flat_map(|(op, v)| encode(*op, v)).collect();
            Fake {
                input: Cursor::new(input),
                output: Vec::new(),
            }
        }

        fn written(&self) -> Vec<(u32, Value)> {
            let mut r = Cursor::new(self.output.clone());
            let mut out = Vec::new();
            while (r.position() as usize) < self.output.len() {
                out.push(read_frame(&mut r).unwrap());
            }
            out
        }
    }

    impl Read for Fake {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.input.read(buf)
        }
    }

    impl Write for Fake {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.output.write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn ready() -> (u32, Value) {
        (
            FRAME,
            json!({"cmd": "DISPATCH", "evt": "READY", "data": {"v": 1}}),
        )
    }

    fn nonce(n: u64) -> String {
        format!("anibeam-{}-{n}", std::process::id())
    }

    #[test]
    fn a_frame_is_the_opcode_the_length_and_the_json() {
        let bytes = encode(HANDSHAKE, &json!({"v": 1}));
        assert_eq!(&bytes[..4], &[0, 0, 0, 0]);
        assert_eq!(&bytes[4..8], &7u32.to_le_bytes());
        assert_eq!(&bytes[8..], br#"{"v":1}"#);
        let (op, v) = read_frame(&mut Cursor::new(bytes)).unwrap();
        assert_eq!((op, v), (HANDSHAKE, json!({"v": 1})));
    }

    #[test]
    fn an_oversized_or_short_frame_is_an_error() {
        let mut huge = 1u32.to_le_bytes().to_vec();
        huge.extend_from_slice(&(MAX_FRAME + 1).to_le_bytes());
        assert!(read_frame(&mut Cursor::new(huge)).is_err());
        let mut short = encode(FRAME, &json!({"a": 1}));
        short.truncate(10);
        assert!(read_frame(&mut Cursor::new(short)).is_err());
    }

    #[test]
    fn the_handshake_names_the_application_and_waits_for_ready() {
        let conn = Connection::handshake(Fake::new(&[ready()]), "123").unwrap();
        assert_eq!(
            conn.stream.written(),
            vec![(HANDSHAKE, json!({"v": 1, "client_id": "123"}))]
        );
        let refused = Connection::handshake(
            Fake::new(&[(CLOSE, json!({"code": 4000, "message": "Invalid Client ID"}))]),
            "123",
        );
        assert!(matches!(refused, Err(SendError::Gone(_))));
        // A stream that ends before answering is no Discord either.
        assert!(matches!(
            Connection::handshake(Fake::new(&[]), "123"),
            Err(SendError::Gone(_))
        ));
    }

    #[test]
    fn set_activity_sends_the_command_and_reads_its_reply() {
        let fake = Fake::new(&[
            ready(),
            (PING, json!({"p": 1})),
            (
                FRAME,
                json!({"cmd": "SET_ACTIVITY", "evt": null, "nonce": nonce(1)}),
            ),
            (
                FRAME,
                json!({"cmd": "SET_ACTIVITY", "evt": null, "nonce": nonce(2)}),
            ),
        ]);
        let mut conn = Connection::handshake(fake, "123").unwrap();
        conn.set_activity(Some(&json!({"details": "Bebop"})))
            .unwrap();
        conn.set_activity(None).unwrap();
        let written = conn.stream.written();
        assert_eq!(written.len(), 4);
        assert_eq!(written[1].1["cmd"], "SET_ACTIVITY");
        assert_eq!(written[1].1["args"]["activity"]["details"], "Bebop");
        assert_eq!(written[1].1["args"]["pid"], std::process::id());
        // The ping was answered with its own payload.
        assert_eq!(written[2], (PONG, json!({"p": 1})));
        assert_eq!(written[3].1["args"]["activity"], Value::Null);
        assert_eq!(written[3].1["nonce"], nonce(2));
    }

    #[test]
    fn an_error_reply_is_a_rejection_and_a_closed_socket_is_gone() {
        let fake = Fake::new(&[
            ready(),
            (
                FRAME,
                json!({"cmd": "SET_ACTIVITY", "evt": "ERROR", "nonce": nonce(1),
                       "data": {"code": 4000, "message": "bad large_image"}}),
            ),
        ]);
        let mut conn = Connection::handshake(fake, "123").unwrap();
        assert!(matches!(
            conn.set_activity(Some(&json!({}))),
            Err(SendError::Rejected(m)) if m == "bad large_image"
        ));
        assert!(matches!(conn.set_activity(None), Err(SendError::Gone(_))));
    }

    #[test]
    fn an_unconnected_transport_sends_nothing() {
        let mut d = Discord::default();
        assert!(matches!(d.send(None), Err(SendError::Gone(_))));
    }

    #[cfg(unix)]
    #[test]
    fn every_candidate_is_a_numbered_ipc_socket() {
        let all = candidates();
        assert!(all.len() >= 10 && all.len().is_multiple_of(10));
        assert!(all.contains(&PathBuf::from("/tmp/discord-ipc-0")));
        assert!(all.iter().all(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            name.starts_with("discord-ipc-")
        }));
    }
}
