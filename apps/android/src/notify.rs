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

#[cfg(test)]
mod tests {
    use super::*;

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
