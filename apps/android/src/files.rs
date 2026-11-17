//! Android's file picker, for the backup and the restore. `PickActivity` asks for the file
//! and copies it between the picked place and a file the app owns, then leaves one word in a
//! result file; a timer looks for that word until it comes. Off the phone nothing can be
//! picked.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How a pick ended.
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Done,
    Cancelled,
    Failed(String),
}

/// Where the file goes or comes from.
#[derive(Clone, Copy, PartialEq)]
pub enum Pick {
    /// The app's file is copied to a place the person picks, named `name` to begin with.
    Save,
    /// A file the person picks is copied over the app's file.
    Open,
}

/// How long a picker may stay open before the app stops looking.
const PATIENCE: Duration = Duration::from_secs(15 * 60);

struct Waiting {
    result: PathBuf,
    since: Instant,
    done: Box<dyn FnOnce(Outcome)>,
    timer: slint::Timer,
}

thread_local! {
    static WAITING: RefCell<Option<Waiting>> = const { RefCell::new(None) };
}

/// What the result file says, if it says anything yet.
fn read(result: &Path) -> Option<Outcome> {
    let word = std::fs::read_to_string(result).ok()?;
    // The picker creates the file before it writes the word: empty is not yet.
    if word.trim().is_empty() {
        return None;
    }
    Some(match word.trim() {
        "ok" => Outcome::Done,
        "cancel" => Outcome::Cancelled,
        other => Outcome::Failed(other.strip_prefix("error: ").unwrap_or(other).to_string()),
    })
}

/// Opens the picker and calls `done` on the UI thread with how it ended. One pick at a
/// time; a second while one is open is refused.
pub fn pick(kind: Pick, path: &Path, name: &str, done: impl FnOnce(Outcome) + 'static) {
    if WAITING.with(|w| w.borrow().is_some()) {
        fail_later(done, tr!("files-already-picking"));
        return;
    }
    let result = path.with_extension("result");
    let _ = std::fs::remove_file(&result);
    if let Err(e) = crate::platform::pick_file(kind, path, name, &result) {
        fail_later(done, e);
        return;
    }
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(300), look);
    WAITING.with(|w| {
        *w.borrow_mut() = Some(Waiting {
            result,
            since: Instant::now(),
            done: Box::new(done),
            timer,
        })
    });
}

/// A pick that could not start says so on the next turn of the event loop, never inside the
/// call that asked: that call may hold the page's state.
fn fail_later(done: impl FnOnce(Outcome) + 'static, why: String) {
    slint::Timer::single_shot(Duration::from_millis(0), move || done(Outcome::Failed(why)));
}

/// One look at the result file. Nothing comes back from the picker but this file, so the
/// timer is the only thing that notices the picker closed.
fn look() {
    let ended = WAITING.with(|w| {
        let w = w.borrow();
        let waiting = w.as_ref()?;
        read(&waiting.result).or_else(|| {
            (waiting.since.elapsed() > PATIENCE)
                .then(|| Outcome::Failed(tr!("files-took-too-long")))
        })
    });
    if let Some(outcome) = ended
        && let Some(waiting) = WAITING.with(|w| w.borrow_mut().take())
    {
        waiting.timer.stop();
        let _ = std::fs::remove_file(&waiting.result);
        (waiting.done)(outcome);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_picker_leaves_one_word() {
        let dir = std::env::temp_dir().join(format!("anibeam-pick-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("result");
        assert_eq!(read(&file), None);
        std::fs::write(&file, "ok").unwrap();
        assert_eq!(read(&file), Some(Outcome::Done));
        // Created and not written yet.
        std::fs::write(&file, "").unwrap();
        assert_eq!(read(&file), None);
        std::fs::write(&file, "cancel").unwrap();
        assert_eq!(read(&file), Some(Outcome::Cancelled));
        std::fs::write(&file, "error: no room").unwrap();
        assert_eq!(read(&file), Some(Outcome::Failed("no room".into())));
        let _ = std::fs::remove_dir_all(dir);
    }
}
