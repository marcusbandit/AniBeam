//! The wire to a running Discord client: its local IPC socket, a Unix socket
//! named `discord-ipc-N` on Linux and macOS and a named pipe of the same name
//! on Windows. Each frame is a little-endian opcode, a little-endian length
//! and that many bytes of JSON. A handshake names the Discord application,
//! Discord answers READY, and from then on each `SET_ACTIVITY` command gets a
//! reply carrying its nonce. Discord drops the activity by itself when the
//! socket closes.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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
/// How long finding Discord may take in all. Sockets that accept and never
/// answer share it rather than costing `IO_TIMEOUT` each.
const CONNECT_BUDGET: Duration = Duration::from_secs(5);

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

/// The real transport: finds the socket, handshakes, and holds the
/// connection until it breaks or is closed.
#[derive(Default)]
pub struct Discord {
    conn: Option<Connection<Socket>>,
}

impl Transport for Discord {
    fn connect(&mut self, client_id: &str) -> Result<(), SendError> {
        self.conn = None;
        self.conn = Some(connect_any(candidates(), client_id, CONNECT_BUDGET)?);
        Ok(())
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

/// The first of `paths` that handshakes, all of them within `budget`: each
/// read and write of a handshake waits no longer than the time left, and no
/// socket is tried once it is gone. The connection then gets `IO_TIMEOUT`.
fn connect_any(
    paths: Vec<PathBuf>,
    client_id: &str,
    budget: Duration,
) -> Result<Connection<Socket>, SendError> {
    let deadline = Instant::now() + budget;
    let mut last = SendError::Gone("no Discord socket".to_string());
    for path in paths {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        let Ok(socket) = open(&path, left.min(IO_TIMEOUT)) else {
            continue;
        };
        match Connection::handshake(socket, client_id) {
            Ok(mut conn) => {
                set_timeout(&mut conn.stream, IO_TIMEOUT)?;
                return Ok(conn);
            }
            Err(e) => last = e,
        }
    }
    Err(last)
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
type Socket = std::os::unix::net::UnixStream;

#[cfg(unix)]
fn open(path: &Path, wait: Duration) -> io::Result<Socket> {
    let mut stream = Socket::connect(path)?;
    set_timeout(&mut stream, wait)?;
    Ok(stream)
}

/// How long each read and each write may wait from now on.
#[cfg(unix)]
fn set_timeout(stream: &mut Socket, wait: Duration) -> io::Result<()> {
    stream.set_read_timeout(Some(wait))?;
    stream.set_write_timeout(Some(wait))
}

#[cfg(windows)]
fn candidates() -> Vec<PathBuf> {
    (0..10)
        .map(|i| PathBuf::from(format!(r"\\?\pipe\discord-ipc-{i}")))
        .collect()
}

#[cfg(windows)]
type Socket = pipe::Pipe;

#[cfg(windows)]
fn open(path: &Path, wait: Duration) -> io::Result<Socket> {
    pipe::Pipe::open(path, wait)
}

/// How long each read and each write may wait from now on.
#[cfg(windows)]
fn set_timeout(pipe: &mut Socket, wait: Duration) -> io::Result<()> {
    pipe.timeout = wait;
    Ok(())
}

/// A named pipe has no read or write timeout of its own, and a synchronous
/// read on one that Discord opened but never answers would wait forever. So
/// the pipe is opened for overlapped I/O, and each read and write is started,
/// waited on for at most the timeout, and cancelled past it.
#[cfg(windows)]
mod pipe {
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;
    use std::path::Path;
    use std::time::Duration;

    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_BROKEN_PIPE, ERROR_IO_PENDING, ERROR_MORE_DATA, GetLastError, HANDLE,
        WAIT_TIMEOUT,
    };
    use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_OVERLAPPED, ReadFile, WriteFile};
    use windows_sys::Win32::System::IO::{
        CancelIoEx, GetOverlappedResult, GetOverlappedResultEx, OVERLAPPED,
    };
    use windows_sys::Win32::System::Threading::CreateEventW;
    use windows_sys::core::BOOL;

    pub struct Pipe {
        file: File,
        /// Signalled when the operation in flight completes.
        event: HANDLE,
        pub timeout: Duration,
    }

    // SAFETY: the event handle belongs to this pipe alone, and a Win32
    // handle may be used from any thread.
    unsafe impl Send for Pipe {}

    impl Pipe {
        pub fn open(path: &Path, timeout: Duration) -> io::Result<Pipe> {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(FILE_FLAG_OVERLAPPED)
                .open(path)?;
            // Manual reset, as overlapped I/O wants; ReadFile and WriteFile
            // reset it as each operation starts.
            // SAFETY: no security attributes and no name, both may be null.
            let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
            if event.is_null() {
                return Err(io::Error::last_os_error());
            }
            Ok(Pipe {
                file,
                event,
                timeout,
            })
        }

        /// Runs one overlapped operation, which `start` begins, and waits at
        /// most the timeout for it. Past that it is cancelled, and the
        /// cancel is waited out: the kernel holds the buffer and the
        /// OVERLAPPED until the operation is over, so neither may go first.
        fn run(&self, start: impl FnOnce(HANDLE, *mut OVERLAPPED) -> BOOL) -> io::Result<usize> {
            let handle: HANDLE = self.file.as_raw_handle();
            let mut ov = OVERLAPPED {
                hEvent: self.event,
                ..Default::default()
            };
            let mut done = 0u32;
            let wait = u32::try_from(self.timeout.as_millis())
                .unwrap_or(u32::MAX - 1)
                .max(1);
            // SAFETY: `ov` and the buffer `start` was handed outlive the
            // operation: every path out of this block either never started
            // it or has seen it complete.
            unsafe {
                if start(handle, &raw mut ov) == 0 {
                    let err = GetLastError();
                    if err != ERROR_IO_PENDING {
                        return Err(error(err));
                    }
                }
                if GetOverlappedResultEx(handle, &ov, &mut done, wait, 0) != 0 {
                    return Ok(done as usize);
                }
                let err = GetLastError();
                if err != WAIT_TIMEOUT {
                    return Err(error(err));
                }
                CancelIoEx(handle, &ov);
                // It may have finished just before the cancel.
                if GetOverlappedResult(handle, &ov, &mut done, 1) != 0 {
                    return Ok(done as usize);
                }
            }
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "the Discord pipe did not answer",
            ))
        }
    }

    fn error(code: u32) -> io::Error {
        io::Error::from_raw_os_error(code as i32)
    }

    impl Read for Pipe {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let len = u32::try_from(buf.len()).unwrap_or(u32::MAX);
            let ptr = buf.as_mut_ptr();
            let result = self.run(|handle, ov| {
                // SAFETY: `ptr` is valid for `len` bytes for as long as `run` waits.
                unsafe { ReadFile(handle, ptr, len, std::ptr::null_mut(), ov) }
            });
            // A pipe the far end closed is the end of the stream, and a read
            // that filled the buffer with more to come is still a read.
            match result {
                Err(e) if e.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) => Ok(0),
                Err(e) if e.raw_os_error() == Some(ERROR_MORE_DATA as i32) => Ok(buf.len()),
                other => other,
            }
        }
    }

    impl Write for Pipe {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let len = u32::try_from(buf.len()).unwrap_or(u32::MAX);
            let ptr = buf.as_ptr();
            self.run(|handle, ov| {
                // SAFETY: `ptr` is valid for `len` bytes for as long as `run` waits.
                unsafe { WriteFile(handle, ptr, len, std::ptr::null_mut(), ov) }
            })
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Drop for Pipe {
        fn drop(&mut self) {
            // SAFETY: the event was made in `open` and nothing else holds it.
            unsafe { CloseHandle(self.event) };
        }
    }
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
    fn silent_sockets_share_one_deadline() {
        let dir = tempfile::tempdir().unwrap();
        // Sockets that accept and never answer, as Discord's do while it
        // starts up.
        let paths: Vec<PathBuf> = (0..4)
            .map(|i| dir.path().join(format!("discord-ipc-{i}")))
            .collect();
        let _listening: Vec<_> = paths
            .iter()
            .map(|p| std::os::unix::net::UnixListener::bind(p).unwrap())
            .collect();
        let started = Instant::now();
        let result = connect_any(paths, "123", Duration::from_millis(300));
        assert!(matches!(result, Err(SendError::Gone(_))));
        let took = started.elapsed();
        assert!(took < Duration::from_secs(2), "took {took:?}");
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
