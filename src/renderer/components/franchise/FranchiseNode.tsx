import { Tv, Film } from 'lucide-react';
import type { ReactNode } from 'react';
import type { FranchiseNode as FranchiseNodeData } from '../../../shared/franchise';
import { Card, Pill } from '../primitives';

export interface FranchiseNodeProps {
  node: FranchiseNodeData;
  /** Display title (already resolved through the user's title-language pref). */
  title: string;
  /** Relation label shown above the title (e.g. "Sequel", "Source"). */
  relationLabel: string | null;
  /** True for the series whose page we're on — gets the highlight ring. */
  isCurrent: boolean;
  /** seriesId if the user owns this entry, else undefined. */
  ownedId?: string;
  /** Optional tracker-list status marker node. */
  statusMarker?: ReactNode;
  onOpenInApp: (seriesId: string) => void;
  onOpenExternal: (node: FranchiseNodeData) => void;
  /** AniList brand icon for the external pill. */
  anilistIcon: ReactNode;
}

export function FranchiseNode(props: FranchiseNodeProps) {
  const { node, title, relationLabel, isCurrent, ownedId, statusMarker } = props;
  const owned = ownedId != null;
  const isManga = node.type === 'MANGA';

  const handleClick = () => {
    if (isCurrent) return;
    if (owned) props.onOpenInApp(ownedId!);
    else props.onOpenExternal(node);
  };

  const tooltip = isCurrent
    ? undefined
    : owned ? `Open ${title} in your library` : `Open ${title} on AniList`;

  return (
    <Card
      variant={owned || isCurrent ? 'internal' : 'external'}
      noLift={isCurrent}
      onClick={isCurrent ? undefined : handleClick}
      tooltip={tooltip}
      aria-current={isCurrent ? 'page' : undefined}
      data-format={node.format ?? ''}
      className="franchise-node"
      data-current={isCurrent ? 'true' : undefined}
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
            <Pill tone="accent">{props.anilistIcon} AniList</Pill>
          )}
        </span>
      </div>
      <div className="relation-card-body">
        {relationLabel && <div className="relation-card-type">{relationLabel}</div>}
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
  );
}
