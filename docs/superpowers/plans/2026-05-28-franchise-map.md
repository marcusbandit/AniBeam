# Franchise Map Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the two-bucket "Related" section on the Series Detail page with a horizontally-scrolling Franchise Map that shows the *entire* franchise as a timeline — current series highlighted in the prequel→sequel spine, sources above, side stories/spin-offs below, alternates as side-branches — built instantly from local metadata and filled in from AniList in the background.

**Architecture:** Pure graph/lane logic in shared + renderer modules (unit-tested via `bun --bun` verify scripts). A main-process service seeds the graph from saved metadata, crawls AniList via the existing rate-limited `anilistHandler.getEnrichment`, and caches the closed graph to a JSON file in `userData`. A renderer hook renders a local seed synchronously, then swaps in the filled graph from IPC. New `franchise/` components render the lanes.

**Tech Stack:** Electron (main/preload/renderer), React + TypeScript, `graphql-request` (existing AniList client), Vite. Tests are standalone `scripts/verify-*.mjs` run with `bun --bun`, asserting via `node:assert/strict` (see `scripts/verify-probe-parser.mjs`). Typecheck: `bun run typecheck` (`tsc --noEmit`).

---

## File Structure

**Create:**
- `src/shared/franchise.ts` — cross-process types (`FranchiseNode`, `FranchiseEdge`, `FranchiseGraph`, `RawRelation`) + pure graph closure (`closeGraph`, `isTraversable`, `TRAVERSABLE`). No Electron imports.
- `src/renderer/components/franchise/laneAssignment.ts` — pure display logic: `relationLane`, `RELATION_LABEL`, `relationLabel`, `compareByYear`.
- `src/renderer/components/franchise/FranchiseNode.tsx` — compact poster-tile for one node.
- `src/renderer/components/franchise/FranchiseMap.tsx` — lane layout + horizontal scroll + center-on-current.
- `src/renderer/components/franchise/index.ts` — barrel export.
- `src/renderer/hooks/useFranchiseGraph.ts` — hybrid consumer (local seed → IPC fill).
- `src/main/services/franchiseGraph.ts` — seed-from-metadata + crawl + cache + closure.
- `scripts/verify-franchise-lanes.mjs` — tests for `laneAssignment.ts`.
- `scripts/verify-franchise-graph.mjs` — tests for `closeGraph`.

**Modify:**
- `src/main/preload.ts` — add `getFranchiseGraph` to the `ElectronAPI` interface and the `exposeInMainWorld` object.
- `src/main/main.ts` — register `ipcMain.handle('franchise:graph', …)`.
- `src/renderer/pages/SeriesDetailPage.tsx` — replace the Related section body with `<FranchiseMap>`; remove the two-bucket logic (`inLibraryRelations`/`externalRelations`/`inLibraryWithSelf`/`sortRelations`/`RELATION_LABEL`/`RELATION_ORDER` and the related JSX).
- `src/renderer/styles/App.css` — Franchise Map styles.
- `package.json` — add `verify:franchise-lanes` and `verify:franchise-graph` scripts.

---

## Task 1: Shared franchise types + graph closure

**Files:**
- Create: `src/shared/franchise.ts`
- Test: `scripts/verify-franchise-graph.mjs`
- Modify: `package.json`

- [ ] **Step 1: Write `src/shared/franchise.ts`**

```ts
// Cross-process franchise-graph types and the pure BFS closure that builds a
// franchise graph from a seed plus an optional async fetcher. No Electron
// imports — safe to use from the main service, the renderer, and verify scripts.

export interface FranchiseNode {
  anilistId: number;
  malId: number | null;
  type: 'ANIME' | 'MANGA' | null;
  format: string | null;
  status: string | null;
  seasonYear: number | null;
  siteUrl: string | null;
  titleRomaji: string | null;
  titleEnglish: string | null;
  poster: string | null;
}

export interface FranchiseEdge {
  /** Source node anilistId. */
  from: number;
  /** Target node anilistId. */
  to: number;
  /** AniList relationType of `to` as seen from `from`. */
  relationType: string;
}

export interface FranchiseGraph {
  /** Smallest anilistId among all nodes — deterministic franchise key. */
  rootId: number;
  nodes: FranchiseNode[];
  edges: FranchiseEdge[];
  /** True when BFS drained without hitting the node cap. */
  complete: boolean;
}

/** A relation edge from a node's perspective, including the target's own info. */
export interface RawRelation {
  relationType: string;
  anilistId: number;
  malId: number | null;
  type: 'ANIME' | 'MANGA' | null;
  format: string | null;
  status: string | null;
  seasonYear: number | null;
  siteUrl: string | null;
  titleRomaji: string | null;
  titleEnglish: string | null;
  poster: string | null;
}

/** relationTypes whose edges we follow when crawling. CHARACTER and OTHER are
 *  excluded so cameos / loose links don't drag in unrelated franchises. */
export const TRAVERSABLE = new Set<string>([
  'PREQUEL', 'SEQUEL', 'SIDE_STORY', 'SPIN_OFF', 'ALTERNATIVE',
  'PARENT', 'CONTAINS', 'SUMMARY', 'COMPILATION', 'SOURCE', 'ADAPTATION',
]);

export function isTraversable(relationType: string): boolean {
  return TRAVERSABLE.has(relationType);
}

function nodeFromRelation(r: RawRelation): FranchiseNode {
  return {
    anilistId: r.anilistId,
    malId: r.malId,
    type: r.type,
    format: r.format,
    status: r.status,
    seasonYear: r.seasonYear,
    siteUrl: r.siteUrl,
    titleRomaji: r.titleRomaji,
    titleEnglish: r.titleEnglish,
    poster: r.poster,
  };
}

export type RelationsFetcher = (anilistId: number) => Promise<RawRelation[] | null>;

export interface CloseGraphOptions {
  /** Known nodes (current series + owned series) with their own info. */
  seedNodes: FranchiseNode[];
  /** anilistId → that node's relations, for nodes we already have locally. */
  seedRelations: Map<number, RawRelation[]>;
  /** Optional async fetcher for relations of nodes not in seedRelations. */
  fetch?: RelationsFetcher;
  /** Stop discovering new nodes past this many. Default 150. */
  nodeCap?: number;
}

/**
 * BFS the franchise graph. CHARACTER edges are dropped entirely; OTHER edges
 * are kept for display but never traversed. Nodes dedup by anilistId; a node
 * reached by multiple edges appears once and accumulates all its edges.
 */
export async function closeGraph(opts: CloseGraphOptions): Promise<FranchiseGraph> {
  const nodeCap = opts.nodeCap ?? 150;
  const nodes = new Map<number, FranchiseNode>();
  const edges: FranchiseEdge[] = [];
  const expanded = new Set<number>();
  const queue: number[] = [];

  for (const n of opts.seedNodes) {
    if (!nodes.has(n.anilistId)) nodes.set(n.anilistId, n);
    queue.push(n.anilistId);
  }

  let hitCap = false;
  while (queue.length > 0) {
    const id = queue.shift()!;
    if (expanded.has(id)) continue;
    expanded.add(id);

    let relations = opts.seedRelations.get(id) ?? null;
    if (relations == null && opts.fetch) relations = await opts.fetch(id);
    if (relations == null) continue;

    for (const r of relations) {
      if (r.relationType === 'CHARACTER') continue; // dropped from the map
      if (!nodes.has(r.anilistId)) {
        if (nodes.size >= nodeCap) { hitCap = true; continue; }
        nodes.set(r.anilistId, nodeFromRelation(r));
      }
      edges.push({ from: id, to: r.anilistId, relationType: r.relationType });
      if (isTraversable(r.relationType) && !expanded.has(r.anilistId)) {
        queue.push(r.anilistId);
      }
    }
  }

  const ids = [...nodes.keys()];
  const rootId = ids.length ? Math.min(...ids) : 0;
  return { rootId, nodes: [...nodes.values()], edges, complete: !hitCap };
}
```

- [ ] **Step 2: Write the failing test `scripts/verify-franchise-graph.mjs`**

```js
import assert from 'node:assert/strict';
import { closeGraph, isTraversable } from '../src/shared/franchise.ts';

const mk = (id, relationType, over = {}) => ({
  relationType, anilistId: id, malId: null, type: 'ANIME', format: 'TV',
  status: 'FINISHED', seasonYear: 2000 + id, siteUrl: null,
  titleRomaji: `T${id}`, titleEnglish: null, poster: null, ...over,
});
const node = (id) => ({
  anilistId: id, malId: null, type: 'ANIME', format: 'TV', status: 'FINISHED',
  seasonYear: 2000 + id, siteUrl: null, titleRomaji: `T${id}`, titleEnglish: null, poster: null,
});

// isTraversable excludes CHARACTER and OTHER
assert.equal(isTraversable('SEQUEL'), true);
assert.equal(isTraversable('CHARACTER'), false);
assert.equal(isTraversable('OTHER'), false);

// Transitive closure through a non-seed node via fetch
const seedRelations = new Map([[1, [mk(2, 'SEQUEL')]]]);
const fetched = new Map([[2, [mk(1, 'PREQUEL'), mk(3, 'SEQUEL')]]]);
const g1 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations,
  fetch: async (id) => fetched.get(id) ?? [],
});
assert.deepEqual(g1.nodes.map((n) => n.anilistId).sort((a, b) => a - b), [1, 2, 3]);
assert.equal(g1.rootId, 1);
assert.equal(g1.complete, true);

// CHARACTER edges are dropped entirely (node 9 must not appear)
const g2 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations: new Map([[1, [mk(9, 'CHARACTER')]]]),
});
assert.deepEqual(g2.nodes.map((n) => n.anilistId), [1]);

// OTHER nodes are displayed but not traversed (node 5 appears, its rels are not followed)
const g3 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations: new Map([[1, [mk(5, 'OTHER')]]]),
  fetch: async () => { throw new Error('should not fetch through OTHER'); },
});
assert.deepEqual(g3.nodes.map((n) => n.anilistId).sort((a, b) => a - b), [1, 5]);

// nodeCap halts discovery and marks incomplete
const g4 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations: new Map([[1, [mk(2, 'SEQUEL'), mk(3, 'SEQUEL'), mk(4, 'SEQUEL')]]]),
  nodeCap: 2,
});
assert.equal(g4.nodes.length, 2);
assert.equal(g4.complete, false);

console.log('OK: franchise graph closure');
```

- [ ] **Step 3: Add the script to `package.json`** (in `"scripts"`, after `verify:folder`):

```json
    "verify:franchise-graph": "bun --bun scripts/verify-franchise-graph.mjs",
```

- [ ] **Step 4: Run the test — expect PASS**

Run: `bun run verify:franchise-graph`
Expected: `OK: franchise graph closure`

- [ ] **Step 5: Typecheck**

Run: `bun run typecheck`
Expected: no output, exit 0.

- [ ] **Step 6: Commit**

```bash
git add src/shared/franchise.ts scripts/verify-franchise-graph.mjs package.json
git commit -m "feat(franchise): shared graph types + BFS closure"
```

---

## Task 2: Lane assignment + label + sort (renderer pure logic)

**Files:**
- Create: `src/renderer/components/franchise/laneAssignment.ts`
- Test: `scripts/verify-franchise-lanes.mjs`
- Modify: `package.json`

- [ ] **Step 1: Write `src/renderer/components/franchise/laneAssignment.ts`**

```ts
// Pure display logic: map a relation onto a lane, label it, and order spine
// nodes chronologically. Used by FranchiseMap and exercised by a verify script.

export type FranchiseLane = 'spine' | 'top' | 'bottom' | 'sidebranch' | 'excluded';

/** Formats treated as upstream source material (not screen adaptations). */
const PRINT_FORMATS = new Set(['MANGA', 'NOVEL', 'LIGHT_NOVEL', 'ONE_SHOT', 'VISUAL_NOVEL']);

/**
 * Lane for a related node, given the edge's relationType and the *target's*
 * media type/format. SOURCE/PARENT sit above; PREQUEL/SEQUEL form the spine;
 * ALTERNATIVE branches; CHARACTER is excluded; everything else hangs below.
 * ADAPTATION direction is resolved by media type: an adaptation that is print
 * material is upstream (top); a screen adaptation is downstream (bottom).
 */
export function relationLane(
  relationType: string,
  targetType: 'ANIME' | 'MANGA' | null,
  targetFormat: string | null,
): FranchiseLane {
  switch (relationType) {
    case 'PREQUEL':
    case 'SEQUEL':
      return 'spine';
    case 'ALTERNATIVE':
      return 'sidebranch';
    case 'CHARACTER':
      return 'excluded';
    case 'SOURCE':
    case 'PARENT':
      return 'top';
    case 'ADAPTATION': {
      const isPrint = targetFormat ? PRINT_FORMATS.has(targetFormat) : targetType === 'MANGA';
      return isPrint ? 'top' : 'bottom';
    }
    default:
      // SIDE_STORY, SPIN_OFF, SUMMARY, COMPILATION, CONTAINS, OTHER, unknown
      return 'bottom';
  }
}

export const RELATION_LABEL: Record<string, string> = {
  SEQUEL: 'Sequel',
  PREQUEL: 'Prequel',
  PARENT: 'Parent story',
  SIDE_STORY: 'Side story',
  SUMMARY: 'Summary',
  ALTERNATIVE: 'Alternative',
  SPIN_OFF: 'Spin-off',
  COMPILATION: 'Compilation',
  SOURCE: 'Source',
  ADAPTATION: 'Adaptation',
  CHARACTER: 'Shared characters',
  CONTAINS: 'Contains',
  OTHER: 'Other',
};

export function relationLabel(relationType: string): string {
  return RELATION_LABEL[relationType] ?? relationType.replace(/_/g, ' ').toLowerCase();
}

/** Chronological compare by seasonYear; nulls sort last. */
export function compareByYear(
  a: { seasonYear: number | null },
  b: { seasonYear: number | null },
): number {
  const ay = a.seasonYear ?? Number.POSITIVE_INFINITY;
  const by = b.seasonYear ?? Number.POSITIVE_INFINITY;
  return ay - by;
}
```

- [ ] **Step 2: Write the test `scripts/verify-franchise-lanes.mjs`**

```js
import assert from 'node:assert/strict';
import { relationLane, relationLabel, compareByYear } from '../src/renderer/components/franchise/laneAssignment.ts';

// spine
assert.equal(relationLane('PREQUEL', 'ANIME', 'TV'), 'spine');
assert.equal(relationLane('SEQUEL', 'ANIME', 'TV'), 'spine');
// upstream
assert.equal(relationLane('SOURCE', 'MANGA', 'MANGA'), 'top');
assert.equal(relationLane('PARENT', 'ANIME', 'TV'), 'top');
// ADAPTATION direction by media type
assert.equal(relationLane('ADAPTATION', 'MANGA', 'MANGA'), 'top');   // print = upstream
assert.equal(relationLane('ADAPTATION', 'ANIME', 'TV'), 'bottom');   // screen = downstream
// branch + excluded
assert.equal(relationLane('ALTERNATIVE', 'ANIME', 'TV'), 'sidebranch');
assert.equal(relationLane('CHARACTER', 'ANIME', 'TV'), 'excluded');
// downstream defaults
assert.equal(relationLane('SIDE_STORY', 'ANIME', 'OVA'), 'bottom');
assert.equal(relationLane('SPIN_OFF', 'ANIME', 'TV'), 'bottom');
assert.equal(relationLane('OTHER', 'ANIME', 'TV'), 'bottom');

// labels
assert.equal(relationLabel('SIDE_STORY'), 'Side story');
assert.equal(relationLabel('SOURCE'), 'Source');
assert.equal(relationLabel('CHARACTER'), 'Shared characters');
assert.equal(relationLabel('WEIRD_NEW_TYPE'), 'weird new type');

// chronological sort, nulls last
const sorted = [{ seasonYear: 2010 }, { seasonYear: null }, { seasonYear: 2005 }].sort(compareByYear);
assert.deepEqual(sorted.map((s) => s.seasonYear), [2005, 2010, null]);

console.log('OK: franchise lanes');
```

- [ ] **Step 3: Add the script to `package.json`** (after `verify:franchise-graph`):

```json
    "verify:franchise-lanes": "bun --bun scripts/verify-franchise-lanes.mjs",
```

- [ ] **Step 4: Run the test — expect PASS**

Run: `bun run verify:franchise-lanes`
Expected: `OK: franchise lanes`

- [ ] **Step 5: Typecheck**

Run: `bun run typecheck`
Expected: exit 0.

- [ ] **Step 6: Commit**

```bash
git add src/renderer/components/franchise/laneAssignment.ts scripts/verify-franchise-lanes.mjs package.json
git commit -m "feat(franchise): lane assignment, labels, chronological sort"
```

---

## Task 3: Main-process service — seed, crawl, cache

**Files:**
- Create: `src/main/services/franchiseGraph.ts`

Reference patterns (verified): `src/main/services/trackerStore.ts` (userData JSON store), `src/main/handlers/anilistHandler.ts:882` (`getEnrichment`), `src/main/handlers/metadataHandler.ts:112` (`metadataHandler.loadMetadata(): Promise<Record<string, unknown>>`, default export), `src/main/services/logger.ts` (`export { logger }`, named).

- [ ] **Step 1: Write `src/main/services/franchiseGraph.ts`**

```ts
import { app } from 'electron';
import { readFile, writeFile, mkdir } from 'fs/promises';
import { join } from 'path';
import anilistHandler from '../handlers/anilistHandler';
import metadataHandler from '../handlers/metadataHandler';
import { logger } from './logger';
import {
  closeGraph,
  type FranchiseGraph,
  type FranchiseNode,
  type RawRelation,
} from '../../shared/franchise';

const CACHE_FILE = 'franchiseGraphCache.json';
const TTL_MS = 14 * 24 * 60 * 60 * 1000; // 14 days

interface CacheEntry { graph: FranchiseGraph; fetchedAt: number; }
interface CacheShape {
  graphs: Record<string, CacheEntry>;   // rootId → entry
  index: Record<string, number>;        // anilistId → rootId
}

function cachePath(): string {
  return join(app.getPath('userData'), CACHE_FILE);
}

async function readCache(): Promise<CacheShape> {
  try {
    const raw = await readFile(cachePath(), 'utf-8');
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === 'object' && parsed.graphs && parsed.index) return parsed as CacheShape;
  } catch { /* missing/corrupt → empty */ }
  return { graphs: {}, index: {} };
}

async function writeCache(cache: CacheShape): Promise<void> {
  await mkdir(app.getPath('userData'), { recursive: true });
  await writeFile(cachePath(), JSON.stringify(cache), 'utf-8');
}

// SeriesMetadata-like shape we read from the saved store.
interface SavedSeries {
  anilistId?: number;
  malId?: number | null;
  type?: 'series' | 'movie';
  format?: string;
  status?: string;
  seasonYear?: number | null;
  titleRomaji?: string;
  titleEnglish?: string | null;
  poster?: string | null;
  relations?: RawRelation[];
}

/** Build seedNodes + seedRelations from every owned series that has an anilistId. */
function buildSeed(meta: Record<string, SavedSeries>): {
  seedNodes: FranchiseNode[];
  seedRelations: Map<number, RawRelation[]>;
} {
  const seedNodes: FranchiseNode[] = [];
  const seedRelations = new Map<number, RawRelation[]>();
  for (const s of Object.values(meta)) {
    if (typeof s.anilistId !== 'number') continue;
    seedNodes.push({
      anilistId: s.anilistId,
      malId: s.malId ?? null,
      type: s.type === 'movie' ? 'ANIME' : 'ANIME',
      format: s.format ?? null,
      status: s.status ?? null,
      seasonYear: s.seasonYear ?? null,
      siteUrl: null,
      titleRomaji: s.titleRomaji ?? null,
      titleEnglish: s.titleEnglish ?? null,
      poster: s.poster ?? null,
    });
    if (Array.isArray(s.relations)) seedRelations.set(s.anilistId, s.relations);
  }
  return { seedNodes, seedRelations };
}

/**
 * Return the closed, filled franchise graph for the given AniList id.
 * Serves a fresh cached graph when one exists (keyed by franchise root via the
 * member index); otherwise seeds from owned metadata, crawls AniList through
 * franchise edges, caches the result, and returns it.
 */
export async function getFranchiseGraph(anilistId: number): Promise<FranchiseGraph> {
  const cache = await readCache();
  const cachedRoot = cache.index[String(anilistId)];
  if (cachedRoot != null) {
    const entry = cache.graphs[String(cachedRoot)];
    if (entry && Date.now() - entry.fetchedAt < TTL_MS) return entry.graph;
  }

  const meta = (await metadataHandler.loadMetadata()) as Record<string, SavedSeries>;
  const { seedNodes, seedRelations } = buildSeed(meta);
  // Ensure the current node is present even if it has no saved relations.
  if (!seedNodes.some((n) => n.anilistId === anilistId)) {
    seedNodes.push({
      anilistId, malId: null, type: 'ANIME', format: null, status: null,
      seasonYear: null, siteUrl: null, titleRomaji: null, titleEnglish: null, poster: null,
    });
  }

  const graph = await closeGraph({
    seedNodes,
    seedRelations,
    fetch: async (id) => {
      const bundle = await anilistHandler.getEnrichment({ anilistId: id });
      return bundle.relations as RawRelation[];
    },
  });

  // Persist under rootId and index every member id → rootId.
  cache.graphs[String(graph.rootId)] = { graph, fetchedAt: Date.now() };
  for (const n of graph.nodes) cache.index[String(n.anilistId)] = graph.rootId;
  try { await writeCache(cache); } catch (e) {
    logger.warn('metadata', `franchise graph cache write failed: ${(e as Error).message}`);
  }
  return graph;
}
```

- [ ] **Step 2: Typecheck**

Run: `bun run typecheck`
Expected: exit 0. If `metadataHandler` is not a default export in your tree, check `src/main/handlers/metadataHandler.ts:191` and import accordingly. Iterate until `tsc --noEmit` is clean.

- [ ] **Step 3: Commit**

```bash
git add src/main/services/franchiseGraph.ts
git commit -m "feat(franchise): main service — seed from metadata, crawl, cache"
```

---

## Task 4: IPC + preload wiring

**Files:**
- Modify: `src/main/main.ts`, `src/main/preload.ts`

- [ ] **Step 1: Register the IPC handler in `src/main/main.ts`**

Add near the other AniList handlers (after `ipcMain.handle('anilist:search', …)` around line 1056). First add the import at the top alongside the other service imports:

```ts
import { getFranchiseGraph } from './services/franchiseGraph';
```

Then the handler:

```ts
ipcMain.handle('franchise:graph', async (_event, anilistId: number) => {
  if (typeof anilistId !== 'number' || !Number.isFinite(anilistId)) return null;
  try {
    return await getFranchiseGraph(anilistId);
  } catch (error) {
    logger.error('metadata', `franchise:graph failed: ${(error as Error).message}`);
    return null;
  }
});
```

(Match the existing `logger` symbol used in `main.ts`.)

- [ ] **Step 2: Add the method to the `ElectronAPI` interface in `src/main/preload.ts`**

Add an import of the graph type near the top of `preload.ts`:

```ts
import type { FranchiseGraph } from '../shared/franchise';
```

Add to the `ElectronAPI` interface (near `searchAnilist`, around line 100):

```ts
  getFranchiseGraph: (anilistId: number) => Promise<FranchiseGraph | null>;
```

- [ ] **Step 3: Implement it in the `contextBridge.exposeInMainWorld('electronAPI', { … })` object**

```ts
  getFranchiseGraph: (anilistId: number) => ipcRenderer.invoke('franchise:graph', anilistId),
```

- [ ] **Step 4: Re-export the type for the renderer in `src/types/electron.d.ts`**

Add `FranchiseGraph` (and `FranchiseNode`, `FranchiseEdge` if useful) to the `export type { … }` block — these come from `../shared/franchise`, so also add an `export type { FranchiseGraph, FranchiseNode, FranchiseEdge } from '../shared/franchise';` line if the file doesn't already re-export shared types.

- [ ] **Step 5: Typecheck**

Run: `bun run typecheck`
Expected: exit 0.

- [ ] **Step 6: Commit**

```bash
git add src/main/main.ts src/main/preload.ts src/types/electron.d.ts
git commit -m "feat(franchise): IPC + preload wiring for franchise:graph"
```

---

## Task 5: useFranchiseGraph hook (hybrid local → fill)

**Files:**
- Create: `src/renderer/hooks/useFranchiseGraph.ts`

- [ ] **Step 1: Write `src/renderer/hooks/useFranchiseGraph.ts`**

```ts
import { useEffect, useMemo, useRef, useState } from 'react';
import type { SeriesMetadata } from './useMetadata';
import {
  closeGraph,
  type FranchiseGraph,
  type FranchiseNode,
  type RawRelation,
} from '../../shared/franchise';

/** Build seedNodes + seedRelations from the in-memory owned-metadata map. */
function buildLocalSeed(allMeta: Record<string, SeriesMetadata>): {
  seedNodes: FranchiseNode[];
  seedRelations: Map<number, RawRelation[]>;
} {
  const seedNodes: FranchiseNode[] = [];
  const seedRelations = new Map<number, RawRelation[]>();
  for (const s of Object.values(allMeta)) {
    if (typeof s.anilistId !== 'number') continue;
    seedNodes.push({
      anilistId: s.anilistId,
      malId: s.malId ?? null,
      type: 'ANIME',
      format: s.format ?? null,
      status: s.status ?? null,
      seasonYear: s.seasonYear ?? null,
      siteUrl: null,
      titleRomaji: s.titleRomaji ?? null,
      titleEnglish: s.titleEnglish ?? null,
      poster: s.posterLocal ?? s.poster ?? null,
    });
    if (Array.isArray(s.relations)) {
      seedRelations.set(s.anilistId, s.relations as unknown as RawRelation[]);
    }
  }
  return { seedNodes, seedRelations };
}

export interface UseFranchiseGraph {
  graph: FranchiseGraph | null;
  /** True while the background AniList fill is in flight. */
  filling: boolean;
}

/**
 * Hybrid franchise-graph consumer. Synchronously builds a local graph from the
 * owned-metadata map for instant render, then requests the closed+cached graph
 * from the main process and swaps it in when it arrives.
 */
export function useFranchiseGraph(
  currentAnilistId: number | null | undefined,
  allMeta: Record<string, SeriesMetadata>,
): UseFranchiseGraph {
  const [filled, setFilled] = useState<FranchiseGraph | null>(null);
  const [filling, setFilling] = useState(false);
  const [localGraph, setLocalGraph] = useState<FranchiseGraph | null>(null);
  const reqIdRef = useRef(0);

  // Local seed — recomputed when the current series or metadata changes.
  useEffect(() => {
    let cancelled = false;
    if (currentAnilistId == null) { setLocalGraph(null); return; }
    const { seedNodes, seedRelations } = buildLocalSeed(allMeta);
    if (!seedNodes.some((n) => n.anilistId === currentAnilistId)) {
      seedNodes.push({
        anilistId: currentAnilistId, malId: null, type: 'ANIME', format: null,
        status: null, seasonYear: null, siteUrl: null, titleRomaji: null,
        titleEnglish: null, poster: null,
      });
    }
    void closeGraph({ seedNodes, seedRelations }).then((g) => {
      if (!cancelled) setLocalGraph(g);
    });
    return () => { cancelled = true; };
  }, [currentAnilistId, allMeta]);

  // Background fill from AniList (cached in main).
  useEffect(() => {
    setFilled(null);
    if (currentAnilistId == null) return;
    const myReq = ++reqIdRef.current;
    setFilling(true);
    void window.electronAPI
      .getFranchiseGraph(currentAnilistId)
      .then((g) => { if (reqIdRef.current === myReq) setFilled(g); })
      .finally(() => { if (reqIdRef.current === myReq) setFilling(false); });
  }, [currentAnilistId]);

  // Prefer the filled graph once present; fall back to the local seed.
  const graph = useMemo(() => filled ?? localGraph, [filled, localGraph]);
  return { graph, filling };
}
```

- [ ] **Step 2: Typecheck**

Run: `bun run typecheck`
Expected: exit 0. (If `posterLocal` isn't on `SeriesMetadata`, it is — see `useMetadata.ts:13`.)

- [ ] **Step 3: Commit**

```bash
git add src/renderer/hooks/useFranchiseGraph.ts
git commit -m "feat(franchise): hybrid useFranchiseGraph hook"
```

---

## Task 6: FranchiseNode tile component

**Files:**
- Create: `src/renderer/components/franchise/FranchiseNode.tsx`

- [ ] **Step 1: Write `src/renderer/components/franchise/FranchiseNode.tsx`**

```tsx
import { Tv, Film } from 'lucide-react';
import type { ReactNode } from 'react';
import type { FranchiseNode as FranchiseNodeData } from '../../../shared/franchise';
import { Card, Pill, Tooltip } from '../primitives';

export interface FranchiseNodeProps {
  node: FranchiseNodeData;
  /** Display title (already resolved through the user's title-language pref). */
  title: string;
  /** Relation label shown above the title (e.g. "Sequel", "Source"). */
  relationLabel: string | null;
  /** True for the series whose page we're on — gets the highlight ring. */
  isCurrent: boolean;
  /** seriesId if the user owns this entry, else undefined. */
  ownedId?: string;
  /** Optional tracker-list status marker node. */
  statusMarker?: ReactNode;
  onOpenInApp: (seriesId: string) => void;
  onOpenExternal: (node: FranchiseNodeData) => void;
  /** AniList brand icon for the external pill. */
  anilistIcon: ReactNode;
}

export function FranchiseNode(props: FranchiseNodeProps) {
  const { node, title, relationLabel, isCurrent, ownedId, statusMarker } = props;
  const owned = ownedId != null;
  const isManga = node.type === 'MANGA';

  const handleClick = () => {
    if (isCurrent) return;
    if (owned) props.onOpenInApp(ownedId!);
    else props.onOpenExternal(node);
  };

  const tooltip = isCurrent
    ? undefined
    : owned ? `Open ${title} in your library` : `Open ${title} on AniList`;

  return (
    <Card
      variant={owned || isCurrent ? 'internal' : 'external'}
      noLift={isCurrent}
      onClick={isCurrent ? undefined : handleClick}
      tooltip={tooltip}
      aria-current={isCurrent ? 'page' : undefined}
      data-format={node.format ?? ''}
      className="franchise-node"
      data-current={isCurrent ? 'true' : undefined}
    >
      <div className="relation-card-poster">
        {node.poster ? (
          <img src={node.poster} alt={title} loading="lazy" decoding="async" />
        ) : (
          <div className="relation-card-poster-empty">
            {isManga ? <Film size={28} /> : <Tv size={28} />}
          </div>
        )}
        <span aria-hidden="true">
          {isCurrent ? (
            <Pill tone="muted">You are here</Pill>
          ) : owned ? (
            <Pill tone="teal">In library</Pill>
          ) : (
            <Pill tone="accent">{props.anilistIcon} AniList</Pill>
          )}
        </span>
      </div>
      <div className="relation-card-body">
        {relationLabel && <div className="relation-card-type">{relationLabel}</div>}
        <div className="relation-card-title">{title}</div>
        <div className="relation-card-meta">
          {node.format && (
            <span className="relation-card-format" data-format={node.format}>{node.format}</span>
          )}
          {node.seasonYear && <span>{node.seasonYear}</span>}
        </div>
      </div>
      {statusMarker}
    </Card>
  );
}
```

Note: this reuses the existing `relation-card-*` CSS classes so it inherits current styling; Task 8 adds `.franchise-node` / `[data-current]` overrides for the compact tile + highlight ring. `Tooltip` is imported for future per-tile use; if unused after wiring, remove it to keep the lint clean.

- [ ] **Step 2: Typecheck**

Run: `bun run typecheck`
Expected: exit 0. (If `Card` doesn't accept `className`/`noLift`/`data-*`, check `src/renderer/components/primitives/Card.tsx` and use the props it does expose — it's already used with `noLift`, `data-format`, `variant`, `onClick`, `tooltip`, `aria-current` in `SeriesDetailPage.tsx`.)

- [ ] **Step 3: Commit**

```bash
git add src/renderer/components/franchise/FranchiseNode.tsx
git commit -m "feat(franchise): compact FranchiseNode tile"
```

---

## Task 7: FranchiseMap layout component

**Files:**
- Create: `src/renderer/components/franchise/FranchiseMap.tsx`, `src/renderer/components/franchise/index.ts`

This component turns a `FranchiseGraph` into lanes. The v1 layout:
- **Spine** = nodes connected to the rest via `PREQUEL`/`SEQUEL` edges (plus the current node), sorted by `compareByYear`, rendered left-to-right.
- For every non-spine node, find the edge that connects it to a spine node (or any node) and place it in **top**/**bottom**/**sidebranch** by `relationLane(edge.relationType, node.type, node.format)`.
- Top and bottom lanes render as their own horizontal rows above/below the spine, each node ordered by `compareByYear`. (Column-perfect alignment + drawn connectors are a later visual refinement; v1 ships three aligned rows + the spine rail, which is enough to verify the data model in dev.)

- [ ] **Step 1: Write `src/renderer/components/franchise/FranchiseMap.tsx`**

```tsx
import { useEffect, useMemo, useRef } from 'react';
import type { ReactNode } from 'react';
import type { FranchiseGraph, FranchiseNode as NodeData } from '../../../shared/franchise';
import { compareByYear, relationLabel, relationLane, type FranchiseLane } from './laneAssignment';
import { FranchiseNode } from './FranchiseNode';

export interface FranchiseMapProps {
  graph: FranchiseGraph;
  currentAnilistId: number;
  /** Resolve a node to an owned seriesId, if any. */
  resolveOwnedId: (node: NodeData) => string | undefined;
  pickTitle: (n: { titleRomaji: string | null; titleEnglish: string | null }) => string;
  onOpenInApp: (seriesId: string) => void;
  onOpenExternal: (node: NodeData) => void;
  statusMarkerFor: (node: NodeData) => ReactNode;
  anilistIcon: ReactNode;
}

interface Placed { node: NodeData; lane: FranchiseLane; relationType: string | null; }

/** Assign each node a lane + the relationType that connected it. The current
 *  node is forced onto the spine. A node's lane comes from the first edge that
 *  points *to* it (relationType is from the source's perspective, which is what
 *  the user reads: "this is a Side story of the thing it hangs off"). */
function placeNodes(graph: FranchiseGraph, currentId: number): Placed[] {
  const incoming = new Map<number, string>();
  for (const e of graph.edges) {
    if (!incoming.has(e.to)) incoming.set(e.to, e.relationType);
  }
  return graph.nodes.map((node) => {
    if (node.anilistId === currentId) return { node, lane: 'spine' as const, relationType: null };
    const rt = incoming.get(node.anilistId) ?? 'OTHER';
    const lane = relationLane(rt, node.type, node.format);
    return { node, lane, relationType: rt };
  });
}

export function FranchiseMap(props: FranchiseMapProps) {
  const { graph, currentAnilistId } = props;
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const currentRef = useRef<HTMLDivElement | null>(null);

  const lanes = useMemo(() => {
    const placed = placeNodes(graph, currentAnilistId).filter((p) => p.lane !== 'excluded');
    const byLane = (lane: FranchiseLane) =>
      placed.filter((p) => p.lane === lane).sort((a, b) => compareByYear(a.node, b.node));
    return {
      top: byLane('top'),
      spine: byLane('spine'),
      bottom: byLane('bottom'),
      branch: byLane('sidebranch'),
    };
  }, [graph, currentAnilistId]);

  // Center the current node on mount / when the graph changes.
  useEffect(() => {
    const c = currentRef.current;
    const s = scrollRef.current;
    if (!c || !s) return;
    s.scrollLeft = c.offsetLeft - s.clientWidth / 2 + c.clientWidth / 2;
  }, [graph]);

  const renderTile = (p: Placed) => {
    const isCurrent = p.node.anilistId === currentAnilistId;
    return (
      <div
        key={p.node.anilistId}
        ref={isCurrent ? currentRef : undefined}
        className="franchise-cell"
      >
        <FranchiseNode
          node={p.node}
          title={props.pickTitle(p.node)}
          relationLabel={isCurrent ? 'Currently viewing' : (p.relationType ? relationLabel(p.relationType) : null)}
          isCurrent={isCurrent}
          ownedId={props.resolveOwnedId(p.node)}
          statusMarker={isCurrent ? null : props.statusMarkerFor(p.node)}
          onOpenInApp={props.onOpenInApp}
          onOpenExternal={props.onOpenExternal}
          anilistIcon={props.anilistIcon}
        />
      </div>
    );
  };

  const spineRow = [...lanes.spine, ...lanes.branch];

  return (
    <div className="franchise-map" ref={scrollRef}>
      <div className="franchise-map__inner">
        {lanes.top.length > 0 && (
          <div className="franchise-lane franchise-lane--top">{lanes.top.map(renderTile)}</div>
        )}
        <div className="franchise-lane franchise-lane--spine">
          {spineRow.length > 1 && <div className="franchise-rail" aria-hidden="true" />}
          {spineRow.map(renderTile)}
        </div>
        {lanes.bottom.length > 0 && (
          <div className="franchise-lane franchise-lane--bottom">{lanes.bottom.map(renderTile)}</div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Write `src/renderer/components/franchise/index.ts`**

```ts
export { FranchiseMap } from './FranchiseMap';
export type { FranchiseMapProps } from './FranchiseMap';
export { FranchiseNode } from './FranchiseNode';
```

- [ ] **Step 3: Typecheck**

Run: `bun run typecheck`
Expected: exit 0.

- [ ] **Step 4: Commit**

```bash
git add src/renderer/components/franchise/FranchiseMap.tsx src/renderer/components/franchise/index.ts
git commit -m "feat(franchise): FranchiseMap lane layout + center-on-current"
```

---

## Task 8: Franchise Map styles

**Files:**
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 1: Append Franchise Map styles to `src/renderer/styles/App.css`**

Use existing design tokens (CSS variables already used elsewhere in this file — match their names by checking the file first with `rg -n "var\(--" src/renderer/styles/App.css | head`). Concrete starting styles:

```css
/* ── Franchise Map ───────────────────────────────────────────── */
.franchise-map {
  overflow-x: auto;
  overflow-y: hidden;
  padding: 8px 4px 16px;
}
.franchise-map__inner {
  display: flex;
  flex-direction: column;
  gap: 24px;
  min-width: max-content;
  align-items: stretch;
}
.franchise-lane {
  display: flex;
  flex-direction: row;
  gap: 16px;
  align-items: flex-start;
}
.franchise-lane--spine {
  position: relative;
  align-items: center;
}
.franchise-rail {
  position: absolute;
  left: 0; right: 0;
  top: 50%;
  height: 2px;
  background: var(--accent-teal, #2dd4bf);
  opacity: 0.35;
  z-index: 0;
}
.franchise-cell { position: relative; z-index: 1; }

/* compact tile: shrink the reused relation-card shell */
.franchise-node { width: 150px; }
.franchise-node .relation-card-poster img,
.franchise-node .relation-card-poster-empty { height: 200px; }

/* current-node highlight ring */
.franchise-node[data-current="true"] {
  outline: 2px solid var(--accent-teal, #2dd4bf);
  outline-offset: 2px;
  border-radius: var(--radius-lg, 16px);
}
```

- [ ] **Step 2: Verify it compiles (Vite picks up CSS at runtime; just typecheck the project)**

Run: `bun run typecheck`
Expected: exit 0 (CSS isn't typechecked, but this confirms nothing else broke).

- [ ] **Step 3: Commit**

```bash
git add src/renderer/styles/App.css
git commit -m "feat(franchise): Franchise Map styles"
```

---

## Task 9: Integrate into SeriesDetailPage; remove the two-bucket Related logic

**Files:**
- Modify: `src/renderer/pages/SeriesDetailPage.tsx`

- [ ] **Step 1: Add imports**

At the top with the other component imports:

```ts
import { FranchiseMap } from "../components/franchise";
import { useFranchiseGraph } from "../hooks/useFranchiseGraph";
import type { FranchiseNode } from "../../shared/franchise";
```

- [ ] **Step 2: Call the hook inside the component**

After `meta`/`allMeta` are available (near the existing `ownedByExternalId` memo, ~line 318):

```ts
  const { graph: franchiseGraph } = useFranchiseGraph(meta?.anilistId, allMeta);

  // Resolve a graph node to an owned seriesId (ANIME only — matches resolveOwnedId).
  const resolveOwnedNode = useCallback((node: FranchiseNode): string | undefined => {
    if (node.type === "MANGA") return undefined;
    return (node.anilistId != null ? ownedByExternalId.byAnilist.get(node.anilistId) : undefined)
      ?? (node.malId != null ? ownedByExternalId.byMal.get(node.malId) : undefined);
  }, [ownedByExternalId]);

  const openExternalNode = useCallback((node: FranchiseNode) => {
    const url = node.siteUrl
      ?? (node.type === "MANGA"
        ? `https://anilist.co/manga/${node.anilistId}`
        : `https://anilist.co/anime/${node.anilistId}`);
    if (url) void window.electronAPI.openExternal(url);
  }, []);
```

- [ ] **Step 3: Replace the Related `<Section>` body**

Find the `<Section title="Related" …>` block (around line 1164) and replace its entire contents (both the `inLibraryRelations` group and the `externalRelations` group) with:

```tsx
        <Section title="Related" count={franchiseGraph ? franchiseGraph.nodes.length - 1 : 0}>
          {franchiseGraph && franchiseGraph.nodes.length > 1 && meta?.anilistId != null && (
            <FranchiseMap
              graph={franchiseGraph}
              currentAnilistId={meta.anilistId}
              resolveOwnedId={resolveOwnedNode}
              pickTitle={(n) => pickTitle({
                titleRomaji: n.titleRomaji,
                titleEnglish: n.titleEnglish,
                folderName: n.titleRomaji ?? n.titleEnglish ?? "Untitled",
              })}
              onOpenInApp={(id) => navigate(`/series/${encodeURIComponent(id)}`)}
              onOpenExternal={openExternalNode}
              statusMarkerFor={(n) => listStatusMarker({ anilistId: n.anilistId, malId: n.malId })}
              anilistIcon={<AniListIcon size={11} />}
            />
          )}
        </Section>
```

Keep the outer gate so the section only renders when there's a franchise. Adjust the surrounding conditional that currently wraps the Related `<Section>` (it gates on `sortedRelations.length`) to instead gate on `(franchiseGraph?.nodes.length ?? 0) > 1`.

- [ ] **Step 4: Leave the Recommendations section unchanged.** Confirm it still renders (it does not depend on the removed code).

- [ ] **Step 5: Remove now-dead code** (do this only after Step 3 compiles):
  - `sortedRelations`, `inLibraryRelations`, `externalRelations`, `inLibraryWithSelf`, `resolveOwnedId`, `CURRENT_RELATION_TYPE` and the synthesized-current logic.
  - Module-level `RELATION_LABEL`, `RELATION_ORDER`, `sortRelations`, `relationFormatLabel` if no longer referenced (grep first).
  - Run `rg -n "sortRelations|inLibraryRelations|externalRelations|RELATION_ORDER|RELATION_LABEL|inLibraryWithSelf|resolveOwnedId" src/renderer/pages/SeriesDetailPage.tsx` to confirm zero remaining references before deleting each.

- [ ] **Step 6: Typecheck**

Run: `bun run typecheck`
Expected: exit 0. Fix any unused-import / unused-var errors surfaced by the removals.

- [ ] **Step 7: Commit**

```bash
git add src/renderer/pages/SeriesDetailPage.tsx
git commit -m "feat(franchise): render FranchiseMap, remove two-bucket Related"
```

---

## Task 10: Manual verification in dev mode

**Files:** none (verification only)

- [ ] **Step 1: Run the unit suites**

Run: `bun run verify:franchise-graph && bun run verify:franchise-lanes`
Expected: `OK: franchise graph closure` and `OK: franchise lanes`.

- [ ] **Step 2: Full lint + typecheck**

Run: `bun run lint`
Expected: clean (eslint + tsc).

- [ ] **Step 3: Launch dev and verify behavior**

Run: `bun run dev`
Verify on a Series Detail page for each shape:
- A **linear multi-season** franchise: spine shows the full prequel→sequel chain, current highlighted + centered, owned tiles open in-app, non-owned open AniList.
- A **manga-rooted** show: the source manga appears in the **top** lane (not the spine), and screen adaptations sit in the spine/bottom.
- A franchise with an **ALTERNATIVE** (reboot): the alternate appears in the spine row's branch group (acceptable v1) — note for a follow-up if a distinct visual branch is wanted.
- Confirm the background fill: nodes that aren't owned still appear (the AniList crawl ran), and a second visit is instant (cache hit).
- Confirm **Recommendations** still renders below as its own section.

- [ ] **Step 4: Note any visual refinements** (column alignment, drawn connectors, distinct side-branch rendering) as follow-up items — they are deliberately out of v1 scope per the spec.

- [ ] **Step 5: Commit any fixes made during verification**, then this feature branch is ready for review/merge.

---

## Self-Review

**Spec coverage:**
- Timeline-first spine + current highlighted → Tasks 7, 9. ✅
- Whole franchise regardless of ownership → Tasks 1, 3, 5 (seed ignores ownership; crawl fills non-owned). ✅
- Hybrid instant + fill + cache → Tasks 3 (cache + crawl), 5 (local seed → IPC swap). ✅
- Lane assignment incl. SOURCE/ADAPTATION direction → Task 2. ✅
- Crawl bounds (exclude CHARACTER/OTHER, node cap) → Task 1 (`closeGraph`, `TRAVERSABLE`). ✅
- Ownership as subtle marker, click routing → Task 6 (pills + handlers), Task 9 (resolve/open). ✅
- CHARACTER dropped from map → Task 1 (skipped in closure) + Task 2 (`excluded` lane). ✅
- Recommendations kept separate → Task 9 Step 4. ✅
- Code extraction out of SeriesDetailPage → Tasks 5–9. ✅
- Structured lanes + horizontal scroll + center → Task 7. ✅
- ALTERNATIVE side-branch → partial in v1 (grouped into spine row); flagged in Task 10 Step 4 as a visual follow-up. Acceptable: data model classifies it correctly (`sidebranch`); only the distinct visual rendering is deferred.

**Placeholder scan:** No "TBD/TODO". The one real-world binding (`loadSavedMetadata`) is explicitly resolved in Task 3 Step 1/Step 3 against the actual code, not left vague.

**Type consistency:** `FranchiseNode`/`FranchiseEdge`/`FranchiseGraph`/`RawRelation` defined once in `src/shared/franchise.ts` and imported everywhere. `FranchiseLane` defined in `laneAssignment.ts`, imported by `FranchiseMap`. `getFranchiseGraph(anilistId)` signature matches across service (Task 3), IPC (Task 4), preload (Task 4), and hook (Task 5). `relationLane(relationType, type, format)` call sites in Task 7 match the Task 2 signature.
