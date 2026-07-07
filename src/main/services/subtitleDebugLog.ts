import { appendFile, mkdir, rename } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

// Subtitle-focused debug log: an append-only file under userData/logs that
// survives packaging, so subtitle behavior can be tailed while the .desktop-
// launched binary runs (`bun run logs:subs`). This is deliberately separate
// from services/logger (the user-facing activity drawer, which is signal-only);
// this file is allowed to be chatty about every probe/extract/JASSUB step.
//
// No electron import: main.ts injects the directory at app-ready time, which
// keeps this module importable from the bun verify scripts.

const LOG_NAME = 'subtitles.log';
const PREV_NAME = 'subtitles.prev.log';
const DATA_CAP = 4000;

let logPath: string | null = null;
// Appends are chained on a single promise so lines land in call order and a
// slow disk can never interleave two writes (same idiom as the metadata
// transaction serialization).
let queue: Promise<void> = Promise.resolve();
let reportedWriteError = false;

function firstStackLine(err: Error): string | null {
  const line = err.stack?.split('\n')[1]?.trim();
  return line || null;
}

// Throw-safe stringify: circular refs collapse, Errors become message + first
// stack line, output is capped so one huge payload can't bloat the log.
function safeStringify(data: unknown): string {
  const seen = new WeakSet<object>();
  let json: string;
  try {
    json = JSON.stringify(data, (_key, value: unknown) => {
      if (value instanceof Error) {
        const at = firstStackLine(value);
        return at ? `${value.message} (${at})` : value.message;
      }
      if (typeof value === 'bigint') return String(value);
      if (typeof value === 'object' && value !== null) {
        if (seen.has(value)) return '[circular]';
        seen.add(value);
      }
      return value;
    }) ?? String(data);
  } catch (err) {
    json = `"[unserializable: ${(err as Error).message}]"`;
  }
  return json.length > DATA_CAP ? `${json.slice(0, DATA_CAP)}...` : json;
}

function append(line: string): void {
  const path = logPath;
  if (!path) return;
  queue = queue.then(async () => {
    try {
      await appendFile(path, line, 'utf8');
    } catch (err) {
      if (!reportedWriteError) {
        reportedWriteError = true;
        console.error('[subtitleDebugLog] write failed, log lines are being dropped:', err);
      }
    }
  });
}

/**
 * Point the sink at a directory and start a fresh session. The previous
 * session's file (if any) rotates to subtitles.prev.log, so the last two
 * sessions are always inspectable. Never throws into the caller.
 */
export function initSubtitleDebugLog(dir: string, header?: Record<string, unknown>): void {
  const path = join(dir, LOG_NAME);
  logPath = path;
  reportedWriteError = false;
  queue = queue.then(async () => {
    try {
      await mkdir(dir, { recursive: true });
      if (existsSync(path)) await rename(path, join(dir, PREV_NAME));
    } catch (err) {
      if (!reportedWriteError) {
        reportedWriteError = true;
        console.error('[subtitleDebugLog] init failed, log lines are being dropped:', err);
      }
    }
  });
  const fields = header ? ` ${safeStringify(header)}` : '';
  append(`--- session start ${new Date().toISOString()}${fields} ---\n`);
}

/**
 * Append one line: `<ISO timestamp> [<scope>] <message> | <json data>`.
 * Fire-and-forget; a no-op before init; never throws.
 */
export function subLog(scope: string, message: string, data?: unknown): void {
  if (!logPath) return;
  // Keep one event per line so `tail -F` and grep stay useful; ffmpeg stderr
  // tails and stack traces carry raw newlines.
  const flat = message.replace(/\r?\n/g, '\\n');
  const suffix = data === undefined ? '' : ` | ${safeStringify(data)}`;
  append(`${new Date().toISOString()} [${scope}] ${flat}${suffix}\n`);
}

/** Active log file path, or null before init. */
export function subtitleLogPath(): string | null {
  return logPath;
}

/** Resolves once every line queued so far has hit disk (for verify scripts). */
export function flushSubtitleDebugLog(): Promise<void> {
  return queue;
}
