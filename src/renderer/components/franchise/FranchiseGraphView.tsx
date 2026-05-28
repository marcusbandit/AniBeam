import '@xyflow/react/dist/style.css';

import { useEffect, useMemo, useState } from 'react';
import type { ReactNode } from 'react';
import {
  ReactFlow,
  ReactFlowProvider,
  Background,
  Handle,
  Panel,
  Position,
  useReactFlow,
  type Node as RFNode,
  type Edge as RFEdge,
  type NodeProps,
} from '@xyflow/react';
import * as dagre from 'dagre';
import { Tv, Film, ZoomIn, ZoomOut, Maximize2, Maximize, Minimize } from 'lucide-react';

import type { FranchiseGraph, FranchiseNode as FranchiseNodeData } from '../../../shared/franchise';
import { relationLabel } from './laneAssignment';
import { categoryFor, type FranchiseCategory, FranchiseFilters } from './FranchiseFilters';
import { Pill } from '../primitives';

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
}

// ─── Constants ────────────────────────────────────────────────────────────────

const NODE_WIDTH = 180;
const NODE_HEIGHT = 240;

// ─── Layout ──────────────────────────────────────────────────────────────────

function layoutGraph(
  graph: FranchiseGraph,
  currentId: number,
  hiddenCategories: ReadonlySet<FranchiseCategory>,
): { nodes: RFNode<FranchiseNodeFlowData>[]; edges: RFEdge[] } {
  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({
    rankdir: 'LR',
    nodesep: 40,
    ranksep: 90,
    marginx: 24,
    marginy: 24,
  });

  // Build incoming-relation map to get the label per node
  const incoming = new Map<number, string>();
  for (const e of graph.edges) {
    if (!incoming.has(e.to)) incoming.set(e.to, e.relationType);
  }

  // Filter edges by hidden categories
  const visibleEdges = graph.edges.filter(
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

  const visibleNodes = graph.nodes.filter((n) => connectedNodeIds.has(n.anilistId));

  for (const node of visibleNodes) {
    g.setNode(String(node.anilistId), { width: NODE_WIDTH, height: NODE_HEIGHT });
  }

  for (const edge of visibleEdges) {
    // Only add edges where both endpoints are still visible
    if (connectedNodeIds.has(edge.from) && connectedNodeIds.has(edge.to)) {
      g.setEdge(String(edge.from), String(edge.to), { relationType: edge.relationType });
    }
  }

  dagre.layout(g);

  const rfNodes: RFNode<FranchiseNodeFlowData>[] = visibleNodes.map((node) => {
    const dagreNode = g.node(String(node.anilistId));
    const isCurrent = node.anilistId === currentId;
    const rt = incoming.get(node.anilistId);
    const relLabel = isCurrent ? 'Currently viewing' : (rt ? relationLabel(rt) : null);

    return {
      id: String(node.anilistId),
      type: 'franchise',
      position: {
        x: dagreNode.x - NODE_WIDTH / 2,
        y: dagreNode.y - NODE_HEIGHT / 2,
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
      },
    };
  });

  const rfEdges: RFEdge[] = visibleEdges
    .filter((edge) => connectedNodeIds.has(edge.from) && connectedNodeIds.has(edge.to))
    .map((edge) => ({
      id: `${edge.from}->${edge.to}:${edge.relationType}`,
      source: String(edge.from),
      target: String(edge.to),
      type: 'smoothstep',
      className: `franchise-edge franchise-edge--${edge.relationType.toLowerCase()}`,
      data: { relationType: edge.relationType },
    }));

  return { nodes: rfNodes, edges: rfEdges };
}

// ─── Custom node component ────────────────────────────────────────────────────

function FranchiseFlowNode({ data }: NodeProps<RFNode<FranchiseNodeFlowData>>) {
  const { node, title, isCurrent, ownedId, relLabel, statusMarker, anilistIcon, onOpenInApp, onOpenExternal } = data;
  const owned = ownedId != null;
  const isManga = node.type === 'MANGA';

  const variantClass = owned || isCurrent ? 'franchise-node--internal' : 'franchise-node--external';
  const className = `franchise-node ${variantClass}`;
  const dataAttrs = {
    'data-format': node.format ?? '',
    'data-current': isCurrent ? 'true' : undefined,
  };

  const handleClick = () => {
    if (isCurrent) return;
    if (ownedId) onOpenInApp(ownedId);
    else onOpenExternal(node);
  };

  const inner = (
    <>
      <Handle type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} />
      <div className="relation-card-poster">
        {node.poster ? (
          <img src={node.poster} alt={title} loading="lazy" decoding="async" />
        ) : (
          <div className="relation-card-poster-empty">
            {isManga ? <Film size={28} /> : <Tv size={28} />}
          </div>
        )}
        <span aria-hidden="true">
          {isCurrent ? (
            <Pill tone="muted">You are here</Pill>
          ) : owned ? (
            <Pill tone="teal">In library</Pill>
          ) : (
            <Pill tone="accent">{anilistIcon} AniList</Pill>
          )}
        </span>
      </div>
      <div className="relation-card-body">
        {relLabel && <div className="relation-card-type">{relLabel}</div>}
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
  } = props;

  const reactFlowInstance = useReactFlow();
  const [isFullscreen, setIsFullscreen] = useState(false);

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

  const { nodes, edges } = useMemo(() => {
    const layout = layoutGraph(graph, currentAnilistId, hiddenCategories);

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

    return { nodes: enrichedNodes, edges: layout.edges };
  }, [graph, currentAnilistId, hiddenCategories, resolveOwnedId, pickTitle, onOpenInApp, onOpenExternal, statusMarkerFor, anilistIcon]);

  useEffect(() => {
    const cur = nodes.find((n) => n.data.isCurrent);
    if (!cur) return;
    reactFlowInstance.setCenter(
      cur.position.x + NODE_WIDTH / 2,
      cur.position.y + NODE_HEIGHT / 2,
      { zoom: 1, duration: 300 },
    );
  }, [graph, currentAnilistId]); // intentionally not depending on `nodes` (referentially stable via useMemo on same deps)

  const handleNodeClick = (_: React.MouseEvent, rfNode: RFNode<FranchiseNodeFlowData>) => {
    const { isCurrent, ownedId, node } = rfNode.data;
    if (isCurrent) return;
    if (ownedId) onOpenInApp(ownedId);
    else onOpenExternal(node);
  };

  return (
    <div className={`franchise-graph${isFullscreen ? ' franchise-graph--fullscreen' : ''}`}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodeClick={handleNodeClick as Parameters<typeof ReactFlow>[0]['onNodeClick']}
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
          <FranchiseFilters hidden={hiddenCategories} onToggle={onToggleCategory} />
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
}

// ─── Public export ────────────────────────────────────────────────────────────

export function FranchiseGraphView(props: FranchiseGraphViewProps) {
  return (
    <ReactFlowProvider>
      <FranchiseGraphCanvas {...props} />
    </ReactFlowProvider>
  );
}
