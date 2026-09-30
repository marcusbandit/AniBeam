//! Theme: the singleton QML reads colours and the theme settings from. The engine pushes
//! resolutions through the Qt thread; the pick* invokables write settings through the
//! engine and apply the result at once, so a switch never waits on a file watcher.

use core::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{
    QColor, QJsonArray, QJsonObject, QJsonValue, QMap, QMapPair_QString_QVariant, QString, QVariant,
};
use tokio::sync::mpsc;

use crate::theme::config::{Corners, Density, ModeSetting, Poster, Source, ThemeSettings};
use crate::theme::engine::{Resolved, resolve};
use crate::theme::{Mode, Palette, format_hue, status_hue};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant = cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
        include!("cxx-qt-lib/qjsonarray.h");
        type QJsonArray = cxx_qt_lib::QJsonArray;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, mode)]
        #[qproperty(QString, source)]
        #[qproperty(i32, accent)]
        #[qproperty(QString, density)]
        #[qproperty(QString, poster)]
        #[qproperty(QString, corners)]
        #[qproperty(QString, theme_dark)]
        #[qproperty(QString, theme_light)]
        #[qproperty(QString, resolved_mode)]
        #[qproperty(QString, source_label)]
        #[qproperty(bool, contrast)]
        #[qproperty(QMap_QString_QVariant, dark)]
        #[qproperty(QMap_QString_QVariant, light)]
        #[qproperty(QJsonArray, themes)]
        #[qproperty(f64, density_factor)]
        #[qproperty(i32, poster_width)]
        #[qproperty(f64, smoothing)]
        #[qproperty(bool, ready)]
        type Theme = super::ThemeRust;

        #[qinvokable]
        fn pick_mode(self: Pin<&mut Self>, mode: &QString);
        #[qinvokable]
        fn pick_source(self: Pin<&mut Self>, source: &QString);
        #[qinvokable]
        fn pick_accent(self: Pin<&mut Self>, slot: i32);
        #[qinvokable]
        fn pick_density(self: Pin<&mut Self>, density: &QString);
        #[qinvokable]
        fn pick_poster(self: Pin<&mut Self>, poster: &QString);
        #[qinvokable]
        fn pick_corners(self: Pin<&mut Self>, corners: &QString);
        #[qinvokable]
        fn pick_theme(self: Pin<&mut Self>, mode: &QString, stem: &QString);
        #[qinvokable]
        fn format_hue(self: &Self, format: &QString) -> QString;
        #[qinvokable]
        fn status_hue(self: &Self, status: &QString) -> QString;
    }

    impl cxx_qt::Threading for Theme {}
    impl cxx_qt::Initialize for Theme {}
}

pub struct ThemeRust {
    mode: QString,
    source: QString,
    accent: i32,
    density: QString,
    poster: QString,
    corners: QString,
    theme_dark: QString,
    theme_light: QString,
    resolved_mode: QString,
    source_label: QString,
    contrast: bool,
    dark: QMap<QMapPair_QString_QVariant>,
    light: QMap<QMapPair_QString_QVariant>,
    themes: QJsonArray,
    density_factor: f64,
    poster_width: i32,
    smoothing: f64,
    ready: bool,
    resolved: Option<Resolved>,
    commands: Option<mpsc::UnboundedSender<ThemeSettings>>,
}

impl Default for ThemeRust {
    fn default() -> Self {
        ThemeRust {
            mode: QString::from("system"),
            source: QString::from("system"),
            accent: 4,
            density: QString::from("normal"),
            poster: QString::from("m"),
            corners: QString::from("smooth"),
            theme_dark: QString::from("anibeam-dark"),
            theme_light: QString::from("anibeam-light"),
            resolved_mode: QString::from("dark"),
            source_label: QString::default(),
            contrast: false,
            dark: QMap::default(),
            light: QMap::default(),
            themes: QJsonArray::default(),
            density_factor: 1.0,
            poster_width: 180,
            smoothing: 0.6,
            ready: false,
            resolved: None,
            commands: None,
        }
    }
}

fn qcolour(c: crate::theme::colour::Rgb) -> QColor {
    let (r, g, b) = c.bytes();
    QColor::from_rgb(i32::from(r), i32::from(g), i32::from(b))
}

fn colour_map(p: &Palette) -> QMap<QMapPair_QString_QVariant> {
    let mut m = QMap::<QMapPair_QString_QVariant>::default();
    for name in Palette::NAMES {
        let c = p.get(name).expect("every name resolves");
        let key = name.replace('.', "_");
        m.insert(QString::from(&key), QVariant::from(&qcolour(c)));
    }
    m
}

impl cxx_qt::Initialize for qobject::Theme {
    fn initialize(mut self: Pin<&mut Self>) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.as_mut().rust_mut().commands = Some(tx);
        let qt = self.qt_thread();
        let paths = crate::runtime::paths().clone();
        crate::runtime::runtime().spawn(crate::theme::engine::run(
            paths,
            move |resolved: Resolved| {
                qt.queue(move |theme: Pin<&mut qobject::Theme>| theme.apply(resolved))
                    .ok();
            },
            rx,
        ));
    }
}

impl qobject::Theme {
    /// Runs on the Qt thread: every property from one resolution.
    pub fn apply(mut self: Pin<&mut Self>, r: Resolved) {
        let s = &r.inputs.settings;
        self.as_mut().set_mode(QString::from(s.mode.as_str()));
        self.as_mut().set_source(QString::from(s.source.as_str()));
        self.as_mut().set_accent(i32::from(s.accent));
        self.as_mut().set_density(QString::from(s.density.as_str()));
        self.as_mut().set_poster(QString::from(s.poster.as_str()));
        self.as_mut().set_corners(QString::from(s.corners.as_str()));
        self.as_mut().set_theme_dark(QString::from(&s.theme_dark));
        self.as_mut().set_theme_light(QString::from(&s.theme_light));
        self.as_mut().set_density_factor(s.density.factor());
        self.as_mut().set_poster_width(s.poster.width());
        self.as_mut().set_smoothing(s.corners.smoothing());
        self.as_mut().set_contrast(r.inputs.portal.contrast);
        let mut themes = QJsonArray::default();
        for t in &r.inputs.themes {
            let mut o = QJsonObject::default();
            o.insert(
                &QString::from("stem"),
                &QJsonValue::from(&QString::from(t.stem())),
            );
            o.insert(
                &QString::from("name"),
                &QJsonValue::from(&QString::from(t.name())),
            );
            o.insert(
                &QString::from("mode"),
                &QJsonValue::from(&QString::from(t.mode().as_str())),
            );
            themes.append(&QJsonValue::from(&o));
        }
        self.as_mut().set_themes(themes);
        self.as_mut().set_dark(colour_map(&r.dark));
        self.as_mut().set_light(colour_map(&r.light));
        let current = match r.mode {
            Mode::Dark => &r.dark,
            Mode::Light => &r.light,
        };
        self.as_mut()
            .set_source_label(QString::from(&current.source_label));
        self.as_mut()
            .set_resolved_mode(QString::from(r.mode.as_str()));
        crate::bridge::helpers::ffi::set_app_palette(
            &qcolour(current.bg),
            &qcolour(current.text),
            &qcolour(current.surface_sunken),
            &qcolour(current.accent),
            &qcolour(current.accent_text),
            &qcolour(current.surface_raised),
            &qcolour(current.text),
        );
        self.as_mut().rust_mut().resolved = Some(r);
        self.as_mut().set_ready(true);
    }

    fn change(mut self: Pin<&mut Self>, edit: impl FnOnce(&mut ThemeSettings)) {
        let Some(mut r) = self.as_ref().resolved.clone() else {
            return;
        };
        edit(&mut r.inputs.settings);
        let settings = r.inputs.settings.clone();
        let fresh = resolve(r.inputs);
        self.as_mut().apply(fresh);
        if let Some(tx) = &self.as_ref().commands {
            tx.send(settings).ok();
        }
    }

    pub fn pick_mode(self: Pin<&mut Self>, mode: &QString) {
        if let Some(m) = ModeSetting::parse(&mode.to_string()) {
            self.change(|s| s.mode = m)
        }
    }
    pub fn pick_source(self: Pin<&mut Self>, source: &QString) {
        if let Some(v) = Source::parse(&source.to_string()) {
            self.change(|s| s.source = v)
        }
    }
    pub fn pick_accent(self: Pin<&mut Self>, slot: i32) {
        if (1..=7).contains(&slot) {
            self.change(|s| s.accent = slot as u8)
        }
    }
    pub fn pick_density(self: Pin<&mut Self>, density: &QString) {
        if let Some(v) = Density::parse(&density.to_string()) {
            self.change(|s| s.density = v)
        }
    }
    pub fn pick_poster(self: Pin<&mut Self>, poster: &QString) {
        if let Some(v) = Poster::parse(&poster.to_string()) {
            self.change(|s| s.poster = v)
        }
    }
    pub fn pick_corners(self: Pin<&mut Self>, corners: &QString) {
        if let Some(v) = Corners::parse(&corners.to_string()) {
            self.change(|s| s.corners = v)
        }
    }
    pub fn pick_theme(self: Pin<&mut Self>, mode: &QString, stem: &QString) {
        let stem = stem.to_string();
        match mode.to_string().as_str() {
            "dark" => self.change(|s| s.theme_dark = stem),
            "light" => self.change(|s| s.theme_light = stem),
            _ => {}
        }
    }
    pub fn format_hue(&self, format: &QString) -> QString {
        QString::from(format_hue(&format.to_string()))
    }
    pub fn status_hue(&self, status: &QString) -> QString {
        QString::from(status_hue(&status.to_string()))
    }
}
