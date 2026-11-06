//! The download notification's permission, as the person meets it: what onboarding and
//! Downloads say about it, and what their button does. Android 13 and later ask once, maybe
//! twice, and after a second Don't allow only the app's notification settings can turn it on,
//! so the app asks once and sends every later tap to those settings.

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

    /// What the button does: Android's dialog the first time, its settings after.
    pub fn action(self) -> Option<Action> {
        match self {
            Notify::On => None,
            Notify::Unasked => Some(Action::Ask),
            Notify::Denied => Some(Action::Settings),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Ask,
    Settings,
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
/// episode 7", or the torrent's own name when it is not a show's.
pub fn called(show: &str, what: &str, name: &str) -> String {
    match (show.is_empty(), what.is_empty()) {
        (false, false) => format!("{show} {}", lower_first(what)),
        (false, true) => show.to_string(),
        _ => name.to_string(),
    }
}

fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The ongoing notification for what is moving: one download by name with the numbers
/// Downloads shows, several as a count with how far along they are together. None while
/// nothing moves.
pub fn ongoing(moving: &[Moving], bytes: impl Fn(u64) -> String) -> Option<Ongoing> {
    let tenths = |p: f64| (p.clamp(0.0, 1.0) * 1000.0).floor() as i32;
    let percent = |p: f64| (p.clamp(0.0, 1.0) * 100.0).floor() as i64;
    match moving {
        [] => None,
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
                title: format!("Downloading {} episodes", many.len()),
                text: format!(
                    "{}%, {} of {}",
                    percent(progress),
                    bytes(done as u64),
                    bytes(size)
                ),
                progress: if size > 0 { tenths(progress) } else { -1 },
            })
        }
    }
}

/// The downloads that finished since the last look: each one seen moving before (in
/// `moving`) that is now whole. `moving` is updated to what moves now, so each finish is
/// told once, and one removed while it moved is forgotten.
pub fn finished(moving: &mut Vec<String>, now: &[(String, bool)]) -> Vec<String> {
    let done: Vec<String> = now
        .iter()
        .filter(|(hash, whole)| *whole && moving.contains(hash))
        .map(|(hash, _)| hash.clone())
        .collect();
    *moving = now
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
        assert_eq!(o.title, "Downloading 2 episodes");
        assert_eq!(o.text, "10%, 200 MB of 2000 MB");
        assert_eq!(o.progress, 100);
        assert!(ongoing(&[], bytes).is_none());
    }

    #[test]
    fn sizes_not_known_yet_leave_the_bar_unsure() {
        let o = ongoing(&[moving("A", 0.0, 0), moving("B", 0.0, 0)], bytes).unwrap();
        assert_eq!(o.progress, -1);
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
    }

    #[test]
    fn a_finish_is_told_once_and_only_for_one_seen_moving() {
        let mut was = vec!["a".to_string(), "b".to_string()];
        let now = [
            ("a".to_string(), true),
            ("b".to_string(), false),
            ("c".to_string(), true),
        ];
        assert_eq!(finished(&mut was, &now), ["a"]);
        assert_eq!(was, ["b"]);
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
        assert_eq!(Notify::On.action(), None);
    }

    #[test]
    fn the_first_tap_brings_androids_dialog() {
        let state = Notify::new(false, false);
        assert_eq!(state, Notify::Unasked);
        assert_eq!(state.action(), Some(Action::Ask));
    }

    #[test]
    fn after_a_denial_the_button_opens_the_settings() {
        let state = Notify::new(false, true);
        assert_eq!(state, Notify::Denied);
        assert_eq!(state.action(), Some(Action::Settings));
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
