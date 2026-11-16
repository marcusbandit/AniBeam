//! The calls the front door answers itself: what this core is, its event log
//! and its jobs, and the preferences and settings every team reads. Every
//! other arm of `Core::call` routes to the module that owns the call.

use crate::contract::*;
use crate::core::Core;
use crate::foundation::services::Services;
use crate::prefs;
use crate::trackers::on_hold;

pub(crate) fn about(services: &Services) -> Result<Reply, CoreError> {
    Ok(Reply::About {
        about: About {
            version: crate::VERSION.to_string(),
            data_dir: services.paths.data_dir.clone(),
            config_dir: services.paths.config_dir.clone(),
            cache_dir: services.paths.cache_dir.clone(),
            db_path: services.paths.db_path().to_string_lossy().into_owned(),
        },
    })
}

pub(crate) fn recent_events(services: &Services, limit: u64) -> Result<Reply, CoreError> {
    Ok(Reply::Events {
        events: services.bus.recent(limit)?,
    })
}

pub(crate) fn clear_events(services: &Services) -> Result<Reply, CoreError> {
    services.bus.clear()?;
    Ok(Reply::Ok)
}

pub(crate) fn list_jobs(services: &Services) -> Result<Reply, CoreError> {
    Ok(Reply::Jobs {
        jobs: services.jobs.list(),
    })
}

pub(crate) fn cancel_job(services: &Services, job: u64) -> Result<Reply, CoreError> {
    services.jobs.cancel(job)?;
    Ok(Reply::Ok)
}

pub(crate) fn preferences(services: &Services) -> Result<Reply, CoreError> {
    Ok(Reply::Preferences {
        preferences: services.store.read(prefs::load_preferences)?,
    })
}

/// Saves the preferences and hands each team the switches it follows: the
/// tailnet host, Discord, the calendar feed and On hold.
pub(crate) fn set_preferences(core: &Core, preferences: Preferences) -> Result<Reply, CoreError> {
    prefs::validate_preferences(&preferences)?;
    let p = preferences;
    let (before, preferences) = core.services.store.write(move |c| {
        let before = prefs::load_preferences(c)?;
        // Only SetTailnetDeviceTrust changes which devices are off,
        // so a stale copy of the preferences saved from a page
        // can never switch a device back on.
        let mut p = p;
        p.tailnet_switched_off = before.tailnet_switched_off.clone();
        prefs::save_preferences(c, &p)?;
        Ok((before, p))
    })?;
    crate::host::apply_preferences(&core.host, &preferences);
    crate::presence::set_enabled(&core.playback, preferences.discord_presence);
    if before.calendar_feed != preferences.calendar_feed {
        crate::calendar::set_enabled(&core.metadata, preferences.calendar_feed);
    }
    // On hold switched on: the entries due now move without
    // waiting for the next launch.
    if on_hold::wants_sweep(&before, &preferences) {
        on_hold::kick(&core.trackers);
    }
    prefs::changed(&core.services, preferences)
}

/// `GetSettings`: the settings as the store holds them.
pub(crate) fn settings(services: &Services) -> Result<Reply, CoreError> {
    Ok(Reply::Settings {
        settings: services.store.read(prefs::load_settings)?,
    })
}

pub(crate) fn set_subtitle_defaults(
    services: &Services,
    defaults: SubtitleDefaults,
) -> Result<Reply, CoreError> {
    prefs::validate_subtitle_defaults(&defaults)?;
    services
        .store
        .write(move |c| prefs::save_subtitle_defaults(c, &defaults))?;
    settings_changed(services, "subtitle defaults changed")
}

pub(crate) fn set_auto_skip(
    services: &Services,
    intro: bool,
    outro: bool,
) -> Result<Reply, CoreError> {
    services
        .store
        .write(move |c| prefs::save_auto_skip(c, &AutoSkip { intro, outro }))?;
    settings_changed(services, "auto-skip changed")
}

pub(crate) fn set_provider_order(
    services: &Services,
    order: Vec<Provider>,
) -> Result<Reply, CoreError> {
    prefs::validate_provider_order(&order)?;
    services
        .store
        .write(move |c| prefs::save_provider_order(c, &order))?;
    settings_changed(services, "provider order changed")
}

/// A saved setting says so on the bus, and the call answers Ok.
fn settings_changed(services: &Services, message: &'static str) -> Result<Reply, CoreError> {
    services
        .bus
        .debug(Stage::Store, message, EventBody::SettingsChanged);
    Ok(Reply::Ok)
}
