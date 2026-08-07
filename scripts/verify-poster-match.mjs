import assert from 'node:assert/strict';
import { mock } from 'bun:test';

// Locks the scoring rationale behind the auto-matcher's "pretty close" gate,
// plus the AniList-only contract. findShowMatch (src/main/utils/posterMatch.ts)
// accepts a candidate iff its bestTitleScore clears THRESHOLD; there is NO
// fallback provider (MAL was removed as a metadata source by explicit user
// decision, 2026-07-14), so anything AniList can't cover returns null and is
// left for manual matching.
//
// Part 1 pins WHY 0.5 is the right bar with pure scoring cases (no network).
// Part 2 exercises findShowMatch itself with a stubbed AniList handler.

// logger.ts imports BrowserWindow; stub electron so importing posterMatch
// works outside the app.
mock.module('electron', () => ({
  BrowserWindow: { getAllWindows: () => [] },
}));

const { bestTitleScore } = await import('../src/main/utils/titleSimilarity.ts');
const malHandler = (await import('../src/main/handlers/malHandler.ts')).default;
const anilistHandler = (await import('../src/main/handlers/anilistHandler.ts')).default;
const { findShowMatch } = await import('../src/main/utils/posterMatch.ts');

const THRESHOLD = 0.5; // keep in sync with posterMatch.ts

const approx = (a, b, eps = 1e-3) => Math.abs(a - b) <= eps;

// --- Part 1: pure scoring around the threshold ------------------------------

// [label, folderName, candidateTitles, shouldAutoApply, expectedScore?]
const cases = [
  // Exact / near-exact folder names sail through.
  ['Bocchi exact', 'Bocchi the Rock', ['Bocchi the Rock!', 'Bocchi the Rock!'], true, 1.0],

  // Romaji primary title, folder uses the short English name. 0.5 is the
  // boundary case the threshold is tuned around - it must be accepted.
  ['Frieren romaji', 'Frieren', ['Sousou no Frieren', "Frieren: Beyond Journey's End"], true, 0.5],

  // Season suffix on the folder still overlaps the English title strongly.
  ['AoT season 3', 'Attack on Titan Season 3', ['Shingeki no Kyojin', 'Attack on Titan'], true, 0.75],

  // Abbreviation WITHOUT synonyms: romaji + english can't reach the bar, so it
  // correctly falls to manual...
  [
    'Konosuba no-synonyms',
    'Konosuba',
    ['Kono Subarashii Sekai ni Shukufuku wo!', "KonoSuba: God's Blessing on This Wonderful World!"],
    false,
  ],
  // ...but WITH the AniList synonym "KonoSuba" it becomes a confident match.
  // This is exactly the lever the matcher change adds.
  [
    'Konosuba +synonym',
    'Konosuba',
    ['Kono Subarashii Sekai ni Shukufuku wo!', "KonoSuba: God's Blessing on This Wonderful World!", 'KonoSuba'],
    true,
    1.0,
  ],

  // Fuzzy-similar but wrong show (shares only particles): must be rejected so a
  // bad match never lands automatically. The classic titleSimilarity fixture.
  ['Wotaku false friend', 'Otaku ni Yasashii Gal wa Inai', ['Wotaku ni Koi wa Muzukashii'], false],
];

let passed = 0;
for (const [label, folder, titles, shouldApply, expectedScore] of cases) {
  const score = bestTitleScore(folder, titles);
  const applies = score >= THRESHOLD;
  assert.equal(
    applies,
    shouldApply,
    `${label}: score ${score.toFixed(3)} vs threshold ${THRESHOLD} -> auto-apply=${applies}, expected ${shouldApply}`,
  );
  if (typeof expectedScore === 'number') {
    assert.ok(
      approx(score, expectedScore),
      `${label}: score ${score.toFixed(3)} expected ~${expectedScore}`,
    );
  }
  passed++;
}

// The synonym lever must strictly help, never hurt: adding a synonym can only
// raise (or hold) the score, since bestTitleScore takes the max over variants.
const base = bestTitleScore('Konosuba', ['Kono Subarashii Sekai ni Shukufuku wo!']);
const withSyn = bestTitleScore('Konosuba', ['Kono Subarashii Sekai ni Shukufuku wo!', 'KonoSuba']);
assert.ok(withSyn >= base, `synonym must not lower score: ${withSyn} < ${base}`);
passed++;

// --- Part 2: AniList-only findShowMatch --------------------------------------

// Structural: the MAL search surface is GONE from the handler, not merely
// bypassed. Only the Jikan episode side-fetch remains.
assert.equal(typeof malHandler.searchAnime, 'undefined', 'malHandler.searchAnime must no longer exist');
assert.equal(typeof malHandler.getEpisodes, 'function', 'the Jikan episode side-fetch must stay');
passed++;

// No AniList match at all -> null. This case previously fell through to the
// MAL fallback; now it must simply refuse.
anilistHandler.searchAnimeMultiple = async () => [];
const emptyRes = await findShowMatch('Otaku ni Yasashii Gal wa Inai');
assert.equal(emptyRes, null, 'no AniList results must return null (no fallback provider)');
passed++;

// A below-threshold AniList candidate -> null too, for the same reason.
anilistHandler.searchAnimeMultiple = async () => [
  {
    id: 99,
    idMal: 100,
    title: { romaji: 'Wotaku ni Koi wa Muzukashii', english: 'Wotakoi: Love is Hard for Otaku', native: 'Wotakoi native' },
    synonyms: [],
    coverImage: { large: 'https://img.example/l.jpg', extraLarge: 'https://img.example/xl.jpg' },
    episodes: 11,
    status: 'FINISHED',
    startDate: { year: 2018, month: 4, day: 13 },
  },
];
const belowRes = await findShowMatch('Otaku ni Yasashii Gal wa Inai');
assert.equal(belowRes, null, 'below-threshold AniList candidate must return null');
passed++;

console.log(`verify-poster-match: ${passed} assertions passed`);
