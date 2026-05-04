// Shared playback-resume helpers. The map lives in localStorage under a single
// key so reads/writes are O(1) and one parse covers every episode card.

const PROGRESS_KEY = 'video-progress-v1';
export const RESUME_HEAD_SKIP = 5;   // < 5s in: ignore, not worth resuming
export const RESUME_TAIL_SKIP = 30;  // within 30s of end: treat as finished

export type ProgressEntry = { t: number; d: number; updated: number };
export type ProgressMap = Record<string, ProgressEntry>;

export function progressId(seriesId: string, episodeNumber: string | number): string {
  return `${seriesId}::${episodeNumber}`;
}

export function readProgress(): ProgressMap {
  try {
    const raw = localStorage.getItem(PROGRESS_KEY);
    return raw ? (JSON.parse(raw) as ProgressMap) : {};
  } catch { return {}; }
}

export function writeProgress(map: ProgressMap): void {
  try { localStorage.setItem(PROGRESS_KEY, JSON.stringify(map)); } catch { /* ignore */ }
}

// Fraction in [0, 1]. Returns 0 when no entry exists (never started OR finished
// — we delete entries on completion, so finished episodes show no bar).
export function getProgressFraction(
  map: ProgressMap,
  seriesId: string,
  episodeNumber: string | number,
): number {
  const entry = map[progressId(seriesId, episodeNumber)];
  if (!entry || entry.d <= 0) return 0;
  return Math.max(0, Math.min(1, entry.t / entry.d));
}
