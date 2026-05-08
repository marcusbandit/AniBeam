import { useEffect, useMemo, useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Tv, ChevronLeft, ChevronRight } from "lucide-react";
import type { LibraryItem } from "../../types/electron";
import { normalizeStatus } from "../utils/airingUtils";
import { useTitleLanguage } from "../contexts/TitleLanguageContext";

const AIRING_PAGE_COLS = 5;
const AIRING_PAGE_ROWS = 2;
const AIRING_PAGE_SIZE = AIRING_PAGE_COLS * AIRING_PAGE_ROWS;

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
  const mo = Math.floor(abs / (86400 * 30));
  return future ? `in ${mo}mo` : `${mo}mo ago`;
}

// (sortKey, latestEpisode) for an airing show. Mirrors FeedPage's logic:
// latest aired-and-on-disk episode → fall back to highest file mtime.
function getAiringSortInfo(item: LibraryItem): { when: number; episode: number | null } | null {
  const nowSec = Math.floor(Date.now() / 1000);
  const onDiskEps = new Set(item.files.map((f) => f.episodeNumber));
  let bestAired: { ts: number; ep: number } | null = null;
  for (const e of item.episodes) {
    if (!e.airDate || !onDiskEps.has(e.episodeNumber)) continue;
    const t = Math.floor(Date.parse(e.airDate) / 1000);
    if (!Number.isFinite(t) || t > nowSec) continue;
    if (!bestAired || t > bestAired.ts) bestAired = { ts: t, ep: e.episodeNumber };
  }
  if (bestAired) return { when: bestAired.ts, episode: bestAired.ep };
  let bestFile: { mtime: number; ep: number } | null = null;
  for (const f of item.files) {
    if (!f.mtime) continue;
    if (!bestFile || f.mtime > bestFile.mtime) bestFile = { mtime: f.mtime, ep: f.episodeNumber };
  }
  if (bestFile) return { when: Math.floor(bestFile.mtime / 1000), episode: bestFile.ep };
  return null;
}

function HomePage() {
  const navigate = useNavigate();
  const { pickTitle } = useTitleLanguage();
  const [items, setItems] = useState<LibraryItem[]>([]);
  const [initialLoading, setInitialLoading] = useState(true);
  const [airingPage, setAiringPage] = useState(0);

  // Reloads triggered by metadata pings update items in place — no
  // setLoading(true), so the page doesn't flash through a "Reading
  // folders…" state on every poster match. Only the first mount shows it.
  // Diff-merge by id so React keeps stable refs for cards that didn't
  // change — prevents poster <img> elements from being unnecessarily
  // re-evaluated on every ping during the match burst at startup.
  const reload = useCallback(async () => {
    try {
      const data = await window.electronAPI.libraryWalk();
      const fresh = Array.isArray(data) ? data : [];
      setItems((prev) => {
        const prevById = new Map(prev.map((p) => [p.id, p]));
        return fresh.map((next) => {
          const old = prevById.get(next.id);
          if (!old) return next;
          // Cheap equality on the few fields the card actually renders.
          if (
            old.posterLocal === next.posterLocal &&
            old.poster === next.poster &&
            old.folderName === next.folderName &&
            old.matchedTitle === next.matchedTitle &&
            old.status === next.status &&
            old.files.length === next.files.length
          ) {
            return old;
          }
          return next;
        });
      });
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

  useEffect(() => {
    const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
      void reload();
    });
    return () => unsubscribe?.();
  }, [reload]);

  // Currently-airing shows the user has on disk, sorted by latest aired
  // (or downloaded) episode. Empty array if nothing is airing yet.
  const airing = useMemo(() => {
    const out: Array<{ item: LibraryItem; when: number; episode: number | null }> = [];
    for (const item of items) {
      if (item.files.length === 0) continue;
      if (normalizeStatus(item.status) !== "releasing") continue;
      const info = getAiringSortInfo(item);
      if (!info) continue;
      out.push({ item, when: info.when, episode: info.episode });
    }
    out.sort((a, b) => b.when - a.when);
    return out;
  }, [items]);

  const airingTotalPages = Math.max(1, Math.ceil(airing.length / AIRING_PAGE_SIZE));
  // Reset page if the airing list shrinks below the current page.
  useEffect(() => {
    if (airingPage >= airingTotalPages) setAiringPage(0);
  }, [airingPage, airingTotalPages]);

  const airingPageItems = airing.slice(
    airingPage * AIRING_PAGE_SIZE,
    (airingPage + 1) * AIRING_PAGE_SIZE,
  );

  if (initialLoading) {
    return (
      <div className="page">
        <div className="loading">Reading folders…</div>
      </div>
    );
  }

  const renderCard = (item: LibraryItem, sub?: { episode: number | null; when: number }) => {
    const posterUrl = item.posterLocal
      ? `media://${encodeURIComponent(item.posterLocal)}`
      : item.poster;
    const displayTitle = pickTitle({
      titleRomaji: item.titleRomaji ?? item.matchedTitle,
      titleEnglish: item.titleEnglish,
      folderName: item.folderName,
    });
    const epNum = sub?.episode != null ? String(sub.episode).padStart(2, "0") : null;
    const ago = sub ? fmtRelativeTime(sub.when) : null;
    const fileCount = `${item.files.length} file${item.files.length === 1 ? "" : "s"}`;
    return (
      <button
        key={item.id}
        type="button"
        className="show-card"
        onClick={() => navigate(`/series/${encodeURIComponent(item.id)}`)}
      >
        <div className="show-card-poster-wrap">
          {epNum && (
            <span className="show-card-ep-badge" aria-label={`Episode ${epNum}`}>
              EP {epNum}
            </span>
          )}
          {posterUrl ? (
            <img className="show-card-poster" src={posterUrl} alt={displayTitle} loading="lazy" />
          ) : (
            <div className="show-card-no-image"><Tv size={32} /></div>
          )}
        </div>
        <div className="show-card-info">
          <div className="show-card-title" title={item.folderName}>
            {displayTitle}
          </div>
          <div className="show-card-meta">
            {ago ? <span>{ago}</span> : <span>{fileCount}</span>}
          </div>
        </div>
      </button>
    );
  };

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Library</h1>
          <p className="page-sub">
            {items.length === 0
              ? "Your scanned folders are empty."
              : `${items.length} folder${items.length === 1 ? "" : "s"}.`}
          </p>
        </div>
      </div>

      {airing.length > 0 && (
        <section className="airing-section">
          <div className="airing-head">
            <h2 className="section-h2">Airing</h2>
            <div className="airing-pager">
              <button
                type="button"
                className="airing-pager-btn"
                onClick={() => setAiringPage((p) => Math.max(0, p - 1))}
                disabled={airingPage === 0}
                aria-label="Previous page"
              >
                <ChevronLeft size={16} />
              </button>
              <span className="airing-pager-count">
                {airingPage + 1} / {airingTotalPages}
              </span>
              <button
                type="button"
                className="airing-pager-btn"
                onClick={() => setAiringPage((p) => Math.min(airingTotalPages - 1, p + 1))}
                disabled={airingPage >= airingTotalPages - 1}
                aria-label="Next page"
              >
                <ChevronRight size={16} />
              </button>
            </div>
          </div>
          <div className="airing-grid">
            {airingPageItems.map(({ item, when, episode }) =>
              renderCard(item, { episode, when }),
            )}
          </div>
        </section>
      )}

      {items.length === 0 ? (
        <div className="empty">
          <div className="empty-icon"><Tv size={48} /></div>
          <div className="empty-title">Your library is empty</div>
          <div className="empty-text">
            Go to <strong>Settings</strong> to add a folder.
          </div>
        </div>
      ) : (
        <>
          <div className="section-head">
            <h2 className="section-h2">All</h2>
            <span className="section-count">
              {items.length} {items.length === 1 ? "folder" : "folders"}
            </span>
          </div>
          <div className="show-grid">
            {items.map((item) => renderCard(item))}
          </div>
        </>
      )}
    </div>
  );
}

export default HomePage;
