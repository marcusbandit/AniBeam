import { app, BrowserWindow } from 'electron';
import { readFile, mkdir, writeFile } from 'fs/promises';
import { watch } from 'node:fs';
import { join } from 'path';
import metadataHandler from '../handlers/metadataHandler';
import { logger } from './logger';
import {
  closeGraph,
  type FranchiseGraph,
  type FranchiseNode,
  type RawRelation,
} from '../../shared/franchise';

const INDEX_FILE = 'franchiseStore.json';
const FRANCHISES_DIR = 'franchises';

// ---------------------------------------------------------------------------
// Disk layout
// ---------------------------------------------------------------------------
// franchiseStore.json — owned-library index
//   { library: { "<anilistId>": { node, relations, fetchedAt, franchise } } }
// franchises/franchise-<rootId>.json — full closure for one connected component
//   { rootId, byId: { "<anilistId>": { node, relations, fetchedAt } } }
//
// The runtime is store-only: getFranchiseGraph reads the index, follows the
// franchise pointer to the per-franchise file, and builds the graph entirely
// from disk. No AniList fetches, no writes.

interface ShowEntry {
  /** The show's own data. Null until we've directly fetched this id. */
  node: FranchiseNode | null;
  /** Show's relations to other shows. */
  relations: RawRelation[];
  /** Last successful fetch from AniList. 0 means seeded-from-saved-metadata (stale). */
  fetchedAt: number;
}

interface LibraryEntry extends ShowEntry {
  /** Per-franchise file key, e.g. "franchise-5081". */
  franchise: string;
}

interface FranchiseStoreIndex {
  library: Record<string, LibraryEntry>;
}

interface FranchiseFile {
  rootId: number;
  byId: Record<string, ShowEntry>;
}

function indexPath(): string {
  return join(app.getPath('userData'), INDEX_FILE);
}

function franchisesDir(): string {
  return join(app.getPath('userData'), FRANCHISES_DIR);
}

function franchiseFilePath(key: string): string {
  return join(franchisesDir(), `${key}.json`);
}

async function readIndex(): Promise<FranchiseStoreIndex> {
  try {
    const raw = await readFile(indexPath(), 'utf-8');
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === 'object' && parsed.library && typeof parsed.library === 'object') {
      return parsed as FranchiseStoreIndex;
    }
  } catch { /* missing/corrupt → empty */ }
  return { library: {} };
}

async function readFranchiseFile(key: string): Promise<FranchiseFile | null> {
  try {
    const raw = await readFile(franchiseFilePath(key), 'utf-8');
    const parsed = JSON.parse(raw);
    if (
      parsed && typeof parsed === 'object'
      && typeof parsed.rootId === 'number'
      && parsed.byId && typeof parsed.byId === 'object'
    ) {
      return parsed as FranchiseFile;
    }
  } catch { /* missing/corrupt → null */ }
  return null;
}

// ---------------------------------------------------------------------------
// Store-file watcher — notifies all renderer windows when the index or any
// per-franchise file changes on disk.
// ---------------------------------------------------------------------------
let indexWatcher: { close(): void } | null = null;
let dirWatcher: { close(): void } | null = null;
let storeNotifyTimer: ReturnType<typeof setTimeout> | null = null;
const NOTIFY_DEBOUNCE_MS = 250;

function scheduleStoreNotify(): void {
  if (storeNotifyTimer) return; // already pending
  storeNotifyTimer = setTimeout(() => {
    storeNotifyTimer = null;
    for (const win of BrowserWindow.getAllWindows()) {
      if (!win.isDestroyed()) {
        win.webContents.send('franchise:store-updated');
      }
    }
  }, NOTIFY_DEBOUNCE_MS);
}

function ensureStoreWatcher(): void {
  if (indexWatcher && dirWatcher) return;
  const userData = app.getPath('userData');
  const idx = indexPath();
  const dir = franchisesDir();

  // fs.watch needs an existing target on Linux — touch the file and mkdir the
  // dir before attaching watchers.
  void mkdir(userData, { recursive: true })
    .then(() => mkdir(dir, { recursive: true }))
    .then(() => writeFile(idx, JSON.stringify({ library: {} }), { flag: 'wx' }).catch(() => {}))
    .then(() => {
      if (!indexWatcher) {
        try {
          indexWatcher = watch(idx, { persistent: false }, () => scheduleStoreNotify());
        } catch (e) {
          logger.warn('metadata', `franchise index watcher failed to attach: ${(e as Error).message}`);
        }
      }
      if (!dirWatcher) {
        try {
          dirWatcher = watch(dir, { persistent: false }, () => scheduleStoreNotify());
        } catch (e) {
          logger.warn('metadata', `franchises dir watcher failed to attach: ${(e as Error).message}`);
        }
      }
    });
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

/** Build a stub FranchiseNode from a SavedSeries entry (anilistId required). */
function nodeFromOwnedSeries(s: SavedSeries): FranchiseNode {
  return {
    anilistId: s.anilistId as number,
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
  };
}

export async function getFranchiseCrawlProgress(): Promise<{ total: number; crawled: number }> {
  const meta = (await metadataHandler.loadMetadata()) as Record<string, SavedSeries>;
  let total = 0;
  const ownedIds: number[] = [];
  for (const s of Object.values(meta)) {
    if (typeof s.anilistId === 'number') {
      total++;
      ownedIds.push(s.anilistId);
    }
  }
  const index = await readIndex();
  let crawled = 0;
  for (const id of ownedIds) {
    const entry = index.library[String(id)];
    if (entry && entry.node != null && entry.fetchedAt > 0) crawled++;
  }
  return { total, crawled };
}

/**
 * Return the closed franchise graph for the given AniList id. Pure disk read:
 *   1. Look up `anilistId` in the index.
 *   2. If present → load its franchise file and close the graph from those entries.
 *   3. If absent → fall back to a single-node graph from owned metadata.
 *
 * No AniList fetches, no writes. The prefill script is responsible for
 * populating the on-disk store.
 */
export async function getFranchiseGraph(anilistId: number): Promise<FranchiseGraph> {
  ensureStoreWatcher();

  const index = await readIndex();
  const lib = index.library[String(anilistId)];

  if (lib != null) {
    const file = await readFranchiseFile(lib.franchise);
    if (file != null) {
      const seedRelations = new Map<number, RawRelation[]>();
      const seedNodes: FranchiseNode[] = [];
      for (const [idStr, entry] of Object.entries(file.byId)) {
        const id = Number(idStr);
        if (!Number.isFinite(id)) continue;
        if (entry.fetchedAt > 0) {
          seedRelations.set(id, Array.isArray(entry.relations) ? entry.relations : []);
        }
        if (entry.node) seedNodes.push(entry.node);
      }

      // Make sure the requested node is in the seed list, even if its node is
      // null on disk — synthesize a bare stub so closeGraph can BFS from it.
      const fileEntry = file.byId[String(anilistId)];
      if (!fileEntry || !fileEntry.node) {
        seedNodes.push({
          anilistId, malId: null, type: 'ANIME' as const, format: null, status: null,
          seasonYear: null, startYear: null, siteUrl: null, titleRomaji: null, titleEnglish: null, poster: null,
        });
      }

      return closeGraph({ seedNodes, seedRelations });
    }
    // Index pointed at a missing franchise file → fall through to metadata fallback.
    logger.warn('metadata', `franchise file missing for ${lib.franchise}; falling back to single-node graph`);
  }

  // Not crawled yet (or franchise file went AWOL) — build a single-node graph
  // from owned metadata so the UI still has something to render.
  const meta = (await metadataHandler.loadMetadata()) as Record<string, SavedSeries>;
  let stub: FranchiseNode | null = null;
  for (const s of Object.values(meta)) {
    if (s && typeof s.anilistId === 'number' && s.anilistId === anilistId) {
      stub = nodeFromOwnedSeries(s);
      break;
    }
  }
  if (!stub) {
    stub = {
      anilistId, malId: null, type: 'ANIME' as const, format: null, status: null,
      seasonYear: null, startYear: null, siteUrl: null, titleRomaji: null, titleEnglish: null, poster: null,
    };
  }
  return closeGraph({ seedNodes: [stub], seedRelations: new Map() });
}
