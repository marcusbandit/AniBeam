//! The terminal palette, resolved from kitty's config chain the way the theme ticket
//! describes: include, globinclude and envinclude in order, last write wins. Read only.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const KEYS: [&str; 18] = [
    "foreground", "background", "color0", "color1", "color2", "color3", "color4", "color5",
    "color6", "color7", "color8", "color9", "color10", "color11", "color12", "color13",
    "color14", "color15",
];

fn expand_tilde(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        crate::library::home().join(rest)
    } else if p == "~" {
        crate::library::home()
    } else {
        PathBuf::from(p)
    }
}

fn resolve(base: &Path, p: &str) -> PathBuf {
    let path = expand_tilde(p);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

struct Walk {
    values: HashMap<String, String>,
    files: Vec<String>,
    depth: usize,
}

impl Walk {
    fn read_file(&mut self, path: &Path) {
        if self.depth > 16 {
            return;
        }
        let Ok(text) = std::fs::read_to_string(path) else { return };
        self.files.push(path.to_string_lossy().into_owned());
        let base = path.parent().map(Path::to_path_buf).unwrap_or_default();
        self.depth += 1;
        self.parse(&text, &base);
        self.depth -= 1;
    }

    fn parse(&mut self, text: &str, base: &Path) {
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = match line.split_once(char::is_whitespace) {
                Some((k, v)) => (k, v.trim()),
                None => (line, ""),
            };
            match key {
                "include" => {
                    let p = resolve(base, value);
                    self.read_file(&p);
                }
                "globinclude" => {
                    let pattern = resolve(base, value);
                    if let Ok(paths) = glob::glob(&pattern.to_string_lossy()) {
                        for p in paths.flatten() {
                            self.read_file(&p);
                        }
                    }
                }
                "envinclude" => {
                    if let Ok(v) = std::env::var(value) {
                        self.depth += 1;
                        if self.depth <= 16 {
                            self.parse(&v, base);
                        }
                        self.depth -= 1;
                    }
                }
                k if KEYS.contains(&k) => {
                    if !value.is_empty() {
                        self.values.insert(k.to_string(), value.to_string());
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn load() -> Value {
    let root = std::env::var_os("KITTY_CONFIG_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::library::home().join(".config").join("kitty"))
        .join("kitty.conf");
    if !root.is_file() {
        return Value::Null;
    }
    let mut walk = Walk { values: HashMap::new(), files: vec![], depth: 0 };
    walk.read_file(&root);

    if KEYS.iter().any(|k| !walk.values.contains_key(*k)) {
        return Value::Null;
    }
    let colors: Vec<Value> = (0..16)
        .map(|i| Value::String(walk.values[&format!("color{i}")].clone()))
        .collect();
    let mut out = json!({
        "source": root.to_string_lossy(),
        "files": walk.files,
        "foreground": walk.values["foreground"],
        "background": walk.values["background"],
        "colors": colors,
    });
    if let Ok(term) = std::env::var("TERMINAL") {
        if term != "kitty" {
            out["terminal"] = Value::String(term);
        }
    }
    out
}
