import assert from 'node:assert/strict';
import { relationLane, relationLabel, compareByYear } from '../src/renderer/components/franchise/laneAssignment.ts';

// spine
assert.equal(relationLane('PREQUEL', 'ANIME', 'TV'), 'spine');
assert.equal(relationLane('SEQUEL', 'ANIME', 'TV'), 'spine');
// upstream
assert.equal(relationLane('SOURCE', 'MANGA', 'MANGA'), 'top');
assert.equal(relationLane('PARENT', 'ANIME', 'TV'), 'top');
// ADAPTATION direction by media type
assert.equal(relationLane('ADAPTATION', 'MANGA', 'MANGA'), 'top');   // print = upstream
assert.equal(relationLane('ADAPTATION', 'ANIME', 'TV'), 'bottom');   // screen = downstream
// ADAPTATION with null format falls back to media type
assert.equal(relationLane('ADAPTATION', 'MANGA', null), 'top');
assert.equal(relationLane('ADAPTATION', 'ANIME', null), 'bottom');
// branch + excluded
assert.equal(relationLane('ALTERNATIVE', 'ANIME', 'TV'), 'sidebranch');
assert.equal(relationLane('CHARACTER', 'ANIME', 'TV'), 'excluded');
// downstream defaults
assert.equal(relationLane('SIDE_STORY', 'ANIME', 'OVA'), 'bottom');
assert.equal(relationLane('SPIN_OFF', 'ANIME', 'TV'), 'bottom');
assert.equal(relationLane('OTHER', 'ANIME', 'TV'), 'bottom');

// labels
assert.equal(relationLabel('SIDE_STORY'), 'Side story');
assert.equal(relationLabel('SOURCE'), 'Source');
assert.equal(relationLabel('CHARACTER'), 'Shared characters');
assert.equal(relationLabel('WEIRD_NEW_TYPE'), 'weird new type');

// chronological sort, nulls last
const sorted = [{ seasonYear: 2010 }, { seasonYear: null }, { seasonYear: 2005 }].sort(compareByYear);
assert.deepEqual(sorted.map((s) => s.seasonYear), [2005, 2010, null]);

console.log('OK: franchise lanes');
