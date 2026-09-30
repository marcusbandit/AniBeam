//! The theme model, spec 4.2: colour sources become one Palette of tokens per mode.
//! Nothing here touches Qt; the bridge in `bridge/theme.rs` hands the result to QML.

pub mod base16;
pub mod colour;
pub mod config;
pub mod engine;
pub mod kitty;
pub mod portal;
pub mod tokens;

use colour::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    /// Dark below the midpoint of the ground's lightness, light above it.
    pub fn of_ground(bg: Rgb) -> Mode {
        if bg.lightness() < 0.5 {
            Mode::Dark
        } else {
            Mode::Light
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Dark => "dark",
            Mode::Light => "light",
        }
    }
}

/// The mix steps of `bg` toward `text`; sunken is away from text. Prototype-tunable
/// defaults the prototype kept unchanged.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Steps {
    pub sunken: f64,
    pub surface: f64,
    pub raised: f64,
    pub line: f64,
    pub line_strong: f64,
    pub faint: f64,
    pub dim: f64,
}

impl Default for Steps {
    fn default() -> Self {
        Steps {
            sunken: 0.03,
            surface: 0.05,
            raised: 0.10,
            line: 0.16,
            line_strong: 0.26,
            faint: 0.45,
            dim: 0.70,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TerminalPalette {
    pub foreground: Rgb,
    pub background: Rgb,
    pub colors: [Rgb; 16],
    /// "kitty" today; what the settings page shows as the source.
    pub source: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Base16Theme {
    pub stem: String,
    pub name: String,
    pub variant: Option<Mode>,
    /// "base0D" unless the file says otherwise; the one non-standard key.
    pub accent: String,
    pub palette: [Rgb; 16],
}

impl Base16Theme {
    pub fn mode(&self) -> Mode {
        self.variant
            .unwrap_or_else(|| Mode::of_ground(self.palette[0]))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Portal {
    pub scheme: Option<Mode>,
    pub contrast: bool,
    pub accent: Option<Rgb>,
}

/// The token set. `scrim` is `bg` at `scrim_alpha`, so it is not a separate colour.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub bg: Rgb,
    pub surface: Rgb,
    pub surface_raised: Rgb,
    pub surface_sunken: Rgb,
    pub surface_pressed: Rgb,
    pub line: Rgb,
    pub line_strong: Rgb,
    pub text: Rgb,
    pub text_dim: Rgb,
    pub text_faint: Rgb,
    pub accent: Rgb,
    pub accent_text: Rgb,
    pub accent_soft: Rgb,
    pub red_soft: Rgb,
    pub focus: Rgb,
    pub red: Rgb,
    pub orange: Rgb,
    pub yellow: Rgb,
    pub green: Rgb,
    pub cyan: Rgb,
    pub blue: Rgb,
    pub purple: Rgb,
    pub brown: Rgb,
    pub scrim_alpha: f64,
    pub mode: Mode,
    pub source_label: String,
}

impl Palette {
    /// The bridge walks this to expose the token set to QML by name.
    pub const NAMES: [&'static str; 23] = [
        "bg",
        "surface",
        "surface.raised",
        "surface.sunken",
        "surface.pressed",
        "line",
        "line.strong",
        "text",
        "text.dim",
        "text.faint",
        "accent",
        "accent.text",
        "accent.soft",
        "red.soft",
        "focus",
        "red",
        "orange",
        "yellow",
        "green",
        "cyan",
        "blue",
        "purple",
        "brown",
    ];

    pub fn get(&self, name: &str) -> Option<Rgb> {
        Some(match name {
            "bg" => self.bg,
            "surface" => self.surface,
            "surface.raised" => self.surface_raised,
            "surface.sunken" => self.surface_sunken,
            "surface.pressed" => self.surface_pressed,
            "line" => self.line,
            "line.strong" => self.line_strong,
            "text" => self.text,
            "text.dim" => self.text_dim,
            "text.faint" => self.text_faint,
            "accent" => self.accent,
            "accent.text" => self.accent_text,
            "accent.soft" => self.accent_soft,
            "red.soft" => self.red_soft,
            "focus" => self.focus,
            "red" => self.red,
            "orange" => self.orange,
            "yellow" => self.yellow,
            "green" => self.green,
            "cyan" => self.cyan,
            "blue" => self.blue,
            "purple" => self.purple,
            "brown" => self.brown,
            _ => return None,
        })
    }
}

/// The ten format colours as fixed mappings onto the hues (spec 4.2, the five open slots
/// settled in this plan's decisions).
pub fn format_hue(anilist_format: &str) -> &'static str {
    match anilist_format {
        "TV" | "TV_SHORT" => "cyan",
        "MOVIE" => "yellow",
        "OVA" => "purple",
        "ONA" => "green",
        "SPECIAL" => "red",
        "MUSIC" => "green",
        "MANGA" => "orange",
        "NOVEL" | "LIGHT_NOVEL" => "red",
        "ONE_SHOT" => "yellow",
        "VISUAL_NOVEL" => "purple",
        _ => "text.dim",
    }
}

/// The list status colours, spec 4.2. The argument is the contract's `ListStatus` name.
pub fn status_hue(list_status: &str) -> &'static str {
    match list_status {
        "Watching" => "accent",
        "Completed" => "blue",
        "Paused" => "yellow",
        "Dropped" => "red",
        "Planning" => "text.faint",
        "Repeating" => "purple",
        _ => "text.faint",
    }
}
