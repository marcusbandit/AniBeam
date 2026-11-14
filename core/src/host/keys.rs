//! This Anibeam's own key, the keys it is paired with, and the proofs it has
//! already seen (#555). The scheme itself is `anibeam_host_api::proof`.
//!
//! The key is made once and kept in its own owner-only file beside the
//! database, not in the keyring: a keyring still locked at start would read
//! as no key, and a new key would undo every pairing. The paired keys are
//! kept in the store; pairing (#557) is what adds them. A request may carry a
//! proof: one that does not check out is refused whatever its address, one
//! from a paired key is known to be that device, and one from any other key
//! changes nothing, so only a paired key's nonces are kept.

use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anibeam_host_api::proof::{self, DeviceKey, ProofError};
use serde::{Deserialize, Serialize};

use crate::contract::CoreError;
use crate::core::Core;
use crate::store::settings;
use crate::sync::recover;

/// More fresh proofs than this at once and new ones are refused, so a
/// paired device signing as fast as it can cannot grow the list without end.
const SEEN_MAX: usize = 100_000;
/// After the key could not be read or made, how long before trying again,
/// so a broken file is not read on every request.
const RETRY_AFTER: Duration = Duration::from_secs(60);

/// One Anibeam this one is paired with.
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
    /// Held while the key is read or made, so two first requests make one.
    mine: Mutex<Mine>,
    paired: Mutex<Vec<Paired>>,
    seen: Mutex<Seen>,
}

#[derive(Default)]
struct Mine {
    key: Option<Arc<DeviceKey>>,
    tried: Option<Instant>,
}

/// The paired keys' nonces still fresh, oldest first by when this machine
/// saw them.
#[derive(Default)]
struct Seen {
    ids: HashSet<(String, String)>,
    order: VecDeque<(i64, (String, String))>,
}

impl Default for Keys {
    fn default() -> Self {
        Keys::new()
    }
}

impl Keys {
    pub fn new() -> Keys {
        Keys {
            mine: Mutex::new(Mine::default()),
            paired: Mutex::new(Vec::new()),
            seen: Mutex::new(Seen::default()),
        }
    }

    /// This machine's key, kept at `path` and made there on first use. None
    /// when it can be neither read nor made, and then not tried again for
    /// `RETRY_AFTER`. Reads the file the first time.
    pub fn mine(&self, path: &Path) -> Option<Arc<DeviceKey>> {
        let mut mine = recover(&self.mine);
        if let Some(key) = &mine.key {
            return Some(key.clone());
        }
        if mine.tried.is_some_and(|at| at.elapsed() < RETRY_AFTER) {
            return None;
        }
        mine.tried = Some(Instant::now());
        mine.key = load_or_make(path).map(Arc::new);
        mine.key.clone()
    }

    /// This machine's key if it was already read, without touching the file.
    pub fn mine_now(&self) -> Option<Arc<DeviceKey>> {
        recover(&self.mine).key.clone()
    }

    /// Uses `key` as this machine's key, for a test.
    pub fn set_mine(&self, key: DeviceKey) {
        recover(&self.mine).key = Some(Arc::new(key));
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
        let paired = recover(&self.paired)
            .iter()
            .find(|p| p.key == proven.key)
            .map(|p| p.name.clone());
        // Another key's proof opens nothing, so replaying it gains nothing
        // either, and keeping its nonce would let anyone fill the list.
        let Some(name) = paired else {
            return Proof::Unpaired;
        };
        let mut seen = recover(&self.seen);
        // A proof is fresh for `FRESH_FOR` either side of its time, so one
        // seen here more than twice that long ago can no longer come again.
        let horizon = now.saturating_sub(2 * proof::FRESH_FOR);
        while seen.order.front().is_some_and(|(at, _)| *at < horizon) {
            if let Some((_, id)) = seen.order.pop_front() {
                seen.ids.remove(&id);
            }
        }
        let id = (proven.key, proven.nonce);
        if seen.ids.contains(&id) {
            return Proof::Refused(REPLAYED);
        }
        if seen.ids.len() >= SEEN_MAX {
            return Proof::Refused(BUSY);
        }
        seen.ids.insert(id.clone());
        seen.order.push_back((now, id));
        Proof::Paired(name)
    }
}

const UNREADABLE: &str = "That request's proof of its device could not be read.";
const FORGED: &str = "That request's proof of its device did not check out.";
const STALE: &str =
    "That request's proof of its device is too old. Check the clock on that device.";
const REPLAYED: &str = "That request's proof of its device was already used.";
const BUSY: &str = "Too many requests at once. Try again in a moment.";

fn load_or_make(path: &Path) -> Option<DeviceKey> {
    match std::fs::read_to_string(path) {
        Ok(text) => match proof::unhex::<32>(text.trim()) {
            Some(seed) => Some(DeviceKey::from_seed(seed)),
            // A key that does not read is left for a person to look at, not
            // replaced: another key would undo every pairing made with it.
            None => {
                tracing::warn!(
                    "host: the device key at {} does not read; requests go unsigned",
                    path.display()
                );
                None
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let key = DeviceKey::from_seed(rand::random::<[u8; 32]>());
            match crate::atomic_file::write_private(path, proof::hex(&key.seed())) {
                Ok(()) => Some(key),
                Err(e) => {
                    tracing::warn!("host: the device key could not be stored: {e}");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!("host: the device key could not be read: {e}");
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
    let Some(bytes) = proof::unhex::<32>(key) else {
        return Err(CoreError::Invalid {
            field: "key".into(),
            message: crate::i18n::tr("core-host-not-a-key", &[("key", format!("{key:?}").into())]),
        });
    };
    let entry = Paired {
        key: proof::hex(&bytes),
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

    fn pair_phone(k: &Keys) {
        k.set_paired(vec![Paired {
            key: phone().public(),
            name: "pixel".into(),
        }]);
    }

    #[test]
    fn a_good_proof_from_another_key_is_only_unpaired_and_never_kept() {
        let (k, host) = keys();
        let h = phone().sign(&host, NOW, "a1", "GET", "/");
        assert_eq!(k.judge(Some(&h), "GET", "/", NOW), Proof::Unpaired);
        // Its replay opens nothing either, and nothing of it was kept.
        assert_eq!(k.judge(Some(&h), "GET", "/", NOW), Proof::Unpaired);
        assert!(recover(&k.seen).ids.is_empty());
    }

    #[test]
    fn a_paired_proof_used_twice_is_refused_the_second_time() {
        let (k, host) = keys();
        pair_phone(&k);
        let h = phone().sign(&host, NOW, "a1", "GET", "/");
        assert_eq!(
            k.judge(Some(&h), "GET", "/", NOW),
            Proof::Paired("pixel".into())
        );
        assert_eq!(
            k.judge(Some(&h), "GET", "/", NOW + 1),
            Proof::Refused(REPLAYED)
        );
    }

    #[test]
    fn a_nonce_is_forgotten_once_its_proof_could_not_come_again() {
        let (k, host) = keys();
        pair_phone(&k);
        let first = phone().sign(&host, NOW, "a1", "GET", "/");
        assert!(matches!(
            k.judge(Some(&first), "GET", "/", NOW),
            Proof::Paired(_)
        ));
        let later = NOW + 2 * proof::FRESH_FOR + 1;
        let next = phone().sign(&host, later, "a2", "GET", "/");
        assert!(matches!(
            k.judge(Some(&next), "GET", "/", later),
            Proof::Paired(_)
        ));
        assert_eq!(recover(&k.seen).ids.len(), 1, "the first nonce went");
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
    fn the_key_is_made_once_and_kept_in_its_own_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("device-key");
        let first = Keys::new().mine(&path).unwrap().public();
        let again = Keys::new().mine(&path).unwrap().public();
        assert_eq!(first, again);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o077, 0, "owner only, {mode:o}");
        }
    }

    #[test]
    fn a_key_that_does_not_read_is_left_alone_and_not_read_on_every_request() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("device-key");
        std::fs::write(&path, "not a key").unwrap();
        let k = Keys::new();
        assert!(k.mine(&path).is_none());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "not a key");
        // Fixed on disk, it still waits out the pause before the next try.
        std::fs::write(&path, proof::hex(&[5; 32])).unwrap();
        assert!(k.mine(&path).is_none());
        assert!(Keys::new().mine(&path).is_some());
    }
}
