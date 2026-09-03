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

Wired into package.json: `verify:logger`, `verify:motion`, `verify:probe`, `verify:folder`, `verify:franchise-graph`, `verify:franchise-crawler`, `verify:franchise-lanes`, `verify:transcode-cancel`, `verify:tmdb-shape`, `verify:mpv-ipc`. Note `scripts/verify-episode-classifier.mjs` exists but has no package.json entry — run it directly. There is no `verify:all`; run them individually.

Most are pure, but two aren't: `verify:transcode-cancel` and `verify:tmdb-shape` stub `electron`/`configHandler` through `bun:test`'s `mock.module` (so they never touch the real config.json), and `verify:mpv-ipc` drives a **real** mpv against an ffmpeg-generated clip — it's the only thing that can prove the JSON-IPC wire format, and skips cleanly when mpv/ffmpeg aren't on PATH. `verify:frame-step-chromium` is the same idea for the player's frame stepping: it drives a real headless Chrome (`google-chrome-stable` or whatever `ANIBEAM_CHROME` points at) over CDP against a generated clip with the frame number burned into the picture, and skips when Chrome or ffmpeg is missing. `verify:frame-step` is the pure half of that pair.

## Architecture

Electron Forge + Vite, three build entries: `src/main/main.ts` (vite.main.config.mjs), `src/main/preload.ts` (vite.preload.config.mjs), `src/renderer/` (vite.renderer.config.mjs). Env vars use the `ANIBEAM_` prefix (see `.env.example`: AniList/MAL client IDs, inlined at build time with a per-user fallback in the Trackers settings tab).

### Process layers

- **`src/main/handlers/`** — singleton domain objects (folderHandler, metadataHandler, transcodeCacheHandler, trackerHandler, anilistHandler, malHandler, …) holding the actual logic. They import each other directly; no DI.
- **`src/main/ipc/`** — thin `register*Ipc()` modules that bind channels to handlers; all registered in `app.whenReady()` in `main.ts`.
- **`src/main/services/`** — long-lived processes: chokidar `watcher.ts`, `logger.ts`, `trackerStore.ts`, `franchiseGraph.ts` store, `viewHistory.ts`.
- **`src/main/preload.ts`** — the single source of truth for the renderer-facing API (`contextBridge` → `window.electronAPI`); `src/types/electron.d.ts` re-exports its types. The renderer must never import main-process code.
- **`src/shared/`** — pure isomorphic logic with no Electron imports, usable from main, renderer, and verify scripts: `franchise.ts` (graph BFS closure), `episodeClassifier.ts` (filename → episode/OP/ED/SP parsing), `extraLabels.ts`, `logTypes.ts`, `trackerConstants.ts`.

Main→renderer push events (via `webContents.send`): `metadata:file-status-changed`, `metadata:transcode-progress`, `transcode:queue-changed`, `log:event`, `tracker:progress-changed`, `playback:view-history-changed`, `playback:mpv-ended`, `franchise:store-updated`.

### Renderer

HashRouter pages in `src/renderer/pages/` (Home, SeriesDetail, Feed, Watching, Subscriptions, MetadataTab, SettingsTab, VideoPlayer). No global store — four domain contexts (`TitleLanguageContext`, `TrackerProgressContext`, `ViewHistoryContext`, `ActivityLogContext`) plus hooks (`useMetadata`, `useFranchiseGraph`, `useTranscodeQueue`, `useLocalStorage`). Reusable UI lives in `src/renderer/components/primitives/` (Card, Tooltip, SegmentedSwitch, …) — check there before building new UI. Styling is plain CSS in `App.css` with design-token custom properties (`--bg-primary`, `--accent-primary`, `--radius-pill`, ambient-cursor vars).

### Key flows

- **Library scanning**: chokidar watcher (`awaitWriteFinish` 500ms) + `addDir` subtree walk + one-shot startup catch-up in `main.ts`. **No intervals or periodic rescans — this is deliberate; don't add polling.** Re-scans reconcile `metadata.json` with disk while preserving persistent per-file fields (transcodedPath, status).
- **Metadata matching**: `metadataMatcher`/`posterMatch` query AniList + Jikan in parallel, score with `titleSimilarity`, persist to `userData/metadata.json` (atomic PID-suffixed tmp+rename, transaction-serialized writes). Failed matches set `posterMatchAttempted` so they're never re-hammered. **`tmdbHandler` is the manual-only escape hatch for the non-anime part of a library** (live-action films, non-anime TV) — AniList has no entry for those at all. It's never part of the automatic sweep; it's reachable only from the match modal's Anime / Film & TV switch. Its output mirrors `anilistHandler.formatMetadata`'s shape with `source: 'tmdb'` and no anilistId/malId, which is correct rather than incomplete: those trackers don't carry live-action. `averageScore` is stored in each provider's **native** scale (AniList 0-100, MAL and TMDB 0-10) and normalised at render time off `source` — don't "fix" one to match another. The API key is per-user in config.json (TMDB keys are personal, so no build-time env fallback).
- **Playback**: in-window video is HTML5 `<video>` + ffmpeg transcode-to-cached-MP4 — **not embedded mpv; Wayland+NVIDIA blocks every mpv embedding path, don't revisit**. `videoProbeHandler` (queued ffprobe with backoff) decides playability; `transcodeCacheHandler` runs one ffmpeg at a time into `userData/transcode-cache/` keyed by `sha256(path:mtime:size)`. ASS subtitles render via JASSUB — **`VideoPlayer.tsx` contains load-bearing JASSUB/libass workarounds; do not "clean up" anything that looks redundant there**. Frame stepping (`,` and `.`) lives in `shared/frameStep.ts` + `hooks/useFrameStep.ts`: each step is a seek anchored on the `requestVideoFrameCallback` timestamp of the frame on screen, never on `currentTime`, and the one-rAF deferral after pausing is measured behaviour (a quarter of first presses anchored stale without it), so keep both.
- **Stopping a transcode**: `transcodeCacheHandler.cancel/cancelAll` kill the active ffmpeg (SIGKILL — TERM makes it finalise the container) or drop the entry. Stops persist to config.json and are restored by `init()` *before* the startup sweep, and `enqueue()` takes a `reason`: `'auto'` is refused for opted-out files, `'user'` (an explicit play) always runs and clears the opt-out. A file that's been dequeued but not yet spawned is tracked as `preparing` with pre-spawn gates — that window contains an ffprobe and can last seconds. **Cancellation resolves, never rejects**: every `enqueue()` call site fire-and-forgets with `void`.
- **External mpv**: "Open with mpv" launches through `services/mpvPlayback`, which attaches a JSON IPC socket (`--input-ipc-server`) and **polls** `time-pos` each second — an *observed* time-pos fires per video frame. On exit, `externalPlaybackHandler` applies the same thresholds the in-window player uses (30s of real forward playback → view history, 85% → AniList/MAL mark, mirroring the hidden-series guard from `tracker:mark-episode`). Watch time only accumulates forward movement at roughly real time, so scrubbing to the credits doesn't count. The resume position round-trips through the renderer (`useMpvPlaybackSync`) because that map is localStorage. A session we never reached over IPC reports `tracked: false` and changes nothing — recording position 0 would wipe a real resume point. Episode attribution comes from `EpisodeRow`'s `data-episode-file`/`-number`/`-extra` attributes, which `ContextMenu` reads off the DOM.
- **Franchise graph**: `shared/franchise.ts` `closeGraph()` BFS-closes relations; CHARACTER and OTHER edges are kept for display but never traversed (cameos must not drag in unrelated franchises). Cross-franchise hops are stored as links/pointers, not embedded duplicate files. `franchiseCrawler` is rate-limit-aware with deferred-retry; store lives in `userData/franchiseStore.json` + `userData/franchises/franchise-<rootId>.json`.
- **Trackers**: AniList (implicit grant) and MAL (PKCE) via `trackerHandler` + `trackerStore`; OAuth loopback constants in `shared/trackerConstants.ts`.

### Cross-cutting constraints

- **Rate limiting**: every AniList/MAL/Jikan call goes through the shared per-provider `RateLimiter` (`src/main/utils/rateLimiter.ts`, exponential backoff on 429). Never call those APIs directly.
- **Activity log is signal-only**: `logger.*` feeds the user-facing drawer. Log state changes only — never per-asset/per-candidate/per-browse chatter.
- **No native `title=` tooltips**: route hover affordances through the custom portal tooltip primitive (`components/primitives/Tooltip.tsx` / `Card.tooltip`).
- **App behavior must be self-contained** — no Hyprland windowrules, hyprctl, or other compositor coupling.
- `vendor/extract-zip-shim/` works around a Node fd-slicer/zlib deadlock by shelling out to `unzip(1)` — leave it in place.
- Persistence lives under Electron `userData/` (config.json, metadata.json, image-cache/ with 30-day expiry, transcode-cache/, franchise store); view history is renderer localStorage.

## Agent skills

### Issue tracker

Issues, specs and wayfinder maps live in this repo's GitHub Issues, driven with `gh`. See `docs/agents/issue-tracker.md`.

### Domain docs

Single-context: one `CONTEXT.md` at the root and ADRs under `docs/adr/`. See `docs/agents/domain.md`.
