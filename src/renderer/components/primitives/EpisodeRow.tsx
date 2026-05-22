import { useEffect, useRef, type ReactNode } from 'react';
import { smoothScalar, type SmoothHandle } from '../../utils/motion';

const LIFT_SPEED = 12;
const LIFT_AMOUNT_PX = 2;

/**
 * Visual variants. "in-progress" intentionally is NOT a row variant — the
 * partial-watch signal is carried by `progressVisibleAtRest` + the seekbar
 * opacity, not by a different row color. Keeping it out of the enum
 * prevents callers from coding to a CSS class that wouldn't exist.
 */
export type EpisodeRowState = 'default' | 'next-up' | 'watched';

interface EpisodeRowProps {
  marker: ReactNode;       // play / check / icon
  code: ReactNode;         // S01E03
  title: ReactNode;
  trailing?: ReactNode;    // pill, flag, "Next up", etc.
  /** 0..1 — hovered seekbar fill. The bar is only visible on hover. */
  progress?: number;
  state?: EpisodeRowState;
  onClick?: () => void;
  disabled?: boolean;
}

/**
 * EpisodeRow primitive. Replaces .bare-episode-row.
 *
 * Layout: [marker | code | title | trailing] above a reserved seekbar row.
 * The seekbar slot is structural (its own row in a grid), so the text's
 * vertical centering is independent of the seekbar's presence — fixing the
 * off-center bug.
 */
export default function EpisodeRow({
  marker, code, title, trailing,
  progress = 0,
  state = 'default', onClick, disabled,
}: EpisodeRowProps) {
  const elRef = useRef<HTMLButtonElement | null>(null);
  const liftRef = useRef<SmoothHandle | null>(null);

  useEffect(() => {
    const el = elRef.current;
    if (!el || disabled) return;
    const handle = smoothScalar(0, LIFT_SPEED, (v) => {
      el.style.transform = Math.abs(v) > 0.05 ? `translateY(${v.toFixed(2)}px)` : '';
    });
    liftRef.current = handle;
    return () => { handle.release(); el.style.transform = ''; };
  }, [disabled]);

  const onEnter = () => { if (!disabled) liftRef.current?.setTarget(-LIFT_AMOUNT_PX); };
  const onLeave = () => liftRef.current?.setTarget(0);

  const className = `episode-row episode-row--${state}`;
  const pct = Math.max(0, Math.min(1, progress)) * 100;

  return (
    <button
      ref={elRef}
      type="button"
      className={className}
      onClick={onClick}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      disabled={disabled}
    >
      <span className="episode-row__marker">{marker}</span>
      <span className="episode-row__code">{code}</span>
      <span className="episode-row__title">{title}</span>
      <span className="episode-row__trailing">{trailing}</span>
      <span className="episode-row__progress" aria-hidden="true">
        <span className="episode-row__progress-fill" style={{ width: `${pct}%` }} />
      </span>
    </button>
  );
}
