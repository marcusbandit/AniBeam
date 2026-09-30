// Release-status vocabulary, shared because both sides need it: the
// renderer to style status chips and pick airing shows, and the main
// process to decide which series are worth re-fetching an airing schedule
// for. Pure and Electron-free so verify scripts can import it directly.

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

/** Everything the watched rules read, already normalised by their own
 *  sources: the release status as stored, the tracker's list status in
 *  its canonical lowercase vocabulary, the watched episode count and the
 *  published total. */
export interface WatchedInputs {
  status?: string | null;
  listStatus?: string | null;
  watched?: number | null;
  totalEpisodes?: number | null;
}

/**
 * Finished with it: completed on the tracker, or the watched count has
 * reached the (known) total — a film with any watch at all counts, since
 * a film with no published total is one watch either way. This is the
 * rule the Progress sort pins by.
 */
export function isWatchedThrough(w: WatchedInputs): boolean {
  if (w.listStatus === "completed") return true;
  if (w.watched == null) return false;
  const total = w.totalEpisodes;
  if (total == null) return w.watched > 0;
  if (total <= 0) return false;
  return w.watched >= total;
}

/**
 * Done with it, full stop: watched-through AND nothing more coming. A
 * still-releasing series is never done — caught up is current, not
 * finished — so it stays in the grid and the airing rail however current
 * the user is. This is the rule that files a series into the Watched tab.
 */
export function isWatchedOut(w: WatchedInputs): boolean {
  return normalizeStatus(w.status) !== "releasing" && isWatchedThrough(w);
}

/**
 * Whether the provider's fresh status says something the stored one does
 * not. The comparison is on the normalised vocabulary, so spelling drift
 * between providers ("Currently Airing" vs RELEASING) is not a change.
 */
export function statusDiffers(stored?: string | null, fresh?: string | null): boolean {
  if (!fresh) return false;
  return normalizeStatus(stored) !== normalizeStatus(fresh);
}
