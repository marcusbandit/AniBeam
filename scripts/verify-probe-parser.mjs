import assert from 'node:assert/strict';
import { mock } from 'bun:test';

// stub electron + path that the logger imports — the probe handler imports the logger
mock.module('electron', () => ({ BrowserWindow: { getAllWindows: () => [] } }));

const { parseFfprobeJson } = await import('../src/main/handlers/videoProbeHandler.ts');

assert.deepEqual(
  parseFfprobeJson(JSON.stringify({ streams: [{ codec_type: 'video' }], format: { duration: '1234.5' } })),
  { ready: true },
);
assert.deepEqual(
  parseFfprobeJson(JSON.stringify({ streams: [{ codec_type: 'audio' }], format: { duration: '10' } })),
  { ready: false, reason: 'no video stream' },
);
assert.deepEqual(
  parseFfprobeJson(JSON.stringify({ streams: [{ codec_type: 'video' }], format: { duration: '0' } })),
  { ready: false, reason: 'no duration' },
);
assert.deepEqual(
  parseFfprobeJson('not json'),
  { ready: false, reason: 'invalid ffprobe output' },
);
console.log('OK: ffprobe parser');

// --- parseDisplayAspect (aspect backfill for player chrome sizing) ---
const { parseDisplayAspect } = await import('../src/main/utils/transcodeProbe.ts');

// Plain 16:9 storage, square pixels.
assert.equal(parseDisplayAspect({ width: 1920, height: 1080 }), 1920 / 1080);
// Explicit DAR wins.
assert.equal(parseDisplayAspect({ width: 720, height: 576, display_aspect_ratio: '16:9' }), 16 / 9);
// Anamorphic: SAR corrects storage dimensions.
assert.equal(parseDisplayAspect({ width: 720, height: 576, sample_aspect_ratio: '64:45' }), (720 * (64 / 45)) / 576);
// Degenerate SAR strings fall back to square pixels.
assert.equal(parseDisplayAspect({ width: 1280, height: 720, sample_aspect_ratio: '0:1' }), 1280 / 720);
// Missing dimensions => unknown.
assert.equal(parseDisplayAspect({}), null);
assert.equal(parseDisplayAspect({ width: 0, height: 1080 }), null);
// Implausible ratios rejected.
assert.equal(parseDisplayAspect({ width: 10000, height: 10 }), null);
console.log('OK: display aspect parser');
