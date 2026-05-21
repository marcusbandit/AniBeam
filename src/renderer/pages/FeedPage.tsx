import { useCallback, useEffect, useMemo, useState } from "react";
import { Activity } from "lucide-react";
import type { LibraryItem } from "../../types/electron";
import { findNextUpcomingEpisode } from "../utils/airingUtils";
import { useDebouncedCallback } from "../hooks/useDebouncedCallback";
import ShowCard from "../components/ShowCard";

interface FeedEntry {
  item: LibraryItem;
  when: number;            // Unix seconds
  episodeNumber: number | null;  // latest episode this entry refers to
  source: "aired" | "downloaded";
}

// Mirrors fmt_relative_time from the C version (src/ui.c:862). Buckets:
// 60s → 1h → 1d → 30d → "months" → "years". 30-day "month" / 12-month
// "year" is intentional — keeps the maths cheap and the labels readable.
function fmtRelativeTime(unixSec: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - unixSec;
  const abs = Math.abs(diff);
  const future = diff < 0;
  if (abs < 60) return "just now";
  if (abs < 3600) {
    const m = Math.floor(abs / 60);
    return future ? `in ${m}m` : `${m}m ago`;
  }
  if (abs < 86400) {
    const h = Math.floor(abs / 3600);
    return future ? `in ${h}h` : `${h}h ago`;
  }
  if (abs < 86400 * 30) {
    const d = Math.floor(abs / 86400);
    return future ? `in ${d}d` : `${d}d ago`;
  }
  const totalMo = Math.floor(abs / (86400 * 30));
  const y = Math.floor(totalMo / 12);
  const mo = totalMo % 12;
  const label = y > 0
    ? (mo > 0 ? `${y}y ${mo}mo` : `${y}y`)
    : `${totalMo}mo`;
  return future ? `in ${label}` : `${label} ago`;
}

// One entry per show. Same logic as the C reference (src/ui.c:955):
//   - If the show has any episode whose airDate <= now, pick the latest such.
//   - Otherwise fall back to the file with the largest mtime.
// Future episodes are ignored. Entries with no information at all are
// dropped (no episodes, no files, or mtime=0).
function buildEntries(items: LibraryItem[]): FeedEntry[] {
  const nowSec = Math.floor(Date.now() / 1000);
  const out: FeedEntry[] = [];
  for (const item of items) {
    if (item.files.length === 0) continue;

    // 1. Try latest aired episode that we ALSO have on disk.
    const onDiskEps = new Set(item.files.map((f) => f.episodeNumber));
    let bestAired: { ts: number; ep: number } | null = null;
    for (const e of item.episodes) {
      if (!e.airDate || !onDiskEps.has(e.episodeNumber)) continue;
      const t = Math.floor(Date.parse(e.airDate) / 1000);
      if (!Number.isFinite(t) || t > nowSec) continue;
      if (!bestAired || t > bestAired.ts) bestAired = { ts: t, ep: e.episodeNumber };
    }
    if (bestAired) {
      out.push({ item, when: bestAired.ts, episodeNumber: bestAired.ep, source: "aired" });
      continue;
    }

    // 2. Fallback: latest file mtime.
    let bestFile: { mtime: number; ep: number } | null = null;
    for (const f of item.files) {
      if (!f.mtime) continue;
      if (!bestFile || f.mtime > bestFile.mtime) {
        bestFile = { mtime: f.mtime, ep: f.episodeNumber };
      }
    }
    if (bestFile) {
      out.push({
        item,
        when: Math.floor(bestFile.mtime / 1000),
        episodeNumber: bestFile.ep,
        source: "downloaded",
      });
    }
  }
  out.sort((a, b) => b.when - a.when);
  return out;
}

function FeedPage() {
  const [items, setItems] = useState<LibraryItem[]>([]);
  const [initialLoading, setInitialLoading] = useState(true);

  const reload = useCallback(async () => {
    try {
      const data = await window.electronAPI.libraryWalk();
      setItems(Array.isArray(data) ? data : []);
    } catch (err) {
      console.error("library:walk failed", err);
      setItems([]);
    } finally {
      setInitialLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  // Debounced — see HomePage for the reasoning. New shows fire bursts of
  // metadata change pings; we want one walk per burst, not 2N+1.
  const debouncedReload = useDebouncedCallback(() => { void reload(); }, 250);
  useEffect(() => {
    const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
      debouncedReload();
    });
    return () => unsubscribe?.();
  }, [debouncedReload]);

  const entries = useMemo(() => buildEntries(items), [items]);

  // Shared coarse tick for per-card countdowns. We render minute-
  // granularity here, so a 30s interval gives at-worst 30s display lag
  // without spinning a 1Hz timer in the background. Only mounted when at
  // least one entry actually has an upcoming episode.
  const [nowMs, setNowMs] = useState<number>(() => Date.now());
  const hasAnyUpcoming = useMemo(
    () => entries.some(({ item }) => findNextUpcomingEpisode(item.episodes, Date.now()) != null),
    [entries],
  );
  useEffect(() => {
    if (!hasAnyUpcoming) return;
    const id = window.setInterval(() => setNowMs(Date.now()), 30_000);
    return () => window.clearInterval(id);
  }, [hasAnyUpcoming]);

  if (initialLoading) {
    return (
      <div className="page">
        <div className="loading">Loading feed…</div>
      </div>
    );
  }

  return (
    <div className="page feed-page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Feed</h1>
          <p className="page-sub">Your library, ordered by latest episode release.</p>
        </div>
      </div>

      {entries.length === 0 ? (
        <div className="empty">
          <div className="empty-icon"><Activity size={48} /></div>
          <div className="empty-title">Nothing yet</div>
          <div className="empty-text">Add a folder in Settings to get started.</div>
        </div>
      ) : (
        <div className="show-grid">
          {entries.map(({ item, when, episodeNumber, source }) => (
            <ShowCard
              key={item.id}
              item={item}
              episodeBadgeNumber={episodeNumber}
              metaLeftText={fmtRelativeTime(when)}
              metaLeftTitle={source === "aired" ? "Episode aired" : "File downloaded"}
              nowMs={nowMs}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default FeedPage;
