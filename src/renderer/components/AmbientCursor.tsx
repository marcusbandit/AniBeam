import { useEffect, useRef } from 'react';
import { smoothScalar } from '../utils/motion';

const POS_SPEED       = 9;
const SIZE_SPEED      = 6;
const INTENSITY_SPEED = 7;

const SIZE_IDLE     = 1200;   // px: wide, exploratory gaze
const SIZE_FOCUSED  = 540;    // px: tight, focused on a target

const INTENSITY_IDLE    = 0.45;   // faded at rest
const INTENSITY_FOCUSED = 1.0;    // full when something's hovered
const FADE_TIMEOUT_MS   = 1800;   // mouse-idle hold (off-target only) before intensity → 0
const HOLD_TIMEOUT_MS   = 350;    // hold focused state when between targets in a cluster

const BIAS_PX = 40;   // constant-magnitude pull toward a card's center

/**
 * Root-level pointer halo. Renders a single fixed-position layer behind
 * .main-content. The halo has three opt-in modes driven by data attributes:
 *
 *   (none)              : idle: massive radius, low intensity, follows the cursor.
 *   data-halo-snap      : focus: lock position to the element's center
 *                          (used by nav buttons / discrete targets).
 *   data-halo-bias      : focus: shift the halo BIAS_PX along the unit vector
 *                          from cursor toward the element's center. Halo
 *                          stays cursor-driven but is gently pulled in
 *                          (used by cards). If the cursor is already
 *                          within BIAS_PX of center, the halo locks to
 *                          center to avoid overshoot.
 *
 * A fourth attribute lets a container debounce the focus → idle transition:
 *
 *   data-halo-cluster   : when the cursor is inside this element but not
 *                          on any halo target, the focused size + intensity
 *                          are held for HOLD_TIMEOUT_MS before shrinking
 *                          back to idle. Prevents the "bounce" between
 *                          two cards in a grid.
 *
 * The mouse-idle fade-out (intensity → 0) is suppressed while the cursor
 * is on a halo target; the user explicitly asked: don't hide what they're
 * actively pointing at.
 *
 * All four properties (x, y, size, intensity) are smoothed by the shared
 * motion engine.
 */
export default function AmbientCursor() {
  const layerRef = useRef<HTMLDivElement | null>(null);
  const fadeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const holdTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const el = layerRef.current;
    if (!el) return;

    const writeX         = (v: number) => { el.style.setProperty('--ambient-x',         v.toFixed(2) + 'px'); };
    const writeY         = (v: number) => { el.style.setProperty('--ambient-y',         v.toFixed(2) + 'px'); };
    const writeSize      = (v: number) => { el.style.setProperty('--ambient-size',      v.toFixed(1) + 'px'); };
    const writeIntensity = (v: number) => { el.style.setProperty('--ambient-intensity', v.toFixed(3)); };

    const x         = smoothScalar(window.innerWidth  / 2, POS_SPEED,       writeX);
    const y         = smoothScalar(window.innerHeight / 2, POS_SPEED,       writeY);
    const size      = smoothScalar(SIZE_IDLE,              SIZE_SPEED,      writeSize);
    const intensity = smoothScalar(0,                      INTENSITY_SPEED, writeIntensity);

    const clearFade = () => {
      if (fadeTimerRef.current) { clearTimeout(fadeTimerRef.current); fadeTimerRef.current = null; }
    };
    const clearHold = () => {
      if (holdTimerRef.current) { clearTimeout(holdTimerRef.current); holdTimerRef.current = null; }
    };

    const scheduleFade = () => {
      clearFade();
      fadeTimerRef.current = setTimeout(() => intensity.setTarget(0), FADE_TIMEOUT_MS);
    };

    const onMove = (ev: MouseEvent) => {
      const target = ev.target as Element | null;
      const snapEl    = target?.closest?.('[data-halo-snap]')    as HTMLElement | null;
      const biasEl    = !snapEl ? (target?.closest?.('[data-halo-bias]')    as HTMLElement | null) : null;
      const clusterEl = (!snapEl && !biasEl)
        ? (target?.closest?.('[data-halo-cluster]') as HTMLElement | null)
        : null;

      if (snapEl) {
        const rect = snapEl.getBoundingClientRect();
        x.setTarget(rect.left + rect.width  / 2);
        y.setTarget(rect.top  + rect.height / 2);
        size.setTarget(SIZE_FOCUSED);
        intensity.setTarget(INTENSITY_FOCUSED);
        // While on a target: never auto-fade.
        clearFade();
        clearHold();
        return;
      }

      if (biasEl) {
        const rect = biasEl.getBoundingClientRect();
        const cx = rect.left + rect.width  / 2;
        const cy = rect.top  + rect.height / 2;
        const dx = cx - ev.clientX;
        const dy = cy - ev.clientY;
        const mag = Math.hypot(dx, dy);
        if (mag <= BIAS_PX) {
          x.setTarget(cx);
          y.setTarget(cy);
        } else {
          x.setTarget(ev.clientX + (dx / mag) * BIAS_PX);
          y.setTarget(ev.clientY + (dy / mag) * BIAS_PX);
        }
        size.setTarget(SIZE_FOCUSED);
        intensity.setTarget(INTENSITY_FOCUSED);
        // While on a target: never auto-fade.
        clearFade();
        clearHold();
        return;
      }

      // Off-target. Position tracks cursor directly.
      x.setTarget(ev.clientX);
      y.setTarget(ev.clientY);

      if (clusterEl) {
        // Inside a cluster: hold the focused size/intensity briefly to
        // avoid the "bounce to massive" between adjacent cards.
        size.setTarget(SIZE_FOCUSED);
        intensity.setTarget(INTENSITY_FOCUSED);
        clearHold();
        holdTimerRef.current = setTimeout(() => {
          size.setTarget(SIZE_IDLE);
          intensity.setTarget(INTENSITY_IDLE);
        }, HOLD_TIMEOUT_MS);
      } else {
        // Outside any cluster: go idle immediately.
        size.setTarget(SIZE_IDLE);
        intensity.setTarget(INTENSITY_IDLE);
        clearHold();
      }

      scheduleFade();
    };

    window.addEventListener('mousemove', onMove, { passive: true });
    return () => {
      window.removeEventListener('mousemove', onMove);
      clearFade();
      clearHold();
      x.release();
      y.release();
      size.release();
      intensity.release();
    };
  }, []);

  return <div ref={layerRef} className="ambient-cursor" aria-hidden="true" />;
}
