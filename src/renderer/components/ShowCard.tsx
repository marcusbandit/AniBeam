import { useNavigate } from "react-router-dom";
import type { SeriesMetadata } from "../hooks/useMetadata";
import type { FileStatus } from "../../shared/fileStatus";
import { Film, Tv } from "lucide-react";
import { getDisplayRating } from "../utils/ratingUtils";
import { classifyWatchProgress, formatEpisodeCode, formatRelativeDate, formatWatchedLabel, getLatestAiredEpisode } from "../utils/airingUtils";
import { useTrackerProgress } from "../contexts/TrackerProgressContext";

function getImageUrl(localPath?: string | null, remotePath?: string | null): string | null {
  if (localPath) return `media://${encodeURIComponent(localPath)}`;
  return remotePath || null;
}

function getYear(startDate?: string | null, seasonYear?: number | null): number | null {
  if (seasonYear) return seasonYear;
  if (!startDate) return null;
  const year = parseInt(startDate.split("-")[0], 10);
  return isNaN(year) ? null : year;
}

interface ShowCardProps {
  seriesId: string;
  seriesData: SeriesMetadata;
  size?: "normal" | "large";
  variant?: "library" | "feed";
}

function ShowCard({ seriesId, seriesData, variant = "library" }: ShowCardProps) {
  const navigate = useNavigate();
  const { getWatched } = useTrackerProgress();
  const isMovie = seriesData.type === "movie" || seriesData.format === "MOVIE";

  const handleClick = () => {
    navigate(`/series/${seriesId}`);
  };

  // For the badge denominator we trust seriesData.totalEpisodes only — the
  // episodes-array length isn't a real total for airing shows (it counts
  // released-with-metadata episodes). formatWatchedLabel handles the null
  // case by rendering "XX/?" so the format stays consistent.
  const totalEpisodes = seriesData.totalEpisodes ?? null;
  // Used by the !isMovie corner badge below — that one wants the best
  // denominator we can show, so fall back to episodes.length when total
  // isn't published yet.
  const cornerTotal = seriesData.totalEpisodes || seriesData.episodes?.length || 0;
  const downloadedEpisodes = seriesData.fileEpisodes?.length || 0;
  const watched = getWatched({ anilistId: seriesData.anilistId, malId: seriesData.malId });
  const latestAiredNum = getLatestAiredEpisode(seriesData)?.episodeNumber ?? null;
  const watchedState = watched != null
    ? classifyWatchProgress({ watched, totalEpisodes, latestAiredEpisode: latestAiredNum })
    : null;
  const watchedLabel = formatWatchedLabel({
    watched,
    totalEpisodes,
    latestAiredEpisode: latestAiredNum,
    state: watchedState,
  });
  const posterUrl = getImageUrl(seriesData.posterLocal, seriesData.poster);

  const files = (seriesData.fileEpisodes ?? []) as Array<{ status?: FileStatus }>;
  // 'transcoding' ranks above 'verifying' (it's a longer-running and
  // more user-visible step) but below 'stalled'. The badge below shows
  // the worst status across all files in the series.
  const order: Record<FileStatus, number> = { ready: 0, verifying: 1, transcoding: 2, stalled: 3 };
  const aggregateStatus: FileStatus = files.reduce<FileStatus>((acc, f) => {
    const s = (f.status ?? 'ready') as FileStatus;
    return order[s] > order[acc] ? s : acc;
  }, 'ready');
  const isReady = aggregateStatus === 'ready';

  const score = seriesData.averageScore
    ? getDisplayRating(seriesData.averageScore, seriesData.source)
    : null;
  const year = getYear(seriesData.startDate, seriesData.seasonYear);
  const firstGenre = seriesData.genres?.[0];

  // Feed variant: latest aired episode + relative date
  const latestEp = variant === "feed" ? getLatestAiredEpisode(seriesData) : null;
  const epCode = latestEp ? formatEpisodeCode(latestEp) : "";
  const epRel = latestEp?.airDate ? formatRelativeDate(latestEp.airDate) : "";

  return (
    <button className={`show-card${isReady ? "" : " not-ready"} status-${aggregateStatus}`} onClick={handleClick}>
      <div className="show-card-poster-wrap">
        {posterUrl ? (
          <img
            src={posterUrl}
            alt={seriesData.title || "Show poster"}
            className="show-card-poster"
            loading="lazy"
            onError={(e) => {
              const target = e.target as HTMLImageElement;
              if (seriesData.poster && target.src !== seriesData.poster) {
                target.src = seriesData.poster;
              } else {
                target.style.display = "none";
              }
            }}
          />
        ) : (
          <div className="show-card-no-image">
            {isMovie ? <Film size={48} /> : <Tv size={48} />}
          </div>
        )}
        {!isMovie && cornerTotal > 0 && (
          <div className="show-card-badge">
            <span className="badge-have">{downloadedEpisodes}</span>
            <span className="badge-sep">/</span>
            <span className="badge-total">{cornerTotal}</span>
          </div>
        )}
        {!isReady && (
          <div className={`status-badge status-badge-${aggregateStatus}`}>
            {aggregateStatus === "verifying" ? "VERIFYING"
              : aggregateStatus === "transcoding" ? "TRANSCODING"
              : "STALLED"}
          </div>
        )}
      </div>
      <div className="show-card-info">
        <div className="show-card-title">{seriesData.title}</div>
        {variant === "feed" ? (
          <div className="show-card-meta">
            {score && <span className="show-card-score">{score}</span>}
            {watchedLabel && <span className={`show-card-watched${watchedState ? ` ${watchedState}` : ''}`}>{watchedLabel}</span>}
            {epCode && <span className="show-card-ep">{epCode}</span>}
            {epRel && <span className="show-card-rel">{epRel}</span>}
          </div>
        ) : (
          <div className="show-card-meta">
            {score && <span className="show-card-score">{score}</span>}
            {watchedLabel && <span className={`show-card-watched${watchedState ? ` ${watchedState}` : ''}`}>{watchedLabel}</span>}
            {year && <span className="show-card-year">{year}</span>}
            {firstGenre && <span className="show-card-genre">{firstGenre}</span>}
          </div>
        )}
      </div>
    </button>
  );
}

export default ShowCard;
