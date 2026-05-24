import { createContext, useContext, useEffect, useState, useCallback, useMemo, type ReactNode } from "react";
import type { ViewHistoryEntry } from "../../types/electron";

interface Ctx {
  /**
   * ms-since-epoch the user last watched this series (or null when there
   * is no recorded session yet). The HomePage "Last viewed" sort uses
   * this directly; missing values get pushed to the end of the list.
   */
  getLastViewed: (seriesId: string) => number | null;
}

const ViewHistoryContext = createContext<Ctx | null>(null);

/**
 * Subscribes to the main-process view-history store and keeps a local copy
 * in renderer state so the sort comparator stays synchronous. Updates are
 * pushed via `onViewHistoryChanged` from main; we never poll.
 */
export function ViewHistoryProvider({ children }: { children: ReactNode }) {
  const [history, setHistory] = useState<Record<string, ViewHistoryEntry>>({});

  const refresh = useCallback(async () => {
    try {
      const api = window.electronAPI;
      if (!api?.getViewHistory) return;
      const next = await api.getViewHistory();
      setHistory(next ?? {});
    } catch (err) {
      console.warn("[view-history] getViewHistory failed:", err);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const unsub = window.electronAPI.onViewHistoryChanged?.(() => {
      void refresh();
    });
    return () => unsub?.();
  }, [refresh]);

  const getLastViewed = useCallback<Ctx["getLastViewed"]>(
    (seriesId) => history[seriesId]?.lastViewedAt ?? null,
    [history],
  );

  const value = useMemo<Ctx>(() => ({ getLastViewed }), [getLastViewed]);

  return (
    <ViewHistoryContext.Provider value={value}>
      {children}
    </ViewHistoryContext.Provider>
  );
}

export function useViewHistory(): Ctx {
  const ctx = useContext(ViewHistoryContext);
  if (!ctx) throw new Error("useViewHistory must be used inside ViewHistoryProvider");
  return ctx;
}
