import { useEffect, useMemo, useRef } from 'react';
import type { ReactNode } from 'react';
import type { FranchiseGraph, FranchiseNode as NodeData } from '../../../shared/franchise';
import { compareByYear, relationLabel, relationLane, type FranchiseLane } from './laneAssignment';
import { FranchiseNode } from './FranchiseNode';

export interface FranchiseMapProps {
  graph: FranchiseGraph;
  currentAnilistId: number;
  /** Resolve a node to an owned seriesId, if any. */
  resolveOwnedId: (node: NodeData) => string | undefined;
  pickTitle: (n: { titleRomaji: string | null; titleEnglish: string | null }) => string;
  onOpenInApp: (seriesId: string) => void;
  onOpenExternal: (node: NodeData) => void;
  statusMarkerFor: (node: NodeData) => ReactNode;
  anilistIcon: ReactNode;
}

interface Placed { node: NodeData; lane: FranchiseLane; relationType: string | null; }

/** Assign each node a lane + the relationType that connected it. The current
 *  node is forced onto the spine. A node's lane comes from the first edge that
 *  points *to* it (relationType is from the source's perspective, which is what
 *  the user reads: "this is a Side story of the thing it hangs off"). */
function placeNodes(graph: FranchiseGraph, currentId: number): Placed[] {
  const incoming = new Map<number, string>();
  for (const e of graph.edges) {
    if (!incoming.has(e.to)) incoming.set(e.to, e.relationType);
  }
  return graph.nodes.map((node) => {
    if (node.anilistId === currentId) return { node, lane: 'spine' as const, relationType: null };
    const rt = incoming.get(node.anilistId) ?? 'OTHER';
    const lane = relationLane(rt, node.type, node.format);
    return { node, lane, relationType: rt };
  });
}

export function FranchiseMap(props: FranchiseMapProps) {
  const { graph, currentAnilistId } = props;
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const currentRef = useRef<HTMLDivElement | null>(null);

  const lanes = useMemo(() => {
    const placed = placeNodes(graph, currentAnilistId).filter((p) => p.lane !== 'excluded');
    const byLane = (lane: FranchiseLane) =>
      placed.filter((p) => p.lane === lane).sort((a, b) => compareByYear(a.node, b.node));
    return {
      top: byLane('top'),
      spine: byLane('spine'),
      bottom: byLane('bottom'),
      branch: byLane('sidebranch'),
    };
  }, [graph, currentAnilistId]);

  // Center the current node on mount / when the graph changes.
  useEffect(() => {
    const c = currentRef.current;
    const s = scrollRef.current;
    if (!c || !s) return;
    // Rect-based so it's independent of which ancestor is the offsetParent
    // (Task 8 may make .franchise-map__inner positioned).
    const cRect = c.getBoundingClientRect();
    const sRect = s.getBoundingClientRect();
    s.scrollLeft += (cRect.left - sRect.left) - s.clientWidth / 2 + cRect.width / 2;
  }, [graph]);

  const renderTile = (p: Placed) => {
    const isCurrent = p.node.anilistId === currentAnilistId;
    return (
      <div
        key={p.node.anilistId}
        ref={isCurrent ? currentRef : undefined}
        className="franchise-cell"
      >
        <FranchiseNode
          node={p.node}
          title={props.pickTitle(p.node)}
          relationLabel={isCurrent ? 'Currently viewing' : (p.relationType ? relationLabel(p.relationType) : null)}
          isCurrent={isCurrent}
          ownedId={props.resolveOwnedId(p.node)}
          statusMarker={isCurrent ? null : props.statusMarkerFor(p.node)}
          onOpenInApp={props.onOpenInApp}
          onOpenExternal={props.onOpenExternal}
          anilistIcon={props.anilistIcon}
        />
      </div>
    );
  };

  const spineRow = [...lanes.spine, ...lanes.branch];

  return (
    <div className="franchise-map" ref={scrollRef}>
      <div className="franchise-map__inner">
        {lanes.top.length > 0 && (
          <div className="franchise-lane franchise-lane--top">{lanes.top.map(renderTile)}</div>
        )}
        <div className="franchise-lane franchise-lane--spine">
          {spineRow.length > 1 && <div className="franchise-rail" aria-hidden="true" />}
          {spineRow.map(renderTile)}
        </div>
        {lanes.bottom.length > 0 && (
          <div className="franchise-lane franchise-lane--bottom">{lanes.bottom.map(renderTile)}</div>
        )}
      </div>
    </div>
  );
}
