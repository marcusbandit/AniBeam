//! The app's icon, picked in Appearance. The window takes it through QGuiApplication, which
//! is also the Dock's icon on a Mac and the taskbar's on Windows. On Linux the launcher and
//! the dock read the icon theme by the desktop entry's name, so Dark is laid over the
//! installed icon in the user's own hicolor folder, and Default takes it away again.

use crate::theme::config::AppIcon;

/// The 512 PNG the window and the Rail show.
pub fn qrc(icon: AppIcon) -> &'static str {
    match icon {
        AppIcon::Default => ":/qt/qml/com/marcusrosado/AniBeam/assets/icon.png",
        AppIcon::Dark => ":/qt/qml/com/marcusrosado/AniBeam/assets/icon-dark.png",
    }
}

/// Lays the picked icon over the installed one, or takes ours away. A run under --root is a
/// sandbox and leaves the user's icon folder alone.
pub fn install(icon: AppIcon) {
    #[cfg(target_os = "linux")]
    if crate::runtime::args().root.is_none()
        && let Err(e) = linux::install(icon)
    {
        eprintln!("app icon: {e}");
    }
    #[cfg(not(target_os = "linux"))]
    let _ = icon;
}

#[cfg(target_os = "linux")]
mod linux {
    use std::path::PathBuf;

    use super::AppIcon;

    const NAME: &str = "com.marcusrosado.AniBeam";

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

    pub fn install(icon: AppIcon) -> std::io::Result<()> {
        for (path, bytes) in files() {
            match icon {
                AppIcon::Dark => {
                    if let Some(dir) = path.parent() {
                        std::fs::create_dir_all(dir)?;
                    }
                    std::fs::write(&path, bytes)?;
                }
                // Only a file that is still ours goes; one the user put there stays.
                AppIcon::Default => {
                    if std::fs::read(&path).is_ok_and(|b| b == bytes) {
                        std::fs::remove_file(&path)?;
                    }
                }
            }
        }
        // Launchers watch the theme folder's time to know their cache is stale.
        if let Some(root) = hicolor()
            && root.exists()
        {
            let now = std::fs::FileTimes::new().set_modified(std::time::SystemTime::now());
            std::fs::File::open(&root)?.set_times(now)?;
        }
        Ok(())
    }
}
