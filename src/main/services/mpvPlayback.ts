// External mpv playback with progress reporting.
//
// The in-window player is HTML5 <video> (see the note in CLAUDE.md: mpv can't
// be embedded on Wayland+NVIDIA, don't revisit). "Open with mpv" therefore
// launches a separate window — but launching it blind meant anything watched
// there left no trace: no resume position, no view history, no tracker bump.
//
// So we launch mpv with a JSON IPC socket and poll it. mpv's IPC is
// newline-delimited JSON over a unix socket: write {"command": [...]} and read
// back {"data": ..., "request_id": n}. We poll `time-pos` rather than
// observing it, because an observed time-pos fires at video framerate (tens of
// messages a second) where a 2s poll is all a resume position needs.
//
// When mpv exits we hand the final position to the caller, which applies the
// same rules the in-window player uses: below the head window it never
// happened, past the tail window it's finished, in between it's a resume point.

import { spawn } from 'node:child_process';
import { connect, type Socket } from 'node:net';
import { join } from 'node:path';
import { mkdir, rm } from 'node:fs/promises';
import { app } from 'electron';
import { logger } from './logger';

/** How often we ask mpv where it is. The reading we ultimately keep is
 *  whatever the last poll before mpv exited returned, so this interval is also
 *  the worst-case staleness of the saved resume point — a second is well
 *  inside the noise for that, and still an order of magnitude cheaper than
 *  observing time-pos (which fires per frame). */
const POLL_INTERVAL_MS = 1000;
/** The socket doesn't exist the instant mpv is spawned; retry the connect for
 *  this long before giving up on tracking (playback itself still works). */
const CONNECT_TIMEOUT_MS = 10_000;
const CONNECT_RETRY_MS = 150;
/** Reject a per-poll jump larger than this as a seek rather than watch time.
 *  Mirrors the in-window player's MAX_TICK_DELTA, scaled to our poll rate. */
const MAX_TICK_DELTA_SEC = (POLL_INTERVAL_MS / 1000) * 1.5;

export interface MpvLaunchOptions {
  /** Where to resume from, in seconds. Omitted or 0 starts at the beginning. */
  startSec?: number;
}

export interface MpvPlaybackReport {
  filePath: string;
  /** Last observed playhead, in seconds. */
  position: number;
  /** Media duration in seconds, 0 when mpv never reported one. */
  duration: number;
  /** Seconds of actual forward playback observed, seeks excluded. Drives the
   *  "did they really watch this" decision the same way the in-window player's
   *  accumulator does. */
  watchedSec: number;
  /** False when we never managed to talk to mpv (socket refused, mpv too old,
   *  killed instantly). The caller must not treat a report like this as "the
   *  user watched 0 seconds" — it means we simply don't know. */
  tracked: boolean;
}

interface Session {
  socketPath: string;
  socket: Socket | null;
  timer: ReturnType<typeof setInterval> | null;
  position: number;
  duration: number;
  watchedSec: number;
  lastPosition: number | null;
  gotAnyData: boolean;
}

// A monotonic counter keeps concurrent mpv windows from colliding on a socket
// name without needing randomness.
let sessionSeq = 0;

function socketDir(): string {
  return join(app.getPath('userData'), 'mpv-ipc');
}

/**
 * Read newline-delimited JSON off the socket. mpv answers a `get_property`
 * with {"data": <value>, "request_id": n, "error": "success"} and also pushes
 * unsolicited {"event": ...} lines, which we ignore.
 */
function attachReader(session: Session, socket: Socket): void {
  let buf = '';
  socket.on('data', (chunk: Buffer) => {
    buf += chunk.toString('utf-8');
    let nl: number;
    while ((nl = buf.indexOf('\n')) >= 0) {
      const line = buf.slice(0, nl).trim();
      buf = buf.slice(nl + 1);
      if (!line) continue;
      let msg: { data?: unknown; request_id?: number; error?: string };
      try {
        msg = JSON.parse(line);
      } catch {
        continue;  // mpv only ever emits JSON lines; a partial one is harmless
      }
      if (msg.error && msg.error !== 'success') continue;
      if (typeof msg.data !== 'number' || !Number.isFinite(msg.data)) continue;
      session.gotAnyData = true;
      // request_id 1 = time-pos, 2 = duration. Set when we send the command.
      if (msg.request_id === 1) {
        const pos = msg.data;
        if (session.lastPosition !== null) {
          const delta = pos - session.lastPosition;
          // Only forward movement at roughly real-time counts. A backwards or
          // oversized jump is a seek, and scrubbing to the credits must not
          // read as having watched the episode.
          if (delta > 0 && delta <= MAX_TICK_DELTA_SEC) session.watchedSec += delta;
        }
        session.lastPosition = pos;
        session.position = pos;
      } else if (msg.request_id === 2) {
        session.duration = msg.data;
      }
    }
  });
}

function poll(session: Session): void {
  const socket = session.socket;
  if (!socket || socket.destroyed) return;
  try {
    socket.write(JSON.stringify({ command: ['get_property', 'time-pos'], request_id: 1 }) + '\n');
    // Duration is stable once known, but ask until we have it — it isn't
    // available at the instant the file opens.
    if (!session.duration) {
      socket.write(JSON.stringify({ command: ['get_property', 'duration'], request_id: 2 }) + '\n');
    }
  } catch {
    // Socket went away (mpv quit between the destroyed check and the write).
    // The exit handler does the reporting; nothing to do here.
  }
}

/** Keep retrying the connect until mpv has created the socket, or we give up. */
function connectWithRetry(session: Session, deadline: number): void {
  const socket = connect(session.socketPath);
  socket.on('connect', () => {
    session.socket = socket;
    attachReader(session, socket);
    poll(session);
    session.timer = setInterval(() => poll(session), POLL_INTERVAL_MS);
  });
  socket.on('error', () => {
    socket.destroy();
    if (session.socket) return;             // already connected on a later try
    if (Date.now() >= deadline) return;     // give up; playback is unaffected
    setTimeout(() => connectWithRetry(session, deadline), CONNECT_RETRY_MS);
  });
}

/**
 * Launch mpv on `filePath` and resolve once it exits, with everything we
 * managed to observe about the session. Rejects only if mpv couldn't be
 * spawned at all — a tracking failure comes back as `tracked: false` rather
 * than an error, because the user did get their video either way.
 */
export function playInMpv(filePath: string, opts?: MpvLaunchOptions): Promise<MpvPlaybackReport> {
  return new Promise<MpvPlaybackReport>((resolve, reject) => {
    void (async () => {
      const dir = socketDir();
      try {
        await mkdir(dir, { recursive: true });
      } catch (err) {
        logger.warn('system', `mpv: could not create IPC dir: ${(err as Error).message}`);
      }
      const socketPath = join(dir, `anibeam-${process.pid}-${++sessionSeq}.sock`);
      const session: Session = {
        socketPath,
        socket: null,
        timer: null,
        position: 0,
        duration: 0,
        watchedSec: 0,
        lastPosition: null,
        gotAnyData: false,
      };

      const args = [`--input-ipc-server=${socketPath}`];
      // Resume where the app (or a previous mpv session) left off. mpv takes
      // a plain seconds value here.
      if (opts?.startSec && opts.startSec > 0) args.push(`--start=${opts.startSec.toFixed(3)}`);
      args.push('--', filePath);

      // NOT detached (the pre-tracking version was): we need mpv's exit to
      // resolve this promise, which is what records the final position. stdio
      // stays ignored — mpv's terminal output isn't ours to surface.
      const child = spawn('mpv', args, { stdio: 'ignore' });

      const cleanup = async (): Promise<void> => {
        if (session.timer) clearInterval(session.timer);
        session.timer = null;
        session.socket?.destroy();
        session.socket = null;
        await rm(socketPath, { force: true }).catch(() => { /* mpv usually unlinks it itself */ });
      };

      child.on('error', (err) => {
        void cleanup();
        reject(err);
      });

      child.on('close', () => {
        // mpv is already gone, so the socket can't be asked again: the report
        // carries the last successful poll, up to POLL_INTERVAL_MS stale. That
        // is fine for both consumers — a resume point a second early, and a
        // finished-detection window (RESUME_TAIL_SKIP) measured in tens of
        // seconds.
        void cleanup().then(() => {
          resolve({
            filePath,
            position: session.position,
            duration: session.duration,
            watchedSec: session.watchedSec,
            tracked: session.gotAnyData,
          });
        });
      });

      connectWithRetry(session, Date.now() + CONNECT_TIMEOUT_MS);
    })();
  });
}
