import type { ReactElement } from 'react';

export type FranchiseCategory = 'spine' | 'source' | 'side' | 'alternative' | 'character' | 'other';

export type FranchiseFormat =
  | 'series' | 'movie' | 'shortform'
  | 'manga' | 'novel' | 'visualnovel' | 'music' | 'other';

export function formatFor(format: string | null | undefined): FranchiseFormat {
  switch (format) {
    case 'TV':
    case 'TV_SHORT':
    case 'ONA':           return 'series';
    case 'MOVIE':         return 'movie';
    case 'OVA':
    case 'SPECIAL':       return 'shortform';
    case 'MANGA':
    case 'ONE_SHOT':      return 'manga';
    case 'NOVEL':
    case 'LIGHT_NOVEL':   return 'novel';
    case 'VISUAL_NOVEL':  return 'visualnovel';
    case 'MUSIC':         return 'music';
    default:              return 'other';
  }
}

const FORMAT_LABELS: Record<FranchiseFormat, string> = {
  series:      'Series',
  movie:       'Movies',
  shortform:   'OVA / Specials',
  manga:       'Manga',
  novel:       'Novels',
  visualnovel: 'Visual novels',
  music:       'Music',
  other:       'Other',
};

export interface FranchiseFiltersProps {
  hidden: ReadonlySet<FranchiseCategory>;
  onToggle: (cat: FranchiseCategory) => void;
  hiddenFormats: ReadonlySet<FranchiseFormat>;
  onToggleFormat: (fmt: FranchiseFormat) => void;
}

/**
 * Map an AniList relationType string to one of the display categories
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
    case 'CHARACTER':
      return 'character';
    default:
      return 'other';
  }
}

export function FranchiseFilters(props: FranchiseFiltersProps): ReactElement {
  const { hidden, onToggle, hiddenFormats, onToggleFormat } = props;
  const CATEGORIES: Array<{ cat: FranchiseCategory; label: string }> = [
    { cat: 'spine',       label: 'Story chain' },
    { cat: 'source',      label: 'Sources & parents' },
    { cat: 'side',        label: 'Side stories & spin-offs' },
    { cat: 'alternative', label: 'Alternatives' },
    { cat: 'character',   label: 'Characters' },
    { cat: 'other',       label: 'Other' },
  ];
  const FORMATS: FranchiseFormat[] = ['series', 'movie', 'shortform', 'manga', 'novel', 'visualnovel', 'music', 'other'];

  return (
    <div className="franchise-filters">
      <div className="franchise-filters__group">
        {CATEGORIES.map(({ cat, label }) => (
          <button
            key={cat}
            type="button"
            data-category={cat}
            className={`franchise-filter-chip${hidden.has(cat) ? ' franchise-filter-chip--off' : ''}`}
            onClick={() => onToggle(cat)}
          >
            {label}
          </button>
        ))}
      </div>
      <div className="franchise-filters__group">
        {FORMATS.map((fmt) => (
          <button
            key={fmt}
            type="button"
            data-format={fmt}
            className={`franchise-filter-chip franchise-filter-chip--format${hiddenFormats.has(fmt) ? ' franchise-filter-chip--off' : ''}`}
            onClick={() => onToggleFormat(fmt)}
          >
            {FORMAT_LABELS[fmt]}
          </button>
        ))}
      </div>
    </div>
  );
}
