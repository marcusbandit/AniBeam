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
