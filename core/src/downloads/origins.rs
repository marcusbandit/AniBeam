//! Which torrent each downloaded file came from (#241), so On disk can
//! tell this release from another by the torrent rather than by the name.
//!
//! The torrent client is the one that knows: a file in the library is
//! recorded once the client lists a finished torrent that holds it, and
//! again when the file is newer than its record (downloaded over). The core reads
//! the client's torrents when the Downloads page does, and once after a
//! scan that brought in files, but only through a client already running:
//! a scan never starts one. Nothing here polls. A file the client never
//! listed, or one from before this, has no origin, and On disk reads its
//! name the way it always has.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

use rusqlite::{Connection, params, params_from_iter};

use crate::contract::*;
use crate::core::Core;
use crate::downloads::qbt::QbtClient;
use crate::store::sql::placeholders;

/// One recorded origin: the torrent's info hash, lower case, and its name.
#[derive(Clone, Debug, PartialEq)]
pub struct Origin {
    pub hash: String,
    pub name: String,
}

/// The origins of these paths, by path, in one read. A path with none is
/// absent.
pub fn of(conn: &Connection, paths: &[&str]) -> Result<HashMap<String, Origin>, CoreError> {
    let mut out = HashMap::new();
    for chunk in paths.chunks(500) {
        let sql = format!(
            "SELECT path, info_hash, torrent_name FROM file_origins WHERE path IN ({})",
            placeholders(chunk.len())
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                Origin {
                    hash: r.get(1)?,
                    name: r.get(2)?,
                },
            ))
        })?;
        for row in rows {
            let (path, origin) = row?;
            out.insert(path, origin);
        }
    }
    Ok(out)
}

/// Whether a search's release is the torrent a file came from: its magnet
/// names the same info hash, or, for a `.torrent` link, which names none,
/// the release reads as the torrent the way it would read as the file
/// (`facts::same_release`, with the torrent's name in the file's place).
pub fn is_release(origin: &Origin, title: &str, link: &str, facts: &ReleaseFacts) -> bool {
    match crate::foundation::magnet::hash(link) {
        Some(hash) => hash.eq_ignore_ascii_case(&origin.hash),
        None => super::facts::same_release(title, facts, &origin.name),
    }
}

/// The finished torrents already read since the library last gained files:
/// a torrent's files do not change once it is done, so it is read once,
/// and a scan that brings files in clears this for the next read.
static SEEN: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Default::default);

fn seen() -> std::sync::MutexGuard<'static, HashSet<String>> {
    SEEN.lock().unwrap_or_else(|e| e.into_inner())
}

/// Records which finished torrent holds each library file whose origin is
/// unknown, or older than the file. A torrent that lists its own files (the
/// phone's engine, #620) is read from that list; otherwise a one-file
/// torrent is its content path and a batch is asked of `client` file by
/// file. A torrent still downloading is left for a later read, and a
/// finished one is read once until the library gains files again, so a
/// client full of done torrents costs no request at all. Two torrents
/// holding one path (a cross-seed) leave the first one read.
pub async fn record_from(
    core: &Arc<Core>,
    client: Option<&QbtClient>,
    torrents: &[Torrent],
) -> Result<u64, CoreError> {
    let fresh: Vec<&Torrent> = {
        let seen = seen();
        torrents
            .iter()
            .filter(|t| t.progress >= 1.0 && !seen.contains(&t.hash.to_ascii_lowercase()))
            .collect()
    };
    if fresh.is_empty() {
        return Ok(0);
    }
    let (unknown, sources) = core
        .store
        .write_async(|c| {
            let mut stmt = c.prepare(
                "SELECT f.path FROM files f
                 LEFT JOIN file_origins o ON o.path = f.path
                 WHERE o.path IS NULL OR o.recorded_at < f.mtime",
            )?;
            let unknown = stmt
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<HashSet<_>, _>>()?;
            let mut stmt = c.prepare("SELECT path FROM sources")?;
            let sources = stmt
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok((unknown, sources))
        })
        .await?;
    let local = Local::new(&sources);
    let mut found: Vec<(String, String, String)> = Vec::new();
    let mut taken: HashSet<String> = HashSet::new();
    for t in fresh {
        let hash = t.hash.to_ascii_lowercase();
        seen().insert(hash.clone());
        if unknown.is_empty() {
            continue;
        }
        let files: Cow<[TorrentFile]> = if !t.files.is_empty() {
            Cow::Borrowed(&t.files)
        } else {
            let content = local.path(&t.content_path);
            if unknown.contains(&content) {
                if taken.insert(content.clone()) {
                    found.push((content, hash, t.name.clone()));
                }
                continue;
            }
            let folder = format!("{}/", content.trim_end_matches('/'));
            let Some(client) = client else { continue };
            if !unknown.iter().any(|p| p.starts_with(&folder)) {
                continue;
            }
            match client.files(&t.hash).await {
                Ok(files) => Cow::Owned(files),
                Err(e) => {
                    tracing::debug!("files of {} not read for origins: {e}", t.name);
                    seen().remove(&hash);
                    continue;
                }
            }
        };
        let base = local.path(&t.save_path);
        for f in files.iter() {
            let path = format!("{}/{}", base.trim_end_matches('/'), slashes(&f.name));
            if unknown.contains(&path) && taken.insert(path.clone()) {
                found.push((path, hash.clone(), t.name.clone()));
            }
        }
    }
    let now = crate::time::now_secs();
    let count = found.len() as u64;
    core.store
        .tx_async(move |tx| {
            {
                let mut stmt = tx.prepare(
                    "INSERT OR REPLACE INTO file_origins (path, info_hash, torrent_name, recorded_at)
                     VALUES (?1, ?2, ?3, ?4)",
                )?;
                for (path, hash, name) in &found {
                    stmt.execute(params![path, hash, name, now])?;
                }
            }
            // A file the library no longer holds takes its origin with it
            tx.execute(
                "DELETE FROM file_origins WHERE path NOT IN (SELECT path FROM files)",
                [],
            )?;
            Ok(())
        })
        .await?;
    Ok(count)
}

/// After a scan brought files in: one read of the client's torrents, only
/// through a client already running, so a scan never starts one. The
/// finished torrents are read again, since one of them may hold the new
/// files. Not a job; a failure is a debug line and the files stay
/// unrecorded until the next read.
pub fn after_scan(core: &Core) {
    seen().clear();
    #[cfg(feature = "phone")]
    crate::phone::after_scan(core);
    let Some(client) = core.downloads.state().qbt.clone() else {
        return;
    };
    let Ok(owner) = core.owner() else { return };
    core.handle.spawn(async move {
        let torrents = match client.torrents().await {
            Ok(t) => t,
            Err(e) => {
                tracing::debug!("origins after a scan skipped: {e}");
                return;
            }
        };
        logged(
            record_from(&owner, Some(&client), &torrents).await,
            "after a scan",
        );
    });
}

/// The debug line a read of origins off the caller's path leaves: nothing
/// when no file was new, else how many, or why none were.
pub(crate) fn logged(recorded: Result<u64, CoreError>, when: &str) {
    match recorded {
        Ok(0) => {}
        Ok(n) => tracing::debug!("recorded the torrent of {n} files {when}"),
        Err(e) => tracing::debug!("origins {when} not recorded: {e}"),
    }
}

/// A client's path as the library writes it: forward slashes on Windows,
/// and a source the library reaches through a symlink written the way the
/// library added it, so the client's real path and the library's agree.
struct Local {
    /// Each source whose real path differs from the one the library keeps:
    /// (real, as kept).
    links: Vec<(String, String)>,
}

impl Local {
    fn new(sources: &[String]) -> Local {
        let links = sources
            .iter()
            .filter_map(|s| {
                let real = std::fs::canonicalize(s).ok()?;
                let real = slashes(&real.to_string_lossy());
                (real != *s).then(|| (real, s.clone()))
            })
            .collect();
        Local { links }
    }

    fn path(&self, client: &str) -> String {
        let p = slashes(client);
        for (real, kept) in &self.links {
            if let Some(rest) = p.strip_prefix(real.as_str())
                && (rest.is_empty() || rest.starts_with('/'))
            {
                return format!("{kept}{rest}");
            }
        }
        p
    }
}

fn slashes(p: &str) -> String {
    if cfg!(windows) {
        p.replace('\\', "/")
    } else {
        p.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;

    fn origin(hash: &str, name: &str) -> Origin {
        Origin {
            hash: hash.into(),
            name: name.into(),
        }
    }

    /// A magnet settles it by the hash alone; anything else by the
    /// torrent's name, with the extension and case set aside.
    #[test]
    fn a_release_is_its_torrent_by_hash_or_by_name() {
        let o = origin(&"a".repeat(40), "[Grp] Show - 01 [1080p].mkv");
        let facts = |title: &str| super::super::facts::read(&release(title, ""));
        let magnet = |h: &str| format!("magnet:?xt=urn:btih:{h}&dn=x");
        assert!(is_release(
            &o,
            "anything",
            &magnet(&"A".repeat(40)),
            &facts("anything")
        ));
        let t = "[Grp] Show - 01 [1080p]";
        assert!(!is_release(&o, t, &magnet(&"f".repeat(40)), &facts(t)));
        // A .torrent link reads as the torrent the way it reads as a file:
        // case, dots and underscores aside
        let t = "[grp]_show_-_01_[1080p]";
        assert!(is_release(
            &o,
            t,
            "https://example.invalid/1.torrent",
            &facts(t)
        ));
        let t = "[Grp] Show - 01 [720p]";
        assert!(!is_release(
            &o,
            t,
            "https://example.invalid/2.torrent",
            &facts(t)
        ));
    }

    fn core_with(http: Arc<FakeHttp>) -> (tempfile::TempDir, Arc<Core>) {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            http,
            crate::trackers::Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        (dir, core)
    }

    /// One show with episode 1 on its own and episode 2 inside a batch
    /// folder, the way two torrents would leave them.
    fn library(core: &Core) -> u64 {
        core.store
            .write(|c| {
                c.execute(
                    "INSERT INTO sources (path, available, added_at) VALUES ('/lib', 1, 0)",
                    [],
                )?;
                c.execute(
                    "INSERT INTO series (source_id, kind, path, folder_name, added_at)
                     VALUES (1, 'show', '/lib/Show', 'Show', 0)",
                    [],
                )?;
                for (path, n) in [
                    ("/lib/Show/[Grp] Show - 01 [1080p].mkv", 1.0),
                    ("/lib/Show/Batch/Show - 02.mkv", 2.0),
                ] {
                    c.execute(
                        "INSERT INTO files (series_id, path, size, mtime, kind, number, label, episode_key, sidecars, seen_at)
                         VALUES (1, ?1, 0, 0, 'episode', ?2, 'EP', ?3, '[]', 0)",
                        params![path, n, format!("{n}")],
                    )?;
                }
                Ok(1)
            })
            .unwrap()
    }

    fn torrent(hash: &str, name: &str, content: &str) -> Torrent {
        let mut t = crate::downloads::qbt::tests_torrent();
        t.progress = 1.0;
        t.hash = hash.into();
        t.name = name.into();
        t.save_path = "/lib/Show".into();
        t.content_path = content.into();
        t
    }

    /// A one-file torrent is its content path; a batch is read file by
    /// file. A torrent elsewhere records nothing, and a second pass over
    /// files already known asks the client for nothing.
    #[test]
    fn the_client_says_which_torrent_holds_each_file() {
        let http = FakeHttp::new();
        let (_dir, core) = core_with(http.clone());
        library(&core);
        http.push_for(
            "torrents/files",
            200,
            r#"[{"name":"Batch/Show - 02.mkv","size":1,"progress":1}]"#,
        );
        let client = QbtClient::new("http://localhost:9090", "me", "pw", http.clone());
        let torrents = vec![
            torrent(
                "AAA",
                "[Grp] Show - 01 [1080p].mkv",
                "/lib/Show/[Grp] Show - 01 [1080p].mkv",
            ),
            torrent("R1BBB", "Batch", "/lib/Show/Batch"),
            torrent("R1CCC", "Elsewhere", "/downloads/Elsewhere"),
        ];
        let n = core
            .handle
            .block_on(record_from(&core, Some(&client), &torrents))
            .unwrap();
        assert_eq!(n, 2);
        let known = core
            .store
            .read(|c| {
                of(
                    c,
                    &[
                        "/lib/Show/[Grp] Show - 01 [1080p].mkv",
                        "/lib/Show/Batch/Show - 02.mkv",
                    ],
                )
            })
            .unwrap();
        assert_eq!(
            known["/lib/Show/[Grp] Show - 01 [1080p].mkv"],
            origin("aaa", "[Grp] Show - 01 [1080p].mkv")
        );
        assert_eq!(
            known["/lib/Show/Batch/Show - 02.mkv"],
            origin("r1bbb", "Batch")
        );

        let asked = http.requests().len();
        let again = core
            .handle
            .block_on(record_from(&core, Some(&client), &torrents))
            .unwrap();
        assert_eq!(again, 0);
        assert_eq!(http.requests().len(), asked);

        // Downloaded over by another torrent: the file is newer than its
        // record, so the new torrent is the origin. A file the library no
        // longer holds loses its origin.
        core.store
            .write(|c| {
                c.execute(
                    "UPDATE files SET mtime = (SELECT recorded_at FROM file_origins LIMIT 1) + 10
                     WHERE number = 1",
                    [],
                )?;
                c.execute("DELETE FROM files WHERE number = 2", [])?;
                Ok(())
            })
            .unwrap();
        seen().clear();
        let over = vec![torrent(
            "R1DDD",
            "[Grp] Show - 01 [1080p].mkv",
            "/lib/Show/[Grp] Show - 01 [1080p].mkv",
        )];
        core.handle
            .block_on(record_from(&core, Some(&client), &over))
            .unwrap();
        let known = core
            .store
            .read(|c| {
                of(
                    c,
                    &[
                        "/lib/Show/[Grp] Show - 01 [1080p].mkv",
                        "/lib/Show/Batch/Show - 02.mkv",
                    ],
                )
            })
            .unwrap();
        assert_eq!(known.len(), 1, "{known:?}");
        assert_eq!(known["/lib/Show/[Grp] Show - 01 [1080p].mkv"].hash, "r1ddd");
        core.shutdown();
    }

    /// The phone's engine lists each torrent's files with it and has no
    /// client to ask (#620): a batch written straight into the show folder
    /// is read from that list, without a request.
    #[test]
    fn a_torrent_that_lists_its_files_is_read_from_them() {
        let http = FakeHttp::new();
        let (_dir, core) = core_with(http.clone());
        library(&core);
        let file = |name: &str| TorrentFile {
            name: name.into(),
            size: 1,
            progress: 1.0,
            priority: 1,
            episode: None,
        };
        // librqbit writes a batch's files straight into its save path, with no root
        // folder, so the content path the engine reports holds none of them.
        let mut batch = torrent("EEE", "Show Batch", "/lib/Show/Batch/Show Batch");
        batch.save_path = "/lib/Show/Batch".into();
        batch.files = vec![file("Show - 02.mkv"), file("Show - 03.mkv")];
        let mut one = torrent(
            "FFF",
            "[Grp] Show - 01 [1080p].mkv",
            "/lib/Show/[Grp] Show - 01 [1080p].mkv",
        );
        one.files = vec![file("[Grp] Show - 01 [1080p].mkv")];
        // Without a list of its own and no client, a batch records nothing, even one
        // whose folder holds a file with no origin.
        let unlisted = torrent("GGG", "Batch", "/lib/Show/Batch");
        let n = core
            .handle
            .block_on(record_from(&core, None, &[batch, one, unlisted]))
            .unwrap();
        assert_eq!(n, 2);
        assert!(http.requests().is_empty());
        let known = core
            .store
            .read(|c| {
                of(
                    c,
                    &[
                        "/lib/Show/[Grp] Show - 01 [1080p].mkv",
                        "/lib/Show/Batch/Show - 02.mkv",
                    ],
                )
            })
            .unwrap();
        assert_eq!(
            known["/lib/Show/Batch/Show - 02.mkv"],
            origin("eee", "Show Batch")
        );
        assert_eq!(
            known["/lib/Show/[Grp] Show - 01 [1080p].mkv"],
            origin("fff", "[Grp] Show - 01 [1080p].mkv")
        );
        core.shutdown();
    }

    fn release(title: &str, link: &str) -> Release {
        serde_json::from_value(serde_json::json!({ "title": title, "link": link })).unwrap()
    }

    /// With the torrent known, On disk is that torrent's alone: a release
    /// whose name reads the same but is another torrent is only another
    /// release on disk, and the known one is on disk whatever its name.
    #[test]
    fn on_disk_goes_by_the_torrent_where_one_is_known() {
        let http = FakeHttp::new();
        let (_dir, core) = core_with(http.clone());
        library(&core);
        core.store
            .write(|c| {
                c.execute(
                    "INSERT INTO file_origins (path, info_hash, torrent_name, recorded_at)
                     VALUES ('/lib/Show/[Grp] Show - 01 [1080p].mkv', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'renamed upload', 0)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        let items = vec![
            release(
                "[Grp] Show - 01 [1080p]",
                "magnet:?xt=urn:btih:0000000000000000000000000000000000000000",
            ),
            release(
                "[Grp] Show - 01 [1080p]",
                "magnet:?xt=urn:btih:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            ),
            // No origin for episode 2: the name still decides
            release("Show - 02", "https://example.invalid/2.torrent"),
        ];
        let (facts, show) = crate::downloads::annotate(&core, "Show", &items);
        assert!(show.is_some());
        assert_eq!((facts[0].on_disk, facts[0].other_on_disk), (false, true));
        assert_eq!((facts[1].on_disk, facts[1].other_on_disk), (true, false));
        assert!(facts[2].on_disk, "{:?}", facts[2]);
        core.shutdown();
    }
}
