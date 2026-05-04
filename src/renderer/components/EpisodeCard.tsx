import { useNavigate } from "react-router-dom";
import type { MouseEvent } from "react";
import type { EpisodeMetadata } from "../hooks/useMetadata";
import type { FileStatus } from "../../shared/fileStatus";
import { Play } from "lucide-react";
import { getProgressFraction, readProgress } from "../utils/playbackProgress";

function getImageUrl(localPath?: string | null, remotePath?: string | null): string | null {
  if (localPath) return `media://${encodeURIComponent(localPath)}`;
  return remotePath || null;
}

function formatEpisodeNumber(episodeNumber: number): string {
  if (!Number.isInteger(episodeNumber)) return episodeNumber.toFixed(1);
  return episodeNumber.toString();
}

function pad(n: number): string {
  return n < 10 ? `0${n}` : `${n}`;
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

  const code = episode.seasonNumber !== null && episode.seasonNumber !== undefined
    ? `S${pad(episode.seasonNumber)}E${formatEpisodeNumber(episode.episodeNumber).padStart(2, "0")}`
    : `EP ${formatEpisodeNumber(episode.episodeNumber)}`;

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
    >
      <div className="episode-thumb">
        {thumbnailUrl ? (
          <img
            src={thumbnailUrl}
            alt={episode.title || `Episode ${episode.episodeNumber}`}
            loading="lazy"
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
          <span className="episode-thumb-number">{formatEpisodeNumber(episode.episodeNumber).padStart(2, "0")}</span>
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
          {episode.title || `Episode ${episode.episodeNumber}`}
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
