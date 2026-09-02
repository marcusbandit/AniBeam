// End-to-end check of frame stepping against a real Chromium: build a clip
// with its frame number burned into the picture, drive the shipped
// FrameStepper through a headless Chrome exactly the way useFrameStep wires
// it, and read the picture back after every step.
//
// This is the only thing that can prove the assumptions shared/frameStep.ts
// rests on. verify-frame-step.mjs pins the logic against a fake video; what it
// cannot know is what Chromium really does: that requestVideoFrameCallback
// fires after a seek while paused, that seeking into the middle of a frame's
// interval paints exactly that frame, and that with those two facts stepping
// is exact forward, backward, across keyframes, under held-key repeat, after
// scrubs, and after pausing out of playback (where the frame on screen may
// not have reported itself yet, which is why the hook defers the first
// request by one rendering step).
//
// Needs ffmpeg and a Chrome or Chromium on PATH (ANIBEAM_CHROME overrides the
// lookup) and takes about twenty seconds of real time; it skips cleanly if
// either is missing. Everything happens in a temp dir that is removed at the
// end, and the shared module is bundled for the browser at run time so what
// is tested is exactly the source that ships.
//
// Run: bun --bun scripts/verify-frame-step-chromium.mjs

import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';

const NAME = 'verify-frame-step-chromium';
const REPO = path.resolve(import.meta.dirname, '..');
const MODULE_SRC = path.join(REPO, 'src', 'shared', 'frameStep.ts');
const D24 = 1001 / 24000;
const WALL_CLOCK_MS = 90_000;
const EVAL_TIMEOUT_MS = 25_000;

// --- tools ----------------------------------------------------------------

function findChrome() {
  const override = process.env.ANIBEAM_CHROME;
  if (override) return Bun.which(override) ?? (Bun.file(override).size > 0 ? override : null);
  for (const bin of ['google-chrome-stable', 'google-chrome', 'chromium', 'chromium-browser']) {
    const found = Bun.which(bin);
    if (found) return found;
  }
  return null;
}

const ffmpeg = Bun.which('ffmpeg');
const chromeBin = findChrome();
if (!ffmpeg || !chromeBin) {
  const missing = [!ffmpeg && 'ffmpeg', !chromeBin && 'a Chrome/Chromium binary'].filter(Boolean).join(' and ');
  console.log(`${NAME}: skipped (needs ${missing} on PATH)`);
  process.exit(0);
}

// --- fixtures -------------------------------------------------------------

const dir = mkdtempSync(path.join(tmpdir(), 'anibeam-frame-step-'));
let chrome = null;
let server = null;
let ws = null;

function cleanup() {
  try { ws?.close(); } catch { /* already closed */ }
  try { chrome?.kill('SIGKILL'); } catch { /* already gone */ }
  try { server?.stop(true); } catch { /* not started */ }
  rmSync(dir, { recursive: true, force: true });
}
process.on('exit', cleanup);
for (const sig of ['SIGINT', 'SIGTERM']) process.on(sig, () => process.exit(1));

const watchdog = setTimeout(() => {
  console.error(`${NAME}: gave up after ${WALL_CLOCK_MS / 1000}s of wall clock`);
  process.exit(1);
}, WALL_CLOCK_MS);

// The clip: 240 frames at 23.976 in 48-frame GOPs. The top half of every
// frame carries its own frame number as eight vertical bands (bit b in band
// b, white = set), which the page reads back through a canvas. The bottom
// half has the number drawn as text for anyone debugging with a screenshot;
// it is decoration, so the clip is rebuilt without it if this ffmpeg has no
// freetype.
const clip = path.join(dir, 'clip.mp4');
const strip = "format=yuv420p,geq=lum='if(lt(Y,H/2), if(gt(bitand(N,pow(2,floor(X*8/W))),0),235,16), 128)':cb=128:cr=128";
const text = ",drawtext=text='%{n}':fontsize=64:x=20:y=h-84:fontcolor=white";
function buildClip(filter) {
  return spawnSync(ffmpeg, [
    '-hide_banner', '-loglevel', 'error', '-y',
    '-f', 'lavfi', '-i', 'color=c=black:s=320x180:r=24000/1001:d=12',
    '-vf', filter, '-frames:v', '240',
    '-c:v', 'libx264', '-g', '48', '-pix_fmt', 'yuv420p', '-crf', '18',
    '-movflags', '+faststart', clip,
  ]);
}
let gen = buildClip(strip + text);
if (gen.status !== 0) gen = buildClip(strip);
assert.equal(gen.status, 0, `ffmpeg could not build the fixture: ${gen.stderr}`);

const build = await Bun.build({ entrypoints: [MODULE_SRC], target: 'browser', format: 'esm' });
assert.ok(build.success, `bundling frameStep.ts failed: ${build.logs.map(String).join('\n')}`);
writeFileSync(path.join(dir, 'frameStep.js'), await build.outputs[0].text());

writeFileSync(path.join(dir, 'index.html'), `<!doctype html>
<meta charset="utf-8">
<title>frame step probe</title>
<video id="v" src="/clip.mp4" muted playsinline preload="auto" style="width:320px;height:180px;display:block"></video>
<canvas id="c" width="320" height="180"></canvas>
`);

// --- serving --------------------------------------------------------------
// Chromium needs byte ranges to seek; without them it silently reloads the
// resource from zero on the first seek and every result below is garbage.

const TYPES = { '.mp4': 'video/mp4', '.js': 'text/javascript', '.html': 'text/html' };
server = Bun.serve({
  port: 0,
  async fetch(req) {
    const url = new URL(req.url);
    const file = url.pathname === '/' ? 'index.html' : url.pathname.slice(1);
    const f = Bun.file(path.join(dir, path.basename(file)));
    if (!(await f.exists())) return new Response('', { status: 404 });
    const type = TYPES[path.extname(file)] ?? 'application/octet-stream';
    const size = f.size;
    const range = req.headers.get('range');
    if (range) {
      const m = /bytes=(\d*)-(\d*)/.exec(range);
      const start = m?.[1] ? Number(m[1]) : 0;
      const end = m?.[2] ? Math.min(Number(m[2]), size - 1) : size - 1;
      return new Response(f.slice(start, end + 1), {
        status: 206,
        headers: {
          'Content-Type': type,
          'Accept-Ranges': 'bytes',
          'Content-Range': `bytes ${start}-${end}/${size}`,
          'Content-Length': String(end - start + 1),
        },
      });
    }
    return new Response(f, { headers: { 'Content-Type': type, 'Accept-Ranges': 'bytes', 'Content-Length': String(size) } });
  },
});
const pageUrl = `http://127.0.0.1:${server.port}/`;

// --- chrome ---------------------------------------------------------------

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function launch(cdpPort) {
  const proc = spawn(chromeBin, [
    '--headless=new',
    `--remote-debugging-port=${cdpPort}`,
    `--user-data-dir=${path.join(dir, 'profile')}`,
    '--no-first-run',
    '--no-default-browser-check',
    '--autoplay-policy=no-user-gesture-required',
    '--window-size=800,600',
    'about:blank',
  ], { stdio: ['ignore', 'ignore', 'pipe'] });
  let stderr = '';
  proc.stderr.on('data', (d) => { stderr += d; });
  for (let i = 0; i < 50; i++) {
    if (proc.exitCode != null) break;
    try {
      const list = await (await fetch(`http://127.0.0.1:${cdpPort}/json/list`)).json();
      const target = list.find((t) => t.type === 'page');
      if (target) return { proc, target };
    } catch { /* not up yet */ }
    await sleep(200);
  }
  proc.kill('SIGKILL');
  return { proc: null, target: null, stderr };
}

let target = null;
let lastStderr = '';
for (let attempt = 0; attempt < 2 && !target; attempt++) {
  const cdpPort = 20_000 + Math.floor(Math.random() * 20_000);
  const r = await launch(cdpPort);
  chrome = r.proc;
  target = r.target;
  lastStderr = r.stderr ?? '';
}
assert.ok(target, `Chrome never exposed a page target:\n${lastStderr.slice(-1500)}`);

ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((res, rej) => {
  ws.onopen = res;
  ws.onerror = () => rej(new Error('CDP websocket failed to open'));
});
let msgId = 0;
const pending = new Map();
ws.onmessage = (ev) => {
  const msg = JSON.parse(ev.data);
  if (!msg.id || !pending.has(msg.id)) return;
  const { res, rej } = pending.get(msg.id);
  pending.delete(msg.id);
  if (msg.error) rej(new Error(JSON.stringify(msg.error)));
  else res(msg.result);
};
function send(method, params = {}) {
  return new Promise((res, rej) => {
    const id = ++msgId;
    pending.set(id, { res, rej });
    ws.send(JSON.stringify({ id, method, params }));
  });
}

// Runs `body` as an async function in the page and returns its value. A
// wedged Chrome fails the script instead of hanging it: the evaluate has its
// own timeout and the websocket round trip is raced against a second one.
async function evaluate(label, body) {
  const timer = new Promise((_, rej) => setTimeout(() => rej(new Error(`${label}: page did not answer in ${EVAL_TIMEOUT_MS}ms`)), EVAL_TIMEOUT_MS + 2000));
  const r = await Promise.race([
    send('Runtime.evaluate', {
      expression: `(async () => { ${body} })()`,
      awaitPromise: true,
      returnByValue: true,
      timeout: EVAL_TIMEOUT_MS,
    }),
    timer,
  ]);
  if (r.exceptionDetails) {
    const desc = r.exceptionDetails.exception?.description ?? r.exceptionDetails.text;
    throw new Error(`${label}: page threw: ${desc}`);
  }
  return r.result.value;
}

await send('Page.enable');
await send('Page.navigate', { url: pageUrl });
await sleep(300);

// --- the page side --------------------------------------------------------
// One setup evaluate builds the stepper the way src/renderer/hooks/useFrameStep.ts
// does and parks everything on window.__fs; each scenario is its own
// evaluate so a failure names the scenario and the earlier ones still print.

await evaluate('setup', `
  const v = document.getElementById('v');
  const ctx = document.getElementById('c').getContext('2d', { willReadFrequently: true });
  // Frame number from the eight bands in the top half: sample the middle of
  // each band a quarter of the way down.
  const readStrip = () => {
    ctx.drawImage(v, 0, 0, 320, 180);
    const d = ctx.getImageData(0, 45, 320, 1).data;
    let n = 0;
    for (let b = 0; b < 8; b++) if (d[Math.floor((b + 0.5) * 40) * 4] > 128) n |= 1 << b;
    return n;
  };
  await new Promise((res, rej) => {
    if (v.readyState >= 1) return res();
    v.addEventListener('loadedmetadata', () => res(), { once: true });
    v.addEventListener('error', () => rej(new Error('video error ' + (v.error && v.error.code))), { once: true });
  });
  const mod = await import('/frameStep.js');
  const landings = [];
  const seeks = [];
  let landedResolve = null;
  const stepper = new mod.FrameStepper({
    seek: (t) => { seeks.push(t); v.currentTime = t; },
    getCurrentTime: () => v.currentTime,
    getDuration: () => v.duration,
    isSeeking: () => v.seeking,
    schedule: (fn, ms) => { const id = setTimeout(fn, ms); return () => clearTimeout(id); },
    onLanded: (pts, d) => {
      landings.push({ pts, d, strip: readStrip() });
      if (landedResolve) { const r = landedResolve; landedResolve = null; r(); }
    },
  });
  v.addEventListener('seeking', () => stepper.onSeeking());
  v.addEventListener('play', () => stepper.cancelPending());
  const loop = (_now, m) => { stepper.onFramePresented(m.mediaTime); v.requestVideoFrameCallback(loop); };
  v.requestVideoFrameCallback(loop);
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const waitLanded = (ms = 3000) => new Promise((res, rej) => {
    const t = setTimeout(() => rej(new Error('no landing within ' + ms + 'ms')), ms);
    landedResolve = () => { clearTimeout(t); res(); };
  });
  // The hook's step(): pause first, and when it had to pause, defer the
  // request by one rendering step so the frame on screen has reported itself.
  const step = (dir) => {
    if (!v.paused) { v.pause(); requestAnimationFrame(() => stepper.request(dir)); return; }
    stepper.request(dir);
  };
  window.__fs = { v, mod, stepper, landings, seeks, readStrip, sleep, waitLanded, step };
`);

// Learn the frame rate from real playback, then park about three seconds in
// (frame 72) so the long backward run never reaches frame zero.
const start = await evaluate('start', `
  const { v, stepper, readStrip, sleep } = window.__fs;
  await v.play(); await sleep(3000); v.pause(); await sleep(100);
  return { frameDuration: stepper.frameDuration, strip: readStrip(), anchorIdx: Math.round(stepper.lastPts / stepper.frameDuration), maxPending: window.__fs.mod.MAX_PENDING_STEPS };
`);

function pass(n, what) { console.log(`${NAME}: ${n}. ${what}`); }

// 1. The estimator learned 23.976 from presented-frame timestamps.
assert.ok(
  Math.abs(start.frameDuration - D24) < 1e-6,
  `frame duration should be 1001/24000 (${D24}) after a second of playback, got ${start.frameDuration}`,
);
assert.equal(start.anchorIdx, start.strip, 'the paused picture is the frame the last callback reported');
assert.ok(start.strip >= 60, `expected to be parked around frame 72, got frame ${start.strip}`);
pass(1, `estimator learned ${start.frameDuration.toFixed(6)}s per frame from playback`);

// 2. Thirty forward steps, one landing each.
const fwd = await evaluate('forward', `
  const { readStrip, waitLanded, step } = window.__fs;
  const pairs = [];
  for (let i = 0; i < 30; i++) { const before = readStrip(); const p = waitLanded(); step(1); await p; pairs.push([before, readStrip()]); }
  return pairs;
`);
for (const [before, after] of fwd) assert.equal(after, before + 1, `forward step from frame ${before} should show ${before + 1}, showed ${after}`);
pass(2, `30 forward steps each advanced exactly one frame (${fwd[0][0]} to ${fwd.at(-1)[1]})`);

// 3. Sixty backward steps; the GOPs are 48 frames so this crosses keyframes.
const back = await evaluate('backward', `
  const { readStrip, waitLanded, step } = window.__fs;
  const pairs = [];
  for (let i = 0; i < 60; i++) { const before = readStrip(); const p = waitLanded(); step(-1); await p; pairs.push([before, readStrip()]); }
  return pairs;
`);
for (const [before, after] of back) assert.equal(after, before - 1, `backward step from frame ${before} should show ${before - 1}, showed ${after}`);
pass(3, `60 backward steps each went back exactly one frame (${back[0][0]} to ${back.at(-1)[1]}, across keyframes)`);

// 4. Twenty presses in one task: a held key at machine speed coalesces.
const burst = await evaluate('burst', `
  const { landings, readStrip, sleep, step } = window.__fs;
  const before = readStrip(); const n0 = landings.length;
  for (let i = 0; i < 20; i++) step(1);
  await sleep(1200);
  return { before, after: readStrip(), landings: landings.slice(n0).map((l) => l.strip) };
`);
assert.ok(
  burst.after - burst.before <= start.maxPending + 1,
  `20 presses in one task should move at most ${start.maxPending + 1} frames, moved ${burst.after - burst.before}`,
);
assert.equal(burst.landings.length, burst.after - burst.before, 'every frame moved was a reported landing');
burst.landings.forEach((s, i) => assert.equal(s, (i === 0 ? burst.before : burst.landings[i - 1]) + 1, `burst landing ${i} should be +1, got ${s}`));
pass(4, `20 presses in one task moved ${burst.after - burst.before} frames (cap ${start.maxPending + 1}), every landing +1`);

// 5. Key repeat at 30 Hz for a second: every landing is exactly the next frame.
const repeat = await evaluate('repeat', `
  const { landings, readStrip, sleep, step } = window.__fs;
  const before = readStrip(); const n0 = landings.length;
  const t0 = performance.now();
  while (performance.now() - t0 < 1000) { step(1); await sleep(33); }
  await sleep(800);
  return { before, after: readStrip(), landings: landings.slice(n0).map((l) => l.strip) };
`);
assert.ok(repeat.landings.length >= 20, `a second of 30 Hz repeat should land at least 20 frames, landed ${repeat.landings.length}`);
repeat.landings.forEach((s, i) => assert.equal(s, (i === 0 ? repeat.before : repeat.landings[i - 1]) + 1, `repeat landing ${i} should be +1, got ${s}`));
assert.equal(repeat.after, repeat.landings.at(-1), 'the picture ends on the last landing');
pass(5, `held key at 30 Hz landed ${repeat.landings.length} frames, each exactly +1`);

// 6. What the HUD would show is the frame in the picture, every time.
const hud = await evaluate('hud', `
  const { landings } = window.__fs;
  return landings.map((l) => [Math.round(l.pts / l.d), l.strip]).filter(([idx, strip]) => idx !== strip);
`);
assert.deepEqual(hud, [], `every landing's pts / frameDuration should equal the burned-in frame number; mismatches (idx, picture): ${JSON.stringify(hud)}`);
pass(6, 'every landing timestamp names the frame in the picture');

// 7. Scrub while paused, let it paint, step back one.
const scrub = await evaluate('scrub-then-step', `
  const { v, readStrip, sleep, waitLanded, step } = window.__fs;
  v.currentTime = 3.0; await sleep(300);
  const before = readStrip(); const p = waitLanded(); step(-1); await p;
  return { before, after: readStrip() };
`);
assert.equal(scrub.after, scrub.before - 1, `after a scrub to 3.0s (frame ${scrub.before}) a back step should show ${scrub.before - 1}, showed ${scrub.after}`);
pass(7, `scrub to 3.0s then step back landed on frame ${scrub.after}`);

// 8. Scrub and press in the same task, before the seeking event has fired:
//    the element already reports seeking, so the step waits for the scrub's
//    frame and lands one past it, not one past the frame being replaced.
const sameTask = await evaluate('scrub-same-task', `
  const { v, landings, readStrip, sleep, step, stepper } = window.__fs;
  const n0 = landings.length;
  v.currentTime = 6.0; step(1);
  await sleep(500);
  return { landings: landings.slice(n0).map((l) => l.strip), after: readStrip(), expected: Math.floor(6.0 / stepper.frameDuration) + 1 };
`);
assert.equal(sameTask.landings.at(-1), sameTask.expected, `scrub to 6.0s plus a press in the same task should end on frame ${sameTask.expected}, landings were ${JSON.stringify(sameTask.landings)}`);
assert.equal(sameTask.after, sameTask.expected, 'the picture shows that frame');
pass(8, `scrub and press in one task ended on frame ${sameTask.after} (landings ${sameTask.landings.join(', ')})`);

// 9. Pause out of playback and step at once, both directions. Without the
//    rendering-step deferral this fails about a quarter of the time.
const trials = await evaluate('pause-then-step', `
  const { v, readStrip, sleep, step } = window.__fs;
  const out = [];
  for (const dir of [1, -1]) {
    for (let i = 0; i < 8; i++) {
      v.currentTime = 1 + Math.random() * 6; await sleep(150);
      await v.play(); await sleep(200 + Math.random() * 200);
      step(dir);
      const before = readStrip();
      await sleep(300);
      out.push({ dir, before, after: readStrip(), paused: v.paused });
    }
  }
  return out;
`);
for (const t of trials) {
  assert.ok(t.paused, 'a step pauses the video');
  assert.equal(t.after, t.before + t.dir, `pause then step ${t.dir > 0 ? 'forward' : 'back'} from frame ${t.before} should show ${t.before + t.dir}, showed ${t.after}`);
}
pass(9, `${trials.length} pause-then-step trials each moved exactly one frame`);

// 10. Presses still queued when playback resumes are dropped, not fired into
//     the playing video.
const resume = await evaluate('play-cancels', `
  const { v, seeks, sleep, step } = window.__fs;
  v.pause(); await sleep(50);
  for (let i = 0; i < 4; i++) step(1);
  const seeksBefore = seeks.length;
  await v.play();
  const t0 = v.currentTime;
  await sleep(600);
  const advanced = v.currentTime - t0;
  v.pause();
  return { advanced, seeksDuringPlay: seeks.length - seeksBefore };
`);
assert.equal(resume.seeksDuringPlay, 0, `no seek should fire once playback resumes, ${resume.seeksDuringPlay} did`);
assert.ok(
  resume.advanced > 0.5 && resume.advanced < 0.75,
  `600ms of playback should advance between 0.5 and 0.75s of media time, advanced ${resume.advanced.toFixed(3)}s`,
);
pass(10, `queued presses were dropped on play; playback advanced ${resume.advanced.toFixed(3)}s in 600ms with no seeks`);

// 11. Frame zero is a wall.
const zero = await evaluate('at-zero', `
  const { v, seeks, readStrip, sleep, step } = window.__fs;
  v.currentTime = 0; await sleep(300);
  const seeksBefore = seeks.length;
  step(-1); await sleep(300);
  return { strip: readStrip(), currentTime: v.currentTime, seeks: seeks.length - seeksBefore };
`);
assert.equal(zero.strip, 0, `a back step at frame 0 should still show frame 0, showed ${zero.strip}`);
assert.equal(zero.currentTime, 0, `currentTime should stay at 0, got ${zero.currentTime}`);
assert.equal(zero.seeks, 0, 'and no seek is issued for it');
pass(11, 'a backward press at frame 0 does nothing');

clearTimeout(watchdog);
console.log(`${NAME}: all assertions passed`);
process.exit(0);
