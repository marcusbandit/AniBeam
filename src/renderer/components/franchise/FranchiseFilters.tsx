import type { ReactElement } from 'react';

export type FranchiseCategory = 'spine' | 'source' | 'side' | 'alternative' | 'other';

export interface FranchiseFiltersProps {
  hidden: ReadonlySet<FranchiseCategory>;
  onToggle: (cat: FranchiseCategory) => void;
}

/**
 * Map an AniList relationType string to one of the five display categories
 * used by the filter chips. Pure function — no side effects.
 */
export function categoryFor(relationType: string): FranchiseCategory {
  switch (relationType) {
    case 'PREQUEL':
    case 'SEQUEL':
      return 'spine';
    case 'SOURCE':
    case 'PARENT':
    case 'ADAPTATION':
      return 'source';
    case 'SIDE_STORY':
    case 'SPIN_OFF':
    case 'SUMMARY':
    case 'COMPILATION':
    case 'CONTAINS':
      return 'side';
    case 'ALTERNATIVE':
      return 'alternative';
    default:
      return 'other';
  }
}

const CHIP_LABELS: { cat: FranchiseCategory; label: string }[] = [
  { cat: 'spine',       label: 'Story chain' },
  { cat: 'source',      label: 'Sources & parents' },
  { cat: 'side',        label: 'Side stories & spin-offs' },
  { cat: 'alternative', label: 'Alternatives' },
  { cat: 'other',       label: 'Other' },
];

export function FranchiseFilters({ hidden, onToggle }: FranchiseFiltersProps): ReactElement {
  return (
    <div className="franchise-filters">
      {CHIP_LABELS.map(({ cat, label }) => (
        <button
          key={cat}
          type="button"
          className={`franchise-filter-chip${hidden.has(cat) ? ' franchise-filter-chip--off' : ''}`}
          data-category={cat}
          onClick={() => onToggle(cat)}
          aria-pressed={!hidden.has(cat)}
        >
          {label}
        </button>
      ))}
    </div>
  );
}
