//! Three ways to fill the token set: the terminal palette, a base16 file, the portal's
//! scheme and accent. The ratios and grounds are the prototype's (`qml/Theme.qml`).

use material_colors::color::Argb;
use material_colors::hct::Hct;
use material_colors::palette::TonalPalette;
use material_colors::scheme::variant::SchemeTonalSpot;

use crate::theme::colour::{Rgb, browned, hue_between};
use crate::theme::{Base16Theme, Mode, Palette, Portal, Steps, TerminalPalette};

const DARK_GROUND: &str = "#101216";
const LIGHT_GROUND: &str = "#f6f7fa";
const DARK_TEXT: &str = "#e4e7ee";
const LIGHT_TEXT: &str = "#1b1e26";
/// The AniBeam teal, the seed when the portal has no accent.
const ANIBEAM_TEAL: &str = "#46e0c4";
pub const SCRIM_ALPHA: f64 = 0.8;

fn hex(s: &str) -> Rgb {
    Rgb::hex(s).expect("a literal colour")
}

struct Mixed {
    surface: Rgb,
    raised: Rgb,
    pressed: Rgb,
    line: Rgb,
    line_strong: Rgb,
    faint: Rgb,
    dim: Rgb,
}

/// The pieces `finish` doesn't take directly: the eight hues (red first, then the derived
/// orange and the six terminal-slot hues), the surface/line/text mixes, the sunken colour
/// and the source label. Grouped so `finish` stays under clippy's argument limit.
struct Parts {
    hues: [Rgb; 8],
    mixed: Mixed,
    sunken: Rgb,
    label: String,
}

/// The mixes every source shares, given the ground and the text.
fn finish(
    bg: Rgb,
    text: Rgb,
    accent: Rgb,
    red: Rgb,
    mode: Mode,
    focus: Rgb,
    parts: Parts,
) -> Palette {
    let Parts {
        hues,
        mixed,
        sunken,
        label,
    } = parts;
    let [_red, orange, yellow, green, cyan, blue, purple, brown] = hues;
    let accent_text = if accent.contrast(bg) > accent.contrast(text) {
        bg
    } else {
        text
    };
    Palette {
        bg,
        surface: mixed.surface,
        surface_raised: mixed.raised,
        surface_sunken: sunken,
        surface_pressed: mixed.pressed,
        line: mixed.line,
        line_strong: mixed.line_strong,
        text,
        text_dim: mixed.dim,
        text_faint: mixed.faint,
        accent,
        accent_text,
        accent_soft: bg.mix(accent, 0.2),
        red_soft: bg.mix(red, 0.2),
        focus,
        red,
        orange,
        yellow,
        green,
        cyan,
        blue,
        purple,
        brown,
        scrim_alpha: SCRIM_ALPHA,
        mode,
        source_label: label,
    }
}

fn mixes(bg: Rgb, text: Rgb, contrast: bool, s: &Steps) -> Mixed {
    let m = if contrast { 1.5 } else { 1.0 };
    Mixed {
        surface: bg.mix(text, s.surface),
        raised: bg.mix(text, s.raised),
        pressed: bg.mix(text, s.raised * 1.5),
        line: bg.mix(text, s.line * m),
        line_strong: bg.mix(text, s.line_strong * m),
        faint: bg.mix(text, s.faint * m),
        dim: bg.mix(text, s.dim),
    }
}

fn away(mode: Mode) -> Rgb {
    match mode {
        Mode::Dark => Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        },
        Mode::Light => Rgb {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
    }
}

/// `slot` is 1 to 6 for a terminal colour, 7 for the derived orange.
pub fn from_terminal(
    term: &TerminalPalette,
    mode: Mode,
    slot: u8,
    contrast: bool,
    steps: &Steps,
) -> Palette {
    let native = Mode::of_ground(term.background);
    let slot = slot.clamp(1, 7) as usize;
    let mut c = term.colors;
    let (bg, text, forced) = if native == mode {
        (term.background, term.foreground, false)
    } else {
        let tint = c[slot.min(6)];
        let ground = match mode {
            Mode::Dark => hex(DARK_GROUND),
            Mode::Light => hex(LIGHT_GROUND),
        };
        let text = match mode {
            Mode::Dark => hex(DARK_TEXT),
            Mode::Light => hex(LIGHT_TEXT),
        };
        for (i, colour) in c.iter_mut().enumerate() {
            if !matches!(i, 0 | 7 | 8 | 15) {
                *colour = colour.retone(mode);
            }
        }
        (ground.mix(tint, 0.03), text, true)
    };
    let (red, green, yellow, blue, purple, cyan) = (c[1], c[2], c[3], c[4], c[5], c[6]);
    let orange = hue_between(red, yellow);
    let brown = browned(orange);
    let accent = if slot == 7 { orange } else { c[slot] };
    let focus = if slot == 7 {
        orange
    } else if term.colors[slot].to_hex() == term.colors[slot + 8].to_hex() {
        accent
    } else {
        c[slot + 8]
    };
    let mixed = mixes(bg, text, contrast, steps);
    let sunken = bg.mix(away(mode), steps.sunken);
    let label = format!(
        "terminal {}{}",
        term.source,
        if forced {
            format!(" (forced {})", mode.as_str())
        } else {
            String::new()
        }
    );
    finish(
        bg,
        text,
        accent,
        red,
        mode,
        focus,
        Parts {
            hues: [red, orange, yellow, green, cyan, blue, purple, brown],
            mixed,
            sunken,
            label,
        },
    )
}

/// The tinted-theming slot roles: base00 bg, base01 surface, base02 raised and line,
/// base03 line.strong and text.faint, base04 text.dim, base05 text, base08 to base0F the hues.
pub fn from_base16(theme: &Base16Theme, contrast: bool, steps: &Steps) -> Palette {
    let p = theme.palette;
    let mode = theme.mode();
    let (bg, text) = (p[0], p[5]);
    let m = if contrast { 1.5 } else { 1.0 };
    let mixed = Mixed {
        surface: p[1],
        raised: p[2],
        pressed: p[3],
        line: if contrast { bg.mix(p[2], m) } else { p[2] },
        line_strong: if contrast {
            bg.mix(p[3], m.min(1.0))
        } else {
            p[3]
        },
        faint: p[3],
        dim: p[4],
    };
    let hues = [p[8], p[9], p[10], p[11], p[12], p[13], p[14], p[15]];
    let accent = match theme.accent.as_str() {
        "base08" => p[8],
        "base09" => p[9],
        "base0A" => p[10],
        "base0B" => p[11],
        "base0C" => p[12],
        "base0E" => p[14],
        "base0F" => p[15],
        _ => p[13],
    };
    let sunken = bg.mix(away(mode), steps.sunken);
    let label = format!("theme {}", theme.name);
    finish(
        bg,
        text,
        accent,
        p[8],
        mode,
        accent,
        Parts {
            hues,
            mixed,
            sunken,
            label,
        },
    )
}

fn argb_to_rgb(a: Argb) -> Rgb {
    Rgb {
        r: a.red as f64 / 255.0,
        g: a.green as f64 / 255.0,
        b: a.blue as f64 / 255.0,
    }
}

fn rgb_to_argb(c: Rgb) -> Argb {
    let (r, g, b) = c.bytes();
    Argb::new(255, r, g, b)
}

/// Material's tonal spot scheme from the seed; the roles land on the tokens as spec 4.2's
/// table says, `text.faint` is the mix step, and the hues are generated at the seed's
/// chroma from fixed HCT hue angles.
pub fn from_portal(portal: &Portal, mode: Mode, contrast: bool, steps: &Steps) -> Palette {
    let seed = portal.accent.unwrap_or_else(|| hex(ANIBEAM_TEAL));
    let seed_hct = Hct::new(rgb_to_argb(seed));
    let level = if contrast || portal.contrast {
        Some(1.0)
    } else {
        Some(0.0)
    };
    let scheme = SchemeTonalSpot::new(seed_hct, mode == Mode::Dark, level).scheme;
    let bg = argb_to_rgb(scheme.background());
    let text = argb_to_rgb(scheme.on_surface());
    let chroma = seed_hct.get_chroma().max(24.0);
    let tone = match mode {
        Mode::Dark => 75,
        Mode::Light => 45,
    };
    let hue_at = |h: f64, t: i32| argb_to_rgb(TonalPalette::of(h, chroma).tone(t));
    let hues = [
        argb_to_rgb(scheme.error()),
        hue_at(55.0, tone),
        hue_at(90.0, tone),
        hue_at(145.0, tone),
        hue_at(200.0, tone),
        hue_at(260.0, tone),
        hue_at(310.0, tone),
        argb_to_rgb(TonalPalette::of(55.0, chroma * 0.5).tone(match mode {
            Mode::Dark => 55,
            Mode::Light => 35,
        })),
    ];
    let mixed = Mixed {
        surface: argb_to_rgb(scheme.surface_container()),
        raised: argb_to_rgb(scheme.surface_container_high()),
        pressed: argb_to_rgb(scheme.surface_container_highest()),
        line: argb_to_rgb(scheme.outline_variant()),
        line_strong: argb_to_rgb(scheme.outline()),
        faint: bg.mix(text, steps.faint * if contrast { 1.5 } else { 1.0 }),
        dim: argb_to_rgb(scheme.on_surface_variant()),
    };
    let accent = argb_to_rgb(scheme.primary());
    let sunken = argb_to_rgb(scheme.surface_container_lowest());
    let label = format!(
        "portal, derived ({})",
        portal.scheme.map(Mode::as_str).unwrap_or("unset")
    );
    let mut p = finish(
        bg,
        text,
        accent,
        hues[0],
        mode,
        accent,
        Parts {
            hues,
            mixed,
            sunken,
            label,
        },
    );
    p.accent_text = argb_to_rgb(scheme.on_primary());
    p.accent_soft = argb_to_rgb(scheme.primary_container());
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::colour::Rgb;
    use crate::theme::{Base16Theme, Mode, Portal, Steps, TerminalPalette};

    fn rose_pine() -> TerminalPalette {
        let hex = |s: &str| Rgb::hex(s).unwrap();
        TerminalPalette {
            foreground: hex("#e0def4"),
            background: hex("#191724"),
            colors: [
                hex("#26233a"),
                hex("#eb6f92"),
                hex("#31748f"),
                hex("#f6c177"),
                hex("#9ccfd8"),
                hex("#c4a7e7"),
                hex("#ebbcba"),
                hex("#e0def4"),
                hex("#6e6a86"),
                hex("#eb6f92"),
                hex("#31748f"),
                hex("#f6c177"),
                hex("#9ccfd8"),
                hex("#c4a7e7"),
                hex("#ebbcba"),
                hex("#e0def4"),
            ],
            source: "kitty".into(),
        }
    }

    #[test]
    fn a_terminal_palette_in_its_own_mode_keeps_bg_text_and_slots() {
        let p = from_terminal(&rose_pine(), Mode::Dark, 4, false, &Steps::default());
        assert_eq!(p.bg.to_hex(), "#191724");
        assert_eq!(p.text.to_hex(), "#e0def4");
        assert_eq!(p.red.to_hex(), "#eb6f92");
        assert_eq!(p.blue.to_hex(), "#9ccfd8");
        assert_eq!(p.accent.to_hex(), "#9ccfd8");
        assert_eq!(p.surface.to_hex(), p.bg.mix(p.text, 0.05).to_hex());
        assert_eq!(p.line.to_hex(), p.bg.mix(p.text, 0.16).to_hex());
        assert_eq!(p.text_faint.to_hex(), p.bg.mix(p.text, 0.45).to_hex());
        // the bright pair equals the normal slot, so the focus ring is the accent itself
        assert_eq!(p.focus.to_hex(), p.accent.to_hex());
        assert_eq!(p.accent_soft.to_hex(), p.bg.mix(p.accent, 0.2).to_hex());
        assert_eq!(p.mode, Mode::Dark);
        assert_eq!(p.source_label, "terminal kitty");
    }

    #[test]
    fn contrast_widens_the_line_and_faint_steps_by_half() {
        let p = from_terminal(&rose_pine(), Mode::Dark, 4, true, &Steps::default());
        assert_eq!(p.line.to_hex(), p.bg.mix(p.text, 0.24).to_hex());
        assert_eq!(p.line_strong.to_hex(), p.bg.mix(p.text, 0.39).to_hex());
        assert_eq!(p.text_faint.to_hex(), p.bg.mix(p.text, 0.675).to_hex());
        assert_eq!(p.text_dim.to_hex(), p.bg.mix(p.text, 0.70).to_hex());
    }

    #[test]
    fn a_forced_mode_derives_a_neutral_ground_and_retones_the_hues() {
        let p = from_terminal(&rose_pine(), Mode::Light, 4, false, &Steps::default());
        let ground = Rgb::hex("#f6f7fa")
            .unwrap()
            .mix(Rgb::hex("#9ccfd8").unwrap(), 0.03);
        assert_eq!(p.bg.to_hex(), ground.to_hex());
        assert_eq!(p.text.to_hex(), "#1b1e26");
        assert!(p.red.to_hsl().2 <= 0.42 + 0.001);
        assert_eq!(p.accent.to_hsl().0, Rgb::hex("#9ccfd8").unwrap().to_hsl().0);
        assert_eq!(p.source_label, "terminal kitty (forced light)");
        assert_eq!(p.mode, Mode::Light);
    }

    #[test]
    fn slot_seven_is_the_derived_orange() {
        let p = from_terminal(&rose_pine(), Mode::Dark, 7, false, &Steps::default());
        assert_eq!(p.accent.to_hex(), p.orange.to_hex());
    }

    fn mocha() -> Base16Theme {
        let hex = |s: &str| Rgb::hex(s).unwrap();
        Base16Theme {
            stem: "catppuccin-mocha".into(),
            name: "Catppuccin Mocha".into(),
            variant: Some(Mode::Dark),
            accent: "base0D".into(),
            palette: [
                hex("#1e1e2e"),
                hex("#181825"),
                hex("#313244"),
                hex("#45475a"),
                hex("#585b70"),
                hex("#cdd6f4"),
                hex("#f5e0dc"),
                hex("#b4befe"),
                hex("#f38ba8"),
                hex("#fab387"),
                hex("#f9e2af"),
                hex("#a6e3a1"),
                hex("#94e2d5"),
                hex("#89b4fa"),
                hex("#cba6f7"),
                hex("#f2cdcd"),
            ],
        }
    }

    #[test]
    fn base16_slots_land_on_their_roles() {
        let p = from_base16(&mocha(), false, &Steps::default());
        assert_eq!(p.bg.to_hex(), "#1e1e2e");
        assert_eq!(p.surface.to_hex(), "#181825");
        assert_eq!(p.surface_raised.to_hex(), "#313244");
        assert_eq!(p.line.to_hex(), "#313244");
        assert_eq!(p.line_strong.to_hex(), "#45475a");
        assert_eq!(p.text_faint.to_hex(), "#45475a");
        assert_eq!(p.text_dim.to_hex(), "#585b70");
        assert_eq!(p.text.to_hex(), "#cdd6f4");
        assert_eq!(p.red.to_hex(), "#f38ba8");
        assert_eq!(p.brown.to_hex(), "#f2cdcd");
        assert_eq!(p.accent.to_hex(), "#89b4fa");
        assert_eq!(p.focus.to_hex(), p.accent.to_hex());
        // sunken: base00 pushed 0.03 away from base05
        let away = Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        assert_eq!(p.surface_sunken.to_hex(), p.bg.mix(away, 0.03).to_hex());
        assert_eq!(p.source_label, "theme Catppuccin Mocha");
        let mut purple = mocha();
        purple.accent = "base0E".into();
        assert_eq!(
            from_base16(&purple, false, &Steps::default())
                .accent
                .to_hex(),
            "#cba6f7"
        );
    }

    #[test]
    fn the_portal_path_derives_from_the_seed_and_generates_the_hues() {
        let portal = Portal {
            scheme: Some(Mode::Dark),
            contrast: false,
            accent: Some(Rgb::hex("#3584e4").unwrap()),
        };
        let p = from_portal(&portal, Mode::Dark, false, &Steps::default());
        assert_eq!(p.mode, Mode::Dark);
        assert!(p.bg.lightness() < 0.2, "a dark scheme has a dark ground");
        assert!(p.text.lightness() > 0.8);
        assert!(
            p.accent.to_hsl().2 > 0.5,
            "the primary reads on a dark ground"
        );
        let (hue, _, _) = p.yellow.to_hsl();
        assert!((0.10..=0.20).contains(&hue), "yellow hue {hue}");
        let (hue, _, _) = p.green.to_hsl();
        assert!((0.25..=0.45).contains(&hue), "green hue {hue}");
        assert_eq!(p.text_faint.to_hex(), p.bg.mix(p.text, 0.45).to_hex());
        assert_eq!(p.source_label, "portal, derived (dark)");
        let none = Portal {
            scheme: None,
            contrast: false,
            accent: None,
        };
        let q = from_portal(&none, Mode::Light, false, &Steps::default());
        assert!(q.bg.lightness() > 0.8);
        assert_eq!(q.source_label, "portal, derived (unset)");
    }

    #[test]
    fn format_and_status_hues() {
        use crate::theme::{format_hue, status_hue};
        assert_eq!(format_hue("TV"), "cyan");
        assert_eq!(format_hue("TV_SHORT"), "cyan");
        assert_eq!(format_hue("MOVIE"), "yellow");
        assert_eq!(format_hue("OVA"), "purple");
        assert_eq!(format_hue("ONA"), "green");
        assert_eq!(format_hue("SPECIAL"), "red");
        assert_eq!(format_hue("MUSIC"), "green");
        assert_eq!(format_hue("MANGA"), "orange");
        assert_eq!(format_hue("LIGHT_NOVEL"), "red");
        assert_eq!(format_hue("ONE_SHOT"), "yellow");
        assert_eq!(format_hue("VISUAL_NOVEL"), "purple");
        assert_eq!(format_hue("weird"), "text.dim");
        assert_eq!(status_hue("Watching"), "accent");
        assert_eq!(status_hue("Repeating"), "purple");
        assert_eq!(status_hue("Planning"), "text.faint");
    }
}
