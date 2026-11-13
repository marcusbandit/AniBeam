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
        #[qproperty(QString, lang)]
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
    }
}

pub struct TrRust {
    lang: QString,
}

impl Default for TrRust {
    /// `ANIBEAM_LANG` picks the language for a run, so a capture can shoot any of them.
    fn default() -> Self {
        if let Some(lang) = std::env::var("ANIBEAM_LANG")
            .ok()
            .and_then(|code| Lang::from_code(&code))
        {
            i18n::set_lang(lang);
        }
        Self {
            lang: QString::from(i18n::lang().code()),
        }
    }
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
        if self.as_ref().rust().lang != *code {
            self.as_mut().set_lang(QString::from(lang.code()));
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
}

/// A JS number stays a number, so it can pick a plural; anything else is its text.
fn arg(value: &QJsonValue) -> Arg<'static> {
    if value.is_double() {
        Arg::Num(value.to_double())
    } else {
        Arg::Str(value.to_string().to_string().into())
    }
}
