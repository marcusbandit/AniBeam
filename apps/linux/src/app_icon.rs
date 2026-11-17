//! The app's icon, picked in Appearance. The window takes it through QGuiApplication, which
//! is also the Dock's icon on a Mac and the taskbar's on Windows. On Linux the launcher and
//! the dock read the icon theme by the desktop entry's name, so Dark is laid over the
//! installed icon in the user's own hicolor folder, and Default takes it away again. On a Mac
//! QGuiApplication only changes the Dock while the app runs, so when Anibeam runs from its
//! .app bundle Dark is also set as the bundle's custom icon through NSWorkspace, which is
//! what Finder, Launchpad and the Dock show when it is closed, and Default removes it.

use crate::theme::config::AppIcon;

/// The Default icon's file under assets/: the release mark, the nightly's or the dev
/// blueprint, by what kind of build this is (build.rs decides).
pub const DEFAULT_FILE: &str = env!("ANIBEAM_ICON_FILE");

/// Where the window's PNGs live under assets/. A Mac takes the Mac-grid twins (a 824 tile in
/// the 1024 canvas, assets/mac/), because the full-bleed ones look oversized in the Dock; the
/// Rail and Appearance read the full-bleed files either way.
#[cfg(target_os = "macos")]
const DIR: &str = "mac/";
#[cfg(not(target_os = "macos"))]
const DIR: &str = "";

/// The channel this binary was built as (release, nightly or dev), kept as a delimited string
/// in the binary so the Mac bundle step can read it back and draw the icns for the same
/// channel (packaging/macos/bundle.sh).
#[cfg(target_os = "macos")]
#[used]
static MARK: &str = concat!("\0anibeam-icon-mark=", env!("ANIBEAM_ICON_KIND"), "\0");

/// The 512 PNG the window shows.
pub fn qrc(icon: AppIcon) -> String {
    match icon {
        AppIcon::Default => format!(":/qt/qml/com/marcusrosado/Anibeam/assets/{DIR}{DEFAULT_FILE}"),
        AppIcon::Dark => format!(":/qt/qml/com/marcusrosado/Anibeam/assets/{DIR}icon-dark.png"),
    }
}

/// Lays the picked icon over the installed one, or takes ours away. Runs at launch and on
/// every change, so a hand edit of theme.toml reaches the launcher too. A run under --root is
/// a sandbox and leaves the user's icon folder alone.
pub fn install(icon: AppIcon) {
    #[cfg(target_os = "linux")]
    if crate::runtime::args().root.is_none()
        && let Err(e) = linux::install(icon)
    {
        eprintln!("app icon: {e}");
    }
    #[cfg(target_os = "macos")]
    if crate::runtime::args().root.is_none()
        && let Err(e) = macos::install(icon)
    {
        eprintln!("app icon: {e}");
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let _ = icon;
}

#[cfg(target_os = "linux")]
mod linux {
    use std::path::PathBuf;

    use super::AppIcon;

    const NAME: &str = "com.marcusrosado.Anibeam";

    macro_rules! sizes {
        ($($size:literal),+) => {
            [$(($size, include_bytes!(concat!("../assets/hicolor/dark/", $size, ".png")).as_slice())),+]
        };
    }
    const DARK: [(u32, &[u8]); 8] = sizes!(16, 24, 32, 48, 64, 128, 256, 512);
    const DARK_SVG: &[u8] = include_bytes!("../assets/icon-dark.svg");

    fn hicolor() -> Option<PathBuf> {
        let data = std::env::var_os("XDG_DATA_HOME")
            .filter(|d| !d.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
        Some(data.join("icons/hicolor"))
    }

    fn files() -> Vec<(PathBuf, &'static [u8])> {
        let Some(root) = hicolor() else {
            return Vec::new();
        };
        let mut out: Vec<(PathBuf, &[u8])> = DARK
            .iter()
            .map(|&(size, bytes)| (root.join(format!("{size}x{size}/apps/{NAME}.png")), bytes))
            .collect();
        out.push((root.join(format!("scalable/apps/{NAME}.svg")), DARK_SVG));
        out
    }

    /// The files the last Dark pick wrote, one path a line, in Anibeam's own data folder.
    /// Default removes exactly these, so a dark set from an older build goes too, and a file
    /// the user put in hicolor themselves is never written over nor removed.
    fn ledger() -> PathBuf {
        PathBuf::from(&crate::runtime::paths().core.data_dir).join("launcher-icons.txt")
    }

    pub fn install(icon: AppIcon) -> std::io::Result<()> {
        let ledger = ledger();
        let ours: Vec<PathBuf> = std::fs::read_to_string(&ledger)
            .unwrap_or_default()
            .lines()
            .map(PathBuf::from)
            .collect();
        let mut changed = false;
        match icon {
            AppIcon::Dark => {
                let mut wrote = Vec::new();
                for (path, bytes) in files() {
                    if path.exists() && !ours.contains(&path) {
                        continue;
                    }
                    if std::fs::read(&path).is_ok_and(|b| b == bytes) {
                        wrote.push(path);
                        continue;
                    }
                    if let Some(dir) = path.parent() {
                        std::fs::create_dir_all(dir)?;
                    }
                    std::fs::write(&path, bytes)?;
                    wrote.push(path);
                    changed = true;
                }
                let list: String = wrote.iter().map(|p| format!("{}\n", p.display())).collect();
                if let Some(dir) = ledger.parent() {
                    std::fs::create_dir_all(dir)?;
                }
                if changed {
                    std::fs::write(&ledger, list)?;
                }
            }
            AppIcon::Default => {
                if ours.is_empty() {
                    return Ok(());
                }
                for path in &ours {
                    match std::fs::remove_file(path) {
                        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
                        _ => {}
                    }
                }
                std::fs::remove_file(&ledger)?;
                changed = true;
            }
        }
        // Launchers watch the theme folder's time to know their cache is stale.
        if changed
            && let Some(root) = hicolor()
            && root.exists()
        {
            let now = std::fs::FileTimes::new().set_modified(std::time::SystemTime::now());
            std::fs::File::open(&root)?.set_times(now)?;
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::path::PathBuf;

    use objc2::AnyThread;
    use objc2_app_kit::{NSImage, NSWorkspace, NSWorkspaceIconCreationOptions};
    use objc2_foundation::{NSData, NSString};

    use super::AppIcon;

    const DARK: &[u8] = include_bytes!("../assets/mac/icon-dark-1024.png");

    /// The .app folder the running executable sits in (`X.app/Contents/MacOS/anibeam`), or
    /// None for a cargo build, which has no bundle to give an icon to.
    fn bundle() -> Option<PathBuf> {
        let exe = std::env::current_exe().ok()?.canonicalize().ok()?;
        let app = exe.parent()?.parent()?.parent()?;
        (app.extension()? == "app" && app.is_dir()).then(|| app.to_path_buf())
    }

    pub fn install(icon: AppIcon) -> Result<(), String> {
        let Some(app) = bundle() else {
            return Ok(());
        };
        let path = NSString::from_str(&app.to_string_lossy());
        let workspace = NSWorkspace::sharedWorkspace();
        let ok = match icon {
            AppIcon::Dark => {
                let data = NSData::with_bytes(DARK);
                let image = NSImage::initWithData(NSImage::alloc(), &data)
                    .ok_or("the dark icon did not decode")?;
                workspace.setIcon_forFile_options(
                    Some(&image),
                    &path,
                    NSWorkspaceIconCreationOptions::empty(),
                )
            }
            AppIcon::Default => workspace.setIcon_forFile_options(
                None,
                &path,
                NSWorkspaceIconCreationOptions::empty(),
            ),
        };
        if ok {
            Ok(())
        } else {
            Err(format!("could not set the icon of {}", app.display()))
        }
    }
}
