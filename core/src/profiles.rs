//! Profiles (#562): several people on one tailnet each keep their own progress on one
//! computer. A profile is the Tailscale user a device belongs to, so nobody signs in: the
//! host reads who owns the asking device off `tailscale status` and answers as their
//! profile. History, resume points and tracker accounts belong to a profile; the library
//! and the metadata are everyone's.
//!
//! Profile 1 is the person the computer belongs to, the owner. This computer's own player,
//! the shell and every call that names no profile are the owner's.

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::CoreError;
pub use crate::foundation::trust::OWNER;
use crate::foundation::trust::{Asker, TailnetUser};

/// The profile to answer `asker` as, made on first sight for a person the host has not
/// seen before, with their Tailscale name kept fresh. `me` is this computer's own
/// Tailscale user when the tailnet says, which names the owner's profile. None for an
/// asker the host cannot place.
pub fn resolve(
    conn: &Connection,
    asker: &Asker,
    me: Option<&TailnetUser>,
    now: i64,
) -> Result<Option<u64>, CoreError> {
    match asker {
        Asker::Owner => {
            if let Some(me) = me {
                // The owner's Tailscale user, once known, so a device of theirs that
                // asks later is placed by it. A profile that already holds that user
                // keeps it.
                conn.execute(
                    "UPDATE profiles SET tailnet_user = ?2, login = ?3, name = ?4
                     WHERE id = ?1 AND NOT EXISTS (
                         SELECT 1 FROM profiles WHERE tailnet_user = ?2 AND id <> ?1)",
                    params![OWNER as i64, me.id, me.login, me.name],
                )?;
            }
            Ok(Some(OWNER))
        }
        Asker::Person(user) => {
            let found: Option<i64> = conn
                .query_row(
                    "SELECT id FROM profiles WHERE tailnet_user = ?1",
                    params![user.id],
                    |r| r.get(0),
                )
                .optional()?;
            let id = match found {
                Some(id) => {
                    conn.execute(
                        "UPDATE profiles SET login = ?2, name = ?3 WHERE id = ?1
                           AND (login IS NOT ?2 OR name IS NOT ?3)",
                        params![id, user.login, user.name],
                    )?;
                    id
                }
                None => {
                    conn.execute(
                        "INSERT INTO profiles (tailnet_user, login, name, created_at)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![user.id, user.login, user.name, now],
                    )?;
                    conn.last_insert_rowid()
                }
            };
            Ok(Some(id as u64))
        }
        Asker::Unknown => Ok(None),
    }
}

/// How a profile is named to the device it answers (#633), as last seen: its Tailscale
/// name and login, and whether it is the owner's. None while the host has never learned
/// a name, as for the owner of a tagged machine.
pub fn named(conn: &Connection, id: u64) -> Result<Option<anibeam_host_api::Profile>, CoreError> {
    let row: Option<(Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT name, login FROM profiles WHERE id = ?1",
            params![id as i64],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(row.and_then(|(name, login)| {
        let user = TailnetUser {
            id: String::new(),
            login: login.unwrap_or_default(),
            name: name.unwrap_or_default(),
        };
        shown(&user, id)
    }))
}

/// Profile `id` named after `user`, by [`TailnetUser::shown`]. None when Tailscale gives
/// them neither a name nor a login.
pub fn shown(user: &TailnetUser, id: u64) -> Option<anibeam_host_api::Profile> {
    Some(anibeam_host_api::Profile {
        name: user.shown()?.to_string(),
        login: user.login.clone(),
        owner: id == OWNER,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: &str, name: &str) -> TailnetUser {
        TailnetUser {
            id: id.to_string(),
            login: format!("{name}@example.com"),
            name: name.to_string(),
        }
    }

    fn open() -> (tempfile::TempDir, std::sync::Arc<crate::store::Store>) {
        let dir = tempfile::tempdir().unwrap();
        let store = crate::store::Store::open(&dir.path().join("anibeam.db")).unwrap();
        (dir, store)
    }

    #[test]
    fn the_owner_is_profile_one_and_learns_its_tailscale_user() {
        let (_dir, store) = open();
        let me = user("100", "Owner");
        let id = store
            .write(move |c| resolve(c, &Asker::Owner, Some(&me), 5))
            .unwrap();
        assert_eq!(id, Some(OWNER));
        let row: (String, String) = store
            .read(|c| {
                Ok(c.query_row(
                    "SELECT tailnet_user, name FROM profiles WHERE id = 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?)
            })
            .unwrap();
        assert_eq!(row, ("100".to_string(), "Owner".to_string()));
    }

    #[test]
    fn a_new_person_gets_a_profile_once_and_keeps_it() {
        let (_dir, store) = open();
        let friend = Asker::Person(user("200", "Friend"));
        let renamed = Asker::Person(user("200", "Friend B"));
        let first = store.write(move |c| resolve(c, &friend, None, 5)).unwrap();
        let again = store.write(move |c| resolve(c, &renamed, None, 6)).unwrap();
        assert_eq!(first, Some(2));
        assert_eq!(again, first, "the same Tailscale user is the same profile");
        let (count, name): (i64, String) = store
            .read(|c| {
                Ok(c.query_row(
                    "SELECT count(*), max(name) FROM profiles WHERE id <> 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?)
            })
            .unwrap();
        assert_eq!((count, name.as_str()), (1, "Friend B"));
    }

    #[test]
    fn a_profile_is_named_by_tailscale_and_the_owner_says_so() {
        let (_dir, store) = open();
        assert_eq!(
            store.read(|c| named(c, OWNER)).unwrap(),
            None,
            "no name learned yet"
        );
        let me = user("100", "Owner");
        let friend = Asker::Person(TailnetUser {
            name: String::new(),
            ..user("200", "friend")
        });
        let id = store
            .write(move |c| {
                resolve(c, &Asker::Owner, Some(&me), 5)?;
                resolve(c, &friend, None, 5)
            })
            .unwrap()
            .unwrap();
        let owner = store.read(|c| named(c, OWNER)).unwrap().unwrap();
        assert_eq!((owner.name.as_str(), owner.owner), ("Owner", true));
        let guest = store.read(move |c| named(c, id)).unwrap().unwrap();
        assert_eq!(
            (guest.name.as_str(), guest.owner),
            ("friend@example.com", false),
            "a person with no name is named by their login"
        );
        assert_eq!(store.read(|c| named(c, 99)).unwrap(), None);
    }

    #[test]
    fn someone_the_host_cannot_place_has_no_profile() {
        let (_dir, store) = open();
        assert_eq!(
            store
                .write(|c| resolve(c, &Asker::Unknown, None, 5))
                .unwrap(),
            None
        );
    }
}
