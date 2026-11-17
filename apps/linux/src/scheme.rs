//! `anibeam://` links reach the app (#888). An install made before rooms wrote its
//! desktop entry without `x-scheme-handler/anibeam`, so a browser's "Open in Anibeam"
//! went nowhere. At launch the user's own entry for this app gets the MimeType line
//! and a `%U` on its Exec when it has no field code for files or links yet, the desktop's cache is rebuilt, and
//! the entry becomes the handler when nothing else is.
//!
//! Only the user's entry is touched (`$XDG_DATA_HOME/applications`), never a system
//! one, and nothing at all when there is none. It runs off the main thread, says
//! nothing, and a failure leaves startup alone.

/// Checks the entry once, in the background. A sandbox run (`--root`, a drive or a shot)
/// leaves the user's files alone.
pub fn ensure() {
    #[cfg(target_os = "linux")]
    {
        let args = crate::runtime::args();
        if args.root.is_some() || args.drive.is_some() || args.shoot.is_some() {
            return;
        }
        let _ = std::thread::Builder::new()
            .name("scheme".into())
            .spawn(linux::ensure);
    }
}

/// The scheme's MIME type, as desktop entries and xdg-mime name it.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
const MIME: &str = "x-scheme-handler/anibeam";

/// `entry` with the MimeType and the `%U` it needs, or `None` when it has both. An Exec
/// that already takes files or links (`%f`, `%F`, `%u` or `%U`) stays as it is: the spec
/// allows one of the four. Only the `[Desktop Entry]` group changes; actions and other
/// groups keep their own Exec lines.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn patch(entry: &str) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    let mut in_main = false;
    let mut main_end: Option<usize> = None;
    let mut has_mime = false;
    let mut changed = false;
    for line in entry.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if in_main {
                main_end = Some(out.len());
            }
            in_main = trimmed == "[Desktop Entry]";
            out.push(line.to_owned());
            continue;
        }
        if !in_main {
            out.push(line.to_owned());
            continue;
        }
        if let Some(value) = line.strip_prefix("MimeType=") {
            has_mime = true;
            if value.split(';').any(|m| m.trim() == MIME) {
                out.push(line.to_owned());
            } else {
                let value = value.trim_end();
                let sep = if value.is_empty() || value.ends_with(';') {
                    ""
                } else {
                    ";"
                };
                out.push(format!("MimeType={value}{sep}{MIME};"));
                changed = true;
            }
        } else if let Some(value) = line.strip_prefix("Exec=") {
            if ["%f", "%F", "%u", "%U"]
                .iter()
                .any(|code| value.contains(code))
            {
                out.push(line.to_owned());
            } else {
                out.push(format!("Exec={} %U", value.trim_end()));
                changed = true;
            }
        } else {
            out.push(line.to_owned());
        }
    }
    if in_main && main_end.is_none() {
        main_end = Some(out.len());
    }
    let main_end = main_end?;
    if !has_mime {
        // After the group's last line that says something, so a blank line before the
        // next group stays where it was
        let mut at = main_end;
        while at > 0 && out[at - 1].trim().is_empty() {
            at -= 1;
        }
        out.insert(at, format!("MimeType={MIME};"));
        changed = true;
    }
    if !changed {
        return None;
    }
    let mut text = out.join("\n");
    if entry.ends_with('\n') {
        text.push('\n');
    }
    Some(text)
}

/// The program an Exec line runs: the first word that is not `env` or a `NAME=value`
/// assignment, unquoted.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
pub(crate) fn program(exec: &str) -> Option<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut any = false;
    let mut chars = exec.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                quoted = !quoted;
                any = true;
            }
            '\\' if quoted => {
                if let Some(next) = chars.next() {
                    word.push(next);
                }
            }
            c if c.is_whitespace() && !quoted => {
                if any || !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                    any = false;
                }
            }
            c => word.push(c),
        }
    }
    if any || !word.is_empty() {
        words.push(word);
    }
    let mut words = words.into_iter().peekable();
    if words
        .peek()
        .is_some_and(|w| w == "env" || w.ends_with("/env"))
    {
        words.next();
    }
    words.find(|w| !(w.contains('=') && !w.starts_with('/')) && !w.starts_with('-'))
}

#[cfg(target_os = "linux")]
pub(crate) mod linux {
    use std::ffi::OsStr;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use super::{MIME, patch, program};

    const MAIN: &str = "com.marcusrosado.Anibeam.desktop";
    const DEV: &str = "com.marcusrosado.Anibeam.Dev.desktop";

    /// The user's own applications folder, `$XDG_DATA_HOME/applications`.
    pub fn applications() -> Option<PathBuf> {
        let data = std::env::var_os("XDG_DATA_HOME")
            .filter(|d| !d.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
        Some(data.join("applications"))
    }

    /// What is running: the AppImage file rather than its mount, else the binary.
    fn running() -> Option<PathBuf> {
        let exe = std::env::var_os("APPIMAGE")
            .filter(|a| !a.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::current_exe().ok())?;
        exe.canonicalize().ok()
    }

    /// The data folders a package's entry can live under: `$XDG_DATA_DIRS`, absolute
    /// paths only as the spec asks, else its default `/usr/local/share:/usr/share`.
    fn data_dirs() -> Vec<PathBuf> {
        let dirs: Vec<PathBuf> = std::env::var_os("XDG_DATA_DIRS")
            .map(|v| {
                std::env::split_paths(&v)
                    .filter(|p| p.is_absolute())
                    .collect()
            })
            .unwrap_or_default();
        if dirs.is_empty() {
            vec!["/usr/local/share".into(), "/usr/share".into()]
        } else {
            dirs
        }
    }

    /// The app's entry in the first of `dirs` that has one: what a package installed.
    pub(crate) fn system_entry_in(dirs: &[PathBuf]) -> Option<PathBuf> {
        dirs.iter()
            .map(|d| d.join("applications").join(MAIN))
            .find(|p| p.is_file())
    }

    /// Where `program` lives: itself when it names a path, else the first match on
    /// `path` (a `$PATH` value).
    pub(crate) fn resolve(program: &str, path: Option<&OsStr>) -> Option<PathBuf> {
        let found = if program.contains('/') {
            Some(PathBuf::from(program))
        } else {
            std::env::split_paths(path?)
                .filter(|d| d.is_absolute())
                .map(|d| d.join(program))
                .find(|p| p.is_file())
        }?;
        found.canonicalize().ok().filter(|p| p.is_file())
    }

    /// The program the entry's Exec runs, where it lives.
    fn target(entry: &Path, path: Option<&OsStr>) -> Option<PathBuf> {
        std::fs::read_to_string(entry)
            .ok()?
            .lines()
            .find_map(|l| l.strip_prefix("Exec="))
            .and_then(program)
            .and_then(|p| resolve(&p, path))
    }

    fn runs(entry: &Path, exe: &Path) -> bool {
        target(entry, std::env::var_os("PATH").as_deref()).is_some_and(|p| p == exe)
    }

    /// The entry that launched this run: the one the launcher named, else the one whose
    /// Exec is this binary, else the app's own entry when it exists.
    fn pick(dir: &Path) -> Option<&'static str> {
        let ours = [MAIN, DEV];
        if let Some(launched) = std::env::var_os("GIO_LAUNCHED_DESKTOP_FILE").map(PathBuf::from)
            && launched.parent() == Some(dir)
            && let Some(name) = ours
                .iter()
                .find(|n| launched.file_name() == Some(n.as_ref()))
        {
            return Some(name);
        }
        if let Some(exe) = running()
            && let Some(name) = ours.iter().find(|n| runs(&dir.join(n), &exe))
        {
            return Some(name);
        }
        dir.join(MAIN).is_file().then_some(MAIN)
    }

    pub fn ensure() {
        let Some(dir) = applications() else { return };
        let Some(name) = pick(&dir) else { return };
        let path = dir.join(name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return;
        };
        if let Some(patched) = patch(&text) {
            if !replace(&path, &patched) {
                return;
            }
            quiet(Command::new("update-desktop-database").arg(&dir));
        }
        let current = Command::new("xdg-mime")
            .args(["query", "default", MIME])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        if let Ok(out) = current
            && String::from_utf8_lossy(&out.stdout).trim().is_empty()
        {
            quiet(Command::new("xdg-mime").args(["default", name, MIME]));
        }
    }

    /// The entry that started this run, in the user's folder or a package's, and whether
    /// it is the user's own, for another handler to start the app the same way (#966,
    /// #1167).
    pub fn launching_entry() -> Option<(PathBuf, bool)> {
        let launched = std::env::var_os("GIO_LAUNCHED_DESKTOP_FILE").map(PathBuf::from);
        choose(
            applications().as_deref(),
            system_entry_in(&data_dirs()).as_deref(),
            launched.as_deref(),
            running().as_deref(),
            std::env::var_os("PATH").as_deref(),
        )
    }

    /// The pick behind `launching_entry`: the entry the launcher named; else the one whose
    /// Exec is this binary, the user's first; else the user's own entry, then the
    /// package's, whichever still runs something. A user entry left behind by an AppImage
    /// that is gone never wins over a package that is there.
    pub(crate) fn choose(
        user: Option<&Path>,
        system: Option<&Path>,
        launched: Option<&Path>,
        exe: Option<&Path>,
        path: Option<&OsStr>,
    ) -> Option<(PathBuf, bool)> {
        let ours = [MAIN, DEV];
        let users: Vec<PathBuf> = user
            .map(|d| {
                ours.iter()
                    .map(|n| d.join(n))
                    .filter(|p| p.is_file())
                    .collect()
            })
            .unwrap_or_default();
        let mine = |p: &Path| user.is_some_and(|d| p.parent() == Some(d));
        if let Some(launched) = launched
            && launched.is_file()
            && (users.iter().any(|p| p == launched) || system == Some(launched))
        {
            return Some((launched.to_path_buf(), mine(launched)));
        }
        let live = |p: &Path| target(p, path);
        if let Some(exe) = exe {
            let runs = |p: &&Path| live(p).is_some_and(|t| t == exe);
            if let Some(p) = users.iter().map(PathBuf::as_path).find(|p| runs(p)) {
                return Some((p.to_path_buf(), true));
            }
            if let Some(p) = system.filter(|p| runs(p)) {
                return Some((p.to_path_buf(), false));
            }
        }
        if let Some(p) = users
            .iter()
            .find(|p| p.file_name() == Some(MAIN.as_ref()) && live(p).is_some())
        {
            return Some((p.clone(), true));
        }
        system
            .filter(|p| live(p).is_some())
            .map(|p| (p.to_path_buf(), mine(p)))
    }

    /// Puts `text` in the file at `path`: written beside the file and moved over it, so a
    /// reader never sees half a file. An entry that is a symlink (into a dotfiles checkout,
    /// say) gets its target written and stays a link.
    pub(crate) fn replace(path: &Path, text: &str) -> bool {
        let Ok(target) = path.canonicalize() else {
            return false;
        };
        let (Some(dir), Some(name)) = (target.parent(), target.file_name()) else {
            return false;
        };
        let tmp = dir.join(format!(".{}.anibeam-tmp", name.to_string_lossy()));
        let written = std::fs::write(&tmp, text).is_ok()
            && std::fs::metadata(&target)
                .and_then(|m| std::fs::set_permissions(&tmp, m.permissions()))
                .is_ok()
            && std::fs::rename(&tmp, &target).is_ok();
        if !written {
            let _ = std::fs::remove_file(&tmp);
        }
        written
    }

    pub(crate) fn quiet(cmd: &mut Command) {
        let _ = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD: &str = "[Desktop Entry]\nType=Application\nName=Anibeam\n\
        Exec=env APPIMAGELAUNCHER_DISABLE=1 /opt/anibeam %U\nIcon=com.marcusrosado.Anibeam\n\
        Terminal=false\n";

    #[test]
    fn an_entry_from_before_rooms_gets_the_scheme() {
        let patched = patch(OLD).unwrap();
        assert!(patched.ends_with("Terminal=false\nMimeType=x-scheme-handler/anibeam;\n"));
        assert!(patched.contains("Exec=env APPIMAGELAUNCHER_DISABLE=1 /opt/anibeam %U\n"));
        assert_eq!(patch(&patched), None);
    }

    #[test]
    fn an_exec_without_a_link_gets_one() {
        let patched = patch("[Desktop Entry]\nExec=env A=1 /x/anibeam\n").unwrap();
        assert_eq!(
            patched,
            "[Desktop Entry]\nExec=env A=1 /x/anibeam %U\nMimeType=x-scheme-handler/anibeam;\n"
        );
        for code in ["%f", "%F", "%u", "%U"] {
            assert_eq!(
                patch(&format!(
                    "[Desktop Entry]\nExec=anibeam {code}\nMimeType=x-scheme-handler/anibeam;\n"
                )),
                None,
                "{code}"
            );
        }
        assert_eq!(
            patch("[Desktop Entry]\nExec=anibeam %F\n").unwrap(),
            "[Desktop Entry]\nExec=anibeam %F\nMimeType=x-scheme-handler/anibeam;\n"
        );
    }

    #[test]
    fn other_types_stay_and_the_scheme_joins_them() {
        assert_eq!(
            patch("[Desktop Entry]\nExec=a %U\nMimeType=video/mp4;\n").unwrap(),
            "[Desktop Entry]\nExec=a %U\nMimeType=video/mp4;x-scheme-handler/anibeam;\n"
        );
        assert_eq!(
            patch("[Desktop Entry]\nExec=a %U\nMimeType=video/mp4\n").unwrap(),
            "[Desktop Entry]\nExec=a %U\nMimeType=video/mp4;x-scheme-handler/anibeam;\n"
        );
    }

    #[test]
    fn actions_keep_their_own_lines() {
        let entry = "[Desktop Entry]\nExec=a %U\n\n[Desktop Action play]\nExec=a --play\n";
        assert_eq!(
            patch(entry).unwrap(),
            "[Desktop Entry]\nExec=a %U\nMimeType=x-scheme-handler/anibeam;\n\n[Desktop Action play]\nExec=a --play\n"
        );
        assert_eq!(patch("Name=x\n"), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_symlinked_entry_stays_a_link() {
        let home = tempfile::tempdir().unwrap();
        let dotfiles = home.path().join("dotfiles");
        let apps = home.path().join("applications");
        std::fs::create_dir_all(&dotfiles).unwrap();
        std::fs::create_dir_all(&apps).unwrap();
        let real = dotfiles.join("entry.desktop");
        let link = apps.join("entry.desktop");
        std::fs::write(&real, OLD).unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert!(linux::replace(&link, "patched\n"));
        assert!(
            std::fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read_to_string(&real).unwrap(), "patched\n");
        assert_eq!(std::fs::read_dir(&apps).unwrap().count(), 1);
        assert_eq!(std::fs::read_dir(&dotfiles).unwrap().count(), 1);

        let plain = apps.join("plain.desktop");
        std::fs::write(&plain, OLD).unwrap();
        assert!(linux::replace(&plain, "again\n"));
        assert_eq!(std::fs::read_to_string(&plain).unwrap(), "again\n");
        assert!(!linux::replace(&apps.join("missing.desktop"), "x"));
    }

    /// A user folder, a package's data folder and a `bin` on the PATH, all empty.
    #[cfg(target_os = "linux")]
    struct Box1167 {
        _root: tempfile::TempDir,
        user: std::path::PathBuf,
        share: std::path::PathBuf,
        bin: std::path::PathBuf,
    }

    #[cfg(target_os = "linux")]
    impl Box1167 {
        fn new() -> Self {
            let root = tempfile::tempdir().unwrap();
            let user = root.path().join("data/applications");
            let share = root.path().join("share");
            let bin = root.path().join("bin");
            for d in [&user, &share.join("applications"), &bin] {
                std::fs::create_dir_all(d).unwrap();
            }
            Self {
                _root: root,
                user,
                share,
                bin,
            }
        }
        fn program(&self, name: &str) -> std::path::PathBuf {
            let p = self.bin.join(name);
            std::fs::write(&p, "").unwrap();
            p.canonicalize().unwrap()
        }
        fn entry(&self, dir: &std::path::Path, name: &str, exec: &str) -> std::path::PathBuf {
            let p = dir.join(name);
            std::fs::write(&p, format!("[Desktop Entry]\nExec={exec} %u\n")).unwrap();
            p
        }
        fn system(&self) -> Option<std::path::PathBuf> {
            linux::system_entry_in(std::slice::from_ref(&self.share))
        }
        fn choose(
            &self,
            launched: Option<&std::path::Path>,
            exe: Option<&std::path::Path>,
        ) -> Option<(std::path::PathBuf, bool)> {
            linux::choose(
                Some(&self.user),
                self.system().as_deref(),
                launched,
                exe,
                Some(self.bin.as_os_str()),
            )
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_package_s_entry_is_found_in_the_data_dirs() {
        let b = Box1167::new();
        assert_eq!(b.system(), None);
        let apps = b.share.join("applications");
        let e = b.entry(&apps, "com.marcusrosado.Anibeam.desktop", "anibeam");
        assert_eq!(
            linux::system_entry_in(&[b.bin.clone(), b.share.clone()]),
            Some(e)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_package_install_starts_from_the_package_s_entry() {
        let b = Box1167::new();
        let exe = b.program("anibeam");
        let apps = b.share.join("applications");
        let sys = b.entry(&apps, "com.marcusrosado.Anibeam.desktop", "anibeam");
        assert_eq!(b.choose(None, Some(&exe)), Some((sys.clone(), false)));
        // deb and rpm: /usr/bin/anibeam is a script that runs the binary in /opt
        let opt = b.program("opt-anibeam");
        assert_eq!(b.choose(None, Some(&opt)), Some((sys.clone(), false)));
        assert_eq!(b.choose(Some(&sys), None), Some((sys, false)));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_dead_appimage_s_entry_loses_to_the_package() {
        let b = Box1167::new();
        let exe = b.program("anibeam");
        let apps = b.share.join("applications");
        let sys = b.entry(&apps, "com.marcusrosado.Anibeam.desktop", "anibeam");
        let gone = b.bin.join("Anibeam.AppImage");
        b.entry(
            &b.user,
            "com.marcusrosado.Anibeam.desktop",
            &gone.display().to_string(),
        );
        assert_eq!(b.choose(None, Some(&exe)), Some((sys.clone(), false)));
        assert_eq!(b.choose(None, None), Some((sys, false)));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn the_user_s_own_entry_wins_when_it_runs_this_binary() {
        let b = Box1167::new();
        let appimage = b.program("Anibeam.AppImage");
        let apps = b.share.join("applications");
        b.program("anibeam");
        let sys = b.entry(&apps, "com.marcusrosado.Anibeam.desktop", "anibeam");
        let own = b.entry(
            &b.user,
            "com.marcusrosado.Anibeam.desktop",
            &appimage.display().to_string(),
        );
        let dev = b.entry(
            &b.user,
            "com.marcusrosado.Anibeam.Dev.desktop",
            "/nowhere/dev",
        );
        assert_eq!(b.choose(None, Some(&appimage)), Some((own.clone(), true)));
        assert_eq!(b.choose(Some(&dev), None), Some((dev, true)));
        assert_eq!(b.choose(Some(&sys), Some(&appimage)), Some((sys, false)));
        assert_eq!(
            b.choose(Some(&b.bin.join("other.desktop")), Some(&appimage)),
            Some((own, true))
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_relative_or_missing_program_resolves_to_nothing() {
        let b = Box1167::new();
        let exe = b.program("anibeam");
        let path = std::env::join_paths([std::path::Path::new("rel"), &b.bin]).unwrap();
        assert_eq!(linux::resolve("anibeam", Some(&path)), Some(exe.clone()));
        assert_eq!(linux::resolve(&exe.display().to_string(), None), Some(exe));
        assert_eq!(linux::resolve("anibeam", None), None);
        assert_eq!(linux::resolve("/nowhere/anibeam", None), None);
    }

    #[test]
    fn the_program_an_exec_runs() {
        assert_eq!(
            program("env APPIMAGELAUNCHER_DISABLE=1 /opt/example/bin/anibeam %U").as_deref(),
            Some("/opt/example/bin/anibeam")
        );
        assert_eq!(
            program("env A=/x B=2 \"/path with space/anibeam\" %u").as_deref(),
            Some("/path with space/anibeam")
        );
        assert_eq!(program("anibeam %u").as_deref(), Some("anibeam"));
    }
}
