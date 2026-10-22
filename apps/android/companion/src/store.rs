//! What the app keeps in its internal storage, so it shows everything with the host away.
//!
//! ```text
//! <dir>/settings.json        the host address
//! <dir>/cache.json           the last snapshot, and when and from where it came
//! <dir>/anilist.json         airing times AniList gave while the host was away
//! <dir>/posters/<id>         one series' poster as the host sent it
//! <dir>/posters/<id>.tag     its Series::poster tag, written after the image
//! ```
//!
//! Every file is written to a temporary name, synced and renamed over the old one, so a
//! kill mid-write leaves the previous version rather than half a file.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::address::DEFAULT_ADDRESS;
use crate::anilist::Schedule;
use crate::api::Snapshot;

/// The last snapshot and where it came from.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Cache {
    /// The address it was fetched from, as typed in Settings.
    pub address: String,
    /// The host's own name, from its hello ("banditbox").
    pub host: String,
    /// The host's AniBeam version.
    #[serde(default)]
    pub host_version: String,
    /// The phone's clock when it arrived.
    pub synced_at: i64,
    pub snapshot: Snapshot,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub address: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { address: DEFAULT_ADDRESS.to_string() }
    }
}

/// AniList's schedules, by AniList id, and when they were asked for.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AniListCache {
    /// The last answer that came back. 0 for never.
    pub fetched_at: i64,
    /// The last time it was asked, answered or not. The once-an-hour limit runs off this.
    pub attempted_at: i64,
    #[serde(default)]
    pub error: Option<String>,
    pub shows: BTreeMap<u64, Schedule>,
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn open(dir: impl Into<PathBuf>) -> std::io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(dir.join("posters"))?;
        Ok(Store { dir })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn read_json<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        let bytes = fs::read(self.dir.join(name)).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    fn write_json<T: Serialize>(&self, name: &str, value: &T) -> std::io::Result<()> {
        let bytes = serde_json::to_vec(value).map_err(std::io::Error::other)?;
        write_atomic(&self.dir.join(name), &bytes)
    }

    pub fn settings(&self) -> Settings {
        self.read_json("settings.json").unwrap_or_default()
    }

    pub fn save_settings(&self, settings: &Settings) -> std::io::Result<()> {
        self.write_json("settings.json", settings)
    }

    pub fn cache(&self) -> Option<Cache> {
        self.read_json("cache.json")
    }

    pub fn save_cache(&self, cache: &Cache) -> std::io::Result<()> {
        self.write_json("cache.json", cache)
    }

    pub fn anilist(&self) -> AniListCache {
        self.read_json("anilist.json").unwrap_or_default()
    }

    pub fn save_anilist(&self, anilist: &AniListCache) -> std::io::Result<()> {
        self.write_json("anilist.json", anilist)
    }

    fn poster_file(&self, series_id: u64) -> PathBuf {
        self.dir.join("posters").join(series_id.to_string())
    }

    fn tag_file(&self, series_id: u64) -> PathBuf {
        self.dir.join("posters").join(format!("{series_id}.tag"))
    }

    /// The tag of the poster on disk, when there is a whole one.
    pub fn poster_tag(&self, series_id: u64) -> Option<String> {
        fs::read_to_string(self.tag_file(series_id)).ok()
    }

    pub fn poster(&self, series_id: u64) -> Option<Vec<u8>> {
        self.poster_tag(series_id)?;
        fs::read(self.poster_file(series_id)).ok()
    }

    /// The image first, then its tag: a tag on disk always means the image beside it is
    /// the one it names.
    pub fn save_poster(&self, series_id: u64, tag: &str, bytes: &[u8]) -> std::io::Result<()> {
        let _ = fs::remove_file(self.tag_file(series_id));
        write_atomic(&self.poster_file(series_id), bytes)?;
        write_atomic(&self.tag_file(series_id), tag.as_bytes())
    }

    /// Deletes the posters of series that are no longer in the library.
    pub fn prune_posters(&self, keep: &HashSet<u64>) {
        let Ok(entries) = fs::read_dir(self.dir.join("posters")) else { return };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let id = name.split('.').next().and_then(|s| s.parse::<u64>().ok());
            if id.is_none_or(|id| !keep.contains(&id)) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    // Unique per write: a relaunched activity can overlap the old one's sync thread.
    static WRITES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = WRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = path.with_extension(format!("tmp{}-{n}", std::process::id()));
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("anibeam-store-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_fresh_store_points_at_banditbox_and_has_nothing_cached() {
        let store = Store::open(scratch("fresh")).unwrap();
        assert_eq!(store.settings().address, "banditbox:8790");
        assert!(store.cache().is_none());
        assert_eq!(store.anilist().fetched_at, 0);
    }

    #[test]
    fn the_cache_and_settings_round_trip() {
        let store = Store::open(scratch("roundtrip")).unwrap();
        let snapshot: Snapshot = serde_json::from_str(include_str!("../../../../host-api/fixtures/snapshot.json")).unwrap();
        let cache = Cache {
            address: "10.0.2.2:8790".into(),
            host: "banditbox".into(),
            host_version: "2.1.0".into(),
            synced_at: 1_792_242_100,
            snapshot,
        };
        store.save_cache(&cache).unwrap();
        assert_eq!(store.cache().unwrap(), cache);
        store.save_settings(&Settings { address: "10.0.2.2:8790".into() }).unwrap();
        assert_eq!(store.settings().address, "10.0.2.2:8790");
    }

    #[test]
    fn posters_are_whole_or_absent_and_prune_drops_strangers() {
        let store = Store::open(scratch("posters")).unwrap();
        store.save_poster(12, "abc", b"jpeg bytes").unwrap();
        store.save_poster(3, "def", b"png bytes").unwrap();
        assert_eq!(store.poster_tag(12).as_deref(), Some("abc"));
        assert_eq!(store.poster(12).unwrap(), b"jpeg bytes");
        // An image without its tag is half a write: not a poster.
        fs::remove_file(store.tag_file(3)).unwrap();
        assert!(store.poster(3).is_none());
        store.prune_posters(&HashSet::from([12]));
        assert!(store.poster_file(12).exists());
        assert!(!store.poster_file(3).exists());
    }
}
