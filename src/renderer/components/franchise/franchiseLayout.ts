import type { FranchiseEdge, FranchiseGraph, FranchiseNode } from '../../../shared/franchise';
import { relationLane, relationLabel } from './laneAssignment';

// ── Visual layout constants ─────────────────────────────────────────────────
const H_GAP      = 240;   // min horizontal slot per leaf
const V_GAP      = 500;   // vertical distance between chain rows
// @ts-ignore -- temporarily unused; will be reintroduced
const SPINE_X_MIN = 280;  // minimum horizontal gap between adjacent chain nodes
const SPINE_X_GAP = 320;  // regular horizontal step between anchor-chain nodes

/** For each reciprocal pair, the "kept" direction is parent→child. The map's
 *  key is the relationType to DROP when the reciprocal of the kept type also
 *  exists in the opposite direction between the same two nodes. */
const RECIPROCAL_DROPS: Map<string, string> = new Map([
  ['SOURCE',    'ADAPTATION'],
  ['PARENT',    'SIDE_STORY'],
  ['PREQUEL',   'SEQUEL'],
]);

/**
 * Normalize and dedupe edges so each connection is represented exactly once
 * in the temporally-correct direction:
 *   - ADAPTATION/SOURCE edges always flow print/source → screen/adaptation,
 *     stored as `ADAPTATION`.
 *   - PARENT/SIDE_STORY: existing reciprocal-drop (keep SIDE_STORY).
 *   - PREQUEL/SEQUEL: existing reciprocal-drop (keep SEQUEL).
 *
 * Note: for a manga adaptation of an anime (anime → manga ADAPTATION), the
 * heuristic will reverse the edge to manga → anime ADAPTATION, treating the
 * manga as the source. This is an acceptable false positive — the overwhelmingly
 * common case (manga is source of anime) is correctly handled.
 */
export function dedupeReciprocalEdges(
  edges: ReadonlyArray<FranchiseEdge>,
  nodeById?: ReadonlyMap<number, FranchiseNode>,
): FranchiseEdge[] {
  // 1. Normalize ADAPTATION/SOURCE edges to canonical "print → screen ADAPTATION".
  const normalized: FranchiseEdge[] = edges.map((e) => {
    if (!nodeById) return e;
    if (e.relationType !== 'ADAPTATION' && e.relationType !== 'SOURCE') return e;
    const from = nodeById.get(e.from);
    const to = nodeById.get(e.to);
    if (!from || !to) return e;
    const fromPrint = isPrintTarget(from);
    const toPrint = isPrintTarget(to);

    if (e.relationType === 'ADAPTATION') {
      // ADAPTATION canonical: from = print, to = screen.
      if (!fromPrint && toPrint) {
        // Anti-canonical (screen → print). Reverse direction; keep type ADAPTATION.
        return { from: e.to, to: e.from, relationType: 'ADAPTATION' };
      }
      // Canonical or both-print/both-screen — keep as-is.
      return e;
    }
    // SOURCE: target is described as "my source". The visual flow we want is
    // source → adaptation, so we always rewrite to ADAPTATION.
    if (fromPrint && !toPrint) {
      // Source claim from print to screen — nonsensical ("anime is novel's source");
      // keep direction, just change type.
      return { from: e.from, to: e.to, relationType: 'ADAPTATION' };
    }
    if (!fromPrint && toPrint) {
      // Canonical SOURCE direction (screen → print). Reverse so arrow flows print → screen.
      return { from: e.to, to: e.from, relationType: 'ADAPTATION' };
    }
    // Both print or both screen — change type to ADAPTATION, keep direction (best we can do).
    return { from: e.from, to: e.to, relationType: 'ADAPTATION' };
  });

  // 2. Drop exact duplicates after normalization.
  const seen = new Set<string>();
  const uniqued: FranchiseEdge[] = [];
  for (const e of normalized) {
    const k = `${e.from}|${e.to}|${e.relationType}`;
    if (seen.has(k)) continue;
    seen.add(k);
    uniqued.push(e);
  }

  // 3. Existing different-type reciprocal drop (PARENT↔SIDE_STORY, PREQUEL↔SEQUEL).
  //    SOURCE↔ADAPTATION is now redundant since SOURCE no longer exists post-normalize.
  const present = new Set<string>(uniqued.map((e) => `${e.from}|${e.to}|${e.relationType}`));
  return uniqued.filter((e) => {
    const keep = RECIPROCAL_DROPS.get(e.relationType);
    if (keep != null && present.has(`${e.to}|${e.from}|${keep}`)) return false;
    return true;
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
  const SPINE_RELS = new Set(['PREQUEL', 'SEQUEL']);
  const seen = new Set<number>([currentId]);
  const q: number[] = [currentId];
  while (q.length > 0) {
    const id = q.shift()!;
    for (const e of adj.get(id) ?? []) {
      if (SPINE_RELS.has(e.relationType) && !seen.has(e.other)) {
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

/**
 * Walk upstream from currentId via ADAPTATION edges to find the franchise's
 * absolute source node, then return its anilistId (or null for empty graphs).
 * Uses the same upstream-walk logic as layoutFranchise so the result is
 * consistent with how the layout picks its spine root.
 */
export function findFranchiseRoot(graph: FranchiseGraph, currentId: number): number | null {
  if (graph.nodes.length === 0) return null;
  const nodeById = new Map(graph.nodes.map((n) => [n.anilistId, n]));
  const edges = dedupeReciprocalEdges(graph.edges, nodeById);
  const dist = new Map<number, number>([[currentId, 0]]);
  const queue: number[] = [currentId];
  let root = currentId;
  let maxDist = 0;
  while (queue.length > 0) {
    const id = queue.shift()!;
    const d = dist.get(id) ?? 0;
    for (const e of edges) {
      if (e.to === id && e.relationType === 'ADAPTATION' && !dist.has(e.from)) {
        dist.set(e.from, d + 1);
        queue.push(e.from);
        if (d + 1 > maxDist) { maxDist = d + 1; root = e.from; }
      }
    }
  }
  return root;
}

/** BFS tree outward from spine: returns parent + children-of maps. */
// @ts-ignore -- temporarily unused; will be reintroduced
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
// @ts-ignore -- temporarily unused; will be reintroduced
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
  for (const e of edges) {
    if (e.from === nodeId && e.to === spineId) {
      const reversed = REVERSE_RELATION[e.relationType] ?? e.relationType;
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
// @ts-ignore -- temporarily unused; will be reintroduced
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

  const nodeById = new Map(graph.nodes.map((n) => [n.anilistId, n]));
  const edges = dedupeReciprocalEdges(graph.edges, nodeById);

  // 1. Walk upstream from currentId via ADAPTATION-incoming edges to find the
  //    franchise's absolute source. After normalization every ADAPTATION edge
  //    flows source→derivative, so walking the reverse direction climbs toward
  //    the original work.
  const dist = new Map<number, number>([[currentId, 0]]);
  const queue: number[] = [currentId];
  let absoluteSource = currentId;
  let maxDist = 0;
  while (queue.length > 0) {
    const id = queue.shift()!;
    const d = dist.get(id) ?? 0;
    for (const e of edges) {
      if (e.to === id && e.relationType === 'ADAPTATION' && !dist.has(e.from)) {
        dist.set(e.from, d + 1);
        queue.push(e.from);
        if (d + 1 > maxDist) {
          maxDist = d + 1;
          absoluteSource = e.from;
        }
      }
    }
  }

  // 2. Pull the absolute source's PREQUEL/SEQUEL connected component.
  const adj = buildAdjacency(edges);
  const chainSet = findSpine(absoluteSource, adj);
  const chainOrdered = topoSortSpine(chainSet, edges, nodeById);

  // 3. Position the chain as a single horizontal line at y=0, evenly spaced.
  chainOrdered.forEach((node, i) => {
    positions.set(node.anilistId, { x: i * SPINE_X_GAP, y: 0 });
  });

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
  const nodeById = new Map(graph.nodes.map((n) => [n.anilistId, n]));
  const edges = dedupeReciprocalEdges(graph.edges, nodeById);
  const adj = buildAdjacency(edges);
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
