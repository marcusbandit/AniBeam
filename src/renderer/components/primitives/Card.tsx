import {
  forwardRef,
  useEffect,
  useRef,
  useState,
  type ButtonHTMLAttributes,
  type HTMLAttributes,
  type ReactNode,
} from 'react';
import { createPortal } from 'react-dom';
import { smoothScalar, type SmoothHandle } from '../../utils/motion';

const LIFT_SPEED = 12;
const LIFT_AMOUNT_PX = 4;
// Hover-pause delay before the custom tooltip appears. ~600ms matches the
// native browser tooltip cadence closely enough that the swap-in feels like
// the same affordance, just in our design language.
const TOOLTIP_DELAY_MS = 600;

export type CardVariant = 'default' | 'internal' | 'external';

interface CardBaseProps {
  variant?: CardVariant;
  children: ReactNode;
  /** Disable hover lift (e.g., for cards that are purely informational). */
  noLift?: boolean;
  /**
   * Custom hover-pause tooltip text. Renders via portal (so it escapes the
   * card's `overflow: hidden`) in the app's design language instead of the
   * browser's native gray title bubble. When set, the underlying element's
   * `title` attribute is intentionally NOT used — pass the same text here.
   */
  tooltip?: string;
}
type CardAsButton = CardBaseProps & { onClick: () => void } & Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'onClick'>;
type CardAsDiv    = CardBaseProps & { onClick?: undefined } & HTMLAttributes<HTMLDivElement>;
type CardProps = CardAsButton | CardAsDiv;

/**
 * Card primitive. Owns:
 *  - inset edge-glow per variant (replaces the broken 3px ::before stripe).
 *  - hover lift via the shared smoothing engine.
 * Page-specific content (poster, body, badges) lives inside as children.
 */
const Card = forwardRef<HTMLElement, CardProps>(function Card(props, _ref) {
  const { variant = 'default', children, noLift, tooltip } = props;
  const elRef = useRef<HTMLElement | null>(null);
  const liftRef = useRef<SmoothHandle | null>(null);
  const tooltipTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [tooltipBox, setTooltipBox] = useState<{ x: number; y: number } | null>(null);

  useEffect(() => {
    if (noLift) return;
    const el = elRef.current;
    if (!el) return;
    const handle = smoothScalar(0, LIFT_SPEED, (v) => {
      // Skip the inline style entirely when essentially zero.
      el.style.transform = Math.abs(v) > 0.05 ? `translateY(${v.toFixed(2)}px)` : '';
    });
    liftRef.current = handle;
    return () => { handle.release(); el.style.transform = ''; };
  }, [noLift]);

  useEffect(() => () => {
    if (tooltipTimer.current) clearTimeout(tooltipTimer.current);
  }, []);

  const onEnter = () => {
    liftRef.current?.setTarget(-LIFT_AMOUNT_PX);
    if (!tooltip) return;
    if (tooltipTimer.current) clearTimeout(tooltipTimer.current);
    tooltipTimer.current = setTimeout(() => {
      const el = elRef.current;
      if (!el) return;
      const r = el.getBoundingClientRect();
      // Anchor centered above the card. The bubble's own CSS handles the
      // horizontal centering via transform: translateX(-50%).
      setTooltipBox({ x: r.left + r.width / 2, y: r.top });
    }, TOOLTIP_DELAY_MS);
  };
  const onLeave = () => {
    liftRef.current?.setTarget(0);
    if (tooltipTimer.current) clearTimeout(tooltipTimer.current);
    tooltipTimer.current = null;
    setTooltipBox(null);
  };

  const className = `card card--${variant}`;
  // Tooltip portal. Rendered into <body> so the card's overflow:hidden
  // doesn't clip it, and a high z-index keeps it above peers (timeline rail,
  // hovered siblings). `pointer-events: none` on the bubble prevents it from
  // interfering with mouseleave when the cursor drifts toward the label.
  const tooltipNode = tooltip && tooltipBox && typeof document !== 'undefined'
    ? createPortal(
        <div
          className="card-tooltip"
          role="tooltip"
          style={{ left: `${tooltipBox.x}px`, top: `${tooltipBox.y}px` }}
        >
          {tooltip}
        </div>,
        document.body,
      )
    : null;

  if (props.onClick) {
    const { onClick, variant: _v, children: _c, noLift: _n, tooltip: _t, className: callerClass, ...rest } = props as CardAsButton & { tooltip?: string };
    return (
      <>
        <button
          ref={(node) => { elRef.current = node; }}
          type="button"
          className={callerClass ? `${className} ${callerClass}` : className}
          onClick={onClick}
          onMouseEnter={onEnter}
          onMouseLeave={onLeave}
          {...rest}
        >
          {children}
        </button>
        {tooltipNode}
      </>
    );
  }
  const { onClick: _o, variant: _v, children: _c, noLift: _n, tooltip: _t, className: callerClass, ...rest } = props as CardAsDiv & { tooltip?: string };
  return (
    <>
      <div
        ref={(node) => { elRef.current = node; }}
        className={callerClass ? `${className} ${callerClass}` : className}
        onMouseEnter={onEnter}
        onMouseLeave={onLeave}
        {...rest}
      >
        {children}
      </div>
      {tooltipNode}
    </>
  );
});

export default Card;
