# AniBeam

<img src="assets/icon.png" alt="AniBeam icon" width="96" align="right" />

AniBeam browses, plays and tracks a local anime library. It scans the folders you point it
at, matches each series against AniList (TVmaze for what AniList does not carry), plays
video in the window through libmpv, and syncs watch progress with AniList and MyAnimeList.
It runs on Linux, Windows and Apple silicon Macs: a Qt 6 shell over a Rust core. The Electron app
it replaced was retired in October 2026; the `v2.0.0` tag holds its source.

## Getting started

One install is all AniBeam needs, on every platform. The video player, anirss (which
searches release feeds) and a qBittorrent for downloads all come with it.

### 1. Install

**Mac** (Apple silicon, macOS 15 or later)

With Homebrew:

```bash
brew tap marcusbandit/anibeam https://github.com/marcusbandit/AniBeam
brew install --cask marcusbandit/anibeam/anibeam
```

The full name in the install trusts the tap's cask, which Homebrew asks for before it loads
one from outside its own taps. `brew upgrade` picks up each new release.

Or by hand:

1. Download `AniBeam-<version>-macos-arm64.dmg` from [Releases](https://github.com/marcusbandit/AniBeam/releases).
2. Open it and drag AniBeam into Applications.

Either way, open AniBeam once. macOS stops it, because the build is not notarized: go to
System Settings > Privacy & Security, scroll to the AniBeam line and click Open Anyway.
From a terminal, `xattr -dr com.apple.quarantine /Applications/AniBeam.app` does the same.

**Windows** (64-bit, Windows 10 or later)

1. Download `AniBeam-<version>-windows-x64-setup.exe` from [Releases](https://github.com/marcusbandit/AniBeam/releases).
2. Run it. It installs for you alone, with no administrator prompt, and adds AniBeam to the
   Start menu.

The installer is not signed, so SmartScreen may say "Windows protected your PC": click More
info, then Run anyway. `AniBeam-<version>-windows-x64.zip` is the same app without an
installer; unzip it anywhere and run `anibeam.exe`.

**Arch Linux**

```bash
yay -S anibeam        # or: paru -S anibeam
```

The AUR package builds the latest release and pulls in anirss and qbittorrent-nox.

**Any other Linux** (glibc 2.39 or newer: Ubuntu 24.04, Debian 13, Fedora 40 and later)

Download `AniBeam-<version>-x86_64.AppImage` from
[Releases](https://github.com/marcusbandit/AniBeam/releases), then:

```bash
chmod +x AniBeam-*-x86_64.AppImage
./AniBeam-*-x86_64.AppImage
```

### 2. Add your library

The first time AniBeam opens, Home says "No series yet". Click Add a folder (or go to
Settings > Library > Folders > Add folder) and pick the folder your anime lives in. One
folder per show works best, for example `Anime/Sousou no Frieren/Frieren - 01.mkv`.

AniBeam scans it, matches each show against AniList (and TVmaze for anything that is not
anime), and fetches posters, synopses and airing dates. Files that land in the folder later
show up by themselves. A show it could not match says "Needs a match": open it, click Find
a match, and pick the right one.

### 3. Connect AniList or MyAnimeList (optional)

Settings > Library > Trackers > Connect. Your browser opens; sign in, approve AniBeam, and
come back. From then on, watching most of an episode marks it on your list, and the cards
show your progress. Main tracker picks whose progress wins when you connect both.

### 4. Watch

Click a show, then an episode. Space plays and pauses, and `?` shows every key. AniBeam
remembers where you stopped and offers the next episode when one ends.

### 5. Download new episodes (optional)

Open Downloads (the arrow in the left rail). There is nothing to install or set up first:

- AniBeam starts its own anirss, the tool that searches the release feeds.
- If a qBittorrent of yours is already running with its Web UI on, AniBeam uses it.
  If not, AniBeam sets up its own qBittorrent the first time you open the page, and
  says so in the status line at the bottom.

Search for a show (`Frieren 1080p`), look over the releases, and click Subscribe. New
episodes then download by themselves as they come out, into `~/Downloads/Anime` unless you
pick another folder. Add that folder to your library (step 2) so they show up there. Download
grabs a single release instead.

AniBeam's own qBittorrent keeps running after you close AniBeam, so subscriptions keep
downloading. Settings > Downloads > Torrent client shows which qBittorrent is in use. To
switch to your own, turn on its Web UI (in qBittorrent: Preferences > Web UI, tick the box,
set a username and password), choose Mine, fill in the address, username and password, and
Save.

### When something is off

- **macOS says AniBeam "cannot be opened"**: that is the Open Anyway step in 1.
- **Windows says it protected your PC**: that is SmartScreen on an unsigned installer. Click
  More info, then Run anyway.
- **The AppImage does not start and mentions FUSE**: install `libfuse2` (`libfuse2t64` on
  Ubuntu 24.04), or run it with `--appimage-extract-and-run`.
- **Downloads says qBittorrent did not answer**: open Settings > Downloads > Torrent client
  and press Test; the State line says what went wrong. Choosing "Pick for me" and saving
  makes AniBeam look again on the next connect.
- **Sign-in stays on "Finish in your browser"**: approve AniBeam in the browser tab it
  opened. If that tab is gone, click Open the sign-in page again.
- **Where things are kept**: on a Mac in `~/Library/Application Support/anibeam`, tracker
  tokens in the login keychain. On Linux in `~/.local/share/anibeam` and
  `~/.config/anibeam`, tracker tokens in the Secret Service keyring (GNOME Keyring, KWallet)
  or, without one, in a file only you can read. "Use my mpv.conf" reads
  `~/.config/mpv/mpv.conf` on both. On Windows in `%LOCALAPPDATA%\anibeam` and
  `%APPDATA%\anibeam`, tracker tokens in the Credential Manager, and "Use my mpv.conf"
  reads `%APPDATA%\mpv\mpv.conf`.

## Share over Tailscale

AniBeam can share a read-only copy of your library with the AniBeam app on your phone.
Turn on Settings > Library > Phone app > Share over Tailscale. It needs
[Tailscale](https://tailscale.com) running on this computer and on the phone, signed in to
the same tailnet. The phone app lives in `apps/android`; its README covers building it,
sideloading it and pointing it at this computer.

When the switch is on, AniBeam listens on port 8790, on this computer's Tailscale addresses
only. Nothing on your home network or the internet can reach it, so the tailnet is the
lock. The setting shows the addresses it answers on, and the MagicDNS name when the
`tailscale` command can say it. If Tailscale is not connected yet, AniBeam says so and looks
again every minute.

The phone reads three things and changes none of them: your library with its watched
progress and airing times, your anirss subscriptions, and the torrents qBittorrent holds. If
anirss or qBittorrent is not running, the phone gets the rest and a short note about which
half is missing.

You can see what it serves with `curl http://<address>:8790/api/v1/hello` and
`/api/v1/snapshot`. For development, `ANIBEAM_HOST_BIND=127.0.0.1:8790` picks the address by
hand. `0.0.0.0:8790` lets an Android emulator reach it at `10.0.2.2`. The file
`host-api/src/lib.rs` describes the JSON.

## Build from source

You need Rust 1.95 or newer, Qt 6.11 or newer (`qmake6` on PATH; base, declarative, svg),
libmpv, KDE's MpvQt and the lld linker. On Arch that is
`pacman -S rust lld qt6-base qt6-declarative qt6-svg mpv mpvqt`.

```bash
git clone https://github.com/marcusbandit/AniBeam.git
cd AniBeam
cargo build --release -p anibeam     # target/release/anibeam
```

An installed build reads its themes and its bundled `mpv.conf` from `/usr/share/anibeam`. A
run straight from the checkout points at the checkout's copies instead, and `--root` keeps
all of its data in one directory so it never touches a real install's:

```bash
ANIBEAM_THEMES_DIR=apps/linux/themes ANIBEAM_MPV_CONF=apps/linux/mpv.conf \
  target/release/anibeam --root /tmp/anibeam-dev
```

A build from source finds anirss and qbittorrent-nox on your PATH (anirss 0.5.0 or newer,
`yay -S anirss qbittorrent-nox` on Arch).

A build from source has no AniList or MyAnimeList app registration of its own, so the
Trackers tab asks for a client ID the first time you connect. To bundle your own, copy
`.env.example` to `.env.local` and fill it in before building; it explains where to
register.

On a Mac, `apps/linux/packaging/macos/deps.sh` builds libmpv and MpvQt, and `bundle.sh`
turns the binary into `AniBeam.app` and a dmg. `.github/workflows/macos.yml` runs both.
On Windows, `apps/linux/packaging/windows/deps.sh` and `bundle.sh` do the same for the
installer and the zip, and `.github/workflows/windows.yml` runs them.

## Development

The Rust core lives in `core/`, the Qt shell in `apps/linux/` (it builds for the Mac
too), and a terminal shell over the
same core in `apps/cli/`. Cargo builds all three.

```bash
cargo build --release -p anibeam-cli   # target/release/anibeam-cli
cargo test --workspace                 # unit tests plus core/tests/
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

`CONTRIBUTING.md` covers issues, branch and commit names and the PR flow.
`apps/linux/README.md` covers the shell's own tooling: sandboxed runs, offscreen captures
with `scripts/shoot.sh` and scripted drives with `scripts/drive.sh`. `CONTEXT.md` is the
vocabulary the code and the docs share, and the design behind the native line is
`docs/superpowers/specs/2026-09-04-native-line-design.md`.

The CLI is the core's door: `sources`, `scan`, `list`, `show`, `jobs`,
`import`, `export`, `events --follow`, and `call <Name> --json '{...}'` for
anything else. `--root <dir>` puts the database, the config, the cache and
the secrets under one directory instead of the XDG ones, which is how you
drive it against a real library without touching the app's own data.

```bash
anibeam-cli call AddSource --json '{"path": "/mnt/media/anime"}'
anibeam-cli scan --wait
anibeam-cli list --tab all --sort alpha --direction asc
```

## Releasing

Push a version tag. `.github/workflows/release.yml` builds the Mac dmg, the Linux
AppImage and the Windows installer and zip, runs the three smoke tests, and leaves a draft release with the files and install
notes on it. Read the draft, then press Publish.

```bash
git tag v2.1.0
git push origin v2.1.0
```

Once the release is published, point the AUR at it:

```bash
apps/linux/packaging/aur/publish.sh 2.1.0
```

and point the Homebrew cask at its dmg, then open a PR with the change to
`Casks/anibeam.rb`:

```bash
apps/linux/packaging/brew/bump.sh 2.1.0
```

Bump `version` in the workspace `Cargo.toml` before tagging: a build from the AUR's
tarball has no git history and reports the Cargo version.
