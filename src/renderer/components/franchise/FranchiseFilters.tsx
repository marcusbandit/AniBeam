import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import type { ReactElement, ReactNode } from 'react';
import { createPortal } from 'react-dom';
import { ChevronDown } from 'lucide-react';

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
 * used by the filter chips. Pure function - no side effects.
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

interface FilterMenuProps {
  label: string;
  /** How many of this group's filters are active (i.e. hiding something). */
  hiddenCount: number;
  /** Controlled by the parent so at most one menu is ever open. */
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: ReactNode;
}

/**
 * Click-to-open filter dropdown. The two wrapping chip rows cluttered the
 * graph's top-center panel, so each group collapses behind a single trigger
 * button and the chips live in a portalled popover, positioned with fixed
 * viewport coords that track the trigger on scroll/resize.
 *
 * Portal target mirrors ScorePicker: normally <body>, but when the Fullscreen
 * API top layer is active we portal into document.fullscreenElement, because
 * anything portalled to <body> renders BEHIND the top layer. The graph's own
 * fullscreen is CSS position:fixed (z-index 99999), not the Fullscreen API,
 * so <body> is correct there and the popover out-stacks it via z-index; the
 * fullscreenElement fallback stays for safety should an ancestor ever use
 * the real API.
 */
function FilterMenu({ label, hiddenCount, open, onOpenChange, children }: FilterMenuProps): ReactElement {
  const [anchor, setAnchor] = useState<{ left: number; top: number } | null>(null);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const panelRef = useRef<HTMLDivElement | null>(null);

  // Recompute anchor on open + on scroll/resize while open so the panel
  // tracks the trigger if anything moves underneath it.
  const recomputeAnchor = useCallback(() => {
    const el = triggerRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    setAnchor({ left: r.left + r.width / 2, top: r.bottom + 6 });
  }, []);

  useLayoutEffect(() => {
    if (!open) { setAnchor(null); return; }
    recomputeAnchor();
  }, [open, recomputeAnchor]);

  useEffect(() => {
    if (!open) return;
    // Ignore scrolls that originate inside the panel itself - they cannot
    // move the trigger, so recomputing would just burn CPU per wheel tick.
    const onScroll = (e: Event) => {
      if (panelRef.current?.contains(e.target as Node)) return;
      recomputeAnchor();
    };
    const onResize = () => recomputeAnchor();
    window.addEventListener('scroll', onScroll, true);
    window.addEventListener('resize', onResize);
    return () => {
      window.removeEventListener('scroll', onScroll, true);
      window.removeEventListener('resize', onResize);
    };
  }, [open, recomputeAnchor]);

  // Click-outside and Escape both close without touching any filter. The
  // panel is portalled, so "outside" must check the trigger root AND the
  // panel ref. Mousedown (not click) so a fresh press on the trigger toggles
  // cleanly instead of close-then-reopen.
  useEffect(() => {
    if (!open) return;
    const onMouseDown = (e: MouseEvent) => {
      const target = e.target as Node;
      if (rootRef.current?.contains(target)) return;
      if (panelRef.current?.contains(target)) return;
      onOpenChange(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onOpenChange(false);
    };
    window.addEventListener('mousedown', onMouseDown);
    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('mousedown', onMouseDown);
      window.removeEventListener('keydown', onKey);
    };
  }, [open, onOpenChange]);

  // Re-home the portal if fullscreen toggles while open (same fix as
  // ScorePicker: a re-render re-reads document.fullscreenElement below).
  const [, bumpPortalTarget] = useState(0);
  useEffect(() => {
    if (!open) return;
    const onFs = () => bumpPortalTarget((n) => n + 1);
    document.addEventListener('fullscreenchange', onFs);
    return () => document.removeEventListener('fullscreenchange', onFs);
  }, [open]);
  const portalTarget = typeof document !== 'undefined'
    ? (document.fullscreenElement ?? document.body)
    : null;

  const panel = open && anchor && portalTarget
    ? createPortal(
        <div
          ref={panelRef}
          className="franchise-filter-menu-panel"
          data-liquid-glass=""
          data-lg-bezel="10"
          role="group"
          aria-label={`${label} filters`}
          style={{
            position: 'fixed',
            left: `${anchor.left}px`,
            top: `${anchor.top}px`,
            transform: 'translateX(-50%)',
          }}
        >
          {children}
        </div>,
        portalTarget,
      )
    : null;

  return (
    <div ref={rootRef} className="franchise-filter-menu">
      <button
        ref={triggerRef}
        type="button"
        className={`chip chip--sm chip--toggle franchise-filter-menu-trigger${open ? ' is-open' : ''}`}
        onClick={() => onOpenChange(!open)}
        aria-haspopup="true"
        aria-expanded={open}
      >
        <span>{label}</span>
        {hiddenCount > 0 && <span className="franchise-filter-menu-count">{hiddenCount}</span>}
        <ChevronDown size={14} strokeWidth={2.25} aria-hidden="true" />
      </button>
      {panel}
    </div>
  );
}

export function FranchiseFilters(props: FranchiseFiltersProps): ReactElement {
  const { hidden, onToggle, hiddenFormats, onToggleFormat } = props;
  // At most one popover open at a time; opening one closes the other.
  const [openMenu, setOpenMenu] = useState<'relations' | 'formats' | null>(null);
  const onRelationsOpenChange = useCallback((open: boolean) => {
    setOpenMenu(open ? 'relations' : null);
  }, []);
  const onFormatsOpenChange = useCallback((open: boolean) => {
    setOpenMenu(open ? 'formats' : null);
  }, []);

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
      <FilterMenu
        label="Relations"
        hiddenCount={hidden.size}
        open={openMenu === 'relations'}
        onOpenChange={onRelationsOpenChange}
      >
        <div className="franchise-filters__group">
          {CATEGORIES.map(({ cat, label }) => (
            <button
              key={cat}
              type="button"
              data-category={cat}
              aria-pressed={!hidden.has(cat)}
              className={`chip chip--sm chip--toggle franchise-filter-chip${hidden.has(cat) ? '' : ' is-on'}`}
              onClick={() => onToggle(cat)}
            >
              <span className="chip__dot" aria-hidden="true" />
              {label}
            </button>
          ))}
        </div>
      </FilterMenu>

      <FilterMenu
        label="Formats"
        hiddenCount={hiddenFormats.size}
        open={openMenu === 'formats'}
        onOpenChange={onFormatsOpenChange}
      >
        <div className="franchise-filters__group">
          {/* data-format-group (not data-format) so the primitives' .chip[data-format]
              AniList-enum tinting can't half-match these grouped values. */}
          {FORMATS.map((fmt) => (
            <button
              key={fmt}
              type="button"
              data-format-group={fmt}
              aria-pressed={!hiddenFormats.has(fmt)}
              className={`chip chip--sm chip--toggle franchise-filter-chip${hiddenFormats.has(fmt) ? '' : ' is-on'}`}
              onClick={() => onToggleFormat(fmt)}
            >
              <span className="chip__dot" aria-hidden="true" />
              {FORMAT_LABELS[fmt]}
            </button>
          ))}
        </div>
      </FilterMenu>
    </div>
  );
}
