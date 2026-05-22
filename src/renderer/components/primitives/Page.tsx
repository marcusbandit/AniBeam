import type { ReactNode } from 'react';

interface PageProps {
  /** Optional page head — title, search, actions. Rendered with --s6 below. */
  head?: ReactNode;
  children: ReactNode;
  /** Override the default max-width for this page (e.g., metadata). */
  maxWidth?: number;
}

/**
 * Page shell. Replaces the bare <div className="page"> wrapping at the top
 * of every page component. Provides max-width, default vertical padding,
 * and a structural slot for the page head with consistent spacing.
 */
export default function Page({ head, children, maxWidth }: PageProps) {
  const style = maxWidth ? { maxWidth: `${maxWidth}px` } : undefined;
  return (
    <div className="page page--primitive" style={style}>
      {head && <div className="page__head">{head}</div>}
      {children}
    </div>
  );
}
