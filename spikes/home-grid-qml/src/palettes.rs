//! Every colour source the QML side can derive tokens from: the terminal palette, the
//! portal's appearance keys, and the built-in base16 themes embedded at build time.

use serde_json::{json, Map, Value};
use std::process::Command;

const THEMES: [(&str, &str); 6] = [
    ("anibeam-dark", include_str!("../themes/anibeam-dark.yaml")),
    ("anibeam-light", include_str!("../themes/anibeam-light.yaml")),
    ("catppuccin-mocha", include_str!("../themes/catppuccin-mocha.yaml")),
    ("catppuccin-latte", include_str!("../themes/catppuccin-latte.yaml")),
    ("gruvbox-dark-medium", include_str!("../themes/gruvbox-dark-medium.yaml")),
    ("tokyo-night-dark", include_str!("../themes/tokyo-night-dark.yaml")),
];

fn busctl_read(key: &str) -> Option<String> {
    let out = Command::new("timeout")
        .args([
            "2", "busctl", "--user", "call", "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop", "org.freedesktop.portal.Settings", "ReadOne",
            "ss", "org.freedesktop.appearance", key,
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn portal() -> Value {
    let scheme = match busctl_read("color-scheme").as_deref() {
        Some("v u 1") => "dark",
        Some("v u 2") => "light",
        _ => "unset",
    };
    let contrast = matches!(busctl_read("contrast").as_deref(), Some("v u 1"));
    let accent = busctl_read("accent-color").and_then(|s| {
        // `v (ddd) 0.2 0.5 0.9`
        let nums: Vec<f64> = s
            .split_whitespace()
            .filter_map(|t| t.parse::<f64>().ok())
            .collect();
        if nums.len() == 3 {
            let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            Some(format!("#{:02x}{:02x}{:02x}", c(nums[0]), c(nums[1]), c(nums[2])))
        } else {
            None
        }
    });
    json!({ "scheme": scheme, "contrast": contrast, "accent": accent })
}

fn yaml_value(raw: &str) -> String {
    let mut v = raw.trim();
    // Drop a trailing comment; a colour never contains a space before its `#`.
    if let Some(i) = v.find(" #") {
        v = v[..i].trim();
    }
    v.trim_matches('"').trim_matches('\'').to_string()
}

fn parse_theme(slug: &str, text: &str) -> Value {
    let mut name = slug.to_string();
    let mut variant = String::new();
    let mut accent = "base0D".to_string();
    let mut palette = Map::new();
    let mut in_palette = false;
    for raw in text.lines() {
        let line = raw.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let indented = line.starts_with(' ') || line.starts_with('\t');
        if !indented {
            in_palette = false;
        }
        let Some((k, v)) = line.trim().split_once(':') else { continue };
        let (k, v) = (k.trim(), yaml_value(v));
        if !indented {
            match k {
                "palette" => in_palette = true,
                "name" => name = v,
                "variant" => variant = v,
                "accent" => accent = v,
                _ => {}
            }
        } else if in_palette && k.starts_with("base") {
            palette.insert(k.to_string(), Value::String(v));
        }
    }
    if variant.is_empty() {
        variant = palette
            .get("base00")
            .and_then(Value::as_str)
            .map(|h| if lightness(h) < 0.5 { "dark" } else { "light" })
            .unwrap_or("dark")
            .to_string();
    }
    json!({ "slug": slug, "name": name, "variant": variant, "accent": accent, "palette": palette })
}

fn lightness(hex: &str) -> f64 {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return 0.0;
    }
    let c = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0) as f64 / 255.0;
    0.2126 * c(0) + 0.7152 * c(2) + 0.0722 * c(4)
}

pub fn load() -> Value {
    let themes: Vec<Value> = THEMES.iter().map(|(slug, text)| parse_theme(slug, text)).collect();
    json!({
        "terminal": crate::kitty::load(),
        "portal": portal(),
        "themes": themes,
    })
}
