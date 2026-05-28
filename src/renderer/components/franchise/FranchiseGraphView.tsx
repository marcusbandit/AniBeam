import '@xyflow/react/dist/style.css';

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
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

import type { FranchiseGraph, FranchiseNode as FranchiseNodeData } from '../../../shared/franchise';
import { relationLabel } from './laneAssignment';
import { categoryFor, formatFor, type FranchiseCategory, type FranchiseFormat, FranchiseFilters } from './FranchiseFilters';
import { layoutFranchise, pickHandles, dedupeReciprocalEdges } from './franchiseLayout';

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
}

interface FranchiseNodeFlowData extends Record<string, unknown> {
  node: FranchiseNodeData;
  title: string;
  isCurrent: boolean;
  ownedId: string | undefined;
  relLabel: string | null;
  statusMarker: ReactNode;
  anilistIcon: ReactNode;
  onOpenInApp: (seriesId: string) => void;
  onOpenExternal: (node: FranchiseNodeData) => void;
  dimmed: boolean;
}

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
  hiddenCategories: ReadonlySet<FranchiseCategory>,
  hiddenFormats: ReadonlySet<FranchiseFormat>,
): { nodes: RFNode<FranchiseNodeFlowData>[]; edges: RFEdge[] } {
  // Dedupe reciprocal edges (SOURCE↔ADAPTATION, PARENT↔SIDE_STORY, PREQUEL↔SEQUEL)
  const dedupedEdges = dedupeReciprocalEdges(graph.edges);

  // Build incoming-relation map to get the label per node
  const incoming = new Map<number, string>();
  for (const e of dedupedEdges) {
    if (!incoming.has(e.to)) incoming.set(e.to, e.relationType);
  }

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
  const visibleEdges = filteredEdges.filter(
    (e) => !hiddenCategories.has(categoryFor(e.relationType)),
  );

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

  const rfNodes: RFNode<FranchiseNodeFlowData>[] = visibleNodes
    .filter((node) => positions.has(node.anilistId))
    .map((node) => {
    const isCurrent = node.anilistId === currentId;
    const rt = incoming.get(node.anilistId);
    const relLabel = isCurrent ? 'Currently viewing' : (rt ? relationLabel(rt) : null);
    const p = positions.get(node.anilistId) ?? { x: 0, y: 0 };

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

  const rfEdges: RFEdge[] = visibleEdges
    .filter((edge) => connectedNodeIds.has(edge.from) && connectedNodeIds.has(edge.to))
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

  return { nodes: rfNodes, edges: rfEdges };
}

// ─── Custom node component ────────────────────────────────────────────────────

function FranchiseFlowNode({ data }: NodeProps<RFNode<FranchiseNodeFlowData>>) {
  const { node, title, isCurrent, ownedId, relLabel, statusMarker, anilistIcon, onOpenInApp, onOpenExternal, dimmed } = data;
  const owned = ownedId != null;
  const isManga = node.type === 'MANGA';

  const variantClass = owned || isCurrent ? 'franchise-node--internal' : 'franchise-node--external';
  const className = `franchise-node ${variantClass}`;
  const dataAttrs = {
    'data-format': node.format ?? '',
    'data-current': isCurrent ? 'true' : undefined,
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
          {relLabel && <span className="relation-card-type">{relLabel}</span>}
          {!isCurrent && (owned
            ? <Library size={14} className="franchise-node__owned-icon" aria-label="In library" />
            : <span className="franchise-node__anilist-icon" aria-label="On AniList">{anilistIcon}</span>
          )}
        </div>
        <div className="relation-card-title">{title}</div>
        <div className="relation-card-meta">
          {node.format && (
            <span className="relation-card-format" data-format={node.format}>{node.format}</span>
          )}
          {node.seasonYear && <span>{node.seasonYear}</span>}
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
  } = props;

  const reactFlowInstance = useReactFlow();
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [hoveredId, setHoveredId] = useState<number | null>(null);
  const hoverTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const HOVER_DELAY_MS = 500;

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
  const { baseRfNodes, baseRfEdges } = useMemo(() => {
    const layout = layoutGraph(graph, currentAnilistId, hiddenCategories, hiddenFormats);

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

    return { baseRfNodes: enrichedNodes, baseRfEdges: layout.edges };
  }, [graph, currentAnilistId, hiddenCategories, hiddenFormats, resolveOwnedId, pickTitle, onOpenInApp, onOpenExternal, statusMarkerFor, anilistIcon]);

  // Light memo: compute the neighbor highlight set from hovered node + base edges.
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

  // Light memo: apply dim flag to nodes without touching layout.
  const nodes = useMemo(() => {
    if (highlightSet == null) {
      return baseRfNodes.map((n) => (n.data.dimmed ? { ...n, data: { ...n.data, dimmed: false } } : n));
    }
    return baseRfNodes.map((n) => {
      const id = Number(n.id);
      const dimmed = !highlightSet.has(id);
      if (n.data.dimmed === dimmed) return n;
      return { ...n, data: { ...n.data, dimmed } };
    });
  }, [baseRfNodes, highlightSet]);

  // Light memo: apply dimmed class to edges without touching layout.
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
    const cur = nodes.find((n) => n.data.isCurrent);
    if (!cur) return;
    reactFlowInstance.setCenter(
      cur.position.x + NODE_W / 2,
      cur.position.y + NODE_H / 2,
      { zoom: 1, duration: 300 },
    );
  }, [graph, currentAnilistId]); // intentionally not depending on `nodes` (referentially stable via useMemo on same deps)

  const handleNodeClick = (_: React.MouseEvent, rfNode: RFNode<FranchiseNodeFlowData>) => {
    const { isCurrent, ownedId, node } = rfNode.data;
    if (isCurrent) return;
    if (ownedId) onOpenInApp(ownedId);
    else onOpenExternal(node);
  };

  const containerClass = `franchise-graph${isFullscreen ? ' franchise-graph--fullscreen' : ''}`;
  const content = (
    <div className={containerClass}>
      <ReactFlow
        nodes={nodes}
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
