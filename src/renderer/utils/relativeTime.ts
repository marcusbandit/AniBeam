/**
 * Single relative-time module for the whole renderer. Replaces the divergent
 * per-page formatters (HomePage / FeedPage / WatchingPage) and takes over
 * display formatting from airingUtils.
 *
 * Three voices, one per context:
 *  - fmtShort:     card meta rows ("53m ago", "3d ago", "1y 2mo ago", "in 2d")
 *  - fmtCountdown: live next-episode countdowns ("3d 18h 19m")
 *  - fmtVerbose:   tooltips ("yesterday", "2 days ago", "in 1 week")
 */

const SEC = 1000;
const MIN = 60 * SEC;
const HOUR = 60 * MIN;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;
/** 30-day month / 12-month year: deliberately cheap maths, readable labels. */
const MONTH = 30 * DAY;
const YEAR = 365 * DAY;

const pad = (n: number) => String(n).padStart(2, "0");

/**
 * Compact relative timestamp, past and future. Buckets: "just now" under a
 * minute, then m / h / d, then months with rollover to years at 12
 * ("14mo" reads as "1y 2mo").
 */
export function fmtShort(tsMs: number, nowMs: number = Date.now()): string {
  const diff = nowMs - tsMs;
  const abs = Math.abs(diff);
  const future = diff < 0;
  if (abs < MIN) return "just now";
  let label: string;
  if (abs < HOUR) {
    label = `${Math.floor(abs / MIN)}m`;
  } else if (abs < DAY) {
    label = `${Math.floor(abs / HOUR)}h`;
  } else if (abs < MONTH) {
    label = `${Math.floor(abs / DAY)}d`;
  } else {
    const totalMo = Math.floor(abs / MONTH);
    const y = Math.floor(totalMo / 12);
    const mo = totalMo % 12;
    label = y > 0 ? (mo > 0 ? `${y}y ${mo}mo` : `${y}y`) : `${totalMo}mo`;
  }
  return future ? `in ${label}` : `${label} ago`;
}

/**
 * Minute-granularity countdown for live "next episode in" chips. Same output
 * shape as airingUtils.formatCountdownMinutes ("3d 18h 19m", lower units
 * zero-padded, "now" once the moment passes) but takes total minutes so
 * callers that already work in minutes don't round-trip through ms.
 */
export function fmtCountdown(totalMinutes: number): string {
  if (totalMinutes <= 0) return "now";
  const mins = Math.floor(totalMinutes);
  const d = Math.floor(mins / 1440);
  const h = Math.floor((mins % 1440) / 60);
  const m = mins % 60;
  if (d > 0) return `${d}d ${pad(h)}h ${pad(m)}m`;
  if (h > 0) return `${h}h ${pad(m)}m`;
  return `${m}m`;
}

/**
 * Human-friendly relative date for tooltips: "5 minutes ago", "yesterday",
 * "2 days ago", "in 1 week". Behavioural port of
 * airingUtils.formatRelativeDate operating on a ms timestamp, so
 * formatRelativeDate can delegate here without changing any rendered string.
 */
export function fmtVerbose(tsMs: number, nowMs: number = Date.now()): string {
  const diff = nowMs - tsMs;
  const future = diff < 0;
  const abs = Math.abs(diff);

  let value: number;
  let unit: string;

  if (abs < HOUR) {
    value = Math.max(1, Math.round(abs / MIN));
    unit = value === 1 ? "minute" : "minutes";
  } else if (abs < DAY) {
    value = Math.round(abs / HOUR);
    unit = value === 1 ? "hour" : "hours";
  } else if (abs < DAY * 1.5) {
    return future ? "tomorrow" : "yesterday";
  } else if (abs < WEEK) {
    value = Math.round(abs / DAY);
    unit = "days";
  } else if (abs < MONTH) {
    value = Math.round(abs / WEEK);
    unit = value === 1 ? "week" : "weeks";
  } else if (abs < YEAR) {
    value = Math.round(abs / MONTH);
    unit = value === 1 ? "month" : "months";
  } else {
    value = Math.round(abs / YEAR);
    unit = value === 1 ? "year" : "years";
  }

  return future ? `in ${value} ${unit}` : `${value} ${unit} ago`;
}
