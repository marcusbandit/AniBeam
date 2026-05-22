import assert from 'node:assert/strict';

const { smoothStep } = await import('../src/renderer/utils/motion.ts');

// --- Identity: dt=0 does not move ---
assert.equal(smoothStep(0, 100, 10, 0), 0);
assert.equal(smoothStep(50, 50, 10, 0.016), 50);

// --- Monotonic progression: each step moves toward the target ---
let cur = 0;
for (let i = 0; i < 10; i++) {
  const next = smoothStep(cur, 100, 10, 0.016);
  assert.ok(next > cur, `step ${i}: expected monotonic increase, cur=${cur} next=${next}`);
  assert.ok(next <= 100, `step ${i}: overshot, next=${next}`);
  cur = next;
}

// --- Snap-to-target when very close ---
assert.equal(smoothStep(99.99, 100, 10, 0.016), 100);
assert.equal(smoothStep(-99.99, -100, 10, 0.016), -100);

// --- Frame-rate independence: same total dt → same result regardless of
//     how many sub-steps. Total dt is 0.04s, inside the engine's 0.05s
//     clamp window so neither path is truncated. Real-world dts are ~0.016s,
//     so this test exercises the regime the engine actually operates in.
//     Allow 0.5 unit tolerance for the iterative path's discretization error.
function manySteps(steps, totalDt, target, speed) {
  let v = 0;
  const dt = totalDt / steps;
  for (let i = 0; i < steps; i++) v = smoothStep(v, target, speed, dt);
  return v;
}
const oneShot    = smoothStep(0, 100, 10, 0.04);
const splitFour  = manySteps(4,  0.04, 100, 10);
const splitTwenty = manySteps(20, 0.04, 100, 10);
assert.ok(Math.abs(oneShot - splitFour)     < 0.5, `frame-rate sensitivity 1↔4: ${oneShot} vs ${splitFour}`);
assert.ok(Math.abs(splitFour - splitTwenty) < 0.5, `frame-rate sensitivity 4↔20: ${splitFour} vs ${splitTwenty}`);

// --- dt clamp prevents teleport from huge gaps ---
const after10sGap = smoothStep(0, 100, 10, 10); // 10 second gap; clamped to 0.05
const after005s   = smoothStep(0, 100, 10, 0.05);
assert.equal(after10sGap, after005s, 'dt should clamp to 0.05');

console.log('motion: ok');
