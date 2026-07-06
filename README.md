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

## Releasing

Push a version tag and CI builds the Linux zip and attaches it to a GitHub Release:

```bash
git tag v1.1.0
git push origin v1.1.0
```
