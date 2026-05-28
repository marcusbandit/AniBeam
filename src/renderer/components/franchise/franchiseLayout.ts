import type { FranchiseEdge, FranchiseGraph, FranchiseNode } from '../../../shared/franchise';
import { relationLane, relationLabel } from './laneAssignment';

// ── Visual layout constants ─────────────────────────────────────────────────
const NODE_W = 180;   // tile width
const H_GAP  = 240;   // min horizontal slot per leaf
const V_GAP  = 500;   // vertical distance between tree levels
const SPINE_X_MIN = 280; // minimum horizontal gap between adjacent spine nodes

const SPINE_RELATIONS = new Set(['PREQUEL', 'SEQUEL']);

/** For each reciprocal pair, the "kept" direction is parent→child. The map's
 *  key is the relationType to DROP when the reciprocal of the kept type also
 *  exists in the opposite direction between the same two nodes. */
const RECIPROCAL_DROPS: Map<string, string> = new Map([
  ['SOURCE',    'ADAPTATION'],
  ['PARENT',    'SIDE_STORY'],
  ['PREQUEL',   'SEQUEL'],
]);

/**
 * Drop reciprocal duplicate edges. For each edge whose relationType is in
 * RECIPROCAL_DROPS, if the reciprocal edge exists in the opposite direction
 * with the "kept" relationType, drop this one. Asymmetric edges (only one
 * direction exists in the graph) are kept as-is.
 */
export function dedupeReciprocalEdges(edges: ReadonlyArray<FranchiseEdge>): FranchiseEdge[] {
  const present = new Set<string>(edges.map((e) => `${e.from}|${e.to}|${e.relationType}`));
  return edges.filter((e) => {
    const keep = RECIPROCAL_DROPS.get(e.relationType);
    if (keep == null) return true;
    return !present.has(`${e.to}|${e.from}|${keep}`);
  });
}

interface AdjEdge { other: number; relationType: string; direction: 'out' | 'in'; }

function buildAdjacency(edges: ReadonlyArray<FranchiseEdge>): Map<number, AdjEdge[]> {
  const adj = new Map<number, AdjEdge[]>();
  const push = (k: number, v: AdjEdge) => {
    const a = adj.get(k);
    if (a) a.push(v); else adj.set(k, [v]);
  };
  for (const e of edges) {
    push(e.from, { other: e.to,   relationType: e.relationType, direction: 'out' });
    push(e.to,   { other: e.from, relationType: e.relationType, direction: 'in' });
  }
  return adj;
}

/** spine = BFS from current through PREQUEL/SEQUEL edges (either direction). */
function findSpine(currentId: number, adj: Map<number, AdjEdge[]>): Set<number> {
  const seen = new Set<number>([currentId]);
  const q: number[] = [currentId];
  while (q.length > 0) {
    const id = q.shift()!;
    for (const e of adj.get(id) ?? []) {
      if (SPINE_RELATIONS.has(e.relationType) && !seen.has(e.other)) {
        seen.add(e.other);
        q.push(e.other);
      }
    }
  }
  return seen;
}

function topoSortSpine(
  spineSet: Set<number>,
  edges: ReadonlyArray<FranchiseEdge>,
  nodeById: Map<number, FranchiseNode>,
): FranchiseNode[] {
  // Edges INSIDE the spine, where relationType === SEQUEL → from must precede to.
  const internalSequels = edges.filter(
    (e) => e.relationType === 'SEQUEL' && spineSet.has(e.from) && spineSet.has(e.to),
  );

  // Build in-degree map and adjacency for Kahn's algorithm.
  const inDeg = new Map<number, number>();
  const out = new Map<number, number[]>();
  for (const id of spineSet) inDeg.set(id, 0);
  for (const e of internalSequels) {
    inDeg.set(e.to, (inDeg.get(e.to) ?? 0) + 1);
    const arr = out.get(e.from);
    if (arr) arr.push(e.to); else out.set(e.from, [e.to]);
  }

  // Year-then-id comparator for stable, intuitive ordering within a topo level.
  const cmp = (a: number, b: number): number => {
    const na = nodeById.get(a);
    const nb = nodeById.get(b);
    const ay = na?.seasonYear ?? Number.POSITIVE_INFINITY;
    const by = nb?.seasonYear ?? Number.POSITIVE_INFINITY;
    if (ay !== by) return ay - by;
    return a - b;
  };

  // Kahn's with a min-heap-ish ready set (we just sort the ready array each pop,
  // fine for spine sizes — usually <30 nodes).
  const ready: number[] = [...spineSet].filter((id) => (inDeg.get(id) ?? 0) === 0);
  ready.sort(cmp);
  const ordered: number[] = [];
  while (ready.length > 0) {
    // Pop smallest by year (tiebreak id).
    const next = ready.shift()!;
    ordered.push(next);
    for (const succ of out.get(next) ?? []) {
      const d = (inDeg.get(succ) ?? 0) - 1;
      inDeg.set(succ, d);
      if (d === 0) {
        ready.push(succ);
        ready.sort(cmp);
      }
    }
  }

  // Map back to nodes; fall back to year-sort if topo couldn't order everyone
  // (e.g. a cycle from bad data — defensive).
  if (ordered.length !== spineSet.size) {
    return [...spineSet]
      .map((id) => nodeById.get(id))
      .filter((n): n is FranchiseNode => n != null)
      .sort((a, b) => {
        const ay = a.seasonYear ?? Number.POSITIVE_INFINITY;
        const by = b.seasonYear ?? Number.POSITIVE_INFINITY;
        if (ay !== by) return ay - by;
        return a.anilistId - b.anilistId;
      });
  }
  return ordered.map((id) => nodeById.get(id)!).filter(Boolean);
}

/** BFS tree outward from spine: returns parent + children-of maps. */
function buildBfsTree(
  spineSet: Set<number>,
  adj: Map<number, AdjEdge[]>,
): { parents: Map<number, number>; children: Map<number, number[]> } {
  const parents = new Map<number, number>();
  const children = new Map<number, number[]>();
  const seen = new Set<number>(spineSet);
  const q: number[] = [...spineSet];
  while (q.length > 0) {
    const id = q.shift()!;
    for (const e of adj.get(id) ?? []) {
      if (seen.has(e.other)) continue;
      seen.add(e.other);
      parents.set(e.other, id);
      const kids = children.get(id);
      if (kids) kids.push(e.other); else children.set(id, [e.other]);
      q.push(e.other);
    }
  }
  return { parents, children };
}

/** Determine the lane (+1 below / -1 above) for a node relative to a spine node
 *  by inspecting the edge between them in either direction. */
function laneRelativeToSpine(
  spineId: number,
  nodeId: number,
  node: FranchiseNode,
  edges: ReadonlyArray<FranchiseEdge>,
): -1 | 1 {
  // Forward edge spine → node
  for (const e of edges) {
    if (e.from === spineId && e.to === nodeId) {
      const lane = relationLane(e.relationType, node.type, node.format);
      return lane === 'top' ? -1 : 1;
    }
  }
  // Reverse edge node → spine: reverse the relation to spine's perspective
  const REVERSE: Record<string, string> = {
    SOURCE:     'ADAPTATION',
    ADAPTATION: 'SOURCE',
    PARENT:     'SIDE_STORY',
    SIDE_STORY: 'PARENT',
    PREQUEL:    'SEQUEL',
    SEQUEL:     'PREQUEL',
  };
  for (const e of edges) {
    if (e.from === nodeId && e.to === spineId) {
      const reversed = REVERSE[e.relationType] ?? e.relationType;
      const lane = relationLane(reversed, node.type, node.format);
      return lane === 'top' ? -1 : 1;
    }
  }
  return 1; // default to bottom
}

/** Compute the recursive width of the subtree rooted at `nodeId`. Each leaf is
 *  at least H_GAP wide; internal nodes are at least the sum of their children's
 *  widths (so siblings don't overlap). */
function measureSubtree(nodeId: number, children: Map<number, number[]>): number {
  const kids = children.get(nodeId) ?? [];
  if (kids.length === 0) return H_GAP;
  const childTotal = kids.reduce((sum, k) => sum + measureSubtree(k, children), 0);
  return Math.max(H_GAP, childTotal);
}

/** Recursively place a subtree. Parent is placed at (cx, cy); each child is
 *  placed at (cx_child, cy + dir*V_GAP), where children's x positions are
 *  centered on cx and spaced by their measured subtree widths. */
function placeSubtree(
  nodeId: number,
  cx: number,
  cy: number,
  dir: -1 | 1,
  children: Map<number, number[]>,
  positions: Map<number, { x: number; y: number }>,
): void {
  positions.set(nodeId, { x: cx, y: cy });
  const kids = children.get(nodeId) ?? [];
  if (kids.length === 0) return;
  const widths = kids.map((k) => measureSubtree(k, children));
  const total = widths.reduce((s, w) => s + w, 0);
  let left = cx - total / 2;
  for (let i = 0; i < kids.length; i++) {
    const childCx = left + widths[i] / 2;
    const childCy = cy + dir * V_GAP;
    placeSubtree(kids[i], childCx, childCy, dir, children, positions);
    left += widths[i];
  }
}

export function layoutFranchise(graph: FranchiseGraph, currentId: number): Map<number, { x: number; y: number }> {
  const positions = new Map<number, { x: number; y: number }>();
  if (graph.nodes.length === 0) return positions;

  // 1. Dedupe reciprocal edges
  const edges = dedupeReciprocalEdges(graph.edges);

  // 2. Build adjacency + anime spine set
  const adj = buildAdjacency(edges);
  const nodeById = new Map(graph.nodes.map((n) => [n.anilistId, n]));
  const animeSpineSet = findSpine(currentId, adj);
  const animeSpineList = topoSortSpine(animeSpineSet, edges, nodeById);

  // ── Step A: detect direct sources of anime spine nodes ─────────────────────
  // directSourceOf: animeId → set of sourceNodeIds
  // animeForSource: sourceId → first animeId it's a source of
  const directSourceOf = new Map<number, Set<number>>();
  const animeForSource = new Map<number, number>();

  const recordSource = (animeId: number, sourceId: number) => {
    if (!directSourceOf.has(animeId)) directSourceOf.set(animeId, new Set());
    directSourceOf.get(animeId)!.add(sourceId);
    if (!animeForSource.has(sourceId)) animeForSource.set(sourceId, animeId);
  };

  for (const animeId of animeSpineSet) {
    for (const e of edges) {
      if (e.from === animeId) {
        const target = nodeById.get(e.to);
        if (!target) continue;
        if (canonicalRelation(e.relationType, target) === 'SOURCE') recordSource(animeId, e.to);
      } else if (e.to === animeId) {
        const target = nodeById.get(e.from);
        if (!target) continue;
        const reversed = REVERSE_RELATION[e.relationType] ?? e.relationType;
        if (canonicalRelation(reversed, target) === 'SOURCE') recordSource(animeId, e.from);
      }
    }
  }

  // ── Step B: expand direct sources to a full source spine via PREQUEL/SEQUEL ─
  const sourceSpineSet = new Set<number>();
  const allDirectSources = new Set([...directSourceOf.values()].flatMap((s) => [...s]));
  for (const sourceId of allDirectSources) {
    const chain = findSpine(sourceId, adj);
    for (const id of chain) {
      if (!animeSpineSet.has(id)) sourceSpineSet.add(id);
    }
  }
  const sourceSpineList = topoSortSpine(sourceSpineSet, edges, nodeById);

  // ── Step C: position source spine above anime spine, aligned by adaptation ──
  const SOURCE_Y = -2 * V_GAP;
  const sourcePositioned = new Set<number>();

  // First pass: source nodes with a direct anime counterpart get x = anime.x
  // (anime spine isn't placed yet — we'll pin them after step 6, so we use a
  // two-stage approach: record the animeId and resolve x after the anime spine
  // x-positions are computed in step 6).

  // ── Unified spine set for BFS tree roots ───────────────────────────────────
  const allSpineSet = new Set([...animeSpineSet, ...sourceSpineSet]);

  // 3. Build BFS tree from BOTH spines outward
  const { children: treeChildren } = buildBfsTree(allSpineSet, adj);

  // 4. Split each anime spine node's children into top/bottom subtrees.
  //    Source spine nodes' children are split similarly after they're positioned.
  const topChildrenOf = new Map<number, number[]>();   // spineId → top-lane children
  const bottomChildrenOf = new Map<number, number[]>(); // spineId → bottom-lane children

  // Process both spine lists for child classification.
  const allSpineNodes = [...animeSpineList, ...sourceSpineList];
  for (const spineNode of allSpineNodes) {
    const allKids = treeChildren.get(spineNode.anilistId) ?? [];
    const tops: number[] = [];
    const bots: number[] = [];
    for (const kid of allKids) {
      const kidNode = nodeById.get(kid);
      if (!kidNode) continue;
      const dir = laneRelativeToSpine(spineNode.anilistId, kid, kidNode, edges);
      (dir === -1 ? tops : bots).push(kid);
    }
    if (tops.length > 0) topChildrenOf.set(spineNode.anilistId, tops);
    if (bots.length > 0) bottomChildrenOf.set(spineNode.anilistId, bots);
  }

  // 5. Measure half-widths for the anime spine nodes (source spine x is derived
  //    from anime spine x, so we only need widths for the anime spine here).
  const halfWidthFor = (spineId: number): number => {
    const tops = topChildrenOf.get(spineId) ?? [];
    const bots = bottomChildrenOf.get(spineId) ?? [];
    const topTotal = tops.reduce((s, k) => s + measureSubtree(k, treeChildren), 0);
    const botTotal = bots.reduce((s, k) => s + measureSubtree(k, treeChildren), 0);
    return Math.max(NODE_W + H_GAP, topTotal, botTotal) / 2;
  };

  // 6. Place anime spine nodes left→right with gaps sized by adjacent half-widths.
  let x = 0;
  animeSpineList.forEach((node, i) => {
    if (i > 0) {
      const prev = animeSpineList[i - 1];
      const gap = Math.max(SPINE_X_MIN, halfWidthFor(prev.anilistId) + halfWidthFor(node.anilistId));
      x += gap;
    }
    positions.set(node.anilistId, { x, y: 0 });
  });

  // ── Step C (continued): now that anime spine x-positions are known, place source spine ──

  // First pass: source nodes that have a direct anime counterpart get pinned x = anime.x
  for (const node of sourceSpineList) {
    const animeId = animeForSource.get(node.anilistId);
    if (animeId != null) {
      const animePos = positions.get(animeId);
      if (animePos) {
        positions.set(node.anilistId, { x: animePos.x, y: SOURCE_Y });
        sourcePositioned.add(node.anilistId);
      }
    }
  }

  // Second pass: source nodes WITHOUT direct adaptations get spread evenly
  // between their nearest topo-positioned bookends.
  let i = 0;
  while (i < sourceSpineList.length) {
    const node = sourceSpineList[i];
    if (sourcePositioned.has(node.anilistId)) { i++; continue; }

    // Find the run of consecutive unmatched nodes starting at i.
    let runEnd = i;
    while (runEnd + 1 < sourceSpineList.length && !sourcePositioned.has(sourceSpineList[runEnd + 1].anilistId)) {
      runEnd++;
    }
    const runLen = runEnd - i + 1;

    // Find the nearest anchored bookend on each side of the run.
    let prevX: number | null = null;
    for (let j = i - 1; j >= 0; j--) {
      const id = sourceSpineList[j].anilistId;
      if (sourcePositioned.has(id)) { prevX = positions.get(id)!.x; break; }
    }
    let nextX: number | null = null;
    for (let j = runEnd + 1; j < sourceSpineList.length; j++) {
      const id = sourceSpineList[j].anilistId;
      if (sourcePositioned.has(id)) { nextX = positions.get(id)!.x; break; }
    }

    // Compute the x for each member of the run.
    let startX: number;
    let step: number;
    if (prevX != null && nextX != null) {
      // Spread runLen nodes evenly in the open gap between prevX and nextX.
      step = (nextX - prevX) / (runLen + 1);
      startX = prevX + step;
    } else if (prevX != null) {
      // Run is at the right end: extend rightward with SPINE_X_MIN spacing.
      step = SPINE_X_MIN;
      startX = prevX + step;
    } else if (nextX != null) {
      // Run is at the left end: extend leftward.
      step = SPINE_X_MIN;
      startX = nextX - step * runLen;
    } else {
      // No anchored siblings at all (source spine has no direct adaptations).
      // Fall back to a sequential layout.
      step = SPINE_X_MIN;
      startX = 0;
    }

    for (let k = 0; k < runLen; k++) {
      const id = sourceSpineList[i + k].anilistId;
      positions.set(id, { x: startX + k * step, y: SOURCE_Y });
      sourcePositioned.add(id);
    }

    i = runEnd + 1;
  }

  // 7. Place each spine node's top and bottom subtrees.
  //    Anime spine: placed at y=0; top subtrees go up (dir=-1), bottom go down (dir=+1).
  //    Source spine: placed at y=SOURCE_Y; top subtrees go further up, bottom go
  //    between source spine and anime spine.
  for (const spineNode of allSpineNodes) {
    const spinePos = positions.get(spineNode.anilistId);
    if (!spinePos) continue; // defensive — all spine nodes should be placed
    for (const dir of [-1, 1] as const) {
      const kids = (dir === -1 ? topChildrenOf : bottomChildrenOf).get(spineNode.anilistId) ?? [];
      if (kids.length === 0) continue;
      const widths = kids.map((k) => measureSubtree(k, treeChildren));
      const total = widths.reduce((s, w) => s + w, 0);
      let left = spinePos.x - total / 2;
      for (let i = 0; i < kids.length; i++) {
        const childCx = left + widths[i] / 2;
        const childCy = spinePos.y + dir * V_GAP;
        placeSubtree(kids[i], childCx, childCy, dir, treeChildren, positions);
        left += widths[i];
      }
    }
  }

  return positions;
}

/** Given source and target positions, pick handle ids that route the edge
 *  along the dominant axis. */
export function pickHandles(
  src: { x: number; y: number },
  tgt: { x: number; y: number },
): { sourceHandle: string; targetHandle: string } {
  const dx = tgt.x - src.x;
  const dy = tgt.y - src.y;
  if (Math.abs(dy) >= Math.abs(dx)) {
    return dy < 0
      ? { sourceHandle: 'top-s',    targetHandle: 'bottom-t' }
      : { sourceHandle: 'bottom-s', targetHandle: 'top-t'    };
  }
  return dx >= 0
    ? { sourceHandle: 'right-s', targetHandle: 'left-t'  }
    : { sourceHandle: 'left-s',  targetHandle: 'right-t' };
}

/** anilistId → 0-based index along the topo-sorted spine. */
export function spineOrderMap(
  graph: FranchiseGraph,
  currentId: number,
): { spineSet: Set<number>; order: Map<number, number> } {
  const edges = dedupeReciprocalEdges(graph.edges);
  const adj = buildAdjacency(edges);
  const nodeById = new Map(graph.nodes.map((n) => [n.anilistId, n]));
  const spineSet = findSpine(currentId, adj);
  const spineList = topoSortSpine(spineSet, edges, nodeById);
  const order = new Map<number, number>();
  spineList.forEach((n, i) => order.set(n.anilistId, i));
  return { spineSet, order };
}

const REVERSE_RELATION: Record<string, string> = {
  SOURCE:     'ADAPTATION',
  ADAPTATION: 'SOURCE',
  PARENT:     'SIDE_STORY',
  SIDE_STORY: 'PARENT',
  PREQUEL:    'SEQUEL',
  SEQUEL:     'PREQUEL',
};

const PRINT_FORMATS_FOR_LABEL = new Set(['MANGA', 'NOVEL', 'LIGHT_NOVEL', 'ONE_SHOT', 'VISUAL_NOVEL']);

function isPrintTarget(target: FranchiseNode | undefined): boolean {
  if (!target) return false;
  if (target.format && PRINT_FORMATS_FOR_LABEL.has(target.format)) return true;
  return target.type === 'MANGA';
}

/** Normalize ADAPTATION/SOURCE direction based on the target's media format.
 *  AniList sometimes returns the wrong direction (e.g. anime→novel tagged as
 *  ADAPTATION when the novel is actually the source). */
function canonicalRelation(relationType: string, target: FranchiseNode | undefined): string {
  if (relationType === 'ADAPTATION' && isPrintTarget(target)) return 'SOURCE';
  if (relationType === 'SOURCE'     && !isPrintTarget(target)) return 'ADAPTATION';
  return relationType;
}

/**
 * Compute the relation label for `nodeId` relative to `refId`.
 * Returns null when no direct edge or spine ordering applies — caller should
 * fall back to its own tree-derived label. The caller is also responsible for
 * the "Currently viewing" line on the reference node itself; we return null
 * for `nodeId === refId`.
 */
export function relationLabelRelativeTo(
  refId: number,
  nodeId: number,
  spineSet: Set<number>,
  spineOrder: Map<number, number>,
  edges: ReadonlyArray<FranchiseEdge>,
  nodeById: ReadonlyMap<number, FranchiseNode>,
): string | null {
  if (nodeId === refId) return null;

  // Spine vs spine: position-based prequel/sequel
  if (spineSet.has(refId) && spineSet.has(nodeId)) {
    const ro = spineOrder.get(refId);
    const no = spineOrder.get(nodeId);
    if (ro != null && no != null) {
      if (no < ro) return 'Prequel';
      if (no > ro) return 'Sequel';
    }
  }

  const target = nodeById.get(nodeId);

  // Direct edge ref → node — canonicalize against target before labeling.
  for (const e of edges) {
    if (e.from === refId && e.to === nodeId) {
      return relationLabel(canonicalRelation(e.relationType, target));
    }
  }
  // Reverse edge node → ref — first reverse to ref's perspective, then canonicalize.
  for (const e of edges) {
    if (e.from === nodeId && e.to === refId) {
      const rev = REVERSE_RELATION[e.relationType] ?? e.relationType;
      return relationLabel(canonicalRelation(rev, target));
    }
  }
  return null;
}
