import { useLayoutEffect, useRef } from "react";
import { useLocation } from "react-router-dom";
import { MAIN_SCROLL_ID } from "./navTrailCore";

/**
 * How many frames a restore is allowed to keep re-trying, roughly a second
 * at 60fps. Page content arrives asynchronously (metadata, posters, tracker
 * progress), so on the first paint the container is usually still far too
 * short to accept a deep offset. A budget rather than a fixed timeout means
 * a fast page lands on frame one and a slow page still gets its chance,
 * while a page that never grows back to size gives up instead of yanking
 * the view seconds after the user started reading.
 */
const RESTORE_FRAME_BUDGET = 60;

/**
 * `.main-content` carries `scroll-behavior: smooth` so in-page jumps glide.
 * A restore is not a jump the user asked for: it has to be already-there by
 * the time they see the page. Asking for an explicit instant behaviour beats
 * mutating the stylesheet, which would leak the non-smooth setting into
 * every other scroll on the page.
 */
function jumpTo(el: HTMLElement, top: number) {
  el.scrollTo({ top, behavior: "instant" });
}

/**
 * Owns the scroll position of the single `.main-content` container.
 *
 * That element is shared by every non-player route and survives route
 * changes, so without this the offset from the page you left is still
 * applied to the page you land on. Two cases, in order:
 *
 * 1. `location.state.restoreScroll` (set by `useNavTrail.goBack`) puts the
 *    user back exactly where they left off. Absent means "leave the scroll
 *    alone", which is NOT the same as restoring to 0: forward navigations
 *    and trails recorded before the offset existed both omit it.
 * 2. Otherwise a PATHNAME change starts the new page at the top. Pathname,
 *    not the whole location: the Library rewrites its search string on every
 *    keystroke (`?q=`, with `replace`), and jumping to the top mid-search
 *    would be unusable.
 */
export function useScrollRestore() {
  const location = useLocation();
  const lastPathname = useRef<string | null>(null);

  const raw = (location.state as { restoreScroll?: unknown } | null)?.restoreScroll;
  const target = typeof raw === "number" && Number.isFinite(raw) ? raw : null;

  // Layout effect, not a plain effect: when the container is already tall
  // enough (a short page, or a return to a page still in memory) the restore
  // completes before paint, so the user never sees a frame at the old offset.
  useLayoutEffect(() => {
    const el = document.getElementById(MAIN_SCROLL_ID);
    const pathChanged = lastPathname.current !== location.pathname;
    lastPathname.current = location.pathname;

    // The player route renders no main content, so there is nothing to own.
    if (!el) return;

    if (target === null) {
      if (pathChanged) jumpTo(el, 0);
      return;
    }

    let frames = 0;
    let raf = 0;
    let stopped = false;

    // The user outranks the restore: the moment they scroll for themselves,
    // any further attempt would be pulling the page out from under them.
    // Listening for input rather than for `scroll` events, since our own
    // `scrollTo` fires those too.
    const stop = () => {
      stopped = true;
    };
    el.addEventListener("wheel", stop, { passive: true });
    el.addEventListener("touchmove", stop, { passive: true });
    el.addEventListener("pointerdown", stop, { passive: true });
    window.addEventListener("keydown", stop);

    /** True once the offset has landed, or once it never can. */
    const attempt = () => {
      jumpTo(el, target);
      const fits = el.scrollHeight - el.clientHeight >= target;
      return fits || Math.abs(el.scrollTop - target) < 1;
    };

    if (!attempt()) {
      const tick = () => {
        if (stopped) return;
        if (attempt() || ++frames >= RESTORE_FRAME_BUDGET) return;
        raf = requestAnimationFrame(tick);
      };
      raf = requestAnimationFrame(tick);
    }

    // Cleanup covers both unmount and the next navigation, so a pending
    // retry loop can never survive to fight the restore that replaced it.
    return () => {
      stopped = true;
      if (raf) cancelAnimationFrame(raf);
      el.removeEventListener("wheel", stop);
      el.removeEventListener("touchmove", stop);
      el.removeEventListener("pointerdown", stop);
      window.removeEventListener("keydown", stop);
    };
  }, [location.key, location.pathname, target]);
}
