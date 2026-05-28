# Franchise Map — design

**Date:** 2026-05-28
**Branch:** `feat/franchise-map` (worktree, from `prod` @ `fe6c286`)
**Replaces:** the current two-bucket "Related" section in `src/renderer/pages/SeriesDetailPage.tsx`

## Context & motivation

The Series Detail page currently renders relations as a **"Related"** section split into
two buckets — **Available** (franchise entries the user has on disk, laid on a teal
timeline rail with a synthesized "you are here" card) and **Discover** (direct AniList
relations not owned, which open AniList externally) — plus a separate **Recommendations**
strip. The owned set is found by BFS-walking the franchise graph but **surfacing only
owned entries**; non-owned entries are shown only as direct (one-hop) relations.

We want to rethink this around three goals the user stated:

1. **Timeline-first** — make the franchise's chronological chain the centerpiece.
2. **Care less about what's on disk** — stop splitting the UI by ownership; ownership
   becomes a subtle per-node marker, not a top-level grouping.
3. **Whole franchise** — show *all* relations across the entire franchise, not just the
   relations of the current season/movie/entry.

## Goals

- A spatial **Franchise Map**: the current series highlighted within the franchise's
  full prequel→sequel **spine**, with sources/parents above, side stories/spin-offs/
  recaps below, and alternate retellings as side-branches.
- Surface the **entire** franchise graph regardless of ownership.
- Render **instantly** from local data, then **fill in** missing nodes from AniList in
  the background and cache the result.

## Non-goals

- A general-purpose pan/zoom graph editor (we chose structured lanes, not a free canvas).
- Traversing loose `CHARACTER`/cameo links into unrelated franchises.
- Reworking the **Recommendations** strip — it stays as its own separate section
  ("similar shows", not franchise) below the map.
- Changing how metadata is fetched at ingest.

## Decisions (from brainstorming)

| Question | Decision |
|---|---|
| What's driving it | Timeline-first + visual; show the *whole* franchise; de-emphasize ownership |
| ALTERNATIVE versions | **Side branch** — short secondary chain forking off the divergence node |
| Graph completeness | **Hybrid** — instant local render, background-fill from AniList, cache |
| Render model | **Structured lanes + horizontal scroll** (current auto-centered) |
| `CHARACTER` relations | **Dropped** from the map (crossovers, not franchise) |
| Recommendations | **Kept** as a separate section |
| Node style | **Compact poster-tiles**, not the current rich relation cards |
| Workflow | Isolated git worktree; iterate in `bun run dev`; package only when done |

## Layout

A horizontally-scrolling map with the current series highlighted and auto-centered on
load. Three persistent lanes plus side-branches:

```
  TOP LANE   ·  sources / upstream      [ manga ]   [ light novel ]   [ parent ]
                                            │           │               │
  SPINE      ·  prequel → sequel chain  …─[S1]─[S2]─[★ CURRENT ★]─[S3]─[Movie]─… ──→ scroll
                                            │           │
  BOTTOM     ·  side / spin / recap      [ OVA ]  [ side story ] [ spin-off ] [ recap ]

  side-branch (ALTERNATIVE): a short secondary spine forking off the divergence node
```

- The **spine** is the entire prequel→sequel chain of the franchise (sorted by start
  date / `seasonYear`), with *current* highlighted **in** the chain (ring + "you are
  here" treatment) rather than pulled out as a separate card.
- The spine carries the existing teal timeline rail, extended across the whole franchise.
- Thin **connectors** drop from each spine node up to its sources and down to its
  companions.
- **Horizontal scroll**; current node is scrolled into center on mount. Long franchises
  get a "jump to current" affordance.

## Lane assignment (relationType → position)

| Lane | relationTypes |
|---|---|
| **Spine** | `PREQUEL`, `SEQUEL` (+ the synthesized current node), sorted chronologically |
| **Top** (upstream) | `SOURCE`, `PARENT`, and `ADAPTATION` *when it points upstream* |
| **Bottom** (downstream/companion) | `SIDE_STORY`, `SPIN_OFF`, `SUMMARY`, `COMPILATION`, `CONTAINS`, `ADAPTATION` *when downstream*, `OTHER` |
| **Side-branch** | `ALTERNATIVE` |
| **Excluded from map** | `CHARACTER` |

### SOURCE / ADAPTATION directionality

AniList edges are directional but the current code conflates them — `RELATION_LABEL`
maps `ADAPTATION` → "Source" and has no `SOURCE` entry. This must be fixed. Direction is
resolved by **media type**, not the label alone:

- An edge from an **anime** to a `MANGA` / `NOVEL` / `LIGHT_NOVEL` / `ONE_SHOT` /
  `VISUAL_NOVEL` is **upstream** → top lane (the source material).
- An edge from a **manga/novel** to an `ANIME` is **downstream** → bottom lane (the
  adaptation).
- Same-type edges fall back to `relationType` for placement.

`CHARACTER` is currently **missing entirely** from `RELATION_LABEL` (would render raw);
since it's excluded from the map this no longer matters for the map, but the label map
should still be corrected for any other use.

## Data — hybrid graph building

The relation graph can only be *traversed* through nodes whose `relations` array we
have. Today that's only series the user owns (fetched at ingest). Non-owned entries are
one-hop leaves. To show the whole franchise:

1. **Instant (local):** build the graph from owned-series metadata (`allMeta`), one hop
   per owned node, deduped by `anilistId`. Render immediately — this is roughly today's
   BFS but **without** filtering out non-owned nodes.
2. **Fill (AniList):** background-crawl via the existing `anilistHandler` query path
   (`MEDIA_BY_ID_QUERY` / enrichment relations) for non-owned nodes, splicing results
   into the live graph as they arrive so the map progressively completes.
3. **Cache:** persist the closed graph to disk keyed by a canonical franchise-root id so
   subsequent visits to *any* series in the franchise are instant and offline.

### Crawl bounds (safety — baked in as defaults)

- **Edge filter:** traverse **only** franchise edges. **Exclude `CHARACTER` and
  `OTHER`** from traversal so cameos/loose links don't drag in unrelated franchises.
- **Node cap:** stop crawling at ~150 nodes (configurable constant); render what we have.
- **Closure:** stop when no new nodes are discovered.
- **Dedup:** by `anilistId`; a node reachable by multiple edges appears once, carrying
  all its relationTypes.

### Cache shape (proposed)

- One entry per franchise, keyed by a canonical root (e.g. the smallest `anilistId` in
  the connected component, computed deterministically so every member resolves the same
  key).
- Stored value: the node list (id, titles, format, type, poster, seasonYear, siteUrl,
  ids) + edge list (from, to, relationType).
- **TTL ~14 days** + refresh-on-demand. Stale-but-present cache still renders instantly;
  refresh happens in the background.
- Persisted as a JSON file in the Electron `userData` dir via the existing main-process
  store pattern (mirror `src/main/services/trackerStore.ts` — `app.getPath('userData')`
  + `readFile`/`writeFile`), e.g. `franchiseGraphCache.json`.

## Nodes & ownership ("care less about disk")

- **Compact poster-tiles**, smaller than today's relation cards, styled with existing
  design tokens (flat surfaces, big radii, dark — per project design direction).
- **Ownership is a subtle corner marker**, not a grouping:
  - Owned → marker + watch-progress ring; click **navigates in-app** to that series.
  - Not owned → marker (AniList); click **opens AniList externally**.
- Each tile shows: poster, title, format chip, year; current node gets the highlight
  ring + "you are here".
- Tracker list-status marker (`listStatusMarker`) carries over.
- **Custom hover tooltip** only (no native `title=`), per project convention.

## Component / code structure

`SeriesDetailPage.tsx` is ~1,440 lines. We extract rather than pile on:

- `src/main/services/franchiseGraph.ts` — crawl + cache + closure (main process); IPC
  endpoint to request a franchise graph by anilist/mal id.
- `src/main/preload.ts` / IPC wiring — expose the franchise-graph request to the renderer.
- `src/renderer/hooks/useFranchiseGraph.ts` — hybrid consumer: returns the local graph
  immediately, subscribes to fill updates, exposes loading state.
- `src/renderer/components/franchise/FranchiseMap.tsx` — the lane layout + scroll +
  centering.
- `src/renderer/components/franchise/FranchiseNode.tsx` — the compact tile.
- `src/renderer/components/franchise/laneAssignment.ts` — relationType + media-type →
  lane, plus chronological sort utilities (replaces `RELATION_LABEL` / `RELATION_ORDER`
  / `sortRelations` usage for the map).
- Styles in `src/renderer/styles/` following existing conventions.

### Removed / changed in `SeriesDetailPage.tsx`

- Remove `inLibraryRelations` / `externalRelations` two-bucket split and the
  "Available"/"Discover" groups.
- Remove the synthesized "current" card logic in its current form (the highlight moves
  into `FranchiseMap`).
- Replace the "Related" section body with `<FranchiseMap seriesId={…} />`.
- Keep the "Recommendations" section as-is.

## Edge cases

- **Branching** (one entry → two sequels) and **merging**: handled with sub-rows within
  the spine lane; connectors fan out. Strict single-row can't represent this.
- **Manga-rooted current series:** direction rules flip (adaptations go downstream).
- **Missing data while filling:** tiles render in a loading/skeleton state until the
  crawl supplies them; ownership/click behavior resolves once ids are known.
- **No relations / standalone:** the map collapses to just the current node (or the
  section is hidden), matching today's behavior of not showing a lonely card.
- **Cache miss + offline:** local graph still renders; fill silently no-ops.

## Testing & verification

- `bun run typecheck` (`tsc --noEmit`) must stay clean throughout.
- Unit tests for `laneAssignment.ts` (relationType + media-type → lane; the
  SOURCE/ADAPTATION direction matrix) and for graph closure/dedup/cap in
  `franchiseGraph.ts`.
- Manual verification in `bun run dev` against real franchises with known shapes:
  a linear multi-season show, a manga-rooted show, and one with an `ALTERNATIVE`
  (reboot) to exercise the side-branch.
- No packaging until the feature is complete; then `bun run package`.

## Workflow notes

- All work happens in the `feat/franchise-map` worktree.
- Iterate in `bun run dev` (no packaging during iteration).
- `node_modules` is symlinked from the main checkout (identical base commit) because the
  castlabs `electron` git dependency can't be cloned in the sandbox.
