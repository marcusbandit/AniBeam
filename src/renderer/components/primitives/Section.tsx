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
 * Section block. The Related-pixel-brother bug is impossible inside this
 * primitive because section-gap is structural, not decorated by hand.
 */
export default function Section({ title, count, action, children, first }: SectionProps) {
  return (
    <section className={`section--primitive${first ? ' section--first' : ''}`}>
      <header className="section__head">
        <div className="section__title-group">
          <h2 className="section__title">{title}</h2>
          {count !== undefined && <span className="section__count">{count}</span>}
        </div>
        {action && <div className="section__action">{action}</div>}
      </header>
      <div className="section__body">{children}</div>
    </section>
  );
}
