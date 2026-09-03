# Replace "Subscriptions" tab with "Watching"

Date: 2026-05-28

## Goal

Subscriptions is rarely used. Move it out of the top-nav and into Settings
(reachable via a button). Reclaim its navbar slot for a new **Watching** tab
that lists the user's AniList "Currently Watching" list with metadata. Shows
present in the local library are **outlined** (the normal card border); shows
on the watching list but not present locally are **not outlined** (borderless).

## Decisions (locked)

- **Click on a non-library card** → open the show's AniList page in the
  default browser. Library cards keep their current behavior (in-app series
  page).
- **Data source** → AniList specifically (not the generic "main provider").
- **Subscriptions entry point** → a button in Settings that navigates to the
  existing `/subscriptions` page (page kept intact, just removed from navbar).
- **Default sort** → recently updated (AniList `updatedAt` desc).
- **Statuses included** → `CURRENT` and `REPEATING` (rewatching counts as
  watching).
- **Non-library visual** → full color, only the border is dropped ("simply not
  outlined", taken literally).

## Architecture

### Why a new fetch is needed
The tracker progress snapshot (`trackerStore.ts`) caches only
`{progress, status, score, rewatch}` keyed by media id — no titles or posters.
That is enough for badges on library cards, but a non-library watching entry
has no `LibraryItem` to draw from. So the Watching page needs a richer,
metadata-carrying fetch.

### Main process — new IPC `tracker:get-watching-list`
- Added to `trackerHandler` as `getAnilistWatchingList()`.
- One AniList `MediaListCollection` query (type ANIME) for the connected
  user, pulling per entry:
  `progress`, `status`, `score(format: POINT_10_DECIMAL)`, `updatedAt`, and
  `media { id idMal title { romaji english } coverImage { large } averageScore episodes siteUrl nextAiringEpisode { episode airingAt } }`.
- Flatten all lists, keep entries whose normalized status is `watching` or
  `repeating`.
- Returns a result wrapper:
  ```ts
  type WatchingListResult =
    | { ok: true; entries: AnilistWatchingEntry[] }
    | { ok: false; error: string; needsAuth?: boolean };
  ```
  `needsAuth: true` when no AniList account is connected, so the page can show
  a "Connect AniList in Settings" empty state. Errors are sanitized with the
  existing `sanitizeTrackerError`.
- No disk cache — fetched fresh each time the page mounts (one cheap query).

`AnilistWatchingEntry` shape (preload-exported):
```ts
interface AnilistWatchingEntry {
  anilistId: number;
  malId: number | null;
  titleRomaji: string | null;
  titleEnglish: string | null;
  coverImage: string | null;       // large
  averageScore: number | null;     // 0-100 (AniList scale)
  totalEpisodes: number | null;
  progress: number;
  status: 'watching' | 'repeating';
  score: number | null;            // 0-10
  updatedAt: number | null;        // unix seconds, sort key
  nextAiringEpisode: { episode: number; airingAtMs: number } | null;
  siteUrl: string;                 // AniList page URL
}
```

Wiring: `src/main/ipc/tracker.ts` registers the channel; `preload.ts` exposes
`trackerGetWatchingList()` and the new types.

### Renderer — new `WatchingPage.tsx`
1. On mount, in parallel: `trackerGetWatchingList()` and `libraryWalk()`.
2. Index `LibraryItem[]` by `anilistId` (skip null ids).
3. Sort entries by `updatedAt` desc (nulls last).
4. For each entry:
   - **In library** (matching `LibraryItem`): render `ShowCard` with the real
     item → outlined, click → in-app series page (unchanged behavior).
   - **Not in library**: build a synthetic `LibraryItem` (id `anilist:<id>`,
     `poster` = coverImage URL, `posterLocal: null`, titles, `totalEpisodes`,
     `anilistId`, `malId`, `averageScore`, `source: 'anilist'`, `files: []`,
     `episodes` synthesized from `nextAiringEpisode` so the countdown works).
     Render `ShowCard` with `outlined={false}` and
     `onActivate={() => openExternal(entry.siteUrl)}`.
5. Watched-progress and personal-score badges already resolve via
   `TrackerProgressContext` by `anilistId`, so they render on both kinds.
6. Per-card air countdown reuses the Feed pattern: a single shared `nowMs`
   ticker (30s) passed to cards when any entry has an upcoming episode.
7. States: loading, `needsAuth` empty state ("Connect AniList in Settings"),
   generic error state, and empty ("Nothing on your watching list").

### `ShowCard.tsx` — two additive props
- `outlined?: boolean` (default `true`). When `false`, adds a `--bare`
  modifier class that sets the poster-wrap border (incl. hover) to
  transparent. Only visual difference for non-library cards.
- `onActivate?: () => void` (default = navigate to `/series/:id`). The
  Watching page passes `openExternal(siteUrl)` for remote cards.

CSS: add `.show-card-poster-wrap--bare { border-color: transparent; }` and a
matching hover override in `App.css`.

### Navbar / routing (`App.tsx`)
- Replace the Subscriptions `NavLink` with **Watching** (`Eye` icon →
  `/watching`).
- Keep the `/subscriptions` route registered (no longer in navbar).
- Add `/watching` route → `WatchingPage`.
- Add `/watching` to `titleForPath`.

### Settings (`SettingsTab.tsx`) + Subscriptions page
- New `Section` "Subscriptions" with a button → `navigate('/subscriptions')`
  (via `useNavigate`).
- `SubscriptionsPage` gets a "← Settings" back link in its header since it is
  no longer reachable from the navbar.

## Out of scope
- No changes to the progress cache, tracker store schema, or MAL paths.
- No in-app detail view for non-library shows (browser link only).
- No persistence of a sort toggle (single default sort for now).

## Files touched
- `src/main/handlers/trackerHandler.ts` — new fetch + handler method
- `src/main/ipc/tracker.ts` — register channel
- `src/main/preload.ts` — expose API + types
- `src/renderer/App.tsx` — navbar + routes + title
- `src/renderer/pages/WatchingPage.tsx` — new
- `src/renderer/components/ShowCard.tsx` — `outlined` + `onActivate` props
- `src/renderer/components/SettingsTab.tsx` — Subscriptions section
- `src/renderer/pages/SubscriptionsPage.tsx` — back link
- `src/renderer/styles/App.css` — `--bare` border modifier
