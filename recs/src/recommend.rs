//! From a fitted model to a short list of things you can start tonight, each with its reasons.

use std::collections::{HashMap, HashSet};

use crate::features::{Side, features};
use crate::model::{Model, Parts, Weights};
use crate::signal::Signal;
use crate::types::{AiringStatus, Catalog, Entry, Format, RelationKind, Show, ShowId, Status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Any,
    /// Low stakes, warm.
    Comfort,
    /// Lands hard: drama, thrillers, tragedy.
    HitsHard,
    /// Action and big set pieces.
    Spectacle,
    /// Strange and ambitious.
    Weird,
}

impl Mood {
    pub fn parse(s: &str) -> Option<Mood> {
        Some(match s {
            "any" => Mood::Any,
            "comfort" => Mood::Comfort,
            "hits-hard" => Mood::HitsHard,
            "spectacle" => Mood::Spectacle,
            "weird" => Mood::Weird,
            _ => return None,
        })
    }

    fn genres(self) -> &'static [&'static str] {
        match self {
            Mood::Any => &[],
            Mood::Comfort => &["Slice of Life", "Comedy", "Romance", "Music"],
            Mood::HitsHard => &["Drama", "Psychological", "Thriller", "Mystery"],
            Mood::Spectacle => &["Action", "Adventure", "Sci-Fi", "Sports", "Mecha"],
            Mood::Weird => &["Psychological", "Sci-Fi", "Supernatural", "Horror"],
        }
    }

    fn fits(self, show: &Show) -> bool {
        self == Mood::Any
            || show
                .genres
                .iter()
                .any(|g| self.genres().contains(&g.as_str()))
    }
}

/// Which part of a franchise a query is after.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entries {
    /// Only shows that start something: no sequels, no side stories. Picks and judge decks.
    First,
    /// Only the next part of something you finished.
    NextSeason,
}

#[derive(Clone, Debug)]
pub struct Query {
    pub mood: Mood,
    /// Only shows you can finish in this many minutes.
    pub max_minutes: Option<u32>,
    pub top: usize,
    /// Also rank shows already on your Planning list.
    pub planning: bool,
    /// Skip shows more people have on their lists than this. Hidden gems mode sets it.
    pub max_popularity: Option<u32>,
    /// Never pick these, such as shows judged "nah".
    pub exclude: HashSet<ShowId>,
    pub entries: Entries,
    /// Shows that may be picked without similar people having rated them: the community's
    /// "if you liked this" for your yeses, which is evidence of its own.
    pub trusted: HashSet<ShowId>,
}

#[derive(Clone, Debug)]
pub struct Pick {
    pub show: ShowId,
    pub title: String,
    /// 0 to 100.
    pub predicted: f32,
    pub parts: Parts,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub planned: bool,
}

/// Points on the 0 to 100 scale shown as tenths, the way people read scores.
fn tenths(points: f32) -> String {
    format!("{:.1}", points.abs() / 10.0)
}

pub fn recommend(
    model: &Model,
    weights: &Weights,
    me: &[Entry],
    signals: &[Signal],
    catalog: &Catalog,
    q: &Query,
) -> Vec<Pick> {
    let on_list: HashMap<ShowId, Status> = me.iter().map(|e| (e.show, e.status)).collect();
    let finished: HashSet<ShowId> = me
        .iter()
        .filter(|e| matches!(e.status, Status::Completed | Status::Repeating))
        .map(|e| e.show)
        .collect();

    // Staff and studios behind the shows you loved, for "same director as ..." reasons.
    let made_by = made_by(signals, model.mean, catalog);

    let mut picks = Vec::new();
    for show in catalog.values() {
        if q.exclude.contains(&show.id) {
            continue;
        }
        let planned = match on_list.get(&show.id) {
            None => false,
            Some(Status::Planning) if q.planning => true,
            Some(_) => continue,
        };
        if show.adult || show.format == Format::Music || !q.mood.fits(show) {
            continue;
        }
        // Not out yet, or never finishing: nothing to press play on.
        if matches!(
            show.status,
            Some(AiringStatus::NotYetAired | AiringStatus::Cancelled)
        ) {
            continue;
        }
        // A recap, retelling or compilation of something you have watched, even in part, is
        // not news to you. Planning doesn't count: you have seen neither version.
        let retold = show.relations.iter().any(|(id, kind)| {
            matches!(
                kind,
                RelationKind::Summary | RelationKind::Alternative | RelationKind::Compilation
            ) && on_list.get(id).is_some_and(|st| *st != Status::Planning)
        });
        if retold {
            continue;
        }
        // A recap retells something you already chose to watch or skip, and a special hanging
        // off a parent show is an extra for its fans, not a pick. AniList tags few recaps, so
        // the shape of the entry decides as much as the tag.
        let extra = show.format == Format::Special
            && show
                .relations
                .iter()
                .any(|(_, k)| *k == RelationKind::Parent);
        if extra || show.tags.iter().any(|t| t.name == "Recap" && t.rank >= 50) {
            continue;
        }
        // Unknown popularity can't prove a show is obscure, so gems mode leaves it out.
        if q.max_popularity
            .is_some_and(|max| show.popularity.is_none_or(|p| p > max))
        {
            continue;
        }
        if let Some(max) = q.max_minutes {
            match show.runtime() {
                Some(t) if t <= max => {}
                _ => continue,
            }
        }
        // A first entry follows nothing. A next season follows a show you finished and hangs
        // off no parent: a compilation film or side story is an extra, however AniList links
        // its prequels. Sequels of shows you are still watching, paused or dropped wait until
        // you finish the first part, so they are in neither list on purpose.
        let has = |kind: RelationKind| show.relations.iter().any(|(_, k)| *k == kind);
        let after_finished = show
            .relations
            .iter()
            .any(|(id, kind)| *kind == RelationKind::Prequel && finished.contains(id));
        let fits = match q.entries {
            Entries::First => !has(RelationKind::Prequel) && !has(RelationKind::Parent),
            Entries::NextSeason => after_finished && !has(RelationKind::Parent),
        };
        if !fits {
            continue;
        }
        let parts = model.parts(show);
        // No opinion from anyone like you means no recommendation, however well it scores.
        if parts.peers.is_none() && !q.trusted.contains(&show.id) {
            continue;
        }
        let predicted = weights.score(model.mean, &parts);
        let (reasons, warnings) = explain(show, &parts, &made_by, catalog);
        picks.push(Pick {
            show: show.id,
            title: show.title.clone(),
            predicted,
            parts,
            reasons,
            warnings,
            planned,
        });
    }
    picks.sort_by(|a, b| b.predicted.total_cmp(&a.predicted));
    picks.truncate(q.top);
    picks
}

/// The staff behind the shows you loved, each mapped to the best of them, for "also made"
/// reasons.
pub fn made_by(signals: &[Signal], mean: f32, catalog: &Catalog) -> HashMap<String, (ShowId, f32)> {
    let mut loved: Vec<&Signal> = signals
        .iter()
        .filter(|s| !s.dropped && s.value >= mean + 10.0)
        .collect();
    loved.sort_by(|a, b| b.value.total_cmp(&a.value));
    let mut out: HashMap<String, (ShowId, f32)> = HashMap::new();
    for s in &loved {
        let Some(show) = catalog.get(&s.show) else {
            continue;
        };
        for f in features(show, Side::Look) {
            if f.key.starts_with("staff:") {
                out.entry(f.key).or_insert((s.show, s.value));
            }
        }
    }
    out
}

/// Reasons and warnings for one show, in words a person reads.
pub fn explain(
    show: &Show,
    parts: &Parts,
    made_by: &HashMap<String, (ShowId, f32)>,
    catalog: &Catalog,
) -> (Vec<String>, Vec<String>) {
    let mut reasons = Vec::new();
    let mut warnings = Vec::new();

    if let Some(p) = &parts.peers {
        reasons.push(format!(
            "{} people with taste like yours rate it {:.1} on average",
            p.raters,
            p.avg / 10.0
        ));
    }
    // The strongest tie: someone behind a show you loved worked on this one.
    // Heaviest roles first, and only roles that shape the look: a shared colour designer is
    // too thin a thread to name.
    let mut look = features(show, Side::Look);
    look.sort_by(|a, b| b.w.total_cmp(&a.w));
    for f in look.iter().filter(|f| f.w >= 0.6) {
        if let Some((id, value)) = made_by.get(&f.key)
            && let Some(theirs) = catalog.get(id)
        {
            reasons.push(format!(
                "{} also made {} (you gave it {:.1})",
                f.label,
                theirs.title,
                value.min(100.0) / 10.0
            ));
            break;
        }
    }
    if let Some(e) = parts
        .look
        .evidence
        .iter()
        .find(|e| e.aff > 2.0 && e.n >= 2 && !e.key.starts_with("era:"))
    {
        reasons.push(format!(
            "{}: you rate their shows {} above your average ({} shows)",
            e.label,
            tenths(e.aff),
            e.n
        ));
    }
    // Spoiler tags count toward the score but are never named.
    let spoiler = |key: &str| {
        show.tags
            .iter()
            .any(|t| t.spoiler && key == format!("tag:{}", t.name))
    };
    if let Some(e) = parts
        .taste
        .evidence
        .iter()
        .find(|e| e.aff > 2.0 && e.n >= 4 && !spoiler(&e.key))
    {
        reasons.push(format!(
            "{}: {} above your average across {} shows",
            e.label,
            tenths(e.aff),
            e.n
        ));
    }
    // Warn about the look, since that is what you drop over.
    for e in parts
        .look
        .evidence
        .iter()
        .filter(|e| e.aff < -3.0 && e.n >= 2 && !e.key.starts_with("era:"))
        .take(2)
    {
        warnings.push(format!(
            "{}: you rate their shows {} below your average ({} shows)",
            e.label,
            tenths(e.aff),
            e.n
        ));
    }
    (reasons, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Model;
    use crate::signal::signals;
    use crate::types::{DropReasons, Neighbor, Tag};

    fn show(id: ShowId, popularity: Option<u32>, format: Format) -> Show {
        Show {
            id,
            title: format!("show {id}"),
            format,
            episodes: Some(12),
            minutes: Some(24),
            year: Some(2020),
            genres: vec!["Drama".into()],
            tags: vec![],
            studios: vec![],
            credits: vec![],
            community: Some(80),
            popularity,
            relations: vec![],
            adult: false,
            status: None,
        }
    }

    fn entry(show: ShowId, score: f32) -> Entry {
        Entry {
            show,
            status: Status::Completed,
            score: Some(score),
            repeat: 0,
        }
    }

    /// Fifteen rated shows of mine (odd ids finished at 50, even ids at 90), two people who
    /// rate them the same way, and candidates both of them loved: an obscure show, a famous
    /// one, a special hanging off one of my shows, a recap, and one of unknown popularity.
    /// `extra` adds or changes candidates; every candidate id from 100 up gets rated.
    fn picks_with(
        max_popularity: Option<u32>,
        entries: Entries,
        extra: impl FnOnce(&mut Catalog),
    ) -> Vec<ShowId> {
        let mut me: Vec<Entry> = (1..=15)
            .map(|i| entry(i, if i % 2 == 0 { 90.0 } else { 50.0 }))
            .collect();
        // One show only planned, never watched.
        me.push(Entry {
            show: 16,
            status: Status::Planning,
            score: None,
            repeat: 0,
        });
        let mut catalog: Catalog = (1..=15)
            .map(|i| (i, show(i, Some(5000), Format::Tv)))
            .collect();
        catalog.insert(100, show(100, Some(5000), Format::Tv));
        catalog.insert(101, show(101, Some(900_000), Format::Tv));
        let mut special = show(102, Some(5000), Format::Special);
        special.relations = vec![(1, RelationKind::Parent)];
        catalog.insert(102, special);
        let mut recap = show(103, Some(5000), Format::Tv);
        recap.tags = vec![Tag {
            name: "Recap".into(),
            rank: 80,
            technical: false,
            spoiler: false,
        }];
        catalog.insert(103, recap);
        catalog.insert(104, show(104, None, Format::Tv));
        extra(&mut catalog);
        let candidates: Vec<ShowId> = catalog.keys().copied().filter(|id| *id >= 100).collect();
        let twin = |user| {
            let mut entries = me.clone();
            entries.extend(candidates.iter().map(|id| entry(*id, 95.0)));
            Neighbor { user, entries }
        };
        let (mean, sig) = signals(&me, &DropReasons::new()).unwrap();
        let model = Model::fit(&sig, mean, &catalog, &[twin(1), twin(2)]);
        let q = Query {
            mood: Mood::Any,
            max_minutes: None,
            top: 20,
            planning: false,
            max_popularity,
            exclude: HashSet::new(),
            entries,
            trusted: HashSet::new(),
        };
        let mut ids: Vec<ShowId> = recommend(&model, &Weights::default(), &me, &sig, &catalog, &q)
            .iter()
            .map(|p| p.show)
            .collect();
        ids.sort();
        ids
    }

    fn picks(max_popularity: Option<u32>) -> Vec<ShowId> {
        picks_with(max_popularity, Entries::First, |_| {})
    }

    #[test]
    fn specials_off_a_parent_and_recaps_are_never_picks() {
        assert_eq!(picks(None), vec![100, 101, 104]);
    }

    #[test]
    fn gems_skip_popular_shows_and_shows_of_unknown_popularity() {
        assert_eq!(picks(Some(50_000)), vec![100]);
    }

    #[test]
    fn retellings_recaps_and_compilations_of_listed_shows_are_dropped() {
        let ids = picks_with(None, Entries::First, |c| {
            for (id, kind) in [
                (110, RelationKind::Summary),
                (111, RelationKind::Alternative),
                (112, RelationKind::Compilation),
            ] {
                let mut s = show(id, Some(5000), Format::Tv);
                s.relations = vec![(2, kind)];
                c.insert(id, s);
            }
            // A retelling of something not on the list is still fair game, and so is one of a
            // show that is only planned.
            let mut free = show(113, Some(5000), Format::Tv);
            free.relations = vec![(999, RelationKind::Alternative)];
            c.insert(113, free);
            let mut planned = show(114, Some(5000), Format::Tv);
            planned.relations = vec![(16, RelationKind::Alternative)];
            c.insert(114, planned);
        });
        assert_eq!(ids, vec![100, 101, 104, 113, 114]);
    }

    #[test]
    fn unreleased_and_cancelled_shows_are_dropped_but_airing_ones_stay() {
        let ids = picks_with(None, Entries::First, |c| {
            for (id, st) in [
                (120, AiringStatus::NotYetAired),
                (121, AiringStatus::Cancelled),
                (122, AiringStatus::Airing),
            ] {
                let mut s = show(id, Some(5000), Format::Tv);
                s.status = Some(st);
                c.insert(id, s);
            }
        });
        assert_eq!(ids, vec![100, 101, 104, 122]);
    }

    #[test]
    fn first_entries_exclude_sequels_and_next_seasons_need_a_finished_prequel() {
        let extra = |c: &mut Catalog| {
            let mut after_finished = show(130, Some(5000), Format::Tv);
            after_finished.relations = vec![(2, RelationKind::Prequel)];
            c.insert(130, after_finished);
            // A compilation film: follows a finished show but hangs off it as its parent.
            let mut compilation = show(132, Some(5000), Format::Movie);
            compilation.relations = vec![(4, RelationKind::Prequel), (4, RelationKind::Parent)];
            c.insert(132, compilation);
            let mut after_unseen = show(131, Some(5000), Format::Tv);
            after_unseen.relations = vec![(999, RelationKind::Prequel)];
            c.insert(131, after_unseen);
        };
        assert_eq!(picks_with(None, Entries::First, extra), vec![100, 101, 104]);
        assert_eq!(picks_with(None, Entries::NextSeason, extra), vec![130]);
    }
}
