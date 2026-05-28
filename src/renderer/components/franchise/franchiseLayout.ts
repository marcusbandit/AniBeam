import type { FranchiseGraph, FranchiseNode } from '../../../shared/franchise';
import { relationLane } from './laneAssignment';

/** Layout constants. Tweak these visual knobs in one place. */
const SPINE_X_GAP = 280;   // horizontal pixels between adjacent spine nodes
const LANE_Y_GAP  = 420;   // vertical pixels between stacked nodes (one node height + breathing room)

const SPINE_RELATIONS = new Set(['PREQUEL', 'SEQUEL']);

interface AdjEdge { other: number; relationType: string; direction: 'out' | 'in'; }

function buildAdjacency(edges: FranchiseGraph['edges']): Map<number, AdjEdge[]> {
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

/**
 * Compute (x, y) positions for every node in the franchise graph using a
 * spine-centric compass layout:
 *   1. The spine is the connected component of PREQUEL/SEQUEL edges that
 *      contains the current node. Spine nodes sit on y=0, ordered left→right
 *      by seasonYear (then anilistId for stable tie-breaks).
 *   2. Every non-spine node gets an "anchor" (the closest spine node, found
 *      via BFS through any edges) and a lane (top / bottom / sidebranch)
 *      derived from the relation that connects it to its anchor.
 *   3. Within each (anchor, lane) bucket, nodes stack vertically away from
 *      the spine, in seasonYear order. Top lane stacks upward, bottom lane
 *      (and side-branch for v1) stacks downward.
 */
export function layoutFranchise(graph: FranchiseGraph, currentId: number): Map<number, { x: number; y: number }> {
  const positions = new Map<number, { x: number; y: number }>();
  if (graph.nodes.length === 0) return positions;

  const adj = buildAdjacency(graph.edges);
  const nodeById = new Map(graph.nodes.map((n) => [n.anilistId, n]));

  // 1. Spine = BFS from currentId through PREQUEL/SEQUEL edges (either direction).
  const spineSet = new Set<number>();
  spineSet.add(currentId);
  const spineQ: number[] = [currentId];
  while (spineQ.length > 0) {
    const id = spineQ.shift()!;
    for (const e of adj.get(id) ?? []) {
      if (SPINE_RELATIONS.has(e.relationType) && !spineSet.has(e.other)) {
        spineSet.add(e.other);
        spineQ.push(e.other);
      }
    }
  }

  // 2. Order spine by seasonYear (ascending), then anilistId for stability.
  const spineList = [...spineSet]
    .map((id) => nodeById.get(id))
    .filter((n): n is FranchiseNode => n != null)
    .sort((a, b) => {
      const ay = a.seasonYear ?? Number.POSITIVE_INFINITY;
      const by = b.seasonYear ?? Number.POSITIVE_INFINITY;
      if (ay !== by) return ay - by;
      return a.anilistId - b.anilistId;
    });

  // 3. Spine positions on y=0.
  spineList.forEach((node, i) => {
    positions.set(node.anilistId, { x: i * SPINE_X_GAP, y: 0 });
  });

  // 4. For every non-spine node, find anchor + lane.
  type BucketKey = string; // `${anchorId}:${'top'|'bottom'|'branch'}`
  const buckets = new Map<BucketKey, FranchiseNode[]>();

  // Helper: BFS from a node through ANY edge to find nearest spine member.
  const findAnchor = (startId: number): number => {
    const visited = new Set<number>([startId]);
    const queue: number[] = [startId];
    while (queue.length > 0) {
      const id = queue.shift()!;
      for (const e of adj.get(id) ?? []) {
        if (visited.has(e.other)) continue;
        if (spineSet.has(e.other)) return e.other;
        visited.add(e.other);
        queue.push(e.other);
      }
    }
    return currentId; // disconnected — fall back to current (defensive)
  };

  // Helper: find the relationType "describing" this node — prefer an edge
  // from anchor → node (most direct, anchor's perspective); else any incoming.
  const relationFor = (anchor: number, nodeId: number): string => {
    for (const e of graph.edges) {
      if (e.from === anchor && e.to === nodeId) return e.relationType;
    }
    for (const e of graph.edges) {
      if (e.to === nodeId) return e.relationType;
    }
    return 'OTHER';
  };

  for (const node of graph.nodes) {
    if (spineSet.has(node.anilistId)) continue;
    const anchor = findAnchor(node.anilistId);
    const rt = relationFor(anchor, node.anilistId);
    const lane = relationLane(rt, node.type, node.format);
    if (lane === 'excluded') continue; // safety; CHARACTER is already filtered upstream
    const laneKey: 'top' | 'bottom' | 'branch' = lane === 'top' ? 'top' : (lane === 'sidebranch' ? 'branch' : 'bottom');
    const key = `${anchor}:${laneKey}`;
    const arr = buckets.get(key);
    if (arr) arr.push(node); else buckets.set(key, [node]);
  }

  // 5. Position non-spine nodes within each bucket, stacking away from spine.
  for (const [key, nodes] of buckets) {
    const [anchorIdStr, lane] = key.split(':');
    const anchorId = Number(anchorIdStr);
    const anchorPos = positions.get(anchorId);
    if (!anchorPos) continue;

    nodes.sort((a, b) => {
      const ay = a.seasonYear ?? Number.POSITIVE_INFINITY;
      const by = b.seasonYear ?? Number.POSITIVE_INFINITY;
      if (ay !== by) return ay - by;
      return a.anilistId - b.anilistId;
    });

    const yDir = lane === 'top' ? -1 : 1; // branch lays out below for v1
    nodes.forEach((node, i) => {
      positions.set(node.anilistId, {
        x: anchorPos.x,
        y: anchorPos.y + yDir * (i + 1) * LANE_Y_GAP,
      });
    });
  }

  return positions;
}

/** Given source and target positions, pick handle ids that route the edge
 *  along the dominant axis (vertical if |dy| >= |dx|, horizontal otherwise). */
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
