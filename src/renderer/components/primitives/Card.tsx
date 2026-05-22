import {
  forwardRef,
  useEffect,
  useRef,
  type ButtonHTMLAttributes,
  type HTMLAttributes,
  type ReactNode,
} from 'react';
import { smoothScalar, type SmoothHandle } from '../../utils/motion';

const LIFT_SPEED = 12;
const LIFT_AMOUNT_PX = 4;

export type CardVariant = 'default' | 'internal' | 'external';

interface CardBaseProps {
  variant?: CardVariant;
  children: ReactNode;
  /** Disable hover lift (e.g., for cards that are purely informational). */
  noLift?: boolean;
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
  const { variant = 'default', children, noLift } = props;
  const elRef = useRef<HTMLElement | null>(null);
  const liftRef = useRef<SmoothHandle | null>(null);

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

  const onEnter = () => liftRef.current?.setTarget(-LIFT_AMOUNT_PX);
  const onLeave = () => liftRef.current?.setTarget(0);

  const className = `card card--${variant}`;

  if (props.onClick) {
    const { onClick, variant: _v, children: _c, noLift: _n, ...rest } = props as CardAsButton;
    return (
      <button
        ref={(node) => { elRef.current = node; }}
        type="button"
        className={className}
        onClick={onClick}
        onMouseEnter={onEnter}
        onMouseLeave={onLeave}
        {...rest}
      >
        {children}
      </button>
    );
  }
  const { onClick: _o, variant: _v, children: _c, noLift: _n, ...rest } = props as CardAsDiv;
  return (
    <div
      ref={(node) => { elRef.current = node; }}
      className={className}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      {...rest}
    >
      {children}
    </div>
  );
});

export default Card;
