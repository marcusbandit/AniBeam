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
        fail_later(done, "A file is already being picked.".into());
        return;
    }
    let result = path.with_extension("result");
    let _ = std::fs::remove_file(&result);
    if let Err(e) = launch(kind, path, name, &result) {
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
                .then(|| Outcome::Failed("The file picker took too long.".into()))
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

#[cfg(target_os = "android")]
fn launch(kind: Pick, path: &Path, name: &str, result: &Path) -> Result<(), String> {
    use jni::JavaVM;
    use jni::objects::{JObject, JValue};
    let go = || -> Result<(), Box<dyn std::error::Error>> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
        let mut env = vm.attach_current_thread()?;
        let app = unsafe { JObject::from_raw(ctx.context().cast()) };
        let started = env.with_local_frame(16, |env| -> Result<(), Box<dyn std::error::Error>> {
            let intent = env.new_object("android/content/Intent", "()V", &[])?;
            let pkg = env.new_string("com.marcusrosado.anibeam")?;
            let class = env.new_string("com.marcusrosado.anibeam.PickActivity")?;
            env.call_method(
                &intent,
                "setClassName",
                "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                &[JValue::Object(&pkg), JValue::Object(&class)],
            )?;
            for (key, value) in [
                ("path", path.to_string_lossy().into_owned()),
                ("name", name.to_string()),
                ("result", result.to_string_lossy().into_owned()),
            ] {
                let key = env.new_string(key)?;
                let value = env.new_string(value)?;
                env.call_method(
                    &intent,
                    "putExtra",
                    "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                    &[JValue::Object(&key), JValue::Object(&value)],
                )?;
            }
            let key = env.new_string("mode")?;
            env.call_method(
                &intent,
                "putExtra",
                "(Ljava/lang/String;I)Landroid/content/Intent;",
                &[
                    JValue::Object(&key),
                    JValue::Int(i32::from(kind == Pick::Open)),
                ],
            )?;
            // FLAG_ACTIVITY_NEW_TASK: the context is the Application.
            env.call_method(
                &intent,
                "addFlags",
                "(I)Landroid/content/Intent;",
                &[JValue::Int(0x1000_0000)],
            )?;
            env.call_method(
                &app,
                "startActivity",
                "(Landroid/content/Intent;)V",
                &[JValue::Object(&intent)],
            )?;
            Ok(())
        });
        // A Java exception left pending would fail the next call on this thread, so it is
        // cleared whether or not the frame got to its end.
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
            return Err("Java threw".into());
        }
        started
    };
    go().map_err(|e| format!("The file picker did not open: {e}"))
}

#[cfg(not(target_os = "android"))]
fn launch(_kind: Pick, _path: &Path, _name: &str, _result: &Path) -> Result<(), String> {
    Err("There is no file picker here.".into())
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
