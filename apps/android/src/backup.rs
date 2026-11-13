//! Data (#748): back up the library to a file, restore one, and clear the image cache. The
//! core does the work as jobs (`Export`, `Import`, `ClearImages`) and says how they ended as
//! events, which `core_event` passes on. A backup is written to a file the app owns, handed
//! to Android's file picker to be saved where the person chooses, and deleted again; a
//! restore copies the picked file in first.

use std::cell::RefCell;
use std::path::PathBuf;

use anibeam_core::{Call, EventBody, ImportSummary, JobKind, Reply};
use slint::{ComponentHandle, Weak};

use crate::files::{self, Outcome, Pick};
use crate::{AppWindow, BackupData};

/// What the page is waiting for the core to finish.
#[derive(Clone, Copy, PartialEq)]
enum Job {
    Export,
    Import,
    Clear,
}

/// The line the page shows under the buttons. Kept as what happened, not as words, so a
/// change of language words it again.
#[derive(Clone, Debug, PartialEq)]
enum Say {
    Nothing,
    BackedUp,
    NotSaved,
    Removed(u64),
    Summary(ImportSummary),
    Starting,
    /// A reason the core or the file picker gave, shown as it came.
    Failed(String),
}

struct Page {
    window: Weak<AppWindow>,
    folder: PathBuf,
    waiting: Option<Job>,
    storage: Option<(u64, u64)>,
    saying: Say,
}

thread_local! {
    static PAGE: RefCell<Option<Page>> = const { RefCell::new(None) };
}

fn with(f: impl FnOnce(&mut Page)) {
    PAGE.with(|cell| {
        if let Some(p) = cell.borrow_mut().as_mut() {
            f(p);
        }
    });
}

pub fn install(window: &AppWindow, data_dir: &std::path::Path) {
    let folder = data_dir.join("backup");
    // A file left by a run that was killed holds the library, maybe its sign-ins.
    let _ = std::fs::remove_dir_all(&folder);
    PAGE.with(|cell| {
        *cell.borrow_mut() = Some(Page {
            window: window.as_weak(),
            folder,
            waiting: None,
            storage: None,
            saying: Say::Nothing,
        })
    });
    let d = window.global::<BackupData>();
    d.on_back_up(|| with(Page::back_up));
    d.on_restore(|| with(Page::restore));
    d.on_clear_images(|| with(Page::clear));
    d.on_read(|| with(Page::measure));
}

/// The language changed: the lines on the page are worded again.
pub fn relocalize() {
    with(|p| {
        p.render();
        p.show_storage();
    });
}

/// Every event the core sends, from whichever thread sent it.
pub fn core_event(event: &anibeam_core::Event) {
    let kind = event.job.as_ref().map(|j| j.kind);
    let done = match &event.body {
        EventBody::ExportFinished { .. } => Done::Exported,
        EventBody::ImportFinished { summary } => Done::Imported(summary.clone()),
        EventBody::ImagesCleared { removed } => Done::Cleared(*removed),
        EventBody::JobFailed { error } => match kind {
            Some(JobKind::Export) => Done::Failed(Job::Export, error.to_string()),
            Some(JobKind::Import) => Done::Failed(Job::Import, error.to_string()),
            Some(JobKind::ClearImages) => Done::Failed(Job::Clear, error.to_string()),
            _ => return,
        },
        _ => return,
    };
    let _ = slint::invoke_from_event_loop(move || with(|p| p.finished(done)));
}

enum Done {
    Exported,
    Imported(ImportSummary),
    Cleared(u64),
    Failed(Job, String),
}

impl Page {
    fn data<R>(&self, f: impl FnOnce(BackupData) -> R) -> Option<R> {
        self.window.upgrade().map(|w| f(w.global::<BackupData>()))
    }

    fn back_up(&mut self) {
        if self.waiting.is_some() {
            return;
        }
        let Some(core) = crate::local::core() else {
            return;
        };
        let private = self.data(|d| d.get_include_private()).unwrap_or(false);
        let _ = std::fs::create_dir_all(&self.folder);
        let file = self.folder.join("export.json");
        self.say(Say::Nothing);
        match core.call(Call::Export {
            path: file.to_string_lossy().into_owned(),
            private,
        }) {
            Ok(_) => {
                self.waiting = Some(Job::Export);
                self.data(|d| d.set_busy(true));
            }
            Err(e) => self.say(Say::Failed(e.to_string())),
        }
    }

    /// The backup is written: offer it to the file picker, then delete it either way.
    fn exported(&mut self) {
        let file = self.folder.join("export.json");
        let name = format!(
            "anibeam-backup-{}.json",
            chrono::Local::now().format("%Y-%m-%d")
        );
        let staged = file.clone();
        files::pick(Pick::Save, &file, &name, move |outcome| {
            let _ = std::fs::remove_file(&staged);
            with(|p| {
                p.waiting = None;
                p.data(|d| d.set_busy(false));
                match outcome {
                    Outcome::Done => p.say(Say::BackedUp),
                    Outcome::Cancelled => p.say(Say::NotSaved),
                    Outcome::Failed(e) => p.say(Say::Failed(e)),
                }
            });
        });
    }

    fn restore(&mut self) {
        if self.waiting.is_some() {
            return;
        }
        let _ = std::fs::create_dir_all(&self.folder);
        let file = self.folder.join("import.json");
        self.say(Say::Nothing);
        self.waiting = Some(Job::Import);
        self.data(|d| d.set_busy(true));
        files::pick(Pick::Open, &file, "", move |outcome| {
            with(|p| match outcome {
                Outcome::Done => p.import(),
                Outcome::Cancelled => p.stop(Say::Nothing),
                Outcome::Failed(e) => p.stop(Say::Failed(e)),
            });
        });
    }

    /// The picked file is in: hand it to the core.
    fn import(&mut self) {
        let Some(core) = crate::local::core() else {
            return self.stop(Say::Starting);
        };
        let file = self.folder.join("import.json");
        match core.call(Call::Import {
            path: file.to_string_lossy().into_owned(),
        }) {
            Ok(_) => {}
            Err(e) => self.stop(Say::Failed(e.to_string())),
        }
    }

    /// Ends a backup or restore that is not going on: clears the spinner and says why.
    fn stop(&mut self, say: Say) {
        self.waiting = None;
        let _ = std::fs::remove_dir_all(&self.folder);
        self.data(|d| d.set_busy(false));
        self.say(say);
    }

    fn say(&mut self, say: Say) {
        self.saying = say;
        self.render();
    }

    /// Words what the page last said, in the language the app speaks now.
    fn render(&self) {
        let (last, summary, failed) = match &self.saying {
            Say::Nothing => (String::new(), String::new(), String::new()),
            Say::BackedUp => (tr!("backup-done"), String::new(), String::new()),
            Say::NotSaved => (tr!("backup-not-saved"), String::new(), String::new()),
            Say::Removed(n) => (
                tr!("backup-removed", count = *n),
                String::new(),
                String::new(),
            ),
            Say::Summary(s) => (String::new(), summary_line(s), String::new()),
            Say::Starting => (String::new(), String::new(), tr!("backup-starting")),
            Say::Failed(why) => (String::new(), String::new(), why.clone()),
        };
        self.data(|d| {
            d.set_last(last.into());
            d.set_summary(summary.into());
            d.set_failed(failed.into());
        });
    }

    fn finished(&mut self, done: Done) {
        match done {
            Done::Exported if self.waiting == Some(Job::Export) => self.exported(),
            Done::Imported(summary) if self.waiting == Some(Job::Import) => {
                self.waiting = None;
                let _ = std::fs::remove_dir_all(&self.folder);
                self.data(|d| d.set_busy(false));
                self.say(Say::Summary(summary));
                crate::local::touch();
            }
            Done::Cleared(removed) if self.waiting == Some(Job::Clear) => {
                self.waiting = None;
                self.say(Say::Removed(removed));
                self.measure();
            }
            Done::Failed(job, error) if self.waiting == Some(job) => {
                if job == Job::Export {
                    let _ = std::fs::remove_dir_all(&self.folder);
                }
                self.stop(Say::Failed(error));
            }
            _ => {}
        }
    }

    fn clear(&mut self) {
        let Some(core) = crate::local::core() else {
            return;
        };
        if self.waiting.is_some() {
            return;
        }
        self.say(Say::Nothing);
        match core.call(Call::ClearImages) {
            Ok(_) => self.waiting = Some(Job::Clear),
            Err(e) => self.say(Say::Failed(e.to_string())),
        }
    }

    /// Reads the image cache's size off the core.
    fn measure(&mut self) {
        let Some(core) = crate::local::core() else {
            return;
        };
        let _ = std::thread::Builder::new()
            .name("storage".into())
            .spawn(move || {
                let answer = core.call(Call::GetStorage);
                let _ = slint::invoke_from_event_loop(move || {
                    with(|p| {
                        p.storage = match answer {
                            Ok(Reply::Storage {
                                image_count,
                                image_bytes,
                            }) => Some((image_count, image_bytes)),
                            _ => None,
                        };
                        p.show_storage();
                    })
                });
            });
    }

    fn show_storage(&self) {
        let storage = self.storage;
        self.data(|d| {
            d.set_images_failed(storage.is_none());
            d.set_can_clear(storage.is_some_and(|(n, _)| n > 0));
            d.set_images_line(images_line(storage).into());
            d.set_images_note(
                if storage.is_none() {
                    tr!("backup-images-retry-note")
                } else {
                    String::new()
                }
                .into(),
            );
            d.set_images_question(
                tr!(
                    "backup-images-question",
                    count = storage.map_or(0, |(n, _)| n)
                )
                .into(),
            );
        });
    }
}

fn images_line(storage: Option<(u64, u64)>) -> String {
    match storage {
        None => tr!("backup-images-unmeasured"),
        Some((0, _)) => tr!("backup-images-none"),
        Some((n, bytes)) => tr!("backup-images-line", size = bytes_label(bytes), count = n),
    }
}

fn bytes_label(n: u64) -> String {
    let mut v = n as f64;
    let mut unit = 0;
    while v >= 1024.0 && unit < 3 {
        v /= 1024.0;
        unit += 1;
    }
    anibeam_companion::format::sized(v, unit)
}

/// What a restore did, in the desktop's words. One message per clause, so a language can
/// order and inflect each its own way; the clauses are joined by the catalog's separator.
fn summary_line(s: &ImportSummary) -> String {
    let clauses = [
        tr!("backup-sum-folders", count = s.sources_added),
        tr!("backup-sum-series", count = s.series_created),
        tr!("backup-sum-matches", count = s.matches_applied),
        tr!(
            "backup-sum-views",
            views = s.views_merged,
            episodes = s.completed_merged
        ),
        tr!("backup-sum-resume", count = s.resume_points_merged),
        tr!("backup-sum-accounts", count = s.accounts_imported),
    ];
    let sep = tr!("backup-sum-sep");
    let mut out = tr!("backup-sum-list", items = clauses.join(sep.as_str()));
    if s.fixes_applied > 0 {
        out += " ";
        out += &tr!("backup-sum-fixes", count = s.fixes_applied);
    }
    if s.sources_unavailable > 0 {
        out += " ";
        out += &tr!("backup-sum-missing", count = s.sources_unavailable);
    }
    if !s.fields_ignored.is_empty() {
        out += " ";
        out += &tr!(
            "backup-sum-left-out",
            fields = s.fields_ignored.join(sep.as_str())
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary() -> ImportSummary {
        ImportSummary {
            sources_added: 1,
            sources_unavailable: 0,
            series_created: 12,
            matches_applied: 9,
            views_merged: 40,
            completed_merged: 1,
            resume_points_merged: 3,
            accounts_imported: 0,
            fixes_applied: 0,
            fields_ignored: Vec::new(),
        }
    }

    #[test]
    fn a_restore_reads_back_in_the_desktops_words() {
        crate::tr::set_lang(crate::tr::Lang::En);
        assert_eq!(
            summary_line(&summary()),
            "1 folder added, 12 series created, 9 matches applied, 40 views and 1 finished episode merged, 3 resume points merged, 0 list accounts brought in."
        );
        let more = ImportSummary {
            fixes_applied: 2,
            sources_unavailable: 1,
            fields_ignored: vec!["theme".into()],
            ..summary()
        };
        let line = summary_line(&more);
        assert!(line.ends_with(
            "2 files take a fix you made. 1 folder was not found on this phone. Left out: theme."
        ));
    }

    #[test]
    fn the_cache_is_named_by_size_and_count() {
        crate::tr::set_lang(crate::tr::Lang::En);
        assert_eq!(images_line(None), "The image cache could not be measured");
        assert_eq!(images_line(Some((0, 0))), "No images cached");
        assert_eq!(
            images_line(Some((340, 12_900_000))),
            "12.3 MB in 340 images"
        );
        assert_eq!(images_line(Some((1, 2048))), "2.0 KB in 1 image");
    }
}
