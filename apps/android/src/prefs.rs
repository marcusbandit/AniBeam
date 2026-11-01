//! The phone's own choices: whether onboarding is done, Wi-Fi only, the storage limit, and
//! which version's notes were last read. One small JSON file beside the core's folder.

use std::path::{Path, PathBuf};

use anibeam_core::changelog::Seen;
use serde::{Deserialize, Serialize};

/// 12 episodes at about 1.5 GB each.
pub const SUGGESTED_LIMIT_GB: f64 = 18.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub onboarded: bool,
    /// Downloads wait for Wi-Fi. On until the user says otherwise.
    pub wifi_only: bool,
    /// Offered during onboarding, applied only when the user turns it on.
    pub limit_on: bool,
    /// Any amount from 1 GB to the phone's whole storage, set with a slider.
    pub limit_gb: f64,
    /// The newest version whose What's new was closed, and the one before it.
    pub notes: Seen,
    /// How an episode from the other device streams.
    pub stream_quality: crate::stream::Quality,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { onboarded: false, wifi_only: true, limit_on: false, limit_gb: SUGGESTED_LIMIT_GB, notes: Seen::default(), stream_quality: Default::default() }
    }
}

fn file(data_dir: &Path) -> PathBuf {
    data_dir.join("phone.json")
}

/// A slider's value as a cap: half gigabytes up to 100 GB, whole ones past it, so a drag
/// lands on numbers a person reads easily.
pub fn snap_limit(gb: f64) -> f64 {
    let gb = gb.max(1.0);
    if gb < 100.0 { (gb * 2.0).round() / 2.0 } else { gb.round() }
}

/// The cap as a person reads it: `18 GB`, `31.5 GB`.
pub fn limit_label(gb: f64) -> String {
    if gb.fract() == 0.0 { format!("{gb:.0} GB") } else { format!("{gb:.1} GB") }
}

/// Whether the phone has saved its settings before.
pub fn exists(data_dir: &Path) -> bool {
    file(data_dir).exists()
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
        assert_eq!(p.limit_gb, 18.0);
    }

    #[test]
    fn a_slid_cap_snaps_to_half_gigabytes_then_whole_ones() {
        assert_eq!(snap_limit(31.27), 31.5);
        assert_eq!(snap_limit(0.2), 1.0);
        assert_eq!(snap_limit(212.6), 213.0);
        assert_eq!(limit_label(18.0), "18 GB");
        assert_eq!(limit_label(31.5), "31.5 GB");
    }

    #[test]
    fn an_old_whole_number_limit_still_loads() {
        let p: Prefs = serde_json::from_str(r#"{"onboarded":true,"limit_gb":18}"#).unwrap();
        assert_eq!(p.limit_gb, 18.0);
        // A file from before What's new has read no notes.
        assert_eq!(p.notes, Seen::default());
    }

    #[test]
    fn prefs_survive_a_save_and_a_load() {
        let dir = std::env::temp_dir().join(format!("anibeam-prefs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let notes = Seen { version: "2.1.0".into(), from: "2.0.8".into() };
        let p = Prefs { onboarded: true, wifi_only: false, limit_on: true, limit_gb: 31.2, notes };
        save(&dir, &p).unwrap();
        assert_eq!(load(&dir), p);
        let _ = std::fs::remove_dir_all(dir);
    }
}
