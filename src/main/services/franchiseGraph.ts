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
      siteUrl: null,
      titleRomaji: s.titleRomaji ?? null,
      titleEnglish: s.titleEnglish ?? null,
      poster: s.poster ?? null,
    });
    if (Array.isArray(s.relations)) seedRelations.set(s.anilistId, s.relations);
  }
  return { ownedNodes, seedRelations };
}

/**
 * Return the closed, filled franchise graph for the given AniList id.
 * Serves a fresh cached graph when one exists and is complete; otherwise seeds
 * from owned metadata (augmented by prior cached edges for non-deferred nodes),
 * crawls AniList for any new/deferred nodes, caches the result (partial or
 * complete), and returns it.
 */
export async function getFranchiseGraph(anilistId: number): Promise<FranchiseGraph> {
  const cache = await readCache();
  const cachedRoot = cache.index[String(anilistId)];
  const priorEntry = cachedRoot != null ? cache.graphs[String(cachedRoot)] : undefined;
  const prior = priorEntry?.graph;

  // Fast path: fresh AND fully complete — serve from cache unchanged.
  if (prior && prior.complete && priorEntry && Date.now() - priorEntry.fetchedAt < TTL_MS) {
    return prior;
  }

  // Build / extend. Start with owned-metadata seed relations…
  const meta = (await metadataHandler.loadMetadata()) as Record<string, SavedSeries>;
  const { ownedNodes, seedRelations } = buildSeed(meta);

  // …then augment with the prior graph's own edges for every non-deferred node
  // so we don't re-fetch anything we already learned. Deferred nodes are NOT
  // augmented — they get a fresh fetch attempt this pass.
  if (prior) {
    const deferredSet = new Set(prior.deferred);
    const nodeById = new Map(prior.nodes.map((n) => [n.anilistId, n]));
    const edgesByFrom = new Map<number, typeof prior.edges>();
    for (const e of prior.edges) {
      const arr = edgesByFrom.get(e.from);
      if (arr) arr.push(e); else edgesByFrom.set(e.from, [e]);
    }
    for (const node of prior.nodes) {
      if (deferredSet.has(node.anilistId)) continue;
      if (seedRelations.has(node.anilistId)) continue; // owned data wins
      const outgoing = edgesByFrom.get(node.anilistId) ?? [];
      const rels: RawRelation[] = outgoing
        .map((e) => {
          const target = nodeById.get(e.to);
          if (!target) return null;
          return { relationType: e.relationType, ...target } satisfies RawRelation;
        })
        .filter((r): r is RawRelation => r != null);
      seedRelations.set(node.anilistId, rels);
    }
  }

  const currentNode = ownedNodes.get(anilistId) ?? {
    anilistId, malId: null, type: 'ANIME' as const, format: null, status: null,
    seasonYear: null, siteUrl: null, titleRomaji: null, titleEnglish: null, poster: null,
  };

  const graph = await closeGraph({
    seedNodes: [currentNode],
    seedRelations,
    fetch: async (id) => {
      const r = await anilistHandler.fetchRelations(id);
      return { relations: r.relations as RawRelation[], ok: r.ok };
    },
  });

  // Cache the result (even partial); index every member id → rootId.
  cache.graphs[String(graph.rootId)] = { graph, fetchedAt: Date.now() };
  for (const n of graph.nodes) cache.index[String(n.anilistId)] = graph.rootId;
  try {
    await writeCache(cache);
  } catch (e) {
    logger.warn('metadata', `franchise graph cache write failed: ${(e as Error).message}`);
  }
  return graph;
}
