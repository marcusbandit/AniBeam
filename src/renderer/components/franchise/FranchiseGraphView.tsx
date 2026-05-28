import '@xyflow/react/dist/style.css';

import { useEffect, useMemo } from 'react';
import type { ReactNode } from 'react';
import {
  ReactFlow,
  ReactFlowProvider,
  Background,
  Controls,
  Handle,
  Position,
  useReactFlow,
  type Node as RFNode,
  type Edge as RFEdge,
  type NodeProps,
} from '@xyflow/react';
import * as dagre from 'dagre';
import { Tv, Film } from 'lucide-react';

import type { FranchiseGraph, FranchiseNode as FranchiseNodeData } from '../../../shared/franchise';
import { relationLabel } from './laneAssignment';
import { Card, Pill } from '../primitives';

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

  for (const node of graph.nodes) {
    g.setNode(String(node.anilistId), { width: NODE_WIDTH, height: NODE_HEIGHT });
  }

  for (const edge of graph.edges) {
    g.setEdge(String(edge.from), String(edge.to), { relationType: edge.relationType });
  }

  dagre.layout(g);

  const rfNodes: RFNode<FranchiseNodeFlowData>[] = graph.nodes.map((node) => {
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

  const rfEdges: RFEdge[] = graph.edges.map((edge) => ({
    id: `${edge.from}->${edge.to}:${edge.relationType}`,
    source: String(edge.from),
    target: String(edge.to),
    type: 'smoothstep',
    className: 'franchise-edge',
    data: { relationType: edge.relationType },
  }));

  return { nodes: rfNodes, edges: rfEdges };
}

// ─── Custom node component ────────────────────────────────────────────────────

function FranchiseFlowNode({ data }: NodeProps<RFNode<FranchiseNodeFlowData>>) {
  const { node, title, isCurrent, ownedId, relLabel, statusMarker, anilistIcon } = data;
  const owned = ownedId != null;
  const isManga = node.type === 'MANGA';

  const tooltip = isCurrent
    ? undefined
    : owned
      ? `Open ${title} in your library`
      : `Open ${title} on AniList`;

  return (
    <div
      className="franchise-node"
      data-current={isCurrent ? 'true' : undefined}
    >
      <Handle type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} />
      <Card
        variant={owned || isCurrent ? 'internal' : 'external'}
        noLift={isCurrent}
        tooltip={tooltip}
        aria-current={isCurrent ? 'page' : undefined}
        data-format={node.format ?? ''}
        style={{ width: NODE_WIDTH, height: NODE_HEIGHT, cursor: isCurrent ? 'default' : 'pointer' }}
      >
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
      </Card>
    </div>
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
  } = props;

  const reactFlowInstance = useReactFlow();

  const { nodes, edges } = useMemo(() => {
    const layout = layoutGraph(graph, currentAnilistId);

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
  }, [graph, currentAnilistId, resolveOwnedId, pickTitle, onOpenInApp, onOpenExternal, statusMarkerFor, anilistIcon]);

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
    <div className="franchise-graph">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodeClick={handleNodeClick as Parameters<typeof ReactFlow>[0]['onNodeClick']}
        fitView
        fitViewOptions={{ padding: 0.2, maxZoom: 1 }}
        minZoom={0.2}
        maxZoom={1.5}
        proOptions={{ hideAttribution: true }}
        nodesDraggable={false}
        nodesConnectable={false}
        elementsSelectable={false}
        panOnDrag
        zoomOnScroll
        zoomOnPinch
      >
        <Background />
        <Controls showInteractive={false} />
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
