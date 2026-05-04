import { spawn } from 'node:child_process';
import { stat, mkdir } from 'node:fs/promises';
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
    try {
      const stats = await stat(videoPath);
      const dir = await ensureCacheDir();
      const filename = `${cacheKeyHash(videoPath, stats.mtimeMs, streamIndex)}.${fmt}`;
      const out = join(dir, filename);
      if (existsSync(out)) return { path: out, format: fmt };
      // ASS extraction: -c:s ass keeps the original styling/positioning.
      // VTT extraction: -c:s webvtt converts SRT/MOV_TEXT to WebVTT.
      await runFfmpeg([
        '-y',
        '-i', videoPath,
        '-map', `0:${streamIndex}`,
        '-c:s', fmt === 'ass' ? 'ass' : 'webvtt',
        out,
      ]);
      logger.info('metadata', `Extracted embedded subtitle stream ${streamIndex} (${fmt}) → cache`, { file: videoPath });
      return { path: out, format: fmt };
    } catch (err) {
      logger.warn('metadata', `Failed to extract embedded subtitle stream ${streamIndex}: ${(err as Error).message}`, { file: videoPath });
      return null;
    }
  },
};

export default subtitleHandler;
