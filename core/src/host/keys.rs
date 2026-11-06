//! This AniBeam's own key, the keys it is paired with, and the proofs it has
//! already seen (#555). The scheme itself is `anibeam_host_api::proof`.
//!
//! The key is made once and kept with the other secrets. The paired keys are
//! kept in the store; pairing (#557) is what adds them. A request may carry a
//! proof: one that does not check out is refused whatever its address, one
//! from a paired key is known to be that device, and one from any other key
//! changes nothing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use anibeam_host_api::proof::{self, DeviceKey, ProofError};
use serde::{Deserialize, Serialize};

use crate::contract::CoreError;
use crate::core::Core;
use crate::store::settings;
use crate::sync::recover;
use crate::trackers::secrets::Secrets;

/// Where the key's seed is kept among the secrets.
pub const SECRET: &str = "anibeam-device-key";
/// More fresh proofs than this at once and new ones are refused, so a
/// device signing as fast as it can cannot grow the list without end.
const SEEN_MAX: usize = 100_000;

/// One AniBeam this one is paired with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Paired {
    /// Its public key, in hex.
    pub key: String,
    /// Its name, as it said at pairing.
    pub name: String,
}

/// What a request's proof says about who sent it.
#[derive(Clone, Debug, PartialEq)]
pub enum Proof {
    /// No proof, or this machine has no key to check one against: the
    /// request is judged by its address alone.
    None,
    /// A good proof from a device this one is paired with, by name.
    Paired(String),
    /// A good proof from a key this one is not paired with.
    Unpaired,
    /// A proof that does not check out, and what the device is told.
    Refused(&'static str),
}

pub struct Keys {
    mine: OnceLock<Option<Arc<DeviceKey>>>,
    paired: Mutex<Vec<Paired>>,
    /// Each fresh proof's key and nonce, with the time it was made.
    seen: Mutex<HashMap<(String, String), i64>>,
}

impl Default for Keys {
    fn default() -> Self {
        Keys::new()
    }
}

impl Keys {
    pub fn new() -> Keys {
        Keys {
            mine: OnceLock::new(),
            paired: Mutex::new(Vec::new()),
            seen: Mutex::new(HashMap::new()),
        }
    }

    /// This machine's key, made and kept on first use. None when the secrets
    /// can be neither read nor written; it is then asked again next time.
    /// Blocks on the keyring the first time.
    pub fn mine(&self, secrets: &Secrets) -> Option<Arc<DeviceKey>> {
        if let Some(key) = self.mine.get() {
            return key.clone();
        }
        let key = load_or_make(secrets).map(Arc::new);
        if key.is_some() {
            let _ = self.mine.set(key.clone());
        }
        key
    }

    /// This machine's key if it was already read, without touching the
    /// secrets.
    pub fn mine_now(&self) -> Option<Arc<DeviceKey>> {
        self.mine.get().cloned().flatten()
    }

    /// Uses `key` as this machine's key, for a test.
    pub fn set_mine(&self, key: DeviceKey) {
        let _ = self.mine.set(Some(Arc::new(key)));
    }

    pub fn set_paired(&self, list: Vec<Paired>) {
        *recover(&self.paired) = list;
    }

    pub fn paired(&self) -> Vec<Paired> {
        recover(&self.paired).clone()
    }

    /// What the proof in `header`, if any, says of a `method` request for
    /// `path` at this machine's time `now`.
    pub fn judge(&self, header: Option<&str>, method: &str, path: &str, now: i64) -> Proof {
        let (Some(header), Some(mine)) = (header, self.mine_now()) else {
            return Proof::None;
        };
        let proven = match proof::check(header, &mine.public(), method, path, now) {
            Ok(proven) => proven,
            Err(ProofError::Unreadable) => return Proof::Refused(UNREADABLE),
            Err(ProofError::Forged) => return Proof::Refused(FORGED),
            Err(ProofError::Stale) => return Proof::Refused(STALE),
        };
        {
            let mut seen = recover(&self.seen);
            seen.retain(|_, at| (now - *at).abs() <= proof::FRESH_FOR);
            if seen.len() >= SEEN_MAX {
                return Proof::Refused(BUSY);
            }
            let id = (proven.key.clone(), proven.nonce.clone());
            if seen.contains_key(&id) {
                return Proof::Refused(REPLAYED);
            }
            seen.insert(id, proven.time);
        }
        match recover(&self.paired).iter().find(|p| p.key == proven.key) {
            Some(p) => Proof::Paired(p.name.clone()),
            None => Proof::Unpaired,
        }
    }
}

const UNREADABLE: &str = "That request's proof of its device could not be read.";
const FORGED: &str = "That request's proof of its device did not check out.";
const STALE: &str =
    "That request's proof of its device is too old. Check the clock on that device.";
const REPLAYED: &str = "That request's proof of its device was already used.";
const BUSY: &str = "Too many requests at once. Try again in a moment.";

fn load_or_make(secrets: &Secrets) -> Option<DeviceKey> {
    match secrets.get(SECRET, None) {
        Ok(Some((seed, _))) => match proof::unhex::<32>(&seed) {
            Some(seed) => return Some(DeviceKey::from_seed(seed)),
            // A seed that does not read is not replaced: another key would
            // undo every pairing made with this one.
            None => {
                tracing::warn!("host: the stored device key does not read; requests go unsigned");
                return None;
            }
        },
        Ok(None) => {}
        Err(e) => {
            tracing::warn!("host: the device key could not be read: {e}");
            return None;
        }
    }
    let key = DeviceKey::from_seed(rand::random::<[u8; 32]>());
    match secrets.set(SECRET, &proof::hex(&key.seed())) {
        Ok(_) => Some(key),
        Err(e) => {
            tracing::warn!("host: the device key could not be stored: {e}");
            None
        }
    }
}

/// Reads the paired keys from the store into the host.
pub fn load(core: &Core) {
    match core
        .store
        .read(|c| settings::get_or_default::<Vec<Paired>>(c, settings::PAIRED_DEVICES))
    {
        Ok(list) => core.host.keys.set_paired(list),
        // The list stays as it was rather than forgetting every pairing.
        Err(e) => tracing::warn!("host: the paired devices could not be read: {e}"),
    }
}

/// Pairs with the device whose public key is `key`, under `name`, or renames
/// it when already paired. Kept in the store.
pub fn pair(core: &Core, key: &str, name: &str) -> Result<(), CoreError> {
    if proof::unhex::<32>(key).is_none() {
        return Err(CoreError::Invalid {
            field: "key".into(),
            message: format!("{key:?} is not a device key"),
        });
    }
    let entry = Paired {
        key: key.to_ascii_lowercase(),
        name: name.to_string(),
    };
    let list = core.store.write(move |c| {
        let mut list: Vec<Paired> = settings::get_or_default(c, settings::PAIRED_DEVICES)?;
        list.retain(|p| p.key != entry.key);
        list.push(entry);
        settings::set(c, settings::PAIRED_DEVICES, &list)?;
        Ok(list)
    })?;
    core.host.keys.set_paired(list);
    Ok(())
}

/// Forgets the pairing with `key`.
pub fn unpair(core: &Core, key: &str) -> Result<(), CoreError> {
    let key = key.to_ascii_lowercase();
    let list = core.store.write(move |c| {
        let mut list: Vec<Paired> = settings::get_or_default(c, settings::PAIRED_DEVICES)?;
        list.retain(|p| p.key != key);
        settings::set(c, settings::PAIRED_DEVICES, &list)?;
        Ok(list)
    })?;
    core.host.keys.set_paired(list);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn keys() -> (Keys, String) {
        let k = Keys::new();
        k.set_mine(DeviceKey::from_seed([9; 32]));
        let host = k.mine_now().unwrap().public();
        (k, host)
    }

    fn phone() -> DeviceKey {
        DeviceKey::from_seed([7; 32])
    }

    #[test]
    fn a_good_proof_from_a_paired_key_names_the_device() {
        let (k, host) = keys();
        k.set_paired(vec![Paired {
            key: phone().public(),
            name: "pixel".into(),
        }]);
        let h = phone().sign(&host, NOW, "a1", "GET", "/api/v1/snapshot");
        assert_eq!(
            k.judge(Some(&h), "GET", "/api/v1/snapshot", NOW),
            Proof::Paired("pixel".into())
        );
    }

    #[test]
    fn a_good_proof_from_another_key_is_only_unpaired() {
        let (k, host) = keys();
        let h = phone().sign(&host, NOW, "a1", "GET", "/");
        assert_eq!(k.judge(Some(&h), "GET", "/", NOW), Proof::Unpaired);
    }

    #[test]
    fn a_proof_used_twice_is_refused_the_second_time() {
        let (k, host) = keys();
        let h = phone().sign(&host, NOW, "a1", "GET", "/");
        assert_eq!(k.judge(Some(&h), "GET", "/", NOW), Proof::Unpaired);
        assert_eq!(
            k.judge(Some(&h), "GET", "/", NOW + 1),
            Proof::Refused(REPLAYED)
        );
        // The same nonce from another device is its own proof.
        let other = DeviceKey::from_seed([3; 32]).sign(&host, NOW, "a1", "GET", "/");
        assert_eq!(k.judge(Some(&other), "GET", "/", NOW), Proof::Unpaired);
    }

    #[test]
    fn a_bad_proof_is_refused_and_says_why() {
        let (k, host) = keys();
        let wrong_host = phone().sign(&phone().public(), NOW, "a1", "GET", "/");
        assert_eq!(
            k.judge(Some(&wrong_host), "GET", "/", NOW),
            Proof::Refused(FORGED)
        );
        let old = phone().sign(&host, NOW - proof::FRESH_FOR - 60, "a2", "GET", "/");
        assert_eq!(k.judge(Some(&old), "GET", "/", NOW), Proof::Refused(STALE));
        assert_eq!(
            k.judge(Some("v9 x"), "GET", "/", NOW),
            Proof::Refused(UNREADABLE)
        );
    }

    #[test]
    fn without_a_proof_or_a_key_the_address_decides() {
        let (k, host) = keys();
        assert_eq!(k.judge(None, "GET", "/", NOW), Proof::None);
        let keyless = Keys::new();
        let h = phone().sign(&host, NOW, "a1", "GET", "/");
        assert_eq!(keyless.judge(Some(&h), "GET", "/", NOW), Proof::None);
    }

    #[test]
    fn the_key_is_made_once_and_kept_with_the_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let secrets = Secrets::file_only(dir.path().join("secrets.json"));
        let first = Keys::new().mine(&secrets).unwrap().public();
        let again = Keys::new().mine(&secrets).unwrap().public();
        assert_eq!(first, again);
    }
}
