//! Tr: the catalog (#792) as a QML singleton. QML asks for words by message ID through
//! `words.t()` (qml/Words.qml), which reads `lang` so every binding that shows words
//! follows a language switch; `lookup` is the plain call underneath.

use core::pin::Pin;

use anibeam_core::i18n::{self, Arg, Lang};
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QJsonArray, QJsonObject, QJsonValue, QString};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qjsonobject.h");
        type QJsonObject = cxx_qt_lib::QJsonObject;
        include!("cxx-qt-lib/qjsonarray.h");
        type QJsonArray = cxx_qt_lib::QJsonArray;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        // Read-only from QML: the language changes through set_language, which moves the
        // core's lookup with it.
        #[qproperty(QString, lang, READ, NOTIFY)]
        // ANIBEAM_LANG set a language for this run: the saved setting does not apply
        #[qproperty(bool, pinned, READ, CONSTANT)]
        type Tr = super::TrRust;

        /// The words for `id` in the current language, `args` filled in: a string or a
        /// number per name, a number also picking the plural.
        #[qinvokable]
        fn lookup(self: &Tr, id: &QString, args: &QJsonObject) -> QString;
        /// Speak the language with this code (`da`, `pt-BR`); an unknown code is ignored.
        #[qinvokable]
        fn set_language(self: Pin<&mut Tr>, code: &QString);
        /// Every language, `{ code, name }` with the name in the language itself.
        #[qinvokable]
        fn languages(self: &Tr) -> QJsonArray;
        /// The language the system asks for, as a code: Danish for `da_DK`, English for
        /// one the app does not speak.
        #[qinvokable]
        fn system_code(self: &Tr) -> QString;
    }
}

pub struct TrRust {
    lang: QString,
    pinned: bool,
}

impl Default for TrRust {
    /// Whatever the core speaks when QML first asks; main() has set it by then.
    fn default() -> Self {
        Self {
            lang: QString::from(i18n::lang().code()),
            pinned: pinned_by_env().is_some(),
        }
    }
}

fn pinned_by_env() -> Option<Lang> {
    std::env::var("ANIBEAM_LANG")
        .ok()
        .and_then(|code| Lang::from_code(&code))
}

/// `ANIBEAM_LANG` picks the language for a run, so a capture can shoot any of them. main()
/// calls this before anything formats a word.
pub fn language_from_env() {
    if let Some(lang) = pinned_by_env() {
        i18n::set_lang(lang);
        super::helpers::ffi::set_default_locale(&QString::from(lang.code()));
    }
}

/// The language picked in Settings, from `[language] code=` in shell.conf, or else the
/// system's; nothing changes when ANIBEAM_LANG pinned one. Words.qml keeps the pick, and
/// this reads it early so nothing is ever shown in English first.
pub fn language_from_settings(conf: &std::path::Path) {
    if pinned_by_env().is_some() {
        return;
    }
    let text = std::fs::read_to_string(conf).unwrap_or_default();
    let lang = saved_language(&text)
        .and_then(|code| Lang::from_code(&code))
        .unwrap_or_else(system_lang);
    i18n::set_lang(lang);
    super::helpers::ffi::set_default_locale(&QString::from(lang.code()));
}

/// The first language the system's person reads that the app speaks, English when none.
fn system_lang() -> Lang {
    first_spoken(&super::helpers::ffi::system_locale_name().to_string())
}

/// The first of a comma separated list of languages ("de-DE,en") that the app speaks.
fn first_spoken(list: &str) -> Lang {
    list.split(',')
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .find_map(|l| {
            let lang = Lang::from_locale(l);
            (lang != Lang::En || l.to_ascii_lowercase().starts_with("en")).then_some(lang)
        })
        .unwrap_or(Lang::En)
}

/// `code=` under `[language]` in a QSettings INI file.
fn saved_language(ini: &str) -> Option<String> {
    let mut in_section = false;
    for line in ini.lines().map(str::trim) {
        if line.starts_with('[') {
            in_section = line == "[language]";
        } else if in_section && let Some(value) = line.strip_prefix("code=") {
            let value = value.trim().trim_matches('"');
            return (!value.is_empty()).then(|| value.to_string());
        }
    }
    None
}

impl qobject::Tr {
    pub fn lookup(&self, id: &QString, args: &QJsonObject) -> QString {
        let names: Vec<String> = args.keys().iter().map(|k| k.to_string()).collect();
        let values: Vec<(&str, Arg)> = names
            .iter()
            .map(|name| (name.as_str(), arg(&args.value(&QString::from(name)))))
            .collect();
        QString::from(&i18n::tr(&id.to_string(), &values))
    }

    pub fn set_language(mut self: Pin<&mut Self>, code: &QString) {
        let Some(lang) = Lang::from_code(&code.to_string()) else {
            return;
        };
        i18n::set_lang(lang);
        super::helpers::ffi::set_default_locale(&QString::from(lang.code()));
        let code = QString::from(lang.code());
        if self.as_ref().rust().lang != code {
            self.as_mut().rust_mut().lang = code;
            self.as_mut().lang_changed();
        }
    }

    pub fn languages(&self) -> QJsonArray {
        let mut out = QJsonArray::default();
        for lang in Lang::ALL {
            let mut entry = QJsonObject::default();
            entry.insert(
                &QString::from("code"),
                &QJsonValue::from(&QString::from(lang.code())),
            );
            entry.insert(
                &QString::from("name"),
                &QJsonValue::from(&QString::from(lang.native_name())),
            );
            out.append(&QJsonValue::from(&entry));
        }
        out
    }

    pub fn system_code(&self) -> QString {
        QString::from(system_lang().code())
    }
}

/// A JS number stays a number, so it can pick a plural; anything else is its text.
fn arg(value: &QJsonValue) -> Arg<'static> {
    if value.is_double() {
        Arg::Num(value.to_double())
    } else {
        Arg::Str(value.to_string().to_string().into())
    }
}

#[cfg(test)]
mod tests {
    use super::{first_spoken, saved_language};

    #[test]
    fn the_saved_language_is_read_from_its_own_section_only() {
        let ini = "[airing]\ncode=fr\n\n[language]\ncode=da\n";
        assert_eq!(saved_language(ini).as_deref(), Some("da"));
        assert_eq!(saved_language("[language]\ncode=\n"), None);
        assert_eq!(
            saved_language("[language]\ncode=pt-BR\n").as_deref(),
            Some("pt-BR")
        );
        assert_eq!(saved_language(""), None);
    }

    #[test]
    fn the_system_language_is_the_first_one_read_that_the_app_speaks() {
        use anibeam_core::i18n::Lang;
        assert_eq!(first_spoken("de-DE,en"), Lang::De);
        assert_eq!(first_spoken("nb-NO,da-DK,en"), Lang::Da);
        assert_eq!(first_spoken("en-US,da-DK"), Lang::En);
        assert_eq!(first_spoken("sv-SE,fi"), Lang::En);
        assert_eq!(first_spoken(""), Lang::En);
    }
}
