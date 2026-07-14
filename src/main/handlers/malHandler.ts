// Jikan per-episode side-fetch ONLY. This handler is NOT a metadata
// source: MAL was removed as a matching/fetching provider by explicit
// user decision (2026-07-14) and must not be re-added. getEpisodes stays
// because it is the only source of per-episode titles and air dates
// (AniList's airingSchedule carries neither), keyed by the malId that
// AniList's idMal cross-reference provides.

import axios from 'axios';
import { logger } from '../services/logger';
import { RateLimiter } from '../utils/rateLimiter';

const JIKAN_API_URL = 'https://api.jikan.moe/v4';

// Jikan published limits: 60 req/min sustained. 1100ms between starts =
// ~54/min with safety margin. 429s on top get exponential backoff via
// the limiter: no per-call retry loops needed downstream.
const limiter = new RateLimiter({
  source: 'Jikan',
  minIntervalMs: 1100,
  maxRetries: 6,
  isRateLimitError,
});

function isRateLimitError(error: unknown): boolean {
  if (axios.isAxiosError(error)) {
    return error.response?.status === 429;
  }
  return false;
}

function logRateLimitWarning(source: string): void {
  logger.warn('metadata', `Rate limited by ${source}. Please wait before trying again.`);
}

interface JikanEpisode {
  mal_id: number;
  episode: number;
  title: string;
  synopsis: string | null;
  aired: string | null;
  images?: {
    jpg?: {
      image_url: string;
    };
  };
}

export interface EpisodeMetadata {
  episodeNumber: number;
  seasonNumber?: number | null;
  title: string;
  description: string | null;
  airDate: string | null;
  thumbnail: string | null;
}

const malHandler = {
  async getEpisodes(animeId: number, totalEpisodes: number | null, seasonNumber?: number | null): Promise<EpisodeMetadata[]> {
    try {
      const response = await limiter.run(() =>
        axios.get<{ data: JikanEpisode[] }>(`${JIKAN_API_URL}/anime/${animeId}/episodes`),
      );

      // Create a map of fetched episodes
      // Use episode number, not mal_id (mal_id is the database ID, not episode number)
      const fetchedEpisodeMap = new Map<number, JikanEpisode>();
      if (response.data?.data) {
        for (const ep of response.data.data) {
          const epNum = ep.episode; // Use episode number, not mal_id
          if (epNum && epNum > 0) {
            fetchedEpisodeMap.set(epNum, ep);
          }
        }
      }

      // Generate episodes based on totalEpisodes count
      const episodeCount = totalEpisodes || fetchedEpisodeMap.size || 0;
      const episodes: EpisodeMetadata[] = [];

      for (let i = 1; i <= episodeCount; i++) {
        const fetchedEp = fetchedEpisodeMap.get(i);
        episodes.push({
          episodeNumber: i,
          seasonNumber: seasonNumber ?? null,
          title: fetchedEp?.title || `Episode ${i}`,
          description: fetchedEp?.synopsis || null,
          airDate: fetchedEp?.aired || null,
          thumbnail: fetchedEp?.images?.jpg?.image_url || null,
        });
      }

      return episodes;
    } catch (error) {
      if (isRateLimitError(error)) {
        logRateLimitWarning('MAL');
        throw error;
      }
      logger.error('metadata', 'Error fetching MAL episodes');
      // If fetching fails but we know totalEpisodes, generate basic entries
      if (totalEpisodes) {
        return Array.from({ length: totalEpisodes }, (_, i) => ({
          episodeNumber: i + 1,
          seasonNumber: seasonNumber ?? null,
          title: `Episode ${i + 1}`,
          description: null,
          airDate: null,
          thumbnail: null,
        }));
      }
      return [];
    }
  },
};

export default malHandler;
