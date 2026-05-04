# Live Scan Feedback, Always-On Watcher, and Dry-Run Probe

**Date:** 2026-05-04
**Status:** Approved (verbal, 2026-05-04)

## Problem

The current scan pipeline (`scan-and-fetch-metadata` → folder walk → metadata fetch → image cache → thumbnail gen → DB write) is silent from the renderer's perspective. The user sees a static "Scanning…" string and a spinner that never updates. All progress signal lives in `console.log` calls in the main process, invisible to the user.

Three additional gaps:

1. Removing a folder from the library (or deleting files on disk) does not invalidate the corresponding metadata, cached images, or generated thumbnails. The "cache" outlives its source.
2. There is no continuous file watcher. New files only appear after a manual rescan.
3. When a file is detected mid-download, there is no signal that it may not yet be playable.

## Goals

- Surface scan activity as a live, in-app event log accessible from any page.
- Make absolute file paths the source of truth: if the file is gone, all derived state for it is gone.
- Run a continuous file watcher on every library root for the lifetime of the app.
- Probe newly-discovered files for playability and reflect "not ready" state in the UI without blocking metadata fetching.

## Non-Goals

- Persisting the activity log across app restarts.
- Re-architecting the existing scan pipeline beyond what these features require.
- Supporting "soft delete" / re-attachment of removed drives — gone is gone.
- Live-reloading the UI for every metadata change unrelated to status.

## Design

### 1. Activity log (drawer)

A persistent drawer mounted in the app shell, accessible from any page via a toggle in the bottom-right corner.

**Main process — `src/main/services/logger.ts`:**

- Ring buffer of ~5000 events held in memory for the session.
- Public API: `log.info(stage, message, ctx?)`, `log.warn(...)`, `log.error(...)`.
- Each event:
  ```ts
  type LogEvent = {
    id: number;          // monotonic
    ts: number;          // epoch ms
    level: 'info' | 'warn' | 'error';
    stage: 'folder' | 'metadata' | 'image' | 'thumbnail' | 'watch' | 'probe' | 'system';
    message: string;
    ctx?: { series?: string; file?: string };
  };
  ```
- On every log call: append to ring buffer, broadcast via `webContents.send('log:event', event)` to all renderer windows, and forward to `console.log` so terminal output is unchanged.

**IPC channels:**

- `log:event` (main → renderer, broadcast) — single new event.
- `log:get-buffer` (renderer → main, request) — returns the current ring buffer (used when the drawer first mounts or reconnects).
- `log:clear` (renderer → main) — clears the buffer.

**Renderer:**

- `src/renderer/contexts/ActivityLogContext.tsx` — subscribes to `log:event`, maintains the in-renderer buffer, exposes filter state (active stages, min level), exposes `clear()`.
- `src/renderer/components/ActivityLogDrawer.tsx` — collapsible drawer:
  - Toggle button in the bottom-right of the app shell, badge with unseen-error count.
  - Header: stage filter chips (`folder` · `metadata` · `image` · `thumbnail` · `watch` · `probe` · `system`), level filter, "Clear", "Copy all".
  - Body: virtualized list (newest at bottom), auto-scroll if scrolled to bottom.
  - Each row: timestamp · stage badge · message · optional series/file context. Severity drives the row tint.
- Visual style follows existing prefs: flat surfaces, mono-forward typography, large radii, dark only, no emoji.

**Migration of existing logging:**

- All `console.log` calls inside the scan pipeline (`folderHandler`, `imageCacheHandler`, `thumbnailHandler`, `metadataHandler`, MAL/AniList handlers, and the `scan-and-fetch-metadata` block in `main.ts`) are replaced with `logger` calls of the appropriate stage.
- The CLI progress bars in `src/main/utils/debugUtils.ts` are preserved — `updateProgress` additionally emits a `log.info('image', '...')` (or whichever stage) so the drawer reflects the same ticks.

### 2. Cache invalidation — paths are truth

"Cache" here means saved metadata entries, cached images, and generated thumbnails.

**Reconcile rules** (single function in `folderHandler.ts`, e.g. `reconcileMetadata()`):

For each series in saved metadata:
- For each episode-file entry: if the absolute path no longer exists on disk, drop the entry.
- If the series ends up with zero file entries:
  - Delete the series row.
  - Delete its cached poster, banner, and any other series-level images from the image cache directory.
  - Delete its generated episode thumbnails.

**Triggers:**

- At the start of every full scan.
- When a library root is removed in Settings (immediate, synchronous, before the next scan).
- After every batch of watcher `unlink` / `unlinkDir` events.

**Behavior:** no soft delete, no "missing" flag, no waiting for the user to plug a drive back in. If the path is not on disk, the entry leaves.

### 3. Always-on file watcher

**Dependency:** add `chokidar` (de facto standard for cross-platform recursive watching; handles atomic writes, renames, and the rough edges of `fs.watch`).

**Service — `src/main/services/watcher.ts`:**

- `start(roots: string[])` — opens a chokidar watcher per root (recursive, ignored: dotfiles, common partial-download extensions).
- `restart(roots: string[])` — called when the library list mutates.
- `stop()` — called on app shutdown.
- Events are debounced + batched on a 1s window:
  - `add` → enqueue to single-file scan path (see below).
  - `unlink` → reconcile that path out, log a `watch` event.
  - `unlinkDir` → reconcile the whole subtree out.
  - `addDir` → no-op (children produce their own `add`).
- All transitions logged via the `watch` stage.

**Single-file scan path:**

- Refactor `folderHandler` to expose `scanSingleFile(absPath: string): Promise<NewFileResult>` that performs the same series-name inference + metadata lookup currently done per-file inside `scanFolder`, but for one file. The full-scan path uses this internally for parity.
- Result is fed into the existing metadata-fetch + image-cache + thumbnail pipeline, scoped to the single file.
- On the resulting episode-file entry, status is set to `'verifying'` and the file is enqueued for probing.

**Lifecycle:**

- Watcher starts in `app.whenReady()` after libraries are loaded.
- Restart hook is wired to whatever Settings code currently mutates the library list (`scan-and-fetch-metadata` add/remove paths).
- Stop hook in the existing `before-quit` handler (or added if absent).

### 4. Dry-run probe

**Handler — `src/main/handlers/videoProbeHandler.ts`:**

- `probe(absPath: string): Promise<{ ready: boolean; reason?: string }>`
- Implementation: `spawn('ffprobe', ['-v', 'error', '-show_streams', '-show_format', '-of', 'json', absPath])`, parse stdout JSON.
- Ready iff: exit code 0 AND at least one stream with `codec_type === 'video'` AND `format.duration > 0`.
- On non-zero exit or parse failure: `{ ready: false, reason: 'ffprobe failed' }`.

**Probe queue:**

- In-memory map: `path → { attempts, nextRunAt, lastSize, lastMtime }`.
- Backoff schedule: 5s, 15s, 30s, then 60s repeating.
- Maximum lifetime per file: 30 minutes from first enqueue → if still not ready, status flips to `'stalled'`. Removed from queue.
- Short-circuit: a separate poll every 2s checks `(size, mtime)` for every queued path; if both are unchanged for 10s, run a probe immediately on top of whatever the backoff schedule says (catches "download just finished").
- On success: status flips to `'ready'`, removed from queue, `probe` log emitted.
- Manual retry: clicking the `STALLED · retry` badge re-enqueues with attempts reset.

**IPC:**

- `probe:retry` (renderer → main, takes a file path) — re-enqueue.
- Probe status changes are persisted via the existing metadata save path. The renderer learns of changes by re-reading metadata after probe events. (Optional optimization later: a dedicated `metadata:file-changed` IPC channel; deferred.)

### 5. Data model changes

The existing episode-file entry shape (in whatever the metadata handler stores) gains:

```ts
status: 'ready' | 'verifying' | 'stalled';
lastProbedAt?: number;  // epoch ms
```

**Migration:** on first load after upgrade, every existing entry without a `status` field is treated as `'ready'`. No probe storm.

### 6. UI status visuals

`EpisodeCard.tsx` and `ShowCard.tsx`:

- When the underlying file (or, for a show card, all of its files) has `status !== 'ready'`:
  - Render at 60% opacity.
  - Render a small mono-uppercase badge in a corner:
    - `verifying` → `VERIFYING`
    - `stalled` → `STALLED · RETRY` (clickable, fires `probe:retry`)
- A series with mixed statuses shows the badge of its "least ready" file (`stalled` > `verifying` > `ready`).

`SettingsTab.tsx`:

- Drop the static `scanProgress` text + spinner block (the drawer replaces it).
- The "Scan" button still exists; it remains the manual trigger for a full reconcile + scan.

## Architecture

```
┌────────────────────────────────────────────────────────────────┐
│ Main process                                                   │
│                                                                │
│  ┌───────────┐   ┌───────────┐   ┌─────────────────────────┐   │
│  │  watcher  │──▶│ scan      │──▶│ metadata / image /      │   │
│  │ (chokidar)│   │ pipeline  │   │ thumbnail handlers      │   │
│  └─────┬─────┘   └─────┬─────┘   └──────────┬──────────────┘   │
│        │ unlink         │                    │ status changes  │
│        ▼               ▼                    ▼                  │
│  ┌─────────────────────────────────────────────────────┐       │
│  │ reconcileMetadata + metadataHandler.save            │       │
│  └─────────────────────────────────────────────────────┘       │
│                                                                │
│  ┌──────────┐      ┌────────────────────────────────────┐      │
│  │ probeQueue├─────▶│ videoProbeHandler (ffprobe spawn) │      │
│  └────┬─────┘      └──────────────┬─────────────────────┘      │
│       │                            │ status update             │
│       └────────────────────────────┘                           │
│                                                                │
│  ┌────────────────────── logger ──────────────────────┐        │
│  │ ring buffer + console.log + IPC broadcast          │        │
│  └─────────────────────────┬──────────────────────────┘        │
└─────────────────────────────┼──────────────────────────────────┘
                              │ webContents.send('log:event', …)
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ Renderer                                                       │
│                                                                │
│  ActivityLogContext  ──▶  ActivityLogDrawer (toggleable)       │
│                                                                │
│  EpisodeCard / ShowCard read status from metadata, render       │
│  badges + opacity; STALLED · RETRY → ipc 'probe:retry'         │
└────────────────────────────────────────────────────────────────┘
```

## Files

**New**
- `src/main/services/logger.ts`
- `src/main/services/watcher.ts`
- `src/main/handlers/videoProbeHandler.ts`
- `src/renderer/contexts/ActivityLogContext.tsx`
- `src/renderer/components/ActivityLogDrawer.tsx`

**Modified**
- `src/main/main.ts` — boot logger + watcher + probe queue; expose new IPC channels; wire single-file scan; keep `scan-and-fetch-metadata` as the manual full-scan entry.
- `src/main/handlers/folderHandler.ts` — route `console.log` through logger; expose `scanSingleFile`; expose `reconcileMetadata`.
- `src/main/handlers/imageCacheHandler.ts` — logger; honor reconcile (delete cached images for dropped series).
- `src/main/handlers/thumbnailHandler.ts` — logger; honor reconcile (delete thumbs for dropped series).
- `src/main/handlers/metadataHandler.ts` — `status` / `lastProbedAt` fields; migration on load.
- `src/main/utils/debugUtils.ts` — fork `updateProgress` output through logger as well as the CLI bar.
- `src/preload.ts` — expose `log:*`, `probe:*` channels.
- `src/renderer/App.tsx` — mount `ActivityLogContext` provider + `ActivityLogDrawer`.
- `src/renderer/components/EpisodeCard.tsx` — status visuals.
- `src/renderer/components/ShowCard.tsx` — status visuals.
- `src/renderer/components/SettingsTab.tsx` — remove the `scanProgress` text block; keep the Scan button.

**Dependencies**
- Add `chokidar` to `dependencies`.
- `ffprobe` is provided by the system `ffmpeg` install (already required by `thumbnailHandler`).

## Testing

- Manual smoke: start app, drop a video file into a library folder, observe drawer events (`watch` add → `metadata` fetch → `probe` verifying → `probe` ready), card transitions from dimmed `VERIFYING` → normal.
- Manual smoke: delete a file from disk, observe `watch` unlink → reconcile → card disappears.
- Manual smoke: copy a large file in (so the probe sees it mid-write) → confirm metadata appears immediately, card stays dimmed until the copy completes, then resolves.
- Manual smoke: remove a library root in Settings → all series rooted there vanish, including their cached images and thumbs.
- Manual smoke: restart the app → no probe storm; existing entries stay `ready`.

## Open questions

None at design time. Known follow-ups for later iterations (out of scope here):

- Persisting the activity log to a rotating file for postmortem.
- Per-file streaming probe events to the renderer (current design relies on metadata save + reload).
- Throttling probe fan-out if the watcher discovers thousands of files at once (current backoff already prevents thrash, but a global concurrency cap could help further).
