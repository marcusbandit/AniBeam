import { readFile, writeFile, mkdir, rename } from 'fs/promises';
import { join } from 'path';
import { app } from 'electron';
import { logger } from '../services/logger';

function getMetadataPath(): string {
  const userDataPath = app.getPath('userData');
  return join(userDataPath, 'metadata.json');
}

async function ensureDataDirectory(): Promise<void> {
  const userDataPath = app.getPath('userData');
  try {
    await mkdir(userDataPath, { recursive: true });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'EEXIST') {
      throw error;
    }
  }
}

// Serialize all read-modify-write transactions on metadata.json. Multiple
// writers (manual scan, watcher ingest, watcher unlink, probe completion)
// would otherwise race: each loads, mutates in-memory, and writes back —
// the last writer wins and any concurrent edits are silently lost.
//
// Use `metadataHandler.transaction(fn)` for any read-then-write operation.
// One-shot reads (`loadMetadata`) and one-shot writes that don't depend on
// the prior state can stay outside the lock.
let writeChain: Promise<unknown> = Promise.resolve();
function runLocked<T>(fn: () => Promise<T>): Promise<T> {
  const next = writeChain.then(() => fn(), () => fn());
  writeChain = next.catch(() => {});
  return next;
}

async function atomicWriteJson(filePath: string, data: unknown): Promise<void> {
  const tmp = `${filePath}.tmp`;
  await writeFile(tmp, JSON.stringify(data, null, 2), 'utf-8');
  await rename(tmp, filePath);
}

function cleanForSave(metadata: Record<string, unknown>): Record<string, unknown> {
  const cleaned: Record<string, unknown> = {};
  for (const [seriesId, seriesData] of Object.entries(metadata)) {
    const data = seriesData as { fileEpisodes?: unknown[] };
    const fileEpisodes = data.fileEpisodes || [];
    if (fileEpisodes.length > 0) {
      cleaned[seriesId] = seriesData;
    }
  }
  return cleaned;
}

async function loadMetadataRaw(): Promise<Record<string, unknown>> {
  await ensureDataDirectory();
  const metadataPath = getMetadataPath();
  let data: string;
  try {
    data = await readFile(metadataPath, 'utf-8');
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return {};
    throw error;
  }
  // Parse failure means corrupt or torn file — DO NOT default to {} here.
  // Returning empty would let the next writer truncate the library to nothing.
  // Throw and let the caller refuse to write on top of a broken state.
  const parsed = JSON.parse(data) as Record<string, unknown>;
  // Migration: ensure every file episode has a status.
  for (const seriesValue of Object.values(parsed)) {
    const series = seriesValue as { fileEpisodes?: unknown[] };
    if (!Array.isArray(series.fileEpisodes)) continue;
    for (const file of series.fileEpisodes) {
      const f = file as { status?: string };
      if (!f.status) f.status = 'ready';
    }
  }
  return parsed;
}

const metadataHandler = {
  async loadMetadata(): Promise<Record<string, unknown>> {
    try {
      return await loadMetadataRaw();
    } catch (error) {
      // ENOENT is handled inside loadMetadataRaw — anything reaching here
      // is a real failure (parse error, EACCES, etc.). Surface it.
      logger.error('metadata', `Error loading metadata: ${(error as Error).message}`);
      throw error;
    }
  },

  async saveMetadata(metadata: Record<string, unknown>): Promise<boolean> {
    try {
      await ensureDataDirectory();
      await atomicWriteJson(getMetadataPath(), cleanForSave(metadata));
      return true;
    } catch (error) {
      logger.error('metadata', `Error saving metadata: ${(error as Error).message}`);
      throw error;
    }
  },

  /**
   * Run a read-modify-write transaction with exclusive access to metadata.json.
   * The callback receives the freshly-loaded metadata and returns the mutated
   * object to persist (or `null` to skip the write). Used by every code path
   * that reads, edits in-memory, and saves — prevents lost updates between
   * watcher events, probe completions, and manual scans.
   */
  transaction<T>(fn: (metadata: Record<string, unknown>) => Promise<{ result?: T; updated?: Record<string, unknown> | null }>): Promise<T | undefined> {
    return runLocked(async () => {
      const meta = await loadMetadataRaw();
      const { result, updated } = await fn(meta);
      if (updated !== null && updated !== undefined) {
        await ensureDataDirectory();
        await atomicWriteJson(getMetadataPath(), cleanForSave(updated));
      }
      return result;
    });
  },

  async updateSeriesMetadata(seriesId: string, seriesData: Record<string, unknown>): Promise<boolean> {
    try {
      await this.transaction(async (metadata) => {
        const existingSeries = metadata[seriesId] as Record<string, unknown> | undefined;
        metadata[seriesId] = { ...existingSeries, ...seriesData };
        return { updated: metadata };
      });
      return true;
    } catch (error) {
      logger.error('metadata', `Error updating series metadata: ${(error as Error).message}`);
      throw error;
    }
  },

  async getSeriesMetadata(seriesId: string): Promise<Record<string, unknown> | null> {
    try {
      const metadata = await this.loadMetadata();
      return (metadata[seriesId] as Record<string, unknown>) || null;
    } catch (error) {
      logger.error('metadata', `Error getting series metadata: ${(error as Error).message}`);
      return null;
    }
  },

  async deleteSeriesMetadata(seriesId: string): Promise<boolean> {
    try {
      await this.transaction(async (metadata) => {
        delete metadata[seriesId];
        return { updated: metadata };
      });
      return true;
    } catch (error) {
      logger.error('metadata', `Error deleting series metadata: ${(error as Error).message}`);
      throw error;
    }
  },
};

export default metadataHandler;
