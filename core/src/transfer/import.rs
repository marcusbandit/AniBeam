//! The Import job: an `anibeam-export` document merged into the tables,
//! the file winning.
//!
//! Four rules shape every section below. Nothing is deleted, so a record
//! only in the library stays. The file wins on matches, flags, accounts
//! and preferences. History keeps whichever side is newer, so a re-import
//! never rewinds. And the same file twice reports zero changes, which is
//! why every counter here counts a write that changed a row rather than an
//! entry that was read.
//!
//! One transaction per section. A section that fails leaves the ones
//! before it committed and fails the job with its own error, rather than
//! rolling a whole library back over one bad entry.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::library::cards;
use crate::library::classifier;
use crate::library::scan;
use crate::metadata::fetch;
use crate::metadata::record;
use crate::prefs;
use crate::time;
use crate::trackers::accounts::{self, access_key, refresh_key, secret_key};
use crate::trackers::secrets::StoreKind;
use crate::transfer::format::{
    self, Account, Document, MatchEntry, ResumeEntry, SeriesEntry, TrackerName,
};
use crate::transfer::{file_name, normalise, under};

/// The five sections a `JobProgress` counts through.
const SECTIONS: u64 = 5;

/// Reads and parses on the calling thread, so a file this core is too old
/// to read is `Err(Version)` before any job starts and an unreadable path
/// is refused the same way. Everything after that is the job's.
pub fn start(core: &Core, path: &str) -> Result<u64, CoreError> {
    // The spec lists no synchronous `Io` for this call, and the argument
    // that failed is the path, so an unreadable file is refused as one.
    let bytes = std::fs::read(path).map_err(|e| CoreError::Invalid {
        field: "path".to_string(),
        message: e.to_string(),
    })?;
    let (doc, ignored) = format::parse(&bytes)?;
    let owner = core
        .arc()
        .ok_or_else(|| CoreError::internal("core is shutting down"))?;
    let started = owner.clone();
    Ok(started
        .jobs
        .clone()
        .start(JobKind::Import, move |ctx| async move {
            run(owner, ctx, doc, ignored).await
        }))
}

async fn run(
    core: Arc<Core>,
    ctx: Arc<JobCtx>,
    doc: Document,
    ignored: Vec<String>,
) -> Result<Finished, CoreError> {
    let Document {
        sources,
        series,
        accounts,
        keys,
        history,
        preferences,
        ..
    } = doc;
    let mut summary = ImportSummary {
        sources_added: 0,
        sources_unavailable: 0,
        series_created: 0,
        matches_applied: 0,
        views_merged: 0,
        completed_merged: 0,
        resume_points_merged: 0,
        accounts_imported: 0,
        fields_ignored: ignored,
    };

    ctx.progress(0, Some(SECTIONS), "sources");
    let now = time::now_secs();
    let sources = import_sources(&core, sources, now, &mut summary).await?;

    ctx.progress(1, Some(SECTIONS), "series");
    let touched = import_series(&core, series, now, &mut summary).await?;

    ctx.progress(2, Some(SECTIONS), "history");
    if let Some(history) = history {
        import_history(&core, history, &mut summary).await?;
    }

    ctx.progress(3, Some(SECTIONS), "accounts");
    // There is no TMDB in the native line, so its key is read, named and
    // dropped rather than stored.
    if let Some(keys) = keys.as_ref().and_then(|k| k.as_object()) {
        for name in keys.keys() {
            summary.fields_ignored.push(format!("keys.{name}"));
        }
    }
    let imported_accounts = accounts.is_some();
    if let Some(accounts) = accounts {
        import_accounts(&core, accounts, now, &mut summary).await?;
    }

    ctx.progress(4, Some(SECTIONS), "preferences");
    let imported_preferences = match preferences {
        Some(entry) => Some(import_preferences(&core, entry).await?),
        None => None,
    };
    ctx.progress(SECTIONS, Some(SECTIONS), "done");

    // What changed, in the order a shell wants to redraw it. The scan the
    // launch runs is what attaches files to the series this created; the
    // import itself starts none.
    for id in sources {
        let state = core.clone();
        if let Some(source) = core
            .store
            .write_async(move |c| scan::load_source(c, &state.library, id))
            .await?
        {
            ctx.emit(
                Level::Info,
                format!("source imported: {}", source.path),
                EventBody::SourceChanged { source },
            );
        }
    }
    if !touched.is_empty() {
        let images_dir = core.paths.images_dir();
        let cards = core
            .store
            .write_async(move |c| cards::cards_for(c, &images_dir, &touched))
            .await?;
        ctx.changed_all(cards);
    }
    if imported_accounts {
        let state = accounts::state_async(&core).await?;
        ctx.emit(
            Level::Debug,
            "trackers changed",
            EventBody::TrackersChanged { state },
        );
    }
    if let Some(preferences) = imported_preferences {
        ctx.emit(
            Level::Debug,
            "preferences changed",
            EventBody::PreferencesChanged { preferences },
        );
    }
    sweep_images(&core).await;

    let message = format!(
        "imported: {} sources, {} series, {} matches, {} views, {} completed, {} resume points, {} accounts, {} fields ignored",
        summary.sources_added,
        summary.series_created,
        summary.matches_applied,
        summary.views_merged,
        summary.completed_merged,
        summary.resume_points_merged,
        summary.accounts_imported,
        summary.fields_ignored.len(),
    );
    Ok(Finished {
        level: Level::Info,
        message,
        body: EventBody::ImportFinished { summary },
    })
}

// Sources --------------------------------------------------------------------

/// A source whose path is not there right now is stored unavailable, the
/// same state as an unplugged drive: its series stay dormant and attach
/// when the path comes back.
async fn import_sources(
    core: &Arc<Core>,
    entries: Vec<format::SourceEntry>,
    now: i64,
    summary: &mut ImportSummary,
) -> Result<Vec<u64>, CoreError> {
    let wanted: Vec<(String, bool)> = entries
        .into_iter()
        .map(|e| {
            let path = normalise(&e.path);
            let available = Path::new(&path).is_dir();
            (path, available)
        })
        .collect();
    let (ids, added, unavailable) = core
        .store
        .tx_async(move |tx| {
            let (mut ids, mut added, mut unavailable) = (Vec::new(), 0u64, 0u64);
            for (path, available) in &wanted {
                let existing: Option<(i64, i64)> = tx
                    .query_row(
                        "SELECT id, available FROM sources WHERE path = ?1",
                        params![path],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .optional()?;
                let (id, changed) = match existing {
                    Some((id, was)) => {
                        let changed = (was == 1) != *available;
                        if changed {
                            tx.execute(
                                "UPDATE sources SET available = ?2 WHERE id = ?1",
                                params![id, i64::from(*available)],
                            )?;
                        }
                        (id as u64, changed)
                    }
                    None => {
                        tx.execute(
                            "INSERT INTO sources (path, available, added_at) VALUES (?1, ?2, ?3)",
                            params![path, i64::from(*available), now],
                        )?;
                        added += 1;
                        (tx.last_insert_rowid() as u64, true)
                    }
                };
                // Counted when this import is what put the source in that
                // state, so the same file twice reports nothing.
                if changed && !*available {
                    unavailable += 1;
                }
                ids.push(id);
            }
            Ok((ids, added, unavailable))
        })
        .await?;
    summary.sources_added = added;
    summary.sources_unavailable = unavailable;
    Ok(ids)
}

// Series and matches ---------------------------------------------------------

/// A series whose path is unknown is created anyway, so its match, its
/// flag and its history have a home; the scanner fills in the files when
/// the path appears.
async fn import_series(
    core: &Arc<Core>,
    entries: Vec<SeriesEntry>,
    now: i64,
    summary: &mut ImportSummary,
) -> Result<Vec<u64>, CoreError> {
    let (touched, created, matched, ignored) = core
        .store
        .tx_async(move |tx| {
            // Every source, not only the file's: a series may well belong
            // to one this library already had.
            let sources = all_sources(tx)?;
            let (mut touched, mut created, mut matched, mut ignored) = (Vec::new(), 0u64, 0u64, Vec::new());
            for (n, entry) in entries.iter().enumerate() {
                let path = normalise(&entry.path);
                let kind = if entry.kind == "movie" { SeriesKind::Movie } else { SeriesKind::Show };
                // `series.source_id` is NOT NULL and the longest prefix is
                // the source that owns the path. An entry under none of
                // them has nowhere to live, so it is skipped and named.
                let Some(source_id) = owning_source(&sources, &path) else {
                    ignored.push(format!("series[{n}].path"));
                    continue;
                };
                let title = if entry.title.trim().is_empty() { file_name(&path) } else { entry.title.clone() };
                let existing: Option<i64> = tx
                    .query_row("SELECT id FROM series WHERE kind = ?1 AND path = ?2", params![kind.as_str(), path], |r| r.get(0))
                    .optional()?;
                let id = match existing {
                    // The file wins on the flag; the row keeps the missing
                    // state it already has, since only a scan can say the
                    // path came back.
                    Some(id) => {
                        tx.execute("UPDATE series SET hidden = ?2 WHERE id = ?1", params![id, i64::from(entry.hidden)])?;
                        id as u64
                    }
                    None => {
                        // The same missing state a scan produces, so the
                        // same scan attaches the files later.
                        let missing_since = (!Path::new(&path).exists()).then_some(now);
                        tx.execute(
                            "INSERT INTO series (source_id, kind, path, folder_name, hidden, missing_since, added_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            params![source_id as i64, kind.as_str(), path, title, i64::from(entry.hidden), missing_since, now],
                        )?;
                        created += 1;
                        tx.last_insert_rowid() as u64
                    }
                };
                if let Some(choice) = &entry.track_choice {
                    let json = serde_json::to_string(choice)?;
                    tx.execute("UPDATE series SET track_choice = ?2 WHERE id = ?1", params![id as i64, json])?;
                }
                if let Some(m) = &entry.match_
                    && apply_match(tx, id, m, now)?
                {
                    matched += 1;
                }
                touched.push(id);
            }
            Ok((touched, created, matched, ignored))
        })
        .await?;
    summary.series_created = created;
    summary.matches_applied = matched;
    summary.fields_ignored.extend(ignored);
    Ok(touched)
}

fn all_sources(conn: &Connection) -> Result<Vec<(u64, String)>, CoreError> {
    let mut stmt = conn.prepare("SELECT id, path FROM sources ORDER BY id")?;
    let rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, i64>(0)? as u64, r.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// The source that contains this path, longest prefix first, so a source
/// nested inside another still claims what is closest to it.
fn owning_source(sources: &[(u64, String)], path: &str) -> Option<u64> {
    sources
        .iter()
        .filter(|(_, root)| under(path, root))
        .max_by_key(|(_, root)| root.len())
        .map(|(id, _)| *id)
}

/// The match columns, written only when they differ from what the row
/// already carries: an imported match is user-confirmed, and a second
/// import of the same file must change nothing at all. Answers whether it
/// wrote.
fn apply_match(
    tx: &Transaction,
    series: u64,
    entry: &MatchEntry,
    now: i64,
) -> Result<bool, CoreError> {
    let wanted = match entry {
        MatchEntry::Tracker {
            provider,
            anilist_id,
            mal_id,
        } => {
            // A tracker match with neither id is nothing to write. The
            // declared provider stands unless it has no id behind it.
            let provider = match (provider, anilist_id, mal_id) {
                (_, None, None) => return Ok(false),
                (TrackerName::Anilist, None, Some(_)) => Provider::Mal,
                (TrackerName::Anilist, _, _) => Provider::Anilist,
                (TrackerName::Mal, _, _) => Provider::Mal,
            };
            Wanted {
                provider,
                anilist_id: *anilist_id,
                mal_id: *mal_id,
                tmdb_id: None,
                tmdb_kind: None,
            }
        }
        MatchEntry::Tmdb {
            tmdb_id, tmdb_kind, ..
        } => Wanted {
            provider: Provider::Tmdb,
            anilist_id: None,
            mal_id: None,
            tmdb_id: Some(*tmdb_id),
            tmdb_kind: Some(tmdb_kind.clone()),
        },
    };
    if current_match(tx, series)? == Some(wanted.clone()) {
        return Ok(false);
    }
    match wanted.provider {
        // TMDB is carried and never fetched: two columns, a confirmed
        // match and nothing behind it.
        Provider::Tmdb => {
            tx.execute(
                "UPDATE series SET provider = 'tmdb', anilist_id = NULL, mal_id = NULL, tmdb_id = ?2, tmdb_kind = ?3,
                        confirmed = 1, matched_at = ?4, attempted_at = ?4
                 WHERE id = ?1",
                params![series as i64, wanted.tmdb_id.map(|id| id as i64), wanted.tmdb_kind, now],
            )?;
        }
        provider => {
            // `series.anilist_id` is a foreign key, so the media row has
            // to exist before the column can point at one. A stub is the
            // honest shape: the id is known and nothing else is yet, and
            // the backfill is what comes for it.
            if let Some(id) = wanted.anilist_id {
                record::write_stub(
                    tx,
                    &record::StubWrite {
                        id,
                        mal_id: wanted.mal_id,
                        ..Default::default()
                    },
                )?;
            }
            fetch::write_match_only(
                tx,
                series,
                provider,
                wanted.anilist_id,
                wanted.mal_id,
                true,
                now,
            )?;
        }
    }
    Ok(true)
}

/// The match columns as a value, so "did this change anything" is one
/// comparison rather than five.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Wanted {
    provider: Provider,
    anilist_id: Option<u64>,
    mal_id: Option<u64>,
    tmdb_id: Option<u64>,
    tmdb_kind: Option<String>,
}

fn current_match(tx: &Transaction, series: u64) -> Result<Option<Wanted>, CoreError> {
    let row = tx
        .query_row(
            "SELECT provider, anilist_id, mal_id, tmdb_id, tmdb_kind, confirmed FROM series WHERE id = ?1",
            params![series as i64],
            |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?,
                    r.get::<_, Option<i64>>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, i64>(5)?,
                ))
            },
        )
        .optional()?;
    let Some((provider, anilist_id, mal_id, tmdb_id, tmdb_kind, confirmed)) = row else {
        return Ok(None);
    };
    // An unconfirmed match is one the auto-match wrote, and the file's is
    // the user's own, so it is always worth writing over.
    if confirmed != 1 {
        return Ok(None);
    }
    let Some(provider) = provider.as_deref().and_then(Provider::from_column) else {
        return Ok(None);
    };
    Ok(Some(Wanted {
        provider,
        anilist_id: anilist_id.map(|v| v as u64),
        mal_id: mal_id.map(|v| v as u64),
        tmdb_id: tmdb_id.map(|v| v as u64),
        tmdb_kind,
    }))
}

// History --------------------------------------------------------------------

/// Views, completed episodes and resume points, all keyed by the series'
/// path and the episode's number. The newer `at` wins on every one of
/// them, so a re-import never rewinds what the library already knows.
async fn import_history(
    core: &Arc<Core>,
    history: format::History,
    summary: &mut ImportSummary,
) -> Result<(), CoreError> {
    let (views, completed, resume) = core
        .store
        .tx_async(move |tx| {
            let series = series_by_path(tx)?;
            let mut views = 0u64;
            for view in &history.views {
                let (Some(row), Some(at)) = (series.get(&normalise(&view.series)), format::parse_instant(&view.at)) else { continue };
                views += tx.execute(
                    "INSERT INTO views (series_id, episode_key, at) VALUES (?1, ?2, ?3)
                     ON CONFLICT(series_id) DO UPDATE SET episode_key = excluded.episode_key, at = excluded.at
                     WHERE excluded.at > views.at",
                    params![row.id as i64, row.key(view.last_episode), at],
                )? as u64;
            }
            let mut completed = 0u64;
            for entry in &history.completed {
                let (Some(row), Some(at)) = (series.get(&normalise(&entry.series)), format::parse_instant(&entry.at)) else { continue };
                completed += tx.execute(
                    "INSERT INTO completed (series_id, episode_key, at) VALUES (?1, ?2, ?3)
                     ON CONFLICT(series_id, episode_key) DO UPDATE SET at = excluded.at WHERE excluded.at > completed.at",
                    params![row.id as i64, row.key(entry.episode), at],
                )? as u64;
            }
            let mut resume = 0u64;
            for entry in &history.resume_points {
                let Some((id, key, position, duration, at)) = resume_target(&series, entry) else { continue };
                resume += tx.execute(
                    "INSERT INTO resume_points (series_id, episode_key, position, duration, at) VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(series_id, episode_key) DO UPDATE SET position = excluded.position, duration = excluded.duration,
                            at = excluded.at
                     WHERE excluded.at > resume_points.at",
                    params![id as i64, key, position, duration, at],
                )? as u64;
            }
            Ok((views, completed, resume))
        })
        .await?;
    summary.views_merged = views;
    summary.completed_merged = completed;
    summary.resume_points_merged = resume;
    Ok(())
}

/// A series as the history sections need it: its id, and how it keys an
/// episode.
struct HistorySeries {
    id: u64,
    kind: SeriesKind,
    path: String,
}

impl HistorySeries {
    /// A film has no episode number, so its history key is its file name,
    /// which is the last segment of its own path. Everything else keys by
    /// the number, `12` or `12.5`, never `12.0`.
    fn key(&self, number: f64) -> String {
        match self.kind {
            SeriesKind::Movie => file_name(&self.path),
            SeriesKind::Show => classifier::format_number(number),
        }
    }
}

fn series_by_path(conn: &Connection) -> Result<HashMap<String, HistorySeries>, CoreError> {
    let mut stmt = conn.prepare("SELECT id, kind, path FROM series ORDER BY id")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(HistorySeries {
                id: r.get::<_, i64>(0)? as u64,
                kind: SeriesKind::from_column(&r.get::<_, String>(1)?).unwrap_or(SeriesKind::Show),
                path: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows.into_iter().map(|r| (r.path.clone(), r)).collect())
}

/// Where a resume point lands: on the series and episode it names, or on
/// the series that owns the file, its folder for a show and the file
/// itself for a film, with the file name as the key.
fn resume_target(
    series: &HashMap<String, HistorySeries>,
    entry: &ResumeEntry,
) -> Option<(u64, String, f64, f64, i64)> {
    match entry {
        ResumeEntry::Series {
            series: path,
            episode,
            position,
            duration,
            at,
        } => {
            let row = series.get(&normalise(path))?;
            Some((
                row.id,
                row.key(*episode),
                *position,
                *duration,
                format::parse_instant(at)?,
            ))
        }
        ResumeEntry::File {
            file,
            position,
            duration,
            at,
        } => {
            let file = normalise(file);
            let row = series
                .values()
                .filter(|r| under(&file, &r.path))
                .max_by_key(|r| r.path.len())?;
            Some((
                row.id,
                file_name(&file),
                *position,
                *duration,
                format::parse_instant(at)?,
            ))
        }
    }
}

// Accounts -------------------------------------------------------------------

/// Tokens to the keyring or its file fallback, the public half to the row,
/// and the main provider to the settings. The keyring writes happen before
/// the transaction: a keyring that will not take a token should fail the
/// section before it has changed a row.
async fn import_accounts(
    core: &Arc<Core>,
    accounts: format::Accounts,
    now: i64,
    summary: &mut ImportSummary,
) -> Result<(), CoreError> {
    let main = Tracker::from_column(&accounts.main);
    let mut imported = 0u64;
    let mut rows: Vec<(Tracker, Account, Option<StoreKind>)> = Vec::new();
    for (t, account) in [
        (Tracker::Anilist, accounts.anilist),
        (Tracker::Mal, accounts.mal),
    ] {
        let Some(account) = account else { continue };
        let (store, wrote) = write_secrets(core, t, &account).await?;
        if wrote {
            imported += 1;
        }
        rows.push((t, account, store));
    }

    core.store
        .tx_async(move |tx| {
            for (t, account, store) in &rows {
                // `connected_at` is what says an account is connected, so
                // it is set when there is a token to connect with. A write
                // that carried none leaves whatever is already there.
                let connected_at = (!account.access_token.is_empty()).then_some(now);
                tx.execute(
                    "INSERT INTO tracker_accounts (tracker, user_id, username, client_id, expires_at, connected_at, secret_store)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(tracker) DO UPDATE SET user_id = excluded.user_id, username = excluded.username,
                            client_id = excluded.client_id, expires_at = excluded.expires_at,
                            connected_at = coalesce(excluded.connected_at, tracker_accounts.connected_at),
                            secret_store = coalesce(excluded.secret_store, tracker_accounts.secret_store)",
                    params![
                        t.as_str(),
                        account.user_id.map(|id| id as i64),
                        account.username,
                        account.client_id,
                        account.expires_at.as_ref().and_then(format::parse_instant),
                        connected_at,
                        store.map(StoreKind::as_str),
                    ],
                )?;
            }
            if let Some(main) = main {
                prefs::save_main_tracker(tx, main)?;
            }
            Ok(())
        })
        .await?;
    summary.accounts_imported = imported;
    Ok(())
}

/// The three secrets an account carries, each written only when it differs
/// from what the store already holds. Answers where the access token went,
/// for the row's `secret_store`, and whether it was written at all, which
/// is what `accounts_imported` counts.
///
/// Every call goes through the blocking pool: the keyring is synchronous
/// zbus underneath and a tokio worker must never block on it.
async fn write_secrets(
    core: &Arc<Core>,
    t: Tracker,
    account: &Account,
) -> Result<(Option<StoreKind>, bool), CoreError> {
    let hint = core
        .store
        .write_async(move |c| accounts::load_row(c, t))
        .await?
        .and_then(|r| r.secret_store);
    let mut store = None;
    let mut wrote = false;
    if !account.access_token.is_empty() {
        let token = account.access_token.clone();
        if let Some(kind) = write_secret(core, access_key(t), token, hint).await? {
            store = Some(kind);
            wrote = true;
        }
    }
    for (key, value) in [
        (refresh_key(t), account.refresh_token.clone()),
        (secret_key(t), account.client_secret.clone()),
    ] {
        let Some(value) = value.filter(|v| !v.is_empty()) else {
            continue;
        };
        write_secret(core, key, value, hint).await?;
    }
    Ok((store, wrote))
}

/// One secret, written only when the store holds something else. `None`
/// when it already held exactly this, so nothing is counted as imported.
async fn write_secret(
    core: &Arc<Core>,
    key: String,
    value: String,
    hint: Option<StoreKind>,
) -> Result<Option<StoreKind>, CoreError> {
    let read = key.clone();
    let held = accounts::with_secrets(core, move |s| s.get(&read, hint)).await?;
    if held.as_ref().map(|(v, _)| v.as_str()) == Some(value.as_str()) {
        return Ok(None);
    }
    accounts::with_secrets(core, move |s| s.set(&key, &value))
        .await
        .map(Some)
}

// Preferences ----------------------------------------------------------------

/// The library and feed view state, and the two auto-skip toggles with it.
/// A value this core does not know keeps whatever the library had, rather
/// than resetting the whole record over one word.
async fn import_preferences(
    core: &Arc<Core>,
    entry: format::PrefsEntry,
) -> Result<Preferences, CoreError> {
    core.store
        .tx_async(move |tx| {
            let current = prefs::load_preferences(tx)?;
            let preferences = Preferences {
                title_language: TitleLanguage::from_column(&entry.title_language)
                    .unwrap_or(current.title_language),
                library_tab: Tab::from_column(&entry.library_tab).unwrap_or(current.library_tab),
                library_sort: Sort::from_column(&entry.library_sort.key)
                    .unwrap_or(current.library_sort),
                library_direction: Direction::from_column(&entry.library_sort.direction)
                    .unwrap_or(current.library_direction),
                feed_sort: FeedSort::from_column(&entry.feed_sort).unwrap_or(current.feed_sort),
            };
            prefs::save_preferences(tx, &preferences)?;
            if let Some(auto_skip) = &entry.auto_skip {
                prefs::save_auto_skip(tx, auto_skip)?;
            }
            Ok(preferences)
        })
        .await
}

/// An import brings in matches whose images the jobs will fetch; the sweep
/// is what keeps the directory from only ever growing. Bookkeeping, so its
/// report goes to the trace log rather than the activity log.
async fn sweep_images(core: &Arc<Core>) {
    let cache = core.images.clone();
    let now = time::now_secs();
    match core.store.write_async(move |c| cache.sweep(c, now)).await {
        Ok(report) => tracing::debug!(
            "image sweep after an import: {} rows removed, {} files removed",
            report.removed_rows,
            report.removed_files
        ),
        Err(e) => tracing::debug!("the image sweep after an import failed: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources() -> Vec<(u64, String)> {
        vec![
            (1, "/lib".to_string()),
            (2, "/lib/anime".to_string()),
            (3, "/other".to_string()),
        ]
    }

    /// Longest prefix wins, so a source nested inside another claims what
    /// is closest to it, and a path under none of them has no owner.
    #[test]
    fn a_series_belongs_to_the_source_closest_above_it() {
        assert_eq!(owning_source(&sources(), "/lib/anime/Show"), Some(2));
        assert_eq!(owning_source(&sources(), "/lib/Show"), Some(1));
        assert_eq!(owning_source(&sources(), "/elsewhere/Show"), None);
        assert_eq!(owning_source(&sources(), "/lib2/Show"), None);
    }

    #[test]
    fn a_film_keys_its_history_by_its_file_name() {
        let film = HistorySeries {
            id: 1,
            kind: SeriesKind::Movie,
            path: "/lib/Movies/Film (2001).mkv".to_string(),
        };
        assert_eq!(film.key(0.0), "Film (2001).mkv");
        let show = HistorySeries {
            id: 2,
            kind: SeriesKind::Show,
            path: "/lib/Show".to_string(),
        };
        assert_eq!(show.key(12.0), "12");
        assert_eq!(show.key(12.5), "12.5");
    }

    /// A resume point keyed by file lands on the series that owns the
    /// file: its folder for a show, the file itself for a film.
    #[test]
    fn a_resume_point_by_file_finds_the_series_that_owns_it() {
        let mut series = HashMap::new();
        series.insert(
            "/lib/Show".to_string(),
            HistorySeries {
                id: 2,
                kind: SeriesKind::Show,
                path: "/lib/Show".to_string(),
            },
        );
        series.insert(
            "/lib/Movies/Film.mkv".to_string(),
            HistorySeries {
                id: 3,
                kind: SeriesKind::Movie,
                path: "/lib/Movies/Film.mkv".to_string(),
            },
        );
        let at = serde_json::Value::from("1970-01-01T00:00:10Z");
        let by_file = |file: &str| ResumeEntry::File {
            file: file.to_string(),
            position: 1.0,
            duration: 2.0,
            at: at.clone(),
        };

        let (id, key, ..) = resume_target(&series, &by_file("/lib/Show/NCOP1.mkv")).unwrap();
        assert_eq!((id, key.as_str()), (2, "NCOP1.mkv"));
        let (id, key, ..) = resume_target(&series, &by_file("/lib/Movies/Film.mkv")).unwrap();
        assert_eq!((id, key.as_str()), (3, "Film.mkv"));
        assert!(resume_target(&series, &by_file("/elsewhere/x.mkv")).is_none());

        // And one keyed by series and episode lands on the number.
        let by_series = ResumeEntry::Series {
            series: "/lib/Show".to_string(),
            episode: 13.0,
            position: 1.0,
            duration: 2.0,
            at,
        };
        let (id, key, ..) = resume_target(&series, &by_series).unwrap();
        assert_eq!((id, key.as_str()), (2, "13"));
    }
}
