import { createContext, useContext, useEffect, useState, useCallback, useMemo, type ReactNode } from "react";
import type {
  TrackerProgressSnapshot,
  TrackerProvider,
  TrackerListStatus,
  TrackerProgressEntry,
} from "../../main/preload";

interface SeriesIds {
  anilistId?: number;
  malId?: number | null;
}

interface Ctx {
  snapshot: TrackerProgressSnapshot | null;
  /**
   * Resolve the best-known watched-episode count for a series. Tries the
   * user's main provider first, falls back to the other one when the
   * primary has no entry. Returns null when neither side has progress.
   */
  getWatched: (ids: SeriesIds) => number | null;
  /**
   * Same provider-fallback logic as getWatched, but returns the canonical
   * list status (`watching`/`planning`/`completed`/`paused`/`dropped`/
   * `repeating`). Returns null when no entry exists for either provider,
   * or when the entry exists without a status (legacy v1 cache rows).
   */
  getListStatus: (ids: SeriesIds) => TrackerListStatus | null;
  /**
   * The user's own score on a 0–10 scale (AniList normalised via
   * POINT_10_DECIMAL, MAL native). Returns null when the user hasn't rated
   * the series on either provider.
   */
  getUserScore: (ids: SeriesIds) => number | null;
  /**
   * Number of completed rewatches (AniList `repeat`, MAL `num_times_rewatched`).
   * Returns null when no tracker entry exists or the user has never recorded
   * a rewatch. The hero only renders the chip when this is > 0.
   */
  getRewatchCount: (ids: SeriesIds) => number | null;
  setMainProvider: (provider: TrackerProvider) => Promise<void>;
}

const TrackerProgressContext = createContext<Ctx | null>(null);

export function TrackerProgressProvider({ children }: { children: ReactNode }) {
  const [snapshot, setSnapshot] = useState<TrackerProgressSnapshot | null>(null);

  const refresh = useCallback(async () => {
    try {
      const api = window.electronAPI;
      if (!api?.trackerGetProgress) {
        console.warn("[tracker-progress] trackerGetProgress missing on electronAPI — preload likely stale");
        return;
      }
      const snap = await api.trackerGetProgress();
      console.log("[tracker-progress] snapshot loaded", {
        main: snap.mainProvider,
        anilistEntries: Object.keys(snap.anilist ?? {}).length,
        malEntries: Object.keys(snap.mal ?? {}).length,
      });
      setSnapshot(snap);
    } catch (err) {
      console.error("[tracker-progress] trackerGetProgress failed:", err);
    }
  }, []);

  useEffect(() => {
    console.log("[tracker-progress] provider mounted");
    void refresh();
    // Defer the live API refresh so it can never block first paint. The
    // disk-cached snapshot from refresh() above is already enough to render
    // watched counts; this just keeps it fresh.
    const handle = window.setTimeout(() => {
      const api = window.electronAPI;
      if (!api?.trackerRefreshProgress) return;
      void api.trackerRefreshProgress().catch((err: unknown) => {
        console.warn("[tracker-progress] refresh-progress IPC failed:", err);
      });
    }, 0);
    const unsub = window.electronAPI.onTrackerProgressChanged?.(() => {
      void refresh();
    });
    return () => {
      window.clearTimeout(handle);
      unsub?.();
    };
  }, [refresh]);

  // Look up a tracker entry using the same main-provider-then-fallback
  // logic both getWatched and getListStatus need. Pulled out so the two
  // public helpers stay one-liners and can't drift apart.
  const lookupEntry = useCallback(
    ({ anilistId, malId }: SeriesIds): TrackerProgressEntry | null => {
      if (!snapshot) return null;
      const main = snapshot.mainProvider;
      const primary = main === "anilist" ? anilistId : malId;
      const secondary = main === "anilist" ? malId : anilistId;
      const primaryMap = main === "anilist" ? snapshot.anilist : snapshot.mal;
      const secondaryMap = main === "anilist" ? snapshot.mal : snapshot.anilist;
      if (typeof primary === "number" && primary in primaryMap) return primaryMap[primary];
      if (typeof secondary === "number" && secondary != null && secondary in secondaryMap) return secondaryMap[secondary];
      return null;
    },
    [snapshot],
  );

  const getWatched = useCallback<Ctx["getWatched"]>(
    (ids) => lookupEntry(ids)?.progress ?? null,
    [lookupEntry],
  );

  const getListStatus = useCallback<Ctx["getListStatus"]>(
    (ids) => lookupEntry(ids)?.status ?? null,
    [lookupEntry],
  );

  const getUserScore = useCallback<Ctx["getUserScore"]>(
    (ids) => lookupEntry(ids)?.score ?? null,
    [lookupEntry],
  );

  const getRewatchCount = useCallback<Ctx["getRewatchCount"]>(
    (ids) => lookupEntry(ids)?.rewatch ?? null,
    [lookupEntry],
  );

  const setMainProvider = useCallback(async (provider: TrackerProvider) => {
    await window.electronAPI.trackerSetMainProvider(provider);
    await refresh();
  }, [refresh]);

  const value = useMemo<Ctx>(
    () => ({ snapshot, getWatched, getListStatus, getUserScore, getRewatchCount, setMainProvider }),
    [snapshot, getWatched, getListStatus, getUserScore, getRewatchCount, setMainProvider],
  );

  return (
    <TrackerProgressContext.Provider value={value}>
      {children}
    </TrackerProgressContext.Provider>
  );
}

export function useTrackerProgress(): Ctx {
  const ctx = useContext(TrackerProgressContext);
  if (!ctx) throw new Error("useTrackerProgress must be used inside TrackerProgressProvider");
  return ctx;
}
