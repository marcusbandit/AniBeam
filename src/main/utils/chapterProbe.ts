// Reads MKV/MP4 chapter markers via ffprobe and decides whether any of
// them describe an opening or closing/credits sequence. Used to short-
// circuit the AniSkip community lookup — if the file already labels
// "Intro" / "Outro", that's authoritative and local.

import { spawn } from 'node:child_process';
import { logger } from '../services/logger';

export interface Chapter {
  start: number;
  end: number;
  title: string;
}

export interface ChapterSkipTimes {
  op?: { start: number; end: number };
  ed?: { start: number; end: number };
}

// Hard cap on a classified chapter's length. Some files have a single
// chapter spanning the entire episode but labelled "Opening" (rare, but
// it happens) — we don't want to skip the whole episode.
const MAX_CHAPTER_SECONDS = 300;

// Whole-word anchors so "Episode 1" or "Chapter 1" never accidentally
// match. `op` / `ed` are short tokens — require a non-letter boundary.
const OP_PATTERN = /^(intro|opening|prologue|op(?:\s*\d+)?)\b/i;
const ED_PATTERN = /^(outro|ending|closing|credits|end\s*credits|ed(?:\s*\d+)?)\b/i;

function ffprobeChapters(path: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const p = spawn('ffprobe', [
      '-v', 'error',
      '-show_chapters',
      '-of', 'json',
      path,
    ], { stdio: ['ignore', 'pipe', 'pipe'] });
    let out = '';
    let err = '';
    p.stdout.on('data', (d) => { out += d.toString(); });
    p.stderr.on('data', (d) => { err += d.toString(); });
    p.on('close', (code) => {
      if (code === 0) resolve(out);
      else reject(new Error(`ffprobe exit ${code}: ${err.slice(-200)}`));
    });
    p.on('error', reject);
  });
}

export async function probeChapters(filePath: string): Promise<Chapter[]> {
  try {
    const raw = await ffprobeChapters(filePath);
    const parsed = JSON.parse(raw) as {
      chapters?: Array<{
        start_time?: string;
        end_time?: string;
        tags?: { title?: string; TITLE?: string };
      }>;
    };
    const out: Chapter[] = [];
    for (const c of parsed.chapters ?? []) {
      const start = parseFloat(c.start_time ?? '');
      const end = parseFloat(c.end_time ?? '');
      const title = (c.tags?.title ?? c.tags?.TITLE ?? '').trim();
      if (!Number.isFinite(start) || !Number.isFinite(end) || end <= start) continue;
      out.push({ start, end, title });
    }
    return out;
  } catch (err) {
    logger.warn('metadata', `Chapter probe failed: ${(err as Error).message}`, { file: filePath });
    return [];
  }
}

export function classifyChapters(chapters: Chapter[]): ChapterSkipTimes {
  const times: ChapterSkipTimes = {};
  for (const c of chapters) {
    if (!c.title) continue;
    if (c.end - c.start > MAX_CHAPTER_SECONDS) continue;
    if (!times.op && OP_PATTERN.test(c.title)) {
      times.op = { start: c.start, end: c.end };
    } else if (!times.ed && ED_PATTERN.test(c.title)) {
      times.ed = { start: c.start, end: c.end };
    }
  }
  return times;
}
