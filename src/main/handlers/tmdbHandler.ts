// TMDB metadata source, for the part of a library AniList can't describe:
// live-action films and non-anime shows.
//
// AniList is the anime authority and stays the default. It simply has no entry
// for, say, a Kurosawa film, so matching one there either fails or lands on
// something unrelated with a similar title. TMDB covers exactly that gap.
//
// Output is deliberately shaped like anilistHandler.formatMetadata's: the same
// series record, with `source: 'tmdb'` and a `tmdbId` instead of anilistId/
// malId. Everything downstream (cards, the detail page, episode rows) then
// works unchanged. The absent tracker ids are meaningful rather than missing —
// AniList and MAL don't track live-action, so there is nothing to sync, and
// the existing "no AniList/MAL id on this series" path already handles it.
//
// The API key is per-user (TMDB keys are personal, so unlike the AniList/MAL
// client ids there's no build-time env fallback) and lives in config.json.

import { RateLimiter } from '../utils/rateLimiter';
import configHandler from './configHandler';
import { logger } from '../services/logger';

const TMDB_API = 'https://api.themoviedb.org/3';
const TMDB_IMAGE = 'https://image.tmdb.org/t/p';

// TMDB's published limit is ~50 req/s, far above anything we do — but the
// limiter also gives us uniform 429 backoff, which is the real reason every
// provider in this app goes through one.
const limiter = new RateLimiter({
  source: 'TMDB',
  minIntervalMs: 120,
  maxRetries: 5,
  isRateLimitError: (err) => {
    const e = err as { status?: number; statusCode?: number; message?: string };
    if (e?.status === 429 || e?.statusCode === 429) return true;
    return typeof e?.message === 'string' && e.message.includes('429');
  },
});

export type TmdbKind = 'movie' | 'tv';

/** One row in the match modal's TMDB results. */
export interface TmdbSearchResult {
  id: number;
  kind: TmdbKind;
  title: string;
  originalTitle: string | null;
  year: number | null;
  overview: string | null;
  posterUrl: string | null;
  /** Episode count for a show; null for a film (which is always one "episode"
   *  as far as the library is concerned). */
  episodes: number | null;
}

export interface TmdbEpisode {
  episodeNumber: number;
  seasonNumber: number | null;
  title: string | null;
  description: string | null;
  airDate: string | null;
  thumbnail: string | null;
}

/** Series record for metadata.json — mirrors anilistHandler.formatMetadata. */
export interface TmdbSeriesMetadata {
  seriesId: string;
  title: string;
  titleRomaji: string | null;
  titleEnglish: string | null;
  titleNative: string | null;
  description: string;
  genres: string[];
  poster: string | null;
  banner: string | null;
  episodes: TmdbEpisode[];
  totalEpisodes: number | null;
  duration: number | null;
  season: null;
  seasonYear: number | null;
  status: string | null;
  format: string;
  averageScore: number | null;
  studios: string[];
  startDate: string | null;
  endDate: string | null;
  anilistId: null;
  malId: null;
  tmdbId: number;
  tmdbKind: TmdbKind;
}

export class TmdbKeyMissingError extends Error {
  constructor() {
    super('No TMDB API key configured');
    this.name = 'TmdbKeyMissingError';
  }
}

async function apiKey(): Promise<string> {
  const cfg = await configHandler.loadConfig();
  const key = (cfg.tmdbApiKey ?? '').trim();
  if (!key) throw new TmdbKeyMissingError();
  return key;
}

async function get<T>(path: string, params: Record<string, string> = {}): Promise<T> {
  const key = await apiKey();
  const url = new URL(`${TMDB_API}${path}`);
  url.searchParams.set('api_key', key);
  for (const [k, v] of Object.entries(params)) url.searchParams.set(k, v);
  return limiter.run(async () => {
    const res = await fetch(url.toString());
    if (!res.ok) {
      // Surface the status in the message so the limiter's 429 detection and
      // the modal's error copy both have something to work with.
      throw new Error(`TMDB ${res.status} ${res.statusText}`);
    }
    return (await res.json()) as T;
  });
}

function imageUrl(path: string | null | undefined, size: 'poster' | 'backdrop' | 'still'): string | null {
  if (!path) return null;
  // TMDB serves fixed size buckets; these are the smallest that still look
  // right at the sizes the UI renders them.
  const bucket = size === 'poster' ? 'w500' : size === 'backdrop' ? 'w1280' : 'w300';
  return `${TMDB_IMAGE}/${bucket}${path}`;
}

function yearOf(date: string | null | undefined): number | null {
  if (!date) return null;
  const y = parseInt(date.slice(0, 4), 10);
  return Number.isFinite(y) ? y : null;
}

interface RawSearchItem {
  id: number;
  media_type?: string;
  title?: string;               // films
  name?: string;                // shows
  original_title?: string;
  original_name?: string;
  release_date?: string;        // films
  first_air_date?: string;      // shows
  overview?: string;
  poster_path?: string | null;
}

/**
 * Search films and shows in one call via TMDB's multi-search, dropping the
 * `person` results it also returns. Ordered by TMDB's own relevance, which is
 * substantially better than title similarity for live-action.
 */
export async function search(query: string, limit = 12): Promise<TmdbSearchResult[]> {
  const trimmed = query.trim();
  if (trimmed.length < 2) return [];
  const data = await get<{ results?: RawSearchItem[] }>('/search/multi', {
    query: trimmed,
    include_adult: 'false',
  });
  const results: TmdbSearchResult[] = [];
  for (const item of data.results ?? []) {
    const kind: TmdbKind | null = item.media_type === 'movie'
      ? 'movie'
      : item.media_type === 'tv' ? 'tv' : null;
    if (!kind) continue;   // person results, mostly
    const title = kind === 'movie' ? item.title : item.name;
    if (!title) continue;
    const original = kind === 'movie' ? item.original_title : item.original_name;
    results.push({
      id: item.id,
      kind,
      title,
      originalTitle: original && original !== title ? original : null,
      year: yearOf(kind === 'movie' ? item.release_date : item.first_air_date),
      overview: item.overview || null,
      posterUrl: imageUrl(item.poster_path, 'poster'),
      // Episode counts aren't in search results; the detail fetch fills it in.
      episodes: null,
    });
    if (results.length >= limit) break;
  }
  return results;
}

interface RawMovie {
  id: number;
  title: string;
  original_title?: string;
  overview?: string;
  genres?: Array<{ name: string }>;
  poster_path?: string | null;
  backdrop_path?: string | null;
  release_date?: string;
  runtime?: number | null;
  status?: string;
  vote_average?: number;
  production_companies?: Array<{ name: string }>;
}

interface RawShow {
  id: number;
  name: string;
  original_name?: string;
  overview?: string;
  genres?: Array<{ name: string }>;
  poster_path?: string | null;
  backdrop_path?: string | null;
  first_air_date?: string;
  last_air_date?: string;
  episode_run_time?: number[];
  number_of_episodes?: number | null;
  status?: string;
  vote_average?: number;
  networks?: Array<{ name: string }>;
  seasons?: Array<{ season_number: number; episode_count: number }>;
}

interface RawSeasonEpisode {
  episode_number: number;
  season_number: number;
  name?: string;
  overview?: string;
  air_date?: string | null;
  still_path?: string | null;
}

function movieToSeries(m: RawMovie): TmdbSeriesMetadata {
  const year = yearOf(m.release_date);
  return {
    seriesId: `tmdb_movie_${m.id}`,
    title: m.title,
    titleRomaji: null,
    titleEnglish: m.title,
    titleNative: m.original_title && m.original_title !== m.title ? m.original_title : null,
    description: m.overview || '',
    genres: (m.genres ?? []).map((g) => g.name),
    poster: imageUrl(m.poster_path, 'poster'),
    banner: imageUrl(m.backdrop_path, 'backdrop'),
    // A film is a single "episode" as far as the library's file rows go, which
    // is also how the AniList MOVIE format already behaves here.
    episodes: [{
      episodeNumber: 1,
      seasonNumber: null,
      title: m.title,
      description: m.overview || null,
      airDate: m.release_date || null,
      thumbnail: imageUrl(m.backdrop_path, 'still'),
    }],
    totalEpisodes: 1,
    duration: m.runtime ?? null,
    season: null,
    seasonYear: year,
    status: m.status ?? null,
    format: 'MOVIE',
    // averageScore is stored in each provider's NATIVE scale and normalised at
    // render time off `source` (AniList 0-100, MAL 0-10 — see normalizeRating).
    // TMDB's vote_average is already 0-10, so it passes through untouched;
    // rescaling it here would make it render as "85.0".
    averageScore: typeof m.vote_average === 'number' ? m.vote_average : null,
    studios: (m.production_companies ?? []).map((c) => c.name),
    startDate: m.release_date || null,
    endDate: m.release_date || null,
    anilistId: null,
    malId: null,
    tmdbId: m.id,
    tmdbKind: 'movie',
  };
}

function showToSeries(s: RawShow, episodes: TmdbEpisode[]): TmdbSeriesMetadata {
  return {
    seriesId: `tmdb_tv_${s.id}`,
    title: s.name,
    titleRomaji: null,
    titleEnglish: s.name,
    titleNative: s.original_name && s.original_name !== s.name ? s.original_name : null,
    description: s.overview || '',
    genres: (s.genres ?? []).map((g) => g.name),
    poster: imageUrl(s.poster_path, 'poster'),
    banner: imageUrl(s.backdrop_path, 'backdrop'),
    episodes,
    totalEpisodes: s.number_of_episodes ?? (episodes.length || null),
    duration: s.episode_run_time?.[0] ?? null,
    season: null,
    seasonYear: yearOf(s.first_air_date),
    status: s.status ?? null,
    format: 'TV',
    // Native 0-10, same as the film path — see the note in movieToSeries.
    averageScore: typeof s.vote_average === 'number' ? s.vote_average : null,
    studios: (s.networks ?? []).map((n) => n.name),
    startDate: s.first_air_date || null,
    endDate: s.last_air_date || null,
    anilistId: null,
    malId: null,
    tmdbId: s.id,
    tmdbKind: 'tv',
  };
}

/**
 * Episode list for one season of a show. Callers pass the season the local
 * folder represents; TMDB numbers episodes within a season, which is what the
 * library's per-season folders expect.
 */
async function fetchSeasonEpisodes(showId: number, seasonNumber: number): Promise<TmdbEpisode[]> {
  try {
    const data = await get<{ episodes?: RawSeasonEpisode[] }>(`/tv/${showId}/season/${seasonNumber}`);
    return (data.episodes ?? []).map((e) => ({
      episodeNumber: e.episode_number,
      seasonNumber: e.season_number,
      title: e.name || null,
      description: e.overview || null,
      airDate: e.air_date || null,
      thumbnail: imageUrl(e.still_path, 'still'),
    }));
  } catch (err) {
    // A missing season is normal (specials, a folder numbered past what TMDB
    // lists). The series is still worth applying without episode detail.
    logger.warn('metadata', `TMDB: season ${seasonNumber} unavailable for show ${showId}: ${(err as Error).message}`);
    return [];
  }
}

/**
 * Full record for a chosen search result.
 *
 * `seasonNumber` picks which season of a show to pull episodes from; it
 * defaults to 1 and is ignored for films. When the requested season has no
 * episodes (or doesn't exist), we fall back to the first season that does, so
 * a mis-numbered folder still gets titles rather than an empty list.
 */
export async function fetchById(
  id: number,
  kind: TmdbKind,
  seasonNumber?: number | null,
): Promise<TmdbSeriesMetadata | null> {
  try {
    if (kind === 'movie') {
      const movie = await get<RawMovie>(`/movie/${id}`);
      return movieToSeries(movie);
    }
    const show = await get<RawShow>(`/tv/${id}`);
    const wanted = seasonNumber && seasonNumber > 0 ? seasonNumber : 1;
    let episodes = await fetchSeasonEpisodes(id, wanted);
    if (episodes.length === 0) {
      // Season 0 is TMDB's specials bucket, never what a numbered folder means.
      const firstReal = (show.seasons ?? []).find((s) => s.season_number > 0 && s.episode_count > 0);
      if (firstReal && firstReal.season_number !== wanted) {
        episodes = await fetchSeasonEpisodes(id, firstReal.season_number);
      }
    }
    return showToSeries(show, episodes);
  } catch (err) {
    if (err instanceof TmdbKeyMissingError) throw err;
    logger.warn('metadata', `TMDB fetch failed for ${kind} ${id}: ${(err as Error).message}`);
    return null;
  }
}

/** Whether a key is configured, for the settings UI and the match modal. */
export async function hasApiKey(): Promise<boolean> {
  const cfg = await configHandler.loadConfig();
  return (cfg.tmdbApiKey ?? '').trim().length > 0;
}

/**
 * Store the key after checking it actually works — a typo'd key would
 * otherwise only surface as a failed search much later.
 */
export async function setApiKey(key: string): Promise<{ ok: boolean; message?: string }> {
  const trimmed = key.trim();
  if (!trimmed) {
    await configHandler.saveConfig({ tmdbApiKey: '' });
    return { ok: true };
  }
  try {
    const url = new URL(`${TMDB_API}/configuration`);
    url.searchParams.set('api_key', trimmed);
    const res = await fetch(url.toString());
    if (res.status === 401) return { ok: false, message: 'TMDB rejected that key.' };
    if (!res.ok) return { ok: false, message: `TMDB returned ${res.status}.` };
  } catch (err) {
    return { ok: false, message: `Could not reach TMDB: ${(err as Error).message}` };
  }
  await configHandler.saveConfig({ tmdbApiKey: trimmed });
  logger.info('metadata', 'TMDB API key saved');
  return { ok: true };
}

const tmdbHandler = { search, fetchById, hasApiKey, setApiKey };
export default tmdbHandler;
