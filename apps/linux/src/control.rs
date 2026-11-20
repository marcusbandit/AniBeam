//! `anibeam ctl` (#1205): an agent on this machine drives the window that is open, with no
//! mouse and nothing to install. The window listens on `anibeam-control.sock` in its runtime
//! directory (inside the sandbox under `--root`), readable by this user alone. A request is
//! one line of JSON, `{"steps": [...], "out": "<dir>", "timeout": ms}`, and the answer is one
//! line back: `{"ok": true, "result": ..., "log": [...]}` or `{"ok": false, "error": ...}`.
//! The steps are the `--drive` steps (Driver.qml), run by Control.qml against the window as it
//! is on screen, so a drive script runs here unchanged. The commands below turn a word and a
//! few arguments into those steps; `anibeam ctl help` is the guide an agent reads.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value, json};

/// How long a request may run when it does not say. A `wait` takes its own.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// The guide `anibeam ctl help` prints. An agent reads this and nothing else.
pub const HELP: &str = r#"anibeam ctl: drive the Anibeam window that is open on this machine

Every command talks to the running window over a socket in its runtime directory, so
nothing needs installing. --root <dir> picks the window started with that root.

  open [page ...] [--offscreen]   start a window when none runs, then go to the page.
                                  --offscreen starts it with no window on the desktop
                                  and no sound; it needs --root
  go <page> [words] [key=value ...]
                                  open a page from anywhere: whatever is open over the
                                  page closes first. words fill the page's main prop:
                                  go settings appearance, go series frieren,
                                  go library q=mushishi, go series id=12
  back, forward                   the window's own Back and Forward
  where                           the page, its props, and what is open over it
  pages                           every page and the props it opens with
  tree [filter]                   what is on screen: kind, words, objectName and box
                                  (x,y wxh in window pixels), one row each
  click <target> [--nth n] [--right] [--double]
  hover <target> [--nth n]
  scroll <target> <notches>       positive scrolls down, one notch is 100 px
  key <combo> ... [--focus]       Escape, Return, Ctrl+K, Alt+Left, Shift+Tab, a
  type <text> [--focus]           into whatever has the keyboard. --focus first asks
                                  for the keyboard, for a window on the desktop that
                                  lacks it; it can take focus from what you are doing
  shot [file.png]                 the window as it is; prints the file
  eval <js>                       in Main.qml's scope: window, frame, Door, Player,
                                  Theme, control. Prints the value as JSON
  wait <js> [--timeout ms]        until the JS is truthy
  run <script.json | ->           a --drive script, against this window (10 minutes
                                  unless --timeout says otherwise)
  quit                            close the window

A target is x,y in window pixels, an objectName, or words on screen. Words match whole
before partial, case aside, an open popup before the page, and only what is on screen;
--nth 2 takes the second match. Read `tree` to see what there is.

--json prints the whole reply. --timeout <ms> bounds each wait (30000 unless set).
-- ends the flags: type -- --focus types the word.
A failure prints why on stderr and exits 1; no window running exits 3.

Useful in eval: control.where(), frame.item.nav.current, frame.item.pageItem,
control.find("objectName"), Door.listSeries("All", "", "Alpha", "Asc", false).
"#;

/// Why a request got no answer.
#[derive(Debug, PartialEq)]
pub enum Fail {
    /// Nothing listens on the socket: no window runs for this root.
    NotRunning,
    Other(String),
}

impl std::fmt::Display for Fail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fail::NotRunning => write!(
                f,
                "no Anibeam window is running; `anibeam ctl open` starts one"
            ),
            Fail::Other(e) => f.write_str(e),
        }
    }
}

/// How a command's answer prints.
#[derive(Debug, PartialEq)]
enum Print {
    /// The result: a string bare, anything else as JSON, null as nothing.
    Result,
    /// The tree's rows, one a line.
    Tree,
    /// The file a shot wrote.
    Shot(PathBuf),
    /// Every log line, then the result.
    Log,
}

#[derive(Debug, PartialEq)]
struct Plan {
    steps: Value,
    out: Option<PathBuf>,
    print: Print,
    /// How long the whole request may take, in ms: a step's own wait is `timeout`
    budget: u64,
}

#[derive(Debug, Default, PartialEq)]
struct Opts {
    json: bool,
    timeout: Option<u64>,
    nth: Option<u64>,
    right: bool,
    double: bool,
    offscreen: bool,
    focus: bool,
    props: Option<String>,
}

/// Splits the flags out of the arguments, wherever they stand.
fn split_opts(args: &[String]) -> Result<(Vec<String>, Opts), String> {
    let mut o = Opts::default();
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--" {
            rest.extend(it.cloned());
            break;
        }
        let mut num = |name: &str| -> Result<u64, String> {
            it.next()
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| format!("{name} needs a number"))
        };
        match a.as_str() {
            "--json" => o.json = true,
            "--timeout" => o.timeout = Some(num("--timeout")?),
            "--nth" => o.nth = Some(num("--nth")?),
            "--right" => o.right = true,
            "--double" => o.double = true,
            "--offscreen" => o.offscreen = true,
            "--focus" => o.focus = true,
            "--props" => o.props = Some(it.next().cloned().ok_or("--props needs a JSON object")?),
            _ => rest.push(a.clone()),
        }
    }
    Ok((rest, o))
}

/// `x,y` as a point, anything else as a name or words.
fn target(t: &str) -> Value {
    if let Some((x, y)) = t.split_once(',')
        && let (Ok(x), Ok(y)) = (x.trim().parse::<f64>(), y.trim().parse::<f64>())
    {
        return json!([x, y]);
    }
    json!(t)
}

/// The prop a page's bare words fill.
fn main_prop(page: &str) -> Option<&'static str> {
    Some(match page {
        "settings" | "watching" => "tab",
        "series" => "find",
        "library" | "metadata" => "q",
        "downloads" => "query",
        "player" => "file",
        "show" => "anilist",
        "stats" => "year",
        _ => return None,
    })
}

/// A page step from `go`'s arguments: the page, then bare words and `key=value` props. A
/// value that reads as JSON (a number, true, an object) is taken as that.
fn page_step(page: &str, args: &[String], props: Option<&str>) -> Result<Value, String> {
    let mut p = match props {
        Some(text) => match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(m)) => m,
            _ => return Err("--props is a JSON object".into()),
        },
        None => serde_json::Map::new(),
    };
    let mut words = Vec::new();
    for a in args {
        match a.split_once('=') {
            Some((k, v)) if !k.is_empty() && !k.contains(' ') => {
                let v = serde_json::from_str::<Value>(v).unwrap_or_else(|_| json!(v));
                p.insert(k.to_string(), v);
            }
            _ => words.push(a.as_str()),
        }
    }
    if !words.is_empty() {
        let Some(key) = main_prop(page) else {
            return Err(format!(
                "{page} takes no words; give its props as key=value"
            ));
        };
        let text = words.join(" ");
        let v = match key {
            "anilist" | "year" => {
                serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!(text))
            }
            _ => json!(text),
        };
        p.insert(key.to_string(), v);
    }
    Ok(json!({"page": page, "props": p, "until": "true", "settle": 150}))
}

/// After a move: wait for the page on screen to be built, then say where the window is.
fn settled(timeout: u64) -> [Value; 2] {
    [
        json!({"until": "control.settled()", "timeout": timeout}),
        json!({"eval": "control.where()"}),
    ]
}

fn plan(cmd: &str, args: &[String], o: &Opts, now_ms: u128) -> Result<Plan, String> {
    let timeout = o.timeout.unwrap_or(DEFAULT_TIMEOUT.as_millis() as u64);
    let one = |steps: Value| Plan {
        steps,
        out: None,
        print: Print::Result,
        budget: timeout,
    };
    // A move waits twice: for the page to be built, then for it to settle
    let moves = |steps: Value| Plan {
        budget: timeout * 2 + 5000,
        ..one(steps)
    };
    let need = |n: usize, usage: &str| -> Result<(), String> {
        if args.len() < n {
            Err(format!("usage: anibeam ctl {usage}"))
        } else {
            Ok(())
        }
    };
    let with_nth = |mut step: Value| {
        if let Some(n) = o.nth {
            step["nth"] = json!(n);
        }
        step
    };
    Ok(match cmd {
        "go" | "open" => {
            if cmd == "go" {
                need(1, "go <page> [words] [key=value ...]")?;
            }
            let mut steps = Vec::new();
            if let Some(page) = args.first() {
                let mut step = page_step(page, &args[1..], o.props.as_deref())?;
                step["timeout"] = json!(timeout);
                steps.push(step);
            }
            steps.extend(settled(timeout));
            moves(Value::Array(steps))
        }
        "back" | "forward" => {
            let mut steps = vec![json!({"eval": format!("frame.item.{cmd}()")})];
            steps.extend(settled(timeout));
            moves(Value::Array(steps))
        }
        "where" => one(json!([{"eval": "control.where()"}])),
        "pages" => one(json!([{"eval": "control.pages()"}])),
        "tree" => Plan {
            steps: json!([{"tree": args.join(" ")}]),
            out: None,
            print: Print::Tree,
            budget: timeout,
        },
        "click" => {
            need(1, "click <target> [--nth n] [--right] [--double]")?;
            let mut step = with_nth(json!({"click": target(&args.join(" "))}));
            if o.right {
                step["button"] = json!("right");
            }
            if o.double {
                step["double"] = json!(true);
            }
            one(json!([step]))
        }
        "hover" => {
            need(1, "hover <target> [--nth n]")?;
            one(json!([with_nth(json!({"hover": target(&args.join(" "))}))]))
        }
        "scroll" => {
            need(2, "scroll <target> <notches>")?;
            let (last, rest) = args.split_last().expect("two arguments");
            let notches: f64 = last
                .parse()
                .map_err(|_| "scroll's last argument is a number of notches".to_string())?;
            let dy = (-notches * 120.0).round() as i64;
            let t = target(&rest.join(" "));
            let step = match t {
                Value::Array(p) => json!({"wheel": [p[0], p[1], dy]}),
                name => json!({"wheel": name, "dy": dy}),
            };
            one(json!([with_nth(step)]))
        }
        "key" => {
            need(1, "key <combo> ...")?;
            one(Value::Array(
                args.iter()
                    .map(|k| {
                        let mut step = json!({"key": k});
                        if o.focus {
                            step["focus"] = json!(true);
                        }
                        step
                    })
                    .collect(),
            ))
        }
        "type" => {
            need(1, "type <text>")?;
            let mut step = json!({"type": args.join(" ")});
            if o.focus {
                step["focus"] = json!(true);
            }
            one(json!([step]))
        }
        "shot" => {
            let file = match args.first() {
                Some(f) => PathBuf::from(f),
                None => std::env::temp_dir().join(format!("anibeam-shot-{now_ms}.png")),
            };
            let file = std::path::absolute(&file).unwrap_or(file);
            let file = if file.extension().is_some_and(|e| e == "png") {
                file
            } else {
                file.with_extension("png")
            };
            let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
            let name = file
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .ok_or("shot takes a file name")?;
            Plan {
                steps: json!([{"shot": name}]),
                out: Some(dir),
                print: Print::Shot(file),
                budget: timeout,
            }
        }
        "eval" => {
            need(1, "eval <js>")?;
            one(json!([{"eval": args.join(" ")}]))
        }
        "wait" => {
            need(1, "wait <js> [--timeout ms]")?;
            one(json!([{"until": args.join(" "), "timeout": timeout}]))
        }
        "run" => {
            need(1, "run <script.json | ->")?;
            let text = if args[0] == "-" {
                let mut s = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut s)
                    .map_err(|e| format!("stdin: {e}"))?;
                s
            } else {
                std::fs::read_to_string(&args[0]).map_err(|e| format!("{}: {e}", args[0]))?
            };
            let steps: Value =
                serde_json::from_str(&text).map_err(|e| format!("{}: bad JSON: {e}", args[0]))?;
            if !steps.is_array() {
                return Err(format!("{}: a script is a JSON array of steps", args[0]));
            }
            let out = std::env::var_os("ANIBEAM_CAPTURES")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir);
            Plan {
                steps,
                out: Some(std::path::absolute(&out).unwrap_or(out)),
                print: Print::Log,
                budget: o.timeout.unwrap_or(600_000),
            }
        }
        "quit" => one(json!([{"eval": "Qt.callLater(Qt.quit)"}])),
        other => {
            return Err(format!(
                "unknown command {other}; `anibeam ctl help` lists them"
            ));
        }
    })
}

/// One tree row as a line: kind, the words in quotes, #objectName, the box, then flags.
fn tree_line(row: &Value) -> String {
    let mut s = row["kind"].as_str().unwrap_or("?").to_string();
    if let Some(t) = row["text"].as_str() {
        s.push_str(&format!(" {}", Value::from(t)));
    }
    if let Some(n) = row["name"].as_str() {
        s.push_str(&format!(" #{n}"));
    }
    let n = |k: &str| row[k].as_i64().unwrap_or(0);
    s.push_str(&format!(" {},{} {}x{}", n("x"), n("y"), n("w"), n("h")));
    if row["on"] == json!(true) {
        s.push_str(" on");
    }
    if row["focus"] == json!(true) {
        s.push_str(" focus");
    }
    s
}

/// One line to stdout. A reader that went away (`| head`) ends the output, not the program.
fn out(line: impl std::fmt::Display) {
    use std::io::Write;
    let _ = writeln!(std::io::stdout().lock(), "{line}");
}

fn print_value(v: &Value) {
    match v {
        Value::Null => {}
        Value::String(s) => out(s),
        other => out(other),
    }
}

/// `anibeam ctl ...`: the exit code. 0 done, 1 failed, 2 a usage mistake, 3 no window.
pub fn cli(argv: &[String]) -> i32 {
    let (args, o) = match split_opts(argv) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("anibeam ctl: {e}");
            return 2;
        }
    };
    let Some((cmd, rest)) = args.split_first() else {
        out(HELP.trim_end());
        return 0;
    };
    if matches!(cmd.as_str(), "help" | "--help" | "-h") {
        out(HELP.trim_end());
        return 0;
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let plan = match plan(cmd, rest, &o, now_ms) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("anibeam ctl: {e}");
            return 2;
        }
    };
    let socket = crate::runtime::paths().control_socket();
    let wait = Duration::from_millis(plan.budget);
    if cmd == "open"
        && let Err(e) = ensure_running(&socket, o.offscreen)
    {
        eprintln!("anibeam ctl: {e}");
        return 1;
    }
    let mut req = json!({"steps": plan.steps, "timeout": wait.as_millis() as u64});
    if let Some(dir) = &plan.out {
        req["out"] = json!(dir.to_string_lossy());
    }
    let reply = match request(&socket, &req, wait) {
        Ok(r) => r,
        Err(Fail::NotRunning) => {
            eprintln!("anibeam ctl: {}", Fail::NotRunning);
            return 3;
        }
        Err(e) => {
            eprintln!("anibeam ctl: {e}");
            return 1;
        }
    };
    if o.json {
        out(&reply);
        return if reply["ok"] == json!(true) { 0 } else { 1 };
    }
    if reply["ok"] != json!(true) {
        for line in reply["log"].as_array().into_iter().flatten() {
            if let Some(l) = line.as_str() {
                eprintln!("{l}");
            }
        }
        eprintln!(
            "anibeam ctl: {}",
            reply["error"]
                .as_str()
                .unwrap_or("the window gave no reason")
        );
        return 1;
    }
    match plan.print {
        Print::Result => print_value(&reply["result"]),
        Print::Tree => {
            for row in reply["result"].as_array().into_iter().flatten() {
                out(tree_line(row));
            }
        }
        Print::Shot(file) => out(file.display()),
        Print::Log => {
            for line in reply["log"].as_array().into_iter().flatten() {
                if let Some(l) = line.as_str() {
                    out(l);
                }
            }
            print_value(&reply["result"]);
        }
    }
    0
}

/// Moves the open window to `page`, for `anibeam --page` while one runs. `props` is the
/// JSON object `--props` gave.
pub fn go(socket: &Path, page: &str, props: Option<&str>) -> Result<(), String> {
    // No control socket there yet: the window is raised and stays where it was, as before
    if cfg!(not(unix)) {
        return Ok(());
    }
    let t = DEFAULT_TIMEOUT.as_millis() as u64;
    let mut step = page_step(page, &[], props)?;
    step["timeout"] = json!(t);
    let mut steps = vec![step];
    steps.extend(settled(t));
    let wait = Duration::from_millis(t * 2 + 5000);
    let req = json!({"steps": steps, "timeout": wait.as_millis() as u64});
    let reply = request(socket, &req, wait).map_err(|e| e.to_string())?;
    if reply["ok"] == json!(true) {
        Ok(())
    } else {
        Err(reply["error"]
            .as_str()
            .unwrap_or("no reason given")
            .to_string())
    }
}

/// Starts a window for this root when none answers, and waits until it does.
fn ensure_running(socket: &Path, offscreen: bool) -> Result<(), String> {
    fits(socket)?;
    if offscreen && crate::runtime::args().root.is_none() {
        // A hidden window on the real library would hold the single-instance lock, and the
        // app would seem not to open from the menu
        return Err("--offscreen needs --root <dir>".into());
    }
    if alive(socket) {
        return Ok(());
    }
    // The AppImage file, not its mount: the mount goes once this process exits
    let exe = std::env::var_os("APPIMAGE")
        .filter(|a| !a.is_empty())
        .map(PathBuf::from)
        .map_or_else(std::env::current_exe, Ok)
        .map_err(|e| format!("cannot find this program: {e}"))?;
    let log = socket.with_file_name("anibeam-ctl-open.log");
    if let Some(dir) = log.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let err = std::fs::File::create(&log).map_err(|e| format!("{}: {e}", log.display()))?;
    let mut cmd = std::process::Command::new(exe);
    if let Some(root) = crate::runtime::args().root.as_deref() {
        cmd.arg("--root").arg(root);
    }
    cmd.stdin(std::process::Stdio::null())
        .stdout(err.try_clone().map_err(|e| e.to_string())?)
        .stderr(err);
    if offscreen {
        // As shoot.sh and drive.sh run it: no window on the desktop, the GPU still paints
        // posters when there is a display to take a context from, and no sound.
        cmd.env("QT_QPA_PLATFORM", "offscreen")
            .env("ANIBEAM_SILENT", "1");
        if std::env::var_os("DISPLAY").is_some() {
            cmd.env("QT_QUICK_BACKEND", "rhi")
                .env("QSG_RHI_BACKEND", "opengl");
        }
    }
    #[cfg(unix)]
    {
        // Its own process group, so a Ctrl+C in the shell that ran ctl leaves it open
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("cannot start Anibeam: {e}"))?;
    let until = std::time::Instant::now() + Duration::from_secs(60);
    while std::time::Instant::now() < until {
        if alive(socket) {
            return Ok(());
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!(
                "Anibeam exited ({status}) before it answered; its log is {}",
                log.display()
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!(
        "Anibeam did not answer within a minute; its log is {}",
        log.display()
    ))
}

/// The room in `sun_path`, terminator included: 108 bytes on Linux, 104 on a Mac.
const SOCKET_PATH_MAX: usize = if cfg!(target_os = "macos") { 104 } else { 108 };

/// Whether a socket can live at `socket`: a deep `--root` can be too long for one.
fn fits(socket: &Path) -> Result<(), String> {
    if socket.as_os_str().len() >= SOCKET_PATH_MAX {
        return Err(format!(
            "{} is too long for a socket; pick a shorter --root",
            socket.display()
        ));
    }
    Ok(())
}

#[cfg(unix)]
pub use unix::{alive, reply, request, start};

#[cfg(not(unix))]
pub fn alive(_socket: &Path) -> bool {
    false
}
#[cfg(not(unix))]
pub fn request(_socket: &Path, _req: &Value, _wait: Duration) -> Result<Value, Fail> {
    Err(Fail::Other(
        "anibeam ctl needs a Unix socket, which this build has not got yet".into(),
    ))
}
#[cfg(not(unix))]
pub fn reply(_id: i32, _text: String) {}

#[cfg(unix)]
mod unix {
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::Path;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::mpsc::{self, Sender};
    use std::time::Duration;

    use cxx_qt_lib::QString;
    use serde_json::{Value, json};

    use super::Fail;

    /// The longest request line taken: a long drive script fits many times over.
    const REQUEST_MAX: u64 = 4 << 20;
    /// Beyond the request's own budget, the time the window gets to answer.
    const GRACE: Duration = Duration::from_secs(3);

    /// Requests waiting on the window, by id.
    static PENDING: Mutex<Option<HashMap<i32, Sender<String>>>> = Mutex::new(None);
    static NEXT: AtomicI32 = AtomicI32::new(1);

    type Thread = cxx_qt::CxxQtThread<crate::bridge::shell::qobject::Shell>;

    /// Listens on `socket` for the life of the process. The caller holds the single-instance
    /// lock, so a socket file already there was left by a window that died, and is cleared.
    pub fn start(socket: &Path, shell: Thread) {
        let listener = match bind(socket) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("anibeam: control: {e}");
                return;
            }
        };
        std::thread::spawn(move || {
            for conn in listener.incoming().flatten() {
                let shell = shell.clone();
                std::thread::spawn(move || answer(conn, shell));
            }
        });
    }

    fn bind(socket: &Path) -> Result<UnixListener, String> {
        if let Some(dir) = socket.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        super::fits(socket)?;
        let _ = std::fs::remove_file(socket);
        let l = UnixListener::bind(socket).map_err(|e| format!("{}: {e}", socket.display()))?;
        // This user alone: a request runs any JS in the window
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(socket, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("{}: {e}", socket.display()))?;
        Ok(l)
    }

    fn answer(mut conn: UnixStream, shell: Thread) {
        let mut line = String::new();
        let read = BufReader::new((&conn).take(REQUEST_MAX)).read_line(&mut line);
        let text = match read {
            Ok(0) | Err(_) => return,
            Ok(_) => dispatch(line.trim(), &shell),
        };
        let _ = conn.write_all(format!("{text}\n").as_bytes());
    }

    /// Hands one request to the window and waits for its reply.
    fn dispatch(line: &str, shell: &Thread) -> String {
        let req: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => return json!({"ok": false, "error": format!("bad JSON: {e}")}).to_string(),
        };
        if !req["steps"].is_array() {
            return json!({"ok": false, "error": "a request is {\"steps\": [...]}"}).to_string();
        }
        let wait = req["timeout"]
            .as_u64()
            .map_or(super::DEFAULT_TIMEOUT, Duration::from_millis)
            + GRACE;
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        PENDING
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get_or_insert_with(HashMap::new)
            .insert(id, tx);
        let text = QString::from(line);
        let queued = shell.queue(move |shell| shell.control_requested(id, text));
        let out = if queued.is_err() {
            json!({"ok": false, "error": "the window is closing"}).to_string()
        } else {
            rx.recv_timeout(wait).unwrap_or_else(|_| {
                // Stopped, so it neither runs on nor holds up the requests behind it
                let _ = shell.queue(move |shell| shell.control_cancelled(id));
                json!({"ok": false, "error": format!("no answer within {} ms", wait.as_millis())})
                    .to_string()
            })
        };
        if let Some(p) = PENDING.lock().unwrap_or_else(|p| p.into_inner()).as_mut() {
            p.remove(&id);
        }
        out
    }

    /// Control.qml's answer to request `id`. A request that gave up waiting has gone.
    pub fn reply(id: i32, text: String) {
        let tx = PENDING
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_mut()
            .and_then(|p| p.remove(&id));
        if let Some(tx) = tx {
            let _ = tx.send(text);
        }
    }

    /// Whether a window answers on `socket`.
    pub fn alive(socket: &Path) -> bool {
        UnixStream::connect(socket).is_ok()
    }

    /// One request, one reply.
    pub fn request(socket: &Path, req: &Value, wait: Duration) -> Result<Value, Fail> {
        super::fits(socket).map_err(Fail::Other)?;
        let mut conn = UnixStream::connect(socket).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused => {
                Fail::NotRunning
            }
            _ => Fail::Other(format!("{}: {e}", socket.display())),
        })?;
        conn.set_read_timeout(Some(wait + GRACE * 2))
            .map_err(|e| Fail::Other(e.to_string()))?;
        conn.write_all(format!("{req}\n").as_bytes())
            .map_err(|e| Fail::Other(format!("sending: {e}")))?;
        let mut line = String::new();
        BufReader::new(&conn)
            .read_line(&mut line)
            .map_err(|e| Fail::Other(format!("no reply: {e}")))?;
        if line.trim().is_empty() {
            return Err(Fail::Other("the window closed the connection".into()));
        }
        serde_json::from_str(line.trim()).map_err(|e| Fail::Other(format!("bad reply: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }
    fn steps(cmd: &str, args: &[&str]) -> Value {
        let (rest, o) = split_opts(&s(args)).unwrap();
        plan(cmd, &rest, &o, 7).unwrap().steps
    }

    #[test]
    fn go_fills_the_page_s_main_prop_and_takes_key_values() {
        let st = steps("go", &["settings", "appearance"]);
        assert_eq!(st[0]["page"], "settings");
        assert_eq!(st[0]["props"], json!({"tab": "appearance"}));
        assert_eq!(st[1]["until"], "control.settled()");
        assert_eq!(st[2]["eval"], "control.where()");
        let st = steps("go", &["series", "made", "in", "abyss"]);
        assert_eq!(st[0]["props"], json!({"find": "made in abyss"}));
        let st = steps("go", &["series", "id=12"]);
        assert_eq!(st[0]["props"], json!({"id": 12}));
        let st = steps(
            "go",
            &["library", "q=a=b", "--props", "{\"sort\":\"Alpha\"}"],
        );
        assert_eq!(st[0]["props"], json!({"q": "a=b", "sort": "Alpha"}));
        let st = steps("go", &["show", "21"]);
        assert_eq!(st[0]["props"], json!({"anilist": 21}));
        let (rest, o) = split_opts(&s(&["settings", "--", "--focus"])).unwrap();
        assert_eq!(
            plan("go", &rest, &o, 0).unwrap().steps[0]["props"],
            json!({"tab": "--focus"})
        );
        let (rest, o) = split_opts(&s(&["home", "hello"])).unwrap();
        assert!(
            plan("go", &rest, &o, 0)
                .unwrap_err()
                .contains("takes no words")
        );
        assert!(plan("go", &[], &Opts::default(), 0).is_err());
    }

    #[test]
    fn open_without_a_page_only_waits_for_the_window() {
        let st = steps("open", &[]);
        assert_eq!(st.as_array().unwrap().len(), 2);
        assert_eq!(steps("open", &["feed"])[0]["page"], "feed");
    }

    #[test]
    fn targets_are_points_names_or_words() {
        assert_eq!(steps("click", &["10,20"])[0]["click"], json!([10.0, 20.0]));
        assert_eq!(
            steps("click", &["Rail-library"])[0]["click"],
            "Rail-library"
        );
        let st = steps(
            "click",
            &["Mark", "watched", "--nth", "2", "--right", "--double"],
        );
        assert_eq!(
            st[0],
            json!({"click": "Mark watched", "nth": 2, "button": "right", "double": true})
        );
        assert_eq!(
            steps("scroll", &["100,200", "3"])[0],
            json!({"wheel": [100.0, 200.0, -360]})
        );
        assert_eq!(
            steps("scroll", &["Episodes", "-1"])[0],
            json!({"wheel": "Episodes", "dy": 120})
        );
    }

    #[test]
    fn keys_types_waits_and_evals() {
        assert_eq!(
            steps("key", &["Escape", "Ctrl+K"]),
            json!([{"key": "Escape"}, {"key": "Ctrl+K"}])
        );
        assert_eq!(
            steps("type", &["hello", "there"]),
            json!([{"type": "hello there"}])
        );
        assert_eq!(
            steps("type", &["--focus", "--", "--right"]),
            json!([{"type": "--right", "focus": true}])
        );
        assert_eq!(
            steps("key", &["a", "--focus"]),
            json!([{"key": "a", "focus": true}])
        );
        assert_eq!(
            steps("wait", &["Door.ready", "--timeout", "900"]),
            json!([{"until": "Door.ready", "timeout": 900}])
        );
        assert_eq!(steps("eval", &["1", "+", "1"]), json!([{"eval": "1 + 1"}]));
        assert_eq!(steps("tree", &[]), json!([{"tree": ""}]));
    }

    #[test]
    fn a_move_and_a_script_get_room_for_their_waits() {
        let t = DEFAULT_TIMEOUT.as_millis() as u64;
        let budget = |cmd: &str, args: &[&str]| {
            let (rest, o) = split_opts(&s(args)).unwrap();
            plan(cmd, &rest, &o, 0).unwrap().budget
        };
        assert_eq!(budget("where", &[]), t);
        assert_eq!(budget("go", &["home"]), t * 2 + 5000);
        assert_eq!(budget("back", &["--timeout", "1000"]), 7000);
        assert_eq!(budget("eval", &["1", "--timeout", "50"]), 50);
    }

    #[test]
    fn a_shot_lands_where_it_is_asked_or_in_temp() {
        let dir = std::env::temp_dir().join("x");
        let asked = dir.join("page").to_string_lossy().into_owned();
        let p = plan("shot", &[asked], &Opts::default(), 0).unwrap();
        assert_eq!(p.steps, json!([{"shot": "page"}]));
        assert_eq!(p.out.as_deref(), Some(dir.as_path()));
        assert_eq!(p.print, Print::Shot(dir.join("page.png")));
        let p = plan("shot", &[], &Opts::default(), 42).unwrap();
        assert_eq!(
            p.print,
            Print::Shot(std::env::temp_dir().join("anibeam-shot-42.png"))
        );
    }

    #[test]
    fn unknown_commands_and_bad_flags_are_usage_errors() {
        assert!(plan("fly", &[], &Opts::default(), 0).is_err());
        assert!(split_opts(&s(&["--timeout", "soon"])).is_err());
        assert!(plan("scroll", &s(&["Episodes", "lots"]), &Opts::default(), 0).is_err());
    }

    #[test]
    fn a_tree_row_prints_on_one_line() {
        let row = json!({"kind": "Button", "text": "Play \"now\"", "name": "play", "x": 1, "y": 2, "w": 30, "h": 40, "on": true});
        assert_eq!(
            tree_line(&row),
            r#"Button "Play \"now\"" #play 1,2 30x40 on"#
        );
    }

    #[cfg(unix)]
    #[test]
    fn no_window_is_not_running() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("c.sock");
        assert_eq!(
            request(&sock, &json!({"steps": []}), Duration::from_millis(100)).unwrap_err(),
            Fail::NotRunning
        );
        // A socket file a dead window left behind refuses the connection
        drop(std::os::unix::net::UnixListener::bind(&sock).unwrap());
        assert_eq!(
            request(&sock, &json!({"steps": []}), Duration::from_millis(100)).unwrap_err(),
            Fail::NotRunning
        );
        assert!(!alive(&sock));
    }

    #[cfg(unix)]
    #[test]
    fn a_request_is_one_line_out_and_one_back() {
        use std::io::{BufRead, BufReader, Write};
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("c.sock");
        let l = std::os::unix::net::UnixListener::bind(&sock).unwrap();
        let t = std::thread::spawn(move || {
            let (mut c, _) = l.accept().unwrap();
            let mut line = String::new();
            BufReader::new(&c).read_line(&mut line).unwrap();
            let req: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(req["steps"][0]["eval"], "1");
            c.write_all(b"{\"ok\":true,\"result\":1,\"log\":[]}\n")
                .unwrap();
        });
        let r = request(
            &sock,
            &json!({"steps": [{"eval": "1"}]}),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(r["result"], 1);
        t.join().unwrap();
    }
}
