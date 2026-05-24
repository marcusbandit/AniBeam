import { cloneElement, useCallback, useEffect, useRef, useState, type ReactElement, type Ref } from 'react';
import { createPortal } from 'react-dom';

// Hover-pause delay before the tooltip appears. ~600ms matches the native
// browser cadence closely enough that the swap-in feels like the same
// affordance, just in our design language.
const TOOLTIP_DELAY_MS = 600;

interface TooltipProps {
  /** The text shown when the cursor hovers the trigger for ~600ms. */
  label: string;
  /**
   * Single child element. Must accept a `ref` and the standard mouse/focus
   * event handlers (i.e. any native HTML element, or a forwardRef component
   * that exposes its root DOM node). Existing handlers on the child are
   * preserved — Tooltip composes its own on top.
   */
  children: ReactElement;
  /** Override the default 600ms delay. */
  delay?: number;
}

interface ChildEventProps {
  onMouseEnter?: (e: React.MouseEvent) => void;
  onMouseLeave?: (e: React.MouseEvent) => void;
  onFocus?: (e: React.FocusEvent) => void;
  onBlur?: (e: React.FocusEvent) => void;
  ref?: Ref<HTMLElement>;
}

/**
 * App-wide hover-pause tooltip. Replaces the native HTML `title=` attribute,
 * whose Chromium popup ignores our dark tokens entirely. The tooltip bubble
 * is rendered via `createPortal` to `document.body` so the trigger's
 * `overflow: hidden` or stacking context never clips it, and the chrome
 * matches the rest of the app (dark surface, mono font, soft border + shadow,
 * small caret pointing down at the trigger).
 *
 * Use the `tooltip` prop on `Card` for cards — Card embeds the same logic
 * directly for the hover-lift integration. For everything else (buttons,
 * chips, icons, truncated text), wrap with `<Tooltip label="...">`.
 */
export function Tooltip({ label, children, delay = TOOLTIP_DELAY_MS }: TooltipProps) {
  const targetRef = useRef<HTMLElement | null>(null);
  const tooltipTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [pos, setPos] = useState<{ x: number; y: number } | null>(null);

  useEffect(() => () => {
    if (tooltipTimer.current) clearTimeout(tooltipTimer.current);
  }, []);

  const show = useCallback(() => {
    const el = targetRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    setPos({ x: r.left + r.width / 2, y: r.top });
  }, []);

  const onEnter = useCallback(() => {
    if (tooltipTimer.current) clearTimeout(tooltipTimer.current);
    tooltipTimer.current = setTimeout(show, delay);
  }, [delay, show]);

  const onLeave = useCallback(() => {
    if (tooltipTimer.current) clearTimeout(tooltipTimer.current);
    tooltipTimer.current = null;
    setPos(null);
  }, []);

  const childProps = children.props as ChildEventProps;
  const originalRef = (children as ReactElement & { ref?: Ref<HTMLElement> }).ref;

  const attachRef = (node: HTMLElement | null) => {
    targetRef.current = node;
    if (typeof originalRef === 'function') originalRef(node);
    else if (originalRef && 'current' in originalRef) {
      (originalRef as { current: HTMLElement | null }).current = node;
    }
  };

  const cloned = cloneElement(children, {
    ref: attachRef,
    onMouseEnter: (e: React.MouseEvent) => { onEnter(); childProps.onMouseEnter?.(e); },
    onMouseLeave: (e: React.MouseEvent) => { onLeave(); childProps.onMouseLeave?.(e); },
    // Mirror to focus/blur so keyboard users see the tooltip too — matches
    // the standard accessibility pattern for hover-only affordances.
    onFocus: (e: React.FocusEvent) => { onEnter(); childProps.onFocus?.(e); },
    onBlur: (e: React.FocusEvent) => { onLeave(); childProps.onBlur?.(e); },
  } as ChildEventProps);

  return (
    <>
      {cloned}
      {pos && typeof document !== 'undefined' && createPortal(
        <div
          className="app-tooltip"
          role="tooltip"
          style={{ left: `${pos.x}px`, top: `${pos.y}px` }}
        >
          {label}
        </div>,
        document.body,
      )}
    </>
  );
}

export default Tooltip;
