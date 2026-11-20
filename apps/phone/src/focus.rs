//! Audio focus, the way Android asks a movie player to behave: a call or another app's sound
//! that will pass pauses the episode and the gain back resumes it; one that takes over for
//! good pauses it and stays paused. A duck is Android's own: it lowers the volume itself.
//!
//! Focus is about sound, never about the window: hiding the app changes nothing here.

/// A change Android reports, by `AudioManager`'s numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// AUDIOFOCUS_GAIN, 1: the focus came back.
    Gain,
    /// AUDIOFOCUS_LOSS, -1: another app took it for good.
    Loss,
    /// AUDIOFOCUS_LOSS_TRANSIENT, -2: a call or a short sound, for a while.
    LossTransient,
}

impl Change {
    /// The change for one of `AudioManager`'s numbers. A duck (-3) is None: Android lowers
    /// the volume itself and tells the app nothing it has to act on.
    pub fn from_code(code: i32) -> Option<Change> {
        match code {
            1 => Some(Change::Gain),
            -1 => Some(Change::Loss),
            -2 => Some(Change::LossTransient),
            _ => None,
        }
    }
}

/// What the player does about a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Pause,
    Resume,
    Nothing,
}

/// The player's side of focus: whether it holds it, and whether a passing loss paused it.
#[derive(Debug, Default)]
pub struct Focus {
    /// The request was granted and nothing has taken it for good since.
    pub held: bool,
    /// A passing loss paused the episode, so the gain back plays it again.
    paused_by_loss: bool,
}

impl Focus {
    /// Android reported `change` while the episode was `paused` or not.
    pub fn changed(&mut self, change: Change, paused: bool) -> Act {
        match change {
            Change::LossTransient => {
                if paused {
                    return Act::Nothing;
                }
                self.paused_by_loss = true;
                Act::Pause
            }
            Change::Loss => {
                // Gone for good: asked again on the next play.
                self.held = false;
                self.paused_by_loss = false;
                if paused { Act::Nothing } else { Act::Pause }
            }
            Change::Gain => {
                // Only a request held through a passing loss gets this back; `held` is
                // already true then. A late one after the focus was given back says nothing.
                let resume = std::mem::take(&mut self.paused_by_loss) && paused;
                if resume { Act::Resume } else { Act::Nothing }
            }
        }
    }

    /// The episode started playing: whether to ask for focus now. A play the user makes
    /// during a passing loss is theirs, so the gain back no longer has a pause to undo.
    pub fn playing(&mut self) -> bool {
        self.paused_by_loss = false;
        !self.held
    }

    /// The episode paused for a reason other than a loss (the person, a cast), so the gain
    /// back must not play it again.
    pub fn user_paused(&mut self) {
        self.paused_by_loss = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held() -> Focus {
        let mut f = Focus::default();
        assert!(f.playing(), "the first play asks");
        f.held = true;
        f
    }

    #[test]
    fn a_call_pauses_and_hanging_up_resumes() {
        let mut f = held();
        assert_eq!(f.changed(Change::LossTransient, false), Act::Pause);
        assert_eq!(f.changed(Change::Gain, true), Act::Resume);
    }

    #[test]
    fn a_passing_loss_while_paused_leaves_it_paused_after() {
        let mut f = held();
        assert_eq!(f.changed(Change::LossTransient, true), Act::Nothing);
        assert_eq!(f.changed(Change::Gain, true), Act::Nothing);
    }

    #[test]
    fn a_pause_the_user_makes_during_the_call_holds_after_it() {
        let mut f = held();
        assert_eq!(f.changed(Change::LossTransient, false), Act::Pause);
        // Played during the call, then paused again by hand.
        assert!(!f.playing());
        f.user_paused();
        assert_eq!(f.changed(Change::Gain, true), Act::Nothing);
    }

    #[test]
    fn a_play_by_hand_during_the_call_is_not_resumed_twice() {
        let mut f = held();
        f.changed(Change::LossTransient, false);
        assert!(!f.playing(), "still held, no second request");
        assert_eq!(f.changed(Change::Gain, false), Act::Nothing);
    }

    #[test]
    fn another_app_taking_over_pauses_for_good_and_the_next_play_asks_again() {
        let mut f = held();
        assert_eq!(f.changed(Change::Loss, false), Act::Pause);
        assert!(!f.held);
        assert!(f.playing(), "asks again");
    }

    #[test]
    fn a_late_gain_after_the_focus_was_given_back_does_not_claim_it() {
        let mut f = Focus::default();
        assert_eq!(f.changed(Change::Gain, true), Act::Nothing);
        assert!(f.playing(), "the next play still asks");
    }

    #[test]
    fn a_duck_is_left_to_android() {
        assert_eq!(Change::from_code(-3), None);
        assert_eq!(Change::from_code(1), Some(Change::Gain));
        assert_eq!(Change::from_code(-1), Some(Change::Loss));
        assert_eq!(Change::from_code(-2), Some(Change::LossTransient));
    }
}
