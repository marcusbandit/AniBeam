// Franchise crawler — the writer half of the on-disk franchise store.
//
// franchiseGraph.ts reads `franchiseStore.json` + `franchises/franchise-<rootId>.json`
// and closes a graph entirely from disk. This module *populates* that store:
// it fetches relations via anilistHandler.fetchRelations, drives BFS through
// closeGraph's `fetch` hook, and persists each node as it lands so the existing
// file-watch push (`franchise:store-updated`) makes the relations tree form
// live, node-by-node, with no renderer changes.
//
// Two entry points:
//   crawlFranchiseLive(seedId)  — crawl the connected component of one series.
//   crawlLibraryGaps()          — crawl every owned series not yet in the store.
//
// Reuses the disk layout, path helpers, readers and writers from
// franchiseGraph.ts; it does NOT redefine the on-disk shape.

import anilistHandler from '../handlers/anilistHandler';
import metadataHandler from '../handlers/metadataHandler';
import { logger } from './logger';
import {
  readIndex,
  writeIndex,
  readFranchiseFile,
  writeFranchiseFile,
  deleteFranchiseFile,
  nodeFromOwnedSeries,
  type FranchiseFile,
  type FranchiseStoreIndex,
  type SavedSeries,
} from './franchiseGraph';
import {
  closeGraph,
  type FranchiseNode,
  type RawRelation,
} from '../../shared/franchise';

type Fetcher = (anilistId: number) => Promise<{
  self: FranchiseNode | null;
  relations: RawRelation[];
  ok: boolean;
}>;

export interface CrawlOpts {
  /** ids to re-fetch even if already fetchedAt>0 (on-click refresh passes [seedId]). */
  forceRefetch?: number[];
  /** test seam — defaults to anilistHandler.fetchRelations. */
  fetch?: Fetcher;
  /** test seam — defaults to Date.now. Never called at module top level. */
  now?: () => number;
}

const PERSIST_DEBOUNCE_MS = 200;
// Skip an on-click refresh if this component crawled within the last minute —
// the shared RateLimiter paces everything, but this stops repeated opens from
// re-queuing the same component over and over.
const REFRESH_THROTTLE_MS = 60_000;
const NODE_CAP = 150;

// One in-flight crawl per component file key. A crawl keyed on the seed's
// current key is also reachable by the seed id, so we lock on both to keep two
// concurrent crawls from writing the same component (write races + duplicate
// provisional files).
const inFlight = new Map<string, Promise<void>>();
// Last completed-crawl wall time per file key, for the on-click throttle.
const lastCrawlAt = new Map<string, number>();

// Serialize every index read-modify-write across all crawls. Concurrent crawls
// of *different* components still both touch the single shared index file; this
// chain makes those edits atomic relative to each other.
let indexChain: Promise<unknown> = Promise.resolve();
function runIndexLocked<T>(fn: () => Promise<T>): Promise<T> {
  const next = indexChain.then(() => fn(), () => fn());
  indexChain = next.catch(() => { /* one failed txn must not poison the chain */ });
  return next;
}

/** Owned series ids from saved metadata, with their SavedSeries record. */
async function loadOwned(): Promise<Map<number, SavedSeries>> {
  const meta = (await metadataHandler.loadMetadata()) as Record<string, SavedSeries>;
  const owned = new Map<number, SavedSeries>();
  for (const s of Object.values(meta)) {
    if (s && typeof s.anilistId === 'number') owned.set(s.anilistId, s);
  }
  return owned;
}

/**
 * Crawl the connected component containing `seedId`, persisting incrementally.
 * Locks per component key so the same component never crawls concurrently.
 */
export async function crawlFranchiseLive(seedId: number, opts: CrawlOpts = {}): Promise<void> {
  if (!Number.isFinite(seedId)) return;

  // Resolve the current component key up front so we can lock + throttle on it.
  const index0 = await readIndex();
  const key = index0.library[String(seedId)]?.franchise ?? `franchise-${seedId}`;

  // If a crawl for this key (or this seed) is already running, ride it.
  const running = inFlight.get(key) ?? inFlight.get(`franchise-${seedId}`);
  if (running) {
    await running;
    return;
  }

  // On-click throttle: skip if this component crawled very recently. The
  // gap-fill path passes no forceRefetch, so it isn't subject to this — but a
  // gap-fill of an already-fetched component is skipped earlier anyway.
  const force = opts.forceRefetch ?? [];
  if (force.length > 0) {
    const last = lastCrawlAt.get(key);
    const nowFn = opts.now ?? Date.now;
    if (last != null && nowFn() - last < REFRESH_THROTTLE_MS) return;
  }

  const task = runCrawl(seedId, key, opts).finally(() => {
    inFlight.delete(key);
    inFlight.delete(`franchise-${seedId}`);
  });
  inFlight.set(key, task);
  inFlight.set(`franchise-${seedId}`, task);
  await task;
}

async function runCrawl(seedId: number, key: string, opts: CrawlOpts): Promise<void> {
  const fetcher = opts.fetch ?? anilistHandler.fetchRelations;
  const now = opts.now ?? Date.now;
  const force = new Set(opts.forceRefetch ?? []);

  // Locate the component on disk (or start a fresh provisional one).
  const file: FranchiseFile =
    (await readFranchiseFile(key)) ?? { rootId: seedId, byId: {} };

  // --- Seed closeGraph -----------------------------------------------------
  // seedRelations: every directly-fetched node NOT being force-refetched. These
  // are NOT re-fetched by closeGraph; everything missing/forced gets fetched.
  const seedRelations = new Map<number, RawRelation[]>();
  const seedNodes: FranchiseNode[] = [];
  for (const [idStr, entry] of Object.entries(file.byId)) {
    const id = Number(idStr);
    if (!Number.isFinite(id)) continue;
    if (entry.fetchedAt > 0 && !force.has(id)) {
      seedRelations.set(id, Array.isArray(entry.relations) ? entry.relations : []);
    }
    if (entry.node) seedNodes.push(entry.node);
  }

  // Make sure BFS can start from the seed even on a brand-new series with no
  // node on disk yet — synthesize a stub from owned metadata if we have it.
  if (!seedNodes.some((n) => n.anilistId === seedId)) {
    const owned = await loadOwned();
    const ownedSeed = owned.get(seedId);
    seedNodes.push(
      ownedSeed
        ? nodeFromOwnedSeries(ownedSeed)
        : {
            anilistId: seedId, malId: null, type: 'ANIME' as const, format: null, status: null,
            seasonYear: null, startYear: null, siteUrl: null, titleRomaji: null, titleEnglish: null, poster: null,
          },
    );
  }

  // --- Debounced incremental persistence -----------------------------------
  // Each flush updates the watched file → 250ms-debounced franchise:store-updated
  // → renderer re-reads → tree grows live.
  let persistTimer: ReturnType<typeof setTimeout> | null = null;
  let persistInFlight: Promise<void> = Promise.resolve();
  const flush = (): void => {
    persistInFlight = persistInFlight
      .then(() => writeFranchiseFile(key, file))
      .catch((e) => logger.warn('metadata', `franchise incremental flush failed: ${(e as Error).message}`));
  };
  const schedulePersist = (): void => {
    if (persistTimer) return;
    persistTimer = setTimeout(() => {
      persistTimer = null;
      flush();
    }, PERSIST_DEBOUNCE_MS);
  };
  const cancelPersist = (): void => {
    if (persistTimer) {
      clearTimeout(persistTimer);
      persistTimer = null;
    }
  };

  // --- Persisting fetch wrapper passed to closeGraph -----------------------
  const fetchHook = async (id: number): Promise<{ relations: RawRelation[]; ok: boolean }> => {
    const r = await fetcher(id);
    if (!r.ok) return { relations: [], ok: false }; // rate-limited → closeGraph defers
    file.byId[String(id)] = {
      node: r.self ?? file.byId[String(id)]?.node ?? null,
      relations: r.relations,
      fetchedAt: now(),
    };
    schedulePersist();
    return { relations: r.relations, ok: true };
  };

  const g = await closeGraph({ seedNodes, seedRelations, fetch: fetchHook, nodeCap: NODE_CAP });

  // --- Backfill discovered-but-unfetched nodes -----------------------------
  // Nodes known only from a relation edge: keep for display, mark stale.
  for (const n of g.nodes) {
    if (!file.byId[String(n.anilistId)]) {
      file.byId[String(n.anilistId)] = { node: n, relations: [], fetchedAt: 0 };
    }
  }

  // --- Finalize root + key -------------------------------------------------
  const rootId = g.rootId;
  const finalKey = `franchise-${rootId}`;
  file.rootId = rootId;

  // Cancel the debounce so a late flush can't resurrect the old provisional key.
  cancelPersist();
  await persistInFlight;

  if (finalKey !== key) {
    // The component's canonical key changed (e.g. crawling 104462 discovered the
    // smaller id 6213). Merge into any existing finalKey file — directly-fetched
    // entries (fetchedAt>0) win over stale ones — then drop the old provisional.
    const existing = await readFranchiseFile(finalKey);
    const merged: FranchiseFile = existing ?? { rootId, byId: {} };
    merged.rootId = rootId;
    for (const [idStr, entry] of Object.entries(file.byId)) {
      const prior = merged.byId[idStr];
      if (!prior) {
        merged.byId[idStr] = entry;
      } else if (entry.fetchedAt > 0 && prior.fetchedAt <= 0) {
        merged.byId[idStr] = entry; // fresh fetch beats a stale stub
      } else if (entry.fetchedAt > 0 && prior.fetchedAt > 0 && entry.fetchedAt >= prior.fetchedAt) {
        merged.byId[idStr] = entry; // newer fetch wins
      }
      // else keep prior (it's fetched and entry is stale, or prior is newer)
    }
    await writeFranchiseFile(finalKey, merged);
    if (key !== finalKey) await deleteFranchiseFile(key);
    // The merged file is the authority now; index off of it.
    await updateIndexForComponent(finalKey, merged);
  } else {
    await writeFranchiseFile(key, file); // final flush
    await updateIndexForComponent(finalKey, file);
  }

  lastCrawlAt.set(finalKey, now());
}

/**
 * Point every owned series present in `file.byId` at `finalKey` in the index.
 * Owned = present in saved metadata with that anilistId. Runs through the
 * serialized index queue so concurrent crawls don't clobber each other.
 */
async function updateIndexForComponent(finalKey: string, file: FranchiseFile): Promise<void> {
  const owned = await loadOwned();
  await runIndexLocked(async () => {
    const index: FranchiseStoreIndex = await readIndex();
    let changed = false;
    for (const [idStr, entry] of Object.entries(file.byId)) {
      const id = Number(idStr);
      if (!Number.isFinite(id) || !owned.has(id)) continue;
      index.library[idStr] = {
        node: entry.node,
        relations: entry.relations,
        fetchedAt: entry.fetchedAt,
        franchise: finalKey,
      };
      changed = true;
    }
    if (changed) await writeIndex(index);
  });
}

/**
 * Crawl every owned series not yet covered by the store (gap-fill). Sequential —
 * the global RateLimiter in anilistHandler already paces AniList, so we never
 * fan out. Crawling one component indexes its owned siblings too, so we re-read
 * the index each iteration to skip ones a prior component already covered.
 */
export async function crawlLibraryGaps(opts: { fetch?: Fetcher; now?: () => number } = {}): Promise<void> {
  const owned = await loadOwned();
  const ownedIds = [...owned.keys()];

  // Count the gap up front for the single start-line.
  let index = await readIndex();
  const isCovered = (idx: FranchiseStoreIndex, id: number): boolean => {
    const entry = idx.library[String(id)];
    return !!(entry && entry.node != null && entry.fetchedAt > 0);
  };
  const todo = ownedIds.filter((id) => !isCovered(index, id));
  if (todo.length === 0) return;

  logger.info('metadata', `Franchise gap-crawl: ${todo.length} to fill`);

  let filled = 0;
  for (const id of todo) {
    // Re-read the index each iteration: a prior component crawl may have already
    // covered this owned member as a sibling.
    index = await readIndex();
    if (isCovered(index, id)) continue;
    try {
      await crawlFranchiseLive(id, { fetch: opts.fetch, now: opts.now });
      filled++;
    } catch (e) {
      logger.warn('metadata', `Franchise gap-crawl failed for ${id}: ${(e as Error).message}`);
    }
  }

  logger.info('metadata', `Franchise gap-crawl: done (${filled} crawled)`);
}
