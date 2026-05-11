import { useEffect, useMemo, useState, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Play, Check, Star, Tv, Film } from "lucide-react";
import type { LibraryItem } from "../../types/electron";
import type { SeriesMetadata } from "../hooks/useMetadata";
import { useDebouncedCallback } from "../hooks/useDebouncedCallback";
import { useTitleLanguage } from "../contexts/TitleLanguageContext";
import { useTrackerProgress } from "../contexts/TrackerProgressContext";
import { getDisplayRating } from "../utils/ratingUtils";
import { formatEpisodeCode, getLatestAiredEpisodeNumber, normalizeStatus } from "../utils/airingUtils";
import type { TrackerListStatus } from "../../main/preload";

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
  const { getWatched, getListStatus } = useTrackerProgress();

  const [item, setItem] = useState<LibraryItem | null>(null);
  const [meta, setMeta] = useState<SeriesMetadata | null>(null);
  const [initialLoading, setInitialLoading] = useState(true);

  const decodedId = seriesId ? decodeURIComponent(seriesId) : "";

  // No setLoading toggle on reload pings — keeps the file list visible
  // while the background match updates posters/dates incrementally.
  const reload = useCallback(async () => {
    try {
      const [all, allMeta] = await Promise.all([
        window.electronAPI.libraryWalk(),
        window.electronAPI.loadMetadata() as Promise<Record<string, SeriesMetadata>>,
      ]);
      const found = (Array.isArray(all) ? all : []).find((i) => i.id === decodedId) ?? null;
      setItem(found);
      setMeta((allMeta && typeof allMeta === "object" ? allMeta[decodedId] : null) ?? null);
    } catch (err) {
      console.error("library:walk failed", err);
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
  // The next-up episode is the first one with a number greater than the
  // tracker's watched count. Used to highlight where the user should pick
  // up from.
  const nextEpNumber = trackedKnown
    ? sorted.find((f) => f.episodeNumber > watchedCount)?.episodeNumber ?? null
    : null;

  return (
    <div className="page series-detail-bare">
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
                <span className="hero-chip hero-chip-rating">
                  <Star size={12} strokeWidth={2.25} />
                  {rating}
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
              {listStatus && (
                <span
                  className={`hero-chip hero-chip-list list-${listStatus}`}
                  title={`On your list: ${LIST_STATUS_LABEL[listStatus]}`}
                >
                  {LIST_STATUS_LABEL[listStatus]}
                </span>
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

      <div className="bare-episode-head">
        <h2 className="section-h2">Episodes</h2>
        <span className="section-count">{sorted.length}</span>
      </div>

      <div className="bare-episode-list">
        {sorted.map((f) => {
          const isWatched = trackedKnown && f.episodeNumber <= watchedCount;
          const isNext = nextEpNumber != null && f.episodeNumber === nextEpNumber;
          const code = formatEpisodeCode({
            episodeNumber: f.episodeNumber,
            seasonNumber: f.seasonNumber,
          });
          return (
            <button
              key={f.filePath}
              type="button"
              className={`bare-episode-row${isWatched ? " watched" : ""}${isNext ? " next-up" : ""}`}
              onClick={() =>
                navigate(`/player/${encodeURIComponent(item.id)}/${f.episodeNumber}`)
              }
            >
              <span className="bare-episode-marker" aria-hidden="true">
                {isWatched ? <Check size={14} strokeWidth={2.5} /> : <Play size={14} />}
              </span>
              <span className="bare-episode-code">{code}</span>
              <span className="bare-episode-title">{f.title}</span>
              {isNext && <span className="bare-episode-pill">Next up</span>}
              {isWatched && <span className="bare-episode-flag">Watched</span>}
            </button>
          );
        })}
      </div>
    </div>
  );
}

export default SeriesDetailPage;
