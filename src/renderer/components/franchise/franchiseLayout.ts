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
  // 1. Normalize edges to canonical direction.
  //    PARENT (raw): the `to` node is the parent of `from` — flip to
  //      canonical parent→child SIDE_STORY so downstream code sees one shape.
  //    ALTERNATIVE: symmetric relation — both raw directions can exist in
  //      AniList data. Force smaller-id → larger-id so stage 2 dedupes
  //      the redundant reciprocal automatically.
  const normalized: FranchiseEdge[] = edges.map((e) => {
    if (e.relationType === 'PARENT') {
      return { from: e.to, to: e.from, relationType: 'SIDE_STORY' };
    }
    // PREQUEL (raw): `to` is the prequel of `from` (comes BEFORE it). Flip to
    // a forward SEQUEL so the arrow points chronologically (prequel → main)
    // AND so topoSortSpine — which only reads SEQUEL — picks up the ordering.
    // Without this, a relationship expressed only as PREQUEL was rendered
    // backwards and ignored by the chain ordering.
    if (e.relationType === 'PREQUEL') {
      return { from: e.to, to: e.from, relationType: 'SEQUEL' };
    }
    if (e.relationType === 'ALTERNATIVE') {
      return e.from <= e.to ? e : { from: e.to, to: e.from, relationType: 'ALTERNATIVE' };
    }
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
  const afterReciprocal = uniqued.filter((e) => {
    const keep = RECIPROCAL_DROPS.get(e.relationType);
    if (keep != null && present.has(`${e.to}|${e.from}|${keep}`)) return false;
    return true;
  });

  // 4. Collapse multiple edges between the SAME ordered pair to a single edge.
  //    AniList often tags one relationship two ways (e.g. manga SPIN_OFF→x AND
  //    x PARENT→manga, which normalizes to manga SIDE_STORY→x) — same direction,
  //    different type → two arrows for one link. Keep the most structural type.
  const TYPE_PRIORITY: Record<string, number> = {
    SEQUEL: 0, PREQUEL: 0, ADAPTATION: 1, SOURCE: 1,
    SIDE_STORY: 2, PARENT: 2, SPIN_OFF: 3, SUMMARY: 4,
    COMPILATION: 4, CONTAINS: 4, ALTERNATIVE: 5, CHARACTER: 6, OTHER: 7,
  };
  const bestByPair = new Map<string, FranchiseEdge>();
  for (const e of afterReciprocal) {
    const key = `${e.from}|${e.to}`;
    const cur = bestByPair.get(key);
    if (!cur) { bestByPair.set(key, e); continue; }
    const pe = TYPE_PRIORITY[e.relationType] ?? 8;
    const pc = TYPE_PRIORITY[cur.relationType] ?? 8;
    if (pe < pc) bestByPair.set(key, e);
  }
  return [...bestByPair.values()];
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
 * The franchise root is the FIRST instance of the franchise to exist — the
 * earliest-released node. A node that has a source/parent above it can never
 * be the root, and "earliest release" captures that directly without trying to
 * reason about (often-inverted) ADAPTATION directions: an original anime with
 * a later manga adaptation correctly stays the root, and a novel/manga that
 * predates its anime correctly is the root.
 *
 * Primary key: earliest year (seasonYear ?? startYear; unknown sorts last).
 * Tie-break: smallest anilistId (originals are typically catalogued first).
 */
export function findFranchiseRoot(graph: FranchiseGraph, _currentId: number): number | null {
  if (graph.nodes.length === 0) return null;
  const yearOf = (n: FranchiseNode) => n.seasonYear ?? n.startYear ?? Number.POSITIVE_INFINITY;
  let best: FranchiseNode | null = null;
  for (const n of graph.nodes) {
    if (best == null) { best = n; continue; }
    const ny = yearOf(n);
    const by = yearOf(best);
    if (ny < by || (ny === by && n.anilistId < best.anilistId)) best = n;
  }
  return best ? best.anilistId : null;
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

export function layoutFranchise(
  graph: FranchiseGraph,
  currentId: number,
  rootId?: number | null,
): Map<number, { x: number; y: number }> {
  const positions = new Map<number, { x: number; y: number }>();
  if (graph.nodes.length === 0) return positions;

  const nodeById = new Map(graph.nodes.map((n) => [n.anilistId, n]));
  const edges = dedupeReciprocalEdges(graph.edges, nodeById);
  const adj = buildAdjacency(edges);

  // Find ALL PREQUEL/SEQUEL connected components.
  const SPINE_RELATIONS = new Set(['PREQUEL', 'SEQUEL']);
  const seen = new Set<number>();
  const chains: Array<{ members: Set<number>; ordered: FranchiseNode[] }> = [];
  for (const n of graph.nodes) {
    if (seen.has(n.anilistId)) continue;
    const members = new Set<number>([n.anilistId]);
    seen.add(n.anilistId);
    const q: number[] = [n.anilistId];
    while (q.length > 0) {
      const id = q.shift()!;
      for (const e of adj.get(id) ?? []) {
        if (SPINE_RELATIONS.has(e.relationType) && !members.has(e.other)) {
          members.add(e.other);
          seen.add(e.other);
          q.push(e.other);
        }
      }
    }
    // Only chains of 2+ are shown — singletons skipped entirely.
    if (members.size >= 2) {
      chains.push({ members, ordered: topoSortSpine(members, edges, nodeById) });
    }
  }

  // Row ordering: BFS discovery from the root chain via every non-spine
  // visible edge. Newly-discovered chains are appended to the placed list,
  // EXCEPT alternative-connections, which are inserted directly below their
  // source chain. When a chain has multiple alts, the CLOSEST alt (by media
  // affinity — same type > same format > nearer year) ends up immediately
  // below, with farther alts cascading down.
  const rootChainIdx = rootId != null ? chains.findIndex((c) => c.members.has(rootId)) : -1;
  type Chain = (typeof chains)[number];
  if (rootChainIdx >= 0) {
    // Per-edge affinity score between two specific nodes — lower = closer.
    const scoreEdgePair = (sourceId: number, targetId: number): number => {
      const src = nodeById.get(sourceId);
      const tgt = nodeById.get(targetId);
      if (!src || !tgt) return Number.POSITIVE_INFINITY;
      let s = 0;
      if (src.type !== tgt.type) s += 1_000_000;
      if (src.format !== tgt.format) s += 1_000;
      const sy = src.seasonYear ?? src.startYear ?? 0;
      const ty = tgt.seasonYear ?? tgt.startYear ?? 0;
      s += Math.abs(sy - ty);
      return s;
    };
    const chainOfNode = new Map<number, Chain>();
    for (const c of chains) for (const id of c.members) chainOfNode.set(id, c);
    const neighborsByChain = new Map<Chain, Array<{ chain: Chain; isAlt: boolean; weight: number }>>();
    for (const c of chains) neighborsByChain.set(c, []);
    const seenEdge = new Set<string>();
    for (const e of edges) {
      if (e.relationType === 'PREQUEL' || e.relationType === 'SEQUEL') continue;
      const fromChain = chainOfNode.get(e.from);
      const toChain = chainOfNode.get(e.to);
      if (!fromChain || !toChain || fromChain === toChain) continue;
      const isAlt = e.relationType === 'ALTERNATIVE';
      const k1 = `${chains.indexOf(fromChain)}|${chains.indexOf(toChain)}|${e.relationType}`;
      const k2 = `${chains.indexOf(toChain)}|${chains.indexOf(fromChain)}|${e.relationType}`;
      if (seenEdge.has(k1) || seenEdge.has(k2)) continue;
      seenEdge.add(k1);
      neighborsByChain.get(fromChain)!.push({
        chain: toChain, isAlt, weight: scoreEdgePair(e.from, e.to),
      });
      neighborsByChain.get(toChain)!.push({
        chain: fromChain, isAlt, weight: scoreEdgePair(e.to, e.from),
      });
    }

    // Parent/source chains of root go ABOVE root, in discovery order, before
    // BFS expands the rest of the graph.
    //   to=root with ADAPTATION  → from is root's print source     (parent)
    //   to=root with SIDE_STORY  → from is root's main         (parent)
    //   from=root with PARENT    → root has `to` as parent      (parent)
    //   from=root with SOURCE    → root has `to` as source      (parent)
    const isRootParentEdge = (e: FranchiseEdge): boolean => {
      if (e.to === rootId && (e.relationType === 'ADAPTATION' || e.relationType === 'SIDE_STORY')) return true;
      if (e.from === rootId && (e.relationType === 'PARENT' || e.relationType === 'SOURCE')) return true;
      return false;
    };
    const rootChain = chains[rootChainIdx];
    const parentChains: Chain[] = [];
    for (const e of edges) {
      if (!isRootParentEdge(e)) continue;
      const other = e.from === rootId ? e.to : e.from;
      const otherChain = chainOfNode.get(other);
      if (!otherChain || otherChain === rootChain) continue;
      if (!parentChains.includes(otherChain)) parentChains.push(otherChain);
    }

    const placed: Chain[] = [...parentChains, rootChain];
    const placedSet = new Set<Chain>(placed);
    // Seed the BFS queue with parent chains first (so their own connections
    // are walked) then root.
    const queue: Chain[] = [...parentChains, rootChain];
    while (queue.length > 0) {
      const cur = queue.shift()!;
      const conns = neighborsByChain.get(cur) ?? [];
      // Order so alts come first (need their "just below" slot before non-alts
      // append at the bottom), and within alts FARTHER ones come first — each
      // splice puts the new chain at curIdx+1, so the LAST alt processed (the
      // CLOSEST) ends up immediately below the source.
      const sorted = [...conns].sort((a, b) => {
        if (a.isAlt !== b.isAlt) return a.isAlt ? -1 : 1;
        if (a.isAlt && b.isAlt) return b.weight - a.weight; // descending → farthest first
        return 0;
      });
      for (const { chain: next, isAlt } of sorted) {
        if (placedSet.has(next)) continue;
        placedSet.add(next);
        const curIdx = placed.indexOf(cur);
        if (isAlt) placed.splice(curIdx + 1, 0, next); // directly below cur, cascade rest down
        else placed.push(next);                          // append at end
        queue.push(next);
      }
    }
    // Any leftover (no edge to anything placed) goes at the very end.
    for (const c of chains) if (!placedSet.has(c)) placed.push(c);

    chains.length = 0;
    chains.push(...placed);
  } else {
    // No root identified — fall back to anchoring on the current viewing chain.
    const anchorIdx = chains.findIndex((c) => c.members.has(currentId));
    if (anchorIdx > 0) {
      const [anchor] = chains.splice(anchorIdx, 1);
      chains.unshift(anchor);
    }
  }

  // Position: each chain at its own y row, evenly spaced from x=0.
  chains.forEach((chain, rowIdx) => {
    chain.ordered.forEach((node, colIdx) => {
      positions.set(node.anilistId, {
        x: colIdx * SPINE_X_GAP,
        y: rowIdx * V_GAP,
      });
    });
  });

  return positions;
}

/** Given source and target positions, pick handle ids that route the edge
 *  along the dominant axis. */
/** Relation types that must always render as a vertical edge from the source's
 *  bottom handle to the child's top handle, regardless of relative position.
 *  These are the "lineage" edges (source/adaptation, parent/side-story) where
 *  the visual hierarchy is the message. */
const FORCED_VERTICAL_RELATIONS = new Set([
  'SOURCE', 'ADAPTATION', 'PARENT', 'SIDE_STORY',
]);

/** Default to the center slot. The caller does a per-(node,side) pass after
 *  ALL edges are built and rewrites handle IDs to spread slots only when
 *  multiple arrow types collide on the same side of the same node. */
export function pickHandles(
  src: { x: number; y: number },
  tgt: { x: number; y: number },
  relationType?: string,
): { sourceHandle: string; targetHandle: string } {
  if (relationType != null && FORCED_VERTICAL_RELATIONS.has(relationType)) {
    // Still forced vertical, but pick direction from actual positions so a
    // child placed ABOVE its source (e.g. a SIDE_STORY singleton above its
    // target chain) connects child.bottom → source.top instead of crashing
    // through the source card.
    return tgt.y >= src.y
      ? { sourceHandle: 'bottom-c-s', targetHandle: 'top-c-t' }
      : { sourceHandle: 'top-c-s',    targetHandle: 'bottom-c-t' };
  }
  const dx = tgt.x - src.x;
  const dy = tgt.y - src.y;
  if (Math.abs(dy) >= Math.abs(dx)) {
    return dy < 0
      ? { sourceHandle: 'top-c-s',    targetHandle: 'bottom-c-t' }
      : { sourceHandle: 'bottom-c-s', targetHandle: 'top-c-t'    };
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
export function canonicalRelation(relationType: string, target: FranchiseNode | undefined): string {
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
