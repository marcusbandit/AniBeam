import { useState, useMemo } from 'react';
import { useMetadata, type SeriesMetadata } from '../hooks/useMetadata';
import { BookOpen, Tv, Film, Search, RefreshCw, Trash2, MoreHorizontal } from 'lucide-react';

type FilterOption = 'all' | 'series' | 'movies' | 'missing';

function isMovie(data: SeriesMetadata): boolean {
  return data.type === 'movie' || data.format === 'MOVIE';
}

function getImageUrl(localPath?: string | null, remotePath?: string | null): string | null {
  if (localPath) return `media://${encodeURIComponent(localPath)}`;
  return remotePath || null;
}

function MetadataTab() {
  const { metadata, loading, updateSeriesMetadata, loadMetadata } = useMetadata();
  const [refreshing, setRefreshing] = useState<Record<string, boolean>>({});
  const [searchQuery, setSearchQuery] = useState('');
  const [filter, setFilter] = useState<FilterOption>('all');
  const [bulkRefreshing, setBulkRefreshing] = useState(false);

  const seriesList = useMemo(() => Object.entries(metadata), [metadata]);

  const filterCounts = useMemo(() => ({
    all: seriesList.length,
    series: seriesList.filter(([, d]) => !isMovie(d)).length,
    movies: seriesList.filter(([, d]) => isMovie(d)).length,
    missing: seriesList.filter(([, d]) => !(d.fileEpisodes?.length)).length,
  }), [seriesList]);

  const filteredSeries = useMemo(() => {
    return seriesList.filter(([id, data]) => {
      if (filter === 'series' && isMovie(data)) return false;
      if (filter === 'movies' && !isMovie(data)) return false;
      if (filter === 'missing' && (data.fileEpisodes?.length ?? 0) > 0) return false;

      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        const hay = `${data.title || id} ${data.titleRomaji || ''} ${data.titleEnglish || ''}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    });
  }, [seriesList, searchQuery, filter]);

  const handleRefresh = async (seriesId: string, seriesName: string) => {
    setRefreshing(prev => ({ ...prev, [seriesId]: true }));
    try {
      const fetchedMetadata = await window.electronAPI.fetchMetadata(seriesName) as SeriesMetadata | null;
      if (fetchedMetadata) {
        await updateSeriesMetadata(seriesId, fetchedMetadata);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      alert('Error refreshing metadata: ' + errorMessage);
    } finally {
      setRefreshing(prev => ({ ...prev, [seriesId]: false }));
    }
  };

  const handleBulkRefresh = async () => {
    if (!confirm(`Refresh metadata for all ${filteredSeries.length} items? This may take a while.`)) return;

    setBulkRefreshing(true);
    let successCount = 0;
    let errorCount = 0;

    for (const [seriesId, data] of filteredSeries) {
      try {
        setRefreshing(prev => ({ ...prev, [seriesId]: true }));
        const fetchedMetadata = await window.electronAPI.fetchMetadata(data.title || seriesId) as SeriesMetadata | null;
        if (fetchedMetadata) {
          await updateSeriesMetadata(seriesId, fetchedMetadata);
          successCount++;
        }
      } catch (err) {
        console.error('Error refreshing:', seriesId, err);
        errorCount++;
      } finally {
        setRefreshing(prev => ({ ...prev, [seriesId]: false }));
      }
      await new Promise(resolve => setTimeout(resolve, 500));
    }

    setBulkRefreshing(false);
    alert(`Refresh complete\nSuccessful: ${successCount}\nFailed: ${errorCount}`);
    await loadMetadata();
  };

  const handleDelete = async (seriesId: string, seriesName: string) => {
    if (!confirm(`Delete "${seriesName}"?\n\nThis will remove all cached images and metadata for this series.`)) return;
    try {
      await window.electronAPI.deleteSeries(seriesId);
      await loadMetadata();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      alert('Error deleting series: ' + errorMessage);
    }
  };

  if (loading) {
    return (
      <div className="page">
        <div className="loading">Loading metadata…</div>
      </div>
    );
  }

  if (seriesList.length === 0) {
    return (
      <div className="page">
        <div className="empty">
          <div className="empty-icon"><BookOpen size={48} /></div>
          <div className="empty-title">No metadata yet</div>
          <div className="empty-text">
            Your library is empty. Add a folder in <strong>Settings</strong> and scan it to get started.
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="page metadata-page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Metadata</h1>
          <p className="page-sub">All matched titles in your library, the source they came from, and the files on disk.</p>
        </div>
        <button
          className="btn btn-secondary"
          onClick={handleBulkRefresh}
          disabled={bulkRefreshing || filteredSeries.length === 0}
        >
          <RefreshCw size={14} className={bulkRefreshing ? 'spin' : ''} />
          <span>{bulkRefreshing ? 'Refreshing…' : 'Refresh all'}</span>
        </button>
      </div>

      <div className="meta-toolbar">
        <div className="filter-pills">
          {([
            { id: 'all', label: 'All' },
            { id: 'series', label: 'Series' },
            { id: 'movies', label: 'Movies' },
            { id: 'missing', label: 'Missing files' },
          ] as Array<{ id: FilterOption; label: string }>).map((f) => (
            <button
              key={f.id}
              className={`filter-pill${filter === f.id ? ' on' : ''}`}
              onClick={() => setFilter(f.id)}
            >
              <span>{f.label}</span>
              <span className="filter-count">{filterCounts[f.id]}</span>
            </button>
          ))}
        </div>
        <div className="meta-search">
          <Search size={14} />
          <input
            placeholder="Filter titles…"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
      </div>

      {filteredSeries.length === 0 ? (
        <div className="empty">
          <div className="empty-icon"><Search size={48} /></div>
          <div className="empty-title">No matches</div>
          <div className="empty-text">No series match your filters.</div>
        </div>
      ) : (
        <div className="meta-table">
          <div className="meta-row meta-row-head">
            <div className="col-thumb"></div>
            <div className="col-title">Title</div>
            <div className="col-type">Type</div>
            <div className="col-source">Source</div>
            <div className="col-files">Files</div>
            <div className="col-updated">Updated</div>
            <div className="col-actions"></div>
          </div>
          {filteredSeries.map(([seriesId, data]) => {
            const movie = isMovie(data);
            const have = data.fileEpisodes?.length || 0;
            const total = data.totalEpisodes || data.episodes?.length || (movie ? 1 : 0);
            const pct = total ? Math.round((have / total) * 100) : 0;
            const sourceClass = data.source ? data.source.toLowerCase() : 'none';
            const posterUrl = getImageUrl(data.posterLocal, data.poster);
            const isRefreshing = refreshing[seriesId];

            return (
              <div key={seriesId} className="meta-row">
                <div className="col-thumb">
                  <div className="meta-thumb">
                    {posterUrl ? (
                      <img
                        src={posterUrl}
                        alt={data.title || seriesId}
                        onError={(e) => {
                          const t = e.target as HTMLImageElement;
                          t.style.display = 'none';
                        }}
                      />
                    ) : (
                      movie ? <Film size={16} /> : <Tv size={16} />
                    )}
                  </div>
                </div>
                <div className="col-title">
                  <div className="meta-title-main">{data.title || seriesId}</div>
                  {data.titleRomaji && data.titleRomaji !== data.title && (
                    <div className="meta-title-alt">{data.titleRomaji}</div>
                  )}
                </div>
                <div className="col-type">
                  <span className="type-tag">{movie ? 'Movie' : 'Series'}</span>
                </div>
                <div className="col-source">
                  <span className={`source-pill source-${sourceClass}`}>
                    {data.source || '—'}
                  </span>
                </div>
                <div className="col-files">
                  <div className="files-bar">
                    <div className="files-bar-track">
                      <div className="files-bar-fill" style={{ width: `${pct}%` }} />
                    </div>
                    <span className="files-bar-text">
                      {have}<span className="muted">/{total}</span>
                    </span>
                  </div>
                </div>
                <div className="col-updated muted">—</div>
                <div className="col-actions">
                  <button
                    className="icon-btn"
                    title="Refresh"
                    onClick={() => handleRefresh(seriesId, data.title || seriesId)}
                    disabled={isRefreshing || bulkRefreshing}
                  >
                    <RefreshCw size={14} className={isRefreshing ? 'spin' : ''} />
                  </button>
                  <button
                    className="icon-btn icon-btn-danger"
                    title="Delete"
                    onClick={() => handleDelete(seriesId, data.title || seriesId)}
                    disabled={isRefreshing || bulkRefreshing}
                  >
                    <Trash2 size={14} />
                  </button>
                  <button
                    className="icon-btn"
                    title="More"
                    disabled
                  >
                    <MoreHorizontal size={14} />
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

export default MetadataTab;
