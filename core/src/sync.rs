//! One helper, used everywhere the core takes a std lock.

use std::sync::{Mutex, MutexGuard};

/// A lock's guard, poisoned or not.
///
/// Nothing behind a lock in this core is worth poisoning the process for:
/// every one of them holds plain data, a set of ids, a map of handles, a
/// queue of paths, and a panic elsewhere leaves all of that perfectly
/// usable. Propagating the poison instead would turn one panicking job
/// into a core that answers nothing.
pub(crate) fn recover<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn a_poisoned_lock_is_still_usable() {
        let lock = Arc::new(Mutex::new(vec![1u8]));
        let panicking = Arc::clone(&lock);
        let _ = std::thread::spawn(move || {
            let _held = panicking.lock().unwrap();
            panic!("boom, with the lock held");
        })
        .join();
        assert!(lock.lock().is_err(), "the lock really is poisoned");
        assert_eq!(*recover(&lock), vec![1u8]);
    }
}
