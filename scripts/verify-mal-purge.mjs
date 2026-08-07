import assert from 'node:assert/strict';

// Locks the one-time MAL purge helpers (src/main/utils/malPurge.ts). MAL was
// removed as a metadata source (2026-07-14); library entries still registered
// to it must be detected, have their registration nulled, and fall back into
// the normal AniList auto-matcher's "never attempted" pool. Cross-reference
// ids (malId, anilistId) and all persistent per-series fields must survive
// the strip verbatim, and the strip must be pure (no input mutation).

const { isMalRegistered, stripMalRegistration } = await import('../src/main/utils/malPurge.ts');

let passed = 0;

// --- isMalRegistered ---------------------------------------------------------

assert.equal(
  isMalRegistered({ source: 'mal', matchSource: 'mal' }),
  true,
  'source: mal must be detected',
);
passed++;

assert.equal(
  isMalRegistered({ source: 'anilist', matchSource: 'mal' }),
  true,
  'matchSource: mal alone must be detected',
);
passed++;

assert.equal(
  isMalRegistered({ source: 'anilist', matchSource: 'anilist' }),
  false,
  'a pure AniList registration must NOT be detected',
);
passed++;

assert.equal(
  isMalRegistered({ title: 'Sourceless Show', posterMatchAttempted: true }),
  false,
  'a sourceless entry must NOT be detected',
);
passed++;

// --- stripMalRegistration ----------------------------------------------------

const malEntry = {
  seriesId: 'grimgar',
  title: 'Grimgar of Fantasy and Ash',
  folderPath: '/library/Grimgar',
  type: 'series',
  source: 'mal',
  matchSource: 'mal',
  matchedTitle: 'Hai to Gensou no Grimgar',
  matchScore: 0.82,
  posterMatched: true,
  posterMatchAttempted: true,
  poster: 'https://cdn.myanimelist.net/images/anime/grimgar.jpg',
  posterLocal: '/cache/grimgar.jpg',
  malId: 31859,
  anilistId: 21243,
  totalEpisodes: 12,
  fileEpisodes: [
    {
      episodeNumber: 1,
      filePath: '/library/Grimgar/ep01.mkv',
      transcodedPath: '/cache/transcode/abc.mp4',
      status: 'ready',
    },
  ],
};
const snapshot = JSON.parse(JSON.stringify(malEntry));

const stripped = stripMalRegistration(malEntry);

// Purity: a NEW object, and the input untouched.
assert.notEqual(stripped, malEntry, 'strip must return a new object, not the input');
passed++;
assert.deepEqual(
  malEntry,
  snapshot,
  'strip must not mutate the input entry',
);
passed++;

// Registration fields nulled, attempt flags reset so the auto-matcher
// picks the entry up again.
assert.equal(stripped.source, null, 'source must become null');
assert.equal(stripped.matchSource, null, 'matchSource must become null');
assert.equal(stripped.matchedTitle, null, 'matchedTitle must become null');
assert.equal(stripped.matchScore, null, 'matchScore must become null');
assert.equal(stripped.posterMatched, false, 'posterMatched must become false');
assert.equal(stripped.posterMatchAttempted, false, 'posterMatchAttempted must become false');
passed++;

// Cross-reference ids survive: malId keys AniSkip + the Jikan episode-title
// side-fetch regardless of provider; anilistId gives the re-matcher a head
// start. Persistent per-series fields survive verbatim too.
assert.equal(stripped.malId, 31859, 'malId must survive');
assert.equal(stripped.anilistId, 21243, 'anilistId must survive');
assert.deepEqual(stripped.fileEpisodes, snapshot.fileEpisodes, 'fileEpisodes must survive verbatim');
assert.equal(stripped.folderPath, '/library/Grimgar', 'folderPath must survive');
assert.equal(stripped.type, 'series', 'type must survive');
assert.equal(stripped.title, 'Grimgar of Fantasy and Ash', 'title must survive');
assert.equal(stripped.poster, snapshot.poster, 'poster must survive');
passed++;

// A stripped entry no longer reads as MAL-registered, and it re-enters the
// matcher pool (posterMatchAttempted false).
assert.equal(isMalRegistered(stripped), false, 'stripped entry must not be MAL-registered');
assert.equal(stripped.posterMatchAttempted, false, 'stripped entry must be re-matchable');
passed++;

console.log(`verify-mal-purge: ${passed} assertions passed`);
