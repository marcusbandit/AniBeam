mod common;
use anibeam_core::*;

#[test]
fn preferences_and_settings_round_trip_with_events() {
    let (_dir, core, c) = common::open_core();
    assert!(matches!(core.call(Call::GetPreferences).unwrap(), Reply::Preferences { preferences } if preferences == Preferences::default()));
    let prefs = Preferences { title_language: TitleLanguage::English, library_tab: Tab::Movies, library_sort: Sort::MyScore, library_direction: Direction::Desc, feed_sort: FeedSort::Upcoming };
    assert!(matches!(core.call(Call::SetPreferences { preferences: prefs.clone() }).unwrap(), Reply::Ok));
    assert!(matches!(core.call(Call::GetPreferences).unwrap(), Reply::Preferences { preferences } if preferences == prefs));
    assert!(c.bodies().iter().any(|b| matches!(b, EventBody::PreferencesChanged { .. })));

    assert!(matches!(core.call(Call::GetSettings).unwrap(), Reply::Settings { settings } if settings == Settings::default()));
    let mut defaults = SubtitleDefaults { scale: 1.5, ..Default::default() };
    assert!(matches!(core.call(Call::SetSubtitleDefaults { defaults: defaults.clone() }).unwrap(), Reply::Ok));
    assert!(matches!(core.call(Call::SetAutoSkip { intro: true, outro: false }).unwrap(), Reply::Ok));
    match core.call(Call::GetSettings).unwrap() {
        Reply::Settings { settings } => {
            assert_eq!(settings.subtitle_defaults, defaults);
            assert_eq!(settings.auto_skip, AutoSkip { intro: true, outro: false });
            assert_eq!(settings.main_tracker, Tracker::Anilist);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(c.bodies().iter().filter(|b| matches!(b, EventBody::SettingsChanged)).count(), 2);

    defaults.scale = 9.0;
    assert!(matches!(core.call(Call::SetSubtitleDefaults { defaults }), Err(CoreError::Invalid { field, .. }) if field == "scale"));
}
