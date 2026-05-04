import assert from 'node:assert/strict';

// Bun supports --preload; we mock electron via a separate preload file.
// Since we're running standalone, we use Bun's module mock API.
import { mock } from 'bun:test';

mock.module('electron', () => ({
  BrowserWindow: { getAllWindows: () => [] },
}));

const { logger } = await import('../src/main/services/logger.ts');

logger.clear();
logger.info('system', 'boot');
logger.warn('folder', 'something odd');
logger.error('probe', 'ffprobe failed', { file: '/tmp/x.mkv' });
const buf = logger.getBuffer();
assert.equal(buf.length, 3);
assert.equal(buf[0].level, 'info');
assert.equal(buf[1].stage, 'folder');
assert.equal(buf[2].ctx?.file, '/tmp/x.mkv');
assert.ok(buf[0].id < buf[2].id, 'ids are monotonic');

// Ring-buffer cap
logger.clear();
for (let i = 0; i < 6000; i++) logger.info('system', `m${i}`);
const after = logger.getBuffer();
assert.equal(after.length, 5000);
assert.equal(after[after.length - 1].message, 'm5999');
console.log('OK: logger ring buffer behaves');
