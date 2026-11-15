//! More, Subtitles: the core's subtitle defaults, read when the page opens and written as
//! they are edited. The desktop's Settings, Subtitles tab saves through one debounced call;
//! so does this, so a drag on a slider is one write and not sixty. The next episode starts
//! with what is saved, as the player reads the defaults from the core when a session opens.

use std::cell::RefCell;
use std::time::Duration;

use anibeam_core::{AssOverride, Call, Colour, Reply, SubtitleDefaults};
use slint::{Color, ComponentHandle, Timer, TimerMode};

use crate::{AppWindow, SubtitleData};

/// How long an edit waits for the next one before it is written.
const SAVE_AFTER: Duration = Duration::from_millis(400);

struct State {
    window: slint::Weak<AppWindow>,
    defaults: SubtitleDefaults,
    /// The saved defaults were read from the core. Until they have been, an edit is not
    /// taken: it would write the stock values over the saved ones, language orders included.
    loaded: bool,
    /// An edit waits on the timer.
    dirty: bool,
    save: Timer,
    /// Fields whose text cannot be saved, by name; nothing is written for them.
    bad: Vec<&'static str>,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

/// `#rrggbb`, six hex digits, opaque.
fn parse_hex(text: &str) -> Option<Colour> {
    let t = text.trim();
    let digits = t.strip_prefix('#')?;
    if digits.len() != 6 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    Some(Colour {
        r: byte(0)?,
        g: byte(2)?,
        b: byte(4)?,
        a: 255,
    })
}

fn hex_of(c: &Colour) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

/// A number, 0 or more.
fn parse_size(text: &str) -> Option<f64> {
    let v: f64 = text.trim().parse().ok()?;
    (v.is_finite() && v >= 0.0).then_some(v)
}

fn number(v: f64) -> String {
    format!("{v}")
}

fn paint(c: &Colour) -> Color {
    Color::from_rgb_u8(c.r, c.g, c.b)
}

fn mode_index(m: &AssOverride) -> i32 {
    match m {
        AssOverride::AsScripted => 0,
        AssOverride::ScaleOnly => 1,
        AssOverride::Force => 2,
    }
}

/// The values that follow an edit. The text fields are left alone: the two-way binding owns
/// them while someone types, and they are filled again only when the page opens or resets.
fn show(d: &SubtitleData, defaults: &SubtitleDefaults, bad: &[&str]) {
    let s = &defaults.text_style;
    d.set_scale(defaults.scale as f32);
    d.set_position(s.position as f32);
    d.set_ass_mode(mode_index(&defaults.ass_override));
    d.set_bold(s.bold);
    d.set_box_opacity(s.box_opacity as f32);
    d.set_colour_paint(paint(&s.colour));
    d.set_outline_paint(paint(&s.outline_colour));
    d.set_outline_num(s.outline_size as f32);
    d.set_shadow_num(s.shadow_offset as f32);
    d.set_bad_font(bad.contains(&"font"));
    d.set_bad_colour(bad.contains(&"colour"));
    d.set_bad_outline(bad.contains(&"outline"));
    d.set_bad_outline_colour(bad.contains(&"outline-colour"));
    d.set_bad_shadow(bad.contains(&"shadow"));
}

fn fill_fields(d: &SubtitleData, defaults: &SubtitleDefaults) {
    let s = &defaults.text_style;
    d.set_font(s.font.clone().into());
    d.set_colour(hex_of(&s.colour).into());
    d.set_outline_size(number(s.outline_size).into());
    d.set_outline_colour(hex_of(&s.outline_colour).into());
    d.set_shadow(number(s.shadow_offset).into());
}

fn saved() -> Option<SubtitleDefaults> {
    match crate::local::core().map(|core| core.call(Call::GetSettings)) {
        Some(Ok(Reply::Settings { settings })) => Some(settings.subtitle_defaults),
        _ => None,
    }
}

fn flush() {
    // The core is checked first: an edit made while it is not up stays waiting.
    let Some(core) = crate::local::core() else {
        return;
    };
    let defaults = STATE.with(|s| {
        s.borrow_mut()
            .as_mut()
            .and_then(|s| std::mem::take(&mut s.dirty).then(|| s.defaults.clone()))
    });
    let Some(defaults) = defaults else { return };
    if let Err(e) = core.call(Call::SetSubtitleDefaults { defaults }) {
        crate::local::log(&format!("subtitles: not saved: {e}"));
    }
}

/// Changes the defaults, shows what followed and writes them after a pause. `field` names a
/// text field the edit came from and whether its text is good: a bad one is flagged and the
/// defaults stay as they were.
fn edit(field: Option<(&'static str, bool)>, f: impl FnOnce(&mut SubtitleDefaults)) {
    let shown = STATE.with(|s| {
        let mut s = s.borrow_mut();
        let s = s.as_mut()?;
        if !s.loaded {
            s.defaults = saved()?;
            s.loaded = true;
        }
        if let Some((name, ok)) = field {
            s.bad.retain(|b| *b != name);
            if !ok {
                s.bad.push(name);
                return Some((s.window.clone(), s.defaults.clone(), s.bad.clone()));
            }
        }
        f(&mut s.defaults);
        s.dirty = true;
        s.save.start(TimerMode::SingleShot, SAVE_AFTER, flush);
        Some((s.window.clone(), s.defaults.clone(), s.bad.clone()))
    });
    if let Some((window, defaults, bad)) = shown
        && let Some(window) = window.upgrade()
    {
        show(&window.global::<SubtitleData>(), &defaults, &bad);
    }
}

/// Wires the page's callbacks. Once, at startup.
pub fn install(window: &AppWindow) {
    STATE.with(|s| {
        *s.borrow_mut() = Some(State {
            window: window.as_weak(),
            defaults: SubtitleDefaults::default(),
            loaded: false,
            dirty: false,
            save: Timer::default(),
            bad: Vec::new(),
        });
    });
    let d = window.global::<SubtitleData>();
    d.on_opened({
        let window = window.as_weak();
        move || {
            // An edit still waiting on the timer is written before the page reads again.
            flush();
            let read = saved();
            let defaults = read.clone().unwrap_or_default();
            let Some(window) = window.upgrade() else {
                return;
            };
            STATE.with(|s| {
                if let Some(s) = s.borrow_mut().as_mut() {
                    s.defaults = defaults.clone();
                    s.loaded = read.is_some();
                    s.bad.clear();
                }
            });
            let d = window.global::<SubtitleData>();
            fill_fields(&d, &defaults);
            show(&d, &defaults, &[]);
        }
    });
    d.on_set_scale(|v| edit(None, |d| d.scale = (v as f64).clamp(0.5, 2.0)));
    d.on_set_position(|v| {
        edit(None, |d| {
            d.text_style.position = (v as f64).clamp(0.0, 150.0)
        })
    });
    d.on_set_ass_mode(|i| {
        edit(None, |d| {
            d.ass_override = match i {
                0 => AssOverride::AsScripted,
                2 => AssOverride::Force,
                _ => AssOverride::ScaleOnly,
            }
        })
    });
    d.on_set_font(|t| {
        let t = t.trim().to_string();
        edit(Some(("font", !t.is_empty())), |d| d.text_style.font = t);
    });
    d.on_set_bold(|on| edit(None, |d| d.text_style.bold = on));
    d.on_set_colour(|t| {
        let c = parse_hex(&t);
        edit(Some(("colour", c.is_some())), |d| {
            if let Some(c) = c {
                d.text_style.colour = c;
            }
        });
    });
    d.on_set_outline_size(|t| {
        let v = parse_size(&t);
        edit(Some(("outline", v.is_some())), |d| {
            if let Some(v) = v {
                d.text_style.outline_size = v;
            }
        });
    });
    d.on_set_outline_colour(|t| {
        let c = parse_hex(&t);
        edit(Some(("outline-colour", c.is_some())), |d| {
            if let Some(c) = c {
                d.text_style.outline_colour = c;
            }
        });
    });
    d.on_set_shadow(|t| {
        let v = parse_size(&t);
        edit(Some(("shadow", v.is_some())), |d| {
            if let Some(v) = v {
                d.text_style.shadow_offset = v;
            }
        });
    });
    d.on_set_box_opacity(|v| {
        edit(None, |d| {
            d.text_style.box_opacity = (v as f64).clamp(0.0, 1.0)
        })
    });
    // Back to the look Anibeam ships with; the language orders stay as they are.
    d.on_reset({
        let window = window.as_weak();
        move || {
            let stock = SubtitleDefaults::default();
            edit(None, |d| {
                d.scale = stock.scale;
                d.ass_override = stock.ass_override;
                d.text_style = stock.text_style.clone();
            });
            STATE.with(|s| {
                if let Some(s) = s.borrow_mut().as_mut() {
                    s.bad.clear();
                }
            });
            if let Some(window) = window.upgrade() {
                let d = window.global::<SubtitleData>();
                fill_fields(&d, &stock);
                show(
                    &d,
                    &STATE
                        .with(|s| s.borrow().as_ref().map(|s| s.defaults.clone()))
                        .unwrap_or(stock),
                    &[],
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_is_six_hex_digits() {
        let c = parse_hex("#ffcc00").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (255, 204, 0, 255));
        assert_eq!(hex_of(&c), "#ffcc00");
        assert!(parse_hex("ffcc00").is_none());
        assert!(parse_hex("#ffcc0").is_none());
        assert!(parse_hex("#ffcc0g").is_none());
        assert!(parse_hex("").is_none());
    }

    #[test]
    fn a_size_is_a_number_of_zero_or_more() {
        assert_eq!(parse_size(" 1.65 "), Some(1.65));
        assert_eq!(parse_size("0"), Some(0.0));
        assert_eq!(parse_size("-1"), None);
        assert_eq!(parse_size(""), None);
        assert_eq!(parse_size("wide"), None);
        assert_eq!(parse_size("inf"), None);
    }

    #[test]
    fn the_stock_defaults_pass_the_core_s_own_checks() {
        let d = SubtitleDefaults::default();
        assert_eq!(number(d.text_style.outline_size), "1.65");
        assert_eq!(mode_index(&d.ass_override), 1);
    }
}
