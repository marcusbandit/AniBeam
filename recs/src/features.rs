//! What a show is made of, split in two: the story side (genres, story tags) and the look side
//! (who directed and drew it, the studio, visual technique, era). Each feature carries a weight
//! for how much it says about the show and a label a person can read.

use std::collections::HashMap;

use crate::types::Show;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Taste,
    Look,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Feature {
    pub key: String,
    pub label: String,
    pub w: f32,
}

/// Tags the community only half agrees on are noise.
const TAG_FLOOR: u8 = 60;
const TECH_FLOOR: u8 = 50;

pub fn features(show: &Show, side: Side) -> Vec<Feature> {
    match side {
        Side::Taste => taste(show),
        Side::Look => look(show),
    }
}

fn taste(show: &Show) -> Vec<Feature> {
    let mut out: Vec<Feature> = show
        .genres
        .iter()
        .map(|g| Feature {
            key: format!("genre:{g}"),
            label: g.clone(),
            w: 1.0,
        })
        .collect();
    for t in &show.tags {
        if !t.technical && t.rank >= TAG_FLOOR {
            out.push(Feature {
                key: format!("tag:{}", t.name),
                label: t.name.clone(),
                w: t.rank as f32 / 100.0,
            });
        }
    }
    out
}

fn look(show: &Show) -> Vec<Feature> {
    // One person can hold several roles on a show; count them once, in their heaviest role.
    let mut people: HashMap<u64, Feature> = HashMap::new();
    for c in &show.credits {
        let f = Feature {
            key: format!("staff:{}", c.staff),
            label: format!("{} ({})", c.name, c.role.label()),
            w: c.role.weight(),
        };
        match people.get(&c.staff) {
            Some(old) if old.w >= f.w => {}
            _ => {
                people.insert(c.staff, f);
            }
        }
    }
    let mut out: Vec<Feature> = people.into_values().collect();
    out.sort_by(|a, b| a.key.cmp(&b.key));
    for s in &show.studios {
        out.push(Feature {
            key: format!("studio:{s}"),
            label: format!("{s} (studio)"),
            w: 0.8,
        });
    }
    for t in &show.tags {
        if t.technical && t.rank >= TECH_FLOOR {
            out.push(Feature {
                key: format!("tech:{}", t.name),
                label: t.name.clone(),
                w: t.rank as f32 / 100.0,
            });
        }
    }
    if let Some(y) = show.year {
        // Animation changes with the tools of its time. Five-year buckets are coarse on purpose.
        let bucket = y - y.rem_euclid(5);
        out.push(Feature {
            key: format!("era:{bucket}"),
            label: format!("made {bucket} to {}", bucket + 4),
            w: 0.4,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Credit, Format, LookRole, Tag};

    pub fn show() -> Show {
        Show {
            id: 1,
            title: "A".into(),
            format: Format::Tv,
            episodes: Some(12),
            minutes: Some(24),
            year: Some(2017),
            genres: vec!["Drama".into()],
            tags: vec![
                Tag {
                    name: "Time Loop".into(),
                    rank: 90,
                    technical: false,
                    spoiler: false,
                },
                Tag {
                    name: "Fringe".into(),
                    rank: 20,
                    technical: false,
                    spoiler: false,
                },
                Tag {
                    name: "CGI".into(),
                    rank: 80,
                    technical: true,
                    spoiler: false,
                },
            ],
            studios: vec!["Kyoto Animation".into()],
            credits: vec![
                Credit {
                    staff: 7,
                    name: "Naoko".into(),
                    role: LookRole::Director,
                },
                Credit {
                    staff: 7,
                    name: "Naoko".into(),
                    role: LookRole::AnimationDirector,
                },
            ],
            community: Some(80),
            popularity: Some(1000),
            relations: vec![],
            adult: false,
            status: None,
        }
    }

    #[test]
    fn story_side_keeps_strong_story_tags_only() {
        let keys: Vec<String> = features(&show(), Side::Taste)
            .into_iter()
            .map(|f| f.key)
            .collect();
        assert_eq!(keys, ["genre:Drama", "tag:Time Loop"]);
    }

    #[test]
    fn look_side_counts_a_person_once_in_their_heaviest_role() {
        let f = features(&show(), Side::Look);
        let staff: Vec<&Feature> = f.iter().filter(|f| f.key.starts_with("staff:")).collect();
        assert_eq!(staff.len(), 1);
        assert_eq!(staff[0].w, LookRole::Director.weight());
        assert!(f.iter().any(|f| f.key == "tech:CGI"));
        assert!(f.iter().any(|f| f.key == "era:2015"));
    }
}
