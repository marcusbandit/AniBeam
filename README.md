# AniBeam

<img src="assets/icon.png" alt="AniBeam icon" width="96" align="right" />

Electron app for browsing, playing, and tracking a local anime library. It scans your configured folders, matches series against AniList/MAL, transcodes incompatible video for in-window playback, renders a franchise relation graph, and syncs watch progress with AniList and MyAnimeList.

Linux x64 is the supported target.

## Install from a GitHub release

1. Download the latest `AniBeam-linux-x64-<version>.zip` from [Releases](https://github.com/marcusbandit/AniBeam/releases).
2. Unzip it wherever you want the app to live, e.g. `~/Apps/AniBeam-linux-x64/`.
3. Run the bundled desktop installer:

```bash
bash ~/Apps/AniBeam-linux-x64/resources/install-desktop.sh
```

That installs the launcher entry (`anibeam.desktop`) and the app icon into `~/.local/share`, no root needed. AniBeam then shows up in your app launcher like any other application.

Runtime requirements: `ffmpeg` and `ffprobe` on PATH (used for playback of formats the browser engine can't decode).

## Install from source

```bash
git clone https://github.com/marcusbandit/AniBeam.git
cd AniBeam
bun install
bun run package          # builds out/AniBeam-linux-x64/
bun run install:desktop  # launcher entry + icon
```

[Bun](https://bun.sh) is the package manager and script runner; npm/yarn are not supported.

## Development

```bash
bun run dev          # typecheck + electron-forge start with HMR
bun run package      # typecheck + package to out/AniBeam-linux-x64/anibeam
bun run typecheck    # tsc --noEmit
bun run lint         # eslint + typecheck
```

Tests are plain bun scripts under `scripts/verify-*.mjs`, wired as `bun run verify:<name>` (see package.json). API client IDs are configured via `ANIBEAM_`-prefixed env vars; see `.env.example`.

## Native line

The Rust core and its terminal shell live beside the Electron app, in `core/`
and `apps/cli/`. They are the successor line; the Electron tree above is
frozen. Cargo builds them, not Bun.

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

## Releasing

Push a version tag and CI builds the Linux zip and attaches it to a GitHub Release:

```bash
git tag v1.1.0
git push origin v1.1.0
```
