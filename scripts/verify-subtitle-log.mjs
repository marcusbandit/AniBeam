import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const {
  initSubtitleDebugLog,
  subLog,
  subtitleLogPath,
  flushSubtitleDebugLog,
} = await import('../src/main/services/subtitleDebugLog.ts');

const dir = await mkdtemp(join(tmpdir(), 'anibeam-sublog-'));
const logFile = join(dir, 'subtitles.log');
const prevFile = join(dir, 'subtitles.prev.log');

try {
  // Before init: silent no-op, no file, no path.
  subLog('main/test', 'dropped before init');
  await flushSubtitleDebugLog();
  assert.equal(subtitleLogPath(), null);
  assert.ok(!existsSync(logFile), 'no file before init');

  initSubtitleDebugLog(dir, { version: '1.0.0', pid: 123 });
  assert.equal(subtitleLogPath(), logFile);

  subLog('main/extract', 'plain line');
  subLog('main/extract', 'with data', { file: '/tmp/x.mkv', streamIndex: 2 });
  const circular = { name: 'loop' };
  circular.self = circular;
  subLog('renderer/jassub', 'circular data', circular);
  subLog('main/prewarm', 'error data', new Error('boom'));
  subLog('main/extract', 'multi\nline\nmessage');
  subLog('main/big', 'big data', { blob: 'x'.repeat(10000) });
  await flushSubtitleDebugLog();

  const text = await readFile(logFile, 'utf8');
  const lines = text.trimEnd().split('\n');
  assert.equal(lines.length, 7, 'header + 6 entries, one line each');
  assert.ok(lines[0].startsWith('--- session start '), 'header line first');
  assert.ok(lines[0].includes('"version":"1.0.0"'), 'header carries fields');
  const ISO_PREFIX = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z \[[^\]]+\] /;
  for (const line of lines.slice(1)) assert.match(line, ISO_PREFIX);
  assert.ok(lines[1].endsWith('[main/extract] plain line'), 'no data suffix without data');
  assert.ok(lines[2].includes('[main/extract] with data | {'), 'data joined with pipe');
  assert.ok(lines[2].includes('"streamIndex":2'));
  assert.ok(lines[3].includes('[circular]'), 'circular refs collapse');
  assert.ok(lines[4].includes('boom'), 'Error data serializes to its message');
  assert.ok(lines[5].includes('multi\\nline\\nmessage'), 'newlines flattened');
  assert.ok(lines[6].length < 4200, 'long data capped');
  assert.ok(lines[6].endsWith('...'), 'cap marker present');
  assert.ok(!text.includes('dropped before init'), 'pre-init line never appears');

  // Re-init rotates the session file and starts fresh.
  initSubtitleDebugLog(dir, { version: '1.0.0', pid: 124 });
  subLog('main/test', 'second session line');
  await flushSubtitleDebugLog();
  const prev = await readFile(prevFile, 'utf8');
  assert.ok(prev.includes('plain line'), 'first session rotated to prev');
  const fresh = await readFile(logFile, 'utf8');
  assert.ok(fresh.startsWith('--- session start '), 'fresh session header');
  assert.ok(fresh.includes('"pid":124'));
  assert.ok(fresh.includes('second session line'));
  assert.ok(!fresh.includes('plain line'), 'fresh log has no first-session lines');
} finally {
  await rm(dir, { recursive: true, force: true });
}

console.log('OK: subtitle debug log sink behaves');
