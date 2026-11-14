//! The calls the front door answers itself: what this core is, its event log
//! and its jobs, and the preferences and settings every team reads. Every
//! other arm of `Core::call` routes to the module that owns the call.

use crate::contract::*;
use crate::core::Core;
use crate::prefs;
use crate::trackers::on_hold;

pub(crate) fn about(core: &Core) -> Result<Reply, CoreError> {
    Ok(Reply::About {
        about: About {
            version: crate::VERSION.to_string(),
            data_dir: core.paths.data_dir.clone(),
            config_dir: core.paths.config_dir.clone(),
            cache_dir: core.paths.cache_dir.clone(),
            db_path: core.paths.db_path().to_string_lossy().into_owned(),
        },
    })
}

pub(crate) fn recent_events(core: &Core, limit: u64) -> Result<Reply, CoreError> {
    Ok(Reply::Events {
        events: core.bus.recent(limit)?,
    })
}

pub(crate) fn clear_events(core: &Core) -> Result<Reply, CoreError> {
    core.bus.clear()?;
    Ok(Reply::Ok)
}

pub(crate) fn list_jobs(core: &Core) -> Result<Reply, CoreError> {
    Ok(Reply::Jobs {
        jobs: core.jobs.list(),
    })
}

pub(crate) fn cancel_job(core: &Core, job: u64) -> Result<Reply, CoreError> {
    core.jobs.cancel(job)?;
    Ok(Reply::Ok)
}

pub(crate) fn preferences(core: &Core) -> Result<Reply, CoreError> {
    Ok(Reply::Preferences {
        preferences: core.store.read(prefs::load_preferences)?,
    })
}

/// Saves the preferences and hands each team the switches it follows: the
/// tailnet host, Discord, the calendar feed and On hold.
pub(crate) fn set_preferences(core: &Core, preferences: Preferences) -> Result<Reply, CoreError> {
    prefs::validate_preferences(&preferences)?;
    let p = preferences;
    let (before, preferences) = core.store.write(move |c| {
        let before = prefs::load_preferences(c)?;
        // Only SetTailnetDeviceTrust changes which devices are off,
        // so a stale copy of the preferences saved from a page
        // can never switch a device back on.
        let mut p = p;
        p.tailnet_switched_off = before.tailnet_switched_off.clone();
        prefs::save_preferences(c, &p)?;
        Ok((before, p))
    })?;
    core.host.set_streaming(preferences.stream_over_tailscale);
    core.host
        .set_switched_off(preferences.tailnet_switched_off.clone());
    core.host.follow(core, preferences.share_over_tailscale);
    crate::presence::set_enabled(core, preferences.discord_presence);
    if before.calendar_feed != preferences.calendar_feed {
        crate::calendar::set_enabled(core, preferences.calendar_feed);
    }
    // On hold switched on: the entries due now move without
    // waiting for the next launch.
    if on_hold::wants_sweep(&before, &preferences)
        && let Some(owner) = core.arc()
    {
        on_hold::kick(&owner);
    }
    preferences_changed(core, preferences)
}

/// The end of every preferences write: the bus says what the preferences
/// are now, and the call answers Ok.
pub(crate) fn preferences_changed(
    core: &Core,
    preferences: Preferences,
) -> Result<Reply, CoreError> {
    core.bus.debug(
        Stage::Store,
        "preferences changed",
        EventBody::PreferencesChanged { preferences },
    );
    Ok(Reply::Ok)
}

pub(crate) fn settings(core: &Core) -> Result<Reply, CoreError> {
    Ok(Reply::Settings {
        settings: core.store.read(prefs::load_settings)?,
    })
}

pub(crate) fn set_subtitle_defaults(
    core: &Core,
    defaults: SubtitleDefaults,
) -> Result<Reply, CoreError> {
    prefs::validate_subtitle_defaults(&defaults)?;
    core.store
        .write(move |c| prefs::save_subtitle_defaults(c, &defaults))?;
    settings_changed(core, "subtitle defaults changed")
}

pub(crate) fn set_auto_skip(core: &Core, intro: bool, outro: bool) -> Result<Reply, CoreError> {
    core.store
        .write(move |c| prefs::save_auto_skip(c, &AutoSkip { intro, outro }))?;
    settings_changed(core, "auto-skip changed")
}

pub(crate) fn set_provider_order(core: &Core, order: Vec<Provider>) -> Result<Reply, CoreError> {
    prefs::validate_provider_order(&order)?;
    core.store
        .write(move |c| prefs::save_provider_order(c, &order))?;
    settings_changed(core, "provider order changed")
}

/// A saved setting says so on the bus, and the call answers Ok.
fn settings_changed(core: &Core, message: &'static str) -> Result<Reply, CoreError> {
    core.bus
        .debug(Stage::Store, message, EventBody::SettingsChanged);
    Ok(Reply::Ok)
}
