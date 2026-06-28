import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { ChevronDown } from 'lucide-react';

interface ScorePickerProps {
  /** Current value as a string in 0.1-step decimal form ("0.0" … "10.0"). */
  value: string;
  onChange: (next: string) => void;
  disabled?: boolean;
  /** Class on the trigger button so callers can size it to match siblings. */
  className?: string;
  /**
   * Optional anchor element id forwarded to ARIA attributes. Default is fine
   * for the inline use cases inside the player and the hero popover.
   */
  ariaLabel?: string;
}

// 0.0 → 10.0 in 0.1 steps. Precomputed at module load so opening the panel
// doesn't allocate 101 strings on every render.
const SCORE_OPTIONS: readonly string[] = Object.freeze(
  Array.from({ length: 101 }, (_, i) => (i / 10).toFixed(1)),
);

/**
 * Custom themed score dropdown - replaces the native <select>, whose
 * Chromium popup ignores our dark tokens entirely. The panel renders in
 * the app's mono/dark language, scrolls smoothly, and snaps the selected
 * option into view on open so a 0.1-step list of 101 values is actually
 * usable.
 */
export function ScorePicker({ value, onChange, disabled, className, ariaLabel }: ScorePickerProps) {
  const [open, setOpen] = useState(false);
  // Anchor coords for the portalled panel. We track viewport-relative left/
  // top/width so position: fixed math is straightforward - when the user
  // scrolls the page or resizes, we recompute these and the panel follows.
  const [anchor, setAnchor] = useState<{ left: number; top: number; width: number } | null>(null);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const panelRef = useRef<HTMLUListElement | null>(null);
  const optionRefs = useRef<Array<HTMLLIElement | null>>([]);

  const options = useMemo(() => SCORE_OPTIONS, []);

  const close = useCallback(() => setOpen(false), []);

  // Recompute anchor on open + on scroll/resize while open so the panel
  // tracks the trigger if anything moves underneath it.
  const recomputeAnchor = useCallback(() => {
    const el = triggerRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    setAnchor({ left: r.left, top: r.bottom + 4, width: r.width });
  }, []);

  useLayoutEffect(() => {
    if (!open) { setAnchor(null); return; }
    recomputeAnchor();
  }, [open, recomputeAnchor]);

  useEffect(() => {
    if (!open) return;
    // Only recompute when something OUTSIDE the panel scrolls - the panel's
    // own scrollTop changes don't move the trigger, so reacting to them
    // would (a) burn CPU on every wheel tick and (b) thrash the snap-to-
    // selected effect, fighting the user's scroll input. Filter by target.
    const onScroll = (e: Event) => {
      if (panelRef.current?.contains(e.target as Node)) return;
      recomputeAnchor();
    };
    const onResize = () => recomputeAnchor();
    window.addEventListener('scroll', onScroll, true);
    window.addEventListener('resize', onResize);
    return () => {
      window.removeEventListener('scroll', onScroll, true);
      window.removeEventListener('resize', onResize);
    };
  }, [open, recomputeAnchor]);

  // Click-outside and Escape - both unmount the panel without changing
  // the value. Mouse-down (not click) so a fresh click on the trigger
  // toggles cleanly without us first closing then reopening. The panel is
  // portalled to <body>, so we have to check both the trigger root AND the
  // panel ref to decide whether the click is "outside".
  useEffect(() => {
    if (!open) return;
    const onMouseDown = (e: MouseEvent) => {
      const target = e.target as Node;
      if (rootRef.current?.contains(target)) return;
      if (panelRef.current?.contains(target)) return;
      close();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    window.addEventListener('mousedown', onMouseDown);
    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('mousedown', onMouseDown);
      window.removeEventListener('keydown', onKey);
    };
  }, [open, close]);

  // Snap the selected option into view ONCE per open. If we re-ran this
  // on every anchor recompute or value change, the user couldn't scroll
  // - every wheel tick would yank the panel back to the selected row.
  // We track whether the current open cycle has already snapped via a
  // ref. Reset on close.
  const snappedRef = useRef(false);
  useLayoutEffect(() => {
    if (!open) { snappedRef.current = false; return; }
    if (!anchor || snappedRef.current) return;
    const idx = options.indexOf(value);
    if (idx < 0) { snappedRef.current = true; return; }
    const panel = panelRef.current;
    const li = optionRefs.current[idx];
    if (!panel || !li) return;
    const offset = li.offsetTop - panel.clientHeight / 2 + li.clientHeight / 2;
    panel.scrollTop = Math.max(0, offset);
    snappedRef.current = true;
  }, [open, anchor, value, options]);

  // Portal target. Normally <body>, but when the app is in the Fullscreen API
  // top layer (the in-window video player calls requestFullscreen() on its
  // wrapper), anything portalled to <body> renders BEHIND the fullscreen
  // element and is invisible — body is an ancestor of the fullscreen element,
  // not a descendant, so it isn't promoted into the top layer. Portalling into
  // the fullscreen element instead keeps the panel in the same top layer so it
  // shows on top. position:fixed viewport coords still line up because the
  // fullscreen element fills the viewport. Re-render on fullscreenchange while
  // open so toggling fullscreen mid-open re-homes the panel.
  const [, bumpPortalTarget] = useState(0);
  useEffect(() => {
    if (!open) return;
    const onFs = () => bumpPortalTarget((n) => n + 1);
    document.addEventListener('fullscreenchange', onFs);
    return () => document.removeEventListener('fullscreenchange', onFs);
  }, [open]);
  const portalTarget = typeof document !== 'undefined'
    ? (document.fullscreenElement ?? document.body)
    : null;

  const panel = open && anchor && portalTarget
    ? createPortal(
        <ul
          ref={panelRef}
          className="score-picker-panel"
          data-liquid-glass=""
          data-lg-bezel="10"
          role="listbox"
          aria-label="Select score"
          tabIndex={-1}
          style={{
            position: 'fixed',
            left: `${anchor.left}px`,
            top: `${anchor.top}px`,
            minWidth: `${anchor.width}px`,
          }}
        >
          {options.map((opt, i) => {
            const selected = opt === value;
            return (
              <li
                key={opt}
                ref={(node) => { optionRefs.current[i] = node; }}
                role="option"
                aria-selected={selected}
                className={`score-picker-option${selected ? ' is-selected' : ''}`}
                onClick={() => { onChange(opt); close(); }}
              >
                {opt}
              </li>
            );
          })}
        </ul>,
        portalTarget,
      )
    : null;

  return (
    <div ref={rootRef} className={`score-picker${open ? ' score-picker--open' : ''}`}>
      <button
        ref={triggerRef}
        type="button"
        className={`score-picker-trigger${className ? ` ${className}` : ''}`}
        onClick={() => setOpen((v) => !v)}
        disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={ariaLabel ?? `Score: ${value}`}
      >
        <span className="score-picker-value">{value}</span>
        <ChevronDown size={14} strokeWidth={2.25} aria-hidden="true" />
      </button>
      {panel}
    </div>
  );
}

export default ScorePicker;
