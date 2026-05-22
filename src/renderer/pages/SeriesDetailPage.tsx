import { useEffect, useMemo, useState, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Play, Check, Star, Tv, Film, Clock, ExternalLink } from "lucide-react";
import type { LibraryItem } from "../../types/electron";
import type { Relation, SeriesMetadata } from "../hooks/useMetadata";
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
import { Page, Section, Card, EpisodeRow, Pill } from "../components/primitives";

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

// Display label for an AniList `relationType`. Falls back to the raw
// SCREAMING_SNAKE form so unknown values stay debuggable instead of
// silently rendering "".
const RELATION_LABEL: Record<string, string> = {
  SEQUEL: "Sequel",
  PREQUEL: "Prequel",
  PARENT: "Parent story",
  SIDE_STORY: "Side story",
  SUMMARY: "Summary",
  ALTERNATIVE: "Alternative",
  SPIN_OFF: "Spin-off",
  COMPILATION: "Compilation",
  ADAPTATION: "Source",
  OTHER: "Other",
  CONTAINS: "Contains",
};

// Lower index = render earlier. Story-progression edges first (prequels
// then sequels), then companion entries (specials, spin-offs, etc.),
// then source-media adaptations, then anything else. Stable secondary
// sort by `seasonYear` ascending so a multi-season franchise renders
// chronologically left-to-right.
const RELATION_ORDER: Record<string, number> = {
  PREQUEL: 0,
  SEQUEL: 1,
  PARENT: 2,
  SIDE_STORY: 3,
  ALTERNATIVE: 4,
  SPIN_OFF: 5,
  SUMMARY: 6,
  COMPILATION: 7,
  ADAPTATION: 8,
  OTHER: 9,
  CONTAINS: 10,
};

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

function sortRelations(relations: ReadonlyArray<Relation>): Relation[] {
  return [...relations].sort((a, b) => {
    const ai = RELATION_ORDER[a.relationType] ?? 99;
    const bi = RELATION_ORDER[b.relationType] ?? 99;
    if (ai !== bi) return ai - bi;
    const ay = a.seasonYear ?? Number.POSITIVE_INFINITY;
    const by = b.seasonYear ?? Number.POSITIVE_INFINITY;
    return ay - by;
  });
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
  const { getWatched, getListStatus, getUserScore } = useTrackerProgress();

  const [item, setItem] = useState<LibraryItem | null>(null);
  const [meta, setMeta] = useState<SeriesMetadata | null>(null);
  // Full library list (not just the current series) — needed to resolve
  // relation clicks. If a related entry's anilistId/malId matches a
  // series the user has on disk, the click navigates in-app; otherwise
  // we open AniList in the system browser.
  const [allItems, setAllItems] = useState<LibraryItem[]>([]);
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

  useEffect(() => {
    const refresh = () => {
      setLocalProgress(readProgress());
      setLastEpMap(readLastEpisodeMap());
    };
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
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
      setAllItems(fresh);
      setItem(found);
      setMeta((allMeta && typeof allMeta === "object" ? allMeta[decodedId] : null) ?? null);
    } catch (err) {
      console.error("library:walk failed", err);
      setAllItems([]);
      setItem(null);
      setMeta(null);
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

  const sortedRelations = useMemo(
    () => sortRelations(meta?.relations ?? []),
    [meta?.relations],
  );

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
  const watchedCount = typeof watched === "number" ? watched : 0;
  const trackedKnown = watched != null;
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
    trackedKnown ? watchedCount : 0,
    lastEpLocal ?? 0,
  );
  const nextEpNumber = effectiveLastWatched > 0
    ? sorted.find((f) => f.episodeNumber === effectiveLastWatched + 1)?.episodeNumber
        ?? sorted.find((f) => f.episodeNumber > effectiveLastWatched)?.episodeNumber
        ?? null
    : (trackedKnown ? sorted.find((f) => f.episodeNumber > watchedCount)?.episodeNumber ?? null : null);

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

            <div className="series-hero-chips">
              {rating && (
                <span className="hero-chip hero-chip-rating" title="Average rating">
                  <Star size={12} strokeWidth={2.25} />
                  {rating}
                </span>
              )}
              {userScoreLabel && (
                <span className="hero-chip hero-chip-myscore" title="Your rating">
                  <Star size={12} strokeWidth={2.25} />
                  {userScoreLabel}
                  <span className="hero-chip-myscore-tag">YOU</span>
                </span>
              )}
              <span className="hero-chip">{formatLabel}</span>
              {year && <span className="hero-chip">{year}</span>}
              {!isMovie && totalEpisodes != null && (
                <span className="hero-chip">{totalEpisodes} ep</span>
              )}
              {statusLabel && (
                <span className={`hero-chip hero-chip-status status-${normalizeStatus(item.status ?? meta?.status ?? null)}`}>
                  {statusLabel}
                </span>
              )}
              {nextUpcoming && (
                <span
                  className="hero-chip hero-chip-next-ep"
                  title={`Episode ${nextUpcoming.episodeNumber} airs ${new Date(nextUpcoming.airDateMs).toLocaleString()}`}
                >
                  <Clock size={12} strokeWidth={2.25} />
                  EP {String(nextUpcoming.episodeNumber).padStart(2, "0")} in {formatCountdown(nextUpcoming.airDateMs - nowMs)}
                </span>
              )}
              {listStatus && (
                <span
                  className={`hero-chip hero-chip-list list-${listStatus}`}
                  title={`On your list: ${LIST_STATUS_LABEL[listStatus]}`}
                >
                  {LIST_STATUS_LABEL[listStatus]}
                </span>
              )}
              {item.anilistId != null && (
                <button
                  type="button"
                  className="hero-chip hero-chip-anilist"
                  title="Open on AniList"
                  onClick={() => {
                    void window.electronAPI.openExternal(
                      `https://anilist.co/anime/${item.anilistId}`,
                    );
                  }}
                >
                  <ExternalLink size={12} strokeWidth={2.25} />
                  AniList
                </button>
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
        </div>
      </section>

      <Section first title="Episodes" count={sorted.length}>
        <div className="episode-list">
          {sorted.map((f) => {
            const isWatched = trackedKnown && f.episodeNumber <= watchedCount;
            const isNext = nextEpNumber != null && f.episodeNumber === nextEpNumber;
            const code = formatEpisodeCode({
              episodeNumber: f.episodeNumber,
              seasonNumber: f.seasonNumber,
            });
            // Fraction in [0, 1] from localStorage — set by the player every
            // 4s and on pause. Zero for episodes that were never started OR
            // that finished (entry is deleted on completion).
            const fraction = getProgressFraction(localProgress, item.id, f.episodeNumber);
            return (
              <EpisodeRow
                key={f.filePath}
                marker={isWatched ? <Check size={14} strokeWidth={2.5} /> : <Play size={14} />}
                code={code}
                title={f.title}
                trailing={
                  isNext ? <Pill tone="accent">Next up</Pill> :
                  isWatched ? <Pill tone="muted">Watched</Pill> :
                  null
                }
                progress={fraction}
                state={isNext ? "next-up" : isWatched ? "watched" : "default"}
                onClick={() =>
                  navigate(`/player/${encodeURIComponent(item.id)}/${f.episodeNumber}`)
                }
              />
            );
          })}
        </div>
      </Section>

      {sortedRelations.length > 0 && (
        <Section title="Related" count={sortedRelations.length}>
          <div className="relations-grid">
            {sortedRelations.map((rel) => {
              const ownedId = rel.type === "ANIME"
                ? (rel.anilistId != null ? ownedByExternalId.byAnilist.get(rel.anilistId) : undefined)
                  ?? (rel.malId != null ? ownedByExternalId.byMal.get(rel.malId) : undefined)
                : undefined;
              const relTitle = pickTitle({
                titleRomaji: rel.titleRomaji,
                titleEnglish: rel.titleEnglish,
                folderName: rel.titleRomaji ?? rel.titleEnglish ?? "Untitled",
              });
              const typeLabel = RELATION_LABEL[rel.relationType]
                ?? rel.relationType.replace(/_/g, " ").toLowerCase();
              const formatLabel = relationFormatLabel(rel.format);
              const isInternal = ownedId != null;
              const handleClick = () => {
                if (isInternal && ownedId) {
                  navigate(`/series/${encodeURIComponent(ownedId)}`);
                  return;
                }
                const url = rel.siteUrl
                  ?? (rel.type === "MANGA"
                    ? `https://anilist.co/manga/${rel.anilistId}`
                    : `https://anilist.co/anime/${rel.anilistId}`);
                if (url) void window.electronAPI.openExternal(url);
              };
              return (
                <Card
                  key={`${rel.type ?? "x"}-${rel.anilistId}-${rel.relationType}`}
                  variant={isInternal ? "internal" : "external"}
                  onClick={handleClick}
                  title={isInternal
                    ? `Open ${relTitle} in your library`
                    : `Open ${relTitle} on AniList`}
                >
                  <div className="relation-card-poster">
                    {rel.poster ? (
                      <img src={rel.poster} alt={relTitle} loading="lazy" decoding="async" />
                    ) : (
                      <div className="relation-card-poster-empty">
                        {rel.type === "MANGA" ? <Film size={28} /> : <Tv size={28} />}
                      </div>
                    )}
                    <span aria-hidden="true">
                      <Pill tone={isInternal ? "teal" : "accent"}>
                        {isInternal ? "In Library" : (
                          <>
                            <ExternalLink size={10} strokeWidth={2.5} />
                            AniList
                          </>
                        )}
                      </Pill>
                    </span>
                  </div>
                  <div className="relation-card-body">
                    <div className="relation-card-type">{typeLabel}</div>
                    <div className="relation-card-title" title={relTitle}>{relTitle}</div>
                    <div className="relation-card-meta">
                      {formatLabel && <span>{formatLabel}</span>}
                      {rel.seasonYear && <span>{rel.seasonYear}</span>}
                    </div>
                  </div>
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
