# Live Scan Feedback, Watcher, and Probe — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stream scan activity into an in-app drawer, treat absolute file paths as the source of truth (purge cache when files vanish), watch library folders for changes while the app is open, and probe newly-discovered files for playability so the UI can show a "not ready" state without blocking metadata fetch.

**Architecture:** Three new main-process services (`logger`, `watcher`, `videoProbeHandler`) feed events through new IPC channels (`log:event`, `log:get-buffer`, `log:clear`, `probe:retry`). All existing `console.log` calls in the scan pipeline get re-routed through the logger so the drawer reflects real progress. A reconcile pass runs at the start of every scan and after every watcher unlink, deleting metadata, cached images, and thumbnails for files that no longer exist on disk. New episode-file entries gain a `status: 'ready' | 'verifying' | 'stalled'` field; the renderer dims cards and shows a mono-uppercase badge for non-ready items.

**Tech Stack:** Electron + Vite + React + TypeScript (ESM, `moduleResolution: bundler`), `chokidar` (new dep) for cross-platform recursive watching, system `ffprobe` (already required by `thumbnailHandler` via system `ffmpeg`) for video verification.

**Note on testing:** This codebase has no test framework configured (`package.json` has no `jest`/`vitest`/`mocha`, and no `*.test.*`/`*.spec.*` files exist). Adding one for this feature is scope creep. The verification approach in this plan:
- **Pure logic** (logger ring buffer, ffprobe output parsing) is verified by small `scripts/verify-*.mjs` scripts that run via Node and assert with `node:assert`.
- **Type correctness** is verified by `bun run typecheck` after every task.
- **Integration / UI** is verified by explicit manual smoke checks with exact reproduction steps.

The spec is at `docs/superpowers/specs/2026-05-04-live-scan-feedback-and-watcher-design.md`.

---

## File Structure

**New files**
| Path | Responsibility |
|------|----------------|
| `src/main/services/logger.ts` | In-memory ring buffer + IPC broadcaster. Single entry point for all main-process diagnostic output. |
| `src/main/services/watcher.ts` | chokidar lifecycle. `start(roots)`, `restart(roots)`, `stop()`. Debounces and dispatches to scan/reconcile/probe. |
| `src/main/handlers/videoProbeHandler.ts` | `probe(path)` via `ffprobe`. Owns the retry queue, size-stable poll, and manual retry. |
| `src/renderer/contexts/ActivityLogContext.tsx` | Subscribes to `log:event`, holds renderer-side buffer, exposes filter state. |
| `src/renderer/components/ActivityLogDrawer.tsx` | Bottom-right toggle + collapsible drawer with virtualized list, filter chips, clear/copy. |
| `scripts/verify-logger.mjs` | Asserts ring-buffer behavior of the logger. |
| `scripts/verify-probe-parser.mjs` | Asserts the ffprobe-output → ready-decision logic. |

**Modified files**
| Path | What changes |
|------|--------------|
| `src/main/main.ts` | Boot logger + watcher + probe queue. Register `log:*` and `probe:retry` IPC. Wire `scan-and-fetch-metadata` to call reconcile first. Restart watcher on `add-folder-source` / `remove-folder-source`. Add `before-quit` handler. Replace local `console.log` calls with `logger.*`. |
| `src/main/preload.ts` | Expose `onLogEvent`, `getLogBuffer`, `clearLog`, `probeRetry`. |
| `src/main/handlers/folderHandler.ts` | Re-route logging through logger. Add `reconcileMetadata(metadata)` and `scanSingleFile(absPath)`. Set `status: 'ready'` on every produced `VideoFile`. |
| `src/main/handlers/imageCacheHandler.ts` | Re-route logging through logger. (`deleteSeriesImages` already exists; reused by reconcile.) |
| `src/main/handlers/thumbnailHandler.ts` | Re-route logging through logger. Add `deleteSeriesThumbnails(filePaths)` so reconcile can purge them. |
| `src/main/handlers/metadataHandler.ts` | Migration: ensure every loaded `fileEpisodes[i]` has `status` defaulted to `'ready'`. |
| `src/main/utils/debugUtils.ts` | `updateProgress` additionally calls `logger.info` so drawer reflects bar ticks. |
| `src/renderer/App.tsx` | Mount `ActivityLogContextProvider` and `ActivityLogDrawer` (outside `<HashRouter>` route content but inside the provider chain). |
| `src/renderer/components/EpisodeCard.tsx` | Read `status` from the file entry; render badge + dim when `status !== 'ready'`. |
| `src/renderer/components/ShowCard.tsx` | Compute "least ready" status across all `fileEpisodes`; render badge + dim. |
| `src/renderer/components/SettingsTab.tsx` | Remove the static `scan-progress` text block (drawer replaces it). Keep the Scan button. |
| `src/renderer/styles/App.css` | Add styles for drawer, drawer toggle, status badge, dim treatment. |
| `package.json` | Add `chokidar` to `dependencies`. |

---

## Task 1: Logger service + IPC plumbing

**Files:**
- Create: `src/main/services/logger.ts`
- Create: `scripts/verify-logger.mjs`
- Modify: `src/main/preload.ts`
- Modify: `src/main/main.ts`

- [ ] **Step 1.1: Create the logger service**

Create `src/main/services/logger.ts`:

```ts
import { BrowserWindow } from 'electron';

export type LogLevel = 'info' | 'warn' | 'error';
export type LogStage =
  | 'folder'
  | 'metadata'
  | 'image'
  | 'thumbnail'
  | 'watch'
  | 'probe'
  | 'system';

export interface LogEvent {
  id: number;
  ts: number;
  level: LogLevel;
  stage: LogStage;
  message: string;
  ctx?: { series?: string; file?: string };
}

const BUFFER_LIMIT = 5000;
let nextId = 1;
const buffer: LogEvent[] = [];

function broadcast(event: LogEvent): void {
  for (const win of BrowserWindow.getAllWindows()) {
    if (!win.isDestroyed()) {
      win.webContents.send('log:event', event);
    }
  }
}

function record(level: LogLevel, stage: LogStage, message: string, ctx?: LogEvent['ctx']): void {
  const event: LogEvent = { id: nextId++, ts: Date.now(), level, stage, message, ctx };
  buffer.push(event);
  if (buffer.length > BUFFER_LIMIT) buffer.shift();
  const consoleMethod = level === 'error' ? console.error : level === 'warn' ? console.warn : console.log;
  const ctxBit = ctx?.series ? ` [${ctx.series}]` : ctx?.file ? ` [${ctx.file}]` : '';
  consoleMethod(`[${stage}]${ctxBit} ${message}`);
  broadcast(event);
}

export const logger = {
  info(stage: LogStage, message: string, ctx?: LogEvent['ctx']): void {
    record('info', stage, message, ctx);
  },
  warn(stage: LogStage, message: string, ctx?: LogEvent['ctx']): void {
    record('warn', stage, message, ctx);
  },
  error(stage: LogStage, message: string, ctx?: LogEvent['ctx']): void {
    record('error', stage, message, ctx);
  },
  getBuffer(): LogEvent[] {
    return buffer.slice();
  },
  clear(): void {
    buffer.length = 0;
  },
};
```

- [ ] **Step 1.2: Create a verification script for the logger**

Create `scripts/verify-logger.mjs`:

```js
import assert from 'node:assert/strict';

// Stub electron's BrowserWindow before importing the logger
import { Module } from 'node:module';
const originalResolve = Module._resolveFilename;
Module._resolveFilename = function (request, parent, ...rest) {
  if (request === 'electron') return 'electron-stub';
  return originalResolve.call(this, request, parent, ...rest);
};
const originalLoad = Module._load;
Module._load = function (request, parent, ...rest) {
  if (request === 'electron') return { BrowserWindow: { getAllWindows: () => [] } };
  return originalLoad.call(this, request, parent, ...rest);
};

const { logger } = await import('../src/main/services/logger.ts');

logger.clear();
logger.info('system', 'boot');
logger.warn('folder', 'something odd');
logger.error('probe', 'ffprobe failed', { file: '/tmp/x.mkv' });
const buf = logger.getBuffer();
assert.equal(buf.length, 3);
assert.equal(buf[0].level, 'info');
assert.equal(buf[1].stage, 'folder');
assert.equal(buf[2].ctx?.file, '/tmp/x.mkv');
assert.ok(buf[0].id < buf[2].id, 'ids are monotonic');

// Ring-buffer cap
logger.clear();
for (let i = 0; i < 6000; i++) logger.info('system', `m${i}`);
const after = logger.getBuffer();
assert.equal(after.length, 5000);
assert.equal(after[after.length - 1].message, 'm5999');
console.log('OK: logger ring buffer behaves');
```

- [ ] **Step 1.3: Run the verification script**

Run: `bun --bun scripts/verify-logger.mjs` (or `npx tsx scripts/verify-logger.mjs` if `bun` cannot import `.ts` directly — install `tsx` only if needed).
Expected output: `OK: logger ring buffer behaves`.

If `bun` chokes on the `.ts` import, the simpler fix is to compile the logger inline. Rewrite the script's import line as:

```js
const loggerModule = await (async () => {
  const ts = await import('typescript');
  const fs = await import('node:fs/promises');
  const src = await fs.readFile(new URL('../src/main/services/logger.ts', import.meta.url), 'utf8');
  const out = ts.default.transpileModule(src, { compilerOptions: { module: 99, target: 99 } }).outputText;
  const dataUrl = 'data:text/javascript;base64,' + Buffer.from(out).toString('base64');
  return import(dataUrl);
})();
const { logger } = loggerModule;
```

(`bun` ships with native TS support; this fallback is only if you're running via Node.)

- [ ] **Step 1.4: Add log IPC channels to preload**

Modify `src/main/preload.ts`. Extend the `ElectronAPI` interface (currently lines 13-39) by adding inside it, before the closing `}`:

```ts
  // Activity log
  onLogEvent: (handler: (event: LogEvent) => void) => () => void;
  getLogBuffer: () => Promise<LogEvent[]>;
  clearLog: () => Promise<void>;

  // Video probe
  probeRetry: (filePath: string) => Promise<void>;
```

Add an export at the top of the file (near the other type imports):

```ts
export type LogLevel = 'info' | 'warn' | 'error';
export type LogStage = 'folder' | 'metadata' | 'image' | 'thumbnail' | 'watch' | 'probe' | 'system';
export interface LogEvent {
  id: number;
  ts: number;
  level: LogLevel;
  stage: LogStage;
  message: string;
  ctx?: { series?: string; file?: string };
}
```

Inside the `contextBridge.exposeInMainWorld('electronAPI', { ... })` call (currently lines 47-73), add as new keys before the closing `}`:

```ts
  // Activity log
  onLogEvent: (handler: (event: LogEvent) => void) => {
    const listener = (_e: unknown, event: LogEvent) => handler(event);
    ipcRenderer.on('log:event', listener);
    return () => ipcRenderer.removeListener('log:event', listener);
  },
  getLogBuffer: () => ipcRenderer.invoke('log:get-buffer'),
  clearLog: () => ipcRenderer.invoke('log:clear'),

  // Video probe
  probeRetry: (filePath: string) => ipcRenderer.invoke('probe:retry', filePath),
```

- [ ] **Step 1.5: Register log IPC handlers in main.ts**

Modify `src/main/main.ts`. At the top of the file, alongside the other handler imports, add:

```ts
import { logger } from './services/logger.js';
```

(Use `.js` extension because the project compiles to ESM — Vite handles this for renderer; for main, the existing imports follow the same convention. If existing imports omit the extension, omit it here too — match the file's prevailing style.)

In the IPC registration block (the section starting around line 232 with `ipcMain.handle('get-folder-sources', ...)`), add three new handlers — placement doesn't matter, but grouping near the other small handlers is tidy:

```ts
ipcMain.handle('log:get-buffer', () => logger.getBuffer());
ipcMain.handle('log:clear', () => {
  logger.clear();
});
```

(The `log:event` channel is broadcast-only and does not need an `ipcMain.handle`.)

In `createWindow` (after `mainWindow = new BrowserWindow(...)`), add a single boot log so we can verify the channel works end-to-end:

```ts
mainWindow.webContents.once('did-finish-load', () => {
  logger.info('system', 'AniBeam ready');
});
```

- [ ] **Step 1.6: Typecheck**

Run: `bun run typecheck`
Expected: no errors.

- [ ] **Step 1.7: Commit**

```bash
git add src/main/services/logger.ts src/main/preload.ts src/main/main.ts scripts/verify-logger.mjs
git commit -m "feat: add main-process logger service and IPC plumbing"
```

---

## Task 2: Activity log context + drawer UI

**Files:**
- Create: `src/renderer/contexts/ActivityLogContext.tsx`
- Create: `src/renderer/components/ActivityLogDrawer.tsx`
- Modify: `src/renderer/App.tsx`
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 2.1: Create the activity log context**

Create `src/renderer/contexts/ActivityLogContext.tsx`:

```tsx
import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import type { LogEvent, LogStage, LogLevel } from '../../main/preload';

interface ActivityLogContextValue {
  events: LogEvent[];
  stageFilter: Set<LogStage>;
  levelFilter: Set<LogLevel>;
  toggleStage: (stage: LogStage) => void;
  toggleLevel: (level: LogLevel) => void;
  clear: () => void;
  visibleEvents: LogEvent[];
  unseenErrorCount: number;
  markErrorsSeen: () => void;
}

const ALL_STAGES: LogStage[] = ['folder', 'metadata', 'image', 'thumbnail', 'watch', 'probe', 'system'];
const ALL_LEVELS: LogLevel[] = ['info', 'warn', 'error'];

const ActivityLogContext = createContext<ActivityLogContextValue | null>(null);

export function ActivityLogProvider({ children }: { children: ReactNode }) {
  const [events, setEvents] = useState<LogEvent[]>([]);
  const [stageFilter, setStageFilter] = useState<Set<LogStage>>(() => new Set(ALL_STAGES));
  const [levelFilter, setLevelFilter] = useState<Set<LogLevel>>(() => new Set(ALL_LEVELS));
  const [lastSeenErrorId, setLastSeenErrorId] = useState(0);

  useEffect(() => {
    let cancelled = false;
    window.electronAPI.getLogBuffer().then((buf) => {
      if (!cancelled) setEvents(buf);
    });
    const unsubscribe = window.electronAPI.onLogEvent((event) => {
      setEvents((prev) => {
        const next = prev.length >= 5000 ? prev.slice(prev.length - 4999) : prev.slice();
        next.push(event);
        return next;
      });
    });
    return () => {
      cancelled = true;
      unsubscribe();
    };
  }, []);

  const toggleStage = (stage: LogStage) => {
    setStageFilter((prev) => {
      const next = new Set(prev);
      if (next.has(stage)) next.delete(stage);
      else next.add(stage);
      return next;
    });
  };

  const toggleLevel = (level: LogLevel) => {
    setLevelFilter((prev) => {
      const next = new Set(prev);
      if (next.has(level)) next.delete(level);
      else next.add(level);
      return next;
    });
  };

  const clear = async () => {
    await window.electronAPI.clearLog();
    setEvents([]);
    setLastSeenErrorId(0);
  };

  const visibleEvents = useMemo(
    () => events.filter((e) => stageFilter.has(e.stage) && levelFilter.has(e.level)),
    [events, stageFilter, levelFilter],
  );

  const unseenErrorCount = useMemo(
    () => events.reduce((n, e) => (e.level === 'error' && e.id > lastSeenErrorId ? n + 1 : n), 0),
    [events, lastSeenErrorId],
  );

  const markErrorsSeen = () => {
    const lastId = events.length > 0 ? events[events.length - 1].id : 0;
    setLastSeenErrorId(lastId);
  };

  return (
    <ActivityLogContext.Provider
      value={{
        events,
        stageFilter,
        levelFilter,
        toggleStage,
        toggleLevel,
        clear,
        visibleEvents,
        unseenErrorCount,
        markErrorsSeen,
      }}
    >
      {children}
    </ActivityLogContext.Provider>
  );
}

export function useActivityLog(): ActivityLogContextValue {
  const ctx = useContext(ActivityLogContext);
  if (!ctx) throw new Error('useActivityLog must be used inside ActivityLogProvider');
  return ctx;
}

export { ALL_STAGES, ALL_LEVELS };
```

- [ ] **Step 2.2: Create the drawer component**

Create `src/renderer/components/ActivityLogDrawer.tsx`:

```tsx
import { useEffect, useRef, useState } from 'react';
import { Activity, X, Trash2, Copy } from 'lucide-react';
import { useActivityLog, ALL_STAGES, ALL_LEVELS } from '../contexts/ActivityLogContext';

function formatTime(ts: number): string {
  const d = new Date(ts);
  const hh = String(d.getHours()).padStart(2, '0');
  const mm = String(d.getMinutes()).padStart(2, '0');
  const ss = String(d.getSeconds()).padStart(2, '0');
  return `${hh}:${mm}:${ss}`;
}

export function ActivityLogDrawer() {
  const {
    visibleEvents,
    stageFilter,
    levelFilter,
    toggleStage,
    toggleLevel,
    clear,
    unseenErrorCount,
    markErrorsSeen,
  } = useActivityLog();
  const [open, setOpen] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);
  const [stickToBottom, setStickToBottom] = useState(true);

  useEffect(() => {
    if (open && stickToBottom && listRef.current) {
      listRef.current.scrollTop = listRef.current.scrollHeight;
    }
  }, [visibleEvents, open, stickToBottom]);

  useEffect(() => {
    if (open) markErrorsSeen();
  }, [open, markErrorsSeen]);

  const handleScroll = () => {
    const el = listRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 8;
    setStickToBottom(atBottom);
  };

  const handleCopy = async () => {
    const text = visibleEvents
      .map((e) => `${formatTime(e.ts)} [${e.level}] [${e.stage}] ${e.ctx?.series ? `(${e.ctx.series}) ` : ''}${e.message}`)
      .join('\n');
    await navigator.clipboard.writeText(text);
  };

  return (
    <>
      <button
        className={`activity-log-toggle${unseenErrorCount > 0 ? ' has-errors' : ''}`}
        onClick={() => setOpen((v) => !v)}
        aria-label="Toggle activity log"
      >
        <Activity size={16} />
        <span>Activity</span>
        {unseenErrorCount > 0 && <span className="activity-log-badge">{unseenErrorCount}</span>}
      </button>
      {open && (
        <aside className="activity-log-drawer">
          <header className="activity-log-header">
            <span className="activity-log-title">Activity</span>
            <div className="activity-log-actions">
              <button className="activity-log-action" onClick={handleCopy} aria-label="Copy log">
                <Copy size={14} />
              </button>
              <button className="activity-log-action" onClick={clear} aria-label="Clear log">
                <Trash2 size={14} />
              </button>
              <button className="activity-log-action" onClick={() => setOpen(false)} aria-label="Close">
                <X size={14} />
              </button>
            </div>
          </header>
          <div className="activity-log-filters">
            {ALL_STAGES.map((stage) => (
              <button
                key={stage}
                className={`activity-log-chip${stageFilter.has(stage) ? ' active' : ''}`}
                onClick={() => toggleStage(stage)}
              >
                {stage}
              </button>
            ))}
            <span className="activity-log-sep" />
            {ALL_LEVELS.map((level) => (
              <button
                key={level}
                className={`activity-log-chip level-${level}${levelFilter.has(level) ? ' active' : ''}`}
                onClick={() => toggleLevel(level)}
              >
                {level}
              </button>
            ))}
          </div>
          <div className="activity-log-list" ref={listRef} onScroll={handleScroll}>
            {visibleEvents.length === 0 && <div className="activity-log-empty">No events.</div>}
            {visibleEvents.map((e) => (
              <div key={e.id} className={`activity-log-row level-${e.level}`}>
                <span className="activity-log-ts">{formatTime(e.ts)}</span>
                <span className="activity-log-stage">{e.stage}</span>
                <span className="activity-log-msg">{e.message}</span>
                {e.ctx?.series && <span className="activity-log-ctx">{e.ctx.series}</span>}
                {e.ctx?.file && !e.ctx.series && <span className="activity-log-ctx">{e.ctx.file}</span>}
              </div>
            ))}
          </div>
        </aside>
      )}
    </>
  );
}
```

- [ ] **Step 2.3: Mount the provider and drawer in App.tsx**

Modify `src/renderer/App.tsx`. Add imports at the top:

```tsx
import { ActivityLogProvider } from './contexts/ActivityLogContext';
import { ActivityLogDrawer } from './components/ActivityLogDrawer';
```

Wrap the existing root render. Replace the existing `App` function body (currently lines 66-74):

```tsx
function App() {
  return (
    <HashRouter>
      <ActivityLogProvider>
        <AppContent />
        <ActivityLogDrawer />
      </ActivityLogProvider>
    </HashRouter>
  );
}
```

(The drawer is sibling to `AppContent` so it overlays the entire app, including the player route.)

- [ ] **Step 2.4: Add styles**

Append to `src/renderer/styles/App.css`:

```css
/* Activity log */
.activity-log-toggle {
  position: fixed;
  bottom: 16px;
  right: 16px;
  z-index: 9000;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: 1px solid #2a2a32;
  background: #14141a;
  color: #d8d8e0;
  border-radius: 14px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  cursor: pointer;
}
.activity-log-toggle:hover { background: #1a1a22; }
.activity-log-toggle.has-errors { border-color: #b54343; }
.activity-log-badge {
  background: #b54343;
  color: #fff;
  border-radius: 10px;
  padding: 0 6px;
  font-size: 11px;
}

.activity-log-drawer {
  position: fixed;
  bottom: 64px;
  right: 16px;
  width: 520px;
  max-width: calc(100vw - 32px);
  height: 60vh;
  max-height: 600px;
  z-index: 9000;
  display: flex;
  flex-direction: column;
  background: #0e0e14;
  border: 1px solid #2a2a32;
  border-radius: 16px;
  overflow: hidden;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  color: #d8d8e0;
}
.activity-log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid #20202a;
}
.activity-log-title { font-weight: 600; letter-spacing: 0.04em; }
.activity-log-actions { display: flex; gap: 4px; }
.activity-log-action {
  background: transparent;
  border: 1px solid transparent;
  color: #9a9aa6;
  padding: 4px;
  border-radius: 8px;
  cursor: pointer;
}
.activity-log-action:hover { background: #1a1a22; color: #d8d8e0; }

.activity-log-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 8px 14px;
  border-bottom: 1px solid #20202a;
}
.activity-log-chip {
  background: transparent;
  border: 1px solid #2a2a32;
  color: #6a6a76;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  cursor: pointer;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.activity-log-chip.active { color: #d8d8e0; border-color: #4a4a58; background: #1a1a22; }
.activity-log-chip.level-warn.active { border-color: #b58843; color: #e0c089; }
.activity-log-chip.level-error.active { border-color: #b54343; color: #e08989; }
.activity-log-sep { width: 1px; align-self: stretch; background: #20202a; margin: 0 4px; }

.activity-log-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
}
.activity-log-empty { padding: 14px; color: #6a6a76; }
.activity-log-row {
  display: grid;
  grid-template-columns: 64px 80px 1fr auto;
  gap: 8px;
  padding: 4px 14px;
  align-items: baseline;
}
.activity-log-row:hover { background: #14141a; }
.activity-log-row.level-warn .activity-log-msg { color: #e0c089; }
.activity-log-row.level-error .activity-log-msg { color: #e08989; }
.activity-log-ts { color: #6a6a76; }
.activity-log-stage { color: #8a8a96; text-transform: uppercase; font-size: 10.5px; letter-spacing: 0.06em; }
.activity-log-msg { color: #d8d8e0; word-break: break-word; }
.activity-log-ctx { color: #6a6a76; font-style: italic; }
```

- [ ] **Step 2.5: Typecheck and dev smoke**

Run: `bun run typecheck`
Expected: no errors.

Run: `bun run dev`
Expected: app starts, an "Activity" pill is visible at bottom-right. Click it. The drawer opens; you should see one entry: `system | AniBeam ready`. Stage filter chips and level chips toggle visibility. Clear empties the list. Close it.

If the drawer doesn't appear: open DevTools (Ctrl+Shift+I in dev mode), check console for IPC errors. Most likely cause is the preload import path or a missing channel registration in `main.ts`.

- [ ] **Step 2.6: Commit**

```bash
git add src/renderer/contexts/ActivityLogContext.tsx src/renderer/components/ActivityLogDrawer.tsx src/renderer/App.tsx src/renderer/styles/App.css
git commit -m "feat: add activity log context and drawer UI"
```

---

## Task 3: Wire scan pipeline through logger

**Files:**
- Modify: `src/main/handlers/folderHandler.ts`
- Modify: `src/main/handlers/imageCacheHandler.ts`
- Modify: `src/main/handlers/thumbnailHandler.ts`
- Modify: `src/main/handlers/metadataHandler.ts`
- Modify: `src/main/handlers/malHandler.ts` (whatever the actual filename is — confirm via `ls src/main/handlers`)
- Modify: `src/main/handlers/anilistHandler.ts`
- Modify: `src/main/utils/debugUtils.ts`
- Modify: `src/main/main.ts`

- [ ] **Step 3.1: Add logger import to each handler**

In each file listed above, add at the top alongside the other imports:

```ts
import { logger } from '../services/logger.js';
```

(`debugUtils.ts` is in `src/main/utils/`, so its import is `'../services/logger.js'` as well.)

- [ ] **Step 3.2: Replace console.log calls in folderHandler.ts**

In `src/main/handlers/folderHandler.ts`, find every `console.log(...)`, `console.warn(...)`, and `console.error(...)`. Replace with the logger:

- `console.log('Scanning ${path}…')` → `logger.info('folder', 'Scanning ${path}…')`
- `console.warn(...)` → `logger.warn('folder', '...')`
- `console.error(...)` → `logger.error('folder', '...')`

When the message references a series or a file path, pass it through `ctx`:

```ts
logger.info('folder', `Found ${files.length} files`, { series: media.name });
logger.warn('folder', `Skipping unreadable file`, { file: filePath });
```

(Console output is preserved automatically — the logger forks to console under the hood. So nothing regresses for the terminal user.)

- [ ] **Step 3.3: Replace console.log calls in imageCacheHandler.ts**

Same pattern, but use stage `'image'`:

- `console.log('Cached ${url} → ${path}')` → `logger.info('image', `Cached ${url}`, { file: url })`
- Etc.

- [ ] **Step 3.4: Replace console.log calls in thumbnailHandler.ts**

Same pattern, stage `'thumbnail'`:

- `console.log('Generated thumbnail for ${path}')` → `logger.info('thumbnail', 'Generated thumbnail', { file: videoPath })`
- `console.error('ffmpeg error: ${err.message}')` → `logger.error('thumbnail', `ffmpeg error: ${err.message}`, { file: videoPath })`

- [ ] **Step 3.5: Replace console.log calls in metadataHandler.ts, malHandler.ts, anilistHandler.ts**

Stages: `'metadata'` for all of them. Same find-and-replace pattern. Pass `series` in `ctx` where the message references a particular series.

- [ ] **Step 3.6: Hook debugUtils.updateProgress into the logger**

Modify `src/main/utils/debugUtils.ts`. The current `updateProgress(header, filename?)` writes only to the CLI bar. Mirror it as a logger event so the drawer reflects bar ticks.

Find the `updateProgress` function and at the very end of its body — after `instance.bar.update(...)` (both branches if there are two: the early-return auto-init branch and the normal branch) — add:

```ts
logger.info('image', filename ? `${header}: ${filename}` : header);
```

Wait — the stage depends on the header. Sometimes the bar tracks images, sometimes thumbnails, sometimes metadata. Map it:

```ts
function stageFor(header: string): import('../services/logger.js').LogStage {
  const h = header.toLowerCase();
  if (h.includes('thumb')) return 'thumbnail';
  if (h.includes('image') || h.includes('cache')) return 'image';
  if (h.includes('metadata') || h.includes('media')) return 'metadata';
  return 'system';
}
```

Place that helper near the top of the file. Then in `updateProgress`, replace the placeholder line above with:

```ts
logger.info(stageFor(header), filename ? `${header}: ${filename}` : header);
```

Apply the same line to both the auto-init branch and the normal branch (just before each function returns / at the end of each).

- [ ] **Step 3.7: Replace console.log calls in main.ts scan pipeline**

In `src/main/main.ts`, in the `scan-and-fetch-metadata` handler (lines 413-806), replace every `console.log` / `console.warn` / `console.error` related to the scan with the logger, choosing the stage that matches the surrounding step (folder enumeration → `'folder'`, MAL/AniList → `'metadata'`, image → `'image'`, thumbnail → `'thumbnail'`, save → `'metadata'`).

For the per-series banner lines like `console.log('--- Processing ${name} ---')`, use:

```ts
logger.info('metadata', `Processing ${name}`, { series: name });
```

Leave the legacy app-lifecycle `console.log` calls in `app.whenReady`, `window-all-closed` etc. alone — those aren't part of the scan pipeline.

- [ ] **Step 3.8: Typecheck**

Run: `bun run typecheck`
Expected: no errors.

- [ ] **Step 3.9: Manual smoke**

Run: `bun run dev`
In the running app, open the Activity drawer. Open Settings, hit Scan on a folder with at least a few series. Watch the drawer: events should stream in across the `folder`, `metadata`, `image`, and `thumbnail` stages. Stage chip toggles should work to filter them. The terminal should still print everything (forked output).

If a stage produces zero events when it should produce many, grep for any `console.log` you missed in that handler.

- [ ] **Step 3.10: Commit**

```bash
git add src/main/handlers/ src/main/utils/debugUtils.ts src/main/main.ts
git commit -m "feat: route scan-pipeline logging through activity logger"
```

---

## Task 4: Reconcile — paths-as-truth cache invalidation

**Files:**
- Modify: `src/main/handlers/folderHandler.ts` (add `reconcileMetadata`)
- Modify: `src/main/handlers/thumbnailHandler.ts` (add `deleteSeriesThumbnails`)
- Modify: `src/main/main.ts` (call reconcile at scan start and on `remove-folder-source`)

- [ ] **Step 4.1: Add reconcileMetadata in folderHandler.ts**

In `src/main/handlers/folderHandler.ts`, alongside the `folderHandler` object's existing methods, add:

```ts
import { existsSync } from 'fs';
import { imageCacheHandler } from './imageCacheHandler.js';
import { thumbnailHandler } from './thumbnailHandler.js';

interface FileEpisodeEntry {
  filePath: string;
  episodeNumber?: number;
  seasonNumber?: number | null;
  subtitlePath?: string | null;
  subtitlePaths?: string[];
  filename?: string;
  title?: string;
  status?: 'ready' | 'verifying' | 'stalled';
  lastProbedAt?: number;
}

interface SeriesEntry {
  fileEpisodes?: FileEpisodeEntry[];
  poster?: string | null;
  banner?: string | null;
  posterLocal?: string | null;
  bannerLocal?: string | null;
  episodes?: Array<{ thumbnail?: string | null; thumbnailLocal?: string | null }>;
  [k: string]: unknown;
}

/**
 * Drop file entries whose absolute path is gone from disk.
 * If a series ends up with zero files, drop the whole series and purge its
 * cached posters/banners and episode thumbnails.
 *
 * Returns the reconciled metadata object (may share references with the input).
 */
async function reconcileMetadata(metadata: Record<string, unknown>): Promise<Record<string, unknown>> {
  const out: Record<string, unknown> = {};
  let removedFiles = 0;
  let removedSeries = 0;

  for (const [seriesId, raw] of Object.entries(metadata)) {
    const series = raw as SeriesEntry;
    const files = Array.isArray(series.fileEpisodes) ? series.fileEpisodes : [];
    const kept = files.filter((f) => {
      const present = !!f?.filePath && existsSync(f.filePath);
      if (!present) removedFiles++;
      return present;
    });

    if (kept.length === 0 && files.length > 0) {
      // The whole series is gone.
      removedSeries++;
      logger.info('folder', `Reconcile: dropping series (all files missing)`, { series: String(series.title ?? seriesId) });
      try {
        await imageCacheHandler.deleteSeriesImages({
          poster: series.poster ?? null,
          banner: series.banner ?? null,
          posterLocal: series.posterLocal ?? null,
          bannerLocal: series.bannerLocal ?? null,
          episodes: series.episodes ?? [],
        });
      } catch (err) {
        logger.warn('image', `Reconcile: failed deleting cached images: ${(err as Error).message}`, { series: String(series.title ?? seriesId) });
      }
      try {
        await thumbnailHandler.deleteSeriesThumbnails(files.map((f) => f.filePath));
      } catch (err) {
        logger.warn('thumbnail', `Reconcile: failed deleting thumbnails: ${(err as Error).message}`, { series: String(series.title ?? seriesId) });
      }
      continue; // skip — drop the series
    }

    if (kept.length !== files.length) {
      logger.info('folder', `Reconcile: dropped ${files.length - kept.length} missing file(s)`, { series: String(series.title ?? seriesId) });
      out[seriesId] = { ...series, fileEpisodes: kept };
    } else {
      out[seriesId] = series;
    }
  }

  if (removedFiles || removedSeries) {
    logger.info('folder', `Reconcile complete: ${removedFiles} file(s), ${removedSeries} series removed`);
  }
  return out;
}
```

Then add `reconcileMetadata` to the exported `folderHandler` object:

```ts
const folderHandler = {
  async scanFolder(folderPath: string): Promise<ScannedMedia[]> { /* existing */ },
  async scanMultipleFolders(folderPaths: string[]): Promise<ScannedMedia[]> { /* existing */ },
  reconcileMetadata,
};
```

- [ ] **Step 4.2: Add deleteSeriesThumbnails in thumbnailHandler.ts**

Modify `src/main/handlers/thumbnailHandler.ts`. The existing `generateThumbnailFilename(videoPath, timestamp)` produces an MD5-named `.jpg` file in the thumbnail cache directory (`getThumbnailCachePath()`). To delete a series's thumbnails, delete the file produced for each `(filePath, 120)` pair (120 is the default timestamp; the existing pipeline only uses one timestamp per file).

Add this method to the `thumbnailHandler` object:

```ts
import { unlink } from 'node:fs/promises';

// ... inside thumbnailHandler:
  async deleteSeriesThumbnails(videoPaths: string[]): Promise<void> {
    for (const videoPath of videoPaths) {
      if (!videoPath) continue;
      const filename = generateThumbnailFilename(videoPath, 120);
      const fullPath = join(getThumbnailCachePath(), filename);
      try {
        await unlink(fullPath);
      } catch (err) {
        if ((err as NodeJS.ErrnoException).code !== 'ENOENT') {
          logger.warn('thumbnail', `Failed to delete thumbnail: ${(err as Error).message}`, { file: videoPath });
        }
      }
    }
  },
```

(Top of file: ensure `import { join } from 'node:path';` is present — the existing file already uses `join` inside `getThumbnailCachePath`, so it is.)

- [ ] **Step 4.3: Wire reconcile into the manual scan**

Modify `src/main/main.ts`. Find the `scan-and-fetch-metadata` handler. Right after `const existingMetadata = await metadataHandler.loadMetadata() as Record<string, unknown>;`, insert:

```ts
const reconciled = await folderHandler.reconcileMetadata(existingMetadata);
if (reconciled !== existingMetadata) {
  await metadataHandler.saveMetadata(reconciled);
}
```

Then use `reconciled` in place of `existingMetadata` for the rest of the handler.

- [ ] **Step 4.4: Make reconcile aware of active library roots**

Removing a library root does not delete the files on disk — `existsSync` still returns true. So the simple "exists?" rule from Step 4.1 won't drop them. Reconcile needs a second condition: drop a file if it is **not under any currently-active library root**.

Replace step 4.1's `reconcileMetadata` body with this richer version that takes active roots into account:

```ts
async function reconcileMetadata(
  metadata: Record<string, unknown>,
  activeRoots?: string[],
): Promise<Record<string, unknown>> {
  const out: Record<string, unknown> = {};
  let removedFiles = 0;
  let removedSeries = 0;

  const isUnderActiveRoot = (filePath: string): boolean => {
    if (!activeRoots || activeRoots.length === 0) return true; // no constraint
    return activeRoots.some((root) => filePath === root || filePath.startsWith(root.endsWith('/') ? root : root + '/'));
  };

  for (const [seriesId, raw] of Object.entries(metadata)) {
    const series = raw as SeriesEntry;
    const files = Array.isArray(series.fileEpisodes) ? series.fileEpisodes : [];
    const kept = files.filter((f) => {
      if (!f?.filePath) return false;
      const reachable = isUnderActiveRoot(f.filePath);
      const present = reachable && existsSync(f.filePath);
      if (!present) removedFiles++;
      return present;
    });

    if (kept.length === 0 && files.length > 0) {
      removedSeries++;
      logger.info('folder', `Reconcile: dropping series`, { series: String(series.title ?? seriesId) });
      try {
        await imageCacheHandler.deleteSeriesImages({
          poster: series.poster ?? null,
          banner: series.banner ?? null,
          posterLocal: series.posterLocal ?? null,
          bannerLocal: series.bannerLocal ?? null,
          episodes: series.episodes ?? [],
        });
      } catch (err) {
        logger.warn('image', `Reconcile: image cleanup failed: ${(err as Error).message}`, { series: String(series.title ?? seriesId) });
      }
      try {
        await thumbnailHandler.deleteSeriesThumbnails(files.map((f) => f.filePath).filter(Boolean) as string[]);
      } catch (err) {
        logger.warn('thumbnail', `Reconcile: thumbnail cleanup failed: ${(err as Error).message}`, { series: String(series.title ?? seriesId) });
      }
      continue;
    }

    if (kept.length !== files.length) {
      logger.info('folder', `Reconcile: dropped ${files.length - kept.length} file(s)`, { series: String(series.title ?? seriesId) });
      out[seriesId] = { ...series, fileEpisodes: kept };
    } else {
      out[seriesId] = series;
    }
  }

  if (removedFiles || removedSeries) {
    logger.info('folder', `Reconcile complete: ${removedFiles} file(s), ${removedSeries} series removed`);
  }
  return out;
}
```

Update both call sites in `main.ts` to pass active roots:

```ts
// In remove-folder-source handler:
const activeRoots = await configHandler.getFolderSources();
const reconciled = await folderHandler.reconcileMetadata(meta, activeRoots);

// In scan-and-fetch-metadata, after loading existingMetadata:
const activeRoots = await configHandler.getFolderSources();
const reconciled = await folderHandler.reconcileMetadata(existingMetadata, activeRoots);
```

- [ ] **Step 4.5: Typecheck**

Run: `bun run typecheck`
Expected: no errors.

- [ ] **Step 4.6: Manual smoke**

Run: `bun run dev`. With the app running:
1. Pick a series in your library that has multiple episode files.
2. Move one file out of the library folder (don't delete — move to `/tmp` so you can restore).
3. Open Settings, hit Scan on that folder.
4. Open the Activity drawer — confirm a `folder | Reconcile: dropped 1 file(s)` event with the series in context.
5. Reload the library page (or revisit the series detail) — the moved file's episode should now be marked "Not on disk" or the file count should be one fewer.
6. Move the file back, hit Scan again — confirm the file reappears.
7. In Settings, remove an entire library root that contains some series. Confirm those series disappear from the Library page and that a `Reconcile complete` log line appears.

- [ ] **Step 4.7: Commit**

```bash
git add src/main/handlers/folderHandler.ts src/main/handlers/thumbnailHandler.ts src/main/main.ts
git commit -m "feat: reconcile metadata against disk — paths are truth"
```

---

## Task 5: Status field migration + scan-time defaulting

**Files:**
- Modify: `src/main/handlers/metadataHandler.ts`
- Modify: `src/main/handlers/folderHandler.ts`
- Modify: `src/main/main.ts`

- [ ] **Step 5.1: Migrate on load in metadataHandler**

Modify `src/main/handlers/metadataHandler.ts`. In `loadMetadata`, after the JSON parse and before returning, add a migration pass that ensures every `fileEpisodes[i]` has a `status` field. The expected current shape of `loadMetadata` returns `Record<string, unknown>`; cast minimally to walk the structure:

```ts
async loadMetadata(): Promise<Record<string, unknown>> {
  try {
    const raw = await fs.readFile(metadataPath, 'utf-8');
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    // Migration: ensure every file episode has a status.
    for (const seriesValue of Object.values(parsed)) {
      const series = seriesValue as { fileEpisodes?: unknown[] };
      if (!Array.isArray(series.fileEpisodes)) continue;
      for (const file of series.fileEpisodes) {
        const f = file as { status?: string };
        if (!f.status) f.status = 'ready';
      }
    }
    return parsed;
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === 'ENOENT') return {};
    throw err;
  }
},
```

(The exact error-handling shape may differ in your file — preserve the existing structure and only insert the migration loop.)

- [ ] **Step 5.2: Add status field to VideoFile**

Modify `src/main/handlers/folderHandler.ts`. Extend the `VideoFile` interface (currently lines 7-17):

```ts
export interface VideoFile {
  filename: string;
  filePath: string;
  title: string;
  episodeNumber: number;
  seasonNumber: number | null;
  subtitlePath: string | null;
  subtitlePaths: string[];
  parentFolder: string;
  status: 'ready' | 'verifying' | 'stalled';
  lastProbedAt?: number;
}
```

In whatever function constructs these `VideoFile` objects during scanning (search for `parentFolder:` to locate the construction sites), add `status: 'ready'` (default for files discovered via a full scan — they're already on disk and probed implicitly by virtue of having existed long enough for the user to scan them). Example:

```ts
const file: VideoFile = {
  filename,
  filePath,
  title,
  episodeNumber,
  seasonNumber,
  subtitlePath,
  subtitlePaths,
  parentFolder,
  status: 'ready',
};
```

- [ ] **Step 5.3: Persist status on save in main.ts**

Modify `src/main/main.ts`. The `scan-and-fetch-metadata` handler maps `media.files` into `fileEpisodes` around line 699:

```ts
fileEpisodes: media.files.map(f => ({
  episodeNumber: f.episodeNumber,
  seasonNumber: f.seasonNumber,
  filePath: f.filePath,
  subtitlePath: f.subtitlePath,
  subtitlePaths: f.subtitlePaths,
  filename: f.filename,
  title: f.title,
}))
```

Extend it to include status:

```ts
fileEpisodes: media.files.map(f => ({
  episodeNumber: f.episodeNumber,
  seasonNumber: f.seasonNumber,
  filePath: f.filePath,
  subtitlePath: f.subtitlePath,
  subtitlePaths: f.subtitlePaths,
  filename: f.filename,
  title: f.title,
  status: f.status,
  lastProbedAt: f.lastProbedAt,
}))
```

When merging with existing metadata (the merge step further down in the same handler), preserve any existing `status` from the previous metadata for unchanged files — they may currently be `'verifying'` or `'stalled'` from a prior watcher event. The simplest preserving merge: when a file with the same `filePath` exists in `existingMetadata[seriesId].fileEpisodes`, copy its `status` and `lastProbedAt` onto the new entry. Add this just before the save:

```ts
function preserveStatus(seriesId: string, files: Array<{ filePath: string; status?: string; lastProbedAt?: number }>): typeof files {
  const prior = existingMetadata[seriesId] as { fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }> } | undefined;
  if (!prior?.fileEpisodes) return files;
  const byPath = new Map(prior.fileEpisodes.map((f) => [f.filePath, f]));
  return files.map((f) => {
    const old = byPath.get(f.filePath);
    if (!old) return f;
    return { ...f, status: old.status ?? f.status, lastProbedAt: old.lastProbedAt ?? f.lastProbedAt };
  });
}
```

Then where the merged series is assembled, run `fileEpisodes` through `preserveStatus(seriesId, fileEpisodes)` before assigning.

- [ ] **Step 5.4: Typecheck**

Run: `bun run typecheck`
Expected: no errors.

- [ ] **Step 5.5: Manual smoke**

Run: `bun run dev`. Trigger a scan on a folder. After it completes, inspect the metadata file at the path printed by `app.getPath('userData') + '/metadata.json'` (typically `~/.config/AniBeam/metadata.json` on Linux). Open it and verify every entry inside `fileEpisodes` has `"status": "ready"`.

- [ ] **Step 5.6: Commit**

```bash
git add src/main/handlers/metadataHandler.ts src/main/handlers/folderHandler.ts src/main/main.ts
git commit -m "feat: add status field to file episodes with migration"
```

---

## Task 6: Video probe handler + queue

**Files:**
- Create: `src/main/handlers/videoProbeHandler.ts`
- Create: `scripts/verify-probe-parser.mjs`
- Modify: `src/main/main.ts`

- [ ] **Step 6.1: Create the probe handler**

Create `src/main/handlers/videoProbeHandler.ts`:

```ts
import { spawn } from 'node:child_process';
import { stat } from 'node:fs/promises';
import { logger } from '../services/logger.js';

export interface ProbeResult {
  ready: boolean;
  reason?: string;
}

interface QueuedFile {
  path: string;
  attempts: number;
  enqueuedAt: number;
  nextRunAt: number;
  lastSize: number;
  lastMtimeMs: number;
  stableSinceMs: number;
}

const BACKOFFS_MS = [5_000, 15_000, 30_000];
const STEADY_BACKOFF_MS = 60_000;
const MAX_LIFETIME_MS = 30 * 60_000;
const SIZE_STABLE_THRESHOLD_MS = 10_000;
const POLL_INTERVAL_MS = 2_000;

const queue = new Map<string, QueuedFile>();
let pollHandle: NodeJS.Timeout | null = null;
let onStatusChange: ((path: string, status: 'ready' | 'verifying' | 'stalled') => Promise<void> | void) | null = null;

export function parseFfprobeJson(stdout: string): ProbeResult {
  let parsed: { streams?: Array<{ codec_type?: string }>; format?: { duration?: string | number } };
  try {
    parsed = JSON.parse(stdout);
  } catch {
    return { ready: false, reason: 'invalid ffprobe output' };
  }
  const hasVideoStream = Array.isArray(parsed.streams) && parsed.streams.some((s) => s.codec_type === 'video');
  if (!hasVideoStream) return { ready: false, reason: 'no video stream' };
  const dur = parsed.format?.duration;
  const durNum = typeof dur === 'string' ? Number(dur) : (dur ?? 0);
  if (!Number.isFinite(durNum) || durNum <= 0) return { ready: false, reason: 'no duration' };
  return { ready: true };
}

function runFfprobe(path: string, timeoutMs = 15_000): Promise<ProbeResult> {
  return new Promise((resolve) => {
    const child = spawn('ffprobe', ['-v', 'error', '-show_streams', '-show_format', '-of', 'json', path], {
      stdio: ['ignore', 'pipe', 'ignore'],
    });
    let stdout = '';
    const timer = setTimeout(() => {
      child.kill();
      resolve({ ready: false, reason: 'ffprobe timeout' });
    }, timeoutMs);
    child.stdout.on('data', (buf) => { stdout += buf.toString(); });
    child.on('error', (err) => {
      clearTimeout(timer);
      resolve({ ready: false, reason: `ffprobe spawn error: ${err.message}` });
    });
    child.on('close', (code) => {
      clearTimeout(timer);
      if (code !== 0) {
        resolve({ ready: false, reason: `ffprobe exit ${code}` });
        return;
      }
      resolve(parseFfprobeJson(stdout));
    });
  });
}

function nextDelay(attempts: number): number {
  if (attempts < BACKOFFS_MS.length) return BACKOFFS_MS[attempts];
  return STEADY_BACKOFF_MS;
}

async function probeOne(path: string): Promise<void> {
  const entry = queue.get(path);
  if (!entry) return;
  entry.attempts++;
  const result = await runFfprobe(path);
  if (result.ready) {
    queue.delete(path);
    logger.info('probe', `Ready`, { file: path });
    if (onStatusChange) await onStatusChange(path, 'ready');
    return;
  }
  if (Date.now() - entry.enqueuedAt > MAX_LIFETIME_MS) {
    queue.delete(path);
    logger.warn('probe', `Stalled (${result.reason ?? 'unknown'})`, { file: path });
    if (onStatusChange) await onStatusChange(path, 'stalled');
    return;
  }
  entry.nextRunAt = Date.now() + nextDelay(entry.attempts);
}

async function tick(): Promise<void> {
  const now = Date.now();
  for (const [path, entry] of queue) {
    // size-stable check
    try {
      const s = await stat(path);
      const sizeChanged = s.size !== entry.lastSize;
      const mtimeChanged = s.mtimeMs !== entry.lastMtimeMs;
      if (sizeChanged || mtimeChanged) {
        entry.lastSize = s.size;
        entry.lastMtimeMs = s.mtimeMs;
        entry.stableSinceMs = now;
      } else if (now - entry.stableSinceMs >= SIZE_STABLE_THRESHOLD_MS) {
        // stable — probe immediately if we haven't already this tick
        await probeOne(path);
        continue;
      }
    } catch {
      // file might be temporarily inaccessible; let backoff handle it
    }
    if (now >= entry.nextRunAt) {
      await probeOne(path);
    }
  }
}

export const videoProbeHandler = {
  /**
   * Single-shot probe. Used by the queue and by manual callers.
   */
  probe(path: string): Promise<ProbeResult> {
    return runFfprobe(path);
  },

  /**
   * Enqueue a file for verification with backoff.
   */
  enqueue(path: string): void {
    if (queue.has(path)) return;
    const now = Date.now();
    queue.set(path, {
      path,
      attempts: 0,
      enqueuedAt: now,
      nextRunAt: now, // probe immediately on next tick
      lastSize: -1,
      lastMtimeMs: -1,
      stableSinceMs: now,
    });
    logger.info('probe', `Verifying`, { file: path });
  },

  /**
   * Re-enqueue a file (clears prior history).
   */
  retry(path: string): void {
    queue.delete(path);
    this.enqueue(path);
  },

  start(handler: (path: string, status: 'ready' | 'verifying' | 'stalled') => Promise<void> | void): void {
    onStatusChange = handler;
    if (pollHandle) return;
    pollHandle = setInterval(() => { void tick(); }, POLL_INTERVAL_MS);
  },

  stop(): void {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
    queue.clear();
  },
};
```

- [ ] **Step 6.2: Verify the parser logic**

Create `scripts/verify-probe-parser.mjs`:

```js
import assert from 'node:assert/strict';
import { Module } from 'node:module';

// stub electron + logger imports
const originalLoad = Module._load;
Module._load = function (request, parent, ...rest) {
  if (request === 'electron') return { BrowserWindow: { getAllWindows: () => [] } };
  return originalLoad.call(this, request, parent, ...rest);
};

const { parseFfprobeJson } = await import('../src/main/handlers/videoProbeHandler.ts');

assert.deepEqual(
  parseFfprobeJson(JSON.stringify({ streams: [{ codec_type: 'video' }], format: { duration: '1234.5' } })),
  { ready: true },
);
assert.deepEqual(
  parseFfprobeJson(JSON.stringify({ streams: [{ codec_type: 'audio' }], format: { duration: '10' } })),
  { ready: false, reason: 'no video stream' },
);
assert.deepEqual(
  parseFfprobeJson(JSON.stringify({ streams: [{ codec_type: 'video' }], format: { duration: '0' } })),
  { ready: false, reason: 'no duration' },
);
assert.deepEqual(
  parseFfprobeJson('not json'),
  { ready: false, reason: 'invalid ffprobe output' },
);
console.log('OK: ffprobe parser');
```

Run: `bun --bun scripts/verify-probe-parser.mjs`
Expected: `OK: ffprobe parser`.

- [ ] **Step 6.3: Wire the probe handler into main.ts**

Modify `src/main/main.ts`. Add at the top:

```ts
import { videoProbeHandler } from './handlers/videoProbeHandler.js';
```

Define a small helper that updates a single file's status in the persisted metadata (place near the other helpers in main.ts):

```ts
async function updateFileStatus(filePath: string, status: 'ready' | 'verifying' | 'stalled'): Promise<void> {
  const meta = (await metadataHandler.loadMetadata()) as Record<string, unknown>;
  let touched = false;
  for (const series of Object.values(meta)) {
    const s = series as { fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }> };
    if (!Array.isArray(s.fileEpisodes)) continue;
    for (const file of s.fileEpisodes) {
      if (file.filePath === filePath) {
        file.status = status;
        file.lastProbedAt = Date.now();
        touched = true;
      }
    }
  }
  if (touched) {
    await metadataHandler.saveMetadata(meta);
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath, status });
    }
  }
}
```

Start the probe queue inside `app.whenReady()` (or in `createWindow`, just after the window is created):

```ts
videoProbeHandler.start(updateFileStatus);
```

Register the manual retry channel near the other `ipcMain.handle` calls:

```ts
ipcMain.handle('probe:retry', (_event, filePath: string) => {
  if (typeof filePath === 'string' && filePath.length > 0) {
    videoProbeHandler.retry(filePath);
  }
});
```

Add an `app.on('before-quit', ...)` handler near the bottom of the file (it doesn't currently exist):

```ts
app.on('before-quit', () => {
  videoProbeHandler.stop();
});
```

- [ ] **Step 6.4: Expose status-change channel in preload (optional optimization)**

Modify `src/main/preload.ts`. Add to the `ElectronAPI` interface:

```ts
  onMetadataFileStatusChanged: (handler: (payload: { filePath: string; status: 'ready' | 'verifying' | 'stalled' }) => void) => () => void;
```

Add to the `contextBridge.exposeInMainWorld` body:

```ts
  onMetadataFileStatusChanged: (handler) => {
    const listener = (_e: unknown, payload: { filePath: string; status: 'ready' | 'verifying' | 'stalled' }) => handler(payload);
    ipcRenderer.on('metadata:file-status-changed', listener);
    return () => ipcRenderer.removeListener('metadata:file-status-changed', listener);
  },
```

- [ ] **Step 6.5: Typecheck**

Run: `bun run typecheck`
Expected: no errors.

- [ ] **Step 6.6: Manual smoke**

Run: `bun run dev`. The Activity drawer should not show a probe storm — `videoProbeHandler.start` is a no-op until you `enqueue`. We test full integration in Task 8 once the watcher exists.

For now, confirm the queue can be poked manually. In a separate terminal, in the project root:

```bash
node -e "process.exit(0)"  # placeholder; nothing to test yet without watcher
```

(Skip; full smoke is part of Task 8.)

- [ ] **Step 6.7: Commit**

```bash
git add src/main/handlers/videoProbeHandler.ts src/main/main.ts src/main/preload.ts scripts/verify-probe-parser.mjs
git commit -m "feat: add ffprobe-based video probe handler with retry queue"
```

---

## Task 7: Status visuals in EpisodeCard / ShowCard + retry button

**Files:**
- Modify: `src/renderer/components/EpisodeCard.tsx`
- Modify: `src/renderer/components/ShowCard.tsx`
- Modify: `src/renderer/styles/App.css`

The renderer needs to know each file's status. The `EpisodeMetadata` type already merges file info onto the episode (the existing component reads `episode.thumbnailLocal`, etc.). Add `status` and `filePath` to whatever interface backs the `episode` prop. Easier path: read the file entry alongside the episode in the parent component. For this plan we extend `EpisodeMetadata`.

- [ ] **Step 7.1: Extend EpisodeMetadata in the renderer types**

Locate where `EpisodeMetadata` is declared (likely `src/renderer/types.ts` or inline in `MetadataTab.tsx` / `SeriesDetailPage.tsx`). Run:

```bash
rg -n "interface EpisodeMetadata|type EpisodeMetadata" src/renderer
```

Add to that interface:

```ts
status?: 'ready' | 'verifying' | 'stalled';
filePath?: string;
lastProbedAt?: number;
```

In whatever code merges the file episode into the episode object that gets passed to `<EpisodeCard>` (find it via `rg -n "fileEpisodes" src/renderer`), copy these fields across:

```ts
const episodeWithFile = {
  ...episode,
  status: matchingFile?.status,
  filePath: matchingFile?.filePath,
  lastProbedAt: matchingFile?.lastProbedAt,
};
```

- [ ] **Step 7.2: Update EpisodeCard to render badge + dim**

Modify `src/renderer/components/EpisodeCard.tsx`. Replace the existing JSX root (currently lines 51-82) with:

```tsx
const status = (episode as { status?: 'ready' | 'verifying' | 'stalled' }).status ?? 'ready';
const filePath = (episode as { filePath?: string }).filePath;
const isReady = status === 'ready';

const handleRetry = (e: React.MouseEvent) => {
  e.stopPropagation();
  if (filePath) window.electronAPI.probeRetry(filePath);
};

return (
  <button
    className={`episode-card ${hasFile ? 'has-file' : 'no-file'}${isReady ? '' : ' not-ready'} status-${status}`}
    onClick={handleClick}
    disabled={!hasFile || !isReady}
  >
    <div className="episode-thumb">
      {thumbnailUrl ? (
        <img
          src={thumbnailUrl}
          alt={episode.title || `Episode ${episode.episodeNumber}`}
          loading="lazy"
          onError={(e) => {
            const target = e.target as HTMLImageElement;
            if (episode.thumbnail && target.src !== episode.thumbnail) {
              target.src = episode.thumbnail;
            } else {
              target.style.display = 'none';
            }
          }}
        />
      ) : (
        <span className="episode-thumb-number">{formatEpisodeNumber(episode.episodeNumber).padStart(2, '0')}</span>
      )}
      {hasFile && isReady && (
        <div className="episode-play-icon">
          <Play size={20} />
        </div>
      )}
      {!isReady && (
        <span
          className={`status-badge status-badge-${status}`}
          onClick={status === 'stalled' ? handleRetry : undefined}
          role={status === 'stalled' ? 'button' : undefined}
        >
          {status === 'verifying' ? 'VERIFYING' : 'STALLED · RETRY'}
        </span>
      )}
    </div>
    <div className="episode-info">
      <div className="episode-row-top">
        <span className="episode-code">{code}</span>
        {!hasFile && <span className="episode-na">Not on disk</span>}
      </div>
      <div className="episode-title">
        {episode.title || `Episode ${episode.episodeNumber}`}
      </div>
      {episode.airDate && (
        <div className="episode-date">
          {new Date(episode.airDate).toLocaleDateString()}
        </div>
      )}
    </div>
  </button>
);
```

Add a `React` import for the event type if not already present:

```ts
import type { MouseEvent } from 'react';
```

(And reference the event as `MouseEvent<HTMLSpanElement>` in the handler if you prefer strict typing — `React.MouseEvent` works too if `React` is imported as a default.)

- [ ] **Step 7.3: Update ShowCard to reflect "least ready" status**

Modify `src/renderer/components/ShowCard.tsx`. Inside the component body, after `const downloadedEpisodes = ...`, compute:

```ts
type FileStatus = 'ready' | 'verifying' | 'stalled';
const files = (seriesData.fileEpisodes ?? []) as Array<{ status?: FileStatus }>;
const order: Record<FileStatus, number> = { ready: 0, verifying: 1, stalled: 2 };
const aggregateStatus: FileStatus = files.reduce<FileStatus>((acc, f) => {
  const s = (f.status ?? 'ready') as FileStatus;
  return order[s] > order[acc] ? s : acc;
}, 'ready');
const isReady = aggregateStatus === 'ready';
```

Then add to the root `<button>`:

```tsx
<button className={`show-card${isReady ? '' : ' not-ready'} status-${aggregateStatus}`} onClick={handleClick}>
```

And inside `.show-card-poster-wrap`, add a badge after the existing badge block (or alongside):

```tsx
{!isReady && (
  <div className={`status-badge status-badge-${aggregateStatus}`}>
    {aggregateStatus === 'verifying' ? 'VERIFYING' : 'STALLED'}
  </div>
)}
```

(No retry click on the show-card — the retry lives on the per-episode card.)

- [ ] **Step 7.4: Add status badge styles**

Append to `src/renderer/styles/App.css`:

```css
/* Status: not-ready treatment */
.episode-card.not-ready,
.show-card.not-ready {
  opacity: 0.6;
}
.episode-card.not-ready { cursor: not-allowed; }

.status-badge {
  position: absolute;
  top: 8px;
  left: 8px;
  padding: 4px 8px;
  background: rgba(20, 20, 26, 0.92);
  color: #d8d8e0;
  border: 1px solid #2a2a32;
  border-radius: 8px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 10.5px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}
.status-badge-verifying { color: #c9c9d0; }
.status-badge-stalled {
  color: #e08989;
  border-color: #b54343;
  cursor: pointer;
}
```

- [ ] **Step 7.5: Optional — listen for status change events to refresh metadata**

In whatever component owns the metadata state (likely `HomePage.tsx` or a metadata context/hook), subscribe to the optional channel from Task 6.4 to trigger a re-fetch:

```tsx
useEffect(() => {
  const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
    void reloadMetadata();
  });
  return () => unsubscribe?.();
}, [reloadMetadata]);
```

If `onMetadataFileStatusChanged` is not present (the optional channel may have been skipped), the page will still refresh on the next manual reload — this just makes it instant.

- [ ] **Step 7.6: Typecheck and manual smoke**

Run: `bun run typecheck`
Expected: no errors.

Run: `bun run dev`. Manually edit `~/.config/AniBeam/metadata.json`: pick one file episode and change its `"status"` from `"ready"` to `"verifying"`. Restart the app. The corresponding episode card should be dimmed with a `VERIFYING` badge. Change it to `"stalled"` and restart — the badge should read `STALLED · RETRY`. Click it — confirm a `probe | Verifying` event appears in the activity drawer (the retry handler will run, and since the file likely passes ffprobe, you'll see a `Ready` event shortly after; the metadata file should flip back to `"ready"`).

- [ ] **Step 7.7: Commit**

```bash
git add src/renderer/components/EpisodeCard.tsx src/renderer/components/ShowCard.tsx src/renderer/styles/App.css src/renderer/types.ts
git commit -m "feat: render status badges and dim treatment for non-ready files"
```

(If `EpisodeMetadata` lives somewhere other than `src/renderer/types.ts`, adjust the path in the `git add` command.)

---

## Task 8: Always-on file watcher

**Files:**
- Modify: `package.json` (add chokidar)
- Create: `src/main/services/watcher.ts`
- Modify: `src/main/handlers/folderHandler.ts` (add `scanSingleFile`)
- Modify: `src/main/main.ts` (boot/restart/stop watcher; wire single-file scan and unlink reconcile)

- [ ] **Step 8.1: Add chokidar dependency**

Run: `bun add chokidar`
Expected: `chokidar` added to `dependencies` in `package.json`. Verify with `cat package.json | grep chokidar`.

- [ ] **Step 8.2: Create the watcher service**

Create `src/main/services/watcher.ts`:

```ts
import chokidar, { type FSWatcher } from 'chokidar';
import { logger } from './logger.js';

export interface WatcherCallbacks {
  onAdd: (filePath: string) => void;
  onUnlink: (filePath: string) => void;
  onUnlinkDir: (dirPath: string) => void;
}

const VIDEO_EXTS = new Set(['.mkv', '.mp4', '.avi', '.mov', '.webm', '.m4v', '.ts']);
const IGNORED_PATTERNS = [/(^|[\/\\])\../, /\.part$/i, /\.crdownload$/i, /\.tmp$/i];

let watcher: FSWatcher | null = null;
let activeRoots: string[] = [];
let callbacks: WatcherCallbacks | null = null;
const debounceMap = new Map<string, NodeJS.Timeout>();
const DEBOUNCE_MS = 1000;

function isVideo(path: string): boolean {
  const dot = path.lastIndexOf('.');
  if (dot < 0) return false;
  return VIDEO_EXTS.has(path.slice(dot).toLowerCase());
}

function debounce(key: string, fn: () => void): void {
  const existing = debounceMap.get(key);
  if (existing) clearTimeout(existing);
  debounceMap.set(key, setTimeout(() => {
    debounceMap.delete(key);
    fn();
  }, DEBOUNCE_MS));
}

function attach(w: FSWatcher): void {
  w.on('add', (path) => {
    if (!isVideo(path)) return;
    debounce(`add:${path}`, () => {
      logger.info('watch', `New file`, { file: path });
      callbacks?.onAdd(path);
    });
  });
  w.on('unlink', (path) => {
    if (!isVideo(path)) return;
    debounce(`unlink:${path}`, () => {
      logger.info('watch', `Removed file`, { file: path });
      callbacks?.onUnlink(path);
    });
  });
  w.on('unlinkDir', (path) => {
    debounce(`unlinkDir:${path}`, () => {
      logger.info('watch', `Removed directory`, { file: path });
      callbacks?.onUnlinkDir(path);
    });
  });
  w.on('error', (err) => {
    logger.error('watch', `Watcher error: ${(err as Error).message}`);
  });
  w.on('ready', () => {
    logger.info('watch', `Watching ${activeRoots.length} root(s)`);
  });
}

export const fileWatcher = {
  async start(roots: string[], cb: WatcherCallbacks): Promise<void> {
    callbacks = cb;
    activeRoots = roots.slice();
    if (roots.length === 0) {
      logger.info('watch', 'No library roots — watcher idle');
      return;
    }
    watcher = chokidar.watch(roots, {
      ignored: IGNORED_PATTERNS,
      ignoreInitial: true, // existing files are handled by full scans
      persistent: true,
      awaitWriteFinish: { stabilityThreshold: 2000, pollInterval: 500 },
    });
    attach(watcher);
  },

  async restart(roots: string[]): Promise<void> {
    await this.stop();
    if (callbacks) await this.start(roots, callbacks);
  },

  async stop(): Promise<void> {
    for (const t of debounceMap.values()) clearTimeout(t);
    debounceMap.clear();
    if (watcher) {
      await watcher.close();
      watcher = null;
    }
    activeRoots = [];
  },
};
```

- [ ] **Step 8.3: Add scanSingleFile to folderHandler**

Modify `src/main/handlers/folderHandler.ts`. The existing `scanFolder` walks a directory and runs the same per-file inference logic on each video it encounters. The single-file path needs the same inference for one path.

Inside `folderHandler.ts`, find the helper(s) used by `scanDirectory` to construct a `VideoFile` from a single path (likely `inferFromFilename`, `extractEpisodeNumber`, etc.). Wrap them into a public method:

```ts
async scanSingleFile(filePath: string): Promise<{ media: ScannedMedia; file: VideoFile } | null> {
  if (!existsSync(filePath)) return null;
  // Use the same inference logic that scanDirectory uses.
  // The exact internal call depends on the existing helpers; the result
  // must be a fully-populated VideoFile and the parent ScannedMedia
  // (series id derived from the parent folder name, same rule as scanDirectory).
  const parentDir = dirname(filePath);
  const directoryResults = await scanDirectory(parentDir);
  // Find the entry that contains this file
  for (const media of directoryResults) {
    const file = media.files.find((f) => f.filePath === filePath);
    if (file) {
      file.status = 'verifying';
      return { media, file };
    }
  }
  return null;
},
```

(Top of file: ensure `import { dirname } from 'node:path';` is present.)

This is a lightweight reuse — `scanDirectory(parentDir)` is cheap for a single directory and avoids duplicating the inference logic. If profiling later shows this is too slow, extract a single-file inference helper from `scanDirectory`.

Add `scanSingleFile` to the exported `folderHandler` object.

- [ ] **Step 8.4a: Refactor — extract `processOneMedia` from `scan-and-fetch-metadata`**

Before the watcher can ingest a single file, the per-series logic inside the existing `scan-and-fetch-metadata` handler needs to be reusable. This step is a pure mechanical extraction — no behavior change.

Open `src/main/main.ts` and locate the `scan-and-fetch-metadata` IPC handler (starts ~line 413). Inside it, find the per-series loop. The loop body — everything that runs once per `media` of `scannedMedia` — currently does:
1. Cache validation (does an entry already exist with a matching title?)
2. Metadata fetch (MAL → AniList fallback)
3. Image collection + `imageCacheHandler.cacheImages(...)`
4. Per-episode thumbnail generation
5. Building the merged series object that is later assigned into `newMetadata[seriesId]`

Lift that body verbatim into a top-level async function defined above the handler:

```ts
async function processOneMedia(
  media: ScannedMedia,
  existingMetadata: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  // Paste the per-series loop body here, verbatim.
  // The function returns a `{ [seriesId]: mergedSeries }` slice (one key).
  // It must NOT mutate `existingMetadata` — read-only.
  // ...
}
```

Then in the original handler, replace the loop body with:

```ts
for (const media of scannedMedia) {
  const slice = await processOneMedia(media, existingMetadata);
  Object.assign(newMetadata, slice);
}
```

(Or merge by-key if there's deeper merging logic.)

**Verify the refactor before moving on:**

Run: `bun run typecheck`. Expected: no errors.

Run: `bun run dev`. Trigger a manual scan via Settings → Scan. Confirm the resulting library is identical to before the refactor (same series count, same episode counts, same metadata). The activity drawer events from Task 3 give you the live trace.

Commit the refactor on its own:

```bash
git add src/main/main.ts
git commit -m "refactor: extract processOneMedia from scan-and-fetch-metadata"
```

- [ ] **Step 8.4b: Wire watcher boot, single-file scan, and unlink reconcile in main.ts**

Modify `src/main/main.ts`. Add at the top:

```ts
import { fileWatcher } from './services/watcher.js';
```

Add a helper that runs the single-file pipeline (scan → metadata fetch → image cache → thumbnail → save → enqueue probe). Place it near `updateFileStatus`:

```ts
async function ingestSingleFile(filePath: string): Promise<void> {
  try {
    const result = await folderHandler.scanSingleFile(filePath);
    if (!result) {
      logger.warn('watch', `scanSingleFile returned null`, { file: filePath });
      return;
    }
    const { media, file } = result;
    logger.info('metadata', `Fetching for new file`, { series: media.name, file: filePath });

    const existing = (await metadataHandler.loadMetadata()) as Record<string, unknown>;
    const slice = await processOneMedia(media, existing);

    // Force this newly-discovered file to 'verifying' in the merged slice.
    for (const seriesValue of Object.values(slice)) {
      const s = seriesValue as { fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }> };
      if (!Array.isArray(s.fileEpisodes)) continue;
      for (const f of s.fileEpisodes) {
        if (f.filePath === filePath) {
          f.status = 'verifying';
          f.lastProbedAt = Date.now();
        }
      }
    }

    const merged = { ...existing, ...slice };
    await metadataHandler.saveMetadata(merged);

    videoProbeHandler.enqueue(filePath);

    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath, status: 'verifying' });
    }
  } catch (err) {
    logger.error('watch', `Failed to ingest new file: ${(err as Error).message}`, { file: filePath });
  }
}
```

For the unlink path:

```ts
async function handleUnlink(filePath: string): Promise<void> {
  const meta = (await metadataHandler.loadMetadata()) as Record<string, unknown>;
  const activeRoots = await configHandler.getFolderSources();
  const reconciled = await folderHandler.reconcileMetadata(meta, activeRoots);
  if (reconciled !== meta) await metadataHandler.saveMetadata(reconciled);
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send('metadata:file-status-changed', { filePath, status: 'ready' /* placeholder; actually removed */ });
  }
}
```

Boot the watcher inside `app.whenReady()` after windows are created and config is loaded:

```ts
const initialRoots = await configHandler.getFolderSources();
await fileWatcher.start(initialRoots, {
  onAdd: (path) => { void ingestSingleFile(path); },
  onUnlink: (path) => { void handleUnlink(path); },
  onUnlinkDir: () => { void handleUnlink('<dir>'); }, // a unlinkDir triggers full reconcile via existsSync misses
});
```

Restart the watcher whenever the library list changes:

```ts
ipcMain.handle('add-folder-source', async (_event, folderPath: string) => {
  try {
    const ok = await configHandler.addFolderSource(folderPath);
    if (ok) {
      const roots = await configHandler.getFolderSources();
      await fileWatcher.restart(roots);
      logger.info('folder', `Added library root: ${folderPath}`);
    }
    return ok;
  } catch (error) {
    logger.error('folder', `Error adding folder source: ${(error as Error).message}`);
    throw error;
  }
});
```

And update the existing `remove-folder-source` handler (already modified in Task 4) to also restart the watcher after the reconcile:

```ts
ipcMain.handle('remove-folder-source', async (_event, folderPath: string) => {
  try {
    const ok = await configHandler.removeFolderSource(folderPath);
    if (ok) {
      logger.info('folder', `Removed library root: ${folderPath}`);
      const meta = (await metadataHandler.loadMetadata()) as Record<string, unknown>;
      const activeRoots = await configHandler.getFolderSources();
      const reconciled = await folderHandler.reconcileMetadata(meta, activeRoots);
      if (reconciled !== meta) await metadataHandler.saveMetadata(reconciled);
      await fileWatcher.restart(activeRoots);
    }
    return ok;
  } catch (error) {
    logger.error('folder', `Error removing folder source: ${(error as Error).message}`);
    throw error;
  }
});
```

Stop the watcher on shutdown — extend the `before-quit` handler from Task 6:

```ts
app.on('before-quit', () => {
  videoProbeHandler.stop();
  void fileWatcher.stop();
});
```

- [ ] **Step 8.5: Typecheck**

Run: `bun run typecheck`
Expected: no errors.

- [ ] **Step 8.6: Manual smoke (the big one)**

Run: `bun run dev`. With the app running:

1. Open the Activity drawer. Confirm `watch | Watching N root(s)` appears.
2. **Add file:** copy a video file (e.g. an existing `.mkv` from another folder) into one of your library folders.
   - Within ~3 seconds, expect `watch | New file` → `metadata | Fetching for new file` → eventually `probe | Verifying` and (because the file already exists fully) `probe | Ready` shortly after.
   - The new episode/series should appear in the Library, initially dimmed with `VERIFYING`, then becoming clickable.
3. **Mid-download simulation:** create a fake "downloading" file:
   ```bash
   dd if=/dev/zero of=/path/to/library/fake-show/Episode01.mkv bs=1M count=5 status=none
   sleep 2
   dd if=/dev/zero of=/path/to/library/fake-show/Episode01.mkv bs=1M count=200 status=none
   ```
   The first `dd` creates a tiny file (probe will fail — no streams). The second appends size after a delay. Expect: `watch | New file` → `metadata | Fetching` (likely fails for "fake-show" — that's fine), then repeated `probe | (ffprobe exit 1)` until the file stabilizes, then a successful `probe | Ready` (since we wrote no real video data, this will actually go to `STALLED` after 30 minutes — that's expected for invalid content; replace with a real partial copy of an actual video to see it transition to ready).
4. **Delete file:** `rm` a file from a library folder. Expect `watch | Removed file` → `folder | Reconcile: dropped 1 file(s)`. The episode should disappear from the UI.
5. **Remove a library root** in Settings — confirm watcher restarts (look for a `watch | Watching N root(s)` line with the new count).

- [ ] **Step 8.7: Commit**

```bash
git add package.json bun.lockb src/main/services/watcher.ts src/main/handlers/folderHandler.ts src/main/main.ts
git commit -m "feat: always-on chokidar watcher with single-file ingest and probe enqueue"
```

(If the lockfile is `package-lock.json` instead of `bun.lockb`, adjust the `git add` accordingly.)

---

## Task 9: Drop the static scan-progress text in Settings

**Files:**
- Modify: `src/renderer/components/SettingsTab.tsx`

- [ ] **Step 9.1: Remove the scan-progress block**

Modify `src/renderer/components/SettingsTab.tsx`. Delete the block at lines 309-314:

```tsx
{scanProgress && (
  <div className="scan-progress">
    {scanning && <div className="loading-spinner small" />}
    <span>{scanProgress}</span>
  </div>
)}
```

Also remove the now-unused `scanProgress` state and its setter (`setScanProgress(...)` calls) throughout the file. The Activity drawer is the source of truth for live feedback now. Keep `scanning` and `scanningPath` if they are used to disable the Scan button while a scan is in progress (they are — leave them).

The `handleScanFolder` and `handleScanAll` functions still call `await window.electronAPI.scanAndFetchMetadata(...)` — those calls remain unchanged. Just remove every line that touches `scanProgress` / `setScanProgress`. The `setTimeout(() => setScanProgress(''), 3000)` calls go away with the rest.

- [ ] **Step 9.2: Typecheck and manual smoke**

Run: `bun run typecheck`
Expected: no errors related to the removal. (If `scanProgress` was imported anywhere else, fix or remove those references.)

Run: `bun run dev`. Open Settings, hit Scan. The static text block should be gone. The Scan button should still disable while scanning. The Activity drawer (always visible toggle) shows the live feed.

- [ ] **Step 9.3: Commit**

```bash
git add src/renderer/components/SettingsTab.tsx
git commit -m "refactor: drop static scan-progress text in favor of activity drawer"
```

---

## Verification checklist (run before declaring done)

- [ ] `bun run typecheck` passes
- [ ] `bun --bun scripts/verify-logger.mjs` prints `OK: logger ring buffer behaves`
- [ ] `bun --bun scripts/verify-probe-parser.mjs` prints `OK: ffprobe parser`
- [ ] `bun run dev` starts without errors; Activity pill is visible bottom-right; drawer opens to show events
- [ ] Manual scan produces events across `folder` / `metadata` / `image` / `thumbnail` stages in the drawer
- [ ] Moving a file out of a library folder + scanning removes that file's entry; moving it back + scanning restores it
- [ ] Removing a library root in Settings drops every series rooted there (and clears their cached images / thumbnails)
- [ ] Dropping a new video into a library folder while the app runs produces a `watch | New file` event, dims its card with `VERIFYING`, and resolves to `READY` once `ffprobe` succeeds
- [ ] Deleting a video from disk while the app runs produces a `watch | Removed file` event and removes the card
- [ ] Restarting the app does NOT re-probe existing files (no `probe | Verifying` storm)
- [ ] Clicking `STALLED · RETRY` on a stalled episode card fires a `probe | Verifying` event for that file

---

## Spec coverage check

| Spec section | Implemented in |
|--------------|----------------|
| 1. Activity log (drawer) | Tasks 1, 2, 3 |
| 2. Cache invalidation — paths are truth | Task 4 |
| 3. Always-on file watcher | Task 8 |
| 4. Dry-run probe | Tasks 5, 6 |
| 5. Data model changes (`status`, `lastProbedAt`, migration) | Task 5 |
| 6. UI status visuals | Task 7 |
| Drop static scanProgress text | Task 9 |
