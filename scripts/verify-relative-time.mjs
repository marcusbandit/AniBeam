import assert from 'node:assert/strict';

const { fmtShort, fmtCountdown, fmtVerbose } =
  await import('../src/renderer/utils/relativeTime.ts');

// Fixed reference instant so every assertion is deterministic.
const NOW = Date.UTC(2026, 6, 6, 12, 0, 0);
const MIN = 60_000;
const HOUR = 60 * MIN;
const DAY = 24 * HOUR;

// --- fmtShort: past ---
assert.equal(fmtShort(NOW, NOW), 'just now');                       // zero diff
assert.equal(fmtShort(NOW - 30_000, NOW), 'just now');              // sub-minute
assert.equal(fmtShort(NOW - 53 * MIN, NOW), '53m ago');
assert.equal(fmtShort(NOW - 5 * HOUR, NOW), '5h ago');
assert.equal(fmtShort(NOW - 23 * HOUR, NOW), '23h ago');
assert.equal(fmtShort(NOW - 3 * DAY, NOW), '3d ago');
assert.equal(fmtShort(NOW - 29 * DAY, NOW), '29d ago');
assert.equal(fmtShort(NOW - 30 * DAY, NOW), '1mo ago');             // day→month boundary
assert.equal(fmtShort(NOW - 60 * DAY, NOW), '2mo ago');
assert.equal(fmtShort(NOW - 11 * 30 * DAY, NOW), '11mo ago');
assert.equal(fmtShort(NOW - 12 * 30 * DAY, NOW), '1y ago');         // exactly 12 months rolls to years
assert.equal(fmtShort(NOW - 14 * 30 * DAY, NOW), '1y 2mo ago');
assert.equal(fmtShort(NOW - 24 * 30 * DAY, NOW), '2y ago');

// --- fmtShort: future ---
assert.equal(fmtShort(NOW + 30_000, NOW), 'just now');
assert.equal(fmtShort(NOW + 53 * MIN, NOW), 'in 53m');
assert.equal(fmtShort(NOW + 2 * DAY, NOW), 'in 2d');
assert.equal(fmtShort(NOW + 12 * 30 * DAY, NOW), 'in 1y');
assert.equal(fmtShort(NOW + 14 * 30 * DAY, NOW), 'in 1y 2mo');

// --- fmtShort: nowMs defaults to Date.now() ---
assert.equal(fmtShort(Date.now()), 'just now');

// --- fmtCountdown: matches the existing card style ("3d 18h 19m") ---
assert.equal(fmtCountdown(0), 'now');
assert.equal(fmtCountdown(-5), 'now');                              // already passed
assert.equal(fmtCountdown(0.5), '0m');                              // sub-minute, same as formatCountdownMinutes
assert.equal(fmtCountdown(19), '19m');
assert.equal(fmtCountdown(60), '1h 00m');
assert.equal(fmtCountdown(79), '1h 19m');
assert.equal(fmtCountdown(1440), '1d 00h 00m');
assert.equal(fmtCountdown(1747), '1d 05h 07m');                     // lower units zero-padded
assert.equal(fmtCountdown(3 * 1440 + 18 * 60 + 19), '3d 18h 19m');
assert.equal(fmtCountdown(79.9), '1h 19m');                         // fractional minutes floor

// --- fmtVerbose: behavioural port of airingUtils.formatRelativeDate ---
assert.equal(fmtVerbose(NOW, NOW), '1 minute ago');                 // zero diff clamps to 1 minute
assert.equal(fmtVerbose(NOW - 5 * MIN, NOW), '5 minutes ago');
assert.equal(fmtVerbose(NOW - 3 * HOUR, NOW), '3 hours ago');
assert.equal(fmtVerbose(NOW - 25 * HOUR, NOW), 'yesterday');
assert.equal(fmtVerbose(NOW + 25 * HOUR, NOW), 'tomorrow');
assert.equal(fmtVerbose(NOW - 2 * DAY, NOW), '2 days ago');
assert.equal(fmtVerbose(NOW - 7 * DAY, NOW), '1 week ago');
assert.equal(fmtVerbose(NOW + 7 * DAY, NOW), 'in 1 week');
assert.equal(fmtVerbose(NOW - 20 * DAY, NOW), '3 weeks ago');
assert.equal(fmtVerbose(NOW - 45 * DAY, NOW), '2 months ago');
// Exactly 12 months (360 days) is still inside the 365-day year window,
// so verbose stays in months; the short form is where 12mo rolls to 1y.
assert.equal(fmtVerbose(NOW - 12 * 30 * DAY, NOW), '12 months ago');
assert.equal(fmtVerbose(NOW - 400 * DAY, NOW), '1 year ago');
assert.equal(fmtVerbose(NOW - 2 * 365 * DAY, NOW), '2 years ago');
assert.equal(fmtVerbose(NOW + 2 * DAY, NOW), 'in 2 days');

// --- fmtVerbose: nowMs defaults to Date.now() ---
assert.equal(fmtVerbose(Date.now() - 2 * DAY), '2 days ago');

console.log('relative-time: ok');
