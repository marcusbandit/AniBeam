import '@xyflow/react/dist/style.css';

import { createContext, memo, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import type { ReactNode } from 'react';
import {
  ReactFlow,
  ReactFlowProvider,
  Background,
  Handle,
  MarkerType,
  Panel,
  Position,
  useReactFlow,
  useStore,
  getBezierPath,
  type Node as RFNode,
  type Edge as RFEdge,
  type NodeProps,
  type EdgeProps,
} from '@xyflow/react';
import { Tv, Film, ZoomIn, ZoomOut, Maximize2, Maximize, Minimize, Library, LocateFixed } from 'lucide-react';

import type { FranchiseEdge, FranchiseGraph, FranchiseNode as FranchiseNodeData } from '../../../shared/franchise';
import { relationLabel } from './laneAssignment';
import { categoryFor, formatFor, type FranchiseCategory, type FranchiseFormat, FranchiseFilters } from './FranchiseFilters';
import { layoutFranchise, pickHandles, dedupeReciprocalEdges, spineOrderMap, relationLabelRelativeTo, findFranchiseRoot, canonicalRelation } from './franchiseLayout';
import { Tooltip } from '../primitives';

// ─── Types ───────────────────────────────────────────────────────────────────

export interface FranchiseGraphViewProps {
  graph: FranchiseGraph;
  currentAnilistId: number;
  resolveOwnedId: (node: FranchiseNodeData) => string | undefined;
  pickTitle: (n: { titleRomaji: string | null; titleEnglish: string | null }) => string;
  /** Current title-language ('JP'|'EN'). Passed so the layout memo re-runs
   *  node titles when the user toggles language (pickTitle itself lives in a
   *  ref to avoid re-running on unrelated parent renders). */
  titleLang?: string;
  onOpenInApp: (seriesId: string) => void;
  onOpenExternal: (node: FranchiseNodeData) => void;
  statusMarkerFor: (node: FranchiseNodeData) => ReactNode;
  anilistIcon: ReactNode;
  hiddenCategories?: ReadonlySet<FranchiseCategory>;
  onToggleCategory: (cat: FranchiseCategory) => void;
  hiddenFormats?: ReadonlySet<FranchiseFormat>;
  onToggleFormat: (fmt: FranchiseFormat) => void;
  /** True while the background AniList fill is in flight. */
  filling?: boolean;
}

interface FranchiseNodeFlowData extends Record<string, unknown> {
  node: FranchiseNodeData;
  title: string;
  isCurrent: boolean;
  isRoot: boolean;
  isStart: boolean;
  /** Visual-aid duplicate of another node; not interactive, render translucent. */
  isGhost: boolean;
  /** CSS transform-origin for the ghost's scale-down. Pin the edge that FACES
   *  the target so the ghost shrinks toward it: '50% 100%' when the ghost is
   *  above its target, '50% 0%' when below. */
  ghostScaleOrigin?: string;
  /** Ordered list of `${type}-${dir}` slot ids on each side (dir 's'=output,
   *  't'=input). Handles render at evenly-spaced positions centered on 50%. */
  topSlots?: ReadonlyArray<string>;
  bottomSlots?: ReadonlyArray<string>;
  ownedId: string | undefined;
  relLabel: string | null;
  statusMarker: ReactNode;
  anilistIcon: ReactNode;
  onOpenInApp: (seriesId: string) => void;
  onOpenExternal: (node: FranchiseNodeData) => void;
  dimmed: boolean;
}

// ─── Hover context (bypasses React Flow's node-data update path) ─────────────
//
// Dim state and hover-relative labels are computed inside FranchiseFlowNode by
// reading this context directly. This means hover state changes never touch
// React Flow's node store - only the affected node components re-render, so
// React Flow never re-applies transforms or recalculates edge paths on hover.

interface HoverCtx {
  hoveredId: number | null;
  /** IDs of the hovered node + its direct neighbours (null = no hover). */
  highlightSet: Set<number> | null;
  /** Ghost rfNode IDs that should stay lit while their origin or target is
   *  hovered. Separate from highlightSet because a ghost's data.node.anilistId
   *  equals its origin - and we DON'T want a ghost to inherit highlight just
   *  because its origin is a secondary neighbour of the hovered node. */
  highlightGhostIds: ReadonlySet<string> | null;
  /** Stable refs from the heavy memo, used for hover-relative labeling. */
  spineSet: Set<number>;
  spineOrder: Map<number, number>;
  visibleEdges: readonly FranchiseEdge[];
  nodeById: ReadonlyMap<number, FranchiseNodeData>;
  /** Ghost rfNode id → the anilistId of the node the ghost duplicates. */
  ghostOriginByGhostId: ReadonlyMap<string, number>;
  /** Origin anilistId → set of altIds reachable via that origin's ghosts. */
  ghostNeighborsByOrigin: ReadonlyMap<number, ReadonlySet<number>>;
}
const HoverContext = createContext<HoverCtx>({
  hoveredId: null,
  highlightSet: null,
  highlightGhostIds: null,
  spineSet: new Set(),
  spineOrder: new Map(),
  visibleEdges: [],
  nodeById: new Map(),
  ghostOriginByGhostId: new Map(),
  ghostNeighborsByOrigin: new Map(),
});

// ─── Constants ────────────────────────────────────────────────────────────────

const NODE_W = 180; // matches the .franchise-node CSS width
const NODE_H = 420; // poster (180×1.5 = 270) + body ~150
// Vertical rhythm between rows - chain rows, chain→singleton, and
// singleton→ghost all step by this so every vertical gap is consistent.
// Mirrors V_GAP in franchiseLayout.ts.
const ROW_GAP = 500;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/* Single source of truth for edge colors. The filter chip dots (franchise.css)
   MUST read from the same tokens so the legend never drifts from the graph:
   spine=teal, source=blue (same as parent-of-root), side=muted, alt=amber,
   character=rose, release-order=rose (thick solid), other=border-hover. */
function arrowColorFor(relationType: string): string {
  if (relationType === 'RELEASE_ORDER') return 'var(--accent-rose)';
  const cat = categoryFor(relationType);
  switch (cat) {
    case 'spine':       return 'var(--accent-primary)';
    case 'source':      return 'var(--accent-blue)';
    case 'alternative': return 'var(--accent-amber)';
    case 'character':   return 'var(--accent-rose)';
    case 'side':        return 'var(--text-muted)';
    case 'other':
    default:            return 'var(--border-hover)';
  }
}

// ─── Layout ──────────────────────────────────────────────────────────────────

interface ChainSettings {
  order: 'chrono' | 'release';
}
const DEFAULT_CHAIN_SETTINGS: ChainSettings = { order: 'chrono' };
function layoutGraph(
  graph: FranchiseGraph,
  currentId: number,
  rootId: number | null,
  hiddenCategories: ReadonlySet<FranchiseCategory>,
  hiddenFormats: ReadonlySet<FranchiseFormat>,
  chainSettings: ReadonlyMap<string, ChainSettings>,
  onUpdateChainSettings: (chainKey: string, patch: Partial<ChainSettings>) => void,
  inlineEnabled: boolean,
): {
  nodes: RFNode<FranchiseNodeFlowData>[];
  edges: RFEdge[];
  visibleEdges: FranchiseEdge[];
  ghostOriginByGhostId: Map<string, number>;
  ghostNeighborsByOrigin: Map<number, Set<number>>;
  collapsedNeighbors: Map<number, Set<number>>;
} {
  // Dedupe reciprocal edges (SOURCE↔ADAPTATION, PARENT↔SIDE_STORY, PREQUEL↔SEQUEL)
  const graphNodeById = new Map<number, FranchiseNodeData>(graph.nodes.map((n) => [n.anilistId, n]));
  const dedupedEdges = dedupeReciprocalEdges(graph.edges, graphNodeById);

  // The FORMAT filter is applied at the very end as a render-stage hide, NOT
  // here as a pre-layout drop. If we removed hidden-format nodes before layout,
  // hiding e.g. "Manga" would cut the edge of a novel that reaches the franchise
  // only THROUGH a manga, and the connectivity cull below would then delete the
  // novel too: the reported "hiding Manga also hides the novel" bug. So every
  // node flows through layout positioned (neighbours stay anchored), and
  // hidden-format cards + their dangling edges are stripped just before return.
  // CATEGORY filtering stays a structural pre-layout drop (via visibleEdges).
  const filteredNodes = graph.nodes;
  const filteredEdges = dedupedEdges;

  // Visible edges = every edge, minus those whose category the user has
  // hidden. categoryFor() is the single source of truth for relation types;
  // there is NO per-type allowlist here - CHARACTER (and any genuinely
  // unwanted type) is dropped upstream in closeGraph, and the category-hide
  // toggles are the only visibility control.
  const visibleEdges = filteredEdges.filter(
    (e) => !hiddenCategories.has(categoryFor(e.relationType)),
  );

  // Build incoming-relation map for the tree-parent fallback label.
  // Uses visibleEdges (chain-filtered) so nodes don't pick up labels from
  // edges that lead to non-visible nodes (e.g. filtered-out anime adaptations).
  const incoming = new Map<number, string>();
  for (const e of visibleEdges) {
    if (!incoming.has(e.to)) incoming.set(e.to, e.relationType);
  }

  // Determine which nodes remain visible: current node always shows;
  // other nodes are dropped if all their incident edges have been filtered out.
  const connectedNodeIds = new Set<number>();
  connectedNodeIds.add(currentId);
  for (const e of visibleEdges) {
    connectedNodeIds.add(e.from);
    connectedNodeIds.add(e.to);
  }

  const visibleNodes = filteredNodes.filter((n) => connectedNodeIds.has(n.anilistId));

  // Layout uses the *fully filtered* graph so positions reflect the actual
  // visible structure - category-filtered nodes are dropped, not just hidden.
  const filteredGraph: FranchiseGraph = { ...graph, nodes: visibleNodes, edges: visibleEdges };
  const rawPositions = layoutFranchise(filteredGraph, currentId, rootId);

  // ── Ghost target detection (must run before edge construction) ────────────
  // For each non-closest alternative target of the root, we'll render a small
  // ghost copy of the root above the alt and connect them with a short stub.
  // The original long alt edge is then suppressed.
  type AltPair = { otherId: number; otherNode: FranchiseNodeData };
  const ghostTargets: AltPair[] = [];
  const ghostTargetIds = new Set<number>();
  if (rootId != null) {
    const rootNode = visibleNodes.find((n) => n.anilistId === rootId);
    if (rootNode) {
      // Dedupe by otherId - raw AniList data sometimes has reciprocal
      // ALTERNATIVE edges (root→X and X→root) and we want to treat each
      // unique alt target exactly once.
      const altMap = new Map<number, AltPair>();
      for (const e of visibleEdges) {
        if (e.relationType !== 'ALTERNATIVE') continue;
        const fromRoot = e.from === rootId;
        const toRoot = e.to === rootId;
        if (!fromRoot && !toRoot) continue;
        const otherId = fromRoot ? e.to : e.from;
        if (altMap.has(otherId)) continue;
        const otherNode = visibleNodes.find((n) => n.anilistId === otherId);
        if (otherNode) altMap.set(otherId, { otherId, otherNode });
      }
      const altPairs = [...altMap.values()];
      if (altPairs.length > 1) {
        const score = (alt: FranchiseNodeData): number => {
          let s = 0;
          if (alt.type !== rootNode.type) s += 1_000_000;
          if (alt.format !== rootNode.format) s += 1_000;
          const ay = alt.seasonYear ?? alt.startYear ?? 0;
          const ry = rootNode.seasonYear ?? rootNode.startYear ?? 0;
          s += Math.abs(ay - ry);
          return s;
        };
        const sorted = [...altPairs].sort((a, b) => score(a.otherNode) - score(b.otherNode));
        for (const t of sorted.slice(1)) {
          ghostTargets.push(t);
          ghostTargetIds.add(t.otherId);
        }
      }
    }
  }

  // ── Push rows containing ghost-target alts down to make room for the ghost.
  // Each unique alt-row Y picks up GHOST_ROW_SHIFT of headroom; downstream
  // rows shift by the cumulative amount so spacing stays consistent.
  // 300px chosen to clear a scale(0.7) ghost (visual height ~294) plus a small
  // gap above and below.
  const GHOST_ROW_SHIFT = 300;
  const ghostRowYsAsc = [...new Set(
    ghostTargets
      .map((t) => rawPositions.get(t.otherId)?.y)
      .filter((y): y is number => y !== undefined),
  )].sort((a, b) => a - b);
  const positions = new Map<number, { x: number; y: number }>();
  for (const [id, pos] of rawPositions) {
    let shiftCount = 0;
    for (const gy of ghostRowYsAsc) if (pos.y >= gy) shiftCount++;
    positions.set(id, { x: pos.x, y: pos.y + shiftCount * GHOST_ROW_SHIFT });
  }

  // ── Chain detection: group nodes by Y. Each row IS a spine chain. ────────
  // chainKey = `chain-<smallest member id>` so it stays stable across re-renders.
  const chainsByKey = new Map<string, { y: number; members: FranchiseNodeData[]; settings: ChainSettings }>();
  {
    const byY = new Map<number, FranchiseNodeData[]>();
    for (const node of visibleNodes) {
      const p = positions.get(node.anilistId);
      if (!p) continue;
      const list = byY.get(p.y);
      if (list) list.push(node);
      else byY.set(p.y, [node]);
    }
    for (const [y, members] of byY) {
      const minId = Math.min(...members.map((m) => m.anilistId));
      const chainKey = `chain-${minId}`;
      const settings: ChainSettings = chainSettings.get(chainKey) ?? DEFAULT_CHAIN_SETTINGS;
      chainsByKey.set(chainKey, { y, members, settings });
    }
  }

  // ── Apply Release order: for chains in release order, reorder member X
  // positions by release year (reusing the existing X slots), then later we
  // also rewrite their edges.
  const releaseChains = new Set<string>();
  const releaseOrderByChain = new Map<string, number[]>(); // chainKey → ordered anilistIds
  for (const [chainKey, chain] of chainsByKey) {
    if (chain.settings.order !== 'release' || chain.members.length < 2) continue;
    releaseChains.add(chainKey);
    const xSlots = chain.members
      .map((m) => positions.get(m.anilistId)!.x)
      .sort((a, b) => a - b);
    const sortedByYear = [...chain.members].sort((a, b) => {
      const ay = a.seasonYear ?? a.startYear ?? Number.POSITIVE_INFINITY;
      const by = b.seasonYear ?? b.startYear ?? Number.POSITIVE_INFINITY;
      if (ay !== by) return ay - by;
      return a.anilistId - b.anilistId;
    });
    sortedByYear.forEach((node, i) => {
      positions.set(node.anilistId, { x: xSlots[i], y: chain.y });
    });
    releaseOrderByChain.set(chainKey, sortedByYear.map((n) => n.anilistId));
  }
  // Lookup: id → chainKey (for "are both endpoints in same release chain?" checks)
  const idToChainKey = new Map<number, string>();
  for (const [chainKey, chain] of chainsByKey) {
    for (const m of chain.members) idToChainKey.set(m.anilistId, chainKey);
  }

  // ── Inline-source shifting ────────────────────────────────────────────────
  // When the global Inline toggle is on, walk every chain top-to-bottom.
  // For each member left-to-right:
  //   forward case (avg source col >= my col): shift me+suffix right by the
  //     delta so this member lands under its source(s).
  //   backward case (avg source col < my col): leave me put - ghost every
  //     source that's behind me above the target instead, dropping the
  //     direct source→target edge.
  const SPINE_X_GAP = 320;
  const inlineChainsTopDown = inlineEnabled
    ? [...chainsByKey.values()]
        .filter((c) => c.members.length >= 1)
        .sort((a, b) => a.y - b.y)
    : [];
  type SourceGhost = { sourceId: number; targetId: number };
  const sourceGhosts: SourceGhost[] = [];
  // Which side a source ghost should sit on relative to its target. Inline
  // source ghosts default to 'above'; singleton ghosts follow the singleton's
  // far side (away from its chain). Keyed by targetId.
  const sourceGhostSideByTarget = new Map<number, 'above' | 'below'>();
  // Relation type backing each source ghost so its edge is colored correctly
  // (source → blue, side-story → grey, etc.) instead of a fixed green.
  const sourceGhostRelByTarget = new Map<number, string>();
  for (const chain of inlineChainsTopDown) {
    const ordered = [...chain.members]
      .map((m) => ({ m, x: positions.get(m.anilistId)?.x }))
      .filter((it): it is { m: FranchiseNodeData; x: number } => it.x !== undefined)
      .sort((a, b) => a.x - b.x)
      .map((it) => it.m);
    if (ordered.length === 0) continue;
    const cols = ordered.map((_, i) => i);
    for (let i = 0; i < ordered.length; i++) {
      const member = ordered[i];
      const sourcePairs: Array<{ sourceId: number; col: number; rel: string }> = [];
      for (const e of dedupedEdges) {
        if (e.to !== member.anilistId) continue;
        if (e.relationType !== 'ADAPTATION' && e.relationType !== 'SIDE_STORY') continue;
        const srcPos = positions.get(e.from);
        if (!srcPos) continue;
        sourcePairs.push({ sourceId: e.from, col: Math.round(srcPos.x / SPINE_X_GAP), rel: e.relationType });
      }
      if (sourcePairs.length === 0) continue;
      const sourceCols = sourcePairs.map((p) => p.col);
      const avgCol = Math.floor(sourceCols.reduce((a, b) => a + b, 0) / sourceCols.length);
      const myCol = cols[i];
      if (avgCol >= myCol) {
        const delta = avgCol - myCol;
        if (delta > 0) for (let j = i; j < ordered.length; j++) cols[j] += delta;
      } else {
        // Source behind target. Allow 1 col of leeway - that's close enough
        // to draw the direct diagonal edge without a ghost.
        for (const { sourceId, col, rel } of sourcePairs) {
          if (col >= myCol) continue;
          if (myCol - col <= 1) continue; // within 1 col → keep direct edge
          sourceGhosts.push({ sourceId, targetId: member.anilistId });
          sourceGhostRelByTarget.set(member.anilistId, rel);
        }
      }
    }
    ordered.forEach((m, i) => {
      positions.set(m.anilistId, { x: cols[i] * SPINE_X_GAP, y: chain.y });
    });
  }
  const sourceGhostByEdgeKey = new Map<string, SourceGhost>();
  for (const g of sourceGhosts) {
    sourceGhostByEdgeKey.set(`${g.sourceId}|${g.targetId}`, g);
  }

  // Make vertical room above each row that has at least one source-ghost so
  // the ghost doesn't overlap the row above (same pattern as alt-ghost shift).
  const SOURCE_GHOST_ROW_SHIFT = 300;
  const sourceGhostRowYs = [...new Set(
    sourceGhosts
      .map((g) => positions.get(g.targetId)?.y)
      .filter((y): y is number => y !== undefined),
  )].sort((a, b) => a - b);
  if (sourceGhostRowYs.length > 0) {
    const updated = new Map<number, { x: number; y: number }>();
    for (const [id, pos] of positions) {
      let shiftCount = 0;
      for (const gy of sourceGhostRowYs) if (pos.y >= gy) shiftCount++;
      updated.set(id, { x: pos.x, y: pos.y + shiftCount * SOURCE_GHOST_ROW_SHIFT });
    }
    for (const [id, p] of updated) positions.set(id, p);
  }

  // ── Singleton "satellite" placement (SIDE_STORY orbiters) ────────────────
  // Visible nodes that have SIDE_STORY edges to a positioned chain member
  // but aren't in any chain themselves (no PREQUEL/SEQUEL connections -
  // standalone OVAs, specials, supplementary novels, etc.). Place each in
  // the same column grid as its target, on whichever side is empty:
  // above when no source/alt ghost is already there, otherwise below.
  // Surrounding chain rows shift to make space.
  const altGhostTargetRowYs = new Set<number>();
  for (const t of ghostTargets) {
    const y = positions.get(t.otherId)?.y;
    if (y != null) altGhostTargetRowYs.add(y);
  }
  const sourceGhostTargetRowYs = new Set<number>();
  for (const g of sourceGhosts) {
    const y = positions.get(g.targetId)?.y;
    if (y != null) sourceGhostTargetRowYs.add(y);
  }

  interface SingletonPlan {
    id: number;
    primaryTargetId: number;
    rowY: number;
    placeAbove: boolean;
    midX: number;
    /** Cross-chain source/parent nodes that should ghost-attach to this
     *  singleton instead of drawing a long line to another chain. */
    ghostSourceIds: number[];
  }
  const singletonPlans: SingletonPlan[] = [];
  for (const node of visibleNodes) {
    if (positions.has(node.anilistId)) continue; // already in a chain
    // SIDE_STORY targets that position this singleton.
    const targetSet = new Set<number>();
    for (const e of dedupedEdges) {
      if (e.relationType !== 'SIDE_STORY') continue;
      if (e.from === node.anilistId && positions.has(e.to)) targetSet.add(e.to);
      if (e.to === node.anilistId && positions.has(e.from)) targetSet.add(e.from);
    }
    if (targetSet.size === 0) continue;
    // Determine the PRIMARY chain = the chain holding the most SIDE_STORY
    // targets (this is the chain the side-story "is of"). Position relative
    // to that chain's targets.
    const targets = [...targetSet];
    const countByChain = new Map<string, number>();
    for (const t of targets) {
      const k = idToChainKey.get(t);
      if (k) countByChain.set(k, (countByChain.get(k) ?? 0) + 1);
    }
    let primaryChainKey: string | undefined;
    let bestCount = -1;
    for (const [k, c] of countByChain) {
      if (c > bestCount) { bestCount = c; primaryChainKey = k; }
    }
    const primaryTargets = targets
      .filter((t) => idToChainKey.get(t) === primaryChainKey)
      .sort((a, b) => (positions.get(a)?.x ?? 0) - (positions.get(b)?.x ?? 0));
    if (primaryTargets.length === 0) continue;
    const primaryTargetId = primaryTargets[0];
    const rowY = positions.get(primaryTargetId)!.y;
    const primaryXs = primaryTargets.map((t) => positions.get(t)!.x);
    const midX = (Math.min(...primaryXs) + Math.max(...primaryXs)) / 2;

    // Cross-chain source/parent edges → ghost. Any edge touching this
    // singleton (source/adaptation/side-story/parent) whose far endpoint is
    // positioned but lives in a DIFFERENT chain than the primary one.
    const GHOSTABLE = new Set(['ADAPTATION', 'SOURCE', 'SIDE_STORY']);
    const ghostSourceIdSet = new Set<number>();
    let ghostRel = 'ADAPTATION';
    for (const e of dedupedEdges) {
      if (!GHOSTABLE.has(e.relationType)) continue;
      const far = e.from === node.anilistId ? e.to : (e.to === node.anilistId ? e.from : null);
      if (far == null || !positions.has(far)) continue;
      if (idToChainKey.get(far) === primaryChainKey) continue; // same chain → keep direct
      ghostSourceIdSet.add(far);
      ghostRel = e.relationType;
    }

    // Top is "occupied" when an alt or source ghost is already drawn above
    // this row → fall back to bottom in that case.
    const ghostSourceIds = [...ghostSourceIdSet];
    const topOccupied = altGhostTargetRowYs.has(rowY) || sourceGhostTargetRowYs.has(rowY);
    if (ghostSourceIds.length > 0) sourceGhostRelByTarget.set(node.anilistId, ghostRel);
    singletonPlans.push({
      id: node.anilistId,
      primaryTargetId,
      rowY,
      placeAbove: !topOccupied,
      midX,
      ghostSourceIds,
    });
  }

  // ── Occupancy grid for satellite placement (singletons + ghosts) ─────────
  // Every real node + already-placed satellite sits in a (column, row-band)
  // cell. Before dropping a new satellite into a cell we check it's free; if
  // taken we nudge to the nearest free column on that row so nothing overlaps.
  const cellKey = (x: number, y: number) =>
    `${Math.round(x / SPINE_X_GAP)}|${Math.round(y / ROW_GAP)}`;
  const occupiedCells = new Set<string>();
  const claimCell = (x: number, y: number): number => {
    if (!occupiedCells.has(cellKey(x, y))) { occupiedCells.add(cellKey(x, y)); return x; }
    const col0 = Math.round(x / SPINE_X_GAP);
    for (let d = 1; d <= 24; d++) {
      for (const col of [col0 + d, col0 - d]) {
        const nx = col * SPINE_X_GAP;
        if (!occupiedCells.has(cellKey(nx, y))) { occupiedCells.add(cellKey(nx, y)); return nx; }
      }
    }
    occupiedCells.add(cellKey(x, y));
    return x;
  };

  // Side-story group frames: when one show has >3 side stories, box them.
  const sideStoryFrameEdges: RFEdge[] = [];

  if (singletonPlans.length > 0) {
    // One ROW_GAP fits a card + gap. A singleton that also carries a
    // cross-chain source ghost stacks 2 cards on that side → 2 units.
    const SINGLETON_UNIT = ROW_GAP;
    // Per target-row, how many units of clearance the singleton stack needs.
    const unitsByRow = (above: boolean) => {
      const m = new Map<number, number>();
      for (const p of singletonPlans) {
        if (p.placeAbove !== above) continue;
        const need = p.ghostSourceIds.length > 0 ? 2 : 1;
        m.set(p.rowY, Math.max(m.get(p.rowY) ?? 0, need));
      }
      return m;
    };
    const aboveUnits = unitsByRow(true);
    const belowUnits = unitsByRow(false);
    if (aboveUnits.size > 0 || belowUnits.size > 0) {
      const shifted = new Map<number, { x: number; y: number }>();
      for (const [id, pos] of positions) {
        let units = 0;
        for (const [sy, u] of aboveUnits) if (pos.y >= sy) units += u;
        for (const [sy, u] of belowUnits) if (pos.y > sy) units += u;
        shifted.set(id, { x: pos.x, y: pos.y + units * SINGLETON_UNIT });
      }
      for (const [id, p] of shifted) positions.set(id, p);
    }
    // Occupancy grid (defined here so singleton placement can use it too).
    for (const [, pos] of positions) occupiedCells.add(cellKey(pos.x, pos.y));
    // Place each singleton in the newly-created space, one unit off its chain.
    // Route through claimCell so multiple singletons sharing a target (e.g.
    // several SIDE_STORY/spin-off manga all hanging off the root) spread into
    // separate columns instead of stacking on the exact same cell.
    for (const plan of singletonPlans) {
      const newTargetY = positions.get(plan.primaryTargetId)!.y;
      const yOffset = plan.placeAbove ? -SINGLETON_UNIT : SINGLETON_UNIT;
      const y = newTargetY + yOffset;
      const x = claimCell(plan.midX, y);
      positions.set(plan.id, { x, y });
    }
    // Frame shows with MORE THAN 3 side stories - box the group under a
    // "Side stories" label (anchored leftmost.top-s → rightmost.bottom-t).
    const plansByTarget = new Map<number, SingletonPlan[]>();
    for (const plan of singletonPlans) {
      const arr = plansByTarget.get(plan.primaryTargetId);
      if (arr) arr.push(plan); else plansByTarget.set(plan.primaryTargetId, [plan]);
    }
    for (const [targetId, group] of plansByTarget) {
      if (group.length <= 3) continue;
      const sorted = [...group].sort((a, b) => (positions.get(a.id)?.x ?? 0) - (positions.get(b.id)?.x ?? 0));
      const left = sorted[0].id;
      const right = sorted[sorted.length - 1].id;
      sideStoryFrameEdges.push({
        id: `sidestory-frame-${targetId}`,
        source: String(left),
        target: String(right),
        sourceHandle: 'top-s',
        targetHandle: 'bottom-t',
        type: 'sideStoryFrame',
        selectable: false,
        focusable: false,
      });
    }
    // Register cross-chain source/parent ghosts so they render via the shared
    // ghost pipeline (translucent source copy above the singleton + blue
    // edge), and drop the corresponding direct cross-graph lines.
    for (const plan of singletonPlans) {
      // Ghost sits on the singleton's FAR side from its chain: a singleton
      // placed above its chain → ghost above it; placed below → ghost below.
      if (plan.ghostSourceIds.length > 0) {
        sourceGhostSideByTarget.set(plan.id, plan.placeAbove ? 'above' : 'below');
      }
      for (const sourceId of plan.ghostSourceIds) {
        sourceGhosts.push({ sourceId, targetId: plan.id });
        // Direct edge could be stored in either orientation - drop both.
        sourceGhostByEdgeKey.set(`${sourceId}|${plan.id}`, { sourceId, targetId: plan.id });
        sourceGhostByEdgeKey.set(`${plan.id}|${sourceId}`, { sourceId, targetId: plan.id });
      }
    }

    // Side-story ↔ side-story connections that land on DIFFERENT rows would
    // draw long cross-row lines. Ghost the source endpoint next to the target
    // instead. Ghost the one on the HIGHER row (smaller index) as a copy by
    // the lower one - pick deterministically by id so it's stable.
    const singletonIds = new Set(singletonPlans.map((p) => p.id));
    const bandOf = (id: number) => Math.round((positions.get(id)?.y ?? 0) / ROW_GAP);
    for (const e of dedupedEdges) {
      if (!singletonIds.has(e.from) || !singletonIds.has(e.to)) continue;
      if (!positions.has(e.from) || !positions.has(e.to)) continue;
      if (bandOf(e.from) === bandOf(e.to)) continue; // same row → direct edge is fine
      // Ghost the source (e.from) adjacent to the target (e.to).
      const srcY = positions.get(e.from)!.y;
      const tgtY = positions.get(e.to)!.y;
      sourceGhosts.push({ sourceId: e.from, targetId: e.to });
      sourceGhostSideByTarget.set(e.to, srcY < tgtY ? 'above' : 'below');
      sourceGhostRelByTarget.set(e.to, e.relationType);
      sourceGhostByEdgeKey.set(`${e.from}|${e.to}`, { sourceId: e.from, targetId: e.to });
      sourceGhostByEdgeKey.set(`${e.to}|${e.from}`, { sourceId: e.from, targetId: e.to });
    }
  }

  // Seed the occupancy grid for the no-singleton case (when singletons exist
  // it was already seeded before placing them). Idempotent - Set.
  if (occupiedCells.size === 0) {
    for (const [, pos] of positions) occupiedCells.add(cellKey(pos.x, pos.y));
  }

  // ── Root anchor ──────────────────────────────────────────────────────────
  // The franchise root can resolve to a PRINT source (novel/manga) that only
  // connects via ADAPTATION - no spine, no side-story - so it never got a grid
  // position and showed only as translucent source-ghosts (→ "no root
  // visible"). Place it as a real card above the chain it adapts into, restore
  // its direct edges, and drop its redundant ghosts.
  if (rootId != null && !positions.has(rootId) && visibleNodes.some((n) => n.anilistId === rootId)) {
    // Positioned ADAPTATION targets of the root (canonical print→screen: root = from).
    const targets: number[] = [];
    for (const e of dedupedEdges) {
      if (e.relationType !== 'ADAPTATION') continue;
      if (e.from === rootId && positions.has(e.to)) targets.push(e.to);
    }
    if (targets.length > 0) {
      // Primary row = the chain row holding the most adaptation targets.
      const byRow = new Map<number, number[]>();
      for (const t of targets) {
        const y = positions.get(t)!.y;
        (byRow.get(y) ?? byRow.set(y, []).get(y)!).push(t);
      }
      let primaryRowY = 0; let best = -1;
      for (const [y, ts] of byRow) if (ts.length > best) { best = ts.length; primaryRowY = y; }
      const rowTargets = byRow.get(primaryRowY)!;
      const xs = rowTargets.map((t) => positions.get(t)!.x);
      const midX = (Math.min(...xs) + Math.max(...xs)) / 2;
      // Make a row of room above the primary chain: push it + everything below down.
      const shifted = new Map<number, { x: number; y: number }>();
      for (const [id, pos] of positions) {
        shifted.set(id, pos.y >= primaryRowY ? { x: pos.x, y: pos.y + ROW_GAP } : pos);
      }
      for (const [id, p] of shifted) positions.set(id, p);
      occupiedCells.clear();
      for (const [, pos] of positions) occupiedCells.add(cellKey(pos.x, pos.y));
      const rootY = (primaryRowY + ROW_GAP) - ROW_GAP; // = primaryRowY, now the freed gap
      positions.set(rootId, { x: claimCell(midX, rootY), y: rootY });
      // The real root replaces its ghosts: drop any source-ghost OF the root and
      // un-drop its direct ADAPTATION edges so they render from the real card.
      for (let i = sourceGhosts.length - 1; i >= 0; i--) {
        if (sourceGhosts[i].sourceId === rootId) {
          const t = sourceGhosts[i].targetId;
          sourceGhostByEdgeKey.delete(`${rootId}|${t}`);
          sourceGhostByEdgeKey.delete(`${t}|${rootId}`);
          sourceGhosts.splice(i, 1);
        }
      }
    }
  }

  // ── Compact above-singletons downward ────────────────────────────────────
  // The root-anchor shift pushes the spine (and thus singleton targets) down a
  // row without moving the singletons, leaving them floating two rows above
  // their target. Pull each above-singleton down to sit directly above its
  // target (targetY − ROW_GAP) when the cells below it are free. Skip
  // ghost-bearing singletons (their source ghost stacks above them).
  for (const plan of singletonPlans) {
    if (!plan.placeAbove || plan.ghostSourceIds.length > 0) continue;
    const cur = positions.get(plan.id);
    const targetY = positions.get(plan.primaryTargetId)?.y;
    if (!cur || targetY == null) continue;
    const desiredY = targetY - ROW_GAP;
    if (cur.y >= desiredY) continue; // already directly above (or below)
    occupiedCells.delete(cellKey(cur.x, cur.y));
    let newY = cur.y;
    for (let y = cur.y + ROW_GAP; y <= desiredY; y += ROW_GAP) {
      if (occupiedCells.has(cellKey(cur.x, y))) break;
      newY = y;
    }
    occupiedCells.add(cellKey(cur.x, newY));
    positions.set(plan.id, { x: cur.x, y: newY });
  }

  // Compute spine order for reference-relative labeling
  const { spineSet, order: spineOrder } = spineOrderMap(
    { ...graph, nodes: visibleNodes, edges: visibleEdges },
    currentId,
  );

  // Node lookup map for label canonicalization
  const layoutNodeById = new Map<number, FranchiseNodeData>(visibleNodes.map((n) => [n.anilistId, n]));

  // Spine-start detection: a node is the "Start" of a SEQUEL chain in the
  // visible graph if it has no incoming SEQUEL edges (nothing came before it)
  // and at least one outgoing SEQUEL edge (it leads to a chain).
  const seqIn = new Map<number, number>();
  const seqOut = new Map<number, number>();
  for (const e of visibleEdges) {
    if (e.relationType !== 'SEQUEL') continue;
    seqIn.set(e.to,   (seqIn.get(e.to)   ?? 0) + 1);
    seqOut.set(e.from, (seqOut.get(e.from) ?? 0) + 1);
  }
  const startIds = new Set<number>();
  for (const n of visibleNodes) {
    if ((seqIn.get(n.anilistId) ?? 0) === 0 && (seqOut.get(n.anilistId) ?? 0) > 0) {
      startIds.add(n.anilistId);
    }
  }

  // ── Collapse a shared source/parent into ONE line to the chain frame ──────
  // When EVERY member of a framed chain shares the same source/parent S, draw
  // a single line S → frame (a tiny anchor node on the frame's top edge)
  // instead of N lines to each entry. This OVERRIDES ghosting for S: if S was
  // ghosted into these members (a combined hub), the ghost is removed and the
  // real S connects once to the frame.
  const frameAnchorEdges: RFEdge[] = [];
  const collapsedEdgeKeys = new Set<string>(); // `${from}|${to}` direct edges to drop
  // Logical adjacency for collapsed source↔frame-member pairs, so hovering the
  // source still lights up the whole frame (and vice-versa) even though the
  // per-member edges were replaced by one frameLink edge.
  const collapsedNeighbors = new Map<number, Set<number>>();
  for (const [chainKey, chain] of chainsByKey) {
    if (chain.members.length < 2) continue;
    if (!chain.members.every((m) => positions.has(m.anilistId))) continue;
    // Per member: its source/parent neighbours (S → member, S a positioned
    // real node outside this chain). Ghost status is IGNORED - the frame
    // collapse takes precedence and prunes the ghost below.
    const perMember = chain.members.map((m) => {
      const s = new Set<number>();
      for (const e of dedupedEdges) {
        if (e.relationType !== 'ADAPTATION' && e.relationType !== 'SIDE_STORY') continue;
        if (e.to !== m.anilistId) continue;             // canonical: e.from = source/parent
        if (!positions.has(e.from)) continue;
        if (idToChainKey.get(e.from) === chainKey) continue; // within the same chain - skip
        s.add(e.from);
      }
      return s;
    });
    if (perMember.some((s) => s.size === 0)) continue; // not ALL members have a source
    // Intersect: sources shared by every member.
    let shared = new Set<number>(perMember[0]);
    for (let i = 1; i < perMember.length; i++) shared = new Set([...shared].filter((x) => perMember[i].has(x)));
    if (shared.size === 0) continue;

    const memberIds = new Set(chain.members.map((m) => m.anilistId));
    for (const S of shared) {
      // Remove any source-ghost of S that targeted these members - the real S
      // now connects to the frame directly.
      for (let i = sourceGhosts.length - 1; i >= 0; i--) {
        if (sourceGhosts[i].sourceId === S && memberIds.has(sourceGhosts[i].targetId)) {
          sourceGhosts.splice(i, 1);
        }
      }
      // Connect S to the whole frame: a frameLink edge draws to the frame's
      // facing-edge centre using the members' LIVE measured rects (no NODE_H
      // estimate, no overshoot). source/target are real nodes for React Flow's
      // bookkeeping; the path is computed from memberIds.
      const sPos = positions.get(S)!;
      let nearest = chain.members[0];
      for (const m of chain.members) {
        if (Math.abs(positions.get(m.anilistId)!.x - sPos.x) < Math.abs(positions.get(nearest.anilistId)!.x - sPos.x)) nearest = m;
      }
      const memberIdList = chain.members.map((m) => m.anilistId);
      // Leave the parent from the side that FACES the frame: frame above → top
      // handle, frame below → bottom handle. Without this React Flow picks a
      // default handle and the line can attach to the wrong side of the parent.
      const memberMidY = chain.members.reduce((a, m) => a + positions.get(m.anilistId)!.y, 0) / chain.members.length;
      const sSide = memberMidY < sPos.y ? 'top' : 'bottom';
      frameAnchorEdges.push({
        id: `frameedge-${chainKey}-${S}`,
        source: String(S),
        target: String(nearest.anilistId),
        sourceHandle: `${sSide}-source-s`,
        type: 'frameLink',
        data: { memberIds: memberIdList, className: 'franchise-edge--adaptation', color: arrowColorFor('ADAPTATION') },
        markerEnd: { type: MarkerType.ArrowClosed, color: arrowColorFor('ADAPTATION'), width: 18, height: 18 },
      });
      for (const m of chain.members) collapsedEdgeKeys.add(`${S}|${m.anilistId}`);
      // Logical adjacency for hover: S ↔ every member.
      for (const m of chain.members) {
        (collapsedNeighbors.get(S) ?? collapsedNeighbors.set(S, new Set()).get(S)!).add(m.anilistId);
        (collapsedNeighbors.get(m.anilistId) ?? collapsedNeighbors.set(m.anilistId, new Set()).get(m.anilistId)!).add(S);
      }
    }
  }

  // ── Side-story frame: connect the shared parent ONCE to the frame ─────────
  // A side-story frame is, by definition, >3 singletons sharing one parent
  // (primaryTargetId). Instead of a line from the parent to each entry, draw
  // one line parent → side-story-frame.
  {
    const byParent = new Map<number, SingletonPlan[]>();
    for (const plan of singletonPlans) {
      const arr = byParent.get(plan.primaryTargetId);
      if (arr) arr.push(plan); else byParent.set(plan.primaryTargetId, [plan]);
    }
    for (const [parentId, group] of byParent) {
      if (group.length <= 3) continue;
      if (!positions.has(parentId)) continue;
      if (!group.every((p) => positions.has(p.id))) continue;
      // One frameLink edge parent → the side-story frame (draws to the frame's
      // facing edge using members' live rects).
      const pPos = positions.get(parentId)!;
      let nearest = group[0];
      for (const p of group) {
        if (Math.abs(positions.get(p.id)!.x - pPos.x) < Math.abs(positions.get(nearest.id)!.x - pPos.x)) nearest = p;
      }
      const memberIds = group.map((p) => p.id);
      // Leave the parent from the side that FACES the frame (top if the frame
      // sits above the parent, bottom if below).
      const memberMidY = group.reduce((a, p) => a + positions.get(p.id)!.y, 0) / group.length;
      const pSide = memberMidY < pPos.y ? 'top' : 'bottom';
      frameAnchorEdges.push({
        id: `ssframeedge-${parentId}`,
        source: String(parentId),
        target: String(nearest.id),
        sourceHandle: `${pSide}-side-s`,
        type: 'frameLink',
        data: { memberIds, className: 'franchise-edge--side_story', color: arrowColorFor('SIDE_STORY') },
        markerEnd: { type: MarkerType.ArrowClosed, color: arrowColorFor('SIDE_STORY'), width: 18, height: 18 },
      });
      // Drop the per-entry parent→side-story edges (both orientations) + add
      // logical adjacency so hovering the parent lights the whole frame.
      for (const plan of group) {
        collapsedEdgeKeys.add(`${parentId}|${plan.id}`);
        collapsedEdgeKeys.add(`${plan.id}|${parentId}`);
        (collapsedNeighbors.get(parentId) ?? collapsedNeighbors.set(parentId, new Set()).get(parentId)!).add(plan.id);
        (collapsedNeighbors.get(plan.id) ?? collapsedNeighbors.set(plan.id, new Set()).get(plan.id)!).add(parentId);
      }
    }
  }

  const rfNodes: RFNode<FranchiseNodeFlowData>[] = visibleNodes
    .filter((node) => positions.has(node.anilistId))
    .map((node) => {
    const isCurrent = node.anilistId === currentId;
    const isRoot = rootId != null && node.anilistId === rootId;
    const isStart = startIds.has(node.anilistId);
    const p = positions.get(node.anilistId) ?? { x: 0, y: 0 };

    // Reference-relative label (spine topology + direct edge), falling back to
    // tree-parent BFS label for multi-hop nodes.
    const relativeLabel = isCurrent
      ? null
      : relationLabelRelativeTo(currentId, node.anilistId, spineSet, spineOrder, visibleEdges, layoutNodeById);
    const fallbackLabel = incoming.get(node.anilistId);
    const baseLabel = isCurrent
      ? (isRoot ? 'Viewing · Root' : 'Viewing')
      : (relativeLabel ?? (fallbackLabel
          ? relationLabel(canonicalRelation(fallbackLabel, node))
          : null));
    // "Start" only surfaces when the node has no stronger identity:
    // Viewing and Root both already say "this is something" - Start would be redundant.
    const relLabel = (isStart && !isCurrent && !isRoot) ? 'Start' : baseLabel;

    return {
      id: String(node.anilistId),
      type: 'franchise',
      position: {
        x: p.x - NODE_W / 2,
        y: p.y - NODE_H / 2,
      },
      data: {
        node,
        title: '',       // enriched below in useMemo
        isCurrent,
        isRoot,
        isStart,
        isGhost: false,
        ownedId: undefined,
        relLabel,
        statusMarker: null,
        anilistIcon: null,
        onOpenInApp: () => {},
        onOpenExternal: () => {},
        dimmed: false,
      },
    };
  });

  // Only emit React Flow edges where both endpoints have a layout position.
  // The simplified layoutFranchise only positions spine/chain nodes, so edges
  // to non-chain nodes would be orphan edges (target node missing from rfNodes)
  // - React Flow's internal node-position lookups on those orphan edges trigger
  // cascading updates on every hover state change, causing the visible flash.
  const rfEdges: RFEdge[] = visibleEdges
    .filter((edge) => positions.has(edge.from) && positions.has(edge.to))
    // Drop alt edges that a ghost will replace - those go from ghost→alt
    // instead of root→alt and are appended below.
    .filter((edge) => {
      if (edge.relationType !== 'ALTERNATIVE') return true;
      if (rootId == null) return true;
      const other = edge.from === rootId ? edge.to : (edge.to === rootId ? edge.from : null);
      return other == null || !ghostTargetIds.has(other);
    })
    // Drop SEQUEL/PREQUEL edges INSIDE a release-mode chain - they get
    // replaced by artificial RELEASE_ORDER edges appended below.
    .filter((edge) => {
      if (edge.relationType !== 'SEQUEL' && edge.relationType !== 'PREQUEL') return true;
      const k1 = idToChainKey.get(edge.from);
      const k2 = idToChainKey.get(edge.to);
      if (k1 == null || k2 == null || k1 !== k2) return true;
      return !releaseChains.has(k1);
    })
    // Drop any direct edge that a ghost replaces (inline source ghosts,
    // cross-chain singleton sources, cross-row side-story↔side-story links).
    .filter((edge) => !sourceGhostByEdgeKey.has(`${edge.from}|${edge.to}`))
    // Drop per-member edges collapsed into a single source→frame line.
    .filter((edge) => !collapsedEdgeKeys.has(`${edge.from}|${edge.to}`))
    .map((edge) => {
      const { sourceHandle, targetHandle } = pickHandles(
        positions.get(edge.from) ?? { x: 0, y: 0 },
        positions.get(edge.to)   ?? { x: 0, y: 0 },
        edge.relationType,
      );
      // Parent-of-root edges get a heavier blue stroke so the lineage flowing into
      // the franchise root reads at a glance.
      const isParentOfRoot = rootId != null && (
        (edge.to === rootId && (edge.relationType === 'ADAPTATION' || edge.relationType === 'SIDE_STORY')) ||
        (edge.from === rootId && (edge.relationType === 'PARENT' || edge.relationType === 'SOURCE'))
      );
      // Alternative relations are symmetric - neither side is the "parent".
      // Render with arrows on both ends to communicate that.
      const isAlternative = edge.relationType === 'ALTERNATIVE';
      const color = isParentOfRoot ? 'var(--accent-blue)' : arrowColorFor(edge.relationType);
      const marker = { type: MarkerType.ArrowClosed, color, width: 18, height: 18 };
      return {
        id: `${edge.from}->${edge.to}:${edge.relationType}`,
        source: String(edge.from),
        target: String(edge.to),
        sourceHandle,
        targetHandle,
        type: 'default',
        className: `franchise-edge franchise-edge--${edge.relationType.toLowerCase()}${isParentOfRoot ? ' franchise-edge--parent-of-root' : ''}`,
        data: { relationType: edge.relationType },
        markerEnd: marker,
        ...(isAlternative ? { markerStart: marker } : {}),
      };
    });

  // ── Append ghost nodes + ghost edges (positions already shifted above) ────
  if (rootId != null && ghostTargets.length > 0) {
    const rootNode = visibleNodes.find((n) => n.anilistId === rootId);
    if (rootNode) {
      // Ghost sits one ROW_GAP above its target - same rhythm as every other
      // vertical step so all ghost gaps are consistent.
      for (const t of ghostTargets) {
        const altPos = positions.get(t.otherId);
        if (!altPos) continue;
        const ghostCenterY = altPos.y - ROW_GAP;
        const ghostCenterX = claimCell(altPos.x, ghostCenterY);
        const ghostId = `ghost-${rootId}-${t.otherId}`;

        rfNodes.push({
          id: ghostId,
          type: 'franchise',
          position: { x: ghostCenterX - NODE_W / 2, y: ghostCenterY - NODE_H / 2 },
          selectable: false,
          draggable: false,
          data: {
            node: rootNode,
            title: '',
            isCurrent: false,
            isRoot: false,
            isStart: false,
            isGhost: true,
            ghostScaleOrigin: '50% 100%', // alt ghosts always sit above
            ownedId: undefined,
            relLabel: 'Alternative',
            statusMarker: null,
            anilistIcon: null,
            onOpenInApp: () => {},
            onOpenExternal: () => {},
            dimmed: false,
          },
        });

        const handles = pickHandles(
          { x: ghostCenterX, y: ghostCenterY },
          altPos,
          'ALTERNATIVE',
        );
        const marker = {
          type: MarkerType.ArrowClosed,
          color: 'var(--accent-amber)',
          width: 18,
          height: 18,
        };
        rfEdges.push({
          id: `ghost-edge-${rootId}-${t.otherId}`,
          source: ghostId,
          target: String(t.otherId),
          sourceHandle: handles.sourceHandle,
          targetHandle: handles.targetHandle,
          type: 'default',
          className: 'franchise-edge franchise-edge--alternative franchise-edge--ghost',
          data: { relationType: 'ALTERNATIVE' },
          markerEnd: marker,
          markerStart: marker,
        });
      }
    }
  }

  // ── Artificial RELEASE_ORDER edges + chain frame nodes ───────────────────
  // For each release-mode chain, connect consecutive year-ordered members
  // with red arrows. Then drop a frame node around every chain (chrono and
  // release alike) that hosts the toggle widget.
  for (const [chainKey, ordered] of releaseOrderByChain) {
    for (let i = 0; i < ordered.length - 1; i++) {
      const fromId = ordered[i];
      const toId = ordered[i + 1];
      const fromPos = positions.get(fromId);
      const toPos = positions.get(toId);
      if (!fromPos || !toPos) continue;
      const handles = pickHandles(fromPos, toPos, 'RELEASE_ORDER');
      const color = arrowColorFor('RELEASE_ORDER');
      rfEdges.push({
        id: `release-${chainKey}-${fromId}->${toId}`,
        source: String(fromId),
        target: String(toId),
        sourceHandle: handles.sourceHandle,
        targetHandle: handles.targetHandle,
        type: 'default',
        className: 'franchise-edge franchise-edge--release_order',
        data: { relationType: 'RELEASE_ORDER' },
        markerEnd: { type: MarkerType.ArrowClosed, color, width: 18, height: 18 },
      });
    }
  }
  // Chain frame: split into TWO SVG edges per chain so the background can
  // paint BEHIND real arrows while the toolbar/toggle paints ABOVE them.
  //   - chainBg     → first in array, paints behind everything
  //   - chainToggle → last  in array, paints above everything (in the edge SVG)
  // Both use the same leftmost.top-s → rightmost.bottom-t handles so each
  // component receives the actual rendered top/bottom of the chain row.
  const frameBgEdges: RFEdge[] = [];
  const frameToggleEdges: RFEdge[] = [];
  for (const [chainKey, chain] of chainsByKey) {
    if (chain.members.length < 2) continue;
    const sortedByX = [...chain.members]
      .map((m) => ({ m, x: positions.get(m.anilistId)?.x }))
      .filter((it): it is { m: FranchiseNodeData; x: number } => it.x !== undefined)
      .sort((a, b) => a.x - b.x);
    if (sortedByX.length < 2) continue;
    const left = sortedByX[0].m.anilistId;
    const right = sortedByX[sortedByX.length - 1].m.anilistId;
    const sharedEdge = {
      source: String(left),
      target: String(right),
      sourceHandle: 'top-s',
      targetHandle: 'bottom-t',
      data: { chainKey, settings: chain.settings, onUpdate: onUpdateChainSettings },
      selectable: false,
      focusable: false,
    };
    frameBgEdges.push({
      ...sharedEdge,
      id: `chain-bg-${chainKey}`,
      type: 'chainBg',
    });
    frameToggleEdges.push({
      ...sharedEdge,
      id: `chain-toggle-${chainKey}`,
      type: 'chainToggle',
    });
  }

  // ── Source ghosts (inline mode, source-behind-target conflict) ───────────
  // For each conflict, drop a translucent copy of the SOURCE node above the
  // TARGET row, then connect with a short blue edge.
  // Grouping rules:
  //   1. Same source + same row → ONE combined ghost centered over the range
  //      of those targets, edges fanning out to each target.
  //   2. A target with multiple distinct sources still gets one ghost per
  //      source, spread horizontally so they don't stack.
  type GhostGroup = { sourceId: number; rowY: number; targetIds: number[] };
  const groupByKey = new Map<string, GhostGroup>();
  for (const g of sourceGhosts) {
    const p = positions.get(g.targetId);
    if (!p) continue;
    const key = `${g.sourceId}|${p.y}`;
    const existing = groupByKey.get(key);
    if (existing) existing.targetIds.push(g.targetId);
    else groupByKey.set(key, { sourceId: g.sourceId, rowY: p.y, targetIds: [g.targetId] });
  }
  // Pin each group's render-X: combined groups → midpoint of target Xs;
  // single-target groups → target X (will be spread later if needed).
  type Placed = GhostGroup & { renderX: number };
  const placedGroups: Placed[] = [];
  for (const group of groupByKey.values()) {
    const xs = group.targetIds
      .map((t) => positions.get(t)?.x)
      .filter((x): x is number => x !== undefined);
    if (xs.length === 0) continue;
    const renderX = (Math.min(...xs) + Math.max(...xs)) / 2;
    placedGroups.push({ ...group, renderX });
  }
  // Apply horizontal spread when MULTIPLE single-target groups land at the
  // exact same X (i.e., a target has more than one distinct source ghost).
  const singleGroupsByTargetX = new Map<string, Placed[]>(); // `${rowY}|${x}` → groups
  for (const g of placedGroups) {
    if (g.targetIds.length !== 1) continue;
    const k = `${g.rowY}|${g.renderX}`;
    const list = singleGroupsByTargetX.get(k);
    if (list) list.push(g); else singleGroupsByTargetX.set(k, [g]);
  }
  const GHOST_SPREAD_X = NODE_W * 0.7;
  for (const list of singleGroupsByTargetX.values()) {
    if (list.length <= 1) continue;
    // Sort by source X for stable left→right order, then spread.
    list.sort((a, b) => (positions.get(a.sourceId)?.x ?? 0) - (positions.get(b.sourceId)?.x ?? 0));
    const n = list.length;
    for (let i = 0; i < n; i++) {
      list[i].renderX += (i - (n - 1) / 2) * GHOST_SPREAD_X;
    }
  }
  // Fine-grained ghost collision: keep ghosts tightly packed (GHOST_SPREAD_X
  // steps) rather than snapping to full 320px columns like claimCell does -
  // the latter spread same-target source ghosts way too far apart.
  const ghostFineCells = new Set<string>();
  const fineKey = (x: number, y: number) =>
    `${Math.round(x / GHOST_SPREAD_X)}|${Math.round(y / ROW_GAP)}`;
  const claimFine = (x: number, y: number): number => {
    if (!ghostFineCells.has(fineKey(x, y))) { ghostFineCells.add(fineKey(x, y)); return x; }
    for (let d = 1; d <= 40; d++) {
      for (const nx of [x + d * GHOST_SPREAD_X, x - d * GHOST_SPREAD_X]) {
        if (!ghostFineCells.has(fineKey(nx, y))) { ghostFineCells.add(fineKey(nx, y)); return nx; }
      }
    }
    return x;
  };
  // Now render every group: one ghost node per group + one edge per target.
  for (const group of placedGroups) {
    const sourceNode = visibleNodes.find((n) => n.anilistId === group.sourceId);
    if (!sourceNode) continue;
    // Side: ghost above the target by default; below when the target asked for
    // it (a below-placed singleton). Group targets share a side here because
    // inline ghosts are always 'above' and a singleton is a single target.
    // Offset uses the SAME row rhythm (ROW_GAP = 500) as chain rows and the
    // chain→singleton gap, so every vertical step in the graph is consistent.
    const side = sourceGhostSideByTarget.get(group.targetIds[0]) ?? 'above';
    const ghostCenterY = side === 'below'
      ? group.rowY + ROW_GAP
      : group.rowY - ROW_GAP;
    // Keep ghosts tightly packed (fine step), not snapped to full columns.
    const ghostCenterX = claimFine(group.renderX, ghostCenterY);
    const sortedTargetIds = [...group.targetIds].sort((a, b) => a - b);
    const ghostId = `source-ghost-${group.sourceId}-${sortedTargetIds.join('+')}`;
    rfNodes.push({
      id: ghostId,
      type: 'franchise',
      position: { x: ghostCenterX - NODE_W / 2, y: ghostCenterY - NODE_H / 2 },
      selectable: false,
      draggable: false,
      data: {
        node: sourceNode,
        title: '',
        isCurrent: false,
        isRoot: false,
        isStart: false,
        isGhost: true,
        // Pin the edge facing the target: above → bottom, below → top.
        ghostScaleOrigin: side === 'below' ? '50% 0%' : '50% 100%',
        ownedId: undefined,
        relLabel: 'Source',
        statusMarker: null,
        anilistIcon: null,
        onOpenInApp: () => {},
        onOpenExternal: () => {},
        dimmed: false,
      },
    });
    // Color by the actual relation (source → blue), not a fixed green.
    const rel = sourceGhostRelByTarget.get(group.targetIds[0]) ?? 'ADAPTATION';
    const color = arrowColorFor(rel);
    for (const targetId of group.targetIds) {
      const targetPos = positions.get(targetId);
      if (!targetPos) continue;
      const handles = pickHandles(
        { x: ghostCenterX, y: ghostCenterY },
        targetPos,
        rel,
      );
      rfEdges.push({
        id: `source-ghost-edge-${group.sourceId}-${targetId}`,
        source: ghostId,
        target: String(targetId),
        sourceHandle: handles.sourceHandle,
        targetHandle: handles.targetHandle,
        type: 'default',
        className: `franchise-edge franchise-edge--${rel.toLowerCase()} franchise-edge--ghost`,
        data: { relationType: rel },
        markerEnd: { type: MarkerType.ArrowClosed, color, width: 18, height: 18 },
      });
    }
  }

  // Build maps to link ghost ↔ origin for the hover system. Both alternative
  // ghosts AND inline source ghosts get registered so hovering any ghost lights
  // up its origin, and hovering the origin highlights all ghost-connected
  // targets.
  const ghostOriginByGhostId = new Map<string, number>();
  const ghostNeighborsByOrigin = new Map<number, Set<number>>();
  const linkGhost = (ghostId: string, originId: number, targetId: number) => {
    ghostOriginByGhostId.set(ghostId, originId);
    const neighbors = ghostNeighborsByOrigin.get(originId) ?? new Set<number>();
    neighbors.add(targetId);
    ghostNeighborsByOrigin.set(originId, neighbors);
  };
  if (rootId != null && ghostTargets.length > 0) {
    for (const t of ghostTargets) {
      linkGhost(`ghost-${rootId}-${t.otherId}`, rootId, t.otherId);
    }
  }
  // Source ghosts are rendered as ONE rfNode per group (combined targets in
  // the id like `source-ghost-X-A+B+C`). The link map must use the SAME
  // combined ghost id - otherwise highlightGhostIds can't find the actual
  // rendered ghost and hovering a target won't keep the ghost lit.
  for (const group of placedGroups) {
    const sortedTargetIds = [...group.targetIds].sort((a, b) => a - b);
    const ghostId = `source-ghost-${group.sourceId}-${sortedTargetIds.join('+')}`;
    for (const targetId of group.targetIds) {
      linkGhost(ghostId, group.sourceId, targetId);
    }
  }

  // ── Per-(node, side) dynamic slot assignment ─────────────────────────────
  // For each top/bottom side of each node, collect the SET of arrow types
  // arriving/leaving. Each type gets its own handle ID; the node renders the
  // handles at evenly-spaced positions centered around 50% - N types →
  // positions (i+1)/(N+1) for i in [0,N). Single-type sides land at 50%.
  // No hardcoded slot positions.
  // Slot type granularity mirrors the EDGE COLOR categories so visually
  // distinct edges (e.g. ADAPTATION-source vs SIDE_STORY-side) get their OWN
  // slot instead of sharing one and overlapping.
  type ArrowType = 'source' | 'ghost' | 'side' | 'spine' | 'alt' | 'character' | 'other';
  const typeForRelation = (rel?: string): ArrowType | null => {
    if (!rel) return null;
    switch (rel) {
      case 'ADAPTATION': case 'SOURCE': case 'PARENT': return 'source';
      case 'SIDE_STORY': case 'SPIN_OFF': case 'SUMMARY': case 'COMPILATION': case 'CONTAINS': return 'side';
      case 'ALTERNATIVE': return 'alt';
      case 'CHARACTER': return 'character';
      case 'SEQUEL': case 'PREQUEL': case 'RELEASE_ORDER': return 'spine';
      default: return 'other';
    }
  };
  const typeForEdge = (e: RFEdge): ArrowType | null => {
    if (ghostOriginByGhostId.has(e.source) || ghostOriginByGhostId.has(e.target)) {
      return 'ghost';
    }
    return typeForRelation((e.data as { relationType?: string } | undefined)?.relationType);
  };
  const sideOfHandle = (h: string | null | undefined): 'top' | 'bottom' | null => {
    if (!h) return null;
    if (h.startsWith('top-')) return 'top';
    if (h.startsWith('bottom-')) return 'bottom';
    return null;
  };
  // A "slot" is a (type, direction) pair: an INPUT (target, '-t') and an
  // OUTPUT (source, '-s') each reserve their own slot on a side, so a side
  // carrying e.g. an outgoing spine and incoming sources spreads them apart
  // instead of stacking everything on dead-center. Slot id = `${type}-${dir}`.
  const CANONICAL_ORDER: ReadonlyArray<ArrowType> = ['source', 'ghost', 'side', 'spine', 'alt', 'character', 'other'];
  const slotRank = (slotId: string): number => {
    const dash = slotId.lastIndexOf('-');
    const type = slotId.slice(0, dash) as ArrowType;
    const dir = slotId.slice(dash + 1); // 's' | 't'
    const ti = CANONICAL_ORDER.indexOf(type);
    return (ti < 0 ? CANONICAL_ORDER.length : ti) * 2 + (dir === 's' ? 0 : 1);
  };
  const slotsByNode = new Map<string, { top: Set<string>; bottom: Set<string> }>();
  const ensureSlots = (nodeId: string) => {
    let s = slotsByNode.get(nodeId);
    if (!s) { s = { top: new Set(), bottom: new Set() }; slotsByNode.set(nodeId, s); }
    return s;
  };
  for (const e of rfEdges) {
    const t = typeForEdge(e);
    if (!t) continue;
    const sSide = sideOfHandle(e.sourceHandle);
    const tSide = sideOfHandle(e.targetHandle);
    if (sSide) ensureSlots(e.source)[sSide].add(`${t}-s`);
    if (tSide) ensureSlots(e.target)[tSide].add(`${t}-t`);
  }
  // frameLink edges (parent → frame) carry an explicit sourceHandle but live in
  // frameAnchorEdges, not rfEdges, so reserve their source slot here too - else
  // the handle never renders and the line snaps to a default (wrong) side.
  for (const e of frameAnchorEdges) {
    const side = sideOfHandle(e.sourceHandle);
    if (!side) continue;
    ensureSlots(e.source!)[side].add(e.sourceHandle!.slice(side.length + 1));
  }
  // Convert sets → ordered arrays per (node, side).
  const orderedSlotsByNode = new Map<string, { top: string[]; bottom: string[] }>();
  for (const [nodeId, slots] of slotsByNode) {
    orderedSlotsByNode.set(nodeId, {
      top:    [...slots.top].sort((a, b) => slotRank(a) - slotRank(b)),
      bottom: [...slots.bottom].sort((a, b) => slotRank(a) - slotRank(b)),
    });
  }
  // Rewrite handle IDs to type-specific names so the dynamically-rendered
  // handle (at its computed % position) matches.
  for (const e of rfEdges) {
    const t = typeForEdge(e);
    if (!t) continue;
    const srcSide = sideOfHandle(e.sourceHandle);
    const tgtSide = sideOfHandle(e.targetHandle);
    if (srcSide) e.sourceHandle = `${srcSide}-${t}-s`;
    if (tgtSide) e.targetHandle = `${tgtSide}-${t}-t`;
  }
  // Stash slot arrays on each rfNode's data so the node component knows how
  // many handles to render and at what positions.
  for (const rfNode of rfNodes) {
    const slots = orderedSlotsByNode.get(rfNode.id);
    rfNode.data = {
      ...rfNode.data,
      topSlots: slots?.top ?? [],
      bottomSlots: slots?.bottom ?? [],
    };
  }


  // Outline rings (current/root) are rendered as overlays INSIDE the node
  // (see FranchiseFlowNode) so they paint above the card + posters. They're
  // no longer edges.

  // Restrict visibleEdges to the same set as rfEdges (both endpoints positioned)
  // so the light hover memos only consider neighbors that are actually rendered.
  const positionedVisibleEdges = visibleEdges.filter(
    (e) => positions.has(e.from) && positions.has(e.to),
  );

  // Format filter: render-stage hide. Drop hidden-format cards now that
  // everything is laid out (so the nodes they anchored kept their positions).
  // Remove the real card, its ghost copies (via ghostOriginByGhostId), and
  // EVERY edge touching a removed node: a dangling edge whose endpoint node
  // is gone re-triggers React Flow position lookups and causes the hover
  // flash this file is careful to avoid. The current node is never hidden
  // even if its own format is filtered.
  const formatHiddenIds = new Set<string>();
  for (const n of graph.nodes) {
    if (n.anilistId !== currentId && hiddenFormats.has(formatFor(n.format))) {
      formatHiddenIds.add(String(n.anilistId));
    }
  }
  const allRfNodes = rfNodes;
  const allRfEdges = [...frameBgEdges, ...sideStoryFrameEdges, ...rfEdges, ...frameAnchorEdges, ...frameToggleEdges];
  let outNodes = allRfNodes;
  let outEdges = allRfEdges;
  let outVisibleEdges = positionedVisibleEdges;
  if (formatHiddenIds.size > 0) {
    outNodes = allRfNodes.filter((rn) => {
      if (formatHiddenIds.has(rn.id)) return false; // real hidden-format card
      const origin = ghostOriginByGhostId.get(rn.id);
      return origin == null || !formatHiddenIds.has(String(origin)); // ghost of a hidden node
    });
    let keptIds = new Set(outNodes.map((n) => n.id));
    outEdges = allRfEdges.filter((e) => keptIds.has(e.source) && keptIds.has(e.target));
    // A ghost copy whose only edge pointed AT a now-hidden node would otherwise
    // float as a disconnected translucent card (origin visible, target hidden).
    // Drop any ghost left without a surviving edge, then re-prune edges.
    const edgeEndpoints = new Set<string>();
    for (const e of outEdges) { edgeEndpoints.add(e.source); edgeEndpoints.add(e.target); }
    outNodes = outNodes.filter((rn) => !ghostOriginByGhostId.has(rn.id) || edgeEndpoints.has(rn.id));
    keptIds = new Set(outNodes.map((n) => n.id));
    outEdges = allRfEdges.filter((e) => keptIds.has(e.source) && keptIds.has(e.target));
    outVisibleEdges = positionedVisibleEdges.filter(
      (e) => !formatHiddenIds.has(String(e.from)) && !formatHiddenIds.has(String(e.to)),
    );
  }

  return {
    nodes: outNodes,
    // Paint order (SVG, back to front): chain + side-story backgrounds →
    // real edges → frame-collapse edges → chain toggles. (Outline rings live
    // in the node DOM now.)
    edges: outEdges,
    visibleEdges: outVisibleEdges,
    ghostOriginByGhostId,
    ghostNeighborsByOrigin,
    collapsedNeighbors,
  };
}

// ─── Custom node component ────────────────────────────────────────────────────

// Handles are extracted so React.memo can keep them stable across hover-only
// re-renders. FranchiseFlowNode re-renders on every HoverContext change
// because it reads the context; without this memo, the dynamic <Handle>
// elements would be recreated each hover and React Flow would re-measure
// them → visible flash on every arrow.
type FlowSide = 'top' | 'bottom';
// Each entry is a `${type}-${dir}` slot id (dir 's'=output, 't'=input). Input
// and output each get their own slot so they spread instead of overlapping.
const EMPTY_SLOTS: ReadonlyArray<string> = [];
const NodeHandles = memo(function NodeHandles({
  topSlots,
  bottomSlots,
  outset = 0,
}: {
  topSlots: ReadonlyArray<string>;
  bottomSlots: ReadonlyArray<string>;
  /** Push top/bottom handles this many px OUTSIDE the card so edges meet
   *  beyond the current/root ring instead of tucking under it. */
  outset?: number;
}) {
  const renderSide = (side: FlowSide, slots: ReadonlyArray<string>) => {
    const pos = side === 'top' ? Position.Top : Position.Bottom;
    const edgeStyle = side === 'top' ? { top: -outset } : { bottom: -outset };
    return slots.map((slotId, i, arr) => {
      const x = ((i + 1) / (arr.length + 1)) * 100;
      const isSource = slotId.endsWith('-s');
      return (
        <Handle
          key={`${side}-${slotId}`}
          id={`${side}-${slotId}`}
          type={isSource ? 'source' : 'target'}
          position={pos}
          style={{ left: `${x}%`, ...edgeStyle }}
        />
      );
    });
  };
  return (
    <>
      {renderSide('top', topSlots)}
      {renderSide('bottom', bottomSlots)}
      <Handle id="right-s" type="source" position={Position.Right}  />
      <Handle id="right-t" type="target" position={Position.Right}  />
      <Handle id="left-s"  type="source" position={Position.Left}   />
      <Handle id="left-t"  type="target" position={Position.Left}   />
      {/* Back-compat handles at top/bottom center for outline + chain-frame edges. */}
      <Handle id="top-s"    type="source" position={Position.Top}    style={{ visibility: 'hidden' }} />
      <Handle id="top-t"    type="target" position={Position.Top}    style={{ visibility: 'hidden' }} />
      <Handle id="bottom-s" type="source" position={Position.Bottom} style={{ visibility: 'hidden' }} />
      <Handle id="bottom-t" type="target" position={Position.Bottom} style={{ visibility: 'hidden' }} />
    </>
  );
});

function FranchiseFlowNode({ id, data }: NodeProps<RFNode<FranchiseNodeFlowData>>) {
  const { node, title, isCurrent, isRoot, isStart, isGhost, ownedId, relLabel, statusMarker, anilistIcon, onOpenInApp, onOpenExternal } = data;
  const { hoveredId, highlightSet, highlightGhostIds, spineSet, spineOrder, visibleEdges, nodeById } = useContext(HoverContext);
  // Compute dim state from context - never from node.data - so this component
  // re-renders individually via context, not via React Flow's node-store updates.
  // Ghost nodes use a SEPARATE highlight set keyed by ghost rfNode id, so a
  // ghost only lights up when its own origin or target is hovered (not when
  // its origin happens to be a secondary highlight).
  const dimmed = isGhost
    ? (highlightGhostIds != null && !highlightGhostIds.has(id))
    : (highlightSet != null && !highlightSet.has(node.anilistId));
  // Overlay the relation label with the hover-relative one when this node is hovered
  // or is a direct neighbour of the hovered node.
  let displayLabel = relLabel;
  if (hoveredId != null && hoveredId !== node.anilistId) {
    const hoverLabel = relationLabelRelativeTo(hoveredId, node.anilistId, spineSet, spineOrder, visibleEdges, nodeById);
    if (hoverLabel != null) displayLabel = hoverLabel;
  } else if (hoveredId === node.anilistId) {
    // Hovering counts as Viewing - Root may co-occur; Start does not.
    displayLabel = isRoot ? 'Viewing · Root' : 'Viewing';
  } else if (isRoot && displayLabel == null) {
    // Root has no parent-relation by definition - surface "Root" so the card isn't
    // visually empty above the title.
    displayLabel = 'Root';
  } else if (isStart && displayLabel == null) {
    displayLabel = 'Start';
  }
  const owned = ownedId != null;
  const isManga = node.type === 'MANGA';

  const variantClass = owned || isCurrent ? 'franchise-node--internal' : 'franchise-node--external';
  const className = `franchise-node ${variantClass}${isGhost ? ' franchise-node--ghost' : ''}`;
  const dataAttrs = {
    'data-format': node.format ?? '',
    'data-current': isCurrent ? 'true' : undefined,
    'data-root': isRoot ? 'true' : undefined,
    'data-dimmed': dimmed ? 'true' : undefined,
    'data-ghost': isGhost ? 'true' : undefined,
  };

  const handleClick = () => {
    if (isCurrent || isGhost) return;
    if (ownedId) onOpenInApp(ownedId);
    else onOpenExternal(node);
  };

  // Right-click menu: lets you open on AniList even for owned shows
  // (where left-click opens the in-app series page). Ghost duplicates are
  // visual references only - they get no menu.
  const [menuPos, setMenuPos] = useState<{ x: number; y: number } | null>(null);
  const closeMenu = useCallback(() => setMenuPos(null), []);
  const handleContextMenu = (e: React.MouseEvent) => {
    if (isGhost) return;
    if (!ownedId && !node.siteUrl) return;
    e.preventDefault();
    e.stopPropagation();
    setMenuPos({ x: e.clientX, y: e.clientY });
  };

  useEffect(() => {
    if (menuPos == null) return;
    const onDown = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      if (target?.closest('.franchise-context-menu')) return;
      closeMenu();
    };
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') closeMenu(); };
    document.addEventListener('mousedown', onDown);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDown);
      document.removeEventListener('keydown', onKey);
    };
  }, [menuPos, closeMenu]);

  // Outline rings for current / root, rendered as the node's own box-shadow
  // so they paint ABOVE the card + posters (and above edges, since nodes sit
  // over the edge layer) and are NOT clipped by the node's overflow:hidden.
  // Each ring = a gap shadow + a colored ring shadow. The gap must be the
  // OPAQUE well color (--bg-deep): the legacy --bg-card is translucent glass
  // now and would let the outer ring bleed through the gap. Concentric when
  // both current+root: teal (viewing) inside, amber (root) outside. The
  // current node also gets the whisper accent glow, and the node's CSS
  // glass highlight is re-appended because this inline shadow replaces it.
  const TEAL = 'var(--accent-a)';
  const AMBER = 'var(--accent-amber)';
  const GAP = 'var(--bg-deep)';
  const ringSpecs: Array<{ color: string; offset: number }> = isGhost
    ? []
    : isCurrent && isRoot
      ? [{ color: TEAL, offset: 2 }, { color: AMBER, offset: 7 }]
      : isCurrent
        ? [{ color: TEAL, offset: 3 }]
        : isRoot
          ? [{ color: AMBER, offset: 3 }]
          : [];
  const ringLayers = ringSpecs.map((r) => `0 0 0 ${r.offset}px ${GAP}, 0 0 0 ${r.offset + 2}px ${r.color}`);
  if (ringLayers.length && isCurrent) ringLayers.push('var(--glow-accent)');
  if (ringLayers.length) ringLayers.push('var(--glass-highlight)');
  const ringBoxShadow = ringLayers.length ? ringLayers.join(', ') : undefined;
  // Push handles out past the OUTERMOST ring (offset + 2px stroke) so edges
  // meet outside the ring rather than under it.
  const ringOutset = ringSpecs.length ? Math.max(...ringSpecs.map((r) => r.offset + 2)) : 0;

  const inner = (
    <>
      <NodeHandles topSlots={data.topSlots ?? EMPTY_SLOTS} bottomSlots={data.bottomSlots ?? EMPTY_SLOTS} outset={ringOutset} />
      <div className="relation-card-poster">
        {node.poster ? (
          <img src={node.poster} alt={title} loading="lazy" decoding="async" />
        ) : (
          <div className="relation-card-poster-empty">
            {isManga ? <Film size={28} /> : <Tv size={28} />}
          </div>
        )}
      </div>
      <div className="relation-card-body">
        <div className="franchise-node__relation-row">
          {displayLabel && <span className="relation-card-type">{displayLabel}</span>}
          {owned
            ? <Library size={14} className="franchise-node__owned-icon" aria-label="In library" />
            : <span className="franchise-node__anilist-icon" aria-label="On AniList">{anilistIcon}</span>
          }
        </div>
        <Tooltip label={title}>
          <div className="relation-card-title">{title}</div>
        </Tooltip>
        <div className="relation-card-meta">
          {node.status === 'NOT_YET_RELEASED' && (
            <span className="franchise-node__status-tag">Not yet released</span>
          )}
          {node.format && (
            <span className="relation-card-format" data-format={node.format}>{node.format}</span>
          )}
          {(node.seasonYear ?? node.startYear) && <span>{node.seasonYear ?? node.startYear}</span>}
        </div>
      </div>
      {statusMarker}
      {/* Ghost dashed ring as an overlay so it paints ABOVE the poster (the
          old CSS `outline` sat behind it). inset:0 stays inside the box so
          overflow:hidden doesn't clip it. */}
      {isGhost && <div className="franchise-node__ghost-ring" />}
    </>
  );

  const menu = menuPos && createPortal(
    <div
      className="franchise-context-menu"
      data-liquid-glass=""
      data-lg-bezel="10"
      style={{ left: menuPos.x, top: menuPos.y }}
      role="menu"
    >
      {ownedId && (
        <button
          type="button"
          className="franchise-context-menu__item"
          role="menuitem"
          onClick={() => { onOpenInApp(ownedId); closeMenu(); }}
        >
          Open in AniBeam
        </button>
      )}
      {node.siteUrl && (
        <button
          type="button"
          className="franchise-context-menu__item"
          role="menuitem"
          onClick={() => { onOpenExternal(node); closeMenu(); }}
        >
          Open on AniList
        </button>
      )}
    </div>,
    document.body,
  );

  if (isGhost) {
    return (
      <div
        className={className}
        {...dataAttrs}
        aria-hidden="true"
        style={{ transformOrigin: data.ghostScaleOrigin ?? '50% 100%' }}
      >
        {inner}
      </div>
    );
  }
  return isCurrent ? (
    <>
      <div className={className} {...dataAttrs} style={{ boxShadow: ringBoxShadow }} onContextMenu={handleContextMenu}>
        {inner}
      </div>
      {menu}
    </>
  ) : (
    <>
      <button type="button" className={className} {...dataAttrs} style={{ boxShadow: ringBoxShadow }} onClick={handleClick} onContextMenu={handleContextMenu}>
        {inner}
      </button>
      {menu}
    </>
  );
}

const nodeTypes = { franchise: FranchiseFlowNode };

// Edge that connects a source/parent to a whole FRAME (a group of member
// nodes), drawing to the frame's facing edge centre using the members' LIVE
// measured rects - so it lands on the frame border, not on one member, and
// never overshoots (no NODE_H estimate).
interface FrameLinkData extends Record<string, unknown> {
  memberIds: number[];
  className?: string;
  color?: string;
}
function FrameLinkEdge({ sourceX, sourceY, data, markerEnd }: EdgeProps) {
  const d = data as FrameLinkData | undefined;
  const memberKey = d ? d.memberIds.join(',') : '';
  // Return a stable STRING (not a fresh object) so React Flow's Object.is
  // equality doesn't re-render this edge on every store tick.
  const boundsKey = useStore((s) => {
    if (!d) return '';
    let left = Infinity, right = -Infinity, top = Infinity, bottom = -Infinity;
    for (const id of memberKey.split(',')) {
      const n = s.nodeLookup.get(id);
      if (!n) continue;
      const x = n.internals.positionAbsolute.x;
      const y = n.internals.positionAbsolute.y;
      const w = n.measured?.width ?? NODE_W;
      const h = n.measured?.height ?? NODE_H;
      left = Math.min(left, x); right = Math.max(right, x + w);
      top = Math.min(top, y); bottom = Math.max(bottom, y + h);
    }
    return Number.isFinite(left) ? `${left}|${right}|${top}|${bottom}` : '';
  });
  if (!d || !boundsKey) return null;
  const [left, right, top, bottom] = boundsKey.split('|').map(Number);
  const PAD = 16;
  const cx = (left + right) / 2;
  // Connect to whichever frame edge faces the source.
  const ty = sourceY <= top ? top - PAD : bottom + PAD;
  const [path] = getBezierPath({
    sourceX, sourceY,
    sourcePosition: sourceY <= ty ? Position.Bottom : Position.Top,
    targetX: cx, targetY: ty,
    targetPosition: sourceY <= ty ? Position.Top : Position.Bottom,
  });
  const color = d.color ?? 'var(--accent-blue)';
  return (
    <g className={`franchise-edge ${d.className ?? ''}`} pointerEvents="none">
      <path d={path} fill="none" stroke={color} strokeWidth={2} className="react-flow__edge-path" markerEnd={markerEnd} />
    </g>
  );
}

// ─── Chain frame edges ───────────────────────────────────────────────────────
// Two edges per chain. Both use leftmost.top-s → rightmost.bottom-t so the
// component receives the chain's ACTUAL top/bottom from React Flow.
//   - ChainBgEdge      → paints first (background, behind real arrows)
//   - ChainToggleEdge  → paints last  (toolbar, above real arrows)
interface ChainFrameEdgeData extends Record<string, unknown> {
  chainKey: string;
  settings: ChainSettings;
  onUpdate: (chainKey: string, patch: Partial<ChainSettings>) => void;
}
const FRAME_PAD = 16; // identical gap on all four sides
function chainFrameBounds(sourceX: number, sourceY: number, targetX: number, targetY: number) {
  return {
    left:   Math.min(sourceX, targetX) - NODE_W / 2 - FRAME_PAD,
    right:  Math.max(sourceX, targetX) + NODE_W / 2 + FRAME_PAD,
    top:    Math.min(sourceY, targetY) - FRAME_PAD,
    bottom: Math.max(sourceY, targetY) + FRAME_PAD,
  };
}
function ChainBgEdge({ sourceX, sourceY, targetX, targetY }: EdgeProps) {
  const { left, top, right, bottom } = chainFrameBounds(sourceX, sourceY, targetX, targetY);
  return (
    <g className="franchise-chain-frame-edge" pointerEvents="none">
      <rect
        x={left}
        y={top}
        width={right - left}
        height={bottom - top}
        rx={24}
        ry={24}
        className="franchise-chain-frame__bg"
      />
    </g>
  );
}
function ChainToggleEdge({ sourceX, sourceY, targetX, targetY, data }: EdgeProps) {
  const d = data as ChainFrameEdgeData | undefined;
  if (!d) return null;
  const { top, right } = chainFrameBounds(sourceX, sourceY, targetX, targetY);
  const TOOLBAR_W = 280;
  const TOOLBAR_H = 32;
  return (
    <g pointerEvents="none">
      <foreignObject
        x={right - TOOLBAR_W}
        y={top - TOOLBAR_H - 8}
        width={TOOLBAR_W}
        height={TOOLBAR_H}
        style={{ pointerEvents: 'auto', overflow: 'visible' }}
      >
        <div className="franchise-chain-frame__toolbar">
          <div className="franchise-chain-frame__toggle" role="group" aria-label="Order">
            <Tooltip label="Topological by SEQUEL">
              <button
                type="button"
                className={`franchise-chain-frame__toggle-opt${d.settings.order === 'chrono' ? ' is-active' : ''}`}
                aria-pressed={d.settings.order === 'chrono'}
                onClick={(e) => { e.stopPropagation(); d.onUpdate(d.chainKey, { order: 'chrono' }); }}
              >Chrono</button>
            </Tooltip>
            <Tooltip label="Sorted by release year">
              <button
                type="button"
                className={`franchise-chain-frame__toggle-opt${d.settings.order === 'release' ? ' is-active' : ''}`}
                aria-pressed={d.settings.order === 'release'}
                onClick={(e) => { e.stopPropagation(); d.onUpdate(d.chainKey, { order: 'release' }); }}
              >Release</button>
            </Tooltip>
          </div>
        </div>
      </foreignObject>
    </g>
  );
}

// ─── Side-story group frame ──────────────────────────────────────────────────
// When a single show has >3 side stories, they're boxed together under a
// "Side stories" label instead of floating as loose cards. Anchored leftmost
// .top-s → rightmost.bottom-t like the chain frame so it spans the group.
function SideStoryFrameEdge({ sourceX, sourceY, targetX, targetY }: EdgeProps) {
  const { left, top, right, bottom } = chainFrameBounds(sourceX, sourceY, targetX, targetY);
  const LABEL_RESERVE = 30;
  return (
    <g className="franchise-sidestory-frame-edge" pointerEvents="none">
      <rect
        x={left}
        y={top - LABEL_RESERVE}
        width={right - left}
        height={(bottom - top) + LABEL_RESERVE}
        rx={20}
        ry={20}
        className="franchise-sidestory-frame__bg"
      />
      <foreignObject x={left + 14} y={top - LABEL_RESERVE + 4} width={right - left - 28} height={24} style={{ overflow: 'visible' }}>
        <div className="franchise-sidestory-frame__label">Side stories</div>
      </foreignObject>
    </g>
  );
}

const edgeTypes = { chainBg: ChainBgEdge, chainToggle: ChainToggleEdge, sideStoryFrame: SideStoryFrameEdge, frameLink: FrameLinkEdge };

// Fullscreen persists across series navigation. Clicking a library node
// navigates to that series, which briefly unmounts the graph (while the new
// series' data loads) and would otherwise reset fullscreen. Holding it at
// module scope keeps fullscreen on through the remount.
let persistedFullscreen = false;

// ─── Inner canvas (requires ReactFlowProvider ancestor) ──────────────────────

function FranchiseGraphCanvas(props: FranchiseGraphViewProps) {
  const {
    graph,
    currentAnilistId,
    resolveOwnedId,
    pickTitle,
    onOpenInApp,
    onOpenExternal,
    statusMarkerFor,
    anilistIcon,
    hiddenCategories = new Set(),
    onToggleCategory,
    hiddenFormats = new Set(),
    onToggleFormat,
    filling = false,
    titleLang,
  } = props;

  const reactFlowInstance = useReactFlow();
  const [isFullscreen, setIsFullscreenState] = useState(persistedFullscreen);
  // Mirror every change to the module-scope flag so it survives remounts.
  const setIsFullscreen = useCallback((next: boolean | ((v: boolean) => boolean)) => {
    setIsFullscreenState((prev) => {
      const v = typeof next === 'function' ? next(prev) : next;
      persistedFullscreen = v;
      return v;
    });
  }, []);
  const [hoveredId, setHoveredId] = useState<number | null>(null);
  const hoverTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const HOVER_DELAY_MS = 280;

  const handleNodeMouseEnter = useCallback((_: React.MouseEvent, n: RFNode) => {
    if (hoverTimerRef.current) clearTimeout(hoverTimerRef.current);
    // Ghosts inherit their origin's identity for hover purposes - hovering a
    // ghost lights up the origin and everything the origin is connected to.
    const data = n.data as FranchiseNodeFlowData | undefined;
    const id = data?.isGhost ? data.node.anilistId : Number(n.id);
    if (!Number.isFinite(id)) return;
    hoverTimerRef.current = setTimeout(() => {
      hoverTimerRef.current = null;
      setHoveredId(id);
    }, HOVER_DELAY_MS);
  }, []);

  const handleNodeMouseLeave = useCallback(() => {
    if (hoverTimerRef.current) {
      clearTimeout(hoverTimerRef.current);
      hoverTimerRef.current = null;
    }
    setHoveredId(null);
  }, []);

  useEffect(() => () => {
    if (hoverTimerRef.current) clearTimeout(hoverTimerRef.current);
  }, []);

  // Per-chain order setting (chrono/release). Persisted to localStorage so a
  // chain's choice carries across sessions. Keyed by chainKey =
  // `chain-<smallest member id>` - stable across franchises since the key
  // includes the actual anilistId.
  const CHAIN_SETTINGS_KEY = 'franchise:chain-settings:v1';
  const [chainSettings, setChainSettingsState] = useState<Map<string, ChainSettings>>(() => {
    try {
      const raw = localStorage.getItem(CHAIN_SETTINGS_KEY);
      if (!raw) return new Map();
      const parsed = JSON.parse(raw) as Record<string, ChainSettings>;
      const m = new Map<string, ChainSettings>();
      for (const [k, v] of Object.entries(parsed)) {
        if (v && typeof v === 'object' && (v.order === 'chrono' || v.order === 'release')) {
          m.set(k, { order: v.order });
        }
      }
      return m;
    } catch {
      return new Map();
    }
  });
  useEffect(() => {
    try {
      const obj: Record<string, ChainSettings> = {};
      for (const [k, v] of chainSettings) obj[k] = v;
      localStorage.setItem(CHAIN_SETTINGS_KEY, JSON.stringify(obj));
    } catch { /* quota or disabled - fine */ }
  }, [chainSettings]);
  // Global "inline source" toggle - applies the source-alignment algorithm to
  // every chain at once instead of per-chain. Default ON.
  const [inlineEnabled, setInlineEnabled] = useState(true);
  const updateChainSettings = useCallback((chainKey: string, patch: Partial<ChainSettings>) => {
    setChainSettingsState((prev) => {
      const next = new Map(prev);
      const current = prev.get(chainKey) ?? DEFAULT_CHAIN_SETTINGS;
      next.set(chainKey, { ...current, ...patch });
      return next;
    });
  }, []);

  const handleZoomIn  = () => reactFlowInstance.zoomIn({ duration: 200 });
  const handleZoomOut = () => reactFlowInstance.zoomOut({ duration: 200 });
  const handleFitView = () => reactFlowInstance.fitView({ padding: 0.2, duration: 250 });

  useEffect(() => {
    if (!isFullscreen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setIsFullscreen(false);
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isFullscreen]);

  // Dev/testing affordance: Ctrl+Alt+G opens the franchise graph fullscreen so
  // it can be brought up programmatically (e.g. injected keystroke) without
  // hunting for the toggle or scrolling the series page.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.altKey && (e.key === 'g' || e.key === 'G')) {
        e.preventDefault();
        setIsFullscreen(true);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // The parent often passes inline functions for pickTitle / resolveOwnedId /
  // statusMarkerFor / onOpenInApp / onOpenExternal / anilistIcon. If we put
  // those directly in the heavy-memo deps, ANY parent re-render (e.g. background
  // metadata fetches) invalidates the layout cache → React Flow re-renders all
  // edges → CSS transitions restart → "blink on hover" looping symptom.
  // Stash them in a ref instead: the memo always reads the latest values but
  // doesn't invalidate on their identity change.
  const propsRef = useRef({ pickTitle, resolveOwnedId, statusMarkerFor, anilistIcon, onOpenInApp, onOpenExternal });
  propsRef.current = { pickTitle, resolveOwnedId, statusMarkerFor, anilistIcon, onOpenInApp, onOpenExternal };

  // Heavy memo: layout + enrichment. Does NOT depend on hoveredId or on the
  // unstable parent function refs (those are pulled from propsRef).
  const { baseRfNodes, baseRfEdges, visibleEdges, spineSet, spineOrder, nodeById, ghostOriginByGhostId, ghostNeighborsByOrigin, collapsedNeighbors } = useMemo(() => {
    const { pickTitle, resolveOwnedId, statusMarkerFor, anilistIcon, onOpenInApp, onOpenExternal } = propsRef.current;
    const rootId = findFranchiseRoot(graph, currentAnilistId);
    const layout = layoutGraph(graph, currentAnilistId, rootId, hiddenCategories, hiddenFormats, chainSettings, updateChainSettings, inlineEnabled);

    // Real franchise nodes only (exclude frame-anchor helper nodes).
    const realNodes = layout.nodes.filter((n) => n.type !== 'frameAnchor' && n.data.node);
    // Build node lookup map for label canonicalization.
    const nodeById = new Map<number, FranchiseNodeData>(
      realNodes.map((n) => [n.data.node.anilistId, n.data.node]),
    );

    // Compute spine order for the hover-relabeling memo (same visible graph).
    const { spineSet: sSet, order: sOrder } = spineOrderMap(
      { ...graph, nodes: realNodes.map((n) => n.data.node), edges: layout.visibleEdges },
      currentAnilistId,
    );

    // Enrich node data with display fields.
    const enrichedNodes = layout.nodes.map((rfNode) => {
      // Frame-anchor nodes carry no franchise data - pass through untouched.
      if (rfNode.type === 'frameAnchor' || !rfNode.data.node) return rfNode;
      const nodeData = rfNode.data.node;
      if (rfNode.data.isGhost) {
        return {
          ...rfNode,
          data: {
            ...rfNode.data,
            title: pickTitle(nodeData),
            ownedId: undefined,
            statusMarker: null,
            anilistIcon,
            onOpenInApp,
            onOpenExternal,
          },
        };
      }
      return {
        ...rfNode,
        data: {
          ...rfNode.data,
          title: pickTitle(nodeData),
          ownedId: resolveOwnedId(nodeData),
          statusMarker: statusMarkerFor(nodeData),
          anilistIcon,
          onOpenInApp,
          onOpenExternal,
        },
      };
    });

    return {
      baseRfNodes: enrichedNodes,
      baseRfEdges: layout.edges,
      visibleEdges: layout.visibleEdges,
      spineSet: sSet,
      spineOrder: sOrder,
      nodeById,
      ghostOriginByGhostId: layout.ghostOriginByGhostId,
      ghostNeighborsByOrigin: layout.ghostNeighborsByOrigin,
      collapsedNeighbors: layout.collapsedNeighbors,
    };
    // Intentionally NOT including the parent-passed callbacks/render props -
    // they're routed via propsRef so unstable refs don't blow the memo.
  }, [graph, currentAnilistId, hiddenCategories, hiddenFormats, chainSettings, updateChainSettings, inlineEnabled, titleLang]);

  // Light memo: compute the neighbour highlight set for the hover context.
  // Only the hover context value changes on hover - React Flow's node store
  // never receives updated node objects, eliminating the transform-restart flash.
  const highlightSet = useMemo<Set<number> | null>(() => {
    if (hoveredId == null) return null;
    const s = new Set<number>([hoveredId]);
    // Real edges: simple numeric endpoint check.
    for (const e of baseRfEdges) {
      const from = Number(e.source);
      const to = Number(e.target);
      if (Number.isFinite(from) && Number.isFinite(to)) {
        if (from === hoveredId) s.add(to);
        if (to === hoveredId) s.add(from);
        continue;
      }
      // Ghost edges: one endpoint is a synthetic ghostId, the other is the alt.
      const fromGhostOrigin = ghostOriginByGhostId.get(e.source);
      const toGhostOrigin   = ghostOriginByGhostId.get(e.target);
      if (fromGhostOrigin != null && Number.isFinite(to)) {
        // ghost (=fromGhostOrigin) ↔ alt (=to)
        if (fromGhostOrigin === hoveredId) s.add(to);
        if (to === hoveredId)              s.add(fromGhostOrigin);
      } else if (toGhostOrigin != null && Number.isFinite(from)) {
        if (toGhostOrigin === hoveredId) s.add(from);
        if (from === hoveredId)          s.add(toGhostOrigin);
      }
    }
    // Belt-and-suspenders: if the hovered node is a ghost origin, ensure every
    // alt reachable via any of its ghosts ends up in the set.
    const altsViaOriginGhosts = ghostNeighborsByOrigin.get(hoveredId);
    if (altsViaOriginGhosts) {
      for (const altId of altsViaOriginGhosts) s.add(altId);
    }
    // Collapsed frame links: hovering the parent/source lights the whole frame
    // (its per-member edges were replaced by one frameLink edge), and hovering
    // any framed member lights the parent.
    const viaCollapse = collapsedNeighbors.get(hoveredId);
    if (viaCollapse) {
      for (const id of viaCollapse) s.add(id);
    }
    return s;
  }, [hoveredId, baseRfEdges, ghostOriginByGhostId, ghostNeighborsByOrigin, collapsedNeighbors]);

  // A ghost is highlighted ONLY if its origin === hoveredId, or its target via
  // a ghost edge === hoveredId. Without this, a ghost would inherit highlight
  // whenever its origin (= ghost.data.node.anilistId) happened to be in
  // highlightSet as a secondary neighbour - e.g., hovering A would light up
  // the ghost of A's sequel C, which the user explicitly doesn't want.
  const highlightGhostIds = useMemo<Set<string> | null>(() => {
    if (hoveredId == null) return null;
    const s = new Set<string>();
    // Ghosts whose origin is exactly the hovered node.
    for (const [ghostId, originId] of ghostOriginByGhostId) {
      if (originId === hoveredId) s.add(ghostId);
    }
    // Ghosts directly connected to the hovered node via a ghost edge.
    for (const e of baseRfEdges) {
      if (ghostOriginByGhostId.has(e.source) && Number(e.target) === hoveredId) {
        s.add(e.source);
      }
      if (ghostOriginByGhostId.has(e.target) && Number(e.source) === hoveredId) {
        s.add(e.target);
      }
    }
    return s;
  }, [hoveredId, baseRfEdges, ghostOriginByGhostId]);

  // Stable context value - changes only when hoveredId / graph data change.
  // FranchiseFlowNode reads this to compute dim + hover-relative labels locally,
  // so React Flow's node store is never touched on hover.
  const hoverCtx = useMemo<HoverCtx>(() => ({
    hoveredId,
    highlightSet,
    highlightGhostIds,
    spineSet,
    spineOrder,
    visibleEdges,
    nodeById,
    ghostOriginByGhostId,
    ghostNeighborsByOrigin,
  }), [hoveredId, highlightSet, highlightGhostIds, spineSet, spineOrder, visibleEdges, nodeById, ghostOriginByGhostId, ghostNeighborsByOrigin]);

  // Light memo: apply dimmed class to edges without touching node layout.
  // Ghost edges have synthetic string IDs as source/target - they need to map
  // through ghostOriginByGhostId so hovering the source highlights the ghost's
  // connection too (and hovering the ghost = hovering the source already, via
  // handleNodeMouseEnter).
  const edges = useMemo(() => {
    if (hoveredId == null) {
      return baseRfEdges.map((e) => {
        const cls = e.className?.replace(/\s*franchise-edge--dimmed/g, '') ?? '';
        return cls === e.className ? e : { ...e, className: cls };
      });
    }
    const touchesHover = (endpointId: string): boolean => {
      const n = Number(endpointId);
      if (Number.isFinite(n) && n === hoveredId) return true;
      const origin = ghostOriginByGhostId.get(endpointId);
      return origin === hoveredId;
    };
    return baseRfEdges.map((e) => {
      const isDimmed = !touchesHover(e.source) && !touchesHover(e.target);
      const baseClass = (e.className ?? '').replace(/\s*franchise-edge--dimmed/g, '');
      const cls = isDimmed ? `${baseClass} franchise-edge--dimmed` : baseClass;
      return cls === e.className ? e : { ...e, className: cls };
    });
  }, [baseRfEdges, hoveredId, ghostOriginByGhostId]);

  // Center the current card ONCE per viewed series. The graph object changes
  // identity on every background franchise:store-updated re-fetch (the crawler
  // writes on each open), so centering on `graph` would repeatedly yank the
  // viewport back to the current card while the user is panning/zooming out.
  // Guard on the series id: center when the current node first appears for an
  // id, then never again until the user opens a different series.
  const centeredIdRef = useRef<number | null>(null);
  useEffect(() => {
    if (centeredIdRef.current === currentAnilistId) return;
    const cur = baseRfNodes.find((n) => n.data.isCurrent);
    if (!cur) return; // current node not laid out yet: retry on the next update
    centeredIdRef.current = currentAnilistId ?? null;
    reactFlowInstance.setCenter(
      cur.position.x + NODE_W / 2,
      cur.position.y + NODE_H / 2,
      { zoom: 1, duration: 300 },
    );
  }, [baseRfNodes, currentAnilistId, reactFlowInstance]);

  // Manual recenter (controls panel): same target node, zoom and duration as
  // the center-once effect above, just on demand.
  const handleRecenter = () => {
    const cur = baseRfNodes.find((n) => n.data.isCurrent);
    if (!cur) return;
    reactFlowInstance.setCenter(
      cur.position.x + NODE_W / 2,
      cur.position.y + NODE_H / 2,
      { zoom: 1, duration: 300 },
    );
  };

  const handleNodeClick = (_: React.MouseEvent, rfNode: RFNode<FranchiseNodeFlowData>) => {
    const { isCurrent, ownedId, node } = rfNode.data;
    if (isCurrent) return;
    if (ownedId) onOpenInApp(ownedId);
    else onOpenExternal(node);
  };

  // ─── Debug panel values ───────────────────────────────────────────────────────
  // Telemetry is opt-in: dev builds, or localStorage 'anibeam.graphDebug' = '1'.
  // Production users never see node/edge counts.
  const showDebug = import.meta.env.DEV
    || (typeof localStorage !== 'undefined' && localStorage.getItem('anibeam.graphDebug') === '1');
  const rootId = findFranchiseRoot(graph, currentAnilistId);
  const rootNode = rootId != null ? graph.nodes.find((n) => n.anilistId === rootId) : undefined;
  const rootTitle = rootNode?.titleRomaji ?? rootNode?.titleEnglish ?? undefined;

  const statusLabel = (() => {
    if (!graph) return 'Loading…';
    if (filling) return 'Refetching…';
    if (!graph.complete) return `Crawling… (${graph.deferred.length} deferred)`;
    return 'Ready';
  })();

  const totalEdges = graph.edges.length;
  const edgeFilterDiff = totalEdges !== visibleEdges.length;

  const containerClass = `franchise-graph${isFullscreen ? ' franchise-graph--fullscreen' : ''}`;
  const content = (
    <div className={containerClass}>
      {/* HoverContext.Provider wraps ReactFlow so that FranchiseFlowNode can read
          hoveredId + highlightSet directly. This means React Flow's node store is
          NEVER updated on hover - only the individual node components re-render via
          context subscription, eliminating the transform-restart flash. */}
      <HoverContext.Provider value={hoverCtx}>
        <ReactFlow
          nodes={baseRfNodes}
          edges={edges}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          onNodeClick={handleNodeClick as Parameters<typeof ReactFlow>[0]['onNodeClick']}
          onNodeMouseEnter={handleNodeMouseEnter as Parameters<typeof ReactFlow>[0]['onNodeMouseEnter']}
          onNodeMouseLeave={handleNodeMouseLeave}
          fitView
          fitViewOptions={{ padding: 0.2, maxZoom: 1 }}
          minZoom={0.05}
          maxZoom={2.5}
          proOptions={{ hideAttribution: true }}
          nodesDraggable={false}
          nodesConnectable={false}
          elementsSelectable={false}
          panOnDrag
          zoomOnScroll
          zoomOnPinch
        >
          <Background />
          {showDebug && (
            <Panel position="top-left" className="franchise-debug">
              <div>{statusLabel}</div>
              <div>nodes: {graph.nodes.length}</div>
              <div>edges: {totalEdges}{edgeFilterDiff ? ` (visible ${visibleEdges.length})` : ''}</div>
              <div>root: {rootTitle ?? 'none'}{rootId != null ? ` (${rootId})` : ''}</div>
            </Panel>
          )}
          <Panel position="top-center" className="franchise-filters-panel" data-liquid-glass="" data-lg-bezel="12">
            <FranchiseFilters hidden={hiddenCategories} onToggle={onToggleCategory} hiddenFormats={hiddenFormats} onToggleFormat={onToggleFormat} />
          </Panel>
          <Panel position="bottom-center" className="franchise-inline-panel">
            <Tooltip label="Align each entry under its source column">
              <button
                type="button"
                role="switch"
                aria-checked={inlineEnabled}
                className={`franchise-inline-toggle${inlineEnabled ? ' is-active' : ''}`}
                onClick={() => setInlineEnabled((v) => !v)}
              >
                <span className="franchise-inline-toggle__dot" aria-hidden="true" />
                <span>Inline source</span>
                <span className="franchise-inline-toggle__state">{inlineEnabled ? 'On' : 'Off'}</span>
              </button>
            </Tooltip>
          </Panel>
          <Panel position="bottom-left" className="franchise-controls" data-liquid-glass="" data-lg-bezel="10">
            <Tooltip label="Zoom in">
              <button type="button" onClick={handleZoomIn} aria-label="Zoom in"><ZoomIn size={14} /></button>
            </Tooltip>
            <Tooltip label="Zoom out">
              <button type="button" onClick={handleZoomOut} aria-label="Zoom out"><ZoomOut size={14} /></button>
            </Tooltip>
            <Tooltip label="Fit view">
              <button type="button" onClick={handleFitView} aria-label="Fit view"><Maximize2 size={14} /></button>
            </Tooltip>
            <Tooltip label="Center on current">
              <button type="button" onClick={handleRecenter} aria-label="Center on current"><LocateFixed size={14} /></button>
            </Tooltip>
          </Panel>
          <Panel position="top-right" className="franchise-fullscreen-toggle">
            <Tooltip label={isFullscreen ? 'Exit fullscreen' : 'Enter fullscreen'}>
              <button type="button" onClick={() => setIsFullscreen((v) => !v)} aria-label={isFullscreen ? 'Exit fullscreen' : 'Enter fullscreen'}>
                {isFullscreen ? <Minimize size={14} /> : <Maximize size={14} />}
              </button>
            </Tooltip>
          </Panel>
        </ReactFlow>
      </HoverContext.Provider>
    </div>
  );
  return isFullscreen && typeof document !== 'undefined'
    ? createPortal(content, document.body)
    : content;
}

// ─── Public export ────────────────────────────────────────────────────────────

export function FranchiseGraphView(props: FranchiseGraphViewProps) {
  return (
    <ReactFlowProvider>
      <FranchiseGraphCanvas {...props} />
    </ReactFlowProvider>
  );
}
