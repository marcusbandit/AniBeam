import {
  useEffect,
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

      if (trackRef.current) {
        trackRef.current.style.setProperty("--thumb-pos", s.pos.toFixed(4));
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, []);

  const segmentFromClientX = (clientX: number, rect: DOMRect): number => {
    const rel = (clientX - rect.left) / Math.max(1, rect.width);
    const idx = Math.floor(rel * count);
    return Math.max(0, Math.min(count - 1, idx));
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
      const rect = e.currentTarget.getBoundingClientRect();
      const idx = segmentFromClientX(e.clientX, rect);
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

  // CSS owns the visuals via `--seg-count` (sets the thumb width and the
  // grid column count) and `--thumb-pos` (drives translate as a multiple of
  // one-segment width). Keeping the math in CSS keeps drag follow exact.
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
