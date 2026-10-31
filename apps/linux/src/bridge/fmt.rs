//! Fmt: the format helpers as a QML singleton. Every method is a pure call into `format`,
//! except `posterTone`, which reads (and caches) a poster's average colour.

use cxx_qt_lib::{QColor, QString};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};

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
        /// "24 min", "1 h 52 min" or "2 h"; an unknown length (0, negative, NaN) reads empty.
        #[qinvokable]
        fn runtime(self: &Fmt, minutes: f64) -> QString;
        #[qinvokable]
        fn plural(self: &Fmt, n: f64, one: &QString, many: &QString) -> QString;
        /// `watched` and `total` are -1 for none.
        #[qinvokable]
        fn watched_chip(self: &Fmt, watched: i32, total: i32, estimate: bool) -> QString;
        #[qinvokable]
        fn score(self: &Fmt, x: f64) -> QString;
        /// An episode's title, else "Episode N"; pass "" for no title.
        #[qinvokable]
        fn episode_name(self: &Fmt, title: &QString, number: f64) -> QString;
        /// "Episode 7", or "Episodes 7 to 9" for a drop of several; `through` 0 for one.
        #[qinvokable]
        fn episode_span(self: &Fmt, first: i32, through: i32) -> QString;
        /// The poster's tone as "#rrggbb", or "" for no poster or an unreadable one. Takes a
        /// path or a file:// URL.
        #[qinvokable]
        fn poster_tone(self: &Fmt, path: &QString) -> QString;
        /// The URL of a local path for an Image source or openLink, "" for "". A value that
        /// already carries a scheme (file:, qrc:, http:, https:, image:) comes back as given.
        #[qinvokable]
        fn file_url(self: &Fmt, path: &QString) -> QString;
        /// The local path of a file URL, percent-decoded; anything that is not a file URL
        /// comes back as given.
        #[qinvokable]
        fn local_path(self: &Fmt, url: &QString) -> QString;
        /// A folder's name as a title to search for: release-group brackets, a trailing
        /// episode range and a trailing END gone.
        #[qinvokable]
        fn folder_title(self: &Fmt, name: &QString) -> QString;
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
    pub fn runtime(&self, minutes: f64) -> QString {
        if minutes.is_nan() || minutes < 0.5 {
            return QString::default();
        }
        QString::from(&crate::format::runtime(minutes.round() as u32))
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
    pub fn episode_name(&self, title: &QString, number: f64) -> QString {
        let t = title.to_string();
        QString::from(crate::format::episode_name(Some(t.as_str()), number))
    }
    pub fn episode_span(&self, first: i32, through: i32) -> QString {
        QString::from(&crate::format::episode_span(
            first.max(0) as u32,
            through.max(0) as u32,
        ))
    }
    pub fn poster_tone(&self, path: &QString) -> QString {
        QString::from(&tone(&path.to_string()))
    }
    pub fn file_url(&self, path: &QString) -> QString {
        QString::from(&file_url(&path.to_string(), cfg!(windows)))
    }
    pub fn local_path(&self, url: &QString) -> QString {
        QString::from(&local_path(&url.to_string(), cfg!(windows)))
    }

    pub fn folder_title(&self, name: &QString) -> QString {
        QString::from(&anibeam_core::library::walk::clean_folder_title(
            &name.to_string(),
        ))
    }
}

/// The characters a URL would read as syntax inside a path: `%`, `#` and `?`. Spaces and
/// non-ASCII stay as they are, which is how QML has always been handed these paths.
const URL_PATH: &AsciiSet = &CONTROLS.add(b'%').add(b'#').add(b'?');

const SCHEMES: [&str; 5] = ["file:", "qrc:", "http:", "https:", "image:"];

/// A local path as a URL. `windows` turns backslashes into slashes and puts a drive path
/// under `file:///`; a `\\host\share` path becomes `file://host/share`.
fn file_url(path: &str, windows: bool) -> String {
    if path.is_empty() || SCHEMES.iter().any(|s| path.starts_with(s)) {
        return path.to_string();
    }
    let path = if windows {
        path.replace('\\', "/")
    } else {
        path.to_string()
    };
    let enc = |p: &str| utf8_percent_encode(p, URL_PATH).to_string();
    if windows && path.starts_with("//") {
        format!("file:{}", enc(&path))
    } else if path.starts_with('/') {
        format!("file://{}", enc(&path))
    } else {
        format!("file:///{}", enc(&path))
    }
}

/// The local path of a `file:` URL. `windows` drops the slash before a drive letter.
fn local_path(url: &str, windows: bool) -> String {
    let Some(rest) = url.strip_prefix("file:") else {
        return url.to_string();
    };
    let decode = |p: &str| percent_decode_str(p).decode_utf8_lossy().into_owned();
    let Some(after) = rest.strip_prefix("//") else {
        return decode(rest);
    };
    // file://host/path: an empty or "localhost" host is this machine
    let (host, path) = match after.find('/') {
        Some(i) => after.split_at(i),
        None => (after, ""),
    };
    let path = decode(path);
    if !host.is_empty() && host != "localhost" {
        return format!("//{host}{path}");
    }
    let bytes = path.as_bytes();
    if windows
        && bytes.len() >= 3
        && bytes[0] == b'/'
        && bytes[1].is_ascii_alphabetic()
        && bytes[2] == b':'
    {
        return path[1..].to_string();
    }
    path
}

/// The poster tone of a path or file:// URL as "#rrggbb", "" when there is none.
fn tone(path: &str) -> String {
    let path = local_path(path, cfg!(windows));
    if path.is_empty() {
        return String::new();
    }
    hex(&crate::bridge::helpers::ffi::poster_tone(&QString::from(
        path.as_str(),
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
    use super::{file_url, local_path, tone};

    #[test]
    fn file_url_by_platform() {
        assert_eq!(file_url("", false), "");
        assert_eq!(file_url("/home/x/a b.jpg", false), "file:///home/x/a b.jpg");
        assert_eq!(file_url("/a/#b/c?.png", false), "file:///a/%23b/c%3F.png");
        assert_eq!(file_url("/a/100%.png", false), "file:///a/100%25.png");
        assert_eq!(file_url("/home/x\\y", false), "file:///home/x\\y");
        assert_eq!(
            file_url("C:\\Users\\x\\AppData\\a.jpg", true),
            "file:///C:/Users/x/AppData/a.jpg"
        );
        assert_eq!(file_url("C:/lib/Show", true), "file:///C:/lib/Show");
        assert_eq!(
            file_url("\\\\nas\\share\\a.jpg", true),
            "file://nas/share/a.jpg"
        );
        assert_eq!(
            file_url("C:\\lib\\a #1.jpg", true),
            "file:///C:/lib/a %231.jpg"
        );
    }

    #[test]
    fn file_url_passes_urls_through() {
        for u in [
            "file:///C:/x.jpg",
            "qrc:/a.png",
            "http://h/a.jpg",
            "https://h/a.jpg",
            "image://thumb/1",
        ] {
            assert_eq!(file_url(u, true), u);
            assert_eq!(file_url(u, false), u);
        }
    }

    #[test]
    fn local_path_by_platform() {
        assert_eq!(local_path("file:///home/x", false), "/home/x");
        assert_eq!(local_path("file:///C:/lib", true), "C:/lib");
        assert_eq!(local_path("file:///C:/lib", false), "/C:/lib");
        assert_eq!(local_path("file:///a/b%20c%23d", false), "/a/b c#d");
        assert_eq!(
            local_path("file:///C:/My%20Anime/%E3%81%82", true),
            "C:/My Anime/\u{3042}"
        );
        assert_eq!(local_path("file://localhost/home/x", false), "/home/x");
        assert_eq!(local_path("file://nas/share/a", true), "//nas/share/a");
        assert_eq!(local_path("file:/home/x", false), "/home/x");
        assert_eq!(local_path("/plain/100%20path", false), "/plain/100%20path");
        assert_eq!(local_path("C:\\plain", true), "C:\\plain");
        assert_eq!(local_path("", true), "");
    }

    #[test]
    fn local_path_inverts_file_url() {
        for (p, w) in [
            ("/a/#b/c?.png", false),
            ("/x/100%/y z.jpg", false),
            ("C:\\a #\\b%.jpg", true),
        ] {
            let back = local_path(&file_url(p, w), w);
            assert_eq!(
                back,
                if w {
                    p.replace('\\', "/")
                } else {
                    p.to_string()
                }
            );
        }
    }

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
            tone(&file_url(&p, cfg!(windows))),
            first,
            "a URL reads the same file"
        );
        std::fs::remove_file(&p).unwrap();
        assert_eq!(tone(&p), first, "cached per path for the process");
        assert_eq!(tone(""), "");
        assert_eq!(tone(&dir.path().join("missing.jpg").to_string_lossy()), "");
    }
}
