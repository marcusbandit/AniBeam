// Single-frame stepping for the in-window player. HTML5 video has no
// frame-step API, so a step is a seek by one frame duration. Two things make
// that reliable rather than approximate:
//
// 1. The frame duration is learned, not assumed. requestVideoFrameCallback
//    reports the presentation timestamp (mediaTime) of every frame that
//    reaches the screen; the deltas between consecutive timestamps are frame
//    durations. We keep the minimum of a recent window, because a dropped
//    presentation only ever makes a delta bigger, never smaller. Until any
//    frame has been seen we fall back to 23.976, which nearly all anime is.
//
// 2. Every step is anchored on the timestamp of the frame actually on
//    screen, never on currentTime. After a seek, currentTime is the time we
//    asked for, which deliberately sits mid-frame; anchoring on it would
//    compound half a frame per step and start skipping frames. Targets land
//    in the middle of the wanted frame's interval (anchor + 1.5d forward,
//    anchor - 0.5d back), so an estimate that is a little off still lands on
//    the right frame.
//
// Pure logic, no Electron or React, so the verify script can import it.

export const FALLBACK_FRAME_DURATION = 1001 / 24000;
export const MIN_FRAME_DURATION = 1 / 240;
export const MAX_FRAME_DURATION = 1 / 10;
export const MAX_PENDING_STEPS = 3;
export const PAINT_TIMEOUT_MS = 500;
const DELTA_WINDOW = 64;

export type StepDirection = 1 | -1;

export class FrameDurationEstimator {
  private lastMediaTime: number | null = null;
  private deltas: number[] = [];

  observe(mediaTime: number): void {
    if (!Number.isFinite(mediaTime)) return;
    const prev = this.lastMediaTime;
    this.lastMediaTime = mediaTime;
    if (prev == null) return;
    const delta = mediaTime - prev;
    // Anything outside the plausible band is a seek, a repaint of the same
    // frame, or a backwards jump, none of which is a frame duration.
    if (delta < MIN_FRAME_DURATION || delta > MAX_FRAME_DURATION) return;
    this.deltas.push(delta);
    if (this.deltas.length > DELTA_WINDOW) this.deltas.shift();
  }

  get frameDuration(): number {
    if (this.deltas.length === 0) return FALLBACK_FRAME_DURATION;
    let min = this.deltas[0];
    for (const d of this.deltas) if (d < min) min = d;
    return min;
  }

  get observed(): boolean {
    return this.deltas.length > 0;
  }

  reset(): void {
    this.lastMediaTime = null;
    this.deltas = [];
  }
}

/**
 * Where to seek so the frame after (or before) the one at `anchorPts` is the
 * one Chromium paints. A non-finite or non-positive duration means the upper
 * bound is unknown and only the lower clamp applies.
 */
export function stepTarget(
  anchorPts: number,
  direction: StepDirection,
  frameDuration: number,
  duration: number,
): number {
  const raw = direction > 0 ? anchorPts + 1.5 * frameDuration : anchorPts - 0.5 * frameDuration;
  const upper = Number.isFinite(duration) && duration > 0 ? duration : Infinity;
  return Math.min(upper, Math.max(0, raw));
}

/** Frame number of the frame at `pts`, assuming a constant frame rate. */
export function estimatedFrameIndex(pts: number, frameDuration: number): number {
  if (!Number.isFinite(pts) || !Number.isFinite(frameDuration) || frameDuration <= 0) return 0;
  return Math.max(0, Math.round(pts / frameDuration));
}

function pad2(n: number): string {
  return n < 10 ? `0${n}` : `${n}`;
}

/** "m:ss.mmm", or "h:mm:ss.mmm" once there is an hour to show. */
export function formatTimeMs(s: number): string {
  if (!Number.isFinite(s) || s < 0) return '0:00.000';
  const totalMs = Math.round(s * 1000);
  const ms = totalMs % 1000;
  const total = Math.floor(totalMs / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const sec = total % 60;
  const msStr = String(ms).padStart(3, '0');
  if (h > 0) return `${h}:${pad2(m)}:${pad2(sec)}.${msStr}`;
  return `${m}:${pad2(sec)}.${msStr}`;
}

export interface FrameStepIo {
  seek(t: number): void;
  getCurrentTime(): number;
  getDuration(): number;
  /**
   * True while the element has a seek in flight. The `seeking` event is a
   * queued task, so a press that lands in the same task as a scrub would
   * otherwise anchor on the frame the scrub is about to replace.
   */
  isSeeking?(): boolean;
  /** Run `fn` after `ms`; returns a cancel. */
  schedule(fn: () => void, ms: number): () => void;
  /**
   * HUD feedback. Fires for every presented frame while a stepping session is
   * active (a step in flight or queued), so what the HUD shows is always the
   * frame that is really on screen.
   */
  onLanded?(pts: number, frameDuration: number): void;
}

/**
 * Serialises step requests against the video's own seek/paint cycle. Only one
 * seek is ever in flight; further presses queue (bounded, and opposite
 * directions cancel out) and each one is anchored on the frame the previous
 * one actually landed on.
 */
export class FrameStepper {
  private pending = 0;
  private busy = false;
  private paintPending = false;
  private pts: number | null = null;
  private target = 0;
  private cancelTimer: (() => void) | null = null;
  private readonly estimator: FrameDurationEstimator;

  constructor(
    private readonly io: FrameStepIo,
    estimator: FrameDurationEstimator = new FrameDurationEstimator(),
  ) {
    this.estimator = estimator;
  }

  get lastPts(): number | null {
    return this.pts;
  }

  get frameDuration(): number {
    return this.estimator.frameDuration;
  }

  request(direction: StepDirection): void {
    const next = this.pending + direction;
    this.pending = Math.max(-MAX_PENDING_STEPS, Math.min(MAX_PENDING_STEPS, next));
    this.drain();
  }

  /** A seek started (ours or the scrubber's): a new paint is on its way. */
  onSeeking(): void {
    this.paintPending = true;
    this.armPaintTimeout();
  }

  onFramePresented(mediaTime: number): void {
    this.estimator.observe(mediaTime);
    this.pts = mediaTime;
    this.paintPending = false;
    this.clearTimer();
    const sessionActive = this.busy || this.pending !== 0;
    this.busy = false;
    if (sessionActive) this.io.onLanded?.(mediaTime, this.estimator.frameDuration);
    this.drain();
  }

  reset(): void {
    this.pending = 0;
    this.busy = false;
    this.paintPending = false;
    this.pts = null;
    this.estimator.reset();
    this.clearTimer();
  }

  /**
   * Playback resumed: whatever is still queued must not fire into a playing
   * video. Keeps the learned frame duration and the last presented frame.
   */
  cancelPending(): void {
    this.pending = 0;
    this.busy = false;
    this.paintPending = false;
    this.clearTimer();
  }

  private drain(): void {
    if (this.busy || this.pending === 0 || this.paintPending) return;
    if (this.io.isSeeking?.()) {
      this.onSeeking();
      return;
    }
    const direction: StepDirection = this.pending > 0 ? 1 : -1;
    this.pending -= direction;
    const anchor = this.pts ?? this.io.getCurrentTime();
    const target = stepTarget(anchor, direction, this.estimator.frameDuration, this.io.getDuration());
    // Clamped into place: nothing before frame zero, nothing past the end.
    // Everything still queued points the same way, so it goes nowhere either;
    // drop it rather than leave it to fire into the next presented frame.
    if (target === anchor) {
      this.pending = 0;
      return;
    }
    this.busy = true;
    this.paintPending = true;
    this.target = target;
    this.armPaintTimeout();
    this.io.seek(target);
  }

  // If the paint never arrives (no requestVideoFrameCallback, or a seek that
  // repainted nothing) assume the frame we asked for is the one showing and
  // keep going, so stepping still works, it just cannot self-correct.
  private armPaintTimeout(): void {
    this.clearTimer();
    this.cancelTimer = this.io.schedule(() => {
      this.cancelTimer = null;
      this.paintPending = false;
      if (this.busy) {
        this.busy = false;
        this.pts = this.target - 0.5 * this.estimator.frameDuration;
      }
      this.drain();
    }, PAINT_TIMEOUT_MS);
  }

  private clearTimer(): void {
    if (this.cancelTimer) {
      this.cancelTimer();
      this.cancelTimer = null;
    }
  }
}
