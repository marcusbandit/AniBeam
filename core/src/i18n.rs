//! The catalog (#792): every word a person reads, in every language the app speaks.
//!
//! The words live in Fluent files at the repo root, `i18n/<lang>/<file>.ftl`, one file per
//! part of the app so each can be edited alone: `core.ftl` for what the core says,
//! `desktop.ftl` for the Qt shell and `android.ftl` for the phone. Every file of a language
//! goes into one bundle, so an ID or a `-term` in one file is seen from the others.
//! English is the source; any other language may lack an ID, and then the English is shown.
//!
//! The files are built into the binary, so no shell needs them on disk. A shell sets the
//! language once with [`set_lang`] and asks for words with [`tr`], from any thread.

use std::borrow::Cow;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use fluent_syntax::ast::Pattern;
use unic_langid::LanguageIdentifier;

/// A language the app speaks: the same 13 as the website.
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
    /// Every language, English first, in the order a picker lists them.
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

    /// The BCP 47 tag, which is also the language's folder under `i18n/`.
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

    /// The language's name in itself, as a picker shows it: "Dansk", "日本語".
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

    /// The language a locale string asks for, in any of the shapes systems hand out:
    /// `da_DK.UTF-8`, `pt-BR`, `zh-Hans-CN`, `en_US@euro`. Only the language subtag
    /// decides: every Portuguese gets Brazilian, every Chinese gets Simplified. A language
    /// the app does not speak, and `C` or `POSIX`, give English.
    pub fn from_locale(locale: &str) -> Lang {
        let lang = locale
            .split(['.', '@'])
            .next()
            .unwrap_or_default()
            .split(['_', '-'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
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
            "zh" => Lang::ZhCn,
            _ => Lang::En,
        }
    }

    /// The language with this exact code from [`Lang::code`], as a saved setting holds it.
    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }

    /// The locale chrono names months and weekdays in, for a date pattern from the catalog
    /// formatted with `format_localized`. English reads the British way: 17 Oct, 24-hour.
    pub fn chrono_locale(self) -> chrono::Locale {
        chrono_locale(self)
    }

    fn index(self) -> usize {
        Lang::ALL.iter().position(|l| *l == self).unwrap_or(0)
    }
}

/// One argument to a message: text, or a number, which also picks the plural form.
#[derive(Clone, Debug, PartialEq)]
pub enum Arg<'a> {
    Str(Cow<'a, str>),
    Num(f64),
}

impl<'a> From<&'a str> for Arg<'a> {
    fn from(s: &'a str) -> Self {
        Arg::Str(Cow::Borrowed(s))
    }
}

impl From<String> for Arg<'_> {
    fn from(s: String) -> Self {
        Arg::Str(Cow::Owned(s))
    }
}

macro_rules! num_arg {
    ($($t:ty),*) => {$(
        impl From<$t> for Arg<'_> {
            fn from(n: $t) -> Self {
                Arg::Num(n as f64)
            }
        }
    )*};
}
num_arg!(i32, i64, u32, u64, usize, f32, f64);

/// The files of one language, in the order they are added to its bundle.
macro_rules! files {
    ($code:literal) => {
        [
            (
                "core.ftl",
                include_str!(concat!("../../i18n/", $code, "/core.ftl")),
            ),
            (
                "desktop.ftl",
                include_str!(concat!("../../i18n/", $code, "/desktop.ftl")),
            ),
            (
                "android.ftl",
                include_str!(concat!("../../i18n/", $code, "/android.ftl")),
            ),
        ]
    };
}

/// Every file of every language, in [`Lang::ALL`]'s order.
const CATALOG: [[(&str, &str); 3]; 13] = [
    files!("en"),
    files!("da"),
    files!("de"),
    files!("es"),
    files!("fr"),
    files!("pt-BR"),
    files!("it"),
    files!("nl"),
    files!("pl"),
    files!("ru"),
    files!("ja"),
    files!("ko"),
    files!("zh-CN"),
];

type Bundle = FluentBundle<FluentResource>;

/// One language's bundle, built the first time it is asked for. A message that fails to
/// parse is dropped and the rest of its file still loads; the catalog tests catch it.
fn bundle(lang: Lang) -> &'static Bundle {
    static BUNDLES: [OnceLock<Bundle>; 13] = [const { OnceLock::new() }; 13];
    BUNDLES[lang.index()].get_or_init(|| {
        let id: LanguageIdentifier = lang.code().parse().unwrap_or_default();
        let mut bundle = FluentBundle::new_concurrent(vec![id]);
        // Qt and Slint draw the isolation marks Fluent puts around arguments as boxes,
        // and every language here runs left to right.
        bundle.set_use_isolating(false);
        for (name, text) in CATALOG[lang.index()] {
            let resource = FluentResource::try_new(text.to_string()).unwrap_or_else(|(r, e)| {
                tracing::warn!("i18n/{}/{name}: {} parse errors", lang.code(), e.len());
                r
            });
            if let Err(e) = bundle.add_resource(resource) {
                tracing::warn!("i18n/{}/{name}: {} clashing IDs", lang.code(), e.len());
            }
        }
        bundle
    })
}

/// [`Lang::chrono_locale`] as a free function, beside [`tr`].
pub fn chrono_locale(lang: Lang) -> chrono::Locale {
    use chrono::Locale as L;
    match lang {
        Lang::En => L::en_GB,
        Lang::Da => L::da_DK,
        Lang::De => L::de_DE,
        Lang::Es => L::es_ES,
        Lang::Fr => L::fr_FR,
        Lang::PtBr => L::pt_BR,
        Lang::It => L::it_IT,
        Lang::Nl => L::nl_NL,
        Lang::Pl => L::pl_PL,
        Lang::Ru => L::ru_RU,
        Lang::Ja => L::ja_JP,
        Lang::Ko => L::ko_KR,
        Lang::ZhCn => L::zh_CN,
    }
}

static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// Switch every later [`tr`] to `lang`. Words already on screen stay as they are until
/// the shell asks again.
pub fn set_lang(lang: Lang) {
    CURRENT.store(lang.index(), Ordering::Relaxed);
}

/// The language [`tr`] speaks now, English until a shell sets one.
pub fn lang() -> Lang {
    Lang::ALL[CURRENT.load(Ordering::Relaxed)]
}

/// The words for message `id` in the current language, with `args` filled in.
///
/// `id` names a message (`player-skip-intro`) or one of its attributes
/// (`player-skip-intro.tip`). An ID the language lacks comes out in English; one English
/// lacks too comes out as the ID itself, so a gap shows on screen instead of a blank.
pub fn tr(id: &str, args: &[(&str, Arg)]) -> String {
    tr_in(lang(), id, args)
}

/// [`tr`] in a given language, whatever the current one is.
pub fn tr_in(lang: Lang, id: &str, args: &[(&str, Arg)]) -> String {
    format(bundle(lang), id, args)
        .or_else(|| {
            (lang != Lang::En)
                .then(|| format(bundle(Lang::En), id, args))
                .flatten()
        })
        .unwrap_or_else(|| id.to_string())
}

/// Whether English has message `id` (or `id.attr`). The catalog check and the tests use
/// it to prove every ID the code asks for exists.
pub fn exists(id: &str) -> bool {
    lookup(bundle(Lang::En), id).is_some()
}

/// The pattern behind `id`: the message's value, or the attribute after a dot.
fn lookup<'b>(bundle: &'b Bundle, id: &str) -> Option<&'b Pattern<&'b str>> {
    let (name, attr) = match id.split_once('.') {
        Some((name, attr)) => (name, Some(attr)),
        None => (id, None),
    };
    let message = bundle.get_message(name)?;
    match attr {
        Some(attr) => Some(message.get_attribute(attr)?.value()),
        None => message.value(),
    }
}

fn format(bundle: &Bundle, id: &str, args: &[(&str, Arg)]) -> Option<String> {
    let pattern = lookup(bundle, id)?;
    let mut fluent_args = FluentArgs::new();
    for (key, arg) in args {
        let value = match arg {
            Arg::Str(s) => FluentValue::from(s.to_string()),
            Arg::Num(n) => FluentValue::from(*n),
        };
        fluent_args.set(*key, value);
    }
    let mut errors = Vec::new();
    let text = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
    if !errors.is_empty() {
        tracing::debug!("i18n {id}: {errors:?}");
    }
    Some(text.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_locale_in_any_shape_picks_its_language() {
        assert_eq!(Lang::from_locale("da_DK.UTF-8"), Lang::Da);
        assert_eq!(Lang::from_locale("pt_BR"), Lang::PtBr);
        assert_eq!(Lang::from_locale("pt-PT"), Lang::PtBr);
        assert_eq!(Lang::from_locale("zh-Hans-CN"), Lang::ZhCn);
        assert_eq!(Lang::from_locale("ja"), Lang::Ja);
        assert_eq!(Lang::from_locale("DE-at"), Lang::De);
        assert_eq!(Lang::from_locale("en_US@euro"), Lang::En);
        assert_eq!(Lang::from_locale("nb-NO"), Lang::En);
        assert_eq!(Lang::from_locale("C"), Lang::En);
        assert_eq!(Lang::from_locale(""), Lang::En);
    }

    #[test]
    fn every_code_names_one_language_and_parses_as_a_tag() {
        for lang in Lang::ALL {
            assert_eq!(Lang::from_code(lang.code()), Some(lang));
            assert_eq!(Lang::from_locale(lang.code()), lang);
            assert!(lang.code().parse::<LanguageIdentifier>().is_ok());
        }
        assert_eq!(Lang::from_code("pt-br"), None);
    }

    #[test]
    fn every_file_parses_and_no_id_is_defined_twice() {
        for lang in Lang::ALL {
            let mut bundle: Bundle = FluentBundle::new_concurrent(vec![]);
            for (name, text) in CATALOG[lang.index()] {
                let resource = FluentResource::try_new(text.to_string())
                    .unwrap_or_else(|(_, e)| panic!("i18n/{}/{name}: {e:?}", lang.code()));
                bundle
                    .add_resource(resource)
                    .unwrap_or_else(|e| panic!("i18n/{}/{name}: {e:?}", lang.code()));
            }
        }
    }

    #[test]
    fn every_translated_id_exists_in_english() {
        for lang in Lang::ALL {
            for (name, text) in CATALOG[lang.index()] {
                let resource = FluentResource::try_new(text.to_string()).unwrap();
                for entry in resource.entries() {
                    if let fluent_syntax::ast::Entry::Message(m) = entry {
                        assert!(
                            exists(m.id.name),
                            "i18n/{}/{name}: {} is not in English",
                            lang.code(),
                            m.id.name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_date_names_its_month_in_the_language() {
        let day = chrono::NaiveDate::from_ymd_opt(2026, 10, 17).unwrap();
        let month = |lang| {
            day.format_localized("%-d %b", chrono_locale(lang))
                .to_string()
        };
        assert_eq!(month(Lang::En), "17 Oct");
        assert_eq!(month(Lang::De), "17 Okt");
        assert!(month(Lang::Ru).contains("окт"));
    }

    #[test]
    fn a_number_picks_the_plural_its_language_uses() {
        let eps = |lang, n: u64| tr_in(lang, "count-episodes", &[("count", n.into())]);
        assert_eq!(eps(Lang::En, 1), "1 episode");
        assert_eq!(eps(Lang::En, 5), "5 episodes");
        assert_eq!(eps(Lang::Da, 1), "1 afsnit");
        assert_eq!(eps(Lang::Pl, 1), "1 odcinek");
        assert_eq!(eps(Lang::Pl, 3), "3 odcinki");
        assert_eq!(eps(Lang::Pl, 5), "5 odcinków");
        assert_eq!(eps(Lang::Pl, 22), "22 odcinki");
    }

    #[test]
    fn a_missing_id_falls_back_to_english_then_to_itself() {
        let args = [("count", Arg::from(2))];
        assert_eq!(tr_in(Lang::De, "count-episodes", &args), "2 episodes");
        assert_eq!(tr_in(Lang::De, "no-such-message", &[]), "no-such-message"); // i18n: none
        let attr = tr_in(Lang::En, "count-episodes.tip", &args); // i18n: none
        assert_eq!(attr, "count-episodes.tip");
        assert!(exists("count-episodes"));
        assert!(!exists("no-such-message"));
    }

    #[test]
    fn set_lang_switches_what_tr_speaks() {
        let args = [("count", Arg::from(2))];
        set_lang(Lang::Da);
        assert_eq!(lang(), Lang::Da);
        let da = tr("count-shows", &args);
        set_lang(Lang::En);
        assert_eq!(da, "2 serier");
        assert_eq!(tr("count-shows", &args), "2 shows");
    }
}
