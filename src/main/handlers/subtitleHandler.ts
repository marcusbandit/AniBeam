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

// PGS / DVD subtitles are bitmap formats — ffmpeg can't convert them to WebVTT
// without OCR. We list-but-skip them for now.
const TEXT_SUB_CODECS = new Set(['subrip', 'ass', 'ssa', 'webvtt', 'mov_text']);

function getCacheDir(): string {
  return join(app.getPath('userData'), 'subtitle-cache');
}

async function ensureCacheDir(): Promise<string> {
  const dir = getCacheDir();
  await mkdir(dir, { recursive: true });
  return dir;
}

function cacheKey(videoPath: string, mtimeMs: number, streamIndex: number): string {
  const h = createHash('md5').update(`${videoPath}:${mtimeMs}:${streamIndex}`).digest('hex');
  return `${h}.vtt`;
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
      // Filter out non-text subtitle codecs (PGS, DVD) — we can't convert them.
      const text = all.filter((s) => TEXT_SUB_CODECS.has(s.codec.toLowerCase()));
      if (all.length !== text.length) {
        logger.info('metadata', `Skipping ${all.length - text.length} non-text subtitle stream(s) (bitmap)`, { file: videoPath });
      }
      return text;
    } catch (err) {
      logger.warn('metadata', `Failed to list embedded subtitles: ${(err as Error).message}`, { file: videoPath });
      return [];
    }
  },

  async extractEmbedded(videoPath: string, streamIndex: number): Promise<string | null> {
    if (!existsSync(videoPath)) return null;
    try {
      const stats = await stat(videoPath);
      const dir = await ensureCacheDir();
      const filename = cacheKey(videoPath, stats.mtimeMs, streamIndex);
      const out = join(dir, filename);
      if (existsSync(out)) return out;
      // -map 0:<absolute-stream-index> picks that exact stream out of the input.
      // -c:s webvtt converts SRT/ASS/SSA → WebVTT (loses ASS styling, keeps text).
      await runFfmpeg([
        '-y',
        '-i', videoPath,
        '-map', `0:${streamIndex}`,
        '-c:s', 'webvtt',
        out,
      ]);
      logger.info('metadata', `Extracted embedded subtitle stream ${streamIndex} → cache`, { file: videoPath });
      return out;
    } catch (err) {
      logger.warn('metadata', `Failed to extract embedded subtitle stream ${streamIndex}: ${(err as Error).message}`, { file: videoPath });
      return null;
    }
  },
};

export default subtitleHandler;
