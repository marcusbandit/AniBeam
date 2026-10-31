//! How this copy of the desktop app takes an update, and taking it. `Install::detect` reads
//! where the running binary lives, `Install::asset` picks the release file that kind of
//! copy is updated from, and `fetch` downloads it, checks it against GitHub's digest and
//! does whatever can be done while the app still runs. The `Prepared` it hands back starts a
//! small helper as the app exits (`Prepared::finish`): the helper waits for this process to
//! end, finishes the swap and, on a restart, opens the new version.
//!
//! - AppImage: the new file is written beside `$APPIMAGE` and renamed over it. The running
//!   copy keeps its mount, so the swap happens at once and a restart only has to open it.
//! - macOS app: the dmg's .app is copied beside the running one; after exit the helper
//!   swaps the two. A Homebrew cask is the same .app in /Applications.
//! - Windows setup: after exit the helper runs the new setup with no questions.
//! - Windows portable zip: after exit the helper unpacks the new zip over the folder.
//! - A copy under /usr or /opt belongs to the system's package manager, and a build from a
//!   checkout to whoever builds it; neither is updated here.
//!
//! Errors are sentences for the person, since the update notice shows them as they are.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use super::{Asset, Update};

/// How the running copy was installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Install {
    AppImage {
        path: PathBuf,
    },
    MacApp {
        bundle: PathBuf,
    },
    WindowsSetup {
        exe: PathBuf,
    },
    WindowsPortable {
        exe: PathBuf,
    },
    /// Under /usr or /opt. `package` is pacman's name for it, when pacman owns the file.
    Package {
        package: Option<String>,
    },
    Source,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Os {
    Linux,
    Mac,
    Windows,
}

impl Os {
    pub fn current() -> Os {
        if cfg!(target_os = "macos") {
            Os::Mac
        } else if cfg!(windows) {
            Os::Windows
        } else {
            Os::Linux
        }
    }
}

impl Install {
    /// This process's own install.
    pub fn detect() -> Install {
        let Ok(exe) = std::env::current_exe() else {
            return Install::Source;
        };
        let appimage = std::env::var_os("APPIMAGE")
            .map(PathBuf::from)
            .zip(std::env::var_os("APPDIR").map(PathBuf::from));
        match Install::classify(&exe, appimage, Os::current()) {
            Install::Package { .. } => Install::Package {
                package: pacman_owner(&exe),
            },
            other => other,
        }
    }

    /// The install a binary at `exe` is, on `os`. The AppImage runtime names its file in
    /// `$APPIMAGE` and its mount in `$APPDIR`, passed here as the pair; both are inherited
    /// by everything an AppImage starts, so they count only for a binary inside that
    /// mount. On Windows the setup leaves its uninstaller beside the binary and the
    /// portable zip a `qml` folder; a binary with neither is a `cargo` build.
    pub fn classify(exe: &Path, appimage: Option<(PathBuf, PathBuf)>, os: Os) -> Install {
        match os {
            Os::Linux => {
                if let Some((path, _)) =
                    appimage.filter(|(path, appdir)| path.is_absolute() && exe.starts_with(appdir))
                {
                    return Install::AppImage { path };
                }
                if exe.starts_with("/usr") || exe.starts_with("/opt") {
                    return Install::Package { package: None };
                }
                Install::Source
            }
            Os::Mac => {
                // <name>.app/Contents/MacOS/anibeam
                let mut up = exe.ancestors().skip(1);
                let (macos, contents, bundle) = (up.next(), up.next(), up.next());
                let named =
                    |p: Option<&Path>, n: &str| p.and_then(Path::file_name) == Some(n.as_ref());
                match bundle {
                    Some(b)
                        if named(macos, "MacOS")
                            && named(contents, "Contents")
                            && b.extension() == Some("app".as_ref()) =>
                    {
                        Install::MacApp {
                            bundle: b.to_path_buf(),
                        }
                    }
                    _ => Install::Source,
                }
            }
            Os::Windows => {
                let Some(dir) = exe.parent() else {
                    return Install::Source;
                };
                let exe = exe.to_path_buf();
                if dir.join("unins000.exe").is_file() {
                    Install::WindowsSetup { exe }
                } else if dir.join("qml").is_dir() {
                    Install::WindowsPortable { exe }
                } else {
                    Install::Source
                }
            }
        }
    }

    /// Whether `fetch` can update this copy.
    pub fn updates_itself(&self) -> bool {
        self.suffix(std::env::consts::ARCH).is_some()
    }

    /// The end of the release file's name for this install on `arch` (Rust's name for it).
    fn suffix(&self, arch: &str) -> Option<String> {
        Some(match self {
            Install::AppImage { .. } => format!("-{arch}.AppImage"),
            Install::MacApp { .. } => match arch {
                "aarch64" => "-macos-arm64.dmg".to_string(),
                other => format!("-macos-{other}.dmg"),
            },
            Install::WindowsSetup { .. } => format!("-windows-{}-setup.exe", win_arch(arch)),
            Install::WindowsPortable { .. } => format!("-windows-{}.zip", win_arch(arch)),
            Install::Package { .. } | Install::Source => return None,
        })
    }

    /// The file on the release this copy updates from, if the release has one for it.
    pub fn asset<'a>(&self, assets: &'a [Asset]) -> Option<&'a Asset> {
        self.asset_for(assets, std::env::consts::ARCH)
    }

    fn asset_for<'a>(&self, assets: &'a [Asset], arch: &str) -> Option<&'a Asset> {
        let suffix = self.suffix(arch)?;
        assets.iter().find(|a| a.name.ends_with(&suffix))
    }
}

fn win_arch(arch: &str) -> &str {
    match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    }
}

/// pacman's name for the package that owns `exe`, when pacman is there and owns it.
fn pacman_owner(exe: &Path) -> Option<String> {
    let out = std::process::Command::new("pacman")
        .arg("-Qqo")
        .arg(exe)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    let name = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (out.status.success() && !name.is_empty()).then_some(name)
}

/// The command that updates `package` from the AUR, with the first AUR helper on PATH.
pub fn aur_command(package: &str) -> String {
    let helper = ["paru", "yay", "pikaur", "trizen"]
        .into_iter()
        .find(|h| on_path(h));
    match helper {
        Some(h) => format!("{h} -Syu {package}"),
        None => format!(
            "git clone https://aur.archlinux.org/{package}.git && cd {package} && makepkg -si"
        ),
    }
}

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(program).is_file()))
}

/// What `fetch` left ready, finished by `finish` as the app exits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prepared {
    /// Already in place; only a restart has anything left to do.
    AppImage {
        path: PathBuf,
    },
    MacApp {
        bundle: PathBuf,
        staged: PathBuf,
    },
    WindowsSetup {
        setup: PathBuf,
        exe: PathBuf,
    },
    WindowsPortable {
        zip: PathBuf,
        exe: PathBuf,
        work: PathBuf,
    },
}

/// Downloads `update`'s file for `install` and readies it, reporting the bytes so far and
/// the total as it goes. `work` is a directory of the update's own, emptied first. Setting
/// `cancel` stops the download and removes what it wrote.
pub async fn fetch(
    install: &Install,
    update: &Update,
    work: &Path,
    cancel: &AtomicBool,
    progress: impl FnMut(u64, u64) + Send,
) -> Result<Prepared, String> {
    let asset = install.asset(&update.assets).ok_or_else(|| {
        format!(
            "AniBeam {} has no download for this computer",
            update.version
        )
    })?;
    let _ = tokio::fs::remove_dir_all(work).await;
    tokio::fs::create_dir_all(work)
        .await
        .map_err(|e| format!("Could not make {}: {e}", work.display()))?;
    match install {
        Install::AppImage { path } => {
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
            let dir = path.parent().unwrap_or(Path::new("/"));
            let next = dir.join(format!(".{}.update", name.unwrap_or_default()));
            download(asset, &next, cancel, progress).await?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                tokio::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o755))
                    .await
                    .map_err(|e| format!("Could not mark the new AppImage runnable: {e}"))?;
            }
            // The running copy is mounted from the old file's inode, which the rename
            // leaves alone until the app ends.
            tokio::fs::rename(&next, path)
                .await
                .map_err(|e| format!("Could not replace {}: {e}", path.display()))?;
            Ok(Prepared::AppImage { path: path.clone() })
        }
        Install::MacApp { bundle } => {
            let dmg = work.join(&asset.name);
            download(asset, &dmg, cancel, progress).await?;
            let staged = staged_bundle(bundle);
            stage_from_dmg(&dmg, &work.join("mount"), &staged).await?;
            let _ = tokio::fs::remove_file(&dmg).await;
            Ok(Prepared::MacApp {
                bundle: bundle.clone(),
                staged,
            })
        }
        Install::WindowsSetup { exe } => {
            let setup = work.join(&asset.name);
            download(asset, &setup, cancel, progress).await?;
            Ok(Prepared::WindowsSetup {
                setup,
                exe: exe.clone(),
            })
        }
        Install::WindowsPortable { exe } => {
            let zip = work.join(&asset.name);
            download(asset, &zip, cancel, progress).await?;
            Ok(Prepared::WindowsPortable {
                zip,
                exe: exe.clone(),
                work: work.to_path_buf(),
            })
        }
        Install::Package { .. } | Install::Source => Err("This copy is not updated here".into()),
    }
}

/// `asset` into `to`, through `to` with `.part` on the end until it is whole and matches
/// the digest GitHub keeps for it.
async fn download(
    asset: &Asset,
    to: &Path,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64, u64) + Send,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent(format!("AniBeam/{}", crate::VERSION))
        .connect_timeout(Duration::from_secs(20))
        .read_timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Could not start the download: {e}"))?;
    let mut res = client
        .get(&asset.url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| format!("The download did not start: {e}"))?;
    let total = res.content_length().unwrap_or(asset.size);
    let mut part = to.as_os_str().to_owned();
    part.push(".part");
    let part = PathBuf::from(part);
    let mut file = tokio::fs::File::create(&part)
        .await
        .map_err(|e| format!("Could not write {}: {e}", part.display()))?;
    let mut hash = Sha256::new();
    let mut got = 0u64;
    let mut told = u64::MAX;
    let result: Result<(), String> = async {
        while let Some(chunk) = res
            .chunk()
            .await
            .map_err(|e| format!("The download stopped: {e}"))?
        {
            if cancel.load(Ordering::SeqCst) {
                return Err("Cancelled".into());
            }
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Could not write {}: {e}", part.display()))?;
            hash.update(&chunk);
            got += chunk.len() as u64;
            // Once a percent, not once a chunk
            let pct = got * 100 / total.max(1);
            if pct != told {
                told = pct;
                progress(got, total);
            }
        }
        file.flush()
            .await
            .map_err(|e| format!("Could not write {}: {e}", part.display()))?;
        if got != asset.size {
            return Err(format!(
                "The download ended early ({got} of {} bytes)",
                asset.size
            ));
        }
        if let Some(want) = asset
            .digest
            .as_deref()
            .and_then(|d| d.strip_prefix("sha256:"))
        {
            let have = hex(&hash.finalize());
            if !have.eq_ignore_ascii_case(want) {
                return Err("The download does not match GitHub's checksum".into());
            }
        }
        Ok(())
    }
    .await;
    drop(file);
    if let Err(e) = result {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(e);
    }
    tokio::fs::rename(&part, to)
        .await
        .map_err(|e| format!("Could not move the download into place: {e}"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Where the new .app waits beside the running one: hidden, on the same volume, so the swap
/// after exit is two renames.
fn staged_bundle(bundle: &Path) -> PathBuf {
    let name = bundle.file_name().map(|n| n.to_string_lossy().into_owned());
    bundle.with_file_name(format!(".{}.update", name.unwrap_or_default()))
}

/// Mounts `dmg` at `mount`, copies the .app inside it to `staged` and unmounts it.
async fn stage_from_dmg(dmg: &Path, mount: &Path, staged: &Path) -> Result<(), String> {
    use tokio::process::Command;
    run(Command::new("hdiutil")
        .args([
            "attach",
            "-nobrowse",
            "-readonly",
            "-noautoopen",
            "-mountpoint",
        ])
        .arg(mount)
        .arg(dmg))
    .await
    .map_err(|e| format!("Could not open the dmg: {e}"))?;
    let copied = async {
        let mut entries = tokio::fs::read_dir(mount)
            .await
            .map_err(|e| format!("Could not read the dmg: {e}"))?;
        let mut app = None;
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.path().extension() == Some("app".as_ref()) {
                app = Some(entry.path());
                break;
            }
        }
        let app = app.ok_or("The dmg holds no app")?;
        let _ = tokio::fs::remove_dir_all(staged).await;
        run(Command::new("ditto").arg(&app).arg(staged))
            .await
            .map_err(|e| format!("Could not copy the new app beside this one: {e}"))
    }
    .await;
    let _ = run(Command::new("hdiutil")
        .arg("detach")
        .arg(mount)
        .arg("-force"))
    .await;
    copied
}

/// Runs a command to its end; its stderr is the error when it fails.
async fn run(cmd: &mut tokio::process::Command) -> Result<(), String> {
    let out = cmd
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(err.trim().lines().last().unwrap_or("failed").to_string())
    }
}

/// A program and its arguments, the helper `finish` starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Helper {
    pub program: PathBuf,
    pub args: Vec<OsString>,
}

/// Waits for the process the first argument names to end.
const WAIT_SH: &str = r#"while kill -0 "$1" 2>/dev/null; do sleep 0.2; done"#;

impl Prepared {
    /// Starts the helper that finishes the update once this process has ended, and opens
    /// the new version with this run's arguments when `reopen`. Called as the app exits.
    pub fn finish(&self, reopen: bool) -> std::io::Result<()> {
        let args: Vec<OsString> = std::env::args_os().skip(1).collect();
        let Some(helper) = self.helper(std::process::id(), reopen, &args) else {
            return Ok(());
        };
        let mut cmd = std::process::Command::new(&helper.program);
        cmd.args(&helper.args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        if let Prepared::AppImage { .. } = self {
            // The AppImage runtime pointed these at the old copy's mount, which goes when
            // this process does; the new copy's runtime sets them again.
            let appdir = std::env::var("APPDIR").ok().filter(|d| !d.is_empty());
            for (name, value) in std::env::vars_os() {
                let mounted = appdir
                    .as_deref()
                    .is_some_and(|d| value.to_string_lossy().contains(d));
                let runtime = ["APPIMAGE", "APPDIR", "ARGV0", "OWD"]
                    .iter()
                    .any(|n| name == *n);
                if mounted || runtime {
                    cmd.env_remove(name);
                }
            }
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
        }
        cmd.spawn().map(drop)
    }

    /// The helper for `finish`, None when nothing is left to do. `args` go to the new
    /// version when it opens.
    pub fn helper(&self, pid: u32, reopen: bool, args: &[OsString]) -> Option<Helper> {
        let pid = pid.to_string();
        match self {
            Prepared::AppImage { path } => reopen.then(|| Helper {
                program: "/bin/sh".into(),
                args: [
                    "-c".into(),
                    format!(r#"{WAIT_SH}; shift; exec "$@""#).into(),
                    "sh".into(),
                    pid.into(),
                    path.into(),
                ]
                .into_iter()
                .chain(args.iter().cloned())
                .collect(),
            }),
            Prepared::MacApp { bundle, staged } => {
                // The old .app is moved aside rather than removed first, so a swap that
                // fails halfway puts it back, and a restart opens whichever is in place.
                let script = format!(
                    r#"{WAIT_SH}
old="$3.old"; rm -rf "$old"
if mv "$2" "$old"; then
  if mv "$3" "$2"; then rm -rf "$old"; else mv "$old" "$2"; fi
fi
if [ "$4" = reopen ]; then app="$2"; shift 4; exec open "$app" --args "$@"; fi"#
                );
                Some(Helper {
                    program: "/bin/sh".into(),
                    args: [
                        "-c".into(),
                        script.into(),
                        "sh".into(),
                        pid.into(),
                        bundle.into(),
                        staged.into(),
                        if reopen { "reopen" } else { "stay" }.into(),
                    ]
                    .into_iter()
                    .chain(args.iter().cloned())
                    .collect(),
                })
            }
            Prepared::WindowsSetup { setup, exe } => {
                let mut script = format!(
                    "Wait-Process -Id {pid} -ErrorAction SilentlyContinue\n\
                     Start-Process -FilePath {} -ArgumentList '/SILENT','/SUPPRESSMSGBOXES','/NORESTART','/CLOSEAPPLICATIONS' -Wait\n\
                     Remove-Item -LiteralPath {} -Force -ErrorAction SilentlyContinue",
                    ps(setup),
                    ps(setup),
                );
                if reopen {
                    script += &start(exe, args);
                }
                Some(powershell(script))
            }
            Prepared::WindowsPortable { zip, exe, work } => {
                let dir = exe.parent().unwrap_or(Path::new("."));
                let unpacked = work.join("unpacked");
                // A failed unpack stops before the folder is touched. The folders the
                // setup clears are moved aside rather than removed, and a copy that fails
                // puts them back. The new version, or the old one, opens either way.
                let mut script = format!(
                    "$ErrorActionPreference = 'Stop'\n\
                     Wait-Process -Id {pid} -ErrorAction SilentlyContinue\n\
                     try {{\n\
                     Get-Process qbittorrent-nox -ErrorAction SilentlyContinue | Where-Object {{ $_.Path -like ({dir} + '\\*') }} | Stop-Process -Force\n\
                     Remove-Item -LiteralPath {unpacked} -Recurse -Force -ErrorAction SilentlyContinue\n\
                     Expand-Archive -LiteralPath {zip} -DestinationPath {unpacked} -Force\n\
                     $old = Join-Path {work} 'old'\n\
                     New-Item -ItemType Directory -Path $old -Force | Out-Null\n\
                     foreach ($d in 'qml','anirss') {{ $p = Join-Path {dir} $d; if (Test-Path -LiteralPath $p) {{ Move-Item -LiteralPath $p -Destination $old }} }}\n\
                     try {{\n\
                     Get-ChildItem -LiteralPath (Join-Path {unpacked} 'AniBeam') | Copy-Item -Destination {dir} -Recurse -Force\n\
                     }} catch {{\n\
                     foreach ($d in 'qml','anirss') {{ $o = Join-Path $old $d; if (Test-Path -LiteralPath $o) {{ Remove-Item -LiteralPath (Join-Path {dir} $d) -Recurse -Force -ErrorAction SilentlyContinue; Move-Item -LiteralPath $o -Destination {dir} }} }}\n\
                     throw\n\
                     }}\n\
                     Remove-Item -LiteralPath {work} -Recurse -Force -ErrorAction SilentlyContinue\n\
                     }} finally {{",
                    dir = ps(dir),
                    unpacked = ps(&unpacked),
                    zip = ps(zip),
                    work = ps(work),
                );
                if reopen {
                    script += &start(exe, args);
                }
                script += "\n}";
                Some(powershell(script))
            }
        }
    }
}

/// A PowerShell single-quoted string: nothing inside is expanded, and a quote is doubled.
fn ps(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "''"))
}

/// The line that opens `exe` with `args`.
fn start(exe: &Path, args: &[OsString]) -> String {
    let mut line = format!("\nStart-Process -FilePath {}", ps(exe));
    if !args.is_empty() {
        // Start-Process joins the list with spaces, so each argument carries its own quotes
        let list: Vec<String> = args
            .iter()
            .map(|a| ps(Path::new(&format!("\"{}\"", a.to_string_lossy()))))
            .collect();
        line += &format!(" -ArgumentList {}", list.join(","));
    }
    line
}

fn powershell(script: String) -> Helper {
    Helper {
        program: "powershell.exe".into(),
        args: [
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
        ]
        .into_iter()
        .map(OsString::from)
        .chain([OsString::from(script)])
        .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str) -> Asset {
        Asset {
            name: name.into(),
            url: format!("https://dl/{name}"),
            size: 1,
            digest: None,
        }
    }

    fn release() -> Vec<Asset> {
        [
            "AniBeam-2.5.0-macos-arm64.dmg",
            "AniBeam-2.5.0-windows-x64-setup.exe",
            "AniBeam-2.5.0-windows-x64.zip",
            "AniBeam-2.5.0-x86_64.AppImage",
        ]
        .into_iter()
        .map(asset)
        .collect()
    }

    fn picked(install: Install, arch: &str) -> Option<String> {
        install.asset_for(&release(), arch).map(|a| a.name.clone())
    }

    #[test]
    fn each_install_updates_from_its_own_file() {
        let p = PathBuf::from("/x");
        assert_eq!(
            picked(Install::AppImage { path: p.clone() }, "x86_64").as_deref(),
            Some("AniBeam-2.5.0-x86_64.AppImage")
        );
        assert_eq!(
            picked(Install::AppImage { path: p.clone() }, "aarch64"),
            None
        );
        assert_eq!(
            picked(Install::MacApp { bundle: p.clone() }, "aarch64").as_deref(),
            Some("AniBeam-2.5.0-macos-arm64.dmg")
        );
        assert_eq!(
            picked(Install::WindowsSetup { exe: p.clone() }, "x86_64").as_deref(),
            Some("AniBeam-2.5.0-windows-x64-setup.exe")
        );
        assert_eq!(
            picked(Install::WindowsPortable { exe: p }, "x86_64").as_deref(),
            Some("AniBeam-2.5.0-windows-x64.zip")
        );
        assert_eq!(picked(Install::Source, "x86_64"), None);
        assert_eq!(picked(Install::Package { package: None }, "x86_64"), None);
    }

    /// Unix paths: Windows reads `/home/...` as relative.
    #[cfg(unix)]
    #[test]
    fn linux_tells_an_appimage_a_package_and_a_build_apart() {
        let exe = Path::new("/tmp/.mount_AniBeaXYZ/usr/bin/anibeam");
        let runtime = || {
            Some((
                PathBuf::from("/home/a/AniBeam.AppImage"),
                PathBuf::from("/tmp/.mount_AniBeaXYZ"),
            ))
        };
        assert_eq!(
            Install::classify(exe, runtime(), Os::Linux),
            Install::AppImage {
                path: "/home/a/AniBeam.AppImage".into()
            }
        );
        // Started from a terminal inside some AppImage: its variables are not ours
        assert_eq!(
            Install::classify(Path::new("/usr/bin/anibeam"), runtime(), Os::Linux),
            Install::Package { package: None }
        );
        assert_eq!(
            Install::classify(Path::new("/usr/bin/anibeam"), None, Os::Linux),
            Install::Package { package: None }
        );
        assert_eq!(
            Install::classify(
                Path::new("/home/a/AniBeam/target/release/anibeam"),
                None,
                Os::Linux
            ),
            Install::Source
        );
    }

    #[test]
    fn a_mac_app_is_the_bundle_around_the_binary() {
        assert_eq!(
            Install::classify(
                Path::new("/Applications/AniBeam.app/Contents/MacOS/anibeam"),
                None,
                Os::Mac
            ),
            Install::MacApp {
                bundle: "/Applications/AniBeam.app".into()
            }
        );
        assert_eq!(
            Install::classify(
                Path::new("/Users/a/AniBeam/target/release/anibeam"),
                None,
                Os::Mac
            ),
            Install::Source
        );
    }

    #[test]
    fn windows_reads_the_folder_the_binary_sits_in() {
        let setup = tempfile::tempdir().unwrap();
        std::fs::write(setup.path().join("unins000.exe"), b"").unwrap();
        let exe = setup.path().join("anibeam.exe");
        assert_eq!(
            Install::classify(&exe, None, Os::Windows),
            Install::WindowsSetup { exe }
        );
        let portable = tempfile::tempdir().unwrap();
        std::fs::create_dir(portable.path().join("qml")).unwrap();
        let exe = portable.path().join("anibeam.exe");
        assert_eq!(
            Install::classify(&exe, None, Os::Windows),
            Install::WindowsPortable { exe }
        );
        let cargo = tempfile::tempdir().unwrap();
        assert_eq!(
            Install::classify(&cargo.path().join("anibeam.exe"), None, Os::Windows),
            Install::Source
        );
    }

    #[test]
    fn an_appimage_restart_waits_then_opens_the_same_path() {
        let p = Prepared::AppImage {
            path: "/home/a/AniBeam.AppImage".into(),
        };
        assert_eq!(p.helper(42, false, &[]), None);
        let h = p
            .helper(42, true, &["--root".into(), "/tmp/s".into()])
            .unwrap();
        assert_eq!(h.program, PathBuf::from("/bin/sh"));
        assert_eq!(h.args[3], "42");
        assert_eq!(h.args[4], "/home/a/AniBeam.AppImage");
        assert_eq!(h.args[5..], ["--root", "/tmp/s"]);
    }

    #[test]
    fn a_mac_swap_runs_on_quit_and_reopens_only_on_restart() {
        let p = Prepared::MacApp {
            bundle: "/Applications/AniBeam.app".into(),
            staged: "/Applications/.AniBeam.app.update".into(),
        };
        let quit = p.helper(7, false, &[]).unwrap();
        assert_eq!(quit.args.last().unwrap(), "stay");
        let restart = p.helper(7, true, &[]).unwrap();
        assert_eq!(restart.args.last().unwrap(), "reopen");
        assert_eq!(restart.args[5], "/Applications/.AniBeam.app.update");
    }

    #[test]
    fn a_windows_path_is_quoted_for_powershell() {
        assert_eq!(
            ps(Path::new(r"C:\Users\O'Neil\AniBeam")),
            r"'C:\Users\O''Neil\AniBeam'"
        );
        let p = Prepared::WindowsSetup {
            setup: r"C:\t\setup.exe".into(),
            exe: r"C:\a\anibeam.exe".into(),
        };
        let script = |reopen, args: &[OsString]| {
            p.helper(9, reopen, args)
                .unwrap()
                .args
                .last()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        };
        let restart = script(true, &[]);
        assert!(restart.starts_with("Wait-Process -Id 9 "));
        assert!(restart.contains("'/SILENT'"));
        assert!(restart.ends_with(r"Start-Process -FilePath 'C:\a\anibeam.exe'"));
        let rooted = script(true, &["--root".into(), r"C:\my root".into()]);
        assert!(rooted.ends_with(r#"-ArgumentList '"--root"','"C:\my root"'"#));
        let quiet = script(false, &[]);
        assert!(!quiet.contains("anibeam.exe"));
    }

    /// The sh scripts are real shell: run the AppImage one against a process that ends.
    #[cfg(unix)]
    #[test]
    fn the_sh_helper_waits_for_the_process_to_end() {
        let dir = tempfile::tempdir().unwrap();
        let mark = dir.path().join("opened");
        let opener = dir.path().join("new.AppImage");
        std::fs::write(&opener, format!("#!/bin/sh\ntouch '{}'\n", mark.display())).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&opener, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut sleeper = std::process::Command::new("sleep")
            .arg("0.5")
            .spawn()
            .unwrap();
        let h = Prepared::AppImage { path: opener }
            .helper(sleeper.id(), true, &[])
            .unwrap();
        let mut helper = std::process::Command::new(&h.program)
            .args(&h.args)
            .spawn()
            .unwrap();
        std::thread::sleep(Duration::from_millis(200));
        assert!(!mark.exists(), "opened before the old process ended");
        sleeper.wait().unwrap();
        helper.wait().unwrap();
        assert!(mark.exists());
    }

    /// The Mac swap, run on any unix with a stand-in for `open`.
    #[cfg(unix)]
    #[test]
    fn the_mac_swap_puts_the_new_bundle_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("AniBeam.app");
        let staged = dir.path().join(".AniBeam.app.update");
        std::fs::create_dir_all(bundle.join("old")).unwrap();
        std::fs::create_dir_all(staged.join("new")).unwrap();
        let h = Prepared::MacApp {
            bundle: bundle.clone(),
            staged: staged.clone(),
        }
        .helper(u32::MAX >> 2, false, &[])
        .unwrap();
        let status = std::process::Command::new(&h.program)
            .args(&h.args)
            .status()
            .unwrap();
        assert!(status.success());
        assert!(bundle.join("new").is_dir());
        // A swap that cannot happen leaves the app that was there
        let gone = Prepared::MacApp {
            bundle: bundle.clone(),
            staged: dir.path().join(".missing.update"),
        }
        .helper(u32::MAX >> 2, false, &[])
        .unwrap();
        std::process::Command::new(&gone.program)
            .args(&gone.args)
            .status()
            .unwrap();
        assert!(bundle.join("new").is_dir());
        assert!(!staged.exists());
        assert!(!dir.path().join(".AniBeam.app.update.old").exists());
    }
}
