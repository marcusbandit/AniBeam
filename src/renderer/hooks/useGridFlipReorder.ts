import { useLayoutEffect, useRef, type RefObject } from "react";
import { smoothScalar, type SmoothHandle } from "../utils/motion";

// Matches the codebase animation preference: exponential smoothing via the
// shared rAF engine. SPEED ~18 settles a reorder in roughly 200ms — quick
// enough to feel like a direct response to the click, slow enough to read
// the motion.
const FLIP_SPEED = 18;

interface CardAnim {
  el: HTMLElement;
  xHandle: SmoothHandle;
  yHandle: SmoothHandle;
  x: number;
  y: number;
}

function prefersReducedMotion(): boolean {
  if (typeof window === "undefined" || !window.matchMedia) return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/**
 * FLIP-style reorder animation for a grid of keyed children.
 *
 * Each animated child must carry a stable `data-flip-id` attribute. On
 * every change to `orderKey`:
 *   1. Read the new bounding rects (the "Last" in FLIP — React has already
 *      committed the new layout by the time a useLayoutEffect runs).
 *   2. For each child present in the previous snapshot, compute
 *      prev−new and set the transform to that offset *instantly*, pinning
 *      the visual position at where it used to be.
 *   3. Ease the offset back to 0 via the shared exponential-smoothing
 *      engine. The element drifts to its real grid slot under animation.
 *
 * Brand-new and removed cards are ignored — only nodes present in both
 * snapshots animate.  Respects `prefers-reduced-motion`.
 */
export function useGridFlipReorder(
  containerRef: RefObject<HTMLElement | null>,
  orderKey: string,
): void {
  const prevRects = useRef<Map<string, DOMRect>>(new Map());
  const animations = useRef<Map<string, CardAnim>>(new Map());

  useLayoutEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const cards = container.querySelectorAll<HTMLElement>("[data-flip-id]");
    const next = new Map<string, DOMRect>();
    cards.forEach((el) => {
      const id = el.dataset.flipId;
      if (id) next.set(id, el.getBoundingClientRect());
    });

    if (prefersReducedMotion()) {
      // Skip the animation entirely, but still record positions so the
      // next non-reduced run has a baseline to FLIP from.
      prevRects.current = next;
      return;
    }

    cards.forEach((el) => {
      const id = el.dataset.flipId;
      if (!id) return;
      const prev = prevRects.current.get(id);
      const now = next.get(id);
      if (!prev || !now) return;
      const dx = prev.left - now.left;
      const dy = prev.top - now.top;
      if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5) return;

      // Re-arming a card that's mid-animation: stop the old smoothers
      // before creating new ones so we don't leak handles in the engine.
      const prior = animations.current.get(id);
      if (prior) {
        prior.xHandle.release();
        prior.yHandle.release();
      }

      const anim: CardAnim = {
        el,
        // Handles assigned in the next two lines — null! avoids requiring
        // the engine to expose a no-op handle for construction.
        xHandle: null as unknown as SmoothHandle,
        yHandle: null as unknown as SmoothHandle,
        x: dx,
        y: dy,
      };
      const writeTransform = () => {
        if (Math.abs(anim.x) < 0.1 && Math.abs(anim.y) < 0.1) {
          el.style.transform = "";
          el.style.willChange = "";
          anim.xHandle.release();
          anim.yHandle.release();
          animations.current.delete(id);
          return;
        }
        el.style.transform = `translate(${anim.x.toFixed(2)}px, ${anim.y.toFixed(2)}px)`;
      };
      anim.xHandle = smoothScalar(dx, FLIP_SPEED, (v) => { anim.x = v; writeTransform(); });
      anim.yHandle = smoothScalar(dy, FLIP_SPEED, (v) => { anim.y = v; writeTransform(); });
      animations.current.set(id, anim);

      // Pin at the old visual position before the first rAF tick — the
      // engine's first callback is one frame away, and without this the
      // card would jump to its new slot for one frame.
      el.style.willChange = "transform";
      el.style.transform = `translate(${dx.toFixed(2)}px, ${dy.toFixed(2)}px)`;

      // Start the glide.
      anim.xHandle.setTarget(0);
      anim.yHandle.setTarget(0);
    });

    prevRects.current = next;
  }, [orderKey, containerRef]);

  // Release any in-flight handles when the host unmounts.
  useLayoutEffect(() => {
    const inflight = animations.current;
    return () => {
      inflight.forEach((a) => { a.xHandle.release(); a.yHandle.release(); });
      inflight.clear();
    };
  }, []);
}
