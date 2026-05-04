import { useMemo, useState, useEffect } from "react";
import { useMetadata, type SeriesMetadata } from "../hooks/useMetadata";
import ShowCard from "../components/ShowCard";
import SearchBar from "../components/SearchBar";
import LatestUpdatesCarousel from "../components/LatestUpdatesCarousel";
import Fuse from "fuse.js";
import { Search, Tv } from "lucide-react";

interface ShowWithId extends SeriesMetadata {
  seriesId: string;
}

function HomePage() {
  const { metadata, loading, error, loadMetadata } = useMetadata();
  const [searchQuery, setSearchQuery] = useState("");

  useEffect(() => {
    const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
      void loadMetadata();
    });
    return () => unsubscribe?.();
  }, [loadMetadata]);

  const allItems = useMemo(() => {
    const items: ShowWithId[] = [];
    Object.entries(metadata).forEach(([seriesId, seriesData]) => {
      const fileEpisodes = seriesData.fileEpisodes || [];
      if (fileEpisodes.length > 0) {
        items.push({ ...seriesData, seriesId });
      }
    });
    return items;
  }, [metadata]);

  const fuse = useMemo(() => {
    try {
      if (!Fuse || typeof Fuse !== "function") return null;
      return new Fuse(allItems, {
        keys: [
          { name: "title", weight: 0.4 },
          { name: "titleRomaji", weight: 0.3 },
          { name: "titleEnglish", weight: 0.3 },
          { name: "titleNative", weight: 0.2 },
          { name: "genres", weight: 0.1 },
          { name: "description", weight: 0.05 },
        ],
        threshold: 0.4,
        includeScore: true,
        minMatchCharLength: 1,
      });
    } catch (error) {
      console.error("Fuse.js initialization failed:", error);
      return null;
    }
  }, [allItems]);

  const filteredItems = useMemo(() => {
    if (!searchQuery.trim()) return allItems;
    if (!fuse) {
      const queryLower = searchQuery.toLowerCase();
      return allItems.filter((item) => {
        const title = (item.title || "").toLowerCase();
        const titleRomaji = (item.titleRomaji || "").toLowerCase();
        const titleEnglish = (item.titleEnglish || "").toLowerCase();
        const titleNative = (item.titleNative || "").toLowerCase();
        return (
          title.includes(queryLower) ||
          titleRomaji.includes(queryLower) ||
          titleEnglish.includes(queryLower) ||
          titleNative.includes(queryLower) ||
          item.genres?.some((g) => g.toLowerCase().includes(queryLower))
        );
      });
    }
    try {
      const results = fuse.search(searchQuery);
      return results.map((result: { item: ShowWithId }) => result.item);
    } catch (error) {
      console.error("Fuse.js search failed:", error);
      return allItems;
    }
  }, [searchQuery, fuse, allItems]);

  if (loading) {
    return (
      <div className="page">
        <div className="loading">Loading your library…</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="page">
        <div className="error">Error: {error}</div>
      </div>
    );
  }

  const series: ShowWithId[] = [];
  const movies: ShowWithId[] = [];
  filteredItems.forEach((item: ShowWithId) => {
    const isMovie =
      item.type === "movie" ||
      (item.fileEpisodes?.length === 1 && !item.totalEpisodes) ||
      item.format === "MOVIE";
    if (isMovie) movies.push(item);
    else series.push(item);
  });

  const totalShows = allItems.length;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Library</h1>
          <p className="page-sub">
            {totalShows === 0
              ? "Your scanned folders are empty."
              : `${totalShows} ${totalShows === 1 ? "title" : "titles"} across your scanned folders.`}
          </p>
        </div>
      </div>

      <SearchBar onSearch={setSearchQuery} placeholder="Search titles, genres, studios…" />

      {!searchQuery.trim() && <LatestUpdatesCarousel metadata={metadata} />}

      {totalShows === 0 ? (
        <div className="empty">
          <div className="empty-icon"><Tv size={48} /></div>
          <div className="empty-title">Your library is empty</div>
          <div className="empty-text">
            Go to <strong>Settings</strong> to select a folder with your anime collection.
          </div>
        </div>
      ) : series.length === 0 && movies.length === 0 ? (
        <div className="empty">
          <div className="empty-icon"><Search size={48} /></div>
          <div className="empty-title">No matches</div>
          <div className="empty-text">Nothing in your library matches "{searchQuery}".</div>
        </div>
      ) : (
        <>
          {series.length > 0 && (
            <>
              <div className="section-head">
                <h2 className="section-h2">Series</h2>
                <span className="section-count">
                  {series.length} {series.length === 1 ? "title" : "titles"}
                </span>
              </div>
              <div className="show-grid">
                {series.map((show) => (
                  <ShowCard
                    key={show.seriesId}
                    seriesId={show.seriesId}
                    seriesData={show}
                  />
                ))}
              </div>
            </>
          )}

          {movies.length > 0 && (
            <>
              <div className="section-head">
                <h2 className="section-h2">Movies</h2>
                <span className="section-count">
                  {movies.length} {movies.length === 1 ? "title" : "titles"}
                </span>
              </div>
              <div className="show-grid">
                {movies.map((show) => (
                  <ShowCard
                    key={show.seriesId}
                    seriesId={show.seriesId}
                    seriesData={show}
                  />
                ))}
              </div>
            </>
          )}
        </>
      )}
    </div>
  );
}

export default HomePage;
