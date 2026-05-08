import { useEffect, useRef, type PointerEvent as ReactPointerEvent, type KeyboardEvent as ReactKeyboardEvent } from "react";
import { useTitleLanguage, type TitleLanguage } from "../contexts/TitleLanguageContext";

// Frame-rate-independent exponential smoothing per the lisyarus article:
//
//   pos += (target - pos) * (1 - exp(-speed * dt))
//
// `speed` (1/s) is the decay rate — 1/speed seconds is the time the gap
// closes by a factor of e (~63%). Empirical sweet spot for small UI
// motions is in the 5..50 range. 22 settles in ~250ms which feels
// snappy without being instant. Adjust here, not by adding easing curves
// elsewhere — exp() handles all dt values correctly without overshoot.
const SMOOTHING_SPEED = 22;
// Snap when residual is imperceptible; lets the rAF loop idle on a
// stable value instead of asymptotically approaching it forever.
const SETTLE_EPSILON = 0.0005;
// Pointer movement (as a fraction of track width) below which a release
// is treated as a click, not a drag. Prevents accidental drift from
// stealing taps.
const DRAG_THRESHOLD = 0.04;

interface DragState {
  pointerId: number;
  startX: number;
  startPos: number;
  trackWidth: number;
  moved: boolean;
}

function getReducedMotion(): boolean {
  return typeof window !== "undefined"
    && typeof window.matchMedia === "function"
    && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export default function LangSwitch() {
  const { lang, setLang } = useTitleLanguage();
  const trackRef = useRef<HTMLButtonElement>(null);
  const stateRef = useRef({
    pos: lang === "EN" ? 1 : 0,
    target: lang === "EN" ? 1 : 0,
  });
  const dragRef = useRef<DragState | null>(null);
  // Set right after a drag release so the synthetic click that follows
  // pointerup doesn't undo the drag-decided value.
  const justDraggedRef = useRef(false);

  // External lang change → update the rAF target. Don't write to .pos
  // directly; let the smoother glide there.
  useEffect(() => {
    stateRef.current.target = lang === "EN" ? 1 : 0;
  }, [lang]);

  // Single rAF driver for the lifetime of the component. Writes the
  // CSS custom property directly via ref — no React re-render per
  // frame.
  useEffect(() => {
    const reduced = getReducedMotion();
    let prev = performance.now();
    let raf = 0;

    const tick = (now: number) => {
      // Clamp dt: if the tab was backgrounded, dt could be huge and
      // we'd "settle" instantly. Cap to 100ms — at SPEED=22 that's
      // already ~90% closed, plenty close enough to be invisible.
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

  const onPointerDown = (e: ReactPointerEvent<HTMLButtonElement>) => {
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

  const onPointerMove = (e: ReactPointerEvent<HTMLButtonElement>) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== e.pointerId) return;
    const dx = (e.clientX - drag.startX) / Math.max(1, drag.trackWidth);
    if (Math.abs(dx) > DRAG_THRESHOLD) drag.moved = true;
    // Each segment is 50% of the track, so a full-track drag would
    // span 2× the thumb's available travel — multiply by 2 to keep
    // "thumb follows finger" 1:1 in pixel space. Clamp to [0, 1].
    const next = Math.max(0, Math.min(1, drag.startPos + dx * 2));
    stateRef.current.target = next;
  };

  const finishDrag = (e: ReactPointerEvent<HTMLButtonElement>) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== e.pointerId) return;
    try {
      e.currentTarget.releasePointerCapture(e.pointerId);
    } catch {
      // ignore — can throw if capture was already released by a cancel
    }
    dragRef.current = null;
    if (drag.moved) {
      const final: TitleLanguage = stateRef.current.target >= 0.5 ? "EN" : "JP";
      justDraggedRef.current = true;
      // setLang is idempotent; even if final == lang, the rAF target
      // has already been updated by onPointerMove and will settle.
      stateRef.current.target = final === "EN" ? 1 : 0;
      setLang(final);
    }
  };

  const onClick = () => {
    if (justDraggedRef.current) {
      justDraggedRef.current = false;
      return;
    }
    // Pure toggle. Click anywhere on the track flips. Drag is the
    // only way to choose a side directly.
    setLang(lang === "EN" ? "JP" : "EN");
  };

  const onKeyDown = (e: ReactKeyboardEvent<HTMLButtonElement>) => {
    if (e.key === "ArrowLeft" || e.key === "Home") {
      e.preventDefault();
      setLang("JP");
    } else if (e.key === "ArrowRight" || e.key === "End") {
      e.preventDefault();
      setLang("EN");
    } else if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      setLang(lang === "EN" ? "JP" : "EN");
    }
  };

  return (
    <button
      ref={trackRef}
      type="button"
      role="switch"
      aria-checked={lang === "EN"}
      aria-label={`Title language (currently ${lang === "EN" ? "English" : "Japanese romaji"})`}
      className="lang-switch"
      onClick={onClick}
      onKeyDown={onKeyDown}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={finishDrag}
      onPointerCancel={finishDrag}
    >
      <span className="lang-switch-thumb" aria-hidden="true" />
      <span className="lang-switch-label" data-active={lang === "JP"}>JP</span>
      <span className="lang-switch-label" data-active={lang === "EN"}>EN</span>
    </button>
  );
}
