/**
 * The React-free half of the nav trail: route naming, defensive state
 * reading, and the rail's own list of destinations. Lives apart from
 * `useNavTrail` so it can be exercised by `scripts/verify-nav-trail.mjs`
 * without a renderer, a router, or a DOM.
 */

/**
 * One step up the browsing tree: where the user came from, and what to
 * call it on the Back button.
 */
export interface TrailEntry {
  /** Full route including search, so a query/tab state survives the trip back. */
  path: string;
  /** Button text: "Feed", "Library", a series title, … */
  label: string;
  /**
   * Scroll offset of the main content area when the user left this page,
   * so Back lands them where they were instead of at the top of a long
   * grid. Optional: entries written before this existed, and pages with no
   * scroll container (the player), simply omit it.
   */
  scroll?: number;
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
export const MAX_TRAIL = 12;

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
  const out: TrailEntry[] = [];
  for (const e of raw) {
    if (!e) continue;
    const { path, label, scroll } = e as Partial<TrailEntry>;
    if (typeof path !== "string" || typeof label !== "string") continue;
    // A junk offset is dropped rather than carried: an entry with a bad
    // scroll is still a perfectly good Back target.
    out.push(
      typeof scroll === "number" && Number.isFinite(scroll)
        ? { path, label, scroll }
        : { path, label },
    );
  }
  return out;
}

/**
 * Id of the single `.main-content` scroll container. App.tsx puts this id on
 * the element; the trail reads `scrollTop` off it when descending, and
 * restores it when Back lands.
 */
export const MAIN_SCROLL_ID = "main-scroll";

/** A top-level destination in the left rail. */
export interface RailTab {
  /** Route the rail link navigates to, and the identity `activeTab` returns. */
  path: string;
  /** Rail link text. */
  label: string;
}

/**
 * The rail's destinations, in rail order. Single source of truth: App.tsx
 * renders the rail from this list (mapping path → icon on its side, since
 * icons are React components and this file stays React-free), and
 * `activeTab` decides highlighting from the same data. A new tab needs an
 * entry here and an icon in App.tsx's RAIL_ICONS, and nothing else: no
 * per-route branching anywhere.
 */
export const RAIL_TABS: ReadonlyArray<RailTab> = [
  { path: "/", label: "Library" },
  { path: "/feed", label: "Feed" },
  { path: "/watching", label: "Watching" },
  { path: "/metadata", label: "Metadata" },
  { path: "/settings", label: "Settings" },
];

/** Strip search/hash so a stored `path` compares against a bare pathname. */
function pathnameOf(path: string): string {
  return path.split(/[?#]/, 1)[0];
}

/**
 * Which rail tab should light up for the current view.
 *
 * A series page belongs to whatever tab the user browsed in from, so the
 * rail keeps showing where they are in the app rather than going blank the
 * moment they open a show. That context is exactly what the trail already
 * carries, so we read the tab off the ROOT of the trail (the first entry
 * that is a rail tab), not off the pathname.
 *
 * Falls back to Library, which is also the right answer for a cold deep
 * link or a reload with no state.
 */
export function activeTab(pathname: string, trail: TrailEntry[]): string {
  const here = pathnameOf(pathname);
  const isTab = (p: string) => RAIL_TABS.some((t) => t.path === p);

  if (isTab(here)) return here;
  // The one route with no rail entry of its own: it is only reachable from
  // Settings, so it keeps Settings lit.
  if (here === "/subscriptions") return "/settings";

  for (const entry of trail) {
    const p = pathnameOf(entry.path);
    if (isTab(p)) return p;
  }
  return "/";
}

/**
 * The durable part of router state, for a page rewriting its own URL in
 * place (the Library parking its search text in `?q=`, say).
 *
 * The trail is durable: it says where the user came from, and a same-page
 * rewrite must not drop it. `restoreScroll` is the opposite, a one-shot
 * instruction aimed at the single navigation that carried it, so it is
 * deliberately left behind. Forwarding it would re-arm the scroll restore on
 * every keystroke, snapping the view back to wherever Back had landed.
 */
export function carryState(state: unknown): { trail: TrailEntry[] } | undefined {
  const trail = readTrail(state);
  return trail.length ? { trail } : undefined;
}
