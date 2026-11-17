//! An episode's versions as a series row offers them: the releases it is kept in, each a
//! file to play and the release's name. Shared by this phone's page and the other device's.
//!
//! Slint tells a changed row by comparing its models by pointer, and the pages are rebuilt
//! every second, so the same list hands back the same model: a row whose versions did not
//! change is not redrawn.

use std::cell::RefCell;
use std::collections::HashMap;

use slint::{ModelRc, VecModel};

use crate::EpisodeVersionLine;

/// The models handed out, by the list each shows.
type Models = HashMap<Vec<(i32, String)>, ModelRc<EpisodeVersionLine>>;

thread_local! {
    static MODELS: RefCell<Models> = RefCell::new(HashMap::new());
}

/// The releases a row offers, as (file, label): each one whole on disk, and only when there
/// are two or more to pick between. `playable` says whether a file can play now; a copy a
/// torrent is still writing cannot. A file id past what Slint's int holds is left out.
pub fn offered(versions: &[(u64, String)], playable: impl Fn(u64) -> bool) -> Vec<(i32, String)> {
    let list: Vec<(i32, String)> = versions
        .iter()
        .filter(|(file, _)| playable(*file))
        .filter_map(|(file, label)| Some((i32::try_from(*file).ok()?, label.clone())))
        .collect();
    if list.len() > 1 { list } else { Vec::new() }
}

/// The model for `list`, the same one each time the same list comes back.
pub fn model(list: Vec<(i32, String)>) -> ModelRc<EpisodeVersionLine> {
    if list.is_empty() {
        return ModelRc::default();
    }
    MODELS.with(|m| {
        let mut m = m.borrow_mut();
        // Lists of series no longer open would pile up; starting over now and then costs
        // one redraw.
        if m.len() > 4096 {
            m.clear();
        }
        m.entry(list.clone())
            .or_insert_with(|| {
                let rows: Vec<EpisodeVersionLine> = list
                    .iter()
                    .map(|(file, label)| EpisodeVersionLine {
                        file: *file,
                        label: label.as_str().into(),
                    })
                    .collect();
                ModelRc::new(VecModel::from(rows))
            })
            .clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(file: u64) -> (u64, String) {
        (file, format!("release {file}"))
    }

    #[test]
    fn a_copy_still_downloading_is_not_offered() {
        let list = offered(&[v(1), v(2), v(3)], |f| f != 2);
        assert_eq!(list.iter().map(|(f, _)| *f).collect::<Vec<_>>(), [1, 3]);
        assert!(
            offered(&[v(1), v(2)], |f| f != 2).is_empty(),
            "one left is no pick"
        );
    }

    #[test]
    fn a_file_id_past_an_int_is_left_out() {
        assert!(offered(&[v(1), v(u64::MAX)], |_| true).is_empty());
    }

    #[test]
    fn the_same_list_hands_back_the_same_model() {
        let a = model(vec![(1, "A".into()), (2, "B".into())]);
        let b = model(vec![(1, "A".into()), (2, "B".into())]);
        assert!(a == b);
        assert!(model(Vec::new()) == model(Vec::new()));
    }
}
