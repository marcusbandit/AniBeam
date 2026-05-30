# Spec: Franchise crawler (the missing store writer)

## Problem

`src/main/services/franchiseGraph.ts` is a **read-only** consumer of an on-disk
franchise store (`franchiseStore.json` + `franchises/franchise-<rootId>.json` in
`app.getPath('userData')`). Its own header comment says *"No AniList fetches, no
writes. The prefill script is responsible for populating the on-disk store."* —
but **no such writer exists** anywhere in the source or the compiled binary.

Consequences observed:
- `anilistHandler.fetchRelations()` (`src/main/handlers/anilistHandler.ts`) — the
  purpose-built, rate-limit-aware crawl fetcher — has **zero call sites**.
- `closeGraph()` (`src/shared/franchise.ts`) accepts an optional live `fetch`
  callback for incremental BFS, but **every caller passes none**.
- The on-disk store was populated once by an offline prefill (leftover
  `franchiseStore.raw.json` proves it) and is now frozen. Any series added after
  that run (e.g. Railgun T, anilistId 104462) is absent → `getFranchiseGraph`
  returns a lone "Root" node and never grows.

## Goal

Re-introduce the **writer half**: a crawler that fetches relations via
`fetchRelations`, drives BFS through `closeGraph`'s `fetch` hook, and **persists
each node as it lands** so the existing file-watch push (`franchise:store-updated`)
makes the relations tree **form live** node-by-node — no renderer changes needed.

Two entry points (both confirmed with the user):

1. **Background gap-fill** at startup — crawl every owned series whose franchise
   isn't in the store yet. **Keep + fill gaps**: never re-fetch nodes that already
   have `fetchedAt > 0`; only fetch missing ones.
2. **Gentle on-click refresh** — when a series is opened, slowly re-crawl *its*
   franchise to catch updates (a newly announced sequel, etc.). Force-refetch only
   the opened series' own node; let newly revealed nodes fill via gap logic.

## Disk layout (unchanged — must stay compatible with the reader)

- `franchiseStore.json`: `{ library: { "<ownedId>": { node, relations, fetchedAt, franchise } } }`
  — only **owned** series; `franchise` is the per-component file key
  `"franchise-<rootId>"`.
- `franchises/franchise-<rootId>.json`: `{ rootId, byId: { "<anyId>": { node, relations, fetchedAt } } }`
  — full closure for one connected component. `rootId` = smallest anilistId in
  the component. `fetchedAt: 0` means "node known from a relation entry but not
  directly fetched" (stale); `> 0` means directly fetched.

## New module: `src/main/services/franchiseCrawler.ts`

### Reuse, don't duplicate

Export these from `franchiseGraph.ts` and import them here (do **not** redefine the
disk layout): `indexPath`, `franchisesDir`, `franchiseFilePath`, `readIndex`,
`readFranchiseFile`, plus the `FranchiseStoreIndex`/`FranchiseFile`/`ShowEntry`/
`LibraryEntry` types. Add two new writers there too:

```ts
export async function writeIndex(index: FranchiseStoreIndex): Promise<void>
export async function writeFranchiseFile(key: string, file: FranchiseFile): Promise<void>
```

Both write atomically: write to `<path>.tmp` then `rename` over the target
(mirror `metadataHandler.ts`'s tmp+rename pattern so a half-written JSON never
lands where the reader or `fs.watch` sees it). `writeFranchiseFile` must `mkdir`
the franchises dir first.

### Public API

```ts
type Fetcher = (anilistId: number) => Promise<{
  self: FranchiseNode | null;
  relations: RawRelation[];
  ok: boolean;
}>;

interface CrawlOpts {
  /** ids to re-fetch even if already fetchedAt>0 (on-click refresh passes [seedId]). */
  forceRefetch?: number[];
  /** test seam — defaults to anilistHandler.fetchRelations. */
  fetch?: Fetcher;
}

/** Crawl the connected component containing seedId, persisting incrementally. */
export async function crawlFranchiseLive(seedId: number, opts?: CrawlOpts): Promise<void>;

/** Crawl every owned series not yet covered by the store (gap-fill). */
export async function crawlLibraryGaps(opts?: { fetch?: Fetcher }): Promise<void>;
```

### `crawlFranchiseLive` algorithm

1. **Lock by component.** Keep a module-level `Map<number, Promise>` keyed by the
   component's current file key. If a crawl for the same key (or seedId) is in
   flight, await/skip it — never crawl the same component concurrently (avoids
   write races + duplicate provisional files). All index writes go through a
   single serialized read-modify-write queue (`let chain = chain.then(...)`).

2. **Locate the component.** `key = index.library[seedId]?.franchise ?? `franchise-${seedId}``.
   `file = (await readFranchiseFile(key)) ?? { rootId: seedId, byId: {} }`.

3. **Seed `closeGraph`.**
   - `seedRelations`: for every `byId[id]` with `fetchedAt > 0` **and** `id` not in
     `forceRefetch`, put `id → its relations`. (This is what implements
     keep+fill-gaps and force-refetch: anything seeded is NOT re-fetched by
     `closeGraph`; anything missing/forced gets fetched.)
   - `seedNodes`: every `byId[id].node` that is non-null, plus a stub for `seedId`
     built from owned metadata (`nodeFromOwnedSeries` equivalent) if it has no node
     yet, so BFS can start even on a brand-new series.

4. **Persisting fetch wrapper** passed to `closeGraph`:
   ```
   fetch(id):
     r = await fetcher(id)              // anilistHandler.fetchRelations by default
     if (!r.ok) return { relations: [], ok: false }   // rate-limited → closeGraph defers
     file.byId[id] = {
       node: r.self ?? file.byId[id]?.node ?? null,
       relations: r.relations,
       fetchedAt: nowMs,                // pass a clock in; see "time" below
     }
     schedulePersist()                  // debounced incremental flush → live render
     return { relations: r.relations, ok: true }
   ```
   `schedulePersist` = ~200ms debounced `writeFranchiseFile(key, file)`. Each flush
   creates/updates the file in the watched dir → 250ms-debounced
   `franchise:store-updated` → renderer re-reads → **tree grows live**.

5. **Run** `await closeGraph({ seedNodes, seedRelations, fetch, nodeCap: 150 })`.

6. **Backfill discovered-but-unfetched nodes.** For every `g.nodes[n]` not already
   in `file.byId`, add `{ node: n, relations: [], fetchedAt: 0 }` (these are nodes
   known only from a relation edge — kept for display, marked stale).

7. **Finalize root + key.** `rootId = g.rootId` (smallest id). `finalKey =
   `franchise-${rootId}``. `file.rootId = rootId`.
   - If `finalKey !== key`: this component's canonical key changed (common — e.g.
     crawling Railgun T 104462 discovers id 6213, so root becomes 6213). Merge into
     any existing `finalKey` file (existing `fetchedAt>0` entries win over stale),
     `writeFranchiseFile(finalKey, mergedFile)`, then delete the old provisional
     `franchise-<seed>.json` if it differs. Cancel the pending debounced flush so it
     can't resurrect the old key.
   - Else `writeFranchiseFile(key, file)` (final flush; cancel debounce).

8. **Update the library index** (through the serialized queue): re-read index, and
   for **every owned series** present in `file.byId`, set
   `library[id] = { node, relations, fetchedAt, franchise: finalKey }`. (Owned =
   present in `metadataHandler.loadMetadata()` with that anilistId.) `writeIndex`.

9. **Release lock.**

### `crawlLibraryGaps`

```
meta = await metadataHandler.loadMetadata()
index = await readIndex()
ownedIds = [ s.anilistId for s in meta if typeof anilistId === 'number' ]
for id of ownedIds:
  entry = index.library[id]
  if (entry && entry.node != null && entry.fetchedAt > 0) continue   // already crawled — keep
  await crawlFranchiseLive(id)   // serialize; the rate limiter already paces AniList
```

Process **sequentially** (the global `RateLimiter` in `anilistHandler` already caps
to ~75 req/min; don't fan out). Skip ids already covered. Crawling one component
will index sibling owned members too, so re-check `index` membership each
iteration to skip ones a prior component already covered. Log a single
state-change line at start (`logger.info('metadata', 'Franchise gap-crawl: N to fill')`)
and one at completion — **no per-node/per-series chatter** (see the activity-log
signal-only rule).

### Time / determinism

`closeGraph` and the crawler need `Date.now()` for `fetchedAt`. Wrap it: accept an
optional `now: () => number` (default `Date.now`) so the verify script can pass a
fixed clock. Do **not** call `Date.now()` at module top level.

## Wiring

### Startup (`src/main/main.ts`, `runStartupCatchUp`)

After the existing `void backfillRelationsForLibrary();` (~line 331), add:
```ts
void crawlLibraryGaps();
```
Fire-and-forget, same as the poster matcher. It must not block startup.

### On-click refresh (`src/main/main.ts`, `franchise:graph` IPC handler ~line 1071)

After computing the graph to return, fire-and-forget a gentle refresh of the
opened series so updates stream in via the watcher — no new IPC channel:
```ts
ipcMain.handle('franchise:graph', async (_event, anilistId: number) => {
  try {
    const graph = await getFranchiseGraph(anilistId);
    void crawlFranchiseLive(anilistId, { forceRefetch: [anilistId] }); // gentle: refetch just this node
    return graph;
  } catch (error) { /* unchanged */ }
});
```
"Gentle/slow" is satisfied by: only the seed node is force-refetched (≈1 query),
the rest is gap-fill, and the shared `RateLimiter` paces everything behind the
background crawl. Guard against hammering: skip if a crawl for this component ran
within the last ~60s (track last-crawl time per key in the module).

## Tests / verify

Add `scripts/verify-franchise-crawler.mjs` (mirror `verify-franchise-graph.mjs`),
run with `bun --bun`. Use an **injected fake fetcher** (no network) + a fixed clock.
Cover:
1. **Brand-new series**: seed 104462 whose fake graph reaches a smaller id 6213 →
   final file key is `franchise-6213`, `byId` contains the whole component, owned
   seed is indexed pointing at `franchise-6213`, provisional `franchise-104462.json`
   is gone.
2. **Keep + fill gaps**: pre-seed a file with one node `fetchedAt>0` and one missing
   → fetcher is called only for the missing id (assert call set), existing node's
   relations untouched.
3. **forceRefetch**: pre-seed seed node `fetchedAt>0`, crawl with
   `forceRefetch:[seed]` → fetcher IS called for seed (refresh path).
4. **Rate-limit defer**: fetcher returns `ok:false` for one id → that id stays
   `deferred`/unfetched, `complete:false`-equivalent, no crash, partial tree persisted.
5. **Incremental persistence**: assert `writeFranchiseFile` (or a flush spy) is
   called more than once during a multi-node crawl (proves live forming).

Add a `package.json` script `"verify:franchise-crawler"`.

## Out of scope / constraints

- **No renderer changes.** The live-forming UX rides entirely on the existing
  file-watcher → `franchise:store-updated` → `getFranchiseGraph` re-read.
- Respect `nodeCap: 150` from `closeGraph`.
- Honor the franchise memory rule: cross-franchise orphans are linked, not embedded
  — but that is already handled by `closeGraph` traversal (CHARACTER/OTHER not
  traversed). Don't change traversal.
- Activity log stays signal-only: state-change lines only, no per-node logging.
- Atomic writes only (tmp+rename); never leave a partial JSON where the watcher fires.
