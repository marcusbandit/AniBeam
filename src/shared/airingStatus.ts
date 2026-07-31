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
