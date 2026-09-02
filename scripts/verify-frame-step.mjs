import assert from 'node:assert/strict';

const {
  FALLBACK_FRAME_DURATION,
  MAX_PENDING_STEPS,
  PAINT_TIMEOUT_MS,
  FrameDurationEstimator,
  FrameStepper,
  stepTarget,
  estimatedFrameIndex,
  formatTimeMs,
} = await import('../src/shared/frameStep.ts');

// Frame stepping is a seek by one learned frame duration, anchored on the
// frame that is really on screen. These blocks pin the estimator (what a
// frame duration is), the target maths (where a step lands), and the stepper
// (how presses are serialised against the video's seek/paint cycle).

const D24 = 1001 / 24000; // 23.976 fps
const D60 = 1 / 60;
const close = (a, b, eps = 1e-9) => assert.ok(Math.abs(a - b) < eps, `${a} vs ${b}`);

// --- estimator: learns the frame duration from presented-frame timestamps ---
{
  const est = new FrameDurationEstimator();
  // Nothing seen yet: assume 23.976, which is what nearly all anime is.
  assert.equal(est.observed, false);
  assert.equal(est.frameDuration, FALLBACK_FRAME_DURATION);

  // Clean 23.976 playback lands on exactly 1001/24000.
  for (let i = 0; i < 10; i++) est.observe(i * D24);
  assert.equal(est.observed, true);
  close(est.frameDuration, D24);

  // A dropped presentation shows up as a 2x delta. It must not raise the
  // estimate: the minimum is the frame duration.
  est.observe(12 * D24);
  close(est.frameDuration, D24);

  // A 5 s seek and a backwards jump are not frame durations.
  est.observe(12 * D24 + 5);
  est.observe(12 * D24 + 5 - 3);
  close(est.frameDuration, D24);

  // reset forgets everything, including the previous timestamp, so the first
  // observation after it produces no delta.
  est.reset();
  assert.equal(est.observed, false);
  assert.equal(est.frameDuration, FALLBACK_FRAME_DURATION);
  est.observe(7);
  assert.equal(est.observed, false);
}
{
  // MKV timestamps are millisecond-quantised, so 23.976 arrives as 42, 41,
  // 42, 42 ms deltas. The estimate is the smallest one (41 ms); the 0.5-frame
  // landing margin in stepTarget absorbs the 2% error.
  const est = new FrameDurationEstimator();
  for (const t of [0, 0.042, 0.083, 0.125, 0.167]) est.observe(t);
  close(est.frameDuration, 0.041);
}
{
  // The window is bounded: after 64 fresh deltas the older ones are gone.
  // 60 fps playback, then a switch to 24 fps for 65 frames: the 1/60 deltas
  // must have fallen out, so the minimum is now 1/24.
  const est = new FrameDurationEstimator();
  let t = 0;
  for (let i = 0; i < 10; i++) { est.observe(t); t += D60; }
  close(est.frameDuration, D60);
  for (let i = 0; i < 65; i++) { t += 1 / 24; est.observe(t); }
  close(est.frameDuration, 1 / 24);
  // 64 of them would still keep one 1/60 delta alive.
  const est2 = new FrameDurationEstimator();
  t = 0;
  for (let i = 0; i < 2; i++) { est2.observe(t); t += D60; }
  for (let i = 0; i < 63; i++) { t += 1 / 24; est2.observe(t); }
  close(est2.frameDuration, D60);
}

// --- stepTarget: land mid-frame so a slightly wrong estimate still hits ---
{
  const d = D24;
  close(stepTarget(10, 1, d, 100), 10 + 1.5 * d);
  close(stepTarget(10, -1, d, 100), 10 - 0.5 * d);
  // Clamped at zero and at the end.
  assert.equal(stepTarget(0, -1, d, 100), 0);
  assert.equal(stepTarget(0.01, -1, d, 100), 0);
  assert.equal(stepTarget(100, 1, d, 100), 100);
  assert.equal(stepTarget(99.99, 1, d, 100), 100);
  // Unknown duration: no upper clamp.
  close(stepTarget(99.99, 1, d, NaN), 99.99 + 1.5 * d);
  close(stepTarget(99.99, 1, d, 0), 99.99 + 1.5 * d);
  close(stepTarget(99.99, 1, d, Infinity), 99.99 + 1.5 * d);
}

// --- frame index and the HUD timestamp ---
{
  assert.equal(estimatedFrameIndex(0, D24), 0);
  assert.equal(estimatedFrameIndex(10 * D24, D24), 10);
  assert.equal(estimatedFrameIndex(10.4 * D24, D24), 10);
  assert.equal(estimatedFrameIndex(10.6 * D24, D24), 11);
  assert.equal(estimatedFrameIndex(-1, D24), 0);
  assert.equal(estimatedFrameIndex(NaN, D24), 0);
  assert.equal(estimatedFrameIndex(5, 0), 0);

  assert.equal(formatTimeMs(0), '0:00.000');
  assert.equal(formatTimeMs(65.5), '1:05.500');
  assert.equal(formatTimeMs(754.583), '12:34.583');
  assert.equal(formatTimeMs(3600), '1:00:00.000');
  assert.equal(formatTimeMs(3661.0075), '1:01:01.008');
  assert.equal(formatTimeMs(-3), '0:00.000');
  assert.equal(formatTimeMs(NaN), '0:00.000');
  assert.equal(formatTimeMs(Infinity), '0:00.000');
  // Rounding never spills a "1000" into the millisecond field.
  assert.equal(formatTimeMs(1.9996), '0:02.000');
}

// --- stepper: a fake <video> that records seeks and runs timers by hand ---
function fakeIo({ currentTime = 0, duration = 100 } = {}) {
  const io = {
    seeks: [],
    landed: [],
    timers: [],
    currentTime,
    duration,
    seek(t) { io.seeks.push(t); },
    getCurrentTime() { return io.currentTime; },
    getDuration() { return io.duration; },
    schedule(fn, ms) {
      const timer = { fn, ms, cancelled: false };
      io.timers.push(timer);
      return () => { timer.cancelled = true; };
    },
    onLanded(pts, d) { io.landed.push({ pts, d }); },
    // Fire every live timer, in order, as if the wall clock advanced.
    fireTimers() {
      const live = io.timers.filter((t) => !t.cancelled);
      io.timers = [];
      for (const t of live) t.fn();
    },
    liveTimers() { return io.timers.filter((t) => !t.cancelled).length; },
  };
  return io;
}

// Playback at 23.976 that paused on frame 100; the stepper has seen it.
function primed() {
  const io = fakeIo();
  const est = new FrameDurationEstimator();
  const stepper = new FrameStepper(io, est);
  for (let i = 90; i <= 100; i++) stepper.onFramePresented(i * D24);
  assert.equal(io.landed.length, 0, 'plain playback never fires onLanded');
  close(stepper.lastPts, 100 * D24);
  close(stepper.frameDuration, D24);
  return { io, stepper };
}

// a. One press seeks to pts + 1.5d. A second press while that seek is in
//    flight queues; when the frame lands, onLanded fires and the queued step
//    is anchored on the landed frame, so two presses move two frames, not
//    three.
{
  const { io, stepper } = primed();
  stepper.request(1);
  assert.equal(io.seeks.length, 1);
  close(io.seeks[0], 100 * D24 + 1.5 * D24);
  stepper.request(1);
  assert.equal(io.seeks.length, 1, 'busy: second press is queued, not seeked');
  // The paint arrives: Chromium shows frame 101.
  stepper.onFramePresented(101 * D24);
  assert.equal(io.landed.length, 1);
  close(io.landed[0].pts, 101 * D24);
  assert.equal(io.seeks.length, 2);
  close(io.seeks[1], 101 * D24 + 1.5 * D24);
  stepper.onFramePresented(102 * D24);
  assert.equal(io.landed.length, 2);
  close(io.landed[1].pts, 102 * D24);
  assert.equal(io.seeks.length, 2, 'nothing queued: no further seek');
  // Session over: a frame presented now (say, from a scrub) is not "landed".
  stepper.onFramePresented(500 * D24);
  assert.equal(io.landed.length, 2);
}

// b. Holding the key: presses coalesce to at most MAX_PENDING_STEPS queued.
{
  const { io, stepper } = primed();
  for (let i = 0; i < 20; i++) stepper.request(1);
  assert.equal(io.seeks.length, 1);
  // Drain everything that queued; each landing releases exactly one seek.
  let frame = 100;
  let seeks = 1;
  for (;;) {
    frame += 1;
    stepper.onFramePresented(frame * D24);
    if (io.seeks.length === seeks) break;
    seeks = io.seeks.length;
  }
  assert.equal(io.seeks.length, 1 + MAX_PENDING_STEPS);
}

// c. Forward then back while the forward seek is in flight: the in-flight
//    seek cannot be recalled, so the back press queues and, once frame 101
//    lands, steps back to frame 100. Net movement zero, ending where the
//    user started. Opposite presses that are both still queued do cancel.
{
  const { io, stepper } = primed();
  stepper.request(1);
  stepper.request(-1);
  stepper.onFramePresented(101 * D24);
  assert.equal(io.seeks.length, 2);
  close(io.seeks[1], 101 * D24 - 0.5 * D24);
  stepper.onFramePresented(100 * D24);
  assert.equal(io.seeks.length, 2);
  assert.equal(io.liveTimers(), 0, 'nothing in flight, no timer armed');

  // Two queued forward presses and two queued back presses while busy: the
  // queue nets to zero and the landing releases nothing.
  const q = primed();
  q.stepper.request(1);
  q.stepper.request(1);
  q.stepper.request(1);
  q.stepper.request(-1);
  q.stepper.request(-1);
  q.stepper.onFramePresented(101 * D24);
  assert.equal(q.io.seeks.length, 1);
  assert.equal(q.io.liveTimers(), 0);
}

// d. Before any frame has been presented the anchor is currentTime.
{
  const io = fakeIo({ currentTime: 42 });
  const stepper = new FrameStepper(io);
  assert.equal(stepper.lastPts, null);
  stepper.request(1);
  close(io.seeks[0], 42 + 1.5 * FALLBACK_FRAME_DURATION);
  const io2 = fakeIo({ currentTime: 42 });
  new FrameStepper(io2).request(-1);
  close(io2.seeks[0], 42 - 0.5 * FALLBACK_FRAME_DURATION);
}

// e. Nothing before frame zero, nothing past the end: the step is dropped.
{
  const io = fakeIo({ currentTime: 0 });
  const stepper = new FrameStepper(io);
  stepper.request(-1);
  assert.equal(io.seeks.length, 0);
  assert.equal(io.liveTimers(), 0);
  // And it is not left in the queue to fire later.
  stepper.onFramePresented(0);
  assert.equal(io.seeks.length, 0);

  const io2 = fakeIo({ currentTime: 100, duration: 100 });
  const stepper2 = new FrameStepper(io2);
  stepper2.onFramePresented(100);
  stepper2.request(1);
  assert.equal(io2.seeks.length, 0);

  // Presses that queued behind a seek onto the last frame are dropped with
  // it, not left to fire into whatever frame is presented next (a scrub, or
  // playback resuming).
  const io3 = fakeIo({ duration: 100 });
  const stepper3 = new FrameStepper(io3);
  stepper3.onFramePresented(100 - D24);
  stepper3.request(1);
  assert.equal(io3.seeks.length, 1);
  stepper3.request(1);
  stepper3.request(1);
  stepper3.onFramePresented(100);
  assert.equal(io3.landed.length, 1);
  assert.equal(io3.seeks.length, 1, 'clamped: no further seek');
  stepper3.onFramePresented(50);
  assert.equal(io3.seeks.length, 1);
  assert.equal(io3.landed.length, 1, 'session over: a later frame is not a landing');
}

// e2. A press in the same task as a scrub, before the seeking event has
//     fired: the element already reports seeking, so the step waits for the
//     scrub's frame instead of anchoring on the one it is replacing.
{
  const { io, stepper } = primed();
  io.isSeeking = () => true;
  stepper.request(1);
  assert.equal(io.seeks.length, 0, 'deferred behind the in-flight scrub');
  assert.equal(io.liveTimers(), 1, 'and guarded by the paint timeout');
  io.isSeeking = () => false;
  stepper.onFramePresented(300 * D24);
  assert.equal(io.seeks.length, 1);
  close(io.seeks[0], 300 * D24 + 1.5 * D24);
}

// e3. Playback resumed with presses still queued: cancelPending drops them
//     but keeps what was learned, so the next paused step is still exact.
{
  const { io, stepper } = primed();
  stepper.request(1);
  stepper.request(1);
  stepper.request(1);
  assert.equal(io.seeks.length, 1);
  stepper.cancelPending();
  assert.equal(io.liveTimers(), 0);
  stepper.onFramePresented(101 * D24);
  stepper.onFramePresented(102 * D24);
  assert.equal(io.seeks.length, 1, 'nothing queued fires into playback');
  assert.equal(io.landed.length, 0);
  close(stepper.frameDuration, D24);
  stepper.request(1);
  close(io.seeks[1], 102 * D24 + 1.5 * D24);
}

// f. The paint never comes (no requestVideoFrameCallback): the timeout
//    assumes the frame we asked for is showing and the next step anchors on
//    it, so stepping keeps working without self-correction.
{
  const { io, stepper } = primed();
  stepper.request(1);
  const target = io.seeks[0];
  assert.equal(io.liveTimers(), 1);
  assert.equal(io.timers[0].ms, PAINT_TIMEOUT_MS);
  io.fireTimers();
  close(stepper.lastPts, target - 0.5 * D24);
  close(stepper.lastPts, 101 * D24);
  stepper.request(1);
  assert.equal(io.seeks.length, 2);
  close(io.seeks[1], 101 * D24 + 1.5 * D24);
}

// g. An external scrub (seeking event) while a press is pending: the step
//    waits for that scrub's frame, then anchors on it.
{
  const { io, stepper } = primed();
  stepper.onSeeking();
  stepper.request(1);
  assert.equal(io.seeks.length, 0, 'deferred behind the scrub');
  stepper.onFramePresented(300 * D24);
  assert.equal(io.seeks.length, 1);
  close(io.seeks[0], 300 * D24 + 1.5 * D24);
  // A scrub whose paint never arrives is released by the timeout too.
  const p = primed();
  p.stepper.onSeeking();
  p.stepper.request(-1);
  assert.equal(p.io.seeks.length, 0);
  p.io.fireTimers();
  assert.equal(p.io.seeks.length, 1);
  close(p.io.seeks[0], 100 * D24 - 0.5 * D24);
}

// h. reset (new source) clears everything, including the timer.
{
  const { io, stepper } = primed();
  stepper.request(1);
  stepper.request(1);
  assert.equal(io.liveTimers(), 1);
  stepper.reset();
  assert.equal(stepper.lastPts, null);
  assert.equal(stepper.frameDuration, FALLBACK_FRAME_DURATION);
  assert.equal(io.liveTimers(), 0);
  // Nothing left queued: a landing frame releases no seek.
  stepper.onFramePresented(5);
  assert.equal(io.seeks.length, 1);
  assert.equal(io.landed.length, 0);
}

console.log('verify-frame-step: all assertions passed');
