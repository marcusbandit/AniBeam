//! What a batch holds. A release whose title names no range of episodes is asked of the core
//! (`InspectRelease`), which reads the torrent's file list and answers with the episode
//! numbers the files carry (`ReleaseInspected`). The release list shows the answer under
//! the row, the way the desktop's Get lights the episodes a batch covers.

use std::cell::RefCell;
use std::collections::HashMap;

use anibeam_core::{Call, EventBody};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{AppWindow, FindData};

enum Answer {
    Asking,
    Done(Vec<u32>),
}

#[derive(Default)]
struct Notes {
    window: Option<slint::Weak<AppWindow>>,
    /// The links of the release list as it is drawn, in row order.
    links: Vec<String>,
    answers: HashMap<String, Answer>,
}

thread_local! {
    static NOTES: RefCell<Notes> = RefCell::new(Notes::default());
}

/// Episode numbers as short runs: 1-12, 14, 20-24.
fn runs(list: &[u32]) -> String {
    let mut out = Vec::new();
    let mut i = 0;
    while i < list.len() {
        let mut j = i;
        while j + 1 < list.len() && list[j + 1] == list[j] + 1 {
            j += 1;
        }
        out.push(if j > i {
            format!("{}-{}", list[i], list[j])
        } else {
            list[i].to_string()
        });
        i = j + 1;
    }
    out.join(", ")
}

fn note(answer: Option<&Answer>) -> String {
    match answer {
        None => String::new(),
        Some(Answer::Asking) => "Reading its file list".into(),
        Some(Answer::Done(eps)) if eps.is_empty() => String::new(),
        Some(Answer::Done(eps)) => format!(
            "{} in it: {}",
            if eps.len() == 1 {
                "1 episode".to_string()
            } else {
                format!("{} episodes", eps.len())
            },
            runs(eps)
        ),
    }
}

/// Puts each row's note in place.
fn render() {
    let (window, notes) = NOTES.with(|n| {
        let n = n.borrow();
        (
            n.window.clone(),
            n.links
                .iter()
                .map(|l| SharedString::from(note(n.answers.get(l))))
                .collect::<Vec<_>>(),
        )
    });
    if let Some(window) = window.and_then(|w| w.upgrade()) {
        window
            .global::<FindData>()
            .set_release_notes(ModelRc::new(VecModel::from(notes)));
    }
}

pub fn install(window: &AppWindow) {
    NOTES.with(|n| n.borrow_mut().window = Some(window.as_weak()));
}

/// The release list was drawn: each row's link and whether it is a batch that names no
/// range. The ones not asked before are asked now.
pub fn track(rows: Vec<(String, bool)>) {
    let ask: Vec<String> = NOTES.with(|n| {
        let mut n = n.borrow_mut();
        n.links = rows.iter().map(|(l, _)| l.clone()).collect();
        let mut ask = Vec::new();
        for (link, batch) in &rows {
            if *batch && !link.is_empty() && !n.answers.contains_key(link) {
                n.answers.insert(link.clone(), Answer::Asking);
                ask.push(link.clone());
            }
        }
        ask
    });
    if let Some(core) = crate::local::core() {
        for link in ask {
            if let Err(e) = core.call(Call::InspectRelease { link: link.clone() }) {
                crate::local::log(&format!("inspect: {e}"));
                NOTES.with(|n| {
                    n.borrow_mut()
                        .answers
                        .insert(link, Answer::Done(Vec::new()));
                });
            }
        }
    }
    render();
}

/// The core's answer for one link.
pub fn core_event(body: &EventBody) {
    let EventBody::ReleaseInspected { link, episodes, .. } = body else {
        return;
    };
    let (link, mut episodes) = (link.clone(), episodes.clone());
    episodes.sort_unstable();
    episodes.dedup();
    let _ = slint::invoke_from_event_loop(move || {
        NOTES.with(|n| {
            let mut n = n.borrow_mut();
            if n.answers.contains_key(&link) {
                n.answers.insert(link, Answer::Done(episodes));
            }
        });
        render();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn episodes_read_as_short_runs() {
        assert_eq!(runs(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]), "1-12");
        assert_eq!(runs(&[1, 2, 3, 5, 7, 8]), "1-3, 5, 7-8");
        assert_eq!(runs(&[4]), "4");
        assert_eq!(runs(&[]), "");
    }

    #[test]
    fn a_row_says_what_it_holds() {
        assert_eq!(note(None), "");
        assert_eq!(note(Some(&Answer::Asking)), "Reading its file list");
        assert_eq!(note(Some(&Answer::Done(vec![]))), "");
        assert_eq!(note(Some(&Answer::Done(vec![5]))), "1 episode in it: 5");
        assert_eq!(
            note(Some(&Answer::Done(vec![1, 2, 3]))),
            "3 episodes in it: 1-3"
        );
    }
}
