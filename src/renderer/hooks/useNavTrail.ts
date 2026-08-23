import { useCallback, useMemo } from "react";
import { useLocation, useNavigate } from "react-router-dom";

/**
 * One step up the browsing tree: where the user came from, and what to
 * call it on the Back button.
 */
export interface TrailEntry {
  /** Full route including search, so a query/tab state survives the trip back. */
  path: string;
  /** Button text: "Feed", "Library", a series title, … */
  label: string;
}

/**
 * The trail rides in react-router location state rather than being derived
 * from browser history, because history is unreliable here: the player
 * pushes forward navigations (next episode), the franchise graph can loop
 * back onto a series already visited, and a reload wipes the stack. Keeping
 * an explicit ancestor list means Back always walks the tree the user
 * actually browsed - Feed → series → related series → player unwinds one
 * level per press. Capped so a long franchise crawl can't grow state without
 * bound.
 */
const MAX_TRAIL = 12;

const ROUTE_LABELS: ReadonlyArray<[RegExp, string]> = [
  [/^\/$/, "Library"],
  [/^\/feed/, "Feed"],
  [/^\/watching/, "Watching"],
  [/^\/subscriptions/, "Subscriptions"],
  [/^\/settings/, "Settings"],
  [/^\/metadata/, "Metadata"],
  [/^\/series\//, "Series"],
  [/^\/player\//, "Player"],
];

/** Generic name for a route, used when the caller has nothing better. */
export function labelForPath(pathname: string): string {
  for (const [re, label] of ROUTE_LABELS) {
    if (re.test(pathname)) return label;
  }
  return "Back";
}

/** Defensive read: state comes from history and may be anything. */
export function readTrail(state: unknown): TrailEntry[] {
  const raw = (state as { trail?: unknown } | null | undefined)?.trail;
  if (!Array.isArray(raw)) return [];
  return raw.filter(
    (e): e is TrailEntry =>
      !!e &&
      typeof (e as TrailEntry).path === "string" &&
      typeof (e as TrailEntry).label === "string",
  );
}

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
   */
  const descend = useCallback(
    (extra?: Record<string, unknown>) => ({
      ...extra,
      trail: [
        ...trail.filter((e) => e.path !== here),
        { path: here, label: hereLabel },
      ].slice(-MAX_TRAIL),
    }),
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
      navigate(target.path, { state: rest.length ? { trail: rest } : undefined });
    },
    [trail, navigate],
  );

  return { trail, descend, keep, backTarget, goBack };
}
