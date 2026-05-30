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

// Extras (openings/endings/PVs/specials) share an episodeNumber with a real
// episode — ED1, PV01 both classify to "episode 1" for sort purposes — so they
// can't be keyed by (seriesId, episodeNumber) without their resume position
// colliding with the real episode's. Key them by their unique file path
// instead. Both the series page (resume bar) and the player (save/restore)
// derive the key through this token so the two stay in lockstep.
export function extraProgressToken(filePath: string): string {
  return `x:${filePath}`;
}

export function getExtraProgressFraction(
  map: ProgressMap,
  seriesId: string,
  filePath: string,
): number {
  return getProgressFraction(map, seriesId, extraProgressToken(filePath));
}

// "Last completed episode" per series — tracked separately from the in-progress
// map (which gets cleared on completion) so the SeriesDetailPage can keep
// showing a "Next up" marker even after every episode is marked watched in
// the tracker. Survives rewatches: the value advances on each completion.

const LAST_EP_KEY = 'video-last-ep-v1';
export type LastEpisodeMap = Record<string, { ep: number; updated: number }>;

export function readLastEpisodeMap(): LastEpisodeMap {
  try {
    const raw = localStorage.getItem(LAST_EP_KEY);
    return raw ? (JSON.parse(raw) as LastEpisodeMap) : {};
  } catch { return {}; }
}

function writeLastEpisodeMap(map: LastEpisodeMap): void {
  try { localStorage.setItem(LAST_EP_KEY, JSON.stringify(map)); } catch { /* ignore */ }
}

export function getLastEpisode(seriesId: string): number | null {
  const entry = readLastEpisodeMap()[seriesId];
  return entry?.ep ?? null;
}

export function recordEpisodeCompleted(seriesId: string, episodeNumber: number): void {
  if (!Number.isFinite(episodeNumber) || episodeNumber <= 0) return;
  const map = readLastEpisodeMap();
  // Always overwrite — the latest completion is what "last watched" means,
  // even when the user is rewatching from earlier and the number goes down.
  map[seriesId] = { ep: episodeNumber, updated: Date.now() };
  writeLastEpisodeMap(map);
}
