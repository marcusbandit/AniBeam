import { useEffect, useMemo, useState, useCallback, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Play, Check, Star, Tv, Film, Clock, Users, RotateCw, Eye, EyeOff, AlertTriangle } from "lucide-react";

// AniList brand mark. Inline rather than fetched so it ships with the
// renderer bundle and stays available offline. Geometry is the official
// stylized "A" from anilist.co/img/icons — currentColor so it tints with
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
import type { LibraryItem } from "../../types/electron";
import type { Character, Recommendation, SeriesMetadata, Tag } from "../hooks/useMetadata";
import { useDebouncedCallback } from "../hooks/useDebouncedCallback";
import { useTitleLanguage } from "../contexts/TitleLanguageContext";
import { useTrackerProgress } from "../contexts/TrackerProgressContext";
import { getDisplayRating, formatRating } from "../utils/ratingUtils";
import {
  formatEpisodeCode,
  getLatestAiredEpisodeNumber,
  normalizeStatus,
  findNextUpcomingEpisode,
  formatCountdown,
} from "../utils/airingUtils";
import {
  readProgress,
  readLastEpisodeMap,
  getProgressFraction,
  type ProgressMap,
  type LastEpisodeMap,
} from "../utils/playbackProgress";
import type { TrackerListStatus } from "../../main/preload";
import { Page, Section, Card, EpisodeRow, Pill, ScorePicker, Tooltip } from "../components/primitives";
import { FranchiseGraphView } from "../components/franchise";
import type { FranchiseCategory, FranchiseFormat } from "../components/franchise";
import { useFranchiseGraph } from "../hooks/useFranchiseGraph";
import type { FranchiseNode } from "../../shared/franchise";

const LIST_STATUS_LABEL: Record<TrackerListStatus, string> = {
  watching: "Watching",
  planning: "Planning",
  completed: "Completed",
  paused: "Paused",
  dropped: "Dropped",
  repeating: "Rewatching",
};

// AniList descriptions ship with HTML (<br>, <i>, <b>). Strip tags and
// collapse whitespace so the hero blurb reads cleanly. We don't need a
// full sanitizer — this string is never rendered as HTML, only as text.
function stripHtml(s: string | null | undefined): string {
  if (!s) return "";
  return s.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
}

function formatStatus(status: string | null | undefined): string | null {
  const norm = normalizeStatus(status);
  if (!norm) return null;
  const map: Record<string, string> = {
    releasing: "Airing",
    finished: "Finished",
    upcoming: "Upcoming",
    cancelled: "Cancelled",
    hiatus: "Hiatus",
  };
  return map[norm] ?? norm.replace(/_/g, " ");
}


function relationFormatLabel(format: string | null): string | null {
  if (!format) return null;
  const map: Record<string, string> = {
    TV: "TV",
    TV_SHORT: "TV Short",
    MOVIE: "Movie",
    OVA: "OVA",
    ONA: "ONA",
    SPECIAL: "Special",
    MUSIC: "Music",
    MANGA: "Manga",
    NOVEL: "Novel",
    LIGHT_NOVEL: "Light Novel",
    ONE_SHOT: "One-shot",
    VISUAL_NOVEL: "Visual Novel",
  };
  return map[format] ?? format.replace(/_/g, " ");
}


function formatYear(startDate: string | null | undefined, seasonYear: number | null | undefined): string | null {
  if (typeof seasonYear === "number") return String(seasonYear);
  if (!startDate) return null;
  const y = parseInt(startDate.split("-")[0], 10);
  return Number.isFinite(y) ? String(y) : null;
}

function SeriesDetailPage() {
  const { seriesId } = useParams<{ seriesId: string }>();
  const navigate = useNavigate();
  const { pickTitle } = useTitleLanguage();
  const { getWatched, getListStatus, getUserScore, getRewatchCount } = useTrackerProgress();
  // Tag-panel spoiler toggle. Default off — opt-in only, per the user's
  // standing preference to hide plot-spoiler tags until explicitly revealed.
  const [showSpoilerTags, setShowSpoilerTags] = useState(false);
  // Franchise filter chips — tracks which relation categories are hidden.
  const [hiddenCategories, setHiddenCategories] = useState<ReadonlySet<FranchiseCategory>>(
    () => new Set(),
  );
  const [hiddenFormats, setHiddenFormats] = useState<ReadonlySet<FranchiseFormat>>(() => new Set());
  const toggleFormat = useCallback((fmt: FranchiseFormat) => {
    setHiddenFormats((prev) => {
      const n = new Set(prev);
      if (n.has(fmt)) n.delete(fmt); else n.add(fmt);
      return n;
    });
  }, []);

  const [item, setItem] = useState<LibraryItem | null>(null);
  const [meta, setMeta] = useState<SeriesMetadata | null>(null);
  // Full library list (not just the current series) — needed to resolve
  // relation clicks. If a related entry's anilistId/malId matches a
  // series the user has on disk, the click navigates in-app; otherwise
  // we open AniList in the system browser.
  const [allItems, setAllItems] = useState<LibraryItem[]>([]);
  // Full metadata map keyed by seriesId. Needed for transitive franchise
  // expansion — to find S3 reachable from S1 through S2, we have to walk
  // each in-library series's own `relations` array, not just the current
  // series's.
  const [allMeta, setAllMeta] = useState<Record<string, SeriesMetadata>>({});
  const [initialLoading, setInitialLoading] = useState(true);
  // Per-episode resume position + per-series last-finished episode, both
  // sourced from localStorage. Refreshed on mount and on window-focus so a
  // session in the player updates the bars and "Next up" marker as soon as
  // the user navigates back here. Cheap — both are single JSON.parse calls.
  const [localProgress, setLocalProgress] = useState<ProgressMap>(() => readProgress());
  const [lastEpMap, setLastEpMap] = useState<LastEpisodeMap>(() => readLastEpisodeMap());
  // 1Hz tick driving the next-episode countdown. Only runs while a future
  // air date exists for the active series so an off-air finished show
  // doesn't spin the timer for nothing.
  const [nowMs, setNowMs] = useState<number>(() => Date.now());
  // Inline score popover state. Anchored to the score chip in the hero row
  // and used as the fallback path when the auto-prompt at the end of the
  // final episode is dismissed (or never fires because the user marked the
  // series outside the player). Default 8.0 matches the player popup.
  const [scoreEditing, setScoreEditing] = useState(false);
  const [scoreDraft, setScoreDraft] = useState<string>('8.0');
  const [scoreBusy, setScoreBusy] = useState(false);
  const [scoreError, setScoreError] = useState<string | null>(null);
  // Marker track/untrack cascade. `wave` is the active animated range + phase:
  // 'in' (hover), 'out' (reverse wave on un-hover), 'commit' (after a click).
  // [lo, hi] is the affected episode range; `anchor` is the cursor/click origin
  // the stagger emanates from. `optimisticWatched` flips icons/colours the moment
  // a click lands so the cascade isn't gated on the tracker round-trip.
  const [wave, setWave] = useState<
    { mode: "track" | "untrack"; phase: "in" | "out" | "commit"; lo: number; hi: number; anchor: number } | null
  >(null);
  const [optimisticWatched, setOptimisticWatched] = useState<number | null>(null);
  const waveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const leaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const refresh = () => {
      setLocalProgress(readProgress());
      setLastEpMap(readLastEpisodeMap());
    };
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, []);

  // Clear any pending marker-cascade timers on unmount.
  useEffect(() => () => {
    if (waveTimerRef.current) clearTimeout(waveTimerRef.current);
    if (leaveTimerRef.current) clearTimeout(leaveTimerRef.current);
  }, []);

  const decodedId = seriesId ? decodeURIComponent(seriesId) : "";

  // No setLoading toggle on reload pings — keeps the file list visible
  // while the background match updates posters/dates incrementally.
  const reload = useCallback(async () => {
    try {
      const [all, allMeta] = await Promise.all([
        window.electronAPI.libraryWalk(),
        window.electronAPI.loadMetadata() as Promise<Record<string, SeriesMetadata>>,
      ]);
      const fresh = Array.isArray(all) ? all : [];
      const found = fresh.find((i) => i.id === decodedId) ?? null;
      const metaMap = (allMeta && typeof allMeta === "object")
        ? (allMeta as Record<string, SeriesMetadata>)
        : {};
      setAllItems(fresh);
      setItem(found);
      setAllMeta(metaMap);
      setMeta(metaMap[decodedId] ?? null);
    } catch (err) {
      console.error("library:walk failed", err);
      setAllItems([]);
      setItem(null);
      setMeta(null);
      setAllMeta({});
    } finally {
      setInitialLoading(false);
    }
  }, [decodedId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  // Debounced — bursts of metadata pings on new ingests would otherwise
  // re-fetch the entire library walk for every event.
  const debouncedReload = useDebouncedCallback(() => { void reload(); }, 250);
  useEffect(() => {
    const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
      debouncedReload();
    });
    return () => unsubscribe?.();
  }, [debouncedReload]);

  // Re-derive each render — cheap (single pass over the episode list) and
  // automatically rolls to the next-next episode once the current one's
  // airDate slips into the past.
  const nextUpcoming = findNextUpcomingEpisode(item?.episodes ?? null, nowMs);

  // 1Hz tick — only while a future air date exists. Resets the interval
  // when the upcoming episode changes (e.g. rollover after airing) so we
  // never drift on top of a stale schedule.
  useEffect(() => {
    if (!nextUpcoming) return;
    const id = window.setInterval(() => setNowMs(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, [nextUpcoming?.airDateMs]);

  // anilistId / malId → seriesId index over the user's library. Used by
  // the Related strip below to decide whether a click navigates in-app or
  // opens AniList externally.
  const ownedByExternalId = useMemo(() => {
    const byAnilist = new Map<number, string>();
    const byMal = new Map<number, string>();
    for (const it of allItems) {
      if (it.anilistId != null) byAnilist.set(it.anilistId, it.id);
      if (it.malId != null) byMal.set(it.malId, it.id);
    }
    return { byAnilist, byMal };
  }, [allItems]);

  const { graph: franchiseGraph, filling: franchiseFilling } = useFranchiseGraph(meta?.anilistId, allMeta);

  const resolveOwnedNode = useCallback((node: FranchiseNode): string | undefined => {
    if (node.type === "MANGA") return undefined;
    return (node.anilistId != null ? ownedByExternalId.byAnilist.get(node.anilistId) : undefined)
      ?? (node.malId != null ? ownedByExternalId.byMal.get(node.malId) : undefined);
  }, [ownedByExternalId]);

  const openExternalNode = useCallback((node: FranchiseNode) => {
    const url = node.siteUrl
      ?? (node.type === "MANGA"
        ? `https://anilist.co/manga/${node.anilistId}`
        : `https://anilist.co/anime/${node.anilistId}`);
    if (url) void window.electronAPI.openExternal(url);
  }, []);

  // Stable order: by season then episode number, falling back to filename.
  const sorted = useMemo(() => {
    if (!item) return [];
    return [...item.files].sort((a, b) => {
      const sa = a.seasonNumber ?? 0;
      const sb = b.seasonNumber ?? 0;
      if (sa !== sb) return sa - sb;
      if (a.episodeNumber !== b.episodeNumber) return a.episodeNumber - b.episodeNumber;
      return a.filename.localeCompare(b.filename);
    });
  }, [item]);

  // filePath → true for any episode that's being served from the on-disk
  // transcode cache (i.e. the source codec needed conversion before
  // Chromium could play it). LibraryItem.files doesn't carry this — the
  // signal only lives on metadata.json's fileEpisodes — so we index it
  // here and the row look-up stays O(1).
  const transcodedByPath = useMemo(() => {
    const set = new Set<string>();
    for (const fe of meta?.fileEpisodes ?? []) {
      if (fe.transcodedPath) set.add(fe.filePath);
    }
    return set;
  }, [meta?.fileEpisodes]);

  // Pre-build the episode-title lookup from API metadata so the episode
  // list can swap file-derived titles for canonical AniList/MAL titles.
  // Defined here (above the early-return guards) so the hook order stays
  // stable across the initial-loading → loaded transition.
  const apiTitleByEp = useMemo(() => {
    const map = new Map<number, string>();
    const eps = (meta?.episodes ?? []) as Array<{ episodeNumber: number; title?: string | null }>;
    for (const e of eps) {
      if (typeof e.episodeNumber === 'number' && typeof e.title === 'string' && e.title.length > 0) {
        map.set(e.episodeNumber, e.title);
      }
    }
    return map;
  }, [meta?.episodes]);
  // Tags — sort by AniList rank desc, filter spoilers/adult unless the user
  // has toggled them on. We never cap at a fixed count here; the panel's CSS
  // clips overflow visually, so "however many fit" is layout-driven.
  const allTags: Tag[] = (meta?.tags ?? []) as Tag[];
  const visibleTags = useMemo(() => {
    return [...allTags]
      .filter((t) => showSpoilerTags || (!t.isMediaSpoiler && !t.isGeneralSpoiler && !t.isAdult))
      .sort((a, b) => (b.rank ?? 0) - (a.rank ?? 0));
  }, [allTags, showSpoilerTags]);
  const hasHiddenSpoilerTags = useMemo(
    () => allTags.some((t) => t.isMediaSpoiler || t.isGeneralSpoiler || t.isAdult),
    [allTags],
  );

  if (initialLoading) {
    return (
      <div className="page">
        <div className="loading">Reading folder…</div>
      </div>
    );
  }

  if (!item) {
    return (
      <div className="page">
        <div className="error">Folder not found.</div>
        <button className="btn btn-secondary" onClick={() => navigate("/")}>
          <ArrowLeft size={14} /> Library
        </button>
      </div>
    );
  }

  const isMovie = item.type === "movie" || meta?.format === "MOVIE";

  const displayTitle = pickTitle({
    titleRomaji: item.titleRomaji ?? meta?.titleRomaji ?? null,
    titleEnglish: item.titleEnglish ?? meta?.titleEnglish ?? null,
    folderName: item.folderName,
  });
  // Surface the alternate title underneath, so the hero shows both
  // identities at a glance without needing to flip the navbar switch.
  const titleRomaji = item.titleRomaji ?? meta?.titleRomaji ?? null;
  const titleEnglish = item.titleEnglish ?? meta?.titleEnglish ?? null;
  const altTitle = (() => {
    if (titleRomaji && titleEnglish && titleRomaji !== titleEnglish && displayTitle !== (titleRomaji === displayTitle ? titleEnglish : titleRomaji)) {
      return displayTitle === titleEnglish ? titleRomaji : titleEnglish;
    }
    return null;
  })();

  const posterUrl = item.posterLocal
    ? `media://${encodeURIComponent(item.posterLocal)}`
    : item.poster;

  const bannerUrl = (() => {
    const local = (meta as unknown as { bannerLocal?: string | null })?.bannerLocal;
    const remote = (meta as unknown as { banner?: string | null })?.banner;
    if (local) return `media://${encodeURIComponent(local)}`;
    return remote ?? null;
  })();

  const description = stripHtml(meta?.description);
  const rating = meta?.averageScore != null
    ? getDisplayRating(meta.averageScore, meta.source ?? (item.matchSource ?? null))
    : null;
  // User's own rating from MAL/AniList. The tracker layer normalises both
  // providers to a 0–10 scale and returns null for unrated, so we just have
  // to format it. Hidden when missing instead of "—" so the chip row stays
  // tidy on unrated series.
  const userScore = getUserScore({
    anilistId: item.anilistId ?? undefined,
    malId: item.malId ?? undefined,
  });
  const userScoreLabel = userScore != null ? formatRating(userScore) : null;

  const year = formatYear(item.startDate ?? meta?.startDate ?? null, meta?.seasonYear ?? null);
  const statusLabel = formatStatus(item.status ?? meta?.status ?? null);
  const totalEpisodes = item.totalEpisodes ?? meta?.totalEpisodes ?? null;
  const formatLabel = (() => {
    if (isMovie) return "Movie";
    const f = meta?.format;
    if (!f) return "Series";
    if (f === "TV") return "TV";
    if (f === "TV_SHORT") return "TV Short";
    if (f === "OVA") return "OVA";
    if (f === "ONA") return "ONA";
    if (f === "SPECIAL") return "Special";
    return f.replace(/_/g, " ");
  })();

  const trackerIds = {
    anilistId: item.anilistId ?? undefined,
    malId: item.malId ?? undefined,
  };
  const watched = getWatched(trackerIds);
  const listStatus = getListStatus(trackerIds);
  const rewatchCount = getRewatchCount(trackerIds);
  const hasTrackerId = item.anilistId != null || item.malId != null;
  // Animation studio — explicit `animationStudio` (set by the matcher when
  // AniList flagged it main + isAnimationStudio) wins; otherwise show the
  // first entry of `studios` so something is still surfaced.
  const studioName = (meta as unknown as { animationStudio?: string | null })?.animationStudio
    ?? (meta?.studios && meta.studios.length > 0 ? meta.studios[0] : null);
  const characters: Character[] = (meta?.characters ?? []) as Character[];
  // Show every stored recommendation (the main process caps storage at 8).
  // The strip below uses a single-row grid that shrinks cards to share the
  // width, so all picks stay on one line. AniList sorts by RATING_DESC.
  const recommendations: Recommendation[] = (meta?.recommendations ?? []) as Recommendation[];
  // Status marker for a Related/Recommended entry that's on the user's
  // tracker list. Resolves by external id (works whether or not the entry is
  // on disk) and returns the colour-coded corner dot, wrapped in a hover-pause
  // Tooltip naming the status — so a forgotten colour is one hover away. The
  // presence of the dot already means "on your list", so the label is just the
  // bare status ("Completed"), not "On your list · Completed". Off-list entries
  // render nothing.
  const listStatusMarker = (ids: { anilistId: number; malId: number | null }) => {
    const status = getListStatus(ids);
    if (!status) return null;
    const label = LIST_STATUS_LABEL[status];
    return (
      <Tooltip label={label}>
        <span
          className="list-status-dot"
          data-status={status}
          role="img"
          aria-label={label}
        />
      </Tooltip>
    );
  };
  const submitSeriesScore = async (raw: string) => {
    const score = parseFloat(raw);
    if (!Number.isFinite(score)) {
      setScoreError('invalid score');
      return;
    }
    setScoreBusy(true);
    setScoreError(null);
    const total = item.totalEpisodes ?? meta?.totalEpisodes ?? null;
    type ScoreRes = Awaited<ReturnType<typeof window.electronAPI.trackerSetScore>>;
    const calls: Promise<ScoreRes>[] = [];
    if (item.anilistId != null) {
      calls.push(window.electronAPI.trackerSetScore('anilist', item.anilistId, score, total));
    }
    if (item.malId != null) {
      calls.push(window.electronAPI.trackerSetScore('mal', item.malId, score, total));
    }
    const results = await Promise.allSettled(calls);
    let lastErr: string | null = null;
    let anyOk = false;
    for (const r of results) {
      if (r.status === 'rejected') { lastErr = (r.reason as Error)?.message ?? 'unknown error'; continue; }
      const v = r.value;
      if (v.ok) anyOk = true;
      else if (v.reason === 'error') lastErr = v.message ?? null;
      else if (v.reason === 'no-account') lastErr = lastErr ?? 'no tracker connected';
    }
    setScoreBusy(false);
    if (anyOk) {
      setScoreEditing(false);
    } else {
      setScoreError(lastErr ?? 'no tracker connected');
    }
  };
  // Set watched progress to an exact value on every connected tracker. Used by
  // the "untrack to here" markers (can decrease — corrects over-counts). The
  // shared progress cache refreshes via the tracker:progress-changed broadcast.
  const applyProgress = async (value: number) => {
    const target = Math.max(0, Math.floor(value));
    if (!Number.isFinite(target)) return;
    const total = item.totalEpisodes ?? meta?.totalEpisodes ?? null;
    type ProgRes = Awaited<ReturnType<typeof window.electronAPI.trackerSetProgress>>;
    const calls: Promise<ProgRes>[] = [];
    if (item.anilistId != null) {
      calls.push(window.electronAPI.trackerSetProgress('anilist', item.anilistId, target, total));
    }
    if (item.malId != null) {
      calls.push(window.electronAPI.trackerSetProgress('mal', item.malId, target, total));
    }
    await Promise.allSettled(calls);
  };
  const watchedCount = typeof watched === "number" ? watched : 0;
  const trackedKnown = watched != null;
  // Optimistic overlay: a click reflects immediately, then clears once the
  // tracker broadcast lands the real value (effect below).
  const effWatched = optimisticWatched ?? watchedCount;
  const effTrackedKnown = trackedKnown || optimisticWatched != null;
  // Denominator priority: published total → latest aired episode → files
  // on disk. Aired-but-not-final shows up as "+" in the label so the user
  // sees "04/05+" instead of a misleading "04/05" or a useless "04/?".
  const latestAired = getLatestAiredEpisodeNumber(item.episodes);
  const totalKnown = totalEpisodes != null && totalEpisodes > 0;
  const denom = totalKnown
    ? totalEpisodes!
    : (latestAired != null && latestAired > 0
        ? Math.max(latestAired, watchedCount)
        : sorted.length);
  const denomIsAiringEstimate = !totalKnown && latestAired != null && latestAired > 0;
  // Width of the watched-progress strip. When totalEpisodes is unknown we
  // fall back to the latest-aired count (or files on disk) so the bar
  // still reflects *something* meaningful instead of staying flat.
  const progressPct = denom > 0 ? Math.min(100, (watchedCount / denom) * 100) : 0;
  // The next-up episode is "last completed + 1", where last-completed is the
  // max of (tracker watched count, locally finished episode from the player).
  // The local fallback keeps the marker accurate when the tracker is behind
  // or the user is rewatching after the list is already marked completed —
  // in the rewatch case the row may itself be marked "watched", and we show
  // the marker anyway so the user always knows where they left off.
  const lastEpLocal = lastEpMap[item.id]?.ep ?? null;
  const effectiveLastWatched = Math.max(
    effTrackedKnown ? effWatched : 0,
    lastEpLocal ?? 0,
  );
  const nextEpNumber = effectiveLastWatched > 0
    ? sorted.find((f) => f.episodeNumber === effectiveLastWatched + 1)?.episodeNumber
        ?? sorted.find((f) => f.episodeNumber > effectiveLastWatched)?.episodeNumber
        ?? null
    : (effTrackedKnown ? sorted.find((f) => f.episodeNumber > effWatched)?.episodeNumber ?? null : null);

  // ---- Marker track/untrack cascade handlers ----
  const CASCADE_STEP_MS = 45;
  const CASCADE_DUR_MS = 340;
  // Auto-clear the wave after the staggered animation has fully played out.
  const scheduleWaveClear = (span: number) => {
    if (waveTimerRef.current) clearTimeout(waveTimerRef.current);
    waveTimerRef.current = setTimeout(
      () => setWave(null),
      Math.min(span, 12) * CASCADE_STEP_MS + CASCADE_DUR_MS + 80,
    );
  };
  // Entering the CIRCLE initiates / re-anchors the wave.
  const onMarkerEnter = (ep: number, isW: boolean) => {
    if (leaveTimerRef.current) { clearTimeout(leaveTimerRef.current); leaveTimerRef.current = null; }
    if (waveTimerRef.current) { clearTimeout(waveTimerRef.current); waveTimerRef.current = null; }
    if (isW) setWave({ mode: "untrack", phase: "in", lo: ep, hi: effWatched, anchor: ep });
    else setWave({ mode: "track", phase: "in", lo: effWatched + 1, hi: ep, anchor: ep });
  };
  // Entering a hit-zone (anywhere, not necessarily the circle) only KEEPS an
  // active hover alive — it cancels a pending leave but never starts a wave. So
  // moving between adjacent zones stays hovered, but merely being inside a zone
  // (without touching a circle) won't initiate one.
  const onMarkerZoneEnter = () => {
    if (leaveTimerRef.current) { clearTimeout(leaveTimerRef.current); leaveTimerRef.current = null; }
  };
  // Leaving a hit-zone is debounced: if another zone's enter fires within the
  // grace window we stay hovered (just moving between circles). Only when the
  // cursor is in NO zone does the reverse cascade play out from where it left.
  const onMarkerLeave = () => {
    if (leaveTimerRef.current) clearTimeout(leaveTimerRef.current);
    const w = wave;
    leaveTimerRef.current = setTimeout(() => {
      leaveTimerRef.current = null;
      if (!w || w.phase !== "in") return;
      setWave({ ...w, phase: "out" });
      scheduleWaveClear(w.hi - w.lo);
    }, 60);
  };
  const onMarkerClick = (ep: number, isW: boolean) => {
    const newWatched = isW ? ep - 1 : ep;
    const mode = isW ? "untrack" : "track";
    const lo = isW ? ep : effWatched + 1;
    const hi = isW ? effWatched : ep;
    setOptimisticWatched(newWatched);
    setWave({ mode, phase: "commit", lo, hi, anchor: ep });
    scheduleWaveClear(hi - lo);
    // Drop the optimistic overlay once the IPC resolves and the
    // tracker:progress-changed broadcast has had a tick to refresh the real
    // value. The guard keeps a later click's optimistic value from being cleared.
    void applyProgress(newWatched).finally(() => {
      window.setTimeout(() => setOptimisticWatched((cur) => (cur === newWatched ? null : cur)), 500);
    });
  };

  // Files numbered past the matched episode count (e.g. a 25th file on a
  // 24-episode series) are almost always misnamed, duplicates, or stray
  // specials. Split them out so they don't masquerade as real episodes and
  // are easy to spot and fix — but only when we actually know the expected
  // count, and only for the main season so multi-season folders (whose
  // episode numbers reset) aren't false-flagged.
  const extraEpisodes = (!isMovie && totalEpisodes != null && totalEpisodes > 0)
    ? sorted.filter((f) => (f.seasonNumber == null || f.seasonNumber <= 1) && f.episodeNumber > totalEpisodes)
    : [];
  const extraPaths = new Set(extraEpisodes.map((f) => f.filePath));
  const regularEpisodes = extraEpisodes.length > 0
    ? sorted.filter((f) => !extraPaths.has(f.filePath))
    : sorted;

  // Shared row renderer so the main list and the "Extra files" list stay in
  // lockstep. `extra` adds the warning pill and suppresses the "Next up"
  // marker (an out-of-range file is never the next episode to watch).
  const renderEpisodeRow = (f: (typeof sorted)[number], opts?: { extra?: boolean }) => {
    // A movie has no episodes — the file IS the movie. Render it as a single
    // "Movie" row with no episode code, next-up, or extra flagging. Without
    // this, a filename like "…Dai 63-kai…" parses to a bogus episode 63 and
    // gets flagged as an extra file. No marker track/untrack cascade either —
    // a movie's tracker progress isn't episode-indexed.
    if (isMovie) {
      const movieWatched = listStatus === "completed" || (watched != null && watched > 0);
      const fraction = getProgressFraction(localProgress, item.id, f.episodeNumber);
      const isTranscoded = transcodedByPath.has(f.filePath);
      return (
        <EpisodeRow
          key={f.filePath}
          marker={movieWatched ? <Check size={14} strokeWidth={2.5} /> : <Play size={14} />}
          code="Movie"
          title={sorted.length === 1 ? displayTitle : f.title}
          trailing={
            (isTranscoded || movieWatched) ? (
              <>
                {isTranscoded && (
                  <Tooltip label="Source codec wasn't browser-playable — this episode plays from the on-disk h.264 transcode cache">
                    <span>
                      <Pill tone="amber">Re-encoded</Pill>
                    </span>
                  </Tooltip>
                )}
                {movieWatched && <Pill tone="muted">Watched</Pill>}
              </>
            ) : null
          }
          progress={fraction}
          state={movieWatched ? "watched" : "default"}
          onClick={() =>
            navigate(`/player/${encodeURIComponent(item.id)}/${f.episodeNumber}`)
          }
        />
      );
    }
    const isExtra = opts?.extra ?? false;
    const isWatched = effTrackedKnown && f.episodeNumber <= effWatched;
    const isNext = !isExtra && nextEpNumber != null && f.episodeNumber === nextEpNumber;
    const code = formatEpisodeCode({
      episodeNumber: f.episodeNumber,
      seasonNumber: f.seasonNumber,
    });
    // Fraction in [0, 1] from localStorage — set by the player every 4s and
    // on pause. Zero for episodes never started OR finished (entry deleted).
    const fraction = getProgressFraction(localProgress, item.id, f.episodeNumber);
    const isTranscoded = transcodedByPath.has(f.filePath);
    const statusPill =
      isNext ? <Pill tone="accent">Next up</Pill> :
      isWatched ? <Pill tone="muted">Watched</Pill> :
      null;
    // Prefer the canonical API title over the noisy filename-derived one.
    const episodeTitle = apiTitleByEp.get(f.episodeNumber) ?? f.title;
    const ep = f.episodeNumber;
    const inWave = wave != null && ep >= wave.lo && ep <= wave.hi;
    const markerMode = inWave ? wave!.mode : undefined;
    const markerPhase = inWave ? wave!.phase : undefined;
    const markerCascadeDelayMs = inWave
      ? Math.min(wave!.mode === "untrack" ? ep - wave!.anchor : wave!.anchor - ep, 12) * 45
      : 0;
    const trailing = (isExtra || isTranscoded || statusPill) ? (
      <>
        {isExtra && <Pill tone="amber">Extra</Pill>}
        {isTranscoded && (
          <Tooltip label="Source codec wasn't browser-playable — this episode plays from the on-disk h.264 transcode cache">
            <span>
              <Pill tone="amber">Re-encoded</Pill>
            </span>
          </Tooltip>
        )}
        {statusPill}
      </>
    ) : null;
    return (
      <EpisodeRow
        key={f.filePath}
        marker={isWatched ? <Check size={14} strokeWidth={2.5} /> : <Play size={14} />}
        code={code}
        title={episodeTitle}
        trailing={trailing}
        progress={fraction}
        state={isNext ? "next-up" : isWatched ? "watched" : "default"}
        onClick={() =>
          navigate(`/player/${encodeURIComponent(item.id)}/${f.episodeNumber}`)
        }
        // Markers double as a track/untrack control. Watched: "untrack to here"
        // (set progress to this ep − 1). Untracked: "track to here" (set to this
        // ep). Hovering paints the cascade range.
        markerTooltip={
          hasTrackerId ? (isWatched ? "untrack to here" : "track to here") : undefined
        }
        markerMode={markerMode}
        markerPhase={markerPhase}
        markerCascadeDelayMs={markerCascadeDelayMs}
        onMarkerClick={hasTrackerId ? () => onMarkerClick(ep, isWatched) : undefined}
        onMarkerEnter={hasTrackerId ? () => onMarkerEnter(ep, isWatched) : undefined}
        onMarkerZoneEnter={hasTrackerId ? onMarkerZoneEnter : undefined}
        onMarkerLeave={hasTrackerId ? onMarkerLeave : undefined}
      />
    );
  };

  return (
    <Page>
      <div className="series-detail-bare">
        <button className="detail-back" onClick={() => navigate("/")}>
          <ArrowLeft size={14} />
          <span>Library</span>
        </button>

      <section
        className={`series-hero${bannerUrl ? " has-banner" : ""}`}
        style={bannerUrl ? { ["--hero-banner" as string]: `url("${bannerUrl}")` } : undefined}
      >
        {bannerUrl && <div className="series-hero-banner" aria-hidden="true" />}
        <div className="series-hero-inner">
          <div className="series-hero-poster">
            {posterUrl ? (
              <img
                src={posterUrl}
                alt={displayTitle}
                loading="lazy"
                decoding="async"
                onError={(e) => {
                  const t = e.target as HTMLImageElement;
                  t.style.display = "none";
                }}
              />
            ) : (
              <div className="series-hero-poster-empty">
                {isMovie ? <Film size={40} /> : <Tv size={40} />}
              </div>
            )}
          </div>

          <div className="series-hero-body">
            <h1 className="series-hero-title">{displayTitle}</h1>
            {altTitle && <p className="series-hero-alt-title">{altTitle}</p>}

            <div className="series-hero-chips series-hero-chips--ratings">
              {rating && (
                <Tooltip label="Average rating">
                  <span className="hero-chip hero-chip-rating">
                    <Star size={12} strokeWidth={2.25} />
                    {rating}
                  </span>
                </Tooltip>
              )}
              {hasTrackerId && (
                <span className="hero-chip-myscore-anchor">
                  <Tooltip label={userScoreLabel ? "Click to change your rating" : "Click to rate this show"}>
                    <button
                      type="button"
                      className="hero-chip hero-chip-myscore hero-chip-myscore-button"
                      onClick={() => {
                        setScoreDraft(userScore != null ? userScore.toFixed(1) : '8.0');
                        setScoreError(null);
                        setScoreEditing((v) => !v);
                      }}
                    >
                      <Star size={12} strokeWidth={2.25} />
                      {userScoreLabel ?? 'Rate'}
                      <span className="hero-chip-myscore-tag">YOU</span>
                    </button>
                  </Tooltip>
                  {scoreEditing && (
                    <div className="hero-score-popover" role="dialog">
                      <ScorePicker
                        value={scoreDraft}
                        onChange={setScoreDraft}
                        disabled={scoreBusy}
                        ariaLabel="Your rating"
                      />
                      <button
                        type="button"
                        className="hero-score-submit"
                        onClick={() => void submitSeriesScore(scoreDraft)}
                        disabled={scoreBusy}
                      >
                        {scoreBusy ? 'Saving…' : 'Save'}
                      </button>
                      {userScore != null && (
                        <Tooltip label="Clear rating">
                          <button
                            type="button"
                            className="hero-score-clear"
                            onClick={() => void submitSeriesScore('0')}
                            disabled={scoreBusy}
                          >
                            Clear
                          </button>
                        </Tooltip>
                      )}
                      {scoreError && <span className="hero-score-error">{scoreError}</span>}
                    </div>
                  )}
                </span>
              )}
              {item.anilistId != null && (
                <Tooltip label="Open on AniList">
                  <button
                    type="button"
                    className="hero-chip hero-chip-anilist hero-chip-anilist--icon"
                    aria-label="Open on AniList"
                    onClick={() => {
                      void window.electronAPI.openExternal(
                        `https://anilist.co/anime/${item.anilistId}`,
                      );
                    }}
                  >
                    <AniListIcon size={14} />
                  </button>
                </Tooltip>
              )}
            </div>

            <div className="series-hero-chips series-hero-chips--info">
              <span className="hero-chip">{formatLabel}</span>
              {year && <span className="hero-chip">{year}</span>}
              {!isMovie && totalEpisodes != null && (
                <span className="hero-chip">{totalEpisodes} ep</span>
              )}
              {studioName && (
                <Tooltip label="Animation studio">
                  <span className="hero-chip hero-chip-studio">{studioName}</span>
                </Tooltip>
              )}
              {statusLabel && (
                <span className={`hero-chip hero-chip-status status-${normalizeStatus(item.status ?? meta?.status ?? null)}`}>
                  {statusLabel}
                </span>
              )}
              {nextUpcoming && (
                <Tooltip label={`Episode ${nextUpcoming.episodeNumber} airs ${new Date(nextUpcoming.airDateMs).toLocaleString()}`}>
                  <span className="hero-chip hero-chip-next-ep">
                    <Clock size={12} strokeWidth={2.25} />
                    EP {String(nextUpcoming.episodeNumber).padStart(2, "0")} in {formatCountdown(nextUpcoming.airDateMs - nowMs)}
                  </span>
                </Tooltip>
              )}
              {listStatus && (
                <Tooltip label={`On your list: ${LIST_STATUS_LABEL[listStatus]}`}>
                  <span className={`hero-chip hero-chip-list list-${listStatus}`}>
                    {LIST_STATUS_LABEL[listStatus]}
                  </span>
                </Tooltip>
              )}
              {rewatchCount != null && rewatchCount > 0 && (
                <Tooltip label={`Rewatched ${rewatchCount}${rewatchCount === 1 ? " time" : " times"}`}>
                  <span className="hero-chip hero-chip-rewatch">
                    <RotateCw size={12} strokeWidth={2.25} />
                    {rewatchCount}× rewatched
                  </span>
                </Tooltip>
              )}
            </div>

            {description && (
              <p className="series-hero-desc">{description}</p>
            )}

            {!isMovie && (
              <div className="series-hero-progress">
                <div className="series-hero-progress-meta">
                  <span className="series-hero-progress-label">
                    {trackedKnown ? "Tracked" : "Not tracked"}
                  </span>
                  <span className="series-hero-progress-count">
                    {trackedKnown
                      ? `${String(watchedCount).padStart(String(denom || 1).length, "0")} / ${denom > 0 ? denom : "?"}${denomIsAiringEstimate ? "+" : ""}`
                      : `${sorted.length} on disk`}
                  </span>
                </div>
                <div className="series-hero-progress-track" aria-hidden="true">
                  <div
                    className={`series-hero-progress-fill${trackedKnown ? "" : " untracked"}`}
                    style={{ width: `${trackedKnown ? progressPct : 0}%` }}
                  />
                </div>
              </div>
            )}
          </div>

          {allTags.length > 0 && (
            <aside className="series-hero-tags" aria-label="Tags">
              <div className="series-hero-tags-head">
                <span className="series-hero-tags-label">Tags</span>
                {hasHiddenSpoilerTags && (
                  <Tooltip label={showSpoilerTags ? "Hide spoiler tags" : "Show spoiler tags"}>
                    <button
                      type="button"
                      className={`series-hero-tags-toggle${showSpoilerTags ? " is-active" : ""}`}
                      aria-pressed={showSpoilerTags}
                      onClick={() => setShowSpoilerTags((v) => !v)}
                    >
                      {showSpoilerTags ? <EyeOff size={12} strokeWidth={2.25} /> : <Eye size={12} strokeWidth={2.25} />}
                      <span>{showSpoilerTags ? "Hide spoilers" : "Show spoilers"}</span>
                    </button>
                  </Tooltip>
                )}
              </div>
              <ul className="series-hero-tags-list">
                {visibleTags.map((t) => {
                  const isSpoilery = t.isMediaSpoiler || t.isGeneralSpoiler || t.isAdult;
                  return (
                    <li key={t.name} className={`series-hero-tag${isSpoilery ? " is-spoiler" : ""}`}>
                      <span className="series-hero-tag-name">{t.name}</span>
                      {typeof t.rank === "number" && (
                        <span className="series-hero-tag-rank">{t.rank}</span>
                      )}
                    </li>
                  );
                })}
              </ul>
            </aside>
          )}
        </div>
      </section>

      <Section
        first
        title={isMovie ? "Movie" : "Episodes"}
        count={isMovie ? (sorted.length > 1 ? sorted.length : undefined) : regularEpisodes.length}
      >
        <div className="episode-list">
          {regularEpisodes.map((f) => renderEpisodeRow(f))}
        </div>
      </Section>

      {extraEpisodes.length > 0 && (
        <Section title="Extra files" count={extraEpisodes.length}>
          <div className="extra-episodes-note" role="note">
            <AlertTriangle size={16} aria-hidden="true" />
            <span>
              {extraEpisodes.length === 1 ? "1 file goes" : `${extraEpisodes.length} files go`}{" "}
              beyond the expected {totalEpisodes} episode{totalEpisodes === 1 ? "" : "s"} for this
              title — likely misnamed, duplicates, or specials. Review them and rename or remove
              what doesn't belong.
            </span>
          </div>
          <div className="episode-list">
            {extraEpisodes.map((f) => renderEpisodeRow(f, { extra: true }))}
          </div>
        </Section>
      )}

      {characters.length > 0 && (
        <Section title="Characters" count={characters.length}>
          <div className="character-grid">
            {characters.map((c) => (
              <a
                key={c.anilistId}
                className="character-card"
                href={c.siteUrl ?? `https://anilist.co/character/${c.anilistId}`}
                onClick={(e) => {
                  e.preventDefault();
                  const url = c.siteUrl ?? `https://anilist.co/character/${c.anilistId}`;
                  void window.electronAPI.openExternal(url);
                }}
              >
                <div className="character-card-portrait">
                  {c.image ? (
                    <img src={c.image} alt={c.name ?? "Character"} loading="lazy" decoding="async" />
                  ) : (
                    <div className="character-card-portrait-empty">
                      <Users size={24} />
                    </div>
                  )}
                </div>
                <div className="character-card-body">
                  <div className="character-card-name">{c.name ?? "Unknown"}</div>
                  {c.role && (
                    <div className="character-card-role">{c.role.toLowerCase()}</div>
                  )}
                </div>
              </a>
            ))}
          </div>
        </Section>
      )}

      {(franchiseGraph?.nodes.length ?? 0) > 1 && meta?.anilistId != null && (
        <Section title="Related" count={(franchiseGraph?.nodes.length ?? 1) - 1}>
          <FranchiseGraphView
            graph={franchiseGraph!}
            currentAnilistId={meta.anilistId}
            filling={franchiseFilling}
            resolveOwnedId={resolveOwnedNode}
            pickTitle={(n) => pickTitle({
              titleRomaji: n.titleRomaji,
              titleEnglish: n.titleEnglish,
              folderName: n.titleRomaji ?? n.titleEnglish ?? "Untitled",
            })}
            onOpenInApp={(id) => navigate(`/series/${encodeURIComponent(id)}`)}
            onOpenExternal={openExternalNode}
            statusMarkerFor={(n) => listStatusMarker({ anilistId: n.anilistId, malId: n.malId })}
            anilistIcon={<AniListIcon size={11} />}
            hiddenCategories={hiddenCategories}
            onToggleCategory={(cat) =>
              setHiddenCategories((prev) => {
                const n = new Set(prev);
                if (n.has(cat)) n.delete(cat);
                else n.add(cat);
                return n;
              })
            }
            hiddenFormats={hiddenFormats}
            onToggleFormat={toggleFormat}
          />
        </Section>
      )}

      {recommendations.length > 0 && (
        <Section title="Recommendations" count={recommendations.length}>
          <div className="relations-grid relations-grid--single-row">
            {recommendations.map((r) => {
              const recTitle = pickTitle({
                titleRomaji: r.titleRomaji,
                titleEnglish: r.titleEnglish,
                folderName: r.titleRomaji ?? r.titleEnglish ?? "Untitled",
              });
              const fmtLabel = relationFormatLabel(r.format);
              // Same library resolution as the Related strip: if the
              // recommended entry is an ANIME the user already owns, route
              // in-app instead of bouncing out to AniList.
              const ownedId = r.type === "ANIME"
                ? (r.anilistId != null ? ownedByExternalId.byAnilist.get(r.anilistId) : undefined)
                  ?? (r.malId != null ? ownedByExternalId.byMal.get(r.malId) : undefined)
                : undefined;
              const handleClick = () => {
                if (ownedId) {
                  navigate(`/series/${encodeURIComponent(ownedId)}`);
                  return;
                }
                const url = r.siteUrl
                  ?? (r.type === "MANGA"
                    ? `https://anilist.co/manga/${r.anilistId}`
                    : `https://anilist.co/anime/${r.anilistId}`);
                if (url) void window.electronAPI.openExternal(url);
              };
              const mark = listStatusMarker({ anilistId: r.anilistId, malId: r.malId });
              return (
                <Card
                  key={`rec-${r.anilistId}`}
                  variant={ownedId ? "internal" : "external"}
                  onClick={handleClick}
                  tooltip={ownedId ? `Open ${recTitle} in your library` : `Open ${recTitle} on AniList`}
                  data-format={r.format ?? ""}
                >
                  <div className="relation-card-poster">
                    {r.poster ? (
                      <img src={r.poster} alt={recTitle} loading="lazy" decoding="async" />
                    ) : (
                      <div className="relation-card-poster-empty">
                        {r.type === "MANGA" ? <Film size={28} /> : <Tv size={28} />}
                      </div>
                    )}
                    <span aria-hidden="true">
                      {ownedId ? (
                        <Pill tone="teal">Available</Pill>
                      ) : (
                        <Pill tone="accent">
                          <AniListIcon size={11} />
                          AniList
                        </Pill>
                      )}
                    </span>
                  </div>
                  <div className="relation-card-body">
                    <div className="relation-card-type">Recommended</div>
                    <div className="relation-card-title">{recTitle}</div>
                    <div className="relation-card-meta">
                      {fmtLabel && (
                        <span className="relation-card-format" data-format={r.format ?? ""}>
                          {fmtLabel}
                        </span>
                      )}
                      {r.seasonYear && <span>{r.seasonYear}</span>}
                    </div>
                  </div>
                  {mark}
                </Card>
              );
            })}
          </div>
        </Section>
      )}
      </div>
    </Page>
  );
}

export default SeriesDetailPage;
