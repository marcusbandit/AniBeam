import { useNavigate } from "react-router-dom";
import type { SeriesMetadata } from "../hooks/useMetadata";
import type { FileStatus } from "../../shared/fileStatus";
import { Film, Tv } from "lucide-react";
import { getDisplayRating } from "../utils/ratingUtils";
import { formatEpisodeCode, formatRelativeDate, getLatestAiredEpisode } from "../utils/airingUtils";

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
  const isMovie = seriesData.type === "movie" || seriesData.format === "MOVIE";

  const handleClick = () => {
    navigate(`/series/${seriesId}`);
  };

  const totalEpisodes = seriesData.totalEpisodes || seriesData.episodes?.length || 0;
  const downloadedEpisodes = seriesData.fileEpisodes?.length || 0;
  const posterUrl = getImageUrl(seriesData.posterLocal, seriesData.poster);

  const files = (seriesData.fileEpisodes ?? []) as Array<{ status?: FileStatus }>;
  const order: Record<FileStatus, number> = { ready: 0, verifying: 1, stalled: 2 };
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
        {!isMovie && totalEpisodes > 0 && (
          <div className="show-card-badge">
            <span className="badge-have">{downloadedEpisodes}</span>
            <span className="badge-sep">/</span>
            <span className="badge-total">{totalEpisodes}</span>
          </div>
        )}
        {!isReady && (
          <div className={`status-badge status-badge-${aggregateStatus}`}>
            {aggregateStatus === "verifying" ? "VERIFYING" : "STALLED"}
          </div>
        )}
      </div>
      <div className="show-card-info">
        <div className="show-card-title">{seriesData.title}</div>
        {variant === "feed" ? (
          <div className="show-card-meta">
            {score && <span className="show-card-score">{score}</span>}
            {epCode && <span className="show-card-ep">{epCode}</span>}
            {epRel && <span className="show-card-rel">{epRel}</span>}
          </div>
        ) : (
          <div className="show-card-meta">
            {score && <span className="show-card-score">{score}</span>}
            {year && <span className="show-card-year">{year}</span>}
            {firstGenre && <span className="show-card-genre">{firstGenre}</span>}
          </div>
        )}
      </div>
    </button>
  );
}

export default ShowCard;
