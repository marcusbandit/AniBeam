//! Builds before 2.5 named the app `com.marcusrosado.Ani` + `Beam`, with a capital B (#1047).
//! The user's own desktop entries, launcher icons, `mimeapps.list` lines and screenshot folder
//! still carry that spelling, and the desktop matches the window to its entry by the id, so
//! at launch they are renamed once to the id this build runs under. The old spelling is
//! split in two below so a search for it finds nothing.
//!
//! Only the user's files are touched, never a system one. A file that already exists under
//! the new name is left as it is, a symlink is never followed, and a failure leaves startup
//! alone.

/// Renames what the old id left behind, before anything reads the entries.
pub fn adopt() {
    #[cfg(target_os = "linux")]
    {
        let args = crate::runtime::args();
        if args.root.is_some() || args.drive.is_some() || args.shoot.is_some() {
            return;
        }
        linux::adopt();
    }
}

/// The old id, and the folder name the old builds saved screenshots in.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
const OLD_ID: &str = concat!("com.marcusrosado.Ani", "Beam");
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
const OLD_FOLDER: &str = concat!("Ani", "Beam");

/// `name` under the new id, when it is one of the old id's files: the id itself or the id
/// followed by a dot (`.desktop`, `.Dev.png`).
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn renamed(name: &str) -> Option<String> {
    let rest = name.strip_prefix(OLD_ID)?;
    (rest.is_empty() || rest.starts_with('.')).then(|| format!("{}{rest}", crate::APP_ID))
}

/// A desktop entry under the new id: the id replaced everywhere, and the name the launcher
/// shows (`Name=`, `Comment=`, `GenericName=`, `X-GNOME-FullName=`, translations too) spelled
/// Anibeam. Other lines keep their words, so an `Exec=` path into a folder of the old name
/// still runs.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn respelled_entry(text: &str) -> String {
    text.replace(OLD_ID, crate::APP_ID)
        .split_inclusive('\n')
        .map(|line| {
            let key = line.split_once('=').map(|(k, _)| k.trim());
            let key = key.map(|k| k.split_once('[').map_or(k, |(k, _)| k).trim_end());
            if matches!(
                key,
                Some("Name" | "Comment" | "GenericName" | "X-GNOME-FullName")
            ) {
                line.replace(OLD_FOLDER, "Anibeam")
            } else {
                line.to_string()
            }
        })
        .collect()
}

#[cfg(target_os = "linux")]
mod linux {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::{OLD_FOLDER, OLD_ID, renamed, respelled_entry};

    fn data_home() -> Option<PathBuf> {
        xdg("XDG_DATA_HOME", ".local/share")
    }

    fn xdg(var: &str, fallback: &str) -> Option<PathBuf> {
        std::env::var_os(var)
            .filter(|d| !d.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(fallback)))
    }

    /// Every regular file in `dir` named under the old id, with its path under the new one.
    fn old_files(dir: &Path) -> Vec<(PathBuf, PathBuf)> {
        let Ok(read) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        read.flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .filter_map(|e| {
                let new = renamed(&e.file_name().to_string_lossy())?;
                Some((e.path(), dir.join(new)))
            })
            .collect()
    }

    /// Writes the entry at `from` under the new id and name, at `to`, and removes `from`.
    fn rewrite(from: &Path, to: &Path) -> bool {
        let Ok(text) = std::fs::read_to_string(from) else {
            return false;
        };
        let moved = std::fs::write(to, respelled_entry(&text)).is_ok()
            && std::fs::metadata(from)
                .and_then(|m| std::fs::set_permissions(to, m.permissions()))
                .is_ok();
        if moved {
            let _ = std::fs::remove_file(from);
        } else {
            let _ = std::fs::remove_file(to);
        }
        moved
    }

    /// Replaces the old id in a file that stays where it is, when it mentions it. A link
    /// (into a dotfiles checkout, say) gets its target written and stays a link.
    fn respell(path: &Path) {
        if let Ok(text) = std::fs::read_to_string(path)
            && text.contains(OLD_ID)
        {
            let _ = crate::scheme::linux::replace(path, &text.replace(OLD_ID, crate::APP_ID));
        }
    }

    /// Every `mimeapps.list` in `dir`, the desktop's own ones (`kde-mimeapps.list`) too.
    fn mime_lists(dir: &Path) -> Vec<PathBuf> {
        let Ok(read) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        read.flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with("mimeapps.list"))
            .map(|e| e.path())
            .collect()
    }

    /// Every regular desktop entry in `dir` named under the new id (`{APP_ID}.desktop` or
    /// `{APP_ID}.*.desktop`).
    fn new_desktop_entries(dir: &Path) -> Vec<PathBuf> {
        let Ok(read) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        read.flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .filter_map(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                let rest = name.strip_prefix(crate::APP_ID)?;
                (rest == ".desktop" || (rest.starts_with('.') && rest.ends_with(".desktop")))
                    .then(|| e.path())
            })
            .collect()
    }

    /// Fixes the visible name in desktop entries already under the new id, for installs
    /// that migrated under an earlier 2.5 build before the name lines were respelled.
    pub(super) fn respell_entries_in(data: &Path) -> bool {
        let apps = data.join("applications");
        let mut changed = false;
        for path in new_desktop_entries(&apps) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let next = respelled_entry(&text);
            if next != text && crate::scheme::linux::replace(&path, &next) {
                changed = true;
            }
        }
        changed
    }

    fn write_marker(path: &Path) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, "");
    }

    pub fn adopt() {
        let Some(data) = data_home() else { return };
        // Once per install: after the first launch of a renamed build there is nothing left
        // to look for, and a launch should not pay for the look.
        let data_dir = PathBuf::from(&crate::runtime::paths().core.data_dir);
        let done = data_dir.join("old-name-done");
        let respelled = data_dir.join("old-name-respelled");
        let mut changed = false;
        if !done.exists() {
            let ledger = data_dir.join("launcher-icons.txt");
            let config = xdg("XDG_CONFIG_HOME", ".config");
            changed |= adopt_in(&data, config.as_deref(), &ledger, pictures().as_deref());
            write_marker(&done);
        }
        if !respelled.exists() {
            changed |= respell_entries_in(&data);
            write_marker(&respelled);
        }
        if changed {
            let apps = data.join("applications");
            crate::scheme::linux::quiet(Command::new("update-desktop-database").arg(&apps));
        }
    }

    /// Does the renaming under these folders. True when a desktop entry moved or went, so
    /// the desktop's cache wants rebuilding.
    pub(super) fn adopt_in(
        data: &Path,
        config: Option<&Path>,
        ledger: &Path,
        pictures: Option<&Path>,
    ) -> bool {
        let apps = data.join("applications");
        let mut entries = false;
        for (from, to) in old_files(&apps) {
            if to.exists() {
                // The new entry is there already (the installer wrote it): the old one is a
                // second launcher entry with a dead icon, so it goes.
                entries |= std::fs::remove_file(&from).is_ok();
            } else {
                entries |= rewrite(&from, &to);
            }
        }

        let hicolor = data.join("icons/hicolor");
        let mut moved = Vec::new();
        if let Ok(sizes) = std::fs::read_dir(&hicolor) {
            for size in sizes.flatten() {
                for (from, to) in old_files(&size.path().join("apps")) {
                    if !to.exists() && std::fs::rename(&from, &to).is_ok() {
                        moved.push((from, to));
                    }
                }
            }
        }
        if !moved.is_empty() {
            // The launcher re-reads hicolor when the folder's time changes.
            let _ = std::fs::File::open(&hicolor)
                .and_then(|f| f.set_modified(std::time::SystemTime::now()));
            // The list of icons a Dark pick wrote follows the ones that moved, and only
            // those, so a Default pick never removes a file this build did not write.
            if let Ok(text) = std::fs::read_to_string(ledger) {
                let lines: String = text
                    .lines()
                    .map(|line| {
                        let path = Path::new(line);
                        let to = moved
                            .iter()
                            .find(|(from, _)| from == path)
                            .map(|(_, to)| to);
                        format!("{}\n", to.map_or(path, |t| t.as_path()).display())
                    })
                    .collect();
                if lines != text {
                    let _ = crate::scheme::linux::replace(ledger, &lines);
                }
            }
        }

        for dir in config.into_iter().chain([apps.as_path()]) {
            for list in mime_lists(dir) {
                respell(&list);
            }
        }

        if let Some(pictures) = pictures {
            let (from, to) = (pictures.join(OLD_FOLDER), pictures.join("Anibeam"));
            if from.is_dir() && !from.is_symlink() && !to.exists() {
                let _ = std::fs::rename(from, to);
            }
        }
        entries
    }

    /// The Pictures folder the player saves screenshots in: the XDG one, else ~/Pictures,
    /// which is where Qt's PicturesLocation falls back to. Never the home folder itself.
    fn pictures() -> Option<PathBuf> {
        let home = directories::BaseDirs::new()?.home_dir().to_path_buf();
        let dir = directories::UserDirs::new()
            .and_then(|u| u.picture_dir().map(Path::to_path_buf))
            .unwrap_or_else(|| home.join("Pictures"));
        (dir.is_absolute() && dir != home).then_some(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_old_ids_files_get_the_new_id() {
        let old = |rest: &str| format!("{OLD_ID}{rest}");
        assert_eq!(
            renamed(&old(".desktop")).as_deref(),
            Some("com.marcusrosado.Anibeam.desktop")
        );
        assert_eq!(
            renamed(&old(".Dev.png")).as_deref(),
            Some("com.marcusrosado.Anibeam.Dev.png")
        );
        assert_eq!(
            renamed(&old("")).as_deref(),
            Some("com.marcusrosado.Anibeam")
        );
        assert_eq!(renamed(&old("Fan.desktop")), None);
        assert_eq!(renamed("com.marcusrosado.Anibeam.desktop"), None);
        assert_eq!(renamed("org.other.App.desktop"), None);
    }

    #[test]
    fn a_moved_entry_shows_the_new_name() {
        let old = format!(
            "[Desktop Entry]\nName={OLD_FOLDER} Dev\nName[da]={OLD_FOLDER} Dev\n\
             GenericName = {OLD_FOLDER} player\nComment={OLD_FOLDER} for anime\n\
             X-GNOME-FullName={OLD_FOLDER}\n\
             Exec=/opt/{OLD_FOLDER}/anibeam\nIcon={OLD_ID}.Dev\nNoName={OLD_FOLDER}"
        );
        assert_eq!(
            respelled_entry(&old),
            format!(
                "[Desktop Entry]\nName=Anibeam Dev\nName[da]=Anibeam Dev\n\
                 GenericName = Anibeam player\nComment=Anibeam for anime\n\
                 X-GNOME-FullName=Anibeam\n\
                 Exec=/opt/{OLD_FOLDER}/anibeam\nIcon={}.Dev\nNoName={OLD_FOLDER}",
                crate::APP_ID
            )
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn an_already_moved_entry_still_gets_the_new_name() {
        use std::fs;
        let home =
            std::env::temp_dir().join(format!("anibeam-old-name-respell-{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        let data = home.join("data");
        let apps = data.join("applications");
        fs::create_dir_all(&apps).unwrap();

        let main = apps.join(format!("{}.desktop", crate::APP_ID));
        let dev = apps.join(format!("{}.Dev.desktop", crate::APP_ID));
        let other = apps.join(format!("{}Fan.desktop", crate::APP_ID));
        fs::write(
            &main,
            format!(
                "[Desktop Entry]\nName={OLD_FOLDER}\nExec=/opt/{OLD_FOLDER}/anibeam\nIcon={}\n",
                crate::APP_ID
            ),
        )
        .unwrap();
        fs::write(
            &dev,
            format!("[Desktop Entry]\nName={OLD_FOLDER} Dev\nX-GNOME-FullName={OLD_FOLDER} Dev\n"),
        )
        .unwrap();
        fs::write(&other, format!("[Desktop Entry]\nName={OLD_FOLDER} Fan\n")).unwrap();

        assert!(linux::respell_entries_in(&data));
        assert_eq!(
            fs::read_to_string(&main).unwrap(),
            format!(
                "[Desktop Entry]\nName=Anibeam\nExec=/opt/{OLD_FOLDER}/anibeam\nIcon={}\n",
                crate::APP_ID
            )
        );
        assert_eq!(
            fs::read_to_string(&dev).unwrap(),
            "[Desktop Entry]\nName=Anibeam Dev\nX-GNOME-FullName=Anibeam Dev\n"
        );
        assert_eq!(
            fs::read_to_string(&other).unwrap(),
            format!("[Desktop Entry]\nName={OLD_FOLDER} Fan\n")
        );
        // A second pass sees nothing left to change.
        assert!(!linux::respell_entries_in(&data));
        let _ = fs::remove_dir_all(&home);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn an_old_install_takes_the_new_id() {
        use std::fs;
        let home = std::env::temp_dir().join(format!("anibeam-old-name-{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        let (data, config, pictures) = (
            home.join("data"),
            home.join("config"),
            home.join("Pictures"),
        );
        let apps = data.join("applications");
        let icons = data.join("icons/hicolor/512x512/apps");
        let small = data.join("icons/hicolor/256x256/apps");
        for dir in [&apps, &icons, &small, &config, &pictures.join(OLD_FOLDER)] {
            fs::create_dir_all(dir).unwrap();
        }
        // The entry the way 2.4 wrote it, old spelling in the name and the id.
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName={OLD_FOLDER}\n\
             Comment=Browse, play, and track your local anime library\nExec=anibeam\n\
             Icon={OLD_ID}\nKeywords=anibeam;anime;media;video;\nStartupWMClass={OLD_ID}\n"
        );
        fs::write(apps.join(format!("{OLD_ID}.desktop")), &entry).unwrap();
        fs::write(apps.join(format!("{OLD_ID}.Dev.desktop")), &entry).unwrap();
        // The installer already wrote the new main entry's Dev twin: the old one only goes.
        let dev = format!("{}.Dev.desktop", crate::APP_ID);
        fs::write(apps.join(&dev), "installer's").unwrap();
        // Someone else's entry that only starts with the same letters stays as it is.
        fs::write(apps.join(format!("{OLD_ID}Fan.desktop")), "x").unwrap();
        fs::write(icons.join(format!("{OLD_ID}.png")), "png").unwrap();
        // A 256 icon with the new name is already there, so the old one stays where it is.
        fs::write(small.join(format!("{OLD_ID}.png")), "old").unwrap();
        fs::write(small.join(format!("{}.png", crate::APP_ID)), "new").unwrap();
        fs::write(
            config.join("kde-mimeapps.list"),
            format!("[Added Associations]\nx-scheme-handler/anibeam={OLD_ID}.desktop;\n"),
        )
        .unwrap();
        fs::write(
            config.join("mimeapps.list"),
            format!("[Default Applications]\nx-scheme-handler/anibeam={OLD_ID}.desktop\n"),
        )
        .unwrap();
        let ledger = home.join("launcher-icons.txt");
        fs::write(
            &ledger,
            format!(
                "{}\n{}\n",
                icons.join(format!("{OLD_ID}.png")).display(),
                small.join(format!("{OLD_ID}.png")).display()
            ),
        )
        .unwrap();

        assert!(linux::adopt_in(
            &data,
            Some(&config),
            &ledger,
            Some(&pictures)
        ));

        let new = |rest: &str| format!("{}{rest}", crate::APP_ID);
        let moved = fs::read_to_string(apps.join(new(".desktop"))).unwrap();
        assert_eq!(
            moved,
            format!(
                "[Desktop Entry]\nType=Application\nName=Anibeam\n\
                 Comment=Browse, play, and track your local anime library\nExec=anibeam\n\
                 Icon={0}\nKeywords=anibeam;anime;media;video;\nStartupWMClass={0}\n",
                crate::APP_ID
            )
        );
        assert!(!moved.contains(OLD_FOLDER));
        assert_eq!(fs::read_to_string(apps.join(&dev)).unwrap(), "installer's");
        assert!(!apps.join(format!("{OLD_ID}.Dev.desktop")).exists());
        assert!(!apps.join(format!("{OLD_ID}.desktop")).exists());
        assert!(apps.join(format!("{OLD_ID}Fan.desktop")).exists());
        assert!(icons.join(new(".png")).is_file());
        assert!(
            !fs::read_to_string(config.join("mimeapps.list"))
                .unwrap()
                .contains(OLD_ID)
        );
        assert!(
            !fs::read_to_string(config.join("kde-mimeapps.list"))
                .unwrap()
                .contains(OLD_ID)
        );
        assert_eq!(
            fs::read_to_string(small.join(format!("{OLD_ID}.png"))).unwrap(),
            "old"
        );
        assert_eq!(
            fs::read_to_string(&ledger).unwrap(),
            format!(
                "{}\n{}\n",
                icons.join(new(".png")).display(),
                small.join(format!("{OLD_ID}.png")).display()
            )
        );
        assert!(pictures.join("Anibeam").is_dir() && !pictures.join(OLD_FOLDER).exists());

        // A second launch finds nothing left to move.
        assert!(!linux::adopt_in(
            &data,
            Some(&config),
            &ledger,
            Some(&pictures)
        ));
        let _ = fs::remove_dir_all(&home);
    }
}
