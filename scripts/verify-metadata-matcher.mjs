import assert from 'node:assert/strict';
import { mock } from 'bun:test';

// Locks the AniList-only contract of the metadata best-matcher.
// findBestMatch (src/main/utils/metadataMatcher.ts) queries AniList's
// relevance-ordered search (the same list the manual picker shows) and
// NOTHING else: MAL (Jikan) was removed as a metadata source by explicit
// user decision (2026-07-14) and must never be consulted for matching.
// (malHandler.getEpisodes survives solely as the per-episode title
// side-fetch used elsewhere; the matcher must not touch it.) Below
// MIN_TITLE_SCORE the matcher refuses with a signal-level warn so bad
// data never lands in metadata.json.
//
// Handlers are the real singletons with their network methods
// monkey-patched; anilistHandler.formatMetadata stays REAL so the
// fixture exercises the actual field mapping. Calls are recorded in
// `calls`, warns in `warns`.

// logger.ts imports BrowserWindow to broadcast to renderer windows; stub
// electron so importing the matcher works outside the app.
mock.module('electron', () => ({
  BrowserWindow: { getAllWindows: () => [] },
}));

const malHandler = (await import('../src/main/handlers/malHandler.ts')).default;
const anilistHandler = (await import('../src/main/handlers/anilistHandler.ts')).default;
const { findBestMatch } = await import('../src/main/utils/metadataMatcher.ts');
const { bestTitleScore } = await import('../src/main/utils/titleSimilarity.ts');
const { logger } = await import('../src/main/services/logger.ts');

const MIN_TITLE_SCORE = 0.4; // keep in sync with metadataMatcher.ts

// --- fixtures ---------------------------------------------------------------
// Complete enough for the REAL anilistHandler.formatMetadata.

const aniMedia = (over = {}) => ({
  id: 1234,
  idMal: 47917,
  title: { romaji: 'Bocchi the Rock!', english: 'Bocchi the Rock!', native: 'BTR native' },
  synonyms: ['BTR'],
  description: 'Guitar hermit forms a band.',
  genres: ['Music'],
  coverImage: { large: 'https://img.example/l.jpg', extraLarge: 'https://img.example/xl.jpg' },
  bannerImage: null,
  episodes: 12,
  duration: 24,
  season: 'FALL',
  seasonYear: 2022,
  status: 'FINISHED',
  format: 'TV',
  startDate: { year: 2022, month: 10, day: 9 },
  endDate: { year: 2022, month: 12, day: 25 },
  averageScore: 87,
  studios: { nodes: [{ name: 'CloverWorks' }] },
  ...over,
});

// --- stub installer ----------------------------------------------------------

const calls = [];
const warns = [];
function install({ anilist = async () => [] } = {}) {
  calls.length = 0;
  warns.length = 0;
  anilistHandler.searchAnimeMultiple = async (q, limit) => {
    calls.push('anilist.search');
    return anilist(q, limit);
  };
  anilistHandler.getEpisodes = async () => {
    calls.push('anilist.episodes');
    return [];
  };
  // getEpisodes is the KEPT Jikan episode-title side-fetch (used by
  // fetchEpisodeAirDates, not by matching). A call from the matcher is a
  // contract violation, so the stub throws.
  malHandler.getEpisodes = async () => {
    calls.push('mal.episodes');
    throw new Error('malHandler.getEpisodes must never be consulted during matching');
  };
  logger.warn = (_category, message) => {
    warns.push(String(message));
  };
}

let passed = 0;

// ===========================================================================
// Case 0 (structural): the MAL search/metadata surface is GONE from the
// handler, not merely bypassed. If someone re-adds it, this fails first.
// ===========================================================================
{
  assert.equal(typeof malHandler.searchAnime, 'undefined', 'case 0: malHandler.searchAnime must no longer exist');
  assert.equal(typeof malHandler.searchAndFetchMetadata, 'undefined', 'case 0: malHandler.searchAndFetchMetadata must no longer exist');
  assert.equal(typeof malHandler.formatMetadata, 'undefined', 'case 0: malHandler.formatMetadata must no longer exist');
  assert.equal(typeof malHandler.getEpisodes, 'function', 'case 0: the Jikan episode side-fetch must stay');
  passed++;
}

// ===========================================================================
// Case 1: AniList has a strong match (score >= MIN_TITLE_SCORE). AniList wins
// and Jikan is never touched.
// ===========================================================================
{
  install({ anilist: async () => [aniMedia()] });
  const res = await findBestMatch('Bocchi the Rock', null, null, undefined);
  assert.ok(res, 'case 1: expected a match');
  assert.equal(res.source, 'anilist', 'case 1: source must be anilist');
  assert.ok(res.score >= MIN_TITLE_SCORE, `case 1: score ${res.score} must clear ${MIN_TITLE_SCORE}`);
  assert.ok(
    !calls.includes('mal.episodes'),
    `case 1: Jikan must never be consulted during matching (calls: ${calls.join(', ')})`,
  );
  assert.equal(res.metadata.anilistId, 1234, 'case 1: AniList metadata carries anilistId');
  assert.equal(res.metadata.titleRomaji, 'Bocchi the Rock!', 'case 1: AniList metadata carries titleRomaji');
  passed++;
}

// ===========================================================================
// Case 2: AniList returns nothing. There is no fallback provider anymore:
// the matcher returns null and Jikan stays untouched.
// ===========================================================================
{
  install({ anilist: async () => [] });
  const res = await findBestMatch('Bocchi the Rock', null, null, undefined);
  assert.equal(res, null, 'case 2: empty AniList must return null (no MAL fallback)');
  assert.ok(calls.includes('anilist.search'), 'case 2: AniList must have been searched');
  assert.ok(
    !calls.includes('mal.episodes'),
    `case 2: Jikan must never be consulted during matching (calls: ${calls.join(', ')})`,
  );
  assert.ok(
    warns.some((w) => w.includes('No candidates')),
    `case 2: expected the no-candidates warn (warns: ${warns.join(' | ')})`,
  );
  passed++;
}

// ===========================================================================
// Case 3: AniList search THROWS (network error). Matcher must not crash: the
// failure degrades to "no candidates" and returns null.
// ===========================================================================
{
  install({
    anilist: async () => { throw new Error('ECONNRESET'); },
  });
  const res = await findBestMatch('Bocchi the Rock', null, null, undefined);
  assert.equal(res, null, 'case 3: a dead AniList must yield null, not a crash');
  assert.ok(
    warns.some((w) => w.includes('AniList search failed')),
    `case 3: expected the search-failed warn (warns: ${warns.join(' | ')})`,
  );
  assert.ok(!calls.includes('mal.episodes'), 'case 3: Jikan must stay untouched');
  passed++;
}

// ===========================================================================
// Case 4: best candidate scores below MIN_TITLE_SCORE. Returns null with the
// refuse warn. The classic titleSimilarity false friend: shares only
// particles with the query.
// ===========================================================================
{
  const query = 'Otaku ni Yasashii Gal wa Inai';
  const wotakuTitles = ['Wotaku ni Koi wa Muzukashii', 'Wotakoi: Love is Hard for Otaku'];
  const premise = bestTitleScore(query, wotakuTitles);
  assert.ok(premise < MIN_TITLE_SCORE, `case 4 premise: fixture score ${premise} must be below ${MIN_TITLE_SCORE}`);
  install({
    anilist: async () => [
      aniMedia({
        id: 99,
        idMal: 100,
        title: { romaji: wotakuTitles[0], english: wotakuTitles[1], native: 'Wotakoi native' },
        synonyms: [],
      }),
    ],
  });
  const res = await findBestMatch(query, null, null, undefined);
  assert.equal(res, null, 'case 4: below-threshold candidates must be refused');
  assert.ok(
    warns.some((w) => w.includes('refusing to match')),
    `case 4: expected the refuse warn (warns: ${warns.join(' | ')})`,
  );
  assert.ok(!calls.includes('mal.episodes'), 'case 4: Jikan must stay untouched');
  passed++;
}

console.log(`verify-metadata-matcher: ${passed} cases passed`);
