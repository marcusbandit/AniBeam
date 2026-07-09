import assert from 'node:assert/strict';

// Locks the scoring rationale behind the auto-matcher's "pretty close" gate.
// findShowMatch (src/main/utils/posterMatch.ts) accepts a candidate iff its
// bestTitleScore clears THRESHOLD. These cases pin WHY 0.5 is the right bar:
// real titles the user expects to auto-match clear it, far-off ones don't, and
// the synonyms lever rescues common abbreviations that romaji/english alone
// would drop to manual. Pure function, no network - safe to run anywhere.

const { bestTitleScore } = await import('../src/main/utils/titleSimilarity.ts');

const THRESHOLD = 0.5; // keep in sync with posterMatch.ts

const approx = (a, b, eps = 1e-3) => Math.abs(a - b) <= eps;

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

console.log(`verify-poster-match: ${passed} assertions passed`);
