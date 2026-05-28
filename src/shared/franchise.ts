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
  /** startDate.year from AniList — populated for non-anime (manga, novels, etc.)
   *  where seasonYear is null. seasonYear takes precedence when both are present. */
  startYear: number | null;
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
  /** True when BFS drained AND no nodes were deferred by the fetcher. */
  complete: boolean;
  /** AniList ids whose relations we couldn't fetch this pass (rate-limited).
   *  A future closeGraph call can retry these. Empty when complete. */
  deferred: number[];
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
  startYear: number | null;
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
    startYear: r.startYear,
    siteUrl: r.siteUrl,
    titleRomaji: r.titleRomaji,
    titleEnglish: r.titleEnglish,
    poster: r.poster,
  };
}

/** Fetch result: `ok: false` means the fetch was rate-limited and the caller
 *  should defer this node for a future retry; the node's relations remain
 *  unknown. `ok: true` with an empty array means we know there are none. */
export type RelationsFetcher = (anilistId: number) => Promise<{ relations: RawRelation[]; ok: boolean }>;

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
  const seenEdges = new Set<string>();
  const expanded = new Set<number>();
  const queue: number[] = [];

  for (const n of opts.seedNodes) {
    if (!nodes.has(n.anilistId)) nodes.set(n.anilistId, n);
    queue.push(n.anilistId);
  }

  let hitCap = false;
  const deferred = new Set<number>();
  while (queue.length > 0) {
    const id = queue.shift()!;
    if (expanded.has(id)) continue;
    expanded.add(id);
    deferred.delete(id); // clear any prior deferral now that we're successfully expanding

    let relations: RawRelation[] | null = opts.seedRelations.get(id) ?? null;
    if (relations == null && opts.fetch) {
      const result = await opts.fetch(id);
      if (!result.ok) {
        deferred.add(id);
        // Roll back the "expanded" mark so a future call can retry this node.
        expanded.delete(id);
        continue;
      }
      relations = result.relations;
    }
    if (relations == null) continue;

    for (const r of relations) {
      if (r.relationType === 'CHARACTER') continue; // dropped from the map
      if (!nodes.has(r.anilistId)) {
        if (nodes.size >= nodeCap) { hitCap = true; continue; }
        nodes.set(r.anilistId, nodeFromRelation(r));
      }
      const edgeKey = `${id}->${r.anilistId}:${r.relationType}`;
      if (!seenEdges.has(edgeKey)) {
        seenEdges.add(edgeKey);
        edges.push({ from: id, to: r.anilistId, relationType: r.relationType });
      }
      if (isTraversable(r.relationType) && !expanded.has(r.anilistId)) {
        queue.push(r.anilistId);
      }
    }
  }

  const ids = [...nodes.keys()];
  const rootId = ids.length ? Math.min(...ids) : 0;
  return { rootId, nodes: [...nodes.values()], edges, complete: !hitCap && deferred.size === 0, deferred: [...deferred] };
}
