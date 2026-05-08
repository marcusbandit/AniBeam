// Per-provider request queue with exponential-backoff retry on rate-limit
// errors. Solves two problems at once:
//
//  1. Bursts. `Promise.all([mal.search(...), anilist.search(...)])` per
//     scanned folder × N folders fans out into 100s of parallel requests.
//     AniList allows ~90/min, Jikan ~60/min — both providers cascade 429s
//     and our per-call retry loops compound the problem (each retry hits
//     the same overloaded window).
//  2. Recovery. After a 429, the queue waits with exponential backoff and
//     keeps the rest of the work pending. No request is dropped silently;
//     the caller's awaited promise resolves once the request lands.
//
// One limiter instance per provider — they don't share rate budgets and
// shouldn't block each other.

import { logger } from '../services/logger';

interface LimiterOptions {
  source: string;                                // for log messages
  minIntervalMs: number;                         // minimum gap between request starts
  maxRetries: number;                            // 429 retry ceiling per request
  isRateLimitError: (e: unknown) => boolean;     // provider-specific 429 detection
}

const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

export class RateLimiter {
  private queue: Array<() => Promise<void>> = [];
  private draining = false;

  constructor(private readonly opts: LimiterOptions) {}

  run<T>(fn: () => Promise<T>): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      this.queue.push(async () => {
        try {
          resolve(await this.attempt(fn, 0));
        } catch (err) {
          reject(err);
        }
      });
      void this.drain();
    });
  }

  private async drain(): Promise<void> {
    if (this.draining) return;
    this.draining = true;
    try {
      while (this.queue.length > 0) {
        const task = this.queue.shift()!;
        await task();
        if (this.queue.length > 0) {
          await sleep(this.opts.minIntervalMs);
        }
      }
    } finally {
      this.draining = false;
    }
  }

  private async attempt<T>(fn: () => Promise<T>, retries: number): Promise<T> {
    try {
      return await fn();
    } catch (err) {
      if (this.opts.isRateLimitError(err) && retries < this.opts.maxRetries) {
        // 1s, 2s, 4s, 8s, 16s, 32s — cap at 60s. Six retries cover a
        // typical rolling-minute window with room to spare.
        const delay = Math.min(60_000, 1000 * 2 ** retries);
        logger.warn(
          'metadata',
          `${this.opts.source} 429 — backoff ${delay}ms (retry ${retries + 1}/${this.opts.maxRetries})`,
        );
        await sleep(delay);
        return this.attempt(fn, retries + 1);
      }
      throw err;
    }
  }
}
