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
  const seenEdges = new Set<string>();
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
  return { rootId, nodes: [...nodes.values()], edges, complete: !hitCap };
}
