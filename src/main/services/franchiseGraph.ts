import { app, BrowserWindow } from 'electron';
import { readFile, writeFile, mkdir, unlink } from 'fs/promises';
import { watch } from 'node:fs';
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

const STORE_FILE = 'franchiseStore.json';
const STORE_TTL_MS = 30 * 24 * 60 * 60 * 1000; // 30 days

// ---------------------------------------------------------------------------
// Store-file watcher — notifies all renderer windows when the file changes.
// ---------------------------------------------------------------------------
let storeWatcher: { close(): void } | null = null;
let storeNotifyTimer: ReturnType<typeof setTimeout> | null = null;
const NOTIFY_DEBOUNCE_MS = 250;

function ensureStoreWatcher(): void {
  if (storeWatcher) return;
  const path = join(app.getPath('userData'), STORE_FILE);
  // Make sure the file exists before fs.watch — fs.watch requires an existing
  // target on Linux.
  void mkdir(app.getPath('userData'), { recursive: true })
    .then(() => writeFile(path, JSON.stringify({ byId: {} }), { flag: 'a' }).catch(() => {}))
    .then(() => {
      try {
        storeWatcher = watch(path, { persistent: false }, () => {
          if (storeNotifyTimer) return; // already pending
          storeNotifyTimer = setTimeout(() => {
            storeNotifyTimer = null;
            for (const win of BrowserWindow.getAllWindows()) {
              if (!win.isDestroyed()) {
                win.webContents.send('franchise:store-updated');
              }
            }
          }, NOTIFY_DEBOUNCE_MS);
        });
      } catch (e) {
        logger.warn('metadata', `franchise store watcher failed to attach: ${(e as Error).message}`);
      }
    });
}

interface ShowEntry {
  /** The show's own data. Null until we've directly fetched this id. */
  node: FranchiseNode | null;
  /** Show's relations to other shows. */
  relations: RawRelation[];
  /** Last successful fetch from AniList. 0 means seeded-from-saved-metadata (stale). */
  fetchedAt: number;
}

interface FranchiseStore {
  byId: Record<string, ShowEntry>;
}

function storePath(): string {
  return join(app.getPath('userData'), STORE_FILE);
}

async function readStore(): Promise<FranchiseStore> {
  try {
    const raw = await readFile(storePath(), 'utf-8');
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === 'object' && parsed.byId && typeof parsed.byId === 'object') {
      // Migration: any entry missing `node` key gets node: null (treated as stale)
      for (const key of Object.keys(parsed.byId)) {
        const entry = parsed.byId[key];
        if (entry && !('node' in entry)) {
          entry.node = null;
          entry.fetchedAt = 0;
        }
      }
      return parsed as FranchiseStore;
    }
  } catch { /* missing/corrupt → empty */ }
  return { byId: {} };
}

async function writeStore(store: FranchiseStore): Promise<void> {
  await mkdir(app.getPath('userData'), { recursive: true });
  await writeFile(storePath(), JSON.stringify(store), 'utf-8');
}

// Guard: clean up legacy caches once per process lifetime.
let legacyCleaned = false;
async function cleanupLegacyCaches(): Promise<void> {
  if (legacyCleaned) return;
  legacyCleaned = true;
  const legacy = ['franchiseGraphCache.json', 'franchiseRelationCache.json'];
  for (const name of legacy) {
    try { await unlink(join(app.getPath('userData'), name)); } catch { /* missing → fine */ }
  }
}

// SeriesMetadata-like shape we read from the saved store.
interface SavedSeries {
  anilistId?: number;
  malId?: number | null;
  format?: string;
  status?: string;
  seasonYear?: number | null;
  titleRomaji?: string;
  titleEnglish?: string | null;
  poster?: string | null;
  relations?: RawRelation[];
}

const yearFromStartDate = (sd: unknown): number | null => {
  if (typeof sd === 'string') {
    const m = sd.match(/^(\d{4})/);
    if (m) return Number(m[1]);
  }
  if (sd && typeof sd === 'object' && 'year' in sd) {
    const y = (sd as { year?: number | null }).year;
    return typeof y === 'number' ? y : null;
  }
  return null;
};

/** Build ownedNodes + seedRelations from every owned series that has an anilistId.
 *  The library only contains anime, so every owned node is type ANIME. */
function buildSeed(meta: Record<string, SavedSeries>): {
  ownedNodes: Map<number, FranchiseNode>;
  seedRelations: Map<number, RawRelation[]>;
} {
  const ownedNodes = new Map<number, FranchiseNode>();
  const seedRelations = new Map<number, RawRelation[]>();
  for (const s of Object.values(meta)) {
    if (typeof s.anilistId !== 'number') continue;
    ownedNodes.set(s.anilistId, {
      anilistId: s.anilistId,
      malId: s.malId ?? null,
      type: 'ANIME',
      format: s.format ?? null,
      status: s.status ?? null,
      seasonYear: s.seasonYear ?? null,
      startYear: s.seasonYear != null ? null : yearFromStartDate((s as unknown as { startDate?: unknown }).startDate) ?? null,
      siteUrl: null,
      titleRomaji: s.titleRomaji ?? null,
      titleEnglish: s.titleEnglish ?? null,
      poster: s.poster ?? null,
    });
    if (Array.isArray(s.relations)) seedRelations.set(s.anilistId, s.relations);
  }
  return { ownedNodes, seedRelations };
}

export async function getFranchiseCrawlProgress(): Promise<{ total: number; crawled: number }> {
  const meta = (await metadataHandler.loadMetadata()) as Record<string, SavedSeries>;
  const ownedIds = new Set<number>();
  for (const s of Object.values(meta)) {
    if (typeof s.anilistId === 'number') ownedIds.add(s.anilistId);
  }
  const store = await readStore();
  let crawled = 0;
  for (const id of ownedIds) {
    const entry = store.byId[String(id)];
    if (entry && entry.node != null && entry.fetchedAt > 0) crawled++;
  }
  return { total: ownedIds.size, crawled };
}

/**
 * Return the closed, filled franchise graph for the given AniList id.
 * Builds the graph from the per-show store on every call (no separate graph
 * cache). Store entries are considered fresh within STORE_TTL_MS; stale or
 * missing entries are fetched from AniList.
 */
export async function getFranchiseGraph(anilistId: number): Promise<FranchiseGraph> {
  ensureStoreWatcher();
  await cleanupLegacyCaches();

  const store = await readStore();
  let storeDirty = false;
  const now = Date.now();

  // Build seed from saved metadata; populate any missing store entries with
  // fetchedAt: 0 (stale but data available as fallback).
  const meta = (await metadataHandler.loadMetadata()) as Record<string, SavedSeries>;
  const { ownedNodes, seedRelations } = buildSeed(meta);

  for (const [id, node] of ownedNodes) {
    if (!store.byId[String(id)]) {
      store.byId[String(id)] = {
        node,
        relations: seedRelations.get(id) ?? [],
        fetchedAt: 0,
      };
      storeDirty = true;
    }
  }

  // Derive current node from store or owned metadata or a bare stub.
  const storeEntry = store.byId[String(anilistId)];
  const currentNode: FranchiseNode = storeEntry?.node ?? ownedNodes.get(anilistId) ?? {
    anilistId, malId: null, type: 'ANIME' as const, format: null, status: null,
    seasonYear: null, startYear: null, siteUrl: null, titleRomaji: null, titleEnglish: null, poster: null,
  };

  // Build seedRelations map from the store for closeGraph.
  const storeSeedRelations = new Map<number, RawRelation[]>();
  for (const [idStr, entry] of Object.entries(store.byId)) {
    const id = Number(idStr);
    if (Number.isFinite(id) && entry.fetchedAt > 0 && now - entry.fetchedAt < STORE_TTL_MS) {
      // Fresh store entry — use its relations as seed so closeGraph doesn't fetch.
      storeSeedRelations.set(id, entry.relations);
    }
    // Stale/seeded entries are NOT added to storeSeedRelations — closeGraph will
    // call the fetch callback, which will update the store.
  }

  const graph = await closeGraph({
    seedNodes: [currentNode],
    seedRelations: storeSeedRelations,
    fetch: async (id) => {
      // Check store again: fresh hit (may have been written this session).
      const cached = store.byId[String(id)];
      if (cached && cached.fetchedAt > 0 && now - cached.fetchedAt < STORE_TTL_MS) {
        return { relations: cached.relations, ok: true };
      }
      // Miss or stale — call AniList.
      const result = await anilistHandler.fetchRelations(id);
      if (result.ok) {
        store.byId[String(id)] = {
          node: result.self as FranchiseNode | null,
          relations: result.relations as RawRelation[],
          fetchedAt: Date.now(),
        };
        storeDirty = true;
      }
      return { relations: result.relations as RawRelation[], ok: result.ok };
    },
  });

  // Persist the store if we added or updated any entries this run.
  if (storeDirty) {
    try {
      await writeStore(store);
    } catch (e) {
      logger.warn('metadata', `franchise store write failed: ${(e as Error).message}`);
    }
  }

  return graph;
}
