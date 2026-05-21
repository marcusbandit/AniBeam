import type { EpisodeMetadata, SeriesMetadata } from "../hooks/useMetadata";

export interface AiringShow {
  seriesId: string;
  data: SeriesMetadata;
  latestEpisode: EpisodeMetadata | null;
  latestAirDate: Date;
}

/**
 * Normalize a status string from any source (AniList: RELEASING / FINISHED;
 * MAL: "Currently Airing" / "Finished Airing") into one of:
 * "releasing" | "finished" | "upcoming" | "cancelled" | "hiatus" | "" (unknown).
 */
export function normalizeStatus(status?: string | null): string {
  if (!status) return "";
  const s = status.trim().toLowerCase().replace(/[\s-]+/g, "_");
  if (s === "releasing" || s === "currently_airing" || s === "airing" || s === "ongoing") {
    return "releasing";
  }
  if (s === "finished" || s === "finished_airing" || s === "ended" || s === "completed") {
    return "finished";
  }
  if (s === "not_yet_released" || s === "not_yet_aired" || s === "upcoming" || s === "tba") {
    return "upcoming";
  }
  if (s === "cancelled" || s === "canceled") return "cancelled";
  if (s === "hiatus" || s === "on_hiatus") return "hiatus";
  return s;
}

function isReleasing(data: SeriesMetadata): boolean {
  return normalizeStatus(data.status) === "releasing";
}

function hasFiles(data: SeriesMetadata): boolean {
  return (data.fileEpisodes?.length ?? 0) > 0;
}

function parseDate(value: string | null | undefined): Date | null {
  if (!value) return null;
  const d = new Date(value);
  return isNaN(d.getTime()) ? null : d;
}

/**
 * Returns the most recent already-aired episode for a series, or null
 * if no episode has airDate metadata or all are in the future.
 */
export function getLatestAiredEpisode(series: SeriesMetadata): EpisodeMetadata | null {
  const now = Date.now();
  let latest: { ep: EpisodeMetadata; ts: number } | null = null;
  for (const ep of series.episodes ?? []) {
    const d = parseDate(ep.airDate);
    if (!d) continue;
    const ts = d.getTime();
    if (ts > now) continue;
    if (!latest || ts > latest.ts) latest = { ep, ts };
  }
  return latest?.ep ?? null;
}

/**
 * Highest episode number whose airDate is in the past. Works on the lighter
 * `LibraryEpisodeAirDate` shape used by the main library walk so HomePage
 * and FeedPage can call it directly. Returns null if nothing has aired yet.
 */
export function getLatestAiredEpisodeNumber(
  episodes: ReadonlyArray<{ episodeNumber: number; airDate: string | null }> | null | undefined,
): number | null {
  if (!episodes) return null;
  const now = Date.now();
  let best: number | null = null;
  for (const e of episodes) {
    if (!e.airDate) continue;
    const t = Date.parse(e.airDate);
    if (!Number.isFinite(t) || t > now) continue;
    if (best == null || e.episodeNumber > best) best = e.episodeNumber;
  }
  return best;
}

export type WatchProgressState = "watched" | "caught-up" | "behind";

/**
 * Classify watched progress against released/total counts:
 *  - "watched": user has watched every episode (only when totalEpisodes is known).
 *  - "behind": at least one already-aired episode is unwatched.
 *  - "caught-up": fully up-to-date with what's released, but more is still
 *    coming or total is unknown.
 */
export function classifyWatchProgress(args: {
  watched: number;
  totalEpisodes: number | null | undefined;
  latestAiredEpisode: number | null | undefined;
}): WatchProgressState {
  const { watched, totalEpisodes, latestAiredEpisode } = args;
  if (totalEpisodes != null && totalEpisodes > 0 && watched >= totalEpisodes) {
    return "watched";
  }
  if (latestAiredEpisode != null && watched < latestAiredEpisode) {
    return "behind";
  }
  return "caught-up";
}

/**
 * Single source of truth for the watched-count badge label. Use this
 * everywhere a card renders a watched badge so HomePage / FeedPage /
 * ShowCard all agree on the format.
 *
 * Output shape:
 *   - null            — no tracker entry for this series; show no badge
 *   - "Watched"       — series fully watched (only when totalEpisodes known)
 *   - "XX/YY"         — watched/total, watched zero-padded to total's width
 *   - "XX/YY+"        — total isn't published yet, but YY episodes have
 *                       aired so far. The "+" signals "and more coming".
 *                       Used for currently-airing shows where AniList
 *                       hasn't committed to a final episode count.
 *   - "XX/?"          — watched is known but neither total nor aired
 *                       count are available (rare — usually means we
 *                       haven't matched metadata yet).
 */
export function formatWatchedLabel(args: {
  watched: number | null;
  totalEpisodes: number | null | undefined;
  latestAiredEpisode?: number | null;
  state: WatchProgressState | null;
}): string | null {
  const { watched, totalEpisodes, latestAiredEpisode, state } = args;
  if (watched == null) return null;
  if (state === "watched") return "Watched";
  if (totalEpisodes != null && totalEpisodes > 0) {
    return `${String(watched).padStart(String(totalEpisodes).length, "0")}/${totalEpisodes}`;
  }
  if (latestAiredEpisode != null && latestAiredEpisode > 0) {
    const denom = Math.max(latestAiredEpisode, watched);
    return `${String(watched).padStart(String(denom).length, "0")}/${denom}+`;
  }
  return `${String(watched).padStart(2, "0")}/?`;
}

/**
 * Earliest episode with airDate > now. Used by the series detail chip and
 * the feed-card meta row to render a live countdown. Returns null when
 * nothing upcoming is known (finished shows, releasing shows whose
 * schedule isn't cached yet).
 */
export function findNextUpcomingEpisode(
  episodes: ReadonlyArray<{ episodeNumber: number; airDate: string | null }> | null | undefined,
  nowMs: number,
): { episodeNumber: number; airDateMs: number } | null {
  if (!episodes) return null;
  let best: { episodeNumber: number; airDateMs: number } | null = null;
  for (const e of episodes) {
    if (!e.airDate) continue;
    const t = Date.parse(e.airDate);
    if (!Number.isFinite(t) || t <= nowMs) continue;
    if (!best || t < best.airDateMs) best = { episodeNumber: e.episodeNumber, airDateMs: t };
  }
  return best;
}

/**
 * Compact countdown — minute granularity, no seconds. Use when the
 * countdown shares a tight row (e.g. the feed card meta line) and a
 * second-by-second twitch would just add visual noise.
 */
export function formatCountdownMinutes(diffMs: number): string {
  if (diffMs <= 0) return "now";
  const totalSec = Math.floor(diffMs / 1000);
  const d = Math.floor(totalSec / 86400);
  const h = Math.floor((totalSec % 86400) / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const pad = (n: number) => String(n).padStart(2, "0");
  if (d > 0) return `${d}d ${pad(h)}h ${pad(m)}m`;
  if (h > 0) return `${h}h ${pad(m)}m`;
  return `${m}m`;
}

/**
 * Live-countdown formatter: largest non-zero unit down to seconds. Seconds
 * are always included — callers using this expect a visible 1Hz tick. Use
 * a tabular-nums CSS rule on the enclosing element to keep widths stable.
 */
export function formatCountdown(diffMs: number): string {
  if (diffMs <= 0) return "now";
  const totalSec = Math.floor(diffMs / 1000);
  const d = Math.floor(totalSec / 86400);
  const h = Math.floor((totalSec % 86400) / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const s = totalSec % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  if (d > 0) return `${d}d ${pad(h)}h ${pad(m)}m ${pad(s)}s`;
  if (h > 0) return `${h}h ${pad(m)}m ${pad(s)}s`;
  if (m > 0) return `${m}m ${pad(s)}s`;
  return `${s}s`;
}

/**
 * Returns currently-airing on-disk shows, sorted by most recent aired
 * episode date descending. Falls back to startDate when no past airDate
 * is available so the show still appears.
 */
export function getAiringShows(metadata: Record<string, SeriesMetadata>): AiringShow[] {
  const out: AiringShow[] = [];
  Object.entries(metadata).forEach(([seriesId, data]) => {
    if (!isReleasing(data) || !hasFiles(data)) return;

    const latestEpisode = getLatestAiredEpisode(data);
    const sortDate = parseDate(latestEpisode?.airDate)
      ?? parseDate(data.startDate)
      ?? new Date(0);

    out.push({ seriesId, data, latestEpisode, latestAirDate: sortDate });
  });

  out.sort((a, b) => b.latestAirDate.getTime() - a.latestAirDate.getTime());
  return out;
}

/**
 * Human-friendly relative time: "today", "2 days ago", "in 1 day", "3 weeks ago".
 * Returns empty string for null input.
 */
export function formatRelativeDate(date: Date | string | null | undefined): string {
  const d = typeof date === "string" ? parseDate(date) : date instanceof Date ? date : null;
  if (!d) return "";

  const now = Date.now();
  const diffMs = now - d.getTime();
  const future = diffMs < 0;
  const absMs = Math.abs(diffMs);
  const minute = 60 * 1000;
  const hour = 60 * minute;
  const day = 24 * hour;
  const week = 7 * day;
  const month = 30 * day;
  const year = 365 * day;

  let value: number;
  let unit: string;

  if (absMs < hour) {
    value = Math.max(1, Math.round(absMs / minute));
    unit = value === 1 ? "minute" : "minutes";
  } else if (absMs < day) {
    value = Math.round(absMs / hour);
    unit = value === 1 ? "hour" : "hours";
  } else if (absMs < day * 1.5) {
    return future ? "tomorrow" : "yesterday";
  } else if (absMs < week) {
    value = Math.round(absMs / day);
    unit = "days";
  } else if (absMs < month) {
    value = Math.round(absMs / week);
    unit = value === 1 ? "week" : "weeks";
  } else if (absMs < year) {
    value = Math.round(absMs / month);
    unit = value === 1 ? "month" : "months";
  } else {
    value = Math.round(absMs / year);
    unit = value === 1 ? "year" : "years";
  }

  if (absMs < day && unit !== "minute" && unit !== "minutes" && unit !== "hour" && unit !== "hours") {
    return "today";
  }

  return future ? `in ${value} ${unit}` : `${value} ${unit} ago`;
}

/**
 * Format an episode code: "Special" for episode 0 / season 0 (Specials
 * convention from the file scanner), S01E04 when seasons are known, EP 4
 * otherwise.
 */
export function formatEpisodeCode(ep: EpisodeMetadata | null): string {
  if (!ep) return "";
  if (ep.episodeNumber === 0 || ep.seasonNumber === 0) return "Special";
  const epNum = Number.isInteger(ep.episodeNumber)
    ? String(ep.episodeNumber).padStart(2, "0")
    : ep.episodeNumber.toFixed(1);
  if (ep.seasonNumber !== null && ep.seasonNumber !== undefined) {
    const s = String(ep.seasonNumber).padStart(2, "0");
    return `S${s}E${epNum}`;
  }
  return `EP ${epNum}`;
}
