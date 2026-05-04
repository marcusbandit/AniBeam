import type { EpisodeMetadata, SeriesMetadata } from "../hooks/useMetadata";

export interface AiringShow {
  seriesId: string;
  data: SeriesMetadata;
  latestEpisode: EpisodeMetadata | null;
  latestAirDate: Date;
}

/**
 * Normalize a status string from any source (AniList: RELEASING / FINISHED;
 * MAL: "Currently Airing" / "Finished Airing"; TVDB: similar) into one of:
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
