//! Theme files: tinted-theming base16 YAML (verbatim, plus the optional `accent` key) and
//! a kitty `.conf` as the second format. No YAML crate: a base16 file is flat.

use std::path::Path;

use crate::theme::kitty;
use crate::theme::{Base16Theme, Mode, TerminalPalette, colour::Rgb};

fn unquote(v: &str) -> String {
    let v = match v.find(" #") {
        Some(i) => &v[..i],
        None => v,
    };
    v.trim().trim_matches('"').trim_matches('\'').to_string()
}

pub fn parse(stem: &str, text: &str) -> Option<Base16Theme> {
    let mut name = stem.to_string();
    let mut variant = None;
    let mut accent = "base0D".to_string();
    let mut slots: [Option<Rgb>; 16] = [None; 16];
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = unquote(value);
        match key {
            "name" => name = value,
            "variant" => {
                variant = match value.as_str() {
                    "dark" => Some(Mode::Dark),
                    "light" => Some(Mode::Light),
                    _ => None,
                }
            }
            "accent" => accent = value,
            k if k.starts_with("base") && k.len() == 6 => {
                if let (Ok(i), Some(c)) = (usize::from_str_radix(&k[4..], 16), Rgb::hex(&value))
                    && i < 16
                {
                    slots[i] = Some(c);
                }
            }
            _ => {}
        }
    }
    let mut palette = [Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    }; 16];
    for (i, s) in slots.iter().enumerate() {
        palette[i] = (*s)?;
    }
    Some(Base16Theme {
        stem: stem.to_string(),
        name,
        variant,
        accent,
        palette,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum Theme {
    Base16(Base16Theme),
    Kitty(TerminalPalette),
}

impl Theme {
    pub fn stem(&self) -> &str {
        match self {
            Theme::Base16(t) => &t.stem,
            Theme::Kitty(k) => &k.source,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Theme::Base16(t) => &t.name,
            Theme::Kitty(k) => &k.source,
        }
    }

    pub fn mode(&self) -> Mode {
        match self {
            Theme::Base16(t) => t.mode(),
            Theme::Kitty(k) => Mode::of_ground(k.background),
        }
    }
}

fn kitty_theme(stem: &str, text: &str) -> Option<TerminalPalette> {
    let conf = kitty::parse_conf(text);
    let colour = |k: &str| conf.values.get(k).and_then(|v| Rgb::hex(v));
    let mut colors = [Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    }; 16];
    for (i, slot) in colors.iter_mut().enumerate() {
        *slot = colour(&format!("color{i}"))?;
    }
    Some(TerminalPalette {
        foreground: colour("foreground")?,
        background: colour("background")?,
        colors,
        source: stem.to_string(),
    })
}

pub fn load_dir(dir: &Path) -> Vec<Theme> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut out = Vec::new();
    for entry in rd.flatten() {
        let path = entry.path();
        let (Some(stem), Some(ext)) = (
            path.file_stem().and_then(|s| s.to_str()),
            path.extension().and_then(|e| e.to_str()),
        ) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let theme = match ext {
            "yaml" | "yml" => parse(stem, &text).map(Theme::Base16),
            "conf" => kitty_theme(stem, &text).map(Theme::Kitty),
            _ => None,
        };
        if let Some(t) = theme {
            out.push(t);
        }
    }
    out.sort_by(|a, b| a.stem().cmp(b.stem()));
    out
}

/// Built-ins first, then the user's directory, which replaces a built-in of the same stem.
pub fn load_all(builtin: &Path, user: &Path) -> Vec<Theme> {
    let mut all = load_dir(builtin);
    for t in load_dir(user) {
        match all.iter_mut().find(|b| b.stem() == t.stem()) {
            Some(slot) => *slot = t,
            None => all.push(t),
        }
    }
    all.sort_by(|a, b| a.stem().cmp(b.stem()));
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Mode;

    const MOCHA: &str = r##"system: "base16"
name: "Catppuccin Mocha"
author: "https://github.com/catppuccin/catppuccin"
variant: "dark"
palette:
  base00: "#1e1e2e" # base
  base01: "#181825" # mantle
  base02: "#313244" # surface0
  base03: "#45475a" # surface1
  base04: "#585b70" # surface2
  base05: "#cdd6f4" # text
  base06: "#f5e0dc" # rosewater
  base07: "#b4befe" # lavender
  base08: "#f38ba8" # red
  base09: "#fab387" # peach
  base0A: "#f9e2af" # yellow
  base0B: "#a6e3a1" # green
  base0C: "#94e2d5" # teal
  base0D: "#89b4fa" # blue
  base0E: "#cba6f7" # mauve
  base0F: "#f2cdcd" # flamingo
"##;

    #[test]
    fn parses_a_tinted_theming_file_and_the_accent_key() {
        let t = parse("catppuccin-mocha", MOCHA).unwrap();
        assert_eq!(t.name, "Catppuccin Mocha");
        assert_eq!(t.variant, Some(Mode::Dark));
        assert_eq!(t.accent, "base0D");
        assert_eq!(t.palette[0].to_hex(), "#1e1e2e");
        assert_eq!(t.palette[15].to_hex(), "#f2cdcd");
        let with_accent = format!("{MOCHA}accent: \"base0E\"\n");
        assert_eq!(parse("x", &with_accent).unwrap().accent, "base0E");
        let no_variant = MOCHA.replace("variant: \"dark\"\n", "");
        assert_eq!(parse("x", &no_variant).unwrap().mode(), Mode::Dark);
        assert!(
            parse("x", "name: nothing\n").is_none(),
            "sixteen slots are required"
        );
    }

    #[test]
    fn a_directory_loads_yaml_and_kitty_files_and_the_user_dir_overrides_by_stem() {
        let dir = tempfile::tempdir().unwrap();
        let builtin = dir.path().join("builtin");
        let user = dir.path().join("user");
        std::fs::create_dir_all(&builtin).unwrap();
        std::fs::create_dir_all(&user).unwrap();
        std::fs::write(builtin.join("catppuccin-mocha.yaml"), MOCHA).unwrap();
        std::fs::write(
            user.join("catppuccin-mocha.yaml"),
            MOCHA.replace("Catppuccin Mocha", "My Mocha"),
        )
        .unwrap();
        let mut kitty = String::from("foreground #e0def4\nbackground #191724\n");
        for i in 0..16 {
            kitty.push_str(&format!(
                "color{i} #{:02x}{:02x}{:02x}\n",
                i * 10,
                i * 10,
                i * 10
            ));
        }
        std::fs::write(user.join("rose.conf"), kitty).unwrap();
        std::fs::write(user.join("notes.txt"), "ignored").unwrap();
        let all = load_all(&builtin, &user);
        let stems: Vec<&str> = all.iter().map(|t| t.stem()).collect();
        assert_eq!(stems, vec!["catppuccin-mocha", "rose"]);
        assert_eq!(all[0].name(), "My Mocha");
        assert_eq!(all[1].mode(), Mode::Dark);
        assert!(matches!(all[1], Theme::Kitty(_)));
    }

    #[test]
    fn every_shipped_theme_parses() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("themes");
        let all = load_dir(&dir);
        assert_eq!(all.len(), 30, "thirty built-ins");
        assert!(
            all.iter()
                .any(|t| t.stem() == "anibeam-dark" && t.mode() == Mode::Dark)
        );
        assert!(
            all.iter()
                .any(|t| t.stem() == "anibeam-light" && t.mode() == Mode::Light)
        );
        assert!(all.iter().any(|t| t.stem() == "kanagawa-dragon"));
    }
}
