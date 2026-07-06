import type { ReactNode } from 'react';

interface SectionProps {
  title: string;
  /** Optional small count badge next to the title (e.g., episode total). */
  count?: number | string;
  /** Optional right-aligned actions (e.g., "See all" link). */
  action?: ReactNode;
  children: ReactNode;
  /** Set true for the first section in a Page; suppresses the top gap. */
  first?: boolean;
}

/**
 * Section block with the eyebrow head treatment: mono uppercase label,
 * count in a small chip, a hairline rule filling the remaining width, and
 * a right-aligned action slot. The Related-pixel-brother bug is impossible
 * inside this primitive because section-gap is structural, not decorated
 * by hand.
 */
export default function Section({ title, count, action, children, first }: SectionProps) {
  return (
    <section className={`section--primitive${first ? ' section--first' : ''}`}>
      <header className="section__head">
        <h2 className="section__title">{title}</h2>
        {count !== undefined && <span className="chip chip--sm section__count">{count}</span>}
        <span className="section__rule" aria-hidden="true" />
        {action && <div className="section__action">{action}</div>}
      </header>
      <div className="section__body">{children}</div>
    </section>
  );
}
