import { spawn } from 'node:child_process';
import { stat } from 'node:fs/promises';
import { logger } from '../services/logger';
import type { FileStatus } from '../../shared/fileStatus';

export interface ProbeResult {
  ready: boolean;
  reason?: string;
}

interface QueuedFile {
  path: string;
  attempts: number;
  enqueuedAt: number;
  nextRunAt: number;
  lastSize: number;
  lastMtimeMs: number;
  stableSinceMs: number;
}

const BACKOFFS_MS = [5_000, 15_000, 30_000];
const STEADY_BACKOFF_MS = 60_000;
const MAX_LIFETIME_MS = 30 * 60_000;
const SIZE_STABLE_THRESHOLD_MS = 10_000;
const POLL_INTERVAL_MS = 2_000;

const queue = new Map<string, QueuedFile>();
let pollHandle: NodeJS.Timeout | null = null;
let onStatusChange: ((path: string, status: FileStatus) => Promise<void> | void) | null = null;

export function parseFfprobeJson(stdout: string): ProbeResult {
  let parsed: { streams?: Array<{ codec_type?: string }>; format?: { duration?: string | number } };
  try {
    parsed = JSON.parse(stdout);
  } catch {
    return { ready: false, reason: 'invalid ffprobe output' };
  }
  const hasVideoStream = Array.isArray(parsed.streams) && parsed.streams.some((s) => s.codec_type === 'video');
  if (!hasVideoStream) return { ready: false, reason: 'no video stream' };
  const dur = parsed.format?.duration;
  const durNum = typeof dur === 'string' ? Number(dur) : (dur ?? 0);
  if (!Number.isFinite(durNum) || durNum <= 0) return { ready: false, reason: 'no duration' };
  return { ready: true };
}

function runFfprobe(path: string, timeoutMs = 15_000): Promise<ProbeResult> {
  return new Promise((resolve) => {
    const child = spawn('ffprobe', ['-v', 'error', '-show_streams', '-show_format', '-of', 'json', path], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let stdout = '';
    let stderr = '';
    const timer = setTimeout(() => {
      child.kill();
      resolve({ ready: false, reason: 'ffprobe timeout' });
    }, timeoutMs);
    child.stdout.on('data', (buf) => { stdout += buf.toString(); });
    child.stderr.on('data', (buf) => { stderr += buf.toString(); });
    child.on('error', (err) => {
      clearTimeout(timer);
      resolve({ ready: false, reason: `ffprobe spawn error: ${err.message}` });
    });
    child.on('close', (code) => {
      clearTimeout(timer);
      if (code !== 0) {
        const detail = stderr.trim().split('\n').slice(-1)[0]?.slice(0, 200);
        resolve({ ready: false, reason: detail ? `ffprobe exit ${code}: ${detail}` : `ffprobe exit ${code}` });
        return;
      }
      resolve(parseFfprobeJson(stdout));
    });
  });
}

function nextDelay(attempts: number): number {
  if (attempts < BACKOFFS_MS.length) return BACKOFFS_MS[attempts];
  return STEADY_BACKOFF_MS;
}

async function probeOne(path: string): Promise<void> {
  const entry = queue.get(path);
  if (!entry) return;
  entry.attempts++;
  const result = await runFfprobe(path);
  if (result.ready) {
    queue.delete(path);
    logger.info('probe', `Ready`, { file: path });
    if (onStatusChange) await onStatusChange(path, 'ready');
    return;
  }
  if (Date.now() - entry.enqueuedAt > MAX_LIFETIME_MS) {
    queue.delete(path);
    logger.warn('probe', `Stalled (${result.reason ?? 'unknown'})`, { file: path });
    if (onStatusChange) await onStatusChange(path, 'stalled');
    return;
  }
  entry.nextRunAt = Date.now() + nextDelay(entry.attempts);
}

// Concurrency guard — `setInterval` doesn't await `tick()`, and a single
// hung ffprobe (15s timeout) would otherwise let multiple ticks overlap and
// double-probe the same entry.
let tickInFlight = false;

async function tick(): Promise<void> {
  if (tickInFlight) return;
  tickInFlight = true;
  try {
    await tickInner();
  } finally {
    tickInFlight = false;
  }
}

async function tickInner(): Promise<void> {
  const now = Date.now();
  for (const [path, entry] of queue) {
    // size-stable check
    try {
      const s = await stat(path);
      const sizeChanged = s.size !== entry.lastSize;
      const mtimeChanged = s.mtimeMs !== entry.lastMtimeMs;
      if (sizeChanged || mtimeChanged) {
        entry.lastSize = s.size;
        entry.lastMtimeMs = s.mtimeMs;
        entry.stableSinceMs = now;
      } else if (now - entry.stableSinceMs >= SIZE_STABLE_THRESHOLD_MS) {
        // stable — probe immediately if we haven't already this tick
        await probeOne(path);
        continue;
      }
    } catch {
      // file might be temporarily inaccessible; let backoff handle it
    }
    if (now >= entry.nextRunAt) {
      await probeOne(path);
    }
  }
}

const videoProbeHandler = {
  /**
   * Single-shot probe. Used by the queue and by manual callers.
   */
  probe(path: string): Promise<ProbeResult> {
    return runFfprobe(path);
  },

  /**
   * Enqueue a file for verification with backoff.
   */
  enqueue(path: string): void {
    if (queue.has(path)) return;
    const now = Date.now();
    queue.set(path, {
      path,
      attempts: 0,
      enqueuedAt: now,
      nextRunAt: now,
      lastSize: -1,
      lastMtimeMs: -1,
      stableSinceMs: now,
    });
    logger.info('probe', `Verifying`, { file: path });
  },

  /**
   * Re-enqueue a file (clears prior history).
   */
  retry(path: string): void {
    queue.delete(path);
    this.enqueue(path);
  },

  start(handler: (path: string, status: FileStatus) => Promise<void> | void): void {
    onStatusChange = handler;
    if (pollHandle) return;
    pollHandle = setInterval(() => { void tick(); }, POLL_INTERVAL_MS);
  },

  stop(): void {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
    queue.clear();
  },
};

export default videoProbeHandler;
