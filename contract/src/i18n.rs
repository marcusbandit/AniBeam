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
        LANGS[self.index()].code
    }

    /// The language's name in itself, as a picker shows it: "Dansk", "日本語".
    pub fn native_name(self) -> &'static str {
        LANGS[self.index()].name
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
        Lang::ALL
            .into_iter()
            .find(|l| l.code().split('-').next() == Some(lang.as_str()))
            .unwrap_or(Lang::En)
    }

    /// The language with this exact code from [`Lang::code`], as a saved setting holds it.
    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }

    /// The mark between a number's whole part and its fraction: "1,5 GB" in Danish, German
    /// and most of Europe, "1.5 GB" in English and the three East Asian languages.
    pub fn decimal_mark(self) -> char {
        match self {
            Lang::En | Lang::Ja | Lang::Ko | Lang::ZhCn => '.',
            _ => ',',
        }
    }

    /// The language's place in [`Lang::ALL`], which lists them in declaration order.
    fn index(self) -> usize {
        self as usize
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

impl<'a> From<&'a String> for Arg<'a> {
    fn from(s: &'a String) -> Self {
        Arg::Str(Cow::Borrowed(s.as_str()))
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

/// What the app keeps of one language: its tag, its name in itself, the locale chrono
/// names months and weekdays in, and its files, in the order they go into its bundle.
struct Info {
    code: &'static str,
    name: &'static str,
    locale: chrono::Locale,
    files: [(&'static str, &'static str); 3],
}

macro_rules! lang {
    ($code:literal, $name:literal, $locale:ident) => {
        Info {
            code: $code,
            name: $name,
            locale: chrono::Locale::$locale,
            files: [
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
            ],
        }
    };
}

/// Every language, in [`Lang::ALL`]'s order.
const LANGS: [Info; 13] = [
    lang!("en", "English", en_GB),
    lang!("da", "Dansk", da_DK),
    lang!("de", "Deutsch", de_DE),
    lang!("es", "Español", es_ES),
    lang!("fr", "Français", fr_FR),
    lang!("pt-BR", "Português (Brasil)", pt_BR),
    lang!("it", "Italiano", it_IT),
    lang!("nl", "Nederlands", nl_NL),
    lang!("pl", "Polski", pl_PL),
    lang!("ru", "Русский", ru_RU),
    lang!("ja", "日本語", ja_JP),
    lang!("ko", "한국어", ko_KR),
    lang!("zh-CN", "简体中文", zh_CN),
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
        // A number argument with a fraction takes the language's decimal mark ("12,5")
        if lang.decimal_mark() == ',' {
            bundle.set_formatter(Some(decimal_comma));
        }
        for (name, text) in LANGS[lang.index()].files {
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

/// The locale chrono names months and weekdays in, for a date pattern from the catalog
/// formatted with `format_localized`. English reads the British way: 17 Oct, 24-hour.
pub fn chrono_locale(lang: Lang) -> chrono::Locale {
    LANGS[lang.index()].locale
}

/// Fluent's formatter for a language that writes "12,5": every number with a fraction.
fn decimal_comma<M>(value: &FluentValue, _: &M) -> Option<String> {
    match value {
        FluentValue::Number(n) => {
            let s = n.as_string();
            s.contains('.').then(|| s.replace('.', ","))
        }
        _ => None,
    }
}

/// `text`, a number already written with a point ("1.5"), with the current language's
/// decimal mark instead.
pub fn decimal(text: &str) -> String {
    match lang().decimal_mark() {
        '.' => text.to_string(),
        mark => text.replace('.', &mark.to_string()),
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

/// [`tr`] in a given language, whatever the current one is. A translation that does not
/// format cleanly, such as one naming a variable English does not pass, gives way to the
/// English rather than showing `{$cout}`.
pub fn tr_in(lang: Lang, id: &str, args: &[(&str, Arg)]) -> String {
    let own = (lang != Lang::En).then(|| bundle(lang));
    words_from(own, bundle(Lang::En), id, args)
}

/// The words for `id` from `own`, a language's bundle, or else from `english`, or else the
/// ID itself. A translation that does not format cleanly gives way to the English.
fn words_from(own: Option<&Bundle>, english: &Bundle, id: &str, args: &[(&str, Arg)]) -> String {
    if let Some((text, clean)) = own.and_then(|b| format(b, id, args)) {
        if clean {
            return text;
        }
        tracing::warn!("i18n: {id} does not format, shown in English");
    }
    format(english, id, args)
        .map(|(text, _)| text)
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

/// The formatted words, and whether they formatted without an error.
fn format(bundle: &Bundle, id: &str, args: &[(&str, Arg)]) -> Option<(String, bool)> {
    let pattern = lookup(bundle, id)?;
    let mut fluent_args = FluentArgs::new();
    for (key, arg) in args {
        let value = match arg {
            Arg::Str(s) => FluentValue::from(s.as_ref()),
            Arg::Num(n) => FluentValue::from(*n),
        };
        fluent_args.set(*key, value);
    }
    let mut errors = Vec::new();
    let text = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
    if !errors.is_empty() {
        tracing::debug!("i18n {id}: {errors:?}");
    }
    Some((text.into_owned(), errors.is_empty()))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use fluent_syntax::ast::{self, Entry};

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
    fn every_language_is_at_its_place() {
        for (i, lang) in Lang::ALL.into_iter().enumerate() {
            assert_eq!(lang.index(), i);
        }
    }

    #[test]
    fn every_file_parses_and_no_id_is_defined_twice() {
        for lang in Lang::ALL {
            let mut bundle: Bundle = FluentBundle::new_concurrent(vec![]);
            for (name, text) in LANGS[lang.index()].files {
                let resource = FluentResource::try_new(text.to_string())
                    .unwrap_or_else(|(_, e)| panic!("i18n/{}/{name}: {e:?}", lang.code()));
                bundle
                    .add_resource(resource)
                    .unwrap_or_else(|e| panic!("i18n/{}/{name}: {e:?}", lang.code()));
            }
        }
    }

    /// Every message and attribute of one language, by `id` or `id.attr`, with the
    /// variables it reads.
    fn patterns(lang: Lang) -> Vec<(String, &'static str, BTreeSet<String>)> {
        let mut out = Vec::new();
        for (name, text) in LANGS[lang.index()].files {
            let resource = FluentResource::try_new(text.to_string()).unwrap();
            for entry in resource.entries() {
                let Entry::Message(m) = entry else { continue };
                if let Some(value) = &m.value {
                    out.push((m.id.name.to_string(), name, Refs::of(value).vars));
                }
                for a in &m.attributes {
                    let id = format!("{}.{}", m.id.name, a.id.name);
                    out.push((id, name, Refs::of(&a.value).vars));
                }
            }
        }
        out
    }

    /// The variables and the `-terms` patterns read, however deep in a select or a call.
    #[derive(Default)]
    struct Refs {
        vars: BTreeSet<String>,
        terms: BTreeSet<String>,
    }

    impl Refs {
        fn of(p: &ast::Pattern<&str>) -> Refs {
            let mut refs = Refs::default();
            refs.pattern(p);
            refs
        }

        fn pattern(&mut self, p: &ast::Pattern<&str>) {
            for el in &p.elements {
                if let ast::PatternElement::Placeable { expression } = el {
                    self.expr(expression);
                }
            }
        }

        fn expr(&mut self, e: &ast::Expression<&str>) {
            match e {
                ast::Expression::Inline(e) => self.inline(e),
                ast::Expression::Select { selector, variants } => {
                    self.inline(selector);
                    variants.iter().for_each(|v| self.pattern(&v.value));
                }
            }
        }

        fn inline(&mut self, e: &ast::InlineExpression<&str>) {
            match e {
                ast::InlineExpression::VariableReference { id } => {
                    self.vars.insert(id.name.to_string());
                }
                ast::InlineExpression::TermReference { id, arguments, .. } => {
                    self.terms.insert(id.name.to_string());
                    if let Some(a) = arguments {
                        self.call(a);
                    }
                }
                ast::InlineExpression::FunctionReference { arguments, .. } => self.call(arguments),
                ast::InlineExpression::Placeable { expression } => self.expr(expression),
                _ => {}
            }
        }

        fn call(&mut self, a: &ast::CallArguments<&str>) {
            a.positional.iter().for_each(|e| self.inline(e));
            a.named.iter().for_each(|n| self.inline(&n.value));
        }
    }

    #[test]
    fn every_translation_has_an_english_source_and_reads_only_its_variables() {
        let english: BTreeMap<String, BTreeSet<String>> = patterns(Lang::En)
            .into_iter()
            .map(|(id, _, vars)| (id, vars))
            .collect();
        for lang in Lang::ALL {
            for (id, file, vars) in patterns(lang) {
                let Some(source) = english.get(&id) else {
                    panic!("i18n/{}/{file}: {id} is not in English", lang.code());
                };
                let extra: Vec<_> = vars.difference(source).collect();
                assert!(
                    extra.is_empty(),
                    "i18n/{}/{file}: {id} reads {extra:?}, which English never passes",
                    lang.code()
                );
            }
        }
    }

    fn patterns_of(lang: Lang) -> Vec<String> {
        patterns(lang).into_iter().map(|(id, _, _)| id).collect()
    }

    #[test]
    fn every_term_a_language_uses_is_defined_in_that_language() {
        // A bundle reads only its own language's files, so a `-term` defined in English alone
        // is an error everywhere else, and the whole message falls back to English
        for lang in Lang::ALL {
            let (mut defined, mut used) = (BTreeSet::new(), Refs::default());
            for (_, text) in LANGS[lang.index()].files {
                let resource = FluentResource::try_new(text.to_string()).unwrap();
                for entry in resource.entries() {
                    match entry {
                        Entry::Term(t) => {
                            defined.insert(t.id.name.to_string());
                            used.pattern(&t.value);
                        }
                        Entry::Message(m) => {
                            if let Some(v) = &m.value {
                                used.pattern(v);
                            }
                            m.attributes.iter().for_each(|a| used.pattern(&a.value));
                        }
                        _ => {}
                    }
                }
            }
            let missing: Vec<_> = used.terms.difference(&defined).collect();
            assert!(
                missing.is_empty(),
                "i18n/{} uses {missing:?} without defining them",
                lang.code()
            );
        }
    }

    #[test]
    fn a_message_that_does_not_format_says_so() {
        let mut bundle: Bundle = FluentBundle::new_concurrent(vec![]);
        bundle.set_use_isolating(false);
        let text = "broken = { $cout } shows\nfine = { $count } shows\n";
        bundle
            .add_resource(FluentResource::try_new(text.to_string()).unwrap())
            .unwrap();
        let args = [("count", Arg::from(2))];
        assert!(!format(&bundle, "broken", &args).unwrap().1);
        assert_eq!(
            format(&bundle, "fine", &args),
            Some(("2 shows".to_string(), true))
        );
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
    fn a_fraction_takes_the_language_s_decimal_mark() {
        let ep = |lang| tr_in(lang, "episode-number", &[("number", 12.5.into())]);
        assert_eq!(ep(Lang::En), "Episode 12.5");
        assert!(ep(Lang::Da).contains("12,5"), "{}", ep(Lang::Da));
        assert!(ep(Lang::Ja).contains("12.5"), "{}", ep(Lang::Ja));
        let whole = tr_in(Lang::De, "episode-number", &[("number", 12.into())]);
        assert!(whole.contains("12") && !whole.contains(','), "{whole}");
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
        let made = |ftl: &str| {
            let mut b: Bundle = FluentBundle::new_concurrent(vec![]);
            b.set_use_isolating(false);
            b.add_resource(FluentResource::try_new(ftl.to_string()).unwrap())
                .unwrap();
            b
        };
        let english = made("only-english = Hello\nboth = Hello both\nbroken = { $n } left\n");
        let own = made("both = Hej begge\nbroken = { $m } tilbage\n");
        let n = [("n", Arg::from(2))];
        assert_eq!(words_from(Some(&own), &english, "both", &[]), "Hej begge");
        assert_eq!(
            words_from(Some(&own), &english, "only-english", &[]),
            "Hello"
        );
        assert_eq!(words_from(Some(&own), &english, "broken", &n), "2 left");
        assert_eq!(words_from(Some(&own), &english, "nowhere", &[]), "nowhere"); // i18n: none

        let args = [("count", Arg::from(2))];
        // Any message one language lacks reads as the English, while translations are partial
        for lang in Lang::ALL {
            let missing = (patterns_of(Lang::En).into_iter())
                .find(|id| !id.contains('.') && lookup(bundle(lang), id).is_none());
            if let Some(id) = missing {
                assert_eq!(tr_in(lang, &id, &[]), tr_in(Lang::En, &id, &[]), "{id}");
            }
        }
        assert_eq!(tr_in(Lang::De, "no-such-message", &[]), "no-such-message"); // i18n: none
        let attr = tr_in(Lang::En, "count-episodes.tip", &args); // i18n: none
        assert_eq!(attr, "count-episodes.tip");
        assert!(exists("count-episodes"));
        assert!(!exists("no-such-message"));
    }
}
