import { spawn } from 'node:child_process';
import { stat, mkdir, rename, unlink } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { app } from 'electron';
import { logger } from '../services/logger';
import { subLog } from '../services/subtitleDebugLog';
import {
  classifySubtitleCodec,
  deriveSubtitleState,
  type SubtitleState,
} from '../../shared/subtitleSupport';

export interface EmbeddedSubInfo {
  streamIndex: number;
  codec: string;
  language: string | null;
  title: string | null;
}

// PGS / DVD subtitles are bitmap formats — we can't render those without OCR.
// For text-based codecs we either keep them as ASS (rendered by libass via
// JASSUB in the renderer) or convert them to WebVTT (browser-native). Codec →
// renderable-format mapping lives in shared/subtitleSupport so the series-view
// probe and the play-time extract agree on what counts as renderable.
function targetFormat(codec: string): 'ass' | 'vtt' | null {
  const k = classifySubtitleCodec(codec);
  if (k === 'ass') return 'ass';
  if (k === 'vtt') return 'vtt';
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

// Probe EVERY subtitle stream in the container (text AND bitmap). The renderer-
// facing listEmbedded filters this down to renderable text; evaluateAvailability
// needs the unfiltered list so it can tell "bitmap-only" apart from "no subs".
async function probeAllStreams(videoPath: string): Promise<EmbeddedSubInfo[]> {
  let json: string;
  try {
    json = await runFfprobe([
      '-v', 'error',
      '-select_streams', 's',
      '-show_entries', 'stream=index,codec_name:stream_tags=language,title',
      '-of', 'json',
      videoPath,
    ]);
  } catch (err) {
    subLog('main/probe', 'ffprobe failed', { file: videoPath, error: err });
    throw err;
  }
  const parsed = JSON.parse(json) as {
    streams?: Array<{ index: number; codec_name?: string; tags?: { language?: string; title?: string } }>;
  };
  const streams = (parsed.streams || []).map((s) => ({
    streamIndex: s.index,
    codec: s.codec_name ?? '',
    language: s.tags?.language ?? null,
    title: s.tags?.title ?? null,
  }));
  subLog('main/probe', `found ${streams.length} subtitle stream(s)`, { file: videoPath, streams });
  return streams;
}

const subtitleHandler = {
  async listEmbedded(videoPath: string): Promise<EmbeddedSubInfo[]> {
    if (!existsSync(videoPath)) return [];
    try {
      const all = await probeAllStreams(videoPath);
      // Filter out subtitle codecs we can't render (bitmap PGS/DVD).
      const text = all.filter((s) => targetFormat(s.codec) !== null);
      if (all.length !== text.length) {
        logger.info('metadata', `Skipping ${all.length - text.length} non-text subtitle stream(s) (bitmap)`, { file: videoPath });
      }
      subLog('main/list-embedded', 'renderable text streams', { file: videoPath, renderable: text.length, bitmapSkipped: all.length - text.length });
      return text;
    } catch (err) {
      logger.warn('metadata', `Failed to list embedded subtitles: ${(err as Error).message}`, { file: videoPath });
      subLog('main/list-embedded', 'listing failed', { file: videoPath, error: err });
      return [];
    }
  },

  /**
   * Cheap (probe-only, no extraction) verdict on whether this file's subtitles
   * will actually display, for the series-view marker. 'unsupported' means the
   * container HAS subtitle streams but none are renderable (bitmap PGS/DVD that
   * mpv shows but we can't, or an unknown codec); 'ok' means a renderable text
   * stream (or external sidecar) exists; null means no subtitle content at all.
   * Returns null on probe failure so a transient ffprobe error never paints a
   * false marker. Does NOT attempt extraction, so it can't return 'failed' —
   * that's the play-time path's job.
   */
  async evaluateAvailability(videoPath: string, hasSidecar: boolean): Promise<SubtitleState | null> {
    if (hasSidecar) {
      subLog('main/evaluate', 'verdict: ok (sidecar present)', { file: videoPath, hasSidecar });
      return 'ok';
    }
    if (!existsSync(videoPath)) return null;
    try {
      const all = await probeAllStreams(videoPath);
      let renderableCount = 0;
      let nonRenderableCount = 0;
      for (const s of all) {
        if (targetFormat(s.codec) !== null) renderableCount++;
        else nonRenderableCount++;
      }
      const state = deriveSubtitleState({ hasSidecar: false, renderableCount, nonRenderableCount });
      subLog('main/evaluate', `verdict: ${state ?? 'none'}`, { file: videoPath, hasSidecar, renderableCount, nonRenderableCount });
      return state;
    } catch (err) {
      logger.warn('metadata', `Failed to evaluate subtitle availability: ${(err as Error).message}`, { file: videoPath });
      subLog('main/evaluate', 'verdict: null (probe failed)', { file: videoPath, hasSidecar, error: err });
      return null;
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
    subLog('main/extract', 'extract requested', { file: videoPath, streamIndex, codec, targetFormat: fmt });
    if (!fmt) {
      subLog('main/extract', 'bail: unrenderable codec', { file: videoPath, streamIndex, codec });
      return null;
    }
    let out: string;
    try {
      const stats = await stat(videoPath);
      const dir = await ensureCacheDir();
      out = join(dir, `${cacheKeyHash(videoPath, stats.mtimeMs, streamIndex)}.${fmt}`);
    } catch (err) {
      logger.warn('metadata', `Failed to resolve subtitle cache path for stream ${streamIndex}: ${(err as Error).message}`, { file: videoPath });
      subLog('main/extract', 'bail: cache path resolve failed', { file: videoPath, streamIndex, error: err });
      return null;
    }
    if (existsSync(out)) {
      // Self-heal: extractions written by the pre-atomic code could be
      // interrupted mid-write, leaving a truncated/empty file that a bare
      // existsSync would trust forever ("this episode never loads subs").
      // An empty file is never a valid extract; drop it and re-extract.
      try {
        const cached = await stat(out);
        if (cached.size > 0) {
          subLog('main/extract', 'cache hit', { file: videoPath, streamIndex, out, bytes: cached.size });
          return { path: out, format: fmt };
        }
        await unlink(out);
        logger.warn('metadata', `Discarded empty cached subtitle extract for stream ${streamIndex}; re-extracting`, { file: videoPath });
        subLog('main/extract', 'discarded empty cached extract, re-extracting', { file: videoPath, streamIndex, out });
      } catch {
        // stat/unlink raced with something else; fall through and re-extract.
      }
    }
    // Coalesce a concurrent extract of the same output (prewarm vs play-time).
    const pending = inFlightExtract.get(out);
    if (pending) {
      subLog('main/extract', 'joining in-flight extraction', { file: videoPath, streamIndex, out });
      return pending;
    }
    const startedAt = Date.now();
    const job = (async () => {
      // ASS extraction keeps the original styling/positioning; everything else
      // converts to WebVTT. `muxer` is BOTH the codec (-c:s) and the forced
      // output format (-f) — see below.
      const muxer: 'ass' | 'webvtt' = fmt === 'ass' ? 'ass' : 'webvtt';
      // Write to a PID-suffixed temp then atomic-rename, so existsSync(out) is
      // only ever true for a COMPLETE file. ffmpeg writes its output in place
      // and incrementally, so without this a cache-hit check (here or in a
      // racing reader) could hand back a half-written .ass — much more likely
      // now that a background prewarm can be mid-extract while the user plays.
      const tmp = `${out}.tmp.${process.pid}.${streamIndex}`;
      // Two attempts. A first failure is often transient (busy disk, ffmpeg
      // losing a race with the file); a deterministic failure just fails twice
      // cheaply and falls through to the marker.
      const ATTEMPTS = 2;
      let lastErr: unknown;
      for (let attempt = 1; attempt <= ATTEMPTS; attempt++) {
        try {
          await runFfmpeg([
            '-y',
            '-i', videoPath,
            '-map', `0:${streamIndex}`,
            '-c:s', muxer,
            // Force the output muxer. `tmp` ends in a numeric suffix
            // (.tmp.<pid>.<streamIndex>), so ffmpeg can't infer the format from
            // the file extension and aborts with "Unable to choose an output
            // format" — which silently broke EVERY embedded extraction. -f makes
            // the muxer explicit and independent of the temp filename.
            '-f', muxer,
            tmp,
          ]);
          // ffmpeg exited 0 but validate anyway: an empty output must count as
          // a failed attempt, never get renamed into the cache.
          const produced = await stat(tmp);
          if (produced.size === 0) throw new Error('extraction produced an empty file');
          await rename(tmp, out);
          logger.info('metadata', `Extracted embedded subtitle stream ${streamIndex} (${fmt}) → cache`, { file: videoPath });
          subLog('main/extract', 'extracted ok', { file: videoPath, streamIndex, format: fmt, ms: Date.now() - startedAt, bytes: produced.size, out });
          return { path: out, format: fmt };
        } catch (err) {
          lastErr = err;
          subLog('main/extract', `ffmpeg attempt ${attempt}/${ATTEMPTS} failed`, { file: videoPath, streamIndex, error: err });
          await unlink(tmp).catch(() => { /* tmp may not exist */ });
        }
      }
      logger.warn('metadata', `Failed to extract embedded subtitle stream ${streamIndex} after ${ATTEMPTS} attempts: ${(lastErr as Error).message}`, { file: videoPath });
      subLog('main/extract', `giving up after ${ATTEMPTS} attempts`, { file: videoPath, streamIndex, ms: Date.now() - startedAt, error: lastErr });
      return null;
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
    if (!videoPath) return;
    if (prewarmSeen.has(videoPath)) {
      subLog('main/prewarm', 'skip: already queued this session', { file: videoPath });
      return;
    }
    prewarmSeen.add(videoPath);
    subLog('main/prewarm', 'queued', { file: videoPath });
    prewarmChain = prewarmChain.then(async () => {
      const startedAt = Date.now();
      try {
        if (!existsSync(videoPath)) {
          subLog('main/prewarm', 'abort: file missing', { file: videoPath });
          return;
        }
        const streams = await subtitleHandler.listEmbedded(videoPath);
        // Warm ONLY the track that will actually display. The player picks
        // the first English stream (else the first stream) and extracts just
        // that at play time; other languages extract lazily on selection.
        // MultiSub releases carry ~10 languages and each extraction demuxes
        // the whole file, so warming them all was 10x wasted IO per episode
        // (and read as "8 tries for one episode" in the activity log).
        const lang = (s: { language: string | null }) => (s.language ?? '').toUpperCase();
        const engMatch = streams.find((s) => lang(s) === 'ENG' || lang(s) === 'EN');
        const target = engMatch ?? streams[0];
        if (target) {
          subLog('main/prewarm', 'target stream chosen', {
            file: videoPath,
            streamIndex: target.streamIndex,
            language: target.language,
            reason: engMatch ? 'english match' : 'first stream',
          });
          await subtitleHandler.extractEmbedded(videoPath, target.streamIndex, target.codec);
          subLog('main/prewarm', 'done', { file: videoPath, ms: Date.now() - startedAt });
        } else {
          subLog('main/prewarm', 'no renderable embedded streams', { file: videoPath });
        }
      } catch (err) {
        /* prewarm is best-effort; play-time extraction still works */
        subLog('main/prewarm', 'failed (swallowed, play-time extract still works)', { file: videoPath, error: err });
      }
    });
  },
};

export default subtitleHandler;
