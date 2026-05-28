import '@xyflow/react/dist/style.css';

import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
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
  type Node as RFNode,
  type Edge as RFEdge,
  type NodeProps,
} from '@xyflow/react';
import { Tv, Film, ZoomIn, ZoomOut, Maximize2, Maximize, Minimize, Library } from 'lucide-react';

import type { FranchiseEdge, FranchiseGraph, FranchiseNode as FranchiseNodeData } from '../../../shared/franchise';
import { relationLabel } from './laneAssignment';
import { categoryFor, formatFor, type FranchiseCategory, type FranchiseFormat, FranchiseFilters } from './FranchiseFilters';
import { layoutFranchise, pickHandles, dedupeReciprocalEdges, spineOrderMap, relationLabelRelativeTo, findFranchiseRoot, canonicalRelation } from './franchiseLayout';

// ─── Types ───────────────────────────────────────────────────────────────────

export interface FranchiseGraphViewProps {
  graph: FranchiseGraph;
  currentAnilistId: number;
  resolveOwnedId: (node: FranchiseNodeData) => string | undefined;
  pickTitle: (n: { titleRomaji: string | null; titleEnglish: string | null }) => string;
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
// React Flow's node store — only the affected node components re-render, so
// React Flow never re-applies transforms or recalculates edge paths on hover.

interface HoverCtx {
  hoveredId: number | null;
  /** IDs of the hovered node + its direct neighbours (null = no hover). */
  highlightSet: Set<number> | null;
  /** Stable refs from the heavy memo, used for hover-relative labeling. */
  spineSet: Set<number>;
  spineOrder: Map<number, number>;
  visibleEdges: readonly FranchiseEdge[];
  nodeById: ReadonlyMap<number, FranchiseNodeData>;
}
const HoverContext = createContext<HoverCtx>({
  hoveredId: null,
  highlightSet: null,
  spineSet: new Set(),
  spineOrder: new Map(),
  visibleEdges: [],
  nodeById: new Map(),
});

// ─── Constants ────────────────────────────────────────────────────────────────

const NODE_W = 180; // matches the .franchise-node CSS width
const NODE_H = 420; // poster (180×1.5 = 270) + body ~150

// ─── Helpers ─────────────────────────────────────────────────────────────────

function arrowColorFor(relationType: string): string {
  const cat = categoryFor(relationType);
  switch (cat) {
    case 'spine':       return 'var(--accent-teal, #14b8a6)';
    case 'source':      return 'var(--accent-secondary, #818cf8)';
    case 'alternative': return 'var(--accent-amber, #f59e0b)';
    case 'side':        return 'var(--text-muted, #64748b)';
    case 'other':
    default:            return 'var(--border-hover, rgba(255,255,255,0.14))';
  }
}

// ─── Layout ──────────────────────────────────────────────────────────────────

function layoutGraph(
  graph: FranchiseGraph,
  currentId: number,
  rootId: number | null,
  hiddenCategories: ReadonlySet<FranchiseCategory>,
  hiddenFormats: ReadonlySet<FranchiseFormat>,
): { nodes: RFNode<FranchiseNodeFlowData>[]; edges: RFEdge[]; visibleEdges: FranchiseEdge[] } {
  // Dedupe reciprocal edges (SOURCE↔ADAPTATION, PARENT↔SIDE_STORY, PREQUEL↔SEQUEL)
  const graphNodeById = new Map<number, FranchiseNodeData>(graph.nodes.map((n) => [n.anilistId, n]));
  const dedupedEdges = dedupeReciprocalEdges(graph.edges, graphNodeById);

  // Apply format filter: the current node is always visible; other nodes are
  // hidden if their format category is in hiddenFormats.
  const visibleNodeIds = new Set<number>();
  for (const n of graph.nodes) {
    if (n.anilistId === currentId || !hiddenFormats.has(formatFor(n.format))) {
      visibleNodeIds.add(n.anilistId);
    }
  }
  const filteredNodes = graph.nodes.filter((n) => visibleNodeIds.has(n.anilistId));
  const filteredEdges = dedupedEdges.filter((e) => visibleNodeIds.has(e.from) && visibleNodeIds.has(e.to));

  // Filter edges by hidden categories
  const categoryFilteredEdges = filteredEdges.filter(
    (e) => !hiddenCategories.has(categoryFor(e.relationType)),
  );

  // Restrict to PREQUEL/SEQUEL only — every other relation type is hidden.
  const visibleEdges = categoryFilteredEdges.filter(
    (e) => e.relationType === 'SEQUEL' || e.relationType === 'PREQUEL',
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
  // visible structure — category-filtered nodes are dropped, not just hidden.
  const filteredGraph: FranchiseGraph = { ...graph, nodes: visibleNodes, edges: visibleEdges };
  const positions = layoutFranchise(filteredGraph, currentId);

  // Compute spine order for reference-relative labeling
  const { spineSet, order: spineOrder } = spineOrderMap(
    { ...graph, nodes: visibleNodes, edges: visibleEdges },
    currentId,
  );

  // Node lookup map for label canonicalization
  const layoutNodeById = new Map<number, FranchiseNodeData>(visibleNodes.map((n) => [n.anilistId, n]));

  const rfNodes: RFNode<FranchiseNodeFlowData>[] = visibleNodes
    .filter((node) => positions.has(node.anilistId))
    .map((node) => {
    const isCurrent = node.anilistId === currentId;
    const isRoot = rootId != null && node.anilistId === rootId;
    const p = positions.get(node.anilistId) ?? { x: 0, y: 0 };

    // Reference-relative label (spine topology + direct edge), falling back to
    // tree-parent BFS label for multi-hop nodes.
    const relativeLabel = isCurrent
      ? null
      : relationLabelRelativeTo(currentId, node.anilistId, spineSet, spineOrder, visibleEdges, layoutNodeById);
    const fallbackLabel = incoming.get(node.anilistId);
    const relLabel = isCurrent
      ? 'Currently viewing'
      : (relativeLabel ?? (fallbackLabel
          ? relationLabel(canonicalRelation(fallbackLabel, node))
          : null));

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
  // — React Flow's internal node-position lookups on those orphan edges trigger
  // cascading updates on every hover state change, causing the visible flash.
  const rfEdges: RFEdge[] = visibleEdges
    .filter((edge) => positions.has(edge.from) && positions.has(edge.to))
    .map((edge) => {
      const { sourceHandle, targetHandle } = pickHandles(
        positions.get(edge.from) ?? { x: 0, y: 0 },
        positions.get(edge.to)   ?? { x: 0, y: 0 },
      );
      return {
        id: `${edge.from}->${edge.to}:${edge.relationType}`,
        source: String(edge.from),
        target: String(edge.to),
        sourceHandle,
        targetHandle,
        type: 'default',
        className: `franchise-edge franchise-edge--${edge.relationType.toLowerCase()}`,
        data: { relationType: edge.relationType },
        markerEnd: {
          type: MarkerType.ArrowClosed,
          color: arrowColorFor(edge.relationType),
          width: 18,
          height: 18,
        },
      };
    });

  // Restrict visibleEdges to the same set as rfEdges (both endpoints positioned)
  // so the light hover memos only consider neighbors that are actually rendered.
  const positionedVisibleEdges = visibleEdges.filter(
    (e) => positions.has(e.from) && positions.has(e.to),
  );
  return { nodes: rfNodes, edges: rfEdges, visibleEdges: positionedVisibleEdges };
}

// ─── Custom node component ────────────────────────────────────────────────────

function FranchiseFlowNode({ data }: NodeProps<RFNode<FranchiseNodeFlowData>>) {
  const { node, title, isCurrent, isRoot, ownedId, relLabel, statusMarker, anilistIcon, onOpenInApp, onOpenExternal } = data;
  const { hoveredId, highlightSet, spineSet, spineOrder, visibleEdges, nodeById } = useContext(HoverContext);
  // Compute dim state from context — never from node.data — so this component
  // re-renders individually via context, not via React Flow's node-store updates.
  const dimmed = highlightSet != null && !highlightSet.has(node.anilistId);
  // Overlay the relation label with the hover-relative one when this node is hovered
  // or is a direct neighbour of the hovered node.
  let displayLabel = relLabel;
  if (hoveredId != null && hoveredId !== node.anilistId) {
    const hoverLabel = relationLabelRelativeTo(hoveredId, node.anilistId, spineSet, spineOrder, visibleEdges, nodeById);
    if (hoverLabel != null) displayLabel = hoverLabel;
  } else if (hoveredId === node.anilistId) {
    displayLabel = 'Viewing';
  }
  const owned = ownedId != null;
  const isManga = node.type === 'MANGA';

  const variantClass = owned || isCurrent ? 'franchise-node--internal' : 'franchise-node--external';
  const className = `franchise-node ${variantClass}`;
  const dataAttrs = {
    'data-format': node.format ?? '',
    'data-current': isCurrent ? 'true' : undefined,
    'data-root': isRoot ? 'true' : undefined,
    'data-dimmed': dimmed ? 'true' : undefined,
  };

  const handleClick = () => {
    if (isCurrent) return;
    if (ownedId) onOpenInApp(ownedId);
    else onOpenExternal(node);
  };

  const inner = (
    <>
      <Handle id="top-s"    type="source" position={Position.Top}    />
      <Handle id="top-t"    type="target" position={Position.Top}    />
      <Handle id="right-s"  type="source" position={Position.Right}  />
      <Handle id="right-t"  type="target" position={Position.Right}  />
      <Handle id="bottom-s" type="source" position={Position.Bottom} />
      <Handle id="bottom-t" type="target" position={Position.Bottom} />
      <Handle id="left-s"   type="source" position={Position.Left}   />
      <Handle id="left-t"   type="target" position={Position.Left}   />
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
          {!isCurrent && (owned
            ? <Library size={14} className="franchise-node__owned-icon" aria-label="In library" />
            : <span className="franchise-node__anilist-icon" aria-label="On AniList">{anilistIcon}</span>
          )}
        </div>
        <div className="relation-card-title">{title}</div>
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
    </>
  );

  return isCurrent ? (
    <div className={className} {...dataAttrs}>
      {inner}
    </div>
  ) : (
    <button type="button" className={className} {...dataAttrs} onClick={handleClick}>
      {inner}
    </button>
  );
}

const nodeTypes = { franchise: FranchiseFlowNode };

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
  } = props;

  const reactFlowInstance = useReactFlow();
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [hoveredId, setHoveredId] = useState<number | null>(null);
  const hoverTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const HOVER_DELAY_MS = 280;

  const handleNodeMouseEnter = useCallback((_: React.MouseEvent, n: RFNode) => {
    if (hoverTimerRef.current) clearTimeout(hoverTimerRef.current);
    const id = Number(n.id);
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

  // Heavy memo: layout + enrichment. Does NOT depend on hoveredId.
  const { baseRfNodes, baseRfEdges, visibleEdges, spineSet, spineOrder, nodeById } = useMemo(() => {
    const rootId = findFranchiseRoot(graph, currentAnilistId);
    const layout = layoutGraph(graph, currentAnilistId, rootId, hiddenCategories, hiddenFormats);

    // Build node lookup map for label canonicalization.
    const nodeById = new Map<number, FranchiseNodeData>(
      layout.nodes.map((n) => [n.data.node.anilistId, n.data.node]),
    );

    // Compute spine order for the hover-relabeling memo (same visible graph).
    const { spineSet: sSet, order: sOrder } = spineOrderMap(
      { ...graph, nodes: layout.nodes.map((n) => n.data.node), edges: layout.visibleEdges },
      currentAnilistId,
    );

    // Enrich node data with display fields
    const enrichedNodes = layout.nodes.map((rfNode) => {
      const nodeData = rfNode.data.node;
      const isCurrent = rfNode.data.isCurrent;
      return {
        ...rfNode,
        data: {
          ...rfNode.data,
          title: pickTitle(nodeData),
          ownedId: resolveOwnedId(nodeData),
          statusMarker: isCurrent ? null : statusMarkerFor(nodeData),
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
    };
  }, [graph, currentAnilistId, hiddenCategories, hiddenFormats, resolveOwnedId, pickTitle, onOpenInApp, onOpenExternal, statusMarkerFor, anilistIcon]);

  // Light memo: compute the neighbour highlight set for the hover context.
  // Only the hover context value changes on hover — React Flow's node store
  // never receives updated node objects, eliminating the transform-restart flash.
  const highlightSet = useMemo<Set<number> | null>(() => {
    if (hoveredId == null) return null;
    const s = new Set<number>([hoveredId]);
    for (const e of baseRfEdges) {
      const from = Number(e.source);
      const to = Number(e.target);
      if (from === hoveredId) s.add(to);
      if (to === hoveredId) s.add(from);
    }
    return s;
  }, [hoveredId, baseRfEdges]);

  // Stable context value — changes only when hoveredId / graph data change.
  // FranchiseFlowNode reads this to compute dim + hover-relative labels locally,
  // so React Flow's node store is never touched on hover.
  const hoverCtx = useMemo<HoverCtx>(() => ({
    hoveredId,
    highlightSet,
    spineSet,
    spineOrder,
    visibleEdges,
    nodeById,
  }), [hoveredId, highlightSet, spineSet, spineOrder, visibleEdges, nodeById]);

  // Light memo: apply dimmed class to edges without touching node layout.
  const edges = useMemo(() => {
    if (hoveredId == null) {
      return baseRfEdges.map((e) => {
        const cls = e.className?.replace(/\s*franchise-edge--dimmed/g, '') ?? '';
        return cls === e.className ? e : { ...e, className: cls };
      });
    }
    return baseRfEdges.map((e) => {
      const from = Number(e.source);
      const to = Number(e.target);
      const isDimmed = from !== hoveredId && to !== hoveredId;
      const baseClass = (e.className ?? '').replace(/\s*franchise-edge--dimmed/g, '');
      const cls = isDimmed ? `${baseClass} franchise-edge--dimmed` : baseClass;
      return cls === e.className ? e : { ...e, className: cls };
    });
  }, [baseRfEdges, hoveredId]);

  useEffect(() => {
    const cur = baseRfNodes.find((n) => n.data.isCurrent);
    if (!cur) return;
    reactFlowInstance.setCenter(
      cur.position.x + NODE_W / 2,
      cur.position.y + NODE_H / 2,
      { zoom: 1, duration: 300 },
    );
  }, [graph, currentAnilistId]); // intentionally narrow deps — baseRfNodes is stable when graph/id are stable

  const handleNodeClick = (_: React.MouseEvent, rfNode: RFNode<FranchiseNodeFlowData>) => {
    const { isCurrent, ownedId, node } = rfNode.data;
    if (isCurrent) return;
    if (ownedId) onOpenInApp(ownedId);
    else onOpenExternal(node);
  };

  // ─── Debug panel values ───────────────────────────────────────────────────────
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
          NEVER updated on hover — only the individual node components re-render via
          context subscription, eliminating the transform-restart flash. */}
      <HoverContext.Provider value={hoverCtx}>
        <ReactFlow
          nodes={baseRfNodes}
          edges={edges}
          nodeTypes={nodeTypes}
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
          <Panel position="top-left" className="franchise-debug">
            <div>{statusLabel}</div>
            <div>nodes: {graph.nodes.length}</div>
            <div>edges: {totalEdges}{edgeFilterDiff ? ` (visible ${visibleEdges.length})` : ''}</div>
            <div>root: {rootTitle ?? '—'}{rootId != null ? ` (${rootId})` : ''}</div>
          </Panel>
          <Panel position="top-center" className="franchise-filters-panel">
            <FranchiseFilters hidden={hiddenCategories} onToggle={onToggleCategory} hiddenFormats={hiddenFormats} onToggleFormat={onToggleFormat} />
          </Panel>
          <Panel position="bottom-left" className="franchise-controls">
            <button type="button" onClick={handleZoomIn}  aria-label="Zoom in"  title="Zoom in"><ZoomIn size={14} /></button>
            <button type="button" onClick={handleZoomOut} aria-label="Zoom out" title="Zoom out"><ZoomOut size={14} /></button>
            <button type="button" onClick={handleFitView} aria-label="Fit view" title="Fit view"><Maximize2 size={14} /></button>
          </Panel>
          <Panel position="top-right" className="franchise-fullscreen-toggle">
            <button type="button" onClick={() => setIsFullscreen((v) => !v)} aria-label={isFullscreen ? 'Exit fullscreen' : 'Enter fullscreen'} title={isFullscreen ? 'Exit fullscreen' : 'Enter fullscreen'}>
              {isFullscreen ? <Minimize size={14} /> : <Maximize size={14} />}
            </button>
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
