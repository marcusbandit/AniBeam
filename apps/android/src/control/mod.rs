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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use command::Command;
pub use ui::{rendered, rendering};

/// How long a command may keep the UI loop before the answer says it did not come back.
const ANSWER: Duration = Duration::from_secs(10);
/// How long a whole line may take. `am broadcast` waits on an ordered background broadcast,
/// and Android calls a receiver that keeps it past 60 s not responding.
const LINE: Duration = Duration::from_secs(45);
/// The most an answer carries. It goes back to the system in one binder call, and past the
/// binder's buffer that call fails, the broadcast never finishes and it ends in an ANR.
const MOST: usize = 200_000;

/// The time a line has left.
#[derive(Clone, Copy)]
struct Budget(Instant);

impl Budget {
    fn left(self) -> Duration {
        self.0.saturating_duration_since(Instant::now())
    }

    /// `d`, or less when the line has less left.
    fn cap(self, d: Duration) -> Duration {
        d.min(self.left())
    }
}

/// Starts the channel: the window the commands act on, and on Android the receiver.
pub fn install(window: &crate::AppWindow) {
    ui::install(window);
    #[cfg(target_os = "android")]
    android::start();
}

/// Runs a command line and answers with what each command printed, one after another. A
/// command that fails stops the line, and its `error: ...` is the answer's last line. The
/// line has `LINE` in all: a command that would run past it is not started.
pub fn serve(line: &str) -> String {
    let commands = match command::parse_line(line) {
        Ok(c) => c,
        Err(e) => return format!("error: {e}"),
    };
    let budget = Budget(Instant::now() + LINE);
    let total = commands.len();
    let mut out = Vec::new();
    for (i, c) in commands.into_iter().enumerate() {
        let said = match worst(&c) {
            Some(need) if need > budget.left() => format!(
                "error: the line would run past its {} s; {} of {total} not run, send them on a line of their own",
                LINE.as_secs(),
                total - i
            ),
            _ if budget.left().is_zero() => format!(
                "error: the line ran out of its {} s; {} of {total} not run",
                LINE.as_secs(),
                total - i
            ),
            _ => run(c, budget),
        };
        let failed = said.starts_with("error:");
        out.push(said);
        if failed {
            break;
        }
    }
    cut(out.join("\n"))
}

/// How long a command can take at most, for the ones that take long on purpose.
fn worst(c: &Command) -> Option<Duration> {
    match c {
        Command::Wait { secs, .. } | Command::Perf(secs) => {
            Some(Duration::from_secs_f32(*secs) + Duration::from_secs(1))
        }
        Command::Swipe { ms, .. } => Some(Duration::from_millis(*ms)),
        _ => None,
    }
}

/// The answer, cut at a line under `MOST` bytes when it is longer.
fn cut(answer: String) -> String {
    if answer.len() <= MOST {
        return answer;
    }
    let mut end = MOST;
    while !answer.is_char_boundary(end) {
        end -= 1;
    }
    let end = answer[..end].rfind('\n').unwrap_or(end);
    let left = answer[end..].lines().filter(|l| !l.is_empty()).count();
    // An error that was the last line stays the last line, so the result code holds.
    let error = answer
        .lines()
        .last()
        .filter(|l| l.starts_with("error:"))
        .map(|l| format!("\n{l}"))
        .unwrap_or_default();
    format!(
        "{}\n... {left} more lines; narrow it with a filter{error}",
        &answer[..end]
    )
}

/// Runs `f` on the UI loop and waits up to `timeout` for what it sends back. When the wait
/// gives up, `f` still sits in the loop's queue; it is skipped when its turn comes, so a
/// caller that retries after "did not answer" never gets the action twice.
fn on_ui<T: Send + 'static>(
    timeout: Duration,
    f: impl FnOnce(mpsc::Sender<T>) + Send + 'static,
) -> Result<T, String> {
    let (tx, rx) = mpsc::channel();
    let gave_up = Arc::new(AtomicBool::new(false));
    let late = gave_up.clone();
    slint::invoke_from_event_loop(move || {
        if late.load(Ordering::Acquire) {
            crate::local::log("control: skipped a command its caller had given up on");
            return;
        }
        f(tx)
    })
    .map_err(|e| format!("error: the UI loop is gone: {e}"))?;
    rx.recv_timeout(timeout).map_err(|_| {
        gave_up.store(true, Ordering::Release);
        format!(
            "error: the UI thread did not answer within {:.0} s; the command was dropped",
            timeout.as_secs_f32()
        )
    })
}

fn run(command: Command, budget: Budget) -> String {
    match command {
        Command::Wait { label, secs } => wait(&label, secs, budget),
        Command::State => state(budget),
        Command::Perf(secs) => perf(secs, budget),
        other => {
            let timeout = match &other {
                Command::Swipe { ms, .. } => ANSWER + Duration::from_millis(*ms),
                _ => ANSWER,
            };
            on_ui(budget.cap(timeout), move |tx| ui::exec(other, tx)).unwrap_or_else(|e| e)
        }
    }
}

/// Looks for the label twice a second, until a control with it shows. Each look walks the
/// window on the UI thread, so it looks no more often than that.
fn wait(label: &str, secs: f32, budget: Budget) -> String {
    let start = Instant::now();
    let until = Duration::from_secs_f32(secs);
    loop {
        let want = label.to_string();
        match on_ui(budget.cap(ANSWER), move |tx| {
            let _ = tx.send(ui::probe(&want));
        }) {
            Ok(Some(line)) => {
                return format!("{line} after {:.1} s", start.elapsed().as_secs_f32());
            }
            Ok(None) if start.elapsed() < until => {
                std::thread::sleep(Duration::from_millis(500));
            }
            Ok(None) => return format!("error: {label:?} did not show within {secs:.0} s"),
            Err(e) => return e,
        }
    }
}

/// How long a call to the UI loop waits before it runs: how busy the UI thread is.
fn ping(budget: Budget) -> Result<Duration, String> {
    let sent = Instant::now();
    on_ui(budget.cap(ANSWER), move |tx| {
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

fn state(budget: Budget) -> String {
    let ui = match on_ui(budget.cap(ANSWER), |tx| {
        let _ = tx.send(ui::state_lines());
    }) {
        Ok(lines) => lines,
        Err(e) => return e,
    };
    let mut out = vec![ui];
    out.push(match ping(budget) {
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
fn perf(secs: f32, budget: Budget) -> String {
    if let Err(e) = on_ui(budget.cap(ANSWER), |tx| {
        ui::frames_mark();
        let _ = tx.send(());
    }) {
        return e;
    }
    // The CPU times and the span they are divided by start at the same moment, after the
    // mark, which a busy UI thread can hold up for seconds.
    let start = Instant::now();
    let before = thread_times();
    let span = Duration::from_secs_f32(secs);
    let mut waits = Vec::new();
    while start.elapsed() < span {
        match ping(budget) {
            Ok(d) => waits.push(d),
            Err(e) => return e,
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let after = thread_times();
    let elapsed = start.elapsed().as_secs_f32();
    let frames = on_ui(budget.cap(ANSWER), |tx| {
        let _ = tx.send(ui::frames_since_mark());
    })
    .unwrap_or_else(|e| e);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_answer_is_cut_at_a_line_and_keeps_its_error_last() {
        assert_eq!(cut("short".into()), "short");
        let mut long: Vec<String> = (0..30_000)
            .map(|i| format!("  text \"row {i}\" at 1,2"))
            .collect();
        long.push("error: the last one failed".into());
        let out = cut(long.join("\n"));
        assert!(out.len() < MOST + 200, "{}", out.len());
        assert!(out.contains("more lines; narrow it with a filter"));
        assert!(out.ends_with("\nerror: the last one failed"));
        assert!(out.lines().all(|l| !l.ends_with("at 1,")));
    }

    #[test]
    fn the_long_commands_say_how_long_they_may_take() {
        let wait = Command::Wait {
            label: "x".into(),
            secs: 30.0,
        };
        assert_eq!(worst(&wait), Some(Duration::from_secs(31)));
        assert_eq!(worst(&Command::Perf(5.0)), Some(Duration::from_secs(6)));
        assert_eq!(worst(&Command::Where), None);
        // Two of the longest waits cannot both fit in one line.
        assert!(2 * worst(&wait).unwrap() > LINE);
        let b = Budget(Instant::now() + Duration::from_secs(3));
        assert!(b.cap(ANSWER) <= Duration::from_secs(3));
    }
}
