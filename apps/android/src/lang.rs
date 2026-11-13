//! The language the app speaks (#793): the phone's own unless one was picked. The words come
//! from the core's catalog by message ID; this module only decides which language, tells the
//! catalog, and has every page re-word itself where it stands.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};

use crate::tr::{self, Lang};

/// A line from the catalog by message ID, in the language the app speaks now:
/// `tr!("look-mode")`, `tr!("home-note", count = n)`, `tr!("host-synced", when = stamp)`.
/// A number argument picks the plural form.
macro_rules! tr {
    ($id:expr) => {
        $crate::tr::tr($id, &[])
    };
    ($id:expr, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::tr::tr($id, &[$((stringify!($name), $crate::tr::Arg::from($value))),+])
    };
}
use crate::{AppWindow, LangChoice, LangData, Tr};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Choice {
    /// A catalog code (`da`, `pt-BR`), or empty to follow the phone.
    pick: String,
}

fn file(data_dir: &Path) -> PathBuf {
    data_dir.join("language.json")
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
        crate::local::log(&format!("language: could not keep the choice: {e}"));
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

/// The phone's language as Android reports it: `da-DK`, `pt-BR`, `zh-Hans-CN`.
fn system_locale() -> String {
    #[cfg(target_os = "android")]
    {
        use jni::JavaVM;
        use jni::objects::{JObject, JString};
        let read = || -> Result<String, Box<dyn std::error::Error>> {
            let ctx = ndk_context::android_context();
            let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
            let mut env = vm.attach_current_thread()?;
            let app = unsafe { JObject::from_raw(ctx.context().cast()) };
            env.with_local_frame(8, |env| -> Result<String, Box<dyn std::error::Error>> {
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
                let list = env
                    .call_method(&cfg, "getLocales", "()Landroid/os/LocaleList;", &[])?
                    .l()?;
                let first = env
                    .call_method(&list, "get", "(I)Ljava/util/Locale;", &[0.into()])?
                    .l()?;
                let tag = env
                    .call_method(&first, "toLanguageTag", "()Ljava/lang/String;", &[])?
                    .l()?;
                Ok(env.get_string(&JString::from(tag))?.into())
            })
        };
        read().unwrap_or_default()
    }
    #[cfg(not(target_os = "android"))]
    std::env::var("LANG").unwrap_or_default()
}

fn phone_lang() -> Lang {
    Lang::from_locale(&system_locale())
}

/// Reads the saved pick and sets the catalog's language, before the first frame.
pub fn install(window: &AppWindow, data_dir: &Path) {
    let choice = load(data_dir);
    window
        .global::<Tr>()
        .on_lookup(|id, _rev, args| lookup(&id, &args).into());
    let d = window.global::<LangData>();
    d.on_pick_lang(|i| {
        with(|p| {
            p.choice.pick = usize::try_from(i)
                .ok()
                .and_then(|i| Lang::ALL.get(i))
                .map(|l| l.code().to_owned())
                .unwrap_or_default();
            save(&p.data_dir, &p.choice);
            p.apply();
        })
    });
    PAGE.with(|cell| {
        *cell.borrow_mut() = Some(Page {
            window: window.as_weak(),
            data_dir: data_dir.to_path_buf(),
            choice,
        })
    });
    with(Page::apply);
}

/// The phone's own language changed; only a phone-following choice follows it.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn system_changed() {
    // A picked language stays, but the "Same as the phone" row names the phone's new one.
    // apply re-words the pages only when the language in use changed.
    with(Page::apply);
}

/// `Tr.lookup`'s arguments, name and value in turn; a `#` before a name marks a number.
fn lookup(id: &str, args: &slint::ModelRc<SharedString>) -> String {
    use slint::Model;
    let flat: Vec<SharedString> = args.iter().collect();
    let pairs: Vec<(&str, tr::Arg)> = flat
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[k, v]| match k.strip_prefix('#') {
            Some(name) => (name, tr::Arg::Num(v.parse().unwrap_or(0.0))),
            None => (k.as_str(), tr::Arg::from(v.to_string())),
        })
        .collect();
    tr::tr(id, &pairs)
}

impl Page {
    fn lang(&self) -> Lang {
        Lang::from_code(&self.choice.pick).unwrap_or_else(phone_lang)
    }

    fn apply(&mut self) {
        let lang = self.lang();
        let changed = tr::lang() != lang;
        tr::set_lang(lang);
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let d = window.global::<LangData>();
        let langs: Vec<LangChoice> = Lang::ALL
            .iter()
            .map(|l| {
                let here = tr::tr(&format!("lang-name-{}", l.code()), &[]);
                LangChoice {
                    name: l.native_name().into(),
                    note: if here == l.native_name() {
                        SharedString::new()
                    } else {
                        here.into()
                    },
                }
            })
            .collect();
        d.set_langs(ModelRc::new(VecModel::from(langs)));
        d.set_pick(
            Lang::from_code(&self.choice.pick)
                .and_then(|l| Lang::ALL.iter().position(|x| *x == l))
                .map_or(-1, |i| i as i32),
        );
        d.set_phone(phone_lang().native_name().into());
        if changed {
            let t = window.global::<Tr>();
            t.set_rev(t.get_rev().wrapping_add(1));
            crate::app::relocalize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_pick_survives_a_reload_and_an_empty_one_follows_the_phone() {
        let dir = std::env::temp_dir().join(format!("anibeam-lang-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(load(&dir).pick, "");
        let c = Choice { pick: "da".into() };
        save(&dir, &c);
        assert_eq!(load(&dir), c);
        let _ = std::fs::remove_dir_all(dir);
    }
}
