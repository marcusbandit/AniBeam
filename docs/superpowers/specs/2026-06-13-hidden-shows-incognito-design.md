# Hidden Shows (Incognito) — Design

**Date:** 2026-06-13
**Status:** Approved for implementation

## Goal

Let the user mark a series as **hidden** (incognito) from its detail page. A hidden series:

1. **Never syncs to external trackers** (AniList/MAL) — no progress pushes, no score pushes, no progress corrections. Existing entries on AniList/MAL are left untouched; we simply stop pushing.
2. **Does not appear on any list page** (Home/library, Feed, Watching, Subscriptions, Metadata) — unless the user enables a **"Show hidden shows"** toggle in Settings.

Local watch data is **unaffected**: resume position and watched marks are still recorded on disk, so playback works normally. Hiding is purely "stop telling the outside world + hide from my own lists."

The "Show hidden shows" toggle is **session-only**: it always boots **OFF** and is never persisted. The user turns it on manually each session.

## Decisions (settled)

- **Incognito scope:** external trackers only. Local view history keeps recording.
- **Reveal styling:** when revealed, hidden shows get a subtle **"Hidden" badge + dimmed card**.
- **Library tabs:** when the toggle is on, the library (HomePage) gets a dedicated **"Hidden"** tab next to All/Series/Movies. Hidden series live **only** in that tab — they are not mixed into All/Series/Movies. On the other pages (Feed/Watching/Subscriptions/Metadata) revealed hidden shows appear inline (badged + dimmed).

## Data model

Add one optional field to `SeriesMetadata` (`src/renderer/hooks/useMetadata.ts`), persisted in `metadata.json` keyed by `seriesId`:

```ts
/** Incognito flag. When true, the series never syncs to external trackers and
 *  is hidden from all list pages unless the session "Show hidden" toggle is on. */
hidden?: boolean;
```

Absent / `false` = visible. No migration needed (catch-all index already permits it; old entries read as not-hidden).

`metadataHandler.updateSeriesMetadata(seriesId, { hidden })` is the write path (already exists, serialized transaction). `cleanForSave` keeps unknown fields, so `hidden` round-trips.

## Component / flow breakdown

### 1. Series detail — Hide/Unhide button

**File:** `src/renderer/pages/SeriesDetailPage.tsx` (hero chips region, ~L906–1021).

- Add a button to the hero chip row: **"Hide"** when `!meta.hidden`, **"Unhide"** when `meta.hidden`, with an eye-off / eye icon (lucide `EyeOff` / `Eye`).
- On click: `await updateSeriesMetadata(decodedId, { hidden: !meta.hidden })` then refresh local `meta` state. Show busy state while writing (mirror the existing `scoreBusy` pattern).
- A custom portal tooltip (per project rule — no native `title=`) explains: "Incognito: stops tracker sync and hides from all lists."
- The detail page loads a series **directly by id** (`libraryWalk` + `loadMetadata`), independent of any list filtering, so a hidden series remains reachable by URL and from the Hidden tab to be un-hidden.

### 2. Tracker guard — the single incognito choke point (main process)

All external pushes funnel through three IPC handlers in `src/main/ipc/tracker.ts`: `tracker:mark-episode`, `tracker:set-score`, `tracker:set-progress`. Guarding here covers **every** caller (VideoPlayer auto-mark, SeriesDetail manual set-progress/score, any future caller) — the renderer cannot leak.

- Add `metadataHandler.isMediaHidden(provider, mediaId): Promise<boolean>` — loads metadata and returns `true` if any series entry with a matching `anilistId` (provider `anilist`) or `malId` (provider `mal`) has `hidden === true`. O(n) scan over the metadata map; pushes are infrequent so cost is negligible.
- In each of the three IPC handlers, after provider/arg validation and before calling the handler:
  ```ts
  if (await metadataHandler.isMediaHidden(provider, mediaId)) {
    return { ok: false, provider, newProgress: null, previousProgress: null, reason: 'hidden' };
  }
  ```
  No `broadcastProgressChanged()` fires (nothing changed).
- Add `'hidden'` to the `MarkResult.reason` union in `src/main/handlers/trackerHandler.ts`.
- **Renderer handling:** the existing result handlers in VideoPlayer/SeriesDetail only surface `reason === 'error'` / `'no-account'`; an unrecognized `'hidden'` already falls through to a silent no-op (no error toast, no "synced" toast). No renderer changes required for correctness — verified against current branches in both files.

This keeps local `markEpisodeViewed` (view history) fully working — only the AniList/MAL network pushes are suppressed.

### 3. Session-only "Show hidden" toggle — new context

**New file:** `src/renderer/contexts/HiddenShowsContext.tsx`, following the `TitleLanguageContext` pattern.

- State: `showHidden: boolean` (init `false`), `setShowHidden(v)`.
- **No persistence** — plain `useState(false)`. Because it lives in React state that mounts fresh each app launch, it always boots OFF. Explicitly do NOT read/write localStorage or config.json for it.
- Export `useHiddenShows()`.
- Wrap the app in `<HiddenShowsProvider>` in `src/renderer/App.tsx` (alongside the other providers).

### 4. Settings — the toggle UI

**File:** `src/renderer/components/SettingsTab.tsx`.

- Add a row using the existing `Toggle` component (L22–40) bound to `useHiddenShows()`. Label: **"Show hidden shows"**, sublabel: "Reveal incognito series across all pages. Resets off when the app restarts." Place it in a sensible existing section (e.g. near Playback/Library prefs).

### 5. List-page filtering

A series is hidden when `metadata[item.id]?.hidden === true`. Each page reads `useHiddenShows()` and the metadata map it already loads.

- **HomePage** (`src/renderer/pages/HomePage.tsx`): the tabbed one.
  - `LibraryTab` type gains `"hidden"`.
  - `TAB_OPTIONS` is computed: base `All / Series / Movies`, plus `{ value: "hidden", label: "Hidden" }` appended **only when `showHidden` is true**.
  - Per-tab item selection:
    - `all` → items that are **not** hidden
    - `series` → non-movie **and** not hidden
    - `movies` → movie **and** not hidden
    - `hidden` → hidden items (any type)
  - When `showHidden` flips off (or on boot), if the persisted `LS_TAB` is `"hidden"` or the active tab is `"hidden"`, fall back to `"all"`. Validate the stored tab against the currently-available option list.
  - Hidden cards (in the Hidden tab) render with the badge + dim treatment (§6).

- **FeedPage / WatchingPage / SubscriptionsPage / MetadataTab**: no tabs — filter inline. Build the displayed list excluding hidden series unless `showHidden`. When `showHidden` is on, hidden entries appear in place with the badge + dim treatment.
  - These pages key off local library items / metadata. For Subscriptions and Watching that also index by `anilistId`/`malId`, the hidden flag is resolved from the local metadata entry for that series (same `metadata[id]?.hidden` lookup); if a row has no local series backing it, it is treated as not hidden (nothing to hide).

### 6. Reveal styling — badge + dim

- Add a `hidden?: boolean` prop path for series cards (reuse the existing `Card` primitive; add an optional `Hidden` pill badge in the card body and a `.card--hidden { opacity: .55 }`-style dim class in `App.css`, using design tokens).
- Badge: small pill reading "Hidden" with the muted/secondary token palette and `--radius-pill`. No emoji.
- Applied wherever a hidden series is rendered while revealed (Home Hidden tab, Feed, Watching, Subscriptions, Metadata).

## Testing / verification

- `bun run typecheck` must pass.
- Existing `verify:*` scripts must stay green (no logic touched in their domains; run the suite).
- Manual smoke (via `bun run dev`):
  1. Hide a series from its detail page → button flips to "Unhide".
  2. It disappears from Home/Feed/Watching/Metadata. Toggle "Show hidden" on → it reappears (badged + dimmed); Home shows a "Hidden" tab containing it; All/Series/Movies do not.
  3. Play an episode of a hidden series → resume position + local watched marks update, but AniList/MAL progress does **not** change (and no error toast).
  4. Restart app → "Show hidden" is OFF again; hidden series are gone from lists; Hidden tab absent.
  5. Unhide → series returns to normal tabs; subsequent watching resumes tracker sync.

## Out of scope (YAGNI)

- No bulk hide/unhide, no "hidden count" UI, no per-page independent toggles.
- No reverting or scrubbing of progress already on AniList/MAL when hiding.
- No persistence of the reveal toggle (by explicit requirement).
