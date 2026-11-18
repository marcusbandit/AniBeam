//! Control over adb (#1207): an agent drives the running app by name instead of by tapping
//! coordinates off a screenshot, and reads back what the screen shows. Slint hands Android
//! no accessibility tree, so `uiautomator` sees one opaque surface.
//!
//! `adb shell am broadcast -a com.marcusrosado.anibeam.CONTROL -p com.marcusrosado.anibeam
//! --es cmd '<command>'` reaches `Control.java`, which the sideload build registers at run
//! time with DUMP as the permission a sender must hold: only the shell user (adb) and the
//! system have it. Java hands the line to `serve` on a thread of its own, and the answer
//! goes back as the broadcast's result data, which `am broadcast` prints.
//! `scripts/ctl.sh` wraps it.
//!
//! `command.rs` reads the line, `ui.rs` acts on the window on the Slint event loop, and this
//! file runs between the two on the receiver's thread: the commands that wait or measure
//! (`wait`, `state`, `perf`) make their round trips to the UI loop from here, so the UI loop
//! never waits on itself. The Play build compiles none of it (build.rs, `cfg(control)`).

// Only Android has a receiver to call `serve`; the host keeps it all for the tests.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

mod command;
mod ui;

#[cfg(target_os = "android")]
mod android;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use command::Command;
pub use ui::{rendered, rendering};

/// How long a command may keep the UI loop before the answer says it did not come back.
const ANSWER: Duration = Duration::from_secs(10);

/// Starts the channel: the window the commands act on, and on Android the receiver.
pub fn install(window: &crate::AppWindow) {
    ui::install(window);
    #[cfg(target_os = "android")]
    android::start();
}

/// Runs a command line and answers with what each command printed, one after another. A
/// command that fails stops the line, and its `error: ...` is the answer's last line.
pub fn serve(line: &str) -> String {
    let commands = match command::parse_line(line) {
        Ok(c) => c,
        Err(e) => return format!("error: {e}"),
    };
    let mut out = Vec::new();
    for c in commands {
        let said = run(c);
        let failed = said.starts_with("error:");
        out.push(said);
        if failed {
            break;
        }
    }
    out.join("\n")
}

/// Runs `f` on the UI loop and waits up to `timeout` for what it sends back.
fn on_ui<T: Send + 'static>(
    timeout: Duration,
    f: impl FnOnce(mpsc::Sender<T>) + Send + 'static,
) -> Result<T, String> {
    let (tx, rx) = mpsc::channel();
    slint::invoke_from_event_loop(move || f(tx))
        .map_err(|e| format!("error: the UI loop is gone: {e}"))?;
    rx.recv_timeout(timeout).map_err(|_| {
        format!(
            "error: the UI thread did not answer within {:.0} s",
            timeout.as_secs_f32()
        )
    })
}

fn run(command: Command) -> String {
    match command {
        Command::Wait { label, secs } => wait(&label, secs),
        Command::State => state(),
        Command::Perf(secs) => perf(secs),
        other => {
            let timeout = match &other {
                Command::Swipe { ms, .. } => ANSWER + Duration::from_millis(*ms),
                _ => ANSWER,
            };
            on_ui(timeout, move |tx| ui::exec(other, tx)).unwrap_or_else(|e| e)
        }
    }
}

/// Looks for a shown control with the label every quarter second, until it shows.
fn wait(label: &str, secs: f32) -> String {
    let start = Instant::now();
    let until = Duration::from_secs_f32(secs);
    loop {
        let want = label.to_string();
        match on_ui(ANSWER, move |tx| {
            let _ = tx.send(ui::probe(&want));
        }) {
            Ok(Some(line)) => {
                return format!("{line} after {:.1} s", start.elapsed().as_secs_f32());
            }
            Ok(None) if start.elapsed() < until => {
                std::thread::sleep(Duration::from_millis(250));
            }
            Ok(None) => return format!("error: {label:?} did not show within {secs:.0} s"),
            Err(e) => return e,
        }
    }
}

/// How long a call to the UI loop waits before it runs: how busy the UI thread is.
fn ping() -> Result<Duration, String> {
    let sent = Instant::now();
    on_ui(ANSWER, move |tx| {
        let _ = tx.send(sent.elapsed());
    })
}

/// Lines of /proc/self/status, by their names.
fn status(keys: &[&str]) -> Vec<String> {
    let text = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    keys.iter()
        .filter_map(|k| {
            text.lines()
                .find(|l| l.starts_with(&format!("{k}:")))
                .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        })
        .collect()
}

/// What the core is running now.
fn jobs() -> String {
    use anibeam_core::{Call, Reply};
    let Some(core) = crate::local::core() else {
        return "core: not open".into();
    };
    match core.call(Call::ListJobs) {
        Ok(Reply::Jobs { jobs }) if jobs.is_empty() => "core: no jobs running".into(),
        Ok(Reply::Jobs { jobs }) => {
            let mut out = vec![format!("core: {} jobs", jobs.len())];
            for j in jobs {
                let age = j.started_at.elapsed().unwrap_or_default().as_secs_f32();
                let progress = j
                    .progress
                    .map(|p| match p.total {
                        Some(t) => format!(", {} of {t} {}", p.done, p.label),
                        None => format!(", {} {}", p.done, p.label),
                    })
                    .unwrap_or_default();
                out.push(format!("  {:?} #{} for {age:.1} s{progress}", j.kind, j.id));
            }
            out.join("\n")
        }
        Ok(_) => "core: no job list".into(),
        Err(e) => format!("core: jobs unreadable: {e}"),
    }
}

fn state() -> String {
    let ui = match on_ui(ANSWER, |tx| {
        let _ = tx.send(ui::state_lines());
    }) {
        Ok(lines) => lines,
        Err(e) => return e,
    };
    let mut out = vec![ui];
    out.push(match ping() {
        Ok(d) => format!("ui loop: answered in {:.1} ms", d.as_secs_f64() * 1000.0),
        Err(e) => e,
    });
    out.push(status(&["Threads", "VmRSS", "VmHWM"]).join(", "));
    out.push(jobs());
    out.join("\n")
}

/// Every thread's name and CPU time so far, in clock ticks.
fn thread_times() -> Vec<(u32, String, u64)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir("/proc/self/task")
        .into_iter()
        .flatten()
        .flatten()
    {
        let Ok(tid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let path = entry.path();
        let name = std::fs::read_to_string(path.join("comm"))
            .unwrap_or_default()
            .trim()
            .to_string();
        let stat = std::fs::read_to_string(path.join("stat")).unwrap_or_default();
        // The fields after the name, which sits in brackets and may hold blanks: the state,
        // then utime and stime as the 12th and 13th.
        let after = stat.rsplit_once(')').map(|(_, a)| a).unwrap_or("");
        let f: Vec<&str> = after.split_whitespace().collect();
        let ticks = |i: usize| f.get(i).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        out.push((tid, name, ticks(11) + ticks(12)));
    }
    out
}

/// Samples the UI loop for `secs`: how long its calls waited, the frames it drew, and the
/// threads that used the CPU meanwhile.
fn perf(secs: f32) -> String {
    let before = thread_times();
    if let Err(e) = on_ui(ANSWER, |tx| {
        ui::frames_mark();
        let _ = tx.send(());
    }) {
        return e;
    }
    let start = Instant::now();
    let span = Duration::from_secs_f32(secs);
    let mut waits = Vec::new();
    while start.elapsed() < span {
        match ping() {
            Ok(d) => waits.push(d),
            Err(e) => return e,
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let frames = on_ui(ANSWER, |tx| {
        let _ = tx.send(ui::frames_since_mark());
    })
    .unwrap_or_else(|e| e);
    let after = thread_times();
    let elapsed = start.elapsed().as_secs_f32();

    waits.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let avg = waits.iter().map(|d| ms(*d)).sum::<f64>() / waits.len().max(1) as f64;
    let p95 = waits
        .get((waits.len() * 95 / 100).min(waits.len().saturating_sub(1)))
        .copied()
        .unwrap_or_default();
    let max = waits.last().copied().unwrap_or_default();
    let mut out = vec![
        format!("perf over {elapsed:.1} s"),
        format!(
            "ui loop: {} calls, waited avg {avg:.1} ms, p95 {:.1} ms, max {:.1} ms",
            waits.len(),
            ms(p95),
            ms(max)
        ),
        frames,
    ];
    // Linux counts CPU time in ticks of 1/100 s on every Android ABI.
    let mut busy: Vec<(String, u32, u64)> = after
        .iter()
        .map(|(tid, name, t)| {
            let was = before
                .iter()
                .find(|(b, _, _)| b == tid)
                .map(|(_, _, t)| *t)
                .unwrap_or(0);
            (name.clone(), *tid, t.saturating_sub(was) * 10)
        })
        .filter(|(_, _, ms)| *ms > 0)
        .collect();
    busy.sort_by_key(|b| std::cmp::Reverse(b.2));
    out.push(format!(
        "threads: {}, {} used the CPU",
        after.len(),
        busy.len()
    ));
    for (name, tid, cpu) in busy.iter().take(8) {
        out.push(format!(
            "  {name} ({tid}) {cpu} ms cpu, {:.0}%",
            *cpu as f32 / (elapsed * 10.0)
        ));
    }
    out.join("\n")
}
