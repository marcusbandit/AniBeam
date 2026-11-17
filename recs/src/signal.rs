//! Turns list entries into training signal: one number per show saying how much the person
//! liked it, plus how much that number should teach the story side and the look side.

use crate::types::{DropReason, DropReasons, Entry, ShowId, Status};

/// How far below your own mean an unrated drop lands.
pub const DROP_GAP: f32 = 20.0;
/// Every rewatch is worth this much on top of the score, up to four of them.
pub const REPEAT_BONUS: f32 = 1.5;

#[derive(Clone, Debug, PartialEq)]
pub struct Signal {
    pub show: ShowId,
    /// 0 to 100.
    pub value: f32,
    /// How much this entry teaches the genre and tag model.
    pub taste_w: f32,
    /// How much this entry teaches the staff, studio and visual model.
    pub look_w: f32,
    pub dropped: bool,
    /// From the judge page rather than the list: one vote may only move a feature so far.
    pub vote: bool,
}

pub fn mean_score(entries: &[Entry]) -> Option<f32> {
    let scored: Vec<f32> = entries.iter().filter_map(|e| e.score).collect();
    if scored.is_empty() {
        return None;
    }
    Some(scored.iter().sum::<f32>() / scored.len() as f32)
}

/// The signals in a list and the mean they are measured against. A list with no ratings at all
/// gives no signal, since there is nothing to measure a drop against.
pub fn signals(entries: &[Entry], reasons: &DropReasons) -> Option<(f32, Vec<Signal>)> {
    let mean = mean_score(entries)?;
    let mut out = Vec::new();
    for e in entries {
        let s = match (e.status, e.score) {
            (Status::Planning, _) => continue,
            (Status::Dropped, score) => {
                // A drop rated above your mean is still a drop: cap it just under the mean.
                let value = score.map_or(mean - DROP_GAP, |s| s.min(mean - 5.0));
                // Drops are usually about the look, so an unexplained drop teaches the look side
                // more. A drop over the story teaches the story side instead.
                let (taste_w, look_w) = match reasons.get(&e.show) {
                    None => (0.5, 1.0),
                    Some(DropReason::Look) => (0.2, 1.5),
                    Some(DropReason::Story | DropReason::Pacing) => (1.0, 0.2),
                    Some(DropReason::Other) => (0.6, 0.6),
                };
                Signal {
                    show: e.show,
                    value,
                    taste_w,
                    look_w,
                    dropped: true,
                    vote: false,
                }
            }
            (Status::Paused, None) => continue,
            (Status::Paused, Some(s)) => Signal {
                show: e.show,
                value: s,
                taste_w: 0.5,
                look_w: 0.5,
                dropped: false,
                vote: false,
            },
            (Status::Watching, Some(s)) => Signal {
                show: e.show,
                value: s,
                taste_w: 0.7,
                look_w: 0.7,
                dropped: false,
                vote: false,
            },
            (Status::Watching, None) => continue,
            (Status::Completed | Status::Repeating, Some(s)) => {
                let bonus = e.repeat.min(4) as f32 * REPEAT_BONUS;
                Signal {
                    show: e.show,
                    value: s + bonus,
                    taste_w: 1.0,
                    look_w: 1.0,
                    dropped: false,
                    vote: false,
                }
            }
            // Finished but never rated: they at least sat through it.
            (Status::Completed | Status::Repeating, None) => Signal {
                show: e.show,
                value: mean + 2.0,
                taste_w: 0.3,
                look_w: 0.3,
                dropped: false,
                vote: false,
            },
        };
        out.push(s);
    }
    Some((mean, out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn e(show: ShowId, status: Status, score: Option<f32>, repeat: u32) -> Entry {
        Entry {
            show,
            status,
            score,
            repeat,
        }
    }

    #[test]
    fn an_unrated_drop_lands_below_the_mean_and_teaches_the_look_side() {
        let list = [
            e(1, Status::Completed, Some(80.0), 0),
            e(2, Status::Dropped, None, 0),
        ];
        let (mean, s) = signals(&list, &HashMap::new()).unwrap();
        assert_eq!(mean, 80.0);
        let drop = s.iter().find(|s| s.show == 2).unwrap();
        assert_eq!(drop.value, 60.0);
        assert!(drop.look_w > drop.taste_w);
    }

    #[test]
    fn a_story_drop_teaches_the_story_side() {
        let list = [
            e(1, Status::Completed, Some(80.0), 0),
            e(2, Status::Dropped, None, 0),
        ];
        let reasons = HashMap::from([(2, DropReason::Story)]);
        let (_, s) = signals(&list, &reasons).unwrap();
        let drop = s.iter().find(|s| s.show == 2).unwrap();
        assert!(drop.taste_w > drop.look_w);
    }

    #[test]
    fn rewatches_add_to_the_score_and_planning_is_ignored() {
        let list = [
            e(1, Status::Completed, Some(90.0), 7),
            e(2, Status::Planning, None, 0),
        ];
        let (_, s) = signals(&list, &HashMap::new()).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].value, 96.0);
    }

    #[test]
    fn a_high_score_on_a_drop_is_capped_under_the_mean() {
        let list = [
            e(1, Status::Completed, Some(70.0), 0),
            e(2, Status::Dropped, Some(90.0), 0),
        ];
        let (mean, s) = signals(&list, &HashMap::new()).unwrap();
        let drop = s.iter().find(|s| s.show == 2).unwrap();
        assert!(drop.value < mean);
    }
}
