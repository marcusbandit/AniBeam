mod common;

use std::fs;
use std::path::Path;

use anibeam_core::transfer::format;
use anibeam_core::*;

/// The spec's own example document, version 1 as Electron wrote it.
const FIXTURE: &str = include_str!("fixtures/anibeam-export-v1.json");

fn started(reply: Reply) -> u64 {
    match reply {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

fn summary_of(event: Event) -> ImportSummary {
    match event.body {
        EventBody::ImportFinished { summary } => summary,
        other => panic!("{other:?}"),
    }
}

/// The fixture with its paths moved into the test's own directory, and the
/// library it describes half on disk: Frieren has a file and the film is
/// there, `Some Unmatched Folder` never existed.
fn stage(dir: &Path) -> String {
    let lib = dir.join("anime");
    fs::create_dir_all(lib.join("Sousou no Frieren")).unwrap();
    fs::write(
        lib.join("Sousou no Frieren")
            .join("Sousou no Frieren - 01.mkv"),
        b"x",
    )
    .unwrap();
    fs::create_dir_all(lib.join("Movies")).unwrap();
    fs::write(lib.join("Movies").join("Perfect Blue (1997).mkv"), b"x").unwrap();
    FIXTURE.replace("/mnt/media/anime", lib.to_str().unwrap())
}

fn write_document(dir: &Path, name: &str, text: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}

fn metadata(core: &Core) -> Vec<MetadataRow> {
    match core
        .call(Call::ListMetadata {
            filter: MetadataFilter::All,
            query: String::new(),
            reveal_hidden: true,
        })
        .unwrap()
    {
        Reply::Metadata { rows, .. } => rows,
        other => panic!("{other:?}"),
    }
}

/// One column of the history tables, joined back to the path that owns it,
/// which is the only identity the document has.
fn rows(core: &Core, sql: &str) -> Vec<(String, String)> {
    core.store()
        .read(|c| {
            let mut stmt = c.prepare(sql)?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .unwrap()
}

fn views(core: &Core) -> Vec<(String, String)> {
    rows(
        core,
        "SELECT s.path, v.episode_key FROM views v JOIN series s ON s.id = v.series_id ORDER BY s.path",
    )
}

fn completed(core: &Core) -> Vec<(String, String)> {
    rows(
        core,
        "SELECT s.path, c.episode_key FROM completed c JOIN series s ON s.id = c.series_id ORDER BY c.episode_key",
    )
}

fn resume_points(core: &Core) -> Vec<(String, String)> {
    rows(
        core,
        "SELECT s.path, r.episode_key FROM resume_points r JOIN series s ON s.id = r.series_id ORDER BY r.episode_key",
    )
}

#[test]
fn the_spec_document_imports_and_a_second_import_changes_nothing() {
    let (dir, core, c) = common::open_core();
    let path = write_document(dir.path(), "anibeam-export.json", &stage(dir.path()));

    let job = started(core.call(Call::Import { path: path.clone() }).unwrap());
    let done = common::wait_job(&c, job);
    assert_eq!(done.level, Level::Info);
    assert_eq!(
        done.message,
        "imported: 1 sources, 3 series, 2 matches, 1 views, 1 completed, 2 resume points, 1 accounts, 1 fields ignored"
    );
    let summary = summary_of(done);
    assert_eq!(summary.sources_added, 1);
    assert_eq!(summary.sources_unavailable, 0);
    assert_eq!(summary.series_created, 3);
    assert_eq!(summary.matches_applied, 2);
    assert_eq!(summary.views_merged, 1);
    assert_eq!(summary.completed_merged, 1);
    assert_eq!(summary.resume_points_merged, 2);
    assert_eq!(summary.accounts_imported, 1);
    // There is no TMDB in the native line: the key is read, ignored and
    // named, rather than dropped in silence.
    assert_eq!(summary.fields_ignored, vec!["keys.tmdb".to_string()]);

    // The import speaks for what it touched, so a shell redraws without a
    // call of its own.
    assert!(c.bodies().iter().any(
        |b| matches!(b, EventBody::SourceChanged { source } if source.path.ends_with("anime"))
    ));
    assert!(
        c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::SeriesChanged { series } if series.len() == 3))
    );
    assert!(
        c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::TrackersChanged { .. }))
    );
    assert!(
        c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::PreferencesChanged { .. }))
    );

    let rows = metadata(&core);
    assert_eq!(rows.len(), 3);
    let frieren = rows
        .iter()
        .find(|r| r.series.path.ends_with("Sousou no Frieren"))
        .unwrap();
    assert_eq!(
        frieren.series.match_info,
        Some(MatchInfo {
            provider: Provider::Anilist,
            anilist_id: Some(154587),
            mal_id: Some(52991),
            tmdb_id: None,
            tmdb_kind: None,
            confirmed: true
        })
    );
    assert!(!frieren.series.missing && !frieren.series.hidden);

    let film = rows
        .iter()
        .find(|r| r.series.kind == SeriesKind::Movie)
        .unwrap();
    assert!(film.series.path.ends_with("Perfect Blue (1997).mkv"));
    assert_eq!(
        film.series.match_info,
        Some(MatchInfo {
            provider: Provider::Tmdb,
            anilist_id: None,
            mal_id: None,
            tmdb_id: Some(10494),
            tmdb_kind: Some(TmdbKind::Movie),
            confirmed: true
        })
    );

    // A path that was never there is the same missing state a scan leaves,
    // so the same scan attaches its files later.
    let unmatched = rows
        .iter()
        .find(|r| r.series.path.ends_with("Some Unmatched Folder"))
        .unwrap();
    assert!(
        unmatched.series.hidden
            && unmatched.series.missing
            && unmatched.series.match_info.is_none()
    );

    match core.call(Call::GetTrackers).unwrap() {
        Reply::Trackers { state } => {
            assert!(state.anilist.connected);
            assert_eq!(state.anilist.username.as_deref(), Some("bandit"));
            assert_eq!(state.anilist.client_id, "12345");
            assert_eq!(state.main, Tracker::Anilist);
            assert!(!state.mal.connected);
        }
        other => panic!("{other:?}"),
    }

    assert_eq!(views(&core).len(), 1);
    assert_eq!(views(&core)[0].1, "12");
    assert_eq!(completed(&core)[0].1, "12");
    // One resume point by series and episode, one by the file it belongs
    // to, which lands on the series that owns the file under its own name.
    let points = resume_points(&core);
    assert_eq!(points.len(), 2);
    assert_eq!(points[0].1, "13");
    assert!(points[0].0.ends_with("Sousou no Frieren"));
    assert_eq!(points[1].1, "NCOP1.mkv");
    assert!(points[1].0.ends_with("Sousou no Frieren"));

    match core.call(Call::GetPreferences).unwrap() {
        Reply::Preferences { preferences } => assert_eq!(
            preferences,
            Preferences {
                title_language: TitleLanguage::Romaji,
                library_tab: Tab::All,
                library_sort: Sort::Alpha,
                library_direction: Direction::Asc,
                feed_sort: FeedSort::Recent,
            }
        ),
        other => panic!("{other:?}"),
    }

    // Merge, file wins, nothing deleted: the same file again is no change
    // at all, and it says so.
    let job = started(core.call(Call::Import { path }).unwrap());
    let again = summary_of(common::wait_job(&c, job));
    assert_eq!(again.sources_added, 0);
    assert_eq!(again.sources_unavailable, 0);
    assert_eq!(again.series_created, 0);
    assert_eq!(again.matches_applied, 0);
    assert_eq!(again.views_merged, 0);
    assert_eq!(again.completed_merged, 0);
    assert_eq!(again.resume_points_merged, 0);
    assert_eq!(again.accounts_imported, 0);
    assert_eq!(again.fields_ignored, vec!["keys.tmdb".to_string()]);
    assert_eq!(metadata(&core).len(), 3);
}

#[test]
fn a_private_export_round_trips_into_a_fresh_core() {
    let (dir, core, c) = common::open_core();
    let path = write_document(dir.path(), "anibeam-export.json", &stage(dir.path()));
    let job = started(core.call(Call::Import { path }).unwrap());
    common::wait_job(&c, job);

    // The two things version 2 added: a track choice, which describes the
    // files rather than the match, and the auto-skip toggles.
    let frieren = metadata(&core)
        .into_iter()
        .find(|r| r.series.path.ends_with("Sousou no Frieren"))
        .unwrap()
        .series
        .id;
    let audio = TrackRef {
        kind: TrackKind::Embedded,
        language: Some("ja".to_string()),
        title: None,
    };
    core.call(Call::SetTrackChoice {
        series: frieren,
        audio: Some(audio.clone()),
        subtitle: Some(SubtitleChoice::Off),
    })
    .unwrap();
    let preferences = Preferences {
        title_language: TitleLanguage::English,
        library_tab: Tab::Movies,
        library_sort: Sort::MyScore,
        library_direction: Direction::Desc,
        feed_sort: FeedSort::Upcoming,
    };
    core.call(Call::SetPreferences {
        preferences: preferences.clone(),
    })
    .unwrap();
    core.call(Call::SetAutoSkip {
        intro: true,
        outro: false,
    })
    .unwrap();

    let out = dir.path().join("anibeam-export-full.json");
    let out_path = out.to_string_lossy().into_owned();
    let job = started(
        core.call(Call::Export {
            path: out_path.clone(),
            private: true,
        })
        .unwrap(),
    );
    let done = common::wait_job(&c, job);
    assert_eq!(done.level, Level::Info);
    assert!(
        matches!(&done.body, EventBody::ExportFinished { path } if *path == out_path),
        "{done:?}"
    );
    assert!(!dir.path().join("anibeam-export-full.json.tmp").exists());

    let (doc, ignored) = format::parse(&fs::read(&out).unwrap()).unwrap();
    assert!(ignored.is_empty(), "{ignored:?}");
    assert_eq!((doc.format.as_str(), doc.version), (format::FORMAT, 2));
    assert_eq!(doc.exported_by.line, "native");
    assert_eq!(doc.exported_by.app, "anibeam");
    assert!(doc.private);
    assert!(doc.exported_at.ends_with('Z'), "{}", doc.exported_at);
    assert_eq!(doc.series.len(), 3);
    assert_eq!(
        doc.series[0].track_choice,
        Some(TrackChoice {
            audio: Some(audio),
            subtitle: Some(SubtitleChoice::Off)
        })
    );
    // No TMDB key to carry, so nothing here is ignored on the way back in.
    assert_eq!(doc.keys, Some(serde_json::json!({})));
    let prefs = doc.preferences.as_ref().unwrap();
    assert_eq!(prefs.title_language, "english");
    assert_eq!(
        prefs.auto_skip,
        Some(AutoSkip {
            intro: true,
            outro: false
        })
    );
    assert_eq!(
        doc.accounts
            .as_ref()
            .unwrap()
            .anilist
            .as_ref()
            .unwrap()
            .access_token,
        "eyJ..."
    );
    assert!(doc.accounts.as_ref().unwrap().mal.is_none());

    // The checkbox is the only guard there is, so the unticked export has
    // to carry none of it.
    let library = dir.path().join("anibeam-export-library.json");
    let job = started(
        core.call(Call::Export {
            path: library.to_string_lossy().into_owned(),
            private: false,
        })
        .unwrap(),
    );
    common::wait_job(&c, job);
    let (library, _) = format::parse(&fs::read(&library).unwrap()).unwrap();
    assert!(!library.private);
    assert_eq!(library.series.len(), 3);
    assert!(
        library.accounts.is_none()
            && library.keys.is_none()
            && library.history.is_none()
            && library.preferences.is_none()
    );

    let (_dir2, fresh, c2) = common::open_core();
    let job = started(fresh.call(Call::Import { path: out_path }).unwrap());
    let summary = summary_of(common::wait_job(&c2, job));
    assert_eq!(summary.sources_added, 1);
    assert_eq!(summary.series_created, 3);
    assert_eq!(summary.matches_applied, 2);
    assert_eq!(summary.views_merged, 1);
    assert_eq!(summary.completed_merged, 1);
    assert_eq!(summary.resume_points_merged, 2);
    assert_eq!(summary.accounts_imported, 1);
    assert!(
        summary.fields_ignored.is_empty(),
        "{:?}",
        summary.fields_ignored
    );

    assert_eq!(
        metadata(&fresh)
            .iter()
            .map(|r| r.series.path.clone())
            .collect::<Vec<_>>(),
        metadata(&core)
            .iter()
            .map(|r| r.series.path.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(views(&fresh), views(&core));
    assert_eq!(completed(&fresh), completed(&core));
    assert_eq!(resume_points(&fresh), resume_points(&core));
    assert!(
        matches!(fresh.call(Call::GetPreferences).unwrap(), Reply::Preferences { preferences: p } if p == preferences)
    );
    assert!(
        matches!(fresh.call(Call::GetSettings).unwrap(), Reply::Settings { settings } if settings.auto_skip == AutoSkip { intro: true, outro: false })
    );
    assert!(
        matches!(fresh.call(Call::GetTrackers).unwrap(), Reply::Trackers { state } if state.anilist.connected)
    );
}

#[test]
fn a_newer_document_and_an_unreadable_one_fail_at_once() {
    let (dir, core, _c) = common::open_core();
    let newer = write_document(
        dir.path(),
        "v3.json",
        &FIXTURE.replace("\"version\": 1", "\"version\": 3"),
    );
    assert!(matches!(
        core.call(Call::Import { path: newer }),
        Err(CoreError::Version {
            found: 3,
            supported: 2
        })
    ));

    let missing = dir
        .path()
        .join("not-here.json")
        .to_string_lossy()
        .into_owned();
    assert!(
        matches!(core.call(Call::Import { path: missing }), Err(CoreError::Invalid { field, .. }) if field == "path")
    );
}
