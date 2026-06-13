# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

AniBeam is an Electron app for browsing, playing, and tracking a local anime library: it scans configured folders, matches series against AniList/MAL, transcodes incompatible video for in-window playback, and renders a franchise relation graph.

## Git Workflow

Use git professionally, even though this is (and will likely stay) a single-developer project:

- **Never commit work directly on `main`.** All work happens on branches (`feat/...`, `fix/...`, `docs/...`), merged back when done.
- **Merge only working code.** Before merging to `main`: `bun run typecheck` must pass and the relevant `verify:*` scripts must be green. Never merge broken or half-finished work — `main` must always build and package.
- Keep branches focused (one feature/fix per branch) and delete them after merging.
- Commit messages follow the existing `type(scope): summary` style (`feat(transcode): ...`, `fix(feed): ...`).

## Commands

Bun is the package manager and script runner (`bun install`, never npm/yarn).

```bash
bun run dev          # typecheck + electron-forge start with DEV_MODE=true (HMR)
bun run package      # typecheck + package to out/AniBeam-linux-x64/anibeam
bun run typecheck    # tsc --noEmit
bun run lint         # eslint + typecheck
```

The user launches the app from a .desktop entry pointing at the **packaged** binary, so `bun run package` after finishing a feature — source edits are invisible to the launcher until then. `bun run dev` is for iterating.

### Tests

There is no test framework; the suite is plain bun scripts in `scripts/verify-*.mjs` using `node:assert/strict`, importing TypeScript sources directly (Bun transpiles on the fly). Run one directly:

```bash
bun --bun scripts/verify-franchise-graph.mjs
```

Wired into package.json: `verify:logger`, `verify:motion`, `verify:probe`, `verify:folder`, `verify:franchise-graph`, `verify:franchise-crawler`, `verify:franchise-lanes`. Note `scripts/verify-episode-classifier.mjs` exists but has no package.json entry — run it directly. There is no `verify:all`; run them individually.

## Architecture

Electron Forge + Vite, three build entries: `src/main/main.ts` (vite.main.config.mjs), `src/main/preload.ts` (vite.preload.config.mjs), `src/renderer/` (vite.renderer.config.mjs). Env vars use the `ANIBEAM_` prefix (see `.env.example`: AniList/MAL client IDs, inlined at build time with a per-user fallback in the Trackers settings tab).

### Process layers

- **`src/main/handlers/`** — singleton domain objects (folderHandler, metadataHandler, transcodeCacheHandler, trackerHandler, anilistHandler, malHandler, …) holding the actual logic. They import each other directly; no DI.
- **`src/main/ipc/`** — thin `register*Ipc()` modules that bind channels to handlers; all registered in `app.whenReady()` in `main.ts`.
- **`src/main/services/`** — long-lived processes: chokidar `watcher.ts`, `logger.ts`, `trackerStore.ts`, `franchiseGraph.ts` store, `viewHistory.ts`.
- **`src/main/preload.ts`** — the single source of truth for the renderer-facing API (`contextBridge` → `window.electronAPI`); `src/types/electron.d.ts` re-exports its types. The renderer must never import main-process code.
- **`src/shared/`** — pure isomorphic logic with no Electron imports, usable from main, renderer, and verify scripts: `franchise.ts` (graph BFS closure), `episodeClassifier.ts` (filename → episode/OP/ED/SP parsing), `extraLabels.ts`, `logTypes.ts`, `trackerConstants.ts`.

Main→renderer push events (via `webContents.send`): `metadata:file-status-changed`, `metadata:transcode-progress`, `transcode:queue-changed`, `log:event`, `tracker:progress-changed`, `playback:view-history-changed`, `franchise:store-updated`.

### Renderer

HashRouter pages in `src/renderer/pages/` (Home, SeriesDetail, Feed, Watching, Subscriptions, MetadataTab, SettingsTab, VideoPlayer). No global store — four domain contexts (`TitleLanguageContext`, `TrackerProgressContext`, `ViewHistoryContext`, `ActivityLogContext`) plus hooks (`useMetadata`, `useFranchiseGraph`, `useTranscodeQueue`, `useLocalStorage`). Reusable UI lives in `src/renderer/components/primitives/` (Card, Tooltip, SegmentedSwitch, …) — check there before building new UI. Styling is plain CSS in `App.css` with design-token custom properties (`--bg-primary`, `--accent-primary`, `--radius-pill`, ambient-cursor vars).

### Key flows

- **Library scanning**: chokidar watcher (`awaitWriteFinish` 500ms) + `addDir` subtree walk + one-shot startup catch-up in `main.ts`. **No intervals or periodic rescans — this is deliberate; don't add polling.** Re-scans reconcile `metadata.json` with disk while preserving persistent per-file fields (transcodedPath, status).
- **Metadata matching**: `metadataMatcher`/`posterMatch` query AniList + Jikan in parallel, score with `titleSimilarity`, persist to `userData/metadata.json` (atomic PID-suffixed tmp+rename, transaction-serialized writes). Failed matches set `posterMatchAttempted` so they're never re-hammered.
- **Playback**: in-window video is HTML5 `<video>` + ffmpeg transcode-to-cached-MP4 — **not embedded mpv; Wayland+NVIDIA blocks every mpv embedding path, don't revisit**. `videoProbeHandler` (queued ffprobe with backoff) decides playability; `transcodeCacheHandler` runs one ffmpeg at a time into `userData/transcode-cache/` keyed by `sha256(path:mtime:size)`. ASS subtitles render via JASSUB — **`VideoPlayer.tsx` contains load-bearing JASSUB/libass workarounds; do not "clean up" anything that looks redundant there**.
- **Franchise graph**: `shared/franchise.ts` `closeGraph()` BFS-closes relations; CHARACTER and OTHER edges are kept for display but never traversed (cameos must not drag in unrelated franchises). Cross-franchise hops are stored as links/pointers, not embedded duplicate files. `franchiseCrawler` is rate-limit-aware with deferred-retry; store lives in `userData/franchiseStore.json` + `userData/franchises/franchise-<rootId>.json`.
- **Trackers**: AniList (implicit grant) and MAL (PKCE) via `trackerHandler` + `trackerStore`; OAuth loopback constants in `shared/trackerConstants.ts`.

### Cross-cutting constraints

- **Rate limiting**: every AniList/MAL/Jikan call goes through the shared per-provider `RateLimiter` (`src/main/utils/rateLimiter.ts`, exponential backoff on 429). Never call those APIs directly.
- **Activity log is signal-only**: `logger.*` feeds the user-facing drawer. Log state changes only — never per-asset/per-candidate/per-browse chatter.
- **No native `title=` tooltips**: route hover affordances through the custom portal tooltip primitive (`components/primitives/Tooltip.tsx` / `Card.tooltip`).
- **App behavior must be self-contained** — no Hyprland windowrules, hyprctl, or other compositor coupling.
- `vendor/extract-zip-shim/` works around a Node fd-slicer/zlib deadlock by shelling out to `unzip(1)` — leave it in place.
- Persistence lives under Electron `userData/` (config.json, metadata.json, image-cache/ with 30-day expiry, transcode-cache/, franchise store); view history is renderer localStorage.
