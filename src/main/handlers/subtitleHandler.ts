import { spawn } from 'node:child_process';
import { stat, mkdir, rename, unlink } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { app } from 'electron';
import { logger } from '../services/logger';

export interface EmbeddedSubInfo {
  streamIndex: number;
  codec: string;
  language: string | null;
  title: string | null;
}

// PGS / DVD subtitles are bitmap formats — we can't render those without OCR.
// For text-based codecs we either keep them as ASS (rendered by libass via
// JASSUB in the renderer) or convert them to WebVTT (browser-native).
const ASS_FORMAT = new Set(['ass', 'ssa']);
const VTT_FORMAT = new Set(['subrip', 'webvtt', 'mov_text']);
function targetFormat(codec: string): 'ass' | 'vtt' | null {
  const c = codec.toLowerCase();
  if (ASS_FORMAT.has(c)) return 'ass';
  if (VTT_FORMAT.has(c)) return 'vtt';
  return null;
}

function getCacheDir(): string {
  return join(app.getPath('userData'), 'subtitle-cache');
}

async function ensureCacheDir(): Promise<string> {
  const dir = getCacheDir();
  await mkdir(dir, { recursive: true });
  return dir;
}

function cacheKeyHash(videoPath: string, mtimeMs: number, streamIndex: number): string {
  return createHash('md5').update(`${videoPath}:${mtimeMs}:${streamIndex}`).digest('hex');
}

function runFfprobe(args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    const p = spawn('ffprobe', args, { stdio: ['ignore', 'pipe', 'pipe'] });
    let out = '';
    let err = '';
    p.stdout.on('data', (d) => { out += d.toString(); });
    p.stderr.on('data', (d) => { err += d.toString(); });
    p.on('close', (code) => {
      if (code === 0) resolve(out);
      else reject(new Error(`ffprobe exit ${code}: ${err.slice(-300)}`));
    });
    p.on('error', reject);
  });
}

function runFfmpeg(args: string[]): Promise<void> {
  return new Promise((resolve, reject) => {
    const p = spawn('ffmpeg', args, { stdio: ['ignore', 'ignore', 'pipe'] });
    let err = '';
    p.stderr.on('data', (d) => { err += d.toString(); });
    p.on('close', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`ffmpeg exit ${code}: ${err.slice(-300)}`));
    });
    p.on('error', reject);
  });
}

// In-flight extractions keyed by the OUTPUT cache path. A background prewarm and
// the play-time extract can target the same stream at once; without this both
// would spawn `ffmpeg -y` writing the same file and corrupt it. Callers racing
// an in-flight job await the same promise instead of starting a second ffmpeg.
const inFlightExtract = new Map<string, Promise<{ path: string; format: 'ass' | 'vtt' } | null>>();

// Prewarm runs strictly one-at-a-time so that sweeping a long episode list
// (each row hover queues a prewarm) can never fan out into many concurrent
// full-file ffmpeg demuxes. Play-time extracts do NOT go through this chain, so
// pressing play never waits behind queued prewarms; the in-flight map above
// still de-dupes a prewarm and a play that target the same file. `prewarmSeen`
// keeps the same path from being queued twice in a session.
let prewarmChain: Promise<void> = Promise.resolve();
const prewarmSeen = new Set<string>();

const subtitleHandler = {
  async listEmbedded(videoPath: string): Promise<EmbeddedSubInfo[]> {
    if (!existsSync(videoPath)) return [];
    try {
      const json = await runFfprobe([
        '-v', 'error',
        '-select_streams', 's',
        '-show_entries', 'stream=index,codec_name:stream_tags=language,title',
        '-of', 'json',
        videoPath,
      ]);
      const parsed = JSON.parse(json) as {
        streams?: Array<{ index: number; codec_name?: string; tags?: { language?: string; title?: string } }>;
      };
      const all = (parsed.streams || []).map((s) => ({
        streamIndex: s.index,
        codec: s.codec_name ?? '',
        language: s.tags?.language ?? null,
        title: s.tags?.title ?? null,
      }));
      // Filter out subtitle codecs we can't render (bitmap PGS/DVD).
      const text = all.filter((s) => targetFormat(s.codec) !== null);
      if (all.length !== text.length) {
        logger.info('metadata', `Skipping ${all.length - text.length} non-text subtitle stream(s) (bitmap)`, { file: videoPath });
      }
      return text;
    } catch (err) {
      logger.warn('metadata', `Failed to list embedded subtitles: ${(err as Error).message}`, { file: videoPath });
      return [];
    }
  },

  /**
   * Extracts an embedded subtitle stream to a cache file. Preserves ASS/SSA
   * as ASS so libass (JASSUB) in the renderer can render it with full styling.
   * Other text formats convert to WebVTT for the browser's native track flow.
   * Returns the cache path and the format so the renderer knows which path
   * to take.
   */
  async extractEmbedded(videoPath: string, streamIndex: number, codec: string): Promise<{ path: string; format: 'ass' | 'vtt' } | null> {
    if (!existsSync(videoPath)) return null;
    const fmt = targetFormat(codec);
    if (!fmt) return null;
    let out: string;
    try {
      const stats = await stat(videoPath);
      const dir = await ensureCacheDir();
      out = join(dir, `${cacheKeyHash(videoPath, stats.mtimeMs, streamIndex)}.${fmt}`);
    } catch (err) {
      logger.warn('metadata', `Failed to resolve subtitle cache path for stream ${streamIndex}: ${(err as Error).message}`, { file: videoPath });
      return null;
    }
    if (existsSync(out)) return { path: out, format: fmt };
    // Coalesce a concurrent extract of the same output (prewarm vs play-time).
    const pending = inFlightExtract.get(out);
    if (pending) return pending;
    const job = (async () => {
      // Write to a PID-suffixed temp then atomic-rename, so existsSync(out) is
      // only ever true for a COMPLETE file. ffmpeg writes its output in place
      // and incrementally, so without this a cache-hit check (here or in a
      // racing reader) could hand back a half-written .ass — much more likely
      // now that a background prewarm can be mid-extract while the user plays.
      const tmp = `${out}.tmp.${process.pid}.${streamIndex}`;
      try {
        // ASS extraction: -c:s ass keeps the original styling/positioning.
        // VTT extraction: -c:s webvtt converts SRT/MOV_TEXT to WebVTT.
        await runFfmpeg([
          '-y',
          '-i', videoPath,
          '-map', `0:${streamIndex}`,
          '-c:s', fmt === 'ass' ? 'ass' : 'webvtt',
          tmp,
        ]);
        await rename(tmp, out);
        logger.info('metadata', `Extracted embedded subtitle stream ${streamIndex} (${fmt}) → cache`, { file: videoPath });
        return { path: out, format: fmt };
      } catch (err) {
        await unlink(tmp).catch(() => { /* tmp may not exist */ });
        logger.warn('metadata', `Failed to extract embedded subtitle stream ${streamIndex}: ${(err as Error).message}`, { file: videoPath });
        return null;
      }
    })().finally(() => inFlightExtract.delete(out));
    inFlightExtract.set(out, job);
    return job;
  },

  /**
   * Warm the embedded-subtitle cache for a file ahead of play time. ffmpeg has
   * to demux the whole container to pull a subtitle stream, which on a cold
   * cache takes roughly as long as an opening plays — so doing it at play time
   * is exactly why subtitles show up late on a first watch. Call this once a
   * file is likely to be played soon (the series page's "next up" episode; the
   * next episode while the current one is playing) so the play-time extract is
   * an instant cache hit. Best-effort and idempotent: a cache hit or in-flight
   * extract is a no-op and all errors are swallowed.
   */
  prewarm(videoPath: string): void {
    if (!videoPath || prewarmSeen.has(videoPath)) return;
    prewarmSeen.add(videoPath);
    prewarmChain = prewarmChain.then(async () => {
      try {
        if (!existsSync(videoPath)) return;
        const streams = await subtitleHandler.listEmbedded(videoPath);
        // Warm EVERY renderable track. The player's play-time buildSubs extracts
        // all embedded tracks sequentially and only then sets the subtitle list
        // that gates JASSUB, so warming just one wouldn't clear the stall on a
        // multi-track file. extractEmbedded caches + de-dupes, so if the user
        // plays this file the play-time loop becomes pure cache hits.
        for (const s of streams) {
          await subtitleHandler.extractEmbedded(videoPath, s.streamIndex, s.codec);
        }
      } catch {
        /* prewarm is best-effort; play-time extraction still works */
      }
    });
  },
};

export default subtitleHandler;
