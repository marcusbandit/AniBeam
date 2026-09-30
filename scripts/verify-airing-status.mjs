// The watched rules and the status comparison in src/shared/airingStatus.ts.
// isWatchedOut is what files a series into the Watched tab and out of the
// airing rail; statusDiffers is what makes the airing refresh's status
// write the one notice that a series stopped airing — the full fetch that
// normally carries the status ran once, at match time, and never again.
import assert from 'node:assert/strict';
import { normalizeStatus, isWatchedThrough, isWatchedOut, statusDiffers } from '../src/shared/airingStatus.ts';

// --- the through rule: the Progress sort's tier ---

// Completed on the tracker says it outright.
assert.equal(isWatchedThrough({ listStatus: 'completed', watched: 4, totalEpisodes: 12 }), true, 'tracker completed is watched through');
// Reached the total.
assert.equal(isWatchedThrough({ watched: 12, totalEpisodes: 12 }), true, 'watched up to the total is through');
assert.equal(isWatchedThrough({ watched: 5, totalEpisodes: 12 }), false, 'midway is not through');
// No published total: a film with any watch at all is done; a series is not.
assert.equal(isWatchedThrough({ watched: 1, totalEpisodes: null }), true, 'a film with any watch is through');
assert.equal(isWatchedThrough({ watched: 0, totalEpisodes: null }), false, 'zero watch is not through');
// Untracked is never through.
assert.equal(isWatchedThrough({ watched: null, totalEpisodes: 12 }), false, 'untracked is not through');
// A zero total is no total.
assert.equal(isWatchedThrough({ watched: 1, totalEpisodes: 0 }), false, 'a zero total is no total');

// --- the out rule: the Watched tab and the airing rail's gate ---

// A still-releasing series is never done, however current the user is —
// caught up is current, not finished. Even the tracker's own COMPLETED
// waits for the end: while it says releasing, it stays among the living.
assert.equal(isWatchedOut({ status: 'RELEASING', listStatus: 'completed', watched: 12, totalEpisodes: 12 }), false, 'releasing is never watched out');
assert.equal(isWatchedOut({ status: 'Currently Airing', watched: 8, totalEpisodes: null }), false, 'releasing with no total is not out either');
// Finished and fully seen: out.
assert.equal(isWatchedOut({ status: 'FINISHED', watched: 12, totalEpisodes: 12 }), true, 'finished and seen is out');
assert.equal(isWatchedOut({ status: 'FINISHED', listStatus: 'completed', watched: 12, totalEpisodes: 12 }), true, 'tracker completed and finished is out');
// Finished but midway: not out.
assert.equal(isWatchedOut({ status: 'FINISHED', watched: 5, totalEpisodes: 12 }), false, 'finished but midway is not out');
// Finished, no total, some watch: with nothing published to reach, the
// film rule applies — any watch at all is done.
assert.equal(isWatchedOut({ status: 'FINISHED', watched: 3, totalEpisodes: null }), true, 'finished with no total follows the film rule');
// Untracked and finished: not out.
assert.equal(isWatchedOut({ status: 'FINISHED', watched: null, totalEpisodes: 12 }), false, 'untracked is never out');
// Upcoming and cancelled: the not-releasing gate lets the other rules decide.
assert.equal(isWatchedOut({ status: 'NOT_YET_RELEASED', watched: 0, totalEpisodes: 12 }), false, 'upcoming with no watch is not out');
assert.equal(isWatchedOut({ status: 'CANCELLED', watched: 13, totalEpisodes: 13 }), true, 'cancelled and seen is out');

// --- statusDiffers: the airing refresh's write gate ---

assert.equal(statusDiffers('RELEASING', 'FINISHED'), true, 'a finished answer differs from releasing');
assert.equal(statusDiffers('Currently Airing', 'RELEASING'), false, 'spelling drift between providers is not a change');
assert.equal(statusDiffers('FINISHED', 'FINISHED'), false, 'the same status is not a change');
assert.equal(statusDiffers(null, 'RELEASING'), true, 'no stored status and a fresh one is a change');
assert.equal(statusDiffers('FINISHED', null), false, 'no fresh status writes nothing');
assert.equal(statusDiffers('FINISHED', ''), false, 'an empty fresh status writes nothing');

// --- normalize still speaks both providers' dialects ---

assert.equal(normalizeStatus('Finished Airing'), 'finished');
assert.equal(normalizeStatus('currently_airing'), 'releasing');
assert.equal(normalizeStatus(null), '');

console.log('OK: airing status rules');
