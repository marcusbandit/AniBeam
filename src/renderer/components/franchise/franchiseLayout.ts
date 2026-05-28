import type { FranchiseEdge, FranchiseGraph, FranchiseNode } from '../../../shared/franchise';
import { relationLane, relationLabel } from './laneAssignment';

// ── Visual layout constants ─────────────────────────────────────────────────
const H_GAP      = 240;   // min horizontal slot per leaf
const V_GAP      = 500;   // vertical distance between chain rows
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
 * Drop reciprocal duplicate edges. For each edge whose relationType is in
 * RECIPROCAL_DROPS, if the reciprocal edge exists in the opposite direction
 * with the "kept" relationType, drop this one. Asymmetric edges (only one
 * direction exists in the graph) are kept as-is.
 *
 * When nodeById is provided, also collapses same-type ADAPTATION and SOURCE
 * reciprocals by keeping only the canonical direction:
 *   ADAPTATION canonical: print → screen (source material is the `from`)
 *   SOURCE canonical:     screen → print (the referencing node is the `from`)
 */
export function dedupeReciprocalEdges(
  edges: ReadonlyArray<FranchiseEdge>,
  nodeById?: ReadonlyMap<number, FranchiseNode>,
): FranchiseEdge[] {
  const present = new Set<string>(edges.map((e) => `${e.from}|${e.to}|${e.relationType}`));
  return edges.filter((e) => {
    // Existing different-type reciprocal drop
    const keep = RECIPROCAL_DROPS.get(e.relationType);
    if (keep != null && present.has(`${e.to}|${e.from}|${keep}`)) return false;

    // Same-type ADAPTATION reciprocal: keep print → screen, drop screen → print
    if (nodeById && e.relationType === 'ADAPTATION'
        && present.has(`${e.to}|${e.from}|ADAPTATION`)) {
      const from = nodeById.get(e.from);
      const to = nodeById.get(e.to);
      if (from && to) {
        const fromPrint = isPrintTarget(from);
        const toPrint = isPrintTarget(to);
        // canonical: from is print, to is screen → keep
        if (fromPrint && !toPrint) { /* keep */ }
        // anti-canonical: from is screen, to is print → drop
        else if (!fromPrint && toPrint) return false;
        // both same kind → tiebreak by id (lower from wins)
        else if (e.from > e.to) return false;
      }
    }

    // Same-type SOURCE reciprocal: keep screen → print, drop print → screen
    if (nodeById && e.relationType === 'SOURCE'
        && present.has(`${e.to}|${e.from}|SOURCE`)) {
      const from = nodeById.get(e.from);
      const to = nodeById.get(e.to);
      if (from && to) {
        const fromPrint = isPrintTarget(from);
        const toPrint = isPrintTarget(to);
        // canonical: from is screen, to is print → keep
        if (!fromPrint && toPrint) { /* keep */ }
        // anti-canonical: from is print, to is screen → drop
        else if (fromPrint && !toPrint) return false;
        // both same kind → tiebreak by id (lower from wins)
        else if (e.from > e.to) return false;
      }
    }

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

  // 1. Dedupe reciprocal edges + build adjacency.
  const nodeById = new Map(graph.nodes.map((n) => [n.anilistId, n]));
  const edges = dedupeReciprocalEdges(graph.edges, nodeById);
  const adj = buildAdjacency(edges);

  // ── Step 1: Find all PREQUEL/SEQUEL chains (connected components). ──────────
  const SPINE_RELS = new Set(['PREQUEL', 'SEQUEL']);
  const chainOf = new Map<number, number>(); // nodeId → chainIndex
  const chains: Array<{ nodes: Set<number>; ordered: FranchiseNode[] }> = [];

  for (const n of graph.nodes) {
    if (chainOf.has(n.anilistId)) continue;
    const members = new Set<number>([n.anilistId]);
    const queue: number[] = [n.anilistId];
    while (queue.length > 0) {
      const id = queue.shift()!;
      for (const e of adj.get(id) ?? []) {
        if (SPINE_RELS.has(e.relationType) && !members.has(e.other)) {
          members.add(e.other);
          queue.push(e.other);
        }
      }
    }
    const idx = chains.length;
    for (const id of members) chainOf.set(id, idx);
    chains.push({ nodes: members, ordered: topoSortSpine(members, edges, nodeById) });
  }

  // ── Step 2: Anchor chain = the chain containing currentId. ─────────────────
  const anchorIdx = chainOf.get(currentId) ?? 0;
  const anchorChain = chains[anchorIdx];

  // ── Step 3: Position anchor chain: current at x=0, others by topo offset. ──
  const anchorOrder = anchorChain.ordered;
  const currentTopoIdx = anchorOrder.findIndex((n) => n.anilistId === currentId);
  anchorOrder.forEach((node, i) => {
    positions.set(node.anilistId, { x: (i - currentTopoIdx) * SPINE_X_GAP, y: 0 });
  });
  const positionedChains = new Set<number>([anchorIdx]);
  const usedChainYs = new Set<number>([0]);

  // ── Step 4: BFS over chains by inter-chain connections. ────────────────────
  // Keep processing until no more reachable multi-node chains remain.
  let progress = true;
  while (progress) {
    progress = false;

    for (let ci = 0; ci < chains.length; ci++) {
      if (positionedChains.has(ci)) continue;
      if (chains[ci].nodes.size < 2) continue; // singletons handled later via tree

      // Does this chain connect to any already-positioned chain?
      let hasConn = false;
      for (const id of chains[ci].nodes) {
        for (const e of adj.get(id) ?? []) {
          const otherChain = chainOf.get(e.other);
          if (otherChain != null && positionedChains.has(otherChain)) {
            hasConn = true;
            break;
          }
        }
        if (hasConn) break;
      }
      if (!hasConn) continue;

      // ── Step 5: Determine direction (above/below) for this chain. ──────────
      // Any SOURCE or PARENT edge to a positioned chain is authoritative: chain
      // goes above. Only fall to below when there are downstream links and zero
      // source/parent links. Default (neither) is also above.
      let hasSourceLink = false;
      let hasOtherDownstreamLink = false;
      const xAnchors = new Map<number, number>(); // chainNodeId → x of connected positioned node

      for (const id of chains[ci].nodes) {
        const node = nodeById.get(id);
        if (!node) continue;
        for (const e of edges) {
          // Edge from `id` to a positioned node
          if (e.from === id) {
            const target = nodeById.get(e.to);
            const otherChain = chainOf.get(e.to);
            if (otherChain == null || !positionedChains.has(otherChain)) continue;
            if (target) {
              const canon = canonicalRelation(e.relationType, target);
              if (canon === 'SOURCE' || canon === 'PARENT') hasSourceLink = true;
              else if (canon !== 'ALTERNATIVE') hasOtherDownstreamLink = true;
            }
            if (!xAnchors.has(id)) xAnchors.set(id, positions.get(e.to)!.x);
          }
          // Edge from positioned node to `id` — reverse perspective
          if (e.to === id) {
            const otherChain = chainOf.get(e.from);
            if (otherChain == null || !positionedChains.has(otherChain)) continue;
            const reversed = REVERSE_RELATION[e.relationType] ?? e.relationType;
            const canon = canonicalRelation(reversed, node);
            if (canon === 'SOURCE' || canon === 'PARENT') hasSourceLink = true;
            else if (canon !== 'ALTERNATIVE') hasOtherDownstreamLink = true;
            if (!xAnchors.has(id)) xAnchors.set(id, positions.get(e.from)!.x);
          }
        }
      }
      // Source/parent wins outright → above. Only go below when there is at
      // least one downstream link and zero source/parent links.
      const dir: -1 | 1 = hasSourceLink ? -1 : (hasOtherDownstreamLink ? 1 : -1);

      // ── Step 6: Find the connected chain to inherit y from (most connections). ──
      const yByConnectedChain = new Map<number, number>();
      for (const id of chains[ci].nodes) {
        for (const e of adj.get(id) ?? []) {
          const otherChain = chainOf.get(e.other);
          if (otherChain == null || !positionedChains.has(otherChain)) continue;
          yByConnectedChain.set(otherChain, (yByConnectedChain.get(otherChain) ?? 0) + 1);
        }
      }
      let bestConnected = -1;
      let bestCount = 0;
      for (const [c, cnt] of yByConnectedChain) {
        if (cnt > bestCount) { bestCount = cnt; bestConnected = c; }
      }
      const baseY = bestConnected >= 0
        ? (positions.get(chains[bestConnected].ordered[0].anilistId)?.y ?? 0)
        : 0;
      // Each chain gets a unique y row. If the natural target is already taken,
      // step further in the same direction until free.
      let candidateY = baseY + dir * V_GAP;
      while (usedChainYs.has(candidateY)) {
        candidateY += dir * V_GAP;
      }
      const newY = candidateY;

      // ── Step 7: Position the chain. First pass: pin x for nodes with direct anchors. ──
      const placed = new Set<number>();
      for (const node of chains[ci].ordered) {
        if (xAnchors.has(node.anilistId)) {
          positions.set(node.anilistId, { x: xAnchors.get(node.anilistId)!, y: newY });
          placed.add(node.anilistId);
        }
      }

      // Second pass: interpolate runs of unanchored nodes between anchored bookends.
      const ord = chains[ci].ordered;
      let i = 0;
      while (i < ord.length) {
        if (placed.has(ord[i].anilistId)) { i++; continue; }
        let runEnd = i;
        while (runEnd + 1 < ord.length && !placed.has(ord[runEnd + 1].anilistId)) runEnd++;
        const runLen = runEnd - i + 1;
        let prevX: number | null = null;
        for (let j = i - 1; j >= 0; j--) {
          if (placed.has(ord[j].anilistId)) { prevX = positions.get(ord[j].anilistId)!.x; break; }
        }
        let nextX: number | null = null;
        for (let j = runEnd + 1; j < ord.length; j++) {
          if (placed.has(ord[j].anilistId)) { nextX = positions.get(ord[j].anilistId)!.x; break; }
        }
        let startX: number;
        let step: number;
        if (prevX != null && nextX != null) {
          step = (nextX - prevX) / (runLen + 1);
          startX = prevX + step;
        } else if (prevX != null) {
          step = SPINE_X_MIN;
          startX = prevX + step;
        } else if (nextX != null) {
          step = SPINE_X_MIN;
          startX = nextX - step * runLen;
        } else {
          step = SPINE_X_MIN;
          startX = 0;
        }
        for (let k = 0; k < runLen; k++) {
          positions.set(ord[i + k].anilistId, { x: startX + k * step, y: newY });
          placed.add(ord[i + k].anilistId);
        }
        i = runEnd + 1;
      }

      // ── Step 8: Strict topo enforcement — sequel must never end up left of prequel. ──
      for (let k = 1; k < ord.length; k++) {
        const prev = positions.get(ord[k - 1].anilistId)!;
        const cur = positions.get(ord[k].anilistId)!;
        if (cur.x < prev.x + SPINE_X_MIN) {
          positions.set(ord[k].anilistId, { x: prev.x + SPINE_X_MIN, y: cur.y });
        }
      }

      positionedChains.add(ci);
      usedChainYs.add(newY);
      progress = true;
    }
  }

  // ── Step 9: Singleton chains + non-chain-tree placement. ───────────────────
  // allSpineSet = union of every node in a positioned multi-node chain.
  // Singletons remain unplaced and become tree leaves attached to the closest
  // already-placed node via the BFS tree logic.
  const allSpineSet = new Set<number>();
  for (let ci = 0; ci < chains.length; ci++) {
    if (chains[ci].nodes.size >= 2 && positionedChains.has(ci)) {
      for (const id of chains[ci].nodes) allSpineSet.add(id);
    }
  }

  // Build BFS tree from all positioned chain nodes outward.
  const { children: treeChildren } = buildBfsTree(allSpineSet, adj);

  // Collect all chain nodes (for subtree placement of their BFS-tree children).
  const allChainNodes: FranchiseNode[] = [];
  for (let ci = 0; ci < chains.length; ci++) {
    if (chains[ci].nodes.size >= 2 && positionedChains.has(ci)) {
      for (const node of chains[ci].ordered) {
        if (nodeById.has(node.anilistId)) allChainNodes.push(node);
      }
    }
  }

  // Place each chain node's BFS-tree children.
  // Anchor row (y === 0): use top/bottom lane semantics (laneRelativeToSpine).
  // Non-anchor rows: ALL descendants go further away from the anchor (sign of y).
  for (const chainNode of allChainNodes) {
    const spinePos = positions.get(chainNode.anilistId);
    if (!spinePos) continue;
    const allKids = treeChildren.get(chainNode.anilistId) ?? [];
    if (allKids.length === 0) continue;

    const isAnchorRow = spinePos.y === 0;

    if (isAnchorRow) {
      // Anchor row: respect top/bottom lane semantics (existing behavior).
      const tops: number[] = [];
      const bots: number[] = [];
      for (const kid of allKids) {
        const kidNode = nodeById.get(kid);
        if (!kidNode) continue;
        const kidDir = laneRelativeToSpine(chainNode.anilistId, kid, kidNode, edges);
        (kidDir === -1 ? tops : bots).push(kid);
      }
      for (const [treeDir, kids] of [[-1, tops], [1, bots]] as const) {
        if (kids.length === 0) continue;
        const widths = kids.map((k) => measureSubtree(k, treeChildren));
        const total = widths.reduce((s, w) => s + w, 0);
        let left = spinePos.x - total / 2;
        for (let ki = 0; ki < kids.length; ki++) {
          const childCx = left + widths[ki] / 2;
          const childCy = spinePos.y + treeDir * V_GAP;
          placeSubtree(kids[ki], childCx, childCy, treeDir, treeChildren, positions);
          left += widths[ki];
        }
      }
    } else {
      // Non-anchor row: ALL descendants go AWAY from the anchor.
      const dir: -1 | 1 = spinePos.y < 0 ? -1 : 1;
      const widths = allKids.map((k) => measureSubtree(k, treeChildren));
      const total = widths.reduce((s, w) => s + w, 0);
      let left = spinePos.x - total / 2;
      for (let i = 0; i < allKids.length; i++) {
        const childCx = left + widths[i] / 2;
        const childCy = spinePos.y + dir * V_GAP;
        placeSubtree(allKids[i], childCx, childCy, dir, treeChildren, positions);
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
