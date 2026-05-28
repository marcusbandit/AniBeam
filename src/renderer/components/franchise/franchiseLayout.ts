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

  // 2. Build adjacency + spine set
  const adj = buildAdjacency(edges);
  const nodeById = new Map(graph.nodes.map((n) => [n.anilistId, n]));
  const spineSet = findSpine(currentId, adj);
  const spineList = topoSortSpine(spineSet, edges, nodeById);

  // 3. Build BFS tree from spine outward
  const { children: treeChildren } = buildBfsTree(spineSet, adj);

  // 4. Split each spine node's children into top/bottom subtrees
  const topChildrenOf = new Map<number, number[]>();   // spineId → top-lane children
  const bottomChildrenOf = new Map<number, number[]>(); // spineId → bottom-lane children

  for (const spineNode of spineList) {
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

  // 5. Measure half-widths (left side + right side of each spine node's subtree)
  //    For each spine node, compute the horizontal half-width occupied by its
  //    top + bottom subtrees combined, so we can space adjacent spine nodes
  //    without overlap.
  const halfWidthFor = (spineId: number): number => {
    const tops = topChildrenOf.get(spineId) ?? [];
    const bots = bottomChildrenOf.get(spineId) ?? [];
    const topTotal = tops.reduce((s, k) => s + measureSubtree(k, treeChildren), 0);
    const botTotal = bots.reduce((s, k) => s + measureSubtree(k, treeChildren), 0);
    return Math.max(NODE_W + H_GAP, topTotal, botTotal) / 2;
  };

  // 6. Place spine nodes left→right with gaps sized by adjacent half-widths.
  let x = 0;
  spineList.forEach((node, i) => {
    if (i > 0) {
      const prev = spineList[i - 1];
      const gap = Math.max(SPINE_X_MIN, halfWidthFor(prev.anilistId) + halfWidthFor(node.anilistId));
      x += gap;
    }
    positions.set(node.anilistId, { x, y: 0 });
  });

  // 7. Place each spine node's top and bottom subtree.
  for (const spineNode of spineList) {
    const spinePos = positions.get(spineNode.anilistId)!;
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

  // Direct edge ref → node
  for (const e of edges) {
    if (e.from === refId && e.to === nodeId) return relationLabel(e.relationType);
  }
  // Reverse edge node → ref: relabel from ref's perspective
  for (const e of edges) {
    if (e.from === nodeId && e.to === refId) {
      const rev = REVERSE_RELATION[e.relationType] ?? e.relationType;
      return relationLabel(rev);
    }
  }
  return null;
}
