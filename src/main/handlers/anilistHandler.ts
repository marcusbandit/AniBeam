import { request, gql } from 'graphql-request';
import { logger } from '../services/logger';
import { RateLimiter } from '../utils/rateLimiter';

const ANILIST_API_URL = 'https://graphql.anilist.co';

// AniList allows ~90 req/min normalized. 800ms between requests = 75/min,
// safely under the cap. The limiter handles 429 backoff on top of this.
const limiter = new RateLimiter({
  source: 'AniList',
  minIntervalMs: 800,
  maxRetries: 6,
  isRateLimitError,
});

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
  idMal: number | null;
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
  malId: number | null;
}

export interface EpisodeMetadata {
  episodeNumber: number;
  seasonNumber?: number | null;
  title: string;
  description: string | null;
  airDate: string | null;
  thumbnail: string | null;
}

export interface RelationEntry {
  relationType: string;
  anilistId: number;
  malId: number | null;
  type: 'ANIME' | 'MANGA' | null;
  format: string | null;
  status: string | null;
  seasonYear: number | null;
  siteUrl: string | null;
  titleRomaji: string | null;
  titleEnglish: string | null;
  poster: string | null;
}

export interface TagEntry {
  name: string;
  rank: number | null;
  isMediaSpoiler: boolean;
  isGeneralSpoiler: boolean;
  isAdult: boolean;
  category: string | null;
}

export interface CharacterEntry {
  anilistId: number;
  name: string | null;
  role: string | null;
  image: string | null;
  siteUrl: string | null;
}

export interface RecommendationEntry {
  rating: number | null;
  anilistId: number;
  malId: number | null;
  type: 'ANIME' | 'MANGA' | null;
  format: string | null;
  status: string | null;
  seasonYear: number | null;
  siteUrl: string | null;
  titleRomaji: string | null;
  titleEnglish: string | null;
  poster: string | null;
}

export interface StudioEntry {
  anilistId: number;
  name: string;
  isMain: boolean;
  isAnimationStudio: boolean;
}

export interface EnrichmentBundle {
  relations: RelationEntry[];
  tags: TagEntry[];
  characters: CharacterEntry[];
  recommendations: RecommendationEntry[];
  studios: StudioEntry[];
  /** Per-episode titles + thumbnails pulled from AniList's `streamingEpisodes`
   *  (the data behind the "Watch" tab on anilist.co). This is the canonical
   *  source for episode names; MAL/Jikan is a fallback for shows AniList
   *  doesn't surface. Sparse — only episodes AniList has data for appear here. */
  episodeTitles: Array<{ episodeNumber: number; title: string; thumbnail: string | null }>;
}

interface RawEnrichmentMedia {
  streamingEpisodes?: Array<{
    title: string | null;
    thumbnail: string | null;
    url: string | null;
    site: string | null;
  }> | null;
  tags?: Array<{
    name: string;
    rank: number | null;
    isMediaSpoiler: boolean | null;
    isGeneralSpoiler: boolean | null;
    isAdult: boolean | null;
    category: string | null;
  }>;
  studios?: {
    edges: Array<{
      isMain: boolean | null;
      node: { id: number; name: string; isAnimationStudio: boolean | null };
    }>;
  } | null;
  characters?: {
    edges: Array<{
      role: string | null;
      node: {
        id: number;
        name: { full: string | null } | null;
        image: { large: string | null; medium: string | null } | null;
        siteUrl: string | null;
      };
    }>;
  } | null;
  recommendations?: {
    edges: Array<{
      node: {
        rating: number | null;
        mediaRecommendation: {
          id: number;
          idMal: number | null;
          type: 'ANIME' | 'MANGA' | null;
          format: string | null;
          status: string | null;
          seasonYear: number | null;
          siteUrl: string | null;
          title: { romaji: string | null; english: string | null } | null;
          coverImage: { large: string | null } | null;
        } | null;
      };
    }>;
  } | null;
  relations?: {
    edges: Array<{
      relationType: string;
      node: {
        id: number;
        idMal: number | null;
        type: 'ANIME' | 'MANGA' | null;
        format: string | null;
        status: string | null;
        seasonYear: number | null;
        siteUrl: string | null;
        title: { romaji: string | null; english: string | null } | null;
        coverImage: { large: string | null } | null;
      };
    }>;
  } | null;
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
      idMal
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
        idMal
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

// Cheap one-shot to map a MAL id to its AniList id. Used by the poster
// matcher when MAL is the primary match so we can populate both ids on
// the series record (trackers + AniSkip read them by name).
const RESOLVE_ID_BY_MAL_QUERY = gql`
  query ($idMal: Int) {
    Media(idMal: $idMal, type: ANIME) {
      id
    }
  }
`;

// Per-episode air dates. AniList exposes them on Media.airingSchedule —
// fetch the full schedule (both aired and not-yet-aired) so the renderer
// can show a live countdown to the next upcoming episode in addition to
// sorting the feed by latest aired. Fetch by AniList id OR by MAL id —
// AniList's Media query accepts either as a filter, so we can resolve air
// dates for MAL-matched series without doing a second title search.
const AIRING_SCHEDULE_QUERY = gql`
  query ($id: Int, $idMal: Int) {
    Media(id: $id, idMal: $idMal, type: ANIME) {
      id
      airingSchedule {
        nodes {
          episode
          airingAt
        }
      }
    }
  }
`;

// Enrichment bundle — one query returns the franchise graph plus the
// extras the series-detail page wants (tags, top characters,
// recommendations, studios). Queryable by AniList id OR MAL id so
// MAL-matched series can still pull this data without a second title
// search. perPage limits keep the response under a few KB even for
// franchises with hundreds of relations.
const ENRICHMENT_QUERY = gql`
  query ($id: Int, $idMal: Int) {
    Media(id: $id, idMal: $idMal) {
      id
      streamingEpisodes {
        title
        thumbnail
        url
        site
      }
      tags {
        name
        rank
        isMediaSpoiler
        isGeneralSpoiler
        isAdult
        category
      }
      studios {
        edges {
          isMain
          node {
            id
            name
            isAnimationStudio
          }
        }
      }
      characters(perPage: 12, sort: [ROLE, RELEVANCE, ID]) {
        edges {
          role
          node {
            id
            name {
              full
            }
            image {
              large
              medium
            }
            siteUrl
          }
        }
      }
      recommendations(perPage: 12, sort: RATING_DESC) {
        edges {
          node {
            rating
            mediaRecommendation {
              id
              idMal
              type
              format
              status
              seasonYear
              siteUrl
              title {
                romaji
                english
              }
              coverImage {
                large
              }
            }
          }
        }
      }
      relations {
        edges {
          relationType
          node {
            id
            idMal
            type
            format
            status
            seasonYear
            siteUrl
            title {
              romaji
              english
            }
            coverImage {
              large
            }
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

const MEDIA_BY_ID_QUERY = gql`
  query ($id: Int) {
    Media(id: $id, type: ANIME) {
      id
      idMal
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

// AniList's streamingEpisodes titles arrive in shapes like
//   "Episode 1 - Ordinary Person"
//   "1 - Ordinary Person"
//   "Episode 1"                       (no real name — drop)
//   "S2 Episode 3 - …"                (multi-season aggregator)
// Parse out the episode number and strip the leading "EpisodeN - " / "N - "
// noise so the renderer can display just the actual episode title.
function parseStreamingEpisodeTitle(raw: string | null | undefined): { episodeNumber: number; title: string } | null {
  if (!raw) return null;
  const trimmed = raw.trim();
  // Capture the first integer; the prefix in front of it is whatever
  // "Episode " / "S2E" / "" — we only care that we find a number.
  const numMatch = trimmed.match(/(?:^|[^\d])(\d{1,3})(?:\D|$)/);
  if (!numMatch) return null;
  const episodeNumber = parseInt(numMatch[1], 10);
  if (!Number.isFinite(episodeNumber) || episodeNumber <= 0) return null;
  // Strip everything up to and including the first " - " separator. If
  // there's no separator, the title is just "Episode N" with no real name;
  // skip rather than persist a placeholder.
  const sepIdx = trimmed.indexOf(' - ');
  if (sepIdx < 0) return null;
  const title = trimmed.slice(sepIdx + 3).trim();
  if (!title || /^Episode\s+\d+$/i.test(title)) return null;
  return { episodeNumber, title };
}

const anilistHandler = {
  async searchAnime(searchTerm: string): Promise<AniListMedia | null> {
    try {
      const variables = { search: searchTerm };
      const data = await limiter.run(() => request<{ Media: AniListMedia }>(ANILIST_API_URL, SEARCH_QUERY, variables));
      return data?.Media ?? null;
    } catch (error) {
      if (isRateLimitError(error)) logRateLimitWarning('AniList');
      else logger.error('metadata', 'Error searching AniList');
      throw error;
    }
  },

  async searchAnimeMultiple(searchTerm: string, limit: number = 10): Promise<AniListMedia[]> {
    try {
      const variables = { search: searchTerm, page: 1, perPage: limit };
      const data = await limiter.run(() =>
        request<{ Page: { media: AniListMedia[] } }>(ANILIST_API_URL, SEARCH_MULTIPLE_QUERY, variables),
      );
      return data?.Page?.media || [];
    } catch (error) {
      if (isRateLimitError(error)) logRateLimitWarning('AniList');
      else logger.error('metadata', 'Error searching AniList (multiple)');
      throw error;
    }
  },

  async resolveAnilistIdByMal(malId: number): Promise<number | null> {
    if (!Number.isFinite(malId) || malId <= 0) return null;
    try {
      const data = await limiter.run(() =>
        request<{ Media: { id: number } | null }>(
          ANILIST_API_URL,
          RESOLVE_ID_BY_MAL_QUERY,
          { idMal: malId },
        ),
      );
      return data?.Media?.id ?? null;
    } catch (error) {
      if (isRateLimitError(error)) logRateLimitWarning('AniList');
      else logger.warn('metadata', `AniList id-by-MAL lookup failed for ${malId}: ${(error as Error).message}`);
      return null;
    }
  },

  async getAiringSchedule(opts: { anilistId?: number; malId?: number }): Promise<Array<{ episode: number; airingAt: number }>> {
    const variables: { id?: number; idMal?: number } = {};
    if (opts.anilistId) variables.id = opts.anilistId;
    if (opts.malId) variables.idMal = opts.malId;
    if (variables.id === undefined && variables.idMal === undefined) return [];
    try {
      const data = await limiter.run(() =>
        request<{ Media: { airingSchedule: { nodes: Array<{ episode: number; airingAt: number }> } } | null }>(
          ANILIST_API_URL,
          AIRING_SCHEDULE_QUERY,
          variables,
        ),
      );
      return data?.Media?.airingSchedule?.nodes ?? [];
    } catch (error) {
      if (isRateLimitError(error)) logRateLimitWarning('AniList');
      else logger.warn('metadata', `AniList airingSchedule failed: ${(error as Error).message}`);
      return [];
    }
  },

  async getEpisodes(animeId: number, totalEpisodes: number | null, seasonNumber?: number | null): Promise<EpisodeMetadata[]> {
    try {
      // Fetch streaming episodes for thumbnails
      const variables = { id: animeId };
      const data = await limiter.run(() =>
        request<{ Media: { streamingEpisodes: StreamingEpisode[] } }>(ANILIST_API_URL, EPISODES_QUERY, variables),
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

  /**
   * One-shot enrichment query — pulls relations + tags + main characters +
   * recommendations + studios in a single GraphQL request. Accepts either
   * an AniList id or a MAL id, so MAL-matched series can still pull this
   * bundle without a second title-search round-trip. Each section is
   * returned in a shape ready for the renderer; the main process slims it
   * further if needed before persisting.
   */
  async getEnrichment(opts: { anilistId?: number; malId?: number }): Promise<EnrichmentBundle> {
    const empty: EnrichmentBundle = { relations: [], tags: [], characters: [], recommendations: [], studios: [], episodeTitles: [] };
    const variables: { id?: number; idMal?: number } = {};
    if (opts.anilistId) variables.id = opts.anilistId;
    if (opts.malId) variables.idMal = opts.malId;
    if (variables.id === undefined && variables.idMal === undefined) return empty;
    try {
      const data = await limiter.run(() =>
        request<{ Media: RawEnrichmentMedia | null }>(ANILIST_API_URL, ENRICHMENT_QUERY, variables),
      );
      const media = data?.Media;
      if (!media) return empty;
      return {
        relations: (media.relations?.edges ?? []).map((e) => ({
          relationType: e.relationType,
          anilistId: e.node.id,
          malId: e.node.idMal,
          type: e.node.type,
          format: e.node.format,
          status: e.node.status,
          seasonYear: e.node.seasonYear,
          siteUrl: e.node.siteUrl,
          titleRomaji: e.node.title?.romaji ?? null,
          titleEnglish: e.node.title?.english ?? null,
          poster: e.node.coverImage?.large ?? null,
        })),
        tags: (media.tags ?? []).map((t) => ({
          name: t.name,
          rank: typeof t.rank === 'number' ? t.rank : null,
          isMediaSpoiler: !!t.isMediaSpoiler,
          isGeneralSpoiler: !!t.isGeneralSpoiler,
          isAdult: !!t.isAdult,
          category: t.category ?? null,
        })),
        characters: (media.characters?.edges ?? []).map((e) => ({
          anilistId: e.node.id,
          name: e.node.name?.full ?? null,
          role: e.role ?? null,
          image: e.node.image?.large ?? e.node.image?.medium ?? null,
          siteUrl: e.node.siteUrl ?? null,
        })),
        recommendations: (media.recommendations?.edges ?? [])
          .filter((e) => e.node.mediaRecommendation != null)
          .map((e) => {
            const r = e.node.mediaRecommendation!;
            return {
              rating: typeof e.node.rating === 'number' ? e.node.rating : null,
              anilistId: r.id,
              malId: r.idMal,
              type: r.type,
              format: r.format,
              status: r.status,
              seasonYear: r.seasonYear,
              siteUrl: r.siteUrl,
              titleRomaji: r.title?.romaji ?? null,
              titleEnglish: r.title?.english ?? null,
              poster: r.coverImage?.large ?? null,
            };
          }),
        studios: (media.studios?.edges ?? []).map((e) => ({
          anilistId: e.node.id,
          name: e.node.name,
          isMain: !!e.isMain,
          isAnimationStudio: !!e.node.isAnimationStudio,
        })),
        episodeTitles: (() => {
          const out: Array<{ episodeNumber: number; title: string; thumbnail: string | null }> = [];
          const seen = new Set<number>();
          for (const se of media.streamingEpisodes ?? []) {
            const parsed = parseStreamingEpisodeTitle(se.title);
            if (!parsed) continue;
            // First entry wins per episode — streaming aggregators sometimes
            // list the same episode for multiple providers (Crunchyroll +
            // HiDive + …) with subtly different formatting.
            if (seen.has(parsed.episodeNumber)) continue;
            seen.add(parsed.episodeNumber);
            out.push({ episodeNumber: parsed.episodeNumber, title: parsed.title, thumbnail: se.thumbnail ?? null });
          }
          return out;
        })(),
      };
    } catch (error) {
      if (isRateLimitError(error)) logRateLimitWarning('AniList');
      else logger.warn('metadata', `AniList enrichment fetch failed: ${(error as Error).message}`);
      return empty;
    }
  },

  /**
   * Crawl-friendly relations fetch. Returns `ok: false` when the AniList
   * call was exhausted by rate-limiting so callers can defer the node and
   * retry on a later invocation. Returns `ok: true` (with possibly-empty
   * relations) on all other outcomes — genuine "no relations" and benign
   * non-rate-limit failures both look like "we know there's nothing here".
   */
  async fetchRelations(anilistId: number): Promise<{ relations: RelationEntry[]; ok: boolean }> {
    try {
      const data = await limiter.run(() =>
        request<{ Media: RawEnrichmentMedia | null }>(ANILIST_API_URL, ENRICHMENT_QUERY, { id: anilistId }),
      );
      const media = data?.Media;
      const relations = (media?.relations?.edges ?? []).map((e) => ({
        relationType: e.relationType,
        anilistId: e.node.id,
        malId: e.node.idMal,
        type: e.node.type,
        format: e.node.format,
        status: e.node.status,
        seasonYear: e.node.seasonYear,
        siteUrl: e.node.siteUrl,
        titleRomaji: e.node.title?.romaji ?? null,
        titleEnglish: e.node.title?.english ?? null,
        poster: e.node.coverImage?.large ?? null,
      }));
      return { relations, ok: true };
    } catch (error) {
      if (isRateLimitError(error)) {
        logRateLimitWarning('AniList');
        return { relations: [], ok: false };
      }
      logger.warn('metadata', `AniList relations fetch failed: ${(error as Error).message}`);
      return { relations: [], ok: true };
    }
  },

  async getMediaById(id: number): Promise<AniListMedia | null> {
    try {
      const data = await limiter.run(() =>
        request<{ Media: AniListMedia | null }>(ANILIST_API_URL, MEDIA_BY_ID_QUERY, { id }),
      );
      return data?.Media ?? null;
    } catch (error) {
      if (isRateLimitError(error)) logRateLimitWarning('AniList');
      else logger.error('metadata', 'Error fetching AniList media by ID');
      throw error;
    }
  },

  // Override path: caller has already chosen a specific AniList ID via the
  // match picker. Skip the search/filter logic — fetch the exact media plus
  // its episodes and format. seasonNumber is used only for episode tagging
  // and the seriesId suffix; pass it when overriding inside a season-specific
  // entry so the seriesId stays stable across re-overrides.
  async fetchMetadataById(id: number, seasonNumber?: number | null): Promise<SeriesMetadata | null> {
    try {
      const media = await this.getMediaById(id);
      if (!media) return null;

      let episodes: EpisodeMetadata[] = [];
      let retries = 0;
      const maxRetries = 3;
      while (retries < maxRetries) {
        try {
          episodes = await this.getEpisodes(media.id, media.episodes, seasonNumber);
          break;
        } catch (error) {
          if (isRateLimitError(error) && retries < maxRetries) {
            retries++;
            const delaySeconds = retries * 2;
            logger.warn('metadata', `Rate limited fetching episodes by id. Waiting ${delaySeconds}s (retry ${retries}/${maxRetries})`);
            await new Promise(resolve => setTimeout(resolve, delaySeconds * 1000));
          } else {
            throw error;
          }
        }
      }

      return this.formatMetadata(media, episodes, seasonNumber);
    } catch (error) {
      if (isRateLimitError(error)) return null;
      logger.error('metadata', 'Error fetching AniList metadata by id');
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
      malId: media.idMal ?? null,
    };
  },
};

export default anilistHandler;
