# AniBeam

<img src="assets/icon.png" alt="AniBeam icon" width="96" align="right" />

AniBeam browses, plays and tracks a local anime library. It scans the folders you point it
at, matches each series against AniList (TVmaze for what AniList does not carry), plays
video in the window through libmpv, and syncs watch progress with AniList and MyAnimeList.
It runs on Linux and on Apple silicon Macs: a Qt 6 shell over a Rust core.

## Install on a Mac

AniBeam ships for Apple silicon Macs on macOS 15 or later.

1. Download `AniBeam-<version>-macos-arm64.dmg` from [Releases](https://github.com/marcusbandit/AniBeam/releases).
2. Open it and drag AniBeam into Applications.
3. Open AniBeam once. macOS stops it, because the build is not notarized: go to
   System Settings > Privacy & Security, scroll to the AniBeam line and click Open Anyway.
   From a terminal, `xattr -dr com.apple.quarantine /Applications/AniBeam.app` does the same.

Everything the app needs is inside the bundle, mpv included. Its data lives in
`~/Library/Application Support/anibeam`, tracker tokens go to the login keychain, and
"Use my mpv.conf" reads `~/.config/mpv/mpv.conf`, as mpv itself does.

## Install on Linux

On Arch, the AUR package builds the latest release:

```bash
yay -S anibeam        # or: paru -S anibeam
```

Anywhere else, download `AniBeam-<version>-x86_64.AppImage` from
[Releases](https://github.com/marcusbandit/AniBeam/releases), make it executable and run it.
It needs glibc 2.39 or newer: Ubuntu 24.04, Debian 13, Fedora 40 and later.

```bash
chmod +x AniBeam-*-x86_64.AppImage
./AniBeam-*-x86_64.AppImage
```

Data lives in the XDG directories (`~/.local/share/anibeam`, `~/.config/anibeam`).
Tracker tokens go to the Secret Service keyring (GNOME Keyring, KWallet) when one is
running, and to a file only you can read when none is.

## The Downloads page

The Downloads page searches release feeds, subscribes to shows and hands torrents to
qBittorrent. It is optional: the library, the player and the trackers work without it.
It needs two programs AniBeam does not ship:

1. [anirss](https://github.com/marcusbandit/anirss) 0.4.0 or newer. AniBeam starts it in
   the background when the page needs it.
   - Mac: `brew install marcusbandit/anirss/anirss`
   - Arch: `yay -S anirss`
   - Anywhere with Python 3.11 or newer: `uv tool install git+https://github.com/marcusbandit/anirss`
2. qBittorrent with its Web UI turned on. In qBittorrent, open Preferences > Web UI, tick
   the Web UI box, and set a username and password (qBittorrent 5 has no default password).
   Then fill in the same address (`http://localhost:8080` unless you changed the port),
   username and password under AniBeam's Settings > Downloads > Torrent client, and Save.
   On a Mac, `brew install --cask qbittorrent` installs it.

If anirss is missing or older than 0.4.0, the Downloads page says so and names the command
that fixes it.

## Share over Tailscale

AniBeam can share a read-only copy of your library with the AniBeam app on your phone.
Turn on Settings > Library > Phone app > Share over Tailscale. It needs
[Tailscale](https://tailscale.com) running on this computer and on the phone, signed in to
the same tailnet.

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

You need Rust 1.95 or newer, Qt 6 (base, declarative, svg), libmpv and KDE's MpvQt.
On Arch that is `pacman -S rust qt6-base qt6-declarative qt6-svg mpv mpvqt`.

```bash
git clone https://github.com/marcusbandit/AniBeam.git
cd AniBeam
cargo build --release -p anibeam     # target/release/anibeam
```

A build from source has no AniList or MyAnimeList app registration of its own, so the
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
