import { useEffect, useMemo, useState, useCallback, useRef } from "react";
import {
  Tv,
  ChevronLeft,
  ChevronRight,
  ArrowDown,
  ArrowUp,
  Eye,
  PieChart,
  Star,
  CaseSensitive,
} from "lucide-react";
import type { LibraryItem } from "../../types/electron";
import { findNextUpcomingEpisode, normalizeStatus } from "../utils/airingUtils";
import { useDebouncedCallback } from "../hooks/useDebouncedCallback";
import { useGridFlipReorder } from "../hooks/useGridFlipReorder";
import { useTitleLanguage } from "../contexts/TitleLanguageContext";
import { useTrackerProgress } from "../contexts/TrackerProgressContext";
import { useViewHistory } from "../contexts/ViewHistoryContext";
import ShowCard from "../components/ShowCard";
import { Page, Section, SegmentedSwitch, Tooltip, type SegmentedOption } from "../components/primitives";

const AIRING_PAGE_COLS = 5;
const AIRING_PAGE_ROWS = 2;
const AIRING_PAGE_SIZE = AIRING_PAGE_COLS * AIRING_PAGE_ROWS;

type LibraryTab = "all" | "series" | "movies";
type SortKey = "alpha" | "lastViewed" | "progress" | "score" | "myScore";
type SortDir = "asc" | "desc";

const LS_TAB = "anibeam.libraryTab";
const LS_SORT_KEY = "anibeam.librarySortKey";
const LS_SORT_DIR = "anibeam.librarySortDir";

// Each sort key has a "natural" direction so the first time the user picks
// one they see the result they'd expect (newest first for recency, highest
// score first, A→Z for alphabetic). The direction toggle then flips it.
const NATURAL_DIR: Record<SortKey, SortDir> = {
  alpha: "asc",
  lastViewed: "desc",
  progress: "desc",
  score: "desc",
  myScore: "desc",
};

const TAB_OPTIONS: SegmentedOption<LibraryTab>[] = [
  { value: "all", label: "All" },
  { value: "series", label: "Series" },
  { value: "movies", label: "Movies" },
];

// Icon glyphs only — the meaning lives in the tooltip (ariaLabel) so each
// button stays compact and the row of switches doesn't crowd the header.
// Star tones are tied to the rating pills on ShowCard so the same colour
// always means the same thing across the app: amber = community, teal =
// you.
const SORT_OPTIONS: SegmentedOption<SortKey>[] = [
  {
    value: "alpha",
    label: <CaseSensitive size={16} aria-hidden="true" />,
    ariaLabel: "Alphabetic",
  },
  {
    value: "lastViewed",
    label: <Eye size={16} aria-hidden="true" />,
    ariaLabel: "Last viewed",
  },
  {
    value: "progress",
    label: <PieChart size={16} aria-hidden="true" />,
    ariaLabel: "Progress",
  },
  {
    value: "score",
    label: (
      <Star
        size={16}
        aria-hidden="true"
        fill="var(--accent-amber)"
        stroke="var(--accent-amber)"
      />
    ),
    ariaLabel: "Community score",
  },
  {
    value: "myScore",
    label: (
      <Star
        size={16}
        aria-hidden="true"
        fill="var(--accent-teal)"
        stroke="var(--accent-teal)"
      />
    ),
    ariaLabel: "My score",
  },
];

function readStored<T extends string>(key: string, allowed: readonly T[], fallback: T): T {
  if (typeof window === "undefined") return fallback;
  const raw = window.localStorage.getItem(key);
  return raw && (allowed as readonly string[]).includes(raw) ? (raw as T) : fallback;
}

function fmtRelativeTime(unixSec: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - unixSec;
  const abs = Math.abs(diff);
  const future = diff < 0;
  if (abs < 60) return "just now";
  if (abs < 3600) {
    const m = Math.floor(abs / 60);
    return future ? `in ${m}m` : `${m}m ago`;
  }
  if (abs < 86400) {
    const h = Math.floor(abs / 3600);
    return future ? `in ${h}h` : `${h}h ago`;
  }
  if (abs < 86400 * 30) {
    const d = Math.floor(abs / 86400);
    return future ? `in ${d}d` : `${d}d ago`;
  }
  const mo = Math.floor(abs / (86400 * 30));
  return future ? `in ${mo}mo` : `${mo}mo ago`;
}

// (sortKey, latestEpisode) for an airing show. Mirrors FeedPage's logic:
// latest aired-and-on-disk episode → fall back to highest file mtime.
function getAiringSortInfo(item: LibraryItem): { when: number; episode: number | null } | null {
  const nowSec = Math.floor(Date.now() / 1000);
  const onDiskEps = new Set(item.files.map((f) => f.episodeNumber));
  let bestAired: { ts: number; ep: number } | null = null;
  for (const e of item.episodes) {
    if (!e.airDate || !onDiskEps.has(e.episodeNumber)) continue;
    const t = Math.floor(Date.parse(e.airDate) / 1000);
    if (!Number.isFinite(t) || t > nowSec) continue;
    if (!bestAired || t > bestAired.ts) bestAired = { ts: t, ep: e.episodeNumber };
  }
  if (bestAired) return { when: bestAired.ts, episode: bestAired.ep };
  let bestFile: { mtime: number; ep: number } | null = null;
  for (const f of item.files) {
    if (!f.mtime) continue;
    if (!bestFile || f.mtime > bestFile.mtime) bestFile = { mtime: f.mtime, ep: f.episodeNumber };
  }
  if (bestFile) return { when: Math.floor(bestFile.mtime / 1000), episode: bestFile.ep };
  return null;
}

// Normalise raw averageScore to a 0–10 scale. AniList serves 0–100, MAL
// 0–10. Anything else (missing source, missing score) becomes null so the
// sort can push it to the end regardless of direction.
function normalisedScore(item: LibraryItem): number | null {
  if (item.averageScore == null) return null;
  if (item.source === "anilist") return item.averageScore / 10;
  return item.averageScore;
}

function HomePage() {
  const [items, setItems] = useState<LibraryItem[]>([]);
  const [initialLoading, setInitialLoading] = useState(true);
  const [airingPage, setAiringPage] = useState(0);
  const { pickTitle } = useTitleLanguage();
  const { getWatched, getUserScore, getListStatus } = useTrackerProgress();
  const { getLastViewed } = useViewHistory();

  const [tab, setTab] = useState<LibraryTab>(() =>
    readStored<LibraryTab>(LS_TAB, ["all", "series", "movies"], "all"),
  );
  const [sortKey, setSortKey] = useState<SortKey>(() =>
    readStored<SortKey>(LS_SORT_KEY, ["alpha", "lastViewed", "progress", "score", "myScore"], "alpha"),
  );
  const [sortDir, setSortDir] = useState<SortDir>(() =>
    readStored<SortDir>(LS_SORT_DIR, ["asc", "desc"], NATURAL_DIR.alpha),
  );

  // Ref to the .show-grid wrapper so the FLIP reorder hook can read each
  // card's bounding rect before and after a sort change. The hook only
  // touches `[data-flip-id]` children, so unrelated DOM in the section
  // header doesn't interfere.
  const gridRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => { window.localStorage.setItem(LS_TAB, tab); }, [tab]);
  useEffect(() => { window.localStorage.setItem(LS_SORT_KEY, sortKey); }, [sortKey]);
  useEffect(() => { window.localStorage.setItem(LS_SORT_DIR, sortDir); }, [sortDir]);

  // Reloads triggered by metadata pings update items in place — no
  // setLoading(true), so the page doesn't flash through a "Reading
  // folders…" state on every poster match. Only the first mount shows it.
  // Diff-merge by id so React keeps stable refs for cards that didn't
  // change — prevents poster <img> elements from being unnecessarily
  // re-evaluated on every ping during the match burst at startup.
  const reload = useCallback(async () => {
    try {
      const data = await window.electronAPI.libraryWalk();
      const fresh = Array.isArray(data) ? data : [];
      setItems((prev) => {
        const prevById = new Map(prev.map((p) => [p.id, p]));
        return fresh.map((next) => {
          const old = prevById.get(next.id);
          if (!old) return next;
          // Cheap equality on the few fields the card actually renders.
          if (
            old.posterLocal === next.posterLocal &&
            old.poster === next.poster &&
            old.folderName === next.folderName &&
            old.matchedTitle === next.matchedTitle &&
            old.status === next.status &&
            old.files.length === next.files.length
          ) {
            return old;
          }
          return next;
        });
      });
    } catch (err) {
      console.error("library:walk failed", err);
      setItems([]);
    } finally {
      setInitialLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  // Debounce the reload — adding a new show fires ~2N+1
  // metadata:file-status-changed events (per-file ingest, per-file probe
  // completion, plus the poster-match landing). Without this, each event
  // triggers a separate library:walk + setItems and the grid flickers
  // through several intermediate states. 250ms covers the burst comfortably.
  const debouncedReload = useDebouncedCallback(() => { void reload(); }, 250);
  useEffect(() => {
    const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
      debouncedReload();
    });
    return () => unsubscribe?.();
  }, [debouncedReload]);

  // Currently-airing shows the user has on disk, sorted by latest aired
  // (or downloaded) episode. Empty array if nothing is airing yet.
  const airing = useMemo(() => {
    const out: Array<{ item: LibraryItem; when: number; episode: number | null }> = [];
    for (const item of items) {
      if (item.files.length === 0) continue;
      if (normalizeStatus(item.status) !== "releasing") continue;
      const info = getAiringSortInfo(item);
      if (!info) continue;
      out.push({ item, when: info.when, episode: info.episode });
    }
    out.sort((a, b) => b.when - a.when);
    return out;
  }, [items]);

  // Split the library into series and movies. The tab switch chooses which
  // collection is shown; "all" merges both. Sort key + direction are
  // shared across all three tabs.
  const seriesItems = useMemo(() => items.filter((i) => i.type !== "movie"), [items]);
  const movieItems = useMemo(() => items.filter((i) => i.type === "movie"), [items]);

  // Active list driven by the tab switch.
  const activeItems =
    tab === "series" ? seriesItems :
    tab === "movies" ? movieItems :
    items;

  // True when the user has finished the show — either marked completed on
  // the tracker, or watched count has reached the (known) total. Used by
  // the Progress sort to always pin completed shows to the bottom of the
  // list regardless of direction, so they don't pollute the "what should I
  // pick up next?" view.
  const isWatchedThrough = useCallback((i: LibraryItem): boolean => {
    const ids = { anilistId: i.anilistId ?? undefined, malId: i.malId };
    if (getListStatus(ids) === "completed") return true;
    const watched = getWatched(ids);
    const total = i.totalEpisodes;
    if (watched == null) return false;
    if (total == null) return watched > 0;  // movies: any watched = done
    if (total <= 0) return false;
    return watched >= total;
  }, [getListStatus, getWatched]);

  // Comparator factory keyed by sort + direction. Items missing a value
  // (no score, no progress, no view history) sort to the end so the
  // direction toggle never strands them at the top.
  const sortedItems = useMemo(() => {
    const dirMul = sortDir === "asc" ? 1 : -1;

    const titleOf = (i: LibraryItem) =>
      pickTitle({
        titleRomaji: i.titleRomaji,
        titleEnglish: i.titleEnglish,
        folderName: i.folderName,
      }).toLocaleLowerCase();

    const valueOf = (i: LibraryItem): number | null => {
      switch (sortKey) {
        case "lastViewed":
          return getLastViewed(i.id);
        case "progress": {
          const watched = getWatched({ anilistId: i.anilistId ?? undefined, malId: i.malId });
          const total = i.totalEpisodes;
          if (total == null) return watched != null && watched > 0 ? 1 : 0;
          if (total <= 0) return null;
          return Math.max(0, Math.min(1, (watched ?? 0) / total));
        }
        case "score":
          return normalisedScore(i);
        case "myScore":
          return getUserScore({ anilistId: i.anilistId ?? undefined, malId: i.malId });
        default:
          return 0;
      }
    };

    // Stable copy → sort. Don't mutate the memoised array from useMemo above.
    const copy = activeItems.slice();
    copy.sort((a, b) => {
      // Watched-through pinning for Progress sort — completed shows always
      // sink to the bottom regardless of direction (the user's rule:
      // "Progress should not include Watched"). Within the watched-through
      // tier we still sort A→Z so it's not arbitrary.
      if (sortKey === "progress") {
        const wa = isWatchedThrough(a);
        const wb = isWatchedThrough(b);
        if (wa !== wb) return wa ? 1 : -1;
        if (wa && wb) return titleOf(a).localeCompare(titleOf(b));
      }
      if (sortKey === "alpha") {
        return titleOf(a).localeCompare(titleOf(b)) * dirMul;
      }
      const va = valueOf(a);
      const vb = valueOf(b);
      if (va == null && vb == null) {
        return titleOf(a).localeCompare(titleOf(b));
      }
      if (va == null) return 1;   // missing → end, regardless of dir
      if (vb == null) return -1;
      if (va === vb) {
        return titleOf(a).localeCompare(titleOf(b)); // tie-breaker: A→Z
      }
      return (va - vb) * dirMul;
    });
    return copy;
  }, [activeItems, sortKey, sortDir, pickTitle, getWatched, getUserScore, getLastViewed, isWatchedThrough]);

  // Animate card positions when the sort order (or active tab) changes.
  // The key is just the ordered id list — any actual reorder produces a
  // different string and re-runs the FLIP. Item *content* changes
  // (poster matching, watched counts) don't change the key, so they
  // don't trigger an animation.
  const orderKey = useMemo(() => sortedItems.map((i) => i.id).join("|"), [sortedItems]);
  useGridFlipReorder(gridRef, orderKey);

  const airingTotalPages = Math.max(1, Math.ceil(airing.length / AIRING_PAGE_SIZE));
  // Reset page if the airing list shrinks below the current page.
  useEffect(() => {
    if (airingPage >= airingTotalPages) setAiringPage(0);
  }, [airingPage, airingTotalPages]);

  const airingPageItems = airing.slice(
    airingPage * AIRING_PAGE_SIZE,
    (airingPage + 1) * AIRING_PAGE_SIZE,
  );

  // Shared 30s tick driving the next-episode countdown on cards that have
  // a known upcoming air date. Only mounted when at least one item is
  // actually airing — finished libraries don't keep a timer alive.
  const [nowMs, setNowMs] = useState<number>(() => Date.now());
  const hasAnyUpcoming = useMemo(
    () => items.some((item) => findNextUpcomingEpisode(item.episodes, Date.now()) != null),
    [items],
  );
  useEffect(() => {
    if (!hasAnyUpcoming) return;
    const id = window.setInterval(() => setNowMs(Date.now()), 30_000);
    return () => window.clearInterval(id);
  }, [hasAnyUpcoming]);

  if (initialLoading) {
    return (
      <Page>
        <div className="loading">Reading folders…</div>
      </Page>
    );
  }

  const renderCard = (item: LibraryItem, sub?: { episode: number | null; when: number }) => (
    <ShowCard
      key={item.id}
      item={item}
      episodeBadgeNumber={sub?.episode ?? null}
      metaLeftText={sub ? fmtRelativeTime(sub.when) : undefined}
      nowMs={nowMs}
    />
  );

  // Switching sort key resets direction to that key's natural default so
  // first-touch lands on the expected order (newest first, A→Z, etc.).
  // The user can immediately flip it if they want the opposite.
  const onSortKeyChange = (next: SortKey) => {
    setSortKey(next);
    setSortDir(NATURAL_DIR[next]);
  };

  const hasLibrary = items.length > 0;
  const showTabs = seriesItems.length > 0 || movieItems.length > 0;

  return (
    <Page
      head={
        <div>
          <h1 className="page-title">Library</h1>
          <p className="page-sub">
            {items.length === 0
              ? "Your scanned folders are empty."
              : `${items.length} folder${items.length === 1 ? "" : "s"}.`}
          </p>
        </div>
      }
    >
      {airing.length > 0 && (
        <Section
          first
          title="Airing"
          action={
            <div className="airing-pager">
              <button
                type="button"
                className="airing-pager-btn"
                onClick={() => setAiringPage((p) => Math.max(0, p - 1))}
                disabled={airingPage === 0}
                aria-label="Previous page"
              >
                <ChevronLeft size={16} />
              </button>
              <span className="airing-pager-count">
                {airingPage + 1} / {airingTotalPages}
              </span>
              <button
                type="button"
                className="airing-pager-btn"
                onClick={() => setAiringPage((p) => Math.min(airingTotalPages - 1, p + 1))}
                disabled={airingPage >= airingTotalPages - 1}
                aria-label="Next page"
              >
                <ChevronRight size={16} />
              </button>
            </div>
          }
        >
          <div className="airing-grid" data-halo-cluster>
            {airingPageItems.map(({ item, when, episode }) =>
              renderCard(item, { episode, when }),
            )}
          </div>
        </Section>
      )}

      {!hasLibrary ? (
        <div className="empty">
          <div className="empty-icon"><Tv size={48} /></div>
          <div className="empty-title">Your library is empty</div>
          <div className="empty-text">
            Go to <strong>Settings</strong> to add a folder.
          </div>
        </div>
      ) : showTabs ? (
        <section className={`section--primitive${airing.length === 0 ? " section--first" : ""}`}>
          <header className="library-tabs-head">
            <div className="library-tabs-head__left">
              <SegmentedSwitch<LibraryTab>
                value={tab}
                options={TAB_OPTIONS}
                onChange={setTab}
                ariaLabel="Library category"
              />
              <span className="library-tabs-head__count">{activeItems.length}</span>
            </div>
            <div className="library-tabs-head__right">
              <span className="library-tabs-head__sort-label">Sort</span>
              <SegmentedSwitch<SortKey>
                value={sortKey}
                options={SORT_OPTIONS}
                onChange={onSortKeyChange}
                ariaLabel="Sort by"
              />
              <Tooltip label={sortDir === "desc" ? "Descending — click for ascending" : "Ascending — click for descending"}>
                <button
                  type="button"
                  className="sort-dir-toggle"
                  aria-label={`Sort direction (currently ${sortDir === "desc" ? "descending" : "ascending"})`}
                  aria-pressed={sortDir === "asc"}
                  onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}
                >
                  {sortDir === "desc"
                    ? <ArrowDown size={14} aria-hidden="true" />
                    : <ArrowUp size={14} aria-hidden="true" />}
                </button>
              </Tooltip>
            </div>
          </header>
          <div className="section__body">
            {activeItems.length === 0 ? (
              <div className="empty">
                <div className="empty-text">
                  No {tab === "series" ? "series" : tab === "movies" ? "movies" : "items"} in your library yet.
                </div>
              </div>
            ) : (
              <div className="show-grid" data-halo-cluster ref={gridRef}>
                {sortedItems.map((item) => renderCard(item))}
              </div>
            )}
          </div>
        </section>
      ) : null}
    </Page>
  );
}

export default HomePage;
