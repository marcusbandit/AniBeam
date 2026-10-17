# AniBeam

<img src="assets/icon.png" alt="AniBeam icon" width="96" align="right" />

AniBeam browses, plays and tracks a local anime library. It scans the folders you point it
at, matches each series against AniList (TVmaze for what AniList does not carry), plays
video in the window through libmpv, and syncs watch progress with AniList and MyAnimeList.
It runs on Linux and on Apple silicon Macs: a Qt 6 shell over a Rust core.

## Getting started

One install is all AniBeam needs, on every platform. The video player, anirss (which
searches release feeds) and a qBittorrent for downloads all come with it.

### 1. Install

**Mac** (Apple silicon, macOS 15 or later)

1. Download `AniBeam-<version>-macos-arm64.dmg` from [Releases](https://github.com/marcusbandit/AniBeam/releases).
2. Open it and drag AniBeam into Applications.
3. Open AniBeam once. macOS stops it, because the build is not notarized: go to
   System Settings > Privacy & Security, scroll to the AniBeam line and click Open Anyway.
   From a terminal, `xattr -dr com.apple.quarantine /Applications/AniBeam.app` does the same.

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

The first time AniBeam opens, Home says "Your library is empty". Click Add a folder (or go to
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
  `~/.config/mpv/mpv.conf` on both.

## Build from source

You need Rust 1.95 or newer, Qt 6 (base, declarative, svg), libmpv and KDE's MpvQt.
On Arch that is `pacman -S rust qt6-base qt6-declarative qt6-svg mpv mpvqt`.

```bash
git clone https://github.com/marcusbandit/AniBeam.git
cd AniBeam
cargo build --release -p anibeam     # target/release/anibeam
```

A build from source finds anirss and qbittorrent-nox on your PATH (anirss 0.4.2 or newer,
`yay -S anirss qbittorrent-nox` on Arch). A build from source has no AniList or MyAnimeList
app registration of its own, so the
Trackers tab asks for a client ID the first time you connect. To bundle your own, copy
`.env.example` to `.env.local` and fill it in before building; it explains where to
register.

On a Mac, `apps/linux/packaging/macos/deps.sh` builds libmpv and MpvQt, and `bundle.sh`
turns the binary into `AniBeam.app` and a dmg. `.github/workflows/macos.yml` runs both.

## Development

The Rust core lives in `core/`, the Qt shell in `apps/linux/` (it builds for the Mac
too), and a terminal shell over the same core in `apps/cli/`.

```bash
cargo build --release -p anibeam-cli   # target/release/anibeam-cli
cargo test --workspace                 # unit tests plus core/tests/
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

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

### The phase 1 exit check

The core is finished for phase 1 when it lists the same library the Electron
app does. Both sides are read from a terminal:

```bash
bun scripts/electron-export.mjs ~/anibeam-export.json   # what Settings > Export writes
bun scripts/electron-list.mjs > /tmp/electron-list.txt  # the Home grid, All tab
```

```bash
CLI="target/release/anibeam-cli --root /tmp/anibeam-exit-check"
$CLI import ~/anibeam-export.json --wait
$CLI scan --wait
$CLI call RefreshAll --wait
$CLI list --tab all --sort alpha --direction asc | awk -F'\t' '{ print $3 }' > /tmp/native-list.txt
diff /tmp/electron-list.txt /tmp/native-list.txt && echo "phase 1 exit: identical"
```

The import carries the matches as provider ids, so the AniList records behind
them still have to be fetched: in the app that is the backfill the launch
queues behind its catch-up scan, and from the terminal it is the `RefreshAll`
above. A series whose fetch failed keeps its folder name as its title until
the next run, so run it again if AniList was having a bad day.

Both lists are the same titles in the same order when the two sides agree.
Titles can drift apart without either side being wrong: AniList's romaji is
edited from time to time, so an Electron entry that has not been refreshed in
months holds an older string for the same id.

## The Electron app

The original Electron app still lives in `src/`, frozen. Its last release is the zip on
v2.0.0. Bun builds it: `bun install`, then `bun run package`.

## Releasing

Push a version tag. `.github/workflows/release.yml` builds the Mac dmg and the Linux
AppImage, runs both smoke tests, and leaves a draft release with the files and install
notes on it. Read the draft, then press Publish.

```bash
git tag v2.1.0
git push origin v2.1.0
```

Once the release is published, point the AUR at it:

```bash
apps/linux/packaging/aur/publish.sh 2.1.0
```

Bump `version` in the workspace `Cargo.toml` before tagging: a build from the AUR's
tarball has no git history and reports the Cargo version.
