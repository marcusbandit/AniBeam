//! theme.toml: the theme settings, written through toml_edit so hand edits and comments
//! survive. Every key has a default; a missing or invalid key takes it.

use std::path::Path;

use toml_edit::{DocumentMut, Item, Table, value};

macro_rules! choice {
    ($name:ident { $($variant:ident = $s:literal),+ $(,)? }, default $default:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub fn as_str(self) -> &'static str { match self { $($name::$variant => $s),+ } }
            pub fn parse(s: &str) -> Option<Self> { match s { $($s => Some($name::$variant),)+ _ => None } }
        }
        impl Default for $name { fn default() -> Self { $name::$default } }
    };
}

choice!(ModeSetting { Dark = "dark", Light = "light", System = "system" }, default System);
choice!(Source { System = "system", Theme = "theme" }, default System);
choice!(Density { Compact = "compact", Normal = "normal", Comfortable = "comfortable" }, default Normal);
choice!(Poster { S = "s", M = "m", L = "l" }, default M);
choice!(Corners { Smooth = "smooth", Plain = "plain" }, default Smooth);

impl Density {
    pub fn factor(self) -> f64 {
        match self {
            Density::Compact => 0.75,
            Density::Normal => 1.0,
            Density::Comfortable => 1.25,
        }
    }
}

impl Poster {
    pub fn width(self) -> i32 {
        match self {
            Poster::S => 140,
            Poster::M => 180,
            Poster::L => 240,
        }
    }
}

impl Corners {
    pub fn smoothing(self) -> f64 {
        match self {
            Corners::Smooth => 0.6,
            Corners::Plain => 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeSettings {
    pub mode: ModeSetting,
    pub source: Source,
    pub accent: u8,
    pub density: Density,
    pub poster: Poster,
    pub corners: Corners,
    pub theme_dark: String,
    pub theme_light: String,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        ThemeSettings {
            mode: ModeSetting::System,
            source: Source::System,
            accent: 4,
            density: Density::Normal,
            poster: Poster::M,
            corners: Corners::Smooth,
            theme_dark: "anibeam-dark".into(),
            theme_light: "anibeam-light".into(),
        }
    }
}

fn str_of(doc: &DocumentMut, key: &str) -> Option<String> {
    doc.get(key).and_then(Item::as_str).map(String::from)
}

pub fn load(path: &Path) -> ThemeSettings {
    let d = ThemeSettings::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        return d;
    };
    let Ok(doc) = text.parse::<DocumentMut>() else {
        return d;
    };
    let theme = doc.get("theme").and_then(Item::as_table);
    let pair = |k: &str, default: &str| {
        theme
            .and_then(|t| t.get(k))
            .and_then(Item::as_str)
            .map(String::from)
            .unwrap_or_else(|| default.to_string())
    };
    ThemeSettings {
        mode: str_of(&doc, "mode")
            .and_then(|s| ModeSetting::parse(&s))
            .unwrap_or(d.mode),
        source: str_of(&doc, "source")
            .and_then(|s| Source::parse(&s))
            .unwrap_or(d.source),
        accent: doc
            .get("accent")
            .and_then(Item::as_integer)
            .filter(|a| (1..=7).contains(a))
            .map(|a| a as u8)
            .unwrap_or(d.accent),
        density: str_of(&doc, "density")
            .and_then(|s| Density::parse(&s))
            .unwrap_or(d.density),
        poster: str_of(&doc, "poster")
            .and_then(|s| Poster::parse(&s))
            .unwrap_or(d.poster),
        corners: str_of(&doc, "corners")
            .and_then(|s| Corners::parse(&s))
            .unwrap_or(d.corners),
        theme_dark: pair("dark", &d.theme_dark),
        theme_light: pair("light", &d.theme_light),
    }
}

pub fn save(path: &Path, s: &ThemeSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut doc = std::fs::read_to_string(path)
        .ok()
        .and_then(|t| t.parse::<DocumentMut>().ok())
        .unwrap_or_default();
    doc["mode"] = value(s.mode.as_str());
    doc["source"] = value(s.source.as_str());
    doc["accent"] = value(i64::from(s.accent));
    doc["density"] = value(s.density.as_str());
    doc["poster"] = value(s.poster.as_str());
    doc["corners"] = value(s.corners.as_str());
    if !doc.contains_table("theme") {
        doc["theme"] = Item::Table(Table::new());
    }
    doc["theme"]["dark"] = value(s.theme_dark.as_str());
    doc["theme"]["light"] = value(s.theme_light.as_str());
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, doc.to_string())?;
    std::fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_missing_and_per_key_when_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("theme.toml");
        assert_eq!(load(&path), ThemeSettings::default());
        std::fs::write(&path, "mode = \"purple\"\naccent = 9\ndensity = \"compact\"\n[theme]\ndark = \"gruvbox-dark-medium\"\n").unwrap();
        let s = load(&path);
        assert_eq!(s.mode, ModeSetting::System, "an unknown mode falls back");
        assert_eq!(s.accent, 4, "an out of range accent falls back");
        assert_eq!(s.density, Density::Compact);
        assert_eq!(s.theme_dark, "gruvbox-dark-medium");
        assert_eq!(s.theme_light, "anibeam-light");
    }

    #[test]
    fn save_keeps_comments_and_unknown_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/theme.toml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "# my notes\nmode = \"dark\"\nfuture_key = 1\n\n[theme]\n# pair\ndark = \"anibeam-dark\"\n").unwrap();
        let mut s = load(&path);
        s.mode = ModeSetting::Light;
        s.poster = Poster::L;
        s.theme_light = "catppuccin-latte".into();
        save(&path, &s).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# my notes"));
        assert!(text.contains("future_key = 1"));
        assert!(text.contains("# pair"));
        assert!(text.contains("mode = \"light\""));
        assert!(text.contains("poster = \"l\""));
        assert!(text.contains("light = \"catppuccin-latte\""));
        assert_eq!(load(&path), s);
    }

    #[test]
    fn save_creates_the_file_and_the_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new/theme.toml");
        save(&path, &ThemeSettings::default()).unwrap();
        assert_eq!(load(&path), ThemeSettings::default());
    }
}
