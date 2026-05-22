/**
 * Exponential smoothing engine.
 *
 * Subscribers register a target/current pair via smoothScalar(); a single
 * module-level requestAnimationFrame loop ticks all of them with
 * frame-rate-independent smoothing:
 *
 *   current += (target - current) * (1 - exp(-speed * dt))
 *
 * dt is clamped to 0.05s so that tab-defocused/long-paused frames don't
 * teleport. The loop starts on the first subscription and stops when the
 * last subscriber releases.
 */

export interface SmoothHandle {
  /** Move the target. Engine animates current toward it. */
  setTarget(value: number): void;
  /** Read the current smoothed value without recomputing. */
  current(): number;
  /** Unsubscribe. */
  release(): void;
}

interface Subscriber {
  target: number;
  current: number;
  speed: number;
  onChange: (v: number) => void;
}

const subscribers = new Set<Subscriber>();
let rafId: number | null = null;
let lastTime = 0;

function tick(now: number) {
  const dt = Math.min(0.05, lastTime ? (now - lastTime) / 1000 : 0.016);
  lastTime = now;

  subscribers.forEach((s) => {
    const k = 1 - Math.exp(-s.speed * dt);
    s.current += (s.target - s.current) * k;
    if (Math.abs(s.current - s.target) < 0.02) s.current = s.target;
    s.onChange(s.current);
  });

  if (subscribers.size > 0) {
    rafId = requestAnimationFrame(tick);
  } else {
    rafId = null;
    lastTime = 0;
  }
}

function ensureRunning() {
  if (rafId === null) {
    lastTime = 0;
    rafId = requestAnimationFrame(tick);
  }
}

/**
 * Subscribe a scalar to the engine.
 * onChange is called every frame with the new smoothed value.
 */
/**
 * Returns true if the user has asked the OS to reduce motion. Re-evaluated
 * per call (cheap and the user can change it at runtime in some OSes).
 */
function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined' || !window.matchMedia) return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export function smoothScalar(
  initial: number,
  speed: number,
  onChange: (v: number) => void,
): SmoothHandle {
  const sub: Subscriber = { target: initial, current: initial, speed, onChange };
  // Reduced-motion mode: skip the rAF loop entirely. setTarget snaps
  // current → target and fires onChange synchronously. The engine still
  // honors release() so cleanup paths stay symmetric.
  if (prefersReducedMotion()) {
    onChange(initial);
    return {
      setTarget(value) {
        sub.target = value;
        sub.current = value;
        onChange(value);
      },
      current() { return sub.current; },
      release() {},
    };
  }
  subscribers.add(sub);
  ensureRunning();
  return {
    setTarget(value) { sub.target = value; },
    current() { return sub.current; },
    release() { subscribers.delete(sub); },
  };
}

/**
 * Pure smoothing step. Exposed for testing and for callers who manage their
 * own RAF loop (e.g., AmbientCursor coupling halo + per-card draws).
 */
export function smoothStep(current: number, target: number, speed: number, dt: number): number {
  const clampedDt = Math.min(0.05, Math.max(0, dt));
  const k = 1 - Math.exp(-speed * clampedDt);
  const next = current + (target - current) * k;
  return Math.abs(next - target) < 0.02 ? target : next;
}
