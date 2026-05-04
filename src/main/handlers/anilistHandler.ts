import { request, gql } from 'graphql-request';
import { logger } from '../services/logger';

const ANILIST_API_URL = 'https://graphql.anilist.co';

function isRateLimitError(error: unknown): boolean {
  // Check for GraphQL rate limit errors
  if (error && typeof error === 'object') {
    const err = error as { response?: { status?: number }; statusCode?: number; message?: string };
    // Check HTTP status code
    if (err.response?.status === 429 || err.statusCode === 429) {
      return true;
    }
    // Check for rate limit in error message
    if (err.message && /rate.?limit/i.test(err.message)) {
      return true;
    }
  }
  return false;
}

function logRateLimitWarning(source: string): void {
  logger.warn('metadata', `Rate limited by ${source}. Please wait before trying again.`);
}

interface AniListMedia {
  id: number;
  title: {
    romaji: string;
    english: string | null;
    native: string;
  };
  description: string | null;
  genres: string[];
  coverImage: {
    large: string;
    extraLarge: string;
  } | null;
  bannerImage: string | null;
  episodes: number | null;
  duration: number | null;
  season: string | null;
  seasonYear: number | null;
  status: string;
  format: string;
  startDate: {
    year: number | null;
    month: number | null;
    day: number | null;
  } | null;
  endDate: {
    year: number | null;
    month: number | null;
    day: number | null;
  } | null;
  averageScore: number | null;
  studios: {
    nodes: { name: string }[];
  } | null;
}

interface StreamingEpisode {
  title: string;
  thumbnail: string | null;
  url: string;
  site: string;
}

export interface SeriesMetadata {
  seriesId: string;
  title: string;
  titleRomaji?: string;
  titleEnglish?: string | null;
  titleNative?: string;
  description: string;
  genres: string[];
  poster: string | null;
  banner: string | null;
  episodes: EpisodeMetadata[];
  totalEpisodes: number | null;
  duration: number | null;
  season: string | null;
  seasonYear: number | null;
  status: string;
  format: string;
  averageScore: number | null;
  studios: string[];
  startDate: string | null;
  endDate: string | null;
  anilistId: number;
}

export interface EpisodeMetadata {
  episodeNumber: number;
  seasonNumber?: number | null;
  title: string;
  description: string | null;
  airDate: string | null;
  thumbnail: string | null;
}

function isReleased(media: AniListMedia): boolean {
  // Skip media that haven't been released yet
  // Allow: RELEASING (currently airing), FINISHED (completed)
  // Skip: NOT_YET_RELEASED (not released), CANCELLED, HIATUS
  const status = media.status?.toUpperCase() || '';
  if (status === 'NOT_YET_RELEASED' || status === 'CANCELLED' || status === 'HIATUS') {
    return false;
  }
  // Allow: RELEASING, FINISHED
  return true;
}

const SEARCH_QUERY = gql`
  query ($search: String) {
    Media(search: $search, type: ANIME) {
      id
      title {
        romaji
        english
        native
      }
      description
      genres
      coverImage {
        large
        extraLarge
      }
      bannerImage
      episodes
      duration
      season
      seasonYear
      status
      format
      startDate {
        year
        month
        day
      }
      endDate {
        year
        month
        day
      }
      averageScore
      studios {
        nodes {
          name
        }
      }
    }
  }
`;

const SEARCH_MULTIPLE_QUERY = gql`
  query ($search: String, $page: Int, $perPage: Int) {
    Page(page: $page, perPage: $perPage) {
      media(search: $search, type: ANIME) {
        id
        title {
          romaji
          english
          native
        }
        description
        genres
        coverImage {
          large
          extraLarge
        }
        bannerImage
        episodes
        duration
        season
        seasonYear
        status
        format
        startDate {
          year
          month
          day
        }
        endDate {
          year
          month
          day
        }
        averageScore
        studios {
          nodes {
            name
          }
        }
      }
    }
  }
`;

const EPISODES_QUERY = gql`
  query ($id: Int) {
    Media(id: $id) {
      id
      streamingEpisodes {
        title
        thumbnail
        url
        site
      }
    }
  }
`;

const anilistHandler = {
  async searchAnime(searchTerm: string): Promise<AniListMedia | null> {
    try {
      const variables = { search: searchTerm };
      const data = await request<{ Media: AniListMedia }>(ANILIST_API_URL, SEARCH_QUERY, variables);
      
      if (data?.Media) {
        return data.Media;
      }
      return null;
    } catch (error) {
      if (isRateLimitError(error)) {
        logRateLimitWarning('AniList');
        throw error;
      }
      logger.error('metadata', 'Error searching AniList');
      throw error;
    }
  },

  async searchAnimeMultiple(searchTerm: string, limit: number = 10): Promise<AniListMedia[]> {
    let retries = 0;
    const maxRetries = 3;
    
    while (retries < maxRetries) {
      try {
        const variables = { search: searchTerm, page: 1, perPage: limit };
        const data = await request<{ Page: { media: AniListMedia[] } }>(ANILIST_API_URL, SEARCH_MULTIPLE_QUERY, variables);
        
        return data?.Page?.media || [];
      } catch (error) {
        if (isRateLimitError(error) && retries < maxRetries) {
          retries++;
          const delaySeconds = retries * 2; // 2, 4, 6 seconds
          logger.warn('metadata', `Rate limited while searching AniList. Waiting ${delaySeconds}s before retry ${retries}/${maxRetries}...`);
          await new Promise(resolve => setTimeout(resolve, delaySeconds * 1000));
        } else {
          if (isRateLimitError(error)) {
            logRateLimitWarning('AniList');
          } else {
            logger.error('metadata', 'Error searching AniList (multiple)');
          }
          throw error;
        }
      }
    }
    
    return [];
  },

  async getEpisodes(animeId: number, totalEpisodes: number | null, seasonNumber?: number | null): Promise<EpisodeMetadata[]> {
    try {
      // Fetch streaming episodes for thumbnails
      const variables = { id: animeId };
      const data = await request<{ Media: { streamingEpisodes: StreamingEpisode[] } }>(
        ANILIST_API_URL, 
        EPISODES_QUERY, 
        variables
      );
      
      // Create a map of streaming episode data by parsing title for episode number
      const streamingMap = new Map<number, StreamingEpisode>();
      if (data?.Media?.streamingEpisodes) {
        for (const ep of data.Media.streamingEpisodes) {
          // Try to extract episode number from title (e.g., "Episode 1" or "1. Title")
          const match = ep.title?.match(/(?:Episode\s*)?(\d+)/i);
          if (match) {
            const epNum = parseInt(match[1], 10);
            if (!streamingMap.has(epNum)) {
              streamingMap.set(epNum, ep);
            }
          }
        }
        
        // If no matches found, use index-based assignment
        if (streamingMap.size === 0) {
          data.Media.streamingEpisodes.forEach((ep, index) => {
            streamingMap.set(index + 1, ep);
          });
        }
      }
      
      // Generate episodes based on totalEpisodes count
      const episodeCount = totalEpisodes || streamingMap.size || 0;
      const episodes: EpisodeMetadata[] = [];
      
      for (let i = 1; i <= episodeCount; i++) {
        const streamingEp = streamingMap.get(i);
        episodes.push({
          episodeNumber: i,
          seasonNumber: seasonNumber ?? null,
          title: streamingEp?.title || `Episode ${i}`,
          description: null,
          airDate: null,
          thumbnail: streamingEp?.thumbnail || null,
        });
      }
      
      return episodes;
    } catch (error) {
      if (isRateLimitError(error)) {
        logRateLimitWarning('AniList');
        throw error;
      }
      logger.error('metadata', 'Error fetching AniList episodes');
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
      // Only include season/part in search query if > 1 (don't search for "Season 1" or "Part 1")
      // Prioritize part number over season number for search
      let searchQuery = seriesName;
      if (partNumber !== null && partNumber !== undefined && partNumber > 1) {
        searchQuery = `${seriesName} Part ${partNumber}`;
      } else if (seasonNumber !== null && seasonNumber !== undefined && seasonNumber > 1) {
        searchQuery = `${seriesName} Season ${seasonNumber}`;
      }
      
      // Search for multiple results (up to 10) to find one with enough episodes
      const searchResults = await this.searchAnimeMultiple(searchQuery, 10);
      logger.info('metadata', `AniList search: "${searchQuery}" => ${searchResults.length} result(s).`);
      
      // If no results, try without season/part (only if we were searching with season/part > 1)
      if (searchResults.length === 0 && ((partNumber && partNumber > 1) || (seasonNumber && seasonNumber > 1))) {
        const resultsWithoutSeason = await this.searchAnimeMultiple(seriesName, 10);
        logger.info('metadata', `AniList search (no season): "${seriesName}" => ${resultsWithoutSeason.length} result(s).`, { series: seriesName });
        if (resultsWithoutSeason.length > 0) {
          // Try each result until we find one with enough episodes
          let foundValidResult = false;
          for (let i = 0; i < resultsWithoutSeason.length; i++) {
            const media = resultsWithoutSeason[i];
            const title = media.title.romaji || media.title.english || media.title.native;
            logger.info('metadata', `[${i + 1}/${resultsWithoutSeason.length}] Checking "${title}" - episodes: ${media.episodes ?? 'unknown'}, status: ${media.status}`, { series: title });

            // Skip if not yet released or doesn't have required episodes
            if (!isReleased(media)) continue;
            if (folderEpisodeCount !== undefined) {
              if (media.episodes === null || media.episodes < folderEpisodeCount) continue;
            }
            
            logger.info('metadata', `Accepting "${title}" - has ${media.episodes} episodes, folder has ${folderEpisodeCount ?? 'unknown'}`, { series: title });
            foundValidResult = true;

            // Retry with delay if we get rate limited after confirming a match
            let episodes: EpisodeMetadata[];
            let retries = 0;
            const maxRetries = 3;
            while (retries < maxRetries) {
              try {
                episodes = await this.getEpisodes(media.id, media.episodes, seasonNumber);
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

            return this.formatMetadata(media, episodes!, seasonNumber);
          }

          // If we have folder episode count but no valid result, try accepting null results as fallback
          if (!foundValidResult && folderEpisodeCount !== undefined) {
            for (let i = 0; i < resultsWithoutSeason.length; i++) {
              const media = resultsWithoutSeason[i];
              const title = media.title.romaji || media.title.english || media.title.native;
              if (media.episodes === null) {
                logger.warn('metadata', `Fallback: Accepting "${title}" with unknown episode count`, { series: title });

                // Retry with delay if we get rate limited
                let episodes: EpisodeMetadata[];
                let retries = 0;
                const maxRetries = 3;
                while (retries < maxRetries) {
                  try {
                    episodes = await this.getEpisodes(media.id, media.episodes, seasonNumber);
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

                return this.formatMetadata(media, episodes!, seasonNumber);
              }
            }
            logger.warn('metadata', `AniList: No suitable results found for "${searchQuery}" or "${seriesName}".`, { series: seriesName });
          }
        }
        return null;
      }

      // Try each result until we find one with enough episodes
      let foundValidResult = false;
      for (let i = 0; i < searchResults.length; i++) {
        const media = searchResults[i];
        const title = media.title.romaji || media.title.english || media.title.native;
        logger.info('metadata', `[${i + 1}/${searchResults.length}] Checking "${title}" - episodes: ${media.episodes ?? 'unknown'}, status: ${media.status}`, { series: title });

        // Skip if not yet released or doesn't have required episodes
        if (!isReleased(media)) continue;
        if (folderEpisodeCount !== undefined) {
          if (media.episodes === null || media.episodes < folderEpisodeCount) continue;
        }

        logger.info('metadata', `Accepting "${title}" - has ${media.episodes} episodes, folder has ${folderEpisodeCount ?? 'unknown'}`, { series: title });
        foundValidResult = true;

        // Retry with delay if we get rate limited after confirming a match
        let episodes: EpisodeMetadata[];
        let retries = 0;
        const maxRetries = 3;
        while (retries < maxRetries) {
          try {
            episodes = await this.getEpisodes(media.id, media.episodes, seasonNumber);
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

        return this.formatMetadata(media, episodes!, seasonNumber);
      }

      // If we have folder episode count but no valid result, try accepting null results as fallback
      if (!foundValidResult && folderEpisodeCount !== undefined) {
        for (let i = 0; i < searchResults.length; i++) {
          const media = searchResults[i];
          const title = media.title.romaji || media.title.english || media.title.native;
          if (media.episodes === null) {
            logger.warn('metadata', `Fallback: Accepting "${title}" with unknown episode count`, { series: title });

            // Retry with delay if we get rate limited
            let episodes: EpisodeMetadata[];
            let retries = 0;
            const maxRetries = 3;
            while (retries < maxRetries) {
              try {
                episodes = await this.getEpisodes(media.id, media.episodes, seasonNumber);
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

            return this.formatMetadata(media, episodes!, seasonNumber);
          }
        }
        logger.warn('metadata', `No results with enough episodes found, or only found series with unknown episode counts.`);
      }

      return null;
    } catch (error) {
      // If it's a rate limit error from searchAnimeMultiple (after retries), we've already logged it
      // Just return null
      if (isRateLimitError(error)) {
        return null;
      }
      logger.error('metadata', 'Error fetching AniList metadata');
      return null;
    }
  },

  formatMetadata(media: AniListMedia, episodes: EpisodeMetadata[], seasonNumber?: number | null): SeriesMetadata {
    const formatDate = (date: { year: number | null; month: number | null; day: number | null } | null): string | null => {
      if (!date?.year) return null;
      const year = date.year;
      const month = String(date.month || 1).padStart(2, '0');
      const day = String(date.day || 1).padStart(2, '0');
      return `${year}-${month}-${day}`;
    };

    // Add season number to title if we searched for a specific season
    let title = media.title.english || media.title.romaji || media.title.native;
    if (seasonNumber) {
      title = `${title} (Season ${seasonNumber})`;
    }

    return {
      seriesId: `anilist_${media.id}${seasonNumber ? `_s${seasonNumber.toString().padStart(2, '0')}` : ''}`,
      title,
      titleRomaji: media.title.romaji,
      titleEnglish: media.title.english,
      titleNative: media.title.native,
      description: media.description || '',
      genres: media.genres || [],
      poster: media.coverImage?.extraLarge || media.coverImage?.large || null,
      banner: media.bannerImage || null,
      episodes,
      totalEpisodes: media.episodes,
      duration: media.duration,
      season: media.season,
      seasonYear: media.seasonYear,
      status: media.status,
      format: media.format,
      averageScore: media.averageScore,
      studios: media.studios?.nodes?.map(s => s.name) || [],
      startDate: formatDate(media.startDate),
      endDate: formatDate(media.endDate),
      anilistId: media.id,
    };
  },
};

export default anilistHandler;
