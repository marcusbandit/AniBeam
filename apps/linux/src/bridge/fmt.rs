//! Fmt: the format helpers as a QML singleton. Every method is a pure call into `format`.

use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type Fmt = super::FmtRust;

        #[qinvokable]
        fn relative(self: &Fmt, ts: f64, now: f64) -> QString;
        #[qinvokable]
        fn countdown(self: &Fmt, secs: f64) -> QString;
        #[qinvokable]
        fn countdown_seconds(self: &Fmt, secs: f64) -> QString;
        #[qinvokable]
        fn clock(self: &Fmt, secs: f64) -> QString;
        #[qinvokable]
        fn clock_ms(self: &Fmt, secs: f64) -> QString;
        #[qinvokable]
        fn bytes(self: &Fmt, n: f64) -> QString;
        #[qinvokable]
        fn plural(self: &Fmt, n: f64, one: &QString, many: &QString) -> QString;
        /// `watched` and `total` are -1 for none.
        #[qinvokable]
        fn watched_chip(self: &Fmt, watched: i32, total: i32, estimate: bool) -> QString;
        #[qinvokable]
        fn score(self: &Fmt, x: f64) -> QString;
    }
}

#[derive(Default)]
pub struct FmtRust;

impl qobject::Fmt {
    pub fn relative(&self, ts: f64, now: f64) -> QString {
        QString::from(&crate::format::relative(ts, now))
    }
    pub fn countdown(&self, secs: f64) -> QString {
        QString::from(&crate::format::countdown(secs))
    }
    pub fn countdown_seconds(&self, secs: f64) -> QString {
        QString::from(&crate::format::countdown_seconds(secs))
    }
    pub fn clock(&self, secs: f64) -> QString {
        QString::from(&crate::format::clock(secs))
    }
    pub fn clock_ms(&self, secs: f64) -> QString {
        QString::from(&crate::format::clock_ms(secs))
    }
    pub fn bytes(&self, n: f64) -> QString {
        QString::from(&crate::format::bytes(n.max(0.0) as u64))
    }
    pub fn plural(&self, n: f64, one: &QString, many: &QString) -> QString {
        QString::from(&crate::format::plural(
            n.max(0.0) as u64,
            &one.to_string(),
            &many.to_string(),
        ))
    }
    pub fn watched_chip(&self, watched: i32, total: i32, estimate: bool) -> QString {
        let opt = |v: i32| if v < 0 { None } else { Some(v as u32) };
        QString::from(&crate::format::watched_chip(
            opt(watched),
            opt(total),
            estimate,
        ))
    }
    pub fn score(&self, x: f64) -> QString {
        QString::from(&crate::format::score(x))
    }
}
