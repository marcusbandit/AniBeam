//! Seasons, films and side stories of one show belong together. A franchise with six entries
//! on your list should count once as "a show you liked", not six times.

use std::collections::{HashMap, HashSet};

use crate::types::{Catalog, RelationKind, ShowId};

/// Each show's franchise, named by the lowest id in it.
pub fn franchises(catalog: &Catalog) -> HashMap<ShowId, ShowId> {
    let mut parent: HashMap<ShowId, ShowId> = catalog.keys().map(|id| (*id, *id)).collect();
    fn root(parent: &mut HashMap<ShowId, ShowId>, mut id: ShowId) -> ShowId {
        while let Some(&p) = parent.get(&id) {
            if p == id {
                break;
            }
            let gp = parent.get(&p).copied().unwrap_or(p);
            parent.insert(id, gp);
            id = p;
        }
        id
    }
    for show in catalog.values() {
        for (other, kind) in &show.relations {
            // "Other" covers crossovers and characters appearing elsewhere: not the same show.
            if *kind == RelationKind::Other || !parent.contains_key(other) {
                continue;
            }
            let (a, b) = (root(&mut parent, show.id), root(&mut parent, *other));
            if a != b {
                parent.insert(a.max(b), a.min(b));
            }
        }
    }
    let ids: Vec<ShowId> = parent.keys().copied().collect();
    ids.into_iter()
        .map(|id| (id, root(&mut parent, id)))
        .collect()
}

/// Things set aside for now: whole franchises, and single traits such as a tag or a studio.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mutes {
    /// Franchise roots, as `franchises` names them.
    pub franchises: HashSet<ShowId>,
    /// Feature keys such as "tag:Military" or "studio:MAPPA".
    pub features: HashSet<String>,
}

impl Mutes {
    pub fn hides(&self, show: ShowId, fr: &HashMap<ShowId, ShowId>) -> bool {
        self.franchises.contains(fr.get(&show).unwrap_or(&show))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Format, Show};

    fn show(id: ShowId, relations: Vec<(ShowId, RelationKind)>) -> Show {
        Show {
            id,
            title: String::new(),
            format: Format::Tv,
            episodes: None,
            minutes: None,
            year: None,
            genres: vec![],
            tags: vec![],
            studios: vec![],
            credits: vec![],
            community: None,
            popularity: None,
            relations,
            adult: false,
            status: None,
        }
    }

    #[test]
    fn seasons_join_and_crossovers_do_not() {
        let catalog = Catalog::from([
            (10, show(10, vec![(11, RelationKind::Sequel)])),
            (
                11,
                show(
                    11,
                    vec![(10, RelationKind::Prequel), (12, RelationKind::Sequel)],
                ),
            ),
            (
                12,
                show(
                    12,
                    vec![(11, RelationKind::Prequel), (20, RelationKind::Other)],
                ),
            ),
            (20, show(20, vec![])),
        ]);
        let fr = franchises(&catalog);
        assert_eq!(fr[&12], 10);
        assert_eq!(fr[&11], 10);
        assert_eq!(fr[&20], 20);
    }
}
