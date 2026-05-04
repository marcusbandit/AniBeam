import { useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useMetadata, type EpisodeMetadata, type FileEpisode } from "../hooks/useMetadata";
import EpisodeCard from "../components/EpisodeCard";
import {
  Tv,
  Film,
  Play,
  AlertTriangle,
  ArrowLeft,
} from "lucide-react";
import { getDisplayRating } from "../utils/ratingUtils";
import { normalizeStatus } from "../utils/airingUtils";

function getImageUrl(localPath?: string | null, remotePath?: string | null): string | null {
  if (localPath) return `media://${encodeURIComponent(localPath)}`;
  return remotePath || null;
}

function formatDuration(minutes: number | null | undefined, isMovie: boolean): string | null {
  if (!minutes) return null;
  if (isMovie) {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    if (hours > 0) {
      return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
    }
    return `${mins}m`;
  } else {
    return `${minutes} min`;
  }
}

function getYear(startDate: string | null | undefined, seasonYear?: number | null): number | null {
  if (seasonYear) return seasonYear;
  if (!startDate) return null;
  const year = parseInt(startDate.split("-")[0], 10);
  return isNaN(year) ? null : year;
}

function formatEpisodeNumber(episodeNumber: number): string {
  if (!Number.isInteger(episodeNumber)) return episodeNumber.toFixed(1);
  return episodeNumber.toString();
}

function pad(n: number): string {
  return n < 10 ? `0${n}` : `${n}`;
}

function SeriesDetailPage() {
  const { seriesId } = useParams<{ seriesId: string }>();
  const navigate = useNavigate();
  const { metadata, loading, loadMetadata } = useMetadata();

  useEffect(() => {
    const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
      void loadMetadata();
    });
    return () => unsubscribe?.();
  }, [loadMetadata]);

  if (loading) {
    return (
      <div className="page">
        <div className="loading">Loading…</div>
      </div>
    );
  }

  if (!seriesId) {
    return (
      <div className="page">
        <div className="error">Series ID not provided.</div>
        <button className="btn btn-secondary" onClick={() => navigate("/")}>
          <ArrowLeft size={14} /> Library
        </button>
      </div>
    );
  }

  const seriesData = metadata[seriesId];
  if (!seriesData) {
    return (
      <div className="page">
        <div className="error">Series not found.</div>
        <button className="btn btn-secondary" onClick={() => navigate("/")}>
          <ArrowLeft size={14} /> Library
        </button>
      </div>
    );
  }

  const metadataEpisodes: EpisodeMetadata[] = seriesData.episodes || [];
  const fileEpisodes: FileEpisode[] = seriesData.fileEpisodes || [];

  const isMovie =
    seriesData.type === "movie" ||
    seriesData.format === "MOVIE" ||
    (fileEpisodes.length === 1 && (seriesData.totalEpisodes === 1 || !seriesData.totalEpisodes));

  // Group episodes by season — same logic as before
  type SeasonEpisodes = {
    seasonNumber: number | null;
    episodes: (EpisodeMetadata & { hasFile: boolean })[];
  };
  const episodesBySeason = new Map<number | null, SeasonEpisodes>();

  if (!isMovie) {
    const fileSeasons = new Set<number | null>();
    fileEpisodes.forEach((ep) => fileSeasons.add(ep.seasonNumber ?? null));

    const metadataHasSeasons = metadataEpisodes.some(
      (ep) => ep.seasonNumber !== null && ep.seasonNumber !== undefined
    );
    const defaultSeason = fileSeasons.size === 1 ? Array.from(fileSeasons)[0] : null;

    const metadataBySeason = new Map<number | null, EpisodeMetadata[]>();
    metadataEpisodes.forEach((ep) => {
      let season = ep.seasonNumber ?? null;
      if (!metadataHasSeasons && defaultSeason !== null) season = defaultSeason;
      if (!metadataBySeason.has(season)) metadataBySeason.set(season, []);
      metadataBySeason.get(season)!.push(ep);
    });

    const allSeasons = new Set<number | null>();
    metadataEpisodes.forEach((ep) => {
      const season = ep.seasonNumber ?? null;
      allSeasons.add(!metadataHasSeasons && defaultSeason !== null ? defaultSeason : season);
    });
    fileEpisodes.forEach((ep) => allSeasons.add(ep.seasonNumber ?? null));

    for (const season of allSeasons) {
      const seasonMetadata = metadataBySeason.get(season) || [];
      const seasonFiles = fileEpisodes.filter((ep) => (ep.seasonNumber ?? null) === season);

      const canonicalFileEpisodes = seasonFiles.filter((ep) => Number.isInteger(ep.episodeNumber));
      const maxCanonicalFileEpisode =
        canonicalFileEpisodes.length > 0
          ? Math.max(...canonicalFileEpisodes.map((ep) => ep.episodeNumber))
          : 0;
      const metadataMaxEpisode =
        seasonMetadata.length > 0 ? Math.max(...seasonMetadata.map((ep) => ep.episodeNumber)) : 0;
      const metadataTotalEpisodes =
        (allSeasons.size === 1 || season === defaultSeason) && seriesData.totalEpisodes
          ? seriesData.totalEpisodes
          : null;
      const seasonTotalEpisodes =
        metadataTotalEpisodes && metadataTotalEpisodes > maxCanonicalFileEpisode
          ? metadataTotalEpisodes
          : Math.max(metadataMaxEpisode, maxCanonicalFileEpisode);

      const seasonMetadataMap = new Map<number, EpisodeMetadata>();
      seasonMetadata.forEach((ep) => seasonMetadataMap.set(ep.episodeNumber, ep));

      const seasonFileMap = new Map<number, FileEpisode>();
      seasonFiles.forEach((ep) => seasonFileMap.set(ep.episodeNumber, ep));

      const seasonEpisodes: (EpisodeMetadata & { hasFile: boolean })[] = [];
      const allEpisodeNumbers = new Set<number>();
      seasonMetadata.forEach((ep) => allEpisodeNumbers.add(ep.episodeNumber));
      seasonFiles.forEach((ep) => allEpisodeNumbers.add(ep.episodeNumber));
      for (let i = 1; i <= seasonTotalEpisodes; i++) allEpisodeNumbers.add(i);

      const sortedEpisodeNumbers = Array.from(allEpisodeNumbers).sort((a, b) => a - b);

      for (const epNum of sortedEpisodeNumbers) {
        const metaEp = seasonMetadataMap.get(epNum);
        const fileEp = seasonFileMap.get(epNum);

        let episodeTitle = `Episode ${formatEpisodeNumber(epNum)}`;
        if (metaEp?.title) {
          const metaTitle = metaEp.title.trim();
          const genericPattern = /^Episode\s+\d+(\.\d+)?$/i;
          if (!genericPattern.test(metaTitle)) {
            episodeTitle = metaTitle;
          } else if (fileEp?.title) {
            const fileTitle = fileEp.title.trim();
            if (fileTitle && !/^Episode\s+\d+(\.\d+)?$/i.test(fileTitle)) {
              episodeTitle = fileTitle;
            } else {
              episodeTitle = metaTitle;
            }
          } else {
            episodeTitle = metaTitle;
          }
        } else if (fileEp?.title) {
          const fileTitle = fileEp.title.trim();
          if (fileTitle && !/^Episode\s+\d+(\.\d+)?$/i.test(fileTitle)) {
            episodeTitle = fileTitle;
          }
        }

        seasonEpisodes.push({
          episodeNumber: epNum,
          seasonNumber: season ?? undefined,
          title: episodeTitle,
          description: metaEp?.description || null,
          airDate: metaEp?.airDate || null,
          thumbnail: metaEp?.thumbnail || null,
          thumbnailLocal: metaEp?.thumbnailLocal || null,
          filePath: fileEp?.filePath,
          subtitlePath: fileEp?.subtitlePath || null,
          subtitlePaths: fileEp?.subtitlePaths || [],
          status: fileEp?.status,
          lastProbedAt: fileEp?.lastProbedAt,
          hasFile: !!fileEp,
        });
      }

      episodesBySeason.set(season, { seasonNumber: season, episodes: seasonEpisodes });
    }
  }

  const allMergedEpisodes = Array.from(episodesBySeason.values()).flatMap((s) => s.episodes);

  const availableCount = isMovie
    ? fileEpisodes.length > 0
      ? 1
      : 0
    : allMergedEpisodes.filter((ep) => ep.hasFile).length;
  const totalCount = isMovie ? 1 : allMergedEpisodes.length;
  const hasFile = fileEpisodes.length > 0;

  const posterUrl = getImageUrl(seriesData.posterLocal, seriesData.poster);

  const score = seriesData.averageScore
    ? getDisplayRating(seriesData.averageScore, seriesData.source)
    : null;
  const year = getYear(seriesData.startDate, seriesData.seasonYear);
  const durationText = formatDuration(seriesData.duration, isMovie);
  const studios = seriesData.studios && seriesData.studios.length > 0 ? seriesData.studios.join(", ") : null;

  const firstAvailableEpisode = allMergedEpisodes.find((ep) => ep.hasFile);
  const handlePlayMovie = () => navigate(`/player/${seriesId}/1`);
  const handlePlaySeries = () => {
    if (firstAvailableEpisode) {
      navigate(`/player/${seriesId}/${firstAvailableEpisode.episodeNumber}`);
    }
  };
  const playLabel = isMovie
    ? "Play movie"
    : firstAvailableEpisode
      ? `Play S${pad(firstAvailableEpisode.seasonNumber ?? 1)}E${pad(Math.floor(firstAvailableEpisode.episodeNumber))}`
      : "Play";

  // Strip HTML from description, truncate
  const cleanDescription = seriesData.description
    ? seriesData.description
        .replace(/<br\s*\/?>/gi, " ")
        .replace(/<[^>]*>/g, "")
        .substring(0, 600) + (seriesData.description.length > 600 ? "…" : "")
    : null;

  return (
    <div className="page series-detail">
      <button className="detail-back" onClick={() => navigate("/")}>
        <ArrowLeft size={14} />
        <span>Library</span>
      </button>

      <div className="detail-header">
        <div className="detail-poster-col">
          {posterUrl ? (
            <img
              src={posterUrl}
              alt={seriesData.title || "Poster"}
              className="detail-poster"
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
            <div className="detail-poster-placeholder">
              {isMovie ? <Film size={64} /> : <Tv size={64} />}
            </div>
          )}
        </div>

        <div className="detail-info-col">
          <div className="detail-eyebrow">
            <span>{isMovie ? "Movie" : "Series"}</span>
            {seriesData.status && !isMovie && (
              <>
                <span className="dot" />
                <span className={`status-${normalizeStatus(seriesData.status)}`}>{seriesData.status}</span>
              </>
            )}
            {seriesData.source && (
              <>
                <span className="dot" />
                <span className="source-tag">{seriesData.source.toUpperCase()}</span>
              </>
            )}
          </div>

          <h1 className="detail-title">{seriesData.title}</h1>

          {seriesData.titleRomaji && seriesData.titleRomaji !== seriesData.title && (
            <div className="detail-alt">
              {seriesData.titleRomaji}
              {seriesData.titleNative && seriesData.titleNative !== seriesData.titleRomaji
                ? `  ·  ${seriesData.titleNative}`
                : ""}
            </div>
          )}

          <div className="detail-stats">
            {score && (
              <div className="stat">
                <span className="stat-label">Score</span>
                <span className="stat-value">{score}</span>
              </div>
            )}
            {year && (
              <div className="stat">
                <span className="stat-label">Year</span>
                <span className="stat-value">{year}</span>
              </div>
            )}
            {durationText && (
              <div className="stat">
                <span className="stat-label">{isMovie ? "Runtime" : "Per ep"}</span>
                <span className="stat-value">{durationText}</span>
              </div>
            )}
            {!isMovie && totalCount > 0 && (
              <div className="stat">
                <span className="stat-label">Episodes</span>
                <span className="stat-value">
                  {availableCount}
                  <span className="stat-of">/{totalCount}</span>
                </span>
              </div>
            )}
            {studios && (
              <div className="stat">
                <span className="stat-label">Studio</span>
                <span className="stat-value sans">{studios}</span>
              </div>
            )}
          </div>

          {cleanDescription && <p className="detail-desc">{cleanDescription}</p>}

          {seriesData.genres && seriesData.genres.length > 0 && (
            <div className="detail-genres">
              {seriesData.genres.map((genre: string) => (
                <span key={genre} className="genre-pill">
                  {genre}
                </span>
              ))}
            </div>
          )}

          <div className="detail-actions">
            {isMovie ? (
              hasFile ? (
                <button className="btn btn-primary" onClick={handlePlayMovie}>
                  <Play size={15} strokeWidth={2.25} />
                  <span>{playLabel}</span>
                </button>
              ) : (
                <div className="detail-unavailable">
                  <AlertTriangle size={15} />
                  <span>Movie file not on disk</span>
                </div>
              )
            ) : firstAvailableEpisode ? (
              <button className="btn btn-primary" onClick={handlePlaySeries}>
                <Play size={15} strokeWidth={2.25} />
                <span>{playLabel}</span>
              </button>
            ) : (
              <div className="detail-unavailable">
                <AlertTriangle size={15} />
                <span>No episodes on disk</span>
              </div>
            )}
          </div>
        </div>
      </div>

      {!isMovie && (
        <div className="episodes-section">
          <div className="section-head">
            <h2 className="section-h2">Episodes</h2>
            <span className="section-count">{availableCount} of {totalCount} on disk</span>
          </div>

          {episodesBySeason.size > 0 ? (
            Array.from(episodesBySeason.values())
              .sort((a, b) => {
                if (a.seasonNumber === null) return -1;
                if (b.seasonNumber === null) return 1;
                return (a.seasonNumber ?? 0) - (b.seasonNumber ?? 0);
              })
              .map((seasonData) => (
                <div key={seasonData.seasonNumber ?? "no-season"} className="season-section">
                  {episodesBySeason.size > 1 && (
                    <h3 className="season-title">
                      {seasonData.seasonNumber !== null
                        ? `Season ${seasonData.seasonNumber}`
                        : "Episodes"}
                      <span className="season-episode-count">
                        {seasonData.episodes.filter((ep) => ep.hasFile).length} /{" "}
                        {seasonData.episodes.length} on disk
                      </span>
                    </h3>
                  )}
                  <div className="episodes-grid">
                    {seasonData.episodes.map((episode) => (
                      <EpisodeCard
                        key={`${seasonData.seasonNumber ?? "no-season"}-${episode.episodeNumber}`}
                        seriesId={seriesId}
                        episode={episode}
                        hasFile={episode.hasFile}
                      />
                    ))}
                  </div>
                </div>
              ))
          ) : (
            <div className="no-episodes">No episodes found</div>
          )}
        </div>
      )}
    </div>
  );
}

export default SeriesDetailPage;
