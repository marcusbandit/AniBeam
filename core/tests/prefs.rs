mod common;
use anibeam_core::*;

#[test]
fn preferences_and_settings_round_trip_with_events() {
    let (_dir, core, c) = common::open_core();
    assert!(
        matches!(core.call(Call::GetPreferences).unwrap(), Reply::Preferences { preferences } if preferences == Preferences::default())
    );
    let prefs = Preferences {
        title_language: TitleLanguage::English,
        library_tab: Tab::Movies,
        library_sort: Sort::MyScore,
        library_direction: Direction::Desc,
        feed_sort: FeedSort::Upcoming,
        library_show_watched: true,
    };
    assert!(matches!(
        core.call(Call::SetPreferences {
            preferences: prefs.clone()
        })
        .unwrap(),
        Reply::Ok
    ));
    assert!(
        matches!(core.call(Call::GetPreferences).unwrap(), Reply::Preferences { preferences } if preferences == prefs)
    );
    assert!(
        c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::PreferencesChanged { .. }))
    );

    assert!(
        matches!(core.call(Call::GetSettings).unwrap(), Reply::Settings { settings } if settings == Settings::default())
    );
    let mut defaults = SubtitleDefaults {
        scale: 1.5,
        ..Default::default()
    };
    assert!(matches!(
        core.call(Call::SetSubtitleDefaults {
            defaults: defaults.clone()
        })
        .unwrap(),
        Reply::Ok
    ));
    assert!(matches!(
        core.call(Call::SetAutoSkip {
            intro: true,
            outro: false
        })
        .unwrap(),
        Reply::Ok
    ));
    match core.call(Call::GetSettings).unwrap() {
        Reply::Settings { settings } => {
            assert_eq!(settings.subtitle_defaults, defaults);
            assert_eq!(
                settings.auto_skip,
                AutoSkip {
                    intro: true,
                    outro: false
                }
            );
            assert_eq!(settings.main_tracker, Tracker::Anilist);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        c.bodies()
            .iter()
            .filter(|b| matches!(b, EventBody::SettingsChanged))
            .count(),
        2
    );

    defaults.scale = 9.0;
    assert!(
        matches!(core.call(Call::SetSubtitleDefaults { defaults }), Err(CoreError::Invalid { field, .. }) if field == "scale")
    );
}

/// The provider order defaults to AniList then TVmaze, takes any reordering
/// of exactly those two, and refuses a list that leaves one out, names one
/// twice or names a provider that cannot match.
#[test]
fn the_provider_order_defaults_reorders_and_refuses_anything_but_a_permutation() {
    let (_dir, core, c) = common::open_core();
    let order = |core: &Core| match core.call(Call::GetSettings).unwrap() {
        Reply::Settings { settings } => settings.provider_order,
        other => panic!("{other:?}"),
    };
    assert_eq!(order(&core), vec![Provider::Anilist, Provider::Tvmaze]);

    assert!(matches!(
        core.call(Call::SetProviderOrder {
            order: vec![Provider::Tvmaze, Provider::Anilist]
        })
        .unwrap(),
        Reply::Ok
    ));
    assert_eq!(order(&core), vec![Provider::Tvmaze, Provider::Anilist]);
    assert!(
        c.bodies()
            .iter()
            .any(|b| matches!(b, EventBody::SettingsChanged))
    );

    for bad in [
        vec![Provider::Anilist],
        vec![Provider::Anilist, Provider::Anilist, Provider::Tvmaze],
        vec![Provider::Anilist, Provider::Tvmaze, Provider::Mal],
        vec![],
    ] {
        assert!(
            matches!(core.call(Call::SetProviderOrder { order: bad.clone() }), Err(CoreError::Invalid { field, .. }) if field == "order"),
            "{bad:?}"
        );
    }
    // A refused order changes nothing.
    assert_eq!(order(&core), vec![Provider::Tvmaze, Provider::Anilist]);

    // A stored order that is no longer a permutation, a row from a build
    // that knew other providers, reads as the default.
    core.store()
        .write(|c| {
            c.execute(
                "UPDATE settings SET value = '[\"Anilist\"]' WHERE key = 'provider_order'",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    assert_eq!(order(&core), vec![Provider::Anilist, Provider::Tvmaze]);
    core.shutdown();
}
