//! Every tracker row is the owner's (#863). A second profile's rows sit in
//! the same tables since migration 0021, and none of this team's reads,
//! updates or deletes may reach them.

use std::path::Path;

use rusqlite::{Connection, params};

use crate::{accounts, cache, move_up, watching, writes};
use anibeam_contract::*;

/// The owner and a friend, both with an AniList account and an entry for
/// the same show. The friend's entry is the older one and carries another
/// status, so a read that mixes the two says so.
fn two_profiles() -> Connection {
    let c = anibeam_foundation::store::testing::conn();
    c.execute_batch(
        "INSERT INTO profiles (id, created_at) VALUES (2, 0);
         INSERT INTO tracker_accounts (profile_id, tracker, user_id, username, connected_at, progress_fetched_at) VALUES
             (1, 'anilist', 1, 'owner', 1, 5), (2, 'anilist', 2, 'friend', 1, 5);
         INSERT INTO anilist_media (id, mal_id, media_type, title_romaji) VALUES
             (10, 110, 'ANIME', 'Shared show'), (20, 120, 'ANIME', 'Friend show');
         INSERT INTO tracker_entries (profile_id, tracker, media_id, status, progress, updated_at, fetched_at) VALUES
             (1, 'anilist', 10, 'watching', 3, 100, 0),
             (2, 'anilist', 10, 'completed', 12, 50, 0),
             (2, 'anilist', 20, 'watching', 1, 900, 0);
         INSERT INTO sources (id, path, added_at) VALUES (1, '/tv', 0);
         INSERT INTO series (id, source_id, kind, path, folder_name, added_at, anilist_id)
             VALUES (1, 1, 'show', '/tv/Shared show', 'Shared show', 0, 10);
         INSERT INTO local_entries (series_id, status, updated_at) VALUES (1, 'paused', 500);",
    )
    .unwrap();
    c
}

fn guest_rows(c: &Connection) -> Vec<(i64, Option<String>, i64)> {
    let mut stmt = c
        .prepare(
            "SELECT media_id, status, progress FROM tracker_entries
              WHERE profile_id = 2 ORDER BY media_id",
        )
        .unwrap();
    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn guest_untouched(c: &Connection) {
    assert_eq!(
        guest_rows(c),
        vec![
            (10, Some("completed".into()), 12),
            (20, Some("watching".into()), 1)
        ]
    );
}

#[test]
fn the_account_read_is_the_owners() {
    let c = two_profiles();
    let row = accounts::load_row(&c, Tracker::Anilist).unwrap().unwrap();
    assert_eq!(row.username.as_deref(), Some("owner"));
    assert_eq!(row.user_id, Some(1));
    // With no account of the owner's, the friend's is not one either.
    c.execute("DELETE FROM tracker_accounts WHERE profile_id = 1", [])
        .unwrap();
    assert!(accounts::load_row(&c, Tracker::Anilist).unwrap().is_none());
}

#[test]
fn the_watching_page_lists_only_the_owners_entries() {
    let c = two_profiles();
    let list = watching::list(&c, Path::new("/images")).unwrap();
    let ids: Vec<u64> = list.entries.iter().map(|e| e.anilist_id).collect();
    assert_eq!(ids, vec![10]);
}

#[test]
fn a_list_refresh_replaces_only_the_owners_rows() {
    let mut c = two_profiles();
    let tx = c.transaction().unwrap();
    let entry = cache::Entry {
        media_id: 10,
        progress: 4,
        status: Some(ListStatus::Watching),
        score: None,
        repeat: 0,
        updated_at: None,
        notes: None,
    };
    cache::replace(&tx, Tracker::Anilist, &[entry], 60).unwrap();
    tx.commit().unwrap();
    // The stamp the fetch brought none for is the owner's own, never the
    // friend's.
    let (progress, updated_at): (i64, i64) = c
        .query_row(
            "SELECT progress, updated_at FROM tracker_entries
              WHERE profile_id = 1 AND tracker = 'anilist' AND media_id = 10",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((progress, updated_at), (4, 100));
    guest_untouched(&c);
    let fetched: Vec<Option<i64>> = c
        .prepare("SELECT progress_fetched_at FROM tracker_accounts ORDER BY profile_id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(fetched, vec![Some(60), Some(5)]);
}

#[test]
fn a_patch_moves_only_the_owners_entry() {
    let c = two_profiles();
    cache::patch_status(&c, Tracker::Anilist, 10, ListStatus::Paused, Some(70), 70).unwrap();
    cache::patch_repeat(&c, Tracker::Anilist, 10, 2, 80).unwrap();
    let (status, repeat): (String, i64) = c
        .query_row(
            "SELECT status, repeat FROM tracker_entries
              WHERE profile_id = 1 AND tracker = 'anilist' AND media_id = ?1",
            params![10],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((status.as_str(), repeat), ("paused", 2));
    guest_untouched(&c);
    let guest_repeat: i64 = c
        .query_row(
            "SELECT repeat FROM tracker_entries WHERE profile_id = 2 AND media_id = 10",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(guest_repeat, 0);
}

#[test]
fn move_up_offers_each_series_once_against_the_owners_entry() {
    let c = two_profiles();
    let rows = move_up::preview(&c, Tracker::Anilist).unwrap();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].tracker_status, Some(ListStatus::Watching));
}

#[test]
fn the_mark_guard_reads_only_the_owners_progress() {
    let c = two_profiles();
    let target = writes::Targets {
        series: 1,
        anilist: Some(10),
        mal: None,
        total: Some(12),
        folder_name: "Shared show".into(),
    };
    let covered = |episode| writes::covered(&c, &target, &[Tracker::Anilist], episode).unwrap();
    // The owner is at 3 and the friend at 12: the owner's 3 is covered, and
    // the friend's 12 says nothing about the owner's 5.
    assert!(covered(3));
    assert!(!covered(5));
    // With no entry of the owner's, the friend's covers nothing either.
    c.execute(
        "DELETE FROM tracker_entries WHERE profile_id = 1 AND media_id = 10",
        [],
    )
    .unwrap();
    assert!(!covered(5));
}

#[test]
fn a_disconnect_signs_out_only_the_owner() {
    let dir = tempfile::tempdir().unwrap();
    let running =
        anibeam_foundation::testing::Running::open(dir.path(), anibeam_net::FakeHttp::new());
    let services = running.services.clone();
    services
        .store
        .write(|c| {
            c.execute_batch(
                "INSERT INTO profiles (id, created_at) VALUES (2, 0);
                 INSERT INTO tracker_accounts (profile_id, tracker, user_id, username, connected_at) VALUES
                     (1, 'anilist', 1, 'owner', 1), (2, 'anilist', 2, 'friend', 1);
                 INSERT INTO tracker_entries (profile_id, tracker, media_id, status, fetched_at) VALUES
                     (1, 'anilist', 10, 'watching', 0), (2, 'anilist', 10, 'completed', 0),
                     (2, 'anilist', 20, 'watching', 0);",
            )?;
            Ok(())
        })
        .unwrap();
    accounts::disconnect(&services, Tracker::Anilist).unwrap();
    let (owner_entries, friend_entries, friend_user): (i64, i64, Option<String>) = services
        .store
        .read(|c| {
            Ok(c.query_row(
                "SELECT (SELECT count(*) FROM tracker_entries WHERE profile_id = 1),
                        (SELECT count(*) FROM tracker_entries WHERE profile_id = 2),
                        (SELECT username FROM tracker_accounts WHERE profile_id = 2)",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?)
        })
        .unwrap();
    assert_eq!(
        (owner_entries, friend_entries, friend_user.as_deref()),
        (0, 2, Some("friend"))
    );
    running.shutdown();
}
