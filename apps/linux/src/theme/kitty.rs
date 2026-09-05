//! The kitty config chain, spec 4.2: the root file, then `include`, `globinclude` and
//! `envinclude` in order, last write wins, so the palette is what kitty itself would
//! paint. The files the chain touched are what the engine watches.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::theme::TerminalPalette;
use crate::theme::colour::Rgb;

const MAX_DEPTH: usize = 16;
const KEYS: [&str; 18] = [
    "foreground",
    "background",
    "color0",
    "color1",
    "color2",
    "color3",
    "color4",
    "color5",
    "color6",
    "color7",
    "color8",
    "color9",
    "color10",
    "color11",
    "color12",
    "color13",
    "color14",
    "color15",
];

/// An enumerable environment: `envinclude`'s prefix match needs every variable name, not
/// just point lookups, so a plain closure cannot stand in for the process environment.
pub trait Env {
    fn get(&self, key: &str) -> Option<String>;
    /// Every variable name, for envinclude's prefix match.
    fn names(&self) -> Vec<String>;
}

pub struct ProcessEnv;

impl Env for ProcessEnv {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn names(&self) -> Vec<String> {
        std::env::vars_os()
            .filter_map(|(k, _)| k.into_string().ok())
            .collect()
    }
}

impl Env for HashMap<String, String> {
    fn get(&self, key: &str) -> Option<String> {
        HashMap::get(self, key).cloned()
    }

    fn names(&self) -> Vec<String> {
        self.keys().cloned().collect()
    }
}

/// One line of a config file that matters to us, in file order: `walk` replays these in
/// sequence so a line after an include wins over anything the include set, exactly as
/// kitty itself would apply them.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Assign(String, String),
    Directive(String, String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Conf {
    pub values: HashMap<String, String>,
    pub directives: Vec<(String, String)>,
    pub entries: Vec<Entry>,
}

pub fn parse_conf(text: &str) -> Conf {
    let mut c = Conf::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, rest)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let value = rest.trim();
        match key {
            "include" | "globinclude" | "envinclude" => {
                c.directives.push((key.to_string(), value.to_string()));
                c.entries
                    .push(Entry::Directive(key.to_string(), value.to_string()));
            }
            k if KEYS.contains(&k) => {
                c.values.insert(k.to_string(), value.to_string());
                c.entries
                    .push(Entry::Assign(k.to_string(), value.to_string()));
            }
            _ => {}
        }
    }
    c
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chain {
    pub files: Vec<PathBuf>,
    pub palette: Option<TerminalPalette>,
}

fn expand(path: &str, base: &Path, env: &dyn Env) -> PathBuf {
    let p = if let Some(rest) = path.strip_prefix("~/") {
        PathBuf::from(env.get("HOME").unwrap_or_default()).join(rest)
    } else {
        PathBuf::from(path)
    };
    if p.is_absolute() { p } else { base.join(p) }
}

fn glob_paths(pattern: &Path) -> Vec<PathBuf> {
    // One directory level of `*` is what kitty configs use; anything deeper is a plain path.
    let Some(parent) = pattern.parent() else {
        return vec![pattern.to_path_buf()];
    };
    let Some(name) = pattern.file_name().and_then(|n| n.to_str()) else {
        return vec![];
    };
    let Some((prefix, suffix)) = name.split_once('*') else {
        return vec![pattern.to_path_buf()];
    };
    let Ok(rd) = std::fs::read_dir(parent) else {
        return vec![];
    };
    let mut out: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(prefix) && n.ends_with(suffix))
        })
        .collect();
    out.sort();
    out
}

fn walk(
    path: &Path,
    depth: usize,
    values: &mut HashMap<String, String>,
    files: &mut Vec<PathBuf>,
    env: &dyn Env,
) {
    files.push(path.to_path_buf());
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let conf = parse_conf(&text);
    let base = path.parent().unwrap_or(Path::new("/"));
    // Replayed in file order, not batched: a plain assignment applies immediately, and a
    // directive recurses right where it sits, so a line after an include still wins.
    for entry in conf.entries {
        match entry {
            Entry::Assign(k, v) => {
                values.insert(k, v);
            }
            Entry::Directive(kind, arg) => match kind.as_str() {
                "include" => walk(&expand(&arg, base, env), depth + 1, values, files, env),
                "globinclude" => {
                    for p in glob_paths(&expand(&arg, base, env)) {
                        walk(&p, depth + 1, values, files, env);
                    }
                }
                "envinclude" => {
                    let prefix = arg.trim_end_matches('*');
                    let mut names: Vec<String> = env
                        .names()
                        .into_iter()
                        .filter(|k| k.starts_with(prefix))
                        .collect();
                    names.sort();
                    for name in names {
                        if let Some(text) = env.get(&name) {
                            for (k, v) in parse_conf(&text).values {
                                values.insert(k, v);
                            }
                        }
                    }
                }
                _ => {}
            },
        }
    }
}

/// Foreground, background and all sixteen `colorN` slots from a resolved key/value map,
/// or `None` if any of the eighteen is missing. Shared with `base16::kitty_theme`, which
/// builds the same shape from a standalone kitty `.conf` theme file.
pub fn palette_from(values: &HashMap<String, String>, source: &str) -> Option<TerminalPalette> {
    let colour = |k: &str| values.get(k).and_then(|v| Rgb::hex(v));
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
        source: source.to_string(),
    })
}

pub fn read_chain(root: &Path, env: &dyn Env) -> Chain {
    let mut values = HashMap::new();
    let mut files = Vec::new();
    walk(root, 0, &mut values, &mut files, env);
    let palette = palette_from(&values, "kitty");
    Chain { files, palette }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terminal {
    Kitty,
    Foot,
    Alacritty,
    Ghostty,
}

fn config_home(env: &dyn Env) -> PathBuf {
    env.get("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env.get("HOME").unwrap_or_default()).join(".config"))
}

pub fn root_config(env: &dyn Env) -> PathBuf {
    match env.get("KITTY_CONFIG_DIRECTORY") {
        Some(dir) => PathBuf::from(dir).join("kitty.conf"),
        None => config_home(env).join("kitty").join("kitty.conf"),
    }
}

/// `$TERMINAL` first, then config presence in the order kitty, foot, alacritty, ghostty.
/// Only kitty has a parser today; the other three make the engine fall back to the portal.
pub fn probe(env: &dyn Env) -> Option<Terminal> {
    if let Some(t) = env.get("TERMINAL") {
        let name = Path::new(&t)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&t);
        return match name {
            "kitty" => Some(Terminal::Kitty),
            "foot" | "footclient" => Some(Terminal::Foot),
            "alacritty" => Some(Terminal::Alacritty),
            "ghostty" => Some(Terminal::Ghostty),
            _ => None,
        };
    }
    let home = config_home(env);
    let present = |rel: &str| home.join(rel).is_file();
    if root_config(env).is_file() {
        Some(Terminal::Kitty)
    } else if present("foot/foot.ini") {
        Some(Terminal::Foot)
    } else if present("alacritty/alacritty.toml") {
        Some(Terminal::Alacritty)
    } else if present("ghostty/config") {
        Some(Terminal::Ghostty)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_of(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn parse_conf_keeps_the_last_value_and_the_directives_in_order() {
        let text = "# comment\nforeground #e0def4\nbackground   #191724\ncolor0 #26233a\ncolor0 #000\ninclude theme.conf\nglobinclude parts/*.conf\nenvinclude KITTY_CONF_*\nfont_size 12\n";
        let c = parse_conf(text);
        assert_eq!(
            c.values.get("foreground").map(String::as_str),
            Some("#e0def4")
        );
        assert_eq!(c.values.get("color0").map(String::as_str), Some("#000"));
        assert!(!c.values.contains_key("font_size"));
        assert_eq!(
            c.directives,
            vec![
                ("include".to_string(), "theme.conf".to_string()),
                ("globinclude".to_string(), "parts/*.conf".to_string()),
                ("envinclude".to_string(), "KITTY_CONF_*".to_string()),
            ]
        );
    }

    #[test]
    fn the_chain_follows_includes_and_the_last_write_wins() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("kitty.conf");
        std::fs::create_dir_all(dir.path().join("parts")).unwrap();
        std::fs::write(&root, "background #111111\nforeground #eeeeee\ninclude parts/a.conf\nglobinclude parts/b*.conf\nenvinclude KITTY_TEST_*\ncolor15 #ffffff\n").unwrap();
        std::fs::write(dir.path().join("parts/a.conf"), "background #222222\ncolor0 #000000\ncolor1 #ff0000\ncolor2 #00ff00\ncolor3 #ffff00\ncolor4 #0000ff\ncolor5 #ff00ff\ncolor6 #00ffff\ncolor7 #cccccc\ncolor8 #444444\ncolor9 #ff8888\ncolor10 #88ff88\ncolor11 #ffff88\ncolor12 #8888ff\ncolor13 #ff88ff\ncolor14 #88ffff\n").unwrap();
        std::fs::write(dir.path().join("parts/b1.conf"), "color4 #123456\n").unwrap();
        let env = env_of(&[("KITTY_TEST_ONE", "color5 #654321")]);
        let chain = read_chain(&root, &env);
        let p = chain.palette.expect("a full palette");
        assert_eq!(
            p.background.to_hex(),
            "#222222",
            "the include wrote after the root"
        );
        assert_eq!(
            p.colors[4].to_hex(),
            "#123456",
            "the glob wrote after the include"
        );
        assert_eq!(
            p.colors[5].to_hex(),
            "#654321",
            "the env include wrote last"
        );
        assert_eq!(p.colors[15].to_hex(), "#ffffff");
        assert_eq!(chain.files.len(), 3);
        assert_eq!(chain.files[0], root);
    }

    #[test]
    fn a_line_after_an_include_wins_over_the_include() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("kitty.conf");
        std::fs::write(
            dir.path().join("other.conf"),
            "background #bbbbbb\nforeground #eeeeee\n",
        )
        .unwrap();
        std::fs::write(
            &root,
            "background #aaaaaa\ninclude other.conf\nbackground #cccccc\n",
        )
        .unwrap();
        let mut colours = String::new();
        for i in 0..16 {
            colours.push_str(&format!("color{i} #{:02x}{:02x}{:02x}\n", i, i, i));
        }
        std::fs::write(
            dir.path().join("other.conf"),
            format!("background #bbbbbb\nforeground #eeeeee\n{colours}"),
        )
        .unwrap();
        let chain = read_chain(&root, &std::collections::HashMap::new());
        assert_eq!(chain.palette.unwrap().background.to_hex(), "#cccccc");
    }

    #[test]
    fn a_missing_slot_means_no_palette_but_the_files_are_still_listed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("kitty.conf");
        std::fs::write(
            &root,
            "background #111111\nforeground #eeeeee\ninclude gone.conf\n",
        )
        .unwrap();
        let chain = read_chain(&root, &HashMap::new());
        assert!(chain.palette.is_none());
        assert_eq!(
            chain.files,
            vec![root.clone(), dir.path().join("gone.conf")]
        );
    }

    #[test]
    fn tilde_and_relative_paths_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(home.join(".config/theme")).unwrap();
        std::fs::write(home.join(".config/theme/k.conf"), "color9 #ff9999\n").unwrap();
        let root = dir.path().join("kitty.conf");
        std::fs::write(&root, "include ~/.config/theme/k.conf\n").unwrap();
        let env = env_of(&[("HOME", home.to_str().unwrap())]);
        let chain = read_chain(&root, &env);
        assert_eq!(chain.files[1], home.join(".config/theme/k.conf"));
    }

    #[test]
    fn probe_prefers_terminal_then_config_presence() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("config");
        std::fs::create_dir_all(cfg.join("foot")).unwrap();
        std::fs::write(cfg.join("foot/foot.ini"), "").unwrap();
        let env = env_of(&[
            ("XDG_CONFIG_HOME", cfg.to_str().unwrap()),
            ("TERMINAL", "kitty"),
        ]);
        assert_eq!(probe(&env), Some(Terminal::Kitty));
        let env = env_of(&[("XDG_CONFIG_HOME", cfg.to_str().unwrap())]);
        assert_eq!(probe(&env), Some(Terminal::Foot));
        assert_eq!(root_config(&env), cfg.join("kitty/kitty.conf"));
    }
}
