import { useEffect, useRef, type ReactNode, type ReactElement } from 'react';
import { smoothScalar, type SmoothHandle } from '../../utils/motion';
import Tooltip from './Tooltip';

const LIFT_SPEED = 12;
const LIFT_AMOUNT_PX = 2;

/**
 * Visual variants. "in-progress" intentionally is NOT a row variant - the
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
  /** 0..1 - hovered seekbar fill. The bar is only visible on hover. */
  progress?: number;
  state?: EpisodeRowState;
  onClick?: () => void;
  /** Pointer entered the row — used to prewarm the episode (e.g. its subtitles)
   *  before the likely click. Fires alongside the internal lift animation. */
  onHover?: () => void;
  disabled?: boolean;
  /** Hover tooltip on the marker circle (e.g. "untrack to here"). */
  markerTooltip?: string;
  /**
   * Cascade tone when this marker is part of an active range: 'untrack' (red) or
   * 'track' (blue). Combined with `markerPhase` it picks the keyframe.
   */
  markerMode?: 'untrack' | 'track';
  /**
   * Cascade phase: 'in' (hover), 'out' (reverse wave on un-hover), 'commit'
   * (after a click - settles to the new tracked/untracked colour).
   */
  markerPhase?: 'in' | 'out' | 'commit';
  /** Per-marker animation-delay (ms) so the wave staggers out from the cursor. */
  markerCascadeDelayMs?: number;
  /** Click on just the marker - fires instead of `onClick` (stops propagation). */
  onMarkerClick?: () => void;
  /** Entering the CIRCLE - initiates / re-anchors the hover wave. */
  onMarkerEnter?: () => void;
  /** Entering the tall hit-zone (but not necessarily the circle) - only keeps an
   *  existing hover alive (cancels the pending leave); never initiates. */
  onMarkerZoneEnter?: () => void;
  /** Leaving the hit-zone - debounced un-hover. */
  onMarkerLeave?: () => void;
}

/**
 * EpisodeRow primitive. Replaces .bare-episode-row.
 *
 * Layout: [marker | code | title | trailing] above a reserved seekbar row.
 * The seekbar slot is structural (its own row in a grid), so the text's
 * vertical centering is independent of the seekbar's presence - fixing the
 * off-center bug.
 */
export default function EpisodeRow({
  marker, code, title, trailing,
  progress = 0,
  state = 'default', onClick, onHover, disabled,
  markerTooltip, markerMode, markerPhase, markerCascadeDelayMs,
  onMarkerClick, onMarkerEnter, onMarkerZoneEnter, onMarkerLeave,
}: EpisodeRowProps) {
  const elRef = useRef<HTMLButtonElement | null>(null);
  const liftRef = useRef<SmoothHandle | null>(null);
  // Hover-intent timer for onHover: fire only after the pointer rests on the
  // row, so sweeping a long episode list doesn't kick off a prewarm per row.
  const hoverTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const el = elRef.current;
    if (!el || disabled) return;
    const handle = smoothScalar(0, LIFT_SPEED, (v) => {
      el.style.transform = Math.abs(v) > 0.05 ? `translateY(${v.toFixed(2)}px)` : '';
    });
    liftRef.current = handle;
    return () => { handle.release(); el.style.transform = ''; };
  }, [disabled]);

  // Clear any pending hover-intent timer on unmount.
  useEffect(() => () => {
    if (hoverTimerRef.current) clearTimeout(hoverTimerRef.current);
  }, []);

  const onEnter = () => {
    if (disabled) return;
    liftRef.current?.setTarget(-LIFT_AMOUNT_PX);
    if (onHover) {
      if (hoverTimerRef.current) clearTimeout(hoverTimerRef.current);
      hoverTimerRef.current = setTimeout(() => { hoverTimerRef.current = null; onHover(); }, 220);
    }
  };
  const onLeave = () => {
    liftRef.current?.setTarget(0);
    if (hoverTimerRef.current) { clearTimeout(hoverTimerRef.current); hoverTimerRef.current = null; }
  };

  const className = `episode-row episode-row--${state}`;
  const pct = Math.max(0, Math.min(1, progress)) * 100;

  const markerInteractive = !!onMarkerClick;
  // mode+phase → keyframe class: --untracking[-out|-commit] / --tracking[-out|-commit].
  const phaseSuffix = markerPhase === 'out' ? '-out' : markerPhase === 'commit' ? '-commit' : '';
  const activeClass = markerMode
    ? ` episode-row__marker--${markerMode === 'untrack' ? 'untracking' : 'tracking'}${phaseSuffix}`
    : '';
  // The visible circle carries the cascade colour + bob, and - when interactive -
  // owns hover INITIATION via its own onMouseEnter.
  const circle = (
    <span
      className={`episode-row__marker${activeClass}`}
      style={markerMode ? { animationDelay: `${markerCascadeDelayMs ?? 0}ms` } : undefined}
      onMouseEnter={markerInteractive ? onMarkerEnter : undefined}
    >
      {marker}
    </span>
  );
  // Interactive marker: a tall transparent hit-zone WRAPS the circle. Entering
  // the zone only keeps an active hover alive (its column tiles, so moving
  // zone→zone never un-hovers); a hover is only INITIATED by entering the circle
  // itself. The zone leaves only fire on true exit (the circle is a child), and
  // the click target is the whole zone. A sibling grid slot holds column 1 since
  // the zone is absolutely positioned and out of grid flow.
  let markerArea: ReactNode = circle;
  if (markerInteractive) {
    let hit: ReactNode = (
      <span
        className="episode-row__marker-hit"
        onMouseEnter={onMarkerZoneEnter}
        onMouseLeave={onMarkerLeave}
        onClick={(e) => { e.stopPropagation(); onMarkerClick?.(); }}
      >
        {circle}
      </span>
    );
    if (markerTooltip) hit = <Tooltip label={markerTooltip}>{hit as ReactElement}</Tooltip>;
    markerArea = (
      <>
        <span className="episode-row__marker-slot" aria-hidden="true" />
        {hit}
      </>
    );
  }

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
      {markerArea}
      <span className="episode-row__code">{code}</span>
      <span className="episode-row__title">{title}</span>
      <span className="episode-row__trailing">{trailing}</span>
      <span className="episode-row__progress" aria-hidden="true">
        <span className="episode-row__progress-fill" style={{ width: `${pct}%` }} />
      </span>
    </button>
  );
}
