import { useCallback, useEffect, useMemo, useState } from "react";
import { Eye } from "lucide-react";
import type { LibraryItem } from "../../types/electron";
import type { AnilistWatchingEntry, WatchingListResult } from "../../main/preload";
import { findNextUpcomingEpisode } from "../utils/airingUtils";
import { fmtShort } from "../utils/relativeTime";
import { useHiddenShows } from "../contexts/HiddenShowsContext";
import ShowCard from "../components/ShowCard";
import { Page, Pill } from "../components/primitives";

// AniList brand mark. Inline rather than fetched so it ships with the
// renderer bundle and stays available offline. Geometry is the official
// stylized "A" from anilist.co/img/icons; currentColor so it tints with
// the surrounding chip styling.
function AniListIcon({ size = 12 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
      focusable="false"
    >
      <path d="M6.361 2.943 0 21.056h4.942l1.077-3.133H11.4l1.052 3.133H22.9c.71 0 1.1-.39 1.1-1.1V17.53c0-.71-.39-1.1-1.1-1.1h-6.483V4.045c0-.71-.39-1.1-1.1-1.1h-2.422c-.71 0-1.1.39-1.1 1.1v1.728L11.295 4.04c-.39-.71-.926-1.097-1.55-1.097H6.361zm2.107 6.508 1.541 4.514H6.928l1.54-4.514z" />
    </svg>
  );
}

// Turn a non-library watching entry into a LibraryItem so it can flow through
// the shared ShowCard unchanged. Empty `files` marks it as "not owned"; the
// single synthesized episode (when a next airing is known) drives the
// countdown. Watched-count and personal-score badges resolve separately via
// TrackerProgressContext keyed on anilistId, so they still render here.
function synthItem(e: AnilistWatchingEntry): LibraryItem {
  return {
    id: `anilist:${e.anilistId}`,
    folderName: e.titleRomaji ?? e.titleEnglish ?? `AniList ${e.anilistId}`,
    folderPath: "",
    type: "series",
    poster: e.coverImage,
    posterLocal: null,
    posterMatched: false,
    posterMatchAttempted: false,
    matchSource: "anilist",
    matchedTitle: e.titleRomaji,
    titleRomaji: e.titleRomaji,
    titleEnglish: e.titleEnglish,
    status: null,
    startDate: null,
    totalEpisodes: e.totalEpisodes,
    anilistId: e.anilistId,
    malId: e.malId,
    hidden: false,
    averageScore: e.averageScore,
    source: "anilist",
    episodes: e.nextAiringEpisode
      ? [{ episodeNumber: e.nextAiringEpisode.episode, airDate: new Date(e.nextAiringEpisode.airingAtMs).toISOString() }]
      : [],
    files: [],
  };
}

interface WatchingCard {
  key: string;
  item: LibraryItem;
  inLibrary: boolean;
  siteUrl: string;
}

// Session-level cache (module scope = lives as long as the window). Switching
// to the Watching tab renders this instantly instead of waiting on the AniList
// round-trip; a background refresh on every mount keeps it fresh and updates
// the UI when it resolves. Not persisted to disk; a fresh app launch still
// does one initial fetch.
let cachedResult: WatchingListResult | null = null;
let cachedLibrary: LibraryItem[] = [];

function WatchingPage() {
  const [result, setResult] = useState<WatchingListResult | null>(() => cachedResult);
  const [libraryItems, setLibraryItems] = useState<LibraryItem[]>(() => cachedLibrary);
  // Spinner only when there's nothing cached to show yet.
  const [loading, setLoading] = useState(() => cachedResult == null);
  const { showHidden } = useHiddenShows();

  const reload = useCallback(async () => {
    setLoading(cachedResult == null);
    try {
      const [list, lib] = await Promise.all([
        window.electronAPI.trackerGetWatchingList(),
        window.electronAPI.libraryWalk(),
      ]);
      cachedLibrary = Array.isArray(lib) ? lib : [];
      setLibraryItems(cachedLibrary);
      // Adopt the new result when it's good, or when we have nothing good
      // cached yet. If a background refresh fails (ok: false) but we already
      // have a working list, keep showing it rather than flashing an error.
      if (list.ok || cachedResult == null || !cachedResult.ok) {
        cachedResult = list;
        setResult(list);
      }
    } catch (err) {
      if (cachedResult == null) setResult({ ok: false, error: (err as Error).message });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  // Index the local library by BOTH ids so a watching entry can find its
  // owned counterpart (solid card, opens the in-app series page) regardless of
  // which provider the library show was matched against. A show matched via
  // MAL has anilistId === null, so anilist-only matching would miss it.
  const libraryIndex = useMemo(() => {
    const byAnilist = new Map<number, LibraryItem>();
    const byMal = new Map<number, LibraryItem>();
    for (const item of libraryItems) {
      if (item.anilistId != null && !byAnilist.has(item.anilistId)) byAnilist.set(item.anilistId, item);
      if (item.malId != null && !byMal.has(item.malId)) byMal.set(item.malId, item);
    }
    return { byAnilist, byMal };
  }, [libraryItems]);

  const cards = useMemo<WatchingCard[]>(() => {
    if (!result?.ok) return [];
    // Recently updated first; entries without updatedAt sink to the bottom.
    const sorted = [...result.entries].sort((a, b) => (b.updatedAt ?? 0) - (a.updatedAt ?? 0));
    // Index ALL items (incl. hidden) so a hidden owned show resolves to `owned`
    // and gets dropped here, rather than falling back to a synth "not owned"
    // card. Revealed hidden owned shows still render (ShowCard badges them).
    const mapped = sorted.map((e): WatchingCard | null => {
      const owned =
        libraryIndex.byAnilist.get(e.anilistId) ??
        (e.malId != null ? libraryIndex.byMal.get(e.malId) : undefined);
      if (owned?.hidden && !showHidden) return null;
      return owned
        ? { key: owned.id, item: owned, inLibrary: true, siteUrl: e.siteUrl }
        : { key: `anilist:${e.anilistId}`, item: synthItem(e), inLibrary: false, siteUrl: e.siteUrl };
    });
    return mapped.filter((c): c is WatchingCard => c !== null);
  }, [result, libraryIndex, showHidden]);

  // updatedAt lookup for the meta row, keyed by anilistId.
  const updatedByAnilist = useMemo(() => {
    const map = new Map<number, number | null>();
    if (result?.ok) for (const e of result.entries) map.set(e.anilistId, e.updatedAt);
    return map;
  }, [result]);

  // Shared coarse countdown ticker (same pattern as the Feed). Only mounted
  // when at least one card has an upcoming episode.
  const [nowMs, setNowMs] = useState<number>(() => Date.now());
  const hasAnyUpcoming = useMemo(
    () => cards.some((c) => findNextUpcomingEpisode(c.item.episodes, Date.now()) != null),
    [cards],
  );
  useEffect(() => {
    if (!hasAnyUpcoming) return;
    const id = window.setInterval(() => setNowMs(Date.now()), 30_000);
    return () => window.clearInterval(id);
  }, [hasAnyUpcoming]);

  const head = (
    <div>
      <h1 className="page-title">Watching</h1>
      <p className="page-sub">Your AniList watching list. Dashed cards are AniList-only; click to open them there.</p>
    </div>
  );

  if (loading && !result) {
    return (
      <Page head={head}>
        <div className="loading">Loading watching list…</div>
      </Page>
    );
  }

  return (
    <Page head={head}>
      {result?.ok ? (
        cards.length === 0 ? (
          <EmptyState title="Nothing on your watching list" hint="Mark a show as Watching on AniList to see it here." />
        ) : (
          <div className="show-grid watching-grid" data-halo-cluster>
            {cards.map((c) => {
              const updated = updatedByAnilist.get(c.item.anilistId ?? -1) ?? null;
              return (
                // Ownership code: owned = the standard solid hairline card;
                // AniList-only = dashed border + the AniList mark in a scrim
                // chip, both applied by the wrapper so ShowCard stays shared.
                <div key={c.key} className={`watching-card${c.inLibrary ? "" : " watching-card--external"}`}>
                  <ShowCard
                    item={c.item}
                    onActivate={c.inLibrary ? undefined : () => void window.electronAPI.openExternal(c.siteUrl)}
                    metaLeftText={updated != null ? fmtShort(updated * 1000) : c.inLibrary ? undefined : "-"}
                    metaLeftTitle={updated != null ? "Last updated on AniList" : undefined}
                    nowMs={nowMs}
                  />
                  {!c.inLibrary && (
                    <div className="watching-card__external-mark" aria-hidden="true">
                      <Pill size="sm" scrim><AniListIcon size={11} /></Pill>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )
      ) : result ? (
        <EmptyState
          title={result.needsAuth ? "AniList not connected" : "Couldn't load watching list"}
          hint={result.needsAuth ? "Connect AniList in Settings → Trackers to see your watching list." : result.error}
        />
      ) : null}
    </Page>
  );
}

function EmptyState({ title, hint }: { title: string; hint: string }) {
  return (
    <div className="empty">
      <div className="empty-icon"><Eye size={48} /></div>
      <div className="empty-title">{title}</div>
      <div className="empty-text">{hint}</div>
    </div>
  );
}

export default WatchingPage;
