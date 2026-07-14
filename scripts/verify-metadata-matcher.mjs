import assert from 'node:assert/strict';
import { mock } from 'bun:test';

// Locks the provider-order contract of the metadata best-matcher.
// findBestMatch (src/main/utils/metadataMatcher.ts) must mirror
// posterMatch.ts: AniList's relevance-ordered search is primary (the same
// list the manual picker shows) and MAL (Jikan) is a fallback consulted
// ONLY when AniList yields nothing at or over MIN_TITLE_SCORE. MAL-sourced
// metadata must reach renderer parity: titleRomaji present, and anilistId
// cross-resolved best-effort (key omitted entirely when unresolved so a
// merge never clobbers an existing good id).
//
// Handlers are the real singletons with their network methods
// monkey-patched; formatMetadata on both sides stays REAL so the fixtures
// exercise the actual field mapping. Call order is recorded in `calls`.

// logger.ts imports BrowserWindow to broadcast to renderer windows; stub
// electron so importing the matcher works outside the app.
mock.module('electron', () => ({
  BrowserWindow: { getAllWindows: () => [] },
}));

const malHandler = (await import('../src/main/handlers/malHandler.ts')).default;
const anilistHandler = (await import('../src/main/handlers/anilistHandler.ts')).default;
const { findBestMatch } = await import('../src/main/utils/metadataMatcher.ts');
const { bestTitleScore } = await import('../src/main/utils/titleSimilarity.ts');

const MIN_TITLE_SCORE = 0.4; // keep in sync with metadataMatcher.ts

// --- fixtures ---------------------------------------------------------------
// Complete enough for the REAL formatMetadata of each handler.

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

const jikanAnime = (over = {}) => ({
  mal_id: 47917,
  title: 'Bocchi the Rock!',
  title_english: 'Bocchi the Rock!',
  title_japanese: 'BTR native',
  synopsis: 'Guitar hermit forms a band.',
  background: null,
  genres: [{ name: 'Music' }],
  images: { jpg: { image_url: 'https://img.example/s.jpg', large_image_url: 'https://img.example/l.jpg' } },
  episodes: 12,
  status: 'Finished Airing',
  type: 'TV',
  score: 8.8,
  studios: [{ name: 'CloverWorks' }],
  aired: { from: '2022-10-09T00:00:00+00:00', to: '2022-12-25T00:00:00+00:00' },
  season: 'fall',
  year: 2022,
  ...over,
});

// --- stub installer ----------------------------------------------------------

const calls = [];
function install({ anilist = async () => [], mal = async () => [], resolveMal = async () => null } = {}) {
  calls.length = 0;
  anilistHandler.searchAnimeMultiple = async (q, limit) => {
    calls.push('anilist.search');
    return anilist(q, limit);
  };
  anilistHandler.getEpisodes = async () => {
    calls.push('anilist.episodes');
    return [];
  };
  anilistHandler.resolveAnilistIdByMal = async (id) => {
    calls.push('anilist.resolveByMal');
    return resolveMal(id);
  };
  malHandler.searchAnime = async (q, limit) => {
    calls.push('mal.search');
    return mal(q, limit);
  };
  malHandler.getEpisodes = async () => {
    calls.push('mal.episodes');
    return [];
  };
}

let passed = 0;

// ===========================================================================
// Case 1: AniList has a strong match (score >= MIN_TITLE_SCORE). AniList wins
// outright and MAL is NEVER consulted.
// ===========================================================================
{
  install({ anilist: async () => [aniMedia()] });
  const res = await findBestMatch('Bocchi the Rock', null, null, undefined);
  assert.ok(res, 'case 1: expected a match');
  assert.equal(res.source, 'anilist', 'case 1: source must be anilist');
  assert.ok(res.score >= MIN_TITLE_SCORE, `case 1: score ${res.score} must clear ${MIN_TITLE_SCORE}`);
  assert.ok(
    !calls.includes('mal.search'),
    `case 1: MAL must never be consulted when AniList matches (calls: ${calls.join(', ')})`,
  );
  assert.equal(res.metadata.anilistId, 1234, 'case 1: AniList metadata carries anilistId');
  assert.equal(res.metadata.titleRomaji, 'Bocchi the Rock!', 'case 1: AniList metadata carries titleRomaji');
  passed++;
}

// ===========================================================================
// Case 2: AniList returns nothing, MAL fallback runs. Order is AniList first,
// then MAL. MAL metadata reaches parity: titleRomaji from Jikan's default
// title, malId set, anilistId cross-resolved via resolveAnilistIdByMal.
// ===========================================================================
{
  install({ anilist: async () => [], mal: async () => [jikanAnime()], resolveMal: async () => 4321 });
  const res = await findBestMatch('Bocchi the Rock', null, null, undefined);
  assert.ok(res, 'case 2: expected a MAL fallback match');
  assert.equal(res.source, 'mal', 'case 2: source must be mal');
  const aniIdx = calls.indexOf('anilist.search');
  const malIdx = calls.indexOf('mal.search');
  assert.ok(aniIdx !== -1 && malIdx !== -1 && aniIdx < malIdx, `case 2: AniList search must precede MAL (calls: ${calls.join(', ')})`);
  assert.equal(res.metadata.titleRomaji, 'Bocchi the Rock!', "case 2: titleRomaji must be Jikan's default title");
  assert.equal(res.metadata.malId, 47917, 'case 2: malId must be set');
  assert.equal(res.metadata.anilistId, 4321, 'case 2: anilistId must be the cross-resolved id');
  // Formatting parity extras: Jikan's lowercase season is normalized to
  // AniList's uppercase convention, and year maps to seasonYear.
  assert.equal(res.metadata.season, 'FALL', 'case 2: season normalized to AniList convention');
  assert.equal(res.metadata.seasonYear, 2022, 'case 2: seasonYear mapped from Jikan year');
  passed++;
}

// ===========================================================================
// Case 3: AniList search THROWS (network error). Matcher must not crash: the
// failure counts as "no AniList candidates" and MAL still produces the match.
// ===========================================================================
{
  install({
    anilist: async () => { throw new Error('ECONNRESET'); },
    mal: async () => [jikanAnime()],
    resolveMal: async () => 4321,
  });
  const res = await findBestMatch('Bocchi the Rock', null, null, undefined);
  assert.ok(res, 'case 3: matcher must fall through to MAL when AniList throws');
  assert.equal(res.source, 'mal', 'case 3: source must be mal');
  passed++;
}

// ===========================================================================
// Case 4: resolveAnilistIdByMal resolves null. The returned metadata must NOT
// contain an anilistId key at all, so downstream merges never clobber an
// existing good id with null/undefined.
// ===========================================================================
{
  install({ anilist: async () => [], mal: async () => [jikanAnime()], resolveMal: async () => null });
  const res = await findBestMatch('Bocchi the Rock', null, null, undefined);
  assert.ok(res, 'case 4: expected a MAL fallback match');
  assert.equal(res.source, 'mal', 'case 4: source must be mal');
  assert.ok(
    !('anilistId' in res.metadata),
    'case 4: anilistId key must be omitted entirely when unresolved',
  );
  passed++;
}

// ===========================================================================
// Case 5: neither provider clears MIN_TITLE_SCORE. Returns null, and MAL WAS
// still consulted (AniList's below-threshold candidate must not end the hunt).
// The classic titleSimilarity false friend: shares only particles.
// ===========================================================================
{
  const query = 'Otaku ni Yasashii Gal wa Inai';
  const wotakuTitles = ['Wotaku ni Koi wa Muzukashii', 'Wotakoi: Love is Hard for Otaku'];
  const premise = bestTitleScore(query, wotakuTitles);
  assert.ok(premise < MIN_TITLE_SCORE, `case 5 premise: fixture score ${premise} must be below ${MIN_TITLE_SCORE}`);
  install({
    anilist: async () => [
      aniMedia({
        id: 99,
        idMal: 100,
        title: { romaji: wotakuTitles[0], english: wotakuTitles[1], native: 'Wotakoi native' },
        synonyms: [],
      }),
    ],
    mal: async () => [
      jikanAnime({ mal_id: 100, title: wotakuTitles[0], title_english: wotakuTitles[1], title_japanese: null }),
    ],
    resolveMal: async () => 4321,
  });
  const res = await findBestMatch(query, null, null, undefined);
  assert.equal(res, null, 'case 5: below-threshold candidates must be refused');
  assert.ok(
    calls.includes('mal.search'),
    `case 5: MAL must still be consulted when AniList scores below threshold (calls: ${calls.join(', ')})`,
  );
  passed++;
}

console.log(`verify-metadata-matcher: ${passed} cases passed`);
