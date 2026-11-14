//! Watch history, resume points and matches merged between trusted AniBeam
//! computers (#561).
//!
//! Each computer serves its own as an `anibeam-export` document at
//! [`wire::HISTORY`]. While Share over Tailscale is on, a computer pulls from
//! every trusted AniBeam on the tailnet once it serves there, and pulls back
//! from any computer that asks for its own, so two that see each other end with
//! the same history whichever came up first. Nothing is polled.
//!
//! The two libraries live in different folders, so a series here is found
//! for a series there by its provider id, or else by its kind and folder
//! name when exactly one series here has them. The history then merges by
//! the export's own rules: the newer of each view, completed episode and
//! resume point wins. A match comes over only onto a series here the person
//! has not confirmed one for; two confirmed matches that differ are two
//! different shows, and nothing merges between them.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anibeam_host_api as wire;
use rusqlite::Connection;

use crate::contract::*;
use crate::core::Core;
use crate::library::cards;
use crate::paths::normalise;
use crate::sync::recover;
use crate::time;
use crate::transfer::format::{self, Document, History, MatchEntry, ResumeEntry, SeriesEntry};
use crate::transfer::{export, file_name, import};

/// One computer is pulled from at most this often.
pub const AGAIN_AFTER: Duration = Duration::from_secs(10 * 60);
/// How long one pull may take, hello and document together.
const PULL_FOR: Duration = Duration::from_secs(60);
/// A history document bigger than this is not one.
const DOCUMENT_MAX: usize = 64 * 1024 * 1024;

/// The document this computer serves: its series, with the matches the person
/// confirmed, and its history. A match the auto-match only guessed is left
/// out, so the other computer never takes a guess for the person's choice.
/// No sources or fixes: those belong to this machine, and a library export
/// already leaves out accounts, keys and preferences.
pub fn document(core: &Core) -> Result<Document, CoreError> {
    let mut doc = export::build(core, false, SystemTime::now())?;
    doc.history = Some(export::history_section(core)?);
    doc.sources.clear();
    let confirmed: std::collections::HashSet<String> = core.store.read(|c| {
        let mut stmt = c.prepare("SELECT path FROM series WHERE confirmed = 1")?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    })?;
    for series in &mut doc.series {
        series.fixes.clear();
        if !confirmed.contains(&series.path) {
            series.match_ = None;
        }
    }
    Ok(doc)
}

/// What one pull merged.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Merged {
    pub views: u64,
    pub completed: u64,
    pub resume_points: u64,
    pub matches: u64,
}

impl Merged {
    pub fn any(&self) -> bool {
        self.views + self.completed + self.resume_points + self.matches > 0
    }
}

/// A series here, as the mapping needs it.
#[derive(Clone, Debug)]
struct Local {
    id: u64,
    kind: String,
    path: String,
    ids: Ids,
    confirmed: bool,
}

/// The provider ids a series is known by.
#[derive(Clone, Debug, Default, PartialEq)]
struct Ids {
    anilist: Option<u64>,
    mal: Option<u64>,
    tvmaze: Option<u64>,
    tmdb: Option<u64>,
}

impl Ids {
    fn of(entry: &MatchEntry) -> Ids {
        match entry {
            MatchEntry::Tracker {
                anilist_id, mal_id, ..
            } => Ids {
                anilist: *anilist_id,
                mal: *mal_id,
                ..Ids::default()
            },
            MatchEntry::Tmdb { tmdb_id, .. } => Ids {
                tmdb: Some(*tmdb_id),
                ..Ids::default()
            },
            MatchEntry::Tvmaze { tvmaze_id, .. } => Ids {
                tvmaze: Some(*tvmaze_id),
                ..Ids::default()
            },
        }
    }

    fn none(&self) -> bool {
        *self == Ids::default()
    }

    /// Whether both name the same show by some provider's id.
    fn shared(&self, other: &Ids) -> bool {
        let same = |a: Option<u64>, b: Option<u64>| a.is_some() && a == b;
        same(self.anilist, other.anilist)
            || same(self.mal, other.mal)
            || same(self.tvmaze, other.tvmaze)
            || same(self.tmdb, other.tmdb)
    }
}

fn locals(conn: &Connection) -> Result<Vec<Local>, CoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, path, anilist_id, mal_id, tvmaze_id, tmdb_id, confirmed FROM series ORDER BY id",
    )?;
    let id = |v: Option<i64>| v.map(|v| v as u64);
    let rows = stmt
        .query_map([], |r| {
            Ok(Local {
                id: r.get::<_, i64>(0)? as u64,
                kind: r.get(1)?,
                path: r.get(2)?,
                ids: Ids {
                    anilist: id(r.get(3)?),
                    mal: id(r.get(4)?),
                    tvmaze: id(r.get(5)?),
                    tmdb: id(r.get(6)?),
                },
                confirmed: r.get::<_, i64>(7)? == 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// The store's `kind` for the document's: `movie` is a film, and anything
/// else a show, the way an import reads it.
fn kind_column(kind: &str) -> &'static str {
    if kind == "movie" { "movie" } else { "show" }
}

/// The last segment of a path written on any system, `/` or `\`.
fn last_segment(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// Which series here each series there is, by its path there: the one
/// series here of its kind that shares a provider id with it, else the one of
/// its kind and folder name, when no other series there has that name too.
/// Several here with its id are told apart by the folder name. A series whose
/// match here is confirmed and differs from its match there is another show,
/// and maps to nothing.
fn map(remote: &[SeriesEntry], here: &[Local]) -> HashMap<String, usize> {
    let name_of = |e: &SeriesEntry| last_segment(&normalise(&e.path)).to_string();
    let mut names_there: HashMap<(&'static str, String), usize> = HashMap::new();
    for entry in remote {
        *names_there
            .entry((kind_column(&entry.kind), name_of(entry)))
            .or_default() += 1;
    }
    let mut out = HashMap::new();
    for entry in remote {
        let kind = kind_column(&entry.kind);
        let name = name_of(entry);
        let ids = entry.match_.as_ref().map(Ids::of).unwrap_or_default();
        let same_kind = |i: usize| here[i].kind == kind;
        let by_name = |among: &mut dyn Iterator<Item = usize>| -> Option<usize> {
            if names_there.get(&(kind, name.clone())) != Some(&1) {
                return None;
            }
            let hits: Vec<usize> = among
                .filter(|&i| same_kind(i) && file_name(&here[i].path) == name)
                .collect();
            (hits.len() == 1).then(|| hits[0])
        };
        let by_id: Vec<usize> = (0..here.len())
            .filter(|&i| same_kind(i) && !ids.none() && here[i].ids.shared(&ids))
            .collect();
        let found = match by_id.len() {
            1 => Some(by_id[0]),
            0 => by_name(&mut (0..here.len())).filter(|&i| {
                // Both sides chose a match and they differ: not the same show.
                let local = &here[i];
                !(local.confirmed && !ids.none() && !local.ids.none() && !local.ids.shared(&ids))
            }),
            _ => by_name(&mut by_id.iter().copied()),
        };
        if let Some(i) = found {
            out.insert(normalise(&entry.path), i);
        }
    }
    out
}

/// A file there under the series path there, as the same path under the
/// series path here: a season folder there stays a season folder here,
/// joined with the separator of the system here. A path that climbs out of
/// the series is no file of it.
fn moved(file: &str, there: &str, here: &str) -> Option<String> {
    let file = normalise(file);
    let rest = file.strip_prefix(there)?;
    if !rest.starts_with(['/', '\\']) {
        return None;
    }
    let mut out = PathBuf::from(here);
    for segment in rest.split(['/', '\\']).filter(|s| !s.is_empty()) {
        if segment == "." || segment == ".." {
            return None;
        }
        out.push(segment);
    }
    Some(out.to_string_lossy().into_owned())
}

/// The history there, rewritten onto the paths of the series here; what
/// belongs to no series here is left out.
fn translate(history: History, paths: &HashMap<String, String>) -> History {
    let series = |there: &str| paths.get(&normalise(there)).cloned();
    let file_of = |there_series: &str, file: Option<String>| -> Option<Option<String>> {
        match file {
            None => Some(None),
            Some(f) => {
                let here = series(there_series)?;
                Some(Some(moved(&f, &normalise(there_series), &here)?))
            }
        }
    };
    let views = history
        .views
        .into_iter()
        .filter_map(|mut v| {
            let file = file_of(&v.series, v.file.take())?;
            v.series = series(&v.series)?;
            v.file = file;
            Some(v)
        })
        .collect();
    let completed = history
        .completed
        .into_iter()
        .filter_map(|mut c| {
            let file = file_of(&c.series, c.file.take())?;
            c.series = series(&c.series)?;
            c.file = file;
            Some(c)
        })
        .collect();
    let resume_points = history
        .resume_points
        .into_iter()
        .filter_map(|entry| match entry {
            ResumeEntry::Series {
                series: there,
                episode,
                season,
                position,
                duration,
                at,
            } => Some(ResumeEntry::Series {
                series: series(&there)?,
                episode,
                season,
                position,
                duration,
                at,
            }),
            ResumeEntry::File {
                file,
                position,
                duration,
                at,
            } => {
                // The longest series path there that holds the file.
                let normal = normalise(&file);
                let (there, here) = paths
                    .iter()
                    .filter(|(there, _)| {
                        normal == **there
                            || normal
                                .strip_prefix(there.as_str())
                                .is_some_and(|r| r.starts_with(['/', '\\']))
                    })
                    .max_by_key(|(there, _)| there.len())?;
                let file = if normal == *there {
                    here.clone()
                } else {
                    moved(&normal, there, here)?
                };
                Some(ResumeEntry::File {
                    file,
                    position,
                    duration,
                    at,
                })
            }
        })
        .collect();
    History {
        views,
        completed,
        resume_points,
        extra: HashMap::new(),
    }
}

/// Merges another computer's document into this library.
pub async fn merge(core: &Arc<Core>, doc: Document) -> Result<Merged, CoreError> {
    // A read, on the writer the async side has, so no runtime thread blocks.
    let here = core.store.write_async(|c| locals(c)).await?;
    let mapping = map(&doc.series, &here);
    let paths: HashMap<String, String> = mapping
        .iter()
        .map(|(there, &i)| (there.clone(), here[i].path.clone()))
        .collect();

    // Matches first, onto series here with none confirmed, so a series that
    // takes one sorts and tracks right before its history lands.
    let adopt: Vec<(u64, MatchEntry)> = doc
        .series
        .iter()
        .filter_map(|entry| {
            let local = &here[*mapping.get(&normalise(&entry.path))?];
            let wanted = entry.match_.clone()?;
            (!local.confirmed).then_some((local.id, wanted))
        })
        .collect();
    let now = time::now_secs();
    let mut touched: Vec<u64> = adopt.iter().map(|(id, _)| *id).collect();
    let matches = core
        .store
        .tx_async(move |tx| {
            let mut n = 0u64;
            for (id, wanted) in &adopt {
                if import::apply_match(tx, *id, wanted, now)? {
                    n += 1;
                }
            }
            Ok(n)
        })
        .await?;

    let history = doc.history.map(|h| translate(h, &paths));
    let ids_by_path: HashMap<&str, u64> = here.iter().map(|l| (l.path.as_str(), l.id)).collect();
    let (views, completed, resume_points) = match history {
        Some(history) => {
            touched.extend(
                history
                    .views
                    .iter()
                    .map(|v| &v.series)
                    .chain(history.completed.iter().map(|c| &c.series))
                    .filter_map(|p| ids_by_path.get(p.as_str()).copied()),
            );
            import::merge_history(core, history).await?
        }
        None => (0, 0, 0),
    };
    let merged = Merged {
        views,
        completed,
        resume_points,
        matches,
    };
    if merged.any() {
        touched.sort_unstable();
        touched.dedup();
        let images = core.paths.images_dir();
        let cards = core
            .store
            .write_async(move |c| cards::cards_for(c, &images, &touched))
            .await?;
        if !cards.is_empty() {
            core.bus.debug(
                Stage::Library,
                "series changed by another computer's history",
                EventBody::SeriesChanged { series: cards },
            );
        }
    }
    Ok(merged)
}

/// Pulls the history of the AniBeam at `base` (`http://<address>:<port>`)
/// and merges it. Nothing is merged from something that is not AniBeam or
/// does not serve its history.
pub async fn pull(core: &Arc<Core>, base: &str) -> Result<Merged, CoreError> {
    let http = reqwest::Client::builder()
        .timeout(PULL_FOR)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .map_err(|e| CoreError::internal(e.to_string()))?;
    let unreachable = |e: reqwest::Error| CoreError::internal(format!("{base}: {e}"));
    let hello: wire::Hello = http
        .get(format!("{base}{}", wire::HELLO))
        .send()
        .await
        .map_err(unreachable)?
        .json()
        .await
        .map_err(unreachable)?;
    if hello.app != wire::APP || !hello.features.iter().any(|f| f == wire::FEATURE_HISTORY) {
        return Ok(Merged::default());
    }
    let mut answer = http
        .get(format!("{base}{}", wire::HISTORY))
        .send()
        .await
        .map_err(unreachable)?;
    if !answer.status().is_success() {
        return Err(CoreError::internal(format!(
            "{base} answered {} for its history",
            answer.status()
        )));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = answer.chunk().await.map_err(unreachable)? {
        bytes.extend_from_slice(&chunk);
        if bytes.len() > DOCUMENT_MAX {
            return Err(CoreError::internal(format!(
                "{base}: the history is too big"
            )));
        }
    }
    let (doc, _) = format::parse(&bytes)?;
    let merged = merge(core, doc).await?;
    if merged.any() {
        let name = if hello.host.is_empty() {
            base
        } else {
            &hello.host
        };
        core.bus.info(
            Stage::Library,
            format!(
                "Merged history from {name}: {} watched, {} resume points, {} matches",
                merged.views + merged.completed,
                merged.resume_points,
                merged.matches
            ),
            EventBody::Notice,
        );
    }
    Ok(merged)
}

/// The device a pull is from, by its Tailscale id when the last read knows
/// its address, so its IPv4 and IPv6 addresses count as one computer.
fn device(core: &Core, base: &str) -> String {
    let ip = base
        .strip_prefix("http://")
        .and_then(|a| a.parse::<std::net::SocketAddr>().ok())
        .map(|a| super::trust::canonical(a.ip()));
    ip.and_then(|ip| {
        core.host
            .tailnet_now()
            .and_then(|t| t.peer(ip).map(|p| p.id.clone()))
    })
    .unwrap_or_else(|| base.to_string())
}

/// Whether the computer at `base` is due a pull, marking it pulled when it is.
fn due(core: &Core, base: &str) -> bool {
    let key = device(core, base);
    let mut pulled = recover(&core.host.pulled);
    if pulled
        .get(&key)
        .is_some_and(|at| at.elapsed() < AGAIN_AFTER)
    {
        return false;
    }
    pulled.insert(key, Instant::now());
    true
}

/// Pulls from `base` once it is due. A pull that fails leaves it due again, so
/// the next ask from that computer, or the next start, tries once more.
async fn pull_if_due(core: &Arc<Core>, base: &str) {
    if !due(core, base) {
        return;
    }
    if let Err(e) = pull(core, base).await {
        tracing::debug!("history: pulling from {base} failed: {e}");
        recover(&core.host.pulled).remove(&device(core, base));
    }
}

/// Pulls from every trusted AniBeam on the tailnet that is due, on a worker.
pub fn pull_all(core: &Core) {
    let Some(core) = core.arc() else { return };
    let handle = core.handle.clone();
    handle.spawn(async move {
        let reader = core.clone();
        let asked = tokio::task::spawn_blocking(move || {
            let tailnet = reader.host.tailnet(false)?;
            Some(super::peers::candidates(
                &tailnet,
                &reader.host.switched_off(),
                wire::DEFAULT_PORT,
            ))
        })
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
        for found in super::peers::find(asked).await {
            pull_if_due(&core, &found.url).await;
        }
    });
}

/// Pulls back from the computer at `from` that just asked for this one's
/// history, when it is due.
pub fn pull_back(core: &Core, from: IpAddr) {
    let from = super::trust::canonical(from);
    if !super::addr::is_tailscale(&from) {
        return;
    }
    let base = format!(
        "http://{}",
        std::net::SocketAddr::new(from, wire::DEFAULT_PORT)
    );
    let Some(core) = core.arc() else { return };
    let handle = core.handle.clone();
    handle.spawn(async move { pull_if_due(&core, &base).await });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transfer::format::{Completed, View};
    use serde_json::json;
    use std::path::Path;

    fn entry(kind: &str, path: &str, anilist: Option<u64>) -> SeriesEntry {
        serde_json::from_value(json!({
            "kind": kind,
            "path": path,
            "match": anilist.map(|id| json!({"provider": "anilist", "anilistId": id})),
        }))
        .unwrap()
    }

    fn local(id: u64, path: &str, anilist: Option<u64>, confirmed: bool) -> Local {
        Local {
            id,
            kind: "show".into(),
            path: path.into(),
            ids: Ids {
                anilist,
                ..Ids::default()
            },
            confirmed,
        }
    }

    #[test]
    fn a_series_there_is_found_here_by_its_id_then_by_its_folder() {
        let here = vec![
            local(1, "/home/a/anime/Frieren", Some(154587), true),
            local(2, "/home/a/anime/Show", None, false),
            local(3, "/home/a/anime/Twice", None, false),
            local(4, "/home/a/other/Twice", None, false),
        ];
        let there = vec![
            entry("series", "D:\\Anime\\Sousou no Frieren", Some(154587)),
            entry("series", "/mnt/media/Show", Some(1)),
            entry("series", "/mnt/media/Twice", None),
            entry("movie", "/mnt/media/Show", None),
            entry("series", "/mnt/media/Elsewhere", None),
        ];
        let found = map(&there, &here);
        assert_eq!(found.get("D:\\Anime\\Sousou no Frieren"), Some(&0));
        assert_eq!(found.get("/mnt/media/Show"), Some(&1));
        assert!(!found.contains_key("/mnt/media/Twice"), "two folders here");
        assert!(!found.contains_key("/mnt/media/Elsewhere"));
        assert_eq!(found.len(), 2, "{found:?}");
    }

    #[test]
    fn two_confirmed_matches_that_differ_are_two_shows() {
        let here = vec![local(1, "/a/Show", Some(10), true)];
        assert!(map(&[entry("series", "/b/Show", Some(20))], &here).is_empty());
        // Unconfirmed here: the folder still finds it, and its match can come over.
        let unsure = vec![local(1, "/a/Show", Some(10), false)];
        assert_eq!(
            map(&[entry("series", "/b/Show", Some(20))], &unsure).len(),
            1
        );
    }

    #[test]
    fn history_lands_on_the_paths_here() {
        let paths = HashMap::from([
            ("C:\\Anime\\Show".to_string(), "/home/a/Show".to_string()),
            ("/m/Film.mkv".to_string(), "/home/a/Film.mkv".to_string()),
        ]);
        let at = json!("2026-11-01T10:00:00Z");
        let history = History {
            views: vec![
                View {
                    series: "C:\\Anime\\Show".into(),
                    last_episode: 3.0,
                    season: None,
                    file: None,
                    at: at.clone(),
                    extra: HashMap::new(),
                },
                View {
                    series: "/m/Gone".into(),
                    last_episode: 1.0,
                    season: None,
                    file: None,
                    at: at.clone(),
                    extra: HashMap::new(),
                },
            ],
            completed: vec![Completed {
                series: "C:\\Anime\\Show".into(),
                episode: 0.0,
                season: None,
                file: Some("C:\\Anime\\Show\\Show - OP1.mkv".into()),
                at: at.clone(),
                extra: HashMap::new(),
            }],
            resume_points: vec![
                ResumeEntry::File {
                    file: "C:\\Anime\\Show\\Show - SP.mkv".into(),
                    position: 10.0,
                    duration: 20.0,
                    at: at.clone(),
                },
                ResumeEntry::File {
                    file: "/m/Film.mkv".into(),
                    position: 5.0,
                    duration: 90.0,
                    at: at.clone(),
                },
            ],
            extra: HashMap::new(),
        };
        let here = translate(history, &paths);
        assert_eq!(here.views.len(), 1);
        assert_eq!(here.views[0].series, "/home/a/Show");
        // The scanner here joins with the separator of the system here, so
        // a file moved onto a series path here does the same.
        let in_show = |name: &str| {
            Path::new("/home/a/Show")
                .join(name)
                .to_string_lossy()
                .into_owned()
        };
        assert_eq!(
            here.completed[0].file.as_deref(),
            Some(in_show("Show - OP1.mkv").as_str())
        );
        let files: Vec<_> = here
            .resume_points
            .iter()
            .map(|r| match r {
                ResumeEntry::File { file, .. } => file.clone(),
                ResumeEntry::Series { series, .. } => series.clone(),
            })
            .collect();
        assert_eq!(files, [in_show("Show - SP.mkv"), "/home/a/Film.mkv".into()]);
    }

    #[test]
    fn a_file_in_a_season_folder_keeps_its_folder_here() {
        let here = Path::new("/home/a/Show")
            .join("Season 1")
            .join("Show - 01.mkv")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            moved("/m/Show/Season 1/Show - 01.mkv", "/m/Show", "/home/a/Show").as_deref(),
            Some(here.as_str())
        );
        assert_eq!(
            moved(
                "C:\\Anime\\Show\\Season 1\\Show - 01.mkv",
                "C:\\Anime\\Show",
                "/home/a/Show"
            )
            .as_deref(),
            Some(here.as_str())
        );

        let paths = HashMap::from([("/m/Show".to_string(), "/home/a/Show".to_string())]);
        let history = History {
            views: vec![],
            completed: vec![],
            resume_points: vec![ResumeEntry::File {
                file: "/m/Show/Season 1/Show - 01.mkv".into(),
                position: 10.0,
                duration: 20.0,
                at: json!("2026-11-01T10:00:00Z"),
            }],
            extra: HashMap::new(),
        };
        let landed = translate(history, &paths);
        assert!(
            matches!(&landed.resume_points[..], [ResumeEntry::File { file, .. }] if *file == here),
            "{:?}",
            landed.resume_points
        );
    }

    #[test]
    fn a_file_that_climbs_out_of_the_series_lands_nowhere() {
        assert_eq!(
            moved("/m/Show/../Other/x.mkv", "/m/Show", "/home/a/Show"),
            None
        );
    }

    #[test]
    fn a_folder_name_two_series_there_share_decides_nothing() {
        let here = vec![local(1, "/home/a/Twice", None, false)];
        let there = vec![
            entry("series", "/mnt/a/Twice", None),
            entry("series", "/mnt/b/Twice", None),
        ];
        assert!(map(&there, &here).is_empty());
    }

    #[test]
    fn an_id_only_finds_a_series_of_the_same_kind() {
        let here = vec![local(1, "/home/a/Show", Some(7), true)];
        assert!(map(&[entry("movie", "/m/Film.mkv", Some(7))], &here).is_empty());
    }

    #[test]
    fn two_series_here_with_the_id_are_told_apart_by_the_folder() {
        let here = vec![
            local(1, "/home/a/Show", Some(7), true),
            local(2, "/home/a/Show [BD]", Some(7), true),
        ];
        let found = map(&[entry("series", "/mnt/Show [BD]", Some(7))], &here);
        assert_eq!(found.get("/mnt/Show [BD]"), Some(&1));
    }
}
