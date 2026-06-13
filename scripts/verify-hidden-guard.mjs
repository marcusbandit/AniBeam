import assert from 'node:assert/strict';
import { isSeriesHidden } from '../src/shared/hiddenMatch.ts';

const meta = {
  a: { anilistId: 111, malId: 222, hidden: true },
  b: { anilistId: 333, malId: 444 },
  c: { anilistId: 555, malId: null, hidden: false },
};

// Hidden series matches by either provider id.
assert.equal(isSeriesHidden(meta, 'anilist', 111), true, 'hidden anilist id matches');
assert.equal(isSeriesHidden(meta, 'mal', 222), true, 'hidden mal id matches');
// Visible series never matches.
assert.equal(isSeriesHidden(meta, 'anilist', 333), false, 'visible anilist id not hidden');
assert.equal(isSeriesHidden(meta, 'mal', 444), false, 'visible mal id not hidden');
assert.equal(isSeriesHidden(meta, 'anilist', 555), false, 'hidden:false not hidden');
// Unknown / zero ids short-circuit to false.
assert.equal(isSeriesHidden(meta, 'anilist', 999), false, 'unknown id not hidden');
assert.equal(isSeriesHidden(meta, 'mal', 0), false, 'zero id short-circuits');
// Provider isolation: anilistId 111 is hidden, but 111 as a MAL id is not.
assert.equal(isSeriesHidden(meta, 'mal', 111), false, 'provider ids do not cross');

console.log('verify-hidden-guard: OK');
