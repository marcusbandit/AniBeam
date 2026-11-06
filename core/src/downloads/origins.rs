//! Which torrent each downloaded file came from (#241), so On disk can
//! tell this release from another by the torrent rather than by the name.
//!
//! The torrent client is the one that knows: a file in the library is
//! recorded once the client lists a torrent that holds it. The core reads
//! the client's torrents when the Downloads page does, and once after a
//! scan that brought in files, but only through a client already running:
//! a scan never starts one. Nothing here polls. A file the client never
//! listed, or one from before this, has no origin, and On disk reads its
//! name the way it always has.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::*;
use crate::core::Core;
use crate::downloads::qbt::QbtClient;

/// One recorded origin: the torrent's info hash, lower case, and its name.
#[derive(Clone, Debug, PartialEq)]
pub struct Origin {
    pub hash: String,
    pub name: String,
}

/// The origins of these paths, by path. A path with none is absent.
pub fn of(conn: &Connection, paths: &[&str]) -> Result<HashMap<String, Origin>, CoreError> {
    let mut out = HashMap::new();
    let mut stmt =
        conn.prepare_cached("SELECT info_hash, torrent_name FROM file_origins WHERE path = ?1")?;
    for path in paths {
        if let Some((hash, name)) = stmt
            .query_row(params![path], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .optional()?
        {
            out.insert((*path).to_string(), Origin { hash, name });
        }
    }
    Ok(out)
}

/// Whether a search's release is the torrent a file came from: its magnet
/// names the same info hash, or its title is the torrent's name, with a
/// file extension and case set aside, which is how a torrent added from a
/// `.torrent` link is named.
pub fn is_release(origin: &Origin, title: &str, link: &str) -> bool {
    if let Some(hash) = crate::downloads::replace::magnet_hash(link) {
        return hash.eq_ignore_ascii_case(&origin.hash);
    }
    let norm = |s: &str| {
        let s = s.trim();
        let s = match s.rsplit_once('.') {
            Some((stem, ext)) if VIDEO.contains(&ext.to_ascii_lowercase().as_str()) => stem,
            _ => s,
        };
        s.trim().to_lowercase()
    };
    norm(title) == norm(&origin.name)
}

const VIDEO: [&str; 7] = ["mkv", "mp4", "avi", "m4v", "webm", "ts", "wmv"];

/// Records every library file the client's torrents hold that has no origin
/// yet. A torrent of one file is its content path; a batch is read file by
/// file, and only when some unrecorded library file sits under its folder,
/// so a client full of finished batches costs nothing once they are known.
pub async fn record_from(
    core: &Arc<Core>,
    client: &QbtClient,
    torrents: &[Torrent],
) -> Result<u64, CoreError> {
    let unknown: HashSet<String> = core
        .store
        .write_async(|c| {
            let mut stmt = c.prepare(
                "SELECT f.path FROM files f
                 WHERE NOT EXISTS (SELECT 1 FROM file_origins o WHERE o.path = f.path)",
            )?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<HashSet<_>, _>>()?;
            Ok(rows)
        })
        .await?;
    if unknown.is_empty() {
        return Ok(0);
    }
    let mut found: Vec<(String, String, String)> = Vec::new();
    for t in torrents {
        let hash = t.hash.to_ascii_lowercase();
        let content = norm_path(&t.content_path);
        if unknown.contains(&content) {
            found.push((content, hash, t.name.clone()));
            continue;
        }
        let folder = format!("{}/", content.trim_end_matches('/'));
        if !unknown.iter().any(|p| p.starts_with(&folder)) {
            continue;
        }
        let files = match client.files(&t.hash).await {
            Ok(files) => files,
            Err(e) => {
                tracing::debug!("files of {} not read for origins: {e}", t.name);
                continue;
            }
        };
        let base = norm_path(&t.save_path);
        for f in files {
            let path = format!("{}/{}", base.trim_end_matches('/'), norm_path(&f.name));
            if unknown.contains(&path) {
                found.push((path, hash.clone(), t.name.clone()));
            }
        }
    }
    let count = found.len() as u64;
    if count > 0 {
        let now = crate::time::now_secs();
        core.store
            .tx_async(move |tx| {
                let mut stmt = tx.prepare(
                    "INSERT OR REPLACE INTO file_origins (path, info_hash, torrent_name, recorded_at)
                     VALUES (?1, ?2, ?3, ?4)",
                )?;
                for (path, hash, name) in &found {
                    stmt.execute(params![path, hash, name, now])?;
                }
                Ok(())
            })
            .await?;
    }
    Ok(count)
}

/// After a scan brought files in: one read of the client's torrents, only
/// through a client already running, so a scan never starts one. Not a
/// job; a failure is a debug line and the files stay unrecorded until the
/// next read.
pub fn after_scan(core: &Core) {
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
        match record_from(&owner, &client, &torrents).await {
            Ok(0) => {}
            Ok(n) => tracing::debug!("recorded the torrent of {n} files"),
            Err(e) => tracing::debug!("origins after a scan not recorded: {e}"),
        }
    });
}

/// A client's path with forward slashes, the way the library stores one on
/// this machine.
fn norm_path(p: &str) -> String {
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
        let o = origin(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "[Grp] Show - 01 [1080p].mkv",
        );
        assert!(is_release(
            &o,
            "anything",
            "magnet:?xt=urn:btih:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&dn=x"
        ));
        assert!(!is_release(
            &o,
            "[Grp] Show - 01 [1080p]",
            "magnet:?xt=urn:btih:ffffffffffffffffffffffffffffffffffffffff"
        ));
        assert!(is_release(
            &o,
            "[grp] show - 01 [1080p]",
            "https://example.invalid/1.torrent"
        ));
        assert!(!is_release(
            &o,
            "[Grp] Show - 01 [720p]",
            "https://example.invalid/2.torrent"
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
            torrent("BBB", "Batch", "/lib/Show/Batch"),
            torrent("CCC", "Elsewhere", "/downloads/Elsewhere"),
        ];
        let n = core
            .handle
            .block_on(record_from(&core, &client, &torrents))
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
            origin("bbb", "Batch")
        );

        let asked = http.requests().len();
        let again = core
            .handle
            .block_on(record_from(&core, &client, &torrents))
            .unwrap();
        assert_eq!(again, 0);
        assert_eq!(http.requests().len(), asked);
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
