import { useNavigate } from "react-router-dom";
import type { MouseEvent } from "react";
import type { EpisodeMetadata } from "../hooks/useMetadata";
import type { FileStatus } from "../../shared/fileStatus";
import { Play } from "lucide-react";
import { getProgressFraction, readProgress } from "../utils/playbackProgress";
import { formatEpisodeCode } from "../utils/airingUtils";

function getImageUrl(localPath?: string | null, remotePath?: string | null): string | null {
  if (localPath) return `media://${encodeURIComponent(localPath)}`;
  return remotePath || null;
}

function formatEpisodeNumber(episodeNumber: number): string {
  if (!Number.isInteger(episodeNumber)) return episodeNumber.toFixed(1);
  return episodeNumber.toString();
}

function isSpecial(ep: EpisodeMetadata): boolean {
  return ep.episodeNumber === 0 || ep.seasonNumber === 0;
}

interface EpisodeCardProps {
  seriesId: string;
  episode: EpisodeMetadata;
  hasFile: boolean;
}

function EpisodeCard({ seriesId, episode, hasFile }: EpisodeCardProps) {
  const navigate = useNavigate();

  const status: FileStatus = episode.status ?? 'ready';
  const filePath = episode.filePath;
  const isReady = status === 'ready';

  const handleClick = () => {
    if (hasFile && isReady) navigate(`/player/${seriesId}/${episode.episodeNumber}`);
  };

  const handleRetry = (e: MouseEvent<HTMLSpanElement>) => {
    e.stopPropagation();
    if (filePath) void window.electronAPI.probeRetry(filePath);
  };

  const thumbnailUrl = getImageUrl(episode.thumbnailLocal, episode.thumbnail);

  const code = formatEpisodeCode(episode);
  const special = isSpecial(episode);

  // Pull progress every render. localStorage parse is microseconds, and the
  // SeriesDetailPage remounts when returning from the player so the value
  // refreshes naturally without needing a context or storage event.
  const progress = hasFile && isReady
    ? getProgressFraction(readProgress(), seriesId, episode.episodeNumber)
    : 0;
  const progressPct = Math.round(progress * 100);

  return (
    <button
      className={`episode-card ${hasFile ? "has-file" : "no-file"}${isReady ? "" : " not-ready"} status-${status}`}
      onClick={handleClick}
      disabled={!hasFile || !isReady}
      data-episode-file={hasFile && filePath ? filePath : undefined}
    >
      <div className="episode-thumb">
        {thumbnailUrl ? (
          <img
            src={thumbnailUrl}
            alt={episode.title || `Episode ${episode.episodeNumber}`}
            loading="lazy"
            decoding="async"
            onError={(e) => {
              const target = e.target as HTMLImageElement;
              if (episode.thumbnail && target.src !== episode.thumbnail) {
                target.src = episode.thumbnail;
              } else {
                target.style.display = "none";
              }
            }}
          />
        ) : (
          <span className="episode-thumb-number">{special ? "SP" : formatEpisodeNumber(episode.episodeNumber).padStart(2, "0")}</span>
        )}
        {hasFile && isReady && (
          <div className="episode-play-icon">
            <Play size={20} />
          </div>
        )}
        {!isReady && (
          <span
            className={`status-badge status-badge-${status}`}
            onClick={status === "stalled" ? handleRetry : undefined}
            role={status === "stalled" ? "button" : undefined}
          >
            {status === "verifying" ? "VERIFYING" : "STALLED · RETRY"}
          </span>
        )}
        {progress > 0 && (
          <div
            className="episode-progress"
            style={{ width: `${progressPct}%` }}
            aria-label={`${progressPct}% watched`}
          />
        )}
      </div>
      <div className="episode-info">
        <div className="episode-row-top">
          <span className="episode-code">{code}</span>
          {!hasFile && <span className="episode-na">Not on disk</span>}
        </div>
        <div className="episode-title">
          {episode.title || (special ? "Special" : `Episode ${episode.episodeNumber}`)}
        </div>
        {episode.airDate && (
          <div className="episode-date">
            {new Date(episode.airDate).toLocaleDateString()}
          </div>
        )}
      </div>
    </button>
  );
}

export default EpisodeCard;
