// Verifies tmdbHandler turns TMDB's payloads into the same series record the
// rest of the app already consumes.
//
// The point isn't to test TMDB — it's that a TMDB-sourced series has to be
// indistinguishable from an AniList-sourced one to every downstream consumer
// (cards, the detail page, episode rows). A missing or differently-named field
// here shows up as a blank hero three screens away, so the shape is asserted
// field by field against anilistHandler.formatMetadata's contract.
//
// `fetch` is stubbed with recorded-shape responses; no network is touched.
//
// Run: bun --bun scripts/verify-tmdb-shape.mjs

import assert from 'node:assert/strict';
import { mock } from 'bun:test';

mock.module('electron', () => ({ app: { getPath: () => '/tmp/anibeam-tmdb-verify' } }));
mock.module('../src/main/services/logger', () => ({
  logger: { info() {}, warn() {}, error() {} },
}));
mock.module('../src/main/handlers/configHandler', () => ({
  default: {
    loadConfig: async () => ({ tmdbApiKey: 'test-key' }),
    saveConfig: async () => true,
  },
}));

// Minimal but real-shaped slices of TMDB's responses.
const MOVIE = {
  id: 346,
  title: 'Seven Samurai',
  original_title: '七人の侍',
  overview: 'A poor village hires seven samurai.',
  genres: [{ name: 'Action' }, { name: 'Drama' }],
  poster_path: '/8OKmBV5BUFzmozIC3pPWKHy17kx.jpg',
  backdrop_path: '/dqcqrxDcuFcYYyEUexcHVDxYqZS.jpg',
  release_date: '1954-04-26',
  runtime: 207,
  status: 'Released',
  vote_average: 8.5,
  production_companies: [{ name: 'Toho' }],
};

const SHOW = {
  id: 1396,
  name: 'Breaking Bad',
  original_name: 'Breaking Bad',
  overview: 'A chemistry teacher turns to cooking meth.',
  genres: [{ name: 'Drama' }],
  poster_path: '/ggFHVNu6YYI5L9pCfOacjizRGt.jpg',
  backdrop_path: '/tsRy63Mu5cu8etL1X7ZLyf7UP1M.jpg',
  first_air_date: '2008-01-20',
  last_air_date: '2013-09-29',
  episode_run_time: [45],
  number_of_episodes: 62,
  status: 'Ended',
  vote_average: 8.9,
  networks: [{ name: 'AMC' }],
  seasons: [
    { season_number: 0, episode_count: 5 },
    { season_number: 1, episode_count: 7 },
    { season_number: 2, episode_count: 13 },
  ],
};

const SEASON_1 = {
  episodes: [
    { episode_number: 1, season_number: 1, name: 'Pilot', overview: 'It begins.', air_date: '2008-01-20', still_path: '/a.jpg' },
    { episode_number: 2, season_number: 1, name: "Cat's in the Bag...", overview: 'Cleanup.', air_date: '2008-01-27', still_path: null },
  ],
};

const SEARCH = {
  results: [
    { id: 346, media_type: 'movie', title: 'Seven Samurai', original_title: '七人の侍', release_date: '1954-04-26', overview: 'x', poster_path: '/p.jpg' },
    { id: 1396, media_type: 'tv', name: 'Breaking Bad', original_name: 'Breaking Bad', first_air_date: '2008-01-20', overview: 'y', poster_path: '/q.jpg' },
    { id: 99, media_type: 'person', name: 'Toshiro Mifune' },
  ],
};

// Route stubbed responses by URL path, and record what got requested so the
// season-fallback assertions can check which calls were actually made.
const requested = [];
globalThis.fetch = async (url) => {
  const path = new URL(url).pathname;
  requested.push(path);
  const body = path === '/3/search/multi' ? SEARCH
    : path === '/3/movie/346' ? MOVIE
      : path === '/3/tv/1396' ? SHOW
        : path === '/3/tv/1396/season/1' ? SEASON_1
          : path === '/3/tv/1396/season/9' ? { episodes: [] }
            : null;
  if (!body) return { ok: false, status: 404, statusText: 'Not Found', json: async () => ({}) };
  return { ok: true, status: 200, statusText: 'OK', json: async () => body };
};

const { search, fetchById } = await import('../src/main/handlers/tmdbHandler.ts');

// --- search ---------------------------------------------------------------

const results = await search('samurai');
assert.equal(results.length, 2, 'person results are dropped, films and shows kept');
assert.equal(results[0].kind, 'movie');
assert.equal(results[0].year, 1954, 'year comes off release_date for a film');
assert.equal(results[1].kind, 'tv');
assert.equal(results[1].year, 2008, 'year comes off first_air_date for a show');
assert.ok(results[0].posterUrl?.startsWith('https://image.tmdb.org/t/p/'), 'poster paths become absolute URLs');
assert.equal(results[0].originalTitle, '七人の侍', 'a differing original title is surfaced');
assert.equal(results[1].originalTitle, null, 'an identical original title is not repeated');

// --- a film ---------------------------------------------------------------

const movie = await fetchById(346, 'movie');
// Every field anilistHandler.formatMetadata produces has to be present, or a
// downstream consumer reads undefined.
for (const field of [
  'seriesId', 'title', 'titleRomaji', 'titleEnglish', 'titleNative', 'description',
  'genres', 'poster', 'banner', 'episodes', 'totalEpisodes', 'duration', 'season',
  'seasonYear', 'status', 'format', 'averageScore', 'studios', 'startDate', 'endDate',
  'anilistId', 'malId',
]) {
  assert.ok(field in movie, `series record is missing ${field}`);
}
assert.equal(movie.seriesId, 'tmdb_movie_346');
assert.equal(movie.format, 'MOVIE');
assert.equal(movie.totalEpisodes, 1, 'a film is one episode as far as the library is concerned');
assert.equal(movie.episodes.length, 1);
assert.equal(movie.episodes[0].episodeNumber, 1);
assert.equal(movie.duration, 207);
// averageScore is stored in the provider's NATIVE scale and normalised at
// render time off `source` (normalizeRating divides only for AniList). TMDB is
// already 0-10, so rescaling it here would render as "85.0".
assert.equal(movie.averageScore, 8.5, 'vote_average is kept on its native 0-10 scale');
assert.equal(movie.anilistId, null, 'no tracker ids - AniList/MAL do not track live-action');
assert.equal(movie.malId, null);
assert.equal(movie.tmdbId, 346);
assert.deepEqual(movie.studios, ['Toho']);

// --- a show ---------------------------------------------------------------

const show = await fetchById(1396, 'tv', 1);
assert.equal(show.seriesId, 'tmdb_tv_1396');
assert.equal(show.format, 'TV');
assert.equal(show.totalEpisodes, 62, 'total comes from the show, not the fetched season');
assert.equal(show.episodes.length, 2);
assert.equal(show.episodes[0].title, 'Pilot');
assert.equal(show.episodes[0].airDate, '2008-01-20');
assert.ok(show.episodes[0].thumbnail?.includes('/a.jpg'), 'stills become absolute URLs');
assert.equal(show.episodes[1].thumbnail, null, 'a missing still stays null for the local-frame fallback');
assert.equal(show.duration, 45);

// --- season fallback ------------------------------------------------------
// A folder numbered past what TMDB lists must still get episode titles rather
// than an empty list, and must never fall back to season 0 (TMDB's specials).

requested.length = 0;
const fallback = await fetchById(1396, 'tv', 9);
assert.ok(requested.includes('/3/tv/1396/season/9'), 'the requested season is tried first');
assert.ok(requested.includes('/3/tv/1396/season/1'), 'an empty season falls back to the first real one');
assert.ok(
  !requested.includes('/3/tv/1396/season/0'),
  'season 0 is TMDB specials and is never the fallback',
);
assert.equal(fallback.episodes.length, 2, 'the fallback season supplied the episodes');

console.log('verify-tmdb-shape: all assertions passed');
