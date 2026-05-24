import { readFile, writeFile, mkdir } from 'fs/promises';
import { join } from 'path';
import { app } from 'electron';

// Per-series record of the most recent playback session. We only keep the
// "latest" — there's no episode-by-episode history because the only consumer
// (Library "Last viewed" sort) needs a single sortable timestamp per series.
// If something else later needs per-episode timestamps, add a parallel map
// rather than expanding this one.
export interface ViewHistoryEntry {
  /** ms-since-epoch when the user crossed the watched-threshold for this
   *  session. */
  lastViewedAt: number;
  /** Episode number of that session, surfaced for any future UI ("you were
   *  last watching ep 7"). */
  lastEpisode: number;
}

export type ViewHistoryMap = Record<string, ViewHistoryEntry>;

interface StoreShape {
  version: 1;
  history: ViewHistoryMap;
}

const DEFAULT_STORE: StoreShape = { version: 1, history: {} };

function storePath(): string {
  return join(app.getPath('userData'), 'view-history.json');
}

async function ensureDir(): Promise<void> {
  try {
    await mkdir(app.getPath('userData'), { recursive: true });
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code !== 'EEXIST') throw err;
  }
}

let cache: StoreShape | null = null;

async function load(): Promise<StoreShape> {
  if (cache) return cache;
  try {
    await ensureDir();
    const raw = await readFile(storePath(), 'utf-8');
    const parsed = JSON.parse(raw) as Partial<StoreShape>;
    // Defensive: shape-check every entry. A corrupt row would otherwise
    // poison the sort comparator with NaN. Drop anything malformed.
    const cleaned: ViewHistoryMap = {};
    if (parsed.history && typeof parsed.history === 'object') {
      for (const [id, v] of Object.entries(parsed.history)) {
        if (!v || typeof v !== 'object') continue;
        const entry = v as Partial<ViewHistoryEntry>;
        if (typeof entry.lastViewedAt !== 'number' || !Number.isFinite(entry.lastViewedAt)) continue;
        if (typeof entry.lastEpisode !== 'number' || !Number.isFinite(entry.lastEpisode)) continue;
        cleaned[id] = { lastViewedAt: entry.lastViewedAt, lastEpisode: entry.lastEpisode };
      }
    }
    cache = { version: 1, history: cleaned };
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === 'ENOENT') {
      cache = { ...DEFAULT_STORE, history: {} };
    } else {
      // Activity log is signal-only — diagnostic failures stay on stderr.
      console.error('[view-history] failed to load:', (err as Error).message);
      cache = { ...DEFAULT_STORE, history: {} };
    }
  }
  return cache;
}

async function save(store: StoreShape): Promise<void> {
  await ensureDir();
  await writeFile(storePath(), JSON.stringify(store, null, 2), 'utf-8');
  cache = store;
}

export async function getViewHistory(): Promise<ViewHistoryMap> {
  const store = await load();
  return { ...store.history };
}

/**
 * Record a viewing event. We only overwrite when the incoming `ts` is newer
 * than what we have so a stale renderer (e.g. a window that was open across
 * a clock change) can't clobber a fresher entry. Returns true when the
 * store actually changed so the caller can decide whether to broadcast.
 */
export async function markViewed(
  seriesId: string,
  episodeNumber: number,
  ts: number,
): Promise<boolean> {
  if (!seriesId || typeof episodeNumber !== 'number' || typeof ts !== 'number') {
    return false;
  }
  const store = await load();
  const prev = store.history[seriesId];
  if (prev && prev.lastViewedAt >= ts) return false;
  store.history[seriesId] = { lastViewedAt: ts, lastEpisode: episodeNumber };
  await save(store);
  return true;
}
