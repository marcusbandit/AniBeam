// Verifies the stop/resume bookkeeping in transcodeCacheHandler without
// running ffmpeg: we drive the queue directly and assert what cancel(),
// cancelAll(), setAuto() and the enqueue() reason gate do to it.
//
// The handler pulls in electron (app.getPath), configHandler and
// metadataHandler, so all three are stubbed through module mocks before the
// import. probeCodecs is stubbed too, which is what keeps ffmpeg out of it:
// every file reports "needs transcoding", so runOne reaches its pre-spawn
// cancel gate and settles there.
//
// The queued files must really exist on disk — runOne's first check is an
// existsSync, and a missing file rejects before any of this is reached.
//
// Run: bun --bun scripts/verify-transcode-cancel.mjs

import assert from 'node:assert/strict';
import { mock } from 'bun:test';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';

const userData = await mkdtemp(join(tmpdir(), 'anibeam-cancel-'));
const library = await mkdtemp(join(tmpdir(), 'anibeam-lib-'));

// Stand-in for a library file. Contents are irrelevant — probeCodecs is mocked.
async function fixture(name) {
  const path = join(library, name);
  await writeFile(path, 'not really a video');
  return path;
}

mock.module('electron', () => ({
  app: { getPath: () => userData },
}));

// Keep the real config file out of it: the handler persists opt-outs on every
// stop, and this test must not touch the user's actual config.json.
let savedConfig = {};
mock.module('../src/main/handlers/configHandler', () => ({
  default: {
    loadConfig: async () => ({ ...savedConfig }),
    saveConfig: async (patch) => { savedConfig = { ...savedConfig, ...patch }; return true; },
  },
}));

mock.module('../src/main/services/logger', () => ({
  logger: { info() {}, warn() {}, error() {} },
}));

mock.module('../src/main/handlers/metadataHandler', () => ({
  default: {
    loadMetadata: async () => ({}),
    transaction: async (fn) => (await fn({})).result,
  },
}));

// Async on purpose: the await is the preparation window a stop has to be able
// to land in. Reports a codec that always needs converting so runOne walks all
// the way to its pre-spawn gate.
mock.module('../src/main/utils/transcodeProbe', () => ({
  probeCodecs: async () => ({ vCodec: 'hevc', aCodec: 'aac', duration: 1400, displayAspect: 16 / 9 }),
  needsTranscode: () => true,
  ensureEncoder: async () => 'libx264',
  ensureEncoderStatus: async () => ({ kind: 'libx264', reason: null }),
}));

const { default: handler } = await import('../src/main/handlers/transcodeCacheHandler.ts');

// --- the reason gate ------------------------------------------------------

handler.setAuto(false);
assert.equal(handler.autoState().auto, false, 'setAuto(false) turns the sweeps off');

const autoWhileOff = await fixture('auto-while-off.mkv');
await handler.enqueue(autoWhileOff);
assert.deepEqual(
  handler.queueSnapshot(),
  { activePath: null, queuedPaths: [] },
  'auto enqueue is a no-op while automatic re-encoding is off',
);

handler.setAuto(true);
assert.equal(handler.autoState().auto, true, 'setAuto(true) turns the sweeps back on');

// --- stopping a file that is mid-preparation ------------------------------
// enqueue() pumps synchronously up to the first await, so by the time it
// returns, the head of the queue has already been shifted into runOne and is
// sitting in neither `queue` nor `active`. Stopping it still has to work —
// that window contains an ffprobe and can last seconds.

const preparing = await fixture('preparing.mkv');
const preparingSettles = handler.enqueue(preparing);
assert.equal(
  handler.cancel(preparing),
  true,
  'cancel() stops a file that is dequeued but not yet spawned',
);
// A stop RESOLVES rather than rejects — every enqueue call site fire-and-forgets
// with `void`, so a rejection here would surface as an unhandled rejection.
await preparingSettles;
assert.equal(handler.isSkipped(preparing), true, 'the stopped file is skipped by later sweeps');

// Re-queueing it automatically must stay refused, or the sweep would undo the
// stop seconds after the user made it.
await handler.enqueue(preparing);
assert.equal(
  handler.queueSnapshot().queuedPaths.includes(preparing),
  false,
  'an automatic sweep does not re-queue a stopped file',
);

// ...but an explicit play does, and clears the opt-out.
const replayed = handler.enqueue(preparing, { reason: 'user' });
assert.equal(handler.isSkipped(preparing), false, 'playing the episode clears the stop');
handler.cancel(preparing);
await replayed;
handler.clearOptOut();

// --- stopping queued files ------------------------------------------------
// The first enqueued file goes straight into preparation; the rest queue up
// behind it. cancelAll() has to cover both.

const many = await Promise.all([fixture('a.mkv'), fixture('b.mkv'), fixture('c.mkv')]);
const settles = many.map((p) => handler.enqueue(p));
assert.equal(
  handler.queueSnapshot().queuedPaths.length,
  many.length - 1,
  'the head of the batch is in preparation, the rest are queued',
);

const cleared = handler.cancelAll();
assert.equal(cleared, many.length, 'cancelAll() stops the queued files AND the one preparing');
await Promise.all(settles);
assert.deepEqual(
  handler.queueSnapshot(),
  { activePath: null, queuedPaths: [] },
  'the queue is empty after cancelAll()',
);
for (const p of many) {
  assert.equal(handler.isSkipped(p), true, `${p} stays stopped after cancelAll()`);
}

// Re-enabling auto forgets the per-file stops — otherwise the toggle would read
// as "on" while every previously-stopped file stayed permanently skipped.
const res = handler.setAuto(true);
assert.equal(res.resumed >= many.length, true, 'turning auto back on resumes stopped files');
for (const p of many) {
  assert.equal(handler.isSkipped(p), false, `${p} is no longer skipped`);
}
assert.equal(handler.autoState().optedOutCount, 0, 'no stops remain');

// --- stopping something that isn't running -------------------------------

const idle = await fixture('never-queued.mkv');
assert.equal(handler.cancel(idle), false, 'cancel() reports false when there was nothing to stop');
assert.equal(handler.isSkipped(idle), false, 'a no-op cancel does not record an opt-out');

// --- "never re-encode" ----------------------------------------------------
// Stronger than a stop: a plain stop is lifted by playing the episode, whereas
// never survives it (playing offers mpv instead) and is only lifted by an
// explicit force.

const never = await fixture('never.mkv');
const neverSettles = handler.enqueue(never);
const marked = handler.setNever(never, true);
assert.equal(marked.stopped, true, 'marking never stops an encode that is already running');
await neverSettles;
assert.equal(handler.isNever(never), true, 'the file is marked never');
assert.equal(handler.isSkipped(never), true, 'automatic sweeps skip it');
assert.equal(handler.autoState().neverCount, 1, 'the mark is counted separately from plain stops');

// The decisive difference from a stop: a user-initiated play must NOT start an
// encode, because "never" exists so that playing offers mpv instead.
await handler.enqueue(never, { reason: 'user' });
assert.equal(
  handler.queueSnapshot().queuedPaths.includes(never),
  false,
  'playing a never-encode file does not start an encode',
);
assert.equal(handler.isNever(never), true, 'and does not clear the mark');

// Only an explicit "re-encode anyway" lifts it.
const forced = handler.enqueue(never, { reason: 'force' });
assert.equal(handler.isNever(never), false, 'force lifts the never mark');
handler.cancel(never);
await forced;
handler.clearOptOut();

// A never mark is a per-file decision, not a side effect of the global switch,
// so re-enabling auto must not quietly undo it.
handler.setNever(never, true);
const reEnabled = handler.setAuto(true);
assert.equal(
  handler.isNever(never),
  true,
  'turning automatic re-encoding back on does not clear a never mark',
);
assert.equal(reEnabled.auto, true);
handler.setNever(never, false);
assert.equal(handler.isNever(never), false, 'the mark can be cleared directly');
assert.equal(handler.autoState().neverCount, 0);

await rm(userData, { recursive: true, force: true });
await rm(library, { recursive: true, force: true });
console.log('verify-transcode-cancel: all assertions passed');
