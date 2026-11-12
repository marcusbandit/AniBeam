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

struct Page {
    window: Weak<AppWindow>,
    folder: PathBuf,
    waiting: Option<Job>,
    storage: Option<(u64, u64)>,
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
        })
    });
    let d = window.global::<BackupData>();
    d.on_back_up(|| with(Page::back_up));
    d.on_restore(|| with(Page::restore));
    d.on_clear_images(|| with(Page::clear));
    d.on_read(|| with(Page::measure));
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
        self.say("", "", "");
        match core.call(Call::Export {
            path: file.to_string_lossy().into_owned(),
            private,
        }) {
            Ok(_) => {
                self.waiting = Some(Job::Export);
                self.data(|d| d.set_busy(true));
            }
            Err(e) => self.say("", "", &e.to_string()),
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
                    Outcome::Done => p.say("Backed up. The file is where you saved it.", "", ""),
                    Outcome::Cancelled => p.say("Not saved. Nothing was written.", "", ""),
                    Outcome::Failed(e) => p.say("", "", &e),
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
        self.say("", "", "");
        self.waiting = Some(Job::Import);
        self.data(|d| d.set_busy(true));
        files::pick(Pick::Open, &file, "", move |outcome| {
            with(|p| match outcome {
                Outcome::Done => p.import(),
                Outcome::Cancelled => p.stop("", ""),
                Outcome::Failed(e) => p.stop("", &e),
            });
        });
    }

    /// The picked file is in: hand it to the core.
    fn import(&mut self) {
        let Some(core) = crate::local::core() else {
            return self.stop("", "AniBeam is still starting. Try again in a moment.");
        };
        let file = self.folder.join("import.json");
        match core.call(Call::Import {
            path: file.to_string_lossy().into_owned(),
        }) {
            Ok(_) => {}
            Err(e) => self.stop("", &e.to_string()),
        }
    }

    /// Ends a backup or restore that is not going on: clears the spinner and says why.
    fn stop(&mut self, note: &str, error: &str) {
        self.waiting = None;
        let _ = std::fs::remove_dir_all(&self.folder);
        self.data(|d| d.set_busy(false));
        self.say(note, "", error);
    }

    fn say(&self, last: &str, summary: &str, failed: &str) {
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
                self.say("", &summary_line(&summary), "");
                crate::local::touch();
            }
            Done::Cleared(removed) if self.waiting == Some(Job::Clear) => {
                self.waiting = None;
                self.say(
                    &format!("Removed {}", plural(removed, "image", "images")),
                    "",
                    "",
                );
                self.measure();
            }
            Done::Failed(job, error) if self.waiting == Some(job) => {
                if job == Job::Export {
                    let _ = std::fs::remove_dir_all(&self.folder);
                }
                self.stop("", &error);
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
        self.say("", "", "");
        match core.call(Call::ClearImages) {
            Ok(_) => self.waiting = Some(Job::Clear),
            Err(e) => self.say("", "", &e.to_string()),
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
                    "Try again in a moment. The images themselves are fine."
                } else {
                    ""
                }
                .into(),
            );
            d.set_images_question(
                format!(
                    "Remove {}?",
                    plural(storage.map_or(0, |(n, _)| n), "image", "images")
                )
                .into(),
            );
        });
    }
}

fn plural(n: u64, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn images_line(storage: Option<(u64, u64)>) -> String {
    match storage {
        None => "The image cache could not be measured".into(),
        Some((0, _)) => "No images cached".into(),
        Some((n, bytes)) => format!("{} in {}", bytes_label(bytes), plural(n, "image", "images")),
    }
}

fn bytes_label(n: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 || v >= 100.0 {
        format!("{v:.0} {}", UNITS[i])
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

/// What a restore did, in the desktop's words.
fn summary_line(s: &ImportSummary) -> String {
    let mut out = [
        format!("{} added", plural(s.sources_added, "folder", "folders")),
        format!("{} created", plural(s.series_created, "series", "series")),
        format!("{} applied", plural(s.matches_applied, "match", "matches")),
        format!(
            "{} and {} merged",
            plural(s.views_merged, "view", "views"),
            plural(s.completed_merged, "finished episode", "finished episodes")
        ),
        format!(
            "{} merged",
            plural(s.resume_points_merged, "resume point", "resume points")
        ),
        format!(
            "{} brought in",
            plural(s.accounts_imported, "list account", "list accounts")
        ),
    ]
    .join(", ")
        + ".";
    if s.fixes_applied > 0 {
        out += &format!(
            " {} a fix you made.",
            plural(s.fixes_applied, "file takes", "files take")
        );
    }
    if s.sources_unavailable > 0 {
        out += &format!(
            " {} not found on this phone.",
            plural(s.sources_unavailable, "folder was", "folders were")
        );
    }
    if !s.fields_ignored.is_empty() {
        out += &format!(" Left out: {}.", s.fields_ignored.join(", "));
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
        assert_eq!(images_line(None), "The image cache could not be measured");
        assert_eq!(images_line(Some((0, 0))), "No images cached");
        assert_eq!(
            images_line(Some((340, 12_900_000))),
            "12.3 MB in 340 images"
        );
        assert_eq!(images_line(Some((1, 2048))), "2.0 KB in 1 image");
    }
}
