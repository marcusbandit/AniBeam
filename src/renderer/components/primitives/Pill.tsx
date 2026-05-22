import type { ReactNode } from 'react';

export type PillTone = 'muted' | 'accent' | 'teal' | 'rose' | 'amber';

interface PillProps {
  tone?: PillTone;
  children: ReactNode;
}

/**
 * Small status badge. Replaces .bare-episode-pill, .bare-episode-flag,
 * .relation-card-pill, .genre-pill (and the visual half of .show-card-badge).
 */
export default function Pill({ tone = 'muted', children }: PillProps) {
  return <span className={`pill pill--${tone}`}>{children}</span>;
}
