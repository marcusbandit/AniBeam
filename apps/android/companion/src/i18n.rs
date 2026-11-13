//! STAND-IN until #792 lands `anibeam_core::i18n` on main. Same names and signatures as the
//! core's, over the same catalog files at the repo root, so the rebase turns this file into
//! `pub use anibeam_core::i18n::*;`. No strings live here.

use std::sync::{OnceLock, RwLock};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Da,
    De,
    Es,
    Fr,
    PtBr,
    It,
    Nl,
    Pl,
    Ru,
    Ja,
    Ko,
    ZhCn,
}

impl Lang {
    pub const ALL: [Lang; 13] = [
        Lang::En,
        Lang::Da,
        Lang::De,
        Lang::Es,
        Lang::Fr,
        Lang::PtBr,
        Lang::It,
        Lang::Nl,
        Lang::Pl,
        Lang::Ru,
        Lang::Ja,
        Lang::Ko,
        Lang::ZhCn,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Da => "da",
            Lang::De => "de",
            Lang::Es => "es",
            Lang::Fr => "fr",
            Lang::PtBr => "pt-BR",
            Lang::It => "it",
            Lang::Nl => "nl",
            Lang::Pl => "pl",
            Lang::Ru => "ru",
            Lang::Ja => "ja",
            Lang::Ko => "ko",
            Lang::ZhCn => "zh-CN",
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Da => "Dansk",
            Lang::De => "Deutsch",
            Lang::Es => "Español",
            Lang::Fr => "Français",
            Lang::PtBr => "Português (Brasil)",
            Lang::It => "Italiano",
            Lang::Nl => "Nederlands",
            Lang::Pl => "Polski",
            Lang::Ru => "Русский",
            Lang::Ja => "日本語",
            Lang::Ko => "한국어",
            Lang::ZhCn => "简体中文",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL
            .into_iter()
            .find(|l| l.code().eq_ignore_ascii_case(code))
    }

    /// The nearest of the 13 to a system locale: `pt_BR`, `zh-Hans-CN`, `nb-NO`, `de-AT`.
    pub fn from_locale(locale: &str) -> Lang {
        let tag = locale.replace('_', "-");
        let mut parts = tag.split(['-', '.', '@']).filter(|p| !p.is_empty());
        let lang = parts.next().unwrap_or("").to_ascii_lowercase();
        let rest: Vec<String> = parts.map(|p| p.to_ascii_lowercase()).collect();
        match lang.as_str() {
            "da" => Lang::Da,
            "de" => Lang::De,
            "es" => Lang::Es,
            "fr" => Lang::Fr,
            "pt" => Lang::PtBr,
            "it" => Lang::It,
            "nl" => Lang::Nl,
            "pl" => Lang::Pl,
            "ru" => Lang::Ru,
            "ja" => Lang::Ja,
            "ko" => Lang::Ko,
            // Traditional script or a Traditional region has no catalog; English reads
            // better there than Simplified.
            "zh" if rest
                .iter()
                .any(|p| ["hant", "tw", "hk", "mo"].contains(&p.as_str())) =>
            {
                Lang::En
            }
            "zh" => Lang::ZhCn,
            _ => Lang::En,
        }
    }

    fn source(self) -> &'static str {
        match self {
            Lang::En => include_str!("../../../../i18n/en/android.ftl"),
            Lang::Da => include_str!("../../../../i18n/da/android.ftl"),
            Lang::De => include_str!("../../../../i18n/de/android.ftl"),
            Lang::Es => include_str!("../../../../i18n/es/android.ftl"),
            Lang::Fr => include_str!("../../../../i18n/fr/android.ftl"),
            Lang::PtBr => include_str!("../../../../i18n/pt-BR/android.ftl"),
            Lang::It => include_str!("../../../../i18n/it/android.ftl"),
            Lang::Nl => include_str!("../../../../i18n/nl/android.ftl"),
            Lang::Pl => include_str!("../../../../i18n/pl/android.ftl"),
            Lang::Ru => include_str!("../../../../i18n/ru/android.ftl"),
            Lang::Ja => include_str!("../../../../i18n/ja/android.ftl"),
            Lang::Ko => include_str!("../../../../i18n/ko/android.ftl"),
            Lang::ZhCn => include_str!("../../../../i18n/zh-CN/android.ftl"),
        }
    }
}

/// The chrono locale whose month and weekday names this language writes. English maps to
/// en_GB: "17 Oct", 24-hour clock, the way the app has always shown dates.
pub fn chrono_locale(lang: Lang) -> chrono::Locale {
    match lang {
        Lang::En => chrono::Locale::en_GB,
        Lang::Da => chrono::Locale::da_DK,
        Lang::De => chrono::Locale::de_DE,
        Lang::Es => chrono::Locale::es_ES,
        Lang::Fr => chrono::Locale::fr_FR,
        Lang::PtBr => chrono::Locale::pt_BR,
        Lang::It => chrono::Locale::it_IT,
        Lang::Nl => chrono::Locale::nl_NL,
        Lang::Pl => chrono::Locale::pl_PL,
        Lang::Ru => chrono::Locale::ru_RU,
        Lang::Ja => chrono::Locale::ja_JP,
        Lang::Ko => chrono::Locale::ko_KR,
        Lang::ZhCn => chrono::Locale::zh_CN,
    }
}

/// An argument to a message: text, or a number (which picks the plural form).
#[derive(Clone, Debug)]
pub enum Arg {
    Str(String),
    Num(f64),
}

impl From<&str> for Arg {
    fn from(s: &str) -> Self {
        Arg::Str(s.into())
    }
}
impl From<String> for Arg {
    fn from(s: String) -> Self {
        Arg::Str(s)
    }
}
impl From<&String> for Arg {
    fn from(s: &String) -> Self {
        Arg::Str(s.clone())
    }
}
macro_rules! num_arg {
    ($($t:ty),*) => {$(
        impl From<$t> for Arg {
            fn from(n: $t) -> Self {
                Arg::Num(n as f64)
            }
        }
    )*};
}
num_arg!(i32, i64, u32, u64, usize, f32, f64);

type Bundle = FluentBundle<FluentResource>;

fn bundle(lang: Lang) -> Bundle {
    let id: LanguageIdentifier = lang.code().parse().unwrap_or_default();
    let mut b = Bundle::new_concurrent(vec![id]);
    // No FSI/PDI marks around arguments: the phone's text renders them as boxes.
    b.set_use_isolating(false);
    match FluentResource::try_new(lang.source().to_owned()) {
        Ok(r) => {
            let _ = b.add_resource(r);
        }
        Err((r, _)) => {
            let _ = b.add_resource(r);
        }
    }
    b
}

struct Catalog {
    lang: Lang,
    picked: Bundle,
    english: Bundle,
}

fn catalog() -> &'static RwLock<Catalog> {
    static C: OnceLock<RwLock<Catalog>> = OnceLock::new();
    C.get_or_init(|| {
        RwLock::new(Catalog {
            lang: Lang::En,
            picked: bundle(Lang::En),
            english: bundle(Lang::En),
        })
    })
}

pub fn set_lang(lang: Lang) {
    let mut c = catalog().write().unwrap_or_else(|e| e.into_inner());
    if c.lang != lang {
        c.lang = lang;
        c.picked = bundle(lang);
    }
}

pub fn lang() -> Lang {
    catalog().read().unwrap_or_else(|e| e.into_inner()).lang
}

fn format(b: &Bundle, id: &str, args: Option<&FluentArgs>) -> Option<String> {
    let msg = b.get_message(id)?;
    let pattern = msg.value()?;
    let mut errors = Vec::new();
    Some(b.format_pattern(pattern, args, &mut errors).into_owned())
}

fn fluent_args(args: &[(&str, Arg)]) -> Option<FluentArgs<'static>> {
    (!args.is_empty()).then(|| {
        let mut fa = FluentArgs::new();
        for (k, v) in args {
            match v {
                Arg::Str(s) => fa.set(k.to_string(), FluentValue::from(s.clone())),
                Arg::Num(n) => fa.set(k.to_string(), FluentValue::from(*n)),
            }
        }
        fa
    })
}

/// The message in one given language, whichever the app speaks: English for a search that
/// should still find English words.
pub fn tr_in(lang: Lang, id: &str, args: &[(&str, Arg)]) -> String {
    let fa = fluent_args(args);
    let c = catalog().read().unwrap_or_else(|e| e.into_inner());
    let own;
    let b = if lang == c.lang {
        &c.picked
    } else if lang == Lang::En {
        &c.english
    } else {
        own = bundle(lang);
        &own
    };
    format(b, id, fa.as_ref())
        .or_else(|| format(&c.english, id, fa.as_ref()))
        .unwrap_or_else(|| id.to_owned())
}

/// The message in the picked language, English where it has none, the ID where neither has.
pub fn tr(id: &str, args: &[(&str, Arg)]) -> String {
    let fa = (!args.is_empty()).then(|| {
        let mut fa = FluentArgs::new();
        for (k, v) in args {
            match v {
                Arg::Str(s) => fa.set(*k, FluentValue::from(s.as_str())),
                Arg::Num(n) => fa.set(*k, FluentValue::from(*n)),
            }
        }
        fa
    });
    let c = catalog().read().unwrap_or_else(|e| e.into_inner());
    format(&c.picked, id, fa.as_ref())
        .or_else(|| format(&c.english, id, fa.as_ref()))
        .unwrap_or_else(|| id.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_locales_land_on_the_nearest_catalog() {
        assert_eq!(Lang::from_locale("da-DK"), Lang::Da);
        assert_eq!(Lang::from_locale("pt_PT"), Lang::PtBr);
        assert_eq!(Lang::from_locale("zh-Hans-CN"), Lang::ZhCn);
        assert_eq!(Lang::from_locale("zh-Hant-TW"), Lang::En);
        assert_eq!(Lang::from_locale("nb-NO"), Lang::En);
        assert_eq!(Lang::from_locale(""), Lang::En);
    }

    /// Every catalog reads as Fluent and has every message the English has, so no screen
    /// falls back to English in one language.
    #[test]
    fn every_catalog_parses_and_has_every_english_message() {
        let ids: Vec<&str> = Lang::En
            .source()
            .lines()
            .filter(|l| l.starts_with(|c: char| c.is_ascii_alphabetic()))
            .filter_map(|l| l.split_once(" =").map(|(id, _)| id.trim()))
            .collect();
        assert!(
            ids.len() > 100,
            "the English catalog reads as {} messages",
            ids.len()
        );
        for lang in Lang::ALL {
            if let Err((_, errors)) = FluentResource::try_new(lang.source().to_owned()) {
                panic!("{} does not parse: {errors:?}", lang.code());
            }
            let b = bundle(lang);
            let missing: Vec<&&str> = ids.iter().filter(|id| !b.has_message(id)).collect();
            assert!(missing.is_empty(), "{} lacks {missing:?}", lang.code());
        }
    }

    #[test]
    fn a_missing_message_falls_back_to_english_then_to_its_id() {
        set_lang(Lang::En);
        assert_eq!(tr("onb-start", &[]), "Start");
        assert_eq!(tr("no-such-message", &[]), "no-such-message");
        assert!(tr("onb-cap-on", &[("cap", "18 GB".into())]).contains("18 GB"));
    }
}
