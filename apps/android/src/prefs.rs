//! The phone's own choices: whether onboarding is done, Wi-Fi only, and the storage limit.
//! One small JSON file beside the core's folder.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 12 episodes at about 1.5 GB each.
pub const SUGGESTED_LIMIT_GB: u32 = 18;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub onboarded: bool,
    /// Downloads wait for Wi-Fi. On until the user says otherwise.
    pub wifi_only: bool,
    /// Offered during onboarding, applied only when the user turns it on.
    pub limit_on: bool,
    pub limit_gb: u32,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { onboarded: false, wifi_only: true, limit_on: false, limit_gb: SUGGESTED_LIMIT_GB }
    }
}

fn file(data_dir: &Path) -> PathBuf {
    data_dir.join("phone.json")
}

pub fn load(data_dir: &Path) -> Prefs {
    std::fs::read(file(data_dir)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

/// Written whole to a temporary file and renamed over the old one, so a kill mid-write
/// leaves the last good file.
pub fn save(data_dir: &Path, prefs: &Prefs) -> std::io::Result<()> {
    let path = file(data_dir);
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(prefs)?)?;
    std::fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_phone_is_wifi_only_with_the_limit_offered_but_off() {
        let p = Prefs::default();
        assert!(p.wifi_only && !p.limit_on && !p.onboarded);
        assert_eq!(p.limit_gb, 18);
    }

    #[test]
    fn prefs_survive_a_save_and_a_load() {
        let dir = std::env::temp_dir().join(format!("anibeam-prefs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = Prefs { onboarded: true, wifi_only: false, limit_on: true, limit_gb: 30 };
        save(&dir, &p).unwrap();
        assert_eq!(load(&dir), p);
        let _ = std::fs::remove_dir_all(dir);
    }
}
