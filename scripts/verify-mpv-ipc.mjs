// End-to-end check of the mpv IPC session: generate a short clip, play it to
// EOF, and assert we actually observed position, duration and watch time.
//
// This is the only thing that proves the JSON-IPC wire format still works —
// the parsing is untestable in the abstract, since what matters is what mpv
// really answers. It needs both `mpv` and `ffmpeg` on PATH and takes about ten
// seconds of real time; it skips cleanly if either is missing.
//
// Playback is headless via MPV_HOME (a temp config with vo=null/ao=null) rather
// than extra CLI args, so the code under test stays exactly what ships.
//
// Run: bun --bun scripts/verify-mpv-ipc.mjs

import assert from 'node:assert/strict';
import { mock } from 'bun:test';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';

function have(bin) {
  return spawnSync('sh', ['-c', `command -v ${bin}`]).status === 0;
}

if (!have('mpv') || !have('ffmpeg')) {
  console.log('verify-mpv-ipc: skipped (needs mpv and ffmpeg on PATH)');
  process.exit(0);
}

const dir = await mkdtemp(join(tmpdir(), 'anibeam-mpv-'));

mock.module('electron', () => ({ app: { getPath: () => dir } }));
mock.module('../src/main/services/logger', () => ({
  logger: { info() {}, warn() {}, error() {} },
}));

const CLIP_SEC = 6;
const clip = join(dir, 'clip.mp4');
const gen = spawnSync('ffmpeg', [
  '-hide_banner', '-loglevel', 'error', '-y',
  '-f', 'lavfi', '-i', `testsrc=size=320x240:rate=10:duration=${CLIP_SEC}`,
  '-f', 'lavfi', '-i', `sine=frequency=440:duration=${CLIP_SEC}`,
  '-c:v', 'libx264', '-preset', 'ultrafast', '-pix_fmt', 'yuv420p',
  '-c:a', 'aac', '-shortest', clip,
]);
assert.equal(gen.status, 0, `ffmpeg could not build the fixture: ${gen.stderr}`);

const mpvHome = join(dir, 'mpv-home');
await mkdir(mpvHome, { recursive: true });
await writeFile(join(mpvHome, 'mpv.conf'), 'vo=null\nao=null\nreally-quiet=yes\n');
process.env.MPV_HOME = mpvHome;

const { playInMpv } = await import('../src/main/services/mpvPlayback.ts');

// --- a full watch ---------------------------------------------------------

const report = await playInMpv(clip, {});
assert.equal(report.tracked, true, 'the IPC socket answered — the session was observed');
assert.ok(
  report.duration > CLIP_SEC - 0.5 && report.duration < CLIP_SEC + 0.5,
  `duration should be about ${CLIP_SEC}s, got ${report.duration}`,
);
// The last poll lands up to POLL_INTERVAL_MS before mpv exits, so the final
// position trails the true end slightly. What matters is that it's inside the
// tail window the resume logic treats as "finished".
assert.ok(
  report.position > CLIP_SEC - 2,
  `final position should be near the end, got ${report.position}`,
);
assert.ok(report.watchedSec > 3, `should have accumulated watch time, got ${report.watchedSec}`);

// --- resuming -------------------------------------------------------------
// --start has to be honoured, or a resumed episode would replay from zero and
// (worse) accumulate a full watch time on a rewatch of a few seconds.

const resumed = await playInMpv(clip, { startSec: 4 });
assert.equal(resumed.tracked, true, 'the resumed session was observed too');
assert.ok(
  resumed.watchedSec < report.watchedSec,
  `a resumed session watches less than a full one (${resumed.watchedSec} vs ${report.watchedSec})`,
);

await rm(dir, { recursive: true, force: true });
console.log('verify-mpv-ipc: all assertions passed');
