//! Where a tracker's tokens live: the desktop keyring when the machine has
//! one, a 0600 JSON file next to the database when it does not.
//!
//! Two rules shape this file. The Secret Service is reached over D-Bus, so
//! asking whether it exists costs a round trip and can put a prompt on the
//! screen; the core must not pay that during `open`, and must never pay it
//! twice. And every keyring call is synchronous zbus underneath, so it
//! belongs on a plain thread: a call runs it on the shell's calling thread,
//! a job runs it through `spawn_blocking`, and nothing here is ever awaited
//! inside a tokio task.
//!
//! Secrets never enter the database. What the database holds is the
//! `StoreKind` a write landed in, so the read after it goes straight to the
//! right store.

use std::any::Any;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};

use keyring_core::api::{CredentialApi, CredentialStoreApi};
use keyring_core::{Credential, CredentialStore, Entry, Error as KeyringError};

use crate::contract::CoreError;

/// Every entry the core writes is under this one service, so the key is
/// the whole address of a secret: `anilist.access_token`,
/// `mal.refresh_token`, `anilist.client_secret`. The Secret Service item's
/// label reads `keyring:<key>@anibeam` in Seahorse.
const SERVICE: &str = "anibeam";

/// The one line the core says when the machine has no keyring. It is said
/// once, by whichever caller finished the probe, and never again.
pub(crate) const KEYRING_UNAVAILABLE: &str = "secrets: keyring unavailable, using secrets.json";

/// Which of the two stores a secret is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoreKind {
    Keyring,
    File,
}

impl StoreKind {
    /// The value every store column holds.
    pub fn as_str(self) -> &'static str {
        match self {
            StoreKind::Keyring => "keyring",
            StoreKind::File => "file",
        }
    }

    pub fn from_column(s: &str) -> Option<StoreKind> {
        match s {
            "keyring" => Some(StoreKind::Keyring),
            "file" => Some(StoreKind::File),
            _ => None,
        }
    }
}

/// The core's one way to a secret. Every method is synchronous and every
/// method may talk to the keyring, so the caller decides which thread it
/// runs on.
pub struct Secrets {
    /// Always built, never probed: creating it opens nothing and touches
    /// no disk, so `Core::open` can do it.
    file: Arc<FileStore>,
    /// The Secret Service store, or `None` when the machine has none.
    /// Empty until the first use: the probe is a D-Bus round trip, so it
    /// waits for someone who actually wants a secret, or for `start`'s
    /// warm-up, whichever comes first. `OnceLock` makes the loser of that
    /// race wait rather than probe a second time.
    primary: OnceLock<Option<Arc<CredentialStore>>>,
    /// Claimed by the caller that should say the keyring is missing, so
    /// the line is written once however many callers raced the probe.
    announced: AtomicBool,
    /// Every key the core knows exists: the file store's keys at init,
    /// plus every key a `set` wrote and minus every key a `delete` took
    /// away, plus whatever a `has` found in the keyring. It is what keeps
    /// `GetTrackers` off the keyring once a key has been looked at once.
    known: Mutex<HashSet<String>>,
    /// Every key the keyring has already been asked about, whatever it
    /// said. Without this a key that is not there costs a D-Bus round trip
    /// on the caller's thread every single time it is asked about, and
    /// `GetTrackers` asks about every key of every tracker; a disconnected
    /// tracker is exactly the case that would pay it forever.
    probed: Mutex<HashSet<String>>,
}

impl Secrets {
    /// The facade the core runs on. Nothing here touches the network, the
    /// D-Bus session or the disk beyond reading `secrets.json` if it is
    /// already there.
    pub fn init(path: PathBuf) -> Arc<Secrets> {
        let file = FileStore::new(path);
        let known = file.keys();
        Arc::new(Secrets {
            file,
            primary: OnceLock::new(),
            announced: AtomicBool::new(false),
            known: Mutex::new(known),
            probed: Mutex::new(HashSet::new()),
        })
    }

    /// The file store and nothing else, for the tests and for a CLI run
    /// under `--root`: the store choice is made here rather than probed,
    /// so no test ever reaches the machine's real keyring. There is
    /// nothing to announce, since this was asked for rather than fallen
    /// back to.
    pub fn file_only(path: PathBuf) -> Arc<Secrets> {
        let secrets = Secrets::init(path);
        let _ = secrets.primary.set(None);
        secrets.announced.store(true, Ordering::SeqCst);
        secrets
    }

    /// Makes the store choice if it has not been made, and returns true to
    /// the one caller that should write the "no keyring" line. Blocking:
    /// the first caller pays the D-Bus probe and the rest wait on it.
    pub fn warm(&self) -> bool {
        let unavailable = self.primary().is_none();
        unavailable && !self.announced.swap(true, Ordering::SeqCst)
    }

    /// `None` until the store choice has been made, then whether the
    /// machine turned out to have no keyring. Reading it never starts the
    /// probe, so a caller that only wants to report the state does not
    /// cause one.
    pub fn keyring_unavailable(&self) -> Option<bool> {
        self.primary.get().map(Option::is_none)
    }

    /// Whether the core holds this secret, answered off memory whenever it
    /// can be. The file store's keys are in the known set from `init` and
    /// from every `set`, so a file-stored key never costs a read; anything
    /// else costs one keyring look, exactly once, whatever the keyring
    /// says. A key written into the keyring by something other than this
    /// core after that look stays invisible until the core is opened
    /// again, which is the trade for never blocking a call on D-Bus twice.
    pub fn has(&self, key: &str) -> bool {
        if self.is_known(key) {
            return true;
        }
        if self.was_probed(key) {
            return false;
        }
        let Some(store) = self.primary() else { return false };
        let found = matches!(read_in(store.as_ref(), key), Ok(Some(_)));
        self.probed().insert(key.to_string());
        if found {
            self.known().insert(key.to_string());
        }
        found
    }

    /// Reads a secret, hinted store first. A miss in one store is tried in
    /// the other, so a token written before the keyring appeared is still
    /// found. A locked or absent keyring is not this read's failure: it
    /// falls through and the file store's answer stands.
    pub fn get(&self, key: &str, hint: Option<StoreKind>) -> Result<Option<(String, StoreKind)>, CoreError> {
        for kind in order(hint) {
            let Some(store) = self.store(kind) else { continue };
            match read_in(store, key) {
                Ok(Some(value)) => return Ok(Some((value, kind))),
                Ok(None) => {}
                Err(e @ (KeyringError::PlatformFailure(_) | KeyringError::NoStorageAccess(_))) if kind == StoreKind::Keyring => {
                    tracing::debug!("keyring read of {key} failed, falling through to the file store: {e}");
                }
                Err(e) => return Err(keyring_error(key, e)),
            }
        }
        Ok(None)
    }

    /// Writes a secret and says where it went. The keyring takes it when
    /// the machine has one and accepts the write; anything else it says
    /// sends the write to the file store, which is the whole point of the
    /// fallback.
    ///
    /// Whichever store loses the write has the key taken out of it. A read
    /// tries the keyring first, so a keyring copy left behind after a
    /// locked collection sent the write to the file would be served as the
    /// current token; and a file copy left behind after the machine gained
    /// a keyring would sit in plaintext for good. One value, one place.
    pub fn set(&self, key: &str, value: &str) -> Result<StoreKind, CoreError> {
        if let Some(store) = self.primary() {
            match write_in(store.as_ref(), key, value) {
                Ok(()) => {
                    clear_loser(self.file.as_ref(), key);
                    self.known().insert(key.to_string());
                    return Ok(StoreKind::Keyring);
                }
                Err(e) => tracing::debug!("keyring write of {key} failed, using the file store: {e}"),
            }
        }
        write_in(self.file.as_ref(), key, value).map_err(|e| keyring_error(key, e))?;
        if let Some(store) = self.primary() {
            clear_loser(store.as_ref(), key);
        }
        self.known().insert(key.to_string());
        Ok(StoreKind::File)
    }

    /// Takes a secret out of both stores, so a disconnect leaves nothing
    /// behind whichever store the write of the day landed in. Nothing
    /// stored is not a failure. Both stores are tried even when the first
    /// fails, and the first failure is what the caller hears about.
    pub fn delete(&self, key: &str) -> Result<(), CoreError> {
        let mut failure: Option<KeyringError> = None;
        if let Some(store) = self.primary()
            && let Err(e) = delete_in(store.as_ref(), key)
        {
            failure = Some(e);
        }
        if let Err(e) = delete_in(self.file.as_ref(), key)
            && failure.is_none()
        {
            failure = Some(e);
        }
        self.known().remove(key);
        match failure {
            Some(e) => Err(keyring_error(key, e)),
            None => Ok(()),
        }
    }

    /// The keyring store, probing for it on the first call. Every caller
    /// after the first finds the answer already there, and callers racing
    /// the first wait on it rather than opening a second session bus.
    fn primary(&self) -> Option<&Arc<CredentialStore>> {
        self.primary
            .get_or_init(|| match zbus_secret_service_keyring_store::Store::new() {
                Ok(store) => {
                    let store: Arc<CredentialStore> = store;
                    Some(store)
                }
                // No session bus, or a bus with no Secret Service on it.
                // Either way there is no keyring here; the file store is
                // the answer and the core says so once.
                Err(e) => {
                    tracing::debug!("no Secret Service: {e}");
                    None
                }
            })
            .as_ref()
    }

    /// The store of a kind, or `None` when this machine has no keyring.
    fn store(&self, kind: StoreKind) -> Option<&dyn CredentialStoreApi> {
        match kind {
            StoreKind::Keyring => self.primary().map(|store| store.as_ref() as &dyn CredentialStoreApi),
            StoreKind::File => Some(self.file.as_ref()),
        }
    }

    fn known(&self) -> MutexGuard<'_, HashSet<String>> {
        self.known.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn probed(&self) -> MutexGuard<'_, HashSet<String>> {
        self.probed.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The membership test on its own, so the guard is gone before any
    /// store call: a keyring read can block for as long as a prompt is on
    /// screen, and no other reader of `known` should wait on that.
    fn is_known(&self, key: &str) -> bool {
        self.known().contains(key)
    }

    /// The same, for the set of keys the keyring has already answered for.
    fn was_probed(&self, key: &str) -> bool {
        self.probed().contains(key)
    }
}

/// The stores to try, in order: the hinted one, then the other. With no
/// hint the keyring goes first, since that is where a write lands when
/// there is one.
fn order(hint: Option<StoreKind>) -> [StoreKind; 2] {
    match hint {
        Some(StoreKind::File) => [StoreKind::File, StoreKind::Keyring],
        _ => [StoreKind::Keyring, StoreKind::File],
    }
}

fn read_in(store: &dyn CredentialStoreApi, key: &str) -> keyring_core::Result<Option<String>> {
    match store.build(SERVICE, key, None)?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(e) => Err(e),
    }
}

fn write_in(store: &dyn CredentialStoreApi, key: &str, value: &str) -> keyring_core::Result<()> {
    store.build(SERVICE, key, None)?.set_password(value)
}

/// Nothing stored is what a delete wanted, so `NoEntry` is success.
fn delete_in(store: &dyn CredentialStoreApi, key: &str) -> keyring_core::Result<()> {
    match store.build(SERVICE, key, None)?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Takes a key out of the store that did not win the write. Nothing there
/// is the ordinary case, and a store that cannot be reached is one nothing
/// could have cleaned anyway, so neither outcome is the write's failure.
fn clear_loser(store: &dyn CredentialStoreApi, key: &str) {
    if let Err(e) = delete_in(store, key) {
        tracing::debug!("could not clear the old {key} out of the store that lost the write: {e}");
    }
}

/// The key never carries the secret, so it is safe in a message.
fn keyring_error(key: &str, e: KeyringError) -> CoreError {
    CoreError::Keyring { message: format!("{key}: {e}") }
}

fn platform(e: std::io::Error) -> KeyringError {
    KeyringError::PlatformFailure(Box::new(e))
}

/// The fallback store: one JSON object at `secrets.json`, keyed
/// `"<service>/<user>"`, created 0600 and rewritten whole through a `.tmp`
/// sibling and a rename. Unencrypted by decision: a key to decrypt it
/// would have to live in a file beside it.
///
/// It is a keyring-core store rather than a plain struct so the facade has
/// one code path for both stores: `build`, then `get_password`.
pub(crate) struct FileStore {
    path: PathBuf,
    /// Held across the read, the change and the rename, so two writes
    /// never lose one another's key.
    writes: Mutex<()>,
    /// A credential holds its store, and the store only becomes an `Arc`
    /// after it is built, so the way back to itself is filled in by `new`.
    me: OnceLock<Weak<FileStore>>,
}

impl FileStore {
    fn new(path: PathBuf) -> Arc<FileStore> {
        let store = Arc::new(FileStore { path, writes: Mutex::new(()), me: OnceLock::new() });
        let _ = store.me.set(Arc::downgrade(&store));
        store
    }

    /// Every key this file holds under the core's service, for the known
    /// set. A file that is missing, empty or unreadable is simply no keys.
    fn keys(&self) -> HashSet<String> {
        let prefix = format!("{SERVICE}/");
        self.read_all()
            .unwrap_or_default()
            .into_keys()
            .filter_map(|k| k.strip_prefix(prefix.as_str()).map(str::to_string))
            .collect()
    }

    /// The whole file. A missing or empty file is an empty object, so the
    /// first write is the same code path as every one after it.
    fn read_all(&self) -> keyring_core::Result<BTreeMap<String, String>> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) if text.trim().is_empty() => Ok(BTreeMap::new()),
            Ok(text) => serde_json::from_str(&text).map_err(|e| KeyringError::BadStoreFormat(format!("{}: {e}", self.path.display()))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(e) => Err(platform(e)),
        }
    }

    /// The whole file again, atomically: a reader either sees the file as
    /// it was or as it is, never half of a write.
    fn write_all(&self, map: &BTreeMap<String, String>) -> keyring_core::Result<()> {
        let text = serde_json::to_string_pretty(map).map_err(|e| KeyringError::PlatformFailure(Box::new(e)))?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(platform)?;
        }
        let tmp = self.tmp_path();
        let mut file = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&tmp).map_err(platform)?;
        file.write_all(text.as_bytes()).map_err(platform)?;
        file.sync_all().map_err(platform)?;
        drop(file);
        // `mode` above only applies when the open created the file, and a
        // crashed write can leave one behind with a wider mode, so say it
        // again before the rename carries it over the real file.
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600)).map_err(platform)?;
        std::fs::rename(&tmp, &self.path).map_err(platform)?;
        Ok(())
    }

    fn tmp_path(&self) -> PathBuf {
        let mut name = self.path.file_name().map(OsString::from).unwrap_or_else(|| OsString::from("secrets.json"));
        name.push(".tmp");
        self.path.with_file_name(name)
    }

    fn arc(&self) -> Option<Arc<FileStore>> {
        self.me.get().and_then(Weak::upgrade)
    }
}

impl CredentialStoreApi for FileStore {
    fn vendor(&self) -> String {
        "AniBeam file store, secrets.json".to_string()
    }

    fn id(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    /// Modifiers are ignored: this store has none, and refusing them would
    /// only break a caller that passed one meant for another store.
    fn build(&self, service: &str, user: &str, _modifiers: Option<&HashMap<&str, &str>>) -> keyring_core::Result<Entry> {
        let store = self.arc().ok_or_else(|| KeyringError::PlatformFailure(Box::from("the file store is gone")))?;
        Ok(Entry::new_with_credential(Arc::new(FileCredential { store, service: service.to_string(), user: user.to_string() })))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// One entry in the file: the store it belongs to and the pair that names
/// it.
struct FileCredential {
    store: Arc<FileStore>,
    service: String,
    user: String,
}

impl FileCredential {
    fn key(&self) -> String {
        format!("{}/{}", self.service, self.user)
    }
}

impl CredentialApi for FileCredential {
    fn set_secret(&self, secret: &[u8]) -> keyring_core::Result<()> {
        let value = std::str::from_utf8(secret).map_err(|e| KeyringError::Invalid("secret".to_string(), e.to_string()))?;
        let _guard = self.store.writes.lock().unwrap_or_else(|e| e.into_inner());
        let mut map = self.store.read_all()?;
        map.insert(self.key(), value.to_string());
        self.store.write_all(&map)
    }

    fn get_secret(&self) -> keyring_core::Result<Vec<u8>> {
        match self.store.read_all()?.get(&self.key()) {
            Some(value) => Ok(value.as_bytes().to_vec()),
            None => Err(KeyringError::NoEntry),
        }
    }

    fn delete_credential(&self) -> keyring_core::Result<()> {
        let _guard = self.store.writes.lock().unwrap_or_else(|e| e.into_inner());
        let mut map = self.store.read_all()?;
        if map.remove(&self.key()).is_none() {
            return Err(KeyringError::NoEntry);
        }
        self.store.write_all(&map)
    }

    /// This credential is already the wrapper for its entry, so the API's
    /// `None` is the right answer: the caller keeps what it has.
    fn get_credential(&self) -> keyring_core::Result<Option<Arc<Credential>>> {
        Ok(None)
    }

    fn get_specifiers(&self) -> Option<(String, String)> {
        Some((self.service.clone(), self.user.clone()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::AtomicUsize;

    /// A facade with a stand-in where the keyring goes, so the two-store
    /// paths can be driven without a Secret Service. The store choice is
    /// made here rather than probed for, exactly as `file_only` does it.
    fn with_stores(primary: Arc<CredentialStore>, path: PathBuf) -> Arc<Secrets> {
        let secrets = Secrets::init(path);
        let _ = secrets.primary.set(Some(primary));
        secrets.announced.store(true, Ordering::SeqCst);
        secrets
    }

    /// A second file store standing in for the keyring, counting the
    /// entries built off it so a test can see whether a call was made.
    struct CountingStore {
        inner: Arc<FileStore>,
        builds: Arc<AtomicUsize>,
    }

    impl CredentialStoreApi for CountingStore {
        fn vendor(&self) -> String {
            "counting stand-in".to_string()
        }

        fn id(&self) -> String {
            self.inner.id()
        }

        fn build(&self, service: &str, user: &str, modifiers: Option<&HashMap<&str, &str>>) -> keyring_core::Result<Entry> {
            self.builds.fetch_add(1, Ordering::SeqCst);
            self.inner.build(service, user, modifiers)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    /// A stand-in shaped like a locked collection: it reads and deletes,
    /// and refuses every write with the error the Secret Service store
    /// raises for a collection it cannot open.
    struct LockedStore {
        inner: Arc<FileStore>,
    }

    impl CredentialStoreApi for LockedStore {
        fn vendor(&self) -> String {
            "locked stand-in".to_string()
        }

        fn id(&self) -> String {
            self.inner.id()
        }

        fn build(&self, service: &str, user: &str, modifiers: Option<&HashMap<&str, &str>>) -> keyring_core::Result<Entry> {
            Ok(Entry::new_with_credential(Arc::new(LockedCredential { inner: self.inner.build(service, user, modifiers)? })))
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    struct LockedCredential {
        inner: Entry,
    }

    impl CredentialApi for LockedCredential {
        fn set_secret(&self, _secret: &[u8]) -> keyring_core::Result<()> {
            Err(KeyringError::NoStorageAccess(Box::from("the collection is locked")))
        }

        fn get_secret(&self) -> keyring_core::Result<Vec<u8>> {
            self.inner.get_secret()
        }

        fn delete_credential(&self) -> keyring_core::Result<()> {
            self.inner.delete_credential()
        }

        fn get_credential(&self) -> keyring_core::Result<Option<Arc<Credential>>> {
            Ok(None)
        }

        fn get_specifiers(&self) -> Option<(String, String)> {
            self.inner.get_specifiers()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    /// What a store holds for a key, read straight off it rather than
    /// through the facade.
    fn stored(path: &std::path::Path, key: &str) -> Option<String> {
        read_in(FileStore::new(path.to_path_buf()).as_ref(), key).unwrap()
    }

    #[test]
    fn file_store_round_trips_and_is_private() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");
        let s = Secrets::file_only(path.clone());
        assert_eq!(s.get("anilist.access_token", None).unwrap(), None);
        assert_eq!(s.set("anilist.access_token", "tok").unwrap(), StoreKind::File);
        assert_eq!(s.get("anilist.access_token", Some(StoreKind::File)).unwrap(), Some(("tok".into(), StoreKind::File)));
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        s.set("anilist.access_token", "tok2").unwrap();
        assert_eq!(s.get("anilist.access_token", None).unwrap().unwrap().0, "tok2");
        s.delete("anilist.access_token").unwrap();
        assert_eq!(s.get("anilist.access_token", None).unwrap(), None);
        assert!(!dir.path().join("secrets.json.tmp").exists());
    }

    #[test]
    fn secrets_know_a_key_after_a_set_and_forget_it_after_a_delete() {
        let dir = tempfile::tempdir().unwrap();
        let s = Secrets::file_only(dir.path().join("secrets.json"));
        assert!(!s.has("mal.refresh_token"));
        s.set("mal.refresh_token", "rt").unwrap();
        assert!(s.has("mal.refresh_token"));
        assert!(!s.has("mal.access_token"));
        s.delete("mal.refresh_token").unwrap();
        assert!(!s.has("mal.refresh_token"));
    }

    #[test]
    fn known_keys_come_back_from_the_file_without_a_store_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");
        std::fs::write(&path, r#"{"anibeam/anilist.client_secret": "cs", "other/thing": "x"}"#).unwrap();
        let s = Secrets::file_only(path);
        assert!(s.has("anilist.client_secret"));
        assert!(!s.has("thing"));
    }

    #[test]
    fn an_empty_or_missing_file_reads_as_nothing_stored() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");
        let missing = Secrets::file_only(path.clone());
        assert_eq!(missing.get("anilist.access_token", None).unwrap(), None);
        std::fs::write(&path, "   \n").unwrap();
        let empty = Secrets::file_only(path);
        assert_eq!(empty.get("anilist.access_token", None).unwrap(), None);
        // Deleting what was never there is what the caller wanted.
        empty.delete("anilist.access_token").unwrap();
    }

    /// `init` makes no store choice, so nothing here reaches D-Bus:
    /// `keyring_unavailable` stays `None` until someone asks for a secret
    /// or `start` warms the probe. `file_only` made the choice itself, so
    /// it answers at once and has nothing to announce.
    #[test]
    fn the_store_choice_is_lazy_and_file_only_makes_it_up_front() {
        let dir = tempfile::tempdir().unwrap();
        let lazy = Secrets::init(dir.path().join("secrets.json"));
        assert_eq!(lazy.keyring_unavailable(), None);
        let file_only = Secrets::file_only(dir.path().join("other.json"));
        assert_eq!(file_only.keyring_unavailable(), Some(true));
        assert!(!file_only.warm());
        assert_eq!(file_only.keyring_unavailable(), Some(true));
    }

    /// The machine gained a keyring, so the write lands there and the old
    /// plaintext copy has to go: leaving it would keep a token in the file
    /// for good.
    #[test]
    fn a_write_the_keyring_takes_clears_the_file_copy() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("secrets.json");
        let keyring_path = dir.path().join("keyring.json");
        write_in(FileStore::new(file_path.clone()).as_ref(), "anilist.access_token", "old").unwrap();
        let keyring = Arc::new(CountingStore { inner: FileStore::new(keyring_path.clone()), builds: Arc::new(AtomicUsize::new(0)) });
        let s = with_stores(keyring, file_path.clone());

        assert_eq!(s.set("anilist.access_token", "new").unwrap(), StoreKind::Keyring);
        assert_eq!(stored(&keyring_path, "anilist.access_token"), Some("new".to_string()));
        assert_eq!(stored(&file_path, "anilist.access_token"), None);
        assert_eq!(s.get("anilist.access_token", None).unwrap(), Some(("new".to_string(), StoreKind::Keyring)));
    }

    /// The collection is locked, so the write falls through to the file.
    /// The keyring's older copy has to go with it: a read tries the
    /// keyring first and would otherwise serve the stale token as current.
    #[test]
    fn a_write_the_keyring_refuses_clears_the_keyring_copy() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("secrets.json");
        let keyring_path = dir.path().join("keyring.json");
        write_in(FileStore::new(keyring_path.clone()).as_ref(), "anilist.access_token", "old").unwrap();
        let keyring = Arc::new(LockedStore { inner: FileStore::new(keyring_path.clone()) });
        let s = with_stores(keyring, file_path.clone());

        assert_eq!(s.set("anilist.access_token", "new").unwrap(), StoreKind::File);
        assert_eq!(stored(&file_path, "anilist.access_token"), Some("new".to_string()));
        assert_eq!(stored(&keyring_path, "anilist.access_token"), None);
        assert_eq!(s.get("anilist.access_token", None).unwrap(), Some(("new".to_string(), StoreKind::File)));
    }

    /// A key that is not there is asked of the keyring once. Every `has`
    /// after that answers off memory, because `GetTrackers` asks about
    /// every key of every tracker and the round trip is on the shell's
    /// calling thread.
    #[test]
    fn a_key_the_keyring_does_not_have_is_asked_for_once() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("secrets.json");
        write_in(FileStore::new(file_path.clone()).as_ref(), "anilist.client_secret", "cs").unwrap();
        let builds = Arc::new(AtomicUsize::new(0));
        let keyring = Arc::new(CountingStore { inner: FileStore::new(dir.path().join("keyring.json")), builds: builds.clone() });
        let s = with_stores(keyring, file_path);

        // A key the file store had at init needs no look at all.
        assert!(s.has("anilist.client_secret"));
        assert_eq!(builds.load(Ordering::SeqCst), 0);

        assert!(!s.has("mal.client_secret"));
        assert_eq!(builds.load(Ordering::SeqCst), 1);
        assert!(!s.has("mal.client_secret"));
        assert!(!s.has("mal.client_secret"));
        assert_eq!(builds.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn store_kinds_round_trip_through_their_column() {
        for kind in [StoreKind::Keyring, StoreKind::File] {
            assert_eq!(StoreKind::from_column(kind.as_str()), Some(kind));
        }
        assert_eq!(StoreKind::from_column("gnome"), None);
    }
}
