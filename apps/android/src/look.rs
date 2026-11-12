//! Appearance (#754): the theme, the accent and the text size. A pick is written to
//! `look.json` and applied to the `Theme` global at once, so every page re-tones where it
//! stands. The themes are a few palettes of the desktop's, cut to what a phone needs; the
//! glass over them (panes, hairlines, soft accent) the theme file derives from the ground
//! and the ink, as the desktop's Tokens do.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use slint::{Color, ComponentHandle, ModelRc, VecModel, Weak};

use crate::{AppWindow, LookData, LookTheme, Theme};

/// One palette: the ground, the three inks, the accent it ships with and the six hues the
/// accent can be moved to.
struct Palette {
    id: &'static str,
    name: &'static str,
    dark: bool,
    ground: u32,
    text: u32,
    dim: u32,
    faint: u32,
    accent: u32,
    /// Red, orange, yellow, green, cyan, blue.
    hues: [u32; 6],
    purple: u32,
}

const PALETTES: [Palette; 8] = [
    Palette {
        id: "graphite-dark",
        name: "Graphite Glass",
        dark: true,
        ground: 0x0e1013,
        text: 0xeef2f5,
        dim: 0x9aa3ad,
        faint: 0x646c75,
        accent: 0xf2f4f6,
        hues: [0xff7a4d, 0xf0955a, 0xe0b25a, 0x8cc58a, 0x6cc4c0, 0x7fb3d4],
        purple: 0xb49be0,
    },
    Palette {
        id: "tokyo-night",
        name: "Tokyo Night",
        dark: true,
        ground: 0x1a1b26,
        text: 0xc0caf5,
        dim: 0x9aa5ce,
        faint: 0x565f89,
        accent: 0x7aa2f7,
        hues: [0xf7768e, 0xff9e64, 0xe0af68, 0x9ece6a, 0x7dcfff, 0x7aa2f7],
        purple: 0xbb9af7,
    },
    Palette {
        id: "catppuccin-mocha",
        name: "Catppuccin Mocha",
        dark: true,
        ground: 0x1e1e2e,
        text: 0xcdd6f4,
        dim: 0xa6adc8,
        faint: 0x6c7086,
        accent: 0xcba6f7,
        hues: [0xf38ba8, 0xfab387, 0xf9e2af, 0xa6e3a1, 0x94e2d5, 0x89b4fa],
        purple: 0xcba6f7,
    },
    Palette {
        id: "nord",
        name: "Nord",
        dark: true,
        ground: 0x2e3440,
        text: 0xeceff4,
        dim: 0xc1c8d6,
        faint: 0x7b88a1,
        accent: 0x88c0d0,
        hues: [0xbf616a, 0xd08770, 0xebcb8b, 0xa3be8c, 0x8fbcbb, 0x81a1c1],
        purple: 0xb48ead,
    },
    Palette {
        id: "gruvbox-dark",
        name: "Gruvbox Dark",
        dark: true,
        ground: 0x282828,
        text: 0xebdbb2,
        dim: 0xbdae93,
        faint: 0x7c6f64,
        accent: 0xfabd2f,
        hues: [0xfb4934, 0xfe8019, 0xfabd2f, 0xb8bb26, 0x8ec07c, 0x83a598],
        purple: 0xd3869b,
    },
    Palette {
        id: "graphite-light",
        name: "Graphite Glass Light",
        dark: false,
        ground: 0xf4f5f7,
        text: 0x14161a,
        dim: 0x5a626c,
        faint: 0x8b929b,
        accent: 0x1a1c20,
        hues: [0xd4491e, 0xc9692a, 0xb07d12, 0x3f8f45, 0x1f8a86, 0x2f6fa8],
        purple: 0x7a56c2,
    },
    Palette {
        id: "catppuccin-latte",
        name: "Catppuccin Latte",
        dark: false,
        ground: 0xeff1f5,
        text: 0x4c4f69,
        dim: 0x6c6f85,
        faint: 0x9ca0b0,
        accent: 0x8839ef,
        hues: [0xd20f39, 0xfe640b, 0xdf8e1d, 0x40a02b, 0x179299, 0x1e66f5],
        purple: 0x8839ef,
    },
    Palette {
        id: "gruvbox-light",
        name: "Gruvbox Light",
        dark: false,
        ground: 0xfbf1c7,
        text: 0x3c3836,
        dim: 0x665c54,
        faint: 0x928374,
        accent: 0xaf3a03,
        hues: [0x9d0006, 0xaf3a03, 0xb57614, 0x79740e, 0x427b58, 0x076678],
        purple: 0x8f3f71,
    },
];

/// The text sizes, as the desktop's theme.toml names them and as its row says them.
const SIZES: [f32; 5] = [0.875, 1.0, 1.15, 1.3, 1.5];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Choice {
    /// 0 dark, 1 light, 2 follow the phone.
    pub mode: i32,
    pub dark: String,
    pub light: String,
    /// -1 is the theme's own accent, 0 to 5 a hue of it.
    pub accent: i32,
    /// An index into the text sizes; 1 is Normal.
    pub size: i32,
}

impl Default for Choice {
    fn default() -> Self {
        Choice {
            mode: 0,
            dark: "graphite-dark".into(),
            light: "graphite-light".into(),
            accent: -1,
            size: 1,
        }
    }
}

fn file(data_dir: &Path) -> PathBuf {
    data_dir.join("look.json")
}

fn load(data_dir: &Path) -> Choice {
    std::fs::read(file(data_dir))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn save(data_dir: &Path, choice: &Choice) {
    let path = file(data_dir);
    let tmp = path.with_extension("json.tmp");
    let written = serde_json::to_vec_pretty(choice)
        .map_err(std::io::Error::other)
        .and_then(|b| std::fs::write(&tmp, b))
        .and_then(|()| std::fs::rename(&tmp, &path));
    if let Err(e) = written {
        crate::local::log(&format!("look: could not keep the choice: {e}"));
    }
}

struct Page {
    window: Weak<AppWindow>,
    data_dir: PathBuf,
    choice: Choice,
}

thread_local! {
    static PAGE: RefCell<Option<Page>> = const { RefCell::new(None) };
}

fn with(f: impl FnOnce(&mut Page)) {
    PAGE.with(|cell| {
        if let Some(p) = cell.borrow_mut().as_mut() {
            f(p);
        }
    });
}

/// Reads the saved choice and dresses the window in it, before the first frame.
pub fn install(window: &AppWindow, data_dir: &Path) {
    let choice = load(data_dir);
    PAGE.with(|cell| {
        *cell.borrow_mut() = Some(Page {
            window: window.as_weak(),
            data_dir: data_dir.to_path_buf(),
            choice,
        })
    });
    let d = window.global::<LookData>();
    d.on_read(|| with(Page::apply));
    d.on_pick_mode(|i| {
        with(|p| {
            p.choice.mode = i.clamp(0, 2);
            p.keep();
        })
    });
    d.on_pick_theme(|dark, i| {
        with(|p| {
            if let Some(pal) = palettes(dark).nth(usize::try_from(i).unwrap_or(usize::MAX)) {
                if dark {
                    p.choice.dark = pal.id.into();
                } else {
                    p.choice.light = pal.id.into();
                }
                p.keep();
            }
        })
    });
    d.on_pick_accent(|i| {
        with(|p| {
            p.choice.accent = i.clamp(-1, 5);
            p.keep();
        })
    });
    d.on_pick_size(|i| {
        with(|p| {
            p.choice.size = i.clamp(0, SIZES.len() as i32 - 1);
            p.keep();
        })
    });
    with(Page::apply);
}

/// The phone's own light or dark setting changed; only System follows it.
pub fn system_changed() {
    with(|p| {
        if p.choice.mode == 2 {
            p.apply();
        }
    });
}

fn palettes(dark: bool) -> impl Iterator<Item = &'static Palette> {
    PALETTES.iter().filter(move |p| p.dark == dark)
}

fn palette(dark: bool, id: &str) -> &'static Palette {
    palettes(dark)
        .find(|p| p.id == id)
        .or_else(|| palettes(dark).next())
        .unwrap_or(&PALETTES[0])
}

fn color(rgb: u32) -> Color {
    Color::from_rgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// `a` moved `t` of the way to `b`, per channel.
fn mix(a: u32, b: u32, t: f32) -> u32 {
    let ch = |shift: u32| {
        let (x, y) = (((a >> shift) & 0xff) as f32, ((b >> shift) & 0xff) as f32);
        ((x + (y - x) * t).round() as u32).min(255) << shift
    };
    ch(16) | ch(8) | ch(0)
}

/// How bright a colour reads, 0 to 1, enough to choose dark or light ink over it.
fn luma(rgb: u32) -> f32 {
    let c = |shift: u32| {
        let v = ((rgb >> shift) & 0xff) as f32 / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * c(16) + 0.7152 * c(8) + 0.0722 * c(0)
}

/// What a dark phone looks like: whether the phone is set to dark.
fn system_dark() -> bool {
    #[cfg(target_os = "android")]
    {
        use jni::JavaVM;
        use jni::objects::JObject;
        let read = || -> Result<bool, Box<dyn std::error::Error>> {
            let ctx = ndk_context::android_context();
            let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
            let mut env = vm.attach_current_thread()?;
            let app = unsafe { JObject::from_raw(ctx.context().cast()) };
            env.with_local_frame(8, |env| -> Result<bool, Box<dyn std::error::Error>> {
                let res = env
                    .call_method(
                        &app,
                        "getResources",
                        "()Landroid/content/res/Resources;",
                        &[],
                    )?
                    .l()?;
                let cfg = env
                    .call_method(
                        &res,
                        "getConfiguration",
                        "()Landroid/content/res/Configuration;",
                        &[],
                    )?
                    .l()?;
                let ui = env.get_field(&cfg, "uiMode", "I")?.i()?;
                // UI_MODE_NIGHT_MASK is 0x30 and UI_MODE_NIGHT_YES 0x20.
                Ok(ui & 0x30 == 0x20)
            })
        };
        read().unwrap_or(true)
    }
    #[cfg(not(target_os = "android"))]
    true
}

impl Page {
    fn keep(&mut self) {
        save(&self.data_dir, &self.choice);
        self.apply();
    }

    fn dark(&self) -> bool {
        match self.choice.mode {
            1 => false,
            2 => system_dark(),
            _ => true,
        }
    }

    fn apply(&mut self) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let dark = self.dark();
        let pal = palette(
            dark,
            if dark {
                &self.choice.dark
            } else {
                &self.choice.light
            },
        );
        let accent = usize::try_from(self.choice.accent)
            .ok()
            .and_then(|i| pal.hues.get(i).copied())
            .unwrap_or(pal.accent);
        let ink = if dark { 0xffffff } else { pal.text };
        let t = window.global::<Theme>();
        t.set_dark(dark);
        t.set_ground(color(pal.ground));
        t.set_ink(color(ink));
        t.set_float(color(mix(pal.ground, ink, if dark { 0.07 } else { 0.06 })));
        t.set_text(color(pal.text));
        t.set_dim(color(pal.dim));
        t.set_faint(color(pal.faint));
        t.set_accent(color(accent));
        // Ink on the accent: the ground when the accent is light, white when it is deep.
        t.set_on_accent(color(if luma(accent) > 0.3 {
            if dark { pal.ground } else { 0x14161a }
        } else {
            0xffffff
        }));
        t.set_ok(color(pal.hues[3]));
        t.set_warn(color(pal.hues[2]));
        t.set_yellow(color(pal.hues[2]));
        t.set_bad(color(pal.hues[0]));
        t.set_cyan(color(pal.hues[4]));
        t.set_purple(color(pal.purple));
        t.set_text_scale(SIZES[self.choice.size.clamp(0, SIZES.len() as i32 - 1) as usize]);
        crate::android::bars(0xff00_0000 | pal.ground);

        let theme_row = |p: &Palette| LookTheme {
            name: p.name.into(),
            ground: color(p.ground),
            text: color(p.text),
            accent: color(p.accent),
        };
        let pick =
            |dark: bool, id: &str| palettes(dark).position(|p| p.id == id).unwrap_or(0) as i32;
        let d = window.global::<LookData>();
        d.set_mode(self.choice.mode);
        d.set_system_line(if system_dark() { "dark" } else { "light" }.into());
        d.set_dark_themes(ModelRc::new(VecModel::from(
            palettes(true).map(theme_row).collect::<Vec<_>>(),
        )));
        d.set_light_themes(ModelRc::new(VecModel::from(
            palettes(false).map(theme_row).collect::<Vec<_>>(),
        )));
        d.set_dark_pick(pick(true, &self.choice.dark));
        d.set_light_pick(pick(false, &self.choice.light));
        d.set_swatches(ModelRc::new(VecModel::from(
            pal.hues.iter().map(|&h| color(h)).collect::<Vec<_>>(),
        )));
        d.set_theme_accent(color(pal.accent));
        d.set_accent(self.choice.accent);
        d.set_size(self.choice.size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_has_a_theme_and_the_defaults_name_real_ones() {
        let c = Choice::default();
        assert_eq!(palette(true, &c.dark).id, "graphite-dark");
        assert_eq!(palette(false, &c.light).id, "graphite-light");
        assert!(palettes(true).count() >= 3 && palettes(false).count() >= 3);
        // A theme file that names one that is gone falls back to the mode's first.
        assert_eq!(palette(true, "gone").id, "graphite-dark");
        assert!(palette(false, "graphite-dark").dark == false);
    }

    #[test]
    fn a_saved_choice_survives_a_reload_and_an_old_file_gets_defaults() {
        let dir = std::env::temp_dir().join(format!("anibeam-look-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let c = Choice {
            mode: 2,
            dark: "nord".into(),
            light: "gruvbox-light".into(),
            accent: 3,
            size: 4,
        };
        save(&dir, &c);
        assert_eq!(load(&dir), c);
        std::fs::write(file(&dir), r#"{"mode":1}"#).unwrap();
        let old = load(&dir);
        assert_eq!((old.mode, old.size, old.accent), (1, 1, -1));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn ink_over_the_accent_is_chosen_by_how_bright_it_is() {
        assert!(luma(0xf2f4f6) > 0.3);
        assert!(luma(0x1a1c20) < 0.3);
        assert_eq!(mix(0x000000, 0xffffff, 0.5), 0x808080);
        assert_eq!(mix(0x0e1013, 0xffffff, 0.0), 0x0e1013);
    }

    #[test]
    fn the_text_sizes_are_the_desktops() {
        assert_eq!(SIZES[1], 1.0);
        assert_eq!(SIZES.len(), 5);
        assert!(SIZES.windows(2).all(|w| w[0] < w[1]));
    }
}
