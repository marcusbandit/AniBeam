import { useEffect, useState, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Play } from "lucide-react";
import type { LibraryItem } from "../../types/electron";

function SeriesDetailPage() {
  const { seriesId } = useParams<{ seriesId: string }>();
  const navigate = useNavigate();
  const [item, setItem] = useState<LibraryItem | null>(null);
  const [initialLoading, setInitialLoading] = useState(true);

  const decodedId = seriesId ? decodeURIComponent(seriesId) : "";

  // No setLoading toggle on reload pings — keeps the file list visible
  // while the background match updates posters/dates incrementally.
  const reload = useCallback(async () => {
    try {
      const all = await window.electronAPI.libraryWalk();
      const found = (Array.isArray(all) ? all : []).find((i) => i.id === decodedId) ?? null;
      setItem(found);
    } catch (err) {
      console.error("library:walk failed", err);
      setItem(null);
    } finally {
      setInitialLoading(false);
    }
  }, [decodedId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  useEffect(() => {
    const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
      void reload();
    });
    return () => unsubscribe?.();
  }, [reload]);

  if (initialLoading) {
    return (
      <div className="page">
        <div className="loading">Reading folder…</div>
      </div>
    );
  }

  if (!item) {
    return (
      <div className="page">
        <div className="error">Folder not found.</div>
        <button className="btn btn-secondary" onClick={() => navigate("/")}>
          <ArrowLeft size={14} /> Library
        </button>
      </div>
    );
  }

  // Stable order: by season then episode number, falling back to filename.
  const sorted = [...item.files].sort((a, b) => {
    const sa = a.seasonNumber ?? 0;
    const sb = b.seasonNumber ?? 0;
    if (sa !== sb) return sa - sb;
    if (a.episodeNumber !== b.episodeNumber) return a.episodeNumber - b.episodeNumber;
    return a.filename.localeCompare(b.filename);
  });

  return (
    <div className="page series-detail-bare">
      <button className="detail-back" onClick={() => navigate("/")}>
        <ArrowLeft size={14} />
        <span>Library</span>
      </button>

      <h1 className="page-title" style={{ marginTop: 8 }}>{item.folderName}</h1>
      <p className="page-sub">
        {sorted.length} file{sorted.length === 1 ? "" : "s"} · {item.folderPath}
      </p>

      <div className="bare-episode-list">
        {sorted.map((f) => (
          <button
            key={f.filePath}
            type="button"
            className="bare-episode-row"
            onClick={() =>
              navigate(`/player/${encodeURIComponent(item.id)}/${f.episodeNumber}`)
            }
          >
            <span className="bare-episode-icon"><Play size={14} /></span>
            <span className="bare-episode-title">{f.title}</span>
          </button>
        ))}
      </div>
    </div>
  );
}

export default SeriesDetailPage;
