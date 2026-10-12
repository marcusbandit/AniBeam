//! Fmt: the format helpers as a QML singleton. Every method is a pure call into `format`,
//! except `posterTone`, which reads (and caches) a poster's average colour.

use cxx_qt_lib::{QColor, QString};

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
        /// The poster's tone as "#rrggbb", or "" for no poster or an unreadable one. Takes a
        /// path or a file:// URL.
        #[qinvokable]
        fn poster_tone(self: &Fmt, path: &QString) -> QString;
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
    pub fn poster_tone(&self, path: &QString) -> QString {
        QString::from(&tone(&path.to_string()))
    }
}

/// The poster tone of a path or file:// URL as "#rrggbb", "" when there is none.
fn tone(path: &str) -> String {
    let path = path.strip_prefix("file://").unwrap_or(path);
    if path.is_empty() {
        return String::new();
    }
    hex(&crate::bridge::helpers::ffi::poster_tone(&QString::from(
        path,
    )))
}

fn hex(c: &QColor) -> String {
    if !c.is_valid() {
        return String::new();
    }
    format!("#{:02x}{:02x}{:02x}", c.red(), c.green(), c.blue())
}

#[cfg(test)]
mod tests {
    use super::tone;

    /// A binary PPM (Qt reads it with no plugin): `w` x `h`, pixel (x, y) from `at`.
    fn ppm(
        dir: &std::path::Path,
        name: &str,
        w: u32,
        h: u32,
        at: impl Fn(u32, u32) -> [u8; 3],
    ) -> String {
        let mut bytes = format!("P6\n{w} {h}\n255\n").into_bytes();
        for y in 0..h {
            for x in 0..w {
                bytes.extend_from_slice(&at(x, y));
            }
        }
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path.to_string_lossy().into_owned()
    }

    fn rgb(hex: &str) -> [i32; 3] {
        let v = |i: usize| i32::from_str_radix(&hex[i..i + 2], 16).unwrap();
        [v(1), v(3), v(5)]
    }

    #[test]
    fn a_flat_poster_is_its_own_colour() {
        let dir = tempfile::tempdir().unwrap();
        let p = ppm(dir.path(), "flat.ppm", 40, 60, |_, _| [0x30, 0x60, 0x90]);
        let [r, g, b] = rgb(&tone(&p));
        assert!(
            (r - 0x30).abs() <= 2 && (g - 0x60).abs() <= 2 && (b - 0x90).abs() <= 2,
            "{}",
            tone(&p)
        );
    }

    #[test]
    fn a_mostly_white_poster_with_a_strong_colour_keeps_its_hue() {
        // three quarters white, a quarter pure red: a plain mean is a pale pink, the weighted
        // one leans well toward the red
        let dir = tempfile::tempdir().unwrap();
        let p = ppm(dir.path(), "white.ppm", 40, 60, |_, y| {
            if y < 45 {
                [255, 255, 255]
            } else {
                [220, 20, 20]
            }
        });
        let [r, g, b] = rgb(&tone(&p));
        assert!(r > g + 60 && r > b + 60, "red dominates: {}", tone(&p));
        let plain_mean_green = (255 * 3 + 20) / 4;
        assert!(
            g < plain_mean_green - 30,
            "weighted past the plain mean: {}",
            tone(&p)
        );
    }

    #[test]
    fn deterministic_cached_and_empty_for_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let p = ppm(dir.path(), "grad.ppm", 30, 45, |x, y| {
            [(x * 8) as u8, (y * 5) as u8, 128]
        });
        let first = tone(&p);
        assert_eq!(first.len(), 7);
        assert_eq!(
            tone(&format!("file://{p}")),
            first,
            "a URL reads the same file"
        );
        std::fs::remove_file(&p).unwrap();
        assert_eq!(tone(&p), first, "cached per path for the process");
        assert_eq!(tone(""), "");
        assert_eq!(tone(&dir.path().join("missing.jpg").to_string_lossy()), "");
    }
}
