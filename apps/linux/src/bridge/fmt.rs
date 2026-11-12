//! Fmt: the format helpers as a QML singleton. Every method is a pure call into `format`.

use cxx_qt_lib::QString;
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
        #[qinvokable]
        fn score(self: &Fmt, x: f64) -> QString;
        /// An episode's title, else "Episode N"; pass "" for no title.
        #[qinvokable]
        fn episode_name(self: &Fmt, title: &QString, number: f64) -> QString;
        /// "Episode 7", or "Episodes 7 to 9" for a drop of several; `through` 0 for one.
        #[qinvokable]
        fn episode_span(self: &Fmt, first: i32, through: i32) -> QString;
        /// The URL of a local path for an Image source or openLink, "" for "". A value that
        /// already carries a scheme (file:, qrc:, http:, https:, image:) comes back as given.
        #[qinvokable]
        fn file_url(self: &Fmt, path: &QString) -> QString;
        /// The local path of a file URL, percent-decoded; anything that is not a file URL
        /// comes back as given.
        #[qinvokable]
        fn local_path(self: &Fmt, url: &QString) -> QString;
        /// The last part of a path, split on either separator, trailing separators ignored.
        #[qinvokable]
        fn base_name(self: &Fmt, path: &QString) -> QString;
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
    pub fn file_url(&self, path: &QString) -> QString {
        QString::from(&file_url(&path.to_string(), cfg!(windows)))
    }
    pub fn local_path(&self, url: &QString) -> QString {
        QString::from(&local_path(&url.to_string(), cfg!(windows)))
    }
    pub fn base_name(&self, path: &QString) -> QString {
        QString::from(base_name(&path.to_string()))
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

/// The last part of a path. Both separators count on every platform, since a library can
/// hold paths written on either.
fn base_name(path: &str) -> &str {
    let path = path.trim_end_matches(['/', '\\']);
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::{base_name, file_url, local_path};

    #[test]
    fn base_name_takes_either_separator() {
        assert_eq!(base_name(""), "");
        assert_eq!(base_name("a.mkv"), "a.mkv");
        assert_eq!(base_name("/lib/Show/a.mkv"), "a.mkv");
        assert_eq!(base_name("C:\\lib\\Show\\a.mkv"), "a.mkv");
        assert_eq!(base_name("C:\\lib/Show\\"), "Show");
        assert_eq!(base_name("/lib/Show//"), "Show");
        assert_eq!(base_name("/"), "");
    }

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
}
