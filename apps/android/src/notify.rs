//! The download notification's permission, as the person meets it: what onboarding and
//! Downloads say about it, and what their button does. Android 13 and later ask once, maybe
//! twice, and after a second Don't allow only the app's notification settings can turn it on,
//! so the app asks once and sends every later tap to those settings.

use std::collections::HashSet;

/// Where the permission stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notify {
    /// Allowed, or an Android that does not ask.
    On,
    /// Never asked: the button brings Android's own dialog.
    Unasked,
    /// Asked and not allowed: the button opens the app's notification settings.
    Denied,
}

impl Notify {
    pub fn new(allowed: bool, asked: bool) -> Notify {
        match (allowed, asked) {
            (true, _) => Notify::On,
            (false, false) => Notify::Unasked,
            (false, true) => Notify::Denied,
        }
    }

    /// The number Slint reads: 0 on, 1 never asked, 2 denied.
    pub fn code(self) -> i32 {
        match self {
            Notify::On => 0,
            Notify::Unasked => 1,
            Notify::Denied => 2,
        }
    }
}

/// Whether the first download puts its question up before it starts: only for someone
/// Android never asked who has not seen the question already, in onboarding or on an
/// earlier download.
pub fn offer_on_download(state: Notify, offered: bool) -> bool {
    state == Notify::Unasked && !offered
}

/// One download that is moving, as the notification reads it: what it is called, the line
/// Downloads shows for it ("420 MB of 1.4 GB, 3.2 MB/s, 5m left"), and how far along.
#[derive(Clone, Debug, PartialEq)]
pub struct Moving {
    pub called: String,
    pub facts: String,
    pub progress: f64,
    pub size: u64,
}

/// The ongoing notification while episodes download.
#[derive(Clone, Debug, PartialEq)]
pub struct Ongoing {
    pub title: String,
    pub text: String,
    /// How far along in tenths of a percent, the bar under the text; -1 for unknown.
    pub progress: i32,
}

/// What a download is called in a notification: the show and its episode, "Frieren
/// episode 7" or "Frieren episodes 1 to 12", the show alone for a batch that names none, or
/// the torrent's own name when it is not a show's.
pub fn called(show: &str, what: &str, name: &str) -> String {
    if show.is_empty() {
        return name.to_string();
    }
    if what.is_empty() || what == "A batch" {
        return show.to_string();
    }
    format!("{show} {}", lower_first(what))
}

/// The finished download's notification: "Frieren episode 7 is ready", "are" for a run of
/// episodes, "A batch of Frieren is ready" for a batch that names none.
pub fn ready_title(show: &str, what: &str, name: &str) -> String {
    if what == "A batch" && !show.is_empty() {
        return format!("A batch of {show} is ready");
    }
    let verb = if what.starts_with("Episodes") {
        "are"
    } else {
        "is"
    };
    format!("{} {verb} ready", called(show, what, name))
}

/// The line under it, the same words onboarding's preview shows.
pub const READY_TEXT: &str = "Downloaded to this phone. Tap to open AniBeam.";

fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The ongoing notification for what is moving: one download by name with the numbers
/// Downloads shows, several as a count with how far along they are together. A size not
/// known yet (a torrent still fetching its file list) leaves the bar unsure. None while
/// nothing moves.
pub fn ongoing(moving: &[Moving], bytes: impl Fn(u64) -> String) -> Option<Ongoing> {
    let tenths = |p: f64| (p.clamp(0.0, 1.0) * 1000.0).floor() as i32;
    let percent = |p: f64| (p.clamp(0.0, 1.0) * 100.0).floor() as i64;
    match moving {
        [] => None,
        [one] if one.size == 0 => Some(Ongoing {
            title: format!("Downloading {}", one.called),
            text: "Starting".into(),
            progress: -1,
        }),
        [one] => Some(Ongoing {
            title: format!("Downloading {}", one.called),
            text: format!("{}%, {}", percent(one.progress), one.facts),
            progress: tenths(one.progress),
        }),
        many => {
            let size: u64 = many.iter().map(|m| m.size).sum();
            let done: f64 = many.iter().map(|m| m.progress * m.size as f64).sum();
            let progress = if size > 0 { done / size as f64 } else { 0.0 };
            Some(Ongoing {
                // Downloads, not episodes: one of them may be a whole batch.
                title: format!("{} downloads", many.len()),
                text: if size > 0 {
                    format!(
                        "{}%, {} of {}",
                        percent(progress),
                        bytes(done as u64),
                        bytes(size)
                    )
                } else {
                    "Starting".into()
                },
                progress: if size > 0 { tenths(progress) } else { -1 },
            })
        }
    }
}

/// The downloads that finished since the last look: each one seen unfinished before (in
/// `unfinished`) that is now whole. `unfinished` is updated to what is unfinished now, so
/// each finish is told once, and one removed before it finished is forgotten.
pub fn finished(unfinished: &mut HashSet<String>, now: &[(String, bool)]) -> Vec<String> {
    let done: Vec<String> = now
        .iter()
        .filter(|(hash, whole)| *whole && unfinished.contains(hash))
        .map(|(hash, _)| hash.clone())
        .collect();
    *unfinished = now
        .iter()
        .filter(|(_, whole)| !*whole)
        .map(|(hash, _)| hash.clone())
        .collect();
    done
}

/// A notification id of its own for each finished download, apart from the service's 1.
pub fn ready_id(hash: &str) -> i32 {
    let h = hash.bytes().fold(2_166_136_261u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(16_777_619)
    });
    (h % 1_000_000_000) as i32 + 2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(n: u64) -> String {
        format!("{} MB", n / 1_000_000)
    }

    fn moving(called: &str, progress: f64, size: u64) -> Moving {
        Moving {
            called: called.into(),
            facts: "420 MB of 1400 MB, 3 MB/s, 5m left".into(),
            progress,
            size,
        }
    }

    #[test]
    fn one_download_reads_as_downloads_shows_it() {
        let o = ongoing(&[moving("Frieren episode 7", 0.3, 1_400_000_000)], bytes).unwrap();
        assert_eq!(o.title, "Downloading Frieren episode 7");
        assert_eq!(o.text, "30%, 420 MB of 1400 MB, 3 MB/s, 5m left");
        assert_eq!(o.progress, 300);
    }

    #[test]
    fn several_count_up_and_weigh_by_size() {
        let o = ongoing(
            &[
                moving("A", 1.0 / 3.0, 600_000_000),
                moving("B", 0.0, 1_400_000_000),
            ],
            bytes,
        )
        .unwrap();
        assert_eq!(o.title, "2 downloads");
        assert_eq!(o.text, "10%, 200 MB of 2000 MB");
        assert_eq!(o.progress, 100);
        assert!(ongoing(&[], bytes).is_none());
    }

    #[test]
    fn sizes_not_known_yet_leave_the_bar_unsure() {
        let o = ongoing(&[moving("A", 0.0, 0), moving("B", 0.0, 0)], bytes).unwrap();
        assert_eq!((o.progress, o.text.as_str()), (-1, "Starting"));
        let one = ongoing(&[moving("A", 0.0, 0)], bytes).unwrap();
        assert_eq!((one.progress, one.text.as_str()), (-1, "Starting"));
    }

    #[test]
    fn a_download_is_called_by_its_show_and_episode() {
        assert_eq!(
            called("Frieren", "Episode 7", "[G] Frieren - 07"),
            "Frieren episode 7"
        );
        assert_eq!(called("Frieren", "", "[G] Frieren"), "Frieren");
        assert_eq!(
            called("", "Episode 7", "[G] Frieren - 07"),
            "[G] Frieren - 07"
        );
        assert_eq!(called("Frieren", "A batch", "[G] Frieren"), "Frieren");
    }

    #[test]
    fn a_batch_is_ready_in_the_plural() {
        assert_eq!(
            ready_title("Frieren", "Episode 7", "x"),
            "Frieren episode 7 is ready"
        );
        assert_eq!(
            ready_title("Frieren", "Episodes 1 to 12", "x"),
            "Frieren episodes 1 to 12 are ready"
        );
        assert_eq!(
            ready_title("Frieren", "A batch", "x"),
            "A batch of Frieren is ready"
        );
        assert_eq!(ready_title("", "", "[G] Film"), "[G] Film is ready");
    }

    #[test]
    fn a_finish_is_told_once_and_only_for_one_seen_moving() {
        let mut was: HashSet<String> = ["a", "b"].map(String::from).into();
        let now = [
            ("a".to_string(), true),
            ("b".to_string(), false),
            ("c".to_string(), true),
        ];
        assert_eq!(finished(&mut was, &now), ["a"]);
        assert_eq!(was, HashSet::from(["b".to_string()]));
        assert!(finished(&mut was, &now).is_empty(), "told once");
        // Removed while it moved: forgotten, never told.
        assert!(finished(&mut was, &[]).is_empty());
        assert!(was.is_empty());
    }

    #[test]
    fn ready_ids_stay_clear_of_the_services() {
        for hash in ["", "a", "ffffffffffffffffffffffffffffffffffffffff"] {
            assert!(ready_id(hash) >= 2);
        }
        assert_eq!(ready_id("abc"), ready_id("abc"));
        assert_ne!(ready_id("abc"), ready_id("abd"));
    }

    #[test]
    fn allowed_is_on_whether_or_not_it_was_asked() {
        assert_eq!(Notify::new(true, false), Notify::On);
        assert_eq!(Notify::new(true, true), Notify::On);
    }

    #[test]
    fn never_asked_is_unasked() {
        assert_eq!(Notify::new(false, false), Notify::Unasked);
    }

    #[test]
    fn asked_and_not_allowed_is_denied() {
        assert_eq!(Notify::new(false, true), Notify::Denied);
    }

    #[test]
    fn the_download_question_shows_once_and_never_after_an_answer() {
        assert!(offer_on_download(Notify::Unasked, false));
        assert!(!offer_on_download(Notify::Unasked, true));
        assert!(!offer_on_download(Notify::Denied, false));
        assert!(!offer_on_download(Notify::On, false));
    }

    #[test]
    fn slint_reads_one_number() {
        assert_eq!(
            [Notify::On, Notify::Unasked, Notify::Denied].map(Notify::code),
            [0, 1, 2]
        );
    }
}
