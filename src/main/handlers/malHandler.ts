import axios from 'axios';
import { logger } from '../services/logger';
import { RateLimiter } from '../utils/rateLimiter';

const JIKAN_API_URL = 'https://api.jikan.moe/v4';
const MAL_SEARCH_LIMIT = 10;

// Jikan published limits: 60 req/min sustained. 1100ms between starts =
// ~54/min with safety margin. 429s on top get exponential backoff via
// the limiter — no per-call retry loops needed downstream.
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

interface JikanAnime {
  mal_id: number;
  title: string;
  title_english: string | null;
  title_japanese: string | null;
  synopsis: string | null;
  background: string | null;
  genres: { name: string }[];
  images: {
    jpg: {
      image_url: string;
      large_image_url: string;
    };
  };
  episodes: number | null;
  status: string;
  type: string;
  score: number | null;
  studios: { name: string }[];
  aired: {
    from: string | null;
    to: string | null;
  };
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

export interface SeriesMetadata {
  seriesId: string;
  title: string;
  titleEnglish?: string | null;
  titleJapanese?: string | null;
  description: string;
  genres: string[];
  poster: string | null;
  banner: null;
  episodes: EpisodeMetadata[];
  totalEpisodes: number | null;
  status: string;
  format: string;
  averageScore: number | null;
  studios: string[];
  startDate: string | null;
  endDate: string | null;
  malId: number;
}

export interface EpisodeMetadata {
  episodeNumber: number;
  seasonNumber?: number | null;
  title: string;
  description: string | null;
  airDate: string | null;
  thumbnail: string | null;
}

function isReleased(anime: JikanAnime): boolean {
  // Skip anime that haven't been released yet
  // Status values like "Not yet aired", "Not yet released" should be skipped
  const status = anime.status?.toLowerCase() || '';
  if (status.includes('not yet') || status.includes('not aired')) {
    return false;
  }
  // Allow: "Currently Airing", "Finished Airing", "On Hold", etc.
  return true;
}

const malHandler = {
  async searchAnime(searchTerm: string, limit: number = MAL_SEARCH_LIMIT): Promise<JikanAnime[]> {
    try {
      const response = await limiter.run(() =>
        axios.get<{ data: JikanAnime[] }>(`${JIKAN_API_URL}/anime`, {
          params: { q: searchTerm, limit },
        }),
      );
      return response.data?.data || [];
    } catch (error) {
      if (isRateLimitError(error)) logRateLimitWarning('MAL');
      else logger.error('metadata', 'Error searching MyAnimeList');
      throw error;
    }
  },

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

  async searchAndFetchMetadata(seriesName: string, seasonNumber?: number | null, partNumber?: number | null, folderEpisodeCount?: number): Promise<SeriesMetadata | null> {
    try {
      // Special handling for numeric-only series names like "86"
      // Try variations: "86" -> "86", "eighty six", "86 anime"
      const searchVariations: string[] = [seriesName];
      if (/^\d+$/.test(seriesName.trim())) {
        // It's a numeric-only name, add variations
        const num = parseInt(seriesName.trim(), 10);
        if (num === 86) {
          searchVariations.push('eighty six', '86 anime', '86 -eighty six-');
        }
      }
      
      // Only include season/part in search query if > 1 (don't search for "Season 1" or "Part 1")
      // Prioritize part number over season number for search
      let searchQuery = seriesName;
      if (partNumber !== null && partNumber !== undefined && partNumber > 1) {
        searchQuery = `${seriesName} Part ${partNumber}`;
      } else if (seasonNumber !== null && seasonNumber !== undefined && seasonNumber > 1) {
        searchQuery = `${seriesName} Season ${seasonNumber}`;
      }
      
      // Search for multiple results (up to 10) to find one with enough episodes
      let searchResults = await this.searchAnime(searchQuery, MAL_SEARCH_LIMIT);
      logger.info('metadata', `MAL search: "${searchQuery}" => ${searchResults.length} result(s).`);

      // If no results and we have variations, try them
      if (searchResults.length === 0 && searchVariations.length > 1) {
        for (const variation of searchVariations.slice(1)) {
          // Only include part/season if > 1
          let variationQuery = variation;
          if (partNumber !== null && partNumber !== undefined && partNumber > 1) {
            variationQuery = `${variation} Part ${partNumber}`;
          } else if (seasonNumber !== null && seasonNumber !== undefined && seasonNumber > 1) {
            variationQuery = `${variation} Season ${seasonNumber}`;
          }
          searchResults = await this.searchAnime(variationQuery, MAL_SEARCH_LIMIT);
          logger.info('metadata', `MAL search (variation): "${variationQuery}" => ${searchResults.length} result(s).`);
          if (searchResults.length > 0) {
            searchQuery = variationQuery;
            break;
          }
        }
      }

      if (searchResults.length === 0 && ((partNumber && partNumber > 1) || (seasonNumber && seasonNumber > 1))) {
        const resultsWithoutSeason = await this.searchAnime(seriesName, MAL_SEARCH_LIMIT);
        logger.info('metadata', `MAL search (no season): "${seriesName}" => ${resultsWithoutSeason.length} result(s).`, { series: seriesName });

        if (resultsWithoutSeason.length > 0) {
          let foundValidResult = false;
          for (const anime of resultsWithoutSeason) {
            if (
              !isReleased(anime) ||
              (folderEpisodeCount !== undefined && (
                anime.episodes === null ||
                anime.episodes < folderEpisodeCount
              ))
            ) continue;

            foundValidResult = true;
            const episodes = await this.getEpisodes(anime.mal_id, anime.episodes, seasonNumber);
            logger.info('metadata', `MAL accepted: "${anime.title}"`, { series: anime.title });
            return this.formatMetadata(anime, episodes, seasonNumber);
          }

          // Fallback: accept result with unknown episode count if needed
          if (!foundValidResult && folderEpisodeCount !== undefined) {
            for (const anime of resultsWithoutSeason) {
              if (anime.episodes === null) {
                const episodes = await this.getEpisodes(anime.mal_id, anime.episodes, seasonNumber);
                logger.info('metadata', `MAL fallback accepted: "${anime.title}" (unknown episode count)`, { series: anime.title });
                return this.formatMetadata(anime, episodes, seasonNumber);
              }
            }
          }
        }
        logger.warn('metadata', `MAL: No suitable results found for "${searchQuery}" or "${seriesName}".`, { series: seriesName });
        return null;
      }

      // Try each result until we find one with enough episodes
      let foundValidResult = false;
      for (let i = 0; i < searchResults.length; i++) {
        const anime = searchResults[i];
        logger.info('metadata', `[${i + 1}/${searchResults.length}] Checking "${anime.title}" - episodes: ${anime.episodes ?? 'unknown'}, status: ${anime.status}`, { series: anime.title });

        // Skip if not yet released or doesn't have required episodes
        if (!isReleased(anime)) continue;
        if (folderEpisodeCount !== undefined) {
          if (anime.episodes === null || anime.episodes < folderEpisodeCount) continue;
        }

        logger.info('metadata', `Accepting "${anime.title}" - has ${anime.episodes} episodes, folder has ${folderEpisodeCount ?? 'unknown'}`, { series: anime.title });
        foundValidResult = true;
        
        // Retry with delay if we get rate limited after confirming a match
        let episodes: EpisodeMetadata[];
        let retries = 0;
        const maxRetries = 3;
        while (retries < maxRetries) {
          try {
            episodes = await this.getEpisodes(anime.mal_id, anime.episodes, seasonNumber);
            break;
          } catch (error) {
            if (isRateLimitError(error) && retries < maxRetries) {
              retries++;
              const delaySeconds = retries * 2; // 2, 4, 6 seconds
              logger.warn('metadata', `Rate limited while fetching episodes. Waiting ${delaySeconds}s before retry ${retries}/${maxRetries}...`);
              await new Promise(resolve => setTimeout(resolve, delaySeconds * 1000));
            } else {
              throw error;
            }
          }
        }

        return this.formatMetadata(anime, episodes!, seasonNumber);
      }

      // If we have folder episode count but no valid result, try accepting null results as fallback
      if (!foundValidResult && folderEpisodeCount !== undefined) {
        for (let i = 0; i < searchResults.length; i++) {
          const anime = searchResults[i];
          if (anime.episodes === null) {
            logger.warn('metadata', `Fallback: Accepting "${anime.title}" with unknown episode count`, { series: anime.title });

            // Retry with delay if we get rate limited
            let episodes: EpisodeMetadata[];
            let retries = 0;
            const maxRetries = 3;
            while (retries < maxRetries) {
              try {
                episodes = await this.getEpisodes(anime.mal_id, anime.episodes, seasonNumber);
                break;
              } catch (error) {
                if (isRateLimitError(error) && retries < maxRetries) {
                  retries++;
                  const delaySeconds = retries * 2; // 2, 4, 6 seconds
                  logger.warn('metadata', `Rate limited while fetching episodes. Waiting ${delaySeconds}s before retry ${retries}/${maxRetries}...`);
                  await new Promise(resolve => setTimeout(resolve, delaySeconds * 1000));
                } else {
                  throw error;
                }
              }
            }
            
            return this.formatMetadata(anime, episodes!, seasonNumber);
          }
        }
        logger.warn('metadata', `No results with enough episodes found, or only found series with unknown episode counts.`);
      }

      return null;
    } catch (error) {
      if (isRateLimitError(error)) {
        return null;
      }
      logger.error('metadata', 'Error fetching MAL metadata');
      return null;
    }
  },

  formatMetadata(anime: JikanAnime, episodes: EpisodeMetadata[], seasonNumber?: number | null): SeriesMetadata {
    // Add season number to title if we searched for a specific season
    let title = anime.title || anime.title_english || anime.title_japanese || '';
    if (seasonNumber) {
      title = `${title} (Season ${seasonNumber})`;
    }

    return {
      seriesId: `mal_${anime.mal_id}${seasonNumber ? `_s${String(seasonNumber).padStart(2, '0')}` : ''}`,
      title,
      titleEnglish: anime.title_english,
      titleJapanese: anime.title_japanese,
      description: anime.synopsis || anime.background || '',
      genres: anime.genres?.map(g => g.name) || [],
      poster: anime.images?.jpg?.large_image_url || anime.images?.jpg?.image_url || null,
      banner: null,
      episodes,
      totalEpisodes: anime.episodes,
      status: anime.status,
      format: anime.type,
      averageScore: anime.score,
      studios: anime.studios?.map(s => s.name) || [],
      startDate: anime.aired?.from ? new Date(anime.aired.from).toISOString().split('T')[0] : null,
      endDate: anime.aired?.to ? new Date(anime.aired.to).toISOString().split('T')[0] : null,
      malId: anime.mal_id,
    };
  },
};

export default malHandler;
