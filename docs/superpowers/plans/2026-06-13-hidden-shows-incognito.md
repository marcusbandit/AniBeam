# Hidden Shows (Incognito) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a per-series "hidden" (incognito) flag — hidden series never sync to AniList/MAL and are absent from every list page unless a session-only "Show hidden shows" toggle is on (always boots OFF).

**Architecture:** A `hidden` boolean persists in `metadata.json`, surfaced onto `LibraryItem` so every list page sees it for free. External tracker pushes are blocked at the single main-process choke point (the three `tracker:*` IPC handlers) via a pure shared matcher. A session-only React context (`useHiddenShows`) drives reveal across pages; HomePage additionally grows a dedicated "Hidden" tab when revealed. Hidden cards render dimmed with a "Hidden" badge.

**Tech Stack:** Electron + Vite, React (renderer), TypeScript, Bun (scripts/tests). No renderer test framework — UI tasks verify via `bun run typecheck` + manual smoke; the one pure unit (the matcher) gets a `scripts/verify-*.mjs` test.

**Key decisions (from spec):** external-tracker-only incognito (local view history keeps recording); reveal = subtle badge + dim; HomePage gets a dedicated Hidden tab (hidden series live only there, not mixed into All/Series/Movies); Feed/Watching/Metadata show revealed hidden series inline (badged+dim); Subscriptions is RSS-feed config (no series) → out of scope.

---

### Task 1: Data model — persist `hidden` and surface it on `LibraryItem`

**Files:**
- Modify: `src/renderer/hooks/useMetadata.ts` (SeriesMetadata interface, ~L41)
- Modify: `src/main/preload.ts` (LibraryItem interface, ~L80)
- Modify: `src/main/ipc/folder.ts` (`library:walk` map, `stored` type ~L48-66 and returned object ~L82-96)

- [ ] **Step 1: Add `hidden` to `SeriesMetadata`**

In `src/renderer/hooks/useMetadata.ts`, inside `interface SeriesMetadata`, add right before `animationStudio` (keep the catch-all index last):

```ts
  /** Incognito flag. When true, the series never syncs to external trackers
   *  and is hidden from all list pages unless the session "Show hidden" toggle
   *  is on. Absent / false = visible. */
  hidden?: boolean;
```

- [ ] **Step 2: Add `hidden` to `LibraryItem`**

In `src/main/preload.ts`, inside `export interface LibraryItem`, add after `malId: number | null;` (L80):

```ts
  /** Incognito flag mirrored from metadata.json so every list page can filter
   *  without a separate metadata fetch. */
  hidden: boolean;
```

- [ ] **Step 3: Surface `hidden` in the `library:walk` map**

In `src/main/ipc/folder.ts`, add to the `stored` destructure type (after `malId?: number | null;`, ~L62):

```ts
        hidden?: boolean;
```

and in the returned object literal, after `malId: stored.malId ?? null,` (~L94):

```ts
        hidden: stored.hidden ?? false,
```

- [ ] **Step 4: Typecheck**

Run: `bun run typecheck`
Expected: PASS (no errors).

- [ ] **Step 5: Commit**

```bash
git add src/renderer/hooks/useMetadata.ts src/main/preload.ts src/main/ipc/folder.ts
git commit -m "feat(hidden): add hidden flag to metadata + LibraryItem"
```

---

### Task 2: Pure incognito matcher + unit test (TDD)

**Files:**
- Create: `src/shared/hiddenMatch.ts`
- Create: `scripts/verify-hidden-guard.mjs`
- Modify: `package.json` (scripts)

- [ ] **Step 1: Write the failing test first**

Create `scripts/verify-hidden-guard.mjs`:

```js
import assert from 'node:assert/strict';
import { isSeriesHidden } from '../src/shared/hiddenMatch.ts';

const meta = {
  a: { anilistId: 111, malId: 222, hidden: true },
  b: { anilistId: 333, malId: 444 },
  c: { anilistId: 555, malId: null, hidden: false },
};

// Hidden series matches by either provider id.
assert.equal(isSeriesHidden(meta, 'anilist', 111), true, 'hidden anilist id matches');
assert.equal(isSeriesHidden(meta, 'mal', 222), true, 'hidden mal id matches');
// Visible series never matches.
assert.equal(isSeriesHidden(meta, 'anilist', 333), false, 'visible anilist id not hidden');
assert.equal(isSeriesHidden(meta, 'mal', 444), false, 'visible mal id not hidden');
assert.equal(isSeriesHidden(meta, 'anilist', 555), false, 'hidden:false not hidden');
// Unknown / zero ids short-circuit to false.
assert.equal(isSeriesHidden(meta, 'anilist', 999), false, 'unknown id not hidden');
assert.equal(isSeriesHidden(meta, 'mal', 0), false, 'zero id short-circuits');
// Provider isolation: anilistId 111 is hidden, but 111 as a MAL id is not.
assert.equal(isSeriesHidden(meta, 'mal', 111), false, 'provider ids do not cross');

console.log('verify-hidden-guard: OK');
```

- [ ] **Step 2: Run it to confirm it fails**

Run: `bun --bun scripts/verify-hidden-guard.mjs`
Expected: FAIL — cannot resolve `../src/shared/hiddenMatch.ts` (module does not exist yet).

- [ ] **Step 3: Implement the matcher**

Create `src/shared/hiddenMatch.ts`:

```ts
// Pure incognito matcher (isomorphic — no Electron imports). Returns true when
// any series entry carrying the given external media id is flagged hidden, so
// the main-process tracker guard can suppress AniList/MAL pushes for hidden
// series. Provider ids never cross: an AniList id is only matched against
// anilistId, a MAL id only against malId.
export type HiddenProvider = 'anilist' | 'mal';

interface HiddenLookupEntry {
  anilistId?: number;
  malId?: number | null;
  hidden?: boolean;
}

export function isSeriesHidden(
  metadata: Record<string, HiddenLookupEntry>,
  provider: HiddenProvider,
  mediaId: number,
): boolean {
  if (!mediaId) return false;
  for (const entry of Object.values(metadata)) {
    if (!entry || entry.hidden !== true) continue;
    if (provider === 'anilist' && entry.anilistId === mediaId) return true;
    if (provider === 'mal' && entry.malId === mediaId) return true;
  }
  return false;
}
```

- [ ] **Step 4: Run the test to confirm it passes**

Run: `bun --bun scripts/verify-hidden-guard.mjs`
Expected: `verify-hidden-guard: OK`

- [ ] **Step 5: Wire the verify script into package.json**

In `package.json`, add to `scripts` next to the other `verify:*` entries:

```json
    "verify:hidden-guard": "bun --bun scripts/verify-hidden-guard.mjs",
```

Run: `bun run verify:hidden-guard`
Expected: `verify-hidden-guard: OK`

- [ ] **Step 6: Commit**

```bash
git add src/shared/hiddenMatch.ts scripts/verify-hidden-guard.mjs package.json
git commit -m "feat(hidden): pure incognito matcher + verify:hidden-guard"
```

---

### Task 3: Block external tracker pushes for hidden series

**Files:**
- Modify: `src/main/handlers/trackerHandler.ts` (`MarkResult.reason` union, ~L373)
- Modify: `src/main/handlers/metadataHandler.ts` (add `isMediaHidden`, before `export default`, ~L196)
- Modify: `src/main/ipc/tracker.ts` (import + guard 3 handlers)

- [ ] **Step 1: Add `'hidden'` to the result reason union**

In `src/main/handlers/trackerHandler.ts`, find `reason?: 'no-account' | 'no-id' | 'not-newer' | 'error';` (~L373) and change to:

```ts
  reason?: 'no-account' | 'no-id' | 'not-newer' | 'error' | 'hidden';
```

- [ ] **Step 2: Add `isMediaHidden` to the metadata handler**

In `src/main/handlers/metadataHandler.ts`, add an import at the top with the other imports:

```ts
import { isSeriesHidden, type HiddenProvider } from '../../shared/hiddenMatch';
```

and add this method to the `metadataHandler` object (e.g. after `updateSeriesMetadata`, before the closing brace / `export default`):

```ts
  /** True if the local series carrying this external media id is flagged
   *  hidden. Used by the tracker IPC guard to keep incognito series from
   *  syncing to AniList/MAL. */
  async isMediaHidden(provider: HiddenProvider, mediaId: number): Promise<boolean> {
    const metadata = await this.loadMetadata();
    return isSeriesHidden(
      metadata as Record<string, { anilistId?: number; malId?: number | null; hidden?: boolean }>,
      provider,
      mediaId,
    );
  },
```

- [ ] **Step 3: Guard the three tracker IPC handlers**

In `src/main/ipc/tracker.ts`, add the import after the existing imports:

```ts
import metadataHandler from '../handlers/metadataHandler';
```

In each of `tracker:mark-episode`, `tracker:set-score`, and `tracker:set-progress`, immediately after the `typeof mediaId !== 'number' …` validation `throw` and before the `await trackerHandler.*` call, insert:

```ts
    if (await metadataHandler.isMediaHidden(provider, mediaId)) {
      return { ok: false, provider, newProgress: null, previousProgress: null, reason: 'hidden' as const };
    }
```

(Three insertions — one per handler. `provider` and `mediaId` are already validated/narrowed at each site.)

- [ ] **Step 4: Typecheck**

Run: `bun run typecheck`
Expected: PASS. Note: the renderer result handlers in `VideoPlayer.tsx` / `SeriesDetailPage.tsx` only branch on `reason === 'error'` / `'no-account'` / `'not-newer'`; an unrecognized `'hidden'` falls through to a silent no-op (no error toast, no "synced" toast) — no renderer change required.

- [ ] **Step 5: Commit**

```bash
git add src/main/handlers/trackerHandler.ts src/main/handlers/metadataHandler.ts src/main/ipc/tracker.ts
git commit -m "feat(hidden): block AniList/MAL pushes for hidden series at the IPC choke point"
```

---

### Task 4: Targeted `set-hidden` IPC + preload method

The only existing metadata write IPC is the full-map `save-metadata`, which can clobber concurrent background transaction writes (poster matches, probes). Add a targeted, transaction-serialized setter.

**Files:**
- Modify: `src/main/main.ts` (register `metadata:set-hidden` near the other metadata `ipcMain.handle` calls)
- Modify: `src/main/preload.ts` (interface + implementation)
- Verify: `src/types/electron.d.ts` re-exports preload types (no edit expected)

- [ ] **Step 1: Register the IPC handler**

In `src/main/main.ts`, near the existing `ipcMain.handle('save-metadata', …)` / `ipcMain.handle('load-metadata', …)` registrations, add (use the existing `metadataHandler` import already present in main.ts; if not imported, add `import metadataHandler from './handlers/metadataHandler';`):

```ts
  ipcMain.handle('metadata:set-hidden', async (_event, seriesId: unknown, hidden: unknown) => {
    if (typeof seriesId !== 'string' || !seriesId) throw new Error('seriesId required');
    const ok = await metadataHandler.updateSeriesMetadata(seriesId, { hidden: hidden === true });
    // Generic "metadata changed, re-walk" ping — same convention as the startup
    // catch-up — so any mounted list page refreshes and the card appears/vanishes.
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath: '', status: 'ready' });
    }
    return ok;
  });
```

(Match the surrounding code's reference to the main window variable — it is `mainWindow` in main.ts. If the metadata handlers live inside `app.whenReady()` where the window is in scope, place this alongside them.)

- [ ] **Step 2: Expose it in preload**

In `src/main/preload.ts`, add to the `ElectronAPI` interface near `saveMetadata` (~L112):

```ts
  setSeriesHidden: (seriesId: string, hidden: boolean) => Promise<boolean>;
```

and to the implementation object near the `saveMetadata` impl (~L304):

```ts
  setSeriesHidden: (seriesId: string, hidden: boolean) =>
    ipcRenderer.invoke('metadata:set-hidden', seriesId, hidden),
```

- [ ] **Step 3: Typecheck**

Run: `bun run typecheck`
Expected: PASS (electron.d.ts re-exports preload's API type, so `window.electronAPI.setSeriesHidden` is typed automatically).

- [ ] **Step 4: Commit**

```bash
git add src/main/main.ts src/main/preload.ts
git commit -m "feat(hidden): add transaction-safe metadata:set-hidden IPC"
```

---

### Task 5: Session-only "Show hidden" context

**Files:**
- Create: `src/renderer/contexts/HiddenShowsContext.tsx`
- Modify: `src/renderer/App.tsx` (import + wrap providers)

- [ ] **Step 1: Create the context**

Create `src/renderer/contexts/HiddenShowsContext.tsx`:

```tsx
import { createContext, useContext, useState, type ReactNode } from 'react';

interface HiddenShowsValue {
  showHidden: boolean;
  setShowHidden: (v: boolean) => void;
}

const HiddenShowsContext = createContext<HiddenShowsValue | undefined>(undefined);

// Session-only by design: `showHidden` is plain React state seeded to `false`,
// so it ALWAYS boots OFF and resets on every app launch. Deliberately NOT
// persisted to localStorage or config.json — the user re-enables it manually.
export function HiddenShowsProvider({ children }: { children: ReactNode }) {
  const [showHidden, setShowHidden] = useState(false);
  return (
    <HiddenShowsContext.Provider value={{ showHidden, setShowHidden }}>
      {children}
    </HiddenShowsContext.Provider>
  );
}

export function useHiddenShows(): HiddenShowsValue {
  const ctx = useContext(HiddenShowsContext);
  if (!ctx) throw new Error('useHiddenShows must be used within HiddenShowsProvider');
  return ctx;
}
```

- [ ] **Step 2: Wire the provider into App.tsx**

In `src/renderer/App.tsx`, add the import with the other context imports:

```tsx
import { HiddenShowsProvider } from "./contexts/HiddenShowsContext";
```

and wrap it inside `TitleLanguageProvider` (around the existing provider stack, ~L120-128). The provider must sit ABOVE the `<Routes>` (which is inside the inner component), so navigating between pages preserves `showHidden` while a full app reload resets it:

```tsx
      <TitleLanguageProvider>
        <HiddenShowsProvider>
          <TrackerProgressProvider>
            <ViewHistoryProvider>
              <ActivityLogProvider>
                {/* …existing inner content… */}
              </ActivityLogProvider>
            </ViewHistoryProvider>
          </TrackerProgressProvider>
        </HiddenShowsProvider>
      </TitleLanguageProvider>
```

- [ ] **Step 3: Typecheck**

Run: `bun run typecheck`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/renderer/contexts/HiddenShowsContext.tsx src/renderer/App.tsx
git commit -m "feat(hidden): session-only show-hidden context (boots off)"
```

---

### Task 6: Settings toggle

**Files:**
- Modify: `src/renderer/components/SettingsTab.tsx` (import + new section)

- [ ] **Step 1: Import the hook**

In `src/renderer/components/SettingsTab.tsx`, add:

```tsx
import { useHiddenShows } from '../contexts/HiddenShowsContext';
```

and inside `SettingsTab()`, near the other hook calls (~L69):

```tsx
  const { showHidden, setShowHidden } = useHiddenShows();
```

- [ ] **Step 2: Add the toggle UI**

Add a new `<Section>` (place it just before the `Playback` section, ~L369), reusing the existing `Toggle`, `pref-list`, `pref-row`, `pref-label`, `pref-help` patterns:

```tsx
      <Section title="Library">
        <div className="pref-list">
          <div className="pref-row">
            <div>
              <div className="pref-label">Show hidden shows</div>
              <div className="pref-help">Reveal incognito series across all pages. Resets off when AniBeam restarts.</div>
            </div>
            <Toggle on={showHidden} onChange={setShowHidden} ariaLabel="Toggle hidden shows" />
          </div>
        </div>
      </Section>
```

- [ ] **Step 3: Typecheck**

Run: `bun run typecheck`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/renderer/components/SettingsTab.tsx
git commit -m "feat(hidden): Settings toggle for revealing hidden shows"
```

---

### Task 7: Hide/Unhide button on the series detail page

**Files:**
- Modify: `src/renderer/pages/SeriesDetailPage.tsx` (state, handler, button in the hero action chips)

`Eye` and `EyeOff` are already imported (L3). The page already holds `decodedId`, `meta`, `setMeta`, `allMeta`, `setAllMeta`, and `Tooltip`.

- [ ] **Step 1: Add busy state**

Near the other `useState` hooks (~L208, by `scoreBusy`):

```tsx
  const [hideBusy, setHideBusy] = useState(false);
```

- [ ] **Step 2: Add the toggle handler**

Add near the other action handlers (e.g. by `submitSeriesScore`, ~L600). Writes through the transaction-safe IPC and updates local state optimistically:

```tsx
  const toggleHidden = async () => {
    if (!meta && !item) return;
    const next = !(meta?.hidden ?? false);
    setHideBusy(true);
    try {
      await window.electronAPI.setSeriesHidden(decodedId, next);
      setMeta((prev) => (prev ? { ...prev, hidden: next } : prev));
      setAllMeta((prev) => ({ ...prev, [decodedId]: { ...prev[decodedId], hidden: next } }));
    } catch (err) {
      console.error('setSeriesHidden failed', err);
    } finally {
      setHideBusy(false);
    }
  };
```

- [ ] **Step 3: Add the button to the hero action chips**

In the `series-hero-chips` action row, immediately after the AniList chip block (after its closing `</Tooltip>` at ~L980, still inside that `<div className="series-hero-chips …">`):

```tsx
              <Tooltip label="Incognito: stops tracker sync and hides from all lists">
                <button
                  type="button"
                  className={`hero-chip hero-chip-hide${meta?.hidden ? ' is-hidden' : ''}`}
                  aria-pressed={meta?.hidden ?? false}
                  disabled={hideBusy}
                  onClick={() => void toggleHidden()}
                >
                  {meta?.hidden ? <Eye size={14} /> : <EyeOff size={14} />}
                  <span>{meta?.hidden ? 'Unhide' : 'Hide'}</span>
                </button>
              </Tooltip>
```

(Confirm the action-row container; the AniList chip lives in the first `series-hero-chips` div, not the `--info` one. Place the button in the same div as the AniList chip.)

- [ ] **Step 4: Add minimal styling**

In `src/renderer/styles/App.css`, near the other `.hero-chip-*` rules, add:

```css
.hero-chip-hide { cursor: pointer; }
.hero-chip-hide.is-hidden {
  background: color-mix(in srgb, var(--accent-primary) 22%, transparent);
  color: var(--accent-primary);
}
```

- [ ] **Step 5: Typecheck**

Run: `bun run typecheck`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/renderer/pages/SeriesDetailPage.tsx src/renderer/styles/App.css
git commit -m "feat(hidden): Hide/Unhide button on series detail page"
```

---

### Task 8: Reveal styling — "Hidden" badge + dim on ShowCard

`ShowCard` (`src/renderer/components/ShowCard.tsx`) is shared by Home, Feed, and Watching and receives `item: LibraryItem` (which now carries `hidden`). A hidden item only ever reaches a card when reveal is on, so the card can badge/dim itself whenever `item.hidden` is true.

**Files:**
- Modify: `src/renderer/components/ShowCard.tsx`
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 1: Add the modifier class + badge in ShowCard**

In the ShowCard root element (the `className="show-card"` element, ~L153), append the modifier:

```tsx
      className={`show-card${item.hidden ? ' show-card--hidden' : ''}`}
```

Inside the poster wrap (`show-card-poster-wrap`, alongside the existing corner badges, ~L160-179), add a top-right "Hidden" pill:

```tsx
          {item.hidden && (
            <span className="show-card-hidden-badge" aria-label="Hidden">Hidden</span>
          )}
```

- [ ] **Step 2: Add the CSS**

In `src/renderer/styles/App.css`, near the other `.show-card-*` rules:

```css
.show-card--hidden { opacity: 0.55; }
.show-card--hidden:hover { opacity: 0.8; }
.show-card-hidden-badge {
  position: absolute;
  top: 0.4rem;
  right: 0.4rem;
  z-index: 3;
  padding: 0.1rem 0.45rem;
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.02em;
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--bg-primary) 70%, transparent);
  color: var(--text-secondary);
  backdrop-filter: blur(4px);
}
```

(If `--text-secondary` is not a defined token, use the nearest muted text token present in `:root`.)

- [ ] **Step 3: Typecheck**

Run: `bun run typecheck`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/renderer/components/ShowCard.tsx src/renderer/styles/App.css
git commit -m "feat(hidden): dim + Hidden badge on revealed cards"
```

---

### Task 9: Filter hidden series across the list pages

**Files:**
- Modify: `src/renderer/pages/HomePage.tsx` (Hidden tab + per-tab filtering + boot/toggle-off fallback)
- Modify: `src/renderer/pages/FeedPage.tsx` (filter feed source)
- Modify: `src/renderer/pages/WatchingPage.tsx` (drop hidden owned cards)
- Modify: `src/renderer/pages/MetadataTab.tsx` (filter + counts)

- [ ] **Step 1: HomePage — type, tab options, hook**

In `src/renderer/pages/HomePage.tsx`:

Change the tab type (L27):

```tsx
type LibraryTab = "all" | "series" | "movies" | "hidden";
```

Replace the static `TAB_OPTIONS` (L46-50) with a base list (the Hidden tab is appended dynamically in render):

```tsx
const BASE_TAB_OPTIONS: SegmentedOption<LibraryTab>[] = [
  { value: "all", label: "All" },
  { value: "series", label: "Series" },
  { value: "movies", label: "Movies" },
];
```

Add the hook inside `HomePage()` near the other hooks (~L185):

```tsx
  const { showHidden } = useHiddenShows();
```

and the import at the top:

```tsx
import { useHiddenShows } from "../contexts/HiddenShowsContext";
```

- [ ] **Step 2: HomePage — split lists by hidden + per-tab selection**

Replace the `seriesItems` / `movieItems` / `activeItems` block (~L279-286) with:

```tsx
  // Hidden series are segregated into their own tab — never mixed into
  // All/Series/Movies. When reveal is off they vanish from every tab.
  const visibleItems = useMemo(() => items.filter((i) => !i.hidden), [items]);
  const hiddenItems = useMemo(() => items.filter((i) => i.hidden), [items]);
  const seriesItems = useMemo(() => visibleItems.filter((i) => i.type !== "movie"), [visibleItems]);
  const movieItems = useMemo(() => visibleItems.filter((i) => i.type === "movie"), [visibleItems]);

  const activeItems =
    tab === "series" ? seriesItems :
    tab === "movies" ? movieItems :
    tab === "hidden" ? hiddenItems :
    visibleItems;
```

- [ ] **Step 3: HomePage — build tab options + guard against stale "hidden" tab**

Add near the other derived values (before the return), computing the visible tab set and snapping off the Hidden tab when reveal is off:

```tsx
  const tabOptions = useMemo<SegmentedOption<LibraryTab>[]>(
    () => (showHidden && hiddenItems.length > 0
      ? [...BASE_TAB_OPTIONS, { value: "hidden", label: "Hidden" }]
      : BASE_TAB_OPTIONS),
    [showHidden, hiddenItems.length],
  );

  // If reveal flips off (or the app booted with a persisted "hidden" tab),
  // fall back to "all" so we never sit on an unavailable tab.
  useEffect(() => {
    if (!tabOptions.some((o) => o.value === tab)) setTab("all");
  }, [tabOptions, tab]);
```

Then in the JSX, pass `options={tabOptions}` to the `SegmentedSwitch` (replacing the static `TAB_OPTIONS` reference, ~L504-509).

- [ ] **Step 4: FeedPage — filter the source items**

In `src/renderer/pages/FeedPage.tsx`, add the import and hook:

```tsx
import { useHiddenShows } from "../contexts/HiddenShowsContext";
```
```tsx
  const { showHidden } = useHiddenShows();
```

Where the feeds are built from `items` (the `buildRecentFeed(items)` / `buildUpcomingFeed(items)` call sites, ~L114-157 region in render), feed them a filtered list:

```tsx
  const feedItems = useMemo(
    () => (showHidden ? items : items.filter((i) => !i.hidden)),
    [items, showHidden],
  );
```

and use `feedItems` in place of `items` for the feed builders. (Revealed hidden entries still render through `ShowCard`, which badges+dims them via Task 8.)

- [ ] **Step 5: WatchingPage — drop hidden owned cards unless revealed**

In `src/renderer/pages/WatchingPage.tsx`, add the import and hook:

```tsx
import { useHiddenShows } from "../contexts/HiddenShowsContext";
```
```tsx
  const { showHidden } = useHiddenShows();
```

In the `cards` builder (~L122-136), after resolving `owned`, drop hidden owned entries when reveal is off (return null, then filter). Keep indexing ALL library items (so a hidden owned show still resolves to `owned` and is dropped, rather than falling back to a synth AniList card):

```tsx
    const mapped = sorted.map((e) => {
      const owned =
        libraryIndex.byAnilist.get(e.anilistId) ??
        (e.malId != null ? libraryIndex.byMal.get(e.malId) : undefined);
      if (owned?.hidden && !showHidden) return null;
      return owned
        ? { key: owned.id, item: owned, inLibrary: true, siteUrl: e.siteUrl }
        : { key: `anilist:${e.anilistId}`, item: synthItem(e), inLibrary: false, siteUrl: e.siteUrl };
    });
    return mapped.filter((c): c is WatchingCard => c !== null);
```

Add `showHidden` to the `cards` useMemo dependency array.

- [ ] **Step 6: MetadataTab — filter rows + counts**

In `src/renderer/pages/MetadataTab.tsx`, add the import and hook:

```tsx
import { useHiddenShows } from "../contexts/HiddenShowsContext";
```
```tsx
  const { showHidden } = useHiddenShows();
```

In `filteredSeries` (~L82), add a hidden guard as the first check inside the `.filter`:

```tsx
      if (!showHidden && data.hidden) return false;
```

and update `filterCounts` so the totals match what's shown — wrap each count's source list to exclude hidden when reveal is off. Simplest: derive a base list once:

```tsx
  const visibleSeries = useMemo(
    () => (showHidden ? seriesList : seriesList.filter(([, d]) => !d.hidden)),
    [seriesList, showHidden],
  );
```

then compute `filterCounts` from `visibleSeries` instead of `seriesList`, and iterate `visibleSeries` in `filteredSeries`. (Add `showHidden`/`visibleSeries` to the relevant dependency arrays.)

- [ ] **Step 7: Typecheck**

Run: `bun run typecheck`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add src/renderer/pages/HomePage.tsx src/renderer/pages/FeedPage.tsx src/renderer/pages/WatchingPage.tsx src/renderer/pages/MetadataTab.tsx
git commit -m "feat(hidden): filter hidden series across Home/Feed/Watching/Metadata (+ Hidden tab)"
```

---

### Task 10: Full verification + package

**Files:** none (verification only)

- [ ] **Step 1: Typecheck**

Run: `bun run typecheck`
Expected: PASS.

- [ ] **Step 2: Run the verify suite**

Run each (none should regress; the new one should pass):

```bash
bun run verify:hidden-guard
bun run verify:logger
bun run verify:motion
bun run verify:probe
bun run verify:folder
bun run verify:franchise-graph
bun run verify:franchise-crawler
bun run verify:franchise-lanes
```

Expected: all print OK / exit 0.

- [ ] **Step 3: Manual smoke (via `bun run dev`)**

1. Hide a series from its detail page → button flips to "Unhide".
2. It disappears from Home/Feed/Watching/Metadata. Toggle "Show hidden" ON in Settings → it reappears (badged + dimmed); Home shows a "Hidden" tab containing it; All/Series/Movies do not.
3. Play an episode of a hidden series → resume position + local watched marks update, but AniList/MAL progress does NOT change, and no error toast appears.
4. Restart app → "Show hidden" is OFF again; hidden series gone from lists; Hidden tab absent.
5. Unhide → series returns to normal tabs; subsequent watching resumes tracker sync.

- [ ] **Step 4: Package**

Run: `bun run package`
Expected: builds to `out/AniBeam-linux-x64/anibeam` so the `.desktop` launcher picks up the change.

- [ ] **Step 5: Final state**

The branch `feat/hidden-shows-incognito` is ready to merge to `main` once typecheck + verify scripts are green and the smoke test passes.

---

## Self-review notes

- **Spec coverage:** button (Task 7) ✓; no-tracking guard (Tasks 2-3) ✓; hidden from all list pages (Task 9) ✓; Settings toggle that boots off (Tasks 5-6) ✓; dedicated Hidden tab (Task 9) ✓; badge+dim (Task 8) ✓; external-only / local history preserved (Task 3 — only the `tracker:*` push IPCs are guarded; `markEpisodeViewed` untouched) ✓.
- **Type consistency:** `hidden` (metadata + LibraryItem), `isSeriesHidden`/`HiddenProvider` (shared), `isMediaHidden` (handler), `setSeriesHidden` (preload), `useHiddenShows`/`showHidden`/`setShowHidden` (context), `LibraryTab` includes `"hidden"`, `BASE_TAB_OPTIONS`/`tabOptions` — names consistent across tasks.
- **Subscriptions:** out of scope — it lists anirss RSS feeds, not local series; nothing hideable there.
