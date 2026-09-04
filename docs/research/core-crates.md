# Core crates: notify, rusqlite, keyring and the AniList client

Research for ticket #6, checked on 2026-09-03 against docs.rs, crates.io, the crate repositories, sqlite.org, the Linux kernel tree, the AniList and MyAnimeList docs, and the RFCs. Each claim carries the URL it came from. Versions are the newest stable release on crates.io on that date.

## Summary

1. The core watches library roots with notify 8.2.0 plus notify-debouncer-full 0.7.0 (the 9.0 line is still a release candidate). inotify charges one watch per directory, not per file, so a few hundred directories cost a few hundred kilobytes against a limit of 524288 on the owner's desktop.
2. The debouncer is not awaitWriteFinish. It emits Create about `timeout` after creation whether or not the writer has finished, so the scan job must ingest on `Access(Close(Write))` and rename events on Linux, and on macOS fall back to the per-file size check chokidar does today.
3. Files created inside a brand-new directory can be missed on every backend; the subtree walk the Electron app does on a new directory stays.
4. Storage is rusqlite 0.40.2 with `bundled` (SQLite 3.53.2), WAL mode with `synchronous=NORMAL`, one writer connection on its own thread, and every multi-table write inside one IMMEDIATE transaction. That replaces the serialised JSON writes with the same all-or-nothing guarantee.
5. Migrations use rusqlite_migration 2.6.0 (SQL in Rust consts, `PRAGMA user_version`). refinery 0.9.2 still caps rusqlite at 0.39 and needs a history table.
6. Secrets go through keyring-core 1.0.0 with zbus-secret-service-keyring-store 1.0.1 on Linux and apple-native-keyring-store 1.0.2 on macOS; the all-in-one `keyring` 4.2.0 crate is a thin wrapper around the same pieces. Keyring calls run on a blocking thread, never inside a tokio task.
7. The file fallback is a small store the core writes itself against the keyring-core traits: a 0600 JSON file in the data directory, chosen when the Secret Service store fails to construct. No maintained file store exists that is worth a second SQLite engine.
8. AniList queries stay hand-rolled over reqwest 0.13.4 with serde, porting today's query strings verbatim; graphql_client 0.16.0 works but pays a checked-in schema file and a derive per operation for queries the schema has not moved under.
9. Pacing is governor 0.10.4 per provider; 429 handling is backon 1.6.0 with `when` on 429 and `adjust` reading `Retry-After`. AniList is documented at 90 per minute and currently throttled to 30.
10. OAuth stays a hand-rolled loopback listener on 127.0.0.1:53682 as today: AniList's implicit grant puts the token in the URL fragment so the listener serves a forwarder page; MAL supports only the `plain` PKCE method. Paths come from directories 6.0.0 with `ProjectDirs::from("", "", "anibeam")`, which lands the config directory exactly where Electron keeps its userData today.

## Watching the library: notify and notify-debouncer-full

### Versions

notify's newest published version is 9.0.0-rc.5 (2026-08-30) and its newest stable is 8.2.0 (2025-08-03, MSRV 1.77). https://crates.io/api/v1/crates/notify and https://crates.io/api/v1/crates/notify/8.2.0

notify-debouncer-full's newest stable is 0.7.0 (2026-01-23, MSRV 1.85); 0.8.0-rc.2 tracks the notify 9 release candidates. 0.7.0 depends on notify ^8.2.0, notify-types ^2.0.0, file-id ^0.2.3 and walkdir ^2.4.0. https://crates.io/api/v1/crates/notify-debouncer-full and https://crates.io/api/v1/crates/notify-debouncer-full/0.7.0/dependencies

The event types live in the separate notify-types crate, 2.1.0. https://crates.io/api/v1/crates/notify-types

Recommendation: notify 8.2.0 with notify-debouncer-full 0.7.0. Move to 9.0 once it leaves release candidate; it adds `EventKindMask` for kernel-side filtering on inotify and `Config::with_fsevent_latency`, both useful here. https://raw.githubusercontent.com/notify-rs/notify/main/notify/CHANGELOG.md

### Recursive watching on Linux

`RecursiveMode::Recursive` means "Watch all sub-directories as well, including directories created after installing the watch". https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/config.rs

The inotify backend walks the root with walkdir and keeps directories only (`filter_dir` returns the entry only `if e.file_type().is_dir()`), so it adds one inotify watch per directory and none per file. When a `CREATE` event with `ISDIR` arrives under a recursive watch, it walks the new subtree and adds watches for it. It emits no events for files that already exist in that directory by then. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/inotify.rs

The kernel documents that race: "by the time you create a watch for the new subdirectory, new files (and subdirectories) may already exist inside the subdirectory. Therefore, you may want to scan the contents of the subdirectory immediately after adding the watch". https://man7.org/linux/man-pages/man7/inotify.7.html

That is the reason the Electron watcher walks a new directory's subtree itself on `addDir` ("files already present at dir-creation time do NOT reliably get their own `add` event"). The core keeps that walk. `src/main/services/watcher.ts` in this repository.

One inaccuracy in the crate docs: they say "for recursive watched folders each file and folder inside counts towards the limit". The source above only watches directories, and issue #69 records the 2016 change that stopped watching files. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/lib.rs and https://github.com/notify-rs/notify/issues/69

### inotify limits and the sysctl

inotify(7): `max_user_watches` "specifies an upper limit on the number of watches that can be created per real user ID"; `max_user_instances` limits inotify instances per user; `max_queued_events` caps the queue per instance and "Events in excess of this limit are dropped, but an IN_Q_OVERFLOW event is always generated." https://man7.org/linux/man-pages/man7/inotify.7.html

On the owner's desktop, `/proc/sys/fs/inotify/max_user_watches` is 524288, `max_user_instances` is 1024 and `max_queued_events` is 16384 (read locally on 2026-09-03, kernel 7.1.5-arch1-2).

The kernel default since 5.11 is memory scaled: `watches_max = (((si.totalram - si.totalhigh) / 100) << PAGE_SHIFT) / INOTIFY_WATCH_COST; watches_max = clamp(watches_max, 8192UL, 1048576UL);` where `INOTIFY_WATCH_COST` is `sizeof(struct inotify_inode_mark) + 2 * sizeof(struct inode)`, about 1 kB on 64-bit. https://raw.githubusercontent.com/torvalds/linux/master/fs/notify/inotify/inotify_user.c and https://github.com/torvalds/linux/commit/92890123749bafc317bbfacbe0a62ce08d78efb7

When the limit is hit, `inotify_add_watch` fails with ENOSPC: "The user limit on the total number of inotify watches was reached or the kernel failed to allocate a needed resource." https://man7.org/linux/man-pages/man2/inotify_add_watch.2.html

notify maps that to `ErrorKind::MaxFilesWatch`, "Can't watch (more) files, limit on the total number of inotify watches reached", and forwards it to the event handler. https://docs.rs/notify/latest/notify/enum.ErrorKind.html and https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/inotify.rs

A queue overflow arrives as `EventKind::Other` with `Flag::Rescan`; the scan job should treat that flag as "rescan this root". https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/inotify.rs

Cost for this library: several thousand files across a few hundred directories is a few hundred watches, well under a megabyte of kernel memory and far below 524288. The crate docs do warn that "When watching a very large amount of files, notify may fail to receive all events", which is the overflow case above. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/lib.rs

### macOS: FSEvents

`RecommendedWatcher` is `FsEventWatcher` on macOS unless the `macos_kqueue` feature is set, `INotifyWatcher` on Linux, and `PollWatcher` only for targets outside that list. `macos_fsevent` is a default feature. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/lib.rs and https://raw.githubusercontent.com/notify-rs/notify/main/notify/Cargo.toml

FSEvents is recursive by nature: "the file system events daemon will post a notification every time that any file in the monitored directory hierarchy changes." It also coalesces: "You will always receive at least one notification after the last change is made." https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/TechnologyOverview/TechnologyOverview.html

notify opens the stream with `kFSEventStreamCreateFlagFileEvents | kFSEventStreamCreateFlagNoDefer | kFSEventStreamCreateFlagWatchRoot`, so events name individual files. Apple: "Use this flag with care as it will generate significantly more events than without it." Default latency is zero. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/fsevent.rs and https://developer.apple.com/tutorials/data/documentation/coreservices/kfseventstreamcreateflagfileevents.json

When FSEvents drops events it sets `kFSEventStreamEventFlagMustScanSubDirs` and "you must recursively rescan the path listed in the event"; notify surfaces that as `EventKind::Other` with `Flag::Rescan`, the same shape as the inotify overflow. https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html and https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/fsevent.rs

There is no FSEvents equivalent of inotify's close-on-write: `fsevent.rs` has no `AccessKind` mapping at all. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/fsevent.rs

### Renames and moves

inotify pairs `IN_MOVED_FROM` and `IN_MOVED_TO` by a shared cookie, but the pairing is "inherently racy" and "if an object is renamed outside of a monitored directory, there may not even be an IN_MOVED_TO event." https://man7.org/linux/man-pages/man7/inotify.7.html

notify emits `Modify(Name(RenameMode::From))` and `Modify(Name(RenameMode::To))` with the cookie as the event tracker, and when the To's cookie matches the stored From it also emits `RenameMode::Both` carrying both paths. A move in from outside the tree yields only To; a move out yields only From and drops the watch. A directory renamed inside the tree gets its old watches removed and the new subtree walked and re-watched, with no events for the files inside. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/inotify.rs

FSEvents has no cookie; notify maps `ITEM_RENAMED` to `RenameMode::Any` and the debouncer decides From or To by whether the path still exists. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/fsevent.rs

The debouncer stitches pairs by tracker or by file ID and rewrites pending events on the old name to the new one, inserting one `RenameMode::Both` event with `paths: [old, new]`. Its `FileIdMap` cache "is used to stitch together rename events in case the notification back-end doesn't emit rename cookies"; `RecommendedCache` is `NoCache` on Linux and `FileIdMap` on macOS. https://raw.githubusercontent.com/notify-rs/notify/main/notify-debouncer-full/src/lib.rs and https://raw.githubusercontent.com/notify-rs/notify/main/notify-debouncer-full/src/cache.rs

Consequence for the scan job: a `.part` file renamed to its final name arrives as one Both event (Linux) or a From plus a To stitched by file ID (macOS). The ingest trigger for a moved-in episode is `RenameMode::To` or `Both` whose new path is a video.

### The awaitWriteFinish equivalent

chokidar today: `awaitWriteFinish: { stabilityThreshold: 500, pollInterval: 100 }`, meaning it polls the new file's size every 100 ms and emits `add` once the size has held for 500 ms. `src/main/services/watcher.ts`.

notify-debouncer-full's documented contract is thin: "Timeout is the amount of time after which a debounced event is emitted. If `tick_rate` is `None`, notify will select a tick rate that is 1/4 of the provided timeout." https://docs.rs/notify-debouncer-full/latest/notify_debouncer_full/fn.new_debouncer.html

The source settles what that means. Each event is stamped on arrival; a path's queue is flushed when its front event is older than `timeout`, and same-kind events collapse, so a stream of Modify events yields one Modify `timeout` after the last write. Create is the exception: while the queue's front is a Create, incoming `Modify(Data | Metadata | Any | Other)` events are dropped (changelog 0.6.0: "skip all `Modify` events right after a `Create` event, unless it's a rename event"). Nothing extends the Create's timer, so a file created and written for 30 seconds produces a Create about `timeout` after creation while still being written. https://raw.githubusercontent.com/notify-rs/notify/main/notify-debouncer-full/src/lib.rs and https://raw.githubusercontent.com/notify-rs/notify/main/notify-debouncer-full/CHANGELOG.md

So the debouncer's Create is not a "write finished" signal. The signals that are:

- On Linux, `IN_CLOSE_WRITE` ("File opened for writing was closed"), which notify exposes as `EventKind::Access(AccessKind::Close(AccessMode::Write))`. The 8.2.0 inotify mask includes `CLOSE_WRITE`, and the debouncer passes Access events through untouched. https://man7.org/linux/man-pages/man7/inotify.7.html, https://raw.githubusercontent.com/notify-rs/notify/notify-8.2.0/notify/src/inotify.rs and https://raw.githubusercontent.com/notify-rs/notify/main/notify-debouncer-full/src/lib.rs
- On both platforms, the rename To or Both for a file that landed by `mv` or by a downloader's final rename.
- On macOS, nothing from the kernel. The core keeps chokidar's rule for that case: stat the candidate file every 100 ms until its size has held for 500 ms, then ingest. That is a bounded check on one new file, not a periodic rescan, so it does not conflict with the no-polling decision.

Delivery lands between `timeout` and `timeout + tick`, so a 500 ms timeout with the default tick puts an event 500 to 625 ms after the last write. https://raw.githubusercontent.com/notify-rs/notify/main/notify-debouncer-full/src/lib.rs

### Filtering

Neither crate filters by path. `Config` has only `poll_interval`, `compare_contents`, `follow_symlinks`, `event_kinds`, `windows_path_separator_style` and `fsevent_latency`; the debouncer's own example filters in the handler. The dotfile, `.part`, `.crdownload` and `.tmp` rules from `watcher.ts` move into the core's handler. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/config.rs and https://github.com/notify-rs/notify/blob/main/examples/debouncer_full.rs

There is no hook to skip directories during the recursive walk; the only barrier is for symlink watches. Ignoring a directory by path in the handler still leaves its watch in place. To keep an ignored subtree out of the kernel entirely the core would have to walk the tree itself and add `NonRecursive` watches per directory; for this library that is not worth it. https://raw.githubusercontent.com/notify-rs/notify/main/notify/src/inotify.rs

### Cost of the debouncer's cache

`Debouncer::watch` calls `cache.add_path`, and `FileIdMap::add_path` walks the whole tree and stats every entry, files included, to record inode numbers. On Linux `RecommendedCache` is `NoCache`, so that cost is zero; on macOS it is one `metadata` call per file at watch time. https://raw.githubusercontent.com/notify-rs/notify/main/notify-debouncer-full/src/file_id_map.rs and https://raw.githubusercontent.com/notify-rs/notify/main/notify-debouncer-full/src/cache.rs

### Polling

`PollWatcher` exists ("By default scans through all files and checks for changed entries based on their change date") and its default interval is 30 s, which the docs call "extremely expensive for large file trees". notify never falls back to it at runtime; `RecommendedWatcher` is a compile-time alias. The core does not use it. https://docs.rs/notify/latest/notify/poll/struct.PollWatcher.html and https://docs.rs/notify/latest/notify/struct.Config.html

### Not verified

- docs.rs builds notify only for x86_64 Linux, so there is no `FsEventWatcher` page; the FSEvents claims come from `fsevent.rs` and Apple's guide.
- No Apple or notify statement says whether Create events are delivered for files inside a brand-new directory under FSEvents. Apple's guidance for a per-directory event is to scan that directory, so the subtree walk stays on macOS too.
- Startup time for adding watches over a large tree: inotify(7) says it "can take a significant amount time for large directory trees"; no measurement exists in the notify docs or tracker for trees of this size.

## Storage: rusqlite with the bundled SQLite

### Versions and features

rusqlite 0.40.2 was published 2026-08-08 and depends on libsqlite3-sys 0.38.2. https://crates.io/api/v1/crates/rusqlite

The bundled SQLite in 0.40.2 is 3.53.2 (`#define SQLITE_VERSION "3.53.2"` in the vendored header). https://raw.githubusercontent.com/rusqlite/rusqlite/v0.40.2/libsqlite3-sys/sqlite3/sqlite3.h

`bundled` means "libsqlite3-sys will use the cc crate to compile SQLite or SQLCipher from source and link against that." No system libsqlite3 is needed, which keeps the PKGBUILD free of a versioned sqlite dependency. https://raw.githubusercontent.com/rusqlite/rusqlite/master/README.md

Feature wiring: `bundled = ["libsqlite3-sys?/bundled", "modern_sqlite"]`; `bundled-full` adds the whole `modern-full` set (array, backup, blob, chrono, collation, csvtab, functions, hooks, jiff, serde_json, time, trace, url, uuid, vtab, window and more). https://raw.githubusercontent.com/rusqlite/rusqlite/master/Cargo.toml

Recommendation: `rusqlite = { version = "0.40", features = ["bundled", "serde_json", "backup"] }`, adding `functions`, `hooks` or `trace` only when a use appears. `bundled-full` drags in csv, url, jiff and time that the core does not need.

The bundled build passes `-DSQLITE_DEFAULT_FOREIGN_KEYS=1`, `-DSQLITE_ENABLE_FTS5`, `-DSQLITE_ENABLE_JSON1`, `-DSQLITE_ENABLE_RTREE`, `-DSQLITE_THREADSAFE=1` and `-DSQLITE_USE_URI`, and never sets `SQLITE_OMIT_JSON`. https://raw.githubusercontent.com/rusqlite/rusqlite/master/libsqlite3-sys/build.rs

MSRV: the v0.40.2 release note says "Lower MSRV to 1.88.0". https://github.com/rusqlite/rusqlite/releases/tag/v0.40.2

### Thread model

`Connection` is `Send` and `!Sync`; `Transaction` is neither. https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html and https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html

`Connection::open` uses `SQLITE_OPEN_NO_MUTEX`, under which "separate threads are allowed to use SQLite at the same time, as long as each thread is using a different database connection." https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.open and https://sqlite.org/c3ref/open.html

tokio's guidance: "Use spawn_blocking for short-lived blocking operations" and "Use dedicated threads for long-lived or persistent blocking workloads". https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html

tokio-rusqlite 0.7.0 (2025-11-16) implements exactly the dedicated-thread pattern ("A thread is spawned for each opened connection handle ... provided function is boxed, sent to the thread through mpsc channel and executed") but pins rusqlite 0.37, three minors behind, with dependabot bumps to 0.39 and 0.40 open and unmerged. https://crates.io/api/v1/crates/tokio-rusqlite, https://raw.githubusercontent.com/programatik29/tokio-rusqlite/master/Cargo.toml and https://github.com/programatik29/tokio-rusqlite/pulls

Recommendation: the core owns one writer `Connection` on a dedicated std thread fed by an mpsc of boxed closures with a oneshot per call. That is about thirty lines and sits naturally behind the synchronous facade, so no dependency stuck on an old rusqlite. Reader connections can be opened per job or from a small pool.

### WAL mode

"PRAGMA journal_mode=WAL is persistent. If a process sets WAL mode, then closes and reopens the database, the database will come back in WAL mode." "Writers and readers can run at the same time. However, since there is only one WAL file, there can only be one writer at a time." The `-wal` and `-shm` files live beside the database, opening needs write access on that directory, and the last connection to close checkpoints and deletes them. WAL "does not work over a network filesystem". https://sqlite.org/wal.html

Auto checkpoint runs "whenever a COMMIT occurs that causes the WAL file to be 1000 pages or more in size". https://sqlite.org/pragma.html#pragma_wal_autocheckpoint

`synchronous=NORMAL` is the right setting: "WAL mode is always consistent with synchronous=NORMAL, but WAL mode does lose durability. A transaction committed in WAL mode with synchronous=NORMAL might roll back following a power loss or system crash. Transactions are durable across application crashes regardless of the synchronous setting or journal mode." https://sqlite.org/pragma.html#pragma_synchronous

Set it with `conn.pragma_update(None, "journal_mode", "WAL")`; `pragma_update_and_check` returns the applied value, which matters for journal_mode because SQLite may refuse it. https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.pragma_update

Foreign keys are off by default in stock SQLite and "must be enabled separately for each database connection"; the bundled build flips the default on. Set `PRAGMA foreign_keys=ON` per connection anyway so a future switch to a system SQLite cannot silently drop enforcement. https://sqlite.org/foreignkeys.html and https://raw.githubusercontent.com/rusqlite/rusqlite/master/libsqlite3-sys/build.rs

`Connection::busy_timeout` sets "a busy handler that sleeps for a specified amount of time when a table is locked." Set it on every connection. https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.busy_timeout

### Atomic multi-table writes

Today every persisted JSON document is written to a PID-suffixed temp file and renamed, with writers serialised through a transaction queue. The SQLite equivalent:

- `conn.transaction()` begins DEFERRED; `transaction_with_behavior(TransactionBehavior::Immediate)` starts the write at BEGIN. "Transactions will roll back by default. Use commit method to explicitly commit the transaction". https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.transaction and https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html
- `DropBehavior::Rollback` is the default; an early `?` return rolls the whole write back. https://docs.rs/rusqlite/latest/rusqlite/enum.DropBehavior.html
- Under DEFERRED, "Subsequent write statements will upgrade the transaction to a write transaction if possible, or return SQLITE_BUSY." IMMEDIATE "causes the database connection to start a new write immediately". In WAL mode EXCLUSIVE and IMMEDIATE are the same. https://sqlite.org/lang_transaction.html
- The failure IMMEDIATE avoids is `SQLITE_BUSY_SNAPSHOT`, raised when a WAL connection "tries to promote a read transaction into a write transaction but finds that another database connection has already written to the database". https://sqlite.org/rescode.html
- "Atomic commit means that either all database changes within a single transaction occur or none of them occur", including across "an operating system crash or power failure". https://sqlite.org/atomiccommit.html
- Nested work uses savepoints: "Transactions created using BEGIN...COMMIT do not nest. For nested transactions, use the SAVEPOINT and RELEASE commands." rusqlite exposes `Savepoint`, which also rolls back on drop. https://sqlite.org/lang_transaction.html and https://docs.rs/rusqlite/latest/rusqlite/struct.Savepoint.html
- `prepare_cached` (default `cache` feature) keeps hot statements compiled. https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.prepare_cached

Pattern: one writer connection on its own thread; every call that touches more than one table runs inside `transaction_with_behavior(Immediate)` and ends with `commit()`; readers use their own connections and never block the writer under WAL. That gives the old "one document, all or nothing" guarantee across tables, with SQLite rather than rename providing crash safety.

### Migrations

rusqlite_migration 2.6.0 (2026-05-28) pins rusqlite 0.40.0 and followed 0.38 and 0.39 within weeks of each release. https://crates.io/api/v1/crates/rusqlite_migration, https://raw.githubusercontent.com/cljoly/rusqlite_migration/master/Cargo.toml and https://raw.githubusercontent.com/cljoly/rusqlite_migration/master/CHANGELOG.md

It tracks the schema in `PRAGMA user_version` ("It's much lighter as it is just an integer at a fixed offset in the SQLite file"), which SQLite defines as "an integer that is available to applications to use however they want." https://raw.githubusercontent.com/cljoly/rusqlite_migration/master/README.md and https://sqlite.org/pragma.html#pragma_user_version

API: `M::up(sql)` with optional `.down(sql)` and `.foreign_key_check()`; `Migrations::new(vec![...])` or the const `Migrations::from_slice(&[...])`; `to_latest(&mut conn)` applies pending migrations atomically inside one transaction; `validate()` runs them on an in-memory database for a unit test; the `from-directory` feature loads SQL files through include_dir. https://docs.rs/rusqlite_migration/latest/rusqlite_migration/struct.M.html, https://docs.rs/rusqlite_migration/latest/rusqlite_migration/struct.Migrations.html and https://raw.githubusercontent.com/cljoly/rusqlite_migration/master/rusqlite_migration/src/lib.rs

refinery 0.9.2 (2026-06-10) pins `rusqlite = ">= 0.23, <= 0.39"`, so it does not accept 0.40; the PR to allow it was open on 2026-08-31. It reads `V{n}__{name}.sql` files through `embed_migrations!` and records history in a `refinery_schema_history` table (version, name, applied_on, checksum). In the default non-grouped mode each migration and its history row commit separately. https://crates.io/api/v1/crates/refinery, https://raw.githubusercontent.com/rust-db/refinery/main/refinery_core/Cargo.toml, https://github.com/rust-db/refinery/issues?q=rusqlite+0.40, https://docs.rs/refinery/latest/refinery/macro.embed_migrations.html, https://raw.githubusercontent.com/rust-db/refinery/main/refinery_core/src/traits/mod.rs and https://raw.githubusercontent.com/rust-db/refinery/main/refinery_core/src/traits/sync.rs

Recommendation: rusqlite_migration. It is on rusqlite 0.40 now, carries one optional dependency, keeps the SQL in Rust consts beside the code that uses it, and `validate()` gives the migration chain a test for free.

### JSON columns

The `serde_json` feature maps `serde_json::Value` to TEXT on write (NULL and numbers get native types) and parses TEXT or BLOB on read. https://raw.githubusercontent.com/rusqlite/rusqlite/master/src/types/serde_json.rs

"The JSON functions and operators are built into SQLite by default, as of SQLite version 3.38.0", and JSONB storage exists since 3.45.0. The bundled build never defines `SQLITE_OMIT_JSON`, so `json_extract` and `->>` work at 3.53.2. That lets the franchise graph and matched metadata land as JSON columns first and be split into tables when the schema settles. https://sqlite.org/json1.html and https://raw.githubusercontent.com/rusqlite/rusqlite/master/libsqlite3-sys/build.rs

### Backup and integrity

The `backup` feature wraps SQLite's online backup API; `Connection::backup(name, dst_path, progress)` is the one-call form. "The copy operation may be done incrementally, in which case the source database does not need to be locked for the duration of the copy". That is the export path. https://docs.rs/rusqlite/latest/rusqlite/backup/index.html and https://sqlite.org/backup.html

`PRAGMA integrity_check` "does a low-level formatting and consistency check of the database". https://sqlite.org/pragma.html#pragma_integrity_check

### File location

The `-wal` and `-shm` sidecars sit next to the database and need write access on the directory (WAL page above). A missing parent directory surfaces as `SQLITE_CANTOPEN`, so the core runs `create_dir_all` before `Connection::open`. https://sqlite.org/rescode.html

### Not verified

- Whether rusqlite 0.40.2 actually compiles on Rust 1.88.0: the release note claims it, but the tag's Cargo.toml carries no `rust-version`, so cargo will not enforce it.
- rusqlite_migration's `rust-version = "1.95"` comes from its manifest; not tested against a toolchain.
- No sentence on sqlite.org states that the parent directory must exist before `open`; that follows from the CANTOPEN description and `open(2)` behaviour.

## Secrets: keyring

### The ecosystem in 2026

`keyring` 4.0.0 shipped 2026-04-26 and is at 4.2.0 (2026-08-29). The library and API moved into `keyring-core` 1.0.0 (2026-04-21) and every credential store became its own crate. https://crates.io/api/v1/crates/keyring and https://crates.io/api/v1/crates/keyring-core

The wrapper's own guidance: "developers should take dependencies on the keyring-core crate and the specific credential stores they want to use." Its `v1` feature is only `apple-native-keyring-store/keychain`, `windows-native-keyring-store` and `zbus-secret-service-keyring-store`, and its store selection is a `LazyLock` that calls `zbus_secret_service_keyring_store::Store::new()` on Linux and `apple_native_keyring_store::keychain::Store::new()` on macOS. https://raw.githubusercontent.com/open-source-cooperative/keyring-rs/main/README.md, https://raw.githubusercontent.com/open-source-cooperative/keyring-rs/main/Cargo.toml and https://raw.githubusercontent.com/open-source-cooperative/keyring-rs/main/src/v1.rs

In keyring-core the app sets a store at startup with `set_default_store` and releases it with `unset_default_store`; `Entry::new(service, user)` then offers `set_password`, `get_password`, `set_secret`, `get_secret` and `delete_credential`. Store objects must be `Send + Sync`. https://raw.githubusercontent.com/open-source-cooperative/keyring-core/main/README.md, https://docs.rs/keyring-core/latest/keyring_core/struct.Entry.html and https://docs.rs/keyring-core/latest/keyring_core/

Error variants that matter here: `NoEntry` (nothing stored), `Ambiguous` (several items match), `NoStorageAccess` ("the underlying secure storage holding saved items could not be accessed"), `PlatformFailure` ("runtime failure in the underlying platform storage system"), `NoDefaultStore`, `TooLong`, `BadEncoding`. https://docs.rs/keyring-core/latest/keyring_core/error/enum.Error.html

Recommendation: depend on `keyring-core = "1"` plus `zbus-secret-service-keyring-store = "1"` (Linux) and `apple-native-keyring-store = "1"` with the `keychain` feature (macOS), and skip the `keyring` wrapper. The core then owns store selection, which is exactly where the file fallback has to hook in.

### Linux: Secret Service

Two stores talk to the Secret Service. `zbus-secret-service-keyring-store` 1.0.1 is pure Rust on `secret-service` 5.2.0 and zbus 5; `dbus-secret-service-keyring-store` 1.0.1 links libdbus through `dbus-secret-service` 4.1.0 and offers a `vendored` feature to link it statically. https://raw.githubusercontent.com/open-source-cooperative/zbus-secret-service-keyring-store/main/Cargo.toml, https://crates.io/api/v1/crates/secret-service, https://docs.rs/crate/dbus-secret-service-keyring-store/latest/features and https://crates.io/api/v1/crates/dbus-secret-service

Items land in the default collection: "This store, by default, creates items in the _default_ collection (aka the user's login collection)". The controlled attributes are `service`, `username` and optional `target`, and the label defaults to `keyring:{user}@{service}`, which is what gnome-keyring's Seahorse will show. Searches span all collections. https://raw.githubusercontent.com/open-source-cooperative/zbus-secret-service-keyring-store/main/src/lib.rs

Features: enable exactly one of `crypto-rust`, `crypto-openssl`, `rt-tokio-crypto-rust`, `rt-tokio-crypto-openssl`, `rt-async-io-crypto-rust`, `rt-async-io-crypto-openssl`. The README says an app with an async runtime should pick the matching `rt-` variant, so the core uses `rt-tokio-crypto-rust`. https://raw.githubusercontent.com/open-source-cooperative/zbus-secret-service-keyring-store/main/README.md and https://docs.rs/crate/secret-service/latest/features

The store API is synchronous (`Store::new() -> Result<Arc<Self>>`) on top of zbus's blocking layer, and zbus warns "you must not use them in async contexts because of the infamous async sandwich footgun". So every keyring call in the core runs on `spawn_blocking` or a plain thread, never inside a tokio task. https://raw.githubusercontent.com/open-source-cooperative/zbus-secret-service-keyring-store/main/src/store.rs and https://docs.rs/zbus/latest/zbus/blocking/index.html

Error mapping: `Locked`, `NoResult` and `Prompt` from secret-service become `NoStorageAccess`; everything else, including a missing session bus or no Secret Service on it, becomes `PlatformFailure`. https://raw.githubusercontent.com/open-source-cooperative/zbus-secret-service-keyring-store/main/src/errors.rs

Unlocking: a search that hits locked items "will prompt the user to unlock them before returning". On the owner's desktop gnome-keyring unlocks the login collection at login, so no prompt appears. Headless boxes are documented as a known problem and need the keyring unlocked by hand. https://raw.githubusercontent.com/open-source-cooperative/zbus-secret-service-keyring-store/main/src/store.rs and https://raw.githubusercontent.com/open-source-cooperative/zbus-secret-service-keyring-store/main/src/lib.rs

`linux-keyutils-keyring-store` 1.0.0 is a kernel keyring, "completely in-memory and will not persist across reboots"; its own docs say to "Consider the keyring a secure cache". Not a fit for tokens that must survive a reboot. https://docs.rs/linux-keyutils-keyring-store/latest/linux_keyutils_keyring_store/

Choice between the two Secret Service stores: zbus, because the core already runs tokio, it adds no C library, and it is what the `keyring` wrapper itself picks. The dbus store is the swap if zbus misbehaves on some desktop; Arch ships libdbus regardless.

### macOS: Keychain

`apple-native-keyring-store` 1.0.2 has two modules. `keychain` (feature `keychain`) is for apps "not code-signed by a provisioning profile (e.g., command-line apps)" and is the one compatible with keyring v3; `protected` needs a provisioning profile and fails in an unsigned app with "PlatformError with code -34018 A required entitlement isn't present." It builds on security-framework 3.7. https://raw.githubusercontent.com/open-source-cooperative/apple-native-keyring-store/main/README.md and https://docs.rs/apple-native-keyring-store/latest/apple_native_keyring_store/

The macOS shell uses the `keychain` module until it ships signed. That is a note for the spec's macOS appendix.

### The file fallback

Nothing in the ecosystem is a plain file store fit for production:

- keyring-core's `sample` store persists to `keyring-sample-store.ron` in the temp directory and says "This store is explicitly not for use in production apps! It's neither robust nor secure." It is a reference implementation of the store traits. https://docs.rs/keyring-core/latest/keyring_core/sample/index.html
- `db-keystore` 0.5.2 (first release 2026-01-21, one author) is "Encrypted SQLite credential store for the `keyring-core` API, backed by turso", defaulting to `$XDG_STATE_HOME/keystore.db`. Its encryption needs a hex key the app must supply, which for a local fallback only moves the secret into another file, and turso is a second SQLite engine next to the bundled one. https://crates.io/api/v1/crates/db-keystore, https://docs.rs/db-keystore/latest/db_keystore/struct.DbKeyStoreConfig.html and https://raw.githubusercontent.com/stevelr/db-keystore/main/README.md

A custom store needs two traits. `CredentialStoreApi` requires `vendor`, `id`, `build(service, user, modifiers) -> Result<Entry>` and `as_any`, with `persistence` and `search` provided. `CredentialApi` requires `set_secret`, `get_secret`, `delete_credential`, `get_credential`, `get_specifiers` and `as_any`; `set_password` and `get_password` are provided on top. https://docs.rs/keyring-core/latest/keyring_core/api/trait.CredentialStoreApi.html and https://docs.rs/keyring-core/latest/keyring_core/api/trait.CredentialApi.html

Recommendation: the core ships `FileStore`, a hundred-line implementation of those traits over one JSON object at `<data_dir>/secrets.json` created with mode 0600, keyed by `service` and `user`, written with the same tmp-and-rename the Electron app uses. Unencrypted is consistent with the export decision (tokens as plain JSON, trusting the user), and a hand-written store keeps the rest of the core on one `Entry` API.

Selection at startup, on a blocking thread: try `zbus_secret_service_keyring_store::Store::new()`; on `Err` set `FileStore` as the default instead. On a later `PlatformFailure` or `NoStorageAccess` from a set or get, retry through `FileStore` and record the switch so the token's location stays stable. `NoEntry` is not a failure; it means "not connected". The `keyring` wrapper's `Entry::store_status()` shows the same one-time-init shape. https://raw.githubusercontent.com/open-source-cooperative/keyring-rs/main/src/v1.rs

### Not verified

- Whether gnome-keyring or KDE Wallet impose a size limit on an item; the AniList JWT is a few hundred bytes, so `TooLong` is unlikely.
- What the keyring wiki lists as the current store set: its page defers to the CLI crate rather than listing them. https://github.com/open-source-cooperative/keyring-rs/wiki/Keyring
- I did not build the zbus store against tokio to confirm that `spawn_blocking` avoids the sandwich; the rule comes from the zbus docs.

## AniList and MAL: GraphQL client, rate limiter, OAuth loopback

### GraphQL client

graphql_client 0.16.0 (2026-01-15) generates modules at compile time from `#[derive(GraphQLQuery)]` with `schema_path` and `query_path`. The schema must be checked in as SDL (`.graphql`) or introspection JSON, obtained with `cargo install graphql_client_cli` and `graphql-client introspect-schema`; the `reqwest` feature adds `post_graphql`, and custom scalars need matching Rust types in scope. https://crates.io/api/v1/crates/graphql_client, https://docs.rs/graphql_client/latest/graphql_client/, https://raw.githubusercontent.com/graphql-rust/graphql-client/main/README.md and https://crates.io/api/v1/crates/graphql_client_cli

AniList publishes no SDL file in its docs repository (only the guide markdown); the schema would come from introspecting https://graphql.anilist.co. https://github.com/AniList/docs

The hand-rolled shape is a `POST` to `https://graphql.anilist.co` with `{"query": ..., "variables": ...}` and a response of `{"data": ..., "errors": [...]}`, which is what `anilistHandler.ts` sends today, and a 429 arrives as `{"data": null, "errors": [{"message": "Too Many Requests.", "status": 429}]}`. https://docs.anilist.co/guide/rate-limiting

Recommendation: hand-rolled over reqwest and serde, one Rust struct per response shape. The Electron app carries about seventeen operations as query strings across `anilistHandler.ts` and `trackerHandler.ts` (search, media with relations for the franchise graph, airing schedule, viewer, list entries, save and delete list entry, and their MAL counterparts), and the port copies those strings verbatim with a serde struct each. graphql_client would add a checked-in introspection file and a derive per operation for type checking against a schema that has not moved under these queries. If schema drift starts breaking queries, graphql_client is the upgrade, and nothing in the hand-rolled shape blocks it.

### reqwest

reqwest 0.13.4 (2026-05-25). 0.13.0 made rustls the default TLS backend ("rustls is now the default TLS backend, instead of native-tls"), renamed `rustls-tls` to `rustls`, switched the crypto provider to aws-lc, and uses `rustls-platform-verifier` for roots, so the binary trusts the system store with no OpenSSL runtime dependency. `query` and `form` became opt-in features. https://crates.io/api/v1/crates/reqwest and https://raw.githubusercontent.com/seanmonstar/reqwest/master/CHANGELOG.md

Features for the core: `json` and `form` (the MAL token exchange is form encoded). `http2`, `default-tls` and `charset` are on by default. https://docs.rs/reqwest/latest/reqwest/

"The `Client` holds a connection pool internally to improve performance by reusing connections and avoiding setup overhead, so it is advised that you create one and reuse it." It "already uses an `Arc` internally", so one client per provider, cloned into jobs. https://docs.rs/reqwest/latest/reqwest/struct.Client.html

Authenticated AniList requests carry the token "in the `Authorization` header of your request as a "Bearer" token"; the token is a JWT the core can decode for the user id and expiry. https://docs.anilist.co/guide/auth/authenticated-requests

### Provider limits

AniList: "The AniList API has a rate limit of 90 requests per minute." with the standing warning "The API is currently in a degraded state and is limited to **30 requests per minute**." Responses carry `X-RateLimit-Limit` and `X-RateLimit-Remaining`; exceeding it earns "a 1 minute timeout" and 429 responses with `Retry-After` ("the number of seconds you should wait before making another request") and `X-RateLimit-Reset` (a Unix timestamp). A separate burst limiter exists "to stop you from hammering the API with too many requests in a very short period of time", with no number given. Raises are not being granted. https://docs.anilist.co/guide/rate-limiting

Jikan: "Per Minute 60 requests, Per Second 3 requests", daily unlimited, responses cached for 24 hours, and 429 when "You are being rate limited by Jikan or MyAnimeList is rate-limiting our servers". https://raw.githubusercontent.com/jikan-me/jikan-rest/master/storage/api-docs/api-docs.json (the spec behind https://docs.api.jikan.moe/)

TMDB: no hard limit, "some upper limits to help mitigate needlessly high bulk scraping", roughly 40 requests per second, and "respect the `429` if you receive one". https://developer.themoviedb.org/docs/rate-limiting

MAL's own API documents no request limit that I could find on the authorization page; treat 429 as the signal.

Today's limiter runs one queue per provider with a fixed gap between request starts (800 ms for AniList, 120 ms for TMDB) and retries a 429 at 1, 2, 4, 8, 16 and 32 seconds capped at 60. `src/main/utils/rateLimiter.ts` and `src/main/handlers/anilistHandler.ts`.

### governor for pacing

governor 0.10.4 (2025-12-16) "implements the Generic Cell Rate Algorithm". `Quota::per_minute(n)` allows a burst of n at once; `Quota::with_period(d)` replenishes one cell per `d` and `allow_burst(n)` caps the burst. `RateLimiter::direct(quota)` needs the `std` feature; `until_ready()` "Asynchronously resolves as soon as the rate limiter allows it" and `until_ready_with_jitter` spreads waiters. The limiter is `Send + Sync` and the async waits go through futures-timer, so it is runtime agnostic and works under tokio. https://crates.io/api/v1/crates/governor, https://docs.rs/governor/latest/governor/, https://docs.rs/governor/latest/governor/struct.Quota.html and https://docs.rs/governor/latest/governor/struct.RateLimiter.html

A `with_period` quota reproduces today's fixed gaps: 800 ms with burst 1 for AniList, 1100 ms with burst 1 for Jikan (which also satisfies its 3 per second rule), 120 ms for TMDB. `src/main/handlers/malHandler.ts` and `src/main/handlers/tmdbHandler.ts`. Whether AniList's gap should widen to 2 s to sit under the degraded 30 per minute is a contract decision; the 429 path below covers either choice.

governor never reads a response. It paces; it does not back off.

### backon for the 429 path

backon 1.6.0 (2025-10-18). `Retryable` adds `.retry(backoff)` to async functions; `ExponentialBuilder` gives min delay, max delay, factor, max attempts and jitter; `when(|e| ...)` limits retries to matching errors; `adjust(|e, next| ...)` "Sets the function to adjust the backoff duration for retry attempts" and returning `None` stops; the `tokio-sleep` feature uses `tokio::time::sleep`. https://crates.io/api/v1/crates/backon, https://docs.rs/backon/latest/backon/ and https://docs.rs/backon/latest/backon/struct.Retry.html

Recommendation: each provider client wraps a request in `governor.until_ready()` then backon with `when` matching 429 and `adjust` returning `Retry-After` seconds when the header is present, else the exponential schedule (1 s min, 60 s max, six attempts) that mirrors today's numbers. `oauth2`-style crates and `tokio-retry` are not needed. The `backoff` crate was not evaluated; backon is maintained and covers the need.

### AniList OAuth: implicit grant

"Scopes are not supported. Access tokens provide (almost) full access to a user's data." "AniList access tokens are long-lived. They will remain valid for 1 year from the time they are issued." "Refresh tokens are not currently supported. Once a token expires, you will need to re-authenticate your users." The registered redirect URL "can be any valid URI, including custom URI schemes." https://docs.anilist.co/guide/auth/

Implicit grant: redirect to `https://anilist.co/api/v2/oauth/authorize?client_id={client_id}&response_type=token`; after approval "they will be redirected back to the redirect URI you specified in your application settings. Their redirect will include a JWT `access_token` parameter in the URL **fragment**." The docs list only `client_id` and `response_type`, no `redirect_uri` and no `state`. https://docs.anilist.co/guide/auth/implicit

The authorization code grant posts `client_id`, `client_secret`, `redirect_uri` and `code` to `https://anilist.co/api/v2/oauth/token`, so it needs the client secret and is for "server-based applications". A desktop binary cannot keep a secret, so the core stays on the implicit grant, as the Electron app does. https://docs.anilist.co/guide/auth/authorization-code and https://docs.anilist.co/guide/auth/

Auth pin: setting the redirect URL to `https://anilist.co/api/v2/oauth/pin` makes AniList show the token for manual copy. It "can be used for both the Authorization Code Grant and the Implicit Grant" and is the escape hatch for a machine where the loopback port is blocked. https://docs.anilist.co/guide/auth/

Observed, not documented: the Electron handler notes that adding `redirect_uri`, `state` or `scope` to AniList's authorize URL returns an `unsupported_grant_type` error, so it sends only the two documented parameters. `src/main/handlers/trackerHandler.ts`

Consequence for the listener: a URL fragment never reaches an HTTP server. The listener answers the first hit on `/callback` with a page whose script rewrites `location.hash` into a query string and reloads, then reads `access_token` from the second hit. That is the `FRAGMENT_FORWARDER` page the Electron app serves; the core ships the same page as a string constant. `src/main/handlers/trackerHandler.ts`

Because the implicit grant carries no `state`, the CSRF binding RFC 6749 asks for ("the client MUST implement CSRF protection for its redirection URI", section 10.12) cannot be a round-tripped value. The core's substitute: the listener only exists while a connect call is pending, accepts one token, and the core validates it with a `Viewer` query before storing it. https://www.rfc-editor.org/rfc/rfc6749#section-10.12

### MAL OAuth: authorization code with PKCE

MAL's page: the code verifier needs "A minimum length of 43 characters and a maximum length of 128 characters"; for `code_challenge_method`, "Currently, only the `plain` method is supported." and it "Defaults to `plain` if not present." Authorize at `https://myanimelist.net/v1/oauth2/authorize` with `response_type=code`, `client_id`, `code_challenge`, `state` (RECOMMENDED) and `redirect_uri` ("If you registered only one redirection URI in advance, you can omit this parameter. If you set this, the value must exactly match one of your pre-registered URIs."). Exchange at `https://myanimelist.net/v1/oauth2/token` with `grant_type=authorization_code`, `code`, `code_verifier`, `client_id` and, if the app has one, `client_secret`; refresh with `grant_type=refresh_token`. https://myanimelist.net/apiconfig/references/authorization

Token lifetime on that page is contradictory: the overview table says "Access Token lifetime: One hour." and "Refresh Token lifetime: One month.", while the example response shows `"expires_in": 2415600` (about 28 days) and the refresh section says "The expiration date of the new token you receive is _one month_ from when you receive it." The core trusts `expires_in` from the actual response, as `exchangeMalCode` does today, and refreshes when the stored expiry nears. https://myanimelist.net/apiconfig/references/authorization and `src/main/handlers/trackerHandler.ts`

RFC 7636 says "If the client is capable of using 'S256', it MUST use 'S256'", and warns that plain "does not protect against the eavesdropping of the initial request." MAL leaves no choice; the verifier is 32 random bytes base64url encoded (43 characters), sent as its own challenge. https://www.rfc-editor.org/rfc/rfc7636#section-4.2 and https://www.rfc-editor.org/rfc/rfc7636#section-7.2

The oauth2 crate 5.0.0 (2025-01-21) supports this (`PkceCodeChallenge::new_random_plain`, "Use is discouraged unless the endpoint does not support SHA-256"), but the flow is two URLs and one form post; the hand-rolled version in `trackerHandler.ts` is under a hundred lines and the core ports it rather than adopting a typed OAuth client for two providers. https://crates.io/api/v1/crates/oauth2 and https://docs.rs/oauth2/latest/oauth2/struct.PkceCodeChallenge.html

### The loopback listener

RFC 8252 section 7.3: native apps "can use the loopback interface to receive the OAuth redirect" at `http://127.0.0.1:{port}/{path}`; section 8.3: plain http "is acceptable for loopback interface redirect URIs as the HTTP request never leaves the device", and "Specifying a redirect URI with the loopback IP literal rather than localhost avoids inadvertently listening on network interfaces other than the loopback interface." The RFC wants servers to accept any port, but both providers match the registered URI exactly, so the port stays fixed. https://www.rfc-editor.org/rfc/rfc8252#section-7.3 and https://www.rfc-editor.org/rfc/rfc8252#section-8.3

The registered URI today is `http://127.0.0.1:53682/callback`, pinned "so the URL you register with AniList / MAL once keeps working forever". The core keeps host, port and path. `src/shared/trackerConstants.ts`

Implementation: `tokio::net::TcpListener::bind(("127.0.0.1", 53682))`, accept one connection at a time, read the request line, answer with the forwarder page, the success page or the error page, close. That is a few dozen lines with no HTTP crate; `tiny_http` or axum would each be a larger dependency than the code they replace. A bind failure (`EADDRINUSE`) is reported to the shell as "another AniBeam is mid-connect", as today.

### Not verified

- MAL's app types and which of them get a client secret: that list is on the logged-in `apiconfig` page, which I could not read. The Electron app registers as "Web" and sends a secret; the core keeps that.
- Whether MAL accepts more than one registered redirect URI, and whether it accepts a 127.0.0.1 URI beyond the fact that the current registration works.
- AniList's burst limiter threshold is undocumented.
- The `X-RateLimit-Limit` header shows 90 in the docs example while the live limit is 30; which value the header reports today I did not test.

## Paths: the directories crate

directories 6.0.0 (2025-01-12) is the newest release; its repository is dirs-dev/directories-rs. https://crates.io/api/v1/crates/directories

`ProjectDirs::from(qualifier, organization, application)` derives a per-platform project path. On Linux the application name is trimmed, lowercased and stripped of spaces (`trim_and_lowercase_then_replace_spaces(application, "")`), so `from("", "", "AniBeam")` and `from("", "", "anibeam")` both give `anibeam`. On macOS the identifier is the non-empty parts of qualifier, organization and application joined with dots after replacing spaces with hyphens, so `from("", "", "anibeam")` gives `anibeam` and `from("com", "Foo Corp", "Bar App")` gives `com.Foo-Corp.Bar-App`. https://raw.githubusercontent.com/dirs-dev/directories-rs/main/src/lin.rs, https://raw.githubusercontent.com/dirs-dev/directories-rs/main/src/mac.rs and https://docs.rs/directories/latest/directories/struct.ProjectDirs.html

Linux results, from the README table:

- `config_dir` and `preference_dir`: `$XDG_CONFIG_HOME/anibeam`, else `$HOME/.config/anibeam`
- `data_dir`: `$XDG_DATA_HOME/anibeam`, else `$HOME/.local/share/anibeam`
- `cache_dir`: `$XDG_CACHE_HOME/anibeam`, else `$HOME/.cache/anibeam`
- `state_dir`: `Some($XDG_STATE_HOME/anibeam)`, else `$HOME/.local/state/anibeam`
- `runtime_dir`: `Some($XDG_RUNTIME_DIR/anibeam)`, `None` when the variable is unset

https://raw.githubusercontent.com/dirs-dev/directories-rs/main/README.md

macOS results: `config_dir` and `data_dir` are `$HOME/Library/Application Support/<id>`, `cache_dir` is `$HOME/Library/Caches/<id>`, `preference_dir` is `$HOME/Library/Preferences/<id>`, and `state_dir` and `runtime_dir` are `None`. https://raw.githubusercontent.com/dirs-dev/directories-rs/main/README.md

The XDG spec defines the roles. `$XDG_STATE_HOME` holds "actions history (logs, history, recently used files, …)" and "current state of the application that can be reused on a restart"; `$XDG_CACHE_HOME` is for "non-essential data files"; `$XDG_RUNTIME_DIR` holds "sockets, named pipes, ...", "Its Unix access mode MUST be 0700" and its lifetime is bound to the login session. Relative values "should consider the path invalid and ignore it". https://specifications.freedesktop.org/basedir/latest/

Electron's `appData` is "`$XDG_CONFIG_HOME` or `~/.config` on Linux" and "`~/Library/Application Support` on macOS", and `userData` "by default is the `appData` directory appended with your app's name". The app name is `anibeam`, so the current tree is `~/.config/anibeam` (config.json, metadata.json, image-cache/, transcode-cache/, franchiseStore.json, logs/) on Linux and `~/Library/Application Support/anibeam` on macOS. https://www.electronjs.org/docs/latest/api/app#appgetpathname and `package.json`

Mapping for the core, with `ProjectDirs::from("", "", "anibeam")`:

- `config_dir` for config: the same directory Electron uses, so the import finds `config.json` and `metadata.json` without a second lookup.
- `data_dir` for the SQLite database and the file fallback for secrets.
- `cache_dir` for the image cache, which is rebuildable.
- `state_dir` for logs; on macOS, where it is `None`, a `logs` folder under `data_dir`.
- `runtime_dir` for the local socket a shell connects over; when `None`, a 0700 folder under `cache_dir`.

The crate "does not create directories or check for their existence", so the core runs `create_dir_all` for each at startup. https://raw.githubusercontent.com/dirs-dev/directories-rs/main/README.md

Alternatives, in one line each: `dirs` is the same author's per-user directory crate with no project scoping; `etcetera` lets the caller pick the XDG or Apple strategy per platform; `xdg` is Linux only. None of them is needed once `directories` gives the XDG split and the macOS mapping in one call.

### Not verified

- directories' MSRV: its manifest carries no `rust-version`.
- Whether the repository is active beyond the January 2025 release; the crate is small and the spec it implements has not changed.
