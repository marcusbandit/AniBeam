//! Why a show is recommended, in terms a person can check against their own list: which of
//! their shows it is closest to, which people and tags it shares with the shows they rated, and
//! how far each of those sits above or below their usual score.

use std::collections::HashMap;

use serde::Serialize;

use crate::features::{Feature, Side, features};
use crate::model::{Model, Weights};
use crate::signal::Signal;
use crate::types::{Catalog, Show, ShowId};

/// One of your shows, as evidence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Yours {
    pub title: String,
    /// Your score, 0 to 10.
    pub score: f32,
}

/// One thing the show shares with your list.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Matched {
    /// The trait's key, for setting it aside: "tag:Military", "studio:MAPPA".
    pub key: String,
    /// "Makoto Shinkai (director)", "MAPPA (studio)", "Time Loop".
    pub label: String,
    /// "staff", "studio", "visual", "tag" or "genre".
    pub kind: &'static str,
    /// How far your shows carrying it sit above your usual, 0 to 10 scale.
    pub above: f32,
    /// How many of your rated shows carry it.
    pub shows: u32,
    /// Your best-rated shows carrying it.
    pub examples: Vec<Yours>,
}

/// A show on your list close to this one.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Near {
    pub id: ShowId,
    pub title: String,
    pub score: f32,
    /// What the two have in common, strongest first. Never this pick's spoiler tags.
    pub shared: Vec<String>,
    /// How many of this pick's spoiler tags your show also has. They count toward the
    /// closeness, and are only ever counted, never named.
    pub hidden_shared: u32,
    /// Your show's own strongest tags, spoilers included: you have already seen it.
    pub tags: Vec<YourTag>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YourTag {
    pub name: String,
    pub spoiler: bool,
}

/// A spoiler tag of the pick that you rate highly: shown as "a hidden tag you love",
/// never by name.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Hidden {
    /// The strongest one's distance above your usual, 0 to 10 scale.
    pub above: f32,
    /// How many of your rated shows carry it.
    pub shows: u32,
    /// How many such tags there are.
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Why {
    /// Predicted score, 0 to 10.
    pub predicted: f32,
    /// Share of your rated shows you'd likely rate below this one, 0 to 100.
    pub match_pct: u8,
    /// What the look side and the story side think, 0 to 10 above or below your usual.
    pub look: f32,
    pub story: f32,
    pub look_matches: Vec<Matched>,
    pub story_matches: Vec<Matched>,
    /// Shared traits you tend to rate low.
    pub against: Vec<Matched>,
    pub hidden: Option<Hidden>,
    pub near: Vec<Near>,
}

/// Your list arranged for explaining: which of your shows carry each trait, and your scores.
pub struct Index {
    mean: f32,
    values: Vec<f32>,
    by_key: HashMap<String, Vec<(ShowId, f32)>>,
    rated: Vec<Rated>,
    franchise: HashMap<ShowId, ShowId>,
}

struct Rated {
    id: ShowId,
    value: f32,
    features: Vec<Feature>,
}

/// A candidate neighbour: how much it shares, which of your shows it is, and what it shares.
struct Overlap<'a> {
    weight: f32,
    rated: &'a Rated,
    shared: Vec<(f32, String)>,
    hidden: u32,
}

fn kind(key: &str) -> &'static str {
    match key.split(':').next() {
        Some("staff") => "staff",
        Some("studio") => "studio",
        Some("tech") => "visual",
        Some("genre") => "genre",
        _ => "tag",
    }
}

/// A hidden tag is only worth teasing when you clearly love it: 0.4 above your usual or more.
const HIDDEN_FLOOR: f32 = 4.0;

/// The tag keys a show must never display: its spoiler tags.
fn spoilers(show: &Show) -> std::collections::HashSet<String> {
    show.tags
        .iter()
        .filter(|t| t.spoiler)
        .map(|t| format!("tag:{}", t.name))
        .collect()
}

/// Traits worth naming when two shows share them. A shared genre or era says little.
fn telling(key: &str) -> bool {
    !key.starts_with("genre:") && !key.starts_with("era:")
}

impl Index {
    /// `signals` must be the list alone: a swiped poster is not a show you rated.
    pub fn new(signals: &[Signal], mean: f32, catalog: &Catalog) -> Index {
        let mut by_key: HashMap<String, Vec<(ShowId, f32)>> = HashMap::new();
        let mut rated = Vec::new();
        let mut values = Vec::new();
        for s in signals.iter().filter(|s| !s.vote) {
            values.push(s.value);
            let Some(show) = catalog.get(&s.show) else {
                continue;
            };
            let mut fs = features(show, Side::Look);
            fs.extend(features(show, Side::Taste));
            for f in &fs {
                by_key
                    .entry(f.key.clone())
                    .or_default()
                    .push((s.show, s.value));
            }
            rated.push(Rated {
                id: s.show,
                value: s.value,
                features: fs,
            });
        }
        for v in by_key.values_mut() {
            v.sort_by(|a, b| b.1.total_cmp(&a.1));
        }
        values.sort_by(f32::total_cmp);
        Index {
            mean,
            values,
            by_key,
            rated,
            franchise: crate::franchise::franchises(catalog),
        }
    }

    fn group(&self, id: ShowId) -> ShowId {
        *self.franchise.get(&id).unwrap_or(&id)
    }

    fn examples(&self, key: &str, catalog: &Catalog) -> Vec<Yours> {
        let mut seen = std::collections::HashSet::new();
        self.by_key
            .get(key)
            .into_iter()
            .flatten()
            // Never cite a show for a tag that is a spoiler on that show.
            .filter(|(id, _)| {
                catalog
                    .get(id)
                    .is_none_or(|show| !spoilers(show).contains(key))
            })
            // One entry per franchise: six seasons of one show are one example.
            .filter(|(id, _)| seen.insert(self.group(*id)))
            .take(2)
            .filter_map(|(id, v)| {
                Some(Yours {
                    title: catalog.get(id)?.title.clone(),
                    score: (v / 10.0).min(10.0),
                })
            })
            .collect()
    }

    fn matched(&self, key: &str, label: &str, aff: f32, n: u32, catalog: &Catalog) -> Matched {
        Matched {
            key: key.to_owned(),
            label: label.to_owned(),
            kind: kind(key),
            above: aff / 10.0,
            shows: n,
            examples: self.examples(key, catalog),
        }
    }

    /// The share of your rated shows that sit below a predicted score, 0 to 100.
    pub fn percentile(&self, predicted: f32) -> u8 {
        if self.values.is_empty() {
            return 50;
        }
        let below = self.values.partition_point(|v| *v < predicted);
        ((below as f32 / self.values.len() as f32) * 100.0)
            .round()
            .min(100.0) as u8
    }

    /// The shows on your list sharing the most telling traits with `show`.
    pub fn near(&self, show: &Show, catalog: &Catalog, top: usize) -> Vec<Near> {
        let hidden = spoilers(show);
        let mut mine: Vec<Feature> = features(show, Side::Look);
        mine.extend(features(show, Side::Taste));
        let mine: HashMap<&str, &Feature> = mine
            .iter()
            .filter(|f| telling(&f.key))
            .map(|f| (f.key.as_str(), f))
            .collect();
        let mut scored: Vec<Overlap> = self
            .rated
            .iter()
            .filter(|r| r.id != show.id)
            .filter_map(|r| {
                // A shared spoiler tag counts toward closeness, but only as a number.
                let (mut weight, mut hidden_n) = (0.0f32, 0u32);
                let mut shared: Vec<(f32, String)> = Vec::new();
                // A tag that is a spoiler for either show is counted, never named.
                let theirs = catalog.get(&r.id).map(spoilers).unwrap_or_default();
                for f in &r.features {
                    let Some(m) = mine.get(f.key.as_str()) else {
                        continue;
                    };
                    let w = m.w.min(f.w);
                    weight += w;
                    if hidden.contains(&f.key) || theirs.contains(&f.key) {
                        hidden_n += 1;
                    } else {
                        shared.push((w, m.label.clone()));
                    }
                }
                if weight == 0.0 {
                    return None;
                }
                shared.sort_by(|a, b| b.0.total_cmp(&a.0));
                Some(Overlap {
                    weight,
                    rated: r,
                    shared,
                    hidden: hidden_n,
                })
            })
            .collect();
        scored.sort_by(|a, b| {
            b.weight
                .total_cmp(&a.weight)
                .then(b.rated.value.total_cmp(&a.rated.value))
        });
        // One poster per franchise, and never the show's own franchise.
        let own = self.group(show.id);
        let mut seen = std::collections::HashSet::from([own]);
        scored
            .into_iter()
            .filter(|o| seen.insert(self.group(o.rated.id)))
            .take(top)
            .filter_map(|o| {
                let yours = catalog.get(&o.rated.id)?;
                // Your show's tags, minus anything that could give a twist away: none of this
                // pick's spoiler tags (that would reveal the pick) and none of your show's own,
                // finished or not, since a story can go on in later seasons.
                let mut tags: Vec<&crate::types::Tag> = yours
                    .tags
                    .iter()
                    .filter(|t| !t.technical && !t.spoiler && t.rank >= 60)
                    .filter(|t| !hidden.contains(&format!("tag:{}", t.name)))
                    .collect();
                tags.sort_by_key(|t| std::cmp::Reverse(t.rank));
                Some(Near {
                    id: o.rated.id,
                    title: yours.title.clone(),
                    score: (o.rated.value / 10.0).min(10.0),
                    shared: o.shared.into_iter().take(4).map(|(_, l)| l).collect(),
                    hidden_shared: o.hidden,
                    tags: tags
                        .into_iter()
                        .take(8)
                        .map(|t| YourTag {
                            name: t.name.clone(),
                            spoiler: t.spoiler,
                        })
                        .collect(),
                })
            })
            .collect()
    }

    pub fn why(&self, model: &Model, weights: &Weights, catalog: &Catalog, show: &Show) -> Why {
        let parts = model.parts(show);
        let predicted = weights.score(self.mean, &parts);
        // Spoiler tags still count toward the prediction above, but are never named.
        let hidden = spoilers(show);
        let named = |e: &crate::affinity::Evidence| {
            (telling(&e.key) || e.key.starts_with("genre:")) && !hidden.contains(&e.key)
        };
        let look_matches = parts
            .look
            .evidence
            .iter()
            .filter(|e| {
                e.aff > 0.5 && e.n >= 2 && !e.key.starts_with("era:") && !hidden.contains(&e.key)
            })
            .take(3)
            .map(|e| self.matched(&e.key, &e.label, e.aff, e.n, catalog))
            .collect();
        let story_matches = parts
            .taste
            .evidence
            .iter()
            .filter(|e| e.aff > 0.5 && e.n >= 3 && named(e))
            .take(4)
            .map(|e| self.matched(&e.key, &e.label, e.aff, e.n, catalog))
            .collect();
        let mut against: Vec<Matched> = parts
            .look
            .evidence
            .iter()
            .chain(&parts.taste.evidence)
            .filter(|e| e.aff < -2.0 && e.n >= 2 && named(e))
            .take(2)
            .map(|e| self.matched(&e.key, &e.label, e.aff, e.n, catalog))
            .collect();
        against.sort_by(|a, b| a.above.total_cmp(&b.above));
        // Spoiler tags you love: say they are there, never what they are.
        let loved: Vec<&crate::affinity::Evidence> = parts
            .taste
            .evidence
            .iter()
            .filter(|e| hidden.contains(&e.key) && e.aff >= HIDDEN_FLOOR && e.n >= 2)
            .collect();
        let hidden_love = loved
            .iter()
            .max_by(|a, b| a.aff.total_cmp(&b.aff))
            .map(|e| Hidden {
                above: e.aff / 10.0,
                shows: e.n,
                count: loved.len() as u32,
            });
        Why {
            predicted: predicted / 10.0,
            match_pct: self.percentile(predicted),
            look: parts.look.dev / 10.0,
            story: parts.taste.dev / 10.0,
            look_matches,
            story_matches,
            against,
            hidden: hidden_love,
            near: self.near(show, catalog, 3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Credit, Format, LookRole, Tag};

    fn show(id: ShowId, director: u64, tag: &str) -> Show {
        Show {
            id,
            title: format!("show {id}"),
            format: Format::Tv,
            episodes: None,
            minutes: None,
            year: None,
            genres: vec!["Drama".into()],
            tags: vec![Tag {
                name: tag.into(),
                rank: 90,
                technical: false,
                spoiler: false,
            }],
            studios: vec![],
            credits: vec![Credit {
                staff: director,
                name: format!("D{director}"),
                role: LookRole::Director,
            }],
            community: None,
            popularity: None,
            relations: vec![],
            adult: false,
            status: None,
        }
    }

    fn signal(show: ShowId, value: f32) -> Signal {
        Signal {
            show,
            value,
            taste_w: 1.0,
            look_w: 1.0,
            dropped: false,
            vote: false,
        }
    }

    #[test]
    fn the_nearest_show_is_the_one_sharing_the_director_and_tag() {
        let catalog = Catalog::from([
            (1, show(1, 7, "Time Loop")),
            (2, show(2, 8, "Time Loop")),
            (3, show(3, 9, "Idol")),
            (9, show(9, 7, "Time Loop")),
        ]);
        let index = Index::new(
            &[signal(1, 95.0), signal(2, 80.0), signal(3, 60.0)],
            78.0,
            &catalog,
        );
        let near = index.near(&catalog[&9], &catalog, 3);
        assert_eq!(near[0].title, "show 1");
        assert_eq!(near[0].shared, vec!["D7 (director)", "Time Loop"]);
        assert_eq!(near.len(), 2, "show 3 shares nothing telling");
    }

    #[test]
    fn percentile_counts_the_shows_below() {
        let catalog = Catalog::new();
        let index = Index::new(
            &[
                signal(1, 50.0),
                signal(2, 70.0),
                signal(3, 90.0),
                signal(4, 95.0),
            ],
            76.0,
            &catalog,
        );
        assert_eq!(index.percentile(80.0), 50);
        assert_eq!(index.percentile(99.0), 100);
    }

    #[test]
    fn votes_are_never_evidence() {
        let catalog = Catalog::from([(1, show(1, 7, "Time Loop"))]);
        let mut vote = signal(1, 95.0);
        vote.vote = true;
        let index = Index::new(&[vote], 78.0, &catalog);
        assert!(index.near(&show(9, 7, "Time Loop"), &catalog, 3).is_empty());
    }

    #[test]
    fn a_franchise_shows_once_among_the_nearest() {
        use crate::types::RelationKind;
        let mut s1 = show(1, 7, "Time Loop");
        s1.relations = vec![(2, RelationKind::Sequel)];
        let mut s2 = show(2, 7, "Time Loop");
        s2.relations = vec![(1, RelationKind::Prequel)];
        let catalog = Catalog::from([
            (1, s1),
            (2, s2),
            (3, show(3, 7, "Idol")),
            (9, show(9, 7, "Time Loop")),
        ]);
        let index = Index::new(
            &[signal(1, 95.0), signal(2, 90.0), signal(3, 60.0)],
            78.0,
            &catalog,
        );
        let titles: Vec<String> = index
            .near(&catalog[&9], &catalog, 3)
            .into_iter()
            .map(|n| n.title)
            .collect();
        assert_eq!(titles, vec!["show 1", "show 3"]);
    }

    #[test]
    fn a_spoiler_tag_counts_but_is_never_named() {
        let mut secret = show(9, 7, "Time Loop");
        secret.tags.push(Tag {
            name: "Tragedy".into(),
            rank: 95,
            technical: false,
            spoiler: true,
        });
        let mut mine = show(1, 8, "Idol");
        mine.tags.push(Tag {
            name: "Tragedy".into(),
            rank: 95,
            technical: false,
            spoiler: false,
        });
        let catalog = Catalog::from([(1, mine.clone()), (2, mine.clone()), (3, mine), (9, secret)]);
        let twin = |id| {
            let mut s = catalog[&1].clone();
            s.id = id;
            s
        };
        let catalog = Catalog::from([
            (1, twin(1)),
            (2, twin(2)),
            (3, twin(3)),
            (9, catalog[&9].clone()),
        ]);
        let sig = [signal(1, 95.0), signal(2, 94.0), signal(3, 96.0)];
        let index = Index::new(&sig, 70.0, &catalog);
        let model = crate::model::Model::fit(&sig, 70.0, &catalog, &[]);
        let why = index.why(
            &model,
            &crate::model::Weights::default(),
            &catalog,
            &catalog[&9],
        );
        // The prediction still feels it...
        assert!(model.parts(&catalog[&9]).taste.dev > 0.0);
        // ...but nothing on the card says it.
        let named: Vec<&str> = why
            .story_matches
            .iter()
            .chain(&why.look_matches)
            .map(|m| m.label.as_str())
            .collect();
        assert!(!named.contains(&"Tragedy"), "{named:?}");
        assert!(
            why.near
                .iter()
                .all(|n| !n.shared.iter().any(|s| s == "Tragedy"))
        );
    }

    #[test]
    fn no_spoiler_tag_is_ever_listed_for_your_shows_either() {
        let mut yours = show(1, 7, "Time Loop");
        yours.tags.push(Tag {
            name: "Memory Manipulation".into(),
            rank: 90,
            technical: false,
            spoiler: true,
        });
        let catalog = Catalog::from([(1, yours), (9, show(9, 7, "Time Loop"))]);
        let index = Index::new(&[signal(1, 95.0)], 70.0, &catalog);
        let near = index.near(&catalog[&9], &catalog, 3);
        assert!(near[0].tags.iter().all(|t| t.name != "Memory Manipulation"));
        // Nor in "both have", when the pick has it openly but your show has it as a spoiler.
        let mut pick = show(9, 7, "Time Loop");
        pick.tags.push(Tag {
            name: "Memory Manipulation".into(),
            rank: 90,
            technical: false,
            spoiler: false,
        });
        let catalog = Catalog::from([(1, catalog[&1].clone()), (9, pick)]);
        let near = Index::new(&[signal(1, 95.0)], 70.0, &catalog).near(&catalog[&9], &catalog, 3);
        assert!(
            !near[0].shared.iter().any(|s| s == "Memory Manipulation"),
            "{:?}",
            near[0].shared
        );
        assert_eq!(near[0].hidden_shared, 1);
    }
}
