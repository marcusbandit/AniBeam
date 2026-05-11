// Shared codec / encoder probe utilities. Used by:
//   - transcodeCacheHandler: decides whether a file needs pre-transcoding
//     and picks the encoder to use.
//   - transcodeHandler (HLS, disabled): same.
//
// Kept separate so the same definitions of "what codec is browser-safe"
// and "which encoder is available" don't drift between callers.

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { logger } from '../services/logger';

// What Chromium's <video> element can demux+decode through MSE / native
// playback. h264 + aac is the universal baseline. Everything else gets
// transcoded.
const BROWSER_VIDEO = new Set(['h264', 'avc1', 'vp8', 'vp9', 'av1']);
const BROWSER_AUDIO = new Set(['aac', 'mp3', 'mpeg', 'opus', 'vorbis', 'flac']);

export interface CodecProbe {
  duration: number;
  vCodec: string;
  aCodec: string;
  width: number;
  height: number;
}

export type EncoderKind = 'vaapi' | 'nvenc' | 'libx264';

function ffprobeJson(path: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const p = spawn('ffprobe', [
      '-v', 'error',
      '-show_streams', '-show_format',
      '-of', 'json',
      path,
    ], { stdio: ['ignore', 'pipe', 'pipe'] });
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

export async function probeCodecs(filePath: string): Promise<CodecProbe | null> {
  try {
    const raw = await ffprobeJson(filePath);
    const parsed = JSON.parse(raw) as {
      streams?: Array<{ codec_type?: string; codec_name?: string; width?: number; height?: number }>;
      format?: { duration?: string | number };
    };
    const streams = parsed.streams ?? [];
    const v = streams.find((s) => s.codec_type === 'video');
    const a = streams.find((s) => s.codec_type === 'audio');
    const dur = parsed.format?.duration;
    const duration = typeof dur === 'string' ? parseFloat(dur) : Number(dur ?? 0);
    if (!v || !Number.isFinite(duration) || duration <= 0) return null;
    return {
      duration,
      vCodec: (v.codec_name ?? '').toLowerCase(),
      aCodec: (a?.codec_name ?? '').toLowerCase(),
      width: v.width ?? 0,
      height: v.height ?? 0,
    };
  } catch (err) {
    logger.warn('system', `Codec probe failed: ${(err as Error).message}`, { file: filePath });
    return null;
  }
}

export function needsTranscode(p: CodecProbe): boolean {
  if (!BROWSER_VIDEO.has(p.vCodec)) return true;
  if (p.aCodec && !BROWSER_AUDIO.has(p.aCodec)) return true;
  return false;
}

// Probes which hardware encoder actually works on this machine. ffmpeg
// can list encoders by name without telling you whether they'll succeed
// against the local GPU/driver, so we exercise each candidate against a
// tiny synthetic clip. First one that exits 0 wins. Result is cached
// for the app's lifetime — encoder availability doesn't change at
// runtime.
//
// 256x256 frame: small enough to encode in <100ms, large enough that
// NVENC accepts it (the API rejects sub-145px dimensions).
let cachedKind: EncoderKind | null = null;
let probeInFlight: Promise<EncoderKind> | null = null;

async function tryEncoder(args: string[]): Promise<boolean> {
  return new Promise((resolve) => {
    const p = spawn('ffmpeg', args, { stdio: ['ignore', 'ignore', 'ignore'] });
    p.on('close', (code) => resolve(code === 0));
    p.on('error', () => resolve(false));
  });
}

async function detectEncoder(): Promise<EncoderKind> {
  if (existsSync('/dev/dri/renderD128')) {
    const ok = await tryEncoder([
      '-v', 'error',
      '-hwaccel', 'vaapi', '-vaapi_device', '/dev/dri/renderD128',
      '-f', 'lavfi', '-i', 'color=c=black:s=256x256:d=0.1',
      '-vf', 'format=nv12,hwupload',
      '-c:v', 'h264_vaapi',
      '-f', 'null', '-',
    ]);
    if (ok) return 'vaapi';
  }
  const nvencOk = await tryEncoder([
    '-v', 'error',
    '-f', 'lavfi', '-i', 'color=c=black:s=256x256:d=0.1',
    '-c:v', 'h264_nvenc', '-preset', 'p1',
    '-f', 'null', '-',
  ]);
  if (nvencOk) return 'nvenc';
  return 'libx264';
}

export async function ensureEncoder(): Promise<EncoderKind> {
  if (cachedKind) return cachedKind;
  if (probeInFlight) return probeInFlight;
  probeInFlight = detectEncoder().then((kind) => {
    cachedKind = kind;
    probeInFlight = null;
    logger.info('system', `Transcode encoder: ${kind}`);
    return kind;
  });
  return probeInFlight;
}
