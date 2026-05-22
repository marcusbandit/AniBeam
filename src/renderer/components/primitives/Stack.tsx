import type { CSSProperties, HTMLAttributes, ReactNode } from 'react';

export type SpaceToken =
  | 's1' | 's2' | 's3' | 's4' | 's5' | 's6' | 's8' | 's10' | 's12' | 's16';

interface StackProps extends HTMLAttributes<HTMLDivElement> {
  gap?: SpaceToken;
  children: ReactNode;
}
interface InlineProps extends StackProps {
  /** flexbox align-items — defaults to 'center'. */
  align?: CSSProperties['alignItems'];
  /** flexbox justify-content. */
  justify?: CSSProperties['justifyContent'];
  /** wrap rows when content overflows. */
  wrap?: boolean;
}

/**
 * Vertical flex container with a token-driven gap. Replaces ad-hoc
 * margin-top/bottom decisions in page-local CSS.
 */
export function Stack({ gap = 's4', style, children, ...rest }: StackProps) {
  return (
    <div
      style={{ display: 'flex', flexDirection: 'column', gap: `var(--${gap})`, ...style }}
      {...rest}
    >
      {children}
    </div>
  );
}

/**
 * Horizontal flex container with a token-driven gap.
 */
export function Inline({
  gap = 's2',
  align = 'center',
  justify,
  wrap,
  style,
  children,
  ...rest
}: InlineProps) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: align,
        justifyContent: justify,
        flexWrap: wrap ? 'wrap' : 'nowrap',
        gap: `var(--${gap})`,
        ...style,
      }}
      {...rest}
    >
      {children}
    </div>
  );
}
