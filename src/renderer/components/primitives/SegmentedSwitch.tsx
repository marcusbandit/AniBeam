import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  type PointerEvent as ReactPointerEvent,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
} from "react";
import Tooltip from "./Tooltip";

// Frame-rate-independent exponential smoothing per the lisyarus article:
//
//   pos += (target - pos) * (1 - exp(-speed * dt))
//
// Same constants as LangSwitch — see the comments there for why these numbers.
const SMOOTHING_SPEED = 22;
const SETTLE_EPSILON = 0.0005;
// Pointer movement (as a fraction of one segment width) below which a release
// is treated as a click, not a drag. Independent of segment count.
const DRAG_THRESHOLD = 0.08;

interface DragState {
  pointerId: number;
  startX: number;
  startPos: number;
  trackWidth: number;
  moved: boolean;
}

export interface SegmentedOption<T extends string> {
  value: T;
  /** Visible label inside the segment. Strings render as the segment text;
   *  ReactNode lets callers drop an icon (e.g., direction arrows). */
  label: ReactNode;
  /** Optional aria-label override when `label` is a non-text node. */
  ariaLabel?: string;
}

interface SegmentedSwitchProps<T extends string> {
  value: T;
  options: SegmentedOption<T>[];
  onChange: (next: T) => void;
  /** Accessible name for the whole control (e.g. "Sort key"). */
  ariaLabel: string;
  /** Extra class so callers can size or tweak per-instance. */
  className?: string;
}

function getReducedMotion(): boolean {
  return typeof window !== "undefined"
    && typeof window.matchMedia === "function"
    && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/**
 * N-position role="switch" track with a smoothly animated thumb.
 *
 * Mirrors the LangSwitch recipe — rAF + Math.exp owns the motion, the DOM
 * never re-renders on a tick — but generalised so the same shape works for
 * 2 to N options. Persistence is the caller's job; this just owns motion,
 * pointer/drag handling, keyboard nav, and ARIA.
 */
export default function SegmentedSwitch<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
  className,
}: SegmentedSwitchProps<T>) {
  const count = options.length;
  const activeIndex = Math.max(0, options.findIndex((o) => o.value === value));
  const trackRef = useRef<HTMLDivElement>(null);
  const stateRef = useRef({ pos: activeIndex, target: activeIndex });
  const dragRef = useRef<DragState | null>(null);
  // Per-segment geometry (left offset + width, px, relative to the track's
  // padding box) so the thumb can size & slide to content-width segments
  // instead of assuming every segment is an equal 1/N slice. Re-measured on
  // mount, option changes, and any track resize.
  const geomRef = useRef<Array<{ left: number; width: number }>>([]);

  const measure = useCallback(() => {
    const track = trackRef.current;
    if (!track) return;
    const trackRect = track.getBoundingClientRect();
    const borderL = parseFloat(getComputedStyle(track).borderLeftWidth) || 0;
    const labels = Array.from(
      track.querySelectorAll<HTMLElement>(".segmented-switch__label"),
    );
    geomRef.current = labels.map((el) => {
      const r = el.getBoundingClientRect();
      // Convert from the track's border-box origin to its padding box, which
      // is what `left: 0` on the absolutely-positioned thumb anchors to.
      return { left: r.left - trackRect.left - borderL, width: r.width };
    });
  }, []);

  // Measure before paint so the thumb never flashes at zero width, and keep
  // it in sync with layout changes (font load, container resize, option set).
  useLayoutEffect(() => {
    measure();
    const track = trackRef.current;
    if (!track || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(() => measure());
    ro.observe(track);
    return () => ro.disconnect();
  }, [measure, options]);
  // Set right after a drag release so the synthetic click on the segment
  // button that follows pointerup doesn't immediately re-select something
  // different from what the drag landed on.
  const justDraggedRef = useRef(false);

  // External value change → glide to the new index. Don't snap `.pos`.
  useEffect(() => {
    stateRef.current.target = activeIndex;
  }, [activeIndex]);

  // Single rAF driver for the lifetime of the component. Writes the CSS
  // custom property directly via ref — no React re-render per frame.
  useEffect(() => {
    const reduced = getReducedMotion();
    let prev = performance.now();
    let raf = 0;

    const tick = (now: number) => {
      const dt = Math.min(0.1, (now - prev) / 1000);
      prev = now;

      const s = stateRef.current;
      if (reduced) {
        s.pos = s.target;
      } else {
        s.pos += (s.target - s.pos) * (1 - Math.exp(-SMOOTHING_SPEED * dt));
        if (Math.abs(s.pos - s.target) < SETTLE_EPSILON) s.pos = s.target;
      }

      // Interpolate the thumb's box between the two segments `pos` straddles,
      // so it both slides AND morphs width as it crosses a narrow→wide gap.
      const geom = geomRef.current;
      if (trackRef.current && geom.length > 0) {
        const maxIdx = geom.length - 1;
        const clamped = Math.max(0, Math.min(maxIdx, s.pos));
        const i0 = Math.floor(clamped);
        const i1 = Math.min(maxIdx, i0 + 1);
        const frac = clamped - i0;
        const g0 = geom[i0];
        const g1 = geom[i1];
        const x = g0.left + (g1.left - g0.left) * frac;
        const w = g0.width + (g1.width - g0.width) * frac;
        trackRef.current.style.setProperty("--thumb-x", `${x.toFixed(2)}px`);
        trackRef.current.style.setProperty("--thumb-w", `${w.toFixed(2)}px`);
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, []);

  const segmentFromClientX = (clientX: number): number => {
    const track = trackRef.current;
    const geom = geomRef.current;
    if (!track || geom.length === 0) return activeIndex;
    const trackRect = track.getBoundingClientRect();
    const borderL = parseFloat(getComputedStyle(track).borderLeftWidth) || 0;
    const x = clientX - trackRect.left - borderL;
    for (let i = 0; i < geom.length; i++) {
      if (x < geom[i].left + geom[i].width) return i;
    }
    return geom.length - 1;
  };

  const onPointerDown = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    const el = e.currentTarget;
    const rect = el.getBoundingClientRect();
    el.setPointerCapture(e.pointerId);
    dragRef.current = {
      pointerId: e.pointerId,
      startX: e.clientX,
      startPos: stateRef.current.pos,
      trackWidth: rect.width,
      moved: false,
    };
  };

  const onPointerMove = (e: ReactPointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== e.pointerId) return;
    // Convert pointer dx into "segments traveled" — 1 full track width
    // == (count - 1) segments of thumb travel.
    const segmentDx = ((e.clientX - drag.startX) / Math.max(1, drag.trackWidth)) * (count - 1);
    if (Math.abs(segmentDx) > DRAG_THRESHOLD) drag.moved = true;
    const next = Math.max(0, Math.min(count - 1, drag.startPos + segmentDx));
    stateRef.current.target = next;
  };

  const finishDrag = (e: ReactPointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== e.pointerId) return;
    try {
      e.currentTarget.releasePointerCapture(e.pointerId);
    } catch {
      // capture may have already been released by a cancel
    }
    dragRef.current = null;
    if (drag.moved) {
      const finalIdx = Math.round(stateRef.current.target);
      justDraggedRef.current = true;
      stateRef.current.target = finalIdx;
      const opt = options[finalIdx];
      if (opt && opt.value !== value) onChange(opt.value);
    } else {
      // Treat as a click — segment under the pointer wins.
      const idx = segmentFromClientX(e.clientX);
      const opt = options[idx];
      if (opt && opt.value !== value) onChange(opt.value);
    }
  };

  const onSegmentClick = (idx: number) => {
    if (justDraggedRef.current) {
      justDraggedRef.current = false;
      return;
    }
    const opt = options[idx];
    if (opt && opt.value !== value) onChange(opt.value);
  };

  const onKeyDown = (e: ReactKeyboardEvent<HTMLDivElement>) => {
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      const next = Math.max(0, activeIndex - 1);
      if (next !== activeIndex) onChange(options[next].value);
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      const next = Math.min(count - 1, activeIndex + 1);
      if (next !== activeIndex) onChange(options[next].value);
    } else if (e.key === "Home") {
      e.preventDefault();
      if (activeIndex !== 0) onChange(options[0].value);
    } else if (e.key === "End") {
      e.preventDefault();
      if (activeIndex !== count - 1) onChange(options[count - 1].value);
    }
  };

  // Segments are content-width (`--seg-count` still drives the grid column
  // count); the thumb is positioned & sized in px via `--thumb-x` / `--thumb-w`,
  // which the rAF loop interpolates from the measured per-segment geometry so
  // it tracks variable-width segments exactly.
  return (
    <div
      ref={trackRef}
      role="radiogroup"
      aria-label={ariaLabel}
      className={`segmented-switch${className ? ` ${className}` : ""}`}
      style={{ ["--seg-count" as never]: count } as React.CSSProperties}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={finishDrag}
      onPointerCancel={finishDrag}
      onKeyDown={onKeyDown}
      tabIndex={0}
    >
      <span className="segmented-switch__thumb" aria-hidden="true" />
      {options.map((opt, idx) => {
        const button = (
          <button
            type="button"
            role="radio"
            aria-checked={idx === activeIndex}
            aria-label={opt.ariaLabel}
            className="segmented-switch__label"
            data-active={idx === activeIndex}
            tabIndex={-1}
            onClick={() => onSegmentClick(idx)}
          >
            {opt.label}
          </button>
        );
        // Custom tooltip mirrors aria-label so hovering an icon-only segment
        // reveals what it means in the app's design language.
        return opt.ariaLabel
          ? <Tooltip key={opt.value} label={opt.ariaLabel}>{button}</Tooltip>
          : <span key={opt.value}>{button}</span>;
      })}
    </div>
  );
}
