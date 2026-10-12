//! Preferences (the library and feed view state) and settings (subtitle
//! defaults, auto-skip, the main tracker and the provider order), both stored as JSON blobs in
//! the settings table keyed by `store::settings`'s key constants.

use rusqlite::Connection;

use crate::contract::*;
use crate::store::settings;

pub fn load_preferences(conn: &Connection) -> Result<Preferences, CoreError> {
    settings::get_or_default(conn, settings::PREFERENCES)
}

pub fn save_preferences(conn: &Connection, p: &Preferences) -> Result<(), CoreError> {
    settings::set(conn, settings::PREFERENCES, p)
}

pub fn load_main_tracker(conn: &Connection) -> Result<Tracker, CoreError> {
    Ok(settings::get::<Tracker>(conn, settings::MAIN_TRACKER)?.unwrap_or(Tracker::Anilist))
}

pub fn save_main_tracker(conn: &Connection, t: Tracker) -> Result<(), CoreError> {
    settings::set(conn, settings::MAIN_TRACKER, &t)
}

pub fn load_settings(conn: &Connection) -> Result<Settings, CoreError> {
    Ok(Settings {
        subtitle_defaults: settings::get_or_default(conn, settings::SUBTITLE_DEFAULTS)?,
        auto_skip: settings::get_or_default(conn, settings::AUTO_SKIP)?,
        main_tracker: load_main_tracker(conn)?,
        provider_order: load_provider_order(conn)?,
    })
}

/// The match providers in the order the auto-match walks them. A stored
/// order that no longer names every provider exactly once, a row from a
/// build that knew fewer of them, reads as the default rather than
/// leaving a provider out of every match for good.
pub fn load_provider_order(conn: &Connection) -> Result<Vec<Provider>, CoreError> {
    let stored: Option<Vec<Provider>> = settings::get(conn, settings::PROVIDER_ORDER)?;
    Ok(stored
        .filter(|order| validate_provider_order(order).is_ok())
        .unwrap_or_else(|| MATCH_PROVIDERS.to_vec()))
}

pub fn save_provider_order(conn: &Connection, order: &[Provider]) -> Result<(), CoreError> {
    settings::set(conn, settings::PROVIDER_ORDER, &order)
}

/// Every match provider exactly once, and nothing that is not one: an
/// order is a reordering, never a way to switch a provider off.
pub fn validate_provider_order(order: &[Provider]) -> Result<(), CoreError> {
    if let Some(p) = order.iter().find(|p| !MATCH_PROVIDERS.contains(p)) {
        return Err(CoreError::invalid(
            "order",
            format!("{} is not a match provider", p.label()),
        ));
    }
    for p in MATCH_PROVIDERS {
        let n = order.iter().filter(|o| **o == p).count();
        if n != 1 {
            return Err(CoreError::invalid(
                "order",
                format!(
                    "{} appears {n} times; every provider appears once",
                    p.label()
                ),
            ));
        }
    }
    Ok(())
}

pub fn save_subtitle_defaults(conn: &Connection, d: &SubtitleDefaults) -> Result<(), CoreError> {
    settings::set(conn, settings::SUBTITLE_DEFAULTS, d)
}

pub fn save_auto_skip(conn: &Connection, a: &AutoSkip) -> Result<(), CoreError> {
    settings::set(conn, settings::AUTO_SKIP, a)
}

fn range(field: &str, v: f64, lo: f64, hi: f64) -> Result<(), CoreError> {
    if v.is_finite() && (lo..=hi).contains(&v) {
        Ok(())
    } else {
        Err(CoreError::invalid(
            field,
            format!("{v} is outside {lo} to {hi}"),
        ))
    }
}

pub fn validate_subtitle_defaults(d: &SubtitleDefaults) -> Result<(), CoreError> {
    range("scale", d.scale, 0.5, 2.0)?;
    range("position", d.text_style.position, 0.0, 150.0)?;
    range("box_opacity", d.text_style.box_opacity, 0.0, 1.0)?;
    range("outline_size", d.text_style.outline_size, 0.0, f64::MAX)?;
    range("shadow_offset", d.text_style.shadow_offset, 0.0, f64::MAX)?;
    if d.text_style.font.trim().is_empty() {
        return Err(CoreError::invalid("font", "empty"));
    }
    Ok(())
}
