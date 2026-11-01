//! Not interested, kept the same on this phone and the computer. The computer's list is the
//! one both follow: a mark made here goes to this phone's core at once and to the computer
//! when it answers, and waits in `not-interested.json` until it has. Each sync sends what
//! waits, then makes this phone's list match the computer's.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use anibeam_companion::address::HostAddress;
use anibeam_companion::client::{self, HostClient};
use anibeam_core::{Call, Reply};
use serde::{Deserialize, Serialize};

/// A mark not yet on the computer: the show, and whether it is marked or taken back.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Waiting {
    anilist_id: u64,
    on: bool,
}

static FILE: OnceLock<PathBuf> = OnceLock::new();
static DATA: OnceLock<PathBuf> = OnceLock::new();
/// One sync or one send at a time, so a mark is never sent twice or lost between them.
static LOCK: Mutex<()> = Mutex::new(());

pub fn start(data_dir: &std::path::Path) {
    let _ = FILE.set(data_dir.join("not-interested.json"));
    let _ = DATA.set(data_dir.to_path_buf());
}

/// The computer as saved in More, not as the address field reads while it is edited.
fn saved_address() -> Option<HostAddress> {
    let store = anibeam_companion::store::Store::open(DATA.get()?).ok()?;
    HostAddress::parse(&store.settings().address).ok()
}

fn waiting() -> Vec<Waiting> {
    FILE.get().and_then(|f| std::fs::read(f).ok()).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn keep(list: &[Waiting]) {
    let Some(file) = FILE.get() else { return };
    let written = if list.is_empty() {
        std::fs::remove_file(file).or_else(|e| if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
    } else {
        serde_json::to_vec(list).map_err(std::io::Error::other).and_then(|b| std::fs::write(file, b))
    };
    if let Err(e) = written {
        crate::local::log(&format!("not interested: could not keep the marks waiting: {e}"));
    }
}

/// Marks a show, or takes the mark back, here and then on the computer. Blocking; call it
/// off the UI thread. The lock is held from the local mark on, so a sync never sees the
/// mark here without it waiting to go out.
pub fn mark(anilist_id: u64, on: bool) {
    let Some(core) = crate::local::core() else { return };
    let _held = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if let Err(e) = core.call(Call::SetNotInterested { anilist_id, not_interested: on }) {
        crate::local::log(&format!("not interested: {e}"));
        return;
    }
    let address = saved_address();
    let mut list = waiting();
    // A later mark of the same show replaces the earlier one.
    list.retain(|w| w.anilist_id != anilist_id);
    list.push(Waiting { anilist_id, on });
    keep(&list);
    if let Some(address) = address {
        send(&HostClient::new(client::agent(), address), &mut list);
        keep(&list);
    }
}

/// Sends what waits, in order, and keeps whatever the computer did not take.
fn send(host: &HostClient, list: &mut Vec<Waiting>) {
    while let Some(w) = list.first() {
        let call = Call::SetNotInterested { anilist_id: w.anilist_id, not_interested: w.on };
        match crate::stream::call_on(host, "your computer", &call) {
            Ok(_) => {
                list.remove(0);
            }
            Err(e) => {
                crate::local::log(&format!("not interested: the computer did not take a mark yet: {e}"));
                return;
            }
        }
    }
}

/// After a sync with the computer: send what waits, then follow the computer's list. A
/// computer too old to answer leaves this phone's list as it is.
pub fn sync(address: &HostAddress) {
    let Some(core) = crate::local::core() else { return };
    let _held = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let host = HostClient::new(client::agent(), address.clone());
    let mut list = waiting();
    send(&host, &mut list);
    keep(&list);
    if !list.is_empty() {
        return;
    }
    let theirs = match crate::stream::call_on(&host, "your computer", &Call::ListNotInterested) {
        Ok(Reply::NotInterested { shows }) => shows.into_iter().map(|s| s.anilist_id).collect::<HashSet<u64>>(),
        Ok(_) => return,
        Err(e) => {
            crate::local::log(&format!("not interested: the computer's list did not come: {e}"));
            return;
        }
    };
    let ours: HashSet<u64> = match core.call(Call::ListNotInterested) {
        Ok(Reply::NotInterested { shows }) => shows.into_iter().map(|s| s.anilist_id).collect(),
        _ => return,
    };
    let mut changed = false;
    for (&id, on) in theirs.difference(&ours).map(|id| (id, true)).chain(ours.difference(&theirs).map(|id| (id, false))) {
        changed |= core.call(Call::SetNotInterested { anilist_id: id, not_interested: on }).is_ok();
    }
    if changed {
        crate::local::touch();
    }
}
