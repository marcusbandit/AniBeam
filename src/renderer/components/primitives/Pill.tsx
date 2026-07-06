import type { ReactNode } from 'react';

/** 'accent' is a legacy alias kept for existing callers; it renders teal. */
export type PillTone = 'muted' | 'accent' | 'teal' | 'rose' | 'amber' | 'blue' | 'violet';

interface PillProps {
  tone?: PillTone;
  /** Compact size (--chip-font-sm). Default is the regular chip size. */
  size?: 'sm';
  /** Poster-overlay dark-glass variant (the only scrim recipe in the app). */
  scrim?: boolean;
  /** Interactive chip (filter pill / tag toggle): hover paint + press scale. */
  toggle?: boolean;
  /** Toggle on-state: teal text, teal-tinted border, --accent-glow bg. */
  on?: boolean;
  /** Leading state dot in currentColor; 'pulse' animates it for live states. */
  dot?: boolean | 'pulse';
  /** Tracker list status (watching/completed/...); wins over `tone`. */
  status?: string;
  /** AniList MediaFormat (TV/MOVIE/...); wins over `tone`. */
  format?: string;
  /** When set, renders a <button> so the chip is really clickable. */
  onClick?: () => void;
  children: ReactNode;
}

/**
 * Chip primitive (the one badge system). Renders the `.chip` family from
 * styles/primitives.css. Replaces every badge/pill/tag shell in the app:
 * hero chips, format pills, show-card badges, source pills, filter pills,
 * franchise status tags.
 */
export default function Pill({
  tone = 'muted', size, scrim, toggle, on, dot, status, format, onClick, children,
}: PillProps) {
  const classes = ['chip'];
  if (size === 'sm') classes.push('chip--sm');
  // data-status / data-format carry their own coloring; only add a tone
  // class when neither is present so the data-attr recipe stays the sole
  // paint source.
  if (!status && !format) classes.push(`chip--${tone === 'accent' ? 'teal' : tone}`);
  if (scrim) classes.push('chip--scrim');
  if (toggle) classes.push('chip--toggle');
  if (on) classes.push('is-on');

  const dotNode = dot
    ? <span className={`chip__dot${dot === 'pulse' ? ' chip__dot--pulse' : ''}`} aria-hidden="true" />
    : null;
  const className = classes.join(' ');

  if (onClick) {
    return (
      <button
        type="button"
        className={className}
        data-status={status}
        data-format={format}
        onClick={onClick}
        aria-pressed={toggle ? !!on : undefined}
      >
        {dotNode}
        {children}
      </button>
    );
  }
  return (
    <span className={className} data-status={status} data-format={format}>
      {dotNode}
      {children}
    </span>
  );
}
