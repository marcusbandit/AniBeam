import { Fragment, useCallback, useEffect, useMemo, useState } from "react";
import { Activity, History, CalendarClock } from "lucide-react";
import type { LibraryItem } from "../../types/electron";
import { findNextUpcomingEpisode } from "../utils/airingUtils";
import { fmtShort } from "../utils/relativeTime";
import { useDebouncedCallback } from "../hooks/useDebouncedCallback";
import { useHiddenShows } from "../contexts/HiddenShowsContext";
import ShowCard from "../components/ShowCard";
import { Page, Inline, SegmentedSwitch, type SegmentedOption } from "../components/primitives";

type FeedSort = "recent" | "upcoming";

const LS_FEED_SORT = "anibeam.feedSort";

const FEED_SORT_OPTIONS: SegmentedOption<FeedSort>[] = [
  { value: "recent", label: <History size={16} aria-hidden="true" />, ariaLabel: "Recently released" },
  { value: "upcoming", label: <CalendarClock size={16} aria-hidden="true" />, ariaLabel: "Coming soon" },
];

interface FeedEntry {
  item: LibraryItem;
  // Unix seconds. Past for "recent" (last release), future for "upcoming"
  // (next air date).
  when: number;
  episodeNumber: number | null;  // the episode this entry refers to
  source: "aired" | "downloaded" | "upcoming";
  // For "upcoming" entries only: the show's LAST release (aired/downloaded),
  // shown in the meta row. The future air time itself is the countdown chip's
  // job; repeating it on the left would state the same fact twice.
  lastWhen?: number;
  lastSource?: "aired" | "downloaded";
}

// Highest episode number actually sitting on disk: the value the "EP NN"
// badge shows everywhere else in the app (Library, Airing). 0 when nothing is
// on disk. The single source of truth for what "you have" means on a card.
function highestOnDiskEpisode(item: LibraryItem): number {
  let max = 0;
  for (const f of item.files) if (f.episodeNumber > max) max = f.episodeNumber;
  return max;
}

// One entry per show. Derived from the C reference (src/ui.c:955), but the
// shown episode is the newest one ACTUALLY on disk, not just the latest with a
// known past airDate: a freshly-downloaded episode whose airDate metadata
// hasn't landed yet (or is flagged future) still counts. Without this, a show
// with 8 files on disk but only 7 dated episodes reads "EP 07". Mirrors
// HomePage.getAiringSortInfo. Future episodes are ignored for the timestamp.
// Returns null when there's no information at all (no episodes, no files).
function buildRecentEntry(item: LibraryItem, nowSec: number): FeedEntry | null {
  if (item.files.length === 0) return null;

  // Latest aired-and-on-disk episode (needs a known past airDate); used for
  // the "aired" timestamp/source.
  const onDiskEps = new Set(item.files.map((f) => f.episodeNumber));
  let bestAired: { ts: number; ep: number } | null = null;
  for (const e of item.episodes) {
    if (!e.airDate || !onDiskEps.has(e.episodeNumber)) continue;
    const t = Math.floor(Date.parse(e.airDate) / 1000);
    if (!Number.isFinite(t) || t > nowSec) continue;
    if (!bestAired || t > bestAired.ts) bestAired = { ts: t, ep: e.episodeNumber };
  }

  // Newest episode on disk drives the badge; its newest mtime is the
  // "downloaded" fallback timestamp.
  const highestOnDisk = highestOnDiskEpisode(item);
  let newestMtime = 0;
  for (const f of item.files) {
    if (f.mtime && f.mtime > newestMtime) newestMtime = f.mtime;
  }
  const episodeNumber = Math.max(highestOnDisk, bestAired?.ep ?? 0) || null;

  // Prefer the shown episode's own past airDate (an "aired" entry at that
  // time); else the newest file mtime ("downloaded"); else the best aired
  // time; else bail.
  const epAir = item.episodes.find((e) => e.episodeNumber === episodeNumber && e.airDate);
  const epAirT = epAir?.airDate ? Math.floor(Date.parse(epAir.airDate) / 1000) : NaN;
  if (Number.isFinite(epAirT) && epAirT <= nowSec) {
    return { item, when: epAirT, episodeNumber, source: "aired" };
  }
  if (newestMtime) {
    return { item, when: Math.floor(newestMtime / 1000), episodeNumber, source: "downloaded" };
  }
  if (bestAired) {
    return { item, when: bestAired.ts, episodeNumber, source: "aired" };
  }
  return null;
}

// "Recent": every show ordered by its latest release/download, newest first.
function buildRecentFeed(items: LibraryItem[]): FeedEntry[] {
  const nowSec = Math.floor(Date.now() / 1000);
  const out: FeedEntry[] = [];
  for (const item of items) {
    const entry = buildRecentEntry(item, nowSec);
    if (entry) out.push(entry);
  }
  out.sort((a, b) => b.when - a.when);
  return out;
}

// "Coming soon": shows with a known upcoming episode float to the top, soonest
// air date first, so you can see what's next. Shows without a scheduled next
// episode (finished, or no airing schedule cached) keep their recent ordering
// below, so the feed still lists everything you own. The render inserts an
// eyebrow divider between the two groups.
function buildUpcomingFeed(items: LibraryItem[]): FeedEntry[] {
  const nowMs = Date.now();
  const nowSec = Math.floor(nowMs / 1000);
  const upcoming: FeedEntry[] = [];
  const rest: FeedEntry[] = [];
  for (const item of items) {
    if (item.files.length === 0) continue;
    const next = findNextUpcomingEpisode(item.episodes, nowMs);
    if (next) {
      // The meta row shows the show's LAST release, not the future air time
      // (the countdown chip already owns that fact).
      const last = buildRecentEntry(item, nowSec);
      upcoming.push({
        item,
        when: Math.floor(next.airDateMs / 1000),
        // Badge the newest episode ON DISK, same as the Airing view and the
        // rest of the app, NOT the upcoming one. `when` (next air date) still
        // drives the soonest-first sort, and the countdown chip conveys
        // what's coming. Badging next.episodeNumber here made the feed read
        // "EP 09" while only 8 episodes were on disk.
        episodeNumber: highestOnDiskEpisode(item) || null,
        source: "upcoming",
        lastWhen: last?.when,
        // buildRecentEntry only ever yields aired/downloaded; the guard just
        // narrows the type.
        lastSource: last && last.source !== "upcoming" ? last.source : undefined,
      });
    } else {
      const entry = buildRecentEntry(item, nowSec);
      if (entry) rest.push(entry);
    }
  }
  upcoming.sort((a, b) => a.when - b.when);  // soonest first
  rest.sort((a, b) => b.when - a.when);
  return [...upcoming, ...rest];
}

const META_LEFT_TITLE: Record<"aired" | "downloaded", string> = {
  aired: "Episode aired",
  downloaded: "File downloaded",
};

function FeedPage() {
  const [items, setItems] = useState<LibraryItem[]>([]);
  const [initialLoading, setInitialLoading] = useState(true);
  const { showHidden } = useHiddenShows();
  const [sortMode, setSortMode] = useState<FeedSort>(() => {
    if (typeof window === "undefined") return "recent";
    const raw = window.localStorage.getItem(LS_FEED_SORT);
    return raw === "upcoming" || raw === "recent" ? raw : "recent";
  });
  useEffect(() => { window.localStorage.setItem(LS_FEED_SORT, sortMode); }, [sortMode]);

  const reload = useCallback(async () => {
    try {
      const data = await window.electronAPI.libraryWalk();
      setItems(Array.isArray(data) ? data : []);
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

  // Debounced; see HomePage for the reasoning. New shows fire bursts of
  // metadata change pings; we want one walk per burst, not 2N+1.
  const debouncedReload = useDebouncedCallback(() => { void reload(); }, 250);
  useEffect(() => {
    const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
      debouncedReload();
    });
    return () => unsubscribe?.();
  }, [debouncedReload]);

  // Hidden series drop out of the feed unless reveal is on; revealed ones
  // still render through ShowCard, which badges + dims them.
  const feedItems = useMemo(
    () => (showHidden ? items : items.filter((i) => !i.hidden)),
    [items, showHidden],
  );

  const entries = useMemo(
    () => (sortMode === "upcoming" ? buildUpcomingFeed(feedItems) : buildRecentFeed(feedItems)),
    [feedItems, sortMode],
  );

  // Where the "has a known next episode" group ends and the recent-order
  // backfill begins (upcoming sort only). -1 or 0 means there is no boundary
  // to mark: either everything has a next episode, or nothing does.
  const dividerIndex = useMemo(
    () => (sortMode === "upcoming" ? entries.findIndex((e) => e.source !== "upcoming") : -1),
    [entries, sortMode],
  );

  // Shared coarse tick for per-card countdowns. We render minute-
  // granularity here, so a 30s interval gives at-worst 30s display lag
  // without spinning a 1Hz timer in the background. Only mounted when at
  // least one entry actually has an upcoming episode.
  const [nowMs, setNowMs] = useState<number>(() => Date.now());
  const hasAnyUpcoming = useMemo(
    () => entries.some(({ item }) => findNextUpcomingEpisode(item.episodes, Date.now()) != null),
    [entries],
  );
  useEffect(() => {
    if (!hasAnyUpcoming) return;
    const id = window.setInterval(() => setNowMs(Date.now()), 30_000);
    return () => window.clearInterval(id);
  }, [hasAnyUpcoming]);

  if (initialLoading) {
    return (
      <Page>
        <div className="loading">Loading feed…</div>
      </Page>
    );
  }

  return (
    <Page
      head={
        <Inline justify="space-between" align="flex-start">
          <div>
            <h1 className="page-title">Feed</h1>
            <p className="page-sub">
              {sortMode === "upcoming"
                ? "Upcoming episodes, soonest first."
                : "Your library, ordered by latest episode release."}
            </p>
          </div>
          <SegmentedSwitch<FeedSort>
            value={sortMode}
            options={FEED_SORT_OPTIONS}
            onChange={setSortMode}
            ariaLabel="Feed order"
          />
        </Inline>
      }
    >
      {entries.length === 0 ? (
        <div className="empty">
          <div className="empty-icon"><Activity size={48} /></div>
          <div className="empty-title">Nothing here yet</div>
          <div className="empty-text">Add a folder in Settings to build your feed.</div>
        </div>
      ) : (
        <div className="show-grid" data-halo-cluster>
          {entries.map((entry, idx) => {
            const { item, when, episodeNumber, source } = entry;
            // Upcoming cards: the future air time lives in the countdown chip
            // alone; the meta row shows the last release instead (or falls
            // back to ShowCard's file count when the show has none recorded).
            const metaWhen = source === "upcoming" ? entry.lastWhen : when;
            const metaSource: "aired" | "downloaded" | undefined =
              source === "upcoming" ? entry.lastSource : source;
            return (
              <Fragment key={item.id}>
                {dividerIndex > 0 && idx === dividerIndex && (
                  <div className="feed-divider" role="presentation">
                    <span className="feed-divider__label">Everything else</span>
                    <span className="feed-divider__rule" aria-hidden="true" />
                  </div>
                )}
                <ShowCard
                  item={item}
                  episodeBadgeNumber={episodeNumber}
                  metaLeftText={metaWhen != null ? fmtShort(metaWhen * 1000) : undefined}
                  metaLeftTitle={metaWhen != null && metaSource ? META_LEFT_TITLE[metaSource] : undefined}
                  nowMs={nowMs}
                />
              </Fragment>
            );
          })}
        </div>
      )}
    </Page>
  );
}

export default FeedPage;
