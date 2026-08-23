import { useCallback, useMemo } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { MAIN_SCROLL_ID, MAX_TRAIL, labelForPath, readTrail } from "./navTrailCore";
import type { TrailEntry } from "./navTrailCore";

// The pure half (route labels, defensive state reading, the rail's tab list)
// lives in ./navTrailCore so it can be tested without React. Re-exported here
// so the existing import sites keep pointing at the hook.
export { labelForPath, readTrail };
export type { TrailEntry };

/**
 * Browsing-tree navigation.
 *
 * @param currentLabel What to call THIS page on a deeper page's Back button
 *                     (a series page passes its title). Defaults to the
 *                     route's generic name.
 */
export function useNavTrail(currentLabel?: string) {
  const location = useLocation();
  const navigate = useNavigate();

  const trail = useMemo(() => readTrail(location.state), [location.state]);
  const here = location.pathname + location.search;
  const hereLabel = currentLabel ?? labelForPath(location.pathname);

  /**
   * Router state for a navigation one level DEEPER (library → series,
   * series → player): appends this page to the trail. Extra state (a query,
   * a skipResume flag) can ride along.
   *
   * The scroll offset rides in the trail entry for the same reason the trail
   * itself does: browser history can't be trusted to hand it back. A forward
   * push from the player, a loop back onto a series already visited, or a
   * reload all leave the native scroll restoration pointing at the wrong
   * entry (or at nothing). Reading `scrollTop` at navigation time and
   * carrying it with the ancestor means Back restores the exact view the
   * user left, however they got back to it.
   */
  const descend = useCallback(
    (extra?: Record<string, unknown>) => {
      // Absent on routes with no rail or main content (the player), in which
      // case there is no offset to remember.
      const scroll = document.getElementById(MAIN_SCROLL_ID)?.scrollTop ?? 0;
      return {
        ...extra,
        trail: [
          ...trail.filter((e) => e.path !== here),
          { path: here, label: hereLabel, scroll },
        ].slice(-MAX_TRAIL),
      };
    },
    [trail, here, hereLabel],
  );

  /**
   * Router state for a SIDEWAYS navigation (episode → next episode): keeps
   * the trail untouched so Back still exits one level, not one episode.
   */
  const keep = useCallback(
    (extra?: Record<string, unknown>) =>
      trail.length ? { ...extra, trail } : extra,
    [trail],
  );

  /** Where Back goes, or null when we arrived cold (deep link / reload). */
  const backTarget = trail.length ? trail[trail.length - 1] : null;

  /** Up one level; `fallback` covers the cold-arrival case. */
  const goBack = useCallback(
    (fallback = "/") => {
      const target = trail[trail.length - 1];
      if (!target) {
        navigate(fallback);
        return;
      }
      const rest = trail.slice(0, -1);
      // `restoreScroll` is undefined for entries recorded before the offset
      // existed, or for a page that had no scroll container; App.tsx treats
      // that as "leave the scroll alone".
      navigate(target.path, {
        state: {
          ...(rest.length ? { trail: rest } : {}),
          restoreScroll: target.scroll,
        },
      });
    },
    [trail, navigate],
  );

  return { trail, descend, keep, backTarget, goBack };
}
