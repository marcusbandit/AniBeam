// Eager pre-transcode cache. When the scanner finds a file with a
// codec Chromium's <video> can't decode (HEVC mostly), we transcode it
// once to a cached h.264/aac .mp4 under userData/transcode-cache/.
// The path is recorded on the file episode in metadata.json, so
// subsequent app launches see the cached file and skip the encode.
//
// One ffmpeg at a time. NVENC throughput is plenty fast; multiple
// concurrent encodes would just contend on the same GPU encoder slot
// and increase wall time per file.
//
// Cache key = sha256(filePath + mtime + size). Includes mtime+size so
// that if the user replaces a file in place (rip a new version of the
// same episode) we re-transcode automatically instead of serving the
// stale cache.

import { spawn, ChildProcess } from 'node:child_process';
import { mkdir, stat as fsStat, unlink, rename, readdir } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { app } from 'electron';
import { logger } from '../services/logger';
import metadataHandler from './metadataHandler';
import type { FileStatus } from '../../shared/fileStatus';
import type { FileEpisodeEntry } from '../../shared/fileEpisode';
import { probeCodecs, needsTranscode, ensureEncoder, type EncoderKind } from '../utils/transcodeProbe';

// Hard upper bound on the on-disk cache. When we cross this, oldest .mp4s
// (by mtime) get deleted until we're back under. 20 GB is a few hundred
// HEVC episodes worth of h264 cache — plenty for a typical browsing session
// without slowly eating the whole drive.
const MAX_CACHE_BYTES = 20 * 1024 * 1024 * 1024;

interface QueueEntry {
  filePath: string;
  resolve: () => void;
  reject: (err: Error) => void;
}

const queue: QueueEntry[] = [];
let active: { filePath: string; child: ChildProcess; outTmp: string } | null = null;
let onStatusChange: ((path: string, status: FileStatus) => Promise<void> | void) | null = null;
let onTranscodeReady: ((path: string, transcodedPath: string) => Promise<void> | void) | null = null;

function cacheDir(): string {
  return join(app.getPath('userData'), 'transcode-cache');
}

async function ensureCacheDir(): Promise<string> {
  const dir = cacheDir();
  await mkdir(dir, { recursive: true });
  return dir;
}

async function cacheKeyFor(filePath: string): Promise<string> {
  const s = await fsStat(filePath);
  return createHash('sha256')
    .update(`${filePath}:${s.mtimeMs}:${s.size}`)
    .digest('hex');
}

function cachePathForKey(key: string): string {
  return join(cacheDir(), `${key}.mp4`);
}

function ffmpegArgsFor(kind: EncoderKind, src: string, dst: string): string[] {
  // Common pieces — keep verbose logging off, drop subs/attachments (we
  // extract those separately from the original .mkv in subtitleHandler).
  const head = [
    '-hide_banner',
    '-loglevel', 'warning',
    '-nostats',
    '-y',
    '-analyzeduration', '500K', '-probesize', '500K',
  ];
  const select = [
    '-map', '0:v:0', '-map', '0:a:0?',
    '-sn', '-dn', '-map_chapters', '-1',
  ];
  // `+faststart` rewrites the moov atom to the front of the file so that
  // <video> can start decoding before the file is fully read off disk.
  const tail = [
    '-c:a', 'aac', '-b:a', '192k', '-ac', '2',
    '-movflags', '+faststart',
    dst,
  ];

  if (kind === 'vaapi') {
    return [
      ...head,
      '-hwaccel', 'vaapi', '-vaapi_device', '/dev/dri/renderD128',
      '-hwaccel_output_format', 'vaapi',
      '-i', src,
      '-vf', 'scale_vaapi=format=nv12',
      '-c:v', 'h264_vaapi',
      '-b:v', '5M', '-maxrate', '6M', '-bufsize', '12M',
      ...select,
      ...tail,
    ];
  }
  if (kind === 'nvenc') {
    return [
      ...head,
      '-hwaccel', 'cuda', '-hwaccel_output_format', 'cuda',
      '-i', src,
      '-vf', 'scale_cuda=format=nv12',
      '-c:v', 'h264_nvenc', '-preset', 'p4', '-tune', 'hq',
      '-b:v', '5M', '-maxrate', '6M', '-bufsize', '12M',
      ...select,
      ...tail,
    ];
  }
  return [
    ...head,
    '-i', src,
    '-c:v', 'libx264', '-preset', 'veryfast', '-crf', '22',
    '-pix_fmt', 'yuv420p',
    ...select,
    ...tail,
  ];
}

async function emitStatus(path: string, status: FileStatus): Promise<void> {
  if (onStatusChange) {
    try {
      await onStatusChange(path, status);
    } catch (err) {
      logger.warn('system', `transcodeCache status emit failed: ${(err as Error).message}`);
    }
  }
}

async function persistTranscodedPath(filePath: string, transcodedPath: string): Promise<void> {
  await metadataHandler.transaction<boolean>(async (meta) => {
    let changed = false;
    for (const series of Object.values(meta)) {
      const s = series as { fileEpisodes?: Array<{ filePath: string; transcodedPath?: string | null }> };
      if (!Array.isArray(s.fileEpisodes)) continue;
      for (const file of s.fileEpisodes) {
        if (file.filePath === filePath) {
          file.transcodedPath = transcodedPath;
          changed = true;
        }
      }
    }
    return { result: changed, updated: changed ? meta : null };
  });
}

async function runOne(entry: QueueEntry): Promise<void> {
  const { filePath } = entry;
  if (!existsSync(filePath)) {
    entry.reject(new Error(`File missing: ${filePath}`));
    return;
  }

  // Re-check codec at run time — the scanner may have flagged the file
  // based on filename or pre-existing metadata. If the file's actually
  // browser-compatible (someone re-encoded it manually) we skip work.
  const probe = await probeCodecs(filePath);
  if (!probe) {
    entry.reject(new Error('probe failed'));
    return;
  }
  if (!needsTranscode(probe)) {
    // Already compatible — record nothing, status back to ready.
    await emitStatus(filePath, 'ready');
    entry.resolve();
    return;
  }

  const dir = await ensureCacheDir();
  const key = await cacheKeyFor(filePath);
  const finalPath = cachePathForKey(key);

  // Cache hit on disk but no transcodedPath in metadata yet: persist
  // and bail out without re-encoding.
  if (existsSync(finalPath)) {
    if (onTranscodeReady) await onTranscodeReady(filePath, finalPath);
    await emitStatus(filePath, 'ready');
    entry.resolve();
    return;
  }

  const tmpPath = join(dir, `${key}.tmp.mp4`);
  // If a previous run was killed mid-encode, the .tmp will linger. Try to
  // remove unconditionally; ENOENT just means there was nothing to clean.
  await unlink(tmpPath).catch((err: NodeJS.ErrnoException) => {
    if (err.code !== 'ENOENT') logger.warn('system', `transcodeCache: stray tmp cleanup failed: ${err.message}`);
  });

  await emitStatus(filePath, 'transcoding');
  const encoder = await ensureEncoder();
  const args = ffmpegArgsFor(encoder, filePath, tmpPath);
  logger.info('system', `Transcoding (${probe.vCodec}→h264 via ${encoder})`, { file: filePath });

  await new Promise<void>((resolve, reject) => {
    const child = spawn('ffmpeg', args, { stdio: ['ignore', 'ignore', 'pipe'] });
    active = { filePath, child, outTmp: tmpPath };
    let stderr = '';
    child.stderr.on('data', (buf: Buffer) => {
      stderr += buf.toString();
      // Don't let stderr grow unbounded over a long encode. Keep just
      // the tail for diagnostics on failure.
      if (stderr.length > 8192) stderr = stderr.slice(-4096);
    });
    child.on('error', (err) => {
      active = null;
      reject(err);
    });
    child.on('close', (code) => {
      active = null;
      if (code === 0) resolve();
      else reject(new Error(`ffmpeg exit ${code}: ${stderr.trim().split('\n').slice(-3).join(' | ')}`));
    });
  })
    .then(async () => {
      // Atomic publish — rename succeeds only after ffmpeg fully closed
      // the file, so callers never see a partial .mp4.
      await rename(tmpPath, finalPath);
      if (onTranscodeReady) await onTranscodeReady(filePath, finalPath);
      await emitStatus(filePath, 'ready');
      logger.info('system', `Transcoded → cache (${finalPath})`, { file: filePath });
      entry.resolve();
    })
    .catch(async (err) => {
      // Clean up any partial output. Leave the source untouched. ENOENT is
      // fine — ffmpeg may have been killed before writing anything.
      await unlink(tmpPath).catch((cleanupErr: NodeJS.ErrnoException) => {
        if (cleanupErr.code !== 'ENOENT') {
          logger.warn('system', `transcodeCache: tmp cleanup after failure: ${cleanupErr.message}`);
        }
      });
      await emitStatus(filePath, 'stalled');
      logger.warn('system', `Transcode failed: ${(err as Error).message}`, { file: filePath });
      entry.reject(err as Error);
    });
}

let pumpInFlight = false;
async function pump(): Promise<void> {
  if (pumpInFlight) return;
  pumpInFlight = true;
  try {
    while (queue.length > 0) {
      const next = queue.shift()!;
      try { await runOne(next); } catch { /* runOne already reported */ }
    }
  } finally {
    pumpInFlight = false;
  }
}

const transcodeCacheHandler = {
  /**
   * Set callbacks. `onStatus` mirrors videoProbeHandler.start's contract.
   * `onReady` fires when a transcode completes so the caller can persist
   * the path to metadata (this module also persists internally, but the
   * caller gets a hook for renderer notification).
   */
  start(
    onStatus: (path: string, status: FileStatus) => Promise<void> | void,
    onReady?: (path: string, transcodedPath: string) => Promise<void> | void,
  ): void {
    onStatusChange = onStatus;
    onTranscodeReady = async (path, transcoded) => {
      await persistTranscodedPath(path, transcoded);
      if (onReady) await onReady(path, transcoded);
    };
  },

  /**
   * Add a file to the queue. Returns a promise that settles when this
   * file's encode finishes (or fails / is skipped because the cache
   * already contains a usable copy). Safe to call multiple times for
   * the same path — duplicates collapse into a single in-flight encode.
   */
  enqueue(filePath: string): Promise<void> {
    if (active?.filePath === filePath) {
      return new Promise((resolve, reject) => {
        const c = active!.child;
        c.on('close', (code) => (code === 0 ? resolve() : reject(new Error(`ffmpeg exit ${code}`))));
        c.on('error', reject);
      });
    }
    const existing = queue.find((e) => e.filePath === filePath);
    if (existing) {
      return new Promise((resolve, reject) => {
        // Chain onto the existing entry's settlement.
        const origResolve = existing.resolve;
        const origReject = existing.reject;
        existing.resolve = () => { origResolve(); resolve(); };
        existing.reject = (err) => { origReject(err); reject(err); };
      });
    }
    return new Promise((resolve, reject) => {
      queue.push({ filePath, resolve, reject });
      void pump();
    });
  },

  /**
   * Compute the cache path a file SHOULD have, without enqueueing.
   * Useful for the startup-validation pass that confirms previously
   * cached files still exist on disk.
   */
  async cachePathFor(filePath: string): Promise<string | null> {
    if (!existsSync(filePath)) return null;
    try {
      const key = await cacheKeyFor(filePath);
      return cachePathForKey(key);
    } catch {
      return null;
    }
  },

  /**
   * Best-effort cleanup when an original file is removed. Looks up the
   * cache path for the (now possibly missing) source and deletes the
   * cached .mp4 if it exists.
   */
  async cleanupFor(filePath: string): Promise<void> {
    try {
      // We need mtime/size to compute the key; if the file's already gone
      // we can't. Scanning the cache dir for orphans would be heavier than
      // it's worth — let it ride. A future "purge orphaned cache entries"
      // maintenance task can sweep.
      if (!existsSync(filePath)) return;
      const key = await cacheKeyFor(filePath);
      const cached = cachePathForKey(key);
      await unlink(cached).then(
        () => logger.info('system', `Removed cached transcode for deleted file`, { file: cached }),
        (err: NodeJS.ErrnoException) => {
          if (err.code !== 'ENOENT') {
            logger.warn('system', `transcodeCache cleanup failed: ${err.message}`);
          }
        },
      );
    } catch (err) {
      logger.warn('system', `transcodeCache cleanup failed: ${(err as Error).message}`);
    }
  },

  /**
   * Probe + decide. Returns true if this file would need transcoding
   * to be browser-playable. Used by the main process after probe-ready
   * to decide whether to enqueue.
   */
  async shouldTranscode(filePath: string): Promise<boolean> {
    const p = await probeCodecs(filePath);
    if (!p) return false;
    return needsTranscode(p);
  },

  /**
   * Maintenance sweep. Two passes:
   *   1. Drop cache files whose `transcodedPath` is no longer referenced
   *      by any fileEpisode (e.g. metadata.json was edited externally,
   *      a series was deleted, the source file moved and re-keyed).
   *   2. If the remaining cache is still over MAX_CACHE_BYTES, evict
   *      oldest-by-mtime until we're back under.
   *
   * Best-effort, never throws. Designed to run once at app startup.
   */
  async pruneCacheNow(): Promise<void> {
    try {
      const dir = await ensureCacheDir();
      let entries: string[];
      try {
        entries = await readdir(dir);
      } catch (err) {
        if ((err as NodeJS.ErrnoException).code === 'ENOENT') return;
        throw err;
      }

      // Build the set of paths metadata still claims to own.
      const referenced = new Set<string>();
      const meta = await metadataHandler.loadMetadata();
      for (const series of Object.values(meta)) {
        const s = series as { fileEpisodes?: FileEpisodeEntry[] };
        if (!Array.isArray(s.fileEpisodes)) continue;
        for (const f of s.fileEpisodes) {
          if (f.transcodedPath) referenced.add(f.transcodedPath);
        }
      }

      // Recovery pass: before deleting "orphan" .mp4s, see if any are
      // actually a valid cache for a source file whose `transcodedPath`
      // was wiped (the ingestSingleFile bug used to do this). Re-derive
      // the cache key from each source file; any match → re-bind metadata.
      // Saves the user a full re-transcode on the next launch.
      const cacheBasenames = new Set(entries.filter((n) => n.endsWith('.mp4')));
      const recoveredBindings: Array<{ filePath: string; cachePath: string }> = [];
      for (const series of Object.values(meta)) {
        const s = series as { fileEpisodes?: FileEpisodeEntry[] };
        if (!Array.isArray(s.fileEpisodes)) continue;
        for (const f of s.fileEpisodes) {
          if (f.transcodedPath) continue;
          if (!existsSync(f.filePath)) continue;
          try {
            const key = await cacheKeyFor(f.filePath);
            const candidateName = `${key}.mp4`;
            if (!cacheBasenames.has(candidateName)) continue;
            const candidatePath = join(dir, candidateName);
            recoveredBindings.push({ filePath: f.filePath, cachePath: candidatePath });
            referenced.add(candidatePath);
          } catch {
            // stat/hash failure — let this file be re-transcoded later.
          }
        }
      }
      if (recoveredBindings.length > 0) {
        await metadataHandler.transaction<boolean>(async (current) => {
          let changed = false;
          const byPath = new Map(recoveredBindings.map((b) => [b.filePath, b.cachePath]));
          for (const series of Object.values(current)) {
            const s = series as { fileEpisodes?: FileEpisodeEntry[] };
            if (!Array.isArray(s.fileEpisodes)) continue;
            for (const file of s.fileEpisodes) {
              const cachePath = byPath.get(file.filePath);
              if (cachePath && !file.transcodedPath) {
                file.transcodedPath = cachePath;
                changed = true;
              }
            }
          }
          return { result: changed, updated: changed ? current : null };
        });
        logger.info('system', `Transcode cache: recovered ${recoveredBindings.length} orphan(s) by re-binding to metadata`);
      }

      const survivors: Array<{ path: string; size: number; mtimeMs: number }> = [];
      let totalSize = 0;
      let orphansRemoved = 0;
      for (const name of entries) {
        if (!name.endsWith('.mp4')) continue;
        const fullPath = join(dir, name);
        try {
          const st = await fsStat(fullPath);
          if (!referenced.has(fullPath)) {
            await unlink(fullPath).catch(() => { /* race ok */ });
            orphansRemoved++;
            continue;
          }
          survivors.push({ path: fullPath, size: st.size, mtimeMs: st.mtimeMs });
          totalSize += st.size;
        } catch (err) {
          if ((err as NodeJS.ErrnoException).code !== 'ENOENT') {
            logger.warn('system', `transcodeCache prune stat failed: ${(err as Error).message}`, { file: fullPath });
          }
        }
      }

      let quotaEvicted = 0;
      if (totalSize > MAX_CACHE_BYTES) {
        survivors.sort((a, b) => a.mtimeMs - b.mtimeMs);
        while (totalSize > MAX_CACHE_BYTES && survivors.length > 0) {
          const victim = survivors.shift()!;
          await unlink(victim.path).catch(() => { /* race ok */ });
          totalSize -= victim.size;
          quotaEvicted++;
        }
        // Drop now-stale transcodedPath references in metadata too.
        if (quotaEvicted > 0) {
          await metadataHandler.transaction<boolean>(async (current) => {
            let changed = false;
            for (const series of Object.values(current)) {
              const s = series as { fileEpisodes?: FileEpisodeEntry[] };
              if (!Array.isArray(s.fileEpisodes)) continue;
              for (const f of s.fileEpisodes) {
                if (f.transcodedPath && !existsSync(f.transcodedPath)) {
                  f.transcodedPath = null;
                  changed = true;
                }
              }
            }
            return { result: changed, updated: changed ? current : null };
          });
        }
      }

      if (orphansRemoved > 0 || quotaEvicted > 0) {
        logger.info('system', `Transcode cache pruned: ${orphansRemoved} orphan(s), ${quotaEvicted} over-quota`);
      }
    } catch (err) {
      logger.warn('system', `Transcode cache prune failed: ${(err as Error).message}`);
    }
  },
};

export default transcodeCacheHandler;
