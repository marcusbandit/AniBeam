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

use super::{Activity, Incoming, SendError, Transport};

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
/// How long one connect may spend before it stops opening sockets. Each that accepts and stays
/// silent costs `IO_TIMEOUT`, and the handshake already under way when the budget runs out
/// still finishes. On Windows the pipe has no timeout, so a silent one is not bounded at all.
const CONNECT_BUDGET: Duration = Duration::from_secs(12);

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

/// The events this app asks Discord for after the handshake: someone pressed
/// Ask to Join on this person's profile, and this person pressed Join or
/// accepted an invite, which hands over the host's join secret.
pub const SUBSCRIBED: [&str; 2] = ["ACTIVITY_JOIN", "ACTIVITY_JOIN_REQUEST"];

/// An open, handshaken connection over any byte stream, so the tests can
/// speak to it from memory.
pub struct Connection<S> {
    stream: S,
    nonce: u64,
    /// What Discord said unasked while a reply was awaited, for the next poll.
    events: Vec<Incoming>,
}

impl<S: Stream> Connection<S> {
    /// Says which application this is, waits for READY and asks for the join
    /// events. A Discord that refuses or ignores a subscription still shows
    /// activities, so the subscriptions are never waited on. A Discord that
    /// answers the handshake with a close or an error refused it, and says
    /// why (#1029); one that never answers is no Discord.
    pub fn handshake(mut stream: S, client_id: &str) -> Result<Connection<S>, SendError> {
        stream.write_all(&encode(HANDSHAKE, &json!({"v": 1, "client_id": client_id})))?;
        let mut conn = Connection {
            stream,
            nonce: 0,
            events: Vec::new(),
        };
        let reply = match conn.next_reply(|_| true) {
            Ok(reply) => reply,
            Err(SendError::Gone(e)) => {
                return Err(match e.strip_prefix(CLOSED) {
                    Some(why) => SendError::Rejected(why.to_string()),
                    None => SendError::Gone(e),
                });
            }
            Err(e) => return Err(e),
        };
        if reply["evt"] != "READY" {
            // An error that says why is a refusal; anything else is not a
            // Discord that can be named.
            return Err(match reply["data"]["message"].as_str() {
                Some(why) if !why.trim().is_empty() => SendError::Rejected(why.trim().to_string()),
                _ => SendError::Gone(format!("no READY: {reply}")),
            });
        }
        // Sent without waiting on the replies: a server that ignores SUBSCRIBE still shows
        // activities, and the replies are skipped by nonce on the way to the next one.
        for evt in SUBSCRIBED {
            conn.write_command("SUBSCRIBE", Some(evt), json!({}))?;
        }
        Ok(conn)
    }

    /// Writes one command frame, with a nonce no other command on this
    /// connection has carried, and gives the nonce its reply will carry.
    fn write_command(
        &mut self,
        cmd: &str,
        evt: Option<&str>,
        args: Value,
    ) -> Result<String, SendError> {
        self.nonce += 1;
        let nonce = format!("anibeam-{}-{}", std::process::id(), self.nonce);
        let mut command = json!({"cmd": cmd, "args": args, "nonce": nonce});
        if let Some(evt) = evt {
            command["evt"] = json!(evt);
        }
        self.stream.write_all(&encode(FRAME, &command))?;
        Ok(nonce)
    }

    /// Sends one command and reads its reply, so the socket never fills. An
    /// ERROR reply is a rejection; anything else that goes wrong means the
    /// connection is gone.
    fn command(&mut self, cmd: &str, args: Value) -> Result<Value, SendError> {
        let nonce = self.write_command(cmd, None, args)?;
        let reply = self.next_reply(|v| v["nonce"] == nonce.as_str())?;
        if reply["evt"] == "ERROR" {
            let message = reply["data"]["message"].as_str().unwrap_or("rejected");
            return Err(SendError::Rejected(message.to_string()));
        }
        Ok(reply)
    }

    /// Sets the activity, or clears it with `None`.
    pub fn set_activity(&mut self, activity: Option<&Value>) -> Result<(), SendError> {
        let args = json!({"pid": std::process::id(), "activity": activity});
        self.command("SET_ACTIVITY", args).map(|_| ())
    }

    /// Answers someone's Ask to Join: an invite, which hands them the join
    /// secret, or a close, which takes the request off this person's screen.
    pub fn answer(&mut self, user_id: &str, invite: bool) -> Result<(), SendError> {
        let cmd = if invite {
            "SEND_ACTIVITY_JOIN_INVITE"
        } else {
            "CLOSE_ACTIVITY_JOIN_REQUEST"
        };
        self.command(cmd, json!({"user_id": user_id})).map(|_| ())
    }

    /// Everything Discord has said unasked since the last poll, reading only
    /// what is already waiting on the socket.
    pub fn poll(&mut self) -> Result<Vec<Incoming>, SendError> {
        let mut read = 0;
        while read < MAX_SKIPPED && self.stream.ready()? {
            read += 1;
            let (op, value) = read_frame(&mut self.stream)?;
            self.unasked(op, value)?;
        }
        Ok(std::mem::take(&mut self.events))
    }

    /// The first command frame `wanted` accepts. Pings are answered and
    /// events kept on the way; a close frame, or one too many frames that
    /// are not it, is the end.
    fn next_reply(&mut self, wanted: impl Fn(&Value) -> bool) -> Result<Value, SendError> {
        for _ in 0..MAX_SKIPPED {
            let (op, value) = read_frame(&mut self.stream)?;
            if op == FRAME && wanted(&value) {
                return Ok(value);
            }
            self.unasked(op, value)?;
        }
        Err(SendError::Gone("no reply".to_string()))
    }

    /// A frame nobody waited for: a ping, a close, or an event.
    fn unasked(&mut self, op: u32, value: Value) -> Result<(), SendError> {
        match op {
            PING => self.stream.write_all(&encode(PONG, &value))?,
            CLOSE => return Err(SendError::Gone(format!("{CLOSED}{}", said(&value)))),
            FRAME => self.events.extend(incoming(&value)),
            _ => {}
        }
        Ok(())
    }
}

/// What a close frame's message starts with, which the handshake reads as
/// Discord refusing it.
const CLOSED: &str = "closed: ";

/// The words in a close or error frame: its message, else the frame itself.
fn said(value: &Value) -> String {
    match value["message"].as_str() {
        Some(m) if !m.trim().is_empty() => m.trim().to_string(),
        _ => value.to_string(),
    }
}

/// The event in a DISPATCH frame, when it is one this app subscribed to and
/// says what it must. A user id is a Discord snowflake, all digits, and an
/// avatar a hash, so neither can turn the avatar's URL into another one.
pub fn incoming(v: &Value) -> Option<Incoming> {
    if v["cmd"] != "DISPATCH" {
        return None;
    }
    match v["evt"].as_str()? {
        "ACTIVITY_JOIN" => {
            let secret = v["data"]["secret"].as_str()?;
            Some(Incoming::Join {
                secret: secret.to_string(),
            })
        }
        "ACTIVITY_JOIN_REQUEST" => {
            let user = &v["data"]["user"];
            let id = user["id"].as_str()?;
            if id.is_empty() || id.len() > 24 || !id.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let name = [&user["global_name"], &user["username"]]
                .into_iter()
                .filter_map(|n| n.as_str())
                .map(str::trim)
                .find(|n| !n.is_empty())
                .unwrap_or(id);
            let avatar = user["avatar"]
                .as_str()
                .filter(|h| {
                    !h.is_empty()
                        && h.len() <= 64
                        && h.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                })
                .map(|h| format!("https://cdn.discordapp.com/avatars/{id}/{h}.png?size=128"));
            Some(Incoming::JoinRequest {
                user_id: id.to_string(),
                name: name.chars().take(64).collect(),
                avatar,
            })
        }
        _ => None,
    }
}

/// A byte stream to Discord that can say whether a frame is waiting, so a
/// poll never blocks on a quiet socket.
pub trait Stream: Read + Write + Send {
    /// Whether a read would find bytes now. An ended stream says yes, so the
    /// read that follows finds the end and the connection is gone.
    fn ready(&mut self) -> io::Result<bool>;
}

impl<S: Stream + ?Sized> Stream for Box<S> {
    fn ready(&mut self) -> io::Result<bool> {
        (**self).ready()
    }
}

/// A Unix socket with the bytes a readiness check read ahead, which the next read hands
/// out first. Std has no stable peek on a Unix socket, so the check reads instead.
#[cfg(unix)]
pub struct Socket {
    stream: std::os::unix::net::UnixStream,
    ahead: std::collections::VecDeque<u8>,
}

#[cfg(unix)]
impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.ahead.is_empty() {
            return self.stream.read(buf);
        }
        let n = buf.len().min(self.ahead.len());
        for (slot, byte) in buf.iter_mut().zip(self.ahead.drain(..n)) {
            *slot = byte;
        }
        Ok(n)
    }
}

#[cfg(unix)]
impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

#[cfg(unix)]
impl Stream for Socket {
    fn ready(&mut self) -> io::Result<bool> {
        if !self.ahead.is_empty() {
            return Ok(true);
        }
        self.stream.set_nonblocking(true)?;
        let mut chunk = [0u8; 4096];
        let got = self.stream.read(&mut chunk);
        self.stream.set_nonblocking(false)?;
        match got {
            // The end: the read that follows finds it too.
            Ok(0) => Ok(true),
            Ok(n) => {
                self.ahead.extend(&chunk[..n]);
                Ok(true)
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(windows)]
impl Stream for std::fs::File {
    fn ready(&mut self) -> io::Result<bool> {
        use std::os::windows::io::AsRawHandle;
        let mut avail: u32 = 0;
        // SAFETY: the handle is this open pipe's, and with no buffer Windows only
        // writes the byte count into `avail`.
        let ok = unsafe {
            windows_sys::Win32::System::Pipes::PeekNamedPipe(
                self.as_raw_handle() as _,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut avail,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(avail > 0)
    }
}

/// The real transport: finds the socket, handshakes, and holds the
/// connection until it breaks or is closed.
#[derive(Default)]
pub struct Discord {
    conn: Option<Connection<Box<dyn Stream>>>,
    /// The candidate the next connect tries first: the one that last answered, or the
    /// one a spent budget stopped short of, so silent sockets early in the list never
    /// hide a real Discord later in it.
    next: usize,
}

impl Transport for Discord {
    fn connect(&mut self, client_id: &str) -> Result<(), SendError> {
        self.conn = None;
        let paths = candidates();
        let began = std::time::Instant::now();
        let (result, next) = walk(
            paths.len(),
            self.next,
            || began.elapsed() >= CONNECT_BUDGET,
            |i| {
                let stream = open(&paths[i]).ok()?;
                Some(Connection::handshake(stream, client_id))
            },
        );
        self.next = next;
        self.conn = Some(result?);
        Ok(())
    }

    fn send(&mut self, activity: Option<&Activity>) -> Result<(), SendError> {
        self.with_conn(|conn| conn.set_activity(activity.map(Activity::to_json).as_ref()))
            .unwrap_or_else(not_connected)
    }

    /// With no connection there is nothing to read, which is no failure.
    fn poll(&mut self) -> Result<Vec<Incoming>, SendError> {
        self.with_conn(Connection::poll)
            .unwrap_or_else(|| Ok(Vec::new()))
    }

    fn answer(&mut self, user_id: &str, invite: bool) -> Result<(), SendError> {
        self.with_conn(|conn| conn.answer(user_id, invite))
            .unwrap_or_else(not_connected)
    }

    fn close(&mut self) {
        self.conn = None;
    }
}

impl Discord {
    /// Runs `f` on the open connection, or gives None when there is none. A connection
    /// `f` finds gone is let go of; one Discord only refused something is kept.
    fn with_conn<T>(
        &mut self,
        f: impl FnOnce(&mut Connection<Box<dyn Stream>>) -> Result<T, SendError>,
    ) -> Option<Result<T, SendError>> {
        let result = f(self.conn.as_mut()?);
        if matches!(result, Err(SendError::Gone(_))) {
            self.conn = None;
        }
        Some(result)
    }
}

fn not_connected() -> Result<(), SendError> {
    Err(SendError::Gone("not connected".to_string()))
}

/// Tries `len` candidates in turn from `from`, wrapping round, until one answers or
/// `spent` says the budget is gone. `attempt` gives None for a candidate with nothing
/// listening. Returns the outcome and where the next walk should start.
fn walk<T>(
    len: usize,
    from: usize,
    mut spent: impl FnMut() -> bool,
    mut attempt: impl FnMut(usize) -> Option<Result<T, SendError>>,
) -> (Result<T, SendError>, usize) {
    let from = if len == 0 { 0 } else { from % len };
    // A refusal outlives a later socket that only went quiet or refused
    // too, so Settings names what the first Discord that answered said (#1029).
    let mut last = SendError::Gone("no Discord socket".to_string());
    for step in 0..len {
        let i = (from + step) % len;
        if spent() {
            return (Err(last), i);
        }
        match attempt(i) {
            Some(Ok(found)) => return (Ok(found), i),
            // The first refusal stands, so the reason Settings shows does
            // not swap between two Discords from one retry to the next.
            Some(Err(e)) if matches!(last, SendError::Gone(_)) => last = e,
            Some(Err(_)) | None => {}
        }
    }
    (Err(last), from)
}

/// Every place a Discord client may have put its socket, in the order the
/// official library looks: the runtime dir, the temp dirs, then `/tmp`, with
/// the Flatpak and Snap builds' own corners of the runtime dir.
/// `ANIBEAM_DISCORD_IPC` names the one socket to try, for a stand-in Discord
/// beside a real one. Not part of the contract.
fn override_path() -> Option<PathBuf> {
    std::env::var_os("ANIBEAM_DISCORD_IPC")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
}

#[cfg(unix)]
fn candidates() -> Vec<PathBuf> {
    if let Some(path) = override_path() {
        return vec![path];
    }
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(run) = std::env::var_os("XDG_RUNTIME_DIR") {
        let run = PathBuf::from(run);
        dirs.push(run.join("app/com.discordapp.Discord"));
        dirs.push(run.join("app/dev.vencord.Vesktop"));
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
    Ok(Box::new(Socket {
        stream,
        ahead: Default::default(),
    }))
}

#[cfg(windows)]
fn candidates() -> Vec<PathBuf> {
    if let Some(path) = override_path() {
        return vec![path];
    }
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

    #[test]
    fn a_spent_budget_resumes_where_it_stopped_so_a_late_discord_is_reached() {
        // Candidates 0 to 2 accept and stay silent; Discord is at 4. The budget
        // covers three candidates a walk.
        let attempt = |i: usize| match i {
            0..=2 => Some(Err::<usize, _>(SendError::Gone("silent".to_string()))),
            4 => Some(Ok(4)),
            _ => None,
        };
        let budget = |n: u32| {
            let mut tried = 0;
            move || {
                tried += 1;
                tried > n
            }
        };
        let (first, next) = walk(6, 0, budget(3), attempt);
        assert!(first.is_err());
        assert_eq!(next, 3);
        let (second, next) = walk(6, next, budget(3), attempt);
        assert_eq!(second.ok(), Some(4));
        // The next connect goes straight back to the one that answered.
        assert_eq!(next, 4);
        let (third, _) = walk(6, next, budget(0), attempt);
        assert!(third.is_err());
        assert_eq!(walk(6, 4, budget(1), attempt).0.ok(), Some(4));
    }

    #[test]
    fn a_refusal_is_kept_over_a_later_socket_that_only_went_quiet() {
        let (result, _) = walk(
            3,
            0,
            || false,
            |i| match i {
                0 => Some(Err::<(), _>(SendError::Rejected(
                    "Invalid Client ID".into(),
                ))),
                1 => Some(Err(SendError::Gone("timed out".into()))),
                _ => None,
            },
        );
        assert!(matches!(result, Err(SendError::Rejected(m)) if m == "Invalid Client ID"));
        // Nor does a second refusal take its place.
        let (result, _) = walk(
            2,
            0,
            || false,
            |i| Some(Err::<(), _>(SendError::Rejected(format!("refusal {i}")))),
        );
        assert!(matches!(result, Err(SendError::Rejected(m)) if m == "refusal 0"));
    }

    #[test]
    fn a_walk_with_nothing_listening_starts_over_from_the_same_place() {
        let (result, next) = walk(3, 7, || false, |_| None::<Result<(), SendError>>);
        assert!(result.is_err());
        assert_eq!(next, 1);
        assert!(
            walk(0, 5, || false, |_| None::<Result<(), SendError>>)
                .0
                .is_err()
        );
    }

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

    impl Stream for Fake {
        fn ready(&mut self) -> io::Result<bool> {
            Ok((self.input.position() as usize) < self.input.get_ref().len())
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

    /// READY and the replies to the two subscriptions, as a Discord gives them.
    fn opened() -> Vec<(u32, Value)> {
        vec![
            ready(),
            (
                FRAME,
                json!({"cmd": "SUBSCRIBE", "evt": null, "nonce": nonce(1)}),
            ),
            (
                FRAME,
                json!({"cmd": "SUBSCRIBE", "evt": null, "nonce": nonce(2)}),
            ),
        ]
    }

    fn with(mut frames: Vec<(u32, Value)>, more: Vec<(u32, Value)>) -> Fake {
        frames.extend(more);
        Fake::new(&frames)
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
    fn the_handshake_names_the_application_waits_for_ready_and_subscribes() {
        let conn = Connection::handshake(Fake::new(&opened()), "123").unwrap();
        let written = conn.stream.written();
        assert_eq!(written[0], (HANDSHAKE, json!({"v": 1, "client_id": "123"})));
        assert_eq!(written.len(), 3);
        for (frame, evt) in written[1..].iter().zip(SUBSCRIBED) {
            assert_eq!(frame.1["cmd"], "SUBSCRIBE");
            assert_eq!(frame.1["evt"], evt);
        }
        let refused = Connection::handshake(
            Fake::new(&[(CLOSE, json!({"code": 4000, "message": "Invalid Client ID"}))]),
            "123",
        );
        // A close in answer to the handshake is Discord refusing it, in its words (#1029).
        assert!(matches!(refused, Err(SendError::Rejected(m)) if m == "Invalid Client ID"));
        // So is an error in place of READY.
        let error = Connection::handshake(
            Fake::new(&[(
                FRAME,
                json!({"cmd": "DISPATCH", "evt": "ERROR", "data": {"code": 4000, "message": "Invalid Origin"}}),
            )]),
            "123",
        );
        assert!(matches!(error, Err(SendError::Rejected(m)) if m == "Invalid Origin"));
        // A first frame that is neither READY nor an error that says why is no Discord.
        let other = Connection::handshake(
            Fake::new(&[(FRAME, json!({"cmd": "DISPATCH", "evt": "SOMETHING_ELSE"}))]),
            "123",
        );
        assert!(matches!(other, Err(SendError::Gone(_))));
        // A stream that ends before answering is no Discord.
        assert!(matches!(
            Connection::handshake(Fake::new(&[]), "123"),
            Err(SendError::Gone(_))
        ));
    }

    #[test]
    fn a_refused_or_ignored_subscription_still_connects_and_shows() {
        // Ignored: READY and nothing more.
        assert!(Connection::handshake(Fake::new(&[ready()]), "123").is_ok());
        // Refused, then a set that is answered.
        let refused = |n| {
            (
                FRAME,
                json!({"cmd": "SUBSCRIBE", "evt": "ERROR", "nonce": nonce(n),
                       "data": {"code": 4006, "message": "Not authenticated"}}),
            )
        };
        let fake = Fake::new(&[
            ready(),
            refused(1),
            refused(2),
            (
                FRAME,
                json!({"cmd": "SET_ACTIVITY", "evt": null, "nonce": nonce(3)}),
            ),
        ]);
        let mut conn = Connection::handshake(fake, "123").unwrap();
        conn.set_activity(None).unwrap();
    }

    #[test]
    fn set_activity_sends_the_command_and_reads_its_reply() {
        let fake = with(
            opened(),
            vec![
                (PING, json!({"p": 1})),
                (
                    FRAME,
                    json!({"cmd": "SET_ACTIVITY", "evt": null, "nonce": nonce(3)}),
                ),
                (
                    FRAME,
                    json!({"cmd": "SET_ACTIVITY", "evt": null, "nonce": nonce(4)}),
                ),
            ],
        );
        let mut conn = Connection::handshake(fake, "123").unwrap();
        conn.set_activity(Some(&json!({"details": "Bebop"})))
            .unwrap();
        conn.set_activity(None).unwrap();
        let written = conn.stream.written();
        assert_eq!(written.len(), 6);
        assert_eq!(written[3].1["cmd"], "SET_ACTIVITY");
        assert_eq!(written[3].1["args"]["activity"]["details"], "Bebop");
        assert_eq!(written[3].1["args"]["pid"], std::process::id());
        // The ping was answered with its own payload.
        assert_eq!(written[4], (PONG, json!({"p": 1})));
        assert_eq!(written[5].1["args"]["activity"], Value::Null);
        assert_eq!(written[5].1["nonce"], nonce(4));
    }

    #[test]
    fn an_error_reply_is_a_rejection_and_a_closed_socket_is_gone() {
        let fake = with(
            opened(),
            vec![(
                FRAME,
                json!({"cmd": "SET_ACTIVITY", "evt": "ERROR", "nonce": nonce(3),
                       "data": {"code": 4000, "message": "bad large_image"}}),
            )],
        );
        let mut conn = Connection::handshake(fake, "123").unwrap();
        assert!(matches!(
            conn.set_activity(Some(&json!({}))),
            Err(SendError::Rejected(m)) if m == "bad large_image"
        ));
        assert!(matches!(conn.set_activity(None), Err(SendError::Gone(_))));
    }

    fn ask(id: &str, avatar: Value) -> (u32, Value) {
        (
            FRAME,
            json!({"cmd": "DISPATCH", "evt": "ACTIVITY_JOIN_REQUEST", "nonce": null,
                   "data": {"user": {"id": id, "username": "spike", "global_name": "Spike S",
                                     "avatar": avatar}}}),
        )
    }

    #[test]
    fn a_poll_reads_waiting_events_and_those_that_came_with_a_reply() {
        let fake = with(
            opened(),
            vec![
                ask("101", json!("abc123")),
                (
                    FRAME,
                    json!({"cmd": "SET_ACTIVITY", "evt": null, "nonce": nonce(3)}),
                ),
                (
                    FRAME,
                    json!({"cmd": "DISPATCH", "evt": "ACTIVITY_JOIN", "nonce": null,
                               "data": {"secret": "room:K7QM2X"}}),
                ),
                (
                    FRAME,
                    json!({"cmd": "DISPATCH", "evt": "ACTIVITY_SPECTATE", "data": {}}),
                ),
            ],
        );
        let mut conn = Connection::handshake(fake, "123").unwrap();
        conn.set_activity(None).unwrap();
        let events = conn.poll().unwrap();
        assert_eq!(
            events,
            vec![
                Incoming::JoinRequest {
                    user_id: "101".into(),
                    name: "Spike S".into(),
                    avatar: Some(
                        "https://cdn.discordapp.com/avatars/101/abc123.png?size=128".into()
                    ),
                },
                Incoming::Join {
                    secret: "room:K7QM2X".into()
                },
            ]
        );
        // Nothing waiting is nothing, and the poll did not block.
        assert_eq!(conn.poll().unwrap(), vec![]);
    }

    #[test]
    fn a_request_with_an_odd_id_is_dropped_and_an_odd_avatar_is_none() {
        assert_eq!(incoming(&ask("1/../x", json!("abc")).1), None);
        let Some(Incoming::JoinRequest { avatar, .. }) = incoming(&ask("7", json!("a/b")).1) else {
            panic!("a request with a good id is kept")
        };
        assert_eq!(avatar, None);
    }

    #[test]
    fn an_answer_invites_or_closes_the_request() {
        let fake = with(
            opened(),
            vec![
                (
                    FRAME,
                    json!({"cmd": "SEND_ACTIVITY_JOIN_INVITE", "evt": null, "nonce": nonce(3)}),
                ),
                (
                    FRAME,
                    json!({"cmd": "CLOSE_ACTIVITY_JOIN_REQUEST", "evt": null, "nonce": nonce(4)}),
                ),
            ],
        );
        let mut conn = Connection::handshake(fake, "123").unwrap();
        conn.answer("101", true).unwrap();
        conn.answer("102", false).unwrap();
        let written = conn.stream.written();
        assert_eq!(written[3].1["cmd"], "SEND_ACTIVITY_JOIN_INVITE");
        assert_eq!(written[3].1["args"]["user_id"], "101");
        assert_eq!(written[4].1["cmd"], "CLOSE_ACTIVITY_JOIN_REQUEST");
        assert_eq!(written[4].1["args"]["user_id"], "102");
    }

    #[test]
    fn an_unconnected_transport_sends_nothing() {
        let mut d = Discord::default();
        assert!(matches!(d.send(None), Err(SendError::Gone(_))));
        assert_eq!(d.poll().unwrap(), vec![]);
    }

    /// A refusal keeps the connection, and a connection found gone is let go of, so the
    /// worker connects again rather than writing to a dead socket.
    #[test]
    fn a_refusal_keeps_the_connection_and_a_gone_one_is_dropped() {
        let fake = with(
            opened(),
            vec![(
                FRAME,
                json!({"cmd": "SET_ACTIVITY", "evt": "ERROR", "nonce": nonce(3),
                       "data": {"code": 4000, "message": "bad large_image"}}),
            )],
        );
        let conn = Connection::handshake(Box::new(fake) as Box<dyn Stream>, "123").unwrap();
        let mut d = Discord {
            conn: Some(conn),
            next: 0,
        };
        assert!(matches!(d.send(None), Err(SendError::Rejected(_))));
        assert!(d.conn.is_some(), "a refusal leaves the connection standing");
        // Nothing more on the wire: the next send finds the stream at its end.
        assert!(matches!(d.send(None), Err(SendError::Gone(_))));
        assert!(d.conn.is_none(), "a gone connection is let go of");
    }
}
