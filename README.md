# AniBeam

<img src="assets/icon.png" alt="AniBeam icon" width="96" align="right" />

AniBeam browses, plays and tracks a local anime library. It scans the folders you point it
at, matches each series against AniList (TVmaze for what AniList does not carry), plays
video in the window through libmpv, and syncs watch progress with AniList and MyAnimeList.

Since 2.0 it is a native app: a Qt 6 QML shell over a Rust core. Linux is the supported
platform. The Electron app it replaced is still in the tree, frozen, until it is retired.

## Install

[v2.0.0](https://github.com/marcusbandit/AniBeam/releases/tag/v2.0.0) carries `anibeam-2.0.0`, a
Linux x86_64 binary built on Arch against Qt 6, MpvQt 1.2 and libmpv. Those have to be
installed beside it; on Arch:

```bash
sudo pacman -S qt6-base qt6-declarative qt6-svg mpv mpvqt
```

Rename it to `anibeam`, put it on your PATH and run it. For a launcher entry, copy
`apps/linux/com.marcusrosado.AniBeam.desktop` into `~/.local/share/applications/`.

The `AniBeam-linux-x64-*.zip` on the same release is the legacy Electron app.

There is no package yet. An AUR package, an AppImage and a Mac dmg are in progress on the
`feat/release-pipeline` branch.

## Getting started

### Add your library

The first time AniBeam opens, Home says "No series yet". Click Add a folder (or go to
Settings > Library > Folders > Add folder) and pick the folder your anime lives in. One
folder per show works best, for example `Anime/Sousou no Frieren/Frieren - 01.mkv`.

AniBeam scans it, matches each show against AniList (and TVmaze for anything that is not
anime), and fetches posters, synopses and airing dates. Files that land in the folder later
show up by themselves. A show it could not match says "Needs a match": open it, click Find
a match, and pick the right one.

### Connect AniList or MyAnimeList (optional)

Settings > Library > Trackers > Connect. Your browser opens; sign in, approve AniBeam, and
come back. From then on, watching most of an episode marks it on your list, and the cards
show your progress. Main tracker picks whose progress wins when you connect both.

### Watch

Click a show, then an episode. Space plays and pauses, and `?` shows every key. AniBeam
remembers where you stopped and offers the next episode when one ends.

### Download new episodes (optional)

The Downloads page fronts [anirss](https://github.com/marcusbandit/anirss), which searches
the release feeds, and a qBittorrent with its Web UI turned on. Neither ships with AniBeam
yet: install both (`yay -S anirss qbittorrent-nox` on Arch), then point Settings >
Downloads > Torrent client at your qBittorrent. Search for a show, look over the releases,
and click Subscribe; new episodes then download by themselves. Add the download folder to
your library so they show up there.

### Where things are kept

`~/.local/share/anibeam`, `~/.config/anibeam` and `~/.cache/anibeam`. Tracker tokens go to
the Secret Service keyring (GNOME Keyring, KWallet) or, without one, to a file only you can
read. "Use my mpv settings" reads `~/.config/mpv/mpv.conf`.

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

A build from source has no AniList or MyAnimeList app registration of its own, so the
Trackers tab asks for a client ID the first time you connect. To bundle your own, copy
`.env.example` to `.env.local` and fill it in before building; it explains where to
register.

## macOS

The core and the CLI build and run on a Mac with Cargo as they are. The Qt shell does not
build on `main` yet: its build looks for MpvQt under `/usr`. The port
(MpvQt from source, the Keychain for tokens, an `AniBeam.app` and a dmg) is on the
`feat/release-pipeline` branch. The longer-term Mac shell is SwiftUI over the same core;
Appendix A of `docs/superpowers/specs/2026-09-04-native-line-design.md` describes it.

## Development

The Rust core lives in `core/`, the Qt shell in `apps/linux/`, and a terminal shell over the
same core in `apps/cli/`. Cargo builds all three, not Bun.

```bash
cargo build --release -p anibeam-cli   # target/release/anibeam-cli
cargo test --workspace                 # unit tests plus core/tests/
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

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

The original Electron app still lives in `src/`, frozen: it takes fixes only, and phase 3
of the native line deletes it in one commit. Its last release is the zip on v2.0.0. Bun
builds it, and the `bun run verify:*` scripts in `package.json` test it, not the native
app.

```bash
bun install
bun run dev          # typecheck + electron-forge start with HMR
bun run package      # typecheck + package to out/AniBeam-linux-x64/anibeam
```

## Releasing

Push a version tag. `.github/workflows/release.yml` still builds only the Electron zip and
attaches it to a GitHub Release; the native `anibeam` binary is built locally and attached
by hand, as it was for v2.0.0.

```bash
git tag v2.1.0
git push origin v2.1.0
```
