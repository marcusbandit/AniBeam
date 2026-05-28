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
    if (parsed && typeof parsed === 'object' && parsed.graphs && parsed.index) {
      return parsed as CacheShape;
    }
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
  format?: string;
  status?: string;
  seasonYear?: number | null;
  titleRomaji?: string;
  titleEnglish?: string | null;
  poster?: string | null;
  relations?: RawRelation[];
}

/** Build seedNodes + seedRelations from every owned series that has an anilistId.
 *  The library only contains anime, so every owned node is type ANIME. */
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
      type: 'ANIME',
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
  try {
    await writeCache(cache);
  } catch (e) {
    logger.warn('metadata', `franchise graph cache write failed: ${(e as Error).message}`);
  }
  return graph;
}
