//! Resolve the inputs into both modes' palettes, and keep them current: notify on the
//! kitty chain, the theme directories and theme.toml; the portal's SettingChanged; and the
//! settings the Theme singleton writes. Nothing here is a rule about the library.

use std::path::PathBuf;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use tokio::sync::mpsc;

use crate::paths::ShellPaths;
use crate::theme::base16::{self, Theme as ThemeFile};
use crate::theme::config::{self, ModeSetting, Source, ThemeSettings};
use crate::theme::{Mode, Palette, Portal, Steps, TerminalPalette, kitty, portal, tokens};

/// What `new_debouncer` hands back; named so the rewatch closure can be typed.
type Watchers = notify_debouncer_full::Debouncer<
    notify::RecommendedWatcher,
    notify_debouncer_full::RecommendedCache,
>;

#[derive(Clone, Debug, PartialEq)]
pub struct Inputs {
    pub settings: ThemeSettings,
    pub terminal: Option<TerminalPalette>,
    pub terminal_files: Vec<PathBuf>,
    pub portal: Portal,
    pub themes: Vec<ThemeFile>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    pub inputs: Inputs,
    pub dark: Palette,
    pub light: Palette,
    pub mode: Mode,
}

fn palette_for(i: &Inputs, mode: Mode) -> Palette {
    let steps = Steps::default();
    let contrast = i.portal.contrast;
    let accent = i.settings.accent;
    let from_theme = |stem: &str| -> Option<Palette> {
        i.themes.iter().find(|t| t.stem() == stem).map(|t| match t {
            ThemeFile::Base16(b) => tokens::from_base16(b, contrast, &steps),
            ThemeFile::Kitty(k) => tokens::from_terminal(k, mode, accent, contrast, &steps),
        })
    };
    match i.settings.source {
        Source::System => match &i.terminal {
            Some(term) => tokens::from_terminal(term, mode, accent, contrast, &steps),
            None => tokens::from_portal(&i.portal, mode, contrast, &steps),
        },
        Source::Theme => {
            let (stem, builtin) = match mode {
                Mode::Dark => (i.settings.theme_dark.as_str(), "anibeam-dark"),
                Mode::Light => (i.settings.theme_light.as_str(), "anibeam-light"),
            };
            from_theme(stem)
                .or_else(|| from_theme(builtin))
                .unwrap_or_else(|| tokens::from_portal(&i.portal, mode, contrast, &steps))
        }
    }
}

pub fn resolve(inputs: Inputs) -> Resolved {
    let system = match &inputs.terminal {
        Some(t) => Mode::of_ground(t.background),
        None => inputs.portal.scheme.unwrap_or(Mode::Dark),
    };
    let mode = match inputs.settings.mode {
        ModeSetting::System => system,
        ModeSetting::Dark => Mode::Dark,
        ModeSetting::Light => Mode::Light,
    };
    let dark = palette_for(&inputs, Mode::Dark);
    let light = palette_for(&inputs, Mode::Light);
    Resolved {
        inputs,
        dark,
        light,
        mode,
    }
}

pub fn read_inputs(paths: &ShellPaths, portal: Portal) -> Inputs {
    let env = kitty::ProcessEnv;
    let settings = config::load(&paths.theme_toml());
    let (terminal, terminal_files) = match kitty::probe(&env) {
        Some(kitty::Terminal::Kitty) => {
            let chain = kitty::read_chain(&kitty::root_config(&env), &env);
            (chain.palette, chain.files)
        }
        _ => (None, vec![]),
    };
    let themes = base16::load_all(&paths.builtin_themes_dir(), &paths.user_themes_dir());
    Inputs {
        settings,
        terminal,
        terminal_files,
        portal,
        themes,
    }
}

/// A wedged portal must not leave the shell without colours: every bus call is bounded and
/// a timeout counts as "the portal said nothing".
const BUS_TIMEOUT: Duration = Duration::from_secs(3);

/// The two directories under config the engine watches. A directory that does not exist
/// cannot be watched, and nothing else creates them before the first pick, so `run` makes
/// them once. The built-in directory belongs to the package and is never created here.
fn ensure_dirs(paths: &ShellPaths) {
    for d in [paths.config_dir(), paths.user_themes_dir()] {
        if let Err(e) = std::fs::create_dir_all(&d) {
            eprintln!("anibeam: {}: {e}", d.display());
        }
    }
}

/// Every directory the engine watches, sorted and deduped: the config directory (theme.toml
/// is replaced by rename on save, so the directory is the watch), both theme directories,
/// and the directory of every file the kitty chain touched.
fn watch_dirs(paths: &ShellPaths, inputs: &Inputs) -> Vec<PathBuf> {
    let mut wanted = vec![
        paths.config_dir(),
        paths.user_themes_dir(),
        paths.builtin_themes_dir(),
    ];
    wanted.extend(
        inputs
            .terminal_files
            .iter()
            .filter_map(|f| f.parent().map(PathBuf::from)),
    );
    wanted.sort();
    wanted.dedup();
    wanted
}

/// The engine's loop. `push` receives every new resolution on whatever thread produced it;
/// the bridge hops it to the Qt thread. `commands` carries settings the singleton wrote.
///
/// The first push happens before the bus is touched: xdg-desktop-portal may need D-Bus
/// activation, and the window's ground must not wait on it. The portal's answer, bounded by
/// `BUS_TIMEOUT`, arrives as a second push when it changes anything.
pub async fn run(
    paths: ShellPaths,
    push: impl Fn(Resolved) + Send + Sync + 'static,
    mut commands: mpsc::UnboundedReceiver<ThemeSettings>,
) {
    ensure_dirs(&paths);
    let (wake_tx, mut wake_rx) = mpsc::unbounded_channel::<()>();

    // Files: the chain, both theme directories, theme.toml's directory. The debouncer's own
    // thread sends a wake; the loop re-reads. Rewatching is unconditional, because a
    // directory that did not exist at start-up becomes watchable the moment it appears.
    let mut watched: Vec<PathBuf> = Vec::new();
    let wake = wake_tx.clone();
    let mut debouncer = new_debouncer(Duration::from_millis(200), None, move |_res| {
        let _ = wake.send(());
    })
    .ok();
    let rewatch =
        |debouncer: &mut Option<Watchers>, watched: &mut Vec<PathBuf>, inputs: &Inputs| {
            let Some(d) = debouncer else { return };
            for p in watched.drain(..) {
                let _ = d.unwatch(&p);
            }
            for p in watch_dirs(&paths, inputs) {
                if p.is_dir() && d.watch(&p, RecursiveMode::NonRecursive).is_ok() {
                    watched.push(p);
                }
            }
        };
    let mut inputs = read_inputs(&paths, Portal::default());
    rewatch(&mut debouncer, &mut watched, &inputs);
    push(resolve(inputs.clone()));

    let conn = tokio::time::timeout(BUS_TIMEOUT, zbus::Connection::session())
        .await
        .ok()
        .and_then(Result::ok);
    if let Some(c) = &conn {
        let state = tokio::time::timeout(BUS_TIMEOUT, portal::read(c))
            .await
            .unwrap_or_default();
        if state != inputs.portal {
            inputs.portal = state;
            push(resolve(inputs.clone()));
        }
    }

    if let Some(c) = conn.clone() {
        let wake = wake_tx.clone();
        tokio::spawn(portal::watch(c, move || {
            let _ = wake.send(());
        }));
    }

    loop {
        tokio::select! {
            Some(settings) = commands.recv() => {
                if let Err(e) = config::save(&paths.theme_toml(), &settings) {
                    eprintln!("anibeam: theme.toml: {e}");
                }
                inputs.settings = settings;
                rewatch(&mut debouncer, &mut watched, &inputs);
                push(resolve(inputs.clone()));
            }
            Some(()) = wake_rx.recv() => {
                let portal_state = match &conn {
                    Some(c) => tokio::time::timeout(BUS_TIMEOUT, portal::read(c)).await.unwrap_or_default(),
                    None => Portal::default(),
                };
                let fresh = read_inputs(&paths, portal_state);
                rewatch(&mut debouncer, &mut watched, &fresh);
                if fresh != inputs {
                    inputs = fresh;
                    push(resolve(inputs.clone()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::ShellPaths;
    use crate::theme::base16::Theme as ThemeFile;
    use crate::theme::colour::Rgb;
    use crate::theme::config::{ModeSetting, Source, ThemeSettings};
    use crate::theme::{Base16Theme, Mode, Portal, TerminalPalette};

    fn term(bg: &str) -> TerminalPalette {
        let c = Rgb::hex("#888888").unwrap();
        TerminalPalette {
            foreground: Rgb::hex("#e0def4").unwrap(),
            background: Rgb::hex(bg).unwrap(),
            colors: [c; 16],
            source: "kitty".into(),
        }
    }

    fn theme(stem: &str, mode: Mode) -> ThemeFile {
        let (bg, fg) = match mode {
            Mode::Dark => ("#101010", "#f0f0f0"),
            Mode::Light => ("#f0f0f0", "#101010"),
        };
        let mut palette = [Rgb::hex("#777777").unwrap(); 16];
        palette[0] = Rgb::hex(bg).unwrap();
        palette[5] = Rgb::hex(fg).unwrap();
        ThemeFile::Base16(Base16Theme {
            stem: stem.into(),
            name: stem.into(),
            variant: Some(mode),
            accent: "base0D".into(),
            palette,
        })
    }

    fn inputs() -> Inputs {
        Inputs {
            settings: ThemeSettings::default(),
            terminal: Some(term("#191724")),
            terminal_files: vec![],
            portal: Portal {
                scheme: Some(Mode::Light),
                contrast: false,
                accent: None,
            },
            themes: vec![
                theme("anibeam-dark", Mode::Dark),
                theme("anibeam-light", Mode::Light),
                theme("mocha", Mode::Dark),
            ],
        }
    }

    #[test]
    fn system_mode_follows_the_terminal_before_the_portal() {
        let r = resolve(inputs());
        assert_eq!(
            r.mode,
            Mode::Dark,
            "the terminal is dark, the portal says light"
        );
        assert_eq!(r.dark.bg.to_hex(), "#191724");
        assert_eq!(
            r.light.mode,
            Mode::Light,
            "the other mode is still resolved for the preview"
        );
        let mut no_terminal = inputs();
        no_terminal.terminal = None;
        let r = resolve(no_terminal);
        assert_eq!(r.mode, Mode::Light, "with no terminal the portal decides");
        assert!(r.light.source_label.starts_with("portal"));
    }

    #[test]
    fn a_forced_mode_and_a_theme_pair() {
        let mut i = inputs();
        i.settings.mode = ModeSetting::Light;
        assert_eq!(resolve(i.clone()).mode, Mode::Light);
        i.settings.source = Source::Theme;
        i.settings.theme_dark = "mocha".into();
        let r = resolve(i.clone());
        assert_eq!(r.dark.source_label, "theme mocha");
        assert_eq!(r.light.source_label, "theme anibeam-light");
        i.settings.theme_light = "missing".into();
        let r = resolve(i);
        assert_eq!(
            r.light.source_label, "theme anibeam-light",
            "a missing stem falls back to the built-in"
        );
    }

    #[test]
    fn a_kitty_theme_file_is_a_terminal_palette() {
        let mut i = inputs();
        i.settings.source = Source::Theme;
        i.settings.theme_dark = "rose".into();
        i.themes.push(ThemeFile::Kitty(TerminalPalette {
            source: "rose".into(),
            ..term("#191724")
        }));
        let r = resolve(i);
        assert_eq!(r.dark.source_label, "terminal rose");
        assert_eq!(r.dark.bg.to_hex(), "#191724");
    }

    #[test]
    fn the_watch_set_covers_config_and_the_chain_once_the_directories_exist() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ShellPaths::resolve(Some(dir.path())).unwrap();
        assert!(!paths.config_dir().is_dir(), "nothing has made it yet");
        ensure_dirs(&paths);
        assert!(
            paths.config_dir().is_dir(),
            "theme.toml's directory is watchable"
        );
        assert!(paths.user_themes_dir().is_dir());

        let mut i = inputs();
        i.terminal_files = vec![
            PathBuf::from("/home/x/.config/kitty/kitty.conf"),
            PathBuf::from("/home/x/.config/kitty/theme.conf"),
        ];
        let dirs = watch_dirs(&paths, &i);
        assert!(dirs.contains(&paths.config_dir()));
        assert!(dirs.contains(&paths.user_themes_dir()));
        assert!(dirs.contains(&paths.builtin_themes_dir()));
        assert_eq!(
            dirs.iter().filter(|p| p.ends_with("kitty")).count(),
            1,
            "one entry per directory, not per file of the chain"
        );
        let mut sorted = dirs.clone();
        sorted.sort();
        assert_eq!(dirs, sorted, "sorted, so the watch set is stable");
    }
}
